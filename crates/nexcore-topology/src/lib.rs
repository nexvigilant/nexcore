//! # nexcore-topology — Crates as Code
//!
//! Type-level organizational model for a Rust workspace, extending the
//! nautical metaphor (crate → cargo) with four container types:
//!
//! | Type | Dominant | Purpose |
//! |------|----------|---------|
//! | [`Hold`] | ∂ Boundary | Crates under shared governance |
//! | [`Compartment`] | σ Sequence | Acyclic hold-level DAG within a layer |
//! | [`Stack`] | → Causality | Connected Foundation→Service path |
//! | [`Bay`] | × Product | Complete crate-to-hold allocation |
//!
//! Every type implements [`GroundsTo`] tracing to T1 Lex Primitiva.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::string_slice,
    clippy::as_conversions
)]

pub mod bridge;
pub mod reconcile;

use nexcore_lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition, StateMode};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

// ============================================================================
// Error
// ============================================================================

/// Errors from topology construction and validation.
#[derive(Debug, Clone)]
pub enum TopologyError {
    /// Hold was constructed with zero members.
    EmptyHold { name: String },
    /// Compartment was constructed with zero holds.
    EmptyCompartment { name: String },
    /// A crate appears in more than one hold.
    DuplicateCrate {
        crate_name: String,
        hold_a: String,
        hold_b: String,
    },
    /// An edge references a hold that does not exist.
    MissingHold { name: String },
    /// The hold-level DAG contains a cycle.
    CycleDetected { hold: String },
    /// A stack has no segments.
    EmptyStack,
    /// Stack segments are not ordered by layer depth.
    SegmentOrderViolation { detail: String },
    /// Bay has no holds.
    EmptyBay,
    /// Workspace scan failed.
    ScanFailed(String),
    /// JSON serialization failed.
    SerializeFailed(String),
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyHold { name } => write!(f, "hold '{name}' has no members"),
            Self::EmptyCompartment { name } => write!(f, "compartment '{name}' has no holds"),
            Self::DuplicateCrate {
                crate_name,
                hold_a,
                hold_b,
            } => write!(f, "crate '{crate_name}' in both '{hold_a}' and '{hold_b}'"),
            Self::MissingHold { name } => write!(f, "edge references unknown hold '{name}'"),
            Self::CycleDetected { hold } => {
                write!(f, "cycle detected involving hold '{hold}'")
            }
            Self::EmptyStack => write!(f, "stack has no segments"),
            Self::SegmentOrderViolation { detail } => {
                write!(f, "stack segment order violation: {detail}")
            }
            Self::EmptyBay => write!(f, "bay has no holds"),
            Self::ScanFailed(msg) => write!(f, "workspace scan failed: {msg}"),
            Self::SerializeFailed(msg) => write!(f, "serialization failed: {msg}"),
        }
    }
}

impl std::error::Error for TopologyError {}

// ============================================================================
// Supporting Types
// ============================================================================

/// Workspace layer (dependency flows DOWN only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Layer {
    /// Core primitives, no domain knowledge (0-3 internal deps).
    Foundation = 0,
    /// Business logic, uses foundation types (2-25 internal deps).
    Domain = 1,
    /// Workflow coordination (3-5 internal deps).
    Orchestration = 2,
    /// External interfaces, binary targets (5-76 internal deps).
    Service = 3,
}

impl Layer {
    /// Numeric depth (Foundation=0, Service=3).
    #[must_use]
    pub const fn depth(&self) -> u8 {
        match self {
            Self::Foundation => 0,
            Self::Domain => 1,
            Self::Orchestration => 2,
            Self::Service => 3,
        }
    }

    /// All layers in dependency order (Foundation first).
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::Foundation,
            Self::Domain,
            Self::Orchestration,
            Self::Service,
        ]
    }

    /// Label for display.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Foundation => "Foundation",
            Self::Domain => "Domain",
            Self::Orchestration => "Orchestration",
            Self::Service => "Service",
        }
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Governance policy applied to all crates in a hold.
#[derive(Debug, Clone)]
pub struct GovernancePolicy {
    /// Team or individual responsible.
    pub owner: String,
    /// Whether changes require review approval.
    pub review_required: bool,
}

/// Sub-layer subdivision within a large compartment.
#[derive(Debug, Clone)]
pub struct SubLayer {
    /// Human-readable name (e.g., "core-primitives", "signal-pipeline").
    pub name: String,
    /// Relative depth within the compartment (0-based).
    pub depth: u32,
    /// Hold names assigned to this sub-layer.
    pub holds: Vec<String>,
}

/// One segment of a capability path (stack).
#[derive(Debug, Clone)]
pub struct StackSegment {
    /// Layer this segment belongs to.
    pub layer: Layer,
    /// Hold name at this layer.
    pub hold_name: String,
    /// Representative crate name.
    pub crate_name: String,
}

/// Typed diff produced by [`Bay::reconcile`].
#[derive(Debug, Clone, Default)]
pub struct BayDiff {
    /// Holds present in `other` but not `self`.
    pub holds_added: Vec<String>,
    /// Holds present in `self` but not `other`.
    pub holds_removed: Vec<String>,
    /// Crates that moved between holds: (crate, from_hold, to_hold).
    pub crates_moved: Vec<(String, String, String)>,
    /// Crates present in `other` but not `self`.
    pub crates_added: Vec<String>,
    /// Crates present in `self` but not `other`.
    pub crates_removed: Vec<String>,
}

impl BayDiff {
    /// True when no differences exist.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.holds_added.is_empty()
            && self.holds_removed.is_empty()
            && self.crates_moved.is_empty()
            && self.crates_added.is_empty()
            && self.crates_removed.is_empty()
    }
}

// ============================================================================
// Hold — governance boundary around a set of crates
// ============================================================================

/// A group of crates under shared governance.
///
/// The Hold extends Rust's nautical metaphor: where `crate` is a unit of
/// compilation (cargo), a `Hold` is a ship's compartment grouping related cargo
/// under a single governance policy.
///
/// ## Invariant
/// A hold must contain at least one crate.
#[derive(Debug, Clone)]
pub struct Hold {
    name: String,
    members: BTreeSet<String>,
    governance: GovernancePolicy,
    cycle_partners: Vec<(String, String)>,
    /// Explicit layer assignment (overrides heuristic inference).
    layer: Option<Layer>,
}

impl Hold {
    /// Construct a hold, validating non-empty membership.
    ///
    /// # Errors
    /// Returns [`TopologyError::EmptyHold`] if `members` is empty.
    pub fn new(
        name: impl Into<String>,
        members: BTreeSet<String>,
        governance: GovernancePolicy,
    ) -> Result<Self, TopologyError> {
        let name = name.into();
        if members.is_empty() {
            return Err(TopologyError::EmptyHold { name });
        }
        Ok(Self {
            name,
            members,
            governance,
            cycle_partners: Vec::new(),
            layer: None,
        })
    }

    /// Declare mutual dependency cycles between crate pairs.
    #[must_use]
    pub fn with_cycle_partners(mut self, partners: Vec<(String, String)>) -> Self {
        self.cycle_partners = partners;
        self
    }

    /// Set explicit layer assignment.
    #[must_use]
    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layer = Some(layer);
        self
    }

    /// Explicit layer assignment, if set.
    #[must_use]
    pub fn layer(&self) -> Option<Layer> {
        self.layer
    }

    /// Hold name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set of crate names in this hold.
    #[must_use]
    pub fn members(&self) -> &BTreeSet<String> {
        &self.members
    }

    /// Governance policy.
    #[must_use]
    pub fn governance(&self) -> &GovernancePolicy {
        &self.governance
    }

    /// Declared cycle-partner pairs.
    #[must_use]
    pub fn cycle_partners(&self) -> &[(String, String)] {
        &self.cycle_partners
    }

    /// Number of crates.
    #[must_use]
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Whether any cycle partners are declared.
    #[must_use]
    pub fn has_cycles(&self) -> bool {
        !self.cycle_partners.is_empty()
    }

    /// Check membership.
    #[must_use]
    pub fn contains(&self, crate_name: &str) -> bool {
        self.members.contains(crate_name)
    }
}

// CALIBRATION: Hold grounds to ∂ (Boundary) dominant because a hold
// defines a governance boundary around crates. Supporting primitives:
// ∃ (Existence) — members are instantiated within the hold;
// N (Quantity) — hold has countable membership;
// ς (State) — governance policy is a stateful, modal configuration
// (active / archived / frozen).
// Confidence 0.70: high — containment is the primary semantic.
impl GroundsTo for Hold {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.7)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        // CALIBRATION: Governance policy is a discrete FSM (active/archived/frozen)
        Some(StateMode::Modal)
    }
}

// ============================================================================
// Compartment — acyclic hold-level DAG within a layer
// ============================================================================

/// An acyclic DAG of holds within a single workspace layer.
///
/// A Compartment partitions a layer's holds into an ordered dependency
/// structure. Large compartments support sub-layers for finer-grained
/// organization (e.g., Service layer with 126 crates).
///
/// ## Invariants
/// - Must contain at least one hold.
/// - All edge endpoints must reference holds in this compartment.
/// - The hold-level dependency graph must be acyclic (verified via Kahn's).
#[derive(Debug, Clone)]
pub struct Compartment {
    name: String,
    layer: Layer,
    holds: BTreeMap<String, Hold>,
    edges: Vec<(String, String)>,
    sub_layers: Vec<SubLayer>,
}

impl Compartment {
    /// Construct a compartment, validating acyclicity and edge integrity.
    ///
    /// `edges` are directed pairs `(from, to)` meaning `from` depends on `to`.
    ///
    /// # Errors
    /// - [`TopologyError::EmptyCompartment`] if `holds` is empty.
    /// - [`TopologyError::MissingHold`] if an edge references a non-existent hold.
    /// - [`TopologyError::CycleDetected`] if the hold DAG contains a cycle.
    pub fn new(
        name: impl Into<String>,
        layer: Layer,
        holds: BTreeMap<String, Hold>,
        edges: Vec<(String, String)>,
    ) -> Result<Self, TopologyError> {
        let name = name.into();
        if holds.is_empty() {
            return Err(TopologyError::EmptyCompartment { name });
        }

        // Validate edge endpoints
        for (from, to) in &edges {
            if !holds.contains_key(from) {
                return Err(TopologyError::MissingHold { name: from.clone() });
            }
            if !holds.contains_key(to) {
                return Err(TopologyError::MissingHold { name: to.clone() });
            }
        }

        // Validate acyclicity via Kahn's algorithm
        validate_acyclic(&holds, &edges)?;

        Ok(Self {
            name,
            layer,
            holds,
            edges,
            sub_layers: Vec::new(),
        })
    }

    /// Attach sub-layer subdivisions.
    #[must_use]
    pub fn with_sub_layers(mut self, sub_layers: Vec<SubLayer>) -> Self {
        self.sub_layers = sub_layers;
        self
    }

    /// Compartment name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Layer this compartment belongs to.
    #[must_use]
    pub fn layer(&self) -> Layer {
        self.layer
    }

    /// Holds in this compartment.
    #[must_use]
    pub fn holds(&self) -> &BTreeMap<String, Hold> {
        &self.holds
    }

    /// Dependency edges between holds.
    #[must_use]
    pub fn edges(&self) -> &[(String, String)] {
        &self.edges
    }

    /// Sub-layer subdivisions (empty if not set).
    #[must_use]
    pub fn sub_layers(&self) -> &[SubLayer] {
        &self.sub_layers
    }

    /// Number of holds.
    #[must_use]
    pub fn hold_count(&self) -> usize {
        self.holds.len()
    }

    /// Total crate count across all holds.
    #[must_use]
    pub fn crate_count(&self) -> usize {
        self.holds.values().map(Hold::member_count).sum()
    }

    /// Topological ordering of hold names (leaves first).
    ///
    /// # Errors
    /// Returns [`TopologyError::CycleDetected`] if the graph is cyclic
    /// (should not happen after construction validation, but defended in depth).
    pub fn topological_order(&self) -> Result<Vec<String>, TopologyError> {
        topological_sort(&self.holds, &self.edges)
    }
}

// CALIBRATION: Compartment grounds to σ (Sequence) dominant because it
// represents ordered layer structure — holds are arranged in dependency
// order within a layer. Supporting primitives:
// ∂ (Boundary) — layer boundary separates this compartment from others;
// μ (Mapping) — holds map to crate sets;
// ς (State) — layer assignment is a discrete FSM mode.
// Confidence 0.60: moderate-high — ordering is the primary semantic but
// the containment (∂) role is also significant.
impl GroundsTo for Compartment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.6)
        .with_state_mode(StateMode::Modal)
    }

    fn state_mode() -> Option<StateMode> {
        // CALIBRATION: Layer assignment is a discrete mode (Foundation/Domain/Orchestration/Service)
        Some(StateMode::Modal)
    }
}

// ============================================================================
// Stack — connected Foundation→Service capability path
// ============================================================================

/// A connected path from Foundation to Service through the layer stack.
///
/// Stacks model capability paths: how a service-layer crate traces its
/// dependency chain back through orchestration and domain to foundation.
/// Paths shorter than 4 layers are accepted (not every path traverses
/// all layers).
///
/// ## Invariants
/// - Must contain at least one segment.
/// - Segments must be ordered by increasing layer depth.
#[derive(Debug, Clone)]
pub struct Stack {
    segments: Vec<StackSegment>,
}

impl Stack {
    /// Construct a stack, validating connectivity and layer ordering.
    ///
    /// # Errors
    /// - [`TopologyError::EmptyStack`] if `segments` is empty.
    /// - [`TopologyError::SegmentOrderViolation`] if segments are not ordered
    ///   by increasing layer depth.
    pub fn new(segments: Vec<StackSegment>) -> Result<Self, TopologyError> {
        if segments.is_empty() {
            return Err(TopologyError::EmptyStack);
        }

        // Validate ordering: each segment's layer depth must be >= previous
        let mut prev_depth: Option<u8> = None;
        for seg in &segments {
            let depth = seg.layer.depth();
            if let Some(pd) = prev_depth {
                if depth < pd {
                    return Err(TopologyError::SegmentOrderViolation {
                        detail: format!(
                            "layer {} (depth {}) appears after layer depth {}",
                            seg.layer, depth, pd
                        ),
                    });
                }
            }
            prev_depth = Some(depth);
        }

        Ok(Self { segments })
    }

    /// The path segments in order.
    #[must_use]
    pub fn segments(&self) -> &[StackSegment] {
        &self.segments
    }

    /// Number of layers traversed.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// First segment (origin, typically Foundation).
    #[must_use]
    pub fn origin(&self) -> Option<&StackSegment> {
        self.segments.first()
    }

    /// Last segment (terminus, typically Service).
    #[must_use]
    pub fn terminus(&self) -> Option<&StackSegment> {
        self.segments.last()
    }

    /// Whether this stack passes through the given layer.
    #[must_use]
    pub fn spans_layer(&self, layer: Layer) -> bool {
        self.segments.iter().any(|s| s.layer == layer)
    }

    /// Whether this stack spans the full Foundation→Service range.
    #[must_use]
    pub fn is_full_span(&self) -> bool {
        let has_foundation = self.segments.iter().any(|s| s.layer == Layer::Foundation);
        let has_service = self.segments.iter().any(|s| s.layer == Layer::Service);
        has_foundation && has_service
    }
}

// CALIBRATION: Stack grounds to → (Causality) dominant because it
// represents the cause-to-effect chain: a service-layer binary exists
// BECAUSE of the foundation primitives it ultimately depends on.
// Supporting primitives:
// σ (Sequence) — the path is ordered Foundation→Service;
// μ (Mapping) — each segment maps a layer to a concrete crate;
// ∃ (Existence) — the path must actually exist in the dependency graph.
// Confidence 0.65: moderate-high — causality is the primary semantic
// (dependency implies causation) but the ordered traversal (σ) is
// also structurally important.
impl GroundsTo for Stack {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.65)
    }
}

// ============================================================================
// Bay — complete crate-to-hold allocation
// ============================================================================

/// Complete allocation of every workspace crate to exactly one hold.
///
/// The Bay is the top-level container: it guarantees that every crate
/// belongs to exactly one hold (no orphans, no duplicates). The
/// `reconcile` method produces a typed diff when comparing two Bay
/// snapshots.
///
/// ## Invariants
/// - Must contain at least one hold.
/// - Every crate appears in exactly one hold (enforced at construction).
#[derive(Debug, Clone)]
pub struct Bay {
    holds: BTreeMap<String, Hold>,
    crate_index: BTreeMap<String, String>,
}

impl Bay {
    /// Construct a bay, validating exclusive crate allocation.
    ///
    /// # Errors
    /// - [`TopologyError::EmptyBay`] if `holds` is empty.
    /// - [`TopologyError::DuplicateCrate`] if any crate appears in more
    ///   than one hold.
    pub fn new(holds: Vec<Hold>) -> Result<Self, TopologyError> {
        if holds.is_empty() {
            return Err(TopologyError::EmptyBay);
        }

        let mut crate_index: BTreeMap<String, String> = BTreeMap::new();
        let mut hold_map: BTreeMap<String, Hold> = BTreeMap::new();

        for hold in holds {
            for crate_name in hold.members() {
                if let Some(existing_hold) = crate_index.get(crate_name) {
                    return Err(TopologyError::DuplicateCrate {
                        crate_name: crate_name.clone(),
                        hold_a: existing_hold.clone(),
                        hold_b: hold.name().to_owned(),
                    });
                }
                crate_index.insert(crate_name.clone(), hold.name().to_owned());
            }
            hold_map.insert(hold.name().to_owned(), hold);
        }

        Ok(Self {
            holds: hold_map,
            crate_index,
        })
    }

    /// All holds.
    #[must_use]
    pub fn holds(&self) -> &BTreeMap<String, Hold> {
        &self.holds
    }

    /// Total crate count.
    #[must_use]
    pub fn crate_count(&self) -> usize {
        self.crate_index.len()
    }

    /// Number of holds.
    #[must_use]
    pub fn hold_count(&self) -> usize {
        self.holds.len()
    }

    /// Find which hold a crate belongs to.
    #[must_use]
    pub fn find_crate(&self, crate_name: &str) -> Option<&str> {
        self.crate_index.get(crate_name).map(String::as_str)
    }

    /// Look up a hold by name.
    #[must_use]
    pub fn find_hold(&self, hold_name: &str) -> Option<&Hold> {
        self.holds.get(hold_name)
    }

    /// Produce a typed diff between `self` and `other`.
    #[must_use]
    pub fn reconcile(&self, other: &Bay) -> BayDiff {
        let mut diff = BayDiff::default();

        // Holds added/removed
        for name in other.holds.keys() {
            if !self.holds.contains_key(name) {
                diff.holds_added.push(name.clone());
            }
        }
        for name in self.holds.keys() {
            if !other.holds.contains_key(name) {
                diff.holds_removed.push(name.clone());
            }
        }

        // Crates added/removed/moved
        for (crate_name, other_hold) in &other.crate_index {
            match self.crate_index.get(crate_name) {
                None => diff.crates_added.push(crate_name.clone()),
                Some(self_hold) if self_hold != other_hold => {
                    diff.crates_moved.push((
                        crate_name.clone(),
                        self_hold.clone(),
                        other_hold.clone(),
                    ));
                }
                Some(_) => {} // Same hold, no diff
            }
        }
        for crate_name in self.crate_index.keys() {
            if !other.crate_index.contains_key(crate_name) {
                diff.crates_removed.push(crate_name.clone());
            }
        }

        diff
    }
}

// CALIBRATION: Bay grounds to × (Product) dominant because it is the
// conjunctive composition of all holds — every hold must be present,
// and every crate must be allocated. This mirrors how × (Product) is
// the structural primitive for tuples and structs (ALL fields present).
// Supporting primitives:
// ∂ (Boundary) — the bay defines the workspace boundary;
// N (Quantity) — countable total of crates and holds;
// ∃ (Existence) — completeness guarantee (no orphan crates).
// Confidence 0.60: moderate-high — conjunction is the primary semantic
// but the completeness guarantee (∃) and boundary (∂) roles are
// also significant at this top-level container.
impl GroundsTo for Bay {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Product, 0.6)
    }
}

// ============================================================================
// Internal: Kahn's Algorithm for Cycle Detection
// ============================================================================

/// Validate that the hold-level DAG is acyclic (Kahn's algorithm).
fn validate_acyclic(
    holds: &BTreeMap<String, Hold>,
    edges: &[(String, String)],
) -> Result<(), TopologyError> {
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for name in holds.keys() {
        in_degree.insert(name.as_str(), 0);
        adj.insert(name.as_str(), Vec::new());
    }

    for (from, to) in edges {
        if let Some(neighbors) = adj.get_mut(from.as_str()) {
            neighbors.push(to.as_str());
        }
        if let Some(deg) = in_degree.get_mut(to.as_str()) {
            *deg = deg.saturating_add(1);
        }
    }

    // BFS: start with all zero-in-degree nodes
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, deg)| *deg == &0)
        .map(|(name, _)| *name)
        .collect();

    let mut processed: usize = 0;

    while let Some(node) = queue.pop() {
        processed = processed.saturating_add(1);
        let neighbors: Vec<&str> = adj.get(node).cloned().unwrap_or_default();
        for neighbor in neighbors {
            if let Some(deg) = in_degree.get_mut(neighbor) {
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push(neighbor);
                }
            }
        }
    }

    if processed == holds.len() {
        Ok(())
    } else {
        // Find a cycle member (guaranteed to exist since processed < total)
        let mut cycle_hold = String::new();
        for (name, deg) in &in_degree {
            if *deg > 0 {
                cycle_hold = (*name).to_owned();
                break;
            }
        }
        Err(TopologyError::CycleDetected { hold: cycle_hold })
    }
}

/// Topological sort returning hold names in dependency order (leaves first).
fn topological_sort(
    holds: &BTreeMap<String, Hold>,
    edges: &[(String, String)],
) -> Result<Vec<String>, TopologyError> {
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut adj: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for name in holds.keys() {
        in_degree.insert(name.as_str(), 0);
        adj.insert(name.as_str(), Vec::new());
    }

    for (from, to) in edges {
        if let Some(neighbors) = adj.get_mut(from.as_str()) {
            neighbors.push(to.as_str());
        }
        if let Some(deg) = in_degree.get_mut(to.as_str()) {
            *deg = deg.saturating_add(1);
        }
    }

    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, deg)| *deg == &0)
        .map(|(name, _)| *name)
        .collect();

    let mut result: Vec<String> = Vec::new();

    while let Some(node) = queue.pop() {
        result.push(node.to_owned());
        let neighbors: Vec<&str> = adj.get(node).cloned().unwrap_or_default();
        for neighbor in neighbors {
            if let Some(deg) = in_degree.get_mut(neighbor) {
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push(neighbor);
                }
            }
        }
    }

    if result.len() == holds.len() {
        // Reverse: Kahn's gives dependents-first, we want dependencies-first
        result.reverse();
        Ok(result)
    } else {
        let mut cycle_hold = String::new();
        for (name, deg) in &in_degree {
            if *deg > 0 {
                cycle_hold = (*name).to_owned();
                break;
            }
        }
        Err(TopologyError::CycleDetected { hold: cycle_hold })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "Tests use unwrap/assert/unreachable for validation"
)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::Tier;

    fn test_governance() -> GovernancePolicy {
        GovernancePolicy {
            owner: "test-team".to_owned(),
            review_required: true,
        }
    }

    fn make_hold(name: &str, crates: &[&str]) -> Hold {
        let members: BTreeSet<String> = crates.iter().map(|s| (*s).to_owned()).collect();
        Hold::new(name, members, test_governance()).unwrap_or_else(|_| unreachable!())
    }

    // ── Hold tests ─────────────────────────────────────────────────────

    #[test]
    fn hold_construction_valid() {
        let members: BTreeSet<String> = ["crate-a", "crate-b"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let hold = Hold::new("test-hold", members, test_governance());
        assert!(hold.is_ok());
        let h = hold.unwrap_or_else(|_| unreachable!());
        assert_eq!(h.name(), "test-hold");
        assert_eq!(h.member_count(), 2);
        assert!(h.contains("crate-a"));
        assert!(!h.contains("crate-c"));
        assert!(!h.has_cycles());
    }

    #[test]
    fn hold_rejects_empty() {
        let result = Hold::new("empty", BTreeSet::new(), test_governance());
        assert!(result.is_err());
        match result {
            Err(TopologyError::EmptyHold { name }) => assert_eq!(name, "empty"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn hold_cycle_partners() {
        let h = make_hold("cyclic", &["a", "b", "c"])
            .with_cycle_partners(vec![("a".to_owned(), "b".to_owned())]);
        assert!(h.has_cycles());
        assert_eq!(h.cycle_partners().len(), 1);
    }

    #[test]
    fn hold_grounding() {
        assert_eq!(Hold::dominant_primitive(), Some(LexPrimitiva::Boundary));
        assert!(!Hold::is_pure_primitive());
        assert_eq!(Hold::tier(), Tier::T2Composite);
        assert_eq!(Hold::state_mode(), Some(StateMode::Modal));
    }

    // ── Compartment tests ──────────────────────────────────────────────

    #[test]
    fn compartment_construction_valid() {
        let h1 = make_hold("core", &["primitives", "id"]);
        let h2 = make_hold("stem", &["stem-math", "stem-phys"]);
        let mut holds = BTreeMap::new();
        holds.insert("core".to_owned(), h1);
        holds.insert("stem".to_owned(), h2);

        let edges = vec![("stem".to_owned(), "core".to_owned())];
        let comp = Compartment::new("foundation", Layer::Foundation, holds, edges);
        assert!(comp.is_ok());
        let c = comp.unwrap_or_else(|_| unreachable!());
        assert_eq!(c.name(), "foundation");
        assert_eq!(c.layer(), Layer::Foundation);
        assert_eq!(c.hold_count(), 2);
        assert_eq!(c.crate_count(), 4);
    }

    #[test]
    fn compartment_rejects_empty() {
        let result = Compartment::new("empty", Layer::Domain, BTreeMap::new(), vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn compartment_rejects_missing_edge_endpoint() {
        let h1 = make_hold("core", &["a"]);
        let mut holds = BTreeMap::new();
        holds.insert("core".to_owned(), h1);

        let edges = vec![("core".to_owned(), "nonexistent".to_owned())];
        let result = Compartment::new("bad", Layer::Foundation, holds, edges);
        assert!(result.is_err());
        match result {
            Err(TopologyError::MissingHold { name }) => assert_eq!(name, "nonexistent"),
            _ => unreachable!(),
        }
    }

    #[test]
    fn compartment_rejects_cycle() {
        let h1 = make_hold("a", &["crate-a"]);
        let h2 = make_hold("b", &["crate-b"]);
        let mut holds = BTreeMap::new();
        holds.insert("a".to_owned(), h1);
        holds.insert("b".to_owned(), h2);

        let edges = vec![
            ("a".to_owned(), "b".to_owned()),
            ("b".to_owned(), "a".to_owned()),
        ];
        let result = Compartment::new("cyclic", Layer::Domain, holds, edges);
        assert!(result.is_err());
        match result {
            Err(TopologyError::CycleDetected { .. }) => {}
            _ => unreachable!(),
        }
    }

    #[test]
    fn compartment_topological_order() {
        let h1 = make_hold("leaf", &["a"]);
        let h2 = make_hold("mid", &["b"]);
        let h3 = make_hold("root", &["c"]);
        let mut holds = BTreeMap::new();
        holds.insert("leaf".to_owned(), h1);
        holds.insert("mid".to_owned(), h2);
        holds.insert("root".to_owned(), h3);

        // root → mid → leaf
        let edges = vec![
            ("root".to_owned(), "mid".to_owned()),
            ("mid".to_owned(), "leaf".to_owned()),
        ];
        let comp = Compartment::new("test", Layer::Foundation, holds, edges)
            .unwrap_or_else(|_| unreachable!());
        let order = comp.topological_order().unwrap_or_else(|_| unreachable!());
        // Leaf should appear before mid, mid before root
        let leaf_pos = order.iter().position(|n| n == "leaf");
        let mid_pos = order.iter().position(|n| n == "mid");
        let root_pos = order.iter().position(|n| n == "root");
        assert!(leaf_pos < mid_pos);
        assert!(mid_pos < root_pos);
    }

    #[test]
    fn compartment_sub_layers() {
        let h1 = make_hold("core", &["a"]);
        let mut holds = BTreeMap::new();
        holds.insert("core".to_owned(), h1);

        let comp = Compartment::new("test", Layer::Service, holds, vec![])
            .unwrap_or_else(|_| unreachable!())
            .with_sub_layers(vec![SubLayer {
                name: "api-layer".to_owned(),
                depth: 0,
                holds: vec!["core".to_owned()],
            }]);
        assert_eq!(comp.sub_layers().len(), 1);
    }

    #[test]
    fn compartment_grounding() {
        assert_eq!(
            Compartment::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(Compartment::tier(), Tier::T2Composite);
        assert_eq!(Compartment::state_mode(), Some(StateMode::Modal));
    }

    // ── Stack tests ────────────────────────────────────────────────────

    #[test]
    fn stack_full_span() {
        let segments = vec![
            StackSegment {
                layer: Layer::Foundation,
                hold_name: "core".to_owned(),
                crate_name: "primitives".to_owned(),
            },
            StackSegment {
                layer: Layer::Domain,
                hold_name: "pv".to_owned(),
                crate_name: "vigilance".to_owned(),
            },
            StackSegment {
                layer: Layer::Orchestration,
                hold_name: "orch".to_owned(),
                crate_name: "brain".to_owned(),
            },
            StackSegment {
                layer: Layer::Service,
                hold_name: "svc".to_owned(),
                crate_name: "mcp".to_owned(),
            },
        ];
        let stack = Stack::new(segments);
        assert!(stack.is_ok());
        let s = stack.unwrap_or_else(|_| unreachable!());
        assert_eq!(s.depth(), 4);
        assert!(s.is_full_span());
        assert!(s.spans_layer(Layer::Domain));
        assert_eq!(s.origin().map(|seg| seg.layer), Some(Layer::Foundation));
        assert_eq!(s.terminus().map(|seg| seg.layer), Some(Layer::Service));
    }

    #[test]
    fn stack_short_path() {
        let segments = vec![
            StackSegment {
                layer: Layer::Foundation,
                hold_name: "core".to_owned(),
                crate_name: "id".to_owned(),
            },
            StackSegment {
                layer: Layer::Service,
                hold_name: "svc".to_owned(),
                crate_name: "api".to_owned(),
            },
        ];
        let stack = Stack::new(segments).unwrap_or_else(|_| unreachable!());
        assert_eq!(stack.depth(), 2);
        assert!(stack.is_full_span());
        assert!(!stack.spans_layer(Layer::Orchestration));
    }

    #[test]
    fn stack_rejects_empty() {
        assert!(Stack::new(vec![]).is_err());
    }

    #[test]
    fn stack_rejects_wrong_order() {
        let segments = vec![
            StackSegment {
                layer: Layer::Service,
                hold_name: "svc".to_owned(),
                crate_name: "mcp".to_owned(),
            },
            StackSegment {
                layer: Layer::Foundation,
                hold_name: "core".to_owned(),
                crate_name: "primitives".to_owned(),
            },
        ];
        let result = Stack::new(segments);
        assert!(result.is_err());
    }

    #[test]
    fn stack_grounding() {
        assert_eq!(Stack::dominant_primitive(), Some(LexPrimitiva::Causality));
        assert_eq!(Stack::tier(), Tier::T2Composite);
        assert_eq!(Stack::state_mode(), None);
    }

    // ── Bay tests ──────────────────────────────────────────────────────

    #[test]
    fn bay_construction_valid() {
        let h1 = make_hold("core", &["primitives", "id"]);
        let h2 = make_hold("pv", &["vigilance", "pvos"]);
        let bay = Bay::new(vec![h1, h2]);
        assert!(bay.is_ok());
        let b = bay.unwrap_or_else(|_| unreachable!());
        assert_eq!(b.crate_count(), 4);
        assert_eq!(b.hold_count(), 2);
        assert_eq!(b.find_crate("primitives"), Some("core"));
        assert_eq!(b.find_crate("vigilance"), Some("pv"));
        assert_eq!(b.find_crate("nonexistent"), None);
        assert!(b.find_hold("core").is_some());
        assert!(b.find_hold("missing").is_none());
    }

    #[test]
    fn bay_rejects_empty() {
        assert!(Bay::new(vec![]).is_err());
    }

    #[test]
    fn bay_rejects_duplicate_crate() {
        let h1 = make_hold("hold-a", &["shared-crate", "unique-a"]);
        let h2 = make_hold("hold-b", &["shared-crate", "unique-b"]);
        let result = Bay::new(vec![h1, h2]);
        assert!(result.is_err());
        match result {
            Err(TopologyError::DuplicateCrate {
                crate_name,
                hold_a,
                hold_b,
            }) => {
                assert_eq!(crate_name, "shared-crate");
                assert_eq!(hold_a, "hold-a");
                assert_eq!(hold_b, "hold-b");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn bay_reconcile_identical() {
        let h1 = make_hold("core", &["a", "b"]);
        let h2 = make_hold("core", &["a", "b"]);
        let bay1 = Bay::new(vec![h1]).unwrap_or_else(|_| unreachable!());
        let bay2 = Bay::new(vec![h2]).unwrap_or_else(|_| unreachable!());
        let diff = bay1.reconcile(&bay2);
        assert!(diff.is_empty());
    }

    #[test]
    fn bay_reconcile_added_removed() {
        let h1 = make_hold("old", &["a"]);
        let h2 = make_hold("new", &["b"]);
        let bay1 = Bay::new(vec![h1]).unwrap_or_else(|_| unreachable!());
        let bay2 = Bay::new(vec![h2]).unwrap_or_else(|_| unreachable!());
        let diff = bay1.reconcile(&bay2);
        assert!(!diff.is_empty());
        assert_eq!(diff.holds_added, vec!["new"]);
        assert_eq!(diff.holds_removed, vec!["old"]);
        assert_eq!(diff.crates_added, vec!["b"]);
        assert_eq!(diff.crates_removed, vec!["a"]);
    }

    #[test]
    fn bay_reconcile_moved_crate() {
        let h1a = make_hold("hold-a", &["migrating", "stable-a"]);
        let h1b = make_hold("hold-b", &["stable-b"]);
        let h2a = make_hold("hold-a", &["stable-a"]);
        let h2b = make_hold("hold-b", &["migrating", "stable-b"]);

        let bay1 = Bay::new(vec![h1a, h1b]).unwrap_or_else(|_| unreachable!());
        let bay2 = Bay::new(vec![h2a, h2b]).unwrap_or_else(|_| unreachable!());
        let diff = bay1.reconcile(&bay2);
        assert_eq!(diff.crates_moved.len(), 1);
        let (crate_name, from, to) = diff.crates_moved.first().unwrap_or_else(|| unreachable!());
        assert_eq!(crate_name, "migrating");
        assert_eq!(from, "hold-a");
        assert_eq!(to, "hold-b");
    }

    #[test]
    fn bay_grounding() {
        assert_eq!(Bay::dominant_primitive(), Some(LexPrimitiva::Product));
        assert_eq!(Bay::tier(), Tier::T2Composite);
        assert_eq!(Bay::state_mode(), None);
    }

    // ── Layer tests ────────────────────────────────────────────────────

    #[test]
    fn layer_ordering() {
        assert!(Layer::Foundation < Layer::Domain);
        assert!(Layer::Domain < Layer::Orchestration);
        assert!(Layer::Orchestration < Layer::Service);
    }

    #[test]
    fn layer_depth() {
        assert_eq!(Layer::Foundation.depth(), 0);
        assert_eq!(Layer::Service.depth(), 3);
    }

    // ── BayDiff tests ──────────────────────────────────────────────────

    #[test]
    fn bay_diff_default_is_empty() {
        let diff = BayDiff::default();
        assert!(diff.is_empty());
    }

    // ── Error Display tests ────────────────────────────────────────────

    #[test]
    fn error_display() {
        let e = TopologyError::EmptyHold {
            name: "test".to_owned(),
        };
        assert!(format!("{e}").contains("test"));

        let e = TopologyError::CycleDetected {
            hold: "cyclic".to_owned(),
        };
        assert!(format!("{e}").contains("cyclic"));
    }
}
