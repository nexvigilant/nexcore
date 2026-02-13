// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Antipattern scanner - detects threats in code content.
//!
//! ## Primitive Grounding: κ (Comparison) + σ (Sequence) + μ (Mapping)
//!
//! Scans content by comparing against antibody patterns (κ),
//! iterating through lines (σ), and mapping matches to threats (μ).

use crate::error::{ImmunityError, ImmunityResult};
use crate::types::{Antibody, AntibodyRegistry, ScanMetrics, ScanResult, ThreatMatch};
use regex::Regex;
use std::collections::HashMap;

/// Compiled antibody with precompiled regex patterns.
#[derive(Debug)]
struct CompiledAntibody {
    antibody: Antibody,
    code_patterns: Vec<Regex>,
    error_patterns: Vec<Regex>,
    exception_patterns: Vec<Regex>,
    file_globs: Vec<glob::Pattern>,
}

/// Scanner for detecting antipatterns.
///
/// ## Tier: T2-C (κ + σ + μ + π)
pub struct ImmunityScanner {
    compiled: Vec<CompiledAntibody>,
}

impl ImmunityScanner {
    /// Create a new scanner from a registry.
    ///
    /// # Errors
    ///
    /// Returns error if any pattern fails to compile.
    pub fn new(registry: &AntibodyRegistry) -> ImmunityResult<Self> {
        let mut compiled = Vec::with_capacity(registry.len());

        for ab in &registry.antibodies {
            let mut code_patterns = Vec::new();
            let mut error_patterns = Vec::new();
            let mut exception_patterns = Vec::new();
            let mut file_globs = Vec::new();

            // Compile code patterns
            for cp in &ab.detection.code_patterns {
                let regex = Regex::new(&cp.pattern).map_err(|e| ImmunityError::InvalidPattern {
                    pattern: cp.pattern.clone(),
                    reason: e.to_string(),
                })?;
                code_patterns.push(regex);
            }

            // Compile error patterns
            for ep in &ab.detection.error_patterns {
                // Error patterns are often literal strings, escape for regex
                let escaped = regex::escape(ep);
                let regex = Regex::new(&escaped).map_err(|e| ImmunityError::InvalidPattern {
                    pattern: ep.clone(),
                    reason: e.to_string(),
                })?;
                error_patterns.push(regex);
            }

            // Compile exception patterns
            for exc in &ab.detection.exceptions {
                let escaped = regex::escape(exc);
                let regex = Regex::new(&escaped).map_err(|e| ImmunityError::InvalidPattern {
                    pattern: exc.clone(),
                    reason: e.to_string(),
                })?;
                exception_patterns.push(regex);
            }

            // Compile file globs
            for fg in &ab.detection.file_contexts {
                if let Ok(pattern) = glob::Pattern::new(fg) {
                    file_globs.push(pattern);
                }
            }

            compiled.push(CompiledAntibody {
                antibody: ab.clone(),
                code_patterns,
                error_patterns,
                exception_patterns,
                file_globs,
            });
        }

        tracing::debug!("Compiled {} antibodies for scanning", compiled.len());

        Ok(Self { compiled })
    }

    /// Check if antibody applies to a given file type.
    fn applies_to_file(&self, compiled: &CompiledAntibody, file_path: Option<&str>) -> bool {
        // If no file globs defined, applies to all
        if compiled.file_globs.is_empty() {
            return true;
        }

        // If no file path provided, can't match globs
        let Some(path) = file_path else {
            return true; // Default to matching if no path
        };

        // Check if any glob matches
        compiled.file_globs.iter().any(|g| g.matches(path))
    }

    /// Check if content has an exception that prevents matching.
    fn has_exception(&self, compiled: &CompiledAntibody, content: &str) -> bool {
        compiled
            .exception_patterns
            .iter()
            .any(|exc| exc.is_match(content))
    }

    /// Scan content for threats.
    ///
    /// # Arguments
    ///
    /// * `content` - The content to scan
    /// * `file_path` - Optional file path for context-aware scanning
    ///
    /// # Returns
    ///
    /// Scan result with all detected threats.
    #[must_use]
    pub fn scan(&self, content: &str, file_path: Option<&str>) -> ScanResult {
        let mut threats = Vec::new();
        let mut antibodies_applied = Vec::new();

        for compiled in &self.compiled {
            // Check if this antibody applies to this file type
            if !self.applies_to_file(compiled, file_path) {
                continue;
            }

            // Check for exceptions
            if self.has_exception(compiled, content) {
                continue;
            }

            // Check code patterns
            for (line_num, line) in content.lines().enumerate() {
                for pattern in &compiled.code_patterns {
                    if pattern.is_match(line) {
                        threats.push(ThreatMatch {
                            antibody_id: compiled.antibody.id.clone(),
                            antibody_name: compiled.antibody.name.clone(),
                            threat_type: compiled.antibody.threat_type,
                            severity: compiled.antibody.severity,
                            location: Some(line_num + 1),
                            matched_content: line.trim().to_string(),
                            confidence: compiled.antibody.confidence,
                            response: compiled.antibody.response.strategy,
                        });
                        if !antibodies_applied.contains(&compiled.antibody.id) {
                            antibodies_applied.push(compiled.antibody.id.clone());
                        }
                    }
                }
            }

            // Check error patterns (full content)
            for pattern in &compiled.error_patterns {
                if pattern.is_match(content) {
                    threats.push(ThreatMatch {
                        antibody_id: compiled.antibody.id.clone(),
                        antibody_name: compiled.antibody.name.clone(),
                        threat_type: compiled.antibody.threat_type,
                        severity: compiled.antibody.severity,
                        location: None,
                        matched_content: pattern.to_string(),
                        confidence: compiled.antibody.confidence,
                        response: compiled.antibody.response.strategy,
                    });
                    if !antibodies_applied.contains(&compiled.antibody.id) {
                        antibodies_applied.push(compiled.antibody.id.clone());
                    }
                }
            }
        }

        let threats_detected = threats.len() as u32;

        ScanResult {
            clean: threats.is_empty(),
            threats,
            antibodies_applied,
            metrics: ScanMetrics {
                total_scanned: 1,
                threats_detected,
                auto_fixed: 0,
                false_positives: 0,
            },
        }
    }

    /// Scan compiler/tool output for error patterns.
    ///
    /// This is specifically for learning from build failures.
    #[must_use]
    pub fn scan_errors(&self, stderr: &str) -> ScanResult {
        let mut threats = Vec::new();
        let mut antibodies_applied = Vec::new();

        for compiled in &self.compiled {
            for pattern in &compiled.error_patterns {
                if pattern.is_match(stderr) {
                    threats.push(ThreatMatch {
                        antibody_id: compiled.antibody.id.clone(),
                        antibody_name: compiled.antibody.name.clone(),
                        threat_type: compiled.antibody.threat_type,
                        severity: compiled.antibody.severity,
                        location: None,
                        matched_content: format!("Error pattern matched: {}", pattern),
                        confidence: compiled.antibody.confidence,
                        response: compiled.antibody.response.strategy,
                    });
                    if !antibodies_applied.contains(&compiled.antibody.id) {
                        antibodies_applied.push(compiled.antibody.id.clone());
                    }
                }
            }
        }

        let threats_detected = threats.len() as u32;

        ScanResult {
            clean: threats.is_empty(),
            threats,
            antibodies_applied,
            metrics: ScanMetrics {
                total_scanned: 1,
                threats_detected,
                auto_fixed: 0,
                false_positives: 0,
            },
        }
    }

    /// Get statistics about loaded antibodies.
    #[must_use]
    pub fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total".to_string(), self.compiled.len());

        let pamp_count = self
            .compiled
            .iter()
            .filter(|c| c.antibody.threat_type == crate::types::ThreatType::Pamp)
            .count();
        let damp_count = self.compiled.len() - pamp_count;

        stats.insert("pamp".to_string(), pamp_count);
        stats.insert("damp".to_string(), damp_count);

        let critical = self
            .compiled
            .iter()
            .filter(|c| c.antibody.severity == crate::types::ThreatLevel::Critical)
            .count();
        let high = self
            .compiled
            .iter()
            .filter(|c| c.antibody.severity == crate::types::ThreatLevel::High)
            .count();

        stats.insert("critical".to_string(), critical);
        stats.insert("high".to_string(), high);

        stats
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::load_from_str;

    const TEST_REGISTRY: &str = r##"
version: "1.0"
antibodies:
  - id: UNWRAP-001
    name: unwrap-detector
    threat_type: DAMP
    severity: high
    description: Detects .unwrap() calls
    detection:
      code_patterns:
        - pattern: '\.unwrap\(\)'
          language: rust
      file_contexts:
        - "*.rs"
      exceptions:
        - "#[cfg(test)]"
    response:
      strategy: warn
    confidence: 0.95

  - id: DUPLICATE-001
    name: duplicate-key
    threat_type: PAMP
    severity: high
    description: Detects duplicate keys
    detection:
      error_patterns:
        - "duplicate key"
      file_contexts:
        - "*.toml"
    response:
      strategy: block
    confidence: 0.90
"##;

    fn make_scanner() -> ImmunityScanner {
        let registry = load_from_str(TEST_REGISTRY).ok().unwrap_or_default();
        ImmunityScanner::new(&registry).ok().unwrap_or_else(|| {
            ImmunityScanner::new(&crate::types::AntibodyRegistry::new())
                .ok()
                .unwrap_or_else(|| panic!("Failed to create scanner"))
        })
    }

    #[test]
    fn test_scan_clean_code() {
        let scanner = make_scanner();
        let code = r#"
fn main() {
    let x = some_option.ok_or(Error::NotFound)?;
    println!("{}", x);
}
"#;
        let result = scanner.scan(code, Some("main.rs"));
        assert!(result.clean);
        assert!(result.threats.is_empty());
    }

    #[test]
    fn test_scan_unwrap() {
        let scanner = make_scanner();
        let code = r#"
fn main() {
    let x = some_option.unwrap();
    println!("{}", x);
}
"#;
        let result = scanner.scan(code, Some("main.rs"));
        assert!(!result.clean);
        assert_eq!(result.threats.len(), 1);
        assert_eq!(result.threats[0].antibody_id, "UNWRAP-001");
    }

    #[test]
    fn test_scan_with_exception() {
        let scanner = make_scanner();
        let code = r##"
#[cfg(test)]
mod tests {
    fn test_something() {
        let x = some_option.unwrap();
    }
}
"##;
        let result = scanner.scan(code, Some("lib.rs"));
        // Should not match because of #[cfg(test)] exception
        assert!(result.clean);
    }

    #[test]
    fn test_scan_error_pattern() {
        let scanner = make_scanner();
        let stderr = "error: duplicate key `name` in table";
        let result = scanner.scan_errors(stderr);
        assert!(!result.clean);
        assert_eq!(result.threats.len(), 1);
        assert_eq!(result.threats[0].antibody_id, "DUPLICATE-001");
    }

    #[test]
    fn test_scan_file_context() {
        let scanner = make_scanner();
        let code = "let x = foo.unwrap();";

        // .rs file should match (antibody has file_contexts: ["*.rs"])
        let result = scanner.scan(code, Some("test.rs"));
        assert!(!result.clean);

        // .py file should NOT match for Rust patterns
        // File globs filter which antibodies apply
        let result = scanner.scan(code, Some("test.py"));
        assert!(result.clean); // Clean because antibody doesn't apply to .py files
    }

    #[test]
    fn test_stats() {
        let scanner = make_scanner();
        let stats = scanner.stats();

        assert_eq!(stats.get("total").copied().unwrap_or(0), 2);
        assert_eq!(stats.get("damp").copied().unwrap_or(0), 1);
        assert_eq!(stats.get("pamp").copied().unwrap_or(0), 1);
    }

    #[test]
    fn test_multiple_matches() {
        let scanner = make_scanner();
        let code = r#"
fn main() {
    let a = x.unwrap();
    let b = y.unwrap();
    let c = z.unwrap();
}
"#;
        let result = scanner.scan(code, Some("main.rs"));
        assert!(!result.clean);
        assert_eq!(result.threats.len(), 3);
    }
}
