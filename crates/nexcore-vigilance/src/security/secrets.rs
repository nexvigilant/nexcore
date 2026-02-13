//! Secret detection for security scanning.
//!
//! Detects hardcoded secrets, API keys, tokens, and credentials using:
//! - Pattern matching (30+ secret patterns)
//! - Entropy analysis for high-entropy strings

use crate::security::config::SeverityLevel;
use crate::security::scanner::{SecurityIssue, SecurityScanner};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// A secret pattern definition.
#[derive(Debug, Clone)]
pub struct SecretPattern {
    pub name: &'static str,
    pub pattern: &'static str,
    pub severity: SeverityLevel,
}

/// Built-in secret patterns.
pub const SECRET_PATTERNS: &[SecretPattern] = &[
    // AWS
    SecretPattern {
        name: "AWS Access Key",
        pattern: r"AKIA[0-9A-Z]{16}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "AWS Secret Key",
        pattern: r#"aws(.{0,20})?["'][0-9a-zA-Z/+]{40}["']"#,
        severity: SeverityLevel::Critical,
    },
    // GCP
    SecretPattern {
        name: "GCP API Key",
        pattern: r"AIza[0-9A-Za-z\-_]{35}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "Google OAuth",
        pattern: r"[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com",
        severity: SeverityLevel::High,
    },
    SecretPattern {
        name: "Google Service Account",
        pattern: r#""type":\s*"service_account""#,
        severity: SeverityLevel::Critical,
    },
    // GitHub
    SecretPattern {
        name: "GitHub PAT",
        pattern: r"ghp_[0-9a-zA-Z]{36}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "GitHub OAuth",
        pattern: r"gho_[0-9a-zA-Z]{36}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "GitHub App Token",
        pattern: r"(ghu|ghs)_[0-9a-zA-Z]{36}",
        severity: SeverityLevel::Critical,
    },
    // GitLab
    SecretPattern {
        name: "GitLab PAT",
        pattern: r"glpat-[0-9a-zA-Z\-]{20}",
        severity: SeverityLevel::Critical,
    },
    // Slack
    SecretPattern {
        name: "Slack Token",
        pattern: r"xox[baprs]-[0-9a-zA-Z]{10,48}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "Slack Webhook",
        pattern: r"https://hooks\.slack\.com/services/T[a-zA-Z0-9_]{8}/B[a-zA-Z0-9_]{8}/[a-zA-Z0-9_]{24}",
        severity: SeverityLevel::High,
    },
    // AI APIs
    SecretPattern {
        name: "OpenAI API Key",
        pattern: r"sk-[0-9a-zA-Z]{48}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "Anthropic API Key",
        pattern: r"sk-ant-[0-9a-zA-Z\-]{95}",
        severity: SeverityLevel::Critical,
    },
    // Payment
    SecretPattern {
        name: "Stripe API Key",
        pattern: r"sk_live_[0-9a-zA-Z]{24}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "Square OAuth",
        pattern: r"sq0csp-[0-9A-Za-z\-_]{43}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "Square Access Token",
        pattern: r"sqOatp-[0-9A-Za-z\-_]{22}",
        severity: SeverityLevel::Critical,
    },
    // Communication
    SecretPattern {
        name: "Twilio API Key",
        pattern: r"SK[0-9a-fA-F]{32}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "SendGrid API Key",
        pattern: r"SG\.[0-9A-Za-z\-_]{22}\.[0-9A-Za-z\-_]{43}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "Mailgun API Key",
        pattern: r"key-[0-9a-zA-Z]{32}",
        severity: SeverityLevel::High,
    },
    SecretPattern {
        name: "Mailchimp API Key",
        pattern: r"[0-9a-f]{32}-us[0-9]{1,2}",
        severity: SeverityLevel::High,
    },
    // Social
    SecretPattern {
        name: "Facebook Access Token",
        pattern: r"EAACEdEose0cBA[0-9A-Za-z]+",
        severity: SeverityLevel::High,
    },
    // Package Managers
    SecretPattern {
        name: "NPM Token",
        pattern: r"npm_[0-9a-zA-Z]{36}",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "PyPI Token",
        pattern: r"pypi-AgEIcHlwaS5vcmc[0-9A-Za-z\-_]{50,}",
        severity: SeverityLevel::Critical,
    },
    // Cloud Providers
    SecretPattern {
        name: "Heroku API Key",
        pattern: r"[0-9A-F]{8}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{12}",
        severity: SeverityLevel::High,
    },
    SecretPattern {
        name: "DigitalOcean Token",
        pattern: r"dop_v1_[a-f0-9]{64}",
        severity: SeverityLevel::Critical,
    },
    // Generic patterns
    SecretPattern {
        name: "Generic API Key",
        pattern: r#"[aA][pP][iI][_]?[kK][eE][yY].*["'][0-9a-zA-Z]{32,45}["']"#,
        severity: SeverityLevel::High,
    },
    SecretPattern {
        name: "Generic Secret",
        pattern: r#"[sS][eE][cC][rR][eE][tT].*["'][0-9a-zA-Z]{32,45}["']"#,
        severity: SeverityLevel::High,
    },
    SecretPattern {
        name: "Generic Password",
        pattern: r#"[pP][aA][sS][sS][wW][oO][rR][dD].*["'][^"']{8,}["']"#,
        severity: SeverityLevel::High,
    },
    SecretPattern {
        name: "Generic Token",
        pattern: r#"[tT][oO][kK][eE][nN].*["'][0-9a-zA-Z]{32,45}["']"#,
        severity: SeverityLevel::High,
    },
    // Keys and Certificates
    SecretPattern {
        name: "Private Key",
        pattern: r"-----BEGIN (RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "SSH Private Key",
        pattern: r"-----BEGIN OPENSSH PRIVATE KEY-----",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "PGP Private Key",
        pattern: r"-----BEGIN PGP PRIVATE KEY BLOCK-----",
        severity: SeverityLevel::Critical,
    },
    // Database
    SecretPattern {
        name: "Database Connection",
        pattern: r"(postgresql|mysql|mongodb|redis)://[^\s]+:[^\s]+@",
        severity: SeverityLevel::Critical,
    },
    SecretPattern {
        name: "JDBC Connection",
        pattern: r"jdbc:[a-z]+://[^\s]+",
        severity: SeverityLevel::High,
    },
];

/// False positive indicators to reduce noise.
const FALSE_POSITIVE_WORDS: &[&str] = &[
    "example",
    "sample",
    "test",
    "dummy",
    "fake",
    "placeholder",
    "xxx",
    "000",
    "yyy",
    "zzz",
    "your-",
    "your_",
    "replace-",
    "insert-",
];

/// Environment variable patterns (usually safe).
const ENV_VAR_PATTERNS: &[&str] = &[
    r"export\s+\w+=",
    r"process\.env\.",
    r"os\.environ",
    r"os\.getenv",
    r"\$\{[A-Z_]+\}",
];

/// Secrets detector.
pub struct SecretsDetector<'a> {
    scanner: &'a SecurityScanner,
    compiled_patterns: Vec<(Regex, &'static SecretPattern)>,
    env_patterns: Vec<Regex>,
    quoted_string_re: Option<Regex>,
    assignment_re: Option<Regex>,
}

impl<'a> SecretsDetector<'a> {
    /// Create a new secrets detector.
    pub fn new(scanner: &'a SecurityScanner) -> Self {
        let compiled_patterns: Vec<_> = SECRET_PATTERNS
            .iter()
            .filter_map(|p| Regex::new(p.pattern).ok().map(|r| (r, p)))
            .collect();

        let env_patterns: Vec<_> = ENV_VAR_PATTERNS
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        // Pre-compile regex for entropy detection
        let quoted_string_re = Regex::new(r#"["']([^"']{20,100})["']"#).ok();
        let assignment_re = Regex::new(r"=\s*([a-zA-Z0-9+/]{20,100})").ok();

        Self {
            scanner,
            compiled_patterns,
            env_patterns,
            quoted_string_re,
            assignment_re,
        }
    }

    /// Calculate Shannon entropy of a string.
    #[must_use]
    pub fn calculate_entropy(s: &str) -> f64 {
        if s.is_empty() {
            return 0.0;
        }

        let mut freq: HashMap<char, usize> = HashMap::new();
        for c in s.chars() {
            *freq.entry(c).or_insert(0) += 1;
        }

        let len = s.len() as f64;
        freq.values()
            .map(|&count| {
                let p = count as f64 / len;
                -p * p.log2()
            })
            .sum()
    }

    /// Check if string has high entropy (potentially a secret).
    #[must_use]
    pub fn is_high_entropy(s: &str, threshold: f64) -> bool {
        if s.len() < 20 || s.len() > 100 {
            return false;
        }

        let alphanumeric: usize = s.chars().filter(|c| c.is_alphanumeric()).count();
        if (alphanumeric as f64 / s.len() as f64) < 0.8 {
            return false;
        }

        Self::calculate_entropy(s) >= threshold
    }

    /// Check if match is likely a false positive.
    fn is_likely_false_positive(&self, line: &str, matched: &str) -> bool {
        // Check for environment variable usage
        for pattern in &self.env_patterns {
            if pattern.is_match(line) {
                return true;
            }
        }

        // Check for comments
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            return true;
        }

        // Check whitelisted secrets
        for whitelist in &self.scanner.config().whitelisted_secrets {
            if matched.contains(whitelist.as_str()) {
                return true;
            }
        }

        // Check for placeholder words in line
        let line_lower = line.to_lowercase();
        for word in FALSE_POSITIVE_WORDS {
            if line_lower.contains(word) {
                return true;
            }
        }

        false
    }

    /// Get remediation advice for a secret type.
    fn get_remediation(secret_type: &str) -> String {
        if secret_type.contains("AWS") {
            return "Remove AWS credentials. Use IAM roles or environment variables. Rotate immediately.".to_string();
        }
        if secret_type.contains("GCP") || secret_type.contains("Google") {
            return "Remove GCP credentials. Use Application Default Credentials or Secret Manager. Rotate immediately.".to_string();
        }
        if secret_type.contains("GitHub") {
            return "Remove GitHub token. Use GitHub Secrets for CI/CD. Revoke token immediately."
                .to_string();
        }
        if secret_type.contains("Private Key") {
            return "Remove private key. Store in secure key management system. Generate new keypair.".to_string();
        }
        if secret_type.contains("Database") || secret_type.contains("Connection") {
            return "Remove database credentials. Use environment variables or secret manager. Rotate credentials.".to_string();
        }
        "Remove hardcoded secret. Use environment variables or secret management. Rotate immediately.".to_string()
    }

    /// Detect secrets in a single file.
    pub fn detect_in_file(&self, file_path: &Path) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let config = self.scanner.config();

        let lines = match self.scanner.read_file_lines(file_path) {
            Some(l) => l,
            None => return issues,
        };

        let relative_path = file_path
            .strip_prefix(self.scanner.root_path())
            .unwrap_or(file_path);

        for (line_num, line) in lines.iter().enumerate() {
            let line_num = line_num + 1;

            // Pattern-based detection
            if config.secret_patterns_enabled {
                for (regex, pattern) in &self.compiled_patterns {
                    if let Some(m) = regex.find(line) {
                        let matched = m.as_str();

                        if self.is_likely_false_positive(line, matched) {
                            continue;
                        }

                        let confidence = if pattern.name.contains("Generic") {
                            "medium"
                        } else {
                            "high"
                        };

                        let mut metadata = HashMap::new();
                        metadata.insert("secret_type".to_string(), pattern.name.to_string());

                        issues.push(SecurityIssue {
                            severity: pattern.severity,
                            category: "secrets".to_string(),
                            title: format!("Potential {} Detected", pattern.name),
                            description: format!(
                                "Found pattern matching {} in code.",
                                pattern.name
                            ),
                            file_path: relative_path.to_path_buf(),
                            line_number: Some(line_num),
                            code_snippet: Some(line.chars().take(200).collect()),
                            remediation: Some(Self::get_remediation(pattern.name)),
                            cwe_id: Some("CWE-798".to_string()),
                            confidence: confidence.to_string(),
                            tov_signal: None,
                            metadata,
                        });
                    }
                }
            }

            // Entropy-based detection
            if config.entropy_check_enabled {
                // Extract potential secrets (quoted strings, assignments)
                let mut candidates = Vec::new();
                if let Some(re) = &self.quoted_string_re {
                    for cap in re.captures_iter(line) {
                        if let Some(m) = cap.get(1) {
                            candidates.push(m.as_str());
                        }
                    }
                }
                if let Some(re) = &self.assignment_re {
                    for cap in re.captures_iter(line) {
                        if let Some(m) = cap.get(1) {
                            candidates.push(m.as_str());
                        }
                    }
                }

                for candidate in candidates {
                    if Self::is_high_entropy(candidate, config.high_entropy_threshold) {
                        if self.is_likely_false_positive(line, candidate) {
                            continue;
                        }

                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "entropy".to_string(),
                            format!("{:.2}", Self::calculate_entropy(candidate)),
                        );

                        issues.push(SecurityIssue {
                            severity: SeverityLevel::High,
                            category: "secrets".to_string(),
                            title: "High Entropy String Detected".to_string(),
                            description: "Found high-entropy string that may be a secret.".to_string(),
                            file_path: relative_path.to_path_buf(),
                            line_number: Some(line_num),
                            code_snippet: Some(line.chars().take(200).collect()),
                            remediation: Some("If this is a secret, move it to environment variables or a secure secret manager.".to_string()),
                            cwe_id: Some("CWE-798".to_string()),
                            confidence: "medium".to_string(),
                            tov_signal: None,
                            metadata,
                        });
                    }
                }
            }
        }

        issues
    }

    /// Run secret detection on all files.
    pub fn detect(&self) -> Vec<SecurityIssue> {
        if !self.scanner.config().detect_secrets {
            return Vec::new();
        }

        let files = self.scanner.iter_files();
        files.iter().flat_map(|f| self.detect_in_file(f)).collect()
    }
}
