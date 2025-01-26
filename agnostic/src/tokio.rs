use super::*;
pub use agnostic_lite::tokio::*;

/// Network abstractions for [`tokio`] runtime
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub use agnostic_net::tokio as net;

/// Process abstractions for [`tokio`] runtime
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process {
  pub use agnostic_process::tokio::*;
}

/// Quinn abstractions for [`tokio`] runtime
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "quinn")]
#[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
pub mod quinn {
  pub use quinn::TokioRuntime as TokioQuinnRuntime;
}

impl Runtime for TokioRuntime {
  #[cfg(feature = "net")]
  type Net = net::Net;

  #[cfg(feature = "process")]
  type Process = process::TokioProcess;

  #[cfg(feature = "quinn")]
  type Quinn = quinn::TokioQuinnRuntime;

  #[cfg(feature = "quinn")]
  fn quinn() -> Self::Quinn {
    quinn::TokioQuinnRuntime
  }
}
