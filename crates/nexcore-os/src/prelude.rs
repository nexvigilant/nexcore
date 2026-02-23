// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # OS Prelude
//!
//! Convenience re-exports of the most-used types from `nexcore-os`.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_os::prelude::*;
//! ```
//!
//! This brings into scope the kernel, all key subsystem types, and the
//! Lex Primitiva grounding infrastructure.

// Kernel
pub use crate::kernel::NexCoreOs;

// Boot
pub use crate::boot::{BootPhase, BootSequence};

// Error
pub use crate::error::OsError;

// IPC
pub use crate::ipc::EventBus;

// Network
pub use crate::network::{NetworkManager, NetworkState};

// Secure boot
pub use crate::secure_boot::{BootStage, SecureBootChain};

// Security
pub use crate::security::{
    Damp, Pamp, SecurityLevel, SecurityMonitor, SecurityResponse, ThreatPattern, ThreatSeverity,
};

// Service
pub use crate::service::{Service, ServiceId, ServiceState};

// User
pub use crate::user::{
    AccountStatus, AuthError, Session, UserId, UserManager, UserRecord, UserRole,
};

// Vault
pub use crate::vault::{OsVault, SecretCategory, VaultState};

// App clearance
pub use crate::app_clearance::{AppClearanceLevel, AppPermission, ClearanceResult};

// Grounding
pub use crate::primitives::{GroundsTo, LexPrimitiva, PrimitiveComposition, Tier};
