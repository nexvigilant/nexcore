//! # Capability 2: Bayesian Credibility Layer (ECS Engine)
//!
//! Implementation of the Edge Confidence Score (ECS) as a core
//! structural capability within the HUD domain.
//!
//! Grounding: Adapted from EBGM/BCPNN in pharmacovigilance.
//! Formula: ECS = U × R × T
//!
//! Where:
//! - U (Unexpectedness): Signal rarity/surprise.
//! - R (Reliability): Data source quality factors.
//! - T (Temporal): Information decay as event approaches.

use nexcore_labs::betting::{EcsResult, ReliabilityInput, SportType, calculate_ecs};
use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};

/// T3: BayesianCredibilityLayer - Capability 2 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianCredibilityLayer {
    pub id: String,
    pub ecs_engine_active: bool,
}

impl BayesianCredibilityLayer {
    pub fn new() -> Self {
        Self {
            id: "CAP-002".into(),
            ecs_engine_active: true,
        }
    }

    /// Calculate the credibility of a signal using the ECS engine.
    /// Returns a Measured<EcsResult> ensuring uncertainty is quantified.
    pub fn calculate_credibility(
        &self,
        public_pct: f64,
        line_move_dir: i8,
        steam_detected: bool,
        hours_to_event: f64,
    ) -> Measured<EcsResult> {
        let reliability = ReliabilityInput::default();
        let result = calculate_ecs(
            public_pct,
            line_move_dir,
            steam_detected,
            &reliability,
            hours_to_event,
            SportType::Nfl, // Defaulting to NFL for general simulation
        );

        let confidence = if result.is_actionable {
            Confidence::new(0.9)
        } else {
            Confidence::new(0.3)
        };

        Measured::uncertain(result, confidence)
    }
}
