use std::{
  future::Future,
  io,
  net::SocketAddr,
  pin::Pin,
  sync::Arc,
  task::{Context, Poll},
  time::Duration,
};

use atomic::{Atomic, Ordering};
use futures_util::{AsyncReadExt, AsyncWriteExt, FutureExt};
use smol::net::{TcpListener, TcpStream, UdpSocket};
#[cfg(feature = "compat")]
use tokio_util::compat::FuturesAsyncWriteCompatExt;

use crate::{
  net::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs},
  Runtime,
};

use super::SmolRuntime;

pub use super::quinn_::SmolRuntime as SmolQuinnRuntime;

#[derive(Debug, Default, Clone, Copy)]
pub struct SmolNet;

impl Net for SmolNet {
  type TcpListener = SmolTcpListener;

  type TcpStream = SmolTcpStream;

  type UdpSocket = SmolUdpSocket;

  #[cfg(feature = "quinn")]
  type Quinn = SmolQuinnRuntime;
}

pub struct SmolTcpListener {
  ln: TcpListener,
  write_timeout: Atomic<Option<Duration>>,
  read_timeout: Atomic<Option<Duration>>,
}

impl crate::net::TcpListener for SmolTcpListener {
  type Stream = SmolTcpStream;
  type Runtime = SmolRuntime;

  fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized,
  {
    async move {
      let mut addrs = addr.to_socket_addrs().await?;

      let res = if addrs.size_hint().0 <= 1 {
        if let Some(addr) = addrs.next() {
          TcpListener::bind(addr).await
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        TcpListener::bind(addrs.collect::<Vec<_>>().as_slice()).await
      };

      res.map(|ln| Self {
        ln,
        write_timeout: Atomic::new(None),
        read_timeout: Atomic::new(None),
      })
    }
  }

  fn accept(&self) -> impl Future<Output = io::Result<(Self::Stream, SocketAddr)>> + Send + '_ {
    async move {
      self.ln.accept().await.map(|(stream, addr)| {
        (
          SmolTcpStream {
            stream,
            write_timeout: Atomic::new(self.write_timeout.load(Ordering::SeqCst)),
            read_timeout: Atomic::new(self.read_timeout.load(Ordering::SeqCst)),
          },
          addr,
        )
      })
    }
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::SeqCst);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::SeqCst)
  }

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::SeqCst);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::SeqCst)
  }
}

pub struct SmolTcpStream {
  stream: TcpStream,
  write_timeout: Atomic<Option<Duration>>,
  read_timeout: Atomic<Option<Duration>>,
}

impl futures_util::AsyncRead for SmolTcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    if let Some(d) = self.read_timeout.load(Ordering::Relaxed) {
      if !d.is_zero() {
        let timeout = SmolRuntime::timeout(d, self.stream.read(buf));
        futures_util::pin_mut!(timeout);
        match timeout.poll(cx) {
          Poll::Ready(rst) => match rst {
            Ok(rst) => return Poll::Ready(rst),
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, e))),
          },
          Poll::Pending => return Poll::Pending,
        }
      }
    }

    Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for SmolTcpStream {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
    if let Some(d) = self.write_timeout.load(Ordering::Relaxed) {
      if !d.is_zero() {
        let timeout = SmolRuntime::timeout(d, self.stream.write(buf));
        futures_util::pin_mut!(timeout);
        match timeout.poll(cx) {
          Poll::Ready(rst) => match rst {
            Ok(rst) => return Poll::Ready(rst),
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, e))),
          },
          Poll::Pending => return Poll::Pending,
        }
      }
    }

    Pin::new(&mut (&mut self.stream)).poll_write(cx, buf)
  }

  fn poll_flush(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream)).poll_flush(cx)
  }

  fn poll_close(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream)).poll_close(cx)
  }
}

#[cfg(feature = "tokio-compat")]
impl tokio::io::AsyncRead for SmolTcpStream {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(
      self.get_mut(),
    ))
    .poll_read(cx, buf)
  }
}

#[cfg(feature = "tokio-compat")]
impl tokio::io::AsyncWrite for SmolTcpStream {
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<Result<usize, io::Error>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_flush(cx)
  }

  fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_shutdown(cx)
  }
}

/// Error indicating that two halves were not from the same socket, and thus could
/// not be reunited.
#[derive(Debug)]
pub struct ReuniteError(
  pub SmolTcpStreamOwnedReadHalf,
  pub SmolTcpStreamOwnedWriteHalf,
);

impl core::fmt::Display for ReuniteError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(
      f,
      "tried to reunite halves that are not from the same socket"
    )
  }
}

impl std::error::Error for ReuniteError {}

#[derive(Debug)]
pub struct SmolTcpStreamOwnedReadHalf {
  stream: TcpStream,
  read_timeout: Atomic<Option<Duration>>,
  id: usize,
}

#[derive(Debug)]
pub struct SmolTcpStreamOwnedWriteHalf {
  stream: TcpStream,
  write_timeout: Atomic<Option<Duration>>,
  shutdown_on_drop: bool,
  id: usize,
}

impl SmolTcpStreamOwnedWriteHalf {
  pub fn forget(mut self) {
    self.shutdown_on_drop = false;
    drop(self);
  }
}

impl Drop for SmolTcpStreamOwnedWriteHalf {
  fn drop(&mut self) {
    if self.shutdown_on_drop {
      let _ = self.stream.shutdown(std::net::Shutdown::Write);
    }
  }
}

impl futures_util::AsyncRead for SmolTcpStreamOwnedReadHalf {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    if let Some(d) = self.read_timeout.load(Ordering::Relaxed) {
      if !d.is_zero() {
        let timeout = SmolRuntime::timeout(d, self.stream.read(buf));
        futures_util::pin_mut!(timeout);
        match timeout.poll(cx) {
          Poll::Ready(rst) => match rst {
            Ok(rst) => return Poll::Ready(rst),
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, e))),
          },
          Poll::Pending => return Poll::Pending,
        }
      }
    }

    Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for SmolTcpStreamOwnedWriteHalf {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
    if let Some(d) = self.write_timeout.load(Ordering::Relaxed) {
      if !d.is_zero() {
        let timeout = SmolRuntime::timeout(d, self.stream.write(buf));
        futures_util::pin_mut!(timeout);
        match timeout.poll(cx) {
          Poll::Ready(rst) => match rst {
            Ok(rst) => return Poll::Ready(rst),
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, e))),
          },
          Poll::Pending => return Poll::Pending,
        }
      }
    }

    Pin::new(&mut (&mut self.stream)).poll_write(cx, buf)
  }

  fn poll_flush(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream)).poll_flush(cx)
  }

  fn poll_close(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream)).poll_close(cx)
  }
}

#[cfg(feature = "tokio-compat")]
impl tokio::io::AsyncRead for SmolTcpStreamOwnedReadHalf {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(
      self.get_mut(),
    ))
    .poll_read(cx, buf)
  }
}

#[cfg(feature = "tokio-compat")]
impl tokio::io::AsyncWrite for SmolTcpStreamOwnedWriteHalf {
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<Result<usize, io::Error>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_flush(cx)
  }

  fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_shutdown(cx)
  }
}

impl TcpStreamOwnedReadHalf for SmolTcpStreamOwnedReadHalf {
  type Runtime = SmolRuntime;

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::Release);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::Acquire)
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }
}

impl TcpStreamOwnedWriteHalf for SmolTcpStreamOwnedWriteHalf {
  type Runtime = SmolRuntime;

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::Release);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::Acquire)
  }

  fn forget(mut self) {
    self.shutdown_on_drop = false;
    drop(self);
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }
}

impl crate::net::TcpStream for SmolTcpStream {
  type Runtime = SmolRuntime;
  type OwnedReadHalf = SmolTcpStreamOwnedReadHalf;
  type OwnedWriteHalf = SmolTcpStreamOwnedWriteHalf;
  type ReuniteError = ReuniteError;

  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized,
  {
    async move {
      let mut addrs = addr.to_socket_addrs().await?;

      let res = if addrs.size_hint().0 <= 1 {
        if let Some(addr) = addrs.next() {
          TcpStream::connect(addr).await
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        TcpStream::connect(&addrs.collect::<Vec<_>>().as_slice()).await
      };

      res.map(|stream| Self {
        stream,
        write_timeout: Atomic::new(None),
        read_timeout: Atomic::new(None),
      })
    }
  }

  fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized,
  {
    async move {
      SmolRuntime::timeout(timeout, Self::connect(addr))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
        .and_then(|res| res)
    }
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.stream.set_ttl(ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    self.stream.ttl()
  }

  fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
    self.stream.set_nodelay(nodelay)
  }

  fn nodelay(&self) -> io::Result<bool> {
    self.stream.nodelay()
  }

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::SeqCst);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::SeqCst)
  }

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::SeqCst);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::SeqCst)
  }

  fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
    let id = &self.stream as *const _ as usize;
    (
      SmolTcpStreamOwnedReadHalf {
        stream: self.stream.clone(),
        read_timeout: self.read_timeout,
        id,
      },
      SmolTcpStreamOwnedWriteHalf {
        stream: self.stream,
        write_timeout: self.write_timeout,
        shutdown_on_drop: true,
        id,
      },
    )
  }

  fn reunite(
    read: Self::OwnedReadHalf,
    mut write: Self::OwnedWriteHalf,
  ) -> Result<Self, Self::ReuniteError>
  where
    Self: Sized,
  {
    if read.id == write.id {
      write.shutdown_on_drop = false;
      Ok(Self {
        stream: read.stream,
        write_timeout: Atomic::new(write.write_timeout.load(Ordering::Relaxed)),
        read_timeout: read.read_timeout,
      })
    } else {
      Err(ReuniteError(read, write))
    }
  }
}

pub struct SmolUdpSocket {
  socket: UdpSocket,
  write_timeout: Atomic<Option<Duration>>,
  read_timeout: Atomic<Option<Duration>>,
}

impl crate::net::UdpSocket for SmolUdpSocket {
  type Runtime = SmolRuntime;

  fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized,
  {
    async move {
      let mut addrs = addr.to_socket_addrs().await?;

      let res = if addrs.size_hint().0 <= 1 {
        if let Some(addr) = addrs.next() {
          UdpSocket::bind(addr).await
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        UdpSocket::bind(&addrs.collect::<Vec<_>>().as_slice()).await
      };
      res.map(|socket| Self {
        socket,
        write_timeout: Atomic::new(None),
        read_timeout: Atomic::new(None),
      })
    }
  }

  fn bind_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized,
  {
    async move {
      SmolRuntime::timeout(timeout, Self::bind(addr))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
        .and_then(|res| res)
    }
  }

  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
  ) -> impl Future<Output = io::Result<()>> + Send {
    async move {
      let mut addrs = addr.to_socket_addrs().await?;

      if addrs.size_hint().0 <= 1 {
        if let Some(addr) = addrs.next() {
          self.socket.connect(addr).await
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        self
          .socket
          .connect(&addrs.collect::<Vec<_>>().as_slice())
          .await
      }
    }
  }

  fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<()>> + Send {
    async move {
      SmolRuntime::timeout(timeout, self.connect(addr))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
        .and_then(|res| res)
    }
  }

  fn recv(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send {
    async move {
      if let Some(timeout) = self.read_timeout.load(Ordering::Relaxed) {
        if !timeout.is_zero() {
          return match SmolRuntime::timeout(timeout, self.socket.recv(buf)).await {
            Ok(timeout) => timeout,
            Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
          };
        }
      }
      self.socket.recv(buf).await
    }
  }

  fn recv_from(
    &self,
    buf: &mut [u8],
  ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send {
    async move {
      if let Some(timeout) = self.read_timeout.load(Ordering::Relaxed) {
        if !timeout.is_zero() {
          return match SmolRuntime::timeout(timeout, self.socket.recv_from(buf)).await {
            Ok(timeout) => timeout,
            Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
          };
        }
      }
      self.socket.recv_from(buf).await
    }
  }

  fn send(&self, buf: &[u8]) -> impl Future<Output = io::Result<usize>> + Send {
    async move {
      if let Some(timeout) = self.write_timeout.load(Ordering::Relaxed) {
        if !timeout.is_zero() {
          return match SmolRuntime::timeout(timeout, self.socket.send(buf)).await {
            Ok(timeout) => timeout,
            Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
          };
        }
      }
      self.socket.send(buf).await
    }
  }

  fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> impl Future<Output = io::Result<usize>> + Send {
    async move {
      let mut addrs = target.to_socket_addrs().await?;
      if addrs.size_hint().0 <= 1 {
        if let Some(addr) = addrs.next() {
          if let Some(timeout) = self.write_timeout.load(Ordering::Relaxed) {
            if !timeout.is_zero() {
              return match SmolRuntime::timeout(timeout, self.socket.send_to(buf, addr)).await {
                Ok(timeout) => timeout,
                Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
              };
            }
          }
          self.socket.send_to(buf, addr).await
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        let addrs = addrs.collect::<Vec<_>>();
        if let Some(timeout) = self.write_timeout.load(Ordering::Relaxed) {
          if !timeout.is_zero() {
            return match SmolRuntime::timeout(timeout, self.socket.send_to(buf, addrs.as_slice()))
              .await
            {
              Ok(timeout) => timeout,
              Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
            };
          }
        }
        self.socket.send_to(buf, addrs.as_slice()).await
      }
    }
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.socket.set_ttl(ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    self.socket.ttl()
  }

  fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
    self.socket.set_broadcast(broadcast)
  }

  fn broadcast(&self) -> io::Result<bool> {
    self.socket.broadcast()
  }

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::SeqCst);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::SeqCst)
  }

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::SeqCst);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::SeqCst)
  }

  fn set_read_buffer(&self, size: usize) -> io::Result<()> {
    #[cfg(not(any(unix, windows)))]
    {
      panic!("unsupported platform");
    }

    #[cfg(all(unix, feature = "socket2"))]
    {
      use std::os::fd::AsRawFd;
      return crate::net::set_read_buffer(self.socket.as_raw_fd(), size);
    }

    #[cfg(all(windows, feature = "socket2"))]
    {
      use std::os::windows::io::AsRawSocket;
      return crate::net::set_read_buffer(self.socket.as_raw_socket(), size);
    }

    let _ = size;
    Ok(())
  }

  fn set_write_buffer(&self, size: usize) -> io::Result<()> {
    #[cfg(not(any(unix, windows)))]
    {
      panic!("unsupported platform");
    }

    #[cfg(all(unix, feature = "socket2"))]
    {
      use std::os::fd::AsRawFd;
      return crate::net::set_write_buffer(self.socket.as_raw_fd(), size);
    }

    #[cfg(all(windows, feature = "socket2"))]
    {
      use std::os::windows::io::AsRawSocket;
      return crate::net::set_write_buffer(self.socket.as_raw_socket(), size);
    }
    let _ = size;
    Ok(())
  }

  fn poll_recv_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>> {
    let fut = self.socket.recv_from(buf);
    futures_util::pin_mut!(fut);
    fut.poll_unpin(cx)
  }

  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>> {
    let fut = self.socket.send_to(buf, target);
    futures_util::pin_mut!(fut);
    fut.poll_unpin(cx)
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.socket.local_addr()
  }
}
