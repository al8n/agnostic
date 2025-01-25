use std::{
  io,
  net::{Shutdown, SocketAddr},
  pin::Pin,
  task::{Context, Poll}, time::Duration,
};

use atomic_time::AtomicDuration;
use futures_util::FutureExt;
use smol::net::{TcpListener, TcpStream, UdpSocket};
use triomphe::Arc;

use super::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs};

use agnostic_lite::{smol::SmolRuntime, RuntimeLite};

/// The [`Net`] implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
#[derive(Debug, Default, Clone, Copy)]
pub struct SmolNet;

impl Net for SmolNet {
  type Runtime = SmolRuntime;
  type TcpListener = SmolTcpListener;
  type TcpStream = SmolTcpStream;
  type UdpSocket = SmolUdpSocket;
}

/// The [`TcpListener`](super::TcpListener) implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolTcpListener {
  ln: TcpListener,
}

impl From<TcpListener> for SmolTcpListener {
  fn from(ln: TcpListener) -> Self {
    Self { ln }
  }
}

impl TryFrom<std::net::TcpListener> for SmolTcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpListener) -> Result<Self, Self::Error> {
    TcpListener::try_from(stream).map(Self::from)
  }
}

impl TryFrom<socket2::Socket> for SmolTcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as!(SmolTcpListener.ln);

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

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.ln.set_ttl(ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }
}

/// The [`TcpStream`](super::TcpStream) implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
#[derive(Debug)]
#[repr(transparent)]
pub struct SmolTcpStream {
  stream: TcpStream,
}

impl From<TcpStream> for SmolTcpStream {
  fn from(stream: TcpStream) -> Self {
    Self { stream }
  }
}

impl TryFrom<std::net::TcpStream> for SmolTcpStream {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    TcpStream::try_from(stream).map(Self::from)
  }
}

impl TryFrom<socket2::Socket> for SmolTcpStream {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpStream::from(socket))
  }
}

impl_as!(SmolTcpStream.stream);

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

#[cfg(feature = "tokio-io")]
impl tokio::io::AsyncRead for SmolTcpStream {
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
impl tokio::io::AsyncWrite for SmolTcpStream {
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

/// The [`TcpStreamOwnedReadHalf`](super::TcpStreamOwnedReadHalf) implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
#[derive(Debug)]
pub struct SmolTcpStreamOwnedReadHalf {
  stream: TcpStream,
  id: usize,
}

/// The [`TcpStreamOwnedWriteHalf`](super::TcpStreamOwnedWriteHalf) implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
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

#[cfg(feature = "tokio-io")]
impl tokio::io::AsyncRead for SmolTcpStreamOwnedReadHalf {
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
impl tokio::io::AsyncWrite for SmolTcpStreamOwnedWriteHalf {
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

impl TcpStreamOwnedReadHalf for SmolTcpStreamOwnedReadHalf {
  type Runtime = SmolRuntime;

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
}

/// The [`UdpSocket`](super::UdpSocket) implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
#[derive(Debug)]
pub struct SmolUdpSocket {
  socket: UdpSocket,
  recv_timeout: Arc<AtomicDuration>,
  send_timeout: Arc<AtomicDuration>,
}

impl From<UdpSocket> for SmolUdpSocket {
  #[inline]
  fn from(socket: UdpSocket) -> Self {
    Self {
      socket,
      recv_timeout: Arc::new(AtomicDuration::new(Duration::ZERO)),
      send_timeout: Arc::new(AtomicDuration::new(Duration::ZERO)),
    }
  }
}

impl TryFrom<socket2::Socket> for SmolUdpSocket {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::UdpSocket::from(socket))
  }
}

impl TryFrom<std::net::UdpSocket> for SmolUdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    UdpSocket::try_from(socket).map(Self::from)
  }
}

impl_as!(SmolUdpSocket.socket);

impl super::UdpSocket for SmolUdpSocket {
  type Runtime = SmolRuntime;

  udp_common_methods_impl!(UdpSocket.socket);  

  timeout_methods_impl!(recv <=> send);
  
  fn poll_recv_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>> {
    match self.recv_timeout() {
      Some(timeout) => {
        let fut = self.socket.recv_from(buf);
        let timeout = <Self::Runtime as RuntimeLite>::timeout(timeout, fut);
        futures_util::pin_mut!(timeout);
        timeout
          .poll_unpin(cx)
          .map(|res| match res {
            Ok(Ok(res)) => Ok(res),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e.into()),
          })
      }
      None => {
        let fut = self.socket.recv_from(buf);
        futures_util::pin_mut!(fut);
        fut.poll_unpin(cx)
      },
    }
  }
  
  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>> {
    match self.send_timeout() {
      Some(timeout) => {
        let fut = self.socket.send_to(buf, target);
        let timeout = <Self::Runtime as RuntimeLite>::timeout(timeout, fut);
        futures_util::pin_mut!(timeout);
        timeout
          .poll_unpin(cx)
          .map(|res| match res {
            Ok(Ok(len)) => Ok(len),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e.into()),
          })
      }
      None => {
        let fut = self.socket.send_to(buf, target);
        futures_util::pin_mut!(fut);
        fut.poll_unpin(cx)
      },
    }
  }
}
