use std::{
  io,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll}, time::Duration,
};

use ::async_std::net::{TcpListener, TcpStream, UdpSocket};
use atomic_time::AtomicDuration;
use futures_util::FutureExt;
use triomphe::Arc;

use super::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs};

use agnostic_lite::async_std::AsyncStdRuntime;

macro_rules! impl_as_fd_async_std {
  ($name:ident.$field:ident) => {
    #[cfg(unix)]
    impl std::os::fd::AsFd for $name {
      fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        use std::os::fd::{AsRawFd, BorrowedFd};
        let raw_fd = self.$field.as_raw_fd();

        // Safety:
        // The resource pointed to by `fd` remains open for the duration of
        // the returned `BorrowedFd`, and it must not have the value `-1`.
        unsafe { BorrowedFd::borrow_raw(raw_fd) }
      }
    }

    #[cfg(windows)]
    impl std::os::windows::io::AsSocket for $name {
      fn as_socket(&self) -> &std::os::windows::io::Socket {
        use std::os::fd::{AsRawSocket, BorrowedFd};
        use std::os::windows::io::{AsRawSocket, BorrowedSocket};
        let raw_socket = self.$field.as_raw_socket();

        // Safety:
        // The resource pointed to by raw must remain open for the duration of the returned BorrowedSocket,
        // and it must not have the value INVALID_SOCKET.
        unsafe { BorrowedSocket::borrow_raw(raw_socket) }
      }
    }
  };
}

/// Network abstractions for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug, Default, Clone, Copy)]
pub struct AsyncStdNet;

impl Net for AsyncStdNet {
  type Runtime = AsyncStdRuntime;
  type TcpListener = AsyncStdTcpListener;
  type TcpStream = AsyncStdTcpStream;
  type UdpSocket = AsyncStdUdpSocket;
}

/// [`TcpListener`](super::TcpListener) implementation for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
#[repr(transparent)]
pub struct AsyncStdTcpListener {
  ln: TcpListener,
}

impl From<TcpListener> for AsyncStdTcpListener {
  fn from(ln: TcpListener) -> Self {
    Self { ln }
  }
}

impl TryFrom<std::net::TcpListener> for AsyncStdTcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(ln: std::net::TcpListener) -> Result<Self, Self::Error> {
    Ok(Self {
      ln: TcpListener::from(ln),
    })
  }
}

impl TryFrom<socket2::Socket> for AsyncStdTcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as_raw_fd!(AsyncStdTcpListener.ln);
impl_as_fd_async_std!(AsyncStdTcpListener.ln);

impl super::TcpListener for AsyncStdTcpListener {
  type Stream = AsyncStdTcpStream;
  type Runtime = AsyncStdRuntime;

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
      .map(|(stream, addr)| (AsyncStdTcpStream { stream }, addr))
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    super::set_ttl(self, ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    super::ttl(self)
  }
}

/// [`TcpStream`](super::TcpStream) implementation for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
#[repr(transparent)]
pub struct AsyncStdTcpStream {
  stream: TcpStream,
}

impl From<TcpStream> for AsyncStdTcpStream {
  fn from(stream: TcpStream) -> Self {
    Self { stream }
  }
}

impl TryFrom<std::net::TcpStream> for AsyncStdTcpStream {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    Ok(Self {
      stream: TcpStream::from(stream)
    })
  }
}

impl TryFrom<socket2::Socket> for AsyncStdTcpStream {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpStream::from(socket))
  }
}

impl_as_raw_fd!(AsyncStdTcpStream.stream);
impl_as_fd_async_std!(AsyncStdTcpStream.stream);

impl futures_util::AsyncRead for AsyncStdTcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for AsyncStdTcpStream {
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

#[cfg(feature = "tokio-io")]
impl tokio::io::AsyncRead for AsyncStdTcpStream {
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

#[cfg(feature = "tokio-io")]
impl tokio::io::AsyncWrite for AsyncStdTcpStream {
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<Result<usize, io::Error>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_flush(cx)
  }

  fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_shutdown(cx)
  }
}

/// Error indicating that two halves were not from the same socket, and thus could
/// not be reunited.
#[derive(Debug)]
pub struct ReuniteError(
  pub AsyncStdTcpStreamOwnedReadHalf,
  pub AsyncStdTcpStreamOwnedWriteHalf,
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

/// The owned read half of a [`TcpStream`](super::TcpStream) for the [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
pub struct AsyncStdTcpStreamOwnedReadHalf {
  stream: TcpStream,
  id: usize,
}

/// The owned write half of a [`TcpStream`](super::TcpStream) for the [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
pub struct AsyncStdTcpStreamOwnedWriteHalf {
  stream: TcpStream,
  shutdown_on_drop: bool,
  id: usize,
}

impl Drop for AsyncStdTcpStreamOwnedWriteHalf {
  fn drop(&mut self) {
    if self.shutdown_on_drop {
      let _ = self.stream.shutdown(std::net::Shutdown::Write);
    }
  }
}

impl futures_util::AsyncRead for AsyncStdTcpStreamOwnedReadHalf {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream)).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for AsyncStdTcpStreamOwnedWriteHalf {
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

#[cfg(feature = "tokio-io")]
impl tokio::io::AsyncRead for AsyncStdTcpStreamOwnedReadHalf {
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

#[cfg(feature = "tokio-io")]
impl tokio::io::AsyncWrite for AsyncStdTcpStreamOwnedWriteHalf {
  fn poll_write(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<Result<usize, io::Error>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_flush(cx)
  }

  fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
    Pin::new(&mut super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_shutdown(cx)
  }
}

impl TcpStreamOwnedReadHalf for AsyncStdTcpStreamOwnedReadHalf {
  type Runtime = AsyncStdRuntime;

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

impl TcpStreamOwnedWriteHalf for AsyncStdTcpStreamOwnedWriteHalf {
  type Runtime = AsyncStdRuntime;

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

impl super::TcpStream for AsyncStdTcpStream {
  type Runtime = AsyncStdRuntime;
  type OwnedReadHalf = AsyncStdTcpStreamOwnedReadHalf;
  type OwnedWriteHalf = AsyncStdTcpStreamOwnedWriteHalf;
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
      AsyncStdTcpStreamOwnedReadHalf {
        stream: self.stream.clone(),
        id,
      },
      AsyncStdTcpStreamOwnedWriteHalf {
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
}

/// [`UdpSocket`](super::UdpSocket) implementation for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
pub struct AsyncStdUdpSocket {
  socket: UdpSocket,
  recv_timeout: Arc<AtomicDuration>,
  send_timeout: Arc<AtomicDuration>,
}

impl From<UdpSocket> for AsyncStdUdpSocket {
  fn from(socket: UdpSocket) -> Self {
    Self { socket, recv_timeout: Arc::new(AtomicDuration::new(Duration::ZERO)), send_timeout: Arc::new(AtomicDuration::new(Duration::ZERO)) }
  }
}

impl TryFrom<std::net::UdpSocket> for AsyncStdUdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    Ok(Self::from(UdpSocket::from(socket)))
  }
}

impl TryFrom<socket2::Socket> for AsyncStdUdpSocket {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::UdpSocket::from(socket))
  }
}

impl_as_raw_fd!(AsyncStdUdpSocket.socket);
impl_as_fd_async_std!(AsyncStdUdpSocket.socket);

impl super::UdpSocket for AsyncStdUdpSocket {
  type Runtime = AsyncStdRuntime;

  udp_common_methods_impl!(UdpSocket.socket);

  timeout_methods_impl!(recv <=> send);

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
}
