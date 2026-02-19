//! File persistence for knowledge packs.
//!
//! Packs stored at `~/.claude/brain/knowledge_packs/<name>/v<N>/pack.json`.

use std::path::{Path, PathBuf};

use crate::error::{KnowledgeEngineError, Result};
use crate::knowledge_pack::{KnowledgePack, PackIndex};

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
        let dir = std::env::temp_dir().join(format!("ke-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir)?;
        Ok(Self { root: dir })
    }

    /// Get the root path of this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Save a pack to disk.
    pub fn save_pack(&self, pack: &KnowledgePack) -> Result<PathBuf> {
        let pack_dir = self
            .root
            .join(&pack.name)
            .join(format!("v{}", pack.version));
        std::fs::create_dir_all(&pack_dir)?;

        let pack_path = pack_dir.join("pack.json");
        let content = serde_json::to_string_pretty(pack)?;
        std::fs::write(&pack_path, content)?;

        // Also write index entry
        let index_path = pack_dir.join("index.json");
        let index = pack.to_index();
        let index_content = serde_json::to_string_pretty(&index)?;
        std::fs::write(&index_path, index_content)?;

        Ok(pack_path)
    }

    /// Load a pack by name and version.
    pub fn load_pack(&self, name: &str, version: u32) -> Result<KnowledgePack> {
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
        let version = self
            .latest_version(name)
            .ok_or_else(|| KnowledgeEngineError::PackNotFound(name.to_string()))?;
        self.load_pack(name, version)
    }

    /// Get the latest version number for a pack name.
    pub fn latest_version(&self, name: &str) -> Option<u32> {
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
}
