use super::{Runtime, UdpSocket};
use futures_util::{future::FutureExt, select_biased};
use std::{future::Future, io, marker::PhantomData, net::SocketAddr, pin::Pin, time::Duration};

use trust_dns_proto::Time;
use trust_dns_resolver::{
  name_server::{ConnectionProvider, GenericConnector, RuntimeProvider, Spawn},
  AsyncResolver,
};

/// Agnostic aysnc DNS resolver
pub type Dns<R> = AsyncResolver<AsyncConnectionProvider<R>>;

#[doc(hidden)]
#[repr(transparent)]
pub struct AsyncSpawn<R: Runtime> {
  _marker: PhantomData<R>,
}

impl<R: Runtime> Clone for AsyncSpawn<R> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<R: Runtime> Copy for AsyncSpawn<R> {}

impl<R: Runtime> Spawn for AsyncSpawn<R> {
  fn spawn_bg<F>(&mut self, future: F)
  where
    F: Future<Output = Result<(), trust_dns_proto::error::ProtoError>> + Send + 'static,
  {
    R::spawn_detach(async move {
      let _ = future.await;
    });
  }
}

/// Defines which async runtime that handles IO and timers.
#[doc(hidden)]
pub struct AsyncRuntimeProvider<R: Runtime> {
  runtime: AsyncSpawn<R>,
}

impl<R: Runtime> Default for AsyncRuntimeProvider<R> {
  fn default() -> Self {
    Self::new()
  }
}

impl<R: Runtime> AsyncRuntimeProvider<R> {
  pub fn new() -> Self {
    Self {
      runtime: AsyncSpawn {
        _marker: PhantomData,
      },
    }
  }
}

impl<R> Clone for AsyncRuntimeProvider<R>
where
  R: Runtime,
{
  fn clone(&self) -> Self {
    *self
  }
}

impl<R> Copy for AsyncRuntimeProvider<R> where R: Runtime {}

pub struct Timer<R: Runtime>(PhantomData<R>);

#[async_trait::async_trait]
impl<R: Runtime> Time for Timer<R> {
  async fn delay_for(duration: Duration) {
    let _ = R::sleep(duration).await;
  }

  async fn timeout<F: 'static + Future + Send>(
    duration: Duration,
    future: F,
  ) -> Result<F::Output, std::io::Error> {
    select_biased! {
      rst = future.fuse() => {
        return Ok(rst);
      }
      _ = R::sleep(duration).fuse() => {
        return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out"));
      }
    }
  }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct AgnosticTime<R: Runtime>(PhantomData<R>);

#[async_trait::async_trait]
impl<R> Time for AgnosticTime<R>
where
  R: Runtime,
{
  async fn delay_for(duration: Duration) {
    R::sleep(duration).await;
  }

  async fn timeout<F: 'static + Future + Send>(
    duration: Duration,
    future: F,
  ) -> Result<F::Output, std::io::Error> {
    R::timeout(duration, future).await
  }
}

#[doc(hidden)]
pub struct AsyncDnsTcp<R: Runtime>(<R::Net as super::Net>::TcpStream);

impl<R: Runtime> trust_dns_proto::tcp::DnsTcpStream for AsyncDnsTcp<R> {
  type Time = AgnosticTime<R>;
}

impl<R: Runtime> AsyncDnsTcp<R> {
  async fn connect(addr: SocketAddr) -> std::io::Result<Self> {
    <<R::Net as super::Net>::TcpStream as super::TcpStream>::connect(addr)
      .await
      .map(Self)
  }
}

impl<R: Runtime> futures_util::AsyncRead for AsyncDnsTcp<R> {
  fn poll_read(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &mut [u8],
  ) -> std::task::Poll<io::Result<usize>> {
    futures_util::AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
  }
}

impl<R: Runtime> futures_util::AsyncWrite for AsyncDnsTcp<R> {
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

#[doc(hidden)]
pub struct AsyncDnsUdp<R: Runtime>(<R::Net as super::Net>::UdpSocket);

impl<R: Runtime> AsyncDnsUdp<R> {
  async fn bind(addr: SocketAddr) -> std::io::Result<Self> {
    <<R::Net as super::Net>::UdpSocket as super::UdpSocket>::bind(addr)
      .await
      .map(Self)
  }
}

impl<R: Runtime> trust_dns_proto::udp::DnsUdpSocket for AsyncDnsUdp<R> {
  type Time = AgnosticTime<R>;

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

impl<R: Runtime> RuntimeProvider for AsyncRuntimeProvider<R> {
  type Handle = AsyncSpawn<R>;

  type Timer = Timer<R>;

  type Udp = AsyncDnsUdp<R>;

  type Tcp = AsyncDnsTcp<R>;

  fn create_handle(&self) -> Self::Handle {
    self.runtime
  }

  fn connect_tcp(
    &self,
    addr: SocketAddr,
  ) -> std::pin::Pin<Box<dyn Send + Future<Output = io::Result<Self::Tcp>>>> {
    AsyncDnsTcp::connect(addr).boxed()
  }

  fn bind_udp(
    &self,
    local_addr: SocketAddr,
    _server_addr: SocketAddr,
  ) -> std::pin::Pin<Box<dyn Send + Future<Output = io::Result<Self::Udp>>>> {
    AsyncDnsUdp::bind(local_addr).boxed()
  }
}

/// Create `DnsHandle` with the help of `AsyncRuntimeProvider`.
#[doc(hidden)]
pub struct AsyncConnectionProvider<R: Runtime> {
  runtime_provider: AsyncRuntimeProvider<R>,
  connection_provider: GenericConnector<AsyncRuntimeProvider<R>>,
}

impl<R: Runtime> Default for AsyncConnectionProvider<R> {
  fn default() -> Self {
    Self::new()
  }
}

impl<R: Runtime> AsyncConnectionProvider<R> {
  pub fn new() -> Self {
    Self {
      runtime_provider: AsyncRuntimeProvider::new(),
      connection_provider: GenericConnector::new(AsyncRuntimeProvider::new()),
    }
  }
}

impl<R: Runtime> Clone for AsyncConnectionProvider<R> {
  fn clone(&self) -> Self {
    Self {
      runtime_provider: self.runtime_provider,
      connection_provider: self.connection_provider.clone(),
    }
  }
}

impl<R: Runtime> ConnectionProvider for AsyncConnectionProvider<R> {
  type Conn = <GenericConnector<AsyncRuntimeProvider<R>> as ConnectionProvider>::Conn;
  type FutureConn = <GenericConnector<AsyncRuntimeProvider<R>> as ConnectionProvider>::FutureConn;
  type RuntimeProvider = AsyncRuntimeProvider<R>;

  fn new_connection(
    &self,
    config: &trust_dns_resolver::config::NameServerConfig,
    options: &trust_dns_resolver::config::ResolverOpts,
  ) -> Self::FutureConn {
    self.connection_provider.new_connection(config, options)
  }
}

pub use dns_util::read_resolv_conf;

#[cfg(unix)]
mod dns_util {
  use std::{io, path::Path};

  use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

  /// Read the DNS configuration from a file.
  pub fn read_resolv_conf<P: AsRef<Path>>(path: P) -> io::Result<(ResolverConfig, ResolverOpts)> {
    std::fs::read_to_string(path).and_then(trust_dns_resolver::system_conf::parse_resolv_conf)
  }
}

#[cfg(not(unix))]
mod dns_util {
  use std::{
    fs::File,
    io::{self, Read},
    net::SocketAddr,
    path::Path,
    time::Duration,
  };
  use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    Name,
  };

  const DEFAULT_PORT: u16 = 53;

  /// Read the DNS configuration from a file.
  pub fn read_resolv_conf<P: AsRef<Path>>(path: P) -> io::Result<(ResolverConfig, ResolverOpts)> {
    let mut data = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut data)?;
    parse_resolv_conf(&data)
  }

  fn parse_resolv_conf<T: AsRef<[u8]>>(data: T) -> io::Result<(ResolverConfig, ResolverOpts)> {
    let parsed_conf = resolv_conf::Config::parse(&data).map_err(|e| {
      io::Error::new(
        io::ErrorKind::Other,
        format!("Error parsing resolv.conf: {e}"),
      )
    })?;
    into_resolver_config(parsed_conf)
  }

  fn into_resolver_config(
    parsed_config: resolv_conf::Config,
  ) -> io::Result<(ResolverConfig, ResolverOpts)> {
    let domain = None;

    // nameservers
    let mut nameservers = Vec::<NameServerConfig>::with_capacity(parsed_config.nameservers.len());
    for ip in &parsed_config.nameservers {
      nameservers.push(NameServerConfig {
        socket_addr: SocketAddr::new(ip.into(), DEFAULT_PORT),
        protocol: Protocol::Udp,
        tls_dns_name: None,
        trust_negative_responses: false,
        #[cfg(feature = "dns-over-rustls")]
        tls_config: None,
        bind_addr: None,
      });
      nameservers.push(NameServerConfig {
        socket_addr: SocketAddr::new(ip.into(), DEFAULT_PORT),
        protocol: Protocol::Tcp,
        tls_dns_name: None,
        trust_negative_responses: false,
        #[cfg(feature = "dns-over-rustls")]
        tls_config: None,
        bind_addr: None,
      });
    }
    if nameservers.is_empty() {
      #[cfg(feature = "tracing")]
      tracing::warn!(
        target = "agnostic.read_resolv_conf",
        "no nameservers found in resolv conf"
      );
    }

    // search
    let mut search = vec![];
    for search_domain in parsed_config.get_last_search_or_domain() {
      search.push(Name::from_str_relaxed(search_domain).map_err(|e| {
        io::Error::new(
          io::ErrorKind::Other,
          format!("Error parsing resolv.conf: {e}"),
        )
      })?);
    }

    let config = ResolverConfig::from_parts(domain, search, nameservers);

    let mut options = ResolverOpts::default();
    options.timeout = Duration::from_secs(parsed_config.timeout as u64);
    options.attempts = parsed_config.attempts as usize;
    options.ndots = parsed_config.ndots as usize;

    Ok((config, options))
  }
}
