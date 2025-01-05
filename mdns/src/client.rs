use core::{mem, net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6}, str::FromStr, time::Duration};
use std::{collections::{hash_map::Entry, HashMap}, io};

use agnostic::{net::{Net, UdpSocket}, Runtime};
use async_channel::{Receiver, Sender};
use futures_util::FutureExt;
use hickory_proto::{op::{Message, Query}, rr::{DNSClass, Name, RData, RecordType}, serialize::binary::BinEncodable};

use crate::{IPV4_MDNS, IPV6_MDNS, MAX_PAYLOAD_SIZE, MDNS_PORT};



/// Returned after we query for a service.
#[derive(Default, Clone)]
pub struct ServiceEntry {
  name: Name,
  host: Name,
  port: u16,
  
  has_txt: bool,
  sent: bool,
}

impl ServiceEntry {
  
}

/// How a lookup is performed.
pub struct QueryParam {
  service: Name,
  domain: Option<Name>,
  timeout: Duration,
  ipv4_interface: Ipv4Addr,
  ipv6_interface: u32,
  want_unicast_response: bool, // Unicast response desired, as per 5.4 in RFC
  // Whether to disable usage of IPv4 for MDNS operations. Does not affect discovered addresses.
  disable_ipv4: bool,
  // Whether to disable usage of IPv6 for MDNS operations. Does not affect discovered addresses.
  disable_ipv6: bool,
}

/// Provides a query interface that can be used to
/// search for service providers using mDNS
struct Client<R: Runtime> {
  use_ipv4: bool,
  use_ipv6: bool,

  ipv4_unicast_conn: Option<<R::Net as Net>::UdpSocket>,
  ipv6_unicast_conn: Option<<R::Net as Net>::UdpSocket>,

  ipv4_multicast_conn: Option<<R::Net as Net>::UdpSocket>,
  ipv6_multicast_conn: Option<<R::Net as Net>::UdpSocket>,
}

impl<R: Runtime> Client<R> {
  /// Looks up a given service, in a domain, waiting at most
  /// for a timeout before finishing the query. The results are streamed
  /// to a channel. Sends will not block, so clients should make sure to
  /// either read or buffer. This method will attempt to stop the query
  /// on cancellation.
  pub async fn query_with(params: QueryParam) -> io::Result<()> {
    // create a new client
    let client = Self::new(
      !params.disable_ipv4,
      !params.disable_ipv6,
      params.ipv4_interface,
      params.ipv6_interface,
    ).await?;

    let (shutdown_tx, shutdown_rx) = async_channel::bounded(1);

    let timeout = R::sleep(params.timeout.max(Duration::from_secs(1)));
    let shutdown_rx = shutdown_rx.clone();
    R::spawn_detach(async move {
      futures_util::select! {
        _ = timeout.fuse() => {
          shutdown_tx.close();
        }
        _ = shutdown_rx.recv().fuse() => {}
      }
    });

    // Ensure defaults are set
    let domain = params.domain.unwrap_or_else(|| Name::from_str("local").unwrap());


    client.query_in(params).await
  }

  async fn query_in(self, params: QueryParam) -> io::Result<()> {
    // Start listening for response packets
    let (msg_tx, msg_rx) = async_channel::bounded::<(Message, SocketAddr)>(32);

    if self.use_ipv4 {
      
    }

    if self.use_ipv6 {
      
    }

    // Send the query
    let mut message = Message::new();
    let mut q = Query::new();
    q
      .set_name(params.service)
      .set_query_type(RecordType::PTR);

    // RFC 6762, section 18.12.  Repurposing of Top Bit of qclass in Question
	  // Section
	  //
	  // In the Question Section of a Multicast DNS query, the top bit of the qclass
	  // field is used to indicate that unicast responses are preferred for this
	  // particular question.  (See Section 5.4.)
    if params.want_unicast_response {
      q.set_query_class((1 | (1 << 15)).into());
    }
    message.add_query(q).set_recursion_desired(false);
    
    self.send_query(message).await?;

    // Map the in-progress responses
    let mut inprogress = HashMap::new();

    // Listen until we reach the timeout
    let finish = R::sleep(params.timeout);
    futures_util::pin_mut!(finish);
    let (mut msg, addr): (Message, SocketAddr) = msg_rx.recv().await.unwrap();



    let mut inp = ServiceEntry::default();

    let answers = mem::take(msg.answers_mut());
    let additionals = mem::take(msg.additionals_mut());

    let mut inp = None;
    for answer in answers.into_iter().chain(additionals.into_iter()) {
      // TODO(reddaly): Check that response corresponds to serviceAddr?
      let parts = answer.into_parts();
      if let Some(data) = parts.rdata {
        match data {
          RData::PTR(data) => {
            // Create new entry for this
            let ent = ensure_name(&mut inprogress, data.0);
            inp = Some(ent);
          },
          RData::SRV(data) => {
            // Check for a target mismatch
            if data.target().ne(&parts.name_labels) {
              // alias(&mut inprogress, parts.name_labels, data.)
            }

            // Get the port
            let ent = ensure_name(&mut inprogress, parts.name_labels);
            ent.host = parts.name_labels;
            ent.port = data.port();
          },
          RData::TXT(data) => {},
          RData::A(data) => {},
          RData::AAAA(data) => {},
          _ => {},
        }
      }
    }

    loop {
      futures_util::select! {
        resp = msg_rx.recv().fuse() => {
          if let Ok((msg, src_addr)) = resp {
            
          }
        },
        _ = (&mut finish).fuse() => {
          return Ok(());
        }
      }
    }
  }

  async fn send_query(&self, message: Message) -> io::Result<()> {
    let buf = message.to_bytes().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if let Some(conn) = &self.ipv4_multicast_conn {
      conn.send_to(&buf, (IPV4_MDNS, MDNS_PORT)).await?;
    }

    if let Some(conn) = &self.ipv6_multicast_conn {
      conn.send_to(&buf, (IPV6_MDNS, MDNS_PORT)).await?;
    }

    Ok(())
  }

  async fn new(
    mut v4: bool,
    mut v6: bool,
    ipv4_interface: Ipv4Addr,
    ipv6_interface: u32,
  ) -> io::Result<Self> {
    if !v4 && !v6 {
      return Err(io::Error::new(io::ErrorKind::InvalidInput, "must enable at least one of IPv4 or IPv6 querying"));
    }

    // Establish unicast connections
    let mut uconn4 = if v4 {
      match <<R::Net as Net>::UdpSocket as UdpSocket>::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await {
        Err(e) =>{
          tracing::error!(err=%e, "mdns: failed to bind to udp4 port");
          None
        }
        Ok(conn) => Some(conn)
      }
    } else {
      None
    };

    let mut uconn6 = if v6 {
      match <<R::Net as Net>::UdpSocket as UdpSocket>::bind((Ipv6Addr::UNSPECIFIED, 0)).await {
        Err(e) =>{
          tracing::error!(err=%e, "mdns: failed to bind to udp6 port");
          None
        }
        Ok(conn) => Some(conn)
      }
    } else {
      None
    };

    // Establish multicast connections
    let mut mconn4 = if v4 {
      match <<R::Net as Net>::UdpSocket as UdpSocket>::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await {
        Err(e) =>{
          tracing::error!(err=%e, "mdns: failed to bind to udp4 port");
          None
        }
        Ok(conn) => match conn.join_multicast_v4(IPV4_MDNS, ipv4_interface) {
          Err(e) =>{
            tracing::error!(err=%e, "mdns: failed to join udp4 multicast group");
            None
          }
          Ok(_) => Some(conn)
        }
      }
    } else {
      None
    };

    let mut mconn6 = if v6 {
      match <<R::Net as Net>::UdpSocket as UdpSocket>::bind((Ipv6Addr::UNSPECIFIED, 0)).await {
        Err(e) =>{
          tracing::error!(err=%e, "mdns: failed to bind to udp6 port");
          None
        }
        Ok(conn) => match conn.join_multicast_v6(&IPV6_MDNS, ipv6_interface) {
          Err(e) =>{
            tracing::error!(err=%e, "mdns: failed to join udp6 multicast group");
            None
          }
          Ok(_) => Some(conn)
        }
      }
    } else {
      None
    };


    // Check that unicast and multicast connections have been made for IPv4 and IPv6
	  // and disable the respective protocol if not.
    if uconn4.is_none() || mconn4.is_none() {
      tracing::info!("mdns: failed to listen to both unicast and multicast on IPv4");
      v4 = false;
      uconn4 = None;
      mconn4 = None;
    }

    if uconn6.is_none() || mconn6.is_none() {
      tracing::info!("mdns: failed to listen to both unicast and multicast on IPv6");
      v6 = false;
      uconn6 = None;
      mconn6 = None;
    }

    if !v4 && !v6 {
      return Err(io::Error::new(io::ErrorKind::InvalidInput, "at least one of IPv4 and IPv6 must be enabled for querying"));
    }

    Ok(Self {
      use_ipv4: v4,
      use_ipv6: v6,
      ipv4_unicast_conn: uconn4,
      ipv6_unicast_conn: uconn6,
      ipv4_multicast_conn: mconn4,
      ipv6_multicast_conn: mconn6,
    })
  }

  // fn shutdown(&self) {
  //   if !self.shutdown_tx.close() {
  //     return;
  //   }

  //   tracing::info!("mdns: closing client");

  //   // TODO: close connections
  // }
}

fn ensure_name(inprogress: &mut HashMap<Name, ServiceEntry>, name: Name) -> &mut ServiceEntry {
  match inprogress.entry(name) {
    Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
    Entry::Vacant(vacant_entry) => {
      vacant_entry.insert(ServiceEntry::default())
    },
  }
}