use crate::tokio::TokioProcess;
use std::future::Future;

fn run<F>(f: F)
where
  F: Future<Output = ()>,
{
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(f);
}

test_suites!(tokio);
