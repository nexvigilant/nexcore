//! Inflected Tool Families (μ-Stacking)
//!
//! Latin packs 5 grammatical dimensions into one verb form.
//! We pack multiple tool operations into one tool family,
//! inflected by a mode parameter.
//!
//! Instead of: `insight_ingest`, `insight_status`, `insight_config`, ...
//! We get:     `insight(mode: ingest|status|config, ...)`
//!
//! ## Primitive Grounding
//! μ Mapping (dominant) + σ Sequence + ∂ Boundary = T2-C

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tier: T2-C — μ Mapping + σ Sequence + ∂ Boundary
///
/// A tool family — a single stem with multiple inflected forms (modes).
/// Like a Latin verb conjugation: one stem, many endings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolFamily {
    /// The stem (family name), e.g. "insight", "caesura", "dna".
    pub stem: String,
    /// Available inflections (modes), e.g. ["scan", "metrics", "report"].
    pub inflections: Vec<Inflection>,
    /// The μ-density: how many operations packed per family.
    pub mu_density: usize,
}

/// A single inflection (mode) within a tool family.
///
/// Tier: T2-P — μ + ∂
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inflection {
    /// The mode name (suffix), e.g. "scan", "metrics", "report".
    pub mode: String,
    /// The full tool name this inflection replaces.
    pub replaces: String,
    /// Parameter names for this mode.
    pub params: Vec<String>,
    /// Whether this mode mutates state (voice: active vs passive).
    pub mutates: bool,
}

/// Tier: T2-C — μ + σ + κ
///
/// Analysis of tool families within a tool catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InflectionAnalysis {
    /// Total individual tools.
    pub total_tools: usize,
    /// Number of tool families detected.
    pub family_count: usize,
    /// Average μ-density (inflections per family).
    pub avg_mu_density: f64,
    /// Compression ratio: families / total_tools.
    pub compression_ratio: f64,
    /// Detected families.
    pub families: Vec<ToolFamily>,
    /// Singleton tools (no family detected).
    pub singletons: Vec<String>,
}

/// Extract tool families from a flat list of tool names.
///
/// Groups tools by common prefix (stem) and treats each suffix as an inflection.
/// A family must have at least 2 members.
pub fn extract_families(tool_names: &[&str]) -> InflectionAnalysis {
    let mut prefix_groups: HashMap<String, Vec<String>> = HashMap::new();

    // Group by prefix: split on `_` and try progressively longer prefixes
    for &name in tool_names {
        let parts: Vec<&str> = name.split('_').collect();
        if parts.len() >= 2 {
            let prefix = parts[0].to_string();
            prefix_groups
                .entry(prefix)
                .or_default()
                .push(name.to_string());
        }
    }

    let mut families = Vec::new();
    let mut singletons = Vec::new();
    let mut claimed: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (stem, members) in &prefix_groups {
        if members.len() >= 2 {
            let inflections: Vec<Inflection> = members
                .iter()
                .map(|name| {
                    let suffix = name
                        .strip_prefix(&format!("{stem}_"))
                        .unwrap_or(name)
                        .to_string();
                    claimed.insert(name.clone());
                    Inflection {
                        mode: suffix,
                        replaces: name.clone(),
                        params: Vec::new(),
                        mutates: false,
                    }
                })
                .collect();

            let mu_density = inflections.len();
            families.push(ToolFamily {
                stem: stem.clone(),
                inflections,
                mu_density,
            });
        }
    }

    // Find singletons
    for &name in tool_names {
        if !claimed.contains(name) {
            singletons.push(name.to_string());
        }
    }

    let total_tools = tool_names.len();
    let family_count = families.len();
    let avg_mu = if family_count > 0 {
        families.iter().map(|f| f.mu_density as f64).sum::<f64>() / family_count as f64
    } else {
        0.0
    };
    let compression = if total_tools > 0 {
        family_count as f64 / total_tools as f64
    } else {
        0.0
    };

    InflectionAnalysis {
        total_tools,
        family_count,
        avg_mu_density: avg_mu,
        compression_ratio: compression,
        families,
        singletons,
    }
}

/// Compute the μ-power of a tool family.
///
/// μ-power = number of independent dimensions packed into the family.
/// Like Latin *amāvissēmus* = μ⁵ (person × number × tense × mood × voice).
pub fn mu_power(family: &ToolFamily) -> usize {
    // Each inflection adds one μ dimension.
    // Parameterized inflections add more (each param is a sub-inflection).
    family.inflections.len()
        + family
            .inflections
            .iter()
            .map(|i| i.params.len())
            .sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_families_basic() {
        let tools = [
            "insight_ingest",
            "insight_status",
            "insight_config",
            "insight_connect",
            "dna_encode",
            "dna_decode",
            "dna_eval",
            "standalone_tool",
        ];

        let analysis = extract_families(&tools);
        assert_eq!(analysis.total_tools, 8);
        assert_eq!(analysis.family_count, 2);
        assert!(analysis.avg_mu_density > 2.0);
        assert_eq!(analysis.singletons.len(), 1);
        assert!(analysis.singletons.contains(&"standalone_tool".to_string()));
    }

    #[test]
    fn test_extract_families_no_families() {
        let tools = ["alpha", "beta", "gamma"];
        let analysis = extract_families(&tools);
        assert_eq!(analysis.family_count, 0);
        assert_eq!(analysis.singletons.len(), 3);
    }

    #[test]
    fn test_mu_power_basic() {
        let family = ToolFamily {
            stem: "insight".to_string(),
            inflections: vec![
                Inflection {
                    mode: "ingest".to_string(),
                    replaces: "insight_ingest".to_string(),
                    params: vec!["text".to_string()],
                    mutates: true,
                },
                Inflection {
                    mode: "status".to_string(),
                    replaces: "insight_status".to_string(),
                    params: vec![],
                    mutates: false,
                },
            ],
            mu_density: 2,
        };
        // 2 inflections + 1 param = 3
        assert_eq!(mu_power(&family), 3);
    }

    #[test]
    fn test_compression_ratio() {
        let tools = ["caesura_scan", "caesura_metrics", "caesura_report"];
        let analysis = extract_families(&tools);
        assert!(analysis.compression_ratio > 0.0);
        assert!(analysis.compression_ratio < 1.0);
    }

    #[test]
    fn test_family_stem() {
        let tools = ["pv_signal", "pv_landscape", "pv_metrics"];
        let analysis = extract_families(&tools);
        assert_eq!(analysis.family_count, 1);
        assert_eq!(analysis.families[0].stem, "pv");
    }
}
