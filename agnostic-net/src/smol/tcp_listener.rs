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

impl_as!(TcpListener.ln);

impl crate::TcpListener for TcpListener {
  type Runtime = SmolRuntime;
  type Stream = super::TcpStream;
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
    self.ln.set_ttl(ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    self.ln.ttl()
  }

  tcp_listener_common_methods!(SmolTcpListener.ln);
}

tcp_listener_incoming!(smol::net::Incoming<'a> => super::TcpStream);
