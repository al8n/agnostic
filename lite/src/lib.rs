//! `agnostic-lite` is lightweight [`agnostic`](https://crates.io/crates/agnostic).
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![allow(warnings)]
#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

mod spawner;
pub use spawner::*;

#[cfg(any(feature = "std", test))]
extern crate std;

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
