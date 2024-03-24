use core::future::Future;

/// Detaches the task related to the join handle to let it keep running in the background.
pub trait Detach: Sized {
  /// Detaches the task to let it keep running in the background.
  fn detach(self) {
    drop(self)
  }
}

#[cfg(any(feature = "std", test))]
impl<T> Detach for std::thread::JoinHandle<T> {}

/// A spawner trait for spawning futures.
pub trait AsyncSpawner: Copy + Send + Sync + 'static {
  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: Detach + Future + Send + Sync + 'static
  where
    F: Send + 'static;

  /// Spawn a future.
  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  /// Spawn a future and detach it.
  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    core::mem::drop(Self::spawn(future));
  }
}

/// A spawner trait for spawning futures.
pub trait AsyncLocalSpawner: Copy + 'static {
  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: Detach + Future + 'static
  where
    F: 'static;

  /// Spawn a future.
  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static;

  /// Spawn a future and detach it.
  fn spawn_local_detach<F>(future: F)
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    core::mem::drop(Self::spawn_local(future));
  }
}

/// A spawner trait for spawning blocking.
pub trait AsyncBlockingSpawner: Copy + 'static {
  /// The join handle type for blocking tasks
  type JoinHandle<R>: Detach
  where
    R: Send + 'static;

  /// Spawn a blocking function onto the runtime
  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static;

  /// Spawn a blocking function onto the runtime and detach it
  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    Self::spawn_blocking(f);
  }
}

/// Canceled
#[derive(Debug, Clone, Copy)]
#[cfg(all(
  feature = "time",
  any(
    feature = "async-std",
    feature = "tokio",
    feature = "smol",
    feature = "wasm"
  )
))]
pub(crate) struct Canceled;

#[cfg(all(
  feature = "time",
  any(
    feature = "async-std",
    feature = "tokio",
    feature = "smol",
    feature = "wasm"
  )
))]
impl core::fmt::Display for Canceled {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "after canceled")
  }
}

#[cfg(all(
  feature = "time",
  any(
    feature = "async-std",
    feature = "tokio",
    feature = "smol",
    feature = "wasm"
  )
))]
impl std::error::Error for Canceled {}

/// Error of [`AfterHandle`]'s output
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
#[derive(Debug)]
pub enum AfterHandleError<E> {
  /// The after function was canceled
  Canceled,
  /// Task failed to execute to completion.
  Join(E),
}

#[cfg(feature = "time")]
impl<E: core::fmt::Display> core::fmt::Display for AfterHandleError<E> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Canceled => write!(f, "after function was canceled"),
      Self::Join(e) => write!(f, "{e}"),
    }
  }
}

#[cfg(feature = "time")]
impl<E: core::fmt::Debug + core::fmt::Display> std::error::Error for AfterHandleError<E> {}

#[cfg(all(
  feature = "time",
  any(
    feature = "async-std",
    feature = "tokio",
    feature = "smol",
    feature = "wasm"
  )
))]
pub(crate) struct AfterHandleSignals {
  finished: core::sync::atomic::AtomicBool,
  expired: core::sync::atomic::AtomicBool,
}

#[cfg(all(
  feature = "time",
  any(
    feature = "async-std",
    feature = "tokio",
    feature = "smol",
    feature = "wasm"
  )
))]
impl AfterHandleSignals {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self {
      finished: core::sync::atomic::AtomicBool::new(false),
      expired: core::sync::atomic::AtomicBool::new(false),
    }
  }

  #[inline]
  pub(crate) fn set_finished(&self) {
    self
      .finished
      .store(true, core::sync::atomic::Ordering::Release);
  }

  #[inline]
  pub(crate) fn set_expired(&self) {
    self
      .expired
      .store(true, core::sync::atomic::Ordering::Release);
  }

  #[inline]
  pub(crate) fn is_finished(&self) -> bool {
    self.finished.load(core::sync::atomic::Ordering::Acquire)
  }

  #[inline]
  pub(crate) fn is_expired(&self) -> bool {
    self.expired.load(core::sync::atomic::Ordering::Acquire)
  }
}

/// The handle returned by the [`AsyncAfterSpawner`] when a after future is spawned.
///
/// Drop the handle to detach the task.
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub trait AfterHandle<F: Send + 'static, E: Send>:
  Send + Sync + Detach + Future<Output = Result<F, AfterHandleError<E>>> + 'static
{
  /// Cancels the task related to this handle.
  ///
  /// Returns the task’s output if it was completed just before it got canceled, or `None` if it didn’t complete.
  fn cancel(self) -> impl Future<Output = Option<Result<F, AfterHandleError<E>>>> + Send;

  /// Aborts the task related to this handle.
  fn abort(self);

  /// Returns `true` if the timer has expired.
  fn is_expired(&self) -> bool;

  /// Returns `true` if the task has finished.
  fn is_finished(&self) -> bool;
}

/// A spawner trait for spawning futures. Go's `time.AfterFunc` equivalent.
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub trait AsyncAfterSpawner: Copy + Send + Sync + 'static {
  /// The join error type for the join handle
  type JoinError: core::fmt::Debug + core::fmt::Display + Send + 'static;

  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: AfterHandle<F, Self::JoinError>
  where
    F: Send + 'static;

  /// Spawn a future onto the runtime and run the given future after the given duration
  fn spawn_after<F>(duration: core::time::Duration, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  /// Spawn and detach a future onto the runtime and run the given future after the given duration
  fn spawn_after_detach<F>(duration: core::time::Duration, future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    core::mem::drop(Self::spawn_after(duration, future));
  }

  /// Spawn a future onto the runtime and run the given future after reach the given instant
  fn spawn_after_at<F>(instant: std::time::Instant, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static;

  /// Spawn and detach a future onto the runtime and run the given future after reach the given instant
  fn spawn_after_at_detach<F>(instant: std::time::Instant, future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    Self::spawn_after_at(instant, future).detach()
  }
}

/// The handle returned by the [`AsyncLocalAfterSpawner`] when a after future is spawned.
///
/// Drop the handle to detach the task.
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub trait LocalAfterHandle<F: 'static, E>:
  Detach + Future<Output = Result<F, AfterHandleError<E>>> + 'static
{
  /// Cancels the task related to this handle.
  ///
  /// Returns the task’s output if it was completed just before it got canceled, or `None` if it didn’t complete.
  fn cancel(self) -> impl Future<Output = Option<Result<F, AfterHandleError<E>>>>;

  /// Aborts the task related to this handle.
  fn abort(self);

  /// Returns `true` if the timer has expired.
  fn is_expired(&self) -> bool;

  /// Returns `true` if the task has finished.
  fn is_finished(&self) -> bool;
}

/// A spawner trait for spawning futures locally. Go's `time.AfterFunc` equivalent.
#[cfg(feature = "time")]
#[cfg_attr(docsrs, doc(cfg(feature = "time")))]
pub trait AsyncLocalAfterSpawner: Copy + 'static {
  /// The join error type for the join handle
  type JoinError: core::fmt::Debug + core::fmt::Display + 'static;
  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: LocalAfterHandle<F, Self::JoinError>
  where
    F: 'static;

  /// Spawn a future onto the runtime and run the given future after the given duration
  fn spawn_local_after<F>(duration: core::time::Duration, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static;

  /// Spawn and detach a future onto the runtime and run the given future after the given duration
  fn spawn_local_after_detach<F>(duration: core::time::Duration, future: F)
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    Self::spawn_local_after(duration, future).detach()
  }

  /// Spawn a future onto the runtime and run the given future after reach the given instant
  fn spawn_local_after_at<F>(instant: std::time::Instant, future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static;

  /// Spawn and detach a future onto the runtime and run the given future after reach the given instant
  fn spawn_local_after_at_detach<F>(instant: std::time::Instant, future: F)
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    Self::spawn_local_after_at(instant, future).detach()
  }
}
