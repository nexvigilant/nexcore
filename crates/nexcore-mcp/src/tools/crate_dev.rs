//! Crate development framework tools — scaffold + audit
//!
//! Gold standard template for nexcore crates, extracted from `nexcore-cloud`
//! (65 tests, 35 types, 100% grounding, 0 warnings).

use crate::params::{CrateDevAuditParams, CrateDevScaffoldParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;

/// Suggest tier distribution based on total type count.
fn tier_distribution(total: usize) -> (usize, usize, usize, usize) {
    // Heuristic: ~20% T1, ~40% T2-P, ~25% T2-C, ~15% T3
    let t1 = (total as f64 * 0.20).ceil() as usize;
    let t2p = (total as f64 * 0.40).ceil() as usize;
    let t2c = (total as f64 * 0.25).ceil() as usize;
    let t3 = total.saturating_sub(t1 + t2p + t2c);
    (t1, t2p, t2c, t3)
}

/// Generate Cargo.toml template content.
fn cargo_toml_template(
    name: &str,
    domain: &str,
    desc: &str,
    t1: usize,
    t2p: usize,
    t2c: usize,
    t3: usize,
) -> String {
    let total = t1 + t2p + t2c + t3;
    format!(
        r#"[package]
name = "nexcore-{name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
description = "{desc} primitives: {total} types grounded to Lex Primitiva ({t1} T1 + {t2p} T2-P + {t2c} T2-C + {t3} T3)"
license = "MIT"
repository = "https://github.com/nexvigilant/nexcore"
keywords = ["{domain}", "primitives", "grounding"]
categories = ["data-structures"]

[lib]
path = "src/lib.rs"

[dependencies]
nexcore-lex-primitiva = {{ version = "0.1.0", path = "../nexcore-lex-primitiva", registry = "nexcore" }}
serde = {{ version = "1.0.204", features = ["derive"] }}
serde_json = {{ version = "1.0.120" }}
"#
    )
}

/// Generate lib.rs template content.
fn lib_rs_template(
    name: &str,
    domain: &str,
    t1: usize,
    t2p: usize,
    t2c: usize,
    t3: usize,
) -> String {
    let total = t1 + t2p + t2c + t3;
    format!(
        r#"//! # NexVigilant Core — {domain}
//!
//! {domain} primitives grounded to Lex Primitiva.
//!
//! ## Type Inventory ({total} types)
//!
//! | Tier | Count | Description |
//! |------|-------|-------------|
//! | T1   | {t1}     | Universal primitives |
//! | T2-P | {t2p}     | Cross-domain primitives |
//! | T2-C | {t2c}     | Composites |
//! | T3   | {t3}     | Domain-specific |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod composites;
pub mod grounding;
pub mod prelude;
pub mod primitives;
pub mod transfer;

// Re-export at crate root for convenience
pub use composites::*;
pub use primitives::*;
"#
    )
}

/// Generate primitives.rs template content.
fn primitives_rs_template(domain: &str) -> String {
    format!(
        r#"//! # {domain} Primitives
//!
//! Irreducible types: T1 Universal + T2-P Cross-Domain.

use serde::{{Deserialize, Serialize}};

// ============================================================================
// T1 Universal Primitives
// ============================================================================

// TODO: Add T1 types (1 unique primitive each)
// Example:
// /// Unique identity for a {domain} entity.
// ///
// /// Transfers: biology (genome), economics (identifier), PV (case ID).
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct Identity {{
//     pub value: String,
// }}

// ============================================================================
// T2-P Cross-Domain Primitives
// ============================================================================

// TODO: Add T2-P types (2-3 unique primitives each)
// Remember: clamp all numeric inputs at construction
// Example:
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct Threshold {{
//     pub value: f64,
//     pub label: Option<String>,
// }}
// impl Threshold {{
//     pub fn new(value: f64) -> Self {{
//         Self {{ value: value.max(0.0), label: None }}
//     }}
// }}
"#
    )
}

/// Generate composites.rs template content.
fn composites_rs_template(domain: &str) -> String {
    format!(
        r#"//! # {domain} Composites (T2-C)
//!
//! Composite types built from T1/T2-P primitives.
//! Composition method: struct-of-structs (fields are T2-P/T1 types).

use serde::{{Deserialize, Serialize}};
// use crate::primitives::{{...}};

// TODO: Add T2-C types (4-5 unique primitives each)
// Composition rule: T2-C types contain T2-P types as plain struct fields.
// Anti-pattern: trait objects, generics, or inheritance for composition.

// Example:
// /// Composed from Threshold + Metering + Permission.
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// pub struct AccessControl {{
//     pub threshold: Threshold,
//     pub metering: Metering,
//     pub permission: Permission,
// }}
"#
    )
}

/// Generate grounding.rs template content.
fn grounding_rs_template(domain: &str) -> String {
    format!(
        r#"//! # GroundsTo Implementations
//!
//! Maps all {domain} types to their Lex Primitiva compositions.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{{LexPrimitiva, PrimitiveComposition}};
// use nexcore_lex_primitiva::state_mode::StateMode;

// use crate::primitives::{{...}};
// use crate::composites::{{...}};

// TODO: Implement GroundsTo for EVERY type.
//
// Confidence calibration:
//   T1 (1 primitive):  0.95-1.00
//   T2-P (2-3):        0.85-0.92
//   T2-C (4-5):        0.78-0.85
//   T3 (6+):           0.65-0.75
//
// For types involving State (ς), declare StateMode:
//   Modal — binary state flip (active/expired)
//   Mutable — continuously changing state (counters, buffers)

// Example:
// impl GroundsTo for Identity {{
//     fn primitive_composition() -> PrimitiveComposition {{
//         PrimitiveComposition::new(vec![LexPrimitiva::Existence])
//             .with_dominant(LexPrimitiva::Existence, 1.0)
//     }}
// }}
"#
    )
}

/// Generate transfer.rs template content.
fn transfer_rs_template(domain: &str) -> String {
    format!(
        r#"//! # Cross-Domain Transfer Confidence
//!
//! Maps {domain} concepts to PV, Biology, and Economics
//! with calibrated transfer confidence scores.

use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {{
    pub source_type: &'static str,
    pub domain: &'static str,
    pub analog: &'static str,
    pub confidence: f64,
}}

/// All transfer mappings for {domain} types.
/// Rule: 3 mappings per T1/T2-P type (PV, Biology, Economics).
pub fn transfer_mappings() -> Vec<TransferMapping> {{
    vec![
        // TODO: Add 3 mappings per type
        // TransferMapping {{
        //     source_type: "Identity",
        //     domain: "PV",
        //     analog: "case identifier",
        //     confidence: 0.90,
        // }},
    ]
}}

/// Look up transfer confidence for a specific type and target domain.
pub fn transfer_confidence(source_type: &str, domain: &str) -> Option<f64> {{
    transfer_mappings()
        .iter()
        .find(|m| m.source_type == source_type && m.domain == domain)
        .map(|m| m.confidence)
}}

/// Get all mappings for a given source type.
pub fn transfers_for_type(source_type: &str) -> Vec<&TransferMapping> {{
    transfer_mappings()
        .iter()
        .filter(|m| m.source_type == source_type)
        .collect()
}}
"#
    )
}

/// Generate prelude.rs template content.
fn prelude_rs_template() -> String {
    r#"//! # Prelude
//!
//! Convenient re-exports for `use nexcore_<domain>::prelude::*`.

// T1 Universal
// pub use crate::primitives::{...};

// T2-P Cross-Domain
// pub use crate::primitives::{...};

// T2-C Composites
// pub use crate::composites::{...};

// Grounding (re-export trait)
pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
// pub use nexcore_lex_primitiva::tier::Tier;

// Transfer
pub use crate::transfer::{TransferMapping, transfer_confidence, transfer_mappings};
"#
    .to_string()
}

/// Scaffold a new nexcore crate.
pub fn scaffold_crate(params: CrateDevScaffoldParams) -> Result<CallToolResult, McpError> {
    let name = params.name.trim().to_lowercase();
    let domain = params.domain.trim();
    let description = params
        .description
        .unwrap_or_else(|| format!("{domain} primitives for NexCore"));
    let type_count = params.type_count.unwrap_or(10);
    let (t1, t2p, t2c, t3) = tier_distribution(type_count);

    let nexcore_root =
        std::env::var("NEXCORE_ROOT").unwrap_or_else(|_| format!("{}/nexcore", env!("HOME")));
    let crate_dir = PathBuf::from(&nexcore_root)
        .join("crates")
        .join(format!("nexcore-{name}"));

    if crate_dir.exists() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({
                "error": format!("Crate directory already exists: {}", crate_dir.display()),
                "hint": "Use crate_dev_audit to check existing crate quality instead."
            })
            .to_string(),
        )]));
    }

    // Generate all file contents
    let files = vec![
        (
            "Cargo.toml",
            cargo_toml_template(&name, domain, &description, t1, t2p, t2c, t3),
        ),
        (
            "src/lib.rs",
            lib_rs_template(&name, domain, t1, t2p, t2c, t3),
        ),
        ("src/primitives.rs", primitives_rs_template(domain)),
        ("src/composites.rs", composites_rs_template(domain)),
        ("src/grounding.rs", grounding_rs_template(domain)),
        ("src/transfer.rs", transfer_rs_template(domain)),
        ("src/prelude.rs", prelude_rs_template()),
    ];

    // Create directory structure
    let src_dir = crate_dir.join("src");
    if let Err(e) = std::fs::create_dir_all(&src_dir) {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": format!("Failed to create directory: {e}")}).to_string(),
        )]));
    }

    // Write all files
    let mut written = Vec::new();
    for (rel_path, content) in &files {
        let full_path = crate_dir.join(rel_path);
        if let Err(e) = std::fs::write(&full_path, content) {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": format!("Failed to write {rel_path}: {e}")}).to_string(),
            )]));
        }
        written.push(*rel_path);
    }

    let result = json!({
        "status": "scaffolded",
        "crate": format!("nexcore-{name}"),
        "path": crate_dir.display().to_string(),
        "domain": domain,
        "type_distribution": {
            "total": type_count,
            "T1": t1,
            "T2-P": t2p,
            "T2-C": t2c,
            "T3": t3
        },
        "files_created": written,
        "next_steps": [
            "1. Add types to src/primitives.rs (T1 + T2-P)",
            "2. Add composites to src/composites.rs (T2-C struct-of-structs)",
            "3. Add service models if needed (T3 with telescoping)",
            "4. Implement GroundsTo for ALL types in src/grounding.rs",
            "5. Add 3 transfer mappings per T1/T2-P type in src/transfer.rs",
            "6. Update src/prelude.rs with re-exports",
            "7. Add crate to workspace members in root Cargo.toml",
            format!("8. cargo check -p nexcore-{name}"),
            format!("9. cargo test -p nexcore-{name} --lib"),
            format!("10. cargo clippy -p nexcore-{name} -- -D warnings"),
        ],
        "quality_checklist": {
            "forbid_unsafe": "#![forbid(unsafe_code)] in lib.rs",
            "deny_unwrap": "#![deny(clippy::unwrap_used)] in lib.rs",
            "all_grounded": "impl GroundsTo for every type",
            "transfers_complete": "3 per T1/T2-P type (PV, Biology, Economics)",
            "serde_roundtrip": "one test per type",
            "deps_minimal": "lex-primitiva + serde only"
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Audit a nexcore crate against gold standard quality checks.
pub fn audit_crate(params: CrateDevAuditParams) -> Result<CallToolResult, McpError> {
    let crate_name = if params.crate_name.starts_with("nexcore-") {
        params.crate_name.clone()
    } else {
        format!("nexcore-{}", params.crate_name)
    };

    let nexcore_root =
        std::env::var("NEXCORE_ROOT").unwrap_or_else(|_| format!("{}/nexcore", env!("HOME")));
    let crate_dir = PathBuf::from(&nexcore_root)
        .join("crates")
        .join(&crate_name);

    if !crate_dir.exists() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({
                "error": format!("Crate not found: {}", crate_dir.display()),
                "hint": "Check the crate name or use crate_dev_scaffold to create it."
            })
            .to_string(),
        )]));
    }

    let mut checks = Vec::new();
    let mut score: f64 = 0.0;
    let max_score: f64 = 10.0;

    // Check 1: lib.rs exists and has safety denials
    let lib_rs = crate_dir.join("src/lib.rs");
    if lib_rs.exists() {
        let content = std::fs::read_to_string(&lib_rs).unwrap_or_default();
        let has_forbid_unsafe = content.contains("forbid(unsafe_code)");
        // Check deny/forbid context — handles both individual and combined attributes:
        //   #![deny(clippy::unwrap_used)]  (individual)
        //   #![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]  (combined)
        let has_deny = content.contains("deny(") || content.contains("forbid(");
        let has_deny_unwrap = has_deny && content.contains("clippy::unwrap_used");
        let has_deny_expect = has_deny && content.contains("clippy::expect_used");
        let has_deny_panic = has_deny && content.contains("clippy::panic");

        if has_forbid_unsafe {
            score += 1.0;
            checks.push(json!({"check": "forbid_unsafe", "status": "PASS"}));
        } else {
            checks.push(json!({"check": "forbid_unsafe", "status": "FAIL", "fix": "Add #![forbid(unsafe_code)] to lib.rs"}));
        }

        if has_deny_unwrap && has_deny_expect && has_deny_panic {
            score += 1.0;
            checks.push(json!({"check": "deny_panics", "status": "PASS"}));
        } else {
            let mut missing = Vec::new();
            if !has_deny_unwrap {
                missing.push("clippy::unwrap_used");
            }
            if !has_deny_expect {
                missing.push("clippy::expect_used");
            }
            if !has_deny_panic {
                missing.push("clippy::panic");
            }
            checks.push(json!({"check": "deny_panics", "status": "FAIL", "missing": missing}));
        }
    } else {
        checks.push(json!({"check": "lib_rs", "status": "FAIL", "fix": "src/lib.rs not found"}));
    }

    // Check 2: Required modules exist
    let required_modules = [
        "primitives.rs",
        "composites.rs",
        "grounding.rs",
        "transfer.rs",
        "prelude.rs",
    ];
    let mut modules_found = 0;
    for module in &required_modules {
        let module_path = crate_dir.join("src").join(module);
        if module_path.exists() {
            modules_found += 1;
        }
    }
    let module_score = modules_found as f64 / required_modules.len() as f64;
    score += module_score * 2.0; // Worth 2 points
    checks.push(json!({
        "check": "gold_standard_modules",
        "status": if modules_found == required_modules.len() { "PASS" } else { "PARTIAL" },
        "found": modules_found,
        "expected": required_modules.len(),
        "missing": required_modules.iter()
            .filter(|m| !crate_dir.join("src").join(m).exists())
            .collect::<Vec<_>>()
    }));

    // Check 3: GroundsTo implementations
    let grounding_path = crate_dir.join("src/grounding.rs");
    let grounding_count = if grounding_path.exists() {
        let content = std::fs::read_to_string(&grounding_path).unwrap_or_default();
        content.matches("impl GroundsTo for").count()
    } else {
        0
    };

    // Count types in primitives + composites
    let mut type_count = 0;
    for module in &["primitives.rs", "composites.rs", "service_models.rs"] {
        let path = crate_dir.join("src").join(module);
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            type_count += content.matches("pub struct ").count();
            type_count += content.matches("pub enum ").count();
        }
    }

    let grounding_coverage = if type_count > 0 {
        grounding_count as f64 / type_count as f64
    } else {
        0.0
    };
    score += grounding_coverage.min(1.0) * 2.0; // Worth 2 points
    checks.push(json!({
        "check": "grounding_coverage",
        "status": if grounding_count >= type_count && type_count > 0 { "PASS" } else { "PARTIAL" },
        "grounded": grounding_count,
        "total_types": type_count,
        "coverage": format!("{:.0}%", grounding_coverage * 100.0)
    }));

    // Check 4: Transfer mappings
    let transfer_path = crate_dir.join("src/transfer.rs");
    let transfer_count = if transfer_path.exists() {
        let content = std::fs::read_to_string(&transfer_path).unwrap_or_default();
        content.matches("TransferMapping {").count() + content.matches("TransferMapping{").count()
    } else {
        0
    };
    let expected_transfers = type_count * 3; // 3 per type
    let transfer_coverage = if expected_transfers > 0 {
        transfer_count as f64 / expected_transfers as f64
    } else {
        0.0
    };
    score += transfer_coverage.min(1.0) * 2.0; // Worth 2 points
    checks.push(json!({
        "check": "transfer_mappings",
        "status": if transfer_count >= expected_transfers && expected_transfers > 0 { "PASS" } else { "PARTIAL" },
        "mappings": transfer_count,
        "expected": expected_transfers,
        "coverage": format!("{:.0}%", transfer_coverage * 100.0)
    }));

    // Check 5: Cargo.toml deps minimality
    let cargo_toml = crate_dir.join("Cargo.toml");
    let dep_count = if cargo_toml.exists() {
        let content = std::fs::read_to_string(&cargo_toml).unwrap_or_default();
        // Count [dependencies] entries (rough heuristic)
        let in_deps = content
            .lines()
            .skip_while(|l| !l.starts_with("[dependencies]"))
            .skip(1)
            .take_while(|l| !l.starts_with('['))
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .count();
        in_deps
    } else {
        0
    };
    let deps_minimal = dep_count <= 5;
    if deps_minimal {
        score += 1.0;
    }
    checks.push(json!({
        "check": "deps_minimal",
        "status": if deps_minimal { "PASS" } else { "WARN" },
        "dependency_count": dep_count,
        "threshold": 5,
        "note": if deps_minimal { "Lean dependency set" } else { "Consider reducing dependencies" }
    }));

    // Check 6: Serde derives on types
    let mut serde_count = 0;
    for module in &["primitives.rs", "composites.rs", "service_models.rs"] {
        let path = crate_dir.join("src").join(module);
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            serde_count += content.matches("Serialize, Deserialize").count();
        }
    }
    let serde_coverage = if type_count > 0 {
        serde_count as f64 / type_count as f64
    } else {
        0.0
    };
    score += serde_coverage.min(1.0) * 1.0; // Worth 1 point
    checks.push(json!({
        "check": "serde_derives",
        "status": if serde_count >= type_count && type_count > 0 { "PASS" } else { "PARTIAL" },
        "with_serde": serde_count,
        "total_types": type_count,
        "coverage": format!("{:.0}%", serde_coverage * 100.0)
    }));

    // Grade
    let grade = match score {
        s if s >= 9.0 => "GOLD",
        s if s >= 7.0 => "SILVER",
        s if s >= 5.0 => "BRONZE",
        _ => "UNRATED",
    };

    let result = json!({
        "crate": crate_name,
        "path": crate_dir.display().to_string(),
        "grade": grade,
        "score": format!("{:.1}/{:.1}", score, max_score),
        "type_count": type_count,
        "checks": checks,
        "tier_breakdown": {
            "grounded_types": grounding_count,
            "transfer_mappings": transfer_count,
            "serde_types": serde_count
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
