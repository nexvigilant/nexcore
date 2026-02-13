use crate::minesweeper::sentinel::*;

fn test_class_cascade_and_silent_risk() {
    let mut grid = SentinelGrid::new();
    let drugs = ["Drug_A", "Drug_B"];
    let event = "Rare_Cardio";
    for d in drugs {
        grid.add_cell(d, event, "all");
    }
    grid.set_adjacency(
        "Drug_A|Rare_Cardio|all",
        "Drug_B|Rare_Cardio|all",
        SentinelAdjacency::Mechanistic,
        Some(0.9),
    );
    if let Some(cell_b) = grid.get_cell_mut("Drug_B|Rare_Cardio|all") {
        cell_b.observable = Observable::no();
        cell_b.update_signal(10, 1000.0, false, false, 0, 30);
    }
    let a_id = "Drug_A|Rare_Cardio|all";
    if let Some(cell_a) = grid.get_cell_mut(a_id) {
        cell_a.update_signal(10000, 1.0, true, false, 0, 30);
    }
    let _ = grid.propagate_signal(a_id);
    if let Some(cell_b) = grid.get_cell("Drug_B|Rare_Cardio|all") {
        assert!(cell_b.signal.value() > 0.01);
        if let Some(cell_a_mut) = grid.get_cell_mut(a_id) {
            cell_a_mut.update_signal(1000000000, 0.0001, true, false, 0, 30);
        }
        let _ = grid.propagate_signal(a_id);
        if let Some(cb_f) = grid.get_cell("Drug_B|Rare_Cardio|all") {
            if cb_f.signal.value() >= S_MODERATE {
                assert_eq!(cb_f.status, SentinelStatus::SilentRisk);
            }
        }
    }
}

fn test_uniqueness_bound() {
    let mut cell = SentinelCellState::new("Drug", "Event", "all");
    cell.update_signal(1000000000, 0.000000001, true, false, 0, 30);
    assert!(cell.unrepeatable.value() > 0.0);
}

fn test_attenuation_path() {
    let mut grid = SentinelGrid::new();
    let nodes = ["C1", "C2", "C3"];
    for n in nodes {
        grid.add_cell(n, "E", "all");
        if let Some(c) = grid.get_cell_mut(&format!("{}|E|all", n)) {
            c.update_signal(10, 1000.0, false, false, 0, 30);
        }
    }
    grid.set_adjacency(
        "C1|E|all",
        "C2|E|all",
        SentinelAdjacency::Mechanistic,
        Some(0.8),
    );
    grid.set_adjacency(
        "C2|E|all",
        "C3|E|all",
        SentinelAdjacency::Mechanistic,
        Some(0.8),
    );
    if let Some(c1) = grid.get_cell_mut("C1|E|all") {
        c1.update_signal(500, 1.0, true, false, 0, 30);
    }
    let _ = grid.propagate_signal("C1|E|all");
    let _ = grid.propagate_signal("C2|E|all");
    let s1 = grid
        .get_cell("C1|E|all")
        .map(|c| c.signal.value())
        .unwrap_or(0.0);
    let s2 = grid
        .get_cell("C2|E|all")
        .map(|c| c.signal.value())
        .unwrap_or(0.0);
    let s3 = grid
        .get_cell("C3|E|all")
        .map(|c| c.signal.value())
        .unwrap_or(0.0);
    assert!(s1 > s2);
    assert!(s2 > s3);
}

#[cfg(test)]
mod experiential_tests {
    use super::*;
    #[test]
    fn run_all_scenarios() {
        test_class_cascade_and_silent_risk();
        test_uniqueness_bound();
        test_attenuation_path();
    }
}
