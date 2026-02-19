//! # Algorithm Block Extraction and Analysis
//!
//! Parse algorithm blocks from SKILL.md markdown and analyze determinism.
//! Translated from Python `autonomous-skill-runtime/src/skill_parser.py`.
//!
//! ## Features
//! - Extract code blocks from markdown
//! - Classify nodes as deterministic vs ambiguous
//! - Calculate coverage (deterministic ratio)
//! - Extract hook annotations

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

// ═══════════════════════════════════════════════════════════════════════════
// NODE TYPES (Tier: T2-P)
// ═══════════════════════════════════════════════════════════════════════════

/// Types of decision tree nodes in algorithm pseudocode.
///
/// Tier: T2-P (Cross-domain primitive - reusable across skill domains)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Function definition (`def foo():`)
    Function = 0,
    /// Conditional (`if`, `elif`, `else`)
    Condition = 1,
    /// Loop (`for`, `while`)
    Loop = 2,
    /// Variable assignment (`x = ...`)
    Assignment = 3,
    /// Return statement
    Return = 4,
    /// Function call (`foo()`)
    Call = 5,
    /// Shell command hook (`@hook:`)
    Hook = 6,
    /// Explicit LLM generation marker
    LlmGenerate = 7,
    /// Ambiguous - requires LLM interpretation
    Ambiguous = 8,
}

impl NodeType {
    /// Check if this node type is inherently deterministic.
    #[must_use]
    pub const fn is_deterministic_type(self) -> bool {
        matches!(
            self,
            Self::Function
                | Self::Condition
                | Self::Loop
                | Self::Assignment
                | Self::Return
                | Self::Call
                | Self::Hook
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK (Tier: T2-C)
// ═══════════════════════════════════════════════════════════════════════════

/// A shell command hook for deterministic execution.
///
/// Tier: T2-C (Cross-domain composite)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    /// Shell command to execute
    pub command: String,
    /// Line number in source
    pub line_number: usize,
    /// Execution phase (if annotated)
    pub phase: Option<String>,
    /// Output variable (if annotated)
    pub output_var: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// DECISION NODE (Tier: T2-C)
// ═══════════════════════════════════════════════════════════════════════════

/// A node in the algorithm decision tree.
///
/// Tier: T2-C (Cross-domain composite)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmNode {
    /// Unique node ID
    pub id: String,
    /// Node classification
    pub node_type: NodeType,
    /// Source content (the line of code)
    pub content: String,
    /// Line number in source
    pub line_number: usize,
    /// Whether this node can be executed deterministically
    pub is_deterministic: bool,
    /// Reason for ambiguity (if not deterministic)
    pub ambiguity_reason: Option<String>,
    /// Child nodes (for control flow)
    pub children: Vec<AlgorithmNode>,
}

// ═══════════════════════════════════════════════════════════════════════════
// ALGORITHM BLOCK (Tier: T2-C)
// ═══════════════════════════════════════════════════════════════════════════

/// A code block containing algorithm pseudocode.
///
/// Tier: T2-C (Cross-domain composite)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgorithmBlock {
    /// Function/block name
    pub name: String,
    /// Language (python, pseudocode, etc.)
    pub language: String,
    /// Raw content
    pub content: String,
    /// Start line in source
    pub start_line: usize,
    /// End line in source
    pub end_line: usize,
    /// Parsed nodes
    pub nodes: Vec<AlgorithmNode>,
    /// Extracted hooks
    pub hooks: Vec<Hook>,
}

impl AlgorithmBlock {
    /// Count total nodes recursively.
    #[must_use]
    pub fn node_count(&self) -> usize {
        fn count_recursive(nodes: &[AlgorithmNode]) -> usize {
            nodes.iter().map(|n| 1 + count_recursive(&n.children)).sum()
        }
        count_recursive(&self.nodes)
    }

    /// Count deterministic nodes recursively.
    #[must_use]
    pub fn deterministic_count(&self) -> usize {
        fn count_recursive(nodes: &[AlgorithmNode]) -> usize {
            nodes
                .iter()
                .map(|n| {
                    let self_count = usize::from(n.is_deterministic);
                    self_count + count_recursive(&n.children)
                })
                .sum()
        }
        count_recursive(&self.nodes)
    }

    /// Calculate determinism coverage (0.0 - 1.0).
    #[must_use]
    pub fn coverage(&self) -> f64 {
        let total = self.node_count();
        if total == 0 {
            return 0.0;
        }
        self.deterministic_count() as f64 / total as f64
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COVERAGE STATS (Tier: T3)
// ═══════════════════════════════════════════════════════════════════════════

/// Coverage statistics for a skill's algorithm blocks.
///
/// Tier: T3 (Domain-specific - skill analysis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageStats {
    /// Total algorithm blocks
    pub block_count: usize,
    /// Total nodes across all blocks
    pub total_nodes: usize,
    /// Deterministic nodes
    pub deterministic_nodes: usize,
    /// Total hooks
    pub hook_count: usize,
    /// Coverage ratio (0.0 - 1.0)
    pub coverage: f64,
}

// ═══════════════════════════════════════════════════════════════════════════
// PATTERNS (compiled once, infallible via Option)
// ═══════════════════════════════════════════════════════════════════════════

/// Pattern cache for algorithm parsing.
struct Patterns {
    function: Option<Regex>,
    condition: Option<Regex>,
    loop_: Option<Regex>,
    return_: Option<Regex>,
    assignment: Option<Regex>,
    call: Option<Regex>,
    hook: Option<Regex>,
    llm_generate: Option<Regex>,
    phase: Option<Regex>,
    ambiguity: Vec<(Regex, &'static str)>,
}

impl Patterns {
    fn new() -> Self {
        Self {
            function: Regex::new(r"^\s*def\s+(\w+)\s*\(").ok(),
            condition: Regex::new(r"(?i)^\s*(if|elif|else)\b").ok(),
            loop_: Regex::new(r"(?i)^\s*(for|while)\b").ok(),
            return_: Regex::new(r"^\s*return\b").ok(),
            assignment: Regex::new(r"^\s*(\w+(?:\.\w+)*)\s*[=←]").ok(),
            call: Regex::new(r"^\s*(\w+(?:\.\w+)*)\s*\(").ok(),
            hook: Regex::new(r"#\s*@hook:\s*(.+)$").ok(),
            llm_generate: Regex::new(r"\[LLM_GENERATE\]:").ok(),
            phase: Regex::new(r"(?i)#\s*PHASE\s*\d+[:\s]+([A-Z_]+)").ok(),
            ambiguity: [
                (r"(?i)\binfer\b", "Requires inference"),
                (r"(?i)\bsensible defaults?\b", "Relies on defaults"),
                (r"(?i)\bask\b.*\bquestion", "Requires user interaction"),
                (r"(?i)\bclaude\b", "Explicit LLM reference"),
                (r"(?i)\bgenerate\b", "Generative operation"),
                (r"(?i)\banalyze\b", "Requires analysis"),
                (r"(?i)\bdetermine\b", "Requires determination"),
                (r"(?i)\bextract\b.*\bfrom\b", "Extraction from unstructured"),
            ]
            .into_iter()
            .filter_map(|(pat, reason)| Regex::new(pat).ok().map(|r| (r, reason)))
            .collect(),
        }
    }

    fn matches_function(&self, line: &str) -> bool {
        self.function.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn capture_function_name(&self, line: &str) -> Option<String> {
        self.function
            .as_ref()
            .and_then(|r| r.captures(line))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn matches_condition(&self, line: &str) -> bool {
        self.condition.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn matches_loop(&self, line: &str) -> bool {
        self.loop_.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn matches_return(&self, line: &str) -> bool {
        self.return_.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn matches_assignment(&self, line: &str) -> bool {
        self.assignment.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn matches_call(&self, line: &str) -> bool {
        self.call.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn matches_hook(&self, line: &str) -> bool {
        self.hook.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn capture_hook_command(&self, line: &str) -> Option<String> {
        self.hook
            .as_ref()
            .and_then(|r| r.captures(line))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    fn matches_llm_generate(&self, line: &str) -> bool {
        self.llm_generate.as_ref().is_some_and(|r| r.is_match(line))
    }

    fn capture_phase(&self, line: &str) -> Option<String> {
        self.phase
            .as_ref()
            .and_then(|r| r.captures(line))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_uppercase())
    }

    fn check_ambiguity(&self, line: &str) -> Option<&'static str> {
        for (pattern, reason) in &self.ambiguity {
            if pattern.is_match(line) {
                return Some(*reason);
            }
        }
        None
    }
}

static PATTERNS: OnceLock<Patterns> = OnceLock::new();

fn patterns() -> &'static Patterns {
    PATTERNS.get_or_init(Patterns::new)
}

// ═══════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════

/// Algorithm block parser for SKILL.md files.
#[derive(Debug, Default)]
pub struct AlgorithmParser {
    node_counter: usize,
}

impl AlgorithmParser {
    /// Create a new parser.
    #[must_use]
    pub fn new() -> Self {
        Self { node_counter: 0 }
    }

    fn next_node_id(&mut self) -> String {
        self.node_counter += 1;
        format!("node_{}", self.node_counter)
    }

    /// Extract algorithm blocks from markdown content.
    #[must_use]
    pub fn extract_blocks(&mut self, content: &str) -> Vec<AlgorithmBlock> {
        let p = patterns();
        let mut blocks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        let mut in_block = false;
        let mut block_lang = String::new();
        let mut block_content: Vec<&str> = Vec::new();
        let mut block_start = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;

            if line.starts_with("```") {
                if !in_block {
                    in_block = true;
                    block_lang = line[3..].trim().to_string();
                    block_content.clear();
                    block_start = line_num;
                } else {
                    in_block = false;

                    if matches!(block_lang.as_str(), "python" | "pseudocode" | "") {
                        let full_content = block_content.join("\n");

                        if p.matches_function(&full_content) {
                            let name = p
                                .capture_function_name(&full_content)
                                .unwrap_or_else(|| "anonymous".to_string());

                            let nodes = self.build_decision_tree(&full_content, block_start);
                            let hooks = self.extract_hooks(&full_content, block_start);

                            blocks.push(AlgorithmBlock {
                                name,
                                language: if block_lang.is_empty() {
                                    "python".to_string()
                                } else {
                                    block_lang.clone()
                                },
                                content: full_content,
                                start_line: block_start,
                                end_line: line_num,
                                nodes,
                                hooks,
                            });
                        }
                    }
                }
            } else if in_block {
                block_content.push(line);
            }
        }

        blocks
    }

    /// Extract hooks from algorithm content.
    fn extract_hooks(&self, content: &str, base_line: usize) -> Vec<Hook> {
        let p = patterns();
        let mut hooks = Vec::new();
        let mut current_phase: Option<String> = None;

        for (i, line) in content.lines().enumerate() {
            let line_num = base_line + i;

            if let Some(phase) = p.capture_phase(line) {
                current_phase = Some(phase);
            }

            if let Some(command) = p.capture_hook_command(line) {
                hooks.push(Hook {
                    command,
                    line_number: line_num,
                    phase: current_phase.clone(),
                    output_var: None,
                });
            }
        }

        hooks
    }

    /// Build decision tree from algorithm content.
    fn build_decision_tree(&mut self, content: &str, base_line: usize) -> Vec<AlgorithmNode> {
        let mut root_nodes = Vec::new();

        for (i, line) in content.lines().enumerate() {
            let line_num = base_line + i;

            if let Some(node) = self.classify_line(line, line_num) {
                root_nodes.push(node);
            }
        }

        root_nodes
    }

    /// Classify a line of code into a decision node.
    fn classify_line(&mut self, line: &str, line_number: usize) -> Option<AlgorithmNode> {
        let p = patterns();
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("\"\"\"") {
            return None;
        }

        // Hook annotation
        if p.matches_hook(line) {
            let command = p
                .capture_hook_command(line)
                .unwrap_or_else(|| line.to_string());

            return Some(AlgorithmNode {
                id: self.next_node_id(),
                node_type: NodeType::Hook,
                content: command,
                line_number,
                is_deterministic: true,
                ambiguity_reason: None,
                children: Vec::new(),
            });
        }

        // Explicit LLM marker
        if p.matches_llm_generate(line) {
            return Some(AlgorithmNode {
                id: self.next_node_id(),
                node_type: NodeType::LlmGenerate,
                content: line.to_string(),
                line_number,
                is_deterministic: false,
                ambiguity_reason: Some("Explicit LLM generation marker".to_string()),
                children: Vec::new(),
            });
        }

        // Skip comments
        if trimmed.starts_with('#') {
            return None;
        }

        // Check ambiguity
        let ambiguity = p.check_ambiguity(line);
        let mut is_deterministic = ambiguity.is_none();
        let mut ambiguity_reason = ambiguity.map(String::from);

        // Classify node type
        let node_type = if p.matches_function(line) {
            NodeType::Function
        } else if p.matches_condition(line) {
            NodeType::Condition
        } else if p.matches_loop(line) {
            NodeType::Loop
        } else if p.matches_return(line) {
            NodeType::Return
        } else if p.matches_assignment(line) {
            NodeType::Assignment
        } else if p.matches_call(line) {
            NodeType::Call
        } else {
            NodeType::Ambiguous
        };

        if node_type == NodeType::Ambiguous {
            is_deterministic = false;
            if ambiguity_reason.is_none() {
                ambiguity_reason = Some("Unclassified statement".to_string());
            }
        }

        Some(AlgorithmNode {
            id: self.next_node_id(),
            node_type,
            content: line.to_string(),
            line_number,
            is_deterministic,
            ambiguity_reason,
            children: Vec::new(),
        })
    }

    /// Calculate coverage statistics for extracted blocks.
    #[must_use]
    pub fn calculate_coverage(blocks: &[AlgorithmBlock]) -> CoverageStats {
        let total_nodes: usize = blocks.iter().map(AlgorithmBlock::node_count).sum();
        let deterministic_nodes: usize =
            blocks.iter().map(AlgorithmBlock::deterministic_count).sum();
        let hook_count: usize = blocks.iter().map(|b| b.hooks.len()).sum();

        CoverageStats {
            block_count: blocks.len(),
            total_nodes,
            deterministic_nodes,
            hook_count,
            coverage: if total_nodes > 0 {
                deterministic_nodes as f64 / total_nodes as f64
            } else {
                0.0
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SKILL: &str = r#"
# My Skill

## Algorithm

```python
def process_input(data):
    # @hook: echo "Starting"
    if data.is_valid:
        result = transform(data)
        return result
    else:
        [LLM_GENERATE]: Infer the correct action
        return None
```
"#;

    #[test]
    fn test_extract_blocks() {
        let mut parser = AlgorithmParser::new();
        let blocks = parser.extract_blocks(SAMPLE_SKILL);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, "process_input");
        assert_eq!(blocks[0].language, "python");
    }

    #[test]
    fn test_hook_extraction() {
        let mut parser = AlgorithmParser::new();
        let blocks = parser.extract_blocks(SAMPLE_SKILL);

        assert_eq!(blocks[0].hooks.len(), 1);
        assert_eq!(blocks[0].hooks[0].command, "echo \"Starting\"");
    }

    #[test]
    fn test_node_classification() {
        let mut parser = AlgorithmParser::new();
        let blocks = parser.extract_blocks(SAMPLE_SKILL);

        let nodes = &blocks[0].nodes;
        assert!(nodes.iter().any(|n| n.node_type == NodeType::Function));
        assert!(nodes.iter().any(|n| n.node_type == NodeType::Hook));
        assert!(nodes.iter().any(|n| n.node_type == NodeType::Condition));
        assert!(nodes.iter().any(|n| n.node_type == NodeType::LlmGenerate));
    }

    #[test]
    fn test_coverage_calculation() {
        let mut parser = AlgorithmParser::new();
        let blocks = parser.extract_blocks(SAMPLE_SKILL);
        let stats = AlgorithmParser::calculate_coverage(&blocks);

        assert!(stats.total_nodes > 0);
        assert!(stats.deterministic_nodes > 0);
        assert!(stats.coverage > 0.0);
        assert!(stats.coverage < 1.0);
    }

    #[test]
    fn test_ambiguity_detection() {
        let p = patterns();

        let reason = p.check_ambiguity("Infer the best option");
        assert_eq!(reason, Some("Requires inference"));

        let reason = p.check_ambiguity("x = 5");
        assert!(reason.is_none());
    }
}
