use agnostic_lite::RuntimeLite;

use crate::{Net, UdpSocket};

use super::{next_test_ip4, next_test_ip6};

use async_channel::unbounded;

use std::{
  future::Future,
  io::ErrorKind,
  net::SocketAddr,
  thread,
  time::{Duration, Instant},
};

async fn each_ip<F>(f: &mut dyn FnMut(SocketAddr, SocketAddr) -> F)
where
  F: Future,
{
  f(next_test_ip4(), next_test_ip4()).await;
  f(next_test_ip6(), next_test_ip6()).await;
}

macro_rules! t {
  ($e:expr) => {
    match $e {
      Ok(t) => t,
      Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
    }
  };
}

async fn bind_error<N: Net>() {
  match <N::UdpSocket as UdpSocket>::bind("1.1.1.1:9999").await {
    Ok(..) => panic!(),
    Err(e) => assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
  }
}

async fn socket_smoke_test_ip4<N: Net>() {
  each_ip(&mut |server_ip, client_ip| async move {
    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let client = t!(<N::UdpSocket as UdpSocket>::bind(&client_ip).await);
      rx1.recv().await.unwrap();
      t!(client.send_to(&[99], &server_ip).await);
      tx2.send(()).await.unwrap();
    });

    let server: N::UdpSocket = t!(<N::UdpSocket as UdpSocket>::bind(&server_ip).await);
    tx1.send(()).await.unwrap();
    let mut buf = [0];
    let (nread, src) = t!(server.recv_from(&mut buf).await);
    assert_eq!(nread, 1);
    assert_eq!(buf[0], 99);
    assert_eq!(src, client_ip);
    rx2.recv().await.unwrap();
  })
  .await
}

async fn socket_name<N: Net>() {
  each_ip(&mut |addr, _| async move {
    let server = t!(<N::UdpSocket as UdpSocket>::bind(&addr).await);
    assert_eq!(addr, t!(server.local_addr()));
  })
  .await
}

async fn socket_peer<N: Net>() {
  each_ip(&mut |addr1, addr2| async move {
    let server = t!(<N::UdpSocket as UdpSocket>::bind(&addr1).await);
    assert_eq!(
      server.peer_addr().unwrap_err().kind(),
      ErrorKind::NotConnected
    );
    t!(server.connect(&addr2).await);
    assert_eq!(addr2, t!(server.peer_addr()));
  })
  .await
}

async fn udp_clone_smoke<N: Net>() {
  each_ip(&mut |addr1, addr2| async move {
    let sock1 = t!(<N::UdpSocket as UdpSocket>::bind(&addr1).await);
    let sock2 = t!(<N::UdpSocket as UdpSocket>::bind(&addr2).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut buf = [0, 0];
      assert_eq!(sock2.recv_from(&mut buf).await.unwrap(), (1, addr1));
      assert_eq!(buf[0], 1);
      t!(sock2.send_to(&[2], &addr1).await);
    });

    let sock3 = t!(sock1.try_clone());

    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      rx1.recv().await.unwrap();
      t!(sock3.send_to(&[1], &addr2).await);
      tx2.send(()).await.unwrap();
    });
    tx1.send(()).await.unwrap();
    let mut buf = [0, 0];
    assert_eq!(sock1.recv_from(&mut buf).await.unwrap(), (1, addr2));
    rx2.recv().await.unwrap();
  })
  .await
}

async fn udp_clone_two_read<N: Net>() {
  each_ip(&mut |addr1, addr2| async move {
    let sock1 = t!(<N::UdpSocket as UdpSocket>::bind(&addr1).await);
    let sock2 = t!(<N::UdpSocket as UdpSocket>::bind(&addr2).await);
    let (tx1, rx) = unbounded();
    let tx2 = tx1.clone();

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      t!(sock2.send_to(&[1], &addr1).await);
      rx.recv().await.unwrap();
      t!(sock2.send_to(&[2], &addr1).await);
      rx.recv().await.unwrap();
    });

    let sock3 = t!(sock1.try_clone());

    let (done, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut buf = [0, 0];
      t!(sock3.recv_from(&mut buf).await);
      tx2.send(()).await.unwrap();
      done.send(()).await.unwrap();
    });
    let mut buf = [0, 0];
    t!(sock1.recv_from(&mut buf).await);
    tx1.send(()).await.unwrap();

    rx.recv().await.unwrap();
  })
  .await
}

async fn udp_clone_two_write<N: Net>() {
  each_ip(&mut |addr1, addr2| async move {
    let sock1 = t!(<N::UdpSocket as UdpSocket>::bind(&addr1).await);
    let sock2 = t!(<N::UdpSocket as UdpSocket>::bind(&addr2).await);

    let (tx1, rx1) = unbounded();
    let (serv_tx, serv_rx) = unbounded();

    <N::Runtime as RuntimeLite>::spawn_detach(async move {
      let mut buf = [0, 1];
      rx1.recv().await.unwrap();
      t!(sock2.recv_from(&mut buf).await);
      serv_tx.send(()).await.unwrap();
    });

    let sock3 = t!(sock1.try_clone());

    let (done, done_rx) = unbounded();
    let tx2 = tx1.clone();
    <N::Runtime as RuntimeLite>::spawn_detach(async move {
      if sock3.send_to(&[1], &addr2).await.is_ok() {
        let _ = tx2.send(()).await;
      }
      done.send(()).await.unwrap();
    });

    match sock1.send_to(&[2], &addr2).await {
      Ok(_) => tx1.send(()).await.unwrap(),
      Err(e) => panic!("{e}"),
    };
    drop(tx1);

    if let Err(e) = done_rx.recv().await {
      panic!("{e}");
    }
    serv_rx.recv().await.unwrap();
  })
  .await
}

async fn connect_send_recv<N: Net>() {
  let addr = next_test_ip4();

  let socket = t!(<N::UdpSocket as UdpSocket>::bind(&addr).await);
  t!(socket.connect(addr).await);

  t!(socket.send(b"hello world").await);

  let mut buf = [0; 11];
  t!(socket.recv(&mut buf).await);
  assert_eq!(b"hello world", &buf[..]);
}

async fn connect_send_peek_recv<N: Net>() {
  each_ip(&mut |addr, _| async move {
    let socket = t!(<N::UdpSocket as UdpSocket>::bind(&addr).await);
    t!(socket.connect(addr).await);

    t!(socket.send(b"hello world").await);

    for _ in 1..3 {
      let mut buf = [0; 11];
      let size = t!(socket.peek(&mut buf).await);
      assert_eq!(b"hello world", &buf[..]);
      assert_eq!(size, 11);
    }

    let mut buf = [0; 11];
    let size = t!(socket.recv(&mut buf).await);
    assert_eq!(b"hello world", &buf[..]);
    assert_eq!(size, 11);
  })
  .await
}

async fn peek<N: Net>() {
  each_ip(&mut |addr, _| async move {
    let socket = t!(<N::UdpSocket as UdpSocket>::bind(&addr).await);
    t!(socket.send_to(b"hello world", &addr).await);

    for _ in 1..3 {
      let mut buf = [0; 11];
      let size = t!(socket.peek(&mut buf).await);
      assert_eq!(b"hello world", &buf[..]);
      assert_eq!(size, 11);
    }

    let mut buf = [0; 11];
    let size = t!(socket.recv(&mut buf).await);
    assert_eq!(b"hello world", &buf[..]);
    assert_eq!(size, 11);
  })
  .await
}

async fn peek_from<N: Net>() {
  each_ip(&mut |addr, _| async move {
    // TODO(al8n): remove this feature gate when async-net fixes https://github.com/smol-rs/async-net/issues/33
    #[cfg(target_os = "macos")]
    let should_run = {
      let is_smol = <N::Runtime as RuntimeLite>::name() == "smol";
      !is_smol
    };

    #[cfg(not(target_os = "macos"))]
    let should_run = true;

    if should_run {
      let socket = t!(<N::UdpSocket as UdpSocket>::bind(&addr).await);
      t!(socket.send_to(b"hello world", &addr).await);

      for _ in 1..3 {
        let mut buf = [0; 11];
        let (size, _) = t!(socket.peek_from(&mut buf).await);
        assert_eq!(b"hello world", &buf[..]);
        assert_eq!(size, 11);
      }

      let mut buf = [0; 11];
      let (size, _) = t!(socket.recv_from(&mut buf).await);
      assert_eq!(b"hello world", &buf[..]);
      assert_eq!(size, 11);
    }
  })
  .await
}

async fn ttl<N: Net>() {
  let ttl = 100;

  let addr = next_test_ip4();

  let stream = t!(<N::UdpSocket as UdpSocket>::bind(&addr).await);

  t!(stream.set_ttl(ttl));
  assert_eq!(ttl, t!(stream.ttl()));
}

macro_rules! test_suites {
  ($runtime:ident) => {
    paste::paste! {
      #[test]
      fn [<test_ $runtime _bind_error>]() {
        run(bind_error::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _socket_smoke_test_ip4>]() {
        run(socket_smoke_test_ip4::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _socket_name>]() {
        run(socket_name::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _socket_peer>]() {
        run(socket_peer::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _udp_clone_smoke>]() {
        run(udp_clone_smoke::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _udp_clone_two_read>]() {
        run(udp_clone_two_read::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _udp_clone_two_write>]() {
        run(udp_clone_two_write::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _connect_send_recv>]() {
        run(connect_send_recv::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _connect_send_peek_recv>]() {
        run(connect_send_peek_recv::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _peek>]() {
        run(peek::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _peek_from>]() {
        run(peek_from::<[< $runtime:camel Net >]>());
      }

      #[test]
      fn [<test_ $runtime _ttl>]() {
        run(ttl::<[< $runtime:camel Net >]>());
      }
    }
  };
}

#[cfg(feature = "async-std")]
mod async_std;

#[cfg(feature = "smol")]
mod smol;

#[cfg(feature = "tokio")]
mod tokio;
