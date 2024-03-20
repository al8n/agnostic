use super::*;
use ::async_std::channel;
use async_io::Timer;
use futures_util::FutureExt;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

#[cfg(feature = "net")]
pub mod net;

struct DelayFuncHandle<F: Future> {
  handle: ::async_std::task::JoinHandle<Option<F::Output>>,
  reset_tx: channel::Sender<Duration>,
  finished: Arc<AtomicBool>,
}

pub struct AsyncStdDelay<F: Future> {
  handle: Option<DelayFuncHandle<F>>,
}

impl<F> Delay<F> for AsyncStdDelay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self {
    let finished = Arc::new(AtomicBool::new(false));
    let ff = finished.clone();
    let (reset_tx, reset_rx) = channel::bounded(1);
    let handle = ::async_std::task::spawn(async move {
      let mut sleep = Timer::after(delay);
      loop {
        ::futures_util::select! {
          _ = sleep.fuse() => {
            let rst = fut.await;
            finished.store(true, ::std::sync::atomic::Ordering::SeqCst);
            return Some(rst);
          },
          remaining = reset_rx.recv().fuse() => {
            if let Ok(remaining) = remaining {
              sleep = Timer::after(remaining);
            } else {
              return None;
            }
          }
        }
      }
    });
    Self {
      handle: Some(DelayFuncHandle {
        reset_tx,
        handle,
        finished: ff,
      }),
    }
  }

  fn reset(&mut self, dur: Duration) -> impl Future<Output = ()> + Send + '_ {
    async move {
      if let Some(handle) = &mut self.handle {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.reset_tx.try_send(dur);
      }
    }
  }

  fn cancel(&mut self) -> impl Future<Output = Option<F::Output>> + Send + '_ {
    async move {
      if let Some(handle) = self.handle.take() {
        if handle.finished.load(Ordering::SeqCst) {
          return handle.handle.await;
        } else {
          // if we fail to send a message, which means the rx has been dropped, and that thread has exited
          handle.handle.cancel().await;
          return None;
        }
      }
      None
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct AsyncStdRuntime;

impl core::fmt::Display for AsyncStdRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "async-std")
  }
}

impl Runtime for AsyncStdRuntime {
  type Spawner = AsyncStdSpawner;
  type LocalSpawner = AsyncStdLocalSpawner;
  type Interval = AsyncIoInterval;
  type LocalInterval = AsyncIoInterval;
  type Sleep = AsyncIoSleep;
  type LocalSleep = AsyncIoSleep;
  type Timeout<F> = AsyncIoTimeout<F> where F: Future + Send;
  type LocalTimeout<F> = AsyncIoTimeout<F> where F: Future;

  type BlockJoinHandle<R>
  where
    R: Send + 'static,
  = ::async_std::task::JoinHandle<R>;
  type Delay<F> = AsyncStdDelay<F> where F: Future + Send + 'static, F::Output: Send;

  #[cfg(feature = "net")]
  type Net = net::AsyncStdNet;

  fn new() -> Self {
    Self
  }

  fn spawn_blocking<F, R>(f: F) -> Self::BlockJoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::async_std::task::spawn_blocking(f)
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    ::async_std::task::block_on(f)
  }

  async fn yield_now() {
    ::async_std::task::yield_now().await
  }

  fn delay<F>(delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    AsyncStdDelay::new(delay, fut)
  }

  fn interval(duration: Duration) -> Self::Interval {
    AsyncIoInterval::interval(duration)
  }

  fn interval_local(duration: Duration) -> Self::LocalInterval {
    AsyncIoInterval::interval(duration)
  }

  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    AsyncIoInterval::interval_at(start, period)
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    AsyncIoInterval::interval_at(start, period)
  }

  fn sleep(duration: Duration) -> Self::Sleep {
    AsyncIoSleep::sleep(duration)
  }

  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    AsyncIoSleep::sleep(duration)
  }

  fn sleep_until(instant: Instant) -> Self::Sleep {
    AsyncIoSleep::sleep_until(instant)
  }

  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    AsyncIoSleep::sleep_until(instant)
  }
}
