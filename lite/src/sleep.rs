use std::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::{Duration, Instant},
};

/// The sleep abstraction for a runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub trait AsyncSleep: Future<Output = Instant> {
  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: Pin<&mut Self>, deadline: Instant);
}

/// Extension trait for [`AsyncSleep`].
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
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

#[cfg(all(feature = "tokio", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "tokio"))))]
pub use _tokio::TokioSleep;

#[cfg(all(feature = "tokio", feature = "std"))]
mod _tokio {
  use super::*;

  pin_project_lite::pin_project! {
    /// The [`AsyncSleep`] implementation for tokio runtime
    #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "tokio"))))]
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

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
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
    fn sleep(after: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: tokio::time::sleep(after),
      }
    }

    fn sleep_until(deadline: Instant) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    const ORIGINAL: Duration = Duration::from_secs(1);
    const RESET: Duration = Duration::from_secs(2);
    const BOUND: Duration = Duration::from_millis(10);

    #[tokio::test]
    async fn test_object_safe() {
      let _a: Box<dyn AsyncSleep> = Box::new(TokioSleep::sleep(ORIGINAL));
    }

    #[tokio::test]
    async fn test_tokio_sleep() {
      let start = Instant::now();
      let sleep = TokioSleep::sleep(ORIGINAL);
      let ins = sleep.await;
      assert!(ins >= start + ORIGINAL);
      let elapsed = start.elapsed();
      assert!(elapsed >= ORIGINAL && elapsed < ORIGINAL + BOUND);
    }

    #[tokio::test]
    async fn test_tokio_sleep_until() {
      let start = Instant::now();
      let sleep = TokioSleep::sleep_until(start + ORIGINAL);
      let ins = sleep.await;
      assert!(ins >= start + ORIGINAL);
      let elapsed = start.elapsed();
      assert!(elapsed >= ORIGINAL && elapsed < ORIGINAL + BOUND);
    }

    #[tokio::test]
    async fn test_tokio_sleep_reset() {
      let start = Instant::now();
      let sleep = TokioSleep::sleep(ORIGINAL);
      tokio::pin!(sleep);
      sleep.as_mut().reset(Instant::now() + RESET);
      let ins = sleep.await;
      assert!(ins >= start + RESET);
      let elapsed = start.elapsed();
      assert!(elapsed >= RESET && elapsed < RESET + BOUND);
    }

    #[tokio::test]
    async fn test_tokio_sleep_reset2() {
      let start = Instant::now();
      let sleep = TokioSleep::sleep_until(start + ORIGINAL);
      tokio::pin!(sleep);
      sleep.as_mut().reset(Instant::now() + RESET);
      let ins = sleep.await;
      assert!(ins >= start + RESET);
      let elapsed = start.elapsed();
      assert!(elapsed >= RESET && elapsed < RESET + BOUND);
    }
  }
}

#[cfg(all(feature = "async-io", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async-io"))))]
pub use _async_io::AsyncIoSleep;

#[cfg(all(feature = "async-io", feature = "std"))]
mod _async_io {
  use super::*;
  use async_io::Timer;

  pin_project_lite::pin_project! {
    /// The [`AsyncSleep`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
    #[derive(Debug)]
    #[repr(transparent)]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async-io"))))]
    pub struct AsyncIoSleep {
      #[pin]
      t: Timer,
    }
  }

  impl From<Timer> for AsyncIoSleep {
    fn from(t: Timer) -> Self {
      Self { t }
    }
  }

  impl From<AsyncIoSleep> for Timer {
    fn from(s: AsyncIoSleep) -> Self {
      s.t
    }
  }

  impl Future for AsyncIoSleep {
    type Output = Instant;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      self.project().t.poll(cx)
    }
  }

  impl AsyncSleepExt for AsyncIoSleep {
    fn sleep(after: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        t: async_io::Timer::after(after),
      }
    }

    fn sleep_until(deadline: Instant) -> Self
    where
      Self: Sized,
    {
      Self {
        t: async_io::Timer::at(deadline),
      }
    }
  }

  impl AsyncSleep for AsyncIoSleep {
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
    let _a: Box<dyn AsyncSleep> = Box::new(AsyncIoSleep::sleep(Duration::from_secs(1)));
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    const ORIGINAL: Duration = Duration::from_secs(1);
    const RESET: Duration = Duration::from_secs(2);
    const BOUND: Duration = Duration::from_millis(10);

    #[test]
    fn test_object_safe() {
      let _a: Box<dyn AsyncSleep> = Box::new(AsyncIoSleep::sleep(ORIGINAL));
    }

    #[test]
    fn test_asyncio_sleep() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let sleep = AsyncIoSleep::sleep(ORIGINAL);
        let ins = sleep.await;
        assert!(ins >= start + ORIGINAL);
        let elapsed = start.elapsed();
        assert!(elapsed >= ORIGINAL && elapsed < ORIGINAL + BOUND);
      });
    }

    #[test]
    fn test_asyncio_sleep_until() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let sleep = AsyncIoSleep::sleep_until(start + ORIGINAL);
        let ins = sleep.await;
        assert!(ins >= start + ORIGINAL);
        let elapsed = start.elapsed();
        assert!(elapsed >= ORIGINAL && elapsed < ORIGINAL + BOUND);
      });
    }

    #[test]
    fn test_asyncio_sleep_reset() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let mut sleep = AsyncIoSleep::sleep(ORIGINAL);
        let pin = Pin::new(&mut sleep);
        pin.reset(Instant::now() + RESET);
        let ins = sleep.await;
        assert!(ins >= start + RESET);
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET && elapsed < RESET + BOUND);
      });
    }

    #[test]
    fn test_asyncio_sleep_reset2() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let mut sleep = AsyncIoSleep::sleep_until(start + ORIGINAL);
        let pin = Pin::new(&mut sleep);
        pin.reset(Instant::now() + RESET);
        let ins = sleep.await;
        assert!(ins >= start + RESET);
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET && elapsed < RESET + BOUND);
      });
    }
  }
}

#[cfg(all(feature = "wasm", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "wasm"))))]
pub use _wasm::WasmSleep;

#[cfg(all(feature = "wasm", feature = "std"))]
mod _wasm {
  use super::*;
  use futures_timer::Delay;

  pin_project_lite::pin_project! {
    /// The [`AsyncSleep`] implementation for wasm-bindgen based runtime.
    #[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "wasm"))))]
    pub struct WasmSleep {
      #[pin]
      pub(crate) sleep: Delay,
      pub(crate) ddl: Instant,
      pub(crate) duration: Duration,
    }
  }

  impl Future for WasmSleep {
    type Output = Instant;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      let ddl = self.ddl;
      self.project().sleep.poll(cx).map(|_| ddl)
    }
  }

  impl AsyncSleep for WasmSleep {
    fn reset(self: Pin<&mut Self>, deadline: Instant) {
      let mut this = self.project();
      let ddl = deadline - Instant::now();
      this.sleep.reset(ddl);
      *this.ddl = deadline;
    }
  }

  impl AsyncSleepExt for WasmSleep {
    fn sleep(after: Duration) -> Self
    where
      Self: Sized,
    {
      Self {
        ddl: Instant::now() + after,
        sleep: Delay::new(after),
        duration: after,
      }
    }

    fn sleep_until(deadline: Instant) -> Self
    where
      Self: Sized,
    {
      let duration = deadline - Instant::now();
      Self {
        sleep: Delay::new(duration),
        ddl: deadline,
        duration,
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    const ORIGINAL: Duration = Duration::from_secs(1);
    const RESET: Duration = Duration::from_secs(2);
    const BOUND: Duration = Duration::from_millis(10);

    #[test]
    fn test_object_safe() {
      let _a: Box<dyn AsyncSleep> = Box::new(WasmSleep::sleep(ORIGINAL));
    }

    #[test]
    fn test_wasm_sleep() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let sleep = WasmSleep::sleep(ORIGINAL);
        let ins = sleep.await;
        assert!(ins >= start + ORIGINAL);
        let elapsed = start.elapsed();
        assert!(elapsed >= ORIGINAL && elapsed < ORIGINAL + BOUND);
      });
    }

    #[test]
    fn test_wasm_sleep_until() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let sleep = WasmSleep::sleep_until(start + ORIGINAL);
        let ins = sleep.await;
        assert!(ins >= start + ORIGINAL);
        let elapsed = start.elapsed();
        assert!(elapsed >= ORIGINAL && elapsed < ORIGINAL + BOUND);
      });
    }

    #[test]
    fn test_wasm_sleep_reset() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let mut sleep = WasmSleep::sleep(ORIGINAL);
        let pin = Pin::new(&mut sleep);
        pin.reset(Instant::now() + RESET);
        let ins = sleep.await;
        assert!(ins >= start + RESET);
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET && elapsed < RESET + BOUND);
      });
    }

    #[test]
    fn test_wasm_sleep_reset2() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let mut sleep = WasmSleep::sleep_until(start + ORIGINAL);
        let pin = Pin::new(&mut sleep);
        pin.reset(Instant::now() + RESET);
        let ins = sleep.await;
        assert!(ins >= start + RESET);
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET && elapsed < RESET + BOUND);
      });
    }
  }
}
