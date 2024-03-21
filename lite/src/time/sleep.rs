use std::{
  future::Future,
  pin::Pin,
  time::{Duration, Instant},
};

/// The sleep abstraction for a runtime.
pub trait AsyncSleep: Future<Output = Instant> + Send {
  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: Pin<&mut Self>, deadline: Instant);
}

/// Extension trait for [`AsyncSleep`].
pub trait AsyncSleepExt: AsyncSleep {
  /// Creates a timer that emits an event once after the given duration of time.
  fn sleep(after: Duration) -> Self
  where
    Self: Sized;

  /// Creates a timer that emits an event once at the given time instant.
  fn sleep_until(deadline: Instant) -> Self
  where
    Self: Sized;
}

impl<T: Send + AsyncLocalSleep> AsyncSleep for T {
  fn reset(self: Pin<&mut Self>, deadline: Instant) {
    AsyncLocalSleep::reset(self, deadline)
  }
}

impl<T: Send + AsyncLocalSleepExt> AsyncSleepExt for T {
  fn sleep(after: Duration) -> Self
  where
    Self: Sized,
  {
    AsyncLocalSleepExt::sleep_local(after)
  }

  fn sleep_until(deadline: Instant) -> Self
  where
    Self: Sized,
  {
    AsyncLocalSleepExt::sleep_local_until(deadline)
  }
}

/// Like [`AsyncSleep`], but does not requires `Send`.
pub trait AsyncLocalSleep: Future<Output = Instant> {
  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: Pin<&mut Self>, deadline: Instant);
}

/// Extension trait for [`AsyncLocalSleep`].
pub trait AsyncLocalSleepExt: AsyncLocalSleep {
  /// Creates a timer that emits an event once after the given duration of time.
  fn sleep_local(after: Duration) -> Self
  where
    Self: Sized;

  /// Creates a timer that emits an event once at the given time instant.
  fn sleep_local_until(deadline: Instant) -> Self
  where
    Self: Sized;
}
