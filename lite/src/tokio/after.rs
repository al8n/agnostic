use core::{
  pin::Pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{Context, Poll},
};

use std::sync::Arc;

use tokio::{
  sync::oneshot::{channel, Sender},
  task::{JoinError, JoinHandle},
};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled, Detach,
};

use super::{super::RuntimeLite, *};

/// The handle return by [`RuntimeLite::spawn_after`] or [`RuntimeLite::spawn_after_at`]
#[pin_project::pin_project]
pub struct TokioAfterHandle<O>
where
  O: 'static,
{
  #[pin]
  handle: JoinHandle<Result<O, Canceled>>,
  finished: Arc<AtomicBool>,
  tx: Sender<()>,
}

impl<O: 'static> Future for TokioAfterHandle<O> {
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

impl<O> Detach for TokioAfterHandle<O> where O: 'static {}

impl<O> AfterHandle<O, JoinError> for TokioAfterHandle<O>
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

impl<O> LocalAfterHandle<O, JoinError> for TokioAfterHandle<O>
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

impl AsyncAfterSpawner for TokioSpawner {
  type JoinError = JoinError;

  type JoinHandle<F> = TokioAfterHandle<F>
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
    let h = TokioRuntime::spawn(async move {
      let delay = TokioRuntime::sleep_until(instant);
      tokio::pin!(delay);
      tokio::pin!(rx);
      tokio::pin!(future);

      tokio::select! {
        _ = &mut delay => {
          tokio::select! {
            res = &mut future => {
              f1.store(true, std::sync::atomic::Ordering::Release);
              Ok(res)
            }
            canceled = &mut rx => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              delay.await;
              Ok(future.await)
            }
          }
        }
        canceled = &mut rx => {
          if canceled.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          Ok(future.await)
        }
      }
    });

    TokioAfterHandle {
      handle: h,
      finished,
      tx,
    }
  }
}

impl AsyncLocalAfterSpawner for TokioSpawner {
  type JoinError = JoinError;

  type JoinHandle<F> = TokioAfterHandle<F>
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
    let h = TokioRuntime::spawn_local(async move {
      let delay = TokioRuntime::sleep_local_until(instant);
      tokio::pin!(delay);
      tokio::pin!(rx);
      tokio::pin!(future);

      tokio::select! {
        _ = &mut delay => {
          tokio::select! {
            res = &mut future => {
              f1.store(true, std::sync::atomic::Ordering::Release);
              Ok(res)
            }
            canceled = &mut rx => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              delay.await;
              Ok(future.await)
            }
          }
        }
        canceled = &mut rx => {
          if canceled.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          Ok(future.await)
        }
      }
    });

    TokioAfterHandle {
      handle: h,
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
      crate::tests::spawn_after_unittest::<TokioRuntime>().await;
    });
  }

  #[test]
  fn test_after_drop() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_drop_unittest::<TokioRuntime>().await;
    });
  }

  #[test]
  fn test_after_cancel() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_cancel_unittest::<TokioRuntime>().await;
    });
  }
}
