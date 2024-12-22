use std::{
  io,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll},
};

use futures_util::FutureExt;
use smol::net::{TcpListener, TcpStream, UdpSocket};

use crate::net::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs};

use super::SmolRuntime;

#[cfg(feature = "quinn")]
pub use quinn_::SmolQuinnRuntime;

#[cfg(feature = "quinn")]
mod quinn_ {
  use quinn::{Runtime, SmolRuntime};

  /// A Quinn runtime for tokio
  #[derive(Debug)]
  #[repr(transparent)]
  pub struct SmolQuinnRuntime(SmolRuntime);

  impl Default for SmolQuinnRuntime {
    fn default() -> Self {
      Self(SmolRuntime)
    }
  }

  impl Runtime for SmolQuinnRuntime {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
      self.0.new_timer(i)
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>>) {
      self.0.spawn(future)
    }

    fn wrap_udp_socket(
      &self,
      t: std::net::UdpSocket,
    ) -> std::io::Result<std::sync::Arc<dyn quinn::AsyncUdpSocket>> {
      self.0.wrap_udp_socket(t)
    }
  }
}

/// The [`Net`] implementation for [`smol`](::smol) runtime
#[derive(Debug, Default, Clone, Copy)]
pub struct SmolNet;

impl Net for SmolNet {
  type TcpListener = SmolTcpListener;

  type TcpStream = SmolTcpStream;

  type UdpSocket = SmolUdpSocket;

  #[cfg(feature = "quinn")]
  type Quinn = SmolQuinnRuntime;
}

/// The [`TcpListener`](crate::net::TcpListener) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolTcpListener {
  ln: TcpListener,
}

impl crate::net::TcpListener for SmolTcpListener {
  type Stream = SmolTcpStream;
  type Runtime = SmolRuntime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
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

    res.map(|ln| Self { ln })
  }

  async fn accept(&self) -> io::Result<(Self::Stream, SocketAddr)> {
    self
      .ln
      .accept()
      .await
      .map(|(stream, addr)| (SmolTcpStream { stream }, addr))
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }
}

/// The [`TcpStream`](crate::net::TcpStream) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolTcpStream {
  stream: TcpStream,
}

impl futures_util::AsyncRead for SmolTcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for SmolTcpStream {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
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

impl core::error::Error for ReuniteError {}

/// The [`TcpStreamOwnedReadHalf`](crate::net::TcpStreamOwnedReadHalf) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
pub struct SmolTcpStreamOwnedReadHalf {
  stream: TcpStream,
  id: usize,
}

/// The [`TcpStreamOwnedWriteHalf`](crate::net::TcpStreamOwnedWriteHalf) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
pub struct SmolTcpStreamOwnedWriteHalf {
  stream: TcpStream,
  shutdown_on_drop: bool,
  id: usize,
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
    Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for SmolTcpStreamOwnedWriteHalf {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
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

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }
}

impl TcpStreamOwnedWriteHalf for SmolTcpStreamOwnedWriteHalf {
  type Runtime = SmolRuntime;

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

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
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

    res.map(|stream| Self { stream })
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

  fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
    let id = &self.stream as *const _ as usize;
    (
      SmolTcpStreamOwnedReadHalf {
        stream: self.stream.clone(),
        id,
      },
      SmolTcpStreamOwnedWriteHalf {
        stream: self.stream,
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
      })
    } else {
      Err(ReuniteError(read, write))
    }
  }

  fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
    #[cfg(unix)]
    {
      use std::os::fd::{AsRawFd, FromRawFd};
      unsafe { socket2::Socket::from_raw_fd(self.stream.as_raw_fd()) }.shutdown(how)
    }

    #[cfg(windows)]
    {
      use std::os::windows::io::{AsRawSocket, FromRawSocket};
      unsafe { socket2::Socket::from_raw_socket(self.stream.as_raw_socket()) }.shutdown(how)
    }

    #[cfg(not(any(unix, windows)))]
    {
      panic!("unsupported platform");
    }
  }
}

/// The [`UdpSocket`](crate::net::UdpSocket) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolUdpSocket {
  socket: UdpSocket,
}

impl crate::net::UdpSocket for SmolUdpSocket {
  type Runtime = SmolRuntime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
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
    res.map(|socket| Self { socket })
  }

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> io::Result<()> {
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

  async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
    self.socket.recv(buf).await
  }

  async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
    self.socket.recv_from(buf).await
  }

  async fn send(&self, buf: &[u8]) -> io::Result<usize> {
    self.socket.send(buf).await
  }

  async fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> io::Result<usize> {
    let mut addrs = target.to_socket_addrs().await?;
    if let Some(addr) = addrs.next() {
      self.socket.send_to(buf, addr).await
    } else {
      return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid socket address",
      ));
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

  fn set_read_buffer(&self, size: usize) -> io::Result<()> {
    #[cfg(not(any(unix, windows)))]
    {
      panic!("unsupported platform");
    }

    #[cfg(unix)]
    {
      use std::os::fd::AsRawFd;
      return crate::net::set_read_buffer(self.socket.as_raw_fd(), size);
    }

    #[cfg(windows)]
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

    #[cfg(unix)]
    {
      use std::os::fd::AsRawFd;
      return crate::net::set_write_buffer(self.socket.as_raw_fd(), size);
    }

    #[cfg(windows)]
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
