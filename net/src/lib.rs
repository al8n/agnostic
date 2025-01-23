use std::net::SocketAddr;

use agnostic_lite::{cfg_async_std, cfg_smol, cfg_tokio, RuntimeLite};
use futures_util::Future;

/// Traits, helpers, and type definitions for asynchronous I/O functionality.
pub use agnostic_io as io;

mod to_socket_addrs;

/// Agnostic async DNS provider.
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub mod dns;

cfg_tokio!(
  /// Network abstractions for [`tokio`] runtime
  ///
  /// [`tokio`]: https://docs.rs/tokio
  pub mod tokio;
);

cfg_smol!(
  /// Network abstractions for [`smol`] runtime
  ///
  /// [`smol`]: https://docs.rs/smol
  pub mod smol;
);

cfg_async_std!(
  /// Network abstractions for [`async-std`] runtime
  ///
  /// [`async-std`]: https://docs.rs/async-std
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
}

#[cfg(all(unix, any(feature = "tokio", feature = "smol", feature = "async-std")))]
#[inline]
pub(crate) fn set_read_buffer(fd: std::os::fd::RawFd, size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::fd::FromRawFd;

  unsafe { Socket::from_raw_fd(fd) }.set_recv_buffer_size(size)
}

#[cfg(all(unix, any(feature = "tokio", feature = "smol", feature = "async-std")))]
#[inline]
pub(crate) fn set_write_buffer(fd: std::os::fd::RawFd, size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::fd::FromRawFd;
  unsafe { Socket::from_raw_fd(fd).set_send_buffer_size(size) }
}

#[cfg(all(
  windows,
  any(feature = "tokio", feature = "smol", feature = "async-std")
))]
#[inline]
pub(crate) fn set_read_buffer(fd: std::os::windows::io::RawSocket, size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::windows::io::FromRawSocket;

  unsafe { Socket::from_raw_socket(fd) }.set_recv_buffer_size(size)
}

#[cfg(all(
  windows,
  any(feature = "tokio", feature = "smol", feature = "async-std")
))]
#[inline]
pub(crate) fn set_write_buffer(fd: std::os::windows::io::RawSocket, size: usize) -> io::Result<()> {
  use socket2::Socket;
  use std::os::windows::io::FromRawSocket;

  unsafe { Socket::from_raw_socket(fd) }.set_send_buffer_size(size)
}
