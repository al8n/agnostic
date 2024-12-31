use core::net::{Ipv4Addr, Ipv6Addr};

use super::*;

fn make_service() -> Service {
  ServiceBuilder::new(
    Name::from_str("hostname").unwrap(),
    "_http._tcp".parse().unwrap(),
  )
  .with_domain("local.".parse().unwrap())
  .with_hostname("testhost.".parse().unwrap())
  .with_port(80)
  .with_ip("192.168.0.42".parse().unwrap())
  .with_ip("2620:0:1000:1900:b0c2:d0b2:c411:18bc".parse().unwrap())
  .with_txt_record("Local web server".into())
  .finalize()
  .unwrap()
}

#[tokio::test]
async fn bad_addr() {
  let s = make_service();
  let q = Question::any(Name::from_str("randome").unwrap());

  let recs = s.records(q).await;
  assert!(recs.is_empty(), "bad: {recs:?}");
}

#[tokio::test]
async fn service_addr() {
  let s = make_service();
  let q = Question::any("_http._tcp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 5, "bad: {recs:?}");

  let ptr = recs[0].data().cloned().unwrap().into_ptr().unwrap();
  assert_eq!(ptr.0, Name::from_str("hostname._http._tcp.local.").unwrap());

  recs[1].data().cloned().unwrap().into_srv().expect("SRV");

  recs[2].data().cloned().unwrap().into_a().expect("A");

  recs[3].data().cloned().unwrap().into_aaaa().expect("AAAA");

  recs[4].data().cloned().unwrap().into_txt().expect("TXT");
}

#[tokio::test]
async fn instance_addr_any() {
  let s = make_service();
  let q = Question::any("hostname._http._tcp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 4, "bad: {recs:?}");

  recs[0].data().cloned().unwrap().into_srv().expect("SRV");

  recs[1].data().cloned().unwrap().into_a().expect("A");

  recs[2].data().cloned().unwrap().into_aaaa().expect("AAAA");

  recs[3].data().cloned().unwrap().into_txt().expect("TXT");
}

#[tokio::test]
async fn instance_addr_srv() {
  let s = make_service();
  let q = Question::srv("hostname._http._tcp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 3, "bad: {recs:?}");

  let srv = recs[0].data().cloned().unwrap().into_srv().expect("SRV");

  recs[1].data().cloned().unwrap().into_a().expect("A");

  recs[2].data().cloned().unwrap().into_aaaa().expect("AAAA");

  assert_eq!(srv.port(), s.port);
}

#[tokio::test]
async fn instance_addr_a() {
  let s = make_service();
  let q = Question::ipv4("hostname._http._tcp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 1, "bad: {recs:?}");

  let a = recs[0].data().cloned().unwrap().into_a().expect("A");

  assert_eq!(a.0, "192.168.0.42".parse::<Ipv4Addr>().unwrap());
}

#[tokio::test]
async fn instance_addr_aaaa() {
  let s = make_service();
  let q = Question::ipv6("hostname._http._tcp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 1, "bad: {recs:?}");

  let aaaa = recs[0].data().cloned().unwrap().into_aaaa().expect("AAAA");

  assert_eq!(
    aaaa.0,
    "2620:0:1000:1900:b0c2:d0b2:c411:18bc"
      .parse::<Ipv6Addr>()
      .unwrap()
  );
}

#[tokio::test]
async fn instance_addr_txt() {
  let s = make_service();
  let q = Question::txt("hostname._http._tcp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 1, "bad: {recs:?}");

  let txt = recs[0].data().cloned().unwrap().into_txt().expect("TXT");

  assert_eq!(&*txt.txt_data()[0], s.txt[0].as_bytes());
}

#[tokio::test]
async fn hostname_query() {
  let questions = [
    (
      Question::ipv4("testhost.".parse().unwrap()),
      OneOrMore::from(Record::from_rdata(
        Name::from_str("testhost.").unwrap(),
        120,
        RData::A(A("192.168.0.42".parse().unwrap())),
      )),
    ),
    (
      Question::ipv6("testhost.".parse().unwrap()),
      OneOrMore::from(Record::from_rdata(
        Name::from_str("testhost.").unwrap(),
        120,
        RData::AAAA(AAAA(
          "2620:0:1000:1900:b0c2:d0b2:c411:18bc".parse().unwrap(),
        )),
      )),
    ),
  ];

  let s = make_service();

  for (q, r) in questions.iter() {
    let recs = s.records(q.clone()).await;
    assert_eq!(recs.len(), 1, "bad: {recs:?}");

    assert_eq!(&recs, r);
  }
}

#[tokio::test]
async fn service_enum_ptr() {
  let s = make_service();
  let q = Question::ptr("_services._dns-sd._udp.local.".parse().unwrap());

  let recs = s.records(q).await;
  assert_eq!(recs.len(), 1, "bad: {recs:?}");

  let ptr = recs[0].data().cloned().unwrap().into_ptr().unwrap();
  assert_eq!(ptr.0, Name::from_str("_http._tcp.local.").unwrap());
}
