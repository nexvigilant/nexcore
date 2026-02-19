use burn::module::Module;
use burn::record::{DefaultFileRecorder, FullPrecisionSettings};
use burn::tensor::backend::Backend;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Training checkpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    pub epoch: usize,
    pub batch: usize,
    pub global_step: usize,
    pub loss: f32,
    pub accuracy: f32,
    pub learning_rate: f32,
}

impl CheckpointMetadata {
    pub fn new(
        epoch: usize,
        batch: usize,
        global_step: usize,
        loss: f32,
        accuracy: f32,
        learning_rate: f32,
    ) -> Self {
        Self {
            epoch,
            batch,
            global_step,
            loss,
            accuracy,
            learning_rate,
        }
    }
}

/// Checkpoint manager for saving/loading training state and model weights
pub struct CheckpointManager {
    checkpoint_dir: String,
}

impl CheckpointManager {
    pub fn new(checkpoint_dir: &str) -> Self {
        let _ = fs::create_dir_all(checkpoint_dir);
        Self {
            checkpoint_dir: checkpoint_dir.to_string(),
        }
    }

    /// Save checkpoint metadata as JSON
    pub fn save_metadata(
        &self,
        metadata: &CheckpointMetadata,
        name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = format!("{}/{}.json", self.checkpoint_dir, name);
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(&path, json)?;
        println!("Checkpoint metadata saved: {}", path);
        Ok(())
    }

    /// Load checkpoint metadata from JSON
    pub fn load_metadata(
        &self,
        name: &str,
    ) -> Result<CheckpointMetadata, Box<dyn std::error::Error>> {
        let path = format!("{}/{}.json", self.checkpoint_dir, name);
        if !Path::new(&path).exists() {
            return Err(format!("Checkpoint not found: {}", path).into());
        }
        let json = fs::read_to_string(&path)?;
        let metadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    /// Save model weights using Burn's NamedMpk recorder
    pub fn save_model<B, M>(&self, model: M, name: &str) -> Result<(), Box<dyn std::error::Error>>
    where
        B: Backend,
        M: Module<B>,
    {
        let recorder = DefaultFileRecorder::<FullPrecisionSettings>::new();
        let path = format!("{}/{}_model", self.checkpoint_dir, name);
        model
            .save_file(path.clone(), &recorder)
            .map_err(|e| format!("Failed to save model: {}", e))?;
        println!("Model weights saved: {}.mpk", path);
        Ok(())
    }

    /// Load model weights using Burn's NamedMpk recorder
    pub fn load_model<B, M>(
        &self,
        model: M,
        name: &str,
        device: &B::Device,
    ) -> Result<M, Box<dyn std::error::Error>>
    where
        B: Backend,
        M: Module<B>,
    {
        let recorder = DefaultFileRecorder::<FullPrecisionSettings>::new();
        let path = format!("{}/{}_model", self.checkpoint_dir, name);
        let model = model
            .load_file(path, &recorder, device)
            .map_err(|e| format!("Failed to load model: {}", e))?;
        println!("Model weights loaded");
        Ok(model)
    }

    /// Get list of available checkpoints (by metadata JSON files)
    pub fn list_checkpoints(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut checkpoints = Vec::new();
        for entry in fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(name) = path.file_stem() {
                    checkpoints.push(name.to_string_lossy().to_string());
                }
            }
        }
        checkpoints.sort();
        Ok(checkpoints)
    }

    /// Get latest checkpoint name
    pub fn latest_checkpoint(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let checkpoints = self.list_checkpoints()?;
        Ok(checkpoints.last().cloned())
    }
}
