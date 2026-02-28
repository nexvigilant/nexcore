//! Bridge between on-disk manifests and in-memory topology types.
//!
//! Three capabilities:
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`manifest`] | TOML manifest loading and serialization |
//! | [`scanner`] | Workspace Cargo.toml parsing for dependency edges |
//! | [`bootstrap`] | Topology JSON → TOML manifest generation |

pub mod bootstrap;
pub mod manifest;
pub mod scanner;

#[cfg(test)]
mod tests;
