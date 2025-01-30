use ::async_io::Timer;
use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

use std::time::Instant;

use crate::time::{AsyncLocalSleep, AsyncLocalSleepExt};

pin_project_lite::pin_project! {
  /// The [`AsyncSleep`] implementation for any runtime based on [`async-io`](async_io), e.g. `async-std` and `smol`.
  #[derive(Debug)]
  #[repr(transparent)]
  #[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
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

impl AsyncLocalSleepExt for AsyncIoSleep {
  fn sleep_local(after: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      t: Timer::after(after),
    }
  }

  fn sleep_local_until(deadline: Instant) -> Self
  where
    Self: Sized,
  {
    Self {
      t: Timer::at(deadline),
    }
  }
}

impl AsyncLocalSleep for AsyncIoSleep {
  type Instant = Instant;

  /// Sets the timer to emit an event once at the given time instant.
  ///
  /// Note that resetting a timer is different from creating a new sleep by [`sleep()`][`Runtime::sleep()`] because
  /// `reset()` does not remove the waker associated with the task.
  fn reset(self: Pin<&mut Self>, deadline: Instant) {
    self.project().t.as_mut().set_at(deadline)
  }
}

#[cfg(test)]
mod tests {
  use super::AsyncIoSleep;
  use crate::time::{AsyncSleep, AsyncSleepExt};
  use core::pin::Pin;
  use std::time::{Duration, Instant};

  const ORIGINAL: Duration = Duration::from_secs(1);
  const RESET: Duration = Duration::from_secs(2);
  const BOUND: Duration = Duration::from_millis(10);

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
