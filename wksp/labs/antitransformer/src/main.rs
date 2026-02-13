//! # Antitransformer CLI
//!
//! Reads JSON text samples from stdin, runs 5-feature detection pipeline,
//! writes JSON verdicts to stdout.
//!
//! ## Input Format
//! ```json
//! [{"id": "h1", "text": "...", "label": "human"}]
//! ```
//!
//! ## Output Format
//! ```json
//! [{"id": "h1", "verdict": "human", "probability": 0.12, "confidence": 0.76, "features": {...}}]
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use tracing::{error, info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use antitransformer::{aggregation, burstiness, classify, entropy, perplexity, tokenize, zipf};

/// Antitransformer: AI text detector via statistical fingerprints.
#[derive(Parser, Debug)]
#[command(name = "antitransformer")]
#[command(version = "0.1.0")]
#[command(about = "Detect AI-generated text through statistical fingerprints")]
struct Args {
    /// Verbose output (-v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Dry run — validate configuration only.
    #[arg(long)]
    dry_run: bool,

    /// Custom decision threshold (default: 0.5).
    #[arg(long, default_value = "0.5")]
    threshold: f64,

    /// Entropy window size (default: 50).
    #[arg(long, default_value = "50")]
    window_size: usize,
}

/// Input sample format.
#[derive(Debug, Deserialize)]
struct InputSample {
    id: String,
    text: String,
    #[serde(default)]
    label: Option<String>,
}

/// Feature detail for output transparency.
#[derive(Debug, Serialize)]
struct FeatureDetail {
    zipf_alpha: f64,
    zipf_deviation: f64,
    entropy_std: f64,
    burstiness: f64,
    perplexity_var: f64,
    ttr: f64,
    ttr_deviation: f64,
    normalized: [f64; 5],
    beer_lambert: f64,
    composite: f64,
    hill_score: f64,
}

/// Output verdict format.
#[derive(Debug, Serialize)]
struct OutputVerdict {
    id: String,
    verdict: String,
    probability: f64,
    confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    correct: Option<bool>,
    features: FeatureDetail,
}

#[derive(Debug, Default)]
struct PipelineStats {
    records_processed: u64,
    human_count: u64,
    generated_count: u64,
    correct_count: u64,
    labeled_count: u64,
    duration_secs: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let level = match args.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_writer(io::stderr),
        )
        .with(tracing_subscriber::filter::LevelFilter::from_level(level))
        .init();

    info!(
        pipeline = "antitransformer",
        version = "0.1.0",
        "Starting pipeline"
    );

    if args.dry_run {
        info!("Dry run — configuration valid");
        return Ok(());
    }

    let result = run_pipeline(args.threshold, args.window_size).await;
    match result {
        Ok(stats) => {
            info!(
                processed = stats.records_processed,
                human = stats.human_count,
                generated = stats.generated_count,
                duration = stats.duration_secs,
                "Pipeline completed"
            );
            if stats.labeled_count > 0 {
                let accuracy = stats.correct_count as f64 / stats.labeled_count as f64;
                eprintln!(
                    "Accuracy: {}/{} ({:.1}%)",
                    stats.correct_count,
                    stats.labeled_count,
                    accuracy * 100.0
                );
            }
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Pipeline failed");
            Err(e)
        }
    }
}

#[instrument(skip_all)]
async fn run_pipeline(threshold: f64, window_size: usize) -> Result<PipelineStats> {
    let start = std::time::Instant::now();
    let mut stats = PipelineStats::default();

    // Read JSON from stdin
    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .collect();

    let json_str = if lines.len() == 1 && lines[0].trim().starts_with('[') {
        lines[0].clone()
    } else {
        format!("[{}]", lines.join(","))
    };

    let samples: Vec<InputSample> =
        serde_json::from_str(&json_str).context("Failed to parse JSON input")?;

    info!(samples = samples.len(), "Ingested samples");

    let mut verdicts = Vec::new();

    for sample in &samples {
        stats.records_processed += 1;

        // Stage 1: Tokenize (σ)
        let token_stats = tokenize::tokenize(&sample.text);
        info!(
            id = %sample.id,
            tokens = token_stats.total_tokens,
            unique = token_stats.unique_tokens,
            ttr = token_stats.ttr,
            "Tokenized"
        );

        if token_stats.total_tokens < 10 {
            // Too short for reliable analysis
            verdicts.push(OutputVerdict {
                id: sample.id.clone(),
                verdict: "insufficient_data".to_string(),
                probability: 0.0,
                confidence: 0.0,
                label: sample.label.clone(),
                correct: None,
                features: FeatureDetail {
                    zipf_alpha: 0.0,
                    zipf_deviation: 0.0,
                    entropy_std: 0.0,
                    burstiness: 0.0,
                    perplexity_var: 0.0,
                    ttr: token_stats.ttr,
                    ttr_deviation: 0.0,
                    normalized: [0.0; 5],
                    beer_lambert: 0.0,
                    composite: 0.0,
                    hill_score: 0.0,
                },
            });
            continue;
        }

        // Stage 2: Zipf analysis (κ + N)
        let zipf_result = zipf::zipf_analysis(&token_stats.frequencies);
        info!(id = %sample.id, alpha = zipf_result.alpha, r2 = zipf_result.r_squared, "Zipf");

        // Stage 3: Entropy profile (Σ + N)
        let entropy_result =
            entropy::entropy_profile(&token_stats.tokens, window_size, window_size / 2);
        info!(id = %sample.id, mean = entropy_result.mean, std = entropy_result.std_dev, "Entropy");

        // Stage 4: Burstiness (ν + ∂)
        let burst_result =
            burstiness::burstiness_analysis(&token_stats.tokens, &token_stats.frequencies);
        info!(id = %sample.id, burstiness = burst_result.coefficient, "Burstiness");

        // Stage 5: Perplexity variance (ν + κ)
        let perp_result = perplexity::perplexity_variance(&sample.text);
        info!(id = %sample.id, var = perp_result.variance, sentences = perp_result.sentence_count, "Perplexity");

        // Stage 6: Aggregation (Σ + ρ) — Beer-Lambert + Hill
        let raw_features = aggregation::RawFeatures {
            zipf_deviation: zipf_result.deviation,
            entropy_std: entropy_result.std_dev,
            burstiness: burst_result.coefficient.max(0.0), // clamp negative burstiness
            perplexity_var: perp_result.variance,
            ttr_deviation: tokenize::ttr_deviation(token_stats.ttr),
        };
        let agg_result = aggregation::aggregate(&raw_features);
        info!(
            id = %sample.id,
            composite = agg_result.composite,
            hill = agg_result.hill_score,
            "Aggregated"
        );

        // Stage 7: Classification (∂ + →) — Arrhenius gate
        let classification = classify::classify_with_threshold(agg_result.hill_score, threshold);
        info!(
            id = %sample.id,
            verdict = %classification.verdict,
            prob = classification.probability,
            conf = classification.confidence,
            "Classified"
        );

        match classification.verdict {
            classify::Verdict::Human => stats.human_count += 1,
            classify::Verdict::Generated => stats.generated_count += 1,
        }

        // Check against label if provided
        let correct = sample.label.as_ref().map(|label| {
            let is_correct = match classification.verdict {
                classify::Verdict::Human => label == "human",
                classify::Verdict::Generated => {
                    label == "generated" || label == "ai" || label == "llm"
                }
            };
            if is_correct {
                stats.correct_count += 1;
            }
            stats.labeled_count += 1;
            is_correct
        });

        verdicts.push(OutputVerdict {
            id: sample.id.clone(),
            verdict: classification.verdict.to_string(),
            probability: classification.probability,
            confidence: classification.confidence,
            label: sample.label.clone(),
            correct,
            features: FeatureDetail {
                zipf_alpha: zipf_result.alpha,
                zipf_deviation: zipf_result.deviation,
                entropy_std: entropy_result.std_dev,
                burstiness: burst_result.coefficient,
                perplexity_var: perp_result.variance,
                ttr: token_stats.ttr,
                ttr_deviation: tokenize::ttr_deviation(token_stats.ttr),
                normalized: agg_result.normalized,
                beer_lambert: agg_result.beer_lambert_score,
                composite: agg_result.composite,
                hill_score: agg_result.hill_score,
            },
        });
    }

    // Write JSON to stdout
    let output = serde_json::to_string_pretty(&verdicts).context("Failed to serialize output")?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}").context("Failed to write output")?;

    stats.duration_secs = start.elapsed().as_secs_f64();
    Ok(stats)
}
