use std::{
  future::Future,
  io,
  net::{Ipv4Addr, Ipv6Addr, SocketAddr},
  pin::Pin,
  task::{Context, Poll},
};

use agnostic_lite::RuntimeLite;

use super::{Fd, ToSocketAddrs};

#[cfg(any(feature = "smol", feature = "tokio"))]
macro_rules! udp_common_methods_impl {
  ($ty:ident.$field:ident) => {
    type Bind<A>
      = $crate::udp::Bind<Self, A>
    where
      A: $crate::ToSocketAddrs<Self::Runtime>,
      Self: Sized;

    type Connect<'a, A>
      = $crate::udp::Connect<'a, Self, A>
    where
      Self: 'a,
      A: $crate::ToSocketAddrs<Self::Runtime>;

    type Recv<'a>
      = $crate::udp::Recv<'a, Self>
    where
      Self: 'a;

    type RecvFrom<'a>
      = $crate::udp::RecvFrom<'a, Self>
    where
      Self: 'a;

    type Send<'a>
      = $crate::udp::Send_<'a, Self>
    where
      Self: 'a;

    type SendTo<'a, A>
      = $crate::udp::SendTo<'a, Self, A>
    where
      Self: 'a,
      A: $crate::ToSocketAddrs<Self::Runtime>;

    type Peek<'a>
      = $crate::udp::Peek<'a, Self>
    where
      Self: 'a;

    type PeekFrom<'a>
      = $crate::udp::PeekFrom<'a, Self>
    where
      Self: 'a;

    fn bind<A: $crate::ToSocketAddrs<Self::Runtime>>(addr: A) -> Self::Bind<A>
    where
      Self: Sized,
    {
      $crate::udp::Bind::new(addr)
    }

    fn connect<A: $crate::ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> Self::Connect<'_, A> {
      $crate::udp::Connect::new(self, addr)
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
      self.$field.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
      self.$field.peer_addr()
    }

    fn recv<'a>(&'a self, buf: &'a mut [u8]) -> Self::Recv<'a> {
      $crate::udp::Recv::new(self, buf)
    }

    fn recv_from<'a>(&'a self, buf: &'a mut [u8]) -> Self::RecvFrom<'a> {
      $crate::udp::RecvFrom::new(self, buf)
    }

    fn send<'a>(&'a self, buf: &'a [u8]) -> Self::Send<'a> {
      $crate::udp::Send_::new(self, buf)
    }

    fn send_to<'a, A: $crate::ToSocketAddrs<Self::Runtime>>(
      &'a self,
      buf: &'a [u8],
      target: A,
    ) -> Self::SendTo<'a, A> {
      $crate::udp::SendTo::new(self, buf, target)
    }

    fn peek<'a>(&'a self, buf: &'a mut [u8]) -> Self::Peek<'a> {
      $crate::udp::Peek::new(self, buf)
    }

    fn peek_from<'a>(&'a self, buf: &'a mut [u8]) -> Self::PeekFrom<'a> {
      $crate::udp::PeekFrom::new(self, buf)
    }

    fn join_multicast_v4(
      &self,
      multiaddr: ::std::net::Ipv4Addr,
      interface: ::std::net::Ipv4Addr,
    ) -> io::Result<()> {
      self.$field.join_multicast_v4(multiaddr, interface)
    }

    fn join_multicast_v6(
      &self,
      multiaddr: &::std::net::Ipv6Addr,
      interface: u32,
    ) -> io::Result<()> {
      self.$field.join_multicast_v6(multiaddr, interface)
    }

    fn leave_multicast_v4(
      &self,
      multiaddr: ::std::net::Ipv4Addr,
      interface: ::std::net::Ipv4Addr,
    ) -> io::Result<()> {
      self.$field.leave_multicast_v4(multiaddr, interface)
    }

    fn leave_multicast_v6(
      &self,
      multiaddr: &::std::net::Ipv6Addr,
      interface: u32,
    ) -> io::Result<()> {
      self.$field.leave_multicast_v6(multiaddr, interface)
    }

    fn multicast_loop_v4(&self) -> io::Result<bool> {
      self.$field.multicast_loop_v4()
    }

    fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()> {
      self.$field.set_multicast_loop_v4(on)
    }

    fn multicast_ttl_v4(&self) -> io::Result<u32> {
      self.$field.multicast_ttl_v4()
    }

    fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()> {
      self.$field.set_multicast_ttl_v4(ttl)
    }

    fn multicast_loop_v6(&self) -> io::Result<bool> {
      self.$field.multicast_loop_v6()
    }

    fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()> {
      self.$field.set_multicast_loop_v6(on)
    }

    fn set_ttl(&self, ttl: u32) -> io::Result<()> {
      self.$field.set_ttl(ttl)
    }

    fn ttl(&self) -> io::Result<u32> {
      self.$field.ttl()
    }

    fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
      self.$field.set_broadcast(broadcast)
    }

    fn broadcast(&self) -> io::Result<bool> {
      self.$field.broadcast()
    }
  };
}

#[inline]
fn no_addresses_error() -> io::Error {
  io::Error::new(
    io::ErrorKind::InvalidInput,
    "could not resolve to any address",
  )
}

pin_project_lite::pin_project! {
  #[project = BindStateProj]
  enum BindState<S, A>
  where
    S: UdpSocket,
    A: ToSocketAddrs<S::Runtime>,
  {
    Resolving {
      #[pin]
      fut: A::Future,
    },
    // Resolution finished; the binding is performed synchronously on the next poll.
    Resolved {
      addrs: Option<A::Iter>,
    },
  }
}

pin_project_lite::pin_project! {
  /// The future returned by [`UdpSocket::bind`].
  pub struct Bind<S, A>
  where
    S: UdpSocket,
    A: ToSocketAddrs<S::Runtime>,
  {
    #[pin]
    state: BindState<S, A>,
  }
}

impl<S, A> Bind<S, A>
where
  S: UdpSocket,
  A: ToSocketAddrs<S::Runtime>,
{
  #[inline]
  pub(crate) fn new(addr: A) -> Self {
    Self {
      state: BindState::Resolving {
        fut: addr.to_socket_addrs(),
      },
    }
  }
}

impl<S, A> Future for Bind<S, A>
where
  S: UdpSocket,
  A: ToSocketAddrs<S::Runtime>,
{
  type Output = io::Result<S>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.project().state;
    loop {
      match state.as_mut().project() {
        BindStateProj::Resolving { fut } => {
          let addrs = futures_util::ready!(fut.poll(cx))?;
          state.set(BindState::Resolved { addrs: Some(addrs) });
        }
        BindStateProj::Resolved { addrs } => {
          let addrs = addrs.take().expect("polled `Bind` after completion");
          let mut last_err = None;
          for addr in addrs {
            // The socket must be non-blocking before handing it to a runtime: `tokio`'s
            // `from_std` rejects a blocking fd, and `smol`/`async-net` expects it too.
            let res = std::net::UdpSocket::bind(addr).and_then(|socket| {
              socket.set_nonblocking(true)?;
              S::try_from(socket)
            });
            match res {
              Ok(socket) => return Poll::Ready(Ok(socket)),
              Err(e) => last_err = Some(e),
            }
          }
          return Poll::Ready(Err(last_err.unwrap_or_else(no_addresses_error)));
        }
      }
    }
  }
}

pin_project_lite::pin_project! {
  #[project = ConnectStateProj]
  enum ConnectState<'a, S, A>
  where
    S: UdpSocket,
    A: ToSocketAddrs<S::Runtime>,
  {
    Resolving {
      socket: &'a S,
      #[pin]
      fut: A::Future,
    },
    // Resolution finished; the connection is performed synchronously on the next poll.
    Resolved {
      socket: &'a S,
      addrs: Option<A::Iter>,
    },
  }
}

pin_project_lite::pin_project! {
  /// The future returned by [`UdpSocket::connect`].
  pub struct Connect<'a, S, A>
  where
    S: UdpSocket,
    A: ToSocketAddrs<S::Runtime>,
  {
    #[pin]
    state: ConnectState<'a, S, A>,
  }
}

impl<'a, S, A> Connect<'a, S, A>
where
  S: UdpSocket,
  A: ToSocketAddrs<S::Runtime>,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, addr: A) -> Self {
    Self {
      state: ConnectState::Resolving {
        socket,
        fut: addr.to_socket_addrs(),
      },
    }
  }
}

impl<S, A> Future for Connect<'_, S, A>
where
  S: UdpSocket,
  A: ToSocketAddrs<S::Runtime>,
{
  type Output = io::Result<()>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.project().state;
    loop {
      match state.as_mut().project() {
        ConnectStateProj::Resolving { socket, fut } => {
          let socket: &S = socket;
          let addrs = futures_util::ready!(fut.poll(cx))?;
          state.set(ConnectState::Resolved {
            socket,
            addrs: Some(addrs),
          });
        }
        ConnectStateProj::Resolved { socket, addrs } => {
          let addrs = addrs.take().expect("polled `Connect` after completion");
          let mut last_err = None;
          for addr in addrs {
            // UDP `connect` only sets the default peer, so it is a synchronous syscall.
            match socket2::SockRef::from(*socket).connect(&addr.into()) {
              Ok(()) => return Poll::Ready(Ok(())),
              Err(e) => last_err = Some(e),
            }
          }
          return Poll::Ready(Err(last_err.unwrap_or_else(no_addresses_error)));
        }
      }
    }
  }
}

/// The future returned by [`UdpSocket::recv_from`].
pub struct RecvFrom<'a, S: ?Sized> {
  socket: &'a S,
  buf: &'a mut [u8],
}

impl<'a, S> RecvFrom<'a, S>
where
  S: UdpSocket,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, buf: &'a mut [u8]) -> Self {
    Self { socket, buf }
  }
}

impl<S> Future for RecvFrom<'_, S>
where
  S: UdpSocket,
{
  type Output = io::Result<(usize, SocketAddr)>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    this.socket.poll_recv_from(cx, this.buf)
  }
}

/// The future returned by [`UdpSocket::recv`].
pub struct Recv<'a, S: ?Sized> {
  inner: RecvFrom<'a, S>,
}

impl<'a, S> Recv<'a, S>
where
  S: UdpSocket,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, buf: &'a mut [u8]) -> Self {
    Self {
      inner: RecvFrom::new(socket, buf),
    }
  }
}

impl<S> Future for Recv<'_, S>
where
  S: UdpSocket,
{
  type Output = io::Result<usize>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    Pin::new(&mut this.inner).poll(cx).map_ok(|(len, _)| len)
  }
}

/// The future returned by [`UdpSocket::peek_from`].
pub struct PeekFrom<'a, S: ?Sized> {
  socket: &'a S,
  buf: &'a mut [u8],
}

impl<'a, S> PeekFrom<'a, S>
where
  S: UdpSocket,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, buf: &'a mut [u8]) -> Self {
    Self { socket, buf }
  }
}

impl<S> Future for PeekFrom<'_, S>
where
  S: UdpSocket,
{
  type Output = io::Result<(usize, SocketAddr)>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    this.socket.poll_peek_from(cx, this.buf)
  }
}

/// The future returned by [`UdpSocket::peek`].
pub struct Peek<'a, S: ?Sized> {
  inner: PeekFrom<'a, S>,
}

impl<'a, S> Peek<'a, S>
where
  S: UdpSocket,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, buf: &'a mut [u8]) -> Self {
    Self {
      inner: PeekFrom::new(socket, buf),
    }
  }
}

impl<S> Future for Peek<'_, S>
where
  S: UdpSocket,
{
  type Output = io::Result<usize>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    Pin::new(&mut this.inner).poll(cx).map_ok(|(len, _)| len)
  }
}

/// The future returned by [`UdpSocket::send`].
pub struct Send_<'a, S: ?Sized> {
  socket: &'a S,
  buf: &'a [u8],
  // The peer is resolved once so the resolution error (a disconnected socket) surfaces before any
  // send is attempted; `None` once that error has been taken.
  peer: Option<io::Result<SocketAddr>>,
}

impl<'a, S> Send_<'a, S>
where
  S: UdpSocket,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, buf: &'a [u8]) -> Self {
    Self {
      socket,
      buf,
      peer: Some(socket.peer_addr()),
    }
  }
}

impl<S> Future for Send_<'_, S>
where
  S: UdpSocket,
{
  type Output = io::Result<usize>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    match &this.peer {
      Some(Ok(_)) => this.socket.poll_send(cx, this.buf),
      _ => Poll::Ready(Err(match this.peer.take() {
        Some(Err(e)) => e,
        _ => no_addresses_error(),
      })),
    }
  }
}

pin_project_lite::pin_project! {
  #[project = SendToStateProj]
  enum SendToState<'a, S, A>
  where
    S: UdpSocket,
    A: ToSocketAddrs<S::Runtime>,
  {
    Resolving {
      socket: &'a S,
      buf: &'a [u8],
      #[pin]
      fut: A::Future,
    },
    // Driving `poll_send_to` over the resolved address(es).
    Sending {
      socket: &'a S,
      buf: &'a [u8],
      addrs: A::Iter,
      current: Option<SocketAddr>,
      last_err: Option<io::Error>,
    },
  }
}

pin_project_lite::pin_project! {
  /// The future returned by [`UdpSocket::send_to`].
  pub struct SendTo<'a, S, A>
  where
    S: UdpSocket,
    A: ToSocketAddrs<S::Runtime>,
  {
    #[pin]
    state: SendToState<'a, S, A>,
  }
}

impl<'a, S, A> SendTo<'a, S, A>
where
  S: UdpSocket,
  A: ToSocketAddrs<S::Runtime>,
{
  #[inline]
  pub(crate) fn new(socket: &'a S, buf: &'a [u8], target: A) -> Self {
    Self {
      state: SendToState::Resolving {
        socket,
        buf,
        fut: target.to_socket_addrs(),
      },
    }
  }
}

impl<S, A> Future for SendTo<'_, S, A>
where
  S: UdpSocket,
  A: ToSocketAddrs<S::Runtime>,
{
  type Output = io::Result<usize>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.project().state;
    loop {
      match state.as_mut().project() {
        SendToStateProj::Resolving { socket, buf, fut } => {
          let socket: &S = socket;
          let buf: &[u8] = buf;
          let addrs = futures_util::ready!(fut.poll(cx))?;
          state.set(SendToState::Sending {
            socket,
            buf,
            addrs,
            current: None,
            last_err: None,
          });
        }
        SendToStateProj::Sending {
          socket,
          buf,
          addrs,
          current,
          last_err,
        } => {
          if current.is_none() {
            match addrs.next() {
              Some(addr) => *current = Some(addr),
              None => {
                return Poll::Ready(Err(last_err.take().unwrap_or_else(no_addresses_error)));
              }
            }
          }

          let addr = current.expect("`current` was just set");
          match socket.poll_send_to(cx, buf, addr) {
            Poll::Ready(Ok(n)) => return Poll::Ready(Ok(n)),
            Poll::Ready(Err(e)) => {
              *last_err = Some(e);
              *current = None;
            }
            Poll::Pending => return Poll::Pending,
          }
        }
      }
    }
  }
}

/// The abstraction of a UDP socket.
pub trait UdpSocket:
  TryFrom<std::net::UdpSocket, Error = io::Error> + Fd + Unpin + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;

  /// The future returned by [`bind`](UdpSocket::bind).
  type Bind<A>: Future<Output = io::Result<Self>>
  where
    A: ToSocketAddrs<Self::Runtime>,
    Self: Sized;

  /// The future returned by [`connect`](UdpSocket::connect).
  type Connect<'a, A>: Future<Output = io::Result<()>>
  where
    Self: 'a,
    A: ToSocketAddrs<Self::Runtime>;

  /// The future returned by [`recv`](UdpSocket::recv).
  type Recv<'a>: Future<Output = io::Result<usize>>
  where
    Self: 'a;

  /// The future returned by [`recv_from`](UdpSocket::recv_from).
  type RecvFrom<'a>: Future<Output = io::Result<(usize, SocketAddr)>>
  where
    Self: 'a;

  /// The future returned by [`send`](UdpSocket::send).
  type Send<'a>: Future<Output = io::Result<usize>>
  where
    Self: 'a;

  /// The future returned by [`send_to`](UdpSocket::send_to).
  type SendTo<'a, A>: Future<Output = io::Result<usize>>
  where
    Self: 'a,
    A: ToSocketAddrs<Self::Runtime>;

  /// The future returned by [`peek`](UdpSocket::peek).
  type Peek<'a>: Future<Output = io::Result<usize>>
  where
    Self: 'a;

  /// The future returned by [`peek_from`](UdpSocket::peek_from).
  type PeekFrom<'a>: Future<Output = io::Result<(usize, SocketAddr)>>
  where
    Self: 'a;

  /// Binds this socket to the specified address.
  fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> Self::Bind<A>
  where
    Self: Sized;

  /// Connects this socket to the specified address.
  fn connect<A: ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> Self::Connect<'_, A>;

  /// Returns the local address that this listener is bound to.
  ///
  /// This can be useful, for example, when binding to port `0` to figure out which port was actually bound.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Returns the peer address that this listener is connected to.
  ///
  /// This can be useful, for example, when connect to port `0` to figure out which port was actually connected.
  fn peer_addr(&self) -> io::Result<SocketAddr>;

  /// Receives data from the socket. Returns the number of bytes read and the source address.
  fn recv<'a>(&'a self, buf: &'a mut [u8]) -> Self::Recv<'a>;

  /// Receives data from the socket, returning the number of bytes read and the source address.
  fn recv_from<'a>(&'a self, buf: &'a mut [u8]) -> Self::RecvFrom<'a>;

  /// Sends data by the socket.
  fn send<'a>(&'a self, buf: &'a [u8]) -> Self::Send<'a>;

  /// Sends data by the socket to the given address.
  fn send_to<'a, A: ToSocketAddrs<Self::Runtime>>(
    &'a self,
    buf: &'a [u8],
    target: A,
  ) -> Self::SendTo<'a, A>;

  /// Receives data from the socket without removing it from the queue.
  ///
  /// On success, returns the number of bytes peeked.
  fn peek<'a>(&'a self, buf: &'a mut [u8]) -> Self::Peek<'a>;

  /// Receives data from socket without removing it from the queue.
  ///
  /// On success, returns the number of bytes peeked and the origin.
  fn peek_from<'a>(&'a self, buf: &'a mut [u8]) -> Self::PeekFrom<'a>;

  /// Executes an operation of the `IP_ADD_MEMBERSHIP` type.
  ///
  /// This function specifies a new multicast group for this socket to join.
  /// The address must be a valid multicast address, and `interface` is the
  /// address of the local interface with which the system should join the
  /// multicast group. If it's equal to `INADDR_ANY` then an appropriate
  /// interface is chosen by the system.
  fn join_multicast_v4(&self, multiaddr: Ipv4Addr, interface: Ipv4Addr) -> io::Result<()>;

  /// Executes an operation of the `IPV6_ADD_MEMBERSHIP` type.
  ///
  /// This function specifies a new multicast group for this socket to join.
  /// The address must be a valid multicast address, and `interface` is the
  /// index of the interface to join/leave (or 0 to indicate any interface).
  fn join_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()>;

  /// Executes an operation of the `IP_DROP_MEMBERSHIP` type.
  ///
  /// For more information about this option, see [`join_multicast_v4`].
  ///
  /// [`join_multicast_v4`]: method@UdpSocket::join_multicast_v4
  fn leave_multicast_v4(&self, multiaddr: Ipv4Addr, interface: Ipv4Addr) -> io::Result<()>;

  /// Executes an operation of the `IPV6_DROP_MEMBERSHIP` type.
  ///
  /// For more information about this option, see [`join_multicast_v6`].
  ///
  /// [`join_multicast_v6`]: method@UdpSocket::join_multicast_v6
  fn leave_multicast_v6(&self, multiaddr: &Ipv6Addr, interface: u32) -> io::Result<()>;

  /// Gets the value of the `IP_MULTICAST_LOOP` option for this socket.
  ///
  /// For more information about this option, see [`set_multicast_loop_v4`].
  ///
  /// [`set_multicast_loop_v4`]: #method.set_multicast_loop_v4
  fn multicast_loop_v4(&self) -> io::Result<bool>;

  /// Sets the value of the `IP_MULTICAST_LOOP` option for this socket.
  ///
  /// If enabled, multicast packets will be looped back to the local socket.
  ///
  /// # Note
  ///
  /// This may not have any affect on IPv6 sockets.
  fn set_multicast_loop_v4(&self, on: bool) -> io::Result<()>;

  /// Gets the value of the `IP_MULTICAST_TTL` option for this socket.
  ///
  /// For more information about this option, see [`set_multicast_ttl_v4`].
  ///
  /// [`set_multicast_ttl_v4`]: #method.set_multicast_ttl_v4
  fn multicast_ttl_v4(&self) -> io::Result<u32>;

  /// Sets the value of the `IP_MULTICAST_TTL` option for this socket.
  ///
  /// Indicates the time-to-live value of outgoing multicast packets for this socket. The default
  /// value is 1 which means that multicast packets don't leave the local network unless
  /// explicitly requested.
  ///
  /// # Note
  ///
  /// This may not have any affect on IPv6 sockets.
  fn set_multicast_ttl_v4(&self, ttl: u32) -> io::Result<()>;

  /// Gets the value of the `IPV6_MULTICAST_LOOP` option for this socket.
  ///
  /// For more information about this option, see [`set_multicast_loop_v6`].
  ///
  /// [`set_multicast_loop_v6`]: #method.set_multicast_loop_v6
  fn multicast_loop_v6(&self) -> io::Result<bool>;

  /// Sets the value of the `IPV6_MULTICAST_LOOP` option for this socket.
  ///
  /// Controls whether this socket sees the multicast packets it sends itself.
  ///
  /// # Note
  ///
  /// This may not have any affect on IPv4 sockets.
  fn set_multicast_loop_v6(&self, on: bool) -> io::Result<()>;

  /// Sets the ttl of this UDP socket.
  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  /// Gets the ttl of this UDP socket.
  fn ttl(&self) -> io::Result<u32>;

  /// Sets the broadcast flag for this UDP socket.
  fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;

  /// Gets the broadcast flag of this UDP socket.
  fn broadcast(&self) -> io::Result<bool>;

  /// Creates a new independently owned handle to the underlying socket.
  ///
  /// The returned `UdpSocket` is a reference to the same socket that this
  /// object references. Both handles will read and write the same port, and
  /// options set on one socket will be propagated to the other.
  fn try_clone(&self) -> ::std::io::Result<Self> {
    super::os::duplicate::<_, std::net::UdpSocket>(self).and_then(Self::try_from)
  }

  /// Get the value of the `IPV6_V6ONLY` option for this socket.
  fn only_v6(&self) -> io::Result<bool> {
    super::os::only_v6(self)
  }

  /// Set value for the `SO_RCVBUF` option on this socket.
  ///
  /// Changes the size of the operating system’s receive buffer associated with the socket.
  fn set_recv_buffer_size(&self, size: usize) -> io::Result<()> {
    super::os::set_recv_buffer_size(self, size)
  }

  /// Get value for the `SO_RCVBUF` option on this socket.
  ///
  /// For more information about this option, see [`set_recv_buffer_size`](UdpSocket::set_recv_buffer_size).
  fn recv_buffer_size(&self) -> io::Result<usize> {
    super::os::recv_buffer_size(self)
  }

  /// Set value for the `SO_SNDBUF` option on this socket.
  ///
  /// Changes the size of the operating system’s send buffer associated with the socket.
  fn set_send_buffer_size(&self, size: usize) -> io::Result<()> {
    super::os::set_send_buffer_size(self, size)
  }

  /// Get the value of the `SO_SNDBUF` option on this socket.
  ///
  /// For more information about this option, see [`set_send_buffer_size`](UdpSocket::set_send_buffer_size).
  fn send_buffer_size(&self) -> io::Result<usize> {
    super::os::send_buffer_size(self)
  }

  /// Attempts to receive a single datagram on the socket.
  ///
  /// Note that on multiple calls to a `poll_*` method in the recv direction, only the
  /// `Waker` from the `Context` passed to the most recent call will be scheduled to
  /// receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not ready to read
  /// * `Poll::Ready(Ok(addr))` reads data from `addr` into `ReadBuf` if the socket is ready
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  ///
  /// # Notes
  /// Note that the socket address **cannot** be implicitly trusted, because it is relatively
  /// trivial to send a UDP datagram with a spoofed origin in a [packet injection attack].
  /// Because UDP is stateless and does not validate the origin of a packet,
  /// the attacker does not need to be able to intercept traffic in order to interfere.
  /// It is important to be aware of this when designing your application-level protocol.
  ///
  /// [packet injection attack]: https://en.wikipedia.org/wiki/Packet_injection
  fn poll_recv_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>>;

  /// Attempts to receive a single datagram on the socket, without removing it from the queue.
  ///
  /// Note that on multiple calls to a `poll_*` method in the recv direction, only the
  /// `Waker` from the `Context` passed to the most recent call will be scheduled to
  /// receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not ready to read
  /// * `Poll::Ready(Ok((n, addr)))` reads `n` bytes of data peeked from `addr` if the socket is ready
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_peek_from(
    &self,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<(usize, SocketAddr)>>;

  /// Attempts to send data on the connected socket.
  ///
  /// This uses the connected peer set by [`connect`](UdpSocket::connect) rather than an explicit
  /// address, so it issues a plain `send` syscall. On BSD-derived platforms a `sendto` with an
  /// address would fail with `EISCONN` on a connected datagram socket, which is why a connected
  /// send needs its own primitive instead of reusing [`poll_send_to`](UdpSocket::poll_send_to).
  ///
  /// Note that on multiple calls to a `poll_*` method in the send direction, only the
  /// `Waker` from the `Context` passed to the most recent call will be scheduled to
  /// receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not ready to write
  /// * `Poll::Ready(Ok(n))` `n` is the number of bytes sent.
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_send(&self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>>;

  /// Attempts to send data on the socket to a given address.
  ///
  /// Note that on multiple calls to a `poll_*` method in the send direction, only the
  /// `Waker` from the `Context` passed to the most recent call will be scheduled to
  /// receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not ready to write
  /// * `Poll::Ready(Ok(n))` `n` is the number of bytes sent.
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_send_to(
    &self,
    cx: &mut Context<'_>,
    buf: &[u8],
    target: SocketAddr,
  ) -> Poll<io::Result<usize>>;
}
