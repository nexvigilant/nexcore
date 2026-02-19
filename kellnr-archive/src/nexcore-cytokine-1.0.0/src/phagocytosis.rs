//! Phagocytosis - threat engulfment and digestion.
//!
//! ## T1 Grounding
//!
//! - `ThreatDigest` → ρ (recursion) - recursive pattern consumption
//! - `Phagocyte` → ∅ (void) - threats are neutralized/destroyed
//! - `EngulfmentResult` → ∃ (existence) - threat absorbed or not
//!
//! ## Biological Analog
//!
//! Phagocytes (macrophages, neutrophils) engulf and destroy threats:
//! 1. **Detection**: Pattern recognition receptors (PRRs) bind pathogen
//! 2. **Engulfment**: Pseudopod extension wraps around threat
//! 3. **Digestion**: Phagosome fuses with lysosome, enzymes destroy threat
//! 4. **Presentation**: Processed antigens displayed for adaptive immune response
//!
//! ## Claude Code Analog
//!
//! Antipattern detection, absorption, and neutralization:
//! - **Detection**: Pattern match against known threat signatures
//! - **Engulfment**: Capture full context of the threat (file, line, content)
//! - **Digestion**: Analyze and classify the threat
//! - **Presentation**: Report findings to the broader system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of attempting to engulf a threat.
///
/// # Tier: T2-P
/// Grounds to: ∃ (existence — was the threat consumed?)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngulfmentResult {
    /// Threat successfully engulfed
    Engulfed,
    /// Threat too large to engulf (exceeds capacity)
    TooLarge,
    /// Threat not recognized by pattern receptors
    Unrecognized,
    /// Phagocyte already at maximum load
    Saturated,
}

/// Classification of the digested threat.
///
/// # Tier: T2-P
/// Grounds to: Σ (sum type — categorical classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatClass {
    /// Code quality issue (style, complexity)
    Quality,
    /// Safety violation (unwrap, unsafe, panic)
    Safety,
    /// Security vulnerability (injection, exposure)
    Security,
    /// Performance issue (N+1, unbounded allocation)
    Performance,
    /// Unknown/unclassified
    Unknown,
}

/// Record of a digested threat.
///
/// # Tier: T2-C
/// Grounds to: ρ (recursion — threat consumed) + ∅ (void — threat destroyed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatDigest {
    /// What pattern was matched
    pub pattern: String,
    /// Where the threat was found
    pub location: String,
    /// Threat classification
    pub class: ThreatClass,
    /// Severity score \[0, 100\]
    pub severity: u8,
    /// When the threat was engulfed
    pub engulfed_at: DateTime<Utc>,
    /// When digestion completed
    pub digested_at: Option<DateTime<Utc>>,
    /// Whether the threat was fully neutralized
    pub neutralized: bool,
}

/// A phagocyte that can detect, engulf, and digest threats.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ρ (recursion — iterative consumption) + ∅ (void — destruction)
#[derive(Debug)]
pub struct Phagocyte {
    /// Phagocyte identifier
    id: String,
    /// Patterns this phagocyte recognizes
    patterns: Vec<PatternReceptor>,
    /// Currently digesting threats
    digesting: Vec<ThreatDigest>,
    /// Fully processed threats (antigen library)
    antigen_library: Vec<ThreatDigest>,
    /// Maximum concurrent threats being digested
    max_load: usize,
    /// Lifetime stats
    total_engulfed: u64,
    total_neutralized: u64,
}

/// A pattern recognition receptor (PRR).
///
/// # Tier: T2-P
/// Grounds to: μ (mapping — pattern to threat class)
#[derive(Debug, Clone)]
pub struct PatternReceptor {
    /// Pattern to match (substring or regex-like)
    pub pattern: String,
    /// What class of threat this pattern indicates
    pub threat_class: ThreatClass,
    /// Severity assigned to matches
    pub severity: u8,
}

impl Phagocyte {
    /// Create a new phagocyte with default capacity.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            patterns: Vec::new(),
            digesting: Vec::new(),
            antigen_library: Vec::new(),
            max_load: 10,
            total_engulfed: 0,
            total_neutralized: 0,
        }
    }

    /// Set maximum concurrent digestion load.
    pub fn with_max_load(mut self, max_load: usize) -> Self {
        self.max_load = max_load;
        self
    }

    /// Register a pattern recognition receptor.
    pub fn add_receptor(&mut self, receptor: PatternReceptor) {
        self.patterns.push(receptor);
    }

    /// Register a pattern receptor (builder pattern).
    pub fn with_receptor(mut self, receptor: PatternReceptor) -> Self {
        self.patterns.push(receptor);
        self
    }

    /// Detect threats in content by matching against registered patterns.
    ///
    /// Returns list of matches with their classifications.
    pub fn detect(&self, content: &str, location: &str) -> Vec<(String, ThreatClass, u8)> {
        self.patterns
            .iter()
            .filter(|p| content.contains(&p.pattern))
            .map(|p| (p.pattern.clone(), p.threat_class, p.severity))
            .collect()
    }

    /// Attempt to engulf a detected threat.
    pub fn engulf(
        &mut self,
        pattern: &str,
        location: &str,
        class: ThreatClass,
        severity: u8,
    ) -> EngulfmentResult {
        // Check capacity
        if self.digesting.len() >= self.max_load {
            return EngulfmentResult::Saturated;
        }

        // Check pattern recognition
        let recognized = self.patterns.iter().any(|p| p.pattern == pattern);

        if !recognized && !self.patterns.is_empty() {
            return EngulfmentResult::Unrecognized;
        }

        let digest = ThreatDigest {
            pattern: pattern.to_string(),
            location: location.to_string(),
            class,
            severity,
            engulfed_at: Utc::now(),
            digested_at: None,
            neutralized: false,
        };

        self.digesting.push(digest);
        self.total_engulfed += 1;
        EngulfmentResult::Engulfed
    }

    /// Scan content, detect threats, and engulf all found.
    ///
    /// Returns number of threats engulfed.
    pub fn scan_and_engulf(&mut self, content: &str, location: &str) -> usize {
        let threats = self.detect(content, location);
        let mut count = 0;
        for (pattern, class, severity) in threats {
            if self.engulf(&pattern, location, class, severity) == EngulfmentResult::Engulfed {
                count += 1;
            }
        }
        count
    }

    /// Digest all currently engulfed threats.
    ///
    /// Marks them as neutralized and moves to antigen library.
    /// Returns number of threats digested.
    pub fn digest(&mut self) -> usize {
        let now = Utc::now();
        let mut digested = 0;

        for threat in &mut self.digesting {
            if !threat.neutralized {
                threat.digested_at = Some(now);
                threat.neutralized = true;
                digested += 1;
            }
        }

        // Move neutralized to antigen library
        let (neutralized, still_active): (Vec<_>, Vec<_>) =
            self.digesting.drain(..).partition(|t| t.neutralized);

        self.antigen_library.extend(neutralized);
        self.digesting = still_active;
        self.total_neutralized += digested as u64;

        digested
    }

    /// Present antigens (return digested threat records for learning).
    pub fn present_antigens(&self) -> &[ThreatDigest] {
        &self.antigen_library
    }

    /// Get currently digesting threats.
    pub fn active_threats(&self) -> &[ThreatDigest] {
        &self.digesting
    }

    /// Get phagocyte load (fraction of capacity in use).
    pub fn load(&self) -> f64 {
        if self.max_load == 0 {
            return 1.0;
        }
        self.digesting.len() as f64 / self.max_load as f64
    }

    /// Get lifetime statistics.
    pub fn stats(&self) -> PhagocyteStats {
        let mut by_class = std::collections::HashMap::new();
        for threat in &self.antigen_library {
            *by_class
                .entry(format!("{:?}", threat.class))
                .or_insert(0u64) += 1;
        }

        PhagocyteStats {
            id: self.id.clone(),
            total_engulfed: self.total_engulfed,
            total_neutralized: self.total_neutralized,
            currently_digesting: self.digesting.len(),
            antigen_library_size: self.antigen_library.len(),
            load: self.load(),
            by_class,
        }
    }
}

/// Statistics for a phagocyte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhagocyteStats {
    /// Phagocyte identifier
    pub id: String,
    /// Lifetime threats engulfed
    pub total_engulfed: u64,
    /// Lifetime threats neutralized
    pub total_neutralized: u64,
    /// Currently digesting count
    pub currently_digesting: usize,
    /// Size of antigen library
    pub antigen_library_size: usize,
    /// Current load fraction
    pub load: f64,
    /// Threats by classification
    pub by_class: std::collections::HashMap<String, u64>,
}

/// Pre-built pattern receptors for common code threats.
pub mod receptors {
    use super::*;

    /// Safety receptors: detect unwrap, expect, panic, unsafe.
    pub fn safety() -> Vec<PatternReceptor> {
        vec![
            PatternReceptor {
                pattern: ".unwrap()".to_string(),
                threat_class: ThreatClass::Safety,
                severity: 60,
            },
            PatternReceptor {
                pattern: ".expect(".to_string(),
                threat_class: ThreatClass::Safety,
                severity: 50,
            },
            PatternReceptor {
                pattern: "panic!".to_string(),
                threat_class: ThreatClass::Safety,
                severity: 80,
            },
            PatternReceptor {
                pattern: "unsafe {".to_string(),
                threat_class: ThreatClass::Safety,
                severity: 90,
            },
        ]
    }

    /// Security receptors: detect common vulnerability patterns.
    pub fn security() -> Vec<PatternReceptor> {
        vec![
            PatternReceptor {
                pattern: "sql!".to_string(),
                threat_class: ThreatClass::Security,
                severity: 95,
            },
            PatternReceptor {
                pattern: "format!(\"SELECT".to_string(),
                threat_class: ThreatClass::Security,
                severity: 90,
            },
            PatternReceptor {
                pattern: "eval(".to_string(),
                threat_class: ThreatClass::Security,
                severity: 85,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phagocyte_creation() {
        let phag = Phagocyte::new("macrophage_1");
        assert_eq!(phag.load(), 0.0);
        assert!(phag.active_threats().is_empty());
        assert!(phag.present_antigens().is_empty());
    }

    #[test]
    fn test_detect_threats() {
        let phag = Phagocyte::new("test").with_receptor(PatternReceptor {
            pattern: ".unwrap()".to_string(),
            threat_class: ThreatClass::Safety,
            severity: 60,
        });

        let threats = phag.detect("let x = foo.unwrap();", "lib.rs:42");
        assert_eq!(threats.len(), 1);
        assert_eq!(threats[0].1, ThreatClass::Safety);
    }

    #[test]
    fn test_engulf_threat() {
        let mut phag = Phagocyte::new("test").with_receptor(PatternReceptor {
            pattern: "panic!".to_string(),
            threat_class: ThreatClass::Safety,
            severity: 80,
        });

        let result = phag.engulf("panic!", "main.rs:10", ThreatClass::Safety, 80);
        assert_eq!(result, EngulfmentResult::Engulfed);
        assert_eq!(phag.active_threats().len(), 1);
    }

    #[test]
    fn test_engulf_unrecognized() {
        let mut phag = Phagocyte::new("test").with_receptor(PatternReceptor {
            pattern: "panic!".to_string(),
            threat_class: ThreatClass::Safety,
            severity: 80,
        });

        let result = phag.engulf("unknown_pattern", "lib.rs:1", ThreatClass::Unknown, 10);
        assert_eq!(result, EngulfmentResult::Unrecognized);
    }

    #[test]
    fn test_engulf_at_capacity() {
        let mut phag = Phagocyte::new("test").with_max_load(1);

        // No receptors = accept anything
        let result = phag.engulf("a", "x.rs:1", ThreatClass::Quality, 10);
        assert_eq!(result, EngulfmentResult::Engulfed);

        let result = phag.engulf("b", "x.rs:2", ThreatClass::Quality, 10);
        assert_eq!(result, EngulfmentResult::Saturated);
    }

    #[test]
    fn test_scan_and_engulf() {
        let mut phag = Phagocyte::new("scanner");
        for receptor in receptors::safety() {
            phag.add_receptor(receptor);
        }

        let code = r#"
            let val = map.get("key").unwrap();
            panic!("fatal error");
        "#;

        let count = phag.scan_and_engulf(code, "bad_code.rs");
        assert_eq!(count, 2); // unwrap + panic
    }

    #[test]
    fn test_digest_and_present() {
        let mut phag = Phagocyte::new("test");
        phag.engulf("threat_a", "x.rs:1", ThreatClass::Safety, 50);
        phag.engulf("threat_b", "x.rs:2", ThreatClass::Security, 90);

        let digested = phag.digest();
        assert_eq!(digested, 2);
        assert!(phag.active_threats().is_empty());

        let antigens = phag.present_antigens();
        assert_eq!(antigens.len(), 2);
        assert!(antigens.iter().all(|a| a.neutralized));
    }

    #[test]
    fn test_phagocyte_stats() {
        let mut phag = Phagocyte::new("stats_test");
        phag.engulf("t1", "a.rs:1", ThreatClass::Safety, 50);
        phag.engulf("t2", "b.rs:1", ThreatClass::Security, 80);
        phag.digest();

        let stats = phag.stats();
        assert_eq!(stats.total_engulfed, 2);
        assert_eq!(stats.total_neutralized, 2);
        assert_eq!(stats.antigen_library_size, 2);
    }

    #[test]
    fn test_safety_receptors() {
        let receptors = receptors::safety();
        assert_eq!(receptors.len(), 4);
        assert!(
            receptors
                .iter()
                .all(|r| r.threat_class == ThreatClass::Safety)
        );
    }

    #[test]
    fn test_security_receptors() {
        let receptors = receptors::security();
        assert_eq!(receptors.len(), 3);
        assert!(
            receptors
                .iter()
                .all(|r| r.threat_class == ThreatClass::Security)
        );
    }
}
