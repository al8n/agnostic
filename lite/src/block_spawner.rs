/// A spawner trait for spawning blocking.
pub trait AsyncBlockingSpawner: Copy + 'static {
  /// The join handle type for blocking tasks
  type JoinHandle<R>
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
