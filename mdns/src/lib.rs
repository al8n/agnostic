#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc as std;

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
mod tests;

use core::{
  net::{Ipv4Addr, Ipv6Addr},
  str::FromStr,
};

pub use hickory_proto::{
  error::ProtoError,
  rr::{Name, RecordType},
};

const IPV4_MDNS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const IPV6_MDNS: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0xfb);
const MDNS_PORT: u16 = 5353;
// See RFC 6762, https://datatracker.ietf.org/doc/rfc6762/
const MAX_PAYLOAD_SIZE: usize = 9000;

pub mod client;
pub mod server;

mod zone;
pub use zone::*;

mod utils;

/// Returns the hostname of the current machine.
///
/// On wasm target, this function always returns `None`.
///
/// ## Examples
///
/// ```
/// use agnostic_mdns::hostname;
///
/// let hostname = hostname();
/// println!("hostname: {hostname:?}");
/// ```
#[allow(unreachable_code)]
pub fn hostname() -> Result<Name, ProtoError> {
  #[cfg(not(any(target_arch = "wasm32", windows)))]
  return {
    let name = rustix::system::uname();
    let name = name.nodename();
    let name = format!("{}.", name.to_string_lossy());
    Name::from_str(&name)
  };

  #[cfg(windows)]
  return {
    match ::hostname::get() {
      Ok(name) => {
        let name = name.to_string_lossy();
        Name::from_str(&name)
      }
      Err(e) => Err(e.into()),
    }
  };

  Err("cannot get hostname".into())
}
