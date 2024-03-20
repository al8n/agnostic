//! Agnostic is a trait for users who want to write async runtime-agnostic crate.
#![allow(warnings)]
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

use futures_util::Stream;

/// Simlilar to Go's `time.AfterFunc`
pub trait Delay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self;

  fn reset(&mut self, dur: Duration) -> impl Future<Output = ()> + Send + '_;

  fn cancel(&mut self) -> impl Future<Output = Option<F::Output>> + Send + '_;
}

/// Runtime trait
pub trait Runtime: Sized + Unpin + Copy + Send + Sync + 'static {
  type Spawner: AsyncSpawner;
  type LocalSpawner: AsyncLocalSpawner;
  type BlockJoinHandle<R>
  where
    R: Send + 'static;
  type Interval: AsyncInterval;
  type LocalInterval: AsyncLocalInterval;
  type Sleep: AsyncSleep;
  type LocalSleep: AsyncLocalSleep;
  type Delay<F>: Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;
  type Timeout<F>: AsyncTimeout<F>
  where
    F: Future + Send;
  type LocalTimeout<F>: AsyncLocalTimeout<F>
  where
    F: Future;

  #[cfg(feature = "net")]
  type Net: net::Net;

  fn new() -> Self;

  fn spawn<F>(future: F) -> <Self::Spawner as AsyncSpawner>::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    <Self::Spawner as AsyncSpawner>::spawn(future)
  }

  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    <Self::Spawner as AsyncSpawner>::spawn_detach(future);
  }

  fn spawn_local<F>(future: F) -> <Self::LocalSpawner as AsyncLocalSpawner>::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self::LocalSpawner as AsyncLocalSpawner>::spawn(future)
  }

  fn spawn_local_detach<F>(future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self::LocalSpawner as AsyncLocalSpawner>::spawn_detach(future)
  }

  fn spawn_blocking<F, R>(f: F) -> Self::BlockJoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    <Self as Runtime>::spawn_blocking(f);
  }

  fn block_on<F: Future>(f: F) -> F::Output;

  fn interval(interval: Duration) -> Self::Interval;

  fn interval_at(start: Instant, period: Duration) -> Self::Interval;

  fn interval_local(interval: Duration) -> Self::LocalInterval;

  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval;

  fn sleep(duration: Duration) -> Self::Sleep;

  fn sleep_until(instant: Instant) -> Self::Sleep;

  fn sleep_local(duration: Duration) -> Self::LocalSleep;

  fn sleep_local_until(instant: Instant) -> Self::LocalSleep;

  fn yield_now() -> impl Future<Output = ()> + Send;

  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send + Sync + 'static;

  fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    <Self::Timeout<F> as AsyncTimeout<F>>::timeout(duration, future)
  }

  fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    <Self::Timeout<F> as AsyncTimeout<F>>::timeout_at(deadline, future)
  }

  fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    <Self::LocalTimeout<F> as AsyncLocalTimeout<F>>::timeout_local(duration, future)
  }

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
