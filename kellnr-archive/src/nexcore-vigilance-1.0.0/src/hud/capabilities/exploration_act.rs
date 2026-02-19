//! # Capability 30: Exploration Act (Epistemic Frontiers)
//!
//! Maps to codebase exploration, knowledge gap detection, and domain discovery.
//! Enables the Explore agent pattern for systematic frontier mapping.

use nexcore_primitives::measurement::{Confidence, Measured};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// T1: ExplorationScope - Depth of exploration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExplorationScope {
    /// Quick scan (surface level).
    Quick,
    /// Medium depth.
    Medium,
    /// Very thorough (comprehensive).
    Thorough,
}

impl ExplorationScope {
    /// Get expected file count to examine.
    pub fn expected_files(&self) -> usize {
        match self {
            Self::Quick => 10,
            Self::Medium => 50,
            Self::Thorough => 200,
        }
    }
}

/// T2-P: DiscoveryIndex - Significance of a finding (0.0-1.0).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DiscoveryIndex(pub f64);

impl DiscoveryIndex {
    /// Create new index (clamped).
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Is this a significant discovery (>0.7)?
    pub fn is_significant(&self) -> bool {
        self.0 > 0.7
    }
}

/// T2-C: MissionManifest - A planned exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionManifest {
    /// Mission ID.
    pub id: String,
    /// Target domain/path.
    pub target: String,
    /// Objective description.
    pub objective: String,
    /// Exploration scope.
    pub scope: ExplorationScope,
    /// Search patterns.
    pub patterns: Vec<String>,
    /// Success probability.
    pub success_prob: f64,
}

/// T2-C: Discovery - A finding from exploration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discovery {
    /// Discovery ID.
    pub id: String,
    /// What was found.
    pub finding: String,
    /// File location (if applicable).
    pub location: Option<String>,
    /// Significance index.
    pub significance: DiscoveryIndex,
    /// Related discoveries.
    pub related: Vec<String>,
}

/// T2-C: FrontierMap - Knowledge boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontierMap {
    /// Known domains.
    pub known: Vec<String>,
    /// Unknown/unexplored areas.
    pub unknown: Vec<String>,
    /// Knowledge gaps identified.
    pub gaps: Vec<String>,
    /// Coverage percentage.
    pub coverage: f64,
}

/// T3: ExplorationAct - Capability 30 of 37.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationAct {
    /// Capability ID.
    pub id: String,
    /// Active status.
    pub exploration_active: bool,
    /// Active missions.
    missions: HashMap<String, MissionManifest>,
    /// Discoveries made.
    discoveries: Vec<Discovery>,
    /// Current frontier map.
    frontier: FrontierMap,
}

impl Default for ExplorationAct {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplorationAct {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            id: "CAP-030".into(),
            exploration_active: true,
            missions: HashMap::new(),
            discoveries: Vec::new(),
            frontier: FrontierMap {
                known: Vec::new(),
                unknown: Vec::new(),
                gaps: Vec::new(),
                coverage: 0.0,
            },
        }
    }

    /// Launch exploration mission.
    pub fn launch_mission(&mut self, manifest: MissionManifest) -> Measured<MissionManifest> {
        let confidence = manifest.success_prob;
        self.missions.insert(manifest.id.clone(), manifest.clone());
        Measured::uncertain(manifest, Confidence::new(confidence))
    }

    /// Record a discovery.
    pub fn record_discovery(&mut self, discovery: Discovery) -> Measured<DiscoveryIndex> {
        let significance = discovery.significance;
        self.discoveries.push(discovery);
        Measured::uncertain(significance, Confidence::new(significance.0))
    }

    /// Update frontier map.
    pub fn update_frontier(&mut self, known: Vec<String>, unknown: Vec<String>) {
        let total = known.len() + unknown.len();
        let coverage = if total > 0 {
            known.len() as f64 / total as f64
        } else {
            0.0
        };

        self.frontier = FrontierMap {
            known,
            unknown,
            gaps: Vec::new(),
            coverage,
        };
    }

    /// Get frontier map.
    pub fn get_frontier(&self) -> &FrontierMap {
        &self.frontier
    }

    /// Get discoveries.
    pub fn get_discoveries(&self) -> &[Discovery] {
        &self.discoveries
    }

    /// Get significant discoveries.
    pub fn significant_discoveries(&self) -> Vec<&Discovery> {
        self.discoveries
            .iter()
            .filter(|d| d.significance.is_significant())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_mission() {
        let mut exp = ExplorationAct::new();
        let manifest = MissionManifest {
            id: "mission-1".into(),
            target: "src/".into(),
            objective: "Find error handlers".into(),
            scope: ExplorationScope::Medium,
            patterns: vec!["error".into(), "Error".into()],
            success_prob: 0.8,
        };

        let result = exp.launch_mission(manifest);
        assert_eq!(result.value.scope, ExplorationScope::Medium);
    }

    #[test]
    fn test_record_discovery() {
        let mut exp = ExplorationAct::new();
        let discovery = Discovery {
            id: "disc-1".into(),
            finding: "Error handler in auth module".into(),
            location: Some("src/auth/error.rs:45".into()),
            significance: DiscoveryIndex::new(0.85),
            related: vec![],
        };

        let result = exp.record_discovery(discovery);
        assert!(result.value.is_significant());
    }

    #[test]
    fn test_frontier_coverage() {
        let mut exp = ExplorationAct::new();
        exp.update_frontier(vec!["src/".into(), "tests/".into()], vec!["docs/".into()]);

        let frontier = exp.get_frontier();
        assert!((frontier.coverage - 0.666).abs() < 0.01);
    }
}
