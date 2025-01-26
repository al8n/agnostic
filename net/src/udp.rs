use std::{
  future::Future,
  io,
  net::{Ipv4Addr, Ipv6Addr, SocketAddr},
  task::{Context, Poll},
};

use agnostic_lite::RuntimeLite;

use super::{Fd, ToSocketAddrs};

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! udp_common_methods_impl {
  ($ty:ident.$field:ident) => {
    async fn bind<A: $crate::ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
    where
      Self: Sized,
    {
      let addrs = addr.to_socket_addrs().await?;

      let mut last_err = None;
      for addr in addrs {
        match $ty::bind(addr).await {
          Ok(socket) => return Ok(Self::from(socket)),
          Err(e) => {
            last_err = Some(e);
          }
        }
      }

      Err(last_err.unwrap_or_else(|| {
        io::Error::new(
          io::ErrorKind::InvalidInput,
          "could not resolve to any address",
        )
      }))
    }

    async fn connect<A: $crate::ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> io::Result<()> {
      let addrs = addr.to_socket_addrs().await?;

      let mut last_err = None;

      for addr in addrs {
        match self.$field.connect(addr).await {
          Ok(()) => return Ok(()),
          Err(e) => last_err = Some(e),
        }
      }

      Err(last_err.unwrap_or_else(|| {
        io::Error::new(
          io::ErrorKind::InvalidInput,
          "could not resolve to any address",
        )
      }))
    }

    fn local_addr(&self) -> io::Result<SocketAddr> {
      self.$field.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
      self.$field.peer_addr()
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
      call!(self.$field.recv(buf))
    }

    async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
      call!(self.$field.recv_from(buf))
    }

    async fn send(&self, buf: &[u8]) -> io::Result<usize> {
      call!(self.$field.send(buf))
    }

    async fn send_to<A: $crate::ToSocketAddrs<Self::Runtime>>(
      &self,
      buf: &[u8],
      target: A,
    ) -> io::Result<usize> {
      call!(@send_to self.$field(buf, target))
    }

    async fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
      call!(self.$field.peek(buf))
    }

    async fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
      call!(self.$field.peek_from(buf))
    }

    fn join_multicast_v4(&self, multiaddr: ::std::net::Ipv4Addr, interface: ::std::net::Ipv4Addr) -> io::Result<()> {
      self.$field.join_multicast_v4(multiaddr, interface)
    }

    fn join_multicast_v6(&self, multiaddr: &::std::net::Ipv6Addr, interface: u32) -> io::Result<()> {
      self.$field.join_multicast_v6(multiaddr, interface)
    }

    fn leave_multicast_v4(&self, multiaddr: ::std::net::Ipv4Addr, interface: ::std::net::Ipv4Addr) -> io::Result<()> {
      self.$field.leave_multicast_v4(multiaddr, interface)
    }

    fn leave_multicast_v6(&self, multiaddr: &::std::net::Ipv6Addr, interface: u32) -> io::Result<()> {
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

/// The abstraction of a UDP socket.
pub trait UdpSocket:
  TryFrom<std::net::UdpSocket, Error = io::Error> + Fd + Unpin + Send + Sync + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;

  /// Binds this socket to the specified address.
  fn bind<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  /// Connects this socket to the specified address.
  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
  ) -> impl Future<Output = io::Result<()>> + Send;

  /// Returns the local address that this listener is bound to.
  ///
  /// This can be useful, for example, when binding to port `0` to figure out which port was actually bound.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Returns the peer address that this listener is connected to.
  ///
  /// This can be useful, for example, when connect to port `0` to figure out which port was actually connected.
  fn peer_addr(&self) -> io::Result<SocketAddr>;

  /// Receives data from the socket. Returns the number of bytes read and the source address.
  fn recv(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send;

  /// Receives data from the socket, returning the number of bytes read and the source address.
  fn recv_from(
    &self,
    buf: &mut [u8],
  ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send;

  /// Sends data by the socket.
  fn send(&self, buf: &[u8]) -> impl Future<Output = io::Result<usize>> + Send;

  /// Sends data by the socket to the given address.
  fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> impl Future<Output = io::Result<usize>> + Send;

  /// Receives data from the socket without removing it from the queue.
  ///
  /// On success, returns the number of bytes peeked.
  fn peek(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send;

  /// Receives data from socket without removing it from the queue.
  ///
  /// On success, returns the number of bytes peeked and the origin.
  fn peek_from(
    &self,
    buf: &mut [u8],
  ) -> impl Future<Output = io::Result<(usize, SocketAddr)>> + Send;

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
