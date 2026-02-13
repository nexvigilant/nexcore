//! Dependency graph operations.

use crate::types::Primitive;
use indexmap::IndexMap;

/// Build dependency graph from primitives.
#[must_use]
pub fn build_dependency_map(primitives: &[Primitive]) -> IndexMap<String, Vec<String>> {
    let mut map: IndexMap<String, Vec<String>> = IndexMap::new();
    for p in primitives {
        for dep in &p.depends_on {
            map.entry(dep.clone()).or_default().push(p.name.clone());
        }
    }
    map
}
