use super::{super::async_std_run as run, *};

use crate::{Net, async_std::Net as AsyncStdNet};
use agnostic_lite::async_std::AsyncStdRuntime;

test_suites!(async_std);
