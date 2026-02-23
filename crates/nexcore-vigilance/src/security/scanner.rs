//! Base security scanner infrastructure.

use crate::security::config::{ScanConfig, SeverityLevel};
use chrono::{DateTime, Utc};
use nexcore_fs::glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A security issue found during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub severity: SeverityLevel,
    pub category: String,
    pub title: String,
    pub description: String,
    pub file_path: PathBuf,
    pub line_number: Option<usize>,
    pub code_snippet: Option<String>,
    pub remediation: Option<String>,
    pub cwe_id: Option<String>,
    pub confidence: String,
    /// ToV Signal Score (S = U x R x T)
    #[serde(default)]
    pub tov_signal: Option<f64>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Results from a security scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub project_path: PathBuf,
    pub scan_time: DateTime<Utc>,
    pub issues: Vec<SecurityIssue>,
    pub files_scanned: usize,
    pub scan_duration_ms: u64,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ScanResult {
    /// Count issues by severity.
    #[must_use]
    pub fn count_by_severity(&self, severity: SeverityLevel) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
    }

    /// Get critical issue count.
    #[must_use]
    pub fn critical_count(&self) -> usize {
        self.count_by_severity(SeverityLevel::Critical)
    }

    /// Get high severity issue count.
    #[must_use]
    pub fn high_count(&self) -> usize {
        self.count_by_severity(SeverityLevel::High)
    }

    /// Get medium severity issue count.
    #[must_use]
    pub fn medium_count(&self) -> usize {
        self.count_by_severity(SeverityLevel::Medium)
    }

    /// Get low severity issue count.
    #[must_use]
    pub fn low_count(&self) -> usize {
        self.count_by_severity(SeverityLevel::Low)
    }

    /// Calculate security score (0-100).
    ///
    /// Score = 100 - (critical * 20 + high * 10 + medium * 5 + low * 2)
    #[must_use]
    pub fn calculate_score(&self) -> u32 {
        let deductions: u32 = self.issues.iter().map(|i| i.severity.weight()).sum();
        100u32.saturating_sub(deductions)
    }

    /// Check if scan should fail based on config.
    #[must_use]
    pub fn should_fail(&self, config: &ScanConfig) -> bool {
        if config.fail_on_critical && self.critical_count() > 0 {
            return true;
        }
        if config.fail_on_high && self.high_count() > 0 {
            return true;
        }
        if self.calculate_score() < config.min_security_score {
            return true;
        }
        false
    }

    /// Get issues grouped by category.
    #[must_use]
    pub fn by_category(&self) -> HashMap<&str, Vec<&SecurityIssue>> {
        let mut grouped: HashMap<&str, Vec<&SecurityIssue>> = HashMap::new();
        for issue in &self.issues {
            grouped
                .entry(issue.category.as_str())
                .or_default()
                .push(issue);
        }
        grouped
    }
}

impl SecurityIssue {
    /// Calculate the ToV signal score for this issue.
    ///
    /// Uses Codex-compliant `SignalEquationResult` from §20-§23.
    pub fn calculate_tov_signal(&mut self) {
        use crate::pv::signals::signal_equation::SignalEquationResult;

        let severity_weight = self.severity.weight() as u64;
        let is_dme = self.severity == SeverityLevel::Critical;

        let result = SignalEquationResult::evaluate(
            severity_weight,
            1.0, // expected (baseline)
            is_dme,
            false, // reported previously (assume new)
            0,     // days since first
            30,    // total days (fixed window)
        );

        self.tov_signal = Some(result.signal_strength.value());
    }
}

/// Base security scanner.
pub struct SecurityScanner {
    root_path: PathBuf,
    config: ScanConfig,
    gitignore_patterns: Vec<String>,
    exclude_globs: Vec<Pattern>,
}

impl SecurityScanner {
    /// Create a new scanner for the given path.
    #[must_use]
    pub fn new(root_path: &Path, config: ScanConfig) -> Self {
        let root_path = root_path.to_path_buf();
        let gitignore_patterns = Self::load_gitignore(&root_path);

        let exclude_globs = config
            .exclude_patterns
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect();

        Self {
            root_path,
            config,
            gitignore_patterns,
            exclude_globs,
        }
    }

    /// Get the root path.
    #[must_use]
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Get the config.
    #[must_use]
    pub fn config(&self) -> &ScanConfig {
        &self.config
    }

    fn load_gitignore(root: &Path) -> Vec<String> {
        let gitignore_path = root.join(".gitignore");
        if !gitignore_path.exists() {
            return Vec::new();
        }

        fs::read_to_string(&gitignore_path)
            .map(|content| {
                content
                    .lines()
                    .filter(|l| !l.is_empty() && !l.starts_with('#'))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a path should be excluded.
    #[must_use]
    pub fn should_exclude(&self, path: &Path) -> bool {
        let relative = match path.strip_prefix(&self.root_path) {
            Ok(r) => r,
            Err(_) => return true,
        };

        let relative_str = relative.to_string_lossy();

        // Check exclude patterns
        for glob in &self.exclude_globs {
            if glob.matches(&relative_str)
                || glob.matches(
                    path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .as_ref(),
                )
            {
                return true;
            }
        }

        // Check gitignore patterns
        for pattern in &self.gitignore_patterns {
            if relative_str.contains(pattern.trim_end_matches('*').trim_end_matches('/')) {
                return true;
            }
        }

        // Check whitelisted paths
        for whitelist in &self.config.whitelisted_paths {
            if relative_str.starts_with(whitelist) {
                return false;
            }
        }

        false
    }

    /// Check if a file should be scanned.
    #[must_use]
    pub fn should_scan_file(&self, path: &Path) -> bool {
        if self.should_exclude(path) {
            return false;
        }

        // Check file size
        if let Ok(metadata) = path.metadata() {
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
            if size_mb > self.config.max_file_size_mb {
                return false;
            }
        }

        // Check extension
        if self.config.scan_extensions.is_empty() {
            return true;
        }

        if let Some(ext) = path.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy().to_lowercase());
            if self.config.scan_extensions.contains(&ext_str) {
                return true;
            }
        }

        // Check config file names
        let config_files = [
            "dockerfile",
            ".dockerignore",
            "makefile",
            ".env",
            ".gitignore",
            "package.json",
            "requirements.txt",
            "gemfile",
            "go.mod",
            "cargo.toml",
        ];

        if let Some(name) = path.file_name() {
            let name_lower = name.to_string_lossy().to_lowercase();
            if config_files.contains(&name_lower.as_str()) {
                return true;
            }
        }

        false
    }

    /// Iterate over all files to scan.
    #[must_use]
    pub fn iter_files(&self) -> Vec<PathBuf> {
        self.collect_files_from(&self.root_path)
    }

    fn collect_files_from(&self, dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return files,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                if !self.should_exclude(&path) {
                    files.extend(self.collect_files_from(&path));
                }
            } else if self.should_scan_file(&path) {
                files.push(path);
            }
        }
        files
    }

    /// Read file contents safely.
    #[must_use]
    pub fn read_file(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path)
            .or_else(|_| {
                // Try latin-1 encoding as fallback
                fs::read(path).map(|bytes| bytes.iter().map(|&b| b as char).collect())
            })
            .ok()
    }

    /// Read file as lines.
    #[must_use]
    pub fn read_file_lines(&self, path: &Path) -> Option<Vec<String>> {
        self.read_file(path)
            .map(|content| content.lines().map(String::from).collect())
    }

    /// Perform a full security scan.
    #[must_use]
    pub fn scan(&self) -> ScanResult {
        let start_time = std::time::Instant::now();
        let mut issues = Vec::new();
        let files = self.iter_files();
        let files_count = files.len();

        // 1. Detect Secrets
        if self.config.detect_secrets {
            let detector = crate::security::SecretsDetector::new(self);
            issues.extend(detector.detect());
        }

        // 2. Analyze Code
        if self.config.analyze_code {
            let analyzer = crate::security::CodeAnalyzer::new(self);
            issues.extend(analyzer.detect());
        }

        // 3. Check Dependencies
        if self.config.check_dependencies {
            let checker = crate::security::DependencyChecker::new(self);
            issues.extend(checker.detect());
        }

        // 4. Axiomatic Scoring
        for issue in &mut issues {
            issue.calculate_tov_signal();
        }

        ScanResult {
            project_path: self.root_path.clone(),
            scan_time: Utc::now(),
            issues,
            files_scanned: files_count,
            scan_duration_ms: start_time.elapsed().as_millis() as u64,
            metadata: HashMap::new(),
        }
    }
}
