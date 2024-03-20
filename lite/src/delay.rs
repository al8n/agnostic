use core::{
  future::Future,
  pin::Pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{Context, Poll},
};
use std::time::{Duration, Instant};

/// Delay is aborted
#[derive(Debug, Clone, Copy)]
pub struct Aborted(());

impl core::fmt::Display for Aborted {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "delay aborted")
  }
}

impl std::error::Error for Aborted {}

fn _assert1(_: Box<dyn AsyncLocalDelay<impl Future>>) {}
fn _assert2(_: Box<dyn AsyncDelay<impl Future>>) {}

/// Simlilar to Go's `time.AfterFunc`
pub trait AsyncDelay<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future + Send,
{
  /// Abort the delay, if future has not yet completed, then it will never be polled again.
  fn abort(&self);

  /// Cancel the delay, running the future immediately
  fn cancel(&self);

  /// Reset the delay to a new duration
  fn reset(self: Pin<&mut Self>, dur: Duration);

  /// Resets the delay to a new instant
  fn reset_at(self: Pin<&mut Self>, at: Instant);
}

/// Extension trait for [`AsyncLocalDelay`]
pub trait AsyncDelayExt<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future + Send,
{
  /// Create a new delay, the future will be polled after the duration has elapsed
  fn delay(dur: Duration, fut: F) -> Self;

  /// Create a new delay, the future will be polled after the instant has elapsed
  fn delay_at(at: Instant, fut: F) -> Self;
}

impl<F: Future + Send, T> AsyncDelay<F> for T
where
  T: AsyncLocalDelay<F>,
{
  fn abort(&self) {
    AsyncLocalDelay::abort(self);
  }

  fn cancel(&self) {
    AsyncLocalDelay::cancel(self);
  }

  fn reset(self: Pin<&mut Self>, dur: Duration) {
    AsyncLocalDelay::reset(self, dur);
  }

  fn reset_at(self: Pin<&mut Self>, at: Instant) {
    AsyncLocalDelay::reset_at(self, at);
  }
}

impl<F: Future + Send, T> AsyncDelayExt<F> for T
where
  T: AsyncLocalDelayExt<F>,
{
  fn delay(dur: Duration, fut: F) -> Self {
    AsyncLocalDelayExt::delay(dur, fut)
  }

  fn delay_at(at: Instant, fut: F) -> Self {
    AsyncLocalDelayExt::delay_at(at, fut)
  }
}

/// Like [`Delay`] but does not require `Send`
pub trait AsyncLocalDelay<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future,
{
  /// Abort the delay, if future has not yet completed, then it will never be polled again.
  fn abort(&self);

  /// Cancel the delay, running the future immediately
  fn cancel(&self);

  /// Reset the delay to a new duration
  fn reset(self: Pin<&mut Self>, dur: Duration);

  /// Resets the delay to a new instant
  fn reset_at(self: Pin<&mut Self>, at: Instant);
}

/// Extension trait for [`AsyncLocalDelay`]
pub trait AsyncLocalDelayExt<F>: Future<Output = Result<F::Output, Aborted>>
where
  F: Future,
{
  /// Create a new delay, the future will be polled after the duration has elapsed
  fn delay(dur: Duration, fut: F) -> Self;

  /// Create a new delay, the future will be polled after the instant has elapsed
  fn delay_at(at: Instant, fut: F) -> Self;
}

pin_project_lite::pin_project! {
  /// [`AsyncDelay`] implementation for wasm bindgen runtime
  pub struct Delay<F, S> {
    #[pin]
    fut: Option<F>,
    #[pin]
    sleep: S,
    aborted: AtomicBool,
    canceled: AtomicBool,
  }
}

impl<F: Future, S: Future> Future for Delay<F, S> {
  type Output = Result<F::Output, Aborted>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if self.aborted.load(Ordering::Acquire) {
      return Poll::Ready(Err(Aborted(())));
    }

    let this = self.project();
    if !this.canceled.load(Ordering::Acquire) && !this.sleep.poll(cx).is_ready() {
      return Poll::Pending;
    }

    if let Some(fut) = this.fut.as_pin_mut() {
      return fut.poll(cx).map(Ok);
    }

    Poll::Pending
  }
}

impl<F, S> AsyncLocalDelay<F> for Delay<F, S>
where
  F: Future,
  S: AsyncLocalSleep,
{
  fn abort(&self) {
    self.aborted.store(true, Ordering::Release)
  }

  fn cancel(&self) {
    self.canceled.store(true, Ordering::Release)
  }

  fn reset(self: Pin<&mut Self>, dur: Duration) {
    self.project().sleep.as_mut().reset(Instant::now() + dur);
  }

  fn reset_at(self: Pin<&mut Self>, at: Instant) {
    self.project().sleep.as_mut().reset(at);
  }
}

impl<F, S> AsyncLocalDelayExt<F> for Delay<F, S>
where
  F: Future,
  S: AsyncLocalSleepExt,
{
  fn delay(dur: Duration, fut: F) -> Self {
    Self {
      fut: Some(fut),
      sleep: S::sleep_local(dur),
      aborted: AtomicBool::new(false),
      canceled: AtomicBool::new(false),
    }
  }

  fn delay_at(at: Instant, fut: F) -> Self {
    Self {
      fut: Some(fut),
      sleep: S::sleep_local_until(at),
      aborted: AtomicBool::new(false),
      canceled: AtomicBool::new(false),
    }
  }
}

#[cfg(all(feature = "time", feature = "tokio"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "time", feature = "tokio"))))]
pub use _tokio::*;

#[cfg(all(feature = "time", feature = "tokio"))]
mod _tokio {
  /// Alias for [`Delay`] using [`tokio`] runtime.
  pub type TokioDelay<F> = super::Delay<F, crate::TokioSleep>;

  #[cfg(test)]
  mod tests {
    use crate::{AsyncDelay, AsyncDelayExt};

    use super::TokioDelay;
    use std::{
      sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
      },
      time::{Duration, Instant},
    };

    const DELAY: Duration = Duration::from_millis(1);
    const RESET: Duration = Duration::from_millis(2);
    const BOUND: Duration = Duration::from_millis(50);

    #[test]
    fn test_delay() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= DELAY);
        assert!(elapsed < DELAY + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_at() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay_at(start + DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= DELAY);
        assert!(elapsed < DELAY + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_reset() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        futures_util::pin_mut!(delay);
        AsyncDelay::reset(delay.as_mut(), RESET);
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET);
        assert!(elapsed < RESET + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_abort() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        AsyncDelay::abort(&delay);
        assert!(delay.await.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed < DELAY);
        assert_eq!(ctr.load(Ordering::SeqCst), 0);
      });
    }

    #[test]
    fn test_delay_cancel() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        AsyncDelay::cancel(&delay);
        assert!(delay.await.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed < BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }
  }
}

#[cfg(feature = "async-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
pub use _async_io::*;

#[cfg(feature = "async-io")]
mod _async_io {
  use super::*;
  use crate::sleep::AsyncIoSleep;

  /// Alias for [`Delay`] using runtime based on [`async-io`](async_io), e.g. `async-std`, `smol`
  pub type AsyncIoDelay<F> = Delay<F, AsyncIoSleep>;

  #[cfg(test)]
  mod tests {
    use super::{AsyncDelay, AsyncDelayExt, AsyncIoDelay};
    use std::{
      sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
      },
      time::{Duration, Instant},
    };

    const DELAY: Duration = Duration::from_millis(1);
    const RESET: Duration = Duration::from_millis(2);
    const BOUND: Duration = Duration::from_millis(50);

    #[test]
    fn test_delay() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = AsyncIoDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= DELAY);
        assert!(elapsed < DELAY + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_at() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = AsyncIoDelay::delay_at(start + DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= DELAY);
        assert!(elapsed < DELAY + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_reset() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = AsyncIoDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        futures_util::pin_mut!(delay);
        AsyncDelay::reset(delay.as_mut(), RESET);
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET);
        assert!(elapsed < RESET + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_abort() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = AsyncIoDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        AsyncDelay::abort(&delay);
        assert!(delay.await.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed < DELAY);
        assert_eq!(ctr.load(Ordering::SeqCst), 0);
      });
    }

    #[test]
    fn test_delay_cancel() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = AsyncIoDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        AsyncDelay::cancel(&delay);
        assert!(delay.await.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed < BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }
  }
}

#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub use _wasm::*;

use crate::{AsyncLocalSleep, AsyncLocalSleepExt};

#[cfg(feature = "wasm")]
mod _wasm {
  use super::*;
  use crate::sleep::WasmSleep;

  /// Alias for [`Delay`] using wasm bindgen runtime.
  pub type WasmDelay<F> = Delay<F, WasmSleep>;

  #[cfg(test)]
  mod tests {
    use super::{AsyncDelay, AsyncDelayExt, WasmDelay};
    use std::{
      sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
      },
      time::{Duration, Instant},
    };

    const DELAY: Duration = Duration::from_millis(1);
    const RESET: Duration = Duration::from_millis(2);
    const BOUND: Duration = Duration::from_millis(50);

    #[test]
    fn test_delay() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = WasmDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= DELAY);
        assert!(elapsed < DELAY + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_at() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = WasmDelay::delay_at(start + DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= DELAY);
        assert!(elapsed < DELAY + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_reset() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = WasmDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        futures_util::pin_mut!(delay);
        AsyncDelay::reset(delay.as_mut(), RESET);
        delay.await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= RESET);
        assert!(elapsed < RESET + BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }

    #[test]
    fn test_delay_abort() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = WasmDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        AsyncDelay::abort(&delay);
        assert!(delay.await.is_err());
        let elapsed = start.elapsed();
        assert!(elapsed < DELAY);
        assert_eq!(ctr.load(Ordering::SeqCst), 0);
      });
    }

    #[test]
    fn test_delay_cancel() {
      futures::executor::block_on(async {
        let start = Instant::now();
        let ctr = Arc::new(AtomicUsize::new(0));
        let ctr1 = ctr.clone();
        let delay = WasmDelay::delay(DELAY, async move {
          ctr1.fetch_add(1, Ordering::SeqCst);
        });
        AsyncDelay::cancel(&delay);
        assert!(delay.await.is_ok());
        let elapsed = start.elapsed();
        assert!(elapsed < BOUND);
        assert_eq!(ctr.load(Ordering::SeqCst), 1);
      });
    }
  }
}
