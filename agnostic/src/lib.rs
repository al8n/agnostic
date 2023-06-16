//! Agnostic is a trait for users who want to write async runtime-agnostic crate.
#![deny(warnings)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::needless_return)]

#[cfg(all(feature = "compat", not(feature = "net")))]
compile_error!("`compat` feature is enabled, but `net` feature is disabled, `compact` feature must only be enabled with `net` feature");

#[macro_use]
mod macros;

/// [`tokio`] runtime adapter
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

/// [`async_std`] runtime adapter
///
/// [`async_std`]: https://docs.rs/async-std
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std;

/// [`smol`] runtime adapter
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol;

// /// [`monoio`] runtime adapter
// ///
// /// [`monoio`]: https://docs.rs/monoio
// #[cfg(not(all(feature = "monoio", feature = "net")))]
// #[cfg_attr(docsrs, doc(cfg(feature = "monoio")))]
// pub mod monoio;

#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

use std::{
  future::Future,
  time::{Duration, Instant},
};

use futures_util::Stream;

/// Simlilar to Go's `time.AfterFunc`
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

/// Runtime trait
pub trait Runtime: Send + Sync + 'static {
  type JoinHandle<F>: Future;
  type Interval: Stream;
  type Sleep: Future;
  type Delay<F>: Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;
  type Timeout<F>: Future
  where
    F: Future;
  #[cfg(any(
    feature = "tokio-net",
    feature = "async-std-net",
    feature = "smol-net",
    feature = "wasm-net"
  ))]
  type Net: net::Net;

  fn new() -> Self;

  fn spawn<F>(&self, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  fn spawn_detach<F>(&self, future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    self.spawn(future);
  }

  fn spawn_local<F>(&self, future: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static;

  fn spawn_local_detach<F>(&self, future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    self.spawn_local(future);
  }

  fn spawn_blocking<F, R>(&self, f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  fn spawn_blocking_detach<F, R>(&self, f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    self.spawn_blocking(f);
  }

  fn interval(&self, interval: Duration) -> Self::Interval;

  fn interval_at(&self, start: Instant, period: Duration) -> Self::Interval;

  fn sleep(&self, duration: Duration) -> Self::Sleep;

  fn sleep_until(&self, instant: Instant) -> Self::Sleep;

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
    #[cfg_attr(docsrs, doc(cfg(any(feature = "async-std", feature = "smol"))))]
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
