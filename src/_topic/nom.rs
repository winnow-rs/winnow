//! # For `nom` users
//!
//! ## Migrating from `nom`
//!
//! For comparisons with `nom`, see
//! - [Why `winnow`][super::why]
//! - [parse-rosetta-rs](https://github.com/rosetta-rs/parse-rosetta-rs/)
//!
//! What approach you take depends on the size and complexity of your parser.
//! For small, simple parsers, its likely easiest to directly port from `nom`.
//! When trying to look for the equivalent of a `nom` combinator, search in the docs for the name
//! of the `nom` combinator.  It is expected that, where names diverge, a doc alias exists.
//! See also the [List of combinators][crate::combinator].
//!
//! ### Complex migrations
//!
//! For larger parsers, it is likely best to take smaller steps
//! - Easier to debug when something goes wrong
//! - Deprecation messages will help assist through the process
//!
//! The workflow goes something like:
//! 1. Run `cargo rm nom && cargo add winnow@0.3`
//! 1. Ensure everything compiles and tests pass, ignoring deprecation messages (see [migration
//!    notes](https://github.com/winnow-rs/winnow/blob/main/CHANGELOG.md#nom-migration-guide))
//! 1. Commit
//! 1. Switch any `impl FnMut(I) -> IResult<I, O, E>` to `impl Parser<I, O, E>`
//! 1. Resolve deprecation messages
//! 1. Commit
//! 1. Run `cargo add winnow@0.4`
//! 1. Ensure everything compiles and tests pass, ignoring deprecation messages (see [changelog](https://github.com/winnow-rs/winnow/blob/main/CHANGELOG.md#040---2023-03-18) for more details)
//! 1. Commit
//! 1. Resolve deprecation messages
//! 1. Commit
//! 1. Run `cargo add winnow@0.5`
//! 1. Ensure everything compiles and tests pass, ignoring deprecation messages (see [migration
//!     notes](https://github.com/winnow-rs/winnow/blob/main/CHANGELOG.md#050---2023-07-13))
//! 1. Commit
//! 1. Resolve deprecation messages
//! 1. Commit
//!
//! ### Examples
//!
//! For example migrations, see
//! - [git-config-env](https://github.com/gitext-rs/git-config-env/pull/11) (nom to winnow 0.3)
//! - [git-conventional](https://github.com/crate-ci/git-conventional/pull/37) (nom to winnow 0.3,
//!   adds explicit tracing for easier debugging)
//! - [typos](https://github.com/crate-ci/typos/pull/664) (nom to winnow 0.3)
//! - [cargo-smart-release](https://github.com/Byron/gitoxide/pull/948) (gradual migration from nom
//!   to winnow 0.5)
//! - [gix-config](https://github.com/Byron/gitoxide/pull/951) (gradual migration from nom
//!   to winnow 0.5)
//! - [gix-protocol](https://github.com/Byron/gitoxide/pull/1009) (gradual migration from nom
//!   to winnow 0.5)
//! - [gitoxide](https://github.com/Byron/gitoxide/pull/956) (gradual migration from nom
//!   to winnow 0.5)
//!
//! ## API differences
//!
//! ### Renamed APIs
//!
//! Names have changed for consistency or clarity.
//!
//! To find a parser you are looking for,
//! - Search the docs for the `nom` parser
//! - See the [List of combinators][crate::combinator]
//!
//! ### Partial/streaming parsers
//!
//! `nom` differentiated some parsers by being `streaming` or `complete`.
//! Instead, we tag the input type (`I`) by wrapping it in [`Partial<I>`] and parsers will adjust
//! their behavior accordingly.
//! See [partial] special topic.
//!
//! ### `&mut I`
//!
//! For an explanation of this change, see [Why `winnow`][super::why]
//!
//! To save and restore from intermediate states, [`Stream::checkpoint`] and [`Stream::reset`] can help:
//! ```rust
//! use winnow::prelude::*;
//! # let mut i = "";
//! # let i = &mut i;
//!
//! let start = i.checkpoint();
//! // ...
//! i.reset(&start);
//! ```
//!
//! When the Output of a parser is a slice, you have to add a lifetime:
//! ```rust
//! # use winnow::prelude::*;
//! fn foo<'i>(i: &mut &'i str) -> ModalResult<&'i str> {
//!     // ...
//! #   winnow::token::rest.parse_next(i)
//! }
//! ```
//!
//! When writing a closure, you need to annotate the type:
//! ```rust
//! # use winnow::prelude::*;
//! # use winnow::combinator::trace;
//! fn foo(i: &mut &str) -> ModalResult<usize> {
//!     trace("foo", |i: &mut _| {
//!         // ...
//! #       Ok(0)
//!     }).parse_next(i)
//! }
//! ```
//!
//! ### Optional [`ErrMode`]
//!
//! Called `Err` in `nom`, [`ErrMode`] is responsible for
//! - Deciding whether to backtrack and try another branch in cases like `alt` or report back to
//!   the error back to users
//! - Tracking incomplete input on partial parsing
//!
//! As this isn't needed in every parser, it was made optional.  [`ModalResult`] is a convenience
//! type for using [`ErrMode`].

#![allow(unused_imports)]
use crate::_topic::partial;
use crate::error::ErrMode;
use crate::error::ModalResult;
use crate::stream::Partial;
use crate::stream::Stream;
