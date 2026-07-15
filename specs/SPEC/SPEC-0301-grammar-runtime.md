---
spec_id: SPEC-0301
title: Grammar Runtime — GVM Execution & Explanation Generation
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0001-C8: Security, Validation & Error Handling
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0101: Morphology Engine
  - SPEC-0201: Rule Engine
  - SPEC-0401: Knowledge Graph Engine
  - SPEC-0501: Explanation Engine
  - RFC-0002: Grammar Bytecode Format
  - RFC-0003: Grammar Virtual Machine
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
  - ADR-0001: Compiler Architecture Rationale
  - ADR-0002: Why Grammar Bytecode
---

# SPEC-0301: Grammar Runtime — GVM Execution & Explanation Generation

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Internal Component Model](#3-internal-component-model)
4. [GVM Integration & Lifecycle](#4-gvm-integration--lifecycle)
5. [GVM Execution Pipeline](#5-gvm-execution-pipeline)
6. [GVM Memory & Resource Management](#6-gvm-memory--resource-management)
7. [GVM Diagnostics & Verification](#7-gvm-diagnostics--verification)
8. [Explanation Engine Architecture](#8-explanation-engine-architecture)
9. [Template System & Localization](#9-template-system--localization)
10. [LLM Integration](#10-llm-integration)
11. [Output Formatting & Rendering](#11-output-formatting--rendering)
12. [Performance & Optimization](#12-performance--optimization)
13. [Testing Strategy](#13-testing-strategy)
14. [Implementation Guidance](#14-implementation-guidance)
15. [Cross-References](#15-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

This specification defines the **Grammar Runtime (GR)** — the runtime execution and explanation layer of the AGOS platform. The Grammar Runtime sits at the boundary between the Compilation Layer (MOD-01 through MOD-09) and the Application Layer. It is responsible for:

1. **Executing Grammar Bytecode** via the Grammar Virtual Machine (MOD-10 / GVM) to produce deterministic `AnalysisResult` objects.
2. **Generating human-readable explanations** (I'rab breakdowns, grammatical descriptions, linguistic flags) via the Explanation Engine (MOD-11).
3. **Managing runtime resources** — memory budgets, execution steps, caching, and sandboxing — to ensure safe, bounded, predictable execution.

The Grammar Runtime is the **public face** of the AGOS analysis pipeline. All applications, APIs, and end users interact with AGOS through the outputs produced by the Grammar Runtime.

### 1.2 Scope

**In scope:**

| Area | Coverage |
|------|----------|
| **MOD-10 (GVM)** | Execution lifecycle, memory management, instruction dispatch, verification, diagnostics, multi-instance management |
| **MOD-11 (ExplanationEngine)** | Template system, localization, output formatting (text/HTML/JSON), LLM integration, I'rab breakdown generation, construction identification |
| **Runtime Layer coordination** | How MOD-10 and MOD-11 interact, shared caching, resource limits, error propagation |
| **Plugin integration** | `explanation` plugin type injection points, custom template providers |
| **Deployment profiles** | Interactive (Class 1), Server (Class 2), Batch (Class 3) — runtime implications |

**Out of scope (covered elsewhere):**

| Out of Scope | Covered By |
|-------------|------------|
| GVM instruction set design | RFC-0003 (Grammar Virtual Machine) |
| Bytecode binary format | RFC-0002 (Grammar Bytecode Format) |
| Bytecode generation (MOD-09) | SPEC-0001-C3 (Compilation Pipeline) |
| LLM service architecture | SPEC-0501 (Explanation Engine) |
| Cache Manager (MOD-13) | SPEC-0001-C4 (Module Interfaces) |
| API Gateway (MOD-14) | SPEC-0001-C4 (Module Interfaces) |
| Plugin Loader (MOD-12) | SPEC-0001-C7 (Plugin Architecture) |

### 1.3 Relationship to Other Specifications

```
SPEC-0001-C2 (Architecture)
    └── Defines Runtime Layer boundaries
SPEC-0001-C3 (Pipeline)
    └── MOD-10 and MOD-11 within the pipeline
SPEC-0001-C4 (Interfaces)
    └── Formal interfaces for MOD-10 and MOD-11
SPEC-0001-C5 (Data Flow)
    └── IR-9 → MOD-10 → IR-10 → MOD-11 → IR-11
SPEC-0001-C7 (Plugins)
    └── explanation plugin type, injection point
SPEC-0001-C8 (Security)
    └── GVM sandboxing, input validation
SPEC-0001-C9 (Performance)
    └── Runtime latency/memory targets
RFC-0002 (Bytecode Format)
    └── Binary format consumed by MOD-10
RFC-0003 (GVM Specification)
    └── Full instruction set, stack model, memory regions
SPEC-0101 (Morphology Engine)
    └── Feature extraction pipeline consumed by GVM
SPEC-0501 (Explanation Engine)
    └── Detailed MOD-11 implementation (LLM, templates)
```

### 1.4 Design Principles

1. **Determinism is absolute.** The Grammar Runtime MUST produce identical output for identical input bytecode and configuration. No randomness, no external state, no time-dependent behavior.

2. **Safety above performance.** The GVM enforces strict bounds on steps and memory before executing the first instruction. Performance is optimized within these safety constraints.

3. **No silent failure.** Every error — from bytecode corruption to LLM service unavailability — MUST be communicated through structured error codes and flags. No data is silently discarded or fabricated.

4. **LLM augments, never replaces.** The LLM enhancement in MOD-11 receives the `AnalysisResult` and generates explanatory text. It NEVER modifies or overrides the deterministic analysis.

5. **Cache-friendly.** Both MOD-10 and MOD-11 are designed for efficient caching. The AnalysisResult (IR-10) is cacheable by its bytecode hash. The ExplanationOutput (IR-11) is cacheable by its AnalysisResult hash.

6. **Minimal runtime footprint.** The Grammar Runtime has zero external runtime dependencies. All required data (bytecode, templates, language packs) is loaded at initialization or embedded in the bytecode itself.

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
                         Compilation Layer
                              │
                              ▼
                    ┌──────────────────────┐
                    │   IR-9: Grammar      │
                    │   Bytecode (.agos)   │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │   GRAMMAR RUNTIME     │
                    │                      │
                    │  ┌────────────────┐  │
    Plugin Layer ───┼─►│  GVM (MOD-10)  │  │
                    │  │                │  │
                    │  │  • Loader      │  │
                    │  │  • Verifier    │  │
                    │  │  • Execution   │  │
                    │  │    Engine       │  │
                    │  │  • State Mgmt  │  │
                    │  └───────┬────────┘  │
                    │          │           │
                    │  ┌───────▼────────┐  │
                    │  │  IR-10:       │  │
                    │  │  AnalysisResult│  │
                    │  └───────┬────────┘  │
                    │          │           │
                    │  ┌───────▼────────┐  │
    Plugin Layer ───┼─►│  Explain.      │  │
                    │  │  Engine(MOD-11)│  │
                    │  │                │  │
                    │  │  • Template    │  │
                    │  │  • I'rab Gen.  │  │
                    │  │  • LLM (opt)   │  │
                    │  │  • Formatter   │  │
                    │  └───────┬────────┘  │
                    └──────────┼───────────┘
                               │
                    ┌──────────▼───────────┐
                    │  IR-11: Explanation  │
                    │  Output              │
                    └──────────────────────┘
                              │
                              ▼
                      Application Layer
                     (API Gateway, UI, etc.)
```

### 2.2 Data Flow Through the Runtime

```
IR-9 (GrammarBytecode)
    │
    │  Byte size: 2–10 KB per sentence
    │  Format: Binary (.agos)
    │  Self-contained: all analysis data embedded
    ▼
┌──────────────────────────────────────────────┐
│  MOD-10: GRAMMAR VIRTUAL MACHINE              │
│                                               │
│  LOAD ──► VERIFY ──► EXECUTE ──► ASSEMBLE    │
│                                               │
│  Input:  GrammarBytecode + GVMConfig          │
│  Steps:  100–10,000 per sentence              │
│  Memory: 64 KB–1 MB per instance              │
│  Output: AnalysisResult                       │
└──────────────────────────────────────────────┘
    │
    │  size: 5–50 KB JSON
    │  structure: tokens + trees + flags + evidence
    ▼
┌──────────────────────────────────────────────┐
│  MOD-11: EXPLANATION ENGINE                   │
│                                               │
│  ANALYZE ──► TEMPLATE ──► RENDER              │
│              │                      │
│              ▼                      ▼
│         I'rab Breakdown       Formatted Output
│         Constructions         (text/HTML/JSON)
│         Overview Text
│                                               │
│  Input:  AnalysisResult + ExplainConfig       │
│  Output: ExplanationOutput                    │
│  LLM:    Optional enhancement                 │
└──────────────────────────────────────────────┘
    │
    │  size: 2–20 KB JSON (text format: 0.5–5 KB)
    ▼
IR-11 (ExplanationOutput)
```

### 2.3 Caching Boundaries

```
Cache Point        Key                           Value          Hit Time
──────────         ───                           ─────          ────────
After MOD-10       hash(bytecode + GVMConfig)     AnalysisResult  < 1 μs
After MOD-11       hash(AnalysisResult + Config)  ExplanationOut. < 1 μs
```

### 2.4 Module Interaction Summary

| Interaction | Origin | Target | Data |
|-------------|--------|--------|------|
| `execute()` | Pipeline Orchestrator | MOD-10 | GrammarBytecode → AnalysisResult |
| `verify()` | Pipeline Orchestrator | MOD-10 | GrammarBytecode → VerificationResult |
| `version()` | Pipeline Orchestrator / Diagnostics | MOD-10 | () → GVMVersion |
| `explain()` | Pipeline Orchestrator | MOD-11 | AnalysisResult → ExplanationOutput |
| `supported_formats()` | API Gateway | MOD-11 | () → string[] |
| `supported_languages()` | API Gateway | MOD-11 | () → string[] |
| `get_plugin()` (explanation) | MOD-11 | PluginLoader | PluginID → ExplanationPlugin |

### 2.5 Key Architectural Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **GVM execution model** | Stack-based interpreter | Simpler to verify deterministically; sufficient for domain-specific instruction count (~57 ops); no JIT complexity needed for v1 |
| **Memory model** | Pre-allocated typed regions | Zero dynamic allocation during execution; bounds-checked at every access; no GC pauses |
| **Explanation fallback** | Template-based with optional LLM | Ensures 100% availability without external services; LLM is pure enhancement |
| **I'rab generation** | Rule-based from AnalysisResult | Deterministic, fast (< 1 ms), covers all standard Arabic constructions |
| **Localization** | JSON-based language packs | Lean (~200 KB per language), no runtime compilation, hot-reloadable |
| **Output format independence** | Logical IR-11 → format-specific renderer | Single generation pipeline; multiple output formats without duplication |

---

## 3. Internal Component Model

### 3.1 Core Data Structures

#### 3.1.1 GVMState (Internal Runtime State)

```rust
/// Complete runtime state of a GVM instance during execution.
/// Pre-allocated at initialization; capacity declared in bytecode header.
struct GVMState {
    // Execution control
    pc: u32,                            // Program counter (byte offset)
    halted: bool,                       // Execution completed
    error: Option<GVMError>,            // Execution error (if any)

    // Stacks
    operand_stack: Stack<Value>,        // Max depth: 1024 (configurable)
    call_stack: Stack<u32>,             // Max depth: 64 (configurable)

    // Typed Memory Regions
    token_region: Region<Token>,        // Token data from bytecode
    feature_region: Region<u64>,        // Packed feature bitfields
    constituent_region: Region<ConstituentNode>,  // Tree structure
    string_region: StringTable,         // Immutable strings from bytecode
    rule_region: Region<RuleRecord>,    // Rule application records
    evidence_region: Region<EvidenceRecord>,  // Evidence trail
    scratch_region: ScratchBuffer,      // Temporary workspace

    // Metrics
    step_count: u64,                    // Instructions executed
    peak_memory_used: u64,              // Max memory during execution
    started_at: Instant,               // Execution start wall-clock time
}

struct Stack<T> {
    data: Vec<T>,
    max_depth: u32,
}

struct Region<T> {
    data: Vec<Cell<T>>,                // Pre-allocated, fixed capacity
    size: u32,                         // Current number of valid entries
    capacity: u32,                     // Maximum entries (from bytecode header)
}

struct StringTable {
    strings: Vec<String>,              // Immutable; loaded from bytecode
    lookup: HashMap<u32, usize>,       // Index → position in strings
}

struct ScratchBuffer {
    data: Vec<u8>,                     // Raw byte buffer
    capacity: u32,
    cursor: u32,                       // Current write position
}
```

#### 3.1.2 Value (Operand Stack Item)

```rust
/// Values that can appear on the GVM operand stack.
enum Value {
    I32(i32),
    I64(i64),
    F64(f64),
    StringIndex(u32),        // Index into string table
    TokenIndex(u32),         // Index into token region
    FeatureBits(u64),        // Packed morphological feature bits
    ConstituentPtr(u32),     // Index into constituent region
    Bool(bool),
    Null,
}
```

#### 3.1.3 GVMConfig (Runtime Configuration)

```rust
/// Configuration for a single GVM execution.
struct GVMConfig {
    max_execution_steps: u32,           // Default: 100,000
    max_memory_bytes: u64,              // Default: 64 MiB
    sandbox_mode: bool,                 // Default: true (must be true in production)
    tracing_enabled: bool,              // Default: false
    max_call_depth: u32,                // Default: 64
    max_stack_depth: u32,               // Default: 1024
    performance_profile: Profile,       // interactive | server | batch | debug | educational
}

enum Profile {
    Interactive,   // Strict latency budget (< 10 ms), step limit: 10,000
    Server,        // Balanced throughput, step limit: 100,000
    Batch,         // Higher throughput, step limit: 1,000,000
    Debug,         // Full tracing, step limit: 10,000,000
    Educational,   // Tracing with human-readable output, step limit: 50,000
}
```

#### 3.1.4 AnalysisResult (IR-10)

```rust
/// The canonical output of the GVM — a fully executed grammatical analysis.
/// This is the sole input to the Explanation Engine (MOD-11).
struct AnalysisResult {
    // Schema metadata
    spec: String,                        // "SPEC-0001/IR-10"
    version: String,                     // "1.0"

    // Execution metadata
    metadata: AnalysisMetadata,

    // Content
    input_text: String,                  // Original input text
    input_text_hash: String,             // SHA-256 of input text

    trees: Vec<AnalysisTree>,            // One per successful parse (1–3 typical)

    flags: Vec<GrammaticalFlag>,         // All grammatical flags
    evidence: Vec<EvidenceEntry>,        // Complete evidence trail
}

struct AnalysisMetadata {
    executed_at: String,                 // ISO 8601
    execution_time_ms: f64,
    steps_executed: u64,
    memory_used: u64,                    // Bytes
    bytecode_size: u64,                  // Input bytecode size in bytes
    bytecode_version: String,            // e.g., "1.0.0"
    gvm_version: String,                 // e.g., "1.2.3"
}

struct AnalysisTree {
    id: String,                          // Unique tree ID
    tree_type: String,                   // e.g., "jumlah_fi'liyyah"
    tokens: Vec<AnalysisToken>,
    constituents: Vec<Constituent>,       // From GVM constituent region
    flags: Vec<GrammaticalFlag>,
    confidence: f64,                     // 0.0 to 1.0
}

struct AnalysisToken {
    index: u32,                          // Token index
    text: String,                        // Token text
    features: TokenFeatures,             // Combined feature sets
    evidence: Vec<EvidenceEntry>,
}

struct TokenFeatures {
    morphological: MorphologicalFeatures,
    syntactic: SyntacticFeatures,
    semantic: SemanticFeatures,
}

struct MorphologicalFeatures {
    root: Option<String>,                // e.g., "كتب"
    wazan: Option<String>,               // e.g., "فَعَلَ"
    pos: String,                         // Part of speech
    gender: Option<String>,
    number: Option<String>,
    person: Option<String>,
    tense: Option<String>,
    mood: Option<String>,
    voice: Option<String>,
    case: Option<String>,
    state: Option<String>,               // definite / indefinite
    verb_form: Option<u8>,               // I–XV
    noun_type: Option<String>,
    transitivity: Option<String>,
}

struct SyntacticFeatures {
    role: Option<String>,                // e.g., "fi'l", "fa'il", "mubtada'"
    governor: Option<u32>,               // Token index of governing word
}

struct SemanticFeatures {
    tags: Vec<String>,                   // e.g., ["human", "action"]
    definition: Option<String>,
    root_meaning: Option<String>,
}
```

#### 3.1.5 ExplainConfig (MOD-11 Configuration)

```rust
/// Configuration for the Explanation Engine.
struct ExplainConfig {
    language: String,                    // Default: "en"
    format: OutputFormat,               // Default: JSON
    include_evidence: bool,              // Default: false
    include_flags: bool,                 // Default: true
    include_raw: bool,                   // Include formatted raw string; default: true
    enable_llm: bool,                    // Default: false
    llm: Option<LLMConfig>,
}

enum OutputFormat { Text, Html, Json, Pdf }

struct LLMConfig {
    provider: String,                    // e.g., "openai"
    model: String,                       // e.g., "gpt-4"
    temperature: f64,                    // 0.0–1.0; default: 0.3
    max_tokens: u32,                     // Default: 500
    timeout_ms: u32,                     // Default: 5000
}
```

#### 3.1.6 ExplanationOutput (IR-11)

```rust
/// The final output of the AGOS pipeline — human-readable explanations.
struct ExplanationOutput {
    // Schema metadata
    spec: String,                        // "SPEC-0001/IR-11"
    version: String,                     // "1.0"

    // Generation metadata
    metadata: ExplanationMetadata,

    // Content
    input_text: String,
    overview: String,                    // Summary of the grammatical analysis
    sentence_type: Option<String>,

    // I'rab breakdown (word-by-word)
    irab_breakdown: Vec<IrabEntry>,

    // Notable grammatical constructions
    constructions: Vec<Construction>,

    // Flags (errors, warnings, info)
    flags: Vec<FlagDisplay>,

    // Evidence trail (if include_evidence)
    evidence: Vec<EvidenceEntry>,

    // Formatted output string
    raw: String,                         // According to 'format' field
}

struct ExplanationMetadata {
    generated_at: String,                // ISO 8601
    language: String,
    format: String,
    llm_enhanced: bool,
    generation_time_ms: f64,
    pipeline_timing_ms: PipelineTiming,  // Per-stage breakdown
}

struct PipelineTiming {
    total: f64,
    validation: f64,
    lexing: f64,
    tokenization: f64,
    morphology: f64,
    syntax: f64,
    gir_construction: f64,
    rule_engine: f64,
    kg_resolution: f64,
    bytecode_generation: f64,
    gvm_execution: f64,
    explanation: f64,
}

struct IrabEntry {
    token: String,
    root: Option<String>,
    pos: String,
    features: Vec<FeatureDisplay>,
    syntactic_role: Option<String>,
    explanation: String,                 // Localized natural language
}

struct FeatureDisplay {
    name: String,                        // e.g., "Gender"
    value: String,                       // e.g., "Masculine"
    category: String,                    // e.g., "inflectional"
}

struct Construction {
    name: String,                        // e.g., "Idafa (Construct State)"
    description: String,                 // Localized description
    tokens: Vec<u32>,                    // Token indices involved
}

struct FlagDisplay {
    flag_type: String,                   // "error" | "warning" | "info"
    code: String,                        // e.g., "SUBJECT_VERB_AGREEMENT"
    message: String,
    tokens: Vec<u32>,
}
```

### 3.2 Runtime Configuration Model

```rust
/// Complete runtime configuration, combining MOD-10 and MOD-11 settings.
struct GrammarRuntimeConfig {
    // GVM configuration
    gvm: GVMConfig,

    // Explanation Engine configuration
    explanation: ExplainConfig,

    // Shared runtime settings
    cache_enabled: bool,
    cache_ttl_seconds: Option<u64>,      // Default: 3600
    plugin_directories: Vec<String>,      // Default: ["./plugins"]
    template_path: Option<String>,        // Custom template directory
    language_pack_path: Option<String>,   // Custom language pack directory

    // Resource limits
    max_concurrent_gvm_instances: u32,    // Default: 64
    gvm_instance_pool_size: u32,          // Default: 16 (pre-allocated pool)
    evidence_max_entries: u32,           // Default: 2048
}

impl Default for GrammarRuntimeConfig {
    fn default() -> Self {
        Self {
            gvm: GVMConfig {
                max_execution_steps: 100_000,
                max_memory_bytes: 64 * 1024 * 1024,
                sandbox_mode: true,
                tracing_enabled: false,
                max_call_depth: 64,
                max_stack_depth: 1024,
                performance_profile: Profile::Server,
            },
            explanation: ExplainConfig {
                language: "en".into(),
                format: OutputFormat::Json,
                include_evidence: false,
                include_flags: true,
                include_raw: true,
                enable_llm: false,
                llm: None,
            },
            cache_enabled: true,
            cache_ttl_seconds: Some(3600),
            plugin_directories: vec!["./plugins".into()],
            template_path: None,
            language_pack_path: None,
            max_concurrent_gvm_instances: 64,
            gvm_instance_pool_size: 16,
            evidence_max_entries: 2048,
        }
    }
}
```

### 3.3 Plugin Integration Points

```rust
/// Explanation plugin trait — injected into MOD-11 at initialization.
trait ExplanationPlugin {
    fn plugin_id(&self) -> &str;
    fn plugin_type(&self) -> PluginType;       // PluginType::Explanation
    fn priority(&self) -> u8;                  // 0–255 (higher = applied first)

    /// Generate or augment explanation for a single token.
    fn explain_token(
        &self,
        token: &AnalysisToken,
        context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError>;

    /// Generate or augment the overview summary.
    fn explain_overview(
        &self,
        result: &AnalysisResult,
        context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError>;

    /// Get custom constructions recognized by this plugin.
    fn custom_constructions(&self) -> Vec<ConstructionDef>;
}

struct ExplanationContext {
    language: String,
    format: OutputFormat,
    analysis: AnalysisResult,
    template_registry: TemplateRegistry,
    language_pack: LanguagePack,
}

struct ConstructionDef {
    name: String,
    description_template: String,        // Template string with {tokens} placeholder
    condition: String,                   // DSL condition string (future use)
}

/// Built-in template registry — can be extended by plugins.
struct TemplateRegistry {
    templates: HashMap<String, Template>,  // Template name → compiled template
    fallback_language: String,             // Default: "en"
}

/// Language pack — localized strings for all explanation components.
struct LanguagePack {
    language: String,
    labels: HashMap<String, String>,         // e.g., "tense.past" → "Past Tense"
    descriptions: HashMap<String, String>,   // e.g., "jumlah_fi'liyyah" → "A verbal sentence begins with a verb..."
    pos_names: HashMap<String, String>,      // Part of speech names
    feature_names: HashMap<String, String>,  // Feature display names
    role_names: HashMap<String, String>,     // Syntactic role display names
    error_messages: HashMap<String, String>, // Grammatical error messages
}
```

---

## 4. GVM Integration & Lifecycle

### 4.1 GVM Instance Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                        GVM Instance Lifecycle                        │
│                                                                     │
│  ┌────────┐    ┌─────────┐    ┌───────────┐    ┌─────────┐        │
│  │ CREATE │───►│ LOAD    │───►│ VERIFY    │───►│ EXECUTE │───►    │
│  │ (pool) │    │ bytecode│    │ bytecode  │    │ instrs  │         │
│  └────────┘    └─────────┘    └───────────┘    └────┬────┘         │
│                                                      │              │
│                                          ┌───────────┼──────────┐   │
│                                          ▼           ▼          ▼   │
│                                     ┌────────┐ ┌────────┐ ┌───────┐│
│                                     │ HALTED │ │ ERROR  │ │TIMEOUT││
│                                     │  (ok)  │ │        │ │       ││
│                                     └────────┘ └────────┘ └───────┘│
│                                                      │              │
│                                                     ┌▼────────────┐│
│                                                     │ RESET/POOL  ││
│                                                     │ (reuse)     ││
│                                                     └─────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

**Lifecycle states:**

| State | Description | Next States |
|-------|-------------|-------------|
| **CREATED** | Instance allocated from pool; memory regions reserved | LOADING |
| **LOADING** | Bytecode being parsed; sections mapped to regions | LOADED, ERROR |
| **LOADED** | Bytecode parsed; ready for verification | VERIFYING |
| **VERIFYING** | Checksums, bounds, version being checked | VERIFIED, ERROR |
| **VERIFIED** | Bytecode valid; ready for execution | EXECUTING |
| **EXECUTING** | Instructions being fetched/decoded/executed | HALTED, ERROR, TIMEOUT |
| **HALTED** | Execution completed successfully (HALT reached) | OUTPUT_COLLECTED |
| **ERROR** | Fatal error during any phase | None (instance recycled) |
| **TIMEOUT** | Step limit exceeded | None (instance recycled) |
| **OUTPUT_COLLECTED** | AnalysisResult assembled and returned | RECYCLED |
| **RECYCLED** | Instance returned to pool; all regions cleared | CREATED |

### 4.2 Instance Pool Management

```rust
/// Thread-safe pool of pre-initialized GVM instances.
/// Avoids allocation overhead for every execution request.
struct GVMInstancePool {
    instances: Vec<GVMInstance>,
    config: GVMConfig,
    max_pool_size: u32,
    active_count: AtomicU32,
}

impl GVMInstancePool {
    /// Acquire an instance from the pool (blocking if pool is empty).
    fn acquire(&self) -> GVMInstance { ... }

    /// Release an instance back to the pool.
    fn release(&self, instance: GVMInstance) { ... }

    /// Pre-warm the pool with `count` instances.
    fn pre_warm(&mut self, count: u32) -> Result<(), GVMError> { ... }
}

/// Each GVM instance has its own isolated state.
struct GVMInstance {
    id: u64,
    state: GVMState,
    bytecode: Option<GrammarBytecode>,
    config: GVMConfig,
    created_at: Instant,
    last_used_at: Instant,
}
```

### 4.3 Execution Flow

```rust
/// Complete MOD-10 execution flow.
fn execute(bytecode: GrammarBytecode, config: GVMConfig) -> Result<AnalysisResult, GVMError> {
    // 1. Acquire instance from pool
    let mut instance = pool.acquire();

    // 2. Load bytecode
    instance.load(&bytecode)?;

    // 3. Verify bytecode
    let verification = instance.verify();
    if !verification.valid {
        return Err(GVMError::BytecodeCorrupted(verification.issues));
    }

    // 4. Execute instructions
    instance.run()?;

    // 5. Assemble output
    let result = instance.assemble_output();

    // 6. Release instance back to pool
    pool.release(instance);

    Ok(result)
}
```

### 4.4 Concurrency Model

| Aspect | Specification |
|--------|---------------|
| **Per-instance threading** | Single-threaded. One GVM instance executes on one thread. |
| **Concurrency** | Multiple GVM instances can run in parallel on separate threads. |
| **Instance isolation** | Instances share no mutable state. All data is copied at load time. |
| **Pool synchronization** | `Mutex` or lock-free queue for pool acquire/release. |
| **Async support** | GVM execution is CPU-bound; use `spawn_blocking` or thread pool for async runtimes. |
| **Maximum concurrency** | `max_concurrent_gvm_instances` (default: 64). |

### 4.5 Error Handling & Recovery

```rust
enum GVMError {
    // Bytecode errors (detected at load/verify time)
    UnsupportedBytecodeVersion { bytecode: (u16, u16, u16), gvm: (u16, u16, u16) },
    BytecodeCorrupted { issues: Vec<VerificationIssue> },
    InvalidMagic,
    SectionMissing { section_id: u8 },
    SectionOrderingViolation,

    // Execution errors
    MaxStepsExceeded { steps: u64, limit: u64 },
    MaxMemoryExceeded { memory: u64, limit: u64 },
    ExecutionFailure { description: String, pc: u32 },
    StackOverflow { stack_type: String, depth: u32, max: u32 },
    StackUnderflow,
    TypeError { expected: String, got: String, pc: u32 },
    InvalidOpcode { opcode: u8, pc: u32 },
    TokenIndexOutOfBounds { index: u32, count: u32 },
    StringIndexOutOfBounds { index: u32, count: u32 },
    FeatureIndexOutOfBounds { index: u32, count: u32 },
    ConstituentIndexOutOfBounds { index: u32, count: u32 },
    CallStackOverflow { depth: u32, max: u32 },
    CallStackUnderflow,
    InvalidFeatureId { feature_id: u32 },
    ScratchOverflow { cursor: u32, capacity: u32 },
    DivisionByZero { pc: u32 },
    InternalError { description: String },
}

/// Severity classification for error handling.
impl GVMError {
    fn is_fatal(&self) -> bool {
        match self {
            // All GVM errors are fatal — execution cannot continue
            _ => true,
        }
    }

    fn recovery_hint(&self) -> Option<&str> {
        match self {
            GVMError::UnsupportedBytecodeVersion { .. } =>
                Some("Update the GVM to a version that supports this bytecode version"),
            GVMError::BytecodeCorrupted { .. } =>
                Some("Regenerate the bytecode from the source GIR"),
            GVMError::MaxStepsExceeded { .. } =>
                Some("Increase max_execution_steps or simplify the input"),
            GVMError::MaxMemoryExceeded { .. } =>
                Some("Increase max_memory or reduce the input complexity"),
            GVMError::InvalidFeatureId { .. } =>
                Some("The bytecode references features not in the current KB-0007 taxonomy"),
            _ => None,
        }
    }
}
```

### 4.6 Bytecode Version Compatibility

```rust
/// Version compatibility: GVM vM.x executes bytecode vM.y where y <= x.
fn check_version_compatibility(
    bytecode_version: (u16, u16, u16),
    gvm_version: (u16, u16, u16),
) -> Result<(), GVMError> {
    let (bc_major, bc_minor, _bc_patch) = bytecode_version;
    let (gvm_major, gvm_minor, _gvm_patch) = gvm_version;

    // Major version must match
    if bc_major != gvm_major {
        return Err(GVMError::UnsupportedBytecodeVersion {
            bytecode: bytecode_version,
            gvm: gvm_version,
        });
    }

    // Bytecode minor must not exceed GVM minor
    if bc_minor > gvm_minor {
        return Err(GVMError::UnsupportedBytecodeVersion {
            bytecode: bytecode_version,
            gvm: gvm_version,
        });
    }

    Ok(())
}
```

---

## 5. GVM Execution Pipeline

### 5.1 Instruction Cycle

The GVM uses a classic fetch-decode-execute cycle. Each cycle is one **step**:

```rust
fn execute_cycle(state: &mut GVMState, bytecode: &[u8]) -> Result<(), GVMError> {
    // 1. FETCH — read opcode and flags from bytecode stream
    let (opcode, flags, instruction_len) = fetch_instruction(bytecode, state.pc);

    // 2. DECODE — parse operands based on opcode
    let operands = decode_operands(bytecode, state.pc + 2, opcode)?;

    // 3. VALIDATE — type-check operands, bounds-check region references
    validate_operands(&operands, &state, opcode)?;

    // 4. EXECUTE — dispatch to instruction handler
    execute_instruction(opcode, flags, &operands, state)?;

    // 5. UPDATE — advance program counter, increment step count
    state.pc += instruction_len;
    state.step_count += 1;

    // 6. TRACE (optional)
    if state.config.tracing_enabled {
        record_trace(state, opcode, &operands);
    }

    // 7. BOUNDS CHECK
    if state.step_count >= state.config.max_execution_steps {
        return Err(GVMError::MaxStepsExceeded {
            steps: state.step_count,
            limit: config.max_execution_steps,
        });
    }

    Ok(())
}
```

### 5.2 Instruction Dispatch

The instruction dispatch is implemented as a flat opcode table for O(1) lookup:

```rust
/// Dispatch table mapping opcodes to handler functions.
/// 256 entries (0x00–0xFF); unused entries return InvalidOpcode error.
const INSTRUCTION_TABLE: [Option<InstructionHandler>; 256] = {
    // Flow Control (0x00–0x0F)
    [0x00] = Some(handle_halt),
    [0x01] = Some(handle_jump),
    [0x02] = Some(handle_jump_if_true),
    [0x03] = Some(handle_jump_if_false),
    [0x04] = Some(handle_call),
    [0x05] = Some(handle_return),
    [0x06] = Some(handle_die),
    // ... remaining flow control slots reserved

    // Stack Operations (0x10–0x1F)
    [0x10] = Some(handle_push_i32),
    [0x11] = Some(handle_push_i64),
    [0x12] = Some(handle_push_f64),
    [0x13] = Some(handle_push_bool),
    [0x14] = Some(handle_push_string),
    [0x15] = Some(handle_push_null),
    [0x16] = Some(handle_pop),
    [0x17] = Some(handle_dup),
    [0x18] = Some(handle_swap),
    // ... remaining stack slots reserved

    // Token Operations (0x20–0x2F)
    [0x20] = Some(handle_load_token),
    [0x21] = Some(handle_token_get_text),
    [0x22] = Some(handle_token_get_offset),
    [0x23] = Some(handle_token_get_type),
    [0x24] = Some(handle_token_get_features),
    [0x25] = Some(handle_token_iterate),
    [0x26] = Some(handle_token_count),
    // ... remaining token slots reserved

    // Feature Operations (0x30–0x3F)
    [0x30] = Some(handle_feature_get),
    [0x31] = Some(handle_feature_set),
    [0x32] = Some(handle_feature_has),
    [0x33] = Some(handle_feature_compare_eq),
    [0x34] = Some(handle_feature_compare_mask),
    [0x35] = Some(handle_feature_pack),
    // ... remaining feature slots reserved

    // Constituent Operations (0x40–0x4F)
    [0x40] = Some(handle_const_make),
    [0x41] = Some(handle_const_add_child),
    [0x42] = Some(handle_const_get_child),
    [0x43] = Some(handle_const_get_role),
    [0x44] = Some(handle_const_set_role),
    [0x45] = Some(handle_const_attach_tokens),
    [0x46] = Some(handle_const_traverse),
    // ... remaining constituent slots reserved

    // Rule Operations (0x50–0x5F)
    [0x50] = Some(handle_rule_apply),
    [0x51] = Some(handle_rule_confirm),
    [0x52] = Some(handle_rule_reject),
    [0x53] = Some(handle_rule_modify),
    [0x54] = Some(handle_rule_flag),
    [0x55] = Some(handle_rule_resolve),
    // ... remaining rule slots reserved

    // Evidence Operations (0x60–0x6F)
    [0x60] = Some(handle_evidence_push),
    [0x61] = Some(handle_evidence_query),
    [0x62] = Some(handle_evidence_emit),
    // ... remaining evidence slots reserved

    // Output Operations (0x70–0x7F)
    [0x70] = Some(handle_output_set_metadata),
    [0x71] = Some(handle_output_add_tree),
    [0x72] = Some(handle_output_add_token),
    [0x73] = Some(handle_output_set_input),
    [0x74] = Some(handle_output_finalize),
    // ... remaining output slots reserved

    // Reserved (0x80–0xFF): all None
};

type InstructionHandler = fn(flags: u8, operands: &[Operand], state: &mut GVMState) -> Result<(), GVMError>;
```

### 5.3 Execution Guarantees

| Guarantee | Mechanism | Enforced By |
|-----------|-----------|-------------|
| **Bounded steps** | Step counter checked before every instruction | `execute_cycle()` step 7 |
| **Bounded memory** | Pre-allocated regions; all writes bounds-checked | `Region::write()` accessor |
| **No infinite loops** | Step limit and HALT instruction | Both hardware and software enforcement |
| **No side effects** | No I/O, no syscalls, no external process calls | Language runtime (WASM/process isolation) |
| **Deterministic output** | Fixed instruction order, no randomness, no concurrency | Design-by-construction |
| **No stack corruption** | Type-checked operand stack; every PUSH/POP validated | `validate_operands()` before each dispatch |
| **No memory aliasing** | All region access is by typed index, not pointer | Region type system |

### 5.4 Instruction Cost Model

Each instruction has a known cost in steps and approximate wall-clock time:

| Category | Instructions | Steps per Instr | Approx Time (ns) |
|----------|-------------|-----------------|-------------------|
| Flow Control | HALT, JUMP, CALL, RETURN | 1 | 5–20 |
| Stack Ops | PUSH, POP, DUP, SWAP | 1 | 5–15 |
| Token Ops | LOAD_TOKEN, TOKEN_GET_FEATURES | 1–2 | 10–30 |
| Feature Ops | FEATURE_GET, FEATURE_SET | 1–2 | 10–25 |
| Constituent Ops | CONST_MAKE, CONST_ADD_CHILD | 1–3 | 20–50 |
| Rule Ops | RULE_APPLY, RULE_CONFIRM | 1 | 15–30 |
| Evidence Ops | EVIDENCE_PUSH | 2 | 20–40 |
| Output Ops | OUTPUT_ADD_TREE, OUTPUT_FINALIZE | 1–2 | 25–60 |

**Typical instruction mix for a 10-word sentence:**
- 40% Token/Feature operations (loading data)
- 25% Constituent operations (building analysis trees)
- 15% Stack operations (managing values)
- 10% Rule operations (recording rule applications)
- 5% Evidence operations (building evidence trail)
- 3% Output operations (assembling result)
- 2% Flow control (loops, calls)

**Estimated instruction count per sentence:** 200–2,000 instructions (varying with ambiguity).

### 5.5 Output Assembly

After execution completes (HALT reached), the GVM assembles the `AnalysisResult`:

```rust
fn assemble_output(state: &GVMState) -> AnalysisResult {
    // 1. Collect metadata from execution
    let metadata = AnalysisMetadata {
        executed_at: now_iso8601(),
        execution_time_ms: state.started_at.elapsed().as_micros_f64(),
        steps_executed: state.step_count,
        memory_used: calculate_peak_memory(state),
        bytecode_size: state.bytecode_size,
        bytecode_version: state.bytecode_version.clone(),
        gvm_version: GVM_VERSION_STRING.into(),
    };

    // 2. Build analysis trees from constituent region
    let trees = build_analysis_trees(state);

    // 3. Collect flags from rule region
    let flags = collect_flags(state);

    // 4. Collect evidence from evidence region
    let evidence = collect_evidence(state);

    // 5. Verify structural integrity
    debug_assert!(trees.iter().all(|t| t.tokens.len() > 0));
    debug_assert!(trees.windows(2).all(|w| w[0].confidence >= w[1].confidence)); // sorted

    AnalysisResult {
        spec: "SPEC-0001/IR-10".into(),
        version: "1.0".into(),
        metadata,
        input_text: state.input_text.clone(),
        input_text_hash: state.input_text_hash.clone(),
        trees,
        flags,
        evidence,
    }
}
```

---

## 6. GVM Memory & Resource Management

### 6.1 Memory Region Architecture

```
GVM Memory (per instance)
│
├── Token Region         (capacity: 256 tokens, 32–64 bytes each)
│   └── Stores token-indices, text references, offsets, feature refs
│
├── Feature Region       (capacity: 512 bitfields, 8 bytes each)
│   └── Stores packed u64 morphological feature bitfields
│
├── Constituent Region   (capacity: 1024 nodes, ~48 bytes each)
│   └── Stores tree nodes: type, role, child refs, feature refs
│
├── String Region        (capacity: 65535 strings, variable)
│   └── Immutable; loaded from bytecode string table
│
├── Rule Region           (capacity: 512 records, ~32 bytes each)
│   └── Stores rule application records
│
├── Evidence Region      (capacity: 1024 entries, ~64 bytes each)
│   └── Stores evidence trail entries
│
├── Scratch Buffer       (capacity: 4096 bytes)
│   └── Temporary workspace for string manipulation, etc.
│
├── Operand Stack        (max depth: 1024 entries, ~16 bytes each)
│   └── Value stack for instruction operands
│
└── Call Stack           (max depth: 64 entries, 4 bytes each)
    └── Return addresses for CALL/RETURN
```

### 6.2 Capacity Tuning

| Region | Default Capacity | Per-Entry Size | Max Total | Adjust When |
|--------|-----------------|---------------|-----------|-------------|
| Token | 256 | 48 B | 12 KB | Sentences > 50 tokens |
| Feature | 512 | 8 B | 4 KB | High per-token ambiguity (> 4 analyses/token) |
| Constituent | 1024 | 48 B | 48 KB | Complex nested constructions |
| String | (from bytecode) | variable | < 10 KB (in bytecode) | Never (bytecode-defined) |
| Rule | 512 | 32 B | 16 KB | > 500 rule applications expected |
| Evidence | 1024 | 64 B | 64 KB | Full evidence trail for long sentences |
| Scratch | 4096 | 1 B | 4 KB | Never (sufficient for all operations) |
| Operand Stack | 1024 | 16 B | 16 KB | Deeply nested rule conditions |
| Call Stack | 64 | 4 B | 256 B | Deeply nested subroutine calls |

**Default total memory per GVM instance:** ~170 KB (all regions).
**Maximum (with all capacities at max):** ~1 MB.

### 6.3 Memory Safety Model

```
┌─────────────────────────────────────────────────────────────┐
│                  Memory Safety Guarantees                     │
│                                                              │
│  1. NO RAW POINTERS                                          │
│     All memory access is through typed indices (u32).        │
│     No way to construct an arbitrary memory address.         │
│                                                              │
│  2. NO DYNAMIC ALLOCATION                                    │
│     All memory is pre-allocated at instance creation.        │
│     "Allocation" is just incrementing a size counter         │
│     within capacity limits.                                  │
│                                                              │
│  3. BOUNDS CHECKED ON EVERY ACCESS                           │
│     Every read/write validates: index < region.size.         │
│     OOB access returns STACK_OVERFLOW / *_OUT_OF_BOUNDS.     │
│                                                              │
│  4. NO USE-AFTER-FREE                                        │
│     No deallocation during execution. Memory freed only      │
│     when GVM instance is recycled to pool (full reset).      │
│                                                              │
│  5. TYPE SAFETY ON STACK                                      │
│     Every Value on the operand stack is tagged with its      │
│     type. Type mismatches are detected before execution.     │
│                                                              │
│  6. SANDBOX ISOLATION                                        │
│     In sandbox mode (default), the GVM process cannot        │
│     perform file I/O, network calls, or execute syscalls.    │
└─────────────────────────────────────────────────────────────┘
```

### 6.4 Memory Budget Calculation

```rust
/// Calculate the memory budget required for a given bytecode.
fn calculate_memory_budget(bytecode: &GrammarBytecode, config: &GVMConfig) -> u64 {
    let header = &bytecode.header;

    let token_region = header.token_count as u64 * TOKEN_ENTRY_SIZE;
    let feature_region = header.feature_count as u64 * FEATURE_ENTRY_SIZE;
    let constituent_region = header.constituent_count as u64 * CONSTITUENT_ENTRY_SIZE;
    let string_region = bytecode.string_table.byte_size() as u64;
    let rule_region = min(header.rule_count as u64, config.max_rule_applications as u64) * RULE_ENTRY_SIZE;
    let evidence_region = config.evidence_max_entries as u64 * EVIDENCE_ENTRY_SIZE;
    let scratch = SCRATCH_DEFAULT_SIZE as u64;
    let operand_stack = config.max_stack_depth as u64 * VALUE_ENTRY_SIZE;
    let call_stack = config.max_call_depth as u64 * 4; // u32 return addresses
    let overhead = 4096; // VM overhead (structs, enums, etc.)

    token_region + feature_region + constituent_region + string_region +
    rule_region + evidence_region + scratch + operand_stack + call_stack + overhead
}

const TOKEN_ENTRY_SIZE: u64 = 48;
const FEATURE_ENTRY_SIZE: u64 = 8;
const CONSTITUENT_ENTRY_SIZE: u64 = 48;
const RULE_ENTRY_SIZE: u64 = 32;
const EVIDENCE_ENTRY_SIZE: u64 = 64;
const VALUE_ENTRY_SIZE: u64 = 16;
const SCRATCH_DEFAULT_SIZE: u64 = 4096;
```

### 6.5 Sandboxing

```rust
/// Sandbox modes for GVM execution.
enum SandboxMode {
    /// Full sandbox: no I/O, no syscalls, bounded resources.
    /// REQUIRED for production. Default.
    Full,

    /// Permissive: allows file I/O for debugging (tracing output).
    /// NOT for production use.
    DebugOnly {
        allowed_paths: Vec<String>,     // Only these paths can be written
    },

    /// No sandboxing. ONLY for testing.
    /// MUST NOT be used in production.
    Disabled,
}

/// Sandbox capabilities checked at GVM initialization.
struct SandboxCapabilities {
    can_read_fs: bool,
    can_write_fs: bool,
    can_network: bool,
    can_exec: bool,
    can_environ: bool,
    max_memory: u64,
    max_cpu_time_ms: u64,
}
```

---

## 7. GVM Diagnostics & Verification

### 7.1 Bytecode Verification

```rust
/// Verification result for a bytecode file.
struct VerificationResult {
    valid: bool,
    issues: Vec<VerificationIssue>,
}

struct VerificationIssue {
    severity: Severity,          // error | warning
    code: String,                // e.g., "CHECKSUM_MISMATCH"
    message: String,
    offset: Option<u32>,         // Byte offset in bytecode
    section_id: Option<u8>,      // Section ID if applicable
}

enum Severity { Error, Warning }

/// Full verification pipeline.
fn verify_bytecode(bytecode: &GrammarBytecode) -> VerificationResult {
    let mut issues = Vec::new();

    // 1. Magic bytes check
    verify_magic(bytecode, &mut issues);

    // 2. Version compatibility
    verify_version(bytecode, &mut issues);

    // 3. CRC32C checksum verification (per section)
    verify_checksums(bytecode, &mut issues);

    // 4. Section ordering and structure
    verify_sections(bytecode, &mut issues);

    // 5. Instruction stream validity
    verify_instructions(bytecode, &mut issues);

    // 6. Jump target validation
    verify_jump_targets(bytecode, &mut issues);

    // 7. String table integrity
    verify_string_table(bytecode, &mut issues);

    // 8. Feature ID validation against KB-0007 taxonomy
    verify_feature_ids(bytecode, &mut issues);

    // 9. Resource bounds check
    verify_resource_bounds(bytecode, &mut issues);

    VerificationResult {
        valid: issues.iter().all(|i| i.severity != Error),
        issues,
    }
}
```

### 7.2 Execution Tracing

```rust
/// Trace entry produced when tracing is enabled.
struct TraceEntry {
    instruction_number: u64,         // Sequential instruction number
    pc: u32,                          // Program counter
    opcode: u8,                       // Instruction opcode
    opcode_name: String,              // Human-readable name
    operands: Vec<String>,            // Operand values (human-readable)
    stack_before: Vec<Value>,         // Top 5 values of operand stack before
    stack_after: Vec<Value>,          // Top 5 values after
    memory_delta: Option<MemoryDelta>, // Memory changes from this instruction
    wall_time_ns: u64,                // Execution time in nanoseconds
}

struct MemoryDelta {
    region: String,                   // e.g., "token", "feature"
    index: u32,
    old_value: String,
    new_value: String,
}

/// Trace output structure.
struct ExecutionTrace {
    bytecode_info: BytecodeInfo,
    config: GVMConfig,
    entries: Vec<TraceEntry>,
    summary: TraceSummary,
}

struct TraceSummary {
    total_instructions: u64,
    total_time_ns: u64,
    peak_memory_bytes: u64,
    instruction_breakdown: HashMap<u8, InstructionStats>,  // opcode → stats
}

struct InstructionStats {
    count: u64,
    total_time_ns: u64,
    avg_time_ns: f64,
    min_time_ns: u64,
    max_time_ns: u64,
}
```

### 7.3 Disassembler

The GVM includes a disassembler that converts bytecode to human-readable instruction listings:

```rust
/// Disassemble bytecode to a human-readable instruction listing.
fn disassemble(bytecode: &GrammarBytecode) -> Disassembly {
    let header = &bytecode.header;
    let instructions = parse_instructions(bytecode);

    let lines: Vec<DisassemblyLine> = instructions.iter().map(|instr| {
        DisassemblyLine {
            offset: instr.offset,
            opcode: instr.opcode,
            mnemonic: opcode_to_mnemonic(instr.opcode),
            operands: format_operands(&instr.operands, &bytecode.string_table),
            comment: None,
        }
    }).collect();

    Disassembly {
        version: format!("{}.{}.{}", header.version_major, header.version_minor, header.version_patch),
        token_count: header.token_count,
        instruction_count: header.instruction_count,
        total_size: bytecode.raw.len(),
        lines,
    }
}

struct Disassembly {
    version: String,
    token_count: u16,
    instruction_count: u32,
    total_size: usize,
    lines: Vec<DisassemblyLine>,
}

struct DisassemblyLine {
    offset: u32,
    opcode: u8,
    mnemonic: String,
    operands: String,
    comment: Option<String>,
}
```

### 7.4 CLI Interface

```bash
# Execute bytecode and produce AnalysisResult (JSON)
agos gvm run --bytecode=analysis.agos --output=result.json

# Execute with custom config
agos gvm run --bytecode=analysis.agos \
    --max-steps=50000 \
    --max-memory=32MB \
    --profile=interactive

# Verify bytecode without executing
agos gvm verify --bytecode=analysis.agos

# Disassemble to human-readable listing
agos gvm disassemble --bytecode=analysis.agos

# Execute with full tracing
agos gvm run --bytecode=analysis.agos --trace=trace.json

# Run conformance tests
agos gvm test --suite=conformance-v1

# Benchmark GVM performance
agos gvm bench --bytecode=benchmark_set/*.agos

# Get GVM version information
agos gvm version
```

---

## 8. Explanation Engine Architecture

### 8.1 Overview

The Explanation Engine (MOD-11) transforms the deterministic `AnalysisResult` into human-readable explanations. It operates in three phases:

```
AnalysisResult
    │
    ▼
┌───────────────────────────────────────────────────────┐
│  PHASE 1: ANALYZE                                     │
│                                                       │
│  • Identify sentence type (jumlah fi'liyyah, etc.)    │
│  • Extract I'rab-relevant features from each token    │
│  • Identify notable grammatical constructions         │
│  • Generate overview summary                          │
│  • Collect flags with localized messages              │
│                                                       │
└───────────────────────────────────────────────────────┘
    │
    ▼
┌───────────────────────────────────────────────────────┐
│  PHASE 2: TEMPLATE                                    │
│                                                       │
│  • Select templates for current language              │
│  • Apply templates to each I'rab entry                │
│  • Apply templates to constructions                   │
│  • Apply templates to overview                        │
│  • Apply templates to flags                           │
│  • Plugin augmentation (custom explanations)          │
│                                                       │
└───────────────────────────────────────────────────────┘
    │
    ▼
┌───────────────────────────────────────────────────────┐
│  PHASE 3: RENDER                                      │
│                                                       │
│  • Render to JSON (structured data)                  │
│  • Render to Text (plain text I'rab)                  │
│  • Render to HTML (styled, with CSS classes)          │
│  • LLM enhancement (optional — augments text only)   │
│  • Assemble final ExplanationOutput                   │
│                                                       │
└───────────────────────────────────────────────────────┘
    │
    ▼
ExplanationOutput (IR-11)
```

### 8.2 Phase 1: Analysis

```rust
/// Analyze the AnalysisResult to extract explanation-relevant data.
fn analyze_for_explanation(
    result: &AnalysisResult,
    language_pack: &LanguagePack,
    template_registry: &TemplateRegistry,
) -> ExplanationAnalysis {
    // 1. Identify primary tree (highest confidence)
    let primary_tree = result.trees.first()
        .expect("At least one analysis tree must exist");

    // 2. Determine sentence type
    let sentence_type = identify_sentence_type(primary_tree);

    // 3. Build I'rab breakdown
    let irab_entries: Vec<IrabEntry> = primary_tree.tokens.iter()
        .map(|token| build_irab_entry(token, language_pack))
        .collect();

    // 4. Identify notable constructions
    let constructions = identify_constructions(primary_tree);

    // 5. Generate overview summary
    let overview = generate_overview(
        &irab_entries,
        &sentence_type,
        &constructions,
        language_pack,
    );

    // 6. Localize flags
    let flags = localize_flags(result, language_pack);

    ExplanationAnalysis {
        primary_tree_id: primary_tree.id.clone(),
        sentence_type,
        irab_entries,
        constructions,
        overview,
        flags,
    }
}

struct ExplanationAnalysis {
    primary_tree_id: String,
    sentence_type: Option<SentenceType>,
    irab_entries: Vec<IrabEntry>,
    constructions: Vec<Construction>,
    overview: String,
    flags: Vec<FlagDisplay>,
}
```

### 8.3 Sentence Type Identification

```rust
/// Identify the sentence type from the analysis tree.
fn identify_sentence_type(tree: &AnalysisTree) -> Option<SentenceType> {
    match tree.tree_type.as_str() {
        "jumlah_fi'liyyah" => Some(SentenceType::Verbal),
        "jumlah_ismiyyah" => Some(SentenceType::Nominal),
        "jumlah_shartiyyah" => Some(SentenceType::Conditional),
        "jumlah_zarfiyyah" => Some(SentenceType::Adverbial),
        "jumlah_kana" => Some(SentenceType::Kana),
        "jumlah_inna" => Some(SentenceType::Inna),
        "jumlah_qasam" => Some(SentenceType::Oath),
        _ => None,  // "incomplete" or "unknown"
    }
}

enum SentenceType {
    Verbal,        // jumlah fi'liyyah — starts with a verb
    Nominal,       // jumlah ismiyyah — starts with a noun
    Conditional,   // jumlah shartiyyah — contains in/ law/ etc.
    Adverbial,     // jumlah zarfiyyah — adverbial clause
    Kana,          // kana and her sisters
    Inna,          // inna and her sisters
    Oath,          // qasam — oath construction
}
```

### 8.4 I'rab Entry Generation

```rust
/// Build an I'rab entry for a single token.
fn build_irab_entry(token: &AnalysisToken, language_pack: &LanguagePack) -> IrabEntry {
    let mf = &token.features.morphological;
    let sf = &token.features.syntactic;

    // Build feature display list (sorted: inflectional → derivational → prosodic)
    let mut features = Vec::new();
    if let Some(gender) = &mf.gender {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("gender"),
            value: language_pack.feature_value("gender", gender),
            category: "inflectional".into(),
        });
    }
    if let Some(number) = &mf.number {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("number"),
            value: language_pack.feature_value("number", number),
            category: "inflectional".into(),
        });
    }
    if let Some(person) = &mf.person {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("person"),
            value: language_pack.feature_value("person", person),
            category: "inflectional".into(),
        });
    }
    if let Some(tense) = &mf.tense {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("tense"),
            value: language_pack.feature_value("tense", tense),
            category: "inflectional".into(),
        });
    }
    if let Some(mood) = &mf.mood {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("mood"),
            value: language_pack.feature_value("mood", mood),
            category: "inflectional".into(),
        });
    }
    if let Some(voice) = &mf.voice {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("voice"),
            value: language_pack.feature_value("voice", voice),
            category: "inflectional".into(),
        });
    }
    if let Some(case) = &mf.case {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("case"),
            value: language_pack.feature_value("case", case),
            category: "inflectional".into(),
        });
    }
    if let Some(state) = &mf.state {
        features.push(FeatureDisplay {
            name: language_pack.feature_name("state"),
            value: language_pack.feature_value("state", state),
            category: "inflectional".into(),
        });
    }

    // Generate natural language explanation
    let explanation = generate_token_explanation(token, &features, language_pack);

    IrabEntry {
        token: token.text.clone(),
        root: mf.root.clone(),
        pos: language_pack.pos_name(&mf.pos),
        features,
        syntactic_role: sf.role.as_ref().map(|r| language_pack.role_name(r)),
        explanation,
    }
}
```

### 8.5 Construction Identification

```rust
/// Identify notable grammatical constructions in the analysis tree.
fn identify_constructions(tree: &AnalysisTree) -> Vec<Construction> {
    let mut constructions = Vec::new();

    // Walk the constituent tree to find constructions
    for node in &tree.constituents {
        match node.role.as_str() {
            "idafa" => constructions.push(Construction {
                name: "Idafa (Construct State)".into(),
                description: "A possessive construction where two nouns are linked, \
                             the first in construct state and the second in the genitive case.".into(),
                tokens: node.token_ids.clone(),
            }),
            "na'at" => constructions.push(Construction {
                name: "Na'at (Adjective Agreement)".into(),
                description: "An adjective following a noun, agreeing in gender, \
                             number, case, and definiteness.".into(),
                tokens: node.token_ids.clone(),
            }),
            "tawkid" => constructions.push(Construction {
                name: "Tawkid (Emphasis)".into(),
                description: "An emphatic construction using nafs, 'ayn, or kull.".into(),
                tokens: node.token_ids.clone(),
            }),
            "badal" => constructions.push(Construction {
                name: "Badal (Apposition)".into(),
                description: "A noun following another noun, explaining or replacing it.".into(),
                tokens: node.token_ids.clone(),
            }),
            "istithna" => constructions.push(Construction {
                name: "Istithna (Exception)".into(),
                description: "An exceptive construction using illa, ghayr, or siwa.".into(),
                tokens: node.token_ids.clone(),
            }),
            "nida" => constructions.push(Construction {
                name: "Nida (Vocative)".into(),
                description: "A vocative construction using ya, a, or ayyuha.".into(),
                tokens: node.token_ids.clone(),
            }),
            "shart" => constructions.push(Construction {
                name: "Shart wa Jaza (Conditional)".into(),
                description: "A conditional sentence with condition and result clauses.".into(),
                tokens: node.token_ids.clone(),
            }),
            _ => {}  // Skip other roles
        }
    }

    constructions
}
```

### 8.6 Token Explanation Generation

```rust
/// Generate a natural language explanation for a single token's grammatical state.
fn generate_token_explanation(
    token: &AnalysisToken,
    features: &[FeatureDisplay],
    language_pack: &LanguagePack,
) -> String {
    let mf = &token.features.morphological;
    let sf = &token.features.syntactic;

    // Pattern-matched explanation based on POS and syntactic role
    match (mf.pos.as_str(), sf.role.as_deref()) {
        ("verb", Some("fi'l")) => {
            format!(
                "{} verb in the {} tense, {} voice, {} form. \
                 It agrees with a {} {} {} subject.",
                language_pack.feature_value("person", mf.person.as_deref().unwrap_or("third")),
                language_pack.feature_value("tense", mf.tense.as_deref().unwrap_or("past")),
                language_pack.feature_value("voice", mf.voice.as_deref().unwrap_or("active")),
                mf.verb_form.map(|f| format!("Form {}", f)).unwrap_or_default(),
                language_pack.feature_value("gender", mf.gender.as_deref().unwrap_or("masculine")),
                language_pack.feature_value("number", mf.number.as_deref().unwrap_or("singular")),
                language_pack.feature_value("person", mf.person.as_deref().unwrap_or("third")),
            )
        }
        ("verb", _) => {
            format!(
                "A {} verb in the {} tense, {} voice.",
                language_pack.feature_value("person", mf.person.as_deref().unwrap_or("third")),
                language_pack.feature_value("tense", mf.tense.as_deref().unwrap_or("past")),
                language_pack.feature_value("voice", mf.voice.as_deref().unwrap_or("active")),
            )
        }
        ("noun", Some(role)) => {
            let case_info = mf.case.as_ref()
                .map(|c| format!(" in the {} case", language_pack.feature_value("case", c)))
                .unwrap_or_default();
            let state_info = mf.state.as_ref()
                .map(|s| format!(", {}", language_pack.feature_value("state", s)))
                .unwrap_or_default();
            format!(
                "A {} {} noun{}{}. It functions as {}.",
                language_pack.feature_value("gender", mf.gender.as_deref().unwrap_or("masculine")),
                language_pack.feature_value("number", mf.number.as_deref().unwrap_or("singular")),
                case_info,
                state_info,
                language_pack.role_name(role),
            )
        }
        ("noun", None) => {
            format!(
                "A {} {} noun{}.",
                language_pack.feature_value("gender", mf.gender.as_deref().unwrap_or("masculine")),
                language_pack.feature_value("number", mf.number.as_deref().unwrap_or("singular")),
                mf.case.as_ref()
                    .map(|c| format!(" in the {} case", language_pack.feature_value("case", c)))
                    .unwrap_or_default(),
            )
        }
        ("particle", _) => {
            format!("A grammatical particle.")
        }
        ("pronoun", Some(role)) => {
            format!(
                "A {} {} pronoun functioning as {}.",
                language_pack.feature_value("person", mf.person.as_deref().unwrap_or("third")),
                language_pack.feature_value("number", mf.number.as_deref().unwrap_or("singular")),
                language_pack.role_name(role),
            )
        }
        ("pronoun", None) => {
            format!(
                "A {} {} pronoun.",
                language_pack.feature_value("person", mf.person.as_deref().unwrap_or("third")),
                language_pack.feature_value("number", mf.number.as_deref().unwrap_or("singular")),
            )
        }
        _ => {
            format!(
                "A {}.",
                language_pack.pos_name(&mf.pos),
            )
        }
    }
}
```

### 8.7 Overview Generation

```rust
/// Generate a grammatical overview/summary of the analysis.
fn generate_overview(
    irab_entries: &[IrabEntry],
    sentence_type: &Option<SentenceType>,
    constructions: &[Construction],
    language_pack: &LanguagePack,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Sentence type
    match sentence_type {
        Some(SentenceType::Verbal) => parts.push(
            "This is a verbal sentence (jumlah fi'liyyah) — it begins with a verb.".into()
        ),
        Some(SentenceType::Nominal) => parts.push(
            "This is a nominal sentence (jumlah ismiyyah) — it begins with a noun or pronoun.".into()
        ),
        Some(SentenceType::Conditional) => parts.push(
            "This is a conditional sentence (jumlah shartiyyah) — it contains a condition and a result.".into()
        ),
        Some(SentenceType::Adverbial) => parts.push(
            "This is an adverbial clause (jumlah zarfiyyah).".into()
        ),
        Some(SentenceType::Kana) => parts.push(
            "This sentence begins with kana or one of her sisters, which raises the subject \
             to the nominative and assigns the accusative to the predicate.".into()
        ),
        Some(SentenceType::Inna) => parts.push(
            "This sentence begins with inna or one of her sisters, which assigns the accusative \
             to the subject and the nominative to the predicate.".into()
        ),
        Some(SentenceType::Oath) => parts.push(
            "This is an oath construction (jumlah qasam).".into()
        ),
        None => parts.push(
            "The sentence type could not be definitively determined.".into()
        ),
    }

    // Word count
    let word_count = irab_entries.len();
    parts.push(format!(
        "It consists of {} word{}.", word_count,
        if word_count == 1 { "" } else { "s" },
    ));

    // Notable constructions
    if !constructions.is_empty() {
        let constr_names: Vec<&str> = constructions.iter().map(|c| c.name.as_str()).collect();
        parts.push(format!(
            "Notable grammatical constructions: {}.",
            constr_names.join(", "),
        ));
    }

    parts.join(" ")
}
```

---

## 9. Template System & Localization

### 9.1 Template Architecture

```
Language Packs (JSON)
│
├── en.json        (English)
├── ar.json        (Arabic)
├── ur.json        (Urdu)
├── ms.json        (Malay)
├── id.json        (Indonesian)
├── fr.json        (French)
├── tr.json        (Turkish)
└── ... (community contributions)
    │
    ▼
┌────────────────────────────────────┐
│  Template Engine                    │
│                                    │
│  • Compile templates on load       │
│  • Resolve variable placeholders   │
│  • Support pluralization rules     │
│  • Support gender agreement        │
│  • Plugin extension points         │
│  • Fallback chain: requested → en  │
└────────────────────────────────────┘
    │
    ▼
Localized ExplanationOutput
```

### 9.2 Language Pack Schema

```json
{
    "language": "en",
    "meta": {
        "name": "English",
        "native_name": "English",
        "direction": "ltr",
        "plural_forms": "one_other",
        "version": "1.0.0",
        "author": "AGOS Localization Team"
    },
    "labels": {
        "grammatical_analysis": "Grammatical Analysis",
        "irab_breakdown": "I'rab (إعراب) Breakdown",
        "sentence_type": "Sentence Type",
        "overview": "Overview",
        "constructions": "Grammatical Constructions",
        "flags": "Flags",
        "evidence": "Evidence Trail",
        "token": "Token",
        "root": "Root",
        "pos": "Part of Speech",
        "features": "Features",
        "syntactic_role": "Syntactic Role",
        "explanation": "Explanation"
    },
    "pos": {
        "verb": "Verb",
        "noun": "Noun",
        "particle": "Particle",
        "pronoun": "Pronoun",
        "adjective": "Adjective",
        "adverb": "Adverb",
        "preposition": "Preposition",
        "conjunction": "Conjunction",
        "proper_noun": "Proper Noun",
        "interrogative": "Interrogative",
        "unknown": "Unknown"
    },
    "features": {
        "gender": { "label": "Gender", "values": {
            "masculine": "Masculine",
            "feminine": "Feminine",
            "common": "Common"
        }},
        "number": { "label": "Number", "values": {
            "singular": "Singular",
            "dual": "Dual",
            "plural": "Plural"
        }},
        "person": { "label": "Person", "values": {
            "first": "First",
            "second": "Second",
            "third": "Third"
        }},
        "tense": { "label": "Tense", "values": {
            "past": "Past (الماضي)",
            "present": "Present (المضارع)",
            "imperative": "Imperative (الأمر)"
        }},
        "mood": { "label": "Mood", "values": {
            "indicative": "Indicative (الرفع)",
            "subjunctive": "Subjunctive (النصب)",
            "jussive": "Jussive (الجزم)",
            "energetic": "Energetic (التوكيد)"
        }},
        "voice": { "label": "Voice", "values": {
            "active": "Active (معلوم)",
            "passive": "Passive (مجھول)"
        }},
        "case": { "label": "Case", "values": {
            "nominative": "Nominative (الرفع)",
            "accusative": "Accusative (النصب)",
            "genitive": "Genitive (الجر)"
        }},
        "state": { "label": "State", "values": {
            "definite": "Definite (معرفة)",
            "indefinite": "Indefinite (نكرة)"
        }}
    },
    "roles": {
        "fi'l": "Verb (فعل)",
        "fa'il": "Subject (فاعل)",
        "mubtada": "Topic (مبتدأ)",
        "khabar": "Comment (خبر)",
        "maf'ul_bi-hi": "Direct Object (مفعول به)",
        "idafa": "Construct State (إضافة)",
        "mudaf": "First Term of Idafa (مضاف)",
        "mudaf_ilayh": "Second Term of Idafa (مضاف إليه)",
        "harf_jarr": "Preposition (حرف جر)",
        "majrur": "Governed by Preposition (مجرور)",
        "na'at": "Adjective/Qualifier (نعت)",
        "hal": "Circumstantial Accusative (حال)",
        "tamyiz": "Specification (تمييز)",
        "zarf": "Adverb of Time/Place (ظرف)",
        "ta'kid": "Emphasizer (توكيد)",
        "badal": "Apposition (بدل)",
        "istithna": "Exception (استثناء)",
        "nida": "Vocative (نداء)",
        "shart": "Condition (شرط)",
        "jaza": "Result (جزاء)"
    },
    "sentence_types": {
        "jumlah_fi'liyyah": "Verbal Sentence (جملة فعلية)",
        "jumlah_ismiyyah": "Nominal Sentence (جملة اسمية)",
        "jumlah_shartiyyah": "Conditional Sentence (جملة شرطية)",
        "jumlah_zarfiyyah": "Adverbial Clause (جملة ظرفية)"
    },
    "errors": {
        "SUBJECT_VERB_PERSON_MISMATCH": "The subject and verb do not agree in person.",
        "SUBJECT_VERB_GENDER_MISMATCH": "The subject and verb do not agree in gender.",
        "SUBJECT_VERB_NUMBER_MISMATCH": "The subject and verb might not agree in number in this construction.",
        "CASE_MISMATCH": "The expected case does not match the assigned case.",
        "MOOD_MISMATCH": "The verb mood does not match the governing particle's requirement.",
        "UNRESOLVABLE_AMBIGUITY": "This sentence has unresolvable grammatical ambiguity."
    }
}
```

### 9.3 Template Compilation

```rust
/// Compiled template for fast string generation.
struct CompiledTemplate {
    /// Static text segments and variable slots in order.
    segments: Vec<TemplateSegment>,
}

enum TemplateSegment {
    Text(String),
    Variable(String),           // Variable name to resolve
    Plural {                    // Pluralization: count variable
        count_var: String,
        one: String,
        other: String,
    },
    Conditional {               // Conditional inclusion
        var: String,
        equals: Option<String>,  // If None, checks for presence (non-null)
        template: Box<CompiledTemplate>,
        otherwise: Option<Box<CompiledTemplate>>,
    },
}

/// Template engine — compiles and renders templates.
struct TemplateEngine {
    templates: HashMap<String, CompiledTemplate>,
    language_packs: HashMap<String, LanguagePack>,
    default_language: String,
}

impl TemplateEngine {
    /// Compile a template string into an optimized execution form.
    fn compile(&mut self, name: &str, template_str: &str) -> Result<(), TemplateError> {
        // Parse template syntax: {variable}, {count:plural(one|other)}, {?var}...{/var}
        let segments = parse_template(template_str)?;
        self.templates.insert(name.to_string(), CompiledTemplate { segments });
        Ok(())
    }

    /// Render a compiled template with variable values.
    fn render(&self, name: &str, vars: &HashMap<String, String>, language: &str) -> Result<String, TemplateError> {
        let template = self.templates.get(name)
            .ok_or(TemplateError::NotFound(name.to_string()))?;
        let lang_pack = self.language_packs.get(language)
            .unwrap_or_else(|| self.language_packs.get(&self.default_language).unwrap());
        render_segments(&template.segments, vars, lang_pack)
    }

    /// Load a language pack from JSON.
    fn load_language_pack(&mut self, json: &str) -> Result<(), TemplateError> {
        let pack: LanguagePack = serde_json::from_str(json)?;
        self.language_packs.insert(pack.language.clone(), pack);
        Ok(())
    }
}
```

### 9.4 Fallback Chain

```
Requested language: "ms" (Malay)
    │
    ├── 1. Try "ms" → if loaded, use it
    │
    ├── 2. Try "id" → if loaded, use as close variant
    │
    ├── 3. Try "en" → always built-in, guaranteed available
    │
    └── 4. If all missing → ERROR (should never happen; English is always available)
```

---

## 10. LLM Integration

### 10.1 Design Principles

1. **LLM NEVER modifies the analysis.** The LLM receives the entire `AnalysisResult` as context. It generates explanatory text only. It MUST NOT modify, override, or reinterpret any part of the deterministic analysis.

2. **LLM is additive, not required.** If the LLM service is unavailable (timeout, error, rate limit), the Explanation Engine falls back to template-based explanations. The `llm_enhanced` field in the output metadata is set to `false`.

3. **LLM output is clearly marked.** Any text generated by the LLM is prefixed with a notice (e.g., "AI-generated explanation") to distinguish it from deterministic analysis.

4. **LLM receives structured data, not raw text.** The LLM prompt includes the structured `AnalysisResult` JSON, not the raw Arabic input. This prevents the LLM from performing its own (non-deterministic, potentially incorrect) analysis.

### 10.2 LLM Integration Flow

```
AnalysisResult (deterministic)
    │
    ▼
┌────────────────────────────────────────────┐
│  LLM Integration Layer                      │
│                                             │
│  1. BUILD PROMPT                            │
│     • System prompt (role + constraints)    │
│     • AnalysisResult as JSON context        │
│     • User request (language + format)      │
│                                             │
│  2. CALL LLM SERVICE                        │
│     • Send to configured provider           │
│     • Timeout: 5000 ms (configurable)       │
│     • Retry: 1 attempt on failure           │
│                                             │
│  3. PARSE & VALIDATE RESPONSE               │
│     • Parse LLM response text               │
│     • Validate response structure           │
│     • If invalid → fall back to templates   │
│                                             │
│  4. MERGE                                   │
│     • LLM text replaces template-generated  │
│       overview and explanations             │
│     • All structural data (I'rab, features, │
│       roles) remains from deterministic     │
│       analysis                              │
│     • Set llm_enhanced = true               │
│                                             │
└────────────────────────────────────────────┘
    │
    ▼
ExplanationOutput (with LLM-enhanced text)
```

### 10.3 Prompt Template

```text
System: You are an expert Arabic grammar teacher (nahw and sarf).
You receive a structured grammatical analysis and must generate
clear, pedagogically helpful explanations in {language}.

RULES:
1. NEVER contradict or modify the provided analysis.
2. Explain grammatical concepts in simple terms.
3. Use Arabic grammatical terminology alongside translations.
4. Keep explanations concise (2-4 sentences per token).
5. If the analysis has ambiguity, mention it neutrally.
6. Do not add grammatical information not present in the analysis.

User Request:
Language: {language}
Format: {format}

Analysis:
```json
{analysis_result_json}
```

Please generate:
1. A brief overview of the sentence's grammatical structure
2. For each word: an explanation of its grammatical state (I'rab)
3. Highlights of any notable grammatical constructions
```

### 10.4 Provider Interface

```rust
/// Abstract LLM provider interface.
trait LLMProvider {
    fn provider_name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;

    /// Send a prompt and receive a response.
    fn generate(
        &self,
        prompt: &LLMPrompt,
        config: &LLMConfig,
    ) -> Result<LLMResponse, LLMError>;
}

struct LLMPrompt {
    system: String,
    user: String,
    temperature: f64,
    max_tokens: u32,
}

struct LLMResponse {
    text: String,
    model: String,
    input_tokens: u32,
    output_tokens: u32,
    latency_ms: f64,
}

enum LLMError {
    ServiceUnavailable { provider: String, reason: String },
    Timeout { provider: String, duration_ms: u64 },
    RateLimited { provider: String, retry_after_ms: u64 },
    InvalidResponse { provider: String, details: String },
    ConfigurationError { message: String },
}

/// Built-in LLM provider implementations.
struct OpenAIProvider {
    api_key: String,
    base_url: String,
    http_client: HttpClient,
}

struct AnthropicProvider {
    api_key: String,
    base_url: String,
    http_client: HttpClient,
}

/// No-op provider for when LLM is disabled.
struct NoopProvider;
impl LLMProvider for NoopProvider {
    fn generate(&self, _: &LLMPrompt, _: &LLMConfig) -> Result<LLMResponse, LLMError> {
        Err(LLMError::ServiceUnavailable {
            provider: "noop".into(),
            reason: "LLM enhancement is disabled".into(),
        })
    }
}
```

### 10.5 LLM Output Validation

```rust
/// Validate LLM output to ensure it doesn't contradict or modify the analysis.
fn validate_llm_output(
    llm_text: &str,
    original_analysis: &AnalysisResult,
) -> Result<(), LLMError> {
    // 1. Check that the LLM didn't hallucinate token counts
    let token_count_in_text = count_token_references(llm_text);
    if token_count_in_text > original_analysis.trees[0].tokens.len() * 2 {
        return Err(LLMError::InvalidResponse {
            provider: "llm".into(),
            details: "LLM mentioned more tokens than exist in the analysis".into(),
        });
    }

    // 2. Check that the LLM didn't fabricate grammatical terms
    let known_terms = load_known_grammatical_terms();
    let unknown_terms = find_unknown_terms(llm_text, &known_terms);
    if !unknown_terms.is_empty() {
        // Log warning but don't reject — the LLM may use valid pedagogical language
        log::warn!("LLM used potentially unknown terms: {:?}", unknown_terms);
    }

    // 3. Check that the LLM didn't contradict the analysis
    // (e.g., saying "this verb is past tense" when analysis says "present")
    let contradictions = find_contradictions(llm_text, original_analysis);
    if !contradictions.is_empty() {
        return Err(LLMError::InvalidResponse {
            provider: "llm".into(),
            details: format!("LLM contradicted analysis on: {:?}", contradictions),
        });
    }

    Ok(())
}
```

---

## 11. Output Formatting & Rendering

### 11.1 Format Support Matrix

| Format | Use Case | Size (10-word sentence) | Generation Time |
|--------|----------|------------------------|-----------------|
| **JSON** | API consumers, programmatic access | 5–20 KB | < 100 μs |
| **Text** | CLI output, simple display | 0.5–3 KB | < 200 μs |
| **HTML** | Web display, rich formatting | 3–10 KB | < 300 μs |
| **PDF** | Printable reports, educational materials | 50–200 KB | 1–5 s (with PDF lib) |

### 11.2 JSON Renderer

```rust
/// Render ExplanationOutput as JSON (the canonical serialization format).
fn render_json(output: &ExplanationOutput) -> Result<String, RenderError> {
    serde_json::to_string_pretty(output)
        .map_err(|e| RenderError::SerializationFailed(e.to_string()))
}
```

### 11.3 Text Renderer

```rust
/// Render ExplanationOutput as plain text (I'rab-style output).
fn render_text(output: &ExplanationOutput) -> Result<String, RenderError> {
    let mut lines = Vec::new();

    // Header
    lines.push("═".repeat(60));
    lines.push(format!("  {}", output.overview));
    lines.push("═".repeat(60));
    lines.push("");

    // Sentence type
    if let Some(ref st) = output.sentence_type {
        lines.push(format!("Sentence Type: {}", st));
        lines.push("");
    }

    // I'rab breakdown
    lines.push("I'rab (إعراب) Breakdown:");
    lines.push("─".repeat(60));

    for entry in &output.irab_breakdown {
        lines.push(format!("  {} — {}", entry.token, entry.pos));
        if let Some(ref root) = entry.root {
            lines.push(format!("    Root: {}", root));
        }
        if let Some(ref role) = entry.syntactic_role {
            lines.push(format!("    Role: {}", role));
        }
        for feature in &entry.features {
            lines.push(format!("    {}: {}", feature.name, feature.value));
        }
        lines.push(format!("    → {}", entry.explanation));
        lines.push("");
    }

    // Constructions
    if !output.constructions.is_empty() {
        lines.push("Notable Constructions:");
        lines.push("─".repeat(60));
        for constr in &output.constructions {
            lines.push(format!("  • {} — {}", constr.name, constr.description));
        }
        lines.push("");
    }

    // Flags
    if !output.flags.is_empty() {
        lines.push("Grammatical Flags:");
        lines.push("─".repeat(60));
        for flag in &output.flags {
            let icon = match flag.flag_type.as_str() {
                "error" => "✗",
                "warning" => "⚠",
                _ => "ℹ",
            };
            lines.push(format!("  {} [{}] {}", icon, flag.code, flag.message));
        }
    }

    Ok(lines.join("\n"))
}
```

**Example text output:**

```
════════════════════════════════════════════════════════════
  This is a verbal sentence (jumlah fi'liyyah) — it begins
  with a verb. It consists of 3 words.
════════════════════════════════════════════════════════════

Sentence Type: Verbal Sentence (جملة فعلية)

I'rab (إعراب) Breakdown:
────────────────────────────────────────────────────────────
  كَتَبَ — Verb
    Root: كتب
    Role: Verb (فعل)
    Gender: Masculine
    Number: Singular
    Person: Third
    Tense: Past (الماضي)
    Voice: Active (معلوم)
    → Third person masculine singular verb in the past
      tense, active voice, Form I.

  مُحَمَّدٌ — Proper Noun
    Root: حمد
    Role: Subject (فاعل)
    Gender: Masculine
    Number: Singular
    Case: Nominative (الرفع)
    State: Indefinite (نكرة)
    → A masculine singular proper noun in the nominative
      case, indefinite. It functions as the subject.

  رِسَالَةً — Noun
    Root: ر س ل
    Role: Direct Object (مفعول به)
    Gender: Feminine
    Number: Singular
    Case: Accusative (النصب)
    State: Indefinite (نكرة)
    → A feminine singular noun in the accusative case,
      indefinite. It functions as the direct object.

Notable Constructions:
────────────────────────────────────────────────────────────
  • Verbal Sentence — A verbal sentence beginning with a
    past tense verb, followed by its subject and object.
```

### 11.4 HTML Renderer

```rust
/// Render ExplanationOutput as styled HTML.
fn render_html(output: &ExplanationOutput) -> Result<String, RenderError> {
    let mut html = String::new();

    html.push_str(r#"<!DOCTYPE html><html lang=""#);
    html.push_str(&output.metadata.language);
    html.push_str(r#""><head><meta charset="UTF-8"><style>"#);
    html.push_str(INLINE_CSS);  // Embedded CSS for standalone HTML
    html.push_str(r#"</style></head><body><div class="agos-explanation">"#);

    // Overview
    html.push_str(&format!(
        r#"<div class="overview"><h2>Grammatical Analysis</h2><p>{}</p></div>"#,
        escape_html(&output.overview)
    ));

    // I'rab table
    html.push_str(r#"<table class="irab-table"><thead><tr>"#);
    html.push_str(r#"<th>Token</th><th>Root</th><th>POS</th>"#);
    html.push_str(r#"<th>Role</th><th>Features</th><th>Explanation</th>"#);
    html.push_str(r#"</tr></thead><tbody>"#);

    for entry in &output.irab_breakdown {
        html.push_str("<tr>");
        html.push_str(&format!(r#"<td class="token">{}</td>"#, escape_html(&entry.token)));
        html.push_str(&format!(r#"<td>{}</td>"#,
            entry.root.as_ref().map_or("-", |r| r.as_str())));
        html.push_str(&format!(r#"<td><span class="pos">{}</span></td>"#, escape_html(&entry.pos)));
        html.push_str(&format!(r#"<td>{}</td>"#,
            entry.syntactic_role.as_ref().map_or("-", |r| r.as_str())));
        html.push_str(r#"<td class="features">"#);
        for f in &entry.features {
            html.push_str(&format!(
                r#"<span class="feature {}">{}</span>"#,
                f.category, escape_html(&f.value)
            ));
        }
        html.push_str("</td>");
        html.push_str(&format!(r#"<td class="explanation">{}</td>"#, escape_html(&entry.explanation)));
        html.push_str("</tr>");
    }

    html.push_str(r#"</tbody></table>"#);

    // Constructions
    if !output.constructions.is_empty() {
        html.push_str(r#"<div class="constructions"><h3>Notable Constructions</h3><ul>"#);
        for c in &output.constructions {
            html.push_str(&format!(
                r#"<li><strong>{}</strong>: {}</li>"#,
                escape_html(&c.name), escape_html(&c.description)
            ));
        }
        html.push_str(r#"</ul></div>"#);
    }

    // Flags
    if !output.flags.is_empty() {
        html.push_str(r#"<div class="flags"><h3>Flags</h3><ul>"#);
        for f in &output.flags {
            let cls = f.flag_type.as_str(); // "error", "warning", "info"
            html.push_str(&format!(
                r#"<li class="flag {}"><strong>{}</strong>: {}</li>"#,
                cls, escape_html(&f.code), escape_html(&f.message)
            ));
        }
        html.push_str(r#"</ul></div>"#);
    }

    // LLM notice
    if output.metadata.llm_enhanced {
        html.push_str(r#"<div class="llm-notice">AI-enhanced explanations</div>"#);
    }

    html.push_str(r#"</div></body></html>"#);

    Ok(html)
}

const INLINE_CSS: &str = r#"
body { font-family: 'Segoe UI', system-ui, sans-serif; max-width: 900px; margin: 2em auto; padding: 0 1em; color: #1a1a2e; }
.agos-explanation { background: #f8f9fa; border-radius: 12px; padding: 2em; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.overview { margin-bottom: 2em; }
.overview h2 { color: #1a1a2e; border-bottom: 2px solid #e94560; padding-bottom: 0.3em; }
.irab-table { width: 100%; border-collapse: collapse; margin: 1em 0; }
.irab-table th { background: #1a1a2e; color: white; padding: 0.75em 1em; text-align: left; font-size: 0.85em; text-transform: uppercase; letter-spacing: 0.05em; }
.irab-table td { padding: 0.75em 1em; border-bottom: 1px solid #dee2e6; vertical-align: top; }
.irab-table tr:hover { background: #e9ecef; }
.token { font-size: 1.4em; direction: rtl; font-family: 'Traditional Arabic', 'Scheherazade New', serif; }
.pos { background: #e94560; color: white; padding: 0.15em 0.5em; border-radius: 4px; font-size: 0.85em; white-space: nowrap; }
.features { max-width: 200px; }
.feature { display: inline-block; background: #e2e8f0; padding: 0.1em 0.4em; border-radius: 3px; margin: 0.1em; font-size: 0.85em; }
.feature.inflectional { background: #dbeafe; }
.feature.derivational { background: #fef3c7; }
.explanation { font-size: 0.9em; color: #4a5568; }
.constructions h3, .flags h3 { color: #1a1a2e; margin-top: 1.5em; }
.constructions li { margin: 0.5em 0; }
.flag.error { color: #dc2626; }
.flag.warning { color: #d97706; }
.flag.info { color: #2563eb; }
.llm-notice { margin-top: 2em; padding: 0.5em 1em; background: #fef3c7; border-radius: 6px; font-size: 0.85em; color: #92400e; text-align: center; }
"#;
```

---

## 12. Performance & Optimization

### 12.1 Performance Targets

From SPEC-0001-C9:

#### MOD-10: GVM

| Metric | Target | Condition |
|--------|--------|-----------|
| **Latency (p50)** | < 1 ms per sentence | 10-word sentence, interpreted |
| **Latency (p99)** | < 5 ms per sentence | 30-word sentence, full evidence |
| **Throughput** | > 2K sentences/s | Single core, Server profile |
| **Execution speed** | > 100K instructions/s | Interpreted |
| **Step limit** | 100,000 | Server profile (default) |
| **Memory per instance** | < 200 KB | Default capacities |
| **Cache hit latency** | < 0.5 μs | Cache hit (bytecode → result) |
| **Startup time** | < 1 ms | First execution from cold pool |

#### MOD-11: ExplanationEngine

| Metric | Target | Condition |
|--------|--------|-----------|
| **Latency (p50) — template** | < 100 μs | 10-word sentence, JSON output |
| **Latency (p99) — template** | < 500 μs | 30-word sentence, HTML output |
| **Latency — LLM enhanced** | +500–2,000 ms | With LLM service call |
| **Throughput — template** | > 10K sentences/s | Single core |
| **Language pack load** | < 50 ms per language | At initialization |
| **Template compile** | < 10 ms | 50 templates |
| **Memory per language pack** | ~200 KB | Full pack (English) |

#### Combined Runtime

| Metric | Target | Condition |
|--------|--------|-----------|
| **End-to-end (p50)** | < 1.5 ms | Template-based, no LLM |
| **End-to-end (p99)** | < 10 ms | Template-based, no LLM |
| **End-to-end (LLM)** | < 2,500 ms | With LLM enhancement |
| **Throughput** | > 1,500 sentences/s | Single core, template-based |
| **Cache hit (full runtime)** | < 2 μs | Both MOD-10 and MOD-11 cached |

### 12.2 Optimization Strategies

#### GVM Optimizations

| Optimization | Technique | Expected Impact |
|-------------|-----------|-----------------|
| **Instance pooling** | Pre-allocate GVM instances; avoid malloc/free per request | 5–10× improvement in p50 latency |
| **Instruction fusion** | Common sequences (LOAD_TOKEN + TOKEN_GET_FEATURES) fused into single opcode | 10–20% fewer instructions |
| **Hot-path inlining** | Most common instruction paths (feature get/set, token load) inlined in dispatch | 5–15% faster execution |
| **String table direct indexing** | String references use direct pointer to string data (not hash lookup) | 2–5× faster string access |
| **Jump cache** | Cache resolved jump targets to avoid re-decoding | 5–10% faster branch-heavy bytecode |
| **Early HALT detection** | Detect when all tokens processed and no more rules can fire | 10–30% fewer instructions for simple sentences |

#### Explanation Engine Optimizations

| Optimization | Technique | Expected Impact |
|-------------|-----------|-----------------|
| **Lazy rendering** | Only render the requested output format; skip others | Avoids up to 3× unnecessary work |
| **Template caching** | Compiled templates cached in TemplateRegistry; no re-parsing | 10–100× faster template application |
| **Language pack memoization** | Feature/role name lookups cached per session | 2–5× faster I'rab generation |
| **HTML fragment caching** | Common HTML fragments (table headers, CSS) pre-rendered | 10–20% faster HTML generation |
| **Parallel rendering (future)** | I'rab entries rendered in parallel for long sentences | 2–4× faster for 50+ word sentences |

### 12.3 Memory Budget

```
Grammar Runtime Memory (per server instance)
│
├── GVM Instance Pool (16 instances × 200 KB)        3.2 MB
├── Language Packs (5 languages × 200 KB)            1.0 MB
├── Template Registry (50 compiled templates)         0.1 MB
├── Plugin Instances (2 plugins × 1 MB)              2.0 MB
├── Working Set (per-request, peak)                   0.5 MB
├── Bytecode Cache (256 entries × 5 KB)              1.3 MB
│
├── Static Code (compiled binary footprint)           10 MB
│
└── Total (approximate, server deployment)           ~18 MB
```

### 12.4 Caching Strategy

```rust
/// Cache keys and value types for the Grammar Runtime.
enum RuntimeCacheKey {
    /// Key: hash(bytecode + GVMConfig)
    /// Value: AnalysisResult
    /// TTL: configurable (default: 3600s), invalidated on bytecode source change
    GVMExecution {
        bytecode_hash: String,
        config_hash: String,
    },

    /// Key: hash(AnalysisResult + ExplainConfig.language + ExplainConfig.format)
    /// Value: ExplanationOutput
    /// TTL: configurable (default: 3600s), invalidated on AnalysisResult change
    ExplanationGeneration {
        result_hash: String,
        language: String,
        format: String,
        llm_enabled: bool,
    },
}

/// Cache hit ratio targets:
/// GVM execution cache:     > 90% (many repeated bytecodes)
/// Explanation cache:       > 85% (same result → same explanation)
/// Combined full pipeline:  > 80% (bytecode + explanation)

fn build_runtime_cache_key(
    bytecode: &GrammarBytecode,
    gvm_config: &GVMConfig,
    explain_config: &ExplainConfig,
) -> (RuntimeCacheKey, RuntimeCacheKey) {
    let bytecode_hash = sha256(&bytecode.raw);
    let config_hash = sha256(&serde_json::to_vec(gvm_config).unwrap());

    let gvm_key = RuntimeCacheKey::GVMExecution {
        bytecode_hash: bytecode_hash.clone(),
        config_hash,
    };

    let result_hash = sha256(&bytecode_hash); // Predict the AnalysisResult hash
    let explain_key = RuntimeCacheKey::ExplanationGeneration {
        result_hash,
        language: explain_config.language.clone(),
        format: format!("{:?}", explain_config.format),
        llm_enabled: explain_config.enable_llm,
    };

    (gvm_key, explain_key)
}
```

---

## 13. Testing Strategy

### 13.1 Test Categories

| Category | Tests | Scope | Target Coverage |
|----------|-------|-------|-----------------|
| **Unit** | 200+ | Individual GVM instructions, template functions, I'rab generators | 95%+ |
| **Integration** | 100+ | Full bytecode → AnalysisResult → ExplanationOutput pipeline | 90%+ |
| **Conformance** | 163+ | GVM conformance (per RFC-0003 §11) | 100% of spec |
| **Localization** | 50+ | All language packs, all template strings | 100% of keys |
| **LLM Integration** | 20+ | Prompt generation, response validation, fallback behavior | 90%+ |
| **Performance** | 30+ | Latency budgets, throughput, memory limits | Pass/fail per target |

### 13.2 GVM Conformance Tests

From RFC-0003 §11, the GVM MUST pass 163+ conformance tests covering:

| Category | Tests | Description |
|----------|-------|-------------|
| Flow Control | 20 | HALT, JUMP, CALL/RETURN, conditional jumps, DIE |
| Stack Operations | 15 | PUSH all types, POP, DUP, SWAP, PUSH_NULL |
| Token Operations | 15 | LOAD_TOKEN, TOKEN_COUNT, TOKEN_ITERATE, TOKEN_GET_TEXT |
| Feature Operations | 20 | FEATURE_GET/SET/HAS, FEATURE_COMPARE_EQ/MASK, FEATURE_PACK |
| Constituent Operations | 15 | CONST_MAKE, CONST_ADD_CHILD, CONST_TRAVERSE |
| Rule Operations | 10 | RULE_APPLY, RULE_CONFIRM, RULE_REJECT, RULE_MODIFY |
| Evidence Operations | 8 | EVIDENCE_PUSH, EVIDENCE_QUERY, EVIDENCE_EMIT |
| Output Operations | 10 | OUTPUT_SET_METADATA, OUTPUT_ADD_TREE, OUTPUT_FINALIZE |
| Bounds Checking | 25 | Every bounds-check error condition |
| Error Handling | 20 | Every error code |
| Cross-Implementation | 5 | Same output across Rust, C, Python, JS |

### 13.3 Integration Test Fixtures

```rust
/// Integration test scenario: full runtime pipeline.
struct RuntimeTestScenario {
    name: &'static str,
    description: &'static str,
    bytecode: &'static [u8],             // Pre-generated .agos bytecode
    input_text: &'static str,             // Original Arabic text
    gvm_config: GVMConfig,
    explain_config: ExplainConfig,
    expected: ExpectedRuntimeOutput,
}

struct ExpectedRuntimeOutput {
    // GVM expectations
    gvm_status: ExpectedStatus,           // completed | error | timeout
    min_trees: usize,
    max_trees: usize,
    token_count: usize,
    expected_sentence_types: Vec<&'static str>,

    // Explanation expectations
    expected_constructions: Vec<&'static str>,
    expected_flags: Vec<&'static str>,
    must_contain: Vec<&'static str>,      // Substrings in explanation text
    must_not_contain: Vec<&'static str>,
}

/// Example test scenarios:
const RUNTIME_TEST_SCENARIOS: &[RuntimeTestScenario] = &[
    RuntimeTestScenario {
        name: "verbal-sentence-past",
        description: "Simple past tense verbal sentence: kataba Muhammadun risalatan",
        bytecode: include_bytes!("fixtures/verbal-past.agos"),
        input_text: "كتب محمد رسالة",
        gvm_config: GVMConfig::default(),
        explain_config: ExplainConfig { language: "en".into(), format: OutputFormat::Text, ..Default::default() },
        expected: ExpectedRuntimeOutput {
            gvm_status: ExpectedStatus::Completed,
            min_trees: 1,
            max_trees: 1,
            token_count: 3,
            expected_sentence_types: vec!["jumlah_fi'liyyah"],
            expected_constructions: vec!["Verbal Sentence"],
            expected_flags: vec![],
            must_contain: vec!["verb", "past", "subject", "object"],
            must_not_contain: vec!["error", "ambiguous"],
        },
    },
    RuntimeTestScenario {
        name: "nominal-sentence-mubtada-khabar",
        description: "Nominal sentence: al-baytu kabirun",
        bytecode: include_bytes!("fixtures/nominal-mubtada.agos"),
        input_text: "البيت كبير",
        gvm_config: GVMConfig::default(),
        explain_config: ExplainConfig { language: "en".into(), format: OutputFormat::Text, ..Default::default() },
        expected: ExpectedRuntimeOutput {
            gvm_status: ExpectedStatus::Completed,
            min_trees: 1,
            max_trees: 2,
            token_count: 2,
            expected_sentence_types: vec!["jumlah_ismiyyah"],
            expected_constructions: vec!["Nominal Sentence"],
            expected_flags: vec![],
            must_contain: vec!["nominal sentence", "topic", "comment"],
            must_not_contain: vec!["error"],
        },
    },
    RuntimeTestScenario {
        name: "idafa-construction",
        description: "Idafa construction: kitabu al-mudarrisi",
        bytecode: include_bytes!("fixtures/idafa.agos"),
        input_text: "كتاب المدرس",
        gvm_config: GVMConfig::default(),
        explain_config: ExplainConfig { language: "en".into(), format: OutputFormat::Text, ..Default::default() },
        expected: ExpectedRuntimeOutput {
            gvm_status: ExpectedStatus::Completed,
            min_trees: 1,
            max_trees: 1,
            token_count: 2,
            expected_sentence_types: vec!["jumlah_ismiyyah"],
            expected_constructions: vec!["Idafa"],
            expected_flags: vec![],
            must_contain: vec!["construct state", "genitive"],
            must_not_contain: vec![],
        },
    },
    RuntimeTestScenario {
        name: "ambiguous-sentence",
        description: "Ambiguous sentence: daraba musa eesa",
        bytecode: include_bytes!("fixtures/ambiguous.agos"),
        input_text: "ضرب موسى عيسى",
        gvm_config: GVMConfig::default(),
        explain_config: ExplainConfig { language: "en".into(), format: OutputFormat::Text, ..Default::default() },
        expected: ExpectedRuntimeOutput {
            gvm_status: ExpectedStatus::Completed,
            min_trees: 2,              // Two possible parses
            max_trees: 4,
            token_count: 3,
            expected_sentence_types: vec!["jumlah_fi'liyyah"],
            expected_constructions: vec![],
            expected_flags: vec!["AMBIGUOUS_ROLE"],
            must_contain: vec!["ambiguity", "could be"],
            must_not_contain: vec![],
        },
    },
    RuntimeTestScenario {
        name: "llm-enhanced-explanation",
        description: "LLM-enhanced explanation for simple sentence",
        bytecode: include_bytes!("fixtures/simple-past.agos"),
        input_text: "ذهب الولد إلى المدرسة",
        gvm_config: GVMConfig { performance_profile: Profile::Interactive, ..Default::default() },
        explain_config: ExplainConfig {
            language: "en".into(),
            format: OutputFormat::Text,
            enable_llm: true,
            llm: Some(LLMConfig {
                provider: "openai".into(),
                model: "gpt-4".into(),
                temperature: 0.3,
                max_tokens: 500,
                timeout_ms: 5000,
            }),
        },
        expected: ExpectedRuntimeOutput {
            gvm_status: ExpectedStatus::Completed,
            min_trees: 1,
            max_trees: 1,
            token_count: 5,
            expected_sentence_types: vec!["jumlah_fi'liyyah"],
            expected_constructions: vec!["Verbal Sentence"],
            expected_flags: vec![],
            must_contain: vec!["went", "boy", "school"],
            must_not_contain: vec![],
        },
    },
];
```

### 13.4 Performance Tests

```rust
/// Performance test: verify GVM execution meets latency targets.
#[test]
fn test_gvm_latency_target() {
    let bytecode = load_bytecode("fixtures/standard_10_word.agos");
    let config = GVMConfig {
        performance_profile: Profile::Interactive,
        ..Default::default()
    };

    // Warmup
    for _ in 0..100 {
        let _ = execute(&bytecode, &config).unwrap();
    }

    // Benchmark
    let mut times = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let start = Instant::now();
        let _ = execute(&bytecode, &config).unwrap();
        times.push(start.elapsed());
    }

    times.sort();
    let p50 = times[times.len() / 2];
    let p99 = times[(times.len() as f64 * 0.99) as usize];

    assert!(p50 < Duration::from_micros(1000), "p50 latency: {:?}", p50);
    assert!(p99 < Duration::from_micros(5000), "p99 latency: {:?}", p99);
}

/// Performance test: verify explanation generation meets latency targets.
#[test]
fn test_explanation_latency_target() {
    let analysis = load_analysis("fixtures/standard_analysis.json");
    let engine = ExplanationEngine::new(load_default_templates());
    let config = ExplainConfig {
        language: "en".into(),
        format: OutputFormat::Text,
        ..Default::default()
    };

    // Warmup
    for _ in 0..100 {
        let _ = engine.explain(&analysis, &config).unwrap();
    }

    // Benchmark
    let mut times = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let start = Instant::now();
        let _ = engine.explain(&analysis, &config).unwrap();
        times.push(start.elapsed());
    }

    times.sort();
    let p50 = times[times.len() / 2];
    let p99 = times[(times.len() as f64 * 0.99) as usize];

    assert!(p50 < Duration::from_micros(100), "p50 latency: {:?}", p50);
    assert!(p99 < Duration::from_micros(500), "p99 latency: {:?}", p99);
}
```

### 13.5 Localization Tests

```rust
/// Verify all language packs have all required keys.
#[test]
fn test_language_pack_completeness() {
    let reference_keys = get_required_language_keys();
    let packs = load_all_language_packs();

    for (lang, pack) in &packs {
        let pack_keys = collect_all_keys(pack);
        let missing: Vec<&str> = reference_keys.iter()
            .filter(|k| !pack_keys.contains(*k))
            .collect();
        assert!(missing.is_empty(),
            "Language pack '{}' is missing {} keys: {:?}",
            lang, missing.len(), &missing[..5.min(missing.len())]);
    }
}

/// Verify all template strings compile and render correctly.
#[test]
fn test_template_rendering() {
    let mut engine = TemplateEngine::new();
    engine.load_language_pack(include_str!("languages/en.json")).unwrap();

    // Compile all built-in templates
    for (name, template_str) in get_built_in_templates() {
        engine.compile(name, template_str).unwrap();
    }

    // Render with sample variables
    let vars = hashmap! {
        "gender" => "masculine",
        "number" => "singular",
        "tense" => "past",
    };
    let rendered = engine.render("token_explanation_verb", &vars, "en").unwrap();
    assert!(!rendered.is_empty(), "Template rendered to empty string");
    assert!(rendered.contains("masculine"), "Template should contain resolved variable");
}
```

---

## 14. Implementation Guidance

### 14.1 Recommended Implementation Order

```
Phase 1: GVM Foundation
├── Step 1: Implement bytecode loader & verifier
│   └── Parse header, section table, CRC32C checksums
├── Step 2: Implement stack & memory model
│   └── Operand stack, call stack, typed regions, bounds checking
├── Step 3: Implement instruction decoder
│   └── Opcode dispatch table, operand decoder
│
Phase 2: GVM Core Execution
├── Step 4: Implement flow control (HALT, JUMP, CALL, RETURN)
├── Step 5: Implement stack operations (PUSH, POP, DUP, SWAP)
├── Step 6: Implement token & feature operations
├── Step 7: Implement constituent operations
├── Step 8: Implement rule & evidence operations
├── Step 9: Implement output operations & AnalysisResult assembly
│
Phase 3: GVM Production Readiness
├── Step 10: Implement instance pool management
├── Step 11: Implement tracing & diagnostics
├── Step 12: Implement disassembler
├── Step 13: Write conformance tests (~163 tests)
│
Phase 4: Explanation Engine
├── Step 14: Implement language pack loader
├── Step 15: Implement template engine (compile + render)
├── Step 16: Implement I'rab generation & sentence type identification
├── Step 17: Implement construction identification
├── Step 18: Implement overview generation
├── Step 19: Implement output formatters (JSON, Text, HTML, PDF)
│
Phase 5: LLM Integration & Polish
├── Step 20: Implement LLM provider interface & prompt builder
├── Step 21: Implement OpenAI & Anthropic providers
├── Step 22: Implement LLM output validation
├── Step 23: Write integration & performance tests
```

### 14.2 Language-Specific Guidance

#### Rust (Primary Implementation)

```rust
// GVM core as a trait
pub trait GrammarVirtualMachine: Send {
    fn execute(&mut self, bytecode: &GrammarBytecode, config: &GVMConfig)
        -> Result<AnalysisResult, GVMError>;
    fn verify(&self, bytecode: &GrammarBytecode) -> VerificationResult;
    fn version() -> GVMVersion;
}

// Memory-safe region access using Rust's type system
#[derive(Clone)]
pub struct Region<T: Copy> {
    data: Vec<UnsafeCell<T>>,
    size: AtomicU32,
    capacity: u32,
}

impl<T: Copy> Region<T> {
    pub fn read(&self, index: u32) -> Result<T, GVMError> {
        if index >= self.capacity {
            return Err(GVMError::TokenIndexOutOfBounds { ... });
        }
        unsafe { Ok(*self.data[index as usize].get()) }
    }

    pub fn write(&self, index: u32, value: T) -> Result<(), GVMError> {
        if index >= self.capacity {
            return Err(GVMError::TokenIndexOutOfBounds { ... });
        }
        unsafe { *self.data[index as usize].get() = value; }
        self.size.fetch_max(index + 1, Ordering::Relaxed);
        Ok(())
    }
}

// Explanation Engine
pub struct ExplanationEngine {
    template_engine: TemplateEngine,
    language_packs: HashMap<String, LanguagePack>,
    llm_providers: HashMap<String, Box<dyn LLMProvider>>,
    plugins: Vec<Box<dyn ExplanationPlugin>>,
}

impl ExplanationEngine {
    pub fn explain(&self, analysis: &AnalysisResult, config: &ExplainConfig)
        -> Result<ExplanationOutput, ExplanationError>
    {
        let lang_pack = self.get_language_pack(&config.language)?;
        let analysis_data = self.analyze(analysis, lang_pack);
        let mut output = self.render(&analysis_data, config, lang_pack)?;

        if config.enable_llm {
            if let Some(llm_result) = self.enhance_with_llm(analysis, config) {
                output = self.merge_llm_output(output, llm_result);
                output.metadata.llm_enhanced = true;
            }
        }

        Ok(output)
    }
}
```

#### C (Secondary Implementation)

- Use `enum` + `union` for `Value` type (tagged union).
- Use fixed-size arrays (not `malloc`) for memory regions.
- Implement bounds checking with explicit `if` guards.
- Use `setjmp`/`longjmp` for error handling or explicit error propagation.
- Provide a C ABI entry point: `agos_gvm_execute(const uint8_t* bytecode, size_t len, ...)`.

#### Python (Ecosystem Implementation)

- Use `dataclasses` for all data structures.
- Use `match`/`case` for instruction dispatch (Python 3.10+).
- Performance: 10–100× slower than Rust. Suitable for prototyping and education.
- Memory regions: `array('Q')` for feature bitfields, `list` for others.

#### JavaScript/TypeScript (Ecosystem Implementation)

- Use `TypedArray` (Uint8Array, BigUint64Array) for memory regions.
- Instruction dispatch via `switch` or lookup table.
- Web Workers for concurrent GVM instances.
- WASM-based GVM compiled from Rust for production browser use.

### 14.3 Implementation Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Instruction dispatch performance** | Medium | High | Use flat opcode table (not match/if-else chain); inline hot paths |
| **Memory region exhaustion in production** | Low | High | Set generous defaults (1024 tokens, 2048 constituents); monitor usage |
| **LLM service flakiness** | Medium | Low | Always have template fallback; configurable timeouts/retries |
| **Language pack translation gaps** | Medium | Medium | CI check for completeness; fallback to English for missing keys |
| **Cross-platform endianness** | Low | Medium | Enforce little-endian in bytecode spec; test on ARM (both endiannesses) |
| **Template syntax errors** | Low | Medium | Compile all templates at initialization; fail fast with clear error |
| **GVM instruction count explosion** | Low | Medium | Profile worst-case bytecodes; set conservative step limits |
| **HTML injection via token text** | Low | Medium | Always escape HTML entities in token text before rendering |
| **Pool starvation under high concurrency** | Medium | Medium | Dynamic pool resizing; queue requests when pool is empty |
| **Bytecode version drift** | Medium | Medium | Strict version check at load time; migration tool for breaking changes |

### 14.4 Design Rules Summary

```
1. GVM instances MUST be isolated — no shared mutable state.
2. Every memory access MUST be bounds-checked.
3. Every instruction MUST have a documented stack effect.
4. Every error MUST produce a structured GVMError with recovery hint.
5. Template-based explanations MUST always be available (no LLM dependency).
6. LLM output MUST be validated before merging.
7. Language packs MUST fall back to English for missing keys.
8. Output renderers MUST escape user-visible text (XSS prevention).
9. Caching MUST use deterministic keys (input hashes, not timestamps).
10. All runtime configuration MUST be hot-reloadable (SIGHUP or API call).
```

---

## 15. Cross-References

### 15.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 §4.10 | GVM Module Description | GVM within the Runtime Layer architecture |
| SPEC-0001-C2 §4.11 | ExplanationEngine Module Description | MOD-11 within pipeline |
| SPEC-0001-C3 §10 | MOD-10 GVM Pipeline | GVM execution within the pipeline |
| SPEC-0001-C4 §12 | MOD-10 Interface | Formal GVM interface (execute, verify, version) |
| SPEC-0001-C4 §13 | MOD-11 Interface | Formal ExplanationEngine interface (explain, formats, languages) |
| SPEC-0001-C5 §11 | IR-10 Schema | AnalysisResult schema |
| SPEC-0001-C5 §12 | IR-11 Schema | ExplanationOutput schema |
| SPEC-0001-C7 §3.4 | Explanation Plugin Type | Plugin injection point for MOD-11 |
| SPEC-0001-C8 §5 | GVM Error Codes | Error handling for runtime modules |
| SPEC-0001-C9 §3.2 | MOD-10/MOD-11 Performance Targets | Latency, throughput, memory targets |
| SPEC-0101 §17 | Morphology ↔ GVM Interface | Feature taxonomy compatibility |
| SPEC-0201 §12 | Rule Engine ↔ Runtime Interface | Rule evidence consumption by GVM |
| SPEC-0401 §12 | KG Resolver ↔ GVM Interface | Resolved data consumption by bytecode |
| SPEC-0501 | Explanation Engine | Detailed MOD-11 specification |
| RFC-0002 | Grammar Bytecode Format | Binary format consumed by MOD-10 |
| RFC-0003 | Grammar Virtual Machine | Instruction set, execution model, conformance tests |
| ADR-0001 | Compiler Architecture Rationale | Why compiler architecture with GVM |
| ADR-0002 | Why Grammar Bytecode | Why bytecode + GVM separates compilation from runtime |

### 15.2 Knowledge Base References

| KB | How MOD-10/MOD-11 Uses It |
|----|---------------------------|
| KB-0003 (Verb Forms) | Verb form display names in language packs |
| KB-0004 (Noun Patterns) | Noun pattern display in I'rab explanations |
| KB-0005 (Particles) | Particle type display, governance explanation |
| KB-0006 (Pronouns) | Pronoun type display, anaphora explanation |
| KB-0007 (Features) | Feature ID ↔ display name mapping in language packs |

### 15.3 External References

| Reference | Relevance |
|-----------|-----------|
| JVM Specification (Java 8) | Stack-based VM design, verification, class file format |
| WebAssembly Specification | Sandboxing, deterministic execution, module format |
| Lua 5.4 VM | Minimal instruction set, register-based design |
| CPython Bytecode | Instruction set design patterns |
| Jinja2 / Handlebars | Template engine design patterns |
| ICU MessageFormat | Pluralization and localization patterns |
| ISO 639-1 | Language codes for language packs |
| RFC 3339 | Timestamp format for metadata |

---

## Progress Summary

**SPEC-0301: Grammar Runtime**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Architecture Overview | ✓ COMPLETE |
| 3 | Internal Component Model | ✓ COMPLETE |
| 4 | GVM Integration & Lifecycle | ✓ COMPLETE |
| 5 | GVM Execution Pipeline | ✓ COMPLETE |
| 6 | GVM Memory & Resource Management | ✓ COMPLETE |
| 7 | GVM Diagnostics & Verification | ✓ COMPLETE |
| 8 | Explanation Engine Architecture | ✓ COMPLETE |
| 9 | Template System & Localization | ✓ COMPLETE |
| 10 | LLM Integration | ✓ COMPLETE |
| 11 | Output Formatting & Rendering | ✓ COMPLETE |
| 12 | Performance & Optimization | ✓ COMPLETE |
| 13 | Testing Strategy | ✓ COMPLETE |
| 14 | Implementation Guidance | ✓ COMPLETE |
| 15 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001-C2/C3/C4/C5/C7/C8/C9, SPEC-0101, SPEC-0201, SPEC-0401, SPEC-0501, RFC-0002, RFC-0003, KB-0003–0007, ADR-0001, ADR-0002.

**Recommended next step:** SPEC-0501 (Explanation Engine) — the detailed specification for MOD-11 covering advanced template customization, LLM prompt engineering, and educational application support.
