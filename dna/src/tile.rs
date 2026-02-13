//! 2D Pixel Tile encoding for DNA programs.
//!
//! An 8×8 tile encodes a program as RGBA pixels where color carries
//! semantic information from the instruction set.
//!
//! ## Pixel Layout (32 bits RGBA)
//!
//! ```text
//! R[7:5] = P0 (glyph family, 3 bits)     — instruction category
//! R[4:0] = reserved (5 bits, zero)
//! G[7:5] = P1 (glyph variant, 3 bits)    — specific operation
//! G[4:0] = reserved (5 bits, zero)
//! B[7:0] = literal operand (8 bits)       — for Lit instructions
//! A[7:0] = confidence (8 bits, 255=full)  — epistemic state
//! ```
//!
//! The glyph family (P0) determines the pixel's dominant hue, making
//! instruction categories visually distinct in the tile.
//!
//! ## Tile Structure
//!
//! ```text
//! Rows 0-5: PROGRAM (48 instruction pixels)
//! Row 6:    SPECIFICATION (8 spec pixels)
//! Row 7:    CHECKSUM (8 hash pixels)
//! ```
//!
//! Tier: T2-C (σ Sequence + μ Mapping + ∂ Boundary + λ Location)

use crate::isa::{self, Instruction};

// ---------------------------------------------------------------------------
// Pixel
// ---------------------------------------------------------------------------

/// A single RGBA pixel encoding one instruction.
///
/// Tier: T2-P (μ Mapping + ∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    /// Red: P0 (family) in upper 3 bits, reserved in lower 5.
    pub r: u8,
    /// Green: P1 (variant) in upper 3 bits, reserved in lower 5.
    pub g: u8,
    /// Blue: literal operand (0 for non-literal instructions).
    pub b: u8,
    /// Alpha: confidence (0=uncertain, 255=full confidence).
    pub a: u8,
}

impl Pixel {
    /// Black transparent pixel (empty slot).
    pub const EMPTY: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    /// Create a pixel from an instruction with full confidence.
    #[must_use]
    pub fn from_instruction(instr: &Instruction) -> Self {
        match instr {
            Instruction::Lit(n) => {
                // Lit: P0=7 (N/Quantity family), P1=7, B=lower 8 bits of value
                let b = (*n & 0xFF) as u8;
                Self {
                    r: 7 << 5,
                    g: 7 << 5,
                    b,
                    a: 255,
                }
            }
            other => {
                if let Some(idx) = codon_index(other) {
                    let p0 = idx / 8;
                    let p1 = idx % 8;
                    Self {
                        r: p0 << 5,
                        g: p1 << 5,
                        b: 0,
                        a: 255,
                    }
                } else {
                    Self::EMPTY
                }
            }
        }
    }

    /// Decode a pixel back to an instruction.
    ///
    /// Returns `None` if the pixel is empty (alpha = 0).
    #[must_use]
    pub fn to_instruction(&self) -> Option<Instruction> {
        if self.a == 0 {
            return None;
        }

        let p0 = self.r >> 5;
        let p1 = self.g >> 5;

        // Lit encoding: P0=7, P1=7, B=value
        if p0 == 7 && p1 == 7 {
            return Some(Instruction::Lit(i64::from(self.b)));
        }

        let idx = p0 * 8 + p1;
        Some(isa::decode_index(idx))
    }

    /// Get the glyph family (P0) index: 0-7.
    #[must_use]
    pub fn family(&self) -> u8 {
        self.r >> 5
    }

    /// Get the glyph variant (P1) index: 0-7.
    #[must_use]
    pub fn variant(&self) -> u8 {
        self.g >> 5
    }

    /// Get confidence as a 0.0-1.0 float.
    #[must_use]
    pub fn confidence(&self) -> f64 {
        f64::from(self.a) / 255.0
    }

    /// Set confidence from a 0.0-1.0 float.
    #[must_use]
    pub fn with_confidence(mut self, conf: f64) -> Self {
        self.a = (conf.clamp(0.0, 1.0) * 255.0) as u8;
        self
    }

    /// Approximate RGB color for visualization.
    ///
    /// Maps glyph families to biologically-inspired hues:
    /// - P0=0 (σ data):     gray   (nonpolar, interior)
    /// - P0=1 (μ transform): green  (polar, surface)
    /// - P0=2 (ς state):    amber  (memory, warm)
    /// - P0=3 (ρ recurse):  teal   (cyclic)
    /// - P0=4 (∂ boundary): red    (acidic, lifecycle)
    /// - P0=5 (→ control):  blue   (basic, high-energy)
    /// - P0=6 (κ compare):  purple (aromatic, testing)
    /// - P0=7 (N quantity): yellow (constants, special)
    #[must_use]
    pub fn display_color(&self) -> (u8, u8, u8) {
        let base = match self.family() {
            0 => (160, 160, 160), // gray
            1 => (80, 200, 80),   // green
            2 => (220, 160, 40),  // amber
            3 => (40, 180, 180),  // teal
            4 => (220, 60, 60),   // red
            5 => (60, 100, 220),  // blue
            6 => (160, 60, 200),  // purple
            7 => (220, 200, 40),  // yellow
            _ => (0, 0, 0),
        };

        // Modulate brightness by variant: higher variant = slightly brighter
        let v = self.variant();
        let scale = |c: u8| -> u8 {
            let adjusted = i32::from(c) + i32::from(v) * 4;
            adjusted.clamp(0, 255) as u8
        };

        (scale(base.0), scale(base.1), scale(base.2))
    }
}

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

/// An 8×8 program tile: 64 pixels.
///
/// Layout:
/// - Rows 0-5 (48 pixels): program instructions
/// - Row 6 (8 pixels): specification / assertions
/// - Row 7 (8 pixels): output checksum
///
/// Tier: T2-C (σ Sequence + ∂ Boundary + λ Location + μ Mapping)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub pixels: [[Pixel; 8]; 8],
}

impl Tile {
    /// Create an empty tile (all transparent black).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            pixels: [[Pixel::EMPTY; 8]; 8],
        }
    }

    /// Encode a sequence of instructions into a tile.
    ///
    /// Instructions fill rows 0-5 (up to 48). Excess instructions are truncated.
    /// Rows 6-7 remain empty (for spec/checksum).
    #[must_use]
    pub fn from_instructions(instrs: &[Instruction]) -> Self {
        let mut tile = Self::empty();
        for (i, instr) in instrs.iter().take(48).enumerate() {
            let row = i / 8;
            let col = i % 8;
            tile.pixels[row][col] = Pixel::from_instruction(instr);
        }
        tile
    }

    /// Decode the program region (rows 0-5) back to instructions.
    ///
    /// Empty pixels (alpha=0) are skipped.
    #[must_use]
    pub fn to_instructions(&self) -> Vec<Instruction> {
        let mut instrs = Vec::new();
        for row in 0..6 {
            for col in 0..8 {
                if let Some(instr) = self.pixels[row][col].to_instruction() {
                    instrs.push(instr);
                }
            }
        }
        instrs
    }

    /// Get a pixel at (row, col).
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<&Pixel> {
        self.pixels.get(row).and_then(|r| r.get(col))
    }

    /// Set a pixel at (row, col).
    pub fn set(&mut self, row: usize, col: usize, pixel: Pixel) {
        if row < 8 && col < 8 {
            self.pixels[row][col] = pixel;
        }
    }

    /// Count non-empty pixels in the program region (rows 0-5).
    #[must_use]
    pub fn instruction_count(&self) -> usize {
        let mut count = 0;
        for row in 0..6 {
            for col in 0..8 {
                if self.pixels[row][col].a > 0 {
                    count += 1;
                }
            }
        }
        count
    }

    /// Compute glyph family coverage: how many of the 8 families are represented.
    ///
    /// Full coverage (8/8) indicates a "GROUNDED" program that exercises
    /// all semantic categories: data, transform, state, recursion,
    /// boundary, control, comparison, quantity.
    #[must_use]
    pub fn glyph_coverage(&self) -> u8 {
        let mut seen = [false; 8];
        for row in 0..6 {
            for col in 0..8 {
                let px = &self.pixels[row][col];
                if px.a > 0 {
                    let fam = px.family() as usize;
                    if fam < 8 {
                        seen[fam] = true;
                    }
                }
            }
        }
        seen.iter().filter(|&&s| s).count() as u8
    }

    /// Set the specification row (row 6) from expected output values.
    ///
    /// Each byte in the spec is stored as a pixel's blue channel.
    pub fn set_spec(&mut self, spec: &[u8]) {
        for (col, &val) in spec.iter().take(8).enumerate() {
            self.pixels[6][col] = Pixel {
                r: 0,
                g: 0,
                b: val,
                a: 128, // half-confidence marks spec pixels
            };
        }
    }

    /// Read the specification row (row 6).
    #[must_use]
    pub fn spec(&self) -> Vec<u8> {
        (0..8).map(|col| self.pixels[6][col].b).collect()
    }

    /// Set the checksum row (row 7) from output hash bytes.
    pub fn set_checksum(&mut self, hash: &[u8]) {
        for (col, &val) in hash.iter().take(8).enumerate() {
            self.pixels[7][col] = Pixel {
                r: 0,
                g: 0,
                b: val,
                a: 64, // quarter-confidence marks checksum pixels
            };
        }
    }

    /// Compute a position-sensitive FNV-1a checksum of the program region.
    ///
    /// Each pixel's (row, col, R, G, B) feeds into FNV-1a, making the hash
    /// sensitive to both content AND position. This eliminates the XOR
    /// collision class where swapping same-contribution pixels was undetectable.
    ///
    /// Output: 8 bytes extracted from the 64-bit FNV-1a hash.
    #[must_use]
    pub fn compute_checksum(&self) -> [u8; 8] {
        // FNV-1a constants (64-bit)
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x00000100000001B3;

        let mut h = FNV_OFFSET;
        for row in 0..6u8 {
            for col in 0..8u8 {
                let px = &self.pixels[row as usize][col as usize];
                // Feed position first (makes hash position-sensitive)
                for byte in [row, col, px.r, px.g, px.b] {
                    h ^= byte as u64;
                    h = h.wrapping_mul(FNV_PRIME);
                }
            }
        }

        // Extract 8 bytes from 64-bit hash
        h.to_le_bytes()
    }

    /// Verify the tile: does the checksum in row 7 match the program?
    #[must_use]
    pub fn verify(&self) -> bool {
        let computed = self.compute_checksum();
        let stored: Vec<u8> = (0..8).map(|col| self.pixels[7][col].b).collect();

        // Only verify if checksum row is populated (alpha > 0)
        let has_checksum = (0..8).any(|col| self.pixels[7][col].a > 0);
        if !has_checksum {
            return true; // no checksum set, trivially valid
        }

        computed.iter().zip(stored.iter()).all(|(a, b)| a == b)
    }

    /// Serialize tile to raw 256-byte RGBA buffer.
    #[must_use]
    pub fn to_rgba(&self) -> [u8; 256] {
        let mut buf = [0u8; 256];
        for row in 0..8 {
            for col in 0..8 {
                let px = &self.pixels[row][col];
                let offset = (row * 8 + col) * 4;
                buf[offset] = px.r;
                buf[offset + 1] = px.g;
                buf[offset + 2] = px.b;
                buf[offset + 3] = px.a;
            }
        }
        buf
    }

    /// Deserialize tile from raw 256-byte RGBA buffer.
    #[must_use]
    pub fn from_rgba(buf: &[u8; 256]) -> Self {
        let mut tile = Self::empty();
        for row in 0..8 {
            for col in 0..8 {
                let offset = (row * 8 + col) * 4;
                tile.pixels[row][col] = Pixel {
                    r: buf[offset],
                    g: buf[offset + 1],
                    b: buf[offset + 2],
                    a: buf[offset + 3],
                };
            }
        }
        tile
    }

    /// Average confidence across all non-empty program pixels.
    #[must_use]
    pub fn average_confidence(&self) -> f64 {
        let mut sum = 0u64;
        let mut count = 0u64;
        for row in 0..6 {
            for col in 0..8 {
                if self.pixels[row][col].a > 0 {
                    sum += u64::from(self.pixels[row][col].a);
                    count += 1;
                }
            }
        }
        if count == 0 {
            0.0
        } else {
            (sum as f64) / (count as f64) / 255.0
        }
    }
}

// ---------------------------------------------------------------------------
// Tile Inspector — rich state visualization
// ---------------------------------------------------------------------------

/// Family names matching glyph indices (0-7).
const FAMILY_NAMES: [&str; 8] = [
    "σ data", "μ xfrm", "ς stat", "ρ recr", "∂ bnd ", "→ ctrl", "κ cmp ", "N qty ",
];

/// Family ANSI color codes for terminal display.
const FAMILY_COLORS: [&str; 8] = [
    "\x1b[37m", // 0 σ: white/gray
    "\x1b[32m", // 1 μ: green
    "\x1b[33m", // 2 ς: yellow/amber
    "\x1b[36m", // 3 ρ: cyan/teal
    "\x1b[31m", // 4 ∂: red
    "\x1b[34m", // 5 →: blue
    "\x1b[35m", // 6 κ: magenta/purple
    "\x1b[93m", // 7 N: bright yellow
];

const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";

/// Rich inspection of a tile's internal state.
///
/// Produces formatted output showing every pixel's instruction, family,
/// color, confidence, and tile-level metadata.
///
/// Tier: T2-C (μ Mapping + λ Location + σ Sequence + κ Comparison)
pub struct TileInspector<'a> {
    tile: &'a Tile,
}

impl<'a> TileInspector<'a> {
    /// Create an inspector for a tile.
    #[must_use]
    pub fn new(tile: &'a Tile) -> Self {
        Self { tile }
    }

    /// Generate full inspection report as a string.
    #[must_use]
    pub fn report(&self) -> String {
        let mut out = String::with_capacity(4096);

        // Header
        out.push_str(&format!(
            "{}╔══════════════════════════════════════════════════════════════╗{}\n",
            BOLD, RESET
        ));
        out.push_str(&format!(
            "{}║  TILE INSPECTOR — 8×8 Pixel Program Tile                   ║{}\n",
            BOLD, RESET
        ));
        out.push_str(&format!(
            "{}╚══════════════════════════════════════════════════════════════╝{}\n\n",
            BOLD, RESET
        ));

        // Pixel grid with color
        out.push_str(&self.format_grid());
        out.push('\n');

        // Instruction listing
        out.push_str(&self.format_instruction_listing());
        out.push('\n');

        // Tile metadata
        out.push_str(&self.format_metadata());
        out.push('\n');

        // Family distribution
        out.push_str(&self.format_family_distribution());
        out.push('\n');

        // Checksum state
        out.push_str(&self.format_checksum_state());

        out
    }

    /// Format the 8×8 pixel grid with colored instruction mnemonics.
    #[must_use]
    pub fn format_grid(&self) -> String {
        let mut out = String::with_capacity(2048);

        out.push_str(
            "  ┌────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┐\n",
        );

        for row in 0..8 {
            let row_label = match row {
                0..=5 => format!("{} ", row),
                6 => "S ".to_string(),
                7 => "H ".to_string(),
                _ => "  ".to_string(),
            };
            out.push_str(&row_label);
            out.push('│');

            for col in 0..8 {
                let px = &self.tile.pixels[row][col];
                if px.a == 0 {
                    out.push_str(&format!("{DIM}  · · · {RESET}│"));
                } else {
                    let fam = px.family() as usize;
                    let color = if fam < 8 { FAMILY_COLORS[fam] } else { RESET };

                    if row < 6 {
                        // Program pixel: show mnemonic
                        let mnemonic = pixel_mnemonic(px);
                        out.push_str(&format!("{color}{:^8}{RESET}│", mnemonic));
                    } else if row == 6 {
                        // Spec pixel
                        out.push_str(&format!("{DIM} S={:<4}{RESET}│", px.b));
                    } else {
                        // Checksum pixel
                        out.push_str(&format!("{DIM} H={:<4}{RESET}│", px.b));
                    }
                }
            }

            let region = match row {
                0 => " ← PROGRAM",
                5 => " ← PROGRAM (end)",
                6 => " ← SPEC",
                7 => " ← CHECKSUM",
                _ => "",
            };
            out.push_str(&format!(" {region}\n"));

            if row == 5 || row == 6 {
                out.push_str(
                    "  ├────────┼────────┼────────┼────────┼────────┼────────┼────────┼────────┤\n",
                );
            }
        }

        out.push_str(
            "  └────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘\n",
        );
        out.push_str("     C0       C1       C2       C3       C4       C5       C6       C7\n");

        out
    }

    /// Format instruction listing with details.
    #[must_use]
    pub fn format_instruction_listing(&self) -> String {
        let mut out = String::with_capacity(1024);
        let instrs = self.tile.to_instructions();

        out.push_str(&format!("{BOLD}Instructions ({}):{RESET}\n", instrs.len()));
        out.push_str(
            "  ┌─────┬────────────┬──────────┬─────────────┬──────┬────────────────────┐\n",
        );
        out.push_str(
            "  │ Idx │ Instruction│ Family   │ Variant     │ Lit  │ Color (R,G,B,A)    │\n",
        );
        out.push_str(
            "  ├─────┼────────────┼──────────┼─────────────┼──────┼────────────────────┤\n",
        );

        for (i, instr) in instrs.iter().enumerate() {
            let row = i / 8;
            let col = i % 8;
            let px = &self.tile.pixels[row][col];
            let fam = px.family() as usize;
            let var = px.variant();
            let color = if fam < 8 { FAMILY_COLORS[fam] } else { RESET };
            let fam_name = if fam < 8 { FAMILY_NAMES[fam] } else { "?" };

            let lit_str = match instr {
                crate::isa::Instruction::Lit(n) => format!("{n}"),
                _ => "—".to_string(),
            };

            out.push_str(&format!(
                "  │ {i:>3} │ {color}{:<10}{RESET} │ {color}{:<8}{RESET} │ P1={var:<9} │ {:<4} │ ({:>3},{:>3},{:>3},{:>3}) │\n",
                format!("{instr:?}").chars().take(10).collect::<String>(),
                fam_name,
                lit_str,
                px.r, px.g, px.b, px.a
            ));
        }

        out.push_str(
            "  └─────┴────────────┴──────────┴─────────────┴──────┴────────────────────┘\n",
        );
        out
    }

    /// Format tile metadata summary.
    #[must_use]
    pub fn format_metadata(&self) -> String {
        let instrs = self.tile.to_instructions();
        let mut out = String::with_capacity(512);

        out.push_str(&format!("{BOLD}Tile Metadata:{RESET}\n"));
        out.push_str(&format!(
            "  Instructions:     {}/48\n",
            self.tile.instruction_count()
        ));
        out.push_str(&format!(
            "  Glyph coverage:   {}/8 families\n",
            self.tile.glyph_coverage()
        ));
        out.push_str(&format!(
            "  Avg confidence:   {:.1}%\n",
            self.tile.average_confidence() * 100.0
        ));
        out.push_str("  RGBA size:        256 bytes (fixed)\n");
        out.push_str(&format!(
            "  Bits/instruction: {:.1}\n",
            if instrs.is_empty() {
                0.0
            } else {
                2048.0 / instrs.len() as f64
            }
        ));

        // Lit analysis
        let lit_count = instrs
            .iter()
            .filter(|i| matches!(i, crate::isa::Instruction::Lit(_)))
            .count();
        let lit_pct = if instrs.is_empty() {
            0.0
        } else {
            lit_count as f64 / instrs.len() as f64 * 100.0
        };
        out.push_str(&format!(
            "  Lit density:      {lit_count}/{} ({lit_pct:.0}%)\n",
            instrs.len()
        ));

        // Spec row
        let spec = self.tile.spec();
        let has_spec = spec.iter().any(|&s| s != 0);
        if has_spec {
            out.push_str(&format!("  Spec values:      {:?}\n", spec));
        }

        out
    }

    /// Format family distribution as a histogram.
    #[must_use]
    pub fn format_family_distribution(&self) -> String {
        let mut counts = [0u32; 8];
        let mut total = 0u32;

        for row in 0..6 {
            for col in 0..8 {
                let px = &self.tile.pixels[row][col];
                if px.a > 0 {
                    let fam = px.family() as usize;
                    if fam < 8 {
                        counts[fam] += 1;
                        total += 1;
                    }
                }
            }
        }

        let mut out = String::with_capacity(512);
        out.push_str(&format!("{BOLD}Family Distribution:{RESET}\n"));

        let max_count = counts.iter().copied().max().unwrap_or(1).max(1);

        for (i, &count) in counts.iter().enumerate() {
            let bar_len = (count * 30 / max_count) as usize;
            let bar: String = "█".repeat(bar_len);
            let pct = if total == 0 {
                0.0
            } else {
                count as f64 / total as f64 * 100.0
            };
            let color = FAMILY_COLORS[i];

            if count > 0 {
                out.push_str(&format!(
                    "  {color}{:<8}{RESET} │{color}{bar}{RESET} {count} ({pct:.0}%)\n",
                    FAMILY_NAMES[i]
                ));
            } else {
                out.push_str(&format!("  {DIM}{:<8}{RESET} │ — \n", FAMILY_NAMES[i]));
            }
        }

        out
    }

    /// Format checksum state.
    #[must_use]
    pub fn format_checksum_state(&self) -> String {
        let mut out = String::with_capacity(256);
        let has_checksum = (0..8).any(|col| self.tile.pixels[7][col].a > 0);

        out.push_str(&format!("{BOLD}Checksum:{RESET}\n"));
        if has_checksum {
            let stored: Vec<u8> = (0..8).map(|col| self.tile.pixels[7][col].b).collect();
            let computed = self.tile.compute_checksum();
            let valid = self.tile.verify();

            out.push_str(&format!("  Stored:   {:?}\n", stored));
            out.push_str(&format!("  Computed: {:?}\n", computed.to_vec()));
            if valid {
                out.push_str(&format!("  Status:   {}\x1b[32m✓ VALID{RESET}\n", BOLD));
            } else {
                out.push_str(&format!("  Status:   {}\x1b[31m✗ CORRUPTED{RESET}\n", BOLD));
            }
        } else {
            out.push_str(&format!("  {DIM}No checksum set{RESET}\n"));
        }

        out
    }
}

/// Get a short mnemonic for a pixel's instruction.
fn pixel_mnemonic(px: &Pixel) -> String {
    if px.a == 0 {
        return "·".to_string();
    }

    let p0 = px.family();
    let p1 = px.variant();

    // Lit encoding
    if p0 == 7 && p1 == 7 {
        return format!("L={}", px.b);
    }

    // Decode to instruction and use Debug format
    if let Some(instr) = px.to_instruction() {
        let s = format!("{instr:?}");
        // Truncate to 6 chars
        if s.len() <= 6 { s } else { s[..6].to_string() }
    } else {
        format!("{p0}:{p1}")
    }
}

// ---------------------------------------------------------------------------
// Helper: codon index from instruction (without producing a Codon)
// ---------------------------------------------------------------------------

/// Get the codon index (0-63) for a real instruction.
///
/// Delegates to `isa::encode` to avoid maintaining a duplicate mapping table.
fn codon_index(instr: &Instruction) -> Option<u8> {
    crate::isa::encode(instr).map(|c| c.index())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_from_add() {
        let px = Pixel::from_instruction(&Instruction::Add);
        // v3 ISA: Add = codon 8, P0 = 8/8 = 1 (μ), P1 = 8%8 = 0 (σ)
        assert_eq!(px.family(), 1);
        assert_eq!(px.variant(), 0);
        assert_eq!(px.b, 0);
        assert_eq!(px.a, 255);
    }

    #[test]
    fn pixel_from_lit() {
        let px = Pixel::from_instruction(&Instruction::Lit(42));
        assert_eq!(px.family(), 7);
        assert_eq!(px.variant(), 7);
        assert_eq!(px.b, 42);
        assert_eq!(px.a, 255);
    }

    #[test]
    fn pixel_roundtrip() {
        let instrs = [
            Instruction::Nop,
            Instruction::Add,
            Instruction::Halt,
            Instruction::Call,
            Instruction::Push0,
            Instruction::Eq,
        ];
        for instr in &instrs {
            let px = Pixel::from_instruction(instr);
            let decoded = px.to_instruction();
            assert_eq!(decoded, Some(*instr), "roundtrip failed for {instr:?}");
        }
    }

    #[test]
    fn pixel_lit_roundtrip() {
        let px = Pixel::from_instruction(&Instruction::Lit(100));
        let decoded = px.to_instruction();
        assert_eq!(decoded, Some(Instruction::Lit(100)));
    }

    #[test]
    fn pixel_lit_large_truncated() {
        // Values > 255 are truncated to lower 8 bits
        let px = Pixel::from_instruction(&Instruction::Lit(300));
        assert_eq!(px.b, 44); // 300 & 0xFF = 44
    }

    #[test]
    fn pixel_empty_decodes_none() {
        assert_eq!(Pixel::EMPTY.to_instruction(), None);
    }

    #[test]
    fn pixel_confidence() {
        let px = Pixel::from_instruction(&Instruction::Add);
        assert!((px.confidence() - 1.0).abs() < f64::EPSILON);

        let dim = px.with_confidence(0.5);
        assert!((dim.confidence() - 0.5).abs() < 0.01);
        assert_eq!(dim.a, 127); // 0.5 * 255 ≈ 127
    }

    #[test]
    fn pixel_display_color() {
        // v3 ISA: Add → P0=1 (μ transform) → green
        let px = Pixel::from_instruction(&Instruction::Add); // codon 8, P0=1
        let (r, g, _b) = px.display_color();
        assert!(g > r, "mapping family should be green-dominant");
    }

    #[test]
    fn tile_from_instructions() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(5),
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ];
        let tile = Tile::from_instructions(&instrs);
        assert_eq!(tile.instruction_count(), 5);
    }

    #[test]
    fn tile_roundtrip() {
        let instrs = vec![
            Instruction::Nop,
            Instruction::Push0,
            Instruction::Push1,
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ];
        let tile = Tile::from_instructions(&instrs);
        let decoded = tile.to_instructions();
        assert_eq!(decoded, instrs);
    }

    #[test]
    fn tile_max_48_instructions() {
        let instrs: Vec<Instruction> = (0..60).map(|_| Instruction::Nop).collect();
        let tile = Tile::from_instructions(&instrs);
        assert_eq!(tile.instruction_count(), 48); // capped at rows 0-5
    }

    #[test]
    fn tile_glyph_coverage() {
        // Single family
        let instrs = vec![Instruction::Add, Instruction::Sub, Instruction::Mul];
        let tile = Tile::from_instructions(&instrs);
        assert_eq!(tile.glyph_coverage(), 1); // all P0=5

        // Multiple families
        let instrs = vec![
            Instruction::Nop,   // P0=5 (codon 40)
            Instruction::Halt,  // P0=2 (codon 16)
            Instruction::Push0, // P0=0 (codon 4)
            Instruction::Eq,    // P0=4 (codon 36)
            Instruction::Min,   // P0=7 (codon 56)
        ];
        let tile = Tile::from_instructions(&instrs);
        assert!(tile.glyph_coverage() >= 4);
    }

    #[test]
    fn tile_spec_and_checksum() {
        let instrs = vec![Instruction::Push1, Instruction::Output, Instruction::Halt];
        let mut tile = Tile::from_instructions(&instrs);

        // Set spec: expect output value 1
        tile.set_spec(&[1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(tile.spec()[0], 1);

        // Set and verify checksum
        let hash = tile.compute_checksum();
        tile.set_checksum(&hash);
        assert!(tile.verify());
    }

    #[test]
    fn tile_checksum_detects_corruption() {
        let instrs = vec![Instruction::Add, Instruction::Halt];
        let mut tile = Tile::from_instructions(&instrs);

        let hash = tile.compute_checksum();
        tile.set_checksum(&hash);
        assert!(tile.verify());

        // Corrupt one instruction pixel
        tile.pixels[0][0] = Pixel::from_instruction(&Instruction::Sub);
        assert!(!tile.verify());
    }

    #[test]
    fn tile_rgba_roundtrip() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(42),
            Instruction::Output,
            Instruction::Halt,
        ];
        let tile = Tile::from_instructions(&instrs);
        let buf = tile.to_rgba();
        let restored = Tile::from_rgba(&buf);
        assert_eq!(tile, restored);
    }

    #[test]
    fn tile_empty_verifies() {
        let tile = Tile::empty();
        assert!(tile.verify()); // no checksum set = trivially valid
    }

    #[test]
    fn tile_average_confidence() {
        let instrs = vec![Instruction::Add, Instruction::Sub];
        let tile = Tile::from_instructions(&instrs);
        assert!((tile.average_confidence() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tile_average_confidence_mixed() {
        let mut tile = Tile::empty();
        tile.pixels[0][0] = Pixel::from_instruction(&Instruction::Add);
        tile.pixels[0][1] = Pixel::from_instruction(&Instruction::Sub).with_confidence(0.0);
        // with_confidence(0.0) sets a=0, which means the pixel becomes empty
        // So only 1 pixel counts
        assert!((tile.average_confidence() - 1.0).abs() < f64::EPSILON);
    }

    // -----------------------------------------------------------------------
    // TileInspector tests
    // -----------------------------------------------------------------------

    #[test]
    fn inspector_report_contains_all_sections() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(5),
            Instruction::Lit(3),
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let report = inspector.report();

        // Header
        assert!(
            report.contains("TILE INSPECTOR"),
            "missing TILE INSPECTOR header"
        );
        // Grid (box-drawing chars)
        assert!(report.contains("┌────"), "missing grid border");
        // Instruction listing section
        assert!(
            report.contains("Instructions ("),
            "missing Instructions section"
        );
        // Tile Metadata section
        assert!(
            report.contains("Tile Metadata"),
            "missing Tile Metadata section"
        );
        // Family Distribution section
        assert!(
            report.contains("Family Distribution"),
            "missing Family Distribution section"
        );
        // Checksum section
        assert!(report.contains("Checksum"), "missing Checksum section");
    }

    #[test]
    fn inspector_metadata_shows_instruction_count() {
        let instrs = vec![
            Instruction::Push0,
            Instruction::Push1,
            Instruction::Add,
            Instruction::Output,
        ];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let meta = inspector.format_metadata();

        assert!(meta.contains("4/48"), "should show 4/48 instructions");
        assert!(meta.contains("Lit density"), "should show Lit density");
        assert!(meta.contains("0/4"), "should show 0 Lits out of 4");
    }

    #[test]
    fn inspector_metadata_shows_lit_density() {
        let instrs = vec![
            Instruction::Lit(10),
            Instruction::Lit(20),
            Instruction::Add,
            Instruction::Output,
        ];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let meta = inspector.format_metadata();

        assert!(meta.contains("2/4"), "should show 2 Lits out of 4");
        assert!(meta.contains("50%"), "should show 50% Lit density");
    }

    #[test]
    fn inspector_family_distribution_counts() {
        // All same family → one bar at 100%
        let instrs = vec![Instruction::Add, Instruction::Sub, Instruction::Mul];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let dist = inspector.format_family_distribution();

        assert!(dist.contains("100%"), "single family should be 100%");
    }

    #[test]
    fn inspector_family_distribution_multiple() {
        let instrs = vec![
            Instruction::Push0, // σ data
            Instruction::Halt,  // ς state
            Instruction::Add,   // μ transform
            Instruction::Eq,    // κ compare
        ];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let dist = inspector.format_family_distribution();

        // Each family should show 25%
        assert!(dist.contains("25%"), "each family should be 25%");
    }

    #[test]
    fn inspector_checksum_no_checksum() {
        let instrs = vec![Instruction::Nop, Instruction::Halt];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let cs = inspector.format_checksum_state();

        assert!(cs.contains("No checksum set"), "should say no checksum set");
    }

    #[test]
    fn inspector_checksum_valid() {
        let instrs = vec![Instruction::Push1, Instruction::Output, Instruction::Halt];
        let mut tile = Tile::from_instructions(&instrs);
        let hash = tile.compute_checksum();
        tile.set_checksum(&hash);

        let inspector = TileInspector::new(&tile);
        let cs = inspector.format_checksum_state();

        assert!(cs.contains("VALID"), "valid checksum should say VALID");
    }

    #[test]
    fn inspector_checksum_corrupted() {
        let instrs = vec![Instruction::Push1, Instruction::Output, Instruction::Halt];
        let mut tile = Tile::from_instructions(&instrs);
        let hash = tile.compute_checksum();
        tile.set_checksum(&hash);

        // Corrupt a pixel
        tile.pixels[0][0] = Pixel::from_instruction(&Instruction::Nop);

        let inspector = TileInspector::new(&tile);
        let cs = inspector.format_checksum_state();

        assert!(
            cs.contains("CORRUPTED"),
            "corrupted checksum should say CORRUPTED"
        );
    }

    #[test]
    fn inspector_grid_has_8_rows() {
        let instrs = vec![Instruction::Add, Instruction::Sub];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let grid = inspector.format_grid();

        // The grid header + 8 data rows
        let lines: Vec<&str> = grid.lines().collect();
        // At least 9 lines: header + 8 grid rows
        assert!(lines.len() >= 9, "grid should have at least 9 lines");
    }

    #[test]
    fn inspector_instruction_listing_entries() {
        let instrs = vec![
            Instruction::Entry,
            Instruction::Lit(42),
            Instruction::Output,
            Instruction::Halt,
        ];
        let tile = Tile::from_instructions(&instrs);
        let inspector = TileInspector::new(&tile);
        let listing = inspector.format_instruction_listing();

        // Should show each instruction
        assert!(listing.contains("Entry"), "should list Entry");
        assert!(listing.contains("Lit(42)"), "should list Lit(42)");
        assert!(listing.contains("Output"), "should list Output");
        assert!(listing.contains("Halt"), "should list Halt");
    }

    #[test]
    fn inspector_empty_tile() {
        let tile = Tile::empty();
        let inspector = TileInspector::new(&tile);
        let report = inspector.report();

        assert!(report.contains("0/48"), "empty tile should show 0/48");
        assert!(
            report.contains("No checksum set"),
            "empty tile has no checksum"
        );
    }

    #[test]
    fn pixel_mnemonic_empty() {
        assert_eq!(pixel_mnemonic(&Pixel::EMPTY), "·");
    }

    #[test]
    fn pixel_mnemonic_lit() {
        let px = Pixel::from_instruction(&Instruction::Lit(99));
        assert_eq!(pixel_mnemonic(&px), "L=99");
    }

    #[test]
    fn pixel_mnemonic_instruction() {
        let px = Pixel::from_instruction(&Instruction::Add);
        let m = pixel_mnemonic(&px);
        assert_eq!(m, "Add");
    }
}
