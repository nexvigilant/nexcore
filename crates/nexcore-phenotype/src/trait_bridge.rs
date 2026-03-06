//! # Trait Bridge
//!
//! Multi-system aggregation: Muscular + Skeletal + Nervous → Phenotype.
//!
//! Converts health metrics from three internal systems into observable
//! phenotypic traits — the outward expression of underlying system health.
//!
//! ```text
//! Muscular::MuscularHealth  ─┐
//! Skeletal::SkeletalHealth  ─┤──► PhenotypicProfile (observable fitness)
//! Nervous::NervousHealth    ─┘
//! ```
//!
//! **Biological mapping**: A phenotype is the observable expression of
//! underlying genotype + environment. In the same way, this bridge
//! aggregates internal system health metrics into an observable profile
//! that represents how the organism *appears* from outside. A fatigued
//! muscular system, a weak skeleton, or a slow nervous system all
//! manifest as degraded phenotypic traits — visible to external
//! monitoring, adversarial probes, and drift detection.

use nexcore_muscular::MuscularHealth;
use nexcore_nervous::NervousHealth;
use nexcore_skeletal::SkeletalHealth;

/// Aggregated phenotypic profile derived from multiple system health metrics.
///
/// **Biological mapping**: The phenotype — what you can observe from outside.
/// Internal health scores from muscular (performance), skeletal (structure),
/// and nervous (responsiveness) systems combine into an overall fitness
/// profile with per-system trait scores.
#[derive(Debug, Clone, PartialEq)]
pub struct PhenotypicProfile {
    /// Overall fitness score (0.0–1.0). Average of system trait scores.
    pub fitness: f64,
    /// Muscular trait: performance capacity (0.0–1.0).
    pub performance: f64,
    /// Skeletal trait: structural integrity (0.0–1.0).
    pub integrity: f64,
    /// Nervous trait: responsiveness / signal speed (0.0–1.0).
    pub responsiveness: f64,
    /// Number of systems contributing to this profile.
    pub systems_assessed: usize,
    /// Whether the phenotype is considered healthy (fitness >= 0.6).
    pub healthy: bool,
}

/// Convert muscular health into a performance trait score (0.0–1.0).
///
/// **Biological mapping**: Muscular performance phenotype — grip strength,
/// endurance, coordination. Reflects whether the muscular system can
/// sustain work without excessive fatigue or imbalance.
pub fn muscular_to_trait(health: &MuscularHealth) -> f64 {
    let mut score = 0.0;
    if health.size_principle_compliant {
        score += 0.25;
    }
    if health.antagonistic_pairs_defined {
        score += 0.25;
    }
    if health.recruitment_balanced {
        score += 0.20;
    }
    if health.cardiac_running {
        score += 0.15;
    }
    // Fatigue reduces performance: 0.0 fatigue = +0.15, 1.0 fatigue = +0.0
    score += 0.15 * (1.0 - health.fatigue_level.clamp(0.0, 1.0));
    score
}

/// Convert skeletal health into a structural integrity trait score (0.0–1.0).
///
/// **Biological mapping**: Skeletal integrity phenotype — posture, bone
/// density, joint stability. Reflects whether the structural framework
/// supports the organism properly. Missing CLAUDE.md (skull) or inactive
/// Wolff's Law feedback weakens the phenotypic expression.
pub fn skeletal_to_trait(health: &SkeletalHealth) -> f64 {
    let mut score = 0.0;
    if health.claude_md_present {
        score += 0.30;
    }
    if health.corrections_feeding_claude_md {
        score += 0.25;
    }
    if health.settings_versioned {
        score += 0.20;
    }
    if health.wolff_law_active {
        score += 0.25;
    }
    score
}

/// Convert nervous health into a responsiveness trait score (0.0–1.0).
///
/// **Biological mapping**: Nervous responsiveness phenotype — reaction
/// time, reflex speed, sensory acuity. High myelination ratio and low
/// signal latency indicate a responsive nervous system that manifests
/// as quick, coordinated phenotypic behavior.
pub fn nervous_to_trait(health: &NervousHealth) -> f64 {
    let mut score = 0.0;

    // Myelination ratio contributes to signal speed
    // Ratio of 1.0 = fully myelinated = fast, 0.0 = unmyelinated = slow
    score += 0.30 * health.myelination_ratio.clamp(0.0, 1.0);

    // Low latency is good. Assume <50ms is excellent, >500ms is poor.
    let latency_factor = if health.avg_signal_latency_ms <= 0.0 {
        1.0
    } else {
        (1.0 - (health.avg_signal_latency_ms / 500.0)).clamp(0.0, 1.0)
    };
    score += 0.25 * latency_factor;

    // Sensory integration
    if health.sensory_integration_ok {
        score += 0.20;
    }

    // Active neurons and reflex arcs (capacity)
    // More is better, but normalize: assume 10+ neurons = full score
    let neuron_factor = (health.neuron_count as f64 / 10.0).min(1.0);
    score += 0.15 * neuron_factor;

    let reflex_factor = (health.reflex_arcs_active as f64 / 5.0).min(1.0);
    score += 0.10 * reflex_factor;

    score
}

/// Aggregate health metrics from all three systems into a phenotypic profile.
///
/// **Biological mapping**: Phenotypic expression — the integrated,
/// observable output of all underlying biological systems. Just as a
/// doctor assesses a patient's phenotype by observing strength (muscular),
/// posture (skeletal), and reflexes (nervous), this function combines
/// three health assessments into one observable profile.
pub fn aggregate_phenotype(
    muscular: &MuscularHealth,
    skeletal: &SkeletalHealth,
    nervous: &NervousHealth,
) -> PhenotypicProfile {
    let performance = muscular_to_trait(muscular);
    let integrity = skeletal_to_trait(skeletal);
    let responsiveness = nervous_to_trait(nervous);
    let fitness = (performance + integrity + responsiveness) / 3.0;

    PhenotypicProfile {
        fitness,
        performance,
        integrity,
        responsiveness,
        systems_assessed: 3,
        healthy: fitness >= 0.6,
    }
}

/// Compute the throughput metric for phenotypic assessment.
///
/// Returns the number of healthy traits (score >= 0.5) out of the three
/// system assessments.
///
/// **Biological mapping**: Phenotypic vigor — how many observable traits
/// reach a healthy threshold. An organism with all three systems healthy
/// has high phenotypic throughput; one with degraded systems shows
/// reduced phenotypic expression.
pub fn phenotype_throughput(
    muscular: &MuscularHealth,
    skeletal: &SkeletalHealth,
    nervous: &NervousHealth,
) -> usize {
    let mut count = 0usize;
    if muscular_to_trait(muscular) >= 0.5 {
        count = count.saturating_add(1);
    }
    if skeletal_to_trait(skeletal) >= 0.5 {
        count = count.saturating_add(1);
    }
    if nervous_to_trait(nervous) >= 0.5 {
        count = count.saturating_add(1);
    }
    count
}

/// Identify which mutation types are most likely based on system health.
///
/// **Biological mapping**: Phenotype–genotype correlation — certain
/// phenotypic weaknesses predispose the organism to specific mutation
/// types. A structurally weak skeleton (missing CLAUDE.md) predisposes
/// to `StructureSwap` mutations. A fatigued muscular system predisposes
/// to `RangeExpand` (overextension) mutations.
pub fn health_to_mutation_risk(
    muscular: &MuscularHealth,
    skeletal: &SkeletalHealth,
    nervous: &NervousHealth,
) -> Vec<crate::Mutation> {
    let mut risks = Vec::new();

    // Low muscular performance → risk of range expansion (overextension)
    if muscular_to_trait(muscular) < 0.5 {
        risks.push(crate::Mutation::RangeExpand);
    }

    // Weak skeletal integrity → risk of structural swap
    if skeletal_to_trait(skeletal) < 0.5 {
        risks.push(crate::Mutation::StructureSwap);
    }

    // Slow nervous responsiveness → risk of type mismatch (misinterpretation)
    if nervous_to_trait(nervous) < 0.5 {
        risks.push(crate::Mutation::TypeMismatch);
    }

    // If all systems are weak, also risk field removal (degradation)
    if muscular_to_trait(muscular) < 0.3
        && skeletal_to_trait(skeletal) < 0.3
        && nervous_to_trait(nervous) < 0.3
    {
        risks.push(crate::Mutation::RemoveField);
    }

    risks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_healthy_muscular() -> MuscularHealth {
        MuscularHealth {
            size_principle_compliant: true,
            antagonistic_pairs_defined: true,
            recruitment_balanced: true,
            fatigue_level: 0.1,
            cardiac_running: true,
        }
    }

    fn make_unhealthy_muscular() -> MuscularHealth {
        MuscularHealth {
            size_principle_compliant: false,
            antagonistic_pairs_defined: false,
            recruitment_balanced: false,
            fatigue_level: 0.9,
            cardiac_running: false,
        }
    }

    fn make_healthy_skeletal() -> SkeletalHealth {
        SkeletalHealth {
            claude_md_present: true,
            corrections_feeding_claude_md: true,
            settings_versioned: true,
            wolff_law_active: true,
        }
    }

    fn make_unhealthy_skeletal() -> SkeletalHealth {
        SkeletalHealth {
            claude_md_present: false,
            corrections_feeding_claude_md: false,
            settings_versioned: false,
            wolff_law_active: false,
        }
    }

    fn make_healthy_nervous() -> NervousHealth {
        NervousHealth {
            neuron_count: 20,
            reflex_arcs_active: 8,
            myelination_ratio: 0.9,
            avg_signal_latency_ms: 30.0,
            sensory_integration_ok: true,
        }
    }

    fn make_unhealthy_nervous() -> NervousHealth {
        NervousHealth {
            neuron_count: 1,
            reflex_arcs_active: 0,
            myelination_ratio: 0.1,
            avg_signal_latency_ms: 450.0,
            sensory_integration_ok: false,
        }
    }

    #[test]
    fn test_muscular_healthy_trait() {
        let health = make_healthy_muscular();
        let score = muscular_to_trait(&health);
        assert!(score > 0.8, "Healthy muscular should score >0.8: got {score}");
    }

    #[test]
    fn test_muscular_unhealthy_trait() {
        let health = make_unhealthy_muscular();
        let score = muscular_to_trait(&health);
        assert!(score < 0.1, "Unhealthy muscular should score <0.1: got {score}");
    }

    #[test]
    fn test_skeletal_healthy_trait() {
        let health = make_healthy_skeletal();
        let score = skeletal_to_trait(&health);
        assert!((score - 1.0).abs() < f64::EPSILON, "All-healthy skeletal should score 1.0: got {score}");
    }

    #[test]
    fn test_skeletal_unhealthy_trait() {
        let health = make_unhealthy_skeletal();
        let score = skeletal_to_trait(&health);
        assert!((score - 0.0).abs() < f64::EPSILON, "All-unhealthy skeletal should score 0.0: got {score}");
    }

    #[test]
    fn test_nervous_healthy_trait() {
        let health = make_healthy_nervous();
        let score = nervous_to_trait(&health);
        assert!(score > 0.8, "Healthy nervous should score >0.8: got {score}");
    }

    #[test]
    fn test_nervous_unhealthy_trait() {
        let health = make_unhealthy_nervous();
        let score = nervous_to_trait(&health);
        assert!(score < 0.3, "Unhealthy nervous should score <0.3: got {score}");
    }

    #[test]
    fn test_aggregate_healthy_phenotype() {
        let profile = aggregate_phenotype(
            &make_healthy_muscular(),
            &make_healthy_skeletal(),
            &make_healthy_nervous(),
        );
        assert!(profile.healthy, "All-healthy systems should produce healthy phenotype");
        assert!(profile.fitness > 0.8, "Fitness should be >0.8: got {}", profile.fitness);
        assert_eq!(profile.systems_assessed, 3);
    }

    #[test]
    fn test_aggregate_unhealthy_phenotype() {
        let profile = aggregate_phenotype(
            &make_unhealthy_muscular(),
            &make_unhealthy_skeletal(),
            &make_unhealthy_nervous(),
        );
        assert!(!profile.healthy, "All-unhealthy systems should produce unhealthy phenotype");
        assert!(profile.fitness < 0.2, "Fitness should be <0.2: got {}", profile.fitness);
    }

    #[test]
    fn test_phenotype_throughput_all_healthy() {
        let throughput = phenotype_throughput(
            &make_healthy_muscular(),
            &make_healthy_skeletal(),
            &make_healthy_nervous(),
        );
        assert_eq!(throughput, 3, "All healthy systems should have throughput 3");
    }

    #[test]
    fn test_phenotype_throughput_all_unhealthy() {
        let throughput = phenotype_throughput(
            &make_unhealthy_muscular(),
            &make_unhealthy_skeletal(),
            &make_unhealthy_nervous(),
        );
        assert_eq!(throughput, 0, "All unhealthy systems should have throughput 0");
    }

    #[test]
    fn test_health_to_mutation_risk_healthy() {
        let risks = health_to_mutation_risk(
            &make_healthy_muscular(),
            &make_healthy_skeletal(),
            &make_healthy_nervous(),
        );
        assert!(risks.is_empty(), "Healthy systems should have no mutation risks");
    }

    #[test]
    fn test_health_to_mutation_risk_unhealthy() {
        let risks = health_to_mutation_risk(
            &make_unhealthy_muscular(),
            &make_unhealthy_skeletal(),
            &make_unhealthy_nervous(),
        );
        assert!(risks.contains(&crate::Mutation::RangeExpand), "Weak muscular should risk RangeExpand");
        assert!(risks.contains(&crate::Mutation::StructureSwap), "Weak skeletal should risk StructureSwap");
        assert!(risks.contains(&crate::Mutation::TypeMismatch), "Weak nervous should risk TypeMismatch");
        assert!(risks.contains(&crate::Mutation::RemoveField), "All-weak should risk RemoveField");
    }

    #[test]
    fn test_mixed_health_partial_risks() {
        let risks = health_to_mutation_risk(
            &make_healthy_muscular(),
            &make_unhealthy_skeletal(),
            &make_healthy_nervous(),
        );
        assert_eq!(risks.len(), 1, "Only skeletal is weak, should have 1 risk");
        assert_eq!(risks[0], crate::Mutation::StructureSwap);
    }
}
