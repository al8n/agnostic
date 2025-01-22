use std::future::Future;

use crate::smol::AsyncProcess as SmolProcess;

fn run<F>(f: F)
where
  F: Future<Output = ()>,
{
  smol::block_on(f);
}

test_suites!(smol);
