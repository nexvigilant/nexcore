//! CLI binary for nexcore-measure.
//!
//! ```text
//! nexcore-measure crate <name>         # Single crate health
//! nexcore-measure workspace            # Full workspace assessment
//! nexcore-measure graph [--format dot] # Dependency graph analysis
//! nexcore-measure drift [--window 5]   # Detect metric drift
//! nexcore-measure record               # Save snapshot to history
//! nexcore-measure history [--limit 10] # Show measurement history
//! ```

use clap::{Parser, Subcommand};
use nexcore_measure::{collect, composite, graph, history, skill, skill_graph};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nexcore-measure", about = "Workspace quality measurement")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Workspace root (default: ~/nexcore)
    #[arg(long, global = true)]
    workspace: Option<PathBuf>,

    /// Output format: json or text
    #[arg(long, global = true, default_value = "json")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Measure a single crate
    Crate {
        /// Crate name
        name: String,
    },
    /// Measure the entire workspace
    Workspace,
    /// Analyze dependency graph
    Graph,
    /// Detect metric drift
    Drift {
        /// Window size (number of records per side)
        #[arg(long, default_value = "5")]
        window: usize,
    },
    /// Save current snapshot to history
    Record,
    /// Show measurement history
    History {
        /// Number of records to show
        #[arg(long, default_value = "10")]
        limit: usize,
    },
    /// Measure skill ecosystem health
    Skill {
        /// Skills directory (default: ~/.claude/skills)
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Measure a single skill by name
        #[arg(long)]
        name: Option<String>,
    },
    /// Analyze skill dependency graph
    SkillGraph {
        /// Skills directory (default: ~/.claude/skills)
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let ws = workspace_root(&cli.workspace);

    let result = match cli.command {
        Commands::Crate { name } => cmd_crate(&ws, &name, &cli.format),
        Commands::Workspace => cmd_workspace(&ws, &cli.format),
        Commands::Graph => cmd_graph(&ws, &cli.format),
        Commands::Drift { window } => cmd_drift(window, &cli.format),
        Commands::Record => cmd_record(&ws),
        Commands::History { limit } => cmd_history(limit, &cli.format),
        Commands::Skill { dir, name } => cmd_skill(dir, name, &cli.format),
        Commands::SkillGraph { dir } => cmd_skill_graph(dir, &cli.format),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn workspace_root(custom: &Option<PathBuf>) -> PathBuf {
    custom.clone().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("nexcore")
    })
}

fn cmd_crate(
    ws: &PathBuf,
    name: &str,
    format: &str,
) -> Result<(), nexcore_measure::error::MeasureError> {
    let measurement = collect::measure_crate(ws, name)?;
    let now = nexcore_measure::types::MeasureTimestamp::now().epoch_secs();
    let health = composite::crate_health(&measurement, now, now);

    if format == "text" {
        print_crate_text(&measurement, &health);
    } else {
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "measurement": measurement,
            "health": health,
        }))?;
        println!("{json}");
    }
    Ok(())
}

fn cmd_workspace(ws: &PathBuf, format: &str) -> Result<(), nexcore_measure::error::MeasureError> {
    let wm = collect::measure_workspace(ws)?;
    let now = nexcore_measure::types::MeasureTimestamp::now().epoch_secs();
    let wh = composite::workspace_health(&wm.crates, now);

    if format == "text" {
        print_workspace_text(&wm, &wh);
    } else {
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "measurement": wm,
            "health": wh,
        }))?;
        println!("{json}");
    }
    Ok(())
}

fn cmd_graph(ws: &PathBuf, format: &str) -> Result<(), nexcore_measure::error::MeasureError> {
    let dep_graph = graph::build_dep_graph(ws)?;
    let analysis = dep_graph.analyze();

    if format == "text" {
        print_graph_text(&analysis);
    } else {
        let json = serde_json::to_string_pretty(&analysis)?;
        println!("{json}");
    }
    Ok(())
}

fn cmd_drift(window: usize, format: &str) -> Result<(), nexcore_measure::error::MeasureError> {
    let path = history::default_history_path();
    let hist = history::MeasureHistory::load(&path)?;
    let drifts = history::detect_drift(&hist, window)?;

    if format == "text" {
        for d in &drifts {
            let sig = if d.significant { "***" } else { "" };
            println!(
                "{:<15} t={:>7.3} p={:.4} {}{}",
                d.metric, d.t_statistic, d.p_value, d.direction, sig
            );
        }
    } else {
        let json = serde_json::to_string_pretty(&drifts)?;
        println!("{json}");
    }
    Ok(())
}

fn cmd_record(ws: &PathBuf) -> Result<(), nexcore_measure::error::MeasureError> {
    let wm = collect::measure_workspace(ws)?;
    let now = nexcore_measure::types::MeasureTimestamp::now().epoch_secs();
    let wh = composite::workspace_health(&wm.crates, now);
    let path = history::default_history_path();
    let mut hist = history::MeasureHistory::load(&path)?;
    hist.record(&wm, wh.mean_score.value());
    hist.save(&path)?;
    println!(
        "Recorded snapshot #{} to {}",
        hist.records.len(),
        path.display()
    );
    Ok(())
}

fn cmd_history(limit: usize, format: &str) -> Result<(), nexcore_measure::error::MeasureError> {
    let path = history::default_history_path();
    let hist = history::MeasureHistory::load(&path)?;
    let records = hist.last_n(limit);

    if format == "text" {
        println!(
            "{:<12} {:>8} {:>8} {:>6} {:>7}",
            "Timestamp", "LOC", "Tests", "Crates", "Health"
        );
        for r in records {
            println!(
                "{:<12} {:>8} {:>8} {:>6} {:>7.1}",
                r.timestamp, r.total_loc, r.total_tests, r.crate_count, r.mean_health
            );
        }
    } else {
        let json = serde_json::to_string_pretty(&records)?;
        println!("{json}");
    }
    Ok(())
}

fn skills_dir(custom: &Option<PathBuf>) -> PathBuf {
    custom.clone().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".claude").join("skills")
    })
}

fn cmd_skill(
    dir: Option<PathBuf>,
    name: Option<String>,
    format: &str,
) -> Result<(), nexcore_measure::error::MeasureError> {
    let sd = skills_dir(&dir);
    if let Some(skill_name) = name {
        cmd_skill_single(&sd, &skill_name, format)
    } else {
        cmd_skill_ecosystem(&sd, format)
    }
}

fn cmd_skill_single(
    sd: &PathBuf,
    name: &str,
    format: &str,
) -> Result<(), nexcore_measure::error::MeasureError> {
    let path = sd.join(name).join("SKILL.md");
    let content = std::fs::read_to_string(&path)?;
    let m = skill::measure_skill(name, &content)?;
    let h = skill::skill_health(&m, 0.5); // default uniqueness
    if format == "text" {
        print_skill_text(&m, &h);
    } else {
        let json = serde_json::to_string_pretty(&serde_json::json!({
            "measurement": m, "health": h,
        }))?;
        println!("{json}");
    }
    Ok(())
}

fn cmd_skill_ecosystem(
    sd: &PathBuf,
    format: &str,
) -> Result<(), nexcore_measure::error::MeasureError> {
    let eco = skill::measure_ecosystem(sd)?;
    if format == "text" {
        print_skill_ecosystem_text(&eco);
    } else {
        let json = serde_json::to_string_pretty(&eco)?;
        println!("{json}");
    }
    Ok(())
}

fn print_skill_text(m: &skill::SkillMeasurement, h: &skill::SkillHealth) {
    println!("Skill: {}", m.name);
    println!(
        "  Lines: {} ({} content) | Tokens: {} ({} unique)",
        m.total_lines, m.content_lines, m.total_tokens, m.unique_tokens
    );
    println!(
        "  Sections: {} | Entropy: {:.3} bits",
        m.section_count,
        m.section_entropy.value()
    );
    println!(
        "  Info Density: {:.2} tok/line | Structural: {:.0}% | Grounding: {} refs",
        m.info_density,
        m.structural_completeness.value() * 100.0,
        m.grounding_refs
    );
    println!("  Health: {} ({})", h.score, h.rating);
    println!(
        "  Components: ID={:.3} ST={:.3} SB={:.3} UQ={:.3} GR={:.3}",
        h.components.info_density_norm,
        h.components.structural_norm,
        h.components.section_balance_norm,
        h.components.uniqueness_norm,
        h.components.grounding_norm
    );
}

fn cmd_skill_graph(
    dir: Option<PathBuf>,
    format: &str,
) -> Result<(), nexcore_measure::error::MeasureError> {
    let sd = skills_dir(&dir);
    let adj = skill_graph::build_skill_graph(&sd)?;
    let analysis = adj.analyze();
    if format == "text" {
        print_skill_graph_text(&analysis);
    } else {
        let json = serde_json::to_string_pretty(&analysis)?;
        println!("{json}");
    }
    Ok(())
}

fn print_skill_graph_text(a: &skill_graph::SkillGraphAnalysis) {
    println!(
        "Skill Graph: {} nodes, {} edges, density={:.4}",
        a.node_count,
        a.edge_count,
        a.density.value()
    );
    println!("  Max depth: {} | Cycles: {}", a.max_depth, a.cycle_count);
    if !a.cycles.is_empty() {
        println!("  Mutual reference cycles:");
        for cycle in &a.cycles {
            println!("    {}", cycle.join(" ↔ "));
        }
    }

    // Role distribution
    let mut foundations = 0_usize;
    let mut orchestrators = 0_usize;
    let mut hubs = 0_usize;
    let mut leaves = 0_usize;
    for n in &a.nodes {
        match n.role {
            skill_graph::SkillRole::Foundation => {
                foundations += 1;
            }
            skill_graph::SkillRole::Orchestrator => {
                orchestrators += 1;
            }
            skill_graph::SkillRole::Hub => {
                hubs += 1;
            }
            skill_graph::SkillRole::Leaf => {
                leaves += 1;
            }
        }
    }
    println!(
        "  Roles: {} Hub, {} Foundation, {} Orchestrator, {} Leaf",
        hubs, foundations, orchestrators, leaves
    );

    println!();
    println!(
        "{:<35} {:>4} {:>4} {:>8} {:>8} {:>12}",
        "Skill", "In", "Out", "Coupling", "Between.", "Role"
    );
    println!("{}", "-".repeat(85));
    for n in &a.nodes {
        println!(
            "{:<35} {:>4} {:>4} {:>8.4} {:>8.4} {:>12}",
            n.name,
            n.fan_in,
            n.fan_out,
            n.coupling_ratio.value(),
            n.betweenness.value(),
            n.role
        );
    }
}

fn print_skill_ecosystem_text(eco: &skill::SkillEcosystemHealth) {
    println!(
        "Skill Ecosystem: {} skills | {} lines | {:.3} bits ecosystem entropy",
        eco.skill_count,
        eco.total_lines,
        eco.ecosystem_entropy.value()
    );
    println!(
        "  Health: {} ({}) | Uniqueness: {:.3}",
        eco.mean_score,
        eco.mean_rating,
        eco.mean_uniqueness.value()
    );
    println!(
        "  Distribution: C={} W={} A={} G={} E={}",
        eco.rating_distribution.critical,
        eco.rating_distribution.weak,
        eco.rating_distribution.adequate,
        eco.rating_distribution.good,
        eco.rating_distribution.excellent
    );
    println!();

    // Sort by score ascending to highlight weakest first
    let mut sorted: Vec<&skill::SkillHealth> = eco.skill_healths.iter().collect();
    sorted.sort_by(|a, b| a.score.value().total_cmp(&b.score.value()));

    println!(
        "{:<35} {:>6} {:>10}  {:>5} {:>5} {:>5} {:>5} {:>5}",
        "Skill", "Score", "Rating", "ID", "ST", "SB", "UQ", "GR"
    );
    println!("{}", "-".repeat(95));
    for h in &sorted {
        println!(
            "{:<35} {:>6.1} {:>10}  {:.3} {:.3} {:.3} {:.3} {:.3}",
            h.name,
            h.score.value(),
            format!("{}", h.rating),
            h.components.info_density_norm,
            h.components.structural_norm,
            h.components.section_balance_norm,
            h.components.uniqueness_norm,
            h.components.grounding_norm
        );
    }
}

fn print_crate_text(
    m: &nexcore_measure::types::CrateMeasurement,
    h: &nexcore_measure::types::CrateHealth,
) {
    println!("Crate: {}", m.crate_id);
    println!(
        "  LOC: {} | Tests: {} | Modules: {}",
        m.loc, m.test_count, m.module_count
    );
    println!("  Entropy: {} | Redundancy: {}", m.entropy, m.redundancy);
    println!("  Test Density: {} | CDI: {}", m.test_density, m.cdi);
    println!(
        "  Fan-in: {} | Fan-out: {} | Coupling: {}",
        m.fan_in, m.fan_out, m.coupling_ratio
    );
    println!("  Health: {} ({})", h.score, h.rating);
}

fn print_workspace_text(
    wm: &nexcore_measure::types::WorkspaceMeasurement,
    wh: &nexcore_measure::types::WorkspaceHealth,
) {
    println!(
        "Workspace: {} crates | {} LOC | {} tests",
        wm.crate_count, wm.total_loc, wm.total_tests
    );
    println!(
        "  Graph: density={} depth={} cycles={}",
        wm.graph_density, wm.max_depth, wm.scc_count
    );
    let avg_cdi = if !wh.crate_healths.is_empty() {
        wh.crate_healths
            .iter()
            .map(|ch| ch.components.cdi_norm)
            .sum::<f64>()
            / (wh.crate_healths.len() as f64)
    } else {
        0.0
    };
    println!(
        "  Health: {} ({}) | CDI Avg: {:.4}",
        wh.mean_score, wh.mean_rating, avg_cdi
    );
    println!(
        "  Distribution: C={} W={} A={} G={} E={}",
        wh.rating_distribution.critical,
        wh.rating_distribution.weak,
        wh.rating_distribution.adequate,
        wh.rating_distribution.good,
        wh.rating_distribution.excellent
    );
}

fn print_graph_text(a: &nexcore_measure::types::GraphAnalysis) {
    println!("Graph: {} nodes, {} edges", a.node_count, a.edge_count);
    println!(
        "  Density: {} | Depth: {} | Cycles: {}",
        a.density, a.max_depth, a.cycle_count
    );
    if !a.cycles.is_empty() {
        println!("  Circular dependencies:");
        for cycle in &a.cycles {
            let names: Vec<&str> = cycle.iter().map(|c| c.name()).collect();
            println!("    {}", names.join(" → "));
        }
    }
    println!("\nTop crates by betweenness centrality:");
    let mut sorted: Vec<&nexcore_measure::types::GraphNode> = a.nodes.iter().collect();
    sorted.sort_by(|a, b| {
        b.betweenness_centrality
            .value()
            .total_cmp(&a.betweenness_centrality.value())
    });
    for node in sorted.iter().take(10) {
        println!(
            "  {:<30} bc={:.4} dc={:.4} in={} out={}",
            node.crate_id,
            node.betweenness_centrality,
            node.degree_centrality,
            node.fan_in,
            node.fan_out
        );
    }
}
