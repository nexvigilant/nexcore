//! grounded-skill: Run the GROUNDED feedback loop on Claude Code skills.
//!
//! Usage:
//!   grounded-skill <skill-path>           # Assess one skill
//!   grounded-skill --all <skills-dir>     # Assess all skills

use std::path::{Path, PathBuf};

use grounded::skill::{SkillContext, SkillSummary};
use grounded::{
    BashWorld, Confidence, EvidenceChain, ExperienceStore, GroundedLoop, Learning, MemoryStore,
    SqliteStore, Verdict,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: grounded-skill <skill-path|--all <dir>>");
        std::process::exit(1);
    }

    let db_path = std::env::var("GROUNDED_DB").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        format!("{home}/.claude/brain/grounded.db")
    });

    if args[1] == "--all" {
        let dir = args.get(2).map(|s| s.as_str()).unwrap_or_else(|| {
            eprintln!("--all requires a directory argument");
            std::process::exit(1);
        });
        run_all(dir, &db_path);
    } else {
        let skill_path = &args[1];
        let (summary, chain) = assess_skill(skill_path);
        print!("{summary}");
        print!("{chain}");
    }
}

/// Assess a single skill, returning summary and evidence chain.
/// For single-skill mode (no SQLite persistence needed).
fn assess_skill(skill_path: &str) -> (SkillSummary, EvidenceChain) {
    let (summary, _, chain) = assess_skill_with_learnings(skill_path);
    (summary, chain)
}

/// Assess a single skill, returning summary, collected learnings, and evidence chain.
/// Used by run_all to enable SQLite persistence of learnings.
fn assess_skill_with_learnings(skill_path: &str) -> (SkillSummary, Vec<Learning>, EvidenceChain) {
    let path = resolve_path(skill_path);
    if !path.exists() {
        eprintln!("Skill path does not exist: {}", path.display());
        std::process::exit(1);
    }

    let ctx = SkillContext::new(&path);
    let skill_name = ctx.skill_name().to_string();

    let mut chain = EvidenceChain::new(
        format!("skill compliance: {skill_name}"),
        Confidence::new(0.5).unwrap_or(Confidence::NONE),
    );

    let world = BashWorld::new("bash", 30);
    let store = MemoryStore::new();
    let mut grounded = GroundedLoop::new(ctx, world, store);
    let mut collected_learnings: Vec<Learning> = Vec::new();

    // Loop up to 9 times (current CHECKS.len()); GroundedLoop::iterate() returns
    // Err("all checks exhausted") once all checks are consumed, so this breaks early
    // if CHECKS shrinks, and silently skips any beyond 9 if CHECKS grows.
    for _ in 0..9 {
        match grounded.iterate() {
            Ok(learning) => {
                let verdict = learning.value().verdict;
                let insight = learning.value().insight.clone();
                let conf = Confidence::new(0.3).unwrap_or(Confidence::NONE);
                match verdict {
                    Verdict::Supported => chain.strengthen(&insight, conf),
                    Verdict::Refuted => chain.weaken(&insight, conf),
                    Verdict::Inconclusive => {}
                }
                collected_learnings.push(learning.into_value());
            }
            Err(_) => break,
        }
    }

    let summary = grounded.context().summary();
    (summary, collected_learnings, chain)
}

fn run_all(dir: &str, db_path: &str) {
    let dir_path = resolve_path(dir);
    let mut entries: Vec<PathBuf> = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(&dir_path) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").exists() {
                entries.push(path);
            }
        }
    }

    entries.sort();

    let mut tier_counts: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();
    let mut total_score = 0.0;

    // Open SQLite store once so learnings from ALL skills are persisted this run.
    let mut sqlite = SqliteStore::open(Path::new(db_path)).ok();

    for entry in &entries {
        let path_str = entry.to_string_lossy().to_string();
        let (summary, learnings, _chain) = assess_skill_with_learnings(&path_str);
        let tier = summary.tier.clone();
        *tier_counts.entry(tier).or_insert(0) += 1;
        total_score += summary.compliance_score;

        // Persist learnings from this skill to the durable store.
        if let Some(ref mut store) = sqlite {
            for learning in &learnings {
                if let Err(e) = store.persist(learning) {
                    eprintln!("warn: failed to persist learning: {e}");
                }
            }
        }

        let score_pct = (summary.compliance_score * 100.0) as u32;
        let gaps = if summary.checks_failed.is_empty() {
            String::new()
        } else {
            format!(" [{}]", summary.checks_failed.join(", "))
        };
        println!(
            "  {:<35} {:>6}  {:>3}%{}",
            summary.skill_name, summary.tier, score_pct, gaps
        );
    }

    let count = entries.len();
    let avg_score = if count > 0 {
        total_score / count as f64
    } else {
        0.0
    };

    println!("\n=== GROUNDED ECOSYSTEM SUMMARY ===");
    println!("Total skills: {count}");
    println!("Average score: {:.0}%", avg_score * 100.0);
    println!();

    let mut tiers: Vec<_> = tier_counts.iter().collect();
    // Sort descending by count; BTreeMap guarantees deterministic iteration for ties
    tiers.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (tier, count) in &tiers {
        let bar_len = (**count as usize) * 2;
        let bar: String = "█".repeat(bar_len.min(40));
        println!("  {tier:<10} {count:>3} {bar}");
    }

    // Report total cumulative learnings persisted across all runs.
    if let Some(store) = sqlite {
        let total_learnings = store.count().unwrap_or(0);
        println!("\nCumulative learnings in store: {total_learnings}");
    }
}

fn resolve_path(p: &str) -> PathBuf {
    let path = PathBuf::from(p);
    if path.is_absolute() {
        return path;
    }
    if p.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(p.replacen('~', &home, 1));
        }
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(path)
}
