use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

use embassy_time::Timer;

use crate::time::{AsyncLocalTimeout, AsyncTimeout, Elapsed};

use super::{Instant, to_embassy_duration};

pin_project_lite::pin_project! {
  /// The [`AsyncTimeout`] implementation for the embassy runtime.
  pub struct EmbassyTimeout<F> {
    #[pin]
    future: F,
    #[pin]
    delay: Timer,
  }
}

impl<F: Future> Future for EmbassyTimeout<F> {
  type Output = Result<F::Output, Elapsed>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    match this.future.poll(cx) {
      Poll::Ready(v) => Poll::Ready(Ok(v)),
      Poll::Pending => match this.delay.poll(cx) {
        Poll::Ready(()) => Poll::Ready(Err(Elapsed)),
        Poll::Pending => Poll::Pending,
      },
    }
  }
}

impl<F: Future + Send> AsyncTimeout<F> for EmbassyTimeout<F> {
  type Instant = Instant;

  fn timeout(t: Duration, fut: F) -> Self
  where
    Self: Sized,
  {
    <Self as AsyncLocalTimeout<F>>::timeout_local(t, fut)
  }

  fn timeout_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized,
  {
    <Self as AsyncLocalTimeout<F>>::timeout_local_at(deadline, fut)
  }
}

impl<F: Future> AsyncLocalTimeout<F> for EmbassyTimeout<F> {
  type Instant = Instant;

  fn timeout_local(timeout: Duration, fut: F) -> Self
  where
    Self: Sized,
  {
    Self {
      future: fut,
      delay: Timer::after(to_embassy_duration(timeout)),
    }
  }

  fn timeout_local_at(deadline: Instant, fut: F) -> Self
  where
    Self: Sized,
  {
    Self {
      future: fut,
      delay: Timer::at(deadline.into_embassy()),
    }
  }
}
