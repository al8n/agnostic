/// Alias for [`Delay`] using [`tokio`] runtime.
pub type AsyncIoDelay<F> = crate::time::Delay<F, super::AsyncIoSleep>;

#[cfg(test)]
mod tests {
  use crate::time::{AsyncDelay, AsyncDelayExt};

  use super::AsyncIoDelay;

  use std::{
    sync::{
      atomic::{AtomicUsize, Ordering},
      Arc,
    },
    time::{Duration, Instant},
  };

  const DELAY: Duration = Duration::from_millis(1);
  const RESET: Duration = Duration::from_millis(2);
  const BOUND: Duration = Duration::from_millis(50);

  #[test]
  fn test_delay() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let ctr = Arc::new(AtomicUsize::new(0));
      let ctr1 = ctr.clone();
      let delay = <AsyncIoDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
        ctr1.fetch_add(1, Ordering::SeqCst);
      });
      delay.await.unwrap();
      let elapsed = start.elapsed();
      assert!(elapsed >= DELAY);
      assert!(elapsed < DELAY + BOUND);
      assert_eq!(ctr.load(Ordering::SeqCst), 1);
    });
  }

  #[test]
  fn test_delay_at() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let ctr = Arc::new(AtomicUsize::new(0));
      let ctr1 = ctr.clone();
      let delay = <AsyncIoDelay<_> as AsyncDelayExt<_>>::delay_at(start + DELAY, async move {
        ctr1.fetch_add(1, Ordering::SeqCst);
      });
      delay.await.unwrap();
      let elapsed = start.elapsed();
      assert!(elapsed >= DELAY);
      assert!(elapsed < DELAY + BOUND);
      assert_eq!(ctr.load(Ordering::SeqCst), 1);
    });
  }

  #[test]
  fn test_delay_reset() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let ctr = Arc::new(AtomicUsize::new(0));
      let ctr1 = ctr.clone();
      let delay = <AsyncIoDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
        ctr1.fetch_add(1, Ordering::SeqCst);
      });
      futures_util::pin_mut!(delay);
      AsyncDelay::reset(delay.as_mut(), RESET);
      delay.await.unwrap();
      let elapsed = start.elapsed();
      assert!(elapsed >= RESET);
      assert!(elapsed < RESET + BOUND);
      assert_eq!(ctr.load(Ordering::SeqCst), 1);
    });
  }

  #[test]
  fn test_delay_abort() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let ctr = Arc::new(AtomicUsize::new(0));
      let ctr1 = ctr.clone();
      let delay = <AsyncIoDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
        ctr1.fetch_add(1, Ordering::SeqCst);
      });
      AsyncDelay::abort(&delay);
      assert!(delay.await.is_err());
      let elapsed = start.elapsed();
      assert!(elapsed < DELAY);
      assert_eq!(ctr.load(Ordering::SeqCst), 0);
    });
  }

  #[test]
  fn test_delay_cancel() {
    futures::executor::block_on(async {
      let start = Instant::now();
      let ctr = Arc::new(AtomicUsize::new(0));
      let ctr1 = ctr.clone();
      let delay = <AsyncIoDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
        ctr1.fetch_add(1, Ordering::SeqCst);
      });
      AsyncDelay::cancel(&delay);
      assert!(delay.await.is_ok());
      let elapsed = start.elapsed();
      assert!(elapsed < BOUND);
      assert_eq!(ctr.load(Ordering::SeqCst), 1);
    });
  }
}
