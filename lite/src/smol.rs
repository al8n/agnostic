use core::future::Future;

cfg_time!(
  mod after;
  pub use after::*;
  use crate::async_io::*;
  use std::time::{Duration, Instant};
);

use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder};

pub use crate::spawner::handle::JoinError;

impl<T> super::Detach for ::smol::Task<T> {
  fn detach(self) {
    ::smol::Task::detach(self)
  }
}

/// A [`AsyncSpawner`] that uses the [`smol`] runtime.
///
/// [`smol`]: https://docs.rs/smol
#[derive(Debug, Clone, Copy)]
pub struct SmolSpawner;

impl Yielder for SmolSpawner {
  async fn yield_now() {
    ::smol::future::yield_now().await
  }

  async fn yield_now_local() {
    ::smol::future::yield_now().await
  }
}

impl AsyncSpawner for SmolSpawner {
  type JoinHandle<F>
    = ::smol::Task<F>
  where
    F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
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
  type JoinHandle<F>
    = ::smol::Task<F>
  where
    F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static,
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

join_handle!(::smol::Task<T>);

impl AsyncBlockingSpawner for SmolSpawner {
  type JoinHandle<R>
    = JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::unblock(f).into()
  }

  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::unblock(f).detach()
  }
}

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`smol`] runtime.
///
/// [`smol`]: https://docs.rs/smol
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

  cfg_time!(
    type AfterSpawner = SmolSpawner;
    type LocalAfterSpawner = SmolSpawner;

    type Interval = AsyncIoInterval;
    type LocalInterval = AsyncIoInterval;
    type Sleep = AsyncIoSleep;
    type LocalSleep = AsyncIoSleep;
    type Delay<F>
      = AsyncIoDelay<F>
    where
      F: Future + Send;
    type LocalDelay<F>
      = AsyncIoDelay<F>
    where
      F: Future;
    type Timeout<F>
      = AsyncIoTimeout<F>
    where
      F: Future + Send;
    type LocalTimeout<F>
      = AsyncIoTimeout<F>
    where
      F: Future;
  );

  fn new() -> Self {
    Self
  }

  fn name() -> &'static str {
    "smol"
  }

  fn fqname() -> &'static str {
    "smol"
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::smol::block_on(f)
  }

  async fn yield_now() {
    ::smol::future::yield_now().await
  }

  cfg_time!(
    fn interval(interval: Duration) -> Self::Interval {
      AsyncIoInterval::interval(interval)
    }

    fn interval_at(start: Instant, period: Duration) -> Self::Interval {
      AsyncIoInterval::interval_at(start, period)
    }

    fn interval_local(interval: Duration) -> Self::LocalInterval {
      AsyncIoInterval::interval(interval)
    }

    fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
      AsyncIoInterval::interval_at(start, period)
    }

    fn sleep(duration: Duration) -> Self::Sleep {
      use crate::time::AsyncSleepExt;

      AsyncIoSleep::sleep(duration)
    }

    fn sleep_until(instant: Instant) -> Self::Sleep {
      use crate::time::AsyncSleepExt;

      AsyncIoSleep::sleep_until(instant)
    }

    fn sleep_local(duration: Duration) -> Self::LocalSleep {
      use crate::time::AsyncSleepExt;

      AsyncIoSleep::sleep(duration)
    }

    fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
      use crate::time::AsyncSleepExt;

      AsyncIoSleep::sleep_until(instant)
    }

    fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncDelayExt;

      <AsyncIoDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
    }

    fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalDelayExt;

      <AsyncIoDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
    }

    fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncDelayExt;

      <AsyncIoDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
    }

    fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalDelayExt;

      <AsyncIoDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
    }

    fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncTimeout;

      <AsyncIoTimeout<F> as AsyncTimeout<F>>::timeout(duration, future)
    }

    fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncTimeout;

      <AsyncIoTimeout<F> as AsyncTimeout<F>>::timeout_at(deadline, future)
    }

    fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalTimeout;

      <AsyncIoTimeout<F> as AsyncLocalTimeout<F>>::timeout_local(duration, future)
    }

    fn timeout_local_at<F>(deadline: Instant, future: F) -> Self::LocalTimeout<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalTimeout;

      <AsyncIoTimeout<F> as AsyncLocalTimeout<F>>::timeout_local_at(deadline, future)
    }
  );
}
