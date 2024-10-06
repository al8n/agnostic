use core::{
  pin::Pin,
  sync::atomic::Ordering,
  task::{Context, Poll},
};

use std::sync::Arc;

use crate::{
  spawner::{AfterHandle, LocalAfterHandle},
  time::AsyncSleep,
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, AsyncLocalAfterSpawner, Canceled,
  Detach,
};
use atomic_time::AtomicOptionDuration;
use tokio::{
  sync::{
    mpsc,
    oneshot::{channel, Sender},
  },
  task::{JoinError, JoinHandle},
};

use super::{super::RuntimeLite, *};

pub(crate) struct Resetter {
  duration: Arc<AtomicOptionDuration>,
  tx: mpsc::UnboundedSender<()>,
}

impl Resetter {
  pub(crate) fn new(duration: Arc<AtomicOptionDuration>, tx: mpsc::UnboundedSender<()>) -> Self {
    Self { duration, tx }
  }

  pub(crate) fn reset(&self, duration: Duration) {
    self.duration.store(Some(duration), Ordering::Release);
  }
}

macro_rules! spawn_after {
  ($spawn:ident, $sleep:ident -> ($instant:ident, $future:ident)) => {{
    paste::paste! {
      let (tx, rx) = channel::<()>();
      let (reset_tx, mut reset_rx) = mpsc::unbounded_channel();
      let signals = Arc::new(AfterHandleSignals::new());
      let duration = Arc::new(AtomicOptionDuration::none());
      let resetter = Resetter::new(duration.clone(), reset_tx);
      let s1 = signals.clone();
      let h = TokioRuntime::$spawn(async move {
        let delay = TokioRuntime::$sleep($instant);
        tokio::pin!(delay);
        tokio::pin!(rx);
        tokio::pin!($future);

        loop {
          tokio::select! {
            biased;
            canceled = &mut rx => {
              if canceled.is_ok() {
                return Err(Canceled);
              }
              delay.await;
              s1.set_expired();
              let res = $future.await;
              s1.set_finished();
              return Ok(res);
            }
            signal = reset_rx.recv() => {
              if signal.is_some() {
                if let Some(d) = duration.load(Ordering::Acquire) {
                  if $instant.checked_sub(d).is_some() {
                    s1.set_expired();

                    tokio::select! {
                      canceled = &mut rx => {
                        if canceled.is_ok() {
                          return Err(Canceled);
                        }
                        delay.await;
                        s1.set_expired();
                        let res = $future.await;
                        s1.set_finished();
                        return Ok(res);
                      }
                      res = &mut $future => {
                        s1.set_finished();
                        return Ok(res);
                      }
                    }
                  }

                  match $instant.checked_sub(d) {
                    Some(v) => {
                      delay.as_mut().reset(v);
                    },
                    None => {
                      match d.checked_sub($instant.elapsed()) {
                        Some(v) => {
                          delay.as_mut().reset(Instant::now() + v);
                        },
                        None => {
                          s1.set_expired();

                          tokio::select! {
                            canceled = &mut rx => {
                              if canceled.is_ok() {
                                return Err(Canceled);
                              }
                              delay.await;
                              s1.set_expired();
                              let res = $future.await;
                              s1.set_finished();
                              return Ok(res);
                            }
                            res = &mut $future => {
                              s1.set_finished();
                              return Ok(res);
                            }
                          }
                        },
                      }
                    },
                  }
                }
              } else {
                delay.await;
                s1.set_expired();
                let res = $future.await;
                s1.set_finished();
                return Ok(res);
              }
            }
            _ = &mut delay => {
              s1.set_expired();

              tokio::select! {
                biased;
                canceled = &mut rx => {
                  if canceled.is_ok() {
                    return Err(Canceled);
                  }
                  delay.await;
                  s1.set_expired();
                  let res = $future.await;
                  s1.set_finished();
                  return Ok(res);
                }
                res = &mut $future => {
                  s1.set_finished();
                  return Ok(res);
                }
              }
            }
          }
        }
      });

      TokioAfterHandle {
        handle: h,
        signals,
        resetter,
        tx,
      }
    }
  }};
}

/// The handle return by [`RuntimeLite::spawn_after`] or [`RuntimeLite::spawn_after_at`]
#[pin_project::pin_project]
pub struct TokioAfterHandle<O>
where
  O: 'static,
{
  #[pin]
  handle: JoinHandle<Result<O, Canceled>>,
  signals: Arc<AfterHandleSignals>,
  resetter: Resetter,
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

  fn reset(&self, duration: core::time::Duration) {
    self.resetter.reset(duration);
    let _ = self.resetter.tx.send(());
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

  fn reset(&self, duration: core::time::Duration) {
    self.resetter.reset(duration);
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

  type JoinHandle<F>
    = TokioAfterHandle<F>
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
    spawn_after!(spawn, sleep_until -> (instant, future))
  }
}

impl AsyncLocalAfterSpawner for TokioSpawner {
  type JoinError = JoinError;

  type JoinHandle<F>
    = TokioAfterHandle<F>
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
    spawn_after!(spawn_local, sleep_local_until -> (instant, future))
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

  #[tokio::test]
  async fn test_after_reset_to_pass() {
    crate::tests::spawn_after_reset_to_pass_unittest::<TokioRuntime>().await;
  }

  #[tokio::test]
  async fn test_after_reset_to_future() {
    crate::tests::spawn_after_reset_to_future_unittest::<TokioRuntime>().await;
  }
}
