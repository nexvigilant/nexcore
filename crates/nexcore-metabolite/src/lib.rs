// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Phase I/II metabolite and degradant prediction engine.
//!
//! This crate provides rule-based prediction of Phase I metabolites
//! (CYP-mediated oxidation, N-dealkylation, O-dealkylation, hydroxylation,
//! epoxidation), Phase II conjugates (glucuronidation, sulfation, acetylation,
//! glutathione conjugation), and degradation products (hydrolysis) for a
//! parent molecule supplied as a SMILES string.
//!
//! # Architecture
//!
//! ```text
//! predict_from_smiles(smiles)
//!   └─ parse(smiles) → MolGraph
//!         ├─ phase1::predict_phase1  → Vec<Metabolite>
//!         ├─ phase2::predict_phase2  → Vec<Metabolite>
//!         └─ degradant::predict_degradants → Vec<Metabolite>
//!               └─ MetaboliteTree
//! ```
//!
//! # Quick Start
//!
//! ```rust
//! use nexcore_metabolite::predict_from_smiles;
//! use nexcore_metabolite::types::Transformation;
//!
//! // Predict metabolites for ethanol
//! let tree = predict_from_smiles("CCO").unwrap_or_default();
//! assert_eq!(tree.parent_smiles, "CCO");
//! assert!(!tree.phase2.is_empty());
//!
//! // Invalid SMILES returns an error
//! assert!(predict_from_smiles("INVALID$$").is_err());
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]
pub mod degradant;
pub mod error;
pub mod phase1;
pub mod phase2;
pub mod predict;
pub mod types;

pub use error::{MetaboliteError, MetaboliteResult};
pub use predict::{predict_from_smiles, predict_metabolites};
pub use types::{Metabolite, MetaboliteTree, Transformation};
