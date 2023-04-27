//! # Performance
//!
//! ## Runtime Performance
//!
//! See also the general Rust [Performance Book](https://nnethercote.github.io/perf-book/)
//!
//! Tips
//! - When enough cases of an [`alt`] have unique prefixes, prefer [`dispatch`]
//! - When parsing text, try to parse is as bytes (`u8`) rather than `char`s ([`BStr`] can make
//!   debugging easier)
//! - Find simplified subsets of the grammar to parse, falling back to the full grammar when it
//!   doesn't work. For example, when parsing json strings, parse them without support for escapes,
//!   falling back to escape support if it fails.
//! - Watch for large return types.  A surprising place these can show up is when chaining parsers
//!   with a tuple.
//!
//! ## Built-time Performance
//!
//! Returning complex types as `impl Trait` can negatively impact build times.  This can hit in
//! surprising cases like:
//! ```rust
//! # use winnow::prelude::*;
//! fn foo<I, O, E>() -> impl Parser<I, O, E>
//! # where
//! #    I: winnow::stream::Stream<Token=O>,
//! #    I: winnow::stream::StreamIsPartial,
//! #    E: winnow::error::ParseError<I>,
//! {
//!     // ...some chained combinators...
//! # winnow::token::any
//! }
//! ```
//!
//! Instead, wrap the combinators in a closure to simplify the type:
//! ```rust
//! # use winnow::prelude::*;
//! fn foo<I, O, E>() -> impl Parser<I, O, E>
//! # where
//! #    I: winnow::stream::Stream<Token=O>,
//! #    I: winnow::stream::StreamIsPartial,
//! #    E: winnow::error::ParseError<I>,
//! {
//!     move |input: I| {
//!         // ...some chained combinators...
//! # winnow::token::any
//!             .parse_next(input)
//!     }
//! }
//! ```

#![allow(unused_imports)]
use crate::combinator::alt;
use crate::combinator::dispatch;
use crate::stream::BStr;
