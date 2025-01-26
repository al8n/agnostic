use std::{io, pin::Pin, task::{Context, Poll}};

use agnostic_io::tokio_compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use agnostic_lite::tokio::TokioRuntime;

/// The [`OwnedReadHalf`](super::super::super::OwnedReadHalf) implementation for [`tokio`] runtime.
pub struct OwnedReadHalf {
  pub(super) stream: ::tokio::net::tcp::OwnedReadHalf,
}

impl From<::tokio::net::tcp::OwnedReadHalf> for OwnedReadHalf {
  fn from(stream: ::tokio::net::tcp::OwnedReadHalf) -> Self {
    Self { stream }
  }
}

impl From<OwnedReadHalf> for ::tokio::net::tcp::OwnedReadHalf {
  fn from(stream: OwnedReadHalf) -> Self {
    stream.stream
  }
}

impl futures_util::io::AsyncRead for OwnedReadHalf {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    Pin::new(&mut (&mut self.stream).compat()).poll_read(cx, buf)
  }
}

impl tokio::io::AsyncRead for OwnedReadHalf {
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

/// The [`OwnedWriteHalf`](super::super::super::OwnedWriteHalf) implementation for [`tokio`] runtime.
pub struct OwnedWriteHalf {
  pub(super) stream: ::tokio::net::tcp::OwnedWriteHalf,
}

impl From<::tokio::net::tcp::OwnedWriteHalf> for OwnedWriteHalf {
  fn from(stream: ::tokio::net::tcp::OwnedWriteHalf) -> Self {
    Self { stream }
  }
}

impl From<OwnedWriteHalf> for ::tokio::net::tcp::OwnedWriteHalf {
  fn from(stream: OwnedWriteHalf) -> Self {
    stream.stream
  }
}

impl futures_util::io::AsyncWrite for OwnedWriteHalf {
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

impl ::tokio::io::AsyncWrite for OwnedWriteHalf {
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

impl super::super::super::OwnedReadHalf for OwnedReadHalf {
  type Runtime = TokioRuntime;

  tcp_stream_owned_read_half_common_methods!(stream);
}

impl super::super::super::OwnedWriteHalf for OwnedWriteHalf {
  type Runtime = TokioRuntime;

  fn forget(self) {
    self.stream.forget()
  }

  tcp_stream_owned_write_half_common_methods!(stream);
}