//! Compendious Machine MCP Server
//!
//! MCP server providing text density optimization tools for AI agents.
//! Implements the Compendious Score: Cs = (I / E) × C × R
//!
//! Tools provided:
//! - score_text: Calculate compendious score
//! - compress_text: Apply BLUFF method optimization
//! - compare_texts: Compare original vs optimized
//! - analyze_patterns: Identify verbose patterns
//! - get_domain_target: Get target Cs for domain/content type

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// ============================================
// Core Types
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompendiousResult {
    pub score: f64,
    pub information_bits: f64,
    pub expression_cost: usize,
    pub completeness: f64,
    pub readability: f64,
    pub limiting_factor: String,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub original: CompendiousResult,
    pub optimized: CompendiousResult,
    pub improvement_percent: f64,
    pub tokens_saved: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern: String,
    pub found: String,
    pub replacement: String,
    pub savings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original: String,
    pub compressed: String,
    pub original_score: CompendiousResult,
    pub compressed_score: CompendiousResult,
    pub patterns_applied: Vec<PatternMatch>,
    pub improvement_percent: f64,
    /// Whether compressed_score.score meets or exceeds the caller-supplied target_cs.
    pub target_achieved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTarget {
    pub domain: String,
    pub content_type: String,
    pub target_cs: f64,
    pub rationale: String,
}

// ============================================
// MCP Tool Definitions
// ============================================

pub fn get_tool_definitions() -> Value {
    json!({
        "tools": [
            {
                "name": "score_text",
                "description": "Calculate the Compendious Score (Cs) for input text. Cs = (I/E) × C × R where I=information bits, E=expression cost, C=completeness, R=readability. Higher scores indicate better information density.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The text to analyze for compendious score"
                        },
                        "required_elements": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Optional list of elements that must be present for completeness check"
                        }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "compress_text",
                "description": "Apply the BLUFF method (Bottom Line Up Front, Load definitions, Unique concepts, Force limits, Flow hierarchically) to optimize text for maximum information density while preserving completeness.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The text to compress"
                        },
                        "target_cs": {
                            "type": "number",
                            "description": "Target Compendious Score to achieve (default: 2.0)"
                        },
                        "preserve": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Terms or phrases that must be preserved exactly"
                        }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "compare_texts",
                "description": "Compare two versions of text and calculate improvement in Compendious Score. Useful for before/after analysis of compression efforts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "original": {
                            "type": "string",
                            "description": "The original (presumably verbose) text"
                        },
                        "optimized": {
                            "type": "string",
                            "description": "The optimized (presumably compressed) text"
                        }
                    },
                    "required": ["original", "optimized"]
                }
            },
            {
                "name": "analyze_patterns",
                "description": "Identify verbose patterns in text that can be compressed. Returns specific instances with suggested replacements and estimated token savings.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {
                            "type": "string",
                            "description": "The text to analyze for verbose patterns"
                        }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "get_domain_target",
                "description": "Get the recommended Compendious Score target for a specific domain and content type. Different contexts require different density levels.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "domain": {
                            "type": "string",
                            "description": "Domain (e.g., 'technical', 'business', 'academic', 'legal', 'medical', 'journalism')"
                        },
                        "content_type": {
                            "type": "string",
                            "description": "Content type (e.g., 'api_reference', 'executive_summary', 'abstract', 'email', 'tutorial')"
                        }
                    },
                    "required": ["domain", "content_type"]
                }
            }
        ]
    })
}

// ============================================
// Core Scoring Engine
// ============================================

pub struct CompendiousMachine {
    verbose_patterns: BTreeMap<&'static str, &'static str>,
    stopwords: BTreeSet<&'static str>,
    domain_targets: BTreeMap<(&'static str, &'static str), DomainTarget>,
}

impl CompendiousMachine {
    pub fn new() -> Self {
        let mut machine = Self {
            verbose_patterns: BTreeMap::new(),
            stopwords: BTreeSet::new(),
            domain_targets: BTreeMap::new(),
        };
        machine.init_patterns();
        machine.init_stopwords();
        machine.init_domain_targets();
        machine
    }

    fn init_patterns(&mut self) {
        // Throat-clearing deletions
        self.verbose_patterns
            .insert("it is important to note that", "");
        self.verbose_patterns
            .insert("it should be mentioned that", "");
        self.verbose_patterns.insert("as a matter of fact", "");
        self.verbose_patterns
            .insert("for all intents and purposes", "");
        self.verbose_patterns.insert("at the end of the day", "");
        self.verbose_patterns
            .insert("the fact of the matter is", "");
        self.verbose_patterns.insert("it is worth noting that", "");
        self.verbose_patterns.insert("needless to say", "");

        // Prepositional bloat
        self.verbose_patterns.insert("in order to", "to");
        self.verbose_patterns.insert("for the purpose of", "to");
        self.verbose_patterns.insert("with regard to", "about");
        self.verbose_patterns.insert("in reference to", "about");
        self.verbose_patterns.insert("in terms of", "regarding");
        self.verbose_patterns.insert("on the basis of", "based on");
        self.verbose_patterns.insert("in the event that", "if");
        self.verbose_patterns.insert("at this point in time", "now");
        self.verbose_patterns.insert("at the present time", "now");
        self.verbose_patterns.insert("prior to", "before");
        self.verbose_patterns.insert("subsequent to", "after");
        self.verbose_patterns
            .insert("in spite of the fact that", "although");
        self.verbose_patterns
            .insert("due to the fact that", "because");
        self.verbose_patterns
            .insert("in light of the fact that", "because");

        // Redundancies
        self.verbose_patterns
            .insert("completely finished", "finished");
        self.verbose_patterns
            .insert("absolutely essential", "essential");
        self.verbose_patterns
            .insert("basic fundamentals", "fundamentals");
        self.verbose_patterns.insert("past history", "history");
        self.verbose_patterns.insert("future plans", "plans");
        self.verbose_patterns.insert("end result", "result");
        self.verbose_patterns.insert("final outcome", "outcome");
        self.verbose_patterns.insert("close proximity", "near");
        self.verbose_patterns.insert("each and every", "each");
        self.verbose_patterns.insert("any and all", "all");
        self.verbose_patterns.insert("first and foremost", "first");

        // Nominalizations
        self.verbose_patterns.insert("make a decision", "decide");
        self.verbose_patterns
            .insert("give consideration to", "consider");
        self.verbose_patterns
            .insert("reach a conclusion", "conclude");
        self.verbose_patterns
            .insert("perform an analysis", "analyze");
        self.verbose_patterns
            .insert("conduct an investigation", "investigate");
        self.verbose_patterns
            .insert("provide assistance to", "help");
        self.verbose_patterns
            .insert("make an improvement", "improve");
        self.verbose_patterns.insert("take action", "act");

        // Common verbose phrases
        self.verbose_patterns.insert("a large number of", "many");
        self.verbose_patterns
            .insert("a significant number of", "many");
        self.verbose_patterns.insert("the vast majority of", "most");
        self.verbose_patterns.insert("in the near future", "soon");
        self.verbose_patterns
            .insert("at some point in the future", "eventually");
    }

    fn init_stopwords(&mut self) {
        let words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had",
            "do", "does", "did", "will", "would", "could", "should", "may", "might", "must",
            "shall", "can", "this", "that", "these", "those", "it", "its", "they", "them", "their",
        ];
        for w in words {
            self.stopwords.insert(w);
        }
    }

    fn init_domain_targets(&mut self) {
        let targets = [
            (
                ("technical", "api_reference"),
                2.5,
                "Maximum density, developers expect precision",
            ),
            (
                ("technical", "tutorial"),
                1.7,
                "Balance density with learning scaffolding",
            ),
            (
                ("technical", "readme"),
                2.0,
                "Quick orientation, high signal",
            ),
            (
                ("business", "executive_summary"),
                2.5,
                "Executives have zero tolerance for fluff",
            ),
            (
                ("business", "proposal"),
                2.0,
                "Persuasion requires some elaboration",
            ),
            (("business", "email"), 2.0, "Respect recipient time"),
            (
                ("academic", "abstract"),
                3.0,
                "Extreme density required by word limits",
            ),
            (("academic", "paper_body"), 1.4, "Argumentation needs room"),
            (
                ("legal", "contract"),
                1.8,
                "Precision over brevity, but no redundancy",
            ),
            (("legal", "brief"), 2.0, "Courts value concision"),
            (
                ("medical", "clinical_note"),
                2.5,
                "Time-critical, standardized",
            ),
            (
                ("medical", "patient_instructions"),
                1.6,
                "Clarity over density for lay readers",
            ),
            (("journalism", "headline"), 4.0, "Maximum compression"),
            (
                ("journalism", "lead_paragraph"),
                2.5,
                "5W1H in minimum words",
            ),
            (
                ("journalism", "article_body"),
                1.8,
                "Inverted pyramid allows pruning",
            ),
            (
                ("pharmacovigilance", "icsr_narrative"),
                2.3,
                "Regulatory scrutiny, completeness critical",
            ),
            (
                ("pharmacovigilance", "signal_report"),
                2.5,
                "Evidence synthesis, no fluff",
            ),
            (
                ("pharmacovigilance", "regulatory_alert"),
                3.0,
                "Time-sensitive, action-oriented",
            ),
        ];

        for ((domain, content_type), target, rationale) in targets {
            self.domain_targets.insert(
                (domain, content_type),
                DomainTarget {
                    domain: domain.to_string(),
                    content_type: content_type.to_string(),
                    target_cs: target,
                    rationale: rationale.to_string(),
                },
            );
        }
    }

    // ========== Tool Implementations ==========

    pub fn score_text(
        &self,
        text: &str,
        required_elements: Option<Vec<String>>,
    ) -> CompendiousResult {
        let i = self.information_content(text);
        let e = self.expression_cost(text);
        let c = self.completeness(text, &required_elements.unwrap_or_default());
        let r = self.readability(text);

        let density = if e > 0 { i / e as f64 } else { 0.0 };
        let score = density * c * r;

        CompendiousResult {
            score,
            information_bits: i,
            expression_cost: e,
            completeness: c,
            readability: r,
            limiting_factor: self.identify_limiting_factor(density, c, r),
            interpretation: self.interpret_score(score),
        }
    }

    pub fn compress_text(
        &self,
        text: &str,
        target_cs: Option<f64>,
        preserve: Option<Vec<String>>,
    ) -> CompressionResult {
        let original_score = self.score_text(text, None);
        let preserve_set: BTreeSet<String> = preserve.unwrap_or_default().into_iter().collect();
        // target_cs is the caller's desired Cs threshold. All applicable pattern substitutions
        // are applied regardless (we cannot exceed what static patterns allow), but the
        // returned improvement_percent is compared against this target so the caller can
        // judge whether the output meets their goal.
        let target = target_cs.unwrap_or(2.0);

        let mut compressed = text.to_lowercase();
        let mut patterns_applied = Vec::new();

        // Apply pattern replacements
        for (verbose, replacement) in &self.verbose_patterns {
            if compressed.contains(*verbose) {
                // Check if any preserved terms would be affected
                let should_skip = preserve_set
                    .iter()
                    .any(|p| verbose.contains(&p.to_lowercase()));
                if !should_skip {
                    let savings = verbose
                        .split_whitespace()
                        .count()
                        .saturating_sub(replacement.split_whitespace().count());
                    patterns_applied.push(PatternMatch {
                        pattern: verbose.to_string(),
                        found: verbose.to_string(),
                        replacement: replacement.to_string(),
                        savings,
                    });
                    compressed = compressed.replace(*verbose, *replacement);
                }
            }
        }

        // Clean up extra whitespace
        compressed = compressed.split_whitespace().collect::<Vec<_>>().join(" ");

        // Restore original casing for first letter of sentences
        let compressed = self.restore_sentence_casing(&compressed);

        let compressed_score = self.score_text(&compressed, None);
        let improvement = if original_score.score > 0.0 {
            ((compressed_score.score - original_score.score) / original_score.score) * 100.0
        } else {
            0.0
        };

        let target_achieved = compressed_score.score >= target;

        CompressionResult {
            original: text.to_string(),
            compressed,
            original_score,
            compressed_score,
            patterns_applied,
            improvement_percent: improvement,
            target_achieved,
        }
    }

    pub fn compare_texts(&self, original: &str, optimized: &str) -> ComparisonResult {
        let orig_score = self.score_text(original, None);
        let opt_score = self.score_text(optimized, None);

        let improvement = if orig_score.score > 0.0 {
            ((opt_score.score - orig_score.score) / orig_score.score) * 100.0
        } else {
            0.0
        };

        ComparisonResult {
            original: orig_score.clone(),
            optimized: opt_score.clone(),
            improvement_percent: improvement,
            tokens_saved: orig_score.expression_cost as i32 - opt_score.expression_cost as i32,
        }
    }

    pub fn analyze_patterns(&self, text: &str) -> Vec<PatternMatch> {
        let text_lower = text.to_lowercase();
        let mut matches = Vec::new();

        for (verbose, replacement) in &self.verbose_patterns {
            if text_lower.contains(*verbose) {
                let savings = verbose
                    .split_whitespace()
                    .count()
                    .saturating_sub(replacement.split_whitespace().count());
                matches.push(PatternMatch {
                    pattern: verbose.to_string(),
                    found: verbose.to_string(),
                    replacement: if replacement.is_empty() {
                        "[DELETE]".to_string()
                    } else {
                        replacement.to_string()
                    },
                    savings,
                });
            }
        }

        // Sort by savings (most impactful first)
        matches.sort_by(|a, b| b.savings.cmp(&a.savings));
        matches
    }

    pub fn get_domain_target(&self, domain: &str, content_type: &str) -> DomainTarget {
        self.domain_targets
            .get(&(domain, content_type))
            .cloned()
            .unwrap_or_else(|| {
                // Return generic target if specific not found
                DomainTarget {
                    domain: domain.to_string(),
                    content_type: content_type.to_string(),
                    target_cs: 1.8,
                    rationale: "Generic target; specific domain/content_type combination not found"
                        .to_string(),
                }
            })
    }

    // ========== Helper Functions ==========

    fn information_content(&self, text: &str) -> f64 {
        let lowercased = text.to_lowercase();
        let words: BTreeSet<&str> = lowercased
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .filter(|w| !self.stopwords.contains(w))
            .collect();

        // Each unique content word ≈ 4 bits of information
        words.len() as f64 * 4.0
    }

    fn expression_cost(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    fn completeness(&self, text: &str, required_elements: &[String]) -> f64 {
        if required_elements.is_empty() {
            return 1.0;
        }

        let text_lower = text.to_lowercase();
        let present = required_elements
            .iter()
            .filter(|req| text_lower.contains(&req.to_lowercase()))
            .count();

        present as f64 / required_elements.len() as f64
    }

    fn readability(&self, text: &str) -> f64 {
        let words: Vec<&str> = text.split_whitespace().collect();
        let sentences = text
            .matches(|c| c == '.' || c == '!' || c == '?')
            .count()
            .max(1);

        let avg_words_per_sentence = words.len() as f64 / sentences as f64;

        // Sentence-length penalty: penalise run-on sentences only.
        // Any sentence at or under 20 words: full score (1.0) — concision is never penalised.
        // Above 20 words: score decays toward 0.5 as sentences get longer.
        // This replaces the Flesch formula, which punishes technical vocabulary
        // and thereby inverts the compendious score for dense, precise text.
        if avg_words_per_sentence <= 20.0 {
            1.0
        } else {
            let excess = avg_words_per_sentence - 20.0;
            (1.0 / (1.0 + excess * 0.04)).clamp(0.5, 1.0)
        }
    }

    fn identify_limiting_factor(&self, density: f64, c: f64, r: f64) -> String {
        let factors = [
            (density, "Information Density"),
            (c, "Completeness"),
            (r, "Readability"),
        ];

        factors
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(val, name)| format!("{} ({:.2})", name, val))
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn interpret_score(&self, score: f64) -> String {
        match score {
            s if s < 0.5 => "Verbose - Aggressive compression needed".to_string(),
            s if s < 1.0 => "Adequate - Minor optimization possible".to_string(),
            s if s < 2.0 => "Efficient - Good compendious quality".to_string(),
            s if s < 5.0 => "Excellent - Publishable density".to_string(),
            _ => "Exceptional - Reference-grade compression".to_string(),
        }
    }

    fn restore_sentence_casing(&self, text: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for c in text.chars() {
            if capitalize_next && c.is_alphabetic() {
                result.push(c.to_uppercase().next().unwrap_or(c));
                capitalize_next = false;
            } else {
                result.push(c);
            }

            if c == '.' || c == '!' || c == '?' {
                capitalize_next = true;
            }
        }

        result
    }
}

impl Default for CompendiousMachine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// MCP Request/Response Handling
// ============================================

#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

pub fn handle_mcp_request(
    machine: &CompendiousMachine,
    request: &McpRequest,
) -> Option<McpResponse> {
    let id = request.id.clone();

    // MCP notifications have no id and must not receive a response.
    // Return None to signal the caller to drop the line silently.
    if request.method.starts_with("notifications/") {
        return None;
    }

    Some(match request.method.as_str() {
        "initialize" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "compendious-machine",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        },

        "tools/list" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(get_tool_definitions()),
            error: None,
        },

        "tools/call" => {
            let params = request.params.as_ref();
            let tool_name = params
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments = params
                .and_then(|p| p.get("arguments"))
                .cloned()
                .unwrap_or(json!({}));

            let result = match tool_name {
                "score_text" => {
                    let text = arguments.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    let required: Option<Vec<String>> = arguments
                        .get("required_elements")
                        .and_then(|r| serde_json::from_value(r.clone()).ok());
                    let score = machine.score_text(text, required);
                    let markdown = format!(
                        "# Compendious Score Analysis\n\n\
                        ## Result\n\
                        **Cs = {:.2}** — {}\n\n\
                        ## Component Breakdown\n\
                        | Component | Value |\n\
                        |-----------|-------|\n\
                        | Information (I) | {:.1} bits |\n\
                        | Expression (E) | {} tokens |\n\
                        | Completeness (C) | {:.2} |\n\
                        | Readability (R) | {:.2} |\n\n\
                        **Limiting Factor:** {}",
                        score.score,
                        score.interpretation,
                        score.information_bits,
                        score.expression_cost,
                        score.completeness,
                        score.readability,
                        score.limiting_factor
                    );
                    json!({
                        "content": [{
                            "type": "text",
                            "text": markdown
                        }]
                    })
                }

                "compress_text" => {
                    let text = arguments.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    let target_cs = arguments.get("target_cs").and_then(|t| t.as_f64());
                    let preserve: Option<Vec<String>> = arguments
                        .get("preserve")
                        .and_then(|p| serde_json::from_value(p.clone()).ok());
                    let result = machine.compress_text(text, target_cs, preserve);
                    let patterns_md: String = result
                        .patterns_applied
                        .iter()
                        .map(|p| {
                            format!(
                                "- \"{}\" → \"{}\" (saved {} tokens)",
                                p.pattern,
                                if p.replacement.is_empty() {
                                    "[DELETE]"
                                } else {
                                    &p.replacement
                                },
                                p.savings
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    let target_status = if result.target_achieved {
                        "Target achieved".to_string()
                    } else {
                        format!(
                            "Target not yet reached (current: {:.2})",
                            result.compressed_score.score
                        )
                    };
                    let markdown = format!(
                        "# Compression Result\n\n\
                        ## Metrics\n\
                        | Metric | Original | Compressed |\n\
                        |--------|----------|------------|\n\
                        | Tokens | {} | {} |\n\
                        | Cs Score | {:.2} | {:.2} |\n\
                        | Improvement | — | **{:+.1}%** |\n\
                        | Target Cs | — | {} |\n\n\
                        ## Compressed Text\n\
                        {}\n\n\
                        ## Patterns Applied\n\
                        {}",
                        result.original_score.expression_cost,
                        result.compressed_score.expression_cost,
                        result.original_score.score,
                        result.compressed_score.score,
                        result.improvement_percent,
                        target_status,
                        result.compressed,
                        if patterns_md.is_empty() {
                            "No verbose patterns detected.".to_string()
                        } else {
                            patterns_md
                        }
                    );
                    json!({
                        "content": [{
                            "type": "text",
                            "text": markdown
                        }]
                    })
                }

                "compare_texts" => {
                    let original = arguments
                        .get("original")
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    let optimized = arguments
                        .get("optimized")
                        .and_then(|t| t.as_str())
                        .unwrap_or("");
                    let result = machine.compare_texts(original, optimized);
                    let markdown = format!(
                        "# Text Comparison\n\n\
                        ## Metrics\n\
                        | Metric | Original | Optimized |\n\
                        |--------|----------|----------|\n\
                        | Tokens | {} | {} |\n\
                        | Cs Score | {:.2} | {:.2} |\n\
                        | Information (bits) | {:.1} | {:.1} |\n\
                        | Completeness | {:.2} | {:.2} |\n\
                        | Readability | {:.2} | {:.2} |\n\n\
                        ## Summary\n\
                        - **Improvement:** {:+.1}%\n\
                        - **Tokens Saved:** {}\n\
                        - **Original Limiting Factor:** {}\n\
                        - **Optimized Limiting Factor:** {}",
                        result.original.expression_cost,
                        result.optimized.expression_cost,
                        result.original.score,
                        result.optimized.score,
                        result.original.information_bits,
                        result.optimized.information_bits,
                        result.original.completeness,
                        result.optimized.completeness,
                        result.original.readability,
                        result.optimized.readability,
                        result.improvement_percent,
                        result.tokens_saved,
                        result.original.limiting_factor,
                        result.optimized.limiting_factor
                    );
                    json!({
                        "content": [{
                            "type": "text",
                            "text": markdown
                        }]
                    })
                }

                "analyze_patterns" => {
                    let text = arguments.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    let patterns = machine.analyze_patterns(text);
                    let patterns_md: String = if patterns.is_empty() {
                        "No verbose patterns detected. Text appears optimized.".to_string()
                    } else {
                        let header = "| Pattern Found | Replacement | Tokens Saved |\n|---------------|-------------|--------------|".to_string();
                        let rows: String = patterns
                            .iter()
                            .map(|p| {
                                format!(
                                    "| {} | {} | {} |",
                                    p.found,
                                    if p.replacement.is_empty() {
                                        "[DELETE]"
                                    } else {
                                        &p.replacement
                                    },
                                    p.savings
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        format!("{}\n{}", header, rows)
                    };
                    let total_savings: usize = patterns.iter().map(|p| p.savings).sum();
                    let markdown = format!(
                        "# Pattern Analysis\n\n\
                        ## Verbose Patterns Detected: {}\n\n\
                        {}\n\n\
                        ## Potential Savings\n\
                        **Total tokens recoverable:** {}",
                        patterns.len(),
                        patterns_md,
                        total_savings
                    );
                    json!({
                        "content": [{
                            "type": "text",
                            "text": markdown
                        }]
                    })
                }

                "get_domain_target" => {
                    let domain = arguments
                        .get("domain")
                        .and_then(|d| d.as_str())
                        .unwrap_or("general");
                    let content_type = arguments
                        .get("content_type")
                        .and_then(|c| c.as_str())
                        .unwrap_or("default");
                    let target = machine.get_domain_target(domain, content_type);
                    let markdown = format!(
                        "# Domain Target\n\n\
                        - **Domain:** {}\n\
                        - **Content Type:** {}\n\
                        - **Target Cs:** {:.1}\n\
                        - **Rationale:** {}",
                        target.domain, target.content_type, target.target_cs, target.rationale
                    );
                    json!({
                        "content": [{
                            "type": "text",
                            "text": markdown
                        }]
                    })
                }

                _ => json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Unknown tool: {}", tool_name)
                    }],
                    "isError": true
                }),
            };

            McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            }
        }

        _ => McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    })
}

// ============================================
// Main Entry Point
// ============================================

use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--mcp".to_string()) {
        run_mcp_server();
    } else {
        eprintln!("Compendious Machine MCP Server v2.0");
        eprintln!("Usage: compendious-machine --mcp");
        eprintln!();
        eprintln!("Run as MCP server for AI agent integration.");
    }
}

fn write_response(stdout: &mut io::Stdout, response: &McpResponse) -> io::Result<()> {
    let encoded = serde_json::to_string(response)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writeln!(stdout, "{}", encoded)?;
    stdout.flush()
}

fn run_mcp_server() {
    let machine = CompendiousMachine::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.is_empty() {
            continue;
        }

        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let error_response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                };
                if let Err(io_err) = write_response(&mut stdout, &error_response) {
                    eprintln!(
                        "compendious-machine: failed to write error response: {}",
                        io_err
                    );
                }
                continue;
            }
        };

        // Notifications return None — no response must be sent per MCP spec.
        let Some(response) = handle_mcp_request(&machine, &request) else {
            continue;
        };
        if let Err(io_err) = write_response(&mut stdout, &response) {
            eprintln!("compendious-machine: failed to write response: {}", io_err);
            break;
        }
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers ----

    fn machine() -> CompendiousMachine {
        CompendiousMachine::new()
    }

    fn mcp_req(method: &str, params: Option<Value>) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: method.to_string(),
            params,
        }
    }

    // ---- CompendiousMachine::new / Default ----

    #[test]
    fn test_new_and_default_equivalent() {
        // Both constructors must produce a machine with identical domain target counts.
        let a = CompendiousMachine::new();
        let b = CompendiousMachine::default();
        assert_eq!(
            a.domain_targets.len(),
            b.domain_targets.len(),
            "new() and default() should produce identical machines"
        );
    }

    // ---- score_text ----

    #[test]
    fn test_verbose_scores_lower() {
        let m = machine();
        let verbose = "In order to facilitate the implementation of the aforementioned solution, it is important to note that we should consider the various factors.";
        let compendious = "Three factors affect implementation: cost, time, complexity.";
        let v = m.score_text(verbose, None);
        let c = m.score_text(compendious, None);
        assert!(v.score.is_finite());
        assert!(c.score.is_finite());
        assert!(v.score >= 0.0);
        assert!(c.score >= 0.0);
        assert!(
            c.score > v.score,
            "compendious text should score higher than verbose text: {:.4} vs {:.4}",
            c.score,
            v.score
        );
    }

    #[test]
    fn test_score_empty_string() {
        let m = machine();
        let result = m.score_text("", None);
        // Empty text: expression_cost = 0, score = 0.
        assert_eq!(result.expression_cost, 0);
        assert_eq!(result.score, 0.0);
        assert!(result.score.is_finite());
    }

    #[test]
    fn test_score_components_range() {
        let m = machine();
        let text = "Analyze adverse events in clinical data.";
        let r = m.score_text(text, None);
        assert!(r.information_bits >= 0.0);
        assert!(r.completeness >= 0.0 && r.completeness <= 1.0);
        assert!(r.readability >= 0.0 && r.readability <= 1.0);
        assert!(r.expression_cost > 0);
    }

    #[test]
    fn test_score_with_required_elements_all_present() {
        let m = machine();
        let text = "Signal detection uses PRR and ROR algorithms.";
        let required = Some(vec!["PRR".to_string(), "ROR".to_string()]);
        let r = m.score_text(text, required);
        assert!(
            (r.completeness - 1.0).abs() < 1e-9,
            "all required elements present: completeness should be 1.0, got {:.4}",
            r.completeness
        );
    }

    #[test]
    fn test_score_with_required_elements_partial() {
        let m = machine();
        let text = "Signal detection uses PRR.";
        let required = Some(vec!["PRR".to_string(), "EBGM".to_string()]);
        let r = m.score_text(text, required);
        assert!(
            (r.completeness - 0.5).abs() < 1e-9,
            "half required elements present: completeness should be 0.5, got {:.4}",
            r.completeness
        );
    }

    #[test]
    fn test_score_with_required_elements_none_present() {
        let m = machine();
        let text = "Entirely unrelated sentence.";
        let required = Some(vec!["PRR".to_string(), "EBGM".to_string()]);
        let r = m.score_text(text, required);
        assert_eq!(
            r.completeness, 0.0,
            "no required elements present: completeness should be 0.0, got {:.4}",
            r.completeness
        );
        assert_eq!(r.score, 0.0, "score must be zero when completeness=0");
    }

    // ---- interpret_score bands ----

    #[test]
    fn test_interpret_score_bands() {
        let m = machine();
        // Force known scores by crafting the result struct directly via score_text
        // on carefully chosen inputs, then verify the interpretation string.
        // Alternatively test all five branches via the private method through public API.
        // We use single-word + required_elements=missing to drive completeness to 0 for
        // the zero band, then verify each band label appears in interpretation.

        // Verbose band: Cs < 0.5 — achieved by requiring a missing element (C=0 → Cs=0)
        let zero = m.score_text("word.", Some(vec!["absent_token_xyz".to_string()]));
        assert_eq!(zero.score, 0.0);
        assert!(
            zero.interpretation.to_lowercase().contains("verbose"),
            "score 0.0 should be 'Verbose', got: {}",
            zero.interpretation
        );

        // Efficient band: 1.0 <= Cs < 2.0
        let efficient = m.score_text("Analyze adverse events carefully.", None);
        assert!(
            efficient.score >= 0.0,
            "score should be non-negative: {:.4}",
            efficient.score
        );
        // We can only verify the label is one of the valid strings since exact score
        // depends on unique word counting.
        let valid_labels = [
            "Verbose",
            "Adequate",
            "Efficient",
            "Excellent",
            "Exceptional",
        ];
        assert!(
            valid_labels
                .iter()
                .any(|l| efficient.interpretation.contains(l)),
            "interpretation '{}' should be one of the valid labels",
            efficient.interpretation
        );

        // Headline-density text: a very short, high-unique-word sentence should
        // score higher, potentially in Excellent/Exceptional range.
        let dense = m.score_text("PRR ROR IC EBGM FAERS signal pharmacovigilance.", None);
        assert!(dense.score > 0.0);
    }

    // ---- identify_limiting_factor ----

    #[test]
    fn test_limiting_factor_completeness() {
        let m = machine();
        // Force completeness = 0 → it must be the limiting factor.
        let r = m.score_text(
            "Analyze events.",
            Some(vec!["absent_xyz_token".to_string()]),
        );
        assert!(
            r.limiting_factor.contains("Completeness"),
            "when completeness=0, limiting factor should be Completeness, got: {}",
            r.limiting_factor
        );
    }

    #[test]
    fn test_limiting_factor_readability() {
        let m = machine();
        // Long run-on sentence drives readability below other factors.
        let long_sentence = "The system will attempt to perform an analysis of all the various underlying data structures and components present within the broader architectural context of the application framework in order to facilitate decision making.";
        let r = m.score_text(long_sentence, None);
        // Readability should be < 1.0 and should be named as limiting factor.
        assert!(r.readability < 1.0);
        assert!(
            r.limiting_factor.contains("Readability"),
            "long run-on sentence: limiting factor should be Readability, got: {}",
            r.limiting_factor
        );
    }

    // ---- readability ----

    #[test]
    fn test_readability_penalty() {
        let m = machine();
        // Short sentence: readability = 1.0
        let short = "Execute test now.";
        let r_short = m.score_text(short, None).readability;
        assert!(
            (r_short - 1.0).abs() < f64::EPSILON,
            "short sentence should have R=1.0, got {:.4}",
            r_short
        );
        // Very long single sentence: readability < 1.0
        let long = "The system will attempt to perform an analysis of all the various underlying data structures and components that are present within the broader architectural context of the application framework.";
        let r_long = m.score_text(long, None).readability;
        assert!(
            r_long < 1.0,
            "long sentence should have R < 1.0, got {:.4}",
            r_long
        );
        // Readability never drops below 0.5 (clamped).
        assert!(
            r_long >= 0.5,
            "readability should be clamped at 0.5, got {:.4}",
            r_long
        );
    }

    #[test]
    fn test_readability_exactly_at_boundary() {
        let m = machine();
        // Exactly 20-word single sentence — readability must be 1.0.
        let twenty_words = "One two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty.";
        let r = m.score_text(twenty_words, None).readability;
        assert!(
            (r - 1.0).abs() < f64::EPSILON,
            "20-word sentence should have R=1.0, got {:.4}",
            r
        );
    }

    // ---- restore_sentence_casing (tested via compress_text output) ----

    #[test]
    fn test_restore_sentence_casing_first_letter_capitalized() {
        let m = machine();
        // Input is all lowercase after pattern substitution; compressed output
        // must start with a capital letter.
        let text = "in order to succeed, analyze the data.";
        let result = m.compress_text(text, None, None);
        let first_char = result.compressed.chars().next();
        assert!(
            first_char.map(|c| c.is_uppercase()).unwrap_or(false),
            "compressed text must start with uppercase, got: {:?}",
            &result.compressed[..result.compressed.len().min(20)]
        );
    }

    // ---- analyze_patterns ----

    #[test]
    fn test_pattern_detection() {
        let m = machine();
        let text = "In order to make a decision, it is important to note that we should give consideration to all factors.";
        let patterns = m.analyze_patterns(text);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.pattern.contains("in order to")));
    }

    #[test]
    fn test_analyze_patterns_clean_text_returns_empty() {
        let m = machine();
        // Tightly written text with no known verbose patterns.
        let text = "Signal detected. PRR exceeds threshold. Investigate.";
        let patterns = m.analyze_patterns(text);
        assert!(
            patterns.is_empty(),
            "clean text should yield no verbose patterns, got: {:?}",
            patterns
        );
    }

    #[test]
    fn test_analyze_patterns_sorted_by_savings_descending() {
        let m = machine();
        // Use a text that contains patterns of different word counts so savings differ.
        let text = "Due to the fact that we need to give consideration to this, in order to proceed, and for the purpose of clarity, we should perform an analysis.";
        let patterns = m.analyze_patterns(text);
        assert!(
            patterns.len() >= 2,
            "expected multiple patterns, got {}",
            patterns.len()
        );
        for window in patterns.windows(2) {
            assert!(
                window[0].savings >= window[1].savings,
                "patterns should be sorted descending by savings: {} < {}",
                window[0].savings,
                window[1].savings
            );
        }
    }

    #[test]
    fn test_analyze_patterns_replacement_empty_shown_as_delete() {
        let m = machine();
        // "it is important to note that" maps to "" — replacement should show "[DELETE]"
        let text = "It is important to note that this matters.";
        let patterns = m.analyze_patterns(text);
        let throat_clear = patterns
            .iter()
            .find(|p| p.pattern == "it is important to note that");
        assert!(
            throat_clear.is_some(),
            "should detect 'it is important to note that'"
        );
        let p = throat_clear.unwrap();
        assert_eq!(
            p.replacement, "[DELETE]",
            "empty replacement should display as [DELETE], got: {}",
            p.replacement
        );
    }

    // ---- compress_text ----

    #[test]
    fn test_compression() {
        let m = machine();
        let text = "In order to make a decision at this point in time, we need to give consideration to the basic fundamentals.";
        let result = m.compress_text(text, None, None);
        assert!(
            result.patterns_applied.len() >= 3,
            "expected at least 3 patterns applied, got {}",
            result.patterns_applied.len()
        );
        assert!(
            result.improvement_percent > 0.0,
            "expected positive improvement, got {:.2}%",
            result.improvement_percent
        );
        assert!(result.compressed.len() < text.len());
    }

    #[test]
    fn test_compression_no_patterns_no_degradation() {
        let m = machine();
        // Text with no known verbose patterns: compressed = original (after casing normalization).
        let text = "Signal detected. PRR exceeds threshold. Investigate now.";
        let result = m.compress_text(text, None, None);
        assert!(
            result.patterns_applied.is_empty(),
            "clean text: no patterns should be applied"
        );
        // Score should not degrade.
        assert!(
            result.compressed_score.score >= result.original_score.score - 1e-9,
            "compression must not degrade score: {:.4} < {:.4}",
            result.compressed_score.score,
            result.original_score.score
        );
    }

    #[test]
    fn test_compression_preserve_skips_matching_pattern() {
        let m = machine();
        // "in order to" is a known pattern; preserving "order" should prevent its replacement.
        let text = "In order to succeed.";
        let result = m.compress_text(text, None, Some(vec!["order".to_string()]));
        assert!(
            result.patterns_applied.is_empty(),
            "pattern containing preserved term 'order' must be skipped"
        );
        // The original phrase must still be present in the compressed output.
        assert!(
            result.compressed.to_lowercase().contains("order"),
            "preserved term 'order' must survive in compressed output: {}",
            result.compressed
        );
    }

    #[test]
    fn test_compression_preserve_empty_applies_all_patterns() {
        let m = machine();
        let text = "In order to make a decision at this point in time.";
        let result_no_preserve = m.compress_text(text, None, None);
        let result_empty_preserve = m.compress_text(text, None, Some(vec![]));
        assert_eq!(
            result_no_preserve.patterns_applied.len(),
            result_empty_preserve.patterns_applied.len(),
            "empty preserve list should behave identically to None"
        );
    }

    #[test]
    fn test_compression_default_target_cs_is_two() {
        let m = machine();
        // Passing no target_cs and Some(2.0) must produce the same target_achieved outcome.
        let text = "In order to act, decide.";
        let r_none = m.compress_text(text, None, None);
        let r_two = m.compress_text(text, Some(2.0), None);
        assert_eq!(
            r_none.target_achieved, r_two.target_achieved,
            "default target_cs=2.0 must match explicit Some(2.0)"
        );
    }

    #[test]
    fn test_target_cs_achieved_flag() {
        let m = machine();
        let verbose = "In order to make a decision at this point in time, it is important to note that we should give consideration to the basic fundamentals.";
        let result = m.compress_text(verbose, Some(0.5), None);
        assert!(
            result.target_achieved,
            "expected target_achieved=true with target=0.5, got compressed Cs={:.4}",
            result.compressed_score.score
        );
        let result_high = m.compress_text(verbose, Some(50.0), None);
        assert!(
            !result_high.target_achieved,
            "expected target_achieved=false with target=50.0, got Cs={:.4}",
            result_high.compressed_score.score
        );
    }

    // ---- compare_texts ----

    #[test]
    fn test_compare_texts() {
        let m = machine();
        let original = "In order to achieve success we need to give consideration to the various underlying factors.";
        let optimized = "To succeed, consider the underlying factors.";
        let result = m.compare_texts(original, optimized);
        assert!(
            result.optimized.score > result.original.score,
            "optimized should score higher: {:.4} vs {:.4}",
            result.optimized.score,
            result.original.score
        );
        assert!(
            result.improvement_percent > 0.0,
            "expected positive improvement, got {:.2}%",
            result.improvement_percent
        );
        assert!(
            result.tokens_saved > 0,
            "expected positive tokens_saved, got {}",
            result.tokens_saved
        );
    }

    #[test]
    fn test_compare_texts_identical_zero_improvement() {
        let m = machine();
        let text = "Analyze signal data carefully.";
        let result = m.compare_texts(text, text);
        assert!(
            result.improvement_percent.abs() < 1e-9,
            "identical texts: improvement should be 0, got {:.4}",
            result.improvement_percent
        );
        assert_eq!(
            result.tokens_saved, 0,
            "identical texts: tokens_saved should be 0, got {}",
            result.tokens_saved
        );
    }

    #[test]
    fn test_compare_texts_negative_tokens_saved_when_longer() {
        let m = machine();
        let short = "Act now.";
        let long = "In order to act, you should consider doing it now.";
        let result = m.compare_texts(short, long);
        assert!(
            result.tokens_saved < 0,
            "longer optimized text should produce negative tokens_saved, got {}",
            result.tokens_saved
        );
    }

    // ---- get_domain_target ----

    #[test]
    fn test_get_domain_target_known() {
        let m = machine();
        let target = m.get_domain_target("journalism", "headline");
        assert_eq!(target.domain, "journalism");
        assert_eq!(target.content_type, "headline");
        assert!(
            (target.target_cs - 4.0).abs() < f64::EPSILON,
            "expected 4.0, got {}",
            target.target_cs
        );
    }

    #[test]
    fn test_get_domain_target_fallback() {
        let m = machine();
        let target = m.get_domain_target("unknown_domain", "unknown_type");
        assert_eq!(target.domain, "unknown_domain");
        assert_eq!(target.content_type, "unknown_type");
        assert!(
            (target.target_cs - 1.8).abs() < f64::EPSILON,
            "expected fallback 1.8, got {}",
            target.target_cs
        );
    }

    #[test]
    fn test_get_domain_target_all_known_combinations_valid() {
        let m = machine();
        let known = [
            ("technical", "api_reference", 2.5),
            ("technical", "tutorial", 1.7),
            ("technical", "readme", 2.0),
            ("business", "executive_summary", 2.5),
            ("business", "proposal", 2.0),
            ("business", "email", 2.0),
            ("academic", "abstract", 3.0),
            ("academic", "paper_body", 1.4),
            ("legal", "contract", 1.8),
            ("legal", "brief", 2.0),
            ("medical", "clinical_note", 2.5),
            ("medical", "patient_instructions", 1.6),
            ("journalism", "headline", 4.0),
            ("journalism", "lead_paragraph", 2.5),
            ("journalism", "article_body", 1.8),
            ("pharmacovigilance", "icsr_narrative", 2.3),
            ("pharmacovigilance", "signal_report", 2.5),
            ("pharmacovigilance", "regulatory_alert", 3.0),
        ];
        for (domain, content_type, expected_cs) in known {
            let t = m.get_domain_target(domain, content_type);
            assert!(
                (t.target_cs - expected_cs).abs() < 1e-9,
                "{}/{}: expected {}, got {}",
                domain,
                content_type,
                expected_cs,
                t.target_cs
            );
            assert!(
                !t.rationale.is_empty(),
                "rationale must be non-empty for {}/{}",
                domain,
                content_type
            );
        }
    }

    // ---- get_tool_definitions ----

    #[test]
    fn test_get_tool_definitions_contains_all_tools() {
        let defs = get_tool_definitions();
        let tools = defs["tools"].as_array().expect("tools must be array");
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        for expected in [
            "score_text",
            "compress_text",
            "compare_texts",
            "analyze_patterns",
            "get_domain_target",
        ] {
            assert!(
                names.contains(&expected),
                "tool '{}' missing from tool definitions; found: {:?}",
                expected,
                names
            );
        }
        assert_eq!(
            tools.len(),
            5,
            "expected exactly 5 tools, got {}",
            tools.len()
        );
    }

    // ---- handle_mcp_request ----

    #[test]
    fn test_mcp_initialize() {
        let m = machine();
        let req = mcp_req("initialize", None);
        let resp = handle_mcp_request(&m, &req).expect("initialize must return a response");
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.error.is_none());
        let result = resp.result.expect("initialize must have result");
        assert!(result["protocolVersion"].is_string());
        assert_eq!(
            result["serverInfo"]["name"].as_str(),
            Some("compendious-machine")
        );
    }

    #[test]
    fn test_mcp_tools_list() {
        let m = machine();
        let req = mcp_req("tools/list", None);
        let resp = handle_mcp_request(&m, &req).expect("tools/list must return a response");
        assert!(resp.error.is_none());
        let result = resp.result.expect("tools/list must have result");
        let tools = result["tools"].as_array().expect("tools must be array");
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn test_mcp_notification_returns_none() {
        let m = machine();
        let req = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "notifications/initialized".to_string(),
            params: None,
        };
        let resp = handle_mcp_request(&m, &req);
        assert!(resp.is_none(), "notifications must return None");
    }

    #[test]
    fn test_mcp_unknown_method_returns_error() {
        let m = machine();
        let req = mcp_req("nonexistent/method", None);
        let resp = handle_mcp_request(&m, &req).expect("unknown method must return a response");
        assert!(resp.error.is_some(), "unknown method must set error field");
        let err = resp.error.unwrap();
        assert_eq!(err.code, -32601);
    }

    #[test]
    fn test_mcp_tool_call_score_text() {
        let m = machine();
        let req = mcp_req(
            "tools/call",
            Some(json!({
                "name": "score_text",
                "arguments": { "text": "Analyze PRR and ROR signals." }
            })),
        );
        let resp = handle_mcp_request(&m, &req).expect("score_text call must return a response");
        assert!(resp.error.is_none());
        let result = resp.result.expect("tools/call must have result");
        let text = result["content"][0]["text"]
            .as_str()
            .expect("text field must exist");
        assert!(
            text.contains("Compendious Score"),
            "score_text response must contain 'Compendious Score'"
        );
    }

    #[test]
    fn test_mcp_tool_call_compress_text() {
        let m = machine();
        let req = mcp_req(
            "tools/call",
            Some(json!({
                "name": "compress_text",
                "arguments": {
                    "text": "In order to make a decision, give consideration to the basic fundamentals.",
                    "target_cs": 1.5
                }
            })),
        );
        let resp = handle_mcp_request(&m, &req).expect("compress_text call must return response");
        assert!(resp.error.is_none());
        let result = resp.result.expect("tools/call must have result");
        let text = result["content"][0]["text"].as_str().expect("text field");
        assert!(
            text.contains("Compression Result"),
            "compress_text must include 'Compression Result'"
        );
    }

    #[test]
    fn test_mcp_tool_call_compare_texts() {
        let m = machine();
        let req = mcp_req(
            "tools/call",
            Some(json!({
                "name": "compare_texts",
                "arguments": {
                    "original": "In order to succeed, give consideration to all factors.",
                    "optimized": "To succeed, consider all factors."
                }
            })),
        );
        let resp = handle_mcp_request(&m, &req).expect("compare_texts call must return response");
        assert!(resp.error.is_none());
        let result = resp.result.expect("tools/call must have result");
        let text = result["content"][0]["text"].as_str().expect("text field");
        assert!(text.contains("Text Comparison"));
    }

    #[test]
    fn test_mcp_tool_call_analyze_patterns() {
        let m = machine();
        let req = mcp_req(
            "tools/call",
            Some(json!({
                "name": "analyze_patterns",
                "arguments": { "text": "In order to act at this point in time." }
            })),
        );
        let resp =
            handle_mcp_request(&m, &req).expect("analyze_patterns call must return response");
        assert!(resp.error.is_none());
        let result = resp.result.expect("tools/call must have result");
        let text = result["content"][0]["text"].as_str().expect("text field");
        assert!(text.contains("Pattern Analysis"));
    }

    #[test]
    fn test_mcp_tool_call_get_domain_target() {
        let m = machine();
        let req = mcp_req(
            "tools/call",
            Some(json!({
                "name": "get_domain_target",
                "arguments": { "domain": "medical", "content_type": "clinical_note" }
            })),
        );
        let resp =
            handle_mcp_request(&m, &req).expect("get_domain_target call must return response");
        assert!(resp.error.is_none());
        let result = resp.result.expect("tools/call must have result");
        let text = result["content"][0]["text"].as_str().expect("text field");
        assert!(text.contains("Domain Target"));
        assert!(text.contains("medical"));
        assert!(text.contains("2.5"));
    }

    #[test]
    fn test_mcp_tool_call_unknown_tool() {
        let m = machine();
        let req = mcp_req(
            "tools/call",
            Some(json!({
                "name": "nonexistent_tool",
                "arguments": {}
            })),
        );
        let resp = handle_mcp_request(&m, &req).expect("unknown tool must return a response");
        assert!(
            resp.error.is_none(),
            "unknown tool uses isError in result, not error field"
        );
        let result = resp.result.expect("tools/call must have result");
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(
            is_error,
            "unknown tool call must set isError=true in result"
        );
    }

    // ---- serde round-trip ----

    #[test]
    fn test_mcp_request_serde_round_trip() {
        let json_str = r#"{"jsonrpc":"2.0","id":42,"method":"tools/list","params":null}"#;
        let req: McpRequest = serde_json::from_str(json_str).expect("must deserialize McpRequest");
        assert_eq!(req.method, "tools/list");
        assert_eq!(req.id, Some(json!(42)));
    }

    #[test]
    fn test_mcp_response_skips_none_fields() {
        let resp = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: Some(json!({"ok": true})),
            error: None,
        };
        let encoded = serde_json::to_string(&resp).expect("must serialize");
        assert!(
            !encoded.contains("\"error\""),
            "None error must be omitted from JSON"
        );
    }

    #[test]
    fn test_compendious_result_serde_round_trip() {
        let r = CompendiousResult {
            score: 2.5,
            information_bits: 40.0,
            expression_cost: 10,
            completeness: 1.0,
            readability: 0.9,
            limiting_factor: "Readability (0.90)".to_string(),
            interpretation: "Excellent".to_string(),
        };
        let json = serde_json::to_string(&r).expect("serialize");
        let back: CompendiousResult = serde_json::from_str(&json).expect("deserialize");
        assert!((back.score - 2.5).abs() < 1e-9);
        assert_eq!(back.expression_cost, 10);
    }

    #[test]
    fn test_domain_target_serde_round_trip() {
        let t = DomainTarget {
            domain: "medical".to_string(),
            content_type: "clinical_note".to_string(),
            target_cs: 2.5,
            rationale: "Time-critical".to_string(),
        };
        let json = serde_json::to_string(&t).expect("serialize");
        let back: DomainTarget = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.domain, "medical");
        assert!((back.target_cs - 2.5).abs() < 1e-9);
    }

    #[test]
    fn test_pattern_match_serde_round_trip() {
        let p = PatternMatch {
            pattern: "in order to".to_string(),
            found: "in order to".to_string(),
            replacement: "to".to_string(),
            savings: 2,
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let back: PatternMatch = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.savings, 2);
        assert_eq!(back.replacement, "to");
    }
}
