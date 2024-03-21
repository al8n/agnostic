use std::{
  future::Future,
  time::{Duration, Instant},
};

/// The timeout abstraction for async runtime.
pub trait AsyncTimeout<F: Future + Send>:
  Future<Output = Result<F::Output, Elapsed>> + Send
{
  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout(timeout: Duration, fut: F) -> Self
  where
    Self: Sized;

  /// Requires a `Future` to complete before the specified instant in time.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized;
}

/// Like [`AsyncTimeout`], but this does not require `Send`.
pub trait AsyncLocalTimeout<F: Future>: Future<Output = Result<F::Output, Elapsed>> {
  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_local(timeout: Duration, fut: F) -> Self
  where
    Self: Sized;

  /// Requires a `Future` to complete before the specified instant in time.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_local_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized;
}

/// Elapsed error
#[derive(Debug, PartialEq, Eq)]
pub struct Elapsed;

impl core::fmt::Display for Elapsed {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "deadline has elapsed")
  }
}

impl std::error::Error for Elapsed {}

impl From<Elapsed> for std::io::Error {
  fn from(_: Elapsed) -> Self {
    std::io::ErrorKind::TimedOut.into()
  }
}

#[cfg(all(feature = "tokio", feature = "time"))]
impl From<::tokio::time::error::Elapsed> for Elapsed {
  fn from(_: ::tokio::time::error::Elapsed) -> Self {
    Elapsed
  }
}
