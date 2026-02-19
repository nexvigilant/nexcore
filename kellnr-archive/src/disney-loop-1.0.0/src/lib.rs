#![doc = "Disney Loop: ρ(t) → ∂(¬σ⁻¹) → ∃(ν) → ρ(t+1)"]
#![doc = ""]
#![doc = "Forward-only compound discovery pipeline."]
#![doc = "Assess state → reject regression → search for novelty → arrive at new state."]
#![forbid(unsafe_code)]

use polars::prelude::*;
use std::path::Path;

/// Errors specific to the Disney Loop pipeline.
#[derive(Debug, thiserror::Error)]
pub enum DisneyError {
    #[error("polars error: {0}")]
    Polars(#[from] PolarsError),

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
pub fn transform_anti_regression_gate(df: LazyFrame) -> Result<LazyFrame> {
    tracing::info!(
        stage = "anti-regression-gate",
        expression = "direction != 'backward'",
        "Applying filter: reject regression"
    );
    let filtered = df.filter(col("direction").neq(lit("backward")));
    Ok(filtered)
}

/// Stage 3: ∃(ν) — Curiosity Search
///
/// Aggregates novelty by domain: sums `novelty_score` and counts discoveries.
pub fn transform_curiosity_search(df: LazyFrame) -> Result<LazyFrame> {
    tracing::info!(stage = "curiosity-search", "Aggregating novelty by domain");
    let aggregated = df.group_by([col("domain")]).agg([
        col("novelty_score").sum().alias("total_novelty"),
        col("discovery").count().alias("discoveries"),
    ]);
    Ok(aggregated)
}

/// Stage 4: ρ(t+1) — New State Sink
///
/// Writes the transformed state to a JSON file. The old state is gone;
/// the new state is all that remains. Forward only.
pub fn sink_new_state(df: LazyFrame, output_path: &Path) -> Result<u64> {
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

    let mut collected = df.collect()?;
    let rows = collected.height() as u64;

    let mut file = std::fs::File::create(output_path)?;
    JsonWriter::new(&mut file)
        .with_json_format(JsonFormat::Json)
        .finish(&mut collected)?;

    tracing::info!(records = rows, "State written successfully");
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn sample_frame() -> LazyFrame {
        df!(
            "domain" => ["signals", "signals", "primitives", "primitives", "regression"],
            "direction" => ["forward", "forward", "forward", "backward", "backward"],
            "novelty_score" => [10i64, 20, 15, 5, 0],
            "discovery" => ["prr", "ror", "sigma", "none", "none"]
        )
        .unwrap_or_else(|_| DataFrame::empty())
        .lazy()
    }

    #[test]
    fn anti_regression_gate_filters_backward() {
        let result = transform_anti_regression_gate(sample_frame());
        assert!(result.is_ok());
        let collected = result.and_then(|lf| lf.collect().map_err(DisneyError::from));
        assert!(collected.is_ok());
        if let Ok(df) = collected {
            // Started with 5 rows, 2 are "backward" → 3 remain
            assert_eq!(df.height(), 3);
        }
    }

    #[test]
    fn curiosity_search_aggregates_by_domain() {
        // First filter, then aggregate (the real pipeline order)
        let filtered = transform_anti_regression_gate(sample_frame());
        assert!(filtered.is_ok());
        let result = filtered.and_then(transform_curiosity_search);
        assert!(result.is_ok());
        let collected = result.and_then(|lf| lf.collect().map_err(DisneyError::from));
        assert!(collected.is_ok());
        if let Ok(df) = collected {
            // After filtering backward: signals(2 rows) + primitives(1 row) = 2 domains
            assert_eq!(df.height(), 2);
        }
    }

    #[test]
    fn sink_writes_json_file() {
        let dir = tempfile::tempdir();
        assert!(dir.is_ok());
        if let Ok(dir) = dir {
            let path = dir.path().join("state_next.json");
            let input = df!(
                "domain" => ["signals"],
                "total_novelty" => [30i64],
                "discoveries" => [2u32]
            )
            .unwrap_or_else(|_| DataFrame::empty())
            .lazy();

            let result = sink_new_state(input, &path);
            assert!(result.is_ok());
            assert!(path.exists());

            // Verify the file has content
            let mut contents = String::new();
            let read_result =
                std::fs::File::open(&path).and_then(|mut f| f.read_to_string(&mut contents));
            assert!(read_result.is_ok());
            assert!(contents.contains("signals"));
        }
    }

    #[test]
    fn full_pipeline_forward_only() {
        let dir = tempfile::tempdir();
        assert!(dir.is_ok());
        if let Ok(dir) = dir {
            let path = dir.path().join("output/state.json");

            let result = transform_anti_regression_gate(sample_frame())
                .and_then(transform_curiosity_search)
                .and_then(|lf| sink_new_state(lf, &path));

            assert!(result.is_ok());
            if let Ok(rows) = result {
                assert_eq!(rows, 2); // 2 forward domains
            }
        }
    }
}
