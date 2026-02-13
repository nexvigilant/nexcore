//! Architecture & Design Integrity Framework (Module 3)

pub mod dependency_graph;
pub mod api_surface;
pub mod coupling;

use std::path::PathBuf;

/// A cycle detected in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyCycle {
    /// Modules in the cycle
    pub modules: Vec<String>,
}

/// A layer violation
#[derive(Debug, Clone)]
pub struct LayerViolation {
    /// Source module
    pub source: String,
    /// Source layer
    pub source_layer: String,
    /// Target module
    pub target: String,
    /// Target layer
    pub target_layer: String,
    /// Message
    pub message: String,
}

/// Layer definition
#[derive(Debug, Clone)]
pub struct LayerDefinition {
    /// Name
    pub name: String,
    /// Patterns
    pub patterns: Vec<String>,
    /// Dependencies
    pub can_depend_on: Vec<String>,
}

/// Default layers
pub fn default_layers() -> Vec<LayerDefinition> {
    vec![
        LayerDefinition {
            name: "domain".to_string(),
            patterns: vec!["domain".to_string(), "model".to_string()],
            can_depend_on: vec![],
        },
        LayerDefinition {
            name: "application".to_string(),
            patterns: vec!["service".to_string(), "usecase".to_string()],
            can_depend_on: vec!["domain".to_string()],
        },
        LayerDefinition {
            name: "infrastructure".to_string(),
            patterns: vec!["db".to_string(), "api".to_string(), "adapter".to_string()],
            can_depend_on: vec!["domain".to_string(), "application".to_string()],
        },
    ]
}

/// Identify layer
pub fn identify_layer(module: &str, layers: &[LayerDefinition]) -> Option<String> {
    let m = module.to_lowercase();
    for layer in layers {
        for p in &layer.patterns {
            if m.contains(p) {
                return Some(layer.name.clone());
            }
        }
    }
    None
}

/// Coupling zone
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouplingZone {
    /// Ideal
    MainSequence,
    /// Concrete and stable
    ZoneOfPain,
    /// Abstract and unstable
    ZoneOfUselessness,
    /// Acceptable
    Acceptable,
}

impl CouplingZone {
    /// Display name
    pub fn display(&self) -> &'static str {
        match self {
            Self::MainSequence => "Main Sequence",
            Self::ZoneOfPain => "Zone of Pain",
            Self::ZoneOfUselessness => "Zone of Uselessness",
            Self::Acceptable => "Acceptable",
        }
    }
}

/// Classify zone
pub fn classify_zone(abstractness: f64, instability: f64) -> CouplingZone {
    if abstractness < 0.3 && instability < 0.3 {
        CouplingZone::ZoneOfPain
    } else if abstractness > 0.7 && instability > 0.7 {
        CouplingZone::ZoneOfUselessness
    } else if (abstractness + instability - 1.0).abs() < 0.3 {
        CouplingZone::MainSequence
    } else {
        CouplingZone::Acceptable
    }
}

/// Architecture directory
pub fn architecture_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("verified")
        .join("architecture")
}

/// Ensure directory exists
pub fn ensure_architecture_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(architecture_dir())
}
