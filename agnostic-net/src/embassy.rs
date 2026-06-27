//! Unified `embassy`-style facade for both `no_std` and `std` builds.
//!
//! In `no_std` mode this module keeps re-exporting `embassy-net` types directly.
//! In `std` mode it preserves the same public paths and method names, while routing the
//! implementation through the crate's `tokio` backend.

#[cfg(not(feature = "std"))]
pub use embassy_net::*;

#[cfg(all(feature = "std", not(feature = "tokio")))]
compile_error!("`agnostic-net` requires the `tokio` feature for `embassy` std facade support");

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use core::marker::PhantomData;

#[cfg(feature = "std")]
use crate::io::{Error, ErrorKind, Result};
#[cfg(feature = "std")]
use crate::tokio::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream};

#[cfg(feature = "std")]
pub use embassy_net::{IpAddress, IpEndpoint, Ipv4Address};

#[cfg(feature = "std")]
fn map_std_err(err: std::io::Error) -> Error {
  err
}

#[cfg(feature = "std")]
fn socket_addr(endpoint: IpEndpoint) -> Result<std::net::SocketAddr> {
  match endpoint.addr {
    IpAddress::Ipv4(addr) => {
      let octets = addr.octets();
      Ok(std::net::SocketAddr::from((
        std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]),
        endpoint.port,
      )))
    }
    #[allow(unreachable_patterns)]
    _ => Err(Error::from(ErrorKind::Unsupported)),
  }
}

/// DNS types used by the facade.
#[cfg(feature = "std")]
pub mod dns {
  pub use embassy_net::dns::DnsQueryType;
}

/// Network stack handle used by the facade.
#[cfg(feature = "std")]
#[derive(Debug, Default, Clone, Copy)]
pub struct Stack<'a> {
  marker: PhantomData<&'a ()>,
}

#[cfg(feature = "std")]
impl<'a> Stack<'a> {
  /// Creates a stack handle for the host-side facade.
  #[inline]
  pub const fn new() -> Self {
    Self {
      marker: PhantomData,
    }
  }

  /// Resolves a hostname to IP addresses.
  pub async fn dns_query(&self, host: &str, query_type: dns::DnsQueryType) -> Result<Vec<IpAddress>> {
    match query_type {
      dns::DnsQueryType::A => {}
      _ => return Err(Error::from(ErrorKind::Unsupported)),
    }

    let mut out = Vec::new();
    let addrs = ::tokio::net::lookup_host((host, 0))
      .await
      .map_err(map_std_err)?;

    for addr in addrs {
      if let std::net::IpAddr::V4(v4) = addr.ip() {
        let octets = v4.octets();
        out.push(IpAddress::Ipv4(Ipv4Address::new(
          octets[0], octets[1], octets[2], octets[3],
        )));
      }
    }

    if out.is_empty() {
      return Err(Error::from(ErrorKind::Other));
    }

    Ok(out)
  }
}

/// TCP types used by the facade.
#[cfg(feature = "std")]
pub mod tcp {
  use super::{PhantomData, Result, TokioTcpListener, TokioTcpStream, map_std_err, socket_addr};
  use crate::{TcpListener as _, TcpStream as _};
  use futures_util::io::{AsyncReadExt, AsyncWriteExt};

  /// `embassy`-style TCP socket facade backed by the crate's `tokio` TCP types.
  #[derive(Default)]
  pub struct TcpSocket<'a> {
    listener: Option<TokioTcpListener>,
    stream: Option<TokioTcpStream>,
    marker: PhantomData<&'a mut [u8]>,
  }

  impl<'a> TcpSocket<'a> {
    /// Creates a TCP socket.
    ///
    /// The buffers are ignored on the host-side backend but kept in the signature so the
    /// public API matches the `no_std` path.
    #[inline]
    pub fn new(_stack: super::Stack<'a>, _rx_buf: &'a mut [u8], _tx_buf: &'a mut [u8]) -> Self {
      Self {
        listener: None,
        stream: None,
        marker: PhantomData,
      }
    }

    /// Connects the socket to a remote endpoint.
    pub async fn connect(&mut self, remote_endpoint: super::IpEndpoint) -> Result<()> {
      let stream = TokioTcpStream::connect(socket_addr(remote_endpoint)?)
        .await
        .map_err(map_std_err)?;
      self.stream = Some(stream);
      Ok(())
    }

    /// Accepts one TCP connection on the given port.
    ///
    /// The first call binds a listener to `0.0.0.0:port`; later calls reuse that listener.
    pub async fn accept(&mut self, port: u16) -> Result<()> {
      if self.listener.is_none() {
        let listener = TokioTcpListener::bind(("0.0.0.0", port))
          .await
          .map_err(map_std_err)?;
        self.listener = Some(listener);
      }

      let (stream, _) = self
        .listener
        .as_ref()
        .expect("listener is initialized")
        .accept()
        .await
        .map_err(map_std_err)?;
      self.stream = Some(stream);
      Ok(())
    }

    /// Aborts the active connection and leaves the listener available for the next accept.
    #[inline]
    pub fn abort(&mut self) {
      self.stream = None;
    }

    /// Reads bytes from the active connection.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
      self.stream_mut()?
        .read(buf)
        .await
        .map_err(map_std_err)
    }

    /// Writes bytes to the active connection.
    pub async fn write(&mut self, buf: &[u8]) -> Result<usize> {
      self.stream_mut()?
        .write(buf)
        .await
        .map_err(map_std_err)
    }

    /// Flushes the active connection.
    pub async fn flush(&mut self) -> Result<()> {
      self.stream_mut()?
        .flush()
        .await
        .map_err(map_std_err)
    }

    fn stream_mut(&mut self) -> Result<&mut TokioTcpStream> {
      self
        .stream
        .as_mut()
        .ok_or_else(|| super::Error::from(super::ErrorKind::NotConnected))
    }
  }
}
