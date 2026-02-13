//! SessionStart Debug Manager Hook
//!
//! Autonomous debug log cleanup and temporal renaming.
//!
//! # Event
//! SessionStart
//!
//! # Actions
//! 1. Rename UUID files → `YYYY-MM-DDT{HH-MM-SS}_{UUID-prefix}.txt`
//! 2. Delete files older than 7 days
//! 3. Cap total directory size at 500 MB (oldest first)
//! 4. Report summary to stderr
//!
//! # Exit Codes
//! - 0: Always (best-effort, never blocks)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MAX_AGE_DAYS: u64 = 7;
const MAX_DIR_SIZE_BYTES: u64 = 500 * 1024 * 1024;

struct Stats {
    renamed: u32,
    deleted: u32,
    deleted_bytes: u64,
}

fn main() {
    let debug_dir = debug_dir_path();
    if !debug_dir.is_dir() {
        std::process::exit(0);
    }

    let mut stats = Stats {
        renamed: 0,
        deleted: 0,
        deleted_bytes: 0,
    };

    rename_uuid_files(&debug_dir, &mut stats);
    delete_old_files(&debug_dir, &mut stats);
    enforce_size_cap(&debug_dir, &mut stats);
    report(&debug_dir, &stats);

    std::process::exit(0);
}

fn debug_dir_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("debug")
}

fn list_txt_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "txt"))
        .filter(|p| !is_index_file(p))
        .collect()
}

fn rename_uuid_files(dir: &Path, stats: &mut Stats) {
    for path in list_txt_files(dir) {
        if !is_uuid_debug_file(&path) {
            continue;
        }
        let Some(new_name) = temporal_name_for(&path) else {
            continue;
        };
        let new_path = dir.join(&new_name);
        if new_path.exists() {
            continue;
        }
        if fs::rename(&path, &new_path).is_ok() {
            stats.renamed += 1;
        }
    }
}

fn delete_old_files(dir: &Path, stats: &mut Stats) {
    let now = SystemTime::now();
    let max_age = Duration::from_secs(MAX_AGE_DAYS * 86400);

    for path in list_txt_files(dir) {
        let age = file_age(&path, now);
        if age <= max_age {
            continue;
        }
        delete_and_track(&path, stats);
    }
}

fn enforce_size_cap(dir: &Path, stats: &mut Stats) {
    let mut files: Vec<(PathBuf, u64, SystemTime)> = list_txt_files(dir)
        .into_iter()
        .filter_map(|p| {
            let meta = fs::metadata(&p).ok()?;
            let modified = meta.modified().unwrap_or(UNIX_EPOCH);
            Some((p, meta.len(), modified))
        })
        .collect();

    let total_size: u64 = files.iter().map(|f| f.1).sum();
    if total_size <= MAX_DIR_SIZE_BYTES {
        return;
    }

    files.sort_by_key(|f| f.2); // oldest first

    let mut remaining = total_size;
    for (path, size, _) in &files {
        if remaining <= MAX_DIR_SIZE_BYTES {
            break;
        }
        if fs::remove_file(path).is_ok() {
            remaining -= size;
            stats.deleted += 1;
            stats.deleted_bytes += size;
        }
    }
}

fn report(dir: &Path, stats: &Stats) {
    if stats.renamed == 0 && stats.deleted == 0 {
        return;
    }
    let remaining = fs::read_dir(dir)
        .map(|e| e.flatten().count())
        .unwrap_or(0);
    let mb = stats.deleted_bytes as f64 / (1024.0 * 1024.0);
    eprintln!(
        "[debug-manager] Renamed {}, cleaned {} files ({:.1} MB), {} remaining",
        stats.renamed, stats.deleted, mb, remaining
    );
}

fn is_uuid_debug_file(path: &Path) -> bool {
    let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() == 5
        && [8, 4, 4, 4, 12]
            .iter()
            .zip(&parts)
            .all(|(len, part)| part.len() == *len && part.chars().all(|c| c.is_ascii_hexdigit()))
}

fn temporal_name_for(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let first_line = content.lines().next()?;
    let ts = first_line.get(..19)?;

    if !ts.starts_with("20") || ts.as_bytes().get(4) != Some(&b'-') {
        return None;
    }

    let safe_ts: String = ts.chars().map(|c| if c == ':' { '-' } else { c }).collect();
    let stem = path.file_stem()?.to_str()?;
    let prefix = &stem[..stem.len().min(8)];

    Some(format!("{safe_ts}_{prefix}.txt"))
}

fn is_index_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name == "error_index.jsonl" || name == "error_trends.json"
}

fn file_age(path: &Path, now: SystemTime) -> Duration {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|m| now.duration_since(m).ok())
        .unwrap_or_default()
}

fn delete_and_track(path: &Path, stats: &mut Stats) {
    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if fs::remove_file(path).is_ok() {
        stats.deleted += 1;
        stats.deleted_bytes += size;
    }
}
