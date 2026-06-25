//! [`TcpStream`], [`TcpListener`] and their owned-half abstractions, plus the named future types
//! returned by their async methods.

use std::{
  future::Future,
  io,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

use agnostic_lite::{RuntimeLite, time::Elapsed};

use super::{
  Fd, ToSocketAddrs,
  io::{AsyncRead, AsyncReadWrite, AsyncWrite},
};

#[cfg(any(feature = "smol", feature = "tokio"))]
macro_rules! tcp_listener_common_methods {
  ($ty:ident.$field:ident) => {
    type Bind<A>
      = $crate::tcp::Bind<Self, A>
    where
      A: $crate::ToSocketAddrs<Self::Runtime>,
      Self: Sized;

    type Accept<'a>
      = $crate::tcp::Accept<'a, Self>
    where
      Self: 'a;

    fn bind<A: $crate::ToSocketAddrs<Self::Runtime>>(addr: A) -> Self::Bind<A>
    where
      Self: Sized,
    {
      $crate::tcp::Bind::new(addr)
    }

    fn accept(&self) -> Self::Accept<'_> {
      $crate::tcp::Accept::new(self)
    }

    fn local_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.local_addr()
    }
  };
}

#[cfg(any(feature = "smol", feature = "tokio"))]
macro_rules! tcp_stream_common_methods {
  ($field:ident) => {
    type Connect<A>
      = $crate::tcp::Connect<Self, A>
    where
      A: $crate::ToSocketAddrs<Self::Runtime>,
      Self: Sized;

    type ConnectTimeout<A>
      = $crate::tcp::ConnectTimeout<Self, A>
    where
      A: $crate::ToSocketAddrs<Self::Runtime>,
      Self: Sized;

    type Peek<'a>
      = $crate::tcp::Peek<'a, Self>
    where
      Self: 'a;

    fn connect<A: $crate::ToSocketAddrs<Self::Runtime>>(addr: A) -> Self::Connect<A>
    where
      Self: Sized,
    {
      $crate::tcp::Connect::new(addr)
    }

    fn connect_timeout<A: $crate::ToSocketAddrs<Self::Runtime>>(
      addr: A,
      timeout: ::std::time::Duration,
    ) -> Self::ConnectTimeout<A>
    where
      Self: Sized,
    {
      $crate::tcp::ConnectTimeout::new(addr, timeout)
    }

    fn peek<'a>(&'a self, buf: &'a mut [u8]) -> Self::Peek<'a> {
      $crate::tcp::Peek::new(self, buf)
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

#[cfg(any(feature = "smol", feature = "tokio"))]
macro_rules! tcp_stream_owned_read_half_common_methods {
  ($field:ident) => {
    type Peek<'a>
      = $crate::tcp::ReadHalfPeek<'a, Self>
    where
      Self: 'a;

    fn local_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.local_addr()
    }

    fn peer_addr(&self) -> ::std::io::Result<::std::net::SocketAddr> {
      self.$field.peer_addr()
    }

    fn peek<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Peek<'a> {
      $crate::tcp::ReadHalfPeek::new(self, buf)
    }
  };
}

#[cfg(any(feature = "smol", feature = "tokio"))]
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
    S: TcpListener,
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
  /// The future returned by [`TcpListener::bind`].
  pub struct Bind<S, A>
  where
    S: TcpListener,
    A: ToSocketAddrs<S::Runtime>,
  {
    #[pin]
    state: BindState<S, A>,
  }
}

impl<S, A> Bind<S, A>
where
  S: TcpListener,
  A: ToSocketAddrs<S::Runtime>,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
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
  S: TcpListener,
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
            // The listener must be non-blocking before handing it to a runtime: `tokio`'s
            // `from_std` rejects a blocking fd, and `smol`/`async-net` expects it too.
            let res = std::net::TcpListener::bind(addr).and_then(|ln| {
              ln.set_nonblocking(true)?;
              S::try_from(ln)
            });
            match res {
              Ok(ln) => return Poll::Ready(Ok(ln)),
              Err(e) => last_err = Some(e),
            }
          }
          return Poll::Ready(Err(last_err.unwrap_or_else(no_addresses_error)));
        }
      }
    }
  }
}

/// The future returned by [`TcpListener::accept`].
pub struct Accept<'a, S: ?Sized> {
  listener: &'a S,
}

impl<'a, S> Accept<'a, S>
where
  S: TcpListener,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(listener: &'a S) -> Self {
    Self { listener }
  }
}

impl<S> Future for Accept<'_, S>
where
  S: TcpListener,
{
  type Output = io::Result<(S::Stream, SocketAddr)>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.get_mut().listener.poll_accept(cx)
  }
}

/// A stream of incoming TCP connections.
///
/// This stream is infinite, i.e awaiting the next connection will never result in [`None`]. It is
/// created by the [`TcpListener::incoming`] method.
pub struct Incoming<'a, S: ?Sized> {
  listener: &'a S,
}

impl<'a, S> Incoming<'a, S>
where
  S: TcpListener,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(listener: &'a S) -> Self {
    Self { listener }
  }
}

impl<S> futures_util::stream::Stream for Incoming<'_, S>
where
  S: TcpListener,
{
  type Item = io::Result<S::Stream>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self
      .get_mut()
      .listener
      .poll_accept(cx)
      .map(|res| Some(res.map(|(stream, _)| stream)))
  }
}

/// A stream over the connections being received on a [`TcpListener`].
///
/// This stream is infinite, i.e awaiting the next connection will never result in [`None`]. It is
/// created by the [`TcpListener::into_incoming`] method.
pub struct IntoIncoming<S> {
  listener: S,
}

impl<S> IntoIncoming<S>
where
  S: TcpListener,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(listener: S) -> Self {
    Self { listener }
  }
}

impl<S> futures_util::stream::Stream for IntoIncoming<S>
where
  S: TcpListener,
{
  type Item = io::Result<S::Stream>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self
      .get_mut()
      .listener
      .poll_accept(cx)
      .map(|res| Some(res.map(|(stream, _)| stream)))
  }
}

pin_project_lite::pin_project! {
  #[project = ConnectStateProj]
  enum ConnectState<S, A>
  where
    S: TcpStream,
    A: ToSocketAddrs<S::Runtime>,
  {
    Resolving {
      #[pin]
      fut: A::Future,
    },
    // Resolution finished; the next address is dialed (non-blocking) on the next poll.
    Connecting {
      addrs: A::Iter,
      // The stream whose write-readiness is currently being awaited; `None` between attempts.
      stream: Option<S>,
      last_err: Option<io::Error>,
    },
  }
}

pin_project_lite::pin_project! {
  /// The future returned by [`TcpStream::connect`].
  pub struct Connect<S, A>
  where
    S: TcpStream,
    A: ToSocketAddrs<S::Runtime>,
  {
    #[pin]
    state: ConnectState<S, A>,
  }
}

impl<S, A> Connect<S, A>
where
  S: TcpStream,
  A: ToSocketAddrs<S::Runtime>,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(addr: A) -> Self {
    Self {
      state: ConnectState::Resolving {
        fut: addr.to_socket_addrs(),
      },
    }
  }
}

// Dials `addr` in non-blocking mode and hands the connecting socket to the runtime reactor. The
// returned stream is registered but its connection is not yet established; write-readiness must be
// awaited (and `take_error` checked) before it can be used.
fn start_connect<S>(addr: SocketAddr) -> io::Result<S>
where
  S: TcpStream,
{
  let domain = socket2::Domain::for_address(addr);
  let socket = socket2::Socket::new(domain, socket2::Type::STREAM, Some(socket2::Protocol::TCP))?;
  socket.set_nonblocking(true)?;
  match socket.connect(&addr.into()) {
    Ok(()) => {}
    Err(e) if is_connect_in_progress(&e) => {}
    Err(e) => return Err(e),
  }
  S::try_from(std::net::TcpStream::from(socket))
}

// A non-blocking `connect` does not complete inline; it reports that the handshake is in progress
// and completes once the socket becomes writable. On unix this is `EINPROGRESS`; on Windows it
// surfaces as `WouldBlock` (`WSAEWOULDBLOCK`).
#[inline]
fn is_connect_in_progress(e: &io::Error) -> bool {
  if e.kind() == io::ErrorKind::WouldBlock {
    return true;
  }
  #[cfg(unix)]
  {
    e.raw_os_error() == Some(rustix::io::Errno::INPROGRESS.raw_os_error())
  }
  #[cfg(not(unix))]
  {
    false
  }
}

impl<S, A> Future for Connect<S, A>
where
  S: TcpStream,
  A: ToSocketAddrs<S::Runtime>,
{
  type Output = io::Result<S>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let mut state = self.project().state;
    loop {
      match state.as_mut().project() {
        ConnectStateProj::Resolving { fut } => {
          let addrs = futures_util::ready!(fut.poll(cx))?;
          state.set(ConnectState::Connecting {
            addrs,
            stream: None,
            last_err: None,
          });
        }
        ConnectStateProj::Connecting {
          addrs,
          stream,
          last_err,
        } => {
          if stream.is_none() {
            // Dial the next candidate address; on a synchronous failure record it and try the next.
            loop {
              match addrs.next() {
                Some(addr) => match start_connect::<S>(addr) {
                  Ok(s) => {
                    *stream = Some(s);
                    break;
                  }
                  Err(e) => *last_err = Some(e),
                },
                None => {
                  return Poll::Ready(Err(last_err.take().unwrap_or_else(no_addresses_error)));
                }
              }
            }
          }

          let s = stream.as_ref().expect("`stream` was just set");
          match s.poll_write_ready(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Err(e)) => {
              *last_err = Some(e);
              *stream = None;
            }
            Poll::Ready(Ok(())) => {
              // Write-readiness only means the handshake finished; `SO_ERROR` distinguishes a
              // successful connect from a deferred failure (e.g. connection refused).
              match socket2::SockRef::from(s).take_error() {
                Ok(None) => return Poll::Ready(Ok(stream.take().expect("stream is `Some`"))),
                Ok(Some(e)) | Err(e) => {
                  *last_err = Some(e);
                  *stream = None;
                }
              }
            }
          }
        }
      }
    }
  }
}

pin_project_lite::pin_project! {
  /// The future returned by [`TcpStream::connect_timeout`].
  pub struct ConnectTimeout<S, A>
  where
    S: TcpStream,
    A: ToSocketAddrs<S::Runtime>,
    Connect<S, A>: Send,
  {
    #[pin]
    timeout: <S::Runtime as RuntimeLite>::Timeout<Connect<S, A>>,
  }
}

impl<S, A> ConnectTimeout<S, A>
where
  S: TcpStream,
  A: ToSocketAddrs<S::Runtime>,
  Connect<S, A>: Send,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(addr: A, timeout: Duration) -> Self {
    Self {
      timeout: <S::Runtime as RuntimeLite>::timeout(timeout, Connect::new(addr)),
    }
  }
}

impl<S, A> Future for ConnectTimeout<S, A>
where
  S: TcpStream,
  A: ToSocketAddrs<S::Runtime>,
  Connect<S, A>: Send,
  <S::Runtime as RuntimeLite>::Timeout<Connect<S, A>>:
    Future<Output = Result<io::Result<S>, Elapsed>>,
{
  type Output = io::Result<S>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    match futures_util::ready!(self.project().timeout.poll(cx)) {
      Ok(res) => Poll::Ready(res),
      Err(elapsed) => Poll::Ready(Err(elapsed.into())),
    }
  }
}

/// The future returned by [`TcpStream::peek`].
pub struct Peek<'a, S: ?Sized> {
  stream: &'a S,
  buf: &'a mut [u8],
}

impl<'a, S> Peek<'a, S>
where
  S: TcpStream,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(stream: &'a S, buf: &'a mut [u8]) -> Self {
    Self { stream, buf }
  }
}

impl<S> Future for Peek<'_, S>
where
  S: TcpStream,
{
  type Output = io::Result<usize>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    this.stream.poll_peek(cx, this.buf)
  }
}

/// The future returned by [`OwnedReadHalf::peek`].
pub struct ReadHalfPeek<'a, S: ?Sized> {
  stream: &'a mut S,
  buf: &'a mut [u8],
}

impl<'a, S> ReadHalfPeek<'a, S>
where
  S: OwnedReadHalf,
{
  #[cfg(any(feature = "tokio", feature = "smol"))]
  #[inline]
  pub(crate) fn new(stream: &'a mut S, buf: &'a mut [u8]) -> Self {
    Self { stream, buf }
  }
}

impl<S> Future for ReadHalfPeek<'_, S>
where
  S: OwnedReadHalf,
{
  type Output = io::Result<usize>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let this = self.get_mut();
    this.stream.poll_peek(cx, this.buf)
  }
}

/// The abstraction of a owned read half of a TcpStream.
pub trait OwnedReadHalf: AsyncRead + Unpin + 'static {
  /// The async runtime.
  type Runtime: RuntimeLite;

  /// The future returned by [`peek`](OwnedReadHalf::peek).
  type Peek<'a>: Future<Output = io::Result<usize>>
  where
    Self: 'a;

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
  fn peek<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::Peek<'a>;

  /// Attempts to receive data on the socket, without removing it from the queue.
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
  /// * `Poll::Ready(Ok(n))` `n` is the number of bytes peeked.
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_peek(&mut self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>>;
}

/// The abstraction of a owned write half of a TcpStream.
pub trait OwnedWriteHalf: AsyncWrite + Unpin + 'static {
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
pub trait ReuniteError<T>: core::error::Error + Unpin + 'static
where
  T: TcpStream,
{
  /// Consumes the error and returns the read half and write half of the socket.
  fn into_components(self) -> (T::OwnedReadHalf, T::OwnedWriteHalf);
}

/// The abstraction of a TCP stream.
pub trait TcpStream:
  TryFrom<std::net::TcpStream, Error = io::Error> + Fd + AsyncReadWrite + Unpin + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;
  /// The owned read half of the stream.
  type OwnedReadHalf: OwnedReadHalf;
  /// The owned write half of the stream.
  type OwnedWriteHalf: OwnedWriteHalf;
  /// Error indicating that two halves were not from the same socket, and thus could not be reunited.
  type ReuniteError: ReuniteError<Self>;

  /// The future returned by [`connect`](TcpStream::connect).
  type Connect<A>: Future<Output = io::Result<Self>>
  where
    A: ToSocketAddrs<Self::Runtime>,
    Self: Sized;

  /// The future returned by [`connect_timeout`](TcpStream::connect_timeout).
  type ConnectTimeout<A>: Future<Output = io::Result<Self>>
  where
    A: ToSocketAddrs<Self::Runtime>,
    Self: Sized;

  /// The future returned by [`peek`](TcpStream::peek).
  type Peek<'a>: Future<Output = io::Result<usize>>
  where
    Self: 'a;

  /// Connects to the specified address.
  fn connect<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> Self::Connect<A>
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
  fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> Self::ConnectTimeout<A>
  where
    Self: Sized;

  /// Receives data on the socket from the remote address to which it is connected, without
  /// removing that data from the queue.
  ///
  /// On success, returns the number of bytes peeked.
  ///
  /// Successive calls return the same data. This is accomplished by passing `MSG_PEEK` as a flag
  /// to the underlying `recv` system call.
  fn peek<'a>(&'a self, buf: &'a mut [u8]) -> Self::Peek<'a>;

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

  /// Receives data on the socket from the remote address to which it is connected, without
  /// removing that data from the queue.
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
  /// * `Poll::Ready(Ok(n))` `n` is the number of bytes peeked.
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_peek(&self, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>>;

  /// Attempts to complete an in-progress connection, by polling for write-readiness.
  ///
  /// This is used to drive a non-blocking [`connect`](TcpStream::connect) to completion: the
  /// connection handshake has finished once the socket reports write-readiness (a successful
  /// connect, or a deferred error surfaced via `SO_ERROR`).
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if the socket is not yet writable
  /// * `Poll::Ready(Ok(()))` once the socket is writable
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
}

/// An abstraction layer for TCP listener.
pub trait TcpListener:
  TryFrom<std::net::TcpListener, Error = io::Error> + Fd + Unpin + 'static
{
  /// The async runtime.
  type Runtime: RuntimeLite;
  /// Stream of incoming connections.
  type Stream: TcpStream<Runtime = Self::Runtime>;

  /// The future returned by [`bind`](TcpListener::bind).
  type Bind<A>: Future<Output = io::Result<Self>>
  where
    A: ToSocketAddrs<Self::Runtime>,
    Self: Sized;

  /// The future returned by [`accept`](TcpListener::accept).
  type Accept<'a>: Future<Output = io::Result<(Self::Stream, SocketAddr)>>
  where
    Self: 'a;

  /// A stream of incoming TCP connections.
  ///
  /// This stream is infinite, i.e awaiting the next connection will never result in [`None`]. It is
  /// created by the [`TcpListener::incoming()`] method.
  type Incoming<'a>: futures_util::stream::Stream<Item = io::Result<Self::Stream>> + 'a
  where
    Self: 'a;

  /// A stream over the connections being received on this listener, created by
  /// [`TcpListener::into_incoming`].
  type IntoIncoming: futures_util::stream::Stream<Item = io::Result<Self::Stream>>;

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
  fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> Self::Bind<A>
  where
    Self: Sized;

  /// Accepts a new incoming connection from this listener.
  ///
  /// This function will yield once a new TCP connection is established. When established,
  /// the corresponding [`TcpStream`] and the remote peer's address will be returned.
  fn accept(&self) -> Self::Accept<'_>;

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
  fn into_incoming(self) -> Self::IntoIncoming
  where
    Self: Sized;

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

  /// Attempts to accept a new incoming connection from this listener.
  ///
  /// Note that on multiple calls to `poll_accept`, only the `Waker` from the `Context` passed to
  /// the most recent call will be scheduled to receive a wakeup.
  ///
  /// # Return value
  ///
  /// The function returns:
  ///
  /// * `Poll::Pending` if no connection is ready to be accepted
  /// * `Poll::Ready(Ok((stream, addr)))` on a newly accepted connection
  /// * `Poll::Ready(Err(e))` if an error is encountered.
  ///
  /// # Errors
  ///
  /// This function may encounter any standard I/O error except `WouldBlock`.
  fn poll_accept(&self, cx: &mut Context<'_>) -> Poll<io::Result<(Self::Stream, SocketAddr)>>;
}
