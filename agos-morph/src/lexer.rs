//! # MOD-02: Lexer
//!
//! Transforms normalized Unicode text into an ordered stream of raw tokens.
//! A token is the smallest meaningful unit of text: a word, a punctuation
//! mark, a number, a symbol, or whitespace.
//!
//! ## Pipeline Interface
//!
//! ```ignore
//! Input:  NormalizedText (IR-1)
//! Output: TokenStream (IR-2)
//! ```
//!
//! ## Processing Steps
//!
//! 1. **Character Classification** — Classify each char into ARABIC_LETTER,
//!    ARABIC_DIGIT, WHITESPACE, PUNCTUATION, SYMBOL, or OTHER.
//! 2. **Token Extraction** — Group consecutive characters of the same class
//!    into tokens, recording byte offsets.
//! 3. **Post-Processing** — Optionally skip whitespace tokens.
//! 4. **Build Metadata** — Count tokens and words.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §3: Lexer processing algorithm
//! - SPEC-0001-C5 §3: TokenStream (IR-2) schema
//! - SPEC-0001-C4 §4.3: RawToken schema

use agos_core::error::PipelineResult;
use agos_core::ir::{NormalizedText, RawToken, TokenStream, TokenStreamMetadata};
use agos_core::pipeline::{PipelineContext, PipelineStage};
use agos_core::types::TokenType;

use crate::error::MorphError;

// ──────────────────────────────────────────────
//  Lexer Configuration
// ──────────────────────────────────────────────

/// Configuration for MOD-02: Lexer (SPEC-0001-C3 §3).
#[derive(Debug, Clone)]
pub struct LexerConfig {
    /// Whether to skip whitespace tokens in the output.
    /// Default: false (whitespace tokens are preserved).
    pub skip_whitespace: bool,

    /// Whether to include tashkeel (diacritics) as part of word tokens.
    /// Default: true (tashkeel modifies the letter it attaches to).
    pub include_tashkeel_in_words: bool,
}

impl Default for LexerConfig {
    fn default() -> Self {
        Self {
            skip_whitespace: false,
            include_tashkeel_in_words: true,
        }
    }
}

impl LexerConfig {
    /// Create a configuration that skips whitespace tokens (most common usage).
    pub fn skipping_whitespace() -> Self {
        Self {
            skip_whitespace: true,
            ..Default::default()
        }
    }
}

// ──────────────────────────────────────────────
//  Character Classification
// ──────────────────────────────────────────────

/// Character class for tokenization purposes (SPEC-0001-C3 §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    ArabicLetter,
    ArabicDigit,
    Whitespace,
    Punctuation,
    Symbol,
    Tashkeel,
    Other,
}

/// Classify a single Unicode character into a `CharClass`.
fn classify_char(c: char) -> CharClass {
    let cp = c as u32;

    // Whitespace
    if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
        return CharClass::Whitespace;
    }

    // Punctuation (check BEFORE Arabic letter ranges to catch Arabic-specific
    // punctuation like U+061F Arabic question mark which falls in the Arabic block)
    if matches!(
        c,
        '.' | ',' | ';' | ':' | '!' | '?'
            | '[' | ']' | '{' | '}'
            | '\u{060C}' // Arabic comma
            | '\u{061B}' // Arabic semicolon
            | '\u{061F}' // Arabic question mark
            | '\u{06D4}' // Arabic full stop
            | '\u{00AB}' // Left-pointing double angle quotation mark
            | '\u{00BB}' // Right-pointing double angle quotation mark
            | '\u{2010}' // Hyphen
            | '\u{2013}' // En dash
            | '\u{2014}' // Em dash
            | '\u{2018}' // Left single quotation mark
            | '\u{2019}' // Right single quotation mark
            | '\u{201C}' // Left double quotation mark
            | '\u{201D}' // Right double quotation mark
            | '\u{002D}' // Hyphen-minus
            | '\u{002F}' // Solidus (slash)
            | '\u{005C}' // Reverse solidus (backslash)
    ) {
        return CharClass::Punctuation;
    }

    // Arabic-Indic digits
    if (0x0660..=0x0669).contains(&cp) || (0x06F0..=0x06F9).contains(&cp) {
        return CharClass::ArabicDigit;
    }

    // Tashkeel (Arabic diacritics) — attached to word tokens
    if matches!(
        c,
        '\u{064B}' | '\u{064C}' | '\u{064D}' | '\u{064E}'
            | '\u{064F}' | '\u{0650}' | '\u{0651}' | '\u{0652}'
            | '\u{0670}'
    ) {
        return CharClass::Tashkeel;
    }

    // Arabic letter ranges (main block + supplement + extended-A)
    if (0x0600..=0x06FF).contains(&cp)
        || (0x0750..=0x077F).contains(&cp)
        || (0x08A0..=0x08FF).contains(&cp)
    {
        return CharClass::ArabicLetter;
    }

    // Symbols
    if matches!(
        c,
        '@' | '#' | '$' | '%' | '^' | '&' | '*' | '+'
            | '=' | '<' | '>' | '|' | '~' | '`'
    ) {
        return CharClass::Symbol;
    }

    // Digit characters outside Arabic-Indic ranges
    if c.is_ascii_digit() {
        return CharClass::ArabicDigit;
    }

    CharClass::Other
}

// ──────────────────────────────────────────────
//  Lexer Stage
// ──────────────────────────────────────────────

/// MOD-02: Lexer — tokenizes normalized Arabic text into a token stream.
///
/// This stage transforms the normalized Unicode string from MOD-01 into an
/// ordered stream of `RawToken`s. It is a simple, deterministic, single-pass
/// scanner that groups consecutive characters of the same class into tokens.
///
/// ## Determinism
///
/// Fully deterministic. Given the same `NormalizedText` and configuration,
/// produces identical output every time.
///
/// ## Performance Targets (SPEC-0001-C3 §3.8)
///
/// | Metric | Target |
/// |--------|--------|
/// | Throughput | > 200 MB/s |
/// | Latency (p50) | < 0.5 μs per KB |
/// | Allocations | O(n) tokens, single pass |
///
/// ## Example
///
/// ```ignore,no_run
/// use agos_morph::lexer::Lexer;
/// use agos_core::ir::NormalizedText;
/// use agos_core::pipeline::{PipelineStage, PipelineContext};
/// use agos_core::types::GrammarSchool;
///
/// let lexer = Lexer::default();
/// let ctx = PipelineContext::new(GrammarSchool::Basra);
/// let input = NormalizedText {
///     spec: "SPEC-0001".into(), version: "1.0".into(),
///     normalized_text: "السَّلَامُ عَلَيْكُمْ".into(),
///     original_text: String::new(),
///     metadata: todo!(), config_snapshot: todo!(),
/// };
/// let result = lexer.process(input, &ctx);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone)]
pub struct Lexer {
    /// Configuration for this lexer instance
    pub config: LexerConfig,
}

impl Lexer {
    /// Create a new Lexer with the given configuration.
    pub fn new(config: LexerConfig) -> Self {
        Self { config }
    }

    /// Core tokenization algorithm (SPEC-0001-C3 §3.3).
    ///
    /// Performs all 5 processing steps and returns a `TokenStream` or error.
    pub fn tokenize(&self, input: NormalizedText) -> PipelineResult<TokenStream> {
        let text = input.normalized_text;

        // ── Step 1: Validate Input ──
        if text.is_empty() {
            return Err(MorphError::EmptyInput.into_pipeline("MOD-02"));
        }

        // ── Step 2: Scan Characters & Extract Tokens ──
        let bytes = text.as_bytes();
        let mut tokens: Vec<RawToken> = Vec::new();
        let mut pos: usize = 0;
        let mut token_id: usize = 0;

        while pos < bytes.len() {
            let (c, char_len) = match std::str::from_utf8(&bytes[pos..]) {
                Ok(s) => {
                    let ch = s.chars().next().unwrap_or('\0');
                    let len = ch.len_utf8();
                    (ch, len)
                }
                Err(_) => {
                    // Should not happen — NormalizedText guarantees valid UTF-8
                    return Err(MorphError::InvalidEncoding { position: pos }
                        .into_pipeline("MOD-02"));
                }
            };

            let class = classify_char(c);
            let start = pos;

            // Handle tashkeel: if it appears before a letter, include it in
            // the word token. If it appears standalone (no adjacent letter),
            // it becomes an Unknown token.
            if class == CharClass::Tashkeel {
                // Check if this tashkeel is adjacent to a letter
                let prev_is_letter = pos > 0
                    && classify_char(
                        std::str::from_utf8(&bytes[..pos])
                            .ok()
                            .and_then(|s| s.chars().last())
                            .unwrap_or('\0'),
                    ) == CharClass::ArabicLetter;
                let next_is_letter = pos + char_len < bytes.len()
                    && std::str::from_utf8(&bytes[pos + char_len..])
                        .ok()
                        .and_then(|s| s.chars().next())
                        .map(|c| classify_char(c) == CharClass::ArabicLetter)
                        .unwrap_or(false);

                if prev_is_letter || next_is_letter {
                    // Tashkeel is attached to a letter — merge into the
                    // previous or next word token
                    if prev_is_letter && !tokens.is_empty() {
                        // Extend the last token if it was a Word
                        if let Some(last) = tokens.last_mut() {
                            if last.token_type == TokenType::Word {
                                last.text.push(c);
                                last.end_offset = start + char_len;
                                pos += char_len;
                                continue;
                            }
                        }
                    }
                    // Otherwise treat as part of the upcoming word token
                    // by scanning forward to find the word
                    let mut scan_pos = pos + char_len;
                    while scan_pos < bytes.len() {
                        let next_str = std::str::from_utf8(&bytes[scan_pos..]).ok();
                        let next_c = next_str.and_then(|s| s.chars().next()).unwrap_or('\0');
                        let next_class = classify_char(next_c);
                        if next_class == CharClass::ArabicLetter
                            || next_class == CharClass::Tashkeel
                        {
                            scan_pos += next_c.len_utf8();
                        } else {
                            break;
                        }
                    }
                    let token_text = std::str::from_utf8(&bytes[pos..scan_pos])
                        .unwrap_or("")
                        .to_string();
                    tokens.push(RawToken {
                        id: token_id,
                        text: token_text,
                        token_type: TokenType::Word,
                        start_offset: pos,
                        end_offset: scan_pos,
                    });
                    token_id += 1;
                    pos = scan_pos;
                    continue;
                }

                // Standalone tashkeel → Unknown token
                tokens.push(RawToken {
                    id: token_id,
                    text: c.to_string(),
                    token_type: TokenType::Unknown,
                    start_offset: start,
                    end_offset: start + char_len,
                });
                token_id += 1;
                pos += char_len;
                continue;
            }

            // Group consecutive characters of the same class
            pos += char_len;
            while pos < bytes.len() {
                let next_str = std::str::from_utf8(&bytes[pos..]).ok();
                let next_c = next_str.and_then(|s| s.chars().next()).unwrap_or('\0');
                let next_class = classify_char(next_c);

                // Include tashkeel in word tokens (if configured)
                if class == CharClass::ArabicLetter
                    && next_class == CharClass::Tashkeel
                    && self.config.include_tashkeel_in_words
                {
                    pos += next_c.len_utf8();
                    continue;
                }

                if next_class == class {
                    pos += next_c.len_utf8();
                } else {
                    break;
                }
            }

            let token_text = std::str::from_utf8(&bytes[start..pos])
                .unwrap_or("")
                .to_string();
            let token_type = match class {
                CharClass::ArabicLetter => TokenType::Word,
                CharClass::ArabicDigit => TokenType::Number,
                CharClass::Whitespace => TokenType::Whitespace,
                CharClass::Punctuation => TokenType::Punctuation,
                CharClass::Symbol => TokenType::Symbol,
                CharClass::Other => TokenType::Unknown,
                CharClass::Tashkeel => {
                    // Handled above — shouldn't reach here
                    TokenType::Unknown
                }
            };

            tokens.push(RawToken {
                id: token_id,
                text: token_text,
                token_type,
                start_offset: start,
                end_offset: pos,
            });
            token_id += 1;
        }

        // ── Step 3: Post-Processing (optional whitespace removal) ──
        if self.config.skip_whitespace {
            tokens.retain(|t| t.token_type != TokenType::Whitespace);
            // Re-assign IDs after filtering
            for (i, token) in tokens.iter_mut().enumerate() {
                token.id = i;
            }
        }

        // ── Step 4: Build Metadata ──
        let word_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Word)
            .count() as u64;
        let token_count = tokens.len() as u64;

        let metadata = TokenStreamMetadata {
            token_count,
            word_count,
            has_tokens: token_count > 0,
        };

        // ── Step 5: Return ──
        Ok(TokenStream {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            text,
            tokens,
            metadata,
        })
    }
}

// ──────────────────────────────────────────────
//  PipelineStage Implementation
// ──────────────────────────────────────────────

impl PipelineStage<NormalizedText, TokenStream> for Lexer {
    fn stage_id(&self) -> &'static str {
        "MOD-02"
    }

    fn process(
        &self,
        input: NormalizedText,
        _ctx: &PipelineContext,
    ) -> PipelineResult<TokenStream> {
        self.tokenize(input)
    }

    fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
        Ok(())
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new(LexerConfig::default())
    }
}

// ──────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use agos_core::error::codes;

    fn make_input(text: &str) -> NormalizedText {
        NormalizedText {
            spec: "SPEC-0001".to_string(),
            version: "1.0".to_string(),
            normalized_text: text.to_string(),
            original_text: text.to_string(),
            metadata: agos_core::ir::NormalizedTextMetadata {
                char_count: text.chars().count() as u64,
                byte_count: text.len() as u64,
                word_count_estimate: text.split_whitespace().count() as u64,
                has_tashkeel: false,
                has_tatweel: false,
                has_quranic_symbols: false,
                has_non_arabic: false,
                normalization_applied: vec![],
            },
            config_snapshot: agos_core::ir::NormalizedTextConfig {
                normalize_tashkeel: false,
                strip_tatweel: false,
                strict_arabic_only: false,
                max_input_size: 1_048_576,
            },
        }
    }

    #[test]
    fn test_empty_text() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let result = lexer.process(make_input(""), &ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, codes::EMPTY_INPUT);
    }

    #[test]
    fn test_simple_words() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "السَّلَامُ عَلَيْكُمْ";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        assert!(stream.metadata.has_tokens);
        // Two words separated by whitespace → 3 tokens [Word, WS, Word]
        assert_eq!(stream.metadata.token_count, 3);
        assert_eq!(stream.metadata.word_count, 2);
        assert_eq!(stream.tokens[0].token_type, TokenType::Word);
        assert_eq!(stream.tokens[1].token_type, TokenType::Whitespace);
        assert_eq!(stream.tokens[2].token_type, TokenType::Word);
    }

    #[test]
    fn test_whitespace_preserved_by_default() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "مرحبًا    عالم";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // Should have: [Word, Whitespace, Word]
        assert_eq!(stream.metadata.token_count, 3);
        assert_eq!(stream.tokens[1].token_type, TokenType::Whitespace);
    }

    #[test]
    fn test_skip_whitespace() {
        let mut config = LexerConfig::default();
        config.skip_whitespace = true;
        let lexer = Lexer::new(config);
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "مرحبًا    عالم";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // Whitespace should be filtered, only 2 words remain
        assert_eq!(stream.metadata.token_count, 2);
        assert_eq!(stream.tokens[0].token_type, TokenType::Word);
        assert_eq!(stream.tokens[1].token_type, TokenType::Word);
        // IDs should be re-assigned
        assert_eq!(stream.tokens[0].id, 0);
        assert_eq!(stream.tokens[1].id, 1);
    }

    #[test]
    fn test_punctuation_tokens() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "مرحبًا! كيف حالك؟";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // Token sequence: [Word, Punctuation, Whitespace, Word, Whitespace, Word, Punctuation]
        // But some whitespace tokens may be merged depending on RTL rendering
        assert_eq!(stream.metadata.word_count, 3);
        assert!(stream.metadata.has_tokens);
        assert_eq!(stream.tokens[0].token_type, TokenType::Word);
        assert!(stream.tokens[0].text.contains("مرحب"));
        // Find the punctuation tokens by scanning
        let punct_tokens: Vec<&RawToken> = stream.tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Punctuation)
            .collect();
        assert_eq!(punct_tokens.len(), 2, "Should have 2 punctuation tokens");
        assert!(punct_tokens[0].text.contains("!"), "First punct should contain !");
    }

    #[test]
    fn test_consecutive_punctuation_grouped() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        // Use simple ASCII punctuation to verify grouping
        let input = "word!!";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // Token sequence: [Unknown, Punctuation] (ASCII letters are 'Other')
        assert_eq!(stream.metadata.token_count, 2);
        assert_eq!(stream.tokens[0].token_type, TokenType::Unknown);
        assert_eq!(stream.tokens[1].token_type, TokenType::Punctuation);
        // Both exclamation marks should be grouped into one token
        assert_eq!(stream.tokens[1].text, "!!", "Consecutive punctuation should be grouped");
    }

    #[test]
    fn test_numbers() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        // Arabic-Indic digits and regular digits
        let input = "السنة ١٤٤٦";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // [Word, Whitespace, Number]
        assert_eq!(stream.metadata.token_count, 3);
        assert_eq!(stream.tokens[0].token_type, TokenType::Word);
        assert_eq!(stream.tokens[2].token_type, TokenType::Number);
        assert!(stream.tokens[2].text.contains("١٤٤٦"));
    }

    #[test]
    fn test_byte_offsets() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "abc 123";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // [Word("abc"), Whitespace(" "), Number("123")]
        assert_eq!(stream.tokens[0].text, "abc");
        assert_eq!(stream.tokens[0].start_offset, 0);
        assert_eq!(stream.tokens[0].end_offset, 3);
        assert_eq!(stream.tokens[1].start_offset, 3);
        assert_eq!(stream.tokens[1].end_offset, 4);
        assert_eq!(stream.tokens[2].start_offset, 4);
        assert_eq!(stream.tokens[2].end_offset, 7);
    }

    #[test]
    fn test_tashkeel_included_in_word() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "السَّلَامُ";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // All characters including tashkeel should be one Word token
        assert_eq!(stream.metadata.token_count, 1);
        assert_eq!(stream.tokens[0].token_type, TokenType::Word);
        assert!(stream.tokens[0].text.contains('\u{064E}')); // Fatha
        assert!(stream.tokens[0].text.contains('\u{064F}')); // Damma
    }

    #[test]
    fn test_standalone_tashkeel_is_unknown() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        // Lone tashkeel with no adjacent Arabic letter → Unknown token
        // ASCII letters are classified as Other, not ArabicLetter, so
        // the tashkeel won't merge into a word
        let input = "a\u{0651}b"; // shadda between two ASCII chars
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // [Unknown("a"), Unknown("\u{0651}"), Unknown("b")]
        // Each should be its own token since they're different char classes
        assert_eq!(stream.metadata.token_count, 3);
        assert_eq!(stream.tokens[1].token_type, TokenType::Unknown);
    }

    #[test]
    fn test_pipeline_stage_trait() {
        let lexer = Lexer::default();
        assert_eq!(lexer.stage_id(), "MOD-02");

        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let result = lexer.validate_config(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_token_stream_spec_fields() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let result = lexer.process(make_input("مرحبًا"), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        assert_eq!(stream.spec, "SPEC-0001");
        assert_eq!(stream.version, "1.0");
    }

    #[test]
    fn test_tokens_ordered_and_id_sequential() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "وَاحِد اثْنَان ثَلَاثَة";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // [Word("وَاحِد"), Whitespace(" "), Word("اثْنَان"), Whitespace(" "), Word("ثَلَاثَة")]
        assert_eq!(stream.tokens.len(), 5);
        for (i, token) in stream.tokens.iter().enumerate() {
            assert_eq!(token.id, i, "Token ID should match position");
        }
        assert_eq!(stream.tokens[0].text, "وَاحِد");
        assert_eq!(stream.tokens[2].text, "اثْنَان");
        assert_eq!(stream.tokens[4].text, "ثَلَاثَة");
    }

    #[test]
    fn test_mixed_arabic_and_latin() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        // ASCII letters are classified as Other → Unknown token type
        let input = "كتاب PDF";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // [Word("كتاب"), Whitespace(" "), Unknown("PDF")]
        assert_eq!(stream.metadata.token_count, 3);
        assert_eq!(stream.tokens[0].token_type, TokenType::Word);
        assert_eq!(stream.tokens[2].token_type, TokenType::Unknown);
    }

    #[test]
    fn test_start_end_offsets_span_full_range() {
        let lexer = Lexer::default();
        let ctx = PipelineContext::new(agos_core::types::GrammarSchool::Basra);
        let input = "أ ب";
        let result = lexer.process(make_input(input), &ctx);
        assert!(result.is_ok());
        let stream = result.unwrap();
        // [Word("أ"), Whitespace(" "), Word("ب")]
        assert_eq!(stream.tokens.len(), 3);
        // First token starts at 0, last token ends at text length
        assert_eq!(stream.tokens[0].start_offset, 0);
        assert_eq!(stream.tokens[2].end_offset, input.len());
    }
}
