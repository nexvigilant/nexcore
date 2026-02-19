//! Build stage: invokes `cargo build --release` on a generated crate.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{CompilerError, Result};

/// Result of a successful build.
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the compiled binary.
    pub binary_path: PathBuf,
    /// Build stdout.
    pub stdout: String,
    /// Build stderr.
    pub stderr: String,
}

/// Build a generated compound skill crate.
///
/// Runs `cargo build --release` in the crate directory.
///
/// # Errors
///
/// Returns `CompilerError::BuildFailed` if cargo returns non-zero.
pub fn build(crate_dir: &Path, binary_name: &str) -> Result<BuildResult> {
    tracing::info!("Building compound skill at {}", crate_dir.display());

    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(crate_dir)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(CompilerError::BuildFailed {
            code: output.status.code().unwrap_or(-1),
            stderr,
        });
    }

    let binary_path = crate_dir.join("target").join("release").join(binary_name);

    if !binary_path.exists() {
        return Err(CompilerError::BuildFailed {
            code: 0,
            stderr: format!("Binary not found at {}", binary_path.display()),
        });
    }

    tracing::info!("Build succeeded: {}", binary_path.display());

    Ok(BuildResult {
        binary_path,
        stdout,
        stderr,
    })
}
