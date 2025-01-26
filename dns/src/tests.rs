use super::*;

async fn resolve<N: net::Net>() {
  let dns = Dns::<N>::new(
    Default::default(),
    Default::default(),
    AsyncConnectionProvider::new(),
  );
  let res = dns.lookup_ip("google.com.").await.unwrap();
  for ip in res {
    println!("{}", ip);
  }
}

#[test]
fn read_conf() {
  #[cfg(unix)]
  const PATH: &str = "/etc/resolv.conf";
  #[cfg(windows)]
  const PATH: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";

  let (config, conf) = read_resolv_conf(PATH).unwrap();
  println!("{:?}", config);
  println!("{:?}", conf);
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
#[cfg(feature = "async-std")]
fn async_std_resolve() {
  agnostic_net::runtime::async_std::AsyncStdRuntime::block_on(resolve::<
    agnostic_net::async_std::Net,
  >());
}

#[test]
#[cfg(feature = "smol")]
fn smol_resolve() {
  agnostic_net::runtime::smol::SmolRuntime::block_on(resolve::<agnostic_net::smol::Net>());
}
