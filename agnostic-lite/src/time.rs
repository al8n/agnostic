use core::time::Duration;

mod delay;
mod interval;
mod sleep;
mod timeout;

pub use delay::*;
pub use interval::*;
pub use sleep::*;
pub use timeout::*;

macro_rules! instant_methods {
  () => {
    /// Returns an instant corresponding to "now".
    #[must_use]
    fn now() -> Self;

    /// Returns the amount of time elapsed since this instant.
    fn elapsed(&self) -> Duration;

    /// Returns `Some(t)` where `t` is the time `self + duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    fn checked_add(&self, duration: Duration) -> Option<Self>;

    /// Returns `Some(t)` where `t` is the time `self - duration` if `t` can be represented as
    /// `Instant` (which means it's inside the bounds of the underlying data structure), `None`
    /// otherwise.
    fn checked_sub(&self, duration: Duration) -> Option<Self>;

    /// Returns the amount of time elapsed from another instant to this one,
    /// or None if that instant is later than this one.
    fn checked_duration_since(&self, earlier: Self) -> Option<Duration>;

    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    fn duration_since(&self, earlier: Self) -> Duration {
      self.checked_duration_since(earlier).unwrap_or_default()
    }
    
    /// Returns the amount of time elapsed from another instant to this one,
    /// or zero duration if that instant is later than this one.
    fn saturating_duration_since(&self, earlier: Self) -> Duration {
      self.checked_duration_since(earlier).unwrap_or_default()
    }  
  };
}


/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`Duration`].
#[cfg(not(feature = "std"))]
pub trait Instant:
  Copy
  + Clone
  + PartialEq
  + Eq
  + PartialOrd
  + Ord
  + core::fmt::Debug
  + core::hash::Hash
  + core::ops::Add<Duration, Output = Self>
  + core::ops::AddAssign<Duration>
  + core::ops::Sub<Self>
  + core::ops::Sub<Duration>
  + core::ops::SubAssign<Duration>
  + Send
  + Sync
  + Unpin
  + 'static
{
  instant_methods!();
}

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`Duration`].
#[cfg(feature = "std")]
pub trait Instant:
  Copy
  + Clone
  + PartialEq
  + Eq
  + PartialOrd
  + Ord
  + core::fmt::Debug
  + core::hash::Hash
  + core::ops::Add<Duration, Output = Self>
  + core::ops::AddAssign<Duration>
  + core::ops::Sub<Self>
  + core::ops::Sub<Duration>
  + core::ops::SubAssign<Duration>
  + From<std::time::Instant>
  + Into<std::time::Instant>
  + Send
  + Sync
  + Unpin
  + 'static
{
  instant_methods!();
}

#[cfg(feature = "std")]
const _: () = {
  use std::time::Instant as StdInstant;

  impl Instant for StdInstant {
    #[inline]
    fn now() -> Self {
      StdInstant::now()
    }

    #[inline]
    fn elapsed(&self) -> Duration {
      StdInstant::elapsed(self)
    }

    #[inline]
    fn checked_add(&self, duration: Duration) -> Option<Self> {
      StdInstant::checked_add(self, duration)
    }

    #[inline]
    fn checked_sub(&self, duration: Duration) -> Option<Self> {
      StdInstant::checked_sub(self, duration)
    }

    #[inline]
    fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
      StdInstant::checked_duration_since(self, earlier)
    }
  }
};

#[cfg(all(feature = "tokio", feature = "time", feature = "std"))]
const _: () = {
  use tokio::time::Instant as TokioInstant;

  impl Instant for TokioInstant {
    #[inline]
    fn now() -> Self {
      TokioInstant::now()
    }

    #[inline]
    fn elapsed(&self) -> Duration {
      TokioInstant::elapsed(self)
    }

    #[inline]
    fn checked_add(&self, duration: Duration) -> Option<Self> {
      TokioInstant::checked_add(self, duration)
    }

    #[inline]
    fn checked_sub(&self, duration: Duration) -> Option<Self> {
      TokioInstant::checked_sub(self, duration)
    }

    #[inline]
    fn checked_duration_since(&self, earlier: Self) -> Option<Duration> {
      TokioInstant::checked_duration_since(self, earlier)
    }
  }
};