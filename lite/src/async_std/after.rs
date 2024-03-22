use core::{
  convert::Infallible,
  pin::Pin,
  sync::atomic::{AtomicBool, Ordering},
  task::{Context, Poll},
};

use std::sync::Arc;

use async_std::{
  channel::oneshot::{channel, Sender},
  task::JoinHandle,
};
use futures_util::future::{select, Either};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  AfterHandleError, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled, Detach,
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
  finished: Arc<AtomicBool>,
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
    if self.finished.load(Ordering::Acquire) {
      return Some(self.handle.await.map_err(|_| AfterHandleError::Canceled));
    }

    let _ = self.tx.send(());
    None
  }
}

impl<O> LocalAfterHandle<O, Infallible> for AsyncStdAfterHandle<O>
where
  O: 'static,
{
  async fn cancel(self) -> Option<Result<O, AfterHandleError<Infallible>>> {
    if self.finished.load(Ordering::Acquire) {
      return Some(self.handle.await.map_err(|_| AfterHandleError::Canceled));
    }

    let _ = self.tx.send(());
    None
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
    let finished = Arc::new(AtomicBool::new(false));
    let f1 = finished.clone();
    let h = AsyncStdRuntime::spawn(async move {
      let delay = AsyncStdRuntime::sleep_until(instant);
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

    AsyncStdAfterHandle {
      handle: h,
      finished,
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
    let finished = Arc::new(AtomicBool::new(false));
    let f1 = finished.clone();
    let h = AsyncStdRuntime::spawn_local(async move {
      let delay = AsyncStdRuntime::sleep_local_until(instant);
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

    AsyncStdAfterHandle {
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
}
