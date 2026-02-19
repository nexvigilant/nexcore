//! Molecular Biology MCP tools
//!
//! Maps Prima compilation to the Central Dogma of Molecular Biology.
//! Exposes codon translation, mRNA → protein, ADME phases, and Central Dogma stages.
//!
//! ## Tier: T2-C (μ + σ + → + ρ)

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    MolecularAdmePhaseParams, MolecularCentralDogmaParams, MolecularTranslateCodonParams,
    MolecularTranslateMrnaParams,
};

// ============================================================================
// Genetic Code (64 codons → 20 amino acids + Stop)
// ============================================================================

/// Standard genetic code mapping codon → amino acid single-letter code.
fn translate_codon_internal(codon: &str) -> Option<char> {
    match codon.to_uppercase().as_str() {
        "UUU" | "UUC" => Some('F'),
        "UUA" | "UUG" | "CUU" | "CUC" | "CUA" | "CUG" => Some('L'),
        "AUU" | "AUC" | "AUA" => Some('I'),
        "AUG" => Some('M'),
        "GUU" | "GUC" | "GUA" | "GUG" => Some('V'),
        "UCU" | "UCC" | "UCA" | "UCG" | "AGU" | "AGC" => Some('S'),
        "CCU" | "CCC" | "CCA" | "CCG" => Some('P'),
        "ACU" | "ACC" | "ACA" | "ACG" => Some('T'),
        "GCU" | "GCC" | "GCA" | "GCG" => Some('A'),
        "UAU" | "UAC" => Some('Y'),
        "CAU" | "CAC" => Some('H'),
        "CAA" | "CAG" => Some('Q'),
        "AAU" | "AAC" => Some('N'),
        "AAA" | "AAG" => Some('K'),
        "GAU" | "GAC" => Some('D'),
        "GAA" | "GAG" => Some('E'),
        "UGU" | "UGC" => Some('C'),
        "UGG" => Some('W'),
        "CGU" | "CGC" | "CGA" | "CGG" | "AGA" | "AGG" => Some('R'),
        "GGU" | "GGC" | "GGA" | "GGG" => Some('G'),
        "UAA" | "UAG" | "UGA" => Some('*'),
        _ => None,
    }
}

/// Get amino acid full name from single-letter code.
fn amino_acid_name(code: char) -> &'static str {
    match code {
        'A' => "Alanine",
        'C' => "Cysteine",
        'D' => "Aspartic acid",
        'E' => "Glutamic acid",
        'F' => "Phenylalanine",
        'G' => "Glycine",
        'H' => "Histidine",
        'I' => "Isoleucine",
        'K' => "Lysine",
        'L' => "Leucine",
        'M' => "Methionine",
        'N' => "Asparagine",
        'P' => "Proline",
        'Q' => "Glutamine",
        'R' => "Arginine",
        'S' => "Serine",
        'T' => "Threonine",
        'V' => "Valine",
        'W' => "Tryptophan",
        'Y' => "Tyrosine",
        '*' => "Stop",
        _ => "Unknown",
    }
}

// ============================================================================
// Tool: translate_codon
// ============================================================================

fn codon_success_result(codon: &str, aa: char) -> serde_json::Value {
    serde_json::json!({
        "codon": codon,
        "amino_acid_code": aa.to_string(),
        "amino_acid_name": amino_acid_name(aa),
        "is_start": codon == "AUG",
        "is_stop": aa == '*',
        "prima_mapping": {
            "biology_term": "codon",
            "prima_equivalent": "c[N₁,N₂,N₃]",
            "tier": "T2-P"
        }
    })
}

/// Translate a single codon (3 nucleotides) to amino acid.
pub fn translate_codon(params: MolecularTranslateCodonParams) -> Result<CallToolResult, McpError> {
    let codon = params.codon.trim().to_uppercase();

    if codon.len() != 3 {
        let result =
            serde_json::json!({"error": "Codon must be 3 nucleotides", "length": codon.len()});
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let result = match translate_codon_internal(&codon) {
        Some(aa) => codon_success_result(&codon, aa),
        None => serde_json::json!({"error": "Invalid codon", "codon": codon}),
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Tool: translate_mrna
// ============================================================================

fn do_mrna_translation(mrna: &str, start: usize) -> (String, usize, bool) {
    let mut protein = String::new();
    let mut count = 0;
    let mut stopped = false;
    let bytes = mrna.as_bytes();
    let mut pos = start;

    while pos + 3 <= bytes.len() {
        let codon = String::from_utf8_lossy(&bytes[pos..pos + 3]);
        if let Some(aa) = translate_codon_internal(&codon) {
            if aa == '*' {
                stopped = true;
                break;
            }
            protein.push(aa);
            count += 1;
        }
        pos += 3;
    }
    (protein, count, stopped)
}

/// Translate mRNA sequence to protein string.
pub fn translate_mrna(params: MolecularTranslateMrnaParams) -> Result<CallToolResult, McpError> {
    let mrna = params.mrna.trim().to_uppercase().replace(' ', "");
    let start_pos = mrna.find("AUG");
    let translation_start = if params.from_start.unwrap_or(false) {
        start_pos.unwrap_or(0)
    } else {
        0
    };

    let (protein, count, stopped) = do_mrna_translation(&mrna, translation_start);

    let result = serde_json::json!({
        "mrna_length": mrna.len(),
        "protein": protein,
        "protein_length": protein.len(),
        "codons_translated": count,
        "start_codon_position": start_pos,
        "stopped_at_stop_codon": stopped,
        "prima_mapping": {"biology_term": "translation", "prima_equivalent": "μ[AST→Value]", "tier": "T2-C"}
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Tool: central_dogma
// ============================================================================

fn dogma_transcription() -> serde_json::Value {
    serde_json::json!({
        "biology_stage": "Transcription",
        "prima_stage": "Lexer/Parser",
        "prima_equivalent": "μ[source→AST]",
        "key_enzyme": "RNA Polymerase",
        "tier": "T2-P"
    })
}

fn dogma_translation() -> serde_json::Value {
    serde_json::json!({
        "biology_stage": "Translation",
        "prima_stage": "Interpreter",
        "prima_equivalent": "μ[AST→Value]",
        "key_enzyme": "Ribosome",
        "tier": "T2-P"
    })
}

fn dogma_folding() -> serde_json::Value {
    serde_json::json!({
        "biology_stage": "Protein Folding",
        "prima_stage": "Type Inference",
        "prima_equivalent": "μ[Type→Type]",
        "tier": "T2-C"
    })
}

fn dogma_replication() -> serde_json::Value {
    serde_json::json!({
        "biology_stage": "Replication",
        "prima_stage": "Homoiconicity",
        "prima_equivalent": "ρ[self→self]",
        "tier": "T1 (ρ)"
    })
}

fn dogma_proofreading() -> serde_json::Value {
    serde_json::json!({
        "biology_stage": "Proofreading",
        "prima_stage": "Type Checking",
        "prima_equivalent": "κ[T₁,T₂]→Bool",
        "tier": "T2-P"
    })
}

fn dogma_full_table() -> serde_json::Value {
    serde_json::json!({
        "central_dogma_mapping": [
            {"biology": "Transcription", "prima": "μ[source→AST]", "stage": "Lexer/Parser"},
            {"biology": "Translation", "prima": "μ[AST→Value]", "stage": "Interpreter"},
            {"biology": "Folding", "prima": "μ[Type→Type]", "stage": "Type Inference"},
            {"biology": "Replication", "prima": "ρ[self→self]", "stage": "Homoiconicity"},
            {"biology": "Proofreading", "prima": "κ[T₁,T₂]→Bool", "stage": "Type Checking"}
        ]
    })
}

/// Map Central Dogma stages to Prima compilation phases.
pub fn central_dogma(params: MolecularCentralDogmaParams) -> Result<CallToolResult, McpError> {
    let stage = params.stage.to_lowercase();
    let mapping = match stage.as_str() {
        "transcription" => dogma_transcription(),
        "translation" => dogma_translation(),
        "folding" => dogma_folding(),
        "replication" => dogma_replication(),
        "proofreading" => dogma_proofreading(),
        _ => dogma_full_table(),
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&mapping).unwrap_or_default(),
    )]))
}

// ============================================================================
// Tool: adme_phase
// ============================================================================

fn adme_absorption() -> serde_json::Value {
    serde_json::json!({
        "phase": "Absorption", "symbol": "A",
        "prima_equivalent": "π:read", "primitive": "π (Persistence)",
        "computation_analog": "Input / IO read", "tier": "T1"
    })
}

fn adme_distribution() -> serde_json::Value {
    serde_json::json!({
        "phase": "Distribution", "symbol": "D",
        "prima_equivalent": "ς:alloc", "primitive": "ς (State)",
        "computation_analog": "Memory allocation", "tier": "T1"
    })
}

fn adme_metabolism() -> serde_json::Value {
    serde_json::json!({
        "phase": "Metabolism", "symbol": "M",
        "prima_equivalent": "→:compute", "primitive": "→ (Causality)",
        "computation_analog": "Transformation", "tier": "T1"
    })
}

fn adme_elimination() -> serde_json::Value {
    serde_json::json!({
        "phase": "Elimination", "symbol": "E",
        "prima_equivalent": "∅:free", "primitive": "∅ (Void)",
        "computation_analog": "GC / Return", "tier": "T1"
    })
}

fn adme_full_table() -> serde_json::Value {
    serde_json::json!({
        "adme_lifecycle": [
            {"phase": "A", "name": "Absorption", "prima": "π:read", "primitive": "π"},
            {"phase": "D", "name": "Distribution", "prima": "ς:alloc", "primitive": "ς"},
            {"phase": "M", "name": "Metabolism", "prima": "→:compute", "primitive": "→"},
            {"phase": "E", "name": "Elimination", "prima": "∅:free", "primitive": "∅"}
        ],
        "transfer_confidence": 0.85
    })
}

/// Map ADME pharmacokinetic phases to Prima computation lifecycle.
pub fn adme_phase(params: MolecularAdmePhaseParams) -> Result<CallToolResult, McpError> {
    let phase = params.phase.to_uppercase();
    let mapping = match phase.as_str() {
        "A" | "ABSORPTION" => adme_absorption(),
        "D" | "DISTRIBUTION" => adme_distribution(),
        "M" | "METABOLISM" => adme_metabolism(),
        "E" | "ELIMINATION" => adme_elimination(),
        _ => adme_full_table(),
    };

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&mapping).unwrap_or_default(),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_codon_start() {
        assert_eq!(translate_codon_internal("AUG"), Some('M'));
    }

    #[test]
    fn test_translate_codon_stop() {
        assert_eq!(translate_codon_internal("UAA"), Some('*'));
    }

    #[test]
    fn test_amino_acid_name() {
        assert_eq!(amino_acid_name('M'), "Methionine");
    }

    #[test]
    fn test_mrna_translation() {
        let (protein, count, stopped) = do_mrna_translation("AUGUUUUAA", 0);
        assert_eq!(protein, "MF");
        assert_eq!(count, 2);
        assert!(stopped);
    }
}
