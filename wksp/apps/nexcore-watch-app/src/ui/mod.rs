#![allow(dead_code)]
//! UI module — Slint bindings for watch screens.
//!
//! ## Primitive Grounding
//! - μ (Mapping): data model → visual properties
//! - ς (State): screen navigation FSM
//! - σ (Sequence): render pipeline
//!
//! ## Tier: T3

pub mod alerts_screen;
pub mod guardian_screen;
pub mod navigation;
pub mod signal_screen;
