//! # NexVigilant Core — Circulatory System
//!
//! Data transport pipeline modeled after the biological cardiovascular system.
//! Collects data from sources, enriches with metadata, routes to destinations.
//!
//! ## Pipeline
//!
//! ```text
//! Heart(collect → oxygenate → distribute) → Vessels(arteries/veins) → Blood(carry/detect/repair)
//! ```
//!
//! ## Organ Mapping (Apps Script → Rust)
//!
//! | JS Organ | Rust Type | Function |
//! |----------|-----------|----------|
//! | `CIRCULATORY.heart` | `Heart` | Central pump: collect, enrich, distribute |
//! | `CIRCULATORY.vessels` | `Vessels` | Route data via arteries (out) and veins (return) |
//! | `CIRCULATORY.blood.redCells` | `BloodCell` | Data carriers with payload and priority |
//! | `CIRCULATORY.blood.whiteCells` | White cell detection | Threat detection in transit |
//! | `CIRCULATORY.blood.platelets` | `Platelet` | Breach repair agents |
//! | `CIRCULATORY.blood.plasma` | `BloodPressure` | System pressure/capacity monitoring |
//!
//! ## Claude Code MCP Transport Mapping (Biological Alignment v2.0 section 6)
//!
//! The `claude_code` module maps this circulatory system to Claude Code's MCP
//! server transport network: Heart = MCP Server Manager (stdio/HTTP dual
//! circulation), Vessels = MCP transport channels (arterial outbound, venous
//! return), Blood = data payload (red cells = tool results, white cells =
//! permission checks, platelets = error recovery, plasma = JSON protocol).
//! Additional mappings: portal filtration = MAX_MCP_OUTPUT_TOKENS, selective
//! perfusion = deferred tool loading, Frank-Starling = MCP_TIMEOUT scaling.

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![allow(
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::as_conversions,
    clippy::arithmetic_side_effects,
    reason = "Circulatory domain types are intentionally closed and use bounded transport metrics"
)]

pub mod claude_code;
pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// Error Type
// ============================================================================

/// Errors during circulatory operations.
/// Tier: T2-P (Boundary + Sum), dominant Boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CirculatoryError {
    /// No data to pump
    EmptyBloodstream,
    /// Pressure too low (system underloaded)
    LowPressure(f64),
    /// Pressure too high (system overloaded)
    HighPressure(f64),
}

impl core::fmt::Display for CirculatoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyBloodstream => write!(f, "empty bloodstream — no data to pump"),
            Self::LowPressure(p) => write!(f, "low pressure: {p:.2}"),
            Self::HighPressure(p) => write!(f, "high pressure: {p:.2}"),
        }
    }
}

impl std::error::Error for CirculatoryError {}

// ============================================================================
// CellKind — Blood cell classification
// ============================================================================

/// Classification of blood cells by their transport role.
/// Maps JS: `cell.type` in blood collection
/// Tier: T2-P (Sum + Comparison), dominant Sum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellKind {
    /// Data carrier (red blood cell analog)
    Data,
    /// Configuration/property carrier
    Config,
    /// Signal/event carrier
    Signal,
}

// ============================================================================
// Destination — Routing targets
// ============================================================================

/// Where data should be routed after enrichment.
/// Maps JS: `heart.routeToOrgan(data, organ)` → "DIGESTIVE", "NERVOUS", "STORAGE"
/// Tier: T2-P (Sum + Location), dominant Sum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Destination {
    /// Route to digestive system for processing
    Digestive,
    /// Route to nervous system for decision-making
    Nervous,
    /// Route to storage for persistence
    Storage,
    /// Route to immune system for threat analysis
    Immune,
}

// ============================================================================
// BloodCell — Data carrier
// ============================================================================

/// A blood cell carrying data through the circulatory system.
/// Maps JS: `heart.collectBlood()` → { source, type, content }
/// Tier: T2-C (Mapping + Product + Sequence + State), dominant Mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodCell {
    /// Source system that produced this data
    pub source: String,
    /// Classification of the carried data
    pub kind: CellKind,
    /// The data payload (serialized)
    pub payload: serde_json::Value,
    /// Whether this cell has been processed
    pub processed: bool,
}

impl BloodCell {
    /// Create a new data-carrying blood cell.
    pub fn data(source: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            source: source.into(),
            kind: CellKind::Data,
            payload,
            processed: false,
        }
    }

    /// Create a new config-carrying blood cell.
    pub fn config(source: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            source: source.into(),
            kind: CellKind::Config,
            payload,
            processed: false,
        }
    }

    /// Create a new signal-carrying blood cell.
    pub fn signal(source: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            source: source.into(),
            kind: CellKind::Signal,
            payload,
            processed: false,
        }
    }
}

// ============================================================================
// Enriched — Oxygenated blood cell
// ============================================================================

/// A blood cell enriched with routing metadata (oxygenated).
/// Maps JS: `heart.oxygenate(blood)` → adds timestamp, priority, processed flag
/// Tier: T2-P (Causality + Product + Quantity), dominant Causality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enriched {
    /// The original blood cell
    pub cell: BloodCell,
    /// Enrichment timestamp
    pub enriched_at: String,
    /// Routing priority (1 = highest, 3 = normal)
    pub priority: u8,
}

// ============================================================================
// RouteDecision — Routing result
// ============================================================================

/// The result of a routing decision: which destination for which cell.
/// Maps JS: `heart.distribute(blood)` → routes each cell to an organ
/// Tier: T2-P (Causality + Mapping), dominant Causality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    /// The enriched cell being routed
    pub cell: Enriched,
    /// The destination organ system
    pub destination: Destination,
}

// ============================================================================
// BloodPressure — System capacity monitoring
// ============================================================================

/// System pressure: ratio of available capacity to total.
/// Maps JS: `blood.plasma.pressure()` → available / total
/// Tier: T2-P (Quantity + Boundary + Comparison), dominant Quantity
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BloodPressure {
    /// Total system capacity
    pub total: usize,
    /// Currently available capacity
    pub available: usize,
}

impl BloodPressure {
    /// Create a new pressure reading.
    pub fn new(total: usize, available: usize) -> Self {
        Self {
            total,
            available: available.min(total),
        }
    }

    /// Pressure ratio (0.0 to 1.0). Below 0.7 is low.
    pub fn ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.available as f64 / self.total as f64
    }

    /// Whether pressure is in healthy range (0.7 to 1.0).
    pub fn is_healthy(&self) -> bool {
        let r = self.ratio();
        r >= 0.7 && r <= 1.0
    }
}

// ============================================================================
// Pulse — Heartbeat result
// ============================================================================

/// Result of a single heartbeat cycle.
/// Maps JS: `circulatoryPulse()` → pump + pressure check
/// Tier: T2-P (Sequence + Frequency + Quantity), dominant Sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pulse {
    /// Number of cells collected
    pub collected: usize,
    /// Number of cells enriched
    pub enriched: usize,
    /// Number of cells distributed
    pub distributed: usize,
    /// Current blood pressure
    pub pressure: BloodPressure,
    /// Timestamp of this heartbeat
    pub timestamp: String,
}

// ============================================================================
// Platelet — Repair agent
// ============================================================================

/// A repair agent that seals breaches in the data pipeline.
/// Maps JS: `blood.platelets.clot(wound)` → seal, createStub, reconnect
/// Tier: T2-P (Boundary + Causality), dominant Boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platelet {
    /// What was repaired
    pub target: String,
    /// How it was repaired
    pub method: RepairMethod,
    /// Whether repair was successful
    pub sealed: bool,
}

/// Methods a platelet can use to repair damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepairMethod {
    /// Create a stub replacement
    Stub,
    /// Reconnect a broken link
    Reconnect,
    /// Patch missing data with defaults
    Patch,
}

// ============================================================================
// Heart — Central pump
// ============================================================================

/// The heart: collects, enriches, and distributes data through the system.
/// Maps JS: `CIRCULATORY.heart`
pub struct Heart {
    /// Total system capacity for pressure tracking
    pub capacity: usize,
    /// Cells currently in the bloodstream
    bloodstream: Vec<BloodCell>,
}

impl Default for Heart {
    fn default() -> Self {
        Self {
            capacity: 5000,
            bloodstream: Vec::new(),
        }
    }
}

impl Heart {
    /// Add cells to the bloodstream for pumping.
    /// Maps JS: `heart.collectBlood()`
    pub fn collect(&mut self, cells: Vec<BloodCell>) {
        self.bloodstream.extend(cells);
    }

    /// Enrich all cells with metadata (oxygenation).
    /// Maps JS: `heart.oxygenate(blood)`
    pub fn oxygenate(&self, cells: &[BloodCell]) -> Vec<Enriched> {
        cells
            .iter()
            .map(|cell| {
                let priority = calculate_priority(cell);
                Enriched {
                    cell: cell.clone(),
                    enriched_at: nexcore_chrono::DateTime::now().to_rfc3339(),
                    priority,
                }
            })
            .collect()
    }

    /// Route enriched cells to destinations.
    /// Maps JS: `heart.distribute(blood)` → routeToOrgan per cell
    pub fn distribute(&self, enriched: Vec<Enriched>) -> Vec<RouteDecision> {
        enriched
            .into_iter()
            .map(|cell| {
                let destination = route_by_kind(cell.cell.kind);
                RouteDecision { cell, destination }
            })
            .collect()
    }

    /// Run a full pump cycle: collect → oxygenate → distribute.
    /// Maps JS: `heart.pump()`
    pub fn pump(&mut self) -> Pulse {
        let cells: Vec<BloodCell> = self.bloodstream.drain(..).collect();
        let collected = cells.len();

        let enriched = self.oxygenate(&cells);
        let enriched_count = enriched.len();

        let routed = self.distribute(enriched);
        let distributed = routed.len();

        let pressure = BloodPressure::new(self.capacity, self.capacity.saturating_sub(distributed));

        Pulse {
            collected,
            enriched: enriched_count,
            distributed,
            pressure,
            timestamp: nexcore_chrono::DateTime::now().to_rfc3339(),
        }
    }

    /// Current blood pressure based on bloodstream load.
    pub fn pressure(&self) -> BloodPressure {
        BloodPressure::new(
            self.capacity,
            self.capacity.saturating_sub(self.bloodstream.len()),
        )
    }

    /// Number of cells currently in the bloodstream.
    pub fn bloodstream_len(&self) -> usize {
        self.bloodstream.len()
    }
}

// ============================================================================
// Vessels — Routing infrastructure
// ============================================================================

/// Vessels that route blood cells between organs.
/// Maps JS: `CIRCULATORY.vessels`
pub struct Vessels;

impl Vessels {
    /// Route through main artery (high-capacity primary route).
    /// Maps JS: `vessels.arteries.mainPipeline(data)`
    pub fn arterial_route<'a>(&self, decisions: &'a [RouteDecision]) -> Vec<&'a RouteDecision> {
        // Arteries carry high-priority (1) and normal (2) traffic
        decisions.iter().filter(|d| d.cell.priority <= 2).collect()
    }

    /// Route through veins (return path, lower priority).
    /// Maps JS: `vessels.veins.collect()`
    pub fn venous_route<'a>(&self, decisions: &'a [RouteDecision]) -> Vec<&'a RouteDecision> {
        // Veins carry normal (3+) priority traffic
        decisions.iter().filter(|d| d.cell.priority >= 3).collect()
    }

    /// Merge multiple routing streams.
    /// Maps JS: `vessels.veins.merge(streams)`
    pub fn merge<'a>(&self, streams: Vec<Vec<&'a RouteDecision>>) -> Vec<&'a RouteDecision> {
        streams.into_iter().flatten().collect()
    }
}

// ============================================================================
// WhiteCells — Threat detection in transit
// ============================================================================

/// White blood cell detection: identifies threats in blood cells during transit.
/// Maps JS: `blood.whiteCells.detect(threat)` + `neutralize(threat)`
pub struct WhiteCells;

impl WhiteCells {
    /// Detect if a blood cell carries a potential threat.
    /// Maps JS: `whiteCells.detect(threat)`
    pub fn detect(&self, cell: &BloodCell) -> bool {
        // Check for known threat patterns in payload
        let payload_str = cell.payload.to_string();
        payload_str.contains("error")
            || payload_str.contains("anomaly")
            || payload_str.contains("threat")
    }

    /// Neutralize a threat by marking the cell as processed.
    /// Maps JS: `whiteCells.neutralize(threat)`
    pub fn neutralize(&self, cell: &mut BloodCell) -> bool {
        cell.processed = true;
        true
    }
}

// ============================================================================
// Platelets — Repair system
// ============================================================================

/// Platelet factory: creates repair agents for pipeline breaches.
/// Maps JS: `blood.platelets`
pub struct Platelets;

impl Platelets {
    /// Create a repair agent for a missing component.
    /// Maps JS: `platelets.clot(wound)` → seal
    pub fn repair(&self, target: &str, method: RepairMethod) -> Platelet {
        Platelet {
            target: target.to_string(),
            method,
            sealed: true,
        }
    }
}

// ============================================================================
// CirculatorySystem — Full system orchestrator
// ============================================================================

/// The complete circulatory system.
/// Maps JS: `CIRCULATORY` top-level object
pub struct CirculatorySystem {
    pub heart: Heart,
    pub vessels: Vessels,
    pub white_cells: WhiteCells,
    pub platelets: Platelets,
}

impl Default for CirculatorySystem {
    fn default() -> Self {
        Self {
            heart: Heart::default(),
            vessels: Vessels,
            white_cells: WhiteCells,
            platelets: Platelets,
        }
    }
}

impl CirculatorySystem {
    /// Run a full circulation cycle with threat detection.
    pub fn circulate(&mut self, cells: Vec<BloodCell>) -> Result<Pulse, CirculatoryError> {
        if cells.is_empty() {
            return Err(CirculatoryError::EmptyBloodstream);
        }

        // Threat scan before pumping
        let mut clean_cells = Vec::with_capacity(cells.len());
        for mut cell in cells {
            if self.white_cells.detect(&cell) {
                self.white_cells.neutralize(&mut cell);
            }
            clean_cells.push(cell);
        }

        self.heart.collect(clean_cells);
        let pulse = self.heart.pump();

        if !pulse.pressure.is_healthy() {
            let ratio = pulse.pressure.ratio();
            if ratio < 0.7 {
                return Err(CirculatoryError::LowPressure(ratio));
            }
        }

        Ok(pulse)
    }
}

// ============================================================================
// Helper functions
// ============================================================================

fn calculate_priority(cell: &BloodCell) -> u8 {
    let source_lower = cell.source.to_lowercase();
    if source_lower.contains("urgent") || source_lower.contains("critical") {
        1
    } else if source_lower.contains("priority") {
        2
    } else {
        3
    }
}

fn route_by_kind(kind: CellKind) -> Destination {
    match kind {
        CellKind::Data => Destination::Digestive,
        CellKind::Config => Destination::Nervous,
        CellKind::Signal => Destination::Immune,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- BloodCell tests ---

    #[test]
    fn blood_cell_data_creation() {
        let cell = BloodCell::data("test", serde_json::json!({"key": "value"}));
        assert_eq!(cell.kind, CellKind::Data);
        assert!(!cell.processed);
    }

    #[test]
    fn blood_cell_config_creation() {
        let cell = BloodCell::config("settings", serde_json::json!(42));
        assert_eq!(cell.kind, CellKind::Config);
    }

    #[test]
    fn blood_cell_signal_creation() {
        let cell = BloodCell::signal("event_bus", serde_json::json!("alert"));
        assert_eq!(cell.kind, CellKind::Signal);
    }

    // --- BloodPressure tests ---

    #[test]
    fn blood_pressure_ratio() {
        let bp = BloodPressure::new(100, 90);
        let diff = (bp.ratio() - 0.9).abs();
        assert!(diff < f64::EPSILON);
    }

    #[test]
    fn blood_pressure_healthy_range() {
        assert!(BloodPressure::new(100, 80).is_healthy());
        assert!(!BloodPressure::new(100, 50).is_healthy());
    }

    #[test]
    fn blood_pressure_zero_total() {
        let bp = BloodPressure::new(0, 0);
        let diff = bp.ratio().abs();
        assert!(diff < f64::EPSILON);
    }

    #[test]
    fn blood_pressure_clamps_available() {
        let bp = BloodPressure::new(100, 200);
        assert_eq!(bp.available, 100);
    }

    // --- Heart tests ---

    #[test]
    fn heart_collect_and_pump() {
        let mut heart = Heart::default();
        heart.collect(vec![
            BloodCell::data("src1", serde_json::json!("a")),
            BloodCell::config("src2", serde_json::json!("b")),
        ]);
        assert_eq!(heart.bloodstream_len(), 2);

        let pulse = heart.pump();
        assert_eq!(pulse.collected, 2);
        assert_eq!(pulse.distributed, 2);
        assert_eq!(heart.bloodstream_len(), 0);
    }

    #[test]
    fn heart_oxygenate_adds_priority() {
        let heart = Heart::default();
        let cells = vec![
            BloodCell::data("urgent-fix", serde_json::json!("x")),
            BloodCell::data("normal", serde_json::json!("y")),
        ];
        let enriched = heart.oxygenate(&cells);
        assert_eq!(enriched[0].priority, 1); // "urgent" in source
        assert_eq!(enriched[1].priority, 3); // normal
    }

    #[test]
    fn heart_distribute_routes_by_kind() {
        let heart = Heart::default();
        let enriched = vec![
            Enriched {
                cell: BloodCell::data("src", serde_json::json!("x")),
                enriched_at: String::new(),
                priority: 3,
            },
            Enriched {
                cell: BloodCell::config("src", serde_json::json!("y")),
                enriched_at: String::new(),
                priority: 3,
            },
            Enriched {
                cell: BloodCell::signal("src", serde_json::json!("z")),
                enriched_at: String::new(),
                priority: 3,
            },
        ];
        let routed = heart.distribute(enriched);
        assert_eq!(routed[0].destination, Destination::Digestive);
        assert_eq!(routed[1].destination, Destination::Nervous);
        assert_eq!(routed[2].destination, Destination::Immune);
    }

    // --- Vessels tests ---

    #[test]
    fn vessels_arterial_filters_high_priority() {
        let vessels = Vessels;
        let decisions = vec![
            RouteDecision {
                cell: Enriched {
                    cell: BloodCell::data("a", serde_json::json!(1)),
                    enriched_at: String::new(),
                    priority: 1,
                },
                destination: Destination::Digestive,
            },
            RouteDecision {
                cell: Enriched {
                    cell: BloodCell::data("b", serde_json::json!(2)),
                    enriched_at: String::new(),
                    priority: 3,
                },
                destination: Destination::Storage,
            },
        ];
        let arterial = vessels.arterial_route(&decisions);
        assert_eq!(arterial.len(), 1);
    }

    // --- WhiteCells tests ---

    #[test]
    fn white_cells_detect_threats() {
        let wc = WhiteCells;
        let threat = BloodCell::data("src", serde_json::json!({"status": "error"}));
        let clean = BloodCell::data("src", serde_json::json!({"status": "ok"}));
        assert!(wc.detect(&threat));
        assert!(!wc.detect(&clean));
    }

    #[test]
    fn white_cells_neutralize() {
        let wc = WhiteCells;
        let mut cell = BloodCell::data("src", serde_json::json!("error"));
        assert!(!cell.processed);
        wc.neutralize(&mut cell);
        assert!(cell.processed);
    }

    // --- Platelets tests ---

    #[test]
    fn platelets_repair_creates_sealed_platelet() {
        let platelets = Platelets;
        let repair = platelets.repair("broken_pipe", RepairMethod::Reconnect);
        assert!(repair.sealed);
        assert_eq!(repair.method, RepairMethod::Reconnect);
    }

    // --- Full system tests ---

    #[test]
    fn full_circulation_cycle() {
        let mut system = CirculatorySystem::default();
        let cells = vec![
            BloodCell::data("source1", serde_json::json!({"data": "hello"})),
            BloodCell::signal("events", serde_json::json!({"type": "info"})),
        ];
        let result = system.circulate(cells);
        assert!(result.is_ok());

        let pulse = result.ok().unwrap_or_else(|| Pulse {
            collected: 0,
            enriched: 0,
            distributed: 0,
            pressure: BloodPressure::new(1, 1),
            timestamp: String::new(),
        });
        assert_eq!(pulse.collected, 2);
    }

    #[test]
    fn full_circulation_empty_rejects() {
        let mut system = CirculatorySystem::default();
        let result = system.circulate(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn full_circulation_detects_threats() {
        let mut system = CirculatorySystem::default();
        let cells = vec![BloodCell::data(
            "alarm",
            serde_json::json!({"status": "error in module"}),
        )];
        let result = system.circulate(cells);
        // Should succeed but cell is neutralized (processed = true)
        assert!(result.is_ok());
    }
}
