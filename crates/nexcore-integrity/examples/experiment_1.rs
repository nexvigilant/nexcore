//! # Experiment 1: KSB Integrity Detection Validation
//!
//! Runs all 20 KSB fixtures × 2 responses = 40 samples through the integrity
//! pipeline and reports per-Bloom accuracy, false positive rate (FPR), and
//! false negative rate (FNR).
//!
//! ## Success Criteria
//! - Overall accuracy ≥ 70%
//! - FPR < 15% (human responses falsely flagged as AI)
//! - Bloom-level trend: higher Bloom = better discrimination
//!
//! ## Run
//! ```bash
//! cargo run -p nexcore-integrity --example experiment_1 --features fixtures
//! ```

use nexcore_integrity::assess_ksb_response;
use nexcore_integrity::classify::Verdict;
use nexcore_integrity::fixtures::{KsbFixture, all_fixtures};

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Experiment 1: KSB Integrity Detection Validation");
    println!("  20 fixtures × 2 responses = 40 samples");
    println!("═══════════════════════════════════════════════════════════════\n");

    let fixtures = all_fixtures();
    let total_fixtures = fixtures.len();

    // Per-Bloom level accumulators
    let mut bloom_stats: std::collections::BTreeMap<u8, BloomStats> =
        std::collections::BTreeMap::new();

    let mut total_correct = 0usize;
    let mut total_samples = 0usize;
    let mut total_fp = 0usize; // Human classified as Generated
    let mut total_fn = 0usize; // Generated classified as Human
    let mut total_human = 0usize;
    let mut total_ai = 0usize;
    let mut errors = Vec::new();

    println!("─── Per-Sample Results ──────────────────────────────────────\n");

    for fixture in &fixtures {
        let stats = bloom_stats
            .entry(fixture.bloom_level)
            .or_insert_with(|| BloomStats::new(fixture.bloom_level));

        // Test human response (expected: Human)
        match assess_ksb_response(
            fixture.human_response,
            fixture.bloom_level,
            Some(fixture.domain_id),
        ) {
            Ok(result) => {
                total_samples += 1;
                total_human += 1;
                stats.human_total += 1;

                let correct = matches!(result.classification.verdict, Verdict::Human);
                if correct {
                    total_correct += 1;
                    stats.human_correct += 1;
                } else {
                    total_fp += 1;
                    stats.false_positives += 1;
                    println!(
                        "  FP  {} L{} human: p={:.3} hill={:.3} threshold={:.3}",
                        fixture.ksb_id,
                        fixture.bloom_level,
                        result.classification.probability,
                        result.features.hill_score,
                        result.threshold,
                    );
                }
            }
            Err(e) => {
                errors.push(format!("{} human: {e}", fixture.ksb_id));
            }
        }

        // Test AI response (expected: Generated)
        match assess_ksb_response(
            fixture.ai_response,
            fixture.bloom_level,
            Some(fixture.domain_id),
        ) {
            Ok(result) => {
                total_samples += 1;
                total_ai += 1;
                stats.ai_total += 1;

                let correct = matches!(result.classification.verdict, Verdict::Generated);
                if correct {
                    total_correct += 1;
                    stats.ai_correct += 1;
                } else {
                    total_fn += 1;
                    stats.false_negatives += 1;
                    println!(
                        "  FN  {} L{} ai:    p={:.3} hill={:.3} threshold={:.3}",
                        fixture.ksb_id,
                        fixture.bloom_level,
                        result.classification.probability,
                        result.features.hill_score,
                        result.threshold,
                    );
                }
            }
            Err(e) => {
                errors.push(format!("{} ai: {e}", fixture.ksb_id));
            }
        }
    }

    // ── Summary ──────────────────────────────────────────────────────

    println!("\n─── Per-Bloom Results ──────────────────────────────────────\n");
    println!(
        "  {:>5}  {:>8}  {:>8}  {:>5}  {:>5}  {:>8}",
        "Bloom", "H-Acc", "AI-Acc", "FP", "FN", "Overall"
    );
    println!(
        "  {:─>5}  {:─>8}  {:─>8}  {:─>5}  {:─>5}  {:─>8}",
        "", "", "", "", "", ""
    );

    for (level, stats) in &bloom_stats {
        let h_acc = if stats.human_total > 0 {
            stats.human_correct as f64 / stats.human_total as f64
        } else {
            0.0
        };
        let ai_acc = if stats.ai_total > 0 {
            stats.ai_correct as f64 / stats.ai_total as f64
        } else {
            0.0
        };
        let overall_total = stats.human_total + stats.ai_total;
        let overall_correct = stats.human_correct + stats.ai_correct;
        let overall_acc = if overall_total > 0 {
            overall_correct as f64 / overall_total as f64
        } else {
            0.0
        };

        println!(
            "  L{:<4}  {:>7.1}%  {:>7.1}%  {:>5}  {:>5}  {:>7.1}%",
            level,
            h_acc * 100.0,
            ai_acc * 100.0,
            stats.false_positives,
            stats.false_negatives,
            overall_acc * 100.0,
        );
    }

    println!("\n─── Aggregate Metrics ──────────────────────────────────────\n");

    let accuracy = if total_samples > 0 {
        total_correct as f64 / total_samples as f64
    } else {
        0.0
    };
    let fpr = if total_human > 0 {
        total_fp as f64 / total_human as f64
    } else {
        0.0
    };
    let fnr = if total_ai > 0 {
        total_fn as f64 / total_ai as f64
    } else {
        0.0
    };

    println!("  Fixtures:       {total_fixtures}");
    println!("  Samples:        {total_samples} ({total_human} human + {total_ai} AI)");
    println!("  Correct:        {total_correct}");
    println!("  Accuracy:       {:.1}%", accuracy * 100.0);
    println!("  False Positive: {total_fp} (FPR = {:.1}%)", fpr * 100.0);
    println!("  False Negative: {total_fn} (FNR = {:.1}%)", fnr * 100.0);

    if !errors.is_empty() {
        println!("\n─── Errors ────────────────────────────────────────────────\n");
        for e in &errors {
            println!("  ERR  {e}");
        }
    }

    println!("\n─── Verdict ───────────────────────────────────────────────\n");

    let pass_accuracy = accuracy >= 0.70;
    let pass_fpr = fpr < 0.15;

    println!(
        "  Accuracy ≥ 70%:  {} ({:.1}%)",
        if pass_accuracy { "PASS" } else { "FAIL" },
        accuracy * 100.0
    );
    println!(
        "  FPR < 15%:       {} ({:.1}%)",
        if pass_fpr { "PASS" } else { "FAIL" },
        fpr * 100.0
    );
    println!(
        "\n  Overall:         {}",
        if pass_accuracy && pass_fpr {
            "EXPERIMENT PASSED"
        } else {
            "EXPERIMENT NEEDS TUNING"
        }
    );
    println!("\n═══════════════════════════════════════════════════════════════");
}

struct BloomStats {
    #[allow(dead_code)]
    level: u8,
    human_total: usize,
    human_correct: usize,
    ai_total: usize,
    ai_correct: usize,
    false_positives: usize,
    false_negatives: usize,
}

impl BloomStats {
    fn new(level: u8) -> Self {
        Self {
            level,
            human_total: 0,
            human_correct: 0,
            ai_total: 0,
            ai_correct: 0,
            false_positives: 0,
            false_negatives: 0,
        }
    }
}
