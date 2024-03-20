use std::{io, task::Poll};

use ::tokio::sync::mpsc;
use tokio_stream::wrappers::IntervalStream;

use super::*;

#[cfg(feature = "net")]
pub mod net;

struct DelayFuncHandle<F: Future> {
  handle: ::tokio::task::JoinHandle<Option<F::Output>>,
  reset_tx: mpsc::Sender<Duration>,
  stop_tx: mpsc::Sender<()>,
}

pub struct TokioDelay<F: Future> {
  handle: Option<DelayFuncHandle<F>>,
}

impl<F> Delay<F> for TokioDelay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self {
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let (reset_tx, mut reset_rx) = mpsc::channel(1);
    let handle = ::tokio::spawn(async move {
      let sleep = ::tokio::time::sleep(delay);
      ::tokio::pin!(sleep);
      loop {
        ::tokio::select! {
          _ = &mut sleep => {
            return Some(fut.await);
          },
          _ = stop_rx.recv() => return None,
          remaining = reset_rx.recv() => {
            if let Some(remaining) = remaining {
              sleep.as_mut().reset(::tokio::time::Instant::now() + remaining);
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
        stop_tx,
      }),
    }
  }

  fn reset(&mut self, dur: Duration) -> impl Future<Output = ()> + Send + '_ {
    async move {
      if let Some(handle) = &mut self.handle {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.reset_tx.send(dur).await;
      }
    }
  }

  fn cancel(&mut self) -> impl Future<Output = Option<F::Output>> + Send + '_ {
    async move {
      if let Some(handle) = self.handle.take() {
        if handle.handle.is_finished() {
          return match handle.handle.await {
            Ok(rst) => rst,
            Err(_) => None,
          };
        }

        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.stop_tx.send(()).await;
        return None;
      }
      None
    }
  }
}

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
  type BlockJoinHandle<R>
  where
    R: Send + 'static,
  = ::tokio::task::JoinHandle<R>;
  type Interval = TokioInterval;
  type LocalInterval = TokioInterval;
  type Sleep = TokioSleep;
  type LocalSleep = TokioSleep;
  type Delay<F> = TokioDelay<F> where F: Future + Send + 'static, F::Output: Send;
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

  fn yield_now() -> impl Future<Output = ()> + Send {
    ::tokio::task::yield_now()
  }

  fn interval(interval: Duration) -> Self::Interval {
    TokioInterval::interval(interval)
  }

  fn interval_local(interval: Duration) -> Self::LocalInterval {
    TokioInterval::interval(interval)
  }

  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    TokioInterval::interval_at(start, period)
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    TokioInterval::interval_at(start, period)
  }

  fn sleep(duration: Duration) -> Self::Sleep {
    TokioSleep::sleep(duration)
  }

  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    TokioSleep::sleep(duration)
  }

  fn sleep_until(instant: Instant) -> Self::Sleep {
    TokioSleep::sleep_until(instant)
  }

  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    TokioSleep::sleep_until(instant)
  }

  fn delay<F>(delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    TokioDelay::new(delay, fut)
  }
}
