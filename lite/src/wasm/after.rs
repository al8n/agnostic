use core::sync::atomic::{AtomicBool, Ordering};

use std::sync::Arc;

use futures_util::future::{select, Either};
use wasm::channel::oneshot::{channel, Canceled as JoinError, Sender};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled, Detach,
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
  finished: Arc<AtomicBool>,
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
    if self.finished.load(Ordering::Acquire) {
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
}

impl<O> LocalAfterHandle<O, JoinError> for WasmAfterHandle<O>
where
  O: 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<JoinError>>> {
    if self.finished.load(Ordering::Acquire) {
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
    let finished = Arc::new(AtomicBool::new(false));
    let f1 = finished.clone();
    let h = WasmRuntime::spawn(async move {
      let delay = WasmRuntime::sleep_until(instant);
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);

      match select(&mut delay, rx).await {
        Either::Left((_, rx)) => {
          futures_util::pin_mut!(future);
          match select(future, rx).await {
            Either::Left((res, _)) => {
              f1.store(true, Ordering::Release);
              Ok(res)
            }
            Either::Right((canceled, fut)) => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              Ok(fut.await)
            }
          }
        }
        Either::Right((canceled, _)) => {
          if canceled.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          Ok(future.await)
        }
      }
    });

    WasmAfterHandle {
      handle: h,
      finished,
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
    let finished = Arc::new(AtomicBool::new(false));
    let f1 = finished.clone();
    let h = WasmRuntime::spawn_local(async move {
      let delay = WasmRuntime::sleep_local_until(instant);
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);

      match select(&mut delay, rx).await {
        Either::Left((_, rx)) => {
          futures_util::pin_mut!(future);
          match select(future, rx).await {
            Either::Left((res, _)) => {
              f1.store(true, Ordering::Release);
              Ok(res)
            }
            Either::Right((canceled, fut)) => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              Ok(fut.await)
            }
          }
        }
        Either::Right((canceled, _)) => {
          if canceled.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          Ok(future.await)
        }
      }
    });

    WasmAfterHandle {
      handle: h,
      finished,
      tx,
    }
  }
}
