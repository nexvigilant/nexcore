//! # Lymph Bridge
//!
//! Inter-crate pipeline: Lymphatic → Urinary.
//!
//! Converts lymphatic `OverflowItem`s into urinary `Bladder` inputs
//! for filtration and eventual excretion.
//!
//! ```text
//! Lymphatic::OverflowItem → Bladder::add → flush → excretion
//! ```

use nexcore_lymphatic::OverflowItem;

use crate::{Bladder, FilterCategory};

/// Content prefix for pressure alert overflow items (set by `circulation_bridge`).
const PREFIX_PRESSURE_ALERT: &str = "pressure-alert";
/// Content prefix for pulse overflow items (set by `circulation_bridge`).
const PREFIX_PULSE_OVERFLOW: &str = "pulse-overflow";

/// Convert a lymphatic overflow item into a bladder-compatible string.
///
/// **Biological mapping**: Lymph → kidney filtration — lymphatic fluid
/// drains into the venous system, which passes through the kidneys
/// for waste extraction.
pub fn overflow_to_bladder_item(item: &OverflowItem) -> String {
    format!("[{}:p{}] {}", item.source, item.priority, item.content)
}

/// Convert a batch of overflow items into bladder inputs.
///
/// Filters out items that have already been drained (processed by
/// lymphatic system), only forwarding unprocessed waste.
pub fn overflow_batch_to_bladder(items: &[OverflowItem]) -> Vec<String> {
    items
        .iter()
        .filter(|item| !item.drained)
        .map(|item| overflow_to_bladder_item(item))
        .collect()
}

/// Classify a lymphatic overflow item into a urinary filter category.
///
/// **Biological mapping**: Glomerular selectivity — the kidney filters
/// different substances into different tubular segments based on type.
pub fn classify_overflow(item: &OverflowItem) -> FilterCategory {
    if item.content.contains(PREFIX_PRESSURE_ALERT) || item.content.contains("threat") {
        FilterCategory::Telemetry
    } else if item.content.contains("log") || item.content.contains(PREFIX_PULSE_OVERFLOW) {
        FilterCategory::Logs
    } else if item.content.contains("session") {
        FilterCategory::Sessions
    } else if item.content.contains("artifact") {
        FilterCategory::Artifacts
    } else {
        FilterCategory::TempFiles
    }
}

/// Feed overflow items into a bladder, returning the count accepted.
///
/// Items that exceed bladder capacity are rejected (returns count < input).
/// Pre-filters drained items via [`overflow_batch_to_bladder`].
pub fn drain_to_bladder(bladder: &mut Bladder, items: &[OverflowItem]) -> usize {
    let formatted = overflow_batch_to_bladder(items);
    let mut accepted = 0;
    for item in formatted {
        if bladder.add(item) {
            accepted += 1;
        }
    }
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(content: &str, source: &str, priority: u8) -> OverflowItem {
        OverflowItem::new(content, source, priority)
    }

    #[test]
    fn test_overflow_to_bladder_item() {
        let item = make_item("test-content", "circulatory", 2);
        let result = overflow_to_bladder_item(&item);

        assert!(result.contains("circulatory"));
        assert!(result.contains("p2"));
        assert!(result.contains("test-content"));
    }

    #[test]
    fn test_overflow_batch_filters_drained() {
        let mut items = vec![
            make_item("first", "circ", 1),
            make_item("second", "circ", 2),
        ];
        // Mark first as drained
        items[0].drained = true;

        let result = overflow_batch_to_bladder(&items);
        assert_eq!(result.len(), 1);
        assert!(result[0].contains("second"));
    }

    #[test]
    fn test_classify_overflow_telemetry() {
        let item = make_item("pressure-alert:ratio=0.50", "circ", 1);
        assert_eq!(classify_overflow(&item), FilterCategory::Telemetry);
    }

    #[test]
    fn test_classify_overflow_logs() {
        let item = make_item("pulse-overflow-ts-0", "circ", 3);
        assert_eq!(classify_overflow(&item), FilterCategory::Logs);
    }

    #[test]
    fn test_classify_overflow_default() {
        let item = make_item("random-waste", "circ", 3);
        assert_eq!(classify_overflow(&item), FilterCategory::TempFiles);
    }

    #[test]
    fn test_drain_to_bladder() {
        let mut bladder = Bladder::new(100);
        let items = vec![
            make_item("waste-1", "lymph", 2),
            make_item("waste-2", "lymph", 3),
        ];

        let accepted = drain_to_bladder(&mut bladder, &items);
        assert_eq!(accepted, 2);
    }

    #[test]
    fn test_drain_to_bladder_skips_drained() {
        let mut bladder = Bladder::new(100);
        let mut items = vec![
            make_item("waste-1", "lymph", 2),
            make_item("waste-2", "lymph", 3),
        ];
        items[0].drained = true;

        let accepted = drain_to_bladder(&mut bladder, &items);
        assert_eq!(accepted, 1);
    }
}
