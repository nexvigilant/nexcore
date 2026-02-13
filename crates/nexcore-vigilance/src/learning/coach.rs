//! Intervention generation for coaching.

use super::models::{AndonSignal, Intervention, InterventionType};
use chrono::Utc;
use nexcore_id::NexId;

/// Alert from monitoring system.
pub struct MonitorAlert {
    /// Alert type (e.g., "error", "success", "warning")
    pub alert_type: String,
    /// Severity signal
    pub severity: AndonSignal,
    /// Alert message
    pub message: String,
}

/// Generates coaching interventions from alerts.
pub struct InterventionGenerator {
    model: String,
}

impl InterventionGenerator {
    /// Create a new intervention generator.
    #[must_use]
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
        }
    }

    /// Get the model identifier.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Generate an intervention from an alert.
    #[must_use]
    pub fn generate_intervention(&self, alert: &MonitorAlert) -> Intervention {
        let intervention_type = match alert.alert_type.as_str() {
            "error" => {
                if alert.severity == AndonSignal::Red {
                    InterventionType::Halt
                } else {
                    InterventionType::Correction
                }
            }
            "success" => InterventionType::Encouragement,
            _ => InterventionType::Guidance,
        };

        Intervention {
            id: NexId::v4(),
            timestamp: Utc::now(),
            trigger: format!("{}: {}", alert.alert_type, alert.message),
            intervention_type,
            signal: alert.severity,
            message: Self::get_fallback_message(intervention_type, &alert.message),
            context: serde_json::json!({}),
            student_response: None,
            effective: None,
        }
    }

    fn get_fallback_message(intervention_type: InterventionType, msg: &str) -> String {
        match intervention_type {
            InterventionType::Halt => format!("Please stop. A critical issue occurred: {msg}"),
            InterventionType::Correction => format!("Consider adjusting your approach. {msg}"),
            InterventionType::Encouragement => format!("Good progress! {msg}"),
            InterventionType::Guidance => format!("Suggestion: {msg}"),
        }
    }
}
