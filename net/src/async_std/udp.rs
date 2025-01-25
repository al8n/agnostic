use std::{io, net::SocketAddr, task::{Context, Poll}};

use agnostic_lite::async_std::AsyncStdRuntime;
use async_std::net::UdpSocket as AsyncStdUdpSocket;
use futures_util::FutureExt;


/// [`UdpSocket`](super::UdpSocket) implementation for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug)]
pub struct UdpSocket {
  socket: AsyncStdUdpSocket,
}

impl From<AsyncStdUdpSocket> for UdpSocket {
  fn from(socket: AsyncStdUdpSocket) -> Self {
    Self { socket }
  }
}

impl TryFrom<std::net::UdpSocket> for UdpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(socket: std::net::UdpSocket) -> Result<Self, Self::Error> {
    Ok(Self::from(AsyncStdUdpSocket::from(socket)))
  }
}

impl TryFrom<socket2::Socket> for UdpSocket {
  type Error = io::Error;

  fn try_from(socket: socket2::Socket) -> io::Result<Self> {
    Self::try_from(std::net::UdpSocket::from(socket))
  }
}

impl_as_raw_fd!(UdpSocket.socket);
impl_as_fd_async_std!(UdpSocket.socket);

impl crate::UdpSocket for UdpSocket {
  type Runtime = AsyncStdRuntime;

  udp_common_methods_impl!(AsyncStdUdpSocket.socket);

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
