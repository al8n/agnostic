use core::future::Future;

/// A spawner trait for spawning futures.
pub trait AsyncLocalSpawner: Copy + 'static {
  /// The handle returned by the spawner when a future is spawned.
  type JoinHandle<F>: Future + 'static
  where
    F: 'static;

  /// Spawn a future.
  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static;

  /// Spawn a future and detach it.
  fn spawn_detach<F>(future: F)
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    core::mem::drop(Self::spawn(future));
  }
}

/// A [`AsyncLocalSpawner`] that uses the [`tokio`] runtime.
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
#[derive(Debug, Clone, Copy)]
pub struct TokioLocalSpawner;

#[cfg(feature = "tokio")]
impl AsyncLocalSpawner for TokioLocalSpawner {
  type JoinHandle<F> = tokio::task::JoinHandle<F> where
  F: 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    tokio::task::spawn_local(future)
  }
}

/// A [`AsyncLocalSpawner`] that uses the [`async-std`](async_std) runtime.
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
#[derive(Debug, Clone, Copy)]
pub struct AsyncStdLocalSpawner;

#[cfg(feature = "async-std")]
impl AsyncLocalSpawner for AsyncStdLocalSpawner {
  type JoinHandle<F> = async_std::task::JoinHandle<F> where F: 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    async_std::task::spawn_local(future)
  }
}

/// A [`AsyncLocalSpawner`] that uses the [`smol`] runtime.
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
#[derive(Debug, Clone, Copy)]
pub struct SmolLocalSpawner;

#[cfg(feature = "smol")]
impl AsyncLocalSpawner for SmolLocalSpawner {
  type JoinHandle<F> = smol::Task<F> where F: 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: core::future::Future + 'static,
  {
    smol::LocalExecutor::new().spawn(future)
  }

  fn spawn_detach<F>(future: F)
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

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
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
