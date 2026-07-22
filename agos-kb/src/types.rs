//! # Knowledge Base Types
//!
//! Defines the data types for each of the 8 AGOS Knowledge Bases.
//! These types represent entries as they would be stored in the compiled
//! binary format and loaded at runtime.
//!
//! ## KB Suite
//!
//! | KB | ID | Content | Storage |
//! |----|-----|---------|---------|
//! | KB-0001 | Roots | ~15,000–20,000 Arabic roots | Binary trie |
//! | KB-0002 | Wazan | ~300–450 morphological patterns | Hash index |
//! | KB-0003 | VerbForms | ~180–250 conjugation tables | Table binary |
//! | KB-0004 | NounPatterns | ~135–180 noun patterns | Table binary |
//! | KB-0005 | Particles | ~120–200 particle entries | Hash index |
//! | KB-0006 | Pronouns | ~60–80 pronoun entries | Hash index |
//! | KB-0007 | Features | 19 features, ~107 values, ~33 rules | Feature map |
//! | KB-0008 | ParticlesDev | Compiled particle module | `.agos-kb` binary |
//!
//! ## Spec Alignment
//!
//! - KB-OVERVIEW: KB Suite Overview & Architecture
//! - KB-0001 through KB-0008: Individual KB specifications

use serde::{Deserialize, Serialize};

use agos_core::feature::{
    AgreementRule, FeatureConstraint, FeatureDefinition, InferenceRule,
};
use agos_core::version::SemVer;

/// Unique identifier for each knowledge base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KbId {
    Kb0001,  // Roots
    Kb0002,  // Wazan
    Kb0003,  // Verb Forms
    Kb0004,  // Noun Patterns
    Kb0005,  // Particles
    Kb0006,  // Pronouns
    Kb0007,  // Morphological Features
    Kb0008,  // Particles (Developer Reference)
}

impl KbId {
    /// Get the canonical KB ID string (e.g., "KB-0001").
    pub fn as_str(&self) -> &'static str {
        match self {
            KbId::Kb0001 => "KB-0001",
            KbId::Kb0002 => "KB-0002",
            KbId::Kb0003 => "KB-0003",
            KbId::Kb0004 => "KB-0004",
            KbId::Kb0005 => "KB-0005",
            KbId::Kb0006 => "KB-0006",
            KbId::Kb0007 => "KB-0007",
            KbId::Kb0008 => "KB-0008",
        }
    }

    /// Get the human-readable name (e.g., "Roots").
    pub fn name(&self) -> &'static str {
        match self {
            KbId::Kb0001 => "Roots",
            KbId::Kb0002 => "Wazan",
            KbId::Kb0003 => "VerbForms",
            KbId::Kb0004 => "NounPatterns",
            KbId::Kb0005 => "Particles",
            KbId::Kb0006 => "Pronouns",
            KbId::Kb0007 => "Features",
            KbId::Kb0008 => "ParticlesDev",
        }
    }

    /// Get the storage file name (without extension).
    pub fn file_name(&self) -> &'static str {
        match self {
            KbId::Kb0001 => "kb-0001-roots",
            KbId::Kb0002 => "kb-0002-wazan",
            KbId::Kb0003 => "kb-0003-verb-forms",
            KbId::Kb0004 => "kb-0004-noun-patterns",
            KbId::Kb0005 => "kb-0005-particles",
            KbId::Kb0006 => "kb-0006-pronouns",
            KbId::Kb0007 => "kb-0007-features",
            KbId::Kb0008 => "kb-0008-particles-dev",
        }
    }

    /// Get the expected compiled binary file extension.
    pub fn extension(&self) -> &'static str {
        "agos-kb"
    }
}

/// Metadata for a compiled knowledge base file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbMetadata {
    /// KB identifier (e.g., "KB-0001")
    pub kb_id: KbId,

    /// Semantic version of this KB
    pub version: SemVer,

    /// Name of the KB
    pub name: String,

    /// Description of the KB contents
    pub description: String,

    /// SHA-256 checksum of the KB content (for integrity verification)
    pub checksum_sha256: String,

    /// Format of the compiled data
    pub format: KbFormat,

    /// Entry count
    pub entry_count: u64,

    /// Compilation timestamp (ISO 8601)
    pub compiled_at: String,

    /// Dependency versions (KB ID → minimum version)
    pub dependencies: std::collections::HashMap<String, String>,

    /// KB compiler version used to produce this file
    pub compiler_version: String,
}

/// Format of a compiled KB's data section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KbFormat {
    BinaryTrie,
    HashIndex,
    TableBinary,
    FeatureMap,
    CompiledKb,
}

// ──────────────────────────────────────────────
//  KB-0001: Roots
// ──────────────────────────────────────────────

/// A single root entry from KB-0001.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootEntry {
    /// Unique entry ID within KB-0001
    pub id: String,

    /// Root consonants (e.g., "كتب")
    pub root: String,

    /// Root type classification
    pub root_type: RootType,

    /// Primary meaning in English
    pub meaning: String,

    /// Arabic meaning/gloss
    pub meaning_ar: Option<String>,

    /// Verb forms supported by this root (I through XV)
    pub verb_forms: Vec<u8>,

    /// Derived noun types from this root
    pub derived_nouns: Vec<String>,

    /// Semantic field classification
    pub semantic_field: Option<String>,

    /// Cognate roots (related roots with semantic connection)
    pub cognates: Vec<String>,

    /// Cross-references to other KB entries
    pub cross_references: RootCrossReferences,

    /// Notable attestations (Quran, Hadith, etc.)
    pub attestations: Vec<String>,
}

/// Cross-references for a root entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCrossReferences {
    pub related_roots: Vec<String>,
    pub antonyms: Vec<String>,
    pub synonyms: Vec<String>,
}

/// Root type classification (KB-0001 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RootType {
    /// Sound (صحيح سالم) — all root letters are regular consonants
    Sound,
    /// Weak initial (مثال) — first radical is و or ي
    WeakInitial,
    /// Weak middle (أجوف) — middle radical is و or ي
    WeakMiddle,
    /// Weak final (ناقص) — final radical is و or ي
    WeakFinal,
    /// Hamzated (مهموز) — one or more radicals is hamza
    Hamzated,
    /// Doubled (مضاعف) — final two radicals are identical
    Doubled,
    /// Sound quadriliteral (رباعي سالم)
    SoundQuadriliteral,
    /// Weak quadriliteral (رباعي معتل)
    WeakQuadriliteral,
    /// Unspecified
    Unspecified,
}

// ──────────────────────────────────────────────
//  KB-0002: Wazan
// ──────────────────────────────────────────────

/// A single wazan (morphological pattern) entry from KB-0002.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WazanEntry {
    /// Unique entry ID within KB-0002
    pub id: String,

    /// Pattern template (e.g., "فَعَلَ", "مَفْعُول")
    pub pattern: String,

    /// Pattern type
    pub pattern_type: PatternType,

    /// Semantic meaning of this pattern
    pub meaning: String,

    /// Verb form number (I-XV) if this is a verb pattern
    pub form: Option<u8>,

    /// Example word using this pattern
    pub example: String,

    /// Signature hash for O(1) pattern matching
    pub signature_hash: u64,

    /// Phonological template (consonant/vowel skeleton)
    pub phonological_template: String,

    /// Weak root variants of this pattern
    pub weak_variants: Vec<String>,
}

/// Type of morphological pattern (KB-0002 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    VerbForm,
    NounPattern,
    WeakVariant,
    Quadriliteral,
}

// ──────────────────────────────────────────────
//  KB-0003: Verb Forms
// ──────────────────────────────────────────────

/// A conjugation paradigm table from KB-0003.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbParadigm {
    /// Paradigm ID
    pub id: String,

    /// Verb form (I-XV)
    pub form: u8,

    /// Conjugation class
    pub class: ConjugationClass,

    /// Root type this paradigm applies to
    pub root_type: RootType,

    /// The 13 conjugation slots
    pub perfect: ConjugationSlots,
    pub imperfect: ConjugationSlots,
    pub moods: MoodSlots,
}

/// Conjugation class for verbs (KB-0003 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConjugationClass {
    Sound,           // صحيح سالم
    Hollow,          // أجوف (wawi/yai)
    Defective,       // ناقص (wawi/yai)
    Assimilated,     // مثال (wawi/yai)
    Doubled,         // مضاعف
    HamzatedFirst,   // مهموز الفاء
    HamzatedMid,     // مهموز العين
    HamzatedLast,    // مهموز اللام
    LafifMafruq,     // لفيف مفروق
    LafifMakrun,     // لفيف مقرون
    SoundQuad,       // رباعي سالم
    WeakQuad,        // رباعي معتل
}

/// The 13 conjugation slots (3ms, 3fs, 2ms, 2fs, 1s, 3md, 3fd, 2d, 3mp, 3fp, 2mp, 2fp, 1p).
pub type ConjugationSlots = [String; 13];

/// Mood conjugation for imperfect verbs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodSlots {
    pub indicative: ConjugationSlots,
    pub subjunctive: ConjugationSlots,
    pub jussive: ConjugationSlots,
    pub energetic_i: ConjugationSlots,
    pub energetic_ii: ConjugationSlots,
}

// ──────────────────────────────────────────────
//  KB-0004: Noun Patterns
// ──────────────────────────────────────────────

/// A derived noun pattern from KB-0004.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NounPatternEntry {
    /// Entry ID
    pub id: String,

    /// Noun type
    pub noun_type: NounType,

    /// Pattern template
    pub pattern: String,

    /// Meaning/function of this noun type
    pub meaning: String,

    /// Grammatical gender
    pub gender: NounGender,

    /// Declension class
    pub declension: DeclensionClass,

    /// Feminine form (if applicable)
    pub feminine_form: Option<String>,

    /// Sound plural formation
    pub sound_plural: Option<String>,

    /// Broken plural mappings (pattern → frequency ranking)
    pub broken_plurals: Vec<BrokenPluralMapping>,

    /// Weak root variants
    pub weak_variants: Vec<String>,
}

/// Noun type classification (KB-0004 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NounType {
    Masdar,
    IsmFail,           // Active participle (اسم فاعل)
    IsmMafUl,          // Passive participle (اسم مفعول)
    IsmMakan,          // Noun of place (اسم مكان)
    IsmZaman,          // Noun of time (اسم زمان)
    IsmAlah,           // Noun of instrument (اسم آلة)
    SifahMushabbahah,  // Resembling adjective (صفة مشبهة)
    Tafdil,            // Elative (اسم تفضيل)
    Nisbah,            // Relative adjective (نسبة)
    JamTaksir,         // Broken plural (جمع تكسير)
    IsmMarrati,        // Noun of instance (اسم مرة)
    IsmHayati,         // Noun of manner (اسم هيئة)
    Jins,              // Noun of genus (اسم جنس)
    IsmTasghir,        // Diminutive (اسم تصغير)
    NotANoun,
}

/// Grammatical gender for nouns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NounGender {
    Masculine,
    Feminine,
    Common,
}

/// Declension class for nouns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeclensionClass {
    Triptote,     // منصرف — fully declinable
    Diptote,      // غير منصرف — partially declinable
    Indeclinable, // مبني — invariable
}

/// Mapping from a singular noun pattern to a broken plural pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenPluralMapping {
    pub plural_pattern: String,
    pub frequency: u8,  // 1 = most common
    pub notes: Option<String>,
}

// ──────────────────────────────────────────────
//  KB-0005: Particles
// ──────────────────────────────────────────────

/// A particle entry from KB-0005.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEntry {
    /// Entry ID
    pub id: String,

    /// Particle text (e.g., "مِن", "عَلَى", "إِنَّ")
    pub text: String,

    /// Particle functional category
    pub category: ParticleCategory,

    /// Grammatical effect of this particle
    pub governance: ParticleGovernance,

    /// Subcategories or specific types
    pub sub_type: Option<String>,

    /// Meaning/function in English
    pub meaning: String,

    /// Notes on usage
    pub notes: Option<String>,

    /// Homograph resolution score (for ambiguous forms like ما)
    pub homograph_score: Option<u8>,
}

/// Particle functional category (KB-0005 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleCategory {
    Preposition,       // حرف جر
    Conjunction,       // حرف عطف
    Subjunctive,       // حرف نصب
    Jussive,           // حرف جزم
    Conditional,       // حرف شرط
    Interrogative,     // حرف استفهام
    Negative,          // حرف نفي
    Vocative,          // حرف نداء
    InnaSister,        // إنَّ وأخواتها
    KanaSister,        // كان وأخواتها
    AnswerException,   // جواب واستثناء
    MasdarForming,     // حرف مصدري
    Other,
}

/// Grammatical government effect of a particle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleGovernance {
    /// Case governed (for prepositions)
    pub case_government: Option<CaseGovernment>,
    /// Mood governed (for subjunctive/jussive particles)
    pub mood_government: Option<MoodGovernment>,
    /// Type of governance
    pub government_type: GovernmentType,
}

/// Case government types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseGovernment {
    Genitive,    // جر
    Accusative,  // نصب
    Nominative,  // رفع
}

/// Mood government types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoodGovernment {
    Subjunctive,  // نصب
    Jussive,      // جزم
}

/// Type of governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernmentType {
    Independent,          // مستقل — particle itself governs
    RequiresComplement,   // يحتاج إلى متعلق
}

// ──────────────────────────────────────────────
//  KB-0006: Pronouns
// ──────────────────────────────────────────────

/// A pronoun entry from KB-0006.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PronounEntry {
    /// Entry ID
    pub id: String,

    /// Pronoun text
    pub text: String,

    /// Pronoun type
    pub pronoun_type: PronounType,

    /// Person (1st, 2nd, 3rd)
    pub person: u8,

    /// Number
    pub number: PronounNumber,

    /// Gender
    pub pronoun_gender: PronounGender,

    /// Attachment type
    pub attachment: AttachmentType,

    /// Phonological variants (vowel-dependent form changes)
    pub phonetic_variants: Vec<String>,

    /// Script forms (different written forms)
    pub script_forms: Vec<String>,
}

/// Pronoun type classification (KB-0006 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PronounType {
    PersonalAttached,
    PersonalDetached,
    Demonstrative,
    Relative,
    Interrogative,
    Conditional,
    Compound,
    NotAPronoun,
}

/// Pronoun number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PronounNumber {
    Singular,
    Dual,
    Plural,
}

/// Pronoun gender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PronounGender {
    Masculine,
    Feminine,
    Common,
}

/// Attachment type for pronouns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentType {
    Standalone,   // منفصل
    Suffix,       // متصل (مفعول به / جار)
    Prefix,       // متصل (فاعل)
}

// ──────────────────────────────────────────────
//  KB-0007: Features
// ──────────────────────────────────────────────

/// The complete KB-0007 feature database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDatabase {
    /// All feature definitions
    pub features: Vec<FeatureDefinition>,

    /// Agreement rules
    pub agreement_rules: Vec<AgreementRule>,

    /// Inference rules
    pub inference_rules: Vec<InferenceRule>,

    /// Feature constraints
    pub constraints: Vec<FeatureConstraint>,
}

// ──────────────────────────────────────────────
//  General KB wrapper
// ──────────────────────────────────────────────

/// A loaded knowledge base instance at runtime.
#[derive(Debug, Clone)]
pub enum KnowledgeBase {
    Roots(KbStore<RootEntry>),
    Wazan(KbStore<WazanEntry>),
    VerbForms(KbStore<VerbParadigm>),
    NounPatterns(KbStore<NounPatternEntry>),
    Particles(KbStore<ParticleEntry>),
    Pronouns(KbStore<PronounEntry>),
    Features(FeatureDatabase),
}

/// Generic store for KB entries, representing a loaded compiled KB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbStore<T> {
    /// KB metadata
    pub metadata: KbMetadata,

    /// Loaded entries
    pub entries: Vec<T>,

    /// Index by key (e.g., root text, particle text) for O(1) lookup
    pub index: std::collections::HashMap<String, Vec<usize>>,
}

impl<T> KbStore<T> {
    /// Create a new empty KB store.
    pub fn new(metadata: KbMetadata) -> Self {
        Self {
            metadata,
            entries: Vec::new(),
            index: std::collections::HashMap::new(),
        }
    }

    /// Get an entry by its index.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.entries.get(index)
    }

    /// Look up entries by key.
    pub fn lookup(&self, key: &str) -> Option<&Vec<usize>> {
        self.index.get(key)
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
