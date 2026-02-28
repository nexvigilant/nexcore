//! TOML manifest loading for topology types.
//!
//! Three manifest formats, all loadable from disk:
//!
//! ## Hold Manifest (`hold.toml`)
//!
//! ```toml
//! [hold]
//! name = "core-primitives"
//! owner = "foundation-team"
//! review_required = true
//! members = ["nexcore-primitives", "nexcore-id", "nexcore-constants"]
//!
//! [[hold.cycle_partners]]
//! a = "stem-math"
//! b = "nexcore-lex-primitiva"
//! ```
//!
//! ## Compartment Manifest (`compartment.toml`)
//!
//! ```toml
//! [compartment]
//! name = "foundation"
//! layer = "Foundation"
//!
//! [[compartment.holds]]
//! name = "core-primitives"
//! owner = "foundation-team"
//! review_required = true
//! members = ["nexcore-primitives", "nexcore-id"]
//!
//! [[compartment.edges]]
//! from = "stem"
//! to = "core-primitives"
//! ```
//!
//! ## Bay Manifest (`bay.toml`)
//!
//! ```toml
//! [bay]
//! name = "nexcore-workspace"
//!
//! [[bay.holds]]
//! name = "core-primitives"
//! owner = "foundation-team"
//! review_required = true
//! members = ["nexcore-primitives", "nexcore-id"]
//! ```

use crate::{Bay, Compartment, GovernancePolicy, Hold, Layer, TopologyError};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

// ============================================================================
// TOML Schema Types (deserialization targets)
// ============================================================================

/// Top-level wrapper for a hold manifest file.
#[derive(Debug, Deserialize)]
pub struct HoldManifest {
    /// The hold definition.
    pub hold: HoldDef,
}

/// A hold definition within a manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct HoldDef {
    /// Hold name (must be unique within a bay).
    pub name: String,
    /// Team or individual responsible.
    pub owner: String,
    /// Whether changes require review.
    #[serde(default)]
    pub review_required: bool,
    /// Explicit layer assignment (Foundation/Domain/Orchestration/Service).
    #[serde(default)]
    pub layer: Option<String>,
    /// Crate names belonging to this hold.
    pub members: Vec<String>,
    /// Declared mutual dependency cycles.
    #[serde(default)]
    pub cycle_partners: Vec<CyclePartnerDef>,
}

/// A cycle partner pair.
#[derive(Debug, Clone, Deserialize)]
pub struct CyclePartnerDef {
    /// First crate in the cycle.
    pub a: String,
    /// Second crate in the cycle.
    pub b: String,
}

/// Top-level wrapper for a compartment manifest file.
#[derive(Debug, Deserialize)]
pub struct CompartmentManifest {
    /// The compartment definition.
    pub compartment: CompartmentDef,
}

/// A compartment definition within a manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct CompartmentDef {
    /// Compartment name.
    pub name: String,
    /// Layer this compartment belongs to.
    pub layer: String,
    /// Holds within this compartment.
    pub holds: Vec<HoldDef>,
    /// Dependency edges between holds.
    #[serde(default)]
    pub edges: Vec<EdgeDef>,
}

/// A directed edge between two holds.
#[derive(Debug, Clone, Deserialize)]
pub struct EdgeDef {
    /// Source hold (depends on `to`).
    pub from: String,
    /// Target hold.
    pub to: String,
}

/// Top-level wrapper for a bay manifest file.
#[derive(Debug, Deserialize)]
pub struct BayManifest {
    /// The bay definition.
    pub bay: BayDef,
}

/// A bay definition within a manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct BayDef {
    /// Bay name (informational).
    pub name: String,
    /// All holds in the workspace.
    pub holds: Vec<HoldDef>,
}

// ============================================================================
// Error Extension
// ============================================================================

/// Errors specific to manifest loading.
#[derive(Debug, Clone)]
pub enum ManifestError {
    /// TOML parsing failed.
    TomlParse(String),
    /// Unknown layer string.
    UnknownLayer(String),
    /// Underlying topology construction error.
    Topology(TopologyError),
    /// IO error reading a file.
    Io(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TomlParse(msg) => write!(f, "TOML parse error: {msg}"),
            Self::UnknownLayer(s) => write!(f, "unknown layer '{s}'"),
            Self::Topology(e) => write!(f, "topology error: {e}"),
            Self::Io(msg) => write!(f, "IO error: {msg}"),
        }
    }
}

impl std::error::Error for ManifestError {}

impl From<TopologyError> for ManifestError {
    fn from(e: TopologyError) -> Self {
        Self::Topology(e)
    }
}

// ============================================================================
// Conversion: HoldDef → Hold
// ============================================================================

/// Convert a [`HoldDef`] into a [`Hold`].
///
/// # Errors
/// Returns [`ManifestError::Topology`] if the hold has no members.
pub fn hold_from_def(def: &HoldDef) -> Result<Hold, ManifestError> {
    let members: BTreeSet<String> = def.members.iter().cloned().collect();
    let governance = GovernancePolicy {
        owner: def.owner.clone(),
        review_required: def.review_required,
    };
    let mut hold = Hold::new(def.name.clone(), members, governance)?;
    if !def.cycle_partners.is_empty() {
        let partners: Vec<(String, String)> = def
            .cycle_partners
            .iter()
            .map(|cp| (cp.a.clone(), cp.b.clone()))
            .collect();
        hold = hold.with_cycle_partners(partners);
    }
    if let Some(layer_str) = &def.layer {
        let layer = parse_layer(layer_str)?;
        hold = hold.with_layer(layer);
    }
    Ok(hold)
}

// ============================================================================
// Layer Parsing
// ============================================================================

/// Parse a layer string into a [`Layer`].
///
/// Accepts: "Foundation", "Domain", "Orchestration", "Service" (case-insensitive).
///
/// # Errors
/// Returns [`ManifestError::UnknownLayer`] if the string is not recognized.
pub fn parse_layer(s: &str) -> Result<Layer, ManifestError> {
    match s.to_lowercase().as_str() {
        "foundation" => Ok(Layer::Foundation),
        "domain" => Ok(Layer::Domain),
        "orchestration" => Ok(Layer::Orchestration),
        "service" => Ok(Layer::Service),
        _ => Err(ManifestError::UnknownLayer(s.to_owned())),
    }
}

// ============================================================================
// Public Loaders
// ============================================================================

/// Load a [`Hold`] from a TOML string.
///
/// # Errors
/// Returns [`ManifestError`] on parse failure or construction error.
pub fn load_hold(toml_str: &str) -> Result<Hold, ManifestError> {
    let manifest: HoldManifest =
        toml::from_str(toml_str).map_err(|e| ManifestError::TomlParse(e.to_string()))?;
    hold_from_def(&manifest.hold)
}

/// Load a [`Compartment`] from a TOML string.
///
/// # Errors
/// Returns [`ManifestError`] on parse failure, unknown layer, or construction error.
pub fn load_compartment(toml_str: &str) -> Result<Compartment, ManifestError> {
    let manifest: CompartmentManifest =
        toml::from_str(toml_str).map_err(|e| ManifestError::TomlParse(e.to_string()))?;
    let def = &manifest.compartment;

    let layer = parse_layer(&def.layer)?;

    let mut holds = BTreeMap::new();
    for hold_def in &def.holds {
        let hold = hold_from_def(hold_def)?;
        holds.insert(hold_def.name.clone(), hold);
    }

    let edges: Vec<(String, String)> = def
        .edges
        .iter()
        .map(|e| (e.from.clone(), e.to.clone()))
        .collect();

    Ok(Compartment::new(def.name.clone(), layer, holds, edges)?)
}

/// Load a [`Bay`] from a TOML string.
///
/// # Errors
/// Returns [`ManifestError`] on parse failure or construction error.
pub fn load_bay(toml_str: &str) -> Result<Bay, ManifestError> {
    let manifest: BayManifest =
        toml::from_str(toml_str).map_err(|e| ManifestError::TomlParse(e.to_string()))?;

    let mut holds = Vec::new();
    for hold_def in &manifest.bay.holds {
        holds.push(hold_from_def(hold_def)?);
    }

    Ok(Bay::new(holds)?)
}

/// Load a [`Hold`] from a TOML file on disk.
///
/// # Errors
/// Returns [`ManifestError::Io`] on read failure, plus any parsing errors.
pub fn load_hold_file(path: &std::path::Path) -> Result<Hold, ManifestError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ManifestError::Io(format!("{}: {e}", path.display())))?;
    load_hold(&content)
}

/// Load a [`Compartment`] from a TOML file on disk.
///
/// # Errors
/// Returns [`ManifestError::Io`] on read failure, plus any parsing errors.
pub fn load_compartment_file(path: &std::path::Path) -> Result<Compartment, ManifestError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ManifestError::Io(format!("{}: {e}", path.display())))?;
    load_compartment(&content)
}

/// Load a [`Bay`] from a TOML file on disk.
///
/// # Errors
/// Returns [`ManifestError::Io`] on read failure, plus any parsing errors.
pub fn load_bay_file(path: &std::path::Path) -> Result<Bay, ManifestError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ManifestError::Io(format!("{}: {e}", path.display())))?;
    load_bay(&content)
}

/// Load a [`Bay`] from a directory of hold TOML files.
///
/// Reads every `*.toml` file in `holds_dir`, parses each as a [`HoldManifest`],
/// and constructs a [`Bay`] from the collected holds.
///
/// # Errors
/// Returns [`ManifestError::Io`] on directory read failure, plus any parsing errors.
pub fn load_bay_from_holds_dir(holds_dir: &std::path::Path) -> Result<Bay, ManifestError> {
    let entries = std::fs::read_dir(holds_dir)
        .map_err(|e| ManifestError::Io(format!("{}: {e}", holds_dir.display())))?;

    let mut holds = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| ManifestError::Io(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            holds.push(load_hold_file(&path)?);
        }
    }

    Bay::new(holds).map_err(ManifestError::from)
}

// ============================================================================
// Serialization (TOML generation)
// ============================================================================

/// Generate a hold manifest TOML string from a [`Hold`].
#[must_use]
pub fn hold_to_toml(hold: &Hold) -> String {
    let mut out = String::from("[hold]\n");
    out.push_str(&format!("name = {:?}\n", hold.name()));
    out.push_str(&format!("owner = {:?}\n", hold.governance().owner));
    out.push_str(&format!(
        "review_required = {}\n",
        hold.governance().review_required
    ));
    if let Some(layer) = hold.layer() {
        out.push_str(&format!("layer = {:?}\n", layer.label()));
    }
    out.push_str("members = [");
    let members: Vec<&String> = hold.members().iter().collect();
    for (i, m) in members.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!("{m:?}"));
    }
    out.push_str("]\n");

    for cp in hold.cycle_partners() {
        out.push_str("\n[[hold.cycle_partners]]\n");
        out.push_str(&format!("a = {:?}\n", cp.0));
        out.push_str(&format!("b = {:?}\n", cp.1));
    }

    out
}

/// Generate a bay manifest TOML string from a [`Bay`].
#[must_use]
pub fn bay_to_toml(bay: &Bay, name: &str) -> String {
    let mut out = String::from("[bay]\n");
    out.push_str(&format!("name = {name:?}\n"));

    for hold in bay.holds().values() {
        out.push_str("\n[[bay.holds]]\n");
        out.push_str(&format!("name = {:?}\n", hold.name()));
        out.push_str(&format!("owner = {:?}\n", hold.governance().owner));
        out.push_str(&format!(
            "review_required = {}\n",
            hold.governance().review_required
        ));
        if let Some(layer) = hold.layer() {
            out.push_str(&format!("layer = {:?}\n", layer.label()));
        }
        out.push_str("members = [");
        let members: Vec<&String> = hold.members().iter().collect();
        for (i, m) in members.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{m:?}"));
        }
        out.push_str("]\n");
    }

    out
}
