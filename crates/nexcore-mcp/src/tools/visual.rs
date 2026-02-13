//! Visual Primitives MCP Tools
//!
//! Shape and color pattern matching for Prima integration.
//! Exposes nexcore-renderer's visual primitive extraction.

use nexcore_renderer::paint::Point;
use nexcore_renderer::style::Color;
use nexcore_renderer::visual_primitives::{ColorPrimitive, ShapeKind, ShapePrimitive};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{VisualColorAnalyzeParams, VisualShapeClassifyParams, VisualShapeListParams};

/// Classify a shape by its T1 primitive composition.
///
/// Returns the primitive grounding, transfer confidence, and recommended
/// Prima representation.
pub fn classify_shape(params: VisualShapeClassifyParams) -> Result<CallToolResult, McpError> {
    let kind = match params.shape.to_lowercase().as_str() {
        "circle" | "⊙" => ShapeKind::Circle,
        "triangle" | "△" => ShapeKind::Triangle,
        "line" | "→" => ShapeKind::Line,
        "rectangle" | "rect" | "□" => ShapeKind::Rectangle,
        "point" | "λ" => ShapeKind::Point,
        "polygon" => ShapeKind::Polygon,
        _ => {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Unknown shape '{}'. Valid shapes: circle/⊙, triangle/△, line/→, rectangle/□, point/λ, polygon",
                params.shape
            ))]));
        }
    };

    let result = serde_json::json!({
        "shape": format!("{:?}", kind),
        "primitives": kind.primitives(),
        "primitive_count": kind.primitive_count(),
        "transfer_confidence": kind.transfer_confidence(),
        "prima_symbol": match kind {
            ShapeKind::Circle => "⊙",
            ShapeKind::Triangle => "△",
            ShapeKind::Line => "→",
            ShapeKind::Rectangle => "□",
            ShapeKind::Point => "λ",
            ShapeKind::Polygon => "σ[λ...]",
        },
        "grounding_chain": format!(
            "{} → {} → T1",
            format!("{:?}", kind),
            kind.primitives()
        ),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Analyze a color and decompose to T1 primitives.
///
/// Returns RGBA quantities, luminance, named match, and grounding.
pub fn analyze_color(params: VisualColorAnalyzeParams) -> Result<CallToolResult, McpError> {
    // Parse color from hex or name
    let color = if params.color.starts_with('#') {
        Color::parse(&params.color)
    } else {
        Color::parse(&params.color)
    };

    let Some(color) = color else {
        return Ok(CallToolResult::success(vec![Content::text(format!(
            "Could not parse color '{}'. Use hex (#ff0000) or named (red, blue, green, white, black)",
            params.color
        ))]));
    };

    let cp = ColorPrimitive::from_color(color);

    let result = serde_json::json!({
        "input": params.color,
        "primitives": ColorPrimitive::primitives(),
        "rgba": {
            "r": cp.r,
            "g": cp.g,
            "b": cp.b,
            "a": cp.a,
        },
        "luminance": cp.luminance(),
        "is_dark": cp.is_dark(),
        "named_match": cp.named_match(),
        "prima_representation": format!(
            "μ(r={:.2}, g={:.2}, b={:.2}, a={:.2})",
            cp.r, cp.g, cp.b, cp.a
        ),
        "grounding_chain": "Color → N(r) + N(g) + N(b) + N(a) → T1",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// List all available shape primitives.
///
/// Returns the catalog of visual primitives with their T1 groundings.
pub fn list_shapes(_params: VisualShapeListParams) -> Result<CallToolResult, McpError> {
    let shapes = [
        ShapeKind::Point,
        ShapeKind::Line,
        ShapeKind::Triangle,
        ShapeKind::Rectangle,
        ShapeKind::Circle,
        ShapeKind::Polygon,
    ];

    let catalog: Vec<_> = shapes
        .iter()
        .map(|kind| {
            serde_json::json!({
                "shape": format!("{:?}", kind),
                "symbol": match kind {
                    ShapeKind::Point => "λ",
                    ShapeKind::Line => "→",
                    ShapeKind::Triangle => "△",
                    ShapeKind::Rectangle => "□",
                    ShapeKind::Circle => "⊙",
                    ShapeKind::Polygon => "σ[λ...]",
                },
                "primitives": kind.primitives(),
                "count": kind.primitive_count(),
                "confidence": kind.transfer_confidence(),
            })
        })
        .collect();

    let result = serde_json::json!({
        "shapes": catalog,
        "total": shapes.len(),
        "grounding_summary": "All shapes ground to T1: λ (location), N (quantity), σ (sequence), → (causality)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Create a shape primitive from parameters.
///
/// Internal helper for constructing ShapePrimitive instances.
#[allow(dead_code)]
fn create_shape(kind: ShapeKind, params: &[f32], color: Color) -> Option<ShapePrimitive> {
    match kind {
        ShapeKind::Circle if params.len() >= 3 => Some(ShapePrimitive::circle(
            Point::new(params[0], params[1]),
            params[2],
            color,
        )),
        ShapeKind::Triangle if params.len() >= 6 => Some(ShapePrimitive::triangle(
            Point::new(params[0], params[1]),
            Point::new(params[2], params[3]),
            Point::new(params[4], params[5]),
            color,
        )),
        ShapeKind::Line if params.len() >= 5 => Some(ShapePrimitive::line(
            Point::new(params[0], params[1]),
            Point::new(params[2], params[3]),
            params[4],
            color,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_circle() {
        let params = VisualShapeClassifyParams {
            shape: "circle".to_string(),
        };
        let result = classify_shape(params);
        assert!(result.is_ok());
        let r = result.ok();
        assert!(r.is_some());
    }

    #[test]
    fn test_classify_with_symbol() {
        let params = VisualShapeClassifyParams {
            shape: "⊙".to_string(),
        };
        let result = classify_shape(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_hex_color() {
        let params = VisualColorAnalyzeParams {
            color: "#ff0000".to_string(),
        };
        let result = analyze_color(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_named_color() {
        let params = VisualColorAnalyzeParams {
            color: "blue".to_string(),
        };
        let result = analyze_color(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_shapes() {
        let params = VisualShapeListParams {};
        let result = list_shapes(params);
        assert!(result.is_ok());
    }
}
