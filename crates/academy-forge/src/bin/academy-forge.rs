// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # academy-forge — micro-app
//!
//! Unified CLI for the full Academy Forge pipeline:
//! extract → scaffold → atomize → validate → compile → graph.
//!
//! ```bash
//! academy-forge extract ./crates/nexcore-tov --domain vigilance
//! academy-forge atomize pathway.json
//! academy-forge validate pathway.json
//! academy-forge compile pathway.json ./output/
//! academy-forge graph atomized1.json atomized2.json
//! academy-forge scaffold --crate ./crates/nexcore-tov --output tov-01.json
//! academy-forge pipeline ./crates/nexcore-tov --domain vigilance --output-dir ./out/
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Academy Forge — knowledge extraction and learning pathway compiler.
///
/// Transforms NexCore Rust crates into structured learning pathways
/// via a 6-stage pipeline: extract → scaffold → atomize → validate → compile → graph.
#[derive(Parser)]
#[command(name = "academy-forge", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Extract a Rust crate into CrateAnalysis IR (JSON to stdout).
    Extract {
        /// Path to the crate root (directory containing Cargo.toml).
        crate_path: PathBuf,
        /// Optional domain plugin name (e.g., "vigilance").
        #[arg(long)]
        domain: Option<String>,
        /// Write output to file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Decompose a pathway JSON into Atomic Learning Objects.
    Atomize {
        /// Path to the pathway JSON file.
        pathway: PathBuf,
        /// Write output to file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate pathway JSON against the 27-rule engine.
    Validate {
        /// One or more pathway JSON files to validate.
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Compile pathway JSON into Studio TypeScript files.
    Compile {
        /// Path to the pathway JSON file.
        pathway: PathBuf,
        /// Directory to write TypeScript files into.
        output_dir: PathBuf,
        /// Skip existing files instead of overwriting.
        #[arg(long)]
        no_overwrite: bool,
    },

    /// Merge atomized pathways into a unified LearningGraph.
    Graph {
        /// One or more atomized pathway JSON files.
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// Enable fuzzy similarity detection.
        #[arg(long)]
        fuzzy: bool,
        /// Similarity threshold for fuzzy matching (0.0-1.0).
        #[arg(long, default_value = "0.75")]
        threshold: f32,
        /// Write output to file instead of stdout.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate a pathway scaffold from extracted IR.
    Scaffold {
        /// Path to the crate root (for extraction).
        #[arg(long)]
        crate_path: PathBuf,
        /// Domain plugin name (required for scaffold generation).
        #[arg(long)]
        domain: String,
        /// Pathway ID prefix (e.g., "tov-01"). Auto-derived from crate name if omitted.
        #[arg(long)]
        pathway_id: Option<String>,
        /// Pathway title. Auto-derived from crate name if omitted.
        #[arg(long)]
        title: Option<String>,
        /// Output pathway JSON file.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Run the full pipeline: extract → atomize → validate → graph.
    Pipeline {
        /// Path to the crate root.
        crate_path: PathBuf,
        /// Domain plugin name.
        #[arg(long)]
        domain: Option<String>,
        /// Output directory for all artifacts.
        #[arg(long, default_value = "./academy-forge-out")]
        output_dir: PathBuf,
        /// Also compile to TypeScript.
        #[arg(long)]
        compile: bool,
    },

    /// Show the IR schema (JSON Schema for StaticPathway).
    Schema,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Extract {
            crate_path,
            domain,
            output,
        } => cmd_extract(&crate_path, domain.as_deref(), output.as_deref()),

        Command::Atomize { pathway, output } => cmd_atomize(&pathway, output.as_deref()),

        Command::Validate { files } => cmd_validate(&files),

        Command::Compile {
            pathway,
            output_dir,
            no_overwrite,
        } => cmd_compile(&pathway, &output_dir, !no_overwrite),

        Command::Graph {
            files,
            fuzzy,
            threshold,
            output,
        } => cmd_graph(&files, fuzzy, threshold, output.as_deref()),

        Command::Scaffold {
            crate_path,
            domain,
            pathway_id,
            title,
            output,
        } => cmd_scaffold(
            &crate_path,
            &domain,
            pathway_id.as_deref(),
            title.as_deref(),
            output.as_deref(),
        ),

        Command::Pipeline {
            crate_path,
            domain,
            output_dir,
            compile,
        } => cmd_pipeline(&crate_path, domain.as_deref(), &output_dir, compile),

        Command::Schema => cmd_schema(),
    }
}

// ── Subcommand implementations ──────────────────────────────────────────────

fn cmd_extract(crate_path: &Path, domain: Option<&str>, output: Option<&Path>) -> ExitCode {
    if !crate_path.exists() {
        eprintln!("Error: crate path not found: {}", crate_path.display());
        return ExitCode::from(1);
    }

    match academy_forge::extract_crate(crate_path, domain) {
        Ok(analysis) => {
            let json = match serde_json::to_string_pretty(&analysis) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error serializing IR: {e}");
                    return ExitCode::from(1);
                }
            };

            if let Some(out_path) = output {
                if let Err(e) = std::fs::write(out_path, &json) {
                    eprintln!("Error writing to {}: {e}", out_path.display());
                    return ExitCode::from(1);
                }
                eprintln!(
                    "Extracted {} ({} modules, {} types, {} enums) → {}",
                    analysis.name,
                    analysis.modules.len(),
                    analysis.public_types.len(),
                    analysis.public_enums.len(),
                    out_path.display()
                );
            } else {
                println!("{json}");
                eprintln!(
                    "Extracted {} ({} modules, {} types, {} enums)",
                    analysis.name,
                    analysis.modules.len(),
                    analysis.public_types.len(),
                    analysis.public_enums.len(),
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Extract failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_atomize(pathway: &Path, output: Option<&Path>) -> ExitCode {
    let raw = match std::fs::read_to_string(pathway) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {e}", pathway.display());
            return ExitCode::from(1);
        }
    };

    let content: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON in {}: {e}", pathway.display());
            return ExitCode::from(1);
        }
    };

    match academy_forge::atomize(&content) {
        Ok(atomized) => {
            let json = match serde_json::to_string_pretty(&atomized) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error serializing atomized pathway: {e}");
                    return ExitCode::from(1);
                }
            };

            if let Some(out_path) = output {
                if let Err(e) = std::fs::write(out_path, &json) {
                    eprintln!("Error writing to {}: {e}", out_path.display());
                    return ExitCode::from(1);
                }
                eprintln!(
                    "Atomized {} → {} ALOs, {} edges → {}",
                    atomized.id,
                    atomized.alos.len(),
                    atomized.edges.len(),
                    out_path.display()
                );
            } else {
                println!("{json}");
                eprintln!(
                    "Atomized {} → {} ALOs, {} edges",
                    atomized.id,
                    atomized.alos.len(),
                    atomized.edges.len(),
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Atomize failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_validate(files: &[PathBuf]) -> ExitCode {
    let workspace_root = resolve_workspace_root();
    let mut all_passed = true;
    let mut validated = 0u32;

    for path in files {
        if !path.exists() {
            eprintln!("[SKIP] {} — file not found", path.display());
            continue;
        }

        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[ERROR] {} — cannot read: {e}", path.display());
                all_passed = false;
                continue;
            }
        };

        let content: serde_json::Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[ERROR] {} — invalid JSON: {e}", path.display());
                all_passed = false;
                continue;
            }
        };

        let domain_analysis =
            content
                .get("domain")
                .and_then(|d| d.as_str())
                .and_then(|domain_name| {
                    workspace_root.as_ref().and_then(|root| {
                        academy_forge::domain::extract_domain(domain_name, root).ok()
                    })
                });

        let report = academy_forge::validate(&content, domain_analysis.as_ref());
        validated += 1;

        if report.passed {
            eprintln!(
                "[PASS] {} — {} warnings, {} advisories",
                path.display(),
                report.warning_count,
                report.advisory_count
            );
        } else {
            all_passed = false;
            eprintln!(
                "[FAIL] {} — {} errors, {} warnings, {} advisories",
                path.display(),
                report.error_count,
                report.warning_count,
                report.advisory_count
            );
            for finding in &report.findings {
                let severity = match finding.severity {
                    academy_forge::validate::Severity::Error => "ERROR",
                    academy_forge::validate::Severity::Warning => "WARN",
                    academy_forge::validate::Severity::Advisory => "INFO",
                    _ => "UNKNOWN",
                };
                let field = finding.field_path.as_deref().unwrap_or("-");
                eprintln!(
                    "  [{severity}] {}: {} ({})",
                    finding.rule, finding.message, field
                );
            }
        }
    }

    if validated == 0 {
        eprintln!("No files validated.");
        return ExitCode::from(2);
    }

    eprintln!(
        "\n{validated} file(s) validated. {}",
        if all_passed {
            "All passed."
        } else {
            "Some failed."
        }
    );

    if all_passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn cmd_compile(pathway: &Path, output_dir: &Path, overwrite: bool) -> ExitCode {
    if !pathway.exists() {
        eprintln!("Error: input file not found: {}", pathway.display());
        return ExitCode::from(1);
    }

    let params = academy_forge::CompileParams::new(
        pathway.to_path_buf(),
        output_dir.to_path_buf(),
        overwrite,
    );

    match academy_forge::compile_pathway(&params) {
        Ok(result) => {
            eprintln!(
                "Compiled {} stages from {} into {}",
                result.stages_compiled,
                pathway.display(),
                output_dir.display()
            );

            for path in &result.files_written {
                eprintln!("  wrote: {}", path.display());
            }

            if !result.warnings.is_empty() {
                eprintln!();
                for warning in &result.warnings {
                    eprintln!("  warning: {warning}");
                }
            }

            eprintln!(
                "\n{} files written, {} warnings.",
                result.files_written.len(),
                result.warnings.len()
            );

            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Compile failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_graph(files: &[PathBuf], fuzzy: bool, threshold: f32, output: Option<&Path>) -> ExitCode {
    let mut pathways = Vec::new();

    for path in files {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading {}: {e}", path.display());
                return ExitCode::from(1);
            }
        };

        let atomized: academy_forge::ir::AtomizedPathway = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Invalid atomized pathway JSON in {}: {e}", path.display());
                return ExitCode::from(1);
            }
        };

        pathways.push(atomized);
    }

    match academy_forge::build_graph(&pathways, fuzzy, threshold) {
        Ok(graph) => {
            let json = match serde_json::to_string_pretty(&graph) {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error serializing graph: {e}");
                    return ExitCode::from(1);
                }
            };

            if let Some(out_path) = output {
                if let Err(e) = std::fs::write(out_path, &json) {
                    eprintln!("Error writing to {}: {e}", out_path.display());
                    return ExitCode::from(1);
                }
                eprintln!(
                    "Graph: {} nodes, {} edges, {} components, {} overlaps → {}",
                    graph.metadata.node_count,
                    graph.metadata.edge_count,
                    graph.metadata.connected_components,
                    graph.overlap_clusters.len(),
                    out_path.display()
                );
            } else {
                println!("{json}");
                eprintln!(
                    "Graph: {} nodes, {} edges, {} components, {} overlaps",
                    graph.metadata.node_count,
                    graph.metadata.edge_count,
                    graph.metadata.connected_components,
                    graph.overlap_clusters.len(),
                );
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Graph build failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_scaffold(
    crate_path: &Path,
    domain: &str,
    pathway_id: Option<&str>,
    title: Option<&str>,
    output: Option<&Path>,
) -> ExitCode {
    if !crate_path.exists() {
        eprintln!("Error: crate path not found: {}", crate_path.display());
        return ExitCode::from(1);
    }

    // Extract crate IR first (with domain)
    let analysis = match academy_forge::extract_crate(crate_path, Some(domain)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Extract failed: {e}");
            return ExitCode::from(1);
        }
    };

    let domain_analysis = match &analysis.domain {
        Some(d) => d,
        None => {
            eprintln!("Error: no domain analysis produced for domain '{domain}'");
            return ExitCode::from(1);
        }
    };

    let pid = pathway_id
        .map(String::from)
        .unwrap_or_else(|| format!("{}-01", analysis.name.replace("nexcore-", "")));
    let t = title
        .map(String::from)
        .unwrap_or_else(|| format!("{} Academy Pathway", analysis.name));

    let params = academy_forge::ScaffoldParams::new(pid, t, domain);

    let scaffold_json = academy_forge::scaffold(domain_analysis, &params);

    let json = match serde_json::to_string_pretty(&scaffold_json) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error serializing scaffold: {e}");
            return ExitCode::from(1);
        }
    };

    if let Some(out_path) = output {
        if let Err(e) = std::fs::write(out_path, &json) {
            eprintln!("Error writing to {}: {e}", out_path.display());
            return ExitCode::from(1);
        }
        eprintln!("Scaffold written → {}", out_path.display());
    } else {
        println!("{json}");
    }
    ExitCode::SUCCESS
}

fn cmd_pipeline(
    crate_path: &Path,
    domain: Option<&str>,
    output_dir: &Path,
    compile: bool,
) -> ExitCode {
    eprintln!("═══ Academy Forge Pipeline ═══");
    eprintln!();

    // Create output directory
    if let Err(e) = std::fs::create_dir_all(output_dir) {
        eprintln!("Error creating output dir: {e}");
        return ExitCode::from(1);
    }

    // Step 1: Extract
    eprintln!("Step 1/5: Extract crate IR...");
    let analysis = match academy_forge::extract_crate(crate_path, domain) {
        Ok(a) => {
            eprintln!(
                "  ✓ {} — {} modules, {} types",
                a.name,
                a.modules.len(),
                a.public_types.len()
            );
            a
        }
        Err(e) => {
            eprintln!("  ✗ Extract failed: {e}");
            return ExitCode::from(1);
        }
    };

    // Write extraction IR
    let extract_path = output_dir.join("extraction.json");
    if let Ok(json) = serde_json::to_string_pretty(&analysis) {
        let _ = std::fs::write(&extract_path, &json);
    }

    // Step 2: Scaffold
    eprintln!("Step 2/5: Generate pathway scaffold...");
    let scaffold_json = if let Some(domain_analysis) = &analysis.domain {
        let pid = format!("{}-01", analysis.name.replace("nexcore-", ""));
        let t = format!("{} Academy Pathway", analysis.name);
        let domain_name = domain.unwrap_or("general");
        let params = academy_forge::ScaffoldParams::new(pid, t, domain_name);
        let s = academy_forge::scaffold(domain_analysis, &params);
        eprintln!("  ✓ Scaffold generated");
        s
    } else {
        eprintln!("  ⚠ No domain analysis — generating minimal scaffold");
        serde_json::json!({
            "id": format!("{}-01", analysis.name.replace("nexcore-", "")),
            "title": format!("{} Academy Pathway", analysis.name),
            "description": analysis.description,
            "domain": domain.unwrap_or("general"),
            "stages": []
        })
    };

    let scaffold_path = output_dir.join("scaffold.json");
    if let Ok(json) = serde_json::to_string_pretty(&scaffold_json) {
        let _ = std::fs::write(&scaffold_path, &json);
    }

    // Step 3: Atomize
    eprintln!("Step 3/5: Atomize into ALOs...");
    let atomized = match academy_forge::atomize(&scaffold_json) {
        Ok(a) => {
            eprintln!("  ✓ {} ALOs, {} edges", a.alos.len(), a.edges.len());
            a
        }
        Err(e) => {
            eprintln!("  ✗ Atomize failed: {e}");
            return ExitCode::from(1);
        }
    };

    let atomized_path = output_dir.join("atomized.json");
    if let Ok(json) = serde_json::to_string_pretty(&atomized) {
        let _ = std::fs::write(&atomized_path, &json);
    }

    // Step 4: Validate
    eprintln!("Step 4/5: Validate content...");
    let domain_analysis = analysis.domain.as_ref();
    let report = academy_forge::validate(&scaffold_json, domain_analysis);
    if report.passed {
        eprintln!(
            "  ✓ Passed — {} warnings, {} advisories",
            report.warning_count, report.advisory_count
        );
    } else {
        eprintln!(
            "  ⚠ {} errors, {} warnings, {} advisories",
            report.error_count, report.warning_count, report.advisory_count
        );
        for finding in &report.findings {
            if matches!(finding.severity, academy_forge::validate::Severity::Error) {
                let field = finding.field_path.as_deref().unwrap_or("-");
                eprintln!(
                    "    [ERROR] {}: {} ({})",
                    finding.rule, finding.message, field
                );
            }
        }
    }

    let report_path = output_dir.join("validation.json");
    if let Ok(json) = serde_json::to_string_pretty(&report) {
        let _ = std::fs::write(&report_path, &json);
    }

    // Step 5: Graph
    eprintln!("Step 5/5: Build learning graph...");
    match academy_forge::build_graph(&[atomized], false, 0.75) {
        Ok(graph) => {
            eprintln!(
                "  ✓ {} nodes, {} edges, {} components, diameter {}",
                graph.metadata.node_count,
                graph.metadata.edge_count,
                graph.metadata.connected_components,
                graph.metadata.diameter,
            );

            let graph_path = output_dir.join("graph.json");
            if let Ok(json) = serde_json::to_string_pretty(&graph) {
                let _ = std::fs::write(&graph_path, &json);
            }
        }
        Err(e) => {
            eprintln!("  ⚠ Graph build failed: {e}");
        }
    }

    // Optional: Compile to TypeScript
    if compile {
        eprintln!("\nBonus: Compile to TypeScript...");
        let ts_dir = output_dir.join("typescript");
        let compile_params = academy_forge::CompileParams::new(scaffold_path, ts_dir.clone(), true);

        match academy_forge::compile_pathway(&compile_params) {
            Ok(result) => {
                eprintln!(
                    "  ✓ {} stages → {} files in {}",
                    result.stages_compiled,
                    result.files_written.len(),
                    ts_dir.display()
                );
            }
            Err(e) => {
                eprintln!("  ⚠ Compile failed: {e}");
            }
        }
    }

    eprintln!();
    eprintln!("═══ Pipeline complete ═══");
    eprintln!("Artifacts in {}", output_dir.display());

    ExitCode::SUCCESS
}

fn cmd_schema() -> ExitCode {
    let schema = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "StaticPathway",
  "description": "Academy Forge pathway schema. See academy_forge::ir for Rust types.",
  "type": "object",
  "required": ["id", "title", "description", "domain", "stages"],
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9-]+-\\d{2}$" },
    "title": { "type": "string" },
    "description": { "type": "string" },
    "domain": { "type": "string" },
    "componentCount": { "type": "integer", "minimum": 1 },
    "estimatedDuration": { "type": "string" },
    "stages": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "title", "activities"],
        "properties": {
          "id": { "type": "string" },
          "title": { "type": "string" },
          "description": { "type": "string" },
          "bloomLevel": { "type": "string", "enum": ["Remember", "Understand", "Apply", "Analyze", "Evaluate", "Create"] },
          "passingScore": { "type": "integer", "minimum": 0, "maximum": 100 },
          "estimatedDuration": { "type": "string" },
          "activities": {
            "type": "array",
            "items": {
              "type": "object",
              "required": ["id", "title", "type"],
              "properties": {
                "id": { "type": "string" },
                "title": { "type": "string" },
                "type": { "type": "string", "enum": ["reading", "interactive", "case-study", "quiz"] },
                "estimatedDuration": { "type": "string" },
                "content": { "type": "string" },
                "quiz": {
                  "type": "object",
                  "properties": {
                    "questions": {
                      "type": "array",
                      "items": {
                        "type": "object",
                        "required": ["id", "type", "question"],
                        "properties": {
                          "id": { "type": "string" },
                          "type": { "type": "string" },
                          "question": { "type": "string" },
                          "options": { "type": "array", "items": { "type": "string" } },
                          "correctAnswer": { "type": "integer" },
                          "points": { "type": "integer" },
                          "explanation": { "type": "string" }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}"#;

    println!("{schema}");
    ExitCode::SUCCESS
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn resolve_workspace_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let cargo = dir.join("Cargo.toml");
        if cargo.exists() {
            if let Ok(text) = std::fs::read_to_string(&cargo) {
                if text.contains("[workspace]") {
                    return Some(dir);
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
