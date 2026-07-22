//! # Pipeline Error Types
//!
//! Defines the standard error types used across all AGOS pipeline modules.
//! Every module returns errors that conform to the `PipelineError` envelope,
//! ensuring consistent error handling throughout the platform.
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C4 §2.2: PipelineError envelope
//! - SPEC-0001-C8: Security, Validation & Error Handling

use serde::{Deserialize, Serialize};

/// The standard error envelope for all AGOS pipeline modules.
///
/// Every module returns either a success value or a structured `PipelineError`.
/// Errors are never returned as exceptions, panics, or side effects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineError {
    /// Machine-readable error code (e.g., "INVALID_ENCODING", "KB_MISSING")
    pub code: String,

    /// Human-readable description of what went wrong
    pub message: String,

    /// Stage ID where the error originated (e.g., "MOD-01", "MOD-04")
    pub stage: String,

    /// Whether the pipeline should stop (true) or can continue in degraded mode (false)
    pub is_fatal: bool,

    /// Optional suggestion for recovering from the error
    pub recovery_hint: Option<String>,

    /// Optional wrapped inner error (for debugging)
    pub inner: Option<String>,
}

impl PipelineError {
    /// Create a new fatal pipeline error.
    pub fn fatal(code: impl Into<String>, message: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            stage: stage.into(),
            is_fatal: true,
            recovery_hint: None,
            inner: None,
        }
    }

    /// Create a new non-fatal (degraded) pipeline error.
    pub fn degraded(code: impl Into<String>, message: impl Into<String>, stage: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            stage: stage.into(),
            is_fatal: false,
            recovery_hint: None,
            inner: None,
        }
    }

    /// Add a recovery hint to the error.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hint = Some(hint.into());
        self
    }

    /// Add inner error context.
    pub fn with_inner(mut self, inner: impl Into<String>) -> Self {
        self.inner = Some(inner.into());
        self
    }
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.stage, self.code, self.message)
    }
}

impl std::error::Error for PipelineError {}

/// A specialized `Result` type for AGOS pipeline operations.
pub type PipelineResult<T> = Result<T, PipelineError>;

/// Pre-defined error codes for AGOS validation errors (SPEC-0001-C8).
pub mod codes {
    // --- MOD-01: UnicodeValidation ---
    pub const INVALID_ENCODING: &str = "INVALID_ENCODING";
    pub const EMPTY_INPUT: &str = "EMPTY_INPUT";
    pub const MAX_LENGTH_EXCEEDED: &str = "MAX_LENGTH_EXCEEDED";
    pub const NON_ARABIC_CHAR: &str = "NON_ARABIC_CHAR";
    pub const UNSUPPORTED_CHAR: &str = "UNSUPPORTED_CHAR";

    // --- MOD-03: Tokenizer ---
    pub const MAX_SEGMENTATIONS_EXCEEDED: &str = "MAX_SEGMENTATIONS_EXCEEDED";

    // --- MOD-04: MorphologicalParser ---
    pub const MAX_ANALYSES_EXCEEDED: &str = "MAX_ANALYSES_EXCEEDED";
    pub const KB_MISSING: &str = "KB_MISSING";
    pub const KB_VERSION_MISMATCH: &str = "KB_VERSION_MISMATCH";

    // --- MOD-05: SyntaxParser ---
    pub const MAX_TREES_EXCEEDED: &str = "MAX_TREES_EXCEEDED";
    pub const SENTENCE_TOO_LONG: &str = "SENTENCE_TOO_LONG";
    pub const PARSE_FAILURE: &str = "PARSE_FAILURE";

    // --- MOD-06: GIRConstructor ---
    pub const TOKEN_MISMATCH: &str = "TOKEN_MISMATCH";
    pub const GIR_VERSION_INCOMPATIBLE: &str = "GIR_VERSION_INCOMPATIBLE";

    // --- MOD-07: RuleEngine ---
    pub const RULE_SET_NOT_FOUND: &str = "RULE_SET_NOT_FOUND";
    pub const RULE_VERSION_MISMATCH: &str = "RULE_VERSION_MISMATCH";
    pub const RULE_APPLICATION_LIMIT: &str = "RULE_APPLICATION_LIMIT";
    pub const RULE_CONFLICT: &str = "RULE_CONFLICT";

    // --- MOD-08: KnowledgeGraphResolver ---
    pub const KB_LOAD_FAILURE: &str = "KB_LOAD_FAILURE";

    // --- MOD-09: BytecodeGenerator ---
    pub const BYTECODE_VERSION_UNSUPPORTED: &str = "BYTECODE_VERSION_UNSUPPORTED";
    pub const GIR_VALIDATION_FAILED: &str = "GIR_VALIDATION_FAILED";
    pub const BYTECODE_TOO_LARGE: &str = "BYTECODE_TOO_LARGE";

    // --- MOD-10: GVM ---
    pub const UNSUPPORTED_BYTECODE_VERSION: &str = "UNSUPPORTED_BYTECODE_VERSION";
    pub const BYTECODE_CORRUPTED: &str = "BYTECODE_CORRUPTED";
    pub const MAX_STEPS_EXCEEDED: &str = "MAX_STEPS_EXCEEDED";
    pub const MAX_MEMORY_EXCEEDED: &str = "MAX_MEMORY_EXCEEDED";
    pub const EXECUTION_FAILURE: &str = "EXECUTION_FAILURE";

    // --- MOD-11: ExplanationEngine ---
    pub const UNSUPPORTED_LANGUAGE: &str = "UNSUPPORTED_LANGUAGE";
    pub const UNSUPPORTED_FORMAT: &str = "UNSUPPORTED_FORMAT";
    pub const LLM_SERVICE_UNAVAILABLE: &str = "LLM_SERVICE_UNAVAILABLE";

    // --- MOD-12: PluginLoader ---
    pub const PLUGIN_NOT_FOUND: &str = "PLUGIN_NOT_FOUND";
    pub const PLUGIN_INVALID_MANIFEST: &str = "PLUGIN_INVALID_MANIFEST";
    pub const PLUGIN_VERSION_MISMATCH: &str = "PLUGIN_VERSION_MISMATCH";
    pub const PLUGIN_DEPENDENCY_MISSING: &str = "PLUGIN_DEPENDENCY_MISSING";
    pub const PLUGIN_LOAD_FAILED: &str = "PLUGIN_LOAD_FAILED";
    pub const PLUGIN_SANDBOX_VIOLATION: &str = "PLUGIN_SANDBOX_VIOLATION";

    // --- MOD-13: CacheManager ---
    pub const BACKEND_UNAVAILABLE: &str = "BACKEND_UNAVAILABLE";
    pub const SERIALIZATION_FAILED: &str = "SERIALIZATION_FAILED";
    pub const STORAGE_FULL: &str = "STORAGE_FULL";

    // --- MOD-14: APIGateway ---
    pub const INVALID_REQUEST: &str = "INVALID_REQUEST";
    pub const UNSUPPORTED_SCHOOL: &str = "UNSUPPORTED_SCHOOL";
    pub const RATE_LIMITED: &str = "RATE_LIMITED";
    pub const SERVICE_UNAVAILABLE: &str = "SERVICE_UNAVAILABLE";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
}

