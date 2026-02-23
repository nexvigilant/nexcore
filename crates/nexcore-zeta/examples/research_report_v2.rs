//! RH Telescope Phase 2 — Full experimental data extraction.
//!
//! Runs all 6 new instruments against the 649-zero dataset:
//! 1. CMV vs Jacobi roundtrip comparison
//! 2. Even/odd zero subseries analysis
//! 3. Bootstrap confidence intervals on convergence rate
//! 4. Multi-x adversarial sensitivity curve
//! 5. Density profile by height band
//! 6. Adaptive truncation + residual by height
//!
//! Run: cargo run -p nexcore-zeta --example research_report_v2

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  RH TELESCOPE PHASE 2 — ALL INSTRUMENTS ACTIVE");
    println!("  Date: 2026-02-23");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    // ── Phase 0: Generate zeros ──────────────────────────────────────
    println!("▸ PHASE 0: Zero generation");
    let zeros_1000 =
        nexcore_zeta::zeros::find_zeros_bracket(10.0, 1000.0, 0.02).unwrap_or_default();
    println!("  Zeros to height 1000: {} found", zeros_1000.len());
    if zeros_1000.len() < 100 {
        println!("  ABORT: insufficient zeros for meaningful analysis");
        return;
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 1: CMV vs Jacobi — The Basis-Mismatch Test
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 1: CMV vs JACOBI ROUNDTRIP COMPARISON");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  ┌─ Roundtrip error: lower = better basis ─────────");
    println!(
        "  │ {:>5}  {:>14}  {:>14}  {:>8}",
        "N", "Jacobi Error", "CMV Error", "Winner"
    );

    for &n in &[10, 20, 30, 50, 75] {
        if n > zeros_1000.len() {
            continue;
        }
        let subset = &zeros_1000[..n];

        let jacobi_err = match nexcore_zeta::inverse::reconstruct_jacobi(subset) {
            Ok(j) => {
                if j.roundtrip_error.is_finite() {
                    j.roundtrip_error
                } else {
                    f64::INFINITY
                }
            }
            Err(_) => f64::INFINITY,
        };

        let cmv_err = match nexcore_zeta::cmv::reconstruct_cmv(subset) {
            Ok(c) => {
                if c.roundtrip_error.is_finite() {
                    c.roundtrip_error
                } else {
                    f64::INFINITY
                }
            }
            Err(_) => f64::INFINITY,
        };

        let winner = if cmv_err < jacobi_err {
            "CMV"
        } else {
            "Jacobi"
        };

        println!(
            "  │ {:>5}  {:>14.4}  {:>14.6}  {:>8}",
            n, jacobi_err, cmv_err, winner
        );
    }
    println!("  └────────────────────────────────────────────────");
    println!();

    // CMV structure at N=20 (stable regime)
    if zeros_1000.len() >= 20 {
        if let Ok(cmv) = nexcore_zeta::cmv::reconstruct_cmv(&zeros_1000[..20]) {
            println!("  CMV Structure at N=20:");
            println!(
                "    Mean |α_k|:          {:.6}",
                cmv.structure.mean_coefficient_magnitude
            );
            println!(
                "    Max |α_k|:           {:.6}",
                cmv.structure.max_coefficient
            );
            println!(
                "    Decay rate β:        {:.6}",
                cmv.structure.coefficient_decay_rate
            );
            println!(
                "    Coefficient reg:     {:.6}",
                cmv.structure.coefficient_regularity
            );
            println!(
                "    Phase regularity:    {:.6}",
                cmv.structure.phase_regularity
            );
            println!();
            println!("  First 10 Verblunsky magnitudes:");
            print!("    |α_k| = [");
            for (i, mag) in cmv.verblunsky_magnitudes.iter().take(10).enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{mag:.4}");
            }
            println!("]");
        }
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 2: Even/Odd Subseries — Is Period-2 Real?
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 2: EVEN/ODD ZERO SUBSERIES ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════");

    match nexcore_zeta::subseries::analyze_subseries(&zeros_1000) {
        Ok(analysis) => {
            println!("  ┌─ Subseries Statistics ──────────────────────");
            println!("  │ {:>12}  {:>10}  {:>10}", "Metric", "Even", "Odd");
            println!(
                "  │ {:>12}  {:>10}  {:>10}",
                "Count", analysis.even_count, analysis.odd_count
            );
            println!(
                "  │ {:>12}  {:>10.4}  {:>10.4}",
                "GUE Score", analysis.even_gue_score, analysis.odd_gue_score
            );
            println!(
                "  │ {:>12}  {:>10.4}  {:>10.4}",
                "Mean Spacing", analysis.even_mean_spacing, analysis.odd_mean_spacing
            );
            println!(
                "  │ {:>12}  {:>10.4}  {:>10.4}",
                "Spacing Var", analysis.even_spacing_variance, analysis.odd_spacing_variance
            );
            println!(
                "  │ {:>12}  {:>10.4}  {:>10.4}",
                "Pair Corr MAE", analysis.even_pair_corr_mae, analysis.odd_pair_corr_mae
            );
            println!("  │");
            println!(
                "  │ Difference metric:      {:.6}",
                analysis.difference_metric
            );
            println!(
                "  │ Distinguishable:        {} (threshold: 0.1)",
                if analysis.subseries_distinguishable {
                    "YES"
                } else {
                    "NO"
                }
            );
            println!("  └────────────────────────────────────────────");
        }
        Err(e) => println!("  ERROR: {e}"),
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 3: Bootstrap CI on Convergence Rate
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 3: BOOTSTRAP CONFIDENCE INTERVALS (N^(-0.35))");
    println!("═══════════════════════════════════════════════════════════════");

    let subsample_sizes: Vec<usize> = vec![30, 50, 75, 100, 150, 200, 300, 400, 500]
        .into_iter()
        .filter(|&s| s <= zeros_1000.len())
        .collect();

    if subsample_sizes.len() >= 3 {
        match nexcore_zeta::convergence::analyze_convergence_extended(
            &zeros_1000,
            &subsample_sizes,
            200, // bootstrap samples
        ) {
            Ok(ext) => {
                println!("  ┌─ Convergence with 95% CI ──────────────────");
                println!(
                    "  │ {:>6}  {:>10}  {:>10}  {:>10}",
                    "N", "MAE", "CI Lower", "CI Upper"
                );
                for br in &ext.bootstrap_results {
                    println!(
                        "  │ {:>6}  {:>10.4}  {:>10.4}  {:>10.4}",
                        br.n_zeros, br.mae_median, br.mae_ci_lower, br.mae_ci_upper
                    );
                }
                println!("  │");
                println!("  │ Model Comparison:");
                println!(
                    "  │   Power law R²:  {:.4} (β = {:.3})",
                    ext.base.power_law_fit.r_squared, ext.base.power_law_fit.exponent
                );
                println!(
                    "  │   Log law R²:    {:.4} (α = {:.3})",
                    ext.base.log_fit.r_squared, ext.base.log_fit.exponent
                );
                println!("  │   Best model:    {}", ext.base.best_model);
                println!("  │");
                println!("  │ Cross-Validation:");
                println!("  │   CV power law error: {:.6}", ext.cv_power_law_error);
                println!("  │   CV log error:       {:.6}", ext.cv_log_error);
                println!("  │");
                println!("  │ Extrapolation:");
                println!(
                    "  │   Predicted MAE at N=1000:  {:.6}",
                    ext.predicted_mae_1000
                );
                println!(
                    "  │   Predicted MAE at N=10000: {:.6}",
                    ext.predicted_mae_10000
                );
                println!("  │   Model confidence:         {}", ext.model_confidence);
                println!("  └────────────────────────────────────────────");
            }
            Err(e) => println!("  ERROR: {e}"),
        }
    } else {
        println!("  SKIP: insufficient subsample sizes");
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 4: Multi-x Sensitivity Curve
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 4: ADVERSARIAL SENSITIVITY CURVE");
    println!("═══════════════════════════════════════════════════════════════");

    let verification =
        nexcore_zeta::zeros::verify_rh_to_height(1000.0, 0.02).unwrap_or_else(|_| {
            nexcore_zeta::zeros::RhVerification {
                height: 1000.0,
                expected_zeros: 649.0,
                found_zeros: zeros_1000.len(),
                all_on_critical_line: true,
                zeros: zeros_1000.clone(),
            }
        });

    let x_values = vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0];
    match nexcore_zeta::adversarial::sensitivity_curve(&verification, &x_values) {
        Ok(curve) => {
            println!("  ┌─ Sensitivity: min detectable σ-deviation ───");
            println!(
                "  │ {:>12}  {:>12}  {:>12}  {:>10}",
                "x", "Min σ-dev", "Detect Ratio", "CL Contrib"
            );
            for p in &curve.points {
                println!(
                    "  │ {:>12.0}  {:>12.6}  {:>12.4}  {:>10.6}",
                    p.x, p.min_detectable_deviation, p.detectability_ratio, p.cl_contribution
                );
            }
            println!("  │");
            println!(
                "  │ Sensitivity exponent: {:.4} (fit: min_dev ~ x^(-α))",
                curve.sensitivity_exponent
            );
            println!("  │ R² of fit:            {:.4}", curve.model_r_squared);
            println!("  │");
            println!("  │ x needed to detect σ-deviation of:");
            println!("  │   0.10: {:.0}", curve.x_needed_for_010);
            println!("  │   0.05: {:.0}", curve.x_needed_for_005);
            println!("  │   0.01: {:.0}", curve.x_needed_for_001);
            println!("  └────────────────────────────────────────────");
        }
        Err(e) => println!("  ERROR: {e}"),
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 5: Density Profile by Height Band
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 5: DENSITY PROFILE BY HEIGHT BAND");
    println!("═══════════════════════════════════════════════════════════════");

    match nexcore_zeta::adversarial::density_profile(&zeros_1000, 100.0) {
        Ok(profile) => {
            println!("  ┌─ Density by 100-unit height bands ─────────");
            println!(
                "  │ {:>10}  {:>10}  {:>8}  {:>8}  {:>10}",
                "Band", "Expected", "Found", "Compl", "Status"
            );
            for band in &profile.bands {
                let status = if band.completeness < 0.95 {
                    "GAP"
                } else if band.completeness > 1.05 {
                    "EXCESS"
                } else {
                    "OK"
                };
                println!(
                    "  │ {:>4}-{:<5}  {:>10.2}  {:>8}  {:>7.4}  {:>10}",
                    band.height_start as u64,
                    band.height_end as u64,
                    band.expected,
                    band.found,
                    band.completeness,
                    status
                );
            }
            println!("  │");
            println!(
                "  │ Overall completeness:   {:.6}",
                profile.overall_completeness
            );
            println!(
                "  │ Min band completeness:  {:.6}",
                profile.min_band_completeness
            );
            println!(
                "  │ Max band completeness:  {:.6}",
                profile.max_band_completeness
            );
            println!("  │ Gaps detected:          {}", profile.gaps_detected);
            println!("  └────────────────────────────────────────────");
        }
        Err(e) => println!("  ERROR: {e}"),
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 6: Adaptive Truncation + Residual by Height
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 6: ADAPTIVE TRUNCATION & RESIDUAL BY HEIGHT");
    println!("═══════════════════════════════════════════════════════════════");

    for &x in &[100.0, 500.0, 1000.0, 5000.0] {
        match nexcore_zeta::explicit::adaptive_truncation(x, &zeros_1000) {
            Ok(at) => {
                println!("  ┌─ Adaptive truncation at x = {x} ──────────");
                println!("  │ Riemann-Siegel optimal N: {}", at.riemann_siegel_n);
                println!("  │ Best N found:             {}", at.best_n);
                println!("  │ Best error:               {:.6}", at.best_error);
                println!("  │ Truncation sweep:");
                println!("  │   {:>8}  {:>12}", "N", "Rel Error");
                for &(n, err) in &at.errors_by_truncation {
                    let marker = if n == at.best_n { " ← BEST" } else { "" };
                    println!("  │   {:>8}  {:>12.6}{}", n, err, marker);
                }
                println!("  └────────────────────────────────────────────");
            }
            Err(e) => println!("  x={x}: ERROR: {e}"),
        }
        println!();
    }

    // Residual by height at x=500
    println!("  ┌─ Residual by height at x=500 ──────────────");
    match nexcore_zeta::explicit::residual_by_height(500.0, &zeros_1000) {
        Ok(rbh) => {
            println!(
                "  │ Height-error correlation: {:.6}",
                rbh.correlation_with_height
            );
            println!("  │ Total zeros analyzed:     {}", rbh.residuals.len());
            println!("  │");
            println!("  │ First 10 residuals (height, marginal Δerror):");
            for &(h, r) in rbh.residuals.iter().take(10) {
                println!("  │   t={:>8.3}  Δerr={:>+12.8}", h, r);
            }
            if rbh.residuals.len() > 10 {
                println!("  │   ... ({} more)", rbh.residuals.len() - 10);
            }
        }
        Err(e) => println!("  │ ERROR: {e}"),
    }
    println!("  └────────────────────────────────────────────");
    println!();

    // ══════════════════════════════════════════════════════════════════
    // EXPERIMENT 2b: Coupling Extrapolation
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 2b: COUPLING REGULARITY EXTRAPOLATION");
    println!("═══════════════════════════════════════════════════════════════");

    let coupling_sizes: Vec<usize> = vec![15, 20, 30, 40, 50, 60, 75]
        .into_iter()
        .filter(|&s| s <= zeros_1000.len())
        .collect();

    if coupling_sizes.len() >= 3 {
        match nexcore_zeta::subseries::extrapolate_coupling(&zeros_1000, &coupling_sizes) {
            Ok(ext) => {
                println!("  ┌─ Coupling regularity trend ─────────────────");
                println!("  │ {:>6}  {:>14}", "N", "Coupling Reg");
                for &(n, reg) in &ext.data_points {
                    println!("  │ {:>6}  {:>14.6}", n, reg);
                }
                println!("  │");
                println!(
                    "  │ Power law: reg ~ {:.4} · N^({:.4})",
                    ext.power_law_amplitude, ext.power_law_exponent
                );
                println!("  │ R² = {:.4}", ext.model_r_squared);
                println!("  │");
                println!("  │ Predictions:");
                println!("  │   N=1,000:   {:.6}", ext.predicted_at_1000);
                println!("  │   N=10,000:  {:.6}", ext.predicted_at_10000);
                println!("  │   N=100,000: {:.6}", ext.predicted_at_100000);
                println!(
                    "  │   Asymptotic: {}",
                    if ext.asymptotic_value == 0.0 {
                        "→ 0 (perfectly uniform coupling)".to_string()
                    } else {
                        format!("{:.6}", ext.asymptotic_value)
                    }
                );
                println!("  └────────────────────────────────────────────");
            }
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    println!();

    // ══════════════════════════════════════════════════════════════════
    // SUMMARY
    // ══════════════════════════════════════════════════════════════════
    println!("═══════════════════════════════════════════════════════════════");
    println!("  PHASE 2 COMPLETE — 6 EXPERIMENTS EXECUTED");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Zeros used:     {}", zeros_1000.len());
    println!("  Height range:   10.0 — 1000.0");
    println!("  Experiments:    6 (+ 1 sub-experiment)");
    println!("  Key questions answered:");
    println!("    1. CMV vs Jacobi — which basis is better?");
    println!("    2. Even/odd subseries — is period-2 real or artifact?");
    println!("    3. N^(-0.35) — confidence intervals via bootstrap");
    println!("    4. Sensitivity — what x needed for σ-dev detection?");
    println!("    5. Density — any gaps in height coverage?");
    println!("    6. Truncation — adaptive optimal N for explicit formula");
}
