//! # nexcore-caesura — Structural Seam Detection
//!
//! Inspired by codicology (the study of manuscript discontinuities),
//! caesura detection finds structural seams in codebases where coding style,
//! architecture, dependencies, or temporal patterns shift abruptly.
//!
//! These seams are where bugs and tech debt concentrate — the software
//! equivalent of a manuscript rebinding.
//!
//! ## Primitive Grounding
//!
//! ∂ Boundary (dominant) + ς State + ∝ Irreversibility + ν Frequency = T2-C
//!
//! ## Detection Strata
//!
//! | Stratum | Detector | What It Finds |
//! |---------|----------|---------------|
//! | Style | `StyleDetector` | Naming convention shifts, comment density changes, line length divergence |
//! | Architecture | `ArchDetector` | Coupling density shifts, pub surface changes, import pattern breaks |
//! | Dependency | `DepDetector` | Dep clusters, non-workspace ratio, git deps in workspace context |
//!
//! ## Example
//!
//! ```rust,no_run
//! use nexcore_caesura::CaesuraDetector;
//! use std::path::Path;
//!
//! let detector = CaesuraDetector::with_sensitivity(1.5);
//! let caesuras = detector.scan(Path::new("src/")).expect("scan failed");
//! let report = CaesuraDetector::report(&caesuras);
//! println!("{report}");
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod architecture;
pub mod dependency;
pub mod detector;
pub mod grounding;
pub mod metrics;
pub mod style;
pub mod types;

// Re-export key types at crate root for ergonomic access.
pub use detector::CaesuraDetector;
pub use metrics::{ArchMetrics, DepMetrics, StyleMetrics};
pub use types::{Caesura, CaesuraScore, CaesuraSeverity, CaesuraType, Stratum, StratumLocation};
