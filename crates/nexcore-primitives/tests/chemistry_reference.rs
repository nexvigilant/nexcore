//! # Chemistry Reference Validation Tests
//!
//! These integration tests validate the chemistry module functions against
//! published textbook reference values. Each test includes:
//! - The published formula
//! - The source / standard reference
//! - The tolerance used for comparison
//!
//! Reference: Atkins' Physical Chemistry (12th ed.), IUPAC Gold Book,
//! NIST Chemistry WebBook.

use nexcore_primitives::chemistry;

// ============================================================================
// Helper: assert within relative tolerance
// ============================================================================

fn assert_relative(actual: f64, expected: f64, tolerance: f64, context: &str) {
    let rel_err = if expected.abs() < 1e-30 {
        actual.abs()
    } else {
        ((actual - expected) / expected).abs()
    };
    assert!(
        rel_err <= tolerance,
        "{context}: expected {expected:.6e}, got {actual:.6e}, relative error {rel_err:.6e} exceeds tolerance {tolerance:.6e}"
    );
}

fn assert_absolute(actual: f64, expected: f64, tolerance: f64, context: &str) {
    let abs_err = (actual - expected).abs();
    assert!(
        abs_err <= tolerance,
        "{context}: expected {expected:.6}, got {actual:.6}, absolute error {abs_err:.6} exceeds tolerance {tolerance:.6}"
    );
}

// ============================================================================
// 1. Arrhenius Equation: k = A * exp(-Ea / RT)
//    Source: Atkins' Physical Chemistry, R = 8.314 J/(mol*K)
// ============================================================================

#[test]
fn arrhenius_reference_standard_conditions() {
    // Formula: k = A * exp(-Ea / RT)
    // Source: Standard Arrhenius equation, R = 8.314 J/(mol*K)
    // Tolerance: 1% relative
    //
    // A = 1e13 s^-1, Ea = 100 kJ/mol, T = 298.15 K
    // exponent = -100000 / (8.314 * 298.15) = -100000 / 2478.82 = -40.340
    // k = 1e13 * exp(-40.340) = 1e13 * 2.942e-18 = 2.942e-5 s^-1
    let k = chemistry::arrhenius_rate(1e13, 100.0, 298.15).expect("valid Arrhenius parameters");

    let expected = 1e13 * (-100_000.0_f64 / (8.314 * 298.15)).exp();
    assert_relative(k, expected, 0.01, "Arrhenius at 298.15K, Ea=100 kJ/mol");
    // Verify order of magnitude
    assert!(k > 1e-6 && k < 1e-4, "k should be ~2.9e-5, got {k:.3e}");
}

#[test]
fn arrhenius_reference_low_barrier() {
    // Formula: k = A * exp(-Ea / RT)
    // Source: Standard Arrhenius equation
    // Tolerance: 1% relative
    //
    // A = 1e13, Ea = 50 kJ/mol, T = 298.15 K
    // exponent = -50000 / (8.314 * 298.15) = -20.170
    // k = 1e13 * exp(-20.170) = 1e13 * 1.725e-9 = 1.725e4
    let k = chemistry::arrhenius_rate(1e13, 50.0, 298.15).expect("valid Arrhenius parameters");

    let expected = 1e13 * (-50_000.0_f64 / (8.314 * 298.15)).exp();
    assert_relative(k, expected, 0.01, "Arrhenius at 298.15K, Ea=50 kJ/mol");
}

#[test]
fn arrhenius_temperature_dependence() {
    // Formula: k = A * exp(-Ea / RT)
    // Source: Arrhenius temperature dependence (van't Hoff rule: ~2-3x per 10K)
    // Tolerance: 1% relative
    //
    // Compare rate at 298K vs 308K for Ea = 75 kJ/mol
    let k_298 = chemistry::arrhenius_rate(1e13, 75.0, 298.15).expect("valid at 298K");
    let k_308 = chemistry::arrhenius_rate(1e13, 75.0, 308.15).expect("valid at 308K");

    // Higher temperature must give higher rate
    assert!(k_308 > k_298, "Rate must increase with temperature");

    // Verify ratio against exact calculation
    let expected_ratio =
        ((-75_000.0_f64 / (8.314 * 308.15)).exp()) / ((-75_000.0_f64 / (8.314 * 298.15)).exp());
    let actual_ratio = k_308 / k_298;
    assert_relative(
        actual_ratio,
        expected_ratio,
        0.01,
        "Arrhenius temperature ratio 308K/298K",
    );
}

// ============================================================================
// 2. Michaelis-Menten: v = Vmax * [S] / (Km + [S])
//    Source: Michaelis & Menten (1913), Biochemistry textbook standard
// ============================================================================

#[test]
fn michaelis_menten_reference_standard() {
    // Formula: v = Vmax * [S] / (Km + [S])
    // Source: Standard Michaelis-Menten kinetics
    // Tolerance: 0.01 absolute
    //
    // Vmax = 100, [S] = 50, Km = 25
    // v = 100 * 50 / (25 + 50) = 5000 / 75 = 66.667
    let v = chemistry::michaelis_menten_rate(50.0, 100.0, 25.0).expect("valid MM parameters");

    assert_absolute(v, 66.667, 0.01, "MM: Vmax=100, [S]=50, Km=25");
}

#[test]
fn michaelis_menten_at_km() {
    // Formula: v = Vmax * [S] / (Km + [S])
    // Source: Definition of Km: when [S] = Km, v = Vmax/2
    // Tolerance: 0.001 absolute
    //
    // At [S] = Km: v = Vmax * Km / (Km + Km) = Vmax / 2
    let vmax = 200.0;
    let km = 50.0;
    let v = chemistry::michaelis_menten_rate(km, vmax, km).expect("valid MM parameters");

    assert_absolute(
        v,
        vmax / 2.0,
        0.001,
        "MM at half-max: v should equal Vmax/2",
    );
}

#[test]
fn michaelis_menten_approaching_saturation() {
    // Formula: v = Vmax * [S] / (Km + [S])
    // Source: At [S] >> Km, v approaches Vmax asymptotically
    // Tolerance: 1% relative
    //
    // [S] = 1000 * Km: v = Vmax * 1000Km / (Km + 1000Km) = Vmax * 1000/1001
    let vmax = 100.0;
    let km = 10.0;
    let substrate = 10_000.0; // 1000 * Km
    let v = chemistry::michaelis_menten_rate(substrate, vmax, km).expect("valid MM parameters");

    let expected = vmax * substrate / (km + substrate);
    assert_relative(v, expected, 0.01, "MM near saturation");
    // Should be very close to Vmax
    assert!(
        (v - vmax).abs() < 0.2,
        "At [S]=1000*Km, v should be within 0.1% of Vmax"
    );
}

#[test]
fn michaelis_menten_lineweaver_burk_consistency() {
    // Formula: 1/v = (Km/Vmax)(1/[S]) + 1/Vmax  (Lineweaver-Burk)
    // Source: Lineweaver & Burk (1934), double-reciprocal plot
    // Tolerance: 0.1% relative
    //
    // Verify MM output is consistent with Lineweaver-Burk transform
    let vmax = 150.0;
    let km = 30.0;
    let substrates = [5.0, 10.0, 20.0, 50.0, 100.0];

    for &s in &substrates {
        let v = chemistry::michaelis_menten_rate(s, vmax, km).expect("valid MM parameters");
        // Lineweaver-Burk: 1/v = (Km/(Vmax*[S])) + 1/Vmax
        let inv_v_expected = (km / (vmax * s)) + (1.0 / vmax);
        let inv_v_actual = 1.0 / v;
        assert_relative(
            inv_v_actual,
            inv_v_expected,
            0.001,
            &format!("Lineweaver-Burk at [S]={s}"),
        );
    }
}

// ============================================================================
// 3. Nernst Equation: E = E0 - (RT/nF) * ln(Q)
//    Source: Nernst (1889), F = 96485 C/mol, R = 8.314 J/(mol*K)
// ============================================================================

#[test]
fn nernst_reference_standard_hydrogen() {
    // Formula: E = E0 - (RT/nF) * ln(Q)
    // Source: Standard electrochemistry, F = 96485 C/mol
    // Tolerance: 0.001 V absolute
    //
    // E0 = 0.80 V, T = 298.15 K, n = 1, Q = 10
    // RT/nF = (8.314 * 298.15) / (1 * 96485) = 2478.82 / 96485 = 0.025693 V
    // E = 0.80 - 0.025693 * ln(10) = 0.80 - 0.025693 * 2.30259 = 0.80 - 0.05916
    // E = 0.7408 V
    let e = chemistry::nernst_potential(0.80, 298.15, 1.0, 10.0);

    let rt_nf = (8.314 * 298.15) / (1.0 * 96_485.0);
    let expected = 0.80 - rt_nf * 10.0_f64.ln();
    assert_absolute(e, expected, 0.001, "Nernst: E0=0.80V, n=1, Q=10");
}

#[test]
fn nernst_at_equilibrium() {
    // Formula: E = E0 - (RT/nF) * ln(Q)
    // Source: At Q=1, ln(1)=0, so E = E0
    // Tolerance: 0.0001 V absolute
    //
    // At equilibrium (Q=1), cell potential equals standard potential
    let e0 = 1.10;
    let e = chemistry::nernst_potential(e0, 298.15, 2.0, 1.0);

    assert_absolute(e, e0, 0.0001, "Nernst at equilibrium: E should equal E0");
}

#[test]
fn nernst_59mv_per_decade() {
    // Formula: E = E0 - (RT/nF) * ln(Q)
    // Source: At 298.15K with n=1, potential shifts ~59.16 mV per decade of Q
    // Tolerance: 0.5 mV absolute
    //
    // Shift from Q=1 to Q=10 with n=1:
    // delta_E = -(RT/F) * ln(10) = -0.025693 * 2.30259 = -0.05916 V = -59.16 mV
    let e_q1 = chemistry::nernst_potential(0.0, 298.15, 1.0, 1.0);
    let e_q10 = chemistry::nernst_potential(0.0, 298.15, 1.0, 10.0);

    let shift_mv = (e_q1 - e_q10) * 1000.0;
    assert_absolute(shift_mv, 59.16, 0.5, "Nernst 59 mV per decade rule");
}

#[test]
fn nernst_two_electron_transfer() {
    // Formula: E = E0 - (RT/nF) * ln(Q)
    // Source: For n=2, shift is half: ~29.58 mV per decade
    // Tolerance: 0.001 V absolute
    //
    // E0=1.10V, T=298.15K, n=2, Q=0.01
    // RT/nF = 2478.82 / (2*96485) = 0.012847 V
    // E = 1.10 - 0.012847 * ln(0.01) = 1.10 - 0.012847 * (-4.60517) = 1.10 + 0.05916
    // E = 1.1592 V
    let e = chemistry::nernst_potential(1.10, 298.15, 2.0, 0.01);

    let rt_nf = (8.314 * 298.15) / (2.0 * 96_485.0);
    let expected = 1.10 - rt_nf * 0.01_f64.ln();
    assert_absolute(e, expected, 0.001, "Nernst: n=2, Q=0.01");
}

// ============================================================================
// 4. Hill Equation: theta = [L]^n / (K^n + [L]^n)
//    Source: Hill (1910), cooperativity in ligand binding
// ============================================================================

#[test]
fn hill_at_half_saturation() {
    // Formula: theta = [L]^n / (K^n + [L]^n)
    // Source: Definition of K_half: theta = 0.5 when [L] = K, for any n
    // Tolerance: 0.001 absolute
    //
    // At [L] = K_half, theta = K^n / (K^n + K^n) = 0.5, regardless of n
    for n in [0.5, 1.0, 2.0, 3.0, 4.0, 10.0] {
        let theta = chemistry::hill_response(10.0, 10.0, n);
        assert_absolute(
            theta,
            0.5,
            0.001,
            &format!("Hill at K_half: n={n}, theta must be 0.5"),
        );
    }
}

#[test]
fn hill_positive_cooperativity() {
    // Formula: theta = [L]^n / (K^n + [L]^n)
    // Source: Hill equation with n=2 (positive cooperativity)
    // Tolerance: 0.001 absolute
    //
    // [L] = 2*K, n = 2:
    // theta = (2K)^2 / (K^2 + (2K)^2) = 4K^2 / (K^2 + 4K^2) = 4/5 = 0.8
    let k = 10.0;
    let theta = chemistry::hill_response(2.0 * k, k, 2.0);
    assert_absolute(theta, 0.8, 0.001, "Hill: [L]=2K, n=2 -> theta=0.8");
}

#[test]
fn hill_ultrasensitive() {
    // Formula: theta = [L]^n / (K^n + [L]^n)
    // Source: Hill equation with n=4 (ultrasensitive)
    // Tolerance: 0.001 absolute
    //
    // [L] = 0.5*K, n = 4:
    // theta = (0.5K)^4 / (K^4 + (0.5K)^4) = 0.0625K^4 / (K^4 + 0.0625K^4)
    //       = 0.0625 / 1.0625 = 0.05882
    let k = 10.0;
    let theta = chemistry::hill_response(0.5 * k, k, 4.0);
    let expected = 0.0625 / 1.0625;
    assert_absolute(
        theta,
        expected,
        0.001,
        "Hill: [L]=0.5K, n=4 -> theta~0.0588",
    );
}

#[test]
fn hill_reduces_to_michaelis_menten() {
    // Formula: theta = [L]^1 / (K^1 + [L]^1) = [L] / (K + [L])
    // Source: When n=1, Hill equation is identical to Michaelis-Menten
    // Tolerance: 0.001 absolute
    //
    // For n=1, Hill should match MM (normalized form)
    let k = 25.0;
    let substrates = [5.0, 10.0, 25.0, 50.0, 100.0];

    for &s in &substrates {
        let hill_val = chemistry::hill_response(s, k, 1.0);
        let mm_val = chemistry::michaelis_menten_rate(s, 1.0, k).expect("valid MM parameters");
        assert_absolute(
            hill_val,
            mm_val,
            0.001,
            &format!("Hill(n=1) should equal MM(Vmax=1) at [S]={s}"),
        );
    }
}

#[test]
fn hill_switch_like_at_high_n() {
    // Formula: theta = [L]^n / (K^n + [L]^n)
    // Source: As n -> infinity, Hill becomes a step function at K
    // Tolerance: verified by inequality
    //
    // For very high n, response below K is ~0 and above K is ~1
    let k = 10.0;
    let n = 20.0;
    let below = chemistry::hill_response(9.0, k, n);
    let above = chemistry::hill_response(11.0, k, n);

    assert!(
        below < 0.15,
        "Hill(n=20) well below K should be near 0, got {below}"
    );
    assert!(
        above > 0.85,
        "Hill(n=20) well above K should be near 1, got {above}"
    );
}

// ============================================================================
// 5. Beer-Lambert Law: A = epsilon * l * c
//    Source: Beer (1852), Lambert (1760)
// ============================================================================

#[test]
fn beer_lambert_reference_standard() {
    // Formula: A = epsilon * l * c
    // Source: Beer-Lambert law (linear absorbance)
    // Tolerance: 0.001 absolute
    //
    // epsilon = 100 L/(mol*cm), l = 1.0 cm, c = 0.01 mol/L
    // A = 100 * 1.0 * 0.01 = 1.0
    let a = chemistry::beer_lambert_absorbance(100.0, 1.0, 0.01)
        .expect("valid Beer-Lambert parameters");

    assert_absolute(a, 1.0, 0.001, "Beer-Lambert: eps=100, l=1, c=0.01");
}

#[test]
fn beer_lambert_linearity() {
    // Formula: A = epsilon * l * c (linear in concentration)
    // Source: Beer-Lambert linearity principle
    // Tolerance: 0.001 absolute
    //
    // Doubling concentration should double absorbance
    let eps = 500.0;
    let l = 2.0;

    let a1 = chemistry::beer_lambert_absorbance(eps, l, 0.001).expect("valid parameters");
    let a2 = chemistry::beer_lambert_absorbance(eps, l, 0.002).expect("valid parameters");

    assert_absolute(
        a2 / a1,
        2.0,
        0.001,
        "Beer-Lambert linearity: 2x conc = 2x absorbance",
    );
}

#[test]
fn beer_lambert_path_length_dependence() {
    // Formula: A = epsilon * l * c (linear in path length)
    // Source: Beer-Lambert law
    // Tolerance: 0.001 absolute
    //
    // Doubling path length should double absorbance
    let eps = 200.0;
    let c = 0.005;

    let a1 = chemistry::beer_lambert_absorbance(eps, 1.0, c).expect("valid parameters");
    let a2 = chemistry::beer_lambert_absorbance(eps, 2.0, c).expect("valid parameters");

    assert_absolute(a2 / a1, 2.0, 0.001, "Beer-Lambert: 2x path = 2x absorbance");
}

#[test]
fn beer_lambert_transmittance_relationship() {
    // Formula: T = 10^(-A), A = -log10(T)
    // Source: Relationship between absorbance and transmittance
    // Tolerance: 0.001 absolute
    //
    // A=1 -> T=0.1, A=2 -> T=0.01, A=0 -> T=1.0
    let a = chemistry::beer_lambert_absorbance(100.0, 1.0, 0.01).expect("valid parameters");
    let t = chemistry::signal_intensity::transmittance(a);

    // A=1.0 -> T = 10^(-1) = 0.1
    assert_absolute(t, 0.1, 0.001, "Transmittance at A=1 should be 0.1");
}

// ============================================================================
// 6. Half-Life Decay: N(t) = N0 * exp(-lambda*t), lambda = ln(2)/t_half
//    Source: Rutherford & Soddy (1902), first-order kinetics
// ============================================================================

#[test]
fn decay_one_half_life() {
    // Formula: N(t) = N0 * exp(-ln(2)*t/t_half)
    // Source: Definition of half-life: 50% remaining after 1 half-life
    // Tolerance: 0.01 absolute
    //
    // N0 = 100, t_half = 10, t = 10 (1 half-life)
    // N = 100 * exp(-ln(2)) = 100 * 0.5 = 50
    let remaining =
        chemistry::remaining_after_time(100.0, 10.0, 10.0).expect("valid decay parameters");

    assert_absolute(remaining, 50.0, 0.01, "Decay: 1 half-life -> 50% remaining");
}

#[test]
fn decay_two_half_lives() {
    // Formula: N(t) = N0 * exp(-ln(2)*t/t_half)
    // Source: After 2 half-lives: 25% remaining
    // Tolerance: 0.01 absolute
    //
    // N0 = 100, t_half = 10, t = 20 (2 half-lives)
    // N = 100 * exp(-2*ln(2)) = 100 * 0.25 = 25
    let remaining =
        chemistry::remaining_after_time(100.0, 10.0, 20.0).expect("valid decay parameters");

    assert_absolute(
        remaining,
        25.0,
        0.01,
        "Decay: 2 half-lives -> 25% remaining",
    );
}

#[test]
fn decay_three_half_lives() {
    // Formula: N(t) = N0 * exp(-ln(2)*t/t_half)
    // Source: After 3 half-lives: 12.5% remaining
    // Tolerance: 0.1 absolute
    //
    // N0 = 1000, t_half = 5, t = 15 (3 half-lives)
    // N = 1000 * exp(-3*ln(2)) = 1000 * 0.125 = 125
    let remaining =
        chemistry::remaining_after_time(1000.0, 5.0, 15.0).expect("valid decay parameters");

    assert_absolute(
        remaining,
        125.0,
        0.1,
        "Decay: 3 half-lives -> 12.5% remaining",
    );
}

#[test]
fn decay_at_time_zero() {
    // Formula: N(0) = N0 * exp(0) = N0
    // Source: Initial condition
    // Tolerance: 0.0001 absolute
    //
    // At t=0, no decay has occurred
    let remaining =
        chemistry::remaining_after_time(100.0, 10.0, 0.0).expect("valid decay parameters");

    assert_absolute(remaining, 100.0, 0.0001, "Decay: t=0 -> 100% remaining");
}

#[test]
fn decay_constant_half_life_relationship() {
    // Formula: t_half = ln(2) / k, k = ln(2) / t_half
    // Source: First-order kinetics relationship
    // Tolerance: 0.001 relative
    //
    // Roundtrip: half_life -> decay_constant -> half_life
    let original_half_life = 42.0;
    let k = chemistry::decay_constant_from_half_life(original_half_life).expect("valid half-life");
    let recovered = chemistry::half_life_from_decay_constant(k).expect("valid k");

    assert_relative(
        recovered,
        original_half_life,
        0.001,
        "Decay constant roundtrip",
    );

    // Verify k * t_half = ln(2)
    let product = k * original_half_life;
    assert_absolute(
        product,
        std::f64::consts::LN_2,
        0.0001,
        "k * t_half = ln(2)",
    );
}

#[test]
fn decay_carbon14_reference() {
    // Formula: N(t) = N0 * exp(-ln(2)*t/t_half)
    // Source: Carbon-14 half-life = 5730 years (standard reference value)
    // Tolerance: 1% relative
    //
    // After 5730 years, exactly 50% should remain
    // After 11460 years (2 half-lives), 25% remains
    let n0 = 1.0e6; // Initial C-14 atoms
    let t_half = 5730.0; // years

    let after_1 = chemistry::remaining_after_time(n0, t_half, 5730.0).expect("valid parameters");
    let after_2 = chemistry::remaining_after_time(n0, t_half, 11460.0).expect("valid parameters");

    assert_relative(after_1, 500_000.0, 0.01, "C-14: 1 half-life");
    assert_relative(after_2, 250_000.0, 0.01, "C-14: 2 half-lives");
}

// ============================================================================
// 7. Gibbs Free Energy: delta_G = delta_H - T * delta_S
//    Source: Gibbs (1876), thermodynamic spontaneity
// ============================================================================

#[test]
fn gibbs_reference_exothermic_entropy_increase() {
    // Formula: delta_G = delta_H - T * delta_S
    // Source: Gibbs free energy (standard thermodynamics)
    // Tolerance: 0.01 kJ/mol absolute
    //
    // NOTE: In this implementation, delta_s is in J/(mol*K) and is internally
    // divided by 1000 to convert to kJ/(mol*K) for unit consistency with delta_h.
    //
    // delta_H = -100 kJ/mol, delta_S = 200 J/(mol*K), T = 298.15 K
    // delta_G = -100 - 298.15 * (200/1000) = -100 - 59.63 = -159.63 kJ/mol
    let dg = chemistry::gibbs_free_energy(-100.0, 200.0, 298.15).expect("valid Gibbs parameters");

    let expected = -100.0 - 298.15 * (200.0 / 1000.0);
    assert_absolute(dg, expected, 0.01, "Gibbs: exothermic + entropy increase");
    assert!(dg < 0.0, "Should be spontaneous (always favorable)");
}

#[test]
fn gibbs_reference_endothermic_entropy_decrease() {
    // Formula: delta_G = delta_H - T * delta_S
    // Source: Standard thermodynamics (never favorable case)
    // Tolerance: 0.01 kJ/mol absolute
    //
    // delta_H = +50 kJ/mol, delta_S = -100 J/(mol*K), T = 298.15 K
    // delta_G = 50 - 298.15 * (-100/1000) = 50 + 29.815 = 79.815 kJ/mol
    let dg = chemistry::gibbs_free_energy(50.0, -100.0, 298.15).expect("valid Gibbs parameters");

    let expected = 50.0 - 298.15 * (-100.0 / 1000.0);
    assert_absolute(dg, expected, 0.01, "Gibbs: endothermic + entropy decrease");
    assert!(dg > 0.0, "Should be non-spontaneous (never favorable)");
}

#[test]
fn gibbs_temperature_crossover() {
    // Formula: delta_G = delta_H - T * delta_S
    // Source: Temperature at which reaction becomes favorable: T_cross = delta_H / delta_S
    // Tolerance: 0.1 K
    //
    // delta_H = +30 kJ/mol, delta_S = +100 J/(mol*K) = 0.1 kJ/(mol*K)
    // Crossover: delta_G = 0 when T = delta_H / (delta_S/1000) = 30 / 0.1 = 300 K
    let delta_h = 30.0;
    let delta_s = 100.0; // J/(mol*K)

    // Below crossover: unfavorable
    let dg_low = chemistry::gibbs_free_energy(delta_h, delta_s, 290.0).expect("valid parameters");
    assert!(dg_low > 0.0, "Below crossover should be non-spontaneous");

    // Above crossover: favorable
    let dg_high = chemistry::gibbs_free_energy(delta_h, delta_s, 310.0).expect("valid parameters");
    assert!(dg_high < 0.0, "Above crossover should be spontaneous");

    // At crossover: approximately zero
    let dg_cross = chemistry::gibbs_free_energy(delta_h, delta_s, 300.0).expect("valid parameters");
    assert_absolute(dg_cross, 0.0, 0.01, "At crossover T, delta_G ~ 0");
}

#[test]
fn gibbs_water_freezing_reference() {
    // Formula: delta_G = delta_H - T * delta_S
    // Source: Water freezing: delta_H = -6.01 kJ/mol, delta_S = -22.0 J/(mol*K)
    //         Standard reference value (Atkins' Physical Chemistry)
    // Tolerance: 0.1 kJ/mol
    //
    // At T = 273.15 K (0 C): delta_G = -6.01 - 273.15*(-22.0/1000) = -6.01 + 6.01 ~ 0
    // At T = 263.15 K (-10 C): delta_G < 0 (spontaneous freezing)
    // At T = 283.15 K (+10 C): delta_G > 0 (no spontaneous freezing)
    let delta_h = -6.01;
    let delta_s = -22.0; // J/(mol*K)

    let dg_at_mp = chemistry::gibbs_free_energy(delta_h, delta_s, 273.15).expect("valid");
    assert_absolute(dg_at_mp, 0.0, 0.1, "Gibbs at water melting point ~ 0");

    let dg_below = chemistry::gibbs_free_energy(delta_h, delta_s, 263.15).expect("valid");
    assert!(dg_below < 0.0, "Freezing should be spontaneous below 0C");

    let dg_above = chemistry::gibbs_free_energy(delta_h, delta_s, 283.15).expect("valid");
    assert!(
        dg_above > 0.0,
        "Freezing should not be spontaneous above 0C"
    );
}

// ============================================================================
// 8. Eyring Equation: k = kappa * (kB*T/h) * exp(-delta_G_act / RT)
//    Source: Eyring (1935), transition state theory
//    kB = 1.380649e-23 J/K, h = 6.62607015e-34 J*s
// ============================================================================

#[test]
fn eyring_zero_barrier() {
    // Formula: k = kappa * (kB*T/h) * exp(-delta_G / RT)
    // Source: Eyring equation at zero barrier (diffusion-limited)
    // Tolerance: 5% relative (thermal prefactor precision)
    //
    // At delta_G = 0, k = kB*T/h (thermal frequency)
    // At T = 298.15 K: k = 1.380649e-23 * 298.15 / 6.62607015e-34
    //                    = 4.1166e-21 / 6.6261e-34 = 6.212e12 s^-1
    let rate = chemistry::eyring_rate(0.0, 298.15, 1.0);
    let expected = 1.380649e-23 * 298.15 / 6.62607015e-34;

    assert_relative(rate, expected, 0.05, "Eyring at zero barrier: kB*T/h");
}

#[test]
fn eyring_with_activation_barrier() {
    // Formula: k = kappa * (kB*T/h) * exp(-delta_G / RT)
    // Source: Eyring equation with typical enzymatic barrier
    // Tolerance: 1% relative
    //
    // delta_G = 60000 J/mol, T = 298.15 K, kappa = 1.0
    // exponent = -60000 / (8.314 * 298.15) = -24.204
    // k = 6.212e12 * exp(-24.204) = 6.212e12 * 3.025e-11 = 187.9 s^-1
    let rate = chemistry::eyring_rate(60_000.0, 298.15, 1.0);

    let kb_t_h = 1.380649e-23 * 298.15 / 6.62607015e-34;
    let expected = kb_t_h * (-60_000.0_f64 / (8.314 * 298.15)).exp();
    assert_relative(rate, expected, 0.01, "Eyring: delta_G=60 kJ/mol at 298K");
}

#[test]
fn eyring_from_enthalpy_entropy() {
    // Formula: k = (kB*T/h) * exp(-(delta_H - T*delta_S) / RT)
    // Source: Eyring equation decomposed via delta_G = delta_H - T*delta_S
    // Tolerance: 1% relative
    //
    // delta_H = 60000 J/mol, delta_S = -10 J/(mol*K), T = 298.15 K
    // delta_G = 60000 - 298.15*(-10) = 60000 + 2981.5 = 62981.5 J/mol
    let delta_h = 60_000.0;
    let delta_s = -10.0;
    let t = 298.15;
    let delta_g = chemistry::gibbs_activation(delta_h, delta_s, t);

    assert_absolute(
        delta_g,
        62_981.5,
        1.0,
        "Gibbs activation energy from components",
    );

    let rate = chemistry::eyring_rate(delta_g, t, 1.0);
    let kb_t_h = 1.380649e-23 * t / 6.62607015e-34;
    let expected = kb_t_h * (-delta_g / (8.314 * t)).exp();
    assert_relative(rate, expected, 0.01, "Eyring from H/S components");
}

#[test]
fn eyring_transmission_coefficient() {
    // Formula: k = kappa * (kB*T/h) * exp(-delta_G / RT)
    // Source: Transmission coefficient scales rate linearly
    // Tolerance: 1% relative
    //
    // kappa=0.5 should give exactly half the rate of kappa=1.0
    let rate_full = chemistry::eyring_rate(50_000.0, 298.15, 1.0);
    let rate_half = chemistry::eyring_rate(50_000.0, 298.15, 0.5);

    assert_relative(
        rate_half / rate_full,
        0.5,
        0.01,
        "Eyring: kappa=0.5 gives half the rate",
    );
}

#[test]
fn eyring_temperature_sensitivity() {
    // Formula: k = (kB*T/h) * exp(-delta_G / RT)
    // Source: Eyring equation temperature dependence
    // Tolerance: verified by inequality
    //
    // Higher temperature increases both prefactor and exponential
    let rate_298 = chemistry::eyring_rate(70_000.0, 298.15, 1.0);
    let rate_310 = chemistry::eyring_rate(70_000.0, 310.15, 1.0);
    let rate_373 = chemistry::eyring_rate(70_000.0, 373.15, 1.0);

    assert!(
        rate_310 > rate_298,
        "Rate should increase from 298K to 310K"
    );
    assert!(
        rate_373 > rate_310,
        "Rate should increase from 310K to 373K"
    );

    // Verify relative magnitudes match expected exponential behavior
    let ratio_310_298 = rate_310 / rate_298;
    assert!(
        ratio_310_298 > 1.0 && ratio_310_298 < 100.0,
        "12K increase should give moderate rate increase, got {ratio_310_298:.2}x"
    );
}

// ============================================================================
// Cross-module consistency checks
// ============================================================================

#[test]
fn arrhenius_eyring_consistency() {
    // Formula: Arrhenius k = A*exp(-Ea/RT), Eyring k = (kB*T/h)*exp(-delta_G/RT)
    // Source: Arrhenius is approximation of Eyring when A = (kB*T/h)*exp(delta_S/R)
    // Tolerance: Qualitative consistency (same order of magnitude)
    //
    // For reactions where Ea ~ delta_H and A ~ (kB*T/h)*exp(delta_S/R),
    // the two equations should give comparable rates
    let t = 298.15;
    let ea_kj = 75.0; // kJ/mol

    // Arrhenius with typical A factor
    let k_arr = chemistry::arrhenius_rate(1e13, ea_kj, t).expect("valid Arrhenius parameters");

    // Eyring with delta_G = Ea (rough approximation when delta_S ~ 0)
    let k_eyr = chemistry::eyring_rate(ea_kj * 1000.0, t, 1.0);

    // Both should be positive and in the same general magnitude range
    assert!(k_arr > 0.0, "Arrhenius rate should be positive");
    assert!(k_eyr > 0.0, "Eyring rate should be positive");

    // Log ratio should be within a few orders of magnitude
    let log_ratio = (k_arr / k_eyr).abs().log10();
    assert!(
        log_ratio.abs() < 5.0,
        "Arrhenius and Eyring should be within 5 orders of magnitude, log ratio = {log_ratio:.1}"
    );
}

#[test]
fn saturation_fraction_matches_hill_n1() {
    // Formula: saturation_fraction = [S]/(Km+[S]) should equal Hill(n=1)
    // Source: Mathematical equivalence when n=1
    // Tolerance: 0.0001 absolute
    //
    // Both should produce identical results for n=1
    let km = 15.0;
    let concentrations = [1.0, 5.0, 15.0, 30.0, 100.0];

    for &c in &concentrations {
        let sat = chemistry::saturation_fraction(c, km).expect("valid parameters");
        let hill = chemistry::hill_response(c, km, 1.0);
        assert_absolute(
            sat,
            hill,
            0.0001,
            &format!("saturation_fraction vs hill(n=1) at c={c}"),
        );
    }
}

#[test]
fn beer_lambert_inverse_roundtrip() {
    // Formula: A = eps*l*c, c = A/(eps*l)
    // Source: Beer-Lambert and its algebraic inverse
    // Tolerance: 0.0001 relative
    //
    // Compute absorbance then infer concentration; should recover original
    let eps = 350.0;
    let l = 2.5;
    let c_original = 0.0042;

    let a = chemistry::beer_lambert_absorbance(eps, l, c_original).expect("valid parameters");
    let c_recovered = chemistry::infer_concentration(a, eps, l).expect("valid parameters");

    assert_relative(
        c_recovered,
        c_original,
        0.0001,
        "Beer-Lambert inverse roundtrip",
    );
}
