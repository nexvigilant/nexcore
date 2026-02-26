//! # Humanization Loop Stages
//!
//! Implements the Disney Loop for text humanization:
//! ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)
//!
//! 1. ρ(t): Current state (robotic text)
//! 2. ∂(¬σ⁻¹): Anti-regression gate — threshold filter on `prob_generated`
//! 3. ∃(ν): Phrasing discovery — group-by-id aggregation (STUB: LLM integration pending)
//! 4. ρ(t+1): New state (humanized text)

use crate::Result;
use nexcore_dataframe::{Agg, DataFrame};

/// Stage 2: ∂(¬σ⁻¹) — Humanization Gate
///
/// Filters rows where `prob_generated` exceeds `threshold`, retaining
/// only text that reads as sufficiently human. `threshold` is clamped
/// to `[0.0, 1.0]`.
///
/// When a real antitransformer scorer is integrated, it will feed the
/// `prob_generated` column that this gate consumes.
pub fn transform_humanization_gate(df: DataFrame, threshold: f64) -> Result<DataFrame> {
    let threshold = threshold.clamp(0.0, 1.0);
    tracing::info!(
        stage = "humanization-gate",
        threshold = threshold,
        "Applying anti-regression filter for AI-generated text"
    );

    // Filter: keep if probability_generated < threshold
    let filtered = df.filter_by("prob_generated", |v| {
        v.as_f64().is_some_and(|p| p < threshold)
    })?;

    Ok(filtered)
}

/// Stage 3: ∃(ν) — Phrasing Discovery (STUB)
///
/// Groups rows by `id`, keeping the first `text` value and the minimum
/// `prob_generated` score per id. Output columns: `id`, `text_first`,
/// `prob_generated_min`.
///
/// STUB: when nexcore-transform integration lands, this stage will use
/// concept identification and phrasing suggestion rather than a bare
/// first-value aggregation.
pub fn transform_phrasing_discovery(df: DataFrame) -> Result<DataFrame> {
    tracing::info!(
        stage = "phrasing-discovery",
        "Searching for natural phrasing alternatives"
    );

    // Aggregate by id — first text, min prob_generated
    let aggregated = df
        .group_by(&["id"])?
        .agg(&[Agg::First("text".into()), Agg::Min("prob_generated".into())])?;

    Ok(aggregated)
}

/// Run a single humanization pass on text.
#[allow(dead_code, reason = "placeholder for LLM integration")]
pub fn humanize_text(text: &str) -> String {
    // This would ideally call an LLM to rephrase.
    // For the autonomous loop, we use it as a placeholder for the ∃(ν) phase.
    format!("Refactored: {}", text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_dataframe::Column;

    fn humanize_frame() -> crate::Result<DataFrame> {
        Ok(DataFrame::new(vec![
            Column::from_strs("id", &["a", "a", "b", "b"]),
            Column::from_strs(
                "text",
                &["hello world", "greetings earth", "foo bar", "baz qux"],
            ),
            Column::from_f64s("prob_generated", vec![0.3, 0.8, 0.1, 0.9]),
        ])?)
    }

    #[test]
    fn humanization_gate_filters_above_threshold() -> crate::Result<()> {
        let df = transform_humanization_gate(humanize_frame()?, 0.5)?;
        // prob_generated: 0.3, 0.8, 0.1, 0.9 — only 0.3 and 0.1 < 0.5
        assert_eq!(df.height(), 2);
        Ok(())
    }

    #[test]
    fn humanization_gate_clamps_threshold() -> crate::Result<()> {
        // threshold > 1.0 gets clamped to 1.0 — all pass
        let df = transform_humanization_gate(humanize_frame()?, 2.0)?;
        assert_eq!(df.height(), 4);

        // threshold < 0.0 gets clamped to 0.0 — none pass
        let df = transform_humanization_gate(humanize_frame()?, -1.0)?;
        assert_eq!(df.height(), 0);
        Ok(())
    }

    #[test]
    fn phrasing_discovery_aggregates_by_id() -> crate::Result<()> {
        use nexcore_dataframe::Scalar;
        let df = transform_phrasing_discovery(humanize_frame()?)?;
        // 2 unique ids: "a" and "b"
        assert_eq!(df.height(), 2);

        let ids = df.column("id")?;
        let mins = df.column("prob_generated_min")?;
        // text_first must exist
        let _text = df.column("text_first")?;

        let mut found_a = false;
        let mut found_b = false;
        for i in 0..df.height() {
            match ids.get(i).as_ref().map(|s| s.to_string()).as_deref() {
                Some("a") => {
                    // min(0.3, 0.8) = 0.3
                    if let Some(Scalar::Float64(v)) = mins.get(i) {
                        assert!(
                            (v - 0.3_f64).abs() < 1e-9,
                            "id=a min prob_generated must be 0.3, got {v}"
                        );
                    }
                    found_a = true;
                }
                Some("b") => {
                    // min(0.1, 0.9) = 0.1
                    if let Some(Scalar::Float64(v)) = mins.get(i) {
                        assert!(
                            (v - 0.1_f64).abs() < 1e-9,
                            "id=b min prob_generated must be 0.1, got {v}"
                        );
                    }
                    found_b = true;
                }
                _ => {}
            }
        }
        assert!(found_a, "id=a must appear in result");
        assert!(found_b, "id=b must appear in result");
        Ok(())
    }

    #[test]
    fn humanize_text_stub_is_non_empty_for_non_empty_input() {
        // humanize_text() is a placeholder for LLM integration.
        // This test documents the STUB CONTRACT only: any non-empty input
        // produces a non-empty, non-identical output string.
        // When real LLM integration lands, this test must be replaced with
        // assertions against actual humanization quality metrics.
        let output = humanize_text("test input");
        assert!(!output.is_empty(), "stub must return non-empty string");
        assert_ne!(
            output, "test input",
            "stub must transform input, not echo it"
        );
    }
}
