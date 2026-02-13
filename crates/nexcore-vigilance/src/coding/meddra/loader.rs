//! `MedDRA` ASCII file parser.
//!
//! Parses standard `MedDRA` ASCII distribution files with `$` delimiter:
//! - `llt.asc` - Lowest Level Terms
//! - `pt.asc` - Preferred Terms
//! - `hlt.asc` - High Level Terms
//! - `hlgt.asc` - High Level Group Terms
//! - `soc.asc` - System Organ Classes
//! - `hlt_pt.asc` - HLT to PT relationships
//! - `hlgt_hlt.asc` - HLGT to HLT relationships
//! - `soc_hlgt.asc` - SOC to HLGT relationships

use super::super::error::CodingError;
use super::types::{Hlgt, Hlt, Llt, MeddraVersion, Pt, Soc};

/// Parse LLT records from ASCII file content.
///
/// Format: `llt_code$llt_name$pt_code$llt_whoart_code$llt_harts_code$llt_costart_sym$llt_icd9_code$llt_icd9cm_code$llt_icd10_code$llt_currency$llt_jart_code$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if:
/// - A line has fewer than 10 fields.
/// - Numeric fields (code, PT code) cannot be parsed.
///
/// # Complexity
///
/// - TIME: O(n) where n is file size
/// - SPACE: O(k) where k is number of LLTs
pub fn parse_llt(content: &str) -> Result<Vec<Llt>, CodingError> {
    let mut llts = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 10 {
            return Err(CodingError::parse_error(format!(
                "LLT line {}: expected 10+ fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("LLT line {}: invalid code: {e}", line_num + 1))
        })?;

        let pt_code = parts[2].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("LLT line {}: invalid PT code: {e}", line_num + 1))
        })?;

        let is_current = parts[9].eq_ignore_ascii_case("y");

        llts.push(Llt {
            code,
            name: parts[1].to_string(),
            pt_code,
            is_current,
        });
    }

    Ok(llts)
}

/// Parse PT records from ASCII file content.
///
/// Format: `pt_code$pt_name$null_field$pt_soc_code$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if:
/// - A line has fewer than 4 fields.
/// - Numeric fields (code, SOC code) cannot be parsed.
///
/// # Complexity
///
/// - TIME: O(n) where n is file size
/// - SPACE: O(k) where k is number of PTs
pub fn parse_pt(content: &str) -> Result<Vec<Pt>, CodingError> {
    let mut pts = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 4 {
            return Err(CodingError::parse_error(format!(
                "PT line {}: expected 4+ fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("PT line {}: invalid code: {e}", line_num + 1))
        })?;

        let primary_soc_code = parts[3].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("PT line {}: invalid SOC code: {e}", line_num + 1))
        })?;

        pts.push(Pt {
            code,
            name: parts[1].to_string(),
            primary_soc_code,
        });
    }

    Ok(pts)
}

/// Parse HLT records from ASCII file content.
///
/// Format: `hlt_code$hlt_name$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if line format is invalid.
pub fn parse_hlt(content: &str) -> Result<Vec<Hlt>, CodingError> {
    let mut hlts = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 2 {
            return Err(CodingError::parse_error(format!(
                "HLT line {}: expected 2+ fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("HLT line {}: invalid code: {e}", line_num + 1))
        })?;

        hlts.push(Hlt {
            code,
            name: parts[1].to_string(),
        });
    }

    Ok(hlts)
}

/// Parse HLGT records from ASCII file content.
///
/// Format: `hlgt_code$hlgt_name$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if line format is invalid.
pub fn parse_hlgt(content: &str) -> Result<Vec<Hlgt>, CodingError> {
    let mut hlgts = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 2 {
            return Err(CodingError::parse_error(format!(
                "HLGT line {}: expected 2+ fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("HLGT line {}: invalid code: {e}", line_num + 1))
        })?;

        hlgts.push(Hlgt {
            code,
            name: parts[1].to_string(),
        });
    }

    Ok(hlgts)
}

/// Parse SOC records from ASCII file content.
///
/// Format: `soc_code$soc_name$soc_abbrev$soc_whoart_code$soc_harts_code$soc_costart_sym$soc_icd9_code$soc_icd9cm_code$soc_icd10_code$soc_jart_code$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if line format is invalid.
pub fn parse_soc(content: &str) -> Result<Vec<Soc>, CodingError> {
    let mut socs = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 3 {
            return Err(CodingError::parse_error(format!(
                "SOC line {}: expected 3+ fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!("SOC line {}: invalid code: {e}", line_num + 1))
        })?;

        // Use saturating cast or try_from if exact order matters, but for intl_order u16 is sufficient
        // line_num won't exceed u16 max (65535) for standard MedDRA files (27 SOCs)
        #[allow(clippy::cast_possible_truncation)]
        let intl_order = (line_num + 1) as u16;

        socs.push(Soc {
            code,
            name: parts[1].to_string(),
            abbrev: parts[2].to_string(),
            intl_order,
        });
    }

    Ok(socs)
}

/// Parse HLT-PT relationship file.
///
/// Format: `hlt_code$pt_code$`
///
/// Returns: Vec<(`hlt_code`, `pt_code`)>
///
/// # Errors
///
/// Returns `CodingError::ParseError` if line format is invalid.
#[allow(clippy::similar_names)]
pub fn parse_hlt_pt(content: &str) -> Result<Vec<(u32, u32)>, CodingError> {
    let mut relationships = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 2 {
            return Err(CodingError::parse_error(format!(
                "HLT_PT line {}: expected 2 fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let hlt_code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!(
                "HLT_PT line {}: invalid HLT code: {e}",
                line_num + 1
            ))
        })?;

        let pt_code = parts[1].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!(
                "HLT_PT line {}: invalid PT code: {e}",
                line_num + 1
            ))
        })?;

        relationships.push((hlt_code, pt_code));
    }

    Ok(relationships)
}

/// Parse HLGT-HLT relationship file.
///
/// Format: `hlgt_code$hlt_code$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if line format is invalid.
#[allow(clippy::similar_names)]
pub fn parse_hlgt_hlt(content: &str) -> Result<Vec<(u32, u32)>, CodingError> {
    let mut relationships = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 2 {
            return Err(CodingError::parse_error(format!(
                "HLGT_HLT line {}: expected 2 fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let hlgt_code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!(
                "HLGT_HLT line {}: invalid HLGT code: {e}",
                line_num + 1
            ))
        })?;

        let hlt_code = parts[1].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!(
                "HLGT_HLT line {}: invalid HLT code: {e}",
                line_num + 1
            ))
        })?;

        relationships.push((hlgt_code, hlt_code));
    }

    Ok(relationships)
}

/// Parse SOC-HLGT relationship file.
///
/// Format: `soc_code$hlgt_code$`
///
/// # Errors
///
/// Returns `CodingError::ParseError` if line format is invalid.
#[allow(clippy::similar_names)]
pub fn parse_soc_hlgt(content: &str) -> Result<Vec<(u32, u32)>, CodingError> {
    let mut relationships = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('$').collect();
        if parts.len() < 2 {
            return Err(CodingError::parse_error(format!(
                "SOC_HLGT line {}: expected 2 fields, got {}",
                line_num + 1,
                parts.len()
            )));
        }

        let soc_code = parts[0].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!(
                "SOC_HLGT line {}: invalid SOC code: {e}",
                line_num + 1
            ))
        })?;

        let hlgt_code = parts[1].parse::<u32>().map_err(|e| {
            CodingError::parse_error(format!(
                "SOC_HLGT line {}: invalid HLGT code: {e}",
                line_num + 1
            ))
        })?;

        relationships.push((soc_code, hlgt_code));
    }

    Ok(relationships)
}

/// Parse `MedDRA` version from `mdhier.asc` or version file.
///
/// Extracts version from content like "`MedDRA` Version 26.1"
#[must_use]
pub fn parse_version(content: &str) -> Option<MeddraVersion> {
    // Try to extract version pattern like "26.1" or "26_1"
    for line in content.lines() {
        let line = line.to_lowercase();
        // Check for 'v' char specifically or "version" string
        if line.contains("meddra") && (line.contains("version") || line.contains('v')) {
            // Look for pattern X.Y or X_Y
            for word in line.split_whitespace() {
                if let Some((major, minor)) = parse_version_string(word) {
                    return Some(MeddraVersion::new(major, minor, "English"));
                }
            }
        }
    }
    None
}

fn parse_version_string(s: &str) -> Option<(u8, u8)> {
    // Try X.Y format
    if let Some((maj, min)) = s.split_once('.') {
        if let (Ok(major), Ok(minor)) = (maj.parse::<u8>(), min.parse::<u8>()) {
            return Some((major, minor));
        }
    }
    // Try X_Y format
    if let Some((maj, min)) = s.split_once('_') {
        if let (Ok(major), Ok(minor)) = (maj.parse::<u8>(), min.parse::<u8>()) {
            return Some((major, minor));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llt() {
        // LLT format: llt_code$llt_name$pt_code$whoart$harts$costart$icd9$icd9cm$icd10$currency$jart$
        // Fields:     0        1        2       3      4     5       6    7      8     9        10
        let content = "10019211$Headache$10019231$$$$$$$Y$$\n10019212$Head pain$10019231$$$$$$$Y$$";
        let llts = parse_llt(content).expect("parse failed");
        assert_eq!(llts.len(), 2);
        assert_eq!(llts[0].code, 10019211);
        assert_eq!(llts[0].name, "Headache");
        assert_eq!(llts[0].pt_code, 10019231);
        assert!(llts[0].is_current);
    }

    #[test]
    fn test_parse_pt() {
        let content = "10019231$Headache$$10029205$";
        let pts = parse_pt(content).expect("parse failed");
        assert_eq!(pts.len(), 1);
        assert_eq!(pts[0].code, 10019231);
        assert_eq!(pts[0].name, "Headache");
        assert_eq!(pts[0].primary_soc_code, 10029205);
    }

    #[test]
    fn test_parse_soc() {
        let content = "10029205$Nervous system disorders$Nerv$";
        let socs = parse_soc(content).expect("parse failed");
        assert_eq!(socs.len(), 1);
        assert_eq!(socs[0].code, 10029205);
        assert_eq!(socs[0].name, "Nervous system disorders");
        assert_eq!(socs[0].abbrev, "Nerv");
    }

    #[test]
    fn test_parse_hlt_pt() {
        let content = "10019233$10019231$\n10019233$10028813$";
        let rels = parse_hlt_pt(content).expect("parse failed");
        assert_eq!(rels.len(), 2);
        assert_eq!(rels[0], (10019233, 10019231));
    }

    #[test]
    fn test_parse_version() {
        let content = "MedDRA Version 26.1\nEnglish";
        let version = parse_version(content);
        assert!(version.is_some());
        let v = version.expect("version should exist");
        assert_eq!(v.major, 26);
        assert_eq!(v.minor, 1);
    }

    #[test]
    fn test_empty_lines_skipped() {
        let content = "\n10019231$Headache$$10029205$\n\n";
        let pts = parse_pt(content).expect("parse failed");
        assert_eq!(pts.len(), 1);
    }
}
