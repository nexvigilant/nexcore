//! nexcore Vault - Zero-dependency local encrypted secret manager.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod cipher;

pub mod config;

pub mod error;

pub mod grounding;

pub mod kdf;

pub mod persistence;

pub mod store;

pub mod types;
