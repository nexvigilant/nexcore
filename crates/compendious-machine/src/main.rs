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

    #[test]
    fn test_verbose_scores_lower() {
        let machine = CompendiousMachine::new();

        let verbose = "In order to facilitate the implementation of the aforementioned solution, it is important to note that we should consider the various factors.";
        let compendious = "Three factors affect implementation: cost, time, complexity.";

        let v_score = machine.score_text(verbose, None);
        let c_score = machine.score_text(compendious, None);

        assert!(v_score.score.is_finite());
        assert!(c_score.score.is_finite());
        assert!(v_score.score >= 0.0);
        assert!(c_score.score >= 0.0);
        // Core invariant: the concise text must score strictly higher than the verbose one.
        assert!(
            c_score.score > v_score.score,
            "compendious text should score higher than verbose text: {:.4} vs {:.4}",
            c_score.score,
            v_score.score
        );
    }

    #[test]
    fn test_pattern_detection() {
        let machine = CompendiousMachine::new();
        let text = "In order to make a decision, it is important to note that we should give consideration to all factors.";

        let patterns = machine.analyze_patterns(text);
        assert!(!patterns.is_empty());
        assert!(patterns.iter().any(|p| p.pattern.contains("in order to")));
    }

    #[test]
    fn test_compression() {
        let machine = CompendiousMachine::new();
        let text = "In order to make a decision at this point in time, we need to give consideration to the basic fundamentals.";

        let result = machine.compress_text(text, None, None);
        // At least 3 known patterns in the input must have been detected and applied.
        assert!(
            result.patterns_applied.len() >= 3,
            "expected at least 3 patterns applied, got {}",
            result.patterns_applied.len()
        );
        // Score must strictly improve, not just stay the same.
        assert!(
            result.improvement_percent > 0.0,
            "expected positive improvement, got {:.2}%",
            result.improvement_percent
        );
        assert!(result.compressed.len() < text.len());
    }

    #[test]
    fn test_compare_texts() {
        let machine = CompendiousMachine::new();
        let original = "In order to achieve success we need to give consideration to the various underlying factors.";
        let optimized = "To succeed, consider the underlying factors.";

        let result = machine.compare_texts(original, optimized);

        // Optimized must score higher than original.
        assert!(
            result.optimized.score > result.original.score,
            "optimized should score higher: {:.4} vs {:.4}",
            result.optimized.score,
            result.original.score
        );
        // Improvement percent must be positive.
        assert!(
            result.improvement_percent > 0.0,
            "expected positive improvement, got {:.2}%",
            result.improvement_percent
        );
        // Tokens saved must be positive (shorter text).
        assert!(
            result.tokens_saved > 0,
            "expected positive tokens_saved, got {}",
            result.tokens_saved
        );
    }

    #[test]
    fn test_get_domain_target_known() {
        let machine = CompendiousMachine::new();

        // Known combination: journalism/headline = 4.0
        let target = machine.get_domain_target("journalism", "headline");
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
        let machine = CompendiousMachine::new();

        // Unknown combination: should return generic fallback at 1.8.
        let target = machine.get_domain_target("unknown_domain", "unknown_type");
        assert_eq!(target.domain, "unknown_domain");
        assert_eq!(target.content_type, "unknown_type");
        assert!(
            (target.target_cs - 1.8).abs() < f64::EPSILON,
            "expected fallback 1.8, got {}",
            target.target_cs
        );
    }

    #[test]
    fn test_target_cs_achieved_flag() {
        let machine = CompendiousMachine::new();

        // High-verbosity text: after compression should improve, and with a low
        // target_cs (0.5) the target_achieved flag must be true.
        let verbose = "In order to make a decision at this point in time, it is important to note that we should give consideration to the basic fundamentals.";
        let result = machine.compress_text(verbose, Some(0.5), None);
        assert!(
            result.target_achieved,
            "expected target_achieved=true with target=0.5, got compressed Cs={:.4}",
            result.compressed_score.score
        );

        // With an unreachably high target (50.0), target_achieved must be false.
        let result_high = machine.compress_text(verbose, Some(50.0), None);
        assert!(
            !result_high.target_achieved,
            "expected target_achieved=false with target=50.0, got Cs={:.4}",
            result_high.compressed_score.score
        );
    }

    #[test]
    fn test_readability_penalty() {
        let machine = CompendiousMachine::new();

        // Short sentence (5 words) should get readability=1.0 (no penalty).
        let short = "Execute test now.";
        let r_short = machine.score_text(short, None).readability;
        assert!(
            (r_short - 1.0).abs() < f64::EPSILON,
            "short sentence should have R=1.0, got {:.4}",
            r_short
        );

        // Very long single sentence (>>20 words) should get R < 1.0.
        let long = "The system will attempt to perform an analysis of all the various underlying data structures and components that are present within the broader architectural context of the application framework.";
        let r_long = machine.score_text(long, None).readability;
        assert!(
            r_long < 1.0,
            "long sentence should have R < 1.0, got {:.4}",
            r_long
        );
    }
}
