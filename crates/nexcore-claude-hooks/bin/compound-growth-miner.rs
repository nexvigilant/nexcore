//! Compound Growth Miner - Stop Hook
//!
//! Triggers on session end to analyze session activity and prompt the next
//! session to extract primitives from this session's work. Turns every session
//! into a primitive mining opportunity, feeding the compound growth tracker.
//!
//! T1 Grounding: ∂ (boundary) + μ (mapping) + ρ (recursion) + π (persistence)
//!
//! Protocol:
//! - Input: JSON on stdin with session_id
//! - Output: Mining prompt on stdout (informational)
//! - Exit: 0 (always passes - mining is advisory)
//!
//! Formula: V(t) = B(t) x eta(t) x r(t)
//!   where B = basis primitives, eta = transfer efficiency, r = reuse rate

use nexcore_hook_lib::cytokine::emit_hook_completed;

use chrono::Utc;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const HOOK_NAME: &str = "compound-growth-miner";

/// Transfer efficiency multipliers by tier (from Triangular Transfer Law).
const TRANSFER_T1: f64 = 1.0;
const TRANSFER_T2_P: f64 = 0.9;
const TRANSFER_T2_C: f64 = 0.7;
const _TRANSFER_T3: f64 = 0.4;

/// Decay constant (delta = 0.1) — used in transfer law, not in reuse rate.
const _DELTA: f64 = 0.1;

#[derive(Deserialize)]
struct HookInput {
    session_id: Option<String>,
}

/// Snapshot of primitive basis from the growth log.
#[derive(Debug, Default, Deserialize)]
struct BasisSnapshot {
    #[serde(default)]
    t1: u32,
    #[serde(default)]
    t2_p: u32,
    #[serde(default)]
    t2_c: u32,
    #[serde(default)]
    t3: u32,
    #[serde(default)]
    reused: u32,
    #[serde(default)]
    total_needed: u32,
}

impl BasisSnapshot {
    fn basis_size(&self) -> u32 {
        self.t1 + self.t2_p + self.t2_c + self.t3
    }

    fn transfer_efficiency(&self) -> f64 {
        let total = self.basis_size();
        if total == 0 {
            return 0.0;
        }
        let weighted = (self.t1 as f64 * TRANSFER_T1)
            + (self.t2_p as f64 * TRANSFER_T2_P)
            + (self.t2_c as f64 * TRANSFER_T2_C)
            + (self.t3 as f64 * _TRANSFER_T3);
        weighted / total as f64
    }

    fn reuse_rate(&self) -> f64 {
        if self.total_needed == 0 {
            return 1.0;
        }
        self.reused as f64 / self.total_needed as f64
    }

    fn velocity(&self) -> f64 {
        self.basis_size() as f64 * self.transfer_efficiency() * self.reuse_rate()
    }
}

/// Session activity metrics from file system analysis.
struct SessionActivity {
    files_modified: usize,
    files_created: usize,
    rust_files: usize,
    skill_files: usize,
    brain_artifacts: usize,
    test_files: usize,
}

impl SessionActivity {
    /// Estimate the mining potential of this session.
    fn mining_potential(&self) -> &'static str {
        let score = self.rust_files * 3
            + self.skill_files * 2
            + self.brain_artifacts
            + self.test_files
            + self.files_created * 2;

        match score {
            0..=2 => "LOW",
            3..=8 => "MEDIUM",
            9..=15 => "HIGH",
            _ => "EXCEPTIONAL",
        }
    }

    /// Suggest the tier most likely to yield new primitives.
    fn suggested_tier(&self) -> &'static str {
        if self.rust_files > 5 {
            "T2-P (cross-domain patterns emerging from Rust work)"
        } else if self.skill_files > 0 {
            "T2-C (composite patterns from skill composition)"
        } else if self.files_created > 3 {
            "T2-P (new abstractions from fresh code)"
        } else {
            "T2-C (composite patterns from modifications)"
        }
    }
}

/// Check if a file was modified within the last 3 hours.
fn is_recent(meta: &fs::Metadata) -> bool {
    meta.modified()
        .map(|t| t.elapsed().map(|d| d.as_secs() < 10800).unwrap_or(false))
        .unwrap_or(false)
}

/// Count recently modified files (within last 3 hours).
fn analyze_session_activity(home: &Path) -> SessionActivity {
    let mut activity = SessionActivity {
        files_modified: 0,
        files_created: 0,
        rust_files: 0,
        skill_files: 0,
        brain_artifacts: 0,
        test_files: 0,
    };

    // Check recent brain artifacts
    let brain_dir = home.join(".claude/brain/sessions");
    if let Ok(entries) = fs::read_dir(&brain_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata()
                && is_recent(&meta)
            {
                activity.brain_artifacts += 1;
            }
        }
    }

    // Check recently modified skills
    let skills_dir = home.join(".claude/skills");
    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let skill_md = entry.path().join("SKILL.md");
            if let Ok(meta) = fs::metadata(&skill_md)
                && is_recent(&meta)
            {
                activity.skill_files += 1;
            }
        }
    }

    // Check nexcore workspace for recent Rust changes
    let nexcore_crates = home.join("nexcore/crates");
    count_recent_files(&nexcore_crates, &mut activity, 2);

    // Check prima workspace for recent Rust changes
    let prima_crates = home.join("prima/crates");
    count_recent_files(&prima_crates, &mut activity, 2);

    activity
}

/// Recursively count recently modified files with depth limit.
fn count_recent_files(dir: &Path, activity: &mut SessionActivity, depth: u8) {
    if depth == 0 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count_recent_files(&path, activity, depth - 1);
            continue;
        }

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if !is_recent(&meta) {
            continue;
        }

        activity.files_modified += 1;

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name.ends_with(".rs") {
            activity.rust_files += 1;
            if name.starts_with("test") || name.contains("_test") {
                activity.test_files += 1;
            }
        }

        // Heuristic: new files often have create time == modify time
        if let Ok(created) = meta.created()
            && let Ok(modified) = meta.modified()
        {
            let diff = modified
                .duration_since(created)
                .map(|d| d.as_secs())
                .unwrap_or(999);
            if diff < 60 {
                activity.files_created += 1;
            }
        }
    }
}

/// Load current basis snapshot from the growth log.
fn load_basis(home: &Path) -> BasisSnapshot {
    let growth_log = home.join(".claude/compound_growth.json");
    if let Ok(content) = fs::read_to_string(&growth_log)
        && let Ok(snap) = serde_json::from_str::<BasisSnapshot>(&content)
    {
        return snap;
    }

    // Default basis snapshot (as of 2026-02-05)
    BasisSnapshot {
        t1: 15,
        t2_p: 2,
        t2_c: 13,
        t3: 0,
        reused: 28,
        total_needed: 30,
    }
}

/// Save updated basis snapshot.
fn save_basis(home: &Path, basis: &BasisSnapshot) {
    let growth_log = home.join(".claude/compound_growth.json");
    let json = serde_json::json!({
        "t1": basis.t1,
        "t2_p": basis.t2_p,
        "t2_c": basis.t2_c,
        "t3": basis.t3,
        "reused": basis.reused,
        "total_needed": basis.total_needed,
        "last_updated": Utc::now().to_rfc3339(),
    });

    if let Ok(content) = serde_json::to_string_pretty(&json) {
        let _ = fs::write(&growth_log, content);
    }
}

/// Generate the mining prompt for the next session.
fn generate_mining_prompt(
    session_id: &str,
    activity: &SessionActivity,
    basis: &BasisSnapshot,
) -> String {
    let velocity = basis.velocity();
    let t2p_advantage = TRANSFER_T2_P / TRANSFER_T2_C;

    format!(
        r#"
## Compound Growth Mining Report

**Session**: {} | **Time**: {}
**Mining Potential**: {} | **Suggested Tier**: {}

### Session Activity
| Metric | Count |
|--------|-------|
| Files Modified | {} |
| Files Created | {} |
| Rust Files | {} |
| Skills Modified | {} |
| Brain Artifacts | {} |
| Test Files | {} |

### Current Basis
| Tier | Count | Transfer |
|------|-------|----------|
| T1   | {}    | 1.00     |
| T2-P | {}    | 0.90     |
| T2-C | {}    | 0.70     |
| T3   | {}    | 0.40     |

**Velocity**: {:.4} | **Basis Size**: {} | **Reuse**: {}/{}

### Mining Directive

> Extract T2-P primitives FIRST (compounds {:.0}% faster than T2-C).
> Then compose T2-C from exhausted T2-P extraction.
> Feed results: `mcp__nexcore__compound_growth(add_tier="T2-P", add_count=N)`

### Recursive Growth Formula
```
V(t) = B(t) x eta(t) x r(t)
delta = 0.1 (Triangular Transfer Law)
C(n) = 1.0 - T(n-1) x delta
```
"#,
        session_id,
        Utc::now().format("%Y-%m-%d %H:%M"),
        activity.mining_potential(),
        activity.suggested_tier(),
        activity.files_modified,
        activity.files_created,
        activity.rust_files,
        activity.skill_files,
        activity.brain_artifacts,
        activity.test_files,
        basis.t1,
        basis.t2_p,
        basis.t2_c,
        basis.t3,
        velocity,
        basis.basis_size(),
        basis.reused,
        basis.total_needed,
        (t2p_advantage - 1.0) * 100.0,
    )
}

fn main() {
    // Read hook input from stdin
    let stdin = io::stdin();
    let input: HookInput = match serde_json::from_reader(stdin.lock()) {
        Ok(i) => i,
        Err(_) => {
            println!("{{}}");
            std::process::exit(0);
        }
    };

    let session_id = input
        .session_id
        .unwrap_or_else(|| "unknown".to_string())
        .chars()
        .take(8)
        .collect::<String>();

    // Get home directory
    let home = match std::env::var("HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
            println!("{{}}");
            std::process::exit(0);
        }
    };

    // Analyze session activity
    let activity = analyze_session_activity(&home);

    // Skip if minimal activity (< 3 files touched)
    if activity.files_modified < 3 {
        println!("{{}}");
        std::process::exit(0);
    }

    // Load current basis
    let basis = load_basis(&home);

    // Generate mining prompt
    let prompt = generate_mining_prompt(&session_id, &activity, &basis);

    // Save mining prompt as brain artifact
    let mining_dir = home.join(".claude/brain/sessions/compound-growth");
    if !mining_dir.exists() {
        let _ = fs::create_dir_all(&mining_dir);
    }

    let artifact_path = mining_dir.join(format!(
        "mining-{}-{}.md",
        session_id,
        Utc::now().format("%Y%m%d")
    ));
    let _ = fs::write(&artifact_path, &prompt);

    // Update the basis snapshot with timestamp (track session without changing counts)
    save_basis(&home, &basis);

    // Emit cytokine signal (IL-6 = growth signal)
    emit_hook_completed(
        HOOK_NAME,
        0,
        &format!(
            "mining_potential={},velocity={:.2}",
            activity.mining_potential(),
            basis.velocity()
        ),
    );

    // Output mining report
    eprintln!(
        "Compound Growth: {} potential | V={:.2} | B={} | Next: extract {} primitives",
        activity.mining_potential(),
        basis.velocity(),
        basis.basis_size(),
        activity.suggested_tier()
    );

    // Always pass
    println!("{{}}");
    std::process::exit(0);
}
