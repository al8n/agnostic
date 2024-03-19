use std::{future::Future, pin::Pin, task::{Context, Poll}, time::{Duration, Instant}};

/// The timeout abstraction for async runtime.
pub trait AsyncTimeout<F: Future>:
  Future<Output = Result<F::Output, Elapsed>>
{
  /// Requires a `Future` to complete before the specified duration has elapsed.
  /// 
  /// The behavior of this function may different in different runtime implementations.
  fn timeout(timeout: Duration, fut: F) -> Self where Self: Sized;

  /// Requires a `Future` to complete before the specified instant in time.
  /// 
  /// The behavior of this function may different in different runtime implementations.
  fn timeout_at(deadline: Instant, fut: F) -> Self where Self: Sized;
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
  use tokio::time::{Timeout, timeout, timeout_at};


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

    fn poll(
      self: Pin<&mut Self>,
      cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
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
        Self: Sized {
      Self {
        inner: timeout(t, fut),
      }
    }

    fn timeout_at(deadline: Instant, fut: F) -> Self
      where
        Self: Sized {
      Self {
        inner: timeout_at(tokio::time::Instant::from_std(deadline), fut),
      }
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
  use futures_util::future::{Either, Select, select};

  
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

    fn poll(
      self: Pin<&mut Self>,
      cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
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
        Self: Sized {
      Self {
        inner: select(Box::pin(fut), Timer::after(timeout)),
      }
    }

    fn timeout_at(deadline: Instant, fut: F) -> Self
      where
        Self: Sized {
      Self {
        inner: select(Box::pin(fut), Timer::at(deadline)),
      }
    }
  }
}