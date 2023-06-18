use std::{
  io,
  net::SocketAddr,
  task::{Context, Poll},
  time::Duration,
};

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

  /// Attempts to receive a single datagram on the socket.
  ///
  /// Note that on multiple calls to a `poll_*` method in the recv direction, only the
  /// `Waker` from the `Context` passed to the most recent call will be scheduled to
  /// receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not ready to read
  /// * `Poll::Ready(Ok(addr))` reads data from `addr` into `ReadBuf` if the socket is ready
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  ///
  /// # Notes
  /// Note that the socket address **cannot** be implicitly trusted, because it is relatively
  /// trivial to send a UDP datagram with a spoofed origin in a [packet injection attack].
  /// Because UDP is stateless and does not validate the origin of a packet,
  /// the attacker does not need to be able to intercept traffic in order to interfere.
  /// It is important to be aware of this when designing your application-level protocol.
  ///
  /// [packet injection attack]: https://en.wikipedia.org/wiki/Packet_injection
  fn poll_recv_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>>;

  /// Attempts to send data on the socket to a given address.
  ///
  /// Note that on multiple calls to a `poll_*` method in the send direction, only the
  /// `Waker` from the `Context` passed to the most recent call will be scheduled to
  /// receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not ready to write
  /// * `Poll::Ready(Ok(n))` `n` is the number of bytes sent.
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>>;
}

#[cfg(feature = "net")]
pub trait Net {
  type TcpListener: TcpListener<Stream = Self::TcpStream>;
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
