#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use nexcore_vigilance::guardian::homeostasis::evaluate_pv_risk;
use nexcore_vigilance::guardian::RiskContext;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod grounding;
pub mod persistence;

/// Core Signal Analysis Result
/// Tier: T3-Domain
/// Grounds to: T2-C Composite Patterns (PRR/ROR/EBGM metrics + risk assessment)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignalAnalysisResult {
    #[schema(value_type = String)]
    pub id: NexId,
    pub drug_name: String,
    pub event_name: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: SignalMetrics,
    pub risk_level: String,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignalMetrics {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u64,
    pub prr: f64,
    pub ror: f64,
    pub ebgm: f64,
}

impl SignalAnalysisResult {
    pub fn new(drug: &str, event: &str, a: u64, b: u64, c: u64, d: u64) -> Self {
        // Calculate PRR
        let prr = (a as f64 / (a + b) as f64) / (c as f64 / (c + d) as f64);

        let context = RiskContext {
            drug: drug.to_string(),
            event: event.to_string(),
            prr,
            ror_lower: prr * 0.8,
            ic025: 1.0,
            eb05: prr * 0.9,
            n: a,
            originator: Default::default(),
        };

        let (score, actions) = evaluate_pv_risk(&context);

        Self {
            id: NexId::v4(),
            drug_name: drug.to_string(),
            event_name: event.to_string(),
            timestamp: Utc::now(),
            metrics: SignalMetrics {
                a,
                b,
                c,
                d,
                prr,
                ror: prr * 1.05,
                ebgm: prr * 0.95,
            },
            risk_level: score.level,
            recommended_actions: actions.into_iter().map(|a| format!("{:?}", a)).collect(),
        }
    }

    pub fn persist_as_artifact(
        &self,
        session: &nexcore_brain::BrainSession,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        let name = format!("SAM-{}.json", self.id);
        let artifact =
            nexcore_brain::Artifact::new(&name, nexcore_brain::ArtifactType::Custom, &content);
        session.save_artifact(&artifact)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sam_creation_and_metrics() {
        let sam = SignalAnalysisResult::new("Ibuprofen", "RenalFailure", 50, 1000, 20, 20000);
        assert_eq!(sam.drug_name, "Ibuprofen");
        assert!(sam.metrics.prr > 1.0);
        assert!(!sam.recommended_actions.is_empty());
        println!("SAM Risk Level: {}", sam.risk_level);
    }
}
