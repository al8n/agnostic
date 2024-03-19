use std::{
  task::{Context, Poll},
  time::{Duration, Instant},
};

use futures_util::stream::Stream;

/// The interval abstraction for a runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncInterval: Stream<Item = Instant> {
  /// Resets the interval to a [`Duration`]. Sets the next tick after the specified [`Duration`].
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(&mut self, interval: Duration);

  /// Resets the interval to a specific instant. Sets the next tick to expire at the given instant.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset_at(&mut self, instant: Instant);

  /// Polls for the next instant in the interval to be reached.
  ///
  /// This method can return the following values:
  ///
  ///  * `Poll::Pending` if the next instant has not yet been reached.
  ///  * `Poll::Ready(instant)` if the next instant has been reached.
  ///
  /// When this method returns `Poll::Pending`, the current task is scheduled
  /// to receive a wakeup when the instant has elapsed. Note that on multiple
  /// calls to `poll_tick`, only the [`Waker`](std::task::Waker) from the
  /// [`Context`](std::task::Context) passed to the most recent call is scheduled to receive a
  /// wakeup.
  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant>;
}

/// Extension trait for [`AsyncInterval`].
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncIntervalExt: AsyncInterval {
  /// Creates a timer that emits events periodically.
  fn interval(period: Duration) -> Self
  where
    Self: Sized;

  /// Creates a timer that emits events periodically, starting at `start`.
  fn interval_at(start: Instant, period: Duration) -> Self
  where
    Self: Sized;
}

#[cfg(all(feature = "tokio", feature = "time"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "tokio", feature = "time"))))]
pub use _tokio::TokioInterval;

#[cfg(all(feature = "tokio", feature = "std"))]
mod _tokio {
  use super::*;
  use core::pin::Pin;

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
    type Item = Instant;

    fn poll_next(
      self: Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
      self
        .project()
        .inner
        .poll_tick(cx)
        .map(|ins| Some(ins.into()))
    }
  }

  impl AsyncInterval for TokioInterval {
    fn reset(&mut self, interval: Duration) {
      self.inner.reset_after(interval);
    }

    fn reset_at(&mut self, instant: Instant) {
      self.inner.reset_at(tokio::time::Instant::from_std(instant));
    }

    fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
      self.inner.poll_tick(cx).map(|ins| ins.into())
    }
  }

  impl AsyncIntervalExt for TokioInterval {
    fn interval(period: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: tokio::time::interval(period),
      }
    }

    fn interval_at(start: Instant, period: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: tokio::time::interval_at(tokio::time::Instant::from_std(start), period),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use futures::StreamExt;

    use super::*;

    const INTERVAL: Duration = Duration::from_millis(100);
    const BOUND: Duration = Duration::from_millis(20);
    const IMMEDIATE: Duration = Duration::from_millis(1);

    #[tokio::test]
    async fn test_object_safe() {
      let _: Box<dyn AsyncInterval> = Box::new(TokioInterval::interval(Duration::from_secs(1)));
    }

    #[tokio::test]
    async fn test_interval() {
      let start = Instant::now();
      let interval = TokioInterval::interval(INTERVAL);
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
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_interval_reset_at() {
      let start = Instant::now();
      let mut interval = TokioInterval::interval(INTERVAL);
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
    }
  }
}

#[cfg(feature = "async-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
pub use _async_io::AsyncIoInterval;

#[cfg(feature = "async-io")]
mod _async_io {
  use super::*;
  use core::pin::Pin;
  use futures_util::FutureExt;

  pin_project_lite::pin_project! {
    /// The [`AsyncInterval`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
    #[repr(transparent)]
    pub struct AsyncIoInterval {
      #[pin]
      inner: async_io::Timer,
    }
  }

  impl From<async_io::Timer> for AsyncIoInterval {
    fn from(timer: async_io::Timer) -> Self {
      Self { inner: timer }
    }
  }

  impl From<AsyncIoInterval> for async_io::Timer {
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

  impl AsyncInterval for AsyncIoInterval {
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

  impl AsyncIntervalExt for AsyncIoInterval {
    fn interval(period: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: async_io::Timer::interval_at(Instant::now(), period),
      }
    }

    fn interval_at(start: Instant, period: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: async_io::Timer::interval_at(start, period),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use futures::StreamExt;

    use super::*;

    const INTERVAL: Duration = Duration::from_millis(100);
    const BOUND: Duration = Duration::from_millis(20);
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
}

#[cfg(feature = "wasm-time")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm-time")))]
pub use _wasm::WasmInterval;

#[cfg(feature = "wasm-time")]
mod _wasm {
  use super::*;

  use crate::{AsyncSleep, AsyncSleepExt as _, WasmSleep};
  use core::pin::Pin;
  use futures_util::FutureExt;

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

  impl AsyncInterval for WasmInterval {
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

  impl AsyncIntervalExt for WasmInterval {
    fn interval(period: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: Box::pin(WasmSleep::sleep(period)),
        first: true,
      }
    }

    fn interval_at(start: Instant, period: Duration) -> Self
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

    use super::*;

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
}
