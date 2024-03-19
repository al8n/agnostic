use std::{
  future::Future,
  time::{Duration, Instant},
};

/// The timeout abstraction for async runtime.
pub trait AsyncTimeout<F: Future>: Future<Output = Result<F::Output, Elapsed>> {
  /// Requires a `Future` to complete before the specified duration has elapsed.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout(timeout: Duration, fut: F) -> Self
  where
    Self: Sized;

  /// Requires a `Future` to complete before the specified instant in time.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized;
}

/// Elapsed error
#[derive(Debug, PartialEq, Eq)]
pub struct Elapsed(());

impl core::fmt::Display for Elapsed {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "deadline has elapsed")
  }
}

impl std::error::Error for Elapsed {}

impl From<Elapsed> for std::io::Error {
  fn from(_: Elapsed) -> Self {
    std::io::ErrorKind::TimedOut.into()
  }
}

#[cfg(feature = "tokio")]
impl From<::tokio::time::error::Elapsed> for Elapsed {
  fn from(_: ::tokio::time::error::Elapsed) -> Self {
    Elapsed(())
  }
}

#[cfg(all(feature = "tokio", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "tokio"))))]
pub use _tokio::*;

#[cfg(all(feature = "tokio", feature = "std"))]
mod _tokio {
  use super::*;
  use core::{
    pin::Pin,
    task::{Context, Poll},
  };
  use tokio::time::{timeout, timeout_at, Timeout};

  pin_project_lite::pin_project! {
    /// The [`AsyncTimeout`] implementation for tokio runtime
    #[repr(transparent)]
    pub struct TokioTimeout<F> {
      #[pin]
      inner: Timeout<F>,
    }
  }

  impl<F> From<Timeout<F>> for TokioTimeout<F> {
    fn from(timeout: Timeout<F>) -> Self {
      Self { inner: timeout }
    }
  }

  impl<F> From<TokioTimeout<F>> for Timeout<F> {
    fn from(timeout: TokioTimeout<F>) -> Self {
      timeout.inner
    }
  }

  impl<F: Future> Future for TokioTimeout<F> {
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      match self.project().inner.poll(cx) {
        Poll::Ready(Ok(rst)) => Poll::Ready(Ok(rst)),
        Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
        Poll::Pending => Poll::Pending,
      }
    }
  }

  impl<F: Future> AsyncTimeout<F> for TokioTimeout<F> {
    fn timeout(t: Duration, fut: F) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: timeout(t, fut),
      }
    }

    fn timeout_at(deadline: Instant, fut: F) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: timeout_at(tokio::time::Instant::from_std(deadline), fut),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    const BAD: Duration = Duration::from_secs(1);
    const GOOD: Duration = Duration::from_millis(10);
    const TIMEOUT: Duration = Duration::from_millis(200);
    const BOUND: Duration = Duration::from_secs(10);

    #[tokio::test(flavor = "multi_thread")]
    async fn test_timeout() {
      futures::executor::block_on(async {
        let fut = async {
          tokio::time::sleep(BAD).await;
          1
        };
        let start = Instant::now();
        let rst = TokioTimeout::timeout(TIMEOUT, fut).await;
        assert!(rst.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

        let fut = async {
          tokio::time::sleep(GOOD).await;
          1
        };

        let start = Instant::now();
        let rst = TokioTimeout::timeout(TIMEOUT, fut).await;
        assert!(rst.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed >= GOOD && elapsed <= GOOD + BOUND);
      });
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_timeout_at() {
      futures::executor::block_on(async {
        let fut = async {
          tokio::time::sleep(BAD).await;
          1
        };
        let start = Instant::now();
        let rst = TokioTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
        assert!(rst.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

        let fut = async {
          tokio::time::sleep(GOOD).await;
          1
        };

        let start = Instant::now();
        let rst = TokioTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
        assert!(rst.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed >= GOOD && elapsed <= GOOD + BOUND);
      });
    }
  }
}

#[cfg(all(feature = "async-io", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "async-io"))))]
pub use _async_io::*;

#[cfg(all(feature = "async-io", feature = "std"))]
mod _async_io {
  use super::*;
  use async_io::Timer;
  use core::{
    pin::Pin,
    task::{Context, Poll},
  };
  use futures_util::future::{select, Either, Select};

  pin_project_lite::pin_project! {
    /// The [`AsyncSleep`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
    #[repr(transparent)]
    pub struct AsyncIoTimeout<F> {
      #[pin]
      inner: Select<Pin<Box<F>>, Timer>,
    }
  }

  impl<F: Future> Future for AsyncIoTimeout<F> {
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      let this = self.project();
      match this.inner.poll(cx) {
        Poll::Ready(Either::Left((output, _))) => Poll::Ready(Ok(output)),
        Poll::Ready(Either::Right(_)) => Poll::Ready(Err(Elapsed(()))),
        Poll::Pending => Poll::Pending,
      }
    }
  }

  impl<F: Future> AsyncTimeout<F> for AsyncIoTimeout<F> {
    fn timeout(timeout: Duration, fut: F) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: select(Box::pin(fut), Timer::after(timeout)),
      }
    }

    fn timeout_at(deadline: Instant, fut: F) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: select(Box::pin(fut), Timer::at(deadline)),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    const BAD: Duration = Duration::from_secs(1);
    const GOOD: Duration = Duration::from_millis(10);
    const TIMEOUT: Duration = Duration::from_millis(200);
    const BOUND: Duration = Duration::from_secs(10);

    #[test]
    fn test_timeout() {
      futures::executor::block_on(async {
        let fut = async {
          Timer::after(BAD).await;
          1
        };
        let start = Instant::now();
        let rst = AsyncIoTimeout::timeout(TIMEOUT, fut).await;
        assert!(rst.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

        let fut = async {
          Timer::after(GOOD).await;
          1
        };

        let start = Instant::now();
        let rst = AsyncIoTimeout::timeout(TIMEOUT, fut).await;
        assert!(rst.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed >= GOOD && elapsed <= GOOD + BOUND);
      });
    }

    #[test]
    fn test_timeout_at() {
      futures::executor::block_on(async {
        let fut = async {
          Timer::after(BAD).await;
          1
        };
        let start = Instant::now();
        let rst = AsyncIoTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
        assert!(rst.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

        let fut = async {
          Timer::after(GOOD).await;
          1
        };

        let start = Instant::now();
        let rst = AsyncIoTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
        assert!(rst.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed >= GOOD && elapsed <= GOOD + BOUND);
      });
    }
  }
}

#[cfg(all(feature = "wasm", feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "std", feature = "wasm"))))]
pub use _wasm::*;

#[cfg(all(feature = "wasm", feature = "std"))]
mod _wasm {
  use super::*;
  use core::{
    pin::Pin,
    task::{Context, Poll},
  };
  use futures_timer::Delay;
  use futures_util::future::{select, Either, Select};

  pin_project_lite::pin_project! {
    /// The [`AsyncTimeout`] implementation for wasm bindgen
    pub struct WasmTimeout<F> {
      #[pin]
      inner: Select<Pin<Box<F>>, Delay>,
    }
  }

  impl<F: Future> Future for WasmTimeout<F> {
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      let this = self.project();
      match this.inner.poll(cx) {
        Poll::Ready(Either::Left((output, _))) => Poll::Ready(Ok(output)),
        Poll::Ready(Either::Right(_)) => Poll::Ready(Err(Elapsed(()))),
        Poll::Pending => Poll::Pending,
      }
    }
  }

  impl<F: Future> AsyncTimeout<F> for WasmTimeout<F> {
    fn timeout(timeout: Duration, fut: F) -> Self
    where
      Self: Sized,
    {
      Self {
        inner: select(Box::pin(fut), Delay::new(timeout)),
      }
    }

    fn timeout_at(deadline: Instant, fut: F) -> Self
    where
      Self: Sized,
    {
      let duration = deadline - Instant::now();
      Self {
        inner: select(Box::pin(fut), Delay::new(duration)),
      }
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    const BAD: Duration = Duration::from_secs(1);
    const GOOD: Duration = Duration::from_millis(10);
    const TIMEOUT: Duration = Duration::from_millis(200);
    const BOUND: Duration = Duration::from_secs(10);

    #[test]
    fn test_timeout() {
      futures::executor::block_on(async {
        let fut = async {
          futures_timer::Delay::new(BAD).await;
          1
        };
        let start = Instant::now();
        let rst = WasmTimeout::timeout(TIMEOUT, fut).await;
        assert!(rst.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

        let fut = async {
          futures_timer::Delay::new(GOOD).await;
          1
        };

        let start = Instant::now();
        let rst = WasmTimeout::timeout(TIMEOUT, fut).await;
        assert!(rst.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed >= GOOD && elapsed <= GOOD + BOUND);
      });
    }

    #[test]
    fn test_timeout_at() {
      futures::executor::block_on(async {
        let fut = async {
          futures_timer::Delay::new(BAD).await;
          1
        };
        let start = Instant::now();
        let rst = WasmTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
        assert!(rst.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

        let fut = async {
          futures_timer::Delay::new(GOOD).await;
          1
        };

        let start = Instant::now();
        let rst = WasmTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
        assert!(rst.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed >= GOOD && elapsed <= GOOD + BOUND);
      });
    }
  }
}
