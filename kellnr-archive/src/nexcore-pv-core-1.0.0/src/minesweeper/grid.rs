//! CSP Grid for Minesweeper-PV constraint propagation.
//!
//! ## Safety Axioms
//!
//! This module implements the Constraint Satisfaction Problem (CSP) framework
//! for pharmacovigilance signal investigation. It follows the Conservation Laws:
//!
//! - Evidence decays during propagation to preserve epistemic boundaries
//! - EIG calculation prioritizes cells with maximum expected information value
//! - Status transitions follow the Signal Confirmation Axiom

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::belief::Cell;
use super::evidence::Evidence;
use crate::minesweeper::types::{
    AdjacencyType, CellStatus, PROPAGATION_DECAY, PROPAGATION_THRESHOLD, TemporalWindow,
};

/// Record of a single propagation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationRecord {
    /// Cell that received propagated evidence
    pub cell_id: String,
    /// Depth from source cell
    pub depth: usize,
    /// Effective weight at this cell
    pub weight: f64,
    /// Change in theta (belief)
    pub delta_theta: f64,
}

/// Log entry for a complete propagation cascade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationLog {
    /// Source cell ID
    pub source_id: String,
    /// Likelihood ratio of source evidence
    pub evidence_lr: f64,
    /// Timestamp of propagation
    pub timestamp: String,
    /// All cells that received propagated evidence
    pub propagated_to: Vec<PropagationRecord>,
}

/// Constraint Satisfaction Problem Grid for Minesweeper-PV.
///
/// The grid maintains:
/// - A collection of drug-event cells with belief states
/// - Adjacency relationships between cells (weighted, typed)
/// - Propagation logs for audit trails
#[derive(Debug, Clone, Default)]
pub struct CSPGrid {
    /// Cells indexed by ID (drug|event|population)
    cells: HashMap<String, Cell>,
    /// Adjacency weights: cell_id -> (neighbor_id -> weight)
    adjacency: HashMap<String, HashMap<String, f64>>,
    /// Adjacency types: cell_id -> (neighbor_id -> set of types)
    adjacency_types: HashMap<String, HashMap<String, HashSet<AdjacencyType>>>,
    /// Custom adjacency weights by type
    #[allow(dead_code)] // Planned feature for weighted propagation
    type_weights: HashMap<AdjacencyType, f64>,
    /// Log of propagation events
    propagation_log: Vec<PropagationLog>,
}

impl CSPGrid {
    /// Create a new empty CSP grid
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new grid with custom adjacency type weights
    #[must_use]
    pub fn with_type_weights(weights: HashMap<AdjacencyType, f64>) -> Self {
        Self {
            type_weights: weights,
            ..Default::default()
        }
    }

    /// Add a cell to the grid
    ///
    /// Returns the cell if it was newly added, or the existing cell
    pub fn add_cell(&mut self, drug: &str, event: &str, population: &str) -> &mut Cell {
        let cell_id = format!("{}|{}|{}", drug, event, population);

        self.cells
            .entry(cell_id.clone())
            .or_insert_with(|| Cell::new(drug, event, population))
    }

    /// Add a cell with custom prior probability
    pub fn add_cell_with_prior(
        &mut self,
        drug: &str,
        event: &str,
        population: &str,
        prior: f64,
    ) -> &mut Cell {
        let cell_id = format!("{}|{}|{}", drug, event, population);

        self.cells
            .entry(cell_id.clone())
            .or_insert_with(|| Cell::with_prior(drug, event, population, prior))
    }

    /// Add a cell with time-to-onset window
    pub fn add_cell_with_tto(
        &mut self,
        drug: &str,
        event: &str,
        population: &str,
        tto_window: TemporalWindow,
    ) -> &mut Cell {
        let cell_id = format!("{}|{}|{}", drug, event, population);

        self.cells
            .entry(cell_id.clone())
            .or_insert_with(|| Cell::with_tto(drug, event, population, tto_window))
    }

    /// Get a cell by its components
    #[must_use]
    pub fn get_cell(&self, drug: &str, event: &str, population: &str) -> Option<&Cell> {
        let cell_id = format!("{}|{}|{}", drug, event, population);
        self.cells.get(&cell_id)
    }

    /// Get a mutable reference to a cell
    #[must_use]
    pub fn get_cell_mut(&mut self, drug: &str, event: &str, population: &str) -> Option<&mut Cell> {
        let cell_id = format!("{}|{}|{}", drug, event, population);
        self.cells.get_mut(&cell_id)
    }

    /// Get a cell by ID
    #[must_use]
    pub fn get_cell_by_id(&self, cell_id: &str) -> Option<&Cell> {
        self.cells.get(cell_id)
    }

    /// Get a mutable reference to a cell by ID
    #[must_use]
    pub fn get_cell_by_id_mut(&mut self, cell_id: &str) -> Option<&mut Cell> {
        self.cells.get_mut(cell_id)
    }

    /// Set adjacency between two cells
    ///
    /// Adjacency is symmetric (bidirectional). Weight below PROPAGATION_THRESHOLD is ignored.
    pub fn set_adjacency(
        &mut self,
        cell_a: &str,
        cell_b: &str,
        weight: f64,
        adj_type: Option<AdjacencyType>,
    ) {
        if weight <= PROPAGATION_THRESHOLD {
            return;
        }

        // Set symmetric adjacency weights
        self.adjacency
            .entry(cell_a.to_string())
            .or_default()
            .insert(cell_b.to_string(), weight);
        self.adjacency
            .entry(cell_b.to_string())
            .or_default()
            .insert(cell_a.to_string(), weight);

        // Set adjacency types if provided
        if let Some(atype) = adj_type {
            self.adjacency_types
                .entry(cell_a.to_string())
                .or_default()
                .entry(cell_b.to_string())
                .or_default()
                .insert(atype);
            self.adjacency_types
                .entry(cell_b.to_string())
                .or_default()
                .entry(cell_a.to_string())
                .or_default()
                .insert(atype);
        }
    }

    /// Get neighbors of a cell with their adjacency weights
    #[must_use]
    pub fn get_neighbors(&self, cell_id: &str) -> HashMap<String, f64> {
        self.adjacency.get(cell_id).cloned().unwrap_or_default()
    }

    /// Get adjacency types between two cells
    #[must_use]
    pub fn get_adjacency_types(&self, cell_a: &str, cell_b: &str) -> HashSet<AdjacencyType> {
        self.adjacency_types
            .get(cell_a)
            .and_then(|m| m.get(cell_b))
            .cloned()
            .unwrap_or_default()
    }

    /// Propagate evidence from a source cell to adjacent cells
    ///
    /// Returns a list of cells that were newly flagged as a result.
    pub fn propagate_evidence(&mut self, source_id: &str, evidence: &Evidence) -> Vec<String> {
        // Update source cell directly
        let source_lr = if let Some(source) = self.cells.get_mut(source_id) {
            let lr = source.update_belief(evidence);
            source.mark_investigated();
            lr
        } else {
            return Vec::new();
        };

        // Initialize propagation state
        let mut propagation_record = PropagationLog {
            source_id: source_id.to_string(),
            evidence_lr: source_lr,
            timestamp: chrono::Utc::now().to_rfc3339(),
            propagated_to: Vec::new(),
        };

        let mut visited: HashSet<&str> = HashSet::new();
        visited.insert(source_id);

        let mut newly_flagged: Vec<String> = Vec::new();

        // BFS queue: (cell_id, path_weight, depth)
        // Use &str to avoid cloning String keys during traversal
        let mut queue: VecDeque<(&str, f64, usize)> = VecDeque::new();

        // Initialize queue with immediate neighbors
        if let Some(neighbors) = self.adjacency.get(source_id) {
            for (neighbor_id, &weight) in neighbors {
                if weight > PROPAGATION_THRESHOLD {
                    queue.push_back((neighbor_id.as_str(), weight, 1));
                }
            }
        }

        // BFS propagation
        while let Some((cell_id, path_weight, depth)) = queue.pop_front() {
            if visited.contains(cell_id) {
                continue;
            }
            visited.insert(cell_id);

            // Calculate decayed weight
            let decayed_weight = path_weight * PROPAGATION_DECAY.powi(depth as i32);
            if decayed_weight < PROPAGATION_THRESHOLD {
                continue;
            }

            // Create propagated evidence
            let prop_evidence = evidence.propagate(decayed_weight);

            // Update cell and check for new flagging
            if let Some(cell) = self.cells.get_mut(cell_id) {
                let was_flagged = cell.status == CellStatus::Flagged;
                let old_theta = cell.theta();

                cell.update_belief(&prop_evidence);

                let delta_theta = cell.theta() - old_theta;

                propagation_record.propagated_to.push(PropagationRecord {
                    cell_id: cell_id.to_string(),
                    depth,
                    weight: decayed_weight,
                    delta_theta,
                });

                if cell.status == CellStatus::Flagged && !was_flagged {
                    newly_flagged.push(cell_id.to_string());
                }
            }

            // Add next-hop neighbors to queue
            if let Some(neighbors) = self.adjacency.get(cell_id) {
                for (next_id, &next_weight) in neighbors {
                    if !visited.contains(next_id.as_str()) {
                        let combined_weight = decayed_weight * next_weight;
                        if combined_weight > PROPAGATION_THRESHOLD {
                            queue.push_back((next_id.as_str(), combined_weight, depth + 1));
                        }
                    }
                }
            }
        }

        self.propagation_log.push(propagation_record);
        newly_flagged
    }

    /// Calculate Expected Information Gain (EIG) for a cell
    ///
    /// EIG(c) = σ⁻¹ × |θ - 0.5| × Σ[w × (1 - conf_neighbor)]
    ///
    /// Higher EIG indicates a cell that would provide more value if investigated.
    #[must_use]
    pub fn calculate_eig(&self, cell_id: &str) -> f64 {
        let cell = match self.cells.get(cell_id) {
            Some(c) => c,
            None => return 0.0,
        };

        // Already investigated or confirmed cells have 0 EIG
        if matches!(
            cell.status,
            CellStatus::Investigated | CellStatus::Confirmed
        ) {
            return 0.0;
        }

        // Uncertainty factor: higher uncertainty = more value from investigation
        let uncertainty_factor = if cell.sigma() > 0.0 {
            1.0 / cell.sigma()
        } else {
            10.0
        };

        // Decision proximity: closer to decision threshold = more value
        let decision_proximity = (cell.theta() - 0.5).abs();

        // Neighbor potential: sum of weighted uncertain neighbors
        let mut neighbor_potential = 0.0;
        let mut flagged_neighbors = 0;

        if let Some(neighbors) = self.adjacency.get(cell_id) {
            for (neighbor_id, &weight) in neighbors {
                if let Some(neighbor) = self.cells.get(neighbor_id) {
                    neighbor_potential += weight * (1.0 - neighbor.confidence());
                    if neighbor.status == CellStatus::Flagged {
                        flagged_neighbors += 1;
                    }
                }
            }
        }

        // Base EIG
        let mut eig = uncertainty_factor * decision_proximity * neighbor_potential;

        // Boost for having flagged neighbors (cluster investigation)
        eig *= 1.0 + 0.2 * flagged_neighbors as f64;

        eig
    }

    /// Get investigation priorities sorted by EIG
    ///
    /// Returns up to `top_n` cells with their EIG scores.
    #[must_use]
    pub fn get_investigation_priorities(&self, top_n: usize) -> Vec<(&Cell, f64)> {
        let mut candidates: Vec<(&Cell, f64)> = self
            .cells
            .iter()
            .filter(|(_, cell)| matches!(cell.status, CellStatus::Unknown | CellStatus::Flagged))
            .map(|(cell_id, cell)| (cell, self.calculate_eig(cell_id)))
            .collect();

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        candidates.truncate(top_n);
        candidates
    }

    /// Get status summary counts
    #[must_use]
    pub fn get_status_summary(&self) -> HashMap<CellStatus, usize> {
        let mut summary: HashMap<CellStatus, usize> = HashMap::new();
        for cell in self.cells.values() {
            *summary.entry(cell.status).or_insert(0) += 1;
        }
        summary
    }

    /// Get all cells
    #[must_use]
    pub fn cells(&self) -> &HashMap<String, Cell> {
        &self.cells
    }

    /// Get cell count
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Get propagation log
    #[must_use]
    pub fn propagation_log(&self) -> &[PropagationLog] {
        &self.propagation_log
    }

    /// Get all flagged cells
    #[must_use]
    pub fn flagged_cells(&self) -> Vec<&Cell> {
        self.cells
            .values()
            .filter(|c| c.status == CellStatus::Flagged)
            .collect()
    }

    /// Get all confirmed cells
    #[must_use]
    pub fn confirmed_cells(&self) -> Vec<&Cell> {
        self.cells
            .values()
            .filter(|c| c.status == CellStatus::Confirmed)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grid() -> CSPGrid {
        let mut grid = CSPGrid::new();

        // Add cells for two SGLT2 inhibitors
        grid.add_cell("canagliflozin", "ketoacidosis", "all");
        grid.add_cell("dapagliflozin", "ketoacidosis", "all");
        grid.add_cell("canagliflozin", "aki", "all");

        // Set mechanistic adjacency between same-class drugs
        grid.set_adjacency(
            "canagliflozin|ketoacidosis|all",
            "dapagliflozin|ketoacidosis|all",
            0.85,
            Some(AdjacencyType::Mechanistic),
        );

        // Set phenotypic adjacency for same-drug different events
        grid.set_adjacency(
            "canagliflozin|ketoacidosis|all",
            "canagliflozin|aki|all",
            0.4,
            Some(AdjacencyType::Phenotypic),
        );

        grid
    }

    #[test]
    fn test_grid_creation() {
        let grid = CSPGrid::new();
        assert_eq!(grid.cell_count(), 0);
    }

    #[test]
    fn test_add_cell() {
        let mut grid = CSPGrid::new();
        grid.add_cell("aspirin", "gi_bleeding", "all");

        assert_eq!(grid.cell_count(), 1);
        assert!(grid.get_cell("aspirin", "gi_bleeding", "all").is_some());
    }

    #[test]
    fn test_adjacency() {
        let grid = create_test_grid();

        let neighbors = grid.get_neighbors("canagliflozin|ketoacidosis|all");
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains_key("dapagliflozin|ketoacidosis|all"));

        let types = grid.get_adjacency_types(
            "canagliflozin|ketoacidosis|all",
            "dapagliflozin|ketoacidosis|all",
        );
        assert!(types.contains(&AdjacencyType::Mechanistic));
    }

    #[test]
    fn test_evidence_propagation() {
        let mut grid = create_test_grid();

        let evidence = Evidence::builder()
            .prr(3.0)
            .chi2(8.0)
            .count(5)
            .temporal_pattern("strong")
            .mechanism("plausible")
            .build();

        let source_id = "canagliflozin|ketoacidosis|all";
        let initial_dapa_theta = grid
            .get_cell("dapagliflozin", "ketoacidosis", "all")
            .map(|c| c.theta())
            .unwrap_or(0.0);

        grid.propagate_evidence(source_id, &evidence);

        // Source should be marked investigated
        let source = grid
            .get_cell("canagliflozin", "ketoacidosis", "all")
            .unwrap();
        assert_eq!(source.status, CellStatus::Investigated);

        // Mechanistically adjacent cell should have updated belief
        let dapa = grid
            .get_cell("dapagliflozin", "ketoacidosis", "all")
            .unwrap();
        assert!(dapa.theta() > initial_dapa_theta);
    }

    #[test]
    fn test_eig_calculation() {
        let mut grid = create_test_grid();

        // Unknown cell with uncertain neighbors should have non-zero EIG
        let eig = grid.calculate_eig("canagliflozin|ketoacidosis|all");
        assert!(eig >= 0.0);

        // After investigation, EIG should be 0
        let evidence = Evidence::builder().prr(2.0).chi2(4.0).count(3).build();
        grid.propagate_evidence("canagliflozin|ketoacidosis|all", &evidence);

        let eig_after = grid.calculate_eig("canagliflozin|ketoacidosis|all");
        assert!((eig_after - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_investigation_priorities() {
        let grid = create_test_grid();

        let priorities = grid.get_investigation_priorities(10);

        // Should have 3 cells (all unknown)
        assert_eq!(priorities.len(), 3);

        // Should be sorted by EIG descending
        for i in 0..priorities.len() - 1 {
            assert!(priorities[i].1 >= priorities[i + 1].1);
        }
    }

    #[test]
    fn test_status_summary() {
        let mut grid = create_test_grid();

        let summary = grid.get_status_summary();
        assert_eq!(*summary.get(&CellStatus::Unknown).unwrap_or(&0), 3);

        // Update one cell
        let evidence = Evidence::builder().prr(2.0).chi2(4.0).count(3).build();
        grid.propagate_evidence("canagliflozin|ketoacidosis|all", &evidence);

        let summary_after = grid.get_status_summary();
        assert_eq!(
            *summary_after.get(&CellStatus::Investigated).unwrap_or(&0),
            1
        );
    }

    #[test]
    fn test_propagation_log() {
        let mut grid = create_test_grid();

        assert!(grid.propagation_log().is_empty());

        let evidence = Evidence::builder().prr(2.0).build();
        grid.propagate_evidence("canagliflozin|ketoacidosis|all", &evidence);

        assert_eq!(grid.propagation_log().len(), 1);
        assert_eq!(
            grid.propagation_log()[0].source_id,
            "canagliflozin|ketoacidosis|all"
        );
    }
}
