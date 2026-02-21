//! Primitive extraction from text
//! Tier: T2-C (composes pattern matching)

use crate::models::ExtractedPrimitive;

/// Suggest primitives from text using heuristic pattern matching
pub fn suggest_primitives(text: &str) -> Vec<ExtractedPrimitive> {
    let lower = text.to_lowercase();
    let mut prims = Vec::new();

    // T1 primitives
    add_t1_primitives(&lower, &mut prims);

    // T2-P hook patterns
    add_t2p_primitives(&lower, &mut prims);

    // T2-C composition patterns
    add_t2c_primitives(&lower, &mut prims);

    // T3 domain patterns
    add_t3_primitives(&lower, &mut prims);

    prims
}

fn add_t1_primitives(text: &str, prims: &mut Vec<ExtractedPrimitive>) {
    if text.contains("sequence") || text.contains("iterate") || text.contains("loop") {
        prims.push(ExtractedPrimitive::t1("Sequence (σ)", "Ordered iteration"));
    }
    if text.contains("map") || text.contains("lookup") || text.contains("key") {
        prims.push(ExtractedPrimitive::t1("Mapping (μ)", "Key-value association"));
    }
    if text.contains("state") || text.contains("mutate") || text.contains("update") {
        prims.push(ExtractedPrimitive::t1("State (ς)", "State mutation"));
    }
    if text.contains("recursive") || text.contains("tree") || text.contains("traverse") {
        prims.push(ExtractedPrimitive::t1("Recursion (ρ)", "Recursive structure"));
    }
    if text.contains("filter") || text.contains("guard") || text.contains("validate") {
        prims.push(ExtractedPrimitive::t1("Existence (∃)", "Validation/filtering"));
    }
}

fn add_t2p_primitives(text: &str, prims: &mut Vec<ExtractedPrimitive>) {
    if text.contains("exit code") || text.contains("allow") || text.contains("block") {
        prims.push(ExtractedPrimitive::t2p("DecisionGate", "Hook decision pattern"));
    }
    if text.contains("timeout") || text.contains("limit") || text.contains("threshold") {
        prims.push(ExtractedPrimitive::t2p("Threshold", "Limit enforcement"));
    }
    if text.contains("transform") || text.contains("convert") || text.contains("parse") {
        prims.push(ExtractedPrimitive::t2p("Transform", "Data transformation"));
    }
}

fn add_t2c_primitives(text: &str, prims: &mut Vec<ExtractedPrimitive>) {
    if text.contains("pipeline") || text.contains("chain") || text.contains("compose") {
        prims.push(ExtractedPrimitive::t2c("Pipeline", "Composable pipeline"));
    }
}

fn add_t3_primitives(text: &str, prims: &mut Vec<ExtractedPrimitive>) {
    if text.contains("pretooluse") || text.contains("posttooluse") {
        prims.push(ExtractedPrimitive::t3("ToolInterceptor", "Tool lifecycle interception"));
    }
    if text.contains("sessionstart") || text.contains("stop") {
        prims.push(ExtractedPrimitive::t3("SessionLifecycle", "Session boundary handling"));
    }
}
