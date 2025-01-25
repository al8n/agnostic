use std::{io, net::SocketAddr, task::{Context, Poll}};

use agnostic_lite::smol::SmolRuntime;
use futures_util::FutureExt;
use smol::net::UdpSocket as SmolUdpSocket;

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
    Self {
      socket,
    }
  }
}

impl TryFrom<socket2::Socket> for UdpSocket {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::UdpSocket::from(socket))
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
    let fut = self.socket.recv_from(buf);
    futures_util::pin_mut!(fut);
    fut.poll_unpin(cx)
  }
  
  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>> {
    let fut = self.socket.send_to(buf, target);
    futures_util::pin_mut!(fut);
    fut.poll_unpin(cx)
  }
}
