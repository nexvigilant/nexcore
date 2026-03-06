//! Firebase Auth, Firestore, and Stripe REST API clients.
//!
//! Enable the `client` feature (default) for async HTTP implementations.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
pub mod auth;
pub mod firestore;
pub mod grounding;
pub mod stripe;
