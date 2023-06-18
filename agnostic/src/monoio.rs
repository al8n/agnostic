use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use super::*;

use async_channel as channel;
use futures_util::FutureExt;

struct MonoioDelayHandle<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  // TODO: monoio
  // handle: ::monoio::task::JoinHandle<Option<F::Output>>,
  reset_tx: channel::Sender<Duration>,
  stop_tx: channel::Sender<()>,
  finished: Arc<AtomicBool>,
  marker: std::marker::PhantomData<F>,
}

pub struct MonoioDelay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  handle: Option<MonoioDelayHandle<F>>,
}

#[cfg_attr(not(feature = "nightly"), async_trait::async_trait)]
impl<F> Delay<F> for MonoioDelay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self {
    let (reset_tx, reset_rx) = channel::bounded::<Duration>(1);
    let (stop_tx, stop_rx) = channel::bounded::<()>(1);
    let finished = Arc::new(AtomicBool::new(false));
    let ff = finished.clone();
    let _handle = ::monoio::spawn(async move {
      let mut sleep = ::monoio::time::sleep(delay);
      loop {
        futures_util::select! {
          _ = sleep.fuse() => {
            let rst = fut.await;
            finished.store(true, Ordering::SeqCst);
            return Some(rst);
          }
          _ = stop_rx.recv().fuse() => {
            return None;
          }
          remaining = reset_rx.recv().fuse() => {
            if let Ok(remaining) = remaining {
              sleep = ::monoio::time::sleep(remaining);
            } else {
              return None;
            }
          }
        }
      }
    });
    Self {
      handle: Some(MonoioDelayHandle {
        reset_tx,
        stop_tx,
        finished: ff,
        // handle,
        marker: std::marker::PhantomData,
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
        return None;
      } else {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.stop_tx.send(()).await;
      }
    }
    None
  }
}

#[derive(Debug, Copy, Clone)]
pub struct MonoioRuntime;

impl core::fmt::Display for MonoioRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "monoio")
  }
}

#[cfg(not(all(feature = "monoio", feature = "net")))]
impl Runtime for MonoioRuntime {
  type JoinHandle<T> = ::monoio::task::JoinHandle<T>;
  type Interval = IntervalStream;
  type Sleep = ::monoio::time::Sleep;
  type Delay<F> = MonoioDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = ::monoio::time::Timeout<F> where F: Future;

  fn new() -> Self {
    Self
  }

  fn spawn<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    ::monoio::spawn(fut)
  }

  fn spawn_local<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    ::monoio::spawn(fut)
  }

  fn spawn_blocking<F, R>(&self, _f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    panic!("MonoioRuntime does not support spawn blocking because it has different spwan_blocking fn signature, please use spawn_blocking_detach instead")
  }

  fn spawn_blocking_detach<F, R>(&self, f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::monoio::spawn_blocking(f);
  }

  fn interval(&self, interval: Duration) -> Self::Interval {
    ::monoio::time::interval(interval).into()
  }

  fn interval_at(&self, start: Instant, period: Duration) -> Self::Interval {
    ::monoio::time::interval_at(start.into(), period).into()
  }

  fn sleep(&self, duration: Duration) -> Self::Sleep {
    ::monoio::time::sleep(duration)
  }

  fn sleep_until(&self, deadline: Instant) -> Self::Sleep {
    ::monoio::time::sleep_until(deadline.into())
  }

  fn delay<F>(&self, delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send + Sync + 'static,
  {
    MonoioDelay::new(delay, fut)
  }

  fn timeout<F>(&self, duration: Duration, fut: F) -> Self::Timeout<F>
  where
    F: Future,
  {
    ::monoio::time::timeout(duration, fut)
  }

  fn timeout_at<F>(&self, instant: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future,
  {
    ::monoio::time::timeout_at(instant.into(), fut)
  }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct IntervalStream {
  interval: ::monoio::time::Interval,
}

impl From<::monoio::time::Interval> for IntervalStream {
  fn from(interval: ::monoio::time::Interval) -> Self {
    Self { interval }
  }
}

impl Stream for IntervalStream {
  type Item = Instant;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    self.interval.poll_tick(cx).map(|x| Some(x.into()))
  }
}
