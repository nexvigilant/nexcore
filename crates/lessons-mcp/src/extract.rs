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
        prims.push(ExtractedPrimitive::t1(
            "Mapping (μ)",
            "Key-value association",
        ));
    }
    if text.contains("state") || text.contains("mutate") || text.contains("update") {
        prims.push(ExtractedPrimitive::t1("State (ς)", "State mutation"));
    }
    if text.contains("recursive") || text.contains("tree") || text.contains("traverse") {
        prims.push(ExtractedPrimitive::t1(
            "Recursion (ρ)",
            "Recursive structure",
        ));
    }
    if text.contains("filter") || text.contains("guard") || text.contains("validate") {
        prims.push(ExtractedPrimitive::t1(
            "Existence (∃)",
            "Validation/filtering",
        ));
    }
}

fn add_t2p_primitives(text: &str, prims: &mut Vec<ExtractedPrimitive>) {
    if text.contains("exit code") || text.contains("allow") || text.contains("block") {
        prims.push(ExtractedPrimitive::t2p(
            "DecisionGate",
            "Hook decision pattern",
        ));
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
        prims.push(ExtractedPrimitive::t3(
            "ToolInterceptor",
            "Tool lifecycle interception",
        ));
    }
    if text.contains("sessionstart") || text.contains("stop") {
        prims.push(ExtractedPrimitive::t3(
            "SessionLifecycle",
            "Session boundary handling",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_no_primitives() {
        assert!(suggest_primitives("").is_empty());
    }

    #[test]
    fn sequence_detected() {
        let prims = suggest_primitives("iterate over a sequence of items");
        assert!(prims.iter().any(|p| p.name.contains("Sequence")));
    }

    #[test]
    fn mapping_detected() {
        let prims = suggest_primitives("lookup the key in the map");
        assert!(prims.iter().any(|p| p.name.contains("Mapping")));
    }

    #[test]
    fn state_detected() {
        let prims = suggest_primitives("update the mutable state");
        assert!(prims.iter().any(|p| p.name.contains("State")));
    }

    #[test]
    fn threshold_detected() {
        let prims = suggest_primitives("enforce a timeout limit");
        assert!(prims.iter().any(|p| p.name.contains("Threshold")));
    }

    #[test]
    fn pipeline_detected() {
        let prims = suggest_primitives("compose a pipeline chain");
        assert!(prims.iter().any(|p| p.name.contains("Pipeline")));
    }

    #[test]
    fn tool_interceptor_detected() {
        let prims = suggest_primitives("PreToolUse hook blocks write");
        assert!(prims.iter().any(|p| p.name.contains("ToolInterceptor")));
    }

    #[test]
    fn session_lifecycle_detected() {
        let prims = suggest_primitives("SessionStart fires at boot");
        assert!(prims.iter().any(|p| p.name.contains("SessionLifecycle")));
    }

    #[test]
    fn multiple_tiers_extracted() {
        let prims = suggest_primitives("iterate and validate with timeout in a pipeline");
        let has_t1 = prims
            .iter()
            .any(|p| p.tier == crate::models::PrimitiveTier::T1);
        let has_t2p = prims
            .iter()
            .any(|p| p.tier == crate::models::PrimitiveTier::T2P);
        let has_t2c = prims
            .iter()
            .any(|p| p.tier == crate::models::PrimitiveTier::T2C);
        let tier_count = [has_t1, has_t2p, has_t2c].iter().filter(|&&b| b).count();
        assert!(tier_count >= 2, "expected multiple tiers, got {tier_count}");
    }
}
