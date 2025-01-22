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

/// Traits, helpers, and type definitions for asynchronous I/O functionality.
pub use agnostic_io as io;

use std::{
  ffi::OsStr,
  future::Future,
  path::Path,
  process::{ExitStatus, Output, Stdio},
};

#[cfg(test)]
mod tests;

/// A trait for converting into a [`Stdio`].
pub trait IntoStdio {
  /// Convert into [`std::process::Stdio`].
  fn into_stdio(self) -> impl Future<Output = io::Result<Stdio>> + Send;
}

macro_rules! std_trait {
  (
    $(#[$attr:meta])*
    trait $name:ident: $trait:ident
  ) => {
    $(#[$attr])*
    #[cfg(unix)]
    pub trait $name: std::os::fd::AsFd + std::os::fd::AsRawFd {}

    #[cfg(unix)]
    impl<T> $name for T where T: std::os::fd::AsFd + std::os::fd::AsRawFd {}

    $(#[$attr])*
    #[cfg(not(unix))]
    pub trait $name {}

    #[cfg(not(unix))]
    impl<T> $name for T {}
  };
}

macro_rules! child_std {
  (
    $(#[$attr:meta])*
    $name:ident
  ) => {
    $(#[$attr])*
    pub struct $name<T>(T);

    impl<T> From<T> for $name<T> {
      fn from(inner: T) -> Self {
        $name(inner)
      }
    }

    impl<T> core::convert::AsRef<T> for $name<T> {
      fn as_ref(&self) -> &T {
        &self.0
      }
    }

    impl<T> core::convert::AsMut<T> for $name<T> {
      fn as_mut(&mut self) -> &mut T {
        &mut self.0
      }
    }

    impl<T> core::ops::Deref for $name<T> {
      type Target = T;

      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl<T> core::ops::DerefMut for $name<T> {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }

    impl<T> $name<T> {
      /// Creates a new instance of the wrapper.
      #[inline]
      pub const fn new(inner: T) -> Self {
        Self(inner)
      }

      /// Consumes the wrapper, returning the inner value.
      #[inline]
      pub fn into_inner(self) -> T {
        self.0
      }

      /// Gets a reference to the inner value.
      #[inline]
      pub const fn as_inner(&self) -> &T {
        &self.0
      }

      /// Gets a mutable reference to the inner value.
      #[inline]
      pub const fn as_inner_mut(&mut self) -> &mut T {
        &mut self.0
      }
    }
  };
}

macro_rules! converter {
  ($outer:ident($inner:ty)) => {
    impl From<$outer<$inner>> for $inner {
      fn from(outer: $outer<$inner>) -> Self {
        outer.0
      }
    }

    impl<'a> From<$outer<&'a mut $inner>> for &'a mut $inner {
      fn from(outer: $outer<&'a mut $inner>) -> &'a mut $inner {
        outer.0
      }
    }

    impl<'a> From<$outer<&'a $inner>> for &'a $inner {
      fn from(outer: $outer<&'a $inner>) -> &'a $inner {
        outer.0
      }
    }

    cfg_unix!(
      impl std::os::fd::AsFd for $outer<$inner> {
        fn as_fd(&self) -> std::os::fd::BorrowedFd {
          self.0.as_fd()
        }
      }

      impl std::os::fd::AsRawFd for $outer<$inner> {
        fn as_raw_fd(&self) -> std::os::raw::c_int {
          self.0.as_raw_fd()
        }
      }

      impl std::os::fd::AsFd for $outer<&mut $inner> {
        fn as_fd(&self) -> std::os::fd::BorrowedFd {
          self.0.as_fd()
        }
      }

      impl std::os::fd::AsRawFd for $outer<&mut $inner> {
        fn as_raw_fd(&self) -> std::os::raw::c_int {
          self.0.as_raw_fd()
        }
      }

      impl std::os::fd::AsFd for $outer<&$inner> {
        fn as_fd(&self) -> std::os::fd::BorrowedFd {
          self.0.as_fd()
        }
      }

      impl std::os::fd::AsRawFd for $outer<&$inner> {
        fn as_raw_fd(&self) -> std::os::raw::c_int {
          self.0.as_raw_fd()
        }
      }
    );
  };
}

std_trait!(
  /// Marker trait for the standard input (stdin) handle of a child process.
  trait Stdin: AsyncWrite
);

std_trait!(
  /// Marker trait for the standard output (stdout) handle of a child process.
  trait Stdout: AsyncRead
);

std_trait!(
  /// Marker trait for the standard error (stderr) handle of a child process.
  trait Stderr: AsyncRead
);

child_std!(
  /// A handle to a child process’s standard input (stdin).
  ChildStdin
);

child_std!(
  /// A handle to a child process’s standard output (stdout).
  ChildStdout
);

child_std!(
  /// A handle to a child process’s standard error (stderr).
  ChildStderr
);

/// An abstraction of a spawned child process.
pub trait Child {
  /// The standard input (stdin) handle type.
  type Stdin: Stdin + IntoStdio;
  /// The standard output (stdout) handle type.
  type Stdout: Stdout + IntoStdio;
  /// The standard error (stderr) handle type.
  type Stderr: Stderr + IntoStdio;

  /// Returns the stdin handle if it was configured.
  fn stdin(&self) -> Option<ChildStdin<&Self::Stdin>>;

  /// Returns the stdout handle if it was configured.
  fn stdout(&self) -> Option<ChildStdout<&Self::Stdout>>;

  /// Returns the stderr handle if it was configured.
  fn stderr(&self) -> Option<ChildStderr<&Self::Stderr>>;

  /// Returns a mutable reference to the stdin handle if it was configured.
  fn stdin_mut(&mut self) -> Option<ChildStdin<&mut Self::Stdin>>;

  /// Returns a mutable reference to the stdout handle if it was configured.
  fn stdout_mut(&mut self) -> Option<ChildStdout<&mut Self::Stdout>>;

  /// Returns a mutable reference to the stderr handle if it was configured.
  fn stderr_mut(&mut self) -> Option<ChildStderr<&mut Self::Stderr>>;

  /// Sets the stdin handle.
  fn set_stdin(&mut self, stdin: Option<Self::Stdin>);

  /// Sets the stdout handle.
  fn set_stdout(&mut self, stdout: Option<Self::Stdout>);

  /// Sets the stderr handle.
  fn set_stderr(&mut self, stderr: Option<Self::Stderr>);

  /// Takes the stdin handle.
  fn take_stdin(&mut self) -> Option<ChildStdin<Self::Stdin>>;

  /// Takes the stdout handle.
  fn take_stdout(&mut self) -> Option<ChildStdout<Self::Stdout>>;

  /// Takes the stderr handle.
  fn take_stderr(&mut self) -> Option<ChildStderr<Self::Stderr>>;

  /// Returns the OS-assigned process identifier associated with this child.
  fn id(&self) -> Option<u32>;

  /// Forces the child process to exit.
  ///
  /// If the child has already exited, an [`InvalidInput`] error is returned.
  ///
  /// This is equivalent to sending a SIGKILL on Unix platforms.
  ///
  /// [`InvalidInput`]: `std::io::ErrorKind::InvalidInput`
  fn kill(&mut self) -> impl Future<Output = io::Result<()>> + Send;

  /// Returns the exit status if the process has exited.
  ///
  /// Unlike [`wait()`][Child::wait], this method will not drop the stdin handle.
  fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;

  /// Drops the stdin handle and waits for the process to exit.
  ///
  /// Closing the stdin of the process helps avoid deadlocks. It ensures that the process does
  /// not block waiting for input from the parent process while the parent waits for the child to
  /// exit.
  fn wait(&mut self) -> impl Future<Output = io::Result<ExitStatus>> + Send;

  /// Drops the stdin handle and collects the output of the process.
  ///
  /// Closing the stdin of the process helps avoid deadlocks. It ensures that the process does
  /// not block waiting for input from the parent process while the parent waits for the child to
  /// exit.
  ///
  /// In order to capture the output of the process, [`Command::stdout()`] and
  /// [`Command::stderr()`] must be configured with [`Stdio::piped()`].
  fn wait_with_output(self) -> impl Future<Output = io::Result<Output>> + Send;

  cfg_windows!(
    /// Extracts the raw handle of the process associated with this child while
    /// it is still running. Returns `None` if the child has exited.
    fn raw_handle(&self) -> Option<std::os::windows::io::RawHandle>;
  );
}

/// An abstraction of a builder for spawning processes.
pub trait Command: Sized + From<std::process::Command> {
  /// A spawned child process.
  type Child: Child;

  /// Constructs a new [`Command`] for launching `program`.
  ///
  /// The initial configuration (the working directory and environment variables) is inherited
  /// from the current process.
  fn new<S>(program: S) -> Self
  where
    S: AsRef<OsStr>;

  /// Adds a single argument to pass to the program.
  fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self;

  /// Adds multiple arguments to pass to the program.
  fn args<I, S>(&mut self, args: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>;

  /// Configures an environment variable for the new process.
  ///
  /// Note that environment variable names are case-insensitive (but case-preserving) on Windows,
  /// and case-sensitive on all other platforms.
  fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
  where
    K: AsRef<OsStr>,
    V: AsRef<OsStr>;

  /// Configures multiple environment variables for the new process.
  ///
  /// Note that environment variable names are case-insensitive (but case-preserving) on Windows,
  /// and case-sensitive on all other platforms.
  fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>;

  /// Removes an environment variable mapping.
  fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self;

  /// Removes all environment variable mappings.
  fn env_clear(&mut self) -> &mut Self;

  /// Configures the working directory for the new process.
  fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self;

  /// Configures the standard input (stdin) for the new process.
  fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

  /// Configures the standard output (stdout) for the new process.
  fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self;

  /// Configures the standard error (stderr) for the new process.
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
  fn kill_on_drop(&mut self, kill_on_drop: bool) -> &mut Self;

  /// Executes the command and returns the [`Child`] handle to it.
  ///
  /// If not configured, stdin, stdout and stderr will be set to [`Stdio::inherit()`].
  fn spawn(&mut self) -> io::Result<Self::Child>;

  /// Executes the command, waits for it to exit, and returns the exit status.
  ///
  /// If not configured, stdin, stdout and stderr will be set to [`Stdio::inherit()`].
  fn status(&mut self) -> impl Future<Output = io::Result<ExitStatus>> + Send;

  /// Executes the command and collects its output.
  ///
  /// If not configured, stdin will be set to [`Stdio::null()`], and stdout and stderr will be
  /// set to [`Stdio::piped()`].
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

/// Trait for spawning a child process.
pub trait Process {
  /// The command type.
  type Command: Command<Child = Self::Child>;
  /// The child process type.
  type Child: Child<Stdin = Self::Stdin, Stdout = Self::Stdout, Stderr = Self::Stderr>;
  /// The standard input (stdin) handle type.
  type Stdin: Stdin + IntoStdio;
  /// The standard output (stdout) handle type.
  type Stdout: Stdout + IntoStdio;
  /// The standard error (stderr) handle type.
  type Stderr: Stderr + IntoStdio;
}

#[cfg(feature = "async-process")]
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
