use core::{
  future::Future,
  pin::Pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{Context, Poll},
};
use std::time::{Duration, Instant};

use super::{AsyncLocalSleep, AsyncLocalSleepExt};

/// Delay is aborted
#[derive(Debug, Clone, Copy)]
pub struct Aborted;

impl core::fmt::Display for Aborted {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "delay aborted")
  }
}

impl std::error::Error for Aborted {}

fn _assert1(_: Box<dyn AsyncLocalDelay<impl Future>>) {}
fn _assert2(_: Box<dyn AsyncDelay<impl Future>>) {}

/// Simlilar to Go's `time.AfterFunc`
pub trait AsyncDelay<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future + Send,
{
  /// Abort the delay, if future has not yet completed, then it will never be polled again.
  fn abort(&self);

  /// Cancel the delay, running the future immediately
  fn cancel(&self);

  /// Reset the delay to a new duration
  fn reset(self: Pin<&mut Self>, dur: Duration);

  /// Resets the delay to a new instant
  fn reset_at(self: Pin<&mut Self>, at: Instant);
}

/// Extension trait for [`AsyncLocalDelay`]
pub trait AsyncDelayExt<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future + Send,
{
  /// Create a new delay, the future will be polled after the duration has elapsed
  fn delay(dur: Duration, fut: F) -> Self;

  /// Create a new delay, the future will be polled after the instant has elapsed
  fn delay_at(at: Instant, fut: F) -> Self;
}

impl<F: Future + Send, T> AsyncDelay<F> for T
where
  T: AsyncLocalDelay<F>,
{
  fn abort(&self) {
    AsyncLocalDelay::abort(self);
  }

  fn cancel(&self) {
    AsyncLocalDelay::cancel(self);
  }

  fn reset(self: Pin<&mut Self>, dur: Duration) {
    AsyncLocalDelay::reset(self, dur);
  }

  fn reset_at(self: Pin<&mut Self>, at: Instant) {
    AsyncLocalDelay::reset_at(self, at);
  }
}

impl<F: Future + Send, T> AsyncDelayExt<F> for T
where
  T: AsyncLocalDelayExt<F>,
{
  fn delay(dur: Duration, fut: F) -> Self {
    AsyncLocalDelayExt::delay(dur, fut)
  }

  fn delay_at(at: Instant, fut: F) -> Self {
    AsyncLocalDelayExt::delay_at(at, fut)
  }
}

/// Like [`Delay`] but does not require `Send`
pub trait AsyncLocalDelay<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future,
{
  /// Abort the delay, if future has not yet completed, then it will never be polled again.
  fn abort(&self);

  /// Cancel the delay, running the future immediately
  fn cancel(&self);

  /// Reset the delay to a new duration
  fn reset(self: Pin<&mut Self>, dur: Duration);

  /// Resets the delay to a new instant
  fn reset_at(self: Pin<&mut Self>, at: Instant);
}

/// Extension trait for [`AsyncLocalDelay`]
pub trait AsyncLocalDelayExt<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future,
{
  /// Create a new delay, the future will be polled after the duration has elapsed
  fn delay(dur: Duration, fut: F) -> Self;

  /// Create a new delay, the future will be polled after the instant has elapsed
  fn delay_at(at: Instant, fut: F) -> Self;
}

pin_project_lite::pin_project! {
  /// [`AsyncDelay`] implementation for wasm bindgen runtime
  pub struct Delay<F, S> {
    #[pin]
    fut: Option<F>,
    #[pin]
    sleep: S,
    aborted: AtomicBool,
    canceled: AtomicBool,
  }
}

impl<F: Future, S: Future> Future for Delay<F, S> {
  type Output = Result<F::Output, Aborted>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if self.aborted.load(Ordering::Acquire) {
      return Poll::Ready(Err(Aborted));
    }

    let this = self.project();
    if !this.canceled.load(Ordering::Acquire) && !this.sleep.poll(cx).is_ready() {
      return Poll::Pending;
    }

    if let Some(fut) = this.fut.as_pin_mut() {
      return fut.poll(cx).map(Ok);
    }

    Poll::Pending
  }
}

impl<F, S> AsyncLocalDelay<F> for Delay<F, S>
where
  F: Future,
  S: AsyncLocalSleep,
{
  fn abort(&self) {
    self.aborted.store(true, Ordering::Release)
  }

  fn cancel(&self) {
    self.canceled.store(true, Ordering::Release)
  }

  fn reset(self: Pin<&mut Self>, dur: Duration) {
    self.project().sleep.as_mut().reset(Instant::now() + dur);
  }

  fn reset_at(self: Pin<&mut Self>, at: Instant) {
    self.project().sleep.as_mut().reset(at);
  }
}

impl<F, S> AsyncLocalDelayExt<F> for Delay<F, S>
where
  F: Future,
  S: AsyncLocalSleepExt,
{
  fn delay(dur: Duration, fut: F) -> Self {
    Self {
      fut: Some(fut),
      sleep: S::sleep_local(dur),
      aborted: AtomicBool::new(false),
      canceled: AtomicBool::new(false),
    }
  }

  fn delay_at(at: Instant, fut: F) -> Self {
    Self {
      fut: Some(fut),
      sleep: S::sleep_local_until(at),
      aborted: AtomicBool::new(false),
      canceled: AtomicBool::new(false),
    }
  }
}

#[test]
fn test_aborted_error() {
  assert_eq!(Aborted.to_string(), "delay aborted");
}
