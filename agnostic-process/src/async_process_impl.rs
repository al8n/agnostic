use std::{
  ffi::OsStr,
  io,
  path::Path,
  pin::Pin,
  process::{ExitStatus, Output, Stdio},
  task::{Context, Poll},
};

use super::{
  ChildStderr as ChildStderrWrapper, ChildStdin as ChildStdinWrapper,
  ChildStdout as ChildStdoutWrapper,
};
pub use async_process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use futures_util::io::{AsyncRead, AsyncWrite};

macro_rules! impl_async_write {
  ($outer:ident($($inner:ty),+$(,)?)) => {
    $(
      impl AsyncWrite for $outer<$inner> {
        fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
          Pin::new(&mut self.0).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
          Pin::new(&mut self.0).poll_flush(cx)
        }

        fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
          Pin::new(&mut self.0).poll_close(cx)
        }
      }

      #[cfg(feature = "tokio-io")]
      impl ::tokio::io::AsyncWrite for $outer<$inner> {
        fn poll_write(
          mut self: Pin<&mut Self>,
          cx: &mut Context<'_>,
          buf: &[u8],
        ) -> Poll<Result<usize, io::Error>> {
          let pinned = Pin::new(&mut self.0);
          let mut compat = super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(pinned);
          let pinned = Pin::new(&mut compat);

          ::tokio::io::AsyncWrite::poll_write(pinned, cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
          let pinned = Pin::new(&mut self.0);
          let mut compat = super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(pinned);
          let pinned = Pin::new(&mut compat);

          ::tokio::io::AsyncWrite::poll_flush(pinned, cx)
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
          let pinned = Pin::new(&mut self.0);
          let mut compat = super::io::tokio_compat::FuturesAsyncWriteCompatExt::compat_write(pinned);
          let pinned = Pin::new(&mut compat);

          ::tokio::io::AsyncWrite::poll_shutdown(pinned, cx)
        }
      }
    )+
  };
}

macro_rules! impl_async_read {
  ($outer:ident($($inner:ty),+$(,)?)) => {
    $(
      impl AsyncRead for $outer<$inner> {
        fn poll_read(
          mut self: Pin<&mut Self>,
          cx: &mut Context<'_>,
          buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
          Pin::new(&mut self.0).poll_read(cx, buf)
        }
      }

      #[cfg(feature = "tokio-io")]
      impl ::tokio::io::AsyncRead for $outer<$inner> {
        fn poll_read(
          mut self: Pin<&mut Self>,
          cx: &mut Context<'_>,
          buf: &mut tokio::io::ReadBuf,
        ) -> Poll<io::Result<()>> {
          let pinned = Pin::new(&mut self.0);
          let mut compat = super::io::tokio_compat::FuturesAsyncReadCompatExt::compat(pinned);
          let pinned = Pin::new(&mut compat);

          ::tokio::io::AsyncRead::poll_read(pinned, cx, buf)
        }
      }
    )*
  };
}

macro_rules! impl_into_stdio {
  ($($ident:ident), +$(,)?) => {
    $(
      impl super::IntoStdio for $ident {
        async fn into_stdio(self) -> std::io::Result<std::process::Stdio> {
          $ident::into_stdio(self).await
        }
      }
    )*
  };
}

impl_into_stdio!(ChildStdin, ChildStdout, ChildStderr);

converter!(ChildStdinWrapper(ChildStdin));
impl_async_write!(ChildStdinWrapper(ChildStdin, &mut ChildStdin));
converter!(ChildStdoutWrapper(ChildStdout));
impl_async_read!(ChildStdoutWrapper(ChildStdout, &mut ChildStdout));
converter!(ChildStderrWrapper(ChildStderr));
impl_async_read!(ChildStderrWrapper(ChildStderr, &mut ChildStderr));

/// Process abstraction for [`async-process`](::async_process)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsyncProcess;

impl super::Process for AsyncProcess {
  type Command = Command;
  type Child = Child;
  type Stdin = ChildStdin;
  type Stdout = ChildStdout;
  type Stderr = ChildStderr;
}

impl super::Child for Child {
  type Stdin = ChildStdin;

  type Stdout = ChildStdout;

  type Stderr = ChildStderr;

  fn stdin(&self) -> Option<ChildStdinWrapper<&Self::Stdin>> {
    self.stdin.as_ref().map(ChildStdinWrapper)
  }

  fn stdout(&self) -> Option<ChildStdoutWrapper<&Self::Stdout>> {
    self.stdout.as_ref().map(ChildStdoutWrapper)
  }

  fn stderr(&self) -> Option<ChildStderrWrapper<&Self::Stderr>> {
    self.stderr.as_ref().map(ChildStderrWrapper)
  }

  fn stdin_mut(&mut self) -> Option<ChildStdinWrapper<&mut Self::Stdin>> {
    self.stdin.as_mut().map(ChildStdinWrapper)
  }

  fn stdout_mut(&mut self) -> Option<ChildStdoutWrapper<&mut Self::Stdout>> {
    self.stdout.as_mut().map(ChildStdoutWrapper)
  }

  fn stderr_mut(&mut self) -> Option<ChildStderrWrapper<&mut Self::Stderr>> {
    self.stderr.as_mut().map(ChildStderrWrapper)
  }

  fn set_stdin(&mut self, stdin: Option<Self::Stdin>) {
    self.stdin = stdin;
  }

  fn set_stdout(&mut self, stdout: Option<Self::Stdout>) {
    self.stdout = stdout;
  }

  fn set_stderr(&mut self, stderr: Option<Self::Stderr>) {
    self.stderr = stderr;
  }

  fn take_stdin(&mut self) -> Option<ChildStdinWrapper<Self::Stdin>> {
    self.stdin.take().map(ChildStdinWrapper)
  }

  fn take_stdout(&mut self) -> Option<ChildStdoutWrapper<Self::Stdout>> {
    self.stdout.take().map(ChildStdoutWrapper)
  }

  fn take_stderr(&mut self) -> Option<ChildStderrWrapper<Self::Stderr>> {
    self.stderr.take().map(ChildStderrWrapper)
  }

  fn id(&self) -> Option<u32> {
    Some(Child::id(self))
  }

  async fn kill(&mut self) -> io::Result<()> {
    Child::kill(self)
  }

  fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
    Child::try_status(self)
  }

  async fn wait(&mut self) -> io::Result<ExitStatus> {
    Child::status(self).await
  }

  async fn wait_with_output(self) -> io::Result<Output> {
    Child::output(self).await
  }

  cfg_windows!(
    fn raw_handle(&self) -> Option<std::os::windows::io::RawHandle> {
      use std::os::windows::io::AsRawHandle;

      Some(self.as_raw_handle())
    }
  );
}

impl super::Command for Command {
  type Child = Child;

  fn new<S>(program: S) -> Self
  where
    S: AsRef<OsStr>,
  {
    Command::new(program)
  }

  fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
    Command::arg(self, arg)
  }

  fn args<I, S>(&mut self, args: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    Command::args(self, args)
  }

  fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
  where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
  {
    Command::env(self, key, val)
  }

  fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
  {
    Command::envs(self, vars)
  }

  fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
    Command::env_remove(self, key)
  }

  fn env_clear(&mut self) -> &mut Self {
    Command::env_clear(self)
  }

  fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
    Command::current_dir(self, dir)
  }

  fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
    Command::stdin(self, cfg)
  }

  fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
    Command::stdout(self, cfg)
  }

  fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
    Command::stderr(self, cfg)
  }

  fn kill_on_drop(&mut self, kill_on_drop: bool) -> &mut Self {
    Command::kill_on_drop(self, kill_on_drop)
  }

  fn spawn(&mut self) -> io::Result<Self::Child> {
    Command::spawn(self)
  }

  async fn status(&mut self) -> io::Result<ExitStatus> {
    Command::status(self).await
  }

  async fn output(&mut self) -> io::Result<Output> {
    Command::output(self).await
  }

  cfg_unix!(
    fn uid(&mut self, id: u32) -> &mut Self {
      use async_process::unix::CommandExt;
      CommandExt::uid(self, id)
    }

    fn gid(&mut self, id: u32) -> &mut Self {
      use async_process::unix::CommandExt;
      CommandExt::gid(self, id)
    }

    fn exec(&mut self) -> io::Error {
      use async_process::unix::CommandExt;
      CommandExt::exec(self)
    }

    fn arg0<S>(&mut self, arg: S) -> &mut Self
    where
      S: AsRef<OsStr>,
    {
      use async_process::unix::CommandExt;
      CommandExt::arg0(self, arg)
    }
  );

  cfg_windows!(
    fn creation_flags(&mut self, flags: u32) -> &mut Self {
      use async_process::windows::CommandExt;
      CommandExt::creation_flags(self, flags)
    }

    fn raw_arg<S: AsRef<OsStr>>(&mut self, text_to_append_as_is: S) -> &mut Self {
      use async_process::windows::CommandExt;
      CommandExt::raw_arg(self, text_to_append_as_is)
    }
  );
}
