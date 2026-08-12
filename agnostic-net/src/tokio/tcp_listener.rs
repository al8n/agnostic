use std::{
  io,
  net::SocketAddr,
  task::{Context, Poll},
};

use agnostic_lite::tokio::TokioRuntime;
use ::tokio::net::TcpListener as TokioTcpListener;

/// A [`TcpListener`](super::super::TcpListener) implementation for [`tokio`] runtime.
///
/// [`tokio`]: https://docs.rs/tokio
pub struct TcpListener {
  ln: TokioTcpListener,
}

impl From<TokioTcpListener> for TcpListener {
  fn from(ln: TokioTcpListener) -> Self {
    Self { ln }
  }
}

impl TryFrom<std::net::TcpListener> for TcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpListener) -> Result<Self, Self::Error> {
    TokioTcpListener::try_from(stream).map(Self::from)
  }
}

impl_as!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Runtime = TokioRuntime;
  type Stream = super::TcpStream;
  type Incoming<'a> = crate::tcp::Incoming<'a, Self>;
  type IntoIncoming = crate::tcp::IntoIncoming<Self>;

  fn incoming(&self) -> Self::Incoming<'_> {
    crate::tcp::Incoming::new(self)
  }

  fn into_incoming(self) -> Self::IntoIncoming {
    crate::tcp::IntoIncoming::new(self)
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.ln.set_ttl(ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }

  fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<io::Result<(Self::Stream, SocketAddr)>> {
    self
      .ln
      .poll_accept(cx)
      .map_ok(|(stream, addr)| (stream.into(), addr))
  }

  tcp_listener_common_methods!(TokioTcpListener.ln);
}
