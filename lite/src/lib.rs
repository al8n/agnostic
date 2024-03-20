#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(any(feature = "std", test))]
extern crate std;

mod spawner;
pub use spawner::*;

mod local_spawner;
pub use local_spawner::*;

#[cfg(feature = "std")]
mod sleep;
#[cfg(feature = "std")]
pub use sleep::*;

#[cfg(feature = "std")]
mod interval;
#[cfg(feature = "std")]
pub use interval::*;

#[cfg(feature = "std")]
mod timeout;
#[cfg(feature = "std")]
pub use timeout::*;

#[cfg(feature = "std")]
mod delay;
#[cfg(feature = "std")]
pub use delay::*;
