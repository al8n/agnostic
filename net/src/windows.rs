use std::mem::zeroed;

use socket2::SockRef;
use windows_sys::Win32::Networking::WinSock::{
  WSADuplicateSocketW, WSAGetLastError, WSASocketW, INVALID_SOCKET, SOCKET_ERROR, WSAPROTOCOL_INFOW,
};

pub use std::os::windows::io::{
  AsRawSocket, AsSocket, BorrowedSocket, FromRawSocket, IntoRawSocket, RawSocket,
};

macro_rules! socket2_fn {
  ($name:ident(
    $($field_name:ident: $field_ty:ty),*
  ) -> $return_ty:ty) => {
    #[cfg(windows)]
    #[inline]
    pub(super) fn $name<T>(this: &T, $($field_name: $field_ty,)*) -> std::io::Result<$return_ty>
    where
      T: AsSocket,
    {
      SockRef::from(this).$name($($field_name,)*)
    }
  };
}

super::cfg_async_std!(
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

pub(super) fn duplicate<T, O>(this: &T) -> std::io::Result<O>
where
  T: AsSocket,
  O: FromRawSocket,
{
  let mut info: WSAPROTOCOL_INFOW = unsafe { zeroed() };
  if unsafe { WSADuplicateSocketW(this.as_raw_socket() as _, std::process::id(), &mut info) }
    == SOCKET_ERROR
  {
    return Err(std::io::Error::from_raw_os_error(unsafe {
      WSAGetLastError()
    }));
  }

  let socket = unsafe {
    WSASocketW(
      info.iAddressFamily,
      info.iSocketType,
      info.iProtocol,
      &info as *const _ as _,
      0,
      0,
    )
  };

  if socket == INVALID_SOCKET {
    return Err(std::io::Error::from_raw_os_error(unsafe {
      WSAGetLastError()
    }));
  }

  Ok(unsafe { O::from_raw_socket(socket as u64) })
}
