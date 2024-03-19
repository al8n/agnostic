use std::{io, task::Poll};

use ::tokio::sync::mpsc;
use tokio_stream::wrappers::IntervalStream;

use super::*;

#[cfg(feature = "net")]
pub mod net;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimeoutError;

impl core::fmt::Display for TimeoutError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "timeout")
  }
}

impl std::error::Error for TimeoutError {}

pin_project_lite::pin_project! {
  pub struct TokioSleep {
    #[pin]
    inner: ::tokio::time::Sleep,
  }
}

impl From<::tokio::time::Sleep> for TokioSleep {
  fn from(sleep: ::tokio::time::Sleep) -> Self {
    Self { inner: sleep }
  }
}

impl From<TokioSleep> for ::tokio::time::Sleep {
  fn from(sleep: TokioSleep) -> Self {
    sleep.inner
  }
}

impl futures_util::Future for TokioSleep {
  type Output = Instant;

  fn poll(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Self::Output> {
    let this = self.project();
    let ddl = this.inner.deadline().into();
    match this.inner.poll(cx) {
      Poll::Ready(_) => Poll::Ready(ddl),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl super::Sleep for TokioSleep {
  fn reset(mut self: std::pin::Pin<&mut Self>, deadline: Instant) {
    self.project().inner.as_mut().reset(deadline.into())
  }
}

pub struct TokioInterval(IntervalStream);

impl From<IntervalStream> for TokioInterval {
  fn from(stream: IntervalStream) -> Self {
    Self(stream)
  }
}

impl From<TokioInterval> for IntervalStream {
  fn from(internal: TokioInterval) -> Self {
    internal.0
  }
}

impl futures_util::Stream for TokioInterval {
  type Item = Instant;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let pinned = std::pin::Pin::new(&mut self.0);
    match pinned.poll_next(cx) {
      Poll::Ready(Some(instant)) => Poll::Ready(Some(instant.into())),
      Poll::Ready(None) => Poll::Ready(None),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl super::Interval for TokioInterval {}

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

pin_project_lite::pin_project! {
  pub struct TokioTimeout<F> {
    #[pin] inner: ::tokio::time::Timeout<F>
  }
}

impl<F> From<::tokio::time::Timeout<F>> for TokioTimeout<F> {
  fn from(timeout: ::tokio::time::Timeout<F>) -> Self {
    Self { inner: timeout }
  }
}

impl<F> From<TokioTimeout<F>> for ::tokio::time::Timeout<F> {
  fn from(timeout: TokioTimeout<F>) -> Self {
    timeout.inner
  }
}

impl<F: Future> Future for TokioTimeout<F> {
  type Output = Result<F::Output, Elapsed>;

  fn poll(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Self::Output> {
    match self.project().inner.poll(cx) {
      Poll::Ready(Ok(rst)) => Poll::Ready(Ok(rst)),
      Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl<F: Future + Send> Timeoutable<F> for TokioTimeout<F> {
  fn poll_elapsed(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<F::Output, Elapsed>> {
    match self.poll(cx) {
      Poll::Ready(Ok(rst)) => Poll::Ready(Ok(rst)),
      Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
      Poll::Pending => Poll::Pending,
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
  type JoinHandle<T> = ::tokio::task::JoinHandle<T>
  where
    T: Send + 'static,
    <Self::JoinHandle<T> as Future>::Output: Send;
  type BlockJoinHandle<R>
  where
    R: Send + 'static,
  = ::tokio::task::JoinHandle<R>;
  type LocalJoinHandle<F> = ::tokio::task::JoinHandle<F>;
  type Interval = TokioInterval;
  type Sleep = TokioSleep;
  type Delay<F> = TokioDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = TokioTimeout<F> where F: Future + Send;

  #[cfg(feature = "net")]
  type Net = self::net::TokioNet;

  fn new() -> Self {
    Self
  }

  fn spawn<F>(fut: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    ::tokio::spawn(fut)
  }

  fn spawn_local<F>(fut: F) -> Self::LocalJoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    ::tokio::task::spawn_local(fut)
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
    IntervalStream::new(::tokio::time::interval(interval)).into()
  }

  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    IntervalStream::new(::tokio::time::interval_at(start.into(), period)).into()
  }

  fn sleep(duration: Duration) -> Self::Sleep {
    ::tokio::time::sleep(duration).into()
  }

  fn sleep_until(instant: Instant) -> Self::Sleep {
    ::tokio::time::sleep_until(instant.into()).into()
  }

  fn yield_now() -> impl Future<Output = ()> + Send {
    ::tokio::task::yield_now()
  }

  fn delay<F>(delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    TokioDelay::new(delay, fut)
  }

  fn timeout<F>(duration: Duration, fut: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    ::tokio::time::timeout(duration, fut).into()
  }

  fn timeout_at<F>(deadline: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    ::tokio::time::timeout_at(deadline.into(), fut).into()
  }
}
