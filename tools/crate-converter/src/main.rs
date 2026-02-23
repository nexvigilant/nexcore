use clap::Parser;
use nexcore_error::{Context, Result, bail};
use std::path::PathBuf;
use toml_edit::DocumentMut;

#[derive(Parser, Debug)]
#[command(
    name = "crate-converter",
    about = "Converts workspace-dependent Cargo.toml files to standalone packages for registry publishing"
)]
struct Cli {
    /// Path to the workspace root (directory containing the root Cargo.toml)
    #[arg(long)]
    workspace: PathBuf,

    /// Path to a specific crate directory to convert
    #[arg(long, conflicts_with = "all")]
    crate_path: Option<PathBuf>,

    /// Convert all crates in the workspace
    #[arg(long, conflicts_with = "crate_path")]
    all: bool,

    /// Dry run: print the converted Cargo.toml without writing
    #[arg(long)]
    dry_run: bool,

    /// Name of the private registry for internal dependencies
    #[arg(long, default_value = "nexcore")]
    registry: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let workspace_root = cli
        .workspace
        .canonicalize()
        .with_context(|| format!("Workspace path does not exist: {}", cli.workspace.display()))?;

    // Verify workspace Cargo.toml is readable (error will provide context if missing)
    std::fs::read_to_string(workspace_root.join("Cargo.toml")).with_context(|| {
        format!(
            "Cannot read workspace Cargo.toml at {}",
            workspace_root.join("Cargo.toml").display()
        )
    })?;

    if let Some(crate_path) = &cli.crate_path {
        convert_single_crate(crate_path, &workspace_root, &cli.registry, cli.dry_run)?;
    } else if cli.all {
        convert_all_crates(&workspace_root, &cli.registry, cli.dry_run)?;
    } else {
        bail!("Either --crate-path or --all must be specified");
    }

    Ok(())
}

fn convert_single_crate(
    crate_path: &std::path::Path,
    workspace_root: &std::path::Path,
    registry_name: &str,
    dry_run: bool,
) -> Result<()> {
    let crate_path = crate_path
        .canonicalize()
        .with_context(|| format!("Crate path does not exist: {}", crate_path.display()))?;

    let cargo_path = crate_path.join("Cargo.toml");
    let original = std::fs::read_to_string(&cargo_path)
        .with_context(|| format!("No Cargo.toml found at {}", cargo_path.display()))?;
    let converted =
        crate_converter::convert_crate_file(&cargo_path, workspace_root, registry_name)?;

    if original == converted {
        println!("[skip] {} — no workspace references", cargo_path.display());
        return Ok(());
    }

    // Validate the converted TOML is well-formed before writing
    let _: DocumentMut = converted.parse().with_context(|| {
        format!(
            "Converted TOML for {} is not valid TOML",
            cargo_path.display()
        )
    })?;

    if dry_run {
        println!("=== {} (dry-run) ===", cargo_path.display());
        println!("{converted}");
    } else {
        std::fs::write(&cargo_path, &converted)?;
        println!("[done] {}", cargo_path.display());
    }

    Ok(())
}

fn convert_all_crates(
    workspace_root: &std::path::Path,
    registry_name: &str,
    dry_run: bool,
) -> Result<()> {
    let crates_dir = workspace_root.join("crates");
    std::fs::read_dir(&crates_dir)
        .with_context(|| format!("No crates/ directory found at {}", crates_dir.display()))?;

    let mut count_converted = 0;
    let mut count_skipped = 0;
    let mut count_errors = 0;

    for entry in walkdir::WalkDir::new(&crates_dir).min_depth(1).max_depth(2) {
        let entry = entry?;
        if entry.file_name() != "Cargo.toml" {
            continue;
        }

        let cargo_path = entry.path();
        let crate_dir = cargo_path.parent().ok_or_else(|| {
            nexcore_error::nexerror!(
                "Cannot determine parent directory of {}",
                cargo_path.display()
            )
        })?;

        match crate_converter::convert_crate_file(cargo_path, workspace_root, registry_name) {
            Ok(converted) => {
                let original = std::fs::read_to_string(cargo_path)?;
                if original == converted {
                    count_skipped += 1;
                    continue;
                }

                // Validate the converted TOML is well-formed before writing
                let _: DocumentMut = converted.parse().with_context(|| {
                    format!(
                        "Converted TOML for {} is not valid TOML",
                        cargo_path.display()
                    )
                })?;

                if dry_run {
                    println!("=== {} (dry-run) ===", cargo_path.display());
                    println!("{converted}");
                    println!();
                } else {
                    std::fs::write(cargo_path, &converted)?;
                    println!("[done] {}", crate_dir.display());
                }
                count_converted += 1;
            }
            Err(e) => {
                eprintln!("[error] {}: {e:#}", cargo_path.display());
                count_errors += 1;
            }
        }
    }

    println!();
    println!(
        "Summary: {count_converted} converted, {count_skipped} skipped, {count_errors} errors"
    );

    if count_errors > 0 {
        bail!("{count_errors} crate(s) had errors");
    }

    Ok(())
}
