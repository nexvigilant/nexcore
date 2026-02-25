//! Route caching and mesh state persistence.
//!
//! ## Primitive Foundation
//! - `MeshSnapshot`: π (Persistence) + ς (State) + μ (Mapping) + σ (Sequence)
//! - `SnapshotStore`: π (Persistence) + μ (Mapping) + ∂ (Boundary) + → (Causality) + λ (Location) + ς (State)
//!
//! ## Design
//!
//! `RoutingTable` and `NeighborRegistry` use `Arc<DashMap>` internally,
//! which isn't directly serializable. We define snapshot types that capture
//! a point-in-time view as plain `Vec`s, then serialize those to JSON files.

use crate::neighbor::{Neighbor, NeighborRegistry};
use crate::node::NodeState;
use crate::routing::{Route, RoutingTable};
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ============================================================================
// Snapshot Types
// ============================================================================

/// Serializable snapshot of a routing table.
///
/// Tier: T2-P | Dominant: π (Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSnapshot {
    /// Destination → routes mapping, flattened from DashMap.
    pub entries: Vec<(String, Vec<Route>)>,
}

/// Serializable snapshot of a neighbor registry.
///
/// Tier: T2-P | Dominant: π (Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborSnapshot {
    /// All neighbors at time of snapshot.
    pub neighbors: Vec<Neighbor>,
}

/// Complete mesh state snapshot combining routes, neighbors, and node metadata.
///
/// Tier: T2-C | Dominant: π (Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshSnapshot {
    /// Node ID that produced this snapshot.
    pub node_id: String,
    /// Node state at time of snapshot.
    pub node_state: NodeState,
    /// Routing table snapshot.
    pub routes: RouteSnapshot,
    /// Neighbor registry snapshot.
    pub neighbors: NeighborSnapshot,
    /// ISO 8601 timestamp when the snapshot was taken.
    pub timestamp: DateTime,
    /// Snapshot format version for future compatibility.
    pub version: u32,
}

impl MeshSnapshot {
    /// Create a snapshot from live node components.
    pub fn capture(
        node_id: &str,
        node_state: NodeState,
        routing: &RoutingTable,
        neighbors: &NeighborRegistry,
    ) -> Self {
        Self {
            node_id: node_id.to_string(),
            node_state,
            routes: RouteSnapshot {
                entries: routing.snapshot(),
            },
            neighbors: NeighborSnapshot {
                neighbors: neighbors.snapshot(),
            },
            timestamp: DateTime::now(),
            version: 1,
        }
    }

    /// Number of route destinations in the snapshot.
    pub fn route_destination_count(&self) -> usize {
        self.routes.entries.len()
    }

    /// Total number of routes across all destinations.
    pub fn total_route_count(&self) -> usize {
        self.routes.entries.iter().map(|(_, v)| v.len()).sum()
    }

    /// Number of neighbors in the snapshot.
    pub fn neighbor_count(&self) -> usize {
        self.neighbors.neighbors.len()
    }
}

// ============================================================================
// PersistenceConfig
// ============================================================================

/// Configuration for snapshot persistence.
///
/// Tier: T2-P | Dominant: ∂ (Boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Directory path for cache files.
    pub cache_dir: String,
    /// Maximum number of snapshots to retain per node.
    pub max_snapshots: usize,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            cache_dir: "~/nexcore/mesh/cache".to_string(),
            max_snapshots: 5,
        }
    }
}

// ============================================================================
// SnapshotStore — Load/save snapshots to JSON files
// ============================================================================

/// Persistent store for mesh snapshots.
///
/// Tier: T3 | Dominant: π (Persistence)
///
/// Saves and loads `MeshSnapshot` as JSON files on disk.
/// Each node gets its own file: `<cache_dir>/<node_id>.json`.
#[derive(Debug, Clone)]
pub struct SnapshotStore {
    /// Configuration.
    pub config: PersistenceConfig,
}

impl SnapshotStore {
    /// Create a new snapshot store with the given config.
    pub fn new(config: PersistenceConfig) -> Self {
        Self { config }
    }

    /// Create with default config.
    pub fn with_defaults() -> Self {
        Self::new(PersistenceConfig::default())
    }

    /// Save a snapshot to a JSON file.
    pub fn save(&self, snapshot: &MeshSnapshot, path: &Path) -> Result<(), SnapshotError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| SnapshotError::Io(e.to_string()))?;
            }
        }

        let json = serde_json::to_string_pretty(snapshot)
            .map_err(|e| SnapshotError::Serialize(e.to_string()))?;

        std::fs::write(path, json).map_err(|e| SnapshotError::Io(e.to_string()))?;

        Ok(())
    }

    /// Load a snapshot from a JSON file.
    pub fn load(&self, path: &Path) -> Result<MeshSnapshot, SnapshotError> {
        let json = std::fs::read_to_string(path).map_err(|e| SnapshotError::Io(e.to_string()))?;

        let snapshot: MeshSnapshot =
            serde_json::from_str(&json).map_err(|e| SnapshotError::Deserialize(e.to_string()))?;

        Ok(snapshot)
    }

    /// Restore a routing table from a snapshot.
    pub fn restore_routing(routing: &RoutingTable, snapshot: &MeshSnapshot) {
        routing.restore(&snapshot.routes.entries);
    }

    /// Restore a neighbor registry from a snapshot.
    pub fn restore_neighbors(registry: &NeighborRegistry, snapshot: &MeshSnapshot) {
        registry.restore(&snapshot.neighbors.neighbors);
    }
}

// ============================================================================
// SnapshotError
// ============================================================================

/// Errors that can occur during snapshot operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotError {
    /// I/O error (file not found, permissions, etc.)
    Io(String),
    /// Serialization error
    Serialize(String),
    /// Deserialization error
    Deserialize(String),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "snapshot I/O error: {e}"),
            Self::Serialize(e) => write!(f, "snapshot serialize error: {e}"),
            Self::Deserialize(e) => write!(f, "snapshot deserialize error: {e}"),
        }
    }
}

impl std::error::Error for SnapshotError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{Path as MeshPath, RouteQuality};

    fn make_route(dest: &str, next: &str, reliability: f64) -> Route {
        let mut path = MeshPath::new("self", 16);
        let _ = path.add_hop(dest);
        Route::new(dest, next, path, RouteQuality::new(50.0, reliability, 1))
    }

    fn make_neighbor(id: &str, reliability: f64) -> Neighbor {
        Neighbor::new(id, RouteQuality::new(50.0, reliability, 1))
    }

    #[test]
    fn mesh_snapshot_capture() {
        let routing = RoutingTable::with_defaults();
        routing.upsert(make_route("dest-1", "via-a", 0.9));
        routing.upsert(make_route("dest-2", "via-b", 0.8));

        let registry = NeighborRegistry::new(10);
        let _ = registry.add(make_neighbor("n1", 0.95));
        let _ = registry.add(make_neighbor("n2", 0.85));

        let snapshot = MeshSnapshot::capture("node-x", NodeState::Active, &routing, &registry);

        assert_eq!(snapshot.node_id, "node-x");
        assert_eq!(snapshot.node_state, NodeState::Active);
        assert_eq!(snapshot.route_destination_count(), 2);
        assert_eq!(snapshot.total_route_count(), 2);
        assert_eq!(snapshot.neighbor_count(), 2);
        assert_eq!(snapshot.version, 1);
    }

    #[test]
    fn snapshot_save_and_load_roundtrip() {
        let routing = RoutingTable::with_defaults();
        routing.upsert(make_route("alpha", "hop-a", 0.95));
        routing.upsert(make_route("beta", "hop-b", 0.88));

        let registry = NeighborRegistry::new(10);
        let _ = registry.add(make_neighbor("peer-1", 0.9));

        let snapshot =
            MeshSnapshot::capture("roundtrip-node", NodeState::Active, &routing, &registry);

        let dir = std::env::temp_dir().join("nexcore-mesh-test-persistence");
        let path = dir.join("roundtrip.json");

        let store = SnapshotStore::with_defaults();
        let save_result = store.save(&snapshot, &path);
        assert!(save_result.is_ok(), "save failed: {:?}", save_result.err());

        let loaded = store.load(&path);
        assert!(loaded.is_ok(), "load failed: {:?}", loaded.err());

        let loaded = loaded.ok();
        assert!(loaded.is_some());
        if let Some(loaded) = loaded {
            assert_eq!(loaded.node_id, "roundtrip-node");
            assert_eq!(loaded.route_destination_count(), 2);
            assert_eq!(loaded.neighbor_count(), 1);
            assert_eq!(loaded.version, 1);
        }

        // Cleanup
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn restore_routing_table_from_snapshot() {
        let routing = RoutingTable::with_defaults();
        routing.upsert(make_route("d1", "h1", 0.9));
        routing.upsert(make_route("d2", "h2", 0.8));

        let registry = NeighborRegistry::new(10);
        let snapshot = MeshSnapshot::capture("n", NodeState::Active, &routing, &registry);

        // Create a fresh routing table and restore
        let fresh = RoutingTable::with_defaults();
        assert_eq!(fresh.destination_count(), 0);

        SnapshotStore::restore_routing(&fresh, &snapshot);
        assert_eq!(fresh.destination_count(), 2);
        assert!(fresh.has_route_to("d1"));
        assert!(fresh.has_route_to("d2"));
    }

    #[test]
    fn restore_neighbor_registry_from_snapshot() {
        let registry = NeighborRegistry::new(10);
        let _ = registry.add(make_neighbor("a", 0.95));
        let _ = registry.add(make_neighbor("b", 0.85));

        let routing = RoutingTable::with_defaults();
        let snapshot = MeshSnapshot::capture("n", NodeState::Active, &routing, &registry);

        // Create a fresh registry and restore
        let fresh = NeighborRegistry::new(10);
        assert!(fresh.is_empty());

        SnapshotStore::restore_neighbors(&fresh, &snapshot);
        assert_eq!(fresh.len(), 2);
        assert!(fresh.contains("a"));
        assert!(fresh.contains("b"));
    }

    #[test]
    fn load_missing_file_returns_error() {
        let store = SnapshotStore::with_defaults();
        let result = store.load(Path::new("/tmp/nonexistent-mesh-snapshot-xyz.json"));
        assert!(result.is_err());
        if let Err(SnapshotError::Io(_)) = result {
            // expected
        } else {
            panic!("expected Io error, got: {:?}", result);
        }
    }

    #[test]
    fn snapshot_contains_timestamp() {
        let routing = RoutingTable::with_defaults();
        let registry = NeighborRegistry::new(10);
        let before = DateTime::now();
        let snapshot = MeshSnapshot::capture("ts-node", NodeState::Active, &routing, &registry);
        let after = DateTime::now();

        assert!(snapshot.timestamp >= before);
        assert!(snapshot.timestamp <= after);
    }

    #[test]
    fn empty_routing_table_snapshot() {
        let routing = RoutingTable::with_defaults();
        let registry = NeighborRegistry::new(10);
        let snapshot = MeshSnapshot::capture("empty", NodeState::Initializing, &routing, &registry);

        assert_eq!(snapshot.route_destination_count(), 0);
        assert_eq!(snapshot.total_route_count(), 0);
        assert_eq!(snapshot.neighbor_count(), 0);
    }

    #[test]
    fn snapshot_error_display() {
        let e = SnapshotError::Io("file not found".to_string());
        assert!(e.to_string().contains("I/O error"));

        let e = SnapshotError::Serialize("bad data".to_string());
        assert!(e.to_string().contains("serialize error"));

        let e = SnapshotError::Deserialize("invalid json".to_string());
        assert!(e.to_string().contains("deserialize error"));
    }

    #[test]
    fn persistence_config_defaults() {
        let cfg = PersistenceConfig::default();
        assert!(cfg.cache_dir.contains("mesh/cache"));
        assert_eq!(cfg.max_snapshots, 5);
    }

    #[test]
    fn routing_table_snapshot_restore_roundtrip() {
        let table = RoutingTable::with_defaults();
        table.upsert(make_route("x", "a", 0.9));
        table.upsert(make_route("x", "b", 0.7));
        table.upsert(make_route("y", "c", 0.8));

        let snap = table.snapshot();
        table.clear();
        assert_eq!(table.destination_count(), 0);

        table.restore(&snap);
        assert!(table.has_route_to("x"));
        assert!(table.has_route_to("y"));
        assert!(table.total_routes() >= 2); // at least x(2) + y(1) entries
    }

    #[test]
    fn neighbor_registry_snapshot_restore_roundtrip() {
        let reg = NeighborRegistry::new(5);
        let _ = reg.add(make_neighbor("p1", 0.9));
        let _ = reg.add(make_neighbor("p2", 0.8));

        let snap = reg.snapshot();
        reg.restore(&[]); // clear
        assert!(reg.is_empty());

        reg.restore(&snap);
        assert_eq!(reg.len(), 2);
        assert!(reg.contains("p1"));
        assert!(reg.contains("p2"));
    }
}
