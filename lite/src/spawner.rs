use core::future::Future;


/// A spawner trait for spawning futures.
pub trait AsyncSpawner: Copy + Send + Sync + 'static {
  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: Future + Send + Sync + 'static
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

/// A [`AsyncSpawner`] that uses the [`tokio`] runtime.
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[derive(Debug, Clone, Copy)]
pub struct TokioSpawner;

#[cfg(feature = "tokio")]
impl AsyncSpawner for TokioSpawner {
  type JoinHandle<F> = tokio::task::JoinHandle<F> where
  F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    tokio::task::spawn(future)
  }
}

/// A [`AsyncSpawner`] that uses the [`async-std`](async_std) runtime.
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
#[derive(Debug, Clone, Copy)]
pub struct AsyncStdSpawner;

#[cfg(feature = "async-std")]
impl AsyncSpawner for AsyncStdSpawner {
  type JoinHandle<F> = async_std::task::JoinHandle<F> where F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    async_std::task::spawn(future)
  }
}

/// A [`AsyncSpawner`] that uses the [`smol`] runtime.
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
#[derive(Debug, Clone, Copy)]
pub struct SmolSpawner;

#[cfg(feature = "smol")]
impl AsyncSpawner for SmolSpawner {
  type JoinHandle<F> = smol::Task<F> where F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: core::future::Future + Send + 'static,
  {
    smol::spawn(future)
  }

  fn spawn_detach<F>(future: F)
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    smol::spawn(future).detach()
  }
}
