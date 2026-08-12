use std::{
  io,
  net::SocketAddr,
  sync::Arc,
  task::{Context, Poll},
};

use agnostic_lite::smol::SmolRuntime;
use async_io::Async;
use smol::net::TcpListener as SmolTcpListener;

/// The [`TcpListener`](crate::TcpListener) implementation for [`smol`] runtime
///
/// [`smol`]: https://docs.rs/smol
#[derive(Debug)]
#[repr(transparent)]
pub struct TcpListener {
  ln: SmolTcpListener,
}

impl From<SmolTcpListener> for TcpListener {
  fn from(ln: SmolTcpListener) -> Self {
    Self { ln }
  }
}

impl TryFrom<std::net::TcpListener> for TcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpListener) -> Result<Self, Self::Error> {
    SmolTcpListener::try_from(stream).map(Self::from)
  }
}

impl_as!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Runtime = SmolRuntime;
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
    // Mirror the UDP `poll_recv_from` strategy: drive readiness on the persistent `Async` source
    // and run the accept as a non-blocking syscall, since re-creating an `async` future per poll
    // does not reliably deliver readiness wakeups when polled directly.
    let inner: Arc<Async<std::net::TcpListener>> = self.ln.clone().into();
    loop {
      match inner.get_ref().accept() {
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => match inner.poll_readable(cx) {
          Poll::Ready(Ok(())) => continue,
          Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
          Poll::Pending => return Poll::Pending,
        },
        Ok((stream, addr)) => {
          // `try_from` registers the accepted (blocking) socket with the reactor and switches it to
          // non-blocking mode.
          return Poll::Ready(Self::Stream::try_from(stream).map(|stream| (stream, addr)));
        }
        Err(e) => return Poll::Ready(Err(e)),
      }
    }
  }

  tcp_listener_common_methods!(SmolTcpListener.ln);
}
