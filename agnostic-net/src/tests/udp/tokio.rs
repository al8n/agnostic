use super::{super::tokio_run as run, *};

use crate::{Net, tokio::Net as TokioNet};
use agnostic_lite::tokio::TokioRuntime;

test_suites!(tokio);
