#![doc = include_str!("../README.md")]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::needless_return)]
#![allow(unreachable_code)]

pub use agnostic_lite::{
  cfg_async_std, cfg_smol, cfg_tokio, time, AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner,
  RuntimeLite, Yielder,
};

/// Traits, helpers, and type definitions for asynchronous I/O functionality.
pub use agnostic_io as io;

/// [`tokio`] runtime adapter
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

/// [`async_std`] runtime adapter
///
/// [`async_std`]: https://docs.rs/async-std
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std;

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

/// Agnostic async DNS provider.
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub use agnostic_dns as dns;

/// Quinn related traits
#[cfg(feature = "quinn")]
#[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
pub mod quinn;

/// Process related traits
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process;

/// Runtime trait
pub trait Runtime: RuntimeLite {
  /// The network abstraction for this runtime
  #[cfg(feature = "net")]
  #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
  type Net: net::Net;

  /// The process abstraction for this runtime
  #[cfg(feature = "process")]
  #[cfg_attr(docsrs, doc(cfg(feature = "process")))]
  type Process: process::Process;

  /// The Quinn abstraction for this runtime
  #[cfg(feature = "quinn")]
  #[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
  type Quinn: quinn::QuinnRuntime;

  /// Returns the runtime for [`quinn`](::quinn)
  #[cfg(feature = "quinn")]
  #[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
  fn quinn() -> Self::Quinn;
}
