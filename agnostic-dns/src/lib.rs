#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

use agnostic_net::{runtime::RuntimeLite, Net, TcpSocket, UdpSocket};
use futures_util::future::FutureExt;
use hickory_proto::{
  ProtoError,
  runtime::{RuntimeProvider, Spawn, Time},
};
#[cfg(feature = "quic")]
use hickory_proto::runtime::QuicSocketBinder;
use hickory_resolver::{
  Resolver,
  name_server::{ConnectionProvider, GenericConnector},
};

use std::{future::Future, io, marker::PhantomData, net::SocketAddr, pin::Pin, time::Duration};

pub use hickory_resolver::config::*;
pub use agnostic_net as net;

#[cfg(test)]
mod tests;

/// Agnostic aysnc DNS resolver
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
  fn spawn_bg<F>(&mut self, future: F)
  where
    F: Future<Output = Result<(), ProtoError>> + Send + 'static,
  {
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

impl<N: Net> hickory_proto::tcp::DnsTcpStream for AsyncDnsTcp<N> {
  type Time = AgnosticTime<N>;
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

impl<N: Net> hickory_proto::udp::DnsUdpSocket for AsyncDnsUdp<N> {
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

impl<N: Net> RuntimeProvider for AsyncRuntimeProvider<N> {
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
    bind_addr: Option<SocketAddr>,
    wait_for: Option<Duration>,
  ) -> Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
    Box::pin(async move {
      let socket = match server_addr {
        SocketAddr::V4(_) => <N::TcpSocket as TcpSocket>::new_v4(),
        SocketAddr::V6(_) => <N::TcpSocket as TcpSocket>::new_v6(),
      }?;

      if let Some(bind_addr) = bind_addr {
        socket.bind(bind_addr)?;
      }

      socket.set_nodelay(true)?;
      let future = socket.connect(server_addr);
      let wait_for = wait_for.unwrap_or(Duration::from_secs(5));
      match <N::Runtime as RuntimeLite>::timeout(wait_for, future).await {
        Ok(Ok(socket)) => Ok(AsyncDnsTcp(socket)),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(io::Error::new(
          io::ErrorKind::TimedOut,
          format!("connection to {server_addr:?} timed out after {wait_for:?}"),
        )),
      }
    })
  }

  fn bind_udp(
    &self,
    local_addr: SocketAddr,
    _server_addr: SocketAddr,
  ) -> std::pin::Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
    AsyncDnsUdp::bind(local_addr).boxed()
  }

  fn quic_binder(&self) -> Option<&dyn hickory_proto::runtime::QuicSocketBinder> {
    #[cfg(feature = "quic")]
    {

    }

    None
  }
}

// impl<> QuicSocketBinder for AsyncRuntimeQuinnSocketBinder {

// }

/// Create `DnsHandle` with the help of `AsyncRuntimeProvider`.
pub struct AsyncConnectionProvider<N: Net> {
  runtime_provider: AsyncRuntimeProvider<N>,
  connection_provider: GenericConnector<AsyncRuntimeProvider<N>>,
}

impl<N: Net> Default for AsyncConnectionProvider<N> {
  fn default() -> Self {
    Self::new()
  }
}

impl<N: Net> AsyncConnectionProvider<N> {
  /// Create a new `AsyncConnectionProvider`.
  pub fn new() -> Self {
    Self {
      runtime_provider: AsyncRuntimeProvider::new(),
      connection_provider: GenericConnector::new(AsyncRuntimeProvider::new()),
    }
  }
}

impl<N: Net> Clone for AsyncConnectionProvider<N> {
  fn clone(&self) -> Self {
    Self {
      runtime_provider: self.runtime_provider,
      connection_provider: self.connection_provider.clone(),
    }
  }
}

impl<N: Net> ConnectionProvider for AsyncConnectionProvider<N> {
  type Conn = <GenericConnector<AsyncRuntimeProvider<N>> as ConnectionProvider>::Conn;
  type FutureConn = <GenericConnector<AsyncRuntimeProvider<N>> as ConnectionProvider>::FutureConn;
  type RuntimeProvider = AsyncRuntimeProvider<N>;

  fn new_connection(
    &self,
    config: &NameServerConfig,
    options: &ResolverOpts,
  ) -> Result<Self::FutureConn, io::Error> {
    self.connection_provider.new_connection(config, options)
  }
}

#[cfg(unix)]
pub use dns_util::read_resolv_conf;

#[cfg(unix)]
pub use hickory_resolver::system_conf::parse_resolv_conf;
pub use hickory_resolver::system_conf::read_system_conf;

#[cfg(unix)]
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
