// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! ICH M7 structural alert library with substructure matching.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod error;
pub mod alert;
pub mod library;
pub mod matcher;

pub use error::{AlertError, AlertResult};
pub use alert::{AlertCategory, AlertMatch, AlertSource, StructuralAlert};
pub use library::AlertLibrary;
pub use matcher::{scan, scan_smiles};
