use std::{future::Future, io, net::SocketAddr, time::Duration};

use agnostic_lite::RuntimeLite;

use super::{
  io::{AsyncRead, AsyncReadWrite, AsyncWrite},
  Fd, ToSocketAddrs,
};

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! resolve_address_error {
  () => {{
    ::std::io::Error::new(
      ::std::io::ErrorKind::InvalidInput,
      "could not resolve to any address",
    )
  }};
}

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! tcp_listener_common_methods {
  ($ty:ident.$field:ident) => {
    async fn bind<A: $crate::ToSocketAddrs<Self::Runtime>>(addr: A) -> std::io::Result<Self>
    where
      Self: Sized,
    {
      let addrs = addr.to_socket_addrs().await?;

      let mut last_err = core::option::Option::None;
      for addr in addrs {
        match $ty::bind(addr).await {
          ::core::result::Result::Ok(ln) => return ::core::result::Result::Ok(Self { ln }),
          ::core::result::Result::Err(e) => last_err = core::option::Option::Some(e),
        }
      }

      ::core::result::Result::Err(last_err.unwrap_or_else(|| resolve_address_error!()))
    }

    async fn accept(&self) -> ::std::io::Result<(Self::Stream, ::std::net::SocketAddr)> {
      self
        .$field
        .accept()
        .await
        .map(|(stream, addr)| (Self::Stream::from(stream), addr))
    }

    fn local_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.local_addr()
    }
  };
}

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! tcp_stream_common_methods {
  ($runtime:literal::$field:ident) => {
    async fn connect<A: $crate::ToSocketAddrs<Self::Runtime>>(addr: A) -> ::std::io::Result<Self>
    where
      Self: Sized,
    {
      let addrs = addr.to_socket_addrs().await?;

      let mut last_err = ::core::option::Option::None;

      for addr in addrs {
        paste::paste! {
          match ::[< $runtime:snake >]::net::TcpStream::connect(addr).await {
            ::core::result::Result::Ok(stream) => return ::core::result::Result::Ok(Self::from(stream)),
            ::core::result::Result::Err(e) => last_err = ::core::option::Option::Some(e),
          }
        }
      }

      ::core::result::Result::Err(last_err.unwrap_or_else(|| resolve_address_error!()))
    }

    async fn connect_timeout(
      addr: &::std::net::SocketAddr,
      timeout: ::std::time::Duration,
    ) -> ::std::io::Result<Self>
    where
      Self: Sized
    {
      let res = <Self::Runtime as ::agnostic_lite::RuntimeLite>::timeout(timeout, Self::connect(addr)).await;

      match res {
        ::core::result::Result::Ok(stream) => stream,
        ::core::result::Result::Err(err) => Err(err.into()),
      }
    }

    async fn peek(&self, buf: &mut [u8]) -> ::std::io::Result<usize> {
      self.$field.peek(buf).await
    }

    fn local_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.local_addr()
    }

    fn peer_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.peer_addr()
    }

    fn set_ttl(&self, ttl: u32) -> ::std::io::Result<()> {
      self.$field.set_ttl(ttl)
    }

    fn ttl(&self) -> ::std::io::Result<u32> {
      self.$field.ttl()
    }

    fn set_nodelay(&self, nodelay: bool) -> ::std::io::Result<()> {
      self.$field.set_nodelay(nodelay)
    }

    fn nodelay(&self) -> ::std::io::Result<bool> {
      self.$field.nodelay()
    }
  };
}

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! tcp_stream_owned_read_half_common_methods {
  ($field:ident) => {
    fn local_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.local_addr()
    }

    fn peer_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.peer_addr()
    }

    async fn peek(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
      self.$field.peek(buf).await
    }
  };
}

#[cfg(any(feature = "async-std", feature = "smol", feature = "tokio"))]
macro_rules! tcp_stream_owned_write_half_common_methods {
  ($field:ident) => {
    fn local_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.local_addr()
    }

    fn peer_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.peer_addr()
    }
  };
}

#[cfg(any(feature = "async-std", feature = "smol"))]
macro_rules! tcp_listener_incoming {
  ($ty:ty => $stream:ty) => {
    pin_project_lite::pin_project! {
      /// A stream of incoming TCP connections.
      ///
      /// This stream is infinite, i.e awaiting the next connection will never result in [`None`]. It is
      /// created by the [`TcpListener::incoming()`](crate::TcpListener::incoming) method.
      pub struct Incoming<'a> {
        #[pin]
        inner: $ty,
      }
    }

    impl core::fmt::Debug for Incoming<'_> {
      fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Incoming {{ ... }}")
      }
    }

    impl<'a> From<$ty> for Incoming<'a> {
      fn from(inner: $ty) -> Self {
        Self { inner }
      }
    }

    impl<'a> From<Incoming<'a>> for $ty {
      fn from(incoming: Incoming<'a>) -> Self {
        incoming.inner
      }
    }

    impl<'a> ::futures_util::stream::Stream for Incoming<'a> {
      type Item = ::std::io::Result<$stream>;

      fn poll_next(
        self: ::std::pin::Pin<&mut Self>,
        cx: &mut ::std::task::Context<'_>,
      ) -> ::std::task::Poll<::core::option::Option<Self::Item>> {
        self
          .project()
          .inner
          .poll_next(cx)
          .map(|stream| stream.map(|stream| stream.map(<$stream>::from)))
      }
    }
  };
}

/// The abstraction of a owned read half of a TcpStream.
pub trait OwnedReadHalf: AsyncRead + Unpin + Send + Sync + 'static {
  /// The async runtime.
  type Runtime: RuntimeLite;

  /// Returns the local address that this stream is bound to.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Returns the remote address that this stream is connected to.
  fn peer_addr(&self) -> io::Result<SocketAddr>;

  /// Receives data on the socket from the remote address to which it is connected, without
  /// removing that data from the queue.
  ///
  /// On success, returns the number of bytes peeked.
  ///
  /// Successive calls return the same data. This is accomplished by passing `MSG_PEEK` as a flag
  /// to the underlying `recv` system call.
  fn peek(&mut self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send;
}

/// The abstraction of a owned write half of a TcpStream.
pub trait OwnedWriteHalf: AsyncWrite + Unpin + Send + Sync + 'static {
  /// The async runtime.
  type Runtime: RuntimeLite;

  /// Shuts down the write half and without closing the read half.
  fn forget(self);

  /// Returns the local address that this stream is bound to.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Returns the remote address that this stream is connected to.
  fn peer_addr(&self) -> io::Result<SocketAddr>;
}

/// Error indicating that two halves were not from the same socket, and thus could not be reunited.
pub trait ReuniteError<T>: core::error::Error + Unpin + Send + Sync + 'static
where
  T: TcpStream,
{
  /// Consumes the error and returns the read half and write half of the socket.
  fn into_components(self) -> (T::OwnedReadHalf, T::OwnedWriteHalf);
}

/// The abstraction of a TCP stream.
pub trait TcpStream:
  TryFrom<std::net::TcpStream, Error = io::Error>
  + Fd
  + AsyncReadWrite
  + Unpin
  + Send
  + Sync
  + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;
  /// The owned read half of the stream.
  type OwnedReadHalf: OwnedReadHalf;
  /// The owned write half of the stream.
  type OwnedWriteHalf: OwnedWriteHalf;
  /// Error indicating that two halves were not from the same socket, and thus could not be reunited.
  type ReuniteError: ReuniteError<Self>;

  /// Connects to the specified address.
  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  /// Opens a TCP connection to a remote host with a timeout.
  ///
  /// Unlike `connect`, `connect_timeout` takes a single [`SocketAddr`] since
  /// timeout must be applied to individual addresses.
  ///
  /// It is an error to pass a zero `Duration` to this function.
  ///
  /// Unlike other methods on `TcpStream`, this does not correspond to a
  /// single system call. It instead calls `connect` in nonblocking mode and
  /// then uses an OS-specific mechanism to await the completion of the
  /// connection request.
  fn connect_timeout(
    addr: &SocketAddr,
    timeout: Duration,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  /// Receives data on the socket from the remote address to which it is connected, without
  /// removing that data from the queue.
  ///
  /// On success, returns the number of bytes peeked.
  ///
  /// Successive calls return the same data. This is accomplished by passing `MSG_PEEK` as a flag
  /// to the underlying `recv` system call.
  fn peek(&self, buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + Send;

  /// Returns the local address that this stream is bound to.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Returns the remote address that this stream is connected to.
  fn peer_addr(&self) -> io::Result<SocketAddr>;

  /// Sets the time-to-live value for this socket.  
  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  /// Gets the time-to-live value of this socket.
  fn ttl(&self) -> io::Result<u32>;

  /// Sets the value of the `TCP_NODELAY` option on this socket.
  fn set_nodelay(&self, nodelay: bool) -> io::Result<()>;

  /// Gets the value of the `TCP_NODELAY` option on this socket.
  fn nodelay(&self) -> io::Result<bool>;

  /// Splits the stream to read and write halves.
  fn into_split(self) -> (Self::OwnedReadHalf, Self::OwnedWriteHalf);

  /// Shuts down the read, write, or both halves of this connection.
  fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
    super::os::shutdown(self, how)
  }

  /// Creates a new independently owned handle to the underlying socket.
  ///
  /// The returned `UdpSocket` is a reference to the same socket that this
  /// object references. Both handles will read and write the same port, and
  /// options set on one socket will be propagated to the other.
  fn try_clone(&self) -> io::Result<Self> {
    super::os::duplicate::<_, std::net::TcpStream>(self).and_then(Self::try_from)
  }

  /// Get the value of the `IPV6_V6ONLY` option for this socket.
  fn only_v6(&self) -> io::Result<bool> {
    super::os::only_v6(self)
  }

  /// Gets the value of the `SO_LINGER` option on this socket.
  ///
  /// For more information about this option, see [`TcpStream::set_linger`].
  fn linger(&self) -> io::Result<Option<std::time::Duration>> {
    super::os::linger(self)
  }

  /// Sets the value of the `SO_LINGER` option on this socket.
  ///
  /// This value controls how the socket is closed when data remains to be sent.
  /// If `SO_LINGER` is set, the socket will remain open for the specified duration as the system attempts to send pending data.
  /// Otherwise, the system may close the socket immediately, or wait for a default timeout.
  fn set_linger(&self, duration: Option<std::time::Duration>) -> io::Result<()> {
    super::os::set_linger(self, duration)
  }

  /// Attempts to put the two halves of a TcpStream back together and recover the original socket. Succeeds only if the two halves originated from the same call to [`into_split`][TcpStream::into_split].
  fn reunite(
    read: Self::OwnedReadHalf,
    write: Self::OwnedWriteHalf,
  ) -> Result<Self, Self::ReuniteError>
  where
    Self: Sized;
}

/// An abstraction layer for TCP listener.
pub trait TcpListener:
  TryFrom<std::net::TcpListener, Error = io::Error> + Fd + Unpin + Send + Sync + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;
  /// Stream of incoming connections.
  type Stream: TcpStream<Runtime = Self::Runtime>;

  /// A stream of incoming TCP connections.
  ///
  /// This stream is infinite, i.e awaiting the next connection will never result in [`None`]. It is
  /// created by the [`TcpListener::incoming()`] method.
  type Incoming<'a>: futures_util::stream::Stream<Item = io::Result<Self::Stream>>
    + Send
    + Sync
    + Unpin
    + 'a;

  /// Creates a new TcpListener, which will be bound to the specified address.
  ///
  /// The returned listener is ready for accepting connections.
  ///
  /// Binding with a port number of 0 will request that the OS assigns a port
  /// to this listener. The port allocated can be queried via the `local_addr`
  /// method.
  ///
  /// The address type can be any implementor of the [`ToSocketAddrs`] trait.
  /// If `addr` yields multiple addresses, bind will be attempted with each of
  /// the addresses until one succeeds and returns the listener. If none of
  /// the addresses succeed in creating a listener, the error returned from
  /// the last attempt (the last address) is returned.
  ///
  /// This function sets the `SO_REUSEADDR` option on the socket.
  fn bind<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
  ) -> impl Future<Output = io::Result<Self>> + Send
  where
    Self: Sized;

  /// Accepts a new incoming connection from this listener.
  ///
  /// This function will yield once a new TCP connection is established. When established,
  /// the corresponding [`TcpStream`] and the remote peer's address will be returned.
  fn accept(&self) -> impl Future<Output = io::Result<(Self::Stream, SocketAddr)>> + Send;

  /// Returns a stream of incoming connections.
  ///
  /// Iterating over this stream is equivalent to calling [`accept()`][`TcpListener::accept()`]
  /// in a loop. The stream of connections is infinite, i.e awaiting the next connection will
  /// never result in [`None`].
  ///
  /// See also [`TcpListener::into_incoming`].
  fn incoming(&self) -> Self::Incoming<'_>;

  /// Turn this into a stream over the connections being received on this
  /// listener.
  ///
  /// The returned stream is infinite and will also not yield
  /// the peer's [`SocketAddr`] structure. Iterating over it is equivalent to
  /// calling [`TcpListener::accept`] in a loop.
  ///
  /// See also [`TcpListener::incoming`].
  fn into_incoming(
    self,
  ) -> impl futures_util::stream::Stream<Item = io::Result<Self::Stream>> + Send;

  /// Returns the local address that this listener is bound to.
  ///
  /// This can be useful, for example, when binding to port 0 to figure out which port was actually bound.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Sets the time-to-live value for this socket.  
  fn set_ttl(&self, ttl: u32) -> io::Result<()>;

  /// Gets the time-to-live value of this socket.
  fn ttl(&self) -> io::Result<u32>;

  /// Creates a new independently owned handle to the underlying socket.
  ///
  /// The returned `UdpSocket` is a reference to the same socket that this
  /// object references. Both handles will read and write the same port, and
  /// options set on one socket will be propagated to the other.
  fn try_clone(&self) -> io::Result<Self> {
    super::os::duplicate::<_, std::net::TcpListener>(self).and_then(Self::try_from)
  }
}
