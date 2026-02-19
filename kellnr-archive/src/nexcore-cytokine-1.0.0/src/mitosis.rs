//! Mitosis - controlled replication with population limits.
//!
//! ## T1 Grounding
//!
//! - `SpawnRequest` → ρ (recursion) - self-replication
//! - `Lineage` → σ (sequence) - ordered ancestry chain
//! - `PopulationControl` → κ (comparison) - count vs limits
//!
//! ## Biological Analog
//!
//! Cell division with controlled proliferation:
//! 1. **G1 checkpoint**: Conditions favorable? (growth signals present)
//! 2. **S phase**: DNA replication (copy state)
//! 3. **G2 checkpoint**: Replication complete and error-free?
//! 4. **M phase**: Division (parent → 2 daughters with shared lineage)
//! 5. **Contact inhibition**: Stop dividing when population is dense
//!
//! ## Claude Code Analog
//!
//! Subagent spawning with ancestry tracking and population limits:
//! - **Growth signal**: IL-2/CSF triggers spawn
//! - **State copying**: Parent configuration inherited by child
//! - **Population cap**: Maximum concurrent agents enforced
//! - **Lineage tracking**: Parent→child chain for accountability

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a cell/agent in the population.
///
/// # Tier: T2-P
/// Grounds to: λ (location — identity in population space)
pub type CellId = String;

/// A record in the lineage (ancestry chain).
///
/// # Tier: T2-P
/// Grounds to: σ (sequence — ordered ancestry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageRecord {
    /// This cell's ID
    pub cell_id: CellId,
    /// Parent cell ID (None for root/original)
    pub parent_id: Option<CellId>,
    /// Generation number (0 for root)
    pub generation: u32,
    /// When this cell was created
    pub born_at: DateTime<Utc>,
}

/// Result of a spawn attempt.
///
/// # Tier: T2-P
/// Grounds to: ∃ (existence — did new cell come into being?)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnResult {
    /// Successfully spawned
    Spawned,
    /// Population at capacity — contact inhibition
    PopulationFull,
    /// Maximum generation depth reached
    MaxGenerationReached,
    /// Growth signals insufficient
    InsufficientSignal,
}

/// Population controller that manages cell division.
///
/// Enforces population limits, tracks lineage, and controls replication.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ρ (recursion — spawn chains) + κ (comparison — population vs limit)
#[derive(Debug)]
pub struct PopulationController {
    /// Maximum population size (contact inhibition)
    max_population: usize,
    /// Maximum generation depth
    max_generation: u32,
    /// All living cells and their lineage
    population: Vec<LineageRecord>,
    /// Counter for generating unique IDs
    next_id: u64,
    /// Lifetime spawn count
    total_spawned: u64,
    /// Lifetime death count
    total_died: u64,
}

impl PopulationController {
    /// Create a new population controller.
    pub fn new(max_population: usize) -> Self {
        Self {
            max_population,
            max_generation: 10,
            population: Vec::new(),
            next_id: 0,
            total_spawned: 0,
            total_died: 0,
        }
    }

    /// Set maximum generation depth.
    pub fn with_max_generation(mut self, max: u32) -> Self {
        self.max_generation = max;
        self
    }

    /// Seed the population with an initial cell (generation 0).
    pub fn seed(&mut self, id: impl Into<String>) -> CellId {
        let cell_id: CellId = id.into();
        self.population.push(LineageRecord {
            cell_id: cell_id.clone(),
            parent_id: None,
            generation: 0,
            born_at: Utc::now(),
        });
        self.total_spawned += 1;
        cell_id
    }

    /// Attempt to spawn a new cell from a parent.
    pub fn spawn(&mut self, parent_id: &str) -> (SpawnResult, Option<CellId>) {
        // Check population limit
        if self.population.len() >= self.max_population {
            return (SpawnResult::PopulationFull, None);
        }

        // Find parent and get generation
        let parent_gen = match self.population.iter().find(|c| c.cell_id == parent_id) {
            Some(parent) => parent.generation,
            None => return (SpawnResult::InsufficientSignal, None),
        };

        // Check generation limit
        if parent_gen >= self.max_generation {
            return (SpawnResult::MaxGenerationReached, None);
        }

        // Generate new cell ID
        self.next_id += 1;
        let child_id = format!("cell_{}", self.next_id);

        self.population.push(LineageRecord {
            cell_id: child_id.clone(),
            parent_id: Some(parent_id.to_string()),
            generation: parent_gen + 1,
            born_at: Utc::now(),
        });

        self.total_spawned += 1;
        (SpawnResult::Spawned, Some(child_id))
    }

    /// Remove a cell from the population (apoptosis/death).
    pub fn remove(&mut self, cell_id: &str) -> bool {
        let before = self.population.len();
        self.population.retain(|c| c.cell_id != cell_id);
        let removed = self.population.len() < before;
        if removed {
            self.total_died += 1;
        }
        removed
    }

    /// Get current population size.
    pub fn size(&self) -> usize {
        self.population.len()
    }

    /// Get remaining capacity.
    pub fn remaining_capacity(&self) -> usize {
        self.max_population.saturating_sub(self.population.len())
    }

    /// Check if population is at capacity.
    pub fn is_full(&self) -> bool {
        self.population.len() >= self.max_population
    }

    /// Get the lineage chain for a cell (ancestors back to root).
    pub fn lineage(&self, cell_id: &str) -> Vec<&LineageRecord> {
        let mut chain = Vec::new();
        let mut current = cell_id;

        loop {
            match self.population.iter().find(|c| c.cell_id == current) {
                Some(record) => {
                    chain.push(record);
                    match &record.parent_id {
                        Some(parent) => current = parent,
                        None => break, // Root cell
                    }
                }
                None => break, // Parent no longer alive
            }
        }

        chain
    }

    /// Get all children of a cell.
    pub fn children(&self, cell_id: &str) -> Vec<&LineageRecord> {
        self.population
            .iter()
            .filter(|c| c.parent_id.as_deref() == Some(cell_id))
            .collect()
    }

    /// Get maximum generation in the current population.
    pub fn max_active_generation(&self) -> u32 {
        self.population
            .iter()
            .map(|c| c.generation)
            .max()
            .unwrap_or(0)
    }

    /// Get population statistics.
    pub fn stats(&self) -> PopulationStats {
        let mut by_generation = std::collections::HashMap::new();
        for cell in &self.population {
            *by_generation.entry(cell.generation).or_insert(0usize) += 1;
        }

        PopulationStats {
            current_size: self.population.len(),
            max_population: self.max_population,
            utilization: if self.max_population > 0 {
                self.population.len() as f64 / self.max_population as f64
            } else {
                1.0
            },
            max_active_generation: self.max_active_generation(),
            total_spawned: self.total_spawned,
            total_died: self.total_died,
            by_generation,
        }
    }
}

/// Statistics for the population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationStats {
    /// Current living population
    pub current_size: usize,
    /// Maximum allowed
    pub max_population: usize,
    /// Utilization fraction
    pub utilization: f64,
    /// Deepest generation alive
    pub max_active_generation: u32,
    /// Lifetime spawns
    pub total_spawned: u64,
    /// Lifetime deaths
    pub total_died: u64,
    /// Count per generation
    pub by_generation: std::collections::HashMap<u32, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_population_seed() {
        let mut pop = PopulationController::new(10);
        let root = pop.seed("root");
        assert_eq!(root, "root");
        assert_eq!(pop.size(), 1);
    }

    #[test]
    fn test_spawn_child() {
        let mut pop = PopulationController::new(10);
        pop.seed("root");

        let (result, child_id) = pop.spawn("root");
        assert_eq!(result, SpawnResult::Spawned);
        assert!(child_id.is_some());
        assert_eq!(pop.size(), 2);
    }

    #[test]
    fn test_spawn_at_capacity() {
        let mut pop = PopulationController::new(2);
        pop.seed("root");
        pop.spawn("root");

        let (result, _) = pop.spawn("root");
        assert_eq!(result, SpawnResult::PopulationFull);
        assert_eq!(pop.size(), 2);
    }

    #[test]
    fn test_spawn_max_generation() {
        let mut pop = PopulationController::new(100).with_max_generation(2);
        pop.seed("gen0");

        let (_, gen1) = pop.spawn("gen0");
        let gen1_id = gen1.unwrap_or_default();

        let (_, gen2) = pop.spawn(&gen1_id);
        let gen2_id = gen2.unwrap_or_default();

        // Gen 2 trying to spawn gen 3 — exceeds max
        let (result, _) = pop.spawn(&gen2_id);
        assert_eq!(result, SpawnResult::MaxGenerationReached);
    }

    #[test]
    fn test_lineage_tracking() {
        let mut pop = PopulationController::new(10);
        pop.seed("root");
        let (_, child) = pop.spawn("root");
        let child_id = child.unwrap_or_default();
        let (_, grandchild) = pop.spawn(&child_id);
        let gc_id = grandchild.unwrap_or_default();

        let lineage = pop.lineage(&gc_id);
        assert_eq!(lineage.len(), 3); // gc → child → root
        assert_eq!(lineage[0].generation, 2);
        assert_eq!(lineage[2].generation, 0);
    }

    #[test]
    fn test_children() {
        let mut pop = PopulationController::new(10);
        pop.seed("parent");
        pop.spawn("parent");
        pop.spawn("parent");
        pop.spawn("parent");

        let children = pop.children("parent");
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_remove_cell() {
        let mut pop = PopulationController::new(10);
        pop.seed("root");
        let (_, child) = pop.spawn("root");

        assert!(pop.remove(&child.unwrap_or_default()));
        assert_eq!(pop.size(), 1);
    }

    #[test]
    fn test_population_stats() {
        let mut pop = PopulationController::new(20);
        pop.seed("root");
        pop.spawn("root");
        pop.spawn("root");

        let stats = pop.stats();
        assert_eq!(stats.current_size, 3);
        assert_eq!(stats.total_spawned, 3);
        assert_eq!(stats.max_active_generation, 1);
    }

    #[test]
    fn test_spawn_nonexistent_parent() {
        let mut pop = PopulationController::new(10);
        let (result, _) = pop.spawn("ghost");
        assert_eq!(result, SpawnResult::InsufficientSignal);
    }
}
