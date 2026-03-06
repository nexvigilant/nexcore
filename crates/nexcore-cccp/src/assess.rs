//! Phase 2: Assess — Gap analysis using the 15-dimensional DomainStateVector.
//!
//! Algorithm: COMPARE (κ(A, B) = |A△B|) + PERSPECTIVE (∂₂(ς) where ς = ς₁)
//! Templates: NV-COR-SOP-002 Gap Analysis Matrix + Strategic Assessment Report

use serde::{Deserialize, Serialize};

use nexcore_vigilance::caba::{
    CPACategory, DomainCategory, DomainStateVector, EPACategory, EPATier, ProficiencyLevel,
    cpa_required_domains, epa_required_domains,
};

// Variant arrays sourced from CABA canonical ALL consts — compile-time complete.

/// A single domain gap with current and desired proficiency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGap {
    pub domain: DomainCategory,
    pub current: ProficiencyLevel,
    pub desired: ProficiencyLevel,
    pub gap: i8,
}

/// EPA readiness assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaReadiness {
    pub epa: EPACategory,
    pub ready: bool,
    pub blocking_domains: Vec<DomainCategory>,
    /// The minimum proficiency level required for this EPA's domains.
    pub threshold: ProficiencyLevel,
}

/// CPA maturity assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpaMaturity {
    pub cpa: CPACategory,
    pub readiness_score: f64,
    pub blocking_epas: Vec<EPACategory>,
}

/// The complete Phase 2 output — a gap analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    pub current: DomainStateVector,
    pub desired: DomainStateVector,
    pub domain_gaps: Vec<DomainGap>,
    pub epa_readiness: Vec<EpaReadiness>,
    pub cpa_maturity: Vec<CpaMaturity>,
    pub overall_readiness: f64,
}

impl GapAnalysis {
    /// Compute a full gap analysis from current and desired state vectors.
    pub fn compute(current: DomainStateVector, desired: DomainStateVector) -> Self {
        let gap_values = current.gap_from(&desired);

        let domain_gaps: Vec<DomainGap> = DomainCategory::ALL
            .iter()
            .enumerate()
            .map(|(i, &domain)| DomainGap {
                domain,
                current: current.get(domain),
                desired: desired.get(domain),
                gap: gap_values[i],
            })
            .collect();

        let epa_readiness: Vec<EpaReadiness> = EPACategory::ALL
            .iter()
            .map(|&epa| {
                // EPA threshold based on tier:
                // Core (EPA-01 to 08): L3 Competent — can perform independently
                // Advanced (EPA-09 to 14, 17): L4 Proficient — can supervise others
                // Expert (EPA-15, 16, 18-21): L5 Expert — can innovate and lead
                let threshold = match epa.tier() {
                    EPATier::Core => ProficiencyLevel::L3Competent,
                    EPATier::Advanced => ProficiencyLevel::L4Proficient,
                    EPATier::Expert => ProficiencyLevel::L5Expert,
                };
                let required = epa_required_domains(epa);
                let blocking: Vec<DomainCategory> = required
                    .iter()
                    .filter(|&&d| current.get(d) < threshold)
                    .copied()
                    .collect();
                EpaReadiness {
                    epa,
                    ready: blocking.is_empty(),
                    blocking_domains: blocking,
                    threshold,
                }
            })
            .collect();

        let cpa_maturity: Vec<CpaMaturity> = CPACategory::ALL
            .iter()
            .map(|&cpa| {
                let required_domains = cpa_required_domains(cpa);
                // CPA readiness: each domain must meet at least L3 (Competent)
                let cpa_threshold = ProficiencyLevel::L3Competent;
                let met = required_domains
                    .iter()
                    .filter(|d| current.get(**d) >= cpa_threshold)
                    .count();
                let total = required_domains.len().max(1);
                let readiness_score = met as f64 / total as f64;

                let blocking_epas: Vec<EPACategory> = epa_readiness
                    .iter()
                    .filter(|er| !er.ready)
                    .filter(|er| {
                        let epa_domains = epa_required_domains(er.epa);
                        epa_domains.iter().any(|d| required_domains.contains(d))
                    })
                    .map(|er| er.epa)
                    .collect();

                CpaMaturity {
                    cpa,
                    readiness_score,
                    blocking_epas,
                }
            })
            .collect();

        let overall_readiness = current.domains_met(&desired) as f64 / 15.0;

        Self {
            current,
            desired,
            domain_gaps,
            epa_readiness,
            cpa_maturity,
            overall_readiness,
        }
    }

    /// Domains with the largest gaps (sorted descending).
    pub fn priority_gaps(&self) -> Vec<&DomainGap> {
        let mut gaps: Vec<&DomainGap> = self.domain_gaps.iter().filter(|g| g.gap > 0).collect();
        gaps.sort_by(|a, b| b.gap.cmp(&a.gap));
        gaps
    }

    /// EPAs that are NOT ready.
    pub fn blocked_epas(&self) -> Vec<&EpaReadiness> {
        self.epa_readiness.iter().filter(|e| !e.ready).collect()
    }

    /// CPAs below a readiness threshold.
    pub fn immature_cpas(&self, threshold: f64) -> Vec<&CpaMaturity> {
        self.cpa_maturity
            .iter()
            .filter(|c| c.readiness_score < threshold)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(level: ProficiencyLevel) -> DomainStateVector {
        DomainStateVector::new([level; 15])
    }

    #[test]
    fn test_gap_analysis_compute() {
        let current = uniform(ProficiencyLevel::L1Novice);
        let desired = uniform(ProficiencyLevel::L3Competent);

        let analysis = GapAnalysis::compute(current, desired);

        assert_eq!(analysis.domain_gaps.len(), 15);
        for gap in &analysis.domain_gaps {
            assert_eq!(gap.gap, 2);
        }
        assert!(!analysis.blocked_epas().is_empty());
        assert!((analysis.overall_readiness - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gap_analysis_all_met() {
        let state = uniform(ProficiencyLevel::L3Competent);
        let analysis = GapAnalysis::compute(state.clone(), state);

        assert!(analysis.priority_gaps().is_empty());
        assert!((analysis.overall_readiness - 1.0).abs() < f64::EPSILON);
    }
}
