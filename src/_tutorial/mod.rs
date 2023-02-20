//! # Tutorial
//!
//! This tutorial assumes that you are:
//! - Already familiar with Rust
//! - Using `winnow` for the first time
//!
//! The focus will be on parsing in-memory strings (`&str`). Once done, you might want to check the
//! [Special Topics][_topic] for more specialized topics or examples.
//!
//! ## About
//!
//! `winnow` is a parser-combinator library. In other words, it gives you tools to define:
//! - "parsers", or functions that takes an input and gives back an output
//! - "combinators", or functions that take parsers and _combine_ them together!
//!
//! While "combinator" might be an unfamiliar word, you are likely using them in your rust code
//! today, like with the [`Iterator`] trait:
//! ```rust
//! let data = vec![1, 2, 3, 4, 5];
//! let even_count = data.iter()
//!     .copied()  // combinator
//!     .filter(|d| d % 2 == 0)  // combinator
//!     .count();  // combinator
//! ```
//!
//! Parser combinators allow building parsers for complex formats from simple, reusable parsers.
//!
//! [*next*][chapter_1]

#![allow(unused_imports)]
use crate::_topic;
use std::iter::Iterator;

pub mod chapter_1;
pub mod chapter_2;
pub mod chapter_3;
pub mod chapter_4;
pub mod chapter_5;
pub mod chapter_6;
pub mod chapter_7;
