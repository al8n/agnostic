//! The acceptance proof for the `LocalRuntimeLite` / `RuntimeLite` split: a
//! runtime whose timers and join handles are **deliberately `!Send`** (a
//! thread-pinned, `LocalSet`-shaped host) can implement the local core on
//! stable Rust, even though it can never satisfy
//! [`RuntimeLite`](agnostic_lite::RuntimeLite)'s `Send` family. Before the
//! split, `RuntimeLite`'s monolithic associated types made such a runtime
//! unimplementable outright. (Completion-based runtimes are deliberately NOT
//! targeted by this abstraction — they warrant native driver integrations —
//! so this fixture proves a bound-shape claim, not a compio integration.)
//!
//! Every timer carrier here embeds `PhantomData<Rc<()>>`, so the language
//! itself guarantees the types are `!Send` — a `Send` bound anywhere on the
//! local core's associated types would fail this file's compilation.
#![cfg(all(feature = "std", feature = "time"))]

use core::{
  future::Future,
  marker::PhantomData,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};
use std::{rc::Rc, time::Instant};

use agnostic_lite::{
  AsyncBlockingSpawner, AsyncLocalSpawner, JoinHandle, LocalJoinHandle, LocalRuntimeLite, Yielder,
  time::{
    AsyncLocalInterval, AsyncLocalSleep, AsyncLocalSleepExt, AsyncLocalTimeout, Delay, Elapsed,
  },
};
use futures_util::Stream;

/// The `!Send` pin: an `Rc` marker makes every carrier type thread-bound,
/// exactly like a completion-based runtime's timer registrations.
type NotSend = PhantomData<Rc<()>>;

struct LocalSleep {
  deadline: Instant,
  _pin: NotSend,
}

impl Future for LocalSleep {
  type Output = Instant;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // A busy-poll shim is enough for a compile-shaped fixture.
    if Instant::now() >= self.deadline {
      Poll::Ready(self.deadline)
    } else {
      cx.waker().wake_by_ref();
      Poll::Pending
    }
  }
}

impl AsyncLocalSleep for LocalSleep {
  type Instant = Instant;

  fn reset(mut self: Pin<&mut Self>, deadline: Self::Instant) {
    self.deadline = deadline;
  }
}

impl AsyncLocalSleepExt for LocalSleep {
  fn sleep_local(after: Duration) -> Self {
    Self {
      deadline: Instant::now() + after,
      _pin: PhantomData,
    }
  }

  fn sleep_local_until(deadline: Self::Instant) -> Self {
    Self {
      deadline,
      _pin: PhantomData,
    }
  }
}

struct LocalInterval {
  next: Instant,
  period: Duration,
  _pin: NotSend,
}

impl Stream for LocalInterval {
  type Item = Instant;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.poll_tick(cx).map(Some)
  }
}

impl AsyncLocalInterval for LocalInterval {
  type Instant = Instant;

  fn reset(&mut self, interval: Duration) {
    self.period = interval;
    self.next = Instant::now() + interval;
  }

  fn reset_at(&mut self, instant: Self::Instant) {
    self.next = instant;
  }

  fn poll_tick(&mut self, cx: &mut Context<'_>) -> Poll<Self::Instant> {
    if Instant::now() >= self.next {
      let tick = self.next;
      self.next = tick + self.period;
      Poll::Ready(tick)
    } else {
      cx.waker().wake_by_ref();
      Poll::Pending
    }
  }
}

pin_project_lite::pin_project! {
  struct LocalTimeout<F> {
    #[pin]
    sleep: LocalSleep,
    #[pin]
    fut: F,
  }
}

impl<F: Future> Future for LocalTimeout<F> {
  type Output = Result<F::Output, Elapsed>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.project();
    if let Poll::Ready(out) = this.fut.poll(cx) {
      return Poll::Ready(Ok(out));
    }
    match this.sleep.poll(cx) {
      Poll::Ready(_) => Poll::Ready(Err(Elapsed)),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl<F: Future> AsyncLocalTimeout<F> for LocalTimeout<F> {
  type Instant = Instant;

  fn timeout_local(timeout: Duration, fut: F) -> Self {
    Self {
      sleep: LocalSleep::sleep_local(timeout),
      fut,
    }
  }

  fn timeout_local_at(deadline: Self::Instant, fut: F) -> Self {
    Self {
      sleep: LocalSleep::sleep_local_until(deadline),
      fut,
    }
  }
}

/// A spawner that panics on local spawn — the [`AsyncLocalSpawner`] contract
/// note permits it, and it mirrors how a `block_on`-hosted, thread-pinned
/// consumer uses the local core: the caller drives the future directly and
/// never spawns. Blocking spawn is a plain thread.
#[derive(Debug, Clone, Copy)]
struct LocalOnlySpawner;

impl Yielder for LocalOnlySpawner {
  async fn yield_now() {}

  async fn yield_now_local() {}
}

/// A never-completing local handle, well-formed but unreachable (spawn panics
/// before one is ever constructed).
struct NeverHandle<O> {
  // `fn() -> O`, not `O`: the handle carries no output value, so its `Unpin`
  // (required by `LocalJoinHandle`) must not condition on `O`'s.
  _out: PhantomData<fn() -> O>,
  _pin: NotSend,
}

#[derive(Debug)]
struct NeverJoinError;

impl core::fmt::Display for NeverJoinError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "unreachable: the local-only fixture never spawns")
  }
}

impl core::error::Error for NeverJoinError {}

impl From<NeverJoinError> for std::io::Error {
  fn from(e: NeverJoinError) -> Self {
    std::io::Error::other(e)
  }
}

impl<O> Future for NeverHandle<O> {
  type Output = Result<O, NeverJoinError>;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    Poll::Pending
  }
}

impl<O> LocalJoinHandle<O> for NeverHandle<O> {
  type JoinError = NeverJoinError;
}

impl AsyncLocalSpawner for LocalOnlySpawner {
  type JoinHandle<O>
    = NeverHandle<O>
  where
    O: 'static;

  fn spawn_local<F>(_future: F) -> Self::JoinHandle<F::Output>
  where
    F::Output: 'static,
    F: Future + 'static,
  {
    panic!("the local-only fixture hosts via block_on; it never spawns")
  }
}

/// A thread-backed blocking handle: `Unpin + Send` via the oneshot receiver.
struct ThreadHandle<R> {
  rx: futures::channel::oneshot::Receiver<R>,
}

impl<R> Future for ThreadHandle<R> {
  type Output = Result<R, NeverJoinError>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match Pin::new(&mut self.rx).poll(cx) {
      Poll::Ready(Ok(v)) => Poll::Ready(Ok(v)),
      Poll::Ready(Err(_)) => Poll::Ready(Err(NeverJoinError)),
      Poll::Pending => Poll::Pending,
    }
  }
}

impl<R> JoinHandle<R> for ThreadHandle<R> {
  type JoinError = NeverJoinError;

  fn abort(self) {}
}

impl AsyncBlockingSpawner for LocalOnlySpawner {
  type JoinHandle<R>
    = ThreadHandle<R>
  where
    R: Send + 'static;

  fn spawn_blocking<F, R>(f: F) -> Self::JoinHandle<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
      let _ = tx.send(f());
    });
    ThreadHandle { rx }
  }
}

/// The thread-pinned runtime marker: only the LOCAL core is implementable —
/// and with the 0.7 split, only the local core is required.
#[derive(Debug, Clone, Copy)]
struct LocalOnlyRuntime;

impl LocalRuntimeLite for LocalOnlyRuntime {
  type LocalSpawner = LocalOnlySpawner;
  type BlockingSpawner = LocalOnlySpawner;

  type Instant = Instant;
  type LocalInterval = LocalInterval;
  type LocalSleep = LocalSleep;
  type LocalDelay<F>
    = Delay<F, LocalSleep>
  where
    F: Future;
  type LocalTimeout<F>
    = LocalTimeout<F>
  where
    F: Future;

  fn new() -> Self {
    Self
  }

  fn name() -> &'static str {
    "local-only"
  }

  fn fqname() -> &'static str {
    "agnostic-lite/tests/local-only"
  }

  fn block_on<F: Future>(f: F) -> F::Output {
    futures::executor::block_on(f)
  }

  fn interval_local(interval: Duration) -> Self::LocalInterval {
    LocalInterval {
      next: Instant::now() + interval,
      period: interval,
      _pin: PhantomData,
    }
  }

  fn interval_local_at(start: Self::Instant, period: Duration) -> Self::LocalInterval {
    LocalInterval {
      next: start,
      period,
      _pin: PhantomData,
    }
  }

  fn sleep_local(duration: Duration) -> Self::LocalSleep {
    LocalSleep::sleep_local(duration)
  }

  fn sleep_local_until(instant: Self::Instant) -> Self::LocalSleep {
    LocalSleep::sleep_local_until(instant)
  }

  fn delay_local<F>(duration: Duration, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use agnostic_lite::time::AsyncLocalDelayExt;

    <Delay<F, LocalSleep> as AsyncLocalDelayExt<F>>::delay(duration, fut)
  }

  fn delay_local_at<F>(deadline: Self::Instant, fut: F) -> Self::LocalDelay<F>
  where
    F: Future,
  {
    use agnostic_lite::time::AsyncLocalDelayExt;

    <Delay<F, LocalSleep> as AsyncLocalDelayExt<F>>::delay_at(deadline, fut)
  }

  fn timeout_local<F>(duration: Duration, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    LocalTimeout::timeout_local(duration, future)
  }

  fn timeout_local_at<F>(deadline: Self::Instant, future: F) -> Self::LocalTimeout<F>
  where
    F: Future,
  {
    LocalTimeout::timeout_local_at(deadline, future)
  }
}

fn assert_local_runtime<R: LocalRuntimeLite>() {}

#[test]
fn local_only_runtime_implements_the_core() {
  assert_local_runtime::<LocalOnlyRuntime>();

  // The core is genuinely usable, not just implementable: an already-ready
  // future inside a generous local timeout resolves through block_on.
  let out = LocalOnlyRuntime::block_on(async {
    LocalOnlyRuntime::timeout_local(Duration::from_secs(5), async { 42u32 }).await
  });
  assert_eq!(out, Ok(42));
}

#[test]
fn local_sleep_reset_moves_the_deadline() {
  let mut sleep = LocalOnlyRuntime::sleep_local(Duration::from_secs(600));
  AsyncLocalSleep::reset(Pin::new(&mut sleep), Instant::now());
  assert!(
    LocalOnlyRuntime::block_on(async { sleep.await }) <= Instant::now(),
    "the reset deadline is already reached"
  );
}
