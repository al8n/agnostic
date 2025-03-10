use core::{
  future::Future,
  time::Duration,
};

/// The timeout abstraction for async runtime.
pub trait AsyncTimeout<F>
where
  F: Future + Send,
  Self: Future<Output = Result<F::Output, Elapsed>> + Send
{
  /// The instant type
  type Instant: super::Instant;

  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout(timeout: Duration, fut: F) -> Self
  where
    F: Future + Send,
    Self: Future<Output = Result<F::Output, Elapsed>> + Send + Sized;

  /// Requires a `Future` to complete before the specified instant in time.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_at(deadline: Self::Instant, fut: F) -> Self
  where
    F: Future + Send,
    Self: Future<Output = Result<F::Output, Elapsed>> + Send + Sized;
}

/// Like [`AsyncTimeout`], but this does not require `Send`.
pub trait AsyncLocalTimeout<F: Future>: Future<Output = Result<F::Output, Elapsed>> {
  /// The instant type
  type Instant: super::Instant;

  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_local(timeout: Duration, fut: F) -> Self
  where
    Self: Sized + Future<Output = Result<F::Output, Elapsed>>,
    F: Future;

  /// Requires a `Future` to complete before the specified instant in time.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_local_at(deadline: Self::Instant, fut: F) -> Self
  where
    Self: Sized + Future<Output = Result<F::Output, Elapsed>>,
    F: Future;
}

/// Elapsed error
#[derive(Debug, PartialEq, Eq)]
pub struct Elapsed;

impl core::fmt::Display for Elapsed {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "deadline has elapsed")
  }
}

impl core::error::Error for Elapsed {}

#[cfg(feature = "std")]
impl From<Elapsed> for std::io::Error {
  fn from(_: Elapsed) -> Self {
    std::io::ErrorKind::TimedOut.into()
  }
}

#[cfg(feature = "tokio")]
impl From<::tokio::time::error::Elapsed> for Elapsed {
  fn from(_: ::tokio::time::error::Elapsed) -> Self {
    Elapsed
  }
}

#[test]
#[cfg(feature = "std")]
fn test_elapsed_error() {
  assert_eq!(Elapsed.to_string(), "deadline has elapsed");
  let _: std::io::Error = Elapsed.into();
}
