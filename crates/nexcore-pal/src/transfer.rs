// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Cross-Domain Transfer
//!
//! Maps between `nexcore-pal` types and other domains.
//!
//! ## Transfer Directions
//!
//! | Source | Destination | Bridge |
//! |--------|-------------|--------|
//! | `InputEvent` | `nexcore-os` IPC | OS event bus |
//! | `PowerState` | `nexcore-os` energy | Power management |
//! | `Resolution` | `nexcore-compositor` | Surface geometry |
//! | `FormFactor` | `nexcore-shell` layout | Shell layout selection |
//!
//! Transfer mappings will be added as cross-domain bridges are identified.

// Transfer mappings will be added as cross-domain bridges are identified.
