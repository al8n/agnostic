use rustix::fd::OwnedFd;

use std::{io, os::fd::{FromRawFd, IntoRawFd}};
use crate::os;


/// A [`TcpSocket`](super::super::TcpSocket) implementation for [`tokio`] runtime.
/// 
/// [`tokio`]: https://docs.rs/tokio
#[derive(Debug)]
pub struct TcpSocket {
  socket: OwnedFd,
}

impl_as!(TcpSocket.socket);

impl TryFrom<std::net::TcpStream> for TcpSocket {
  type Error = io::Error;

  #[inline]
  fn try_from(stream: std::net::TcpStream) -> Result<Self, Self::Error> {
   
    Ok(Self {
      socket: unsafe { OwnedFd::from_raw_fd(stream.into_raw_fd()) },
    })
  }
}

impl crate::TcpSocket for TcpSocket {
  type Stream = super::TcpStream;

  type Listener = super::TcpListener;

  type Runtime = super::SmolRuntime;

  fn new_v4() -> io::Result<Self> {
    os::socket_v4().map(|socket| Self { socket })
  }

  fn new_v6() -> io::Result<Self> {
    os::socket_v6().map(|socket| Self { socket })
  }

  fn bind(&self, addr: std::net::SocketAddr) -> io::Result<()> {
    os::bind(&self.socket, addr)
  }

  async fn connect(self, addr: std::net::SocketAddr) -> io::Result<Self::Stream> {
    os::connect(&self.socket, addr).and_then(|_| {
      std::net::TcpStream::from(self.socket).try_into()
    })
  }

  fn listen(self, backlog: u32) -> io::Result<Self::Listener> {
    os::listen(&self.socket, backlog).and_then(|_| {
      std::net::TcpListener::from(self.socket).try_into()
    })
  }

  fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
    os::local_addr(&self.socket)
  }

  fn set_keepalive(&self, keepalive: bool) -> io::Result<()> {
    os::set_keepalive(&self.socket, keepalive)
  }

  fn keepalive(&self) -> io::Result<bool> {
    os::keepalive(&self.socket)
  }

  fn set_reuseaddr(&self, reuseaddr: bool) -> io::Result<()> {
    os::set_reuseaddr(&self.socket, reuseaddr)
  }

  fn reuseaddr(&self) -> io::Result<bool> {
    os::reuseaddr(&self.socket)
  }

  fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
    os::set_nodelay(&self.socket, nodelay)
  }

  fn nodelay(&self) -> io::Result<bool> {
    os::nodelay(&self.socket)
  }

  fn set_linger(&self, dur: Option<std::time::Duration>) -> io::Result<()> {
    os::set_linger(&self.socket, dur)
  }

  fn linger(&self) -> io::Result<Option<std::time::Duration>> {
    os::linger(&self.socket)
  }

  fn set_send_buffer_size(&self, size: u32) -> io::Result<()> {
    os::set_send_buffer_size(&self.socket, size as usize)
  }

  fn send_buffer_size(&self) -> io::Result<u32> {
    os::send_buffer_size(&self.socket).map(|size| size as u32)
  }

  fn set_recv_buffer_size(&self, size: u32) -> io::Result<()> {
    os::set_recv_buffer_size(&self.socket, size as usize)
  }

  fn recv_buffer_size(&self) -> io::Result<u32> {
    os::recv_buffer_size(&self.socket).map(|size| size as u32)
  }

  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))
  )]
  fn set_reuseport(&self, reuesport: bool) -> io::Result<()> {
    os::set_reuseport(&self.socket, reuesport)
  }

  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))
  )]
  fn reuseport(&self) -> io::Result<bool> {
    os::reuseport(&self.socket)
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
    os::tos(&self.socket).map(|tos| tos as u32)
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
    os::set_tos(&self.socket, tos as u8) 
  }

  // TODO(al8n): uncomment below code when rustix supports SO_BINDTODEVICE
  // /// Gets the value for the SO_BINDTODEVICE option on this socket
  // ///
  // /// This value gets the socket binded device’s interface name.
  // #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
  // fn device(&self) -> io::Result<Option<Vec<u8>>> {
  //   todo!()
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
  //   todo!()
  // }
}
