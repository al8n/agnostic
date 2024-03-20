use core::{
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::time::Instant;

use futures_util::{stream::Stream, FutureExt};

use crate::time::{AsyncLocalInterval, AsyncLocalIntervalExt, AsyncSleep, AsyncSleepExt};

use super::WasmSleep;

pin_project_lite::pin_project! {
  /// The [`AsyncInterval`] implementation for wasm runtime.
  ///
  /// **Note:** `WasmInterval` is not accurate below second level.
  pub struct WasmInterval {
    #[pin]
    inner: Pin<Box<WasmSleep>>,
    first: bool,
  }
}

impl Stream for WasmInterval {
  type Item = Instant;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    if self.first {
      self.first = false;
      return Poll::Ready(Some(self.inner.ddl - self.inner.duration));
    }

    let mut this = self.project();
    match this.inner.poll_unpin(cx) {
      Poll::Ready(ins) => {
        let duration = this.inner.duration;
        Pin::new(&mut **this.inner).reset(Instant::now() + duration);
        Poll::Ready(Some(ins))
      }
      Poll::Pending => Poll::Pending,
    }
  }
}

impl AsyncLocalInterval for WasmInterval {
  fn reset(&mut self, interval: Duration) {
    Pin::new(&mut *self.inner).reset(Instant::now() + interval);
  }

  fn reset_at(&mut self, instant: Instant) {
    Pin::new(&mut *self.inner).reset(instant);
  }

  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
    if self.first {
      self.first = false;
      return Poll::Ready(self.inner.ddl - self.inner.duration);
    }

    let duration = self.inner.duration;
    let mut this = Pin::new(&mut *self.inner);
    match this.poll_unpin(cx) {
      Poll::Ready(ins) => {
        this.reset(Instant::now() + duration);
        Poll::Ready(ins)
      }
      Poll::Pending => Poll::Pending,
    }
  }
}

impl AsyncLocalIntervalExt for WasmInterval {
  fn interval_local(period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: Box::pin(WasmSleep::sleep(period)),
      first: true,
    }
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: Box::pin(WasmSleep::sleep_until(start + period)),
      first: true,
    }
  }
}

#[cfg(test)]
mod tests {
  use futures::StreamExt;

  use super::WasmInterval;
  use crate::time::{AsyncInterval, AsyncIntervalExt};
  use std::time::{Duration, Instant};

  const INTERVAL: Duration = Duration::from_millis(100);
  const BOUND: Duration = Duration::from_millis(20);
  const IMMEDIATE: Duration = Duration::from_millis(1);

  #[test]
  fn test_object_safe() {
    let _: Box<dyn AsyncInterval> = Box::new(WasmInterval::interval(Duration::from_secs(1)));
  }

  #[test]
  fn test_interval() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let interval = WasmInterval::interval(INTERVAL);
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
      let interval = WasmInterval::interval_at(Instant::now(), INTERVAL);
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
      let mut interval = WasmInterval::interval(INTERVAL);
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
      let mut interval = WasmInterval::interval(INTERVAL);
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
