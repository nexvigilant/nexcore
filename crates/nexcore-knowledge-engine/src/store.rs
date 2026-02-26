//! File persistence for knowledge packs.
//!
//! Packs stored at `~/.claude/brain/knowledge_packs/<name>/v<N>/pack.json`.

use nexcore_fs::dirs;
use std::path::{Path, PathBuf};

use crate::error::{KnowledgeEngineError, Result};
use crate::knowledge_pack::{KnowledgePack, PackIndex};

/// Validate pack name to prevent path traversal.
///
/// Names must be non-empty and contain only alphanumeric characters, hyphens, and
/// underscores. Path separators, dots, and other special characters are rejected.
fn validate_pack_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains(['/', '\\', '.', '\0'])
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(KnowledgeEngineError::InvalidPackName(format!(
            "{name:?} — names must match [a-zA-Z0-9_-]+"
        )));
    }
    Ok(())
}

/// File-based knowledge pack store.
#[derive(Debug, Clone)]
pub struct KnowledgeStore {
    root: PathBuf,
}

impl KnowledgeStore {
    /// Create a store at the default location (`~/.claude/brain/knowledge_packs/`).
    pub fn default_location() -> Result<Self> {
        let root = dirs::home_dir()
            .ok_or_else(|| KnowledgeEngineError::Store("No home directory".to_string()))?
            .join(".claude")
            .join("brain")
            .join("knowledge_packs");
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Create a store at a specific path.
    pub fn at(root: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Create a temporary store (for testing).
    pub fn temp() -> Result<Self> {
        let dir = std::env::temp_dir().join(format!("ke-{}", nexcore_id::NexId::v4()));
        std::fs::create_dir_all(&dir)?;
        Ok(Self { root: dir })
    }

    /// Get the root path of this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Save a pack to disk using atomic writes (tmp file + rename).
    ///
    /// Each file is written to a `.tmp` sibling and renamed into place so that
    /// a mid-write crash leaves no partial files. `list_packs` reads `index.json`
    /// to discover packs — if that write fails the pack.json is also rolled back.
    pub fn save_pack(&self, pack: &KnowledgePack) -> Result<PathBuf> {
        validate_pack_name(&pack.name)?;
        let pack_dir = self
            .root
            .join(&pack.name)
            .join(format!("v{}", pack.version));
        std::fs::create_dir_all(&pack_dir)?;

        // Atomic write for pack.json
        let pack_path = pack_dir.join("pack.json");
        let pack_tmp = pack_dir.join("pack.json.tmp");
        let content = serde_json::to_string_pretty(pack)?;
        std::fs::write(&pack_tmp, content)?;
        std::fs::rename(&pack_tmp, &pack_path)?;

        // Atomic write for index.json
        let index_path = pack_dir.join("index.json");
        let index_tmp = pack_dir.join("index.json.tmp");
        let index = pack.to_index();
        let index_content = serde_json::to_string_pretty(&index)?;
        std::fs::write(&index_tmp, index_content)?;
        std::fs::rename(&index_tmp, &index_path)?;

        Ok(pack_path)
    }

    /// Load a pack by name and version.
    pub fn load_pack(&self, name: &str, version: u32) -> Result<KnowledgePack> {
        validate_pack_name(name)?;
        let pack_path = self
            .root
            .join(name)
            .join(format!("v{}", version))
            .join("pack.json");
        if !pack_path.exists() {
            return Err(KnowledgeEngineError::PackNotFound(format!(
                "{}/v{}",
                name, version
            )));
        }
        let content = std::fs::read_to_string(&pack_path)?;
        let pack: KnowledgePack = serde_json::from_str(&content)?;
        Ok(pack)
    }

    /// Load the latest version of a pack by name.
    pub fn load_latest(&self, name: &str) -> Result<KnowledgePack> {
        validate_pack_name(name)?;
        let version = self
            .latest_version(name)
            .ok_or_else(|| KnowledgeEngineError::PackNotFound(name.to_string()))?;
        self.load_pack(name, version)
    }

    /// Get the latest version number for a pack name.
    ///
    /// Returns `None` if validation fails or no versions exist.
    pub fn latest_version(&self, name: &str) -> Option<u32> {
        if validate_pack_name(name).is_err() {
            return None;
        }
        let pack_dir = self.root.join(name);
        if !pack_dir.exists() {
            return None;
        }

        let mut max_version = None;
        if let Ok(entries) = std::fs::read_dir(&pack_dir) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if let Some(stripped) = dir_name.strip_prefix('v') {
                    if let Ok(v) = stripped.parse::<u32>() {
                        max_version = Some(max_version.map_or(v, |prev: u32| prev.max(v)));
                    }
                }
            }
        }
        max_version
    }

    /// Delete all versions of a pack. Returns the number of versions removed.
    pub fn delete_pack(&self, name: &str) -> Result<usize> {
        validate_pack_name(name)?;
        let pack_dir = self.root.join(name);
        if !pack_dir.exists() {
            return Err(KnowledgeEngineError::PackNotFound(name.to_string()));
        }
        let count = self.version_count(name);
        std::fs::remove_dir_all(&pack_dir)?;
        Ok(count)
    }

    /// Delete a specific version of a pack.
    pub fn delete_version(&self, name: &str, version: u32) -> Result<()> {
        validate_pack_name(name)?;
        let version_dir = self.root.join(name).join(format!("v{version}"));
        if !version_dir.exists() {
            return Err(KnowledgeEngineError::PackNotFound(format!(
                "{name}/v{version}"
            )));
        }
        std::fs::remove_dir_all(&version_dir)?;

        // If no versions remain, remove the empty pack directory
        let pack_dir = self.root.join(name);
        if pack_dir.exists() && self.version_count(name) == 0 {
            std::fs::remove_dir(&pack_dir)?;
        }
        Ok(())
    }

    /// Keep only the `keep` most recent versions of a pack. Returns the number pruned.
    ///
    /// If fewer than `keep` versions exist, nothing is removed.
    pub fn prune_old_versions(&self, name: &str, keep: usize) -> Result<usize> {
        validate_pack_name(name)?;
        let pack_dir = self.root.join(name);
        if !pack_dir.exists() {
            return Err(KnowledgeEngineError::PackNotFound(name.to_string()));
        }

        let mut versions = self.list_versions(name);
        if versions.len() <= keep {
            return Ok(0);
        }

        // Sort descending — keep the highest N
        versions.sort_unstable_by(|a, b| b.cmp(a));
        let to_remove = &versions[keep..];
        let mut removed = 0_usize;

        for &v in to_remove {
            let version_dir = pack_dir.join(format!("v{v}"));
            if version_dir.exists() {
                std::fs::remove_dir_all(&version_dir)?;
                removed = removed.saturating_add(1);
            }
        }

        Ok(removed)
    }

    /// List all version numbers for a pack.
    fn list_versions(&self, name: &str) -> Vec<u32> {
        let pack_dir = self.root.join(name);
        let mut versions = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&pack_dir) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if let Some(stripped) = dir_name.strip_prefix('v') {
                    if let Ok(v) = stripped.parse::<u32>() {
                        versions.push(v);
                    }
                }
            }
        }
        versions
    }

    /// Count the number of versions for a pack.
    fn version_count(&self, name: &str) -> usize {
        self.list_versions(name).len()
    }

    /// Find a fragment by ID across all latest pack versions.
    ///
    /// Returns the fragment along with the pack name it belongs to.
    /// Searches packs in creation-order (newest first).
    pub fn find_fragment(
        &self,
        fragment_id: &str,
    ) -> Result<(String, crate::ingest::KnowledgeFragment)> {
        let indices = self.list_packs()?;
        for idx in &indices {
            if let Ok(pack) = self.load_latest(&idx.name) {
                if let Ok(frag) = pack.get_fragment(fragment_id) {
                    return Ok((pack.name.clone(), frag.clone()));
                }
            }
        }
        Err(KnowledgeEngineError::FragmentNotFound(
            fragment_id.to_string(),
        ))
    }

    /// List all pack indices.
    pub fn list_packs(&self) -> Result<Vec<PackIndex>> {
        let mut indices = Vec::new();

        if !self.root.exists() {
            return Ok(indices);
        }

        let entries = std::fs::read_dir(&self.root)?;
        for entry in entries.flatten() {
            let pack_name = entry.file_name().to_string_lossy().to_string();
            if let Some(version) = self.latest_version(&pack_name) {
                let index_path = self
                    .root
                    .join(&pack_name)
                    .join(format!("v{}", version))
                    .join("index.json");
                if let Ok(content) = std::fs::read_to_string(&index_path) {
                    if let Ok(index) = serde_json::from_str::<PackIndex>(&content) {
                        indices.push(index);
                    }
                }
            }
        }

        indices.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::concept_graph::ConceptGraph;

    #[test]
    fn save_and_load() {
        let store = KnowledgeStore::temp().unwrap();
        let pack = KnowledgePack::new("test".to_string(), 1, vec![], ConceptGraph::new());

        let path = store.save_pack(&pack).unwrap();
        assert!(path.exists());

        let loaded = store.load_pack("test", 1).unwrap();
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn latest_version_tracking() {
        let store = KnowledgeStore::temp().unwrap();

        let pack1 = KnowledgePack::new("mypack".to_string(), 1, vec![], ConceptGraph::new());
        store.save_pack(&pack1).unwrap();

        let pack2 = KnowledgePack::new("mypack".to_string(), 2, vec![], ConceptGraph::new());
        store.save_pack(&pack2).unwrap();

        assert_eq!(store.latest_version("mypack"), Some(2));
    }

    #[test]
    fn list_packs() {
        let store = KnowledgeStore::temp().unwrap();

        let pack = KnowledgePack::new("alpha".to_string(), 1, vec![], ConceptGraph::new());
        store.save_pack(&pack).unwrap();

        let indices = store.list_packs().unwrap();
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0].name, "alpha");
    }

    #[test]
    fn load_nonexistent_fails() {
        let store = KnowledgeStore::temp().unwrap();
        assert!(store.load_pack("nonexistent", 1).is_err());
    }

    #[test]
    fn path_traversal_rejected() {
        let store = KnowledgeStore::temp().unwrap();
        let pack = KnowledgePack::new(
            "../../../tmp/evil".to_string(),
            1,
            vec![],
            ConceptGraph::new(),
        );
        let err = store.save_pack(&pack).unwrap_err();
        assert!(
            matches!(err, KnowledgeEngineError::InvalidPackName(_)),
            "path traversal should be rejected: {err}"
        );
    }

    #[test]
    fn dot_in_name_rejected() {
        let store = KnowledgeStore::temp().unwrap();
        assert!(store.load_pack("my.pack", 1).is_err());
    }

    #[test]
    fn valid_names_accepted() {
        assert!(validate_pack_name("my-pack").is_ok());
        assert!(validate_pack_name("my_pack_v2").is_ok());
        assert!(validate_pack_name("TestPack123").is_ok());
    }

    #[test]
    fn delete_pack_removes_all_versions() {
        let store = KnowledgeStore::temp().unwrap();
        let p1 = KnowledgePack::new("to-delete".to_string(), 1, vec![], ConceptGraph::new());
        let p2 = KnowledgePack::new("to-delete".to_string(), 2, vec![], ConceptGraph::new());
        store.save_pack(&p1).unwrap();
        store.save_pack(&p2).unwrap();

        let removed = store.delete_pack("to-delete").unwrap();
        assert_eq!(removed, 2);
        assert!(store.load_pack("to-delete", 1).is_err());
        assert!(store.load_pack("to-delete", 2).is_err());
    }

    #[test]
    fn delete_pack_nonexistent_fails() {
        let store = KnowledgeStore::temp().unwrap();
        assert!(store.delete_pack("ghost").is_err());
    }

    #[test]
    fn delete_version_removes_single() {
        let store = KnowledgeStore::temp().unwrap();
        let p1 = KnowledgePack::new("versioned".to_string(), 1, vec![], ConceptGraph::new());
        let p2 = KnowledgePack::new("versioned".to_string(), 2, vec![], ConceptGraph::new());
        store.save_pack(&p1).unwrap();
        store.save_pack(&p2).unwrap();

        store.delete_version("versioned", 1).unwrap();
        assert!(store.load_pack("versioned", 1).is_err());
        // v2 still exists
        assert!(store.load_pack("versioned", 2).is_ok());
    }

    #[test]
    fn delete_last_version_cleans_up_directory() {
        let store = KnowledgeStore::temp().unwrap();
        let p1 = KnowledgePack::new("cleanup".to_string(), 1, vec![], ConceptGraph::new());
        store.save_pack(&p1).unwrap();

        store.delete_version("cleanup", 1).unwrap();
        // Pack directory should be gone
        assert!(!store.root().join("cleanup").exists());
    }

    #[test]
    fn prune_keeps_newest_versions() {
        let store = KnowledgeStore::temp().unwrap();
        for v in 1..=5 {
            let p = KnowledgePack::new("prunable".to_string(), v, vec![], ConceptGraph::new());
            store.save_pack(&p).unwrap();
        }

        let pruned = store.prune_old_versions("prunable", 2).unwrap();
        assert_eq!(pruned, 3);
        // v4 and v5 survive
        assert!(store.load_pack("prunable", 4).is_ok());
        assert!(store.load_pack("prunable", 5).is_ok());
        // v1-v3 are gone
        assert!(store.load_pack("prunable", 1).is_err());
        assert!(store.load_pack("prunable", 2).is_err());
        assert!(store.load_pack("prunable", 3).is_err());
    }

    #[test]
    fn prune_noop_when_fewer_than_keep() {
        let store = KnowledgeStore::temp().unwrap();
        let p1 = KnowledgePack::new("small".to_string(), 1, vec![], ConceptGraph::new());
        store.save_pack(&p1).unwrap();

        let pruned = store.prune_old_versions("small", 5).unwrap();
        assert_eq!(pruned, 0);
        assert!(store.load_pack("small", 1).is_ok());
    }

    #[test]
    fn find_fragment_across_packs() {
        use crate::compiler::{CompileOptions, KnowledgeCompiler};
        use crate::ingest::{KnowledgeSource, RawKnowledge};
        use nexcore_chrono::DateTime;

        let store = KnowledgeStore::temp().unwrap();
        let compiler = KnowledgeCompiler::new(store.clone());
        let pack = compiler
            .compile(CompileOptions {
                name: "frag-search".to_string(),
                include_distillations: false,
                include_artifacts: false,
                include_implicit: false,
                sources: vec![RawKnowledge {
                    text: "Signal detection uses PRR for pharmacovigilance safety.".to_string(),
                    source: KnowledgeSource::FreeText,
                    domain: Some("pv".to_string()),
                    timestamp: DateTime::now(),
                }],
            })
            .unwrap();

        // Get a fragment ID from the compiled pack
        let frag_id = pack.fragments[0].id.clone();
        let (pack_name, found) = store.find_fragment(&frag_id).unwrap();
        assert_eq!(pack_name, "frag-search");
        assert_eq!(found.id, frag_id);
    }

    #[test]
    fn find_fragment_not_found() {
        let store = KnowledgeStore::temp().unwrap();
        assert!(store.find_fragment("nonexistent").is_err());
    }
}
