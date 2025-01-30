use core::{future::Future, pin::Pin, task::{Context, Poll}, time::Duration};

use crate::time::{AsyncLocalSleep, AsyncLocalSleepExt};

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
  type Output = ::tokio::time::Instant;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    let ddl = this.inner.deadline();
    match this.inner.poll(cx) {
      Poll::Ready(_) => Poll::Ready(ddl),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl AsyncLocalSleep for TokioSleep {
  type Instant = ::tokio::time::Instant;

  fn reset(self: Pin<&mut Self>, deadline: Self::Instant) {
    self.project().inner.as_mut().reset(deadline)
  }
}

impl AsyncLocalSleepExt for TokioSleep {
  fn sleep_local(after: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: tokio::time::sleep(after),
    }
  }

  fn sleep_local_until(deadline: Self::Instant) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: tokio::time::sleep_until(deadline),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::TokioSleep;
  use crate::time::{AsyncSleep, AsyncSleepExt};
  use tokio::time::{Duration, Instant};

  const ORIGINAL: Duration = Duration::from_secs(1);
  const RESET: Duration = Duration::from_secs(2);
  const BOUND: Duration = Duration::from_millis(10);

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
