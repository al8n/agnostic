use std::{
  io,
  net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr},
  pin::Pin,
  task::{Context, Poll},
};

use futures_util::FutureExt;
use smol::net::{TcpListener, TcpStream, UdpSocket};

use super::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs};

use agnostic_lite::smol::SmolRuntime;

#[cfg(feature = "quinn")]
pub use quinn_::SmolQuinnRuntime;

#[cfg(feature = "quinn")]
mod quinn_ {
  use std::net::UdpSocket;

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
      t: UdpSocket,
    ) -> std::io::Result<std::sync::Arc<dyn quinn::AsyncUdpSocket>> {
      self.0.wrap_udp_socket(t)
    }
  }
}

/// The [`Net`] implementation for [`smol`](::smol) runtime
#[derive(Debug, Default, Clone, Copy)]
pub struct SmolNet;

impl Net for SmolNet {
  type Runtime = SmolRuntime;
  type TcpListener = SmolTcpListener;
  type TcpStream = SmolTcpStream;
  type UdpSocket = SmolUdpSocket;

  #[cfg(feature = "quinn")]
  type Quinn = SmolQuinnRuntime;
}

/// The [`TcpListener`](super::TcpListener) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolTcpListener {
  ln: TcpListener,
}

impl TryFrom<std::net::TcpListener> for SmolTcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(ln: std::net::TcpListener) -> Result<Self, Self::Error> {
    TcpListener::try_from(ln).map(|ln| Self { ln })
  }
}

impl super::TcpListener for SmolTcpListener {
  type Stream = SmolTcpStream;
  type Runtime = SmolRuntime;

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
      .map(|(stream, addr)| (SmolTcpStream { stream }, addr))
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }
}

/// The [`TcpStream`](super::TcpStream) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolTcpStream {
  stream: TcpStream,
}

impl TryFrom<std::net::TcpStream> for SmolTcpStream {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    TcpStream::try_from(stream).map(|stream| Self { stream })
  }
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

/// The [`TcpStreamOwnedReadHalf`](super::TcpStreamOwnedReadHalf) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
pub struct SmolTcpStreamOwnedReadHalf {
  stream: TcpStream,
  id: usize,
}

/// The [`TcpStreamOwnedWriteHalf`](super::TcpStreamOwnedWriteHalf) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
pub struct SmolTcpStreamOwnedWriteHalf {
  stream: TcpStream,
  shutdown_on_drop: bool,
  id: usize,
}

impl Drop for SmolTcpStreamOwnedWriteHalf {
  fn drop(&mut self) {
    if self.shutdown_on_drop {
      let _ = self.stream.shutdown(Shutdown::Write);
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

  // async fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
  //   self.stream.peek(buf).await
  // }
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

impl super::TcpStream for SmolTcpStream {
  type Runtime = SmolRuntime;
  type OwnedReadHalf = SmolTcpStreamOwnedReadHalf;
  type OwnedWriteHalf = SmolTcpStreamOwnedWriteHalf;
  type ReuniteError = ReuniteError;

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

/// The [`UdpSocket`](super::UdpSocket) implementation for [`smol`](::smol) runtime
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolUdpSocket {
  socket: UdpSocket,
}

impl TryFrom<std::net::UdpSocket> for SmolUdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    UdpSocket::try_from(socket).map(|socket| Self { socket })
  }
}

impl super::UdpSocket for SmolUdpSocket {
  type Runtime = SmolRuntime;

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
    }

    #[cfg(unix)]
    {
      use std::os::fd::AsRawFd;
      return super::set_read_buffer(self.socket.as_raw_fd(), size);
    }

    #[cfg(windows)]
    {
      use std::os::windows::io::AsRawSocket;
      return super::set_read_buffer(self.socket.as_raw_socket(), size);
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
      return super::set_write_buffer(self.socket.as_raw_fd(), size);
    }

    #[cfg(windows)]
    {
      use std::os::windows::io::AsRawSocket;
      return super::set_write_buffer(self.socket.as_raw_socket(), size);
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
