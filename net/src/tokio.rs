use std::{
  io, net::SocketAddr, pin::Pin, task::{Context, Poll}, time::Duration,
};

use atomic_time::AtomicDuration;
use futures_util::FutureExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use triomphe::Arc;
use super::io::tokio_compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::{Net, TcpStreamOwnedReadHalf, TcpStreamOwnedWriteHalf, ToSocketAddrs};

use agnostic_lite::{tokio::TokioRuntime, RuntimeLite};
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

impl From<TcpListener> for TokioTcpListener {
  fn from(ln: TcpListener) -> Self {
    Self { ln }
  }
}

impl TryFrom<std::net::TcpListener> for TokioTcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpListener) -> Result<Self, Self::Error> {
    TcpListener::try_from(stream).map(Self::from)
  }
}

impl TryFrom<socket2::Socket> for TokioTcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as!(TokioTcpListener.ln);

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
  
  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.ln.set_ttl(ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }
}

/// A [`TcpStream`](super::TcpStream) implementation for [`tokio`] runtime.
pub struct TokioTcpStream {
  stream: TcpStream,
}

impl From<TcpStream> for TokioTcpStream {
  fn from(stream: TcpStream) -> Self {
    Self { stream }
  }
}

impl TryFrom<std::net::TcpStream> for TokioTcpStream {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    TcpStream::from_std(stream).map(Self::from)
  }
}

impl TryFrom<socket2::Socket> for TokioTcpStream {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpStream::from(socket))
  }
}

impl_as!(TokioTcpStream.stream);

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
}

/// The [`UdpSocket`](super::UdpSocket) implementation for [`tokio`] runtime.
pub struct TokioUdpSocket {
  socket: UdpSocket,
  recv_timeout: Arc<AtomicDuration>,
  send_timeout: Arc<AtomicDuration>,
}

impl From<UdpSocket> for TokioUdpSocket {
  fn from(socket: UdpSocket) -> Self {
    Self {
      socket,
      recv_timeout: Arc::new(AtomicDuration::new(Duration::ZERO)),
      send_timeout: Arc::new(AtomicDuration::new(Duration::ZERO)),
    }
  }
}

impl TryFrom<socket2::Socket> for TokioUdpSocket {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::UdpSocket::from(socket))
  }
}

impl TryFrom<std::net::UdpSocket> for TokioUdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    UdpSocket::from_std(socket).map(Self::from)
  }
}

impl_as!(TokioUdpSocket.socket);

impl super::UdpSocket for TokioUdpSocket {
  type Runtime = TokioRuntime;

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
        let mut buf = tokio::io::ReadBuf::new(buf);
        let addr = futures_util::ready!(UdpSocket::poll_recv_from(&self.socket, cx, &mut buf))?;
        let len = buf.filled().len();
        Poll::Ready(Ok((len, addr)))
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
      None => self.socket.poll_send_to(cx, buf, target),
    }
  }
}
