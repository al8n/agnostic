use super::*;
pub use agnostic_lite::{async_io::*, smol::*};

/// Network abstractions for [`smol`] runtime
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub use agnostic_net::smol as net;

/// Process abstractions for [`smol`] runtime
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "process")]
#[cfg_attr(docsrs, doc(cfg(feature = "process")))]
pub mod process {
  pub use agnostic_process::smol::*;
}

/// Quinn abstractions for [`smol`] runtime
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "quinn")]
#[cfg_attr(docsrs, doc(cfg(feature = "quinn")))]
pub mod quinn {
  pub use quinn::SmolRuntime as SmolQuinnRuntime;
}

impl Runtime for SmolRuntime {
  #[cfg(feature = "net")]
  type Net = net::Net;

  #[cfg(feature = "process")]
  type Process = process::AsyncProcess;

  #[cfg(feature = "quinn")]
  type Quinn = quinn::SmolQuinnRuntime;

  #[cfg(feature = "quinn")]
  fn quinn() -> Self::Quinn {
    quinn::SmolQuinnRuntime
  }
}
