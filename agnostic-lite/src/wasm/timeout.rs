use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::time::Instant;
use wasm::Delay;

use crate::time::{AsyncLocalTimeout, AsyncTimeout, Elapsed};

pin_project_lite::pin_project! {
  /// The [`AsyncTimeout`] implementation for wasm bindgen
  pub struct WasmTimeout<F> {
    #[pin]
    future: F,
    #[pin]
    delay: Delay,
  }
}

impl<F: Future> Future for WasmTimeout<F> {
  type Output = Result<F::Output, Elapsed>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    match this.future.poll(cx) {
      Poll::Ready(v) => Poll::Ready(Ok(v)),
      Poll::Pending => match this.delay.poll(cx) {
        Poll::Ready(_) => Poll::Ready(Err(Elapsed)),
        Poll::Pending => Poll::Pending,
      },
    }
  }
}

impl<F: Future + Send> AsyncTimeout<F> for WasmTimeout<F> {
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

impl<F> AsyncLocalTimeout<F> for WasmTimeout<F>
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
      delay: Delay::new(timeout),
    }
  }

  fn timeout_local_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized,
  {
    let duration = deadline - Instant::now();
    Self {
      future: fut,
      delay: Delay::new(duration),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{AsyncTimeout, WasmTimeout};
  use std::time::{Duration, Instant};

  const BAD: Duration = Duration::from_secs(1);
  const GOOD: Duration = Duration::from_millis(10);
  const TIMEOUT: Duration = Duration::from_millis(200);
  const BOUND: Duration = Duration::from_secs(10);

  #[test]
  fn test_timeout() {
    futures::executor::block_on(async {
      let fut = async {
        wasm::Delay::new(BAD).await;
        1
      };
      let start = Instant::now();
      let rst = WasmTimeout::timeout(TIMEOUT, fut).await;
      assert!(rst.is_err());
      let elapsed = start.elapsed();
      assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

      let fut = async {
        wasm::Delay::new(GOOD).await;
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
        wasm::Delay::new(BAD).await;
        1
      };
      let start = Instant::now();
      let rst = WasmTimeout::timeout_at(Instant::now() + TIMEOUT, fut).await;
      assert!(rst.is_err());
      let elapsed = start.elapsed();
      assert!(elapsed >= TIMEOUT && elapsed <= TIMEOUT + BOUND);

      let fut = async {
        wasm::Delay::new(GOOD).await;
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
