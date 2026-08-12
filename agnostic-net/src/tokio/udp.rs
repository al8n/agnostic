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

  fn poll_peek_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>> {
    // `try_peek_from` only reports readiness via `WouldBlock`, so loop until the readiness
    // registered by `poll_recv_ready` actually yields data (or a non-`WouldBlock` error).
    loop {
      futures_util::ready!(self.socket.poll_recv_ready(cx))?;
      match self.socket.try_peek_from(buf) {
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
        res => return Poll::Ready(res),
      }
    }
  }

  fn poll_send(&self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    self.socket.poll_send(cx, buf)
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
