use super::*;
pub use agnostic_lite::{async_io::*, async_std::*};

/// Network abstractions for [`async-std`] runtime
///
/// [`async-std`]: https://docs.rs/async-std
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub use agnostic_net::async_std as net;

/// Network abstractions for [`async-std`] runtime
///
/// [`async-std`]: https://docs.rs/async-std
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process {
  pub use agnostic_process::async_std::*;
}

/// Quinn abstractions for [`async-std`] runtime
///
/// [`async-std`]: https://docs.rs/async-std
#[cfg(feature = "quinn")]
#[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
pub mod quinn {
  pub use quinn::AsyncStdRuntime as AsyncStdQuinnRuntime;
}

impl Runtime for AsyncStdRuntime {
  #[cfg(feature = "net")]
  type Net = net::AsyncStdNet;

  #[cfg(feature = "process")]
  type Process = process::AsyncProcess;

  #[cfg(feature = "quinn")]
  type Quinn = quinn::AsyncStdQuinnRuntime;

  #[cfg(feature = "quinn")]
  fn quinn() -> Self::Quinn {
    quinn::AsyncStdQuinnRuntime
  }
}
