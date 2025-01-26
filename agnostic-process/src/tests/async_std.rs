use std::future::Future;

use crate::async_std::AsyncProcess as AsyncStdProcess;

fn run<F>(f: F)
where
  F: Future<Output = ()>,
{
  async_std::task::block_on(f);
}

test_suites!(async_std);
