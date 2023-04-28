//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::combinator;

pub use combinator::delimited;
pub use combinator::preceded;
pub use combinator::separated_pair;
pub use combinator::terminated;
