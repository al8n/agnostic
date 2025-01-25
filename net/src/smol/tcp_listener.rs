use std::io;

use agnostic_lite::smol::SmolRuntime;
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

impl TryFrom<socket2::Socket> for TcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Stream = super::TcpStream;
  type Runtime = SmolRuntime;

  tcp_listener_common_methods!(SmolTcpListener.ln);

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.ln.set_ttl(ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }
}