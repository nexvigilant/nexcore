//! Cross-Domain Vigilance Bridge.
//!
//! Maps signals between domains using T1 primitives:
//! - PV (∃ Drug × f Harm) → Finance (∃ Asset × f Sentiment)
//! - Confidence is preserved via Transfer Multipliers.

use crate::events::EventBus;
use crate::models::{Event, Urgency};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Transfer domains for cross-domain mapping.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Domain {
    Pharmacovigilance,
    Finance,
    Software,
    Infrastructure,
}

/// A signal identified in a source domain.
#[derive(Debug, Clone, Deserialize)]
pub struct SourceSignal {
    pub source_domain: Domain,
    pub entity: String,
    pub algorithm: String,
    pub value: f64,
    pub confidence: f64,
}

/// The Vigilance Bridge orchestrator.
pub struct VigilanceBridge {
    pub bus: EventBus,
}

impl VigilanceBridge {
    /// Transfer a signal to related domains and emit Vigil events.
    pub async fn transfer(&self, signal: SourceSignal) {
        match signal.source_domain {
            Domain::Pharmacovigilance => {
                self.transfer_pv_to_finance(signal).await;
            }
            _ => {
                info!(
                    "Signal transfer not yet implemented for {:?}",
                    signal.source_domain
                );
            }
        }
    }

    async fn transfer_pv_to_finance(&self, signal: SourceSignal) {
        // Transfer Multiplier for PV -> Finance is 0.92
        let multiplier = 0.92;
        let transferred_confidence = signal.confidence * multiplier;

        // Map PV entity to Finance entity (e.g., Ozempic -> Novo Nordisk)
        // For prototype, we use a pass-through or suffix
        let finance_entity = format!("{} (Related Equity)", signal.entity);

        let payload = serde_json::json!({
            "original_signal": {
                "domain": "Pharmacovigilance",
                "algorithm": signal.algorithm,
                "value": signal.value,
            },
            "transferred_signal": {
                "domain": "Finance",
                "type": "Sentiment",
                "entity": finance_entity,
                "projected_impact": signal.value,
                "transfer_confidence": transferred_confidence,
            },
            "primitives": ["N", "κ", "∝"]
        });

        let event = Event {
            source: "vigilance_bridge".to_string(),
            event_type: "cross_domain_transfer".to_string(),
            priority: if signal.value > 4.0 {
                Urgency::High
            } else {
                Urgency::Normal
            },
            payload,
            ..Event::default()
        };

        info!(
            entity = %signal.entity,
            impact = signal.value,
            "pv_to_finance_transfer_executed"
        );

        self.bus.emit(event).await;
    }
}
