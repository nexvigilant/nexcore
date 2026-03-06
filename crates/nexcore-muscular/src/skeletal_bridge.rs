//! # Skeletal Bridge
//!
//! Inter-crate pipeline: Skeletal → Muscular.
//!
//! Converts skeletal structural assessments into muscular load factors
//! and maps bone health to fatigue modifiers.
//!
//! ```text
//! Skeletal::SkeletalHealth → Fatigue modifier
//! Skeletal::ProjectSkeleton → MotorActivation load hints
//! Skeletal::WolffsLaw → SizePrinciple stress feedback
//! ```
//!
//! **Biological mapping**: The musculoskeletal system — muscles attach to
//! bones via tendons. Bone health directly affects muscle performance:
//! weak bones limit force production, and skeletal stress patterns
//! (Wolff's Law) indicate where muscles must compensate.

use nexcore_skeletal::{ProjectSkeleton, SkeletalHealth, WolffsLaw};

use crate::Fatigue;

/// Convert skeletal health into a fatigue modifier (0.0–1.0).
///
/// **Biological mapping**: Bone integrity affects muscle endurance —
/// osteoporotic bones fatigue muscles faster because the skeleton cannot
/// properly support the load. A healthy skeleton (score 4/4) allows full
/// muscle capacity; a weakened skeleton increases effective fatigue.
pub fn skeletal_health_to_fatigue_modifier(health: &SkeletalHealth) -> f64 {
    // Score is 0–4; convert to a fatigue penalty.
    // Score 4 → 0.0 penalty (no added fatigue)
    // Score 0 → 0.25 penalty (significant added fatigue)
    let score = health.score();
    (4u32.saturating_sub(score)) as f64 * 0.0625 // 0.0625 = 1/16 per missing indicator
}

/// Apply skeletal health as a modifier to an existing fatigue tracker.
///
/// Increases the consumed tokens proportionally to skeletal weakness,
/// simulating how poor structural health accelerates muscle fatigue.
///
/// **Biological mapping**: Compensatory fatigue — when bones are weak,
/// muscles must work harder to stabilize joints, consuming more energy.
pub fn apply_skeletal_modifier(fatigue: &mut Fatigue, health: &SkeletalHealth) {
    let modifier = skeletal_health_to_fatigue_modifier(health);
    if modifier > 0.0 {
        // Add a fraction of total capacity as extra fatigue
        let penalty = (fatigue.total_context_tokens as f64 * modifier) as u64;
        fatigue.consume(penalty);
    }
}

/// Map project skeleton bone count to a complexity estimate.
///
/// Returns a task complexity value (0–9) based on how many structural
/// components are present. More bones = more complex musculoskeletal load.
///
/// **Biological mapping**: Body mass and muscle recruitment — a larger
/// skeleton with more bones requires more motor units to operate.
pub fn skeleton_to_complexity(skeleton: &ProjectSkeleton) -> u8 {
    // bone_count is 0–6; map to complexity 0–9
    let bones = skeleton.bone_count();
    let complexity = (bones as f64 * 1.5).min(9.0) as u8;
    complexity
}

/// Extract a throughput metric from Wolff's Law stress data.
///
/// Returns the total correction count — a measure of how much
/// musculoskeletal stress the system is under.
///
/// **Biological mapping**: Stress load — more corrections mean more
/// bone remodeling, which corresponds to higher muscular demand to
/// support the changing skeletal structure.
pub fn wolffs_law_throughput(wolff: &WolffsLaw) -> u32 {
    wolff.total_corrections()
}

/// Determine if the musculoskeletal system needs reinforcement.
///
/// Returns true if Wolff's Law has identified areas needing codification
/// in CLAUDE.md — indicating structural weak points that muscles
/// (tools) must compensate for.
///
/// **Biological mapping**: Stress fracture risk — when bone stress
/// exceeds remodeling capacity, the musculoskeletal system is at risk.
pub fn needs_reinforcement(wolff: &WolffsLaw) -> bool {
    !wolff.areas_needing_reinforcement().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn healthy_skeleton() -> SkeletalHealth {
        SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: true,
            settings_versioned: true,
            wolff_law_active: true,
        }
    }

    fn weak_skeleton() -> SkeletalHealth {
        SkeletalHealth {
            claude_md_present: false,
            corrections_feeding_claude_md: false,
            settings_versioned: false,
            wolff_law_active: false,
        }
    }

    #[test]
    fn test_healthy_skeleton_no_fatigue_modifier() {
        let modifier = skeletal_health_to_fatigue_modifier(&healthy_skeleton());
        assert!((modifier - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_weak_skeleton_has_fatigue_modifier() {
        let modifier = skeletal_health_to_fatigue_modifier(&weak_skeleton());
        assert!(modifier > 0.0, "Weak skeleton should add fatigue: got {modifier}");
        assert!((modifier - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_partial_skeleton_intermediate_modifier() {
        let health = SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: false,
            settings_versioned: true,
            wolff_law_active: false,
        };
        let modifier = skeletal_health_to_fatigue_modifier(&health);
        // Score 2/4 → modifier = 2 * 0.0625 = 0.125
        assert!((modifier - 0.125).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_skeletal_modifier_healthy() {
        let mut fatigue = Fatigue::new(100_000);
        apply_skeletal_modifier(&mut fatigue, &healthy_skeleton());
        // No modification — fatigue should remain 0
        assert!((fatigue.fatigue_level() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_apply_skeletal_modifier_weak() {
        let mut fatigue = Fatigue::new(100_000);
        apply_skeletal_modifier(&mut fatigue, &weak_skeleton());
        // Weak skeleton adds 25% of capacity as penalty
        assert!(fatigue.fatigue_level() > 0.0);
        assert!((fatigue.fatigue_level() - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_skeleton_to_complexity_full() {
        let skeleton = nexcore_skeletal::ProjectSkeleton {
            skull: nexcore_skeletal::SkullState {
                path: "CLAUDE.md".to_string(),
                exists: true,
            },
            spine_present: true,
            ribs_count: 12,
            arms_crate_count: 90,
            legs_present: true,
            hands_present: true,
        };
        let complexity = skeleton_to_complexity(&skeleton);
        assert_eq!(complexity, 9); // 6 bones * 1.5 = 9.0
    }

    #[test]
    fn test_skeleton_to_complexity_empty() {
        let skeleton = nexcore_skeletal::ProjectSkeleton {
            skull: nexcore_skeletal::SkullState {
                path: "CLAUDE.md".to_string(),
                exists: false,
            },
            spine_present: false,
            ribs_count: 0,
            arms_crate_count: 0,
            legs_present: false,
            hands_present: false,
        };
        let complexity = skeleton_to_complexity(&skeleton);
        assert_eq!(complexity, 0);
    }

    #[test]
    fn test_wolffs_law_throughput() {
        let mut wolff = WolffsLaw::new(3);
        wolff.record_correction("area1", "fix1");
        wolff.record_correction("area1", "fix2");
        wolff.record_correction("area2", "fix1");
        assert_eq!(wolffs_law_throughput(&wolff), 3);
    }

    #[test]
    fn test_wolffs_law_throughput_empty() {
        let wolff = WolffsLaw::new(3);
        assert_eq!(wolffs_law_throughput(&wolff), 0);
    }

    #[test]
    fn test_needs_reinforcement_no() {
        let wolff = WolffsLaw::new(5);
        assert!(!needs_reinforcement(&wolff));
    }

    #[test]
    fn test_needs_reinforcement_yes() {
        let mut wolff = WolffsLaw::new(2);
        wolff.record_correction("hot-area", "fix");
        wolff.record_correction("hot-area", "fix");
        assert!(needs_reinforcement(&wolff));
    }
}
