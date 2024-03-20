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

#[cfg(feature = "tokio")]
impl AsyncLocalSpawner for super::TokioSpawner {
  type JoinHandle<F> = tokio::task::JoinHandle<F> where
  F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    tokio::task::spawn_local(future)
  }
}

#[cfg(feature = "async-std")]
impl AsyncLocalSpawner for super::AsyncStdSpawner {
  type JoinHandle<F> = async_std::task::JoinHandle<F> where F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    async_std::task::spawn_local(future)
  }
}

#[cfg(feature = "smol")]
impl AsyncLocalSpawner for super::SmolSpawner {
  type JoinHandle<F> = smol::Task<F> where F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    smol::LocalExecutor::new().spawn(future)
  }

  fn spawn_local_detach<F>(future: F)
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    smol::LocalExecutor::new().spawn(future).detach();
  }
}

#[cfg(feature = "wasm")]
impl AsyncLocalSpawner for super::WasmSpawner {
  type JoinHandle<F> = super::WasmJoinHandle<F> where F: 'static;

  fn spawn_local<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    use futures_util::FutureExt;

    let (tx, rx) = futures_channel::oneshot::channel();
    let (stop_tx, stop_rx) = futures_channel::oneshot::channel();
    wasm_bindgen_futures::spawn_local(async {
      futures_util::pin_mut!(future);

      futures_util::select! {
        sig = stop_rx.fuse() => {
          match sig {
            Ok(true) => {
              // if we receive a stop signal, we just stop this task.
            },
            Ok(false) | Err(_) => {
              let _ = future.await;
            },
          }
        },
        future = (&mut future).fuse() => {
          let _ = tx.send(future);
        }
      }
    });
    super::WasmJoinHandle { stop_tx, rx }
  }
}
