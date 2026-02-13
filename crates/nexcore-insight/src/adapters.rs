//! # Domain Adapters — Converting Domain Data to Insight Observations
//!
//! Stateless conversion layers that translate domain-specific data into
//! `Observation` objects ready for `NexCoreInsight::ingest_from()`.
//!
//! Each adapter is a zero-sized struct with associated functions — no state,
//! no dependencies on domain crates, no circular dependency risk.
//!
//! ## Architecture
//!
//! ```text
//!   PV Signal → PvInsightAdapter::from_signal() → Observation(tagged "pv")
//!   FAERS Data → FaersInsightAdapter::from_drug_events() → Observation(tagged "faers")
//!   Guardian → GuardianInsightAdapter::from_risk() → Observation(tagged "guardian")
//!   Brain → BrainInsightAdapter::from_artifact() → Observation(tagged "brain")
//! ```
//!
//! ## T1 Grounding
//! - μ (Mapping): Domain data → Observation conversion
//! - ∂ (Boundary): Each adapter defines its domain boundary
//! - σ (Sequence): Observations are temporally ordered by creation

use crate::engine::Observation;

// ============================================================================
// PV Insight Adapter — Signal Detection → Observations
// ============================================================================

/// Converts pharmacovigilance signal detection data into InsightEngine observations.
///
/// Maps PV concepts:
/// - Drug-event pair → key: `"drug:event"`, value: signal metric
/// - PRR/ROR/IC/EBGM → numeric observations with metric tags
/// - Signal flag → boolean observation with `"signal"` tag
///
/// ## Domain Tag: `"pv"`
///
/// Tier: T2-P (μ + ∂ — Mapping with PV domain boundary)
pub struct PvInsightAdapter;

impl PvInsightAdapter {
    /// Convert a signal detection result into observations.
    ///
    /// Produces up to 3 observations:
    /// 1. Drug-event pair observation (always)
    /// 2. PRR numeric observation (if > 0)
    /// 3. ROR numeric observation (if > 0)
    ///
    /// If `is_signal` is true, all observations get the `"signal"` tag.
    #[must_use]
    pub fn from_signal(
        drug: &str,
        event: &str,
        prr: f64,
        ror: f64,
        is_signal: bool,
    ) -> Vec<Observation> {
        let pair_key = format!("{drug}:{event}");
        let mut observations = Vec::new();

        // Primary drug-event pair observation
        let mut primary = Observation::new(
            &pair_key,
            if is_signal {
                "signal_detected"
            } else {
                "no_signal"
            },
        )
        .with_tag("pv")
        .with_tag("drug_event_pair");

        if is_signal {
            primary = primary.with_tag("signal");
        }
        observations.push(primary);

        // PRR metric observation
        if prr > 0.0 {
            let mut prr_obs = Observation::with_numeric(format!("{pair_key}:prr"), prr)
                .with_tag("pv")
                .with_tag("prr");
            if is_signal {
                prr_obs = prr_obs.with_tag("signal");
            }
            observations.push(prr_obs);
        }

        // ROR metric observation
        if ror > 0.0 {
            let mut ror_obs = Observation::with_numeric(format!("{pair_key}:ror"), ror)
                .with_tag("pv")
                .with_tag("ror");
            if is_signal {
                ror_obs = ror_obs.with_tag("signal");
            }
            observations.push(ror_obs);
        }

        observations
    }

    /// Convert a 2x2 contingency table into observations.
    ///
    /// The four cells of the disproportionality table:
    /// - a: Drug + Event (target)
    /// - b: Drug + No Event
    /// - c: No Drug + Event
    /// - d: No Drug + No Event
    #[must_use]
    pub fn from_contingency(
        drug: &str,
        event: &str,
        a: u64,
        b: u64,
        c: u64,
        d: u64,
    ) -> Vec<Observation> {
        let pair_key = format!("{drug}:{event}");
        let total = a + b + c + d;
        let expected = if total > 0 {
            ((a + b) as f64 * (a + c) as f64) / total as f64
        } else {
            0.0
        };

        vec![
            Observation::with_numeric(format!("{pair_key}:a"), a as f64)
                .with_tag("pv")
                .with_tag("contingency"),
            Observation::with_numeric(format!("{pair_key}:expected"), expected)
                .with_tag("pv")
                .with_tag("contingency"),
            Observation::with_numeric(format!("{pair_key}:total_n"), total as f64)
                .with_tag("pv")
                .with_tag("contingency"),
        ]
    }

    /// Convert a causality assessment into an observation.
    ///
    /// Maps Naranjo/WHO-UMC causality scores to numeric observations.
    #[must_use]
    pub fn from_causality(drug: &str, event: &str, score: f64, category: &str) -> Observation {
        Observation::with_numeric(format!("{drug}:{event}:causality"), score)
            .with_tag("pv")
            .with_tag("causality")
            .with_tag(category)
    }
}

// ============================================================================
// FAERS Insight Adapter — FDA Adverse Event Data → Observations
// ============================================================================

/// Converts FDA FAERS adverse event data into InsightEngine observations.
///
/// Maps FAERS concepts:
/// - Drug-event counts → numeric observations
/// - Signal velocity → rate-of-change observations (triggers Suddenness)
/// - Geographic patterns → location-tagged observations
///
/// ## Domain Tag: `"faers"`
///
/// Tier: T2-P (μ + ∂ — Mapping with FAERS domain boundary)
pub struct FaersInsightAdapter;

impl FaersInsightAdapter {
    /// Convert FAERS drug-event results into observations.
    ///
    /// Each (event_name, count) pair becomes a numeric observation
    /// tagged with `"faers"`.
    #[must_use]
    pub fn from_drug_events(drug: &str, events: &[(String, u64)]) -> Vec<Observation> {
        events
            .iter()
            .map(|(event, count)| {
                Observation::with_numeric(format!("{drug}:{event}"), *count as f64)
                    .with_tag("faers")
                    .with_tag("drug_event_count")
            })
            .collect()
    }

    /// Convert FAERS signal velocity into an observation.
    ///
    /// Velocity measures rate of new reports — high velocity may trigger
    /// Suddenness detection in the InsightEngine.
    #[must_use]
    pub fn from_signal_velocity(drug: &str, event: &str, velocity: f64) -> Observation {
        Observation::with_numeric(format!("{drug}:{event}:velocity"), velocity)
            .with_tag("faers")
            .with_tag("velocity")
    }

    /// Convert FAERS geographic divergence into observations.
    ///
    /// Each (region, divergence_score) becomes a location-tagged observation.
    #[must_use]
    pub fn from_geographic(drug: &str, regions: &[(String, f64)]) -> Vec<Observation> {
        regions
            .iter()
            .map(|(region, score)| {
                Observation::with_numeric(format!("{drug}:geo:{region}"), *score)
                    .with_tag("faers")
                    .with_tag("geographic")
                    .with_tag(region.as_str())
            })
            .collect()
    }

    /// Convert a FAERS polypharmacy result into observations.
    ///
    /// Maps co-prescribed drug combinations as related observations,
    /// enabling the InsightEngine to detect drug interaction patterns.
    #[must_use]
    pub fn from_polypharmacy(
        primary_drug: &str,
        concomitants: &[(String, u64)],
    ) -> Vec<Observation> {
        concomitants
            .iter()
            .map(|(drug, count)| {
                Observation::with_numeric(format!("{primary_drug}+{drug}"), *count as f64)
                    .with_tag("faers")
                    .with_tag("polypharmacy")
            })
            .collect()
    }
}

// ============================================================================
// Guardian Insight Adapter — Threat Sensing → Observations
// ============================================================================

/// Converts Guardian homeostasis/threat data into InsightEngine observations.
///
/// Maps Guardian concepts:
/// - Risk levels → numeric observations
/// - PAMP/DAMP signals → tagged observations
/// - Actuator actions → state-change observations
///
/// ## Domain Tag: `"guardian"`
///
/// Tier: T2-P (μ + ∂ — Mapping with Guardian domain boundary)
pub struct GuardianInsightAdapter;

impl GuardianInsightAdapter {
    /// Convert a Guardian risk evaluation into an observation.
    #[must_use]
    pub fn from_risk(entity: &str, risk_score: f64, risk_level: &str) -> Observation {
        Observation::with_numeric(format!("guardian:risk:{entity}"), risk_score)
            .with_tag("guardian")
            .with_tag("risk")
            .with_tag(risk_level)
    }

    /// Convert a PAMP detection into an observation.
    ///
    /// PAMPs (Pathogen-Associated Molecular Patterns) are external threat signals.
    #[must_use]
    pub fn from_pamp(threat_type: &str, severity: f64) -> Observation {
        Observation::with_numeric(format!("guardian:pamp:{threat_type}"), severity)
            .with_tag("guardian")
            .with_tag("pamp")
            .with_tag("threat")
    }

    /// Convert a DAMP detection into an observation.
    ///
    /// DAMPs (Damage-Associated Molecular Patterns) are internal distress signals.
    #[must_use]
    pub fn from_damp(indicator: &str, intensity: f64) -> Observation {
        Observation::with_numeric(format!("guardian:damp:{indicator}"), intensity)
            .with_tag("guardian")
            .with_tag("damp")
            .with_tag("internal")
    }

    /// Convert a homeostasis tick result into an observation.
    #[must_use]
    pub fn from_tick(iteration: u64, sensor_count: usize, actuator_count: usize) -> Observation {
        Observation::with_numeric("guardian:tick", iteration as f64)
            .with_tag("guardian")
            .with_tag("homeostasis")
            .with_tag(format!("sensors:{sensor_count}"))
            .with_tag(format!("actuators:{actuator_count}"))
    }
}

// ============================================================================
// Brain Insight Adapter — Learning/Memory → Observations
// ============================================================================

/// Converts Brain working memory data into InsightEngine observations.
///
/// Maps Brain concepts:
/// - Session artifacts → content observations
/// - Implicit learnings → preference observations
/// - Code tracking → change observations
///
/// ## Domain Tag: `"brain"`
///
/// Tier: T2-P (μ + ∂ — Mapping with Brain domain boundary)
pub struct BrainInsightAdapter;

impl BrainInsightAdapter {
    /// Convert an artifact save/resolve into an observation.
    #[must_use]
    pub fn from_artifact(name: &str, artifact_type: &str, size_bytes: u64) -> Observation {
        Observation::with_numeric(format!("brain:artifact:{name}"), size_bytes as f64)
            .with_tag("brain")
            .with_tag("artifact")
            .with_tag(artifact_type)
    }

    /// Convert an implicit learning event into an observation.
    #[must_use]
    pub fn from_implicit_learning(key: &str, value: &str) -> Observation {
        Observation::new(format!("brain:implicit:{key}"), value)
            .with_tag("brain")
            .with_tag("implicit")
    }

    /// Convert a code tracking change detection into an observation.
    #[must_use]
    pub fn from_code_change(file_path: &str, changed: bool) -> Observation {
        Observation::new(
            format!("brain:code:{file_path}"),
            if changed { "changed" } else { "unchanged" },
        )
        .with_tag("brain")
        .with_tag("code_tracking")
    }

    /// Convert a session milestone into an observation.
    #[must_use]
    pub fn from_session(session_id: &str, event_type: &str) -> Observation {
        Observation::new(format!("brain:session:{session_id}"), event_type)
            .with_tag("brain")
            .with_tag("session")
            .with_tag(event_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── PV Adapter Tests ────────────────────────────────────────────────

    #[test]
    fn test_pv_from_signal_with_signal() {
        let obs = PvInsightAdapter::from_signal("aspirin", "bleeding", 3.5, 4.2, true);
        assert_eq!(obs.len(), 3);
        assert_eq!(obs[0].key, "aspirin:bleeding");
        assert_eq!(obs[0].value, "signal_detected");
        assert!(obs[0].tags.contains(&"signal".to_string()));
        assert!(obs[0].tags.contains(&"pv".to_string()));
    }

    #[test]
    fn test_pv_from_signal_no_signal() {
        let obs = PvInsightAdapter::from_signal("aspirin", "headache", 1.2, 1.1, false);
        assert_eq!(obs[0].value, "no_signal");
        assert!(!obs[0].tags.contains(&"signal".to_string()));
    }

    #[test]
    fn test_pv_from_signal_zero_metrics() {
        let obs = PvInsightAdapter::from_signal("drug", "event", 0.0, 0.0, false);
        // Only the primary observation (no PRR/ROR when 0)
        assert_eq!(obs.len(), 1);
    }

    #[test]
    fn test_pv_from_contingency() {
        let obs = PvInsightAdapter::from_contingency("drug_a", "rash", 15, 100, 20, 10000);
        assert_eq!(obs.len(), 3);
        assert!(obs[0].tags.contains(&"contingency".to_string()));
        // a=15
        assert!((obs[0].numeric_value.unwrap_or(0.0) - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pv_from_causality() {
        let obs = PvInsightAdapter::from_causality("drug_x", "hepatitis", 7.0, "probable");
        assert_eq!(obs.key, "drug_x:hepatitis:causality");
        assert!(obs.tags.contains(&"causality".to_string()));
        assert!(obs.tags.contains(&"probable".to_string()));
        assert!((obs.numeric_value.unwrap_or(0.0) - 7.0).abs() < f64::EPSILON);
    }

    // ── FAERS Adapter Tests ─────────────────────────────────────────────

    #[test]
    fn test_faers_from_drug_events() {
        let events = vec![("nausea".to_string(), 500), ("headache".to_string(), 300)];
        let obs = FaersInsightAdapter::from_drug_events("ibuprofen", &events);
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].key, "ibuprofen:nausea");
        assert!((obs[0].numeric_value.unwrap_or(0.0) - 500.0).abs() < f64::EPSILON);
        assert!(obs[0].tags.contains(&"faers".to_string()));
    }

    #[test]
    fn test_faers_from_signal_velocity() {
        let obs = FaersInsightAdapter::from_signal_velocity("drug_a", "rash", 2.5);
        assert_eq!(obs.key, "drug_a:rash:velocity");
        assert!(obs.tags.contains(&"velocity".to_string()));
        assert!((obs.numeric_value.unwrap_or(0.0) - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_faers_from_geographic() {
        let regions = vec![("US".to_string(), 0.8), ("EU".to_string(), 0.3)];
        let obs = FaersInsightAdapter::from_geographic("drug_b", &regions);
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].key, "drug_b:geo:US");
        assert!(obs[0].tags.contains(&"geographic".to_string()));
    }

    #[test]
    fn test_faers_from_polypharmacy() {
        let concomitants = vec![
            ("metformin".to_string(), 150),
            ("lisinopril".to_string(), 90),
        ];
        let obs = FaersInsightAdapter::from_polypharmacy("aspirin", &concomitants);
        assert_eq!(obs.len(), 2);
        assert_eq!(obs[0].key, "aspirin+metformin");
        assert!(obs[0].tags.contains(&"polypharmacy".to_string()));
    }

    // ── Guardian Adapter Tests ──────────────────────────────────────────

    #[test]
    fn test_guardian_from_risk() {
        let obs = GuardianInsightAdapter::from_risk("entity_x", 0.85, "high");
        assert_eq!(obs.key, "guardian:risk:entity_x");
        assert!(obs.tags.contains(&"guardian".to_string()));
        assert!(obs.tags.contains(&"risk".to_string()));
        assert!(obs.tags.contains(&"high".to_string()));
    }

    #[test]
    fn test_guardian_from_pamp() {
        let obs = GuardianInsightAdapter::from_pamp("brute_force", 0.9);
        assert_eq!(obs.key, "guardian:pamp:brute_force");
        assert!(obs.tags.contains(&"pamp".to_string()));
        assert!(obs.tags.contains(&"threat".to_string()));
    }

    #[test]
    fn test_guardian_from_damp() {
        let obs = GuardianInsightAdapter::from_damp("memory_pressure", 0.7);
        assert_eq!(obs.key, "guardian:damp:memory_pressure");
        assert!(obs.tags.contains(&"damp".to_string()));
        assert!(obs.tags.contains(&"internal".to_string()));
    }

    #[test]
    fn test_guardian_from_tick() {
        let obs = GuardianInsightAdapter::from_tick(42, 3, 2);
        assert_eq!(obs.key, "guardian:tick");
        assert!((obs.numeric_value.unwrap_or(0.0) - 42.0).abs() < f64::EPSILON);
        assert!(obs.tags.contains(&"homeostasis".to_string()));
    }

    // ── Brain Adapter Tests ─────────────────────────────────────────────

    #[test]
    fn test_brain_from_artifact() {
        let obs = BrainInsightAdapter::from_artifact("task.md", "plan", 2048);
        assert_eq!(obs.key, "brain:artifact:task.md");
        assert!(obs.tags.contains(&"brain".to_string()));
        assert!(obs.tags.contains(&"artifact".to_string()));
        assert!(obs.tags.contains(&"plan".to_string()));
    }

    #[test]
    fn test_brain_from_implicit_learning() {
        let obs = BrainInsightAdapter::from_implicit_learning("indent_style", "4_spaces");
        assert_eq!(obs.key, "brain:implicit:indent_style");
        assert_eq!(obs.value, "4_spaces");
        assert!(obs.tags.contains(&"implicit".to_string()));
    }

    #[test]
    fn test_brain_from_code_change() {
        let obs = BrainInsightAdapter::from_code_change("src/lib.rs", true);
        assert_eq!(obs.key, "brain:code:src/lib.rs");
        assert_eq!(obs.value, "changed");
        assert!(obs.tags.contains(&"code_tracking".to_string()));
    }

    #[test]
    fn test_brain_from_session() {
        let obs = BrainInsightAdapter::from_session("abc123", "created");
        assert_eq!(obs.key, "brain:session:abc123");
        assert!(obs.tags.contains(&"session".to_string()));
        assert!(obs.tags.contains(&"created".to_string()));
    }

    // ── Integration: Adapter → NexCoreInsight Pipeline ──────────────────

    #[test]
    fn test_pv_adapter_to_compositor() {
        use crate::orchestrator::NexCoreInsight;
        use crate::traits::Insight;

        let mut nc = NexCoreInsight::new();
        nc.register_domain("pv", "Signal detection");

        let observations = PvInsightAdapter::from_signal("aspirin", "bleeding", 3.5, 4.2, true);
        for obs in observations {
            let _events = nc.ingest_from("pv", obs);
        }

        assert_eq!(nc.domain_observation_count("pv"), 3);
        assert_eq!(nc.observation_count(), 3);
    }

    #[test]
    fn test_faers_adapter_to_compositor() {
        use crate::orchestrator::NexCoreInsight;
        use crate::traits::Insight;

        let mut nc = NexCoreInsight::new();
        nc.register_domain("faers", "FDA adverse events");

        let events = vec![("nausea".to_string(), 500), ("headache".to_string(), 300)];
        let observations = FaersInsightAdapter::from_drug_events("ibuprofen", &events);
        for obs in observations {
            let _events = nc.ingest_from("faers", obs);
        }

        assert_eq!(nc.domain_observation_count("faers"), 2);
    }

    #[test]
    fn test_multi_adapter_cross_domain() {
        use crate::engine::InsightConfig;
        use crate::orchestrator::NexCoreInsight;
        use crate::traits::Insight;

        let mut nc = NexCoreInsight::with_config(InsightConfig {
            pattern_min_occurrences: 2,
            ..InsightConfig::default()
        });
        nc.register_domain("pv", "Signal detection");
        nc.register_domain("faers", "FDA adverse events");

        // Both domains report on the same drug-event pair
        let pv_obs = PvInsightAdapter::from_signal("aspirin", "bleeding", 3.5, 4.2, true);
        for obs in pv_obs {
            let _e = nc.ingest_from("pv", obs);
        }

        let faers_obs =
            FaersInsightAdapter::from_drug_events("aspirin", &[("bleeding".to_string(), 500)]);
        for obs in faers_obs {
            let _e = nc.ingest_from("faers", obs);
        }

        // Observations from both domains are in the unified engine
        assert_eq!(nc.observation_count(), 4);
        let summary = nc.summary();
        assert_eq!(summary.domain_count, 2);
    }
}
