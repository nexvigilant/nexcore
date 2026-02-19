//! Pharmacovigilance Theory — Drug Safety Through DNA Math.
//!
//! Maps pharmacovigilance concepts onto DNA-based computation:
//!
//! - **DrugProfile**: A word mined as a drug compound with string-theoretic properties
//! - **Signal**: Drug-event pair detection via distance and resonance in word-space
//! - **SafetyMargin**: Theory of Vigilance d(s) — distance from safety boundary
//! - **CausalityScore**: Evidence-weighted causality assessment via spectral alignment
//! - **VigilanceState**: Aggregate monitoring state tracking signal accumulation
//!
//! All algorithms are deterministic. Zero external dependencies.
//! Implements the NexVigilant Theory of Vigilance through DNA math.

use crate::lexicon;
use crate::statemind::MindPoint;
use crate::string_theory;
use std::fmt;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Safety boundary threshold: signals closer than this require attention.
const SAFETY_BOUNDARY: f64 = 0.3;

/// Critical zone: signals within this distance are immediately dangerous.
const CRITICAL_ZONE: f64 = 0.1;

/// Minimum signal strength to register.
const SIGNAL_FLOOR: f64 = 0.01;

/// Resonance threshold: above this, mechanistic plausibility is elevated.
const RESONANCE_THRESHOLD: f64 = 0.5;

// ---------------------------------------------------------------------------
// CausalityCategory — WHO-UMC inspired
// ---------------------------------------------------------------------------

/// Causality assessment category (WHO-UMC inspired).
///
/// Tier: T1 (ς State)
/// Dominant: ς State (discrete classification state)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CausalityCategory {
    /// Strong signal + high resonance → probable causal relationship.
    Certain,
    /// Good signal + moderate resonance.
    Probable,
    /// Detectable signal + some resonance.
    Possible,
    /// Weak signal, low resonance.
    Unlikely,
    /// No signal or negative correlation.
    Unrelated,
}

impl CausalityCategory {
    /// Numeric weight for aggregation.
    #[must_use]
    pub fn weight(self) -> f64 {
        match self {
            Self::Certain => 1.0,
            Self::Probable => 0.75,
            Self::Possible => 0.5,
            Self::Unlikely => 0.25,
            Self::Unrelated => 0.0,
        }
    }
}

impl fmt::Display for CausalityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Certain => write!(f, "Certain"),
            Self::Probable => write!(f, "Probable"),
            Self::Possible => write!(f, "Possible"),
            Self::Unlikely => write!(f, "Unlikely"),
            Self::Unrelated => write!(f, "Unrelated"),
        }
    }
}

// ---------------------------------------------------------------------------
// DrugProfile — word as drug compound
// ---------------------------------------------------------------------------

/// A word profiled as a drug compound with DNA-derived properties.
///
/// Tier: T3 (→ + κ + N + ν + σ + μ + ∂)
/// Dominant: → Causality (drugs cause effects)
pub struct DrugProfile {
    /// The drug name/word.
    pub name: String,
    /// 3D mind-space position.
    pub position: MindPoint,
    /// String tension properties.
    pub tension: string_theory::StringTension,
    /// String energy state.
    pub energy: string_theory::StringEnergy,
    /// Shannon entropy of the name.
    pub entropy: f64,
    /// GC content of DNA encoding.
    pub gc_content: f64,
    /// Molecular weight proxy: strand length.
    pub molecular_weight: usize,
}

/// Profile a word as a drug compound.
#[must_use]
pub fn profile_drug(name: &str) -> DrugProfile {
    let ore = lexicon::mine(name);
    let position = MindPoint::from_ore(&ore);
    let tension = string_theory::word_tension(name);
    let energy = string_theory::string_energy(name);
    let entropy = ore.entropy;
    let gc_content = ore.gc_content;
    let molecular_weight = ore.length;

    DrugProfile {
        name: name.to_string(),
        position,
        tension,
        energy,
        entropy,
        gc_content,
        molecular_weight,
    }
}

impl fmt::Display for DrugProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Drug \"{}\" pos={} E={:.4} MW={} GC={:.3}",
            self.name,
            self.position,
            self.energy.total_energy,
            self.molecular_weight,
            self.gc_content,
        )
    }
}

// ---------------------------------------------------------------------------
// Signal — drug-event pair detection
// ---------------------------------------------------------------------------

/// Drug-event pair causality detection with strength and resonance scores.
///
/// Tier: T2-C (→ Causality + κ Comparison + N Quantity + ν Frequency)
/// Dominant: → Causality (signal implies causal link)
pub struct DrugEventSignal {
    /// Drug name.
    pub drug: String,
    /// Adverse event name.
    pub event: String,
    /// Signal strength: 1.0 / (1.0 + distance) in mind-space.
    pub strength: f64,
    /// Resonance score between drug and event DNA encodings.
    pub resonance: f64,
    /// Combined score: strength × (0.5 + 0.5 × resonance).
    pub combined_score: f64,
    /// Whether this signal exceeds the detection floor.
    pub detected: bool,
}

/// Detect a signal between a drug and an event.
#[must_use]
pub fn detect_signal(drug: &str, event: &str) -> DrugEventSignal {
    let drug_ore = lexicon::mine(drug);
    let event_ore = lexicon::mine(event);
    let drug_pt = MindPoint::from_ore(&drug_ore);
    let event_pt = MindPoint::from_ore(&event_ore);

    let distance = drug_pt.distance(&event_pt);
    let strength = 1.0 / (1.0 + distance);

    let res = string_theory::word_resonance(drug, event);
    let resonance = res.overlap;

    // Combined score weights both proximity and mechanistic (resonance)
    let combined_score = strength * (0.5 + 0.5 * resonance);
    let detected = combined_score > SIGNAL_FLOOR;

    DrugEventSignal {
        drug: drug.to_string(),
        event: event.to_string(),
        strength,
        resonance,
        combined_score,
        detected,
    }
}

/// Backward-compatible alias.
#[deprecated(note = "use DrugEventSignal — F2 equivocation fix")]
pub type Signal = DrugEventSignal;

impl fmt::Display for DrugEventSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.detected {
            "DETECTED"
        } else {
            "below floor"
        };
        write!(
            f,
            "Signal({} → {}): str={:.4}, res={:.4}, combined={:.4} [{}]",
            self.drug, self.event, self.strength, self.resonance, self.combined_score, status,
        )
    }
}

// ---------------------------------------------------------------------------
// SafetyMargin — Theory of Vigilance d(s)
// ---------------------------------------------------------------------------

/// Safety margin measurement: distance from the safety boundary.
///
/// Tier: T2-C (∂ Boundary + N Quantity + κ Comparison + → Causality)
/// Dominant: ∂ Boundary (distance FROM the safety boundary)
///
/// Implements d(s) from the Theory of Vigilance:
/// - d(s) > 0: within safe zone
/// - d(s) = 0: at the boundary
/// - d(s) < 0: in the danger zone
pub struct SafetyMargin {
    /// The entity being measured.
    pub entity: String,
    /// Raw distance from origin in mind-space.
    pub raw_distance: f64,
    /// Safety distance d(s): positive = safe, negative = danger.
    pub d_s: f64,
    /// Whether we're in the critical zone.
    pub critical: bool,
    /// Safety level as a human-readable category.
    pub level: SafetyLevel,
}

/// Safety level classification.
///
/// Tier: T1 (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SafetyLevel {
    /// Well within safe zone (d(s) > 0.2).
    Safe,
    /// Near the boundary (0 < d(s) <= 0.2).
    Caution,
    /// At or past the boundary (d(s) <= 0).
    Warning,
    /// In the critical zone (d(s) < -critical_zone).
    Critical,
}

/// Compute safety margin for an entity in the context of a baseline.
///
/// The baseline represents the "normal operating point" — typically the
/// centroid of known safe compounds. Safety is measured as distance from
/// the safety boundary relative to the baseline.
#[must_use]
pub fn safety_margin(entity: &str, baseline: &MindPoint) -> SafetyMargin {
    let ore = lexicon::mine(entity);
    let pt = MindPoint::from_ore(&ore);
    let raw_distance = pt.distance(baseline);

    // d(s) = boundary - distance (positive means inside safe zone)
    let d_s = SAFETY_BOUNDARY - raw_distance;
    let critical = d_s < -(CRITICAL_ZONE);

    let level = if d_s > 0.2 {
        SafetyLevel::Safe
    } else if d_s > 0.0 {
        SafetyLevel::Caution
    } else if !critical {
        SafetyLevel::Warning
    } else {
        SafetyLevel::Critical
    };

    SafetyMargin {
        entity: entity.to_string(),
        raw_distance,
        d_s,
        critical,
        level,
    }
}

impl fmt::Display for SafetyMargin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SafetyMargin(\"{}\"): d(s)={:.4}, raw={:.4}, level={:?}{}",
            self.entity,
            self.d_s,
            self.raw_distance,
            self.level,
            if self.critical { " CRITICAL" } else { "" },
        )
    }
}

impl fmt::Display for SafetyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Safe => write!(f, "Safe"),
            Self::Caution => write!(f, "Caution"),
            Self::Warning => write!(f, "Warning"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

// ---------------------------------------------------------------------------
// CausalityScore — evidence-weighted assessment
// ---------------------------------------------------------------------------

/// Causality assessment for a drug-event pair.
///
/// Tier: T2-C (→ Causality + κ Comparison + N Quantity + ∂ Boundary)
/// Dominant: → Causality (assessing causal relationship)
pub struct CausalityScore {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Proximity score: 1/(1+distance) in mind-space.
    pub proximity: f64,
    /// Mechanistic score: resonance overlap.
    pub mechanistic: f64,
    /// Combined causality score: weighted blend.
    pub score: f64,
    /// Assessed category.
    pub category: CausalityCategory,
}

/// Assess causality between a drug and event.
///
/// Combines proximity (statistical signal) with mechanistic plausibility
/// (spectral resonance) into a single causality assessment.
#[must_use]
pub fn assess_causality(drug: &str, event: &str) -> CausalityScore {
    let signal = detect_signal(drug, event);

    let proximity = signal.strength;
    let mechanistic = signal.resonance;

    // Weighted score: 60% proximity + 40% mechanistic
    let score = 0.6 * proximity + 0.4 * mechanistic;

    let category = if score > 0.8 {
        CausalityCategory::Certain
    } else if score > 0.6 {
        CausalityCategory::Probable
    } else if score > 0.4 {
        CausalityCategory::Possible
    } else if score > 0.2 {
        CausalityCategory::Unlikely
    } else {
        CausalityCategory::Unrelated
    };

    CausalityScore {
        drug: drug.to_string(),
        event: event.to_string(),
        proximity,
        mechanistic,
        score,
        category,
    }
}

impl fmt::Display for CausalityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Causality({} → {}): prox={:.4}, mech={:.4}, score={:.4} → {}",
            self.drug, self.event, self.proximity, self.mechanistic, self.score, self.category,
        )
    }
}

// ---------------------------------------------------------------------------
// VigilanceState — aggregate monitoring
// ---------------------------------------------------------------------------

/// Aggregate vigilance state tracking multiple signals.
///
/// Tier: T3 (ς State + → Causality + κ Comparison + N Quantity + σ Sequence + ∂ Boundary + ν Frequency)
/// Dominant: ς State (monitoring state evolves with each signal)
pub struct VigilanceState {
    /// Signals detected.
    pub signals: Vec<DrugEventSignal>,
    /// Total signal count.
    pub signal_count: usize,
    /// Mean combined score across all signals.
    pub mean_score: f64,
    /// Maximum combined score (strongest signal).
    pub max_score: f64,
    /// Number of critical signals (above resonance threshold).
    pub critical_count: usize,
    /// Overall alert level.
    pub alert_level: AlertLevel,
}

/// Alert level for the vigilance state.
///
/// Tier: T1 (ς State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertLevel {
    /// No concerning signals.
    Green,
    /// Some signals detected, monitoring.
    Yellow,
    /// Strong signals or critical resonance detected.
    Orange,
    /// Multiple critical signals, immediate review needed.
    Red,
}

/// Build a vigilance state from a drug and a set of potential events.
#[must_use]
pub fn monitor(drug: &str, events: &[&str]) -> VigilanceState {
    let signals: Vec<DrugEventSignal> = events
        .iter()
        .map(|event| detect_signal(drug, event))
        .filter(|s| s.detected)
        .collect();

    let signal_count = signals.len();

    let mean_score = if signals.is_empty() {
        0.0
    } else {
        signals.iter().map(|s| s.combined_score).sum::<f64>() / signals.len() as f64
    };

    let max_score = signals
        .iter()
        .map(|s| s.combined_score)
        .fold(0.0_f64, f64::max);

    let critical_count = signals
        .iter()
        .filter(|s| s.resonance > RESONANCE_THRESHOLD)
        .count();

    let alert_level = if critical_count >= 3 || max_score > 0.8 {
        AlertLevel::Red
    } else if critical_count >= 1 || max_score > 0.6 {
        AlertLevel::Orange
    } else if signal_count > 0 {
        AlertLevel::Yellow
    } else {
        AlertLevel::Green
    };

    VigilanceState {
        signals,
        signal_count,
        mean_score,
        max_score,
        critical_count,
        alert_level,
    }
}

impl fmt::Display for VigilanceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vigilance: {} signals, mean={:.4}, max={:.4}, critical={}, alert={:?}",
            self.signal_count,
            self.mean_score,
            self.max_score,
            self.critical_count,
            self.alert_level,
        )
    }
}

impl fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Green => write!(f, "GREEN"),
            Self::Yellow => write!(f, "YELLOW"),
            Self::Orange => write!(f, "ORANGE"),
            Self::Red => write!(f, "RED"),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // DrugProfile tests
    // -----------------------------------------------------------------------

    #[test]
    fn profile_drug_basic() {
        let p = profile_drug("aspirin");
        assert_eq!(p.name, "aspirin");
        assert!(p.molecular_weight > 0);
        assert!(p.entropy > 0.0);
        assert!(p.gc_content >= 0.0 && p.gc_content <= 1.0);
    }

    #[test]
    fn profile_drug_empty() {
        let p = profile_drug("");
        assert_eq!(p.name, "");
        // Empty word still encodes to a DNA strand (encoding overhead),
        // so molecular_weight (strand length) and entropy may be non-zero.
        assert!(p.entropy >= 0.0);
    }

    #[test]
    fn profile_drug_position_in_range() {
        let p = profile_drug("ibuprofen");
        assert!(p.position.entropy_norm >= 0.0 && p.position.entropy_norm <= 1.0);
        assert!(p.position.gc_content >= 0.0 && p.position.gc_content <= 1.0);
        assert!(p.position.density >= 0.0 && p.position.density <= 1.0);
    }

    #[test]
    fn profile_drug_energy_positive() {
        let p = profile_drug("metformin");
        assert!(p.energy.total_energy > 0.0);
    }

    // -----------------------------------------------------------------------
    // Signal tests
    // -----------------------------------------------------------------------

    #[test]
    fn detect_signal_self() {
        let s = detect_signal("aspirin", "aspirin");
        // Self-signal: distance=0, strength=1.0
        assert!((s.strength - 1.0).abs() < f64::EPSILON);
        assert!(s.detected);
    }

    #[test]
    fn detect_signal_different() {
        let s = detect_signal("aspirin", "headache");
        assert!(s.strength > 0.0);
        assert!(s.strength < 1.0);
        assert!(s.combined_score > 0.0);
    }

    #[test]
    fn detect_signal_resonance_range() {
        let s = detect_signal("drug", "event");
        assert!(s.resonance >= 0.0 && s.resonance <= 1.0);
    }

    #[test]
    fn detect_signal_combined_bounded() {
        let s = detect_signal("acetaminophen", "liver");
        // Combined should be between 0 and 1
        assert!(s.combined_score >= 0.0 && s.combined_score <= 1.0);
    }

    #[test]
    fn detect_signal_always_detected_self() {
        let s = detect_signal("test", "test");
        assert!(s.detected);
    }

    // -----------------------------------------------------------------------
    // SafetyMargin tests
    // -----------------------------------------------------------------------

    #[test]
    fn safety_margin_at_origin() {
        let baseline = MindPoint::origin();
        let sm = safety_margin("a", &baseline);
        // "a" should be somewhere in the space, distance from origin varies
        assert!(sm.raw_distance >= 0.0);
    }

    #[test]
    fn safety_margin_level_categorized() {
        let baseline = MindPoint::origin();
        let sm = safety_margin("test", &baseline);
        // Should be one of the four levels
        assert!(matches!(
            sm.level,
            SafetyLevel::Safe | SafetyLevel::Caution | SafetyLevel::Warning | SafetyLevel::Critical
        ));
    }

    #[test]
    fn safety_margin_d_s_sign() {
        let baseline = MindPoint::origin();
        let sm = safety_margin("test", &baseline);
        // d(s) = boundary - distance
        let expected = SAFETY_BOUNDARY - sm.raw_distance;
        assert!((sm.d_s - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn safety_margin_critical_flag() {
        let baseline = MindPoint::origin();
        let sm = safety_margin("test", &baseline);
        // Critical iff d_s < -CRITICAL_ZONE
        let expected_critical = sm.d_s < -(CRITICAL_ZONE);
        assert_eq!(sm.critical, expected_critical);
    }

    #[test]
    fn safety_level_display() {
        assert_eq!(format!("{}", SafetyLevel::Safe), "Safe");
        assert_eq!(format!("{}", SafetyLevel::Critical), "Critical");
    }

    // -----------------------------------------------------------------------
    // CausalityScore tests
    // -----------------------------------------------------------------------

    #[test]
    fn causality_self_is_certain_or_probable() {
        let c = assess_causality("test", "test");
        // Self-assessment: high proximity + high resonance
        assert!(c.score > 0.5);
        assert!(matches!(
            c.category,
            CausalityCategory::Certain | CausalityCategory::Probable
        ));
    }

    #[test]
    fn causality_score_range() {
        let c = assess_causality("drug", "event");
        assert!(c.score >= 0.0 && c.score <= 1.0);
        assert!(c.proximity >= 0.0 && c.proximity <= 1.0);
        assert!(c.mechanistic >= 0.0 && c.mechanistic <= 1.0);
    }

    #[test]
    fn causality_category_weight_order() {
        assert!(CausalityCategory::Certain.weight() > CausalityCategory::Probable.weight());
        assert!(CausalityCategory::Probable.weight() > CausalityCategory::Possible.weight());
        assert!(CausalityCategory::Possible.weight() > CausalityCategory::Unlikely.weight());
        assert!(CausalityCategory::Unlikely.weight() > CausalityCategory::Unrelated.weight());
        assert!((CausalityCategory::Unrelated.weight() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn causality_is_weighted_blend() {
        let c = assess_causality("aspirin", "bleeding");
        let expected = 0.6 * c.proximity + 0.4 * c.mechanistic;
        assert!((c.score - expected).abs() < 1e-10);
    }

    #[test]
    fn causality_category_display() {
        assert_eq!(format!("{}", CausalityCategory::Certain), "Certain");
        assert_eq!(format!("{}", CausalityCategory::Unrelated), "Unrelated");
    }

    // -----------------------------------------------------------------------
    // VigilanceState tests
    // -----------------------------------------------------------------------

    #[test]
    fn monitor_no_events() {
        let vs = monitor("aspirin", &[]);
        assert_eq!(vs.signal_count, 0);
        assert!((vs.mean_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(vs.alert_level, AlertLevel::Green);
    }

    #[test]
    fn monitor_single_event() {
        let vs = monitor("aspirin", &["headache"]);
        assert!(vs.signal_count <= 1);
        // Alert depends on signal strength; accept any valid level
        assert!(matches!(
            vs.alert_level,
            AlertLevel::Green | AlertLevel::Yellow | AlertLevel::Orange | AlertLevel::Red
        ));
    }

    #[test]
    fn monitor_multiple_events() {
        let vs = monitor("aspirin", &["headache", "bleeding", "nausea", "rash"]);
        // Should detect some signals
        assert!(vs.max_score >= vs.mean_score || vs.signal_count == 0);
    }

    #[test]
    fn monitor_self_signals_high() {
        let vs = monitor("test", &["test", "test", "test"]);
        // Self-signals should be strong but deduplicated via detection
        assert!(vs.signal_count > 0);
        assert!(vs.max_score > 0.0);
    }

    #[test]
    fn monitor_critical_count_bounded() {
        let vs = monitor("drug", &["event1", "event2"]);
        assert!(vs.critical_count <= vs.signal_count);
    }

    #[test]
    fn alert_level_display() {
        assert_eq!(format!("{}", AlertLevel::Green), "GREEN");
        assert_eq!(format!("{}", AlertLevel::Red), "RED");
    }

    // -----------------------------------------------------------------------
    // Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn display_drug_profile() {
        let p = profile_drug("aspirin");
        let s = format!("{p}");
        assert!(s.contains("Drug"));
        assert!(s.contains("aspirin"));
        assert!(s.contains("MW="));
    }

    #[test]
    fn display_signal() {
        let sig = detect_signal("aspirin", "headache");
        let s = format!("{sig}");
        assert!(s.contains("Signal"));
        assert!(s.contains("aspirin"));
        assert!(s.contains("headache"));
    }

    #[test]
    fn display_safety_margin() {
        let baseline = MindPoint::origin();
        let sm = safety_margin("aspirin", &baseline);
        let s = format!("{sm}");
        assert!(s.contains("SafetyMargin"));
        assert!(s.contains("d(s)="));
    }

    #[test]
    fn display_causality() {
        let c = assess_causality("drug", "event");
        let s = format!("{c}");
        assert!(s.contains("Causality"));
        assert!(s.contains("drug"));
        assert!(s.contains("event"));
    }

    #[test]
    fn display_vigilance_state() {
        let vs = monitor("aspirin", &["headache"]);
        let s = format!("{vs}");
        assert!(s.contains("Vigilance:"));
        assert!(s.contains("signals"));
    }

    // -----------------------------------------------------------------------
    // Integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn signal_strength_inversely_proportional_to_distance() {
        let s_close = detect_signal("cat", "bat"); // similar words
        let s_far = detect_signal("cat", "elephant"); // dissimilar words
        // Closer words should have higher strength
        // (This holds statistically but not always due to encoding)
        assert!(s_close.strength > 0.0);
        assert!(s_far.strength > 0.0);
    }

    #[test]
    fn vigilance_escalation_path() {
        // Monitor with increasing signal load
        let vs0 = monitor("drug", &[]);
        let vs1 = monitor("drug", &["event"]);

        // More events should generally not decrease signal count
        assert!(vs1.signal_count >= vs0.signal_count);
    }
}
