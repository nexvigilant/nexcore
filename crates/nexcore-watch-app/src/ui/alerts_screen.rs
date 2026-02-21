#![allow(dead_code)]
//! Alert queue screen — Slint bindings for P0-P5 alert display.
//!
//! ## Primitive Grounding
//! - ∂ (Boundary): P0-P5 priority thresholds
//! - σ (Sequence): ordered alert queue
//! - κ (Comparison): priority ordering for sort
//! - μ (Mapping): alert → display model
//!
//! ## Tier: T3

use nexcore_watch_core::{Alert, AlertLevel};

/// Alert screen view model — formatted alert for Slint binding.
///
/// ## Primitive: μ (Mapping) — domain → view
/// ## Tier: T2-C
#[derive(Debug, Clone)]
pub struct AlertViewModel {
    /// Priority badge — ∂ (Boundary)
    pub priority_badge: String,
    /// Priority color hex — μ (Mapping)
    pub priority_color: &'static str,
    /// Title text — σ (Sequence)
    pub title: String,
    /// Source text — λ (Location)
    pub source: String,
    /// Response deadline text — ν (Frequency)
    pub deadline_text: String,
    /// Whether to show attention indicator
    pub is_urgent: bool,
}

impl AlertViewModel {
    /// Build view model from Alert.
    ///
    /// ## Primitive: μ (Mapping) — Alert → ViewModel
    /// ## Tier: T2-C
    #[must_use]
    pub fn from_alert(alert: &Alert) -> Self {
        Self {
            priority_badge: format!("P{}", alert.level.code()),
            priority_color: priority_color(alert.level),
            title: alert.title.clone(),
            source: alert.source.clone(),
            deadline_text: deadline_text(alert.level),
            is_urgent: alert.level.is_harm_related(),
        }
    }
}

/// Map priority to display color.
///
/// ## Primitive: μ (Mapping)
/// ## Tier: T1
fn priority_color(level: AlertLevel) -> &'static str {
    match level {
        AlertLevel::P0PatientSafety => "#F44336",   // Red
        AlertLevel::P1SignalIntegrity => "#FF5722", // Red-Orange
        AlertLevel::P2Regulatory => "#FF9800",      // Orange
        AlertLevel::P3DataQuality => "#FFC107",     // Amber
        AlertLevel::P4Operational => "#8BC34A",     // Light Green
        AlertLevel::P5Cost => "#9E9E9E",            // Grey
    }
}

/// Format response deadline for display.
///
/// ## Primitive: μ (Mapping) + ν (Frequency)
/// ## Tier: T2-P
fn deadline_text(level: AlertLevel) -> String {
    match level.max_response_hours() {
        Some(0) => "IMMEDIATE".to_string(),
        Some(h) if h < 24 => format!("{h}h"),
        Some(h) => format!("{}d", h / 24),
        None => "—".to_string(),
    }
}

/// Sort alerts by priority (P0 first).
///
/// ## Primitive: κ (Comparison) + σ (Sequence)
/// ## Tier: T2-P
pub fn sort_alerts(alerts: &mut [Alert]) {
    alerts.sort_by_key(|a| a.level);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p0_alert_view_model() {
        let alert = Alert {
            level: AlertLevel::P0PatientSafety,
            title: "Fatal AE Detected".to_string(),
            body: String::new(),
            source: "Guardian".to_string(),
            timestamp_ms: 0,
            action_url: None,
        };
        let vm = AlertViewModel::from_alert(&alert);
        assert_eq!(vm.priority_badge, "P0");
        assert_eq!(vm.priority_color, "#F44336");
        assert_eq!(vm.deadline_text, "IMMEDIATE");
        assert!(vm.is_urgent);
    }

    #[test]
    fn p5_alert_view_model() {
        let alert = Alert {
            level: AlertLevel::P5Cost,
            title: "Budget Report".to_string(),
            body: String::new(),
            source: "Finance".to_string(),
            timestamp_ms: 0,
            action_url: None,
        };
        let vm = AlertViewModel::from_alert(&alert);
        assert_eq!(vm.priority_badge, "P5");
        assert_eq!(vm.deadline_text, "—");
        assert!(!vm.is_urgent);
    }

    #[test]
    fn sort_puts_p0_first() {
        let mut alerts = vec![
            Alert {
                level: AlertLevel::P3DataQuality,
                title: "DQ".to_string(),
                body: String::new(),
                source: String::new(),
                timestamp_ms: 0,
                action_url: None,
            },
            Alert {
                level: AlertLevel::P0PatientSafety,
                title: "PS".to_string(),
                body: String::new(),
                source: String::new(),
                timestamp_ms: 0,
                action_url: None,
            },
        ];
        sort_alerts(&mut alerts);
        assert_eq!(alerts[0].level, AlertLevel::P0PatientSafety);
        assert_eq!(alerts[1].level, AlertLevel::P3DataQuality);
    }
}
