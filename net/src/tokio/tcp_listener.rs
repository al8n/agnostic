use std::io;

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

impl TryFrom<socket2::Socket> for TcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as!(TcpListener.ln);

impl super::super::TcpListener for TcpListener {
  type Stream = super::TcpStream;
  type Runtime = TokioRuntime;

  tcp_listener_common_methods!(TokioTcpListener.ln);
  
  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.ln.set_ttl(ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }
}