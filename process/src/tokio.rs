use std::{
  ffi::OsStr,
  io,
  path::Path,
  process::{ExitStatus, Output, Stdio},
};


use ::tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};

impl super::Child for Child {
  type Stdin = ChildStdin;

  type Stdout = ChildStdout;

  type Stderr = ChildStderr;

  fn id(&self) -> Option<u32> {
    Child::id(self)
  }

  async fn kill(&mut self) -> io::Result<()> {
    Child::kill(self).await
  }

  fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
    Child::try_wait(self)
  }

  async fn wait(&mut self) -> io::Result<ExitStatus> {
    Child::wait(self).await
  }

  async fn wait_with_output(self) -> io::Result<Output> {
    Child::wait_with_output(self).await
  }
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
      Command::uid(self, id)
    }

    fn gid(&mut self, id: u32) -> &mut Self {
      Command::gid(self, id)
    }

    fn exec(&mut self) -> io::Error {
      use std::os::unix::process::CommandExt;

      self.as_std_mut().exec()
    }

    fn arg0<S>(&mut self, arg: S) -> &mut Self
    where
      S: AsRef<OsStr>,
    {
      Command::arg0(self, arg)
    }
  );

  cfg_windows!(
    fn creation_flags(&mut self, flags: u32) -> &mut Self {
      Command::creation_flags(self, flags)
    }

    fn raw_arg<S: AsRef<OsStr>>(&mut self, text_to_append_as_is: S) -> &mut Self {
      Command::raw_arg(self, text_to_append_as_is)
    }
  );
}
