//! # Morphological Feature Bitfield
//!
//! Types for the 64-bit feature bitfield encoding defined in KB-0007.
//! Features are packed into a compact representation for use in the GVM
//! bytecode format (RFC-0002).
//!
//! ## Bitfield Layout (KB-0007 §10)
//!
//! | Bits   | Feature       | Width | Values                                                                 |
//! |--------|---------------|-------|------------------------------------------------------------------------|
//! | 0–3    | pos           | 4     | verb, noun, particle, pronoun, adjective, adverb, prep, conj, ...      |
//! | 4–5    | gender        | 2     | masculine, feminine, common, unspecified                                |
//! | 6–7    | number        | 2     | singular, dual, plural, unspecified                                     |
//! | 8–9    | person        | 2     | first, second, third, unspecified                                       |
//! | 10–11  | tense         | 2     | past, present, imperative, unspecified                                  |
//! | 12–14  | mood          | 3     | indicative, subjunctive, jussive, energetic, unspecified                |
//! | 15     | voice         | 1     | active, passive                                                         |
//! | 16     | case          | 2     | nominative, accusative, genitive, unspecified                           |
//! | 18     | state         | 1     | definite, indefinite                                                     |
//! | 19–23  | verb_form     | 5     | I–XV, not_a_verb                                                        |
//! | 24–27  | noun_type     | 4     | masdar, ism_fail, ism_maful, ism_makan, ..., not_a_noun                 |
//! | 28–31  | pronoun_type  | 4     | personal_attached, personal_detached, demonstrative, ..., not_a_pronoun |
//! | 32–34  | transitivity  | 3     | intransitive, transitive_1, transitive_2, ditransitive, unspecified      |
//! | 35–38  | root_type     | 4     | sound, weak_initial, weak_middle, ..., unspecified                       |
//! | 39–40  | stress_pattern| 2     | final, penultimate, antepenultimate, unspecified                         |
//! | 41–44  | syllable_count| 4     | 0–8 (0 = unspecified)                                                   |
//! | 45     | has_shadda    | 1     | true, false                                                              |
//! | 46     | has_madd      | 1     | true, false                                                              |
//! | 47     | has_hamza     | 1     | true, false                                                              |
//! | 48–63  | reserved      | 16    | Plugin extensions                                                        |
//!
//! ## Spec Alignment
//!
//! - KB-0007: Morphological Features Taxonomy (complete feature definitions)
//! - SPEC-0102: Morphological Features — Encoding, Validation & Resolution
//! - RFC-0002: Grammar Bytecode Format (§Feature Bitfield)

use serde::{Deserialize, Serialize};

/// The 64-bit feature bitfield for morphological features (KB-0007 §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeatureBitfield(u64);

impl FeatureBitfield {
    /// Create a new zeroed (all-unspecified) feature bitfield.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create from a raw u64 value.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Get the raw u64 value.
    pub const fn to_raw(&self) -> u64 {
        self.0
    }

    // ──────────────────────────────────────────
    //  Bitfield Positions (from KB-0007 §10)
    // ──────────────────────────────────────────

    const POS_BITS: (usize, usize) = (0, 4);
    const GENDER_BITS: (usize, usize) = (4, 2);
    const NUMBER_BITS: (usize, usize) = (6, 2);
    const PERSON_BITS: (usize, usize) = (8, 2);
    const TENSE_BITS: (usize, usize) = (10, 2);
    const MOOD_BITS: (usize, usize) = (12, 3);
    const VOICE_BITS: (usize, usize) = (15, 1);
    const CASE_BITS: (usize, usize) = (16, 2);
    const STATE_BITS: (usize, usize) = (18, 1);
    const VERB_FORM_BITS: (usize, usize) = (19, 5);
    const NOUN_TYPE_BITS: (usize, usize) = (24, 4);
    const PRONOUN_TYPE_BITS: (usize, usize) = (28, 4);
    const TRANSITIVITY_BITS: (usize, usize) = (32, 3);
    const ROOT_TYPE_BITS: (usize, usize) = (35, 4);
    const STRESS_PATTERN_BITS: (usize, usize) = (39, 2);
    const SYLLABLE_COUNT_BITS: (usize, usize) = (41, 4);
    const HAS_SHADDA_BITS: (usize, usize) = (45, 1);
    const HAS_MADD_BITS: (usize, usize) = (46, 1);
    const HAS_HAMZA_BITS: (usize, usize) = (47, 1);

    // ──────────────────────────────────────────
    //  Field Getters
    // ──────────────────────────────────────────

    pub fn pos(&self) -> u8 {
        self.get_bits(Self::POS_BITS)
    }
    pub fn gender(&self) -> u8 {
        self.get_bits(Self::GENDER_BITS)
    }
    pub fn number(&self) -> u8 {
        self.get_bits(Self::NUMBER_BITS)
    }
    pub fn person(&self) -> u8 {
        self.get_bits(Self::PERSON_BITS)
    }
    pub fn tense(&self) -> u8 {
        self.get_bits(Self::TENSE_BITS)
    }
    pub fn mood(&self) -> u8 {
        self.get_bits(Self::MOOD_BITS)
    }
    pub fn voice(&self) -> u8 {
        self.get_bits(Self::VOICE_BITS)
    }
    pub fn case(&self) -> u8 {
        self.get_bits(Self::CASE_BITS)
    }
    pub fn state(&self) -> u8 {
        self.get_bits(Self::STATE_BITS)
    }
    pub fn verb_form(&self) -> u8 {
        self.get_bits(Self::VERB_FORM_BITS)
    }
    pub fn noun_type(&self) -> u8 {
        self.get_bits(Self::NOUN_TYPE_BITS)
    }
    pub fn pronoun_type(&self) -> u8 {
        self.get_bits(Self::PRONOUN_TYPE_BITS)
    }
    pub fn transitivity(&self) -> u8 {
        self.get_bits(Self::TRANSITIVITY_BITS)
    }
    pub fn root_type(&self) -> u8 {
        self.get_bits(Self::ROOT_TYPE_BITS)
    }
    pub fn stress_pattern(&self) -> u8 {
        self.get_bits(Self::STRESS_PATTERN_BITS)
    }
    pub fn syllable_count(&self) -> u8 {
        self.get_bits(Self::SYLLABLE_COUNT_BITS)
    }
    pub fn has_shadda(&self) -> bool {
        self.get_bits(Self::HAS_SHADDA_BITS) != 0
    }
    pub fn has_madd(&self) -> bool {
        self.get_bits(Self::HAS_MADD_BITS) != 0
    }
    pub fn has_hamza(&self) -> bool {
        self.get_bits(Self::HAS_HAMZA_BITS) != 0
    }

    // ──────────────────────────────────────────
    //  Field Setters
    // ──────────────────────────────────────────

    pub fn with_pos(mut self, value: u8) -> Self {
        self.set_bits(Self::POS_BITS, value);
        self
    }
    pub fn with_gender(mut self, value: u8) -> Self {
        self.set_bits(Self::GENDER_BITS, value);
        self
    }
    pub fn with_number(mut self, value: u8) -> Self {
        self.set_bits(Self::NUMBER_BITS, value);
        self
    }
    pub fn with_person(mut self, value: u8) -> Self {
        self.set_bits(Self::PERSON_BITS, value);
        self
    }
    pub fn with_tense(mut self, value: u8) -> Self {
        self.set_bits(Self::TENSE_BITS, value);
        self
    }
    pub fn with_mood(mut self, value: u8) -> Self {
        self.set_bits(Self::MOOD_BITS, value);
        self
    }
    pub fn with_voice(mut self, value: u8) -> Self {
        self.set_bits(Self::VOICE_BITS, value);
        self
    }
    pub fn with_case(mut self, value: u8) -> Self {
        self.set_bits(Self::CASE_BITS, value);
        self
    }
    pub fn with_state(mut self, value: u8) -> Self {
        self.set_bits(Self::STATE_BITS, value);
        self
    }
    pub fn with_verb_form(mut self, value: u8) -> Self {
        self.set_bits(Self::VERB_FORM_BITS, value);
        self
    }
    pub fn with_noun_type(mut self, value: u8) -> Self {
        self.set_bits(Self::NOUN_TYPE_BITS, value);
        self
    }
    pub fn with_pronoun_type(mut self, value: u8) -> Self {
        self.set_bits(Self::PRONOUN_TYPE_BITS, value);
        self
    }
    pub fn with_transitivity(mut self, value: u8) -> Self {
        self.set_bits(Self::TRANSITIVITY_BITS, value);
        self
    }
    pub fn with_root_type(mut self, value: u8) -> Self {
        self.set_bits(Self::ROOT_TYPE_BITS, value);
        self
    }
    pub fn with_stress_pattern(mut self, value: u8) -> Self {
        self.set_bits(Self::STRESS_PATTERN_BITS, value);
        self
    }
    pub fn with_syllable_count(mut self, value: u8) -> Self {
        self.set_bits(Self::SYLLABLE_COUNT_BITS, value);
        self
    }
    pub fn with_has_shadda(mut self, value: bool) -> Self {
        self.set_bit(Self::HAS_SHADDA_BITS, value);
        self
    }
    pub fn with_has_madd(mut self, value: bool) -> Self {
        self.set_bit(Self::HAS_MADD_BITS, value);
        self
    }
    pub fn with_has_hamza(mut self, value: bool) -> Self {
        self.set_bit(Self::HAS_HAMZA_BITS, value);
        self
    }

    // ──────────────────────────────────────────
    //  Bit Manipulation Primitives
    // ──────────────────────────────────────────

    /// Extract a bit field at (start_bit, width).
    fn get_bits(&self, (start, width): (usize, usize)) -> u8 {
        let mask = (1u64 << width) - 1;
        ((self.0 >> start) & mask) as u8
    }

    /// Set a bit field at (start_bit, width).
    fn set_bits(&mut self, (start, width): (usize, usize), value: u8) {
        let mask = (1u64 << width) - 1;
        self.0 = (self.0 & !(mask << start)) | ((value as u64 & mask) << start);
    }

    /// Set a single bit at position `bit`.
    fn set_bit(&mut self, (start, _): (usize, usize), value: bool) {
        if value {
            self.0 |= 1u64 << start;
        } else {
            self.0 &= !(1u64 << start);
        }
    }
}

impl Default for FeatureBitfield {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for FeatureBitfield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FeatureBitfield(0x{:016x})", self.0)
    }
}

/// A single morphological feature with its value (SPEC-0001-C5 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedFeature {
    /// Feature name (e.g., "gender", "number", "tense")
    pub name: String,

    /// Feature value (e.g., "masculine", "plural", "past")
    pub value: String,

    /// Feature category
    pub category: FeatureCategory,

    /// Confidence in this feature assignment (0.0 to 1.0)
    pub confidence: f64,

    /// Source of this feature (KB entry ID or rule ID)
    pub source: String,
}

/// Feature category classification (KB-0007 §1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureCategory {
    /// Part of Speech
    Pos,
    /// Inflectional features (gender, number, person, tense, etc.)
    Inflectional,
    /// Derivational features (verb_form, noun_type, etc.)
    Derivational,
    /// Prosodic features (stress_pattern, syllable_count)
    Prosodic,
    /// Orthographic features (has_shadda, has_madd, has_hamza)
    Orthographic,
}

impl std::fmt::Display for FeatureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureCategory::Pos => write!(f, "pos"),
            FeatureCategory::Inflectional => write!(f, "inflectional"),
            FeatureCategory::Derivational => write!(f, "derivational"),
            FeatureCategory::Prosodic => write!(f, "prosodic"),
            FeatureCategory::Orthographic => write!(f, "orthographic"),
        }
    }
}

/// Full feature definition entry from KB-0007.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    /// Unique feature identifier (e.g., "KB-0007:gender")
    pub id: String,

    /// Canonical feature name (e.g., "gender")
    pub feature_name: String,

    /// Feature category
    pub category: FeatureCategory,

    /// Starting bit position in the 64-bit field
    pub bitfield_position: u8,

    /// Number of bits used
    pub bitfield_width: u8,

    /// All valid values for this feature
    pub values: Vec<FeatureValue>,

    /// Default value when feature is unspecified
    pub default_value: Option<String>,

    /// Description of what this feature encodes
    pub description: String,

    /// Which POS types this feature applies to
    pub applies_to_pos: Vec<String>,
}

/// A single valid value for a morphological feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureValue {
    /// Value name (e.g., "masculine")
    pub name: String,

    /// Numeric code in the bitfield
    pub code: u8,

    /// Human-readable label
    pub label: String,

    /// Optional description of this value
    pub description: Option<String>,

    /// Arabic grammatical term (if applicable)
    pub arabic_term: Option<String>,
}

/// Feature agreement rule (KB-0007 §11).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgreementRule {
    pub id: String,
    pub rule_type: AgreementRuleType,
    pub description: String,
    pub source_feature: String,
    pub target_feature: String,
    pub constraint: String,
    pub exceptions: Option<String>,
    pub applies_to: Vec<String>,
}

/// Type of agreement rule (KB-0007 §11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementRuleType {
    SubjectVerb,
    NounAdjective,
    Government,
    MoodGovernment,
}

/// Feature inference rule (KB-0007 §12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRule {
    pub id: String,
    pub input_feature: String,
    pub input_value: String,
    pub inferred_feature: String,
    pub inferred_value: String,
    pub priority: u32,
    pub condition: Option<String>,
    pub notes: Option<String>,
}

/// Feature constraint (KB-0007 §14).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConstraint {
    pub id: String,
    pub constraint_type: ConstraintType,
    pub description: String,
    pub features: Vec<String>,
    pub condition: Option<String>,
    pub allowed_combinations: Option<Vec<Vec<String>>>,
}

/// Type of feature constraint (KB-0007 §14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    ValidCombination,
    MutualExclusion,
    Dependency,
    ConditionalRequired,
}
