use crate::time::{AsyncLocalTimeout, AsyncTimeout, Elapsed};
use ::async_io::Timer;
use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::time::Instant;

pin_project_lite::pin_project! {
  /// The [`AsyncTimeout`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
  pub struct AsyncIoTimeout<F> {
    #[pin]
    future: F,
    #[pin]
    delay: Timer,
  }
}

impl<F: Future> Future for AsyncIoTimeout<F> {
  type Output = Result<F::Output, Elapsed>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    match this.future.poll(cx) {
      Poll::Pending => {}
      other => return other.map(Ok),
    }

    if this.delay.poll(cx).is_ready() {
      Poll::Ready(Err(Elapsed))
    } else {
      Poll::Pending
    }
  }
}

impl<F: Future + Send> AsyncTimeout<F> for AsyncIoTimeout<F> {
  type Instant = Instant;

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

impl<F> AsyncLocalTimeout<F> for AsyncIoTimeout<F>
where
  F: Future,
{
  type Instant = Instant;

  fn timeout_local(timeout: Duration, fut: F) -> Self
  where
    Self: Sized,
  {
    Self {
      future: fut,
      delay: Timer::after(timeout),
    }
  }

  fn timeout_local_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized,
  {
    Self {
      future: fut,
      delay: Timer::at(deadline),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{AsyncIoTimeout, AsyncTimeout, Timer};
  use std::time::{Duration, Instant};

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
