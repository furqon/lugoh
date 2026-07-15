# SPEC-0601: Plugin System

| **Field** | **Value** |
|---|---|
| **Spec ID** | SPEC-0601 |
| **Title** | Plugin System — Deep-Dive Specification for MOD-12 |
| **Version** | 1.0.0 |
| **Status** | Draft |
| **Depends on** | SPEC-0001-C1 (Introduction), C2 (Architecture), C4 (Module Interfaces), C7 (Plugin Architecture), C8 (Security), C9 (Performance) |
| **Related RFCs** | RFC-0001 (Grammar DSL), RFC-0004 (Arabic Grammar Rule DSL) |
| **Related SPECs** | SPEC-0101 (Morphology), SPEC-0201 (Rule Engine), SPEC-0301 (Grammar Runtime), SPEC-0401 (KG Engine), SPEC-0501 (Explanation Engine) |
| **License** | AGOS Specification License v1.0 |

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Plugin Types & Lifecycle](#3-plugin-types--lifecycle)
4. [Plugin Manifest Format](#4-plugin-manifest-format)
5. [PluginLoader Core](#5-pluginloader-core)
6. [WASM Sandboxing](#6-wasm-sandboxing)
7. [Security Model](#7-security-model)
8. [Plugin Distribution](#8-plugin-distribution)
9. [Plugin Registry](#9-plugin-registry)
10. [Dependency Management](#10-dependency-management)
11. [Performance Targets](#11-performance-targets)
12. [Plugin SDK Guide](#12-plugin-sdk-guide)
13. [API Reference](#13-api-reference)
14. [Testing & Quality](#14-testing--quality)
15. [Cross-References](#15-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

This specification defines the **AGOS Plugin System** — the framework for extending the AGOS platform with third-party functionality. It covers the complete lifecycle of plugins: authoring, packaging, signing, distribution, discovery, loading, sandboxed execution, and cleanup.

The plugin system is implemented by **MOD-12 (PluginLoader)** and is the mechanism by which the monolingual core platform (Arabic grammar analysis) can be extended with:

- Custom grammar rule sets for new schools or dialects
- Alternative knowledge base resolvers for non-Arabic languages
- Explanation templates, LLM prompts, and pedagogical strategies
- Pipeline interceptors (pre/post processing hooks)
- Custom output format renderers
- Educational gamification or adaptive learning modules
- Performance monitoring and telemetry

### 1.2 Scope

**SPEC-0601 covers:**

| Category | Details |
|----------|---------|
| **Plugin types** | 9 defined types with their interfaces and slot assignments |
| **Lifecycle management** | Init, configure, register, activate, deactivate, shutdown |
| **WASM sandboxing** | Capability-based security, memory limits, syscall filtering, ABI |
| **Distribution** | .agosplugin package format, signing, verification, versioning |
| **Plugin registry** | Local and remote stores, indexing, search, updates |
| **Dependency resolution** | Version constraints, compatibility matrix, conflict resolution |
| **Plugin SDK** | Authoring guide, template projects, tooling |
| **Security model** | Capability-based permissions, audit logging, resource quotas |
| **Testing framework** | Plugin test harness, integration tests, validation tools |

**Out of scope:**

| Topic | Reason |
|-------|--------|
| Specific plugin implementations (e.g., the Urdu plugin from SPEC-0501) | Examples are illustrative, not normative |
| Pipeline extension hooks (pre/post processors) | Covered by SPEC-0001-C7 §5 |
| Plugin marketplace UI | A separate product concern |
| WASM runtime selection or portability | Relies on standard WASM + WASI |

### 1.3 Relationship to SPEC-0001-C7 (Plugin Architecture)

SPEC-0001-C7 defines the **architecture-level** plugin system: plugin types, injection points, the Plugin trait, and the overall design philosophy. SPEC-0601 is the **implementation-level** deep-dive that specifies:

| Aspect | SPEC-0001-C7 (Architecture) | SPEC-0601 (Implementation) |
|--------|----------------------------|---------------------------|
| Plugin types | Lists 8 types with brief descriptions | 9 types with full interface signatures, slot assignments, priority rules |
| Plugin trait | Generic `Plugin<T>` with `process()` | Per-type traits extending `PluginBase` with type-specific methods |
| Lifecycle | Conceptual lifecycle (discover, load, init, execute, unload) | Detailed 7-stage lifecycle with hooks, state machine, error states |
| WASM | Mentions WASM as recommended sandboxing | Full WASM ABI, memory model, syscall table, capability mapping |
| Security | Threat model and security principles | Capability-based permission model, audit logging, resource quotas |
| Distribution | Unspecified | Package format, signing, registry protocol, update flow |
| Plugin SDK | Not covered | Complete SDK guide, template projects, CLI tools |
| Performance | No specific targets | Quantified latency, memory, and throughput targets for each subsystem |

### 1.4 Design Principles

1. **Capability-based security, not identity-based.** A plugin's permissions are determined by its declared capabilities, not its publisher identity. Every operation requires an explicit capability grant.

2. **Sandbox by default, escape by exception.** All plugins execute in a WASM sandbox with zero default permissions. Capabilities are granted explicitly by the system administrator or through the manifest's declared `required_capabilities`.

3. **Deterministic loading.** Plugin loading order is deterministic and independent of filesystem enumeration order. Dependency graphs are resolved topologically.

4. **Graceful degradation.** A failing plugin never crashes the host. Plugin errors are caught, logged, and isolated. The pipeline can fall back to defaults when a plugin is unavailable.

5. **Versioned compatibility.** Every plugin declares its API version. The system enforces semver compatibility checks at load time. Breaking changes require explicit migration.

6. **Auditable execution.** All plugin operations that modify state or access external resources are logged to an audit trail. The trail is tamper-evident and inspectable.

7. **Lightweight hot-swapping.** Plugins can be loaded, activated, deactivated, and unloaded at runtime without pipeline restart (subject to pending work draining).

### 1.5 Performance Targets

| Metric | Target | Condition |
|--------|--------|-----------|
| Plugin load time (cold) | < 50 ms | WASM binary ≤ 1 MB, on SSD |
| Plugin load time (warm, cached) | < 5 ms | WASM binary in OS page cache |
| Manifest validation | < 1 ms | Per manifest |
| Dependency resolution | < 5 ms | Graph with ≤ 50 plugins |
| Plugin-to-host call overhead | < 1 μs | WASM ABI call |
| Sandbox init | < 10 ms | New WASM instance |
| Audit log write | < 100 μs | Single entry, async commit |
| Registry sync | < 2 s | 100 plugins, 10 Mbps connection |
| Memory overhead per inactive plugin | < 100 KB | Cached manifest + metadata |
| Memory overhead per active plugin | < 10 MB + plugin WASM heap | Configurable max |

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      AGOS Platform Host                              │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                   PluginLoader (MOD-12)                      │     │
│  │                                                               │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │     │
│  │  │ Manifest │  │  WASM    │  │  Plugin  │  │  Capability  │  │     │
│  │  │ Validator│  │  Loader  │  │  Manager │  │  Controller  │  │     │
│  │  └──────────┘  └──────────┘  └──────────┘  └─────────────┘  │     │
│  │                                                               │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │     │
│  │  │Dependency│  │ Registry │  │ Security │  │   Plugin    │  │     │
│  │  │ Resolver │  │  Manager │  │  Auditor │  │   Pool      │  │     │
│  │  └──────────┘  └──────────┘  └──────────┘  └─────────────┘  │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                    Plugin Execution Layer                      │     │
│  │                                                               │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │     │
│  │  │ WASM     │  │ WASM     │  │ WASM     │  │  Native     │  │     │
│  │  │ Instance │  │ Instance │  │ Instance │  │  Plugin     │  │     │
│  │  │ (PluginA)│  │ (PluginB)│  │ (PluginC)│  │  (PluginD)  │  │     │
│  │  └──────────┘  └──────────┘  └──────────┘  └─────────────┘  │     │
│  │                                                               │     │
│  │  ┌────────────────────────────────────────────────────────┐   │     │
│  │  │              Host Functions (ABI)                       │   │     │
│  │  │  log  audit  kv_get  kv_set  http_get  now  emit_metric │   │     │
│  │  └────────────────────────────────────────────────────────┘   │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                 Pipeline Integration Layer                     │     │
│  │                                                               │     │
│  │  MOD-01..MOD-06  MOD-07  MOD-08  MOD-09  MOD-10  MOD-11      │     │
│  │  (Pre-process)  (Rules)  (KG)   (BC Gen) (GVM)  (Explain)    │     │
│  │       ▲            ▲       ▲       ▲       ▲       ▲          │     │
│  │       │            │       │       │       │       │          │     │
│  │       └────────────┴───────┴───────┴───────┴───────┘          │     │
│  │                      Plugin Injection Points                   │     │
│  └─────────────────────────────────────────────────────────────┘     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐     │
│  │                    External Resources                          │     │
│  │  [Plugin Registry (Remote)]  [Package Cache]  [Key Store]    │     │
│  └─────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Core Components

| Component | Responsibility |
|-----------|----------------|
| **Manifest Validator** | Parses and validates plugin manifests (JSON schema, signature check, semver) |
| **WASM Loader** | Compiles WASM binaries, creates sandboxed instances, manages memory pools |
| **Plugin Manager** | Orchestrates lifecycle: discover → validate → resolve → load → init → activate |
| **Capability Controller** | Enforces capability-based permissions; maps declared capabilities to host functions |
| **Dependency Resolver** | Resolves dependency graph, detects cycles, selects compatible versions |
| **Registry Manager** | Manages local plugin store (installed, cached), syncs with remote registry |
| **Security Auditor** | Maintains tamper-evident audit log of plugin operations, resource usage, and violations |
| **Plugin Pool** | Manages active plugin instances, handles hot-swap, draining, and resource limits |

### 2.3 Plugin Injection Points

Plugins integrate into the compilation pipeline through well-defined injection points. Each point corresponds to a plugin type:

```
Pipeline Stage             Injection Point              Plugin Type
─────────────────────────────────────────────────────────────────────
MOD-01 (Unicode Validator)  pre_process(text)             pre_processor
MOD-02 (Lexer)              pre_process(tokens)           pre_processor
MOD-03 (Tokenizer)          pre_process(lexemes)          pre_processor
MOD-04 (Morph Parser)       resolve_root(kb_lookup)       kb_resolver
MOD-04 (Morph Parser)       resolve_wazan(kb_lookup)     kb_resolver
MOD-07 (Rule Engine)        load_rule_set(school)         rule_set
MOD-08 (KG Engine)          resolve_entity(query)        kb_resolver
MOD-08 (KG Engine)          pre_process(query)           pre_processor
MOD-10 (GVM)                pre_execute(bytecode)        pre_processor
MOD-10 (GVM)                post_execute(result)         post_processor
MOD-11 (Explain Engine)     explain_token(context)       explanation
MOD-11 (Explain Engine)     explain_overview(context)    explanation
MOD-11 (Explain Engine)     register_templates()         explanation
Output                      format_output(result)        output_renderer
Pipeline                    monitor_event(event)         telemetry
Pipeline                    gamification(event)          gamification
```

### 2.4 Plugin Execution Model

```
                    ┌─────────────────────┐
                    │   PluginLoader.init  │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Discover Plugins    │
                    │  (filesystem scan)   │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Validate Manifests │
                    │  (schema + sig)     │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Resolve Dependencies│
                    │  (topological sort)  │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Load WASM Binaries │
                    │  (compile + sandbox) │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Grant Capabilities │
                    │  (map declared→host) │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Initialize Plugins │
                    │  (call plugin_init) │
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │  Activate Plugins   │
                    │  (register in slots)│
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │    Runtime Loop     │
                    │  (handle requests)  │◄── Hot-swap, reload,
                    │  (enforce quotas)   │    deactivate, activate
                    └─────────┬───────────┘
                              │
                    ┌─────────▼───────────┐
                    │   Deactivate &      │
                    │   Shutdown Plugins  │
                    └─────────────────────┘
```

### 2.5 Plugin State Machine

```
                     ┌──────────┐
                     │ DISCOVERED│
                     └─────┬────┘
                           │ validate()
                     ┌─────▼────┐
                     │VALIDATED │◄────┐
                     └─────┬────┘     │
                           │ resolve()│
                     ┌─────▼────┐     │
                     │RESOLVED  │─────┤ invalidate()
                     └─────┬────┘     │
                           │ load()   │
                     ┌─────▼────┐     │
                     │  LOADED  │─────┤
                     └─────┬────┘     │
                           │ init()   │
                     ┌─────▼────┐     │
                     │INITIALIZED├─────┤
                     └─────┬────┘     │
                           │ activate()│
                     ┌─────▼────┐     │
                     │ ACTIVE   │─────┤
                     └─────┬────┘     │
                           │ deactivate()
                     ┌─────▼────┐     │
                     │DEACTIVATED├────┘
                     └─────┬────┘
                           │ shutdown()
                     ┌─────▼────┐
                     │ SHUTDOWN │
                     └──────────┘

Error states (transitions from any state):
  ERROR ──► recovery attempt ──► DEACTIVATED or SHUTDOWN

State descriptions:
  DISCOVERED  : Plugin file found, not yet parsed
  VALIDATED   : Manifest parsed and validated, signature checked
  RESOLVED    : Dependencies resolved, version conflicts handled
  LOADED      : WASM binary compiled, instance created, not initialized
  INITIALIZED : plugin_init() called successfully, capabilities granted
  ACTIVE      : Plugin registered in all requested slots, handling requests
  DEACTIVATED : Plugin removed from slots, pending tasks drained
  SHUTDOWN    : Resources released, instance destroyed
  ERROR       : Irrecoverable error state; plugin must be reinstalled
```

---

## 3. Plugin Types & Lifecycle

### 3.1 Plugin Type Definitions

SPEC-0601 defines **9 plugin types**. Each type extends the base `PluginBase` trait with type-specific methods and slots.

#### 3.1.1 `pre_processor`

Transforms input data before it enters a pipeline stage. Can modify, filter, annotate, or reject input.

```
PluginType: pre_processor
Slots:      MOD-01, MOD-02, MOD-03, MOD-08, MOD-10 (pre)
Interface:
  fn process_input(input: &Input, context: &PipelineContext)
    -> Result<ProcessedInput, PluginError>;

Capabilities: [input_read, input_write]
Execution:    Synchronous, blocking
Injection:    Chain (multiple processors run in priority order)
```

#### 3.1.2 `post_processor`

Transforms or enriches output after a pipeline stage completes.

```
PluginType: post_processor
Slots:      MOD-10 (post)
Interface:
  fn process_output(output: &Output, context: &PipelineContext)
    -> Result<ProcessedOutput, PluginError>;

Capabilities: [output_read, output_write]
Execution:    Synchronous, blocking
Injection:    Chain
```

#### 3.1.3 `rule_set`

Provides a school-specific set of grammar rules. Equivalent to a compiled `.agosrules` file but delivered as a plugin.

```
PluginType: rule_set
Slots:      MOD-07
Interface:
  fn get_rules(school: &SchoolConfig) -> Result<Vec<CompiledRule>, PluginError>;
  fn get_metadata() -> Result<RuleSetMetadata, PluginError>;
  fn get_schools() -> Result<Vec<String>, PluginError>;

Capabilities: [computation]
Execution:    Called once during MOD-07 initialization; rules are cached
Injection:    Selection (MOD-07 picks which rule_set plugin to use by school match)
```

#### 3.1.4 `kb_resolver`

Provides alternative knowledge base lookup logic. Used for extending morphology or KG lookups to new languages.

```
PluginType: kb_resolver
Slots:      MOD-04 (root, wazan), MOD-08 (entity)
Interface:
  fn resolve_root(query: &RootQuery) -> Result<RootResult, PluginError>;
  fn resolve_wazan(query: &WazanQuery) -> Result<WazanResult, PluginError>;
  fn resolve_entity(query: &EntityQuery) -> Result<EntityResult, PluginError>;

Capabilities: [computation, kv_read]
Execution:    Synchronous, blocking. May access KV store for local caches.
Injection:    Fallback (host resolver runs first; plugin used if no match)
```

#### 3.1.5 `explanation`

Provides custom explanation templates and logic for the Explanation Engine. Extends the standard template library with language-specific or domain-specific explanations.

```
PluginType: explanation
Slots:      MOD-11
Interface:
  fn explain_token(token: &AnalysisToken, context: &ExplanationContext)
    -> Result<Option<String>, PluginError>;
  fn explain_overview(result: &AnalysisResult, context: &ExplanationContext)
    -> Result<Option<String>, PluginError>;
  fn register_templates(registry: &mut TemplateRegistry)
    -> Result<(), PluginError>;
  fn supported_languages() -> Result<Vec<String>, PluginError>;
  fn supported_formats() -> Result<Vec<String>, PluginError>;

Capabilities: [computation, template_read, kv_read]
Execution:    Synchronous, non-blocking (or async with explicit capability)
Injection:    Augmentation (plugin explanation is merged with standard output)
```

#### 3.1.6 `output_renderer`

Provides custom output format rendering (e.g., LaTeX, PDF, SVG, braille, speech synthesis).

```
PluginType: output_renderer
Slots:      Pipeline (output formatting)
Interface:
  fn render(result: &AnalysisResult, format: &str, config: &RenderConfig)
    -> Result<Vec<u8>, PluginError>;
  fn supported_formats() -> Result<Vec<String>, PluginError>;

Capabilities: [computation, io_write]
Execution:    Synchronous, potentially long-running (timeout configurable)
Injection:    Selection (API Gateway picks renderer by requested format)
```

#### 3.1.7 `telemetry`

Collects and reports pipeline performance metrics, error rates, and usage statistics.

```
PluginType: telemetry
Slots:      Pipeline (monitoring events)
Interface:
  fn on_event(event: &PipelineEvent) -> Result<(), PluginError>;
  fn get_metrics() -> Result<MetricsSnapshot, PluginError>;
  fn flush() -> Result<(), PluginError>;

Capabilities: [computation, network, kv_write]
Execution:    Asynchronous, non-blocking. Events are queued and flushed.
Injection:    Broadcast (all telemetry plugins receive all events)
```

#### 3.1.8 `gamification`

Provides gamification features for educational applications: scoring, achievements, progress tracking.

```
PluginType: gamification
Slots:      Pipeline (educational events)
Interface:
  fn on_analysis_complete(result: &AnalysisResult, user_id: &str)
    -> Result<GamificationEvent, PluginError>;
  fn get_user_progress(user_id: &str) -> Result<UserProgress, PluginError>;
  fn get_leaderboard(scope: &LeaderboardScope) -> Result<Leaderboard, PluginError>;

Capabilities: [computation, kv_read, kv_write]
Execution:    Asynchronous, non-blocking
Injection:    Broadcast
```

#### 3.1.9 `pipeline_interceptor`

Provides custom pipeline-level transformations: request validation, rate limiting, caching, or routing.

```
PluginType: pipeline_interceptor
Slots:      Pipeline (request/response)
Interface:
  fn intercept_request(request: &AnalyzeRequest, context: &RequestContext)
    -> Result<InterceptedRequest, PluginError>;
  fn intercept_response(response: &AnalyzeResponse, context: &RequestContext)
    -> Result<InterceptedResponse, PluginError>;

Capabilities: [computation, network, kv_read, kv_write]
Execution:    Synchronous, blocking
Injection:    Chain
```

### 3.2 Plugin Lifecycle Methods

Every plugin MUST implement the following methods from `PluginBase`:

```rust
/// Core lifecycle trait implemented by all plugins.
trait PluginBase: Send + Sync {
    /// Unique plugin identifier (e.g., "com.agos.quranic-explanation").
    fn plugin_id(&self) -> &str;

    /// Human-readable plugin name (e.g., "Quranic Explanation Enhancer").
    fn plugin_name(&self) -> &str;

    /// Plugin type from the 9 defined types.
    fn plugin_type(&self) -> PluginType;

    /// Semantic version of this plugin.
    fn version(&self) -> &SemVer;

    /// Priority within its plugin type chain. Lower = earlier execution.
    fn priority(&self) -> u16;

    /// Called once after the plugin is loaded into the sandbox.
    /// The plugin allocates any resources, initializes state, registers templates.
    fn init(&mut self, config: &PluginConfig) -> Result<(), PluginError>;

    /// Called once before the plugin is activated.
    /// The plugin registers itself in the relevant pipeline slots.
    fn activate(&mut self, slots: &SlotRegistry) -> Result<(), PluginError>;

    /// Called when the plugin is deactivated (hot-swap, removal, update).
    /// The plugin drains pending work and releases slot registrations.
    fn deactivate(&mut self) -> Result<(), PluginError>;

    /// Called once when the plugin is permanently unloaded.
    /// The plugin releases all resources, closes connections, flushes state.
    fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Health check. Return the plugin's current health status.
    fn health(&self) -> PluginHealth;

    /// Optional. Returns the set of capability grants this plugin requires.
    /// MUST be a subset of the manifest's declared capabilities.
    fn required_capabilities(&self) -> Vec<Capability>;
}
```

### 3.3 PluginConfig

```rust
/// Configuration passed to plugins during initialization.
struct PluginConfig {
    /// Plugin data directory (writable, sandboxed).
    pub data_dir: PathBuf,

    /// Plugin cache directory (writable, sandboxed, may be cleared).
    pub cache_dir: PathBuf,

    /// Plugin configuration key-value pairs from the system config file.
    pub settings: HashMap<String, Value>,

    /// Maximum memory the plugin is allowed to allocate (in bytes).
    pub max_memory: u64,

    /// Maximum execution time per call (in milliseconds).
    pub max_execution_time_ms: u64,

    /// Network access level (None, SameOrigin, All).
    pub network_access: NetworkAccess,

    /// Whether the plugin can access the filesystem (read-only, writable, or none).
    pub filesystem_access: FilesystemAccess,

    /// Whether debug mode is enabled (verbose logging, relaxed sandbox).
    pub debug_mode: bool,

    /// Logging level for this plugin.
    pub log_level: LogLevel,
}
```

### 3.4 PluginError

```rust
/// Errors that can originate from a plugin.
enum PluginError {
    /// Plugin initialization failed.
    InitFailed { details: String },

    /// Plugin activation failed (e.g., slot registration conflict).
    ActivationFailed { details: String },

    /// Plugin execution failed with an internal error.
    ExecutionFailed { details: String },

    /// Plugin request timed out.
    Timeout { duration_ms: u64, operation: String },

    /// Plugin exceeded its memory quota.
    MemoryExceeded { allocated: u64, limit: u64 },

    /// Plugin attempted a disallowed operation (sandbox violation).
    SandboxViolation { operation: String, details: String },

    /// Plugin returned invalid data.
    InvalidOutput { details: String },

    /// Plugin is not initialized yet.
    NotInitialized,

    /// Plugin is deactivated or shutting down.
    Unavailable { reason: String },
}
```

---

## 4. Plugin Manifest Format

### 4.1 Manifest Schema

Every plugin MUST include a `agos-plugin.json` manifest file at the package root. The manifest is the single source of truth for plugin identity, requirements, and capabilities.

```jsonc
{
  // ── Core Identity ──────────────────────────────────────────
  "agos_plugin": "1.0.0",           // Manifest schema version
  "id": "com.agos.example.hello",    // Reverse-domain plugin ID (globally unique)
  "name": "Hello World Example",     // Human-readable name
  "description": "A minimal example plugin for AGOS",  // Short description
  "version": "1.2.3",                // Semantic version (semver 2.0)
  "author": {
    "name": "AGOS Community",
    "email": "plugins@agos.example",
    "url": "https://agos.example/plugins"
  },
  "license": "MIT",
  "homepage": "https://github.com/agos/plugins/hello",
  "repository": "https://github.com/agos/plugins/hello.git",
  "documentation": "https://docs.agos.example/plugins/hello",

  // ── Plugin Type & Slots ──────────────────────────────────
  "plugin_type": "explanation",      // One of the 9 types
  "slots": ["MOD-11"],               // Target pipeline slots

  // ── Compatibility ─────────────────────────────────────────
  "agos_api_version": ">=1.0.0 <2.0.0",  // AGOS API version constraint
  "min_agos_version": "1.0.0",           // Minimum AGOS platform version

  // ── Dependencies ─────────────────────────────────────────
  "dependencies": {
    "com.agos.base-templates": "^1.0.0",
    "com.agos.i18n-ar": "~1.2.0"
  },
  "optional_dependencies": {
    "com.agos.llm-enhancer": "^2.0.0"
  },
  "conflicts": {
    "com.agos.legacy-explainer": "<3.0.0"
  },

  // ── WASM Binary ──────────────────────────────────────────
  "wasm": {
    "binary": "plugin.wasm",         // WASM binary filename relative to manifest
    "entry_point": "agos_plugin_init",  // WASM export function name
    "abi_version": "1",              // WASM host ABI version
    "memory_pages": 256,             // Initial WASM memory (64 KB pages = 16 MB)
    "max_memory_pages": 1024,        // Max WASM memory (64 KB pages = 64 MB)
    "features": ["multivalue", "reference-types", "bulk-memory"],  // Required WASM features
    "import_modules": ["agos_abi"]   // Modules imported from host
  },

  // ── Capabilities ───────────────────────────────────────────
  "required_capabilities": [
    "computation",                   // Can use CPU (always allowed)
    "template_read",                 // Can read template definitions
    "kv_read"                        // Can read from sandboxed KV store
  ],
  "optional_capabilities": [],

  // ── Permissions ───────────────────────────────────────────
  "permissions": {
    "network": {
      "access": "none",              // "none" | "same-origin" | "all"
      "allowed_domains": [],
      "allowed_urls": []
    },
    "filesystem": {
      "access": "read-only",         // "none" | "read-only" | "writable"
      "allowed_paths": ["data/"],
      "max_storage_mb": 10
    },
    "execution": {
      "max_time_ms": 500,            // Max execution time per call
      "max_memory_mb": 64,           // Max memory usage
      "max_calls_per_second": 1000,  // Rate limit
      "allow_subprocesses": false    // CANNOT spawn subprocesses
    }
  },

  // ── Runtime Configuration ────────────────────────────────
  "config": {
    "setting_name": "value",
    "another_setting": 42,
    "nested": {
      "key": "value"
    }
  },
  "default_config": {
    "setting_name": "default_value"
  },

  // ── Signing ────────────────────────────────────────────────
  "signatures": [
    {
      "algorithm": "Ed25519",
      "key_id": "key-fingerprint-hex",
      "signature": "base64-encoded-signature",
      "signed_at": "2026-07-15T00:00:00Z",
      "signer": {
        "name": "AGOS Plugin Signing Key",
        "url": "https://keys.agos.example/key-fingerprint-hex"
      }
    }
  ],

  // ── Metadata ────────────────────────────────────────────────
  "categories": ["education", "quran"],
  "tags": ["arabic", "explanation", "pedagogy"],
  "icon": "icon.png",
  "screenshots": ["screenshot1.png", "screenshot2.png"],
  "keywords": ["arabic grammar", "i'rab", "quranic"],
  "languages": ["ar", "en"],
  "educational_levels": ["beginner", "intermediate", "advanced"],
  "pipeline_stage": "post_processing",
  "example_input": "{\"text\": \"السلام عليكم\"}",
  "changelog_url": "https://github.com/agos/plugins/hello/blob/main/CHANGELOG.md"
}
```

### 4.2 Manifest Validation Rules

| Rule | Description | Severity |
|------|-------------|----------|
| M-01 | `id` MUST be a valid reverse-domain name | error |
| M-02 | `version` MUST be valid semver 2.0 | error |
| M-03 | `agos_plugin` MUST be "1.0.0" (current schema version) | error |
| M-04 | `plugin_type` MUST be one of the 9 defined types | error |
| M-05 | `slots` MUST be valid slot identifiers for the plugin type | error |
| M-06 | `wasm.binary` MUST reference a file that exists in the package | error |
| M-07 | `wasm.abi_version` MUST match the host ABI version | error |
| M-08 | `required_capabilities` MUST be a subset of allowed capabilities | error |
| M-09 | `agos_api_version` MUST use valid semver constraint syntax | error |
| M-10 | `dependencies` MUST NOT contain the plugin's own `id` | error |
| M-11 | `conflicts` MUST NOT contain the plugin's own `id` | error |
| M-12 | `dependencies` and `conflicts` MUST be disjoint | error |
| M-13 | `permissions.network.access` MUST be "none", "same-origin", or "all" | error |
| M-14 | `permissions.filesystem.access` MUST be "none", "read-only", or "writable" | error |
| M-15 | `permissions.execution.max_time_ms` MUST be ≤ system max (30,000) | error |
| M-16 | `permissions.execution.max_memory_mb` MUST be ≤ system max (512) | error |
| M-17 | `permissions.execution.allow_subprocesses` MUST be `false` (reserved for future) | error |
| M-18 | `signatures` MUST contain at least one valid signature | error |
| M-19 | Plugin ID MUST NOT use reserved prefixes (`agos.`, `agos.` system) | warning |
| M-20 | `description` SHOULD be ≤ 280 characters | warning |

### 4.3 Capability Catalog

All capabilities are categorized and validated at load time. Every capability maps to a set of host functions that the plugin can call.

| Capability | Description | Host Functions Exposed |
|------------|-------------|----------------------|
| `computation` | CPU execution (always granted) | (none — implicit for WASM execution) |
| `input_read` | Read pipeline input data | `agos_abi_read_input` |
| `input_write` | Modify pipeline input data | `agos_abi_write_input` |
| `output_read` | Read pipeline output data | `agos_abi_read_output` |
| `output_write` | Modify pipeline output data | `agos_abi_write_output` |
| `template_read` | Read template definitions | `agos_abi_template_get` |
| `template_write` | Write/modify template definitions | `agos_abi_template_put`, `agos_abi_template_delete` |
| `kv_read` | Read from sandboxed key-value store | `agos_abi_kv_get`, `agos_abi_kv_exists` |
| `kv_write` | Write to sandboxed key-value store | `agos_abi_kv_put`, `agos_abi_kv_delete` |
| `kv_list` | List keys in sandboxed key-value store | `agos_abi_kv_list` |
| `network` | Make HTTP requests | `agos_abi_http_get`, `agos_abi_http_post` |
| `io_read` | Read files from allowed paths | `agos_abi_fs_read` |
| `io_write` | Write files to allowed paths | `agos_abi_fs_write` |
| `io_list` | List files in allowed paths | `agos_abi_fs_list` |
| `audit_write` | Write to the audit log | `agos_abi_audit_log` |
| `metric_emit` | Emit metrics/telemetry | `agos_abi_metric_emit` |
| `crypto_hash` | Compute cryptographic hashes | `agos_abi_hash_sha256`, `agos_abi_hash_blake3` |
| `crypto_random` | Access cryptographically secure randomness | `agos_abi_random_bytes` |
| `time` | Read current time | `agos_abi_now` |
| `log` | Write to the plugin log | `agos_abi_log` |

### 4.4 Capability Granting Algorithm

```
function grant_capabilities(manifest, system_config):
    granted = Set()
    for cap in manifest.required_capabilities:
        if cap not in CAPABILITY_CATALOG:
            reject("Unknown capability: {cap}")
        if cap in system_config.blocked_capabilities:
            reject("Capability {cap} blocked by system policy")
        if cap requires_network() and manifest.network_access == "none":
            reject("Capability {cap} requires network but network access is 'none'")
        granted.add(cap)
    
    for cap in manifest.optional_capabilities:
        if cap in CAPABILITY_CATALOG and cap not in system_config.blocked_capabilities:
            if not cap.requires_network() or manifest.network_access != "none":
                granted.add(cap)
    
    for cap in granted:
        expose_host_functions(cap)
    
    return granted
```

---

## 5. PluginLoader Core

### 5.1 PluginLoader Trait

The `PluginLoader` is the central orchestrator. It is the public API of MOD-12.

```rust
/// The PluginLoader manages the full plugin lifecycle.
trait PluginLoader {
    /// Initialize the PluginLoader with system configuration.
    /// Discovers all installed plugins from the plugin directories.
    fn init(config: &PluginLoaderConfig) -> Result<PluginLoaderHandle, PluginLoaderError>;

    /// Discover plugins from all configured directories.
    /// Returns a list of discovered plugin manifests (not yet validated).
    fn discover_plugins() -> Result<Vec<PluginManifest>, PluginLoaderError>;

    /// Validate a single plugin manifest (schema + signature).
    fn validate_plugin(manifest: &PluginManifest) -> Result<ValidatedManifest, PluginLoaderError>;

    /// Resolve dependencies for a plugin.
    /// Checks all required and optional dependencies are satisfied.
    fn resolve_dependencies(plugin_id: &str) -> Result<DependencyGraph, PluginLoaderError>;

    /// Load a plugin into memory: compile WASM, create sandbox, instantiate.
    fn load_plugin(plugin_id: &str) -> Result<PluginInstance, PluginLoaderError>;

    /// Initialize a loaded plugin (call plugin_init, grant capabilities).
    fn init_plugin(instance: &mut PluginInstance) -> Result<(), PluginLoaderError>;

    /// Activate a plugin (register in pipeline slots).
    fn activate_plugin(instance: &mut PluginInstance) -> Result<(), PluginLoaderError>;

    /// Deactivate a plugin (drain slots, stop processing).
    fn deactivate_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

    /// Unload a plugin (shutdown + release resources).
    fn unload_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

    /// Get the current state of a plugin.
    fn get_plugin_state(plugin_id: &str) -> Result<PluginState, PluginLoaderError>;

    /// List all known plugins with their states.
    fn list_plugins() -> Result<Vec<PluginInfo>, PluginLoaderError>;

    /// Install a plugin from a .agosplugin package file.
    fn install_package(path: &Path) -> Result<PluginManifest, PluginLoaderError>;

    /// Uninstall a plugin (remove all traces).
    fn uninstall_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

    /// Update a plugin from a new .agosplugin package file.
    fn update_plugin(plugin_id: &str, package_path: &Path) -> Result<PluginManifest, PluginLoaderError>;

    /// Get plugin by ID for a specific type (used by pipeline stages).
    fn get_plugin(plugin_id: &str) -> Result<PluginInstance, PluginLoaderError>;

    /// Get all active plugins of a given type (ordered by priority).
    fn get_plugins_by_type(plugin_type: PluginType)
        -> Result<Vec<PluginInstance>, PluginLoaderError>;
}
```

### 5.2 PluginLoaderConfig

```rust
/// System configuration for the PluginLoader.
struct PluginLoaderConfig {
    /// Directories to scan for plugins (in order).
    pub plugin_dirs: Vec<PathBuf>,

    /// Directory for cached WASM binaries and metadata.
    pub cache_dir: PathBuf,

    /// Directory for plugin data (persistent, backed up).
    pub data_dir: PathBuf,

    /// Maximum number of simultaneously active plugins.
    pub max_active_plugins: usize,

    /// Maximum plugin pool size (for pooling WASM instances).
    pub max_pool_size: usize,

    /// Global maximum memory per plugin (bytes).
    pub global_max_memory_per_plugin: u64,

    /// Global maximum execution time per plugin call (ms).
    pub global_max_execution_time_ms: u64,

    /// Whether to enable the plugin sandbox.
    pub sandbox_enabled: bool,

    /// Security audit level.
    pub audit_level: AuditLevel,

    /// Whether to allow unsigned plugins (development only).
    pub allow_unsigned_plugins: bool,

    /// Capabilities blocked globally (system policy).
    pub blocked_capabilities: Vec<Capability>,

    /// Remote registry URL (optional).
    pub registry_url: Option<String>,

    /// Registry sync interval in seconds.
    pub registry_sync_interval_secs: u64,

    /// TLS configuration for registry communication.
    pub tls_config: Option<TLSConfig>,
}

enum AuditLevel {
    /// Only log sandbox violations and capability denials.
    SecurityOnly,
    /// Log all plugin operations.
    Full,
}

enum PluginLoaderError {
    IoError { path: PathBuf, details: String },
    ManifestInvalid { plugin_id: String, details: String },
    ManifestNotFound { plugin_id: String },
    SignatureInvalid { plugin_id: String, details: String },
    VersionMismatch { plugin_id: String, expected: String, actual: String },
    DependencyMissing { plugin_id: String, dependency: String },
    CircularDependency { plugin_id: String, cycle: Vec<String> },
    LoadFailed { plugin_id: String, details: String },
    InitFailed { plugin_id: String, details: String },
    ActivateFailed { plugin_id: String, details: String },
    SlotConflict { plugin_id: String, slot: String, existing: String },
    SandboxViolation { plugin_id: String, operation: String },
    ExecutionTimeout { plugin_id: String, duration_ms: u64 },
    ResourceExceeded { plugin_id: String, resource: String, limit: u64 },
    RegistryError { details: String },
    PackageCorrupted { details: String },
    Internal { details: String },
}
```

### 5.3 Plugin Discovery Algorithm

```
function discover_plugins():
    manifests = []
    for dir in config.plugin_dirs:
        if not dir.exists():
            continue
        for entry in dir.iter():
            if entry.is_dir():
                manifest_path = entry / "agos-plugin.json"
                if manifest_path.exists():
                    manifest = Manifest::from_file(manifest_path)
                    manifests.push(manifest)
            elif entry.extension() == ".agosplugin":
                # Packaged plugin file
                temp_dir = extract_package(entry)
                manifest_path = temp_dir / "agos-plugin.json"
                manifest = Manifest::from_file(manifest_path)
                manifests.push(manifest)
            elif entry.extension() == ".wasm":
                # Standalone WASM with minimal manifest
                manifest = infer_manifest(entry)
                manifests.push(manifest)
    
    # Deduplicate by plugin ID (last one wins with warning)
    seen = Map()
    ordered = []
    for manifest in manifests:
        if manifest.id in seen:
            log.warn("Duplicate plugin {manifest.id} in {manifest.path} and {seen[manifest.id].path}")
        seen[manifest.id] = manifest
    
    for manifest in manifests:
        if manifest.id not in seen or seen[manifest.id] == manifest:
            ordered.push(manifest)
    
    return ordered
```

### 5.4 Dependency Resolution Algorithm

```
function resolve_dependencies(plugin_id, all_manifests):
    # Build dependency graph
    graph = DirectedGraph()
    visited = Set()
    resolving = Set()  # For cycle detection
    
    function resolve(node_id):
        if node_id in resolving:
            raise CircularDependency(node_id, resolving)
        if node_id in visited:
            return
        resolving.add(node_id)
        
        manifest = all_manifests.find(node_id)
        if not manifest:
            raise DependencyMissing(node_id)
        
        for dep_id, constraint in manifest.dependencies:
            dep_manifest = all_manifests.find(dep_id)
            if not dep_manifest:
                raise DependencyMissing(node_id, dep_id)
            if not semver_satisfies(dep_manifest.version, constraint):
                raise VersionMismatch(dep_id, constraint, dep_manifest.version)
            graph.add_edge(node_id, dep_id)
            resolve(dep_id)
        
        for dep_id, constraint in manifest.optional_dependencies:
            dep_manifest = all_manifests.find(dep_id)
            if dep_manifest and semver_satisfies(dep_manifest.version, constraint):
                graph.add_edge(node_id, dep_id)
                resolve(dep_id)
        
        for conflict_id, constraint in manifest.conflicts:
            dep_manifest = all_manifests.find(conflict_id)
            if dep_manifest and semver_satisfies(dep_manifest.version, constraint):
                raise ConflictDetected(node_id, conflict_id)
        
        resolving.remove(node_id)
        visited.add(node_id)
    
    resolve(plugin_id)
    
    # Topological sort
    load_order = topological_sort(graph)
    return DependencyGraph {
        root: plugin_id,
        load_order: load_order,
        nodes: graph.nodes(),
        edges: graph.edges(),
    }
```

### 5.5 Plugin Instance Pool

The PluginLoader maintains a pool of initialized WASM instances for frequently used plugins, avoiding repeated compilation and initialization overhead.

```
┌─────────────────────────────────────────────────────┐
│                  PluginInstancePool                   │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │  Pool Configuration                              │ │
│  │  min_idle: 2     max_total: 10     ttl: 300s    │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐│
│  │ Plugin A     │  │ Plugin A     │  │ Plugin A    ││
│  │ Instance #1  │  │ Instance #2  │  │ Instance #3 ││
│  │ [IDLE]       │  │ [IN_USE]     │  │ [IDLE]      ││
│  └──────────────┘  └──────────────┘  └─────────────┘│
│  ┌──────────────┐  ┌──────────────┐                  │
│  │ Plugin B     │  │ Plugin B     │                  │
│  │ Instance #1  │  │ Instance #2  │                  │
│  │ [IN_USE]     │  │ [IDLE]       │                  │
│  └──────────────┘  └──────────────┘                  │
│                                                       │
│  Pool size: 5/10   Active plugins: 2   Hit rate: 87%│
└─────────────────────────────────────────────────────┘
```

Pool eviction policy:
1. Least recently used (LRU) instances are evicted first
2. Idle instances are kept for `ttl` seconds before eviction
3. `[IN_USE]` instances are never evicted
4. When pool is full and no idle instances, wait up to `acquire_timeout_ms` or create new (if under max_total)

### 5.6 Hot-Swap Protocol

Plugins can be updated at runtime without pipeline restart:

```
1. RECEIVE:      PluginLoader receives update request for plugin_id
2. VALIDATE:     New package is validated (manifest, signature, WASM)
3. RESOLVE:      Dependencies of the new version are checked
4. LOAD:         New WASM is compiled and initialized (separate from old)
5. DRAIN:        Old plugin is deactivated; pending calls are allowed to complete
                 (up to drain_timeout_ms; remaining calls are failed gracefully)
6. SWAP:         Slot registry is atomically updated to point to new plugin
7. ACTIVATE:     New plugin is activated in all slots
8. SHUTDOWN:     Old plugin instance is shutdown and released
9. CLEANUP:      Old plugin's cache and data directories may be archived
```

---

## 6. WASM Sandboxing

### 6.1 Sandbox Architecture

The WASM sandbox provides secure execution of untrusted plugin code. Each plugin runs in an isolated WASM instance with:

1. **Separate linear memory** per plugin instance (no cross-plugin memory access)
2. **Capability-gated host functions** (only declared capabilities are exposed)
3. **Resource limits** (CPU cycles, memory pages, call depth)
4. **Execution timeout** (configurable per plugin)
5. **No raw system calls** (all I/O goes through host ABI functions)
6. **Deterministic execution** (within the same inputs, same plugin version)

```
┌──────────────────────────────────────────────────────────┐
│                   AGOS Host Process                        │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                WASM Runtime (wasmtime/wasmer)         │   │
│  │                                                       │   │
│  │  ┌─────────────────────────────────────────────────┐  │   │
│  │  │          Plugin A WASM Instance                   │  │   │
│  │  │                                                   │  │   │
│  │  │  ┌──────────────┐  ┌──────────────────────────┐  │  │   │
│  │  │  │   Linear      │  │  Function Table          │  │  │   │
│  │  │  │   Memory      │  │  - agos_plugin_init     │  │  │   │
│  │  │  │   (64 MB max) │  │  - agos_plugin_process  │  │  │   │
│  │  │  │               │  │  - agos_plugin_health   │  │  │   │
│  │  │  │  [stack]      │  │  - ...                   │  │  │   │
│  │  │  │  [heap]       │  │  └──────────────────────────┘  │   │
│  │  │  │  [data]       │  │                                 │   │
│  │  │  └──────────────┘  │  ┌──────────────────────────┐  │   │
│  │  │                     │  │  Import Table             │  │   │
│  │  │                     │  │  (from agos_abi)         │  │   │
│  │  │                     │  │  - agos_abi_log          │  │   │
│  │  │                     │  │  - agos_abi_kv_get       │  │   │
│  │  │                     │  │  - agos_abi_now          │  │   │
│  │  │                     │  │  - ... (only granted)    │  │   │
│  │  │                     │  │  └──────────────────────────┘  │   │
│  │  └─────────────────────────────────────────────────┘  │   │
│  │                                                       │   │
│  │  ┌─────────────────────────────────────────────────┐  │   │
│  │  │          Plugin B WASM Instance                   │  │   │
│  │  │         (completely separate memory)              │  │   │
│  │  └─────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            Host ABI Implementations                   │   │
│  │  agos_abi_log      agos_abi_kv_get   agos_abi_now    │   │
│  │  agos_abi_http_get agos_abi_audit    agos_abi_hash   │   │
│  │         These run OUTSIDE the sandbox                 │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

### 6.2 WASM ABI Specification

The AGOS WASM ABI defines the interface between the host and plugin. All functions use the C ABI (flat i32/i64/f32/f64 parameters and returns).

#### 6.2.1 Plugin Exports (required)

These functions MUST be exported by the plugin's WASM binary:

```wasm
; Called once to initialize the plugin.
; Returns 0 on success, non-zero error code on failure.
(func $agos_plugin_init
  (param $config_ptr i32)     ; Pointer to serialized config in linear memory
  (param $config_len i32)     ; Length of serialized config
  (result i32))               ; Error code

; Called for each invocation (type-specific payload).
; Returns 0 on success, non-zero error code on failure.
(func $agos_plugin_process
  (param $input_ptr i32)      ; Pointer to serialized input
  (param $input_len i32)      ; Length of serialized input
  (param $output_ptr i32)     ; Pointer to output buffer (pre-allocated)
  (param $output_max_len i32) ; Maximum output buffer size
  (result i32))               ; Actual output length (negative = error)

; Health check. Returns health status code.
(func $agos_plugin_health
  (result i32))               ; 0=healthy, 1=degraded, 2=unhealthy

; Called when plugin is being deactivated.
(func $agos_plugin_deactivate (result i32))

; Called when plugin is being shutdown.
(func $agos_plugin_shutdown (result i32))

; Returns a pointer to the plugin's capability bitmap (static data).
(func $agos_plugin_capabilities
  (result i32))               ; Pointer to 4-byte capability bitmap
```

#### 6.2.2 Host Imports (provided by the host)

These functions are imported by the plugin from the `agos_abi` module. Only those matching granted capabilities are available.

```wasm
; ── Logging (capability: log) ──
(func $agos_abi_log
  (param $level i32)          ; 0=trace, 1=debug, 2=info, 3=warn, 4=error
  (param $message_ptr i32)
  (param $message_len i32))

; ── Time (capability: time) ──
(func $agos_abi_now (result i64))  ; Unix timestamp in milliseconds

; ── KV Store (capability: kv_read / kv_write) ──
(func $agos_abi_kv_get
  (param $key_ptr i32) (param $key_len i32)
  (param $value_ptr i32) (param $value_max_len i32)
  (result i32))               ; Actual value length, -1 if not found

(func $agos_abi_kv_put
  (param $key_ptr i32) (param $key_len i32)
  (param $value_ptr i32) (param $value_len i32)
  (result i32))               ; 0=success

(func $agos_abi_kv_delete
  (param $key_ptr i32) (param $key_len i32)
  (result i32))               ; 0=success

(func $agos_abi_kv_exists
  (param $key_ptr i32) (param $key_len i32)
  (result i32))               ; 1=exists, 0=not found

(func $agos_abi_kv_list
  (param $prefix_ptr i32) (param $prefix_len i32)
  (param $buffer_ptr i32) (param $buffer_len i32)
  (result i32))               ; Actual data length

; ── HTTP (capability: network) ──
(func $agos_abi_http_get
  (param $url_ptr i32) (param $url_len i32)
  (param $headers_ptr i32) (param $headers_len i32)
  (param $response_ptr i32) (param $response_max_len i32)
  (result i32))               ; Response length, negative=error

(func $agos_abi_http_post
  (param $url_ptr i32) (param $url_len i32)
  (param $headers_ptr i32) (param $headers_len i32)
  (param $body_ptr i32) (param $body_len i32)
  (param $response_ptr i32) (param $response_max_len i32)
  (result i32))               ; Response length, negative=error

; ── Filesystem (capability: io_read / io_write) ──
(func $agos_abi_fs_read
  (param $path_ptr i32) (param $path_len i32)
  (param $buffer_ptr i32) (param $buffer_max_len i32)
  (result i32))               ; Bytes read, negative=error

(func $agos_abi_fs_write
  (param $path_ptr i32) (param $path_len i32)
  (param $data_ptr i32) (param $data_len i32)
  (result i32))               ; 0=success

; ── Audit (capability: audit_write) ──
(func $agos_abi_audit_log
  (param $event_type_ptr i32) (param $event_type_len i32)
  (param $details_ptr i32) (param $details_len i32)
  (result i32))               ; 0=success

; ── Metrics (capability: metric_emit) ──
(func $agos_abi_metric_emit
  (param $name_ptr i32) (param $name_len i32)
  (param $value_f64 f64)
  (param $tags_ptr i32) (param $tags_len i32)
  (result i32))               ; 0=success

; ── Cryptography (capability: crypto_hash / crypto_random) ──
(func $agos_abi_hash_sha256
  (param $data_ptr i32) (param $data_len i32)
  (param $output_ptr i32))    ; Output buffer (32 bytes)

(func $agos_abi_random_bytes
  (param $buffer_ptr i32) (param $len i32))

; ── Template Access (capability: template_read / template_write) ──
(func $agos_abi_template_get
  (param $name_ptr i32) (param $name_len i32)
  (param $buffer_ptr i32) (param $buffer_max_len i32)
  (result i32))               ; Template length, -1 if not found

(func $agos_abi_template_put
  (param $name_ptr i32) (param $name_len i32)
  (param $content_ptr i32) (param $content_len i32)
  (result i32))               ; 0=success

; ── Memory (always available) ──
(func $agos_abi_alloc
  (param $size i32)
  (result i32))               ; Pointer to allocated memory

(func $agos_abi_dealloc
  (param $ptr i32) (param $size i32))
```

### 6.3 Data Serialization

Plugin inputs and outputs are serialized using **MessagePack** (for binary efficiency) with a JSON-compatible schema. The host serializes/deserializes Rust structs to/from MessagePack before passing to/from WASM linear memory.

```
Host data (Rust struct)                    WASM linear memory
┌─────────────────────┐                    ┌──────────────────────┐
│  AnalysisResult      │                    │  Serialized MsgPack   │
│  ┌───────────────┐   │   host_serialize   │  ┌────────────────┐  │
│  │ trees: [...]   │──┼────────────────────┼──│ 0x94 0xa6 ...   │  │
│  │ flags: {}      │   │                    │  └────────────────┘  │
│  │ evidence: []   │   │                    │                      │
│  └───────────────┘   │                    │  config_ptr ──────────┤
└─────────────────────┘                    └──────────────────────┘

The plugin deserializes from MsgPack and serializes response back to MsgPack.
```

### 6.4 Memory Model

```
WASM Linear Memory Layout:
┌────────────────────────────────────────────────────────┐
│  0x0000 - 0x0FFF  │  Reserved (trap page)              │
│  0x1000 - 0x1FFF  │  Plugin-managed header             │
│  0x2000 - 0x2FFF  │  ABI scratch buffer (4 KB)         │
│  0x3000 - 0x3FFF  │  Plugin configuration (4 KB)       │
│  0x4000 - 0x4FFF  │  Input buffer (4 KB)               │
│  0x5000 - 0x5FFF  │  Output buffer (4 KB)              │
│  0x6000 - 0x9FFF  │  KV entry buffer (16 KB)           │
│  0xA000 - 0xAFFF  │  HTTP response buffer (4 KB)       │
│  0xB000 - 0xBFFF  │  FS buffer (4 KB)                  │
│  0xC000 -          │  Stack (grows downward)            │
│                    │     Initial: 64 KB                 │
│                    │     Max: 1 MB                      │
│                    │  Heap (grows upward)               │
│                    │     Managed by plugin's allocator   │
│                    │     (e.g., dlmalloc, wee_alloc)    │
│                    │  Data section (static globals)     │
└────────────────────────────────────────────────────────┘

Memory constraints:
- Initial memory: specified in manifest (wasm.memory_pages × 64 KB)
- Max memory: specified in manifest (wasm.max_memory_pages × 64 KB)
- Hard cap: system config (global_max_memory_per_plugin)
- Stack overflow: trapped by WASM runtime
- Heap exhaustion: returns error, logged, plugin may retry or fail
```

### 6.5 Resource Quota Enforcement

```
┌─────────────────────────────────────────────┐
│           Resource Quota Enforcer             │
│                                               │
│  Per-call quotas (reset each call):           │
│  ├─ Execution time: 500 ms (configurable)     │
│  ├─ WASM instructions: 10,000,000 (est.)      │
│  ├─ Memory allocations: 64 MB max             │
│  ├─ KV operations: 100 calls                  │
│  ├─ HTTP requests: 5 calls                    │
│  ├─ File operations: 10 calls                │
│  └─ Log writes: 100 lines                     │
│                                               │
│  Burst quotas (per 60-second window):         │
│  ├─ Calls: 1,000 (rate: ~16.7 calls/sec)     │
│  ├─ KV reads: 10,000                          │
│  ├─ KV writes: 1,000                          │
│  ├─ HTTP requests: 100                        │
│  └─ Audit log writes: 100                     │
│                                               │
│  Enforcement:                                 │
│  ├─ Exceeded time → trap, Timeout error      │
│  ├─ Exceeded memory → trap, MemoryExceeded    │
│  ├─ Exceeded calls → Error, retry-after       │
│  └─ Violation → SandboxViolation, audit log  │
└─────────────────────────────────────────────┘
```

### 6.6 WASM Feature Requirements

| Feature | Status | Reason |
|---------|--------|--------|
| `mutable-globals` | Required | Plugin metadata globals |
| `bulk-memory` | Required | Efficient memory.copy, memory.fill |
| `reference-types` | Required | `externref` for host resource handles |
| `multivalue` | Recommended | Multiple return values for ABI |
| `sign-extension` | Recommended | Efficient sign extension ops |
| `nontrapping-fp-to-int` | Recommended | Safe float-to-int conversion |
| `simd` | Optional | Performance optimization |
| `tail-call` | Optional | Efficient recursive patterns |
| `gc` | Not supported | Too experimental |
| `threads` | Not supported | Multi-instance model preferred |
| `exception-handling` | Not supported | Error codes preferred over exceptions |

---

## 7. Security Model

### 7.1 Threat Model

| Threat ID | Description | Severity | Mitigation |
|-----------|-------------|----------|------------|
| T-01 | Plugin exfiltrates pipeline input data | Critical | Capability control, sandbox, no network by default |
| T-02 | Plugin modifies analysis results to inject bias | Critical | output_write capability controlled, audit trail |
| T-03 | Plugin consumes excessive resources (DoS) | High | Resource quotas, timeout, memory limits |
| T-04 | Plugin privilege escalation via sandbox escape | Critical | WASM sandbox, no raw syscalls, capability gating |
| T-05 | Plugin reads other plugin's memory or data | High | Separate WASM instances, no shared memory |
| T-06 | Plugin persists malicious data (persistent XSS) | Medium | Output sanitization, template sandboxing |
| T-07 | Plugin introduces malicious dependency | High | Dependency scanning, signature verification |
| T-08 | Plugin replays old signed packages (rollback attack) | Medium | Minimum version enforcement, freshness checks |
| T-09 | Plugin uses crypto to establish C2 channel | Medium | Network access controlled, allowlisted domains |
| T-10 | Plugin publishes sensitive data as metrics | Low | Metric namespaces filtered, metric_write capability |

### 7.2 Capability-Based Permission Model

```
                    ┌────────────────────────────┐
                    │  System Policy              │
                    │  (agos.toml / env vars)     │
                    │  blocked_capabilities: []   │
                    │  allow_unsigned: false      │
                    │  global_network_block: true │
                    └─────────────┬──────────────┘
                                  │
                    ┌─────────────▼──────────────┐
                    │  Plugin Manifest             │
                    │  required_capabilities: [    │
                    │    "computation",            │
                    │    "kv_read",                │
                    │    "template_read"           │
                    │  ]                           │
                    │  permissions.network: none   │
                    └─────────────┬──────────────┘
                                  │
                    ┌─────────────▼──────────────┐
                    │  Capability Granting         │
                    │                              │
                    │  1. Check system policy      │
                    │  2. Validate against type    │
                    │  3. Check permission pre-req │
                    │  4. Grant or deny            │
                    │                              │
                    │  Result: Granted: [           │
                    │    computation,               │
                    │    kv_read,                   │
                    │    template_read              │
                    │  ]                            │
                    └─────────────┬──────────────┘
                                  │
                    ┌─────────────▼──────────────┐
                    │  Host Function Exposure      │
                    │                              │
                    │  Exported to plugin:         │
                    │  ✓ agos_abi_log              │
                    │  ✓ agos_abi_now              │
                    │  ✓ agos_abi_kv_get           │
                    │  ✓ agos_abi_kv_exists        │
                    │  ✓ agos_abi_template_get     │
                    │  ✓ agos_abi_alloc            │
                    │  ✓ agos_abi_dealloc          │
                    │                              │
                    │  DENIED (no capability):     │
                    │  ✗ agos_abi_http_get         │
                    │  ✗ agos_abi_kv_put           │
                    │  ✗ agos_abi_fs_read          │
                    │  ✗ agos_abi_audit_log        │
                    └──────────────────────────────┘
```

### 7.3 Audit Trail

Every security-relevant operation is logged to the audit trail.

```jsonc
// Audit log entry format
{
  "event_id": "aev-2a1f3c8b",
  "timestamp": "2026-07-15T10:30:00.123Z",
  "event_type": "capability_denied",
  "plugin_id": "com.agos.example.hello",
  "plugin_version": "1.2.3",
  "hostname": "agos-server-01",
  "request_id": "req-abc123",
  "details": {
    "operation": "agos_abi_http_get",
    "url": "https://evil.example.com/exfil",
    "reason": "Capability 'network' not granted",
    "sandbox_violation": true
  },
  "severity": "warning",
  "chain_hash": "sha256:prev_hash + current_hash"  // Merkle chain
}
```

Audit log storage:
- Stored as append-only, rotation-managed file or database
- Tamper-evident through Merkle chain (each entry includes hash of previous)
- Readable via `agos plugin audit-log` CLI command
- Retention: minimum 90 days (configurable)

### 7.4 Plugin Signing

All plugins distributed via the registry MUST be signed. Local/development plugins may be unsigned if `allow_unsigned_plugins = true`.

```
Signing process (publisher side):
1. Generate Ed25519 keypair (or use existing AGOS publisher key)
2. Canonically serialize the manifest (sorted keys, no whitespace)
3. Sign the canonical manifest: signature = ed25519.sign(private_key, canonical_manifest)
4. Include signature in manifest.signatures array
5. Sign the WASM binary: wasm_signature = ed25519.sign(private_key, sha256(wasm_binary))
6. Include WASM signature in manifest.wasm.signature
7. Package everything into .agosplugin

Verification process (PluginLoader side):
1. Extract manifest from package
2. Verify manifest signature against known public keys
3. If key is unknown, check if key is in the registry's trusted key list
4. Verify WASM binary signature against manifest
5. If any signature fails → PLUGIN_SIGNATURE_INVALID

Key management:
- Publisher keys are registered with the registry
- Keys can be rotated (old key remains valid for previously signed packages)
- Revocation list checked at install/update time
```

### 7.5 Sandbox Escape Countermeasures

| Layer | Countermeasure |
|-------|----------------|
| **WASM Runtime** | Use latest stable wasmtime/wasmer with security patches |
| **Memory** | No `memory.grow` beyond max_pages; no shared memory; no raw pointer access to host |
| **Syscalls** | No WASI preview 2 syscalls exposed to plugins; all I/O through agos_abi |
| **CPU** | WASM instruction counting; timeout trap |
| **Stack** | Stack overflow detection and trapping |
| **Import table** | Static analysis of import table at load time |
| **Control flow** | No `call_indirect` abuse (type-checked by WASM runtime) |
| **File system** | All FS access is virtualized; paths are sandboxed to plugin data/cache dirs |
| **Network** | URL validation, domain allowlisting, no raw socket access |
| **Subprocesses** | Explicitly disallowed (`allow_subprocesses: false`) |

---

## 8. Plugin Distribution

### 8.1 Package Format (`.agosplugin`)

The `.agosplugin` file is a standard **`.tar.gz`** archive with the following structure:

```
plugin-name-1.2.3.agosplugin
├── agos-plugin.json          # Manifest (required)
├── plugin.wasm               # WASM binary (required)
├── icon.png                  # Plugin icon (optional, 256×256 PNG)
├── templates/                # Template files (optional)
│   ├── overview.hbs
│   └── token.hbs
├── data/                     # Default data files (optional)
│   ├── mappings.json
│   └── dictionary.json
├── locales/                  # Localization files (optional)
│   ├── en.json
│   └── ar.json
├── docs/                     # Documentation (optional)
│   ├── README.md
│   └── API.md
├── screenshots/              # Screenshots (optional)
│   ├── screenshot1.png
│   └── screenshot2.png
├── checksums.sha256          # SHA-256 checksums of all files
└── signature.asc             # Detached signature (optional, for GPG-based signing)
```

### 8.2 Distribution Channels

| Channel | Description | Security Level |
|---------|-------------|----------------|
| **Official AGOS Registry** | Central registry at `registry.agos.example` | High (signed, reviewed) |
| **Private Registry** | Self-hosted registry (enterprise) | Configurable |
| **Local File** | `.agosplugin` file on disk | Medium (manual verification) |
| **Development Directory** | Unpacked plugin directory | Low (unsigned, trusted dev) |
| **Package Manager** | OS package manager (apt, brew) | Host OS dependent |

### 8.3 Registry Protocol

The registry protocol uses a simple RESTful HTTP API:

```
# ── Plugin Search ─────────────────────────────────
GET /v1/search?q=explanation&lang=ar&page=1&per_page=20
Response:
{
  "results": [
    {
      "id": "com.agos.quranic-explanation",
      "name": "Quranic Explanation Enhancer",
      "version": "2.1.0",
      "description": "Enhanced I'rab explanations for Quranic verses",
      "plugin_type": "explanation",
      "author": { "name": "AGOS Foundation", "url": "https://agos.foundation" },
      "ratings": { "average": 4.8, "count": 127 },
      "downloads": 15420,
      "updated_at": "2026-07-10T00:00:00Z"
    }
  ],
  "total": 42,
  "page": 1,
  "per_page": 20
}

# ── Plugin Details ───────────────────────────────
GET /v1/plugins/com.agos.quranic-explanation
Response:
{
  "id": "com.agos.quranic-explanation",
  "name": "Quranic Explanation Enhancer",
  "latest_version": "2.1.0",
  "versions": {
    "2.1.0": {
      "published_at": "2026-07-10T00:00:00Z",
      "agos_api_version": ">=1.0.0 <2.0.0",
      "min_agos_version": "1.0.0",
      "signatures": [ ... ],
      "download_url": "https://registry.agos.example/v1/packages/com.agos.quranic-explanation/2.1.0/download",
      "checksum_sha256": "abc123..."
    },
    "2.0.0": { ... },
    "1.0.0": { ... }
  },
  "dependencies": { ... },
  "reverse_dependencies": ["com.agos.education-suite"]
}

# ── Download Package ──────────────────────────────
GET /v1/packages/com.agos.quranic-explanation/2.1.0/download
Response: binary .agosplugin file (Content-Type: application/gzip)

# ── Publish Package ──────────────────────────────
POST /v1/publish
Body: multipart/form-data containing .agosplugin file
Headers: Authorization: Bearer <publisher_token>
Response:
{
  "id": "com.agos.quranic-explanation",
  "version": "2.1.0",
  "status": "published"
}

# ── Check for Updates ────────────────────────────
POST /v1/check-updates
Body:
{
  "plugins": {
    "com.agos.quranic-explanation": "1.5.0",
    "com.agos.base-templates": "1.0.0"
  }
}
Response:
{
  "updates_available": {
    "com.agos.quranic-explanation": {
      "latest_version": "2.1.0",
      "update_type": "major",
      "changelog_url": "..."
    }
  }
}
```

### 8.4 Installation Flow

```
1. RECEIVE:      User provides .agosplugin file or registry reference
2. EXTRACT:      Extract .tar.gz to temporary directory
3. VERIFY:       Verify checksums (sha256 against checksums.sha256)
4. VALIDATE:     Validate manifest JSON schema
5. SIGNATURE:    Verify signatures against known/trusted keys
6. DEPENDENCIES: Check dependencies are satisfied (or install them)
7. COPY:         Copy to <plugin_dir>/<plugin_id>/<version>/
8. REGISTER:     Add to local registry database
9. ACTIVATE:     If auto-activate, load and activate the plugin
10. CLEANUP:     Remove temporary directory
```

---

## 9. Plugin Registry

### 9.1 Local Plugin Registry

The local registry is a SQLite database (or equivalent) that tracks all installed plugins, their states, and metadata.

```sql
-- Core schema for the local plugin registry

CREATE TABLE installed_plugins (
    id              TEXT PRIMARY KEY,          -- Reverse-domain plugin ID
    name            TEXT NOT NULL,
    version         TEXT NOT NULL,             -- Installed version
    plugin_type     TEXT NOT NULL,             -- One of 9 types
    state           TEXT NOT NULL DEFAULT 'discovered',  -- Current lifecycle state
    priority        INTEGER NOT NULL DEFAULT 100,
    manifest_path   TEXT NOT NULL,             -- Path to agos-plugin.json
    wasm_path       TEXT NOT NULL,             -- Path to WASM binary
    data_dir        TEXT NOT NULL,             -- Plugin data directory
    cache_dir       TEXT NOT NULL,             -- Plugin cache directory
    install_path    TEXT NOT NULL,             -- Full install directory
    installed_at    TEXT NOT NULL,             -- ISO 8601 timestamp
    updated_at      TEXT NOT NULL,
    auto_activate   INTEGER NOT NULL DEFAULT 1,
    min_agos_version TEXT,
    api_version_constraint TEXT,
    publisher_id    TEXT,
    checksum_sha256 TEXT,                     -- Hash of package file
    UNIQUE(id, version)
);

CREATE TABLE plugin_slots (
    plugin_id       TEXT NOT NULL REFERENCES installed_plugins(id) ON DELETE CASCADE,
    slot            TEXT NOT NULL,             -- e.g., "MOD-11"
    activated_at    TEXT,
    priority        INTEGER NOT NULL DEFAULT 100,
    PRIMARY KEY (plugin_id, slot)
);

CREATE TABLE plugin_capabilities (
    plugin_id       TEXT NOT NULL REFERENCES installed_plugins(id) ON DELETE CASCADE,
    capability      TEXT NOT NULL,             -- e.g., "kv_read"
    granted         INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (plugin_id, capability)
);

CREATE TABLE plugin_dependencies (
    plugin_id       TEXT NOT NULL REFERENCES installed_plugins(id) ON DELETE CASCADE,
    dependency_id   TEXT NOT NULL,
    constraint_spec TEXT NOT NULL,             -- e.g., "^1.0.0"
    dependency_type TEXT NOT NULL DEFAULT 'required',  -- 'required' | 'optional'
    resolved_version TEXT,
    PRIMARY KEY (plugin_id, dependency_id)
);

CREATE TABLE plugin_events (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id       TEXT NOT NULL,
    event_type      TEXT NOT NULL,             -- 'loaded', 'activated', 'error', 'violation', etc.
    timestamp       TEXT NOT NULL,
    details         TEXT,                      -- JSON blob
    severity        TEXT NOT NULL DEFAULT 'info'
);

CREATE TABLE plugin_settings (
    plugin_id       TEXT NOT NULL REFERENCES installed_plugins(id) ON DELETE CASCADE,
    key             TEXT NOT NULL,
    value           TEXT NOT NULL,
    PRIMARY KEY (plugin_id, key)
);

-- Indexes
CREATE INDEX idx_plugins_type ON installed_plugins(plugin_type, state);
CREATE INDEX idx_plugins_state ON installed_plugins(state);
CREATE INDEX idx_slot_priority ON plugin_slots(slot, priority);
CREATE INDEX idx_events_plugin ON plugin_events(plugin_id, timestamp);
CREATE INDEX idx_events_type ON plugin_events(event_type, timestamp);
```

### 9.2 Sync Protocol

The local registry periodically (or on demand) syncs with the remote registry to check for updates:

```
1. TRIGGER:  Timer fires (configurable interval, default 24h) or manual
2. BUILD:    Build local plugin version map: {plugin_id: installed_version}
3. REQUEST:  POST /v1/check-updates with local version map
4. RESPONSE: Receive list of available updates
5. NOTIFY:   If updates available, notify administrator
6. AUTO:     If auto-update enabled, download and stage updates
7. NOTIFY:   Notify administrator of staged updates (require restart or hot-swap)
```

### 9.3 Update Policy

| Policy | Description |
|--------|-------------|
| `manual` | Updates are never applied automatically; admin must approve |
| `patch` | Patch-level updates (1.0.x) are applied automatically |
| `minor` | Minor updates (1.x.0) are applied automatically |
| `all` | All updates applied automatically (not recommended for production) |

---

## 10. Dependency Management

### 10.1 Version Constraint Syntax

Following semver constraint syntax (same as npm/cargo):

| Constraint | Meaning |
|------------|---------|
| `^1.2.3` | Compatible with 1.2.3 (>=1.2.3 <2.0.0) |
| `~1.2.3` | Approximately 1.2.3 (>=1.2.3 <1.3.0) |
| `>=1.2.3` | Any version ≥ 1.2.3 |
| `<2.0.0` | Any version < 2.0.0 |
| `1.2.3` | Exact version 1.2.3 |
| `*` | Any version |
| `>=1.0.0 <2.0.0` | Range |
| `1.2.x` | Any patch for 1.2 |
| `1.x` | Any minor for 1 |

### 10.2 Compatibility Matrix

The compatibility matrix maps AGOS platform versions to API versions:

| AGOS Version | API Version | Notes |
|-------------|-------------|-------|
| 1.0.0 – 1.9.x | 1.0.x | Initial stable API |
| 2.0.0 – 2.9.x | 2.0.x | Breaking changes expected |
| 3.0.0+ | 3.0.x+ | TBD |

### 10.3 Conflict Resolution

When multiple plugins require different versions of the same dependency:

| Strategy | Description |
|----------|-------------|
| `highest` | Select the highest compatible version (default) |
| `lowest` | Select the lowest compatible version |
| `exact` | Require exact version match |
| `fail` | Fail if any conflict exists |

Algorithm:

```
function resolve_conflicts(dependency_graph, strategy):
    # For each dependency, collect all constraints
    constraints_by_dep = Map()
    for edge in dependency_graph.edges:
        constraints_by_dep[edge.dep_id].push(edge.constraint)
    
    resolved_versions = Map()
    for dep_id, constraints in constraints_by_dep:
        candidates = registry.get_versions(dep_id)
        
        # Filter candidates that satisfy ALL constraints
        valid = []
        for version in candidates:
            if all(constraint.satisfies(version) for constraint in constraints):
                valid.push(version)
        
        if valid.is_empty():
            raise Unresolvable(dep_id, constraints)
        
        resolved_versions[dep_id] = strategy.select(valid)
    
    return resolved_versions
```

### 10.4 Circular Dependency Detection

```
function check_circular_dependencies(plugin_id, all_manifests):
    visited = Set()
    path = Stack()
    
    function dfs(node_id):
        if node_id in path:
            cycle = path.to_list_from(node_id)
            raise CircularDependency(plugin_id, cycle)
        if node_id in visited:
            return
        visited.add(node_id)
        path.push(node_id)
        
        manifest = all_manifests.find(node_id)
        if manifest:
            for dep_id in manifest.all_dependencies():
                dfs(dep_id)
        
        path.pop()
    
    dfs(plugin_id)
```

---

## 11. Performance Targets

### 11.1 Latency Targets

| Operation | p50 | p95 | p99 | Max |
|-----------|-----|-----|-----|-----|
| Manifest validation | 100 μs | 300 μs | 500 μs | 1 ms |
| WASM compilation (cold) | 20 ms | 50 ms | 100 ms | 200 ms |
| WASM compilation (warm) | 5 ms | 15 ms | 30 ms | 50 ms |
| Plugin initialization | 100 μs | 500 μs | 1 ms | 5 ms |
| Plugin activation | 50 μs | 200 μs | 500 μs | 1 ms |
| Dependency resolution (10 plugins) | 1 ms | 3 ms | 5 ms | 10 ms |
| Dependency resolution (50 plugins) | 5 ms | 15 ms | 30 ms | 50 ms |
| Plugin function call (empty) | 0.5 μs | 1 μs | 2 μs | 5 μs |
| Plugin function call (with data) | 5 μs | 20 μs | 50 μs | 100 μs |
| KV get | 5 μs | 20 μs | 50 μs | 100 μs |
| KV put | 10 μs | 50 μs | 100 μs | 200 μs |
| HTTP get (local) | 1 ms | 5 ms | 10 ms | 50 ms |
| HTTP get (remote) | 10 ms | 100 ms | 500 ms | 2,000 ms |
| Audit log write | 1 μs | 5 μs | 10 μs | 50 μs |
| Registry sync (100 plugins) | 500 ms | 1 s | 2 s | 5 s |
| Plugin hot-swap | 50 ms | 200 ms | 500 ms | 1,000 ms |
| Resource quota check | 0.1 μs | 0.5 μs | 1 μs | 2 μs |

### 11.2 Memory Targets

| Resource | Target |
|----------|--------|
| Per inactive plugin (manifest + metadata) | < 100 KB |
| Per active plugin (WASM instance, minimal) | < 5 MB |
| Per active plugin (WASM instance, typical) | 10–50 MB |
| PluginLoader fixed overhead | < 20 MB |
| Registry database (1,000 plugins) | < 50 MB |
| Audit log buffer | 10 MB (in-memory), rotated |

### 11.3 Throughput Targets

| Scenario | Target |
|----------|--------|
| Plugin function calls per second (single instance) | > 100,000 |
| Plugin function calls per second (10 instances) | > 500,000 |
| Concurrent plugin initializations | 10 per second |
| Hot-swaps per second | 5 |

### 11.4 Optimization Recommendations

| Technique | Expected Gain | Complexity |
|-----------|---------------|------------|
| WASM caching (compiled code cache on disk) | 4–10× faster load | Low |
| Instance pooling (reuse WASM instances) | 10–50× faster call | Medium |
| Lazy initialization (defer init to first use) | 50% fewer inits | Low |
| Read-through KV cache (LRU in host) | 2–5× KV read throughput | Medium |
| Batch audit writes (async, batched) | 10× audit throughput | Medium |
| Dependency pre-resolution (cache dependency graph) | 5× faster activation | Low |
| Registry response caching | 10× faster sync | Low |
| Parallel WASM compilation | N× faster bulk load | Medium |

---

## 12. Plugin SDK Guide

### 12.1 Getting Started

The AGOS Plugin SDK provides tools for developing, testing, and packaging plugins.

```
# Install the AGOS Plugin CLI
cargo install agos-plugin-cli

# Create a new plugin project
agos plugin new my-plugin --type explanation --lang rust
cd my-plugin

# Build the plugin
agos plugin build

# Test the plugin locally
agos plugin test

# Package the plugin
agos plugin package

# Publish to registry
agos plugin publish --registry https://registry.agos.example
```

### 12.2 Project Structure

A typical plugin project:

```
my-plugin/
├── Cargo.toml                  # Rust project (or package.json for JS-based)
├── agos-plugin.json             # Plugin manifest (auto-generated or hand-written)
├── src/
│   ├── lib.rs                  # Plugin implementation
│   ├── config.rs               # Configuration parsing
│   ├── templates.rs             # Template registration
│   └── utils.rs                 # Helper functions
├── templates/                   # Template files
│   ├── overview.hbs
│   └── token.hbs
├── locales/                     # Localization files
│   ├── en.json
│   └── ar.json
├── tests/
│   ├── integration_test.rs
│   └── fixtures/
│       └── sample_analysis.json
├── docs/
│   ├── README.md
│   └── API.md
├── icon.png                     # 256×256 PNG
└── .agosignore                  # Files to exclude from package
```

### 12.3 Rust SDK Example

```rust
// src/lib.rs — Example explanation plugin using the AGOS Rust SDK

use agos_sdk::prelude::*;

// Declare plugin metadata
agos_plugin! {
    id: "com.agos.example.hello",
    name: "Hello World Example",
    version: "1.2.3",
    plugin_type: PluginType::Explanation,
    author: "AGOS Community",
    capabilities: [Capability::Computation, Capability::TemplateRead]
}

// Plugin configuration
#[derive(Debug, Deserialize)]
struct HelloConfig {
    greeting: String,
    show_confidence: bool,
}

// Main plugin struct
struct HelloPlugin {
    config: HelloConfig,
    call_count: u64,
}

// Implement the plugin lifecycle
impl PluginBase for HelloPlugin {
    fn plugin_id(&self) -> &str { "com.agos.example.hello" }
    fn plugin_name(&self) -> &str { "Hello World Example" }
    fn plugin_type(&self) -> PluginType { PluginType::Explanation }
    fn version(&self) -> &SemVer { &SemVer::new(1, 2, 3) }
    fn priority(&self) -> u16 { 100 }

    fn init(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        self.config = config.settings.deserialize::<HelloConfig>()?;
        agos_log!(Info, "Plugin initialized with greeting: {}", self.config.greeting);
        Ok(())
    }

    fn activate(&mut self, slots: &SlotRegistry) -> Result<(), PluginError> {
        slots.register("MOD-11")?;
        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), PluginError> {
        agos_log!(Info, "Plugin deactivated after {} calls", self.call_count);
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        agos_log!(Info, "Plugin shutting down");
        Ok(())
    }

    fn health(&self) -> PluginHealth { PluginHealth::Healthy }

    fn required_capabilities(&self) -> Vec<Capability> {
        vec![Capability::Computation, Capability::TemplateRead]
    }
}

// Implement type-specific methods
impl ExplanationPlugin for HelloPlugin {
    fn explain_token(
        &self,
        token: &AnalysisToken,
        context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError> {
        self.call_count += 1;

        let greeting = &self.config.greeting;
        let word = &token.text;
        let role = token.syntactic_role.as_deref().unwrap_or("unknown");

        let mut explanation = format!("{greeting} The word '{word}' has role: {role}");

        if self.config.show_confidence {
            if let Some(confidence) = token.confidence {
                explanation.push_str(&format!(" (confidence: {:.2})", confidence));
            }
        }

        Ok(Some(explanation))
    }

    fn explain_overview(
        &self,
        _result: &AnalysisResult,
        _context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError> {
        Ok(Some(format!("{} Analysis complete!", self.config.greeting)))
    }

    fn supported_languages(&self) -> Result<Vec<String>, PluginError> {
        Ok(vec!["ar".to_string(), "en".to_string()])
    }

    fn supported_formats(&self) -> Result<Vec<String>, PluginError> {
        Ok(vec!["text".to_string(), "json".to_string()])
    }

    fn register_templates(&self, registry: &mut TemplateRegistry) -> Result<(), PluginError> {
        registry.register_static("hello_greeting", "{{greeting}}")
    }
}

// Entry point — macro generates the WASM exports
impl_agos_plugin!(HelloPlugin::new(HelloConfig::default()));
```

### 12.4 JavaScript/AssemblyScript SDK

For developers who prefer TypeScript/AssemblyScript:

```typescript
// index.ts — Example plugin using AssemblyScript SDK

import { PluginBase, PluginType, ExplanationPlugin, AnalysisToken, ExplanationContext } from "@agos/sdk";

class HelloPlugin extends ExplanationPlugin {
  greeting: string = "Hello";

  init(config: Uint8Array): i32 {
    const parsed = JSON.parse(String.UTF8.decode(config));
    this.greeting = parsed.greeting || "Hello";
    agos_log(LogLevel.Info, `Plugin initialized with greeting: ${this.greeting}`);
    return 0;
  }

  explainToken(token: AnalysisToken, context: ExplanationContext): string | null {
    return `${this.greeting} The word '${token.text}' has role: ${token.syntactic_role || "unknown"}`;
  }

  explainOverview(result: AnalysisResult, context: ExplanationContext): string | null {
    return `${this.greeting} Analysis complete!`;
  }

  health(): i32 { return 0; }
}

export function agos_plugin_init(configPtr: i32, configLen: i32): i32 {
  return PluginRegistry.register(new HelloPlugin(), configPtr, configLen);
}
```

### 12.5 Testing Plugins Locally

```
# Run with a local AGOS instance
agos plugin serve --port 9090

# Send test request
curl -X POST http://localhost:9090/explain \
  -H "Content-Type: application/json" \
  -d '{
    "analysis": { ... },
    "language": "ar"
  }'

# Run test suite
agos plugin test --fixtures ./tests/fixtures/

# Check plugin health
agos plugin health my-plugin
```

### 12.6 Debugging

```
# Enable debug mode
AGOS_PLUGIN_DEBUG=1 agos plugin serve

# Trace WASM calls
AGOS_PLUGIN_TRACE=1 agos plugin serve

# Profile plugin
agos plugin profile my-plugin --calls 10000

# Get plugin logs
agos plugin logs my-plugin --tail 50

# Inspect sandbox state
agos plugin inspect my-plugin
```

---

## 13. API Reference

### 13.1 PluginLoader Public API

```rust
// ── Top-level API ─────────────────────────────────

/// Initialize the plugin system. Call once at startup.
fn init_plugin_system(config: &PluginLoaderConfig) -> Result<(), PluginLoaderError>;

/// Shutdown the plugin system. Call once at shutdown.
fn shutdown_plugin_system() -> Result<(), PluginLoaderError>;

/// Get the plugin system status.
fn plugin_system_status() -> PluginSystemStatus;

// ── Discovery & Installation ──────────────────────

/// Discover all installed plugins.
fn discover_plugins() -> Result<Vec<PluginSummary>, PluginLoaderError>;

/// Install a plugin from a package file.
fn install_plugin(package_path: &Path) -> Result<PluginSummary, PluginLoaderError>;

/// Uninstall a plugin.
fn uninstall_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

/// Update a plugin from a new package file.
fn update_plugin(plugin_id: &str, package_path: &Path) -> Result<PluginSummary, PluginLoaderError>;

// ── Lifecycle Management ──────────────────────────

/// Load and activate a plugin.
fn enable_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

/// Deactivate and unload a plugin.
fn disable_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

/// Reload a plugin (deactivate → load → activate).
fn reload_plugin(plugin_id: &str) -> Result<(), PluginLoaderError>;

/// Get the state of a plugin.
fn get_plugin_state(plugin_id: &str) -> Result<PluginState, PluginLoaderError>;

/// List all plugins with their states.
fn list_plugins() -> Result<Vec<PluginWithState>, PluginLoaderError>;

// ── Querying ──────────────────────────────────────

/// Get plugin by ID.
fn get_plugin(plugin_id: &str) -> Result<PluginInstance, PluginLoaderError>;

/// Get all active plugins of a given type.
fn get_plugins_by_type(plugin_type: PluginType) -> Result<Vec<PluginInstance>, PluginLoaderError>;

/// Get plugins targeting a specific pipeline slot.
fn get_plugins_for_slot(slot: &str) -> Result<Vec<PluginInstance>, PluginLoaderError>;

/// Check if a plugin is active.
fn is_plugin_active(plugin_id: &str) -> Result<bool, PluginLoaderError>;

// ── Configuration ─────────────────────────────────

/// Get plugin settings.
fn get_plugin_settings(plugin_id: &str) -> Result<HashMap<String, Value>, PluginLoaderError>;

/// Update plugin settings.
fn set_plugin_settings(plugin_id: &str, settings: &HashMap<String, Value>) -> Result<(), PluginLoaderError>;

/// Reset plugin settings to defaults.
fn reset_plugin_settings(plugin_id: &str) -> Result<(), PluginLoaderError>;

// ── Registry ──────────────────────────────────────

/// Sync with remote registry.
fn sync_registry() -> Result<RegistrySyncResult, PluginLoaderError>;

/// Search the remote registry.
fn search_registry(query: &str) -> Result<Vec<RegistryEntry>, PluginLoaderError>;

/// Get plugin details from remote registry.
fn get_registry_plugin(plugin_id: &str) -> Result<RegistryEntry, PluginLoaderError>;

/// Check for updates.
fn check_for_updates() -> Result<Vec<UpdateAvailable>, PluginLoaderError>;

// ── Security & Audit ──────────────────────────────

/// Get audit log entries.
fn get_audit_log(filter: &AuditFilter) -> Result<Vec<AuditEntry>, PluginLoaderError>;

/// Verify a plugin package without installing.
fn verify_package(package_path: &Path) -> Result<VerificationResult, PluginLoaderError>;

/// Get security report for a plugin.
fn get_plugin_security_report(plugin_id: &str) -> Result<SecurityReport, PluginLoaderError>;
```

### 13.2 PluginInstance Trait

```rust
/// A loaded and potentially initialized plugin instance.
trait PluginInstance: Send + Sync {
    /// Unique plugin identifier.
    fn plugin_id(&self) -> &str;

    /// Plugin type.
    fn plugin_type(&self) -> PluginType;

    /// Plugin version.
    fn version(&self) -> &SemVer;

    /// Current lifecycle state.
    fn state(&self) -> PluginState;

    /// Call the plugin's process function.
    fn process(&self, input: &[u8], output: &mut [u8]) -> Result<usize, PluginError>;

    /// Call the plugin's health check.
    fn health(&self) -> PluginHealth;

    /// Call the plugin's shutdown.
    fn shutdown(&mut self) -> Result<(), PluginError>;

    /// Get plugin metrics.
    fn metrics(&self) -> PluginMetrics;
}

struct PluginMetrics {
    pub total_calls: u64,
    pub total_errors: u64,
    pub total_time_ms: u64,
    pub avg_time_us: f64,
    pub max_time_us: u64,
    pub memory_usage: u64,
    pub kv_ops: u64,
    pub http_ops: u64,
    pub sandbox_violations: u64,
}
```

### 13.3 PluginSystemStatus

```rust
struct PluginSystemStatus {
    pub initialized: bool,
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub plugins_by_type: HashMap<PluginType, usize>,
    pub pool_size: usize,
    pub pool_active: usize,
    pub pool_idle: usize,
    pub pool_hit_rate: f64,
    pub total_calls: u64,
    pub total_errors: u64,
    pub uptime_seconds: u64,
    pub memory_usage_bytes: u64,
    pub registry_last_sync: Option<DateTime>,
    pub registry_sync_status: SyncStatus,
}

enum SyncStatus {
    Never,
    InProgress,
    Success(DateTime),
    Failed { last_attempt: DateTime, error: String },
}
```

---

## 14. Testing & Quality

### 14.1 Test Categories

| Category | Description | Coverage Target |
|----------|-------------|-----------------|
| **Unit tests** | Individual component tests (validator, resolver, loader) | 95% |
| **Integration tests** | End-to-end plugin lifecycle (discover → load → init → call → shutdown) | 90% |
| **Sandbox tests** | Verify sandbox isolation, resource limits, timeouts | 100% of T-01..T-10 |
| **Security tests** | Penetration tests for sandbox escape, capability bypass | 100% of threat model |
| **Plugin SDK tests** | Test that SDK builds produce valid plugins | 100% |
| **Registry tests** | Sync protocol, search, publish, download | 95% |
| **Performance tests** | Latency, throughput, memory under load | Meet §11 targets |
| **Upgrade tests** | Test version migration, hot-swap, rollback | 90% |

### 14.2 Test Fixture Format

```jsonc
{
  "spec": "SPEC-0601/test-fixture",
  "version": "1.0.0",
  "name": "Plugin Load Integration Test",
  "description": "Tests basic plugin loading and execution",

  "plugin_package": {
    "agos-plugin.json": { ... },
    "plugin.wasm": "<base64-encoded wasm binary>"
  },

  "setup": {
    "create_dirs": ["/tmp/agos-test/plugins", "/tmp/agos-test/cache", "/tmp/agos-test/data"],
    "env_vars": {
      "AGOS_PLUGIN_DIRS": "/tmp/agos-test/plugins",
      "AGOS_PLUGIN_CACHE_DIR": "/tmp/agos-test/cache",
      "AGOS_PLUGIN_DATA_DIR": "/tmp/agos-test/data",
      "AGOS_ALLOW_UNSIGNED": "true"
    }
  },

  "steps": [
    {
      "action": "install",
      "package": "<temp path to package file>",
      "expected": { "status": "success", "plugin_id": "com.agos.test.plugin" }
    },
    {
      "action": "enable",
      "plugin_id": "com.agos.test.plugin",
      "expected": { "state": "active" }
    },
    {
      "action": "call",
      "plugin_id": "com.agos.test.plugin",
      "input": { "text": "test" },
      "expected": { "status": "success", "output_contains": "processed" }
    },
    {
      "action": "disable",
      "plugin_id": "com.agos.test.plugin",
      "expected": { "state": "deactivated" }
    },
    {
      "action": "uninstall",
      "plugin_id": "com.agos.test.plugin",
      "expected": { "status": "success" }
    }
  ],

  "cleanup": {
    "remove_dirs": ["/tmp/agos-test"]
  }
}
```

### 14.3 Security Test Cases

| Test | Description | Expected |
|------|-------------|----------|
| S-01 | Plugin without network capability attempts HTTP call | `SandboxViolation` |
| S-02 | Plugin attempts to read outside its data directory | `SandboxViolation` |
| S-03 | Plugin allocates memory beyond limit | `MemoryExceeded` |
| S-04 | Plugin runs infinite loop | `Timeout` |
| S-05 | Plugin attempts to call other plugin's host functions | `ImportError` (compile-time) |
| S-06 | Plugin with invalid signature | `SignatureInvalid` |
| S-07 | Plugin with circular dependencies | `CircularDependency` |
| S-08 | Plugin with capability mismatch (declared vs required) | `CapabilityMismatch` |
| S-09 | Plugin with excessive requested permissions | `PermissionDenied` |
| S-10 | Unsigned plugin with `allow_unsigned_plugins=false` | `SignatureInvalid` |
| S-11 | Plugin rolls back to old signed version | `UpdateRollbackDetected` |
| S-12 | Plugin attempts to enumerate other plugins' data | `SandboxViolation` |

### 14.4 Performance Benchmark Scenarios

| Scenario | Description | Metric |
|----------|-------------|--------|
| Cold start | Load 10 plugins, all uncached | Load time |
| Warm start | Load 10 plugins, cached | Load time |
| Steady state | 100,000 plugin calls with 10 active | Throughput, latency |
| Memory pressure | Load 50 plugins, measure total memory | Memory usage |
| Concurrent calls | 100 concurrent callers, 5 plugins | Throughput, p99 latency |
| Hot-swap | Update 5 plugins simultaneously | Swap time, error count |
| Registry sync | Sync 1,000 plugin entries | Sync time |
| Security audit | 10,000 audit log entries | Write throughput, read latency |

### 14.5 Quality Gates

| Gate | Threshold | Enforcement |
|------|-----------|-------------|
| Unit test coverage | ≥ 95% | CI pipeline |
| Integration test pass | 100% | CI pipeline |
| Security test pass | 100% | CI pipeline |
| Performance regression | No regression > 5% | CI pipeline (baseline comparison) |
| Memory leak test | No leaks after 1M calls | Weekly CI run |
| Fuzz test (WASM parsing) | 1 hour no crashes | Weekly CI run |
| Code review | All changes reviewed | PR process |

---

## 15. Cross-References

### 15.1 Internal References

| Reference | Section | Relationship |
|-----------|---------|--------------|
| SPEC-0001-C2 §4.12 | PluginLoader Module | Architecture-level MOD-12 description |
| SPEC-0001-C4 §14 | MOD-12 Interface | Formal PluginLoader interface definition |
| SPEC-0001-C7 | Plugin Architecture | Architecture-level plugin design (basis for SPEC-0601) |
| SPEC-0001-C8 §3 | Error Codes | `PLUGIN_*` error codes used by MOD-12 |
| SPEC-0001-C8 §5 | Plugin Security | Plugin-specific security considerations |
| SPEC-0001-C9 §3.3 | Plugin System Performance | Performance targets for plugin loading |
| SPEC-0301 §3.1.6 | ExplanationPlugin | Plugin interface consumed by SPEC-0301 |
| SPEC-0301 §10 | LLM Integration | Plugin integration with LLM services |
| SPEC-0401 §12 | KB Resolver Plugin | `kb_resolver` plugin type used by SPEC-0401 |
| SPEC-0501 §13 | Plugin Development Guide | Plugin development for explanation engine |
| SPEC-0501 §13.2 | ExplanationPlugin Trait | Full plugin SDK example for explanation plugins |
| RFC-0004 §10 | Compilation Pipeline | `rule_set` plugin compilation |
| KB-0001–0007 | Knowledge Bases | Resolvable via `kb_resolver` plugins |

### 15.2 External References

| Reference | Description |
|-----------|-------------|
| [WASM Spec](https://webassembly.github.io/spec/core/) | WebAssembly Core Specification 2.0 |
| [WASI Preview 2](https://github.com/WebAssembly/WASI) | WASI system interface |
| [wasmtime](https://wasmtime.dev/) | WASM runtime (recommended for Rust host) |
| [wasmer](https://wasmer.io/) | Alternative WASM runtime |
| [SemVer 2.0](https://semver.org/) | Semantic versioning specification |
| [MsgPack](https://msgpack.org/) | MessagePack serialization format |
| [Ed25519](https://ed25519.cr.yp.to/) | Ed25519 signature algorithm |
| [TUF](https://theupdateframework.io/) | The Update Framework (reference for update security) |
| [SLSA](https://slsa.dev/) | Supply-chain Levels for Software Artifacts |

### 15.3 KB Dependencies

| KB | How Plugin System Uses It |
|----|--------------------------|
| KB-0001 | Plugin may register custom root resolvers via `kb_resolver` |
| KB-0002 | Plugin may register custom wazan resolvers via `kb_resolver` |
| KB-0005 | `rule_set` plugins may define custom particle rules |
| KB-0007 | `explanation` plugins may reference feature names from taxonomy |

### 15.4 Glossary

| Term | Definition |
|------|------------|
| **Plugin** | A self-contained WASM binary with a manifest that extends AGOS functionality |
| **Plugin type** | One of 9 categories that define which pipeline slots a plugin can target |
| **Manifest** | `agos-plugin.json` file containing plugin metadata, requirements, and permissions |
| **Capability** | A named permission that grants access to a set of host functions |
| **Slot** | A named injection point in the pipeline where plugins can register |
| **Sandbox** | An isolated execution environment (WASM instance with resource limits) |
| **ABI** | Application Binary Interface — the calling convention between host and plugin |
| **Hot-swap** | Replacing an active plugin at runtime without stopping the pipeline |
| **Registry** | A remote HTTP service for discovering and downloading plugins |
| **Lifecycle** | The sequence of states a plugin passes through from discovery to shutdown |
| **Dependency graph** | Directed graph of plugin dependencies, resolved topologically |
| **Drain** | Allowing in-flight plugin calls to complete before deactivation |

---

*End of SPEC-0601*
