//! # AGOS Core — Shared Foundation Types
//!
//! This crate defines the foundational types for the AGOS (Arabic Grammar Operating
//! System) platform. Every other crate in the workspace depends on `agos-core` for:
//!
//! - **Error types** (`PipelineError`, error codes) — consistent error handling
//! - **Common types** (`PartOfSpeech`, `SyntacticRole`, `TokenType`, etc.) — shared vocabulary
//! - **Intermediate Representations** (IR-1 through IR-11) — pipeline stage I/O types
//! - **Evidence trail** (`EvidenceEntry`, `EvidenceTrail`) — explainability infrastructure
//! - **Feature bitfield** (`FeatureBitfield`, `FeatureMap`) — 64-bit feature encoding (KB-0007)
//! - **Versioning** (`SemVer`, `KnowledgeVersionMap`) — version tracking
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C4: Module Responsibilities & Interfaces (error types, common types)
//! - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-1 through IR-11)
//! - SPEC-0102 / KB-0007: Morphological Features (feature bitfield, feature taxonomy)
//! - RFC-0002: Grammar Bytecode Format (feature bitfield layout)

pub mod error;
pub mod evidence;
pub mod feature;
pub mod ir;
pub mod pipeline;
pub mod tracing;
pub mod types;
pub mod version;

pub use error::*;
pub use evidence::*;
pub use feature::*;
pub use ir::*;
pub use pipeline::*;
pub use tracing::*;
pub use types::*;
pub use version::*;
