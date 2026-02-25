//! Generic extraction pipeline for Rust crates.
//!
//! Entry point: [`extract_crate`] — parses a crate's source and Cargo.toml
//! into a [`CrateAnalysis`] IR.

pub mod cargo_metadata;
pub mod doc_comments;
pub mod module_tree;
pub mod syn_parser;

use std::path::Path;

use crate::domain;
use crate::error::{ForgeError, ForgeResult};
use crate::ir::{CrateAnalysis, GraphTopology};

/// Extract a complete IR from a Rust crate.
///
/// # Arguments
/// * `crate_path` — Path to the crate root (containing Cargo.toml)
/// * `domain_name` — Optional domain plugin name (e.g., "vigilance")
pub fn extract_crate(crate_path: &Path, domain_name: Option<&str>) -> ForgeResult<CrateAnalysis> {
    if !crate_path.exists() {
        return Err(ForgeError::CrateNotFound(crate_path.to_path_buf()));
    }

    // 1. Parse Cargo.toml
    let meta = cargo_metadata::parse_cargo_toml(crate_path)?;

    // 2. Discover and parse all source files
    let src_path = crate_path.join("src");
    let mut modules = module_tree::discover_modules(&src_path)?;

    let mut all_types = Vec::new();
    let mut all_enums = Vec::new();
    let mut all_constants = Vec::new();
    let mut all_traits = Vec::new();

    for module in &mut modules {
        let file_path = src_path.join(&module.file_path);
        match syn_parser::parse_file(&file_path) {
            Ok(parsed) => {
                if module.doc_comment.is_none() {
                    module.doc_comment = parsed.module_doc;
                }
                module.public_items = parsed.public_item_names;
                all_types.extend(parsed.types);
                all_enums.extend(parsed.enums);
                all_constants.extend(parsed.constants);
                all_traits.extend(parsed.traits);
            }
            Err(ForgeError::ParseError { .. }) => {
                // Skip files that fail to parse (e.g., proc macro crates)
            }
            Err(e) => return Err(e),
        }
    }

    // 3. Build module dependency graph
    let dependency_graph = build_module_graph(&modules);

    // 4. Apply domain plugin if requested
    let domain_analysis = if let Some(name) = domain_name {
        let workspace_root = crate_path
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(crate_path);
        Some(domain::extract_domain(name, workspace_root)?)
    } else {
        None
    };

    Ok(CrateAnalysis {
        name: meta.name,
        version: meta.version,
        description: meta.description,
        modules,
        public_types: all_types,
        public_enums: all_enums,
        constants: all_constants,
        traits: all_traits,
        dependencies: meta.dependencies,
        domain: domain_analysis,
        dependency_graph,
    })
}

/// Build a graph topology from module relationships.
#[allow(
    clippy::as_conversions,
    reason = "usize->f64 cast for line count weight; line counts never exceed f64 precision limits"
)]
fn build_module_graph(modules: &[crate::ir::ModuleInfo]) -> GraphTopology {
    use crate::ir::{GraphEdge, GraphNode};

    let nodes: Vec<GraphNode> = modules
        .iter()
        .map(|m| GraphNode {
            id: m.path.clone(),
            label: m.path.split("::").last().unwrap_or(&m.path).to_string(),
            node_type: "module".to_string(),
            weight: m.line_count as f64,
            metadata: serde_json::json!({
                "file_path": m.file_path,
                "line_count": m.line_count,
            }),
        })
        .collect();

    // Edges: parent module → child module
    let mut edges = Vec::new();
    for module in modules {
        if let Some(parent_pos) = module.path.rfind("::") {
            // parent_pos is a char boundary (rfind returns byte index of ':')
            // but '::' is ASCII so parent_pos points to a valid char boundary.
            let parent = module.path.get(..parent_pos).unwrap_or("");
            if modules.iter().any(|m| m.path == parent) {
                edges.push(GraphEdge {
                    source: parent.to_string(),
                    target: module.path.clone(),
                    edge_type: "contains".to_string(),
                    weight: 1.0,
                });
            }
        }
    }

    GraphTopology { nodes, edges }
}
