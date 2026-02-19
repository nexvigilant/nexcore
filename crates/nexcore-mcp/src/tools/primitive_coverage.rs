//! Primitive Coverage MCP tools — T1 Lex Primitiva coverage analysis.
//!
//! Analyzes Rust source code for coverage of all 16 T1 primitives.
//! Each primitive has detection patterns (Rust syntax markers).

use crate::params::primitive_coverage::{
    PrimitiveCoverageCheckParams, PrimitiveCoverageRulesParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

struct PrimitiveRule {
    symbol: &'static str,
    name: &'static str,
    patterns: &'static [&'static str],
}

const RULES: &[PrimitiveRule] = &[
    PrimitiveRule {
        symbol: "→",
        name: "Causality",
        patterns: &["fn ", "->", "=>", "callback", "handler", "emit", "trigger"],
    },
    PrimitiveRule {
        symbol: "N",
        name: "Quantity",
        patterns: &["u32", "u64", "f64", "usize", "i32", "i64", "count", "len()"],
    },
    PrimitiveRule {
        symbol: "∃",
        name: "Existence",
        patterns: &["new(", "::new", "Some(", "create", "init", "builder"],
    },
    PrimitiveRule {
        symbol: "κ",
        name: "Comparison",
        patterns: &[
            "==",
            "!=",
            "match ",
            "if let",
            "Ord",
            "PartialEq",
            "cmp",
            ">=",
            "<=",
        ],
    },
    PrimitiveRule {
        symbol: "ς",
        name: "State",
        patterns: &[
            "struct ", "mut ", "Mutex", "Cell", "RefCell", "state", "status",
        ],
    },
    PrimitiveRule {
        symbol: "μ",
        name: "Mapping",
        patterns: &[
            ".map(",
            "From",
            "Into",
            ".and_then(",
            "transform",
            "convert",
        ],
    },
    PrimitiveRule {
        symbol: "σ",
        name: "Sequence",
        patterns: &[
            "Iterator",
            ".iter()",
            "Vec<",
            ".collect()",
            ".chain(",
            "for ",
        ],
    },
    PrimitiveRule {
        symbol: "ρ",
        name: "Recursion",
        patterns: &["Box<", "recursive", "self.", "Rc<", "Arc<"],
    },
    PrimitiveRule {
        symbol: "∅",
        name: "Void",
        patterns: &["None", "()", "Option::None", "PhantomData", "Default", "!"],
    },
    PrimitiveRule {
        symbol: "∂",
        name: "Boundary",
        patterns: &[
            "Result<", "Err(", "?;", "boundary", "limit", "max_", "min_", "validate",
        ],
    },
    PrimitiveRule {
        symbol: "ν",
        name: "Frequency",
        patterns: &[
            "count",
            "rate",
            "poll",
            "interval",
            "timer",
            "frequency",
            "per_",
        ],
    },
    PrimitiveRule {
        symbol: "λ",
        name: "Location",
        patterns: &["Path", "url", "index", "offset", "position", "&str", "key"],
    },
    PrimitiveRule {
        symbol: "π",
        name: "Persistence",
        patterns: &[
            "write", "save", "store", "persist", "static ", "log", "db", "file",
        ],
    },
    PrimitiveRule {
        symbol: "∝",
        name: "Irreversibility",
        patterns: &["drop", "consume", "hash", "digest", "into_", "take("],
    },
    PrimitiveRule {
        symbol: "Σ",
        name: "Sum",
        patterns: &["enum ", "Either", "match ", "variant", "union"],
    },
    PrimitiveRule {
        symbol: "×",
        name: "Product",
        patterns: &["struct ", "tuple", ".zip(", "(_, _)", "pair"],
    },
];

/// Analyze T1 primitive coverage in source code.
pub fn check(params: PrimitiveCoverageCheckParams) -> Result<CallToolResult, McpError> {
    let source = if params.is_path.unwrap_or(false) {
        let path = Path::new(&params.source);
        if path.is_dir() {
            collect_dir_source(path)
        } else {
            std::fs::read_to_string(path)
                .map_err(|e| McpError::invalid_params(format!("Cannot read file: {}", e), None))?
        }
    } else {
        params.source.clone()
    };

    let mut coverage = Vec::new();
    let mut covered_count = 0u32;
    let total = RULES.len() as u32;

    for rule in RULES {
        let matches: Vec<&str> = rule
            .patterns
            .iter()
            .filter(|&&p| source.contains(p))
            .copied()
            .collect();

        let is_covered = !matches.is_empty();
        if is_covered {
            covered_count += 1;
        }

        coverage.push(json!({
            "symbol": rule.symbol,
            "name": rule.name,
            "covered": is_covered,
            "matched_patterns": matches,
            "total_patterns": rule.patterns.len(),
        }));
    }

    let coverage_ratio = covered_count as f64 / total as f64;
    let missing: Vec<&serde_json::Value> =
        coverage.iter().filter(|c| c["covered"] == false).collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "coverage": (coverage_ratio * 100.0).round() / 100.0,
            "covered": covered_count,
            "total": total,
            "grade": if coverage_ratio >= 0.9 { "A" }
                     else if coverage_ratio >= 0.75 { "B" }
                     else if coverage_ratio >= 0.5 { "C" }
                     else { "D" },
            "missing": missing,
            "primitives": coverage,
        })
        .to_string(),
    )]))
}

/// Get the detection rules for each T1 primitive.
pub fn rules(_params: PrimitiveCoverageRulesParams) -> Result<CallToolResult, McpError> {
    let rules: Vec<serde_json::Value> = RULES
        .iter()
        .map(|r| {
            json!({
                "symbol": r.symbol,
                "name": r.name,
                "detection_patterns": r.patterns,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({ "rules": rules, "count": rules.len() }).to_string(),
    )]))
}

fn collect_dir_source(dir: &Path) -> String {
    let mut content = String::new();
    collect_rs_rec(dir, &mut content);
    content
}

fn collect_rs_rec(dir: &Path, content: &mut String) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.file_name().map_or(true, |n| n != "target") {
                collect_rs_rec(&path, content);
            } else if path.extension().map_or(false, |e| e == "rs") {
                if let Ok(src) = std::fs::read_to_string(&path) {
                    content.push_str(&src);
                    content.push('\n');
                }
            }
        }
    }
}
