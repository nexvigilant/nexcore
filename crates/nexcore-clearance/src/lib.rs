//! # NexVigilant Core — clearance
//!
//! 5-level security classification system with tiered enforcement for NexVigilant.
//!
//! ## Overview
//!
//! Government-style classification (Public → Top Secret) with Claude behavioral
//! modes (Unrestricted → Lockdown). Foundation layer — no dependency on
//! vigilance or guardian.
//!
//! ## Classification Levels
//!
//! | Level | Ordinal | AccessMode | Key Enforcement |
//! |-------|---------|------------|-----------------|
//! | Public | 0 | Unrestricted | None |
//! | Internal | 1 | Aware | Audit logging |
//! | Confidential | 2 | Guarded | Warn before output |
//! | Secret | 3 | Enforced | Block violations, dual-auth |
//! | Top Secret | 4 | Lockdown | No external tools, full audit |
//!
//! ## Priority
//!
//! Classification sits at P2c in the Guardian hierarchy:
//! P0 Patient Safety > P1 Signal > P2 Regulatory > P2b Privacy > **P2c Classification** > P3-P5
//!
//! ## Example
//!
//! ```rust
//! use nexcore_clearance::{ClassificationLevel, ClearanceGate, TagTarget};
//!
//! let mut gate = ClearanceGate::new();
//! let target = TagTarget::File("algo.rs".into());
//! let result = gate.evaluate_access(&target, ClassificationLevel::Internal, "claude");
//! assert!(result.is_pass());
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod access;
pub mod audit;
pub mod composites;
pub mod config;
pub mod error;
pub mod gate;
pub mod grounding;
pub mod level;
pub mod policy;
pub mod prelude;
pub mod primitives;
pub mod priority;
pub mod tag;
pub mod transfer;
pub mod validator;

// ── Public Re-exports ───────────────────────────────────────────────

pub use access::AccessMode;
pub use audit::{AuditAction, ClearanceAudit, ClearanceEntry};
pub use config::ClearanceConfig;
pub use error::ClearanceError;
pub use gate::{ClearanceGate, GateResult};
pub use level::ClassificationLevel;
pub use policy::ClearancePolicy;
pub use priority::ClearancePriority;
pub use tag::{ClassificationTag, TagTarget};
pub use validator::{ChangeDirection, CrossBoundaryValidator, ValidationResult};
