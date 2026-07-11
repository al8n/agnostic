//! Compile contracts for the 0.7 trait split's supported invocation forms.
//!
//! The `generic_only` module proves the headline claim: a scope whose ONLY
//! `agnostic-lite` import is `RuntimeLite` reaches the ENTIRE former 0.6
//! surface — every moved member and associated type included — through the
//! supertrait bound. The outer scope proves the two migrated forms: a
//! concrete-type call of a moved member with `LocalRuntimeLite` in scope,
//! and a UFCS call naming the owning trait.
#![cfg(all(feature = "std", feature = "time", feature = "smol"))]

use agnostic_lite::{LocalRuntimeLite, RuntimeLite, smol::SmolRuntime};

/// A 0.6 consumer's scope: `RuntimeLite` is the only `agnostic-lite` import;
/// in particular, `LocalRuntimeLite` is NOT in scope here.
mod generic_only {
  use core::{
    future::{Ready, ready},
    marker::PhantomData,
    time::Duration,
  };

  use agnostic_lite::RuntimeLite;

  /// Every member of the former monolithic trait — moved and kept alike —
  /// invoked through the `R: RuntimeLite` bound, with each time carrier
  /// annotated so the associated types resolve in type position too.
  /// Monomorphized by the test but never run: the local spawns panic by
  /// contract on runtimes with no local executor to target, so this function
  /// proves reachability; execution is proven by
  /// [`block_on_reaches_the_moved_core`].
  pub fn every_former_member_resolves<R: RuntimeLite>() {
    // Construction + identity (moved).
    let _: R = R::new();
    let _: &'static str = R::name();
    let _: &'static str = R::fqname();

    // The moved spawner associated types, nameable through the bound alone;
    // the kept ones alongside.
    let _ = PhantomData::<R::LocalSpawner>;
    let _ = PhantomData::<R::BlockingSpawner>;
    let _ = PhantomData::<R::Spawner>;
    let _ = PhantomData::<R::AfterSpawner>;

    // Blocking + local spawn families (moved).
    let _: u32 = R::block_on(async { 7 });
    let _handle = R::spawn_local(async { 7u32 });
    R::spawn_local_detach(async {});
    let _handle = R::spawn_blocking(|| 7u32);
    R::spawn_blocking_detach(|| {});

    // The Local* time family (moved), duration and deadline forms.
    let _now: R::Instant = R::now();
    let _interval: R::LocalInterval = R::interval_local(Duration::from_secs(1));
    let _interval: R::LocalInterval = R::interval_local_at(R::now(), Duration::from_secs(1));
    let _sleep: R::LocalSleep = R::sleep_local(Duration::from_secs(1));
    let _sleep: R::LocalSleep = R::sleep_local_until(R::now());
    let _delay: R::LocalDelay<Ready<()>> = R::delay_local(Duration::from_secs(1), ready(()));
    let _delay: R::LocalDelay<Ready<()>> = R::delay_local_at(R::now(), ready(()));
    let _timeout: R::LocalTimeout<Ready<()>> = R::timeout_local(Duration::from_secs(1), ready(()));
    let _timeout: R::LocalTimeout<Ready<()>> = R::timeout_local_at(R::now(), ready(()));

    // The Send families (kept) resolve in the same scope off the same bound.
    let _handle = R::spawn(async { 7u32 });
    R::spawn_detach(async {});
    let _yield = R::yield_now();
    let _handle = R::spawn_after(Duration::from_secs(1), async { 7u32 });
    let _handle = R::spawn_after_at(R::now(), async { 7u32 });
    let _interval: R::Interval = R::interval(Duration::from_secs(1));
    let _interval: R::Interval = R::interval_at(R::now(), Duration::from_secs(1));
    let _sleep: R::Sleep = R::sleep(Duration::from_secs(1));
    let _sleep: R::Sleep = R::sleep_until(R::now());
    let _delay: R::Delay<Ready<()>> = R::delay(Duration::from_secs(1), ready(()));
    let _delay: R::Delay<Ready<()>> = R::delay_at(R::now(), ready(()));
    let _timeout: R::Timeout<Ready<()>> = R::timeout(Duration::from_secs(1), ready(()));
    let _timeout: R::Timeout<Ready<()>> = R::timeout_at(R::now(), ready(()));

    // A kept member is still reachable by UFCS through this trait.
    let _sleep: R::Sleep = <R as RuntimeLite>::sleep(Duration::from_secs(1));
  }

  /// A moved member actually RUNS through this scope's lone import.
  pub fn block_on_reaches_the_moved_core<R: RuntimeLite>() -> u32 {
    R::block_on(async { 7 })
  }
}

/// The UFCS form for a moved member names the trait that now owns it.
fn ufcs_names_the_owning_trait<R: RuntimeLite>() -> &'static str {
  <R as LocalRuntimeLite>::name()
}

#[test]
fn generic_scope_reaches_every_former_member() {
  // Monomorphize the full-surface contract against a real runtime (the cast
  // forces instantiation); see the function's note for why it is never run.
  let _ = generic_only::every_former_member_resolves::<SmolRuntime> as fn();

  assert_eq!(
    generic_only::block_on_reaches_the_moved_core::<SmolRuntime>(),
    7
  );
}

#[test]
fn concrete_calls_resolve_with_the_core_trait_in_scope() {
  assert_eq!(ufcs_names_the_owning_trait::<SmolRuntime>(), "smol");

  // The concrete form: `LocalRuntimeLite` is in scope (imported above), so a
  // moved member resolves on the concrete type directly.
  assert_eq!(SmolRuntime::name(), "smol");
  let out = SmolRuntime::block_on(async { 7u32 });
  assert_eq!(out, 7);
}
