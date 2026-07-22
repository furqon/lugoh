//! # MOD-03: Tokenizer
//!
//! Segments each raw word token from the lexer into its constituent morphemes:
//! proclitics (prefixes), stem, and enclitics (suffixes). This is the first
//! stage in the AGOS pipeline that requires linguistic knowledge — it must
//! recognize Arabic clitics and separate them from the stem.
//!
//! ## Pipeline Interface
//!
//! ```ignore
//! Input:  TokenStream (IR-2)
//! Output: SegmentedTokenStream (IR-3)
//! ```
//!
//! ## Processing Steps
//!
//! 1. For each word token, identify proclitics and enclitics from known tables
//! 2. Generate candidate segmentations (including the no-clitic default)
//! 3. Assign confidence scores based on clitic count
//! 4. Non-word tokens pass through as single-segment "particle" tokens
//! 5. Build metadata (segmentable, ambiguous tokens, total ambiguity)
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §4: Tokenizer processing algorithm
//! - SPEC-0001-C5 §4: SegmentedTokenStream (IR-3) schema
//! - SPEC-0001-C4 §5.3: SegmentedToken / Morpheme schemas

use agos_core::error::PipelineResult;
use agos_core::ir::{
    Morpheme, RawToken, Segmentation, SegmentedToken,
    SegmentedTokenStream, SegmentedTokenStreamMetadata, TokenStream,
};
use agos_core::pipeline::{PipelineContext, PipelineStage};
use agos_core::types::{MorphemeType, TokenType};

use crate::error::MorphError;

// ──────────────────────────────────────────────
//  Arabic Proclitic Tables (SPEC-0001-C3 §4.4)
// ──────────────────────────────────────────────

/// All known proclitics (prefixes), ordered longest-first for greedy matching.
/// Based on SPEC-0001-C3 §4.4 Arabic Clitic Table.
const PROCLITICS: &[&str] = &[
    "سَوْفَ", // future
    "فَبِ",   // fa + bi (conjunction + preposition)
    "فَلِ",   // fa + li (conjunction + preposition)
    "وَبِ",   // wa + bi (conjunction + preposition)
    "وَال",   // wa + al (conjunction + definite article)
    "فَال",   // fa + al (conjunction + definite article)
    "بِال",   // bi + al (preposition + definite article)
    "لِل",    // li + al (preposition + definite article)
    "سَ",     // future marker
    "فَ",     // conjunction "and so/thus"
    "وَ",     // conjunction "and"
    "بِ",     // preposition "with/by"
    "لِ",     // preposition "for/to"
    "كَ",     // preposition "like"
    "ال",     // definite article (alif-lam)
    "أَ",     // interrogative prefix
];

// ──────────────────────────────────────────────
//  Arabic Enclitic Tables (SPEC-0001-C3 §4.4)
// ──────────────────────────────────────────────

/// All known enclitics (suffixes), ordered longest-first for greedy matching.
/// Based on SPEC-0001-C3 §4.4 Arabic Clitic Table.
/// NOTE: Ta-marbuta (ة) is NOT included — it is part of the stem per spec.
const ENCLITICS: &[&str] = &[
    // Object / possessive pronouns — dual & plural
    "كُمَا", // pronoun_2d
    "هُمَا", // pronoun_3d
    "كُنَّ", // pronoun_2fp
    "هُنَّ", // pronoun_3fp
    "كُمْ",  // pronoun_2mp
    "هُمْ",  // pronoun_3mp
    // Object / possessive pronouns — singular
    "نَا",   // pronoun_1p
    "نِي",   // pronoun_1s_obj
    "كِ",    // pronoun_2fs
    "كَ",    // pronoun_2ms
    "هَا",   // pronoun_3fs
    "هُ",    // pronoun_3ms
    // Verb subject markers — plural
    "تُمَا", // verb_subject_2d
    "تُنَّ", // verb_subject_2fp
    "تُمْ",  // verb_subject_2mp
    "تَا",   // verb_subject_3fd
    // Verb subject markers — singular
    "تُ",    // verb_subject_1s
    "تَ",    // verb_subject_2ms
    "تِ",    // verb_subject_2fs
    "وَا",   // verb_subject_3mp
    // Plural / dual markers
    "ينَ",   // plural_masc
    "ينِ",   // dual_gen
    "انِ",   // dual_nom
    "اتَ",   // plural_fem
    "اتُ",   // plural_fem_nom
    "اتِ",   // plural_fem_gen
    "ونَ",   // plural_masc_nom
    // Verb suffixes
    "ا",     // verb_dual
    "ي",     // verb_suffix
    "نَ",    // verb_plural_fem
];

// ──────────────────────────────────────────────
//  Characters that should NOT be segmented
// ──────────────────────────────────────────────

/// Characters that should not be segmented as clitics.
/// These are part of the stem and handled by the Morphological Parser.
const NON_SEGMENTABLE_CHARS: &[char] = &['ة']; // ta-marbuta

// ──────────────────────────────────────────────
//  Tokenizer Configuration
// ──────────────────────────────────────────────

/// Configuration for MOD-03: Tokenizer (SPEC-0001-C3 §4).
#[derive(Debug, Clone)]
pub struct TokenizerConfig {
    /// Maximum number of segmentation alternatives per token (default: 16).
    pub max_segmentations: usize,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            max_segmentations: 16,
        }
    }
}

// ──────────────────────────────────────────────
//  Tokenizer Stage
// ──────────────────────────────────────────────

/// MOD-03: Tokenizer — segments Arabic word tokens into morphemes.
///
/// This stage identifies Arabic proclitics (prefixes like وَ, فَ, بِ),
/// enclitics (suffixes like object pronouns, verb markers), and separates
/// them from the core stem. Non-word tokens (punctuation, numbers, etc.)
/// pass through as single-segment particles.
///
/// ## Determinism
///
/// Fully deterministic. Given the same TokenStream and configuration,
/// produces identical segmentation output every time.
///
/// ## Performance Targets (SPEC-0001-C3 §4.9)
///
/// | Metric | Target |
/// |--------|--------|
/// | Throughput | > 500K tokens/second |
/// | Latency (p50) | < 2 μs per token |
/// | Memory | O(a) where a = ambiguity factor × token length |
#[derive(Debug, Clone)]
pub struct Tokenizer {
    /// Configuration for this tokenizer instance
    pub config: TokenizerConfig,
}

impl Tokenizer {
    /// Create a new Tokenizer with the given configuration.
    pub fn new(config: TokenizerConfig) -> Self {
        Self { config }
    }

    /// Core segmentation algorithm (SPEC-0001-C3 §4.3).
    ///
    /// Processes each word token in the input stream, identifies clitics,
    /// and generates a segmented token stream with ambiguity alternatives.
    pub fn tokenize(&self, input: TokenStream) -> PipelineResult<SegmentedTokenStream> {
        let mut tokens: Vec<SegmentedToken> = Vec::new();

        for raw_token in &input.tokens {
            let segmented = match raw_token.token_type {
                TokenType::Word => self.segment_word_token(raw_token)?,
                _ => self.segment_non_word_token(raw_token),
            };
            tokens.push(segmented);
        }

        // Build metadata
        let total_tokens = tokens.len() as u64;
        // Segmentable tokens = Word-type tokens (non-word tokens like
        // punctuation, whitespace, numbers are not "segmentable")
        let segmentable_tokens = tokens
            .iter()
            .filter(|t| t.raw_token.token_type == TokenType::Word)
            .count() as u64;
        let ambiguous_tokens = tokens
            .iter()
            .filter(|t| t.segmentations.len() > 1)
            .count() as u64;
        let total_ambiguity = if segmentable_tokens > 0 {
            let total_segs: usize = tokens.iter().map(|t| t.segmentations.len()).sum();
            total_segs as f64 / segmentable_tokens as f64
        } else {
            0.0
        };

        Ok(SegmentedTokenStream {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            tokens,
            metadata: SegmentedTokenStreamMetadata {
                total_tokens,
                segmentable_tokens,
                ambiguous_tokens,
                total_ambiguity,
            },
        })
    }

    /// Segment a word token into morphemes (SPEC-0001-C3 §4.3 Steps 2-6).
    fn segment_word_token(&self, token: &RawToken) -> PipelineResult<SegmentedToken> {
        let text = &token.text;
        let mut segmentations: Vec<Segmentation> = Vec::new();

        // ── Step 1: Generate candidate segmentations ──
        // Try all possible proclitic combinations
        for proclitic_text in PROCLITICS {
            if let Some(rest_after_prefix) = text.strip_prefix(proclitic_text) {
                // Don't segment if the "stem" is empty
                if rest_after_prefix.is_empty() {
                    continue;
                }

                // Try all possible enclitic combinations on the remaining stem
                for enclitic_text in ENCLITICS {
                    if let Some(stem_text) = rest_after_prefix.strip_suffix(enclitic_text) {
                        // Don't segment if the stem is empty
                        if stem_text.is_empty() {
                            continue;
                        }
                        // Also don't segment if stem is just ta-marbuta
                        if stem_text.trim_matches(NON_SEGMENTABLE_CHARS).is_empty() {
                            continue;
                        }
                        segmentations.push(self.build_segmentation(
                            token,
                            proclitic_text,
                            stem_text,
                            enclitic_text,
                            0.9, // Multiple clitics = high confidence
                        ));
                    }
                }

                // Also try with no enclitic (proclitic only)
                if !rest_after_prefix.is_empty() {
                    segmentations.push(self.build_segmentation(
                        token,
                        proclitic_text,
                        rest_after_prefix,
                        "",
                        0.7, // Single clitic = medium confidence
                    ));
                }
            }
        }

        // Try enclitic only (no proclitic)
        for enclitic_text in ENCLITICS {
            if let Some(stem_text) = text.strip_suffix(enclitic_text) {
                if stem_text.is_empty() {
                    continue;
                }
                if stem_text.trim_matches(NON_SEGMENTABLE_CHARS).is_empty() {
                    continue;
                }
                segmentations.push(self.build_segmentation(
                    token,
                    "",
                    stem_text,
                    enclitic_text,
                    0.7, // Single clitic = medium confidence
                ));
            }
        }

        // Always include the default (no segmentation) alternative
        segmentations.push(self.build_segmentation(
            token,
            "",
            text,
            "",
            0.3, // No clitics = low confidence (rare in real Arabic)
        ));

        // ── Step 2: Deduplicate and limit segmentations ──
        segmentations.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        segmentations.dedup_by(|a, b| {
            a.morphemes.len() == b.morphemes.len()
                && a.morphemes
                    .iter()
                    .zip(&b.morphemes)
                    .all(|(ma, mb)| ma.text == mb.text && ma.morpheme_type == mb.morpheme_type)
        });

        // Limit to max_segmentations
        if segmentations.len() > self.config.max_segmentations {
            return Err(MorphError::MaxSegmentationsExceeded {
                token_id: token.id,
                limit: self.config.max_segmentations,
                actual: segmentations.len(),
            }
            .into_pipeline("MOD-03"));
        }

        Ok(SegmentedToken {
            raw_token: token.clone(),
            segmentations,
        })
    }

    /// Handle a non-word token (punctuation, number, whitespace, etc.).
    /// Creates a single segmentation with the entire token as a particle.
    fn segment_non_word_token(&self, token: &RawToken) -> SegmentedToken {
        let morpheme = Morpheme {
            text: token.text.clone(),
            morpheme_type: MorphemeType::Particle,
            original_offset: 0,
            length: token.text.len(),
        };

        let segmentation = Segmentation {
            id: format!("seg-{}-0", token.id),
            morphemes: vec![morpheme],
            confidence: 1.0,
            source: "default".to_string(),
        };

        SegmentedToken {
            raw_token: token.clone(),
            segmentations: vec![segmentation],
        }
    }

    /// Build a segmentation from identified prefix, stem, and suffix.
    fn build_segmentation(
        &self,
        token: &RawToken,
        prefix_text: &str,
        stem_text: &str,
        suffix_text: &str,
        confidence: f64,
    ) -> Segmentation {
        let mut morphemes: Vec<Morpheme> = Vec::new();
        let offset = 0;

        // Add prefix morpheme (if present)
        if !prefix_text.is_empty() {
            morphemes.push(Morpheme {
                text: prefix_text.to_string(),
                morpheme_type: MorphemeType::Prefix,
                original_offset: offset,
                length: prefix_text.len(),
            });
        }

        // Compute the offset of the stem within the full text
        let stem_offset = prefix_text.len();

        // Add stem morpheme
        morphemes.push(Morpheme {
            text: stem_text.to_string(),
            morpheme_type: MorphemeType::Stem,
            original_offset: stem_offset,
            length: stem_text.len(),
        });

        // Add suffix morpheme (if present)
        if !suffix_text.is_empty() {
            let suffix_offset = stem_offset + stem_text.len();
            morphemes.push(Morpheme {
                text: suffix_text.to_string(),
                morpheme_type: MorphemeType::Suffix,
                original_offset: suffix_offset,
                length: suffix_text.len(),
            });
        }

        Segmentation {
            id: format!("seg-{}-{}", token.id, confidence),
            morphemes,
            confidence,
            source: "default".to_string(),
        }
    }
}

// ──────────────────────────────────────────────
//  PipelineStage Implementation
// ──────────────────────────────────────────────

impl PipelineStage<TokenStream, SegmentedTokenStream> for Tokenizer {
    fn stage_id(&self) -> &'static str {
        "MOD-03"
    }

    fn process(
        &self,
        input: TokenStream,
        _ctx: &PipelineContext,
    ) -> PipelineResult<SegmentedTokenStream> {
        self.tokenize(input)
    }

    fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
        if self.config.max_segmentations == 0 {
            return Err(agos_core::error::PipelineError::fatal(
                agos_core::error::codes::INVALID_REQUEST,
                "TokenizerConfig.max_segmentations must be > 0",
                "MOD-03",
            ));
        }
        Ok(())
    }
}

impl Default for Tokenizer {
    fn default() -> Self {
        Self::new(TokenizerConfig::default())
    }
}

// ──────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agos_core::error::codes;
    use agos_core::ir::{RawToken, TokenStream, TokenStreamMetadata};
    use agos_core::pipeline::PipelineContext;
    use agos_core::types::{GrammarSchool, TokenType};

    /// Helper to build a TokenStream from raw text tokens.
    fn make_token_stream(words: &[(&str, TokenType)]) -> TokenStream {
        let mut tokens = Vec::new();
        let mut offset: usize = 0;
        for (i, &(text, tt)) in words.iter().enumerate() {
            tokens.push(RawToken {
                id: i,
                text: text.to_string(),
                token_type: tt,
                start_offset: offset,
                end_offset: offset + text.len(),
            });
            offset += text.len();
        }
        let text: String = words.iter().map(|(t, _)| *t).collect();
        let token_count = tokens.len() as u64;
        let word_count = tokens.iter().filter(|t| t.token_type == TokenType::Word).count() as u64;
        TokenStream {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            text,
            tokens,
            metadata: TokenStreamMetadata {
                token_count,
                word_count,
                has_tokens: token_count > 0,
            },
        }
    }

    fn test_context() -> PipelineContext {
        PipelineContext::new(GrammarSchool::Basra)
    }

    // ── Basic Functionality ──

    #[test]
    fn test_simple_word_no_clitics() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // Simple word without prefixes or suffixes
        let stream = make_token_stream(&[("كتاب", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.tokens.len(), 1);
        // Should have at least the default segmentation
        assert!(output.tokens[0].segmentations.len() >= 1);
    }

    #[test]
    fn test_word_with_wa_prefix() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // وَ (wa = "and") + كَتَبَ (kataba = "he wrote") => وَكَتَبَ
        let stream = make_token_stream(&[("وَكَتَبَ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // Should have at least a segmentation with وَ as prefix
        let wa_seg = segs.iter().find(|s| {
            s.morphemes.first().map(|m| m.text == "وَ" && m.morpheme_type == MorphemeType::Prefix)
                == Some(true)
        });
        assert!(wa_seg.is_some(), "Should find segmentation with و prefix");
    }

    #[test]
    fn test_word_with_pronoun_suffix() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // كتاب (kitab = "book") + هُ ( = "his") => كِتَابُهُ (nominative)
        // Note: the vowel on the enclitic depends on grammatical case
        let stream = make_token_stream(&[("كِتَابُهُ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        let suffix_seg = segs.iter().find(|s| {
            s.morphemes.last().map(|m| m.text == "هُ" && m.morpheme_type == MorphemeType::Suffix)
                == Some(true)
        });
        assert!(suffix_seg.is_some(), "Should find segmentation with ه suffix");
    }

    #[test]
    fn test_word_with_prefix_and_suffix() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // و (wa) + بِ (bi) + كِتَاب (kitab) + هُ (hu) => وَبِكِتَابُهُ (nominative)
        // Note: the vowel on the enclitic depends on grammatical case
        let stream = make_token_stream(&[("وَبِكِتَابُهُ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // Find the full segmentation with prefix and suffix
        let full_seg = segs.iter().find(|s| {
            s.morphemes.len() >= 3
                && s.morphemes[0].morpheme_type == MorphemeType::Prefix
                && s.morphemes[1].morpheme_type == MorphemeType::Stem
                && s.morphemes.last().unwrap().morpheme_type == MorphemeType::Suffix
        });
        assert!(full_seg.is_some(), "Should find full prefix+stem+suffix segmentation");
    }

    #[test]
    fn test_empty_token_stream() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        let stream = make_token_stream(&[]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.tokens.len(), 0);
        assert_eq!(output.metadata.total_tokens, 0);
    }

    // ── Non-Word Token Handling ──

    #[test]
    fn test_non_word_tokens_unchanged() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        let stream = make_token_stream(&[
            ("كتاب", TokenType::Word),
            (" ", TokenType::Whitespace),
            ("!", TokenType::Punctuation),
            ("123", TokenType::Number),
        ]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.tokens.len(), 4);
        // Non-word tokens should have 1 segmentation (particle)
        for token in &output.tokens {
            if token.raw_token.token_type != TokenType::Word {
                assert_eq!(token.segmentations.len(), 1,
                    "Non-word token should have 1 segmentation");
                assert_eq!(
                    token.segmentations[0].morphemes[0].morpheme_type,
                    MorphemeType::Particle
                );
            }
        }
    }

    // ── Confidence Scoring ──

    #[test]
    fn test_confidence_scoring() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // Multiple clitics: وَ + كَتَب + وا (they wrote) => وَكَتَبُوا
        let stream = make_token_stream(&[("وَكَتَبُوا", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // The no-clitic (default) segmentation should have lowest confidence
        let default_seg = segs.iter().find(|s| s.morphemes.len() == 1);
        assert!(default_seg.is_some());
        assert!(
            default_seg.unwrap().confidence < 0.7,
            "Default segmentation should have low confidence"
        );
    }

    // ── Metadata ──

    #[test]
    fn test_metadata_counts() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        let stream = make_token_stream(&[
            ("وَكَتَبَ", TokenType::Word),    // segmentable
            (" ", TokenType::Whitespace),       // not segmentable (non-word)
            ("كِتَابُهُ", TokenType::Word),    // segmentable
            ("!", TokenType::Punctuation),      // not segmentable
        ]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.metadata.total_tokens, 4);
        assert_eq!(output.metadata.segmentable_tokens, 2);
    }

    // ── Clitic Combinations ──

    #[test]
    fn test_combined_proclitic_fabi() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // فَبِ (fa + bi = "and with/by") + اَللَّهِ (allahi = "Allah") => فَبِاللَّهِ
        let stream = make_token_stream(&[("فَبِاللَّهِ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // Should find segmentation with فَبِ as a combined prefix
        let has_fabi = segs.iter().any(|s| {
            s.morphemes.first().map(|m| m.text == "فَبِ") == Some(true)
        });
        assert!(has_fabi, "Should find combined فَبِ prefix");
    }

    #[test]
    fn test_default_segmentation_always_present() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        let stream = make_token_stream(&[("بِسْمِ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // The default (no clitics) segmentation should always be present
        let default = segs.iter().find(|s| s.morphemes.len() == 1);
        assert!(default.is_some(), "Default segmentation should always exist");
        assert_eq!(default.unwrap().morphemes[0].text, "بِسْمِ");
    }

    // ── Edge Cases ──

    #[test]
    fn test_short_token_no_clitic_match() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // Very short token "ل" shouldn't produce extra segmentations
        // (it's shorter than most clitics)
        let stream = make_token_stream(&[("ل", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // Should only have the default segmentation
        assert_eq!(segs.len(), 1, "Short token should only have default segmentation");
    }

    #[test]
    fn test_definite_article_is_segmented() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // الكتاب (al-kitabu = "the book") — ال is now segmented
        // After MOD-01 tashkeel normalization: "الْكِتَابُ" → "الكتاب"
        let stream = make_token_stream(&[("الكتاب", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;

        // There SHOULD be a segmentation that separates ال from the stem
        let al_seg = segs.iter().find(|s| {
            s.morphemes.first()
                .map(|m| m.text == "ال" && m.morpheme_type == MorphemeType::Prefix)
                == Some(true)
                && s.morphemes.iter().any(|m| m.morpheme_type == MorphemeType::Stem)
        });
        assert!(
            al_seg.is_some(),
            "Should find segmentation with ال prefix + stem"
        );

        // The stem should be "كتاب" after removing ال
        if let Some(seg) = al_seg {
            let stem = seg.morphemes.iter()
                .find(|m| m.morpheme_type == MorphemeType::Stem)
                .expect("Segmentation should have a stem");
            assert_eq!(stem.text, "كتاب", "Stem after ال should be كتاب");
        }

        // The default (unsegmented) segmentation should still be present
        let has_default = segs.iter().any(|s| s.morphemes.len() == 1);
        assert!(has_default, "Default (unsegmented) segmentation should still be present");
    }

    #[test]
    fn test_duplicate_segmentations_deduplicated() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // A word that might produce the same segmentation via different paths
        let stream = make_token_stream(&[("اللَّهِ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // Check no duplicate segmentations (same morpheme texts and types)
        for i in 0..segs.len() {
            for j in (i + 1)..segs.len() {
                let same = segs[i].morphemes.len() == segs[j].morphemes.len()
                    && segs[i]
                        .morphemes
                        .iter()
                        .zip(&segs[j].morphemes)
                        .all(|(a, b)| a.text == b.text && a.morpheme_type == b.morpheme_type);
                assert!(!same, "No duplicate segmentations allowed");
            }
        }
    }

    #[test]
    fn test_pipeline_stage_trait() {
        let tokenizer = Tokenizer::default();
        assert_eq!(tokenizer.stage_id(), "MOD-03");
        let ctx = test_context();
        let result = tokenizer.validate_config(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_rejects_zero_max() {
        let tokenizer = Tokenizer::new(TokenizerConfig { max_segmentations: 0 });
        let ctx = test_context();
        let result = tokenizer.validate_config(&ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, codes::INVALID_REQUEST);
    }

    #[test]
    fn test_segmented_token_stream_spec_fields() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        let stream = make_token_stream(&[("السَّلَامُ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.spec, "SPEC-0001");
        assert_eq!(output.version, "1.0");
    }

    #[test]
    fn test_word_with_bi_proclitic() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // بِ (bi = "with/by") + سْمِ (ismi = "name") => بِسْمِ
        let stream = make_token_stream(&[("بِسْمِ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        let bi_seg = segs.iter().find(|s| {
            s.morphemes.first().map(|m| m.text == "بِ" && m.morpheme_type == MorphemeType::Prefix)
                == Some(true)
        });
        assert!(bi_seg.is_some(), "Should find segmentation with بِ prefix");
        // The stem should be the remaining text
        let stem = bi_seg.unwrap().morphemes.iter().find(|m| m.morpheme_type == MorphemeType::Stem);
        assert!(stem.is_some());
        assert_eq!(stem.unwrap().text, "سْمِ");
    }

    #[test]
    fn test_word_with_plural_suffix_una() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // كتاب + ونَ (plural masc nom) => كَتَبُونَ
        let stream = make_token_stream(&[("كَتَبُونَ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        let suffix_seg = segs.iter().find(|s| {
            s.morphemes.last().map(|m| m.text == "ونَ" && m.morpheme_type == MorphemeType::Suffix)
                == Some(true)
        });
        assert!(suffix_seg.is_some(), "Should find segmentation with ونَ suffix");
    }

    #[test]
    fn test_real_arabic_phrase() {
        let tokenizer = Tokenizer::default();
        let ctx = test_context();
        // وَعَلَيْكُمُ (wa + alaykumu = "and upon you all")
        let stream = make_token_stream(&[("وَعَلَيْكُمُ", TokenType::Word)]);
        let result = tokenizer.process(stream, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        let segs = &output.tokens[0].segmentations;
        // Should have at least the default and some clitic-based segmentations
        assert!(segs.len() >= 1);
        // The default segmentation should have the full word as stem
        let default = segs.iter().find(|s| s.morphemes.len() == 1);
        assert!(default.is_some());
        assert_eq!(default.unwrap().morphemes[0].text, "وَعَلَيْكُمُ");
    }
}
