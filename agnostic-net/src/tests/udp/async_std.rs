use super::{super::async_std_run as run, *};

use crate::{async_std::Net as AsyncStdNet, Net};
use agnostic_lite::async_std::AsyncStdRuntime;

test_suites!(async_std);
