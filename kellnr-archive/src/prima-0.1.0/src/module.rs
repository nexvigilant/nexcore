// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Module System
//!
//! Import/export and module resolution for Prima programs.
//!
//! ## Philosophy
//!
//! Modules are λ (Location) — named reference points in the code namespace.
//! Imports check ∃ (Existence) — does the module exist?
//! Exports are Σ (Sum) — which items are publicly visible?
//!
//! ## Tier: T2-C (λ + ∃ + Σ + μ)
//!
//! ## Syntax
//!
//! ```prima
//! // Import
//! use std::math           // Import module
//! use std::math::{sin, cos}  // Import specific items
//! use std::math::*        // Import all exports
//!
//! // Export (at module level)
//! pub fn public_fn() { }  // Exported
//! fn private_fn() { }     // Not exported
//! ```

use crate::ast::{Program, Stmt};
use crate::error::{PrimaError, PrimaResult};
use lex_primitiva::prelude::{LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// ═══════════════════════════════════════════════════════════════════════════
// MODULE PATH — λ (Location in namespace)
// ═══════════════════════════════════════════════════════════════════════════

/// A module path like `std::math::trig`.
///
/// ## Tier: T2-P (λ + σ)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulePath {
    /// Path segments.
    pub segments: Vec<String>,
}

impl ModulePath {
    /// Create from segments.
    #[must_use]
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Parse from string like "std::math::trig".
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let segments = s.split("::").map(String::from).collect();
        Self { segments }
    }

    /// Get the module name (last segment).
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }

    /// Get parent path.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            None
        } else {
            Some(Self {
                segments: self.segments[..self.segments.len() - 1].to_vec(),
            })
        }
    }

    /// Join with another segment.
    #[must_use]
    pub fn join(&self, segment: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.to_string());
        Self { segments }
    }

    /// Check if this is a root module.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.segments.len() == 1
    }

    /// Convert to file path.
    #[must_use]
    pub fn to_file_path(&self, base: &Path) -> PathBuf {
        let mut path = base.to_path_buf();
        for segment in &self.segments {
            path = path.join(segment);
        }
        path.with_extension("true")
    }
}

impl std::fmt::Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.segments.join("::"))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// VISIBILITY — Σ (Sum: public | private)
// ═══════════════════════════════════════════════════════════════════════════

/// Item visibility.
///
/// ## Tier: T1 (Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Visibility {
    /// Public — exported from module.
    Public,
    /// Private — internal to module.
    #[default]
    Private,
}

impl Visibility {
    /// Check if publicly visible.
    #[must_use]
    pub const fn is_public(self) -> bool {
        matches!(self, Self::Public)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MODULE ITEM — ς (State: what's in the module)
// ═══════════════════════════════════════════════════════════════════════════

/// An item exported from a module.
///
/// ## Tier: T2-P (ς + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleItem {
    /// Function definition.
    Function {
        name: String,
        visibility: Visibility,
    },
    /// Type definition.
    Type {
        name: String,
        visibility: Visibility,
    },
    /// Constant value.
    Constant {
        name: String,
        visibility: Visibility,
    },
    /// Submodule.
    Module {
        name: String,
        visibility: Visibility,
    },
}

impl ModuleItem {
    /// Get item name.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Function { name, .. }
            | Self::Type { name, .. }
            | Self::Constant { name, .. }
            | Self::Module { name, .. } => name,
        }
    }

    /// Get visibility.
    #[must_use]
    pub fn visibility(&self) -> Visibility {
        match self {
            Self::Function { visibility, .. }
            | Self::Type { visibility, .. }
            | Self::Constant { visibility, .. }
            | Self::Module { visibility, .. } => *visibility,
        }
    }

    /// Check if exported.
    #[must_use]
    pub fn is_exported(&self) -> bool {
        self.visibility().is_public()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IMPORT — ∃ (Existence check + resolution)
// ═══════════════════════════════════════════════════════════════════════════

/// An import statement.
///
/// ## Tier: T2-C (λ + ∃ + Σ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Import {
    /// Import entire module: `use std::math`
    Module { path: ModulePath },

    /// Import specific items: `use std::math::{sin, cos}`
    Items {
        path: ModulePath,
        items: Vec<String>,
    },

    /// Import all exports: `use std::math::*`
    Glob { path: ModulePath },

    /// Import with alias: `use std::math as m`
    Alias { path: ModulePath, alias: String },
}

impl Import {
    /// Get the module path being imported.
    #[must_use]
    pub fn path(&self) -> &ModulePath {
        match self {
            Self::Module { path }
            | Self::Items { path, .. }
            | Self::Glob { path }
            | Self::Alias { path, .. } => path,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MODULE — μ (Mapping: names → items)
// ═══════════════════════════════════════════════════════════════════════════

/// A compiled module.
///
/// ## Tier: T2-C (μ + σ + λ)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Module {
    /// Module path.
    pub path: ModulePath,
    /// Items in this module.
    pub items: HashMap<String, ModuleItem>,
    /// Imports.
    pub imports: Vec<Import>,
    /// Submodules.
    pub submodules: HashMap<String, ModulePath>,
}

impl Module {
    /// Create a new empty module.
    #[must_use]
    pub fn new(path: ModulePath) -> Self {
        Self {
            path,
            items: HashMap::new(),
            imports: Vec::new(),
            submodules: HashMap::new(),
        }
    }

    /// Add an item.
    pub fn add_item(&mut self, item: ModuleItem) {
        self.items.insert(item.name().to_string(), item);
    }

    /// Add an import.
    pub fn add_import(&mut self, import: Import) {
        self.imports.push(import);
    }

    /// Add a submodule.
    pub fn add_submodule(&mut self, name: String, path: ModulePath) {
        self.submodules.insert(name, path);
    }

    /// Get exported items.
    #[must_use]
    pub fn exports(&self) -> Vec<&ModuleItem> {
        self.items.values().filter(|i| i.is_exported()).collect()
    }

    /// Get all item names.
    #[must_use]
    pub fn item_names(&self) -> HashSet<&str> {
        self.items.keys().map(String::as_str).collect()
    }

    /// Lookup an item by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ModuleItem> {
        self.items.get(name)
    }

    /// Check if item exists and is accessible.
    #[must_use]
    pub fn can_access(&self, name: &str, from_same_module: bool) -> bool {
        self.items
            .get(name)
            .is_some_and(|item| from_same_module || item.is_exported())
    }
}

impl Default for ModulePath {
    fn default() -> Self {
        Self {
            segments: vec!["main".to_string()],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MODULE RESOLVER — → (Causality: path → module)
// ═══════════════════════════════════════════════════════════════════════════

/// Module resolution and loading.
///
/// ## Tier: T2-C (→ + λ + ∃)
#[derive(Debug, Default)]
pub struct ModuleResolver {
    /// Loaded modules.
    modules: HashMap<ModulePath, Module>,
    /// Search paths.
    search_paths: Vec<PathBuf>,
}

impl ModuleResolver {
    /// Create a new resolver.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a search path.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Register a module.
    pub fn register(&mut self, module: Module) {
        self.modules.insert(module.path.clone(), module);
    }

    /// Resolve a module path.
    #[must_use]
    pub fn resolve(&self, path: &ModulePath) -> Option<&Module> {
        self.modules.get(path)
    }

    /// Check if a module exists.
    #[must_use]
    pub fn exists(&self, path: &ModulePath) -> bool {
        self.modules.contains_key(path)
    }

    /// Find file for a module path.
    #[must_use]
    pub fn find_file(&self, path: &ModulePath) -> Option<PathBuf> {
        for search_path in &self.search_paths {
            let file_path = path.to_file_path(search_path);
            if file_path.exists() {
                return Some(file_path);
            }
            // Try .prima extension as fallback
            let prima_path = file_path.with_extension("prima");
            if prima_path.exists() {
                return Some(prima_path);
            }
        }
        None
    }

    /// Get all loaded modules.
    #[must_use]
    pub fn modules(&self) -> &HashMap<ModulePath, Module> {
        &self.modules
    }

    /// Resolve an import, returning accessible items.
    pub fn resolve_import(
        &self,
        import: &Import,
        into: &mut HashMap<String, ModulePath>,
    ) -> PrimaResult<()> {
        let module = self
            .resolve(import.path())
            .ok_or_else(|| PrimaError::undefined(import.path().to_string()))?;

        match import {
            Import::Module { path } => {
                // Import module itself
                if let Some(name) = path.name() {
                    into.insert(name.to_string(), path.clone());
                }
            }
            Import::Items { items, .. } => {
                // Import specific items
                for item_name in items {
                    if module.can_access(item_name, false) {
                        into.insert(item_name.clone(), import.path().join(item_name));
                    } else {
                        return Err(PrimaError::undefined(format!(
                            "{}::{}",
                            import.path(),
                            item_name
                        )));
                    }
                }
            }
            Import::Glob { .. } => {
                // Import all exports
                for item in module.exports() {
                    into.insert(item.name().to_string(), import.path().join(item.name()));
                }
            }
            Import::Alias { path, alias } => {
                into.insert(alias.clone(), path.clone());
            }
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MODULE BUILDER — Extract modules from AST
// ═══════════════════════════════════════════════════════════════════════════

/// Build a module from a parsed program.
#[must_use]
pub fn build_module(path: ModulePath, program: &Program) -> Module {
    let mut module = Module::new(path);

    for stmt in &program.statements {
        match stmt {
            Stmt::FnDef { name, .. } => {
                // Check for `pub` prefix (simplified - real impl would be in parser)
                let visibility = if name.starts_with("pub_") {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                module.add_item(ModuleItem::Function {
                    name: name.clone(),
                    visibility,
                });
            }
            Stmt::TypeDef { name, .. } => {
                module.add_item(ModuleItem::Type {
                    name: name.clone(),
                    visibility: Visibility::Private,
                });
            }
            Stmt::Let { name, .. } => {
                module.add_item(ModuleItem::Constant {
                    name: name.clone(),
                    visibility: Visibility::Private,
                });
            }
            _ => {}
        }
    }

    module
}

/// Get primitive composition for module operations.
#[must_use]
pub fn module_composition() -> PrimitiveComposition {
    PrimitiveComposition::new(vec![
        LexPrimitiva::Location,
        LexPrimitiva::Existence,
        LexPrimitiva::Sum,
        LexPrimitiva::Mapping,
    ])
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_path_parse() {
        let path = ModulePath::parse("std::math::trig");
        assert_eq!(path.segments, vec!["std", "math", "trig"]);
        assert_eq!(path.name(), Some("trig"));
    }

    #[test]
    fn test_module_path_display() {
        let path = ModulePath::parse("std::math");
        assert_eq!(format!("{path}"), "std::math");
    }

    #[test]
    fn test_module_path_parent() {
        let path = ModulePath::parse("std::math::trig");
        let parent = path.parent();
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().to_string(), "std::math");
    }

    #[test]
    fn test_module_path_join() {
        let path = ModulePath::parse("std");
        let joined = path.join("math");
        assert_eq!(joined.to_string(), "std::math");
    }

    #[test]
    fn test_module_path_is_root() {
        assert!(ModulePath::parse("main").is_root());
        assert!(!ModulePath::parse("std::math").is_root());
    }

    #[test]
    fn test_visibility() {
        assert!(Visibility::Public.is_public());
        assert!(!Visibility::Private.is_public());
    }

    #[test]
    fn test_module_item_name() {
        let item = ModuleItem::Function {
            name: "foo".into(),
            visibility: Visibility::Public,
        };
        assert_eq!(item.name(), "foo");
        assert!(item.is_exported());
    }

    #[test]
    fn test_module_add_item() {
        let mut module = Module::new(ModulePath::parse("test"));
        module.add_item(ModuleItem::Function {
            name: "foo".into(),
            visibility: Visibility::Public,
        });
        module.add_item(ModuleItem::Function {
            name: "bar".into(),
            visibility: Visibility::Private,
        });

        assert_eq!(module.exports().len(), 1);
        assert!(module.can_access("foo", false));
        assert!(!module.can_access("bar", false));
        assert!(module.can_access("bar", true));
    }

    #[test]
    fn test_import_path() {
        let import = Import::Module {
            path: ModulePath::parse("std::math"),
        };
        assert_eq!(import.path().to_string(), "std::math");
    }

    #[test]
    fn test_module_resolver() {
        let mut resolver = ModuleResolver::new();

        let mut module = Module::new(ModulePath::parse("math"));
        module.add_item(ModuleItem::Function {
            name: "sin".into(),
            visibility: Visibility::Public,
        });
        resolver.register(module);

        assert!(resolver.exists(&ModulePath::parse("math")));
        assert!(!resolver.exists(&ModulePath::parse("unknown")));
    }

    #[test]
    fn test_resolve_import_module() {
        let mut resolver = ModuleResolver::new();
        let module = Module::new(ModulePath::parse("math"));
        resolver.register(module);

        let import = Import::Module {
            path: ModulePath::parse("math"),
        };
        let mut into = HashMap::new();
        let result = resolver.resolve_import(&import, &mut into);

        assert!(result.is_ok());
        assert!(into.contains_key("math"));
    }

    #[test]
    fn test_resolve_import_items() {
        let mut resolver = ModuleResolver::new();
        let mut module = Module::new(ModulePath::parse("math"));
        module.add_item(ModuleItem::Function {
            name: "sin".into(),
            visibility: Visibility::Public,
        });
        module.add_item(ModuleItem::Function {
            name: "cos".into(),
            visibility: Visibility::Public,
        });
        resolver.register(module);

        let import = Import::Items {
            path: ModulePath::parse("math"),
            items: vec!["sin".into()],
        };
        let mut into = HashMap::new();
        let result = resolver.resolve_import(&import, &mut into);

        assert!(result.is_ok());
        assert!(into.contains_key("sin"));
        assert!(!into.contains_key("cos"));
    }

    #[test]
    fn test_resolve_import_glob() {
        let mut resolver = ModuleResolver::new();
        let mut module = Module::new(ModulePath::parse("math"));
        module.add_item(ModuleItem::Function {
            name: "sin".into(),
            visibility: Visibility::Public,
        });
        module.add_item(ModuleItem::Function {
            name: "cos".into(),
            visibility: Visibility::Public,
        });
        module.add_item(ModuleItem::Function {
            name: "internal".into(),
            visibility: Visibility::Private,
        });
        resolver.register(module);

        let import = Import::Glob {
            path: ModulePath::parse("math"),
        };
        let mut into = HashMap::new();
        let result = resolver.resolve_import(&import, &mut into);

        assert!(result.is_ok());
        assert!(into.contains_key("sin"));
        assert!(into.contains_key("cos"));
        assert!(!into.contains_key("internal")); // Private not exported
    }

    #[test]
    fn test_module_to_file_path() {
        let path = ModulePath::parse("std::math");
        let file_path = path.to_file_path(Path::new("/lib"));
        assert_eq!(file_path, PathBuf::from("/lib/std/math.true"));
    }

    #[test]
    fn test_module_composition() {
        let comp = module_composition();
        let unique = comp.unique();
        assert!(unique.contains(&LexPrimitiva::Location));
        assert!(unique.contains(&LexPrimitiva::Existence));
    }
}
