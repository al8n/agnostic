#![allow(warnings)] // not used on emscripten

use std::{
  env,
  future::Future,
  net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
  sync::atomic::{AtomicUsize, Ordering},
};

use agnostic_lite::RuntimeLite;

use super::ToSocketAddrs;

mod tcp;
mod udp;

#[cfg(feature = "tokio")]
fn tokio_run<F>(f: F)
where
  F: core::future::Future<Output = ()>,
{
  tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(f);
}

#[cfg(feature = "async-std")]
fn async_std_run<F>(f: F)
where
  F: core::future::Future<Output = ()>,
{
  async_std::task::block_on(f);
}

#[cfg(feature = "smol")]
fn smol_run<F>(f: F)
where
  F: core::future::Future<Output = ()>,
{
  smol::block_on(f);
}

static PORT: AtomicUsize = AtomicUsize::new(0);

pub fn next_test_ip4() -> SocketAddr {
  let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + base_port();
  SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

pub fn next_test_ip6() -> SocketAddr {
  let port = PORT.fetch_add(1, Ordering::Relaxed) as u16 + base_port();
  SocketAddr::V6(SocketAddrV6::new(
    Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
    port,
    0,
    0,
  ))
}

// The bots run multiple builds at the same time, and these builds
// all want to use ports. This function figures out which workspace
// it is running in and assigns a port range based on it.
fn base_port() -> u16 {
  let cwd = if cfg!(target_env = "sgx") {
    String::from("sgx")
  } else {
    env::current_dir()
      .unwrap()
      .into_os_string()
      .into_string()
      .unwrap()
  };
  let dirs = [
    "32-opt",
    "32-nopt",
    "musl-64-opt",
    "cross-opt",
    "64-opt",
    "64-nopt",
    "64-opt-vg",
    "64-debug-opt",
    "all-opt",
    "snap3",
    "dist",
    "sgx",
  ];
  dirs
    .iter()
    .enumerate()
    .find(|&(_, dir)| cwd.contains(dir))
    .map(|p| p.0)
    .unwrap_or(0) as u16
    * 1000
    + 19600
}
