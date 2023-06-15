#![forbid(unsafe_code)]
#![deny(warnings)]

use std::{
  future::Future,
  time::{Duration, Instant},
};

use futures_util::Stream;

#[cfg(feature = "tokio")]
pub mod tokio;

#[async_trait::async_trait]
pub trait DelayFunc<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self;

  async fn reset(&mut self, dur: Duration);

  async fn cancel(&mut self) -> Option<F::Output>;
}

#[async_trait::async_trait]
pub trait Runtime {
  type JoinHandle<T>: Future;
  type Interval: Stream;
  type Delay: Future;
  type DelayFunc<F>: DelayFunc<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;
  type Timeout<F>: Future
  where
    F: Future;

  fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  fn interval(&self, interval: Duration) -> Self::Interval;

  fn interval_at(&self, start: Instant, period: Duration) -> Self::Interval;

  fn delay(&self, duration: Duration) -> Self::Delay;

  fn delayed_func<F>(&self, duration: Duration, fut: F) -> Self::DelayFunc<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;

  fn timeout<F>(&self, duration: Duration, future: F) -> Self::Timeout<F>
  where
    F: Future;

  fn timeout_at<F>(&self, instant: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future;
}
