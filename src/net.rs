use std::{
  io,
  net::SocketAddr,
  task::{Context, Poll},
  time::Duration,
};

use super::Runtime;
use futures_util::Future;

mod to_socket_addr;

/// Agnostic async DNS provider.
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub mod dns;

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
  fn to_socket_addrs(&self) -> Self::Future;
}

/// An abstraction layer for TCP listener.
pub trait TcpListener: Unpin + Send + Sync + 'static {
  /// The async runtime.
  type Runtime: Runtime;
  /// Stream of incoming connections.
  type Stream: TcpStream<Runtime = Self::Runtime>;

  /// Creates a new TcpListener, which will be bound to the specified address.
  ///
  /// The returned listener is ready for accepting connections.
  ///
  /// Binding with a port number of 0 will request that the OS assigns a port
  /// to this listener. The port allocated can be queried via the `local_addr`
  /// method.
  ///
  /// The address type can be any implementor of the [`ToSocketAddrs`] trait.
  /// If `addr` yields multiple addresses, bind will be attempted with each of
  /// the addresses until one succeeds and returns the listener. If none of
  /// the addresses succeed in creating a listener, the error returned from
  /// the last attempt (the last address) is returned.
  ///
  /// This function sets the `SO_REUSEADDR` option on the socket.
  fn bind<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  /// Accepts a new incoming connection from this listener.
  ///
  /// This function will yield once a new TCP connection is established. When established,
  /// the corresponding [`TcpStream`] and the remote peer's address will be returned.
  fn accept(&self) -> impl Future<Output = io::Result<(Self::Stream, SocketAddr)>> + Send;

  /// Returns the local address that this listener is bound to.
  ///
  /// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
  fn local_addr(&self) -> io::Result<SocketAddr>;
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

#[doc(hidden)]
#[cfg(not(feature = "tokio-compat"))]
pub trait IORead: futures_util::io::AsyncRead {}

#[cfg(not(feature = "tokio-compat"))]
impl<T: futures_util::io::AsyncRead> IORead for T {}

#[doc(hidden)]
#[cfg(feature = "tokio-compat")]
pub trait IORead: tokio::io::AsyncRead + futures_util::io::AsyncRead {}

#[cfg(feature = "tokio-compat")]
impl<T: tokio::io::AsyncRead + futures_util::io::AsyncRead> IORead for T {}

#[doc(hidden)]
#[cfg(not(feature = "tokio-compat"))]
pub trait IOWrite: futures_util::io::AsyncWrite {}

#[cfg(not(feature = "tokio-compat"))]
impl<T: futures_util::io::AsyncWrite> IOWrite for T {}

#[doc(hidden)]
#[cfg(feature = "tokio-compat")]
pub trait IOWrite: tokio::io::AsyncWrite + futures_util::io::AsyncWrite {}

#[cfg(feature = "tokio-compat")]
impl<T: tokio::io::AsyncWrite + futures_util::io::AsyncWrite> IOWrite for T {}

pub trait TcpStreamOwnedReadHalf: IORead + Unpin + Send + Sync + 'static {
  type Runtime: Runtime;

  fn local_addr(&self) -> io::Result<SocketAddr>;

  fn peer_addr(&self) -> io::Result<SocketAddr>;
}

pub trait TcpStreamOwnedWriteHalf: IOWrite + Unpin + Send + Sync + 'static {
  type Runtime: Runtime;

  fn forget(self);

  fn local_addr(&self) -> io::Result<SocketAddr>;

  fn peer_addr(&self) -> io::Result<SocketAddr>;
}

pub trait TcpStream: IO + Unpin + Send + Sync + 'static {
  type Runtime: Runtime;
  type OwnedReadHalf: TcpStreamOwnedReadHalf;
  type OwnedWriteHalf: TcpStreamOwnedWriteHalf;
  /// Error indicating that two halves were not from the same socket, and thus could not be reunited.
  type ReuniteError: std::error::Error + Unpin + Send + Sync + 'static;

  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  fn local_addr(&self) -> io::Result<SocketAddr>;

  fn peer_addr(&self) -> io::Result<SocketAddr>;

  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  fn ttl(&self) -> io::Result<u32>;

  fn set_nodelay(&self, nodelay: bool) -> io::Result<()>;

  fn nodelay(&self) -> io::Result<bool>;

  fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf);

  fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()>;

  /// Attempts to put the two halves of a TcpStream back together and recover the original socket. Succeeds only if the two halves originated from the same call to [`into_split`][TcpStream::into_split].
  fn reunite(
    read: Self::OwnedReadHalf,
    write: Self::OwnedWriteHalf,
  ) -> Result<Self, Self::ReuniteError>
  where
    Self: Sized;
}

pub trait UdpSocket: Unpin + Send + Sync + 'static {
  type Runtime: Runtime;

  fn bind<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
  ) -> impl Future<Output = io::Result<()>> + Send;

  fn recv(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send;

  fn recv_from(
    &self,
    buf: &mut [u8],
  ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send;

  fn send(&self, buf: &[u8]) -> impl Future<Output = io::Result<usize>> + Send;

  fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> impl Future<Output = io::Result<usize>> + Send;

  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  fn ttl(&self) -> io::Result<u32>;

  fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;

  fn broadcast(&self) -> io::Result<bool>;

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

  fn local_addr(&self) -> io::Result<SocketAddr>;
}

#[cfg(feature = "net")]
pub trait Net {
  type TcpListener: TcpListener<Stream = Self::TcpStream>;
  type TcpStream: TcpStream;
  type UdpSocket: UdpSocket;

  #[cfg(feature = "quinn")]
  type Quinn: quinn::Runtime + Default;
}

#[cfg(all(unix, feature = "socket2"))]
#[inline]
pub(crate) fn set_read_buffer(fd: std::os::fd::RawFd, mut size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::fd::FromRawFd;

  // Safety: the fd we created from the socket is just created, so it is a valid and open file descriptor
  let socket = unsafe { Socket::from_raw_fd(fd) };
  socket.set_nonblocking(true)?;
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
  socket.set_nonblocking(true)?;
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
  socket.set_nonblocking(true)?;
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
  socket.set_nonblocking(true)?;
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
