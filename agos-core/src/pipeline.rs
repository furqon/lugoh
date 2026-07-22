//! # Pipeline Stage Trait & Orchestrator
//!
//! Defines the core `PipelineStage` trait that every AGOS pipeline module
//! implements, along with the `PipelineContext` for shared state and the
//! `PipelineOrchestrator` for composing stages into a working pipeline.
//!
//! ## Design Principles (SPEC-0001-C4 §1)
//!
//! 1. **Strict Input/Output Contracts** — Each stage defines exactly one
//!    input type and one output type.
//! 2. **Stateless by Default** — Stages are pure functions: same input →
//!    same output. State is managed externally via PipelineContext.
//! 3. **Error Handling via Result** — Every stage returns `PipelineResult<O>`.
//! 4. **Serialization Independence** — Interface types are abstract;
//!    implementations may pass data in-process or serialized.

use std::time::Instant;

use crate::error::PipelineResult;
use crate::types::{GrammarSchool, LogLevel, PipelineMode};
use crate::version::KnowledgeVersionMap;

// ──────────────────────────────────────────────
//  Pipeline Stage Trait
// ──────────────────────────────────────────────

/// A single stage in the AGOS compilation pipeline (SPEC-0001-C4 §1).
///
/// Every module in the pipeline implements this trait. Stages are:
/// - **Stateless**: no mutable state between calls
/// - **Deterministic**: same input → same output (given same context)
/// - **Single-responsibility**: one input type, one output type
///
/// # Type Parameters
///
/// * `I` — The input type consumed by this stage
/// * `O` — The output type produced by this stage
pub trait PipelineStage<I, O>: Send + Sync {
    /// The unique identifier for this stage (e.g., "MOD-01", "MOD-04").
    fn stage_id(&self) -> &'static str;

    /// Process the input and produce output, or return an error.
    ///
    /// Implementations MUST:
    /// - Be deterministic (same input → same output)
    /// - Not modify the input
    /// - Return a `PipelineError` on failure (never panic)
    /// - Respect the configured knowledge versions in `ctx`
    fn process(&self, input: I, ctx: &PipelineContext) -> PipelineResult<O>;

    /// Optional: validate that the stage is correctly configured.
    ///
    /// Override this to check e.g. that required KB files exist,
    /// configuration is valid, or dependencies are satisfied.
    /// The default implementation returns `Ok(())`.
    fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
        Ok(())
    }
}

// ──────────────────────────────────────────────
//  Pipeline Context
// ──────────────────────────────────────────────

/// Shared context passed to every pipeline stage during execution.
///
/// Contains configuration, knowledge base versions, and other cross-cutting
/// concerns. Stages read from the context but MUST NOT modify it.
#[derive(Debug, Clone)]
pub struct PipelineContext {
    /// The grammar school to use for analysis
    pub school: GrammarSchool,

    /// The pipeline execution mode
    pub mode: PipelineMode,

    /// Log level for this pipeline run
    pub log_level: LogLevel,

    /// Versions of all loaded knowledge bases
    pub knowledge_versions: KnowledgeVersionMap,

    /// Explanation language requested
    pub explanation_language: String,

    /// Whether to include evidence trail in output
    pub include_evidence: bool,

    /// Whether LLM enhancement is enabled (for explanation only)
    pub enable_llm: bool,

    /// ISO 8601 timestamp of when this pipeline execution started
    pub started_at: String,
}

impl PipelineContext {
    /// Create a new pipeline context with default values.
    pub fn new(school: GrammarSchool) -> Self {
        Self {
            school,
            mode: PipelineMode::Full,
            log_level: LogLevel::Info,
            knowledge_versions: KnowledgeVersionMap::new(),
            explanation_language: "en".to_string(),
            include_evidence: false,
            enable_llm: false,
            started_at: now_iso8601(),
        }
    }

    /// Set the knowledge version map from a loaded KB suite.
    pub fn with_knowledge_versions(mut self, versions: KnowledgeVersionMap) -> Self {
        self.knowledge_versions = versions;
        self
    }

    /// Set the pipeline mode.
    pub fn with_mode(mut self, mode: PipelineMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the explanation language.
    pub fn with_explanation_language(mut self, lang: impl Into<String>) -> Self {
        self.explanation_language = lang.into();
        self
    }
}

// ──────────────────────────────────────────────
//  Timestamp Utility
// ──────────────────────────────────────────────

/// Returns an ISO 8601 UTC timestamp (e.g., `2026-07-21T12:34:56Z`).
///
/// Uses Howard Hinnant's civil date algorithm for calendar date conversion,
/// avoiding a dependency on the `chrono` crate.
pub(crate) fn now_iso8601() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();

    let days = total_secs / 86400;
    let time_secs = total_secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Civil date from days since epoch (Howard Hinnant's algorithm)
    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

// ──────────────────────────────────────────────
//  Pipeline Orchestrator
// ──────────────────────────────────────────────

/// A pipeline orchestrator that composes stages sequentially.
///
/// Handles:
/// - Single-stage execution with timing
/// - Two-stage chaining (output of first → input of second)
/// - Per-stage `validate_config` calls
///
/// ## Spec Alignment
///
/// - SPEC-0001-C4 §17: Pipeline Orchestrator Interface
/// - SPEC-0001-C2 §6: Module Interaction Patterns
#[derive(Default)]
pub struct PipelineOrchestrator {
    /// Per-stage timing from the last execution (stage_id → ms)
    pub last_timing: std::collections::HashMap<String, f64>,
}

impl PipelineOrchestrator {
    pub fn new() -> Self {
        Self {
            last_timing: std::collections::HashMap::new(),
        }
    }

    /// Execute a single stage with timing instrumentation.
    ///
    /// Returns the stage's output. Records execution duration in `last_timing`.
    pub fn run_stage<I, O>(
        &mut self,
        stage: &dyn PipelineStage<I, O>,
        input: I,
        ctx: &PipelineContext,
    ) -> PipelineResult<O> {
        let start = Instant::now();
        let output = stage.process(input, ctx)?;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        self.last_timing
            .insert(stage.stage_id().to_string(), duration_ms);
        Ok(output)
    }

    /// Chain two stages: feed the output of the first as input to the second.
    ///
    /// Supports stages with different I/O types:
    /// `Stage1<I, M>` → `Stage2<M, O>`
    pub fn chain<I, M, O>(
        &mut self,
        first: &dyn PipelineStage<I, M>,
        second: &dyn PipelineStage<M, O>,
        input: I,
        ctx: &PipelineContext,
    ) -> PipelineResult<O> {
        let intermediate = self.run_stage(first, input, ctx)?;
        self.run_stage(second, intermediate, ctx)
    }

    /// Validate a single stage's configuration.
    ///
    /// Calls `validate_config` on the stage, which checks e.g. that
    /// required knowledge bases are available or configuration is valid.
    pub fn validate_stage<I, O>(
        &self,
        stage: &dyn PipelineStage<I, O>,
        ctx: &PipelineContext,
    ) -> PipelineResult<()> {
        stage.validate_config(ctx)
    }

    /// Get recorded timing for a specific stage, if available.
    pub fn stage_timing(&self, stage_id: &str) -> Option<f64> {
        self.last_timing.get(stage_id).copied()
    }

    /// Clear accumulated timing data.
    pub fn reset_timing(&mut self) {
        self.last_timing.clear();
    }
}

// ──────────────────────────────────────────────
//  Tests
// ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_iso8601_format() {
        let ts = now_iso8601();
        // Basic format check: YYYY-MM-DDTHH:MM:SSZ
        assert_eq!(ts.len(), 20, "Expected ISO 8601 length 20, got: {ts}");
        assert!(ts.ends_with('Z'), "Expected Z suffix, got: {ts}");
        assert_eq!(&ts[4..5], "-", "Expected - at position 4");
        assert_eq!(&ts[7..8], "-", "Expected - at position 7");
        assert_eq!(&ts[10..11], "T", "Expected T at position 10");
        assert_eq!(&ts[13..14], ":", "Expected : at position 13");
        assert_eq!(&ts[16..17], ":", "Expected : at position 16");
        // Verify year is plausible (2026 or later)
        let year: i32 = ts[0..4].parse().unwrap();
        assert!(year >= 2026, "Year should be >= 2026, got {year}");
    }

    // Dummy stage for orchestrator testing
    struct TestStage;

    impl PipelineStage<String, String> for TestStage {
        fn stage_id(&self) -> &'static str {
            "MOD-TEST"
        }

        fn process(&self, input: String, _ctx: &PipelineContext) -> PipelineResult<String> {
            Ok(format!("processed: {input}"))
        }
    }

    #[test]
    fn test_orchestrator_run_stage() {
        let mut orch = PipelineOrchestrator::new();
        let ctx = PipelineContext::new(GrammarSchool::Basra);
        let stage = TestStage;

        let result = orch.run_stage(&stage, "hello".to_string(), &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "processed: hello");
        assert!(orch.stage_timing("MOD-TEST").is_some());
    }

    #[test]
    fn test_orchestrator_chain() {
        struct UpperCase;
        impl PipelineStage<String, String> for UpperCase {
            fn stage_id(&self) -> &'static str { "MOD-UPPER" }
            fn process(&self, input: String, _ctx: &PipelineContext) -> PipelineResult<String> {
                Ok(input.to_uppercase())
            }
        }

        struct Exclaim;
        impl PipelineStage<String, String> for Exclaim {
            fn stage_id(&self) -> &'static str { "MOD-EXCLAIM" }
            fn process(&self, input: String, _ctx: &PipelineContext) -> PipelineResult<String> {
                Ok(format!("{input}!"))
            }
        }

        let mut orch = PipelineOrchestrator::new();
        let ctx = PipelineContext::new(GrammarSchool::Basra);

        let result = orch.chain(&UpperCase, &Exclaim, "hello".to_string(), &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HELLO!");
    }

    #[test]
    fn test_orchestrator_validate() {
        struct StrictStage;
        impl PipelineStage<String, String> for StrictStage {
            fn stage_id(&self) -> &'static str { "MOD-STRICT" }
            fn process(&self, input: String, _ctx: &PipelineContext) -> PipelineResult<String> {
                Ok(input)
            }
            fn validate_config(&self, _ctx: &PipelineContext) -> PipelineResult<()> {
                Err(crate::error::PipelineError::fatal(
                    crate::error::codes::INVALID_REQUEST,
                    "validation failed",
                    "MOD-STRICT",
                ))
            }
        }

        let orch = PipelineOrchestrator::new();
        let ctx = PipelineContext::new(GrammarSchool::Basra);
        let stage = StrictStage;

        let result = orch.validate_stage(&stage, &ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "INVALID_REQUEST");
    }
}
