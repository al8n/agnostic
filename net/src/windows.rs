pub use std::os::windows::io::{
  AsRawSocket, AsSocket, BorrowedSocket, FromRawSocket, IntoRawSocket, RawSocket,
};

macro_rules! socket2_fn {
  ($name:ident(
    $($field_name:ident: $field_ty:ty),*
  ) -> $return_ty:ty) => {
    #[cfg(windows)]
    #[inline]
    fn $name<T>(this: &T, $($field_name: $field_ty,)*) -> std::io::Result<$return_ty>
    where
      T: ::std::os::windows::io::AsSocket,
    {
      use socket2::SockRef;

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

fn duplicate<T, O>(this: &T) -> std::io::Result<O>
where
  T: Fd,
  O: std::os::windows::io::FromRawSocket,
{
  use std::mem::zeroed;
  use windows_sys::Win32::Networking::WinSock::{
    WSADuplicateSocketW, WSAGetLastError, WSASocketW, INVALID_SOCKET, SOCKET_ERROR,
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
