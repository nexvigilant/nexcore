//! # Composite Types
//!
//! Compound types composed from this crate's primitives.
//!
//! The vault crate's primary composites are defined across its modules:
//!
//! | Composite        | Tier | Composition |
//! |------------------|------|-------------|
//! | `SecretName`     | T2-P | ∃ + κ (Existence + Comparison/validation) |
//! | `Salt`           | T2-P | ρ (Recursion — KDF input) |
//! | `VaultEntry`     | T2-C | μ + π + ∝ (Mapping + Persist + Irreversibility) |
//! | `VaultFile`      | T3   | Full encrypted store |
//! | `Vault`          | T3   | Stateful secret manager |
//! | `PlaintextExport`| T2-C | μ + ς (Mapping + State — ephemeral form) |
//!
// Currently empty — composites will be added as the crate evolves.
