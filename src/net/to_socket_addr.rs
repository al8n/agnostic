use std::{
  future::Future,
  io,
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
  pin::Pin,
  task::{Context, Poll},
  vec,
};

use crate::Runtime;

#[doc(hidden)]
pub enum ToSocketAddrsFuture<H> {
  Ready(Option<SocketAddr>),
  Blocking(H),
}

type ReadyFuture<T> = std::future::Ready<io::Result<T>>;

impl<T, R: Runtime> crate::net::ToSocketAddrs<R> for &T
where
  T: crate::net::ToSocketAddrs<R> + ?Sized + Send + Sync,
{
  type Iter = T::Iter;
  type Future = T::Future;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    (**self).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for SocketAddr {
  type Iter = std::option::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, _r: &R) -> Self::Future {
    let iter = Some(*self).into_iter();
    std::future::ready(Ok(iter))
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for SocketAddrV4 {
  type Iter = std::option::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    SocketAddr::V4(*self).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for SocketAddrV6 {
  type Iter = std::option::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    SocketAddr::V6(*self).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for (IpAddr, u16) {
  type Iter = std::option::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    SocketAddr::from(*self).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for (Ipv4Addr, u16) {
  type Iter = std::option::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    let (ip, port) = *self;
    SocketAddrV4::new(ip, port).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for (Ipv6Addr, u16) {
  type Iter = std::option::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    let (ip, port) = *self;
    SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for [SocketAddr] {
  type Iter = std::vec::IntoIter<SocketAddr>;
  type Future = ReadyFuture<Self::Iter>;

  fn to_socket_addrs(&self, _r: &R) -> Self::Future {
    #[inline]
    fn slice_to_vec(addrs: &[SocketAddr]) -> Vec<SocketAddr> {
      addrs.to_vec()
    }

    // This uses a helper method because clippy doesn't like the `to_vec()`
    // call here (it will allocate, whereas `self.iter().copied()` would
    // not), but it's actually necessary in order to ensure that the
    // returned iterator is valid for the `'static` lifetime, which the
    // borrowed `slice::Iter` iterator would not be.
    //
    // Note that we can't actually add an `allow` attribute for
    // `clippy::unnecessary_to_owned` here, as Tokio's CI runs clippy lints
    // on Rust 1.52 to avoid breaking LTS releases of Tokio. Users of newer
    // Rust versions who see this lint should just ignore it.
    let iter = slice_to_vec(self).into_iter();
    std::future::ready(Ok(iter))
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for (String, u16)
where
  ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>:
    Future<Output = io::Result<sealed::OneOrMore>> + Send,
{
  type Iter = sealed::OneOrMore;
  type Future = ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    (self.0.as_str(), self.1).to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for String
where
  ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>:
    Future<Output = io::Result<sealed::OneOrMore>> + Send,
{
  type Iter = <str as crate::net::ToSocketAddrs<R>>::Iter;
  type Future = <str as crate::net::ToSocketAddrs<R>>::Future;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    self[..].to_socket_addrs(r)
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for str
where
  ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>:
    Future<Output = io::Result<sealed::OneOrMore>> + Send,
{
  type Iter = sealed::OneOrMore;

  type Future = ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    // First check if the input parses as a socket address
    let res: Result<SocketAddr, _> = self.parse();

    if let Ok(addr) = res {
      return ToSocketAddrsFuture::Ready(Some(addr));
    }

    // Run DNS lookup on the blocking pool
    let s = self.to_owned();

    ToSocketAddrsFuture::Blocking(r.spawn_blocking(move || {
      std::net::ToSocketAddrs::to_socket_addrs(&s).map(sealed::OneOrMore::More)
    }))
  }
}

impl<R: Runtime> crate::net::ToSocketAddrs<R> for (&str, u16)
where
  ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>:
    Future<Output = io::Result<sealed::OneOrMore>> + Send,
{
  type Iter = sealed::OneOrMore;
  type Future = ToSocketAddrsFuture<R::JoinHandle<io::Result<sealed::OneOrMore>>>;

  fn to_socket_addrs(&self, r: &R) -> Self::Future {
    let (host, port) = *self;

    // try to parse the host as a regular IP address first
    if let Ok(addr) = host.parse::<Ipv4Addr>() {
      let addr = SocketAddrV4::new(addr, port);
      let addr = SocketAddr::V4(addr);

      return ToSocketAddrsFuture::Ready(Some(addr));
    }

    if let Ok(addr) = host.parse::<Ipv6Addr>() {
      let addr = SocketAddrV6::new(addr, port, 0, 0);
      let addr = SocketAddr::V6(addr);

      return ToSocketAddrsFuture::Ready(Some(addr));
    }

    let host = host.to_owned();
    ToSocketAddrsFuture::Blocking(r.spawn_blocking(move || {
      std::net::ToSocketAddrs::to_socket_addrs(&(&host[..], port)).map(sealed::OneOrMore::More)
    }))
  }
}

cfg_tokio!(
  impl Future for ToSocketAddrsFuture<tokio::task::JoinHandle<io::Result<sealed::OneOrMore>>> {
    type Output = io::Result<sealed::OneOrMore>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      match self.get_mut() {
        Self::Ready(ref mut i) => {
          Poll::Ready(Ok(sealed::OneOrMore::One(i.take().into_iter())))
        }
        Self::Blocking(ref mut i) => {
          let res = Pin::new(i).poll(cx)?;
          match res {
            Poll::Ready(res) => match res {
              Ok(res) => Poll::Ready(Ok(res)),
              Err(e) => Poll::Ready(Err(e)),
            },
            Poll::Pending => Poll::Pending,
          }
        }
      }
    }
  }
);

cfg_async_std!(
  impl Future for ToSocketAddrsFuture<async_std::task::JoinHandle<io::Result<sealed::OneOrMore>>> {
    type Output = io::Result<sealed::OneOrMore>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      match self.get_mut() {
        Self::Ready(ref mut i) => {
          Poll::Ready(Ok(sealed::OneOrMore::One(i.take().into_iter())))
        }
        Self::Blocking(ref mut i) => {
          let res = Pin::new(i).poll(cx)?;
          match res {
            Poll::Ready(res) => Poll::Ready(Ok(res)),
            Poll::Pending => Poll::Pending,
          }
        }
      }
    }
  }
);

cfg_smol!(
  impl Future for ToSocketAddrsFuture<smol::Task<io::Result<sealed::OneOrMore>>> {
    type Output = io::Result<sealed::OneOrMore>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      match self.get_mut() {
        Self::Ready(ref mut i) => {
          Poll::Ready(Ok(sealed::OneOrMore::One(i.take().into_iter())))
        }
        Self::Blocking(ref mut i) => {
          let res = Pin::new(i).poll(cx)?;
          match res {
            Poll::Ready(res) => Poll::Ready(Ok(res)),
            Poll::Pending => Poll::Pending,
          }
        }
      }
    }
  }
);

cfg_monoio!(
  impl Future for ToSocketAddrsFuture<monoio::task::JoinHandle<io::Result<sealed::OneOrMore>>> {
    type Output = io::Result<sealed::OneOrMore>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      match self.get_mut() {
        Self::Ready(ref mut i) => {
          Poll::Ready(Ok(sealed::OneOrMore::One(i.take().into_iter())))
        }
        Self::Blocking(ref mut i) => {
          let res = Pin::new(i).poll(cx)?;
          match res {
            Poll::Ready(res) => Poll::Ready(Ok(res)),
            Poll::Pending => Poll::Pending,
          }
        }
      }
    }
  }
);

mod sealed {
  use super::*;
  use std::option;

  #[derive(Debug)]
  #[doc(hidden)]
  pub enum OneOrMore {
    One(option::IntoIter<SocketAddr>),
    More(vec::IntoIter<SocketAddr>),
  }

  impl Iterator for OneOrMore {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
      match self {
        OneOrMore::One(i) => i.next(),
        OneOrMore::More(i) => i.next(),
      }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
      match self {
        OneOrMore::One(i) => i.size_hint(),
        OneOrMore::More(i) => i.size_hint(),
      }
    }
  }
}
