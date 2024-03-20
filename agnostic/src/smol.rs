use super::*;

/// Network abstractions for [`smol`](::smol) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

// TODO: remove this when quinn support SmolRuntime
#[cfg(all(feature = "quinn", feature = "net"))]
mod quinn_;

/// [`Runtime`] implementation for [`smol`](::smol)
#[derive(Debug, Copy, Clone)]
pub struct SmolRuntime;

impl core::fmt::Display for SmolRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "smol")
  }
}

impl Runtime for SmolRuntime {
  type Spawner = SmolSpawner;
  type LocalSpawner = SmolLocalSpawner;
  type BlockJoinHandle<R> = ::smol::Task<R> where R: Send + 'static;
  type Interval = AsyncIoInterval;
  type LocalInterval = AsyncIoInterval;
  type Sleep = AsyncIoSleep;
  type LocalSleep = AsyncIoSleep;
  type Delay<F> = AsyncIoDelay<F> where F: Future + Send;

  type LocalDelay<F> = AsyncIoDelay<F> where F: Future;

  type Timeout<F> = AsyncIoTimeout<F> where F: Future + Send;
  type LocalTimeout<F> = AsyncIoTimeout<F> where F: Future;

  #[cfg(feature = "net")]
  type Net = net::SmolNet;

  fn new() -> Self {
    Self
  }

  fn spawn_blocking<F, R>(f: F) -> Self::BlockJoinHandle<R>
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
    ::smol::unblock(f).detach();
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::smol::block_on(f)
  }

  fn interval(duration: Duration) -> Self::Interval {
    AsyncIoInterval::interval(duration)
  }

  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    AsyncIoInterval::interval_at(start, period)
  }

  fn interval_local(duration: Duration) -> Self::LocalInterval {
    AsyncIoInterval::interval(duration)
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    AsyncIoInterval::interval_at(start, period)
  }

  fn sleep(duration: Duration) -> Self::Sleep {
    AsyncIoSleep::sleep(duration)
  }
  fn sleep_until(instant: Instant) -> Self::Sleep {
    AsyncIoSleep::sleep_until(instant)
  }

  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    AsyncIoSleep::sleep(duration)
  }

  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    AsyncIoSleep::sleep_until(instant)
  }

  async fn yield_now() {
    ::smol::future::yield_now().await
  }

  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    <AsyncIoDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
  }

  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    <AsyncIoDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    <AsyncIoDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
  }

  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    <AsyncIoDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }
}
