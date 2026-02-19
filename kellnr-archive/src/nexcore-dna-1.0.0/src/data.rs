//! DNA-native structured data types with TLV encoding and biological operations.
//!
//! Provides typed values, arrays, records, maps, and frames encoded as nucleotide
//! sequences. Uses TLV (Type-Length-Value) encoding with codons as atomic units.
//!
//! Biology mapping:
//! - **Restriction enzyme** -> field extraction (cut at boundaries)
//! - **DNA ligation** -> concatenate/merge structures
//! - **Gene splicing** -> insert/remove fields
//! - **Transcription** -> project to human-readable form

use crate::error::{DnaError, Result};
use crate::storage;
use crate::types::{Codon, Nucleotide, Strand};
use core::fmt;

// ---------------------------------------------------------------------------
// DnaType -- Type discriminant (T1, kappa Comparison)
// ---------------------------------------------------------------------------

/// Type tag for DNA-encoded values.
///
/// Tier: T1 (kappa Comparison)
///
/// Each variant maps to a codon index for TLV encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DnaType {
    /// Null / absence of value. Codon index 0.
    Null,
    /// Boolean. Codon index 1.
    Bool,
    /// 64-bit signed integer. Codon index 2.
    Int,
    /// 64-bit float. Codon index 3.
    Float,
    /// UTF-8 text. Codon index 4.
    Text,
    /// Ordered collection. Codon index 8.
    Array,
    /// Named fields. Codon index 9.
    Record,
    /// Key-value pairs. Codon index 10.
    Map,
    /// Tabular data. Codon index 11.
    Frame,
}

impl DnaType {
    /// Codon index for this type.
    #[must_use]
    pub fn index(&self) -> u8 {
        match self {
            Self::Null => 0,
            Self::Bool => 1,
            Self::Int => 2,
            Self::Float => 3,
            Self::Text => 4,
            Self::Array => 8,
            Self::Record => 9,
            Self::Map => 10,
            Self::Frame => 11,
        }
    }

    /// Encode type tag as a single codon (3 nucleotides).
    pub fn to_codon(&self) -> Result<Codon> {
        Codon::from_index(self.index())
    }

    /// Decode type tag from a codon.
    pub fn from_codon(codon: &Codon) -> Result<Self> {
        Self::from_index(codon.index())
    }

    /// Construct from codon index.
    pub fn from_index(idx: u8) -> Result<Self> {
        match idx {
            0 => Ok(Self::Null),
            1 => Ok(Self::Bool),
            2 => Ok(Self::Int),
            3 => Ok(Self::Float),
            4 => Ok(Self::Text),
            8 => Ok(Self::Array),
            9 => Ok(Self::Record),
            10 => Ok(Self::Map),
            11 => Ok(Self::Frame),
            _ => Err(DnaError::InvalidTlv(format!("unknown type index: {idx}"))),
        }
    }

    /// Human-readable name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Bool => "Bool",
            Self::Int => "Int",
            Self::Float => "Float",
            Self::Text => "Text",
            Self::Array => "Array",
            Self::Record => "Record",
            Self::Map => "Map",
            Self::Frame => "Frame",
        }
    }

    /// True for scalar (non-collection) types.
    #[must_use]
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Self::Null | Self::Bool | Self::Int | Self::Float | Self::Text
        )
    }

    /// True for collection types.
    #[must_use]
    pub fn is_collection(&self) -> bool {
        !self.is_scalar()
    }
}

impl fmt::Display for DnaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ---------------------------------------------------------------------------
// Internal codon-based integer helpers
// ---------------------------------------------------------------------------

/// Encode u32 as 4 codons (12 nucleotides).
fn encode_u32_codons(n: u32) -> Result<Vec<Nucleotide>> {
    let bytes = n.to_be_bytes();
    let mut nucs = Vec::with_capacity(12);
    // 4 bytes -> 4 codons, each byte maps to 1 codon via index
    // Actually: pack 32 bits into 12 nucleotides (2 bits each = 24 bits... not enough)
    // Better approach: use 6 codons for full 32-bit range, but plan says 4 codons.
    // 4 codons = 12 nucleotides = 24 bits of raw data. For length fields up to 16M, fine.
    // But u32 is 32 bits. Use the storage encode for the 4 bytes directly.
    for &b in &bytes {
        // Each byte -> 4 nucleotides (2 bits each)
        for shift in (0..4).rev() {
            let bits = (b >> (shift * 2)) & 0b11;
            nucs.push(Nucleotide::from_bits(bits)?);
        }
    }
    Ok(nucs)
}

/// Decode u32 from 16 nucleotides (4 bytes * 4 nucs/byte).
fn decode_u32_codons(bases: &[Nucleotide]) -> Result<u32> {
    if bases.len() < 16 {
        return Err(DnaError::InvalidTlv(format!(
            "need 16 nucs for u32, got {}",
            bases.len()
        )));
    }
    let mut bytes = [0u8; 4];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let offset = i * 4;
        *byte = decode_nuc_byte(&bases[offset..offset + 4]);
    }
    Ok(u32::from_be_bytes(bytes))
}

/// Encode u16 as 8 nucleotides (2 bytes * 4 nucs/byte).
fn encode_u16_codons(n: u16) -> Result<Vec<Nucleotide>> {
    let bytes = n.to_be_bytes();
    let mut nucs = Vec::with_capacity(8);
    for &b in &bytes {
        for shift in (0..4).rev() {
            let bits = (b >> (shift * 2)) & 0b11;
            nucs.push(Nucleotide::from_bits(bits)?);
        }
    }
    Ok(nucs)
}

/// Decode u16 from 8 nucleotides.
fn decode_u16_codons(bases: &[Nucleotide]) -> Result<u16> {
    if bases.len() < 8 {
        return Err(DnaError::InvalidTlv(format!(
            "need 8 nucs for u16, got {}",
            bases.len()
        )));
    }
    let mut bytes = [0u8; 2];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let offset = i * 4;
        *byte = decode_nuc_byte(&bases[offset..offset + 4]);
    }
    Ok(u16::from_be_bytes(bytes))
}

/// Decode a single byte from 4 nucleotides.
fn decode_nuc_byte(bases: &[Nucleotide]) -> u8 {
    let mut byte: u8 = 0;
    for (i, &nuc) in bases.iter().enumerate().take(4) {
        let shift = (3 - i) * 2;
        byte |= nuc.bits() << shift;
    }
    byte
}

// ---------------------------------------------------------------------------
// DnaValue -- Tagged value (T2-P, Existence)
// ---------------------------------------------------------------------------

/// A typed DNA-encoded value.
///
/// Tier: T2-P (Existence + State)
///
/// Stores the type discriminant and the encoded value strand (without TLV header).
#[derive(Debug, Clone, PartialEq)]
pub struct DnaValue {
    /// The type of this value.
    pub dtype: DnaType,
    /// Encoded value (no TLV header).
    pub strand: Strand,
}

impl DnaValue {
    /// Create a Null value.
    #[must_use]
    pub fn null() -> Self {
        Self {
            dtype: DnaType::Null,
            strand: Strand::new(Vec::new()),
        }
    }

    /// Create a Bool value.
    #[must_use]
    pub fn bool(b: bool) -> Self {
        let nuc = if b { Nucleotide::T } else { Nucleotide::A };
        Self {
            dtype: DnaType::Bool,
            strand: Strand::new(vec![nuc]),
        }
    }

    /// Create an Int value (i64).
    #[must_use]
    pub fn int(n: i64) -> Self {
        let encoded = storage::encode(&n.to_le_bytes());
        Self {
            dtype: DnaType::Int,
            strand: encoded,
        }
    }

    /// Create a Float value (f64).
    #[must_use]
    pub fn float(n: f64) -> Self {
        let encoded = storage::encode(&n.to_bits().to_le_bytes());
        Self {
            dtype: DnaType::Float,
            strand: encoded,
        }
    }

    /// Create a Text value.
    #[must_use]
    pub fn text(s: &str) -> Self {
        let encoded = storage::encode_str(s);
        Self {
            dtype: DnaType::Text,
            strand: encoded,
        }
    }

    /// Extract as bool.
    pub fn as_bool(&self) -> Result<bool> {
        if self.dtype != DnaType::Bool {
            return Err(DnaError::TypeMismatch {
                expected: "Bool".into(),
                found: self.dtype.name().into(),
            });
        }
        if self.strand.bases.is_empty() {
            return Ok(false);
        }
        Ok(self.strand.bases[0] == Nucleotide::T)
    }

    /// Extract as i64.
    pub fn as_int(&self) -> Result<i64> {
        if self.dtype != DnaType::Int {
            return Err(DnaError::TypeMismatch {
                expected: "Int".into(),
                found: self.dtype.name().into(),
            });
        }
        let bytes = storage::decode(&self.strand)?;
        if bytes.len() < 8 {
            return Err(DnaError::InvalidTlv("int requires 8 bytes".into()));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        Ok(i64::from_le_bytes(arr))
    }

    /// Extract as f64.
    pub fn as_float(&self) -> Result<f64> {
        if self.dtype != DnaType::Float {
            return Err(DnaError::TypeMismatch {
                expected: "Float".into(),
                found: self.dtype.name().into(),
            });
        }
        let bytes = storage::decode(&self.strand)?;
        if bytes.len() < 8 {
            return Err(DnaError::InvalidTlv("float requires 8 bytes".into()));
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        Ok(f64::from_bits(u64::from_le_bytes(arr)))
    }

    /// Extract as String.
    pub fn as_text(&self) -> Result<String> {
        if self.dtype != DnaType::Text {
            return Err(DnaError::TypeMismatch {
                expected: "Text".into(),
                found: self.dtype.name().into(),
            });
        }
        storage::decode_str(&self.strand)
    }

    /// Encode this value as TLV: [type: 3 nucs][length: 16 nucs][value: N nucs].
    pub fn encode_tlv(&self) -> Result<Strand> {
        let type_codon = self.dtype.to_codon()?;
        let value_nucs = &self.strand.bases;
        let length = value_nucs.len() as u32;
        let length_nucs = encode_u32_codons(length)?;

        let mut bases = Vec::with_capacity(3 + 16 + value_nucs.len());
        // Type tag: 3 nucleotides
        bases.push(type_codon.0);
        bases.push(type_codon.1);
        bases.push(type_codon.2);
        // Length: 16 nucleotides
        bases.extend_from_slice(&length_nucs);
        // Value
        bases.extend_from_slice(value_nucs);

        Ok(Strand::new(bases))
    }

    /// Decode a TLV-encoded value starting at `offset` in `bases`.
    /// Returns (value, bytes_consumed).
    pub fn decode_tlv(bases: &[Nucleotide], offset: usize) -> Result<(Self, usize)> {
        let header_size = 3 + 16; // type codon + u32 length
        if offset + header_size > bases.len() {
            return Err(DnaError::InvalidTlv("truncated TLV header".into()));
        }

        // Type tag
        let type_codon = Codon(bases[offset], bases[offset + 1], bases[offset + 2]);
        let dtype = DnaType::from_codon(&type_codon)?;

        // Length
        let length = decode_u32_codons(&bases[offset + 3..offset + 3 + 16])? as usize;

        let value_start = offset + header_size;
        if value_start + length > bases.len() {
            return Err(DnaError::InvalidTlv(format!(
                "truncated TLV value: need {length} nucs, have {}",
                bases.len() - value_start
            )));
        }

        let value_bases = bases[value_start..value_start + length].to_vec();
        let total = header_size + length;

        Ok((
            Self {
                dtype,
                strand: Strand::new(value_bases),
            },
            total,
        ))
    }
}

impl fmt::Display for DnaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.dtype {
            DnaType::Null => write!(f, "Null"),
            DnaType::Bool => {
                let b = self.as_bool().map_err(|_| fmt::Error)?;
                write!(f, "Bool({b})")
            }
            DnaType::Int => {
                let n = self.as_int().map_err(|_| fmt::Error)?;
                write!(f, "Int({n})")
            }
            DnaType::Float => {
                let n = self.as_float().map_err(|_| fmt::Error)?;
                write!(f, "Float({n})")
            }
            DnaType::Text => {
                let s = self.as_text().map_err(|_| fmt::Error)?;
                write!(f, "Text(\"{s}\")")
            }
            _ => write!(f, "{}(<{} nucs>)", self.dtype, self.strand.len()),
        }
    }
}

impl Eq for DnaValue {}

// ---------------------------------------------------------------------------
// DnaArray -- Ordered collection (T2-C, sigma Sequence)
// ---------------------------------------------------------------------------

/// Homogeneous ordered collection of DNA values.
///
/// Tier: T2-C (sigma Sequence + N Quantity + partial-boundary)
///
/// Format: `[element_type: 3 nucs][count: 16 nucs][elem_0_tlv]...[elem_n_tlv]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaArray {
    /// Element type (all elements must match).
    pub element_type: DnaType,
    /// Elements in order.
    pub elements: Vec<DnaValue>,
}

impl DnaArray {
    /// Create an empty array with the given element type.
    #[must_use]
    pub fn new(element_type: DnaType) -> Self {
        Self {
            element_type,
            elements: Vec::new(),
        }
    }

    /// Push a value, type-checking against element_type.
    pub fn push(&mut self, value: DnaValue) -> Result<()> {
        if value.dtype != self.element_type {
            return Err(DnaError::TypeMismatch {
                expected: self.element_type.name().into(),
                found: value.dtype.name().into(),
            });
        }
        self.elements.push(value);
        Ok(())
    }

    /// Get element by index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&DnaValue> {
        self.elements.get(index)
    }

    /// Pop the last element.
    pub fn pop(&mut self) -> Option<DnaValue> {
        self.elements.pop()
    }

    /// Number of elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// True if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Iterate over elements.
    pub fn iter(&self) -> impl Iterator<Item = &DnaValue> {
        self.elements.iter()
    }

    /// Encode as DNA strand.
    pub fn encode(&self) -> Result<Strand> {
        let type_codon = self.element_type.to_codon()?;
        let count = self.elements.len() as u32;
        let count_nucs = encode_u32_codons(count)?;

        let mut bases = Vec::new();
        // Element type: 3 nucs
        bases.push(type_codon.0);
        bases.push(type_codon.1);
        bases.push(type_codon.2);
        // Count: 16 nucs
        bases.extend_from_slice(&count_nucs);

        // Elements as TLV
        for elem in &self.elements {
            let tlv = elem.encode_tlv()?;
            bases.extend_from_slice(&tlv.bases);
        }

        Ok(Strand::new(bases))
    }

    /// Decode from DNA strand.
    pub fn decode(strand: &Strand) -> Result<Self> {
        let bases = &strand.bases;
        if bases.len() < 19 {
            return Err(DnaError::InvalidTlv("array too short".into()));
        }

        let type_codon = Codon(bases[0], bases[1], bases[2]);
        let element_type = DnaType::from_codon(&type_codon)?;
        let count = decode_u32_codons(&bases[3..19])? as usize;

        let mut elements = Vec::with_capacity(count);
        let mut offset = 19; // 3 + 16

        for _ in 0..count {
            let (val, consumed) = DnaValue::decode_tlv(bases, offset)?;
            elements.push(val);
            offset += consumed;
        }

        Ok(Self {
            element_type,
            elements,
        })
    }
}

impl fmt::Display for DnaArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items: Vec<String> = self.elements.iter().map(|v| format!("{v}")).collect();
        write!(
            f,
            "[{}; {}]: [{}]",
            self.element_type,
            self.len(),
            items.join(", ")
        )
    }
}

// ---------------------------------------------------------------------------
// DnaRecord -- Named fields (T2-C, mu Mapping)
// ---------------------------------------------------------------------------

/// Named field collection (like a struct or JSON object).
///
/// Tier: T2-C (mu Mapping + lambda Location + sigma Sequence)
///
/// Format: `[field_count: 16 nucs][[name_text_tlv][value_tlv]]...`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaRecord {
    /// Ordered (name, value) pairs.
    pub fields: Vec<(String, DnaValue)>,
}

impl DnaRecord {
    /// Create an empty record.
    #[must_use]
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    /// Set a field (overwrites if exists).
    pub fn set(&mut self, name: String, value: DnaValue) {
        for entry in &mut self.fields {
            if entry.0 == name {
                entry.1 = value;
                return;
            }
        }
        self.fields.push((name, value));
    }

    /// Get a field by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&DnaValue> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    /// Remove a field by name, returning it if it existed.
    pub fn remove(&mut self, name: &str) -> Option<DnaValue> {
        if let Some(pos) = self.fields.iter().position(|(n, _)| n == name) {
            Some(self.fields.remove(pos).1)
        } else {
            None
        }
    }

    /// Check if a field exists.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.fields.iter().any(|(n, _)| n == name)
    }

    /// Get field names.
    pub fn field_names(&self) -> Vec<&str> {
        self.fields.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Number of fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// True if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Iterate over (name, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = &(String, DnaValue)> {
        self.fields.iter()
    }

    /// Encode as DNA strand.
    pub fn encode(&self) -> Result<Strand> {
        let count = self.fields.len() as u32;
        let count_nucs = encode_u32_codons(count)?;

        let mut bases = Vec::new();
        bases.extend_from_slice(&count_nucs);

        for (name, value) in &self.fields {
            // Name as Text TLV
            let name_val = DnaValue::text(name);
            let name_tlv = name_val.encode_tlv()?;
            bases.extend_from_slice(&name_tlv.bases);

            // Value TLV
            let val_tlv = value.encode_tlv()?;
            bases.extend_from_slice(&val_tlv.bases);
        }

        Ok(Strand::new(bases))
    }

    /// Decode from DNA strand.
    pub fn decode(strand: &Strand) -> Result<Self> {
        let bases = &strand.bases;
        if bases.len() < 16 {
            return Err(DnaError::InvalidTlv("record too short".into()));
        }

        let count = decode_u32_codons(&bases[..16])? as usize;
        let mut fields = Vec::with_capacity(count);
        let mut offset = 16;

        for _ in 0..count {
            // Name TLV
            let (name_val, name_consumed) = DnaValue::decode_tlv(bases, offset)?;
            let name = name_val.as_text()?;
            offset += name_consumed;

            // Value TLV
            let (value, val_consumed) = DnaValue::decode_tlv(bases, offset)?;
            offset += val_consumed;

            fields.push((name, value));
        }

        Ok(Self { fields })
    }
}

impl Default for DnaRecord {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DnaRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pairs: Vec<String> = self
            .fields
            .iter()
            .map(|(n, v)| format!("{n}: {v}"))
            .collect();
        write!(f, "{{{}}}", pairs.join(", "))
    }
}

// ---------------------------------------------------------------------------
// DnaMap -- Key-value pairs (T2-C, mu Mapping)
// ---------------------------------------------------------------------------

/// Key-value map where both keys and values are DnaValues.
///
/// Tier: T2-C (mu Mapping + kappa Comparison + Existence)
///
/// Format: `[entry_count: 16 nucs][[key_tlv][value_tlv]]...`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaMap {
    /// Key-value entries.
    pub entries: Vec<(DnaValue, DnaValue)>,
}

impl DnaMap {
    /// Create an empty map.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Insert a key-value pair (overwrites if key exists).
    pub fn insert(&mut self, key: DnaValue, value: DnaValue) {
        for entry in &mut self.entries {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    /// Get value by key.
    #[must_use]
    pub fn get(&self, key: &DnaValue) -> Option<&DnaValue> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Check if key exists.
    #[must_use]
    pub fn contains_key(&self, key: &DnaValue) -> bool {
        self.entries.iter().any(|(k, _)| k == key)
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<&DnaValue> {
        self.entries.iter().map(|(k, _)| k).collect()
    }

    /// Get all values.
    pub fn values(&self) -> Vec<&DnaValue> {
        self.entries.iter().map(|(_, v)| v).collect()
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over (key, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = &(DnaValue, DnaValue)> {
        self.entries.iter()
    }

    /// Encode as DNA strand.
    pub fn encode(&self) -> Result<Strand> {
        let count = self.entries.len() as u32;
        let count_nucs = encode_u32_codons(count)?;

        let mut bases = Vec::new();
        bases.extend_from_slice(&count_nucs);

        for (key, value) in &self.entries {
            let key_tlv = key.encode_tlv()?;
            bases.extend_from_slice(&key_tlv.bases);
            let val_tlv = value.encode_tlv()?;
            bases.extend_from_slice(&val_tlv.bases);
        }

        Ok(Strand::new(bases))
    }

    /// Decode from DNA strand.
    pub fn decode(strand: &Strand) -> Result<Self> {
        let bases = &strand.bases;
        if bases.len() < 16 {
            return Err(DnaError::InvalidTlv("map too short".into()));
        }

        let count = decode_u32_codons(&bases[..16])? as usize;
        let mut entries = Vec::with_capacity(count);
        let mut offset = 16;

        for _ in 0..count {
            let (key, key_consumed) = DnaValue::decode_tlv(bases, offset)?;
            offset += key_consumed;
            let (value, val_consumed) = DnaValue::decode_tlv(bases, offset)?;
            offset += val_consumed;
            entries.push((key, value));
        }

        Ok(Self { entries })
    }
}

impl Default for DnaMap {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DnaMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pairs: Vec<String> = self
            .entries
            .iter()
            .map(|(k, v)| format!("{k} => {v}"))
            .collect();
        write!(f, "{{{}}}", pairs.join(", "))
    }
}

// ---------------------------------------------------------------------------
// DnaFrame -- Tabular data (T3, sigma+mu+N+partial-boundary+lambda)
// ---------------------------------------------------------------------------

/// Tabular data with named columns and typed rows.
///
/// Tier: T3 (sigma + mu + N + partial-boundary + lambda)
///
/// Format: `[col_count: 8 nucs][row_count: 16 nucs][col_names...][rows...]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnaFrame {
    /// Column names.
    pub columns: Vec<String>,
    /// Rows (each row is a DnaRecord with fields matching columns).
    pub rows: Vec<DnaRecord>,
}

impl DnaFrame {
    /// Create an empty frame with the given columns.
    #[must_use]
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
        }
    }

    /// Add a row, checking that it has exactly the right column names.
    pub fn add_row(&mut self, row: DnaRecord) -> Result<()> {
        let row_names: Vec<&str> = row.field_names();
        // Check that every column is present
        for col in &self.columns {
            if !row_names.contains(&col.as_str()) {
                return Err(DnaError::SchemaViolation(format!(
                    "missing column '{col}' in row"
                )));
            }
        }
        // Check that row doesn't have extra columns
        for name in &row_names {
            if !self.columns.iter().any(|c| c.as_str() == *name) {
                return Err(DnaError::SchemaViolation(format!(
                    "unexpected column '{name}' in row"
                )));
            }
        }
        self.rows.push(row);
        Ok(())
    }

    /// Get a row by index.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<&DnaRecord> {
        self.rows.get(index)
    }

    /// Extract a column as a vector of values.
    pub fn column(&self, name: &str) -> Result<Vec<&DnaValue>> {
        if !self.columns.iter().any(|c| c == name) {
            return Err(DnaError::FieldNotFound(name.into()));
        }
        let mut col = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            if let Some(v) = row.get(name) {
                col.push(v);
            }
        }
        Ok(col)
    }

    /// Shape as (rows, columns).
    #[must_use]
    pub fn shape(&self) -> (usize, usize) {
        (self.rows.len(), self.columns.len())
    }

    /// Number of rows.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns.
    #[must_use]
    pub fn col_count(&self) -> usize {
        self.columns.len()
    }

    /// True if no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Encode as DNA strand.
    pub fn encode(&self) -> Result<Strand> {
        let col_count = self.columns.len() as u16;
        let row_count = self.rows.len() as u32;

        let mut bases = Vec::new();
        // Column count: 8 nucs
        bases.extend_from_slice(&encode_u16_codons(col_count)?);
        // Row count: 16 nucs
        bases.extend_from_slice(&encode_u32_codons(row_count)?);

        // Column names as Text TLVs
        for col in &self.columns {
            let name_val = DnaValue::text(col);
            let tlv = name_val.encode_tlv()?;
            bases.extend_from_slice(&tlv.bases);
        }

        // Rows as encoded records
        for row in &self.rows {
            let encoded = row.encode()?;
            // Prefix with record strand length
            let len_nucs = encode_u32_codons(encoded.bases.len() as u32)?;
            bases.extend_from_slice(&len_nucs);
            bases.extend_from_slice(&encoded.bases);
        }

        Ok(Strand::new(bases))
    }

    /// Decode from DNA strand.
    pub fn decode(strand: &Strand) -> Result<Self> {
        let bases = &strand.bases;
        if bases.len() < 24 {
            return Err(DnaError::InvalidTlv("frame too short".into()));
        }

        let col_count = decode_u16_codons(&bases[..8])? as usize;
        let row_count = decode_u32_codons(&bases[8..24])? as usize;

        let mut offset = 24;
        let mut columns = Vec::with_capacity(col_count);

        for _ in 0..col_count {
            let (name_val, consumed) = DnaValue::decode_tlv(bases, offset)?;
            columns.push(name_val.as_text()?);
            offset += consumed;
        }

        let mut rows = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            let row_len = decode_u32_codons(&bases[offset..offset + 16])? as usize;
            offset += 16;
            let row_strand = Strand::new(bases[offset..offset + row_len].to_vec());
            let record = DnaRecord::decode(&row_strand)?;
            rows.push(record);
            offset += row_len;
        }

        Ok(Self { columns, rows })
    }
}

impl fmt::Display for DnaFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let col_names: Vec<&str> = self.columns.iter().map(|s| s.as_str()).collect();
        write!(
            f,
            "Frame({}x{}): [{}]",
            self.rows.len(),
            self.columns.len(),
            col_names.join(", ")
        )
    }
}

// ---------------------------------------------------------------------------
// Biological Operations
// ---------------------------------------------------------------------------

/// Restriction -- extract field by cutting at name boundaries.
pub fn restrict(record: &DnaRecord, field: &str) -> Result<DnaValue> {
    record
        .get(field)
        .cloned()
        .ok_or_else(|| DnaError::FieldNotFound(field.into()))
}

/// Restriction -- extract a range from an array (subslice).
pub fn restrict_range(array: &DnaArray, start: usize, end: usize) -> Result<DnaArray> {
    if start > end || end > array.len() {
        return Err(DnaError::IndexOutOfBounds(end, array.len()));
    }
    let mut result = DnaArray::new(array.element_type);
    for elem in &array.elements[start..end] {
        result.elements.push(elem.clone());
    }
    Ok(result)
}

/// Ligation -- join two arrays of the same type.
pub fn ligate_arrays(a: &DnaArray, b: &DnaArray) -> Result<DnaArray> {
    if a.element_type != b.element_type {
        return Err(DnaError::CollectionTypeMismatch);
    }
    let mut result = DnaArray::new(a.element_type);
    for elem in &a.elements {
        result.elements.push(elem.clone());
    }
    for elem in &b.elements {
        result.elements.push(elem.clone());
    }
    Ok(result)
}

/// Ligation -- merge two records (b's fields override a's).
#[must_use]
pub fn ligate_records(a: &DnaRecord, b: &DnaRecord) -> DnaRecord {
    let mut result = a.clone();
    for (name, value) in &b.fields {
        result.set(name.clone(), value.clone());
    }
    result
}

/// Splicing -- insert or overwrite a field in a record.
pub fn splice_field(record: &mut DnaRecord, name: &str, value: DnaValue) {
    record.set(name.to_string(), value);
}

/// Excision -- remove a field from a record.
pub fn excise_field(record: &mut DnaRecord, name: &str) -> Result<DnaValue> {
    record
        .remove(name)
        .ok_or_else(|| DnaError::FieldNotFound(name.into()))
}

/// Transcription -- human-readable projection of a value.
#[must_use]
pub fn transcribe_value(value: &DnaValue) -> String {
    format!("{value}")
}

/// Transcription -- human-readable projection of a record.
#[must_use]
pub fn transcribe_record(record: &DnaRecord) -> String {
    format!("{record}")
}

/// Transcription -- human-readable projection of a frame.
#[must_use]
pub fn transcribe_frame(frame: &DnaFrame) -> String {
    let mut out = format!("{frame}\n");
    for (i, row) in frame.rows.iter().enumerate() {
        out.push_str(&format!("  row {i}: {row}\n"));
    }
    out
}

// ---------------------------------------------------------------------------
// Tests (~60)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // === 2a: DnaType tests (6) ===

    #[test]
    fn dtype_codon_roundtrip() {
        let types = [
            DnaType::Null,
            DnaType::Bool,
            DnaType::Int,
            DnaType::Float,
            DnaType::Text,
            DnaType::Array,
            DnaType::Record,
            DnaType::Map,
            DnaType::Frame,
        ];
        for dt in &types {
            let codon = dt.to_codon();
            assert!(codon.is_ok(), "to_codon failed for {dt:?}");
            if let Ok(c) = codon {
                let recovered = DnaType::from_codon(&c);
                assert!(recovered.is_ok(), "from_codon failed for {dt:?}");
                if let Ok(r) = recovered {
                    assert_eq!(*dt, r, "roundtrip failed for {dt:?}");
                }
            }
        }
    }

    #[test]
    fn dtype_name() {
        assert_eq!(DnaType::Null.name(), "Null");
        assert_eq!(DnaType::Int.name(), "Int");
        assert_eq!(DnaType::Text.name(), "Text");
        assert_eq!(DnaType::Frame.name(), "Frame");
    }

    #[test]
    fn dtype_is_scalar() {
        assert!(DnaType::Null.is_scalar());
        assert!(DnaType::Bool.is_scalar());
        assert!(DnaType::Int.is_scalar());
        assert!(DnaType::Float.is_scalar());
        assert!(DnaType::Text.is_scalar());
        assert!(!DnaType::Array.is_scalar());
        assert!(!DnaType::Record.is_scalar());
    }

    #[test]
    fn dtype_is_collection() {
        assert!(DnaType::Array.is_collection());
        assert!(DnaType::Record.is_collection());
        assert!(DnaType::Map.is_collection());
        assert!(DnaType::Frame.is_collection());
        assert!(!DnaType::Int.is_collection());
    }

    #[test]
    fn dtype_invalid_index() {
        assert!(DnaType::from_index(5).is_err());
        assert!(DnaType::from_index(7).is_err());
        assert!(DnaType::from_index(63).is_err());
    }

    #[test]
    fn dtype_display() {
        assert_eq!(format!("{}", DnaType::Int), "Int");
        assert_eq!(format!("{}", DnaType::Record), "Record");
        assert_eq!(format!("{}", DnaType::Frame), "Frame");
    }

    // === 2c: DnaValue tests (10) ===

    #[test]
    fn value_null() {
        let v = DnaValue::null();
        assert_eq!(v.dtype, DnaType::Null);
        assert!(v.strand.is_empty());
        assert_eq!(format!("{v}"), "Null");
    }

    #[test]
    fn value_bool() {
        let t = DnaValue::bool(true);
        let f = DnaValue::bool(false);
        assert!(t.as_bool().is_ok());
        if let Ok(b) = t.as_bool() {
            assert!(b);
        }
        if let Ok(b) = f.as_bool() {
            assert!(!b);
        }
    }

    #[test]
    fn value_int() {
        let v = DnaValue::int(42);
        assert_eq!(v.dtype, DnaType::Int);
        if let Ok(n) = v.as_int() {
            assert_eq!(n, 42);
        }
    }

    #[test]
    fn value_float() {
        let v = DnaValue::float(3.14);
        assert_eq!(v.dtype, DnaType::Float);
        if let Ok(n) = v.as_float() {
            assert!((n - 3.14).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn value_text() {
        let v = DnaValue::text("hello");
        assert_eq!(v.dtype, DnaType::Text);
        if let Ok(s) = v.as_text() {
            assert_eq!(s, "hello");
        }
    }

    #[test]
    fn value_tlv_roundtrip() {
        let values = [
            DnaValue::null(),
            DnaValue::bool(true),
            DnaValue::int(-99),
            DnaValue::float(2.718),
            DnaValue::text("DNA data"),
        ];
        for v in &values {
            let encoded = v.encode_tlv();
            assert!(encoded.is_ok(), "encode_tlv failed for {v:?}");
            if let Ok(strand) = encoded {
                let decoded = DnaValue::decode_tlv(&strand.bases, 0);
                assert!(decoded.is_ok(), "decode_tlv failed for {v:?}");
                if let Ok((recovered, _consumed)) = decoded {
                    assert_eq!(v.dtype, recovered.dtype, "type mismatch for {v:?}");
                    assert_eq!(
                        v.strand.bases, recovered.strand.bases,
                        "strand mismatch for {v:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn value_accessor_type_error() {
        let v = DnaValue::int(42);
        assert!(v.as_bool().is_err());
        assert!(v.as_float().is_err());
        assert!(v.as_text().is_err());

        let t = DnaValue::text("hi");
        assert!(t.as_int().is_err());
    }

    #[test]
    fn value_int_extremes() {
        for n in [i64::MIN, i64::MAX, 0, -1, 1] {
            let v = DnaValue::int(n);
            if let Ok(recovered) = v.as_int() {
                assert_eq!(recovered, n, "int extreme failed for {n}");
            }
        }
    }

    #[test]
    fn value_empty_text() {
        let v = DnaValue::text("");
        if let Ok(s) = v.as_text() {
            assert!(s.is_empty());
        }
    }

    #[test]
    fn value_display() {
        assert_eq!(format!("{}", DnaValue::null()), "Null");
        assert_eq!(format!("{}", DnaValue::bool(true)), "Bool(true)");
        assert_eq!(format!("{}", DnaValue::int(42)), "Int(42)");
        let text_display = format!("{}", DnaValue::text("hi"));
        assert!(text_display.contains("Text"));
        assert!(text_display.contains("hi"));
    }

    // === 2d: DnaArray tests (8) ===

    #[test]
    fn array_empty() {
        let arr = DnaArray::new(DnaType::Int);
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
        assert_eq!(arr.element_type, DnaType::Int);
    }

    #[test]
    fn array_push_get() {
        let mut arr = DnaArray::new(DnaType::Int);
        assert!(arr.push(DnaValue::int(10)).is_ok());
        assert!(arr.push(DnaValue::int(20)).is_ok());
        assert_eq!(arr.len(), 2);
        if let Some(v) = arr.get(0) {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 10);
            }
        }
        if let Some(v) = arr.get(1) {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 20);
            }
        }
        assert!(arr.get(2).is_none());
    }

    #[test]
    fn array_pop() {
        let mut arr = DnaArray::new(DnaType::Int);
        assert!(arr.push(DnaValue::int(1)).is_ok());
        assert!(arr.push(DnaValue::int(2)).is_ok());
        let popped = arr.pop();
        assert!(popped.is_some());
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn array_type_mismatch() {
        let mut arr = DnaArray::new(DnaType::Int);
        assert!(arr.push(DnaValue::text("wrong")).is_err());
    }

    #[test]
    fn array_encode_decode() {
        let mut arr = DnaArray::new(DnaType::Int);
        assert!(arr.push(DnaValue::int(100)).is_ok());
        assert!(arr.push(DnaValue::int(200)).is_ok());
        assert!(arr.push(DnaValue::int(300)).is_ok());

        let encoded = arr.encode();
        assert!(encoded.is_ok());
        if let Ok(strand) = encoded {
            let decoded = DnaArray::decode(&strand);
            assert!(decoded.is_ok());
            if let Ok(recovered) = decoded {
                assert_eq!(recovered.element_type, DnaType::Int);
                assert_eq!(recovered.len(), 3);
                if let Some(v) = recovered.get(0) {
                    if let Ok(n) = v.as_int() {
                        assert_eq!(n, 100);
                    }
                }
                if let Some(v) = recovered.get(2) {
                    if let Ok(n) = v.as_int() {
                        assert_eq!(n, 300);
                    }
                }
            }
        }
    }

    #[test]
    fn array_iter() {
        let mut arr = DnaArray::new(DnaType::Int);
        assert!(arr.push(DnaValue::int(1)).is_ok());
        assert!(arr.push(DnaValue::int(2)).is_ok());
        let count = arr.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn array_of_text() {
        let mut arr = DnaArray::new(DnaType::Text);
        assert!(arr.push(DnaValue::text("alpha")).is_ok());
        assert!(arr.push(DnaValue::text("beta")).is_ok());
        assert_eq!(arr.len(), 2);

        let encoded = arr.encode();
        assert!(encoded.is_ok());
        if let Ok(strand) = encoded {
            let decoded = DnaArray::decode(&strand);
            assert!(decoded.is_ok());
            if let Ok(recovered) = decoded {
                assert_eq!(recovered.len(), 2);
                if let Some(v) = recovered.get(1) {
                    if let Ok(s) = v.as_text() {
                        assert_eq!(s, "beta");
                    }
                }
            }
        }
    }

    #[test]
    fn array_len() {
        let mut arr = DnaArray::new(DnaType::Bool);
        assert!(arr.push(DnaValue::bool(true)).is_ok());
        assert!(arr.push(DnaValue::bool(false)).is_ok());
        assert!(arr.push(DnaValue::bool(true)).is_ok());
        assert_eq!(arr.len(), 3);
        assert!(!arr.is_empty());
    }

    // === 2e: DnaRecord tests (10) ===

    #[test]
    fn record_empty() {
        let rec = DnaRecord::new();
        assert!(rec.is_empty());
        assert_eq!(rec.len(), 0);
    }

    #[test]
    fn record_set_get() {
        let mut rec = DnaRecord::new();
        rec.set("name".into(), DnaValue::text("Alice"));
        rec.set("age".into(), DnaValue::int(30));
        assert_eq!(rec.len(), 2);
        if let Some(v) = rec.get("name") {
            if let Ok(s) = v.as_text() {
                assert_eq!(s, "Alice");
            }
        }
        if let Some(v) = rec.get("age") {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 30);
            }
        }
    }

    #[test]
    fn record_overwrite() {
        let mut rec = DnaRecord::new();
        rec.set("x".into(), DnaValue::int(1));
        rec.set("x".into(), DnaValue::int(2));
        assert_eq!(rec.len(), 1);
        if let Some(v) = rec.get("x") {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 2);
            }
        }
    }

    #[test]
    fn record_remove() {
        let mut rec = DnaRecord::new();
        rec.set("a".into(), DnaValue::int(1));
        rec.set("b".into(), DnaValue::int(2));
        let removed = rec.remove("a");
        assert!(removed.is_some());
        assert_eq!(rec.len(), 1);
        assert!(rec.get("a").is_none());
    }

    #[test]
    fn record_field_names() {
        let mut rec = DnaRecord::new();
        rec.set("x".into(), DnaValue::int(1));
        rec.set("y".into(), DnaValue::int(2));
        let names = rec.field_names();
        assert!(names.contains(&"x"));
        assert!(names.contains(&"y"));
    }

    #[test]
    fn record_contains() {
        let mut rec = DnaRecord::new();
        rec.set("hello".into(), DnaValue::null());
        assert!(rec.contains("hello"));
        assert!(!rec.contains("world"));
    }

    #[test]
    fn record_encode_decode() {
        let mut rec = DnaRecord::new();
        rec.set("name".into(), DnaValue::text("Bob"));
        rec.set("score".into(), DnaValue::int(95));

        let encoded = rec.encode();
        assert!(encoded.is_ok());
        if let Ok(strand) = encoded {
            let decoded = DnaRecord::decode(&strand);
            assert!(decoded.is_ok());
            if let Ok(recovered) = decoded {
                assert_eq!(recovered.len(), 2);
                if let Some(v) = recovered.get("name") {
                    if let Ok(s) = v.as_text() {
                        assert_eq!(s, "Bob");
                    }
                }
                if let Some(v) = recovered.get("score") {
                    if let Ok(n) = v.as_int() {
                        assert_eq!(n, 95);
                    }
                }
            }
        }
    }

    #[test]
    fn record_nested() {
        // A record containing various types
        let mut rec = DnaRecord::new();
        rec.set("flag".into(), DnaValue::bool(true));
        rec.set("pi".into(), DnaValue::float(3.14));
        rec.set("label".into(), DnaValue::text("nested"));
        rec.set("empty".into(), DnaValue::null());
        assert_eq!(rec.len(), 4);
    }

    #[test]
    fn record_iter() {
        let mut rec = DnaRecord::new();
        rec.set("a".into(), DnaValue::int(1));
        rec.set("b".into(), DnaValue::int(2));
        let count = rec.iter().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn record_display() {
        let mut rec = DnaRecord::new();
        rec.set("x".into(), DnaValue::int(42));
        let s = format!("{rec}");
        assert!(s.contains("x:"));
        assert!(s.contains("Int(42)"));
    }

    // === 2f: DnaMap tests (8) ===

    #[test]
    fn map_empty() {
        let m = DnaMap::new();
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn map_insert_get() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::text("key1"), DnaValue::int(100));
        m.insert(DnaValue::text("key2"), DnaValue::int(200));
        assert_eq!(m.len(), 2);

        let key = DnaValue::text("key1");
        if let Some(v) = m.get(&key) {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 100);
            }
        }
    }

    #[test]
    fn map_contains() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::int(1), DnaValue::text("one"));
        let key = DnaValue::int(1);
        let missing = DnaValue::int(2);
        assert!(m.contains_key(&key));
        assert!(!m.contains_key(&missing));
    }

    #[test]
    fn map_keys_values() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::text("a"), DnaValue::int(1));
        m.insert(DnaValue::text("b"), DnaValue::int(2));
        assert_eq!(m.keys().len(), 2);
        assert_eq!(m.values().len(), 2);
    }

    #[test]
    fn map_encode_decode() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::text("x"), DnaValue::int(10));
        m.insert(DnaValue::text("y"), DnaValue::int(20));

        let encoded = m.encode();
        assert!(encoded.is_ok());
        if let Ok(strand) = encoded {
            let decoded = DnaMap::decode(&strand);
            assert!(decoded.is_ok());
            if let Ok(recovered) = decoded {
                assert_eq!(recovered.len(), 2);
                let key = DnaValue::text("x");
                if let Some(v) = recovered.get(&key) {
                    if let Ok(n) = v.as_int() {
                        assert_eq!(n, 10);
                    }
                }
            }
        }
    }

    #[test]
    fn map_overwrite() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::text("k"), DnaValue::int(1));
        m.insert(DnaValue::text("k"), DnaValue::int(2));
        assert_eq!(m.len(), 1);
        let key = DnaValue::text("k");
        if let Some(v) = m.get(&key) {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 2);
            }
        }
    }

    #[test]
    fn map_iter() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::int(1), DnaValue::text("one"));
        m.insert(DnaValue::int(2), DnaValue::text("two"));
        assert_eq!(m.iter().count(), 2);
    }

    #[test]
    fn map_display() {
        let mut m = DnaMap::new();
        m.insert(DnaValue::text("a"), DnaValue::int(1));
        let s = format!("{m}");
        assert!(s.contains("=>"));
    }

    // === 2g: DnaFrame tests (8) ===

    #[test]
    fn frame_empty() {
        let f = DnaFrame::new(vec!["name".into(), "age".into()]);
        assert!(f.is_empty());
        assert_eq!(f.col_count(), 2);
        assert_eq!(f.row_count(), 0);
        assert_eq!(f.shape(), (0, 2));
    }

    #[test]
    fn frame_add_row() {
        let mut f = DnaFrame::new(vec!["name".into(), "score".into()]);
        let mut row = DnaRecord::new();
        row.set("name".into(), DnaValue::text("Alice"));
        row.set("score".into(), DnaValue::int(95));
        assert!(f.add_row(row).is_ok());
        assert_eq!(f.row_count(), 1);
    }

    #[test]
    fn frame_column() {
        let mut f = DnaFrame::new(vec!["x".into(), "y".into()]);

        let mut r1 = DnaRecord::new();
        r1.set("x".into(), DnaValue::int(1));
        r1.set("y".into(), DnaValue::int(10));
        assert!(f.add_row(r1).is_ok());

        let mut r2 = DnaRecord::new();
        r2.set("x".into(), DnaValue::int(2));
        r2.set("y".into(), DnaValue::int(20));
        assert!(f.add_row(r2).is_ok());

        let col = f.column("x");
        assert!(col.is_ok());
        if let Ok(vals) = col {
            assert_eq!(vals.len(), 2);
        }

        assert!(f.column("z").is_err());
    }

    #[test]
    fn frame_shape() {
        let mut f = DnaFrame::new(vec!["a".into(), "b".into(), "c".into()]);
        let mut row = DnaRecord::new();
        row.set("a".into(), DnaValue::int(1));
        row.set("b".into(), DnaValue::int(2));
        row.set("c".into(), DnaValue::int(3));
        assert!(f.add_row(row).is_ok());
        assert_eq!(f.shape(), (1, 3));
    }

    #[test]
    fn frame_encode_decode() {
        let mut f = DnaFrame::new(vec!["name".into(), "val".into()]);

        let mut r1 = DnaRecord::new();
        r1.set("name".into(), DnaValue::text("a"));
        r1.set("val".into(), DnaValue::int(1));
        assert!(f.add_row(r1).is_ok());

        let mut r2 = DnaRecord::new();
        r2.set("name".into(), DnaValue::text("b"));
        r2.set("val".into(), DnaValue::int(2));
        assert!(f.add_row(r2).is_ok());

        let encoded = f.encode();
        assert!(encoded.is_ok());
        if let Ok(strand) = encoded {
            let decoded = DnaFrame::decode(&strand);
            assert!(decoded.is_ok());
            if let Ok(recovered) = decoded {
                assert_eq!(recovered.col_count(), 2);
                assert_eq!(recovered.row_count(), 2);
                assert_eq!(recovered.columns[0], "name");
                assert_eq!(recovered.columns[1], "val");
            }
        }
    }

    #[test]
    fn frame_schema_check() {
        let mut f = DnaFrame::new(vec!["x".into()]);

        // Missing column
        let empty_row = DnaRecord::new();
        assert!(f.add_row(empty_row).is_err());

        // Extra column
        let mut extra_row = DnaRecord::new();
        extra_row.set("x".into(), DnaValue::int(1));
        extra_row.set("y".into(), DnaValue::int(2));
        assert!(f.add_row(extra_row).is_err());
    }

    #[test]
    fn frame_display() {
        let f = DnaFrame::new(vec!["name".into(), "age".into()]);
        let s = format!("{f}");
        assert!(s.contains("Frame(0x2)"));
        assert!(s.contains("name"));
        assert!(s.contains("age"));
    }

    #[test]
    fn frame_multiple_rows() {
        let mut f = DnaFrame::new(vec!["id".into()]);
        for i in 0..5 {
            let mut row = DnaRecord::new();
            row.set("id".into(), DnaValue::int(i));
            assert!(f.add_row(row).is_ok());
        }
        assert_eq!(f.row_count(), 5);
        assert_eq!(f.shape(), (5, 1));
    }

    // === 2h: Biological Operations tests (7) ===

    #[test]
    fn bio_restrict_field() {
        let mut rec = DnaRecord::new();
        rec.set("name".into(), DnaValue::text("Alice"));
        rec.set("age".into(), DnaValue::int(30));

        let val = restrict(&rec, "name");
        assert!(val.is_ok());
        if let Ok(v) = val {
            if let Ok(s) = v.as_text() {
                assert_eq!(s, "Alice");
            }
        }
        assert!(restrict(&rec, "missing").is_err());
    }

    #[test]
    fn bio_restrict_range() {
        let mut arr = DnaArray::new(DnaType::Int);
        for i in 0..5 {
            assert!(arr.push(DnaValue::int(i * 10)).is_ok());
        }
        let slice = restrict_range(&arr, 1, 4);
        assert!(slice.is_ok());
        if let Ok(s) = slice {
            assert_eq!(s.len(), 3);
            if let Some(v) = s.get(0) {
                if let Ok(n) = v.as_int() {
                    assert_eq!(n, 10);
                }
            }
        }
        assert!(restrict_range(&arr, 3, 1).is_err());
        assert!(restrict_range(&arr, 0, 10).is_err());
    }

    #[test]
    fn bio_ligate_arrays() {
        let mut a = DnaArray::new(DnaType::Int);
        assert!(a.push(DnaValue::int(1)).is_ok());
        let mut b = DnaArray::new(DnaType::Int);
        assert!(b.push(DnaValue::int(2)).is_ok());

        let joined = ligate_arrays(&a, &b);
        assert!(joined.is_ok());
        if let Ok(j) = joined {
            assert_eq!(j.len(), 2);
        }

        // Type mismatch
        let c = DnaArray::new(DnaType::Text);
        assert!(ligate_arrays(&a, &c).is_err());
    }

    #[test]
    fn bio_ligate_records() {
        let mut a = DnaRecord::new();
        a.set("x".into(), DnaValue::int(1));
        let mut b = DnaRecord::new();
        b.set("y".into(), DnaValue::int(2));
        b.set("x".into(), DnaValue::int(99)); // Override

        let merged = ligate_records(&a, &b);
        assert_eq!(merged.len(), 2);
        if let Some(v) = merged.get("x") {
            if let Ok(n) = v.as_int() {
                assert_eq!(n, 99); // b overrides a
            }
        }
    }

    #[test]
    fn bio_splice_excise() {
        let mut rec = DnaRecord::new();
        rec.set("a".into(), DnaValue::int(1));

        splice_field(&mut rec, "b", DnaValue::int(2));
        assert_eq!(rec.len(), 2);

        let excised = excise_field(&mut rec, "b");
        assert!(excised.is_ok());
        assert_eq!(rec.len(), 1);

        assert!(excise_field(&mut rec, "missing").is_err());
    }

    #[test]
    fn bio_transcribe_record() {
        let mut rec = DnaRecord::new();
        rec.set("name".into(), DnaValue::text("Bob"));
        rec.set("age".into(), DnaValue::int(25));
        let t = transcribe_record(&rec);
        assert!(t.contains("name"));
        assert!(t.contains("Bob"));
        assert!(t.contains("Int(25)"));
    }

    #[test]
    fn bio_transcribe_frame() {
        let mut f = DnaFrame::new(vec!["id".into()]);
        let mut row = DnaRecord::new();
        row.set("id".into(), DnaValue::int(1));
        assert!(f.add_row(row).is_ok());
        let t = transcribe_frame(&f);
        assert!(t.contains("Frame"));
        assert!(t.contains("row 0"));
    }

    // === 2i: Display tests (3) ===

    #[test]
    fn display_dtype() {
        assert_eq!(format!("{}", DnaType::Null), "Null");
        assert_eq!(format!("{}", DnaType::Bool), "Bool");
        assert_eq!(format!("{}", DnaType::Array), "Array");
        assert_eq!(format!("{}", DnaType::Map), "Map");
    }

    #[test]
    fn display_value_variants() {
        assert_eq!(format!("{}", DnaValue::null()), "Null");
        assert_eq!(format!("{}", DnaValue::bool(false)), "Bool(false)");
        assert_eq!(format!("{}", DnaValue::int(0)), "Int(0)");
        let float_s = format!("{}", DnaValue::float(1.0));
        assert!(float_s.starts_with("Float("));
        let text_s = format!("{}", DnaValue::text("abc"));
        assert!(text_s.contains("abc"));
    }

    #[test]
    fn display_collections() {
        // Array
        let mut arr = DnaArray::new(DnaType::Int);
        assert!(arr.push(DnaValue::int(1)).is_ok());
        let s = format!("{arr}");
        assert!(s.contains("[Int; 1]"));

        // Record
        let mut rec = DnaRecord::new();
        rec.set("k".into(), DnaValue::int(42));
        let s = format!("{rec}");
        assert!(s.contains("k:"));

        // Map
        let mut m = DnaMap::new();
        m.insert(DnaValue::text("a"), DnaValue::int(1));
        let s = format!("{m}");
        assert!(s.contains("=>"));

        // Frame
        let f = DnaFrame::new(vec!["col".into()]);
        let s = format!("{f}");
        assert!(s.contains("Frame"));
    }
}
