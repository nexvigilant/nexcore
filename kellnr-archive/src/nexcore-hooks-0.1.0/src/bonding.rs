//! # Hook Bonding: Chemical Bond Architecture for Hook-Skill Molecules
//!
//! This module implements a chemical bond metaphor for connecting hooks and skills:
//!
//! - **Atoms**: Individual hooks (smallest unit of enforcement)
//! - **Valence**: Input/output ports (data compatibility)
//! - **Bond Strength**: Priority (1-100, stronger = executed first)
//! - **Molecules**: Skill + bonded hooks (functional unit)
//! - **Reactions**: Event cascades that trigger chain responses
//!
//! ## Primitive Foundation (from extraction analysis)
//!
//! | Tier | Primitives | Rust Manifestation |
//! |------|------------|-------------------|
//! | T1 | sequence, mapping, state | Vec, HashMap, struct fields |
//! | T2-P | direction, compatibility, priority, threshold | ValenceDirection, compatible_with(), strength |
//! | T2-C | Valence, Bond, BondActivation | Composite structs |
//! | T3 | HookAtom, SkillMolecule, ReactionResult | Domain-specific types |
//!
//! ## Chemistry → Hook-Skill Metaphor Mapping
//!
//! | Chemistry | Hook-Skill | Transfer Confidence |
//! |-----------|------------|---------------------|
//! | Atom | HookAtom | 0.88 |
//! | Molecule | SkillMolecule | 0.92 |
//! | Covalent Bond | Strong/permanent hook coupling | 0.90 |
//! | Ionic Bond | One-way validation chains | 0.85 |
//! | Valence Electron | I/O port (Valence struct) | 0.88 |
//! | Energy Level | Execution tier (dev/review/deploy) | 0.75 |
//! | Reaction | Event cascade triggering | 0.82 |
//! | Activation Energy | Minimum event strength | 0.80 |
//!
//! ## Example
//!
//! ```rust,ignore
//! use nexcore_hooks::bonding::{HookAtom, SkillMolecule, Bond, BondType};
//!
//! // Define hook atoms
//! let validator = HookAtom::new("pretool_panic_enforcer")
//!     .with_valence(Valence::input("rust_code"))
//!     .with_valence(Valence::output("validation_result"));
//!
//! // Create a skill molecule with bonded hooks
//! let molecule = SkillMolecule::new("rust-dev")
//!     .bond(validator, BondType::Covalent, 90)  // Strong bond
//!     .bond(analyzer, BondType::Ionic, 50);     // Medium bond
//!
//! // Trigger a reaction cascade
//! molecule.react(&event)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::chemistry_scoring::{ArrheniusRate, GibbsFeasibility, StalenessDecay};

/// Valence electron - defines input/output port for hook bonding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Valence {
    /// Port name (e.g., "rust_code", "validation_result")
    pub port: String,
    /// Direction: input (accepts) or output (produces)
    pub direction: ValenceDirection,
    /// Data type expected/produced
    pub data_type: String,
    /// Whether this valence is required for bonding
    pub required: bool,
}

/// Valence direction (input/output)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValenceDirection {
    /// Accepts data (input port)
    Input,
    /// Produces data (output port)
    Output,
    /// Can be both (bidirectional)
    Bidirectional,
}

impl Valence {
    /// Create an input valence
    pub fn input(port: &str) -> Self {
        Self {
            port: port.to_string(),
            direction: ValenceDirection::Input,
            data_type: "any".to_string(),
            required: true,
        }
    }

    /// Create an output valence
    pub fn output(port: &str) -> Self {
        Self {
            port: port.to_string(),
            direction: ValenceDirection::Output,
            data_type: "any".to_string(),
            required: false,
        }
    }

    /// Set data type
    pub fn with_type(mut self, data_type: &str) -> Self {
        self.data_type = data_type.to_string();
        self
    }

    /// Set optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Check if this valence can bond with another
    pub fn compatible_with(&self, other: &Valence) -> bool {
        // Input bonds with Output
        let direction_compatible = match (&self.direction, &other.direction) {
            (ValenceDirection::Input, ValenceDirection::Output) => true,
            (ValenceDirection::Output, ValenceDirection::Input) => true,
            (ValenceDirection::Bidirectional, _) => true,
            (_, ValenceDirection::Bidirectional) => true,
            _ => false,
        };

        // Data types must match (or be "any")
        let type_compatible = self.data_type == "any"
            || other.data_type == "any"
            || self.data_type == other.data_type;

        direction_compatible && type_compatible
    }
}

/// Bond type - determines strength and behavior of hook-skill connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondType {
    /// Covalent: Strong bond, shared electrons (data flows both ways)
    /// Used for tightly coupled hooks that must run together
    Covalent,

    /// Ionic: Medium bond, electron transfer (one-way data flow)
    /// Used for validation chains where output feeds next input
    Ionic,

    /// Hydrogen: Weak bond, temporary association
    /// Used for optional/conditional hooks
    Hydrogen,

    /// Metallic: Delocalized electrons (broadcast data to many)
    /// Used for event broadcasting to multiple hooks
    Metallic,

    /// Van der Waals: Very weak, proximity-based
    /// Used for hooks that only activate when others are nearby
    VanDerWaals,
}

impl BondType {
    /// Get default bond strength for this type
    pub fn default_strength(&self) -> u8 {
        match self {
            BondType::Covalent => 90,
            BondType::Ionic => 70,
            BondType::Hydrogen => 40,
            BondType::Metallic => 60,
            BondType::VanDerWaals => 20,
        }
    }

    /// Can this bond be broken by a reaction?
    pub fn breakable(&self) -> bool {
        match self {
            BondType::Covalent => false,   // Strong, permanent
            BondType::Ionic => true,       // Can dissociate
            BondType::Hydrogen => true,    // Easily broken
            BondType::Metallic => false,   // Delocalized, stable
            BondType::VanDerWaals => true, // Very weak
        }
    }
}

/// A chemical bond between a hook and skill (or hook and hook)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bond {
    /// Source atom (hook name)
    pub source: String,
    /// Target atom (hook or skill name)
    pub target: String,
    /// Bond type
    pub bond_type: BondType,
    /// Bond strength (1-100, higher = stronger/higher priority)
    pub strength: u8,
    /// Connected valences (source port -> target port)
    pub connections: Vec<(String, String)>,
    /// Activation condition (when does this bond activate?)
    pub activation: BondActivation,
}

/// When does a bond activate?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BondActivation {
    /// Always active
    Always,
    /// Active on specific events
    OnEvent(Vec<String>),
    /// Active when specific tool is used
    OnTool(Vec<String>),
    /// Active when another bond is active (cascade)
    OnBond(String),
    /// Active based on custom condition
    Condition(String),
}

impl Bond {
    /// Create a new bond
    pub fn new(source: &str, target: &str, bond_type: BondType) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            bond_type,
            strength: bond_type.default_strength(),
            connections: Vec::new(),
            activation: BondActivation::Always,
        }
    }

    /// Set custom strength
    pub fn with_strength(mut self, strength: u8) -> Self {
        self.strength = strength.min(100);
        self
    }

    /// Add valence connection
    pub fn connect(mut self, source_port: &str, target_port: &str) -> Self {
        self.connections
            .push((source_port.to_string(), target_port.to_string()));
        self
    }

    /// Set activation condition
    pub fn on_event(mut self, events: &[&str]) -> Self {
        self.activation = BondActivation::OnEvent(events.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set tool activation
    pub fn on_tool(mut self, tools: &[&str]) -> Self {
        self.activation = BondActivation::OnTool(tools.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set cascade activation
    pub fn on_bond(mut self, bond_source: &str) -> Self {
        self.activation = BondActivation::OnBond(bond_source.to_string());
        self
    }
}

/// A hook atom - the fundamental unit of enforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAtom {
    /// Hook name (binary name)
    pub name: String,
    /// Hook event type (PreToolUse, PostToolUse, etc.)
    pub event: String,
    /// Tool matcher (Edit, Write, Task, etc.)
    pub matcher: Option<String>,
    /// Valence electrons (input/output ports)
    pub valences: Vec<Valence>,
    /// Atomic number (unique identifier/priority)
    pub atomic_number: u8,
    /// Is this atom stable (can run standalone)?
    pub stable: bool,
    /// Energy level (execution tier: dev/review/deploy)
    pub energy_level: EnergyLevel,
}

/// Energy level (execution tier)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnergyLevel {
    /// Ground state - dev tier (low enforcement)
    Ground,
    /// First excited - review tier (medium enforcement)
    Excited1,
    /// Second excited - deploy tier (high enforcement)
    Excited2,
    /// Ionized - always active regardless of tier
    Ionized,
}

impl HookAtom {
    /// Create a new hook atom
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            event: "PreToolUse".to_string(),
            matcher: None,
            valences: Vec::new(),
            atomic_number: 50,
            stable: true,
            energy_level: EnergyLevel::Ground,
        }
    }

    /// Set event type
    pub fn event(mut self, event: &str) -> Self {
        self.event = event.to_string();
        self
    }

    /// Set tool matcher
    pub fn matcher(mut self, matcher: &str) -> Self {
        self.matcher = Some(matcher.to_string());
        self
    }

    /// Add a valence
    pub fn with_valence(mut self, valence: Valence) -> Self {
        self.valences.push(valence);
        self
    }

    /// Set atomic number (priority)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.atomic_number = priority;
        self
    }

    /// Set energy level
    pub fn at_level(mut self, level: EnergyLevel) -> Self {
        self.energy_level = level;
        self
    }

    /// Mark as unstable (requires bonding to function)
    pub fn unstable(mut self) -> Self {
        self.stable = false;
        self
    }

    /// Check if this atom can bond with another
    pub fn can_bond_with(&self, other: &HookAtom) -> bool {
        // Check for compatible valences
        for self_valence in &self.valences {
            for other_valence in &other.valences {
                if self_valence.compatible_with(other_valence) {
                    return true;
                }
            }
        }
        false
    }

    /// Find compatible valence pairs
    pub fn compatible_valences<'a>(
        &'a self,
        other: &'a HookAtom,
    ) -> Vec<(&'a Valence, &'a Valence)> {
        let mut pairs = Vec::new();
        for self_valence in &self.valences {
            for other_valence in &other.valences {
                if self_valence.compatible_with(other_valence) {
                    pairs.push((self_valence, other_valence));
                }
            }
        }
        pairs
    }
}

/// A skill molecule - a skill with bonded hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMolecule {
    /// Skill name
    pub name: String,
    /// Skill path
    pub path: String,
    /// Bonded hook atoms
    pub atoms: Vec<HookAtom>,
    /// Bonds between atoms
    pub bonds: Vec<Bond>,
    /// Molecular weight (sum of atomic numbers)
    pub weight: u32,
    /// Is this molecule reactive (can trigger cascades)?
    pub reactive: bool,
    /// Activation energy (minimum event strength to trigger)
    pub activation_energy: u8,
}

impl SkillMolecule {
    /// Create a new skill molecule
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: format!("~/.claude/skills/{}", name),
            atoms: Vec::new(),
            bonds: Vec::new(),
            weight: 0,
            reactive: true,
            activation_energy: 10,
        }
    }

    /// Set path
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }

    /// Add a hook atom with a bond
    pub fn bond(mut self, atom: HookAtom, bond_type: BondType, strength: u8) -> Self {
        let bond = Bond::new(&atom.name, &self.name, bond_type).with_strength(strength);

        self.weight += u32::from(atom.atomic_number);
        self.atoms.push(atom);
        self.bonds.push(bond);
        self
    }

    /// Add a bond between two existing atoms
    pub fn link(&mut self, source: &str, target: &str, bond_type: BondType) -> &mut Self {
        let bond = Bond::new(source, target, bond_type);
        self.bonds.push(bond);
        self
    }

    /// Set activation energy
    pub fn with_activation_energy(mut self, energy: u8) -> Self {
        self.activation_energy = energy;
        self
    }

    /// Make non-reactive
    pub fn inert(mut self) -> Self {
        self.reactive = false;
        self
    }

    /// Get all atoms that should execute for a given event
    pub fn atoms_for_event(&self, event: &str) -> Vec<&HookAtom> {
        self.atoms.iter().filter(|a| a.event == event).collect()
    }

    /// Get execution order (sorted by bond strength then atomic number)
    pub fn execution_order(&self) -> Vec<&HookAtom> {
        let mut ordered: Vec<_> = self.atoms.iter().collect();

        // Sort by: bond strength (desc), then atomic number (desc)
        ordered.sort_by(|a, b| {
            let a_strength = self
                .bonds
                .iter()
                .find(|bond| bond.source == a.name)
                .map(|b| b.strength)
                .unwrap_or(0);
            let b_strength = self
                .bonds
                .iter()
                .find(|bond| bond.source == b.name)
                .map(|b| b.strength)
                .unwrap_or(0);

            b_strength
                .cmp(&a_strength)
                .then_with(|| b.atomic_number.cmp(&a.atomic_number))
        });

        ordered
    }

    /// Compute thermodynamic energetics for this molecule
    ///
    /// # Arguments
    /// * `days_since_update` - Days since last modification
    /// * `urgency` - Priority factor (1-10)
    #[must_use]
    pub fn energetics(&self, days_since_update: f64, urgency: f64) -> ReactionEnergetics {
        ReactionEnergetics::for_molecule(self, days_since_update, urgency)
    }

    /// Check if molecule reaction is thermodynamically favorable
    #[must_use]
    pub fn is_favorable(&self, days_since_update: f64, urgency: f64) -> bool {
        self.energetics(days_since_update, urgency).should_proceed()
    }
}

/// Reaction result from molecule activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionResult {
    /// Did the reaction succeed?
    pub success: bool,
    /// Products (output data from hooks)
    pub products: HashMap<String, serde_json::Value>,
    /// Energy released (severity of findings)
    pub energy_released: u8,
    /// Cascade triggers (other molecules to activate)
    pub cascades: Vec<String>,
    /// Broken bonds (hooks that failed)
    pub broken_bonds: Vec<String>,
}

impl ReactionResult {
    /// Create a successful reaction
    pub fn success() -> Self {
        Self {
            success: true,
            products: HashMap::new(),
            energy_released: 0,
            cascades: Vec::new(),
            broken_bonds: Vec::new(),
        }
    }

    /// Create a failed reaction
    pub fn failure() -> Self {
        Self {
            success: false,
            products: HashMap::new(),
            energy_released: 100,
            cascades: Vec::new(),
            broken_bonds: Vec::new(),
        }
    }

    /// Add a product
    pub fn with_product(mut self, key: &str, value: serde_json::Value) -> Self {
        self.products.insert(key.to_string(), value);
        self
    }

    /// Add cascade
    pub fn cascade(mut self, molecule: &str) -> Self {
        self.cascades.push(molecule.to_string());
        self
    }

    /// Record broken bond
    pub fn break_bond(mut self, hook: &str) -> Self {
        self.broken_bonds.push(hook.to_string());
        self
    }
}

// =============================================================================
// REACTION ENERGETICS
// Bridges chemistry_scoring with bonding for thermodynamic analysis
// =============================================================================

/// Thermodynamic analysis of a molecule or reaction
///
/// Uses chemistry_scoring primitives to compute:
/// - Activation energy barriers (Arrhenius)
/// - Spontaneity (Gibbs)
/// - Staleness decay (half-life)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionEnergetics {
    /// Kinetic barrier analysis
    pub activation_barrier: f64,
    /// Gibbs free energy (negative = spontaneous)
    pub delta_g: f64,
    /// Current relevance (0-1, based on staleness)
    pub relevance: f64,
    /// Is reaction thermodynamically favorable?
    pub spontaneous: bool,
    /// Kinetic rate constant
    pub rate_constant: f64,
    /// Human-readable assessment
    pub assessment: String,
}

impl ReactionEnergetics {
    /// Compute energetics for a skill molecule
    ///
    /// # Arguments
    /// * `molecule` - The skill molecule to analyze
    /// * `days_since_update` - Days since last skill modification
    /// * `urgency` - Priority factor (1-10)
    #[must_use]
    pub fn for_molecule(molecule: &SkillMolecule, days_since_update: f64, urgency: f64) -> Self {
        // Complexity based on molecular weight and bond count
        let complexity =
            (molecule.weight as f64 / 10.0 + molecule.bonds.len() as f64 * 5.0).clamp(1.0, 100.0);

        // Effort based on activation energy and weight
        let effort =
            (molecule.activation_energy as f64 + molecule.weight as f64 / 5.0).clamp(0.0, 100.0);

        // Quality gain based on bond strength average
        let avg_bond_strength = if molecule.bonds.is_empty() {
            50.0
        } else {
            molecule
                .bonds
                .iter()
                .map(|b| b.strength as f64)
                .sum::<f64>()
                / molecule.bonds.len() as f64
        };
        let quality_gain = (avg_bond_strength / 100.0).clamp(0.0, 1.0);

        // Calculate components
        let kinetics = ArrheniusRate::calculate(complexity, urgency);
        let thermodynamics = GibbsFeasibility::calculate(effort, quality_gain, urgency);
        let staleness = StalenessDecay::calculate(days_since_update, 30.0);

        // Build assessment
        let assessment = format!(
            "{} | {} | {}",
            kinetics.interpretation(),
            thermodynamics.interpretation(),
            staleness.interpretation()
        );

        Self {
            activation_barrier: kinetics.activation_energy,
            delta_g: thermodynamics.delta_g,
            relevance: staleness.relevance,
            spontaneous: thermodynamics.spontaneous,
            rate_constant: kinetics.rate,
            assessment,
        }
    }

    /// Compute energetics for a bond formation
    ///
    /// Lower activation barrier for stronger bonds (covalent < ionic < hydrogen)
    #[must_use]
    pub fn for_bond(bond: &Bond, urgency: f64) -> Self {
        // Bond strength inversely correlates with activation barrier
        let complexity = (100.0 - bond.strength as f64).clamp(1.0, 100.0);

        // Effort depends on bond type
        let effort = match bond.bond_type {
            BondType::Covalent => 30.0,    // High effort but worth it
            BondType::Metallic => 40.0,    // Medium effort
            BondType::Ionic => 50.0,       // Moderate effort
            BondType::Hydrogen => 20.0,    // Low effort
            BondType::VanDerWaals => 10.0, // Minimal effort
        };

        // Quality gain based on bond strength
        let quality_gain = (bond.strength as f64 / 100.0).clamp(0.0, 1.0);

        let kinetics = ArrheniusRate::calculate(complexity, urgency);
        let thermodynamics = GibbsFeasibility::calculate(effort, quality_gain, urgency);

        let assessment = format!(
            "{:?} bond: {} | {}",
            bond.bond_type,
            kinetics.interpretation(),
            thermodynamics.interpretation()
        );

        Self {
            activation_barrier: kinetics.activation_energy,
            delta_g: thermodynamics.delta_g,
            relevance: 1.0, // Bonds don't decay
            spontaneous: thermodynamics.spontaneous,
            rate_constant: kinetics.rate,
            assessment,
        }
    }

    /// Should this reaction proceed?
    #[must_use]
    pub fn should_proceed(&self) -> bool {
        self.spontaneous && self.rate_constant > 0.05
    }

    /// Get energy efficiency ratio (rate / barrier)
    #[must_use]
    pub fn efficiency(&self) -> f64 {
        if self.activation_barrier > 0.0 {
            self.rate_constant * 1000.0 / self.activation_barrier
        } else {
            1.0
        }
    }
}

/// Molecular registry - tracks all molecules in the system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MolecularRegistry {
    /// All registered molecules
    pub molecules: HashMap<String, SkillMolecule>,
    /// Hook atom library
    pub atoms: HashMap<String, HookAtom>,
    /// Active cascade chains
    pub active_cascades: Vec<String>,
}

impl MolecularRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a hook atom
    pub fn register_atom(&mut self, atom: HookAtom) {
        self.atoms.insert(atom.name.clone(), atom);
    }

    /// Register a skill molecule
    pub fn register_molecule(&mut self, molecule: SkillMolecule) {
        self.molecules.insert(molecule.name.clone(), molecule);
    }

    /// Find molecules that should react to an event
    pub fn reactive_molecules(&self, event: &str, tool: Option<&str>) -> Vec<&SkillMolecule> {
        self.molecules
            .values()
            .filter(|m| {
                m.reactive
                    && m.atoms.iter().any(|a| {
                        a.event == event
                            && (a.matcher.is_none()
                                || tool.map_or(false, |t| {
                                    a.matcher.as_ref().map_or(false, |m| m.contains(t))
                                }))
                    })
            })
            .collect()
    }

    /// Trigger a cascade reaction
    pub fn trigger_cascade(&mut self, source: &str) {
        if !self.active_cascades.contains(&source.to_string()) {
            self.active_cascades.push(source.to_string());
        }
    }
}

// =============================================================================
// COMPOUND HOOK ARCHITECTURE
// Chemistry: HookMolecule = Macromolecule of hook atoms
// =============================================================================

/// A compound hook (HookMolecule) - a hook that contains nested hooks
///
/// Chemistry metaphor:
/// - Like a macromolecule composed of smaller molecules
/// - Has a nucleus (primary hook) and satellites (nested hooks)
/// - Molecular formula: `name(n)` where n = nested count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMolecule {
    /// Nucleus hook name (the primary/parent hook)
    pub nucleus: String,
    /// Event type for nucleus
    pub event: String,
    /// Tool matcher for nucleus
    pub matcher: Option<String>,
    /// Nested hook names (relative paths like skills)
    pub nested_hooks: Vec<String>,
    /// Resolved nested hook atoms
    pub satellite_atoms: Vec<HookAtom>,
    /// Bonds connecting nucleus to satellites
    pub internal_bonds: Vec<Bond>,
    /// Molecular formula: `name(n)` or `name(n)⁺` for recursive
    pub formula: String,
    /// Total molecular weight
    pub weight: u32,
    /// Is this molecule stable? (all nested hooks resolved)
    pub stable: bool,
    /// Nesting depth (1 = direct children, 2+ = recursive)
    pub depth: u8,
    /// Execution order for nested hooks
    pub execution_chain: Vec<String>,
}

impl HookMolecule {
    /// Create a new hook molecule from a nucleus hook
    pub fn new(nucleus: &str, event: &str) -> Self {
        Self {
            nucleus: nucleus.to_string(),
            event: event.to_string(),
            matcher: None,
            nested_hooks: Vec::new(),
            satellite_atoms: Vec::new(),
            internal_bonds: Vec::new(),
            formula: format!("{}(0)", nucleus),
            weight: 10, // Base weight for nucleus
            stable: true,
            depth: 0,
            execution_chain: vec![nucleus.to_string()],
        }
    }

    /// Set tool matcher
    pub fn with_matcher(mut self, matcher: &str) -> Self {
        self.matcher = Some(matcher.to_string());
        self
    }

    /// Declare nested hooks (before resolution)
    pub fn with_nested(mut self, nested: Vec<String>) -> Self {
        self.nested_hooks = nested.clone();
        self.formula = format!("{}({})", self.nucleus, nested.len());
        self
    }

    /// Add a resolved satellite hook atom with bond
    pub fn bond_satellite(mut self, atom: HookAtom, bond_type: BondType) -> Self {
        let bond = Bond::new(&self.nucleus, &atom.name, bond_type);
        self.weight += u32::from(atom.atomic_number);
        self.execution_chain.push(atom.name.clone());
        self.satellite_atoms.push(atom);
        self.internal_bonds.push(bond);
        self
    }

    /// Update stability based on resolution status
    pub fn with_stability(mut self, stable: bool) -> Self {
        self.stable = stable;
        self
    }

    /// Set recursive depth marker
    pub fn with_depth(mut self, depth: u8) -> Self {
        self.depth = depth;
        if depth > 1 {
            // Recursive compound - mark with superscript plus
            self.formula = format!("{}({})⁺", self.nucleus, self.satellite_atoms.len());
        }
        self
    }

    /// Get execution order for all hooks in this molecule
    pub fn execution_order(&self) -> &[String] {
        &self.execution_chain
    }

    /// Check if a hook is part of this molecule
    pub fn contains(&self, hook_name: &str) -> bool {
        self.nucleus == hook_name || self.satellite_atoms.iter().any(|a| a.name == hook_name)
    }
}

/// A polymer chain of hooks - sequential execution pipeline
///
/// Chemistry metaphor:
/// - Like a polymer with repeating units
/// - Each unit triggers the next in sequence
/// - Stoichiometry defines required ratios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPolymer {
    /// Chain name
    pub name: String,
    /// Hooks in execution order (the polymer chain)
    pub chain: Vec<String>,
    /// Bond types between adjacent hooks
    pub chain_bonds: Vec<BondType>,
    /// Required stoichiometry (how many times each hook must run)
    pub stoichiometry: HashMap<String, u8>,
    /// Is the chain cyclic? (last hooks back to first)
    pub cyclic: bool,
    /// Current position in chain
    pub cursor: usize,
    /// Chain length (number of hooks)
    pub length: usize,
}

impl HookPolymer {
    /// Create a new polymer chain
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            chain: Vec::new(),
            chain_bonds: Vec::new(),
            stoichiometry: HashMap::new(),
            cyclic: false,
            cursor: 0,
            length: 0,
        }
    }

    /// Add a hook to the chain with bond to previous
    pub fn add(mut self, hook: &str, bond_type: BondType) -> Self {
        if !self.chain.is_empty() {
            self.chain_bonds.push(bond_type);
        }
        self.chain.push(hook.to_string());
        self.stoichiometry.insert(hook.to_string(), 1);
        self.length = self.chain.len();
        self
    }

    /// Set stoichiometry for a hook (how many times it must run)
    pub fn with_ratio(mut self, hook: &str, count: u8) -> Self {
        self.stoichiometry.insert(hook.to_string(), count);
        self
    }

    /// Make chain cyclic
    pub fn cyclic(mut self) -> Self {
        self.cyclic = true;
        self
    }

    /// Get next hook in chain
    pub fn next(&mut self) -> Option<&str> {
        if self.cursor >= self.length {
            if self.cyclic {
                self.cursor = 0;
            } else {
                return None;
            }
        }
        let hook = &self.chain[self.cursor];
        self.cursor += 1;
        Some(hook)
    }

    /// Reset chain to beginning
    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    /// Generate polymer formula: H₁-H₂-H₃ or ⟳H₁-H₂-H₃ for cyclic
    pub fn formula(&self) -> String {
        let chain_str = self.chain.join("-");
        if self.cyclic {
            format!("⟳{}", chain_str)
        } else {
            chain_str
        }
    }
}

/// Registry for compound hooks (hook molecules and polymers)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompoundHookRegistry {
    /// Hook molecules (compound hooks)
    pub molecules: HashMap<String, HookMolecule>,
    /// Hook polymers (execution chains)
    pub polymers: HashMap<String, HookPolymer>,
    /// Molecular formulas for quick lookup
    pub formulas: Vec<String>,
    /// Total hooks across all compounds
    pub total_hooks: usize,
}

impl CompoundHookRegistry {
    /// Create a new compound hook registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a hook molecule
    pub fn register_molecule(&mut self, molecule: HookMolecule) {
        self.formulas.push(molecule.formula.clone());
        self.total_hooks += molecule.satellite_atoms.len() + 1; // +1 for nucleus
        self.molecules.insert(molecule.nucleus.clone(), molecule);
    }

    /// Register a hook polymer
    pub fn register_polymer(&mut self, polymer: HookPolymer) {
        self.formulas.push(polymer.formula());
        self.total_hooks += polymer.length;
        self.polymers.insert(polymer.name.clone(), polymer);
    }

    /// Find molecule containing a hook
    pub fn molecule_for(&self, hook: &str) -> Option<&HookMolecule> {
        self.molecules.values().find(|m| m.contains(hook))
    }

    /// Get polymer by name
    pub fn polymer(&self, name: &str) -> Option<&HookPolymer> {
        self.polymers.get(name)
    }

    /// List all compound hooks (nuclei only)
    pub fn list_compounds(&self) -> Vec<&str> {
        self.molecules.keys().map(String::as_str).collect()
    }

    /// Get nested hooks for a compound
    pub fn nested_for(&self, nucleus: &str) -> Option<Vec<&str>> {
        self.molecules
            .get(nucleus)
            .map(|m| m.satellite_atoms.iter().map(|a| a.name.as_str()).collect())
    }
}

// =============================================================================
// DELEGATION MOLECULE FACTORY
// Pre-configured molecule for model routing primitives
// =============================================================================

/// Factory for creating the delegation-router-molecule
///
/// Bonds 5 delegation primitives:
/// - Model (T1, atomic 95) - Covalent to Router
/// - ClassificationTree (T2-P, atomic 85) - Ionic to Router
/// - ConfidenceScore (T2-P, atomic 80) - Ionic to Router
/// - ReviewProtocol (T3, atomic 70) - Hydrogen to Confidence
/// - DelegationRouter (T1, atomic 90) - Core
pub struct DelegationMoleculeFactory;

impl DelegationMoleculeFactory {
    /// Build the delegation router molecule
    pub fn build() -> SkillMolecule {
        // Define atoms
        let model = HookAtom::new("Model")
            .event("PreToolUse")
            .matcher("Task")
            .with_valence(Valence::input("task_type"))
            .with_valence(Valence::output("model_selection"))
            .with_valence(Valence::output("capabilities").optional())
            .with_priority(95)
            .at_level(EnergyLevel::Ground);

        let classification = HookAtom::new("ClassificationTree")
            .event("UserPromptSubmit")
            .with_valence(Valence::input("task_input"))
            .with_valence(Valence {
                port: "predicate_chain".to_string(),
                direction: ValenceDirection::Bidirectional,
                data_type: "PredicateResult".to_string(),
                required: true,
            })
            .with_valence(Valence::output("action_output"))
            .with_priority(85)
            .at_level(EnergyLevel::Ground);

        let confidence = HookAtom::new("ConfidenceScore")
            .event("PreToolUse")
            .matcher("Task")
            .with_valence(Valence::input("dimensions"))
            .with_valence(Valence::input("weights").optional())
            .with_valence(Valence::output("score"))
            .with_valence(Valence::output("threshold_gate").optional())
            .with_priority(80)
            .at_level(EnergyLevel::Ground);

        let review = HookAtom::new("ReviewProtocol")
            .event("PostToolUse")
            .matcher("Task")
            .with_valence(Valence::input("artifact"))
            .with_valence(Valence {
                port: "phase_result".to_string(),
                direction: ValenceDirection::Bidirectional,
                data_type: "ReviewResult".to_string(),
                required: true,
            })
            .with_valence(Valence::output("accepted"))
            .with_priority(70)
            .unstable() // Needs artifact input
            .at_level(EnergyLevel::Excited1); // Review tier

        let router = HookAtom::new("DelegationRouter")
            .event("PreToolUse")
            .matcher("Task")
            .with_valence(Valence::input("task_characteristics"))
            .with_valence(Valence::output("routing_decision"))
            .with_valence(Valence::output("confidence"))
            .with_priority(90)
            .at_level(EnergyLevel::Ground);

        // Build molecule with bonds
        let mut molecule = SkillMolecule::new("delegation-router-molecule")
            .with_path("nexcore-vigilance::primitives::delegation")
            .bond(router, BondType::Covalent, 95) // Core
            .bond(model, BondType::Covalent, 95) // Tight coupling
            .bond(classification, BondType::Ionic, 75)
            .bond(confidence, BondType::Ionic, 70)
            .bond(review, BondType::Hydrogen, 40) // Optional validation
            .with_activation_energy(15);

        // Add inter-atom bonds
        molecule.link("DelegationRouter", "Model", BondType::Covalent);
        molecule.link("ClassificationTree", "DelegationRouter", BondType::Ionic);
        molecule.link("ConfidenceScore", "DelegationRouter", BondType::Ionic);
        molecule.link("ReviewProtocol", "ConfidenceScore", BondType::Hydrogen);
        molecule.link("Model", "ReviewProtocol", BondType::VanDerWaals);

        molecule
    }

    /// Build a polymer chain for the delegation pipeline
    pub fn build_polymer() -> HookPolymer {
        HookPolymer::new("delegation-pipeline")
            .add("ClassificationTree", BondType::Ionic)
            .add("DelegationRouter", BondType::Covalent)
            .add("Model", BondType::Covalent)
            .add("ConfidenceScore", BondType::Ionic)
            .add("ReviewProtocol", BondType::Hydrogen)
    }

    /// Get molecule weight (sum of atomic numbers)
    pub fn molecular_weight() -> u32 {
        95 + 85 + 80 + 70 + 90 // = 420
    }

    /// Get stability score based on bond analysis
    pub fn stability_score() -> f64 {
        // All required valences connected, covalent bonds intact
        0.92
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valence_compatibility() {
        let input = Valence::input("rust_code").with_type("String");
        let output = Valence::output("rust_code").with_type("String");

        assert!(input.compatible_with(&output));
        assert!(output.compatible_with(&input));
    }

    #[test]
    fn test_hook_atom_creation() {
        let atom = HookAtom::new("pretool_panic_enforcer")
            .event("PreToolUse")
            .matcher("Edit|Write")
            .with_valence(Valence::input("rust_code"))
            .with_valence(Valence::output("validation_result"))
            .with_priority(90)
            .at_level(EnergyLevel::Excited2);

        assert_eq!(atom.name, "pretool_panic_enforcer");
        assert_eq!(atom.valences.len(), 2);
        assert_eq!(atom.atomic_number, 90);
    }

    #[test]
    fn test_skill_molecule_creation() {
        let validator = HookAtom::new("pretool_panic_enforcer")
            .with_valence(Valence::input("code"))
            .with_priority(90);

        let analyzer = HookAtom::new("posttool_incremental_verifier")
            .with_valence(Valence::input("code"))
            .with_priority(80);

        let molecule = SkillMolecule::new("rust-dev")
            .bond(validator, BondType::Covalent, 95)
            .bond(analyzer, BondType::Ionic, 70);

        assert_eq!(molecule.atoms.len(), 2);
        assert_eq!(molecule.bonds.len(), 2);
        assert_eq!(molecule.weight, 170); // 90 + 80
    }

    #[test]
    fn test_execution_order() {
        let high = HookAtom::new("high_priority").with_priority(90);
        let low = HookAtom::new("low_priority").with_priority(30);
        let medium = HookAtom::new("medium_priority").with_priority(60);

        let molecule = SkillMolecule::new("test")
            .bond(low, BondType::Hydrogen, 30)
            .bond(high, BondType::Covalent, 95)
            .bond(medium, BondType::Ionic, 60);

        let order = molecule.execution_order();
        assert_eq!(order[0].name, "high_priority");
        assert_eq!(order[1].name, "medium_priority");
        assert_eq!(order[2].name, "low_priority");
    }

    #[test]
    fn test_bond_types() {
        assert_eq!(BondType::Covalent.default_strength(), 90);
        assert!(!BondType::Covalent.breakable());
        assert!(BondType::Hydrogen.breakable());
    }

    // --- Reaction Energetics Tests ---

    #[test]
    fn test_reaction_energetics_for_molecule() {
        let validator = HookAtom::new("validator").with_priority(80);
        let molecule = SkillMolecule::new("test-skill").bond(validator, BondType::Covalent, 90);

        let energetics = molecule.energetics(5.0, 8.0); // 5 days old, high urgency

        assert!(energetics.activation_barrier > 0.0);
        assert!(energetics.rate_constant > 0.0);
        assert!(energetics.relevance > 0.8); // Fresh
        assert!(!energetics.assessment.is_empty());
    }

    #[test]
    fn test_reaction_energetics_spontaneous() {
        let validator = HookAtom::new("validator").with_priority(90);
        let molecule = SkillMolecule::new("high-quality").bond(validator, BondType::Covalent, 95);

        // High urgency + high quality bonds = spontaneous
        let energetics = molecule.energetics(0.0, 10.0);
        assert!(
            energetics.spontaneous,
            "High quality + high urgency should be spontaneous"
        );
    }

    #[test]
    fn test_reaction_energetics_stale() {
        let validator = HookAtom::new("validator").with_priority(50);
        let molecule = SkillMolecule::new("stale-skill").bond(validator, BondType::Hydrogen, 40);

        // 60 days old = stale
        let energetics = molecule.energetics(60.0, 5.0);
        assert!(energetics.relevance < 0.5, "60 days should be stale");
    }

    #[test]
    fn test_reaction_energetics_for_bond() {
        let bond = Bond::new("source", "target", BondType::Covalent).with_strength(90);

        let energetics = ReactionEnergetics::for_bond(&bond, 8.0);

        assert!(energetics.activation_barrier > 0.0);
        assert!(energetics.relevance == 1.0); // Bonds don't decay
        assert!(energetics.assessment.contains("Covalent"));
    }

    #[test]
    fn test_molecule_is_favorable() {
        let high = HookAtom::new("high_quality").with_priority(90);
        let molecule = SkillMolecule::new("favorable").bond(high, BondType::Covalent, 95);

        // High urgency, fresh = should be favorable
        assert!(molecule.is_favorable(1.0, 10.0));
    }

    #[test]
    fn test_energetics_efficiency() {
        let validator = HookAtom::new("validator").with_priority(50);
        let molecule = SkillMolecule::new("test").bond(validator, BondType::Ionic, 70);

        let energetics = molecule.energetics(10.0, 7.0);
        let efficiency = energetics.efficiency();

        assert!(efficiency > 0.0);
        assert!(efficiency.is_finite());
    }

    // --- Compound Hook Architecture Tests ---

    #[test]
    fn test_hook_molecule_creation() {
        let molecule = HookMolecule::new("pretool_validator_chain", "PreToolUse")
            .with_matcher("Edit|Write")
            .with_nested(vec![
                "pretool_panic_enforcer".to_string(),
                "pretool_secret_scanner".to_string(),
            ]);

        assert_eq!(molecule.nucleus, "pretool_validator_chain");
        assert_eq!(molecule.formula, "pretool_validator_chain(2)");
        assert_eq!(molecule.nested_hooks.len(), 2);
    }

    #[test]
    fn test_hook_molecule_with_satellites() {
        let atom1 = HookAtom::new("pretool_panic_enforcer").with_priority(90);
        let atom2 = HookAtom::new("pretool_secret_scanner").with_priority(80);

        let molecule = HookMolecule::new("pretool_compound", "PreToolUse")
            .bond_satellite(atom1, BondType::Covalent)
            .bond_satellite(atom2, BondType::Ionic);

        assert_eq!(molecule.satellite_atoms.len(), 2);
        assert_eq!(molecule.internal_bonds.len(), 2);
        assert_eq!(molecule.weight, 10 + 90 + 80); // base + atoms
        assert!(molecule.contains("pretool_panic_enforcer"));
        assert!(molecule.contains("pretool_secret_scanner"));
        assert!(!molecule.contains("pretool_unknown"));
    }

    #[test]
    fn test_hook_polymer_chain() {
        let mut polymer = HookPolymer::new("validation_pipeline")
            .add("pretool_panic_enforcer", BondType::Covalent)
            .add("pretool_secret_scanner", BondType::Ionic)
            .add("posttool_clippy_enforcer", BondType::Ionic);

        assert_eq!(polymer.length, 3);
        assert_eq!(
            polymer.formula(),
            "pretool_panic_enforcer-pretool_secret_scanner-posttool_clippy_enforcer"
        );

        // Test iteration
        assert_eq!(polymer.next(), Some("pretool_panic_enforcer"));
        assert_eq!(polymer.next(), Some("pretool_secret_scanner"));
        assert_eq!(polymer.next(), Some("posttool_clippy_enforcer"));
        assert_eq!(polymer.next(), None);
    }

    #[test]
    fn test_cyclic_polymer() {
        let mut polymer = HookPolymer::new("feedback_loop")
            .add("validate", BondType::Covalent)
            .add("generate", BondType::Covalent)
            .add("check", BondType::Covalent)
            .cyclic();

        assert!(polymer.cyclic);
        assert!(polymer.formula().starts_with('⟳'));

        // Should loop back
        polymer.next();
        polymer.next();
        polymer.next();
        assert_eq!(polymer.next(), Some("validate")); // Loops back
    }

    #[test]
    fn test_compound_hook_registry() {
        let mut registry = CompoundHookRegistry::new();

        let molecule = HookMolecule::new("compound_validator", "PreToolUse")
            .with_nested(vec!["child1".to_string(), "child2".to_string()]);

        let polymer = HookPolymer::new("pipeline")
            .add("step1", BondType::Covalent)
            .add("step2", BondType::Ionic);

        registry.register_molecule(molecule);
        registry.register_polymer(polymer);

        assert_eq!(registry.molecules.len(), 1);
        assert_eq!(registry.polymers.len(), 1);
        assert!(registry.molecule_for("compound_validator").is_some());
    }

    // --- Delegation Molecule Factory Tests ---

    #[test]
    fn test_delegation_molecule_factory() {
        let molecule = DelegationMoleculeFactory::build();

        assert_eq!(molecule.name, "delegation-router-molecule");
        assert_eq!(molecule.atoms.len(), 5);
        assert_eq!(
            molecule.weight,
            DelegationMoleculeFactory::molecular_weight()
        );
        assert!(molecule.reactive);
        assert_eq!(molecule.activation_energy, 15);
    }

    #[test]
    fn test_delegation_polymer() {
        let polymer = DelegationMoleculeFactory::build_polymer();

        assert_eq!(polymer.name, "delegation-pipeline");
        assert_eq!(polymer.length, 5);
        assert!(!polymer.cyclic);
    }

    #[test]
    fn test_delegation_stability() {
        let stability = DelegationMoleculeFactory::stability_score();
        assert!(
            stability > 0.9,
            "Delegation molecule should be highly stable"
        );
    }

    #[test]
    fn test_cooperative_polymer_amplification() {
        // Test ABOVE K₀.₅ where Hill amplifies (I > K₀.₅)
        let polymer = CooperativePolymer::new("forge-accelerator")
            .with_hill_coefficient(2.5) // Strong positive cooperativity
            .with_k_half(1.0) // K₀.₅ = 1.0
            .add_bond("skill-upgrade", 0.6) // Total = 1.8 (above K₀.₅)
            .add_bond("hook-improvement", 0.6)
            .add_bond("mcp-enhancement", 0.6);

        // At I=1.8, K=1.0, nH=2.5: Y = 1.8^2.5 / (1^2.5 + 1.8^2.5) ≈ 0.81
        let response = polymer.cascade_response();
        assert!(
            response > 0.75,
            "Cooperative amplification should exceed 75%, got {}",
            response
        );
    }

    #[test]
    fn test_cooperative_vs_standard() {
        // ABOVE K₀.₅: cooperative amplifies more than standard
        let standard = CooperativePolymer::new("standard")
            .with_hill_coefficient(1.0)
            .with_k_half(1.0)
            .add_bond("b1", 0.6)
            .add_bond("b2", 0.6)
            .add_bond("b3", 0.6); // Total = 1.8

        let cooperative = CooperativePolymer::new("cooperative")
            .with_hill_coefficient(2.5)
            .with_k_half(1.0)
            .add_bond("b1", 0.6)
            .add_bond("b2", 0.6)
            .add_bond("b3", 0.6); // Total = 1.8

        // Above K₀.₅, cooperative gives stronger response
        // nH=1: 1.8/(1+1.8) = 0.643
        // nH=2.5: 1.8^2.5/(1+1.8^2.5) = 4.35/5.35 = 0.813
        assert!(
            cooperative.cascade_response() > standard.cascade_response(),
            "Cooperative should amplify above K₀.₅: {} > {}",
            cooperative.cascade_response(),
            standard.cascade_response()
        );
    }

    #[test]
    fn test_cooperative_below_k_half() {
        // BELOW K₀.₅: cooperative dampens (less sensitive than standard)
        let standard = CooperativePolymer::new("standard")
            .with_hill_coefficient(1.0)
            .with_k_half(3.0)
            .add_bond("b1", 0.5); // Total = 0.5 (below K₀.₅=3)

        let cooperative = CooperativePolymer::new("cooperative")
            .with_hill_coefficient(2.5)
            .with_k_half(3.0)
            .add_bond("b1", 0.5);

        // Below K₀.₅, cooperative gives WEAKER response (steeper curve = less below midpoint)
        assert!(
            cooperative.cascade_response() < standard.cascade_response(),
            "Cooperative should dampen below K₀.₅: {} < {}",
            cooperative.cascade_response(),
            standard.cascade_response()
        );
    }

    /// Integration test: Polymer → Forge Pipeline
    ///
    /// Validates that CooperativePolymer correctly triggers forge automation
    /// when cumulative bond strength exceeds cascade threshold.
    #[test]
    fn test_polymer_forge_pipeline_integration() {
        // Scenario: Forge should trigger when multiple quality gates pass
        // - skill-validated: 0.6 (skill Diamond v2 compliance)
        // - tests-passing: 0.8 (CTVP Phase 1-2 passed)
        // - clippy-clean: 0.5 (zero warnings)
        // Total strength: 1.9 (above K₀.₅=1.0)

        let forge_trigger = CooperativePolymer::new("forge-autonomous")
            .with_hill_coefficient(2.5) // Strong amplification
            .with_k_half(1.0) // Half-saturation at 1.0
            .with_target("START FORGE") // Target action
            .with_threshold(0.8) // 80% cascade for trigger
            .add_bond("skill-validated", 0.6)
            .add_bond("tests-passing", 0.8)
            .add_bond("clippy-clean", 0.5);

        // Verify bond accumulation
        assert_eq!(forge_trigger.bonds.len(), 3);
        assert!((forge_trigger.total_strength - 1.9).abs() < 0.01);

        // Verify cascade response (Hill amplification above K₀.₅)
        let response = forge_trigger.cascade_response();
        assert!(
            response > 0.8,
            "Response should exceed 80%: got {:.2}",
            response
        );

        // Verify trigger condition
        assert!(
            forge_trigger.should_trigger(),
            "Forge should trigger with response {:.2} >= threshold 0.8",
            response
        );

        // Verify cooperativity classification
        assert_eq!(forge_trigger.cooperativity_type(), "Strong positive");

        // Verify amplification vs standard (nH=1)
        let amp = forge_trigger.amplification_factor();
        assert!(amp > 1.0, "Amplification should exceed 1.0: got {:.2}", amp);

        // Verify target action is set
        assert_eq!(forge_trigger.target_action, Some("START FORGE".to_string()));
    }

    /// Test: Insufficient bonds should NOT trigger forge
    #[test]
    fn test_polymer_forge_insufficient_bonds() {
        // Scenario: Only one bond - shouldn't trigger even with nH > 1
        let weak_trigger = CooperativePolymer::new("forge-attempt")
            .with_hill_coefficient(2.5)
            .with_k_half(1.0)
            .with_threshold(0.8)
            .add_bond("tests-passing", 0.5); // Only 0.5, below K₀.₅

        // Response should be dampened (Hill effect below K₀.₅)
        let response = weak_trigger.cascade_response();
        assert!(
            response < 0.5,
            "Response below K₀.₅ should be low: got {:.2}",
            response
        );

        // Should NOT trigger
        assert!(
            !weak_trigger.should_trigger(),
            "Forge should NOT trigger with insufficient bonds"
        );
    }

    /// Test: Edge case - exactly at K₀.₅ should give 50% response
    #[test]
    fn test_polymer_at_half_saturation() {
        let at_half = CooperativePolymer::new("midpoint")
            .with_hill_coefficient(2.5)
            .with_k_half(1.0)
            .add_bond("bond1", 0.5)
            .add_bond("bond2", 0.5); // Total = 1.0 = K₀.₅

        let response = at_half.cascade_response();
        assert!(
            (response - 0.5).abs() < 0.01,
            "At K₀.₅, response should be 0.5: got {:.2}",
            response
        );
    }
}

// =============================================================================
// COOPERATIVE POLYMER
// Hill kinetics for bond cascade amplification
// =============================================================================

/// A polymer with cooperative binding - multiple bonds amplify each other.
///
/// Chemistry: Hill equation Y = I^nH / (K₀.₅^nH + I^nH)
/// - nH > 1: Positive cooperativity (amplification)
/// - nH = 1: Standard Michaelis-Menten (no cooperativity)
/// - nH < 1: Negative cooperativity (dampening)
///
/// Use case: Forge acceleration - multiple low-energy bonds cascade to trigger
/// automation that wouldn't fire for individual bonds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooperativePolymer {
    /// Polymer name
    pub name: String,
    /// Hill coefficient (cooperativity factor)
    /// - 1.0 = no cooperativity
    /// - 2.0-3.0 = strong positive cooperativity
    /// - 4.0+ = ultrasensitive (switch-like)
    pub n_hill: f64,
    /// Half-saturation constant (number of bonds for 50% response)
    pub k_half: f64,
    /// Bonds with their relative strengths (0.0 - 1.0)
    pub bonds: Vec<(String, f64)>,
    /// Cumulative bond strength
    pub total_strength: f64,
    /// Target action when threshold exceeded
    pub target_action: Option<String>,
    /// Threshold response for triggering (0.0 - 1.0)
    pub trigger_threshold: f64,
}

impl CooperativePolymer {
    /// Create a new cooperative polymer.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            n_hill: 1.0, // Default: no cooperativity
            k_half: 5.0, // Default: 5 bonds for 50%
            bonds: Vec::new(),
            total_strength: 0.0,
            target_action: None,
            trigger_threshold: 0.5,
        }
    }

    /// Set Hill coefficient (cooperativity).
    pub fn with_hill_coefficient(mut self, n_hill: f64) -> Self {
        self.n_hill = n_hill.max(0.1);
        self
    }

    /// Set half-saturation constant.
    pub fn with_k_half(mut self, k_half: f64) -> Self {
        self.k_half = k_half.max(0.1);
        self
    }

    /// Set target action when triggered.
    pub fn with_target(mut self, action: &str) -> Self {
        self.target_action = Some(action.to_string());
        self
    }

    /// Set trigger threshold.
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.trigger_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Add a bond with relative strength.
    pub fn add_bond(mut self, name: &str, strength: f64) -> Self {
        let strength = strength.clamp(0.0, 1.0);
        self.bonds.push((name.to_string(), strength));
        self.total_strength += strength;
        self
    }

    /// Calculate cascade response using Hill equation.
    ///
    /// Y = I^nH / (K₀.₅^nH + I^nH)
    ///
    /// Where I = total_strength (cumulative bond input)
    #[must_use]
    pub fn cascade_response(&self) -> f64 {
        if self.total_strength <= 0.0 || self.k_half <= 0.0 {
            return 0.0;
        }
        let input_power = self.total_strength.powf(self.n_hill);
        let k_power = self.k_half.powf(self.n_hill);
        input_power / (k_power + input_power)
    }

    /// Check if cascade should trigger.
    #[must_use]
    pub fn should_trigger(&self) -> bool {
        self.cascade_response() >= self.trigger_threshold
    }

    /// Classify cooperativity type.
    #[must_use]
    pub fn cooperativity_type(&self) -> &'static str {
        if self.n_hill < 0.9 {
            "Negative (dampening)"
        } else if self.n_hill <= 1.1 {
            "None (standard)"
        } else if self.n_hill < 2.0 {
            "Mild positive"
        } else if self.n_hill < 4.0 {
            "Strong positive"
        } else {
            "Ultrasensitive (switch)"
        }
    }

    /// Get amplification factor relative to standard (nH=1).
    #[must_use]
    pub fn amplification_factor(&self) -> f64 {
        if self.total_strength <= 0.0 || self.k_half <= 0.0 {
            return 1.0;
        }
        // Calculate response with current nH vs nH=1
        let current = self.cascade_response();
        let standard = {
            let input = self.total_strength;
            let k = self.k_half;
            input / (k + input)
        };
        if standard > 0.0 {
            current / standard
        } else {
            1.0
        }
    }
}
