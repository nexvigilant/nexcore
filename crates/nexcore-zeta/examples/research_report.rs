//! Research data extraction for the RH telescope.
//!
//! Runs all three research modules (inverse, adversarial, convergence)
//! and prints structured findings for documentation.
//!
//! Run: cargo run -p nexcore-zeta --example research_report

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  RH TELESCOPE — EXPERIMENTAL DATA EXTRACTION");
    println!("  Date: 2026-02-23");
    println!("═══════════════════════════════════════════════════════════════");
    println!();

    // ── Phase 0: Generate zeros ──────────────────────────────────────
    println!("▸ PHASE 0: Zero generation");
    let zeros_200 = nexcore_zeta::zeros::find_zeros_bracket(10.0, 200.0, 0.05).unwrap_or_default();
    let zeros_1000 =
        nexcore_zeta::zeros::find_zeros_bracket(10.0, 1000.0, 0.02).unwrap_or_default();

    println!("  Zeros to height 200:  {} found", zeros_200.len());
    println!("  Zeros to height 1000: {} found", zeros_1000.len());
    if let (Some(first), Some(last)) = (zeros_1000.first(), zeros_1000.last()) {
        println!(
            "  First zero: t = {:.6} (ordinal {})",
            first.t, first.ordinal
        );
        println!("  Last zero:  t = {:.6} (ordinal {})", last.t, last.ordinal);
    }
    println!();

    // ── Experiment 1: Inverse Spectral Reconstruction ────────────────
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 1: INVERSE SPECTRAL RECONSTRUCTION");
    println!("═══════════════════════════════════════════════════════════════");

    // Run at multiple scales to see how conditioning changes
    for (label, zeros) in [
        ("n=20 (height ~77)", &zeros_200[..20.min(zeros_200.len())]),
        ("n=50 (height ~150)", &zeros_200[..50.min(zeros_200.len())]),
        ("n=75 (height ~200)", &zeros_200[..]),
        (
            "n=200 (height ~500)",
            &zeros_1000[..200.min(zeros_1000.len())],
        ),
        ("n=ALL (height ~1000)", &zeros_1000[..]),
    ] {
        if zeros.len() < 3 {
            continue;
        }
        match nexcore_zeta::inverse::reconstruct_jacobi(zeros) {
            Ok(j) => {
                let s = &j.structure;
                println!();
                println!("  ┌─ {} ─────────────────────────────", label);
                println!("  │ Eigenvalues used:         {}", j.eigenvalues.len());
                println!("  │ Roundtrip error:           {:.6}", j.roundtrip_error);
                println!("  │ Mean diagonal:             {:.6}", s.mean_diagonal);
                println!("  │ Diagonal variance:         {:.6}", s.diagonal_variance);
                println!("  │ Mean off-diagonal:         {:.6}", s.mean_off_diagonal);
                println!(
                    "  │ Off-diagonal variance:     {:.6}",
                    s.off_diagonal_variance
                );
                println!(
                    "  │ Coupling regularity:       {:.6}",
                    s.coupling_regularity
                );
                println!(
                    "  │ Diagonal growth exponent:  {:.6}",
                    s.diagonal_growth_exponent
                );
                println!(
                    "  │ Off-diag growth exponent:  {:.6}",
                    s.off_diagonal_growth_exponent
                );
                println!(
                    "  │ Spacing variance:          {:.6} (GUE≈0.178, Poisson≈1.0)",
                    s.spacing_variance
                );

                // Print first 10 diagonal and off-diagonal elements
                let n_show = 10.min(j.diagonal.len());
                print!("  │ First {} diag:     [", n_show);
                for (i, d) in j.diagonal.iter().take(n_show).enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{:.2}", d);
                }
                println!("]");
                let n_show_od = 10.min(j.off_diagonal.len());
                print!("  │ First {} off-diag: [", n_show_od);
                for (i, b) in j.off_diagonal.iter().take(n_show_od).enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    print!("{:.2}", b);
                }
                println!("]");
                println!("  └────────────────────────────────────────");
            }
            Err(e) => {
                println!("  {} — FAILED: {e}", label);
            }
        }
    }

    // ── Experiment 2: Adversarial Counterexample Characterization ────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 2: ADVERSARIAL COUNTEREXAMPLE ANALYSIS");
    println!("═══════════════════════════════════════════════════════════════");

    let verification = nexcore_zeta::zeros::verify_rh_to_height(1000.0, 0.02)
        .unwrap_or_else(|e| panic!("verification failed: {e}"));

    println!("  Verification height:   {:.1}", verification.height);
    println!("  Zeros found:           {}", verification.found_zeros);
    println!(
        "  All on critical line:  {}",
        verification.all_on_critical_line
    );
    println!(
        "  Expected (R-vM):       {:.1}",
        verification.expected_zeros
    );

    match nexcore_zeta::adversarial::analyze_exclusions(&verification, 1000.0) {
        Ok(analysis) => {
            println!();
            println!("  ┌─ Exclusion Analysis ─────────────────────");
            println!(
                "  │ Scanned area:              {:.1}",
                analysis.scanned_area
            );
            println!(
                "  │ Min counterexample height:  {:.1}",
                analysis.min_counterexample_height
            );
            println!(
                "  │ Zero-free regions:          {}",
                analysis.zero_free_regions.len()
            );
            for r in &analysis.zero_free_regions {
                println!(
                    "  │   • {} [σ∈({:.4},{:.4}), t∈({:.1},{:.1})]",
                    r.description, r.sigma_min, r.sigma_max, r.t_min, r.t_max
                );
                println!("  │     Source: {}", r.source);
            }
            let dc = &analysis.density_constraints;
            println!(
                "  │ Density completeness:      {:.4} ({}/{})",
                dc.completeness_ratio, dc.actual_count, dc.expected_count_at_verified_height as u64
            );
            println!("  │ Max surplus zeros:          {:.1}", dc.max_surplus);
            let pa = &analysis.perturbation_analysis;
            println!("  │ Perturbation at x=1000:");
            println!(
                "  │   CL zero contrib:          {:.6}",
                pa.cl_zero_contribution
            );
            println!(
                "  │   Off-CL zero contrib:      {:.6}",
                pa.off_cl_contribution
            );
            println!(
                "  │   Detectability ratio:       {:.4}",
                pa.detectability_ratio
            );
            println!(
                "  │   Min detectable σ-dev:      {:.6}",
                pa.min_detectable_deviation
            );
            println!("  └────────────────────────────────────────");
        }
        Err(e) => println!("  Exclusion analysis FAILED: {e}"),
    }

    // Landscape scan
    let landscape =
        nexcore_zeta::adversarial::map_counterexample_landscape(10, (1.0, 2000.0), 20, 1000.0);
    let total = landscape.len();
    let excluded = landscape.iter().filter(|c| c.excluded).count();
    let open = total - excluded;
    println!();
    println!("  ┌─ Landscape Scan ─────────────────────────");
    println!("  │ Grid: 10 σ-values × 20 t-values = {} candidates", total);
    println!(
        "  │ Excluded (t < 1000):  {} ({:.1}%)",
        excluded,
        100.0 * excluded as f64 / total as f64
    );
    println!(
        "  │ Open (t ≥ 1000):      {} ({:.1}%)",
        open,
        100.0 * open as f64 / total as f64
    );

    // Sample some open candidates
    println!("  │ Sample open candidates:");
    for c in landscape.iter().filter(|c| !c.excluded).take(5) {
        println!(
            "  │   σ={:.3}, t={:.1}, partner_σ={:.3}, dist_from_CL={:.3}",
            c.sigma, c.t, c.partner_sigma, c.distance_from_cl
        );
    }
    println!("  └────────────────────────────────────────");

    // ── Experiment 3: GUE Convergence Rate ───────────────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 3: GUE CONVERGENCE RATE");
    println!("═══════════════════════════════════════════════════════════════");

    let max_n = zeros_1000.len();
    let mut sizes: Vec<usize> = vec![30, 50, 75, 100, 150, 200, 300, 400, 500];
    sizes.retain(|&s| s <= max_n);
    if max_n > 500 {
        sizes.push(max_n);
    }

    if sizes.len() >= 3 {
        match nexcore_zeta::convergence::analyze_convergence(&zeros_1000, &sizes) {
            Ok(analysis) => {
                println!("  ┌─ Convergence Points ─────────────────────");
                println!(
                    "  │ {:>6}  {:>12}  {:>10}  {:>10}  {:>10}",
                    "N", "MAE", "MeanSpac", "Variance", "GUE Score"
                );
                println!(
                    "  │ {:>6}  {:>12}  {:>10}  {:>10}  {:>10}",
                    "------", "------------", "----------", "----------", "----------"
                );
                for p in &analysis.points {
                    println!(
                        "  │ {:>6}  {:>12.6}  {:>10.4}  {:>10.4}  {:>10.4}",
                        p.n_zeros,
                        p.pair_correlation_mae,
                        p.mean_spacing,
                        p.spacing_variance,
                        p.gue_match_score
                    );
                }
                println!("  │");
                println!(
                    "  │ Power law fit: MAE ~ {:.4} × N^(-{:.4})",
                    analysis.power_law_fit.amplitude, analysis.power_law_fit.exponent
                );
                println!("  │   R² = {:.6}", analysis.power_law_fit.r_squared);
                println!("  │");
                println!(
                    "  │ Log fit: MAE ~ {:.4} / ln(N)^{:.4}",
                    analysis.log_fit.amplitude, analysis.log_fit.exponent
                );
                println!("  │   R² = {:.6}", analysis.log_fit.r_squared);
                println!("  │");
                println!("  │ Best model: {}", analysis.best_model);
                println!("  │ Faster than O(1/ln N): {}", analysis.faster_than_log);
                println!("  └────────────────────────────────────────");
            }
            Err(e) => println!("  Convergence analysis FAILED: {e}"),
        }
    } else {
        println!(
            "  Insufficient zeros for convergence analysis (need sizes >= 3, have {})",
            sizes.len()
        );
    }

    // ── Experiment 4: Explicit Formula Error vs Truncation ───────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 4: EXPLICIT FORMULA — ERROR vs TRUNCATION DEPTH");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  ┌─ ψ(x) Reconstruction ─────────────────────");
    println!(
        "  │ {:>6}  {:>8}  {:>12}  {:>12}  {:>10}",
        "x", "N_zeros", "ψ_explicit", "ψ_direct", "Rel Error"
    );
    println!(
        "  │ {:>6}  {:>8}  {:>12}  {:>12}  {:>10}",
        "------", "--------", "------------", "------------", "----------"
    );

    for &x in &[100.0, 200.0, 500.0, 1000.0, 2000.0, 5000.0] {
        // Use all available zeros
        match nexcore_zeta::explicit::explicit_psi_comparison(x, &zeros_1000) {
            Ok((psi_explicit, psi_direct, rel_err)) => {
                println!(
                    "  │ {:>6.0}  {:>8}  {:>12.4}  {:>12.4}  {:>10.6}",
                    x,
                    zeros_1000.len(),
                    psi_explicit,
                    psi_direct,
                    rel_err
                );
            }
            Err(e) => println!("  │ {:>6.0}  ERROR: {e}", x),
        }
    }

    // Also test with varying truncation depths at x=500
    println!("  │");
    println!("  │ Truncation study at x=500:");
    println!(
        "  │ {:>8}  {:>12}  {:>10}",
        "N_zeros", "ψ_explicit", "Rel Error"
    );
    for &n in &[10, 25, 50, 100, 200, 400] {
        if n > zeros_1000.len() {
            continue;
        }
        let subset = &zeros_1000[..n];
        match nexcore_zeta::explicit::explicit_psi_comparison(500.0, subset) {
            Ok((psi_explicit, _psi_direct, rel_err)) => {
                println!("  │ {:>8}  {:>12.4}  {:>10.6}", n, psi_explicit, rel_err);
            }
            Err(e) => println!("  │ {:>8}  ERROR: {e}", n),
        }
    }
    println!("  └────────────────────────────────────────");

    // ── Experiment 5: Zero Distribution Statistics ─────────────────
    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  EXPERIMENT 5: ZERO DISTRIBUTION — RAW DATA");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  ┌─ First 20 zeros ──────────────────────────");
    println!("  │ {:>4}  {:>12}  {:>10}  {:>8}", "#", "t", "Z(t)", "Gap");
    for (i, z) in zeros_1000.iter().take(20).enumerate() {
        let gap = if i > 0 {
            z.t - zeros_1000[i - 1].t
        } else {
            0.0
        };
        println!(
            "  │ {:>4}  {:>12.6}  {:>10.6}  {:>8.4}",
            z.ordinal, z.t, z.z_value, gap
        );
    }
    println!("  │");
    println!("  │ Gap statistics (all {} zeros):", zeros_1000.len());
    if zeros_1000.len() > 1 {
        let gaps: Vec<f64> = zeros_1000.windows(2).map(|w| w[1].t - w[0].t).collect();
        let mean_gap: f64 = gaps.iter().sum::<f64>() / gaps.len() as f64;
        let min_gap = gaps.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_gap = gaps.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let var_gap: f64 =
            gaps.iter().map(|g| (g - mean_gap).powi(2)).sum::<f64>() / gaps.len() as f64;
        println!("  │   Mean gap:  {:.6}", mean_gap);
        println!("  │   Min gap:   {:.6}", min_gap);
        println!("  │   Max gap:   {:.6}", max_gap);
        println!("  │   Gap stdev: {:.6}", var_gap.sqrt());
        println!(
            "  │   Gap CV:    {:.4} (coefficient of variation)",
            var_gap.sqrt() / mean_gap
        );
    }
    println!("  └────────────────────────────────────────");

    println!();
    println!("═══════════════════════════════════════════════════════════════");
    println!("  RESEARCH EXTRACTION COMPLETE");
    println!("═══════════════════════════════════════════════════════════════");
}
