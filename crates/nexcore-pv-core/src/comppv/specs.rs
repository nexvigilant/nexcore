//! Formal specifications for the 11 Conservation Laws.
//!
//! Each law has a formal mathematical statement, validation tolerance,
//! and documented safety implications of violation.

use crate::comppv::types::{ConservationLaw, ConservationLawSpec};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Static map of all 11 conservation law specifications.
pub static CONSERVATION_LAW_SPECS: Lazy<HashMap<ConservationLaw, ConservationLawSpec>> =
    Lazy::new(|| {
        let mut m = HashMap::new();

        m.insert(
            ConservationLaw::DrugMassBalance,
            ConservationLawSpec {
                law: ConservationLaw::DrugMassBalance,
                name: "Drug Mass Balance".to_string(),
                statement: "The total mass of drug in the body at any time equals the \
                    administered dose minus the cumulative amount eliminated."
                    .to_string(),
                mathematical_form: "D(t) = D₀ - ∫₀ᵗ CL·C(τ)dτ".to_string(),
                safety_implication: "Accumulation (violation of mass balance) leads to \
                    concentration-dependent toxicity."
                    .to_string(),
                validation_tolerance: 0.05,
            },
        );

        m.insert(
            ConservationLaw::ThermodynamicBinding,
            ConservationLawSpec {
                law: ConservationLaw::ThermodynamicBinding,
                name: "Thermodynamic Binding".to_string(),
                statement: "Drug-target binding is governed by thermodynamic equilibrium; \
                    spontaneous binding requires negative Gibbs free energy."
                    .to_string(),
                mathematical_form: "ΔG = ΔH - TΔS < 0 for spontaneous binding; Kd = exp(ΔG/RT)"
                    .to_string(),
                safety_implication: "Off-target binding occurs when ΔG is favorable for \
                    non-intended targets; selectivity = ΔΔG between targets."
                    .to_string(),
                validation_tolerance: 0.01,
            },
        );

        m.insert(
            ConservationLaw::ReceptorState,
            ConservationLawSpec {
                law: ConservationLaw::ReceptorState,
                name: "Receptor State Conservation".to_string(),
                statement: "Total receptor number is conserved across states: \
                    free, bound, and desensitized."
                    .to_string(),
                mathematical_form: "R_total = R_free + R_bound + R_desensitized = constant"
                    .to_string(),
                safety_implication: "Sustained receptor occupancy leads to desensitization \
                    (tolerance) or upregulation (dependence)."
                    .to_string(),
                validation_tolerance: 0.001,
            },
        );

        m.insert(
            ConservationLaw::PathwayFlux,
            ConservationLawSpec {
                law: ConservationLaw::PathwayFlux,
                name: "Pathway Flux Conservation".to_string(),
                statement: "Signal flux through a pathway must be conserved at steady state; \
                    perturbation at one node redistributes flux to parallel pathways."
                    .to_string(),
                mathematical_form: "Σ J_in = Σ J_out at each node; total system flux = constant"
                    .to_string(),
                safety_implication: "Blocking one pathway may cause compensatory toxicity \
                    through parallel pathway activation."
                    .to_string(),
                validation_tolerance: 0.02,
            },
        );

        m.insert(
            ConservationLaw::EnzymeRegeneration,
            ConservationLawSpec {
                law: ConservationLaw::EnzymeRegeneration,
                name: "Enzyme Regeneration".to_string(),
                statement: "Enzymes are regenerated after catalysis; enzyme mass is conserved \
                    unless inactivated by mechanism-based inhibition."
                    .to_string(),
                mathematical_form:
                    "E_total = E_free + ES + EI; dE_total/dt = k_syn - k_deg - k_inact[I]"
                        .to_string(),
                safety_implication: "Mechanism-based inhibitors (reactive metabolites) \
                    permanently inactivate enzymes, causing prolonged toxicity."
                    .to_string(),
                validation_tolerance: 0.01,
            },
        );

        m.insert(
            ConservationLaw::AdmeRate,
            ConservationLawSpec {
                law: ConservationLaw::AdmeRate,
                name: "ADME Rate Conservation".to_string(),
                statement: "The rate of change of drug amount in any compartment equals \
                    the sum of rates into minus rates out of that compartment."
                    .to_string(),
                mathematical_form: "dA/dt = Rate_in - Rate_out; applies to each ADME compartment"
                    .to_string(),
                safety_implication: "DDIs that alter rates cause unexpected accumulation \
                    or sub-therapeutic exposure."
                    .to_string(),
                validation_tolerance: 0.05,
            },
        );

        m.insert(
            ConservationLaw::SteadyState,
            ConservationLawSpec {
                law: ConservationLaw::SteadyState,
                name: "Steady-State Equilibrium".to_string(),
                statement: "At steady state, drug input rate equals elimination rate; \
                    concentrations become constant."
                    .to_string(),
                mathematical_form: "C_ss = (F·Dose)/(CL·τ); time to steady state ≈ 4-5 half-lives"
                    .to_string(),
                safety_implication: "Delayed toxicity may manifest only after steady state \
                    is reached; loading doses may cause acute toxicity."
                    .to_string(),
                validation_tolerance: 0.1,
            },
        );

        m.insert(
            ConservationLaw::IonizationState,
            ConservationLawSpec {
                law: ConservationLaw::IonizationState,
                name: "Ionization State Conservation".to_string(),
                statement: "The ratio of ionized to un-ionized drug is determined by pH and pKa; \
                    only un-ionized drug crosses membranes passively."
                    .to_string(),
                mathematical_form: "log([A⁻]/[HA]) = pH - pKa (Henderson-Hasselbalch)".to_string(),
                safety_implication: "pH changes (disease, DDI) alter drug distribution; \
                    ion trapping in compartments."
                    .to_string(),
                validation_tolerance: 0.01,
            },
        );

        m.insert(
            ConservationLaw::SaturationKinetics,
            ConservationLawSpec {
                law: ConservationLaw::SaturationKinetics,
                name: "Saturation Kinetics".to_string(),
                statement: "Biological processes with finite capacity exhibit saturable kinetics; \
                    exceeding capacity causes non-linear behavior."
                    .to_string(),
                mathematical_form: "v = V_max·[S]/(K_m + [S]); E/E_max = [D]/(EC₅₀ + [D])"
                    .to_string(),
                safety_implication: "Saturation of elimination causes disproportionate \
                    concentration increase; saturation of protective mechanisms causes toxicity."
                    .to_string(),
                validation_tolerance: 0.02,
            },
        );

        m.insert(
            ConservationLaw::EntropyIncrease,
            ConservationLawSpec {
                law: ConservationLaw::EntropyIncrease,
                name: "Entropy Increase".to_string(),
                statement: "Drug-induced toxicity increases system entropy (disorder); \
                    biological order requires continuous energy input."
                    .to_string(),
                mathematical_form:
                    "ΔS_system > 0 in toxicity; ΔS_total = ΔS_system + ΔS_surroundings ≥ 0"
                        .to_string(),
                safety_implication: "Toxicity is thermodynamically favored without active \
                    maintenance; recovery requires energy expenditure."
                    .to_string(),
                validation_tolerance: 0.0,
            },
        );

        m.insert(
            ConservationLaw::GeneticConservation,
            ConservationLawSpec {
                law: ConservationLaw::GeneticConservation,
                name: "Genetic Conservation".to_string(),
                statement: "Genetic information is conserved unless mutated; \
                    genotoxicity represents a violation of genetic conservation."
                    .to_string(),
                mathematical_form: "DNA sequence before = DNA sequence after (normal); \
                    mutations = violation"
                    .to_string(),
                safety_implication: "Genotoxicity and carcinogenicity are extreme violations \
                    with irreversible consequences."
                    .to_string(),
                validation_tolerance: 0.0,
            },
        );

        m
    });

/// Get the specification for a specific conservation law.
pub fn get_spec(law: ConservationLaw) -> Option<&'static ConservationLawSpec> {
    CONSERVATION_LAW_SPECS.get(&law)
}

/// Get the default validation tolerance for a law.
pub fn get_tolerance(law: ConservationLaw) -> f64 {
    CONSERVATION_LAW_SPECS
        .get(&law)
        .map(|spec| spec.validation_tolerance)
        .unwrap_or(0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_laws_have_specs() {
        for law in ConservationLaw::all() {
            assert!(
                CONSERVATION_LAW_SPECS.contains_key(law),
                "Missing spec for {:?}",
                law
            );
        }
    }

    #[test]
    fn test_spec_fields_not_empty() {
        for spec in CONSERVATION_LAW_SPECS.values() {
            assert!(!spec.name.is_empty());
            assert!(!spec.statement.is_empty());
            assert!(!spec.mathematical_form.is_empty());
            assert!(!spec.safety_implication.is_empty());
        }
    }

    #[test]
    fn test_tolerances_reasonable() {
        for spec in CONSERVATION_LAW_SPECS.values() {
            assert!(
                spec.validation_tolerance >= 0.0 && spec.validation_tolerance <= 1.0,
                "Tolerance for {:?} out of range: {}",
                spec.law,
                spec.validation_tolerance
            );
        }
    }
}
