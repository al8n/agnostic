//! Agnostic is a trait for users who want to write async runtime-agnostic crate.
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::needless_return)]
#![allow(unreachable_code)]

#[cfg(all(feature = "compat", not(feature = "net")))]
compile_error!("`compat` feature is enabled, but `net` feature is disabled, `compact` feature must only be enabled with `net` feature");

#[macro_use]
mod macros;

pub use agnostic_lite::*;

/// [`tokio`] runtime adapter
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

/// [`async_std`] runtime adapter
///
/// [`async_std`]: https://docs.rs/async-std
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std;

/// [`smol`] runtime adapter
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol;

/// Network related traits
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

use std::{
  future::Future,
  time::{Duration, Instant},
};

/// Runtime trait
pub trait Runtime: Sized + Unpin + Copy + Send + Sync + 'static {
  /// The spawner type for this runtime
  type Spawner: AsyncSpawner;
  /// The local spawner type for this runtime
  type LocalSpawner: AsyncLocalSpawner;
  /// The join handle type for blocking tasks
  type BlockJoinHandle<R>
  where
    R: Send + 'static;
  /// The interval type for this runtime
  type Interval: AsyncInterval;
  /// The local interval type for this runtime
  type LocalInterval: AsyncLocalInterval;
  /// The sleep type for this runtime
  type Sleep: AsyncSleep;
  /// The local sleep type for this runtime
  type LocalSleep: AsyncLocalSleep;
  /// The delay type for this runtime
  type Delay<F>: AsyncDelay<F>
  where
    F: Future + Send;
  /// The local delay type for this runtime
  type LocalDelay<F>: AsyncLocalDelay<F>
  where
    F: Future;
  /// The timeout type for this runtime
  type Timeout<F>: AsyncTimeout<F>
  where
    F: Future + Send;
  /// The local timeout type for this runtime
  type LocalTimeout<F>: AsyncLocalTimeout<F>
  where
    F: Future;

  /// The network abstraction for this runtime
  #[cfg(feature = "net")]
  #[cfg_attr(docsrs, doc(cfg(feature = "net")))]
  type Net: net::Net;

  /// Create a new instance of the runtime
  fn new() -> Self;

  /// Spawn a future onto the runtime
  fn spawn<F>(future: F) -> <Self::Spawner as AsyncSpawner>::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    <Self::Spawner as AsyncSpawner>::spawn(future)
  }

  /// Spawn a future onto the runtime and detach it
  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    <Self::Spawner as AsyncSpawner>::spawn_detach(future);
  }

  /// Spawn a future onto the local runtime
  fn spawn_local<F>(future: F) -> <Self::LocalSpawner as AsyncLocalSpawner>::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self::LocalSpawner as AsyncLocalSpawner>::spawn(future)
  }

  /// Spawn a future onto the local runtime and detach it
  fn spawn_local_detach<F>(future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self::LocalSpawner as AsyncLocalSpawner>::spawn_detach(future)
  }

  /// Spawn a blocking function onto the runtime
  fn spawn_blocking<F, R>(f: F) -> Self::BlockJoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  /// Spawn a blocking function onto the runtime and detach it
  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    <Self as Runtime>::spawn_blocking(f);
  }

  /// Block the current thread on the given future
  fn block_on<F: Future>(f: F) -> F::Output;

  /// Create a new interval that starts at the current time and
  /// yields every `period` duration
  fn interval(interval: Duration) -> Self::Interval;

  /// Create a new interval that starts at the given instant and
  /// yields every `period` duration
  fn interval_at(start: Instant, period: Duration) -> Self::Interval;

  /// Create a new interval that starts at the current time and
  /// yields every `period` duration
  fn interval_local(interval: Duration) -> Self::LocalInterval;

  /// Create a new interval that starts at the given instant and
  /// yields every `period` duration
  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval;

  /// Create a new sleep future that completes after the given duration
  /// has elapsed
  fn sleep(duration: Duration) -> Self::Sleep;

  /// Create a new sleep future that completes at the given instant
  /// has elapsed
  fn sleep_until(instant: Instant) -> Self::Sleep;

  /// Create a new sleep future that completes after the given duration
  /// has elapsed
  fn sleep_local(duration: Duration) -> Self::LocalSleep;

  /// Create a new sleep future that completes at the given instant
  /// has elapsed
  fn sleep_local_until(instant: Instant) -> Self::LocalSleep;

  /// Yield the current task
  fn yield_now() -> impl Future<Output = ()> + Send;

  /// Create a new delay future that runs the `fut` after the given duration
  /// has elapsed. The `Future` will never be polled until the duration has
  /// elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send;

  /// Like [`delay`](Runtime::delay), but does not require the `fut` to be `Send`.
  /// Create a new delay future that runs the `fut` after the given duration
  /// has elapsed. The `Future` will never be polled until the duration has
  /// elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future;

  /// Create a new timeout future that runs the `future` after the given deadline.
  /// The `Future` will never be polled until the deadline has reached.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send;

  /// Like [`delay_at`](Runtime::delay_at), but does not require the `fut` to be `Send`.
  /// Create a new timeout future that runs the `future` after the given deadline
  /// The `Future` will never be polled until the deadline has reached.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future;

  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    <Self::Timeout<F> as AsyncTimeout<F>>::timeout(duration, future)
  }

  /// Requires a `Future` to complete before the specified instant in time.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    <Self::Timeout<F> as AsyncTimeout<F>>::timeout_at(deadline, future)
  }

  /// Like [`timeout`](Runtime::timeout), but does not requrie the `future` to be `Send`.
  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    <Self::LocalTimeout<F> as AsyncLocalTimeout<F>>::timeout_local(duration, future)
  }

  /// Like [`timeout_at`](Runtime::timeout_at), but does not requrie the `future` to be `Send`.
  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_local_at<F>(deadline: Instant, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    <Self::LocalTimeout<F> as AsyncLocalTimeout<F>>::timeout_local_at(deadline, future)
  }
}

/// Traits for IO
#[cfg(feature = "io")]
#[cfg_attr(docsrs, doc(cfg(feature = "io")))]
pub mod io {
  pub use futures_util::{AsyncRead, AsyncWrite};

  #[cfg(feature = "tokio-compat")]
  #[cfg_attr(docsrs, doc(cfg(feature = "tokio-compat")))]
  pub use tokio::io::{AsyncRead as TokioAsyncRead, AsyncWrite as TokioAsyncWrite, ReadBuf};
}
