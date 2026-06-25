#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

use agnostic_net::{Net, UdpSocket, runtime::RuntimeLite};
use futures_util::future::FutureExt;
use std::{future::Future, io, marker::PhantomData, net::SocketAddr, pin::Pin, time::Duration};

pub use hickory_resolver::config::*;
use hickory_resolver::{
  Resolver,
  net::runtime::{DnsTcpStream, DnsUdpSocket, RuntimeProvider, Spawn, Time},
};

pub use agnostic_net as net;

#[cfg(test)]
mod tests;

/// Agnostic async DNS resolver
pub type Dns<N> = Resolver<AsyncConnectionProvider<N>>;

/// Async spawner
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct AsyncSpawn<N> {
  _marker: PhantomData<N>,
}

impl<N> Clone for AsyncSpawn<N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<N> Copy for AsyncSpawn<N> {}

impl<N: Net> Spawn for AsyncSpawn<N> {
  fn spawn_bg(&mut self, future: impl Future<Output = ()> + Send + 'static) {
    <N::Runtime as RuntimeLite>::spawn_detach(future);
  }
}

/// Defines which async runtime that handles IO and timers.
pub struct AsyncRuntimeProvider<N> {
  runtime: AsyncSpawn<N>,
}

impl<N> Default for AsyncRuntimeProvider<N> {
  fn default() -> Self {
    Self::new()
  }
}

impl<N> AsyncRuntimeProvider<N> {
  /// Create a new `AsyncRuntimeProvider`.
  pub fn new() -> Self {
    Self {
      runtime: AsyncSpawn {
        _marker: PhantomData,
      },
    }
  }
}

impl<N> Clone for AsyncRuntimeProvider<N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<N> Copy for AsyncRuntimeProvider<N> {}

/// Timer implementation for the dns.
pub struct Timer<N>(PhantomData<N>);

#[async_trait::async_trait]
impl<N: Net> Time for Timer<N> {
  async fn delay_for(duration: Duration) {
    let _ = <N::Runtime as RuntimeLite>::sleep(duration).await;
  }

  async fn timeout<F: 'static + Future + Send>(
    duration: Duration,
    future: F,
  ) -> Result<F::Output, std::io::Error> {
    <N::Runtime as RuntimeLite>::timeout(duration, future)
      .await
      .map_err(Into::into)
  }
}

/// DNS time
#[derive(Clone, Copy, Debug)]
pub struct AgnosticTime<N>(PhantomData<N>);

#[async_trait::async_trait]
impl<N> Time for AgnosticTime<N>
where
  N: Net,
{
  async fn delay_for(duration: Duration) {
    <N::Runtime as RuntimeLite>::sleep(duration).await;
  }

  async fn timeout<F: 'static + Future + Send>(
    duration: Duration,
    future: F,
  ) -> Result<F::Output, std::io::Error> {
    <N::Runtime as RuntimeLite>::timeout(duration, future)
      .await
      .map_err(Into::into)
  }
}

/// DNS tcp
#[doc(hidden)]
pub struct AsyncDnsTcp<N: Net>(N::TcpStream);

impl<N: Net> DnsTcpStream for AsyncDnsTcp<N> {
  type Time = AgnosticTime<N>;
}

impl<N: Net> AsyncDnsTcp<N> {
  async fn connect(addr: SocketAddr) -> std::io::Result<Self> {
    <N::TcpStream as agnostic_net::TcpStream>::connect(addr)
      .await
      .map(Self)
  }
}

impl<N: Net> futures_util::AsyncRead for AsyncDnsTcp<N> {
  fn poll_read(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &mut [u8],
  ) -> std::task::Poll<io::Result<usize>> {
    futures_util::AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
  }
}

impl<N: Net> futures_util::AsyncWrite for AsyncDnsTcp<N> {
  fn poll_write(
    mut self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
    futures_util::AsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
  }

  fn poll_flush(
    mut self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    futures_util::AsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
  }

  fn poll_close(
    mut self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    futures_util::AsyncWrite::poll_close(Pin::new(&mut self.0), cx)
  }
}

/// DNS udp
pub struct AsyncDnsUdp<N: Net>(N::UdpSocket);

impl<N: Net> AsyncDnsUdp<N> {
  async fn bind(addr: SocketAddr) -> std::io::Result<Self> {
    <N::UdpSocket as UdpSocket>::bind(addr).await.map(Self)
  }
}

impl<N: Net> DnsUdpSocket for AsyncDnsUdp<N>
where
  N::UdpSocket: Send + Sync,
{
  type Time = AgnosticTime<N>;

  fn poll_recv_from(
    &self,
    cx: &mut std::task::Context<'_>,
    buf: &mut [u8],
  ) -> std::task::Poll<io::Result<(usize, SocketAddr)>> {
    self.0.poll_recv_from(cx, buf)
  }

  fn poll_send_to(
    &self,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> std::task::Poll<io::Result<usize>> {
    self.0.poll_send_to(cx, buf, target)
  }
}

impl<N: Net> RuntimeProvider for AsyncRuntimeProvider<N>
where
  N::UdpSocket: Send + Sync,
  <N::UdpSocket as UdpSocket>::Bind<SocketAddr>: Send,
{
  type Handle = AsyncSpawn<N>;

  type Timer = Timer<N>;

  type Udp = AsyncDnsUdp<N>;

  type Tcp = AsyncDnsTcp<N>;

  fn create_handle(&self) -> Self::Handle {
    self.runtime
  }

  fn connect_tcp(
    &self,
    server_addr: SocketAddr,
    _bind_addr: Option<SocketAddr>,
    timeout: Option<Duration>,
  ) -> std::pin::Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
    async move {
      let connect_future = AsyncDnsTcp::connect(server_addr);
      match timeout {
        Some(duration) => <N::Runtime as RuntimeLite>::timeout(duration, connect_future)
          .await
          .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))?,
        None => connect_future.await,
      }
    }
    .boxed()
  }

  fn bind_udp(
    &self,
    local_addr: SocketAddr,
    _server_addr: SocketAddr,
  ) -> std::pin::Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
    AsyncDnsUdp::bind(local_addr).boxed()
  }
}

/// Creates DNS connections with the help of [`AsyncRuntimeProvider`].
///
/// Since hickory 0.26 every [`RuntimeProvider`] is also a
/// [`ConnectionProvider`](hickory_resolver::ConnectionProvider), so this is an alias for
/// [`AsyncRuntimeProvider`].
pub type AsyncConnectionProvider<N> = AsyncRuntimeProvider<N>;

#[cfg(all(unix, not(target_os = "android"), not(target_vendor = "apple")))]
pub use dns_util::read_resolv_conf;

#[cfg(all(unix, not(target_os = "android"), not(target_vendor = "apple")))]
pub use hickory_resolver::system_conf::parse_resolv_conf;
#[cfg(any(unix, windows))]
pub use hickory_resolver::system_conf::read_system_conf;

#[cfg(all(unix, not(target_os = "android"), not(target_vendor = "apple")))]
mod dns_util {
  use std::{io, path::Path};

  use hickory_resolver::config::{ResolverConfig, ResolverOpts};

  /// Read the DNS configuration from a file.
  pub fn read_resolv_conf<P: AsRef<Path>>(path: P) -> io::Result<(ResolverConfig, ResolverOpts)> {
    std::fs::read_to_string(path).and_then(|conf| {
      hickory_resolver::system_conf::parse_resolv_conf(conf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    })
  }
}
