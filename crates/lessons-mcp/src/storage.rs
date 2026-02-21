//! Persistent storage for lessons
//! Tier: T2-P (wraps T1 file I/O)

use crate::models::LessonsDb;
use std::fs;
use std::path::PathBuf;

pub fn data_path() -> PathBuf {
    let proj = directories::ProjectDirs::from("dev", "nexvigilant", "lessons-mcp")
        .expect("Could not determine project directories");
    let data_dir = proj.data_dir();
    fs::create_dir_all(data_dir).ok();
    data_dir.join("lessons.json")
}

pub fn load() -> LessonsDb {
    let path = data_path();
    if !path.exists() {
        return LessonsDb::default();
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return LessonsDb::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save(db: &LessonsDb) {
    let path = data_path();
    let Ok(content) = serde_json::to_string_pretty(db) else { return };
    fs::write(path, content).ok();
}
