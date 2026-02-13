//! Skill Primitive Grounding Enforcer - Atomic Hook
//!
//! PreToolUse:Write|Edit hook that enforces T1 Primitive Grounding tables in SKILL.md files.
//! Ensures all new skills are grounded to irreducible T1 primitives per the Lex Primitiva.
//!
//! # Codex Compliance
//! - **Tier**: T3 (Policy Hook)
//! - **Commandments**: VI (Match), VII (Type)
//!
//! # T1 Primitive Grounding
//!
//! | Concept | T1 Primitive | Lex Primitiva Symbol |
//! |:---|:---|:---:|
//! | **Content Inspection** | Existence | exists |
//! | **Pattern Match** | Comparison | kappa |
//! | **Section Detection** | Sequence | sigma |
//! | **Column Validation** | State | varsigma |
//! | **Block Decision** | Causality | arrow |

use nexcore_hook_lib::cytokine::emit_tool_blocked;
use nexcore_hook_lib::{
    block, content_or_pass, file_path_or_pass, pass, read_input, require_edit_tool,
};

const HOOK_NAME: &str = "skill-primitive-grounding-enforcer";

/// Required section header for T1 Primitive Grounding
const SECTION_HEADER: &str = "## T1 Primitive Grounding";

/// Required table columns (pipe-separated in markdown)
const REQUIRED_COLUMNS: &[&str] = &["Concept", "T1 Primitive", "Lex Primitiva Symbol"];

/// Table validation state (T1: varsigma)
#[derive(Default)]
struct TableState {
    found_header: bool,
    found_separator: bool,
    found_data_row: bool,
}

/// Check if file path is a SKILL.md file (T1: kappa)
fn is_skill_md(path: &str) -> bool {
    path.ends_with("SKILL.md") || path.ends_with("/SKILL.md")
}

/// Extract section content between header and next section (T1: sigma)
fn extract_section(content: &str) -> Option<&str> {
    let start = content.find(SECTION_HEADER)?;
    let section_content = &content[start..];
    let end = section_content[SECTION_HEADER.len()..]
        .find("\n## ")
        .map_or(section_content.len(), |pos| SECTION_HEADER.len() + pos);
    Some(&section_content[..end])
}

/// Check if line has all required columns (T1: kappa)
fn has_required_columns(line: &str) -> bool {
    let lower = line.to_lowercase();
    REQUIRED_COLUMNS
        .iter()
        .all(|col| lower.contains(&col.to_lowercase()))
}

/// Check if line is a table separator (T1: kappa)
fn is_separator_row(line: &str) -> bool {
    line.contains("---") || line.contains(":---")
}

/// Check if line is a valid data row with 3+ cells (T1: kappa + N)
fn is_valid_data_row(line: &str) -> bool {
    let cells: Vec<&str> = line
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    cells.len() >= 3
}

/// Process a single table line and update state (T1: mu + varsigma)
fn process_table_line(line: &str, state: &mut TableState) {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.contains('|') {
        return;
    }

    if is_separator_row(trimmed) {
        state.found_separator = true;
        return;
    }

    if !state.found_header && has_required_columns(trimmed) {
        state.found_header = true;
        return;
    }

    if state.found_header && state.found_separator && is_valid_data_row(trimmed) {
        state.found_data_row = true;
    }
}

/// Build error message for validation failure (T1: mu)
fn build_error(state: &TableState) -> String {
    if !state.found_header {
        return format!(
            "T1 Primitive Grounding table missing required columns.\n\
             Required: {}\n\n\
             Example:\n\
             | Concept | T1 Primitive | Lex Primitiva Symbol |\n\
             |:---|:---|:---:|\n\
             | **Your Concept** | Mapping | mu |",
            REQUIRED_COLUMNS.join(", ")
        );
    }

    if !state.found_separator {
        return "T1 Primitive Grounding table missing separator row (|---|---|---|)".to_string();
    }

    "T1 Primitive Grounding table has no data rows.\n\
     Add at least one row mapping a concept to a T1 primitive."
        .to_string()
}

/// Validate that content contains T1 Primitive Grounding section (T1: sigma + arrow)
fn validate_primitive_grounding(content: &str) -> Result<(), String> {
    if !content.contains(SECTION_HEADER) {
        return Err(format!("Missing required section: '{}'", SECTION_HEADER));
    }

    let section = extract_section(content).ok_or("Could not locate section start")?;

    let mut state = TableState::default();
    for line in section.lines() {
        process_table_line(line, &mut state);
    }

    if state.found_header && state.found_separator && state.found_data_row {
        Ok(())
    } else {
        Err(build_error(&state))
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => pass(),
    };

    require_edit_tool(input.tool_name.clone());
    let file_path = file_path_or_pass(&input);

    if !is_skill_md(file_path) {
        pass();
    }

    let content = content_or_pass(&input);

    match validate_primitive_grounding(content) {
        Ok(()) => pass(),
        Err(reason) => {
            // Emit cytokine signal before blocking (TNF-alpha = terminate)
            let tool_name = input
                .tool_name
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_default();
            emit_tool_blocked(
                &tool_name,
                HOOK_NAME,
                "SKILL.md missing T1 Primitive Grounding",
            );

            block(&format!(
                "BLOCKED: SKILL.md missing T1 Primitive Grounding\n\n\
                 File: {}\n\n\
                 {}\n\n\
                 Primitive-first semantics required for all skills.\n\
                 Ground concepts to the 15 T1 primitives:\n\
                 sigma, mu, varsigma, rho, emptyset, partial, nu, exists, pi, arrow, kappa, N, lambda, propto, Sigma\n\n\
                 Reference: ~/.claude/skills/primitive-rust-foundation/SKILL.md",
                file_path, reason
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_skill_md() {
        assert!(is_skill_md("SKILL.md"));
        assert!(is_skill_md("/path/to/skill/SKILL.md"));
        assert!(!is_skill_md("README.md"));
        assert!(!is_skill_md("skill.md"));
    }

    #[test]
    fn test_valid_grounding_table() {
        let content = r#"
# My Skill

## T1 Primitive Grounding

| Concept | T1 Primitive | Lex Primitiva Symbol |
|:---|:---|:---:|
| **Data Flow** | Sequence | sigma |

## Other Section
"#;
        assert!(validate_primitive_grounding(content).is_ok());
    }

    #[test]
    fn test_missing_section() {
        let content = "# My Skill\n\n## Other Section";
        let result = validate_primitive_grounding(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required section"));
    }

    #[test]
    fn test_missing_columns() {
        let content =
            "## T1 Primitive Grounding\n\n| Concept | Description |\n|---|---|\n| A | B |";
        let result = validate_primitive_grounding(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_case_insensitive_columns() {
        let content = r#"
## T1 Primitive Grounding

| concept | t1 primitive | lex primitiva symbol |
|:---|:---|:---:|
| **Flow** | Sequence | sigma |
"#;
        assert!(validate_primitive_grounding(content).is_ok());
    }
}
