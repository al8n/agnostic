use std::{
  future::Future,
  io,
  net::SocketAddr,
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

use atomic::{Atomic, Ordering};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  net::{TcpListener, TcpStream, UdpSocket},
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{
  net::{Net, ToSocketAddrs},
  Runtime,
};

use super::TokioRuntime;

#[derive(Debug, Default, Clone, Copy)]
pub struct TokioNet;

impl Net for TokioNet {
  type TcpListener = TokioTcpListener;

  type TcpStream = TokioTcpStream;

  type UdpSocket = TokioUdpSocket;
}

pub struct TokioTcpListener {
  ln: TcpListener,
  write_timeout: Atomic<Option<Duration>>,
  read_timeout: Atomic<Option<Duration>>,
}

#[async_trait::async_trait]
impl crate::net::TcpListener for TokioTcpListener {
  type Stream = TokioTcpStream;
  type Runtime = TokioRuntime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs(&TokioRuntime).await?;

    let res = if addrs.size_hint().0 <= 1 {
      if let Some(addr) = addrs.next() {
        TcpListener::bind(addr).await
      } else {
        return Err(io::Error::new(
          io::ErrorKind::InvalidInput,
          "invalid socket address",
        ));
      }
    } else {
      TcpListener::bind(addrs.collect::<Vec<_>>().as_slice()).await
    };

    res.map(|ln| Self {
      ln,
      write_timeout: Atomic::new(None),
      read_timeout: Atomic::new(None),
    })
  }

  async fn accept(&self) -> io::Result<(Self::Stream, SocketAddr)> {
    self.ln.accept().await.map(|(stream, addr)| {
      (
        TokioTcpStream {
          stream,
          write_timeout: Atomic::new(self.write_timeout.load(Ordering::SeqCst)),
          read_timeout: Atomic::new(self.read_timeout.load(Ordering::SeqCst)),
        },
        addr,
      )
    })
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.ln.local_addr()
  }

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::SeqCst);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::SeqCst)
  }

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::SeqCst);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::SeqCst)
  }
}

pub struct TokioTcpStream {
  stream: TcpStream,
  write_timeout: Atomic<Option<Duration>>,
  read_timeout: Atomic<Option<Duration>>,
}

impl futures_util::AsyncRead for TokioTcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut [u8],
  ) -> Poll<io::Result<usize>> {
    if let Some(d) = self.read_timeout.load(Ordering::Relaxed) {
      if !d.is_zero() {
        let timeout = TokioRuntime.timeout(d, self.stream.read(buf));
        tokio::pin!(timeout);
        match timeout.poll(cx) {
          Poll::Ready(rst) => match rst {
            Ok(rst) => return Poll::Ready(rst),
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, e))),
          },
          Poll::Pending => return Poll::Pending,
        }
      }
    }

    Pin::new(&mut (&mut self.stream).compat()).poll_read(cx, buf)
  }
}

impl futures_util::AsyncWrite for TokioTcpStream {
  fn poll_write(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
    buf: &[u8],
  ) -> std::task::Poll<io::Result<usize>> {
    if let Some(d) = self.read_timeout.load(Ordering::Relaxed) {
      if !d.is_zero() {
        let timeout = TokioRuntime.timeout(d, self.stream.write(buf));
        tokio::pin!(timeout);
        match timeout.poll(cx) {
          Poll::Ready(rst) => match rst {
            Ok(rst) => return Poll::Ready(rst),
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::TimedOut, e))),
          },
          Poll::Pending => return Poll::Pending,
        }
      }
    }

    Pin::new(&mut (&mut self.stream).compat_write()).poll_write(cx, buf)
  }

  fn poll_flush(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_flush(cx)
  }

  fn poll_close(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<io::Result<()>> {
    Pin::new(&mut (&mut self.stream).compat_write()).poll_close(cx)
  }
}

impl tokio::io::AsyncRead for TokioTcpStream {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
    buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncReadCompatExt::compat(
      self.get_mut(),
    ))
    .poll_read(cx, buf)
  }
}

impl tokio::io::AsyncWrite for TokioTcpStream {
  fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
    Pin::new(&mut tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(self.get_mut()))
      .poll_write(cx, buf)
  }

  fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_flush(cx)
  }

  fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
    Pin::new(&mut self.stream).poll_shutdown(cx)
  }
}

#[async_trait::async_trait]
impl crate::net::TcpStream for TokioTcpStream {
  type Runtime = TokioRuntime;

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs(&TokioRuntime).await?;

    let res = if addrs.size_hint().0 <= 1 {
      if let Some(addr) = addrs.next() {
        TcpStream::connect(addr).await
      } else {
        return Err(io::Error::new(
          io::ErrorKind::InvalidInput,
          "invalid socket address",
        ));
      }
    } else {
      TcpStream::connect(&addrs.collect::<Vec<_>>().as_slice()).await
    };

    res.map(|stream| Self {
      stream,
      write_timeout: Atomic::new(None),
      read_timeout: Atomic::new(None),
    })
  }

  async fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
  where
    Self: Sized,
  {
    TokioRuntime
      .timeout(timeout, Self::connect(addr))
      .await
      .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
      .and_then(|res| res)
  }

  fn local_addr(&self) -> io::Result<SocketAddr> {
    self.stream.local_addr()
  }

  fn peer_addr(&self) -> io::Result<SocketAddr> {
    self.stream.peer_addr()
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.stream.set_ttl(ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    self.stream.ttl()
  }

  fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
    self.stream.set_nodelay(nodelay)
  }

  fn nodelay(&self) -> io::Result<bool> {
    self.stream.nodelay()
  }

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::SeqCst);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::SeqCst)
  }

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::SeqCst);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::SeqCst)
  }
}

pub struct TokioUdpSocket {
  socket: UdpSocket,
  write_timeout: Atomic<Option<Duration>>,
  read_timeout: Atomic<Option<Duration>>,
}

#[async_trait::async_trait]
impl crate::net::UdpSocket for TokioUdpSocket {
  type Runtime = TokioRuntime;

  async fn bind<A: ToSocketAddrs<Self::Runtime>>(addr: A) -> io::Result<Self>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs(&TokioRuntime).await?;

    let res = if addrs.size_hint().0 <= 1 {
      if let Some(addr) = addrs.next() {
        UdpSocket::bind(addr).await
      } else {
        return Err(io::Error::new(
          io::ErrorKind::InvalidInput,
          "invalid socket address",
        ));
      }
    } else {
      UdpSocket::bind(&addrs.collect::<Vec<_>>().as_slice()).await
    };
    res.map(|socket| Self {
      socket,
      write_timeout: Atomic::new(None),
      read_timeout: Atomic::new(None),
    })
  }

  async fn bind_timeout<A: ToSocketAddrs<Self::Runtime>>(
    addr: A,
    timeout: Duration,
  ) -> io::Result<Self>
  where
    Self: Sized,
  {
    TokioRuntime
      .timeout(timeout, Self::bind(addr))
      .await
      .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
      .and_then(|res| res)
  }

  async fn connect<A: ToSocketAddrs<Self::Runtime>>(&self, addr: A) -> io::Result<()>
  where
    Self: Sized,
  {
    let addrs = addr.to_socket_addrs(&TokioRuntime).await?;

    if addrs.size_hint().0 <= 1 {
      if let Some(addr) = addrs.next() {
        self.socket.connect(addr).await
      } else {
        return Err(io::Error::new(
          io::ErrorKind::InvalidInput,
          "invalid socket address",
        ));
      }
    } else {
      self
        .socket
        .connect(&addrs.collect::<Vec<_>>().as_slice())
        .await
    }
  }

  async fn connect_timeout<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    addr: A,
    timeout: Duration,
  ) -> io::Result<()>
  where
    Self: Sized,
  {
    TokioRuntime
      .timeout(timeout, self.connect(addr))
      .await
      .map_err(|e| io::Error::new(io::ErrorKind::TimedOut, e))
      .and_then(|res| res)
  }

  async fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
    if let Some(timeout) = self.read_timeout.load(Ordering::Relaxed) {
      if !timeout.is_zero() {
        return match TokioRuntime.timeout(timeout, self.socket.recv(buf)).await {
          Ok(timeout) => timeout,
          Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
        };
      }
    }
    self.socket.recv(buf).await
  }

  async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
    if let Some(timeout) = self.read_timeout.load(Ordering::Relaxed) {
      if !timeout.is_zero() {
        return match TokioRuntime
          .timeout(timeout, self.socket.recv_from(buf))
          .await
        {
          Ok(timeout) => timeout,
          Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
        };
      }
    }
    self.socket.recv_from(buf).await
  }

  async fn send(&self, buf: &[u8]) -> io::Result<usize> {
    if let Some(timeout) = self.write_timeout.load(Ordering::Relaxed) {
      if !timeout.is_zero() {
        return match TokioRuntime.timeout(timeout, self.socket.send(buf)).await {
          Ok(timeout) => timeout,
          Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
        };
      }
    }
    self.socket.send(buf).await
  }

  async fn send_to<A: ToSocketAddrs<Self::Runtime>>(
    &self,
    buf: &[u8],
    target: A,
  ) -> io::Result<usize> {
    let addrs = target.to_socket_addrs(&TokioRuntime).await?;
    if addrs.size_hint().0 <= 1 {
      if let Some(addr) = addrs.next() {
        if let Some(timeout) = self.write_timeout.load(Ordering::Relaxed) {
          if !timeout.is_zero() {
            return match TokioRuntime
              .timeout(timeout, self.socket.send_to(buf, addr))
              .await
            {
              Ok(timeout) => timeout,
              Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
            };
          }
        }
        self.socket.send_to(buf, addr).await
      } else {
        return Err(io::Error::new(
          io::ErrorKind::InvalidInput,
          "invalid socket address",
        ));
      }
    } else {
      let addrs = addrs.collect::<Vec<_>>();
      if let Some(timeout) = self.write_timeout.load(Ordering::Relaxed) {
        if !timeout.is_zero() {
          return match TokioRuntime
            .timeout(timeout, self.socket.send_to(buf, addrs.as_slice()))
            .await
          {
            Ok(timeout) => timeout,
            Err(e) => Err(io::Error::new(io::ErrorKind::TimedOut, e)),
          };
        }
      }
      self.socket.send_to(buf, addrs.as_slice()).await
    }
  }

  fn set_ttl(&self, ttl: u32) -> io::Result<()> {
    self.socket.set_ttl(ttl)
  }

  fn ttl(&self) -> io::Result<u32> {
    self.socket.ttl()
  }

  fn set_broadcast(&self, broadcast: bool) -> io::Result<()> {
    self.socket.set_broadcast(broadcast)
  }

  fn broadcast(&self) -> io::Result<bool> {
    self.socket.broadcast()
  }

  fn set_write_timeout(&self, timeout: Option<Duration>) {
    self.write_timeout.store(timeout, Ordering::SeqCst);
  }

  fn write_timeout(&self) -> Option<Duration> {
    self.write_timeout.load(Ordering::SeqCst)
  }

  fn set_read_timeout(&self, timeout: Option<Duration>) {
    self.read_timeout.store(timeout, Ordering::SeqCst);
  }

  fn read_timeout(&self) -> Option<Duration> {
    self.read_timeout.load(Ordering::SeqCst)
  }

  #[cfg(feature = "unsafe-net")]
  fn set_read_buffer(&self, size: usize) -> io::Result<()> {
    todo!()
  }

  #[cfg(feature = "unsafe-net")]
  fn set_write_buffer(&self, size: usize) -> io::Result<()> {
    todo!()
  }
}
