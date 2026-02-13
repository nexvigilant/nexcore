// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # PathResolver
//!
//! **Tier**: T2-C (lambda + sigma + exists)
//! **Dominant**: lambda (Location)
//!
//! Hierarchical path resolution with namespace segmentation.
//! Resolves dot-separated paths like "system.auth.login" to registered values.

use core::fmt;
use std::collections::BTreeMap;

/// A segment in a hierarchical path.
type Segment = String;

/// A node in the path tree.
#[derive(Debug, Clone)]
struct PathNode<V> {
    /// Value stored at this node (if any).
    value: Option<V>,
    /// Children by segment name.
    children: BTreeMap<Segment, PathNode<V>>,
}

impl<V> PathNode<V> {
    fn new() -> Self {
        Self {
            value: None,
            children: BTreeMap::new(),
        }
    }
}

/// Hierarchical path resolver with dot-notation addressing.
///
/// ## Tier: T2-C (lambda + sigma + exists)
/// Dominant: lambda (Location)
///
/// Supports:
/// - Exact path lookup: "system.auth.login"
/// - Prefix listing: "system.auth.*"
/// - Ancestor resolution: walks up the tree for fallback values
#[derive(Debug, Clone)]
pub struct PathResolver<V> {
    /// Root of the path tree.
    root: PathNode<V>,
    /// Separator character.
    separator: char,
    /// Total registered paths.
    count: usize,
}

impl<V: Clone + fmt::Debug> PathResolver<V> {
    /// Create a new path resolver with default '.' separator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: PathNode::new(),
            separator: '.',
            count: 0,
        }
    }

    /// Create with a custom separator.
    #[must_use]
    pub fn with_separator(separator: char) -> Self {
        Self {
            root: PathNode::new(),
            separator,
            count: 0,
        }
    }

    /// Register a value at a path.
    pub fn register(&mut self, path: &str, value: V) {
        let segments: Vec<&str> = path.split(self.separator).collect();
        let mut node = &mut self.root;

        for seg in &segments {
            node = node
                .children
                .entry((*seg).to_string())
                .or_insert_with(PathNode::new);
        }

        if node.value.is_none() {
            self.count += 1;
        }
        node.value = Some(value);
    }

    /// Resolve a path to its value (exact match).
    #[must_use]
    pub fn resolve(&self, path: &str) -> Option<&V> {
        let segments: Vec<&str> = path.split(self.separator).collect();
        let mut node = &self.root;

        for seg in &segments {
            match node.children.get(*seg) {
                Some(child) => node = child,
                None => return None,
            }
        }

        node.value.as_ref()
    }

    /// Resolve with ancestor fallback: walks up the tree.
    ///
    /// For path "a.b.c", tries "a.b.c", then "a.b", then "a", then root.
    #[must_use]
    pub fn resolve_with_fallback(&self, path: &str) -> Option<&V> {
        let segments: Vec<&str> = path.split(self.separator).collect();
        let mut node = &self.root;
        let mut last_value = node.value.as_ref();

        for seg in &segments {
            match node.children.get(*seg) {
                Some(child) => {
                    node = child;
                    if node.value.is_some() {
                        last_value = node.value.as_ref();
                    }
                }
                None => break,
            }
        }

        last_value
    }

    /// List all paths under a prefix.
    #[must_use]
    pub fn list_children(&self, prefix: &str) -> Vec<String> {
        let segments: Vec<&str> = if prefix.is_empty() {
            Vec::new()
        } else {
            prefix.split(self.separator).collect()
        };

        let mut node = &self.root;
        for seg in &segments {
            match node.children.get(*seg) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }

        // Collect all leaf paths under this node
        let mut results = Vec::new();
        self.collect_paths(node, prefix, &mut results);
        results
    }

    /// Check if a path exists.
    #[must_use]
    pub fn exists(&self, path: &str) -> bool {
        self.resolve(path).is_some()
    }

    /// Total registered paths.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the resolver is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Recursively collect all paths with values.
    fn collect_paths(&self, node: &PathNode<V>, prefix: &str, results: &mut Vec<String>) {
        for (seg, child) in &node.children {
            let path = if prefix.is_empty() {
                seg.clone()
            } else {
                format!("{prefix}{}{seg}", self.separator)
            };

            if child.value.is_some() {
                results.push(path.clone());
            }
            self.collect_paths(child, &path, results);
        }
    }
}

impl<V: Clone + fmt::Debug> Default for PathResolver<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve() {
        let mut resolver = PathResolver::new();
        resolver.register("system.auth.login", 42);
        resolver.register("system.auth.logout", 43);
        resolver.register("system.db.connect", 100);

        assert_eq!(resolver.resolve("system.auth.login"), Some(&42));
        assert_eq!(resolver.resolve("system.auth.logout"), Some(&43));
        assert_eq!(resolver.resolve("system.db.connect"), Some(&100));
        assert_eq!(resolver.resolve("system.auth.missing"), None);
    }

    #[test]
    fn test_fallback_resolution() {
        let mut resolver = PathResolver::new();
        resolver.register("system", "default");
        resolver.register("system.auth", "auth-default");
        resolver.register("system.auth.login", "login-specific");

        // Exact match
        assert_eq!(
            resolver.resolve_with_fallback("system.auth.login"),
            Some(&"login-specific")
        );
        // Falls back to "system.auth"
        assert_eq!(
            resolver.resolve_with_fallback("system.auth.unknown"),
            Some(&"auth-default")
        );
        // Falls back to "system"
        assert_eq!(
            resolver.resolve_with_fallback("system.unknown.deep"),
            Some(&"default")
        );
    }

    #[test]
    fn test_list_children() {
        let mut resolver = PathResolver::new();
        resolver.register("a.b.c", 1);
        resolver.register("a.b.d", 2);
        resolver.register("a.e", 3);

        let children = resolver.list_children("a");
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_custom_separator() {
        let mut resolver = PathResolver::with_separator('/');
        resolver.register("usr/local/bin", "binary");
        assert_eq!(resolver.resolve("usr/local/bin"), Some(&"binary"));
    }
}
