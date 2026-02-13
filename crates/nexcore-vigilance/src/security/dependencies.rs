//! Dependency vulnerability checking.
//!
//! Parses dependency files and checks for known vulnerabilities:
//! - package.json (npm)
//! - requirements.txt (pip)
//! - go.mod (Go)
//! - Cargo.toml (Rust)

use crate::security::config::SeverityLevel;
use crate::security::scanner::{SecurityIssue, SecurityScanner};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A project dependency.
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub source_file: PathBuf,
    pub line_number: usize,
    pub package_manager: String,
}

/// Known vulnerability record.
#[derive(Debug, Clone)]
pub struct KnownVulnerability {
    pub package: &'static str,
    pub version_pattern: &'static str,
    pub severity: SeverityLevel,
    pub cve_id: &'static str,
    pub description: &'static str,
}

/// Sample known vulnerabilities (in production, use NVD/OSV API).
pub const KNOWN_VULNERABILITIES: &[KnownVulnerability] = &[
    KnownVulnerability {
        package: "lodash",
        version_pattern: r"^4\.[0-9]\.[0-9]$",
        severity: SeverityLevel::High,
        cve_id: "CVE-2020-8203",
        description: "Prototype pollution in lodash < 4.17.19",
    },
    KnownVulnerability {
        package: "axios",
        version_pattern: r"^0\.[0-1][0-9]\.[0-9]$",
        severity: SeverityLevel::Critical,
        cve_id: "CVE-2021-3749",
        description: "Server-Side Request Forgery in axios < 0.21.1",
    },
    KnownVulnerability {
        package: "django",
        version_pattern: r"^[12]\.[0-9]+\.[0-9]+$",
        severity: SeverityLevel::Critical,
        cve_id: "CVE-2021-35042",
        description: "SQL injection in Django < 3.2.5",
    },
    KnownVulnerability {
        package: "flask",
        version_pattern: r"^0\.[0-9]+\.[0-9]+$",
        severity: SeverityLevel::Medium,
        cve_id: "CVE-2019-1010083",
        description: "Path traversal in Flask < 1.0",
    },
    KnownVulnerability {
        package: "requests",
        version_pattern: r"^2\.[0-5]\.[0-9]+$",
        severity: SeverityLevel::High,
        cve_id: "CVE-2018-18074",
        description: "Header injection in requests < 2.20.0",
    },
];

/// Dependency files by package manager.
pub const DEPENDENCY_FILES: &[(&str, &[&str])] = &[
    ("npm", &["package.json", "package-lock.json"]),
    ("pip", &["requirements.txt", "Pipfile", "pyproject.toml"]),
    ("go", &["go.mod", "go.sum"]),
    ("cargo", &["Cargo.toml", "Cargo.lock"]),
    ("bundler", &["Gemfile", "Gemfile.lock"]),
];

/// Dependency checker.
pub struct DependencyChecker<'a> {
    scanner: &'a SecurityScanner,
    vuln_patterns: Vec<(Regex, &'static KnownVulnerability)>,
}

impl<'a> DependencyChecker<'a> {
    /// Create a new dependency checker.
    pub fn new(scanner: &'a SecurityScanner) -> Self {
        let vuln_patterns: Vec<_> = KNOWN_VULNERABILITIES
            .iter()
            .filter_map(|v| Regex::new(v.version_pattern).ok().map(|r| (r, v)))
            .collect();

        Self {
            scanner,
            vuln_patterns,
        }
    }

    /// Find all dependency files.
    pub fn find_dependency_files(&self) -> HashMap<String, Vec<PathBuf>> {
        let mut result: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for (manager, patterns) in DEPENDENCY_FILES {
            let mut files = Vec::new();
            for pattern in *patterns {
                for file in self.scanner.iter_files() {
                    if let Some(name) = file.file_name() {
                        if name.to_string_lossy() == *pattern {
                            files.push(file);
                        }
                    }
                }
            }
            if !files.is_empty() {
                result.insert((*manager).to_string(), files);
            }
        }

        result
    }

    /// Parse package.json.
    fn parse_package_json(&self, path: &Path) -> Vec<Dependency> {
        let content = match self.scanner.read_file(path) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut deps = Vec::new();

        // Simple JSON parsing for dependencies
        let dep_regex = Regex::new(r#""([^"]+)":\s*"([^"]+)""#).ok();

        if let Some(re) = dep_regex {
            let in_deps = content.contains("dependencies") || content.contains("devDependencies");
            if in_deps {
                for cap in re.captures_iter(&content) {
                    let name = cap.get(1).map(|m| m.as_str().to_string());
                    let version = cap.get(2).map(|m| {
                        m.as_str()
                            .trim_start_matches('^')
                            .trim_start_matches('~')
                            .to_string()
                    });

                    if let Some(name) = name {
                        // Skip non-package keys
                        if !name.starts_with('@')
                            && !name.contains('/')
                            && name != "name"
                            && name != "version"
                        {
                            deps.push(Dependency {
                                name,
                                version,
                                source_file: path.to_path_buf(),
                                line_number: 0,
                                package_manager: "npm".to_string(),
                            });
                        }
                    }
                }
            }
        }

        deps
    }

    /// Parse requirements.txt.
    fn parse_requirements(&self, path: &Path) -> Vec<Dependency> {
        let lines = match self.scanner.read_file_lines(path) {
            Some(l) => l,
            None => return Vec::new(),
        };

        let mut deps = Vec::new();
        let re = Regex::new(r"^([a-zA-Z0-9\-_\.]+)([=><~!]+)?(.+)?$").ok();

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(re) = &re {
                if let Some(cap) = re.captures(line) {
                    let name = cap.get(1).map(|m| m.as_str().to_string());
                    let version = cap.get(3).map(|m| m.as_str().trim().to_string());

                    if let Some(name) = name {
                        deps.push(Dependency {
                            name,
                            version,
                            source_file: path.to_path_buf(),
                            line_number: line_num + 1,
                            package_manager: "pip".to_string(),
                        });
                    }
                }
            }
        }

        deps
    }

    /// Parse go.mod.
    fn parse_go_mod(&self, path: &Path) -> Vec<Dependency> {
        let lines = match self.scanner.read_file_lines(path) {
            Some(l) => l,
            None => return Vec::new(),
        };

        let mut deps = Vec::new();
        let mut in_require = false;
        let re = Regex::new(r"(?:require\s+)?([^\s]+)\s+v?([0-9]+\.[0-9]+\.[0-9]+[^\s]*)").ok();

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();

            if line.starts_with("require (") {
                in_require = true;
                continue;
            }
            if line == ")" && in_require {
                in_require = false;
                continue;
            }

            if in_require || line.starts_with("require ") {
                if let Some(re) = &re {
                    if let Some(cap) = re.captures(line) {
                        let name = cap.get(1).map(|m| m.as_str().to_string());
                        let version = cap.get(2).map(|m| m.as_str().to_string());

                        if let Some(name) = name {
                            deps.push(Dependency {
                                name,
                                version,
                                source_file: path.to_path_buf(),
                                line_number: line_num + 1,
                                package_manager: "go".to_string(),
                            });
                        }
                    }
                }
            }
        }

        deps
    }

    /// Parse all dependencies.
    pub fn parse_all(&self) -> Vec<Dependency> {
        let mut all_deps = Vec::new();
        let dep_files = self.find_dependency_files();

        for (manager, files) in dep_files {
            for file in files {
                let deps = match manager.as_str() {
                    "npm"
                        if file
                            .file_name()
                            .map(|n| n == "package.json")
                            .unwrap_or(false) =>
                    {
                        self.parse_package_json(&file)
                    }
                    "pip"
                        if file
                            .file_name()
                            .map(|n| n == "requirements.txt")
                            .unwrap_or(false) =>
                    {
                        self.parse_requirements(&file)
                    }
                    "go" if file.file_name().map(|n| n == "go.mod").unwrap_or(false) => {
                        self.parse_go_mod(&file)
                    }
                    _ => Vec::new(),
                };
                all_deps.extend(deps);
            }
        }

        all_deps
    }

    /// Check if a dependency has known vulnerabilities.
    fn check_vulnerability(&self, dep: &Dependency) -> Option<SecurityIssue> {
        let version = dep.version.as_ref()?;

        for (pattern, vuln) in &self.vuln_patterns {
            if dep.name.to_lowercase() == vuln.package && pattern.is_match(version) {
                let relative_path = dep
                    .source_file
                    .strip_prefix(self.scanner.root_path())
                    .unwrap_or(&dep.source_file);

                let mut metadata = HashMap::new();
                metadata.insert("package".to_string(), dep.name.clone());
                metadata.insert("version".to_string(), version.clone());
                metadata.insert("package_manager".to_string(), dep.package_manager.clone());
                metadata.insert("cve_id".to_string(), vuln.cve_id.to_string());

                return Some(SecurityIssue {
                    severity: vuln.severity,
                    category: "dependencies".to_string(),
                    title: format!("Vulnerable Dependency: {}", dep.name),
                    description: format!("{}\n\nInstalled version: {}", vuln.description, version),
                    file_path: relative_path.to_path_buf(),
                    line_number: if dep.line_number > 0 {
                        Some(dep.line_number)
                    } else {
                        None
                    },
                    code_snippet: Some(format!("{}=={}", dep.name, version)),
                    remediation: Some(format!(
                        "Update {} to a secure version. Check {} for details.",
                        dep.name, vuln.cve_id
                    )),
                    cwe_id: Some("CWE-1035".to_string()),
                    confidence: "high".to_string(),
                    tov_signal: None,
                    metadata,
                });
            }
        }

        None
    }

    /// Run dependency vulnerability check.
    pub fn detect(&self) -> Vec<SecurityIssue> {
        if !self.scanner.config().check_dependencies {
            return Vec::new();
        }

        let deps = self.parse_all();
        deps.iter()
            .filter_map(|d| self.check_vulnerability(d))
            .collect()
    }
}
