#![forbid(unsafe_code)]
#![deny(warnings)]

use std::{
  future::Future,
  time::{Duration, Instant},
};

use futures_util::Stream;

#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "async-std")]
pub mod async_std;

#[cfg(feature = "smol")]
pub mod smol;

#[cfg(feature = "monoio")]
pub mod monoio;

#[async_trait::async_trait]
pub trait Delay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self;

  async fn reset(&mut self, dur: Duration);

  async fn cancel(&mut self) -> Option<F::Output>;
}

#[async_trait::async_trait]
pub trait Runtime {
  type JoinHandle<T>: Future;
  type Interval: Stream;
  type Sleep: Future;
  type Delay<F>: Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;
  type Timeout<F>: Future
  where
    F: Future;

  fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  /// Returns `true` if the spawned thread will be auto detached.
  ///
  /// Some async runtimes' handle (e.g. [`smol::Task<T>`]) will not auto detach the spawned thread,
  /// which means that once the handle of spawn functions is dropped, the future will be canceled.
  /// Hence, if the runtime this `detach_spawn` method is used to let users check if such runtime will
  /// auto detach the spawned thread. If not, users need to manually detach the spawned thread themselves.
  ///
  /// [`smol::Task<T>`]: https://docs.rs/smol/latest/smol/struct.Task.html
  fn detach_spawn(&self) -> bool {
    true
  }

  fn interval(&self, interval: Duration) -> Self::Interval;

  fn interval_at(&self, start: Instant, period: Duration) -> Self::Interval;

  fn sleep(&self, duration: Duration) -> Self::Sleep;

  fn delay<F>(&self, duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send + Sync + 'static;

  fn timeout<F>(&self, duration: Duration, future: F) -> Self::Timeout<F>
  where
    F: Future;

  fn timeout_at<F>(&self, instant: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future;
}

#[cfg(any(feature = "async-std", feature = "smol"))]
mod timer {
  use std::{future::Future, io, task::Poll, time::Duration};

  pin_project_lite::pin_project! {
    /// Future returned by the `FutureExt::timeout` method.
    #[derive(Debug)]
    pub struct Timeout<F>
    where
        F: Future,
    {
      #[pin]
      pub(crate) future: F,
      #[pin]
      pub(crate) timeout: async_io::Timer,
    }
  }

  impl<F> Timeout<F>
  where
    F: Future,
  {
    pub fn new(timeout: Duration, future: F) -> Self {
      Self {
        future,
        timeout: async_io::Timer::after(timeout),
      }
    }
  }

  impl<F> Future for Timeout<F>
  where
    F: Future,
  {
    type Output = io::Result<F::Output>;

    fn poll(
      self: std::pin::Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
      let this = self.project();
      match this.future.poll(cx) {
        Poll::Pending => {}
        other => return other.map(Ok),
      }

      if this.timeout.poll(cx).is_ready() {
        let err = Err(io::Error::new(io::ErrorKind::TimedOut, "future timed out"));
        Poll::Ready(err)
      } else {
        Poll::Pending
      }
    }
  }
}
