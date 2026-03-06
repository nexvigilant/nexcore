//! # Signal Bridge
//!
//! Inter-crate signal conversion: Cytokine → Impulse → MotorCommand → ToolClassification.
//!
//! Bridges the cytokine signaling system (slow, broadcast) through neural processing
//! (fast, targeted) to muscular execution (tool activation).

use nexcore_chrono::DateTime;
use nexcore_cytokine::{Cytokine, ThreatLevel};
use nexcore_muscular::{MuscleType, ToolClassification};

use crate::{Impulse, MotorCommand};

/// Convert a cytokine signal into a nervous system impulse.
///
/// Maps cytokine severity to impulse priority and preserves source/target routing.
///
/// **Biological mapping**: Cytokine receptor activation → afferent nerve impulse.
pub fn cytokine_to_impulse(cytokine: &Cytokine) -> Impulse {
    let priority = match cytokine.severity {
        ThreatLevel::Trace => 0,
        ThreatLevel::Low => 50,
        ThreatLevel::Medium => 100,
        ThreatLevel::High => 200,
        ThreatLevel::Critical => 255,
    };

    let source = cytokine
        .source
        .clone()
        .unwrap_or_else(|| format!("cytokine:{}", cytokine.family));

    let target = cytokine
        .target
        .clone()
        .unwrap_or_else(|| "motor-cortex".to_string());

    Impulse {
        source,
        target,
        payload: cytokine.name.clone(),
        priority,
        timestamp: DateTime::now(),
    }
}

/// Convert a nervous impulse into a motor command for tool execution.
///
/// **Biological mapping**: Efferent nerve impulse → motor neuron activation.
pub fn impulse_to_motor_command(impulse: &Impulse) -> MotorCommand {
    MotorCommand {
        target_tool: impulse.target.clone(),
        params_json: format!(
            r#"{{"source":"{}","payload":"{}"}}"#,
            impulse.source, impulse.payload
        ),
        urgency: impulse.priority,
    }
}

/// Classify a motor command using the muscular system's tool classification.
///
/// **Biological mapping**: Neuromuscular junction — nerve signal activates muscle fiber.
pub fn classify_command(command: &MotorCommand) -> ToolClassification {
    ToolClassification::classify(&command.target_tool)
}

/// Trace the full signal path: Cytokine → Impulse → MotorCommand → ToolClassification.
///
/// Returns all intermediate types for observability.
pub fn trace_signal_path(
    cytokine: &Cytokine,
) -> (Impulse, MotorCommand, ToolClassification) {
    let impulse = cytokine_to_impulse(cytokine);
    let command = impulse_to_motor_command(&impulse);
    let classification = classify_command(&command);
    (impulse, command, classification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_cytokine::CytokineFamily;

    #[test]
    fn test_cytokine_to_impulse_critical() {
        let cytokine = Cytokine::new(CytokineFamily::TnfAlpha, "alert")
            .with_severity(ThreatLevel::Critical);

        let impulse = cytokine_to_impulse(&cytokine);

        assert_eq!(impulse.priority, 255);
        assert_eq!(impulse.payload, "alert");
        assert!(impulse.source.contains("cytokine:"));
        assert_eq!(impulse.target, "motor-cortex");
    }

    #[test]
    fn test_cytokine_to_impulse_trace() {
        let cytokine = Cytokine::new(CytokineFamily::Il10, "info")
            .with_severity(ThreatLevel::Trace);

        let impulse = cytokine_to_impulse(&cytokine);
        assert_eq!(impulse.priority, 0);
    }

    #[test]
    fn test_cytokine_to_impulse_with_source_target() {
        let cytokine = Cytokine::new(CytokineFamily::Il6, "routed")
            .with_severity(ThreatLevel::Medium)
            .with_source("guardian")
            .with_target("bash");

        let impulse = cytokine_to_impulse(&cytokine);

        assert_eq!(impulse.source, "guardian");
        assert_eq!(impulse.target, "bash");
        assert_eq!(impulse.priority, 100);
    }

    #[test]
    fn test_impulse_to_motor_command() {
        let impulse = Impulse {
            source: "sensor".to_string(),
            target: "bash".to_string(),
            payload: "execute".to_string(),
            priority: 200,
            timestamp: DateTime::now(),
        };

        let command = impulse_to_motor_command(&impulse);

        assert_eq!(command.target_tool, "bash");
        assert_eq!(command.urgency, 200);
        assert!(command.params_json.contains("sensor"));
        assert!(command.params_json.contains("execute"));
    }

    #[test]
    fn test_classify_command_bash() {
        let command = MotorCommand {
            target_tool: "bash".to_string(),
            params_json: "{}".to_string(),
            urgency: 100,
        };

        let classification = classify_command(&command);
        assert_eq!(classification.tool_name, "bash");
        assert_eq!(classification.muscle_type, MuscleType::Skeletal);
    }

    #[test]
    fn test_full_signal_path() {
        let cytokine = Cytokine::new(CytokineFamily::TnfAlpha, "threat-signal")
            .with_severity(ThreatLevel::High)
            .with_target("bash");

        let (impulse, command, classification) = trace_signal_path(&cytokine);

        // Cytokine → Impulse: priority maps correctly
        assert_eq!(impulse.priority, 200);
        assert_eq!(impulse.payload, "threat-signal");

        // Impulse → MotorCommand: target preserved
        assert_eq!(command.target_tool, "bash");
        assert_eq!(command.urgency, 200);

        // MotorCommand → ToolClassification: classified correctly
        assert_eq!(classification.tool_name, "bash");
        assert_eq!(classification.muscle_type, MuscleType::Skeletal);
    }

    #[test]
    fn test_all_threat_levels_map() {
        let levels = [
            (ThreatLevel::Trace, 0u8),
            (ThreatLevel::Low, 50),
            (ThreatLevel::Medium, 100),
            (ThreatLevel::High, 200),
            (ThreatLevel::Critical, 255),
        ];

        for (threat, expected_priority) in levels {
            let cytokine = Cytokine::new(CytokineFamily::Il1, "test")
                .with_severity(threat);
            let impulse = cytokine_to_impulse(&cytokine);
            assert_eq!(
                impulse.priority, expected_priority,
                "ThreatLevel::{threat:?} should map to priority {expected_priority}"
            );
        }
    }
}
