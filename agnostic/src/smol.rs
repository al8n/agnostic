use super::*;
pub use agnostic_lite::smol::*;

/// Network abstractions for [`smol`](::smol) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

/// Command abstractions for [`smol`](::smol) runtime
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process {
  pub use agnostic_process::smol::*;
}

impl Runtime for SmolRuntime {
  #[cfg(feature = "net")]
  type Net = net::SmolNet;

  #[cfg(feature = "process")]
  type Process = process::AsyncProcess;
}
