use core::{future::Future, pin::Pin, sync::atomic::Ordering, task::Poll, time::Duration};
use std::sync::Arc;

use atomic_time::AtomicOptionDuration;
use futures_channel::oneshot::{Sender, channel};
use futures_util::{FutureExt, task::AtomicWaker};

use crate::{
  AfterHandleError, AfterHandleSignals, AsyncAfterSpawner, Canceled, RuntimeLite,
  spawner::{AfterHandle, handle::JoinError},
  time::AsyncSleep,
};

use super::{EmbassyRuntime, EmbassySpawner, Instant, JoinHandle};

// Shared between the `EmbassyAfterHandle` and its driving task. `duration` carries the new delay
// requested by `reset` (relative to the original scheduling instant); `waker` wakes the driving
// task so it observes the new duration promptly.
struct ResetState {
  duration: AtomicOptionDuration,
  waker: AtomicWaker,
}

impl ResetState {
  const fn new() -> Self {
    Self {
      duration: AtomicOptionDuration::none(),
      waker: AtomicWaker::new(),
    }
  }
}

pub(crate) struct Resetter {
  state: Arc<ResetState>,
}

impl Resetter {
  fn reset(&self, duration: Duration) {
    self.state.duration.store(Some(duration), Ordering::Release);
    self.state.waker.wake();
  }
}

/// The handle returned by [`RuntimeLite::spawn_after`] or [`RuntimeLite::spawn_after_at`] for the
/// embassy runtime.
#[pin_project::pin_project]
pub struct EmbassyAfterHandle<O>
where
  O: 'static,
{
  #[pin]
  handle: JoinHandle<Result<O, Canceled>>,
  signals: Arc<AfterHandleSignals>,
  abort_tx: Sender<()>,
  resetter: Resetter,
  tx: Sender<()>,
}

impl<O: 'static> Future for EmbassyAfterHandle<O> {
  type Output = Result<O, AfterHandleError<JoinError>>;

  fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
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

impl<O> AfterHandle<O> for EmbassyAfterHandle<O>
where
  O: Send + 'static,
{
  type JoinError = AfterHandleError<JoinError>;

  async fn cancel(self) -> Option<Result<O, Self::JoinError>> {
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

  fn reset(&self, duration: Duration) {
    self.resetter.reset(duration);
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

impl AsyncAfterSpawner for EmbassySpawner {
  type Instant = Instant;

  type JoinHandle<F>
    = EmbassyAfterHandle<F>
  where
    F: Send + 'static;

  fn spawn_after<F>(duration: Duration, future: F) -> Self::JoinHandle<F::Output>
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
    let state = Arc::new(ResetState::new());
    let resetter = Resetter {
      state: state.clone(),
    };
    let s1 = signals.clone();
    // The instant at which the task was scheduled. `reset(d)` reschedules the timer to fire at
    // `start + d` — `d` is measured from the original scheduling instant, not from the `reset` call.
    let start = Instant::now();

    let handle = EmbassyRuntime::spawn(async move {
      let delay = EmbassyRuntime::sleep_until(instant);
      let future = future.fuse();
      futures_util::pin_mut!(delay);
      futures_util::pin_mut!(rx);
      futures_util::pin_mut!(abort_rx);
      futures_util::pin_mut!(future);

      loop {
        let reset = futures_util::future::poll_fn(|cx| {
          state.waker.register(cx.waker());
          match state.duration.swap(None, Ordering::AcqRel) {
            Some(d) => Poll::Ready(d),
            None => Poll::Pending,
          }
        })
        .fuse();
        futures_util::pin_mut!(reset);

        futures_util::select_biased! {
          res = abort_rx => {
            if res.is_ok() {
              return Err(Canceled);
            }
            // The handle was dropped: detach and run to completion after the delay.
            delay.await;
            s1.set_expired();
            let res = future.await;
            s1.set_finished();
            return Ok(res);
          }
          res = rx => {
            if res.is_ok() {
              return Err(Canceled);
            }
            delay.await;
            s1.set_expired();
            let res = future.await;
            s1.set_finished();
            return Ok(res);
          }
          d = reset => {
            let deadline = start + d;
            if deadline <= Instant::now() {
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
            } else {
              AsyncSleep::reset(delay.as_mut(), deadline);
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

    EmbassyAfterHandle {
      handle,
      signals,
      abort_tx,
      resetter,
      tx,
    }
  }
}
