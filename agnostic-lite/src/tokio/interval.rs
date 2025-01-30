use core::{
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::time::Instant;

use futures_util::stream::Stream;

use crate::time::{AsyncLocalInterval, AsyncLocalIntervalExt};

pin_project_lite::pin_project! {
  /// The [`AsyncInterval`] implementation for tokio runtime
  #[repr(transparent)]
  pub struct TokioInterval {
    #[pin]
    inner: ::tokio::time::Interval,
  }
}

impl From<::tokio::time::Interval> for TokioInterval {
  fn from(interval: ::tokio::time::Interval) -> Self {
    Self { inner: interval }
  }
}

impl From<TokioInterval> for ::tokio::time::Interval {
  fn from(interval: TokioInterval) -> Self {
    interval.inner
  }
}

impl Stream for TokioInterval {
  type Item = tokio::time::Instant;

  fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
    self
      .project()
      .inner
      .poll_tick(cx)
      .map(Some)
  }
}

impl AsyncLocalInterval for TokioInterval {
  type Instant = ::tokio::time::Instant;

  fn reset(&mut self, interval: Duration) {
    self.inner.reset_after(interval);
  }

  fn reset_at(&mut self, instant: Self::Instant) {
    self.inner.reset_at(instant);
  }
  
  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Self::Instant> {
    self.inner.poll_tick(cx)
  }
}

impl AsyncLocalIntervalExt for TokioInterval {
  fn interval_local(period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: tokio::time::interval_at((Instant::now() + period).into(), period),
    }
  }

  fn interval_local_at(start: Self::Instant, period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: tokio::time::interval_at(start, period),
    }
  }
}

#[cfg(test)]
mod tests {
  use futures::StreamExt;

  use super::TokioInterval;
  use crate::time::{AsyncInterval, AsyncIntervalExt};
  use tokio::time::{Duration, Instant};

  const INTERVAL: Duration = Duration::from_millis(100);
  const BOUND: Duration = Duration::from_millis(50);
  const IMMEDIATE: Duration = Duration::from_millis(1);

  #[tokio::test]
  async fn test_interval() {
    let start = Instant::now();
    let interval = TokioInterval::interval(INTERVAL);
    let mut interval = interval.take(3);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL - BOUND);
    assert!(elapsed >= INTERVAL - BOUND && elapsed <= INTERVAL + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL * 2 - BOUND);
    assert!(elapsed >= INTERVAL * 2 - BOUND && elapsed <= INTERVAL * 2 + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL * 3 - BOUND);
    assert!(elapsed >= INTERVAL * 3 - BOUND && elapsed <= INTERVAL * 3 + BOUND);

    assert!(interval.next().await.is_none());
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn test_interval_at() {
    let start = Instant::now();
    let interval = TokioInterval::interval_at(Instant::now(), INTERVAL);
    let mut interval = interval.take(4);

    // The first tick is immediate
    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins <= start + IMMEDIATE);
    assert!(elapsed <= IMMEDIATE + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL - BOUND);
    assert!(elapsed >= INTERVAL - BOUND && elapsed <= INTERVAL + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL * 2 - BOUND);
    assert!(elapsed >= INTERVAL * 2 - BOUND && elapsed <= INTERVAL * 2 + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL * 3 - BOUND);
    assert!(elapsed >= INTERVAL * 3 - BOUND && elapsed <= INTERVAL * 3 + BOUND);

    assert!(interval.next().await.is_none());
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn test_interval_reset() {
    let start = Instant::now();
    let mut interval = TokioInterval::interval(INTERVAL);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL - BOUND);
    assert!(elapsed >= INTERVAL - BOUND && elapsed <= INTERVAL + BOUND);

    // Reset the next tick to 2x
    interval.reset(INTERVAL * 2);
    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    // interval + 2x interval, so 3 here
    assert!(ins >= start + INTERVAL * 3 - BOUND);
    assert!(elapsed >= INTERVAL * 3 - BOUND && elapsed <= INTERVAL * 3 + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    // interval + 2x interval + interval, so 4 here
    assert!(ins >= start + INTERVAL * 4 - BOUND);
    assert!(elapsed >= INTERVAL * 4 - BOUND && elapsed <= INTERVAL * 4 + BOUND);
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn test_interval_reset_at() {
    let start = Instant::now();
    let mut interval = TokioInterval::interval(INTERVAL);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    assert!(ins >= start + INTERVAL);
    assert!(elapsed >= INTERVAL && elapsed <= INTERVAL + BOUND);

    // Reset the next tick to 2x
    interval.reset_at(start + INTERVAL * 3);
    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    // interval + 2x interval, so 3 here
    assert!(ins >= start + INTERVAL * 3);
    assert!(elapsed >= INTERVAL * 3 - BOUND && elapsed <= INTERVAL * 3 + BOUND);

    let ins = interval.next().await.unwrap();
    let elapsed = start.elapsed();
    // interval + 2x interval + interval, so 4 here
    assert!(ins >= start + INTERVAL * 4 - BOUND);
    assert!(elapsed >= INTERVAL * 4 - BOUND && elapsed <= INTERVAL * 4 + BOUND);
  }
}
