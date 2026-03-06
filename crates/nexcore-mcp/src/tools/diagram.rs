//! Diagram rendering tool.
//!
//! Renders DOT/Graphviz source to SVG, PNG, or PDF using the system `dot` binary.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::diagram::DiagramRenderParams;

/// Render a DOT diagram to an image file.
pub fn diagram_render(params: DiagramRenderParams) -> Result<CallToolResult, McpError> {
    let format = params.format.unwrap_or_else(|| "svg".to_string());
    let engine = params.engine.unwrap_or_else(|| "dot".to_string());

    // Validate format
    let valid_formats = ["svg", "png", "pdf"];
    if !valid_formats.contains(&format.as_str()) {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Invalid format '{}'. Must be one of: {}",
            format,
            valid_formats.join(", ")
        ))]));
    }

    // Validate engine
    let valid_engines = ["dot", "neato", "circo", "fdp", "twopi"];
    if !valid_engines.contains(&engine.as_str()) {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Invalid engine '{}'. Must be one of: {}",
            engine,
            valid_engines.join(", ")
        ))]));
    }

    // Check dot binary exists
    let dot_path = "/usr/bin/dot";
    if !std::path::Path::new(dot_path).exists() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Graphviz not found at /usr/bin/dot. Install with: sudo apt install graphviz",
        )]));
    }

    // Write source to temp file
    let temp_dir = std::env::temp_dir();
    let input_path = temp_dir.join("nexcore_diagram_input.dot");
    let output_path = temp_dir.join(format!("nexcore_diagram_output.{format}"));

    if let Err(e) = std::fs::write(&input_path, &params.source) {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to write temp file: {e}"
        ))]));
    }

    // Run dot
    let output = std::process::Command::new(dot_path)
        .args([
            &format!("-T{format}"),
            &format!("-K{engine}"),
            "-o",
            output_path.to_str().unwrap_or("output"),
            input_path.to_str().unwrap_or("input"),
        ])
        .output();

    // Clean up input
    let _ = std::fs::remove_file(&input_path);

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "dot failed: {stderr}"
                ))]));
            }

            let file_size = std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0);

            let result = serde_json::json!({
                "success": true,
                "output_path": output_path.display().to_string(),
                "format": format,
                "engine": engine,
                "file_size_bytes": file_size,
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Failed to execute dot: {e}"
        ))])),
    }
}
