// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Cross-Domain Transfer
//!
//! Maps between `nexcore-os` types and other domains.
//!
//! ## Transfer Directions
//!
//! | Source | Destination | Bridge |
//! |--------|-------------|--------|
//! | `SecurityLevel` | Guardian engine | Threat level escalation |
//! | `ServiceState` | Cytokine IPC | Service lifecycle signals |
//! | `BootPhase` | nexcore-shell | Shell readiness gating |
//! | `VaultState` | nexcore-compositor | Lock screen activation |
//! | `UserManager` | nexcore-shell login | Session-based shell access |
//!
//! Transfer mappings will be added as cross-domain bridges are identified.

// Transfer mappings will be added as cross-domain bridges are identified.
