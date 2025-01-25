use super::{super::tokio_run as run, *};

use crate::{tokio::Net as TokioNet, Net};
use agnostic_lite::tokio::TokioRuntime;

test_suites!(tokio);
