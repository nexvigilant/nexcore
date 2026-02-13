//! Perception layer for task analysis.

use super::models::{Complexity, TaskAnalysis};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// Perception layer for analyzing user requests.
pub struct PerceptionLayer {
    additive_words: HashMap<&'static str, i32>,
    simple_words: HashSet<&'static str>,
    domain_keywords: HashMap<&'static str, Vec<&'static str>>,
    intent_patterns: HashMap<&'static str, &'static str>,
    /// Domain priority for tiebreaking (higher index = higher priority)
    domain_priority: Vec<&'static str>,
}

impl Default for PerceptionLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionLayer {
    /// Create a new perception layer with default patterns.
    #[must_use]
    pub fn new() -> Self {
        let mut additive_words = HashMap::new();
        additive_words.insert("and", 1);
        additive_words.insert("then", 1);
        additive_words.insert("also", 1);
        additive_words.insert("if", 2);
        additive_words.insert("depending", 2);
        additive_words.insert("multiple", 2);
        additive_words.insert("all", 2);
        additive_words.insert("comprehensive", 3);
        additive_words.insert("full", 3);
        additive_words.insert("complete", 3);
        additive_words.insert("entire", 3);

        let mut simple_words = HashSet::new();
        for word in ["simple", "quick", "just", "only", "single", "one", "basic"] {
            simple_words.insert(word);
        }

        let mut domain_keywords = HashMap::new();
        // Use word boundary patterns to avoid substring false positives
        // e.g., "ui" should not match in "build"
        domain_keywords.insert(
            "pharmacovigilance",
            vec![
                "signal",
                "adverse",
                "faers",
                "drug safety",
                "icsr",
                "medwatch",
                "e2b",
                "pharmacovigilance",
                "disproportionality",
            ],
        );
        domain_keywords.insert(
            "web",
            vec![
                "frontend",
                "backend",
                "api",
                "component",
                "react",
                "nextjs",
                "typescript",
                "javascript",
                "css",
                "html",
            ],
        );
        domain_keywords.insert(
            "data",
            vec![
                "pandas",
                "analysis",
                "statistics",
                "machine learning",
                "model",
                "dataset",
                "csv",
                "json",
                "database",
                "polars",
            ],
        );

        let mut intent_patterns = HashMap::new();
        intent_patterns.insert(
            "build",
            r"(build|create|implement|add|make|develop|write)\b",
        );
        intent_patterns.insert("fix", r"(fix|debug|repair|resolve|solve)\b");
        intent_patterns.insert(
            "refactor",
            r"(refactor|clean|reorganize|restructure|improve)\b",
        );
        intent_patterns.insert(
            "analyze",
            r"(analyze|investigate|explore|understand|examine)\b",
        );
        intent_patterns.insert("deploy", r"(deploy|ship|release|publish|push)\b");
        intent_patterns.insert("test", r"(test|verify|validate|check)\b");
        intent_patterns.insert("document", r"(document|describe|explain|write docs)\b");

        // Domain priority for deterministic tiebreaking
        let domain_priority = vec!["general", "data", "web", "pharmacovigilance"];

        Self {
            additive_words,
            simple_words,
            domain_keywords,
            intent_patterns,
            domain_priority,
        }
    }

    /// Analyze a user request.
    #[must_use]
    pub fn analyze(&self, request: &str) -> TaskAnalysis {
        let request_lower = request.to_lowercase();
        TaskAnalysis {
            intent: self.extract_intent(&request_lower),
            domain: self.detect_domain(&request_lower),
            complexity: self.estimate_complexity(&request_lower),
            keywords_extracted: self.extract_keywords(&request_lower),
        }
    }

    fn extract_intent(&self, request: &str) -> String {
        for (intent, pattern) in &self.intent_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(request) {
                    return (*intent).to_string();
                }
            }
        }
        "execute".to_string()
    }

    fn detect_domain(&self, request: &str) -> String {
        let words: HashSet<&str> = request
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .collect();

        let mut best_domain = "general".to_string();
        let mut max_score = 0usize;
        let mut best_priority = 0usize;

        for (domain, keywords) in &self.domain_keywords {
            // Count whole-word matches only
            let score = keywords
                .iter()
                .filter(|&&kw| {
                    if kw.contains(' ') {
                        // Multi-word keyword: use contains
                        request.contains(kw)
                    } else {
                        // Single word: must be whole word
                        words.contains(kw)
                    }
                })
                .count();

            let priority = self
                .domain_priority
                .iter()
                .position(|&d| d == *domain)
                .unwrap_or(0);

            // Update if score is higher, or same score but higher priority
            if score > max_score || (score == max_score && score > 0 && priority > best_priority) {
                max_score = score;
                best_domain = (*domain).to_string();
                best_priority = priority;
            }
        }

        best_domain
    }

    fn estimate_complexity(&self, request: &str) -> Complexity {
        let mut score = 0;
        for word in request.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if let Some(&weight) = self.additive_words.get(clean) {
                score += weight;
            }
        }
        for simple in &self.simple_words {
            if request.contains(simple) {
                score -= 2;
            }
        }
        if score >= 5 {
            Complexity::Complex
        } else if score >= 2 {
            Complexity::Moderate
        } else {
            Complexity::Simple
        }
    }

    fn extract_keywords(&self, request: &str) -> Vec<String> {
        request
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| s.len() > 3)
            .collect()
    }
}
