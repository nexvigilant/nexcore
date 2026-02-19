//! Review Protocol - T3 Domain-Specific
//! 5-stage validation: Generated→Compiled→Linted→SpotChecked→Accepted

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum ReviewPhase {
    #[default]
    Generated = 0,
    Compiled = 1,
    Linted = 2,
    SpotChecked = 3,
    Accepted = 4,
}

impl ReviewPhase {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Generated => Some(Self::Compiled),
            Self::Compiled => Some(Self::Linted),
            Self::Linted => Some(Self::SpotChecked),
            Self::SpotChecked => Some(Self::Accepted),
            Self::Accepted => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub phase: ReviewPhase,
    pub passed: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewProtocol {
    pub current: ReviewPhase,
    pub results: Vec<ReviewResult>,
    pub retries: u8,
    pub max_retries: u8,
}

impl ReviewProtocol {
    pub fn new(max_retries: u8) -> Self {
        Self {
            current: ReviewPhase::Generated,
            results: vec![],
            retries: 0,
            max_retries,
        }
    }

    pub fn advance(&mut self, result: ReviewResult) -> Option<ReviewPhase> {
        let passed = result.passed;
        self.results.push(result);
        if passed {
            self.current.next().map(|p| {
                self.current = p;
                p
            })
        } else if self.retries < self.max_retries {
            self.retries += 1;
            Some(self.current)
        } else {
            None
        }
    }

    pub fn accepted(&self) -> bool {
        self.current == ReviewPhase::Accepted
    }
}
