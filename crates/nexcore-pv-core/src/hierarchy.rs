//! # Safety Level Hierarchy
//!
//! 8-level safety hierarchy from molecular to regulatory scale.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Safety level in the pharmacovigilance hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SafetyLevel {
    /// Molecular interactions (ps-μs)
    Molecular = 1,
    /// Cellular effects (s-min)
    Cellular = 2,
    /// Tissue damage (min-hr)
    Tissue = 3,
    /// Organ dysfunction (hr-days)
    Organ = 4,
    /// Multi-organ system (days-wks)
    System = 5,
    /// Clinical manifestation (wks-mos)
    Clinical = 6,
    /// Population epidemiology (mos-yrs)
    Epidemiological = 7,
    /// Regulatory action (yrs-decades)
    Regulatory = 8,
}

impl SafetyLevel {
    /// Get the next level in the hierarchy.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Molecular => Some(Self::Cellular),
            Self::Cellular => Some(Self::Tissue),
            Self::Tissue => Some(Self::Organ),
            Self::Organ => Some(Self::System),
            Self::System => Some(Self::Clinical),
            Self::Clinical => Some(Self::Epidemiological),
            Self::Epidemiological => Some(Self::Regulatory),
            Self::Regulatory => None,
        }
    }
}

/// Metadata for a safety level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyLevelMetadata {
    /// Level enum value
    pub level: SafetyLevel,
    /// Display name
    pub name: String,
    /// Scope description
    pub scope: String,
    /// Minimum time scale
    pub time_scale_min: String,
    /// Maximum time scale
    pub time_scale_max: String,
    /// Minimum system units affected
    pub system_units_min: u64,
    /// Maximum system units affected
    pub system_units_max: u64,
    /// Example phenomena at this level
    pub example_phenomena: Vec<String>,
}

/// Static metadata for all safety levels.
pub static LEVEL_METADATA: Lazy<HashMap<SafetyLevel, SafetyLevelMetadata>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(
        SafetyLevel::Molecular,
        SafetyLevelMetadata {
            level: SafetyLevel::Molecular,
            name: "Molecular".to_string(),
            scope: "Single drug-target binding event; conformational change".to_string(),
            time_scale_min: "ps".to_string(),
            time_scale_max: "μs".to_string(),
            system_units_min: 1,
            system_units_max: 10,
            example_phenomena: vec!["Receptor binding".into(), "Enzyme inhibition".into()],
        },
    );

    m.insert(
        SafetyLevel::Cellular,
        SafetyLevelMetadata {
            level: SafetyLevel::Cellular,
            name: "Cellular".to_string(),
            scope: "Pathway activation; gene expression; organelle response".to_string(),
            time_scale_min: "s".to_string(),
            time_scale_max: "min".to_string(),
            system_units_min: 10,
            system_units_max: 100,
            example_phenomena: vec!["Signal transduction".into(), "Apoptosis initiation".into()],
        },
    );

    m.insert(
        SafetyLevel::Tissue,
        SafetyLevelMetadata {
            level: SafetyLevel::Tissue,
            name: "Tissue".to_string(),
            scope: "Local tissue effects; inflammation; cell death".to_string(),
            time_scale_min: "min".to_string(),
            time_scale_max: "hr".to_string(),
            system_units_min: 100,
            system_units_max: 1000,
            example_phenomena: vec!["Inflammatory response".into(), "Necrosis".into()],
        },
    );

    m.insert(
        SafetyLevel::Organ,
        SafetyLevelMetadata {
            level: SafetyLevel::Organ,
            name: "Organ".to_string(),
            scope: "Organ function impairment; hepatotoxicity; nephrotoxicity".to_string(),
            time_scale_min: "hr".to_string(),
            time_scale_max: "days".to_string(),
            system_units_min: 1000,
            system_units_max: 10000,
            example_phenomena: vec!["Hepatotoxicity".into(), "Nephrotoxicity".into()],
        },
    );

    m.insert(
        SafetyLevel::System,
        SafetyLevelMetadata {
            level: SafetyLevel::System,
            name: "System".to_string(),
            scope: "Multi-organ system effects; cardiovascular; CNS".to_string(),
            time_scale_min: "days".to_string(),
            time_scale_max: "wks".to_string(),
            system_units_min: 10000,
            system_units_max: 100000,
            example_phenomena: vec!["Multi-organ dysfunction".into()],
        },
    );

    m.insert(
        SafetyLevel::Clinical,
        SafetyLevelMetadata {
            level: SafetyLevel::Clinical,
            name: "Clinical".to_string(),
            scope: "Observable adverse event; patient symptoms".to_string(),
            time_scale_min: "wks".to_string(),
            time_scale_max: "mos".to_string(),
            system_units_min: 100000,
            system_units_max: 1000000,
            example_phenomena: vec!["Reported adverse event".into(), "Hospitalization".into()],
        },
    );

    m.insert(
        SafetyLevel::Epidemiological,
        SafetyLevelMetadata {
            level: SafetyLevel::Epidemiological,
            name: "Epidemiological".to_string(),
            scope: "Population-level incidence; risk factor identification".to_string(),
            time_scale_min: "mos".to_string(),
            time_scale_max: "yrs".to_string(),
            system_units_min: 1000000,
            system_units_max: 10000000,
            example_phenomena: vec!["Safety signal detection".into()],
        },
    );

    m.insert(
        SafetyLevel::Regulatory,
        SafetyLevelMetadata {
            level: SafetyLevel::Regulatory,
            name: "Regulatory".to_string(),
            scope: "Label changes; REMS; withdrawal; class-wide effects".to_string(),
            time_scale_min: "yrs".to_string(),
            time_scale_max: "decades".to_string(),
            system_units_min: 10000000,
            system_units_max: 100000000,
            example_phenomena: vec!["Market withdrawal".into(), "Black box warning".into()],
        },
    );

    m
});

/// Tree of Vigilance hierarchy level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ToVLevel {
    /// Molecular & Cellular
    Molecular = 1,
    /// Tissue, Organ, System
    Physiological = 2,
    /// Clinical observations
    Clinical = 3,
    /// Epidemiological/Population
    Population = 4,
    /// Regulatory actions
    Regulatory = 5,
}

/// Map a SafetyLevel to its ToV hierarchy level.
#[must_use]
pub fn map_to_tov_level(level: SafetyLevel) -> ToVLevel {
    match level {
        SafetyLevel::Molecular | SafetyLevel::Cellular => ToVLevel::Molecular,
        SafetyLevel::Tissue | SafetyLevel::Organ | SafetyLevel::System => ToVLevel::Physiological,
        SafetyLevel::Clinical => ToVLevel::Clinical,
        SafetyLevel::Epidemiological => ToVLevel::Population,
        SafetyLevel::Regulatory => ToVLevel::Regulatory,
    }
}

/// Get constituent SafetyLevels for a ToV level.
#[must_use]
pub fn get_constituent_levels(tov_level: ToVLevel) -> Vec<SafetyLevel> {
    match tov_level {
        ToVLevel::Molecular => vec![SafetyLevel::Molecular, SafetyLevel::Cellular],
        ToVLevel::Physiological => {
            vec![SafetyLevel::Tissue, SafetyLevel::Organ, SafetyLevel::System]
        }
        ToVLevel::Clinical => vec![SafetyLevel::Clinical],
        ToVLevel::Population => vec![SafetyLevel::Epidemiological],
        ToVLevel::Regulatory => vec![SafetyLevel::Regulatory],
    }
}

/// Calculate probability of signal propagating through hierarchy.
#[must_use]
pub fn calculate_propagation_probability(
    source: SafetyLevel,
    target: SafetyLevel,
    transition_probs: &HashMap<(SafetyLevel, SafetyLevel), f64>,
) -> f64 {
    if source == target {
        return 1.0;
    }
    if source > target {
        return 0.0;
    }

    let mut probability = 1.0;
    let mut current = source;

    while current < target {
        if let Some(next) = current.next() {
            match transition_probs.get(&(current, next)) {
                Some(&p) => {
                    probability *= p;
                    if probability == 0.0 {
                        return 0.0;
                    }
                }
                None => return 0.0,
            }
            current = next;
        } else {
            return 0.0;
        }
    }

    probability
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_level_ordering() {
        assert!(SafetyLevel::Molecular < SafetyLevel::Regulatory);
        assert!(SafetyLevel::Clinical > SafetyLevel::Organ);
    }

    #[test]
    fn test_tov_mapping() {
        assert_eq!(
            map_to_tov_level(SafetyLevel::Molecular),
            ToVLevel::Molecular
        );
        assert_eq!(map_to_tov_level(SafetyLevel::Cellular), ToVLevel::Molecular);
        assert_eq!(
            map_to_tov_level(SafetyLevel::Organ),
            ToVLevel::Physiological
        );
        assert_eq!(
            map_to_tov_level(SafetyLevel::Regulatory),
            ToVLevel::Regulatory
        );
    }

    #[test]
    fn test_constituent_levels() {
        let molecular = get_constituent_levels(ToVLevel::Molecular);
        assert_eq!(molecular.len(), 2);
        assert!(molecular.contains(&SafetyLevel::Molecular));
        assert!(molecular.contains(&SafetyLevel::Cellular));
    }

    #[test]
    fn test_propagation_same_level() {
        let probs = HashMap::new();
        assert_eq!(
            calculate_propagation_probability(SafetyLevel::Clinical, SafetyLevel::Clinical, &probs),
            1.0
        );
    }

    #[test]
    fn test_propagation_reverse() {
        let probs = HashMap::new();
        assert_eq!(
            calculate_propagation_probability(
                SafetyLevel::Regulatory,
                SafetyLevel::Molecular,
                &probs
            ),
            0.0
        );
    }
}
