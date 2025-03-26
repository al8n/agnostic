// use super::*;

// #[cfg(any(feature = "tokio", feature = "smol"))]
// async fn resolve<N: net::Net>() {
//   let dns = Dns::<N>::new(
//     Default::default(),
//     Default::default(),
//     AsyncConnectionProvider::new(),
//   );
//   let res = dns.lookup_ip("google.com.").await.unwrap();
//   for ip in res {
//     println!("{}", ip);
//   }
// }

// #[test]
// #[cfg(unix)]
// fn read_conf() {
//   const PATH: &str = "/etc/resolv.conf";

//   match read_resolv_conf(PATH) {
//     Ok((config, conf)) => {
//       println!("{:?}", config);
//       println!("{:?}", conf);
//     }
//     Err(e) => {
//       let data = std::fs::read_to_string(PATH).unwrap();
//       panic!("{e}: \n{data}");
//     }
//   }
// }

// #[test]
// #[cfg(feature = "tokio")]
// fn tokio_resolve() {
//   tokio::runtime::Builder::new_current_thread()
//     .enable_all()
//     .build()
//     .unwrap()
//     .block_on(resolve::<agnostic_net::tokio::Net>());
// }

// #[test]
// #[cfg(feature = "smol")]
// fn smol_resolve() {
//   agnostic_net::runtime::smol::SmolRuntime::block_on(resolve::<agnostic_net::smol::Net>());
// }
