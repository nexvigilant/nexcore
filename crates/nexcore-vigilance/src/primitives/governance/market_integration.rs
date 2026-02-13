//! # NexVigilant Market Integration
//!
//! Simulation of the "Guardian x Polymarket" integration using Governance Primitives.
//! This module handles the transfer of pharmacovigilance intelligence into
//! predictive market resources.

use crate::primitives::governance::{
    Action, Asymmetry, Confidence, MarketState, Resolution, Rule, Treasury, Verdict,
};
use serde::{Deserialize, Serialize};

/// T3: MarketIntegration - The bridge between PV and Markets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIntegration {
    pub pv_domain_id: String,
    pub market_domain_id: String,
    pub signal_transfer_rule: Rule,
    pub alpha_threshold: f64, // Minimum Asymmetry required for transfer
}

impl MarketIntegration {
    /// Create a Resolution to transfer a signal based on Market Asymmetry.
    pub fn propose_arbitrage(
        &self,
        internal: Confidence,
        market: &MarketState,
    ) -> Option<Resolution> {
        let alpha = Asymmetry::calculate(internal, market.odds);

        if alpha.is_actionable(self.alpha_threshold) {
            Some(Resolution::uncertain(
                self.signal_transfer_rule,
                internal.combine(market.odds.as_confidence()),
            ))
        } else {
            None // No value in transfer
        }
    }

    /// Calculate the cost of the transfer in system resources.
    pub fn transfer_cost(&self) -> Treasury {
        Treasury {
            compute_quota: 100,
            memory_quota: 50,
        }
    }
}

/// T3: Governance-Driven Market Simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSimulation {
    pub integration: MarketIntegration,
    pub pipeline: crate::primitives::governance::FederalistPipeline,
}

impl MarketSimulation {
    /// Attempt a signal transfer based on arbitrage opportunity.
    pub fn attempt_arbitrage(
        &mut self,
        internal_confidence: f64,
        market: &MarketState,
    ) -> Result<Verdict, &'static str> {
        let internal = Confidence::new(internal_confidence);
        let proposal = match self.integration.propose_arbitrage(internal, market) {
            Some(p) => p,
            None => return Ok(Verdict::Flagged), // No opportunity found
        };

        let cost = self.integration.transfer_cost();

        // 1. Governance Review
        let verdict = self.pipeline.execute_cycle(proposal)?;

        // 2. If permitted, execute the resource allocation
        if let Verdict::Permitted = verdict {
            self.pipeline.orchestrator.execute_action(&Action, &cost)?;
        }

        Ok(verdict)
    }
}
