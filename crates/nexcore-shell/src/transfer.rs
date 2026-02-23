// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Cross-Domain Transfer
//!
//! Maps between `nexcore-shell` types and other domains.
//!
//! ## Transfer Directions
//!
//! | Source | Destination | Bridge |
//! |--------|-------------|--------|
//! | `ShellState` | `nexcore-os` service | OS service lifecycle |
//! | `InputAction` | `nexcore-compositor` input | Compositor focus routing |
//! | `AppId` | `nexcore-compositor` surface | Surface-to-app binding |
//! | `NotificationPriority` | `nexcore-os` security | Priority-to-threat mapping |
//! | `LoginState` | `nexcore-os` user | Authentication session handoff |
//!
//! Transfer mappings will be added as cross-domain bridges are identified.

// Transfer mappings will be added as cross-domain bridges are identified.
