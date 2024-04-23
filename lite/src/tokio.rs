#[cfg(feature = "time")]
mod timeout;
#[cfg(feature = "time")]
pub use timeout::*;

#[cfg(feature = "time")]
mod sleep;
#[cfg(feature = "time")]
pub use sleep::*;

#[cfg(feature = "time")]
mod interval;
#[cfg(feature = "time")]
pub use interval::*;

#[cfg(feature = "time")]
mod delay;
#[cfg(feature = "time")]
pub use delay::*;

#[cfg(feature = "time")]
use std::time::{Duration, Instant};

#[cfg(feature = "time")]
mod after;
#[cfg(feature = "time")]
pub use after::*;

use core::future::Future;

use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder};

/// A [`AsyncSpawner`] that uses the [`tokio`] runtime.
#[derive(Debug, Clone, Copy)]
pub struct TokioSpawner;

impl Yielder for TokioSpawner {
  async fn yield_now() {
    ::tokio::task::yield_now().await
  }
}

impl AsyncSpawner for TokioSpawner {
  type JoinHandle<F> = tokio::task::JoinHandle<F> where
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
  type JoinHandle<F> = ::tokio::task::JoinHandle<F> where
  F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    tokio::task::spawn_local(future)
  }
}

impl<T> super::Detach for ::tokio::task::JoinHandle<T> {}

impl AsyncBlockingSpawner for TokioSpawner {
  type JoinHandle<R> = ::tokio::task::JoinHandle<R>
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

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`tokio`](::tokio) runtime.
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

  #[cfg(feature = "time")]
  type AfterSpawner = TokioSpawner;
  #[cfg(feature = "time")]
  type LocalAfterSpawner = TokioSpawner;

  #[cfg(feature = "time")]
  type Interval = TokioInterval;
  #[cfg(feature = "time")]
  type LocalInterval = TokioInterval;
  #[cfg(feature = "time")]
  type Sleep = TokioSleep;
  #[cfg(feature = "time")]
  type LocalSleep = TokioSleep;
  #[cfg(feature = "time")]
  type Delay<F> = TokioDelay<F> where F: Future + Send;
  #[cfg(feature = "time")]
  type LocalDelay<F> = TokioDelay<F> where F: Future;
  #[cfg(feature = "time")]
  type Timeout<F> = TokioTimeout<F> where F: Future + Send;
  #[cfg(feature = "time")]
  type LocalTimeout<F> = TokioTimeout<F> where F: Future;

  fn new() -> Self {
    Self
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::tokio::runtime::Handle::current().block_on(f)
  }

  #[cfg(feature = "time")]
  fn interval(interval: Duration) -> Self::Interval {
    use crate::time::AsyncIntervalExt;

    TokioInterval::interval(interval)
  }

  #[cfg(feature = "time")]
  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    use crate::time::AsyncIntervalExt;

    TokioInterval::interval_at(start, period)
  }

  #[cfg(feature = "time")]
  fn interval_local(interval: Duration) -> Self::LocalInterval {
    use crate::time::AsyncIntervalExt;

    TokioInterval::interval(interval)
  }

  #[cfg(feature = "time")]
  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    use crate::time::AsyncIntervalExt;

    TokioInterval::interval_at(start, period)
  }

  #[cfg(feature = "time")]
  fn sleep(duration: Duration) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    TokioSleep::sleep(duration)
  }

  #[cfg(feature = "time")]
  fn sleep_until(instant: Instant) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    TokioSleep::sleep_until(instant)
  }

  #[cfg(feature = "time")]
  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    TokioSleep::sleep(duration)
  }

  #[cfg(feature = "time")]
  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    TokioSleep::sleep_until(instant)
  }

  async fn yield_now() {
    ::tokio::task::yield_now().await
  }

  #[cfg(feature = "time")]
  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <TokioDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
  }

  #[cfg(feature = "time")]
  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <TokioDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  #[cfg(feature = "time")]
  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <TokioDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
  }

  #[cfg(feature = "time")]
  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <TokioDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }
}
