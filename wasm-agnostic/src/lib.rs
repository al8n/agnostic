//! Wasm-Agnostic is a helpful crate for users who want to write async runtime-agnostic crate which can be built to wasm target.
//! **Note:** This crate cannot be compiled by `cargo`, you need to use `cargo wasix`. For more details, please see [https://wasix.org/docs](https://wasix.org/docs).
#![cfg_attr(feature = "nightly", feature(return_position_impl_trait_in_trait))]
#![cfg_attr(feature = "nightly", allow(clippy::manual_async_fn))]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]

/// Tokio runtime adapter powered by wasix and tokio
pub mod tokio;

pub use agnostic::*;
