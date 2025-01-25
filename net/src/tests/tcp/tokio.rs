use super::{super::tokio_run as run, *};

use crate::{tokio::Net as TokioNet, Net};
use agnostic_lite::tokio::TokioRuntime;

test_suites!(tokio {
  bind_error,
  connect_error,
  connect_timeout_error,
  listen_localhost,
  connect_loopback,
  smoke,
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
  close_readwrite_smoke,

  clone_while_reading,
  clone_accept_smoke,
  clone_accept_concurrent,
  linger,
  nodelay,
  ttl,
  peek,
  connect_timeout_valid,
});
