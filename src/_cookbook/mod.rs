//! # Cookbook
//!
//! These are short recipes for accomplishing common tasks.
//!
//! - [Elements of Programming Languages][language]
//! - [Arithmetic][arithmetic]
//! - [s-expression][s_expression]
//! - [json]
//! - [Implementing `FromStr`][fromstr]
//! - [Parsing Partial Input][partial]
//! - [Custom stream][stream]
//! - [Custom errors][error]
//!
//! See also parsers written with `winnow`:
//!
//! - [`toml_edit`](https://crates.io/crates/toml_edit)

pub mod arithmetic;
pub mod error;
pub mod fromstr;
pub mod json;
pub mod language;
pub mod partial;
pub mod s_expression;
pub mod stream;
