//! # Deep Recalibration — Chemistry Parameter Sweep
//!
//! The threshold sweep revealed the real problem: the Arrhenius gate compresses
//! human and AI probabilities into a narrow 0.57-0.67 band. The fix must tune
//! the chemistry parameters (Ea, scale, Hill n, Hill K_half) to widen separation.
//!
//! ## Simulations
//! 1. **Raw Feature Comparison**: Per-feature human vs AI deltas
//! 2. **Hill Parameter Sweep**: n_hill × k_half grid → composite separation
//! 3. **Arrhenius Parameter Sweep**: Ea × scale grid → probability separation
//! 4. **Combined Best**: Best Hill + Arrhenius together
//! 5. **Threshold Refit**: Youden's J with new chemistry
//!
//! ```bash
//! cargo run -p nexcore-integrity --example recalibrate_deep --features fixtures
//! ```

use nexcore_integrity::aggregation::{self, RawFeatures, WEIGHTS};
use nexcore_integrity::assessment::{DEFAULT_WINDOW_SIZE, DEFAULT_WINDOW_STEP, MIN_TOKENS};
use nexcore_integrity::burstiness;
use nexcore_integrity::chemistry;
use nexcore_integrity::entropy;
use nexcore_integrity::fixtures::all_fixtures;
use nexcore_integrity::perplexity;
use nexcore_integrity::tokenize;
use nexcore_integrity::zipf;

struct RawSample {
    ksb_id: &'static str,
    bloom_level: u8,
    is_ai: bool,
    features: RawFeatures,
    composite: f64,
    hill_score: f64,
}

fn extract_features(text: &str) -> Option<(RawFeatures, f64, f64)> {
    let stats = tokenize::tokenize(text);
    if stats.total_tokens < MIN_TOKENS {
        return None;
    }
    let zipf_result = zipf::zipf_analysis(&stats.frequencies);
    let entropy_profile =
        entropy::entropy_profile(&stats.tokens, DEFAULT_WINDOW_SIZE, DEFAULT_WINDOW_STEP);
    let burst_result = burstiness::burstiness_analysis(&stats.tokens, &stats.frequencies);
    let perp_result = perplexity::perplexity_variance(text);
    let ttr_dev = tokenize::ttr_deviation(stats.ttr);

    let raw = RawFeatures {
        zipf_deviation: zipf_result.deviation,
        entropy_std: entropy_profile.std_dev,
        burstiness: burst_result.coefficient,
        perplexity_var: perp_result.variance,
        ttr_deviation: ttr_dev,
    };
    let agg = aggregation::aggregate(&raw);
    Some((raw, agg.composite, agg.hill_score))
}

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Deep Recalibration — Chemistry Parameter Sweep");
    println!("═══════════════════════════════════════════════════════════════\n");

    // ── Collect raw features ────────────────────────────────────────────
    let fixtures = all_fixtures();
    let mut samples = Vec::new();

    for fixture in &fixtures {
        if let Some((features, composite, hill_score)) = extract_features(fixture.human_response) {
            samples.push(RawSample {
                ksb_id: fixture.ksb_id,
                bloom_level: fixture.bloom_level,
                is_ai: false,
                features,
                composite,
                hill_score,
            });
        }
        if let Some((features, composite, hill_score)) = extract_features(fixture.ai_response) {
            samples.push(RawSample {
                ksb_id: fixture.ksb_id,
                bloom_level: fixture.bloom_level,
                is_ai: true,
                features,
                composite,
                hill_score,
            });
        }
    }

    // ── Phase 1: Per-Feature Deltas ─────────────────────────────────────
    println!("─── 1. Per-Feature Human vs AI Means ─────────────────────\n");

    let human: Vec<&RawSample> = samples.iter().filter(|s| !s.is_ai).collect();
    let ai: Vec<&RawSample> = samples.iter().filter(|s| s.is_ai).collect();
    let hn = human.len() as f64;
    let an = ai.len() as f64;

    let h_zipf = human.iter().map(|s| s.features.zipf_deviation).sum::<f64>() / hn;
    let a_zipf = ai.iter().map(|s| s.features.zipf_deviation).sum::<f64>() / an;
    let h_ent = human.iter().map(|s| s.features.entropy_std).sum::<f64>() / hn;
    let a_ent = ai.iter().map(|s| s.features.entropy_std).sum::<f64>() / an;
    let h_burst = human.iter().map(|s| s.features.burstiness).sum::<f64>() / hn;
    let a_burst = ai.iter().map(|s| s.features.burstiness).sum::<f64>() / an;
    let h_perp = human.iter().map(|s| s.features.perplexity_var).sum::<f64>() / hn;
    let a_perp = ai.iter().map(|s| s.features.perplexity_var).sum::<f64>() / an;
    let h_ttr = human.iter().map(|s| s.features.ttr_deviation).sum::<f64>() / hn;
    let a_ttr = ai.iter().map(|s| s.features.ttr_deviation).sum::<f64>() / an;
    let h_comp = human.iter().map(|s| s.composite).sum::<f64>() / hn;
    let a_comp = ai.iter().map(|s| s.composite).sum::<f64>() / an;
    let h_hill = human.iter().map(|s| s.hill_score).sum::<f64>() / hn;
    let a_hill = ai.iter().map(|s| s.hill_score).sum::<f64>() / an;

    println!(
        "  {:>18}  {:>10}  {:>10}  {:>10}  {:>6}",
        "Feature", "Human", "AI", "Delta", "Dir"
    );
    println!(
        "  {:─>18}  {:─>10}  {:─>10}  {:─>10}  {:─>6}",
        "", "", "", "", ""
    );
    print_feature_row("zipf_deviation", h_zipf, a_zipf, "higher=AI");
    print_feature_row("entropy_std", h_ent, a_ent, "lower=AI");
    print_feature_row("burstiness", h_burst, a_burst, "lower=AI");
    print_feature_row("perplexity_var", h_perp, a_perp, "lower=AI");
    print_feature_row("ttr_deviation", h_ttr, a_ttr, "higher=AI");
    print_feature_row("composite", h_comp, a_comp, "higher=AI");
    print_feature_row("hill_score", h_hill, a_hill, "higher=AI");

    // Effect sizes (Cohen's d approximation)
    println!("\n  Effect sizes (|mean_delta| / pooled_std):");
    println!(
        "    composite: {:.3}",
        cohens_d(
            &human.iter().map(|s| s.composite).collect::<Vec<_>>(),
            &ai.iter().map(|s| s.composite).collect::<Vec<_>>(),
        )
    );
    println!(
        "    hill_score: {:.3}",
        cohens_d(
            &human.iter().map(|s| s.hill_score).collect::<Vec<_>>(),
            &ai.iter().map(|s| s.hill_score).collect::<Vec<_>>(),
        )
    );

    // ── Phase 2: Hill Parameter Sweep ───────────────────────────────────
    println!("\n─── 2. Hill Parameter Sweep (n_hill × k_half) ──────────\n");
    println!(
        "  {:>6}  {:>6}  {:>10}  {:>10}  {:>10}  {:>10}",
        "n_hill", "k_half", "H_mean", "AI_mean", "Delta", "Cohen_d"
    );
    println!(
        "  {:─>6}  {:─>6}  {:─>10}  {:─>10}  {:─>10}  {:─>10}",
        "", "", "", "", "", ""
    );

    let mut best_hill_d = 0.0f64;
    let mut best_hill_n = 2.5;
    let mut best_hill_k = 0.5;

    for n_x10 in (10..=50).step_by(5) {
        let n_hill = n_x10 as f64 / 10.0;
        for k_x10 in (30..=70).step_by(5) {
            let k_half = k_x10 as f64 / 100.0;

            let h_hills: Vec<f64> = human
                .iter()
                .map(|s| chemistry::hill_amplify(s.composite, k_half, n_hill))
                .collect();
            let a_hills: Vec<f64> = ai
                .iter()
                .map(|s| chemistry::hill_amplify(s.composite, k_half, n_hill))
                .collect();

            let d = cohens_d(&h_hills, &a_hills);
            let h_m = h_hills.iter().sum::<f64>() / h_hills.len() as f64;
            let a_m = a_hills.iter().sum::<f64>() / a_hills.len() as f64;

            if d > best_hill_d {
                best_hill_d = d;
                best_hill_n = n_hill;
                best_hill_k = k_half;
                println!(
                    "  {:>6.1}  {:>6.2}  {:>10.4}  {:>10.4}  {:>10.4}  {:>10.3} ◄",
                    n_hill,
                    k_half,
                    h_m,
                    a_m,
                    a_m - h_m,
                    d,
                );
            }
        }
    }
    println!(
        "\n  Best Hill: n={:.1}, k={:.2} (Cohen's d = {:.3})",
        best_hill_n, best_hill_k, best_hill_d
    );

    // ── Phase 3: Arrhenius Parameter Sweep ──────────────────────────────
    println!("\n─── 3. Arrhenius Parameter Sweep (Ea × scale) ──────────\n");
    println!(
        "  {:>4}  {:>5}  {:>10}  {:>10}  {:>10}  {:>10}",
        "Ea", "Scale", "H_prob", "AI_prob", "Delta", "Cohen_d"
    );
    println!(
        "  {:─>4}  {:─>5}  {:─>10}  {:─>10}  {:─>10}  {:─>10}",
        "", "", "", "", "", ""
    );

    let mut best_arr_d = 0.0f64;
    let mut best_ea = 3.0;
    let mut best_scale = 10.0;

    // Use the best Hill parameters for this sweep
    for ea_x10 in (10..=80).step_by(5) {
        let ea = ea_x10 as f64 / 10.0;
        for scale in (5..=25).step_by(2) {
            let scale_f = scale as f64;

            let h_probs: Vec<f64> = human
                .iter()
                .map(|s| {
                    let hill = chemistry::hill_amplify(s.composite, best_hill_k, best_hill_n);
                    chemistry::arrhenius_probability(ea, hill, scale_f)
                })
                .collect();
            let a_probs: Vec<f64> = ai
                .iter()
                .map(|s| {
                    let hill = chemistry::hill_amplify(s.composite, best_hill_k, best_hill_n);
                    chemistry::arrhenius_probability(ea, hill, scale_f)
                })
                .collect();

            let d = cohens_d(&h_probs, &a_probs);
            let h_m = h_probs.iter().sum::<f64>() / h_probs.len() as f64;
            let a_m = a_probs.iter().sum::<f64>() / a_probs.len() as f64;

            if d > best_arr_d {
                best_arr_d = d;
                best_ea = ea;
                best_scale = scale_f;
                println!(
                    "  {:>4.1}  {:>5.0}  {:>10.4}  {:>10.4}  {:>10.4}  {:>10.3} ◄",
                    ea,
                    scale_f,
                    h_m,
                    a_m,
                    a_m - h_m,
                    d,
                );
            }
        }
    }
    println!(
        "\n  Best Arrhenius: Ea={:.1}, scale={:.0} (Cohen's d = {:.3})",
        best_ea, best_scale, best_arr_d
    );

    // ── Phase 4: Combined Best ──────────────────────────────────────────
    println!("\n─── 4. Combined Best Chemistry ─────────────────────────\n");
    println!(
        "  Hill:      n_hill={:.1}, k_half={:.2}",
        best_hill_n, best_hill_k
    );
    println!("  Arrhenius: Ea={:.1}, scale={:.0}", best_ea, best_scale);

    let mut combined_samples: Vec<(bool, u8, f64)> = Vec::new();

    println!(
        "\n  {:>6}  {:>5}  {:>5}  {:>10}  {:>10}  {:>10}",
        "KSB", "Bloom", "Type", "Composite", "New_Hill", "New_Prob"
    );
    println!(
        "  {:─>6}  {:─>5}  {:─>5}  {:─>10}  {:─>10}  {:─>10}",
        "", "", "", "", "", ""
    );

    for s in &samples {
        let new_hill = chemistry::hill_amplify(s.composite, best_hill_k, best_hill_n);
        let new_prob = chemistry::arrhenius_probability(best_ea, new_hill, best_scale);
        combined_samples.push((s.is_ai, s.bloom_level, new_prob));

        println!(
            "  {:>6}  L{:<4}  {:>5}  {:>10.4}  {:>10.4}  {:>10.4}",
            s.ksb_id.split('-').last().unwrap_or(s.ksb_id),
            s.bloom_level,
            if s.is_ai { "AI" } else { "Hum" },
            s.composite,
            new_hill,
            new_prob,
        );
    }

    let h_new: Vec<f64> = combined_samples
        .iter()
        .filter(|s| !s.0)
        .map(|s| s.2)
        .collect();
    let a_new: Vec<f64> = combined_samples
        .iter()
        .filter(|s| s.0)
        .map(|s| s.2)
        .collect();
    println!(
        "\n  New Human prob range: [{:.4}, {:.4}] mean={:.4}",
        h_new.iter().cloned().fold(f64::INFINITY, f64::min),
        h_new.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        h_new.iter().sum::<f64>() / h_new.len() as f64,
    );
    println!(
        "  New AI    prob range: [{:.4}, {:.4}] mean={:.4}",
        a_new.iter().cloned().fold(f64::INFINITY, f64::min),
        a_new.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        a_new.iter().sum::<f64>() / a_new.len() as f64,
    );
    println!("  Combined Cohen's d:  {:.3}", cohens_d(&h_new, &a_new));

    // ── Phase 5: Threshold Refit ────────────────────────────────────────
    println!("\n─── 5. Threshold Refit with New Chemistry ──────────────\n");

    // Flat sweep with new chemistry
    let mut best_flat_f1 = 0.0;
    let mut best_flat_t = 0.50;
    let mut best_flat_acc = 0.0;

    println!(
        "  {:>9}  {:>8}  {:>8}  {:>5}  {:>5}  {:>8}  {:>8}",
        "Threshold", "Accuracy", "F1", "FP", "FN", "FPR", "FNR"
    );
    println!(
        "  {:─>9}  {:─>8}  {:─>8}  {:─>5}  {:─>5}  {:─>8}  {:─>8}",
        "", "", "", "", "", "", ""
    );

    for t_pct in 0..=100 {
        let threshold = t_pct as f64 / 100.0;
        let (acc, f1, fp, fn_, fpr, fnr) = evaluate_combined(&combined_samples, threshold);
        if f1 > best_flat_f1 {
            best_flat_f1 = f1;
            best_flat_t = threshold;
            best_flat_acc = acc;
            println!(
                "  {:>9.2}  {:>7.1}%  {:>8.3}  {:>5}  {:>5}  {:>7.1}%  {:>7.1}% ◄",
                threshold,
                acc * 100.0,
                f1,
                fp,
                fn_,
                fpr * 100.0,
                fnr * 100.0,
            );
        }
    }
    println!(
        "\n  Best flat: threshold={:.2}, F1={:.3}, acc={:.1}%",
        best_flat_t,
        best_flat_f1,
        best_flat_acc * 100.0
    );

    // Per-Bloom Youden
    println!("\n  Per-Bloom Youden's J with new chemistry:");
    let mut new_thresholds = [0.0f64; 7];
    for level in 1..=7u8 {
        let level_samples: Vec<(bool, f64)> = combined_samples
            .iter()
            .filter(|s| s.1 == level)
            .map(|s| (s.0, s.2))
            .collect();
        if level_samples.is_empty() {
            continue;
        }

        let mut best_j = -1.0f64;
        let mut best_t = 0.50;
        for t_pct in 0..=100 {
            let threshold = t_pct as f64 / 100.0;
            let (sens, spec) = sens_spec_combined(&level_samples, threshold);
            let j = sens + spec - 1.0;
            if j > best_j {
                best_j = j;
                best_t = threshold;
            }
        }
        new_thresholds[(level - 1) as usize] = best_t;
        println!("    L{}: threshold={:.2} (J={:.3})", level, best_t, best_j);
    }

    // Monotonize
    for i in 1..7 {
        if new_thresholds[i] > new_thresholds[i - 1] {
            new_thresholds[i] = new_thresholds[i - 1];
        }
    }

    // Evaluate with Youden thresholds
    let (acc, f1, fp, fn_, fpr, fnr) = evaluate_combined_preset(&combined_samples, &new_thresholds);
    println!(
        "\n  Youden-optimal (monotonized): {:?}",
        new_thresholds.map(|t| (t * 100.0).round() / 100.0)
    );
    println!(
        "  Accuracy={:.1}%, F1={:.3}, FP={}, FN={}, FPR={:.1}%, FNR={:.1}%",
        acc * 100.0,
        f1,
        fp,
        fn_,
        fpr * 100.0,
        fnr * 100.0
    );

    // ── Phase 6: Summary ────────────────────────────────────────────────
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  RECOMMENDED CHANGES");
    println!("═══════════════════════════════════════════════════════════════\n");
    println!("  aggregation.rs:");
    println!(
        "    pub const HILL_K_HALF: f64 = {:.2};  // was 0.50",
        best_hill_k
    );
    println!(
        "    pub const HILL_N: f64 = {:.1};       // was 2.5",
        best_hill_n
    );
    println!();
    println!("  classify.rs:");
    println!(
        "    pub const ACTIVATION_ENERGY: f64 = {:.1};  // was 3.0",
        best_ea
    );
    println!(
        "    pub const SCALE_FACTOR: f64 = {:.1};       // was 10.0",
        best_scale
    );
    println!();
    println!("  bloom.rs (pv_education):");
    print!("    thresholds: [");
    for (i, t) in new_thresholds.iter().enumerate() {
        if i > 0 {
            print!(", ");
        }
        print!("{:.2}", t);
    }
    println!("]");
    println!();
    println!("═══════════════════════════════════════════════════════════════");
}

fn print_feature_row(name: &str, h: f64, a: f64, dir: &str) {
    let delta = a - h;
    println!(
        "  {:>18}  {:>10.4}  {:>10.4}  {:>+10.4}  {:>6}",
        name, h, a, delta, dir,
    );
}

fn cohens_d(a: &[f64], b: &[f64]) -> f64 {
    let a_mean = a.iter().sum::<f64>() / a.len() as f64;
    let b_mean = b.iter().sum::<f64>() / b.len() as f64;
    let a_var = a.iter().map(|x| (x - a_mean).powi(2)).sum::<f64>() / (a.len() as f64 - 1.0);
    let b_var = b.iter().map(|x| (x - b_mean).powi(2)).sum::<f64>() / (b.len() as f64 - 1.0);
    let pooled_std = ((a_var + b_var) / 2.0).sqrt();
    if pooled_std < 1e-15 {
        return 0.0;
    }
    (b_mean - a_mean).abs() / pooled_std
}

fn evaluate_combined(
    samples: &[(bool, u8, f64)],
    threshold: f64,
) -> (f64, f64, usize, usize, f64, f64) {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;
    for &(is_ai, _, prob) in samples {
        let predicted_ai = prob > threshold;
        match (is_ai, predicted_ai) {
            (true, true) => tp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
        }
    }
    metrics(tp, tn, fp, fn_)
}

fn evaluate_combined_preset(
    samples: &[(bool, u8, f64)],
    thresholds: &[f64; 7],
) -> (f64, f64, usize, usize, f64, f64) {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;
    for &(is_ai, level, prob) in samples {
        let threshold = thresholds[(level - 1) as usize];
        let predicted_ai = prob > threshold;
        match (is_ai, predicted_ai) {
            (true, true) => tp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
        }
    }
    metrics(tp, tn, fp, fn_)
}

fn sens_spec_combined(level_samples: &[(bool, f64)], threshold: f64) -> (f64, f64) {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_ = 0;
    for &(is_ai, prob) in level_samples {
        let predicted_ai = prob > threshold;
        match (is_ai, predicted_ai) {
            (true, true) => tp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
            (false, true) => fp += 1,
        }
    }
    let sens = if tp + fn_ > 0 {
        tp as f64 / (tp + fn_) as f64
    } else {
        0.0
    };
    let spec = if tn + fp > 0 {
        tn as f64 / (tn + fp) as f64
    } else {
        0.0
    };
    (sens, spec)
}

fn metrics(tp: usize, tn: usize, fp: usize, fn_: usize) -> (f64, f64, usize, usize, f64, f64) {
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
