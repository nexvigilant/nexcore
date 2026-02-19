//! Frontend & Accessibility tools: WCAG contrast, touch targets, type scale, spacing, a11y audit.
//!
//! Pure computation — no browser or runtime dependencies.
//! Primitives: κ(Comparison) + N(Quantity) + ∂(Boundary) + Σ(Sum)

use crate::params::{
    A11yAuditParams, ColorBlendParams, ContrastPair, SpacingAuditParams, TouchTarget,
    TouchTargetParams, TypeScaleAuditParams, WcagContrastParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// Color math (sRGB → linear → relative luminance → contrast ratio)
// ---------------------------------------------------------------------------

fn srgb_to_linear(c: f64) -> f64 {
    let c = c / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    0.2126 * srgb_to_linear(r) + 0.7152 * srgb_to_linear(g) + 0.0722 * srgb_to_linear(b)
}

fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

fn blend_alpha(fg: &[f64], bg: &[f64]) -> [f64; 3] {
    let a = if fg.len() >= 4 { fg[3] } else { 1.0 };
    [
        (fg[0] * a + bg[0] * (1.0 - a)).round(),
        (fg[1] * a + bg[1] * (1.0 - a)).round(),
        (fg[2] * a + bg[2] * (1.0 - a)).round(),
    ]
}

fn is_large_text(font_size_px: f64, font_weight: u16) -> bool {
    // WCAG: large text is >= 24px, or >= 18.66px if bold (weight >= 700)
    font_size_px >= 24.0 || (font_weight >= 700 && font_size_px >= 18.66)
}

fn wcag_verdict(ratio: f64, large: bool) -> (&'static str, &'static str, &'static str) {
    let aa = if large { ratio >= 3.0 } else { ratio >= 4.5 };
    let aaa = if large { ratio >= 4.5 } else { ratio >= 7.0 };
    (
        if aa { "PASS" } else { "FAIL" },
        if aaa { "PASS" } else { "FAIL" },
        if large { "large" } else { "normal" },
    )
}

// ---------------------------------------------------------------------------
// Tool: frontend_wcag_contrast
// ---------------------------------------------------------------------------

/// Compute WCAG 2.1 contrast ratio between two colors with AA/AAA verdicts.
pub fn wcag_contrast(params: WcagContrastParams) -> Result<CallToolResult, McpError> {
    if params.foreground.len() < 3 || params.background.len() < 3 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Colors must have at least 3 components [r, g, b]"}).to_string(),
        )]));
    }

    let eff_fg = blend_alpha(&params.foreground, &params.background);
    let fg_l = relative_luminance(eff_fg[0], eff_fg[1], eff_fg[2]);
    let bg_l = relative_luminance(
        params.background[0],
        params.background[1],
        params.background[2],
    );
    let ratio = contrast_ratio(fg_l, bg_l);
    let large = is_large_text(params.font_size_px, params.font_weight);
    let (aa, aaa, text_size) = wcag_verdict(ratio, large);

    let ratio_rounded = (ratio * 100.0).round() / 100.0;
    let min_aa = if large { 3.0 } else { 4.5 };
    let min_aaa = if large { 4.5 } else { 7.0 };

    let recommendation = if aa == "FAIL" {
        let needed_opacity = if params.foreground.len() >= 4 && params.foreground[3] < 1.0 {
            // Suggest a minimum opacity to pass AA
            let mut test_a = params.foreground[3];
            while test_a <= 1.0 {
                let test_fg = [
                    (params.foreground[0] * test_a + params.background[0] * (1.0 - test_a)).round(),
                    (params.foreground[1] * test_a + params.background[1] * (1.0 - test_a)).round(),
                    (params.foreground[2] * test_a + params.background[2] * (1.0 - test_a)).round(),
                ];
                let test_l = relative_luminance(test_fg[0], test_fg[1], test_fg[2]);
                let test_ratio = contrast_ratio(test_l, bg_l);
                if test_ratio >= min_aa {
                    break;
                }
                test_a += 0.01;
            }
            if test_a <= 1.0 {
                Some(((test_a * 100.0).ceil() / 100.0).to_string())
            } else {
                None
            }
        } else {
            None
        };
        match needed_opacity {
            Some(opacity) => format!(
                "Increase opacity to >= {} to pass AA ({}:1)",
                opacity, min_aa
            ),
            None => format!(
                "Needs {}:1 for AA. Consider a lighter foreground or darker background.",
                min_aa
            ),
        }
    } else if aaa == "FAIL" {
        format!("Passes AA but not AAA. Needs {}:1 for AAA.", min_aaa)
    } else {
        "Excellent contrast — passes AA and AAA.".to_string()
    };

    let result = json!({
        "contrast_ratio": ratio_rounded,
        "effective_foreground": format!("rgb({}, {}, {})", eff_fg[0], eff_fg[1], eff_fg[2]),
        "text_size_category": text_size,
        "wcag_aa": aa,
        "wcag_aaa": aaa,
        "min_ratio_aa": min_aa,
        "min_ratio_aaa": min_aaa,
        "recommendation": recommendation,
        "t1_grounding": "κ(Comparison) + N(Quantity) + ∂(Boundary)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: frontend_color_blend
// ---------------------------------------------------------------------------

/// Blend an RGBA foreground color onto an opaque background, returning the effective RGB.
pub fn color_blend(params: ColorBlendParams) -> Result<CallToolResult, McpError> {
    if params.foreground.len() < 3 || params.background.len() < 3 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Colors must have at least 3 components"}).to_string(),
        )]));
    }

    let alpha = if params.foreground.len() >= 4 {
        params.foreground[3]
    } else {
        1.0
    };
    let eff = blend_alpha(&params.foreground, &params.background);

    let result = json!({
        "effective_color": [eff[0] as u8, eff[1] as u8, eff[2] as u8],
        "effective_rgb": format!("rgb({}, {}, {})", eff[0] as u8, eff[1] as u8, eff[2] as u8),
        "alpha_applied": alpha,
        "t1_grounding": "μ(Mapping) + N(Quantity)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: frontend_touch_target
// ---------------------------------------------------------------------------

/// Check interactive element dimensions against WCAG touch target requirements.
pub fn touch_target(params: TouchTargetParams) -> Result<CallToolResult, McpError> {
    let min_size = match params.level.as_str() {
        "aaa" => 48.0,
        _ => 44.0, // AA default
    };

    let min_dim = params.width.min(params.height);
    let pass = min_dim >= min_size;
    let deficit = if pass { 0.0 } else { min_size - min_dim };

    let result = json!({
        "width": params.width,
        "height": params.height,
        "min_dimension": min_dim,
        "required_min": min_size,
        "level": params.level,
        "verdict": if pass { "PASS" } else { "FAIL" },
        "deficit_px": deficit,
        "recommendation": if pass {
            "Touch target meets requirements.".to_string()
        } else {
            format!("Increase minimum dimension by {}px to meet {} ({}x{}px)", deficit, params.level.to_uppercase(), min_size, min_size)
        },
        "wcag_criterion": "2.5.5 Target Size (Enhanced) / 2.5.8 Target Size (Minimum)",
        "t1_grounding": "∂(Boundary) + N(Quantity) + κ(Comparison)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: frontend_type_scale_audit
// ---------------------------------------------------------------------------

/// Audit a set of font sizes for modular scale compliance.
/// Identifies gaps, clusters, and ratio deviations.
pub fn type_scale_audit(params: TypeScaleAuditParams) -> Result<CallToolResult, McpError> {
    let mut sizes = params.sizes.clone();
    sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sizes.dedup();

    if sizes.len() < 2 {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "Need at least 2 font sizes to audit"}).to_string(),
        )]));
    }

    let target = params.target_ratio;
    let tol = params.tolerance;

    let mut steps = Vec::new();
    let mut gaps = Vec::new();
    let mut clusters = Vec::new();

    for i in 1..sizes.len() {
        let ratio = sizes[i] / sizes[i - 1];
        let deviation = (ratio - target).abs() / target;
        let compliant = deviation <= tol;

        steps.push(json!({
            "from": sizes[i - 1],
            "to": sizes[i],
            "ratio": (ratio * 1000.0).round() / 1000.0,
            "deviation_pct": (deviation * 100.0).round() / 100.0 * 100.0,
            "compliant": compliant,
        }));

        // Gap: ratio is more than 2x the target (missing intermediate size)
        if ratio > target * 2.0 {
            let suggested = sizes[i - 1] * target;
            gaps.push(json!({
                "between": [sizes[i - 1], sizes[i]],
                "actual_ratio": (ratio * 1000.0).round() / 1000.0,
                "suggested_intermediate": (suggested * 10.0).round() / 10.0,
            }));
        }

        // Cluster: ratio is less than 1.1 (nearly identical sizes)
        if ratio < 1.1 {
            clusters.push(json!({
                "sizes": [sizes[i - 1], sizes[i]],
                "ratio": (ratio * 1000.0).round() / 1000.0,
                "recommendation": "Consider merging — difference is below perceptual threshold"
            }));
        }
    }

    let compliant_count = steps.iter().filter(|s| s["compliant"] == true).count();
    let total_steps = steps.len();
    let score = if total_steps > 0 {
        (compliant_count as f64 / total_steps as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    let result = json!({
        "sizes_audited": sizes,
        "target_ratio": target,
        "tolerance": tol,
        "steps": steps,
        "gaps": gaps,
        "clusters": clusters,
        "compliance_score": score,
        "compliant_steps": compliant_count,
        "total_steps": total_steps,
        "t1_grounding": "N(Quantity) + κ(Comparison) + σ(Sequence)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: frontend_spacing_audit
// ---------------------------------------------------------------------------

/// Audit spacing values against a modular scale (base * ratio^n).
/// Reports which values are on-scale, off-scale, and the nearest scale step.
pub fn spacing_audit(params: SpacingAuditParams) -> Result<CallToolResult, McpError> {
    let mut values = params.values.clone();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values.dedup();

    let base = params.base;
    let ratio = params.ratio;

    // Generate expected scale: base * ratio^n for n = 0..12
    let scale: Vec<f64> = (0..=12)
        .map(|n| (base * ratio.powi(n) * 1000.0).round() / 1000.0)
        .collect();

    let mut audited = Vec::new();
    let mut on_scale_count = 0;

    for &val in &values {
        // Find nearest scale value
        let (nearest_idx, nearest_val) = scale
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                ((**a - val).abs())
                    .partial_cmp(&((**b - val).abs()))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, &v)| (i, v))
            .unwrap_or((0, base));

        let deviation = (val - nearest_val).abs();
        let deviation_pct = if nearest_val > 0.0 {
            deviation / nearest_val * 100.0
        } else {
            100.0
        };
        let on_scale = deviation_pct < 5.0; // Within 5% of a scale step

        if on_scale {
            on_scale_count += 1;
        }

        audited.push(json!({
            "value": val,
            "nearest_scale_step": nearest_val,
            "scale_index": nearest_idx,
            "deviation_px": (deviation * 100.0).round() / 100.0,
            "deviation_pct": (deviation_pct * 10.0).round() / 10.0,
            "on_scale": on_scale,
        }));
    }

    let score = if values.is_empty() {
        0.0
    } else {
        (on_scale_count as f64 / values.len() as f64 * 100.0).round() / 100.0
    };

    let result = json!({
        "base": base,
        "ratio": ratio,
        "expected_scale": scale,
        "audited_values": audited,
        "on_scale_count": on_scale_count,
        "total_values": values.len(),
        "compliance_score": score,
        "t1_grounding": "N(Quantity) + λ(Location) + κ(Comparison)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: frontend_a11y_summary
// ---------------------------------------------------------------------------

/// Combined accessibility audit: contrast pairs + touch targets + heading hierarchy.
pub fn a11y_summary(params: A11yAuditParams) -> Result<CallToolResult, McpError> {
    // --- Contrast ---
    let mut contrast_results = Vec::new();
    let mut contrast_pass = 0;
    let mut contrast_total = 0;

    for pair in &params.contrast_pairs {
        if pair.fg.len() < 3 || pair.bg.len() < 3 {
            continue;
        }
        contrast_total += 1;
        let eff_fg = blend_alpha(&pair.fg, &pair.bg);
        let fg_l = relative_luminance(eff_fg[0], eff_fg[1], eff_fg[2]);
        let bg_l = relative_luminance(pair.bg[0], pair.bg[1], pair.bg[2]);
        let ratio = contrast_ratio(fg_l, bg_l);
        let large = is_large_text(pair.font_size_px, pair.font_weight);
        let (aa, aaa, text_size) = wcag_verdict(ratio, large);

        if aa == "PASS" {
            contrast_pass += 1;
        }

        contrast_results.push(json!({
            "name": pair.name,
            "ratio": (ratio * 100.0).round() / 100.0,
            "text_size": text_size,
            "aa": aa,
            "aaa": aaa,
        }));
    }

    // --- Touch targets ---
    let mut touch_results = Vec::new();
    let mut touch_pass = 0;
    let mut touch_total = 0;

    for target in &params.touch_targets {
        touch_total += 1;
        let min_dim = target.width.min(target.height);
        let pass = min_dim >= 44.0;
        if pass {
            touch_pass += 1;
        }

        touch_results.push(json!({
            "name": target.name,
            "width": target.width,
            "height": target.height,
            "min_dimension": min_dim,
            "verdict": if pass { "PASS" } else { "FAIL" },
        }));
    }

    // --- Heading hierarchy ---
    let mut heading_issues = Vec::new();
    let levels = &params.heading_levels;
    if !levels.is_empty() {
        // Check: first heading should be h1
        if levels[0] != 1 {
            heading_issues.push(format!("First heading is h{}, expected h1", levels[0]));
        }
        // Check: no level skips (h1 → h3 without h2)
        for i in 1..levels.len() {
            if levels[i] > levels[i - 1] + 1 {
                heading_issues.push(format!(
                    "Heading skip: h{} → h{} (missing h{})",
                    levels[i - 1],
                    levels[i],
                    levels[i - 1] + 1
                ));
            }
        }
        // Check: multiple h1s
        let h1_count = levels.iter().filter(|&&l| l == 1).count();
        if h1_count > 1 {
            heading_issues.push(format!("Multiple h1 elements found ({})", h1_count));
        }
    }

    let heading_pass = heading_issues.is_empty() && !levels.is_empty();

    // --- Composite score ---
    let contrast_score = if contrast_total > 0 {
        contrast_pass as f64 / contrast_total as f64
    } else {
        1.0
    };
    let touch_score = if touch_total > 0 {
        touch_pass as f64 / touch_total as f64
    } else {
        1.0
    };
    let heading_score = if heading_pass { 1.0 } else { 0.5 };

    // Weighted: contrast 40%, touch 30%, headings 30%
    let composite =
        ((contrast_score * 0.4 + touch_score * 0.3 + heading_score * 0.3) * 100.0).round() / 100.0;

    let result = json!({
        "contrast": {
            "pass": contrast_pass,
            "total": contrast_total,
            "score": (contrast_score * 100.0).round() / 100.0,
            "results": contrast_results,
        },
        "touch_targets": {
            "pass": touch_pass,
            "total": touch_total,
            "score": (touch_score * 100.0).round() / 100.0,
            "results": touch_results,
        },
        "heading_hierarchy": {
            "levels": levels,
            "valid": heading_pass,
            "issues": heading_issues,
            "score": heading_score,
        },
        "composite_score": composite,
        "grade": if composite >= 0.9 { "A" } else if composite >= 0.7 { "B" } else if composite >= 0.5 { "C" } else { "F" },
        "t1_grounding": "κ(Comparison) + N(Quantity) + ∂(Boundary) + σ(Sequence) + Σ(Sum)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
