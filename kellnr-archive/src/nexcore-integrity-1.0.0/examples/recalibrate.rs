//! # Recalibration Simulation
//!
//! Analyzes score distributions across all 40 samples and sweeps threshold
//! space to find optimal per-Bloom decision boundaries.
//!
//! ## Simulations
//! 1. **Score Distribution**: Raw probability/hill_score for human vs AI per Bloom level
//! 2. **Flat Threshold Sweep**: Single threshold across all levels (0.50-0.80, step 0.01)
//! 3. **Per-Bloom Youden's J**: Optimal cutoff per level via sensitivity+specificity-1
//! 4. **Preset Candidates**: Test 5 candidate presets against all 40 samples
//! 5. **Winner Application**: Report the best preset for bloom.rs
//!
//! ```bash
//! cargo run -p nexcore-integrity --example recalibrate --features fixtures
//! ```

use nexcore_integrity::assessment::{self, AssessmentContext, FeatureReport, IntegrityAssessment};
use nexcore_integrity::bloom::BloomThresholds;
use nexcore_integrity::classify::{self, Verdict};
use nexcore_integrity::fixtures::{KsbFixture, all_fixtures};

/// Collected score for one sample.
struct Sample {
    ksb_id: &'static str,
    bloom_level: u8,
    is_ai: bool,
    hill_score: f64,
    probability: f64,
    composite: f64,
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Recalibration Simulation");
    println!("═══════════════════════════════════════════════════════════════\n");

    // ── Phase 1: Collect raw scores ─────────────────────────────────────
    let fixtures = all_fixtures();
    let mut samples = Vec::new();
    let mut errors = Vec::new();

    for fixture in &fixtures {
        // Use a high threshold (0.99) so the classification doesn't matter —
        // we just want the raw scores.
        let ctx = match AssessmentContext::new(fixture.bloom_level) {
            Ok(c) => c.with_domain(fixture.domain_id).with_threshold(0.99),
            Err(e) => {
                errors.push(format!("{}: {e}", fixture.ksb_id));
                continue;
            }
        };

        // Human response
        if let Ok(result) = assessment::assess_text(fixture.human_response, &ctx) {
            samples.push(Sample {
                ksb_id: fixture.ksb_id,
                bloom_level: fixture.bloom_level,
                is_ai: false,
                hill_score: result.features.hill_score,
                probability: result.classification.probability,
                composite: result.features.composite_score,
            });
        }

        // AI response
        if let Ok(result) = assessment::assess_text(fixture.ai_response, &ctx) {
            samples.push(Sample {
                ksb_id: fixture.ksb_id,
                bloom_level: fixture.bloom_level,
                is_ai: true,
                hill_score: result.features.hill_score,
                probability: result.classification.probability,
                composite: result.features.composite_score,
            });
        }
    }

    if !errors.is_empty() {
        println!("  Errors: {errors:?}\n");
    }

    // ── Phase 2: Score Distribution ─────────────────────────────────────
    println!("─── 1. Score Distribution ─────────────────────────────────\n");
    println!(
        "  {:>6}  {:>5}  {:>10}  {:>10}  {:>10}  {:>10}",
        "KSB", "Bloom", "Type", "Composite", "Hill", "Prob"
    );
    println!(
        "  {:─>6}  {:─>5}  {:─>10}  {:─>10}  {:─>10}  {:─>10}",
        "", "", "", "", "", ""
    );

    for s in &samples {
        println!(
            "  {:>6}  L{:<4}  {:>10}  {:>10.4}  {:>10.4}  {:>10.4}",
            s.ksb_id.split('-').last().unwrap_or(s.ksb_id),
            s.bloom_level,
            if s.is_ai { "AI" } else { "Human" },
            s.composite,
            s.hill_score,
            s.probability,
        );
    }

    // Summary stats per type
    let human_probs: Vec<f64> = samples
        .iter()
        .filter(|s| !s.is_ai)
        .map(|s| s.probability)
        .collect();
    let ai_probs: Vec<f64> = samples
        .iter()
        .filter(|s| s.is_ai)
        .map(|s| s.probability)
        .collect();

    println!(
        "\n  Human probability range: [{:.4}, {:.4}] mean={:.4}",
        human_probs.iter().cloned().fold(f64::INFINITY, f64::min),
        human_probs
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max),
        human_probs.iter().sum::<f64>() / human_probs.len() as f64,
    );
    println!(
        "  AI    probability range: [{:.4}, {:.4}] mean={:.4}",
        ai_probs.iter().cloned().fold(f64::INFINITY, f64::min),
        ai_probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ai_probs.iter().sum::<f64>() / ai_probs.len() as f64,
    );

    let human_hills: Vec<f64> = samples
        .iter()
        .filter(|s| !s.is_ai)
        .map(|s| s.hill_score)
        .collect();
    let ai_hills: Vec<f64> = samples
        .iter()
        .filter(|s| s.is_ai)
        .map(|s| s.hill_score)
        .collect();
    println!(
        "\n  Human hill_score range:  [{:.4}, {:.4}] mean={:.4}",
        human_hills.iter().cloned().fold(f64::INFINITY, f64::min),
        human_hills
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max),
        human_hills.iter().sum::<f64>() / human_hills.len() as f64,
    );
    println!(
        "  AI    hill_score range:  [{:.4}, {:.4}] mean={:.4}",
        ai_hills.iter().cloned().fold(f64::INFINITY, f64::min),
        ai_hills.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ai_hills.iter().sum::<f64>() / ai_hills.len() as f64,
    );

    // Per-Bloom stats
    println!("\n  Per-Bloom probability summary:");
    println!(
        "  {:>5}  {:>12}  {:>12}  {:>12}  {:>12}  {:>5}",
        "Bloom", "H_min", "H_max", "AI_min", "AI_max", "Sep?"
    );
    for level in 1..=7u8 {
        let h: Vec<f64> = samples
            .iter()
            .filter(|s| s.bloom_level == level && !s.is_ai)
            .map(|s| s.probability)
            .collect();
        let a: Vec<f64> = samples
            .iter()
            .filter(|s| s.bloom_level == level && s.is_ai)
            .map(|s| s.probability)
            .collect();
        if h.is_empty() && a.is_empty() {
            continue;
        }
        let h_min = h.iter().cloned().fold(f64::INFINITY, f64::min);
        let h_max = h.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let a_min = a.iter().cloned().fold(f64::INFINITY, f64::min);
        let a_max = a.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let separable = if !h.is_empty() && !a.is_empty() {
            if h_max < a_min { "YES" } else { "OLAP" }
        } else {
            "N/A"
        };
        println!(
            "  L{:<4}  {:>12.4}  {:>12.4}  {:>12.4}  {:>12.4}  {:>5}",
            level,
            if h.is_empty() { f64::NAN } else { h_min },
            if h.is_empty() { f64::NAN } else { h_max },
            if a.is_empty() { f64::NAN } else { a_min },
            if a.is_empty() { f64::NAN } else { a_max },
            separable,
        );
    }

    // ── Phase 3: Flat Threshold Sweep ───────────────────────────────────
    println!("\n─── 2. Flat Threshold Sweep (all levels same) ──────────────\n");
    println!(
        "  {:>9}  {:>8}  {:>8}  {:>5}  {:>5}  {:>8}  {:>8}",
        "Threshold", "Accuracy", "F1", "FP", "FN", "FPR", "FNR"
    );
    println!(
        "  {:─>9}  {:─>8}  {:─>8}  {:─>5}  {:─>5}  {:─>8}  {:─>8}",
        "", "", "", "", "", "", ""
    );

    let mut best_flat_threshold = 0.50;
    let mut best_flat_f1 = 0.0;

    for t_pct in 50..=80 {
        let threshold = t_pct as f64 / 100.0;
        let (acc, f1, fp, fn_, fpr, fnr) = evaluate_flat(&samples, threshold);
        let marker = if f1 > best_flat_f1 {
            best_flat_f1 = f1;
            best_flat_threshold = threshold;
            " ◄"
        } else {
            ""
        };
        if t_pct % 2 == 0 || f1 >= best_flat_f1 - 0.01 {
            println!(
                "  {:>9.2}  {:>7.1}%  {:>8.3}  {:>5}  {:>5}  {:>7.1}%  {:>7.1}%{}",
                threshold,
                acc * 100.0,
                f1,
                fp,
                fn_,
                fpr * 100.0,
                fnr * 100.0,
                marker
            );
        }
    }
    println!(
        "\n  Best flat threshold: {:.2} (F1 = {:.3})",
        best_flat_threshold, best_flat_f1
    );

    // ── Phase 4: Per-Bloom Youden's J ───────────────────────────────────
    println!("\n─── 3. Per-Bloom Optimal Threshold (Youden's J) ───────────\n");
    println!(
        "  {:>5}  {:>9}  {:>8}  {:>8}  {:>8}  {:>8}",
        "Bloom", "Threshold", "Sens", "Spec", "J", "Acc"
    );

    let mut optimal_thresholds = [0.0f64; 7];

    for level in 1..=7u8 {
        let level_samples: Vec<&Sample> =
            samples.iter().filter(|s| s.bloom_level == level).collect();
        if level_samples.is_empty() {
            continue;
        }

        let mut best_j = -1.0f64;
        let mut best_t = 0.65;
        let mut best_sens = 0.0;
        let mut best_spec = 0.0;
        let mut best_acc = 0.0;

        for t_pct in 50..=80 {
            let threshold = t_pct as f64 / 100.0;
            let (sens, spec, acc) = evaluate_level(&level_samples, threshold);
            let j = sens + spec - 1.0;
            if j > best_j {
                best_j = j;
                best_t = threshold;
                best_sens = sens;
                best_spec = spec;
                best_acc = acc;
            }
        }

        // Also try finer resolution around best
        let search_min = ((best_t - 0.05) * 100.0) as i32;
        let search_max = ((best_t + 0.05) * 100.0) as i32;
        for t_pct_fine in search_min..=search_max {
            let threshold = t_pct_fine as f64 / 100.0;
            if threshold < 0.5 || threshold > 0.85 {
                continue;
            }
            let (sens, spec, acc) = evaluate_level(&level_samples, threshold);
            let j = sens + spec - 1.0;
            if j > best_j {
                best_j = j;
                best_t = threshold;
                best_sens = sens;
                best_spec = spec;
                best_acc = acc;
            }
        }

        optimal_thresholds[(level - 1) as usize] = best_t;
        println!(
            "  L{:<4}  {:>9.2}  {:>7.1}%  {:>7.1}%  {:>8.3}  {:>7.1}%",
            level,
            best_t,
            best_sens * 100.0,
            best_spec * 100.0,
            best_j,
            best_acc * 100.0,
        );
    }

    // Ensure monotonically decreasing (or equal)
    for i in 1..7 {
        if optimal_thresholds[i] > optimal_thresholds[i - 1] {
            optimal_thresholds[i] = optimal_thresholds[i - 1];
        }
    }

    println!("\n  Youden-optimal thresholds (monotonized):");
    println!(
        "    {:?}",
        optimal_thresholds.map(|t| (t * 100.0).round() / 100.0)
    );

    // ── Phase 5: Preset Candidates ──────────────────────────────────────
    println!("\n─── 4. Preset Candidate Evaluation ─────────────────────────\n");

    let candidates: Vec<(&str, [f64; 7])> = vec![
        (
            "original_pv_edu",
            [0.70, 0.65, 0.55, 0.50, 0.40, 0.35, 0.30],
        ),
        ("flat_best", [best_flat_threshold; 7]),
        ("youden_optimal", optimal_thresholds),
        ("gradual_high", [0.72, 0.70, 0.68, 0.67, 0.66, 0.65, 0.64]),
        ("gradual_mid", [0.70, 0.69, 0.67, 0.66, 0.65, 0.64, 0.63]),
        ("conservative", [0.75, 0.73, 0.71, 0.69, 0.68, 0.67, 0.66]),
        ("mild_slope", [0.72, 0.70, 0.68, 0.65, 0.63, 0.62, 0.60]),
    ];

    println!(
        "  {:>20}  {:>8}  {:>8}  {:>5}  {:>5}  {:>8}  {:>8}",
        "Preset", "Accuracy", "F1", "FP", "FN", "FPR", "FNR"
    );
    println!(
        "  {:─>20}  {:─>8}  {:─>8}  {:─>5}  {:─>5}  {:─>8}  {:─>8}",
        "", "", "", "", "", "", ""
    );

    let mut best_preset_name = "";
    let mut best_preset_f1 = 0.0;
    let mut best_preset_thresholds = [0.0; 7];

    for (name, thresholds) in &candidates {
        let (acc, f1, fp, fn_, fpr, fnr) = evaluate_preset(&samples, thresholds);
        let marker = if f1 > best_preset_f1 {
            best_preset_f1 = f1;
            best_preset_name = name;
            best_preset_thresholds = *thresholds;
            " ◄ BEST"
        } else {
            ""
        };
        println!(
            "  {:>20}  {:>7.1}%  {:>8.3}  {:>5}  {:>5}  {:>7.1}%  {:>7.1}%{}",
            name,
            acc * 100.0,
            f1,
            fp,
            fn_,
            fpr * 100.0,
            fnr * 100.0,
            marker
        );
    }

    // ── Phase 6: Detailed winner breakdown ──────────────────────────────
    println!("\n─── 5. Winner Breakdown ────────────────────────────────────\n");
    println!("  Best preset: {best_preset_name}");
    println!(
        "  Thresholds:  {:?}",
        best_preset_thresholds.map(|t| (t * 100.0).round() / 100.0)
    );
    println!("  F1 Score:    {:.3}", best_preset_f1);

    println!("\n  Per-Bloom detail:");
    println!(
        "  {:>5}  {:>9}  {:>8}  {:>8}  {:>5}  {:>5}",
        "Bloom", "Threshold", "H-Acc", "AI-Acc", "FP", "FN"
    );

    for level in 1..=7u8 {
        let level_samples: Vec<&Sample> =
            samples.iter().filter(|s| s.bloom_level == level).collect();
        if level_samples.is_empty() {
            continue;
        }

        let threshold = best_preset_thresholds[(level - 1) as usize];
        let mut h_correct = 0;
        let mut h_total = 0;
        let mut a_correct = 0;
        let mut a_total = 0;
        let mut fp = 0;
        let mut fn_ = 0;

        for s in &level_samples {
            let predicted_ai = s.probability > threshold;
            if s.is_ai {
                a_total += 1;
                if predicted_ai {
                    a_correct += 1;
                } else {
                    fn_ += 1;
                }
            } else {
                h_total += 1;
                if !predicted_ai {
                    h_correct += 1;
                } else {
                    fp += 1;
                }
            }
        }

        let h_acc = if h_total > 0 {
            h_correct as f64 / h_total as f64
        } else {
            0.0
        };
        let a_acc = if a_total > 0 {
            a_correct as f64 / a_total as f64
        } else {
            0.0
        };

        println!(
            "  L{:<4}  {:>9.2}  {:>7.1}%  {:>7.1}%  {:>5}  {:>5}",
            level,
            threshold,
            h_acc * 100.0,
            a_acc * 100.0,
            fp,
            fn_,
        );
    }

    // ── Phase 7: Recommended bloom.rs update ────────────────────────────
    println!("\n─── 6. Recommended bloom.rs Update ─────────────────────────\n");
    println!("  pub fn pv_education() -> Self {{");
    println!("      Self {{");
    println!("          name: \"pv_education\",");
    print!("          thresholds: [");
    for (i, t) in best_preset_thresholds.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{:.2}", t);
    }
    println!("],");
    println!("      }}");
    println!("  }}");

    println!("\n═══════════════════════════════════════════════════════════════");
}

/// Evaluate a flat (same threshold for all levels).
fn evaluate_flat(samples: &[Sample], threshold: f64) -> (f64, f64, usize, usize, f64, f64) {
    let mut tp = 0; // AI correctly flagged
    let mut tn = 0; // Human correctly passed
    let mut fp = 0; // Human incorrectly flagged
    let mut fn_ = 0; // AI incorrectly passed

    for s in samples {
        let predicted_ai = s.probability > threshold;
        match (s.is_ai, predicted_ai) {
            (true, true) => tp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
        }
    }

    let total = (tp + tn + fp + fn_) as f64;
    let acc = if total > 0.0 {
        (tp + tn) as f64 / total
    } else {
        0.0
    };
    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let fpr = if tn + fp > 0 {
        fp as f64 / (tn + fp) as f64
    } else {
        0.0
    };
    let fnr = if tp + fn_ > 0 {
        fn_ as f64 / (tp + fn_) as f64
    } else {
        0.0
    };

    (acc, f1, fp, fn_, fpr, fnr)
}

/// Evaluate per-level sensitivity and specificity.
fn evaluate_level(level_samples: &[&Sample], threshold: f64) -> (f64, f64, f64) {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for s in level_samples {
        let predicted_ai = s.probability > threshold;
        match (s.is_ai, predicted_ai) {
            (true, true) => tp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
        }
    }

    let sensitivity = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let specificity = if tn + fp > 0 {
        tn as f64 / (tn + fp) as f64
    } else {
        0.0
    };
    let total = (tp + tn + fp + fn_) as f64;
    let acc = if total > 0.0 {
        (tp + tn) as f64 / total
    } else {
        0.0
    };
    (sensitivity, specificity, acc)
}

/// Evaluate a per-Bloom preset.
fn evaluate_preset(
    samples: &[Sample],
    thresholds: &[f64; 7],
) -> (f64, f64, usize, usize, f64, f64) {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;

    for s in samples {
        let threshold = thresholds[(s.bloom_level - 1) as usize];
        let predicted_ai = s.probability > threshold;
        match (s.is_ai, predicted_ai) {
            (true, true) => tp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
        }
    }

    let total = (tp + tn + fp + fn_) as f64;
    let acc = if total > 0.0 {
        (tp + tn) as f64 / total
    } else {
        0.0
    };
    let precision = if tp + fp > 0 {
        tp as f64 / (tp + fp) as f64
    } else {
        0.0
    };
    let recall = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let f1 = if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    };
    let fpr = if tn + fp > 0 {
        fp as f64 / (tn + fp) as f64
    } else {
        0.0
    };
    let fnr = if tp + fn_ > 0 {
        fn_ as f64 / (tp + fn_) as f64
    } else {
        0.0
    };

    (acc, f1, fp, fn_, fpr, fnr)
}
