//! Ref Integrity Checker - Atomic Hook
//!
//! PostToolUse hook that validates file references in SKILL.md files
//! resolve to actual paths on the filesystem.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Skill Quality Hook)
//! - **Primitives**: ∂(boundary-aware parsing), κ(comparison against filesystem)
//!
//! # Cytokine Integration
//! - **Warn**: Emits IL-6 (acute response) via cytokine bridge
//! - **Pass**: No emission (homeostasis maintained)

use nexcore_hook_lib::cytokine::emit_check_failed;
use nexcore_hook_lib::{
    Confidence, EvidenceLine, Violation, file_path_or_pass, pass, read_input, require_edit_tool,
    warn,
};
use regex::Regex;
use std::collections::HashSet;

const HOOK_NAME: &str = "ref-integrity-checker";
use std::path::Path;

/// Maximum violations to report before truncating.
const MAX_REPORT: usize = 10;

/// File extensions recognized as valid reference targets.
const REF_EXTENSIONS: &str = r"\.(?:md|sh|py|rs|toml|json|yaml|yml)";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    require_edit_tool(input.tool_name.clone());

    let file_path = file_path_or_pass(&input);

    if !file_path.ends_with("SKILL.md") {
        pass();
    }

    let content = input
        .tool_input
        .as_ref()
        .and_then(|t| t.new_string.as_deref().or(t.content.as_deref()))
        .unwrap_or("");

    if content.is_empty() {
        pass();
    }

    let skill_dir = match Path::new(file_path).parent() {
        Some(d) => d,
        None => pass(),
    };

    let violations = check_references(content, skill_dir);

    if violations.is_empty() {
        pass();
    }

    let msg = format_ref_warnings(&violations);
    // Emit cytokine signal before warning (IL-6 = acute response)
    emit_check_failed(HOOK_NAME, &msg);
    warn(&msg);
}

// ── Reference Extraction (∂ boundary-aware) ────────────────────────

/// A file reference extracted from SKILL.md.
#[derive(Debug)]
struct ExtractedRef {
    line_num: usize,
    raw_match: String,
    is_cross_skill: bool,
}

/// Build regex for local refs: `references/file.ext`, `scripts/file.ext`.
fn local_regex() -> Regex {
    let pattern =
        format!(r"(?:^|[\s`(\[])((references|scripts|templates)/[\w./-]+{REF_EXTENSIONS})");
    Regex::new(&pattern).unwrap_or_else(|_| pass())
}

/// Build regex for cross-skill refs: `skill-name/references/file.ext`.
fn cross_skill_regex() -> Regex {
    let pattern = format!(
        r"(?:^|[\s`(\[])(([\w-]+)/(references|scripts|templates)/[\w./-]+{REF_EXTENSIONS})"
    );
    Regex::new(&pattern).unwrap_or_else(|_| pass())
}

/// Should this line be skipped during ref extraction?
fn skip_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("```") || trimmed.starts_with("http")
}

/// Extract cross-skill refs from a single line.
fn extract_cross_refs(
    line: &str,
    line_num: usize,
    re: &Regex,
    seen: &mut HashSet<String>,
) -> Vec<ExtractedRef> {
    re.captures_iter(line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|raw| seen.insert(raw.clone()))
        .map(|raw| ExtractedRef {
            line_num,
            raw_match: raw,
            is_cross_skill: true,
        })
        .collect()
}

/// Extract local refs from a single line.
fn extract_local_refs(
    line: &str,
    line_num: usize,
    re: &Regex,
    seen: &mut HashSet<String>,
) -> Vec<ExtractedRef> {
    re.captures_iter(line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|raw| seen.insert(raw.clone()))
        .map(|raw| ExtractedRef {
            line_num,
            raw_match: raw,
            is_cross_skill: false,
        })
        .collect()
}

/// Extract all file references from SKILL.md content.
fn extract_refs(content: &str) -> Vec<ExtractedRef> {
    let local_re = local_regex();
    let cross_re = cross_skill_regex();
    let mut seen = HashSet::new();
    let mut refs = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        if skip_line(line) {
            continue;
        }
        let line_num = idx + 1;
        refs.extend(extract_cross_refs(line, line_num, &cross_re, &mut seen));
        refs.extend(extract_local_refs(line, line_num, &local_re, &mut seen));
    }

    refs
}

// ── Validation (κ comparison) ──────────────────────────────────────

/// Resolve a ref path to an absolute path and check existence.
fn resolve_ref(extracted: &ExtractedRef, skill_dir: &Path) -> Option<Violation> {
    let check_path = if extracted.is_cross_skill {
        let skills_root = skill_dir.parent().unwrap_or(skill_dir);
        skills_root.join(&extracted.raw_match)
    } else {
        skill_dir.join(&extracted.raw_match)
    };

    if check_path.exists() {
        return None;
    }

    let kind = if extracted.is_cross_skill {
        "broken cross-skill ref"
    } else {
        "broken local ref"
    };
    let snippet = format!("{} → {}", extracted.raw_match, check_path.display());
    let evidence = EvidenceLine::new(extracted.line_num, kind, snippet);
    Some(Violation::new(evidence, Confidence(0.9)))
}

/// Check all references and return violations for broken ones.
fn check_references(content: &str, skill_dir: &Path) -> Vec<Violation> {
    extract_refs(content)
        .iter()
        .filter_map(|r| resolve_ref(r, skill_dir))
        .take(MAX_REPORT)
        .collect()
}

// ── Formatting ─────────────────────────────────────────────────────

/// Format violations into a warning message.
fn format_ref_warnings(violations: &[Violation]) -> String {
    let mut msg = format!(
        "SKILL.MD REF INTEGRITY: {} broken reference(s)\n\n",
        violations.len()
    );

    for (i, v) in violations.iter().enumerate() {
        let line = v.evidence.line.0;
        let kind = &v.evidence.kind.0;
        let snippet = &v.evidence.snippet.0;
        msg.push_str(&format!("  {}. L{}: [{}] {}\n", i + 1, line, kind, snippet));
    }

    if violations.len() >= MAX_REPORT {
        msg.push_str("  ... (truncated)\n");
    }

    msg.push_str("\nFix: Create missing files or update references.");
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_extract_local_refs_basic() {
        let content = "See `references/phases.md` for details.\nRun `scripts/build.sh`.";
        let refs = extract_refs(content);
        assert_eq!(refs.len(), 2);
        assert!(!refs[0].is_cross_skill);
        assert_eq!(refs[0].raw_match, "references/phases.md");
    }

    #[test]
    fn test_extract_cross_skill_refs() {
        let content = "See `primitive-extractor/references/universal_primitives.md`.";
        let refs = extract_refs(content);
        assert_eq!(refs.len(), 1);
        assert!(refs[0].is_cross_skill);
    }

    #[test]
    fn test_deduplicates() {
        let content = "`references/foo.md` and `references/foo.md` again.";
        let refs = extract_refs(content);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_skips_code_fences() {
        let content = "```bash\nsome code\n```";
        let refs = extract_refs(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_check_references_with_temp_dir() {
        let tmp = std::env::temp_dir().join("ref-integrity-test");
        let _ = fs::create_dir_all(tmp.join("references"));
        fs::write(tmp.join("references/exists.md"), "content").ok();

        let content = "`references/exists.md` and `references/missing.md`.";
        let violations = check_references(content, &tmp);

        assert_eq!(violations.len(), 1);
        assert!(violations[0].evidence.snippet.0.contains("missing.md"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
