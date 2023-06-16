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

#[async_trait::async_trait]
pub trait TcpListener: Send {
  type Runtime: Runtime;
  type Stream: TcpStream<Runtime = Self::Runtime>;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized;

  async fn accept(&self) -> io::Result<(Self::Stream, SocketAddr)>;

  fn local_addr(&self) -> io::Result<SocketAddr>;

  fn set_write_timeout(&self, timeout: Option<Duration>);

  fn write_timeout(&self) -> Option<Duration>;

  fn set_read_timeout(&self, timeout: Option<Duration>);

  fn read_timeout(&self) -> Option<Duration>;
}

#[doc(hidden)]
#[cfg(not(any(feature = "compat", feature = "tokio")))]
pub trait IO: futures_util::io::AsyncRead + futures_util::io::AsyncWrite {}

#[cfg(not(any(feature = "compat", feature = "tokio")))]
impl<T: futures_util::io::AsyncRead + futures_util::io::AsyncWrite> IO for T {}

#[doc(hidden)]
#[cfg(any(feature = "compat", feature = "tokio"))]
pub trait IO:
  tokio::io::AsyncRead
  + tokio::io::AsyncWrite
  + futures_util::io::AsyncRead
  + futures_util::io::AsyncWrite
{
}

#[cfg(any(feature = "compat", feature = "tokio"))]
impl<
    T: tokio::io::AsyncRead
      + tokio::io::AsyncWrite
      + futures_util::io::AsyncRead
      + futures_util::io::AsyncWrite,
  > IO for T
{
}

#[async_trait::async_trait]
pub trait TcpStream: IO {
  type Runtime: Runtime;

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized;

  async fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
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

#[async_trait::async_trait]
pub trait UdpSocket {
  type Runtime: Runtime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized;

  async fn bind_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
  where
    Self: Sized;

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> io::Result<()>;

  async fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
    timeout: Duration,
  ) -> io::Result<()>;

  async fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;

  async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;

  async fn send(&self, buf: &[u8]) -> io::Result<usize>;

  async fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> io::Result<usize>;

  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  fn ttl(&self) -> io::Result<u32>;

  fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;

  fn broadcast(&self) -> io::Result<bool>;
}

#[cfg(feature = "net")]
pub trait Net {
  type TcpListener: TcpListener;
  type TcpStream: TcpStream;
  type UdpSocket: UdpSocket;
}
