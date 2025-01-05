use agnostic::Runtime;

use crate::{server::{Server, ServerOptions}, tests::make_service};

use super::make_service_with_service_name;


async fn server_start_stop<R: Runtime>() {
  let s = make_service::<R>().await;
  let serv = Server::<R>::new(s, ServerOptions::default().with_v6_iface(Some(0))).await.unwrap();

  serv.shutdown().await;
}

async fn server_lookup<R: Runtime>() {
  use agnostic::net::UdpSocket;

  // let s = make_service_with_service_name("_foobar._tcp").await;
  // let serv = Server::<R>::new(s, ServerOptions::default().with_v6_iface(Some(0))).await.unwrap();

  let a = <<R::Net as agnostic::net::Net>::UdpSocket as agnostic::net::UdpSocket>::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
  let b = <<R::Net as agnostic::net::Net>::UdpSocket as agnostic::net::UdpSocket>::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();

  println!("{}", a.local_addr().unwrap());
  println!("{}", b.local_addr().unwrap());
}

test_suites!(tokio {
  server_start_stop,
  server_lookup,
});

test_suites!(smol {
  server_start_stop,
  server_lookup,
});

test_suites!(async_std {
  server_start_stop,
  server_lookup,
});