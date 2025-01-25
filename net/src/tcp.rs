use std::{future::Future, io, net::SocketAddr};

use agnostic_lite::RuntimeLite;

use super::{
  io::{AsyncRead, AsyncReadWrite, AsyncWrite},
  As, ToSocketAddrs,
};

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

      ::core::result::Result::Err(last_err.unwrap_or_else(|| {
        ::std::io::Error::new(
          ::std::io::ErrorKind::InvalidInput,
          "could not resolve to any address",
        )
      }))
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

      ::core::result::Result::Err(last_err.unwrap_or_else(|| {
        ::std::io::Error::new(
          ::std::io::ErrorKind::InvalidInput,
          "could not resolve to any address",
        )
      }))
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

/// The abstraction of a TCP stream.
pub trait TcpStream:
  TryFrom<std::net::TcpStream, Error = io::Error>
  + TryFrom<socket2::Socket, Error = io::Error>
  + As
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
  type ReuniteError: core::error::Error + Unpin + Send + Sync + 'static;

  /// Connects to the specified address.
  fn connect<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
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
    super::shutdown(self, how)
  }

  /// Creates a new independently owned handle to the underlying socket.
  ///
  /// The returned `UdpSocket` is a reference to the same socket that this
  /// object references. Both handles will read and write the same port, and
  /// options set on one socket will be propagated to the other.
  fn try_clone(&self) -> io::Result<Self> {
    super::duplicate(self).and_then(Self::try_from)
  }

  /// Get the value of the `IPV6_V6ONLY` option for this socket.
  fn only_v6(&self) -> io::Result<bool> {
    super::only_v6(self)
  }

  /// Gets the value of the `SO_LINGER` option on this socket.
  ///
  /// For more information about this option, see [`TcpStream::set_linger`].
  fn linger(&self) -> io::Result<Option<std::time::Duration>> {
    super::linger(self)
  }

  /// Sets the value of the `SO_LINGER` option on this socket.
  ///
  /// This value controls how the socket is closed when data remains to be sent.
  /// If `SO_LINGER` is set, the socket will remain open for the specified duration as the system attempts to send pending data.
  /// Otherwise, the system may close the socket immediately, or wait for a default timeout.
  fn set_linger(&self, duration: Option<std::time::Duration>) -> io::Result<()> {
    super::set_linger(self, duration)
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
  TryFrom<std::net::TcpListener, Error = io::Error>
  + TryFrom<socket2::Socket, Error = io::Error>
  + As
  + Unpin
  + Send
  + Sync
  + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;
  /// Stream of incoming connections.
  type Stream: TcpStream<Runtime = Self::Runtime>;

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
    super::duplicate(self).and_then(Self::try_from)
  }
}
