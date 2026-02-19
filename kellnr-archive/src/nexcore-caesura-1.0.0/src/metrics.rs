//! Stratum metrics computed during caesura detection.
//!
//! Each detector computes per-file metrics, then flags files
//! whose metrics deviate significantly from the module baseline.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// StyleMetrics — per-file style statistics (T2-C: N+κ+σ)
// ---------------------------------------------------------------------------

/// Tier: T2-C — N Quantity + κ Comparison + σ Sequence
///
/// Style metrics computed for a single source file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleMetrics {
    /// Ratio of snake_case identifiers to total identifiers (0.0-1.0).
    pub snake_case_ratio: f64,
    /// Ratio of camelCase identifiers to total identifiers (0.0-1.0).
    pub camel_case_ratio: f64,
    /// Comment lines / total code lines.
    pub comment_density: f64,
    /// Mean line length in characters.
    pub mean_line_length: f64,
    /// Standard deviation of line lengths.
    pub stddev_line_length: f64,
    /// Total lines in the file.
    pub total_lines: usize,
}

impl StyleMetrics {
    /// Naming entropy: how mixed are the naming conventions?
    /// 0.0 = purely one convention, 1.0 = perfectly mixed.
    pub fn naming_entropy(&self) -> f64 {
        let s = self.snake_case_ratio;
        let c = self.camel_case_ratio;
        if s <= 0.0 || c <= 0.0 {
            return 0.0;
        }
        let total = s + c;
        if total <= 0.0 {
            return 0.0;
        }
        let ps = s / total;
        let pc = c / total;
        -(ps * ps.ln() + pc * pc.ln()) / core::f64::consts::LN_2
    }
}

// ---------------------------------------------------------------------------
// ArchMetrics — per-file architecture statistics (T2-C: N+Σ+∂)
// ---------------------------------------------------------------------------

/// Tier: T2-C — N Quantity + Σ Sum + ∂ Boundary
///
/// Architecture metrics computed for a single source file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchMetrics {
    /// Number of `use` import statements.
    pub import_count: usize,
    /// Number of `pub` items (functions, structs, enums, traits, etc.).
    pub pub_surface: usize,
    /// Number of `mod` declarations.
    pub mod_count: usize,
    /// Ratio of imports to total lines (coupling density).
    pub coupling_density: f64,
}

// ---------------------------------------------------------------------------
// DepMetrics — Cargo.toml dependency statistics (T2-C: N+∂+∝)
// ---------------------------------------------------------------------------

/// Tier: T2-C — N Quantity + ∂ Boundary + ∝ Irreversibility
///
/// Dependency metrics extracted from a Cargo.toml.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepMetrics {
    /// Total dependency count.
    pub dep_count: usize,
    /// Dependencies using workspace = true.
    pub workspace_deps: usize,
    /// Dependencies with explicit version strings.
    pub versioned_deps: usize,
    /// Dependencies with path references.
    pub path_deps: usize,
    /// Dependencies with git references.
    pub git_deps: usize,
}
