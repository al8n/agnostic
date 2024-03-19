use std::{pin::Pin, task::{Context, Poll}, time::{Duration, Instant}};

use futures_util::stream::Stream;

/// The interval abstraction for a runtime.
pub trait AsyncInterval: Stream<Item = Instant> + Send {
  /// Resets the interval to a [`Duration`].
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(&mut self, interval: Duration);

  /// Resets the interval to a specific instant.
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


#[cfg(all(feature = "tokio", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "tokio"))))]
pub use _tokio::TokioInterval;


#[cfg(all(feature = "tokio", feature = "std"))]
mod _tokio {
  use super::*;

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
      self.project().inner.poll_tick(cx).map(|ins| Some(ins.into()))
    }
  }

  impl AsyncInterval for TokioInterval {
    fn reset(&mut self, interval: Duration) {
      self.inner.reset_at(tokio::time::Instant::now() + interval);
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
        Self: Sized {
      Self {
        inner: tokio::time::interval(period),
      }
    }

    fn interval_at(start: Instant, period: Duration) -> Self
      where
        Self: Sized {
      Self {
        inner: tokio::time::interval_at(tokio::time::Instant::from_std(start), period),
      }
    }
  }

  #[test]
  fn test_object_safe() {
    let _: Box<dyn AsyncInterval> = Box::new(TokioInterval::interval(Duration::from_secs(1)));
  }
}

#[cfg(all(feature = "async-io", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async-io"))))]
pub use _async_io::AsyncIoInterval;

#[cfg(all(feature = "async-io", feature = "std"))]
mod _async_io {
  use super::*;
  use futures_lite::FutureExt;

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

    fn poll_next(
      self: Pin<&mut Self>,
      cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
      self.project().inner.poll_next(cx)
    }
  }

  impl AsyncInterval for AsyncIoInterval {
    fn reset(&mut self, interval: Duration) {
      self.inner.set_interval(interval);
    }

    fn reset_at(&mut self, deadline: Instant) {
      let now = Instant::now();
      let period = deadline - now;
      self.inner.set_interval_at(now, period);
    }

    fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
      self.inner.poll(cx)
    }
  }

  impl AsyncIntervalExt for AsyncIoInterval {
    fn interval(period: Duration) -> Self
      where
        Self: Sized {
      Self {
        inner: async_io::Timer::interval(period),
      }
    }

    fn interval_at(start: Instant, period: Duration) -> Self
      where
        Self: Sized {
      Self {
        inner: async_io::Timer::interval_at(start, period),
      }
    }
  }

  #[test]
  fn test_object_safe() {
    let _: Box<dyn AsyncInterval> = Box::new(AsyncIoInterval::interval(Duration::from_secs(1)));
  }
}

