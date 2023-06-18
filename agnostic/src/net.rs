use std::{io, net::SocketAddr, time::Duration};

use super::Runtime;
use futures_util::Future;

mod to_socket_addr;

/// Converts or resolves without blocking base on your async runtime to one or more `SocketAddr` values.
///
/// # DNS
///
/// Implementations of `ToSocketAddrs<R: Runtime>` for string types require a DNS lookup.
#[cfg(feature = "net")]
pub trait ToSocketAddrs<R: Runtime>: Send + Sync {
  /// Returned iterator over socket addresses which this type may correspond to.
  type Iter: Iterator<Item = SocketAddr> + Send + 'static;
  type Future: Future<Output = io::Result<Self::Iter>> + Send + 'static;

  /// Converts this object to an iterator of resolved `SocketAddr`s.
  ///
  /// The returned iterator may not actually yield any values depending on the outcome of any
  /// resolution performed.
  ///
  /// Note that this function may block a backend thread while resolution is performed.
  fn to_socket_addrs(&self, runtime: &R) -> Self::Future;
}

#[cfg_attr(not(feature = "nightly"), async_trait::async_trait)]
pub trait TcpListener: Unpin + Send + Sync + 'static {
  type Runtime: Runtime;
  type Stream: TcpStream<Runtime = Self::Runtime>;

  #[cfg(not(feature = "nightly"))]
  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized;

  #[cfg(feature = "nightly")]
  fn bind<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send + 'a
  where
    Self: Sized;

  #[cfg(not(feature = "nightly"))]
  async fn accept(&self) -> io::Result<(Self::Stream, SocketAddr)>;

  #[cfg(feature = "nightly")]
  fn accept(&self) -> impl Future<Output = io::Result<(Self::Stream, SocketAddr)>> + Send + '_;

  fn local_addr(&self) -> io::Result<SocketAddr>;

  fn set_write_timeout(&self, timeout: Option<Duration>);

  fn write_timeout(&self) -> Option<Duration>;

  fn set_read_timeout(&self, timeout: Option<Duration>);

  fn read_timeout(&self) -> Option<Duration>;
}

#[doc(hidden)]
#[cfg(not(feature = "tokio-compat"))]
pub trait IO: futures_util::io::AsyncRead + futures_util::io::AsyncWrite {}

#[cfg(not(feature = "tokio-compat"))]
impl<T: futures_util::io::AsyncRead + futures_util::io::AsyncWrite> IO for T {}

#[doc(hidden)]
#[cfg(feature = "tokio-compat")]
pub trait IO:
  tokio::io::AsyncRead
  + tokio::io::AsyncWrite
  + futures_util::io::AsyncRead
  + futures_util::io::AsyncWrite
{
}

#[cfg(feature = "tokio-compat")]
impl<
    T: tokio::io::AsyncRead
      + tokio::io::AsyncWrite
      + futures_util::io::AsyncRead
      + futures_util::io::AsyncWrite,
  > IO for T
{
}

#[cfg_attr(not(feature = "nightly"), async_trait::async_trait)]
pub trait TcpStream: IO + Unpin + Send + Sync + 'static {
  type Runtime: Runtime;

  #[cfg(not(feature = "nightly"))]
  async fn connect<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized;

  #[cfg(feature = "nightly")]
  fn connect<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send + 'a
  where
    Self: Sized;

  #[cfg(not(feature = "nightly"))]
  async fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
  where
    Self: Sized;

  #[cfg(feature = "nightly")]
  fn connect_timeout<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    addr: A,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<Self>> + Send + 'a
  where
    Self: Sized;

  fn local_addr(&self) -> io::Result<SocketAddr>;

  fn peer_addr(&self) -> io::Result<SocketAddr>;

  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  fn ttl(&self) -> io::Result<u32>;

  fn set_nodelay(&self, nodelay: bool) -> io::Result<()>;

  fn nodelay(&self) -> io::Result<bool>;

  fn set_write_timeout(&self, timeout: Option<Duration>);

  fn write_timeout(&self) -> Option<Duration>;

  fn set_read_timeout(&self, timeout: Option<Duration>);

  fn read_timeout(&self) -> Option<Duration>;
}

#[cfg_attr(not(feature = "nightly"), async_trait::async_trait)]
pub trait UdpSocket: Unpin + Send + Sync + 'static {
  type Runtime: Runtime;

  #[cfg(not(feature = "nightly"))]
  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized;

  #[cfg(feature = "nightly")]
  fn bind<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send + 'a
  where
    Self: Sized;

  #[cfg(not(feature = "nightly"))]
  async fn bind_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
  where
    Self: Sized;

  #[cfg(feature = "nightly")]
  fn bind_timeout<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    addr: A,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<Self>> + Send + 'a
  where
    Self: Sized;

  #[cfg(not(feature = "nightly"))]
  async fn connect<A: ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> io::Result<()>;

  #[cfg(feature = "nightly")]
  fn connect<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    &'a self,
    addr: A,
  ) -> impl Future<Output = io::Result<()>> + Send + 'a;

  #[cfg(not(feature = "nightly"))]
  async fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
    timeout: Duration,
  ) -> io::Result<()>;

  #[cfg(feature = "nightly")]
  fn connect_timeout<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    &'a self,
    addr: A,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<()>> + Send + 'a;

  #[cfg(not(feature = "nightly"))]
  async fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;

  #[cfg(feature = "nightly")]
  fn recv<'a>(&'a self, buf: &'a mut [u8]) -> impl Future<Output = io::Result<usize>> + Send + 'a;

  #[cfg(not(feature = "nightly"))]
  async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;

  #[cfg(feature = "nightly")]
  fn recv_from<'a>(
    &'a self,
    buf: &'a mut [u8],
  ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send + 'a;

  #[cfg(not(feature = "nightly"))]
  async fn send(&self, buf: &[u8]) -> io::Result<usize>;

  #[cfg(feature = "nightly")]
  fn send<'a>(&'a self, buf: &'a [u8]) -> impl Future<Output = io::Result<usize>> + Send + 'a;

  #[cfg(not(feature = "nightly"))]
  async fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> io::Result<usize>;

  #[cfg(feature = "nightly")]
  fn send_to<'a, A: ToSocketAddrs<Self::Runtime> + 'a>(
    &'a self,
    buf: &'a [u8],
    target: A,
  ) -> impl Future<Output = io::Result<usize>> + Send + 'a;

  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  fn ttl(&self) -> io::Result<u32>;

  fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;

  fn broadcast(&self) -> io::Result<bool>;

  fn set_write_timeout(&self, timeout: Option<Duration>);

  fn write_timeout(&self) -> Option<Duration>;

  fn set_read_timeout(&self, timeout: Option<Duration>);

  fn read_timeout(&self) -> Option<Duration>;

  fn set_read_buffer(&self, size: usize) -> io::Result<()>;

  fn set_write_buffer(&self, size: usize) -> io::Result<()>;
}

/// **NOTE:** Temporary solution because of methods in Rust's trait cannot be async,
/// remove this when #[feature(async_fn_in_trait)] is stable
#[repr(transparent)]
pub struct AgnosticUdpSocket<T: UdpSocket> {
  socket: T,
}

impl<T: UdpSocket> AgnosticUdpSocket<T> {
  pub async fn bind<A: ToSocketAddrs<T::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    T::bind(addr).await.map(|socket| Self { socket })
  }

  pub async fn bind_timeout<A: ToSocketAddrs<T::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
  where
    Self: Sized,
  {
    T::bind_timeout(addr, timeout)
      .await
      .map(|socket| Self { socket })
  }

  pub async fn connect<A: ToSocketAddrs<T::Runtime>>(&self, addr: A) -> io::Result<()> {
    self.socket.connect(addr).await
  }

  pub async fn connect_timeout<A: ToSocketAddrs<T::Runtime>>(
    &self,
    addr: A,
    timeout: Duration,
  ) -> io::Result<()> {
    self.socket.connect_timeout(addr, timeout).await
  }

  pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
    self.socket.recv(buf).await
  }

  pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
    self.socket.recv_from(buf).await
  }

  pub async fn send(&self, buf: &[u8]) -> io::Result<usize> {
    self.socket.send(buf).await
  }

  pub async fn send_to<A: ToSocketAddrs<T::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> io::Result<usize> {
    self.socket.send_to(buf, target).await
  }

  pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.socket.set_ttl(ttl)
  }

  pub fn ttl(&self) -> io::Result<u32> {
    self.socket.ttl()
  }

  pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
    self.socket.set_broadcast(broadcast)
  }

  pub fn broadcast(&self) -> io::Result<bool> {
    self.socket.broadcast()
  }

  pub fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.socket.set_write_timeout(timeout)
  }

  pub fn write_timeout(&self) -> Option<Duration> {
    self.socket.write_timeout()
  }

  pub fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.socket.set_read_timeout(timeout)
  }

  pub fn read_timeout(&self) -> Option<Duration> {
    self.socket.read_timeout()
  }

  pub fn set_read_buffer(&self, size: usize) -> io::Result<()> {
    self.socket.set_read_buffer(size)
  }

  pub fn set_write_buffer(&self, size: usize) -> io::Result<()> {
    self.socket.set_write_buffer(size)
  }
}

#[cfg(feature = "net")]
pub trait Net {
  type TcpListener: TcpListener;
  type TcpStream: TcpStream;
  type UdpSocket: UdpSocket;
}

#[cfg(all(unix, feature = "socket2"))]
#[inline]
pub(crate) fn set_read_buffer(fd: std::os::fd::RawFd, mut size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::fd::FromRawFd;

  // Safety: the fd we created from the socket is just created, so it is a valid and open file descriptor
  let socket = unsafe { Socket::from_raw_fd(fd) };
  let mut err = None;

  while size > 0 {
    match socket.set_recv_buffer_size(size) {
      Ok(()) => return Ok(()),
      Err(e) => {
        err = Some(e);
        size /= 2;
      }
    }
  }
  // This is required to prevent double-closing the file descriptor.
  drop(socket);
  match err {
    Some(err) => Err(err),
    None => Ok(()),
  }
}

#[cfg(all(unix, feature = "socket2"))]
#[inline]
pub(crate) fn set_write_buffer(fd: std::os::fd::RawFd, mut size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::fd::FromRawFd;

  // Safety: the fd we created from the socket is just created, so it is a valid and open file descriptor
  let socket = unsafe { Socket::from_raw_fd(fd) };
  let mut err = None;

  while size > 0 {
    match socket.set_send_buffer_size(size) {
      Ok(()) => return Ok(()),
      Err(e) => {
        err = Some(e);
        size /= 2;
      }
    }
  }

  // This is required to prevent double-closing the file descriptor.
  drop(socket);
  match err {
    Some(err) => Err(err),
    None => Ok(()),
  }
}

#[cfg(all(windows, feature = "socket2"))]
#[inline]
pub(crate) fn set_read_buffer(
  fd: std::os::windows::io::RawSocket,
  mut size: usize,
) -> io::Result<()> {
  use socket2::Socket;
  use std::os::windows::io::FromRawSocket;

  // Safety: the fd we created from the socket is just created, so it is a valid and open file descriptor
  let socket = unsafe { Socket::from_raw_socket(fd) };
  let mut err = None;

  while size > 0 {
    match socket.set_recv_buffer_size(size) {
      Ok(()) => return Ok(()),
      Err(e) => {
        err = Some(e);
        size /= 2;
      }
    }
  }

  // This is required to prevent double-closing the file descriptor.
  drop(socket);
  match err {
    Some(err) => Err(err),
    None => Ok(()),
  }
}

#[cfg(all(windows, feature = "socket2"))]
#[inline]
pub(crate) fn set_write_buffer(
  fd: std::os::windows::io::RawSocket,
  mut size: usize,
) -> io::Result<()> {
  use socket2::Socket;
  use std::os::windows::io::FromRawSocket;

  // Safety: the fd we created from the socket is just created, so it is a valid and open file descriptor
  let socket = unsafe { Socket::from_raw_socket(fd) };
  let mut err = None;

  while size > 0 {
    match socket.set_send_buffer_size(size) {
      Ok(()) => return Ok(()),
      Err(e) => {
        err = Some(e);
        size /= 2;
      }
    }
  }

  // This is required to prevent double-closing the file descriptor.
  drop(socket);
  match err {
    Some(err) => Err(err),
    None => Ok(()),
  }
}
