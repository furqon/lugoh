//! # Knowledge Base Error Types
//!
//! Error types specific to knowledge base loading, compilation, and lookup operations.
//!
//! ## Spec Alignment
//!
//! - KB-OVERVIEW: KB Suite Overview & Architecture
//! - SPEC-0001-C8: Security, Validation & Error Handling

use agos_core::error::PipelineError;
use thiserror::Error;

/// KB-specific error type wrapping the standard PipelineError.
#[derive(Error, Debug)]
pub enum KbError {
    /// The KB file or source was not found at the expected path.
    #[error("KB not found: {0}")]
    NotFound(String),

    /// The KB file is malformed or failed validation.
    #[error("KB format error: {0}")]
    FormatError(String),

    /// The KB version is incompatible with the current platform.
    #[error("KB version mismatch: {0}")]
    VersionMismatch(String),

    /// A required dependency KB is missing or incompatible.
    #[error("KB dependency error: {0}")]
    DependencyError(String),

    /// I/O error while reading KB data.
    #[error("KB I/O error: {0}")]
    IoError(String),

    /// Memory-mapping error.
    #[error("KB mmap error: {0}")]
    MmapError(String),

    /// Deserialization error.
    #[error("KB deserialization error: {0}")]
    DeserializationError(String),

    /// The KB is loaded but the requested entry does not exist.
    #[error("KB entry not found: {0}")]
    EntryNotFound(String),

    /// General internal error.
    #[error("KB internal error: {0}")]
    Internal(String),
}

impl KbError {
    /// Convert this KB error into a PipelineError for the given stage.
    pub fn into_pipeline_error(self, stage: &str) -> PipelineError {
        let (code, msg) = match &self {
            KbError::NotFound(s) => (agos_core::error::codes::KB_MISSING, format!("KB not found: {s}")),
            KbError::FormatError(s) => (agos_core::error::codes::KB_LOAD_FAILURE, format!("KB format error: {s}")),
            KbError::VersionMismatch(s) => {
                (agos_core::error::codes::KB_VERSION_MISMATCH, format!("KB version mismatch: {s}"))
            }
            KbError::DependencyError(s) => (agos_core::error::codes::KB_LOAD_FAILURE, format!("KB dependency: {s}")),
            KbError::IoError(s) => (agos_core::error::codes::KB_LOAD_FAILURE, format!("KB I/O: {s}")),
            KbError::MmapError(s) => (agos_core::error::codes::KB_LOAD_FAILURE, format!("KB mmap: {s}")),
            KbError::DeserializationError(s) => {
                (agos_core::error::codes::KB_LOAD_FAILURE, format!("KB deserialization: {s}"))
            }
            KbError::EntryNotFound(s) => (agos_core::error::codes::KB_MISSING, format!("KB entry not found: {s}")),
            KbError::Internal(s) => (agos_core::error::codes::KB_LOAD_FAILURE, format!("KB internal: {s}")),
        };
        PipelineError::fatal(code, msg, stage)
    }
}

/// Specialized Result type for KB operations.
pub type KbResult<T> = Result<T, KbError>;
