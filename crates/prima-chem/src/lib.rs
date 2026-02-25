// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima-Chem — Molecular Primitives for Prima
//!
//! Grounded in Lex Primitiva for cross-domain molecular translation.
//!
//! ## Primitive Grounding
//!
//! | Type | Primitives | Meaning |
//! |------|------------|---------|
//! | Atom | N + λ + ς | Number + Location + State |
//! | Bond | → + N + ∂ | Causality + Order + Boundary |
//! | Molecule | Σ + μ + σ | Sum + Mapping + Sequence |
//! | Reaction | → | Causality transform |
//!
//! ## Tier Classification
//!
//! - Element: T1 (N only)
//! - Atom: T2-P (N + λ)
//! - Bond: T2-P (→ + N)
//! - Molecule: T2-C (Σ + μ + σ + λ)
//! - Reaction: T2-C (→ + σ + ∂)

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

pub mod element;
pub mod error;
pub mod geometry;
pub mod reaction;
pub mod smiles;
pub mod types;

pub mod prelude {
    pub use crate::element::{Element, PERIODIC_TABLE};
    pub use crate::error::{ChemError, ChemResult};
    pub use crate::geometry::{Geometry, GeometryBuilder, Vec3};
    pub use crate::types::{Atom, AtomId, Bond, BondOrder, BondType, Molecule, MoleculeBuilder};
}

pub use element::Element;
pub use error::{ChemError, ChemResult};
pub use types::{Atom, Bond, Molecule};
