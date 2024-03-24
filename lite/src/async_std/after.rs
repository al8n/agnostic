use core::{
  convert::Infallible,
  pin::Pin,
  task::{Context, Poll},
};

use std::sync::Arc;

use async_std::{
  channel::oneshot::{channel, Sender},
  task::JoinHandle,
};
use futures_util::FutureExt;

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled,
  Detach,
};

use super::{super::RuntimeLite, *};

/// The handle return by [`RuntimeLite::spawn_after`] or [`RuntimeLite::spawn_after_at`]
#[pin_project::pin_project]
pub struct AsyncStdAfterHandle<O>
where
  O: 'static,
{
  #[pin]
  handle: JoinHandle<Result<O, Canceled>>,
  signals: Arc<AfterHandleSignals>,
  abort_tx: Sender<()>,
  tx: Sender<()>,
}

impl<O: 'static> Future for AsyncStdAfterHandle<O> {
  type Output = Result<O, AfterHandleError<Infallible>>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    match this.handle.poll(cx) {
      Poll::Ready(v) => match v {
        Ok(v) => Poll::Ready(Ok(v)),
        Err(_) => Poll::Ready(Err(AfterHandleError::Canceled)),
      },
      Poll::Pending => Poll::Pending,
    }
  }
}

impl<O> Detach for AsyncStdAfterHandle<O> where O: 'static {}

impl<O> AfterHandle<O, Infallible> for AsyncStdAfterHandle<O>
where
  O: Send + 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<Infallible>>> {
    if AfterHandle::is_finished(&self) {
      return Some(self.handle.await.map_err(|_| AfterHandleError::Canceled));
    }

    let _ = self.tx.send(());
    None
  }

  #[inline]
  fn abort(self) {
    let _ = self.tx.send(());
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

impl<O> LocalAfterHandle<O, Infallible> for AsyncStdAfterHandle<O>
where
  O: 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<Infallible>>> {
    if LocalAfterHandle::is_finished(&self) {
      return Some(self.handle.await.map_err(|_| AfterHandleError::Canceled));
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

impl AsyncAfterSpawner for AsyncStdSpawner {
  type JoinError = Infallible;

  type JoinHandle<F> = AsyncStdAfterHandle<F>
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
    let h = AsyncStdRuntime::spawn(async move {
      let delay = AsyncStdRuntime::sleep_until(instant).fuse();
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

    AsyncStdAfterHandle {
      handle: h,
      signals,
      abort_tx,
      tx,
    }
  }
}

impl AsyncLocalAfterSpawner for AsyncStdSpawner {
  type JoinError = Infallible;
  type JoinHandle<F> = AsyncStdAfterHandle<F>
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
    let h = AsyncStdRuntime::spawn_local(async move {
      let delay = AsyncStdRuntime::sleep_local_until(instant).fuse();
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

    AsyncStdAfterHandle {
      handle: h,
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
      crate::tests::spawn_after_unittest::<AsyncStdRuntime>().await;
    });
  }

  #[test]
  fn test_after_drop() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_drop_unittest::<AsyncStdRuntime>().await;
    });
  }

  #[test]
  fn test_after_cancel() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_cancel_unittest::<AsyncStdRuntime>().await;
    });
  }

  #[test]
  fn test_after_abort() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_abort_unittest::<AsyncStdRuntime>().await;
    });
  }
}
