//! # Cookbook
//!
//! These are short recipes for accomplishing common tasks.
//!
//! - [Elements of Programming Languages][language]
//! - [Implementing `FromStr`][fromstr]
//! - [Custom stream][stream]
//! - [Custom errors][error]
//!
//! See also parsers written with `winnow`:
//!
//! - [`toml_edit`](https://crates.io/crates/toml_edit)

pub mod error;
pub mod fromstr;
pub mod language;
pub mod stream;
