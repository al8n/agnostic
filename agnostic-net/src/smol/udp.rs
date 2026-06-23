use std::{
  io,
  net::SocketAddr,
  sync::Arc,
  task::{Context, Poll},
};

use agnostic_lite::smol::SmolRuntime;
use smol::{Async, net::UdpSocket as SmolUdpSocket};

/// The [`UdpSocket`](super::UdpSocket) implementation for [`smol`] runtime
///
/// [`smol`]: https://docs.rs/smol
#[derive(Debug)]
pub struct UdpSocket {
  socket: SmolUdpSocket,
}

impl From<SmolUdpSocket> for UdpSocket {
  #[inline]
  fn from(socket: SmolUdpSocket) -> Self {
    Self { socket }
  }
}

impl TryFrom<std::net::UdpSocket> for UdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    SmolUdpSocket::try_from(socket).map(Self::from)
  }
}

impl_as!(UdpSocket.socket);

impl super::super::UdpSocket for UdpSocket {
  type Runtime = SmolRuntime;

  udp_common_methods_impl!(SmolUdpSocket.socket);

  fn poll_recv_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>> {
    // Poll readiness on the underlying `Async` source (whose waker registration persists across
    // polls) and run the receive as a non-blocking syscall. Re-creating an `async` `recv_from`
    // future on every poll — as the previous impl did — does not reliably deliver readiness
    // wakeups when the socket is polled directly (e.g. by hickory's UDP client), so the receive
    // could stall forever.
    let inner: Arc<Async<std::net::UdpSocket>> = self.socket.clone().into();
    loop {
      match inner.get_ref().recv_from(buf) {
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => match inner.poll_readable(cx) {
          Poll::Ready(Ok(())) => continue,
          Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
          Poll::Pending => return Poll::Pending,
        },
        res => return Poll::Ready(res),
      }
    }
  }

  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>> {
    let inner: Arc<Async<std::net::UdpSocket>> = self.socket.clone().into();
    loop {
      match inner.get_ref().send_to(buf, target) {
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => match inner.poll_writable(cx) {
          Poll::Ready(Ok(())) => continue,
          Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
          Poll::Pending => return Poll::Pending,
        },
        res => return Poll::Ready(res),
      }
    }
  }
}
