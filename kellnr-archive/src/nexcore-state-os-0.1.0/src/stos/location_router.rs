// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Layer 13: Location Router (STOS-LC)
//!
//! **Dominant Primitive**: λ (Location)
//!
//! Routes state machine operations to specific locations,
//! manages partitioning, and handles distributed execution.
//!
//! ## Tier Classification
//!
//! `LocationRouter` is T2-C (λ + μ + Σ) — location, mapping, sum.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::MachineId;

/// A location identifier.
pub type LocationId = u64;

/// Location information.
#[derive(Debug, Clone)]
pub struct Location {
    /// Location ID.
    pub id: LocationId,
    /// Location name.
    pub name: String,
    /// Parent location (for hierarchy).
    pub parent: Option<LocationId>,
    /// Capacity (max machines).
    pub capacity: usize,
    /// Current load.
    pub load: usize,
    /// Whether location is active.
    pub active: bool,
}

impl Location {
    /// Create a new location.
    #[must_use]
    pub fn new(id: LocationId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            parent: None,
            capacity: usize::MAX,
            load: 0,
            active: true,
        }
    }

    /// With parent.
    #[must_use]
    pub fn with_parent(mut self, parent: LocationId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// With capacity.
    #[must_use]
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Available capacity.
    #[must_use]
    pub fn available(&self) -> usize {
        self.capacity.saturating_sub(self.load)
    }

    /// Whether location can accept a machine.
    #[must_use]
    pub fn can_accept(&self) -> bool {
        self.active && self.load < self.capacity
    }

    /// Load percentage.
    #[must_use]
    pub fn load_percentage(&self) -> f64 {
        if self.capacity == 0 || self.capacity == usize::MAX {
            return 0.0;
        }
        (self.load as f64 / self.capacity as f64) * 100.0
    }
}

/// Routing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    /// Round-robin across locations.
    #[default]
    RoundRobin,
    /// Least loaded location.
    LeastLoaded,
    /// Random location.
    Random,
    /// Affinity-based (sticky).
    Affinity,
}

/// A routing rule.
#[derive(Debug, Clone)]
pub struct RoutingRule {
    /// Rule ID.
    pub id: u64,
    /// Target locations.
    pub locations: Vec<LocationId>,
    /// Strategy.
    pub strategy: RoutingStrategy,
    /// Priority (higher = checked first).
    pub priority: u32,
}

impl RoutingRule {
    /// Create a new rule.
    #[must_use]
    pub fn new(id: u64, locations: Vec<LocationId>) -> Self {
        Self {
            id,
            locations,
            strategy: RoutingStrategy::RoundRobin,
            priority: 0,
        }
    }

    /// With strategy.
    #[must_use]
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// With priority.
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
}

/// The location router.
///
/// ## Tier: T2-C (λ + μ + Σ)
///
/// Dominant primitive: λ (Location)
#[derive(Debug, Clone)]
pub struct LocationRouter {
    /// All locations.
    locations: BTreeMap<LocationId, Location>,
    /// Machine to location mapping.
    assignments: BTreeMap<MachineId, LocationId>,
    /// Routing rules.
    rules: Vec<RoutingRule>,
    /// Round-robin index.
    rr_index: usize,
    /// Counter.
    counter: u64,
}

impl LocationRouter {
    /// Create a new location router.
    #[must_use]
    pub fn new() -> Self {
        Self {
            locations: BTreeMap::new(),
            assignments: BTreeMap::new(),
            rules: Vec::new(),
            rr_index: 0,
            counter: 0,
        }
    }

    /// Register a location.
    pub fn register_location(&mut self, location: Location) {
        self.locations.insert(location.id, location);
    }

    /// Create and register a location.
    pub fn create_location(&mut self, name: impl Into<String>) -> LocationId {
        self.counter = self.counter.saturating_add(1);
        let location = Location::new(self.counter, name);
        self.locations.insert(self.counter, location);
        self.counter
    }

    /// Get a location.
    #[must_use]
    pub fn get_location(&self, id: LocationId) -> Option<&Location> {
        self.locations.get(&id)
    }

    /// Get mutable location.
    pub fn get_location_mut(&mut self, id: LocationId) -> Option<&mut Location> {
        self.locations.get_mut(&id)
    }

    /// Deactivate a location.
    pub fn deactivate(&mut self, id: LocationId) {
        if let Some(loc) = self.locations.get_mut(&id) {
            loc.active = false;
        }
    }

    /// Activate a location.
    pub fn activate(&mut self, id: LocationId) {
        if let Some(loc) = self.locations.get_mut(&id) {
            loc.active = true;
        }
    }

    /// Add a routing rule.
    pub fn add_rule(&mut self, rule: RoutingRule) {
        self.rules.push(rule);
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Route a machine to a location.
    pub fn route(&mut self, machine_id: MachineId) -> Option<LocationId> {
        // Use default rule if no rules defined
        if self.rules.is_empty() {
            return self.route_default(machine_id);
        }

        // Clone rules to avoid borrow conflict
        let rules = self.rules.clone();

        // Try rules in priority order
        for rule in &rules {
            if let Some(location_id) = self.apply_rule(rule) {
                self.assign(machine_id, location_id);
                return Some(location_id);
            }
        }

        None
    }

    /// Apply a routing rule.
    fn apply_rule(&mut self, rule: &RoutingRule) -> Option<LocationId> {
        let available: Vec<LocationId> = rule
            .locations
            .iter()
            .filter(|&&id| self.locations.get(&id).is_some_and(|l| l.can_accept()))
            .copied()
            .collect();

        if available.is_empty() {
            return None;
        }

        match rule.strategy {
            RoutingStrategy::RoundRobin => {
                self.rr_index = (self.rr_index + 1) % available.len();
                Some(available[self.rr_index])
            }
            RoutingStrategy::LeastLoaded => available
                .iter()
                .filter_map(|&id| self.locations.get(&id).map(|l| (id, l.load)))
                .min_by_key(|(_, load)| *load)
                .map(|(id, _)| id),
            RoutingStrategy::Random | RoutingStrategy::Affinity => {
                // For simplicity, just pick first available
                available.first().copied()
            }
        }
    }

    /// Default routing (least loaded of all active).
    fn route_default(&mut self, machine_id: MachineId) -> Option<LocationId> {
        let location_id = self
            .locations
            .values()
            .filter(|l| l.can_accept())
            .min_by_key(|l| l.load)
            .map(|l| l.id)?;

        self.assign(machine_id, location_id);
        Some(location_id)
    }

    /// Assign a machine to a location.
    pub fn assign(&mut self, machine_id: MachineId, location_id: LocationId) {
        // Remove from old location
        if let Some(&old_loc) = self.assignments.get(&machine_id) {
            if let Some(loc) = self.locations.get_mut(&old_loc) {
                loc.load = loc.load.saturating_sub(1);
            }
        }

        // Add to new location
        if let Some(loc) = self.locations.get_mut(&location_id) {
            loc.load = loc.load.saturating_add(1);
        }

        self.assignments.insert(machine_id, location_id);
    }

    /// Unassign a machine.
    pub fn unassign(&mut self, machine_id: MachineId) {
        if let Some(location_id) = self.assignments.remove(&machine_id) {
            if let Some(loc) = self.locations.get_mut(&location_id) {
                loc.load = loc.load.saturating_sub(1);
            }
        }
    }

    /// Get a machine's location.
    #[must_use]
    pub fn location_of(&self, machine_id: MachineId) -> Option<LocationId> {
        self.assignments.get(&machine_id).copied()
    }

    /// Get machines at a location.
    #[must_use]
    pub fn machines_at(&self, location_id: LocationId) -> Vec<MachineId> {
        self.assignments
            .iter()
            .filter(|&(_, loc)| *loc == location_id)
            .map(|(&machine, _)| machine)
            .collect()
    }

    /// Total location count.
    #[must_use]
    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Active location count.
    #[must_use]
    pub fn active_location_count(&self) -> usize {
        self.locations.values().filter(|l| l.active).count()
    }
}

impl Default for LocationRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_location() {
        let mut router = LocationRouter::new();

        let id = router.create_location("zone-1");
        assert!(router.get_location(id).is_some());
        assert_eq!(router.location_count(), 1);
    }

    #[test]
    fn test_route_machine() {
        let mut router = LocationRouter::new();

        let loc1 = router.create_location("zone-1");
        let loc2 = router.create_location("zone-2");

        // Route machine
        let assigned = router.route(1);
        assert!(assigned.is_some());
        assert!(router.location_of(1).is_some());
    }

    #[test]
    fn test_least_loaded_routing() {
        let mut router = LocationRouter::new();

        let loc1 = router.create_location("zone-1");
        let loc2 = router.create_location("zone-2");

        // Assign some machines to loc1
        router.assign(1, loc1);
        router.assign(2, loc1);

        // Route new machine should go to loc2 (least loaded)
        let rule =
            RoutingRule::new(1, vec![loc1, loc2]).with_strategy(RoutingStrategy::LeastLoaded);
        router.add_rule(rule);

        let assigned = router.route(3);
        assert_eq!(assigned, Some(loc2));
    }

    #[test]
    fn test_capacity() {
        let mut router = LocationRouter::new();

        let loc = router.create_location("zone-1");
        if let Some(l) = router.get_location_mut(loc) {
            l.capacity = 2;
        }

        router.assign(1, loc);
        router.assign(2, loc);

        // Location should be full
        assert!(!router.get_location(loc).map_or(false, |l| l.can_accept()));
    }

    #[test]
    fn test_deactivate() {
        let mut router = LocationRouter::new();

        let loc = router.create_location("zone-1");
        router.deactivate(loc);

        assert!(!router.get_location(loc).map_or(true, |l| l.active));
        assert_eq!(router.active_location_count(), 0);
    }

    #[test]
    fn test_unassign() {
        let mut router = LocationRouter::new();

        let loc = router.create_location("zone-1");
        router.assign(1, loc);

        assert_eq!(router.get_location(loc).map(|l| l.load), Some(1));

        router.unassign(1);
        assert_eq!(router.get_location(loc).map(|l| l.load), Some(0));
        assert!(router.location_of(1).is_none());
    }
}
