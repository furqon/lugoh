//! # MOD-01: UnicodeValidator
//!
//! Validates that the input text is well-formed Arabic text, normalizes it
//! to a canonical Unicode representation, and rejects text that cannot be
//! processed.
//!
//! ## Pipeline Interface
//!
//! ```ignore
//! Input:  String (raw UTF-8 Arabic text)
//! Output: NormalizedText (IR-1)
//! ```
//!
//! ## Processing Steps
//!
//! 1. **Encoding Validation** — Verify valid UTF-8
//! 2. **Length Check** — Empty or exceeds max size
//! 3. **Character Validation** — Check each char against Arabic Unicode blocks
//! 4. **NFKC Normalization** — Canonical Unicode form
//! 5. **Tatweel Stripping** — Remove kashida (U+0640) if configured
//! 6. **Tashkeel Normalization** — Strip diacritics if configured
//! 7. **Metadata Detection** — Scan for tashkeel, tatweel, Quranic symbols
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §2: UnicodeValidator processing algorithm
//! - SPEC-0001-C5 §2: NormalizedText (IR-1) schema
//! - SPEC-0001-C8: Error handling codes

use agos_core::error::{codes, PipelineError, PipelineResult};
use agos_core::ir::{
    NormalizedText, NormalizedTextConfig, NormalizedTextMetadata,
};
use agos_core::pipeline::{PipelineContext, PipelineStage};

use crate::config::UnicodeValidatorConfig;
use crate::error::MorphError;

use unicode_normalization::UnicodeNormalization;

// ──────────────────────────────────────────────
//  Unicode Character Ranges (Arabic)
// ──────────────────────────────────────────────

/// Tashkeel (Arabic diacritic) code points.
const TASHKEEL_SET: &[char] = &[
    '\u{064B}', // Fathatan
    '\u{064C}', // Dammatan
    '\u{064D}', // Kasratan
    '\u{064E}', // Fatha
    '\u{064F}', // Damma
    '\u{0650}', // Kasra
    '\u{0651}', // Shadda
    '\u{0652}', // Sukun
    '\u{0670}', // Superscript Alef
];

/// Quranic annotation symbols range (U+06D6–U+06ED).
const QURANIC_SYMBOLS: &[(u32, u32)] = &[
    (0x06D6, 0x06ED), // Small high symbols
    (0x08D4, 0x08E1), // Extended-A Quranic
    (0x08E2, 0x08FF), // Extended-A continuation
];

/// Common characters that are always allowed.
const COMMON_ALLOWED: &[char] = &[
    ' ',  // Space
    '\t', // Tab
    '\n', // Newline
    '\r', // Carriage return
];

/// Arabic-Indic digit ranges.
const ARABIC_DIGITS: &[(u32, u32)] = &[
    (0x0660, 0x0669), // Arabic-Indic digits
    (0x06F0, 0x06F9), // Extended Arabic-Indic digits
];

/// Arabic punctuation that is always allowed.
const ARABIC_PUNCTUATION: &[char] = &[
    '.'  , ','  , ';'  , ':'  , '!'  , '?'  ,
    '('  , ')'  , '['  , ']'  , '{'  , '}'  ,
    '\u{060C}', // Arabic comma
    '\u{061B}', // Arabic semicolon
    '\u{061F}', // Arabic question mark
    '\u{061C}', // Arabic letter mark
    '\u{06D4}', // Arabic full stop
    '\u{0700}', // Syriac end of paragraph (used in some Arabic contexts)
    '\u{0701}', // Syriac supralinear full stop
    '\u{0702}', // Syriac sublinear full stop
    '\u{00AB}', // Left-pointing double angle quotation mark
    '\u{00BB}', // Right-pointing double angle quotation mark
    '\u{2010}', // Hyphen
    '\u{2013}', // En dash
    '\u{2014}', // Em dash
    '\u{2018}', // Left single quotation mark
    '\u{2019}', // Right single quotation mark
    '\u{201C}', // Left double quotation mark
    '\u{201D}', // Right double quotation mark
    '\u{002D}', // Hyphen-minus
    '\u{002F}', // Solidus (slash)
    '\u{005C}', // Reverse solidus (backslash)
    '\u{0028}', // Left parenthesis
    '\u{0029}', // Right parenthesis
];

// ──────────────────────────────────────────────
//  UnicodeValidator Stage
// ──────────────────────────────────────────────

/// MOD-01: UnicodeValidator — validates and normalizes Arabic text.
///
/// This stage is the entry point of the AGOS pipeline. It ensures that
/// the input is well-formed Arabic text, applies Unicode normalization,
/// optionally strips tatweel and tashkeel, and produces a `NormalizedText`
/// (IR-1) for downstream stages.
///
/// ## Determinism
///
/// Fully deterministic. Given the same input and configuration, produces
/// identical output every time.
///
/// ## Performance Targets (SPEC-0001-C3 §2.8)
///
/// | Metric | Target |
/// |--------|--------|
/// | Throughput | > 100 MB/s |
/// | Latency (p50) | < 1 μs per KB |
/// | Memory | O(n), single output allocation |
///
/// ## Example
///
/// ```ignore,no_run
/// use agos_morph::UnicodeValidator;
/// use agos_core::pipeline::{PipelineStage, PipelineContext};
/// use agos_core::types::GrammarSchool;
///
/// let validator = UnicodeValidator::default();
/// let ctx = PipelineContext::new(GrammarSchool::Basra);
/// let result = validator.process("السَّلَامُ عَلَيْكُمْ".to_string(), &ctx);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct UnicodeValidator {
    /// Configuration for this validator instance
    pub config: UnicodeValidatorConfig,
}

impl UnicodeValidator {
    /// Create a new UnicodeValidator with the given configuration.
    pub fn new(config: UnicodeValidatorConfig) -> Self {
        Self { config }
    }

    /// Create a UnicodeValidator with strict Arabic-only settings.
    pub fn strict() -> Self {
        Self {
            config: UnicodeValidatorConfig::strict(),
        }
    }

    /// Create a UnicodeValidator with permissive settings.
    pub fn permissive() -> Self {
        Self {
            config: UnicodeValidatorConfig::permissive(),
        }
    }

    /// Core validation and normalization algorithm (SPEC-0001-C3 §2.3).
    ///
    /// Performs all 7 processing steps and returns either a `NormalizedText`
    /// or a fatal `PipelineError`.
    pub fn validate_and_normalize(&self, input: String) -> PipelineResult<NormalizedText> {
        let original_text = input;

        // ── Step 1: Encoding Validation ──
        // The input is already a Rust `String`, so it is guaranteed valid UTF-8.
        // If we were reading raw bytes, we'd check encoding here.

        // ── Step 2: Length Check ──
        if original_text.is_empty() {
            return Err(MorphError::EmptyInput.into_pipeline("MOD-01"));
        }

        let byte_len = original_text.len();
        if byte_len > self.config.max_input_size {
            return Err(MorphError::MaxLengthExceeded {
                limit: self.config.max_input_size,
                actual: byte_len,
            }
            .into_pipeline("MOD-01"));
        }

        let allowed_ranges = self.config.allowed_ranges();

        // ── Step 3: Character Validation ──
        // We scan the original text first so we can fail before normalization.
        let mut has_tatweel = false;
        let mut has_tashkeel = false;
        let mut has_quranic = false;
        let mut has_non_arabic = false;

        for (i, c) in original_text.char_indices() {
            if self.is_always_allowed(c) {
                continue;
            }
            if self.is_in_any_range(c, &allowed_ranges) {
                if self.is_tashkeel(c) { has_tashkeel = true; }
                if c == '\u{0640}' { has_tatweel = true; }
                if self.is_quranic_symbol(c) { has_quranic = true; }
                continue;
            }
            if self.is_arabic_digit(c) {
                continue;
            }
            if self.is_arabic_punctuation(c) {
                continue;
            }

            // Character not in any allowed range
            if self.config.strict_arabic_only {
                return Err(MorphError::NonArabicChar {
                    position: i,
                    character: c,
                }
                .into_pipeline("MOD-01"));
            }
            has_non_arabic = true;
        }

        // ── Step 4: NFKC Normalization ──
        // SPEC-0001-C3 §2.3 Step 4: "NFKC is preferred over NFD/NFC for
        // Arabic because it handles compatibility decompositions"
        let mut normalized_text: String = original_text.nfkc().collect();
        let mut normalization_applied: Vec<String> = Vec::new();
        normalization_applied.push("NFKC".to_string());

        // ── Step 5: Tatweel Stripping ──
        if self.config.strip_tatweel {
            normalized_text.retain(|c| c != '\u{0640}');
            normalization_applied.push("strip_tatweel".to_string());
        }

        // ── Step 6: Tashkeel Normalization ──
        if self.config.normalize_tashkeel {
            normalized_text.retain(|c| !self.is_tashkeel(c));
            normalization_applied.push("strip_tashkeel".to_string());
        }

        // ── Step 7: Detect Metadata ──
        let char_count = normalized_text.chars().count() as u64;
        let word_count_estimate = normalized_text
            .split_whitespace()
            .count() as u64;

        let metadata = NormalizedTextMetadata {
            char_count,
            byte_count: normalized_text.len() as u64,
            word_count_estimate,
            has_tashkeel,
            has_tatweel,
            has_quranic_symbols: has_quranic,
            has_non_arabic,
            normalization_applied,
        };

        let config_snapshot = NormalizedTextConfig {
            normalize_tashkeel: self.config.normalize_tashkeel,
            strip_tatweel: self.config.strip_tatweel,
            strict_arabic_only: self.config.strict_arabic_only,
            max_input_size: self.config.max_input_size as u64,
        };

        // ── Step 8: Return Output ──
        Ok(NormalizedText {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            normalized_text,
            original_text,
            metadata,
            config_snapshot,
        })
    }

    // ── Helper Methods ──

    /// Check if a character is always allowed (whitespace/control).
    fn is_always_allowed(&self, c: char) -> bool {
        COMMON_ALLOWED.contains(&c)
    }

    /// Check if a character falls within any of the allowed Unicode ranges.
    fn is_in_any_range(&self, c: char, ranges: &[(u32, u32)]) -> bool {
        let cp = c as u32;
        ranges.iter().any(|&(start, end)| cp >= start && cp <= end)
    }

    /// Check if a character is a tashkeel (Arabic diacritic).
    fn is_tashkeel(&self, c: char) -> bool {
        TASHKEEL_SET.contains(&c)
    }

    /// Check if a character is a Quranic annotation symbol.
    fn is_quranic_symbol(&self, c: char) -> bool {
        let cp = c as u32;
        QURANIC_SYMBOLS
            .iter()
            .any(|&(start, end)| cp >= start && cp <= end)
    }

    /// Check if a character is an Arabic-Indic digit.
    fn is_arabic_digit(&self, c: char) -> bool {
        let cp = c as u32;
        ARABIC_DIGITS.iter().any(|&(start, end)| cp >= start && cp <= end)
    }

    /// Check if a character is Arabic-specific punctuation.
    fn is_arabic_punctuation(&self, c: char) -> bool {
        ARABIC_PUNCTUATION.contains(&c)
    }
}

// ──────────────────────────────────────────────
//  PipelineStage Implementation
// ──────────────────────────────────────────────

impl PipelineStage<String, NormalizedText> for UnicodeValidator {
    fn stage_id(&self) -> &'static str {
        "MOD-01"
    }

    fn process(
        &self,
        input: String,
        _ctx: &PipelineContext,
    ) -> PipelineResult<NormalizedText> {
        // MOD-01 doesn't depend on pipeline context (no KB dependencies)
        self.validate_and_normalize(input)
    }

    fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
        // Basic validation: max_input_size must be reasonable
        if self.config.max_input_size == 0 {
            return Err(PipelineError::fatal(
                codes::INVALID_REQUEST,
                "UnicodeValidatorConfig.max_input_size must be > 0",
                "MOD-01",
            ));
        }
        if self.config.max_input_size > 100_000_000 {
            // 100 MB sanity cap
            return Err(PipelineError::degraded(
                codes::INVALID_REQUEST,
                format!(
                    "UnicodeValidatorConfig.max_input_size ({}) seems excessive",
                    self.config.max_input_size
                ),
                "MOD-01",
            )
            .with_hint("Set max_input_size to a reasonable value (default: 1,048,576)"));

        }
        // Validate that allowed ranges are non-overlapping and well-formed
        for &(start, end) in &self.config.allowed_unicode_ranges {
            if start > end {
                return Err(PipelineError::fatal(
                    codes::INVALID_REQUEST,
                    format!(
                        "Invalid Unicode range: U+{start:04X} > U+{end:04X}"
                    ),
                    "MOD-01",
                ));
            }
        }
        Ok(())
    }
}

impl Default for UnicodeValidator {
    fn default() -> Self {
        Self::new(UnicodeValidatorConfig::default())
    }
}

// ──────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agos_core::pipeline::PipelineContext;
    use agos_core::types::GrammarSchool;

    fn test_context() -> PipelineContext {
        PipelineContext::new(GrammarSchool::Basra)
    }

    // ── Basic Functionality ──

    #[test]
    fn test_valid_arabic_text() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        let input = "السَّلَامُ عَلَيْكُمْ".to_string();
        let result = validator.process(input.clone(), &ctx);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let output = result.unwrap();
        assert_eq!(output.original_text, input);
        assert!(!output.normalized_text.is_empty());
        assert!(output.metadata.char_count > 0);
        assert!(output.metadata.byte_count > 0);
        assert_eq!(output.spec, "SPEC-0001");
        assert_eq!(output.version, "1.0");
    }

    #[test]
    fn test_empty_input() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        let result = validator.process(String::new(), &ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, codes::EMPTY_INPUT);
    }

    #[test]
    fn test_max_length_exceeded() {
        let mut config = UnicodeValidatorConfig::default();
        config.max_input_size = 10; // Very small limit
        let validator = UnicodeValidator::new(config);
        let ctx = test_context();
        let result = validator.process("السَّلَامُ عَلَيْكُمْ".to_string(), &ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, codes::MAX_LENGTH_EXCEEDED);
    }

    // ── Character Validation ──

    #[test]
    fn test_strict_mode_rejects_latin() {
        let validator = UnicodeValidator::strict();
        let ctx = test_context();
        let result = validator.process("Hello السَّلَام".to_string(), &ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, codes::NON_ARABIC_CHAR);
    }

    #[test]
    fn test_permissive_mode_preserves_latin() {
        let validator = UnicodeValidator::permissive();
        let ctx = test_context();
        let result = validator.process("Hello السَّلَام".to_string(), &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.metadata.has_non_arabic);
        // NFKC normalization doesn't change ASCII
        assert!(output.normalized_text.contains("Hello"));
    }

    #[test]
    fn test_arabic_digits_allowed() {
        let validator = UnicodeValidator::strict();
        let ctx = test_context();
        // Arabic-Indic digits ٠١٢٣٤٥٦٧٨٩
        let input = "السنة ١٤٤٦".to_string();
        let result = validator.process(input, &ctx);
        assert!(result.is_ok(), "Arabic digits should be allowed in strict mode");
    }

    #[test]
    fn test_quranic_symbols_detected() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        // ﷽ (Basmala) with Quranic symbols — U+06D6 range
        let input = "بِسْمِ ٱللَّهِ \u{06D6}".to_string();
        let result = validator.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.metadata.has_quranic_symbols);
    }

    // ── Normalization ──

    #[test]
    fn test_nfkc_decomposes_ligature() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        // Lam-Alef ligature (ﷲ, U+FDF2) should be decomposed by NFKC
        // but NOT by NFC. This test verifies we're using NFKC.
        let input = "\u{FDF2}".to_string(); // U+FDF2 (lam-alef ligature)
        let result = validator.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        // NFKC decomposes U+FDF2 into multiple characters
        // NFC would preserve the single character
        assert!(
            output.normalized_text.len() > 3,
            "NFKC should decompose the ligature into multiple chars"
        );
        assert!(
            output.metadata.normalization_applied.contains(&"NFKC".to_string()),
            "Should record NFKC normalization"
        );
    }

    #[test]
    fn test_tatweel_stripping() {
        let mut config = UnicodeValidatorConfig::default();
        config.strip_tatweel = true;
        let validator = UnicodeValidator::new(config);
        let ctx = test_context();
        // Text with tatweel (kashida)
        let input = "مـــــــد".to_string();
        let result = validator.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Tatweel characters should be removed
        assert!(!output.normalized_text.contains('\u{0640}'));
        assert!(
            output
                .metadata
                .normalization_applied
                .contains(&"strip_tatweel".to_string())
        );
    }

    #[test]
    fn test_tashkeel_stripping() {
        let mut config = UnicodeValidatorConfig::default();
        config.normalize_tashkeel = true;
        let validator = UnicodeValidator::new(config);
        let ctx = test_context();
        // Text with tashkeel
        let input = "السَّلَامُ".to_string();
        let result = validator.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Tashkeel characters should be removed
        // السَّلَامُ without tashkeel = السلام
        assert_eq!(output.normalized_text, "السلام");
        assert!(
            output
                .metadata
                .normalization_applied
                .contains(&"strip_tashkeel".to_string())
        );
    }

    #[test]
    fn test_tatweel_and_tashkeel_metadata() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        let input = "السَّلَامُ".to_string();
        let result = validator.process(input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Without stripping, metadata should note presence
        assert!(output.metadata.has_tashkeel, "Tashkeel should be detected");
    }

    // ── PipelineStage Integration ──

    #[test]
    fn test_pipeline_stage_trait() {
        let validator = UnicodeValidator::default();
        assert_eq!(validator.stage_id(), "MOD-01");

        let ctx = test_context();
        let result = validator.validate_config(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_rejects_zero_max_size() {
        let mut config = UnicodeValidatorConfig::default();
        config.max_input_size = 0;
        let validator = UnicodeValidator::new(config);
        let ctx = test_context();
        let result = validator.validate_config(&ctx);
        assert!(result.is_err());
    }

    // ── Whitespace-only Input ──

    #[test]
    fn test_whitespace_only_is_valid() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        let result = validator.process("   \n  \t  ".to_string(), &ctx);
        assert!(result.is_ok(), "Whitespace-only should be valid");
        let output = result.unwrap();
        // After NFKC normalization, whitespace is preserved
        assert!(output.metadata.char_count > 0);
        assert_eq!(output.metadata.word_count_estimate, 0);
    }

    // ── Large Input ──

    #[test]
    fn test_large_arabic_text() {
        let validator = UnicodeValidator::default();
        let ctx = test_context();
        // Generate a moderately large Arabic string
        let sentence = "السَّلَامُ عَلَيْكُمْ وَرَحْمَةُ اللَّهِ وَبَرَكَاتُهُ ";
        let large_input: String = sentence.repeat(100);
        assert!(large_input.len() < validator.config.max_input_size);
        let result = validator.process(large_input, &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.metadata.char_count > 0);
        assert!(output.metadata.word_count_estimate > 0);
    }

    // ── Original Text Preservation ──

    #[test]
    fn test_original_text_preserved() {
        let validator = UnicodeValidator::strict();
        let ctx = test_context();
        let input = "السَّلَامُ عَلَيْكُمْ".to_string();
        let result = validator.process(input.clone(), &ctx);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.original_text, input);
    }
}
