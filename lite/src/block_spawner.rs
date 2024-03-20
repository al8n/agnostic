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

#[cfg(feature = "tokio")]
impl AsyncBlockingSpawner for super::TokioSpawner {
  type JoinHandle<R> = tokio::task::JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(_f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    #[cfg(not(target_family = "wasm"))]
    {
      ::tokio::task::spawn_blocking(_f)
    }

    #[cfg(target_family = "wasm")]
    {
      panic!("TokioRuntime::spawn_blocking is not supported on wasm")
    }
  }
}

#[cfg(feature = "async-std")]
impl AsyncBlockingSpawner for super::AsyncStdSpawner {
  type JoinHandle<R> = async_std::task::JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    async_std::task::spawn_blocking(f)
  }
}

#[cfg(feature = "smol")]
impl AsyncBlockingSpawner for super::SmolSpawner {
  type JoinHandle<R> = smol::Task<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::unblock(f)
  }

  fn spawn_blocking_detach<F, R>(f: F)
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    ::smol::unblock(f).detach()
  }
}

#[cfg(feature = "wasm")]
impl AsyncBlockingSpawner for super::WasmSpawner {
  type JoinHandle<R> = std::thread::JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    std::thread::spawn(f)
  }
}
