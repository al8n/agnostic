use std::sync::{atomic::Ordering, Arc};

use atomic_time::AtomicOptionDuration;
use futures_util::{FutureExt, StreamExt};
use wasm::channel::{
  mpsc::{unbounded, UnboundedSender},
  oneshot::{channel, Canceled as JoinError, Sender},
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
    let signals = Arc::new(AfterHandleSignals::new());
    let (reset_tx, mut reset_rx) = unbounded::<()>();
    let duration = Arc::new(AtomicOptionDuration::none());
    let resetter = Resetter::new(duration.clone(), reset_tx);
    let s1 = signals.clone();
    let h = WasmRuntime::$spawn(async move {
      let delay = WasmRuntime::$sleep($instant);
      let future = $future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);
      loop {
        futures_util::select_biased! {
          res = abort_rx => {
            if res.is_ok() {
              return Err(Canceled);
            }
            delay.await;
            let res = future.await;
            s1.set_finished();
            return Ok(res);
          }
          res = rx => {
            if res.is_ok() {
              return Err(Canceled);
            }

            delay.await;
            let res = future.await;
            s1.set_finished();
            return Ok(res);
          }
          res = reset_rx.next() => {
            if res.is_none() {
              delay.await;
              let res = future.await;
              s1.set_finished();
              return Ok(res);
            }

            if let Some(d) = duration.load(Ordering::Acquire) {
              if $instant.checked_sub(d).is_some() {
                s1.set_expired();

                futures_util::select_biased! {
                  res = &mut future => {
                    s1.set_finished();
                    return Ok(res);
                  }
                  canceled = &mut rx => {
                    if canceled.is_ok() {
                      return Err(Canceled);
                    }
                    delay.await;
                    s1.set_expired();
                    let res = future.await;
                    s1.set_finished();
                    return Ok(res);
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
                          return Ok(res);
                        }
                        canceled = &mut rx => {
                          if canceled.is_ok() {
                            return Err(Canceled);
                          }
                          delay.await;
                          s1.set_expired();
                          let res = future.await;
                          s1.set_finished();
                          return Ok(res);
                        }
                      }
                    },
                  }
                },
              }
            }
          }
          _ = delay.as_mut().fuse() => {
            s1.set_expired();
            futures_util::select_biased! {
              res = abort_rx => {
                if res.is_ok() {
                  return Err(Canceled);
                }
                let res = future.await;
                s1.set_finished();
                return Ok(res);
              }
              res = rx => {
                if res.is_ok() {
                  return Err(Canceled);
                }
                let res = future.await;
                s1.set_finished();
                return Ok(res);
              }
              res = future => {
                s1.set_finished();
                return Ok(res);
              }
            }
          }
        }
      }
    });

    WasmAfterHandle {
      handle: h,
      resetter,
      signals,
      abort_tx,
      tx,
    }
  }};
}

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
  resetter: Resetter,
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

  fn reset(&self, duration: Duration) {
    self.resetter.reset(duration);
    let _ = self.resetter.tx.unbounded_send(());
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

  fn reset(&self, duration: Duration) {
    self.resetter.reset(duration);
    let _ = self.resetter.tx.unbounded_send(());
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
    spawn_after!(spawn, sleep_until(AsyncSleep) -> (instant, future))
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
    spawn_after!(spawn_local, sleep_local_until(AsyncLocalSleep) -> (instant, future))
  }
}
