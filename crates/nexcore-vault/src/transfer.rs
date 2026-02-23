//! # Cross-Domain Transfer
//!
//! Maps between this crate's types and other domains.
//!
//! ## Established Transfer Confidence
//!
//! | Source Domain       | Target Domain         | Confidence | Bridge Primitive |
//! |---------------------|-----------------------|------------|-----------------|
//! | PBKDF2 key stretching | Secret derivation    | 0.95       | ρ (Recursion — iterated hashing) |
//! | AES-GCM AEAD cipher | Authenticated secrecy | 0.93       | ∂ + π (Boundary + Persistence) |
//! | Unix file permissions | Vault access modes  | 0.80       | κ + ∂ (Comparison + Boundary) |
//!
// Transfer mappings will be added as cross-domain bridges are identified.
