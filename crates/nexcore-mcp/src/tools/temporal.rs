//! Temporal Analysis Tools
//!
//! Time-to-onset, dechallenge/rechallenge, and temporal plausibility MCP tools.

use crate::params::temporal::{
    ChallengeParams, DechallengeParam, RechallengeParam, TemporalPlausibilityParams, TtoParams,
};
use crate::tooling::attach_forensic_meta;
use nexcore_pv_core::temporal::{
    DechallengeResponse, RechallengeResponse, assess_challenge_with_timing, temporal_plausibility,
    time_to_onset, time_to_onset_days,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// =============================================================================
// Conversion helpers
// =============================================================================

fn to_dechallenge(d: &DechallengeParam) -> DechallengeResponse {
    match d {
        DechallengeParam::Positive => DechallengeResponse::Positive,
        DechallengeParam::Negative => DechallengeResponse::Negative,
        DechallengeParam::Partial => DechallengeResponse::Partial,
        DechallengeParam::NotApplicable => DechallengeResponse::NotApplicable,
        DechallengeParam::Unknown => DechallengeResponse::Unknown,
    }
}

fn to_rechallenge(r: &RechallengeParam) -> RechallengeResponse {
    match r {
        RechallengeParam::Positive => RechallengeResponse::Positive,
        RechallengeParam::Negative => RechallengeResponse::Negative,
        RechallengeParam::NotPerformed => RechallengeResponse::NotPerformed,
        RechallengeParam::Unknown => RechallengeResponse::Unknown,
    }
}

// =============================================================================
// TTO Tool
// =============================================================================

/// Calculate time-to-onset with 6-category classification.
///
/// Computes days between exposure and event onset, classifies into
/// Immediate (<1h), Acute (1-24h), SubAcute (1-7d), Delayed (1-4wk),
/// Latent (1-12mo), or Chronic (>12mo), and returns plausibility score.
pub fn temporal_tto(params: TtoParams) -> Result<CallToolResult, McpError> {
    let result = time_to_onset(&params.exposure_date, &params.event_date);

    match result {
        Some(tto) => {
            let json_val = json!({
                "days": tto.days,
                "category": tto.category.to_string(),
                "plausibility": tto.plausibility,
                "interpretation": match tto.category {
                    nexcore_pv_core::temporal::TtoCategory::Immediate =>
                        "Immediate onset (<1 hour): very strong temporal relationship (e.g., anaphylaxis)",
                    nexcore_pv_core::temporal::TtoCategory::Acute =>
                        "Acute onset (1-24 hours): strong temporal relationship",
                    nexcore_pv_core::temporal::TtoCategory::SubAcute =>
                        "Sub-acute onset (1-7 days): moderate-strong temporal relationship",
                    nexcore_pv_core::temporal::TtoCategory::Delayed =>
                        "Delayed onset (1-4 weeks): moderate temporal relationship",
                    nexcore_pv_core::temporal::TtoCategory::Latent =>
                        "Latent onset (1-12 months): weaker temporal relationship",
                    nexcore_pv_core::temporal::TtoCategory::Chronic =>
                        "Chronic onset (>12 months): weakest temporal relationship (e.g., carcinogenicity)",
                },
            });

            let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
            attach_forensic_meta(&mut res, tto.plausibility, None, "pv_temporal");
            Ok(res)
        }
        None => {
            let json_val = json!({
                "error": "Invalid dates or event precedes exposure",
                "exposure_date": params.exposure_date,
                "event_date": params.event_date,
            });
            Ok(CallToolResult::error(vec![Content::text(
                json_val.to_string(),
            )]))
        }
    }
}

// =============================================================================
// Challenge Tool
// =============================================================================

/// Dechallenge/rechallenge assessment with timing bonuses.
///
/// Evaluates drug withdrawal (dechallenge) and re-introduction (rechallenge)
/// responses. Returns combined causality score and confidence.
/// Faster resolution (<7d dechallenge, <3d rechallenge) adds confidence bonus.
pub fn temporal_challenge(params: ChallengeParams) -> Result<CallToolResult, McpError> {
    let dechallenge = to_dechallenge(&params.dechallenge);
    let rechallenge = to_rechallenge(&params.rechallenge);

    let result = assess_challenge_with_timing(
        dechallenge,
        params.dechallenge_days,
        rechallenge,
        params.rechallenge_days,
    );

    let json_val = json!({
        "dechallenge": format!("{:?}", result.dechallenge),
        "rechallenge": format!("{:?}", result.rechallenge),
        "dechallenge_days": result.dechallenge_days,
        "rechallenge_days": result.rechallenge_days,
        "causality_score": result.causality_score,
        "confidence": result.confidence,
        "interpretation": match result.causality_score {
            s if s >= 3 => "Strong positive: both dechallenge and rechallenge support causality",
            s if s >= 1 => "Positive: evidence supports causal relationship",
            0 => "Neutral: insufficient evidence to determine causality",
            s if s >= -1 => "Weak negative: some evidence against causality",
            _ => "Negative: evidence argues against causal relationship",
        },
    });

    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(
        &mut res,
        result.confidence,
        Some(result.causality_score > 0),
        "pv_temporal",
    );
    Ok(res)
}

// =============================================================================
// Temporal Plausibility Tool
// =============================================================================

/// Unified temporal plausibility score.
///
/// Combines time-to-onset assessment and challenge assessment into an
/// overall temporal plausibility score (0.0-1.0). Optionally checks
/// whether TTO falls within an expected mechanism-based onset range.
pub fn temporal_plausibility_tool(
    params: TemporalPlausibilityParams,
) -> Result<CallToolResult, McpError> {
    // Build TTO from dates or direct days
    let tto = if let Some(days) = params.days_to_onset {
        Some(time_to_onset_days(days))
    } else if let (Some(exp), Some(evt)) = (&params.exposure_date, &params.event_date) {
        time_to_onset(exp, evt)
    } else {
        None
    };

    // Build challenge assessment
    let dechallenge = to_dechallenge(&params.dechallenge);
    let rechallenge = to_rechallenge(&params.rechallenge);
    let has_challenge_data = !matches!(dechallenge, DechallengeResponse::Unknown)
        || !matches!(rechallenge, RechallengeResponse::Unknown);

    let challenge = if has_challenge_data {
        Some(assess_challenge_with_timing(
            dechallenge,
            None,
            rechallenge,
            None,
        ))
    } else {
        None
    };

    // Build expected range
    let expected_range = match (params.expected_min_days, params.expected_max_days) {
        (Some(min), Some(max)) => Some((min, max)),
        _ => None,
    };

    let result = temporal_plausibility(tto, challenge, expected_range);

    let json_val = json!({
        "score": (result.score * 1000.0).round() / 1000.0,
        "within_expected": result.within_expected,
        "tto": result.tto.as_ref().map(|t| json!({
            "days": t.days,
            "category": t.category.to_string(),
            "plausibility": t.plausibility,
        })),
        "challenge": result.challenge.as_ref().map(|c| json!({
            "causality_score": c.causality_score,
            "confidence": c.confidence,
        })),
        "expected_range": result.expected_range.map(|(min, max)| json!({
            "min_days": min,
            "max_days": max,
        })),
        "interpretation": if result.score >= 0.8 {
            "Very high temporal plausibility"
        } else if result.score >= 0.6 {
            "Moderate-high temporal plausibility"
        } else if result.score >= 0.4 {
            "Moderate temporal plausibility"
        } else if result.score >= 0.2 {
            "Low temporal plausibility"
        } else {
            "Very low temporal plausibility"
        },
    });

    let mut res = CallToolResult::success(vec![Content::text(json_val.to_string())]);
    attach_forensic_meta(
        &mut res,
        result.score,
        Some(result.within_expected),
        "pv_temporal",
    );
    Ok(res)
}
