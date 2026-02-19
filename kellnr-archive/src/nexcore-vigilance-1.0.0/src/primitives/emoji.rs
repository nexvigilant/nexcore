//! # Emoji Domain Primitives
//!
//! Twenty types grounding the Emoji domain to T1/T2 Lex Primitiva.
//! Each concept appears exactly once, decomposed to its irreducible
//! primitive composition. Together they model the full lifecycle:
//! encoding, rendering, social use, and governance.
//!
//! | Type | Dominant | Primitives | Tier |
//! |------|----------|------------|------|
//! | [`Codepoint`] | ∃ Existence | ∃ N | T2-P |
//! | [`Glyph`] | μ Mapping | ∃ μ | T2-P |
//! | [`UnicodeStandard`] | ∂ Boundary | ∂ κ σ | T2-C |
//! | [`SkinToneModifier`] | ς State | ς × | T2-P |
//! | [`Category`] | κ Comparison | κ Σ | T2-P |
//! | [`ZwjSequence`] | σ Sequence | σ × ∃ | T2-C |
//! | [`Shortcode`] | μ Mapping | μ ∃ | T2-P |
//! | [`Encoding`] | μ Mapping | μ N σ | T2-C |
//! | [`Rendering`] | μ Mapping | μ ∂ λ | T2-C |
//! | [`Reaction`] | → Causality | → ν | T2-P |
//! | [`Sentiment`] | μ Mapping | μ κ | T2-P |
//! | [`InputMethod`] | σ Sequence | σ ∂ | T2-P |
//! | [`EmojiVersion`] | σ Sequence | σ π N | T2-C |
//! | [`CulturalInterpretation`] | λ Location | μ λ κ | T2-C |
//! | [`UsageFrequency`] | ν Frequency | ν N | T2-P |
//! | [`FallbackDisplay`] | ∅ Void | ∅ μ | T2-P |
//! | [`Ambiguity`] | ∅ Void | κ ∅ μ | T2-C |
//! | [`EmojiComposition`] | × Product | × σ ρ | T2-C |
//! | [`ProposalProcess`] | ∂ Boundary | σ ∂ → | T2-C |
//! | [`EmojiEvolution`] | ∝ Irreversibility | ∝ σ Σ | T2-C |

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 1. ∃ (Existence) + N (Quantity) → Codepoint
// ============================================================================

/// Unicode codepoint — the numeric identity of a character.
///
/// Grounds ∃ (Existence): a codepoint IS an existence proof.
/// Every emoji's being is reducible to a single number.
///
/// # Domain Mappings
/// - U+1F600 = Grinning Face
/// - U+2764 = Red Heart
/// - Codepoints exist even before any platform renders them
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Codepoint {
    /// Unicode scalar value (0x0..=0x10FFFF)
    pub value: u32,
}

impl Codepoint {
    /// Creates a new codepoint from a raw value.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self { value }
    }

    /// Returns true if this falls in the emoji range (rough heuristic).
    #[must_use]
    pub const fn is_emoji_range(&self) -> bool {
        // Miscellaneous Symbols, Dingbats, Emoticons, Transport/Map,
        // Supplemental Symbols, and skin tone modifier range
        (self.value >= 0x2600 && self.value <= 0x27BF)
            || (self.value >= 0x1F300 && self.value <= 0x1FAFF)
            || (self.value >= 0x1F900 && self.value <= 0x1F9FF)
            || (self.value >= 0x1FA00 && self.value <= 0x1FA6F)
    }

    /// Returns true if this is a valid Unicode scalar value.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.value <= 0x10FFFF && !(self.value >= 0xD800 && self.value <= 0xDFFF)
    }

    /// Formats as U+XXXX notation.
    #[must_use]
    pub fn to_unicode_notation(&self) -> String {
        if self.value > 0xFFFF {
            format!("U+{:05X}", self.value)
        } else {
            format!("U+{:04X}", self.value)
        }
    }
}

impl fmt::Display for Codepoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "∃:{}", self.to_unicode_notation())
    }
}

// ============================================================================
// 2. μ (Mapping) + ∃ (Existence) → Glyph
// ============================================================================

/// Visual representation of a codepoint — the picture you see.
///
/// Grounds μ (Mapping): a glyph IS a mapping from number to visual.
/// The same codepoint maps to different glyphs across platforms.
///
/// # Domain Mappings
/// - Apple: round, detailed emoji art
/// - Google: blob-style (legacy) → round (modern)
/// - Samsung: often divergent interpretations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Glyph {
    /// The codepoint this glyph represents
    pub codepoint: u32,
    /// Human-readable description (CLDR short name)
    pub description: String,
}

impl Glyph {
    /// Creates a new glyph mapping.
    #[must_use]
    pub fn new(codepoint: u32, description: impl Into<String>) -> Self {
        Self {
            codepoint,
            description: description.into(),
        }
    }

    /// Returns the codepoint as a structured type.
    #[must_use]
    pub const fn codepoint(&self) -> Codepoint {
        Codepoint::new(self.codepoint)
    }

    /// Returns true if description is non-empty (glyph is named).
    #[must_use]
    pub fn is_named(&self) -> bool {
        !self.description.is_empty()
    }
}

impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "μ:U+{:04X}→{}", self.codepoint, self.description)
    }
}

// ============================================================================
// 3. ∂ (Boundary) + κ (Comparison) + σ (Sequence) → UnicodeStandard
// ============================================================================

/// The governing specification defining what IS and IS NOT a valid emoji.
///
/// Grounds ∂ (Boundary): the standard IS the boundary between valid
/// and invalid codepoints. It sequences versions (σ) and compares
/// characters against the rule set (κ).
///
/// # Domain Mappings
/// - Unicode 15.1: 3,782 emoji
/// - Each version adds to the boundary but never removes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnicodeStandard {
    /// Major version number
    pub version_major: u32,
    /// Minor version number
    pub version_minor: u32,
    /// Maximum valid codepoint in this version
    pub codepoint_ceiling: u32,
}

impl UnicodeStandard {
    /// Creates a new standard version descriptor.
    #[must_use]
    pub const fn new(major: u32, minor: u32, ceiling: u32) -> Self {
        Self {
            version_major: major,
            version_minor: minor,
            codepoint_ceiling: ceiling,
        }
    }

    /// Checks whether a codepoint falls within this standard's boundary.
    #[must_use]
    pub const fn contains(&self, cp: &Codepoint) -> bool {
        cp.value <= self.codepoint_ceiling
    }

    /// Returns true if this version is newer than another.
    #[must_use]
    pub const fn is_newer_than(&self, other: &Self) -> bool {
        self.version_major > other.version_major
            || (self.version_major == other.version_major
                && self.version_minor > other.version_minor)
    }
}

impl fmt::Display for UnicodeStandard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "∂:Unicode {}.{} (ceiling U+{:05X})",
            self.version_major, self.version_minor, self.codepoint_ceiling
        )
    }
}

// ============================================================================
// 4. ς (State) + × (Product) → SkinToneModifier
// ============================================================================

/// Fitzpatrick skin tone modifier applied to a base emoji.
///
/// Grounds ς (State): the modifier IS a state variant selector.
/// Combined with a base via product composition (×).
///
/// # Domain Mappings
/// - U+1F3FB (Light) through U+1F3FF (Dark)
/// - Not all emoji support skin tones
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkinToneModifier {
    /// Fitzpatrick Type I-II (U+1F3FB)
    Light,
    /// Fitzpatrick Type III (U+1F3FC)
    MediumLight,
    /// Fitzpatrick Type IV (U+1F3FD)
    Medium,
    /// Fitzpatrick Type V (U+1F3FE)
    MediumDark,
    /// Fitzpatrick Type VI (U+1F3FF)
    Dark,
}

impl SkinToneModifier {
    /// Returns the Unicode codepoint for this modifier.
    #[must_use]
    pub const fn codepoint(&self) -> u32 {
        match self {
            Self::Light => 0x1F3FB,
            Self::MediumLight => 0x1F3FC,
            Self::Medium => 0x1F3FD,
            Self::MediumDark => 0x1F3FE,
            Self::Dark => 0x1F3FF,
        }
    }

    /// Returns the Fitzpatrick scale level (1-5).
    #[must_use]
    pub const fn fitzpatrick_level(&self) -> u8 {
        match self {
            Self::Light => 1,
            Self::MediumLight => 2,
            Self::Medium => 3,
            Self::MediumDark => 4,
            Self::Dark => 5,
        }
    }

    /// Returns all modifier variants in order.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Light,
            Self::MediumLight,
            Self::Medium,
            Self::MediumDark,
            Self::Dark,
        ]
    }
}

impl fmt::Display for SkinToneModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ς:tone-{}", self.fitzpatrick_level())
    }
}

// ============================================================================
// 5. κ (Comparison) + Σ (Sum) → Category
// ============================================================================

/// Emoji category grouping per Unicode CLDR.
///
/// Grounds κ (Comparison): categories exist for classification,
/// comparing emoji against group membership criteria.
/// Sum (Σ) because categories are mutually exclusive.
///
/// # Domain Mappings
/// - Emoji picker tabs in chat applications
/// - Unicode CLDR annotation groups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// Faces, hands, people
    SmileysAndPeople,
    /// Animals, plants, nature
    AnimalsAndNature,
    /// Food, drink, cooking
    FoodAndDrink,
    /// Sports, games, hobbies
    Activities,
    /// Places, vehicles, buildings
    TravelAndPlaces,
    /// Tools, household, office
    Objects,
    /// Hearts, arrows, signs
    Symbols,
    /// Country, regional, special flags
    Flags,
    /// Component glyphs (modifiers, joiners)
    Component,
}

impl Category {
    /// Returns all categories in standard display order.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [
            Self::SmileysAndPeople,
            Self::AnimalsAndNature,
            Self::FoodAndDrink,
            Self::Activities,
            Self::TravelAndPlaces,
            Self::Objects,
            Self::Symbols,
            Self::Flags,
            Self::Component,
        ]
    }

    /// Returns true if this is a user-facing category (not Component).
    #[must_use]
    pub const fn is_user_facing(&self) -> bool {
        !matches!(self, Self::Component)
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SmileysAndPeople => write!(f, "κ:smileys_and_people"),
            Self::AnimalsAndNature => write!(f, "κ:animals_and_nature"),
            Self::FoodAndDrink => write!(f, "κ:food_and_drink"),
            Self::Activities => write!(f, "κ:activities"),
            Self::TravelAndPlaces => write!(f, "κ:travel_and_places"),
            Self::Objects => write!(f, "κ:objects"),
            Self::Symbols => write!(f, "κ:symbols"),
            Self::Flags => write!(f, "κ:flags"),
            Self::Component => write!(f, "κ:component"),
        }
    }
}

// ============================================================================
// 6. σ (Sequence) + × (Product) + ∃ (Existence) → ZwjSequence
// ============================================================================

/// Zero-Width Joiner sequence combining base emoji into compounds.
///
/// Grounds σ (Sequence): the ZWJ IS ordered chaining of codepoints
/// via an invisible joiner (U+200D). Product (×) because the
/// components fuse into new meaning. Existence (∃) because the
/// compound may or may not be recognized by the platform.
///
/// # Domain Mappings
/// - Family emoji: person + ZWJ + person + ZWJ + child
/// - Profession emoji: person + ZWJ + tool/symbol
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZwjSequence {
    /// Codepoints in order (without the ZWJ characters themselves)
    pub components: Vec<u32>,
}

impl ZwjSequence {
    /// Creates a new ZWJ sequence from component codepoints.
    #[must_use]
    pub fn new(components: Vec<u32>) -> Self {
        Self { components }
    }

    /// Creates a pair (two components joined by ZWJ).
    #[must_use]
    pub fn pair(a: u32, b: u32) -> Self {
        Self {
            components: vec![a, b],
        }
    }

    /// Returns the number of components.
    #[must_use]
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Returns the total codepoint length including ZWJ characters.
    #[must_use]
    pub fn total_codepoints(&self) -> usize {
        if self.components.is_empty() {
            return 0;
        }
        // components + ZWJ between each pair
        self.components.len() + self.components.len().saturating_sub(1)
    }

    /// Returns the components as Codepoint types.
    #[must_use]
    pub fn as_codepoints(&self) -> Vec<Codepoint> {
        self.components.iter().map(|&v| Codepoint::new(v)).collect()
    }

    /// Returns true if this is a single-component (degenerate) sequence.
    #[must_use]
    pub fn is_degenerate(&self) -> bool {
        self.components.len() <= 1
    }
}

impl fmt::Display for ZwjSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self
            .components
            .iter()
            .map(|cp| format!("U+{cp:04X}"))
            .collect();
        write!(f, "σ:{}", parts.join("+ZWJ+"))
    }
}

// ============================================================================
// 7. μ (Mapping) + ∃ (Existence) → Shortcode
// ============================================================================

/// Text alias mapping to an emoji codepoint (e.g., `:fire:` → U+1F525).
///
/// Grounds μ (Mapping): a shortcode IS a mapping from text to identity.
/// Existence (∃) because the target codepoint must exist.
///
/// # Domain Mappings
/// - Slack/Discord: `:thumbsup:` → U+1F44D
/// - GitHub: `:rocket:` → U+1F680
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcode {
    /// The text alias (without colons)
    pub code: String,
    /// Target emoji codepoint
    pub target_codepoint: u32,
}

impl Shortcode {
    /// Creates a new shortcode mapping.
    #[must_use]
    pub fn new(code: impl Into<String>, target: u32) -> Self {
        Self {
            code: code.into(),
            target_codepoint: target,
        }
    }

    /// Returns the colon-wrapped form (`:fire:`).
    #[must_use]
    pub fn colon_form(&self) -> String {
        format!(":{}:", self.code)
    }

    /// Returns the target as a structured Codepoint.
    #[must_use]
    pub const fn target(&self) -> Codepoint {
        Codepoint::new(self.target_codepoint)
    }
}

impl fmt::Display for Shortcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "μ::{}:→U+{:04X}", self.code, self.target_codepoint)
    }
}

// ============================================================================
// 8. μ (Mapping) + N (Quantity) + σ (Sequence) → Encoding
// ============================================================================

/// Binary representation scheme mapping codepoints to bytes.
///
/// Grounds μ (Mapping): encoding IS the mapping from abstract
/// number to concrete byte sequence (σ). Quantity (N) because
/// each scheme uses different byte widths.
///
/// # Domain Mappings
/// - UTF-8: variable width (1-4 bytes), web standard
/// - UTF-16: variable width (2 or 4 bytes), JavaScript/Java
/// - UTF-32: fixed width (4 bytes), internal processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Encoding {
    /// Variable-width, 1-4 bytes per codepoint
    Utf8,
    /// Variable-width, 2 or 4 bytes per codepoint
    Utf16,
    /// Fixed-width, 4 bytes per codepoint
    Utf32,
}

impl Encoding {
    /// Returns the minimum bytes needed for a single codepoint.
    #[must_use]
    pub const fn min_bytes(&self) -> u8 {
        match self {
            Self::Utf8 => 1,
            Self::Utf16 => 2,
            Self::Utf32 => 4,
        }
    }

    /// Returns the maximum bytes needed for a single codepoint.
    #[must_use]
    pub const fn max_bytes(&self) -> u8 {
        match self {
            Self::Utf8 => 4,
            Self::Utf16 => 4,
            Self::Utf32 => 4,
        }
    }

    /// Returns true if this encoding has variable width.
    #[must_use]
    pub const fn is_variable_width(&self) -> bool {
        matches!(self, Self::Utf8 | Self::Utf16)
    }

    /// Approximate byte count for an emoji codepoint in this encoding.
    /// Most emoji are in supplementary planes, requiring max bytes.
    #[must_use]
    pub const fn emoji_bytes(&self) -> u8 {
        match self {
            Self::Utf8 => 4,  // supplementary plane = 4 bytes
            Self::Utf16 => 4, // surrogate pair = 4 bytes
            Self::Utf32 => 4, // always 4 bytes
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8 => write!(f, "μ:UTF-8"),
            Self::Utf16 => write!(f, "μ:UTF-16"),
            Self::Utf32 => write!(f, "μ:UTF-32"),
        }
    }
}

// ============================================================================
// 9. μ (Mapping) + ∂ (Boundary) + λ (Location) → Rendering
// ============================================================================

/// Platform-specific visual rendering of an emoji.
///
/// Grounds μ (Mapping): rendering IS the final mapping from
/// codepoint to pixels. Boundary (∂) because each platform defines
/// its own visual boundary. Location (λ) because the rendering
/// varies by platform position in the ecosystem.
///
/// # Domain Mappings
/// - Apple iOS/macOS: detailed, photorealistic style
/// - Google Android: round, simplified style
/// - Samsung: historically divergent interpretations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rendering {
    /// Platform identifier
    pub platform: String,
    /// Whether this platform supports color emoji
    pub supports_color: bool,
    /// Number of emoji supported by this platform's font
    pub supported_count: u32,
}

impl Rendering {
    /// Creates a new rendering descriptor.
    #[must_use]
    pub fn new(platform: impl Into<String>, supports_color: bool, supported_count: u32) -> Self {
        Self {
            platform: platform.into(),
            supports_color,
            supported_count,
        }
    }

    /// Returns true if this is a full-color renderer.
    #[must_use]
    pub const fn is_color(&self) -> bool {
        self.supports_color
    }

    /// Returns the coverage ratio given a total emoji count.
    #[must_use]
    pub fn coverage(&self, total_emoji: u32) -> f64 {
        if total_emoji == 0 {
            return 1.0;
        }
        self.supported_count as f64 / total_emoji as f64
    }
}

impl fmt::Display for Rendering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let color = if self.supports_color { "color" } else { "mono" };
        write!(
            f,
            "μ:{}({}, {} glyphs)",
            self.platform, color, self.supported_count
        )
    }
}

// ============================================================================
// 10. → (Causality) + ν (Frequency) → Reaction
// ============================================================================

/// Social media reaction — an emoji used as a response signal.
///
/// Grounds → (Causality): a reaction IS a causal response to
/// content. Frequency (ν) because reactions accumulate as counts.
///
/// # Domain Mappings
/// - Slack: message reactions with count
/// - GitHub: issue/PR reactions
/// - Social media: like/love/laugh reactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Reaction {
    /// The emoji used as the reaction
    pub emoji_codepoint: u32,
    /// Accumulated reaction count
    pub count: u64,
}

impl Reaction {
    /// Creates a new reaction with a single instance.
    #[must_use]
    pub const fn new(emoji_codepoint: u32) -> Self {
        Self {
            emoji_codepoint,
            count: 1,
        }
    }

    /// Creates a reaction with a specific count.
    #[must_use]
    pub const fn with_count(emoji_codepoint: u32, count: u64) -> Self {
        Self {
            emoji_codepoint,
            count,
        }
    }

    /// Increments the reaction count.
    #[must_use]
    pub const fn increment(self) -> Self {
        Self {
            emoji_codepoint: self.emoji_codepoint,
            count: self.count + 1,
        }
    }

    /// Returns true if this reaction has multiple instances.
    #[must_use]
    pub const fn is_popular(&self) -> bool {
        self.count > 1
    }
}

impl fmt::Display for Reaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "→:U+{:04X} ({}x)", self.emoji_codepoint, self.count)
    }
}

// ============================================================================
// 11. μ (Mapping) + κ (Comparison) → Sentiment
// ============================================================================

/// Emotional valence mapped from an emoji's semantic content.
///
/// Grounds μ (Mapping): sentiment IS a mapping from visual to
/// emotional classification. Comparison (κ) because classification
/// requires comparing against valence criteria.
///
/// # Domain Mappings
/// - NLP: emoji sentiment analysis in text
/// - Social media: post mood classification
/// - Customer feedback: satisfaction scoring
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sentiment {
    /// Clearly positive emotional valence
    Positive,
    /// Neutral or informational
    Neutral,
    /// Clearly negative emotional valence
    Negative,
    /// Multiple valid interpretations (see Ambiguity)
    Mixed,
}

impl Sentiment {
    /// Returns a numeric valence score (-1.0 to 1.0).
    #[must_use]
    pub const fn valence(&self) -> f64 {
        match self {
            Self::Positive => 1.0,
            Self::Neutral => 0.0,
            Self::Negative => -1.0,
            Self::Mixed => 0.0,
        }
    }

    /// Returns true if this is a definitive (non-mixed) sentiment.
    #[must_use]
    pub const fn is_definitive(&self) -> bool {
        !matches!(self, Self::Mixed)
    }
}

impl fmt::Display for Sentiment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Positive => write!(f, "μ:sentiment(+)"),
            Self::Neutral => write!(f, "μ:sentiment(=)"),
            Self::Negative => write!(f, "μ:sentiment(-)"),
            Self::Mixed => write!(f, "μ:sentiment(?)"),
        }
    }
}

// ============================================================================
// 12. σ (Sequence) + ∂ (Boundary) → InputMethod
// ============================================================================

/// Mechanism by which a user selects an emoji for input.
///
/// Grounds σ (Sequence): input IS a sequence of user actions
/// leading to emoji selection. Boundary (∂) because each method
/// constrains what emoji are accessible.
///
/// # Domain Mappings
/// - Mobile: emoji keyboard panel
/// - Desktop: OS-level picker (Ctrl+. / Cmd+Ctrl+Space)
/// - Chat: shortcode typing `:fire:`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputMethod {
    /// On-screen emoji keyboard (mobile/desktop)
    Keyboard,
    /// Graphical picker with search
    Picker,
    /// Text shortcode resolution (`:fire:`)
    ShortcodeEntry,
    /// Natural language search ("fire emoji")
    SearchQuery,
}

impl InputMethod {
    /// Returns estimated keystroke count for a typical emoji selection.
    #[must_use]
    pub const fn typical_keystrokes(&self) -> u8 {
        match self {
            Self::Keyboard => 3,       // open keyboard + navigate + tap
            Self::Picker => 4,         // open picker + type search + select + confirm
            Self::ShortcodeEntry => 7, // : + name + :
            Self::SearchQuery => 10,   // type description + select
        }
    }

    /// Returns true if this method requires text typing.
    #[must_use]
    pub const fn requires_typing(&self) -> bool {
        matches!(self, Self::ShortcodeEntry | Self::SearchQuery)
    }
}

impl fmt::Display for InputMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyboard => write!(f, "σ:keyboard"),
            Self::Picker => write!(f, "σ:picker"),
            Self::ShortcodeEntry => write!(f, "σ:shortcode"),
            Self::SearchQuery => write!(f, "σ:search"),
        }
    }
}

// ============================================================================
// 13. σ (Sequence) + π (Persistence) + N (Quantity) → EmojiVersion
// ============================================================================

/// A specific release of the Unicode Emoji standard.
///
/// Grounds σ (Sequence): versions form an ordered succession.
/// Persistence (π) because once published, a version is immutable.
/// Quantity (N) for the count of new emoji introduced.
///
/// # Domain Mappings
/// - Emoji 15.1 (2023): 118 new emoji
/// - Emoji 16.0 (2024): new emoji batch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmojiVersion {
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Number of new emoji introduced in this version
    pub new_emoji_count: u32,
}

impl EmojiVersion {
    /// Creates a new emoji version descriptor.
    #[must_use]
    pub const fn new(major: u32, minor: u32, new_emoji_count: u32) -> Self {
        Self {
            major,
            minor,
            new_emoji_count,
        }
    }

    /// Returns true if this version is newer than another.
    #[must_use]
    pub const fn is_newer_than(&self, other: &Self) -> bool {
        self.major > other.major || (self.major == other.major && self.minor > other.minor)
    }

    /// Returns the version as a comparable tuple.
    #[must_use]
    pub const fn as_tuple(&self) -> (u32, u32) {
        (self.major, self.minor)
    }
}

impl fmt::Display for EmojiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "σ:Emoji {}.{} (+{} new)",
            self.major, self.minor, self.new_emoji_count
        )
    }
}

// ============================================================================
// 14. μ (Mapping) + λ (Location) + κ (Comparison) → CulturalInterpretation
// ============================================================================

/// Region-specific meaning divergence for an emoji.
///
/// Grounds λ (Location): cultural meaning IS location-dependent.
/// Mapping (μ) because the same glyph maps to different meanings.
/// Comparison (κ) because divergence is measured against a default.
///
/// # Domain Mappings
/// - Thumbs up: positive (US), offensive (Middle East)
/// - OK hand: approval (US), money (Japan), offensive (Brazil)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CulturalInterpretation {
    /// ISO 3166-1 region code
    pub region: String,
    /// Local meaning of the emoji
    pub meaning: String,
    /// Whether this diverges from the CLDR default meaning
    pub diverges_from_default: bool,
}

impl CulturalInterpretation {
    /// Creates a new cultural interpretation.
    #[must_use]
    pub fn new(region: impl Into<String>, meaning: impl Into<String>, diverges: bool) -> Self {
        Self {
            region: region.into(),
            meaning: meaning.into(),
            diverges_from_default: diverges,
        }
    }

    /// Returns true if this interpretation matches the global default.
    #[must_use]
    pub const fn is_universal(&self) -> bool {
        !self.diverges_from_default
    }
}

impl fmt::Display for CulturalInterpretation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = if self.diverges_from_default {
            " [divergent]"
        } else {
            ""
        };
        write!(f, "λ:{}={}{}", self.region, self.meaning, marker)
    }
}

// ============================================================================
// 15. ν (Frequency) + N (Quantity) → UsageFrequency
// ============================================================================

/// Popularity ranking and usage rate of an emoji.
///
/// Grounds ν (Frequency): usage IS the rate of occurrence.
/// Quantity (N) for the measured rank and rate.
///
/// # Domain Mappings
/// - Unicode Emoji Frequency data
/// - Platform-specific usage analytics
/// - Top 10 most-used emoji vary by culture
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UsageFrequency {
    /// Global popularity rank (1 = most popular)
    pub rank: u32,
    /// Uses per million messages (approximate)
    pub uses_per_million: f64,
}

impl UsageFrequency {
    /// Creates a new usage frequency record.
    #[must_use]
    pub fn new(rank: u32, uses_per_million: f64) -> Self {
        Self {
            rank,
            uses_per_million: uses_per_million.max(0.0),
        }
    }

    /// Returns true if this emoji is in the top N by rank.
    #[must_use]
    pub const fn is_top(&self, n: u32) -> bool {
        self.rank <= n
    }

    /// Returns true if this emoji is rarely used (bottom quartile heuristic).
    #[must_use]
    pub fn is_rare(&self) -> bool {
        self.uses_per_million < 1.0
    }
}

impl fmt::Display for UsageFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ν:rank-{} ({:.1}/M)", self.rank, self.uses_per_million)
    }
}

// ============================================================================
// 16. ∅ (Void) + μ (Mapping) → FallbackDisplay
// ============================================================================

/// What appears when an emoji codepoint has no glyph on the device.
///
/// Grounds ∅ (Void): fallback IS the handling of absence.
/// Mapping (μ) because even absence must map to something visible.
///
/// # Domain Mappings
/// - Tofu (empty box): most common fallback
/// - Component sequence: ZWJ fallback shows individual parts
/// - Replacement char: U+FFFD for invalid sequences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FallbackDisplay {
    /// Empty box glyph (tofu / .notdef)
    Tofu,
    /// Unicode replacement character (U+FFFD)
    ReplacementCharacter,
    /// ZWJ decomposition showing individual components
    ComponentSequence,
    /// Text description in place of glyph
    TextFallback,
}

impl FallbackDisplay {
    /// Returns true if this fallback preserves some semantic content.
    #[must_use]
    pub const fn preserves_meaning(&self) -> bool {
        matches!(self, Self::ComponentSequence | Self::TextFallback)
    }

    /// Returns true if this is a total information loss.
    #[must_use]
    pub const fn is_opaque(&self) -> bool {
        matches!(self, Self::Tofu | Self::ReplacementCharacter)
    }
}

impl fmt::Display for FallbackDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tofu => write!(f, "∅:tofu"),
            Self::ReplacementCharacter => write!(f, "∅:U+FFFD"),
            Self::ComponentSequence => write!(f, "∅:components"),
            Self::TextFallback => write!(f, "∅:text"),
        }
    }
}

// ============================================================================
// 17. κ (Comparison) + ∅ (Void) + μ (Mapping) → Ambiguity
// ============================================================================

/// Degree to which an emoji's meaning is contested or unclear.
///
/// Grounds ∅ (Void): ambiguity IS a void of consensus.
/// Comparison (κ) because measuring ambiguity requires comparing
/// interpretations. Mapping (μ) because each interpretation is
/// a different meaning-mapping of the same glyph.
///
/// # Domain Mappings
/// - Slightly smiling face: sincere or passive-aggressive?
/// - Skull emoji: death or "I'm dead (laughing)"?
/// - Peach emoji: fruit or anatomical reference?
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ambiguity {
    /// Number of distinct documented interpretations
    pub interpretation_count: u32,
    /// Agreement score (0.0 = total disagreement, 1.0 = consensus)
    pub consensus_score: f64,
}

impl Ambiguity {
    /// Creates a new ambiguity measure.
    #[must_use]
    pub fn new(interpretation_count: u32, consensus_score: f64) -> Self {
        Self {
            interpretation_count,
            consensus_score: consensus_score.clamp(0.0, 1.0),
        }
    }

    /// Returns true if this emoji is genuinely ambiguous (low consensus).
    #[must_use]
    pub fn is_ambiguous(&self) -> bool {
        self.consensus_score < 0.5 && self.interpretation_count > 1
    }

    /// Returns true if meaning is clear (high consensus).
    #[must_use]
    pub fn is_clear(&self) -> bool {
        self.consensus_score >= 0.8
    }
}

impl fmt::Display for Ambiguity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "∅:ambiguity({} interps, {:.0}% consensus)",
            self.interpretation_count,
            self.consensus_score * 100.0
        )
    }
}

// ============================================================================
// 18. × (Product) + σ (Sequence) + ρ (Recursion) → EmojiComposition
// ============================================================================

/// Compound emoji built from base + modifiers via recursive composition.
///
/// Grounds × (Product): composition IS the product of parts.
/// Sequence (σ) for the ordered application of modifiers.
/// Recursion (ρ) because compositions can nest (ZWJ of ZWJ results).
///
/// # Domain Mappings
/// - Person + skin tone + ZWJ + tool = profession emoji
/// - Flag: regional indicator + regional indicator
/// - Keycap: digit + VS16 + combining enclosing keycap
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmojiComposition {
    /// Base emoji codepoint
    pub base: u32,
    /// Applied modifier codepoints in order
    pub modifiers: Vec<u32>,
    /// Nesting depth (0 = simple, 1+ = compound)
    pub depth: u32,
}

impl EmojiComposition {
    /// Creates a simple (unmodified) emoji.
    #[must_use]
    pub const fn simple(base: u32) -> Self {
        Self {
            base,
            modifiers: Vec::new(),
            depth: 0,
        }
    }

    /// Creates a composed emoji with modifiers.
    #[must_use]
    pub fn composed(base: u32, modifiers: Vec<u32>) -> Self {
        let depth = if modifiers.is_empty() { 0 } else { 1 };
        Self {
            base,
            modifiers,
            depth,
        }
    }

    /// Adds a modifier to the composition, increasing depth.
    #[must_use]
    pub fn with_modifier(mut self, modifier: u32) -> Self {
        self.modifiers.push(modifier);
        self.depth = self.depth.saturating_add(1);
        self
    }

    /// Returns the total component count (base + modifiers).
    #[must_use]
    pub fn component_count(&self) -> usize {
        1 + self.modifiers.len()
    }

    /// Returns true if this is a simple (unmodified) emoji.
    #[must_use]
    pub fn is_simple(&self) -> bool {
        self.modifiers.is_empty()
    }
}

impl fmt::Display for EmojiComposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_simple() {
            write!(f, "×:U+{:04X}", self.base)
        } else {
            let mods: Vec<String> = self
                .modifiers
                .iter()
                .map(|m| format!("U+{m:04X}"))
                .collect();
            write!(
                f,
                "×:U+{:04X}+[{}] (depth {})",
                self.base,
                mods.join(","),
                self.depth
            )
        }
    }
}

// ============================================================================
// 19. σ (Sequence) + ∂ (Boundary) + → (Causality) → ProposalProcess
// ============================================================================

/// Unicode emoji proposal lifecycle — the governance pipeline.
///
/// Grounds ∂ (Boundary): the process IS a gate-keeping boundary.
/// Sequence (σ) for the ordered phases. Causality (→) because
/// each phase's outcome causes the next transition.
///
/// # Domain Mappings
/// - Unicode Technical Committee (UTC) review process
/// - Emoji Subcommittee proposal evaluation
/// - Public review period → ballot → inclusion/rejection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProposalProcess {
    /// Initial proposal document submitted
    Submission,
    /// Under review by Emoji Subcommittee
    Review,
    /// Public review / comment period
    PublicReview,
    /// Approved for inclusion in next version
    Approved,
    /// Rejected (may be resubmitted)
    Rejected,
    /// Included in a published Unicode version
    Included,
}

impl ProposalProcess {
    /// Returns true if the proposal is still in progress.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self, Self::Submission | Self::Review | Self::PublicReview)
    }

    /// Returns true if the proposal reached a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Approved | Self::Rejected | Self::Included)
    }

    /// Returns the phase index in the lifecycle (0-based).
    #[must_use]
    pub const fn phase_index(&self) -> u8 {
        match self {
            Self::Submission => 0,
            Self::Review => 1,
            Self::PublicReview => 2,
            Self::Approved => 3,
            Self::Rejected => 3, // same level, different branch
            Self::Included => 4,
        }
    }
}

impl fmt::Display for ProposalProcess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Submission => write!(f, "∂:submitted"),
            Self::Review => write!(f, "∂:under_review"),
            Self::PublicReview => write!(f, "∂:public_review"),
            Self::Approved => write!(f, "∂:approved"),
            Self::Rejected => write!(f, "∂:rejected"),
            Self::Included => write!(f, "∂:included"),
        }
    }
}

// ============================================================================
// 20. ∝ (Irreversibility) + σ (Sequence) + Σ (Sum) → EmojiEvolution
// ============================================================================

/// The irreversible growth of the emoji standard over time.
///
/// Grounds ∝ (Irreversibility): once added, an emoji cannot be
/// removed from Unicode. Sequence (σ) for the version timeline.
/// Sum (Σ) for the total accumulated set across versions.
///
/// # Domain Mappings
/// - Unicode stability policy: never remove assigned codepoints
/// - Emoji additions are permanent (even regrettable ones)
/// - Total emoji count only grows monotonically
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmojiEvolution {
    /// Emoji version when first added (e.g., "15.1")
    pub version_added: String,
    /// Whether this emoji has been superseded (but never removed)
    pub superseded: bool,
    /// Running total of emoji at this point in history
    pub cumulative_count: u32,
}

impl EmojiEvolution {
    /// Creates a new evolution record.
    #[must_use]
    pub fn new(version_added: impl Into<String>, superseded: bool, cumulative_count: u32) -> Self {
        Self {
            version_added: version_added.into(),
            superseded,
            cumulative_count,
        }
    }

    /// Returns true if this emoji is still the canonical form.
    #[must_use]
    pub const fn is_canonical(&self) -> bool {
        !self.superseded
    }

    /// Returns true if this emoji has been superseded by a newer form.
    #[must_use]
    pub const fn is_superseded(&self) -> bool {
        self.superseded
    }
}

impl fmt::Display for EmojiEvolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.superseded { " [superseded]" } else { "" };
        write!(
            f,
            "∝:v{}(#{}){}",
            self.version_added, self.cumulative_count, status
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- Codepoint (∃) --

    #[test]
    fn test_codepoint_emoji_range() {
        let grinning = Codepoint::new(0x1F600);
        assert!(grinning.is_emoji_range());
        assert!(grinning.is_valid());
        assert_eq!(grinning.to_unicode_notation(), "U+1F600");
    }

    #[test]
    fn test_codepoint_non_emoji() {
        let letter_a = Codepoint::new(0x0041);
        assert!(!letter_a.is_emoji_range());
        assert!(letter_a.is_valid());
    }

    #[test]
    fn test_codepoint_invalid_surrogate() {
        let surrogate = Codepoint::new(0xD800);
        assert!(!surrogate.is_valid());
    }

    #[test]
    fn test_codepoint_display() {
        let cp = Codepoint::new(0x2764);
        assert_eq!(format!("{cp}"), "∃:U+2764");
    }

    // -- Glyph (μ) --

    #[test]
    fn test_glyph_creation() {
        let g = Glyph::new(0x1F600, "grinning face");
        assert!(g.is_named());
        assert_eq!(g.codepoint().value, 0x1F600);
    }

    #[test]
    fn test_glyph_display() {
        let g = Glyph::new(0x1F525, "fire");
        assert_eq!(format!("{g}"), "μ:U+1F525→fire");
    }

    // -- UnicodeStandard (∂) --

    #[test]
    fn test_unicode_standard_contains() {
        let std15 = UnicodeStandard::new(15, 1, 0x10FFFF);
        let cp = Codepoint::new(0x1F600);
        assert!(std15.contains(&cp));
    }

    #[test]
    fn test_unicode_standard_newer() {
        let v15 = UnicodeStandard::new(15, 0, 0x10FFFF);
        let v16 = UnicodeStandard::new(16, 0, 0x10FFFF);
        assert!(v16.is_newer_than(&v15));
        assert!(!v15.is_newer_than(&v16));
    }

    // -- SkinToneModifier (ς) --

    #[test]
    fn test_skin_tone_codepoints() {
        assert_eq!(SkinToneModifier::Light.codepoint(), 0x1F3FB);
        assert_eq!(SkinToneModifier::Dark.codepoint(), 0x1F3FF);
    }

    #[test]
    fn test_skin_tone_fitzpatrick() {
        for (i, tone) in SkinToneModifier::all().iter().enumerate() {
            assert_eq!(tone.fitzpatrick_level(), (i + 1) as u8);
        }
    }

    #[test]
    fn test_skin_tone_display() {
        assert_eq!(format!("{}", SkinToneModifier::Medium), "ς:tone-3");
    }

    // -- Category (κ) --

    #[test]
    fn test_category_count() {
        assert_eq!(Category::all().len(), 9);
    }

    #[test]
    fn test_category_user_facing() {
        assert!(Category::SmileysAndPeople.is_user_facing());
        assert!(!Category::Component.is_user_facing());
    }

    // -- ZwjSequence (σ) --

    #[test]
    fn test_zwj_sequence_pair() {
        let seq = ZwjSequence::pair(0x1F469, 0x1F680); // woman + rocket
        assert_eq!(seq.component_count(), 2);
        assert_eq!(seq.total_codepoints(), 3); // 2 components + 1 ZWJ
        assert!(!seq.is_degenerate());
    }

    #[test]
    fn test_zwj_sequence_display() {
        let seq = ZwjSequence::pair(0x1F469, 0x1F680);
        assert_eq!(format!("{seq}"), "σ:U+1F469+ZWJ+U+1F680");
    }

    // -- Shortcode (μ) --

    #[test]
    fn test_shortcode_colon_form() {
        let sc = Shortcode::new("fire", 0x1F525);
        assert_eq!(sc.colon_form(), ":fire:");
        assert_eq!(sc.target().value, 0x1F525);
    }

    // -- Encoding (μ) --

    #[test]
    fn test_encoding_variable_width() {
        assert!(Encoding::Utf8.is_variable_width());
        assert!(Encoding::Utf16.is_variable_width());
        assert!(!Encoding::Utf32.is_variable_width());
    }

    #[test]
    fn test_encoding_emoji_bytes() {
        // All encodings need 4 bytes for supplementary plane emoji
        assert_eq!(Encoding::Utf8.emoji_bytes(), 4);
        assert_eq!(Encoding::Utf16.emoji_bytes(), 4);
        assert_eq!(Encoding::Utf32.emoji_bytes(), 4);
    }

    // -- Rendering (μ) --

    #[test]
    fn test_rendering_coverage() {
        let r = Rendering::new("Apple", true, 3500);
        assert!(r.is_color());
        let cov = r.coverage(3782);
        assert!(cov > 0.9 && cov < 1.0);
    }

    // -- Reaction (→) --

    #[test]
    fn test_reaction_increment() {
        let r = Reaction::new(0x1F44D);
        assert_eq!(r.count, 1);
        assert!(!r.is_popular());
        let r = r.increment();
        assert_eq!(r.count, 2);
        assert!(r.is_popular());
    }

    // -- Sentiment (μ) --

    #[test]
    fn test_sentiment_valence() {
        assert!((Sentiment::Positive.valence() - 1.0).abs() < f64::EPSILON);
        assert!((Sentiment::Negative.valence() - (-1.0)).abs() < f64::EPSILON);
        assert!(Sentiment::Positive.is_definitive());
        assert!(!Sentiment::Mixed.is_definitive());
    }

    // -- InputMethod (σ) --

    #[test]
    fn test_input_method_typing() {
        assert!(!InputMethod::Keyboard.requires_typing());
        assert!(InputMethod::ShortcodeEntry.requires_typing());
        assert!(InputMethod::SearchQuery.requires_typing());
    }

    // -- EmojiVersion (σ) --

    #[test]
    fn test_emoji_version_ordering() {
        let v15 = EmojiVersion::new(15, 1, 118);
        let v16 = EmojiVersion::new(16, 0, 50);
        assert!(v16.is_newer_than(&v15));
        assert!(!v15.is_newer_than(&v16));
    }

    // -- CulturalInterpretation (λ) --

    #[test]
    fn test_cultural_interpretation_divergence() {
        let us = CulturalInterpretation::new("US", "approval", false);
        assert!(us.is_universal());
        let br = CulturalInterpretation::new("BR", "offensive", true);
        assert!(!br.is_universal());
    }

    // -- UsageFrequency (ν) --

    #[test]
    fn test_usage_frequency_ranking() {
        let top = UsageFrequency::new(1, 5000.0);
        assert!(top.is_top(10));
        assert!(!top.is_rare());
        let rare = UsageFrequency::new(3000, 0.1);
        assert!(rare.is_rare());
    }

    // -- FallbackDisplay (∅) --

    #[test]
    fn test_fallback_meaning_preservation() {
        assert!(FallbackDisplay::ComponentSequence.preserves_meaning());
        assert!(FallbackDisplay::TextFallback.preserves_meaning());
        assert!(!FallbackDisplay::Tofu.preserves_meaning());
        assert!(FallbackDisplay::Tofu.is_opaque());
    }

    // -- Ambiguity (∅) --

    #[test]
    fn test_ambiguity_classification() {
        let clear = Ambiguity::new(1, 0.95);
        assert!(clear.is_clear());
        assert!(!clear.is_ambiguous());

        let ambig = Ambiguity::new(4, 0.3);
        assert!(ambig.is_ambiguous());
        assert!(!ambig.is_clear());
    }

    // -- EmojiComposition (×) --

    #[test]
    fn test_emoji_composition_simple() {
        let simple = EmojiComposition::simple(0x1F600);
        assert!(simple.is_simple());
        assert_eq!(simple.component_count(), 1);
    }

    #[test]
    fn test_emoji_composition_with_modifier() {
        let composed = EmojiComposition::simple(0x1F44B).with_modifier(0x1F3FD); // wave + medium tone
        assert!(!composed.is_simple());
        assert_eq!(composed.component_count(), 2);
        assert_eq!(composed.depth, 1);
    }

    // -- ProposalProcess (∂) --

    #[test]
    fn test_proposal_lifecycle() {
        assert!(ProposalProcess::Submission.is_pending());
        assert!(ProposalProcess::Review.is_pending());
        assert!(!ProposalProcess::Approved.is_pending());
        assert!(ProposalProcess::Approved.is_terminal());
        assert!(ProposalProcess::Rejected.is_terminal());
    }

    #[test]
    fn test_proposal_phase_order() {
        assert!(ProposalProcess::Submission.phase_index() < ProposalProcess::Review.phase_index());
        assert!(
            ProposalProcess::Review.phase_index() < ProposalProcess::PublicReview.phase_index()
        );
    }

    // -- EmojiEvolution (∝) --

    #[test]
    fn test_emoji_evolution() {
        let grinning = EmojiEvolution::new("1.0", false, 722);
        assert!(grinning.is_canonical());
        assert!(!grinning.is_superseded());

        let old = EmojiEvolution::new("1.0", true, 722);
        assert!(!old.is_canonical());
        assert!(old.is_superseded());
    }

    // -- Cross-type integration --

    #[test]
    fn test_shortcode_to_codepoint_to_glyph() {
        // Full mapping chain: shortcode → codepoint → glyph
        let sc = Shortcode::new("fire", 0x1F525);
        let cp = sc.target();
        let glyph = Glyph::new(cp.value, "fire");

        assert!(cp.is_emoji_range());
        assert!(glyph.is_named());
        assert_eq!(cp.value, glyph.codepoint);
    }

    #[test]
    fn test_composition_with_skin_tone() {
        // Compose: wave + medium skin tone
        let tone = SkinToneModifier::Medium;
        let composed = EmojiComposition::simple(0x1F44B).with_modifier(tone.codepoint());

        assert_eq!(composed.component_count(), 2);
        assert_eq!(composed.modifiers[0], 0x1F3FD);
    }
}
