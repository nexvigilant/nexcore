//! # Primitive Extraction Capability
//!
//! Systematic primitive extraction for newly developed files.
//! Run after each significant file is created/modified to catalog primitives.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_hooks::primitives::{PrimitiveRegistry, extract_from_module};
//!
//! // After developing a new module
//! let extraction = extract_from_module("bonding", &module_terms);
//! registry.register(extraction);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primitive tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tier {
    /// T1: Universal primitive (ALL domains)
    T1Universal,
    /// T2-P: Cross-domain primitive (atomic, multiple domains)
    T2Primitive,
    /// T2-C: Cross-domain composite (built from primitives)
    T2Composite,
    /// T3: Domain-specific
    T3Domain,
}

impl Tier {
    /// Short display code
    pub fn code(&self) -> &'static str {
        match self {
            Tier::T1Universal => "T1",
            Tier::T2Primitive => "T2-P",
            Tier::T2Composite => "T2-C",
            Tier::T3Domain => "T3",
        }
    }

    /// Transfer confidence multiplier
    pub fn transfer_multiplier(&self) -> f64 {
        match self {
            Tier::T1Universal => 1.0,
            Tier::T2Primitive => 0.9,
            Tier::T2Composite => 0.7,
            Tier::T3Domain => 0.4,
        }
    }
}

/// A single extracted primitive or composite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    /// Term name
    pub name: String,
    /// Definition
    pub definition: String,
    /// Tier classification
    pub tier: Tier,
    /// Components (empty for true primitives)
    pub components: Vec<String>,
    /// Domains where this appears
    pub domains: Vec<String>,
    /// Rust manifestation (type, trait, function)
    pub rust_form: Option<String>,
    /// Transfer confidence (0.0-1.0)
    pub transfer_confidence: f64,
}

impl Primitive {
    /// Create a T1 universal primitive
    pub fn t1(name: &str, definition: &str) -> Self {
        Self {
            name: name.to_string(),
            definition: definition.to_string(),
            tier: Tier::T1Universal,
            components: Vec::new(),
            domains: vec!["*".to_string()],
            rust_form: None,
            transfer_confidence: 1.0,
        }
    }

    /// Create a T2-P cross-domain primitive
    pub fn t2p(name: &str, definition: &str, domains: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            definition: definition.to_string(),
            tier: Tier::T2Primitive,
            components: Vec::new(),
            domains: domains.iter().map(|s| s.to_string()).collect(),
            rust_form: None,
            transfer_confidence: 0.9,
        }
    }

    /// Create a T2-C composite
    pub fn t2c(name: &str, definition: &str, components: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            definition: definition.to_string(),
            tier: Tier::T2Composite,
            components: components.iter().map(|s| s.to_string()).collect(),
            domains: Vec::new(),
            rust_form: None,
            transfer_confidence: 0.7,
        }
    }

    /// Create a T3 domain-specific term
    pub fn t3(name: &str, definition: &str, domain: &str) -> Self {
        Self {
            name: name.to_string(),
            definition: definition.to_string(),
            tier: Tier::T3Domain,
            components: Vec::new(),
            domains: vec![domain.to_string()],
            rust_form: None,
            transfer_confidence: 0.4,
        }
    }

    /// Set Rust manifestation
    pub fn with_rust(mut self, rust_form: &str) -> Self {
        self.rust_form = Some(rust_form.to_string());
        self
    }

    /// Set components (for composites)
    pub fn with_components(mut self, components: &[&str]) -> Self {
        self.components = components.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Is this a true primitive (no components)?
    pub fn is_atomic(&self) -> bool {
        self.components.is_empty()
    }
}

/// Extraction result for a module/file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    /// Module/file name
    pub module: String,
    /// Source file path
    pub source_path: Option<String>,
    /// Extraction timestamp
    pub extracted_at: u64,
    /// All primitives found
    pub primitives: Vec<Primitive>,
    /// Dependency graph (term -> dependencies)
    pub dependencies: HashMap<String, Vec<String>>,
    /// Graph depth
    pub depth: usize,
}

impl Extraction {
    /// Create a new extraction
    pub fn new(module: &str) -> Self {
        Self {
            module: module.to_string(),
            source_path: None,
            extracted_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            primitives: Vec::new(),
            dependencies: HashMap::new(),
            depth: 0,
        }
    }

    /// Set source path
    pub fn from_path(mut self, path: &str) -> Self {
        self.source_path = Some(path.to_string());
        self
    }

    /// Add a primitive
    pub fn add(&mut self, primitive: Primitive) {
        self.primitives.push(primitive);
    }

    /// Add dependency edge
    pub fn add_dep(&mut self, from: &str, to: &str) {
        self.dependencies
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    /// Count by tier
    pub fn tier_counts(&self) -> HashMap<Tier, usize> {
        let mut counts = HashMap::new();
        for p in &self.primitives {
            *counts.entry(p.tier).or_insert(0) += 1;
        }
        counts
    }

    /// Calculate average transfer confidence
    pub fn avg_transfer_confidence(&self) -> f64 {
        if self.primitives.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.primitives.iter().map(|p| p.transfer_confidence).sum();
        sum / self.primitives.len() as f64
    }

    /// Is this a T2-C factory? (>60% composites)
    pub fn is_factory(&self) -> bool {
        let counts = self.tier_counts();
        let t2c = *counts.get(&Tier::T2Composite).unwrap_or(&0);
        let total = self.primitives.len();
        total > 0 && (t2c as f64 / total as f64) > 0.6
    }

    /// Summary line
    pub fn summary(&self) -> String {
        let counts = self.tier_counts();
        format!(
            "{}: T1={} T2-P={} T2-C={} T3={} (depth={}, transfer={:.0}%)",
            self.module,
            counts.get(&Tier::T1Universal).unwrap_or(&0),
            counts.get(&Tier::T2Primitive).unwrap_or(&0),
            counts.get(&Tier::T2Composite).unwrap_or(&0),
            counts.get(&Tier::T3Domain).unwrap_or(&0),
            self.depth,
            self.avg_transfer_confidence() * 100.0
        )
    }
}

/// Registry of all extractions (capabilities catalog)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimitiveRegistry {
    /// All extractions by module name
    pub extractions: HashMap<String, Extraction>,
    /// Global primitive index (name -> modules containing it)
    pub index: HashMap<String, Vec<String>>,
}

impl PrimitiveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an extraction
    pub fn register(&mut self, extraction: Extraction) {
        // Update index
        for p in &extraction.primitives {
            self.index
                .entry(p.name.clone())
                .or_default()
                .push(extraction.module.clone());
        }
        self.extractions
            .insert(extraction.module.clone(), extraction);
    }

    /// Find modules containing a primitive
    pub fn find(&self, primitive: &str) -> Vec<&str> {
        self.index
            .get(primitive)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get all T1 primitives across all modules
    pub fn all_t1(&self) -> Vec<&Primitive> {
        self.extractions
            .values()
            .flat_map(|e| e.primitives.iter())
            .filter(|p| p.tier == Tier::T1Universal)
            .collect()
    }

    /// Get total counts
    pub fn total_counts(&self) -> HashMap<Tier, usize> {
        let mut counts = HashMap::new();
        for e in self.extractions.values() {
            for (tier, count) in e.tier_counts() {
                *counts.entry(tier).or_insert(0) += count;
            }
        }
        counts
    }

    /// Summary report
    pub fn report(&self) -> String {
        let mut lines = vec![
            "# Primitive Registry".to_string(),
            format!("Modules: {}", self.extractions.len()),
            String::new(),
        ];

        for (module, extraction) in &self.extractions {
            lines.push(format!("## {}", module));
            lines.push(extraction.summary());
            lines.push(String::new());
        }

        let counts = self.total_counts();
        lines.push("## Totals".to_string());
        lines.push(format!(
            "T1={} T2-P={} T2-C={} T3={}",
            counts.get(&Tier::T1Universal).unwrap_or(&0),
            counts.get(&Tier::T2Primitive).unwrap_or(&0),
            counts.get(&Tier::T2Composite).unwrap_or(&0),
            counts.get(&Tier::T3Domain).unwrap_or(&0),
        ));

        lines.join("\n")
    }

    /// Load from JSON file
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| std::io::Error::other(e.to_string()))
    }

    /// Save to JSON file
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let content =
            serde_json::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(path, content)
    }
}

/// Quick extraction helper for common patterns
pub fn extract_from_module(module: &str, terms: &[(&str, &str, Tier)]) -> Extraction {
    let mut extraction = Extraction::new(module);

    for (name, definition, tier) in terms {
        let primitive = match tier {
            Tier::T1Universal => Primitive::t1(name, definition),
            Tier::T2Primitive => Primitive::t2p(name, definition, &[]),
            Tier::T2Composite => Primitive::t2c(name, definition, &[]),
            Tier::T3Domain => Primitive::t3(name, definition, module),
        };
        extraction.add(primitive);
    }

    // Calculate depth from dependencies
    extraction.depth = calculate_depth(&extraction.dependencies);
    extraction
}

/// Calculate dependency graph depth
fn calculate_depth(deps: &HashMap<String, Vec<String>>) -> usize {
    if deps.is_empty() {
        return 0;
    }

    fn dfs(
        node: &str,
        deps: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, usize>,
    ) -> usize {
        if let Some(&d) = visited.get(node) {
            return d;
        }

        let depth = deps
            .get(node)
            .map(|children| {
                children
                    .iter()
                    .map(|c| dfs(c, deps, visited) + 1)
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);

        visited.insert(node.to_string(), depth);
        depth
    }

    let mut visited = HashMap::new();
    deps.keys()
        .map(|k| dfs(k, deps, &mut visited))
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_creation() {
        let p = Primitive::t1("sequence", "Ordered operations").with_rust("Vec<T>, Iterator");

        assert_eq!(p.tier, Tier::T1Universal);
        assert!(p.is_atomic());
        assert_eq!(p.transfer_confidence, 1.0);
    }

    #[test]
    fn test_composite_creation() {
        let p = Primitive::t2c("Valence", "I/O port for bonding", &["direction", "type"])
            .with_rust("struct Valence");

        assert_eq!(p.tier, Tier::T2Composite);
        assert!(!p.is_atomic());
        assert_eq!(p.components.len(), 2);
    }

    #[test]
    fn test_extraction() {
        let mut extraction = Extraction::new("bonding");
        extraction.add(Primitive::t1("sequence", "Ordered ops"));
        extraction.add(Primitive::t2c("Valence", "Port", &["direction"]));
        extraction.add(Primitive::t3("HookAtom", "Hook unit", "hooks"));

        let counts = extraction.tier_counts();
        assert_eq!(counts.get(&Tier::T1Universal), Some(&1));
        assert_eq!(counts.get(&Tier::T2Composite), Some(&1));
        assert_eq!(counts.get(&Tier::T3Domain), Some(&1));
    }

    #[test]
    fn test_registry() {
        let mut registry = PrimitiveRegistry::new();

        let mut e1 = Extraction::new("bonding");
        e1.add(Primitive::t1("sequence", "Order"));

        let mut e2 = Extraction::new("experiment");
        e2.add(Primitive::t1("sequence", "Order"));
        e2.add(Primitive::t2p(
            "threshold",
            "Boundary",
            &["math", "physics"],
        ));

        registry.register(e1);
        registry.register(e2);

        // sequence appears in both modules
        let modules = registry.find("sequence");
        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_factory_detection() {
        let mut extraction = Extraction::new("factory_module");
        // Add 7 T2-C and 3 others = 70% T2-C
        for i in 0..7 {
            extraction.add(Primitive::t2c(&format!("comp{}", i), "Composite", &[]));
        }
        extraction.add(Primitive::t1("seq", "Seq"));
        extraction.add(Primitive::t2p("dir", "Dir", &[]));
        extraction.add(Primitive::t3("dom", "Dom", "x"));

        assert!(extraction.is_factory());
    }

    #[test]
    fn test_depth_calculation() {
        let mut deps = HashMap::new();
        deps.insert("A".to_string(), vec!["B".to_string()]);
        deps.insert("B".to_string(), vec!["C".to_string()]);
        deps.insert("C".to_string(), vec![]);

        assert_eq!(calculate_depth(&deps), 2);
    }
}
