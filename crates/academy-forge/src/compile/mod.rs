//! Compile module — transforms forge pathway JSON into Studio TypeScript files.
//!
//! This module completes the academy-forge pipeline:
//!
//! ```text
//! forge_extract → DomainAnalysis
//!     → forge_scaffold → tov-01.json (StaticPathway JSON)
//!         → Author fills content
//!             → forge_validate → pass/fail
//!                 → forge_compile → TypeScript files (Studio-compatible)
//! ```
//!
//! ## Output
//!
//! Given a pathway JSON file (e.g. `tov-01.json`) the compile step writes:
//!
//! ```text
//! output_dir/
//! ├── stages/
//! │   ├── 01-system-decomposition.ts
//! │   ├── 02-hierarchical-organization.ts
//! │   └── ...
//! ├── config.ts
//! └── index.ts
//! ```
//!
//! The stage files export `CapabilityStage` objects, `config.ts` holds pathway
//! metadata, and `index.ts` assembles the full `StaticPathway`.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use academy_forge::compile::{compile, CompileParams};
//!
//! let params = CompileParams::new(
//!     PathBuf::from("content/pathways/tov-01.json"),
//!     PathBuf::from("src/data/tov-01"),
//!     false,
//! );
//!
//! let result = compile(&params).expect("compile failed");
//! println!("Wrote {} files across {} stages", result.files_written.len(), result.stages_compiled);
//! ```

pub mod typescript;

use std::path::PathBuf;

use serde::Serialize;

use crate::error::{ForgeError, ForgeResult};

// ═══════════════════════════════════════════════════════════════════════════
// Public types
// ═══════════════════════════════════════════════════════════════════════════

/// Parameters for the compile operation.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct CompileParams {
    /// Path to the source pathway JSON file (e.g. `content/pathways/tov-01.json`).
    pub input_path: PathBuf,
    /// Directory to write TypeScript output files into.
    ///
    /// The `stages/` subdirectory is created automatically.
    pub output_dir: PathBuf,
    /// When `false`, existing TypeScript files are not overwritten.
    /// Set to `true` to regenerate all output unconditionally.
    pub overwrite: bool,
}

impl CompileParams {
    /// Construct a new [`CompileParams`].
    ///
    /// # Examples
    ///
    /// ```
    /// use academy_forge::CompileParams;
    /// use std::path::PathBuf;
    ///
    /// let params = CompileParams::new(
    ///     PathBuf::from("content/pathways/tov-01.json"),
    ///     PathBuf::from("output/tov-01"),
    ///     false,
    /// );
    /// assert_eq!(params.overwrite, false);
    /// ```
    #[must_use]
    pub fn new(input_path: PathBuf, output_dir: PathBuf, overwrite: bool) -> Self {
        Self {
            input_path,
            output_dir,
            overwrite,
        }
    }
}

/// Result of a successful compile operation.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct CompileResult {
    /// Absolute paths of all TypeScript files written.
    pub files_written: Vec<PathBuf>,
    /// Number of stages compiled (equals the number of stage files written).
    pub stages_compiled: usize,
    /// Non-fatal warnings encountered during compilation.
    ///
    /// These do not prevent output from being written, but may indicate
    /// content that needs manual review (e.g. very long duration strings).
    pub warnings: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Entry point
// ═══════════════════════════════════════════════════════════════════════════

/// Transform a forge pathway JSON file into Studio-compatible TypeScript files.
///
/// # What it does
///
/// 1. Reads and parses the pathway JSON from `params.input_path`.
/// 2. Creates `params.output_dir/stages/` if it does not exist.
/// 3. For each stage, calls [`typescript::render_stage`] and writes to
///    `stages/NN-slug.ts`.
/// 4. Generates `config.ts` via [`typescript::render_config`].
/// 5. Generates `index.ts` via [`typescript::render_index`].
/// 6. Returns a [`CompileResult`] with all written paths and any warnings.
///
/// # Errors
///
/// - [`ForgeError::IoError`] — file read/write failures.
/// - [`ForgeError::ParseError`] — malformed JSON or missing required fields.
///
/// # Example
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use academy_forge::compile::{compile, CompileParams};
///
/// let params = CompileParams::new(
///     PathBuf::from("content/pathways/tov-01.json"),
///     PathBuf::from("src/data/tov-01"),
///     true,
/// );
/// let result = compile(&params).unwrap();
///
/// assert!(result.stages_compiled > 0);
/// ```
pub fn compile(params: &CompileParams) -> ForgeResult<CompileResult> {
    // ── 1. Read and parse the pathway JSON ──────────────────────────────────
    let json_bytes = std::fs::read(&params.input_path).map_err(|source| ForgeError::IoError {
        path: params.input_path.clone(),
        source,
    })?;

    let pathway: serde_json::Value =
        serde_json::from_slice(&json_bytes).map_err(|e| ForgeError::ParseError {
            file: params
                .input_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("pathway.json")
                .to_string(),
            message: e.to_string(),
        })?;

    let stages = pathway
        .get("stages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| ForgeError::ParseError {
            file: params.input_path.display().to_string(),
            message: "pathway JSON missing 'stages' array".to_string(),
        })?;

    // ── 2. Create output directories ────────────────────────────────────────
    let stages_dir = params.output_dir.join("stages");
    std::fs::create_dir_all(&stages_dir).map_err(|source| ForgeError::IoError {
        path: stages_dir.clone(),
        source,
    })?;

    let mut files_written: Vec<PathBuf> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut stage_imports: Vec<(String, String)> = Vec::new();

    // ── 3. Compile each stage ───────────────────────────────────────────────
    for (idx, stage) in stages.iter().enumerate() {
        let stage_num = idx.saturating_add(1);
        let var_name = format!("stage{stage_num:02}");

        // Derive slug from stage title
        let title = stage
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("stage");
        let slug = typescript::slugify(title);
        let file_stem = format!("{stage_num:02}-{slug}");
        let ts_path = stages_dir.join(format!("{file_stem}.ts"));

        // Respect overwrite flag
        if !params.overwrite && ts_path.exists() {
            warnings.push(format!(
                "skipped existing file (overwrite=false): {}",
                ts_path.display()
            ));
            stage_imports.push((var_name, file_stem));
            continue;
        }

        // Extract passing score for this stage
        let passing_score = stage["passingScore"].as_u64().unwrap_or(70);

        // Render TypeScript content
        let ts_content = typescript::render_stage(stage, &var_name, passing_score)?;

        // Write stage file
        std::fs::write(&ts_path, &ts_content).map_err(|source| ForgeError::IoError {
            path: ts_path.clone(),
            source,
        })?;

        files_written.push(ts_path);
        stage_imports.push((var_name, file_stem));
    }

    let stages_compiled = stage_imports.len();

    // ── 4. Generate config.ts ───────────────────────────────────────────────
    let config_path = params.output_dir.join("config.ts");
    if params.overwrite || !config_path.exists() {
        let config_content = typescript::render_config(&pathway)?;
        std::fs::write(&config_path, &config_content).map_err(|source| ForgeError::IoError {
            path: config_path.clone(),
            source,
        })?;
        files_written.push(config_path);
    } else {
        warnings.push(format!(
            "skipped existing file (overwrite=false): {}",
            config_path.display()
        ));
    }

    // ── 5. Generate index.ts ─────────────────────────────────────────────────
    let index_path = params.output_dir.join("index.ts");
    if params.overwrite || !index_path.exists() {
        let index_content = typescript::render_index(&stage_imports, stages_compiled)?;
        std::fs::write(&index_path, &index_content).map_err(|source| ForgeError::IoError {
            path: index_path.clone(),
            source,
        })?;
        files_written.push(index_path);
    } else {
        warnings.push(format!(
            "skipped existing file (overwrite=false): {}",
            index_path.display()
        ));
    }

    Ok(CompileResult {
        files_written,
        stages_compiled,
        warnings,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn tov_json_path() -> PathBuf {
        // Workspace root is two levels above the crate manifest directory.
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .join("content/pathways/tov-01.json")
    }

    #[test]
    fn compile_tov01_into_tempdir() {
        let json_path = tov_json_path();
        if !json_path.exists() {
            // Skip gracefully if running outside the full workspace.
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        let params = CompileParams {
            input_path: json_path,
            output_dir: tmp.path().to_path_buf(),
            overwrite: true,
        };

        let result = compile(&params).unwrap();

        // At least one stage must have compiled; exact count depends on tov-01.json content
        assert!(
            result.stages_compiled > 0,
            "expected at least one stage from tov-01.json, got 0"
        );
        assert!(
            result.warnings.is_empty(),
            "unexpected warnings: {:?}",
            result.warnings
        );

        // config.ts and index.ts always written in addition to stage files
        let expected_file_count = result.stages_compiled + 2;
        assert_eq!(
            result.files_written.len(),
            expected_file_count,
            "file count mismatch: expected {} stage files + config + index",
            result.stages_compiled
        );

        // All files must exist
        for path in &result.files_written {
            assert!(path.exists(), "missing output file: {}", path.display());
        }
    }

    #[test]
    fn compile_overwrite_false_skips_existing() {
        let json_path = tov_json_path();
        if !json_path.exists() {
            return;
        }

        let tmp = tempfile::tempdir().unwrap();

        // First compile — writes everything
        let params = CompileParams {
            input_path: json_path.clone(),
            output_dir: tmp.path().to_path_buf(),
            overwrite: true,
        };
        compile(&params).unwrap();

        // Second compile with overwrite=false — should produce warnings
        let params2 = CompileParams {
            input_path: json_path,
            output_dir: tmp.path().to_path_buf(),
            overwrite: false,
        };
        let result2 = compile(&params2).unwrap();

        // No new files should be written (all skipped)
        assert_eq!(result2.files_written.len(), 0);
        assert!(!result2.warnings.is_empty(), "expected skip warnings");
    }

    #[test]
    fn compile_missing_input_returns_io_error() {
        let tmp = tempfile::tempdir().unwrap();
        let params = CompileParams {
            input_path: PathBuf::from("/nonexistent/path/pathway.json"),
            output_dir: tmp.path().to_path_buf(),
            overwrite: true,
        };

        let result = compile(&params);
        assert!(matches!(result, Err(ForgeError::IoError { .. })));
    }

    #[test]
    fn compile_invalid_json_returns_parse_error() {
        let tmp = tempfile::tempdir().unwrap();
        let bad_json = tmp.path().join("bad.json");
        std::fs::write(&bad_json, b"not valid json at all").unwrap();

        let params = CompileParams {
            input_path: bad_json,
            output_dir: tmp.path().to_path_buf(),
            overwrite: true,
        };

        let result = compile(&params);
        assert!(matches!(result, Err(ForgeError::ParseError { .. })));
    }

    #[test]
    fn compile_minimal_pathway_produces_files() {
        let tmp = tempfile::tempdir().unwrap();

        // Write a minimal valid pathway JSON
        let pathway_json = serde_json::json!({
            "id": "test-01",
            "title": "Test Pathway",
            "description": "A minimal test pathway.",
            "domain": "test",
            "componentCount": 2,
            "estimatedDuration": "30 minutes",
            "stages": [
                {
                    "id": "test-01-01",
                    "title": "Introduction",
                    "description": "Intro stage.",
                    "passingScore": 70,
                    "estimatedDuration": "15 minutes",
                    "activities": [
                        {
                            "id": "test-01-01-a01",
                            "title": "Reading",
                            "type": "reading",
                            "estimatedDuration": "10 minutes"
                        },
                        {
                            "id": "test-01-01-a02",
                            "title": "Quiz",
                            "type": "quiz",
                            "estimatedDuration": "5 minutes",
                            "quiz": {
                                "questions": [
                                    {
                                        "id": "test-01-01-q01",
                                        "type": "multiple-choice",
                                        "question": "What is 1+1?",
                                        "options": ["2", "3", "4", "5"],
                                        "correctAnswer": 0,
                                        "points": 1,
                                        "explanation": "Basic arithmetic."
                                    }
                                ]
                            }
                        }
                    ]
                }
            ]
        });

        let json_path = tmp.path().join("test-01.json");
        std::fs::write(
            &json_path,
            serde_json::to_string_pretty(&pathway_json).unwrap(),
        )
        .unwrap();

        let output_dir = tmp.path().join("output");
        let params = CompileParams {
            input_path: json_path,
            output_dir: output_dir.clone(),
            overwrite: true,
        };

        let result = compile(&params).unwrap();

        assert_eq!(result.stages_compiled, 1);
        // 1 stage + config + index = 3 files
        assert_eq!(result.files_written.len(), 3);

        // Verify stage file content
        let stage_file = output_dir.join("stages/01-introduction.ts");
        assert!(stage_file.exists());
        let content = std::fs::read_to_string(&stage_file).unwrap();
        assert!(content.contains("export const stage01: CapabilityStage = {"));
        assert!(content.contains("id: 'test-01-01'"));
        assert!(content.contains("type: 'quiz'"));
        assert!(content.contains("passingScore: 70"));
        assert!(content.contains("correctAnswer: 0,"));

        // Verify config.ts
        let config_file = output_dir.join("config.ts");
        assert!(config_file.exists());
        let config_content = std::fs::read_to_string(&config_file).unwrap();
        assert!(config_content.contains("id: 'test-01'"));
        assert!(config_content.contains("estimatedDuration: 30,"));

        // Verify index.ts
        let index_file = output_dir.join("index.ts");
        assert!(index_file.exists());
        let index_content = std::fs::read_to_string(&index_file).unwrap();
        assert!(index_content.contains("import { stage01 } from './stages/01-introduction'"));
        assert!(index_content.contains("export const pathway: StaticPathway = {"));
    }
}
