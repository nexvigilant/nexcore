/// Evidence observed about an entity's trustworthiness.
///
/// Each piece of evidence carries a weight (default 1.0) that determines
/// its impact on the trust score. Higher weights represent stronger signals.
///
/// Tier: T2-P (Causality -> + Quantity N)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evidence {
    /// Positive signal: promise kept, successful interaction, consistent behavior.
    /// The `f64` weight scales the impact (default 1.0).
    Positive(f64),
    /// Negative signal: promise broken, failed interaction, inconsistent behavior.
    /// The `f64` weight scales the impact (default 1.0).
    Negative(f64),
    /// Neutral observation: interaction occurred but no meaningful trust signal.
    Neutral,
}

impl Evidence {
    /// Standard positive evidence with unit weight.
    pub fn positive() -> Self {
        Self::Positive(1.0)
    }

    /// Standard negative evidence with unit weight.
    pub fn negative() -> Self {
        Self::Negative(1.0)
    }

    /// Positive evidence with custom weight.
    /// Weight is clamped to [0.0, +inf).
    pub fn positive_weighted(weight: f64) -> Self {
        Self::Positive(weight.max(0.0))
    }

    /// Negative evidence with custom weight.
    /// Weight is clamped to [0.0, +inf).
    pub fn negative_weighted(weight: f64) -> Self {
        Self::Negative(weight.max(0.0))
    }

    /// Extract the raw weight of this evidence.
    /// Neutral evidence has zero weight.
    pub fn weight(self) -> f64 {
        match self {
            Self::Positive(w) | Self::Negative(w) => w,
            Self::Neutral => 0.0,
        }
    }

    /// Whether this evidence is a positive signal.
    pub fn is_positive(self) -> bool {
        matches!(self, Self::Positive(_))
    }

    /// Whether this evidence is a negative signal.
    pub fn is_negative(self) -> bool {
        matches!(self, Self::Negative(_))
    }
}

impl core::fmt::Display for Evidence {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Positive(w) => write!(f, "+{w:.2}"),
            Self::Negative(w) => write!(f, "-{w:.2}"),
            Self::Neutral => write!(f, "~0.00"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_positive_has_unit_weight() {
        let ev = Evidence::positive();
        assert!((ev.weight() - 1.0).abs() < f64::EPSILON);
        assert!(ev.is_positive());
        assert!(!ev.is_negative());
    }

    #[test]
    fn default_negative_has_unit_weight() {
        let ev = Evidence::negative();
        assert!((ev.weight() - 1.0).abs() < f64::EPSILON);
        assert!(ev.is_negative());
        assert!(!ev.is_positive());
    }

    #[test]
    fn neutral_has_zero_weight() {
        let ev = Evidence::Neutral;
        assert!((ev.weight()).abs() < f64::EPSILON);
        assert!(!ev.is_positive());
        assert!(!ev.is_negative());
    }

    #[test]
    fn negative_input_clamped_to_zero() {
        let ev = Evidence::positive_weighted(-5.0);
        assert!((ev.weight()).abs() < f64::EPSILON);
    }

    #[test]
    fn custom_weight_preserved() {
        let ev = Evidence::negative_weighted(3.5);
        assert!((ev.weight() - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn display_formatting() {
        assert_eq!(format!("{}", Evidence::positive()), "+1.00");
        assert_eq!(format!("{}", Evidence::negative()), "-1.00");
        assert_eq!(format!("{}", Evidence::Neutral), "~0.00");
    }
}
