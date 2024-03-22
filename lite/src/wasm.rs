#[cfg(feature = "time")]
mod timeout;
#[cfg(feature = "time")]
pub use timeout::*;

#[cfg(feature = "time")]
mod after;
#[cfg(feature = "time")]
pub use after::*;

#[cfg(feature = "time")]
mod sleep;
#[cfg(feature = "time")]
pub use sleep::*;

#[cfg(feature = "time")]
mod interval;
#[cfg(feature = "time")]
pub use interval::*;

#[cfg(feature = "time")]
mod delay;
#[cfg(feature = "time")]
pub use delay::*;

use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

#[cfg(feature = "time")]
use std::time::{Duration, Instant};

use wasm::channel::*;

use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner};

impl<T> super::Detach for WasmJoinHandle<T> {}

/// The join handle returned by [`WasmSpawner`].
pub struct WasmJoinHandle<F> {
  pub(crate) stop_tx: oneshot::Sender<bool>,
  pub(crate) rx: oneshot::Receiver<F>,
}

impl<F> Future for WasmJoinHandle<F> {
  type Output = Result<F, oneshot::Canceled>;

  fn poll(
    mut self: core::pin::Pin<&mut Self>,
    cx: &mut core::task::Context<'_>,
  ) -> core::task::Poll<Self::Output> {
    core::pin::Pin::new(&mut self.rx).poll(cx)
  }
}

impl<F> WasmJoinHandle<F> {
  /// Detach the future from the spawner.
  #[inline]
  pub fn detach(self) {
    let _ = self.stop_tx.send(false);
  }

  /// Cancel the future.
  #[inline]
  pub fn cancel(self) {
    let _ = self.stop_tx.send(true);
  }
}

/// A [`AsyncSpawner`] that uses the [`wasm-bindgen-futures`](wasm_bindgen_futures) runtime.
#[derive(Debug, Clone, Copy)]
pub struct WasmSpawner;

impl AsyncSpawner for WasmSpawner {
  type JoinHandle<F> = WasmJoinHandle<F> where F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    <Self as super::AsyncLocalSpawner>::spawn_local(future)
  }
}

impl AsyncLocalSpawner for WasmSpawner {
  type JoinHandle<F> = WasmJoinHandle<F> where F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    use futures_util::FutureExt;

    let (tx, rx) = oneshot::channel();
    let (stop_tx, stop_rx) = oneshot::channel();
    wasm::spawn_local(async {
      futures_util::pin_mut!(future);

      futures_util::select! {
        sig = stop_rx.fuse() => {
          match sig {
            Ok(true) => {
              // if we receive a stop signal, we just stop this task.
            },
            Ok(false) | Err(_) => {
              let _ = future.await;
            },
          }
        },
        future = (&mut future).fuse() => {
          let _ = tx.send(future);
        }
      }
    });
    WasmJoinHandle { stop_tx, rx }
  }
}

impl AsyncBlockingSpawner for WasmSpawner {
  type JoinHandle<R> = std::thread::JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    std::thread::spawn(f)
  }
}

/// Future for the [`yield_now`](RuntimeLite::yield_now) function.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct YieldNow(bool);

impl Future for YieldNow {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if !self.0 {
      self.0 = true;
      cx.waker().wake_by_ref();
      Poll::Pending
    } else {
      Poll::Ready(())
    }
  }
}

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`tokio`](::tokio) runtime.
#[derive(Debug, Clone, Copy)]
pub struct WasmRuntime;

impl core::fmt::Display for WasmRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "wasm-bindgen-futures")
  }
}

impl super::RuntimeLite for WasmRuntime {
  type Spawner = WasmSpawner;
  type LocalSpawner = WasmSpawner;
  type BlockingSpawner = WasmSpawner;

  #[cfg(feature = "time")]
  type AfterSpawner = WasmSpawner;
  #[cfg(feature = "time")]
  type LocalAfterSpawner = WasmSpawner;

  #[cfg(feature = "time")]
  type Interval = WasmInterval;
  #[cfg(feature = "time")]
  type LocalInterval = WasmInterval;
  #[cfg(feature = "time")]
  type Sleep = WasmSleep;
  #[cfg(feature = "time")]
  type LocalSleep = WasmSleep;
  #[cfg(feature = "time")]
  type Delay<F> = WasmDelay<F> where F: Future + Send;
  #[cfg(feature = "time")]
  type LocalDelay<F> = WasmDelay<F> where F: Future;
  #[cfg(feature = "time")]
  type Timeout<F> = WasmTimeout<F> where F: Future + Send;
  #[cfg(feature = "time")]
  type LocalTimeout<F> = WasmTimeout<F> where F: Future;

  fn new() -> Self {
    Self
  }

  fn block_on<F: Future>(_f: F) -> F::Output {
    panic!("RuntimeLite::block_on is not supported on wasm")
  }

  #[cfg(feature = "time")]
  fn interval(interval: Duration) -> Self::Interval {
    use crate::time::AsyncIntervalExt;

    WasmInterval::interval(interval)
  }

  #[cfg(feature = "time")]
  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    use crate::time::AsyncIntervalExt;

    WasmInterval::interval_at(start, period)
  }

  #[cfg(feature = "time")]
  fn interval_local(interval: Duration) -> Self::LocalInterval {
    use crate::time::AsyncIntervalExt;

    WasmInterval::interval(interval)
  }

  #[cfg(feature = "time")]
  fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
    use crate::time::AsyncIntervalExt;

    WasmInterval::interval_at(start, period)
  }

  #[cfg(feature = "time")]
  fn sleep(duration: Duration) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    WasmSleep::sleep(duration)
  }

  #[cfg(feature = "time")]
  fn sleep_until(instant: Instant) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    WasmSleep::sleep_until(instant)
  }

  #[cfg(feature = "time")]
  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    WasmSleep::sleep(duration)
  }

  #[cfg(feature = "time")]
  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    WasmSleep::sleep_until(instant)
  }

  async fn yield_now() {
    YieldNow(false).await
  }

  #[cfg(feature = "time")]
  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <WasmDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
  }

  #[cfg(feature = "time")]
  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <WasmDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  #[cfg(feature = "time")]
  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <WasmDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
  }

  #[cfg(feature = "time")]
  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <WasmDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }
}
