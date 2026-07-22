//! # Intermediate Representations (IR-1 through IR-11)
//!
//! Defines the data types for all 11 intermediate representations in the
//! AGOS compilation pipeline. Each IR is a deterministic snapshot of the
//! analysis state at a particular pipeline stage boundary.
//!
//! ## Pipeline Flow
//!
//! ```text
//! Input → IR-1 (NormalizedText) → IR-2 (TokenStream) → IR-3 (SegmentedTokenStream) →
//! IR-4 (MorphologicalAnalysis) → IR-5 (SyntaxTree) → IR-6 (GrammarIR) →
//! IR-7 (AnnotatedGIR) → IR-8 (ResolvedGIR) → IR-9 (GrammarBytecode) →
//! IR-10 (AnalysisResult) → IR-11 (ExplanationOutput)
//! ```
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C5: Data Flow & Intermediate Representations (complete IR schemas)
//! - SPEC-0001-C4: Module Responsibilities & Interfaces (I/O types for each module)

use serde::{Deserialize, Serialize};

use crate::evidence::EvidenceEntry;
use crate::feature::NamedFeature;
use crate::types::*;
use crate::version::KnowledgeVersionMap;

// ════════════════════════════════════════════
//  IR-1: NormalizedText
// ════════════════════════════════════════════

/// IR-1: Validated and normalized Arabic text (SPEC-0001-C5 §2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedText {
    pub spec: String,
    pub version: String,

    /// NFC-normalized, validated UTF-8 Arabic text
    pub normalized_text: String,

    /// Exact original input (unchanged)
    pub original_text: String,

    /// Metadata about the normalization process
    pub metadata: NormalizedTextMetadata,

    /// Snapshot of the configuration that produced this IR
    pub config_snapshot: NormalizedTextConfig,
}

/// Metadata from Unicode validation (SPEC-0001-C5 §2.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTextMetadata {
    pub char_count: u64,
    pub byte_count: u64,
    pub word_count_estimate: u64,
    pub has_tashkeel: bool,
    pub has_tatweel: bool,
    pub has_quranic_symbols: bool,
    pub has_non_arabic: bool,
    pub normalization_applied: Vec<String>,
}

/// Configuration snapshot for MOD-01 (SPEC-0001-C5 §2.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTextConfig {
    pub normalize_tashkeel: bool,
    pub strip_tatweel: bool,
    pub strict_arabic_only: bool,
    pub max_input_size: u64,
}

// ════════════════════════════════════════════
//  IR-2: TokenStream
// ════════════════════════════════════════════

/// IR-2: Stream of raw tokens from the lexer (SPEC-0001-C5 §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStream {
    pub spec: String,
    pub version: String,

    /// Reference to the normalized input text
    pub text: String,

    /// Ordered list of raw tokens
    pub tokens: Vec<RawToken>,

    pub metadata: TokenStreamMetadata,
}

/// A single raw token produced by the lexer (SPEC-0001-C4 §4.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawToken {
    /// 0-based sequential ID
    pub id: usize,

    /// Token text (substring of the input)
    pub text: String,

    /// Classification of the token
    pub token_type: TokenType,

    /// Byte offset in the input text (start, inclusive)
    pub start_offset: usize,

    /// Byte offset in the input text (end, exclusive)
    pub end_offset: usize,
}

/// Metadata for the token stream (SPEC-0001-C5 §3.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStreamMetadata {
    pub token_count: u64,
    pub word_count: u64,
    pub has_tokens: bool,
}

// ════════════════════════════════════════════
//  IR-3: SegmentedTokenStream
// ════════════════════════════════════════════

/// IR-3: Tokens segmented into morphemes (SPEC-0001-C5 §4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedTokenStream {
    pub spec: String,
    pub version: String,

    /// Tokens with their segmentation alternatives
    pub tokens: Vec<SegmentedToken>,

    pub metadata: SegmentedTokenStreamMetadata,
}

/// A token with its segmentation alternatives (SPEC-0001-C4 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedToken {
    /// The raw token from IR-2
    pub raw_token: RawToken,

    /// Segmentation alternatives, ordered by confidence (descending)
    pub segmentations: Vec<Segmentation>,
}

/// A single segmentation of a token into morphemes (SPEC-0001-C5 §4.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segmentation {
    /// Unique segmentation ID
    pub id: String,

    /// Ordered morphemes: prefix → stem → suffix
    pub morphemes: Vec<Morpheme>,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Source of this segmentation
    pub source: String,
}

/// A single morpheme within a segmentation (SPEC-0001-C5 §4.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Morpheme {
    /// Morpheme text
    pub text: String,

    /// Morpheme type
    pub morpheme_type: MorphemeType,

    /// Offset within the raw token text
    pub original_offset: usize,

    /// Length of the morpheme
    pub length: usize,
}

/// Metadata for the segmented token stream (SPEC-0001-C5 §4.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedTokenStreamMetadata {
    pub total_tokens: u64,
    pub segmentable_tokens: u64,
    pub ambiguous_tokens: u64,
    pub total_ambiguity: f64,
}

// ════════════════════════════════════════════
//  IR-4: MorphologicalAnalysis
// ════════════════════════════════════════════

/// IR-4: Morphological analysis of each token (SPEC-0001-C5 §5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologicalAnalysis {
    pub spec: String,
    pub version: String,

    /// Analyses for each token
    pub token_analyses: Vec<TokenAnalysis>,

    pub metadata: MorphologicalAnalysisMetadata,
}

/// Morphological analysis for a single token (SPEC-0001-C5 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalysis {
    /// Token ID from IR-2/IR-3
    pub token_id: usize,

    /// Stem analyses (one per segmentation), ordered by confidence
    pub stem_analyses: Vec<StemAnalysis>,
}

/// Analysis of a single stem (SPEC-0001-C5 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StemAnalysis {
    /// Unique analysis ID
    pub analysis_id: String,

    /// Reference to the segmentation from IR-3
    pub segmentation_id: String,

    /// The core stem (after clitic removal)
    pub stem: String,

    /// Extracted root (from KB-0001), if applicable
    pub root: Option<RootRef>,

    /// Identified morphological pattern/wazan (from KB-0002), if applicable
    pub wazan: Option<WazanRef>,

    /// Part of speech
    pub pos: PartOfSpeech,

    /// Extracted morphological features
    pub features: Vec<NamedFeature>,

    /// Whether this analysis has alternatives
    pub is_ambiguous: bool,

    /// Alternative analyses (recursive)
    pub alternatives: Vec<StemAnalysis>,

    /// Evidence trail for this analysis
    pub evidence: Vec<EvidenceEntry>,
}

/// Reference to a root entry in KB-0001 (SPEC-0001-C5 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootRef {
    /// Root text (e.g., "كتب")
    pub text: String,

    /// KB-0001 entry ID
    pub source: String,

    /// Confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// Reference to a wazan entry in KB-0002 (SPEC-0001-C5 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WazanRef {
    /// Pattern text (e.g., "فَعَلَ")
    pub text: String,

    /// KB-0002 entry ID
    pub source: String,

    /// Verb form (I-XV), or None for nouns
    pub form: Option<u8>,

    /// Confidence (0.0 to 1.0)
    pub confidence: f64,
}

/// Metadata for morphological analysis (SPEC-0001-C5 §5.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologicalAnalysisMetadata {
    pub total_tokens: u64,
    pub analyzed_tokens: u64,
    pub ambiguous_tokens: u64,
    pub unknown_tokens: u64,
    pub unknown_stems: Vec<String>,
}

// ════════════════════════════════════════════
//  IR-5: SyntaxTree
// ════════════════════════════════════════════

/// IR-5: Syntactic parse trees (SPEC-0001-C5 §6).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTree {
    pub spec: String,
    pub version: String,

    /// Parse trees ordered by confidence (descending)
    pub trees: Vec<ParseTree>,

    pub metadata: SyntaxTreeMetadata,
}

/// A single parse tree (SPEC-0001-C5 §6.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseTree {
    /// Unique tree ID
    pub id: String,

    /// Sentence type
    pub tree_type: SentenceType,

    /// Root constituent
    pub root: Constituent,

    /// Overall confidence (0.0 to 1.0)
    pub confidence: f64,

    /// Grammar school that produced this parse
    pub source: String,
}

/// A constituent in the parse tree (SPEC-0001-C4 §7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constituent {
    /// Node type
    pub node_type: NodeType,

    /// Grammatical role
    pub role: SyntacticRole,

    /// Token IDs that belong to this constituent
    pub token_ids: Vec<usize>,

    /// Child constituents
    pub children: Vec<Constituent>,

    /// Feature map (e.g., {"case": "nominative"})
    pub features: std::collections::HashMap<String, String>,

    /// Whether this constituent is implied (hadhf — ellipsis)
    pub implicit: bool,
}

/// Metadata for syntax parsing (SPEC-0001-C5 §6.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxTreeMetadata {
    pub sentence_count: u64,
    pub tokens_parsed: u64,
    pub ambiguity_count: u64,
    pub parse_time_ms: f64,
}

// ════════════════════════════════════════════
//  IR-6: GrammarIR (GIR)
// ════════════════════════════════════════════

/// IR-6: Unified Grammar Intermediate Representation (SPEC-0001-C5 §7).
///
/// The GIR combines morphological analysis and syntax into a single,
/// versioned, serializable intermediate representation. This is the
/// front-end/back-end boundary in the AGOS pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarIR {
    pub spec: String,
    pub version: String,

    /// Pipeline metadata
    pub metadata: GIRMetadata,

    /// Original input text
    pub text: String,

    /// Enriched tokens with morphology + syntax
    pub tokens: Vec<GIRToken>,

    /// Syntactic trees (ambiguity forest)
    pub trees: Vec<GIRTree>,

    /// Consolidated evidence from MOD-03 through MOD-06
    pub evidence: Vec<EvidenceEntry>,
}

/// GIR pipeline metadata (SPEC-0001-C5 §7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIRMetadata {
    pub created_at: String,
    pub pipeline_version: String,
    pub knowledge_versions: KnowledgeVersionMap,
    pub school: String,
}

/// A token in the GIR with combined morphological and syntactic data (SPEC-0001-C5 §7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIRToken {
    /// 0-based token index
    pub index: usize,

    /// Token text from the original input
    pub original_text: String,

    /// Token text from the normalized input
    pub normalized_text: String,

    /// Byte offset in the original text (start)
    pub start_offset: usize,

    /// Byte offset in the original text (end)
    pub end_offset: usize,

    /// Separated clitics
    pub clitics: TokenClitics,

    /// Morphological analysis (single or ambiguous)
    pub morphology: StemAnalysis,
}

/// Clitics attached to a token (SPEC-0001-C5 §7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClitics {
    pub prefixes: Vec<String>,
    pub suffixes: Vec<String>,
}

/// A syntactic tree in the GIR (SPEC-0001-C5 §7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIRTree {
    pub id: String,
    pub sentence_type: String,
    pub root: GIRConstituent,
    pub confidence: f64,
    pub source: String,
}

/// A constituent in a GIR tree (SPEC-0001-C5 §7.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GIRConstituent {
    pub node_type: NodeType,
    pub role: SyntacticRole,
    pub token_indices: Vec<usize>,
    pub children: Vec<GIRConstituent>,
    pub features: std::collections::HashMap<String, String>,
    pub confidence: f64,
}

// ════════════════════════════════════════════
//  IR-7: AnnotatedGIR
// ════════════════════════════════════════════

/// IR-7: GIR annotated with rule applications (SPEC-0001-C5 §8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotatedGIR {
    /// Inherits all fields from IR-6
    pub gir: GrammarIR,

    /// Rule applications (ordered by application)
    pub rule_applications: Vec<RuleApplication>,

    /// Grammatical flags raised during rule application
    pub flags: Vec<GrammaticalFlag>,

    /// Version of the rule set used
    pub rule_set_version: String,

    /// Grammar school
    pub school: String,
}

/// A single rule application recorded in the evidence trail (SPEC-0001-C4 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleApplication {
    pub rule_id: String,
    pub rule_name: String,
    pub school: String,
    pub version: String,

    /// What this rule applied to
    pub applies_to: RuleApplicationTarget,

    /// Condition that triggered this rule
    pub condition: String,

    /// Action that was taken
    pub action: String,

    /// Result of the rule application
    pub result: RuleApplicationResult,

    /// Evidence entry
    pub evidence: EvidenceEntry,
}

/// Target of a rule application (SPEC-0001-C4 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleApplicationTarget {
    pub token_indices: Vec<usize>,
    pub constituent_path: Vec<String>,
}

/// Result of a rule application (SPEC-0001-C4 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleApplicationResult {
    /// Analysis IDs that were confirmed
    pub confirmed: Vec<String>,

    /// Analysis IDs that were rejected
    pub rejected: Vec<String>,

    /// Features that were modified
    pub modified: Vec<FeatureModification>,

    /// Grammatical flag (if any)
    pub flag: Option<GrammaticalFlag>,
}

/// A single feature modification (SPEC-0001-C4 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureModification {
    pub feature: String,
    pub from: String,
    pub to: String,
}

// ════════════════════════════════════════════
//  IR-8: ResolvedGIR
// ════════════════════════════════════════════

/// IR-8: GIR with resolved knowledge graph references (SPEC-0001-C5 §9).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedGIR {
    /// Inherits all fields from IR-7
    pub annotated_gir: AnnotatedGIR,

    /// Tokens enriched with resolved KB entries
    pub tokens: Vec<ResolvedToken>,

    /// KB versions used during resolution
    pub knowledge_versions: KnowledgeVersionMap,

    /// Resolution statistics
    pub resolution_stats: ResolutionStats,
}

/// A token enriched with resolved KB entries (SPEC-0001-C5 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedToken {
    /// GIR token data
    pub gir_token: GIRToken,

    /// Resolved root entry from KB-0001 (if applicable)
    pub root_entry: Option<RootEntry>,

    /// Resolved wazan entry from KB-0002 (if applicable)
    pub wazan_entry: Option<WazanEntry>,

    /// Dictionary entry (optional)
    pub dictionary_entry: Option<DictionaryEntry>,

    /// Semantic tags
    pub semantic_tags: Vec<String>,
}

/// KB resolution statistics (SPEC-0001-C5 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionStats {
    pub roots_resolved: u64,
    pub patterns_resolved: u64,
    pub unresolved_references: u64,
    pub resolution_time_ms: f64,
}

/// Resolved root entry from KB-0001 (SPEC-0001-C5 §9.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootEntry {
    pub id: String,
    pub root: String,
    pub meaning: String,
    pub forms: Vec<String>,
    pub derived_nouns: Vec<String>,
    pub cognates: Vec<String>,
    pub semantic_field: String,
    pub cross_references: RootCrossReferences,
}

/// Cross-references for a root entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCrossReferences {
    pub related_roots: Vec<String>,
    pub antonyms: Vec<String>,
    pub synonyms: Vec<String>,
}

/// Resolved wazan entry from KB-0002.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WazanEntry {
    pub id: String,
    pub pattern: String,
    pub meaning: String,
    pub form: Option<u8>,
    pub example: String,
}

/// Dictionary entry for a word.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub id: String,
    pub word: String,
    pub definition: String,
    pub translations: std::collections::HashMap<String, String>,
    pub usage_examples: Vec<String>,
}

// ════════════════════════════════════════════
//  IR-9: GrammarBytecode
// ════════════════════════════════════════════

/// IR-9: Compiled Grammar Bytecode (SPEC-0001-C5 §10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarBytecode {
    pub spec: String,
    pub version: String,

    /// Complete serialized bytecode as raw bytes
    #[serde(with = "serde_bytes")]
    pub raw: Vec<u8>,

    /// Logical sections (for debugging/inspection)
    pub sections: Vec<BytecodeSection>,

    /// Total bytecode size in bytes
    pub size: u64,

    /// Metadata
    pub metadata: BytecodeMetadata,
}

/// A logical section within the bytecode (SPEC-0001-C5 §10.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeSection {
    pub section_type: BytecodeSectionType,
    pub data: Vec<u8>,
    pub offset: u64,
    pub size: u64,
}

/// Type of a bytecode section (SPEC-0001-C5 §10.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BytecodeSectionType {
    Header,
    Metadata,
    Tokens,
    Morphology,
    Syntax,
    Rules,
    Evidence,
    Strings,
    End,
}

/// Metadata for bytecode output (SPEC-0001-C5 §10.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeMetadata {
    pub input_text_hash: String,
    pub token_count: u64,
    pub tree_count: u64,
    pub rule_count: u64,
    pub compression_ratio: f64,
    pub gir_json_size_bytes: u64,
}

// ════════════════════════════════════════════
//  IR-10: AnalysisResult
// ════════════════════════════════════════════

/// IR-10: Final analysis result from the GVM (SPEC-0001-C5 §11).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub spec: String,
    pub version: String,

    pub metadata: AnalysisResultMetadata,

    /// Original input text
    pub input_text: String,

    /// SHA-256 hash of the input text
    pub input_text_hash: String,

    /// One analysis tree per successful parse
    pub trees: Vec<AnalysisTree>,

    /// Grammatical flags
    pub flags: Vec<GrammaticalFlag>,

    /// Complete evidence trail
    pub evidence: Vec<EvidenceEntry>,
}

/// Metadata for the analysis result (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResultMetadata {
    pub executed_at: String,
    pub execution_time_ms: f64,
    pub steps_executed: u64,
    pub memory_used: u64,
    pub bytecode_size: u64,
}

/// A single analysis tree (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTree {
    pub id: String,
    pub r#type: String,
    pub tokens: Vec<AnalysisToken>,
    pub constituents: Vec<GIRConstituent>,
    pub flags: Vec<GrammaticalFlag>,
    pub confidence: f64,
}

/// A token in the analysis result with combined features (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisToken {
    pub index: usize,
    pub text: String,
    pub features: AnalysisTokenFeatures,
    pub evidence: Vec<EvidenceEntry>,
}

/// Combined feature view for a token (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTokenFeatures {
    pub morphological: MorphologicalFeatureSet,
    pub syntactic: SyntacticFeatureSet,
    pub semantic: SemanticFeatureSet,
}

/// Morphological features for display (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphologicalFeatureSet {
    pub root: Option<String>,
    pub wazan: Option<String>,
    pub pos: String,
    pub gender: Option<String>,
    pub number: Option<String>,
    pub person: Option<String>,
    pub tense: Option<String>,
    pub mood: Option<String>,
    pub voice: Option<String>,
    pub case: Option<String>,
    pub state: Option<String>,
}

/// Syntactic features for display (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntacticFeatureSet {
    pub role: Option<String>,
    pub governor: Option<usize>,
}

/// Semantic features for display (SPEC-0001-C5 §11.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFeatureSet {
    pub tags: Vec<String>,
    pub definition: Option<String>,
    pub root_meaning: Option<String>,
}

// ════════════════════════════════════════════
//  IR-11: ExplanationOutput
// ════════════════════════════════════════════

/// IR-11: Human-readable explanation output (SPEC-0001-C5 §12).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationOutput {
    pub spec: String,
    pub version: String,

    pub metadata: ExplanationOutputMetadata,

    /// Original input text
    pub input_text: String,

    /// Summary of the grammatical analysis
    pub overview: String,

    /// Sentence type
    pub sentence_type: Option<String>,

    /// Word-by-word I'rab breakdown
    pub irab_breakdown: Vec<IrabEntry>,

    /// Notable grammatical constructions
    pub constructions: Vec<GrammaticalConstruction>,

    /// Grammatical flags
    pub flags: Vec<GrammaticalFlag>,

    /// Evidence trail (if include_evidence == true)
    pub evidence: Vec<EvidenceEntry>,

    /// Formatted output string (text, HTML, etc.)
    pub raw: String,
}

/// Metadata for explanation output (SPEC-0001-C5 §12.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationOutputMetadata {
    pub generated_at: String,
    pub language: String,
    pub format: String,
    pub llm_enhanced: bool,
    pub generation_time_ms: f64,
    pub pipeline_timing_ms: PipelineTiming,
}

/// Per-stage timing breakdown (SPEC-0001-C4 §16.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTiming {
    pub total: f64,
    pub validation: f64,
    pub lexing: f64,
    pub tokenization: f64,
    pub morphology: f64,
    pub syntax: f64,
    pub gir_construction: f64,
    pub rule_engine: f64,
    pub kg_resolution: f64,
    pub bytecode_generation: f64,
    pub gvm_execution: f64,
    pub explanation: f64,
}

/// A word-by-word I'rab entry (SPEC-0001-C5 §12.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrabEntry {
    pub token: String,
    pub root: Option<String>,
    pub pos: String,
    pub features: Vec<FeatureDisplay>,
    pub syntactic_role: Option<String>,
    pub explanation: String,
}

/// A feature name-value pair for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDisplay {
    pub name: String,
    pub value: String,
}

/// A notable grammatical construction (SPEC-0001-C5 §12.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammaticalConstruction {
    pub name: String,
    pub description: String,
    pub tokens: Vec<usize>,
}
