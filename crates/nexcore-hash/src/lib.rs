#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Zero-dependency cryptographic hashing for the NexCore ecosystem.
//!
//! Replaces `sha2`, `hmac`, and `digest` crates with NexVigilant-owned
//! implementations validated against NIST CAVP and RFC 4231 test vectors.

pub mod hmac;
pub mod sha256;
