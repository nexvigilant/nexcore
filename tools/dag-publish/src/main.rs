use std::collections::BTreeSet;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use clap::Parser;
use nexcore_error::{Context, Result, bail};

use dag_publish::{build_dag, group_into_phases, topological_sort};

#[derive(Parser)]
#[command(name = "dag-publish")]
#[command(about = "Topological DAG publisher for nexcore crates → crates.io")]
struct Cli {
    /// Path to the crates directory (e.g., crates/)
    #[arg(long)]
    crates_dir: PathBuf,

    /// Registry name for dependency resolution (not publish target)
    #[arg(long, default_value = "nexcore")]
    registry: String,

    /// Dry run: show publish order without actually publishing
    #[arg(long)]
    dry_run: bool,

    /// Allow publishing with uncommitted changes
    #[arg(long)]
    allow_dirty: bool,

    /// Show parallelizable phases instead of flat order
    #[arg(long)]
    show_phases: bool,

    /// Delay in seconds between publishes (crates.io rate limit)
    #[arg(long, default_value = "10")]
    delay: u64,

    /// Max retries on rate limit (429) before giving up on a crate
    #[arg(long, default_value = "5")]
    max_retries: u32,

    /// Skip crates that are already published on crates.io
    #[arg(long)]
    skip_published: bool,

    /// Only publish crates matching this prefix (e.g., "stem-" or "nexcore-")
    #[arg(long)]
    filter: Option<String>,

    /// Maximum number of crates to publish in this run
    #[arg(long)]
    limit: Option<usize>,

    /// Also scan tools/ directory for additional crates
    #[arg(long)]
    include_tools: bool,
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
        let phases = group_into_phases(&dag)?;
        for (i, phase) in phases.iter().enumerate() {
            println!("Phase {i} ({} crates):", phase.len());
            for name in phase {
                println!("  {name}");
            }
            println!();
        }
        let total: usize = phases.iter().map(|p| p.len()).sum();
        println!("Total: {total} crates in {} phases", phases.len());
        return Ok(());
    }

    let order = topological_sort(&dag)?;

    if cli.dry_run {
        return dry_run(&order, &cli);
    }

    publish(&order, &cli)
}

fn dry_run(order: &[String], cli: &Cli) -> Result<()> {
    let filtered = apply_filters(order, cli);
    println!("Publish order ({} crates):", filtered.len());
    println!();

    for (i, name) in filtered.iter().enumerate() {
        let status = if cli.skip_published && is_published(name) {
            " [SKIP: already published]"
        } else {
            ""
        };
        println!("  {:>3}. {name}{status}", i + 1);
    }

    Ok(())
}

fn apply_filters<'a>(order: &'a [String], cli: &Cli) -> Vec<&'a String> {
    let mut filtered: Vec<&String> = order.iter().collect();

    if let Some(ref prefix) = cli.filter {
        filtered.retain(|name| name.starts_with(prefix.as_str()));
    }

    if let Some(limit) = cli.limit {
        filtered.truncate(limit);
    }

    filtered
}

/// Check if a crate is already published on crates.io by attempting cargo search.
fn is_published(name: &str) -> bool {
    let output = Command::new("cargo")
        .args(["search", name, "--limit", "1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // cargo search output: `name = "version"    # description`
            // If the exact crate exists, the first line starts with `name = `
            stdout.starts_with(&format!("{name} = "))
        }
        Err(_) => false,
    }
}

/// Parse the wait time from a crates.io 429 error message.
/// Looks for "try again after <datetime>" and computes seconds until then.
fn parse_rate_limit_wait(stderr: &str) -> Option<u64> {
    // Pattern: "try again after Sat, 28 Mar 2026 19:33:28 GMT"
    let marker = "try again after ";
    let idx = stderr.find(marker)?;
    let after = &stderr[idx + marker.len()..];
    let end = after.find(" and ")?;
    let datetime_str = &after[..end];

    // Parse HTTP date and compute wait
    // Simple approach: shell out to date command
    let output = Command::new("date")
        .args(["-u", "-d", datetime_str, "+%s"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let target_epoch: u64 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .ok()?;

    let now_output = Command::new("date").args(["-u", "+%s"]).output().ok()?;

    let now_epoch: u64 = String::from_utf8_lossy(&now_output.stdout)
        .trim()
        .parse()
        .ok()?;

    if target_epoch > now_epoch {
        Some(target_epoch - now_epoch + 5) // +5s buffer
    } else {
        Some(10) // Already past, try again soon
    }
}

fn validate_crate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Crate name cannot be empty");
    }
    if name.len() > 64 {
        bail!(
            "Crate name '{name}' exceeds 64 characters (length: {})",
            name.len()
        );
    }
    let Some(first_char) = name.chars().next() else {
        bail!("Crate name is empty");
    };
    if !first_char.is_ascii_alphabetic() {
        bail!("Crate name '{name}' must start with an ASCII letter");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        bail!("Crate name '{name}' contains invalid characters");
    }
    Ok(())
}

fn publish(order: &[String], cli: &Cli) -> Result<()> {
    let filtered = apply_filters(order, cli);
    let total = filtered.len();

    let mut published: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    let start = Instant::now();

    // Pre-check: find already published crates
    let already_published: BTreeSet<String> = if cli.skip_published {
        eprintln!("Checking which crates are already on crates.io...");
        filtered
            .iter()
            .filter(|name| is_published(name))
            .map(|name| name.to_string())
            .collect()
    } else {
        BTreeSet::new()
    };

    if !already_published.is_empty() {
        eprintln!(
            "Skipping {} already-published crates",
            already_published.len()
        );
    }

    for (i, name) in filtered.iter().enumerate() {
        validate_crate_name(name)?;

        if already_published.contains(*name) {
            eprintln!("[{}/{}] SKIP {name} (already published)", i + 1, total);
            skipped.push(name.to_string());
            continue;
        }

        eprintln!("[{}/{}] Publishing {name}...", i + 1, total);

        let mut success = false;
        for attempt in 0..=cli.max_retries {
            if attempt > 0 {
                eprintln!("  retry {attempt}/{} for {name}...", cli.max_retries);
            }

            let mut args = vec!["publish", "-p", name.as_str()];
            if cli.allow_dirty {
                args.push("--allow-dirty");
            }

            let output = Command::new("cargo")
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .with_context(|| format!("Failed to run cargo publish for {name}"))?;

            let stderr_text = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                published.push(name.to_string());
                eprintln!("  ✓ {name} published");
                success = true;

                // Rate limit delay (skip after last crate)
                if i + 1 < total && cli.delay > 0 {
                    eprintln!("  waiting {}s...", cli.delay);
                    std::thread::sleep(Duration::from_secs(cli.delay));
                }
                break;
            }

            // Check for rate limit (429)
            if stderr_text.contains("429") || stderr_text.contains("Too Many Requests") {
                // Parse retry-after time if available, otherwise use exponential backoff
                let wait_secs =
                    parse_rate_limit_wait(&stderr_text).unwrap_or(30 * (1 << attempt.min(4)));
                eprintln!("  ⏳ Rate limited. Waiting {wait_secs}s before retry...");
                std::thread::sleep(Duration::from_secs(wait_secs));
                continue;
            }

            // Non-rate-limit failure — log last line and move on
            let last_line = stderr_text.lines().last().unwrap_or("unknown error");
            eprintln!("  ✗ FAILED: {last_line}");
            failed.push((name.to_string(), last_line.to_string()));
            break;
        }

        if !success && !failed.iter().any(|(n, _)| n == *name) {
            eprintln!("  ✗ FAILED: {name} (exhausted retries)");
            failed.push((name.to_string(), "exhausted retries".to_string()));
        }
    }

    let elapsed = start.elapsed();

    // Summary
    eprintln!();
    eprintln!("═══════════════════════════════════════");
    eprintln!("  DAG PUBLISH COMPLETE ({:.1}s)", elapsed.as_secs_f64());
    eprintln!("═══════════════════════════════════════");
    eprintln!("  Published: {}", published.len());
    eprintln!("  Skipped:   {}", skipped.len());
    eprintln!("  Failed:    {}", failed.len());
    eprintln!("  Total:     {}", total);

    if !failed.is_empty() {
        eprintln!();
        eprintln!("Failed crates:");
        for (name, reason) in &failed {
            eprintln!("  ✗ {name}: {reason}");
        }
    }

    if !published.is_empty() {
        eprintln!();
        eprintln!("Published crates:");
        for name in &published {
            eprintln!("  ✓ {name}");
        }
    }

    Ok(())
}
