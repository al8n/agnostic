use core::{
  future::Future,
  pin::Pin,
  time::Duration,
};

/// The sleep abstraction for a runtime.
pub trait AsyncSleep: Future<Output = Self::Instant> + Send {
  /// The instant type
  type Instant: super::Instant + Send;

  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: Pin<&mut Self>, deadline: Self::Instant);
}

/// Extension trait for [`AsyncSleep`].
pub trait AsyncSleepExt: AsyncSleep {
  /// Creates a timer that emits an event once after the given duration of time.
  fn sleep(after: Duration) -> Self
  where
    Self: Sized;

  /// Creates a timer that emits an event once at the given time instant.
  fn sleep_until(deadline: Self::Instant) -> Self
  where
    Self: Sized;
}

impl<T: Send + AsyncLocalSleep> AsyncSleep for T
where
  T: Send + AsyncLocalSleep,
  T::Instant: Send,
{
  type Instant = T::Instant;

  fn reset(self: Pin<&mut Self>, deadline: Self::Instant) {
    AsyncLocalSleep::reset(self, deadline)
  }
}

impl<T> AsyncSleepExt for T
where
  T: Send + AsyncLocalSleepExt,
  T::Instant: Send,
{
  fn sleep(after: Duration) -> Self
  where
    Self: Sized,
  {
    AsyncLocalSleepExt::sleep_local(after)
  }

  fn sleep_until(deadline: Self::Instant) -> Self
  where
    Self: Sized,
  {
    AsyncLocalSleepExt::sleep_local_until(deadline)
  }
}

/// Like [`AsyncSleep`], but does not requires `Send`.
pub trait AsyncLocalSleep: Future<Output = Self::Instant> {
  /// The instant type
  type Instant: super::Instant;

  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: Pin<&mut Self>, deadline: Self::Instant);
}

/// Extension trait for [`AsyncLocalSleep`].
pub trait AsyncLocalSleepExt: AsyncLocalSleep {
  /// Creates a timer that emits an event once after the given duration of time.
  fn sleep_local(after: Duration) -> Self
  where
    Self: Sized;

  /// Creates a timer that emits an event once at the given time instant.
  fn sleep_local_until(deadline: Self::Instant) -> Self
  where
    Self: Sized;
}
