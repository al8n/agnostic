pub use std::io::{Error, ErrorKind, Result};

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
pub use tokio::io as tokio_io;
#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio-io")))]
pub use tokio_util::compat as tokio_compat;

pub use futures_util::io as futures_io;

/// Marker trait for types that implement `AsyncRead` and `AsyncWrite`.
///
/// When the `tokio` feature is enabled, this trait is implemented for types that implement
/// `tokio::io::AsyncRead`, `tokio::io::AsyncWrite`, `futures::io::AsyncRead` and `futures::io::AsyncWrite`.
///
/// When the `tokio` feature is disabled, this trait is implemented for types that implement
/// `futures::io::AsyncRead` and `futures::io::AsyncWrite`.
#[cfg(not(feature = "tokio"))]
pub trait AsyncReadWrite: futures_io::AsyncRead + futures_io::AsyncWrite {}

#[cfg(not(feature = "tokio"))]
impl<T: futures_io::AsyncRead + futures_io::AsyncWrite> AsyncReadWrite for T {}

/// Marker trait for types that implement `AsyncRead` and `AsyncWrite`.
///
/// When the `tokio` feature is enabled, this trait is implemented for types that implement
/// `tokio::io::AsyncRead`, `tokio::io::AsyncWrite`, `futures::io::AsyncRead` and `futures::io::AsyncWrite`.
///
/// When the `tokio` feature is disabled, this trait is implemented for types that implement
/// `futures::io::AsyncRead` and `futures::io::AsyncWrite`.
#[cfg(feature = "tokio")]
pub trait AsyncReadWrite:
  ::tokio::io::AsyncRead + ::tokio::io::AsyncWrite + futures_io::AsyncRead + futures_io::AsyncWrite
{
}

#[cfg(feature = "tokio")]
impl<
  T: ::tokio::io::AsyncRead + ::tokio::io::AsyncWrite + futures_io::AsyncRead + futures_io::AsyncWrite,
> AsyncReadWrite for T
{
}

/// Marker trait for types that implement `AsyncRead`.
///
/// When the `tokio` feature is enabled, this trait is implemented for types that implement
/// `tokio::io::AsyncRead` and `futures::io::AsyncRead`.
///
/// When the `tokio` feature is disabled, this trait is implemented for types that implement
/// `futures::io::AsyncRead`.
#[cfg(not(feature = "tokio"))]
pub trait AsyncRead: futures_io::AsyncRead {}

#[cfg(not(feature = "tokio"))]
impl<T: futures_io::AsyncRead> AsyncRead for T {}

/// Marker trait for types that implement `AsyncRead`.
///
/// When the `tokio` feature is enabled, this trait is implemented for types that implement
/// `tokio::io::AsyncRead` and `futures::io::AsyncRead`.
///
/// When the `tokio` feature is disabled, this trait is implemented for types that implement
/// `futures::io::AsyncRead`.
#[cfg(feature = "tokio")]
pub trait AsyncRead: ::tokio::io::AsyncRead + futures_io::AsyncRead {}

#[cfg(feature = "tokio")]
impl<T: ::tokio::io::AsyncRead + futures_io::AsyncRead> AsyncRead for T {}

/// Marker trait for types that implement `AsyncWrite`.
///
/// When the `tokio` feature is enabled, this trait is implemented for types that implement
/// `tokio::io::AsyncWrite` and `futures::io::AsyncWrite`.
///
/// When the `tokio` feature is disabled, this trait is implemented for types that implement
/// `futures::io::AsyncWrite`.
#[cfg(not(feature = "tokio"))]
pub trait AsyncWrite: futures_io::AsyncWrite {}

#[cfg(not(feature = "tokio"))]
impl<T: futures_io::AsyncWrite> AsyncWrite for T {}

/// Marker trait for types that implement `AsyncWrite`.
///
/// When the `tokio` feature is enabled, this trait is implemented for types that implement
/// `tokio::io::AsyncWrite` and `futures::io::AsyncWrite`.
///
/// When the `tokio` feature is disabled, this trait is implemented for types that implement
/// `futures::io::AsyncWrite`.
#[cfg(feature = "tokio")]
pub trait AsyncWrite: ::tokio::io::AsyncWrite + futures_io::AsyncWrite {}

#[cfg(feature = "tokio")]
impl<T: ::tokio::io::AsyncWrite + futures_io::AsyncWrite> AsyncWrite for T {}
