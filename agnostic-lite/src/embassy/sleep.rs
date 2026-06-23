use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

use embassy_time::Timer;

use crate::time::{AsyncLocalSleep, AsyncLocalSleepExt};

use super::{Instant, to_embassy_duration};

pin_project_lite::pin_project! {
  /// The [`AsyncSleep`](crate::time::AsyncSleep) implementation for the embassy runtime.
  pub struct EmbassySleep {
    #[pin]
    pub(crate) timer: Timer,
    pub(crate) ddl: Instant,
    pub(crate) duration: Duration,
  }
}

impl Future for EmbassySleep {
  type Output = Instant;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let ddl = self.ddl;
    self.project().timer.poll(cx).map(|()| ddl)
  }
}

impl AsyncLocalSleep for EmbassySleep {
  type Instant = Instant;

  fn reset(self: Pin<&mut Self>, deadline: Instant) {
    let mut this = self.project();
    this.timer.set(Timer::at(deadline.into_embassy()));
    *this.ddl = deadline;
  }
}

impl AsyncLocalSleepExt for EmbassySleep {
  fn sleep_local(after: Duration) -> Self
  where
    Self: Sized,
  {
    Self {
      ddl: Instant::now() + after,
      timer: Timer::after(to_embassy_duration(after)),
      duration: after,
    }
  }

  fn sleep_local_until(deadline: Instant) -> Self
  where
    Self: Sized,
  {
    Self {
      duration: deadline - Instant::now(),
      timer: Timer::at(deadline.into_embassy()),
      ddl: deadline,
    }
  }
}
