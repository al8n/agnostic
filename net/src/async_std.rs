macro_rules! impl_as_fd_async_std {
  ($name:ident.$field:ident) => {
    #[cfg(unix)]
    impl std::os::fd::AsFd for $name {
      fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        use std::os::fd::{AsRawFd, BorrowedFd};
        let raw_fd = self.$field.as_raw_fd();

        // Safety:
        // The resource pointed to by `fd` remains open for the duration of
        // the returned `BorrowedFd`, and it must not have the value `-1`.
        unsafe { BorrowedFd::borrow_raw(raw_fd) }
      }
    }

    #[cfg(windows)]
    impl std::os::windows::io::AsSocket for $name {
      fn as_socket(&self) -> &std::os::windows::io::BorrowedSocket<'_> {
        use std::os::fd::{AsRawSocket, BorrowedFd};
        use std::os::windows::io::{AsRawSocket, BorrowedSocket};
        let raw_socket = self.$field.as_raw_socket();

        // Safety:
        // The resource pointed to by raw must remain open for the duration of the returned BorrowedSocket,
        // and it must not have the value INVALID_SOCKET.
        unsafe { BorrowedSocket::borrow_raw(raw_socket) }
      }
    }
  };
}

use agnostic_lite::async_std::AsyncStdRuntime;

mod udp;
mod tcp_stream;
mod tcp_listener;

pub use udp::*;
pub use tcp_stream::*;
pub use tcp_listener::*;

/// Network abstractions for [`async-std`] runtime
/// 
/// [`async-std`]: https://docs.rs/async-std
#[derive(Debug, Default, Clone, Copy)]
pub struct Net;

impl super::Net for Net {
  type Runtime = AsyncStdRuntime;
  type TcpListener = TcpListener;
  type TcpStream = TcpStream;
  type UdpSocket = UdpSocket;
}

