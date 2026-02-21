#![allow(dead_code)]
//! NexVigilant theme constants for watch UI.
//!
//! ## Primitive Grounding
//! - μ (Mapping): state → color, priority → color
//! - ∂ (Boundary): display constraints (450×450 round)
//! - κ (Comparison): priority ordering reflected in visual hierarchy
//!
//! ## Tier: T2-P (μ + ∂)
//!
//! Ported from Kotlin `Color.kt` — identical hex values.

/// NexVigilant brand colors.
///
/// ## Primitive: μ (Mapping) — semantic name → hex RGB
/// ## Tier: T1
pub struct NexColors;

impl NexColors {
    // Brand
    pub const PRIMARY: u32 = 0xFF1A73E8; // NexVigilant Blue
    pub const PRIMARY_DARK: u32 = 0xFF0D47A1;
    pub const ACCENT: u32 = 0xFF00BCD4;

    // Guardian state colors — match guardian.rs color_hex()
    pub const NOMINAL: u32 = 0xFF4CAF50; // Green
    pub const ELEVATED: u32 = 0xFFFF9800; // Orange
    pub const ALERT: u32 = 0xFFFF5722; // Red-Orange
    pub const CRITICAL: u32 = 0xFFF44336; // Red

    // P0-P5 priority colors
    pub const P0_PATIENT_SAFETY: u32 = 0xFFF44336; // Red — matches Critical
    pub const P1_SIGNAL: u32 = 0xFFFF5722; // Red-Orange
    pub const P2_REGULATORY: u32 = 0xFFFF9800; // Orange
    pub const P3_DATA: u32 = 0xFFFFC107; // Amber
    pub const P4_OPERATIONAL: u32 = 0xFF8BC34A; // Light Green
    pub const P5_COST: u32 = 0xFF9E9E9E; // Grey

    // Background
    pub const BACKGROUND: u32 = 0xFF121212; // Dark
    pub const SURFACE: u32 = 0xFF1E1E1E; // Slightly lighter
    pub const ON_SURFACE: u32 = 0xFFFFFFFF; // White text
    pub const ON_SURFACE_DIM: u32 = 0xB3FFFFFF; // 70% white
}

/// Watch display dimensions.
///
/// ## Primitive: ∂ (Boundary) + N (Quantity)
/// ## Tier: T2-P
pub struct WatchDisplay;

impl WatchDisplay {
    /// Galaxy Watch7 display diameter in pixels
    pub const DIAMETER: u32 = 450;
    /// Usable radius (accounting for bezel)
    pub const USABLE_RADIUS: u32 = 210;
    /// Font sizes
    pub const FONT_TITLE: f32 = 18.0;
    pub const FONT_BODY: f32 = 14.0;
    pub const FONT_METRIC: f32 = 24.0;
    pub const FONT_LABEL: f32 = 11.0;
}
