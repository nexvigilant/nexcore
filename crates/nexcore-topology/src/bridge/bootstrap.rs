//! Bootstrap topology from Link 1 JSON → TOML manifests.
//!
//! Reads the `workspace-topology.json` produced by the Link 1 audit and
//! generates initial TOML manifests that round-trip through the manifest
//! loaders.
//!
//! ## JSON Schema (input)
//!
//! ```json
//! {
//!   "metadata": { "total_crates": 224, ... },
//!   "summary": {
//!     "layers": { "Foundation": 18, "Domain": 33, ... },
//!     "clusters": [{ "name": "...", "count": N, "members": [...] }],
//!     "capability_paths": [{ "service": "...", "path": [...], "depth": N }]
//!   },
//!   "crates": [{
//!     "name": "...",
//!     "layer": "Foundation|Domain|Orchestration|Service",
//!     "domain_cluster": "...",
//!     "internal_dep_count": N,
//!     "dependent_count": N,
//!     "dependencies": [...],
//!     "dependents": [...]
//!   }]
//! }
//! ```

use crate::bridge::manifest::{self, ManifestError};
use crate::{Bay, GovernancePolicy, Hold, Layer};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

// ============================================================================
// JSON Schema Types (deserialization targets for workspace-topology.json)
// ============================================================================

/// Top-level topology JSON.
#[derive(Debug, Deserialize)]
pub struct TopologyJson {
    /// File metadata.
    pub metadata: TopologyMetadata,
    /// Summary section.
    pub summary: TopologySummary,
    /// Per-crate entries.
    pub crates: Vec<CrateEntry>,
}

/// Metadata from the topology JSON.
#[derive(Debug, Deserialize)]
pub struct TopologyMetadata {
    /// Total crate count.
    pub total_crates: usize,
}

/// Summary section of the topology JSON.
#[derive(Debug, Deserialize)]
pub struct TopologySummary {
    /// Layer counts.
    pub layers: BTreeMap<String, usize>,
    /// Domain clusters.
    pub clusters: Vec<ClusterEntry>,
    /// Capability paths.
    #[serde(default)]
    pub capability_paths: Vec<CapabilityPath>,
}

/// A domain cluster.
#[derive(Debug, Deserialize)]
pub struct ClusterEntry {
    /// Cluster name (e.g., "biological-system", "core-primitives").
    pub name: String,
    /// Number of crates.
    pub count: usize,
    /// Member crate names.
    pub members: Vec<String>,
}

/// A capability path from service to foundation.
#[derive(Debug, Deserialize)]
pub struct CapabilityPath {
    /// Service-layer crate at the top of the path.
    pub service: String,
    /// Crate names in order from service to foundation.
    pub path: Vec<String>,
    /// Path depth.
    pub depth: usize,
}

/// Per-crate entry in the topology JSON.
#[derive(Debug, Deserialize)]
pub struct CrateEntry {
    /// Crate name.
    pub name: String,
    /// Assigned layer.
    pub layer: String,
    /// Domain cluster name.
    pub domain_cluster: String,
    /// Count of internal dependencies.
    pub internal_dep_count: usize,
    /// Count of internal dependents.
    pub dependent_count: usize,
    /// Names of internal dependencies.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Names of internal dependents.
    #[serde(default)]
    pub dependents: Vec<String>,
}

// ============================================================================
// Bootstrap Error
// ============================================================================

/// Errors from the bootstrap process.
#[derive(Debug, Clone)]
pub enum BootstrapError {
    /// JSON parsing failed.
    JsonParse(String),
    /// Manifest construction failed.
    Manifest(ManifestError),
    /// IO error.
    Io(String),
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JsonParse(msg) => write!(f, "JSON parse error: {msg}"),
            Self::Manifest(e) => write!(f, "manifest error: {e}"),
            Self::Io(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for BootstrapError {}

impl From<ManifestError> for BootstrapError {
    fn from(e: ManifestError) -> Self {
        Self::Manifest(e)
    }
}

// ============================================================================
// Bootstrap Result
// ============================================================================

/// Result of bootstrapping from topology JSON.
#[derive(Debug)]
pub struct BootstrapResult {
    /// The constructed bay.
    pub bay: Bay,
    /// Generated bay TOML manifest string.
    pub bay_toml: String,
    /// Per-hold TOML strings (hold_name → toml).
    pub hold_tomls: BTreeMap<String, String>,
    /// Crates that could not be assigned to a hold (empty cluster name).
    pub unassigned: Vec<String>,
    /// Statistics about the bootstrap.
    pub stats: BootstrapStats,
}

/// Statistics from the bootstrap process.
#[derive(Debug, Clone)]
pub struct BootstrapStats {
    /// Total crates in the JSON.
    pub total_crates: usize,
    /// Crates successfully assigned to holds.
    pub assigned_crates: usize,
    /// Number of holds generated.
    pub hold_count: usize,
    /// Assignment coverage as a percentage.
    pub coverage_pct: f64,
}

// ============================================================================
// Public API
// ============================================================================

/// Bootstrap a [`Bay`] from a topology JSON string.
///
/// Groups crates by `domain_cluster` to form holds. Each cluster becomes
/// a hold with a default governance policy. Returns both the constructed
/// [`Bay`] and the generated TOML manifests.
///
/// # Errors
/// Returns [`BootstrapError`] on JSON parse failure or hold construction error.
pub fn from_topology_json(json_str: &str) -> Result<BootstrapResult, BootstrapError> {
    let topo: TopologyJson =
        serde_json::from_str(json_str).map_err(|e| BootstrapError::JsonParse(e.to_string()))?;

    // Group crates by domain_cluster
    let mut cluster_members: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut unassigned: Vec<String> = Vec::new();

    for crate_entry in &topo.crates {
        if crate_entry.domain_cluster.is_empty() {
            unassigned.push(crate_entry.name.clone());
        } else {
            cluster_members
                .entry(crate_entry.domain_cluster.clone())
                .or_default()
                .insert(crate_entry.name.clone());
        }
    }

    // Compute majority layer per cluster from per-crate layer assignments
    let cluster_layers = compute_cluster_layers(&topo.crates, &cluster_members);

    // Build Hold for each cluster
    let mut holds: Vec<Hold> = Vec::new();
    let mut hold_tomls: BTreeMap<String, String> = BTreeMap::new();

    for (cluster_name, members) in &cluster_members {
        if members.is_empty() {
            continue;
        }

        let governance = GovernancePolicy {
            owner: format!("{cluster_name}-team"),
            review_required: true,
        };

        let hold = Hold::new(cluster_name.clone(), members.clone(), governance)
            .map_err(|e| BootstrapError::Manifest(ManifestError::from(e)))?;

        // Detect internal cycles within this cluster
        let cycle_partners = detect_intra_cluster_cycles(&topo.crates, members);
        let hold = if cycle_partners.is_empty() {
            hold
        } else {
            hold.with_cycle_partners(cycle_partners)
        };

        // Set explicit layer from majority vote across cluster members
        let hold = if let Some(&layer) = cluster_layers.get(cluster_name.as_str()) {
            hold.with_layer(layer)
        } else {
            hold
        };

        hold_tomls.insert(cluster_name.clone(), manifest::hold_to_toml(&hold));
        holds.push(hold);
    }

    let total_crates = topo.metadata.total_crates;
    let assigned_crates: usize = holds.iter().map(Hold::member_count).sum();
    let hold_count = holds.len();
    #[allow(
        clippy::as_conversions,
        reason = "usize→f64 is safe for counts under 2^53"
    )]
    let coverage_pct = if total_crates > 0 {
        (assigned_crates as f64 / total_crates as f64) * 100.0
    } else {
        0.0
    };

    let bay = Bay::new(holds).map_err(|e| BootstrapError::Manifest(ManifestError::from(e)))?;
    let bay_toml = manifest::bay_to_toml(&bay, "nexcore-workspace");

    Ok(BootstrapResult {
        bay,
        bay_toml,
        hold_tomls,
        unassigned,
        stats: BootstrapStats {
            total_crates,
            assigned_crates,
            hold_count,
            coverage_pct,
        },
    })
}

/// Bootstrap from a topology JSON file on disk.
///
/// # Errors
/// Returns [`BootstrapError::Io`] on read failure.
pub fn from_topology_file(path: &std::path::Path) -> Result<BootstrapResult, BootstrapError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| BootstrapError::Io(format!("{}: {e}", path.display())))?;
    from_topology_json(&content)
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Compute the layer ceiling for each cluster from per-crate dep counts.
///
/// Uses the maximum internal dep count across cluster members to determine
/// the hold's layer. This ensures no member is deeper than the declared
/// layer, which aligns with the reconcile engine's directional check:
/// violations fire only when a crate's depth exceeds the hold's layer.
///
/// Thresholds match `infer_layer_from_depth` in the reconcile engine:
/// Foundation: 0-3, Domain: 4-25, Orchestration: 26-50, Service: 51+.
fn compute_cluster_layers<'a>(
    crates: &[CrateEntry],
    cluster_members: &'a BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<&'a str, Layer> {
    let mut result: BTreeMap<&str, Layer> = BTreeMap::new();

    // Build crate→dep_count lookup
    let crate_dep_counts: BTreeMap<&str, usize> = crates
        .iter()
        .map(|c| (c.name.as_str(), c.internal_dep_count))
        .collect();

    for (cluster_name, members) in cluster_members {
        // Find the maximum dep count across all members
        let max_deps = members
            .iter()
            .filter_map(|m| crate_dep_counts.get(m.as_str()).copied())
            .max()
            .unwrap_or(0);

        result.insert(cluster_name.as_str(), layer_from_dep_count(max_deps));
    }

    result
}

/// Map dependency count to layer using the reconcile engine's thresholds.
///
/// Foundation: 0-3, Domain: 4-25, Orchestration: 26-50, Service: 51+.
fn layer_from_dep_count(dep_count: usize) -> Layer {
    if dep_count <= 3 {
        Layer::Foundation
    } else if dep_count <= 25 {
        Layer::Domain
    } else if dep_count <= 50 {
        Layer::Orchestration
    } else {
        Layer::Service
    }
}

/// Detect mutual dependency cycles within a cluster.
fn detect_intra_cluster_cycles(
    crates: &[CrateEntry],
    cluster_members: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut cycles = Vec::new();

    // Build a quick dependency lookup for cluster members
    let deps_by_name: BTreeMap<&str, &[String]> = crates
        .iter()
        .filter(|c| cluster_members.contains(&c.name))
        .map(|c| (c.name.as_str(), c.dependencies.as_slice()))
        .collect();

    for (name, deps) in &deps_by_name {
        for dep in *deps {
            if cluster_members.contains(dep) {
                // Check if the reverse edge exists
                if let Some(reverse_deps) = deps_by_name.get(dep.as_str()) {
                    if reverse_deps.iter().any(|d| d.as_str() == *name) && *name < dep.as_str() {
                        cycles.push(((*name).to_owned(), dep.clone()));
                    }
                }
            }
        }
    }

    cycles
}

/// Resolve a layer string from the topology JSON to a [`Layer`].
#[must_use]
pub fn resolve_layer(layer_str: &str) -> Option<Layer> {
    match layer_str {
        "Foundation" => Some(Layer::Foundation),
        "Domain" => Some(Layer::Domain),
        "Orchestration" => Some(Layer::Orchestration),
        "Service" => Some(Layer::Service),
        _ => None,
    }
}
