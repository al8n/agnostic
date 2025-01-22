use std::{
  io,
  net::SocketAddr,
};

use agnostic_lite::{cfg_async_std, cfg_smol, cfg_tokio, RuntimeLite};
use futures_util::Future;

mod to_socket_addrs;

/// Agnostic async DNS provider.
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub mod dns;

cfg_tokio!(
  /// Network abstractions for [`tokio`](::tokio) runtime
  pub mod tokio;
);

cfg_smol!(
  /// Network abstractions for [`smol`](::smol) runtime
  pub mod smol;
);

cfg_async_std!(
  /// Network abstractions for [`async-std`](::async_std) runtime
  pub mod async_std;
);

mod tcp;
mod udp;

pub use tcp::*;
pub use udp::*;

/// Converts or resolves without blocking base on your async runtime to one or more `SocketAddr` values.
///
/// # DNS
///
/// Implementations of `ToSocketAddrs<R: RuntimeLite>` for string types require a DNS lookup.
pub trait ToSocketAddrs<R: RuntimeLite>: Send + Sync {
  /// Returned iterator over socket addresses which this type may correspond to.
  type Iter: Iterator<Item = SocketAddr> + Send + 'static;
  /// The future type used to resolve addresses.
  type Future: Future<Output = io::Result<Self::Iter>> + Send + 'static;

  /// Converts this object to an iterator of resolved `SocketAddr`s.
  ///
  /// The returned iterator may not actually yield any values depending on the outcome of any
  /// resolution performed.
  ///
  /// Note that this function may block a backend thread while resolution is performed.
  fn to_socket_addrs(&self) -> Self::Future;
}

#[doc(hidden)]
#[cfg(not(feature = "tokio-compat"))]
pub trait IO: futures_util::io::AsyncRead + futures_util::io::AsyncWrite {}

#[cfg(not(feature = "tokio-compat"))]
impl<T: futures_util::io::AsyncRead + futures_util::io::AsyncWrite> IO for T {}

#[doc(hidden)]
#[cfg(feature = "tokio-compat")]
pub trait IO:
  ::tokio::io::AsyncRead
  + ::tokio::io::AsyncWrite
  + futures_util::io::AsyncRead
  + futures_util::io::AsyncWrite
{
}

#[cfg(feature = "tokio-compat")]
impl<
    T: ::tokio::io::AsyncRead
      + ::tokio::io::AsyncWrite
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
pub trait IORead: ::tokio::io::AsyncRead + futures_util::io::AsyncRead {}

#[cfg(feature = "tokio-compat")]
impl<T: ::tokio::io::AsyncRead + futures_util::io::AsyncRead> IORead for T {}

#[doc(hidden)]
#[cfg(not(feature = "tokio-compat"))]
pub trait IOWrite: futures_util::io::AsyncWrite {}

#[cfg(not(feature = "tokio-compat"))]
impl<T: futures_util::io::AsyncWrite> IOWrite for T {}

#[doc(hidden)]
#[cfg(feature = "tokio-compat")]
pub trait IOWrite: ::tokio::io::AsyncWrite + futures_util::io::AsyncWrite {}

#[cfg(feature = "tokio-compat")]
impl<T: ::tokio::io::AsyncWrite + futures_util::io::AsyncWrite> IOWrite for T {}


/// An abstraction layer for the async runtime's network.
pub trait Net: Unpin + Send + Sync + 'static {
  /// The runtime type
  type Runtime: RuntimeLite;

  /// The [`TcpListener`] implementation
  type TcpListener: TcpListener<Stream = Self::TcpStream>;
  /// The [`TcpStream`] implementation
  type TcpStream: TcpStream;
  /// The [`UdpSocket`] implementation
  type UdpSocket: UdpSocket;

  /// The [`quinn`] runtime
  #[cfg(feature = "quinn")]
  #[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
  type Quinn: quinn::Runtime + Default;
}

#[cfg(all(
  unix,
  feature = "net",
  any(feature = "tokio", feature = "smol", feature = "async-std")
))]
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

#[cfg(all(
  unix,
  feature = "net",
  any(feature = "tokio", feature = "smol", feature = "async-std")
))]
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

#[cfg(all(
  windows,
  feature = "net",
  any(feature = "tokio", feature = "smol", feature = "async-std")
))]
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

#[cfg(all(
  windows,
  feature = "net",
  any(feature = "tokio", feature = "smol", feature = "async-std")
))]
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
