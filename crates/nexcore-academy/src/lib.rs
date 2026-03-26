// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # nexcore-academy — Deterministic Academy Logic
//!
//! Pure computation crate for academy course generation pipeline.
//! All logic is deterministic — no I/O, no async, no network calls.
//!
//! ## Modules
//!
//! | Module | Source (Python) | What It Does |
//! |--------|----------------|--------------|
//! | [`quality`] | `ksb_quality_validator.py` | Source credibility scoring, Bloom's taxonomy, quality thresholds |
//! | [`ksb`] | `models/course.py` | KSB types, framework decomposition, fuzzy matching |
//! | [`formatting`] | `academy_formatter.py` | Duration calculation, course structure, JSON output |
//! | [`validation`] | `course_validation_service.py` | Structure/content/assessment checks, scoring |
//! | [`stage`] | `stage_config_service.py` | Pipeline stage enum, AI model config, agent weights |
//!
//! ## Primitive Grounding
//!
//! This crate is a μ (mapping) from Python orchestration into typed Rust computation.
//! The ∂ (boundary) is sharp: deterministic rules live here, AI orchestration stays in Python.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

pub mod formatting;
pub mod ksb;
pub mod quality;
pub mod stage;
pub mod validation;

// Convenience re-exports
pub use formatting::{
    AcademyCourse, FormatCourseParams, Lesson, Module, estimate_duration_minutes, format_course,
};
pub use ksb::{KsbCategory, KsbComponent, KsbFramework, ksb_similarity, match_ksbs};
pub use quality::{QualityResult, bloom_coverage, source_credibility_score, validate_quality};
pub use stage::{PipelineStage, compute_weighted_score, default_validation_agents};
pub use validation::{QualityRating, ValidationReport, ValidationStatus, validate_course};
