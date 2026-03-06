//! # NexVigilant Core -- Skeletal System
//!
//! Maps the biological skeletal system to Claude Code's structural infrastructure
//! per Biological Alignment v2.0 section 3.
//!
//! ## Biological Mapping
//!
//! ```text
//! Axial Skeleton (skull + spine + ribs) = CLAUDE.md files + settings
//!   Skull         = CLAUDE.md         (protects core project knowledge from context loss)
//!   Spine         = CLAUDE.local.md   (personal vertebrae connecting skull to limbs)
//!   Ribs          = settings.json     (protects project-level configuration)
//!
//! Appendicular Skeleton (limbs) = File structure + src/ tree
//!   Arms          = src/ directory    (where code lives)
//!   Legs          = .claude/ directory (what the system stands on)
//!   Hands         = Cargo.toml        (fine manipulation of dependencies)
//!
//! Joints = Interfaces between rigid structures
//!   Ball-and-socket = MCP server      (full range of motion)
//!   Hinge           = Hook events     (one axis: pre/post)
//!   Fixed suture    = Managed settings (rigid, cannot move)
//!
//! Bone Marrow = CLAUDE.md ALSO PRODUCES resources
//!   Contextual awareness, validation rules, behavioral patterns
//!
//! Wolff's Law = CLAUDE.md strengthens where corrections concentrate
//!   Bone remodels along lines of stress -> CLAUDE.md gains rules
//!   where errors recur
//! ```
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol | Role |
//! |---------|-----------|--------|------|
//! | Structural rigidity | Persistence | pi | Bones persist across sessions |
//! | Interface constraint | Boundary | partial | Joints limit range of motion |
//! | Adaptive reinforcement | Recursion | rho | Wolff's Law feedback loop |
//! | Health assessment | State | varsigma | Skeletal health snapshot |
//! | Stress detection | Comparison | kappa | Compare correction frequency |
//! | Resource production | Existence | exists | Marrow brings guidance into being |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    clippy::disallowed_types,
    reason = "Skeletal mapping types are intentionally closed and use bounded structural scoring math"
)]

pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// Bone Types (Axial vs Appendicular)
// ============================================================================

/// Classification of bone types mapping to structural infrastructure.
///
/// The axial skeleton protects the central nervous system (critical knowledge);
/// the appendicular skeleton provides locomotion (code and tooling).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoneType {
    /// Axial: skull, spine, ribs -- protects critical knowledge
    Axial(AxialBone),
    /// Appendicular: arms, legs, hands -- provides structure for action
    Appendicular(AppendularBone),
}

/// Axial bones: the protective core of the project skeleton.
///
/// - Skull = CLAUDE.md (protects core project knowledge from context loss)
/// - Spine = CLAUDE.local.md (personal vertebrae connecting skull to limbs)
/// - Rib = settings.json (protects project-level configuration)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxialBone {
    /// CLAUDE.md -- the skull protecting core project knowledge
    Skull,
    /// CLAUDE.local.md -- personal vertebrae connecting skull to limbs
    Spine,
    /// settings.json -- ribs protecting project-level configuration
    Rib,
}

impl AxialBone {
    /// Returns the Claude Code file path this bone maps to.
    #[must_use]
    pub const fn claude_code_path(&self) -> &'static str {
        match self {
            Self::Skull => "CLAUDE.md",
            Self::Spine => "CLAUDE.local.md",
            Self::Rib => "settings.json",
        }
    }

    /// Returns the biological function description.
    #[must_use]
    pub const fn biological_function(&self) -> &'static str {
        match self {
            Self::Skull => "Protects the brain (core project knowledge) from context loss",
            Self::Spine => "Connects skull to limbs; personal vertebrae for local config",
            Self::Rib => "Protects vital organs (project-level configuration)",
        }
    }
}

/// Appendicular bones: the limbs providing structure for action.
///
/// - Arm = src/ directory structure (where code lives)
/// - Leg = .claude/ directory (what the system stands on)
/// - Hand = Cargo.toml / package.json (fine manipulation of dependencies)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppendularBone {
    /// src/ directory structure -- where code lives (arms reach out to create)
    Arm,
    /// .claude/ directory -- what the system stands on (legs support)
    Leg,
    /// Cargo.toml / package.json -- fine manipulation of dependencies (dexterity)
    Hand,
}

impl AppendularBone {
    /// Returns the Claude Code infrastructure this bone maps to.
    #[must_use]
    pub const fn claude_code_mapping(&self) -> &'static str {
        match self {
            Self::Arm => "src/ directory structure",
            Self::Leg => ".claude/ directory",
            Self::Hand => "Cargo.toml / package.json",
        }
    }

    /// Returns the biological function description.
    #[must_use]
    pub const fn biological_function(&self) -> &'static str {
        match self {
            Self::Arm => "Upper limbs for reaching and manipulating (code authoring)",
            Self::Leg => "Lower limbs for standing and locomotion (infrastructure support)",
            Self::Hand => "Fine motor control for precise manipulation (dependency management)",
        }
    }
}

// ============================================================================
// Joint Types
// ============================================================================

/// Types of joints connecting rigid structures, mapped to interface patterns.
///
/// - Ball-and-socket = MCP server interface (full range of motion, any tool)
/// - Hinge = Hook event interface (one axis: pre/post)
/// - Fixed suture = Managed settings (rigid, cannot be moved by user)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JointType {
    /// MCP server interface: full range of motion (invoke any tool)
    BallAndSocket,
    /// Hook event interface: one axis of motion (pre or post)
    Hinge,
    /// Managed settings: rigid, cannot be moved by user
    FixedSuture,
}

impl JointType {
    /// Returns the Claude Code interface this joint type maps to.
    #[must_use]
    pub const fn claude_code_interface(&self) -> &'static str {
        match self {
            Self::BallAndSocket => "MCP server interface (full range of tool invocation)",
            Self::Hinge => "Hook event interface (pre/post one-axis)",
            Self::FixedSuture => "Managed settings (rigid, admin-controlled)",
        }
    }

    /// Returns the default range of motion for this joint type.
    ///
    /// 0.0 = completely fixed (suture)
    /// 1.0 = full range of motion (ball-and-socket)
    #[must_use]
    pub const fn default_range_of_motion(&self) -> f64 {
        match self {
            Self::BallAndSocket => 1.0,
            Self::Hinge => 0.5,
            Self::FixedSuture => 0.0,
        }
    }
}

/// A joint connecting two or more bones with a constrained range of motion.
///
/// In biological terms, joints allow controlled movement between rigid structures.
/// In Claude Code, interfaces (MCP, hooks, settings) connect components with
/// defined constraints on what operations are permitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    /// The type of joint determining interface pattern
    pub joint_type: JointType,
    /// Bones connected by this joint
    pub connected_bones: Vec<BoneType>,
    /// Range of motion: 0.0 (fixed suture) to 1.0 (full ball-and-socket)
    pub range_of_motion: f64,
}

impl Joint {
    /// Creates a new joint connecting bones with the default range of motion
    /// for the given joint type.
    #[must_use]
    pub fn new(joint_type: JointType, connected_bones: Vec<BoneType>) -> Self {
        let range_of_motion = joint_type.default_range_of_motion();
        Self {
            joint_type,
            connected_bones,
            range_of_motion,
        }
    }

    /// Creates a new joint with a custom range of motion.
    /// The value is clamped to the 0.0..=1.0 range.
    #[must_use]
    pub fn with_range(joint_type: JointType, connected_bones: Vec<BoneType>, range: f64) -> Self {
        let range_of_motion = range.clamp(0.0, 1.0);
        Self {
            joint_type,
            connected_bones,
            range_of_motion,
        }
    }

    /// Returns true if this joint allows any motion at all.
    #[must_use]
    pub fn is_mobile(&self) -> bool {
        self.range_of_motion > 0.0
    }

    /// Returns true if this joint is fully fixed (suture).
    #[must_use]
    pub fn is_fixed(&self) -> bool {
        self.range_of_motion == 0.0
    }
}

// ============================================================================
// Bone Marrow
// ============================================================================

/// Bone marrow: the productive tissue inside bones.
///
/// In biology, bone marrow produces blood cells. In Claude Code, CLAUDE.md
/// produces runtime resources: contextual awareness, validation rules, and
/// behavioral patterns. The marrow is what makes CLAUDE.md more than a static
/// file -- it actively generates guidance that shapes every interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneMarrow {
    /// Number of knowledge entries in CLAUDE.md
    pub knowledge_entries: u32,
    /// Number of corrections recorded (stress points)
    pub corrections: u32,
    /// Number of behavioral patterns extracted
    pub patterns: u32,
    /// Whether the marrow actively generates guidance (true if CLAUDE.md is read)
    pub generates_guidance: bool,
}

impl BoneMarrow {
    /// Creates new bone marrow with the given resource counts.
    #[must_use]
    pub const fn new(
        knowledge_entries: u32,
        corrections: u32,
        patterns: u32,
        generates_guidance: bool,
    ) -> Self {
        Self {
            knowledge_entries,
            corrections,
            patterns,
            generates_guidance,
        }
    }

    /// Returns true if the marrow is productive (has entries and generates guidance).
    #[must_use]
    pub fn is_productive(&self) -> bool {
        self.generates_guidance && self.knowledge_entries > 0
    }

    /// Returns the total resource count produced by this marrow.
    #[must_use]
    pub fn total_resources(&self) -> u32 {
        self.knowledge_entries
            .saturating_add(self.corrections)
            .saturating_add(self.patterns)
    }
}

// ============================================================================
// Correction (Stress Point)
// ============================================================================

/// A correction recording a stress point where bone remodels.
///
/// In biology, repeated stress causes bone to thicken (Wolff's Law).
/// In Claude Code, each correction is a mistake that was fixed. When the same
/// area accumulates enough corrections, a new rule should be added to CLAUDE.md.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    /// The area/topic where the correction occurred (e.g. "unwrap_usage", "test_patterns")
    pub area: String,
    /// Description of the correction
    pub correction_text: String,
    /// How many times this correction has occurred
    pub frequency: u32,
    /// When the correction was last recorded (ISO 8601 string)
    pub corrected_at: String,
}

impl Correction {
    /// Creates a new correction record.
    #[must_use]
    pub fn new(area: String, correction_text: String, corrected_at: String) -> Self {
        Self {
            area,
            correction_text,
            frequency: 1,
            corrected_at,
        }
    }

    /// Creates a correction with an explicit frequency.
    #[must_use]
    pub fn with_frequency(
        area: String,
        correction_text: String,
        frequency: u32,
        corrected_at: String,
    ) -> Self {
        Self {
            area,
            correction_text,
            frequency,
            corrected_at,
        }
    }
}

// ============================================================================
// Wolff's Law
// ============================================================================

/// Wolff's Law: bone strengthens where stress concentrates.
///
/// In biology, bone tissue remodels to become stronger along lines of repeated
/// mechanical stress. In Claude Code, CLAUDE.md should gain new rules in areas
/// where corrections accumulate. When the correction frequency for an area
/// exceeds the stress threshold, that area should be codified in CLAUDE.md.
///
/// This is the feedback loop: mistakes --> corrections --> CLAUDE.md rules -->
/// fewer mistakes. The recursive nature (rho-dominant) is the key insight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolffsLaw {
    /// All corrections recorded across sessions
    pub corrections: Vec<Correction>,
    /// Frequency threshold above which a correction area should be added to CLAUDE.md
    pub stress_threshold: u32,
}

impl WolffsLaw {
    /// Creates a new Wolff's Law tracker with the given threshold.
    #[must_use]
    pub fn new(stress_threshold: u32) -> Self {
        Self {
            corrections: Vec::new(),
            stress_threshold,
        }
    }

    /// Returns true if the correction count for the given area exceeds the threshold.
    ///
    /// When this returns true, the area has accumulated enough stress that
    /// CLAUDE.md should be updated with a new rule to prevent future errors.
    #[must_use]
    pub fn should_add_to_claude_md(&self, area: &str) -> bool {
        let count: u32 = self
            .corrections
            .iter()
            .filter(|c| c.area == area)
            .map(|c| c.frequency)
            .sum();
        count >= self.stress_threshold
    }

    /// Returns areas sorted by correction frequency (descending).
    ///
    /// This is the stress concentration map: areas at the top are where
    /// the skeleton is under the most strain and most likely to need
    /// reinforcement in CLAUDE.md.
    #[must_use]
    pub fn stress_concentration(&self) -> Vec<(String, u32)> {
        use std::collections::HashMap;
        let mut area_counts: HashMap<String, u32> = HashMap::new();
        for c in &self.corrections {
            *area_counts.entry(c.area.clone()).or_insert(0) += c.frequency;
        }
        let mut sorted: Vec<(String, u32)> = area_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted
    }

    /// Records a correction for the given area. If a correction for that area
    /// already exists, increments its frequency. Otherwise, adds a new correction.
    pub fn record_correction(&mut self, area: &str, text: &str) {
        let now = nexcore_chrono::DateTime::now().to_rfc3339();
        let existing = self.corrections.iter_mut().find(|c| c.area == area);
        match existing {
            Some(c) => {
                c.frequency = c.frequency.saturating_add(1);
                c.corrected_at.clone_from(&now);
                c.correction_text = text.to_string();
            }
            None => {
                self.corrections
                    .push(Correction::new(area.to_string(), text.to_string(), now));
            }
        }
    }

    /// Returns the total number of corrections across all areas.
    #[must_use]
    pub fn total_corrections(&self) -> u32 {
        self.corrections.iter().map(|c| c.frequency).sum()
    }

    /// Returns areas that have exceeded the stress threshold and should
    /// be codified in CLAUDE.md.
    #[must_use]
    pub fn areas_needing_reinforcement(&self) -> Vec<String> {
        self.stress_concentration()
            .into_iter()
            .filter(|(_, count)| *count >= self.stress_threshold)
            .map(|(area, _)| area)
            .collect()
    }
}

// ============================================================================
// Project Skeleton
// ============================================================================

/// A snapshot of the project's structural skeleton.
///
/// Maps the presence/absence and health of each bone in the project:
/// - skull: CLAUDE.md path and whether it exists
/// - spine: whether CLAUDE.local.md exists
/// - ribs: count of settings.json entries
/// - arms: count of src/ crates/modules
/// - legs: whether .claude/ directory exists
/// - hands: whether Cargo.toml is present
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSkeleton {
    /// CLAUDE.md path and whether it exists (the skull)
    pub skull: SkullState,
    /// Whether CLAUDE.local.md exists (the spine)
    pub spine_present: bool,
    /// Count of settings entries (the ribs)
    pub ribs_count: u32,
    /// Count of src/ crates or modules (the arms)
    pub arms_crate_count: u32,
    /// Whether .claude/ directory exists (the legs)
    pub legs_present: bool,
    /// Whether Cargo.toml is present (the hands)
    pub hands_present: bool,
}

/// State of the skull (CLAUDE.md).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkullState {
    /// Path to CLAUDE.md
    pub path: String,
    /// Whether CLAUDE.md exists at that path
    pub exists: bool,
}

impl ProjectSkeleton {
    /// Returns the number of bones present in this skeleton.
    #[must_use]
    pub fn bone_count(&self) -> u32 {
        let mut count = 0u32;
        if self.skull.exists {
            count = count.saturating_add(1);
        }
        if self.spine_present {
            count = count.saturating_add(1);
        }
        if self.ribs_count > 0 {
            count = count.saturating_add(1);
        }
        if self.arms_crate_count > 0 {
            count = count.saturating_add(1);
        }
        if self.legs_present {
            count = count.saturating_add(1);
        }
        if self.hands_present {
            count = count.saturating_add(1);
        }
        count
    }

    /// Returns true if the axial skeleton (skull + spine + ribs) is complete.
    #[must_use]
    pub fn axial_complete(&self) -> bool {
        self.skull.exists && self.spine_present && self.ribs_count > 0
    }

    /// Returns true if the appendicular skeleton (arms + legs + hands) is complete.
    #[must_use]
    pub fn appendicular_complete(&self) -> bool {
        self.arms_crate_count > 0 && self.legs_present && self.hands_present
    }
}

// ============================================================================
// Skeletal Health
// ============================================================================

/// Overall health assessment of the project's skeletal system.
///
/// Evaluates whether the structural components are present and functioning:
/// - Is CLAUDE.md present? (skull intact)
/// - Are corrections feeding back into CLAUDE.md? (Wolff's Law active)
/// - Are settings versioned? (ribs protecting config)
/// - Is the feedback loop operational? (marrow producing resources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletalHealth {
    /// Whether CLAUDE.md is present (skull intact)
    pub claude_md_present: bool,
    /// Whether corrections are feeding back into CLAUDE.md
    pub corrections_feeding_claude_md: bool,
    /// Whether settings are versioned (ribs protecting config)
    pub settings_versioned: bool,
    /// Whether Wolff's Law feedback loop is active
    pub wolff_law_active: bool,
}

impl SkeletalHealth {
    /// Returns true if the skeleton is structurally sound.
    ///
    /// A healthy skeleton requires at minimum:
    /// - CLAUDE.md present (skull)
    /// - Wolff's Law active (adaptive reinforcement)
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.claude_md_present && self.wolff_law_active
    }

    /// Returns a health score from 0 to 4 based on how many indicators are positive.
    #[must_use]
    pub fn score(&self) -> u32 {
        let mut s = 0u32;
        if self.claude_md_present {
            s = s.saturating_add(1);
        }
        if self.corrections_feeding_claude_md {
            s = s.saturating_add(1);
        }
        if self.settings_versioned {
            s = s.saturating_add(1);
        }
        if self.wolff_law_active {
            s = s.saturating_add(1);
        }
        s
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Wolff's Law ----------------------------------------------------------

    #[test]
    fn wolffs_law_empty_does_not_trigger() {
        let wl = WolffsLaw::new(3);
        assert!(!wl.should_add_to_claude_md("unwrap_usage"));
    }

    #[test]
    fn wolffs_law_below_threshold() {
        let mut wl = WolffsLaw::new(3);
        wl.record_correction("unwrap_usage", "Use assert instead of unwrap");
        wl.record_correction("unwrap_usage", "Use assert instead of unwrap");
        assert!(!wl.should_add_to_claude_md("unwrap_usage"));
    }

    #[test]
    fn wolffs_law_at_threshold_triggers() {
        let mut wl = WolffsLaw::new(3);
        wl.record_correction("unwrap_usage", "Use assert instead of unwrap");
        wl.record_correction("unwrap_usage", "Use assert instead of unwrap");
        wl.record_correction("unwrap_usage", "Use assert instead of unwrap");
        assert!(wl.should_add_to_claude_md("unwrap_usage"));
    }

    #[test]
    fn wolffs_law_above_threshold() {
        let mut wl = WolffsLaw::new(3);
        for _ in 0..5 {
            wl.record_correction("test_patterns", "Use structured assertions");
        }
        assert!(wl.should_add_to_claude_md("test_patterns"));
    }

    #[test]
    fn wolffs_law_stress_concentration_sorted() {
        let mut wl = WolffsLaw::new(3);
        wl.record_correction("unwrap_usage", "fix 1");
        wl.record_correction("unwrap_usage", "fix 2");
        wl.record_correction("unwrap_usage", "fix 3");
        wl.record_correction("test_patterns", "fix 1");
        wl.record_correction("import_order", "fix 1");
        wl.record_correction("import_order", "fix 2");

        let stress = wl.stress_concentration();
        assert!(!stress.is_empty());
        // unwrap_usage should be first (frequency 3)
        assert_eq!(stress[0].0, "unwrap_usage");
        assert_eq!(stress[0].1, 3);
    }

    #[test]
    fn wolffs_law_record_increments_existing() {
        let mut wl = WolffsLaw::new(3);
        wl.record_correction("area1", "text1");
        wl.record_correction("area1", "text2");
        // Should have 1 correction with frequency 2
        assert_eq!(wl.corrections.len(), 1);
        assert_eq!(wl.corrections[0].frequency, 2);
    }

    #[test]
    fn wolffs_law_areas_needing_reinforcement() {
        let mut wl = WolffsLaw::new(2);
        wl.record_correction("hot_area", "fix");
        wl.record_correction("hot_area", "fix");
        wl.record_correction("cold_area", "fix");

        let areas = wl.areas_needing_reinforcement();
        assert!(areas.contains(&"hot_area".to_string()));
        assert!(!areas.contains(&"cold_area".to_string()));
    }

    #[test]
    fn wolffs_law_total_corrections() {
        let mut wl = WolffsLaw::new(5);
        wl.record_correction("a", "fix");
        wl.record_correction("b", "fix");
        wl.record_correction("a", "fix");
        assert_eq!(wl.total_corrections(), 3);
    }

    // -- Joint Mechanics ------------------------------------------------------

    #[test]
    fn joint_ball_and_socket_full_range() {
        let joint = Joint::new(
            JointType::BallAndSocket,
            vec![
                BoneType::Axial(AxialBone::Skull),
                BoneType::Appendicular(AppendularBone::Arm),
            ],
        );
        assert_eq!(joint.range_of_motion, 1.0);
        assert!(joint.is_mobile());
        assert!(!joint.is_fixed());
    }

    #[test]
    fn joint_hinge_half_range() {
        let joint = Joint::new(
            JointType::Hinge,
            vec![
                BoneType::Axial(AxialBone::Rib),
                BoneType::Appendicular(AppendularBone::Leg),
            ],
        );
        assert_eq!(joint.range_of_motion, 0.5);
        assert!(joint.is_mobile());
    }

    #[test]
    fn joint_fixed_suture_no_motion() {
        let joint = Joint::new(
            JointType::FixedSuture,
            vec![BoneType::Axial(AxialBone::Skull)],
        );
        assert_eq!(joint.range_of_motion, 0.0);
        assert!(!joint.is_mobile());
        assert!(joint.is_fixed());
    }

    #[test]
    fn joint_custom_range_clamped() {
        let joint = Joint::with_range(JointType::Hinge, vec![], 1.5);
        assert_eq!(joint.range_of_motion, 1.0);

        let joint2 = Joint::with_range(JointType::Hinge, vec![], -0.5);
        assert_eq!(joint2.range_of_motion, 0.0);
    }

    // -- Bone Marrow Production -----------------------------------------------

    #[test]
    fn bone_marrow_productive() {
        let marrow = BoneMarrow::new(50, 10, 20, true);
        assert!(marrow.is_productive());
        assert_eq!(marrow.total_resources(), 80);
    }

    #[test]
    fn bone_marrow_not_productive_no_entries() {
        let marrow = BoneMarrow::new(0, 5, 3, true);
        assert!(!marrow.is_productive());
    }

    #[test]
    fn bone_marrow_not_productive_no_guidance() {
        let marrow = BoneMarrow::new(50, 10, 20, false);
        assert!(!marrow.is_productive());
    }

    // -- Skeletal Health -------------------------------------------------------

    #[test]
    fn skeletal_health_fully_healthy() {
        let health = SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: true,
            settings_versioned: true,
            wolff_law_active: true,
        };
        assert!(health.is_healthy());
        assert_eq!(health.score(), 4);
    }

    #[test]
    fn skeletal_health_minimal() {
        let health = SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: false,
            settings_versioned: false,
            wolff_law_active: true,
        };
        assert!(health.is_healthy());
        assert_eq!(health.score(), 2);
    }

    #[test]
    fn skeletal_health_unhealthy_no_claude_md() {
        let health = SkeletalHealth {
            claude_md_present: false,
            corrections_feeding_claude_md: true,
            settings_versioned: true,
            wolff_law_active: true,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn skeletal_health_unhealthy_no_wolff() {
        let health = SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: true,
            settings_versioned: true,
            wolff_law_active: false,
        };
        assert!(!health.is_healthy());
    }

    // -- Project Skeleton ------------------------------------------------------

    #[test]
    fn project_skeleton_full() {
        let skeleton = ProjectSkeleton {
            skull: SkullState {
                path: "CLAUDE.md".to_string(),
                exists: true,
            },
            spine_present: true,
            ribs_count: 12,
            arms_crate_count: 90,
            legs_present: true,
            hands_present: true,
        };
        assert_eq!(skeleton.bone_count(), 6);
        assert!(skeleton.axial_complete());
        assert!(skeleton.appendicular_complete());
    }

    #[test]
    fn project_skeleton_partial() {
        let skeleton = ProjectSkeleton {
            skull: SkullState {
                path: "CLAUDE.md".to_string(),
                exists: true,
            },
            spine_present: false,
            ribs_count: 0,
            arms_crate_count: 5,
            legs_present: true,
            hands_present: false,
        };
        assert_eq!(skeleton.bone_count(), 3);
        assert!(!skeleton.axial_complete());
        assert!(!skeleton.appendicular_complete());
    }

    // -- Bone Type Mappings ---------------------------------------------------

    #[test]
    fn axial_bone_paths() {
        assert_eq!(AxialBone::Skull.claude_code_path(), "CLAUDE.md");
        assert_eq!(AxialBone::Spine.claude_code_path(), "CLAUDE.local.md");
        assert_eq!(AxialBone::Rib.claude_code_path(), "settings.json");
    }

    #[test]
    fn appendicular_bone_mappings() {
        assert_eq!(
            AppendularBone::Arm.claude_code_mapping(),
            "src/ directory structure"
        );
        assert_eq!(
            AppendularBone::Leg.claude_code_mapping(),
            ".claude/ directory"
        );
        assert_eq!(
            AppendularBone::Hand.claude_code_mapping(),
            "Cargo.toml / package.json"
        );
    }

    #[test]
    fn joint_type_interface_mappings() {
        assert!(
            JointType::BallAndSocket
                .claude_code_interface()
                .contains("MCP")
        );
        assert!(JointType::Hinge.claude_code_interface().contains("Hook"));
        assert!(
            JointType::FixedSuture
                .claude_code_interface()
                .contains("Managed")
        );
    }

    // -- Serialization --------------------------------------------------------

    #[test]
    fn correction_serialization_roundtrip() {
        let correction = Correction::new(
            "unwrap_usage".to_string(),
            "Use assert! instead".to_string(),
            "2025-01-15T10:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&correction);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        let deserialized: Result<Correction, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        if let Ok(c) = deserialized {
            assert_eq!(c.area, "unwrap_usage");
            assert_eq!(c.frequency, 1);
        }
    }

    #[test]
    fn wolffs_law_serialization_roundtrip() {
        let mut wl = WolffsLaw::new(3);
        wl.record_correction("test_area", "fix it");
        let json = serde_json::to_string(&wl);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        let deserialized: Result<WolffsLaw, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());
        if let Ok(w) = deserialized {
            assert_eq!(w.stress_threshold, 3);
            assert_eq!(w.corrections.len(), 1);
        }
    }

    #[test]
    fn skeletal_health_serialization() {
        let health = SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: false,
            settings_versioned: true,
            wolff_law_active: false,
        };
        let json = serde_json::to_string(&health);
        assert!(json.is_ok());
    }
}
