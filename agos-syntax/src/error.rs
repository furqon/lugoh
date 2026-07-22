//! # MOD-05 Error Types
//!
//! Error types specific to the SyntaxParser stage.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C3 §6.8: Error Conditions
//!   - MAX_TREES_EXCEEDED
//!   - SENTENCE_TOO_LONG
//!   - PARSE_FAILURE
//!   - INTERNAL_ERROR

use agos_core::error::PipelineError;

/// Errors produced by MOD-05: SyntaxParser (SPEC-0001-C3 §6.8).
#[derive(Debug, Clone)]
pub enum SyntaxError {
    /// Number of parse trees exceeds configured limit.
    MaxTreesExceeded {
        limit: usize,
        actual: usize,
    },
    /// Sentence exceeds maximum token length for full parse.
    SentenceTooLong {
        token_count: usize,
        max_length: usize,
    },
    /// No valid parse tree could be constructed for any segmentation.
    ParseFailure {
        token_id: Option<usize>,
        reason: String,
    },
    /// Unexpected internal error.
    InternalError {
        message: String,
    },
}

impl SyntaxError {
    /// Convert this SyntaxError into a PipelineError for the pipeline trait,
    /// using a human-readable stage identifier.
    pub fn into_pipeline(self, stage: &str) -> PipelineError {
        match self {
            SyntaxError::MaxTreesExceeded { limit, actual } => {
                if limit == 0 {
                    PipelineError::fatal("MAX_TREES_EXCEEDED", "max_parse_trees must be > 0", stage)
                } else {
                    PipelineError::fatal(
                        "MAX_TREES_EXCEEDED",
                        &format!("parse tree count {actual} exceeds limit {limit}"),
                        stage,
                    )
                }
            }
            SyntaxError::SentenceTooLong {
                token_count,
                max_length,
            } => PipelineError::fatal(
                "SENTENCE_TOO_LONG",
                &format!("sentence has {token_count} tokens, max is {max_length}"),
                stage,
            ),
            SyntaxError::ParseFailure { token_id, reason } => {
                let msg = match token_id {
                    Some(id) => format!("parse failed at token {id}: {reason}"),
                    None => format!("parse failed: {reason}"),
                };
                PipelineError::degraded("PARSE_FAILURE", &msg, stage)
            }
            SyntaxError::InternalError { message } => {
                PipelineError::fatal("INTERNAL_ERROR", &message, stage)
            }
        }
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxError::MaxTreesExceeded { limit, actual } => {
                write!(f, "MAX_TREES_EXCEEDED: {actual} trees exceed limit {limit}")
            }
            SyntaxError::SentenceTooLong {
                token_count,
                max_length,
            } => write!(f, "SENTENCE_TOO_LONG: {token_count} tokens, max {max_length}"),
            SyntaxError::ParseFailure { token_id, reason } => match token_id {
                Some(id) => write!(f, "PARSE_FAILURE at token {id}: {reason}"),
                None => write!(f, "PARSE_FAILURE: {reason}"),
            },
            SyntaxError::InternalError { message } => write!(f, "INTERNAL_ERROR: {message}"),
        }
    }
}
