//! Skill token analysis tools
//!
//! Analyzes token usage in skill files to optimize context window consumption.
//! Provides file-level metrics, total estimates, and recommendations.

use nexcore_fs::dirs;
use std::path::Path;

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Analysis result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    /// Relative path from skill root
    pub file: String,
    /// Character count
    pub chars: usize,
    /// Estimated token count (~4 chars per token)
    pub tokens_est: usize,
    /// Line count
    pub lines: usize,
    /// Number of code blocks (``` pairs)
    pub code_blocks: usize,
}

/// Complete skill token analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTokenAnalysis {
    /// Skill name (directory name)
    pub skill: String,
    /// Per-file analysis
    pub files: Vec<FileAnalysis>,
    /// Total estimated tokens
    pub total_tokens: usize,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Estimate tokens from text (~4 chars per token, matching Python implementation)
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Count code blocks (``` pairs)
fn count_code_blocks(text: &str) -> usize {
    text.matches("```").count() / 2
}

/// Analyze a single file and return metrics
fn analyze_file(filepath: &Path, skill_root: &Path) -> Option<FileAnalysis> {
    let content = std::fs::read_to_string(filepath).ok()?;
    let relative = filepath
        .strip_prefix(skill_root)
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| filepath.display().to_string());

    Some(FileAnalysis {
        file: relative,
        chars: content.len(),
        tokens_est: estimate_tokens(&content),
        lines: content.lines().count().max(1),
        code_blocks: count_code_blocks(&content),
    })
}

/// Scan a directory and analyze all files matching the filter
fn scan_directory(dir: &Path, skill_root: &Path, filter: fn(&Path) -> bool) -> Vec<FileAnalysis> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && filter(&path) {
                analyze_file(&path, skill_root)
            } else {
                None
            }
        })
        .collect()
}

/// Filter: accept all files
fn accept_all(_path: &Path) -> bool {
    true
}

/// Filter: accept only text-based template files
fn accept_text_templates(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(ext, "json" | "md" | "txt" | "yaml" | "yml" | "toml")
}

/// Generate recommendations based on file analysis
fn generate_recommendations(files: &[FileAnalysis], total_tokens: usize) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check SKILL.md size
    for f in files.iter().filter(|f| f.file == "SKILL.md") {
        if f.tokens_est > 2000 {
            recommendations.push(format!(
                "SKILL.md is large ({} tokens). Consider splitting into references/.",
                f.tokens_est
            ));
        }
    }

    // Check for very large reference files
    for f in files.iter().filter(|f| f.file.starts_with("references/")) {
        if f.tokens_est > 3000 {
            recommendations.push(format!(
                "{} is very large ({} tokens). Consider condensing.",
                f.file, f.tokens_est
            ));
        }
    }

    // Total size recommendations
    if total_tokens > 10000 {
        recommendations.push(format!(
            "Total skill size ({} tokens) exceeds recommended limit. Consider nested skills.",
            total_tokens
        ));
    } else if total_tokens > 5000 {
        recommendations.push(format!(
            "Total skill size ({} tokens) is moderately large. Monitor context usage.",
            total_tokens
        ));
    }

    recommendations
}

/// Extract skill name from path
fn extract_skill_name(skill_path: &Path) -> String {
    skill_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Analyze a skill directory for token usage
fn analyze_skill(skill_path: &Path) -> SkillTokenAnalysis {
    let skill_name = extract_skill_name(skill_path);
    let mut files = Vec::new();

    // Analyze SKILL.md
    let skill_md = skill_path.join("SKILL.md");
    if let Some(analysis) = analyze_file(&skill_md, skill_path) {
        files.push(analysis);
    }

    // Analyze subdirectories
    files.extend(scan_directory(
        &skill_path.join("references"),
        skill_path,
        accept_all,
    ));
    files.extend(scan_directory(
        &skill_path.join("templates"),
        skill_path,
        accept_text_templates,
    ));
    files.extend(scan_directory(
        &skill_path.join("scripts"),
        skill_path,
        accept_all,
    ));

    // Calculate total
    let total_tokens: usize = files.iter().map(|f| f.tokens_est).sum();

    // Generate recommendations
    let recommendations = generate_recommendations(&files, total_tokens);

    // Sort by token count descending
    files.sort_by(|a, b| b.tokens_est.cmp(&a.tokens_est));

    SkillTokenAnalysis {
        skill: skill_name,
        files,
        total_tokens,
        recommendations,
    }
}

/// Expand tilde in path (e.g., ~/foo -> /home/user/foo)
fn expand_tilde(path: &str) -> String {
    if !path.starts_with("~/") {
        return path.to_string();
    }
    match dirs::home_dir() {
        Some(home) => format!("{}{}", home.display(), &path[1..]),
        None => path.to_string(),
    }
}

/// Resolve path to skill directory
fn resolve_skill_path(path: &Path) -> Option<std::path::PathBuf> {
    if path.join("SKILL.md").exists() {
        return Some(path.to_path_buf());
    }

    // If they passed SKILL.md directly, use parent
    let is_skill_md = path.is_file() && path.file_name().map(|n| n == "SKILL.md").unwrap_or(false);
    if is_skill_md {
        return path.parent().map(|p| p.to_path_buf());
    }

    None
}

/// Analyze token usage for a skill
///
/// Returns JSON with:
/// - Per-file analysis (chars, tokens, lines, code blocks)
/// - Total estimated tokens
/// - Optimization recommendations
pub fn analyze(params: crate::params::SkillTokenAnalyzeParams) -> Result<CallToolResult, McpError> {
    let expanded = expand_tilde(&params.path);
    let path = Path::new(&expanded);

    if !path.exists() {
        let json = json!({ "error": format!("Path does not exist: {}", params.path) });
        return Ok(CallToolResult::success(vec![Content::text(
            json.to_string(),
        )]));
    }

    let skill_path = match resolve_skill_path(path) {
        Some(p) => p,
        None => {
            let json = json!({
                "error": "Path is not a skill directory (no SKILL.md found)",
                "path": params.path,
                "hint": "Provide a path to a skill directory containing SKILL.md",
            });
            return Ok(CallToolResult::success(vec![Content::text(
                json.to_string(),
            )]));
        }
    };

    let analysis = analyze_skill(&skill_path);

    let json = json!({
        "skill": analysis.skill,
        "files": analysis.files,
        "total_tokens": analysis.total_tokens,
        "recommendations": analysis.recommendations,
        "token_estimation": "~4 chars per token (GPT-style approximation)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string()),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcdefgh"), 2);
    }

    #[test]
    fn test_count_code_blocks() {
        assert_eq!(count_code_blocks("no blocks"), 0);
        assert_eq!(count_code_blocks("```rust\ncode\n```"), 1);
        assert_eq!(count_code_blocks("```a```\n```b```"), 2);
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/test");
        assert!(!expanded.starts_with("~/") || dirs::home_dir().is_none());
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_accept_text_templates() {
        assert!(accept_text_templates(Path::new("foo.json")));
        assert!(accept_text_templates(Path::new("bar.md")));
        assert!(!accept_text_templates(Path::new("binary.exe")));
    }
}
