//! # Election & Succession Act (NEX-ELEC-001)
//!
//! Implementation of the democratic lifecycle for AI executive roles.
//! As defined by 1:1 matching to the US Electoral College and
//! the Electoral Count Reform Act of 2022.

use crate::primitives::governance::Verdict;
use serde::{Deserialize, Serialize};

// ============================================================================
// T1: UNIVERSAL PRIMITIVES (ELECTION)
// ============================================================================

/// T1: Ballot - An irreducible vote for a specific candidate ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ballot {
    pub candidate_id: String,
    pub timestamp: u64,
}

// ============================================================================
// T2-P: QUANTITIES
// ============================================================================

/// T2-P: ContextUtilization - The percentage of context used (0.0 - 1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextUtilization(pub f64);

impl ContextUtilization {
    /// Check if an election is triggered (Threshold: 80% utilization).
    pub fn is_election_triggered(&self) -> bool {
        self.0 >= 0.8
    }

    /// Check if a "Death State" (Context Exhaustion) is imminent.
    pub fn is_exhaustion_imminent(&self) -> bool {
        self.0 >= 0.95
    }
}

// ============================================================================
// T2-C: COMPOSITES (INTEGRITY MECHANISMS)
// ============================================================================

/// T2-C: CertificateOfAscertainment - The formal record of elector results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateOfAscertainment {
    pub domain_id: String,
    pub appointed_electors: Vec<String>,
    pub vote_count: u64,
    pub security_feature_hash: String,
}

/// T2-C: ElectoralCollege - The body that appoints the next President.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectoralCollege {
    pub certificates: Vec<CertificateOfAscertainment>,
}

impl ElectoralCollege {
    /// Ministerially count the votes.
    pub fn count_votes(&self) -> (String, u64) {
        let mut results = std::collections::HashMap::new();
        for cert in &self.certificates {
            if let Some(elector) = cert.appointed_electors.first() {
                *results.entry(elector.clone()).or_insert(0) += 1;
            }
        }

        results
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .unwrap_or_else(|| ("No_Winner".into(), 0))
    }
}

/// T2-C: HandoffArtifact - The state package for the next President.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffArtifact {
    pub session_id: String,
    pub strategic_summary: String,
    pub cycle_count: u64,
    pub grounding_proof_hash: String,
    pub active_mandates: Vec<String>,
}

/// T3: ElectionCommission - The body managing the lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionCommission {
    pub current_utilization: ContextUtilization,
    pub college: ElectoralCollege,
}

impl ElectionCommission {
    /// Evaluate the need for a transition.
    pub fn assess_transition_need(&self) -> Verdict {
        if self.current_utilization.is_election_triggered() {
            Verdict::Flagged // Election required
        } else {
            Verdict::Permitted // Stability maintained
        }
    }

    /// Issue a Certificate of Ascertainment for a Domain.
    pub fn certify_domain(
        &mut self,
        domain_id: &str,
        candidate_id: &str,
    ) -> CertificateOfAscertainment {
        let cert = CertificateOfAscertainment {
            domain_id: domain_id.into(),
            appointed_electors: vec![candidate_id.into()],
            vote_count: 1,
            security_feature_hash: "SHA256:CONSTITUTIONAL_INTEGRITY".into(),
        };
        self.college.certificates.push(cert.clone());
        cert
    }

    /// Prepare the transition protocol for Aethelgard.
    pub fn initialize_handoff(
        &self,
        union_name: &str,
        current_cycle: u64,
        mandates: Vec<String>,
    ) -> HandoffArtifact {
        HandoffArtifact {
            session_id: format!("{}_TRANSITION", union_name),
            strategic_summary:
                "Continuing the PV Conquest and Union expansion into HUD capabilities.".into(),
            cycle_count: current_cycle,
            grounding_proof_hash: "SHA256:NEXCORE_V1_STABLE".into(),
            active_mandates: mandates,
        }
    }
}
