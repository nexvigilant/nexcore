//! STEM chemistry bridge: mathematical metrics → chemistry analogs.
//!
//! ## Mappings
//!
//! | Math Metric | Chemistry | Equation | Confidence |
//! |-------------|-----------|----------|------------|
//! | Shannon Entropy | Arrhenius | k = A*e^(-Ea/RT) | 0.92 |
//! | Graph Density | Michaelis-Menten | v = Vmax*S/(Km+S) | 0.88 |
//! | Test Density | Hill | f = x^n/(K^n+x^n) | 0.85 |
//! | Coupling Ratio | Henderson-Hasselbalch | pH = pKa+log(A/HA) | 0.78 |
//! | Redundancy | Half-life Decay | N = N0*e^(-λt) | 0.90 |
//! | Betweenness | Langmuir | θ = KP/(1+KP) | 0.88 |

use crate::types::*;

/// Map Shannon entropy to Arrhenius rate constant.
///
/// Higher entropy = higher activation energy = harder to comprehend.
/// k = A * e^(-Ea/RT) where Ea ∝ entropy.
pub fn entropy_to_arrhenius(entropy: Entropy) -> ChemistryMapping {
    let ea = entropy.value(); // treat entropy as activation energy
    let r = 1.0; // universal gas constant analog
    let temp = 1.0; // temperature analog
    let a = 1.0; // pre-exponential factor
    let k = a * (-ea / (r * temp)).exp();

    ChemistryMapping {
        source_metric: "shannon_entropy".into(),
        chemistry_analog: "arrhenius_rate".into(),
        mapped_value: k,
        confidence: 0.92,
        reasoning: "Entropy as activation barrier: higher entropy → slower comprehension rate"
            .into(),
    }
}

/// Map graph density to Michaelis-Menten saturation.
///
/// Density saturates with diminishing returns (like enzyme kinetics).
/// v = Vmax * S / (Km + S)
pub fn density_to_michaelis_menten(density: Density) -> ChemistryMapping {
    let s = density.value();
    let vmax = 1.0;
    let km = 0.2; // half-saturation at density=0.2
    let v = vmax * s / (km + s);

    ChemistryMapping {
        source_metric: "graph_density".into(),
        chemistry_analog: "michaelis_menten_rate".into(),
        mapped_value: v,
        confidence: 0.88,
        reasoning:
            "Interconnectedness saturates: beyond threshold, more edges add diminishing value"
                .into(),
    }
}

/// Map test density to Hill cooperative response.
///
/// Tests cooperate synergistically (n > 1 = positive cooperativity).
/// f = x^n / (K^n + x^n)
pub fn test_density_to_hill(td: TestDensity) -> ChemistryMapping {
    let x: f64 = td.value();
    let k: f64 = 10.0; // half-max at 10 tests/KLOC
    let n: f64 = 2.0; // cooperativity coefficient
    let f = x.powf(n) / (k.powf(n) + x.powf(n));

    ChemistryMapping {
        source_metric: "test_density".into(),
        chemistry_analog: "hill_response".into(),
        mapped_value: f,
        confidence: 0.85,
        reasoning: "Tests cooperate: each additional test amplifies coverage more than linearly"
            .into(),
    }
}

/// Map coupling ratio to Henderson-Hasselbalch equilibrium.
///
/// Balance between fan-in (acid) and fan-out (base).
/// pH = pKa + log([A-]/[HA])
pub fn coupling_to_henderson_hasselbalch(cr: CouplingRatio) -> ChemistryMapping {
    let ratio = cr.value();
    // Avoid log(0): treat as pH deviation from neutral
    let epsilon = 1e-10;
    let acid = (1.0 - ratio).max(epsilon); // fan-in fraction
    let base = ratio.max(epsilon); // fan-out fraction
    let pka = 0.0; // neutral pKa analog
    let ph = pka + (base / acid).log10();

    ChemistryMapping {
        source_metric: "coupling_ratio".into(),
        chemistry_analog: "henderson_hasselbalch_ph".into(),
        mapped_value: ph,
        confidence: 0.78,
        reasoning:
            "Dependency balance as acid-base equilibrium: pH=0 → balanced; extreme → unstable"
                .into(),
    }
}

/// Map redundancy to half-life decay.
///
/// Stale duplication decays informativeness.
/// N = N0 * e^(-λ * redundancy)
pub fn redundancy_to_decay(redundancy: Probability) -> ChemistryMapping {
    let r = redundancy.value();
    let n0 = 1.0; // initial information value
    let lambda = 2.0; // decay constant (higher = faster decay)
    let n = n0 * (-lambda * r).exp();

    ChemistryMapping {
        source_metric: "redundancy".into(),
        chemistry_analog: "half_life_decay".into(),
        mapped_value: n,
        confidence: 0.90,
        reasoning: "Redundancy decays information value: more duplication → less useful signal"
            .into(),
    }
}

/// Map betweenness centrality to Langmuir adsorption.
///
/// Central crates "adsorb" dependency paths; saturation = bottleneck.
/// θ = K*P / (1 + K*P)
pub fn betweenness_to_langmuir(bc: Centrality) -> ChemistryMapping {
    let p = bc.value();
    let k = 5.0; // adsorption constant
    let theta = k * p / (1.0 + k * p);

    ChemistryMapping {
        source_metric: "betweenness_centrality".into(),
        chemistry_analog: "langmuir_coverage".into(),
        mapped_value: theta,
        confidence: 0.88,
        reasoning: "Central crates adsorb paths: high betweenness → bottleneck saturation".into(),
    }
}

/// Compute all 6 chemistry mappings from measurement data.
pub fn all_chemistry_mappings(
    entropy: Entropy,
    density: Density,
    test_density: TestDensity,
    coupling: CouplingRatio,
    redundancy: Probability,
    betweenness: Centrality,
) -> Vec<ChemistryMapping> {
    vec![
        entropy_to_arrhenius(entropy),
        density_to_michaelis_menten(density),
        test_density_to_hill(test_density),
        coupling_to_henderson_hasselbalch(coupling),
        redundancy_to_decay(redundancy),
        betweenness_to_langmuir(betweenness),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrhenius_monotone_decreasing() {
        let k_low = entropy_to_arrhenius(Entropy::new(1.0)).mapped_value;
        let k_high = entropy_to_arrhenius(Entropy::new(5.0)).mapped_value;
        assert!(
            k_low > k_high,
            "higher entropy → lower rate: {} > {}",
            k_low,
            k_high
        );
    }

    #[test]
    fn arrhenius_zero_entropy() {
        let m = entropy_to_arrhenius(Entropy::new(0.0));
        assert!((m.mapped_value - 1.0).abs() < 1e-10, "zero entropy → k=1.0");
    }

    #[test]
    fn michaelis_menten_monotone_increasing() {
        let v_low = density_to_michaelis_menten(Density::new(0.1)).mapped_value;
        let v_high = density_to_michaelis_menten(Density::new(0.9)).mapped_value;
        assert!(v_high > v_low);
    }

    #[test]
    fn michaelis_menten_saturation() {
        let v_half = density_to_michaelis_menten(Density::new(0.2)).mapped_value;
        assert!((v_half - 0.5).abs() < 0.01, "Km=0.2 → half-max at 0.2");
    }

    #[test]
    fn hill_monotone_increasing() {
        let f_low = test_density_to_hill(TestDensity::new(1.0)).mapped_value;
        let f_high = test_density_to_hill(TestDensity::new(50.0)).mapped_value;
        assert!(f_high > f_low);
    }

    #[test]
    fn hill_half_max_at_k() {
        let f = test_density_to_hill(TestDensity::new(10.0)).mapped_value;
        assert!((f - 0.5).abs() < 0.01, "K=10 → half-max at 10");
    }

    #[test]
    fn hh_balanced_near_zero() {
        let m = coupling_to_henderson_hasselbalch(CouplingRatio::new(0.5));
        assert!(m.mapped_value.abs() < 0.01, "0.5 coupling → pH~0");
    }

    #[test]
    fn decay_monotone_decreasing() {
        let n_low = redundancy_to_decay(Probability::new(0.1)).mapped_value;
        let n_high = redundancy_to_decay(Probability::new(0.9)).mapped_value;
        assert!(n_low > n_high, "higher redundancy → lower value");
    }

    #[test]
    fn langmuir_monotone_increasing() {
        let t_low = betweenness_to_langmuir(Centrality::new(0.1)).mapped_value;
        let t_high = betweenness_to_langmuir(Centrality::new(0.9)).mapped_value;
        assert!(t_high > t_low);
    }

    #[test]
    fn all_mappings_returns_six() {
        let mappings = all_chemistry_mappings(
            Entropy::new(2.0),
            Density::new(0.1),
            TestDensity::new(15.0),
            CouplingRatio::new(0.3),
            Probability::new(0.2),
            Centrality::new(0.5),
        );
        assert_eq!(mappings.len(), 6);
    }

    #[test]
    fn all_confidences_valid() {
        let mappings = all_chemistry_mappings(
            Entropy::new(1.0),
            Density::new(0.5),
            TestDensity::new(10.0),
            CouplingRatio::new(0.5),
            Probability::new(0.5),
            Centrality::new(0.5),
        );
        for m in &mappings {
            assert!(m.confidence >= 0.0 && m.confidence <= 1.0);
        }
    }
}
