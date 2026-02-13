//! Skill Bonding Tracker - PostToolUse Hook
//!
//! Tracks skill invocations and creates a bond graph showing skill relationships
//! using chemistry metaphors:
//!
//! - **Covalent bonds**: Direct data flow (output of skill A feeds input of skill B)
//! - **Ionic bonds**: Dependency relationships (skill A requires skill B to exist)
//! - **Catalyst bonds**: Skills that enable but don't directly connect (accelerators)
//!
//! Event: PostToolUse
//! Matcher: Task (detects subagent skill invocations)
//!
//! Persists bond data to: ~/.claude/brain/skill_bonds/
//!
//! Bond Graph Format:
//! ```json
//! {
//!   "nodes": [{"id": "skill-a", "invocations": 5}],
//!   "edges": [{"from": "skill-a", "to": "skill-b", "bond_type": "covalent", "strength": 3}]
//! }
//! ```

use chrono::{DateTime, Utc};
use nexcore_hooks::{HookInput, exit_success_auto, exit_success_auto_with, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// =============================================================================
// Bond Types (Chemistry Metaphor)
// =============================================================================

/// Bond type representing the relationship between skills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BondType {
    /// Covalent: Strong bond with shared data (output -> input)
    /// Like atoms sharing electrons - skills share data directly
    Covalent,

    /// Ionic: Dependency relationship (one requires the other to exist)
    /// Like charged atoms - attraction without direct sharing
    Ionic,

    /// Catalyst: Enables execution without being consumed
    /// Like enzymes - accelerates without being part of the reaction
    Catalyst,
}

impl BondType {
    /// Get the display symbol for this bond type
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Covalent => "==", // Double bond (strong)
            Self::Ionic => "->",    // Directional (dependency)
            Self::Catalyst => "~~", // Wavy (enables)
        }
    }

    /// Get bond strength multiplier
    pub fn strength_multiplier(&self) -> f64 {
        match self {
            Self::Covalent => 1.0, // Full strength
            Self::Ionic => 0.6,    // Moderate
            Self::Catalyst => 0.3, // Weak but present
        }
    }
}

// =============================================================================
// Bond Event (Single Invocation Record)
// =============================================================================

/// A single skill invocation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondEvent {
    /// Timestamp of the invocation
    pub timestamp: DateTime<Utc>,

    /// Session ID where the invocation occurred
    pub session_id: String,

    /// The skill that was invoked
    pub skill: String,

    /// Bond type for this invocation
    pub bond_type: BondType,

    /// Skills that provided input to this skill (upstream)
    #[serde(default)]
    pub inputs_from: Vec<String>,

    /// Skills that will receive output from this skill (downstream)
    #[serde(default)]
    pub outputs_to: Vec<String>,

    /// Context about the invocation
    pub context: InvocationContext,
}

/// Context for a skill invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    /// Tool that was used
    pub tool: String,

    /// Working directory
    pub cwd: String,

    /// Custom instructions (may contain skill name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,

    /// Agent type if this was a subagent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,

    /// Prompt snippet (first 100 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_snippet: Option<String>,
}

// =============================================================================
// Bond Graph (Aggregate View)
// =============================================================================

/// A node in the bond graph representing a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillNode {
    /// Skill identifier (slug)
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Total invocation count
    pub invocations: u32,

    /// Last invoked timestamp
    pub last_invoked: DateTime<Utc>,

    /// Tags from the skill (if known)
    #[serde(default)]
    pub tags: Vec<String>,
}

/// An edge in the bond graph representing a relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondEdge {
    /// Source skill ID
    pub from: String,

    /// Target skill ID
    pub to: String,

    /// Type of bond
    pub bond_type: BondType,

    /// Number of times this bond was observed
    pub observations: u32,

    /// Calculated strength (observations * bond_type multiplier)
    pub strength: f64,
}

/// The complete bond graph
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BondGraph {
    /// Skill nodes
    pub nodes: Vec<SkillNode>,

    /// Bond edges between skills
    pub edges: Vec<BondEdge>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Total events processed
    pub total_events: u32,
}

impl BondGraph {
    /// Load the bond graph from disk
    pub fn load() -> Self {
        let path = Self::graph_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(graph) = serde_json::from_str(&content) {
                    return graph;
                }
            }
        }
        Self::default()
    }

    /// Save the bond graph to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::graph_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Get the path to the bond graph file
    fn graph_path() -> PathBuf {
        bonds_dir().join("bond_graph.json")
    }

    /// Record a bond event
    pub fn record_event(&mut self, event: &BondEvent) {
        self.total_events += 1;
        self.updated_at = Utc::now();

        // Update or create node for the invoked skill
        self.upsert_node(&event.skill);

        // Create edges for inputs (upstream skills)
        for input_skill in &event.inputs_from {
            self.upsert_node(input_skill);
            self.upsert_edge(input_skill, &event.skill, BondType::Covalent);
        }

        // Create edges for outputs (downstream skills)
        for output_skill in &event.outputs_to {
            self.upsert_node(output_skill);
            self.upsert_edge(&event.skill, output_skill, BondType::Covalent);
        }

        // If no explicit connections, check for catalyst pattern
        if event.inputs_from.is_empty() && event.outputs_to.is_empty() {
            // Skill invoked standalone - might be a catalyst
            if event.bond_type == BondType::Catalyst {
                // Record catalyst relationship with last invoked skill
                // Clone the ID to avoid borrow conflict
                let last_node_id = self
                    .nodes
                    .iter()
                    .rev()
                    .find(|n| n.id != event.skill)
                    .map(|n| n.id.clone());

                if let Some(target_id) = last_node_id {
                    self.upsert_edge(&event.skill, &target_id, BondType::Catalyst);
                }
            }
        }
    }

    /// Update or insert a skill node
    fn upsert_node(&mut self, skill_id: &str) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == skill_id) {
            node.invocations += 1;
            node.last_invoked = Utc::now();
        } else {
            self.nodes.push(SkillNode {
                id: skill_id.to_string(),
                name: skill_id.replace('-', " "),
                invocations: 1,
                last_invoked: Utc::now(),
                tags: Vec::new(),
            });
        }
    }

    /// Update or insert a bond edge
    fn upsert_edge(&mut self, from: &str, to: &str, bond_type: BondType) {
        if let Some(edge) = self.edges.iter_mut().find(|e| e.from == from && e.to == to) {
            edge.observations += 1;
            edge.strength = edge.observations as f64 * edge.bond_type.strength_multiplier();
            // Upgrade bond type if we see stronger evidence
            if bond_type.strength_multiplier() > edge.bond_type.strength_multiplier() {
                edge.bond_type = bond_type;
            }
        } else {
            self.edges.push(BondEdge {
                from: from.to_string(),
                to: to.to_string(),
                bond_type,
                observations: 1,
                strength: bond_type.strength_multiplier(),
            });
        }
    }

    /// Generate a text visualization of the bond graph
    pub fn visualize(&self) -> String {
        let mut output = String::new();

        output.push_str("=== SKILL BOND GRAPH ===\n\n");

        // Top skills by invocation
        output.push_str("Top Skills (by invocations):\n");
        let mut sorted_nodes: Vec<_> = self.nodes.iter().collect();
        sorted_nodes.sort_by(|a, b| b.invocations.cmp(&a.invocations));
        for node in sorted_nodes.iter().take(10) {
            output.push_str(&format!("  {} ({}x)\n", node.id, node.invocations));
        }

        output.push_str("\nBonds:\n");
        let mut sorted_edges: Vec<_> = self.edges.iter().collect();
        sorted_edges.sort_by(|a, b| {
            b.strength
                .partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for edge in sorted_edges.iter().take(15) {
            output.push_str(&format!(
                "  {} {} {} (strength: {:.1}, type: {:?})\n",
                edge.from,
                edge.bond_type.symbol(),
                edge.to,
                edge.strength,
                edge.bond_type
            ));
        }

        output.push_str(&format!(
            "\nTotal: {} skills, {} bonds, {} events\n",
            self.nodes.len(),
            self.edges.len(),
            self.total_events
        ));

        output
    }
}

// =============================================================================
// Session State (Recent Invocations for Chaining Detection)
// =============================================================================

/// Session-specific state for detecting skill chains
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SessionBondState {
    /// Session ID
    session_id: String,

    /// Recently invoked skills (for chain detection)
    recent_skills: Vec<String>,

    /// Last skill that produced output
    last_producer: Option<String>,

    /// Skills waiting for input
    pending_consumers: Vec<String>,
}

impl SessionBondState {
    /// Load or create session state
    fn load(session_id: &str) -> Self {
        let path = session_state_path(session_id);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self {
            session_id: session_id.to_string(),
            ..Default::default()
        }
    }

    /// Save session state
    fn save(&self) -> std::io::Result<()> {
        let path = session_state_path(&self.session_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Record a skill invocation
    fn record_skill(&mut self, skill: &str) {
        // Add to recent, keep last 10
        self.recent_skills.push(skill.to_string());
        if self.recent_skills.len() > 10 {
            self.recent_skills.remove(0);
        }
    }

    /// Get upstream skills (potential inputs)
    fn get_upstream_skills(&self, current: &str) -> Vec<String> {
        // Skills invoked before this one could be inputs
        self.recent_skills
            .iter()
            .filter(|s| *s != current)
            .take(3)
            .cloned()
            .collect()
    }
}

// =============================================================================
// Path Helpers
// =============================================================================

/// Get the bonds directory
fn bonds_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("brain")
        .join("skill_bonds")
}

/// Get path to session state file
fn session_state_path(session_id: &str) -> PathBuf {
    bonds_dir()
        .join("sessions")
        .join(format!("{}.json", session_id))
}

/// Get path to events log (append-only)
fn events_log_path() -> PathBuf {
    bonds_dir().join("events.jsonl")
}

/// Append an event to the events log
fn append_event(event: &BondEvent) -> std::io::Result<()> {
    use std::io::Write;
    let path = events_log_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{}", line)
}

// =============================================================================
// Skill Detection
// =============================================================================

/// Detect the skill being invoked from the hook input
fn detect_skill(input: &HookInput) -> Option<String> {
    // Check custom_instructions for skill invocation pattern
    if let Some(instructions) = &input.custom_instructions {
        // Look for "Invoke skill: skill-name" or similar patterns
        if let Some(skill) = extract_skill_from_instructions(instructions) {
            return Some(skill);
        }
    }

    // Check tool_input for skill-related fields
    if let Some(tool_input) = &input.tool_input {
        if let Some(skill_name) = tool_input.get("skill").and_then(|v| v.as_str()) {
            return Some(skill_name.to_string());
        }
        if let Some(task_desc) = tool_input.get("description").and_then(|v| v.as_str()) {
            if let Some(skill) = extract_skill_from_text(task_desc) {
                return Some(skill);
            }
        }
    }

    // Check agent_type for skill-based subagents
    if let Some(agent_type) = &input.agent_type {
        if agent_type.contains("skill") || agent_type.starts_with('/') {
            return Some(normalize_skill_name(agent_type));
        }
    }

    // Check prompt for skill invocation patterns
    if let Some(prompt) = input.get_prompt() {
        if let Some(skill) = extract_skill_from_text(prompt) {
            return Some(skill);
        }
    }

    None
}

/// Extract skill name from custom instructions
fn extract_skill_from_instructions(text: &str) -> Option<String> {
    // Pattern: "Invoke skill: name" or "/skill-name"
    let patterns = [
        r"(?i)invoke\s+skill[:\s]+([a-z0-9-]+)",
        r"(?i)skill[:\s]+([a-z0-9-]+)",
        r"/([a-z0-9-]+)",
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }
    None
}

/// Extract skill name from text (prompt or description)
fn extract_skill_from_text(text: &str) -> Option<String> {
    // Look for skill invocation patterns
    let lower = text.to_lowercase();

    // Direct invocation: "run skill-name" or "/skill-name"
    if let Some(pos) = lower.find('/') {
        let rest = &text[pos + 1..];
        let skill: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-')
            .collect();
        if skill.len() >= 3 {
            return Some(skill.to_lowercase());
        }
    }

    // "using skill-name skill" pattern
    if let Ok(re) = regex::Regex::new(r"(?i)using\s+([a-z0-9-]+)\s+skill") {
        if let Some(caps) = re.captures(&lower) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }

    None
}

/// Normalize skill name
fn normalize_skill_name(name: &str) -> String {
    name.trim_start_matches('/')
        .to_lowercase()
        .replace(' ', "-")
}

// =============================================================================
// ADV² — Advisor Self-Improvement Protocol
// =============================================================================

/// A pending advisor recommendation awaiting correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdvisorPending {
    timestamp: DateTime<Utc>,
    session_id: String,
    recommended: Vec<String>,
    context: String,
}

/// Advisor hit/miss statistics for self-improvement
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AdvisorStats {
    hits: std::collections::HashMap<String, u32>,
    misses: std::collections::HashMap<String, u32>,
    total_recommendations: u32,
    updated_at: DateTime<Utc>,
}

impl AdvisorStats {
    fn load() -> Self {
        let path = bonds_dir().join("advisor_stats.json");
        fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or_default()
    }

    fn save(&self) -> std::io::Result<()> {
        let path = bonds_dir().join("advisor_stats.json");
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)
    }
}

/// Jaccard token similarity: tokenize on `-`, compute |∩| / |∪|
fn token_sim(a: &str, b: &str) -> f64 {
    let ta: std::collections::HashSet<&str> = a.split('-').collect();
    let tb: std::collections::HashSet<&str> = b.split('-').collect();
    let isect = ta.intersection(&tb).count();
    let union = ta.union(&tb).count();
    if union == 0 {
        0.0
    } else {
        isect as f64 / union as f64
    }
}

/// Path to advisor pending recommendations file
fn advisor_pending_path() -> PathBuf {
    bonds_dir().join("advisor_pending.json")
}

/// Load pending advisor recommendations from disk
fn load_pendings() -> Vec<AdvisorPending> {
    fs::read_to_string(advisor_pending_path())
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

/// Save pending advisor recommendations to disk
fn save_pendings(pendings: &[AdvisorPending]) {
    if let Ok(json) = serde_json::to_string_pretty(pendings) {
        let _ = fs::write(advisor_pending_path(), json);
    }
}

/// Check if invoked skill matches any recommendation in a pending entry
fn match_recommendation(pending: &AdvisorPending, invoked: &str) -> Option<String> {
    pending
        .recommended
        .iter()
        .find(|rec| *rec == invoked || token_sim(rec, invoked) >= 0.5)
        .cloned()
}

/// Check if a skill correlates with a pending advisor recommendation.
/// Returns the matched recommendation skill name if found.
fn try_correlate_advisor(invoked_skill: &str, session_id: &str) -> Option<String> {
    let pendings = load_pendings();
    let now = Utc::now();
    let window = chrono::Duration::minutes(30);

    pendings
        .iter()
        .filter(|p| p.session_id == session_id)
        .filter(|p| now.signed_duration_since(p.timestamp) <= window)
        .find_map(|p| match_recommendation(p, invoked_skill))
}

/// Record a hit (recommended skill was actually used)
fn record_advisor_hit(matched_skill: &str) {
    let mut stats = AdvisorStats::load();
    *stats.hits.entry(matched_skill.to_string()).or_insert(0) += 1;
    stats.updated_at = Utc::now();
    if let Err(e) = stats.save() {
        eprintln!("Warning: Failed to save advisor stats: {e}");
    }
}

/// Expire stale pendings (>30 min) and record misses for their recommendations
fn expire_stale_pendings() {
    let pendings = load_pendings();
    if pendings.is_empty() {
        return;
    }

    let now = Utc::now();
    let window = chrono::Duration::minutes(30);
    let mut stats = AdvisorStats::load();
    let mut remaining = Vec::new();

    for pending in pendings {
        if now.signed_duration_since(pending.timestamp) > window {
            for rec in &pending.recommended {
                *stats.misses.entry(rec.clone()).or_insert(0) += 1;
            }
            stats.updated_at = now;
        } else {
            remaining.push(pending);
        }
    }

    let _ = stats.save();
    save_pendings(&remaining);
}

/// Detect bond type based on invocation context
fn detect_bond_type(input: &HookInput, session_state: &SessionBondState) -> BondType {
    // If there are recent skills, this might be a covalent bond (data flow)
    if !session_state.recent_skills.is_empty() {
        return BondType::Covalent;
    }

    // If this is a Task subagent, likely ionic (dependency)
    if input.tool_name.as_deref() == Some("Task") {
        return BondType::Ionic;
    }

    // Default to catalyst (enables but doesn't directly connect)
    BondType::Catalyst
}

// =============================================================================
// Main Hook Logic
// =============================================================================

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process relevant tools (Task for subagents, Skill for direct invocations, Bash for CLI)
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if !matches!(tool_name, "Task" | "Skill" | "Bash") {
        exit_success_auto();
    }

    // For Bash, only process skill-related commands
    if tool_name == "Bash" {
        if let Some(cmd) = input.get_command() {
            if !cmd.contains("skill") && !cmd.starts_with('/') {
                exit_success_auto();
            }
        } else {
            exit_success_auto();
        }
    }

    // Detect which skill is being invoked
    let skill = match detect_skill(&input) {
        Some(s) => s,
        None => exit_success_auto(), // No skill detected, nothing to track
    };

    // Load session state
    let mut session_state = SessionBondState::load(&input.session_id);

    // Detect bond type
    let bond_type = detect_bond_type(&input, &session_state);

    // Get upstream skills (potential inputs)
    let inputs_from = session_state.get_upstream_skills(&skill);

    // Create the bond event
    let event = BondEvent {
        timestamp: Utc::now(),
        session_id: input.session_id.clone(),
        skill: skill.clone(),
        bond_type,
        inputs_from: inputs_from.clone(),
        outputs_to: Vec::new(), // Will be populated on next skill invocation
        context: InvocationContext {
            tool: tool_name.to_string(),
            cwd: input.cwd.clone(),
            custom_instructions: input.custom_instructions.clone(),
            agent_type: input.agent_type.clone(),
            prompt_snippet: input.get_prompt().map(|p| p.chars().take(100).collect()),
        },
    };

    // Append to events log - warn on failure but continue
    if let Err(e) = append_event(&event) {
        eprintln!("Warning: Failed to append bond event: {}", e);
    }

    // Update bond graph - warn on failure but continue
    let mut graph = BondGraph::load();
    graph.record_event(&event);
    if let Err(e) = graph.save() {
        eprintln!("Warning: Failed to save bond graph: {}", e);
    }

    // Update session state - warn on failure but continue
    session_state.record_skill(&skill);
    if let Err(e) = session_state.save() {
        eprintln!("Warning: Failed to save session state: {}", e);
    }

    // ADV² — Correlate with pending advisor recommendations
    expire_stale_pendings();
    let adv2_msg = match try_correlate_advisor(&skill, &input.session_id) {
        Some(matched) => {
            record_advisor_hit(&matched);
            format!(" | ADV² hit: {matched}")
        }
        None => String::new(),
    };

    // Format output message
    let bond_msg = if inputs_from.is_empty() {
        format!("Skill bond: {} ({:?}){adv2_msg}", skill, bond_type)
    } else {
        format!(
            "Skill bond: {} {} {} ({:?}){adv2_msg}",
            inputs_from.join(", "),
            bond_type.symbol(),
            skill,
            bond_type
        )
    };

    exit_success_auto_with(&bond_msg);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bond_type_symbols() {
        assert_eq!(BondType::Covalent.symbol(), "==");
        assert_eq!(BondType::Ionic.symbol(), "->");
        assert_eq!(BondType::Catalyst.symbol(), "~~");
    }

    #[test]
    fn test_bond_type_strength() {
        assert!(BondType::Covalent.strength_multiplier() > BondType::Ionic.strength_multiplier());
        assert!(BondType::Ionic.strength_multiplier() > BondType::Catalyst.strength_multiplier());
    }

    #[test]
    fn test_extract_skill_from_instructions() {
        assert_eq!(
            extract_skill_from_instructions("Invoke skill: ctvp-validator"),
            Some("ctvp-validator".to_string())
        );
        assert_eq!(
            extract_skill_from_instructions("/forge"),
            Some("forge".to_string())
        );
        assert_eq!(
            extract_skill_from_instructions("Using skill: primitive-rust"),
            Some("primitive-rust".to_string())
        );
    }

    #[test]
    fn test_extract_skill_from_text() {
        assert_eq!(
            extract_skill_from_text("Run /ctvp-validator on this"),
            Some("ctvp-validator".to_string())
        );
        assert_eq!(
            extract_skill_from_text("using forge skill"),
            Some("forge".to_string())
        );
    }

    #[test]
    fn test_normalize_skill_name() {
        assert_eq!(normalize_skill_name("/CTVP-Validator"), "ctvp-validator");
        assert_eq!(normalize_skill_name("Skill Name"), "skill-name");
    }

    #[test]
    fn test_bond_graph_upsert_node() {
        let mut graph = BondGraph::default();
        graph.upsert_node("test-skill");
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].invocations, 1);

        graph.upsert_node("test-skill");
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].invocations, 2);
    }

    #[test]
    fn test_bond_graph_upsert_edge() {
        let mut graph = BondGraph::default();
        graph.upsert_edge("skill-a", "skill-b", BondType::Covalent);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].observations, 1);
        assert!((graph.edges[0].strength - 1.0).abs() < f64::EPSILON);

        graph.upsert_edge("skill-a", "skill-b", BondType::Covalent);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].observations, 2);
        assert!((graph.edges[0].strength - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_session_state_recent_skills() {
        let mut state = SessionBondState::default();
        state.record_skill("skill-1");
        state.record_skill("skill-2");
        state.record_skill("skill-3");

        let upstream = state.get_upstream_skills("skill-3");
        assert!(upstream.contains(&"skill-1".to_string()));
        assert!(upstream.contains(&"skill-2".to_string()));
        assert!(!upstream.contains(&"skill-3".to_string()));
    }

    // ADV² Tests

    #[test]
    fn test_token_sim_exact() {
        assert!(
            (token_sim("rust-anatomy-expert", "rust-anatomy-expert") - 1.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_token_sim_partial() {
        // 2/3 overlap: {rust, anatomy} / {rust, anatomy, expert}
        let sim = token_sim("rust-anatomy-expert", "rust-anatomy");
        assert!(sim >= 0.5, "Expected ≥0.5, got {sim}");
        assert!((sim - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_token_sim_no_overlap() {
        assert!(token_sim("forge", "brain-dev") < 0.5);
        assert!((token_sim("forge", "brain-dev")).abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_sim_single_token() {
        // "forge" vs "forge" = exact
        assert!((token_sim("forge", "forge") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_match_recommendation_exact() {
        let pending = AdvisorPending {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            recommended: vec!["forge".to_string(), "mcp-dev".to_string()],
            context: "test".to_string(),
        };
        assert_eq!(
            match_recommendation(&pending, "forge"),
            Some("forge".to_string())
        );
        assert_eq!(
            match_recommendation(&pending, "mcp-dev"),
            Some("mcp-dev".to_string())
        );
        assert_eq!(match_recommendation(&pending, "brain-dev"), None);
    }

    #[test]
    fn test_match_recommendation_fuzzy() {
        let pending = AdvisorPending {
            timestamp: Utc::now(),
            session_id: "test".to_string(),
            recommended: vec!["rust-anatomy-expert".to_string()],
            context: "test".to_string(),
        };
        // "rust-anatomy" should fuzzy-match "rust-anatomy-expert" (sim = 0.67)
        assert!(match_recommendation(&pending, "rust-anatomy").is_some());
    }

    #[test]
    fn test_advisor_stats_roundtrip() {
        let mut stats = AdvisorStats::default();
        *stats.hits.entry("forge".to_string()).or_insert(0) += 3;
        *stats.misses.entry("forge".to_string()).or_insert(0) += 1;

        let h = *stats.hits.get("forge").unwrap_or(&0);
        let m = *stats.misses.get("forge").unwrap_or(&0);
        let rate = f64::from(h) / f64::from(h + m);
        assert!((rate - 0.75).abs() < f64::EPSILON);
    }
}
