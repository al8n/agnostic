use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

pub use super::timer::*;
use super::*;
use ::smol::channel;
use async_io::Timer;
use futures_util::FutureExt;

struct DelayFuncHandle<F: Future> {
  handle: ::smol::Task<Option<F::Output>>,
  reset_tx: channel::Sender<Duration>,
  finished: Arc<AtomicBool>,
}

pub struct SmolDelay<F: Future> {
  handle: Option<DelayFuncHandle<F>>,
}

#[async_trait::async_trait]
impl<F> Delay<F> for SmolDelay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self {
    let finished = Arc::new(AtomicBool::new(false));
    let ff = finished.clone();
    let (reset_tx, reset_rx) = channel::bounded(1);
    let handle = ::smol::spawn(async move {
      let mut sleep = Timer::after(delay);
      loop {
        ::futures_util::select! {
          _ = sleep.fuse() => {
            let rst = fut.await;
            ff.store(true, ::std::sync::atomic::Ordering::SeqCst);
            return Some(rst);
          },
          remaining = reset_rx.recv().fuse() => {
            if let Ok(remaining) = remaining {
              sleep = Timer::after(remaining);
            } else {
              return None;
            }
          }
        }
      }
    });
    Self {
      handle: Some(DelayFuncHandle {
        reset_tx,
        handle,
        finished,
      }),
    }
  }

  async fn reset(&mut self, dur: Duration) {
    if let Some(handle) = &mut self.handle {
      // if we fail to send a message, which means the rx has been dropped, and that thread has exited
      let _ = handle.reset_tx.send(dur).await;
    }
  }

  async fn cancel(&mut self) -> Option<F::Output> {
    if let Some(handle) = self.handle.take() {
      if handle.finished.load(Ordering::SeqCst) {
        return handle.handle.await;
      } else {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        handle.handle.cancel().await;
        return None;
      }
    }
    None
  }
}

#[derive(Debug, Copy, Clone)]
pub struct SmolRuntime;

impl core::fmt::Display for SmolRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "smol")
  }
}

#[async_trait::async_trait]
impl Runtime for SmolRuntime {
  type JoinHandle<T> = ::smol::Task<T>;
  type Interval = Timer;
  type Sleep = Timer;
  type Delay<F> = SmolDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = Timeout<F> where F: Future;

  fn spawn<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    ::smol::spawn(fut)
  }

  fn spawn_local<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    ::smol::LocalExecutor::new().spawn(fut)
  }

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::spawn(::smol::unblock(f))
  }

  fn detach_spawn(&self) -> bool {
    false
  }

  fn interval(&self, interval: Duration) -> Self::Interval {
    Timer::interval(interval)
  }

  fn interval_at(&self, start: Instant, period: Duration) -> Self::Interval {
    Timer::interval_at(start, period)
  }

  fn sleep(&self, duration: Duration) -> Self::Sleep {
    Timer::after(duration)
  }

  fn delay<F>(&self, delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    SmolDelay::new(delay, fut)
  }

  fn timeout<F>(&self, duration: Duration, fut: F) -> Self::Timeout<F>
  where
    F: Future,
  {
    Timeout::new(duration, fut)
  }

  fn timeout_at<F>(&self, instant: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future,
  {
    Timeout {
      timeout: Timer::at(instant),
      future: fut,
    }
  }
}
