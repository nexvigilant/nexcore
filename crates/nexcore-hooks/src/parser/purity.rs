//! # UACA Purity Parser
//!
//! Objectively measures code against the UACA Level Purity rules:
//! - L1 Atoms: Max 20 LOC
//! - L2 Molecules: Max 50 LOC

/// Metrics for a Rust file
#[derive(Debug, Clone)]
pub struct PurityMetrics {
    pub logical_loc: usize,
    pub level: u8,
}

/// Analyze the purity of Rust content
pub fn analyze_purity(content: &str, file_path: &str) -> PurityMetrics {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
        .collect();

    let level = if file_path.contains("atoms") || file_path.contains("/atoms/") {
        1
    } else if file_path.contains("molecules") || file_path.contains("detector") {
        2
    } else {
        3 // Organism or higher
    };

    PurityMetrics {
        logical_loc: lines.len(),
        level,
    }
}
