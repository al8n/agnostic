use super::*;

/// Network abstractions for [`tokio`](::tokio) runtime
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

/// [`Runtime`] implementation for [`tokio`](::tokio)
#[derive(Debug, Copy, Clone)]
pub struct TokioRuntime;

impl core::fmt::Display for TokioRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "tokio")
  }
}

impl Runtime for TokioRuntime {
  type Spawner = TokioSpawner;
  type LocalSpawner = TokioLocalSpawner;
  type BlockJoinHandle<R> = ::tokio::task::JoinHandle<R> where R: Send + 'static;
  type Interval = TokioInterval;
  type LocalInterval = TokioInterval;
  type Sleep = TokioSleep;
  type LocalSleep = TokioSleep;
  type Delay<F> = TokioDelay<F> where F: Future + Send;
  type LocalDelay<F> = TokioDelay<F> where F: Future;
  type Timeout<F> = TokioTimeout<F> where F: Future + Send;
  type LocalTimeout<F> = TokioTimeout<F> where F: Future;

  #[cfg(feature = "net")]
  type Net = self::net::TokioNet;

  fn new() -> Self {
    Self
  }

  fn spawn_blocking<F, R>(_f: F) -> Self::BlockJoinHandle<R>
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

  fn block_on<F: Future>(f: F) -> F::Output {
    ::tokio::runtime::Handle::current().block_on(f)
  }

  fn interval(interval: Duration) -> Self::Interval {
    TokioInterval::interval(interval)
  }

  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    TokioInterval::interval_at(start, period)
  }

  fn interval_local(interval: Duration) -> Self::LocalInterval {
    TokioInterval::interval(interval)
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    TokioInterval::interval_at(start, period)
  }

  fn sleep(duration: Duration) -> Self::Sleep {
    TokioSleep::sleep(duration)
  }

  fn sleep_until(instant: Instant) -> Self::Sleep {
    TokioSleep::sleep_until(instant)
  }

  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    TokioSleep::sleep(duration)
  }

  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    TokioSleep::sleep_until(instant)
  }

  fn yield_now() -> impl Future<Output = ()> + Send {
    ::tokio::task::yield_now()
  }

  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    <TokioDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
  }

  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    <TokioDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    <TokioDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
  }

  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    <TokioDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }
}
