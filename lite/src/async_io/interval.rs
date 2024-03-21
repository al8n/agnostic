use core::{
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::time::Instant;

use ::async_io::Timer;
use futures_util::{stream::Stream, FutureExt};

use crate::time::{AsyncLocalInterval, AsyncLocalIntervalExt};

pin_project_lite::pin_project! {
  /// The [`AsyncInterval`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
  #[repr(transparent)]
  pub struct AsyncIoInterval {
    #[pin]
    inner: Timer,
  }
}

impl From<Timer> for AsyncIoInterval {
  fn from(timer: Timer) -> Self {
    Self { inner: timer }
  }
}

impl From<AsyncIoInterval> for Timer {
  fn from(interval: AsyncIoInterval) -> Self {
    interval.inner
  }
}

impl Stream for AsyncIoInterval {
  type Item = Instant;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.project().inner.poll_next(cx)
  }
}

impl AsyncLocalInterval for AsyncIoInterval {
  fn reset(&mut self, interval: Duration) {
    self.inner.set_after(interval)
  }

  fn reset_at(&mut self, deadline: Instant) {
    self.inner.set_at(deadline);
  }

  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
    self.inner.poll_unpin(cx)
  }
}

impl AsyncLocalIntervalExt for AsyncIoInterval {
  fn interval_local(period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: Timer::interval_at(Instant::now(), period),
    }
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: Timer::interval_at(start, period),
    }
  }
}

#[cfg(test)]
mod tests {
  use futures::StreamExt;

  use super::AsyncIoInterval;
  use crate::time::{AsyncInterval, AsyncIntervalExt};
  use std::time::{Duration, Instant};

  const INTERVAL: Duration = Duration::from_millis(100);
  const BOUND: Duration = Duration::from_millis(50);
  const IMMEDIATE: Duration = Duration::from_millis(1);

  #[test]
  fn test_object_safe() {
    let _: Box<dyn AsyncInterval> = Box::new(AsyncIoInterval::interval(Duration::from_secs(1)));
  }

  #[test]
  fn test_interval() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let interval = AsyncIoInterval::interval(INTERVAL);
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
    });
  }

  #[test]
  fn test_interval_at() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let interval = AsyncIoInterval::interval_at(Instant::now(), INTERVAL);
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
    });
  }

  #[test]
  fn test_interval_reset() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let mut interval = AsyncIoInterval::interval(INTERVAL);

      // The first tick is immediate
      let ins = interval.next().await.unwrap();
      let elapsed = start.elapsed();
      assert!(ins <= start + IMMEDIATE);
      assert!(elapsed <= IMMEDIATE + BOUND);

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
    });
  }

  #[test]
  fn test_interval_reset_at() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let mut interval = AsyncIoInterval::interval(INTERVAL);

      // The first tick is immediate
      let ins = interval.next().await.unwrap();
      let elapsed = start.elapsed();
      assert!(ins <= start + IMMEDIATE);
      assert!(elapsed <= IMMEDIATE + BOUND);

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
    });
  }
}
