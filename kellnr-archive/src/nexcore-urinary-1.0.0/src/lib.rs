//! # nexcore-urinary
//!
//! Urinary system for filtration, reabsorption, and waste excretion.
//!
//! Biological mapping:
//! - Kidney → Log/telemetry filter engine
//! - Nephron → Individual filter unit
//! - Glomerular filtration → Initial broad filter pass
//! - Tubular reabsorption → Reclaiming useful data
//! - Urine (excretion) → Disposed waste
//! - Filtration rate (GFR) → Processing throughput
//! - Bladder → Batch collection before disposal

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde::{Deserialize, Serialize};

pub mod claude_code;
pub mod grounding;

/// Category of data being filtered.
///
/// Biological mapping: Types of substances filtered by nephrons.
///
/// Type tier: T2-P (Σ sum, newtype pattern over filter domains)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterCategory {
    /// Telemetry data
    Telemetry,
    /// Log files
    Logs,
    /// Session data
    Sessions,
    /// Artifact storage
    Artifacts,
    /// Temporary files
    TempFiles,
}

/// Individual filter unit in the urinary system.
///
/// Biological mapping: Nephron — functional unit of the kidney that filters blood
/// and produces urine through glomerular filtration, tubular reabsorption, and secretion.
///
/// Type tier: T2-C (∂ boundary + κ comparison + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Nephron {
    /// Category of data this nephron filters
    pub category: FilterCategory,
    /// Maximum age in days before disposal
    pub retention_days: u32,
    /// If true, reabsorb recently accessed items even if old
    pub reabsorb_if_recent: bool,
    /// Total items processed through this nephron
    pub items_filtered: u64,
    /// Items reclaimed via reabsorption
    pub items_reabsorbed: u64,
}

impl Nephron {
    /// Create a new nephron with specified category and retention policy.
    pub fn new(category: FilterCategory, retention_days: u32) -> Self {
        Self {
            category,
            retention_days,
            reabsorb_if_recent: true,
            items_filtered: 0,
            items_reabsorbed: 0,
        }
    }

    /// Filter an item by age. Returns true if item should be kept (reabsorbed),
    /// false if it should be excreted.
    ///
    /// Biological mapping: Glomerular filtration + tubular reabsorption decision.
    pub fn filter(&mut self, age_days: u32) -> bool {
        self.items_filtered += 1;

        if age_days <= self.retention_days {
            self.items_reabsorbed += 1;
            true
        } else {
            false
        }
    }

    /// Get the reabsorption rate (0.0 to 1.0).
    pub fn reabsorption_rate(&self) -> f64 {
        if self.items_filtered == 0 {
            0.0
        } else {
            self.items_reabsorbed as f64 / self.items_filtered as f64
        }
    }
}

/// Initial broad filtration pass.
///
/// Biological mapping: Glomerular filtration — first step of urine formation where
/// blood pressure forces water and small solutes through the glomerular capillaries.
///
/// Type tier: T2-C (∂ boundary + N quantity + κ comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlomerularFiltration {
    /// Total items received for filtering
    pub input_count: usize,
    /// Items that passed initial filter
    pub passed_count: usize,
    /// Items blocked by initial filter
    pub blocked_count: usize,
    /// Processing rate (items per second)
    pub rate_per_second: f64,
}

impl GlomerularFiltration {
    /// Create a new glomerular filtration record.
    pub fn new(input_count: usize, passed_count: usize, rate_per_second: f64) -> Self {
        Self {
            input_count,
            passed_count,
            blocked_count: input_count.saturating_sub(passed_count),
            rate_per_second,
        }
    }

    /// Get filtration efficiency (0.0 to 1.0).
    pub fn efficiency(&self) -> f64 {
        if self.input_count == 0 {
            0.0
        } else {
            self.passed_count as f64 / self.input_count as f64
        }
    }
}

/// Reclamation of useful data from filtered stream.
///
/// Biological mapping: Tubular reabsorption — process by which nephrons extract water
/// and solutes from tubular fluid and return them to circulating blood.
///
/// Type tier: T2-C (κ comparison + N quantity + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reabsorption {
    /// Category of data being reabsorbed
    pub category: FilterCategory,
    /// Number of items reclaimed
    pub reclaimed_count: usize,
    /// Bytes reclaimed
    pub reclaimed_bytes: u64,
    /// Human-readable criteria for reabsorption
    pub criteria: String,
}

impl Reabsorption {
    /// Create a new reabsorption record.
    pub fn new(
        category: FilterCategory,
        reclaimed_count: usize,
        reclaimed_bytes: u64,
        criteria: String,
    ) -> Self {
        Self {
            category,
            reclaimed_count,
            reclaimed_bytes,
            criteria,
        }
    }

    /// Get average item size in bytes.
    pub fn avg_item_size(&self) -> u64 {
        if self.reclaimed_count == 0 {
            0
        } else {
            self.reclaimed_bytes / self.reclaimed_count as u64
        }
    }
}

/// Waste disposal record.
///
/// Biological mapping: Urine excretion — final elimination of metabolic waste,
/// toxins, and excess substances from the body.
///
/// Type tier: T2-C (∝ irreversibility + ∅ void + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Excretion {
    /// Category of waste being disposed
    pub category: FilterCategory,
    /// Number of items disposed
    pub disposed_count: usize,
    /// Bytes disposed
    pub disposed_bytes: u64,
    /// Method of disposal (delete, archive, compress, etc.)
    pub method: String,
}

impl Excretion {
    /// Create a new excretion record.
    pub fn new(
        category: FilterCategory,
        disposed_count: usize,
        disposed_bytes: u64,
        method: String,
    ) -> Self {
        Self {
            category,
            disposed_count,
            disposed_bytes,
            method,
        }
    }

    /// Get average waste item size in bytes.
    pub fn avg_waste_size(&self) -> u64 {
        if self.disposed_count == 0 {
            0
        } else {
            self.disposed_bytes / self.disposed_count as u64
        }
    }
}

/// Processing throughput measurement.
///
/// Biological mapping: Glomerular Filtration Rate (GFR) — volume of fluid filtered
/// from renal glomerular capillaries into Bowman's capsule per unit time.
///
/// Type tier: T2-C (N quantity + ν frequency + κ comparison)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FiltrationRate {
    /// Current filtration rate
    pub current_rate: f64,
    /// Target/optimal filtration rate
    pub target_rate: f64,
    /// Unit of measurement (e.g., "items/sec", "MB/sec")
    pub unit: String,
}

impl FiltrationRate {
    /// Create a new filtration rate measurement.
    pub fn new(current_rate: f64, target_rate: f64, unit: String) -> Self {
        Self {
            current_rate,
            target_rate,
            unit,
        }
    }

    /// Check if current rate is within acceptable range (80-120% of target).
    pub fn is_normal(&self) -> bool {
        let ratio = self.current_rate / self.target_rate;
        ratio >= 0.8 && ratio <= 1.2
    }

    /// Get rate as percentage of target.
    pub fn percent_of_target(&self) -> f64 {
        (self.current_rate / self.target_rate) * 100.0
    }
}

/// Batch collection before disposal.
///
/// Biological mapping: Bladder — hollow muscular organ that stores urine before
/// periodic elimination.
///
/// Type tier: T2-C (σ sequence + ∂ boundary + N quantity)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bladder {
    /// Items awaiting disposal
    pub items: Vec<String>,
    /// Maximum capacity before auto-flush
    pub capacity: usize,
    /// Auto-flush when utilization exceeds this threshold (0.0-1.0)
    pub auto_flush_threshold: f64,
}

impl Bladder {
    /// Create a new bladder with specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::new(),
            capacity,
            auto_flush_threshold: 0.9,
        }
    }

    /// Add an item to the bladder. Returns true if item was added,
    /// false if capacity exceeded.
    pub fn add(&mut self, item: String) -> bool {
        if self.items.len() < self.capacity {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    /// Get current utilization as fraction of capacity (0.0 to 1.0).
    pub fn utilization(&self) -> f64 {
        self.items.len() as f64 / self.capacity as f64
    }

    /// Check if auto-flush threshold has been exceeded.
    pub fn should_flush(&self) -> bool {
        self.utilization() >= self.auto_flush_threshold
    }

    /// Empty the bladder and return all items.
    pub fn flush(&mut self) -> Vec<String> {
        std::mem::take(&mut self.items)
    }
}

/// Overall urinary system health diagnostic.
///
/// Biological mapping: Kidney function tests — comprehensive assessment of renal
/// filtration, reabsorption, and excretion capabilities.
///
/// Type tier: T2-C (ς state + κ comparison + ∂ boundary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrinaryHealth {
    /// Number of active nephrons
    pub nephron_count: usize,
    /// GFR within normal range
    pub gfr_normal: bool,
    /// Bladder capacity adequate
    pub bladder_capacity_ok: bool,
    /// Silent failure detected (filtering stopped without alert)
    pub silent_failure_detected: bool,
    /// Retention policies actively enforced
    pub retention_policies_active: bool,
}

impl UrinaryHealth {
    /// Check if the urinary system is healthy overall.
    pub fn is_healthy(&self) -> bool {
        self.nephron_count > 0
            && self.gfr_normal
            && self.bladder_capacity_ok
            && !self.silent_failure_detected
            && self.retention_policies_active
    }

    /// Get health score (0.0 to 1.0).
    pub fn health_score(&self) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Nephron count (20%)
        max_score += 0.2;
        if self.nephron_count > 0 {
            score += 0.2;
        }

        // GFR normal (20%)
        max_score += 0.2;
        if self.gfr_normal {
            score += 0.2;
        }

        // Bladder capacity (20%)
        max_score += 0.2;
        if self.bladder_capacity_ok {
            score += 0.2;
        }

        // No silent failures (20%)
        max_score += 0.2;
        if !self.silent_failure_detected {
            score += 0.2;
        }

        // Retention policies active (20%)
        max_score += 0.2;
        if self.retention_policies_active {
            score += 0.2;
        }

        score / max_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nephron_new() {
        let nephron = Nephron::new(FilterCategory::Logs, 30);
        assert_eq!(nephron.retention_days, 30);
        assert_eq!(nephron.items_filtered, 0);
        assert_eq!(nephron.items_reabsorbed, 0);
    }

    #[test]
    fn test_nephron_filter_young_item() {
        let mut nephron = Nephron::new(FilterCategory::Logs, 30);
        let keep = nephron.filter(10);
        assert!(keep);
        assert_eq!(nephron.items_filtered, 1);
        assert_eq!(nephron.items_reabsorbed, 1);
    }

    #[test]
    fn test_nephron_filter_old_item() {
        let mut nephron = Nephron::new(FilterCategory::Logs, 30);
        let keep = nephron.filter(45);
        assert!(!keep);
        assert_eq!(nephron.items_filtered, 1);
        assert_eq!(nephron.items_reabsorbed, 0);
    }

    #[test]
    fn test_nephron_reabsorption_rate() {
        let mut nephron = Nephron::new(FilterCategory::Logs, 30);
        nephron.filter(10);
        nephron.filter(45);
        nephron.filter(20);
        assert!((nephron.reabsorption_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_glomerular_filtration_new() {
        let gf = GlomerularFiltration::new(100, 75, 10.5);
        assert_eq!(gf.input_count, 100);
        assert_eq!(gf.passed_count, 75);
        assert_eq!(gf.blocked_count, 25);
    }

    #[test]
    fn test_glomerular_filtration_efficiency() {
        let gf = GlomerularFiltration::new(100, 80, 10.0);
        assert!((gf.efficiency() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_reabsorption_avg_item_size() {
        let reabs = Reabsorption::new(FilterCategory::Sessions, 10, 1000, "recent".to_string());
        assert_eq!(reabs.avg_item_size(), 100);
    }

    #[test]
    fn test_excretion_avg_waste_size() {
        let exc = Excretion::new(FilterCategory::TempFiles, 5, 500, "delete".to_string());
        assert_eq!(exc.avg_waste_size(), 100);
    }

    #[test]
    fn test_filtration_rate_normal() {
        let rate = FiltrationRate::new(9.5, 10.0, "items/sec".to_string());
        assert!(rate.is_normal());
    }

    #[test]
    fn test_filtration_rate_abnormal_low() {
        let rate = FiltrationRate::new(5.0, 10.0, "items/sec".to_string());
        assert!(!rate.is_normal());
    }

    #[test]
    fn test_filtration_rate_percent_of_target() {
        let rate = FiltrationRate::new(8.0, 10.0, "items/sec".to_string());
        assert!((rate.percent_of_target() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_bladder_add() {
        let mut bladder = Bladder::new(3);
        assert!(bladder.add("item1".to_string()));
        assert!(bladder.add("item2".to_string()));
        assert!(bladder.add("item3".to_string()));
        assert!(!bladder.add("item4".to_string()));
    }

    #[test]
    fn test_bladder_utilization() {
        let mut bladder = Bladder::new(10);
        bladder.add("item1".to_string());
        bladder.add("item2".to_string());
        assert!((bladder.utilization() - 0.2).abs() < 0.01);
    }

    #[test]
    fn test_bladder_should_flush() {
        let mut bladder = Bladder::new(10);
        bladder.auto_flush_threshold = 0.5;
        for i in 0..4 {
            bladder.add(format!("item{}", i));
        }
        assert!(!bladder.should_flush());
        bladder.add("item5".to_string());
        assert!(bladder.should_flush());
    }

    #[test]
    fn test_bladder_flush() {
        let mut bladder = Bladder::new(10);
        bladder.add("item1".to_string());
        bladder.add("item2".to_string());
        let items = bladder.flush();
        assert_eq!(items.len(), 2);
        assert_eq!(bladder.items.len(), 0);
    }

    #[test]
    fn test_urinary_health_is_healthy() {
        let health = UrinaryHealth {
            nephron_count: 5,
            gfr_normal: true,
            bladder_capacity_ok: true,
            silent_failure_detected: false,
            retention_policies_active: true,
        };
        assert!(health.is_healthy());
    }

    #[test]
    fn test_urinary_health_unhealthy() {
        let health = UrinaryHealth {
            nephron_count: 0,
            gfr_normal: true,
            bladder_capacity_ok: true,
            silent_failure_detected: false,
            retention_policies_active: true,
        };
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_urinary_health_score() {
        let health = UrinaryHealth {
            nephron_count: 5,
            gfr_normal: true,
            bladder_capacity_ok: true,
            silent_failure_detected: false,
            retention_policies_active: true,
        };
        assert!((health.health_score() - 1.0).abs() < 0.01);
    }
}
