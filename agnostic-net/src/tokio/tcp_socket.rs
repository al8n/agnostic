use tokio::net::TcpSocket as TokioTcpSocket;
use std::io;


/// A [`TcpSocket`](super::super::TcpSocket) implementation for [`tokio`] runtime.
/// 
/// [`tokio`]: https://docs.rs/tokio
#[derive(Debug)]
pub struct TcpSocket {
  socket: TokioTcpSocket,
}

impl From<TokioTcpSocket> for TcpSocket {
  fn from(socket: TokioTcpSocket) -> Self {
    Self { socket }
  }
}

impl From<TcpSocket> for TokioTcpSocket {
  fn from(socket: TcpSocket) -> Self {
    socket.socket
  }
}

impl_as!(TcpSocket.socket);

impl TryFrom<std::net::TcpStream> for TcpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
    Ok(TokioTcpSocket::from_std_stream(stream).into())
  }
}

impl crate::TcpSocket for TcpSocket {
  type Stream = super::TcpStream;
  type Listener = super::TcpListener;
  type Runtime = super::TokioRuntime;

  fn new_v4() -> io::Result<Self> {
    TokioTcpSocket::new_v4().map(Into::into)
  }

  fn new_v6() -> io::Result<Self> {
    TokioTcpSocket::new_v6().map(Into::into)
  }

  fn bind(&self, addr: std::net::SocketAddr) -> io::Result<()> {
    self.socket.bind(addr)
  }

  async fn connect(self, addr: std::net::SocketAddr) -> io::Result<Self::Stream> {
    self.socket.connect(addr).await.map(Into::into)
  }

  fn listen(self, backlog: u32) -> io::Result<Self::Listener> {
    self.socket.listen(backlog).map(Into::into)
  }

  fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
    self.socket.local_addr()
  }

  fn set_keepalive(&self, keepalive: bool) -> io::Result<()> {
    self.socket.set_keepalive(keepalive)
  }

  fn keepalive(&self) -> io::Result<bool> {
    self.socket.keepalive()
  }

  fn set_reuseaddr(&self, reuseaddr: bool) -> io::Result<()> {
    self.socket.set_reuseaddr(reuseaddr)
  }

  fn reuseaddr(&self) -> io::Result<bool> {
    self.socket.reuseaddr()
  }

  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))
  )]
  fn set_reuseport(&self, reuesport: bool) -> io::Result<()> {
    self.socket.set_reuseport(reuesport)
  }

  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))
  )]
  fn reuseport(&self) -> io::Result<bool> {
    self.socket.reuseport()
  }

  fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
    self.socket.set_nodelay(nodelay)
  }

  fn nodelay(&self) -> io::Result<bool> {
    self.socket.nodelay()
  }

  fn set_linger(&self, dur: Option<std::time::Duration>) -> io::Result<()> {
    self.socket.set_linger(dur)
  }

  fn linger(&self) -> io::Result<Option<std::time::Duration>> {
    self.socket.linger()
  }

  fn set_send_buffer_size(&self, size: u32) -> io::Result<()> {
    self.socket.set_send_buffer_size(size)
  }

  fn send_buffer_size(&self) -> io::Result<u32> {
    self.socket.send_buffer_size()
  }

  fn set_recv_buffer_size(&self, size: u32) -> io::Result<()> {
    self.socket.set_recv_buffer_size(size)
  }

  fn recv_buffer_size(&self) -> io::Result<u32> {
    self.socket.recv_buffer_size()
  }

  #[cfg(not(any(
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "haiku"
  )))]
  #[cfg_attr(
    docsrs,
    doc(cfg(not(any(
      target_os = "fuchsia",
      target_os = "redox",
      target_os = "solaris",
      target_os = "illumos",
      target_os = "haiku"
    ))))
  )]
  fn tos(&self) -> io::Result<u32> {
    self.socket.tos()
  }

  #[cfg(not(any(
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "haiku"
  )))]
  #[cfg_attr(
    docsrs,
    doc(cfg(not(any(
      target_os = "fuchsia",
      target_os = "redox",
      target_os = "solaris",
      target_os = "illumos",
      target_os = "haiku"
    ))))
  )]
  fn set_tos(&self, tos: u32) -> io::Result<()> {
    self.socket.set_tos(tos)
  }

  // TODO(al8n): uncomment below code when rustix supports SO_BINDTODEVICE
  // /// Gets the value for the SO_BINDTODEVICE option on this socket
  // ///
  // /// This value gets the socket binded device’s interface name.
  // #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
  // fn device(&self) -> io::Result<Option<Vec<u8>>> {
  //   self.socket.device()
  // }

  // /// Sets the value for the `SO_BINDTODEVICE` option on this socket
  // ///
  // /// If a socket is bound to an interface, only packets received from that
  // /// particular interface are processed by the socket. Note that this only
  // /// works for some socket types, particularly `AF_INET` sockets.
  // ///
  // /// If `interface` is `None` or an empty string it removes the binding.
  // #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
  // fn bind_device(&self, interface: Option<&[u8]>) -> io::Result<()> {
  //   self.socket.bind_device(interface)
  // }
}
