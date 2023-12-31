//! Agnostic is a trait for users who want to write async runtime-agnostic crate.
#![allow(warnings)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![allow(clippy::needless_return)]
#![allow(unreachable_code)]

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

/// Network related traits
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
pub mod net;

use std::{
  future::Future,
  time::{Duration, Instant},
};

use futures_util::Stream;

pub trait Sleep: Future + Send {
  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: std::pin::Pin<&mut Self>, deadline: Instant);
}

/// Simlilar to Go's `time.AfterFunc`
pub trait Delay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self;

  fn reset(&mut self, dur: Duration) -> impl Future<Output = ()> + Send + '_;

  fn cancel(&mut self) -> impl Future<Output = Option<F::Output>> + Send + '_;
}

/// Runtime trait
pub trait Runtime: Sized + Unpin + Copy + Send + Sync + 'static {
  type JoinHandle<F>: Future;
  type Interval: Stream + Send + Unpin;
  type Sleep: Sleep;
  type Delay<F>: Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;
  type Timeout<F>: Future<Output = std::io::Result<F::Output>> + Send
  where
    F: Future + Send;

  #[cfg(feature = "net")]
  type Net: net::Net;

  fn new() -> Self;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    Self::spawn(future);
  }

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static;

  fn spawn_local_detach<F>(future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    Self::spawn_local(future);
  }

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    Self::spawn_blocking(f);
  }

  fn block_on<F: Future>(f: F) -> F::Output;

  fn interval(interval: Duration) -> Self::Interval;

  fn interval_at(start: Instant, period: Duration) -> Self::Interval;

  fn sleep(duration: Duration) -> Self::Sleep;

  fn sleep_until(instant: Instant) -> Self::Sleep;

  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send + Sync + 'static;

  fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
  where
    F: Future + Send;

  fn timeout_at<F>(instant: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future + Send;
}

#[cfg(any(feature = "async-std", feature = "smol"))]
mod timer {
  use super::{Runtime, Sleep};
  use std::{
    future::Future,
    io,
    task::Poll,
    time::{Duration, Instant},
  };

  impl Sleep for async_io::Timer {
    /// Sets the timer to emit an event once at the given time instant.
    ///
    /// Note that resetting a timer is different from creating a new sleep by [`sleep()`][`Runtime::sleep()`] because
    /// `reset()` does not remove the waker associated with the task.
    fn reset(mut self: std::pin::Pin<&mut Self>, deadline: Instant) {
      self.as_mut().set_at(deadline)
    }
  }

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

/// Traits for IO
#[cfg(feature = "io")]
#[cfg_attr(docsrs, doc(cfg(feature = "io")))]
pub mod io {
  pub use futures_util::{AsyncRead, AsyncWrite};

  #[cfg(feature = "tokio-compat")]
  #[cfg_attr(docsrs, doc(cfg(feature = "tokio-compat")))]
  pub use tokio::io::{AsyncRead as TokioAsyncRead, AsyncWrite as TokioAsyncWrite, ReadBuf};
}
