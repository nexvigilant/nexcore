//! ANSI terminal formatting — renders MCP results as terminal-friendly output.
//!
//! Converts structured JSON responses from MCP tools into formatted text
//! suitable for display in xterm.js. Supports tables, key-value pairs,
//! and nested structures with ANSI color codes.
//!
//! ## Primitive Grounding
//!
//! `μ(Mapping) + σ(Sequence) + N(Quantity) + ∂(Boundary)`

/// ANSI escape codes for terminal formatting.
mod ansi {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
    pub const RED: &str = "\x1b[31m";
    pub const WHITE: &str = "\x1b[37m";
}

/// Format an MCP tool result for terminal display.
///
/// Dispatches to the appropriate renderer based on the JSON structure:
/// - Object with "success" key → result banner + content
/// - Array of objects → table
/// - Object → key-value list
/// - Scalar → plain text
#[must_use]
pub fn format_mcp_result(tool_name: &str, value: &serde_json::Value) -> String {
    let mut out = String::new();

    // Tool name header
    out.push_str(&format!(
        "\n{}{} {}{}\n",
        ansi::BOLD,
        ansi::CYAN,
        tool_name,
        ansi::RESET
    ));
    out.push_str(&format!(
        "{}{}─{}\n",
        ansi::DIM,
        "─".repeat(tool_name.len().saturating_add(1)),
        ansi::RESET
    ));

    match value {
        serde_json::Value::Object(map) => {
            // Check for success/error banner
            if let Some(success) = map.get("success").and_then(|v| v.as_bool()) {
                if success {
                    out.push_str(&format!(
                        "{}{} OK{}\n",
                        ansi::GREEN,
                        ansi::BOLD,
                        ansi::RESET
                    ));
                } else {
                    let msg = map
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error");
                    out.push_str(&format!(
                        "{}{} FAIL: {}{}\n",
                        ansi::RED,
                        ansi::BOLD,
                        msg,
                        ansi::RESET
                    ));
                }
            }
            // Render remaining fields as key-value pairs
            format_key_value(map, &mut out, 0);
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                out.push_str("  (empty result)\n");
            } else if arr.iter().all(|v| v.is_object()) {
                format_table(arr, &mut out);
            } else {
                for (i, item) in arr.iter().enumerate() {
                    out.push_str(&format!(
                        "  {}[{}]{} {}\n",
                        ansi::DIM,
                        i,
                        ansi::RESET,
                        format_scalar(item)
                    ));
                }
            }
        }
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => {
            out.push_str(&format!("  {}\n", format_scalar(value)));
        }
    }

    out.push('\n');
    out
}

/// Format an object as indented key-value pairs.
fn format_key_value(
    map: &serde_json::Map<String, serde_json::Value>,
    out: &mut String,
    depth: usize,
) {
    let indent = "  ".repeat(depth.saturating_add(1));

    for (key, value) in map {
        // Skip the "success" key (already rendered as banner)
        if key == "success" && depth == 0 {
            continue;
        }

        match value {
            serde_json::Value::Object(nested) => {
                out.push_str(&format!(
                    "{}{}{}{}: \n",
                    indent,
                    ansi::YELLOW,
                    key,
                    ansi::RESET
                ));
                format_key_value(nested, out, depth.saturating_add(1));
            }
            serde_json::Value::Array(arr) if arr.iter().all(|v| v.is_object()) => {
                out.push_str(&format!(
                    "{}{}{}{}: ({} items)\n",
                    indent,
                    ansi::YELLOW,
                    key,
                    ansi::RESET,
                    arr.len()
                ));
                format_table(arr, out);
            }
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_)
            | serde_json::Value::Array(_) => {
                out.push_str(&format!(
                    "{}{}{}{}: {}\n",
                    indent,
                    ansi::YELLOW,
                    key,
                    ansi::RESET,
                    format_scalar(value)
                ));
            }
        }
    }
}

/// Format an array of objects as an ASCII table.
fn format_table(rows: &[serde_json::Value], out: &mut String) {
    if rows.is_empty() {
        return;
    }

    // Collect all unique keys across all rows for headers
    let mut headers: Vec<String> = Vec::new();
    for row in rows {
        if let Some(obj) = row.as_object() {
            for key in obj.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }

    if headers.is_empty() {
        return;
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(String::len).collect();
    let mut cell_data: Vec<Vec<String>> = Vec::new();

    for row in rows {
        let mut cells = Vec::new();
        for (i, header) in headers.iter().enumerate() {
            let cell = row
                .get(header)
                .map(|v| format_scalar(v))
                .unwrap_or_default();
            if let Some(w) = widths.get_mut(i) {
                if cell.len() > *w {
                    *w = cell.len();
                }
            }
            cells.push(cell);
        }
        cell_data.push(cells);
    }

    // Cap column widths at 40 chars
    for w in &mut widths {
        if *w > 40 {
            *w = 40;
        }
    }

    // Render header row
    out.push_str("  ");
    for (i, header) in headers.iter().enumerate() {
        let w = widths.get(i).copied().unwrap_or(10);
        out.push_str(&format!(
            "{}{}{}{}",
            ansi::BOLD,
            ansi::WHITE,
            pad_or_truncate(header, w),
            ansi::RESET
        ));
        if i < headers.len().saturating_sub(1) {
            out.push_str("  ");
        }
    }
    out.push('\n');

    // Separator
    out.push_str("  ");
    for (i, _) in headers.iter().enumerate() {
        let w = widths.get(i).copied().unwrap_or(10);
        out.push_str(&format!("{}{}{}", ansi::DIM, "─".repeat(w), ansi::RESET));
        if i < headers.len().saturating_sub(1) {
            out.push_str("  ");
        }
    }
    out.push('\n');

    // Data rows
    for cells in &cell_data {
        out.push_str("  ");
        for (i, cell) in cells.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or(10);
            out.push_str(&pad_or_truncate(cell, w));
            if i < cells.len().saturating_sub(1) {
                out.push_str("  ");
            }
        }
        out.push('\n');
    }
}

/// Format a scalar JSON value as a display string.
fn format_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "(null)".to_string(),
        serde_json::Value::Bool(b) => {
            if *b {
                format!("{}true{}", ansi::GREEN, ansi::RESET)
            } else {
                format!("{}false{}", ansi::RED, ansi::RESET)
            }
        }
        serde_json::Value::Number(n) => format!("{}{}{}", ansi::CYAN, n, ansi::RESET),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(map) => format!("{{{} keys}}", map.len()),
    }
}

/// Pad a string to `width` or truncate with ellipsis.
fn pad_or_truncate(s: &str, width: usize) -> String {
    if s.len() > width {
        if width > 3 {
            let truncated = s.get(..width.saturating_sub(3)).unwrap_or(s);
            format!("{truncated}...")
        } else {
            s.chars().take(width).collect()
        }
    } else {
        format!("{:width$}", s, width = width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_simple_object() {
        let value = serde_json::json!({
            "success": true,
            "drug": "aspirin",
            "prr": 2.5
        });
        let output = format_mcp_result("pv_signal_complete", &value);
        assert!(output.contains("pv_signal_complete"));
        assert!(output.contains("OK"));
        assert!(output.contains("drug"));
        assert!(output.contains("aspirin"));
    }

    #[test]
    fn format_error_result() {
        let value = serde_json::json!({
            "success": false,
            "error": "drug not found"
        });
        let output = format_mcp_result("faers_search", &value);
        assert!(output.contains("FAIL"));
        assert!(output.contains("drug not found"));
    }

    #[test]
    fn format_array_as_table() {
        let value = serde_json::json!([
            {"drug": "aspirin", "prr": 2.1, "signal": true},
            {"drug": "ibuprofen", "prr": 1.3, "signal": false}
        ]);
        let output = format_mcp_result("faers_drug_events", &value);
        assert!(output.contains("drug"));
        assert!(output.contains("aspirin"));
        assert!(output.contains("ibuprofen"));
    }

    #[test]
    fn format_empty_array() {
        let value = serde_json::json!([]);
        let output = format_mcp_result("test_tool", &value);
        assert!(output.contains("empty result"));
    }

    #[test]
    fn format_scalar_values() {
        assert_eq!(format_scalar(&serde_json::Value::Null), "(null)");
        assert!(format_scalar(&serde_json::json!(true)).contains("true"));
        assert!(format_scalar(&serde_json::json!(42)).contains("42"));
        assert_eq!(format_scalar(&serde_json::json!("hello")), "hello");
    }

    #[test]
    fn pad_or_truncate_short() {
        assert_eq!(pad_or_truncate("hi", 10), "hi        ");
    }

    #[test]
    fn pad_or_truncate_long() {
        assert_eq!(pad_or_truncate("hello world", 8), "hello...");
    }

    #[test]
    fn format_nested_object() {
        let value = serde_json::json!({
            "summary": {
                "total_reports": 150,
                "serious": 42
            },
            "query": "aspirin"
        });
        let output = format_mcp_result("faers_summary", &value);
        assert!(output.contains("summary"));
        assert!(output.contains("query"));
    }
}
