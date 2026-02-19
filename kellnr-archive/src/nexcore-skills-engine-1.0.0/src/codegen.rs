//! # Code Generation Module
//!
//! Generates Rust validation rules, test scaffolds, and stubs from SMST.
//!
//! ## Features
//! - Parse INVARIANTS and FAILURE_MODES into validation rules
//! - Generate comprehensive test scaffolds (positive, negative, edge cases)
//! - Generate Rust module stubs with proper struct definitions
//! - Convert SMST into executable decision trees
//!
//! ## Performance Targets
//! - Parse to rules: < 1ms for typical SMST
//! - Generate stub: < 5ms

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Generated validation rule from SMST INVARIANTS section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule identifier (derived from skill name + index)
    pub id: String,
    /// Human-readable description of the rule
    pub description: String,
    /// Severity level: error, warning, info
    pub severity: String,
    /// The condition expression (extracted from INVARIANTS)
    pub condition: String,
    /// Error message when rule fails
    pub error_message: String,
}

/// Complete validation ruleset for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRuleset {
    /// Skill name
    pub skill_name: String,
    /// Generated rules from INVARIANTS
    pub invariant_rules: Vec<ValidationRule>,
    /// Generated rules from FAILURE_MODES
    pub failure_mode_rules: Vec<ValidationRule>,
    /// Input validation rules
    pub input_rules: Vec<ValidationRule>,
    /// Output validation rules
    pub output_rules: Vec<ValidationRule>,
    /// Total rule count
    pub total_rules: usize,
}

/// Test case generated from SMST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestCase {
    /// Test function name
    pub name: String,
    /// Test category: positive, negative, edge, stress, adversarial
    pub category: String,
    /// Test description
    pub description: String,
    /// Input values (as JSON-like representation)
    pub inputs: String,
    /// Expected output or behavior
    pub expected: String,
}

/// Test scaffold for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScaffold {
    /// Skill name
    pub skill_name: String,
    /// Module path for tests
    pub module_path: String,
    /// Generated test cases
    pub test_cases: Vec<GeneratedTestCase>,
    /// Rust test module code (ready to use)
    pub rust_code: String,
}

/// Rust code stub generated from SMST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustStub {
    /// Skill name
    pub skill_name: String,
    /// Module name (snake_case)
    pub module_name: String,
    /// Struct definitions
    pub structs: String,
    /// Function signatures
    pub functions: String,
    /// Complete Rust code
    pub full_code: String,
}

/// Generated code artifact (simple interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    /// Artifact type
    pub artifact_type: String,
    /// Generated content
    pub content: String,
    /// Target filename
    pub filename: String,
}

/// Compilation target for rules
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompilationTarget {
    /// Input validation
    Input,
    /// Output validation
    Output,
}

/// Schema for a struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructSchema {
    /// Struct name
    pub struct_name: String,
    /// Fields in the struct
    pub fields: Vec<FieldSchema>,
}

/// Schema for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field name
    pub field_name: String,
    /// Field type (Rust type)
    pub field_type: String,
}

/// SMST skill frontmatter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    /// Skill name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version
    pub version: Option<String>,
    /// Compliance level
    pub compliance_level: Option<String>,
    /// Categories
    pub categories: Option<Vec<String>>,
}

/// SMST machine specification sections
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillMachineSpec {
    /// INPUTS section
    pub inputs: Option<String>,
    /// OUTPUTS section
    pub outputs: Option<String>,
    /// STATE section
    pub state: Option<String>,
    /// OPERATOR MODE section
    pub operator_mode: Option<String>,
    /// PERFORMANCE section
    pub performance: Option<String>,
    /// INVARIANTS section
    pub invariants: Option<String>,
    /// FAILURE_MODES section
    pub failure_modes: Option<String>,
    /// TELEMETRY section
    pub telemetry: Option<String>,
}

/// SMST extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmstResult {
    /// Extracted frontmatter
    pub frontmatter: SkillFrontmatter,
    /// Extracted machine spec
    pub spec: SkillMachineSpec,
    /// Compliance score
    pub score: SmstScore,
    /// Whether skill is Diamond compliant
    pub is_diamond_compliant: bool,
}

/// SMST compliance score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmstScore {
    /// Total score (0-100)
    pub total_score: f32,
    /// Number of sections present
    pub sections_present: u32,
    /// Number of sections required
    pub sections_required: u32,
    /// Has valid frontmatter
    pub has_frontmatter: bool,
    /// Has machine spec header
    pub has_machine_spec: bool,
    /// Computed compliance level
    pub compliance_level: String,
    /// Missing sections
    pub missing_sections: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SMST EXTRACTION
// ═══════════════════════════════════════════════════════════════════════════════

/// Extract SMST from skill markdown content
#[must_use]
pub fn extract_smst(content: &str) -> SmstResult {
    let frontmatter = extract_frontmatter(content);
    let spec = extract_machine_spec(content);
    let score = calculate_smst_score(&frontmatter, &spec);
    let is_diamond_compliant = score.total_score >= 85.0;

    SmstResult {
        frontmatter,
        spec,
        score,
        is_diamond_compliant,
    }
}

/// Extract YAML frontmatter from skill content
fn extract_frontmatter(content: &str) -> SkillFrontmatter {
    // Find frontmatter between --- markers
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        return SkillFrontmatter::default();
    }

    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = Some(i);
            break;
        }
    }

    let Some(end) = end_idx else {
        return SkillFrontmatter::default();
    };

    let yaml_content = lines[1..end].join("\n");

    // Simple parsing (could use serde_yaml for full parsing)
    let mut frontmatter = SkillFrontmatter::default();

    for line in yaml_content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("name:") {
            frontmatter.name = value.trim().trim_matches('"').to_string();
        } else if let Some(value) = line.strip_prefix("description:") {
            frontmatter.description = Some(value.trim().trim_matches('"').to_string());
        } else if let Some(value) = line.strip_prefix("version:") {
            frontmatter.version = Some(value.trim().trim_matches('"').to_string());
        } else if let Some(value) = line.strip_prefix("compliance-level:") {
            frontmatter.compliance_level = Some(value.trim().to_string());
        }
    }

    frontmatter
}

/// Extract machine specification sections
fn extract_machine_spec(content: &str) -> SkillMachineSpec {
    SkillMachineSpec {
        inputs: extract_section(content, "INPUTS"),
        outputs: extract_section(content, "OUTPUTS"),
        state: extract_section(content, "STATE"),
        operator_mode: extract_section(content, "OPERATOR MODE"),
        performance: extract_section(content, "PERFORMANCE"),
        invariants: extract_section(content, "INVARIANTS"),
        failure_modes: extract_section(content, "FAILURE")
            .or_else(|| extract_section(content, "FAILURE_MODES")),
        telemetry: extract_section(content, "TELEMETRY"),
    }
}

/// Extract a specific section from markdown content
fn extract_section(content: &str, section_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_content = Vec::new();

    for line in &lines {
        // Check for section header (e.g., "### 1. INPUTS" or "## INPUTS")
        if line.contains(section_name) && (line.starts_with('#') || line.contains("###")) {
            in_section = true;
            continue;
        }

        // Check for next section header
        if in_section
            && (line.starts_with("### ") || line.starts_with("## "))
            && !line.contains(section_name)
        {
            break;
        }

        if in_section {
            section_content.push(*line);
        }
    }

    if section_content.is_empty() {
        None
    } else {
        Some(section_content.join("\n"))
    }
}

/// Calculate SMST compliance score
fn calculate_smst_score(frontmatter: &SkillFrontmatter, spec: &SkillMachineSpec) -> SmstScore {
    let required_sections = vec![
        ("inputs", spec.inputs.is_some()),
        ("outputs", spec.outputs.is_some()),
        ("performance", spec.performance.is_some()),
        ("invariants", spec.invariants.is_some()),
        ("failure_modes", spec.failure_modes.is_some()),
    ];

    let sections_present = required_sections
        .iter()
        .filter(|(_, present)| *present)
        .count() as u32;
    let sections_required = required_sections.len() as u32;

    let mut missing = Vec::new();
    for (name, present) in &required_sections {
        if !present {
            missing.push(name.to_string());
        }
    }

    let has_frontmatter = !frontmatter.name.is_empty();
    let has_machine_spec = sections_present > 0;

    let base_score = (sections_present as f32 / sections_required as f32) * 80.0;
    let frontmatter_bonus = if has_frontmatter { 20.0 } else { 0.0 };
    let total_score = base_score + frontmatter_bonus;

    let compliance_level = if total_score >= 85.0 {
        "diamond"
    } else if total_score >= 70.0 {
        "platinum"
    } else if total_score >= 50.0 {
        "gold"
    } else if total_score >= 30.0 {
        "silver"
    } else {
        "bronze"
    };

    SmstScore {
        total_score,
        sections_present,
        sections_required,
        has_frontmatter,
        has_machine_spec,
        compliance_level: compliance_level.to_string(),
        missing_sections: missing,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VALIDATION RULES GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate validation rules from SMST
#[must_use]
pub fn generate_validation_rules(smst: &SmstResult) -> ValidationRuleset {
    let skill_name = &smst.frontmatter.name;
    let mut invariant_rules = Vec::new();
    let mut failure_mode_rules = Vec::new();
    let mut input_rules = Vec::new();
    let mut output_rules = Vec::new();

    if let Some(invariants) = &smst.spec.invariants {
        invariant_rules = parse_invariants_to_rules(skill_name, invariants);
    }

    if let Some(failure_modes) = &smst.spec.failure_modes {
        failure_mode_rules = parse_failure_modes_to_rules(skill_name, failure_modes);
    }

    if let Some(inputs) = &smst.spec.inputs {
        input_rules = parse_inputs_to_rules(skill_name, inputs);
    }

    if let Some(outputs) = &smst.spec.outputs {
        output_rules = parse_outputs_to_rules(skill_name, outputs);
    }

    let total_rules =
        invariant_rules.len() + failure_mode_rules.len() + input_rules.len() + output_rules.len();

    ValidationRuleset {
        skill_name: skill_name.clone(),
        invariant_rules,
        failure_mode_rules,
        input_rules,
        output_rules,
        total_rules,
    }
}

/// Parse INVARIANTS section into validation rules
fn parse_invariants_to_rules(skill_name: &str, invariants: &str) -> Vec<ValidationRule> {
    let mut rules = Vec::new();
    let mut current_column_map: HashMap<String, usize> = HashMap::new();

    for (idx, line) in invariants.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Table logic
        if line.starts_with('|') {
            if line.contains("---") {
                continue;
            }

            let parts: Vec<String> = line
                .trim_matches('|')
                .split('|')
                .map(|s| s.trim().to_lowercase())
                .collect();

            // Is this a header row?
            if parts
                .iter()
                .any(|p| p == "condition" || p == "type" || p == "invariant")
            {
                current_column_map.clear();
                for (i, p) in parts.iter().enumerate() {
                    current_column_map.insert(p.clone(), i);
                }
                continue;
            }

            let content = if !current_column_map.is_empty() {
                let actual_parts: Vec<&str> = line.trim_matches('|').split('|').collect();
                let col_idx = current_column_map
                    .get("condition")
                    .or_else(|| current_column_map.get("invariant"))
                    .or_else(|| current_column_map.get("type"))
                    .cloned()
                    .unwrap_or(0);

                actual_parts.get(col_idx).map(|s| s.trim()).unwrap_or("")
            } else {
                line.trim_matches('|')
                    .split('|')
                    .next()
                    .unwrap_or("")
                    .trim()
            };

            if content.is_empty()
                || content.to_lowercase() == "condition"
                || content.to_lowercase() == "invariant"
            {
                continue;
            }

            let (condition, error_message) = extract_invariant_pattern(content);
            rules.push(ValidationRule {
                id: format!("{}_inv_{}", to_snake_case(skill_name), idx),
                description: content.to_string(),
                severity: "error".to_string(),
                condition,
                error_message,
            });
            continue;
        }

        // Extract bullet points
        let content = if line.starts_with('-') || line.starts_with('*') {
            line.trim_start_matches(['-', '*']).trim()
        } else if line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            line.split_once('.')
                .map(|(_, rest)| rest.trim())
                .unwrap_or(line)
        } else {
            continue;
        };

        if content.is_empty() {
            continue;
        }

        let (condition, error_message) = extract_invariant_pattern(content);

        rules.push(ValidationRule {
            id: format!("{}_inv_{}", to_snake_case(skill_name), idx),
            description: content.to_string(),
            severity: "error".to_string(),
            condition,
            error_message,
        });
    }

    rules
}

/// Extract condition and error message from invariant text
fn extract_invariant_pattern(text: &str) -> (String, String) {
    let text_lower = text.to_lowercase();

    if text_lower.contains("must") {
        let condition = text.replace("must", "should").replace("Must", "Should");
        let error = format!("Invariant violation: {text}");
        (condition, error)
    } else if text_lower.contains("always") {
        let condition = text.to_string();
        let error = format!("Always condition failed: {text}");
        (condition, error)
    } else if text_lower.contains("never") {
        let condition = text
            .replace("never", "should not")
            .replace("Never", "Should not");
        let error = format!("Never condition violated: {text}");
        (condition, error)
    } else if text_lower.contains("required") {
        let condition = text.to_string();
        let error = format!("Required condition not met: {text}");
        (condition, error)
    } else {
        (text.to_string(), format!("Validation failed: {text}"))
    }
}

/// Parse FAILURE_MODES section into validation rules
fn parse_failure_modes_to_rules(skill_name: &str, failure_modes: &str) -> Vec<ValidationRule> {
    let mut rules = Vec::new();
    let mut current_column_map: HashMap<String, usize> = HashMap::new();

    for (idx, line) in failure_modes.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Table logic
        if line.starts_with('|') {
            if line.contains("---") {
                continue;
            }

            let parts: Vec<String> = line
                .trim_matches('|')
                .split('|')
                .map(|s| s.trim().to_lowercase())
                .collect();

            if parts
                .iter()
                .any(|p| p == "trigger" || p == "response" || p == "mode")
            {
                current_column_map.clear();
                for (i, p) in parts.iter().enumerate() {
                    current_column_map.insert(p.clone(), i);
                }
                continue;
            }

            let content = if !current_column_map.is_empty() {
                let actual_parts: Vec<&str> = line.trim_matches('|').split('|').collect();
                let col_idx = current_column_map
                    .get("response")
                    .or_else(|| current_column_map.get("trigger"))
                    .or_else(|| current_column_map.get("mode"))
                    .cloned()
                    .unwrap_or(0);

                actual_parts.get(col_idx).map(|s| s.trim()).unwrap_or("")
            } else {
                line.trim_matches('|')
                    .split('|')
                    .next_back()
                    .unwrap_or("")
                    .trim()
            };

            if content.is_empty()
                || content.to_lowercase() == "response"
                || content.to_lowercase() == "trigger"
                || content.to_lowercase() == "mode"
            {
                continue;
            }

            let (severity, error_message) = parse_failure_mode_severity(content);
            rules.push(ValidationRule {
                id: format!("{}_fm_{}", to_snake_case(skill_name), idx),
                description: content.to_string(),
                severity,
                condition: format!("check_{}_fm_{}", to_snake_case(skill_name), idx),
                error_message,
            });
            continue;
        }

        // Look for error code patterns like FM-001
        let content = if line.starts_with('-') || line.starts_with('*') {
            line.trim_start_matches(['-', '*']).trim()
        } else if line.contains(':') || line.contains("FM-") || line.contains("ERR_") {
            line
        } else {
            continue;
        };

        if content.is_empty() {
            continue;
        }

        let (severity, error_message) = parse_failure_mode_severity(content);

        rules.push(ValidationRule {
            id: format!("{}_fm_{}", to_snake_case(skill_name), idx),
            description: content.to_string(),
            severity,
            condition: format!("check_{}_fm_{}", to_snake_case(skill_name), idx),
            error_message,
        });
    }

    rules
}

/// Determine severity from failure mode text
fn parse_failure_mode_severity(text: &str) -> (String, String) {
    let text_lower = text.to_lowercase();

    let severity = if text_lower.contains("critical") || text_lower.contains("fatal") {
        "error"
    } else if text_lower.contains("warning") || text_lower.contains("recoverable") {
        "warning"
    } else if text_lower.contains("info") || text_lower.contains("note") {
        "info"
    } else {
        "error"
    };

    (severity.to_string(), text.to_string())
}

/// Parse INPUTS section into validation rules
fn parse_inputs_to_rules(skill_name: &str, inputs: &str) -> Vec<ValidationRule> {
    let mut rules = Vec::new();

    for (idx, line) in inputs.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let content = if line.starts_with('-') || line.starts_with('*') {
            line.trim_start_matches(['-', '*']).trim()
        } else if line.starts_with('|') {
            if line.contains("---") {
                continue;
            }
            line.trim_matches('|').trim()
        } else {
            continue;
        };

        if content.is_empty() {
            continue;
        }

        let (param_name, param_type) = extract_param_info(content);

        if !param_name.is_empty() {
            rules.push(ValidationRule {
                id: format!("{}_input_{}", to_snake_case(skill_name), idx),
                description: format!("Validate input: {content}"),
                severity: "error".to_string(),
                condition: format!("validate_{}({})", param_name, param_type),
                error_message: format!("Invalid input {param_name}: expected {param_type}"),
            });
        }
    }

    rules
}

/// Extract parameter name and type from input definition
fn extract_param_info(text: &str) -> (String, String) {
    // Pattern: `name` (type) or name: type
    if let Some(start) = text.find('`') {
        if let Some(end) = text[start + 1..].find('`') {
            let name = &text[start + 1..start + 1 + end];
            let type_str = if let Some(paren_start) = text.find('(') {
                if let Some(paren_end) = text.find(')') {
                    text[paren_start + 1..paren_end].to_string()
                } else {
                    "any".to_string()
                }
            } else {
                "any".to_string()
            };
            return (name.to_string(), type_str);
        }
    }

    // Try colon pattern
    if text.contains(':') {
        let parts: Vec<&str> = text.splitn(2, ':').collect();
        if parts.len() == 2 {
            let name = parts[0]
                .trim()
                .trim_matches(|c| c == '`' || c == '*' || c == '_');
            let type_part = parts[1].split_whitespace().next().unwrap_or("any");
            return (name.to_string(), type_part.to_string());
        }
    }

    (String::new(), String::new())
}

/// Parse OUTPUTS section into validation rules
fn parse_outputs_to_rules(skill_name: &str, outputs: &str) -> Vec<ValidationRule> {
    let mut rules = Vec::new();

    for (idx, line) in outputs.lines().enumerate() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let content = if line.starts_with('-') || line.starts_with('*') {
            line.trim_start_matches(['-', '*']).trim()
        } else {
            continue;
        };

        if content.is_empty() {
            continue;
        }

        let (output_name, output_type) = extract_param_info(content);

        if !output_name.is_empty() {
            rules.push(ValidationRule {
                id: format!("{}_output_{}", to_snake_case(skill_name), idx),
                description: format!("Validate output: {content}"),
                severity: "error".to_string(),
                condition: format!("validate_output_{}({})", output_name, output_type),
                error_message: format!("Invalid output {output_name}: expected {output_type}"),
            });
        }
    }

    rules
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST SCAFFOLD GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate test scaffold from SMST
#[must_use]
pub fn generate_test_scaffold(smst: &SmstResult) -> TestScaffold {
    let skill_name = &smst.frontmatter.name;
    let module_name = to_snake_case(skill_name);
    let mut test_cases = Vec::new();

    if let Some(inputs) = &smst.spec.inputs {
        test_cases.extend(generate_positive_tests(skill_name, inputs));
    }

    if let Some(failure_modes) = &smst.spec.failure_modes {
        test_cases.extend(generate_negative_tests(skill_name, failure_modes));
    }

    if let Some(invariants) = &smst.spec.invariants {
        test_cases.extend(generate_edge_tests(skill_name, invariants));
    }

    let rust_code = generate_rust_test_module(&module_name, &test_cases);

    TestScaffold {
        skill_name: skill_name.clone(),
        module_path: format!("tests::{module_name}"),
        test_cases,
        rust_code,
    }
}

fn generate_positive_tests(skill_name: &str, inputs: &str) -> Vec<GeneratedTestCase> {
    let snake_name = to_snake_case(skill_name);
    vec![GeneratedTestCase {
        name: format!("test_{snake_name}_happy_path"),
        category: "positive".to_string(),
        description: format!("Test {skill_name} with valid inputs"),
        inputs: extract_sample_inputs(inputs),
        expected: "Ok(...)".to_string(),
    }]
}

fn generate_negative_tests(skill_name: &str, failure_modes: &str) -> Vec<GeneratedTestCase> {
    let mut tests = Vec::new();
    let snake_name = to_snake_case(skill_name);

    for (idx, line) in failure_modes.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let content = line.trim_start_matches(['-', '*']).trim();
        if content.is_empty() {
            continue;
        }

        tests.push(GeneratedTestCase {
            name: format!("test_{snake_name}_failure_{idx}"),
            category: "negative".to_string(),
            description: format!("Test {skill_name} handles: {}", truncate(content, 50)),
            inputs: "/* trigger failure condition */".to_string(),
            expected: format!("Err(...) // {}", truncate(content, 30)),
        });
    }

    tests
}

fn generate_edge_tests(skill_name: &str, invariants: &str) -> Vec<GeneratedTestCase> {
    let mut tests = Vec::new();
    let snake_name = to_snake_case(skill_name);

    for (idx, line) in invariants.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let content = line.trim_start_matches(['-', '*']).trim();
        if content.is_empty() {
            continue;
        }

        tests.push(GeneratedTestCase {
            name: format!("test_{snake_name}_edge_{idx}"),
            category: "edge".to_string(),
            description: format!("Edge case for invariant: {}", truncate(content, 50)),
            inputs: "/* boundary condition */".to_string(),
            expected: "/* invariant maintained */".to_string(),
        });
    }

    tests
}

fn extract_sample_inputs(inputs: &str) -> String {
    let mut params = Vec::new();

    for line in inputs.lines() {
        let line = line.trim();
        let (name, type_str) = extract_param_info(line);
        if !name.is_empty() {
            params.push(format!("{name}: /* {type_str} */"));
        }
    }

    if params.is_empty() {
        "/* input */".to_string()
    } else {
        params.join(", ")
    }
}

fn generate_rust_test_module(module_name: &str, test_cases: &[GeneratedTestCase]) -> String {
    let mut code = String::new();

    code.push_str(&format!("// Generated tests for {module_name}\n"));
    code.push_str("// Auto-generated by nexcore\n\n");
    code.push_str("#[cfg(test)]\n");
    code.push_str(&format!("mod {module_name}_tests {{\n"));
    code.push_str("    use super::*;\n\n");

    for test in test_cases {
        code.push_str(&format!("    /// {}\n", test.description));
        code.push_str(&format!("    /// Category: {}\n", test.category));
        code.push_str("    #[test]\n");
        code.push_str(&format!("    fn {}() {{\n", test.name));
        code.push_str(&format!("        // Inputs: {}\n", test.inputs));
        code.push_str(&format!("        // Expected: {}\n", test.expected));
        code.push_str("        assert!(true, \"Placeholder test - implement actual logic\");\n");
        code.push_str("    }\n\n");
    }

    code.push_str("}\n");
    code
}

// ═══════════════════════════════════════════════════════════════════════════════
// RUST STUB GENERATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate Rust code stub from SMST
#[must_use]
pub fn generate_rust_stub(smst: &SmstResult) -> RustStub {
    let skill_name = &smst.frontmatter.name;
    let module_name = to_snake_case(skill_name);

    let structs = generate_struct_definitions(smst);
    let functions = generate_function_signatures(smst);
    let full_code = generate_full_rust_module(smst);

    RustStub {
        skill_name: skill_name.clone(),
        module_name,
        structs,
        functions,
        full_code,
    }
}

fn generate_struct_definitions(smst: &SmstResult) -> String {
    let mut code = String::new();
    let skill_name = &smst.frontmatter.name;
    let type_name = to_pascal_case(skill_name);

    // Input struct
    code.push_str(&format!("/// Input for {skill_name}\n"));
    code.push_str("#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n");
    code.push_str(&format!("pub struct {type_name}Input {{\n"));
    if let Some(inputs) = &smst.spec.inputs {
        for line in inputs.lines() {
            let (name, type_str) = extract_param_info(line.trim());
            if !name.is_empty() {
                code.push_str(&format!(
                    "    pub {}: {},\n",
                    to_snake_case(&name),
                    rust_type_from_str(&type_str)
                ));
            }
        }
    }
    code.push_str("}\n\n");

    // Output struct
    code.push_str(&format!("/// Output for {skill_name}\n"));
    code.push_str("#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n");
    code.push_str(&format!("pub struct {type_name}Output {{\n"));
    if let Some(outputs) = &smst.spec.outputs {
        for line in outputs.lines() {
            let (name, type_str) = extract_param_info(line.trim());
            if !name.is_empty() {
                code.push_str(&format!(
                    "    pub {}: {},\n",
                    to_snake_case(&name),
                    rust_type_from_str(&type_str)
                ));
            }
        }
    }
    code.push_str("}\n\n");

    // State struct (if present)
    if let Some(state) = &smst.spec.state {
        if !state.trim().is_empty() {
            code.push_str(&format!("/// State for {skill_name}\n"));
            code.push_str("#[derive(Debug, Clone, Default, Serialize, Deserialize)]\n");
            code.push_str(&format!("pub struct {type_name}State {{\n"));
            for line in state.lines() {
                let (name, type_str) = extract_param_info(line.trim());
                if !name.is_empty() {
                    code.push_str(&format!(
                        "    pub {}: {},\n",
                        to_snake_case(&name),
                        rust_type_from_str(&type_str)
                    ));
                }
            }
            code.push_str("}\n\n");
        }
    }

    code
}

fn generate_function_signatures(smst: &SmstResult) -> String {
    let mut code = String::new();
    let skill_name = &smst.frontmatter.name;
    let type_name = to_pascal_case(skill_name);
    let fn_name = to_snake_case(skill_name);

    code.push_str(&format!("/// Execute {skill_name}\n"));
    if let Some(desc) = &smst.frontmatter.description {
        code.push_str(&format!("///\n/// {desc}\n"));
    }
    code.push_str(&format!(
        "pub fn {fn_name}(input: {type_name}Input) -> Result<{type_name}Output, {type_name}Error> {{\n"
    ));
    code.push_str("    todo!(\"Implement skill logic\")\n");
    code.push_str("}\n");

    code
}

fn generate_full_rust_module(smst: &SmstResult) -> String {
    let mut code = String::new();
    let skill_name = &smst.frontmatter.name;
    let type_name = to_pascal_case(skill_name);

    // Module header
    code.push_str(&format!("//! {skill_name} - Auto-generated Rust module\n"));
    if let Some(desc) = &smst.frontmatter.description {
        code.push_str(&format!("//!\n//! {desc}\n"));
    }
    code.push_str("//!\n//! Generated by: nexcore\n\n");

    code.push_str("use serde::{Deserialize, Serialize};\n");
    code.push_str("use thiserror::Error;\n\n");

    // Error type
    code.push_str(&format!("/// Errors for {skill_name}\n"));
    code.push_str("#[derive(Debug, Error)]\n");
    code.push_str(&format!("pub enum {type_name}Error {{\n"));
    if let Some(failure_modes) = &smst.spec.failure_modes {
        for (idx, line) in failure_modes.lines().enumerate() {
            let line = line.trim().trim_start_matches(['-', '*']).trim();
            if !line.is_empty() && !line.starts_with('#') {
                code.push_str(&format!("    #[error(\"{}\")]\n", truncate(line, 60)));
                code.push_str(&format!("    Failure{idx},\n"));
            }
        }
    }
    code.push_str("    #[error(\"Unknown error: {0}\")]\n");
    code.push_str("    Unknown(String),\n");
    code.push_str("}\n\n");

    // Structs
    code.push_str(&generate_struct_definitions(smst));

    // Functions
    code.push_str(&generate_function_signatures(smst));

    code
}

// ═══════════════════════════════════════════════════════════════════════════════
// SIMPLE API (Backward Compatible)
// ═══════════════════════════════════════════════════════════════════════════════

/// Generate validation rules from skill name and SMST content (simple API)
#[must_use]
pub fn generate_rules(skill_name: &str, smst_content: &str) -> GeneratedCode {
    let smst = extract_smst(smst_content);
    let rules = generate_validation_rules(&smst);

    let mut content = format!("//! Validation rules for {skill_name}\n");
    content.push_str("//! Auto-generated by nexcore\n\n");
    content.push_str(&format!("// Total rules: {}\n\n", rules.total_rules));

    for rule in &rules.invariant_rules {
        content.push_str(&format!("// {}: {}\n", rule.id, rule.description));
    }

    content.push_str("\n/// Validate input\n");
    content.push_str("pub fn validate_input(input: &serde_json::Value) -> Result<(), String> {\n");
    for rule in &rules.input_rules {
        content.push_str(&format!("    // {}\n", rule.condition));
    }
    content.push_str("    Ok(())\n}\n\n");

    content.push_str("/// Validate output\n");
    content
        .push_str("pub fn validate_output(output: &serde_json::Value) -> Result<(), String> {\n");
    for rule in &rules.output_rules {
        content.push_str(&format!("    // {}\n", rule.condition));
    }
    content.push_str("    Ok(())\n}\n");

    GeneratedCode {
        artifact_type: "rules".to_string(),
        content,
        filename: format!("{}_rules.rs", to_snake_case(skill_name)),
    }
}

/// Generate test scaffold from skill name and SMST content (simple API)
#[must_use]
pub fn generate_tests(skill_name: &str, smst_content: &str) -> GeneratedCode {
    let smst = extract_smst(smst_content);
    let scaffold = generate_test_scaffold(&smst);

    GeneratedCode {
        artifact_type: "tests".to_string(),
        content: scaffold.rust_code,
        filename: format!("{}_test.rs", to_snake_case(skill_name)),
    }
}

/// Generate Rust module stub from skill name and SMST content (simple API)
#[must_use]
pub fn generate_stub(skill_name: &str, smst_content: &str) -> GeneratedCode {
    let smst = extract_smst(smst_content);
    let stub = generate_rust_stub(&smst);

    GeneratedCode {
        artifact_type: "stub".to_string(),
        content: stub.full_code,
        filename: format!("{}.rs", to_snake_case(skill_name)),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert string to snake_case
#[must_use]
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        if c == '-' {
            result.push('_');
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}

/// Convert string to PascalCase
#[must_use]
pub fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// Map type strings to Rust types
fn rust_type_from_str(type_str: &str) -> &'static str {
    match type_str.to_lowercase().as_str() {
        "string" | "str" | "text" => "String",
        "int" | "integer" | "i32" => "i32",
        "i64" | "long" => "i64",
        "float" | "f32" => "f32",
        "f64" | "double" | "number" => "f64",
        "bool" | "boolean" => "bool",
        "path" | "filepath" => "std::path::PathBuf",
        "json" | "object" => "serde_json::Value",
        "array" | "list" | "vec" => "Vec<serde_json::Value>",
        "optional" | "option" => "Option<String>",
        _ => "String",
    }
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SKILL: &str = r#"---
name: test-skill
description: A test skill for code generation
version: 1.0.0
compliance-level: diamond
---

# test-skill

## Machine Specification

### 1. INPUTS

- `path` (String): Path to input file
- `threshold` (i32): Score threshold

### 2. OUTPUTS

- `result` (String): Processing result
- `score` (f64): Calculated score

### 3. STATE

- `cache` (Object): Internal cache

### 5. PERFORMANCE

- Latency: <50ms p95

### 6. INVARIANTS

- Score must be between 0 and 100
- Path must exist and be readable

### 7. FAILURE MODES

- FM-001: File not found (critical)
- FM-002: Invalid JSON (recoverable)
"#;

    #[test]
    fn test_extract_smst() {
        let smst = extract_smst(SAMPLE_SKILL);
        assert_eq!(smst.frontmatter.name, "test-skill");
        assert!(smst.spec.inputs.is_some());
        assert!(smst.spec.outputs.is_some());
        assert!(smst.spec.invariants.is_some());
    }

    #[test]
    fn test_generate_validation_rules() {
        let smst = extract_smst(SAMPLE_SKILL);
        let rules = generate_validation_rules(&smst);

        assert_eq!(rules.skill_name, "test-skill");
        assert!(!rules.invariant_rules.is_empty());
        assert!(!rules.failure_mode_rules.is_empty());
        assert!(rules.total_rules > 0);
    }

    #[test]
    fn test_generate_test_scaffold() {
        let smst = extract_smst(SAMPLE_SKILL);
        let scaffold = generate_test_scaffold(&smst);

        assert_eq!(scaffold.skill_name, "test-skill");
        assert!(!scaffold.test_cases.is_empty());
        assert!(scaffold.rust_code.contains("#[test]"));
    }

    #[test]
    fn test_generate_rust_stub() {
        let smst = extract_smst(SAMPLE_SKILL);
        let stub = generate_rust_stub(&smst);

        assert_eq!(stub.skill_name, "test-skill");
        assert_eq!(stub.module_name, "test_skill");
        assert!(stub.structs.contains("TestSkillInput"));
        assert!(stub.structs.contains("TestSkillOutput"));
        assert!(stub.functions.contains("pub fn test_skill"));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("test-skill"), "test_skill");
        assert_eq!(to_snake_case("TestSkill"), "test_skill");
        assert_eq!(to_snake_case("my-cool-skill"), "my_cool_skill");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("test-skill"), "TestSkill");
        assert_eq!(to_pascal_case("my_cool_skill"), "MyCoolSkill");
    }

    #[test]
    fn test_simple_api() {
        let rules = generate_rules("test-skill", SAMPLE_SKILL);
        assert_eq!(rules.artifact_type, "rules");
        assert!(rules.content.contains("test-skill"));

        let tests = generate_tests("test-skill", SAMPLE_SKILL);
        assert_eq!(tests.artifact_type, "tests");
        assert!(tests.content.contains("#[test]"));

        let stub = generate_stub("test-skill", SAMPLE_SKILL);
        assert_eq!(stub.artifact_type, "stub");
        assert!(stub.content.contains("TestSkill"));
    }
}
