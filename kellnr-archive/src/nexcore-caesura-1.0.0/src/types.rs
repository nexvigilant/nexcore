//! Core types for caesura detection.
//!
//! A caesura is a structural seam in a codebase where coding style,
//! architecture, dependencies, or temporal patterns shift abruptly.
//! Named after the codicological concept of manuscript discontinuities.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// CaesuraType — what kind of discontinuity (T2-P: ∂ Boundary)
// ---------------------------------------------------------------------------

/// Tier: T2-P — ∂ Boundary
///
/// Classification of discontinuity stratum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaesuraType {
    /// Naming conventions, comment density, line length divergence.
    Stylistic,
    /// Coupling shifts, public API surface changes, import pattern breaks.
    Architectural,
    /// Cargo.toml dependency cluster changes, semver major bumps.
    Dependency,
    /// Temporal/authorship shifts (future: git-based analysis).
    Temporal,
}

impl CaesuraType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stylistic => "stylistic",
            Self::Architectural => "architectural",
            Self::Dependency => "dependency",
            Self::Temporal => "temporal",
        }
    }
}

// ---------------------------------------------------------------------------
// Stratum — which layer the discontinuity lives in (T2-P: ∂ Boundary)
// ---------------------------------------------------------------------------

/// Tier: T2-P — ∂ Boundary
///
/// The stratum (layer) where a caesura is detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stratum {
    /// Source code style layer (naming, formatting, comments).
    Style,
    /// Module architecture layer (imports, coupling, pub surface).
    Architecture,
    /// Dependency manifest layer (Cargo.toml).
    Dependency,
    /// Temporal layer (commit history, authorship).
    Temporal,
}

// ---------------------------------------------------------------------------
// CaesuraScore — severity measure (T2-P: N+κ)
// ---------------------------------------------------------------------------

/// Tier: T2-P — N Quantity + κ Comparison
///
/// Severity score for a detected caesura. Higher = more severe discontinuity.
/// Range: 0.0 (no discontinuity) to 1.0 (maximum divergence).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CaesuraScore(f64);

impl CaesuraScore {
    /// Create a new score, clamped to [0.0, 1.0].
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Raw numeric value.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Severity classification mirroring DriftSeverity pattern.
    pub fn severity(&self) -> CaesuraSeverity {
        match self.0 {
            v if v < 0.25 => CaesuraSeverity::None,
            v if v < 0.50 => CaesuraSeverity::Mild,
            v if v < 0.75 => CaesuraSeverity::Moderate,
            _ => CaesuraSeverity::Severe,
        }
    }
}

/// Tier: T2-P — κ Comparison
///
/// Severity classification for a caesura score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaesuraSeverity {
    /// Score < 0.25 — no meaningful discontinuity.
    None,
    /// Score 0.25..0.50 — minor style or structural drift.
    Mild,
    /// Score 0.50..0.75 — significant seam, likely different author/era.
    Moderate,
    /// Score >= 0.75 — major discontinuity, probable rebinding point.
    Severe,
}

impl CaesuraSeverity {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Mild => "mild",
            Self::Moderate => "moderate",
            Self::Severe => "severe",
        }
    }
}

// ---------------------------------------------------------------------------
// StratumLocation — where the caesura is (T2-P: λ Location)
// ---------------------------------------------------------------------------

/// Tier: T2-P — λ Location
///
/// Pinpoints where a caesura was detected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StratumLocation {
    /// File or directory path.
    pub path: String,
    /// Optional line range (start, end) within the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_range: Option<(usize, usize)>,
    /// Optional git commit SHA (for temporal strata).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

// ---------------------------------------------------------------------------
// Caesura — full discontinuity record (T2-C: ∂+ς+∝+ν)
// ---------------------------------------------------------------------------

/// Tier: T2-C — ∂ Boundary + ς State + ∝ Irreversibility + ν Frequency
///
/// A detected structural seam in the codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caesura {
    /// What kind of discontinuity.
    pub caesura_type: CaesuraType,
    /// Which stratum it was found in.
    pub stratum: Stratum,
    /// Severity score.
    pub score: CaesuraScore,
    /// Where it was detected.
    pub location: StratumLocation,
    /// Human-readable description of the discontinuity.
    pub description: String,
    /// Files involved in the discontinuity boundary.
    pub boundary_files: Vec<String>,
}
