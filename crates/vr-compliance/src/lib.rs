//! # vr-compliance — PRPaaS Compliance Engine
//!
//! Provides regulatory compliance infrastructure for the Pharmaceutical
//! Research Platform as a Service:
//!
//! - **[`audit`]** — Immutable audit trail capture and querying for all
//!   significant platform actions. Supports SOC 2 evidence collection
//!   and incident investigation.
//!
//! - **[`gdpr`]** — GDPR data subject request management (Articles 15-20),
//!   consent tracking, and deletion manifest generation. Enforces the
//!   30-day response deadline per Article 12(3).
//!
//! - **[`export_control`]** — Compound data export screening against
//!   sanctioned country lists and dual-use chemical indicators (EAR,
//!   CWC schedules, controlled substances).
//!
//! - **[`soc2`]** — SOC 2 Type II control tracking and compliance
//!   scorecards across all five Trust Services Categories (Security,
//!   Availability, Processing Integrity, Confidentiality, Privacy).

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod audit;
pub mod export_control;
pub mod gdpr;
pub mod soc2;

// Re-export key types at crate root for ergonomic imports.
pub use audit::{AuditEvent, AuditEventType, AuditQuery};
pub use export_control::{ExportRisk, ExportScreeningResult};
pub use gdpr::{
    ConsentRecord, ConsentType, DataSubjectRequest, DeletionManifest, RequestStatus, RequestType,
};
pub use soc2::{ComplianceScorecard, ControlStatus, EvidenceType, Soc2Category, Soc2Control};
