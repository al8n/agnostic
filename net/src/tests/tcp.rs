use agnostic_lite::RuntimeLite;
use async_channel::unbounded;
use futures_util::{AsyncReadExt, AsyncWriteExt, StreamExt};

use crate::{Net, TcpListener, TcpStream};

use super::{next_test_ip4, next_test_ip6};
use std::{
  fmt,
  future::Future,
  io::*,
  mem::MaybeUninit,
  net::{Shutdown, SocketAddr},
  thread,
  time::{Duration, Instant},
};

macro_rules! test_suites_in {
  ($runtime:ident {
    $(
      $(#[$meta:meta])*
      $test_name:ident
    ), +$(,)?
  }) => {
    paste::paste! {
      $(
        $(#[$meta])*
        #[test]
        fn [<test_ $runtime _ $test_name>]() {
          run($test_name::<[<$runtime:camel Net>]>());
        }
      )*
    }
  };
}

macro_rules! test_suites {
  ($runtime:ident) => {
    test_suites_in!($runtime {
      bind_error,
      connect_error,
      connect_timeout_error,
      listen_localhost,
      connect_loopback,
      smoke,
      #[cfg(feature = "tokio")]
      tokio_smoke,
      read_eof,
      write_close,
      multiple_connect_serial,
      multiple_connect_interleaved_greedy_schedule,
      multiple_connect_interleaved_lazy_schedule,
      socket_and_peer_name,
      partial_read,
      read_vectored,
      write_vectored,
      double_bind,
      tcp_clone_smoke,
      tcp_clone_two_read,
      tcp_clone_two_write,
      shutdown_smoke,
      #[cfg(not(windows))]
      close_read_wakes_up,
      close_readwrite_smoke,
      clone_while_reading,
      clone_accept_smoke,
      clone_accept_concurrent,
      linger,
      nodelay,
      ttl,
      peek,
      connect_timeout_valid,
      reunite,
      into_split,
      #[cfg(feature = "tokio")]
      tokio_into_split,
    });
  };
}

#[cfg(feature = "async-std")]
mod async_std;

#[cfg(feature = "smol")]
mod smol;

#[cfg(feature = "tokio")]
mod tokio;

macro_rules! t {
  ($e:expr) => {
    match $e {
      Ok(t) => t,
      Err(e) => panic!("received error for `{}`: {}", stringify!($e), e),
    }
  };
}

async fn each_ip<F>(f: &mut dyn FnMut(SocketAddr) -> F)
where
  F: Future,
{
  f(next_test_ip4()).await;
  f(next_test_ip6()).await;
}

async fn bind_error<N: Net>() {
  match <N::TcpListener as TcpListener>::bind("1.1.1.1:9999").await {
    Ok(..) => panic!(),
    Err(e) => assert_eq!(e.kind(), ErrorKind::AddrNotAvailable),
  }
}

async fn connect_error<N: Net>() {
  match <N::TcpStream as TcpStream>::connect("0.0.0.0:1").await {
    Ok(..) => panic!(),
    Err(e) => assert!(
      e.kind() == ErrorKind::ConnectionRefused
        || e.kind() == ErrorKind::InvalidInput
        || e.kind() == ErrorKind::AddrInUse
        || e.kind() == ErrorKind::AddrNotAvailable,
      "bad error: {} {:?}",
      e,
      e.kind()
    ),
  }
}

async fn connect_timeout_error<N: Net>() {
  let socket_addr = next_test_ip4();
  let result = <N::TcpStream as TcpStream>::connect_timeout(&socket_addr, Duration::MAX).await;
  assert!(!matches!(result, Err(e) if e.kind() == ErrorKind::TimedOut));

  let _listener = <N::TcpListener as TcpListener>::bind(&socket_addr)
    .await
    .unwrap();
  assert!(
    <N::TcpStream as TcpStream>::connect_timeout(&socket_addr, Duration::MAX)
      .await
      .is_ok()
  );
}

async fn listen_localhost<N: Net>() {
  let socket_addr = next_test_ip4();
  let listener = t!(<N::TcpListener as TcpListener>::bind(&socket_addr).await);

  let _t = <N::Runtime as RuntimeLite>::spawn(async move {
    let mut stream =
      t!(<N::TcpStream as TcpStream>::connect(&("localhost", socket_addr.port())).await);
    t!(stream.write(&[144]).await);
  });

  let mut stream = t!(listener.accept().await).0;
  let mut buf = [0];
  t!(stream.read(&mut buf).await);
  assert!(buf[0] == 144);
}

async fn connect_loopback<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let host = match addr {
        SocketAddr::V4(..) => "127.0.0.1",
        SocketAddr::V6(..) => "::1",
      };
      let mut stream = t!(<N::TcpStream as TcpStream>::connect(&(host, addr.port())).await);
      t!(stream.write(&[66]).await);
    });

    let mut stream = t!(acceptor.accept().await).0;
    let mut buf = [0];
    t!(stream.read(&mut buf).await);
    assert!(buf[0] == 66);
  })
  .await
}

async fn smoke<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let (tx, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      t!(stream.write(&[99]).await);
      t!(stream.flush().await);
      tx.send(t!(stream.local_addr())).await.unwrap();
      t!(stream.close().await);
    });

    let (mut stream, addr) = t!(acceptor.accept().await);
    let mut buf = [0];
    t!(stream.read(&mut buf).await);
    assert!(buf[0] == 99);
    assert_eq!(addr, t!(rx.recv().await));
  })
  .await
}

#[cfg(feature = "tokio")]
async fn tokio_smoke<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let (tx, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      t!(::tokio::io::AsyncWriteExt::write(&mut stream, &[99]).await);
      t!(::tokio::io::AsyncWriteExt::flush(&mut stream).await);
      tx.send(t!(stream.local_addr())).await.unwrap();
      t!(::tokio::io::AsyncWriteExt::shutdown(&mut stream).await);
    });

    let (mut stream, addr) = t!(acceptor.accept().await);
    let mut buf = [0];
    t!(::tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await);
    assert!(buf[0] == 99);
    assert_eq!(addr, t!(rx.recv().await));
  })
  .await
}

async fn read_eof<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let _stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      // Close
    });

    let mut stream = t!(acceptor.accept().await).0;
    let mut buf = [0];
    let nread = t!(stream.read(&mut buf).await);
    assert_eq!(nread, 0);
    let nread = t!(stream.read(&mut buf).await);
    assert_eq!(nread, 0);
  })
  .await
}

async fn write_close<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let (tx, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      drop(t!(<N::TcpStream as TcpStream>::connect(&addr).await));
      tx.send(()).await.unwrap();
    });

    let mut stream = t!(acceptor.accept().await).0;
    rx.recv().await.unwrap();
    let buf = [0];
    match stream.write(&buf).await {
      Ok(..) => {}
      Err(e) => {
        assert!(
          e.kind() == ErrorKind::ConnectionReset
            || e.kind() == ErrorKind::BrokenPipe
            || e.kind() == ErrorKind::ConnectionAborted,
          "unknown error: {e}"
        );
      }
    }
  })
  .await
}

async fn multiple_connect_serial<N: Net>() {
  each_ip(&mut |addr| async move {
    let max = 10;
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      for _ in 0..max {
        let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
        t!(stream.write(&[99]).await);
      }
    });

    let incoming = acceptor.incoming().take(max);
    futures_util::pin_mut!(incoming);

    for stream in incoming.next().await {
      let mut stream = t!(stream);
      let mut buf = [0];
      t!(stream.read(&mut buf).await);
      assert_eq!(buf[0], 99);
    }
  })
  .await
}

async fn multiple_connect_interleaved_greedy_schedule<N: Net>() {
  const MAX: usize = 10;
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut streams = acceptor.incoming().take(MAX).enumerate();

      for (i, stream) in streams.next().await {
        // Start another thread to handle the connection
        let _t = <N::Runtime as RuntimeLite>::spawn(async move {
          let mut stream = t!(stream);
          let mut buf = [0];
          t!(stream.read(&mut buf).await);
          assert!(buf[0] == i as u8);
        });
      }
    });

    connect::<N>(0, addr).await;
  });

  fn connect<N: Net>(i: usize, addr: SocketAddr) -> impl Future<Output = ()> + Send {
    async move {
      if i == MAX {
        return;
      }

      let t = <N::Runtime as RuntimeLite>::spawn(async move {
        let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
        // Connect again before writing
        connect::<N>(i + 1, addr).await;
        t!(stream.write(&[i as u8]).await);
      });
      t.await.expect("thread panicked");
    }
  }
}

async fn multiple_connect_interleaved_lazy_schedule<N: Net>() {
  const MAX: usize = 10;
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    <N::Runtime as RuntimeLite>::spawn_detach(async move {
      let mut streams = acceptor.incoming().take(MAX);
      for stream in streams.next().await {
        // Start another thread to handle the connection
        <N::Runtime as RuntimeLite>::spawn(async move {
          let mut stream = t!(stream);
          let mut buf = [0];
          t!(stream.read(&mut buf).await);
          assert!(buf[0] == 99);
        })
        .await;
      }
    });

    // Add delay before connect
    <N::Runtime as RuntimeLite>::sleep(Duration::from_millis(100)).await;

    connect::<N>(0, addr).await;
  })
  .await;

  fn connect<N: Net>(i: usize, addr: SocketAddr) -> impl Future<Output = ()> + Send {
    async move {
      if i == MAX {
        return;
      }

      let t = <N::Runtime as RuntimeLite>::spawn(async move {
        let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
        connect::<N>(i + 1, addr).await;
        t!(stream.write(&[99]).await);
      });
      t.await.expect("thread panicked");
    }
  }
}

async fn socket_and_peer_name<N: Net>() {
  each_ip(&mut |addr| async move {
    let listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let so_name = t!(listener.local_addr());
    assert_eq!(addr, so_name);
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      t!(listener.accept().await);
    });

    let stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    assert_eq!(addr, t!(stream.peer_addr()));
  })
  .await
}

async fn partial_read<N: Net>() {
  each_ip(&mut |addr| async move {
    let (tx, rx) = unbounded();
    let srv = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut cl = t!(srv.accept().await).0;
      #[allow(clippy::unused_io_amount)]
      cl.write(&[10]).await.unwrap();
      let mut b = [0];
      t!(cl.read(&mut b).await);
      tx.send(()).await.unwrap();
    });

    let mut c = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    let mut b = [0; 10];
    assert_eq!(c.read(&mut b).await.unwrap(), 1);
    t!(c.write(&[1]).await);
    rx.recv().await.unwrap();
  })
  .await
}

async fn read_vectored<N: Net>() {
  each_ip(&mut |addr| async move {
    let srv = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let mut s1 = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    let mut s2 = t!(srv.accept().await).0;

    let len = s1.write(&[10, 11, 12]).await.unwrap();
    assert_eq!(len, 3);

    let mut a = [];
    let mut b = [0];
    let mut c = [0; 3];
    let len = t!(
      s2.read_vectored(&mut [
        IoSliceMut::new(&mut a),
        IoSliceMut::new(&mut b),
        IoSliceMut::new(&mut c)
      ],)
        .await
    );
    assert!(len > 0);
    assert_eq!(b, [10]);
    // some implementations don't support readv, so we may only fill the first buffer
    assert!(len == 1 || c == [11, 12, 0]);
  })
  .await
}

async fn write_vectored<N: Net>() {
  each_ip(&mut |addr| async move {
    let srv = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let mut s1 = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    let mut s2 = t!(srv.accept().await).0;

    let a = [];
    let b = [10];
    let c = [11, 12];
    t!(
      s1.write_vectored(&[IoSlice::new(&a), IoSlice::new(&b), IoSlice::new(&c)])
        .await
    );

    let mut buf = [0; 4];
    let len = t!(s2.read(&mut buf).await);
    // some implementations don't support writev, so we may only write the first buffer
    if len == 1 {
      assert_eq!(buf, [10, 0, 0, 0]);
    } else {
      assert_eq!(len, 3);
      assert_eq!(buf, [10, 11, 12, 0]);
    }
  })
  .await
}

async fn double_bind<N: Net>() {
  each_ip(&mut |addr| async move {
    let listener1 = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    match <N::TcpListener as TcpListener>::bind(&addr).await {
      Ok(listener2) => panic!(
        "This system (perhaps due to options set by <N::TcpListener as TcpListener>::bind) \
                 permits double binding: {:?} and {:?}",
        listener1.local_addr().unwrap(),
        listener2.local_addr().unwrap()
      ),
      Err(e) => {
        assert!(
          e.kind() == ErrorKind::ConnectionRefused || e.kind() == ErrorKind::AddrInUse,
          "unknown error: {} {:?}",
          e,
          e.kind()
        );
      }
    }
  })
  .await
}

async fn tcp_clone_smoke<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut s = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      let mut buf = [0, 0];
      assert_eq!(s.read(&mut buf).await.unwrap(), 1);
      assert_eq!(buf[0], 1);
      t!(s.write(&[2]).await);
    });

    let mut s1 = t!(acceptor.accept().await).0;
    let s2 = t!(s1.try_clone());

    let (tx1, rx1) = unbounded();
    let (tx2, rx2) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut s2 = s2;
      rx1.recv().await.unwrap();
      t!(s2.write(&[1]).await);
      tx2.send(()).await.unwrap();
    });
    tx1.send(()).await.unwrap();
    let mut buf = [0, 0];
    assert_eq!(s1.read(&mut buf).await.unwrap(), 1);
    rx2.recv().await.unwrap();
  })
  .await
}

async fn tcp_clone_two_read<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let (tx1, rx) = unbounded();
    let tx2 = tx1.clone();

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut s = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      t!(s.write(&[1]).await);
      rx.recv().await.unwrap();
      t!(s.write(&[2]).await);
      rx.recv().await.unwrap();
    });

    let mut s1 = t!(acceptor.accept().await).0;
    let s2 = t!(s1.try_clone());

    let (done, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut s2 = s2;
      let mut buf = [0, 0];
      t!(s2.read(&mut buf).await);
      tx2.send(()).await.unwrap();
      done.send(()).await.unwrap();
    });
    let mut buf = [0, 0];
    t!(s1.read(&mut buf).await);
    tx1.send(()).await.unwrap();

    rx.recv().await.unwrap();
  })
  .await
}

async fn tcp_clone_two_write<N: Net>() {
  each_ip(&mut |addr| async move {
    let acceptor = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut s = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      let mut buf = [0, 1];
      t!(s.read(&mut buf).await);
      t!(s.read(&mut buf).await);
    });

    let mut s1 = t!(acceptor.accept().await).0;
    let s2 = t!(s1.try_clone());

    let (done, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut s2 = s2;
      t!(s2.write(&[1]).await);
      done.send(()).await.unwrap();
    });
    t!(s1.write(&[2]).await);

    rx.recv().await.unwrap();
  })
  .await
}

async fn shutdown_smoke<N: Net>() {
  each_ip(&mut |addr| async move {
    let a = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut c = t!(a.accept().await).0;
      let mut b = [0];
      assert_eq!(c.read(&mut b).await.unwrap(), 0);
      t!(c.write(&[1]).await);
    });

    let mut s = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    t!(s.shutdown(Shutdown::Write));
    assert!(s.write(&[1]).await.is_err());
    let mut b = [0, 0];
    assert_eq!(t!(s.read(&mut b).await), 1);
    assert_eq!(b[0], 1);
  })
  .await
}

async fn close_readwrite_smoke<N: Net>() {
  each_ip(&mut |addr| async move {
    let a = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let (tx, rx) = unbounded::<()>();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let _s = t!(a.accept().await);
      let _ = rx.recv().await;
    });

    let mut b = [0];
    let mut s = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    let mut s2 = t!(s.try_clone());

    // closing should prevent reads/writes
    t!(s.shutdown(Shutdown::Write));
    assert!(s.write(&[0]).await.is_err());
    t!(s.shutdown(Shutdown::Read));
    assert_eq!(s.read(&mut b).await.unwrap(), 0);

    // closing should affect previous handles
    assert!(s2.write(&[0]).await.is_err());
    assert_eq!(s2.read(&mut b).await.unwrap(), 0);

    // closing should affect new handles
    let mut s3 = t!(s.try_clone());
    assert!(s3.write(&[0]).await.is_err());
    assert_eq!(s3.read(&mut b).await.unwrap(), 0);

    // make sure these don't die
    let _ = s2.shutdown(Shutdown::Read);
    let _ = s2.shutdown(Shutdown::Write);
    let _ = s3.shutdown(Shutdown::Read);
    let _ = s3.shutdown(Shutdown::Write);
    drop(tx);
  })
  .await
}

#[cfg(not(windows))]
async fn close_read_wakes_up<N: Net>() {
  each_ip(&mut |addr| async move {
    let listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let (stream, _) = t!(listener.accept().await);
      stream
    });

    let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    let stream2 = t!(stream.try_clone());

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let stream2 = stream2;

      // to make it more likely that `read` happens before `shutdown`
      thread::sleep(Duration::from_millis(1000));

      // this should wake up the reader up
      t!(stream2.shutdown(Shutdown::Read));
    });

    // this `read` should get interrupted by `shutdown`
    assert_eq!(t!(stream.read(&mut [0]).await), 0);
  })
  .await
}

async fn clone_while_reading<N: Net>() {
  each_ip(&mut |addr| async move {
    let accept = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    // Enqueue a thread to write to a socket
    let (tx, rx) = unbounded();
    let (txdone, rxdone) = unbounded();
    let txdone2 = txdone.clone();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut tcp = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
      rx.recv().await.unwrap();
      t!(tcp.write(&[0]).await);
      txdone2.send(()).await.unwrap();
    });

    // Spawn off a reading clone
    let tcp = t!(accept.accept().await).0;
    let tcp2 = t!(tcp.try_clone());
    let txdone3 = txdone.clone();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut tcp2 = tcp2;
      t!(tcp2.read(&mut [0]).await);
      txdone3.send(()).await.unwrap();
    });

    // Try to ensure that the reading clone is indeed reading
    for _ in 0..50 {
      <N::Runtime as RuntimeLite>::yield_now().await;
    }

    // clone the handle again while it's reading, then let it finish the
    // read.
    let _ = t!(tcp.try_clone());
    tx.send(()).await.unwrap();
    rxdone.recv().await.unwrap();
    rxdone.recv().await.unwrap();
  })
  .await
}

async fn clone_accept_smoke<N: Net>() {
  each_ip(&mut |addr| async move {
    let a = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let a2 = t!(a.try_clone());

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let _ = <N::TcpStream as TcpStream>::connect(&addr).await;
    });
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let _ = <N::TcpStream as TcpStream>::connect(&addr).await;
    });

    t!(a.accept().await);
    t!(a2.accept().await);
  })
  .await
}

async fn clone_accept_concurrent<N: Net>() {
  each_ip(&mut |addr| async move {
    let a = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let a2 = t!(a.try_clone());

    let (tx, rx) = unbounded();
    let tx2 = tx.clone();

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      tx.send(t!(a.accept().await)).await.unwrap();
    });
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      tx2.send(t!(a2.accept().await)).await.unwrap();
    });

    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let _ = <N::TcpStream as TcpStream>::connect(&addr).await;
    });
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let _ = <N::TcpStream as TcpStream>::connect(&addr).await;
    });

    rx.recv().await.unwrap();
    rx.recv().await.unwrap();
  })
  .await
}

async fn linger<N: Net>() {
  let addr = next_test_ip4();
  let _listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

  let stream = t!(<N::TcpStream as TcpStream>::connect(&("localhost", addr.port())).await);

  assert_eq!(None, t!(stream.linger()));
  t!(stream.set_linger(Some(Duration::from_secs(1))));
  assert_eq!(Some(Duration::from_secs(1)), t!(stream.linger()));
  t!(stream.set_linger(None));
  assert_eq!(None, t!(stream.linger()));
}

async fn nodelay<N: Net>() {
  let addr = next_test_ip4();
  let _listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

  let stream = t!(<N::TcpStream as TcpStream>::connect(&("localhost", addr.port())).await);

  assert_eq!(false, t!(stream.nodelay()));
  t!(stream.set_nodelay(true));
  assert_eq!(true, t!(stream.nodelay()));
  t!(stream.set_nodelay(false));
  assert_eq!(false, t!(stream.nodelay()));
}

async fn ttl<N: Net>() {
  let ttl = 100;

  let addr = next_test_ip4();
  let listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

  t!(listener.set_ttl(ttl));
  assert_eq!(ttl, t!(listener.ttl()));

  let stream = t!(<N::TcpStream as TcpStream>::connect(&("localhost", addr.port())).await);

  t!(stream.set_ttl(ttl));
  assert_eq!(ttl, t!(stream.ttl()));
}

async fn peek<N: Net>() {
  use crate::tcp::OwnedReadHalf;

  each_ip(&mut |addr| async move {
    let (txdone, rxdone) = unbounded();

    let srv = t!(<N::TcpListener as TcpListener>::bind(&addr).await);
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut cl = t!(srv.accept().await).0;
      #[allow(clippy::unused_io_amount)]
      cl.write(&[1, 3, 3, 7]).await.unwrap();
      t!(rxdone.recv().await);
    });

    let mut c = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    let mut b = [0; 10];
    for _ in 1..3 {
      let len = c.peek(&mut b).await.unwrap();
      assert_eq!(len, 4);
      let (mut r, w) = c.into_split();
      let len = r.peek(&mut b).await.unwrap();
      assert_eq!(len, 4);
      c = t!(<N::TcpStream as TcpStream>::reunite(r, w));
    }
    let len = c.read(&mut b).await.unwrap();
    assert_eq!(len, 4);

    t!(txdone.send(()).await);
  })
  .await
}

async fn connect_timeout_valid<N: Net>() {
  let listener = <N::TcpListener as TcpListener>::bind("127.0.0.1:0")
    .await
    .unwrap();
  let addr = listener.local_addr().unwrap();
  <N::TcpStream as TcpStream>::connect_timeout(&addr, Duration::from_secs(2))
    .await
    .unwrap();
}

async fn reunite<N: Net>() {
  let listener = <N::TcpListener as TcpListener>::bind("127.0.0.1:0")
    .await
    .unwrap();
  let addr = listener.local_addr().unwrap();
  let stream = <N::TcpStream as TcpStream>::connect(&addr).await.unwrap();
  let (r, w) = stream.into_split();
  let stream1 = t!(<N::TcpStream as TcpStream>::reunite(r, w));

  let stream2 = <N::TcpStream as TcpStream>::connect(&addr).await.unwrap();
  let (r, _w) = stream1.into_split();
  let (_r, w) = stream2.into_split();

  assert!(<N::TcpStream as TcpStream>::reunite(r, w).is_err());
}

async fn into_split<N: Net>() {
  each_ip(&mut |addr| async move {
    let listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let (tx, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let (mut stream, _) = t!(listener.accept().await);
      let mut buf = [0];
      t!(stream.read(&mut buf).await);
      tx.send(()).await.unwrap();
    });

    let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    t!(stream.write(&[0]).await);
    stream.flush().await.unwrap();
    rx.recv().await.unwrap();

    let (a, b) = stream.into_split();
    let a = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut a = a;
      let mut buf = [0];
      t!(a.read(&mut buf).await);
    });
    let b = <N::Runtime as RuntimeLite>::spawn(async move {
      let mut b = b;
      t!(b.write(&[1]).await);
      b.flush().await.unwrap();
      let _ = b.close().await;
    });

    futures_util::future::join_all([a, b]).await;
  })
  .await
}

#[cfg(feature = "tokio")]
async fn tokio_into_split<N: Net>() {
  use crate::tcp::{OwnedReadHalf, OwnedWriteHalf};

  each_ip(&mut |addr| async move {
    let listener = t!(<N::TcpListener as TcpListener>::bind(&addr).await);

    let (tx, rx) = unbounded();
    let _t = <N::Runtime as RuntimeLite>::spawn(async move {
      let (mut stream, _) = t!(listener.accept().await);
      let mut buf = [0];
      t!(stream.read(&mut buf).await);
      tx.send(()).await.unwrap();
    });

    let mut stream = t!(<N::TcpStream as TcpStream>::connect(&addr).await);
    t!(stream.write(&[0]).await);
    stream.flush().await.unwrap();
    rx.recv().await.unwrap();

    let (a, b) = stream.into_split();
    let a = <N::Runtime as RuntimeLite>::spawn(async move {
      println!(
        "local_addr {} peer_addr {}",
        a.local_addr().unwrap(),
        a.peer_addr().unwrap()
      );
      let mut a = a;
      let mut buf = [0];
      t!(::tokio::io::AsyncReadExt::read(&mut a, &mut buf).await);
    });
    let b = <N::Runtime as RuntimeLite>::spawn(async move {
      println!(
        "local_addr {} peer_addr {}",
        b.local_addr().unwrap(),
        b.peer_addr().unwrap()
      );
      let mut b = b;
      t!(::tokio::io::AsyncWriteExt::write(&mut b, &[1]).await);
      ::tokio::io::AsyncWriteExt::flush(&mut b).await.unwrap();
      let _ = ::tokio::io::AsyncWriteExt::shutdown(&mut b).await;
      b.forget();
    });

    futures_util::future::join_all([a, b]).await;
  })
  .await
}
