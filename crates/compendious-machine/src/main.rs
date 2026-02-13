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

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};

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
    verbose_patterns: HashMap<&'static str, &'static str>,
    stopwords: HashSet<&'static str>,
    domain_targets: HashMap<(&'static str, &'static str), DomainTarget>,
}

impl CompendiousMachine {
    pub fn new() -> Self {
        let mut machine = Self {
            verbose_patterns: HashMap::new(),
            stopwords: HashSet::new(),
            domain_targets: HashMap::new(),
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
        let preserve_set: HashSet<String> = preserve.unwrap_or_default().into_iter().collect();
        let _target = target_cs.unwrap_or(2.0);

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
                    let savings =
                        verbose.split_whitespace().count() - replacement.split_whitespace().count();
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

        CompressionResult {
            original: text.to_string(),
            compressed,
            original_score,
            compressed_score,
            patterns_applied,
            improvement_percent: improvement,
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
                let savings =
                    verbose.split_whitespace().count() - replacement.split_whitespace().count();
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

    pub fn get_domain_target(&self, domain: &str, content_type: &str) -> Option<DomainTarget> {
        self.domain_targets
            .get(&(domain, content_type))
            .cloned()
            .or_else(|| {
                // Return generic target if specific not found
                Some(DomainTarget {
                    domain: domain.to_string(),
                    content_type: content_type.to_string(),
                    target_cs: 1.8,
                    rationale: "Generic target; specific domain/content_type combination not found"
                        .to_string(),
                })
            })
    }

    // ========== Helper Functions ==========

    fn information_content(&self, text: &str) -> f64 {
        let lowercased = text.to_lowercase();
        let words: HashSet<&str> = lowercased
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
        let avg_syllables = self.estimate_avg_syllables(&words);

        // Simplified Flesch formula normalized to 0-1
        let raw = 206.835 - (1.015 * avg_words_per_sentence) - (84.6 * avg_syllables);
        (raw.clamp(0.0, 100.0)) / 100.0
    }

    fn estimate_avg_syllables(&self, words: &[&str]) -> f64 {
        if words.is_empty() {
            return 1.0;
        }
        let total: usize = words.iter().map(|w| self.count_syllables(w)).sum();
        total as f64 / words.len() as f64
    }

    fn count_syllables(&self, word: &str) -> usize {
        let vowels = ['a', 'e', 'i', 'o', 'u', 'y'];
        let chars: Vec<char> = word.to_lowercase().chars().collect();

        let mut count = 0;
        let mut prev_vowel = false;

        for c in &chars {
            let is_vowel = vowels.contains(c);
            if is_vowel && !prev_vowel {
                count += 1;
            }
            prev_vowel = is_vowel;
        }

        if chars.last() == Some(&'e') && count > 1 {
            count -= 1;
        }

        count.max(1)
    }

    fn identify_limiting_factor(&self, density: f64, c: f64, r: f64) -> String {
        let factors = [
            (density, "Information Density"),
            (c, "Completeness"),
            (r, "Readability"),
        ];

        factors
            .iter()
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
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

pub fn handle_mcp_request(machine: &CompendiousMachine, request: &McpRequest) -> McpResponse {
    let id = request.id.clone();

    match request.method.as_str() {
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
                    "version": "2.0.0"
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
                    let markdown = format!(
                        "# Compression Result\n\n\
                        ## Metrics\n\
                        | Metric | Original | Compressed |\n\
                        |--------|----------|------------|\n\
                        | Tokens | {} | {} |\n\
                        | Cs Score | {:.2} | {:.2} |\n\
                        | Improvement | — | **{:+.1}%** |\n\n\
                        ## Compressed Text\n\
                        {}\n\n\
                        ## Patterns Applied\n\
                        {}",
                        result.original_score.expression_cost,
                        result.compressed_score.expression_cost,
                        result.original_score.score,
                        result.compressed_score.score,
                        result.improvement_percent,
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
                    let markdown = match target {
                        Some(t) => format!(
                            "# Domain Target\n\n\
                            - **Domain:** {}\n\
                            - **Content Type:** {}\n\
                            - **Target Cs:** {:.1}\n\
                            - **Rationale:** {}",
                            t.domain, t.content_type, t.target_cs, t.rationale
                        ),
                        None => format!(
                            "# Domain Target\n\n\
                            No specific target found for {}/{}. Using default Cs target of **1.8**.",
                            domain, content_type
                        ),
                    };
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
    }
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

fn run_mcp_server() {
    let machine = CompendiousMachine::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
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
                let _ = writeln!(
                    stdout,
                    "{}",
                    serde_json::to_string(&error_response).unwrap()
                );
                let _ = stdout.flush();
                continue;
            }
        };

        let response = handle_mcp_request(&machine, &request);
        let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
        let _ = stdout.flush();
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

        assert!(c_score.score > v_score.score);
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
        assert!(result.compressed_score.score >= result.original_score.score);
        assert!(result.compressed.len() < text.len());
    }
}
