// TODO: remove this line when clippy fix the bug
#![allow(clippy::needless_return)]

cfg_time!(
  mod after;
  mod delay;
  mod interval;
  mod sleep;
  mod timeout;

  pub use after::*;
  pub use delay::*;
  pub use interval::*;
  pub use sleep::*;
  pub use timeout::*;

  use core::time::Duration;
);

use core::future::Future;

use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder};

/// A [`AsyncSpawner`] that uses the [`tokio`] runtime.
#[derive(Debug, Clone, Copy)]
pub struct TokioSpawner;

impl Yielder for TokioSpawner {
  async fn yield_now() {
    ::tokio::task::yield_now().await
  }

  async fn yield_now_local() {
    ::tokio::task::yield_now().await
  }
}

impl AsyncSpawner for TokioSpawner {
  type JoinHandle<F>
    = tokio::task::JoinHandle<F>
  where
    F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    ::tokio::task::spawn(future)
  }
}

impl AsyncLocalSpawner for TokioSpawner {
  type JoinHandle<F>
    = ::tokio::task::JoinHandle<F>
  where
    F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    tokio::task::spawn_local(future)
  }
}

impl<T> super::JoinHandle<T> for ::tokio::task::JoinHandle<T> {
  type JoinError = ::tokio::task::JoinError;

  fn abort(self) {
    Self::abort(&self)
  }
}

impl<T> super::LocalJoinHandle<T> for ::tokio::task::JoinHandle<T> {
  type JoinError = ::tokio::task::JoinError;
}

impl AsyncBlockingSpawner for TokioSpawner {
  type JoinHandle<R>
    = ::tokio::task::JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(_f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    #[cfg(not(target_family = "wasm"))]
    {
      ::tokio::task::spawn_blocking(_f)
    }

    #[cfg(target_family = "wasm")]
    {
      panic!("TokioRuntime::spawn_blocking is not supported on wasm")
    }
  }
}

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`tokio`] runtime.
///
/// [`tokio`]: https://docs.rs/tokio
#[derive(Debug, Clone, Copy)]
pub struct TokioRuntime;

impl core::fmt::Display for TokioRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "tokio")
  }
}

impl super::RuntimeLite for TokioRuntime {
  type Spawner = TokioSpawner;
  type LocalSpawner = TokioSpawner;
  type BlockingSpawner = TokioSpawner;

  fn new() -> Self {
    Self
  }

  fn name() -> &'static str {
    "tokio"
  }

  fn fqname() -> &'static str {
    "tokio"
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::tokio::runtime::Handle::current().block_on(f)
  }

  async fn yield_now() {
    ::tokio::task::yield_now().await
  }

  cfg_time!(
    type Instant = ::tokio::time::Instant;
    type AfterSpawner = TokioSpawner;

    type Interval = TokioInterval;
    type LocalInterval = TokioInterval;
    type Sleep = TokioSleep;
    type LocalSleep = TokioSleep;
    type Delay<F>
      = TokioDelay<F>
    where
      F: Future + Send;
    type LocalDelay<F>
      = TokioDelay<F>
    where
      F: Future;
    type Timeout<F>
      = TokioTimeout<F>
    where
      F: Future + Send;
    type LocalTimeout<F>
      = TokioTimeout<F>
    where
      F: Future;
  );

  cfg_time!(
    fn interval(interval: Duration) -> Self::Interval {
      use crate::time::AsyncIntervalExt;

      TokioInterval::interval(interval)
    }

    fn interval_at(start: Self::Instant, period: Duration) -> Self::Interval {
      use crate::time::AsyncIntervalExt;

      TokioInterval::interval_at(start, period)
    }

    fn interval_local(interval: Duration) -> Self::LocalInterval {
      use crate::time::AsyncIntervalExt;

      TokioInterval::interval(interval)
    }

    fn interval_local_at(start: Self::Instant, period: Duration) -> Self::LocalInterval {
      use crate::time::AsyncIntervalExt;

      TokioInterval::interval_at(start, period)
    }

    fn sleep(duration: Duration) -> Self::Sleep {
      use crate::time::AsyncSleepExt;

      TokioSleep::sleep(duration)
    }

    fn sleep_until(instant: Self::Instant) -> Self::Sleep {
      use crate::time::AsyncSleepExt;

      TokioSleep::sleep_until(instant)
    }

    fn sleep_local(duration: Duration) -> Self::LocalSleep {
      use crate::time::AsyncSleepExt;

      TokioSleep::sleep(duration)
    }

    fn sleep_local_until(instant: Self::Instant) -> Self::LocalSleep {
      use crate::time::AsyncSleepExt;

      TokioSleep::sleep_until(instant)
    }

    fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncDelayExt;

      <TokioDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
    }

    fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalDelayExt;

      <TokioDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
    }

    fn delay_at<F>(deadline: Self::Instant, fut: F) -> Self::Delay<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncDelayExt;

      <TokioDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
    }

    fn delay_local_at<F>(deadline: Self::Instant, fut: F) -> Self::LocalDelay<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalDelayExt;

      <TokioDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
    }

    fn timeout<F>(timeout: Duration, fut: F) -> Self::Timeout<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncTimeout;

      <TokioTimeout<F> as AsyncTimeout<F>>::timeout(timeout, fut)
    }

    fn timeout_at<F>(deadline: Self::Instant, future: F) -> Self::Timeout<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncTimeout;

      <TokioTimeout<F> as AsyncTimeout<F>>::timeout_at(deadline, future)
    }

    fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalTimeout;

      <TokioTimeout<F> as AsyncLocalTimeout<F>>::timeout_local(duration, future)
    }

    fn timeout_local_at<F>(deadline: Self::Instant, future: F) -> Self::LocalTimeout<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalTimeout;

      <TokioTimeout<F> as AsyncLocalTimeout<F>>::timeout_local_at(deadline, future)
    }
  );
}
