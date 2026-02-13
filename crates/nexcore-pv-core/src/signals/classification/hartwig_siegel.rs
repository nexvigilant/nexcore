//! Hartwig-Siegel Adverse Drug Reaction Severity Scale
//!
//! Standardized classification system for ADR severity assessment used in
//! pharmacovigilance for signal prioritization and clinical impact evaluation.
//!
//! # Severity Levels
//!
//! | Level | Category | Description |
//! |-------|----------|-------------|
//! | 1 | Mild | No change in treatment, no antidote required |
//! | 2 | Mild | Drug held, discontinued, or changed; no antidote |
//! | 3 | Moderate | Drug held; antidote or other treatment required |
//! | 4 | Moderate | Longer hospital stay or hospital admission required |
//! | 5 | Severe | ICU admission required |
//! | 6 | Severe | Permanent harm |
//! | 7 | Lethal | Death (directly or indirectly) |
//!
//! # Clinical Use
//!
//! - **Signal prioritization**: Severe ADRs get priority investigation
//! - **Risk-benefit assessment**: Weighting ADR impact
//! - **Regulatory reporting**: E2B(R3) severity coding
//!
//! # References
//!
//! - Hartwig SC, Siegel J, Schneider PJ (1992). "Preventability and severity assessment
//!   in reporting adverse drug reactions." American Journal of Hospital Pharmacy
//!   49(9):2229-2232. PMID: [1524068](https://pubmed.ncbi.nlm.nih.gov/1524068/)
//!
//! - Edwards IR, Aronson JK (2000). "Adverse drug reactions: definitions, diagnosis,
//!   and management." The Lancet 356(9237):1255-1259.
//!   DOI: [10.1016/S0140-6736(00)02799-9](https://doi.org/10.1016/S0140-6736(00)02799-9)
//!
//! - Naranjo CA, Busto U, Sellers EM, et al. (1981). "A method for estimating the
//!   probability of adverse drug reactions." Clinical Pharmacology & Therapeutics
//!   30(2):239-245. DOI: [10.1038/clpt.1981.154](https://doi.org/10.1038/clpt.1981.154)

use serde::{Deserialize, Serialize};

/// Hartwig-Siegel severity level (1-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SeverityLevel {
    /// Level 1: Mild - No change in treatment required
    Mild1 = 1,
    /// Level 2: Mild - Drug change required, no antidote
    Mild2 = 2,
    /// Level 3: Moderate - Antidote or treatment required
    Moderate3 = 3,
    /// Level 4: Moderate - Prolonged hospitalization
    Moderate4 = 4,
    /// Level 5: Severe - ICU admission
    Severe5 = 5,
    /// Level 6: Severe - Permanent harm
    Severe6 = 6,
    /// Level 7: Lethal - Death
    Lethal7 = 7,
}

impl SeverityLevel {
    /// Get the numeric level (1-7).
    #[must_use]
    pub const fn level(&self) -> u8 {
        *self as u8
    }

    /// Get the broad category (Mild, Moderate, Severe, Lethal).
    #[must_use]
    pub const fn category(&self) -> SeverityCategory {
        match self {
            Self::Mild1 | Self::Mild2 => SeverityCategory::Mild,
            Self::Moderate3 | Self::Moderate4 => SeverityCategory::Moderate,
            Self::Severe5 | Self::Severe6 => SeverityCategory::Severe,
            Self::Lethal7 => SeverityCategory::Lethal,
        }
    }

    /// Get human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Mild1 => "No change in treatment required",
            Self::Mild2 => "Drug held, discontinued, or changed; no antidote required",
            Self::Moderate3 => "Drug held; antidote or other treatment required",
            Self::Moderate4 => "Longer hospital stay or hospital admission required",
            Self::Severe5 => "ICU admission required",
            Self::Severe6 => "Permanent harm caused",
            Self::Lethal7 => "Death (directly or indirectly contributed)",
        }
    }

    /// Get clinical action required.
    #[must_use]
    pub const fn clinical_action(&self) -> &'static str {
        match self {
            Self::Mild1 => "Monitor; continue treatment if benefit outweighs risk",
            Self::Mild2 => "Consider alternative therapy; document ADR",
            Self::Moderate3 => "Discontinue drug; provide supportive treatment",
            Self::Moderate4 => "Hospitalize/extend stay; intensive monitoring",
            Self::Severe5 => "ICU care; life support measures as needed",
            Self::Severe6 => "Permanent therapy modifications; rehabilitation",
            Self::Lethal7 => "Document for regulatory reporting; root cause analysis",
        }
    }

    /// Whether this requires regulatory expedited reporting (serious ADR).
    ///
    /// ICH E2A defines serious as: death, life-threatening, hospitalization,
    /// disability, congenital anomaly, or medically important.
    #[must_use]
    pub const fn is_serious(&self) -> bool {
        matches!(
            self,
            Self::Moderate4 | Self::Severe5 | Self::Severe6 | Self::Lethal7
        )
    }

    /// Parse from numeric level.
    #[must_use]
    pub fn from_level(level: u8) -> Option<Self> {
        match level {
            1 => Some(Self::Mild1),
            2 => Some(Self::Mild2),
            3 => Some(Self::Moderate3),
            4 => Some(Self::Moderate4),
            5 => Some(Self::Severe5),
            6 => Some(Self::Severe6),
            7 => Some(Self::Lethal7),
            _ => None,
        }
    }
}

/// Broad severity category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SeverityCategory {
    /// Levels 1-2: Self-limiting, minimal intervention
    Mild,
    /// Levels 3-4: Requires treatment or hospitalization
    Moderate,
    /// Levels 5-6: Life-threatening or permanent harm
    Severe,
    /// Level 7: Fatal
    Lethal,
}

impl SeverityCategory {
    /// Get description for the category.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Mild => "Self-limiting ADR with minimal intervention needed",
            Self::Moderate => "ADR requiring treatment or hospitalization",
            Self::Severe => "Life-threatening or causing permanent harm",
            Self::Lethal => "Fatal outcome",
        }
    }

    /// Get signal priority weight (higher = more urgent).
    #[must_use]
    pub const fn priority_weight(&self) -> u8 {
        match self {
            Self::Mild => 1,
            Self::Moderate => 2,
            Self::Severe => 4,
            Self::Lethal => 8,
        }
    }
}

/// Input criteria for severity assessment.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SeverityCriteria {
    /// Treatment was changed (drug held, discontinued, or switched)
    pub treatment_changed: bool,
    /// Antidote or specific treatment was required
    pub antidote_required: bool,
    /// Hospital admission was required (or stay prolonged)
    pub hospitalization_required: bool,
    /// ICU admission was required
    pub icu_required: bool,
    /// Permanent disability or harm resulted
    pub permanent_harm: bool,
    /// Patient died
    pub death: bool,
}

impl SeverityCriteria {
    /// Create new criteria with all fields false.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            treatment_changed: false,
            antidote_required: false,
            hospitalization_required: false,
            icu_required: false,
            permanent_harm: false,
            death: false,
        }
    }

    /// Builder: set treatment changed.
    #[must_use]
    pub const fn with_treatment_change(mut self) -> Self {
        self.treatment_changed = true;
        self
    }

    /// Builder: set antidote required.
    #[must_use]
    pub const fn with_antidote(mut self) -> Self {
        self.antidote_required = true;
        self
    }

    /// Builder: set hospitalization.
    #[must_use]
    pub const fn with_hospitalization(mut self) -> Self {
        self.hospitalization_required = true;
        self
    }

    /// Builder: set ICU.
    #[must_use]
    pub const fn with_icu(mut self) -> Self {
        self.icu_required = true;
        self
    }

    /// Builder: set permanent harm.
    #[must_use]
    pub const fn with_permanent_harm(mut self) -> Self {
        self.permanent_harm = true;
        self
    }

    /// Builder: set death.
    #[must_use]
    pub const fn with_death(mut self) -> Self {
        self.death = true;
        self
    }
}

/// Assess severity based on clinical criteria.
///
/// # Algorithm
///
/// Severity is assessed hierarchically from most to least severe:
/// 1. Death → Level 7
/// 2. Permanent harm → Level 6
/// 3. ICU required → Level 5
/// 4. Hospitalization → Level 4
/// 5. Antidote required → Level 3
/// 6. Treatment changed → Level 2
/// 7. None of the above → Level 1
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::pv::signals::classification::hartwig_siegel::{assess_severity, SeverityCriteria};
///
/// let criteria = SeverityCriteria::new()
///     .with_treatment_change()
///     .with_hospitalization();
///
/// let severity = assess_severity(&criteria);
/// assert!(severity.is_serious()); // Hospitalization makes it serious
/// ```
#[must_use]
pub fn assess_severity(criteria: &SeverityCriteria) -> SeverityLevel {
    if criteria.death {
        SeverityLevel::Lethal7
    } else if criteria.permanent_harm {
        SeverityLevel::Severe6
    } else if criteria.icu_required {
        SeverityLevel::Severe5
    } else if criteria.hospitalization_required {
        SeverityLevel::Moderate4
    } else if criteria.antidote_required {
        SeverityLevel::Moderate3
    } else if criteria.treatment_changed {
        SeverityLevel::Mild2
    } else {
        SeverityLevel::Mild1
    }
}

/// Result of severity assessment with additional context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeverityAssessment {
    /// Assigned severity level
    pub level: SeverityLevel,
    /// Broad category
    pub category: SeverityCategory,
    /// Whether this qualifies as "serious" per ICH E2A
    pub is_serious: bool,
    /// Signal priority weight
    pub priority_weight: u8,
    /// Original criteria used for assessment
    pub criteria: SeverityCriteria,
}

/// Perform full severity assessment.
///
/// Returns detailed assessment including seriousness determination
/// and priority weighting for signal triage.
#[must_use]
pub fn full_assessment(criteria: &SeverityCriteria) -> SeverityAssessment {
    let level = assess_severity(criteria);

    SeverityAssessment {
        level,
        category: level.category(),
        is_serious: level.is_serious(),
        priority_weight: level.category().priority_weight(),
        criteria: criteria.clone(),
    }
}

/// Calculate weighted severity score for a batch of ADRs.
///
/// Useful for signal prioritization based on severity distribution.
///
/// # Returns
///
/// Tuple of (weighted_score, max_severity, serious_count)
#[must_use]
pub fn batch_severity_score(levels: &[SeverityLevel]) -> (f64, Option<SeverityLevel>, usize) {
    if levels.is_empty() {
        return (0.0, None, 0);
    }

    let mut total_weight = 0u32;
    let mut max_level = levels[0];
    let mut serious_count = 0usize;

    for &level in levels {
        total_weight += u32::from(level.category().priority_weight());
        if level > max_level {
            max_level = level;
        }
        if level.is_serious() {
            serious_count += 1;
        }
    }

    let weighted_score = f64::from(total_weight) / levels.len() as f64;

    (weighted_score, Some(max_level), serious_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(SeverityLevel::Mild1 < SeverityLevel::Mild2);
        assert!(SeverityLevel::Mild2 < SeverityLevel::Moderate3);
        assert!(SeverityLevel::Severe6 < SeverityLevel::Lethal7);
    }

    #[test]
    fn test_assess_death() {
        let criteria = SeverityCriteria::new().with_death();
        assert_eq!(assess_severity(&criteria), SeverityLevel::Lethal7);
    }

    #[test]
    fn test_assess_icu() {
        let criteria = SeverityCriteria::new().with_icu();
        assert_eq!(assess_severity(&criteria), SeverityLevel::Severe5);
    }

    #[test]
    fn test_assess_hospitalization() {
        let criteria = SeverityCriteria::new().with_hospitalization();
        assert_eq!(assess_severity(&criteria), SeverityLevel::Moderate4);
    }

    #[test]
    fn test_assess_antidote() {
        let criteria = SeverityCriteria::new().with_antidote();
        assert_eq!(assess_severity(&criteria), SeverityLevel::Moderate3);
    }

    #[test]
    fn test_assess_treatment_change() {
        let criteria = SeverityCriteria::new().with_treatment_change();
        assert_eq!(assess_severity(&criteria), SeverityLevel::Mild2);
    }

    #[test]
    fn test_assess_no_intervention() {
        let criteria = SeverityCriteria::new();
        assert_eq!(assess_severity(&criteria), SeverityLevel::Mild1);
    }

    #[test]
    fn test_hierarchy_death_overrides_all() {
        let criteria = SeverityCriteria {
            treatment_changed: true,
            antidote_required: true,
            hospitalization_required: true,
            icu_required: true,
            permanent_harm: true,
            death: true,
        };
        assert_eq!(assess_severity(&criteria), SeverityLevel::Lethal7);
    }

    #[test]
    fn test_is_serious() {
        assert!(!SeverityLevel::Mild1.is_serious());
        assert!(!SeverityLevel::Mild2.is_serious());
        assert!(!SeverityLevel::Moderate3.is_serious());
        assert!(SeverityLevel::Moderate4.is_serious()); // Hospitalization
        assert!(SeverityLevel::Severe5.is_serious());
        assert!(SeverityLevel::Severe6.is_serious());
        assert!(SeverityLevel::Lethal7.is_serious());
    }

    #[test]
    fn test_category() {
        assert_eq!(SeverityLevel::Mild1.category(), SeverityCategory::Mild);
        assert_eq!(
            SeverityLevel::Moderate4.category(),
            SeverityCategory::Moderate
        );
        assert_eq!(SeverityLevel::Severe5.category(), SeverityCategory::Severe);
        assert_eq!(SeverityLevel::Lethal7.category(), SeverityCategory::Lethal);
    }

    #[test]
    fn test_from_level() {
        assert_eq!(SeverityLevel::from_level(1), Some(SeverityLevel::Mild1));
        assert_eq!(SeverityLevel::from_level(7), Some(SeverityLevel::Lethal7));
        assert_eq!(SeverityLevel::from_level(0), None);
        assert_eq!(SeverityLevel::from_level(8), None);
    }

    #[test]
    fn test_full_assessment() {
        let criteria = SeverityCriteria::new().with_hospitalization();
        let assessment = full_assessment(&criteria);

        assert_eq!(assessment.level, SeverityLevel::Moderate4);
        assert_eq!(assessment.category, SeverityCategory::Moderate);
        assert!(assessment.is_serious);
        assert_eq!(assessment.priority_weight, 2);
    }

    #[test]
    fn test_batch_score() {
        let levels = vec![
            SeverityLevel::Mild1,
            SeverityLevel::Mild2,
            SeverityLevel::Moderate4,
            SeverityLevel::Severe6,
        ];

        let (score, max, serious) = batch_severity_score(&levels);

        assert!(score > 1.0); // Weighted average > 1
        assert_eq!(max, Some(SeverityLevel::Severe6));
        assert_eq!(serious, 2); // Moderate4 and Severe6
    }

    #[test]
    fn test_batch_score_empty() {
        let (score, max, serious) = batch_severity_score(&[]);
        assert_eq!(score, 0.0);
        assert_eq!(max, None);
        assert_eq!(serious, 0);
    }
}
