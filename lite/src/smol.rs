use core::future::Future;

#[cfg(feature = "time")]
mod after;
#[cfg(feature = "time")]
pub use after::*;

#[cfg(feature = "time")]
use crate::async_io::*;
use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder};
#[cfg(feature = "time")]
use std::time::{Duration, Instant};

impl<T> super::Detach for ::smol::Task<T> {
  fn detach(self) {
    ::smol::Task::detach(self)
  }
}

/// A [`AsyncSpawner`] that uses the [`smol`](::smol) runtime.
#[derive(Debug, Clone, Copy)]
pub struct SmolSpawner;

impl Yielder for SmolSpawner {
  async fn yield_now() {
    ::smol::future::yield_now().await
  }
}

impl AsyncSpawner for SmolSpawner {
  type JoinHandle<F> = ::smol::Task<F> where F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    ::smol::spawn(future)
  }

  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    ::smol::spawn(future).detach()
  }
}

impl AsyncLocalSpawner for SmolSpawner {
  type JoinHandle<F> = ::smol::Task<F> where F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    ::smol::LocalExecutor::new().spawn(future)
  }

  fn spawn_local_detach<F>(future: F)
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    ::smol::LocalExecutor::new().spawn(future).detach();
  }
}

impl AsyncBlockingSpawner for SmolSpawner {
  type JoinHandle<R> = ::smol::Task<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::unblock(f)
  }

  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::unblock(f).detach()
  }
}

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`smol`](::smol) runtime.
#[derive(Debug, Clone, Copy)]
pub struct SmolRuntime;

impl core::fmt::Display for SmolRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "smol")
  }
}

impl super::RuntimeLite for SmolRuntime {
  type Spawner = SmolSpawner;
  type LocalSpawner = SmolSpawner;
  type BlockingSpawner = SmolSpawner;

  #[cfg(feature = "time")]
  type AfterSpawner = SmolSpawner;
  #[cfg(feature = "time")]
  type LocalAfterSpawner = SmolSpawner;

  #[cfg(feature = "time")]
  type Interval = AsyncIoInterval;
  #[cfg(feature = "time")]
  type LocalInterval = AsyncIoInterval;
  #[cfg(feature = "time")]
  type Sleep = AsyncIoSleep;
  #[cfg(feature = "time")]
  type LocalSleep = AsyncIoSleep;
  #[cfg(feature = "time")]
  type Delay<F> = AsyncIoDelay<F> where F: Future + Send;
  #[cfg(feature = "time")]
  type LocalDelay<F> = AsyncIoDelay<F> where F: Future;
  #[cfg(feature = "time")]
  type Timeout<F> = AsyncIoTimeout<F> where F: Future + Send;
  #[cfg(feature = "time")]
  type LocalTimeout<F> = AsyncIoTimeout<F> where F: Future;

  fn new() -> Self {
    Self
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::smol::block_on(f)
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
    ::smol::future::yield_now().await
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
