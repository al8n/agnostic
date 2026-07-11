//! smol has no ambient thread-local executor, so its local spawner panics
//! loudly instead of silently losing the task (before 0.7 it spawned onto a
//! temporary `LocalExecutor` that dropped at the end of the statement:
//! `spawn_local_detach` never ran the task, and awaiting `spawn_local`'s
//! handle panicked with "task polled after completion").
#![cfg(feature = "smol")]

use agnostic_lite::{LocalRuntimeLite, smol::SmolRuntime};

#[test]
#[should_panic(expected = "no ambient thread-local executor")]
fn smol_spawn_local_panics_loudly() {
  let _handle = SmolRuntime::spawn_local(async { 42u32 });
}

#[test]
#[should_panic(expected = "no ambient thread-local executor")]
fn smol_spawn_local_detach_panics_loudly() {
  SmolRuntime::spawn_local_detach(async {});
}
