#![doc = include_str!("../README.md")]
#![deny(warnings, missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]

use agnostic_lite::{cfg_async_std, cfg_smol, cfg_tokio, RuntimeLite};
use futures_util::Future;
use std::net::SocketAddr;

pub use agnostic_lite as runtime;

/// Operating system specific types and traits.
#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(unix, path = "unix.rs")]
#[cfg_attr(not(any(unix, windows)), path = "unknown.rs")]
pub mod os;

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! impl_as_raw_fd {
  ($name:ident.$field:ident) => {
    #[cfg(unix)]
    impl $crate::os::AsRawFd for $name {
      fn as_raw_fd(&self) -> $crate::os::RawFd {
        self.$field.as_raw_fd()
      }
    }

    #[cfg(windows)]
    impl $crate::os::AsRawSocket for $name {
      fn as_raw_socket(&self) -> $crate::os::RawSocket {
        self.$field.as_raw_socket()
      }
    }
  };
}

#[cfg(any(feature = "tokio", feature = "smol"))]
macro_rules! impl_as_fd {
  ($name:ident.$field:ident) => {
    #[cfg(unix)]
    impl $crate::os::AsFd for $name {
      fn as_fd(&self) -> $crate::os::BorrowedFd<'_> {
        self.$field.as_fd()
      }
    }

    #[cfg(windows)]
    impl $crate::os::AsSocket for $name {
      fn as_socket(&self) -> $crate::os::BorrowedSocket<'_> {
        self.$field.as_socket()
      }
    }
  };
}

#[cfg(any(feature = "tokio", feature = "smol"))]
macro_rules! impl_as {
  ($name:ident.$field:ident) => {
    impl_as_raw_fd!($name.$field);
    impl_as_fd!($name.$field);
  };
}

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! call {
  ($this:ident.$field:ident.$method:ident($buf:ident)) => {{
    paste::paste! {
      $this.$field.$method($buf).await
    }
  }};
  (@send_to $this:ident.$field:ident($buf:ident, $target:ident)) => {{
    paste::paste! {
      let mut addrs = $crate::ToSocketAddrs::<Self::Runtime>::to_socket_addrs(&$target).await?;
      if let ::core::option::Option::Some(addr) = addrs.next() {
        $this.$field.send_to($buf, addr).await
      } else {
        return ::core::result::Result::Err(::std::io::Error::new(
          ::std::io::ErrorKind::InvalidInput,
          "invalid socket address",
        ));
      }
    }
  }};
}

/// Traits, helpers, and type definitions for asynchronous I/O functionality.
pub use agnostic_io as io;

mod to_socket_addrs;

#[cfg(test)]
mod tests;

#[macro_use]
mod tcp;
#[macro_use]
mod udp;

pub use tcp::*;
pub use udp::*;

#[cfg(any(feature = "smol", feature = "async-std"))]
#[macro_use]
mod async_io;

cfg_tokio!(
  /// Network abstractions for [`tokio`] runtime
  ///
  /// [`tokio`]: https://docs.rs/tokio
  pub mod tokio;
);

cfg_smol!(
  /// Network abstractions for [`smol`] runtime
  ///
  /// [`smol`]: https://docs.rs/smol
  pub mod smol;
);

cfg_async_std!(
  /// Network abstractions for [`async-std`] runtime
  ///
  /// [`async-std`]: https://docs.rs/async-std
  pub mod async_std;
);

#[doc(hidden)]
#[cfg(unix)]
pub trait Fd: os::AsFd + os::AsRawFd {
  fn __as(&self) -> os::BorrowedFd<'_> {
    self.as_fd()
  }

  fn __as_raw(&self) -> os::RawFd {
    self.as_raw_fd()
  }
}

#[cfg(unix)]
impl<T> Fd for T where T: os::AsFd + os::AsRawFd {}

#[doc(hidden)]
#[cfg(windows)]
pub trait Fd: os::AsRawSocket + os::AsSocket {
  fn __as(&self) -> os::BorrowedSocket<'_> {
    self.as_socket()
  }

  fn __as_raw(&self) -> os::RawSocket {
    self.as_raw_socket()
  }
}

#[cfg(windows)]
impl<T> Fd for T where T: os::AsRawSocket + os::AsSocket {}

#[cfg(not(any(unix, windows)))]
pub trait Fd {}

#[cfg(not(any(unix, windows)))]
impl<T> Fd for T {}

/// Converts or resolves without blocking base on your async runtime to one or more `SocketAddr` values.
///
/// # DNS
///
/// Implementations of `ToSocketAddrs<R>` for string types require a DNS lookup.
pub trait ToSocketAddrs<R>: Send + Sync {
  /// Returned iterator over socket addresses which this type may correspond to.
  type Iter: Iterator<Item = SocketAddr> + Send + 'static;
  /// The future type used to resolve addresses.
  type Future: Future<Output = io::Result<Self::Iter>> + Send + 'static;

  /// Converts this object to an iterator of resolved `SocketAddr`s.
  ///
  /// The returned iterator may not actually yield any values depending on the outcome of any
  /// resolution performed.
  ///
  /// Note that this function may block a backend thread while resolution is performed.
  fn to_socket_addrs(&self) -> Self::Future
  where
    R: RuntimeLite;
}

/// An abstraction layer for the async runtime's network.
pub trait Net: Unpin + Send + Sync + 'static {
  /// The runtime type
  type Runtime: RuntimeLite;

  /// The [`TcpListener`] implementation
  type TcpListener: TcpListener<Stream = Self::TcpStream, Runtime = Self::Runtime>;
  /// The [`TcpStream`] implementation
  type TcpStream: TcpStream<Runtime = Self::Runtime>;
  /// The [`UdpSocket`] implementation
  type UdpSocket: UdpSocket<Runtime = Self::Runtime>;
}
