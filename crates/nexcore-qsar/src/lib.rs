// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Rule-based toxicity prediction with applicability domain.
//!
//! `nexcore-qsar` implements Phase 1 of the Chemivigilance QSAR pipeline:
//! rule-based classification of mutagenicity, hepatotoxicity, and
//! cardiotoxicity using molecular descriptors from `nexcore-molcore`.
//!
//! ## Quick start
//!
//! ```rust
//! use nexcore_qsar::predict::predict_from_smiles;
//! use nexcore_qsar::types::{RiskLevel, ToxClass};
//!
//! let profile = predict_from_smiles("CC(=O)Oc1ccccc1C(=O)O", 0, 0)
//!     .unwrap_or_default();
//! assert_eq!(profile.mutagenicity.classification, ToxClass::Negative);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]

pub mod applicability;
pub mod cardiotoxicity;
pub mod error;
pub mod hepatotoxicity;
pub mod mutagenicity;
pub mod predict;
pub mod types;

pub use error::{QsarError, QsarResult};
pub use predict::{predict_from_descriptors, predict_from_smiles, predict_toxicity};
pub use types::{BindingHit, DomainStatus, PredictionResult, RiskLevel, ToxClass, ToxProfile};
