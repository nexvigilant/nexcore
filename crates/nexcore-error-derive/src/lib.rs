//! Derive macro for NexVigilant error types.
//!
//! Drop-in replacement for `thiserror::Error`. Provides `#[derive(Error)]`
//! with `#[error("...")]`, `#[from]`, and `#[source]` attributes.
//!
//! # Usage
//!
//! ```
//! use nexcore_error_derive::Error;
//!
//! #[derive(Debug, Error)]
//! pub enum MyError {
//!     #[error("IO error: {0}")]
//!     Io(#[from] std::io::Error),
//!     #[error("parse error: {msg}")]
//!     Parse { msg: String },
//!     #[error("not found")]
//!     NotFound,
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
mod attr;
mod expand;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// Derive `Display` and `std::error::Error` for an enum or struct.
///
/// # Attributes
///
/// - `#[error("format string")]` — Display format for the variant/struct
/// - `#[error(transparent)]` — Forward Display and source to inner type
/// - `#[from]` on a field — Generate `From<FieldType>` impl
/// - `#[source]` on a field — Use as `Error::source()` return value
#[proc_macro_derive(Error, attributes(error, from, source))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand::expand(&input) {
        Ok(expanded) => expanded.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
