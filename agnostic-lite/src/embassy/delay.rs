use crate::time::Delay;

use super::EmbassySleep;

/// Alias for [`Delay`] using the embassy runtime.
pub type EmbassyDelay<F> = Delay<F, EmbassySleep>;
