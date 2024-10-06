#[cfg(feature = "time")]
use core::future::Future;

#[cfg(feature = "time")]
use crate::async_io::*;
#[cfg(feature = "time")]
mod after;
#[cfg(feature = "time")]
pub use after::*;

#[cfg(feature = "time")]
use std::time::{Duration, Instant};

use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder};

impl<F> super::Detach for ::async_std::task::JoinHandle<F> {}

/// A [`AsyncSpawner`] that uses the [`async-std`](async_std) runtime.
#[derive(Debug, Clone, Copy)]
pub struct AsyncStdSpawner;

impl Yielder for AsyncStdSpawner {
  async fn yield_now() {
    ::async_std::task::yield_now().await
  }

  async fn yield_now_local() {
    ::async_std::task::yield_now().await
  }
}

impl AsyncSpawner for AsyncStdSpawner {
  type JoinHandle<F>
    = ::async_std::task::JoinHandle<F>
  where
    F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    ::async_std::task::spawn(future)
  }
}

impl AsyncLocalSpawner for AsyncStdSpawner {
  type JoinHandle<F>
    = ::async_std::task::JoinHandle<F>
  where
    F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    ::async_std::task::spawn_local(future)
  }
}

impl AsyncBlockingSpawner for AsyncStdSpawner {
  type JoinHandle<R>
    = ::async_std::task::JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::async_std::task::spawn_blocking(f)
  }
}

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`async_std`](::async_std) runtime.
#[derive(Debug, Clone, Copy)]
pub struct AsyncStdRuntime;

impl core::fmt::Display for AsyncStdRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "async-std")
  }
}

impl super::RuntimeLite for AsyncStdRuntime {
  type Spawner = AsyncStdSpawner;
  type LocalSpawner = AsyncStdSpawner;
  type BlockingSpawner = AsyncStdSpawner;

  #[cfg(feature = "time")]
  type AfterSpawner = AsyncStdSpawner;
  #[cfg(feature = "time")]
  type LocalAfterSpawner = AsyncStdSpawner;

  #[cfg(feature = "time")]
  type Interval = AsyncIoInterval;
  #[cfg(feature = "time")]
  type LocalInterval = AsyncIoInterval;
  #[cfg(feature = "time")]
  type Sleep = AsyncIoSleep;
  #[cfg(feature = "time")]
  type LocalSleep = AsyncIoSleep;
  #[cfg(feature = "time")]
  type Delay<F>
    = AsyncIoDelay<F>
  where
    F: Future + Send;
  #[cfg(feature = "time")]
  type LocalDelay<F>
    = AsyncIoDelay<F>
  where
    F: Future;
  #[cfg(feature = "time")]
  type Timeout<F>
    = AsyncIoTimeout<F>
  where
    F: Future + Send;
  #[cfg(feature = "time")]
  type LocalTimeout<F>
    = AsyncIoTimeout<F>
  where
    F: Future;

  fn new() -> Self {
    Self
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::async_std::task::block_on(f)
  }

  #[cfg(feature = "time")]
  fn interval(interval: Duration) -> Self::Interval {
    AsyncIoInterval::interval(interval)
  }

  #[cfg(feature = "time")]
  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    AsyncIoInterval::interval_at(start, period)
  }

  #[cfg(feature = "time")]
  fn interval_local(interval: Duration) -> Self::LocalInterval {
    AsyncIoInterval::interval(interval)
  }

  #[cfg(feature = "time")]
  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    AsyncIoInterval::interval_at(start, period)
  }

  #[cfg(feature = "time")]
  fn sleep(duration: Duration) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    AsyncIoSleep::sleep(duration)
  }

  #[cfg(feature = "time")]
  fn sleep_until(instant: Instant) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    AsyncIoSleep::sleep_until(instant)
  }

  #[cfg(feature = "time")]
  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    AsyncIoSleep::sleep(duration)
  }

  #[cfg(feature = "time")]
  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    AsyncIoSleep::sleep_until(instant)
  }

  async fn yield_now() {
    ::async_std::task::yield_now().await
  }

  #[cfg(feature = "time")]
  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <AsyncIoDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
  }

  #[cfg(feature = "time")]
  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <AsyncIoDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  #[cfg(feature = "time")]
  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <AsyncIoDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
  }

  #[cfg(feature = "time")]
  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <AsyncIoDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }
}
