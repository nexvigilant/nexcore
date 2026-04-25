//! Skill Hunter - Gamified skill issue detector
//!
//! Hunt down issues in your skill ecosystem!

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use nexcore_error::{Context, Result};
use nexcore_fs::dirs;
use nexcore_fs::walk::WalkDir;
use skill_hunter::*;
use std::path::Path;

fn main() -> Result<()> {
    print_banner();

    let skills_dir = dirs::home_dir()
        .map(|h| h.join(".claude/skills"))
        .ok_or_else(|| nexcore_error::NexError::msg("Cannot find home directory"))?;

    if !skills_dir.exists() {
        println!(
            "{} Skills directory not found: {}",
            SKULL,
            skills_dir.display()
        );
        return Ok(());
    }

    let results = hunt_skills(&skills_dir)?;
    let state = calculate_game_state(&results);

    print_results(&results);
    print_summary(&state);
    print_achievements(&state);

    Ok(())
}

fn print_banner() {
    println!();
    println!(
        "{}",
        style("╔═══════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("║     SKILL HUNTER - Issue Detector Game    ║").cyan()
    );
    println!(
        "{}",
        style("║   Hunt bugs. Earn points. Level up.       ║").cyan()
    );
    println!(
        "{}",
        style("╚═══════════════════════════════════════════╝").cyan()
    );
    println!();
}

fn hunt_skills(dir: &Path) -> Result<Vec<SkillResult>> {
    let skill_paths: Vec<_> = WalkDir::new(dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "SKILL.md")
        .map(|e| e.path().to_path_buf())
        .collect();

    let pb = ProgressBar::new(skill_paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );

    let mut results = Vec::new();

    for path in skill_paths {
        let name = extract_skill_name(&path);
        pb.set_message(format!("Hunting: {}", name));

        let issues = validate_skill_file(&path)?;
        let score = calculate_score(&issues);

        results.push(SkillResult {
            name,
            issues,
            score,
        });
        pb.inc(1);
    }

    pb.finish_with_message("Hunt complete!");
    results.sort_by(|a, b| a.score.cmp(&b.score));

    Ok(results)
}

fn validate_skill_file(path: &Path) -> Result<Vec<Issue>> {
    let content = std::fs::read_to_string(path)?;
    let mut issues = Vec::new();

    let frontmatter = parse_frontmatter(&content);
    check_required_fields(&frontmatter, &mut issues);
    check_recommended_fields(&frontmatter, &mut issues);

    let body = extract_body(&content);
    if body.len() < 100 {
        issues.push(Issue {
            severity: DiagnosticLevel::Warning,
            message: "Sparse documentation (<100 chars)".into(),
            fix_hint: "Add more documentation to help users".into(),
        });
    }

    Ok(issues)
}

fn print_results(results: &[SkillResult]) {
    println!();
    println!("{} {} {}", SWORD, style("HUNT RESULTS").bold(), SWORD);
    println!();

    let problematic: Vec<_> = results.iter().filter(|r| r.score < 100).collect();

    if problematic.is_empty() {
        println!("{} All skills are perfect! No bugs found.", TROPHY);
        return;
    }

    println!("{} Found {} skills with issues:", BUG, problematic.len());
    println!();

    for result in problematic.iter().take(10) {
        print_skill_result(result);
    }

    if problematic.len() > 10 {
        println!("  ... and {} more", problematic.len() - 10);
    }
}

fn print_skill_result(result: &SkillResult) {
    let score_color = match result.score {
        0..=49 => style(result.score).red(),
        50..=79 => style(result.score).yellow(),
        _ => style(result.score).green(),
    };

    println!(
        "  {} {} (score: {})",
        if result.score < 50 { SKULL } else { BUG },
        style(&result.name).bold(),
        score_color
    );

    for issue in &result.issues {
        let prefix = match issue.severity {
            DiagnosticLevel::Critical => style("CRIT").red(),
            DiagnosticLevel::Warning => style("WARN").yellow(),
            DiagnosticLevel::Info => style("INFO").dim(),
        };
        println!("    [{}] {}", prefix, issue.message);
        println!("         {}", style(&issue.fix_hint).dim());
    }
    println!();
}

fn print_summary(state: &GameState) {
    println!("{}", style("═══ GAME SUMMARY ═══").cyan());
    println!();
    println!("  Skills scanned:  {}", state.skills_scanned);
    println!("  Total issues:    {}", state.total_issues);
    println!(
        "  {} Critical:      {}",
        SKULL,
        style(state.critical_count).red()
    );
    println!(
        "  {} Warnings:      {}",
        BUG,
        style(state.warning_count).yellow()
    );
    println!(
        "  {} Perfect:       {}",
        SHIELD,
        style(state.perfect_skills).green()
    );
    println!();

    let avg_score = if state.skills_scanned > 0 {
        state.total_score / state.skills_scanned
    } else {
        0
    };

    println!("  Average Score:   {}/100", avg_score);
    println!("  Total Score:     {} XP", state.total_score);
    println!();
}

fn print_achievements(state: &GameState) {
    println!(
        "{} {} {}",
        STAR,
        style("ACHIEVEMENTS UNLOCKED").bold().yellow(),
        STAR
    );
    println!();

    let mut achievements = Vec::new();

    if state.perfect_skills >= 10 {
        achievements.push(("Perfectionist", "10+ perfect skills"));
    }
    if state.perfect_skills >= 30 {
        achievements.push(("Master Craftsman", "30+ perfect skills"));
    }
    if state.critical_count == 0 {
        achievements.push(("Zero Critical", "No critical issues!"));
    }
    if state.skills_scanned >= 50 {
        achievements.push(("Skill Collector", "50+ skills scanned"));
    }
    if state.total_score >= 5000 {
        achievements.push(("High Scorer", "5000+ total XP"));
    }

    if achievements.is_empty() {
        achievements.push(("Bug Hunter", "Started the hunt!"));
    }

    for (name, desc) in achievements {
        println!("  {} {} - {}", TROPHY, style(name).bold().green(), desc);
    }
    println!();
}
