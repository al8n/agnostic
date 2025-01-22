use super::*;
pub use agnostic_lite::tokio::*;

/// Network abstractions for [`tokio`](::tokio) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

/// Command abstractions for [`tokio`](::tokio) runtime
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process {
  pub use ::tokio::process::*;
  pub use agnostic_process::tokio::TokioProcess;
}

impl Runtime for TokioRuntime {
  #[cfg(feature = "net")]
  type Net = net::TokioNet;

  #[cfg(feature = "process")]
  type Process = process::TokioProcess;
}
