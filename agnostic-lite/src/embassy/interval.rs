use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

use futures_util::stream::Stream;

use crate::time::{AsyncLocalInterval, AsyncLocalIntervalExt, AsyncLocalSleep, AsyncSleepExt};

use super::{EmbassySleep, Instant};

/// The [`AsyncInterval`](crate::time::AsyncInterval) implementation for the embassy runtime.
///
/// **Note:** `EmbassyInterval` is not accurate below the resolution of the configured
/// [`embassy-time`](https://docs.rs/embassy-time) tick rate.
pub struct EmbassyInterval {
  inner: Pin<std::boxed::Box<EmbassySleep>>,
  first: bool,
}

impl Stream for EmbassyInterval {
  type Item = Instant;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.get_mut().poll_tick(cx).map(Some)
  }
}

impl AsyncLocalInterval for EmbassyInterval {
  type Instant = Instant;

  fn reset(&mut self, interval: Duration) {
    self.inner.as_mut().reset(Instant::now() + interval);
  }

  fn reset_at(&mut self, instant: Instant) {
    self.inner.as_mut().reset(instant);
  }

  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Instant> {
    if self.first {
      self.first = false;
      return Poll::Ready(self.inner.ddl - self.inner.duration);
    }

    match self.inner.as_mut().poll(cx) {
      Poll::Ready(ins) => {
        let duration = self.inner.duration;
        self.inner.as_mut().reset(Instant::now() + duration);
        Poll::Ready(ins)
      }
      Poll::Pending => Poll::Pending,
    }
  }
}

impl AsyncLocalIntervalExt for EmbassyInterval {
  fn interval_local(period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: std::boxed::Box::pin(EmbassySleep::sleep(period)),
      first: true,
    }
  }

  fn interval_local_at(start: Instant, period: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      inner: std::boxed::Box::pin(EmbassySleep::sleep_until(start + period)),
      first: true,
    }
  }
}
