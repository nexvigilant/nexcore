//! Governance-aware reconciliation engine.
//!
//! Compares **declared state** ([`Bay`] loaded from TOML manifests) against
//! **actual state** ([`WorkspaceScan`] from the filesystem) and produces a
//! [`ForgePlan`] — an ordered list of typed actions to bring the two into
//! alignment.
//!
//! Six action variants, ordered by descending priority:
//!
//! | Variant | Priority | Meaning |
//! |---------|----------|---------|
//! | [`ForgeAction::OrphanCrate`] | 1 | In workspace but not in any hold |
//! | [`ForgeAction::MissingCrate`] | 2 | Declared in hold but absent from workspace |
//! | [`ForgeAction::DirectionViolation`] | 3 | Dep flows against hold-level direction |
//! | [`ForgeAction::LayerViolation`] | 4 | Dep depth ≠ declared layer |
//! | [`ForgeAction::GovernanceDrift`] | 5 | Manifest vs workspace mismatch |
//! | [`ForgeAction::SuggestMove`] | 6 | Dependency pattern suggests different hold |

use crate::bridge::scanner::{self, WorkspaceScan};
use crate::{Bay, Layer, TopologyError};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

// ============================================================================
// Action Types
// ============================================================================

/// A single reconciliation action.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "action_type")]
pub enum ForgeAction {
    /// Crate exists in workspace but is not declared in any hold.
    OrphanCrate { crate_name: String },

    /// Crate is declared in a hold but does not exist in the workspace.
    MissingCrate {
        crate_name: String,
        hold_name: String,
    },

    /// A dependency edge flows against the declared hold-level direction
    /// (higher layer depends on lower, violating the DAG).
    DirectionViolation {
        source_crate: String,
        source_hold: String,
        target_crate: String,
        target_hold: String,
        declared_direction: String,
    },

    /// A crate's actual dependency depth doesn't match its declared layer.
    LayerViolation {
        crate_name: String,
        hold_name: String,
        declared_layer: String,
        actual_depth: u32,
    },

    /// A governance field diverges between manifest and workspace state.
    GovernanceDrift {
        hold_name: String,
        field: String,
        declared_value: String,
        actual_value: String,
    },

    /// Dependency patterns suggest the crate belongs in a different hold.
    SuggestMove {
        crate_name: String,
        current_hold: String,
        suggested_hold: String,
        reason: String,
    },
}

impl ForgeAction {
    /// Priority for ordering (1 = highest).
    #[must_use]
    pub fn priority(&self) -> u8 {
        match self {
            Self::OrphanCrate { .. } => 1,
            Self::MissingCrate { .. } => 2,
            Self::DirectionViolation { .. } => 3,
            Self::LayerViolation { .. } => 4,
            Self::GovernanceDrift { .. } => 5,
            Self::SuggestMove { .. } => 6,
        }
    }

    /// Action type as a display string.
    #[must_use]
    pub fn action_type_name(&self) -> &'static str {
        match self {
            Self::OrphanCrate { .. } => "OrphanCrate",
            Self::MissingCrate { .. } => "MissingCrate",
            Self::DirectionViolation { .. } => "DirectionViolation",
            Self::LayerViolation { .. } => "LayerViolation",
            Self::GovernanceDrift { .. } => "GovernanceDrift",
            Self::SuggestMove { .. } => "SuggestMove",
        }
    }

    /// Primary target of this action (usually a crate or hold name).
    #[must_use]
    pub fn target(&self) -> &str {
        match self {
            Self::OrphanCrate { crate_name } | Self::MissingCrate { crate_name, .. } => crate_name,
            Self::DirectionViolation { source_crate, .. } => source_crate,
            Self::LayerViolation { crate_name, .. } => crate_name,
            Self::GovernanceDrift { hold_name, .. } => hold_name,
            Self::SuggestMove { crate_name, .. } => crate_name,
        }
    }

    /// Human-readable detail line.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::OrphanCrate { crate_name } => {
                format!("{crate_name} exists in workspace but is not assigned to any hold")
            }
            Self::MissingCrate {
                crate_name,
                hold_name,
            } => format!("{crate_name} declared in hold '{hold_name}' but not found in workspace"),
            Self::DirectionViolation {
                source_crate,
                source_hold,
                target_crate,
                target_hold,
                declared_direction,
            } => format!(
                "{source_crate} ({source_hold}) → {target_crate} ({target_hold}) \
                 violates direction: {declared_direction}"
            ),
            Self::LayerViolation {
                crate_name,
                hold_name,
                declared_layer,
                actual_depth,
            } => format!(
                "{crate_name} in hold '{hold_name}': declared layer {declared_layer}, \
                 actual depth {actual_depth}"
            ),
            Self::GovernanceDrift {
                hold_name,
                field,
                declared_value,
                actual_value,
            } => format!(
                "hold '{hold_name}': {field} declared={declared_value}, actual={actual_value}"
            ),
            Self::SuggestMove {
                crate_name,
                current_hold,
                suggested_hold,
                reason,
            } => format!(
                "suggest moving {crate_name} from '{current_hold}' to '{suggested_hold}': {reason}"
            ),
        }
    }
}

// ============================================================================
// Complexity Rating
// ============================================================================

/// Complexity of a reconciliation plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Complexity {
    /// 0-10 actions.
    Low,
    /// 11-50 actions.
    Medium,
    /// 51+ actions.
    High,
}

impl Complexity {
    /// Classify from action count.
    #[must_use]
    pub fn from_count(count: usize) -> Self {
        if count <= 10 {
            Self::Low
        } else if count <= 50 {
            Self::Medium
        } else {
            Self::High
        }
    }
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

// ============================================================================
// Plan Summary
// ============================================================================

/// Summary statistics for a [`ForgePlan`].
#[derive(Debug, Clone, Serialize)]
pub struct PlanSummary {
    /// Total number of actions.
    pub total_actions: usize,
    /// Count by action type.
    pub by_type: BTreeMap<String, usize>,
    /// Complexity rating.
    pub complexity: Complexity,
}

// ============================================================================
// ForgePlan
// ============================================================================

/// Ordered reconciliation plan: actions sorted by priority, serializable.
#[derive(Debug, Clone, Serialize)]
pub struct ForgePlan {
    actions: Vec<ForgeAction>,
}

impl ForgePlan {
    /// Create a plan from unsorted actions, ordering by priority.
    #[must_use]
    pub fn new(mut actions: Vec<ForgeAction>) -> Self {
        actions.sort_by_key(|a| (a.priority(), a.target().to_owned()));
        Self { actions }
    }

    /// Ordered action list.
    #[must_use]
    pub fn actions(&self) -> &[ForgeAction] {
        &self.actions
    }

    /// Total number of actions.
    #[must_use]
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// True when the declared and actual states are in agreement.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.actions.is_empty()
    }

    /// Compute summary statistics.
    #[must_use]
    pub fn summary(&self) -> PlanSummary {
        let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
        for action in &self.actions {
            *by_type
                .entry(action.action_type_name().to_owned())
                .or_insert(0) = by_type
                .get(action.action_type_name())
                .copied()
                .unwrap_or(0)
                .saturating_add(1);
        }
        PlanSummary {
            total_actions: self.actions.len(),
            by_type,
            complexity: Complexity::from_count(self.actions.len()),
        }
    }

    /// Render as markdown report.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let summary = self.summary();
        let mut md = String::from("# Reconciliation Plan\n\n");

        md.push_str(&format!(
            "**Total actions:** {} | **Complexity:** {}\n\n",
            summary.total_actions, summary.complexity
        ));

        if self.actions.is_empty() {
            md.push_str("No actions required — declared and actual states match.\n");
            return md;
        }

        // Summary table
        md.push_str("## Summary by Type\n\n");
        md.push_str("| Action Type | Count |\n|-------------|-------|\n");
        for (name, count) in &summary.by_type {
            md.push_str(&format!("| {name} | {count} |\n"));
        }
        md.push('\n');

        // Detailed actions
        md.push_str("## Actions\n\n");
        md.push_str("| # | Priority | Type | Target | Detail |\n");
        md.push_str("|---|----------|------|--------|--------|\n");

        for (i, action) in self.actions.iter().enumerate() {
            let idx = i.saturating_add(1);
            md.push_str(&format!(
                "| {idx} | {} | {} | {} | {} |\n",
                action.priority(),
                action.action_type_name(),
                action.target(),
                action.detail(),
            ));
        }

        md
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    /// Returns [`TopologyError::SerializeFailed`] on serialization failure.
    pub fn to_json(&self) -> Result<String, TopologyError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| TopologyError::SerializeFailed(e.to_string()))
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Reconcile declared state (Bay) against actual workspace state.
///
/// Scans the workspace at `workspace_root`, then compares against `declared`.
///
/// # Errors
/// - [`TopologyError::ScanFailed`] if the workspace scan fails.
/// - [`TopologyError::SerializeFailed`] only from [`ForgePlan::to_json`].
pub fn reconcile(declared: &Bay, workspace_root: &Path) -> Result<ForgePlan, TopologyError> {
    let scan = scanner::scan_workspace(workspace_root)
        .map_err(|e| TopologyError::ScanFailed(e.to_string()))?;
    reconcile_with_scan(declared, &scan)
}

/// Reconcile declared state against a pre-computed scan (testable core).
///
/// # Errors
/// Returns [`TopologyError`] if reconciliation logic fails.
pub fn reconcile_with_scan(
    declared: &Bay,
    scan: &WorkspaceScan,
) -> Result<ForgePlan, TopologyError> {
    let mut actions: Vec<ForgeAction> = Vec::new();

    // Phase 1: Orphan crates (in workspace but not in any hold)
    let declared_crates: BTreeSet<&str> = declared
        .holds()
        .values()
        .flat_map(|h| h.members().iter().map(String::as_str))
        .collect();

    for ws_crate in &scan.crates {
        if !declared_crates.contains(ws_crate.as_str()) {
            actions.push(ForgeAction::OrphanCrate {
                crate_name: ws_crate.clone(),
            });
        }
    }

    // Phase 2: Missing crates (declared but not in workspace)
    for (hold_name, hold) in declared.holds() {
        for member in hold.members() {
            if !scan.crates.contains(member) {
                actions.push(ForgeAction::MissingCrate {
                    crate_name: member.clone(),
                    hold_name: hold_name.clone(),
                });
            }
        }
    }

    // Phase 3: Direction violations
    // Build hold-level layer map from governance.
    let hold_layers = compute_hold_layers(declared);
    let hold_pair_edges = compute_hold_pair_edges(declared, scan);

    for (source_hold, target_hold, source_crate, target_crate) in &hold_pair_edges {
        let source_layer = hold_layers.get(source_hold.as_str());
        let target_layer = hold_layers.get(target_hold.as_str());

        if let (Some(&sl), Some(&tl)) = (source_layer, target_layer) {
            // Direction violation: source is at a lower layer than target
            // (Foundation depending on Domain, etc.)
            if sl.depth() < tl.depth() {
                actions.push(ForgeAction::DirectionViolation {
                    source_crate: source_crate.clone(),
                    source_hold: source_hold.clone(),
                    target_crate: target_crate.clone(),
                    target_hold: target_hold.clone(),
                    declared_direction: format!(
                        "{} → {} (expected: higher depends on lower)",
                        sl, tl
                    ),
                });
            }
        }
    }

    // Phase 4: Layer violations (actual dep depth exceeds declared layer)
    //
    // A crate is only a violation when its inferred layer is DEEPER than the
    // hold's declared layer (e.g., Domain-depth crate in a Foundation hold).
    // Shallower crates are fine: a Foundation crate in a Domain hold is
    // a normal member, not a violation.
    let dep_depths = compute_dep_depths(scan);
    for (hold_name, hold) in declared.holds() {
        let declared_layer = hold_layers.get(hold_name.as_str());
        if let Some(&layer) = declared_layer {
            for member in hold.members() {
                if let Some(&depth) = dep_depths.get(member.as_str()) {
                    let inferred = infer_layer_from_depth(depth);
                    if inferred.depth() > layer.depth() {
                        actions.push(ForgeAction::LayerViolation {
                            crate_name: member.clone(),
                            hold_name: hold_name.clone(),
                            declared_layer: layer.to_string(),
                            actual_depth: depth,
                        });
                    }
                }
            }
        }
    }

    // Phase 5: Suggest moves — crate whose deps are mostly in another hold
    suggest_moves(declared, scan, &hold_layers, &mut actions);

    Ok(ForgePlan::new(actions))
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Resolve hold layer: explicit field → owner convention → name heuristic.
///
/// Prefers the explicit `layer` field set from manifest data. Falls back to
/// inferring from owner string keywords, then hold name keywords.
///
/// Applies a name-based floor: holds whose names indicate foundation-level
/// content (`"primitive"`, `"stem"`, `"foundation"` keywords) are clamped to
/// [`Layer::Foundation`] regardless of the explicit or inferred layer. This
/// corrects bootstrap-generated layers where a foundation hold's max dep
/// count pushes it to Domain due to a single high-dep member.
fn compute_hold_layers(declared: &Bay) -> BTreeMap<&str, Layer> {
    let mut result: BTreeMap<&str, Layer> = BTreeMap::new();
    for (name, hold) in declared.holds() {
        // Prefer explicit layer from manifest
        let inferred = if let Some(explicit) = hold.layer() {
            explicit
        } else {
            // Fallback: infer from owner convention
            let owner_lower = hold.governance().owner.to_lowercase();
            if owner_lower.contains("foundation") {
                Layer::Foundation
            } else if owner_lower.contains("domain") {
                Layer::Domain
            } else if owner_lower.contains("orchestration") {
                Layer::Orchestration
            } else if owner_lower.contains("service") {
                Layer::Service
            } else {
                // Last resort: infer from hold name
                infer_layer_from_name(name)
            }
        };

        // Name-based floor: foundation holds cannot be reclassified higher.
        // This corrects bootstrap-generated layers where dep-count-based
        // inference assigns Domain to a hold like "core-primitives" because
        // one member (nexcore-core) has many deps.
        let layer = if is_foundation_hold_name(name) {
            Layer::Foundation
        } else {
            inferred
        };

        result.insert(name.as_str(), layer);
    }
    result
}

/// Check if a hold name indicates foundation-level content.
///
/// Matches on hyphen-delimited segments to avoid false positives:
/// `"stem-foundation"` matches ("stem" segment) but `"guardian-system"` does
/// not ("system" ≠ "stem" even though it contains the substring).
fn is_foundation_hold_name(name: &str) -> bool {
    name.to_lowercase().split('-').any(|seg| {
        seg == "foundation" || seg == "stem" || seg == "primitive" || seg == "primitives"
    })
}

/// Infer layer from hold name convention.
///
/// Defaults to [`Layer::Domain`] because most holds are domain holds.
/// Foundation holds are identified by name keywords ("foundation", "stem",
/// "primitive"). Defaulting to Foundation would cause the directional filter
/// to treat domain holds as same-layer as Foundation, masking gravity signals.
///
/// Matches on hyphen-delimited segments to avoid false positives (e.g.
/// `"system"` must not match `"stem"`).
fn infer_layer_from_name(name: &str) -> Layer {
    let lower = name.to_lowercase();
    let has = |kw: &str| lower.split('-').any(|seg| seg == kw);

    if has("foundation") || has("stem") || has("primitive") || has("primitives") {
        Layer::Foundation
    } else if has("domain") || has("vigilance") || has("pv") {
        Layer::Domain
    } else if has("orchestration") || has("brain") || has("vigil") {
        Layer::Orchestration
    } else if has("service") || has("mcp") || has("api") {
        Layer::Service
    } else {
        // Default to Domain: most holds contain domain-level crates.
        // Foundation holds are caught by name keywords above.
        Layer::Domain
    }
}

/// Compute edges between holds based on actual crate-level dependencies.
///
/// Returns (source_hold, target_hold, source_crate, target_crate) tuples.
fn compute_hold_pair_edges(
    declared: &Bay,
    scan: &WorkspaceScan,
) -> Vec<(String, String, String, String)> {
    let mut result = Vec::new();
    for (from_crate, to_crate) in &scan.edges {
        let from_hold = declared.find_crate(from_crate);
        let to_hold = declared.find_crate(to_crate);
        if let (Some(fh), Some(th)) = (from_hold, to_hold) {
            if fh != th {
                result.push((
                    fh.to_owned(),
                    th.to_owned(),
                    from_crate.clone(),
                    to_crate.clone(),
                ));
            }
        }
    }
    result
}

/// Compute max internal dependency depth for each crate.
///
/// Uses the scan edges: depth = number of transitive internal dependencies.
fn compute_dep_depths(scan: &WorkspaceScan) -> BTreeMap<&str, u32> {
    // Simple: count direct internal deps as a proxy for depth.
    let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
    for c in &scan.crates {
        counts.insert(c.as_str(), 0);
    }
    for (from, _to) in &scan.edges {
        if let Some(count) = counts.get_mut(from.as_str()) {
            *count = count.saturating_add(1);
        }
    }
    counts
}

/// Map dependency count to layer using the documented thresholds.
///
/// Foundation: 0-3, Domain: 4-25, Orchestration: 26-50, Service: 51+.
fn infer_layer_from_depth(dep_count: u32) -> Layer {
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

/// Phase 5: Suggest moves for crates whose dependencies cluster in another hold.
///
/// Includes a directional filter: suppress suggestions where the suggested
/// hold is at a LOWER layer depth than the crate's current hold. Domain crates
/// being pulled toward Foundation is normal dependency gravity, not a
/// misplacement signal. Only lateral (same layer) or upward (toward higher
/// layers) moves are surfaced.
fn suggest_moves(
    declared: &Bay,
    scan: &WorkspaceScan,
    hold_layers: &BTreeMap<&str, Layer>,
    actions: &mut Vec<ForgeAction>,
) {
    // For each crate, count how many of its deps are in each hold.
    // If a majority are in a different hold, suggest a move.
    for (from_crate, _) in scan.edges.iter() {
        let current_hold = match declared.find_crate(from_crate) {
            Some(h) => h,
            None => continue, // orphan, already handled
        };

        // Count deps per hold
        let mut hold_dep_counts: BTreeMap<&str, usize> = BTreeMap::new();
        let mut total_deps: usize = 0;

        for (fc, tc) in &scan.edges {
            if fc == from_crate {
                if let Some(target_hold) = declared.find_crate(tc) {
                    *hold_dep_counts.entry(target_hold).or_insert(0) = hold_dep_counts
                        .get(target_hold)
                        .copied()
                        .unwrap_or(0)
                        .saturating_add(1);
                    total_deps = total_deps.saturating_add(1);
                }
            }
        }

        // Need at least 3 deps to make a suggestion
        if total_deps < 3 {
            continue;
        }

        // Find the hold with the most deps
        let mut best_hold = current_hold;
        let mut best_count: usize = 0;
        for (&hold, &count) in &hold_dep_counts {
            if count > best_count {
                best_count = count;
                best_hold = hold;
            }
        }

        // Suggest move if ≥60% of deps are in a different hold
        // threshold = total * 3 / 5 (avoids float)
        let threshold = total_deps.saturating_mul(3) / 5;
        if best_hold != current_hold && best_count > threshold {
            // Directional filter: suppress foundation gravity.
            //
            // Two complementary checks:
            //
            // 1. Foundation-name filter: never suggest moves INTO a hold whose
            //    name indicates foundation-level content ("primitive", "stem",
            //    "foundation"). These holds attract deps from all layers by
            //    structural necessity — a suggestion to move there is always
            //    gravity, never misplacement.
            //
            // 2. Layer-depth filter: suppress moves toward a lower layer.
            //    A Domain crate pulled toward Foundation is dependency gravity.
            //    Only surface lateral (same layer) or upward suggestions.
            if is_foundation_hold_name(best_hold) {
                continue;
            }

            let current_depth = hold_layers.get(current_hold).map_or(0, |l| l.depth());
            let suggested_depth = hold_layers.get(best_hold).map_or(0, |l| l.depth());

            if suggested_depth < current_depth {
                continue;
            }

            actions.push(ForgeAction::SuggestMove {
                crate_name: from_crate.clone(),
                current_hold: current_hold.to_owned(),
                suggested_hold: best_hold.to_owned(),
                reason: format!("{best_count}/{total_deps} dependencies are in hold '{best_hold}'"),
            });
        }
    }

    // Deduplicate: same crate may appear in multiple edges
    actions.sort_by(|a, b| a.target().cmp(b.target()));
    actions
        .dedup_by(|a, b| a.action_type_name() == b.action_type_name() && a.target() == b.target());
}

#[cfg(test)]
mod tests;
