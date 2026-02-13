use crate::models::{DecisionAction, Event, Urgency};
use tracing::{info, warn};

pub struct AuthorityConfig {
    pub autonomous_allowed: Vec<String>,
    pub forbidden: Vec<String>,
    pub requires_confirmation: Vec<String>,
}

pub struct DecisionEngine {
    authority: AuthorityConfig,
}

impl DecisionEngine {
    pub fn new(authority: AuthorityConfig) -> Self {
        Self { authority }
    }

    pub async fn decide(&self, event: &Event) -> DecisionAction {
        info!(
            event_id = ?event.id,
            source = %event.source,
            event_type = %event.event_type,
            "decision_evaluating"
        );

        // Rule 1: Critical events always escalate
        if event.priority == Urgency::Critical {
            info!("decision_escalate: critical_priority");
            return DecisionAction::Escalate;
        }

        // Rule 2: Direct user speech always invokes Claude
        if event.source == "voice" && event.event_type == "user_spoke" {
            info!("decision_invoke: user_speech");
            return DecisionAction::InvokeClaude;
        }

        // Rule 4: Check autonomous authority
        if self.can_handle_autonomously(event) {
            info!("decision_autonomous: authority_granted");
            return DecisionAction::AutonomousAct;
        }

        // Rule 5: Check if forbidden
        if self.is_forbidden(event) {
            warn!(event_type = %event.event_type, "decision_forbidden");
            return DecisionAction::SilentLog;
        }

        // Rule 6: Low priority events just log
        if event.priority == Urgency::Low {
            info!("decision_log: low_priority");
            return DecisionAction::SilentLog;
        }

        // Default
        info!("decision_invoke: default");
        DecisionAction::InvokeClaude
    }

    fn can_handle_autonomously(&self, event: &Event) -> bool {
        let action_type = format!("{}:{}", event.source, event.event_type);
        self.authority.autonomous_allowed.contains(&action_type)
            || self
                .authority
                .autonomous_allowed
                .contains(&event.event_type)
    }

    fn is_forbidden(&self, event: &Event) -> bool {
        let action_type = format!("{}:{}", event.source, event.event_type);
        self.authority.forbidden.contains(&action_type)
            || self.authority.forbidden.contains(&event.event_type)
    }
}
