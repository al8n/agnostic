use std::{future::Future, pin::Pin, task::{Context, Poll}, time::{Duration, Instant}};

/// The sleep abstraction for a runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncSleep: Future<Output = Instant> + Send {
  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: Pin<&mut Self>, deadline: Instant);
}

/// Extension trait for [`AsyncSleep`].
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncSleepExt: AsyncSleep {
  /// Creates a timer that emits an event once after the given duration of time.
  fn sleep(after: Duration) -> Self where Self: Sized;

  /// Creates a timer that emits an event once at the given time instant.
  fn sleep_until(&self, deadline: Instant) -> Self where Self: Sized;
}

#[cfg(all(feature = "tokio", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "tokio"))))]
pub use _tokio::TokioSleep;

#[cfg(all(feature = "tokio", feature = "std"))]
mod _tokio {
  use super::*;

  pin_project_lite::pin_project! {
    /// The [`AsyncSleep`] implementation for tokio runtime
    #[cfg_attr(docsrs, doc(cfg(all(feature = "sleep", feature = "tokio"))))]
    #[repr(transparent)]
    pub struct TokioSleep {
      #[pin]
      inner: ::tokio::time::Sleep,
    }
  }

  impl From<::tokio::time::Sleep> for TokioSleep {
    fn from(sleep: ::tokio::time::Sleep) -> Self {
      Self { inner: sleep }
    }
  }
  
  impl From<TokioSleep> for ::tokio::time::Sleep {
    fn from(sleep: TokioSleep) -> Self {
      sleep.inner
    }
  }
  
  impl Future for TokioSleep {
    type Output = Instant;
  
    fn poll(
      self: std::pin::Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
      let this = self.project();
      let ddl = this.inner.deadline().into();
      match this.inner.poll(cx) {
        Poll::Ready(_) => Poll::Ready(ddl),
        Poll::Pending => Poll::Pending,
      }
    }
  }

  impl AsyncSleep for TokioSleep {
    fn reset(self: std::pin::Pin<&mut Self>, deadline: Instant) {
      self.project().inner.as_mut().reset(deadline.into())
    }
  }

  impl AsyncSleepExt for TokioSleep {
    fn sleep(after: Duration) -> Self where Self: Sized {
      Self {
        inner: tokio::time::sleep(after),
      }
    }

    fn sleep_until(&self, deadline: Instant) -> Self where Self: Sized {
      Self {
        inner: tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)),
      }
    }
  }

  #[test]
  fn test_object_safe() {
    let _a: Box<dyn AsyncSleep> = Box::new(TokioSleep::sleep(Duration::from_secs(1)));
  }
}

#[cfg(all(feature = "async-io", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async-io"))))]
pub use _async_io::AsyncIOSleep;

#[cfg(all(feature = "async-io", feature = "std"))]
mod _async_io {
  use super::*;
  use async_io::Timer;

  pin_project_lite::pin_project! {
    /// The [`AsyncSleep`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
    #[derive(Debug)]
    #[repr(transparent)]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async-io"))))]
    pub struct AsyncIOSleep {
      #[pin]
      t: Timer,
    }
  }

  impl From<Timer> for AsyncIOSleep {
    fn from(t: Timer) -> Self {
      Self { t }
    }
  }

  impl From<AsyncIOSleep> for Timer {
    fn from(s: AsyncIOSleep) -> Self {
      s.t
    }
  }

  impl Future for AsyncIOSleep {
    type Output = Instant;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      self.project().t.poll(cx)
    }
  }

  impl AsyncSleepExt for AsyncIOSleep {
    fn sleep(after: Duration) -> Self where Self: Sized {
      Self {
        t: async_io::Timer::after(after),
      }
    }

    fn sleep_until(&self, deadline: Instant) -> Self where Self: Sized {
      Self {
        t: async_io::Timer::at(deadline),
      }
    }
  }

  impl AsyncSleep for AsyncIOSleep {
    /// Sets the timer to emit an event once at the given time instant.
    ///
    /// Note that resetting a timer is different from creating a new sleep by [`sleep()`][`Runtime::sleep()`] because
    /// `reset()` does not remove the waker associated with the task.
    fn reset(self: Pin<&mut Self>, deadline: Instant) {
      self.project().t.as_mut().set_at(deadline)
    }
  }

  #[test]
  fn test_object_safe() {
    let _a: Box<dyn AsyncSleep> = Box::new(AsyncIOSleep::sleep(Duration::from_secs(1)));
  }
}
