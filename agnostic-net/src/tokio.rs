use agnostic_lite::tokio::TokioRuntime;

mod udp;
mod tcp_stream;
mod tcp_listener;

pub use udp::*;
pub use tcp_stream::*;
pub use tcp_listener::*;

/// The [`Net`](super::Net) implementation for [`tokio`] runtime.
/// 
/// [`tokio`]: https://docs.rs/tokio
#[derive(Debug, Default, Clone, Copy)]
pub struct Net;

impl super::Net for Net {
  type Runtime = TokioRuntime;
  type TcpListener = TcpListener;
  type TcpStream = TcpStream;
  type UdpSocket = UdpSocket;
}

