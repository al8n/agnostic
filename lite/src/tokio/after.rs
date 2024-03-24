use core::{
  pin::Pin,
  task::{Context, Poll},
};

use std::sync::Arc;

use tokio::{
  sync::oneshot::{channel, Sender},
  task::{JoinError, JoinHandle},
};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled,
  Detach,
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
  signals: Arc<AfterHandleSignals>,
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
    if self.signals.is_finished() {
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
  fn abort(self) {
    self.handle.abort();
  }

  #[inline]
  fn is_expired(&self) -> bool {
    self.signals.is_expired()
  }

  #[inline]
  fn is_finished(&self) -> bool {
    self.signals.is_finished()
  }
}

impl<O> LocalAfterHandle<O, JoinError> for TokioAfterHandle<O>
where
  O: 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<JoinError>>> {
    if self.is_finished() {
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

  fn is_finished(&self) -> bool {
    self.signals.is_finished()
  }

  fn is_expired(&self) -> bool {
    self.signals.is_expired()
  }

  fn abort(self) {
    self.handle.abort()
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

    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    let h = TokioRuntime::spawn(async move {
      let delay = TokioRuntime::sleep_until(instant);
      tokio::pin!(delay);
      tokio::pin!(rx);
      tokio::pin!(future);

      tokio::select! {
        _ = &mut delay => {
          s1.set_expired();

          tokio::select! {
            res = &mut future => {
              s1.set_finished();
              Ok(res)
            }
            canceled = &mut rx => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              delay.await;
              s1.set_expired();
              let res = future.await;
              s1.set_finished();
              Ok(res)
            }
          }
        }
        canceled = &mut rx => {
          if canceled.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          s1.set_expired();
          let res = future.await;
          s1.set_finished();
          Ok(res)
        }
      }
    });

    TokioAfterHandle {
      handle: h,
      signals,
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

    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    let h = TokioRuntime::spawn_local(async move {
      let delay = TokioRuntime::sleep_local_until(instant);
      tokio::pin!(delay);
      tokio::pin!(rx);
      tokio::pin!(future);

      tokio::select! {
        _ = &mut delay => {
          s1.set_expired();

          tokio::select! {
            res = &mut future => {
              s1.set_finished();
              Ok(res)
            }
            canceled = &mut rx => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              delay.await;
              s1.set_expired();
              let res = future.await;
              s1.set_finished();
              Ok(res)
            }
          }
        }
        canceled = &mut rx => {
          if canceled.is_ok() {
            return Err(Canceled);
          }
          delay.await;
          s1.set_expired();
          let res = future.await;
          s1.set_finished();
          Ok(res)
        }
      }
    });

    TokioAfterHandle {
      handle: h,
      signals,
      tx,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_after_handle() {
    crate::tests::spawn_after_unittest::<TokioRuntime>().await;
  }

  #[tokio::test]
  async fn test_after_drop() {
    crate::tests::spawn_after_drop_unittest::<TokioRuntime>().await;
  }

  #[tokio::test]
  async fn test_after_cancel() {
    crate::tests::spawn_after_cancel_unittest::<TokioRuntime>().await;
  }

  #[tokio::test]
  async fn test_after_abort() {
    crate::tests::spawn_after_abort_unittest::<TokioRuntime>().await;
  }
}
