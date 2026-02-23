pub mod guardian;

// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

// # NexCore OS — Operating System Core Runtime
//
// The init system, service manager, and process model for NexCore OS.
//
// ## Architecture
//
// ```text
// ┌─────────────────────────────────────────────────────────┐
// │              NexCore Shell (UI Layer)                    │
// ├─────────────────────────────────────────────────────────┤
// │              NexCore App Runtime (Skills)                │
// ├─────────────────────────────────────────────────────────┤
// │              NexCore System Services                     │
// │  Guardian │ Brain │ Vigil │ Cortex │ Vault │ Sentinel    │
// ├─────────────────────────────────────────────────────────┤
// │           NexCore OS Core (this crate)                   │ ← You are here
// │  STOS │ Cytokine IPC │ Clearance │ Energy │ Network │ Audio │ PAL │
// ├─────────────────────────────────────────────────────────┤
// │         Platform Abstraction Layer (nexcore-pal)          │
// ├─────────────────────────────────────────────────────────┤
// │              Linux Kernel (6.x)                          │
// └─────────────────────────────────────────────────────────┘
// ```
//
// ## Boot Sequence (σ Sequence)
//
// 1. PAL initialization (hardware probing)
// 2. STOS kernel boot (state machine runtime)
// 3. System service startup (Guardian, Brain, etc.)
// 4. Shell launch (per-device UI)
//
// ## Primitive Grounding
//
// | Component     | Primitives           | Role                     |
// |---------------|----------------------|--------------------------|
// | Boot sequence | σ + → + ∂            | Ordered causal startup   |
// | Service mgr   | ς + Σ + ∃            | Service state tracking   |
// | Event loop    | σ + ν + ρ            | Recurring event dispatch |
// | Security      | ∂ + ς + κ            | Threat detection/response|
// | Vault         | ∂ + μ + π + ς        | Encrypted secret storage |
// | Network       | Σ + ∂ + ς + μ + →    | Interfaces/DNS/firewall  |
// | Audio         | Σ + σ + ν + ς + ∂ + N| Devices/streams/mixing   |
// | Secure boot   | σ + → + ∂ + ∝ + κ    | Measured boot chain      |
// | Users/Login   | ∂ + κ + ς + π + μ    | Authentication & sessions|
// | Persistence   | π + ∃ + ς            | State crash recovery     |
// | Shutdown      | σ + ∝ + ∅            | Irreversible teardown    |

pub mod app_clearance;
pub mod audio;
pub mod boot;
pub mod brain_bridge;
pub mod composites;
pub mod config;
pub mod diagnostics;
pub mod error;
pub mod grounding;
pub mod guardian_bridge;
pub mod ipc;
pub mod journal;
pub mod kernel;
pub mod network;
pub mod persistence;
pub mod prelude;
pub mod primitives;
pub mod repl;
pub mod secure_boot;
pub mod security;
pub mod service;
pub mod transfer;
pub mod user;
pub mod vault;

// Re-export main types
pub use app_clearance::{
    AppClearanceGate, AppClearanceLevel, AppManifest, AppPermission, ClearanceResult,
};
pub use audio::{AudioManager, AudioState};
pub use boot::BootSequence;
pub use config::{SystemConfig, TrustOsConfig, hill_curve_backoff_ms};
pub use diagnostics::{DiagnosticSnapshot, HealthStatus, ServiceHealth};
pub use error::OsError;
pub use ipc::EventBus;
pub use journal::{
    Field, FieldValue, JournalEntry, JournalFilter, Keywords, OsJournal, Severity, Subsystem,
};
pub use kernel::NexCoreOs;
pub use network::{NetworkManager, NetworkState};
pub use persistence::{OsStateSnapshot, StatePersistence};
pub use repl::{OsRepl, ReplCommand, ReplOutput};
pub use secure_boot::{
    AttestationRecord, BootPolicy, BootQuote, BootStage, ChainVerification, Measurement,
    SecureBootChain, VerifyResult,
};
pub use security::{
    Damp, Pamp, SecurityLevel, SecurityMonitor, SecurityResponse, ThreatPattern, ThreatSeverity,
};
pub use service::{Service, ServiceId, ServiceState};
pub use user::{
    AccountStatus, AuthError, Session, UserId, UserManager, UserRecord, UserRole, UserSummary,
};
pub use vault::{OsVault, SecretCategory, SecretInfo, VaultState};
