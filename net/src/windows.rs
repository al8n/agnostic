macro_rules! socket2_fn {
  ($name:ident(
    $($field_name:ident: $field_ty:ty),*
  ) -> $return_ty:ty) => {
    #[cfg(windows)]
    #[inline]
    fn $name<T>(this: &T, $($field_name: $field_ty,)*) -> std::io::Result<$return_ty>
    where
      T: $crate::Fd,
    {
      use socket2::SockRef;

      SockRef::from(this).$name($($field_name,)*)
    }

    #[cfg(not(any(unix, windows)))]
    #[inline]
    fn $name<T>(_this: &T, $(paste::paste!{ [< _ $field_name >] }: $field_ty,)*) -> std::io::Result<$return_ty>
    where
      T: $crate::Fd,
    {
      panic!("unsupported platform")
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
