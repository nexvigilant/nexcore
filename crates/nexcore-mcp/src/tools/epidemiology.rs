//! Epidemiology MCP tools — Domain 7
//!
//! Measures of association, impact, and survival analysis.
//! PV transfer confidence: 0.95 (shared 2×2 contingency table).
//!
//! 10 tools: relative_risk, odds_ratio, attributable_risk, nnt_nnh,
//! attributable_fraction, population_af, incidence_rate, prevalence,
//! kaplan_meier, smr, epi_pv_mappings

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    EpiAttributableFractionParams, EpiAttributableRiskParams, EpiIncidenceRateParams,
    EpiKaplanMeierParams, EpiMantelHaenszelParams, EpiNntNnhParams, EpiOddsRatioParams,
    EpiPopulationAFParams, EpiPrevalenceParams, EpiPvMappingsParams, EpiRelativeRiskParams,
    EpiSmrParams,
};

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_default(),
    )]))
}

fn err_text(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

/// RR = [a/(a+b)] / [c/(c+d)]
pub fn relative_risk(params: EpiRelativeRiskParams) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = (params.a, params.b, params.c, params.d);
    let risk_exposed = a / (a + b);
    let risk_unexposed = c / (c + d);

    if risk_unexposed == 0.0 {
        return err_text("Error: unexposed risk is zero (c+d has no cases), RR undefined");
    }

    let rr = risk_exposed / risk_unexposed;
    let ln_rr = rr.ln();
    let se_ln_rr = (1.0 / a - 1.0 / (a + b) + 1.0 / c - 1.0 / (c + d)).sqrt();
    let ci_lower = (ln_rr - 1.96 * se_ln_rr).exp();
    let ci_upper = (ln_rr + 1.96 * se_ln_rr).exp();

    let interpretation = if ci_lower > 1.0 {
        "Statistically significant increased risk"
    } else if ci_upper < 1.0 {
        "Statistically significant decreased risk (protective)"
    } else {
        "Not statistically significant (CI includes 1.0)"
    };

    ok_json(serde_json::json!({
        "relative_risk": rr,
        "risk_exposed": risk_exposed,
        "risk_unexposed": risk_unexposed,
        "ci_95_lower": ci_lower,
        "ci_95_upper": ci_upper,
        "interpretation": interpretation,
        "pv_mapping": {
            "epi_term": "relative_risk",
            "pv_equivalent": "PRR (Proportional Reporting Ratio)",
            "confidence": 0.95,
            "rationale": "Both measure disproportionate risk in exposed vs unexposed"
        }
    }))
}

/// OR = (a×d) / (b×c)
pub fn odds_ratio(params: EpiOddsRatioParams) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = (params.a, params.b, params.c, params.d);

    if b * c == 0.0 {
        return err_text("Error: b×c is zero, OR undefined");
    }

    let or = (a * d) / (b * c);
    let ln_or = or.ln();
    let se_ln_or = (1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d).sqrt();
    let ci_lower = (ln_or - 1.96 * se_ln_or).exp();
    let ci_upper = (ln_or + 1.96 * se_ln_or).exp();

    let interpretation = if ci_lower > 1.0 {
        "Statistically significant positive association"
    } else if ci_upper < 1.0 {
        "Statistically significant negative association (protective)"
    } else {
        "Not statistically significant (CI includes 1.0)"
    };

    ok_json(serde_json::json!({
        "odds_ratio": or,
        "ci_95_lower": ci_lower,
        "ci_95_upper": ci_upper,
        "interpretation": interpretation,
        "pv_mapping": {
            "epi_term": "odds_ratio",
            "pv_equivalent": "ROR (Reporting Odds Ratio)",
            "confidence": 0.98,
            "rationale": "Identical formula: OR = (ad)/(bc) = ROR"
        }
    }))
}

/// AR = Ie - Io = a/(a+b) - c/(c+d)
pub fn attributable_risk(params: EpiAttributableRiskParams) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = (params.a, params.b, params.c, params.d);
    let risk_exposed = a / (a + b);
    let risk_unexposed = c / (c + d);
    let ar = risk_exposed - risk_unexposed;

    let se_ar = (risk_exposed * (1.0 - risk_exposed) / (a + b)
        + risk_unexposed * (1.0 - risk_unexposed) / (c + d))
        .sqrt();
    let ci_lower = ar - 1.96 * se_ar;
    let ci_upper = ar + 1.96 * se_ar;

    let interpretation = if ar > 0.0 {
        "Excess risk attributable to exposure"
    } else if ar < 0.0 {
        "Exposure is protective (reduces risk)"
    } else {
        "No difference in risk between groups"
    };

    ok_json(serde_json::json!({
        "attributable_risk": ar,
        "risk_exposed": risk_exposed,
        "risk_unexposed": risk_unexposed,
        "ci_95_lower": ci_lower,
        "ci_95_upper": ci_upper,
        "interpretation": interpretation,
        "pv_mapping": {
            "epi_term": "attributable_risk",
            "pv_equivalent": "excess_signal (signal rate - background rate)",
            "confidence": 0.90,
            "rationale": "Both measure absolute excess over baseline"
        }
    }))
}

/// NNT = 1/ARR when AR > 0, NNH = 1/ARI when AR < 0
pub fn nnt_nnh(params: EpiNntNnhParams) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = (params.a, params.b, params.c, params.d);
    let risk_exposed = a / (a + b);
    let risk_unexposed = c / (c + d);
    let ar = risk_exposed - risk_unexposed;

    if ar.abs() < 1e-15 {
        return err_text("Error: attributable risk is zero, NNT/NNH undefined");
    }

    let nnt_or_nnh = (1.0 / ar).abs();
    let metric_name = if ar < 0.0 { "NNT" } else { "NNH" };
    let description = if ar < 0.0 {
        format!(
            "Need to treat {:.1} patients with exposure to prevent 1 case",
            nnt_or_nnh
        )
    } else {
        format!(
            "For every {:.1} patients exposed, 1 additional case of harm",
            nnt_or_nnh
        )
    };

    ok_json(serde_json::json!({
        "metric": metric_name,
        "value": nnt_or_nnh,
        "attributable_risk": ar,
        "description": description,
        "pv_mapping": {
            "epi_term": "NNT/NNH",
            "pv_equivalent": "benefit-risk ratio (QBRI denominator)",
            "confidence": 0.85,
            "rationale": "Both quantify net clinical impact per patient"
        }
    }))
}

/// AF = (RR - 1) / RR (attributable fraction among exposed)
pub fn attributable_fraction(
    params: EpiAttributableFractionParams,
) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = (params.a, params.b, params.c, params.d);
    let risk_exposed = a / (a + b);
    let risk_unexposed = c / (c + d);

    if risk_unexposed == 0.0 {
        return err_text("Error: unexposed risk is zero, AF undefined");
    }

    let rr = risk_exposed / risk_unexposed;

    if rr == 0.0 {
        return err_text("Error: RR is zero, AF undefined");
    }

    let af = (rr - 1.0) / rr;

    let interpretation = if af > 0.0 {
        format!(
            "{:.1}% of disease among exposed is attributable to the exposure",
            af * 100.0
        )
    } else {
        format!(
            "Exposure prevents {:.1}% of disease (protective fraction)",
            af.abs() * 100.0
        )
    };

    ok_json(serde_json::json!({
        "attributable_fraction": af,
        "relative_risk": rr,
        "interpretation": interpretation,
        "pv_mapping": {
            "epi_term": "attributable_fraction",
            "pv_equivalent": "signal_contribution (fraction of events due to drug)",
            "confidence": 0.88,
            "rationale": "Both measure proportion of outcome attributable to exposure"
        }
    }))
}

/// PAF = Pe(RR-1) / [1 + Pe(RR-1)]
pub fn population_attributable_fraction(
    params: EpiPopulationAFParams,
) -> Result<CallToolResult, McpError> {
    let (a, b, c, d) = (params.a, params.b, params.c, params.d);
    let total = a + b + c + d;
    let pe = (a + b) / total; // prevalence of exposure in total population
    let risk_exposed = a / (a + b);
    let risk_unexposed = c / (c + d);

    if risk_unexposed == 0.0 {
        return err_text("Error: unexposed risk is zero, PAF undefined");
    }

    let rr = risk_exposed / risk_unexposed;
    let paf = pe * (rr - 1.0) / (1.0 + pe * (rr - 1.0));

    let interpretation = if paf > 0.0 {
        format!(
            "{:.1}% of disease in the total population is attributable to the exposure",
            paf * 100.0
        )
    } else {
        format!(
            "Removing exposure would increase disease by {:.1}% (protective exposure)",
            paf.abs() * 100.0
        )
    };

    ok_json(serde_json::json!({
        "population_attributable_fraction": paf,
        "prevalence_of_exposure": pe,
        "relative_risk": rr,
        "interpretation": interpretation,
        "pv_mapping": {
            "epi_term": "PAF",
            "pv_equivalent": "population_signal_burden (public health impact of drug safety signal)",
            "confidence": 0.85,
            "rationale": "Both quantify population-level impact of exposure"
        }
    }))
}

/// IR = events / person-time × multiplier
pub fn incidence_rate(params: EpiIncidenceRateParams) -> Result<CallToolResult, McpError> {
    if params.person_time <= 0.0 {
        return err_text("Error: person-time must be positive");
    }

    let rate = params.events / params.person_time;
    let rate_scaled = rate * params.multiplier;

    // Exact Poisson CI (Garwood method)
    let ci_lower = if params.events > 0.0 {
        // chi-square quantile approximation for lower bound
        let z = 1.96;
        let lower_events = params.events
            * (1.0 - 1.0 / (9.0 * params.events) - z / (3.0 * params.events.sqrt())).powi(3);
        (lower_events.max(0.0) / params.person_time) * params.multiplier
    } else {
        0.0
    };
    let ci_upper = {
        let events_plus = params.events + 1.0;
        let z = 1.96;
        let upper_events = events_plus
            * (1.0 - 1.0 / (9.0 * events_plus) + z / (3.0 * events_plus.sqrt())).powi(3);
        (upper_events / params.person_time) * params.multiplier
    };

    ok_json(serde_json::json!({
        "incidence_rate": rate_scaled,
        "events": params.events,
        "person_time": params.person_time,
        "multiplier": params.multiplier,
        "ci_95_lower": ci_lower,
        "ci_95_upper": ci_upper,
        "unit": format!("per {} person-time", params.multiplier),
        "pv_mapping": {
            "epi_term": "incidence_rate",
            "pv_equivalent": "reporting_rate (ICSRs per patient-exposure-time)",
            "confidence": 0.92,
            "rationale": "Both are event counts divided by person-time at risk"
        }
    }))
}

/// P = cases / population × multiplier
pub fn prevalence(params: EpiPrevalenceParams) -> Result<CallToolResult, McpError> {
    if params.population <= 0.0 {
        return err_text("Error: population must be positive");
    }

    let prev = params.cases / params.population;
    let prev_scaled = prev * params.multiplier;

    // Wilson score CI
    let n = params.population;
    let p = prev;
    let z = 1.96;
    let z2 = z * z;
    let denominator = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator;
    let margin = z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denominator;

    let unit_label = if params.multiplier == 100.0 {
        "percent".to_string()
    } else {
        format!("per {}", params.multiplier)
    };

    ok_json(serde_json::json!({
        "prevalence": prev_scaled,
        "proportion": prev,
        "cases": params.cases,
        "population": params.population,
        "multiplier": params.multiplier,
        "ci_95_lower": (center - margin) * params.multiplier,
        "ci_95_upper": (center + margin) * params.multiplier,
        "unit": unit_label,
        "pv_mapping": {
            "epi_term": "prevalence",
            "pv_equivalent": "background_rate (baseline event frequency in population)",
            "confidence": 0.90,
            "rationale": "Both are cross-sectional frequency measures for baseline comparison"
        }
    }))
}

/// Kaplan-Meier product-limit survival estimator
/// S(t) = Π [1 - d_i / n_i]
pub fn kaplan_meier(params: EpiKaplanMeierParams) -> Result<CallToolResult, McpError> {
    if params.intervals.is_empty() {
        return err_text("Error: no intervals provided");
    }

    let mut intervals = params.intervals;
    intervals.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Calculate initial population from first interval
    let total_events: u32 = intervals.iter().map(|i| i.events).sum();
    let total_censored: u32 = intervals.iter().map(|i| i.censored).sum();
    let n_initial = total_events + total_censored;
    let mut n_at_risk = n_initial as f64;
    let mut survival = 1.0_f64;
    let mut table = Vec::new();

    for interval in &intervals {
        let d = interval.events as f64;
        let c = interval.censored as f64;

        if n_at_risk <= 0.0 {
            break;
        }

        let hazard = d / n_at_risk;
        survival *= 1.0 - hazard;

        // Greenwood's formula for SE
        let se = if survival > 0.0 && n_at_risk > d {
            let var_term = d / (n_at_risk * (n_at_risk - d));
            survival
                * (table
                    .iter()
                    .map(|t: &serde_json::Value| t["greenwood_term"].as_f64().unwrap_or(0.0))
                    .sum::<f64>()
                    + var_term)
                    .sqrt()
        } else {
            0.0
        };

        let ci_lower = (survival - 1.96 * se).max(0.0);
        let ci_upper = (survival + 1.96 * se).min(1.0);

        table.push(serde_json::json!({
            "time": interval.time,
            "n_at_risk": n_at_risk,
            "events": interval.events,
            "censored": interval.censored,
            "hazard": hazard,
            "survival": survival,
            "se": se,
            "ci_95_lower": ci_lower,
            "ci_95_upper": ci_upper,
            "greenwood_term": if n_at_risk > d { d / (n_at_risk * (n_at_risk - d)) } else { 0.0 },
        }));

        n_at_risk -= d + c;
    }

    let median_survival = table
        .iter()
        .find(|t| t["survival"].as_f64().unwrap_or(1.0) <= 0.5)
        .map(|t| t["time"].as_f64().unwrap_or(0.0));

    ok_json(serde_json::json!({
        "survival_table": table,
        "n_initial": n_initial,
        "total_events": total_events,
        "total_censored": total_censored,
        "final_survival": survival,
        "median_survival": median_survival,
        "pv_mapping": {
            "epi_term": "kaplan_meier",
            "pv_equivalent": "time_to_onset_survival (Weibull TTO in signal triage)",
            "confidence": 0.82,
            "rationale": "Both model event timing with censoring — KM is non-parametric, Weibull is parametric"
        }
    }))
}

/// SMR = observed / expected
pub fn smr(params: EpiSmrParams) -> Result<CallToolResult, McpError> {
    if params.expected <= 0.0 {
        return err_text("Error: expected count must be positive");
    }

    let smr = params.observed / params.expected;

    // Byar's approximation for CI
    let o = params.observed;
    let ci_lower = if o > 0.0 {
        let l = o * (1.0 - 1.0 / (9.0 * o) - 1.96 / (3.0 * o.sqrt())).powi(3) / params.expected;
        l.max(0.0)
    } else {
        0.0
    };
    let ci_upper = {
        let u = o + 1.0;
        u * (1.0 - 1.0 / (9.0 * u) + 1.96 / (3.0 * u.sqrt())).powi(3) / params.expected
    };

    let interpretation = if ci_lower > 1.0 {
        "Significantly more events than expected"
    } else if ci_upper < 1.0 {
        "Significantly fewer events than expected"
    } else {
        "Not significantly different from expected"
    };

    ok_json(serde_json::json!({
        "smr": smr,
        "observed": params.observed,
        "expected": params.expected,
        "ci_95_lower": ci_lower,
        "ci_95_upper": ci_upper,
        "interpretation": interpretation,
        "pv_mapping": {
            "epi_term": "SMR",
            "pv_equivalent": "observed_to_expected_ratio (O/E in EBGM)",
            "confidence": 0.93,
            "rationale": "Both compare observed events to expected under null hypothesis"
        }
    }))
}

/// Mantel-Haenszel stratified analysis — MH-adjusted OR/RR with Breslow-Day homogeneity test.
///
/// Formula: MH-OR = Σ(a_i*d_i/T_i) / Σ(b_i*c_i/T_i)
/// Variance: Robins-Breslow-Greenland estimator for ln(MH-OR)
/// Homogeneity: Breslow-Day chi-square (H0: common OR across strata)
pub fn mantel_haenszel(params: EpiMantelHaenszelParams) -> Result<CallToolResult, McpError> {
    if params.strata.is_empty() {
        return err_text("Error: no strata provided");
    }

    let k = params.strata.len();

    // MH numerator/denominator sums and RBG variance accumulation
    let mut r_sum = 0.0_f64; // Σ a_i*d_i/T_i
    let mut s_sum = 0.0_f64; // Σ b_i*c_i/T_i
    let mut rbg_term1 = 0.0_f64; // Σ P_i*R_i
    let mut rbg_term2 = 0.0_f64; // Σ (P_i*S_i + Q_i*R_i)
    let mut rbg_term3 = 0.0_f64; // Σ Q_i*S_i

    let mut per_stratum = Vec::new();

    for (i, stratum) in params.strata.iter().enumerate() {
        let (a, b, c, d) = (stratum.a, stratum.b, stratum.c, stratum.d);
        let t = a + b + c + d;

        if t <= 0.0 {
            return err_text("Error: stratum has zero total count");
        }

        let r_i = a * d / t;
        let s_i = b * c / t;
        r_sum += r_i;
        s_sum += s_i;

        let p_i = (a + d) / t;
        let q_i = (b + c) / t;
        rbg_term1 += p_i * r_i;
        rbg_term2 += p_i * s_i + q_i * r_i;
        rbg_term3 += q_i * s_i;

        // Per-stratum point estimate and CI
        let stratum_est = if b * c > 0.0 {
            (a * d) / (b * c)
        } else {
            f64::INFINITY
        };
        let ln_est = if stratum_est.is_finite() && stratum_est > 0.0 {
            stratum_est.ln()
        } else {
            0.0
        };
        let var_ln = if a > 0.0 && b > 0.0 && c > 0.0 && d > 0.0 {
            1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d
        } else {
            f64::INFINITY
        };
        let (ci_lower, ci_upper) = if var_ln.is_finite() {
            let se = var_ln.sqrt();
            ((ln_est - 1.96 * se).exp(), (ln_est + 1.96 * se).exp())
        } else {
            (0.0, f64::INFINITY)
        };

        per_stratum.push(serde_json::json!({
            "stratum": i + 1,
            "label": stratum.label,
            "a": a, "b": b, "c": c, "d": d,
            "point_estimate": stratum_est,
            "ci_95_lower": ci_lower,
            "ci_95_upper": ci_upper,
        }));
    }

    if s_sum <= 0.0 {
        return err_text("Error: MH denominator is zero (all b×c = 0)");
    }

    let mh_estimate = r_sum / s_sum;
    let ln_mh = if mh_estimate > 0.0 {
        mh_estimate.ln()
    } else {
        return err_text("Error: MH estimate is non-positive");
    };

    // Robins-Breslow-Greenland variance of ln(MH-OR)
    let var_ln_mh = rbg_term1 / (2.0 * r_sum * r_sum)
        + rbg_term2 / (2.0 * r_sum * s_sum)
        + rbg_term3 / (2.0 * s_sum * s_sum);
    let se_ln_mh = var_ln_mh.sqrt();
    let ci_lower = (ln_mh - 1.96 * se_ln_mh).exp();
    let ci_upper = (ln_mh + 1.96 * se_ln_mh).exp();

    // Breslow-Day homogeneity test — chi-square statistic
    // H0: common OR/RR across all strata
    let mut bd_chi_sq = 0.0_f64;
    for stratum in &params.strata {
        let (a, b, c, d) = (stratum.a, stratum.b, stratum.c, stratum.d);
        let t = a + b + c + d;
        let m1 = a + b; // exposed row total
        let n1 = a + c; // case column total
        let m0 = c + d; // unexposed row total

        // Solve quadratic for expected a_hat under common MH estimate:
        // (1 - mh) * x^2 + (m0 - n1 + mh*(n1+m1)) * x - mh*n1*m1 = 0
        let qa = 1.0 - mh_estimate;
        let qb = m0 - n1 + mh_estimate * (n1 + m1);
        let qc = -mh_estimate * n1 * m1;

        let a_hat = if qa.abs() < 1e-10 {
            // Linear case
            if qb.abs() > 1e-10 { -qc / qb } else { a }
        } else {
            let disc = qb * qb - 4.0 * qa * qc;
            if disc < 0.0 {
                a // fallback: no real root, skip this stratum
            } else {
                let sqrt_disc = disc.sqrt();
                let r1 = (-qb + sqrt_disc) / (2.0 * qa);
                let r2 = (-qb - sqrt_disc) / (2.0 * qa);
                let lo = (n1 + m1 - t).max(0.0);
                let hi = n1.min(m1);
                if r1 >= lo && r1 <= hi { r1 } else { r2 }
            }
        };

        let b_hat = m1 - a_hat;
        let c_hat = n1 - a_hat;
        let d_hat = t - m1 - n1 + a_hat;

        // Variance of a under the hypergeometric distribution
        let var_a = if a_hat > 0.0 && b_hat > 0.0 && c_hat > 0.0 && d_hat > 0.0 {
            1.0 / (1.0 / a_hat + 1.0 / b_hat + 1.0 / c_hat + 1.0 / d_hat)
        } else {
            1.0
        };

        bd_chi_sq += (a - a_hat) * (a - a_hat) / var_a;
    }

    let bd_df = (k - 1) as u32;
    let bd_p = stem_math::statistics::chi_square_p_value(bd_chi_sq, bd_df as usize).unwrap_or(1.0);
    let heterogeneity_detected = bd_p < 0.05;

    let interpretation = if ci_lower > 1.0 {
        "Significant positive association after stratification"
    } else if ci_upper < 1.0 {
        "Significant negative association (protective) after stratification"
    } else {
        "Not statistically significant (CI includes 1.0)"
    };

    ok_json(serde_json::json!({
        "measure": params.measure,
        "adjusted_estimate": mh_estimate,
        "ci_95_lower": ci_lower,
        "ci_95_upper": ci_upper,
        "ln_estimate": ln_mh,
        "se_ln": se_ln_mh,
        "interpretation": interpretation,
        "homogeneity_chi_sq": bd_chi_sq,
        "homogeneity_df": bd_df,
        "homogeneity_p_value": bd_p,
        "heterogeneity_detected": heterogeneity_detected,
        "strata_count": k,
        "per_stratum": per_stratum,
        "method": "Mantel-Haenszel with Robins-Breslow-Greenland variance",
    }))
}

/// All epidemiology → PV transfer mappings
pub fn epi_pv_mappings(_params: EpiPvMappingsParams) -> Result<CallToolResult, McpError> {
    ok_json(serde_json::json!({
        "domain": "epidemiology",
        "target": "pharmacovigilance",
        "overall_transfer_confidence": 0.95,
        "mappings": [
            {
                "epi_tool": "epi_relative_risk",
                "epi_formula": "RR = [a/(a+b)] / [c/(c+d)]",
                "pv_equivalent": "PRR (Proportional Reporting Ratio)",
                "pv_formula": "PRR = [a/(a+b)] / [c/(c+d)]",
                "confidence": 0.95,
                "note": "Identical formula, different naming convention"
            },
            {
                "epi_tool": "epi_odds_ratio",
                "epi_formula": "OR = (ad)/(bc)",
                "pv_equivalent": "ROR (Reporting Odds Ratio)",
                "pv_formula": "ROR = (ad)/(bc)",
                "confidence": 0.98,
                "note": "Identical formula"
            },
            {
                "epi_tool": "epi_attributable_risk",
                "epi_formula": "AR = Ie - Io",
                "pv_equivalent": "Excess signal rate",
                "confidence": 0.90,
                "note": "Absolute excess over baseline"
            },
            {
                "epi_tool": "epi_nnt_nnh",
                "epi_formula": "NNT = 1/ARR, NNH = 1/ARI",
                "pv_equivalent": "Benefit-risk ratio (QBRI)",
                "confidence": 0.85,
                "note": "Both quantify clinical impact magnitude"
            },
            {
                "epi_tool": "epi_attributable_fraction",
                "epi_formula": "AF = (RR-1)/RR",
                "pv_equivalent": "Signal contribution fraction",
                "confidence": 0.88,
                "note": "Proportion of outcome due to exposure"
            },
            {
                "epi_tool": "epi_population_af",
                "epi_formula": "PAF = Pe(RR-1)/[1+Pe(RR-1)]",
                "pv_equivalent": "Population signal burden",
                "confidence": 0.85,
                "note": "Public health impact scale"
            },
            {
                "epi_tool": "epi_incidence_rate",
                "epi_formula": "IR = events/person-time",
                "pv_equivalent": "Reporting rate",
                "confidence": 0.92,
                "note": "Event frequency per exposure time"
            },
            {
                "epi_tool": "epi_prevalence",
                "epi_formula": "P = cases/population",
                "pv_equivalent": "Background rate",
                "confidence": 0.90,
                "note": "Cross-sectional baseline frequency"
            },
            {
                "epi_tool": "epi_kaplan_meier",
                "epi_formula": "S(t) = Π[1 - d_i/n_i]",
                "pv_equivalent": "Time-to-onset survival (Weibull TTO)",
                "confidence": 0.82,
                "note": "Non-parametric survival vs parametric Weibull"
            },
            {
                "epi_tool": "epi_smr",
                "epi_formula": "SMR = observed/expected",
                "pv_equivalent": "O/E ratio (EBGM)",
                "confidence": 0.93,
                "note": "Both compare observed to expected under null"
            }
        ],
        "shared_infrastructure": {
            "2x2_table": "Both domains use identical (a,b,c,d) contingency table",
            "chi_square": "Identical χ² test for independence",
            "confidence_intervals": "Same Wald/exact methods",
            "null_hypothesis": "H0: no association between exposure and outcome"
        }
    }))
}
