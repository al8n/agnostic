pub use embedded_io_async;

/// The error type used by the `no_std` path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
  kind: ErrorKind,
}

impl Error {
  /// Creates a new error with the provided kind.
  #[inline]
  pub const fn new(kind: ErrorKind) -> Self {
    Self { kind }
  }

  /// Returns the kind of this error.
  #[inline]
  pub const fn kind(&self) -> ErrorKind {
    self.kind
  }
}

/// Error categories for the `no_std` path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
  /// The operation is not valid for the provided input.
  InvalidInput,
  /// The requested operation is unsupported.
  Unsupported,
  /// The connection is not established.
  NotConnected,
  /// The peer reset the connection.
  ConnectionReset,
  /// The operation timed out.
  TimedOut,
  /// A catch-all category for errors without a more specific kind.
  Other,
}

/// Result type used by the `no_std` path.
pub type Result<T> = core::result::Result<T, Error>;

/// Marker trait for types that implement asynchronous read and write.
pub trait AsyncReadWrite: embedded_io_async::Read + embedded_io_async::Write {}

impl<T: embedded_io_async::Read + embedded_io_async::Write> AsyncReadWrite for T {}

/// Marker trait for types that implement asynchronous read.
pub trait AsyncRead: embedded_io_async::Read {}

impl<T: embedded_io_async::Read> AsyncRead for T {}

/// Marker trait for types that implement asynchronous write.
pub trait AsyncWrite: embedded_io_async::Write {}

impl<T: embedded_io_async::Write> AsyncWrite for T {}
