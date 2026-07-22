//! # Logging & Tracing Infrastructure
//!
//! Provides structured logging and tracing support for the AGOS pipeline.
//! Each pipeline stage creates a tracing span, allowing per-stage log
//! filtering and timing collection.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Requires `use agos_core::tracing::{init_tracing, PipelineSpan};`
//! // and `use agos_core::types::LogLevel;` in scope.
//! // The subscriber can only be initialized once globally.
//!
//! use agos_core::tracing::{init_tracing, PipelineSpan};
//! use agos_core::types::LogLevel;
//!
//! // Initialize once at application startup
//! init_tracing(LogLevel::Info, false).unwrap();
//!
//! // In each stage:
//! let span = PipelineSpan::new("MOD-04", Some("input text hash..."));
//! let _guard = span.enter();
//! tracing::info!("starting morphological analysis");
//! ```
//!
//! ## Spec Alignment
//!
//! - SPEC-0001-C6 §4: Logging & Observability
//! - SPEC-0001-C8 §6.2.1: Audit Trails

use crate::types::LogLevel;

/// Initialize the global tracing subscriber.
///
/// Call this once at application startup, before any pipeline execution.
/// In JSON mode, logs are emitted as structured JSON lines suitable for
/// ingestion by log aggregators (Loki, DataDog, etc.).
pub fn init_tracing(level: LogLevel, json: bool) -> Result<(), TracingError> {
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive(match level {
            LogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG.into(),
            LogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO.into(),
            LogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN.into(),
            LogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR.into(),
        });

    if json {
        let fmt = tracing_subscriber::fmt()
            .json()
            .with_target(true)
            .with_line_number(true)
            .with_file(true)
            .with_env_filter(filter)
            .finish();
        tracing::subscriber::set_global_default(fmt)
            .map_err(|e| TracingError::InitFailed(e.to_string()))?;
    } else {
        let fmt = tracing_subscriber::fmt()
            .with_target(true)
            .with_line_number(true)
            .with_env_filter(filter)
            .finish();
        tracing::subscriber::set_global_default(fmt)
            .map_err(|e| TracingError::InitFailed(e.to_string()))?;
    }

    Ok(())
}

/// A tracing span for a single pipeline stage execution.
///
/// Each stage creates one span, which tracks:
/// - `stage_id`: the module identifier (e.g., "MOD-04")
/// - `input_hash`: optional hash of the input for correlation
///
/// Spans are entered for the duration of stage execution and exited
/// automatically via RAII (the `_guard` is dropped when the scope exits).
pub struct PipelineSpan {
    span: tracing::Span,
}

impl PipelineSpan {
    /// Create a new span for a pipeline stage.
    pub fn new(stage_id: &'static str, input_hash: Option<&str>) -> Self {
        let span = tracing::info_span!(
            "pipeline_stage",
            stage_id = stage_id,
            input_hash = input_hash.unwrap_or("unknown"),
        );
        Self { span }
    }

    /// Enter the span, returning a guard that exits on drop.
    pub fn enter(&self) -> tracing::span::Entered<'_> {
        self.span.enter()
    }

    /// Record a field on the span after creation.
    pub fn record(&self, key: &str, value: &dyn tracing::field::Value) {
        self.span.record(key, value);
    }
}

/// Errors that can occur during tracing initialization.
#[derive(Debug)]
pub enum TracingError {
    /// The global subscriber was already set.
    InitFailed(String),
}

impl std::fmt::Display for TracingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TracingError::InitFailed(msg) => write!(f, "Tracing init failed: {msg}"),
        }
    }
}

impl std::error::Error for TracingError {}

/// Convenience macro for logging with stage context.
///
/// Usage:
/// ```rust,ignore
/// // The macro is in scope when you're using `use agos_core::agos_log;`
/// agos_log!(info, "MOD-04", "root extraction complete: found {} candidates", count);
/// ```
#[macro_export]
macro_rules! agos_log {
    ($level:ident, $stage:expr, $($arg:tt)+) => {
        tracing::$level!(
            target: $stage,
            $($arg)+
        )
    };
}

/// Initialize a simple tracing subscriber that logs to stdout.
///
/// This is a convenience wrapper that:
/// 1. Creates an env-filter from `AGOS_LOG` env var or defaults to INFO
/// 2. Sets up console output with human-readable formatting
///
/// ```rust,ignore
/// // Requires the tracing subscriber to not already be set
/// agos_core::tracing::init_default_tracing().unwrap();
/// ```
pub fn init_default_tracing() -> Result<(), TracingError> {
    let level = match std::env::var("AGOS_LOG").unwrap_or_default().to_lowercase().as_str() {
        "debug" | "d" => LogLevel::Debug,
        "warn" | "w" => LogLevel::Warn,
        "error" | "e" => LogLevel::Error,
        _ => LogLevel::Info,
    };
    let json = std::env::var("AGOS_LOG_JSON").is_ok();
    init_tracing(level, json)
}


