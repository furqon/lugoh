//! # Evidence Trail
//!
//! Types for the complete, immutable record of every decision made during
//! the analysis of a text. The evidence trail satisfies Core Principles 3
//! (Explainability by Design) and 12 (Evidence Trail Completeness).
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C4 §2.3: EvidenceEntry type
//! - SPEC-0001-C5 §13: Evidence Trail Data Model

use serde::{Deserialize, Serialize};

use crate::version::KnowledgeVersionMap;

/// A single entry in the evidence trail (SPEC-0001-C4 §2.3).
///
/// Records one atomic decision made by a pipeline stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    /// Unique entry ID
    pub id: String,

    /// ISO 8601 timestamp of when this decision was made
    pub timestamp: String,

    /// Stage that produced this evidence (e.g., "MOD-04")
    pub stage: String,

    /// Iteration number if the stage ran multiple passes
    pub stage_iteration: u32,

    /// Category of evidence
    pub category: EvidenceCategory,

    /// Rule ID or algorithm name that produced this evidence
    pub rule_or_algorithm: String,

    /// Version of the rule or algorithm
    pub version: String,

    /// Human-readable description of the input state
    pub input_description: String,

    /// Hash of the input state
    pub input_state_hash: String,

    /// Human-readable description of what was decided
    pub output_description: String,

    /// What changed as a result (e.g., "confirmed analysis 2")
    pub output_delta: String,

    /// Confidence in this decision (0.0 to 1.0)
    pub confidence: f64,

    /// Indices of affected tokens
    pub token_indices: Vec<usize>,
}

/// Category of evidence (SPEC-0001-C5 §13.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceCategory {
    Segmentation,
    Morphology,
    Syntax,
    RuleApplication,
    KnowledgeResolution,
    Bytecode,
    Execution,
}

/// The complete evidence trail for a single analysis (SPEC-0001-C5 §13.2).
///
/// Accumulated incrementally across the pipeline: each stage appends entries
/// as decisions are made.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTrail {
    /// Schema identifier
    pub spec: String,

    /// Version of the evidence trail schema
    pub version: String,

    /// Pipeline-level metadata
    pub pipeline: EvidencePipelineMeta,

    /// All evidence entries in chronological order
    pub entries: Vec<EvidenceEntry>,

    /// Summary statistics
    pub summary: EvidenceSummary,
}

/// Pipeline metadata attached to the evidence trail (SPEC-0001-C5 §13.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePipelineMeta {
    /// ISO 8601 timestamp of pipeline start
    pub started_at: String,

    /// ISO 8601 timestamp of pipeline completion
    pub completed_at: String,

    /// AGOS platform version
    pub pipeline_version: String,

    /// Grammar school used
    pub school: String,

    /// Versions of all KBs used
    pub knowledge_versions: KnowledgeVersionMap,
}

/// Summary statistics for an evidence trail (SPEC-0001-C5 §13.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSummary {
    /// Total number of evidence entries
    pub total_entries: u64,

    /// Stages that contributed evidence
    pub stages_involved: Vec<String>,

    /// Count of rule applications
    pub rules_applied: u64,

    /// Count of ambiguities resolved
    pub ambiguities_resolved: u64,

    /// Count of grammatical flags raised
    pub flags_raised: u64,
}
