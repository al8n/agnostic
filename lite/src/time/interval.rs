use std::{
  task::{Context, Poll},
  time::{Duration, Instant},
};

use futures_util::stream::Stream;

/// The interval abstraction for a runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncInterval: Stream<Item = Instant> + Send {
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

impl<T: Send + AsyncLocalInterval> AsyncInterval for T {
  fn reset(&mut self, interval: Duration) {
    AsyncLocalInterval::reset(self, interval)
  }

  fn reset_at(&mut self, instant: Instant) {
    AsyncLocalInterval::reset_at(self, instant)
  }

  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
    AsyncLocalInterval::poll_tick(self, cx)
  }
}

impl<T: Send + AsyncLocalIntervalExt> AsyncIntervalExt for T {
  fn interval(period: Duration) -> Self
  where
    Self: Sized,
  {
    AsyncLocalIntervalExt::interval_local(period)
  }

  fn interval_at(start: Instant, period: Duration) -> Self
  where
    Self: Sized,
  {
    AsyncLocalIntervalExt::interval_local_at(start, period)
  }
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

/// Like [`AsyncInterval`], but does not require `Send`.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncLocalInterval: Stream<Item = Instant> {
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

/// Extension trait for [`AsyncLocalInterval`].
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncLocalIntervalExt: AsyncInterval {
  /// Creates a timer that emits events periodically.
  fn interval_local(period: Duration) -> Self
  where
    Self: Sized;

  /// Creates a timer that emits events periodically, starting at `start`.
  fn interval_local_at(start: Instant, period: Duration) -> Self
  where
    Self: Sized;
}
