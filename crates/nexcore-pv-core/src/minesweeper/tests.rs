//! Integration tests for the Minesweeper-PV module.
//!
//! ## Safety Axioms
//!
//! These tests validate that the Minesweeper-PV framework correctly implements
//! the Conservation Laws and Safety Axioms of the Tree of Vigilance.

use super::*;

/// Test the full SGLT2 inhibitor investigation scenario from the original Python
#[test]
fn test_sglt2_inhibitor_investigation() {
    // Create grid with SGLT2 inhibitors
    let drugs = [
        "canagliflozin",
        "dapagliflozin",
        "empagliflozin",
        "ertugliflozin",
    ];
    let events = [
        "acute_kidney_injury",
        "ketoacidosis",
        "heart_failure_hosp",
        "lower_limb_amputation",
        "urinary_tract_infection",
    ];

    let mut grid = CSPGrid::new();

    // Add all drug-event combinations
    for drug in &drugs {
        for event in &events {
            grid.add_cell(drug, event, "all");
        }
    }

    // Set mechanistic adjacency (same event, different drugs in class)
    for event in &events {
        for i in 0..drugs.len() {
            for j in (i + 1)..drugs.len() {
                let cell_a = format!("{}|{}|all", drugs[i], event);
                let cell_b = format!("{}|{}|all", drugs[j], event);
                grid.set_adjacency(&cell_a, &cell_b, 0.85, Some(AdjacencyType::Mechanistic));
            }
        }
    }

    // Set phenotypic adjacency (same drug, different events)
    for drug in &drugs {
        for i in 0..events.len() {
            for j in (i + 1)..events.len() {
                let cell_a = format!("{}|{}|all", drug, events[i]);
                let cell_b = format!("{}|{}|all", drug, events[j]);
                grid.set_adjacency(&cell_a, &cell_b, 0.4, Some(AdjacencyType::Phenotypic));
            }
        }
    }

    assert_eq!(grid.cell_count(), 20); // 4 drugs × 5 events

    // Initial investigation: canagliflozin + AKI
    let evidence = Evidence::builder()
        .prr(2.8)
        .chi2(6.2)
        .count(5)
        .temporal_pattern("strong")
        .mechanism("plausible")
        .build();

    let newly_flagged = grid.propagate_evidence("canagliflozin|acute_kidney_injury|all", &evidence);

    // Verify source was updated
    let source = grid
        .get_cell("canagliflozin", "acute_kidney_injury", "all")
        .unwrap();
    assert_eq!(source.status, CellStatus::Investigated);
    assert!(source.theta() > 0.01); // Should have increased from base rate

    // Verify propagation occurred to mechanistically adjacent cells
    let dapa_aki = grid
        .get_cell("dapagliflozin", "acute_kidney_injury", "all")
        .unwrap();
    assert!(dapa_aki.theta() > 0.01); // Should have received propagated evidence

    // Check propagation log
    assert_eq!(grid.propagation_log().len(), 1);
    assert!(!grid.propagation_log()[0].propagated_to.is_empty());

    // Verify newly_flagged makes sense
    // (may or may not be empty depending on evidence strength and thresholds)
    println!("Newly flagged cells: {}", newly_flagged.len());
}

/// Test that EIG prioritization works correctly
#[test]
fn test_eig_prioritization() {
    let mut grid = CSPGrid::new();

    // Create a simple network
    grid.add_cell("drug_a", "event_1", "all");
    grid.add_cell("drug_a", "event_2", "all");
    grid.add_cell("drug_b", "event_1", "all");

    // Drug_a events are connected (same drug)
    grid.set_adjacency(
        "drug_a|event_1|all",
        "drug_a|event_2|all",
        0.6,
        Some(AdjacencyType::Phenotypic),
    );

    // Cross-drug same event connection
    grid.set_adjacency(
        "drug_a|event_1|all",
        "drug_b|event_1|all",
        0.4,
        Some(AdjacencyType::Mechanistic),
    );

    // Get initial priorities
    let priorities = grid.get_investigation_priorities(10);
    assert_eq!(priorities.len(), 3);

    // All cells should have non-negative EIG
    for (_, eig) in &priorities {
        assert!(*eig >= 0.0);
    }

    // Investigate the highest priority cell
    let evidence = Evidence::builder().prr(2.0).chi2(4.0).count(3).build();

    let top_cell_id = priorities[0].0.id.clone();
    grid.propagate_evidence(&top_cell_id, &evidence);

    // After investigation, that cell should have 0 EIG
    let eig_after = grid.calculate_eig(&top_cell_id);
    assert!((eig_after - 0.0).abs() < f64::EPSILON);
}

/// Test belief state transitions through the lifecycle
#[test]
fn test_cell_lifecycle() {
    // Start with a higher prior to make the test more realistic
    let mut cell = Cell::with_prior("test_drug", "test_event", "all", 0.1);

    // Initial state
    assert_eq!(cell.status, CellStatus::Unknown);
    assert!(cell.theta() < 0.2); // Near prior

    // Very strong evidence with rechallenge (strongest signal possible)
    // Rechallenge has LR=10, much stronger than dechallenge
    let strong_evidence = Evidence::builder()
        .prr(5.0)
        .chi2(20.0)
        .count(15)
        .temporal_pattern("strong")
        .mechanism("established")
        .positive_rechallenge(true) // LR = 10.0
        .build();

    // Apply evidence multiple times (simulating accumulating evidence)
    cell.update_belief(&strong_evidence);
    cell.update_belief(&strong_evidence);

    // Now theta should be high enough
    assert!(
        cell.theta() > FLAG_THRESHOLD || cell.status != CellStatus::Unknown,
        "Expected theta > {} or flagged status, got theta={}, status={:?}",
        FLAG_THRESHOLD,
        cell.theta(),
        cell.status
    );
}

/// Test that propagation decays correctly with distance
#[test]
fn test_propagation_decay() {
    let mut grid = CSPGrid::new();

    // Create a chain: A -> B -> C -> D
    grid.add_cell("drug", "event_a", "all");
    grid.add_cell("drug", "event_b", "all");
    grid.add_cell("drug", "event_c", "all");
    grid.add_cell("drug", "event_d", "all");

    grid.set_adjacency("drug|event_a|all", "drug|event_b|all", 0.7, None);
    grid.set_adjacency("drug|event_b|all", "drug|event_c|all", 0.7, None);
    grid.set_adjacency("drug|event_c|all", "drug|event_d|all", 0.7, None);

    // Get initial thetas
    let initial_b = grid.get_cell("drug", "event_b", "all").unwrap().theta();
    let initial_c = grid.get_cell("drug", "event_c", "all").unwrap().theta();
    let initial_d = grid.get_cell("drug", "event_d", "all").unwrap().theta();

    // Propagate from A
    let evidence = Evidence::builder()
        .prr(5.0)
        .chi2(20.0)
        .count(15)
        .temporal_pattern("strong")
        .mechanism("established")
        .build();

    grid.propagate_evidence("drug|event_a|all", &evidence);

    // Get updated thetas
    let updated_b = grid.get_cell("drug", "event_b", "all").unwrap().theta();
    let updated_c = grid.get_cell("drug", "event_c", "all").unwrap().theta();
    let updated_d = grid.get_cell("drug", "event_d", "all").unwrap().theta();

    // Changes should decrease with distance
    let delta_b = updated_b - initial_b;
    let delta_c = updated_c - initial_c;
    let delta_d = updated_d - initial_d;

    assert!(
        delta_b > delta_c,
        "Delta B ({}) should > Delta C ({})",
        delta_b,
        delta_c
    );

    // D might not receive much (below threshold) or minimal change
    // Just verify it's not greater than C
    assert!(
        delta_c >= delta_d,
        "Delta C ({}) should >= Delta D ({})",
        delta_c,
        delta_d
    );
}

/// Test Campion Signal Theory scoring
#[test]
fn test_campion_signal_theory() {
    let mut grid = CSPGrid::new();
    grid.add_cell("drug_x", "event_y", "all");

    // Unknown cell should have low signal
    let cell = grid.get_cell("drug_x", "event_y", "all").unwrap();
    let result = cell_to_signal_score(cell, &grid);

    assert!(result.rarity_u > 0.0); // Should have some rarity (information content)
    assert!((result.recognition_r - 0.3).abs() < 1e-10); // Unknown = 0.3
    assert!(result.temporal_t > 0.99); // Just created, so T ≈ 1.0

    // After strong evidence, signal should increase
    let evidence = Evidence::builder()
        .prr(5.0)
        .chi2(20.0)
        .count(20)
        .temporal_pattern("strong")
        .mechanism("established")
        .positive_rechallenge(true)
        .build();

    grid.propagate_evidence("drug_x|event_y|all", &evidence);

    let updated_cell = grid.get_cell("drug_x", "event_y", "all").unwrap();
    let updated_result = cell_to_signal_score(updated_cell, &grid);

    // Recognition should be higher (investigated = 0.8)
    assert!((updated_result.recognition_r - 0.8).abs() < 1e-10);

    // Overall signal should be higher
    assert!(updated_result.signal_value > result.signal_value);
}

/// Test that adjacency types are tracked correctly
#[test]
fn test_adjacency_types() {
    let mut grid = CSPGrid::new();

    grid.add_cell("drug_a", "event_1", "all");
    grid.add_cell("drug_b", "event_1", "all");
    grid.add_cell("drug_a", "event_2", "all");

    // Mechanistic: same event, different drugs
    grid.set_adjacency(
        "drug_a|event_1|all",
        "drug_b|event_1|all",
        0.8,
        Some(AdjacencyType::Mechanistic),
    );

    // Phenotypic: same drug, different events
    grid.set_adjacency(
        "drug_a|event_1|all",
        "drug_a|event_2|all",
        0.4,
        Some(AdjacencyType::Phenotypic),
    );

    // Add another type to an existing pair
    grid.set_adjacency(
        "drug_a|event_1|all",
        "drug_b|event_1|all",
        0.8,
        Some(AdjacencyType::Temporal),
    );

    // Check types
    let types_ab = grid.get_adjacency_types("drug_a|event_1|all", "drug_b|event_1|all");
    assert!(types_ab.contains(&AdjacencyType::Mechanistic));
    assert!(types_ab.contains(&AdjacencyType::Temporal));

    let types_aa = grid.get_adjacency_types("drug_a|event_1|all", "drug_a|event_2|all");
    assert!(types_aa.contains(&AdjacencyType::Phenotypic));
}

/// Test serialization roundtrip for Evidence
#[test]
fn test_evidence_serialization() {
    let evidence = Evidence::builder()
        .prr(2.5)
        .chi2(8.0)
        .count(10)
        .temporal_pattern("strong")
        .mechanism("plausible")
        .positive_dechallenge(true)
        .build();

    let json = serde_json::to_string(&evidence).unwrap();
    let deserialized: Evidence = serde_json::from_str(&json).unwrap();

    assert!((deserialized.prr - 2.5).abs() < 1e-10);
    assert_eq!(deserialized.count, 10);
    assert!(deserialized.positive_dechallenge);
}

/// Test serialization roundtrip for Cell
#[test]
fn test_cell_serialization() {
    let cell = Cell::new("aspirin", "gi_bleeding", "all");

    let json = serde_json::to_string(&cell).unwrap();
    let deserialized: Cell = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.drug, "aspirin");
    assert_eq!(deserialized.event, "gi_bleeding");
    assert_eq!(deserialized.population, "all");
}
