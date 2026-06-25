use super::*;

#[cfg(any(feature = "tokio", feature = "smol"))]
async fn resolve<N: net::Net>()
where
  N::UdpSocket: Send + Sync,
  <N::UdpSocket as UdpSocket>::Bind<std::net::SocketAddr>: Send,
  N::TcpStream: Send + Sync,
  <N::TcpStream as TcpStream>::Connect<std::net::SocketAddr>: Send,
{
  // Pin to IPv4 Google DNS: CI runners frequently lack IPv6 connectivity, so an IPv6 nameserver
  // would never respond.
  use std::net::{IpAddr, Ipv4Addr};
  let config = ResolverConfig::from_parts(
    None,
    vec![],
    vec![
      NameServerConfig::udp_and_tcp(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
      NameServerConfig::udp_and_tcp(IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4))),
    ],
  );
  let dns = Dns::<N>::builder_with_config(config, AsyncConnectionProvider::new())
    .build()
    .unwrap();
  let res = dns.lookup_ip("google.com.").await.unwrap();
  for ip in res.iter() {
    println!("{}", ip);
  }
}

#[test]
#[cfg(all(unix, not(target_os = "android"), not(target_vendor = "apple")))]
fn read_conf() {
  const PATH: &str = "/etc/resolv.conf";

  match read_resolv_conf(PATH) {
    Ok((config, conf)) => {
      println!("{:?}", config);
      println!("{:?}", conf);
    }
    Err(e) => {
      let data = std::fs::read_to_string(PATH).unwrap();
      panic!("{e}: \n{data}");
    }
  }
}

#[test]
#[cfg(feature = "tokio")]
fn tokio_resolve() {
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(resolve::<agnostic_net::tokio::Net>());
}

#[test]
#[cfg(feature = "smol")]
fn smol_resolve() {
  agnostic_net::runtime::smol::SmolRuntime::block_on(resolve::<agnostic_net::smol::Net>());
}
