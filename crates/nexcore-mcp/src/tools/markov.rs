//! Markov chain analysis and estimation MCP tools
//!
//! Unified interface for Markov chain operations: stationary distribution,
//! n-step transition probabilities, ergodicity classification, communicating
//! classes, absorbing states, entropy rate, and mean first passage time.

use crate::params::{MarkovAnalyzeParams, MarkovFromDataParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use stem_math::markov::{MarkovChain, StateClass};
use stem_math::matrix::Matrix;

/// Build a MarkovChain<String> from MCP transition input.
fn build_chain(
    states: &[String],
    transitions: &[crate::params::TransitionInput],
) -> Result<MarkovChain<String>, McpError> {
    let n = states.len();
    if n == 0 {
        return Err(McpError::invalid_params("No states provided", None));
    }

    let trans: Vec<(usize, usize, f64)> = transitions
        .iter()
        .map(|t| (t.from, t.to, t.probability))
        .collect();

    for &(from, to, _) in &trans {
        if from >= n || to >= n {
            return Err(McpError::invalid_params(
                format!("Transition ({from}, {to}) references state beyond count {n}"),
                None,
            ));
        }
    }

    MarkovChain::from_transitions(states.to_vec(), &trans).ok_or_else(|| {
        McpError::invalid_params("Failed to construct Markov chain from transitions", None)
    })
}

/// Analyze a Markov chain.
pub fn markov_analyze(params: MarkovAnalyzeParams) -> Result<CallToolResult, McpError> {
    let mc = build_chain(&params.states, &params.transitions)?;

    let mode = params.analysis.to_lowercase();
    match mode.as_str() {
        "summary" => analyze_summary(&mc),
        "stationary" => analyze_stationary(&mc),
        "n_step" => analyze_n_step(&mc, params.from_state, params.to_state, params.steps),
        "classify" => analyze_classify(&mc),
        "classes" => analyze_classes(&mc),
        "ergodicity" => analyze_ergodicity(&mc),
        "entropy" => analyze_entropy(&mc),
        "mfpt" => analyze_mfpt(&mc, params.from_state, params.to_state),
        other => Err(McpError::invalid_params(
            format!(
                "Unknown analysis '{other}'. Use: summary, stationary, n_step, classify, classes, ergodicity, entropy, mfpt"
            ),
            None,
        )),
    }
}

fn analyze_summary(mc: &MarkovChain<String>) -> Result<CallToolResult, McpError> {
    let summary = mc.summary();
    let pi_data: Vec<_> = mc
        .states()
        .iter()
        .zip(summary.stationary_distribution.iter())
        .map(|(s, &p)| json!({"state": s, "probability": round6(p)}))
        .collect();

    let result = json!({
        "analysis": "summary",
        "state_count": summary.state_count,
        "is_ergodic": summary.is_ergodic,
        "is_irreducible": summary.is_irreducible,
        "is_aperiodic": summary.is_aperiodic,
        "communicating_class_count": summary.communicating_class_count,
        "absorbing_state_count": summary.absorbing_state_count,
        "stationary_distribution": pi_data,
        "stationary_confidence": round6(summary.stationary_confidence),
        "entropy_rate": round6(summary.entropy_rate),
        "grounding": "ς(State) + σ(Sequence) + N(Quantity) + ρ(Recursion)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_stationary(mc: &MarkovChain<String>) -> Result<CallToolResult, McpError> {
    let pi = mc.stationary_distribution(1000, 1e-10);
    let pi_data: Vec<_> = mc
        .states()
        .iter()
        .zip(pi.value.iter())
        .map(|(s, &p)| json!({"state": s, "probability": round6(p)}))
        .collect();

    let result = json!({
        "analysis": "stationary",
        "distribution": pi_data,
        "confidence": round6(pi.confidence.value()),
        "grounding": "ρ(Recursion) + N(Quantity) + ς(State)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_n_step(
    mc: &MarkovChain<String>,
    from: Option<usize>,
    to: Option<usize>,
    steps: u32,
) -> Result<CallToolResult, McpError> {
    let from_idx =
        from.ok_or_else(|| McpError::invalid_params("n_step requires from_state", None))?;
    let to_idx = to.ok_or_else(|| McpError::invalid_params("n_step requires to_state", None))?;

    let prob = mc
        .n_step_probability(from_idx, to_idx, steps)
        .ok_or_else(|| McpError::invalid_params("Invalid state indices", None))?;

    let from_label = mc.state(from_idx).cloned().unwrap_or_default();
    let to_label = mc.state(to_idx).cloned().unwrap_or_default();

    let result = json!({
        "analysis": "n_step",
        "from": from_label,
        "to": to_label,
        "steps": steps,
        "probability": round6(prob),
        "grounding": "ρ(Recursion) + N(Quantity) + σ(Sequence)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_classify(mc: &MarkovChain<String>) -> Result<CallToolResult, McpError> {
    let classified = mc.classify_states();
    let class_data: Vec<_> = classified
        .iter()
        .map(|&(idx, class)| {
            let label = mc.state(idx).cloned().unwrap_or_default();
            json!({
                "state": label,
                "index": idx,
                "class": format!("{class:?}"),
            })
        })
        .collect();

    let result = json!({
        "analysis": "classify",
        "state_count": mc.state_count(),
        "classifications": class_data,
        "grounding": "ς(State) + ∂(Boundary)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_classes(mc: &MarkovChain<String>) -> Result<CallToolResult, McpError> {
    let classes = mc.communicating_classes();
    let class_data: Vec<_> = classes
        .iter()
        .enumerate()
        .map(|(i, class)| {
            let labels: Vec<_> = class
                .states
                .iter()
                .filter_map(|&idx| mc.state(idx).cloned())
                .collect();
            json!({
                "class": i,
                "type": format!("{:?}", class.class_type),
                "size": class.states.len(),
                "states": labels,
            })
        })
        .collect();

    let result = json!({
        "analysis": "classes",
        "class_count": classes.len(),
        "classes": class_data,
        "grounding": "∂(Boundary) + μ(Mapping) + ς(State)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_ergodicity(mc: &MarkovChain<String>) -> Result<CallToolResult, McpError> {
    let result = json!({
        "analysis": "ergodicity",
        "is_ergodic": mc.is_ergodic(),
        "is_irreducible": mc.is_irreducible(),
        "is_aperiodic": mc.is_aperiodic(),
        "interpretation": if mc.is_ergodic() {
            "Chain is ergodic: unique stationary distribution exists and equals limiting distribution"
        } else if !mc.is_irreducible() {
            "Chain is reducible: multiple communicating classes exist"
        } else {
            "Chain is irreducible but periodic: stationary distribution exists but limiting may not"
        },
        "grounding": "ρ(Recursion) + ∂(Boundary) + ς(State)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_entropy(mc: &MarkovChain<String>) -> Result<CallToolResult, McpError> {
    let h = mc.entropy_rate();
    let max_entropy = (mc.state_count() as f64).log2();

    let result = json!({
        "analysis": "entropy",
        "entropy_rate": round6(h.value),
        "max_entropy": round6(max_entropy),
        "normalized_entropy": round6(if max_entropy > 0.0 { h.value / max_entropy } else { 0.0 }),
        "confidence": round6(h.confidence.value()),
        "interpretation": if h.value < 0.1 {
            "Near-deterministic: transitions are highly predictable"
        } else if h.value > max_entropy * 0.8 {
            "Near-uniform: transitions are highly unpredictable"
        } else {
            "Moderate predictability"
        },
        "grounding": "ν(Frequency) + N(Quantity) + ς(State)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

fn analyze_mfpt(
    mc: &MarkovChain<String>,
    from: Option<usize>,
    to: Option<usize>,
) -> Result<CallToolResult, McpError> {
    let from_idx =
        from.ok_or_else(|| McpError::invalid_params("mfpt requires from_state", None))?;
    let to_idx = to.ok_or_else(|| McpError::invalid_params("mfpt requires to_state", None))?;

    let from_label = mc.state(from_idx).cloned().unwrap_or_default();
    let to_label = mc.state(to_idx).cloned().unwrap_or_default();

    match mc.mean_first_passage_time(from_idx, to_idx, 10000) {
        Some(time) => {
            let result = json!({
                "analysis": "mfpt",
                "from": from_label,
                "to": to_label,
                "mean_steps": round6(time),
                "interpretation": format!("Expected {:.1} steps to reach {} from {}", time, to_label, from_label),
                "grounding": "σ(Sequence) + N(Quantity) + ς(State)",
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        None => {
            let result = json!({
                "analysis": "mfpt",
                "from": from_label,
                "to": to_label,
                "mean_steps": null,
                "interpretation": format!("{} is unreachable from {}", to_label, from_label),
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}

/// Estimate a Markov chain from observed state sequences.
pub fn markov_from_data(params: MarkovFromDataParams) -> Result<CallToolResult, McpError> {
    if params.states.is_empty() {
        return Err(McpError::invalid_params("No states provided", None));
    }
    if params.sequences.is_empty() {
        return Err(McpError::invalid_params("No sequences provided", None));
    }

    let mc =
        MarkovChain::from_observed_data(params.states, &params.sequences).ok_or_else(|| {
            McpError::invalid_params("Failed to estimate Markov chain from data", None)
        })?;

    let summary = mc.summary();

    // Build transition matrix display
    let n = mc.state_count();
    let mut transitions = Vec::new();
    for i in 0..n {
        for j in 0..n {
            let p = mc.transition_probability(i, j).unwrap_or(0.0);
            if p > 0.0 {
                transitions.push(json!({
                    "from": mc.state(i).cloned().unwrap_or_default(),
                    "to": mc.state(j).cloned().unwrap_or_default(),
                    "probability": round6(p),
                }));
            }
        }
    }

    let pi_data: Vec<_> = mc
        .states()
        .iter()
        .zip(summary.stationary_distribution.iter())
        .map(|(s, &p)| json!({"state": s, "probability": round6(p)}))
        .collect();

    let total_sequences = params.sequences.len();
    let total_transitions: usize = params
        .sequences
        .iter()
        .map(|s| s.len().saturating_sub(1))
        .sum();

    let result = json!({
        "analysis": "from_data",
        "state_count": n,
        "sequences_analyzed": total_sequences,
        "total_transitions": total_transitions,
        "estimated_transitions": transitions,
        "is_ergodic": summary.is_ergodic,
        "stationary_distribution": pi_data,
        "entropy_rate": round6(summary.entropy_rate),
        "grounding": "ν(Frequency) + ς(State) + N(Quantity) + σ(Sequence)",
    });
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Round to 6 decimal places for JSON output.
fn round6(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{MarkovAnalyzeParams, MarkovFromDataParams, TransitionInput};

    fn weather_transitions() -> Vec<TransitionInput> {
        vec![
            TransitionInput {
                from: 0,
                to: 0,
                probability: 0.8,
            },
            TransitionInput {
                from: 0,
                to: 1,
                probability: 0.2,
            },
            TransitionInput {
                from: 1,
                to: 0,
                probability: 0.4,
            },
            TransitionInput {
                from: 1,
                to: 1,
                probability: 0.6,
            },
        ]
    }

    fn drug_pipeline_transitions() -> Vec<TransitionInput> {
        vec![
            TransitionInput {
                from: 0,
                to: 1,
                probability: 0.6,
            },
            TransitionInput {
                from: 0,
                to: 5,
                probability: 0.4,
            },
            TransitionInput {
                from: 1,
                to: 2,
                probability: 0.5,
            },
            TransitionInput {
                from: 1,
                to: 5,
                probability: 0.5,
            },
            TransitionInput {
                from: 2,
                to: 3,
                probability: 0.4,
            },
            TransitionInput {
                from: 2,
                to: 5,
                probability: 0.6,
            },
            TransitionInput {
                from: 3,
                to: 4,
                probability: 0.6,
            },
            TransitionInput {
                from: 3,
                to: 5,
                probability: 0.4,
            },
            TransitionInput {
                from: 4,
                to: 4,
                probability: 1.0,
            },
            TransitionInput {
                from: 5,
                to: 5,
                probability: 1.0,
            },
        ]
    }

    #[test]
    fn test_markov_summary() {
        let params = MarkovAnalyzeParams {
            states: vec!["Sunny".to_string(), "Rainy".to_string()],
            transitions: weather_transitions(),
            analysis: "summary".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_stationary() {
        let params = MarkovAnalyzeParams {
            states: vec!["Sunny".to_string(), "Rainy".to_string()],
            transitions: weather_transitions(),
            analysis: "stationary".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_n_step() {
        let params = MarkovAnalyzeParams {
            states: vec!["Sunny".to_string(), "Rainy".to_string()],
            transitions: weather_transitions(),
            analysis: "n_step".to_string(),
            from_state: Some(0),
            to_state: Some(1),
            steps: 5,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_classify() {
        let params = MarkovAnalyzeParams {
            states: vec![
                "Preclinical".to_string(),
                "Phase1".to_string(),
                "Phase2".to_string(),
                "Phase3".to_string(),
                "Approved".to_string(),
                "Failed".to_string(),
            ],
            transitions: drug_pipeline_transitions(),
            analysis: "classify".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_classes() {
        let params = MarkovAnalyzeParams {
            states: vec![
                "Preclinical".to_string(),
                "Phase1".to_string(),
                "Phase2".to_string(),
                "Phase3".to_string(),
                "Approved".to_string(),
                "Failed".to_string(),
            ],
            transitions: drug_pipeline_transitions(),
            analysis: "classes".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_ergodicity() {
        let params = MarkovAnalyzeParams {
            states: vec!["Sunny".to_string(), "Rainy".to_string()],
            transitions: weather_transitions(),
            analysis: "ergodicity".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_entropy() {
        let params = MarkovAnalyzeParams {
            states: vec!["Sunny".to_string(), "Rainy".to_string()],
            transitions: weather_transitions(),
            analysis: "entropy".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_mfpt() {
        let params = MarkovAnalyzeParams {
            states: vec!["Sunny".to_string(), "Rainy".to_string()],
            transitions: weather_transitions(),
            analysis: "mfpt".to_string(),
            from_state: Some(0),
            to_state: Some(1),
            steps: 1,
        };
        assert!(markov_analyze(params).is_ok());
    }

    #[test]
    fn test_markov_from_data() {
        let params = MarkovFromDataParams {
            states: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            sequences: vec![vec![0, 1, 2, 0, 1], vec![0, 0, 1, 2]],
        };
        assert!(markov_from_data(params).is_ok());
    }

    #[test]
    fn test_invalid_analysis() {
        let params = MarkovAnalyzeParams {
            states: vec!["A".to_string()],
            transitions: vec![TransitionInput {
                from: 0,
                to: 0,
                probability: 1.0,
            }],
            analysis: "invalid".to_string(),
            from_state: None,
            to_state: None,
            steps: 1,
        };
        assert!(markov_analyze(params).is_err());
    }
}
