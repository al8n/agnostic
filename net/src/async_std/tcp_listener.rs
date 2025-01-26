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
    Ok(Self::from(AsyncStdTcpListener::from(ln)))
  }
}

impl_as_raw_fd!(TcpListener.ln);
impl_as_fd_async_std!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Runtime = AsyncStdRuntime;
  type Stream = TcpStream;
  type Incoming<'a> = Incoming<'a>;

  fn incoming(&self) -> Self::Incoming<'_> {
    self.ln.incoming().into()
  }

  fn into_incoming(self) -> impl ::futures_util::stream::Stream<Item = ::std::io::Result<Self::Stream>> + ::core::marker::Send {
    ::futures_util::stream::unfold(self, |listener| async move {
      let res = listener.accept().await.map(|(stream, _)| stream);
      ::core::option::Option::Some((res, listener))
    })
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    crate::os::set_ttl(self, ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    crate::os::ttl(self)
  }
  
  tcp_listener_common_methods!(AsyncStdTcpListener.ln);
}

tcp_listener_incoming!(async_std::net::Incoming<'a> => TcpStream);
