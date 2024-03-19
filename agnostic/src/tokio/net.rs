use std::{
  future::Future,
  io,
  net::SocketAddr,
  ops::Deref,
  pin::Pin,
  sync::{atomic::Ordering, Arc},
  task::{Context, Poll},
  time::Duration,
};

use tokio::{
  io::{AsyncReadExt, AsyncWriteExt, ReadBuf},
  net::{TcpListener, TcpStream, UdpSocket},
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{
  net::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs},
  Runtime,
};

use super::TokioRuntime;

#[cfg(feature = "quinn")]
pub use quinn_::TokioQuinnRuntime;

#[derive(Debug, Default, Clone, Copy)]
pub struct TokioNet;

impl Net for TokioNet {
  type TcpListener = TokioTcpListener;

  type TcpStream = TokioTcpStream;

  type UdpSocket = TokioUdpSocket;

  #[cfg(feature = "quinn")]
  type Quinn = TokioQuinnRuntime;
}

#[cfg(feature = "quinn")]
mod quinn_ {
  use quinn::{Runtime, TokioRuntime};

  #[derive(Debug)]
  #[repr(transparent)]
  pub struct TokioQuinnRuntime(TokioRuntime);

  impl Default for TokioQuinnRuntime {
    fn default() -> Self {
      Self(TokioRuntime)
    }
  }

  impl Runtime for TokioQuinnRuntime {
    fn new_timer(&self, i: std::time::Instant) -> std::pin::Pin<Box<dyn quinn::AsyncTimer>> {
      self.0.new_timer(i)
    }

    fn spawn(&self, future: std::pin::Pin<Box<dyn core::future::Future<Output = ()> + Send>>) {
      self.0.spawn(future)
    }

    fn wrap_udp_socket(
      &self,
      t: std::net::UdpSocket,
    ) -> std::io::Result<Box<dyn quinn::AsyncUdpSocket>> {
      self.0.wrap_udp_socket(t)
    }
  }
}

pub struct TokioTcpListener {
  ln: TcpListener,
}

impl crate::net::TcpListener for TokioTcpListener {
  type Stream = TokioTcpStream;
  type Runtime = TokioRuntime;

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

      res.map(|ln| Self { ln })
    }
  }

  fn accept(&self) -> impl Future<Output = io::Result<(Self::Stream, SocketAddr)>> + Send {
    async move {
      self
        .ln
        .accept()
        .await
        .map(|(stream, addr)| (TokioTcpStream { stream }, addr))
    }
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }
}

pub struct TokioTcpStream {
  stream: TcpStream,
}

impl futures_util::AsyncRead for TokioTcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat()).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for TokioTcpStream {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_write(cx, buf)
  }

  fn poll_flush(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_flush(cx)
  }

  fn poll_close(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_close(cx)
  }
}

impl tokio::io::AsyncRead for TokioTcpStream {
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

impl tokio::io::AsyncWrite for TokioTcpStream {
  fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_flush(cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_shutdown(cx)
  }
}

pub struct TokioTcpStreamOwnedReadHalf {
  stream: ::tokio::net::tcp::OwnedReadHalf,
}

impl futures_util::io::AsyncRead for TokioTcpStreamOwnedReadHalf {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat()).poll_read(cx, buf)
  }
}

impl tokio::io::AsyncRead for TokioTcpStreamOwnedReadHalf {
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

pub struct TokioTcpStreamOwnedWriteHalf {
  stream: ::tokio::net::tcp::OwnedWriteHalf,
}

impl futures_util::io::AsyncWrite for TokioTcpStreamOwnedWriteHalf {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_write(cx, buf)
  }

  fn poll_flush(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_flush(cx)
  }

  fn poll_close(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_close(cx)
  }
}

impl ::tokio::io::AsyncWrite for TokioTcpStreamOwnedWriteHalf {
  fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_flush(cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_shutdown(cx)
  }
}

impl TcpStreamOwnedReadHalf for TokioTcpStreamOwnedReadHalf {
  type Runtime = TokioRuntime;

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }
}

impl TcpStreamOwnedWriteHalf for TokioTcpStreamOwnedWriteHalf {
  type Runtime = TokioRuntime;

  fn forget(self) {
    self.stream.forget()
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }
}

impl crate::net::TcpStream for TokioTcpStream {
  type Runtime = TokioRuntime;
  type OwnedReadHalf = TokioTcpStreamOwnedReadHalf;
  type OwnedWriteHalf = TokioTcpStreamOwnedWriteHalf;
  type ReuniteError = ::tokio::net::tcp::ReuniteError;

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
          ::std::net::TcpStream::connect(addr)
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        ::std::net::TcpStream::connect(&addrs.collect::<Vec<_>>().as_slice())
      };

      res.and_then(|stream| {
        Ok(Self {
          stream: {
            stream.set_nonblocking(true)?;
            TcpStream::from_std(stream)?
          },
        })
      })
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

  fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
    let (read, write) = self.stream.into_split();
    (
      TokioTcpStreamOwnedReadHalf { stream: read },
      TokioTcpStreamOwnedWriteHalf { stream: write },
    )
  }

  fn reunite(
    read: Self::OwnedReadHalf,
    write: Self::OwnedWriteHalf,
  ) -> Result<Self, Self::ReuniteError> {
    read
      .stream
      .reunite(write.stream)
      .map(|stream| Self { stream })
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

pub struct TokioUdpSocket {
  socket: UdpSocket,
}

impl crate::net::UdpSocket for TokioUdpSocket {
  type Runtime = TokioRuntime;

  fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized,
  {
    async move {
      let mut addrs = addr.to_socket_addrs().await?;

      let res = if addrs.size_hint().0 <= 1 {
        if let Some(addr) = addrs.next() {
          std::net::UdpSocket::bind(addr)
        } else {
          return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid socket address",
          ));
        }
      } else {
        std::net::UdpSocket::bind(&addrs.collect::<Vec<_>>().as_slice())
      };
      res.and_then(|socket| {
        Ok(Self {
          socket: {
            socket.set_nonblocking(true)?;
            UdpSocket::from_std(socket)?
          },
        })
      })
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

  fn recv(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send {
    async move { self.socket.recv(buf).await }
  }

  fn recv_from(
    &self,
    buf: &mut [u8],
  ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send {
    async move { self.socket.recv_from(buf).await }
  }

  fn send(&self, buf: &[u8]) -> impl Future<Output = io::Result<usize>> + Send {
    async move { self.socket.send(buf).await }
  }

  fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> impl Future<Output = io::Result<usize>> + Send {
    async move {
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
    let mut buf = tokio::io::ReadBuf::new(buf);
    let addr = futures_util::ready!(UdpSocket::poll_recv_from(&self.socket, cx, &mut buf))?;
    let len = buf.filled().len();
    Poll::Ready(Ok((len, addr)))
  }

  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>> {
    self.socket.poll_send_to(cx, buf, target)
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.socket.local_addr()
  }
}
