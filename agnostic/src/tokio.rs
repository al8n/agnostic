use std::task::Poll;

use ::tokio::sync::mpsc;
use tokio_stream::wrappers::IntervalStream;

use super::*;

#[cfg(feature = "net")]
pub mod net;

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

pin_project_lite::pin_project! {
  pub struct TokioTimeout<F: Future> {
    #[pin]
    timeout: ::tokio::time::Timeout<F>,
  }
}

impl<F: Future> From<::tokio::time::Timeout<F>> for TokioTimeout<F> {
  fn from(timeout: ::tokio::time::Timeout<F>) -> Self {
    Self { timeout }
  }
}

impl<F: Future> From<TokioTimeout<F>> for ::tokio::time::Timeout<F> {
  fn from(timeout: TokioTimeout<F>) -> Self {
    timeout.timeout
  }
}

impl<F: Future> Future for TokioTimeout<F> {
  type Output = std::io::Result<F::Output>;

  fn poll(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    let this = self.project();
    match this.timeout.poll(cx) {
      Poll::Ready(Ok(t)) => Poll::Ready(Ok(t)),
      Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::TimedOut, e))),
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
  type JoinHandle<T> = ::tokio::task::JoinHandle<T>;
  type Interval = IntervalStream;
  type Sleep = ::tokio::time::Sleep;
  type Delay<F> = TokioDelay<F> where F: Future + Send + 'static, F::Output: Send;
  type Timeout<F> = TokioTimeout<F> where F: Future + Send;
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

  fn spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
  where
    F: Future + 'static,
    F::Output: 'static,
  {
    ::tokio::task::spawn_local(fut)
  }

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
    TokioTimeout {
      timeout: ::tokio::time::timeout(duration, fut),
    }
  }

  fn timeout_at<F>(instant: Instant, fut: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    TokioTimeout {
      timeout: ::tokio::time::timeout_at(instant.into(), fut),
    }
  }
}
