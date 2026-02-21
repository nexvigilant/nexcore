//! Unit tests for OreType
//!
//! Phase 0 Preclinical: Mechanism isolation tests

use borrow_miner::game::{OreType};

#[test]
fn ore_base_values_are_monotonically_increasing() {
    let ores = [
        OreType::Iron,
        OreType::Copper,
        OreType::Silver,
        OreType::Gold,
        OreType::Platinum,
    ];

    for window in ores.windows(2) {
        assert!(
            window[0].base_value() < window[1].base_value(),
            "{:?} ({}) should be less valuable than {:?} ({})",
            window[0], window[0].base_value(),
            window[1], window[1].base_value()
        );
    }
}

#[test]
fn ore_names_are_non_empty() {
    let ores = [
        OreType::Iron,
        OreType::Copper,
        OreType::Silver,
        OreType::Gold,
        OreType::Platinum,
    ];

    for ore in ores {
        assert!(!ore.name().is_empty(), "{:?} should have a non-empty name", ore);
    }
}

#[test]
fn ore_colors_are_valid_hex() {
    let ores = [
        OreType::Iron,
        OreType::Copper,
        OreType::Silver,
        OreType::Gold,
        OreType::Platinum,
    ];

    for ore in ores {
        let color = ore.color();
        assert!(color.starts_with('#'), "{:?} color should start with #", ore);
        assert!(color.len() == 7, "{:?} color should be 7 chars (#RRGGBB)", ore);
    }
}

#[test]
fn only_gold_and_platinum_are_rare() {
    assert!(!OreType::Iron.is_rare());
    assert!(!OreType::Copper.is_rare());
    assert!(!OreType::Silver.is_rare());
    assert!(OreType::Gold.is_rare());
    assert!(OreType::Platinum.is_rare());
}

#[test]
fn ore_symbols_are_non_empty() {
    let ores = [
        OreType::Iron,
        OreType::Copper,
        OreType::Silver,
        OreType::Gold,
        OreType::Platinum,
    ];

    for ore in ores {
        assert!(!ore.symbol().is_empty(), "{:?} should have a symbol", ore);
    }
}
