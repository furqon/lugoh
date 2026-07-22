//! # Common Types
//!
//! Shared enumeration and type definitions used across all AGOS pipeline modules.
//! These types form the shared vocabulary of the platform.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C4: Module Responsibilities & Interfaces
//! - SPEC-0001-C5: Data Flow & Intermediate Representations
//! - KB-0007: Morphological Features Taxonomy
//! - KB-0005: Particles
//! - KB-0006: Pronouns

use crate::version::KnowledgeVersionMap;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
//  Pipeline Configuration
// ──────────────────────────────────────────────

/// Log level for pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Base configuration for every pipeline stage (SPEC-0001-C4 §2.5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageConfig {
    /// Stage identifier (e.g., "MOD-01")
    pub stage_id: String,

    /// Versions of knowledge bases used by this stage
    pub knowledge_versions: KnowledgeVersionMap,

    /// Logging level for this stage
    pub log_level: LogLevel,
}

/// The grammar school configuration (SPEC-0001-C2 §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrammarSchool {
    Basra,
    Kufa,
    Baghdad,
    Andalus,
    Modern,
}

impl GrammarSchool {
    pub fn as_str(&self) -> &'static str {
        match self {
            GrammarSchool::Basra => "basra",
            GrammarSchool::Kufa => "kufa",
            GrammarSchool::Baghdad => "baghdad",
            GrammarSchool::Andalus => "andalus",
            GrammarSchool::Modern => "modern",
        }
    }
}

/// Pipeline execution mode (SPEC-0001-C2 §3.2, §8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineMode {
    Full,
    MorphologyOnly,
    TokenizationOnly,
    SyntaxOnly,
}

/// Explanation output format (SPEC-0001-C4 §13.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplanationFormat {
    Text,
    Html,
    Json,
    Pdf,
}

/// Supported explanation languages (SPEC-0001-C4 §13.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExplanationLanguage {
    Arabic,
    English,
    Urdu,
    Malay,
    Indonesian,
    French,
    Turkish,
    Other(String),
}

impl ExplanationLanguage {
    pub fn code(&self) -> &str {
        match self {
            ExplanationLanguage::Arabic => "ar",
            ExplanationLanguage::English => "en",
            ExplanationLanguage::Urdu => "ur",
            ExplanationLanguage::Malay => "ms",
            ExplanationLanguage::Indonesian => "id",
            ExplanationLanguage::French => "fr",
            ExplanationLanguage::Turkish => "tr",
            ExplanationLanguage::Other(c) => c.as_str(),
        }
    }
}

// ──────────────────────────────────────────────
//  Token Types
// ──────────────────────────────────────────────

/// Raw token type classification (SPEC-0001-C4 §4.3, SPEC-0001-C5 §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    Word,
    Punctuation,
    Number,
    Whitespace,
    Symbol,
    Unknown,
}

/// Morpheme type within a segmented token (SPEC-0001-C4 §5.3, SPEC-0001-C5 §4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorphemeType {
    Prefix,
    Stem,
    Suffix,
    Clitic,
    Particle,
}

// ──────────────────────────────────────────────
//  Part of Speech
// ──────────────────────────────────────────────

/// Part of Speech classification (KB-0007 §5).
///
/// Each POS determines which morphological features apply to a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Verb,
    Noun,
    Particle,
    Pronoun,
    Adjective,
    Adverb,
    Preposition,
    Conjunction,
    ProperNoun,
    Interrogative,
    Unknown,
}

impl PartOfSpeech {
    /// Get the numeric code used in the 64-bit feature bitfield (bits 0-3).
    pub fn code(&self) -> u8 {
        match self {
            PartOfSpeech::Verb => 0,
            PartOfSpeech::Noun => 1,
            PartOfSpeech::Particle => 2,
            PartOfSpeech::Pronoun => 3,
            PartOfSpeech::Adjective => 4,
            PartOfSpeech::Adverb => 5,
            PartOfSpeech::Preposition => 6,
            PartOfSpeech::Conjunction => 7,
            PartOfSpeech::ProperNoun => 8,
            PartOfSpeech::Interrogative => 9,
            PartOfSpeech::Unknown => 15,
        }
    }

    /// Create from numeric code.
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => PartOfSpeech::Verb,
            1 => PartOfSpeech::Noun,
            2 => PartOfSpeech::Particle,
            3 => PartOfSpeech::Pronoun,
            4 => PartOfSpeech::Adjective,
            5 => PartOfSpeech::Adverb,
            6 => PartOfSpeech::Preposition,
            7 => PartOfSpeech::Conjunction,
            8 => PartOfSpeech::ProperNoun,
            9 => PartOfSpeech::Interrogative,
            _ => PartOfSpeech::Unknown,
        }
    }
}

// ──────────────────────────────────────────────
//  Syntactic Roles (I'rab)
// ──────────────────────────────────────────────

/// Syntactic role in Arabic grammar (SPEC-0001-C4 §7.3, SPEC-0001-C5 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntacticRole {
    // Nominal sentence (jumlah ismiyyah)
    Mubtada,
    Khabar,
    // Verbal sentence (jumlah fi'liyyah)
    FiL,
    Fail,
    // Objects
    MafulBiHi,
    MafulMutlaq,
    MafulFih,
    MafulLahu,
    MafulMahU,
    // Modifiers
    Hal,
    Tamyiz,
    Nat,
    // Construct state (idafa)
    Idafa,
    Mudaf,
    MudafIlayh,
    // Particles
    HarfJarr,
    Majrur,
    HarfNasb,
    HarfJazm,
    // Other
    Zarf,
    Qayd,
    Takid,
    Badal,
    Atasf,
    Istithna,
    Nida,
    Jawab,
    Shart,
    Jaza,
    Sila,
    Rabits,
    Unknown,
}

// ──────────────────────────────────────────────
//  Sentence Types
// ──────────────────────────────────────────────

/// Arabic sentence type classification (SPEC-0001-C4 §7.3, SPEC-0001-C5 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentenceType {
    JumlahIsmiyyah,
    JumlahFiliyyah,
    JumlahShartiyyah,
    JumlahZarfiyyah,
    Phrase,
    Incomplete,
    Unknown,
}

// ──────────────────────────────────────────────
//  Constituent Node Types
// ──────────────────────────────────────────────

/// Node type in a syntactic constituency tree (SPEC-0001-C5 §6.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Word,
    Phrase,
    Clause,
}

// ──────────────────────────────────────────────
//  Grammatical Flag
// ──────────────────────────────────────────────

/// A grammatical flag raised during analysis (SPEC-0001-C4 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammaticalFlag {
    /// Severity of the flag
    pub flag_type: FlagSeverity,

    /// Machine-readable error/warning code
    pub code: String,

    /// Human-readable message
    pub message: String,

    /// Indices of affected tokens
    pub token_indices: Vec<usize>,

    /// ID of the rule that raised this flag
    pub rule_id: String,
}

/// Severity of a grammatical flag (SPEC-0001-C4 §9.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagSeverity {
    Error,
    Warning,
    Info,
}

// ──────────────────────────────────────────────
//  Ambiguity
// ──────────────────────────────────────────────

/// Confidence level for ambiguous analyses (SPEC-0001-C3 §1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_f64(&self) -> f64 {
        match self {
            Confidence::High => 0.9,
            Confidence::Medium => 0.6,
            Confidence::Low => 0.3,
        }
    }
}

// ──────────────────────────────────────────────
//  Cache Types
// ──────────────────────────────────────────────

/// Cache backend type (SPEC-0001-C4 §15.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheBackend {
    Memory,
    Redis,
    Filesystem,
    Database,
}

/// Cache statistics (SPEC-0001-C4 §15.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: u64,
    pub total_size_bytes: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub eviction_count: u64,
    pub backend_type: CacheBackend,
}
