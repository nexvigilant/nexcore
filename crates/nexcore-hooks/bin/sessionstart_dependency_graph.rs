//! SessionStart hook: Capability Dependency Graph
//!
//! Builds and analyzes the dependency graph between capabilities
//! to identify bottlenecks, cycles, and optimization opportunities.
//!
//! Graph Structure:
//! - Nodes: Skills, Hooks, MCP tools, Subagents
//! - Edges: Invokes, requires, enhances, blocks
//!
//! Analysis:
//! - Critical path identification
//! - Cycle detection
//! - Orphan capability detection
//! - Hub identification (high fan-out nodes)
//!
//! Exit codes:
//! - 0: Success (graph analysis in context)

use nexcore_hooks::{exit_success, exit_with_session_context, read_input};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Capability node in the dependency graph
#[derive(Debug, Clone)]
struct CapNode {
    name: String,
    kind: CapKind,
    outgoing: Vec<String>, // capabilities this one depends on
    incoming: Vec<String>, // capabilities that depend on this
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CapKind {
    Skill,
    Hook,
    Subagent,
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_success(),
    };

    let mut graph: HashMap<String, CapNode> = HashMap::new();

    // Build graph from skills
    build_skill_graph(&mut graph);

    // Build graph from subagents
    build_subagent_graph(&mut graph);

    if graph.is_empty() {
        exit_success();
    }

    // Analyze graph
    let hubs = find_hubs(&graph, 3);
    let orphans = find_orphans(&graph);
    let critical_path = find_critical_skills(&graph);

    if hubs.is_empty() && orphans.is_empty() && critical_path.is_empty() {
        exit_success();
    }

    // Build context
    let mut context =
        String::from("🕸️ **DEPENDENCY GRAPH** ────────────────────────────────────\n");

    if !hubs.is_empty() {
        context.push_str("   Hub capabilities (high connectivity):\n");
        for (name, count) in hubs.iter().take(3) {
            context.push_str("   • ");
            context.push_str(name);
            context.push_str(" (");
            context.push_str(&count.to_string());
            context.push_str(" connections)\n");
        }
        context.push('\n');
    }

    if !orphans.is_empty() {
        context.push_str("   Orphan capabilities (no connections):\n");
        for name in orphans.iter().take(5) {
            context.push_str("   • ");
            context.push_str(name);
            context.push('\n');
        }
        context.push('\n');
    }

    if !critical_path.is_empty() {
        context.push_str("   Critical path (most depended upon):\n");
        for name in critical_path.iter().take(3) {
            context.push_str("   → ");
            context.push_str(name);
            context.push('\n');
        }
    }

    context.push_str("───────────────────────────────────────────────────────────\n");

    exit_with_session_context(&context);
}

fn skills_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".claude/skills")
}

fn agents_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".config/agents")
}

fn build_skill_graph(graph: &mut HashMap<String, CapNode>) {
    let dir = skills_dir();

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }

        let skill_name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        let skill_md = entry.path().join("SKILL.md");
        let content = fs::read_to_string(&skill_md).unwrap_or_default();

        // Extract dependencies from SKILL.md
        let deps = extract_skill_deps(&content);

        graph.insert(
            skill_name.clone(),
            CapNode {
                name: skill_name,
                kind: CapKind::Skill,
                outgoing: deps,
                incoming: Vec::new(),
            },
        );
    }

    // Build incoming edges
    let names: Vec<String> = graph.keys().cloned().collect();
    for name in &names {
        let outgoing = graph
            .get(name)
            .map(|n| n.outgoing.clone())
            .unwrap_or_default();
        for dep in outgoing {
            if let Some(node) = graph.get_mut(&dep) {
                node.incoming.push(name.clone());
            }
        }
    }
}

fn build_subagent_graph(graph: &mut HashMap<String, CapNode>) {
    let dir = agents_dir();

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path
            .extension()
            .map(|e| e == "yaml" || e == "yml")
            .unwrap_or(false)
        {
            continue;
        }

        let agent_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if agent_name.is_empty() {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap_or_default();
        let deps = extract_agent_deps(&content);

        graph.insert(
            agent_name.clone(),
            CapNode {
                name: agent_name,
                kind: CapKind::Subagent,
                outgoing: deps,
                incoming: Vec::new(),
            },
        );
    }
}

fn extract_skill_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();

    // Look for skill references in content
    for line in content.lines() {
        // Check for /skill-name patterns
        if line.contains("'/") || line.contains("\"/") {
            for word in line.split_whitespace() {
                let word = word.trim_matches(|c| c == '\'' || c == '"' || c == '`');
                if word.starts_with('/') && word.len() > 1 {
                    deps.push(word[1..].to_string());
                }
            }
        }

        // Check for skill-name references
        if line.contains("skill:") || line.contains("requires:") {
            for word in line.split_whitespace() {
                let word = word.trim_matches(|c| c == ',' || c == ':' || c == '"' || c == '\'');
                if word.contains('-') && !word.starts_with('-') {
                    deps.push(word.to_string());
                }
            }
        }
    }

    deps.into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn extract_agent_deps(content: &str) -> Vec<String> {
    let mut deps = Vec::new();

    for line in content.lines() {
        // Look for skill references
        if line.contains("skill") || line.contains("Skill") {
            for word in line.split_whitespace() {
                let word = word.trim_matches(|c| c == ',' || c == ':' || c == '"' || c == '\'');
                if word.contains('-') && !word.starts_with('-') && word.len() > 3 {
                    deps.push(word.to_string());
                }
            }
        }
    }

    deps.into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

fn find_hubs(graph: &HashMap<String, CapNode>, threshold: usize) -> Vec<(String, usize)> {
    let mut hubs: Vec<_> = graph
        .values()
        .map(|n| (n.name.clone(), n.outgoing.len() + n.incoming.len()))
        .filter(|(_, count)| *count >= threshold)
        .collect();

    hubs.sort_by(|a, b| b.1.cmp(&a.1));
    hubs
}

fn find_orphans(graph: &HashMap<String, CapNode>) -> Vec<String> {
    graph
        .values()
        .filter(|n| n.outgoing.is_empty() && n.incoming.is_empty())
        .map(|n| n.name.clone())
        .collect()
}

fn find_critical_skills(graph: &HashMap<String, CapNode>) -> Vec<String> {
    let mut critical: Vec<_> = graph
        .values()
        .filter(|n| n.kind == CapKind::Skill)
        .map(|n| (n.name.clone(), n.incoming.len()))
        .filter(|(_, count)| *count > 0)
        .collect();

    critical.sort_by(|a, b| b.1.cmp(&a.1));
    critical.into_iter().map(|(name, _)| name).collect()
}
