use std::{io, pin::Pin, task::{Context, Poll}};

use agnostic_lite::tokio::TokioRuntime;
use futures_util::Stream;
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

impl TryFrom<socket2::Socket> for TcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Runtime = TokioRuntime;
  type Stream = super::TcpStream;
  type Incoming<'a> = Incoming<'a>;

  fn incoming(&self) -> Self::Incoming<'_> {
    Incoming {
      ln: self,
    }
  }

  fn into_incoming(self) -> impl Stream<Item = io::Result<Self::Stream>> + Send {
    OwnedIncoming {
      ln: self.ln,
    }
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.ln.set_ttl(ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }
  
  tcp_listener_common_methods!(TokioTcpListener.ln);
}

/// A stream of incoming TCP connections.
///
/// This stream is infinite, i.e awaiting the next connection will never result in [`None`]. It is
/// created by the [`TcpListener::incoming()`](crate::TcpListener::incoming) method.
pub struct Incoming<'a> {
  ln: &'a TcpListener,
}

impl core::fmt::Debug for Incoming<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "Incoming {{ ... }}")
  }
}

impl Stream for Incoming<'_> {
  type Item = io::Result<super::TcpStream>;

  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match self.ln.ln.poll_accept(cx) {
      Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(stream.into()))),
      Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
      Poll::Pending => Poll::Pending,
    }
  }
}

struct OwnedIncoming {
  ln: TokioTcpListener,
}

impl Stream for OwnedIncoming {
  type Item = io::Result<super::TcpStream>;

  fn poll_next(
    self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match self.ln.poll_accept(cx) {
      Poll::Ready(Ok((stream, _))) => Poll::Ready(Some(Ok(stream.into()))),
      Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
      Poll::Pending => Poll::Pending,
    }
  }
}