use super::*;
use agnostic_lite::tokio::*;

/// Network abstractions for [`tokio`](::tokio) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

impl Runtime for TokioRuntime {
  #[cfg(feature = "net")]
  type Net = self::net::TokioNet;
}
