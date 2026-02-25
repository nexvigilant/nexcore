//! Multi-taxonomy registry with built-in Golden Dome and Pharmacovigilance.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use crate::cybersecurity::cybersecurity;
use crate::golden_dome::golden_dome;
use crate::pharmacovigilance::pharmacovigilance;
use crate::taxonomy::DomainTaxonomy;

/// Central registry of extracted domain taxonomies.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TaxonomyRegistry {
    taxonomies: HashMap<String, DomainTaxonomy>,
}

impl TaxonomyRegistry {
    /// Create a new registry pre-loaded with built-in taxonomies.
    pub fn new() -> Self {
        let mut reg = Self {
            taxonomies: HashMap::new(),
        };
        let gd = golden_dome();
        reg.taxonomies.insert(normalize_key(&gd.name), gd);
        let pv = pharmacovigilance();
        reg.taxonomies.insert(normalize_key(&pv.name), pv);
        let cs = cybersecurity();
        reg.taxonomies.insert(normalize_key(&cs.name), cs);
        reg
    }

    /// Create an empty registry (no built-ins).
    pub fn empty() -> Self {
        Self {
            taxonomies: HashMap::new(),
        }
    }

    /// Register a new taxonomy. Overwrites if name collision.
    pub fn register(&mut self, taxonomy: DomainTaxonomy) {
        self.taxonomies
            .insert(normalize_key(&taxonomy.name), taxonomy);
    }

    /// Get a taxonomy by name (case-insensitive, kebab-case normalized).
    pub fn get(&self, name: &str) -> Option<&DomainTaxonomy> {
        self.taxonomies.get(&normalize_key(name))
    }

    /// List all taxonomy names (sorted).
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.taxonomies.values().map(|t| t.name.as_str()).collect();
        names.sort();
        names
    }

    /// Names of built-in taxonomies that ship with the crate.
    ///
    /// Useful for discovering valid domain names without instantiating a registry.
    #[must_use]
    pub fn known_builtin_domains() -> &'static [&'static str] {
        &["golden-dome", "pharmacovigilance", "cybersecurity"]
    }

    /// Number of registered taxonomies.
    pub fn len(&self) -> usize {
        self.taxonomies.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.taxonomies.is_empty()
    }

    /// Total primitives across all registered taxonomies.
    pub fn total_primitives(&self) -> usize {
        self.taxonomies.values().map(|t| t.primitives.len()).sum()
    }

    /// Save a taxonomy to a JSON file.
    pub fn save_json(&self, name: &str, path: &Path) -> io::Result<()> {
        let tax = self.get(name).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("taxonomy '{}' not found in registry", name),
            )
        })?;
        let json = serde_json::to_string_pretty(tax)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Load a taxonomy from a JSON file and register it.
    pub fn load_json(&mut self, path: &Path) -> io::Result<String> {
        let data = std::fs::read_to_string(path)?;
        let tax: DomainTaxonomy = serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let name = tax.name.clone();
        self.register(tax);
        Ok(name)
    }

    /// Export all taxonomies as a JSON array.
    pub fn export_all_json(&self) -> Result<String, serde_json::Error> {
        let all: Vec<&DomainTaxonomy> = self.taxonomies.values().collect();
        serde_json::to_string_pretty(&all)
    }
}

impl Default for TaxonomyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize a taxonomy name to lowercase kebab-case.
fn normalize_key(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taxonomy::{DomainTaxonomy, Primitive, Tier};

    #[test]
    fn default_includes_golden_dome() {
        let reg = TaxonomyRegistry::new();
        assert_eq!(reg.len(), 3);
        assert!(reg.get("golden-dome").is_some());
        assert!(reg.get("Golden Dome").is_some()); // case-insensitive
    }

    #[test]
    fn empty_registry() {
        let reg = TaxonomyRegistry::empty();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn default_includes_pharmacovigilance() {
        let reg = TaxonomyRegistry::new();
        assert!(reg.get("pharmacovigilance").is_some());
        assert!(reg.get("Pharmacovigilance").is_some()); // case-insensitive
    }

    #[test]
    fn default_includes_cybersecurity() {
        let reg = TaxonomyRegistry::new();
        assert!(reg.get("cybersecurity").is_some());
        assert!(reg.get("Cybersecurity").is_some()); // case-insensitive
    }

    #[test]
    fn register_custom() {
        let mut reg = TaxonomyRegistry::new();
        let mut tax = DomainTaxonomy::new("SupplyChain", "Supply chain primitives");
        tax.primitives.push(Primitive::new(
            "logistics",
            "Movement coordination",
            Tier::T2P,
        ));
        reg.register(tax);
        assert_eq!(reg.len(), 4);
        assert!(reg.get("supplychain").is_some());
    }

    #[test]
    fn total_primitives() {
        let reg = TaxonomyRegistry::new();
        assert_eq!(reg.total_primitives(), 90); // Golden Dome (30) + PV (30) + Cyber (30)
    }

    #[test]
    fn list_names() {
        let reg = TaxonomyRegistry::new();
        let names = reg.list();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Golden Dome"));
        assert!(names.contains(&"Pharmacovigilance"));
        assert!(names.contains(&"Cybersecurity"));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let reg = TaxonomyRegistry::new();
        let dir = std::env::temp_dir().join("nexcore-dp-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("golden-dome.json");

        // Save
        let save_result = reg.save_json("Golden Dome", &path);
        assert!(save_result.is_ok(), "save failed: {:?}", save_result.err());

        // Load into fresh registry
        let mut reg2 = TaxonomyRegistry::empty();
        let load_result = reg2.load_json(&path);
        assert!(load_result.is_ok(), "load failed: {:?}", load_result.err());

        let loaded = reg2.get("Golden Dome");
        assert!(loaded.is_some());
        let loaded = loaded.unwrap_or_else(|| {
            reg.get("Golden Dome").unwrap_or_else(|| {
                &reg.taxonomies
                    .values()
                    .next()
                    .unwrap_or_else(|| unreachable!())
            })
        });
        assert_eq!(loaded.primitives.len(), 30);
        assert_eq!(loaded.transfers.len(), 56);

        // Cleanup
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn save_nonexistent_taxonomy() {
        let reg = TaxonomyRegistry::new();
        let path = std::env::temp_dir().join("nexcore-dp-test-noexist.json");
        let result = reg.save_json("NoSuchTaxonomy", &path);
        assert!(result.is_err());
    }

    #[test]
    fn export_all_json() {
        let reg = TaxonomyRegistry::new();
        let json = reg.export_all_json();
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        assert!(json_str.contains("Golden Dome"));
        assert!(json_str.contains("Pharmacovigilance"));
        assert!(json_str.contains("Cybersecurity"));
    }
}
