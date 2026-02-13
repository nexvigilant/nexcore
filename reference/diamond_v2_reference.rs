//! Diamond v2 Scoring Module
//!
//! Validates skills against the Diamond v2 Machine Specification standard.
//! Based on the SMST v2 taxonomy with 8 components and 90% threshold.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Component weights from SMST v2 taxonomy (sum = 100)
const WEIGHTS: ComponentWeights = ComponentWeights {
    inputs: 10,
    outputs: 10,
    state: 10,
    operator_mode: 15,
    performance: 10,
    invariants: 15,
    failure_modes: 15,
    telemetry: 15,
};

/// Diamond threshold score
const DIAMOND_THRESHOLD: u32 = 90;

struct ComponentWeights {
    inputs: u32,
    outputs: u32,
    state: u32,
    operator_mode: u32,
    performance: u32,
    invariants: u32,
    failure_modes: u32,
    telemetry: u32,
}

/// Result of Diamond v2 validation
#[derive(Debug, Serialize, Deserialize)]
pub struct DiamondScore {
    pub skill_name: String,
    pub total_score: u32,
    pub diamond_ready: bool,
    pub has_machine_spec: bool,
    pub component_scores: ComponentScores,
    pub gaps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentScores {
    pub inputs: u32,
    pub outputs: u32,
    pub state: u32,
    pub operator_mode: u32,
    pub performance: u32,
    pub invariants: u32,
    pub failure_modes: u32,
    pub telemetry: u32,
}

/// Score a SKILL.md file against Diamond v2 requirements
pub fn score_skill(skill_path: &Path) -> Result<DiamondScore, String> {
    let content = std::fs::read_to_string(skill_path)
        .map_err(|e| format!("Failed to read file: {e}"))?;

    let skill_name = skill_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check for Machine Specification section
    let has_machine_spec = content.contains("## Machine Specification") ||
                           content.contains("### INPUTS");

    if !has_machine_spec {
        return Ok(DiamondScore {
            skill_name,
            total_score: 0,
            diamond_ready: false,
            has_machine_spec: false,
            component_scores: ComponentScores {
                inputs: 0,
                outputs: 0,
                state: 0,
                operator_mode: 0,
                performance: 0,
                invariants: 0,
                failure_modes: 0,
                telemetry: 0,
            },
            gaps: vec!["Missing Machine Specification section".to_string()],
        });
    }

    // Score each component based on presence and completeness
    let mut gaps = Vec::new();

    let inputs_score = score_inputs(&content, &mut gaps);
    let outputs_score = score_outputs(&content, &mut gaps);
    let state_score = score_state(&content, &mut gaps);
    let operator_mode_score = score_operator_mode(&content, &mut gaps);
    let performance_score = score_performance(&content, &mut gaps);
    let invariants_score = score_invariants(&content, &mut gaps);
    let failure_modes_score = score_failure_modes(&content, &mut gaps);
    let telemetry_score = score_telemetry(&content, &mut gaps);

    let total_score = inputs_score + outputs_score + state_score +
                      operator_mode_score + performance_score +
                      invariants_score + failure_modes_score + telemetry_score;

    let diamond_ready = total_score >= DIAMOND_THRESHOLD;

    Ok(DiamondScore {
        skill_name,
        total_score,
        diamond_ready,
        has_machine_spec,
        component_scores: ComponentScores {
            inputs: inputs_score,
            outputs: outputs_score,
            state: state_score,
            operator_mode: operator_mode_score,
            performance: performance_score,
            invariants: invariants_score,
            failure_modes: failure_modes_score,
            telemetry: telemetry_score,
        },
        gaps,
    })
}

fn score_inputs(content: &str, gaps: &mut Vec<String>) -> u32 {
    let mut score = 0;
    let max = WEIGHTS.inputs;

    // Check for TRIGGERS section
    if content.contains("**TRIGGERS:**") || content.contains("TRIGGERS:") {
        score += (max * 40) / 100;
    } else {
        gaps.push("INPUTS: Missing TRIGGERS section".to_string());
    }

    // Check for CONTEXT section
    if content.contains("**CONTEXT:**") || content.contains("CONTEXT:") {
        score += (max * 30) / 100;
    } else {
        gaps.push("INPUTS: Missing CONTEXT section".to_string());
    }

    // Check for PARAMETERS section
    if content.contains("**PARAMETERS:**") || content.contains("PARAMETERS:") {
        score += (max * 30) / 100;
    } else {
        gaps.push("INPUTS: Missing PARAMETERS section".to_string());
    }

    score
}

fn score_outputs(content: &str, gaps: &mut Vec<String>) -> u32 {
    let mut score = 0;
    let max = WEIGHTS.outputs;

    if content.contains("### OUTPUTS") {
        score += (max * 50) / 100; // PRIMARY
        if content.contains("**ARTIFACTS:**") {
            score += (max * 25) / 100;
        }
        if content.contains("**SIDE_EFFECTS:**") {
            score += (max * 25) / 100;
        }
    } else {
        gaps.push("Missing OUTPUTS section".to_string());
    }

    score
}

fn score_state(content: &str, gaps: &mut Vec<String>) -> u32 {
    if content.contains("### STATE") {
        WEIGHTS.state
    } else {
        gaps.push("Missing STATE section".to_string());
        0
    }
}

fn score_operator_mode(content: &str, gaps: &mut Vec<String>) -> u32 {
    let mut score = 0;
    let max = WEIGHTS.operator_mode;

    if content.contains("### OPERATOR MODE") {
        // Check for lookup table reference
        if content.contains("Lookup Table:") || content.contains("**Lookup Table:**") {
            score += (max * 60) / 100;
        } else {
            gaps.push("OPERATOR MODE: Missing Lookup Table reference".to_string());
        }

        // Check for protocol
        if content.contains("Protocol:") || content.contains("**Protocol:**") {
            score += (max * 40) / 100;
        } else {
            gaps.push("OPERATOR MODE: Missing Protocol section".to_string());
        }
    } else {
        gaps.push("Missing OPERATOR MODE section".to_string());
    }

    score
}

fn score_performance(content: &str, gaps: &mut Vec<String>) -> u32 {
    let mut score = 0;
    let max = WEIGHTS.performance;

    if content.contains("### PERFORMANCE") {
        // Check for kernel delegation
        if content.contains("Kernel Delegation:") || content.contains("**Kernel Delegation:**") {
            score += (max * 60) / 100;
        } else {
            gaps.push("PERFORMANCE: Missing Kernel Delegation section".to_string());
        }

        // Check for complexity analysis
        if content.contains("Complexity Analysis:") || content.contains("**Complexity Analysis:**") {
            score += (max * 40) / 100;
        } else {
            gaps.push("PERFORMANCE: Missing Complexity Analysis section".to_string());
        }
    } else {
        gaps.push("Missing PERFORMANCE section".to_string());
    }

    score
}

fn score_invariants(content: &str, gaps: &mut Vec<String>) -> u32 {
    let mut score = 0;
    let max = WEIGHTS.invariants;

    if content.contains("### INVARIANTS") {
        // Check for PRE, POST, DURING
        if content.contains("PRE |") || content.contains("| PRE") {
            score += (max * 40) / 100;
        }
        if content.contains("POST |") || content.contains("| POST") {
            score += (max * 40) / 100;
        }
        if content.contains("DURING |") || content.contains("| DURING") {
            score += (max * 20) / 100;
        }

        if score == 0 {
            gaps.push("INVARIANTS: Missing PRE/POST/DURING conditions".to_string());
        }
    } else {
        gaps.push("Missing INVARIANTS section".to_string());
    }

    score
}

fn score_failure_modes(content: &str, gaps: &mut Vec<String>) -> u32 {
    if content.contains("### FAILURE MODES") || content.contains("### FAILURE_MODES") {
        WEIGHTS.failure_modes
    } else {
        gaps.push("Missing FAILURE MODES section".to_string());
        0
    }
}

fn score_telemetry(content: &str, gaps: &mut Vec<String>) -> u32 {
    let mut score = 0;
    let max = WEIGHTS.telemetry;

    if content.contains("### TELEMETRY") {
        // Check for events
        if content.contains("**Events:**") || content.contains("Events:") {
            score += (max * 60) / 100;
        } else {
            gaps.push("TELEMETRY: Missing Events section".to_string());
        }

        // Check for metrics
        if content.contains("**Metrics:**") || content.contains("Metrics:") {
            score += (max * 40) / 100;
        } else {
            gaps.push("TELEMETRY: Missing Metrics section".to_string());
        }
    } else {
        gaps.push("Missing TELEMETRY section".to_string());
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_empty_content() {
        let score = score_skill(Path::new("/nonexistent"));
        assert!(score.is_err());
    }
}
