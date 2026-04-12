#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! # nexcore-wallace
//!
//! Wallace Protocol scanner — classifies unwrap/expect/clone violations
//! across Rust source files with test-awareness and context analysis.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// A single violation found in source code.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Violation {
    pub file: PathBuf,
    pub line: usize,
    pub kind: ViolationKind,
    pub classification: Classification,
    pub context: String,
}

/// What type of violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ViolationKind {
    Unwrap,
    Expect,
    Clone,
}

/// How the violation is classified after context analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Classification {
    TestCode,
    DocComment,
    Example,
    DetectionCode,
    Allowed,
    MainFunction,
    TypestateInvariant,
    InfallibleStatic,
    Mechanical,
    SignatureLift,
    Movable,
    Unnecessary,
    Necessary,
}

impl Classification {
    pub fn is_actionable(&self) -> bool {
        matches!(
            self,
            Classification::Mechanical
                | Classification::SignatureLift
                | Classification::Movable
                | Classification::Unnecessary
        )
    }
}

/// Per-crate scan results.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CrateReport {
    pub name: String,
    pub violations: Vec<Violation>,
}

impl CrateReport {
    pub fn total(&self) -> usize {
        self.violations.len()
    }

    pub fn actionable(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| v.classification.is_actionable())
            .count()
    }

    pub fn by_kind(&self, kind: ViolationKind) -> usize {
        self.violations.iter().filter(|v| v.kind == kind).count()
    }
}

/// Full workspace scan results.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceReport {
    pub crates: Vec<CrateReport>,
    pub scan_root: PathBuf,
}

impl WorkspaceReport {
    pub fn total_violations(&self) -> usize {
        self.crates.iter().map(|c| c.total()).sum()
    }

    pub fn total_actionable(&self) -> usize {
        self.crates.iter().map(|c| c.actionable()).sum()
    }

    pub fn summary_by_crate(&self) -> BTreeMap<String, (usize, usize)> {
        self.crates
            .iter()
            .map(|c| (c.name.clone(), (c.total(), c.actionable())))
            .collect()
    }
}

/// Scan a crates directory for violations.
pub fn scan_workspace(crates_dir: &Path) -> Result<WorkspaceReport, ScanError> {
    let mut crate_reports: BTreeMap<String, Vec<Violation>> = BTreeMap::new();

    walk_rs_files(crates_dir, &mut |file_path, content| {
        if file_path.components().any(|c| c.as_os_str() == "target") {
            return;
        }
        let crate_name = extract_crate_name(file_path, crates_dir);
        let violations = analyze_file(file_path, content);
        crate_reports
            .entry(crate_name)
            .or_default()
            .extend(violations);
    })?;

    let crates = crate_reports
        .into_iter()
        .map(|(name, violations)| CrateReport { name, violations })
        .collect();

    Ok(WorkspaceReport {
        crates,
        scan_root: crates_dir.to_path_buf(),
    })
}

/// Scan a single crate directory.
pub fn scan_crate(crate_dir: &Path) -> Result<CrateReport, ScanError> {
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return Err(ScanError::NoCrateSource(crate_dir.to_path_buf()));
    }

    let name = crate_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut violations = Vec::new();
    walk_rs_files(&src_dir, &mut |file_path, content| {
        if file_path.components().any(|c| c.as_os_str() == "target") {
            return;
        }
        violations.extend(analyze_file(file_path, content));
    })?;

    Ok(CrateReport { name, violations })
}

fn analyze_file(path: &Path, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let is_test_file = path.file_name().is_some_and(|n| {
        let s = n.to_string_lossy();
        s.ends_with("_test.rs") || s.ends_with("_tests.rs") || s == "tests.rs"
    }) || path.components().any(|c| c.as_os_str() == "tests");
    let is_example = path.components().any(|c| c.as_os_str() == "examples");

    let cfg_test_regions = find_cfg_test_regions(&lines);

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            if (trimmed.starts_with("///") || trimmed.starts_with("//!"))
                && (trimmed.contains(".unwrap()") || trimmed.contains(".expect("))
            {
                let kind = if trimmed.contains(".unwrap()") {
                    ViolationKind::Unwrap
                } else {
                    ViolationKind::Expect
                };
                violations.push(Violation {
                    file: path.to_path_buf(),
                    line: line_num,
                    kind,
                    classification: Classification::DocComment,
                    context: trimmed.to_string(),
                });
            }
            continue;
        }

        if trimmed.contains(".unwrap()") {
            let classification = classify_unwrap_expect(
                trimmed,
                line_num,
                &lines,
                is_test_file,
                is_example,
                &cfg_test_regions,
            );
            violations.push(Violation {
                file: path.to_path_buf(),
                line: line_num,
                kind: ViolationKind::Unwrap,
                classification,
                context: trimmed.to_string(),
            });
        }

        if trimmed.contains(".expect(") && !trimmed.contains("self.expect(") {
            let classification = classify_unwrap_expect(
                trimmed,
                line_num,
                &lines,
                is_test_file,
                is_example,
                &cfg_test_regions,
            );
            violations.push(Violation {
                file: path.to_path_buf(),
                line: line_num,
                kind: ViolationKind::Expect,
                classification,
                context: trimmed.to_string(),
            });
        }

        if trimmed.contains(".clone()") {
            let classification = classify_clone(line_num, &lines, is_test_file, &cfg_test_regions);
            violations.push(Violation {
                file: path.to_path_buf(),
                line: line_num,
                kind: ViolationKind::Clone,
                classification,
                context: trimmed.to_string(),
            });
        }
    }

    violations
}

fn classify_unwrap_expect(
    trimmed: &str,
    line_num: usize,
    lines: &[&str],
    is_test_file: bool,
    is_example: bool,
    cfg_test_regions: &[std::ops::Range<usize>],
) -> Classification {
    if is_test_file || in_cfg_test(line_num, cfg_test_regions) || near_test_attr(line_num, lines) {
        return Classification::TestCode;
    }
    if is_example {
        return Classification::Example;
    }
    if is_in_string_literal(trimmed, ".unwrap()") || is_in_string_literal(trimmed, ".expect(") {
        return Classification::DetectionCode;
    }
    // Multi-line string continuation: previous line ends with `\` (Rust string continuation)
    if line_num >= 2 {
        let prev = lines.get(line_num.saturating_sub(2)).map(|s| s.trim_end());
        if prev.is_some_and(|p| p.ends_with('\\')) {
            return Classification::DetectionCode;
        }
    }
    if has_allow_above(line_num, lines) {
        return Classification::Allowed;
    }
    if is_in_static_init(line_num, lines) {
        return Classification::InfallibleStatic;
    }
    if let Some(true) = enclosing_fn_returns_result(line_num, lines) {
        return Classification::Mechanical;
    }
    if in_main_function(line_num, lines) {
        return Classification::MainFunction;
    }
    Classification::SignatureLift
}

fn classify_clone(
    line_num: usize,
    lines: &[&str],
    is_test_file: bool,
    cfg_test_regions: &[std::ops::Range<usize>],
) -> Classification {
    if is_test_file || in_cfg_test(line_num, cfg_test_regions) || near_test_attr(line_num, lines) {
        return Classification::TestCode;
    }
    // Conservative: future version could do liveness analysis
    Classification::Necessary
}

fn find_cfg_test_regions(lines: &[&str]) -> Vec<std::ops::Range<usize>> {
    let mut regions = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim().contains("#[cfg(test)]") {
            let start = i + 1;
            let mut depth = 0i32;
            let mut j = start;
            while j < lines.len() {
                for ch in lines[j].chars() {
                    if ch == '{' {
                        depth += 1;
                    }
                    if ch == '}' {
                        depth -= 1;
                    }
                }
                if depth <= 0 && j > start {
                    regions.push(start..j + 1);
                    break;
                }
                j += 1;
            }
            // If file ends inside cfg(test) module, close region at EOF
            if j >= lines.len() && depth > 0 {
                regions.push(start..lines.len());
            }
            i = if j < lines.len() { j + 1 } else { j };
        } else {
            i += 1;
        }
    }
    regions
}

fn in_cfg_test(line_num: usize, regions: &[std::ops::Range<usize>]) -> bool {
    let idx = line_num.saturating_sub(1);
    regions.iter().any(|r| r.contains(&idx))
}

fn near_test_attr(line_num: usize, lines: &[&str]) -> bool {
    // Look back up to 30 lines for #[test], checking brace depth
    let start = line_num.saturating_sub(31);
    for i in (start..line_num.saturating_sub(1)).rev() {
        if i < lines.len() && lines[i].trim() == "#[test]" {
            // Verify we're still inside the test fn's braces
            let mut depth = 0i32;
            for j in i..line_num {
                if j < lines.len() {
                    for ch in lines[j].chars() {
                        if ch == '{' {
                            depth += 1;
                        }
                        if ch == '}' {
                            depth -= 1;
                        }
                    }
                }
            }
            if depth > 0 {
                return true;
            }
        }
    }
    false
}

fn has_allow_above(line_num: usize, lines: &[&str]) -> bool {
    let check_start = line_num.saturating_sub(8);
    for i in check_start..line_num.saturating_sub(1) {
        if i < lines.len() {
            let t = lines[i].trim();
            if t.contains("#[allow(clippy::expect_used)]")
                || t.contains("#[allow(clippy::unwrap_used)]")
            {
                return true;
            }
        }
    }
    false
}

fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    if let Some(pat_pos) = line.find(pattern) {
        let before = &line[..pat_pos];
        let quote_count = before.chars().filter(|&c| c == '"').count();
        quote_count % 2 == 1
    } else {
        false
    }
}

fn is_in_static_init(line_num: usize, lines: &[&str]) -> bool {
    let start = line_num.saturating_sub(6);
    for i in start..line_num.saturating_sub(1) {
        if i < lines.len() {
            let t = lines[i].trim();
            if t.starts_with("static ") && t.contains("LazyLock") {
                return true;
            }
        }
    }
    false
}

fn enclosing_fn_returns_result(line_num: usize, lines: &[&str]) -> Option<bool> {
    // Walk backward tracking brace depth to find the enclosing fn
    let mut depth = 0i32;
    let mut fn_line_idx = None;
    for i in (0..line_num.saturating_sub(1)).rev() {
        if i < lines.len() {
            // Walking backward: } means entering nested scope, { means exiting
            for ch in lines[i].chars() {
                if ch == '}' {
                    depth += 1;
                }
                if ch == '{' {
                    depth -= 1;
                }
            }
            let t = lines[i].trim();
            if depth <= 0
                && t.contains("fn ")
                && (t.starts_with("pub")
                    || t.starts_with("fn")
                    || t.starts_with("async")
                    || t.starts_with("pub(crate)"))
            {
                fn_line_idx = Some(i);
                break;
            }
        }
    }

    let fn_idx = fn_line_idx?;
    let mut sig = String::new();
    for i in fn_idx..lines.len().min(fn_idx + 10) {
        sig.push_str(lines[i]);
        sig.push(' ');
        if lines[i].contains('{') {
            break;
        }
    }

    let sig_lower = sig.to_lowercase();
    Some(
        sig_lower.contains("result")
            || sig_lower.contains("option")
            || sig_lower.contains("anyhow"),
    )
}

fn in_main_function(line_num: usize, lines: &[&str]) -> bool {
    let start = line_num.saturating_sub(100);
    for i in (start..line_num.saturating_sub(1)).rev() {
        if i < lines.len() {
            let t = lines[i].trim();
            if t.starts_with("fn main(") {
                return true;
            }
            if t.contains("fn ")
                && !t.contains("fn main")
                && (t.starts_with("pub") || t.starts_with("fn") || t.starts_with("async"))
            {
                return false;
            }
        }
    }
    false
}

fn extract_crate_name(file_path: &Path, crates_dir: &Path) -> String {
    file_path
        .strip_prefix(crates_dir)
        .ok()
        .and_then(|rel| rel.components().next())
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn walk_rs_files(dir: &Path, callback: &mut dyn FnMut(&Path, &str)) -> Result<(), ScanError> {
    if !dir.exists() {
        return Err(ScanError::DirNotFound(dir.to_path_buf()));
    }
    walk_dir_recursive(dir, callback)
}

fn walk_dir_recursive(dir: &Path, callback: &mut dyn FnMut(&Path, &str)) -> Result<(), ScanError> {
    let entries = std::fs::read_dir(dir).map_err(|e| ScanError::Io(dir.to_path_buf(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| ScanError::Io(dir.to_path_buf(), e))?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            walk_dir_recursive(&path, callback)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                callback(&path, &content);
            }
        }
    }
    Ok(())
}

/// Errors from scanning.
#[derive(Debug)]
pub enum ScanError {
    DirNotFound(PathBuf),
    NoCrateSource(PathBuf),
    Io(PathBuf, std::io::Error),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::DirNotFound(p) => write!(f, "directory not found: {}", p.display()),
            ScanError::NoCrateSource(p) => write!(f, "no src/ in crate: {}", p.display()),
            ScanError::Io(p, e) => write!(f, "I/O error at {}: {e}", p.display()),
        }
    }
}

impl std::error::Error for ScanError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn analyze(src: &str) -> Vec<Violation> {
        analyze_file(Path::new("test.rs"), src)
    }

    #[test]
    fn unwrap_in_test_fn_classified_as_test() {
        let src = "#[test]\nfn my_test() {\n    let x = Some(1).unwrap();\n}\n";
        let violations = analyze(src);
        let unwraps: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Unwrap)
            .collect();
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].classification, Classification::TestCode);
    }

    #[test]
    fn unwrap_in_cfg_test_module() {
        let src = "pub fn real() -> i32 { 42 }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it() {\n        let v = vec![1].first().unwrap();\n    }\n}\n";
        let violations = analyze(src);
        let unwraps: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Unwrap)
            .collect();
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].classification, Classification::TestCode);
    }

    #[test]
    fn doc_comment_unwrap() {
        let src =
            "/// let x = compute().unwrap();\npub fn compute() -> Result<i32, String> { Ok(42) }\n";
        let violations = analyze(src);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].classification, Classification::DocComment);
    }

    #[test]
    fn detection_code_string_literal() {
        let src = "pub fn check(code: &str) -> bool {\n    code.contains(\".unwrap()\")\n}\n";
        let violations = analyze(src);
        let unwraps: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Unwrap)
            .collect();
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].classification, Classification::DetectionCode);
    }

    #[test]
    fn allowed_annotation() {
        let src = "pub fn safe_push(v: &mut Vec<i32>) -> &i32 {\n    v.push(42);\n    #[allow(clippy::expect_used)]\n    v.last().expect(\"just pushed\")\n}\n";
        let violations = analyze(src);
        let expects: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Expect)
            .collect();
        assert_eq!(expects.len(), 1);
        assert_eq!(expects[0].classification, Classification::Allowed);
    }

    #[test]
    fn mechanical_in_result_fn() {
        let src = "pub fn parse_num(s: &str) -> Result<i32, String> {\n    let val = s.parse::<i32>().unwrap();\n    Ok(val)\n}\n";
        let violations = analyze(src);
        let unwraps: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Unwrap)
            .collect();
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].classification, Classification::Mechanical);
    }

    #[test]
    fn signature_lift_in_non_result_fn() {
        let src = "pub fn get_name(id: u64) -> String {\n    let data = fetch(id).unwrap();\n    data.name\n}\n";
        let violations = analyze(src);
        let unwraps: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Unwrap)
            .collect();
        assert_eq!(unwraps.len(), 1);
        assert_eq!(unwraps[0].classification, Classification::SignatureLift);
    }

    #[test]
    fn clone_in_test_is_test_code() {
        let src = "#[cfg(test)]\nmod tests {\n    fn helper() {\n        let s = \"hello\".to_string();\n        let s2 = s.clone();\n    }\n}\n";
        let violations = analyze(src);
        let clones: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Clone)
            .collect();
        assert_eq!(clones.len(), 1);
        assert_eq!(clones[0].classification, Classification::TestCode);
    }

    #[test]
    fn clone_in_production_is_necessary() {
        let src = "pub fn dup(s: &str) -> String {\n    let owned = s.to_string();\n    owned.clone()\n}\n";
        let violations = analyze(src);
        let clones: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Clone)
            .collect();
        assert_eq!(clones.len(), 1);
        assert_eq!(clones[0].classification, Classification::Necessary);
    }

    #[test]
    fn parser_self_expect_excluded() {
        let src = "pub fn parse_thing(&mut self) -> Result<Ast, Error> {\n    self.expect(&TokenKind::LParen)?;\n    Ok(Ast::new())\n}\n";
        let violations = analyze(src);
        let expects: Vec<_> = violations
            .iter()
            .filter(|v| v.kind == ViolationKind::Expect)
            .collect();
        assert_eq!(expects.len(), 0);
    }

    #[test]
    fn scan_crate_missing_src_errors() {
        let result = scan_crate(Path::new("/nonexistent/crate"));
        assert!(result.is_err());
    }

    #[test]
    fn classification_actionable_flags() {
        assert!(Classification::Mechanical.is_actionable());
        assert!(Classification::SignatureLift.is_actionable());
        assert!(!Classification::TestCode.is_actionable());
        assert!(!Classification::Allowed.is_actionable());
        assert!(!Classification::Necessary.is_actionable());
    }
}
