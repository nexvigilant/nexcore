//! Zero-dependency codec library for the NexCore ecosystem.
//!
//! # Supply Chain Sovereignty
//!
//! This crate replaces multiple external encoding/decoding crates:
//!
//! | Module | Replaces | Spec |
//! |--------|----------|------|
//! | [`hex`] | `hex` crate | RFC 4648 §8 (Base16) |
//! | [`base64`] | `base64` crate | RFC 4648 §4/§5 (Base64) |
//!
//! # Examples
//!
//! ```
//! // Hex encoding
//! let encoded = nexcore_codec::hex::encode(b"NexVigilant");
//! assert_eq!(encoded, "4e65785669676964616e74");
//!
//! // Base64 encoding
//! let encoded = nexcore_codec::base64::encode(b"Hello");
//! assert_eq!(encoded, "SGVsbG8=");
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod base64;
pub mod hex;
