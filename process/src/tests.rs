// Modified based on https://github.com/smol-rs/async-process/tree/master/tests

use std::{
  env, io,
  process::{Output, Stdio},
};

use agnostic_io::AsyncRead;
use futures_util::AsyncReadExt;

use crate::{ChildStdin, ChildStdout, Stdout};

use super::{Child, Command, IntoStdio, Process};

macro_rules! test_suites {
  ($runtime:ident) => {
    paste::paste! {
      #[test]
      fn smoke() {
        run(super::smoke::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn smoke_failure() {
        run(super::smoke_failure::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn exit_reported_right() {
        run(super::exit_reported_right::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn signal_reported_right() {
        run(super::signal_reported_right::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn stdout_works() {
        run(super::stdout_works::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn process_status() {
        run(super::process_status::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn process_output_fail_to_start() {
        run(super::process_output_fail_to_start::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn process_output_output() {
        run(super::process_output_output::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn process_output_error() {
        run(super::process_output_error::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn finish_once() {
        run(super::finish_once::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn finish_twice() {
        run(super::finish_twice::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn wait_with_output_once() {
        run(super::wait_with_output_once::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn override_env() {
        run(super::override_env::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn add_to_env() {
        run(super::add_to_env::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn capture_env_at_spawn() {
        run(super::capture_env_at_spawn::<[<$runtime:camel Process>]>());
      }

      #[test]
      fn child_status_preserved_with_kill_on_drop() {
        run(super::child_status_preserved_with_kill_on_drop::<[<$runtime:camel Process>]>());
      }

      #[test]
      #[cfg(unix)]
      fn stdin_works() {
        run(super::stdin_works::<[<$runtime:camel Process>]>());
      }

      #[test]
      #[cfg(not(windows))]
      fn set_current_dir_works() {
        run(super::set_current_dir_works::<[<$runtime:camel Process>]>());
      }

      #[test]
      #[cfg(windows)]
      fn child_as_raw_handle() {
        run(super::child_as_raw_handle::<[<$runtime:camel Process>]>());
      }

      #[test]
      #[cfg(unix)]
      fn into_inner() {
        run(super::into_inner::<[<$runtime:camel Process>]>());
      }

      #[test]
      #[cfg(unix)]
      fn spawn_multiple_with_stdio() {
        run(super::spawn_multiple_with_stdio::<[<$runtime:camel Process>]>());
      }

      #[test]
      #[cfg(any(windows, unix))]
      fn sleep() {
        run(super::sleep::<[<$runtime:camel Process>]>());
      }
    }
  };
}

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "smol")]
mod smol;

#[cfg(feature = "async-std")]
mod async_std;

async fn smoke<P: Process>() {
  let p = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "exit 0"])
      .spawn()
  } else {
    <P::Command as Command>::new("true").spawn()
  };
  assert!(p.is_ok());
  let mut p = p.unwrap();
  assert!(p.wait().await.unwrap().success());
}

async fn smoke_failure<P: Process>() {
  assert!(
    <P::Command as Command>::new("if-this-is-a-binary-then-the-world-has-ended")
      .spawn()
      .is_err()
  );
}

async fn exit_reported_right<P: Process>() {
  let p = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "exit 1"])
      .spawn()
  } else {
    <P::Command as Command>::new("false").spawn()
  };
  assert!(p.is_ok());
  let mut p = p.unwrap();
  assert!(p.wait().await.unwrap().code() == Some(1));
  drop(p.wait().await);
}

#[cfg(unix)]
async fn signal_reported_right<P: Process>() {
  use std::os::unix::process::ExitStatusExt;

  let mut p = <P::Command as Command>::new("/bin/sh")
    .arg("-c")
    .arg("read a")
    .stdin(Stdio::piped())
    .spawn()
    .unwrap();
  p.kill().await.unwrap();
  match p.wait().await.unwrap().signal() {
    Some(9) => {}
    result => panic!("not terminated by signal 9 (instead, {:?})", result),
  }
}

async fn run_output<P>(mut cmd: P::Command) -> String
where
  P: Process,
  for<'a> ChildStdout<&'a P::Stdout>: Stdout,
  for<'a> ChildStdout<&'a mut P::Stdout>: AsyncRead + Stdout,
{
  let p = cmd.spawn();
  assert!(p.is_ok());
  let mut p = p.unwrap();
  assert!(p.stdout().is_some());
  let mut ret = String::new();
  p.stdout_mut()
    .unwrap()
    .read_to_string(&mut ret)
    .await
    .unwrap();
  assert!(p.wait().await.unwrap().success());
  ret
}

async fn stdout_works<P>()
where
  P: Process,
  for<'a> ChildStdout<&'a P::Stdout>: Stdout,
  for<'a> ChildStdout<&'a mut P::Stdout>: AsyncRead + Stdout,
{
  if cfg!(target_os = "windows") {
    let mut cmd = <P::Command as Command>::new("cmd");
    cmd.args(["/C", "echo foobar"]).stdout(Stdio::piped());
    assert_eq!(run_output::<P>(cmd).await, "foobar\r\n");
  } else {
    let mut cmd = <P::Command as Command>::new("echo");
    cmd.arg("foobar").stdout(Stdio::piped());
    assert_eq!(run_output::<P>(cmd).await, "foobar\n");
  }
}

#[cfg(not(windows))]
async fn set_current_dir_works<P>()
where
  P: Process,
  for<'a> ChildStdout<&'a P::Stdout>: Stdout,
  for<'a> ChildStdout<&'a mut P::Stdout>: AsyncRead + Stdout,
{
  let mut cmd = <P::Command as Command>::new("/bin/sh");
  cmd
    .arg("-c")
    .arg("pwd")
    .current_dir("/")
    .stdout(Stdio::piped());
  assert_eq!(run_output::<P>(cmd).await, "/\n");
}

#[cfg(not(windows))]
async fn stdin_works<P>()
where
  P: Process,
  for<'a> ChildStdout<&'a P::Stdout>: Stdout,
  for<'a> ChildStdout<&'a mut P::Stdout>: AsyncRead + Stdout,
  for<'a> ChildStdin<&'a mut P::Stdin>: agnostic_io::AsyncWrite,
{
  use futures_util::AsyncWriteExt;

  let mut p = <P::Command as Command>::new("/bin/sh")
    .arg("-c")
    .arg("read line; echo $line")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  #[allow(clippy::unused_io_amount)]
  p.stdin_mut()
    .unwrap()
    .write("foobar".as_bytes())
    .await
    .unwrap();
  drop(p.take_stdin());
  let mut out = String::new();
  p.stdout_mut()
    .unwrap()
    .read_to_string(&mut out)
    .await
    .unwrap();
  assert!(p.wait().await.unwrap().success());
  assert_eq!(out, "foobar\n");
}

async fn process_status<P>()
where
  P: Process,
{
  let mut status = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "exit 1"])
      .status()
      .await
      .unwrap()
  } else {
    <P::Command as Command>::new("false")
      .status()
      .await
      .unwrap()
  };
  assert!(status.code() == Some(1));

  status = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "exit 0"])
      .status()
      .await
      .unwrap()
  } else {
    <P::Command as Command>::new("true").status().await.unwrap()
  };
  assert!(status.success());
}

async fn process_output_fail_to_start<P>()
where
  P: Process,
{
  match <P::Command as Command>::new("/no-binary-by-this-name-should-exist")
    .output()
    .await
  {
    Err(e) => assert_eq!(e.kind(), io::ErrorKind::NotFound),
    Ok(..) => panic!(),
  }
}

async fn process_output_output<P>()
where
  P: Process,
{
  let Output {
    status,
    stdout,
    stderr,
  } = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "echo hello"])
      .output()
      .await
      .unwrap()
  } else {
    <P::Command as Command>::new("echo")
      .arg("hello")
      .output()
      .await
      .unwrap()
  };
  let output_str = core::str::from_utf8(&stdout).unwrap();

  assert!(status.success());
  assert_eq!(output_str.trim().to_string(), "hello");
  assert_eq!(stderr, Vec::new());
}

async fn process_output_error<P>()
where
  P: Process,
{
  let Output {
    status,
    stdout,
    stderr,
  } = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "mkdir ."])
      .output()
      .await
      .unwrap()
  } else {
    <P::Command as Command>::new("mkdir")
      .arg("./")
      .output()
      .await
      .unwrap()
  };

  assert!(status.code() == Some(1));
  assert_eq!(stdout, Vec::new());
  assert!(!stderr.is_empty());
}

async fn finish_once<P>()
where
  P: Process,
{
  let mut prog = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "exit 1"])
      .spawn()
      .unwrap()
  } else {
    <P::Command as Command>::new("false").spawn().unwrap()
  };
  assert!(prog.wait().await.unwrap().code() == Some(1));
}

async fn finish_twice<P>()
where
  P: Process,
{
  let mut prog = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "exit 1"])
      .spawn()
      .unwrap()
  } else {
    <P::Command as Command>::new("false").spawn().unwrap()
  };
  assert!(prog.wait().await.unwrap().code() == Some(1));
  assert!(prog.wait().await.unwrap().code() == Some(1));
}

async fn wait_with_output_once<P>()
where
  P: Process,
{
  let prog = if cfg!(target_os = "windows") {
    <P::Command as Command>::new("cmd")
      .args(["/C", "echo hello"])
      .stdout(Stdio::piped())
      .spawn()
      .unwrap()
  } else {
    <P::Command as Command>::new("echo")
      .arg("hello")
      .stdout(Stdio::piped())
      .spawn()
      .unwrap()
  };

  let Output {
    status,
    stdout,
    stderr,
  } = prog.wait_with_output().await.unwrap();
  let output_str = core::str::from_utf8(&stdout).unwrap();

  assert!(status.success());
  assert_eq!(output_str.trim().to_string(), "hello");
  assert_eq!(stderr, Vec::new());
}

#[cfg(all(unix, not(target_os = "android")))]
pub fn env_cmd<P: Process>() -> P::Command {
  <P::Command as Command>::new("env")
}

#[cfg(target_os = "android")]
pub fn env_cmd<P: Process>() -> P::Command {
  let mut cmd = <P::Command as Command>::new("/system/bin/sh");
  cmd.arg("-c").arg("set");
  cmd
}

#[cfg(windows)]
pub fn env_cmd<P: Process>() -> P::Command {
  let mut cmd = <P::Command as Command>::new("cmd");
  cmd.arg("/c").arg("set");
  cmd
}

async fn override_env<P: Process>() {
  // In some build environments (such as chrooted Nix builds), `env` can
  // only be found in the explicitly-provided PATH env variable, not in
  // default places such as /bin or /usr/bin. So we need to pass through
  // PATH to our sub-process.
  let mut cmd = env_cmd::<P>();
  cmd.env_clear().env("RUN_TEST_NEW_ENV", "123");
  if let Some(p) = env::var_os("PATH") {
    cmd.env("PATH", p);
  }
  let result = cmd.output().await.unwrap();
  let output = String::from_utf8_lossy(&result.stdout).to_string();

  assert!(
    output.contains("RUN_TEST_NEW_ENV=123"),
    "didn't find RUN_TEST_NEW_ENV inside of:\n\n{}",
    output
  );
}

async fn add_to_env<P: Process>() {
  let result = env_cmd::<P>()
    .env("RUN_TEST_NEW_ENV", "123")
    .output()
    .await
    .unwrap();
  let output = String::from_utf8_lossy(&result.stdout).to_string();

  assert!(
    output.contains("RUN_TEST_NEW_ENV=123"),
    "didn't find RUN_TEST_NEW_ENV inside of:\n\n{}",
    output
  );
}

async fn capture_env_at_spawn<P: Process>() {
  let mut cmd = env_cmd::<P>();
  cmd.env("RUN_TEST_NEW_ENV1", "123");

  // This variable will not be present if the environment has already
  // been captured above.
  env::set_var("RUN_TEST_NEW_ENV2", "456");
  let result = cmd.output().await.unwrap();
  env::remove_var("RUN_TEST_NEW_ENV2");

  let output = String::from_utf8_lossy(&result.stdout).to_string();

  assert!(
    output.contains("RUN_TEST_NEW_ENV1=123"),
    "didn't find RUN_TEST_NEW_ENV1 inside of:\n\n{}",
    output
  );
  assert!(
    output.contains("RUN_TEST_NEW_ENV2=456"),
    "didn't find RUN_TEST_NEW_ENV2 inside of:\n\n{}",
    output
  );
}

#[cfg(unix)]
async fn child_status_preserved_with_kill_on_drop<P: Process>() {
  let p = <P::Command as Command>::new("true")
    .kill_on_drop(true)
    .spawn()
    .unwrap();

  // Calling output, since it takes ownership of the child
  // Child::status would work, but without special care,
  // dropping p inside of output would kill the subprocess early,
  // and report the wrong exit status
  let res = p.wait_with_output().await;
  assert!(res.unwrap().status.success());
}

#[cfg(windows)]
async fn child_as_raw_handle<P: Process>() {
  use std::os::windows::io::AsRawHandle;
  use windows_sys::Win32::System::Threading::GetProcessId;

  let p = <P::Command as Command>::new("cmd.exe")
    .arg("/C")
    .arg("pause")
    .kill_on_drop(true)
    .spawn()
    .unwrap();

  let std_pid = p.id();
  assert!(std_pid > 0);

  let handle = p.raw_handle();

  // We verify that we have the correct handle by obtaining the PID
  // with the Windows API rather than via std:
  let win_pid = unsafe { GetProcessId(handle as _) };
  assert_eq!(win_pid, std_pid);
}

#[cfg(unix)]
async fn spawn_multiple_with_stdio<P: Process>() {
  let mut cmd = <P::Command as Command>::new("/bin/sh");
  cmd
    .arg("-c")
    .arg("echo foo; echo bar 1>&2")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  let p1 = cmd.spawn().unwrap();
  let out1 = p1.wait_with_output().await.unwrap();
  assert_eq!(out1.stdout, b"foo\n");
  assert_eq!(out1.stderr, b"bar\n");

  let p2 = cmd.spawn().unwrap();
  let out2 = p2.wait_with_output().await.unwrap();
  assert_eq!(out2.stdout, b"foo\n");
  assert_eq!(out2.stderr, b"bar\n");
}

#[cfg(unix)]
async fn into_inner<P>()
where
  P: Process,
  ChildStdout<P::Stdout>: AsyncRead + Unpin,
{
  use std::process::Stdio;
  use std::str::from_utf8;

  let mut ls_child = <P::Command as Command>::new("cat")
    .arg("Cargo.toml")
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  let stdout = ls_child.take_stdout().unwrap();
  let stdio: Stdio = stdout.into_inner().into_stdio().await.unwrap();

  let mut echo_child = <P::Command as Command>::new("grep")
    .arg("async")
    .stdin(stdio)
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();

  let mut buf = vec![];
  let mut stdout = echo_child.take_stdout().unwrap();

  stdout.read_to_end(&mut buf).await.unwrap();
  dbg!(from_utf8(&buf).unwrap_or(""));
}

#[cfg(unix)]
async fn sleep<P: Process>() {
  let status = <P::Command as Command>::new("sleep")
    .arg("1")
    .status()
    .await
    .unwrap();
  assert!(status.success());
}

#[cfg(windows)]
async fn sleep<P: Process>() {
  let status = <P::Command as Command>::new("ping")
    .args(["-n", "5", "127.0.0.1"])
    .status()
    .await
    .unwrap();
  assert!(status.success());
}
