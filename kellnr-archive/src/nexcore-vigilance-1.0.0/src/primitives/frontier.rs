//! # Frontier T2-P Primitives
//!
//! Eight types completing T1 surface coverage. Each grounds a previously
//! DARK or FRONTIER T1 primitive to its first T2-P representation.
//!
//! | Type | T1 Grounded | Symbol | Composition |
//! |------|-------------|--------|-------------|
//! | [`AuditTrail`] | Persistence | π | π σ ∃ |
//! | [`AbsenceMarker`] | Void | ∅ | ∅ κ ∂ |
//! | [`Pipeline`] | Sequence | σ | σ N ∂ |
//! | [`ConsumptionMark`] | Irreversibility | ∝ | ∝ N → |
//! | [`EntityStatus`] | Existence | ∃ | ∃ Σ κ |
//! | [`ResourcePath`] | Location | λ | λ σ ∃ |
//! | [`RecursionBound`] | Recursion | ρ | ρ N ∂ |
//! | [`RecordStructure`] | Product | × | × ∃ N |

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// π (Persistence) → AuditTrail
// ============================================================================

/// Durable record of events across time.
///
/// Grounds π (Persistence): the first T2-P type representing
/// continuity of state beyond process lifetime.
///
/// # Domain Mappings
/// - PV: ICSR case record chain (creation → amendment → closure)
/// - Brain: artifact versioning (.resolved.N snapshots)
/// - Guardian: homeostasis iteration history
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AuditTrail {
    /// Number of recorded entries in the trail
    pub entry_count: u64,
    /// Whether the trail has been sealed (no further mutations)
    pub sealed: bool,
}

impl AuditTrail {
    /// Creates a new empty audit trail.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entry_count: 0,
            sealed: false,
        }
    }

    /// Creates an audit trail with existing entries.
    #[must_use]
    pub const fn with_entries(count: u64) -> Self {
        Self {
            entry_count: count,
            sealed: false,
        }
    }

    /// Records an event, incrementing the entry count.
    /// Returns `None` if the trail is sealed.
    #[must_use]
    pub const fn record(self) -> Option<Self> {
        if self.sealed {
            return None;
        }
        Some(Self {
            entry_count: self.entry_count + 1,
            sealed: self.sealed,
        })
    }

    /// Seals the trail, preventing further mutations.
    #[must_use]
    pub const fn seal(self) -> Self {
        Self {
            entry_count: self.entry_count,
            sealed: true,
        }
    }

    /// Returns true if the trail has any entries.
    #[must_use]
    pub const fn has_entries(&self) -> bool {
        self.entry_count > 0
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AuditTrail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seal = if self.sealed { " [sealed]" } else { "" };
        write!(f, "AuditTrail({} entries{})", self.entry_count, seal)
    }
}

// ============================================================================
// ∅ (Void) → AbsenceMarker
// ============================================================================

/// Meaningful absence of expected data.
///
/// Grounds ∅ (Void): the first T2-P type representing
/// explicit nothing — absence that carries signal.
///
/// # Domain Mappings
/// - PV: "No cases reported" for a drug-event pair IS signal data
/// - Signal detection: zero cell in 2x2 contingency table
/// - FAERS: drug with no adverse events in database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbsenceMarker {
    /// Expected data not found — this absence is informative
    NotReported,
    /// Data exists but below detection threshold
    BelowThreshold,
    /// Data explicitly marked as absent by source
    ExplicitlyAbsent,
}

impl AbsenceMarker {
    /// Returns true if this represents informative absence.
    #[must_use]
    pub const fn is_informative(&self) -> bool {
        matches!(self, Self::NotReported | Self::ExplicitlyAbsent)
    }

    /// Returns true if this is a detection boundary issue.
    #[must_use]
    pub const fn is_threshold_limited(&self) -> bool {
        matches!(self, Self::BelowThreshold)
    }
}

impl fmt::Display for AbsenceMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReported => write!(f, "∅:not_reported"),
            Self::BelowThreshold => write!(f, "∅:below_threshold"),
            Self::ExplicitlyAbsent => write!(f, "∅:explicitly_absent"),
        }
    }
}

// ============================================================================
// σ (Sequence) → Pipeline
// ============================================================================

/// Ordered phases of execution with bounded progress.
///
/// Grounds σ (Sequence): the first T2-P type representing
/// ordered succession as a first-class value.
///
/// # Domain Mappings
/// - Signal: 12-stage detection pipeline (ingest→...→report)
/// - CCP: 5-phase care process (Collect→...→FollowUp)
/// - CTVP: 5-phase validation (Preclinical→...→Surveillance)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pipeline {
    /// Current phase index (0-based)
    pub current_phase: u32,
    /// Total number of phases
    pub total_phases: u32,
}

impl Pipeline {
    /// Creates a new pipeline with the given number of phases.
    #[must_use]
    pub const fn new(total_phases: u32) -> Self {
        Self {
            current_phase: 0,
            total_phases,
        }
    }

    /// Advances to the next phase. Returns `None` if already at the end.
    #[must_use]
    pub const fn advance(self) -> Option<Self> {
        if self.current_phase >= self.total_phases.saturating_sub(1) {
            return None;
        }
        Some(Self {
            current_phase: self.current_phase + 1,
            total_phases: self.total_phases,
        })
    }

    /// Returns true if the pipeline has completed all phases.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.total_phases == 0 || self.current_phase >= self.total_phases.saturating_sub(1)
    }

    /// Returns progress as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn progress(&self) -> f64 {
        if self.total_phases == 0 {
            return 1.0;
        }
        self.current_phase as f64 / (self.total_phases.saturating_sub(1).max(1)) as f64
    }
}

impl fmt::Display for Pipeline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Pipeline({}/{})",
            self.current_phase + 1,
            self.total_phases
        )
    }
}

// ============================================================================
// ∝ (Irreversibility) → ConsumptionMark
// ============================================================================

/// Irreversible resource consumption marker.
///
/// Grounds ∝ (Irreversibility): the first T2-P type representing
/// one-way state transitions that cannot be undone.
///
/// # Domain Mappings
/// - Energy: token-as-ATP consumption (tADP formation)
/// - PV: case closure (cannot un-close without reopening)
/// - Guardian: actuator action taken (cannot un-send alert)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConsumptionMark {
    /// Amount consumed (always positive, monotonically increasing)
    pub consumed: f64,
    /// Total capacity before depletion
    pub capacity: f64,
}

impl ConsumptionMark {
    /// Creates a new consumption marker with given capacity.
    #[must_use]
    pub fn new(capacity: f64) -> Self {
        Self {
            consumed: 0.0,
            capacity: capacity.max(0.0),
        }
    }

    /// Consumes an amount. Returns `None` if insufficient capacity.
    #[must_use]
    pub fn consume(self, amount: f64) -> Option<Self> {
        let amount = amount.max(0.0);
        if self.consumed + amount > self.capacity {
            return None;
        }
        Some(Self {
            consumed: self.consumed + amount,
            capacity: self.capacity,
        })
    }

    /// Returns remaining capacity.
    #[must_use]
    pub fn remaining(&self) -> f64 {
        (self.capacity - self.consumed).max(0.0)
    }

    /// Returns true if fully depleted.
    #[must_use]
    pub fn is_depleted(&self) -> bool {
        self.consumed >= self.capacity
    }

    /// Returns consumption fraction (0.0 to 1.0).
    #[must_use]
    pub fn fraction(&self) -> f64 {
        if self.capacity <= 0.0 {
            return 1.0;
        }
        (self.consumed / self.capacity).clamp(0.0, 1.0)
    }
}

impl fmt::Display for ConsumptionMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "∝({:.1}/{:.1} consumed)", self.consumed, self.capacity)
    }
}

// ============================================================================
// ∃ (Existence) → EntityStatus
// ============================================================================

/// Lifecycle presence/absence state of a domain entity.
///
/// Grounds ∃ (Existence): the first T2-P type representing
/// whether something IS, and in what mode of being.
///
/// # Domain Mappings
/// - PV: drug lifecycle (Active, Withdrawn, Suspended)
/// - Skills: skill status (Registered, Validated, Deprecated)
/// - MCP: tool availability (Available, Disabled, Removed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityStatus {
    /// Entity exists and is active
    Active,
    /// Entity exists but is temporarily inactive
    Suspended,
    /// Entity has been permanently removed
    Withdrawn,
    /// Entity existence is unknown or unverified
    Unknown,
}

impl EntityStatus {
    /// Returns true if the entity is currently accessible.
    #[must_use]
    pub const fn is_accessible(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns true if the entity still exists in some form.
    #[must_use]
    pub const fn exists(&self) -> bool {
        !matches!(self, Self::Withdrawn)
    }

    /// Returns true if the status is definitive (not unknown).
    #[must_use]
    pub const fn is_definitive(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

impl fmt::Display for EntityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "∃:active"),
            Self::Suspended => write!(f, "∃:suspended"),
            Self::Withdrawn => write!(f, "∃:withdrawn"),
            Self::Unknown => write!(f, "∃:unknown"),
        }
    }
}

// ============================================================================
// λ (Location) → ResourcePath
// ============================================================================

/// Positional addressing of a resource within a hierarchy.
///
/// Grounds λ (Location): the first T2-P type representing
/// positional context as a first-class value.
///
/// # Domain Mappings
/// - Brain: artifact addressing (session/artifact_name.resolved.N)
/// - MCP: tool routing (server__tool_name)
/// - Skills: skill lookup path (category/skill_name)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourcePath {
    /// Path segments from root to resource
    segments: Vec<String>,
}

impl ResourcePath {
    /// Creates a new resource path from segments.
    #[must_use]
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Creates a resource path from a slash-separated string.
    #[must_use]
    pub fn from_str_path(path: &str) -> Self {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        Self { segments }
    }

    /// Returns the depth (number of segments).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.segments.len()
    }

    /// Returns the leaf (final segment), if any.
    #[must_use]
    pub fn leaf(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }

    /// Returns the parent path (all segments except last).
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            return None;
        }
        Some(Self {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        })
    }

    /// Appends a segment to the path.
    #[must_use]
    pub fn join(mut self, segment: &str) -> Self {
        self.segments.push(segment.to_string());
        self
    }

    /// Returns the segments as a slice.
    #[must_use]
    pub fn segments(&self) -> &[String] {
        &self.segments
    }
}

impl fmt::Display for ResourcePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "λ:{}", self.segments.join("/"))
    }
}

// ============================================================================
// ρ (Recursion) → RecursionBound
// ============================================================================

/// Depth limit on self-referential traversal.
///
/// Grounds ρ (Recursion): the first T2-P type representing
/// controlled self-reference with termination guarantee.
///
/// # Domain Mappings
/// - DTree: maximum tree depth during training/prediction
/// - Renderer: DOM nesting depth limit
/// - PVDSL: bytecode execution stack depth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecursionBound {
    /// Current depth in the recursive structure
    pub current_depth: u32,
    /// Maximum allowed depth before forced termination
    pub max_depth: u32,
}

impl RecursionBound {
    /// Creates a new recursion bound with max depth.
    #[must_use]
    pub const fn new(max_depth: u32) -> Self {
        Self {
            current_depth: 0,
            max_depth,
        }
    }

    /// Descends one level deeper. Returns `None` if at max depth.
    #[must_use]
    pub const fn descend(self) -> Option<Self> {
        if self.current_depth >= self.max_depth {
            return None;
        }
        Some(Self {
            current_depth: self.current_depth + 1,
            max_depth: self.max_depth,
        })
    }

    /// Returns true if further recursion is allowed.
    #[must_use]
    pub const fn can_recurse(&self) -> bool {
        self.current_depth < self.max_depth
    }

    /// Returns remaining depth budget.
    #[must_use]
    pub const fn remaining(&self) -> u32 {
        self.max_depth.saturating_sub(self.current_depth)
    }

    /// Returns depth utilization as a fraction (0.0 to 1.0).
    #[must_use]
    pub fn utilization(&self) -> f64 {
        if self.max_depth == 0 {
            return 1.0;
        }
        self.current_depth as f64 / self.max_depth as f64
    }
}

impl fmt::Display for RecursionBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ρ({}/{} depth)", self.current_depth, self.max_depth)
    }
}

// ============================================================================
// × (Product) → RecordStructure
// ============================================================================

/// Named fields combined into a single conjunctive value.
///
/// Grounds × (Product): the first T2-P type representing
/// conjunctive combination — all fields must coexist.
///
/// # Domain Mappings
/// - PV: Drug-event pair as structured record (drug × event)
/// - ICSR: Case report combining reporter, patient, drug, event fields
/// - Rust: `struct`, tuples `(A, B)`, product types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordStructure {
    /// Number of fields in the record
    pub field_count: u32,
    /// Whether the record is sealed (no new fields)
    pub sealed: bool,
}

impl RecordStructure {
    /// Create a new record structure with the given field count.
    #[must_use]
    pub const fn new(field_count: u32) -> Self {
        Self {
            field_count,
            sealed: false,
        }
    }

    /// Create a pair (2-field product).
    #[must_use]
    pub const fn pair() -> Self {
        Self::new(2)
    }

    /// Create a triple (3-field product).
    #[must_use]
    pub const fn triple() -> Self {
        Self::new(3)
    }

    /// Seal the record, preventing structural modification.
    #[must_use]
    pub const fn seal(self) -> Self {
        Self {
            field_count: self.field_count,
            sealed: true,
        }
    }

    /// Returns true if this is a unit product (0 or 1 fields).
    #[must_use]
    pub const fn is_unit(&self) -> bool {
        self.field_count <= 1
    }

    /// Returns the arity of this product (same as field_count).
    #[must_use]
    pub const fn arity(&self) -> u32 {
        self.field_count
    }
}

impl Default for RecordStructure {
    fn default() -> Self {
        Self::pair()
    }
}

impl fmt::Display for RecordStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seal = if self.sealed { " [sealed]" } else { "" };
        write!(f, "×({} fields{})", self.field_count, seal)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- AuditTrail (π) --

    #[test]
    fn test_audit_trail_new() {
        let trail = AuditTrail::new();
        assert_eq!(trail.entry_count, 0);
        assert!(!trail.sealed);
        assert!(!trail.has_entries());
    }

    #[test]
    fn test_audit_trail_record() {
        let trail = AuditTrail::new();
        let trail = trail.record();
        assert!(trail.is_some());
        let trail = trail.map(|t| {
            assert_eq!(t.entry_count, 1);
            assert!(t.has_entries());
            t
        });
        assert!(trail.is_some());
    }

    #[test]
    fn test_audit_trail_seal_blocks_recording() {
        let trail = AuditTrail::with_entries(5).seal();
        assert!(trail.sealed);
        assert!(trail.record().is_none());
    }

    #[test]
    fn test_audit_trail_display() {
        let trail = AuditTrail::with_entries(3);
        assert_eq!(format!("{trail}"), "AuditTrail(3 entries)");
        let sealed = trail.seal();
        assert_eq!(format!("{sealed}"), "AuditTrail(3 entries [sealed])");
    }

    // -- AbsenceMarker (∅) --

    #[test]
    fn test_absence_marker_variants() {
        assert!(AbsenceMarker::NotReported.is_informative());
        assert!(AbsenceMarker::ExplicitlyAbsent.is_informative());
        assert!(!AbsenceMarker::BelowThreshold.is_informative());
        assert!(AbsenceMarker::BelowThreshold.is_threshold_limited());
    }

    #[test]
    fn test_absence_marker_display() {
        assert_eq!(format!("{}", AbsenceMarker::NotReported), "∅:not_reported");
        assert_eq!(
            format!("{}", AbsenceMarker::BelowThreshold),
            "∅:below_threshold"
        );
    }

    // -- Pipeline (σ) --

    #[test]
    fn test_pipeline_creation() {
        let p = Pipeline::new(5);
        assert_eq!(p.current_phase, 0);
        assert_eq!(p.total_phases, 5);
        assert!(!p.is_complete());
    }

    #[test]
    fn test_pipeline_advance() {
        let p = Pipeline::new(3);
        let p = p.advance();
        assert!(p.is_some());
        if let Some(p) = p {
            assert_eq!(p.current_phase, 1);
            let p = p.advance();
            assert!(p.is_some());
            if let Some(p) = p {
                assert!(p.is_complete());
                assert!(p.advance().is_none());
            }
        }
    }

    #[test]
    fn test_pipeline_progress() {
        let p = Pipeline::new(5);
        assert!((p.progress() - 0.0).abs() < f64::EPSILON);
        if let Some(p) = p.advance() {
            assert!((p.progress() - 0.25).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_pipeline_display() {
        let p = Pipeline::new(12);
        assert_eq!(format!("{p}"), "Pipeline(1/12)");
    }

    // -- ConsumptionMark (∝) --

    #[test]
    fn test_consumption_mark_new() {
        let cm = ConsumptionMark::new(100.0);
        assert!((cm.consumed - 0.0).abs() < f64::EPSILON);
        assert!((cm.capacity - 100.0).abs() < f64::EPSILON);
        assert!(!cm.is_depleted());
    }

    #[test]
    fn test_consumption_mark_consume() {
        let cm = ConsumptionMark::new(10.0);
        let cm = cm.consume(7.0);
        assert!(cm.is_some());
        if let Some(cm) = cm {
            assert!((cm.remaining() - 3.0).abs() < f64::EPSILON);
            assert!(cm.consume(5.0).is_none()); // exceeds capacity
        }
    }

    #[test]
    fn test_consumption_mark_depletion() {
        let cm = ConsumptionMark::new(5.0);
        if let Some(cm) = cm.consume(5.0) {
            assert!(cm.is_depleted());
            assert!((cm.fraction() - 1.0).abs() < f64::EPSILON);
        }
    }

    // -- EntityStatus (∃) --

    #[test]
    fn test_entity_status_accessible() {
        assert!(EntityStatus::Active.is_accessible());
        assert!(!EntityStatus::Suspended.is_accessible());
        assert!(!EntityStatus::Withdrawn.is_accessible());
        assert!(!EntityStatus::Unknown.is_accessible());
    }

    #[test]
    fn test_entity_status_exists() {
        assert!(EntityStatus::Active.exists());
        assert!(EntityStatus::Suspended.exists());
        assert!(!EntityStatus::Withdrawn.exists());
        assert!(EntityStatus::Unknown.exists());
    }

    #[test]
    fn test_entity_status_definitive() {
        assert!(EntityStatus::Active.is_definitive());
        assert!(!EntityStatus::Unknown.is_definitive());
    }

    #[test]
    fn test_entity_status_display() {
        assert_eq!(format!("{}", EntityStatus::Active), "∃:active");
        assert_eq!(format!("{}", EntityStatus::Withdrawn), "∃:withdrawn");
    }

    // -- ResourcePath (λ) --

    #[test]
    fn test_resource_path_from_str() {
        let p = ResourcePath::from_str_path("brain/sessions/task.md");
        assert_eq!(p.depth(), 3);
        assert_eq!(p.leaf(), Some("task.md"));
    }

    #[test]
    fn test_resource_path_parent() {
        let p = ResourcePath::from_str_path("a/b/c");
        let parent = p.parent();
        assert!(parent.is_some());
        if let Some(parent) = parent {
            assert_eq!(parent.depth(), 2);
            assert_eq!(parent.leaf(), Some("b"));
        }
    }

    #[test]
    fn test_resource_path_join() {
        let p = ResourcePath::from_str_path("brain").join("artifacts");
        assert_eq!(p.depth(), 2);
        assert_eq!(format!("{p}"), "λ:brain/artifacts");
    }

    #[test]
    fn test_resource_path_root_has_no_parent() {
        let p = ResourcePath::from_str_path("root");
        assert!(p.parent().is_none());
    }

    // -- RecursionBound (ρ) --

    #[test]
    fn test_recursion_bound_new() {
        let rb = RecursionBound::new(10);
        assert_eq!(rb.current_depth, 0);
        assert_eq!(rb.max_depth, 10);
        assert!(rb.can_recurse());
        assert_eq!(rb.remaining(), 10);
    }

    #[test]
    fn test_recursion_bound_descend() {
        let rb = RecursionBound::new(2);
        let rb = rb.descend();
        assert!(rb.is_some());
        if let Some(rb) = rb {
            assert_eq!(rb.current_depth, 1);
            let rb = rb.descend();
            assert!(rb.is_some());
            if let Some(rb) = rb {
                assert!(!rb.can_recurse());
                assert!(rb.descend().is_none());
            }
        }
    }

    #[test]
    fn test_recursion_bound_utilization() {
        let rb = RecursionBound::new(4);
        assert!((rb.utilization() - 0.0).abs() < f64::EPSILON);
        if let Some(rb) = rb.descend() {
            assert!((rb.utilization() - 0.25).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_recursion_bound_display() {
        let rb = RecursionBound::new(8);
        assert_eq!(format!("{rb}"), "ρ(0/8 depth)");
    }

    // -- RecordStructure (×) --

    #[test]
    fn test_record_structure_new() {
        let rs = RecordStructure::new(4);
        assert_eq!(rs.field_count, 4);
        assert!(!rs.sealed);
        assert_eq!(rs.arity(), 4);
    }

    #[test]
    fn test_record_structure_pair_triple() {
        let pair = RecordStructure::pair();
        assert_eq!(pair.arity(), 2);
        assert!(!pair.is_unit());

        let triple = RecordStructure::triple();
        assert_eq!(triple.arity(), 3);
    }

    #[test]
    fn test_record_structure_unit() {
        let unit = RecordStructure::new(1);
        assert!(unit.is_unit());
        let empty = RecordStructure::new(0);
        assert!(empty.is_unit());
    }

    #[test]
    fn test_record_structure_seal() {
        let rs = RecordStructure::new(3).seal();
        assert!(rs.sealed);
        assert_eq!(rs.arity(), 3);
    }

    #[test]
    fn test_record_structure_display() {
        let rs = RecordStructure::new(4);
        assert_eq!(format!("{rs}"), "×(4 fields)");
        let sealed = rs.seal();
        assert_eq!(format!("{sealed}"), "×(4 fields [sealed])");
    }

    #[test]
    fn test_record_structure_default() {
        let rs = RecordStructure::default();
        assert_eq!(rs.arity(), 2); // default is pair
    }
}
