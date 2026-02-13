/// Discrete trust classification derived from continuous score.
///
/// Five tiers partition the [0.0, 1.0] score range into actionable
/// permission boundaries. Each level maps to a policy decision
/// (e.g., "require approval" vs "auto-approve").
///
/// Tier: T2-P (Boundary d + Comparison k)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustLevel {
    /// Score [0.0, 0.2): Entity has demonstrated untrustworthiness.
    Untrusted,
    /// Score [0.2, 0.4): Insufficient positive evidence, some concern.
    Suspicious,
    /// Score [0.4, 0.6): Default state, no strong signal either way.
    Neutral,
    /// Score [0.6, 0.8): Consistent positive track record.
    Trusted,
    /// Score [0.8, 1.0]: Extensive positive history, rare violations.
    HighlyTrusted,
}

impl TrustLevel {
    /// Classify a continuous trust score into a discrete level.
    ///
    /// Scores outside [0.0, 1.0] are clamped before classification.
    pub fn from_score(score: f64) -> Self {
        let clamped = score.clamp(0.0, 1.0);
        if clamped < 0.2 {
            Self::Untrusted
        } else if clamped < 0.4 {
            Self::Suspicious
        } else if clamped < 0.6 {
            Self::Neutral
        } else if clamped < 0.8 {
            Self::Trusted
        } else {
            Self::HighlyTrusted
        }
    }

    /// Lower bound of this trust level's score range.
    pub fn lower_bound(self) -> f64 {
        match self {
            Self::Untrusted => 0.0,
            Self::Suspicious => 0.2,
            Self::Neutral => 0.4,
            Self::Trusted => 0.6,
            Self::HighlyTrusted => 0.8,
        }
    }

    /// Upper bound of this trust level's score range (exclusive, except HighlyTrusted).
    pub fn upper_bound(self) -> f64 {
        match self {
            Self::Untrusted => 0.2,
            Self::Suspicious => 0.4,
            Self::Neutral => 0.6,
            Self::Trusted => 0.8,
            Self::HighlyTrusted => 1.0,
        }
    }

    /// Human-readable label for this trust level.
    pub fn label(self) -> &'static str {
        match self {
            Self::Untrusted => "Untrusted",
            Self::Suspicious => "Suspicious",
            Self::Neutral => "Neutral",
            Self::Trusted => "Trusted",
            Self::HighlyTrusted => "Highly Trusted",
        }
    }

    /// Whether this level grants baseline trust (Trusted or above).
    pub fn is_trusted(self) -> bool {
        matches!(self, Self::Trusted | Self::HighlyTrusted)
    }

    /// Whether this level indicates active distrust (Suspicious or below).
    pub fn is_distrusted(self) -> bool {
        matches!(self, Self::Untrusted | Self::Suspicious)
    }

    /// Classify with custom thresholds.
    ///
    /// Allows domain-specific threshold configuration. Safety-critical
    /// domains may set higher thresholds for "Trusted."
    pub fn from_score_with_thresholds(score: f64, thresholds: &LevelThresholds) -> Self {
        let clamped = score.clamp(0.0, 1.0);
        if clamped < thresholds.suspicious {
            Self::Untrusted
        } else if clamped < thresholds.neutral {
            Self::Suspicious
        } else if clamped < thresholds.trusted {
            Self::Neutral
        } else if clamped < thresholds.highly_trusted {
            Self::Trusted
        } else {
            Self::HighlyTrusted
        }
    }
}

/// Configurable thresholds for trust level classification.
///
/// Boundaries must be monotonically increasing in (0, 1).
/// Fixes: Gap #9 (Configurable Level Thresholds).
///
/// Tier: T2-P (Boundary d)
#[derive(Debug, Clone, Copy)]
pub struct LevelThresholds {
    /// Score below this is Untrusted. Default: 0.2
    pub suspicious: f64,
    /// Score below this is Suspicious. Default: 0.4
    pub neutral: f64,
    /// Score below this is Neutral. Default: 0.6
    pub trusted: f64,
    /// Score below this is Trusted; above is HighlyTrusted. Default: 0.8
    pub highly_trusted: f64,
}

impl LevelThresholds {
    /// Strict thresholds for safety-critical domains.
    /// Requires higher evidence for "Trusted" classification.
    pub fn strict() -> Self {
        Self {
            suspicious: 0.3,
            neutral: 0.5,
            trusted: 0.7,
            highly_trusted: 0.9,
        }
    }

    /// Lenient thresholds for low-stakes domains.
    pub fn lenient() -> Self {
        Self {
            suspicious: 0.15,
            neutral: 0.30,
            trusted: 0.50,
            highly_trusted: 0.70,
        }
    }

    /// Whether the thresholds are valid (monotonically increasing in (0, 1)).
    pub fn is_valid(&self) -> bool {
        0.0 < self.suspicious
            && self.suspicious < self.neutral
            && self.neutral < self.trusted
            && self.trusted < self.highly_trusted
            && self.highly_trusted < 1.0
    }
}

impl Default for LevelThresholds {
    fn default() -> Self {
        Self {
            suspicious: 0.2,
            neutral: 0.4,
            trusted: 0.6,
            highly_trusted: 0.8,
        }
    }
}

impl core::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_classification() {
        assert_eq!(TrustLevel::from_score(0.0), TrustLevel::Untrusted);
        assert_eq!(TrustLevel::from_score(0.19), TrustLevel::Untrusted);
        assert_eq!(TrustLevel::from_score(0.2), TrustLevel::Suspicious);
        assert_eq!(TrustLevel::from_score(0.39), TrustLevel::Suspicious);
        assert_eq!(TrustLevel::from_score(0.4), TrustLevel::Neutral);
        assert_eq!(TrustLevel::from_score(0.5), TrustLevel::Neutral);
        assert_eq!(TrustLevel::from_score(0.6), TrustLevel::Trusted);
        assert_eq!(TrustLevel::from_score(0.79), TrustLevel::Trusted);
        assert_eq!(TrustLevel::from_score(0.8), TrustLevel::HighlyTrusted);
        assert_eq!(TrustLevel::from_score(1.0), TrustLevel::HighlyTrusted);
    }

    #[test]
    fn out_of_range_clamped() {
        assert_eq!(TrustLevel::from_score(-1.0), TrustLevel::Untrusted);
        assert_eq!(TrustLevel::from_score(2.0), TrustLevel::HighlyTrusted);
    }

    #[test]
    fn ordering() {
        assert!(TrustLevel::Untrusted < TrustLevel::Suspicious);
        assert!(TrustLevel::Suspicious < TrustLevel::Neutral);
        assert!(TrustLevel::Neutral < TrustLevel::Trusted);
        assert!(TrustLevel::Trusted < TrustLevel::HighlyTrusted);
    }

    #[test]
    fn is_trusted_and_distrusted() {
        assert!(!TrustLevel::Untrusted.is_trusted());
        assert!(TrustLevel::Untrusted.is_distrusted());
        assert!(!TrustLevel::Neutral.is_trusted());
        assert!(!TrustLevel::Neutral.is_distrusted());
        assert!(TrustLevel::Trusted.is_trusted());
        assert!(!TrustLevel::Trusted.is_distrusted());
        assert!(TrustLevel::HighlyTrusted.is_trusted());
    }

    #[test]
    fn bounds_cover_full_range() {
        let levels = [
            TrustLevel::Untrusted,
            TrustLevel::Suspicious,
            TrustLevel::Neutral,
            TrustLevel::Trusted,
            TrustLevel::HighlyTrusted,
        ];
        assert!((levels[0].lower_bound()).abs() < f64::EPSILON);
        assert!((levels[4].upper_bound() - 1.0).abs() < f64::EPSILON);
        for pair in levels.windows(2) {
            assert!(
                (pair[0].upper_bound() - pair[1].lower_bound()).abs() < f64::EPSILON,
                "Gap between {} and {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn display_labels() {
        assert_eq!(format!("{}", TrustLevel::Untrusted), "Untrusted");
        assert_eq!(format!("{}", TrustLevel::HighlyTrusted), "Highly Trusted");
    }
}
