/// Alias for [`Delay`](crate::time::Delay) using [`tokio`] runtime.
pub type TokioDelay<F> = crate::time::Delay<F, super::TokioSleep>;

#[cfg(test)]
mod tests {
  use crate::time::{AsyncDelay, AsyncDelayExt};

  use super::TokioDelay;

  use core::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
  };
  use std::sync::Arc;
  use tokio::time::Instant;

  const DELAY: Duration = Duration::from_millis(10);
  const RESET: Duration = Duration::from_millis(20);
  const BOUND: Duration = Duration::from_millis(50);

  #[tokio::test]
  async fn test_delay() {
    let start = Instant::now();
    let ctr = Arc::new(AtomicUsize::new(0));
    let ctr1 = ctr.clone();
    let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });
    delay.await.unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed >= DELAY);
    assert!(elapsed < DELAY + BOUND);
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn test_delay_at() {
    let start = Instant::now();
    let ctr = Arc::new(AtomicUsize::new(0));
    let ctr1 = ctr.clone();
    let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay_at(start + DELAY, async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });
    delay.await.unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed >= DELAY);
    assert!(elapsed < DELAY + BOUND);
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn test_delay_reset() {
    let start = Instant::now();
    let ctr = Arc::new(AtomicUsize::new(0));
    let ctr1 = ctr.clone();
    let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });
    futures_util::pin_mut!(delay);
    AsyncDelay::reset(delay.as_mut(), RESET);
    delay.await.unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed >= RESET);
    assert!(elapsed < RESET + BOUND);
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn test_delay_reset_at() {
    let start = Instant::now();
    let ctr = Arc::new(AtomicUsize::new(0));
    let ctr1 = ctr.clone();
    let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });
    futures_util::pin_mut!(delay);
    AsyncDelay::reset_at(delay.as_mut(), Instant::now() + RESET);
    delay.await.unwrap();
    let elapsed = start.elapsed();
    assert!(elapsed >= RESET);
    assert!(elapsed < RESET + BOUND);
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn test_delay_abort() {
    let start = Instant::now();
    let ctr = Arc::new(AtomicUsize::new(0));
    let ctr1 = ctr.clone();
    let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });
    AsyncDelay::abort(&delay);
    assert!(delay.await.is_err());
    let elapsed = start.elapsed();
    assert!(elapsed < DELAY);
    assert_eq!(ctr.load(Ordering::SeqCst), 0);
  }

  #[tokio::test]
  async fn test_delay_cancel() {
    let start = Instant::now();
    let ctr = Arc::new(AtomicUsize::new(0));
    let ctr1 = ctr.clone();
    let delay = <TokioDelay<_> as AsyncDelayExt<_>>::delay(DELAY, async move {
      ctr1.fetch_add(1, Ordering::SeqCst);
    });
    AsyncDelay::cancel(&delay);
    assert!(delay.await.is_ok());
    let elapsed = start.elapsed();
    assert!(elapsed < BOUND);
    assert_eq!(ctr.load(Ordering::SeqCst), 1);
  }
}
