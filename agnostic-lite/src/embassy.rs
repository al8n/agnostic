use core::{
  future::Future,
  pin::Pin,
  task::{Context, Poll},
};

use embassy_executor::{SendSpawner, Spawner};
use embassy_sync::once_lock::OnceLock;
use futures_channel::oneshot;
use futures_util::future::{Either, select};

use crate::{
  AsyncBlockingSpawner, AsyncLocalSpawner, AsyncSpawner, Yielder, spawner::handle::JoinError,
};

mod after;
mod delay;
mod interval;
mod sleep;
mod timeout;

pub use after::*;
pub use delay::*;
pub use interval::*;
pub use sleep::*;
pub use timeout::*;

/// The maximum number of [`spawn`](crate::RuntimeLite::spawn)ed tasks that can be alive at the
/// same time.
///
/// Because [`embassy-executor`](https://docs.rs/embassy-executor) allocates its task storage
/// statically, the number of concurrently running spawned tasks is bounded. If more than
/// `TASK_POOL_SIZE` tasks are alive at once, [`spawn`](crate::RuntimeLite::spawn) fails and the
/// returned [`JoinHandle`] resolves to an error.
pub const TASK_POOL_SIZE: usize = 64;

type DynFuture = Pin<std::boxed::Box<dyn Future<Output = ()> + Send + 'static>>;

#[embassy_executor::task(pool_size = TASK_POOL_SIZE)]
async fn task_runner(fut: DynFuture) {
  fut.await
}

static SPAWNER: OnceLock<SendSpawner> = OnceLock::new();

/// Initializes the global [`embassy-executor`](https://docs.rs/embassy-executor) spawner used by
/// [`EmbassyRuntime`].
///
/// This **must** be called once, from within a running embassy executor (i.e. with the
/// [`Spawner`] handed to you by `#[embassy_executor::main]` or `Executor::run`), before any task
/// is spawned through [`EmbassyRuntime`]. Spawning before initialization will panic.
///
/// Calling this more than once is a no-op; only the first spawner is retained.
///
/// ```rust,ignore
/// #[embassy_executor::main]
/// async fn main(spawner: embassy_executor::Spawner) {
///   agnostic_lite::embassy::init(spawner);
///   // `EmbassyRuntime::spawn(..)` is now usable from anywhere.
/// }
/// ```
pub fn init(spawner: Spawner) {
  let _ = SPAWNER.init(spawner.make_send());
}

#[inline]
fn global_spawner() -> SendSpawner {
  *SPAWNER.try_get().expect(
    "embassy runtime is not initialized; call `agnostic_lite::embassy::init(spawner)` from within \
     a running embassy executor before spawning",
  )
}

fn spawn_send<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let (tx, rx) = oneshot::channel::<F::Output>();
  let (stop_tx, stop_rx) = oneshot::channel::<bool>();

  let wrapped: DynFuture = std::boxed::Box::pin(async move {
    let future = core::pin::pin!(future);
    match select(future, stop_rx).await {
      Either::Left((output, _)) => {
        let _ = tx.send(output);
      }
      Either::Right((signal, future)) => match signal {
        // `abort` was called: drop the task without running it to completion.
        Ok(true) => {}
        // `detach` was called, or the handle was dropped: keep running to completion.
        Ok(false) | Err(_) => {
          let _ = future.await;
        }
      },
    }
  });

  // On `SpawnError::Busy` the task pool is exhausted; `wrapped` (and therefore `tx`) is dropped,
  // so `rx` resolves to a `JoinError` and the caller observes the failure through the handle.
  if let Ok(token) = task_runner(wrapped) {
    global_spawner().spawn(token);
  }

  JoinHandle { stop_tx, rx }
}

/// The [`JoinHandle`](crate::JoinHandle) returned by [`EmbassySpawner`].
///
/// Dropping the handle detaches the task, letting it keep running in the background.
pub struct JoinHandle<T> {
  stop_tx: oneshot::Sender<bool>,
  rx: oneshot::Receiver<T>,
}

impl<T> JoinHandle<T> {
  /// Detaches the task, letting it keep running in the background.
  #[inline]
  pub fn detach(self) {
    let _ = self.stop_tx.send(false);
  }

  /// Aborts the task. If it has not finished yet, it will not be run to completion.
  #[inline]
  pub fn abort(self) {
    let _ = self.stop_tx.send(true);
  }
}

impl<T> Future for JoinHandle<T> {
  type Output = Result<T, JoinError>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    Pin::new(&mut self.rx)
      .poll(cx)
      .map(|res| res.map_err(|_| JoinError::new()))
  }
}

impl<O> crate::JoinHandle<O> for JoinHandle<O> {
  type JoinError = JoinError;

  fn abort(self) {
    Self::abort(self)
  }

  fn detach(self) {
    Self::detach(self)
  }
}

impl<O> crate::LocalJoinHandle<O> for JoinHandle<O> {
  type JoinError = JoinError;

  fn detach(self) {
    Self::detach(self)
  }
}

/// Future for the [`yield_now`](RuntimeLite::yield_now) function.
///
/// [`RuntimeLite::yield_now`]: crate::RuntimeLite::yield_now
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct YieldNow(bool);

impl Future for YieldNow {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if !self.0 {
      self.0 = true;
      cx.waker().wake_by_ref();
      Poll::Pending
    } else {
      Poll::Ready(())
    }
  }
}

/// A [`AsyncSpawner`] that uses the [`embassy-executor`](https://docs.rs/embassy-executor) runtime.
#[derive(Debug, Clone, Copy)]
pub struct EmbassySpawner;

impl Yielder for EmbassySpawner {
  async fn yield_now() {
    YieldNow(false).await
  }

  async fn yield_now_local() {
    YieldNow(false).await
  }
}

impl AsyncSpawner for EmbassySpawner {
  type JoinHandle<F>
    = JoinHandle<F>
  where
    F: Send + 'static;

  fn spawn<F>(future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: Send + 'static,
    F: Future + Send + 'static,
  {
    spawn_send(future)
  }
}

impl AsyncLocalSpawner for EmbassySpawner {
  type JoinHandle<F>
    = JoinHandle<F>
  where
    F: 'static;

  fn spawn_local<F>(_future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    panic!(
      "the embassy backend cannot spawn `!Send` local tasks: its spawner is stored in a global \
       `static`, which can only hold the `Send` spawner. Use `spawn` with a `Send` future instead."
    )
  }
}

impl AsyncBlockingSpawner for EmbassySpawner {
  type JoinHandle<R>
    = JoinHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(_f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    panic!("the embassy backend does not support blocking tasks")
  }
}

/// Runs the given future to completion on the current thread by busy-polling it.
///
/// This uses [`embassy_futures::block_on`], which repeatedly polls the future without sleeping the
/// CPU. It is provided for parity and host testing; on real embedded targets you almost always
/// want to drive futures with an [`embassy_executor`] executor instead.
pub fn block_on<F: Future>(future: F) -> F::Output {
  embassy_futures::block_on(future)
}

#[inline]
pub(crate) fn to_embassy_duration(d: core::time::Duration) -> embassy_time::Duration {
  embassy_time::Duration::from_micros(d.as_micros().min(u64::MAX as u128) as u64)
}

#[inline]
pub(crate) fn from_embassy_duration(d: embassy_time::Duration) -> core::time::Duration {
  core::time::Duration::from_micros(d.as_micros())
}

/// A measurement of a monotonically nondecreasing clock, backed by [`embassy_time::Instant`].
///
/// Sub-microsecond precision is truncated when converting to and from [`core::time::Duration`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(embassy_time::Instant);

impl core::hash::Hash for Instant {
  #[inline]
  fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
    self.0.as_ticks().hash(state);
  }
}

impl Instant {
  /// Returns an instant corresponding to "now".
  #[inline]
  #[must_use]
  pub fn now() -> Self {
    Self(embassy_time::Instant::now())
  }

  /// Returns the underlying [`embassy_time::Instant`].
  #[inline]
  pub const fn into_embassy(self) -> embassy_time::Instant {
    self.0
  }
}

impl From<embassy_time::Instant> for Instant {
  #[inline]
  fn from(instant: embassy_time::Instant) -> Self {
    Self(instant)
  }
}

impl From<Instant> for embassy_time::Instant {
  #[inline]
  fn from(instant: Instant) -> Self {
    instant.0
  }
}

impl core::ops::Add<core::time::Duration> for Instant {
  type Output = Self;

  #[inline]
  fn add(self, rhs: core::time::Duration) -> Self {
    Self(self.0 + to_embassy_duration(rhs))
  }
}

impl core::ops::AddAssign<core::time::Duration> for Instant {
  #[inline]
  fn add_assign(&mut self, rhs: core::time::Duration) {
    self.0 = self.0 + to_embassy_duration(rhs);
  }
}

impl core::ops::Sub<core::time::Duration> for Instant {
  type Output = Self;

  #[inline]
  fn sub(self, rhs: core::time::Duration) -> Self {
    Self(self.0 - to_embassy_duration(rhs))
  }
}

impl core::ops::SubAssign<core::time::Duration> for Instant {
  #[inline]
  fn sub_assign(&mut self, rhs: core::time::Duration) {
    self.0 = self.0 - to_embassy_duration(rhs);
  }
}

impl core::ops::Sub<Instant> for Instant {
  type Output = core::time::Duration;

  #[inline]
  fn sub(self, rhs: Instant) -> core::time::Duration {
    from_embassy_duration(self.0 - rhs.0)
  }
}

impl crate::time::Instant for Instant {
  #[inline]
  fn now() -> Self {
    Self(embassy_time::Instant::now())
  }

  #[inline]
  fn elapsed(&self) -> core::time::Duration {
    from_embassy_duration(self.0.elapsed())
  }

  #[inline]
  fn checked_add(&self, duration: core::time::Duration) -> Option<Self> {
    self.0.checked_add(to_embassy_duration(duration)).map(Self)
  }

  #[inline]
  fn checked_sub(&self, duration: core::time::Duration) -> Option<Self> {
    self.0.checked_sub(to_embassy_duration(duration)).map(Self)
  }

  #[inline]
  fn checked_duration_since(&self, earlier: Self) -> Option<core::time::Duration> {
    self
      .0
      .checked_duration_since(earlier.0)
      .map(from_embassy_duration)
  }
}

// The `Instant` trait requires conversions to and from `std::time::Instant` when the `std` feature
// is enabled. The embassy clock and the std clock are unrelated, so these conversions anchor the
// embassy timeline to the std timeline at "now". They are only meaningful for host interop and
// testing; on real `no_std` targets these impls do not exist.
#[cfg(feature = "std")]
const _: () = {
  use std::time::Instant as StdInstant;

  impl From<StdInstant> for Instant {
    fn from(value: StdInstant) -> Self {
      let now_std = StdInstant::now();
      let now = embassy_time::Instant::now();
      Self(if value >= now_std {
        now + to_embassy_duration(value - now_std)
      } else {
        now
          .checked_sub(to_embassy_duration(now_std - value))
          .unwrap_or(now)
      })
    }
  }

  impl From<Instant> for StdInstant {
    fn from(value: Instant) -> Self {
      let now = embassy_time::Instant::now();
      let now_std = StdInstant::now();
      if value.0 >= now {
        now_std + from_embassy_duration(value.0 - now)
      } else {
        now_std
          .checked_sub(from_embassy_duration(now - value.0))
          .unwrap_or(now_std)
      }
    }
  }
};

/// Concrete [`RuntimeLite`](crate::RuntimeLite) implementation based on
/// [`embassy-executor`](https://docs.rs/embassy-executor).
///
/// Before any task is spawned through this runtime, the global spawner must be installed once with
/// [`init`] from within a running embassy executor. See the [module documentation](self) for the
/// caveats of this backend ([`block_on`] busy-polls, [`spawn_blocking`] and local spawning panic,
/// and the number of live spawned tasks is bounded by [`TASK_POOL_SIZE`]).
///
/// [`spawn_blocking`]: crate::RuntimeLite::spawn_blocking
#[derive(Debug, Clone, Copy)]
pub struct EmbassyRuntime;

impl core::fmt::Display for EmbassyRuntime {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "embassy")
  }
}

impl crate::RuntimeLite for EmbassyRuntime {
  type Spawner = EmbassySpawner;
  type LocalSpawner = EmbassySpawner;
  type BlockingSpawner = EmbassySpawner;

  type Instant = Instant;
  type AfterSpawner = EmbassySpawner;

  type Interval = EmbassyInterval;
  type LocalInterval = EmbassyInterval;

  type Sleep = EmbassySleep;
  type LocalSleep = EmbassySleep;

  type Delay<F>
    = EmbassyDelay<F>
  where
    F: Future + Send;

  type LocalDelay<F>
    = EmbassyDelay<F>
  where
    F: Future;

  type Timeout<F>
    = EmbassyTimeout<F>
  where
    F: Future + Send;

  type LocalTimeout<F>
    = EmbassyTimeout<F>
  where
    F: Future;

  fn new() -> Self {
    Self
  }

  fn name() -> &'static str {
    "embassy"
  }

  fn fqname() -> &'static str {
    "embassy-executor"
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    block_on(f)
  }

  async fn yield_now() {
    YieldNow(false).await
  }

  fn interval(interval: core::time::Duration) -> Self::Interval {
    use crate::time::AsyncIntervalExt;

    EmbassyInterval::interval(interval)
  }

  fn interval_at(start: Instant, period: core::time::Duration) -> Self::Interval {
    use crate::time::AsyncIntervalExt;

    EmbassyInterval::interval_at(start, period)
  }

  fn interval_local(interval: core::time::Duration) -> Self::LocalInterval {
    use crate::time::AsyncIntervalExt;

    EmbassyInterval::interval(interval)
  }

  fn interval_local_at(start: Instant, period: core::time::Duration) -> Self::LocalInterval {
    use crate::time::AsyncIntervalExt;

    EmbassyInterval::interval_at(start, period)
  }

  fn sleep(duration: core::time::Duration) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    EmbassySleep::sleep(duration)
  }

  fn sleep_until(instant: Instant) -> Self::Sleep {
    use crate::time::AsyncSleepExt;

    EmbassySleep::sleep_until(instant)
  }

  fn sleep_local(duration: core::time::Duration) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    EmbassySleep::sleep(duration)
  }

  fn sleep_local_until(instant: Instant) -> Self::LocalSleep {
    use crate::time::AsyncSleepExt;

    EmbassySleep::sleep_until(instant)
  }

  fn delay<F>(duration: core::time::Duration, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <EmbassyDelay<F> as AsyncDelayExt<F>>::delay(duration, fut)
  }

  fn delay_local<F>(duration: core::time::Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <EmbassyDelay<F> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  fn delay_at<F>(deadline: Instant, fut: F) -> Self::Delay<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncDelayExt;

    <EmbassyDelay<F> as AsyncDelayExt<F>>::delay_at(deadline, fut)
  }

  fn delay_local_at<F>(deadline: Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalDelayExt;

    <EmbassyDelay<F> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }

  fn timeout<F>(duration: core::time::Duration, future: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncTimeout;

    <EmbassyTimeout<F> as AsyncTimeout<F>>::timeout(duration, future)
  }

  fn timeout_at<F>(deadline: Instant, future: F) -> Self::Timeout<F>
  where
    F: Future + Send,
  {
    use crate::time::AsyncTimeout;

    <EmbassyTimeout<F> as AsyncTimeout<F>>::timeout_at(deadline, future)
  }

  fn timeout_local<F>(duration: core::time::Duration, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalTimeout;

    <EmbassyTimeout<F> as AsyncLocalTimeout<F>>::timeout_local(duration, future)
  }

  fn timeout_local_at<F>(deadline: Instant, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    use crate::time::AsyncLocalTimeout;

    <EmbassyTimeout<F> as AsyncLocalTimeout<F>>::timeout_local_at(deadline, future)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{RuntimeLite, time::Instant as _};
  use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
  };
  use futures::executor::block_on;
  use futures_util::StreamExt;

  // Embassy's `Executor::run` never returns and the global spawner is set once, so all tests share
  // a single executor running on a background thread. Test futures are driven on the test thread
  // with `futures::executor::block_on` (embassy-time's generic queue, enabled for the test build,
  // lets `Timer` work under any waker); tasks spawned through `EmbassyRuntime` run on the
  // background executor and are bridged back via the channel-backed join handles.
  fn ensure_runtime() {
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
      let (tx, rx) = std::sync::mpsc::channel();
      std::thread::Builder::new()
        .name("embassy-test-executor".into())
        .spawn(move || {
          let executor: &'static mut embassy_executor::Executor =
            std::boxed::Box::leak(std::boxed::Box::new(embassy_executor::Executor::new()));
          executor.run(|spawner| {
            init(spawner);
            let _ = tx.send(());
          });
        })
        .unwrap();
      rx.recv().unwrap();
    });
  }

  #[test]
  fn spawn_and_join() {
    ensure_runtime();
    assert_eq!(
      block_on(async { EmbassyRuntime::spawn(async { 21 * 2 }).await.unwrap() }),
      42
    );
  }

  #[test]
  fn spawn_detach_runs() {
    ensure_runtime();
    let flag = std::sync::Arc::new(AtomicBool::new(false));
    let f = flag.clone();
    EmbassyRuntime::spawn_detach(async move { f.store(true, Ordering::SeqCst) });
    block_on(EmbassyRuntime::sleep(Duration::from_millis(100)));
    assert!(flag.load(Ordering::SeqCst));
  }

  #[test]
  fn block_on_drives_to_completion() {
    let out = EmbassyRuntime::block_on(async {
      EmbassyRuntime::yield_now().await;
      42
    });
    assert_eq!(out, 42);
  }

  #[test]
  fn sleep_waits() {
    block_on(async {
      let start = EmbassyRuntime::now();
      EmbassyRuntime::sleep(Duration::from_millis(50)).await;
      assert!(start.elapsed() >= Duration::from_millis(40));
    });
  }

  #[test]
  fn timeout_expires_and_completes() {
    block_on(async {
      let expired = EmbassyRuntime::timeout(Duration::from_millis(20), async {
        EmbassyRuntime::sleep(Duration::from_millis(500)).await;
      })
      .await;
      assert!(expired.is_err());

      let done = EmbassyRuntime::timeout(Duration::from_millis(200), async { 7 }).await;
      assert_eq!(done.unwrap(), 7);
    });
  }

  #[test]
  fn interval_first_tick_is_immediate() {
    block_on(async {
      let start = EmbassyRuntime::now();
      let mut interval = EmbassyRuntime::interval(Duration::from_millis(50));
      interval.next().await;
      assert!(start.elapsed() < Duration::from_millis(40));
      interval.next().await;
      assert!(start.elapsed() >= Duration::from_millis(40));
    });
  }

  #[test]
  fn delay_runs_after_duration() {
    block_on(async {
      let start = EmbassyRuntime::now();
      let out = EmbassyRuntime::delay(Duration::from_millis(50), async { 9 })
        .await
        .unwrap();
      assert_eq!(out, 9);
      assert!(start.elapsed() >= Duration::from_millis(40));
    });
  }

  #[test]
  fn spawn_after() {
    ensure_runtime();
    block_on(crate::tests::spawn_after_unittest::<EmbassyRuntime>());
  }

  #[test]
  fn spawn_after_cancel() {
    ensure_runtime();
    block_on(crate::tests::spawn_after_cancel_unittest::<EmbassyRuntime>());
  }

  #[test]
  fn spawn_after_drop() {
    ensure_runtime();
    block_on(crate::tests::spawn_after_drop_unittest::<EmbassyRuntime>());
  }

  #[test]
  fn spawn_after_abort() {
    ensure_runtime();
    block_on(crate::tests::spawn_after_abort_unittest::<EmbassyRuntime>());
  }

  #[test]
  fn spawn_after_reset_to_pass() {
    ensure_runtime();
    block_on(crate::tests::spawn_after_reset_to_pass_unittest::<
      EmbassyRuntime,
    >());
  }

  #[test]
  fn spawn_after_reset_to_future() {
    ensure_runtime();
    block_on(crate::tests::spawn_after_reset_to_future_unittest::<
      EmbassyRuntime,
    >());
  }
}
