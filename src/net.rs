use std::net::SocketAddr;

use super::Runtime;
use futures_util::Future;

mod to_socket_addr;

/// Converts or resolves without blocking base on your async runtime to one or more `SocketAddr` values.
///
/// # DNS
///
/// Implementations of `ToSocketAddrs<R: Runtime>` for string types require a DNS lookup.
#[cfg(feature = "net")]
pub trait ToSocketAddrs<R: Runtime> {
  /// Returned iterator over socket addresses which this type may correspond to.
  type Iter: Iterator<Item = SocketAddr> + Send + 'static;
  type Future: Future<Output = std::io::Result<Self::Iter>> + 'static;

  /// Converts this object to an iterator of resolved `SocketAddr`s.
  ///
  /// The returned iterator may not actually yield any values depending on the outcome of any
  /// resolution performed.
  ///
  /// Note that this function may block a backend thread while resolution is performed.
  fn to_socket_addrs(&self, runtime: &R) -> Self::Future;
}

#[cfg(feature = "net")]
pub trait Net {
  type TcpListener;
  type TcpStream;
  type UdpSocket;
}
