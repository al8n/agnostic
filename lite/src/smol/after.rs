use core::{
  pin::Pin,
  sync::atomic::Ordering,
  task::{Context, Poll},
};

use std::sync::Arc;

use atomic_time::AtomicOptionDuration;
use futures_util::{FutureExt, StreamExt};
use smol::{
  sync::{
    mpsc::{unbounded, UnboundedSender},
    oneshot::{channel, Canceled as JoinError, Receiver, Sender},
  },
  // channel::{Sender as UnboundedSender, unbounded}
};

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  time::{AsyncLocalSleep, AsyncSleep},
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled,
  Detach,
};

use super::{super::RuntimeLite, *};

pub(crate) struct Resetter {
  duration: Arc<AtomicOptionDuration>,
  tx: UnboundedSender<()>,
}

impl Resetter {
  pub(crate) fn new(duration: Arc<AtomicOptionDuration>, tx: UnboundedSender<()>) -> Self {
    Self { duration, tx }
  }

  pub(crate) fn reset(&self, duration: Duration) {
    self.duration.store(Some(duration), Ordering::Release);
  }
}

macro_rules! spawn_after {
  ($spawn:ident, $sleep:ident($trait:ident) -> ($instant:ident, $future:ident)) => {{
    let (tx, rx) = channel::<()>();
    let (abort_tx, abort_rx) = channel::<()>();
    let (handle_tx, handle) = channel::<Result<F::Output, Canceled>>();
    let (reset_tx, mut reset_rx) = unbounded();
    let duration = Arc::new(AtomicOptionDuration::none());
    let resetter = Resetter::new(duration.clone(), reset_tx);
    let signals = Arc::new(AfterHandleSignals::new());
    let s1 = signals.clone();
    SmolRuntime::$spawn(async move {
      let delay = SmolRuntime::$sleep($instant);
      let future = $future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);

      loop {
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
            return;
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
            return;
          }
          signal = reset_rx.next().fuse() => {
            if signal.is_some() {
              if let Some(d) = duration.load(Ordering::Acquire) {
                if $instant.checked_sub(d).is_some() {
                  s1.set_expired();

                  futures_util::select_biased! {
                    res = &mut future => {
                      s1.set_finished();
                      let _ = handle_tx.send(Ok(res));
                      return;
                    }
                    canceled = &mut rx => {
                      if canceled.is_ok() {
                        let _ = handle_tx.send(Err(Canceled));
                        return;
                      }
                      delay.await;
                      s1.set_expired();
                      let res = future.await;
                      s1.set_finished();
                      let _ = handle_tx.send(Ok(res));
                      return;
                    }
                  }
                }

                match $instant.checked_sub(d) {
                  Some(v) => {
                    $trait::reset(delay.as_mut(), v);
                  },
                  None => {
                    match d.checked_sub($instant.elapsed()) {
                      Some(v) => {
                        $trait::reset(delay.as_mut(), Instant::now() + v);
                      },
                      None => {
                        s1.set_expired();

                        futures_util::select_biased! {
                          res = &mut future => {
                            s1.set_finished();
                            let _ = handle_tx.send(Ok(res));
                            return;
                          }
                          canceled = &mut rx => {
                            if canceled.is_ok() {
                              let _ = handle_tx.send(Err(Canceled));
                              return;
                            }
                            delay.await;
                            s1.set_expired();
                            let res = future.await;
                            s1.set_finished();
                            let _ = handle_tx.send(Ok(res));
                            return;
                          }
                        }
                      },
                    }
                  },
                }
              }
            } else {
              delay.await;
              let res = future.await;
              s1.set_finished();
              let _ = handle_tx.send(Ok(res));
              return;
            }
          }
          _ = delay.as_mut().fuse() => {
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
                return;
              }
              res = rx => {
                if res.is_ok() {
                  let _ = handle_tx.send(Err(Canceled));
                  return;
                }
                let res = future.await;
                s1.set_finished();
                let _ = handle_tx.send(Ok(res));
                return;
              }
              res = future => {
                s1.set_finished();
                let _ = handle_tx.send(Ok(res));
                return;
              }
            }
          }
        }
      }
    });

    SmolAfterHandle {
      handle,
      resetter,
      signals,
      abort_tx,
      tx,
    }
  }};
}

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
  resetter: Resetter,
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

  fn reset(&self, duration: core::time::Duration) {
    self.resetter.reset(duration);
    let _ = self.resetter.tx.unbounded_send(());
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

  fn reset(&self, duration: core::time::Duration) {
    self.resetter.reset(duration);
    let _ = self.resetter.tx.unbounded_send(());
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
    spawn_after!(spawn_detach, sleep_until(AsyncSleep) -> (instant, future))
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
    spawn_after!(spawn_local_detach, sleep_local_until(AsyncLocalSleep) -> (instant, future))
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

  #[test]
  fn test_after_reset_to_pass() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_reset_to_pass_unittest::<SmolRuntime>().await;
    });
  }

  #[test]
  fn test_after_reset_to_future() {
    futures::executor::block_on(async {
      crate::tests::spawn_after_reset_to_future_unittest::<SmolRuntime>().await;
    });
  }
}
