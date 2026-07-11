//! Compile contracts for the 0.7 trait split's supported invocation forms:
//! a generic `R: RuntimeLite` bound reaches every moved member through the
//! supertrait unchanged, while concrete-type and UFCS calls of moved members
//! name (or import) [`LocalRuntimeLite`]. These functions only need to
//! compile; the smol cell also runs.
#![cfg(all(feature = "std", feature = "time", feature = "smol"))]

use core::time::Duration;

use agnostic_lite::{LocalRuntimeLite, RuntimeLite, smol::SmolRuntime};

/// The generic form: ONLY the `RuntimeLite` bound, every call a 0.6 consumer
/// could write — construction, identity, blocking, both time families —
/// resolves through the same `R::` paths.
fn generic_consumer_paths_are_unchanged<R: RuntimeLite>() {
  let _ = R::new();
  let _ = R::name();
  let _ = R::fqname();
  let _ = R::now();
  let _sleep = R::sleep(Duration::from_secs(1));
  let _local_sleep = R::sleep_local(Duration::from_secs(1));
  let _timeout = R::timeout(Duration::from_secs(1), async {});
  let _local_timeout = R::timeout_local(Duration::from_secs(1), async {});
  let _interval = R::interval(Duration::from_secs(1));
  let _local_interval = R::interval_local(Duration::from_secs(1));
}

/// The UFCS form for a moved member names the owning trait.
fn ufcs_names_the_owning_trait<R: RuntimeLite>() -> &'static str {
  <R as LocalRuntimeLite>::name()
}

#[test]
fn concrete_calls_resolve_with_the_core_trait_in_scope() {
  // Monomorphize the generic contracts against a real runtime.
  let _ = generic_consumer_paths_are_unchanged::<SmolRuntime>;
  assert_eq!(ufcs_names_the_owning_trait::<SmolRuntime>(), "smol");

  // The concrete form: `LocalRuntimeLite` is in scope (imported above), so a
  // moved member resolves on the concrete type directly.
  assert_eq!(SmolRuntime::name(), "smol");
  let out = SmolRuntime::block_on(async { 7u32 });
  assert_eq!(out, 7);
}
