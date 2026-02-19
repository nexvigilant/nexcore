//! # Ghost Mode — Privacy Enforcement Levels
//!
//! ## Primitive Foundation
//!
//! | Primitive | Manifestation |
//! |-----------|---------------|
//! | T1: State (ς) | Four enforcement levels forming an ordered state space |
//!
//! ## Tier: T1 (single primitive: ς State)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Privacy enforcement level.
///
/// Ordered from least to most restrictive. Each level subsumes the
/// guarantees of all lower levels.
///
/// ## Tier: T1 (ς State)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GhostMode {
    /// No privacy enforcement. Development/testing only.
    Off = 0,
    /// Pseudonymize direct identifiers. Reversal permitted with single auth.
    Standard = 1,
    /// Pseudonymize all PII. Dual-authorization required for reversal.
    Strict = 2,
    /// Full anonymization. No reversal path exists.
    Maximum = 3,
}

impl GhostMode {
    /// Whether privacy enforcement is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::Off)
    }

    /// Whether reversal (re-identification) is permitted at this level.
    #[must_use]
    pub const fn allows_reversal(&self) -> bool {
        matches!(self, Self::Off | Self::Standard | Self::Strict)
    }

    /// Whether dual-authorization is required for reversal.
    #[must_use]
    pub const fn requires_dual_auth(&self) -> bool {
        matches!(self, Self::Strict)
    }

    /// Minimum k-anonymity target for this mode.
    #[must_use]
    pub const fn min_k_anonymity(&self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Standard => 3,
            Self::Strict => 5,
            Self::Maximum => 10,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Off => "Ghost: Off",
            Self::Standard => "Ghost: Standard",
            Self::Strict => "Ghost: Strict",
            Self::Maximum => "Ghost: Maximum",
        }
    }

    /// Numeric level (0-3).
    #[must_use]
    pub const fn level(&self) -> u8 {
        *self as u8
    }
}

impl Default for GhostMode {
    fn default() -> Self {
        Self::Standard
    }
}

impl fmt::Display for GhostMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_ordering() {
        assert!(GhostMode::Off < GhostMode::Standard);
        assert!(GhostMode::Standard < GhostMode::Strict);
        assert!(GhostMode::Strict < GhostMode::Maximum);
    }

    #[test]
    fn off_is_inactive() {
        assert!(!GhostMode::Off.is_active());
    }

    #[test]
    fn standard_is_active() {
        assert!(GhostMode::Standard.is_active());
    }

    #[test]
    fn maximum_forbids_reversal() {
        assert!(!GhostMode::Maximum.allows_reversal());
    }

    #[test]
    fn strict_requires_dual_auth() {
        assert!(GhostMode::Strict.requires_dual_auth());
        assert!(!GhostMode::Standard.requires_dual_auth());
    }

    #[test]
    fn k_anonymity_increases_with_mode() {
        assert!(GhostMode::Standard.min_k_anonymity() < GhostMode::Strict.min_k_anonymity());
        assert!(GhostMode::Strict.min_k_anonymity() < GhostMode::Maximum.min_k_anonymity());
    }

    #[test]
    fn default_is_standard() {
        assert_eq!(GhostMode::default(), GhostMode::Standard);
    }

    #[test]
    fn serde_roundtrip() {
        let mode = GhostMode::Strict;
        let json = serde_json::to_string(&mode).unwrap_or_default();
        let back: GhostMode = serde_json::from_str(&json).unwrap_or(GhostMode::Off);
        assert_eq!(back, mode);
    }
}
