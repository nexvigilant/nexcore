//! Chain planner for building execution DAGs.

use super::models::{Chain, ChainNode, ChainOperator, ScoredSkill, Skill, TaskAnalysis};
use std::collections::{HashMap, HashSet};

/// A preset workflow definition.
#[derive(Debug, Clone)]
pub struct Preset {
    /// Name of the preset (without @ prefix)
    pub name: &'static str,
    /// The chain expression to expand to
    pub chain: &'static str,
    /// Human-readable description
    pub description: &'static str,
}

/// Chain planner that builds execution DAGs.
pub struct ChainPlanner {
    phase_order: Vec<&'static str>,
    phase_keywords: HashMap<&'static str, HashSet<&'static str>>,
    presets: HashMap<&'static str, Preset>,
}

impl Default for ChainPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainPlanner {
    /// Create a new chain planner with default phases and presets.
    #[must_use]
    pub fn new() -> Self {
        let phase_order = vec!["research", "design", "implement", "validate", "deliver"];

        let mut phase_keywords = HashMap::new();
        phase_keywords.insert(
            "research",
            [
                "explore",
                "search",
                "literature",
                "context",
                "understand",
                "investigate",
            ]
            .into_iter()
            .collect(),
        );
        phase_keywords.insert(
            "design",
            [
                "algorithm",
                "plan",
                "architect",
                "design",
                "dag",
                "structure",
            ]
            .into_iter()
            .collect(),
        );
        phase_keywords.insert(
            "implement",
            [
                "proceed",
                "build",
                "create",
                "implement",
                "generate",
                "code",
            ]
            .into_iter()
            .collect(),
        );
        phase_keywords.insert(
            "validate",
            [
                "test",
                "lint",
                "typecheck",
                "validate",
                "verify",
                "quality",
                "review",
            ]
            .into_iter()
            .collect(),
        );
        phase_keywords.insert(
            "deliver",
            ["commit", "deploy", "ship", "release", "report", "document"]
                .into_iter()
                .collect(),
        );

        // Initialize presets
        let presets = Self::default_presets();

        Self {
            phase_order,
            phase_keywords,
            presets,
        }
    }

    /// Get the default presets.
    fn default_presets() -> HashMap<&'static str, Preset> {
        let mut presets = HashMap::new();

        presets.insert(
            "research",
            Preset {
                name: "research",
                chain: "explore → context7-toolkit → bluf-report",
                description: "Deep investigation workflow",
            },
        );

        presets.insert(
            "implement",
            Preset {
                name: "implement",
                chain: "algorithm → dag-planner → proceed-lite",
                description: "Design and build workflow",
            },
        );

        presets.insert(
            "quality",
            Preset {
                name: "quality",
                chain: "lint && typecheck && test → code-quality-toolkit",
                description: "Full QA workflow",
            },
        );

        presets.insert(
            "secure",
            Preset {
                name: "secure",
                chain: "red-team → security-audit → proceed",
                description: "Security-first workflow",
            },
        );

        presets.insert(
            "ship",
            Preset {
                name: "ship",
                chain: "lint && typecheck && test → commit → pr-create",
                description: "Release preparation workflow",
            },
        );

        presets
    }

    /// Get a preset by name (with or without @ prefix).
    #[must_use]
    pub fn get_preset(&self, name: &str) -> Option<&Preset> {
        let name = name.strip_prefix('@').unwrap_or(name);
        self.presets.get(name)
    }

    /// Get all available preset names.
    #[must_use]
    pub fn preset_names(&self) -> Vec<&'static str> {
        self.presets.keys().copied().collect()
    }

    /// Expand a preset into a Chain.
    ///
    /// Returns `None` if the preset doesn't exist.
    #[must_use]
    pub fn expand_preset(
        &self,
        preset_name: &str,
        skill_lookup: &HashMap<String, Skill>,
    ) -> Option<Chain> {
        let preset = self.get_preset(preset_name)?;
        let mut chain = self.parse_expression(preset.chain, skill_lookup);
        chain.preset_name = Some(preset.name.to_string());
        Some(chain)
    }

    /// Check if a string is a preset reference.
    #[must_use]
    pub fn is_preset(input: &str) -> bool {
        input.starts_with('@')
    }

    /// Plan a chain from scored skills.
    #[must_use]
    pub fn plan(&self, skills: Vec<ScoredSkill>, analysis: TaskAnalysis) -> Chain {
        let mut phases: HashMap<&str, Vec<Skill>> = HashMap::new();
        for phase in &self.phase_order {
            phases.insert(phase, Vec::new());
        }

        for scored in &skills {
            let phase = self.classify_phase(&scored.skill);
            if let Some(phase_skills) = phases.get_mut(phase.as_str()) {
                phase_skills.push(scored.skill.clone());
            }
        }

        let mut nodes = Vec::new();
        let mut prev_phase_skills = Vec::new();

        let empty_skills = Vec::new();
        for (level, phase) in self.phase_order.iter().enumerate() {
            let phase_skills = phases.get(phase).unwrap_or(&empty_skills);
            if phase_skills.is_empty() {
                continue;
            }

            for (i, skill) in phase_skills.iter().take(2).enumerate() {
                nodes.push(ChainNode {
                    skill_name: skill.name.clone(),
                    operator: if i == 0 {
                        ChainOperator::Sequential
                    } else {
                        ChainOperator::Parallel
                    },
                    level: level as u32,
                    dependencies: prev_phase_skills.clone(),
                });
            }
            prev_phase_skills = phase_skills
                .iter()
                .take(2)
                .map(|s| s.name.clone())
                .collect();
        }

        if let Some(last) = nodes.last_mut() {
            last.operator = ChainOperator::End;
        }

        Chain {
            nodes,
            analysis: Some(analysis),
            confidence: 0.8,
            preset_name: None,
            safety_manifold: None,
        }
    }

    fn classify_phase(&self, skill: &Skill) -> String {
        for phase in &self.phase_order {
            if let Some(keywords) = self.phase_keywords.get(phase) {
                if keywords
                    .iter()
                    .any(|&kw| skill.name.to_lowercase().contains(kw))
                {
                    return (*phase).to_string();
                }
                if skill
                    .keywords
                    .iter()
                    .any(|sk| keywords.contains(sk.as_str()))
                {
                    return (*phase).to_string();
                }
            }
        }
        "implement".to_string()
    }

    /// Parse a chain expression string into a Chain object.
    ///
    /// # Examples
    /// - `"algorithm → proceed-lite"` - Sequential execution
    /// - `"lint && typecheck && test"` - Parallel execution
    /// - `"skill1 || skill2"` - Fallback (try skill2 if skill1 fails)
    #[must_use]
    pub fn parse_expression(
        &self,
        expression: &str,
        skill_lookup: &HashMap<String, Skill>,
    ) -> Chain {
        // Normalize operators: -> to →
        let normalized = expression.replace("->", "→");

        let mut nodes = Vec::new();
        let mut level = 0u32;

        // Split by operators using simple state machine
        let chars: Vec<char> = normalized.chars().collect();
        let mut token_start = 0;
        let mut i = 0;

        while i < chars.len() {
            let remaining: String = chars[i..].iter().collect();

            // Check for operators
            let (op, op_len) = if remaining.starts_with('→') {
                (Some(ChainOperator::Sequential), 1)
            } else if remaining.starts_with("&&") {
                (Some(ChainOperator::Parallel), 2)
            } else if remaining.starts_with("||") {
                (Some(ChainOperator::Fallback), 2)
            } else if remaining.starts_with('?') {
                (Some(ChainOperator::Conditional), 1)
            } else {
                (None, 0)
            };

            if let Some(operator) = op {
                let token: String = chars[token_start..i].iter().collect();
                let token = token.trim();

                if !token.is_empty() {
                    let skill_name = if let Some(skill) = skill_lookup.get(token) {
                        skill.name.clone()
                    } else {
                        token.to_string()
                    };
                    nodes.push(ChainNode {
                        skill_name,
                        operator,
                        level,
                        dependencies: Vec::new(),
                    });

                    if operator != ChainOperator::Parallel {
                        level += 1;
                    }
                }

                i += op_len;
                token_start = i;
            } else {
                i += 1;
            }
        }

        // Handle last token
        let last_token: String = chars[token_start..].iter().collect();
        let last_token = last_token.trim();
        if !last_token.is_empty() {
            let skill_name = if let Some(skill) = skill_lookup.get(last_token) {
                skill.name.clone()
            } else {
                last_token.to_string()
            };
            nodes.push(ChainNode {
                skill_name,
                operator: ChainOperator::End,
                level,
                dependencies: Vec::new(),
            });
        }

        Chain {
            nodes,
            analysis: None,
            confidence: 1.0,
            preset_name: None,
            safety_manifold: None,
        }
    }

    /// Suggest a chain expression from matched skills.
    #[must_use]
    pub fn suggest_chain(&self, skills: &[ScoredSkill]) -> String {
        if skills.is_empty() {
            return String::new();
        }

        let mut design_skills = Vec::new();
        let mut execute_skills = Vec::new();
        let mut quality_skills = Vec::new();

        for scored in skills {
            let phase = self.classify_phase(&scored.skill);
            match phase.as_str() {
                "research" | "design" => design_skills.push(scored.skill.name.clone()),
                "implement" => execute_skills.push(scored.skill.name.clone()),
                "validate" | "deliver" => quality_skills.push(scored.skill.name.clone()),
                _ => execute_skills.push(scored.skill.name.clone()),
            }
        }

        let mut parts = Vec::new();
        if let Some(skill) = design_skills.first() {
            parts.push(skill.clone());
        }
        if let Some(skill) = execute_skills.first() {
            parts.push(skill.clone());
        } else if design_skills.is_empty() {
            parts.push("proceed-lite".to_string());
        }
        if let Some(skill) = quality_skills.first() {
            parts.push(skill.clone());
        }

        parts.join(" → ")
    }
}
