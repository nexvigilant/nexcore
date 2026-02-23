//! Data models for lessons learned
//! Tier: T2-P primitives

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tier classification for extracted primitives
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrimitiveTier {
    T1,  // Universal
    T2P, // Cross-domain primitive
    T2C, // Cross-domain composite
    T3,  // Domain-specific
}

/// An extracted primitive from a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPrimitive {
    pub name: String,
    pub tier: PrimitiveTier,
    pub description: String,
}

impl ExtractedPrimitive {
    pub fn t1(name: &str, desc: &str) -> Self {
        Self {
            name: name.into(),
            tier: PrimitiveTier::T1,
            description: desc.into(),
        }
    }

    pub fn t2p(name: &str, desc: &str) -> Self {
        Self {
            name: name.into(),
            tier: PrimitiveTier::T2P,
            description: desc.into(),
        }
    }

    pub fn t2c(name: &str, desc: &str) -> Self {
        Self {
            name: name.into(),
            tier: PrimitiveTier::T2C,
            description: desc.into(),
        }
    }

    pub fn t3(name: &str, desc: &str) -> Self {
        Self {
            name: name.into(),
            tier: PrimitiveTier::T3,
            description: desc.into(),
        }
    }
}

/// A lesson learned about hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub context: String,
    pub tags: Vec<String>,
    pub primitives: Vec<ExtractedPrimitive>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub source: String,
}

/// Lessons database
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LessonsDb {
    pub lessons: Vec<Lesson>,
    pub next_id: u64,
}

impl LessonsDb {
    pub fn add(&mut self, mut lesson: Lesson) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        lesson.id = id;
        self.lessons.push(lesson);
        id
    }

    pub fn get(&self, id: u64) -> Option<&Lesson> {
        self.lessons.iter().find(|l| l.id == id)
    }

    pub fn search(&self, q: &str) -> Vec<&Lesson> {
        let q = q.to_lowercase();
        self.lessons
            .iter()
            .filter(|l| matches_query(l, &q))
            .collect()
    }

    pub fn by_context(&self, ctx: &str) -> Vec<&Lesson> {
        self.lessons
            .iter()
            .filter(|l| l.context.eq_ignore_ascii_case(ctx))
            .collect()
    }

    pub fn by_tag(&self, tag: &str) -> Vec<&Lesson> {
        let t = tag.to_lowercase();
        self.lessons.iter().filter(|l| has_tag(l, &t)).collect()
    }

    pub fn primitives_summary(&self) -> HashMap<String, (PrimitiveTier, usize)> {
        collect_primitives(&self.lessons)
    }
}

fn matches_query(l: &Lesson, q: &str) -> bool {
    l.title.to_lowercase().contains(q) || l.content.to_lowercase().contains(q)
}

fn has_tag(l: &Lesson, t: &str) -> bool {
    l.tags.iter().any(|tag| tag.to_lowercase() == *t)
}

fn collect_primitives(lessons: &[Lesson]) -> HashMap<String, (PrimitiveTier, usize)> {
    let mut map = HashMap::new();
    for p in lessons.iter().flat_map(|l| &l.primitives) {
        increment_primitive(&mut map, p);
    }
    map
}

fn increment_primitive(map: &mut HashMap<String, (PrimitiveTier, usize)>, p: &ExtractedPrimitive) {
    map.entry(p.name.clone())
        .and_modify(|(_, c)| *c += 1)
        .or_insert((p.tier.clone(), 1));
}
