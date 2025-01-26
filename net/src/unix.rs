pub use rustix::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, RawFd};

macro_rules! rustix_fn {
  ($fn:ident($name:ident(
    $($field_name:ident: $field_ty:ty),*
  ) -> $return_ty:ty)) => {
    #[inline]
    pub(super) fn $name<T>(this: &T, $($field_name: $field_ty,)*) -> ::std::io::Result<$return_ty>
    where
      T: AsFd,
    {
      ::rustix::net::sockopt::$fn(this, $($field_name,)*)
        .map_err(|e| ::std::io::Error::from_raw_os_error(e.raw_os_error()))
    }
  };
}

super::cfg_async_std!(
  rustix_fn!(set_ip_ttl(set_ttl(ttl: u32) -> ()));
  rustix_fn!(get_ip_ttl(ttl() -> u32));
);

rustix_fn!(get_ipv6_v6only(only_v6() -> bool));
rustix_fn!(set_socket_recv_buffer_size(set_recv_buffer_size(size: usize) -> ()));
rustix_fn!(get_socket_recv_buffer_size(recv_buffer_size() -> usize));
rustix_fn!(set_socket_send_buffer_size(set_send_buffer_size(size: usize) -> ()));
rustix_fn!(get_socket_send_buffer_size(send_buffer_size() -> usize));
rustix_fn!(set_socket_linger(set_linger(duration: Option<std::time::Duration>) -> ()));
rustix_fn!(get_socket_linger(linger() -> Option<std::time::Duration>));

pub(super) fn shutdown<T>(this: &T, how: std::net::Shutdown) -> std::io::Result<()>
where
  T: AsFd,
{
  rustix::net::shutdown(
    this,
    match how {
      std::net::Shutdown::Read => rustix::net::Shutdown::Read,
      std::net::Shutdown::Write => rustix::net::Shutdown::Write,
      std::net::Shutdown::Both => rustix::net::Shutdown::ReadWrite,
    },
  )
  .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))
}

pub(super) fn duplicate<T, O>(this: &T) -> std::io::Result<O>
where
  T: AsFd,
  O: FromRawFd,
{
  rustix::io::dup(this)
    .map(|fd| unsafe { O::from_raw_fd(fd.into_raw_fd()) })
    .map_err(|e| std::io::Error::from_raw_os_error(e.raw_os_error()))
}
