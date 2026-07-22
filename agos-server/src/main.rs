//! # AGOS Web API Server
//!
//! Axum-based HTTP server that exposes the AGOS compilation pipeline as a
//! RESTful web API. Accepts Arabic text via `POST /analyze` and returns
//! full morphological and syntactic analysis as JSON.
//!
//! ## Endpoints
//!
//! - `GET /health` — Health check, returns KB-0004 stats
//! - `POST /analyze` — Analyze Arabic text through the 5-stage pipeline
//!
//! ## Pipeline Flow
//!
//! ```text
//! Input (Arabic text)
//!     → MOD-01: UnicodeValidator → NormalizedText (IR-1)
//!     → MOD-02: Lexer → TokenStream (IR-2)
//!     → MOD-03: Tokenizer → SegmentedTokenStream (IR-3)
//!     → MOD-04: MorphologicalParser → MorphologicalAnalysis (IR-4)
//!     → MOD-05: SyntaxParser → SyntaxTree (IR-5)
//! ```
//!
//! ## Usage
//!
//! ```bash
//! curl -X POST localhost:3000/analyze \
//!   -H "Content-Type: application/json" \
//!   -d '{"text": "السَّلَامُ عَلَيْكُمْ", "school": "Basra"}'
//! ```

use std::sync::Arc;

use axum::{
    extract::State,
    http::Method,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use agos_core::ir::{
    MorphologicalAnalysis, NormalizedText, SegmentedTokenStream, SyntaxTree, TokenStream,
};
use agos_core::pipeline::{PipelineContext, PipelineOrchestrator};
use agos_core::types::GrammarSchool;

use agos_kb::Kb0004;
use agos_morph::{Lexer, MorphologicalParser, MorphologicalParserConfig, Tokenizer, UnicodeValidator};
use agos_syntax::SyntaxParser;

// ──────────────────────────────────────────────
//  Application State
// ──────────────────────────────────────────────

/// Shared application state held by all route handlers.
struct AppState {
    /// Pre-configured pipeline stages (stateless, shared via Arc)
    validator: UnicodeValidator,
    lexer: Lexer,
    tokenizer: Tokenizer,
    morph_parser: MorphologicalParser,
    syntax_parser: SyntaxParser,
    /// Optional KB-0004 stats for health endpoint
    kb_stats: Option<KbStats>,
}

/// KB-0004 statistics exposed via the health endpoint.
#[derive(Debug, Clone, Serialize)]
struct KbStats {
    stem_overrides: usize,
    verb_profiles: usize,
    noun_profiles: usize,
}

// ──────────────────────────────────────────────
//  Request / Response Types
// ──────────────────────────────────────────────

/// Request body for POST /analyze.
#[derive(Debug, Deserialize)]
struct AnalyzeRequest {
    /// Arabic text to analyze
    text: String,
    /// Grammar school (default: Basra)
    #[serde(default)]
    school: SchoolParam,
    /// Whether to strip tashkeel (diacritics) before analysis (default: false)
    #[serde(default)]
    strip_tashkeel: bool,
    /// Whether to strip tatweel (kashida) before analysis (default: true)
    #[serde(default = "default_true")]
    strip_tatweel: bool,
}

fn default_true() -> bool {
    true
}

/// Grammar school parameter from the API.
#[derive(Debug, Default, Deserialize)]
enum SchoolParam {
    #[default]
    Basra,
    Kufa,
    Andalus,
    Baghdadi,
    Modern,
}

impl From<SchoolParam> for GrammarSchool {
    fn from(s: SchoolParam) -> Self {
        match s {
            SchoolParam::Basra => GrammarSchool::Basra,
            SchoolParam::Kufa => GrammarSchool::Kufa,
            SchoolParam::Andalus => GrammarSchool::Andalus,
            SchoolParam::Baghdadi => GrammarSchool::Baghdad,
            SchoolParam::Modern => GrammarSchool::Modern,
        }
    }
}

/// Response body for POST /analyze.
#[derive(Debug, Serialize)]
struct AnalyzeResponse {
    /// Whether the pipeline succeeded
    success: bool,
    /// Error message if failed
    error: Option<String>,
    /// Per-stage timing in milliseconds
    timing_ms: std::collections::HashMap<String, f64>,
    /// Stage-by-stage outputs
    stages: StageOutputs,
}

/// Per-stage outputs from the pipeline.
#[derive(Debug, Serialize)]
struct StageOutputs {
    /// MOD-01: Normalized text metadata
    normalized: Option<NormalizedOutput>,
    /// MOD-02: Token stream metadata
    tokens: Option<TokenOutput>,
    /// MOD-03: Segmentation metadata
    segmented: Option<SegmentedOutput>,
    /// MOD-04: Full morphological analysis (all tokens)
    morphology: Option<MorphologicalAnalysis>,
    /// MOD-05: Full syntax tree
    syntax: Option<SyntaxTree>,
}

/// MOD-01 output summary.
#[derive(Debug, Serialize)]
struct NormalizedOutput {
    normalized_text: String,
    char_count: u64,
    word_count_estimate: u64,
    has_tashkeel: bool,
    has_tatweel: bool,
    has_non_arabic: bool,
}

/// MOD-02 output summary.
#[derive(Debug, Serialize)]
struct TokenOutput {
    token_count: u64,
    word_count: u64,
    tokens: Vec<RawTokenSummary>,
}

/// Summary of a single token.
#[derive(Debug, Serialize)]
struct RawTokenSummary {
    id: usize,
    text: String,
    token_type: String,
}

/// MOD-03 output summary.
#[derive(Debug, Serialize)]
struct SegmentedOutput {
    total_tokens: u64,
    segmentable_tokens: u64,
    ambiguous_tokens: u64,
}

// ──────────────────────────────────────────────
//  Health Check
// ──────────────────────────────────────────────

/// GET /health — returns server status and KB-0004 statistics.
async fn health_handler(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let mut info = serde_json::json!({
        "status": "ok",
        "service": "agos-server",
        "pipeline_stages": ["MOD-01", "MOD-02", "MOD-03", "MOD-04", "MOD-05"],
        "kb_loaded": state.kb_stats.is_some(),
    });

    if let Some(ref kb) = state.kb_stats {
        info["kb_stats"] = serde_json::json!({
            "stem_overrides": kb.stem_overrides,
            "verb_profiles": kb.verb_profiles,
            "noun_profiles": kb.noun_profiles,
        });
    }

    Json(info)
}

// ──────────────────────────────────────────────
//  Analyze Endpoint
// ──────────────────────────────────────────────

/// POST /analyze — runs the full 5-stage pipeline on Arabic text.
async fn analyze_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnalyzeRequest>,
) -> Json<AnalyzeResponse> {
    if req.text.trim().is_empty() {
        return Json(AnalyzeResponse {
            success: false,
            error: Some("Empty input text".to_string()),
            timing_ms: std::collections::HashMap::new(),
            stages: StageOutputs {
                normalized: None,
                tokens: None,
                segmented: None,
                morphology: None,
                syntax: None,
            },
        });
    }

    // Build pipeline context
    let ctx = PipelineContext::new(req.school.into());

    // Configure UnicodeValidator based on request params
    let validator = if req.strip_tashkeel || !req.strip_tatweel {
        let mut config = agos_morph::config::UnicodeValidatorConfig::default();
        config.normalize_tashkeel = req.strip_tashkeel;
        config.strip_tatweel = req.strip_tatweel;
        UnicodeValidator::new(config)
    } else {
        state.validator.clone()
    };

    let lexer = state.lexer.clone();
    let tokenizer = state.tokenizer.clone();
    let morph_parser = state.morph_parser.clone();
    let syntax_parser = state.syntax_parser.clone();
    let mut orch = PipelineOrchestrator::new();

    // ── Stage 1: MOD-01 UnicodeValidator ──
    let normalized: NormalizedText = match orch.run_stage(&validator, req.text.clone(), &ctx) {
        Ok(n) => n,
        Err(e) => {
            return Json(AnalyzeResponse {
                success: false,
                error: Some(format!("MOD-01 failed: {e}")),
                timing_ms: orch.last_timing,
                stages: StageOutputs {
                    normalized: None,
                    tokens: None,
                    segmented: None,
                    morphology: None,
                    syntax: None,
                },
            });
        }
    };

    let normalized_summary = NormalizedOutput {
        normalized_text: normalized.normalized_text.clone(),
        char_count: normalized.metadata.char_count,
        word_count_estimate: normalized.metadata.word_count_estimate,
        has_tashkeel: normalized.metadata.has_tashkeel,
        has_tatweel: normalized.metadata.has_tatweel,
        has_non_arabic: normalized.metadata.has_non_arabic,
    };

    // ── Stage 2: MOD-02 Lexer ──
    let tokens: TokenStream = match orch.run_stage(&lexer, normalized.clone(), &ctx) {
        Ok(t) => t,
        Err(e) => {
            return Json(AnalyzeResponse {
                success: false,
                error: Some(format!("MOD-02 failed: {e}")),
                timing_ms: orch.last_timing,
                stages: StageOutputs {
                    normalized: Some(normalized_summary),
                    tokens: None,
                    segmented: None,
                    morphology: None,
                    syntax: None,
                },
            });
        }
    };

    let token_summary = TokenOutput {
        token_count: tokens.metadata.token_count,
        word_count: tokens.metadata.word_count,
        tokens: tokens
            .tokens
            .iter()
            .map(|t| RawTokenSummary {
                id: t.id,
                text: t.text.clone(),
                token_type: format!("{:?}", t.token_type),
            })
            .collect(),
    };

    // ── Stage 3: MOD-03 Tokenizer ──
    let segmented: SegmentedTokenStream = match orch.run_stage(&tokenizer, tokens.clone(), &ctx) {
        Ok(s) => s,
        Err(e) => {
            return Json(AnalyzeResponse {
                success: false,
                error: Some(format!("MOD-03 failed: {e}")),
                timing_ms: orch.last_timing,
                stages: StageOutputs {
                    normalized: Some(normalized_summary),
                    tokens: Some(token_summary),
                    segmented: None,
                    morphology: None,
                    syntax: None,
                },
            });
        }
    };

    let segmented_summary = SegmentedOutput {
        total_tokens: segmented.metadata.total_tokens,
        segmentable_tokens: segmented.metadata.segmentable_tokens,
        ambiguous_tokens: segmented.metadata.ambiguous_tokens,
    };

    // ── Stage 4: MOD-04 MorphologicalParser ──
    let morph: MorphologicalAnalysis = match orch.run_stage(&morph_parser, segmented.clone(), &ctx) {
        Ok(m) => m,
        Err(e) => {
            return Json(AnalyzeResponse {
                success: false,
                error: Some(format!("MOD-04 failed: {e}")),
                timing_ms: orch.last_timing,
                stages: StageOutputs {
                    normalized: Some(normalized_summary),
                    tokens: Some(token_summary),
                    segmented: Some(segmented_summary),
                    morphology: None,
                    syntax: None,
                },
            });
        }
    };

    // ── Stage 5: MOD-05 SyntaxParser ──
    let syntax: SyntaxTree = match orch.run_stage(&syntax_parser, morph.clone(), &ctx) {
        Ok(s) => s,
        Err(e) => {
            return Json(AnalyzeResponse {
                success: false,
                error: Some(format!("MOD-05 failed: {e}")),
                timing_ms: orch.last_timing,
                stages: StageOutputs {
                    normalized: Some(normalized_summary),
                    tokens: Some(token_summary),
                    segmented: Some(segmented_summary),
                    morphology: Some(morph),
                    syntax: None,
                },
            });
        }
    };

    // ── Success ──
    Json(AnalyzeResponse {
        success: true,
        error: None,
        timing_ms: orch.last_timing,
        stages: StageOutputs {
            normalized: Some(normalized_summary),
            tokens: Some(token_summary),
            segmented: Some(segmented_summary),
            morphology: Some(morph),
            syntax: Some(syntax),
        },
    })
}

// ──────────────────────────────────────────────
//  Application Setup
// ──────────────────────────────────────────────

/// Build the shared application state, auto-loading KB-0004 if available.
fn build_app_state() -> AppState {
    tracing::info!("Initializing AGOS server...");

    // Try to auto-load KB-0004 from the knowledge directory
    let kb_path = std::path::Path::new("knowledge/KB-0004");
    let (morph_parser, kb_stats) = if kb_path.exists() {
        match Kb0004::load_from_directory(kb_path) {
            Ok(kb) => {
                let stats = KbStats {
                    stem_overrides: kb.stem_override_count(),
                    verb_profiles: kb.verb_profile_count(),
                    noun_profiles: kb.noun_profile_count(),
                };
                tracing::info!(
                    "KB-0004 loaded: {} stem overrides, {} verb profiles, {} noun profiles",
                    stats.stem_overrides,
                    stats.verb_profiles,
                    stats.noun_profiles,
                );
                let kb_arc: Arc<dyn agos_kb::WazanPatternLookup> = Arc::new(kb);
                let parser = MorphologicalParser::with_kb(
                    MorphologicalParserConfig::default(),
                    kb_arc,
                );
                (parser, Some(stats))
            }
            Err(e) => {
                tracing::warn!("KB-0004 load failed: {e} — falling back to heuristic lists");
                (MorphologicalParser::default(), None)
            }
        }
    } else {
        tracing::info!("No KB-0004 directory found at knowledge/KB-0004 — using heuristic fallback");
        (MorphologicalParser::default(), None)
    };

    AppState {
        validator: UnicodeValidator::default(),
        lexer: Lexer::default(),
        tokenizer: Tokenizer::default(),
        morph_parser,
        syntax_parser: SyntaxParser::default(),
        kb_stats,
    }
}

// ──────────────────────────────────────────────
//  Main
// ──────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agos_server=info,tower_http=info".into()),
        )
        .init();

    // Build shared state
    let state = Arc::new(build_app_state());

    // Configure CORS — allow all origins for development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/analyze", post(analyze_handler))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("AGOS server listening on {addr}");
    tracing::info!("Endpoints:");
    tracing::info!("  GET  /health  — Health check");
    tracing::info!("  POST /analyze — Analyze Arabic text");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
