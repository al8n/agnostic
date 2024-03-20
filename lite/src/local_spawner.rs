use core::future::Future;

/// A spawner trait for spawning futures.
pub trait AsyncLocalSpawner: Copy + 'static {
  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: Future + 'static
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
