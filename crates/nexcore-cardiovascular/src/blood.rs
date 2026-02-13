//! Blood components — carriers, defenders, repairers, medium.
//!
//! ## Biological Model
//!
//! Blood consists of:
//! - **Red Cells (erythrocytes)**: Carry data/oxygen. Capacity-limited.
//! - **White Cells (leukocytes)**: Detect and neutralize threats.
//! - **Platelets (thrombocytes)**: Repair breaches, seal wounds.
//! - **Plasma**: The liquid transport medium. Carries pressure.
//!
//! ## T1 Grounding: μ (Mapping) dominant
//!
//! Blood IS a mapping — it maps payloads to destinations,
//! threats to responses, wounds to repairs.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::{BloodComponent, Pressure, Volume};

/// A red blood cell — data carrier.
///
/// Carries a payload from source to destination.
/// Limited capacity (hemoglobin saturation).
///
/// # Tier: T2-C (μ · N · → · ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedCell {
    /// Payload identifier
    pub payload_id: String,
    /// Source origin
    pub source: String,
    /// Destination target
    pub destination: String,
    /// Capacity (max payload size)
    pub capacity: u64,
    /// Current load (actual payload size)
    pub load: u64,
    /// Oxygen saturation (0.0–1.0)
    pub saturation: f64,
}

impl RedCell {
    /// Create a new red cell with a payload.
    pub fn new(payload_id: impl Into<String>, source: impl Into<String>, capacity: u64) -> Self {
        Self {
            payload_id: payload_id.into(),
            source: source.into(),
            destination: String::new(),
            capacity,
            load: 0,
            saturation: 1.0,
        }
    }

    /// Load data into this cell. Returns amount actually loaded.
    pub fn load_payload(&mut self, amount: u64) -> u64 {
        let space = self.capacity.saturating_sub(self.load);
        let loaded = amount.min(space);
        self.load += loaded;
        loaded
    }

    /// Unload data at destination. Returns amount unloaded.
    pub fn unload(&mut self) -> u64 {
        let unloaded = self.load;
        self.load = 0;
        self.saturation *= 0.7; // Desaturate after delivery
        unloaded
    }

    /// Is this cell fully loaded?
    pub fn is_full(&self) -> bool {
        self.load >= self.capacity
    }

    /// Is this cell oxygenated enough for another delivery?
    pub fn needs_reoxygenation(&self) -> bool {
        self.saturation < 0.5
    }

    /// Reoxygenate (pass through pulmonary circuit).
    pub fn oxygenate(&mut self) {
        self.saturation = 1.0;
    }
}

/// A white blood cell — threat detector and neutralizer.
///
/// # Tier: T2-C (∂ · κ · → · ∃)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteCell {
    /// Cell identifier
    pub id: String,
    /// What types of threats this cell targets
    pub target_threats: Vec<String>,
    /// Whether this cell is currently active
    pub active: bool,
    /// Kills count (neutralized threats)
    pub kills: u64,
}

impl WhiteCell {
    /// Create a new white cell targeting specific threats.
    pub fn new(id: impl Into<String>, targets: Vec<String>) -> Self {
        Self {
            id: id.into(),
            target_threats: targets,
            active: true,
            kills: 0,
        }
    }

    /// Can this cell fight the given threat type?
    pub fn can_target(&self, threat_type: &str) -> bool {
        self.active
            && self
                .target_threats
                .iter()
                .any(|t| t == threat_type || t == "*")
    }

    /// Attempt to neutralize a threat. Returns true if successful.
    pub fn neutralize(&mut self, threat_type: &str) -> bool {
        if self.can_target(threat_type) {
            self.kills += 1;
            true
        } else {
            false
        }
    }
}

/// A platelet — repair agent.
///
/// # Tier: T2-C (∂ · ∝ · ς)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platelet {
    /// Whether this platelet has been activated (used)
    pub activated: bool,
    /// Type of repair this platelet can perform
    pub repair_type: RepairType,
}

/// Types of repair a platelet can perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairType {
    /// Seal a breach (reconnect broken pipe)
    Seal,
    /// Create a stub (temporary function replacement)
    Stub,
    /// Reconnect (restore dropped connection)
    Reconnect,
    /// Generic repair
    Generic,
}

impl fmt::Display for RepairType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seal => write!(f, "Seal"),
            Self::Stub => write!(f, "Stub"),
            Self::Reconnect => write!(f, "Reconnect"),
            Self::Generic => write!(f, "Generic"),
        }
    }
}

impl Platelet {
    /// Create a new platelet.
    pub fn new(repair_type: RepairType) -> Self {
        Self {
            activated: false,
            repair_type,
        }
    }

    /// Activate this platelet to perform repair. Returns true if not already used.
    pub fn activate(&mut self) -> bool {
        if self.activated {
            return false;
        }
        self.activated = true;
        true
    }
}

/// Plasma — the transport medium.
///
/// # Tier: T2-P (N · ∂)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plasma {
    /// Total volume capacity
    pub total_volume: Volume,
    /// Available (unfilled) volume
    pub available_volume: Volume,
}

impl Default for Plasma {
    fn default() -> Self {
        Self {
            total_volume: Volume::new(3000.0),     // ~3L plasma
            available_volume: Volume::new(2700.0), // ~90% available
        }
    }
}

impl Plasma {
    /// Create plasma with given capacity.
    pub fn new(total: f64) -> Self {
        Self {
            total_volume: Volume::new(total),
            available_volume: Volume::new(total * 0.9),
        }
    }

    /// Current pressure (available/total).
    pub fn pressure(&self) -> Pressure {
        let total = self.total_volume.value();
        if total <= 0.0 {
            return Pressure::new(0.0);
        }
        Pressure::new(self.available_volume.value() / total)
    }

    /// Consume volume (data loaded into plasma).
    pub fn consume(&mut self, amount: f64) -> f64 {
        let available = self.available_volume.value();
        let consumed = amount.min(available).max(0.0);
        self.available_volume = Volume::new(available - consumed);
        consumed
    }

    /// Release volume (data delivered from plasma).
    pub fn release(&mut self, amount: f64) -> f64 {
        let new_vol = self.available_volume.value() + amount;
        let capped = new_vol.min(self.total_volume.value());
        let released = capped - self.available_volume.value();
        self.available_volume = Volume::new(capped);
        released.max(0.0)
    }
}

/// Complete blood sample — snapshot of all components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloodSample {
    /// Component counts
    pub red_cell_count: u64,
    /// White cell count
    pub white_cell_count: u64,
    /// Platelet count
    pub platelet_count: u64,
    /// Plasma state
    pub plasma: Plasma,
    /// Overall health assessment
    pub health: BloodHealth,
}

/// Blood health classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BloodHealth {
    /// All counts normal
    Healthy,
    /// Low red cells
    Anemic,
    /// Low white cells (vulnerable to infection)
    Leukopenic,
    /// Low platelets (can't repair)
    Thrombocytopenic,
    /// Low plasma volume (dehydration)
    Hypovolemic,
}

impl fmt::Display for BloodHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "Healthy"),
            Self::Anemic => write!(f, "Anemic"),
            Self::Leukopenic => write!(f, "Leukopenic"),
            Self::Thrombocytopenic => write!(f, "Thrombocytopenic"),
            Self::Hypovolemic => write!(f, "Hypovolemic"),
        }
    }
}

/// Classify blood health from component counts.
pub fn assess_blood_health(
    red_cells: u64,
    white_cells: u64,
    platelets: u64,
    plasma_pressure: Pressure,
) -> BloodHealth {
    // Thresholds (simplified)
    if red_cells < 100 {
        BloodHealth::Anemic
    } else if white_cells < 10 {
        BloodHealth::Leukopenic
    } else if platelets < 50 {
        BloodHealth::Thrombocytopenic
    } else if plasma_pressure.is_hypotensive(0.3) {
        BloodHealth::Hypovolemic
    } else {
        BloodHealth::Healthy
    }
}

/// Describe what computational role a blood component plays.
pub fn component_role(component: BloodComponent) -> &'static str {
    match component {
        BloodComponent::RedCell => "Data carrier — transports payloads between services",
        BloodComponent::WhiteCell => "Threat detector — identifies and neutralizes errors",
        BloodComponent::Platelet => "Repair agent — patches broken connections and stubs",
        BloodComponent::Plasma => "Transport medium — the channel through which all data flows",
    }
}
