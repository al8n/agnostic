use std::{
  future::Future,
  task::Poll,
  time::{Duration, Instant},
};

use ::tokio::sync::mpsc;
use tokio_stream::wrappers::IntervalStream;

use super::*;

pub mod net;

struct DelayFuncHandle<F: Future> {
  handle: ::tokio::task::JoinHandle<Option<F::Output>>,
  reset_tx: mpsc::Sender<Duration>,
  stop_tx: mpsc::Sender<()>,
}

pub struct TokioWasmDelay<F: Future> {
  handle: Option<DelayFuncHandle<F>>,
}

#[cfg_attr(not(feature = "nightly"), async_trait::async_trait)]
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

  #[cfg(not(feature = "nightly"))]
  async fn reset(&mut self, dur: Duration) {
    if let Some(handle) = &mut self.handle {
      // if we fail to send a message, which means the rx has been dropped, and that thread has exited
      let _ = handle.reset_tx.send(dur).await;
    }
  }

  #[cfg(feature = "nightly")]
  fn reset(&mut self, dur: Duration) -> impl Future<Output = ()> + Send + '_ {
    async move {
      if let Some(handle) = &mut self.handle {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.reset_tx.send(dur).await;
      }
    }
  }

  #[cfg(not(feature = "nightly"))]
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

  #[cfg(feature = "nightly")]
  fn cancel(&mut self) -> impl Future<Output = Option<F::Output>> + Send + '_ {
    async move {
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
}

pin_project_lite::pin_project! {
  pub struct TokioWasmTimeout<F: Future> {
    #[pin]
    timeout: ::tokio::time::Timeout<F>,
  }
}

impl<F: Future> From<::tokio::time::Timeout<F>> for TokioWasmTimeout<F> {
  fn from(timeout: ::tokio::time::Timeout<F>) -> Self {
    Self { timeout }
  }
}

impl<F: Future> From<TokioWasmTimeout<F>> for ::tokio::time::Timeout<F> {
  fn from(timeout: TokioWasmTimeout<F>) -> Self {
    timeout.timeout
  }
}

impl<F: Future> Future for TokioWasmTimeout<F> {
  type Output = std::io::Result<F::Output>;

  fn poll(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    let this = self.project();
    match this.timeout.poll(cx) {
      Poll::Ready(Ok(t)) => Poll::Ready(Ok(t)),
      Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::TimedOut, e))),
      Poll::Pending => Poll::Pending,
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct TokioWasmRuntime;

impl core::fmt::Display for TokioWasmRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "tokio")
  }
}

#[cfg_attr(not(feature = "nightly"), async_trait::async_trait)]
impl Runtime for TokioWasmRuntime {
  type JoinHandle<T> = ::tokio::task::JoinHandle<T>;
  type Interval = IntervalStream;
  type Sleep = ::tokio::time::Sleep;
  type Delay<F> = TokioWasmDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = TokioWasmTimeout<F> where F: Future + Send;
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

  fn block_on<F: Future>(&self, f: F) -> F::Output {
    ::tokio::runtime::Handle::current().block_on(f)
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
    F: Future + Send,
  {
    TokioWasmTimeout {
      timeout: ::tokio::time::timeout(duration, fut),
    }
  }

  fn timeout_at<F>(&self, instant: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    TokioWasmTimeout {
      timeout: ::tokio::time::timeout_at(instant.into(), fut),
    }
  }
}
