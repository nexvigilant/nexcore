//! # Access Mode
//!
//! Claude behavioral modes tied to classification levels.
//!
//! ## Primitive Grounding
//! - **Tier**: T1
//! - **Dominant**: ς State (each mode is a behavioral state)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Claude behavioral mode determined by classification context.
///
/// Each mode progressively restricts what Claude can do.
///
/// ## Tier: T1
/// ## Dominant: ς State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum AccessMode {
    /// Full capability, no constraints.
    Unrestricted = 0,
    /// Sees classification tags, logs access.
    Aware = 1,
    /// Warns before crossing classification boundaries.
    Guarded = 2,
    /// Actively refuses classification violations.
    Enforced = 3,
    /// Minimal mode — no external calls, full audit.
    Lockdown = 4,
}

impl AccessMode {
    /// Returns the numeric ordinal (0–4).
    #[must_use]
    pub fn ordinal(self) -> u8 {
        self as u8
    }

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Unrestricted => "Unrestricted",
            Self::Aware => "Aware",
            Self::Guarded => "Guarded",
            Self::Enforced => "Enforced",
            Self::Lockdown => "Lockdown",
        }
    }

    /// Whether enforcement is active (Enforced or Lockdown).
    #[must_use]
    pub fn is_enforcement_active(self) -> bool {
        self.ordinal() >= Self::Enforced.ordinal()
    }

    /// Whether cross-boundary operations are allowed.
    #[must_use]
    pub fn allows_cross_boundary(self) -> bool {
        self.ordinal() < Self::Enforced.ordinal()
    }

    /// Whether external tool calls are permitted.
    #[must_use]
    pub fn allows_external_calls(self) -> bool {
        self.ordinal() < Self::Lockdown.ordinal()
    }

    /// Whether full audit trail is required.
    #[must_use]
    pub fn requires_full_audit(self) -> bool {
        self.ordinal() >= Self::Lockdown.ordinal()
    }

    /// Whether this mode requires logging all access.
    #[must_use]
    pub fn requires_access_log(self) -> bool {
        self.ordinal() >= Self::Aware.ordinal()
    }
}

impl PartialOrd for AccessMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccessMode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

impl fmt::Display for AccessMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinal_values() {
        assert_eq!(AccessMode::Unrestricted.ordinal(), 0);
        assert_eq!(AccessMode::Aware.ordinal(), 1);
        assert_eq!(AccessMode::Guarded.ordinal(), 2);
        assert_eq!(AccessMode::Enforced.ordinal(), 3);
        assert_eq!(AccessMode::Lockdown.ordinal(), 4);
    }

    #[test]
    fn ordering() {
        assert!(AccessMode::Lockdown > AccessMode::Enforced);
        assert!(AccessMode::Enforced > AccessMode::Guarded);
        assert!(AccessMode::Guarded > AccessMode::Aware);
        assert!(AccessMode::Aware > AccessMode::Unrestricted);
    }

    #[test]
    fn is_enforcement_active() {
        assert!(!AccessMode::Unrestricted.is_enforcement_active());
        assert!(!AccessMode::Aware.is_enforcement_active());
        assert!(!AccessMode::Guarded.is_enforcement_active());
        assert!(AccessMode::Enforced.is_enforcement_active());
        assert!(AccessMode::Lockdown.is_enforcement_active());
    }

    #[test]
    fn allows_cross_boundary() {
        assert!(AccessMode::Unrestricted.allows_cross_boundary());
        assert!(AccessMode::Aware.allows_cross_boundary());
        assert!(AccessMode::Guarded.allows_cross_boundary());
        assert!(!AccessMode::Enforced.allows_cross_boundary());
        assert!(!AccessMode::Lockdown.allows_cross_boundary());
    }

    #[test]
    fn allows_external_calls() {
        assert!(AccessMode::Unrestricted.allows_external_calls());
        assert!(AccessMode::Aware.allows_external_calls());
        assert!(AccessMode::Guarded.allows_external_calls());
        assert!(AccessMode::Enforced.allows_external_calls());
        assert!(!AccessMode::Lockdown.allows_external_calls());
    }

    #[test]
    fn requires_full_audit() {
        assert!(!AccessMode::Unrestricted.requires_full_audit());
        assert!(!AccessMode::Aware.requires_full_audit());
        assert!(!AccessMode::Guarded.requires_full_audit());
        assert!(!AccessMode::Enforced.requires_full_audit());
        assert!(AccessMode::Lockdown.requires_full_audit());
    }

    #[test]
    fn requires_access_log() {
        assert!(!AccessMode::Unrestricted.requires_access_log());
        assert!(AccessMode::Aware.requires_access_log());
        assert!(AccessMode::Guarded.requires_access_log());
        assert!(AccessMode::Enforced.requires_access_log());
        assert!(AccessMode::Lockdown.requires_access_log());
    }

    #[test]
    fn serde_roundtrip() {
        let mode = AccessMode::Enforced;
        let json = serde_json::to_string(&mode).unwrap_or_default();
        let parsed: Result<AccessMode, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
        if let Ok(p) = parsed {
            assert_eq!(p, mode);
        }
    }

    #[test]
    fn display() {
        assert_eq!(AccessMode::Unrestricted.to_string(), "Unrestricted");
        assert_eq!(AccessMode::Lockdown.to_string(), "Lockdown");
    }

    #[test]
    fn label_matches_display() {
        for mode in [
            AccessMode::Unrestricted,
            AccessMode::Aware,
            AccessMode::Guarded,
            AccessMode::Enforced,
            AccessMode::Lockdown,
        ] {
            assert_eq!(mode.label(), mode.to_string());
        }
    }
}
