use super::*;
use agnostic_lite::async_std::AsyncStdRuntime;

/// Network abstractions for [`async-std`](::async_std) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

impl Runtime for AsyncStdRuntime {
  #[cfg(feature = "net")]
  type Net = net::AsyncStdNet;
}
