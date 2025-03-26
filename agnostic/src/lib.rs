#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::needless_return)]
#![allow(unreachable_code)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

pub use runtime::*;

/// The runtime related traits and types
mod runtime {
  pub use agnostic_lite::{
    AfterHandle, AfterHandleError, AsyncAfterSpawner, AsyncBlockingSpawner, AsyncLocalSpawner,
    AsyncSpawner, JoinHandle, LocalJoinHandle, RuntimeLite, Yielder, cfg_linux, cfg_smol,
    cfg_tokio, cfg_unix, cfg_windows, time,
  };

  /// Runtime trait
  pub trait Runtime: RuntimeLite {
    /// The network abstraction for this runtime
    #[cfg(feature = "net")]
    #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
    type Net: super::net::Net;

    /// The process abstraction for this runtime
    #[cfg(feature = "process")]
    #[cfg_attr(docsrs, doc(cfg(feature = "process")))]
    type Process: super::process::Process;

    /// The Quinn abstraction for this runtime
    #[cfg(feature = "quinn")]
    #[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
    type Quinn: super::quinn::QuinnRuntime;

    /// Returns the runtime for [`quinn`](::quinn)
    #[cfg(feature = "quinn")]
    #[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
    fn quinn() -> Self::Quinn;
  }
}

/// Traits, helpers, and type definitions for asynchronous I/O functionality.
pub use agnostic_io as io;

/// [`tokio`] runtime adapter
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

/// [`smol`] runtime adapter
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol;

/// Network related traits
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

// /// Agnostic async DNS provider.
// #[cfg(feature = "dns")]
// #[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
// pub mod dns {
//   pub use agnostic_dns::{
//     AgnosticTime, AsyncConnectionProvider, AsyncDnsUdp, AsyncRuntimeProvider, AsyncSpawn,
//     CLOUDFLARE_IPS, Dns, GOOGLE_IPS, LookupIpStrategy, NameServerConfig, NameServerConfigGroup,
//     Protocol, QUAD9_IPS, ResolverConfig, ResolverOpts, ServerOrderingStrategy, Timer,
//     read_system_conf,
//   };

//   #[cfg(feature = "dns-over-rustls")]
//   #[cfg_attr(docsrs, doc(cfg(feature = "dns-over-rustls")))]
//   pub use agnostic_dns::TlsClientConfig;

//   #[cfg(unix)]
//   #[cfg_attr(docsrs, doc(cfg(unix)))]
//   pub use agnostic_dns::{parse_resolv_conf, read_resolv_conf};
// }

/// Quinn related traits
#[cfg(feature = "quinn")]
#[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
pub mod quinn;

/// Process related traits
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process;
