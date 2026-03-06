#![doc = "Disney Loop: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)"]
#![doc = ""]
#![doc = "Forward-only compound discovery pipeline."]
#![doc = "Assess state → reject regression → search for novelty → arrive at new state."]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
use nexcore_dataframe::{Agg, Column, DataFrame, DataFrameError};
use std::path::Path;

/// Errors specific to the Disney Loop pipeline.
#[derive(Debug, nexcore_error::Error)]
#[non_exhaustive]
pub enum DisneyError {
    #[error("dataframe error: {0}")]
    DataFrame(#[from] DataFrameError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("empty pipeline: no records after ingestion")]
    EmptyPipeline,
}

pub type Result<T> = std::result::Result<T, DisneyError>;

pub mod humanize;

/// Stage 2: ∂(¬σ⁻¹) — Anti-Regression Gate
///
/// Filters out any records where `direction == "backward"`.
/// Only forward-moving state survives this gate.
pub fn transform_anti_regression_gate(df: DataFrame) -> Result<DataFrame> {
    tracing::info!(
        stage = "anti-regression-gate",
        expression = "direction != 'backward'",
        "Applying filter: reject regression"
    );
    let filtered = df.filter_by("direction", |v| v.as_str() != Some("backward"))?;

    if filtered.height() == 0 {
        return Err(DisneyError::EmptyPipeline);
    }

    Ok(filtered)
}

/// Stage 3: ∃(ν) — Curiosity Search
///
/// Aggregates novelty by domain: sums `novelty_score` and counts discoveries.
pub fn transform_curiosity_search(df: DataFrame) -> Result<DataFrame> {
    tracing::info!(stage = "curiosity-search", "Aggregating novelty by domain");
    let aggregated = df
        .group_by(&["domain"])?
        .agg(&[Agg::Sum("novelty_score".into()), Agg::Count])?;
    Ok(aggregated)
}

/// Stage 4: ρ(t+1) — New State Sink
///
/// Writes the transformed state to a JSON file. The old state is gone;
/// the new state is all that remains. Forward only.
#[allow(clippy::as_conversions, reason = "DataFrame height fits in u64")]
pub fn sink_new_state(df: DataFrame, output_path: &Path) -> Result<u64> {
    tracing::info!(
        stage = "new-state",
        path = %output_path.display(),
        "Writing new state to JSON"
    );

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let rows = df.height() as u64;
    df.to_json_file(output_path)?;

    tracing::info!(records = rows, "State written successfully");
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn sample_frame() -> Result<DataFrame> {
        Ok(DataFrame::new(vec![
            Column::from_strs(
                "domain",
                &[
                    "signals",
                    "signals",
                    "primitives",
                    "primitives",
                    "regression",
                ],
            ),
            Column::from_strs(
                "direction",
                &["forward", "forward", "forward", "backward", "backward"],
            ),
            Column::from_i64s("novelty_score", vec![10, 20, 15, 5, 0]),
            Column::from_strs("discovery", &["prr", "ror", "sigma", "none", "none"]),
        ])?)
    }

    #[test]
    fn anti_regression_gate_filters_backward() -> Result<()> {
        let df = transform_anti_regression_gate(sample_frame()?)?;
        // Started with 5 rows, 2 are "backward" → 3 remain
        assert_eq!(df.height(), 3);
        Ok(())
    }

    #[test]
    fn curiosity_search_aggregates_by_domain() -> Result<()> {
        use nexcore_dataframe::Scalar;
        // First filter, then aggregate (the real pipeline order)
        let df = transform_curiosity_search(transform_anti_regression_gate(sample_frame()?)?)?;
        // After filtering backward: signals(2 rows) + primitives(1 row) = 2 domains
        assert_eq!(df.height(), 2);
        let sums = df.column("novelty_score_sum")?;
        let counts = df.column("count")?;
        let domains = df.column("domain")?;
        let mut found_signals = false;
        let mut found_primitives = false;
        for i in 0..df.height() {
            match domains.get(i).as_ref().map(|s| s.to_string()).as_deref() {
                Some("signals") => {
                    assert_eq!(
                        sums.get(i),
                        Some(Scalar::Int64(30)),
                        "signals sum must be 30"
                    );
                    assert_eq!(
                        counts.get(i),
                        Some(Scalar::UInt64(2)),
                        "signals count must be 2"
                    );
                    found_signals = true;
                }
                Some("primitives") => {
                    assert_eq!(
                        sums.get(i),
                        Some(Scalar::Int64(15)),
                        "primitives sum must be 15"
                    );
                    assert_eq!(
                        counts.get(i),
                        Some(Scalar::UInt64(1)),
                        "primitives count must be 1"
                    );
                    found_primitives = true;
                }
                _ => {}
            }
        }
        assert!(found_signals, "signals domain must appear in result");
        assert!(found_primitives, "primitives domain must appear in result");
        Ok(())
    }

    #[test]
    fn sink_writes_json_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("state_next.json");
        let input = DataFrame::new(vec![
            Column::from_strs("domain", &["signals"]),
            Column::from_i64s("total_novelty", vec![30]),
            Column::from_i64s("discoveries", vec![2]),
        ])?;
        sink_new_state(input, &path)?;
        assert!(path.exists());
        let mut contents = String::new();
        std::fs::File::open(&path)?.read_to_string(&mut contents)?;
        assert!(contents.contains("signals"));
        Ok(())
    }

    #[test]
    fn anti_regression_gate_rejects_all_backward() -> Result<()> {
        // All records are backward → EmptyPipeline error
        let df = DataFrame::new(vec![
            Column::from_strs("domain", &["signals", "primitives"]),
            Column::from_strs("direction", &["backward", "backward"]),
            Column::from_i64s("novelty_score", vec![5, 0]),
            Column::from_strs("discovery", &["none", "none"]),
        ])?;
        let result = transform_anti_regression_gate(df);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(
                err.to_string().contains("empty pipeline"),
                "Expected EmptyPipeline error, got: {err}"
            );
        }
        Ok(())
    }

    #[test]
    fn full_pipeline_forward_only() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("output/state.json");
        let df = transform_anti_regression_gate(sample_frame()?)?;
        let df = transform_curiosity_search(df)?;
        let rows = sink_new_state(df, &path)?;
        assert_eq!(rows, 2); // 2 forward domains
        Ok(())
    }
}
