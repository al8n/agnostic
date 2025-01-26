use agnostic_lite::smol::SmolRuntime;

mod udp;
mod tcp_stream;
mod tcp_listener;

pub use udp::*;
pub use tcp_stream::*;
pub use tcp_listener::*;

/// The [`Net`](super::Net) implementation for [`smol`] runtime
/// 
/// [`smol`]: https://docs.rs/smol
#[derive(Debug, Default, Clone, Copy)]
pub struct Net;

impl super::Net for Net {
  type Runtime = SmolRuntime;
  type TcpListener = TcpListener;
  type TcpStream = TcpStream;
  type UdpSocket = UdpSocket;
}



