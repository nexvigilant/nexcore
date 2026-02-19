//! # Classification Level
//!
//! 5-level government-style security classification.
//!
//! ## Primitive Grounding
//! - **Tier**: T1
//! - **Dominant**: ς State (each level is a distinct security state)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Five-level security classification for NexVigilant assets.
///
/// Ordered from least restrictive (Public) to most restrictive (TopSecret).
/// The ordinal encoding (`#[repr(u8)]`) enables safe numeric comparison.
///
/// ## Tier: T1
/// ## Dominant: ς State
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ClassificationLevel {
    /// Freely shareable — no restrictions.
    Public = 0,
    /// NexVigilant team-only — basic access logging.
    Internal = 1,
    /// Need-to-know basis — warn before external sharing.
    Confidential = 2,
    /// Trade secrets, proprietary algorithms — block violations.
    Secret = 3,
    /// PHI, crypto keys, regulatory submissions — full lockdown.
    TopSecret = 4,
}

impl ClassificationLevel {
    /// Returns the numeric ordinal (0–4).
    #[must_use]
    pub fn ordinal(self) -> u8 {
        self as u8
    }

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::Internal => "Internal",
            Self::Confidential => "Confidential",
            Self::Secret => "Secret",
            Self::TopSecret => "Top Secret",
        }
    }

    /// Whether this level is considered restricted (Confidential or above).
    #[must_use]
    pub fn is_restricted(self) -> bool {
        self.ordinal() >= Self::Confidential.ordinal()
    }

    /// Whether this level requires audit logging.
    #[must_use]
    pub fn requires_audit(self) -> bool {
        self.ordinal() >= Self::Internal.ordinal()
    }

    /// Whether this level requires dual authorization for sensitive ops.
    #[must_use]
    pub fn requires_dual_auth(self) -> bool {
        self.ordinal() >= Self::Secret.ordinal()
    }

    /// Whether this level allows external tool access.
    #[must_use]
    pub fn allows_external(self) -> bool {
        self.ordinal() < Self::TopSecret.ordinal()
    }

    /// Returns true if `self` outranks `other` (higher restriction).
    #[must_use]
    pub fn outranks(self, other: Self) -> bool {
        self.ordinal() > other.ordinal()
    }

    /// Parse from string (case-insensitive).
    #[must_use]
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "public" => Some(Self::Public),
            "internal" => Some(Self::Internal),
            "confidential" => Some(Self::Confidential),
            "secret" => Some(Self::Secret),
            "topsecret" | "top_secret" | "top secret" => Some(Self::TopSecret),
            _ => None,
        }
    }
}

impl PartialOrd for ClassificationLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClassificationLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordinal().cmp(&other.ordinal())
    }
}

impl fmt::Display for ClassificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinal_values() {
        assert_eq!(ClassificationLevel::Public.ordinal(), 0);
        assert_eq!(ClassificationLevel::Internal.ordinal(), 1);
        assert_eq!(ClassificationLevel::Confidential.ordinal(), 2);
        assert_eq!(ClassificationLevel::Secret.ordinal(), 3);
        assert_eq!(ClassificationLevel::TopSecret.ordinal(), 4);
    }

    #[test]
    fn ordering() {
        assert!(ClassificationLevel::TopSecret > ClassificationLevel::Secret);
        assert!(ClassificationLevel::Secret > ClassificationLevel::Confidential);
        assert!(ClassificationLevel::Confidential > ClassificationLevel::Internal);
        assert!(ClassificationLevel::Internal > ClassificationLevel::Public);
    }

    #[test]
    fn is_restricted() {
        assert!(!ClassificationLevel::Public.is_restricted());
        assert!(!ClassificationLevel::Internal.is_restricted());
        assert!(ClassificationLevel::Confidential.is_restricted());
        assert!(ClassificationLevel::Secret.is_restricted());
        assert!(ClassificationLevel::TopSecret.is_restricted());
    }

    #[test]
    fn requires_audit() {
        assert!(!ClassificationLevel::Public.requires_audit());
        assert!(ClassificationLevel::Internal.requires_audit());
        assert!(ClassificationLevel::Confidential.requires_audit());
        assert!(ClassificationLevel::Secret.requires_audit());
        assert!(ClassificationLevel::TopSecret.requires_audit());
    }

    #[test]
    fn requires_dual_auth() {
        assert!(!ClassificationLevel::Public.requires_dual_auth());
        assert!(!ClassificationLevel::Internal.requires_dual_auth());
        assert!(!ClassificationLevel::Confidential.requires_dual_auth());
        assert!(ClassificationLevel::Secret.requires_dual_auth());
        assert!(ClassificationLevel::TopSecret.requires_dual_auth());
    }

    #[test]
    fn allows_external() {
        assert!(ClassificationLevel::Public.allows_external());
        assert!(ClassificationLevel::Internal.allows_external());
        assert!(ClassificationLevel::Confidential.allows_external());
        assert!(ClassificationLevel::Secret.allows_external());
        assert!(!ClassificationLevel::TopSecret.allows_external());
    }

    #[test]
    fn outranks() {
        assert!(ClassificationLevel::Secret.outranks(ClassificationLevel::Internal));
        assert!(!ClassificationLevel::Internal.outranks(ClassificationLevel::Secret));
        assert!(!ClassificationLevel::Secret.outranks(ClassificationLevel::Secret));
    }

    #[test]
    fn display() {
        assert_eq!(ClassificationLevel::Public.to_string(), "Public");
        assert_eq!(ClassificationLevel::TopSecret.to_string(), "Top Secret");
    }

    #[test]
    fn serde_roundtrip() {
        let level = ClassificationLevel::Confidential;
        let json = serde_json::to_string(&level).unwrap_or_default();
        let parsed: Result<ClassificationLevel, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok());
        if let Ok(p) = parsed {
            assert_eq!(p, level);
        }
    }

    #[test]
    fn from_str_loose_variants() {
        assert_eq!(
            ClassificationLevel::from_str_loose("public"),
            Some(ClassificationLevel::Public)
        );
        assert_eq!(
            ClassificationLevel::from_str_loose("TopSecret"),
            Some(ClassificationLevel::TopSecret)
        );
        assert_eq!(
            ClassificationLevel::from_str_loose("top secret"),
            Some(ClassificationLevel::TopSecret)
        );
        assert_eq!(ClassificationLevel::from_str_loose("unknown"), None);
    }

    #[test]
    fn label_matches_display() {
        for level in [
            ClassificationLevel::Public,
            ClassificationLevel::Internal,
            ClassificationLevel::Confidential,
            ClassificationLevel::Secret,
            ClassificationLevel::TopSecret,
        ] {
            assert_eq!(level.label(), level.to_string());
        }
    }
}
