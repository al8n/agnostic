use super::{Fd, TcpListener, TcpStream};

use std::{io, net::SocketAddr, time::Duration};

/// A TCP socket that has not yet been converted to a `TcpStream` or `TcpListener`.
pub trait TcpSocket:
  TryFrom<std::net::TcpStream, Error = io::Error> + Fd + Unpin + Send + Sync + 'static
{
  /// The stream type that this socket can be converted into.
  type Stream: TcpStream;
  /// The listener type that this socket can be converted into.
  type Listener: TcpListener;
  /// The runtime that this socket is associated with.
  type Runtime: super::RuntimeLite;

  /// Creates a new socket configured for IPv4.
  ///
  /// Calls `socket(2)` with `AF_INET` and `SOCK_STREAM`.
  fn new_v4() -> io::Result<Self>;

  /// Creates a new socket configured for IPv6.
  ///
  /// Calls `socket(2)` with `AF_INET6` and `SOCK_STREAM`.
  fn new_v6() -> io::Result<Self>;

  /// Binds the socket to the given address.
  ///
  /// This calls the bind(2) operating-system function.
  /// Behavior is platform specific. Refer to the target platform’s documentation for more details.
  fn bind(&self, addr: SocketAddr) -> io::Result<()>;

  /// Establishes a TCP connection with a peer at the specified socket address.
  ///
  /// The `TcpSocket` is consumed. Once the connection is established, a
  /// connected [`TcpStream`] is returned. If the connection fails, the
  /// encountered error is returned.
  ///
  /// [`TcpStream`]: TcpStream
  ///
  /// This calls the `connect(2)` operating-system function. Behavior is
  /// platform specific. Refer to the target platform's documentation for more
  /// details.
  fn connect(self, addr: SocketAddr) -> impl Future<Output = io::Result<Self::Stream>> + Send;

  /// Converts the socket into a `TcpListener`.
  ///
  /// `backlog` defines the maximum number of pending connections are queued
  /// by the operating system at any given time. Connection are removed from
  /// the queue with [`TcpListener::accept`]. When the queue is full, the
  /// operating-system will start rejecting connections.
  ///
  /// [`TcpListener::accept`]: TcpListener::accept
  ///
  /// This calls the `listen(2)` operating-system function, marking the socket
  /// as a passive socket. Behavior is platform specific. Refer to the target
  /// platform's documentation for more details.
  fn listen(self, backlog: u32) -> io::Result<Self::Listener>;

  /// Gets the local address of this socket.
  ///
  /// Will fail on windows if called before bind.
  fn local_addr(&self) -> io::Result<SocketAddr>;

  /// Sets value for the `SO_KEEPALIVE` option on this socket.
  fn set_keepalive(&self, keepalive: bool) -> io::Result<()>;

  /// Gets the value of the `SO_KEEPALIVE` option on this socket.
  fn keepalive(&self) -> io::Result<bool>;

  /// Allows the socket to bind to an in-use address.
  ///
  /// Behavior is platform specific. Refer to the target platform's
  /// documentation for more details.
  fn set_reuseaddr(&self, reuseaddr: bool) -> io::Result<()>;

  /// Retrieves the value set for `SO_REUSEADDR` on this socket.
  fn reuseaddr(&self) -> io::Result<bool>;

  /// Sets the value of the `TCP_NODELAY` option on this socket.
  ///
  /// If set, this option disables the Nagle algorithm. This means that segments are always
  /// sent as soon as possible, even if there is only a small amount of data. When not set,
  /// data is buffered until there is a sufficient amount to send out, thereby avoiding
  /// the frequent sending of small packets.
  fn set_nodelay(&self, nodelay: bool) -> io::Result<()>;

  /// Gets the value of the `TCP_NODELAY` option on this socket.
  ///
  /// For more information about this option, see [`set_nodelay`].
  ///
  /// [`set_nodelay`]: TcpSocket::set_nodelay
  fn nodelay(&self) -> io::Result<bool>;

  /// Sets the linger duration of this socket by setting the `SO_LINGER` option.
  ///
  /// This option controls the action taken when a stream has unsent messages and the stream is
  /// closed. If `SO_LINGER` is set, the system shall block the process until it can transmit the
  /// data or until the time expires.
  ///
  /// If `SO_LINGER` is not specified, and the socket is closed, the system handles the call in a
  /// way that allows the process to continue as quickly as possible.
  fn set_linger(&self, dur: Option<Duration>) -> io::Result<()>;

  /// Reads the linger duration for this socket by getting the `SO_LINGER`
  /// option.
  ///
  /// For more information about this option, see [`set_linger`].
  ///
  /// [`set_linger`]: TcpSocket::set_linger
  fn linger(&self) -> io::Result<Option<Duration>>;

  /// Sets the size of the TCP send buffer on this socket.
  ///
  /// On most operating systems, this sets the `SO_SNDBUF` socket option.
  fn set_send_buffer_size(&self, size: u32) -> io::Result<()>;

  /// Returns the size of the TCP send buffer for this socket.
  ///
  /// On most operating systems, this is the value of the `SO_SNDBUF` socket
  /// option.
  ///
  /// Note that if [`set_send_buffer_size`] has been called on this socket
  /// previously, the value returned by this function may not be the same as
  /// the argument provided to `set_send_buffer_size`. This is for the
  /// following reasons:
  ///
  /// * Most operating systems have minimum and maximum allowed sizes for the
  ///   send buffer, and will clamp the provided value if it is below the
  ///   minimum or above the maximum. The minimum and maximum buffer sizes are
  ///   OS-dependent.
  /// * Linux will double the buffer size to account for internal bookkeeping
  ///   data, and returns the doubled value from `getsockopt(2)`. As per `man
  ///   7 socket`:
  ///   > Sets or gets the maximum socket send buffer in bytes. The
  ///   > kernel doubles this value (to allow space for bookkeeping
  ///   > overhead) when it is set using `setsockopt(2)`, and this doubled
  ///   > value is returned by `getsockopt(2)`.
  ///
  /// [`set_send_buffer_size`]: #method.set_send_buffer_size
  fn send_buffer_size(&self) -> io::Result<u32>;

  /// Sets the size of the TCP receive buffer on this socket.
  ///
  /// On most operating systems, this sets the `SO_RCVBUF` socket option.
  fn set_recv_buffer_size(&self, size: u32) -> io::Result<()>;

  /// Returns the size of the TCP receive buffer for this socket.
  ///
  /// On most operating systems, this is the value of the `SO_RCVBUF` socket
  /// option.
  ///
  /// Note that if [`set_recv_buffer_size`] has been called on this socket
  /// previously, the value returned by this function may not be the same as
  /// the argument provided to `set_send_buffer_size`. This is for the
  /// following reasons:
  ///
  /// * Most operating systems have minimum and maximum allowed sizes for the
  ///   receive buffer, and will clamp the provided value if it is below the
  ///   minimum or above the maximum. The minimum and maximum buffer sizes are
  ///   OS-dependent.
  /// * Linux will double the buffer size to account for internal bookkeeping
  ///   data, and returns the doubled value from `getsockopt(2)`. As per `man
  ///   7 socket`:
  ///   > Sets or gets the maximum socket send buffer in bytes. The
  ///   > kernel doubles this value (to allow space for bookkeeping
  ///   > overhead) when it is set using `setsockopt(2)`, and this doubled
  ///   > value is returned by `getsockopt(2)`.
  ///
  /// [`set_recv_buffer_size`]: #method.set_recv_buffer_size
  fn recv_buffer_size(&self) -> io::Result<u32>;

  /// Allows the socket to bind to an in-use port. Only available for unix systems
  /// (excluding Solaris & Illumos).
  ///
  /// Behavior is platform specific. Refer to the target platform's
  /// documentation for more details.
  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))
  )]
  fn set_reuseport(&self, reuesport: bool) -> io::Result<()>;

  /// Allows the socket to bind to an in-use port. Only available for unix systems
  /// (excluding Solaris & Illumos).
  ///
  /// Behavior is platform specific. Refer to the target platform's
  /// documentation for more details.
  #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
  #[cfg_attr(
    docsrs,
    doc(cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos"))))
  )]
  fn reuseport(&self) -> io::Result<bool>;

  /// Gets the value of the `IP_TOS` option for this socket.
  ///
  /// For more information about this option, see [`set_tos`].
  ///
  /// **NOTE:** On Windows, `IP_TOS` is only supported on [Windows 8+ or
  /// Windows Server 2012+.](https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options)
  ///
  /// [`set_tos`]: Self::set_tos
  // https://docs.rs/socket2/0.5.3/src/socket2/socket.rs.html#1464
  #[cfg(not(any(
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "haiku"
  )))]
  #[cfg_attr(
    docsrs,
    doc(cfg(not(any(
      target_os = "fuchsia",
      target_os = "redox",
      target_os = "solaris",
      target_os = "illumos",
      target_os = "haiku"
    ))))
  )]
  fn tos(&self) -> io::Result<u32>;

  /// Sets the value for the `IP_TOS` option on this socket.
  ///
  /// This value sets the type-of-service field that is used in every packet
  /// sent from this socket.
  ///
  /// **NOTE:** On Windows, `IP_TOS` is only supported on [Windows 8+ or
  /// Windows Server 2012+.](https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options)
  // https://docs.rs/socket2/0.5.3/src/socket2/socket.rs.html#1446
  #[cfg(not(any(
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
    target_os = "haiku"
  )))]
  #[cfg_attr(
    docsrs,
    doc(cfg(not(any(
      target_os = "fuchsia",
      target_os = "redox",
      target_os = "solaris",
      target_os = "illumos",
      target_os = "haiku"
    ))))
  )]
  fn set_tos(&self, tos: u32) -> io::Result<()>;

  // TODO(al8n): uncomment below code when rustix supports SO_BINDTODEVICE
  // /// Gets the value for the SO_BINDTODEVICE option on this socket
  // ///
  // /// This value gets the socket binded device’s interface name.
  // #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
  // #[cfg_attr(
  //   docsrs,
  //   doc(cfg(all(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))))
  // )]
  // fn device(&self) -> io::Result<Option<Vec<u8>>>;

  // /// Sets the value for the `SO_BINDTODEVICE` option on this socket
  // ///
  // /// If a socket is bound to an interface, only packets received from that
  // /// particular interface are processed by the socket. Note that this only
  // /// works for some socket types, particularly `AF_INET` sockets.
  // ///
  // /// If `interface` is `None` or an empty string it removes the binding.
  // #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
  // #[cfg_attr(
  //   docsrs,
  //   doc(cfg(all(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))))
  // )]
  // fn bind_device(&self, interface: Option<&[u8]>) -> io::Result<()>;
}
