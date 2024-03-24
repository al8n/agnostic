use std::sync::Arc;

use futures_util::FutureExt;
use wasm::channel::oneshot::{channel, Canceled as JoinError, Sender};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled,
  Detach,
};

use super::{super::RuntimeLite, *};

/// The handle return by [`RuntimeLite::spawn_after`] or [`RuntimeLite::spawn_after_at`]
#[pin_project::pin_project]
pub struct WasmAfterHandle<O>
where
  O: 'static,
{
  #[pin]
  handle: WasmJoinHandle<Result<O, Canceled>>,
  signals: Arc<AfterHandleSignals>,
  abort_tx: Sender<()>,
  tx: Sender<()>,
}

impl<O: 'static> Future for WasmAfterHandle<O> {
  type Output = Result<O, AfterHandleError<JoinError>>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    match this.handle.poll(cx) {
      Poll::Ready(v) => match v {
        Ok(v) => Poll::Ready(v.map_err(|_| AfterHandleError::Canceled)),
        Err(e) => Poll::Ready(Err(AfterHandleError::Join(e))),
      },
      Poll::Pending => Poll::Pending,
    }
  }
}

impl<O> Detach for WasmAfterHandle<O> where O: 'static {}

impl<O> AfterHandle<O, JoinError> for WasmAfterHandle<O>
where
  O: Send + 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<JoinError>>> {
    if AfterHandle::is_finished(&self) {
      return Some(
        self
          .handle
          .await
          .map_err(AfterHandleError::Join)
          .and_then(|v| v.map_err(|_| AfterHandleError::Canceled)),
      );
    }

    let _ = self.tx.send(());
    None
  }

  #[inline]
  fn is_finished(&self) -> bool {
    self.signals.is_finished()
  }

  #[inline]
  fn is_expired(&self) -> bool {
    self.signals.is_expired()
  }

  #[inline]
  fn abort(self) {
    let _ = self.abort_tx.send(());
  }
}

impl<O> LocalAfterHandle<O, JoinError> for WasmAfterHandle<O>
where
  O: 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<JoinError>>> {
    if LocalAfterHandle::is_finished(&self) {
      return Some(
        self
          .handle
          .await
          .map_err(AfterHandleError::Join)
          .and_then(|v| v.map_err(|_| AfterHandleError::Canceled)),
      );
    }

    let _ = self.tx.send(());
    None
  }

  #[inline]
  fn is_finished(&self) -> bool {
    self.signals.is_finished()
  }

  #[inline]
  fn is_expired(&self) -> bool {
    self.signals.is_expired()
  }

  #[inline]
  fn abort(self) {
    let _ = self.abort_tx.send(());
  }
}

impl AsyncAfterSpawner for WasmSpawner {
  type JoinError = JoinError;

  type JoinHandle<F> = WasmAfterHandle<F>
    where
      F: Send + 'static;

  fn spawn_after<F>(duration: core::time::Duration, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    Self::spawn_after_at(Instant::now() + duration, future)
  }

  fn spawn_after_at<F>(instant: Instant, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    let (tx, rx) = channel::<()>();
    let (abort_tx, abort_rx) = channel::<()>();
    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    let h = WasmRuntime::spawn(async move {
      let delay = WasmRuntime::sleep_until(instant).fuse();
      let future = future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);

      futures_util::select_biased! {
        res = abort_rx => {
          if res.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          let res = future.await;
          s1.set_finished();
          Ok(res)
        }
        res = rx => {
          if res.is_ok() {
            return Err(Canceled);
          }

          delay.await;
          let res = future.await;
          s1.set_finished();
          Ok(res)
        }
        _ = delay => {
          s1.set_expired();
          futures_util::select_biased! {
            res = abort_rx => {
              if res.is_ok() {
                return Err(Canceled);
              }
              let res = future.await;
              s1.set_finished();
              Ok(res)
            }
            res = rx => {
              if res.is_ok() {
                return Err(Canceled);
              }
              let res = future.await;
              s1.set_finished();
              Ok(res)
            }
            res = future => {
              s1.set_finished();
              Ok(res)
            }
          }
        }
      }
    });

    WasmAfterHandle {
      handle: h,
      signals,
      abort_tx,
      tx,
    }
  }
}

impl AsyncLocalAfterSpawner for WasmSpawner {
  type JoinError = JoinError;
  type JoinHandle<F> = WasmAfterHandle<F>
    where
      F: 'static;

  fn spawn_local_after<F>(duration: core::time::Duration, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    Self::spawn_local_after_at(Instant::now() + duration, future)
  }

  fn spawn_local_after_at<F>(instant: Instant, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    let (tx, rx) = channel::<()>();
    let (abort_tx, abort_rx) = channel::<()>();
    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    let h = WasmRuntime::spawn_local(async move {
      let delay = WasmRuntime::sleep_local_until(instant).fuse();
      let future = future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);

      futures_util::select_biased! {
        res = abort_rx => {
          if res.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          let res = future.await;
          s1.set_finished();
          Ok(res)
        }
        res = rx => {
          if res.is_ok() {
            return Err(Canceled);
          }

          delay.await;
          let res = future.await;
          s1.set_finished();
          Ok(res)
        }
        _ = delay => {
          s1.set_expired();
          futures_util::select_biased! {
            res = abort_rx => {
              if res.is_ok() {
                return Err(Canceled);
              }
              let res = future.await;
              s1.set_finished();
              Ok(res)
            }
            res = rx => {
              if res.is_ok() {
                return Err(Canceled);
              }
              let res = future.await;
              s1.set_finished();
              Ok(res)
            }
            res = future => {
              s1.set_finished();
              Ok(res)
            }
          }
        }
      }
    });

    WasmAfterHandle {
      handle: h,
      signals,
      abort_tx,
      tx,
    }
  }
}
