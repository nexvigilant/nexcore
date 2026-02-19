//! Formatted output utilities for hook checkpoints.

/// Format a checkpoint box with header and content
pub fn format_checkpoint(header: &str, content: &str) -> String {
    let width = 77;
    let border = "─".repeat(width);
    let formatted_content = format_content(content, width);

    format!("┌─{border}─┐\n│ {header:<width$} │\n├─{border}─┤\n{formatted_content}└─{border}─┘")
}

fn format_content(content: &str, width: usize) -> String {
    let mut result = String::new();
    for line in content.lines() {
        if line.len() <= width {
            result.push_str(&format!("│ {line:<width$} │\n"));
        } else {
            let mut current = String::new();
            for word in line.split_whitespace() {
                if current.len() + word.len() + 1 > width {
                    result.push_str(&format!("│ {current:<width$} │\n"));
                    current = word.to_string();
                } else if current.is_empty() {
                    current = word.to_string();
                } else {
                    current.push(' ');
                    current.push_str(word);
                }
            }
            if !current.is_empty() {
                result.push_str(&format!("│ {current:<width$} │\n"));
            }
        }
    }
    result
}

/// Format a requirements checkpoint message
pub fn format_requirements_checkpoint() -> String {
    format_checkpoint(
        "REQUIREMENTS CHECKPOINT",
        r"
Before implementing, verify requirements are explicit:

EXPLICIT REQUIREMENTS (stated by user):
  - Extract specific requirements from the user's message

IMPLICIT REQUIREMENTS (inferred - REQUIRE CONFIRMATION):
  - List any assumptions being made

SCOPE BOUNDARIES:
  IN SCOPE: What this implementation will do
  OUT OF SCOPE: What this implementation will NOT do

Do not proceed until requirements are clear.
",
    )
}

/// Format an incremental verification checkpoint
pub fn format_verification_checkpoint(lines: u32, files: u32, threshold: u32) -> String {
    format_checkpoint(
        "INCREMENTAL VERIFICATION REQUIRED",
        &format!(
            "Lines since last check: {lines} (threshold: {threshold})\n\
             Files since last check: {files}\n\n\
             REQUIRED ACTION: Run cargo check before adding more code."
        ),
    )
}

/// Format a panic path violation checkpoint
pub fn format_panic_checkpoint(violations: &[(usize, String, &str)]) -> String {
    let mut content = String::new();
    for (line, pattern, fix) in violations {
        content.push_str(&format!("Line {line}: {pattern}\n  Fix: {fix}\n\n"));
    }
    content.push_str("Use // INVARIANT: comment if truly impossible to fail");
    format_checkpoint("PANIC PATH DETECTED", &content)
}
