// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Cross-Domain Transfer
//!
//! Maps between `nexcore-compositor` types and other domains.
//!
//! ## Transfer Directions
//!
//! | Source | Destination | Bridge |
//! |--------|-------------|--------|
//! | `Surface` bounds | `nexcore-shell` layout | Layout region sizing |
//! | `CompositorMode` | `nexcore-shell` UI | Form-factor adaptive shell |
//! | `InputTarget` | `nexcore-shell` input | Focus-routed input events |
//! | `CompositorState` | `nexcore-os` service | OS power/sleep lifecycle |
//! | `TilingLayout` | `nexcore-shell` launcher | Tile-aware app placement |
//!
//! Transfer mappings will be added as cross-domain bridges are identified.

// Transfer mappings will be added as cross-domain bridges are identified.
