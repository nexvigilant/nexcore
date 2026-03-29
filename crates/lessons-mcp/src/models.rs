//! Data models for lessons learned
//! Tier: T2-P primitives

use nexcore_chrono::DateTime;
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
    pub created_at: DateTime,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lesson(title: &str, content: &str, ctx: &str, tags: &[&str]) -> Lesson {
        Lesson {
            id: 0,
            title: title.into(),
            content: content.into(),
            context: ctx.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            primitives: vec![],
            created_at: DateTime::now(),
            source: String::new(),
        }
    }

    #[test]
    fn db_default_empty() {
        let db = LessonsDb::default();
        assert!(db.lessons.is_empty());
        assert_eq!(db.next_id, 0);
    }

    #[test]
    fn db_add_increments_id() {
        let mut db = LessonsDb::default();
        let id1 = db.add(make_lesson("L1", "c1", "hooks", &[]));
        let id2 = db.add(make_lesson("L2", "c2", "hooks", &[]));
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(db.lessons.len(), 2);
    }

    #[test]
    fn db_get_by_id() {
        let mut db = LessonsDb::default();
        db.add(make_lesson("Found", "body", "skills", &[]));
        assert!(db.get(0).is_some());
        assert!(db.get(999).is_none());
    }

    #[test]
    fn db_search_title() {
        let mut db = LessonsDb::default();
        db.add(make_lesson("Hook Timeout", "fix it", "hooks", &[]));
        db.add(make_lesson("Skill Bug", "also fix", "skills", &[]));
        assert_eq!(db.search("timeout").len(), 1);
        assert_eq!(db.search("fix").len(), 2);
        assert_eq!(db.search("nonexistent").len(), 0);
    }

    #[test]
    fn db_search_case_insensitive() {
        let mut db = LessonsDb::default();
        db.add(make_lesson("UPPER", "lower", "hooks", &[]));
        assert_eq!(db.search("upper").len(), 1);
    }

    #[test]
    fn db_by_context() {
        let mut db = LessonsDb::default();
        db.add(make_lesson("H1", "c", "hooks", &[]));
        db.add(make_lesson("S1", "c", "skills", &[]));
        db.add(make_lesson("H2", "c", "hooks", &[]));
        assert_eq!(db.by_context("hooks").len(), 2);
        assert_eq!(db.by_context("HOOKS").len(), 2); // case insensitive
        assert_eq!(db.by_context("mcp").len(), 0);
    }

    #[test]
    fn db_by_tag() {
        let mut db = LessonsDb::default();
        db.add(make_lesson("L1", "c", "hooks", &["safety", "critical"]));
        db.add(make_lesson("L2", "c", "hooks", &["safety"]));
        db.add(make_lesson("L3", "c", "hooks", &["perf"]));
        assert_eq!(db.by_tag("safety").len(), 2);
        assert_eq!(db.by_tag("SAFETY").len(), 2);
        assert_eq!(db.by_tag("perf").len(), 1);
    }

    #[test]
    fn primitives_summary() {
        let mut db = LessonsDb::default();
        let mut l = make_lesson("L1", "c", "hooks", &[]);
        l.primitives = vec![
            ExtractedPrimitive::t1("Sequence", "seq"),
            ExtractedPrimitive::t1("Sequence", "seq again"),
            ExtractedPrimitive::t2p("Transform", "xform"),
        ];
        db.add(l);
        let summary = db.primitives_summary();
        assert_eq!(summary.get("Sequence").map(|(_, c)| *c), Some(2));
        assert_eq!(summary.get("Transform").map(|(_, c)| *c), Some(1));
    }

    #[test]
    fn primitive_tier_constructors() {
        let t1 = ExtractedPrimitive::t1("A", "d");
        assert_eq!(t1.tier, PrimitiveTier::T1);
        let t2p = ExtractedPrimitive::t2p("B", "d");
        assert_eq!(t2p.tier, PrimitiveTier::T2P);
        let t2c = ExtractedPrimitive::t2c("C", "d");
        assert_eq!(t2c.tier, PrimitiveTier::T2C);
        let t3 = ExtractedPrimitive::t3("D", "d");
        assert_eq!(t3.tier, PrimitiveTier::T3);
    }
}
