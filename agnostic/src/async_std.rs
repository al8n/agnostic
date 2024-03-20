use super::*;

/// Network abstractions for [`async-std`](::async_std) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

/// [`Runtime`] implementation for [`async-std`](::async_std)
#[derive(Debug, Clone, Copy)]
pub struct AsyncStdRuntime;

impl core::fmt::Display for AsyncStdRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "async-std")
  }
}

impl Runtime for AsyncStdRuntime {
  type Spawner = AsyncStdSpawner;
  type LocalSpawner = AsyncStdSpawner;
  type BlockingSpawner = AsyncStdSpawner;
  type Interval = AsyncIoInterval;
  type LocalInterval = AsyncIoInterval;
  type Sleep = AsyncIoSleep;
  type LocalSleep = AsyncIoSleep;
  type Delay<F> = AsyncIoDelay<F> where F: Future + Send;

  type LocalDelay<F> = AsyncIoDelay<F> where F: Future;
  type Timeout<F> = AsyncIoTimeout<F> where F: Future + Send;
  type LocalTimeout<F> = AsyncIoTimeout<F> where F: Future;

  #[cfg(feature = "net")]
  type Net = net::AsyncStdNet;

  fn new() -> Self {
    Self
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::async_std::task::block_on(f)
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
    ::async_std::task::yield_now().await
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
