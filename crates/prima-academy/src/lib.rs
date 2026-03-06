// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Prima Academy — Academic Course Classification Primitives
//!
//! Maps university course numbering to Prima's tier system.
//!
//! ## Primitive Grounding
//!
//! | Component | Primitive | Meaning |
//! |-----------|-----------|---------|
//! | Subject code | N | Domain identifier (3-letter) |
//! | Level digit | λ | Year/position in curriculum |
//! | Course number | σ | Sequence within level |
//! | Full code | κ | Enables comparison/prerequisites |
//! | Curriculum | μ | Subject → courses mapping |
//!
//! ## Course Number Format
//!
//! ```text
//! MTH 101
//!  │   │││
//!  │   ││└── Specifier (1-9): Specific course variant
//!  │   │└─── Subcategory (0-9): Track within level
//!  │   └──── Level (1-6+): Year/advancement
//!  └──────── Subject: 3-letter domain code
//! ```
//!
//! ## Tier Mapping (Inverted)
//!
//! | Course Level | Prima Tier | Abstraction |
//! |--------------|------------|-------------|
//! | 100-level | T3 | Domain-specific intro |
//! | 200-level | T3 | Domain intermediate |
//! | 300-level | T2-C | Cross-topic synthesis |
//! | 400-level | T2-C | Capstone integration |
//! | 500-level | T2-P | Graduate research |
//! | 600+ level | T1 | Foundational theory |

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]
#![warn(missing_docs)]
pub mod transfer;

pub use transfer::{AffinityMatrix, CapabilityMultiplier, TransferResult};

use nexcore_error::Error;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// ERRORS
// ═══════════════════════════════════════════════════════════════════════════════

/// Academy errors.
#[derive(Debug, Error)]
pub enum AcademyError {
    #[error("Invalid course code: {0}")]
    InvalidCode(String),
    #[error("Unknown subject: {0}")]
    UnknownSubject(String),
    #[error("Invalid level: {0}")]
    InvalidLevel(u8),
}

/// Result type for academy operations.
pub type AcademyResult<T> = Result<T, AcademyError>;

// ═══════════════════════════════════════════════════════════════════════════════
// SUBJECT DOMAINS (T3)
// ═══════════════════════════════════════════════════════════════════════════════

/// Academic subject domain.
///
/// ## Tier: T3 (Domain-specific)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Subject {
    // STEM
    /// Mathematics (MTH/MAT)
    Mathematics,
    /// Physics (PHY/PHS)
    Physics,
    /// Chemistry (CHM/CHE)
    Chemistry,
    /// Biology (BIO)
    Biology,
    /// Computer Science (CSC/CIS/CS)
    ComputerScience,
    /// Engineering (ENG/ENR)
    Engineering,
    /// Statistics (STA/STT)
    Statistics,

    // Healthcare
    /// Pharmacology (PHR/PHA)
    Pharmacology,
    /// Nursing (NUR/NSG)
    Nursing,
    /// Medicine (MED)
    Medicine,
    /// PublicHealth (PUH/PBH)
    PublicHealth,

    // Humanities
    /// English (ENG/ENL)
    English,
    /// History (HIS/HST)
    History,
    /// Philosophy (PHI/PHL)
    Philosophy,
    /// Psychology (PSY)
    Psychology,

    // Business
    /// Business (BUS/BUA)
    Business,
    /// Economics (ECO/ECN)
    Economics,
    /// Accounting (ACC/ACT)
    Accounting,
    /// Finance (FIN)
    Finance,

    /// Unknown subject with raw code
    Unknown(u16),
}

impl Subject {
    /// Parse subject from 3-letter code.
    pub fn from_code(code: &str) -> Self {
        let upper = code.to_uppercase();
        match upper.as_str() {
            "MTH" | "MAT" | "MATH" => Self::Mathematics,
            "PHY" | "PHS" | "PHYS" => Self::Physics,
            "CHM" | "CHE" | "CHEM" => Self::Chemistry,
            "BIO" | "BIOL" => Self::Biology,
            "CSC" | "CIS" | "CS" | "CSCI" => Self::ComputerScience,
            "EGR" | "ENR" | "ENGR" => Self::Engineering,
            "STA" | "STT" | "STAT" => Self::Statistics,
            "PHR" | "PHA" | "PHAR" => Self::Pharmacology,
            "NUR" | "NSG" | "NURS" => Self::Nursing,
            "MED" | "MEDI" => Self::Medicine,
            "PUH" | "PBH" | "PUBH" => Self::PublicHealth,
            "ENG" | "ENL" | "ENGL" => Self::English,
            "HIS" | "HST" | "HIST" => Self::History,
            "PHI" | "PHL" | "PHIL" => Self::Philosophy,
            "PSY" | "PSYC" => Self::Psychology,
            "BUS" | "BUA" | "BUSI" => Self::Business,
            "ECO" | "ECN" | "ECON" => Self::Economics,
            "ACC" | "ACT" | "ACCT" => Self::Accounting,
            "FIN" | "FINA" => Self::Finance,
            _ => {
                // Encode unknown as numeric hash
                let hash = code.bytes().fold(0u16, |acc, b| acc.wrapping_add(b as u16));
                Self::Unknown(hash)
            }
        }
    }

    /// Get canonical 3-letter code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Mathematics => "MTH",
            Self::Physics => "PHY",
            Self::Chemistry => "CHM",
            Self::Biology => "BIO",
            Self::ComputerScience => "CSC",
            Self::Engineering => "EGR",
            Self::Statistics => "STA",
            Self::Pharmacology => "PHR",
            Self::Nursing => "NUR",
            Self::Medicine => "MED",
            Self::PublicHealth => "PUH",
            Self::English => "ENG",
            Self::History => "HIS",
            Self::Philosophy => "PHI",
            Self::Psychology => "PSY",
            Self::Business => "BUS",
            Self::Economics => "ECO",
            Self::Accounting => "ACC",
            Self::Finance => "FIN",
            Self::Unknown(_) => "UNK",
        }
    }

    /// Check if STEM discipline.
    #[must_use]
    pub fn is_stem(&self) -> bool {
        matches!(
            self,
            Self::Mathematics
                | Self::Physics
                | Self::Chemistry
                | Self::Biology
                | Self::ComputerScience
                | Self::Engineering
                | Self::Statistics
        )
    }

    /// Check if healthcare discipline.
    #[must_use]
    pub fn is_healthcare(&self) -> bool {
        matches!(
            self,
            Self::Pharmacology | Self::Nursing | Self::Medicine | Self::PublicHealth
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COURSE LEVEL (λ Position)
// ═══════════════════════════════════════════════════════════════════════════════

/// Course level (year/advancement).
///
/// ## Tier: T2-P (λ primitive)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CourseLevel {
    /// 100-level: Freshman introductory
    Introductory = 1,
    /// 200-level: Sophomore intermediate
    Intermediate = 2,
    /// 300-level: Junior advanced
    Advanced = 3,
    /// 400-level: Senior capstone
    Capstone = 4,
    /// 500-level: Graduate/Masters
    Graduate = 5,
    /// 600-level: Doctoral/Research
    Doctoral = 6,
    /// 700+: Post-doctoral/Seminar
    PostDoctoral = 7,
}

impl CourseLevel {
    /// Parse from first digit of course number.
    pub fn from_digit(d: u8) -> AcademyResult<Self> {
        match d {
            0 | 1 => Ok(Self::Introductory),
            2 => Ok(Self::Intermediate),
            3 => Ok(Self::Advanced),
            4 => Ok(Self::Capstone),
            5 => Ok(Self::Graduate),
            6 => Ok(Self::Doctoral),
            7..=9 => Ok(Self::PostDoctoral),
            _ => Err(AcademyError::InvalidLevel(d)),
        }
    }

    /// Map to Prima tier (inverted — higher level = lower tier).
    ///
    /// Graduate+ research → T1 foundational
    /// Senior synthesis → T2-C composite
    /// Intro domain → T3 specific
    #[must_use]
    pub fn to_prima_tier(&self) -> PrimaTier {
        match self {
            Self::Introductory | Self::Intermediate => PrimaTier::T3,
            Self::Advanced | Self::Capstone => PrimaTier::T2C,
            Self::Graduate => PrimaTier::T2P,
            Self::Doctoral | Self::PostDoctoral => PrimaTier::T1,
        }
    }

    /// Get numeric level (1-7).
    #[must_use]
    pub fn as_number(&self) -> u8 {
        *self as u8
    }

    /// Get typical year designation.
    #[must_use]
    pub fn year_name(&self) -> &'static str {
        match self {
            Self::Introductory => "Freshman",
            Self::Intermediate => "Sophomore",
            Self::Advanced => "Junior",
            Self::Capstone => "Senior",
            Self::Graduate => "Graduate",
            Self::Doctoral => "Doctoral",
            Self::PostDoctoral => "Post-Doctoral",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRIMA TIER (Knowledge Abstraction)
// ═══════════════════════════════════════════════════════════════════════════════

/// Prima knowledge tier.
///
/// ## Grounding: κ (Comparison) — tiers are ordered
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PrimaTier {
    /// T1: Universal primitives (foundational theory)
    T1 = 1,
    /// T2-P: Cross-domain primitives (research methodology)
    T2P = 2,
    /// T2-C: Cross-domain composites (synthesis)
    T2C = 3,
    /// T3: Domain-specific (applications)
    T3 = 4,
}

impl PrimaTier {
    /// Transfer confidence for cross-domain mapping.
    #[must_use]
    pub fn transfer_confidence(&self) -> f64 {
        match self {
            Self::T1 => 1.0,  // Universal — transfers perfectly
            Self::T2P => 0.9, // Cross-domain primitive
            Self::T2C => 0.7, // Cross-domain composite
            Self::T3 => 0.4,  // Domain-specific — limited transfer
        }
    }

    /// Symbol representation.
    #[must_use]
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::T1 => "T1",
            Self::T2P => "T2-P",
            Self::T2C => "T2-C",
            Self::T3 => "T3",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COURSE (Full Identifier)
// ═══════════════════════════════════════════════════════════════════════════════

/// A complete course identifier.
///
/// ## Tier: T2-C (N + λ + σ + κ)
///
/// Combines subject (T3), level (λ), and number (σ).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Course {
    /// Subject domain.
    pub subject: Subject,
    /// Course number (e.g., 101, 350, 501).
    pub number: u16,
    /// Optional course title.
    pub title: Option<String>,
    /// Credit hours.
    pub credits: u8,
}

impl Course {
    /// Create a new course.
    #[must_use]
    pub fn new(subject: Subject, number: u16) -> Self {
        Self {
            subject,
            number,
            title: None,
            credits: 3, // Default
        }
    }

    /// Parse from string like "MTH 101" or "MTH101".
    pub fn parse(s: &str) -> AcademyResult<Self> {
        let s = s.trim().to_uppercase();

        // Find where letters end and digits begin
        let letter_end = s.chars().take_while(|c| c.is_alphabetic()).count();
        if letter_end == 0 {
            return Err(AcademyError::InvalidCode(s));
        }

        let subject_str = &s[..letter_end];
        let number_str: String = s[letter_end..]
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();

        if number_str.is_empty() {
            return Err(AcademyError::InvalidCode(s));
        }

        let number: u16 = number_str
            .parse()
            .map_err(|_| AcademyError::InvalidCode(s.clone()))?;

        let subject = Subject::from_code(subject_str);

        Ok(Self::new(subject, number))
    }

    /// Get course level.
    #[must_use]
    pub fn level(&self) -> CourseLevel {
        let first_digit = (self.number / 100) as u8;
        CourseLevel::from_digit(first_digit).unwrap_or(CourseLevel::Introductory)
    }

    /// Get Prima tier (inverted mapping).
    #[must_use]
    pub fn prima_tier(&self) -> PrimaTier {
        self.level().to_prima_tier()
    }

    /// Get canonical string representation.
    #[must_use]
    pub fn code(&self) -> String {
        format!("{} {}", self.subject.code(), self.number)
    }

    /// Check if graduate level.
    #[must_use]
    pub fn is_graduate(&self) -> bool {
        self.number >= 500
    }

    /// Check if can be prerequisite for another course.
    #[must_use]
    pub fn can_prereq(&self, other: &Course) -> bool {
        // Same subject, lower number
        self.subject == other.subject && self.number < other.number
    }

    /// Set title.
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set credits.
    #[must_use]
    pub fn with_credits(mut self, credits: u8) -> Self {
        self.credits = credits;
        self
    }
}

impl std::fmt::Display for Course {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CURRICULUM (μ Mapping)
// ═══════════════════════════════════════════════════════════════════════════════

/// A curriculum — collection of courses with prerequisites.
///
/// ## Tier: T2-C (μ + σ + →)
#[derive(Debug, Clone, Default)]
pub struct Curriculum {
    /// All courses in the curriculum.
    pub courses: Vec<Course>,
    /// Prerequisites: course index → required course indices.
    pub prerequisites: Vec<Vec<usize>>,
}

impl Curriculum {
    /// Create empty curriculum.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a course.
    pub fn add_course(&mut self, course: Course) -> usize {
        let idx = self.courses.len();
        self.courses.push(course);
        self.prerequisites.push(Vec::new());
        idx
    }

    /// Add prerequisite relationship.
    pub fn add_prereq(&mut self, course_idx: usize, prereq_idx: usize) {
        if course_idx < self.prerequisites.len() && prereq_idx < self.courses.len() {
            self.prerequisites[course_idx].push(prereq_idx);
        }
    }

    /// Get courses at a specific level.
    #[must_use]
    pub fn courses_at_level(&self, level: CourseLevel) -> Vec<&Course> {
        self.courses.iter().filter(|c| c.level() == level).collect()
    }

    /// Get courses in a subject.
    #[must_use]
    pub fn courses_in_subject(&self, subject: Subject) -> Vec<&Course> {
        self.courses
            .iter()
            .filter(|c| c.subject == subject)
            .collect()
    }

    /// Calculate total credits at each tier.
    #[must_use]
    pub fn credits_by_tier(&self) -> [(PrimaTier, u32); 4] {
        let mut counts = [
            (PrimaTier::T1, 0),
            (PrimaTier::T2P, 0),
            (PrimaTier::T2C, 0),
            (PrimaTier::T3, 0),
        ];

        for course in &self.courses {
            let tier = course.prima_tier();
            match tier {
                PrimaTier::T1 => counts[0].1 += course.credits as u32,
                PrimaTier::T2P => counts[1].1 += course.credits as u32,
                PrimaTier::T2C => counts[2].1 += course.credits as u32,
                PrimaTier::T3 => counts[3].1 += course.credits as u32,
            }
        }

        counts
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // Subject Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_subject_from_code() {
        assert_eq!(Subject::from_code("MTH"), Subject::Mathematics);
        assert_eq!(Subject::from_code("mth"), Subject::Mathematics);
        assert_eq!(Subject::from_code("MATH"), Subject::Mathematics);
        assert_eq!(Subject::from_code("PHY"), Subject::Physics);
        assert_eq!(Subject::from_code("CSC"), Subject::ComputerScience);
        assert_eq!(Subject::from_code("CS"), Subject::ComputerScience);
    }

    #[test]
    fn test_subject_is_stem() {
        assert!(Subject::Mathematics.is_stem());
        assert!(Subject::Physics.is_stem());
        assert!(Subject::ComputerScience.is_stem());
        assert!(!Subject::English.is_stem());
        assert!(!Subject::Philosophy.is_stem());
    }

    #[test]
    fn test_subject_is_healthcare() {
        assert!(Subject::Pharmacology.is_healthcare());
        assert!(Subject::Nursing.is_healthcare());
        assert!(Subject::Medicine.is_healthcare());
        assert!(!Subject::Mathematics.is_healthcare());
    }

    #[test]
    fn test_subject_code_roundtrip() {
        let subjects = [
            Subject::Mathematics,
            Subject::Physics,
            Subject::Chemistry,
            Subject::Biology,
            Subject::ComputerScience,
        ];
        for subj in subjects {
            let code = subj.code();
            let parsed = Subject::from_code(code);
            assert_eq!(parsed, subj);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CourseLevel Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_level_from_digit() {
        assert_eq!(
            CourseLevel::from_digit(1).ok(),
            Some(CourseLevel::Introductory)
        );
        assert_eq!(
            CourseLevel::from_digit(2).ok(),
            Some(CourseLevel::Intermediate)
        );
        assert_eq!(CourseLevel::from_digit(3).ok(), Some(CourseLevel::Advanced));
        assert_eq!(CourseLevel::from_digit(4).ok(), Some(CourseLevel::Capstone));
        assert_eq!(CourseLevel::from_digit(5).ok(), Some(CourseLevel::Graduate));
        assert_eq!(CourseLevel::from_digit(6).ok(), Some(CourseLevel::Doctoral));
    }

    #[test]
    fn test_level_to_prima_tier() {
        // Higher course level → lower (more abstract) Prima tier
        assert_eq!(CourseLevel::Introductory.to_prima_tier(), PrimaTier::T3);
        assert_eq!(CourseLevel::Intermediate.to_prima_tier(), PrimaTier::T3);
        assert_eq!(CourseLevel::Advanced.to_prima_tier(), PrimaTier::T2C);
        assert_eq!(CourseLevel::Capstone.to_prima_tier(), PrimaTier::T2C);
        assert_eq!(CourseLevel::Graduate.to_prima_tier(), PrimaTier::T2P);
        assert_eq!(CourseLevel::Doctoral.to_prima_tier(), PrimaTier::T1);
    }

    #[test]
    fn test_level_ordering() {
        assert!(CourseLevel::Introductory < CourseLevel::Intermediate);
        assert!(CourseLevel::Intermediate < CourseLevel::Advanced);
        assert!(CourseLevel::Advanced < CourseLevel::Capstone);
        assert!(CourseLevel::Capstone < CourseLevel::Graduate);
        assert!(CourseLevel::Graduate < CourseLevel::Doctoral);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PrimaTier Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_tier_transfer_confidence() {
        assert!((PrimaTier::T1.transfer_confidence() - 1.0).abs() < f64::EPSILON);
        assert!((PrimaTier::T2P.transfer_confidence() - 0.9).abs() < f64::EPSILON);
        assert!((PrimaTier::T2C.transfer_confidence() - 0.7).abs() < f64::EPSILON);
        assert!((PrimaTier::T3.transfer_confidence() - 0.4).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tier_ordering() {
        // Lower tier number = more abstract = should be "less" in ordering
        assert!(PrimaTier::T1 < PrimaTier::T2P);
        assert!(PrimaTier::T2P < PrimaTier::T2C);
        assert!(PrimaTier::T2C < PrimaTier::T3);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Course Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_course_parse_with_space() {
        let course = Course::parse("MTH 101")
            .ok()
            .unwrap_or_else(|| Course::new(Subject::Mathematics, 0));
        assert_eq!(course.subject, Subject::Mathematics);
        assert_eq!(course.number, 101);
    }

    #[test]
    fn test_course_parse_without_space() {
        let course = Course::parse("PHY201")
            .ok()
            .unwrap_or_else(|| Course::new(Subject::Physics, 0));
        assert_eq!(course.subject, Subject::Physics);
        assert_eq!(course.number, 201);
    }

    #[test]
    fn test_course_parse_lowercase() {
        let course = Course::parse("csc 350")
            .ok()
            .unwrap_or_else(|| Course::new(Subject::ComputerScience, 0));
        assert_eq!(course.subject, Subject::ComputerScience);
        assert_eq!(course.number, 350);
    }

    #[test]
    fn test_course_level() {
        assert_eq!(
            Course::new(Subject::Mathematics, 101).level(),
            CourseLevel::Introductory
        );
        assert_eq!(
            Course::new(Subject::Mathematics, 201).level(),
            CourseLevel::Intermediate
        );
        assert_eq!(
            Course::new(Subject::Mathematics, 350).level(),
            CourseLevel::Advanced
        );
        assert_eq!(
            Course::new(Subject::Mathematics, 450).level(),
            CourseLevel::Capstone
        );
        assert_eq!(
            Course::new(Subject::Mathematics, 501).level(),
            CourseLevel::Graduate
        );
        assert_eq!(
            Course::new(Subject::Mathematics, 650).level(),
            CourseLevel::Doctoral
        );
    }

    #[test]
    fn test_course_prima_tier() {
        let intro = Course::new(Subject::Mathematics, 101);
        let grad = Course::new(Subject::Mathematics, 550);

        assert_eq!(intro.prima_tier(), PrimaTier::T3);
        assert_eq!(grad.prima_tier(), PrimaTier::T2P);
    }

    #[test]
    fn test_course_is_graduate() {
        assert!(!Course::new(Subject::Physics, 450).is_graduate());
        assert!(Course::new(Subject::Physics, 500).is_graduate());
        assert!(Course::new(Subject::Physics, 650).is_graduate());
    }

    #[test]
    fn test_course_can_prereq() {
        let calc1 = Course::new(Subject::Mathematics, 151);
        let calc2 = Course::new(Subject::Mathematics, 152);
        let physics = Course::new(Subject::Physics, 151);

        assert!(calc1.can_prereq(&calc2)); // Same subject, lower number
        assert!(!calc2.can_prereq(&calc1)); // Higher can't be prereq of lower
        assert!(!calc1.can_prereq(&physics)); // Different subject
    }

    #[test]
    fn test_course_code() {
        let course = Course::new(Subject::Chemistry, 201);
        assert_eq!(course.code(), "CHM 201");
    }

    #[test]
    fn test_course_with_title() {
        let course = Course::new(Subject::Mathematics, 151).with_title("Calculus I");
        assert_eq!(course.title, Some("Calculus I".to_string()));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Curriculum Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_curriculum_add_courses() {
        let mut curriculum = Curriculum::new();
        let idx1 = curriculum.add_course(Course::new(Subject::Mathematics, 101));
        let idx2 = curriculum.add_course(Course::new(Subject::Mathematics, 102));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(curriculum.courses.len(), 2);
    }

    #[test]
    fn test_curriculum_courses_at_level() {
        let mut curriculum = Curriculum::new();
        curriculum.add_course(Course::new(Subject::Mathematics, 101));
        curriculum.add_course(Course::new(Subject::Physics, 101));
        curriculum.add_course(Course::new(Subject::Mathematics, 201));

        let intro = curriculum.courses_at_level(CourseLevel::Introductory);
        assert_eq!(intro.len(), 2);

        let intermediate = curriculum.courses_at_level(CourseLevel::Intermediate);
        assert_eq!(intermediate.len(), 1);
    }

    #[test]
    fn test_curriculum_courses_in_subject() {
        let mut curriculum = Curriculum::new();
        curriculum.add_course(Course::new(Subject::Mathematics, 101));
        curriculum.add_course(Course::new(Subject::Mathematics, 201));
        curriculum.add_course(Course::new(Subject::Physics, 101));

        let math = curriculum.courses_in_subject(Subject::Mathematics);
        assert_eq!(math.len(), 2);

        let physics = curriculum.courses_in_subject(Subject::Physics);
        assert_eq!(physics.len(), 1);
    }

    #[test]
    fn test_curriculum_credits_by_tier() {
        let mut curriculum = Curriculum::new();
        curriculum.add_course(Course::new(Subject::Mathematics, 101).with_credits(3)); // T3
        curriculum.add_course(Course::new(Subject::Mathematics, 201).with_credits(3)); // T3
        curriculum.add_course(Course::new(Subject::Mathematics, 350).with_credits(3)); // T2-C
        curriculum.add_course(Course::new(Subject::Mathematics, 550).with_credits(3)); // T2-P

        let credits = curriculum.credits_by_tier();
        assert_eq!(credits[3].1, 6); // T3: 3+3=6
        assert_eq!(credits[2].1, 3); // T2-C: 3
        assert_eq!(credits[1].1, 3); // T2-P: 3
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Knowledge Funnel Tests (Integration)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_knowledge_funnel_inversion() {
        // Verify the knowledge funnel: higher academic level → lower Prima tier
        let courses = [
            Course::new(Subject::Mathematics, 101), // Freshman intro
            Course::new(Subject::Mathematics, 201), // Sophomore
            Course::new(Subject::Mathematics, 301), // Junior
            Course::new(Subject::Mathematics, 401), // Senior
            Course::new(Subject::Mathematics, 501), // Graduate
            Course::new(Subject::Mathematics, 601), // Doctoral
        ];

        let tiers: Vec<PrimaTier> = courses.iter().map(|c| c.prima_tier()).collect();

        // T3 → T3 → T2-C → T2-C → T2-P → T1 (descending abstraction)
        assert_eq!(tiers[0], PrimaTier::T3);
        assert_eq!(tiers[1], PrimaTier::T3);
        assert_eq!(tiers[2], PrimaTier::T2C);
        assert_eq!(tiers[3], PrimaTier::T2C);
        assert_eq!(tiers[4], PrimaTier::T2P);
        assert_eq!(tiers[5], PrimaTier::T1);
    }

    #[test]
    fn test_transfer_confidence_gradient() {
        // As you go up in academic level, transfer confidence increases
        let freshman = Course::new(Subject::Pharmacology, 101).prima_tier();
        let graduate = Course::new(Subject::Pharmacology, 550).prima_tier();
        let doctoral = Course::new(Subject::Pharmacology, 650).prima_tier();

        assert!(doctoral.transfer_confidence() > graduate.transfer_confidence());
        assert!(graduate.transfer_confidence() > freshman.transfer_confidence());
    }
}
