//! # Custom Errors
//!
//! The most basic error type is [`ParseError`][crate::error::ParseError]
//!
//! Optional traits include:
//! - [`AddContext`][crate::error::AddContext]
//! - [`FromExternalError`][crate::error::FromExternalError]
//!
//! # Example
//!
//!```rust
#![doc = include_str!("../../examples/custom_error.rs")]
//!```
