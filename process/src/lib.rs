#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

#[allow(unused_macros)]
macro_rules! cfg_unix {
  ($($item:item)*) => {
    $(
      #[cfg(unix)]
      #[cfg_attr(docsrs, doc(cfg(unix)))]
      $item
    )*
  }
}

#[allow(unused_macros)]
macro_rules! cfg_windows {
  ($($item:item)*) => {
    $(
      #[cfg(windows)]
      #[cfg_attr(docsrs, doc(cfg(windows)))]
      $item
    )*
  };
}

#[allow(unused_macros)]
macro_rules! cfg_linux {
  ($($item:item)*) => {
    $(
      #[cfg(target_os = "linux")]
      #[cfg_attr(docsrs, doc(cfg(target_os = "linux")))]
      $item
    )*
  };
}

mod async_process_impl;

/// Async process related implementations for `async-std` runtime.
#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub mod async_std;

/// Async process related implementations for `tokio` runtime.
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tokio;

/// Async process related implementations for `smol` runtime.
#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub mod smol;

pub use agnostic_lite::*;

use std::{
  ffi::OsStr,
  future::Future,
  io,
  path::Path,
  process::{ExitStatus, Output, Stdio},
};

/// The components of a child process.
pub struct ChildComponents<C: Child> {
  /// The stdin handle.
  pub stdin: Option<C::Stdin>,
  /// The stdout handle.
  pub stdout: Option<C::Stdout>,
  /// The stderr handle.
  pub stderr: Option<C::Stderr>,
}

/// An abstraction of a spawned child process.
pub trait Child {
  /// A handle to a child process’s standard error (stderr).
  type Stdin;
  /// A handle to a child process’s standard input (stdin).
  type Stdout;
  /// A handle to a child process’s standard output (stdout).
  type Stderr;

  /// Returns the OS-assigned process identifier associated with this child.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::Command;
  ///
  /// let mut child = Command::new("ls").spawn()?;
  /// println!("id: {}", child.id());
  /// # std::io::Result::Ok(()) });
  /// ```
  fn id(&self) -> Option<u32>;

  /// Forces the child process to exit.
  ///
  /// If the child has already exited, an [`InvalidInput`] error is returned.
  ///
  /// This is equivalent to sending a SIGKILL on Unix platforms.
  ///
  /// [`InvalidInput`]: `std::io::ErrorKind::InvalidInput`
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::Command;
  ///
  /// let mut child = Command::new("yes").spawn()?;
  /// child.kill()?;
  /// println!("exit status: {}", child.status().await?);
  /// # std::io::Result::Ok(()) });
  /// ```
  fn kill(&mut self) -> impl Future<Output = io::Result<()>> + Send;

  /// Returns the exit status if the process has exited.
  ///
  /// Unlike [`status()`][`Child::status()`], this method will not drop the stdin handle.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::Command;
  ///
  /// let mut child = Command::new("ls").spawn()?;
  ///
  /// match child.try_status()? {
  ///     None => println!("still running"),
  ///     Some(status) => println!("exited with: {}", status),
  /// }
  /// # std::io::Result::Ok(()) });
  /// ```
  fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;

  /// Drops the stdin handle and waits for the process to exit.
  ///
  /// Closing the stdin of the process helps avoid deadlocks. It ensures that the process does
  /// not block waiting for input from the parent process while the parent waits for the child to
  /// exit.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::{Command, Stdio};
  ///
  /// let mut child = Command::new("cp")
  ///     .arg("a.txt")
  ///     .arg("b.txt")
  ///     .spawn()?;
  ///
  /// println!("exit status: {}", child.status().await?);
  /// # std::io::Result::Ok(()) });
  /// ```
  fn wait(&mut self) -> impl Future<Output = io::Result<ExitStatus>> + Send;

  /// Drops the stdin handle and collects the output of the process.
  ///
  /// Closing the stdin of the process helps avoid deadlocks. It ensures that the process does
  /// not block waiting for input from the parent process while the parent waits for the child to
  /// exit.
  ///
  /// In order to capture the output of the process, [`Command::stdout()`] and
  /// [`Command::stderr()`] must be configured with [`Stdio::piped()`].
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::{Command, Stdio};
  ///
  /// let child = Command::new("ls")
  ///     .stdout(Stdio::piped())
  ///     .stderr(Stdio::piped())
  ///     .spawn()?;
  ///
  /// let out = child.output().await?;
  /// # std::io::Result::Ok(()) });
  /// ```
  fn wait_with_output(self) -> impl Future<Output = io::Result<Output>> + Send;
}

/// An abstraction of a builder for spawning processes.
pub trait Command: Sized + From<std::process::Command> {
  /// A spawned child process.
  type Child: Child;

  /// Constructs a new [`Command`] for launching `program`.
  ///
  /// The initial configuration (the working directory and environment variables) is inherited
  /// from the current process.
  ///
  /// ## Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("ls");
  /// ```
  fn new<S>(program: S) -> Self
  where
    S: AsRef<OsStr>;

  /// Adds a single argument to pass to the program.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("echo");
  /// cmd.arg("hello");
  /// cmd.arg("world");
  /// ```
  fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self;

  /// Adds multiple arguments to pass to the program.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("echo");
  /// cmd.args(&["hello", "world"]);
  /// ```
  fn args<I, S>(&mut self, args: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>;

  /// Configures an environment variable for the new process.
  ///
  /// Note that environment variable names are case-insensitive (but case-preserving) on Windows,
  /// and case-sensitive on all other platforms.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.env("PATH", "/bin");
  /// ```
  fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
  where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>;

  /// Configures multiple environment variables for the new process.
  ///
  /// Note that environment variable names are case-insensitive (but case-preserving) on Windows,
  /// and case-sensitive on all other platforms.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.envs(vec![("PATH", "/bin"), ("TERM", "xterm-256color")]);
  /// ```
  fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>;

  /// Removes an environment variable mapping.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.env_remove("PATH");
  /// ```
  fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self;

  /// Removes all environment variable mappings.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.env_clear();
  /// ```
  fn env_clear(&mut self) -> &mut Self;

  /// Configures the working directory for the new process.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::Command;
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.current_dir("/");
  /// ```
  fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;

  /// Configures the standard input (stdin) for the new process.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::{Command, Stdio};
  ///
  /// let mut cmd = Command::new("cat");
  /// cmd.stdin(Stdio::null());
  /// ```
  fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

  /// Configures the standard output (stdout) for the new process.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::{Command, Stdio};
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.stdout(Stdio::piped());
  /// ```
  fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

  /// Configures the standard error (stderr) for the new process.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::{Command, Stdio};
  ///
  /// let mut cmd = Command::new("ls");
  /// cmd.stderr(Stdio::piped());
  /// ```
  fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

  // /// Configures whether to reap the zombie process when [`Child`] is dropped.
  // ///
  // /// When the process finishes, it becomes a "zombie" and some resources associated with it
  // /// remain until [`Child::try_status()`], [`Child::status()`], or [`Child::output()`] collects
  // /// its exit code.
  // ///
  // /// If its exit code is never collected, the resources may leak forever. This crate has a
  // /// background thread named "async-process" that collects such "zombie" processes and then
  // /// "reaps" them, thus preventing the resource leaks.
  // ///
  // /// The default value of this option is `true`.
  // ///
  // /// # Examples
  // ///
  // /// ```
  // /// use async_process::{Command, Stdio};
  // ///
  // /// let mut cmd = Command::new("cat");
  // /// cmd.reap_on_drop(false);
  // /// ```
  // fn reap_on_drop(&mut self, reap_on_drop: bool) -> &mut Self;

  /// Configures whether to kill the process when [`Child`] is dropped.
  ///
  /// The default value of this option is `false`.
  ///
  /// # Examples
  ///
  /// ```
  /// use async_process::{Command, Stdio};
  ///
  /// let mut cmd = Command::new("cat");
  /// cmd.kill_on_drop(true);
  /// ```
  fn kill_on_drop(&mut self, kill_on_drop: bool) -> &mut Self;

  /// Executes the command and returns the [`Child`] handle to it.
  ///
  /// If not configured, stdin, stdout and stderr will be set to [`Stdio::inherit()`].
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::Command;
  ///
  /// let child = Command::new("ls").spawn()?;
  /// # std::io::Result::Ok(()) });
  /// ```
  fn spawn(&mut self) -> io::Result<Self::Child>;

  /// Executes the command, waits for it to exit, and returns the exit status.
  ///
  /// If not configured, stdin, stdout and stderr will be set to [`Stdio::inherit()`].
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::Command;
  ///
  /// let status = Command::new("cp")
  ///     .arg("a.txt")
  ///     .arg("b.txt")
  ///     .status()
  ///     .await?;
  /// # std::io::Result::Ok(()) });
  /// ```
  fn status(&mut self) -> impl Future<Output = io::Result<ExitStatus>> + Send;

  /// Executes the command and collects its output.
  ///
  /// If not configured, stdin will be set to [`Stdio::null()`], and stdout and stderr will be
  /// set to [`Stdio::piped()`].
  ///
  /// # Examples
  ///
  /// ```no_run
  /// # futures_lite::future::block_on(async {
  /// use async_process::Command;
  ///
  /// let output = Command::new("cat")
  ///     .arg("a.txt")
  ///     .output()
  ///     .await?;
  /// # std::io::Result::Ok(()) });
  /// ```
  fn output(&mut self) -> impl Future<Output = io::Result<Output>> + Send;

  cfg_unix!(
    /// Sets the child process's user ID. This translates to a
    /// `setuid` call in the child process. Failure in the `setuid`
    /// call will cause the spawn to fail.
    fn uid(&mut self, id: u32) -> &mut Self;

    /// Similar to `uid`, but sets the group ID of the child process. This has
    /// the same semantics as the `uid` field.
    fn gid(&mut self, id: u32) -> &mut Self;

    /// Performs all the required setup by this `Command`, followed by calling
    /// the `execvp` syscall.
    ///
    /// On success this function will not return, and otherwise it will return
    /// an error indicating why the exec (or another part of the setup of the
    /// `Command`) failed.
    ///
    /// `exec` not returning has the same implications as calling
    /// [`std::process::exit`] – no destructors on the current stack or any other
    /// thread’s stack will be run. Therefore, it is recommended to only call
    /// `exec` at a point where it is fine to not run any destructors. Note,
    /// that the `execvp` syscall independently guarantees that all memory is
    /// freed and all file descriptors with the `CLOEXEC` option (set by default
    /// on all file descriptors opened by the standard library) are closed.
    ///
    /// This function, unlike `spawn`, will **not** `fork` the process to create
    /// a new child. Like spawn, however, the default behavior for the stdio
    /// descriptors will be to inherited from the current process.
    ///
    /// # Notes
    ///
    /// The process may be in a "broken state" if this function returns in
    /// error. For example the working directory, environment variables, signal
    /// handling settings, various user/group information, or aspects of stdio
    /// file descriptors may have changed. If a "transactional spawn" is
    /// required to gracefully handle errors it is recommended to use the
    /// cross-platform `spawn` instead.
    fn exec(&mut self) -> io::Error;

    /// Set executable argument
    ///
    /// Set the first process argument, `argv[0]`, to something other than the
    /// default executable path.
    fn arg0<S>(&mut self, arg: S) -> &mut Self
    where
      S: AsRef<OsStr>;
  );

  cfg_windows!(
    /// Sets the [process creation flags][1] to be passed to `CreateProcess`.
    ///
    /// These will always be ORed with `CREATE_UNICODE_ENVIRONMENT`.
    ///
    /// [1]: https://docs.microsoft.com/en-us/windows/win32/procthread/process-creation-flags
    fn creation_flags(&mut self, flags: u32) -> &mut Self;

    /// Append literal text to the command line without any quoting or escaping.
    ///
    /// This is useful for passing arguments to applications that don't follow
    /// the standard C run-time escaping rules, such as `cmd.exe /c`.
    ///
    /// # Batch files
    ///
    /// Note the `cmd /c` command line has slightly different escaping rules than batch files
    /// themselves. If possible, it may be better to write complex arguments to a temporary
    /// `.bat` file, with appropriate escaping, and simply run that using:
    ///
    /// ```no_run
    /// # use std::process::Command;
    /// # let temp_bat_file = "";
    /// # #[allow(unused)]
    /// let output = Command::new("cmd").args(["/c", &format!("\"{temp_bat_file}\"")]).output();
    /// ```
    ///
    /// # Example
    ///
    /// Run a batch script using both trusted and untrusted arguments.
    ///
    /// ```no_run
    /// #[cfg(windows)]
    /// // `my_script_path` is a path to known bat file.
    /// // `user_name` is an untrusted name given by the user.
    /// fn run_script(
    ///     my_script_path: &str,
    ///     user_name: &str,
    /// ) -> Result<std::process::Output, std::io::Error> {
    ///     use std::io::{Error, ErrorKind};
    ///     use std::os::windows::process::CommandExt;
    ///     use std::process::Command;
    ///
    ///     // Create the command line, making sure to quote the script path.
    ///     // This assumes the fixed arguments have been tested to work with the script we're using.
    ///     let mut cmd_args = format!(r#""{my_script_path}" "--features=[a,b,c]""#);
    ///
    ///     // Make sure the user name is safe. In particular we need to be
    ///     // cautious of ascii symbols that cmd may interpret specially.
    ///     // Here we only allow alphanumeric characters.
    ///     if !user_name.chars().all(|c| c.is_alphanumeric()) {
    ///         return Err(Error::new(ErrorKind::InvalidInput, "invalid user name"));
    ///     }
    ///
    ///     // now we have validated the user name, let's add that too.
    ///     cmd_args.push_str(" --user ");
    ///     cmd_args.push_str(user_name);
    ///
    ///     // call cmd.exe and return the output
    ///     Command::new("cmd.exe")
    ///         .arg("/c")
    ///         // surround the entire command in an extra pair of quotes, as required by cmd.exe.
    ///         .raw_arg(&format!("\"{cmd_args}\""))
    ///         .output()
    /// }
    /// ````
    fn raw_arg<S: AsRef<OsStr>>(&mut self, text_to_append_as_is: S) -> &mut Self;
  );

  // TODO: feature(linux_pidfd)
  // cfg_linux!(
  //   /// Sets whether a [`PidFd`](struct@PidFd) should be created for the [`Child`]
  //   /// spawned by this [`Command`].
  //   /// By default, no pidfd will be created.
  //   ///
  //   /// The pidfd can be retrieved from the child with [`pidfd`] or [`into_pidfd`].
  //   ///
  //   /// A pidfd will only be created if it is possible to do so
  //   /// in a guaranteed race-free manner. Otherwise, [`pidfd`] will return an error.
  //   ///
  //   /// If a pidfd has been successfully created and not been taken from the `Child`
  //   /// then calls to `kill()`, `wait()` and `try_wait()` will use the pidfd
  //   /// instead of the pid. This can prevent pid recycling races, e.g.
  //   /// those  caused by rogue libraries in the same process prematurely reaping
  //   /// zombie children via `waitpid(-1, ...)` calls.
  //   ///
  //   /// [`Child`]: process::Child
  //   /// [`pidfd`]: fn@ChildExt::pidfd
  //   /// [`into_pidfd`]: ChildExt::into_pidfd
  //   fn create_pidfd(&mut self, val: bool) -> &mut Self;
  // );
}
