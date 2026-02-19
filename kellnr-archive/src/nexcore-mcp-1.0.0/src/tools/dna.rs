//! DNA-based Computation MCP tools.
//!
//! Wraps `nexcore-dna` public API: encode/decode text as DNA, evaluate
//! expressions on the 64-instruction Codon VM, generate tile/voxel
//! visualizations, detect PV signals via DNA math, and inspect the ISA.
//!
//! Tier: T3 (σ + μ + ∂ + ς + N + κ + → + ∃)

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    DnaCompileAsmParams, DnaDecodeParams, DnaEncodeParams, DnaEvalParams, DnaProfileDrugParams,
    DnaPvSignalParams, DnaTileParams, DnaVoxelParams,
};

// ============================================================================
// dna_encode — Encode text as a DNA nucleotide strand
// ============================================================================

/// Encode a UTF-8 string as a DNA strand (A/T/G/C).
///
/// Each byte maps to 4 nucleotides (2 bits each). Prefixed with a 16-nucleotide
/// length header. Reversible via `dna_decode`.
pub fn dna_encode(p: DnaEncodeParams) -> Result<CallToolResult, McpError> {
    let strand = nexcore_dna::storage::encode_str(&p.text);
    let display = strand.to_string_repr();

    let result = serde_json::json!({
        "text": p.text,
        "strand": display,
        "length_nucleotides": strand.bases.len(),
        "length_bytes": p.text.len(),
        "encoding": "2-bit quaternary (A=00, T=01, G=10, C=11)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_decode — Decode a DNA strand back to text
// ============================================================================

/// Decode a DNA nucleotide strand back to a UTF-8 string.
pub fn dna_decode(p: DnaDecodeParams) -> Result<CallToolResult, McpError> {
    let strand = nexcore_dna::types::Strand::parse(&p.strand)
        .map_err(|e| McpError::invalid_params(format!("Invalid DNA strand: {e}"), None))?;

    let text = nexcore_dna::storage::decode_str(&strand)
        .map_err(|e| McpError::invalid_params(format!("Decode failed: {e}"), None))?;

    let result = serde_json::json!({
        "strand": p.strand,
        "text": text,
        "length_nucleotides": strand.bases.len(),
        "length_bytes": text.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_eval — Compile and execute an expression on the Codon VM
// ============================================================================

/// Compile source code and execute it on the 64-instruction Codon VM.
///
/// Returns output buffer, final stack state, cycle count, and halt reason.
pub fn dna_eval(p: DnaEvalParams) -> Result<CallToolResult, McpError> {
    let vm_result = nexcore_dna::lang::compiler::eval(&p.expr)
        .map_err(|e| McpError::invalid_params(format!("Compilation/execution error: {e}"), None))?;

    let result = serde_json::json!({
        "source": p.expr,
        "output": vm_result.output,
        "stack": vm_result.stack,
        "cycles": vm_result.cycles,
        "halt_reason": format!("{:?}", vm_result.halt_reason),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_compile_asm — Compile source to assembly text
// ============================================================================

/// Compile source code to assembly text for inspection.
pub fn dna_compile_asm(p: DnaCompileAsmParams) -> Result<CallToolResult, McpError> {
    let asm = nexcore_dna::lang::compiler::compile_to_asm(&p.source)
        .map_err(|e| McpError::invalid_params(format!("Compilation error: {e}"), None))?;

    let result = serde_json::json!({
        "source": p.source,
        "assembly": asm,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_tile — Generate an 8×8 pixel tile visualization
// ============================================================================

/// Compile source code and visualize the program as an 8×8 pixel tile.
///
/// Each instruction maps to an RGBA pixel where R=family, G=variant,
/// B=literal, A=confidence.
pub fn dna_tile(p: DnaTileParams) -> Result<CallToolResult, McpError> {
    let program = nexcore_dna::lang::compiler::compile(&p.expr)
        .map_err(|e| McpError::invalid_params(format!("Compilation error: {e}"), None))?;

    // Decode codons from the compiled strand → instructions
    let codons = program
        .code
        .codons()
        .map_err(|e| McpError::invalid_params(format!("Codon extraction error: {e}"), None))?;
    let instructions: Vec<nexcore_dna::isa::Instruction> =
        codons.iter().map(nexcore_dna::isa::decode).collect();

    let tile = nexcore_dna::tile::Tile::from_instructions(&instructions);

    // Build a text representation of the tile grid
    let mut grid = Vec::new();
    for row in 0..8 {
        let mut row_data = Vec::new();
        for col in 0..8 {
            let px = &tile.pixels[row][col];
            row_data.push(serde_json::json!({
                "r": px.r,
                "g": px.g,
                "b": px.b,
                "a": px.a,
            }));
        }
        grid.push(row_data);
    }

    let result = serde_json::json!({
        "source": p.expr,
        "dimensions": "8x8",
        "instruction_count": instructions.len(),
        "grid": grid,
        "layout": {
            "rows_0_5": "PROGRAM (up to 48 instruction pixels)",
            "row_6": "SPECIFICATION",
            "row_7": "CHECKSUM",
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_voxel — Generate a 4×4×4 voxel cube visualization
// ============================================================================

/// Generate a 4×4×4 voxel cube mapping program instructions
/// into 3D chemical space (charge × energy × stability).
///
/// Returns concentrations at each voxel and Beer-Lambert projections
/// along all three axes.
pub fn dna_voxel(p: DnaVoxelParams) -> Result<CallToolResult, McpError> {
    let program = nexcore_dna::lang::compiler::compile(&p.expr)
        .map_err(|e| McpError::invalid_params(format!("Compilation error: {e}"), None))?;

    // Decode codons from the compiled strand → instructions
    let codons = program
        .code
        .codons()
        .map_err(|e| McpError::invalid_params(format!("Codon extraction error: {e}"), None))?;
    let instructions: Vec<nexcore_dna::isa::Instruction> =
        codons.iter().map(nexcore_dna::isa::decode).collect();

    let cube = nexcore_dna::voxel::VoxelCube::from_instructions(&instructions);

    // Collect occupied voxels (concentration > 0)
    let mut occupied = Vec::new();
    for x in 0..4u8 {
        for y in 0..4u8 {
            for z in 0..4u8 {
                let pos = nexcore_dna::voxel::VoxelPos { x, y, z };
                let conc = cube.concentration_at(pos);
                if conc > 0.0 {
                    occupied.push(serde_json::json!({
                        "position": [x, y, z],
                        "concentration": conc,
                    }));
                }
            }
        }
    }

    // Beer-Lambert projections (4×4 absorbance images)
    let proj_x = cube.project_x();
    let proj_y = cube.project_y();
    let proj_z = cube.project_z();

    let result = serde_json::json!({
        "source": p.expr,
        "dimensions": "4x4x4",
        "axes": {
            "x": "Charge (negative → positive)",
            "y": "Energy (ground → ionized)",
            "z": "Stability (stable → volatile)",
        },
        "total_concentration": cube.total_concentration(),
        "total_absorbance": cube.total_absorbance(),
        "occupied_voxels": occupied.len(),
        "voxels": occupied,
        "projections": {
            "x_axis": proj_x,
            "y_axis": proj_y,
            "z_axis": proj_z,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_pv_signal — Detect a PV signal between drug and event via DNA math
// ============================================================================

/// Detect a pharmacovigilance signal between a drug and an adverse event
/// using DNA-based string theory (distance + resonance in word-space).
pub fn dna_pv_signal(p: DnaPvSignalParams) -> Result<CallToolResult, McpError> {
    let signal = nexcore_dna::pv_theory::detect_signal(&p.drug, &p.event);

    let result = serde_json::json!({
        "drug": signal.drug,
        "event": signal.event,
        "strength": signal.strength,
        "resonance": signal.resonance,
        "combined_score": signal.combined_score,
        "detected": signal.detected,
        "algorithm": "DNA string-theory: 1/(1+d) × (0.5 + 0.5×resonance)",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_profile_drug — Profile a word as a drug compound via DNA properties
// ============================================================================

/// Profile a word/name as a drug compound using DNA-derived properties:
/// 3D mind-space position, string tension, energy, entropy, GC content.
pub fn dna_profile_drug(p: DnaProfileDrugParams) -> Result<CallToolResult, McpError> {
    let profile = nexcore_dna::pv_theory::profile_drug(&p.name);

    let result = serde_json::json!({
        "name": profile.name,
        "position": {
            "entropy_norm": profile.position.entropy_norm,
            "gc_content": profile.position.gc_content,
            "density": profile.position.density,
        },
        "tension": {
            "mean_tension": profile.tension.mean_tension,
            "gc_tension_ratio": profile.tension.gc_tension_ratio,
            "peak_tension": profile.tension.peak_tension,
            "variance": profile.tension.variance,
        },
        "energy": {
            "total_energy": profile.energy.total_energy,
            "tension_energy": profile.energy.tension_energy,
            "information_energy": profile.energy.information_energy,
            "energy_level": profile.energy.energy_level,
            "energy_density": profile.energy.energy_density,
        },
        "entropy": profile.entropy,
        "gc_content": profile.gc_content,
        "molecular_weight": profile.molecular_weight,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_nexcore_genome — NexCore signal detection algorithms as DNA
// ============================================================================

/// Return the NexCore signal detection genome: 4 algorithms (PRR, ROR, IC, EBGM)
/// encoded as a unified DNA strand with individual gene metadata.
pub fn dna_nexcore_genome() -> Result<CallToolResult, McpError> {
    let programs = nexcore_dna::nexcore_encoding::signal_detection_programs()
        .map_err(|e| McpError::invalid_params(format!("Genome compilation error: {e}"), None))?;

    let genome_strand = nexcore_dna::nexcore_encoding::nexcore_genome_strand()
        .map_err(|e| McpError::invalid_params(format!("Genome strand error: {e}"), None))?;

    let genes: Vec<serde_json::Value> = programs
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "formula": p.formula,
                "strand_length": p.strand.len(),
            })
        })
        .collect();

    // Type registry: 5 foundation types as DnaRecords
    let registry = nexcore_dna::nexcore_encoding::type_registry();
    let types: Vec<serde_json::Value> = registry
        .iter()
        .map(|(name, record)| {
            serde_json::json!({
                "name": name,
                "fields": record.fields.len(),
                "display": nexcore_dna::data::transcribe_record(record),
            })
        })
        .collect();

    // Guardian thresholds
    let guardian = nexcore_dna::nexcore_encoding::guardian_thresholds_record();
    let guardian_display = nexcore_dna::data::transcribe_record(&guardian);

    let result = serde_json::json!({
        "genome_strand": genome_strand,
        "genome_strand_length": genome_strand.len(),
        "algorithms": genes,
        "algorithm_count": 4,
        "type_registry": types,
        "guardian_thresholds": guardian_display,
        "pipeline": "NexCore concepts -> DNA source -> Codon VM bytecode -> ATGC strand",
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ============================================================================
// dna_catalog — Full 64-instruction ISA reference
// ============================================================================

/// Return the complete 64-instruction Codon VM catalog.
///
/// Each entry: index, mnemonic, codon nucleotides, amino acid, stack effect.
pub fn dna_catalog() -> Result<CallToolResult, McpError> {
    let entries = nexcore_dna::isa::catalog();

    let catalog: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "index": e.index,
                "mnemonic": e.mnemonic,
                "codon": e.codon_str,
                "amino_acid": e.amino_acid,
                "stack_effect": e.stack_effect,
            })
        })
        .collect();

    let result = serde_json::json!({
        "total_instructions": 64,
        "families": 8,
        "instructions_per_family": 8,
        "isa_version": "v3",
        "catalog": catalog,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
