//! Observatory personalization tools — auto-detect, get, set, and validate rendering preferences.
//!
//! Pure computation — no I/O or runtime dependencies.
//! Primitives: κ(Comparison) + ς(State) + ∂(Boundary) + μ(Mapping)

use crate::params::observatory::{
    PersonalizeDetectParams, PersonalizeGetParams, PersonalizeSetParams, PersonalizeValidateParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

const VALID_QUALITIES: &[&str] = &["low", "medium", "high", "cinematic"];
const VALID_THEMES: &[&str] = &["default", "warm", "clinical", "high-contrast"];
const VALID_CVD_MODES: &[&str] = &["normal", "deuteranopia", "protanopia", "tritanopia"];
const VALID_EXPLORERS: &[&str] = &["graph", "career", "learning", "state", "math"];
const VALID_LAYOUTS: &[&str] = &["force", "hierarchy", "radial", "grid"];
const VALID_POST_PROCESSING: &[&str] = &["bloom", "ssao", "vignette"];

// ---------------------------------------------------------------------------
// Tool: observatory_personalize_detect
// ---------------------------------------------------------------------------

/// Auto-detect optimal Observatory settings from device capabilities.
pub fn personalize_detect(params: PersonalizeDetectParams) -> Result<CallToolResult, McpError> {
    // Start with defaults
    let mut quality: &str = "medium";
    let mut theme: &str = "default";
    let mut post_processing: Vec<String> = vec!["bloom".to_string(), "ssao".to_string()];
    let mut enable_worker_layout = true;

    // Accessibility signals take top priority
    if params.prefers_reduced_motion.unwrap_or(false) {
        quality = "low";
        post_processing.clear();
    }

    if params
        .prefers_contrast
        .as_deref()
        .unwrap_or("no-preference")
        == "more"
    {
        theme = "high-contrast";
    }

    // Resource-constrained device detection
    let mem = params.device_memory.unwrap_or(8.0);
    let cores = params.hardware_concurrency.unwrap_or(8);

    if mem < 4.0 || cores < 4 {
        quality = "low";
        post_processing.clear();
        enable_worker_layout = false;
    } else if mem >= 8.0 && cores >= 8 && quality != "low" {
        quality = "high";
    }

    // GPU vendor cap
    if let Some(ref renderer) = params.gpu_renderer {
        let renderer_str: &str = renderer.as_str();
        if renderer_str.contains("Intel") && quality == "high" {
            quality = "medium";
        }
    }

    // Cinematic is never auto-selected — requires explicit user opt-in
    let result = json!({
        "recommended": {
            "quality": quality,
            "theme": theme,
            "post_processing": post_processing,
            "enable_worker_layout": enable_worker_layout,
        },
        "detection_signals": {
            "gpu_renderer": params.gpu_renderer,
            "device_pixel_ratio": params.device_pixel_ratio,
            "hardware_concurrency": params.hardware_concurrency,
            "device_memory": params.device_memory,
            "prefers_reduced_motion": params.prefers_reduced_motion,
            "prefers_contrast": params.prefers_contrast,
        },
        "t1_grounding": "κ(Comparison) + ς(State) + ∂(Boundary)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: observatory_personalize_get
// ---------------------------------------------------------------------------

/// Return default Observatory preferences for a named profile.
pub fn personalize_get(params: PersonalizeGetParams) -> Result<CallToolResult, McpError> {
    let profile = params.profile.as_deref().unwrap_or("default");

    let result = match profile {
        "power-user" => json!({
            "profile": "power-user",
            "preferences": {
                "quality": "cinematic",
                "theme": "default",
                "cvd_mode": "normal",
                "default_explorer": "graph",
                "default_layout": "force",
                "enable_worker_layout": true,
                "post_processing": ["bloom", "ssao", "vignette"],
            },
            "description": "Maximum fidelity for high-end workstations.",
        }),
        "accessibility" => json!({
            "profile": "accessibility",
            "preferences": {
                "quality": "medium",
                "theme": "high-contrast",
                "cvd_mode": "deuteranopia",
                "default_explorer": "learning",
                "default_layout": "grid",
                "enable_worker_layout": false,
                "post_processing": [],
            },
            "description": "Maximally accessible — high-contrast, reduced motion, CVD-safe palette.",
        }),
        "mobile" => json!({
            "profile": "mobile",
            "preferences": {
                "quality": "low",
                "theme": "default",
                "cvd_mode": "normal",
                "default_explorer": "graph",
                "default_layout": "force",
                "enable_worker_layout": false,
                "post_processing": [],
            },
            "description": "Performance-first for mobile and low-power devices.",
        }),
        _ => json!({
            "profile": "default",
            "preferences": {
                "quality": "medium",
                "theme": "default",
                "cvd_mode": "normal",
                "default_explorer": "graph",
                "default_layout": "force",
                "enable_worker_layout": true,
                "post_processing": ["bloom"],
            },
            "description": "Balanced defaults for most devices and users.",
        }),
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: observatory_personalize_set
// ---------------------------------------------------------------------------

/// Validate and normalise an Observatory preferences object.
pub fn personalize_set(params: PersonalizeSetParams) -> Result<CallToolResult, McpError> {
    let mut errors: Vec<String> = Vec::new();

    // Validate each field against its allowed set
    if let Some(ref q) = params.quality {
        if !VALID_QUALITIES.contains(&q.as_str()) {
            errors.push(format!(
                "quality '{}' is invalid; allowed: {}",
                q,
                VALID_QUALITIES.join(", "),
            ));
        }
    }
    if let Some(ref t) = params.theme {
        if !VALID_THEMES.contains(&t.as_str()) {
            errors.push(format!(
                "theme '{}' is invalid; allowed: {}",
                t,
                VALID_THEMES.join(", "),
            ));
        }
    }
    if let Some(ref c) = params.cvd_mode {
        if !VALID_CVD_MODES.contains(&c.as_str()) {
            errors.push(format!(
                "cvd_mode '{}' is invalid; allowed: {}",
                c,
                VALID_CVD_MODES.join(", "),
            ));
        }
    }
    if let Some(ref e) = params.default_explorer {
        if !VALID_EXPLORERS.contains(&e.as_str()) {
            errors.push(format!(
                "default_explorer '{}' is invalid; allowed: {}",
                e,
                VALID_EXPLORERS.join(", "),
            ));
        }
    }
    if let Some(ref l) = params.default_layout {
        if !VALID_LAYOUTS.contains(&l.as_str()) {
            errors.push(format!(
                "default_layout '{}' is invalid; allowed: {}",
                l,
                VALID_LAYOUTS.join(", "),
            ));
        }
    }
    if let Some(ref effects) = params.post_processing {
        for effect in effects {
            if !VALID_POST_PROCESSING.contains(&effect.as_str()) {
                errors.push(format!(
                    "post_processing item '{}' is invalid; allowed: {}",
                    effect,
                    VALID_POST_PROCESSING.join(", "),
                ));
            }
        }
    }

    if !errors.is_empty() {
        let result = json!({
            "valid": false,
            "errors": errors,
        });
        return Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let result = json!({
        "valid": true,
        "normalized": {
            "quality": params.quality.unwrap_or_else(|| "medium".to_string()),
            "theme": params.theme.unwrap_or_else(|| "default".to_string()),
            "cvd_mode": params.cvd_mode.unwrap_or_else(|| "normal".to_string()),
            "default_explorer": params.default_explorer.unwrap_or_else(|| "graph".to_string()),
            "default_layout": params.default_layout.unwrap_or_else(|| "force".to_string()),
            "enable_worker_layout": params.enable_worker_layout.unwrap_or(true),
            "post_processing": params.post_processing.unwrap_or_else(|| vec!["bloom".to_string()]),
        },
        "t1_grounding": "ς(State) + κ(Comparison) + ∂(Boundary) + μ(Mapping)"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: observatory_personalize_validate
// ---------------------------------------------------------------------------

/// Cross-validate an Observatory config against explorer capability constraints.
pub fn personalize_validate(params: PersonalizeValidateParams) -> Result<CallToolResult, McpError> {
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut suggestions: Vec<String> = Vec::new();

    let explorer: &str = params.explorer.as_deref().unwrap_or("graph");
    let quality: &str = params.quality.as_deref().unwrap_or("medium");
    let cvd_mode: &str = params.cvd_mode.as_deref().unwrap_or("normal");
    let layout: Option<&str> = params.layout.as_deref();
    let post_processing: &[String] = params.post_processing.as_deref().unwrap_or(&[]);

    // Schema validation first
    if let Some(ref q) = params.quality {
        if !VALID_QUALITIES.contains(&q.as_str()) {
            errors.push(format!("quality '{}' not recognised", q));
        }
    }
    if let Some(ref t) = params.theme {
        if !VALID_THEMES.contains(&t.as_str()) {
            errors.push(format!("theme '{}' not recognised", t));
        }
    }
    if !VALID_CVD_MODES.contains(&cvd_mode) {
        errors.push(format!("cvd_mode '{}' not recognised", cvd_mode));
    }
    if !VALID_EXPLORERS.contains(&explorer) {
        errors.push(format!("explorer '{}' not recognised", explorer));
    }
    if let Some(l) = layout {
        if !VALID_LAYOUTS.contains(&l) {
            errors.push(format!("layout '{}' not recognised", l));
        }
    }
    for effect in post_processing {
        if !VALID_POST_PROCESSING.contains(&effect.as_str()) {
            errors.push(format!("post_processing item '{}' not recognised", effect));
        }
    }

    // Explorer capability constraints
    if explorer == "state" && cvd_mode != "normal" {
        warnings.push(
            "Explorer 'state' does not yet support CVD simulation — cvd_mode will be ignored."
                .to_string(),
        );
    }
    if explorer == "math" && layout.is_some() {
        warnings.push(
            "Explorer 'math' manages its own layout; the 'layout' setting will have no effect."
                .to_string(),
        );
    }

    // Quality / post-processing suggestions
    if quality == "cinematic" && !post_processing.iter().any(|e| e == "bloom") {
        suggestions.push(
            "Quality 'cinematic' pairs best with the 'bloom' post-processing effect.".to_string(),
        );
    }

    let valid = errors.is_empty();
    let result = json!({
        "valid": valid,
        "errors": errors,
        "warnings": warnings,
        "suggestions": suggestions,
        "config_summary": {
            "explorer": explorer,
            "quality": quality,
            "cvd_mode": cvd_mode,
            "layout": layout,
            "theme": params.theme,
            "post_processing": post_processing,
        },
        "t1_grounding": "κ(Comparison) + ∂(Boundary) + ς(State) + ∃(Existence)"
    });

    if valid {
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    } else {
        Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]))
    }
}
