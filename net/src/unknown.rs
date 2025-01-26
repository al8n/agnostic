macro_rules! unknown_fn {
  ($name:ident(
    $($field_name:ident: $field_ty:ty),*
  ) -> $return_ty:ty) => {
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
  unknown_fn!(set_ttl(ttl: u32) -> ());
  unknown_fn!(ttl() -> u32);
);

unknown_fn!(shutdown(how: std::net::Shutdown) -> ());
unknown_fn!(only_v6() -> bool);
unknown_fn!(linger() -> Option<std::time::Duration>);
unknown_fn!(set_linger(duration: Option<std::time::Duration>) -> ());
unknown_fn!(recv_buffer_size() -> usize);
unknown_fn!(set_recv_buffer_size(size: usize) -> ());
unknown_fn!(send_buffer_size() -> usize);
unknown_fn!(set_send_buffer_size(size: usize) -> ());

fn duplicate<T: As, O>(_this: &T) -> std::io::Result<O> {
  panic!("unsupported platform")
}
