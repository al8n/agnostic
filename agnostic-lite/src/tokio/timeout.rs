use crate::time::{AsyncLocalTimeout, AsyncTimeout, Elapsed};

use ::tokio::time::{timeout, timeout_at, Timeout};
use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::time::Instant;

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

impl<F: Future + Send> AsyncTimeout<F> for TokioTimeout<F> {
  fn timeout(t: Duration, fut: F) -> Self
  where
    Self: Sized,
  {
    <Self as AsyncLocalTimeout<F>>::timeout_local(t, fut)
  }

  fn timeout_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized,
  {
    <Self as AsyncLocalTimeout<F>>::timeout_local_at(deadline, fut)
  }
}

impl<F> AsyncLocalTimeout<F> for TokioTimeout<F>
where
  F: Future,
{
  fn timeout_local(t: Duration, fut: F) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: timeout(t, fut),
    }
  }

  fn timeout_local_at(deadline: Instant, fut: F) -> Self
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
  use super::{AsyncTimeout, TokioTimeout};
  use std::time::{Duration, Instant};

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
