//! Causal reasoning MCP tools.
//!
//! Build causal DAGs, run inference to find chains and risk levels,
//! and evaluate counterfactual interventions.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::reason::{ReasonCounterfactualParams, ReasonInferParams, ReasonNodeInput};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_node_type(s: &str) -> nexcore_reason::dag::NodeType {
    match s.to_lowercase().as_str() {
        "metric" => nexcore_reason::dag::NodeType::Metric,
        "pattern" => nexcore_reason::dag::NodeType::Pattern,
        "module" => nexcore_reason::dag::NodeType::Module,
        "risk" => nexcore_reason::dag::NodeType::Risk,
        "recommendation" => nexcore_reason::dag::NodeType::Recommendation,
        _ => nexcore_reason::dag::NodeType::Module,
    }
}

fn build_dag(
    nodes: &[ReasonNodeInput],
    links: &[crate::params::reason::ReasonLinkInput],
) -> Result<nexcore_reason::dag::CausalDag, nexcore_error::NexError> {
    use nexcore_reason::dag::{CausalDag, CausalLink, CausalNode, NodeId};

    let mut dag = CausalDag::new();
    for n in nodes {
        dag.add_node(CausalNode {
            id: NodeId::new(&n.id),
            label: n.label.clone(),
            node_type: parse_node_type(&n.node_type),
        });
    }
    for l in links {
        dag.add_link(CausalLink {
            from: NodeId::new(&l.from),
            to: NodeId::new(&l.to),
            strength: l.strength.unwrap_or(0.5),
            evidence: l.evidence.clone().unwrap_or_default(),
        })?;
    }
    Ok(dag)
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Build a causal DAG and run inference to find chains and risk level.
pub fn reason_infer(p: ReasonInferParams) -> Result<CallToolResult, McpError> {
    let dag = match build_dag(&p.nodes, &p.links) {
        Ok(d) => d,
        Err(e) => return err_result(&e.to_string()),
    };

    let engine = nexcore_reason::inference::InferenceEngine::new(dag);
    let chains = engine.find_causal_chains();

    let chain_items: Vec<serde_json::Value> = chains
        .iter()
        .map(|(path, score)| {
            let ids: Vec<&str> = path.iter().map(|id| id.as_str()).collect();
            json!({"path": ids, "score": score})
        })
        .collect();

    match engine.infer() {
        Ok(report) => ok_json(json!({
            "node_count": p.nodes.len(),
            "link_count": p.links.len(),
            "report": serde_json::to_value(&report).unwrap_or_default(),
            "causal_chains": chain_items,
        })),
        Err(e) => err_result(&e.to_string()),
    }
}

/// Run counterfactual analysis: "what if we remove node X?"
pub fn reason_counterfactual(p: ReasonCounterfactualParams) -> Result<CallToolResult, McpError> {
    use nexcore_reason::counterfactual::{CounterfactualEngine, Intervention};
    use nexcore_reason::dag::NodeId;

    let dag = match build_dag(&p.nodes, &p.links) {
        Ok(d) => d,
        Err(e) => return err_result(&e.to_string()),
    };

    let engine = CounterfactualEngine::new(dag);
    let intervention = Intervention::RemoveNode(NodeId::new(&p.remove_node));

    match engine.evaluate(&intervention) {
        Ok(result) => ok_json(serde_json::to_value(&result).unwrap_or_default()),
        Err(e) => err_result(&e.to_string()),
    }
}
