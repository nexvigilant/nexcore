//! # Case Quality Measurement Primitives
//!
//! Formalized quality metrics for Individual Case Safety Reports (ICSRs).
//! These replace ad-hoc `f64` quality fields with typed, grounded measures.
//!
//! ## Type Inventory
//!
//! | Type | What It Measures | T1 Grounding | Priority |
//! |------|------------------|--------------|----------|
//! | `CaseCompletenessScore` | ICSR field coverage | N (Quantity) | P3 |
//! | `DuplicateProbability` | P(case is duplicate) | κ (Comparison) | P3 |
//! | `NarrativeQualityScore` | Free-text quality | N (Quantity) | P3 |
//! | `CodingAccuracy` | MedDRA coding correctness | κ (Comparison) | P3 |
//!
//! All types are **T2-P** (single T1 dominant).

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 1. CaseCompletenessScore — ICSR field coverage
//    T1: N (Quantity)
//    Priority: P3 (Data Quality)
// ============================================================================

/// Proportion of required/expected ICSR fields that are populated.
///
/// # Lex Primitiva
/// - **N (Quantity)** dominant: numeric coverage score
///
/// # ICH E2B Fields
/// Standard ICSR has ~120 data elements. Critical fields:
/// - Patient demographics (age, sex, weight)
/// - Reporter information (qualification, country)
/// - Drug information (name, dose, indication, dates)
/// - Reaction/event (MedDRA coded, onset date, outcome)
/// - Seriousness criteria
///
/// # Scoring
/// - 1.0: All fields populated
/// - 0.8+: Good quality — suitable for automated analysis
/// - 0.5–0.8: Moderate — may need manual review
/// - < 0.5: Poor — unreliable for signal detection
///
/// Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CaseCompletenessScore {
    /// Overall score [0.0, 1.0]
    score: f64,
    /// Number of populated fields
    populated: u32,
    /// Total expected fields
    total: u32,
    /// Critical fields populated (out of critical total)
    critical_populated: u32,
    /// Total critical fields
    critical_total: u32,
}

impl CaseCompletenessScore {
    /// Standard number of critical ICSR fields per ICH E2B.
    pub const CRITICAL_FIELD_COUNT: u32 = 12;

    /// Compute completeness from field counts.
    ///
    /// Returns `None` if total fields is zero.
    #[must_use]
    pub fn new(
        populated: u32,
        total: u32,
        critical_populated: u32,
        critical_total: u32,
    ) -> Option<Self> {
        if total == 0 || critical_total == 0 {
            return None;
        }

        let score = populated as f64 / total as f64;

        Some(Self {
            score,
            populated,
            total,
            critical_populated,
            critical_total,
        })
    }

    /// Quick computation from just populated/total.
    #[must_use]
    pub fn quick(populated: u32, total: u32) -> Option<Self> {
        Self::new(
            populated,
            total,
            populated.min(Self::CRITICAL_FIELD_COUNT),
            Self::CRITICAL_FIELD_COUNT,
        )
    }

    /// Overall score [0.0, 1.0].
    #[inline]
    #[must_use]
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Critical field completion rate [0.0, 1.0].
    #[must_use]
    pub fn critical_score(&self) -> f64 {
        if self.critical_total == 0 {
            return 0.0;
        }
        self.critical_populated as f64 / self.critical_total as f64
    }

    /// Populated field count.
    #[inline]
    #[must_use]
    pub fn populated(&self) -> u32 {
        self.populated
    }

    /// Total expected fields.
    #[inline]
    #[must_use]
    pub fn total(&self) -> u32 {
        self.total
    }

    /// Is the case good quality for automated analysis?
    #[inline]
    #[must_use]
    pub fn is_good(&self) -> bool {
        self.score >= 0.8 && self.critical_score() >= 0.9
    }

    /// Is the case poor quality?
    #[inline]
    #[must_use]
    pub fn is_poor(&self) -> bool {
        self.score < 0.5 || self.critical_score() < 0.5
    }

    /// Quality tier label.
    #[must_use]
    pub fn quality_tier(&self) -> &'static str {
        if self.is_good() {
            "good"
        } else if self.score >= 0.5 {
            "moderate"
        } else {
            "poor"
        }
    }
}

impl GroundsTo for CaseCompletenessScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

impl fmt::Display for CaseCompletenessScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Completeness={:.0}% ({}/{}, critical={:.0}%, {})",
            self.score * 100.0,
            self.populated,
            self.total,
            self.critical_score() * 100.0,
            self.quality_tier()
        )
    }
}

// ============================================================================
// 2. DuplicateProbability — P(case is duplicate)
//    T1: κ (Comparison)
//    Priority: P3 (Data Quality)
// ============================================================================

/// Probability that an ICSR is a duplicate of an existing case.
/// Used by deduplication algorithms (Jaccard, edit distance, etc.).
///
/// # Lex Primitiva
/// - **κ (Comparison)** dominant: comparing similarity between cases
///
/// # Thresholds
/// - P > 0.9: Almost certainly duplicate → auto-merge
/// - P 0.7–0.9: Probable duplicate → flag for review
/// - P 0.3–0.7: Uncertain → manual review
/// - P < 0.3: Likely unique
///
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DuplicateProbability {
    /// Probability [0.0, 1.0]
    probability: f64,
    /// Similarity score from comparison algorithm
    similarity: f64,
    /// Algorithm used for comparison
    method: DuplicateMethod,
}

/// Method used for duplicate detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DuplicateMethod {
    /// Jaccard similarity on narrative tokens
    Jaccard,
    /// Edit distance on structured fields
    EditDistance,
    /// Composite scoring across multiple methods
    Composite,
    /// Machine learning classifier
    MlClassifier,
}

impl DuplicateProbability {
    /// Create from probability and method.
    ///
    /// Returns `None` if probability is outside [0.0, 1.0].
    #[must_use]
    pub fn new(probability: f64, similarity: f64, method: DuplicateMethod) -> Option<Self> {
        if !(0.0..=1.0).contains(&probability) {
            return None;
        }

        Some(Self {
            probability,
            similarity,
            method,
        })
    }

    /// Quick: derive probability from Jaccard similarity score.
    #[must_use]
    pub fn from_jaccard(similarity: f64) -> Option<Self> {
        if !(0.0..=1.0).contains(&similarity) {
            return None;
        }
        Self::new(similarity, similarity, DuplicateMethod::Jaccard)
    }

    /// Get the probability.
    #[inline]
    #[must_use]
    pub fn probability(&self) -> f64 {
        self.probability
    }

    /// Get the similarity score.
    #[inline]
    #[must_use]
    pub fn similarity(&self) -> f64 {
        self.similarity
    }

    /// Get the detection method.
    #[inline]
    #[must_use]
    pub fn method(&self) -> DuplicateMethod {
        self.method
    }

    /// Should auto-merge (P > 0.9)?
    #[inline]
    #[must_use]
    pub fn should_auto_merge(&self) -> bool {
        self.probability > 0.9
    }

    /// Should flag for review (P 0.7–0.9)?
    #[inline]
    #[must_use]
    pub fn should_flag(&self) -> bool {
        self.probability > 0.7 && self.probability <= 0.9
    }

    /// Is likely unique (P < 0.3)?
    #[inline]
    #[must_use]
    pub fn is_likely_unique(&self) -> bool {
        self.probability < 0.3
    }

    /// Action recommendation.
    #[must_use]
    pub fn recommended_action(&self) -> &'static str {
        if self.should_auto_merge() {
            "auto_merge"
        } else if self.should_flag() {
            "flag_for_review"
        } else if self.probability > 0.3 {
            "manual_review"
        } else {
            "accept_as_unique"
        }
    }
}

impl GroundsTo for DuplicateProbability {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for DuplicateProbability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DupP={:.1}% ({:?}, action={})",
            self.probability * 100.0,
            self.method,
            self.recommended_action()
        )
    }
}

// ============================================================================
// 3. NarrativeQualityScore — Free-text quality assessment
//    T1: N (Quantity)
//    Priority: P3 (Data Quality)
// ============================================================================

/// Quality assessment of ICSR narrative/free-text fields.
/// Measures information density, clinical relevance, and structure.
///
/// # Lex Primitiva
/// - **N (Quantity)** dominant: numeric quality score
///
/// # Scoring Dimensions
/// - Length adequacy (too short = insufficient detail)
/// - Clinical term density (medical terminology frequency)
/// - Temporal information (dates, durations present)
/// - Causal language (suspected, possible, probable)
///
/// Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NarrativeQualityScore {
    /// Composite score [0.0, 1.0]
    score: f64,
    /// Word count of narrative
    word_count: u32,
    /// Clinical term density [0.0, 1.0]
    clinical_density: f64,
    /// Has temporal information
    has_temporal: bool,
    /// Has causal language
    has_causal: bool,
}

impl NarrativeQualityScore {
    /// Compute quality from narrative characteristics.
    #[must_use]
    pub fn new(
        word_count: u32,
        clinical_density: f64,
        has_temporal: bool,
        has_causal: bool,
    ) -> Self {
        // Length score: 0 at 0 words, 1.0 at 100+ words
        let length_score = (word_count as f64 / 100.0).min(1.0);

        // Clinical density score (capped at 0.5 = 50% medical terms)
        let density_score = (clinical_density / 0.5).min(1.0);

        // Temporal bonus
        let temporal_score = if has_temporal { 1.0 } else { 0.0 };

        // Causal bonus
        let causal_score = if has_causal { 1.0 } else { 0.0 };

        // Weighted composite
        let score = (length_score * 0.30
            + density_score * 0.30
            + temporal_score * 0.20
            + causal_score * 0.20)
            .clamp(0.0, 1.0);

        Self {
            score,
            word_count,
            clinical_density,
            has_temporal,
            has_causal,
        }
    }

    /// Get composite score.
    #[inline]
    #[must_use]
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Word count.
    #[inline]
    #[must_use]
    pub fn word_count(&self) -> u32 {
        self.word_count
    }

    /// Clinical term density.
    #[inline]
    #[must_use]
    pub fn clinical_density(&self) -> f64 {
        self.clinical_density
    }

    /// Is the narrative adequate for causality assessment?
    #[inline]
    #[must_use]
    pub fn is_adequate(&self) -> bool {
        self.score >= 0.6 && self.has_temporal
    }

    /// Is the narrative too sparse?
    #[inline]
    #[must_use]
    pub fn is_sparse(&self) -> bool {
        self.word_count < 20 || self.score < 0.3
    }
}

impl GroundsTo for NarrativeQualityScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

impl fmt::Display for NarrativeQualityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NarrativeQ={:.0}% ({}w, clin={:.0}%, temp={}, causal={})",
            self.score * 100.0,
            self.word_count,
            self.clinical_density * 100.0,
            self.has_temporal,
            self.has_causal
        )
    }
}

// ============================================================================
// 4. CodingAccuracy — MedDRA coding correctness
//    T1: κ (Comparison)
//    Priority: P3 (Data Quality)
// ============================================================================

/// Accuracy of MedDRA coding applied to adverse event terms.
/// Compares assigned codes against reference/validated coding.
///
/// # Lex Primitiva
/// - **κ (Comparison)** dominant: matching assigned vs reference codes
///
/// # MedDRA Levels
/// SOC → HLGT → HLT → PT → LLT (5 levels, most-general to most-specific)
///
/// # Scoring
/// - Exact PT match: 1.0
/// - Same HLT: 0.8
/// - Same HLGT: 0.6
/// - Same SOC: 0.4
/// - Wrong SOC: 0.0
///
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CodingAccuracy {
    /// Accuracy score [0.0, 1.0]
    score: f64,
    /// MedDRA hierarchy level of match
    match_level: MedDRAMatchLevel,
    /// Total terms evaluated
    terms_evaluated: u32,
    /// Exact matches at PT level
    exact_matches: u32,
}

/// Level at which MedDRA coding matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MedDRAMatchLevel {
    /// Exact preferred term match
    ExactPT,
    /// Same High Level Term
    SameHLT,
    /// Same High Level Group Term
    SameHLGT,
    /// Same System Organ Class
    SameSOC,
    /// No match at any level
    NoMatch,
}

impl MedDRAMatchLevel {
    /// Score for this match level.
    #[must_use]
    pub const fn score(self) -> f64 {
        match self {
            Self::ExactPT => 1.0,
            Self::SameHLT => 0.8,
            Self::SameHLGT => 0.6,
            Self::SameSOC => 0.4,
            Self::NoMatch => 0.0,
        }
    }
}

impl fmt::Display for MedDRAMatchLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExactPT => write!(f, "ExactPT"),
            Self::SameHLT => write!(f, "SameHLT"),
            Self::SameHLGT => write!(f, "SameHLGT"),
            Self::SameSOC => write!(f, "SameSOC"),
            Self::NoMatch => write!(f, "NoMatch"),
        }
    }
}

impl CodingAccuracy {
    /// Compute aggregate accuracy from individual match levels.
    #[must_use]
    pub fn from_matches(matches: &[MedDRAMatchLevel]) -> Option<Self> {
        if matches.is_empty() {
            return None;
        }

        let total = matches.len() as u32;
        let sum: f64 = matches.iter().map(|m| m.score()).sum();
        let score = sum / matches.len() as f64;
        let exact = matches
            .iter()
            .filter(|m| **m == MedDRAMatchLevel::ExactPT)
            .count() as u32;

        // Best match level achieved
        let best = matches
            .iter()
            .min_by(|a, b| {
                // ExactPT is "best" (lowest ordinal)
                let a_ord = match a {
                    MedDRAMatchLevel::ExactPT => 0,
                    MedDRAMatchLevel::SameHLT => 1,
                    MedDRAMatchLevel::SameHLGT => 2,
                    MedDRAMatchLevel::SameSOC => 3,
                    MedDRAMatchLevel::NoMatch => 4,
                };
                let b_ord = match b {
                    MedDRAMatchLevel::ExactPT => 0,
                    MedDRAMatchLevel::SameHLT => 1,
                    MedDRAMatchLevel::SameHLGT => 2,
                    MedDRAMatchLevel::SameSOC => 3,
                    MedDRAMatchLevel::NoMatch => 4,
                };
                a_ord.cmp(&b_ord)
            })
            .copied()
            .unwrap_or(MedDRAMatchLevel::NoMatch);

        Some(Self {
            score,
            match_level: best,
            terms_evaluated: total,
            exact_matches: exact,
        })
    }

    /// Create from single match.
    #[must_use]
    pub fn single(level: MedDRAMatchLevel) -> Self {
        Self {
            score: level.score(),
            match_level: level,
            terms_evaluated: 1,
            exact_matches: if level == MedDRAMatchLevel::ExactPT {
                1
            } else {
                0
            },
        }
    }

    /// Get accuracy score.
    #[inline]
    #[must_use]
    pub fn score(&self) -> f64 {
        self.score
    }

    /// Best match level.
    #[inline]
    #[must_use]
    pub fn match_level(&self) -> MedDRAMatchLevel {
        self.match_level
    }

    /// Terms evaluated.
    #[inline]
    #[must_use]
    pub fn terms_evaluated(&self) -> u32 {
        self.terms_evaluated
    }

    /// Exact PT matches.
    #[inline]
    #[must_use]
    pub fn exact_matches(&self) -> u32 {
        self.exact_matches
    }

    /// Exact match rate.
    #[must_use]
    pub fn exact_match_rate(&self) -> f64 {
        if self.terms_evaluated == 0 {
            return 0.0;
        }
        self.exact_matches as f64 / self.terms_evaluated as f64
    }

    /// Is coding accuracy acceptable (score >= 0.8)?
    #[inline]
    #[must_use]
    pub fn is_acceptable(&self) -> bool {
        self.score >= 0.8
    }
}

impl GroundsTo for CodingAccuracy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for CodingAccuracy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CodingAccuracy={:.0}% (best={}, exact={}/{}, {})",
            self.score * 100.0,
            self.match_level,
            self.exact_matches,
            self.terms_evaluated,
            if self.is_acceptable() {
                "acceptable"
            } else {
                "review_needed"
            }
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod completeness_tests {
        use super::*;

        #[test]
        fn good_case() {
            let cs = CaseCompletenessScore::new(95, 110, 12, 12);
            assert!(cs.is_some());
            let cs = cs.unwrap_or_else(|| unreachable!());
            assert!(cs.is_good());
            assert!(!cs.is_poor());
            assert_eq!(cs.quality_tier(), "good");
        }

        #[test]
        fn poor_case() {
            let cs = CaseCompletenessScore::new(30, 110, 4, 12);
            assert!(cs.is_some());
            let cs = cs.unwrap_or_else(|| unreachable!());
            assert!(cs.is_poor());
            assert_eq!(cs.quality_tier(), "poor");
        }

        #[test]
        fn moderate_case() {
            let cs = CaseCompletenessScore::new(70, 110, 8, 12);
            assert!(cs.is_some());
            let cs = cs.unwrap_or_else(|| unreachable!());
            assert!(!cs.is_good());
            assert!(!cs.is_poor());
            assert_eq!(cs.quality_tier(), "moderate");
        }

        #[test]
        fn quick_computation() {
            let cs = CaseCompletenessScore::quick(90, 100);
            assert!(cs.is_some());
            let cs = cs.unwrap_or_else(|| unreachable!());
            assert!((cs.score() - 0.9).abs() < f64::EPSILON);
        }

        #[test]
        fn zero_total() {
            assert!(CaseCompletenessScore::new(0, 0, 0, 0).is_none());
        }
    }

    mod duplicate_tests {
        use super::*;

        #[test]
        fn high_probability() {
            let dp = DuplicateProbability::new(0.95, 0.92, DuplicateMethod::Composite);
            assert!(dp.is_some());
            let dp = dp.unwrap_or_else(|| unreachable!());
            assert!(dp.should_auto_merge());
            assert_eq!(dp.recommended_action(), "auto_merge");
        }

        #[test]
        fn flaggable() {
            let dp = DuplicateProbability::new(0.80, 0.78, DuplicateMethod::Jaccard);
            assert!(dp.is_some());
            let dp = dp.unwrap_or_else(|| unreachable!());
            assert!(dp.should_flag());
            assert_eq!(dp.recommended_action(), "flag_for_review");
        }

        #[test]
        fn unique_case() {
            let dp = DuplicateProbability::from_jaccard(0.15);
            assert!(dp.is_some());
            let dp = dp.unwrap_or_else(|| unreachable!());
            assert!(dp.is_likely_unique());
            assert_eq!(dp.recommended_action(), "accept_as_unique");
        }

        #[test]
        fn invalid_probability() {
            assert!(DuplicateProbability::new(1.5, 0.9, DuplicateMethod::Jaccard).is_none());
        }
    }

    mod narrative_tests {
        use super::*;

        #[test]
        fn adequate_narrative() {
            let nq = NarrativeQualityScore::new(150, 0.25, true, true);
            assert!(nq.is_adequate());
            assert!(!nq.is_sparse());
            assert!(nq.score() > 0.6);
        }

        #[test]
        fn sparse_narrative() {
            let nq = NarrativeQualityScore::new(10, 0.0, false, false);
            assert!(nq.is_sparse());
            assert!(!nq.is_adequate());
        }

        #[test]
        fn partial_quality() {
            let nq = NarrativeQualityScore::new(80, 0.15, true, false);
            // Has temporal but no causal — may or may not be adequate
            assert!(nq.score() > 0.3);
        }
    }

    mod coding_tests {
        use super::*;

        #[test]
        fn perfect_coding() {
            let matches = vec![
                MedDRAMatchLevel::ExactPT,
                MedDRAMatchLevel::ExactPT,
                MedDRAMatchLevel::ExactPT,
            ];
            let ca = CodingAccuracy::from_matches(&matches);
            assert!(ca.is_some());
            let ca = ca.unwrap_or_else(|| unreachable!());
            assert!((ca.score() - 1.0).abs() < f64::EPSILON);
            assert!(ca.is_acceptable());
            assert_eq!(ca.exact_matches(), 3);
        }

        #[test]
        fn mixed_coding() {
            let matches = vec![
                MedDRAMatchLevel::ExactPT,
                MedDRAMatchLevel::SameHLT,
                MedDRAMatchLevel::SameSOC,
            ];
            let ca = CodingAccuracy::from_matches(&matches);
            assert!(ca.is_some());
            let ca = ca.unwrap_or_else(|| unreachable!());
            // (1.0 + 0.8 + 0.4) / 3 = 0.7333
            assert!((ca.score() - 0.7333).abs() < 0.01);
            assert!(!ca.is_acceptable());
        }

        #[test]
        fn single_match() {
            let ca = CodingAccuracy::single(MedDRAMatchLevel::SameHLGT);
            assert!((ca.score() - 0.6).abs() < f64::EPSILON);
            assert_eq!(ca.match_level(), MedDRAMatchLevel::SameHLGT);
        }

        #[test]
        fn empty_matches() {
            assert!(CodingAccuracy::from_matches(&[]).is_none());
        }

        #[test]
        fn match_level_scores() {
            assert!((MedDRAMatchLevel::ExactPT.score() - 1.0).abs() < f64::EPSILON);
            assert!((MedDRAMatchLevel::SameHLT.score() - 0.8).abs() < f64::EPSILON);
            assert!((MedDRAMatchLevel::SameHLGT.score() - 0.6).abs() < f64::EPSILON);
            assert!((MedDRAMatchLevel::SameSOC.score() - 0.4).abs() < f64::EPSILON);
            assert!((MedDRAMatchLevel::NoMatch.score() - 0.0).abs() < f64::EPSILON);
        }
    }

    mod grounding_tests {
        use super::*;

        #[test]
        fn all_types_grounded() {
            let _cs = CaseCompletenessScore::primitive_composition();
            let _dp = DuplicateProbability::primitive_composition();
            let _nq = NarrativeQualityScore::primitive_composition();
            let _ca = CodingAccuracy::primitive_composition();
        }
    }
}
