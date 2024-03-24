use core::{
  pin::Pin,
  task::{Context, Poll},
};

use std::sync::Arc;

use futures_util::FutureExt;
// use futures_util::future::{select, Either};
use smol::sync::oneshot::{channel, Canceled as JoinError, Receiver, Sender};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled,
  Detach,
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
  signals: Arc<AfterHandleSignals>,
  abort_tx: Sender<()>,
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
  fn abort(self) {
    let _ = self.abort_tx.send(());
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

impl<O> LocalAfterHandle<O, JoinError> for SmolAfterHandle<O>
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
  fn abort(self) {
    let _ = self.abort_tx.send(());
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
    let (abort_tx, abort_rx) = channel::<()>();
    let (handle_tx, handle) = channel::<Result<F::Output, Canceled>>();
    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    SmolRuntime::spawn_detach(async move {
      let delay = SmolRuntime::sleep_until(instant).fuse();
      let future = future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);

      futures_util::select_biased! {
        res = abort_rx => {
          if res.is_ok() {
            let _ = handle_tx.send(Err(Canceled));
            return;
          }
          delay.await;
          let res = future.await;
          s1.set_finished();
          let _ = handle_tx.send(Ok(res));
        }
        res = rx => {
          if res.is_ok() {
            let _ = handle_tx.send(Err(Canceled));
            return;
          }

          delay.await;
          let res = future.await;
          s1.set_finished();
          let _ = handle_tx.send(Ok(res));
        }
        _ = delay => {
          s1.set_expired();
          futures_util::select_biased! {
            res = abort_rx => {
              if res.is_ok() {
                let _ = handle_tx.send(Err(Canceled));
                return;
              }
              let res = future.await;
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
            }
            res = rx => {
              if res.is_ok() {
                let _ = handle_tx.send(Err(Canceled));
                return;
              }
              let res = future.await;
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
            }
            res = future => {
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
            }
          }
        }
      }
    });

    SmolAfterHandle {
      handle,
      signals,
      abort_tx,
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
    let (abort_tx, abort_rx) = channel::<()>();
    let (handle_tx, handle) = channel::<Result<F::Output, Canceled>>();
    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    SmolRuntime::spawn_local_detach(async move {
      let delay = SmolRuntime::sleep_local_until(instant).fuse();
      let future = future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);

      futures_util::select_biased! {
        res = abort_rx => {
          if res.is_ok() {
            let _ = handle_tx.send(Err(Canceled));
            return;
          }
          delay.await;
          s1.set_expired();
          let res = future.await;
          s1.set_finished();
          let _ = handle_tx.send(Ok(res));
        }
        res = rx => {
          if res.is_ok() {
            let _ = handle_tx.send(Err(Canceled));
            return;
          }

          delay.await;
          s1.set_expired();
          let res = future.await;
          s1.set_finished();
          let _ = handle_tx.send(Ok(res));
        }
        _ = delay => {
          s1.set_expired();
          futures_util::select_biased! {
            res = abort_rx => {
              if res.is_ok() {
                let _ = handle_tx.send(Err(Canceled));
                return;
              }
              let res = future.await;
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
            }
            res = rx => {
              if res.is_ok() {
                let _ = handle_tx.send(Err(Canceled));
                return;
              }
              let res = future.await;
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
            }
            res = future => {
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
            }
          }
        }
      }
    });

    SmolAfterHandle {
      handle,
      signals,
      abort_tx,
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

  #[test]
  fn test_after_abort() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_abort_unittest::<SmolRuntime>().await;
    });
  }
}
