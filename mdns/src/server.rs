use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::io;

use agnostic::{
  net::{Net, UdpSocket},
  AsyncSpawner, Runtime,
};
use async_channel::{Receiver, Sender};
use atomic_refcell::AtomicRefCell;
use either::Either;
use futures_util::{stream::FuturesUnordered, FutureExt, StreamExt as _};
use hickory_proto::{
  error::ProtoError,
  op::{Header, Message, MessageType, OpCode, Query, ResponseCode},
  rr::Record,
};
use smallvec_wrapper::OneOrMore;
use triomphe::Arc;

use super::{Service, Zone, IPV4_MDNS, IPV6_MDNS, MDNS_PORT, MAX_PAYLOAD_SIZE};


const FORCE_UNICAST_RESPONSES: bool = false;



/// The options for [`Server`].
#[derive(Clone, Debug, Default)]
pub struct ServerOptions {
  v4_iface: Option<Ipv4Addr>,
  v6_iface: Option<u32>,
  log_empty_responses: bool,
}

impl ServerOptions {
  /// Returns a new instance of [`ServerOptions`].
  #[inline]
  pub const fn new() -> Self {
    Self {
      v4_iface: None,
      v6_iface: None,
      log_empty_responses: false,
    }
  }

  /// Returns the Ipv4 interface to bind the multicast listener to.
  #[inline]
  pub const fn v4_iface(&self) -> Option<Ipv4Addr> {
    self.v4_iface
  }

  /// Sets the IPv4 interface to bind the multicast listener to.
  /// 
  /// `None` means the server will not listen on IPv4.
  #[inline]
  pub fn with_v4_iface(mut self, iface: Option<Ipv4Addr>) -> Self {
    self.v4_iface = iface;
    self
  }

  /// Returns the Ipv6 interface to bind the multicast listener to.
  #[inline]
  pub const fn v6_iface(&self) -> Option<u32> {
    self.v6_iface
  }

  /// Sets the IPv6 interface to bind the multicast listener to.
  /// 
  /// `None` means the server will not listen on IPv6.
  #[inline]
  pub fn with_v6_iface(mut self, index: Option<u32>) -> Self {
    self.v6_iface = index;
    self
  }

  /// Sets whether the server should print an informative message
  /// when there is an mDNS query for which the server has no response.
  /// 
  /// Default is `false`.
  #[inline]
  pub fn with_log_empty_responses(mut self, log_empty_responses: bool) -> Self {
    self.log_empty_responses = log_empty_responses;
    self
  }
}


/// The builder for [`Server`].
pub struct Server<R: Runtime, Z: Zone = Service<R>> {
  zone: Arc<Z>,
  opts: ServerOptions,
  handles: AtomicRefCell<FuturesUnordered<<R::Spawner as AsyncSpawner>::JoinHandle<()>>>,
  shutdown_tx: Sender<()>,
}

impl<R: Runtime, Z: Zone> Server<R, Z> {
  /// Creates a new mDNS server.
  pub async fn new(zone: Z, opts: ServerOptions) -> io::Result<Self> {
    let (shutdown_tx, shutdown_rx) = async_channel::bounded(1);

    if opts.v4_iface.is_none() && opts.v6_iface.is_none() {
      return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "no interface specified",
      ));
    }

    let zone = Arc::new(zone);
    let handles = FuturesUnordered::new();
    if let Some(v4) = opts.v4_iface {
      let conn = <<R::Net as Net>::UdpSocket as UdpSocket>::bind((Ipv4Addr::UNSPECIFIED, MDNS_PORT)).await?;
      conn.join_multicast_v4(IPV4_MDNS, v4)?;
      let processor = Processor::<R, Z>::new(conn, zone.clone(), opts.log_empty_responses, shutdown_rx.clone());
      handles.push(R::Spawner::spawn(processor.process()));
    }

    if let Some(v6) = opts.v6_iface {
      let conn = <<R::Net as Net>::UdpSocket as UdpSocket>::bind((Ipv6Addr::UNSPECIFIED, MDNS_PORT)).await?;
      conn.join_multicast_v6(&IPV6_MDNS, v6)?;
      let processor = Processor::<R, Z>::new(conn, zone.clone(), opts.log_empty_responses, shutdown_rx.clone());
      handles.push(R::Spawner::spawn(processor.process()));
    }

    Ok(Self {
      zone,
      opts,
      handles: AtomicRefCell::new(handles),
      shutdown_tx,
    })
  }

  /// Shuts down the mDNS server.
  ///
  /// This method is concurrent safe and can be called multiple times, but only the first call
  /// will have an effect.
  pub async fn shutdown(&self) {
    if !self.shutdown_tx.close() {
      return;
    }

    let mut handles = core::mem::take(&mut *self.handles.borrow_mut());
    while handles.next().await.is_some() {}
  }
}

struct Processor<R: Runtime, Z: Zone> {
  zone: Arc<Z>,
  conn: <R::Net as Net>::UdpSocket,
  /// Indicates the server should print an informative message
  /// when there is an mDNS query for which the server has no response.
  log_empty_responses: bool,
  shutdown_rx: Receiver<()>,
}

impl<R: Runtime, Z: Zone> Processor<R, Z> {
  fn new(conn: <R::Net as Net>::UdpSocket, zone: Arc<Z>, log_empty_responses: bool, shutdown_rx: Receiver<()>) -> Self {
    Self {
      conn,
      zone,
      log_empty_responses,
      shutdown_rx,
    }
  }

  async fn process(self) {
    let mut buf = vec![0; MAX_PAYLOAD_SIZE];

    loop {
      futures_util::select! {
        _ = self.shutdown_rx.recv().fuse() => return,
        default => {
          let (len, addr) = match self.conn.recv_from(&mut buf).await {
            Ok((len, addr)) => (len, addr),
            Err(e) => {
              tracing::error!(err=%e, "mdns: failed to receive data from UDP socket");
              continue;
            }
          };

          let msg = match Message::from_vec(&buf[..len]) {
            Ok(msg) => msg,
            Err(e) => {
              tracing::error!(err=%e, "mdns: failed to unpack packet");
              continue;
            }
          };

          self.handle_query(addr, msg).await;
          buf.clear();
        }
      }
    }
  }

  async fn handle_query(&self, from: SocketAddr, query: Message) {
    if query.op_code() != OpCode::Query {
      // "In both multicast query and multicast response messages, the OPCODE MUST
      // be zero on transmission (only standard queries are currently supported
      // over multicast).  Multicast DNS messages received with an OPCODE other
      // than zero MUST be silently ignored."  Note: OpcodeQuery == 0
      tracing::error!("mdns: received query with non-zero OpCode");
      return;
    }

    if query.response_code() != ResponseCode::NoError {
      // "In both multicast query and multicast response messages, the Response
      // Code MUST be zero on transmission.  Multicast DNS messages received with
      // non-zero Response Codes MUST be silently ignored."
      tracing::error!("mdns: received query with non-zero ResponseCode");
      return;
    }

    // TODO(reddaly): Handle "TC (Truncated) Bit":
    //    In query messages, if the TC bit is set, it means that additional
    //    Known-Answer records may be following shortly.  A responder SHOULD
    //    record this fact, and wait for those additional Known-Answer records,
    //    before deciding whether to respond.  If the TC bit is clear, it means
    //    that the querying host has no additional Known Answers.
    if query.header().truncated() {
      tracing::error!(query=%query, "mdns: support for DNS requests with high truncated bit not implemented");
      return;
    }

    let mut multicast_answers = OneOrMore::new();
    let mut unicast_answers = OneOrMore::new();

    // Handle each query
    let queries = query.queries();
    for query in queries {
      match self.handle_question(query).await {
        Ok((mrecs, urecs)) => {
          multicast_answers.extend(mrecs);
          unicast_answers.extend(urecs)
        }
        Err(e) => {
          tracing::error!(query=%query, err=%e, "mdns: fail to handle query");
        }
      }
    }

    if self.log_empty_responses && multicast_answers.is_empty() && unicast_answers.is_empty() {
      let mut questions: Vec<_> = Vec::with_capacity(queries.len());

      for query in queries {
        questions.push(query.name().to_utf8());
      }

      tracing::info!(
        "mdns: no responses for query with questions: {}",
        questions.join(", ")
      );
    }

    // See section 18 of RFC 6762 for rules about DNS headers.
    let resp = |answers: OneOrMore<Record>, unicast: bool| -> Option<Message> {
      // 18.1: ID (Query Identifier)
      // 0 for multicast response, query.Id for unicast response
      let mut id = 0;
      if unicast {
        id = query.id();
      }

      if answers.is_empty() {
        return None;
      }

      let mut msg = Message::new();
      let mut hdr = Header::new();
      hdr
        .set_id(id)
        // 18.3: OPCODE - must be zero in response (OpcodeQuery == 0)
        .set_op_code(OpCode::Query)
        // 18.4: AA (Authoritative Answer) Bit - must be set to 1
        .set_authoritative(true)
        // 18.2: QR (Query/Response) Bit - must be set to 1 in response.
        .set_message_type(MessageType::Response);
      // The following fields must all be set to 0:
      // 18.5: TC (TRUNCATED) Bit
      // 18.6: RD (Recursion Desired) Bit
      // 18.7: RA (Recursion Available) Bit
      // 18.8: Z (Zero) Bit
      // 18.9: AD (Authentic Data) Bit
      // 18.10: CD (Checking Disabled) Bit
      // 18.11: RCODE (Response Code)
      msg.set_header(hdr).add_answers(answers);
      Some(msg)
    };

    if let Some(mresp) = resp(multicast_answers, false) {
      if let Err(e) = self.send_response(mresp, from, false).await {
        tracing::error!(err=%e, "mdns: error sending multicast response");
        return;
      }
    }

    if let Some(uresp) = resp(unicast_answers, true) {
      if let Err(e) = self.send_response(uresp, from, true).await {
        tracing::error!(err=%e, "mdns: error sending unicast response");
      }
    }
  }

  async fn handle_question(
    &self,
    query: &Query,
  ) -> Result<(OneOrMore<Record>, OneOrMore<Record>), Z::Error> {
    let records = self.zone.records(query.name(), query.query_type()).await?;

    if records.is_empty() {
      return Ok((OneOrMore::new(), OneOrMore::new()));
    }

    // Handle unicast and multicast responses.
    // TODO(reddaly): The decision about sending over unicast vs. multicast is not
    // yet fully compliant with RFC 6762.  For example, the unicast bit should be
    // ignored if the records in question are close to TTL expiration.  For now,
    // we just use the unicast bit to make the decision, as per the spec:
    //     RFC 6762, section 18.12.  Repurposing of Top Bit of qclass in Question
    //     Section
    //
    //     In the Question Section of a Multicast DNS query, the top bit of the
    //     qclass field is used to indicate that unicast responses are preferred
    //     for this particular question.  (See Section 5.4.)
    let qc: u16 = query.query_class().into();
    let res = if (qc & (1 << 15)) != 0 || FORCE_UNICAST_RESPONSES {
      (OneOrMore::new(), records)
    } else {
      (records, OneOrMore::new())
    };

    Ok(res)
  }

  async fn send_response(
    &self,
    msg: Message,
    from: SocketAddr,
    _unicast: bool,
  ) -> Result<usize, Either<ProtoError, io::Error>> {
    // TODO(reddaly): Respect the unicast argument, and allow sending responses
    // over multicast.

    let data = msg.to_vec().map_err(Either::Left)?;
    self.conn.send_to(&data, from).await.map_err(Either::Right)
  }
}
