// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! ICH M7 structural alert library with substructure matching.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![warn(missing_docs)]
pub mod alert;
pub mod error;
pub mod library;
pub mod matcher;

pub use alert::{AlertCategory, AlertMatch, AlertSource, StructuralAlert};
pub use error::{AlertError, AlertResult};
pub use library::AlertLibrary;
pub use matcher::{scan, scan_smiles};
