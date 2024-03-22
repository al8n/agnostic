use core::{
  pin::Pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{Context, Poll},
};

use std::sync::Arc;

use futures_util::future::{select, Either};
use smol::sync::oneshot::{channel, Canceled as JoinError, Receiver, Sender};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled, Detach,
};

use super::{super::RuntimeLite, *};

/// The handle return by [`RuntimeLite::spawn_after`] or [`RuntimeLite::spawn_after_at`]
#[pin_project::pin_project]
pub struct SmolAfterHandle<O>
where
  O: 'static,
{
  #[pin]
  handle: Receiver<Result<O, Canceled>>,
  finished: Arc<AtomicBool>,
  tx: Sender<()>,
}

impl<O: 'static> Future for SmolAfterHandle<O> {
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

impl<O> Detach for SmolAfterHandle<O> where O: 'static {}

impl<O> AfterHandle<O, JoinError> for SmolAfterHandle<O>
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

impl<O> LocalAfterHandle<O, JoinError> for SmolAfterHandle<O>
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

impl AsyncAfterSpawner for SmolSpawner {
  type JoinError = JoinError;

  type JoinHandle<F> = SmolAfterHandle<F>
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
    let (handle_tx, handle) = channel::<Result<F::Output, Canceled>>();
    let finished = Arc::new(AtomicBool::new(false));
    let f1 = finished.clone();
    SmolRuntime::spawn_detach(async move {
      let delay = SmolRuntime::sleep_until(instant);
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);

      match select(&mut delay, rx).await {
        Either::Left((_, rx)) => {
          futures_util::pin_mut!(future);
          match select(future, rx).await {
            Either::Left((res, _)) => {
              f1.store(true, Ordering::Release);
              let _ = handle_tx.send(Ok(res));
            }
            Either::Right((canceled, fut)) => {
              if canceled.is_ok() {
                let _ = handle_tx.send(Err(Canceled));
                return;
              }
              let _ = handle_tx.send(Ok(fut.await));
            }
          }
        }
        Either::Right((canceled, _)) => {
          if canceled.is_ok() {
            let _ = handle_tx.send(Err(Canceled));
            return;
          }
          delay.await;
          let _ = handle_tx.send(Ok(future.await));
        }
      }
    });

    SmolAfterHandle {
      handle,
      finished,
      tx,
    }
  }
}

impl AsyncLocalAfterSpawner for SmolSpawner {
  type JoinError = JoinError;
  type JoinHandle<F> = SmolAfterHandle<F>
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
    let (handle_tx, handle) = channel::<Result<F::Output, Canceled>>();
    let finished = Arc::new(AtomicBool::new(false));
    let f1 = finished.clone();
    SmolRuntime::spawn_local_detach(async move {
      let delay = SmolRuntime::sleep_until(instant);
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);

      match select(&mut delay, rx).await {
        Either::Left((_, rx)) => {
          futures_util::pin_mut!(future);
          match select(future, rx).await {
            Either::Left((res, _)) => {
              f1.store(true, Ordering::Release);
              let _ = handle_tx.send(Ok(res));
            }
            Either::Right((canceled, fut)) => {
              if canceled.is_ok() {
                let _ = handle_tx.send(Err(Canceled));
                return;
              }
              let _ = handle_tx.send(Ok(fut.await));
            }
          }
        }
        Either::Right((canceled, _)) => {
          if canceled.is_ok() {
            let _ = handle_tx.send(Err(Canceled));
            return;
          }
          delay.await;
          let _ = handle_tx.send(Ok(future.await));
        }
      }
    });

    SmolAfterHandle {
      handle,
      finished,
      tx,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_after_handle() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_unittest::<SmolRuntime>().await;
    });
  }

  #[test]
  fn test_after_drop() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_drop_unittest::<SmolRuntime>().await;
    });
  }

  #[test]
  fn test_after_cancel() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_cancel_unittest::<SmolRuntime>().await;
    });
  }
}
