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

/// The join handle returned by [`WasmLocalSpawner`].
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub struct WasmLocalJoinHandle<F> {
  stop_tx: futures_channel::oneshot::Sender<bool>,
  rx: futures_channel::oneshot::Receiver<F>,
}

#[cfg(feature = "wasm")]
impl<F> Future for WasmLocalJoinHandle<F> {
  type Output = Result<F, futures_channel::oneshot::Canceled>;

  fn poll(
    mut self: core::pin::Pin<&mut Self>,
    cx: &mut core::task::Context<'_>,
  ) -> core::task::Poll<Self::Output> {
    core::pin::Pin::new(&mut self.rx).poll(cx)
  }
}

#[cfg(feature = "wasm")]
impl<F> WasmLocalJoinHandle<F> {
  /// Detach the future from the spawner.
  #[inline]
  pub fn detach(self) {
    let _ = self.stop_tx.send(false);
  }

  /// Cancel the future.
  #[inline]
  pub fn cancel(self) {
    let _ = self.stop_tx.send(true);
  }
}

/// A [`AsyncLocalSpawner`] that uses the [`wasm-bindgen-futures`](wasm_bindgen_futures) runtime.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[derive(Debug, Clone, Copy)]
pub struct WasmLocalSpawner;

#[cfg(feature = "wasm")]
impl AsyncLocalSpawner for WasmLocalSpawner {
  type JoinHandle<F> = WasmLocalJoinHandle<F> where F: 'static;

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
    WasmLocalJoinHandle { stop_tx, rx }
  }
}
