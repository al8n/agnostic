use std::io;

use agnostic_lite::async_std::AsyncStdRuntime;
use async_std::net::TcpListener as AsyncStdTcpListener;

use super::TcpStream;

/// [`TcpListener`](crate::TcpListener) implementation for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
#[repr(transparent)]
pub struct TcpListener {
  ln: AsyncStdTcpListener,
}

impl From<AsyncStdTcpListener> for TcpListener {
  fn from(ln: AsyncStdTcpListener) -> Self {
    Self { ln }
  }
}

impl TryFrom<std::net::TcpListener> for TcpListener {
  type Error = io::Error;

  #[inline]
  fn try_from(ln: std::net::TcpListener) -> Result<Self, Self::Error> {
    Ok(Self {
      ln: AsyncStdTcpListener::from(ln),
    })
  }
}

impl TryFrom<socket2::Socket> for TcpListener {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::TcpListener::from(socket))
  }
}

impl_as_raw_fd!(TcpListener.ln);
impl_as_fd_async_std!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Stream = TcpStream;
  type Runtime = AsyncStdRuntime;

  tcp_listener_common_methods!(AsyncStdTcpListener.ln);

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    crate::set_ttl(self, ttl)
  }
  
  fn ttl(&self) -> io::Result<u32> {
    crate::ttl(self)
  }
}