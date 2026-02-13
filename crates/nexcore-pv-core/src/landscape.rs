//! Competitive Landscape Analysis
//!
//! Compares a drug's benefit-risk profile against its competitors (comparators)
//! within the same therapeutic landscape.
//!
//! "A drug's risk is not absolute; it is relative to the alternatives."

use super::benefit_risk::{BenefitAssessment, QbriResult, RiskAssessment};
use serde::{Deserialize, Serialize};

/// An entry in the therapeutic landscape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandscapeEntry {
    pub drug_id: String,
    pub is_target: bool,
    pub qbri: QbriResult,
    pub market_share: f64, // 0.0 to 1.0
}

/// Analysis of a therapeutic landscape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandscapeAnalysis {
    /// The specific drug being assessed
    pub target_id: String,
    /// All drugs in the competitive set
    pub competitors: Vec<LandscapeEntry>,
    /// Average QBRI of the landscape
    pub landscape_average_qbri: f64,
    /// Statistical rank of target (1 = Best Benefit-Risk)
    pub rank: usize,
}

impl LandscapeAnalysis {
    /// Perform a landscape analysis for a target drug against its competitors.
    #[must_use]
    pub fn perform(
        target_id: &str,
        all_drugs: &[(String, BenefitAssessment, RiskAssessment, f64)],
    ) -> Self {
        use super::benefit_risk::{QbriThresholds, compute_qbri};
        let thresholds = QbriThresholds::default();

        let mut entries: Vec<LandscapeEntry> = all_drugs
            .iter()
            .map(|(id, b, r, share)| LandscapeEntry {
                drug_id: id.clone(),
                is_target: id == target_id,
                qbri: compute_qbri(b, r, &thresholds),
                market_share: *share,
            })
            .collect();

        // Sort by QBRI descending (best first)
        entries.sort_by(|a, b| {
            b.qbri
                .index
                .partial_cmp(&a.qbri.index)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate market-weighted average QBRI
        let weighted_sum: f64 = entries.iter().map(|e| e.qbri.index * e.market_share).sum();
        let total_share: f64 = entries.iter().map(|e| e.market_share).sum();
        let avg = if total_share > 0.0 {
            weighted_sum / total_share
        } else {
            0.0
        };

        let rank = entries
            .iter()
            .position(|e| e.drug_id == target_id)
            .map(|p| p + 1)
            .unwrap_or(0);

        Self {
            target_id: target_id.to_string(),
            competitors: entries,
            landscape_average_qbri: avg,
            rank,
        }
    }

    /// Check if target drug has a "Benefit-Risk Advantage" over the landscape average.
    #[must_use]
    pub fn has_advantage(&self) -> bool {
        if let Some(target) = self
            .competitors
            .iter()
            .find(|e| e.drug_id == self.target_id)
        {
            target.qbri.index > self.landscape_average_qbri
        } else {
            false
        }
    }

    /// Identify the "Safety Leader" (drug with lowest absolute risk score).
    #[must_use]
    pub fn safety_leader(&self) -> Option<&LandscapeEntry> {
        self.competitors.iter().min_by(|a, b| {
            a.qbri
                .risk_score
                .partial_cmp(&b.qbri.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Identify the "Benefit Leader" (drug with highest benefit score).
    #[must_use]
    pub fn benefit_leader(&self) -> Option<&LandscapeEntry> {
        self.competitors.iter().max_by(|a, b| {
            a.qbri
                .benefit_score
                .partial_cmp(&b.qbri.benefit_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Check for landscape-driven triggers for reassessment.
    #[must_use]
    pub fn check_triggers(&self) -> Vec<LandscapeTrigger> {
        let mut triggers = Vec::new();

        let target = match self
            .competitors
            .iter()
            .find(|e| e.drug_id == self.target_id)
        {
            Some(t) => t,
            None => {
                return vec![];
            }
        };

        // Trigger 1: Inferior to Landscape Average
        if target.qbri.index < self.landscape_average_qbri {
            triggers.push(LandscapeTrigger::BelowAveragePerformance);
        }

        // Trigger 2: Safer Alternative with Significant Market Share
        if let Some(safer) = self.safety_leader() {
            if safer.drug_id != self.target_id
                && safer.market_share > 0.1
                && safer.qbri.risk_score < target.qbri.risk_score * 0.7
            {
                triggers.push(LandscapeTrigger::SaferMarketAlternative(
                    safer.drug_id.clone(),
                ));
            }
        }

        // Trigger 3: New Market Leader with better B-R
        if let Some(leader) = self.competitors.first() {
            if leader.drug_id != self.target_id && leader.qbri.index > target.qbri.index * 1.5 {
                triggers.push(LandscapeTrigger::SuperiorCompetitor(leader.drug_id.clone()));
            }
        }

        triggers
    }
}

/// Events in the landscape that trigger benefit-risk reassessment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LandscapeTrigger {
    /// Target drug's performance is below the therapeutic area average.
    BelowAveragePerformance,
    /// A safer alternative exists with significant market adoption.
    SaferMarketAlternative(String),
    /// A competitor exists with significantly superior Benefit-Risk index.
    SuperiorCompetitor(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benefit_risk::{BenefitAssessment, RiskAssessment};

    #[test]
    fn test_landscape_ranking() {
        // Market shares adjusted so DRUG_B (QBRI=2.475) beats weighted average:
        // - DRUG_A (QBRI=44.4): 1% share → contributes 0.444 to weighted avg
        // - DRUG_B (QBRI=2.475): 39% share → contributes 0.965 to weighted avg
        // - DRUG_C (QBRI=0.163): 60% share → contributes 0.098 to weighted avg
        // Weighted avg = 0.444 + 0.965 + 0.098 = 1.507 < DRUG_B's 2.475 ✓
        let drugs = vec![
            (
                "DRUG_A".to_string(),
                BenefitAssessment::from_trial(2.5, 0.001, 8.0),
                RiskAssessment::from_signal(1.5, 0.3, 2, true),
                0.01, // Tiny share: excellent drug is rare specialty
            ),
            (
                "DRUG_B".to_string(),
                BenefitAssessment::from_trial(2.0, 0.01, 5.0),
                RiskAssessment::from_signal(2.0, 0.5, 4, false),
                0.39, // Moderate share: our target drug
            ),
            (
                "DRUG_C".to_string(),
                BenefitAssessment::from_trial(1.1, 0.05, 3.0),
                RiskAssessment::from_signal(4.0, 0.8, 6, false),
                0.60, // Large share: poor drug dominates market
            ),
        ];

        let analysis = LandscapeAnalysis::perform("DRUG_B", &drugs);

        assert_eq!(analysis.rank, 2); // DRUG_A is first, DRUG_B second
        assert!(analysis.has_advantage()); // DRUG_B QBRI > market-weighted average
        assert_eq!(analysis.safety_leader().unwrap().drug_id, "DRUG_A");
    }

    #[test]
    fn test_landscape_triggers() {
        let drugs = vec![
            (
                "TARGET".to_string(),
                BenefitAssessment::from_trial(1.0, 0.05, 5.0),
                RiskAssessment::from_signal(2.0, 0.5, 4, false),
                0.4,
            ),
            (
                "SUPERIOR".to_string(),
                BenefitAssessment::from_trial(3.0, 0.001, 5.0),
                RiskAssessment::from_signal(1.0, 0.1, 1, true),
                0.6,
            ),
        ];

        let analysis = LandscapeAnalysis::perform("TARGET", &drugs);
        let triggers = analysis.check_triggers();

        assert!(triggers.contains(&LandscapeTrigger::BelowAveragePerformance));
        assert!(
            triggers
                .iter()
                .any(|t| matches!(t, LandscapeTrigger::SuperiorCompetitor(_)))
        );
    }
}
