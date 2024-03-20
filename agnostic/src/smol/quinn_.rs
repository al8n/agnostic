use std::{
  future::Future,
  io,
  pin::Pin,
  task::{Context, Poll},
  time::Instant,
};

use async_io::{Async, Timer};

use futures_util::ready;
use quinn::{AsyncTimer, AsyncUdpSocket, Runtime as QuinnRuntime};
use quinn_udp as udp;

pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[repr(transparent)]
    struct TimerW {#[pin] inner: Timer }
}

impl TimerW {
  fn new(inner: Timer) -> Self {
    Self { inner }
  }
}

impl Future for TimerW {
  type Output = <Timer as Future>::Output;

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.project().inner.poll(cx)
  }
}

/// A Quinn runtime for smol
#[derive(Default, Debug)]
pub struct SmolRuntime;

impl QuinnRuntime for SmolRuntime {
  fn new_timer(&self, t: Instant) -> Pin<Box<dyn AsyncTimer>> {
    Box::pin(TimerW::new(Timer::at(t)))
  }

  fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
    smol::spawn(future).detach();
  }

  fn wrap_udp_socket(&self, sock: std::net::UdpSocket) -> io::Result<Box<dyn AsyncUdpSocket>> {
    udp::UdpSocketState::configure((&sock).into())?;
    Ok(Box::new(UdpSocket {
      io: Async::new(sock)?,
      inner: udp::UdpSocketState::new(),
    }))
  }
}

impl AsyncTimer for TimerW {
  fn reset(mut self: Pin<&mut Self>, t: Instant) {
    self.inner.set_at(t)
  }

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<()> {
    Future::poll(self, cx).map(|_| ())
  }
}

#[derive(Debug)]
struct UdpSocket {
  io: Async<std::net::UdpSocket>,
  inner: quinn_udp::UdpSocketState,
}

impl AsyncUdpSocket for UdpSocket {
  fn poll_send(
    &self,
    state: &udp::UdpState,
    cx: &mut Context,
    transmits: &[udp::Transmit],
  ) -> Poll<io::Result<usize>> {
    loop {
      ready!(self.io.poll_writable(cx))?;
      if let Ok(res) = self.inner.send((&self.io).into(), state, transmits) {
        return Poll::Ready(Ok(res));
      }
    }
  }

  fn poll_recv(
    &self,
    cx: &mut Context,
    bufs: &mut [io::IoSliceMut<'_>],
    meta: &mut [udp::RecvMeta],
  ) -> Poll<io::Result<usize>> {
    loop {
      ready!(self.io.poll_readable(cx))?;
      if let Ok(res) = self.inner.recv((&self.io).into(), bufs, meta) {
        return Poll::Ready(Ok(res));
      }
    }
  }

  fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
    self.io.as_ref().local_addr()
  }

  fn may_fragment(&self) -> bool {
    udp::may_fragment()
  }
}
