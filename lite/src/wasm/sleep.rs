use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use futures_timer::Delay;
use std::time::Instant;

use crate::time::{AsyncLocalSleep, AsyncLocalSleepExt};

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

impl AsyncLocalSleep for WasmSleep {
  fn reset(self: Pin<&mut Self>, deadline: Instant) {
    let mut this = self.project();
    let ddl = deadline - Instant::now();
    this.sleep.reset(ddl);
    *this.ddl = deadline;
  }
}

impl AsyncLocalSleepExt for WasmSleep {
  fn sleep_local(after: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      ddl: Instant::now() + after,
      sleep: Delay::new(after),
      duration: after,
    }
  }

  fn sleep_local_until(deadline: Instant) -> Self
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
  use super::WasmSleep;
  use crate::time::{AsyncSleep, AsyncSleepExt};
  use core::pin::Pin;
  use std::time::{Duration, Instant};

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
