use agnostic_lite::tokio::TokioRuntime;

mod udp;
mod tcp_stream;
mod tcp_listener;
mod tcp_socket;
#[cfg(feature = "quinn")]
mod quinn;

pub use udp::*;
pub use tcp_stream::*;
pub use tcp_listener::*;
pub use tcp_socket::*;


/// The [`Net`](super::Net) implementation for [`tokio`] runtime.
/// 
/// [`tokio`]: https://docs.rs/tokio
#[derive(Debug, Default, Clone, Copy)]
pub struct Net;

impl super::Net for Net {
  type Runtime = TokioRuntime;
  type TcpListener = TcpListener;
  type TcpStream = TcpStream;
  type TcpSocket = TcpSocket;
  type UdpSocket = UdpSocket;

  #[cfg(feature = "quinn")]
  type Quinn = ::quinn::TokioRuntime;
}

