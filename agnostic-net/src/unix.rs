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

rustix_fn!(ipv6_v6only(only_v6() -> bool));
rustix_fn!(set_socket_recv_buffer_size(set_recv_buffer_size(size: usize) -> ()));
rustix_fn!(socket_recv_buffer_size(recv_buffer_size() -> usize));
rustix_fn!(set_socket_send_buffer_size(set_send_buffer_size(size: usize) -> ()));
rustix_fn!(socket_send_buffer_size(send_buffer_size() -> usize));
rustix_fn!(set_socket_linger(set_linger(duration: Option<std::time::Duration>) -> ()));
rustix_fn!(socket_linger(linger() -> Option<std::time::Duration>));

agnostic_lite::cfg_smol! {
  rustix_fn!(set_tcp_nodelay(set_nodelay(nodelay: bool) -> ()));
  rustix_fn!(tcp_nodelay(nodelay() -> bool));
  rustix_fn!(set_socket_keepalive(set_keepalive(keepalive: bool) -> ()));
  rustix_fn!(socket_keepalive(keepalive() -> bool));
  rustix_fn!(set_socket_reuseaddr(set_reuseaddr(reuseaddr: bool) -> ()));
  rustix_fn!(socket_reuseaddr(reuseaddr() -> bool));

  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  rustix_fn!(set_socket_reuseport(set_reuseport(reuseport: bool) -> ()));
  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  rustix_fn!(socket_reuseport(reuseport() -> bool));

  #[cfg(not(any(
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "haiku"
  )))]
  rustix_fn!(set_ip_tos(set_tos(tos: u8) -> ()));

  #[cfg(not(any(
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "haiku"
  )))]
  rustix_fn!(ip_tos(tos() -> u8));


  use rustix::net::{socket, bind as bindix, connect as connectix, listen as listenix, getsockname, AddressFamily, SocketType};

  pub(crate) fn socket_v4() -> std::io::Result<rustix::fd::OwnedFd> {
    socket(AddressFamily::INET, SocketType::STREAM, None).map_err(Into::into)
  }

  pub(crate) fn socket_v6() -> std::io::Result<rustix::fd::OwnedFd> {
    socket(AddressFamily::INET6, SocketType::STREAM, None).map_err(Into::into)
  }

  pub(crate) fn bind(fd: &impl rustix::fd::AsFd, addr: std::net::SocketAddr) -> std::io::Result<()> {
    bindix(fd, &addr).map_err(Into::into)
  }

  pub(crate) fn local_addr(fd: &impl rustix::fd::AsFd) -> std::io::Result<std::net::SocketAddr> {
    getsockname(fd).and_then(TryInto::try_into).map_err(Into::into)
  }

  pub(crate) fn listen(fd: &impl rustix::fd::AsFd, backlog: u32) -> std::io::Result<()> {
    listenix(fd, backlog as i32).map_err(Into::into)
  }

  pub(crate) fn connect(fd: &impl rustix::fd::AsFd, addr: std::net::SocketAddr) -> std::io::Result<()> {
    connectix(fd, &addr).map_err(Into::into)
  }
}

pub(super) fn shutdown<T>(this: &T, how: std::net::Shutdown) -> std::io::Result<()>
where
  T: AsFd,
{
  rustix::net::shutdown(
    this,
    match how {
      std::net::Shutdown::Read => rustix::net::Shutdown::Read,
      std::net::Shutdown::Write => rustix::net::Shutdown::Write,
      std::net::Shutdown::Both => rustix::net::Shutdown::Both,
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
