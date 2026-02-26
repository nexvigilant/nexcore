//! NexCloud Scientific & Technical Foundations Module
//!
//! This module encodes the scientific, computing, and engineering principles
//! that ground NexCloud's architecture. Where `ethics.rs` answers *why* NexCloud
//! behaves as it does, `foundations.rs` answers *how* — and traces every
//! implementation decision to established scientific and engineering discipline.
//!
//! ## Three Pillars
//!
//! | Pillar | Discipline | NexCloud Manifestation |
//! |--------|-----------|----------------------|
//! | **Science** | Thermodynamics | Resource conservation — CPU/memory tracked, never leaked |
//! | | Systems Biology | Homeostasis — health monitor loop (SENSE→COMPARE→ACT) |
//! | | Epidemiology | Signal detection — health checks as adverse event surveillance |
//! | | Information Theory | Channel capacity — bounded EventBus (512), log entropy |
//! | **Computing** | Finite State Machines | ProcessState: 8-state lifecycle FSM |
//! | | Message Passing | EventBus: tokio broadcast channels, no shared mutable state |
//! | | Concurrent Data Structures | ServiceRegistry: DashMap lock-free concurrent map |
//! | | Graph Algorithms | Topological sort for dependency ordering |
//! | | Exponential Backoff | RestartPolicy: geometric retry with cap |
//! | **Technology** | Defense in Depth | TLS + input validation + PID verification + permissions |
//! | | Separation of Concerns | Supervisor / Proxy / Health as independent subsystems |
//! | | Graceful Degradation | SIGTERM → wait → SIGKILL escalation |
//! | | Fail-Fast | Manifest validation before any process spawns |
//! | | Least Privilege | 0600 PID files, 0640 log files, no arbitrary execution |
//!
//! ## Scientific Grounding
//!
//! NexCloud is not merely *inspired* by science — its core abstractions are
//! isomorphic to scientific principles:
//!
//! | Scientific Law | NexCloud Implementation |
//! |---------------|------------------------|
//! | First Law of Thermodynamics (conservation) | Resources allocated = resources tracked. No phantom processes. |
//! | Second Law (entropy increases) | Log files grow; we bound them. EventBus has capacity limits. |
//! | Le Chatelier's Principle (equilibrium) | Health monitor restores equilibrium via restart on failure. |
//! | Koch's Postulates (causal proof) | Health check: isolate service, probe endpoint, confirm state. |
//! | Shannon's Noisy Channel Theorem | Bounded channel capacity (512 events), explicit error types. |
//!
//! ## Computing Science Grounding
//!
//! | CS Concept | Implementation | Location |
//! |-----------|---------------|----------|
//! | FSM (Hopcroft & Ullman) | ProcessState enum, 8 states | `supervisor/registry.rs` |
//! | CSP (Hoare, 1978) | EventBus broadcast channels | `events.rs` |
//! | Topological Sort (in-degree reduction) | Dependency ordering | `manifest.rs` |
//! | Exponential Backoff (Ethernet, 1976) | RestartPolicy with 2x multiplier | `process/restart.rs` |
//! | DashMap (lock-free hashing) | ServiceRegistry concurrent access | `supervisor/registry.rs` |
//! | Proxy Pattern (GoF) | ReverseProxy request forwarding | `proxy/mod.rs` |
//!
//! ## Engineering Standards
//!
//! | Standard | Application |
//! |----------|------------|
//! | POSIX signals (IEEE 1003.1) | SIGTERM/SIGKILL process lifecycle |
//! | TLS 1.2+ (RFC 8446) | rustls with safe defaults, no weak ciphers |
//! | HTTP/1.1 (RFC 9110) | hyper-based reverse proxy |
//! | TOML (RFC-like spec) | Manifest configuration format |
//! | Unix permissions (POSIX) | 0600 PID, 0640 logs — principle of least privilege |
//!
//! Tier: T3 (κ Comparison + ν Frequency + → Causality + ς State + μ Mapping +
//!           σ Sequence + ρ Recursion + ∂ Boundary + π Persistence + ∝ Irreversibility)

/// Scientific disciplines that ground NexCloud's architecture.
///
/// Tier: T2-P (κ Comparison) — each discipline provides a comparison methodology
/// against which system behavior is measured.
///
/// These are not metaphors — they are isomorphisms. The health monitor IS a
/// homeostasis loop. The EventBus IS a bounded channel. The restart policy
/// IS exponential backoff from network science.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScientificDiscipline {
    /// Thermodynamics: Conservation of resources, entropy management.
    ///
    /// First Law: Resources allocated = resources tracked. No phantom processes.
    /// Second Law: Entropy (log growth, state drift) is bounded by capacity limits.
    Thermodynamics,

    /// Systems Biology: Homeostasis feedback loops.
    ///
    /// The health monitor implements SENSE (probe) → COMPARE (threshold) → ACT (restart).
    /// Le Chatelier's principle: perturbation triggers restoration of equilibrium.
    SystemsBiology,

    /// Epidemiology: Signal detection and surveillance methodology.
    ///
    /// Health checks are adverse event surveillance. Unhealthy status is a detected signal.
    /// Koch's postulates: isolate service, probe endpoint, confirm failure, verify causation.
    Epidemiology,

    /// Information Theory: Channel capacity and entropy.
    ///
    /// Shannon's theorem: EventBus bounded to 512 events (channel capacity).
    /// Explicit error types reduce information entropy in failure paths.
    InformationTheory,

    /// Control Theory: Feedback and stability.
    ///
    /// PID-like control: measure health (proportional), track restart count (integral),
    /// backoff rate change (derivative). System converges to stable state or fails explicitly.
    ControlTheory,
}

impl ScientificDiscipline {
    /// All scientific disciplines grounding the system.
    pub const ALL: &'static [ScientificDiscipline] = &[
        ScientificDiscipline::Thermodynamics,
        ScientificDiscipline::SystemsBiology,
        ScientificDiscipline::Epidemiology,
        ScientificDiscipline::InformationTheory,
        ScientificDiscipline::ControlTheory,
    ];
}

impl std::fmt::Display for ScientificDiscipline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thermodynamics => write!(f, "thermodynamics"),
            Self::SystemsBiology => write!(f, "systems biology"),
            Self::Epidemiology => write!(f, "epidemiology"),
            Self::InformationTheory => write!(f, "information theory"),
            Self::ControlTheory => write!(f, "control theory"),
        }
    }
}

/// Computing paradigms implemented in NexCloud.
///
/// Tier: T2-P (μ Mapping) — each paradigm maps abstract CS concepts to concrete
/// Rust implementations.
///
/// Every paradigm listed here has a specific, traceable implementation in the codebase.
/// No aspirational entries — only verified groundings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputingParadigm {
    /// Finite State Machine (Hopcroft & Ullman).
    ///
    /// ProcessState: Pending → Starting → Healthy → Unhealthy → Restarting → Stopping → Stopped → Failed.
    /// Grounded in: `supervisor/registry.rs:8-25`
    FiniteStateMachine,

    /// Communicating Sequential Processes (Hoare, 1978).
    ///
    /// EventBus: tokio broadcast channels. No shared mutable state between
    /// publisher and subscribers — pure message passing.
    /// Grounded in: `events.rs`
    MessagePassing,

    /// Lock-Free Concurrent Data Structures.
    ///
    /// ServiceRegistry backed by DashMap — sharded concurrent HashMap.
    /// No Mutex contention on the hot path.
    /// Grounded in: `supervisor/registry.rs`
    ConcurrentDataStructures,

    /// Graph Algorithms — Topological Sort via in-degree reduction.
    ///
    /// Dependency cycle detection via DFS with 3-color marking (White/Gray/Black).
    /// Topological ordering via in-degree counting with deterministic (sorted) selection.
    /// Grounded in: `manifest.rs` — `topo_sort()`, `check_cycles()`
    GraphAlgorithms,

    /// Exponential Backoff (Metcalfe, Ethernet, 1976).
    ///
    /// RestartPolicy: base_delay * 2^attempt, capped at max_delay.
    /// Prevents thundering herd on cascading failures.
    /// Grounded in: `process/restart.rs`
    ExponentialBackoff,

    /// Proxy Pattern (Gamma et al., Design Patterns, 1994).
    ///
    /// ReverseProxy: intercepts client requests, routes to backend services,
    /// returns responses transparently. Structural pattern.
    /// Grounded in: `proxy/mod.rs`
    ProxyPattern,
}

impl ComputingParadigm {
    /// All computing paradigms in the system.
    pub const ALL: &'static [ComputingParadigm] = &[
        ComputingParadigm::FiniteStateMachine,
        ComputingParadigm::MessagePassing,
        ComputingParadigm::ConcurrentDataStructures,
        ComputingParadigm::GraphAlgorithms,
        ComputingParadigm::ExponentialBackoff,
        ComputingParadigm::ProxyPattern,
    ];
}

impl std::fmt::Display for ComputingParadigm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FiniteStateMachine => write!(f, "finite state machine"),
            Self::MessagePassing => write!(f, "message passing (CSP)"),
            Self::ConcurrentDataStructures => write!(f, "concurrent data structures"),
            Self::GraphAlgorithms => write!(f, "graph algorithms"),
            Self::ExponentialBackoff => write!(f, "exponential backoff"),
            Self::ProxyPattern => write!(f, "proxy pattern"),
        }
    }
}

/// Engineering principles enforced in NexCloud's implementation.
///
/// Tier: T2-P (∂ Boundary) — each principle defines an enforcement boundary
/// that must not be crossed.
///
/// These are not aspirations — they are enforced via compile-time constraints
/// (`#![forbid(unsafe_code)]`), runtime checks, and architectural invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineeringPrinciple {
    /// Defense in Depth: Multiple independent security layers.
    ///
    /// Layers: TLS termination → host validation → input sanitization →
    /// PID verification → file permissions → process isolation.
    DefenseInDepth,

    /// Separation of Concerns: Independent subsystems.
    ///
    /// Supervisor (process lifecycle) / Proxy (request routing) /
    /// Health Monitor (observability) operate as orthogonal subsystems
    /// communicating only through EventBus.
    SeparationOfConcerns,

    /// Graceful Degradation: Orderly failure handling.
    ///
    /// SIGTERM (ask) → 3s wait → SIGKILL (force). Health checks before
    /// declaring failure. Restart attempts before marking as Failed.
    GracefulDegradation,

    /// Fail-Fast: Detect errors at the earliest possible moment.
    ///
    /// Manifest validation catches misconfigurations before any process spawns.
    /// Binary existence checked before `exec`. Dependency cycles detected at parse time.
    FailFast,

    /// Least Privilege: Minimal necessary permissions.
    ///
    /// PID files: 0600 (owner read/write only). Log files: 0640 (owner read/write,
    /// group read). No arbitrary code execution. No runtime manifest mutation.
    LeastPrivilege,

    /// Idempotency: Operations can be safely repeated.
    ///
    /// `nexcloud stop` is safe to call multiple times. Service restart is
    /// atomic: stop + start. Health checks are read-only probes.
    Idempotency,
}

impl EngineeringPrinciple {
    /// All engineering principles enforced in the system.
    pub const ALL: &'static [EngineeringPrinciple] = &[
        EngineeringPrinciple::DefenseInDepth,
        EngineeringPrinciple::SeparationOfConcerns,
        EngineeringPrinciple::GracefulDegradation,
        EngineeringPrinciple::FailFast,
        EngineeringPrinciple::LeastPrivilege,
        EngineeringPrinciple::Idempotency,
    ];
}

impl std::fmt::Display for EngineeringPrinciple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DefenseInDepth => write!(f, "defense in depth"),
            Self::SeparationOfConcerns => write!(f, "separation of concerns"),
            Self::GracefulDegradation => write!(f, "graceful degradation"),
            Self::FailFast => write!(f, "fail-fast"),
            Self::LeastPrivilege => write!(f, "least privilege"),
            Self::Idempotency => write!(f, "idempotency"),
        }
    }
}

/// Engineering standards that NexCloud conforms to.
///
/// Tier: T2-C (∂ Boundary + π Persistence + κ Comparison)
/// Standards define persistent boundaries against which compliance is compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standard {
    /// POSIX Signals (IEEE 1003.1) — SIGTERM/SIGKILL process lifecycle.
    PosixSignals,
    /// TLS 1.2+ (RFC 8446) — rustls with safe defaults.
    Tls12Plus,
    /// HTTP/1.1 (RFC 9110) — hyper-based reverse proxy.
    Http11,
    /// TOML Configuration — structured manifest format.
    Toml,
    /// Unix File Permissions (POSIX) — principle of least privilege.
    UnixPermissions,
}

impl Standard {
    /// All standards the system conforms to.
    pub const ALL: &'static [Standard] = &[
        Standard::PosixSignals,
        Standard::Tls12Plus,
        Standard::Http11,
        Standard::Toml,
        Standard::UnixPermissions,
    ];
}

impl std::fmt::Display for Standard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PosixSignals => write!(f, "POSIX signals (IEEE 1003.1)"),
            Self::Tls12Plus => write!(f, "TLS 1.2+ (RFC 8446)"),
            Self::Http11 => write!(f, "HTTP/1.1 (RFC 9110)"),
            Self::Toml => write!(f, "TOML configuration"),
            Self::UnixPermissions => write!(f, "Unix permissions (POSIX)"),
        }
    }
}

/// A grounding record — proof that a system component is grounded in
/// established science, computing, or engineering.
///
/// Tier: T2-C (→ Causality + ∃ Existence + κ Comparison)
/// Traces the causal chain from implementation to theoretical foundation.
#[derive(Debug, Clone)]
pub struct GroundingRecord {
    /// The system component being grounded.
    pub component: String,
    /// Which scientific discipline grounds it.
    pub science: Option<ScientificDiscipline>,
    /// Which computing paradigm grounds it.
    pub computing: Option<ComputingParadigm>,
    /// Which engineering principle grounds it.
    pub engineering: Option<EngineeringPrinciple>,
    /// Which standard it conforms to.
    pub standard: Option<Standard>,
    /// Source file and line where the grounding is implemented.
    pub source_location: String,
}

/// Validate that a component has at least one grounding.
///
/// A component with no scientific, computing, or engineering grounding
/// is an unmoored abstraction — it should not exist in the system.
pub fn validate_grounding(record: &GroundingRecord) -> bool {
    record.science.is_some()
        || record.computing.is_some()
        || record.engineering.is_some()
        || record.standard.is_some()
}

/// Build the complete grounding map for NexCloud.
///
/// Every major component is traced to its theoretical foundation.
/// This function serves as the living documentation of NexCloud's
/// scientific and engineering provenance.
pub fn system_groundings() -> Vec<GroundingRecord> {
    vec![
        GroundingRecord {
            component: "ProcessState FSM".to_string(),
            science: Some(ScientificDiscipline::ControlTheory),
            computing: Some(ComputingParadigm::FiniteStateMachine),
            engineering: Some(EngineeringPrinciple::FailFast),
            standard: None,
            source_location: "supervisor/registry.rs:8".to_string(),
        },
        GroundingRecord {
            component: "EventBus".to_string(),
            science: Some(ScientificDiscipline::InformationTheory),
            computing: Some(ComputingParadigm::MessagePassing),
            engineering: Some(EngineeringPrinciple::SeparationOfConcerns),
            standard: None,
            source_location: "events.rs:33".to_string(),
        },
        GroundingRecord {
            component: "ServiceRegistry".to_string(),
            science: None,
            computing: Some(ComputingParadigm::ConcurrentDataStructures),
            engineering: None,
            standard: None,
            source_location: "supervisor/registry.rs:61".to_string(),
        },
        GroundingRecord {
            component: "Dependency Ordering".to_string(),
            science: None,
            computing: Some(ComputingParadigm::GraphAlgorithms),
            engineering: Some(EngineeringPrinciple::FailFast),
            standard: None,
            source_location: "manifest.rs:topo_sort".to_string(),
        },
        GroundingRecord {
            component: "RestartPolicy".to_string(),
            science: Some(ScientificDiscipline::ControlTheory),
            computing: Some(ComputingParadigm::ExponentialBackoff),
            engineering: Some(EngineeringPrinciple::GracefulDegradation),
            standard: None,
            source_location: "process/restart.rs:1".to_string(),
        },
        GroundingRecord {
            component: "ReverseProxy".to_string(),
            science: None,
            computing: Some(ComputingParadigm::ProxyPattern),
            engineering: Some(EngineeringPrinciple::SeparationOfConcerns),
            standard: Some(Standard::Http11),
            source_location: "proxy/mod.rs:33".to_string(),
        },
        GroundingRecord {
            component: "HealthChecker".to_string(),
            science: Some(ScientificDiscipline::Epidemiology),
            computing: None,
            engineering: Some(EngineeringPrinciple::FailFast),
            standard: Some(Standard::Http11),
            source_location: "process/health.rs:17".to_string(),
        },
        GroundingRecord {
            component: "Health Monitor Loop".to_string(),
            science: Some(ScientificDiscipline::SystemsBiology),
            computing: Some(ComputingParadigm::MessagePassing),
            engineering: Some(EngineeringPrinciple::GracefulDegradation),
            standard: None,
            source_location: "supervisor/mod.rs:275".to_string(),
        },
        GroundingRecord {
            component: "TLS Termination".to_string(),
            science: None,
            computing: None,
            engineering: Some(EngineeringPrinciple::DefenseInDepth),
            standard: Some(Standard::Tls12Plus),
            source_location: "proxy/tls.rs:11".to_string(),
        },
        GroundingRecord {
            component: "Graceful Shutdown".to_string(),
            science: Some(ScientificDiscipline::Thermodynamics),
            computing: None,
            engineering: Some(EngineeringPrinciple::GracefulDegradation),
            standard: Some(Standard::PosixSignals),
            source_location: "main.rs:155".to_string(),
        },
        GroundingRecord {
            component: "PID File Security".to_string(),
            science: None,
            computing: None,
            engineering: Some(EngineeringPrinciple::LeastPrivilege),
            standard: Some(Standard::UnixPermissions),
            source_location: "main.rs:394".to_string(),
        },
        GroundingRecord {
            component: "Manifest Validation".to_string(),
            science: None,
            computing: Some(ComputingParadigm::GraphAlgorithms),
            engineering: Some(EngineeringPrinciple::FailFast),
            standard: Some(Standard::Toml),
            source_location: "manifest.rs:validate".to_string(),
        },
        GroundingRecord {
            component: "Input Sanitization".to_string(),
            science: None,
            computing: None,
            engineering: Some(EngineeringPrinciple::DefenseInDepth),
            standard: None,
            source_location: "manifest.rs:is_safe_identifier".to_string(),
        },
    ]
}

/// Count of groundings by pillar. Useful for coverage analysis.
pub struct GroundingCoverage {
    /// Components grounded in science.
    pub science_count: usize,
    /// Components grounded in computing.
    pub computing_count: usize,
    /// Components grounded in engineering.
    pub engineering_count: usize,
    /// Components with standards conformance.
    pub standards_count: usize,
    /// Total components analyzed.
    pub total: usize,
}

/// Compute grounding coverage across all system components.
pub fn coverage() -> GroundingCoverage {
    let groundings = system_groundings();
    let total = groundings.len();

    GroundingCoverage {
        science_count: groundings.iter().filter(|g| g.science.is_some()).count(),
        computing_count: groundings.iter().filter(|g| g.computing.is_some()).count(),
        engineering_count: groundings
            .iter()
            .filter(|g| g.engineering.is_some())
            .count(),
        standards_count: groundings.iter().filter(|g| g.standard.is_some()).count(),
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Scientific Discipline Tests ===

    #[test]
    fn scientific_discipline_count() {
        assert_eq!(ScientificDiscipline::ALL.len(), 5);
    }

    #[test]
    fn scientific_discipline_display() {
        assert_eq!(
            format!("{}", ScientificDiscipline::Thermodynamics),
            "thermodynamics"
        );
        assert_eq!(
            format!("{}", ScientificDiscipline::SystemsBiology),
            "systems biology"
        );
        assert_eq!(
            format!("{}", ScientificDiscipline::Epidemiology),
            "epidemiology"
        );
        assert_eq!(
            format!("{}", ScientificDiscipline::InformationTheory),
            "information theory"
        );
        assert_eq!(
            format!("{}", ScientificDiscipline::ControlTheory),
            "control theory"
        );
    }

    #[test]
    fn scientific_discipline_equality() {
        assert_eq!(
            ScientificDiscipline::Thermodynamics,
            ScientificDiscipline::Thermodynamics
        );
        assert_ne!(
            ScientificDiscipline::Thermodynamics,
            ScientificDiscipline::Epidemiology
        );
    }

    #[test]
    fn scientific_discipline_clone() {
        let d = ScientificDiscipline::ControlTheory;
        let c = d;
        assert_eq!(d, c);
    }

    // === Computing Paradigm Tests ===

    #[test]
    fn computing_paradigm_count() {
        assert_eq!(ComputingParadigm::ALL.len(), 6);
    }

    #[test]
    fn computing_paradigm_display() {
        assert_eq!(
            format!("{}", ComputingParadigm::FiniteStateMachine),
            "finite state machine"
        );
        assert_eq!(
            format!("{}", ComputingParadigm::MessagePassing),
            "message passing (CSP)"
        );
        assert_eq!(
            format!("{}", ComputingParadigm::ConcurrentDataStructures),
            "concurrent data structures"
        );
        assert_eq!(
            format!("{}", ComputingParadigm::GraphAlgorithms),
            "graph algorithms"
        );
        assert_eq!(
            format!("{}", ComputingParadigm::ExponentialBackoff),
            "exponential backoff"
        );
        assert_eq!(
            format!("{}", ComputingParadigm::ProxyPattern),
            "proxy pattern"
        );
    }

    #[test]
    fn computing_paradigm_equality() {
        assert_eq!(
            ComputingParadigm::MessagePassing,
            ComputingParadigm::MessagePassing
        );
        assert_ne!(
            ComputingParadigm::MessagePassing,
            ComputingParadigm::GraphAlgorithms
        );
    }

    // === Engineering Principle Tests ===

    #[test]
    fn engineering_principle_count() {
        assert_eq!(EngineeringPrinciple::ALL.len(), 6);
    }

    #[test]
    fn engineering_principle_display() {
        assert_eq!(
            format!("{}", EngineeringPrinciple::DefenseInDepth),
            "defense in depth"
        );
        assert_eq!(
            format!("{}", EngineeringPrinciple::SeparationOfConcerns),
            "separation of concerns"
        );
        assert_eq!(
            format!("{}", EngineeringPrinciple::GracefulDegradation),
            "graceful degradation"
        );
        assert_eq!(format!("{}", EngineeringPrinciple::FailFast), "fail-fast");
        assert_eq!(
            format!("{}", EngineeringPrinciple::LeastPrivilege),
            "least privilege"
        );
        assert_eq!(
            format!("{}", EngineeringPrinciple::Idempotency),
            "idempotency"
        );
    }

    #[test]
    fn engineering_principle_equality() {
        assert_eq!(
            EngineeringPrinciple::FailFast,
            EngineeringPrinciple::FailFast
        );
        assert_ne!(
            EngineeringPrinciple::FailFast,
            EngineeringPrinciple::Idempotency
        );
    }

    // === Standard Tests ===

    #[test]
    fn standard_count() {
        assert_eq!(Standard::ALL.len(), 5);
    }

    #[test]
    fn standard_display() {
        assert_eq!(
            format!("{}", Standard::PosixSignals),
            "POSIX signals (IEEE 1003.1)"
        );
        assert_eq!(format!("{}", Standard::Tls12Plus), "TLS 1.2+ (RFC 8446)");
        assert_eq!(format!("{}", Standard::Http11), "HTTP/1.1 (RFC 9110)");
        assert_eq!(format!("{}", Standard::Toml), "TOML configuration");
        assert_eq!(
            format!("{}", Standard::UnixPermissions),
            "Unix permissions (POSIX)"
        );
    }

    #[test]
    fn standard_equality() {
        assert_eq!(Standard::Tls12Plus, Standard::Tls12Plus);
        assert_ne!(Standard::Tls12Plus, Standard::Http11);
    }

    // === Grounding Record Tests ===

    #[test]
    fn grounding_record_with_all_pillars() {
        let record = GroundingRecord {
            component: "test".to_string(),
            science: Some(ScientificDiscipline::Thermodynamics),
            computing: Some(ComputingParadigm::FiniteStateMachine),
            engineering: Some(EngineeringPrinciple::FailFast),
            standard: Some(Standard::PosixSignals),
            source_location: "test.rs:1".to_string(),
        };
        assert!(validate_grounding(&record));
    }

    #[test]
    fn grounding_record_with_science_only() {
        let record = GroundingRecord {
            component: "test".to_string(),
            science: Some(ScientificDiscipline::Epidemiology),
            computing: None,
            engineering: None,
            standard: None,
            source_location: "test.rs:1".to_string(),
        };
        assert!(validate_grounding(&record));
    }

    #[test]
    fn grounding_record_with_no_grounding_fails() {
        let record = GroundingRecord {
            component: "unmoored".to_string(),
            science: None,
            computing: None,
            engineering: None,
            standard: None,
            source_location: "nowhere.rs:0".to_string(),
        };
        assert!(!validate_grounding(&record));
    }

    #[test]
    fn grounding_record_debug() {
        let record = GroundingRecord {
            component: "test".to_string(),
            science: Some(ScientificDiscipline::InformationTheory),
            computing: None,
            engineering: None,
            standard: None,
            source_location: "test.rs:1".to_string(),
        };
        let debug = format!("{record:?}");
        assert!(debug.contains("InformationTheory"));
    }

    #[test]
    fn grounding_record_clone() {
        let record = GroundingRecord {
            component: "original".to_string(),
            science: Some(ScientificDiscipline::ControlTheory),
            computing: None,
            engineering: Some(EngineeringPrinciple::GracefulDegradation),
            standard: None,
            source_location: "test.rs:1".to_string(),
        };
        let cloned = record.clone();
        assert_eq!(cloned.component, "original");
        assert_eq!(cloned.science, Some(ScientificDiscipline::ControlTheory));
    }

    // === System Groundings Tests ===

    #[test]
    fn system_groundings_not_empty() {
        let groundings = system_groundings();
        assert!(!groundings.is_empty());
        assert!(groundings.len() >= 13); // 13 components documented
    }

    #[test]
    fn all_system_groundings_are_valid() {
        for record in system_groundings() {
            assert!(
                validate_grounding(&record),
                "component '{}' has no grounding",
                record.component
            );
        }
    }

    #[test]
    fn no_empty_source_locations() {
        for record in system_groundings() {
            assert!(
                !record.source_location.is_empty(),
                "component '{}' has empty source location",
                record.component
            );
        }
    }

    #[test]
    fn no_empty_component_names() {
        for record in system_groundings() {
            assert!(
                !record.component.is_empty(),
                "found grounding record with empty component name"
            );
        }
    }

    // === Coverage Tests ===

    #[test]
    fn coverage_totals_match() {
        let c = coverage();
        assert_eq!(c.total, system_groundings().len());
    }

    #[test]
    fn coverage_science_present() {
        let c = coverage();
        assert!(c.science_count > 0, "no science groundings found");
    }

    #[test]
    fn coverage_computing_present() {
        let c = coverage();
        assert!(c.computing_count > 0, "no computing groundings found");
    }

    #[test]
    fn coverage_engineering_present() {
        let c = coverage();
        assert!(c.engineering_count > 0, "no engineering groundings found");
    }

    #[test]
    fn coverage_standards_present() {
        let c = coverage();
        assert!(c.standards_count > 0, "no standards groundings found");
    }

    #[test]
    fn coverage_science_exact() {
        let c = coverage();
        // 6 of 13 components are grounded in science — update if groundings change
        assert_eq!(
            c.science_count, 6,
            "science grounding count changed — update this test if intentional"
        );
    }

    #[test]
    fn coverage_computing_exact() {
        let c = coverage();
        // 8 of 13 components are grounded in computing — update if groundings change
        assert_eq!(
            c.computing_count, 8,
            "computing grounding count changed — update this test if intentional"
        );
    }

    #[test]
    fn coverage_engineering_exact() {
        let c = coverage();
        // 12 of 13 components are grounded in engineering — update if groundings change
        assert_eq!(
            c.engineering_count, 12,
            "engineering grounding count changed — update this test if intentional"
        );
    }

    // === Integration: Grounding ↔ Implementation Traceability ===

    #[test]
    fn process_state_fsm_grounded() {
        let groundings = system_groundings();
        let fsm = groundings
            .iter()
            .find(|g| g.component == "ProcessState FSM");
        assert!(fsm.is_some(), "ProcessState FSM must be grounded");
        if let Some(record) = fsm {
            assert_eq!(
                record.computing,
                Some(ComputingParadigm::FiniteStateMachine)
            );
        }
    }

    #[test]
    fn health_checker_grounded_in_epidemiology() {
        let groundings = system_groundings();
        let hc = groundings.iter().find(|g| g.component == "HealthChecker");
        assert!(hc.is_some(), "HealthChecker must be grounded");
        if let Some(record) = hc {
            assert_eq!(record.science, Some(ScientificDiscipline::Epidemiology));
        }
    }

    #[test]
    fn tls_grounded_in_defense_in_depth() {
        let groundings = system_groundings();
        let tls = groundings.iter().find(|g| g.component == "TLS Termination");
        assert!(tls.is_some(), "TLS Termination must be grounded");
        if let Some(record) = tls {
            assert_eq!(
                record.engineering,
                Some(EngineeringPrinciple::DefenseInDepth)
            );
            assert_eq!(record.standard, Some(Standard::Tls12Plus));
        }
    }

    #[test]
    fn graceful_shutdown_grounded_in_posix() {
        let groundings = system_groundings();
        let shutdown = groundings
            .iter()
            .find(|g| g.component == "Graceful Shutdown");
        assert!(shutdown.is_some(), "Graceful Shutdown must be grounded");
        if let Some(record) = shutdown {
            assert_eq!(record.standard, Some(Standard::PosixSignals));
            assert_eq!(
                record.engineering,
                Some(EngineeringPrinciple::GracefulDegradation)
            );
        }
    }

    #[test]
    fn event_bus_grounded_in_information_theory() {
        let groundings = system_groundings();
        let eb = groundings.iter().find(|g| g.component == "EventBus");
        assert!(eb.is_some(), "EventBus must be grounded");
        if let Some(record) = eb {
            assert_eq!(
                record.science,
                Some(ScientificDiscipline::InformationTheory)
            );
            assert_eq!(record.computing, Some(ComputingParadigm::MessagePassing));
        }
    }
}
