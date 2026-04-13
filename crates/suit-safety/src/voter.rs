//! Voting logic for redundant safety channels.

/// Triple-modular redundancy voter.
#[derive(Debug, Clone)]
pub struct TmrVoter<T> {
    /// Channel A value.
    pub a: T,
    /// Channel B value.
    pub b: T,
    /// Channel C value.
    pub c: T,
}

impl<T: PartialEq + Clone> TmrVoter<T> {
    /// Create a new TMR voter with three channel values.
    pub fn new(a: T, b: T, c: T) -> Self {
        Self { a, b, c }
    }

    /// Vote: returns the majority value if two or more channels agree.
    pub fn vote(&self) -> Option<T> {
        if self.a == self.b || self.a == self.c {
            Some(self.a.clone())
        } else if self.b == self.c {
            Some(self.b.clone())
        } else {
            None
        }
    }
}
