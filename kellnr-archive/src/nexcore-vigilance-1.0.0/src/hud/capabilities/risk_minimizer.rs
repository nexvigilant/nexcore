//! # Capability 4: Risk Minimizer Actuator (Guardrail Protocol)
//!
//! Implementation of the Risk Minimization Actuator as a core structural
//! capability within the HUD domain.
//!
//! This capability integrates the findings of BDI, ECS, and ACA to
//! dynamically apply the 8-Level Risk Hierarchy (ToV §59.2).
//!
//! ## Knowledge Transfer (Silo Cross-Pollination)
//! - **Betting (BDI):** Feeds frequentist disproportionality signals.
//! - **PV (ECS/ACA):** Feeds Bayesian credibility and causal accountability.
//! - **Ferrostack (Translation):** Prepares for the "UI-Actuator" layer to display
//!   guardrails in web-native patterns (Signals/Suspense).

use crate::algorithmovigilance::scoring::AcaScoringInput;
use crate::hud::capabilities::{
    BayesianCredibilityLayer, CausalAttributionEngine, SignalIdentificationProtocol,
};
use crate::primitives::governance::{Guardrail, RiskMinimizationLevel, RiskMinimizer};
use nexcore_labs::betting::bdi::ContingencyTable;
use serde::{Deserialize, Serialize};

/// T3: RiskMinimizerActuator - Capability 4 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMinimizerActuator {
    pub id: String,
    pub minimizer: RiskMinimizer,
}

impl RiskMinimizerActuator {
    pub fn new() -> Self {
        Self {
            id: "CAP-004".into(),
            minimizer: RiskMinimizer {
                level: RiskMinimizationLevel::Information,
                active_guardrails: vec![],
            },
        }
    }

    /// Determine the appropriate Risk Minimization Level based on integrated signals.
    pub fn assess_and_act(
        &mut self,
        bdi: &SignalIdentificationProtocol,
        ecs: &BayesianCredibilityLayer,
        aca: &CausalAttributionEngine,
        table: ContingencyTable,
        aca_input: &AcaScoringInput,
    ) -> RiskMinimizationLevel {
        let bdi_res = bdi.identify_signal(table);
        let ecs_res = ecs.calculate_credibility(0.8, -1, true, 2.0);
        let aca_res = aca.attribute_causality(aca_input);

        // Escalation Logic (integrated)
        if aca_res.confidence.value() > 0.9 && bdi_res.value.bdi > 5.0 {
            self.minimizer.level = RiskMinimizationLevel::Suspension;
            self.minimizer
                .active_guardrails
                .push(Guardrail::HumanReviewRequired);
        } else if ecs_res.value.ecs > 3.0 {
            self.minimizer.level = RiskMinimizationLevel::Guardrails;
            self.minimizer
                .active_guardrails
                .push(Guardrail::ConfidenceThreshold(0.8));
        }

        self.minimizer.level
    }
}
