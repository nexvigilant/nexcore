// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Code generation backends for target languages.
//!
//! ## Tier: T2-C (μ + Σ)

mod c;
mod go;
mod python;
mod rust;
mod typescript;

pub use c::CBackend;
pub use go::GoBackend;
pub use python::PythonBackend;
pub use rust::RustBackend;
pub use typescript::TypeScriptBackend;
