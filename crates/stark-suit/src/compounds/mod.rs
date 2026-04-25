//! # Compounds: 12 components organized into 4 functional groups.
//!
//! Each submodule re-exports the public surface of 2–4 component crates
//! under a single namespace. The grouping reflects natural functional
//! boundaries (sense / decide / energize / interact), not arbitrary
//! alphabetical alias.

pub mod control;
pub mod human_interface;
pub mod perception;
pub mod power;
