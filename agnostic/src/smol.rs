use super::*;

/// Network abstractions for [`smol`](::smol) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

// TODO: remove this when quinn support SmolRuntime
#[cfg(all(feature = "quinn", feature = "net"))]
mod quinn_;
pub use agnostic_lite::smol::*;

impl Runtime for SmolRuntime {
  #[cfg(feature = "net")]
  type Net = net::SmolNet;
}
