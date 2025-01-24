cfg_time!(
  mod after;
  mod delay;
  mod interval;
  mod sleep;
  mod timeout;

  pub use after::*;
  pub use delay::*;
  pub use interval::*;
  pub use sleep::*;
  pub use timeout::*;

  use std::time::{Duration, Instant};
);

use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

use wasm::channel::*;

use crate::{AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder};

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
  type JoinHandle<F>
    = WasmJoinHandle<F>
  where
    F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    <Self as super::AsyncLocalSpawner>::spawn_local(future)
  }
}

impl AsyncLocalSpawner for WasmSpawner {
  type JoinHandle<F>
    = WasmJoinHandle<F>
  where
    F: 'static;

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

/// j
#[derive(Debug)]
pub enum JoinError {
  /// j
  Panic(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl std::error::Error for JoinError {}

impl std::fmt::Display for JoinError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      JoinError::Panic(e) => write!(f, "Thread panicked: {:?}", e),
    }
  }
}

impl From<JoinError> for std::io::Error {
  fn from(e: JoinError) -> Self {
    std::io::Error::new(std::io::ErrorKind::Other, e)
  }
}

///n
pub struct WasmBlockingJoinHandle<R> {
  handle: std::thread::JoinHandle<R>,
}

impl<R> super::Detach for WasmBlockingJoinHandle<R> {
  fn detach(self) {
    self.handle.detach();
  }
}

impl<R> Future for WasmBlockingJoinHandle<R> {
  type Output = Result<R, JoinError>;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    // if self.handle.is_finished() {
    //   match unsafe { Pin::into_inner_unchecked(self) }.handle.join() {
    //     Ok(result) => Poll::Ready(Ok(result)),
    //     Err(e) => Poll::Ready(Err(JoinError::Panic(e))),
    //   }
    // } else {
    //   cx.waker().wake_by_ref();
    //   Poll::Pending
    // }
    todo!()
  }
}

impl<R> super::JoinHandle<R> for WasmBlockingJoinHandle<R> {
  type JoinError = JoinError;
}

impl AsyncBlockingSpawner for WasmSpawner {
  type JoinHandle<R>
    = WasmBlockingJoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    let handle = std::thread::spawn(f);
    WasmBlockingJoinHandle { handle }
  }
}

impl Yielder for WasmSpawner {
  async fn yield_now() {
    YieldNow(false).await
  }

  async fn yield_now_local() {
    YieldNow(false).await
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

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on [`wasm-bindgen-futures`] runtime.
///
/// [`wasm-bindgen-futures`]: https://docs.rs/wasm-bindgen-futures
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

  cfg_time!(
    type AfterSpawner = WasmSpawner;

    type LocalAfterSpawner = WasmSpawner;

    type Interval = WasmInterval;

    type LocalInterval = WasmInterval;

    type Sleep = WasmSleep;

    type LocalSleep = WasmSleep;

    type Delay<F>
      = WasmDelay<F>
    where
      F: Future + Send;

    type LocalDelay<F>
      = WasmDelay<F>
    where
      F: Future;

    type Timeout<F>
      = WasmTimeout<F>
    where
      F: Future + Send;

    type LocalTimeout<F>
      = WasmTimeout<F>
    where
      F: Future;
  );

  fn new() -> Self {
    Self
  }

  fn name() -> &'static str {
    "wasm-bindgen-futures"
  }

  fn fqname() -> &'static str {
    "wasm-bindgen-futures"
  }

  fn block_on<F: Future>(_f: F) -> F::Output {
    panic!("RuntimeLite::block_on is not supported on wasm")
  }

  async fn yield_now() {
    YieldNow(false).await
  }

  cfg_time!(
    fn interval(interval: Duration) -> Self::Interval {
      use crate::time::AsyncIntervalExt;

      WasmInterval::interval(interval)
    }

    fn interval_at(start: Instant, period: Duration) -> Self::Interval {
      use crate::time::AsyncIntervalExt;

      WasmInterval::interval_at(start, period)
    }

    fn interval_local(interval: Duration) -> Self::LocalInterval {
      use crate::time::AsyncIntervalExt;

      WasmInterval::interval(interval)
    }

    fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval {
      use crate::time::AsyncIntervalExt;

      WasmInterval::interval_at(start, period)
    }

    fn sleep(duration: Duration) -> Self::Sleep {
      use crate::time::AsyncSleepExt;

      WasmSleep::sleep(duration)
    }

    fn sleep_until(instant: Instant) -> Self::Sleep {
      use crate::time::AsyncSleepExt;

      WasmSleep::sleep_until(instant)
    }

    fn sleep_local(duration: Duration) -> Self::LocalSleep {
      use crate::time::AsyncSleepExt;

      WasmSleep::sleep(duration)
    }

    fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
      use crate::time::AsyncSleepExt;

      WasmSleep::sleep_until(instant)
    }

    fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncDelayExt;

      <WasmDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
    }

    fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalDelayExt;

      <WasmDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
    }

    fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncDelayExt;

      <WasmDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
    }

    fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalDelayExt;

      <WasmDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
    }

    fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncTimeout;

      <WasmTimeout<F> as AsyncTimeout<F>>::timeout(duration, future)
    }

    fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
    where
      F: Future + Send,
    {
      use crate::time::AsyncTimeout;

      <WasmTimeout<F> as AsyncTimeout<F>>::timeout_at(deadline, future)
    }

    fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalTimeout;

      <WasmTimeout<F> as AsyncLocalTimeout<F>>::timeout_local(duration, future)
    }

    fn timeout_local_at<F>(deadline: Instant, future: F) -> Self::LocalTimeout<F>
    where
      F: Future,
    {
      use crate::time::AsyncLocalTimeout;

      <WasmTimeout<F> as AsyncLocalTimeout<F>>::timeout_local_at(deadline, future)
    }
  );
}
