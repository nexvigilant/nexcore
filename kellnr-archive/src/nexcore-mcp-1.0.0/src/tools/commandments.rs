//! Commandment tools: The 15 Human Commandments verification
//!
//! Exposes the 15 Commandments as MCP tools for governance verification:
//! - Commandments 1-10: Original Foundation
//! - Commandments 11-15: Truth Expansion (Epistemic Rigor)

use crate::params::{
    CommandmentAuditParams, CommandmentInfoParams, CommandmentListParams, CommandmentVerifyParams,
};
use nexcore_vigilance::primitives::governance::{
    Commandment, CommandmentAudit, CommandmentCategory, CompilationProof, FalsifiabilityProof,
    HumanCommandments, OracleCount, PrecedentHash, ProvenanceChain, SourceId, VerificationContext,
};
use nexcore_vigilance::primitives::measurement::Confidence;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ============================================================================
// COMMANDMENT VERIFICATION TOOLS
// ============================================================================

/// Verify a single commandment against proof
pub fn commandment_verify(params: CommandmentVerifyParams) -> Result<CallToolResult, McpError> {
    let commandment = parse_commandment(&params.commandment)?;
    let commandments = HumanCommandments::new(true);
    let verdict = commandments.verify_action(commandment, params.proof_provided);

    let result = json!({
        "commandment": format!("{:?}", commandment),
        "number": commandment as u8,
        "category": format!("{:?}", commandment.category()),
        "proof_provided": params.proof_provided,
        "verdict": format!("{:?}", verdict),
        "passed": verdict == nexcore_vigilance::primitives::governance::Verdict::Permitted,
    });

    Ok(CallToolResult::success(vec![Content::json(&result)?]))
}

/// Get information about a commandment
pub fn commandment_info(params: CommandmentInfoParams) -> Result<CallToolResult, McpError> {
    let commandment = parse_commandment(&params.commandment)?;

    let description = get_commandment_description(commandment);
    let grounding = get_commandment_grounding(commandment);

    let result = json!({
        "commandment": format!("{:?}", commandment),
        "number": commandment as u8,
        "category": format!("{:?}", commandment.category()),
        "description": description,
        "t1_grounding": grounding,
        "enforcement": get_commandment_enforcement(commandment),
    });

    Ok(CallToolResult::success(vec![Content::json(&result)?]))
}

/// List commandments by category
pub fn commandment_list(params: CommandmentListParams) -> Result<CallToolResult, McpError> {
    let all_commandments = Commandment::all();

    let filtered: Vec<_> = if params.category.to_lowercase() == "all" {
        all_commandments
            .iter()
            .map(|c| commandment_to_json(*c))
            .collect()
    } else {
        let category = parse_category(&params.category)?;
        all_commandments
            .iter()
            .filter(|c| c.category() == category)
            .map(|c| commandment_to_json(*c))
            .collect()
    };

    let result = json!({
        "category": params.category,
        "count": filtered.len(),
        "commandments": filtered,
    });

    Ok(CallToolResult::success(vec![Content::json(&result)?]))
}

/// Full audit of all 15 commandments
pub fn commandment_audit(params: CommandmentAuditParams) -> Result<CallToolResult, McpError> {
    let commandments = HumanCommandments::new(true);

    // Build verification context from params
    let context = VerificationContext {
        grounding_proof: params.grounding_proof,
        owner_identified: params.owner_identified,
        audit_trail_exists: params.audit_trail_exists,
        sensing_active: params.sensing_active,
        correction_mechanism: params.correction_mechanism,
        state_public: params.state_public,
        persistence_guaranteed: params.persistence_guaranteed,
        fair_market: params.fair_market,
        human_override_available: params.human_override_available,
        codex_compliant: params.codex_compliant,
        falsifiability: if params.has_falsifiability_test {
            Some(FalsifiabilityProof {
                falsifying_condition: "provided".into(),
                test_exists: true,
                test_executed: false,
                survived: None,
            })
        } else {
            None
        },
        provenance: if params.has_provenance {
            Some(ProvenanceChain::single(
                SourceId("audit_context".into()),
                Confidence::new(1.0),
            ))
        } else {
            None
        },
        oracle_consensus: if params.oracle_total > 0 {
            Some(OracleCount {
                agreeing: params.oracle_agreeing,
                total: params.oracle_total,
            })
        } else {
            None
        },
        precedent_hash: if params.has_precedent {
            Some(PrecedentHash::genesis())
        } else {
            None
        },
        compilation: if params.compiled {
            Some(CompilationProof {
                source_hash: "audit".into(),
                compiler_version: "prima-0.1.0".into(),
                compiled: params.compiled,
                type_checked: params.type_checked,
                effects_verified: params.effects_verified,
            })
        } else {
            None
        },
    };

    let audit = commandments.verify_all(&context);

    let results_json: Vec<_> = audit
        .results
        .iter()
        .map(|(c, v)| {
            json!({
                "commandment": format!("{:?}", c),
                "number": *c as u8,
                "verdict": format!("{:?}", v),
            })
        })
        .collect();

    let result = json!({
        "passed": audit.passed,
        "flagged": audit.flagged,
        "rejected": audit.rejected,
        "overall": format!("{:?}", audit.overall),
        "results": results_json,
        "categories": {
            "epistemic": count_category(&audit, CommandmentCategory::Epistemic),
            "authority": count_category(&audit, CommandmentCategory::Authority),
            "observability": count_category(&audit, CommandmentCategory::Observability),
            "process": count_category(&audit, CommandmentCategory::Process),
            "integrity": count_category(&audit, CommandmentCategory::Integrity),
        }
    });

    Ok(CallToolResult::success(vec![Content::json(&result)?]))
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn parse_commandment(s: &str) -> Result<Commandment, McpError> {
    // Try parsing as number first
    if let Ok(num) = s.parse::<u8>() {
        return match num {
            1 => Ok(Commandment::TruthInGrounding),
            2 => Ok(Commandment::ResponsibilityOfCommand),
            3 => Ok(Commandment::Auditability),
            4 => Ok(Commandment::Vigilance),
            5 => Ok(Commandment::Correction),
            6 => Ok(Commandment::Transparency),
            7 => Ok(Commandment::RespectForState),
            8 => Ok(Commandment::FairnessInMarkets),
            9 => Ok(Commandment::HumanOversight),
            10 => Ok(Commandment::SupremeLaw),
            11 => Ok(Commandment::Falsifiability),
            12 => Ok(Commandment::Provenance),
            13 => Ok(Commandment::Consensus),
            14 => Ok(Commandment::Precedent),
            15 => Ok(Commandment::Compilation),
            _ => Err(McpError::invalid_params(
                format!("Invalid commandment number: {}. Must be 1-15.", num),
                None,
            )),
        };
    }

    // Try parsing as name
    let lower = s.to_lowercase();
    match lower.as_str() {
        "truthingrounding" | "truth" | "grounding" => Ok(Commandment::TruthInGrounding),
        "responsibilityofcommand" | "responsibility" | "command" => {
            Ok(Commandment::ResponsibilityOfCommand)
        }
        "auditability" | "audit" => Ok(Commandment::Auditability),
        "vigilance" => Ok(Commandment::Vigilance),
        "correction" | "correct" => Ok(Commandment::Correction),
        "transparency" | "transparent" => Ok(Commandment::Transparency),
        "respectforstate" | "state" | "persist" => Ok(Commandment::RespectForState),
        "fairnessinmarkets" | "fairness" | "markets" => Ok(Commandment::FairnessInMarkets),
        "humanoversight" | "oversight" | "human" => Ok(Commandment::HumanOversight),
        "supremelaw" | "supreme" | "codex" => Ok(Commandment::SupremeLaw),
        "falsifiability" | "falsifiable" | "disprove" => Ok(Commandment::Falsifiability),
        "provenance" | "origin" | "source" => Ok(Commandment::Provenance),
        "consensus" | "oracle" | "quorum" => Ok(Commandment::Consensus),
        "precedent" | "chain" | "judicial" => Ok(Commandment::Precedent),
        "compilation" | "compile" | "prima" => Ok(Commandment::Compilation),
        _ => Err(McpError::invalid_params(
            format!("Unknown commandment: {}", s),
            None,
        )),
    }
}

fn parse_category(s: &str) -> Result<CommandmentCategory, McpError> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "epistemic" | "truth" | "knowledge" => Ok(CommandmentCategory::Epistemic),
        "authority" | "command" | "control" => Ok(CommandmentCategory::Authority),
        "observability" | "observe" | "monitoring" => Ok(CommandmentCategory::Observability),
        "process" | "procedure" | "flow" => Ok(CommandmentCategory::Process),
        "integrity" | "state" | "stability" => Ok(CommandmentCategory::Integrity),
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown category: {}. Valid: Epistemic, Authority, Observability, Process, Integrity",
                s
            ),
            None,
        )),
    }
}

fn commandment_to_json(c: Commandment) -> serde_json::Value {
    json!({
        "name": format!("{:?}", c),
        "number": c as u8,
        "category": format!("{:?}", c.category()),
        "description": get_commandment_description(c),
    })
}

fn get_commandment_description(c: Commandment) -> &'static str {
    match c {
        Commandment::TruthInGrounding => "Proof required for all claims",
        Commandment::ResponsibilityOfCommand => "Every action has an owner",
        Commandment::Auditability => "All state must be observable",
        Commandment::Vigilance => "Continuous sensing required",
        Commandment::Correction => "Error → fix cycle must exist",
        Commandment::Transparency => "No hidden state allowed",
        Commandment::RespectForState => "Persistence is sacred",
        Commandment::FairnessInMarkets => "No asymmetry abuse",
        Commandment::HumanOversight => "Human veto power preserved",
        Commandment::SupremeLaw => "Codex supersedes all",
        Commandment::Falsifiability => "Claims must be disprovable",
        Commandment::Provenance => "All data has origin chain",
        Commandment::Consensus => "High-stakes require multi-oracle agreement",
        Commandment::Precedent => "Judgments form immutable chain",
        Commandment::Compilation => "Code that compiles is proof",
    }
}

fn get_commandment_grounding(c: Commandment) -> &'static str {
    match c {
        Commandment::TruthInGrounding => "T1: Proof required → Existence (∃)",
        Commandment::ResponsibilityOfCommand => "T1: Owner → Location (λ)",
        Commandment::Auditability => "T1: Observable → State (ς)",
        Commandment::Vigilance => "T1: Sensing → Frequency (ν)",
        Commandment::Correction => "T1: Fix cycle → Causality (→)",
        Commandment::Transparency => "T1: Public → Void of hidden (∅)",
        Commandment::RespectForState => "T1: Persistence (π)",
        Commandment::FairnessInMarkets => "T1: Comparison (κ)",
        Commandment::HumanOversight => "T1: Boundary (∂)",
        Commandment::SupremeLaw => "T1: Mapping (μ)",
        Commandment::Falsifiability => "T1: ¬(¬P) ≠ P → Detect",
        Commandment::Provenance => "T1: Sequence (σ)",
        Commandment::Consensus => "T1: Quantity (N) → k-of-n",
        Commandment::Precedent => "T1: Hash chain → Recursion (ρ)",
        Commandment::Compilation => "T1: Compute → Proof",
    }
}

fn get_commandment_enforcement(c: Commandment) -> &'static str {
    match c {
        Commandment::TruthInGrounding => "JudicialReviewEngine",
        Commandment::ResponsibilityOfCommand => "ExecutiveHierarchy",
        Commandment::Auditability => "StateGuardian",
        Commandment::Vigilance => "Guardian homeostasis loop",
        Commandment::Correction => "Federalist XII",
        Commandment::Transparency => "PublicHealthAct",
        Commandment::RespectForState => "SocialSecurityAct",
        Commandment::FairnessInMarkets => "SecuritiesAct",
        Commandment::HumanOversight => "Executive Unity",
        Commandment::SupremeLaw => "SupremeCompiler",
        Commandment::Falsifiability => "FalsificationEngine",
        Commandment::Provenance => "GroundingTreaty",
        Commandment::Consensus => "OracleQuorum",
        Commandment::Precedent => "PrecedentHash chain",
        Commandment::Compilation => "PrimaOracle",
    }
}

fn count_category(audit: &CommandmentAudit, category: CommandmentCategory) -> serde_json::Value {
    let (passed, flagged, rejected) = audit.results.iter().fold((0, 0, 0), |acc, (c, v)| {
        if c.category() == category {
            match v {
                nexcore_vigilance::primitives::governance::Verdict::Permitted => {
                    (acc.0 + 1, acc.1, acc.2)
                }
                nexcore_vigilance::primitives::governance::Verdict::Flagged => {
                    (acc.0, acc.1 + 1, acc.2)
                }
                nexcore_vigilance::primitives::governance::Verdict::Rejected => {
                    (acc.0, acc.1, acc.2 + 1)
                }
            }
        } else {
            acc
        }
    });

    json!({
        "passed": passed,
        "flagged": flagged,
        "rejected": rejected,
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commandment_by_number() {
        assert_eq!(
            parse_commandment("1").ok(),
            Some(Commandment::TruthInGrounding)
        );
        assert_eq!(parse_commandment("15").ok(), Some(Commandment::Compilation));
        assert!(parse_commandment("0").is_err());
        assert!(parse_commandment("16").is_err());
    }

    #[test]
    fn test_parse_commandment_by_name() {
        assert_eq!(
            parse_commandment("Falsifiability").ok(),
            Some(Commandment::Falsifiability)
        );
        assert_eq!(
            parse_commandment("consensus").ok(),
            Some(Commandment::Consensus)
        );
        assert_eq!(
            parse_commandment("prima").ok(),
            Some(Commandment::Compilation)
        );
    }

    #[test]
    fn test_commandment_verify() {
        let result = commandment_verify(CommandmentVerifyParams {
            commandment: "TruthInGrounding".into(),
            proof_provided: true,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_commandment_list() {
        let result = commandment_list(CommandmentListParams {
            category: "Epistemic".into(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_commandment_audit() {
        let result = commandment_audit(CommandmentAuditParams {
            grounding_proof: true,
            owner_identified: true,
            audit_trail_exists: true,
            sensing_active: true,
            correction_mechanism: true,
            state_public: true,
            persistence_guaranteed: true,
            fair_market: true,
            human_override_available: true,
            codex_compliant: true,
            has_falsifiability_test: true,
            has_provenance: true,
            oracle_agreeing: 3,
            oracle_total: 5,
            has_precedent: true,
            compiled: true,
            type_checked: true,
            effects_verified: true,
        });
        assert!(result.is_ok());
    }
}
