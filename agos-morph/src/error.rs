//! # Morphology Engine Error Types
//!
//! Wraps internal morphology errors into the shared `PipelineError` envelope.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C4 §2.2: PipelineError envelope
//! - SPEC-0001-C8: Security, Validation & Error Handling
//! - SPEC-0001-C3 §2.7: MOD-01 error conditions

use agos_core::error::{codes, PipelineError};

/// Specialized `Result` alias for the morphology engine.
pub type MorphResult<T> = Result<T, MorphError>;

/// Errors that can occur within the morphology engine.
///
/// These are converted to `PipelineError` via `Into<PipelineError>` for
/// use with the `PipelineStage` trait.
#[derive(Debug, Clone)]
pub enum MorphError {
    // ── MOD-01: UnicodeValidator ──
    InvalidEncoding { position: usize },
    EmptyInput,
    MaxLengthExceeded { limit: usize, actual: usize },
    NonArabicChar { position: usize, character: char },
    UnsupportedChar { position: usize, character: char },

    // ── MOD-03: Tokenizer ──
    MaxSegmentationsExceeded { token_id: usize, limit: usize, actual: usize },

    // ── MOD-04: MorphologicalParser ──
    MaxAnalysesExceeded { token_id: usize, limit: usize, actual: usize },
    UnknownStem { stem: String },
    KbMissing { kb_id: String },

    // ── Future MOD-05 errors will be added here ──
}

impl MorphError {
    /// Convert to a fatal `PipelineError` with the given stage ID.
    ///
    /// This is the sole conversion path from `MorphError` to `PipelineError`.
    pub fn into_pipeline(self, stage: &'static str) -> PipelineError {
        match self {
            MorphError::InvalidEncoding { position } => PipelineError::fatal(
                codes::INVALID_ENCODING,
                format!("Invalid UTF-8 encoding at byte position {position}"),
                stage,
            )
            .with_hint("Re-encode input as valid UTF-8"),
            MorphError::EmptyInput => PipelineError::fatal(
                codes::EMPTY_INPUT,
                "Input text is empty",
                stage,
            )
            .with_hint("Provide non-empty Arabic text"),
            MorphError::MaxLengthExceeded { limit, actual } => PipelineError::fatal(
                codes::MAX_LENGTH_EXCEEDED,
                format!(
                    "Input exceeds maximum length: {actual} bytes > {limit} byte limit"
                ),
                stage,
            )
            .with_hint("Split input into smaller segments"),
            MorphError::NonArabicChar { position, character } => PipelineError::fatal(
                codes::NON_ARABIC_CHAR,
                format!(
                    "Non-Arabic character '{}' (U+{:04X}) at position {position}",
                    character, character as u32
                ),
                stage,
            )
            .with_hint(
                "Remove non-Arabic characters or disable strict_arabic_only mode",
            ),
            MorphError::UnsupportedChar { position, character } => PipelineError::fatal(
                codes::UNSUPPORTED_CHAR,
                format!(
                    "Unsupported character '{}' (U+{:04X}) at position {position}",
                    character, character as u32
                ),
                stage,
            )
            .with_hint(
                "Notify AGOS maintainers to add support for this character range",
            ),
            MorphError::MaxSegmentationsExceeded { token_id, limit, actual } => {
                PipelineError::fatal(
                    codes::MAX_SEGMENTATIONS_EXCEEDED,
                    format!(
                        "Token {token_id}: {actual} segmentations exceeds limit of {limit}"
                    ),
                    stage,
                )
                .with_hint("Increase max_segmentations or simplify the input")
            }
            MorphError::MaxAnalysesExceeded { token_id, limit, actual } => {
                PipelineError::fatal(
                    codes::MAX_ANALYSES_EXCEEDED,
                    format!(
                        "Token {token_id}: {actual} analyses exceeds limit of {limit}"
                    ),
                    stage,
                )
                .with_hint("Increase max_morphological_analyses or simplify the input")
            }
            MorphError::UnknownStem { stem } => {
                PipelineError::degraded(
                    codes::KB_MISSING,
                    format!("Stem '{}' could not be analyzed", stem),
                    stage,
                )
                .with_hint("Check that KB-0001 (Roots) and KB-0002 (Wazan) are loaded")
            }
            MorphError::KbMissing { kb_id } => {
                PipelineError::fatal(
                    codes::KB_MISSING,
                    format!("Required knowledge base {} not found", kb_id),
                    stage,
                )
                .with_hint("Verify knowledge base installation")
            }
        }
    }
}

impl std::fmt::Display for MorphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MorphError::InvalidEncoding { position } => {
                write!(f, "Invalid UTF-8 at byte {position}")
            }
            MorphError::EmptyInput => write!(f, "Empty input"),
            MorphError::MaxLengthExceeded { limit, actual } => {
                write!(f, "Input too long: {actual} > {limit} bytes")
            }
            MorphError::NonArabicChar { position, character } => {
                write!(f, "Non-Arabic char '{}' at {position}", character)
            }
            MorphError::UnsupportedChar { position, character } => {
                write!(f, "Unsupported char '{}' at {position}", character)
            }
            MorphError::MaxSegmentationsExceeded { token_id, limit, actual } => {
                write!(f, "Token {token_id}: {actual} segmentations exceeds limit of {limit}")
            }
            MorphError::MaxAnalysesExceeded { token_id, limit, actual } => {
                write!(f, "Token {token_id}: {actual} analyses exceeds limit of {limit}")
            }
            MorphError::UnknownStem { stem } => {
                write!(f, "Unknown stem: {stem}")
            }
            MorphError::KbMissing { kb_id } => {
                write!(f, "Missing KB: {kb_id}")
            }
        }
    }
}

impl std::error::Error for MorphError {}
