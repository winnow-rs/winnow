//! Deprecated, see [`combinator`]
#![deprecated(since = "0.4.2", note = "Replaced with `combinator`")]

use crate::combinator;

pub use combinator::alt;
pub use combinator::dispatch;
pub use combinator::permutation;
pub use combinator::Alt;
pub use combinator::Permutation;
