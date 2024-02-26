use std::{io, task::Poll};

use ::tokio::sync::mpsc;
use tokio_stream::wrappers::IntervalStream;

use super::*;

#[cfg(feature = "net")]
pub mod net;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimeoutError;

impl core::fmt::Display for TimeoutError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "timeout")
  }
}

impl std::error::Error for TimeoutError {}

impl super::Sleep for ::tokio::time::Sleep {
  /// Resets the `Sleep` instance to a new deadline.
  ///
  /// Calling this function allows changing the instant at which the `Sleep`
  /// future completes without having to create new associated state.
  ///
  /// This function can be called both before and after the future has
  /// completed.
  ///
  /// To call this method, you will usually combine the call with
  /// [`Pin::as_mut`], which lets you call the method without consuming the
  /// `Sleep` itself.
  ///
  /// # Example
  ///
  /// ```
  /// use std::time::{Duration, Instant};
  /// use agnostic::tokio::TokioRuntime;
  ///
  /// # #[tokio::main(flavor = "current_thread")]
  /// # async fn main() {
  /// let sleep = TokioRuntime::sleep(Duration::from_millis(10));
  /// tokio::pin!(sleep);
  ///
  /// sleep.as_mut().reset(Instant::now() + Duration::from_millis(20));
  /// # }
  /// ```
  ///
  /// [`Pin::as_mut`]: fn@std::pin::Pin::as_mut
  fn reset(mut self: std::pin::Pin<&mut Self>, deadline: Instant) {
    self.as_mut().reset(deadline.into())
  }
}

struct DelayFuncHandle<F: Future> {
  handle: ::tokio::task::JoinHandle<Option<F::Output>>,
  reset_tx: mpsc::Sender<Duration>,
  stop_tx: mpsc::Sender<()>,
}

pub struct TokioDelay<F: Future> {
  handle: Option<DelayFuncHandle<F>>,
}

impl<F> Delay<F> for TokioDelay<F>
where
  F: Future + Send + 'static,
  F::Output: Send,
{
  fn new(delay: Duration, fut: F) -> Self {
    let (stop_tx, mut stop_rx) = mpsc::channel(1);
    let (reset_tx, mut reset_rx) = mpsc::channel(1);
    let handle = ::tokio::spawn(async move {
      let sleep = ::tokio::time::sleep(delay);
      ::tokio::pin!(sleep);
      loop {
        ::tokio::select! {
          _ = &mut sleep => {
            return Some(fut.await);
          },
          _ = stop_rx.recv() => return None,
          remaining = reset_rx.recv() => {
            if let Some(remaining) = remaining {
              sleep.as_mut().reset(::tokio::time::Instant::now() + remaining);
            } else {
              return None;
            }
          }
        }
      }
    });
    Self {
      handle: Some(DelayFuncHandle {
        reset_tx,
        handle,
        stop_tx,
      }),
    }
  }

  fn reset(&mut self, dur: Duration) -> impl Future<Output = ()> + Send + '_ {
    async move {
      if let Some(handle) = &mut self.handle {
        // if we fail to send a message, which means the rx has been dropped, and that thread has exited
        let _ = handle.reset_tx.send(dur).await;
      }
    }
  }

  fn cancel(&mut self) -> impl Future<Output = Option<F::Output>> + Send + '_ {
    async move {
      if let Some(handle) = self.handle.take() {
        if handle.handle.is_finished() {
          return match handle.handle.await {
            Ok(rst) => rst,
            Err(_) => None,
          };
        } else {
          // if we fail to send a message, which means the rx has been dropped, and that thread has exited
          let _ = handle.stop_tx.send(()).await;
          return None;
        }
      }
      None
    }
  }
}

impl<F: Future> Timeoutable for ::tokio::time::Timeout<F> {
  fn poll_elapsed(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<F::Output, Elapsed>> {
    match self.poll(cx) {
      Poll::Ready(Ok(rst)) => Poll::Ready(Ok(rst)),
      Poll::Ready(Err(_)) => Poll::Ready(Err(Elapsed)),
      Poll::Pending => Poll::Pending,
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct TokioRuntime;

impl core::fmt::Display for TokioRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "tokio")
  }
}

impl Runtime for TokioRuntime {
  type JoinHandle<T> = ::tokio::task::JoinHandle<T>
  where
    T: Send + 'static,
    <Self::JoinHandle<T> as Future>::Output: Send;
  type BlockJoinHandle<R>
  where
    R: Send + 'static,
  = ::tokio::task::JoinHandle<R>;
  type LocalJoinHandle<F> = ::tokio::task::JoinHandle<F>;
  type Interval = IntervalStream;
  type Sleep = ::tokio::time::Sleep;
  type Delay<F> = TokioDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = ::tokio::time::Timeout<F> where F: Future + Send;

  #[cfg(feature = "net")]
  type Net = self::net::TokioNet;

  fn new() -> Self {
    Self
  }

  fn spawn<F>(fut: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    ::tokio::spawn(fut)
  }

  fn spawn_local<F>(fut: F) -> Self::LocalJoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    ::tokio::task::spawn_local(fut)
  }

  fn spawn_blocking<F, R>(_f: F) -> Self::BlockJoinHandle<R>
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

  fn block_on<F: Future>(f: F) -> F::Output {
    ::tokio::runtime::Handle::current().block_on(f)
  }

  fn interval(interval: Duration) -> Self::Interval {
    IntervalStream::new(::tokio::time::interval(interval))
  }

  fn interval_at(start: Instant, period: Duration) -> Self::Interval {
    IntervalStream::new(::tokio::time::interval_at(start.into(), period))
  }

  fn sleep(duration: Duration) -> Self::Sleep {
    ::tokio::time::sleep(duration)
  }

  fn sleep_until(instant: Instant) -> Self::Sleep {
    ::tokio::time::sleep_until(instant.into())
  }

  fn yield_now() -> impl Future<Output = ()> + Send {
    ::tokio::task::yield_now()
  }

  fn delay<F>(delay: Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send + 'static,
    F::Output: Send,
  {
    TokioDelay::new(delay, fut)
  }

  fn timeout<F>(duration: Duration, fut: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    ::tokio::time::timeout(duration, fut)
  }

  fn timeout_at<F>(deadline: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    ::tokio::time::timeout_at(deadline.into(), fut)
  }
}

/// A waitable spawner, when spawning, a wait group is incremented,
/// and when the spawned task is finished, the wait group is decremented.
pub struct TokioWaitGroup {
  wg: wg::tokio::AsyncWaitGroup,
}

impl Clone for TokioWaitGroup {
  fn clone(&self) -> Self {
    Self {
      wg: self.wg.clone(),
    }
  }
}

impl TokioWaitGroup {
  /// Creates a new `WaitableSpawner`
  pub(crate) fn new() -> Self {
    Self {
      wg: wg::tokio::AsyncWaitGroup::new(),
    }
  }
}

impl WaitGroup for TokioWaitGroup {
  type Runtime = TokioRuntime;
  type Wait<'a> = wg::tokio::WaitGroupFuture<'a>;

  /// Spawns a future and increments the wait group
  fn spawn<F>(&self, future: F) -> <Self::Runtime as Runtime>::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    let wg = self.wg.add(1);
    <Self::Runtime as Runtime>::spawn(async move {
      let res = future.await;
      wg.done();
      res
    })
  }

  fn spawn_detach(&self, future: impl Future<Output = ()> + Send + 'static) {
    let wg = self.wg.add(1);
    <Self::Runtime as Runtime>::spawn_detach(async move {
      let res = future.await;
      wg.done();
      res
    });
  }

  /// Spawns a future and increments the wait group
  fn spawn_local<F>(&self, future: F) -> <Self::Runtime as Runtime>::LocalJoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    let wg = self.wg.add(1);
    <Self::Runtime as Runtime>::spawn_local(async move {
      let res = future.await;
      wg.done();
      res
    })
  }

  fn spawn_local_detach<F>(&self, future: F)
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    Self::spawn_local(self, future);
  }

  fn spawn_blocking_detach<F, RR>(&self, f: F)
  where
    F: FnOnce() -> RR + Send + 'static,
    RR: Send + 'static,
  {
    Self::spawn_blocking(self, f);
  }

  /// Spawns a blocking function and increments the wait group
  fn spawn_blocking<F, RR>(&self, f: F) -> <Self::Runtime as Runtime>::BlockJoinHandle<RR>
  where
    F: FnOnce() -> RR + Send + 'static,
    RR: Send + 'static,
  {
    let wg = self.wg.add(1);
    <Self::Runtime as Runtime>::spawn_blocking(move || {
      let res = f();
      wg.done();
      res
    })
  }

  /// Waits for all spawned tasks to finish
  fn wait(&self) -> wg::tokio::WaitGroupFuture<'_> {
    self.wg.wait()
  }

  /// Block waits for all spawned tasks to finish
  fn block_wait(&self) {
    self.wg.block_wait();
  }
}
