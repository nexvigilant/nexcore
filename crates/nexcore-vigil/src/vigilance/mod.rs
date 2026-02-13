//! # Vigilance Subsystem — π(∂·ν)|∝
//!
//! The formal definition of vigilance implemented as a running system:
//!
//! **Vigilance = π(∂·ν)|∝ — Persistent boundary-watching, motivated by irreversibility.**
//!
//! ## 4 Layers = 4 Primitives
//!
//! ```text
//! WatchSources ──→ [ν Watcher] ──→ [∂ Boundary Gate] ──→ [∝ Consequence Pipeline]
//!                       │                   │                       │
//!                       └─────────┬─────────┘                       │
//!                                 ▼                                 ▼
//!                         [π Vigilance Ledger] ◄────────────────────┘
//! ```
//!
//! Every event, every boundary check, every consequence is recorded in
//! the ledger BEFORE execution.

pub mod boundary;
pub mod consequence;
pub mod daemon;
pub mod error;
pub mod event;
pub mod ledger;
pub mod sources;
pub mod vigil_config;
pub mod vigil_grounding;
pub mod watcher;

// Re-exports for ergonomic access
pub use boundary::{BoundaryGate, BoundarySpec, BoundaryViolation, ThresholdCheck};
pub use consequence::{
    AlertConsequence, Consequence, ConsequenceOutcome, ConsequencePipeline, ConsequenceReceipt,
    EscalationLevel, LogConsequence, NotifyConsequence, ShellConsequence, WebhookConsequence,
};
pub use daemon::{ShutdownHandle, VigilDaemon, VigilHealth, VigilStats};
pub use error::{VigilError, VigilResult};
pub use event::{EventId, EventKind, EventSeverity, WatchEvent};
pub use ledger::{LedgerEntry, LedgerEntryType, LedgerQuery, VigilanceLedger};
pub use vigil_config::{ConsequenceConfig, SourceConfig, VigilConfig};
pub use watcher::{WatchSource, Watcher};
