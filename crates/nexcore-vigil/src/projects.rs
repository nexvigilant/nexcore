use nexcore_error::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub priority: u8,
    pub last_active: DateTime<Utc>,
    pub status: ProjectStatus,
    pub focus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStatus {
    Active,
    Stale,
    Blocked,
    Completed,
}

/// Thread-safe, lock-free project registry using DashMap.
pub struct ProjectRegistry {
    projects: Arc<DashMap<String, Project>>,
    data_dir: PathBuf,
}

impl ProjectRegistry {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            projects: Arc::new(DashMap::new()),
            data_dir,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let registry_path = self.data_dir.join("projects.json");
        if registry_path.exists() {
            let content = tokio::fs::read_to_string(&registry_path).await?;
            let map: HashMap<String, Project> = serde_json::from_str(&content)?;
            for (k, v) in map {
                self.projects.insert(k, v);
            }
            info!(count = self.projects.len(), "project_registry_loaded");
        }
        Ok(())
    }

    pub async fn register(&self, project: Project) -> Result<()> {
        debug!(name = %project.name, "registering_project");
        self.projects.insert(project.name.clone(), project);
        self.save().await
    }

    pub async fn update_status(&self, name: &str, status: ProjectStatus) -> Result<()> {
        if let Some(mut project) = self.projects.get_mut(name) {
            project.status = status;
            project.last_active = Utc::now();
            drop(project);
            self.save().await?;
        }
        Ok(())
    }

    /// Atomically saves the registry to disk using a temporary file.
    async fn save(&self) -> Result<()> {
        let data_dir = self.data_dir.clone();
        let projects = self.projects.clone();

        tokio::task::spawn_blocking(move || {
            let temp = tempfile::NamedTempFile::new_in(&data_dir)?;
            serde_json::to_writer_pretty(&temp, &*projects)?;
            temp.persist(data_dir.join("projects.json"))?;
            Ok::<(), nexcore_error::NexError>(())
        })
        .await??;

        Ok(())
    }

    pub fn get_briefing(&self) -> String {
        let mut briefing = String::new();
        let active_count = self
            .projects
            .iter()
            .filter(|p| p.status == ProjectStatus::Active)
            .count();

        if active_count == 0 {
            return "No active projects.".to_string();
        }

        briefing.push_str(&format!("Active projects ({})\n", active_count));
        for entry in self.projects.iter() {
            if entry.status == ProjectStatus::Active {
                briefing.push_str(&format!(
                    "- {}: {}\n",
                    entry.key(),
                    entry.focus.as_deref().unwrap_or("In progress")
                ));
            }
        }
        briefing
    }
}
