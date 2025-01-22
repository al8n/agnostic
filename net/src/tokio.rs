use std::{
  io,
  net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr},
  pin::Pin,
  task::{Context, Poll},
};

use tokio::net::{TcpListener, TcpStream, UdpSocket};
use super::io::tokio_compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs};

use agnostic_lite::tokio::TokioRuntime;
/// The [`Net`] implementation for [`tokio`] runtime.
/// 
/// [`tokio`]: https://docs.rs/tokio
#[derive(Debug, Default, Clone, Copy)]
pub struct TokioNet;

impl Net for TokioNet {
  type Runtime = TokioRuntime;
  type TcpListener = TokioTcpListener;
  type TcpStream = TokioTcpStream;
  type UdpSocket = TokioUdpSocket;
}

/// A [`TcpListener`](super::TcpListener) implementation for [`tokio`] runtime.
/// 
/// [`tokio`]: https://docs.rs/tokio
pub struct TokioTcpListener {
  ln: TcpListener,
}

impl TryFrom<std::net::TcpListener> for TokioTcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(ln: std::net::TcpListener) -> Result<Self, Self::Error> {
    TcpListener::from_std(ln).map(|ln| Self { ln })
  }
}

impl super::TcpListener for TokioTcpListener {
  type Stream = TokioTcpStream;
  type Runtime = TokioRuntime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs().await?;

    let mut last_err = None;

    for addr in addrs {
      match TcpListener::bind(addr).await {
        Ok(ln) => return Ok(Self { ln }),
        Err(e) => last_err = Some(e),
      }
    }

    Err(last_err.unwrap_or_else(|| {
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "could not resolve to any address",
      )
    }))
  }

  async fn accept(&self) -> io::Result<(Self::Stream, SocketAddr)> {
    self
      .ln
      .accept()
      .await
      .map(|(stream, addr)| (TokioTcpStream { stream }, addr))
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }
}

/// A [`TcpStream`](super::TcpStream) implementation for [`tokio`] runtime.
pub struct TokioTcpStream {
  stream: TcpStream,
}

impl TryFrom<std::net::TcpStream> for TokioTcpStream {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    TcpStream::from_std(stream).map(|stream| Self { stream })
  }
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
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncReadCompatExt::compat(
      self.get_mut(),
    ))
    .poll_read(cx, buf)
  }
}

impl tokio::io::AsyncWrite for TokioTcpStream {
  fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_flush(cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_shutdown(cx)
  }
}

/// The [`TcpStreamOwnedReadHalf`] implementation for [`tokio`] runtime.
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
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncReadCompatExt::compat(
      self.get_mut(),
    ))
    .poll_read(cx, buf)
  }
}

/// The [`TcpStreamOwnedWriteHalf`] implementation for [`tokio`] runtime.
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
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
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

  async fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    self.stream.peek(buf).await
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

impl super::TcpStream for TokioTcpStream {
  type Runtime = TokioRuntime;
  type OwnedReadHalf = TokioTcpStreamOwnedReadHalf;
  type OwnedWriteHalf = TokioTcpStreamOwnedWriteHalf;
  type ReuniteError = ::tokio::net::tcp::ReuniteError;

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs().await?;

    let mut last_err = None;

    for addr in addrs {
      match TcpStream::connect(addr).await {
        Ok(stream) => return Ok(Self { stream }),
        Err(e) => last_err = Some(e),
      }
    }

    Err(last_err.unwrap_or_else(|| {
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "could not resolve to any address",
      )
    }))
  }

  async fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
    self.stream.peek(buf).await
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

  fn shutdown(&self, how: Shutdown) -> io::Result<()> {
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

/// The [`UdpSocket`](super::UdpSocket) implementation for [`tokio`] runtime.
pub struct TokioUdpSocket {
  socket: UdpSocket,
}

impl TryFrom<std::net::UdpSocket> for TokioUdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    UdpSocket::from_std(socket).map(|socket| Self { socket })
  }
}

impl super::UdpSocket for TokioUdpSocket {
  type Runtime = TokioRuntime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs().await?;

    let mut last_err = None;
    for addr in addrs {
      match UdpSocket::bind(addr).await {
        Ok(socket) => return Ok(Self { socket }),
        Err(e) => {
          last_err = Some(e);
        }
      }
    }

    Err(last_err.unwrap_or_else(|| {
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "could not resolve to any address",
      )
    }))
  }

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> io::Result<()> {
    let addrs = addr.to_socket_addrs().await?;

    let mut last_err = None;

    for addr in addrs {
      match self.socket.connect(addr).await {
        Ok(()) => return Ok(()),
        Err(e) => last_err = Some(e),
      }
    }

    Err(last_err.unwrap_or_else(|| {
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "could not resolve to any address",
      )
    }))
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
      Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid socket address",
      ))
    }
  }

  async fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
    self.socket.peek(buf).await
  }

  async fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
    self.socket.peek_from(buf).await
  }

  fn join_multicast_v4(&self, multiaddr: Ipv4Addr, interface: Ipv4Addr) -> io::Result<()> {
    self.socket.join_multicast_v4(multiaddr, interface)
  }

  fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
    self.socket.join_multicast_v6(multiaddr, interface)
  }

  fn leave_multicast_v4(&self, multiaddr: Ipv4Addr, interface: Ipv4Addr) -> io::Result<()> {
    self.socket.leave_multicast_v4(multiaddr, interface)
  }

  fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()> {
    self.socket.leave_multicast_v6(multiaddr, interface)
  }

  fn multicast_loop_v4(&self) -> io::Result<bool> {
    self.socket.multicast_loop_v4()
  }

  fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()> {
    self.socket.set_multicast_loop_v4(on)
  }

  fn multicast_ttl_v4(&self) -> io::Result<u32> {
    self.socket.multicast_ttl_v4()
  }

  fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
    self.socket.set_multicast_ttl_v4(ttl)
  }

  fn multicast_loop_v6(&self) -> io::Result<bool> {
    self.socket.multicast_loop_v6()
  }

  fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()> {
    self.socket.set_multicast_loop_v6(on)
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
      let _ = size;
      Ok(())
    }

    #[cfg(all(unix, feature = "socket2"))]
    {
      use std::os::fd::AsRawFd;
      super::set_read_buffer(self.socket.as_raw_fd(), size)
    }

    #[cfg(all(windows, feature = "socket2"))]
    {
      use std::os::windows::io::AsRawSocket;
      super::set_read_buffer(self.socket.as_raw_socket(), size)
    }
  }

  fn set_write_buffer(&self, size: usize) -> io::Result<()> {
    #[cfg(not(any(unix, windows)))]
    {
      panic!("unsupported platform");
      let _ = size;
      Ok(())
    }

    #[cfg(unix)]
    {
      use std::os::fd::AsRawFd;
      super::set_write_buffer(self.socket.as_raw_fd(), size)
    }

    #[cfg(windows)]
    {
      use std::os::windows::io::AsRawSocket;
      super::set_write_buffer(self.socket.as_raw_socket(), size)
    }
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
