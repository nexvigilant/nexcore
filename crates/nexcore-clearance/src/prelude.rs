//! # Clearance Prelude
//!
//! Convenience re-exports for the most common clearance types.
//!
//! ## Usage
//!
//! ```rust
//! use nexcore_clearance::prelude::*;
//!
//! let mut gate = ClearanceGate::new();
//! let target = TagTarget::File("sensitive.rs".into());
//! let result = gate.evaluate_access(&target, ClassificationLevel::Internal, "claude");
//! assert!(result.is_pass());
//! ```

// Classification fundamentals
pub use crate::ClassificationLevel;
pub use crate::ClassificationTag;
pub use crate::TagTarget;

// Gate and result
pub use crate::ClearanceGate;
pub use crate::GateResult;

// Access control
pub use crate::AccessMode;

// Audit
pub use crate::AuditAction;
pub use crate::ClearanceAudit;
pub use crate::ClearanceEntry;

// Policy and priority
pub use crate::ClearancePolicy;
pub use crate::ClearancePriority;

// Configuration
pub use crate::ClearanceConfig;

// Error
pub use crate::ClearanceError;

// Validation
pub use crate::ChangeDirection;
pub use crate::CrossBoundaryValidator;
pub use crate::ValidationResult;
