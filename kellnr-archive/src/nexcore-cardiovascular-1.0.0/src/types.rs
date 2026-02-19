//! Core cardiovascular types.
//!
//! ## T1 Grounding
//!
//! - `Circuit` → Σ (sum type, pulmonary vs systemic)
//! - `VesselType` → Σ (arterial/capillary/venous)
//! - `BloodComponent` → Σ (red cell/white cell/platelet/plasma)
//! - `Pathology` → Σ (atherosclerosis/infarction/hypertension/anemia)

use serde::{Deserialize, Serialize};
use std::fmt;

// ═══════════════════════════════════════════════════════════
// Newtypes — no raw f64 for domain values
// ═══════════════════════════════════════════════════════════

/// Pressure in abstract units (0.0–1.0 normalized).
///
/// Maps: system_load = throughput × downstream_latency
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pressure(f64);

impl Pressure {
    /// Create a new pressure value, clamped to [0.0, max].
    pub fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }

    /// Get the raw value.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if pressure exceeds threshold (hypertension).
    pub fn is_hypertensive(&self, threshold: f64) -> bool {
        self.0 > threshold
    }

    /// Check if pressure is below threshold (hypotension).
    pub fn is_hypotensive(&self, threshold: f64) -> bool {
        self.0 < threshold
    }
}

impl Default for Pressure {
    fn default() -> Self {
        Self(0.5) // Homeostatic set point
    }
}

impl fmt::Display for Pressure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2} mmHg", self.0)
    }
}

/// Flow rate in abstract units per time step.
///
/// Maps: data throughput (items/second)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FlowRate(f64);

impl FlowRate {
    /// Create a new flow rate, clamped to non-negative.
    pub fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }

    /// Get the raw value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for FlowRate {
    fn default() -> Self {
        Self(5.0) // Resting cardiac output ~5 L/min
    }
}

impl fmt::Display for FlowRate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1} L/min", self.0)
    }
}

/// Volume in abstract units.
///
/// Maps: buffer/queue capacity
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Volume(f64);

impl Volume {
    /// Create a new volume, clamped to non-negative.
    pub fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }

    /// Get the raw value.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Fraction of capacity used.
    pub fn utilization(&self, capacity: f64) -> f64 {
        if capacity <= 0.0 {
            return 1.0;
        }
        self.0 / capacity
    }
}

impl Default for Volume {
    fn default() -> Self {
        Self(5000.0) // ~5L total blood volume
    }
}

/// Resistance to flow in a vessel.
///
/// Maps: downstream latency / backpressure
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Resistance(f64);

impl Resistance {
    /// Create a new resistance value.
    pub fn new(value: f64) -> Self {
        Self(if value < 0.0 { 0.0 } else { value })
    }

    /// Get the raw value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for Resistance {
    fn default() -> Self {
        Self(1.0) // Normal peripheral resistance
    }
}

/// Heart rate in beats per time step.
///
/// Maps: polling frequency / processing cycle rate
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeartRate(f64);

impl HeartRate {
    /// Create a new heart rate.
    pub fn new(bpm: f64) -> Self {
        Self(if bpm < 0.0 { 0.0 } else { bpm })
    }

    /// Get the raw value.
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Bradycardia: rate below threshold.
    pub fn is_bradycardic(&self, threshold: f64) -> bool {
        self.0 < threshold
    }

    /// Tachycardia: rate above threshold.
    pub fn is_tachycardic(&self, threshold: f64) -> bool {
        self.0 > threshold
    }
}

impl Default for HeartRate {
    fn default() -> Self {
        Self(72.0) // Normal resting heart rate
    }
}

// ═══════════════════════════════════════════════════════════
// Enums — categorical classification
// ═══════════════════════════════════════════════════════════

/// Circulatory circuit type.
///
/// Dual circulation: pulmonary (refresh) vs systemic (deliver).
///
/// # Tier: T2-P (Σ · →)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Circuit {
    /// Pulmonary: heart → lungs → heart (validation/refresh)
    Pulmonary,
    /// Systemic: heart → body → heart (delivery/work)
    Systemic,
    /// Portal: gut → liver → heart (first-pass validation)
    Portal,
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pulmonary => write!(f, "Pulmonary"),
            Self::Systemic => write!(f, "Systemic"),
            Self::Portal => write!(f, "Portal"),
        }
    }
}

/// Vessel type classification.
///
/// Hierarchical: arteries (high-pressure outbound) → capillaries (exchange) → veins (return).
///
/// # Tier: T2-P (Σ · ∂)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VesselType {
    /// Aorta: largest artery, main output
    Aorta,
    /// Artery: high-pressure outbound
    Artery,
    /// Arteriole: small artery, pressure regulation
    Arteriole,
    /// Capillary: exchange point (deliver + collect)
    Capillary,
    /// Venule: small vein, collection
    Venule,
    /// Vein: low-pressure return
    Vein,
    /// VenaCava: largest vein, main return
    VenaCava,
}

impl VesselType {
    /// Is this an outbound (arterial) vessel?
    pub fn is_arterial(&self) -> bool {
        matches!(self, Self::Aorta | Self::Artery | Self::Arteriole)
    }

    /// Is this an exchange vessel?
    pub fn is_capillary(&self) -> bool {
        matches!(self, Self::Capillary)
    }

    /// Is this a return (venous) vessel?
    pub fn is_venous(&self) -> bool {
        matches!(self, Self::Venule | Self::Vein | Self::VenaCava)
    }

    /// Relative pressure in this vessel type (0.0 to 1.0).
    pub fn typical_pressure(&self) -> f64 {
        match self {
            Self::Aorta => 1.0,
            Self::Artery => 0.85,
            Self::Arteriole => 0.60,
            Self::Capillary => 0.30,
            Self::Venule => 0.15,
            Self::Vein => 0.10,
            Self::VenaCava => 0.05,
        }
    }
}

impl fmt::Display for VesselType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aorta => write!(f, "Aorta"),
            Self::Artery => write!(f, "Artery"),
            Self::Arteriole => write!(f, "Arteriole"),
            Self::Capillary => write!(f, "Capillary"),
            Self::Venule => write!(f, "Venule"),
            Self::Vein => write!(f, "Vein"),
            Self::VenaCava => write!(f, "Vena Cava"),
        }
    }
}

/// Blood component classification.
///
/// # Tier: T2-P (Σ · μ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BloodComponent {
    /// Red blood cell: oxygen/data carrier
    RedCell,
    /// White blood cell: immune/threat response
    WhiteCell,
    /// Platelet: repair/clotting agent
    Platelet,
    /// Plasma: transport medium
    Plasma,
}

impl fmt::Display for BloodComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RedCell => write!(f, "Red Cell"),
            Self::WhiteCell => write!(f, "White Cell"),
            Self::Platelet => write!(f, "Platelet"),
            Self::Plasma => write!(f, "Plasma"),
        }
    }
}

/// Cardiovascular pathology classification.
///
/// # Tier: T2-C (Σ · ∂ · ∝)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pathology {
    /// Pipeline clogging from accumulated debt/latency
    Atherosclerosis {
        /// Which vessel is affected
        vessel: String,
        /// Occlusion percentage (0.0–1.0)
        occlusion: f64,
    },
    /// Critical path failure — blocked artery
    Infarction {
        /// Which vessel is blocked
        vessel: String,
        /// Whether flow is completely blocked
        complete: bool,
    },
    /// Chronic backpressure — sustained high system load
    Hypertension {
        /// Current pressure reading
        pressure: Pressure,
        /// Duration in time steps
        duration: u64,
    },
    /// Insufficient carriers — not enough transport capacity
    Anemia {
        /// Current carrier count
        carrier_count: u64,
        /// Required carrier count
        required: u64,
    },
    /// Low pressure — insufficient flow
    Hypotension {
        /// Current pressure
        pressure: Pressure,
    },
    /// Custom/unknown pathology
    Unknown(String),
}

impl fmt::Display for Pathology {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atherosclerosis { vessel, occlusion } => {
                write!(
                    f,
                    "Atherosclerosis: {} ({:.0}% occluded)",
                    vessel,
                    occlusion * 100.0
                )
            }
            Self::Infarction { vessel, complete } => {
                let status = if *complete { "complete" } else { "partial" };
                write!(f, "Infarction: {} ({})", vessel, status)
            }
            Self::Hypertension { pressure, duration } => {
                write!(f, "Hypertension: {} for {} cycles", pressure, duration)
            }
            Self::Anemia {
                carrier_count,
                required,
            } => write!(f, "Anemia: {}/{} carriers", carrier_count, required),
            Self::Hypotension { pressure } => write!(f, "Hypotension: {}", pressure),
            Self::Unknown(desc) => write!(f, "Unknown: {}", desc),
        }
    }
}
