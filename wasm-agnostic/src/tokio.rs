use std::{
  future::Future,
  time::{Duration, Instant},
};

use agnostic::{Delay, Runtime};
use tokio::sync::mpsc;
use tokio_stream::wrappers::IntervalStream;

pub mod net;

#[derive(Debug, Default, Copy, Clone)]
pub struct TokioWasmRuntime;

struct DelayFuncHandle<F: Future> {
  handle: ::tokio::task::JoinHandle<Option<F::Output>>,
  reset_tx: mpsc::Sender<Duration>,
  stop_tx: mpsc::Sender<()>,
}

pub struct TokioWasmDelay<F: Future> {
  handle: Option<DelayFuncHandle<F>>,
}

#[async_trait::async_trait]
impl<F> Delay<F> for TokioWasmDelay<F>
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

  async fn reset(&mut self, dur: Duration) {
    if let Some(handle) = &mut self.handle {
      // if we fail to send a message, which means the rx has been dropped, and that thread has exited
      let _ = handle.reset_tx.send(dur).await;
    }
  }

  async fn cancel(&mut self) -> Option<F::Output> {
    if let Some(handle) = self.handle.take() {
      if handle.handle.is_finished() {
        return match handle.handle.await {
          Ok(rst) => rst,
          Err(_) => None,
        };
      } else {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.stop_tx.send(()).await;
        return None;
      }
    }
    None
  }
}

impl core::fmt::Display for TokioWasmRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "wasm")
  }
}

impl Runtime for TokioWasmRuntime {
  type JoinHandle<T> = ::tokio::task::JoinHandle<T>;
  type Interval = IntervalStream;
  type Sleep = ::tokio::time::Sleep;
  type Delay<F> = TokioWasmDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = ::tokio::time::Timeout<F> where F: Future;
  type Net = self::net::TokioWasmNet;

  fn new() -> Self {
    Self
  }

  fn spawn<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    ::tokio::spawn(fut)
  }

  fn spawn_local<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    ::tokio::task::spawn_local(fut)
  }

  fn spawn_blocking<F, R>(&self, f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::tokio::task::spawn_blocking(f)
  }

  fn interval(&self, interval: Duration) -> Self::Interval {
    IntervalStream::new(::tokio::time::interval(interval))
  }

  fn interval_at(&self, start: Instant, period: Duration) -> Self::Interval {
    IntervalStream::new(::tokio::time::interval_at(start.into(), period))
  }

  fn sleep(&self, duration: Duration) -> Self::Sleep {
    ::tokio::time::sleep(duration)
  }

  fn sleep_until(&self, instant: Instant) -> Self::Sleep {
    ::tokio::time::sleep_until(instant.into())
  }

  fn delay<F>(&self, delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    TokioWasmDelay::new(delay, fut)
  }

  fn timeout<F>(&self, duration: Duration, fut: F) -> Self::Timeout<F>
  where
    F: Future,
  {
    ::tokio::time::timeout(duration, fut)
  }

  fn timeout_at<F>(&self, instant: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future,
  {
    ::tokio::time::timeout_at(instant.into(), fut)
  }
}
