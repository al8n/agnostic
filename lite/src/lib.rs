#![doc = include_str!("../README.md")]
#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[cfg(any(feature = "std", test))]
extern crate std;

macro_rules! cfg_time_with_docsrs {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "time")]
      #[cfg_attr(docsrs, doc(cfg(feature = "time")))]
      $item
    )*
  };
}

macro_rules! cfg_time {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "time")]
      $item
    )*
  };
}

use core::future::Future;

cfg_time!(
  use std::time::{Duration, Instant};
);

cfg_time_with_docsrs!(
  /// Time related traits
  pub mod time;
);

/// Macro to conditionally compile items for `async-std` feature
#[macro_export]
macro_rules! cfg_async_std {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "async-std")]
      #[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
      $item
    )*
  }
}

/// Macro to conditionally compile items for `tokio` feature
#[macro_export]
macro_rules! cfg_tokio {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "tokio")]
      #[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
      $item
    )*
  }
}

/// Macro to conditionally compile items for `smol` feature
#[macro_export]
macro_rules! cfg_smol {
  ($($item:item)*) => {
    $(
      #[cfg(feature = "smol")]
      #[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
      $item
    )*
  }
}

/// Concrete runtime implementations based on [`tokio`] runtime.
///
/// [`tokio`]: https://docs.rs/tokio
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

/// Concrete runtime implementations based on [`async-std`] runtime.
///
/// [`async-std`]: https://docs.rs/async-std
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std;

/// Concrete runtime implementations based on [`smol`] runtime.
///
/// [`smol`]: https://docs.rs/smol
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol;

/// Concrete runtime implementations based on [`wasm-bindgen-futures`](::wasm_bindgen_futures).
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub mod wasm;

/// Time related traits concrete implementations for runtime based on [`async-io`](::async_io), e.g. [`async-std`], [`smol`].
///
/// [`smol`]: https://docs.rs/smol
/// [`async-std`]: https://docs.rs/async-std
#[cfg(feature = "async-io")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-io")))]
pub mod async_io;

mod spawner;
pub use spawner::*;

/// Yielder hints the runtime to execution back
pub trait Yielder {
  /// Yields execution back to the runtime.
  fn yield_now() -> impl Future<Output = ()> + Send;

  /// Yields execution back to the runtime.
  fn yield_now_local() -> impl Future<Output = ()>;
}

/// Runtime trait
pub trait RuntimeLite: Sized + Unpin + Copy + Send + Sync + 'static {
  /// The spawner type for this runtime
  type Spawner: AsyncSpawner;
  /// The local spawner type for this runtime
  type LocalSpawner: AsyncLocalSpawner;
  /// The blocking spawner type for this runtime
  type BlockingSpawner: AsyncBlockingSpawner;

  cfg_time_with_docsrs!(
    /// The after spawner type for this runtime
    type AfterSpawner: AsyncAfterSpawner;

    /// The local after spawner type for this runtime
    type LocalAfterSpawner: AsyncLocalAfterSpawner;

    /// The interval type for this runtime
    type Interval: time::AsyncInterval;

    /// The local interval type for this runtime
    type LocalInterval: time::AsyncLocalInterval;

    /// The sleep type for this runtime
    type Sleep: time::AsyncSleep;

    /// The local sleep type for this runtime
    type LocalSleep: time::AsyncLocalSleep;

    /// The delay type for this runtime
    type Delay<F>: time::AsyncDelay<F>
    where
      F: Future + Send;

    /// The local delay type for this runtime
    type LocalDelay<F>: time::AsyncLocalDelay<F>
    where
      F: Future;

    /// The timeout type for this runtime
    type Timeout<F>: time::AsyncTimeout<F>
    where
      F: Future + Send;

    /// The local timeout type for this runtime
    type LocalTimeout<F>: time::AsyncLocalTimeout<F>
    where
      F: Future;
  );

  /// Create a new instance of the runtime
  fn new() -> Self;

  /// Spawn a future onto the runtime
  fn spawn<F>(future: F) -> <Self::Spawner as AsyncSpawner>::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    <Self::Spawner as AsyncSpawner>::spawn(future)
  }

  /// Spawn a future onto the runtime and detach it
  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    <Self::Spawner as AsyncSpawner>::spawn_detach(future);
  }

  /// Spawn a future onto the local runtime
  fn spawn_local<F>(future: F) -> <Self::LocalSpawner as AsyncLocalSpawner>::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self::LocalSpawner as AsyncLocalSpawner>::spawn_local(future)
  }

  /// Spawn a future onto the local runtime and detach it
  fn spawn_local_detach<F>(future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    <Self::LocalSpawner as AsyncLocalSpawner>::spawn_local_detach(future)
  }

  /// Spawn a blocking function onto the runtime
  fn spawn_blocking<F, R>(f: F) -> <Self::BlockingSpawner as AsyncBlockingSpawner>::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    <Self::BlockingSpawner as AsyncBlockingSpawner>::spawn_blocking(f)
  }

  /// Spawn a blocking function onto the runtime and detach it
  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    <Self::BlockingSpawner as AsyncBlockingSpawner>::spawn_blocking_detach(f);
  }

  /// Block the current thread on the given future
  fn block_on<F: Future>(f: F) -> F::Output;

  /// Yield the current task
  fn yield_now() -> impl Future<Output = ()> + Send;

  cfg_time_with_docsrs!(
    /// Spawn a future onto the runtime and run the given future after the given duration
    fn spawn_after<F>(
      duration: core::time::Duration,
      future: F,
    ) -> <Self::AfterSpawner as AsyncAfterSpawner>::JoinHandle<F::Output>
    where
      F::Output: Send + 'static,
      F: Future + Send + 'static,
    {
      <Self::AfterSpawner as AsyncAfterSpawner>::spawn_after(duration, future)
    }

    /// Spawn a future onto the runtime and run the given future after the given instant.
    fn spawn_after_at<F>(
      at: std::time::Instant,
      future: F,
    ) -> <Self::AfterSpawner as AsyncAfterSpawner>::JoinHandle<F::Output>
    where
      F::Output: Send + 'static,
      F: Future + Send + 'static,
    {
      <Self::AfterSpawner as AsyncAfterSpawner>::spawn_after_at(at, future)
    }

    /// Like [`spawn_after`](RuntimeLite::spawn_after), but does not require the `future` to be `Send`.
    ///
    /// Spawn a future onto the runtime and run the given future after the given duration
    fn spawn_local_after<F>(
      duration: core::time::Duration,
      future: F,
    ) -> <Self::LocalAfterSpawner as AsyncLocalAfterSpawner>::JoinHandle<F::Output>
    where
      F::Output: Send + 'static,
      F: Future + Send + 'static,
    {
      <Self::LocalAfterSpawner as AsyncLocalAfterSpawner>::spawn_local_after(duration, future)
    }

    /// Like [`spawn_after_at`](RuntimeLite::spawn_after_at), but does not require the `future` to be `Send`.
    ///
    /// Spawn a future onto the runtime and run the given future after the given instant.
    fn spawn_local_after_at<F>(
      at: std::time::Instant,
      future: F,
    ) -> <Self::LocalAfterSpawner as AsyncLocalAfterSpawner>::JoinHandle<F::Output>
    where
      F::Output: Send + 'static,
      F: Future + Send + 'static,
    {
      <Self::LocalAfterSpawner as AsyncLocalAfterSpawner>::spawn_local_after_at(at, future)
    }

    /// Create a new interval that starts at the current time and
    /// yields every `period` duration
    fn interval(interval: Duration) -> Self::Interval;

    /// Create a new interval that starts at the given instant and
    /// yields every `period` duration
    fn interval_at(start: Instant, period: Duration) -> Self::Interval;

    /// Create a new interval that starts at the current time and
    /// yields every `period` duration
    fn interval_local(interval: Duration) -> Self::LocalInterval;

    /// Create a new interval that starts at the given instant and
    /// yields every `period` duration
    fn interval_local_at(start: Instant, period: Duration) -> Self::LocalInterval;

    /// Create a new sleep future that completes after the given duration
    /// has elapsed
    fn sleep(duration: Duration) -> Self::Sleep;

    /// Create a new sleep future that completes at the given instant
    /// has elapsed
    fn sleep_until(instant: Instant) -> Self::Sleep;

    /// Create a new sleep future that completes after the given duration
    /// has elapsed
    fn sleep_local(duration: Duration) -> Self::LocalSleep;

    /// Create a new sleep future that completes at the given instant
    /// has elapsed
    fn sleep_local_until(instant: Instant) -> Self::LocalSleep;

    /// Create a new delay future that runs the `fut` after the given duration
    /// has elapsed. The `Future` will never be polled until the duration has
    /// elapsed.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn delay<F>(duration: Duration, fut: F) -> Self::Delay<F>
    where
      F: Future + Send;

    /// Like [`delay`](RuntimeLite::delay), but does not require the `fut` to be `Send`.
    /// Create a new delay future that runs the `fut` after the given duration
    /// has elapsed. The `Future` will never be polled until the duration has
    /// elapsed.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
    where
      F: Future;

    /// Create a new timeout future that runs the `future` after the given deadline.
    /// The `Future` will never be polled until the deadline has reached.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
    where
      F: Future + Send;

    /// Like [`delay_at`](RuntimeLite::delay_at), but does not require the `fut` to be `Send`.
    /// Create a new timeout future that runs the `future` after the given deadline
    /// The `Future` will never be polled until the deadline has reached.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
    where
      F: Future;

    /// Requires a `Future` to complete before the specified duration has elapsed.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn timeout<F>(duration: Duration, future: F) -> Self::Timeout<F>
    where
      F: Future + Send;

    /// Requires a `Future` to complete before the specified instant in time.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
    where
      F: Future + Send;

    /// Like [`timeout`](RuntimeLite::timeout), but does not requrie the `future` to be `Send`.
    /// Requires a `Future` to complete before the specified duration has elapsed.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
    where
      F: Future;

    /// Like [`timeout_at`](RuntimeLite::timeout_at), but does not requrie the `future` to be `Send`.
    /// Requires a `Future` to complete before the specified duration has elapsed.
    ///
    /// The behavior of this function may different in different runtime implementations.
    fn timeout_local_at<F>(deadline: Instant, future: F) -> Self::LocalTimeout<F>
    where
      F: Future;
  );
}

/// Unit test for the [`RuntimeLite`]
#[cfg(any(test, feature = "test"))]
#[cfg_attr(docsrs, doc(cfg(any(test, feature = "test"))))]
pub mod tests {
  use core::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::Arc;
  use std::time::Duration;

  use super::{AfterHandle, RuntimeLite};

  /// Unit test for the [`RuntimeLite::spawn_after`] function
  pub async fn spawn_after_unittest<R: RuntimeLite>() {
    let ctr = Arc::new(AtomicUsize::new(1));
    let ctr1 = ctr.clone();
    let handle = R::spawn_after(Duration::from_secs(1), async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });

    R::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);

    handle.await.unwrap();
    assert_eq!(ctr.load(Ordering::SeqCst), 2);
  }

  /// Unit test for the [`RuntimeLite::spawn_after`] function
  ///
  /// The task will be canceled before it completes
  pub async fn spawn_after_cancel_unittest<R: RuntimeLite>() {
    let ctr = Arc::new(AtomicUsize::new(1));
    let ctr1 = ctr.clone();
    let handle = R::spawn_after(Duration::from_secs(1), async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });

    R::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);

    let o = handle.cancel().await;
    assert!(o.is_none());
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }

  /// Unit test for the [`RuntimeLite::spawn_after`] function
  ///
  /// The [`AfterHandle`] will be dropped immediately after it is created
  pub async fn spawn_after_drop_unittest<R: RuntimeLite>() {
    let ctr = Arc::new(AtomicUsize::new(1));
    let ctr1 = ctr.clone();
    drop(R::spawn_after(Duration::from_secs(1), async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    }));

    R::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);

    R::sleep(Duration::from_millis(600)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 2);
  }

  /// Unit test for the [`RuntimeLite::spawn_after`] function
  ///
  /// The [`AfterHandle`] will be abort after it is created, and the task will not be executed.
  pub async fn spawn_after_abort_unittest<R: RuntimeLite>() {
    let ctr = Arc::new(AtomicUsize::new(1));
    let ctr1 = ctr.clone();
    let handle = R::spawn_after(Duration::from_secs(1), async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });

    R::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);

    handle.abort();
    R::sleep(Duration::from_millis(600)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }

  /// Unit test for the [`RuntimeLite::spawn_after`] function
  ///
  /// The [`AfterHandle`] will be reset to passed than the original duration after it is created, and the task will be executed after the reset duration.
  pub async fn spawn_after_reset_to_pass_unittest<R: RuntimeLite>() {
    let ctr = Arc::new(AtomicUsize::new(1));
    let ctr1 = ctr.clone();
    let handle = R::spawn_after(Duration::from_secs(1), async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });

    R::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);

    handle.reset(Duration::from_millis(250));
    R::sleep(Duration::from_millis(10)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 2);
  }

  /// Unit test for the [`RuntimeLite::spawn_after`] function
  ///
  /// The [`AfterHandle`] will be reset to future than the original duration after it is created, and the task will be executed after the reset duration.
  pub async fn spawn_after_reset_to_future_unittest<R: RuntimeLite>() {
    let ctr = Arc::new(AtomicUsize::new(1));
    let ctr1 = ctr.clone();
    let handle = R::spawn_after(Duration::from_secs(1), async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });

    R::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctr.load(Ordering::SeqCst), 1);

    handle.reset(Duration::from_millis(1250)); // now delay 1.25s
    R::sleep(Duration::from_millis(750 + 10)).await; // we already delayed 500ms, so remaining is 750ms
    assert_eq!(ctr.load(Ordering::SeqCst), 2);
  }
}
