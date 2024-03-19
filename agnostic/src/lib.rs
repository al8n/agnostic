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

#[derive(Debug, PartialEq, Eq)]
pub struct Elapsed(());

impl core::fmt::Display for Elapsed {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "deadline has elapsed")
  }
}

impl std::error::Error for Elapsed {}

impl From<Elapsed> for std::io::Error {
  fn from(_: Elapsed) -> Self {
    std::io::ErrorKind::TimedOut.into()
  }
}

#[cfg(feature = "tokio")]
impl From<::tokio::time::error::Elapsed> for Elapsed {
  fn from(_: ::tokio::time::error::Elapsed) -> Self {
    Elapsed(())
  }
}

pub trait Sleep: Future<Output = std::time::Instant> + Send {
  /// Resets the Sleep instance to a new deadline.
  ///
  /// The behavior of this function may different in different runtime implementations.
  fn reset(self: std::pin::Pin<&mut Self>, deadline: Instant);
}

pub trait Interval: Stream<Item = Instant> + Send + Unpin {}

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

pub trait Timeoutable<F: Future + Send>:
  Future<Output = Result<F::Output, Elapsed>> + Send
{
  fn poll_elapsed(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<F::Output, Elapsed>>;
}

/// Runtime trait
pub trait Runtime: Sized + Unpin + Copy + Send + Sync + 'static {
  type JoinHandle<F>: Future + Send + Sync + 'static
  where
    F: Send + 'static;
  type LocalJoinHandle<F>: Future;
  type BlockJoinHandle<R>
  where
    R: Send + 'static;
  type Interval: Interval;
  type Sleep: Sleep;
  type Delay<F>: Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send;
  type Timeout<F>: Timeoutable<F>
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
    <Self as Runtime>::spawn(future);
  }

  fn spawn_local<F>(future: F) -> Self::LocalJoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static;

  fn spawn_local_detach<F>(future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self as Runtime>::spawn_local(future);
  }

  fn spawn_blocking<F, R>(f: F) -> Self::BlockJoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    <Self as Runtime>::spawn_blocking(f);
  }

  fn block_on<F: Future>(f: F) -> F::Output;

  fn interval(interval: Duration) -> Self::Interval;

  fn interval_at(start: Instant, period: Duration) -> Self::Interval;

  fn sleep(duration: Duration) -> Self::Sleep;

  fn sleep_until(instant: Instant) -> Self::Sleep;

  fn yield_now() -> impl Future<Output = ()> + Send;

  fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send + Sync + 'static;

  fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
  where
    F: Future + Send;

  fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future + Send;
}

#[cfg(any(feature = "async-std", feature = "smol"))]
mod timer {
  use super::{Interval, Runtime, Sleep};
  use std::{
    future::Future,
    io,
    task::Poll,
    time::{Duration, Instant},
  };

  pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[repr(transparent)]
    pub struct AsyncSleep {
      #[pin]
      t: async_io::Timer,
    }
  }

  impl From<async_io::Timer> for AsyncSleep {
    fn from(t: async_io::Timer) -> Self {
      Self { t }
    }
  }

  impl From<AsyncSleep> for async_io::Timer {
    fn from(s: AsyncSleep) -> Self {
      s.t
    }
  }

  impl Future for AsyncSleep {
    type Output = std::time::Instant;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
      self.project().t.poll(cx)
    }
  }

  impl Sleep for AsyncSleep {
    /// Sets the timer to emit an event once at the given time instant.
    ///
    /// Note that resetting a timer is different from creating a new sleep by [`sleep()`][`Runtime::sleep()`] because
    /// `reset()` does not remove the waker associated with the task.
    fn reset(mut self: std::pin::Pin<&mut Self>, deadline: Instant) {
      self.project().t.as_mut().set_at(deadline)
    }
  }

  impl Interval for async_io::Timer {}

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

    pub fn at(deadline: Instant, future: F) -> Self {
      Self {
        future,
        timeout: async_io::Timer::at(deadline),
      }
    }
  }

  impl<F> Future for Timeout<F>
  where
    F: Future,
  {
    type Output = Result<F::Output, super::Elapsed>;

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
        Poll::Ready(Err(super::Elapsed(())))
      } else {
        Poll::Pending
      }
    }
  }

  impl<F: Future + Send> super::Timeoutable<F> for Timeout<F> {
    fn poll_elapsed(
      self: std::pin::Pin<&mut Self>,
      cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<<F as Future>::Output, super::Elapsed>> {
      let this = self.project();
      match this.future.poll(cx) {
        Poll::Pending => {}
        other => return other.map(Ok),
      }

      if this.timeout.poll(cx).is_ready() {
        Poll::Ready(Err(super::Elapsed(())))
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
