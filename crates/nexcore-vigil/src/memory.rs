use crate::models::Interaction;
use nexcore_error::{Context, NexError, Result};
use nexcore_fs::walk::WalkDir;
use nexcore_hash::sha256::Sha256;

/// Convert any Display error to NexError.
fn ne(e: impl std::fmt::Display) -> NexError {
    NexError::new(e.to_string())
}
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, GetPointsBuilder, PointStruct, UpsertPointsBuilder,
    VectorParamsBuilder,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

const COLLECTION_NAME: &str = "ksb_knowledge";

/// Manages the persistent memory layer using Qdrant vector database.
#[derive(Clone)]
pub struct MemoryLayer {
    client: Arc<Qdrant>,
    ksb_root: PathBuf,
}

impl MemoryLayer {
    /// Creates a new MemoryLayer instance.
    pub async fn new(ksb_root: PathBuf, _data_dir: PathBuf, qdrant_url: &str) -> Result<Self> {
        let client = Qdrant::from_url(qdrant_url).build().map_err(ne)?;
        Ok(Self {
            client: Arc::new(client),
            ksb_root,
        })
    }

    /// Initializes the Qdrant collection and optionally triggers KSB indexing.
    pub async fn initialize(&self, skip_indexing: bool) -> Result<()> {
        if !self
            .client
            .collection_exists(COLLECTION_NAME)
            .await
            .map_err(ne)?
        {
            info!(collection = %COLLECTION_NAME, "creating_qdrant_collection");
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION_NAME)
                        .vectors_config(VectorParamsBuilder::new(1536, Distance::Cosine)),
                )
                .await
                .map_err(ne)?;
        }
        if !skip_indexing {
            self.index_ksb().await?;
        }
        Ok(())
    }

    /// Indexes the Knowledge Skill Base (KSB) directory recursively.
    /// Uses incremental hashing to skip unchanged files.
    pub async fn index_ksb(&self) -> Result<usize> {
        if !self.ksb_root.exists() {
            error!(path = ?self.ksb_root, "ksb_root_not_found");
            return Ok(0);
        }

        let mut indexed = 0;
        let mut skipped = 0;
        let mut points = Vec::new();

        // 1. Gather all potential files
        let files: Vec<_> = WalkDir::new(&self.ksb_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .collect();

        // 2. Optimization: Perform batch check for existing hashes
        let mut existing_hashes = HashMap::new();
        if !files.is_empty() {
            let ids: Vec<_> = files
                .iter()
                .filter_map(|e| match e.path().strip_prefix(&self.ksb_root) {
                    Ok(rel) => Some(rel.to_string_lossy().to_string().into()),
                    Err(err) => {
                        warn!(error = %err, path = ?e.path(), "ksb_path_prefix_mismatch");
                        None
                    }
                })
                .collect();

            if let Ok(resp) = self
                .client
                .get_points(GetPointsBuilder::new(COLLECTION_NAME, ids).with_payload(true))
                .await
            {
                for point in resp.result {
                    let id_str = match &point.id {
                        Some(id) => match &id.point_id_options {
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(n)) => {
                                n.to_string()
                            }
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(s)) => {
                                s.clone()
                            }
                            None => continue,
                        },
                        None => continue,
                    };

                    if let Some(hash) = point.payload.get("content_hash").and_then(|h| h.as_str()) {
                        existing_hashes.insert(id_str, hash.to_string());
                    }
                }
            }
        }

        // 3. Process files
        for entry in files {
            let path = entry.path().to_owned();
            let content = tokio::fs::read_to_string(&path).await?;
            let hash = self.calculate_hash(&content);
            let file_id = path
                .strip_prefix(&self.ksb_root)
                .map_err(ne)?
                .to_string_lossy()
                .to_string();

            if let Some(existing) = existing_hashes.get(&file_id) {
                if existing == &hash {
                    skipped += 1;
                    continue;
                }
            }

            // Real implementation would call an embedding API here
            let vector = vec![0.0; 1536];
            let mut payload = serde_json::Map::new();
            payload.insert("content_hash".to_string(), serde_json::json!(hash));
            payload.insert("content".to_string(), serde_json::json!(content));
            payload.insert(
                "path".to_string(),
                serde_json::json!(path.to_string_lossy()),
            );

            points.push(PointStruct::new(file_id, vector, payload));
            indexed += 1;

            if points.len() >= 50 {
                self.flush_points(&mut points).await?;
            }
        }

        if !points.is_empty() {
            self.flush_points(&mut points).await?;
        }

        info!(indexed, skipped, "ksb_indexing_summary");
        Ok(indexed)
    }

    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        nexcore_codec::hex::encode(hasher.finalize())
    }

    async fn flush_points(&self, points: &mut Vec<PointStruct>) -> Result<()> {
        self.client
            .upsert_points(
                UpsertPointsBuilder::new(COLLECTION_NAME, std::mem::take(points)).wait(true),
            )
            .await
            .context("failed_to_upsert_points")?;
        Ok(())
    }

    /// Performs a semantic search across indexed knowledge.
    pub async fn search(&self, _query: &str, top_k: usize) -> Result<Vec<serde_json::Value>> {
        let vector = vec![0.0; 1536]; // Real vector from embedding needed
        let results = self
            .client
            .search_points(
                qdrant_client::qdrant::SearchPointsBuilder::new(
                    COLLECTION_NAME,
                    vector,
                    top_k as u64,
                )
                .with_payload(true),
            )
            .await
            .map_err(ne)?;
        Ok(results
            .result
            .into_iter()
            .map(|p| serde_json::to_value(p.payload).unwrap_or(serde_json::Value::Null))
            .collect())
    }

    /// Stores a past interaction in memory for future reference.
    pub async fn remember_interaction(&self, _interaction: Interaction) -> Result<()> {
        // Future: Store interaction in a dedicated collection
        Ok(())
    }

    /// Returns memory layer diagnostics.
    pub fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({ "ksb_root": self.ksb_root, "status": "active" })
    }
}
