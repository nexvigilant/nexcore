use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Parser;

use dag_publish::{build_dag, group_into_phases, topological_sort};

#[derive(Parser)]
#[command(name = "dag-publish")]
#[command(about = "Topological DAG publisher for nexcore crates")]
struct Cli {
    /// Path to the crates directory (e.g., crates/)
    #[arg(long)]
    crates_dir: PathBuf,

    /// Registry name to publish to
    #[arg(long, default_value = "nexcore")]
    registry: String,

    /// Dry run: show publish order without actually publishing
    #[arg(long)]
    dry_run: bool,

    /// Show parallelizable phases instead of flat order
    #[arg(long)]
    show_phases: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let crates_dir = cli.crates_dir.canonicalize().with_context(|| {
        format!(
            "Failed to resolve crates directory: {}",
            cli.crates_dir.display()
        )
    })?;

    eprintln!("Scanning crates in: {}", crates_dir.display());

    let dag = build_dag(&crates_dir, &cli.registry)?;
    eprintln!("Found {} crates", dag.len());

    if cli.show_phases {
        return show_phases(&dag);
    }

    let order = topological_sort(&dag)?;

    if cli.dry_run {
        return dry_run(&order);
    }

    publish(&order, &cli.registry)
}

fn show_phases(dag: &[(String, Vec<String>)]) -> Result<()> {
    let phases = group_into_phases(dag)?;

    for (i, phase) in phases.iter().enumerate() {
        println!("Phase {i} ({} crates):", phase.len());
        for name in phase {
            println!("  {name}");
        }
        println!();
    }

    let total: usize = phases.iter().map(|p| p.len()).sum();
    println!(
        "Total: {total} crates in {} phases",
        phases.len()
    );

    Ok(())
}

fn dry_run(order: &[String]) -> Result<()> {
    println!("Publish order ({} crates):", order.len());
    println!();

    for (i, name) in order.iter().enumerate() {
        println!("  {:>3}. {name}", i + 1);
    }

    Ok(())
}

fn validate_crate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Crate name cannot be empty");
    }

    if name.len() > 64 {
        bail!("Crate name '{name}' exceeds 64 characters (length: {})", name.len());
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() {
        bail!("Crate name '{name}' must start with an ASCII letter");
    }

    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        bail!("Crate name '{name}' contains invalid characters (only alphanumeric, '_', and '-' allowed)");
    }

    Ok(())
}

fn publish(order: &[String], registry: &str) -> Result<()> {
    let mut published: Vec<String> = Vec::new();

    for (i, name) in order.iter().enumerate() {
        validate_crate_name(name)?;

        eprintln!(
            "[{}/{}] Publishing {name} to {registry}...",
            i + 1,
            order.len()
        );

        let status = Command::new("cargo")
            .args(["publish", "-p", name, "--registry", registry])
            .status()
            .with_context(|| format!("Failed to run cargo publish for {name}"))?;

        if !status.success() {
            let exit_msg = match status.code() {
                Some(code) => format!("exit code {code}"),
                None => "terminated by signal".to_string(),
            };

            if !published.is_empty() {
                eprintln!("\nSuccessfully published crates before failure:");
                for published_name in &published {
                    eprintln!("  - {published_name}");
                }
            }

            bail!("cargo publish failed for {name} ({exit_msg})");
        }

        published.push(name.clone());
        eprintln!("  Published {name}");
    }

    eprintln!("All {} crates published successfully.", order.len());
    Ok(())
}
