//! Anti-vector classification: maps harm types to their corresponding anti-vector strategy.
//!
//! Each of the 8 harm types (A-H) has a primary anti-vector class and specific
//! countermeasure patterns. This is the inversion of the harm taxonomy.

use crate::types::{AntiVectorClass, BiasType, RiskMinimizationMeasure};
use nexcore_harm_taxonomy::HarmTypeId;

/// Anti-vector strategy for a specific harm type.
#[derive(Debug, Clone)]
pub struct AntiVectorStrategy {
    /// Primary anti-vector class for this harm type
    pub primary_class: AntiVectorClass,
    /// Secondary anti-vector class (if applicable)
    pub secondary_class: Option<AntiVectorClass>,
    /// Recommended risk minimization measures (ordered by proportionality)
    pub measures: Vec<RiskMinimizationMeasure>,
    /// Common biases that generate false signals for this harm type
    pub common_biases: Vec<BiasType>,
    /// Strategy description
    pub description: &'static str,
}

/// Derive the anti-vector strategy for a given harm type.
///
/// This is the core classification function — the inversion table of the harm taxonomy.
/// For every harm vector class, returns the corresponding anti-vector strategy.
#[must_use]
pub fn classify_anti_vector(harm_type: HarmTypeId) -> AntiVectorStrategy {
    match harm_type {
        // Type A: Acute — immediate severe harm, high magnitude
        // Anti-vector: mechanistic (break the acute pathway) + architectural (dose controls)
        HarmTypeId::A => AntiVectorStrategy {
            primary_class: AntiVectorClass::Mechanistic,
            secondary_class: Some(AntiVectorClass::Architectural),
            measures: vec![
                RiskMinimizationMeasure::DoseModification,
                RiskMinimizationMeasure::RequiredMonitoring,
                RiskMinimizationMeasure::Contraindication,
            ],
            common_biases: vec![BiasType::NotorietyBias, BiasType::StimulatedReporting],
            description: "Acute harm: break the rapid-onset pathway via dose control or contraindication",
        },

        // Type B: Cumulative — gradual harm from repeated exposure
        // Anti-vector: architectural (monitoring to catch accumulation) + mechanistic
        HarmTypeId::B => AntiVectorStrategy {
            primary_class: AntiVectorClass::Architectural,
            secondary_class: Some(AntiVectorClass::Mechanistic),
            measures: vec![
                RiskMinimizationMeasure::RequiredMonitoring,
                RiskMinimizationMeasure::DoseModification,
                RiskMinimizationMeasure::LabelUpdate,
            ],
            common_biases: vec![BiasType::DepletionOfSusceptibles, BiasType::ChannelingBias],
            description: "Cumulative harm: monitor accumulation markers, adjust dose over time",
        },

        // Type C: Off-Target — unintended effects on non-target systems
        // Anti-vector: mechanistic (understand off-target binding) + epistemic
        HarmTypeId::C => AntiVectorStrategy {
            primary_class: AntiVectorClass::Mechanistic,
            secondary_class: Some(AntiVectorClass::Epistemic),
            measures: vec![
                RiskMinimizationMeasure::LabelUpdate,
                RiskMinimizationMeasure::MedicationGuide,
                RiskMinimizationMeasure::DhcpLetter,
            ],
            common_biases: vec![BiasType::IndicationBias, BiasType::ProtopathicBias],
            description: "Off-target harm: map the off-target mechanism, distinguish from indication",
        },

        // Type D: Cascade — propagating failure across systems
        // Anti-vector: architectural (circuit breakers) + mechanistic
        HarmTypeId::D => AntiVectorStrategy {
            primary_class: AntiVectorClass::Architectural,
            secondary_class: Some(AntiVectorClass::Mechanistic),
            measures: vec![
                RiskMinimizationMeasure::Rems,
                RiskMinimizationMeasure::RequiredMonitoring,
                RiskMinimizationMeasure::RestrictedDistribution,
            ],
            common_biases: vec![BiasType::StimulatedReporting, BiasType::DuplicateReporting],
            description: "Cascade harm: install circuit breakers at propagation boundaries",
        },

        // Type E: Idiosyncratic — rare harm from unusual susceptibility
        // Anti-vector: epistemic (identify susceptible population) + architectural
        HarmTypeId::E => AntiVectorStrategy {
            primary_class: AntiVectorClass::Epistemic,
            secondary_class: Some(AntiVectorClass::Architectural),
            measures: vec![
                RiskMinimizationMeasure::RequiredMonitoring,
                RiskMinimizationMeasure::Contraindication,
                RiskMinimizationMeasure::MedicationGuide,
            ],
            common_biases: vec![BiasType::ChannelingBias, BiasType::NotorietyBias],
            description: "Idiosyncratic harm: identify the susceptible θ-subspace, screen before exposure",
        },

        // Type F: Saturation — harm from exceeding processing capacity
        // Anti-vector: mechanistic (dose-response curve) + architectural (dose caps)
        HarmTypeId::F => AntiVectorStrategy {
            primary_class: AntiVectorClass::Mechanistic,
            secondary_class: Some(AntiVectorClass::Architectural),
            measures: vec![
                RiskMinimizationMeasure::DoseModification,
                RiskMinimizationMeasure::RequiredMonitoring,
                RiskMinimizationMeasure::LabelUpdate,
            ],
            common_biases: vec![BiasType::WeberEffect, BiasType::StimulatedReporting],
            description: "Saturation harm: find the capacity threshold, enforce dose ceiling",
        },

        // Type G: Interaction — harm from combining multiple perturbations
        // Anti-vector: epistemic (map interaction space) + architectural
        HarmTypeId::G => AntiVectorStrategy {
            primary_class: AntiVectorClass::Epistemic,
            secondary_class: Some(AntiVectorClass::Architectural),
            measures: vec![
                RiskMinimizationMeasure::Contraindication,
                RiskMinimizationMeasure::LabelUpdate,
                RiskMinimizationMeasure::DhcpLetter,
            ],
            common_biases: vec![BiasType::ChannelingBias, BiasType::IndicationBias],
            description: "Interaction harm: map the combination space, contraindicate dangerous pairs",
        },

        // Type H: Population — differential harm across subgroups
        // Anti-vector: epistemic (identify vulnerable subgroup) + architectural
        HarmTypeId::H => AntiVectorStrategy {
            primary_class: AntiVectorClass::Epistemic,
            secondary_class: Some(AntiVectorClass::Architectural),
            measures: vec![
                RiskMinimizationMeasure::LabelUpdate,
                RiskMinimizationMeasure::MedicationGuide,
                RiskMinimizationMeasure::Rems,
            ],
            common_biases: vec![BiasType::ChannelingBias, BiasType::DepletionOfSusceptibles],
            description: "Population harm: identify the vulnerable subgroup, tailor risk communication",
        },

        // Type I: Goal Misalignment (extension) — treat as cascade
        HarmTypeId::I => AntiVectorStrategy {
            primary_class: AntiVectorClass::Architectural,
            secondary_class: Some(AntiVectorClass::Epistemic),
            measures: vec![
                RiskMinimizationMeasure::Rems,
                RiskMinimizationMeasure::RestrictedDistribution,
            ],
            common_biases: vec![],
            description: "Goal misalignment harm: architectural containment with epistemic monitoring",
        },
    }
}
