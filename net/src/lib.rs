use std::net::SocketAddr;

use agnostic_lite::{cfg_async_std, cfg_smol, cfg_tokio, RuntimeLite};
use futures_util::Future;

macro_rules! impl_as_raw_fd {
  ($name:ident.$field:ident) => {
    #[cfg(unix)]
    impl std::os::fd::AsRawFd for $name {
      fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.$field.as_raw_fd()
      }
    }

    #[cfg(windows)]
    impl std::os::windows::io::AsRawSocket for $name {
      fn as_raw_socket(&self) -> std::os::windows::io::RawSocket {
        self.$field.as_raw_socket()
      }
    }
  };
}

macro_rules! impl_as_fd {
  ($name:ident.$field:ident) => {
    #[cfg(unix)]
    impl std::os::fd::AsFd for $name {
      fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.$field.as_fd()
      }
    }

    #[cfg(windows)]
    impl std::os::windows::io::AsSocket for $name {
      fn as_socket(&self) -> &std::os::windows::io::Socket {
        self.$field.as_socket()
      }
    }
  };
}

macro_rules! impl_as {
  ($name:ident.$field:ident) => {
    impl_as_raw_fd!($name.$field);
    impl_as_fd!($name.$field);
  };
}

/// Traits, helpers, and type definitions for asynchronous I/O functionality.
pub use agnostic_io as io;

mod to_socket_addrs;

#[cfg(test)]
mod tests;

/// Agnostic async DNS provider.
#[cfg(feature = "dns")]
#[cfg_attr(docsrs, doc(cfg(feature = "dns")))]
pub mod dns;

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

mod tcp;
mod udp;

pub use tcp::*;
pub use udp::*;

#[doc(hidden)]
#[cfg(unix)]
pub trait As: std::os::fd::AsFd + std::os::fd::AsRawFd {
  fn __as(&self) -> std::os::fd::BorrowedFd<'_> {
    self.as_fd()
  }

  fn __as_raw(&self) -> std::os::fd::RawFd {
    self.as_raw_fd()
  }
}

#[cfg(unix)]
impl<T> As for T where T: std::os::fd::AsFd + std::os::fd::AsRawFd {}

#[doc(hidden)]
#[cfg(windows)]
pub trait As: std::os::windows::io::AsRawSocket + std::os::windows::io::AsSocket {
  fn __as(&self) -> &std::os::windows::io::Socket {
    self.as_socket()
  }

  fn __as_raw(&self) -> std::os::windows::io::RawSocket {
    self.as_raw_socket()
  }
}

#[cfg(windows)]
impl<T> As for T where T: std::os::windows::io::AsRawSocket + std::os::windows::io::AsSocket {}

#[cfg(not(any(unix, windows)))]
pub trait As {}

#[cfg(not(any(unix, windows)))]
impl<T> As for T {}

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

macro_rules! socket2_fn {
  ($name:ident(
    $($field_name:ident: $field_ty:ty),*
  ) -> $return_ty:ty) => {
    #[cfg(unix)]
    #[inline]
    fn $name<T>(this: &T, $($field_name: $field_ty,)*) -> io::Result<$return_ty>
    where
      T: $crate::As,
    {
      use socket2::SockRef;

      SockRef::from(this).$name($($field_name,)*)
    }

    #[cfg(windows)]
    #[inline]
    fn $name<T>(this: &T, $($field_name: $field_ty,)*) -> io::Result<$return_ty>
    where
      T: $crate::As,
    {
      use socket2::SockRef;

      SockRef::from(this).$name($($field_name,)*)
    }

    #[cfg(not(any(unix, windows)))]
    #[inline]
    fn $name<T>(_this: &T, $(paste::paste!{ [< _ $field_name >] }: $field_ty,)*) -> io::Result<$return_ty>
    where
      T: $crate::As,
    {
      panic!("unsupported platform")
    }
  };
}

cfg_async_std!(
  socket2_fn!(set_ttl(ttl: u32) -> ());
  socket2_fn!(ttl() -> u32);
);

socket2_fn!(shutdown(how: std::net::Shutdown) -> ());
socket2_fn!(only_v6() -> bool);
socket2_fn!(linger() -> Option<std::time::Duration>);
socket2_fn!(set_linger(duration: Option<std::time::Duration>) -> ());
socket2_fn!(recv_buffer_size() -> usize);
socket2_fn!(set_recv_buffer_size(size: usize) -> ());
socket2_fn!(send_buffer_size() -> usize);
socket2_fn!(set_send_buffer_size(size: usize) -> ());
socket2_fn!(set_read_timeout(duration: Option<std::time::Duration>) -> ());
socket2_fn!(set_write_timeout(duration: Option<std::time::Duration>) -> ());
socket2_fn!(read_timeout() -> Option<std::time::Duration>);
socket2_fn!(write_timeout() -> Option<std::time::Duration>);

#[cfg(unix)]
fn duplicate<T: As>(this: &T) -> std::io::Result<socket2::Socket> {
  use libc::dup;
  use std::os::fd::FromRawFd;

  let fd = unsafe { dup(this.__as_raw()) };
  if fd < 0 {
    return Err(std::io::Error::last_os_error());
  }

  Ok(unsafe { socket2::Socket::from_raw_fd(fd) })
}

#[cfg(windows)]
fn duplicate<T: As>(this: &T) -> std::io::Result<socket2::Socket> {
  use std::mem::zeroed;
  use windows_sys::Win32::Networking::WinSock::{
    WSADuplicateSocketW, WSAGetLastError, WSASocket, INVALID_SOCKET, SOCKET_ERROR,
    WSAPROTOCOL_INFOW,
  };

  let mut info: WSAPROTOCOL_INFOW = unsafe { zeroed() };
  if unsafe { WSADuplicateSocketW(this.__as_raw() as _, std::process::id(), &mut info) }
    == SOCKET_ERROR
  {
    return Err(std::io::Error::from_raw_os_error(unsafe {
      WSAGetLastError()
    }));
  }

  let socket = unsafe {
    WSASocket(
      info.iAddressFamily,
      info.iSocketType,
      info.iProtocol,
      Some(&info),
      0,
      0,
    )
  };

  if socket == INVALID_SOCKET {
    return Err(std::io::Error::from_raw_os_error(unsafe {
      WSAGetLastError()
    }));
  }

  Ok(unsafe { socket2::Socket::from_raw_socket(socket) })
}

#[cfg(not(any(unix, windows)))]
fn duplicate<T: As>(_this: &T) -> std::io::Result<socket2::Socket> {
  panic!("unsupported platform")
}
