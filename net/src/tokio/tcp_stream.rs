use std::{io, pin::Pin, task::{Context, Poll}};

use agnostic_io::tokio_compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use agnostic_lite::tokio::TokioRuntime;
use ::tokio::net::TcpStream as TokioTcpStream;

mod owned;
pub use owned::*;


/// A [`TcpStream`](super::super::TcpStream) implementation for [`tokio`] runtime.
pub struct TcpStream {
  stream: TokioTcpStream,
}

impl From<TokioTcpStream> for TcpStream {
  fn from(stream: TokioTcpStream) -> Self {
    Self { stream }
  }
}

impl TryFrom<std::net::TcpStream> for TcpStream {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    TokioTcpStream::from_std(stream).map(Self::from)
  }
}

impl TryFrom<socket2::Socket> for TcpStream {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpStream::from(socket))
  }
}

impl_as!(TcpStream.stream);

impl futures_util::AsyncRead for TcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat()).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for TcpStream {
  fn poll_write(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &[u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_write(cx, buf)
  }

  fn poll_flush(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_flush(cx)
  }

  fn poll_close(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_close(cx)
  }
}

impl tokio::io::AsyncRead for TcpStream {
  fn poll_read(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut agnostic_io::tokio_compat::FuturesAsyncReadCompatExt::compat(
      self.get_mut(),
    ))
    .poll_read(cx, buf)
  }
}

impl tokio::io::AsyncWrite for TcpStream {
  fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    Pin::new(&mut agnostic_io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_flush(cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_shutdown(cx)
  }
}

impl super::super::TcpStream for TcpStream {
  type Runtime = TokioRuntime;
  type OwnedReadHalf = OwnedReadHalf;
  type OwnedWriteHalf = OwnedWriteHalf;
  type ReuniteError = ::tokio::net::tcp::ReuniteError;

  tcp_stream_common_methods!("tokio"::stream);

  fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf) {
    let (read, write) = self.stream.into_split();
    (
      Self::OwnedReadHalf { stream: read },
      Self::OwnedWriteHalf { stream: write },
    )
  }

  fn reunite(
    read: Self::OwnedReadHalf,
    write: Self::OwnedWriteHalf,
  ) -> Result<Self, Self::ReuniteError> {
    read
      .stream
      .reunite(write.stream)
      .map(Self::from)
  }
}

