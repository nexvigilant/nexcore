//! Skill matching with weighted scoring.

use super::models::{ScoredSkill, Skill, TaskAnalysis};
use std::collections::{HashMap, HashSet};

/// Skill matcher with weighted scoring algorithm.
pub struct SkillMatcher {
    /// Available skills
    pub skills: Vec<Skill>,
    historical_success: HashMap<String, f64>,
    keyword_weight: f64,
    semantic_weight: f64,
    domain_weight: f64,
    historical_weight: f64,
}

impl SkillMatcher {
    /// Create a new skill matcher.
    #[must_use]
    pub fn new(skills: Vec<Skill>) -> Self {
        Self {
            skills,
            historical_success: HashMap::new(),
            keyword_weight: 1.0,
            semantic_weight: 1.5,
            domain_weight: 1.0,
            historical_weight: 0.5,
        }
    }

    /// Match skills to a task analysis.
    #[must_use]
    pub fn match_skills(&self, analysis: &TaskAnalysis, top_k: usize) -> Vec<ScoredSkill> {
        let mut candidates: Vec<ScoredSkill> = self
            .skills
            .iter()
            .map(|skill| {
                let (score, reasons) = self.score_skill(skill, analysis);
                ScoredSkill {
                    skill: skill.clone(),
                    score,
                    match_reasons: reasons,
                }
            })
            .filter(|s| s.score > 0.1)
            .collect();

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.into_iter().take(top_k).collect()
    }

    fn score_skill(&self, skill: &Skill, analysis: &TaskAnalysis) -> (f64, Vec<String>) {
        let mut reasons = Vec::new();

        let keyword_score = self.calculate_keyword_match(skill, analysis);
        if keyword_score > 0.0 {
            reasons.push(format!("keyword:{keyword_score:.2}"));
        }

        let semantic_score = self.calculate_semantic_match(skill, analysis);
        if semantic_score > 0.0 {
            reasons.push(format!("semantic:{semantic_score:.2}"));
        }

        let domain_score = if skill.domain == analysis.domain {
            1.0
        } else {
            0.0
        };
        if domain_score > 0.0 {
            reasons.push(format!("domain:{domain_score:.2}"));
        }

        let historical_score = *self.historical_success.get(&skill.name).unwrap_or(&0.5);

        let total_score = self.keyword_weight * keyword_score
            + self.semantic_weight * semantic_score
            + self.domain_weight * domain_score
            + self.historical_weight * historical_score;

        (total_score, reasons)
    }

    fn calculate_keyword_match(&self, skill: &Skill, analysis: &TaskAnalysis) -> f64 {
        if analysis.keywords_extracted.is_empty() {
            return 0.0;
        }
        let query_keywords: HashSet<String> = analysis
            .keywords_extracted
            .iter()
            .map(|k| k.to_lowercase())
            .collect();
        let skill_keywords: HashSet<String> =
            skill.keywords.iter().map(|k| k.to_lowercase()).collect();
        let matches = query_keywords.intersection(&skill_keywords).count();
        matches as f64 / query_keywords.len().max(1) as f64
    }

    fn calculate_semantic_match(&self, skill: &Skill, analysis: &TaskAnalysis) -> f64 {
        let mut score: f64 = 0.0;
        let query_keywords: HashSet<String> = analysis
            .keywords_extracted
            .iter()
            .map(|k| k.to_lowercase())
            .collect();

        for trigger in &skill.triggers {
            if query_keywords.contains(&trigger.to_lowercase()) {
                score += 0.8;
            }
        }
        if skill
            .description
            .to_lowercase()
            .contains(&analysis.intent.to_lowercase())
        {
            score += 0.5;
        }
        score.min(2.0)
    }

    /// Update historical success rate for a skill.
    pub fn update_history(&mut self, skill_name: &str, success: bool) {
        let current = self
            .historical_success
            .entry(skill_name.to_string())
            .or_insert(0.5);
        *current = 0.8 * (*current) + 0.2 * (if success { 1.0 } else { 0.0 });
    }
}
