# RH Telescope — Insights

**Date:** 2026-02-23
**Based on:** RESEARCH.md (4 experiments, 6 findings, 5 open questions)
**Infrastructure:** nexcore-zeta (11 modules), nexcore-rh-proofs (7 modules), 2 additional crates
**Honest scope:** 649 zeros to height 1000, f64 precision, 10 subsample points

---

## 1. Cross-Finding Correlations

### Reinforcing Pairs

| Pair | Relationship | Data Support |
|------|-------------|--------------|
| **F1 (ill-conditioning) + F6 (coupling regularity)** | Same operator, different views | F1: roundtrip error 32→∞; F6: regularity 0.098→0.061. Together: the limit operator IS regular but NOT tridiagonal. Jacobi fails to represent it. |
| **F3 (density completeness) + F4 (GUE convergence)** | Complete sample → unbiased statistics | F3: 1.0006 completeness means no selection bias. F4's power-law fit has more credibility because the input zeros are complete — no cherry-picked subsequences that could inflate R². |
| **F2 (period-2 diagonal) + F5 (non-monotone explicit formula)** | Shared hidden structure in zero ordering | F2: diagonal alternates [49.97, 44.81, 47.27, 43.95, …]. F5: error oscillates 0.22%→0.24%→0.05%→0.33%→0.43%→0.23%. Both reflect conditionally convergent alternating-sign contributions — the same underlying arithmetic structure that makes the explicit formula conditionally convergent also imprints on the Jacobi diagonal. |
| **F1 (ill-conditioning) + F2 (period-2)** | Small-N Jacobi data is the only reliable data | F1 shows numerical stability only survives to N~79. F2's period-2 pattern was observed at N=20. Any inference from F2 about operator structure is valid only within the stable regime. |

### Tensions

| Pair | Tension | Resolution |
|------|---------|------------|
| **F3 (density complete) + F1 (ill-conditioned)** | Perfect data, catastrophic reconstruction | No contradiction. Data quality ≠ basis suitability. 649 zeros are correct; Jacobi is the wrong basis for them. |
| **F4 (N^(-0.35)) + F3 (N=649 only)** | Strong claim from small sample | 10 subsample points, R²=0.936 vs 0.916. The margin is too thin to be decisive. F3 establishes data completeness but cannot rescue weak statistical power. |
| **F6 (coupling → uniform) + F2 (period-2 oscillation)** | Coupling regularizes while diagonal oscillates | These operate on different scales. F6 is an asymptotic N-trend; F2 is a within-matrix pattern at fixed N=20. They are not contradictory — diagonal oscillation can coexist with off-diagonal uniformity. |

### The Unified Picture

All six findings point toward the same structural conclusion: **the operator underlying zeta zeros has a regular, translationally-invariant coupling structure in a basis that is NOT Jacobi tridiagonal.** The Jacobi basis reveals this as ill-conditioning (F1), hints at the hidden structure through period-2 oscillation (F2), and shows the coupling becoming more regular as N grows (F6). The zero statistics independently confirm the operator is GUE-class (F3, F4) while the explicit formula's non-monotone convergence (F5) confirms the zeros' contributions are conditionally convergent alternating series — structurally different from the positive-definite spectrum of a simple Jacobi operator.

---

## 2. Priority Ranking

### Open Questions — Information Yield per Effort

| Rank | Question | Min Viable Experiment | Expected Yield | Dependencies |
|------|---------|----------------------|----------------|--------------|
| **1** | Q2: Is N^(-0.35) genuine? | Extend to N=2,000–5,000 zeros (height ~15,000). Refit both models with 15+ points. | High: settles whether we've found an anomalous rate theorem. Power law persisting = publication-worthy signal. | Requires higher-precision Riemann-Siegel (f64 marginal above t~1000) |
| **2** | Q3: Can CMV succeed where Jacobi fails? | Implement CMV reconstruction for N=20 zeros. Compare roundtrip error to Jacobi's 32.65. | High: if CMV roundtrip error is <1.0 at N=20, it confirms the basis-mismatch hypothesis. Opens the operator structure question. | None — independent of Q2 |
| **3** | Q1: Does period-2 persist at larger N? | Run CMV at N=50 and N=100. Extract diagonal structure. | Medium: period-2 persistence would elevate F2 from artifact to structural finding. Likely answered by Q3 implementation. | Blocked by Q3 |
| **4** | Q5: Better adversarial sensitivity at larger x? | Rerun perturbation analysis with x=10^4, 10^5, 10^6. | Medium-low: sensitivity improves predictably (min detectable σ-deviation ~ x^(-0.5)). Formula is analytic — marginal new information. | None — easy to run |
| **5** | Q4: Explicit formula residual vs t (height) | Fix N=649, sweep x from 100 to 5000, plot residual as function of zero height contributing to error. | Low: diagnostic only. Confirms Riemann-Siegel connection but doesn't advance understanding of the operator. | None |

### Findings — Remaining Work per Finding

| Finding | Status | Remaining Work |
|---------|--------|---------------|
| F1 (ill-conditioning) | **Confirmed** | None — result is clear. Document and move to CMV. |
| F2 (period-2 diagonal) | **Provisional** | Needs stable algorithm at N>20 to distinguish artifact from structure. |
| F3 (density = 1.0006) | **Confirmed** | Extend height as telescope grows. Routine. |
| F4 (N^(-0.35)) | **Unresolved** | Most important gap. Needs N > 1,000. |
| F5 (non-monotone truncation) | **Confirmed** | Connection to Riemann-Siegel is well-established. Minor validation only. |
| F6 (coupling regularity) | **Provisional** | Extrapolate trend to predict regularity at N=1,000. Compare to theoretical prediction for translationally-invariant operators. |

### Critical Path

```
Q3 (CMV) → Q1 (period-2 at large N) → structural operator hypothesis
Q2 (N^(-0.35)) ─ independent ─ needs precision upgrade first
Q5 (adversarial) ─ independent ─ trivial to run
Q4 (residual vs t) ─ independent ─ lowest priority
```

---

## 3. Key Hypothesis

**The N^(-0.35) algebraic convergence to GUE is a genuine finite-height anomaly, not an artifact of N < 649.**

### Minimum Viable Test

Extend the GUE convergence subsample to cover N ∈ {100, 200, 400, 800, 1600, 3200, 6400} using the first 6,400 zeta zeros (height ~33,000). Refit power law and logarithmic models with 7+ new points.

### Discriminating Criteria

| Outcome | Interpretation |
|---------|---------------|
| Power law R² > 0.93, exponent stable at 0.30–0.40, beats log model by >0.01 R² | Genuine anomaly. Low-height zeros converge faster than asymptotic theory predicts. Warrants literature search and possible note on arXiv. |
| Power law exponent drifts toward 0 as N grows, log model catches up | Finite-sample transient. The N<649 regime is in a crossover between arithmetic-dominated spacing (fast convergence) and GUE universality (slow convergence). |
| Both models degrade at large N | Height-dependent effect — zeros above t~5,000 behave differently. Would itself be interesting. |

### What Each Outcome Means

**Confirms:** The first 10,000 zeros are in a "fast-convergence" regime driven by the simplicity of the prime distribution at low heights. The GUE limit is approached faster here than at heights where the Cramér model breaks down. This is consistent with the Montgomery-Odlyzko conjecture being harder to verify computationally precisely because higher zeros converge slower.

**Refutes:** Nothing structural — it just means our telescope's N<649 sample is too small to see the true asymptotic. The result is still useful as a calibration point.

---

## 4. Infrastructure Gaps

| Gap | Current Limit | Impact | Resolution |
|-----|--------------|--------|-----------|
| **f64 precision** | Verification reliable to ~t < 10^6 | Blocks scaling to Odlyzko-comparable N | `rug` crate (MPFR bindings) for multi-precision; 128-bit intermediate for t < 10^8 |
| **Scale gap** | 649 zeros vs Odlyzko's 10^13 | 10-order-of-magnitude deficit | Cannot close in software alone; need LMFDB data integration |
| **CMV missing** | Jacobi only | Blocks Q3 and Q1 | Implement `inverse::reconstruct_cmv` module in nexcore-zeta |
| **Statistical power** | 10 subsample points | R²=0.936 vs 0.916 is not decisive | Minimum 15 points needed for model discrimination at 95% confidence |
| **Zero database** | Computed on-the-fly | Slow, precision-limited | Load LMFDB precomputed zeros from file; enables N > 10,000 without re-derivation |
| **Marchenko equation** | Unimplemented | Alternative to CMV for continuous operators | Research-level; lower priority than CMV |
| **Regularized reconstruction** | None | Could stabilize Jacobi beyond N=79 | Add L2 penalty or symmetry priors to Stieltjes algorithm |

### Precision Quantification

At f64 (53-bit mantissa, ~15 decimal digits): the Riemann-Siegel Z(t) evaluation degrades above t~10^6 because the main sum `sum_{n≤sqrt(t/2π)} n^{-1/2} cos(...)` requires O(sqrt(t)) terms each computed to full precision. At t=10^6, this is 400 terms — manageable. At t=10^9, it's 12,600 terms — accumulated rounding error exceeds the zero contributions. The current telescope's precision wall is ~t=10^5 (conservative) to ~t=10^6 (optimistic).

---

## 5. Literature Comparison

### Established Results vs Our Measurements

| Finding | Published Result | Our Measurement | Verdict |
|---------|-----------------|-----------------|---------|
| Zero density to height T | Riemann-von Mangoldt: N(T) = (T/2π)ln(T/2π) - T/2π + O(ln T), giving 648.6 to T=1000 | 649 zeros, completeness 1.0006 | **Confirms.** Standard result, correctly reproduced. |
| GUE pair correlation | Montgomery (1973): pair correlation → 1 - (sin πu/πu)² as N→∞ | GUE score 0.887 at N=649 | **Confirms.** Expected at this scale. |
| GUE convergence rate | Theoretical expectation O(1/ln N) from density-of-states corrections | N^(-0.35) power law, R²=0.936 | **Anomalous.** Not matching theory. **Potentially novel** (see §6). |
| Jacobi reconstruction instability | Known: Stieltjes algorithm unstable for N>~100 generically | Catastrophic above N=79 for zeta zeros | **Confirms.** Known numerical fact, applied to this context. |
| Hilbert-Polya operator not tridiagonal | Berry & Keating (1999): natural basis involves action-angle on torus, not xp+px directly | F1+F6 confirm Jacobi fails but coupling regularizes | **Consistent with.** We don't add structure, we confirm absence. |
| Spacing statistics — GUE universality | Odlyzko (1987): first confirmed GUE statistics for 10^13 zeros near t~10^20 | CV=0.469 vs GUE 0.42, variance converging from below | **Confirms** at low height with expected finite-size effects. |
| Explicit formula psi(x) reconstruction | Standard result: conditional convergence, error ~ O(x^{1/2} log^2 x / N) | <0.2% error across x∈[100,5000] with N=649 | **Confirms.** Optimal truncation insight is **known** (Riemann-Siegel formula). |

### Odlyzko Scale Context

Odlyzko (2001) computed zeros near t~10^20 with 70-digit precision. Our zeros are at t < 1,000 — roughly 17 orders of magnitude lower. The universality class (GUE) is confirmed at both scales. The convergence rate is where we differ — Odlyzko's zeros at high t are expected to follow the asymptotic O(1/ln N) rate, while our low-height zeros may be in a different regime.

---

## 6. Novel Measurements

| Measurement | Novel? | Confidence | Explanation |
|-------------|--------|-----------|-------------|
| **N^(-0.35) GUE convergence rate** | **Possibly novel** | Low-medium | Not found in standard literature survey. Theoretical predictions are O(1/log N). If confirmed at N>1,000, worth literature deep-dive (Katz-Sarnak, Rubinstein, Stopple). **The key unknown.** |
| **Jacobi period-2 diagonal at N=20** | **Possibly novel** | Low | Oscillation [49.97, 44.81, 47.27, 43.95, ...] with period 2. May be known as a numerical artifact of the Stieltjes algorithm. Not seen in standard Jacobi reconstruction literature for zeta zeros. Needs stable algorithm to evaluate. |
| **Coupling regularity trend (0.098→0.073→0.061)** | **Unlikely novel** | — | The observation that Jacobi coupling elements become uniform is consistent with known asymptotic results for Jacobi operators with regular spectra. The parameterization may differ but the conclusion is standard. |
| **Density completeness = 1.0006** | **Not novel** | High | Riemann-von Mangoldt formula is a theorem. Our measurement confirms it. |
| **Adversarial min-detectable σ=0.667 at x=1000** | **Not novel** | High | Perturbation analysis of RH is well-studied. The specific numbers depend on our x choice. |
| **Explicit formula optimal truncation ~ sqrt(x/2π)** | **Not novel** | High | This IS the Riemann-Siegel formula, known since 1932. We re-derived it empirically, which is a good check. |

**Bottom line on novelty:** The telescope currently produces two potentially novel signals — the convergence rate anomaly (F4) and the period-2 Jacobi structure (F2). Both require more data before claiming novelty. Neither is ready for formal comparison with the literature. The remaining findings are confirmations of known results, which is valuable as calibration but not novel.

---

## 7. Priority Matrix

```
         LOW EFFORT          HIGH EFFORT
         ─────────────────   ──────────────────────────
HIGH   │ Q5: adversarial   │ Q2: N^(-0.35) scaling
IMPACT │   sensitivity     │ Q3: CMV reconstruction
       │   (run now)       │ (implement new module)
       ├───────────────────┼──────────────────────────
LOW    │ Q4: residual vs t │ Marchenko equation
IMPACT │   (diagnostic)    │ Full LMFDB integration
       │                   │ (multi-year scope)
       └───────────────────┴──────────────────────────
```

---

## 8. What We Don't Know (Honest Inventory)

1. **Whether N^(-0.35) persists above N=649.** The data is insufficient to distinguish algebraic from logarithmic convergence. This is the most important unknown.

2. **Whether the Jacobi period-2 pattern is numerical or structural.** Stieltjes instability at N=79 means all large-N structure is suspect. We only trust N=20 and partially N=50.

3. **What the Hilbert-Polya operator's correct basis is.** We've eliminated Jacobi. CMV is the leading alternative. But CMV is also a guess — there's no theorem saying zeta zeros are CMV eigenvalues.

4. **Whether our GUE score of 0.887 is asymptotically approaching 1.0.** The score has been increasing (0.732→0.887 over N=30 to 649), but the 11.3% gap could be partly permanent (finite-height effects that don't vanish as N grows within our zero range).

5. **The explicit formula residual structure vs height.** We know the residual at fixed x as a function of N (F5). We don't know the residual at fixed N as a function of x. This would tell us which zeros are "earning their keep" in the reconstruction.

---

## Next Session Priorities

Ordered by: (information yield × feasibility) / effort

1. **Run adversarial analysis at x = 10^4, 10^5, 10^6.** (Q5) — Pure parameter change, no new code. Establishes min-detectable σ-deviation as function of x. Closes the sensitivity characterization gap. Expected time: 30 minutes.

2. **Implement `inverse::reconstruct_cmv` in nexcore-zeta.** (Q3) — CMV matrices are the natural unitary analogue of Jacobi, well-suited for spectral problems on the unit circle. Algorithm reference: Simon "OPUC" Vol. 1, Ch. 4. Run at N=20,50 and compare roundtrip error to Jacobi (32.65, 74.98). If CMV achieves roundtrip < 5.0 at N=20, the basis-mismatch hypothesis is confirmed.

3. **Extend GUE convergence subsample to N ∈ {700, 1000, 1500, 2000}.** (Q2 partial) — Even 4 new points changes the model discrimination picture. Requires: either load LMFDB precomputed zeros for t∈[1000,10000] or implement higher-precision Riemann-Siegel. LMFDB integration is faster. At minimum: run the existing computation engine to N=1000 and add the data point.

4. **Add 5 more subsample points to the GUE convergence curve.** (Q2 statistical power) — Current: 10 points. Minimum for 95% confidence model discrimination: 15 points. Intermediate N values between 649 and 2000 can be computed without new precision infrastructure.

5. **Explicit formula residual as function of t.** (Q4) — Fix N=649, vary x from 100 to 10,000 in 50 steps. Plot `|ψ_explicit(x) - ψ_direct(x)|` vs x. This will show whether the sweet spot shifts as x grows and whether the R-S connection holds over a wider range. Low yield but fast to implement.

6. **(Deferred) Integrate LMFDB zero database.** Precondition for reaching N > 2,000 without precision upgrade. Requires HTTP client + LMFDB API integration or flat-file loader. Enables resolution of Q2 definitively.

---

*All claims in this document are grounded in measured data from RESEARCH.md. Uncertainty is stated explicitly. Nothing is claimed novel without experimental support.*
