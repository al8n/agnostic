use std::{io, net::SocketAddr, task::{Context, Poll}};

use agnostic_lite::tokio::TokioRuntime;
use ::tokio::net::UdpSocket as TokioUdpSocket;

/// The [`UdpSocket`](super::super::UdpSocket) implementation for [`tokio`] runtime.
pub struct UdpSocket {
  socket: TokioUdpSocket,
}

impl From<TokioUdpSocket> for UdpSocket {
  fn from(socket: TokioUdpSocket) -> Self {
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
    TokioUdpSocket::from_std(socket).map(Self::from)
  }
}

impl_as!(UdpSocket.socket);

impl super::super::UdpSocket for UdpSocket {
  type Runtime = TokioRuntime;

  udp_common_methods_impl!(TokioUdpSocket.socket);

  fn poll_recv_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>> {
    let mut buf = tokio::io::ReadBuf::new(buf);
    let addr = futures_util::ready!(TokioUdpSocket::poll_recv_from(&self.socket, cx, &mut buf))?;
    let len = buf.filled().len();
    Poll::Ready(Ok((len, addr)))
  }
  
  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>> {
    self.socket.poll_send_to(cx, buf, target)
  }
}
