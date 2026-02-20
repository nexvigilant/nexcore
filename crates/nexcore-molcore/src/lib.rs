// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-molcore — Molecular Core
//!
//! Pure Rust cheminformatics foundation for the Chemivigilance Platform.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod arom;
pub mod descriptor;
pub mod error;
pub mod fingerprint;
pub mod graph;
pub mod ring;
pub mod smiles;
pub mod substruct;

pub use error::{MolcoreError, MolcoreResult};

/// Re-export prima-chem types used throughout.
pub mod prelude {
    pub use crate::error::{MolcoreError, MolcoreResult};
    pub use crate::graph::MolGraph;
    pub use prima_chem::prelude::*;
}
