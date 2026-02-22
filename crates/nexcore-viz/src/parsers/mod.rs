//! Molecular file format parsers.
//!
//! Each sub-module parses a specific chemical file format and produces
//! [`crate::molecular::Molecule`] values for downstream rendering.
//!
//! # Supported formats
//!
//! | Module | Format | Spec |
//! |--------|--------|------|
//! | [`sdf`] | MDL SDF / MOL V2000 | Elsevier MDL CTfile Formats 2011 |
//!
//! # Error handling
//!
//! Parsers return [`ParseError`] — a crate-local error enum that carries the
//! line number and a human-readable description of the failure.  No `anyhow`
//! or `std::io` coupling at this layer; callers convert as needed.

pub mod pdb;
pub mod sdf;

pub use sdf::ParseError;
