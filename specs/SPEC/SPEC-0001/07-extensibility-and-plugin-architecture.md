---
spec_id: SPEC-0001
chapter: 7
title: Extensibility & Plugin Architecture
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C6: Deployment & Runtime Considerations
  - SPEC-0601: Plugin System
  - RFC-0001: Grammar DSL (proposed)
  - RFC-0002: Grammar Bytecode (proposed)
  - ADR-0005: Why Plugin Architecture
  - KB-0001: Roots (planned)
  - KB-0007: Morphological Features Taxonomy (planned)
---

# Chapter 7: Extensibility & Plugin Architecture

## Table of Contents

1. [Extensibility Philosophy](#1-extensibility-philosophy)
2. [Plugin System Architecture](#2-plugin-system-architecture)
3. [Plugin Types](#3-plugin-types)
4. [Plugin Lifecycle](#4-plugin-lifecycle)
5. [Plugin API & SDK](#5-plugin-api--sdk)
6. [Sandboxing & Security](#6-sandboxing--security)
7. [Grammar DSL Overview](#7-grammar-dsl-overview)
8. [School-Specific Rule Sets](#8-school-specific-rule-sets)
9. [Custom Knowledge Base Plugins](#9-custom-knowledge-base-plugins)
10. [Explanation & Application Plugins](#10-explanation--application-plugins)
11. [Plugin Distribution & Registry](#11-plugin-distribution--registry)
12. [Plugin Testing & Validation](#12-plugin-testing--validation)
13. [Cross-References](#13-cross-references)

---

## 1. Extensibility Philosophy

AGOS is designed from the ground up as an extensible platform. The following principles govern all extensibility mechanisms:

### 1.1 Principles

1. **The core is minimal and stable.** The compilation pipeline (MOD-01 through MOD-11) is a fixed, well-defined core. All variability — grammar schools, language variants, custom explanations — is handled through plugins.

2. **Plugins cannot break the pipeline.** A plugin operates within a sandbox and cannot crash the pipeline, corrupt memory, or access unauthorized resources. The worst consequence of a failing plugin is that the plugin's specific extension is unavailable.

3. **Plugins are versioned independently.** Each plugin declares its own version and the AGOS API version it requires. The PluginLoader verifies compatibility before loading.

4. **Plugins are swappable at runtime.** Plugins can be loaded, unloaded, and reloaded without restarting the pipeline. This enables hot-update of rule sets and knowledge bases.

5. **Everything is a plugin.** Grammar schools, language variants, morphological engines, syntactic theories, explanation templates, and data sources are all plugins. Nothing is hard-coded into the core.

### 1.2 What Can Be Extended

| Extension Point | Plugin Type | Example |
|----------------|-------------|---------|
| Text normalization | `normalizer` | Quranic Uthmani script handler |
| Token classification | `token_classifier` | Poetic meter tokenizer |
| Token segmentation | `segmenter` | Dialect-specific clitic rules |
| Morphological analysis | `morphology_engine` | Alternative root extraction for a specific school |
| Syntactic parsing | `syntax_engine` | Dependency grammar vs. constituency grammar |
| Grammatical rules | `rule_set` | Basra school, Kufa school, Andalus school |
| Knowledge base resolution | `kb_resolver` | Custom dictionary, loanword database |
| Explanation generation | `explanation` | Gamified explanation for children, Urdu translations |
| API middleware | `api_middleware` | Custom authentication, usage tracking |

### 1.3 What Cannot Be Extended (Core Invariants)

| Aspect | Reason |
|--------|--------|
| Pipeline stage order | The pipeline is the fundamental architecture (ADR-0001) |
| GIR format | All stages depend on a common IR format |
| Bytecode format | GVM depends on a stable instruction set |
| Evidence trail format | Explainability depends on a standard evidence format |
| Determinism guarantee | Core Principle 10 prohibits non-deterministic extensions |

---

## 2. Plugin System Architecture

### 2.1 High-Level Architecture

```
                    ┌─────────────────────┐
                    │   Plugin Registry   │
                    │  (index of all      │
                    │   available plugins)│
                    └─────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────┐
│                  PluginLoader (MOD-12)           │
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Discover │→ │ Validate │→ │  Load    │       │
│  │ plugins  │  │ manifest │  │ binary   │       │
│  └──────────┘  └──────────┘  └──────────┘       │
│                                      │           │
│  ┌──────────┐  ┌──────────┐  ┌───────▼──────┐   │
│  │ Unload   │← │  Inject  │← │ Verify      │   │
│  │ plugin   │  │ into     │  │ compat.     │   │
│  └──────────┘  │ pipeline │  └─────────────┘   │
│                └──────────┘                     │
└─────────────────────────────────────────────────┘
        │              │               │
        ▼              ▼               ▼
┌────────────┐ ┌────────────┐ ┌────────────────┐
│ MOD-01..09 │ │ MOD-10..11 │ │ Knowledge Layer│
│ Compilation│ │  Runtime   │ │ (KB extensions) │
│ Stages     │ │  Stages    │ │                │
└────────────┘ └────────────┘ └────────────────┘
        │              │               │
        │   Plugin Injection Points    │
        └──────────────┼───────────────┘
                       ▼
           ┌─────────────────────┐
           │   PipelineContext   │
           │  (injected into     │
           │   plugin at call)   │
           └─────────────────────┘
```

### 2.2 Plugin Injection Points

```
Pipeline Flow with Plugin Injection:

Input Text
    │
    ▼
MOD-01: UnicodeValidator ───► [normalizer plugin]*
    │
    ▼
MOD-02: Lexer ───► [token_classifier plugin]*
    │
    ▼
MOD-03: Tokenizer ───► [segmenter plugin]*
    │
    ▼
MOD-04: MorphologicalParser ───► [morphology_engine plugin]*
    │
    ▼
MOD-05: SyntaxParser ───► [syntax_engine plugin]*
    │
    ▼
MOD-06: GIRConstructor
    │
    ▼
MOD-07: RuleEngine ───► [rule_set plugin]* (one per school)
    │
    ▼
MOD-08: KnowledgeGraphResolver ───► [kb_resolver plugin]*
    │
    ▼
MOD-09: BytecodeGenerator
    │
    ▼
MOD-10: GVM
    │
    ▼
MOD-11: ExplanationEngine ───► [explanation plugin]*
    │
    ▼
MOD-14: APIGateway ───► [api_middleware plugin]*
    │
    ▼
Output

[*] Optional: pipeline functions without any plugins loaded
```

### 2.3 Plugin Chain Order

When multiple plugins of the same type are loaded, they form a **chain**. Each plugin in the chain receives the output of the previous plugin:

```
Pipeline Stage Output
    │
    ▼
Plugin A (type: normalizer, priority: 100)
    │
    ▼
Plugin B (type: normalizer, priority: 50)
    │
    ▼
Plugin C (type: normalizer, priority: 10)
    │
    ▼
Pipeline Stage Input (next stage)

[Ordered by priority descending; higher priority runs first]
```

---

## 3. Plugin Types

### 3.1 Plugin Type Catalog

Each plugin type is defined by:
- The interface it implements
- The pipeline stage it injects into
- The data type it processes and returns

| Type ID | Interface | Injection Point | Input Type | Output Type |
|---------|-----------|-----------------|------------|-------------|
| `normalizer` | `NormalizerPlugin` | MOD-01 | `UnicodeValidatorInput` | `UnicodeValidatorInput` |
| `token_classifier` | `TokenClassifierPlugin` | MOD-02 | `LexerInput` | `LexerInput` |
| `segmenter` | `SegmenterPlugin` | MOD-03 | `TokenizerInput` | `TokenizerInput` |
| `morphology_engine` | `MorphologyPlugin` | MOD-04 | `MorphologicalParserInput` | `MorphologicalParserInput` |
| `syntax_engine` | `SyntaxPlugin` | MOD-05 | `SyntaxParserInput` | `SyntaxParserInput` |
| `rule_set` | `RuleSetPlugin` | MOD-07 | `GrammarIR` | `AnnotatedGIR` |
| `kb_resolver` | `KBResolverPlugin` | MOD-08 | `AnnotatedGIR` | `ResolvedGIR` |
| `explanation` | `ExplanationPlugin` | MOD-11 | `AnalysisResult` | `ExplanationOutput` |
| `api_middleware` | `APIMiddlewarePlugin` | MOD-14 | `AnalyzeRequest` | `AnalyzeRequest` |

### 3.2 Plugin Interface Definitions

#### NormalizerPlugin

```
trait NormalizerPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "normalizer";
    fn priority() -> integer;                    // Higher runs first
    fn supported_scripts() -> string[];          // e.g., ["uthmani", "imla'i"]

    fn normalize(input: UnicodeValidatorInput, context: PipelineContext)
        -> Result<UnicodeValidatorInput, PluginError>;
}
```

#### TokenClassifierPlugin

```
trait TokenClassifierPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "token_classifier";
    fn priority() -> integer;

    fn classify(input: LexerInput, context: PipelineContext)
        -> Result<LexerInput, PluginError>;
}
```

#### SegmenterPlugin

```
trait SegmenterPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "segmenter";
    fn priority() -> integer;
    fn supported_dialects() -> string[];         // e.g., ["egyptian", "levantine"]

    fn segment(input: TokenizerInput, context: PipelineContext)
        -> Result<TokenizerInput, PluginError>;
}
```

#### MorphologyPlugin

```
trait MorphologyPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "morphology_engine";
    fn priority() -> integer;
    fn supported_schools() -> string[];

    fn analyze_morphology(input: MorphologicalParserInput, context: PipelineContext)
        -> Result<MorphologicalParserInput, PluginError>;
}
```

#### SyntaxPlugin

```
trait SyntaxPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "syntax_engine";
    fn priority() -> integer;
    fn supported_schools() -> string[];

    fn parse_syntax(input: SyntaxParserInput, context: PipelineContext)
        -> Result<SyntaxParserInput, PluginError>;
}
```

#### RuleSetPlugin

```
trait RuleSetPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "rule_set";
    fn priority() -> integer;
    fn school() -> string;                       // e.g., "basra"
    fn rule_set_version() -> string;             // Semver
    fn rule_count() -> integer;

    fn apply_rules(input: GrammarIR, context: PipelineContext)
        -> Result<AnnotatedGIR, PluginError>;
}
```

#### KBResolverPlugin

```
trait KBResolverPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "kb_resolver";
    fn priority() -> integer;
    fn supported_kb_types() -> string[];         // e.g., ["dictionary", "etymology"]

    fn resolve(input: AnnotatedGIR, context: PipelineContext)
        -> Result<ResolvedGIR, PluginError>;
}
```

#### ExplanationPlugin

```
trait ExplanationPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "explanation";
    fn priority() -> integer;
    fn supported_languages() -> string[];
    fn supported_formats() -> string[];

    fn explain(input: AnalysisResult, context: PipelineContext)
        -> Result<ExplanationOutput, PluginError>;
}
```

#### APIMiddlewarePlugin

```
trait APIMiddlewarePlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "api_middleware";
    fn priority() -> integer;
    fn route_pattern() -> string;                // e.g., "/v1/analyze"

    fn process_request(input: AnalyzeRequest, context: PipelineContext)
        -> Result<AnalyzeRequest, PluginError>;

    fn process_response(input: AnalyzeResponse, context: PipelineContext)
        -> Result<AnalyzeResponse, PluginError>;
}
```

### 3.3 PipelineContext (Shared Context)

Every plugin receives a `PipelineContext` at invocation:

```
type PipelineContext = {
    request_id: string,                          // Unique request ID for tracing
    stage_id: string,                            // e.g., "MOD-04"
    pipeline_config: {
        school: string,
        mode: string,
        knowledge_versions: KnowledgeVersionMap,
    },
    plugin_config: { [key: string]: any },       // Plugin-specific configuration
    log: (level: LogLevel, message: string) => void,  // Logger callback
    cache: CacheManager,                         // Access to shared cache
    metrics: MetricsCollector,                   // For custom metrics
}
```

---

## 4. Plugin Lifecycle

### 4.1 Lifecycle States

```
                    ┌─────────────┐
                    │  DISCOVERED │
                    └──────┬──────┘
                           │ validate
                           ▼
                    ┌─────────────┐
                    │  VALIDATED  │
                    └──────┬──────┘
                           │ load
                           ▼
                    ┌─────────────┐
              ┌─────│   LOADED    │─────┐
              │     └──────┬──────┘     │
              │            │ inject     │
              │            ▼            │
              │     ┌─────────────┐     │
              │     │  ACTIVE     │     │
              │     └──────┬──────┘     │
              │            │            │
              │    ┌───────┴───────┐    │
              │    │               │    │
              ▼    ▼               ▼    ▼
       ┌──────────┐         ┌──────────┐
       │ UNLOADED │         │  ERROR   │
       └──────────┘         └──────────┘
```

| State | Description | Transitions |
|-------|-------------|-------------|
| **DISCOVERED** | Plugin manifest found in plugin directory | → VALIDATED |
| **VALIDATED** | Manifest passed schema validation; dependencies resolved | → LOADED |
| **LOADED** | Plugin binary loaded into memory; not yet injected | → ACTIVE, UNLOADED |
| **ACTIVE** | Plugin injected into pipeline; processing requests | → UNLOADED, ERROR |
| **ERROR** | Plugin encountered a runtime error | → UNLOADED |
| **UNLOADED** | Plugin removed from memory; resources released | Terminal state |

### 4.2 Discovery

Plugin discovery scans configured directories for plugin manifests:

```
Algorithm: discover_plugins
Input: plugin_directories (string[])

For each directory in plugin_directories:
    For each file matching *.agos-plugin.yaml or *.agos-plugin.json:
        1. Parse manifest
        2. Validate against manifest schema
        3. If valid → add to discovered list
        4. If invalid → log warning with validation errors

Return: discovered plugin manifests
```

### 4.3 Validation

```
Algorithm: validate_plugin
Input: manifest (PluginManifest)

1. Schema Validation:
   - Validate manifest against the PluginManifest JSON Schema
   - If invalid → return errors with field paths

2. Version Compatibility:
   - Check manifest.api_version against current AGOS API version
   - MAJOR must match; MINOR must be ≤ current; PATCH ignored
   - If incompatible → return PLUGIN_VERSION_MISMATCH

3. Dependency Resolution:
   - For each dependency in manifest.dependencies:
     a. Find the dependency in loaded or discovered plugins
     b. Check dependency version compatibility
     c. If not found → return PLUGIN_DEPENDENCY_MISSING
   - Check for circular dependencies (A → B → A)
   - If circular → return PLUGIN_CIRCULAR_DEPENDENCY

4. Permission Validation:
   - Check that requested permissions are allowed by server config
   - If server disallows "network_access" and plugin requests it →
     return PLUGIN_PERMISSION_DENIED

Return: valid = true | valid = false with errors
```

### 4.4 Loading

```
Algorithm: load_plugin
Input: manifest (PluginManifest)

1. Load Plugin Binary:
   - Read the binary from manifest.entry_point
   - Supported binary formats: WASM (.wasm), native (.so/.dylib/.dll)
   - WASM is the RECOMMENDED format for cross-platform compatibility

2. Sandbox Initialization:
   - Create sandbox environment (see Section 6)
   - Allocate memory limits
   - Set up allowed API surface

3. Instantiation:
   - Call the plugin's init() function (if WASM: call _start or init export)
   - Pass initial configuration from manifest
   - Verify the plugin responds to plugin_id() and plugin_type()

4. Register with PluginLoader:
   - Add to loaded_plugins registry
   - Record load time and memory usage

Return: PluginInstance (or error)
```

### 4.5 Injection

```
Algorithm: inject_plugin
Input: plugin (PluginInstance), pipeline (PipelineOrchestrator)

1. Determine Injection Point:
   - Based on plugin.plugin_type(), find the target stage in the pipeline

2. Register Plugin:
   - Add the plugin to the stage's plugin chain
   - Order by plugin.priority() (descending)

3. Verify Injection:
   - Send a test input through the stage with the plugin
   - Verify the plugin processes and returns correctly
   - If test fails → roll back injection, move plugin to ERROR state

4. Mark Active:
   - Update plugin state to ACTIVE
   - Log successful injection

Return: success
```

### 4.6 Unloading

```
Algorithm: unload_plugin
Input: plugin_id (string)

1. Deactivate:
   - Remove plugin from pipeline stage's plugin chain
   - Wait for any in-flight requests using this plugin to complete
     (configurable timeout, default: 30 seconds)

2. Shutdown:
   - Call the plugin's shutdown() function
   - Release allocated resources (memory, file handles, network connections)

3. Deregister:
   - Remove from loaded_plugins registry
   - Update state to UNLOADED

4. Notify:
   - Log successful unload
   - If other plugins depended on this one, move them to ERROR state

Return: success
```

### 4.7 Hot-Reload

```
Algorithm: reload_plugin
Input: plugin_id (string), new_manifest (PluginManifest | null)

1. If new_manifest is null → re-read manifest from disk

2. Version Check:
   - If major version changed → full unload + load (may affect dependencies)
   - If minor/patch changed → hot reload (swap in-place)

3. Hot Reload (same major version):
   - Load new plugin binary into separate memory space
   - Initialize new instance with existing configuration
   - Route new requests to new instance
   - Drain old instance (wait for in-flight requests)
   - Unload old instance
   - Update registry with new version

4. Cold Reload (new major version):
   - Unload plugin (Section 4.6)
   - Validate new manifest
   - Load new plugin
   - Inject into pipeline

Return: success
```

---

## 5. Plugin API & SDK

### 5.1 Plugin Manifest Schema

```yaml
# Example: basra-school-rule-set.agos-plugin.yaml
api_version: "1.0"                        # AGOS plugin API version
id: "basra-rules-v2"                      # Unique plugin ID
name: "Basra School Grammar Rules v2"     # Human-readable name
version: "2.3.1"                          # Plugin version (semver)
plugin_type: "rule_set"                   # From PluginType enum
author:
  name: "AGOS Grammar Committee"
  email: "grammar@agos.org"
  url: "https://agos.org/plugins/basra-rules"
description: >
  Complete Basra school grammar rule set.
  Covers nahw, sarf, and i'rab according to the Basra tradition.
  Includes rules by Sibawayh, Al-Akhfash, and Al-Mubarrad.

entry_point: "basra-rules.wasm"            # Plugin binary
language: "rust"                           # Implementation language
compiled_for: ["linux_x86_64", "macos_arm64", "wasm32"]

dependencies:                               # Other plugins required
  - id: "kb-roots-v2"
    min_version: "1.0.0"
    max_version: "2.0.0"

permissions:                                # Sandbox permissions
  - "read_kb"                               # Can read knowledge bases
  # "network_access"                        # NOT requested

config_schema:                             # JSON Schema for plugin config
  type: object
  properties:
    strict_mode:
      type: boolean
      default: false
    max_rules_per_sentence:
      type: integer
      default: 100
  required: []

supported_schools: ["basra", "baghdad"]
rule_count: 1250
rule_set_version: "2.3.1"
```

### 5.2 Plugin SDK

A Plugin SDK SHOULD be provided for the following languages:

| Language | SDK Status | Format |
|----------|-----------|--------|
| **Rust** | Primary SDK | Compiles to WASM + native |
| **C** | Secondary SDK | Compiles to native .so/.dylib |
| **Go** | Community SDK | Compiles to WASM (via TinyGo) |
| **AssemblyScript** | Community SDK | Compiles to WASM |

The SDK provides:
- Plugin trait definitions (Rust traits, C structs with function pointers)
- PipelineContext type with logging, caching, and metrics helpers
- Serialization helpers for plugin input/output types
- Test harness for offline plugin testing
- Build tooling (Cargo WASM build, size optimization)

### 5.3 Plugin Binary Format

Plugins are compiled to **WebAssembly (WASM)** as the primary binary format:

| Property | Specification |
|----------|--------------|
| **Format** | Core WebAssembly MVP (no GC or exception handling dependencies) |
| **WASI** | Optional; only for KB-reading plugins |
| **Memory** | 64 MB initial; up to 256 MB maximum |
| **Exports** | `init`, `process`, `shutdown` (see below) |
| **Imports** | Limited to `agos:plugin/*` namespace |
| **Size target** | < 5 MB compressed; < 20 MB uncompressed |

#### Required WASM Exports

```wasm
;; All plugins MUST export:
(export "agos_plugin_init" (func $init))           ;; (env) -> void
(export "agos_plugin_process" (func $process))     ;; (ptr, len) -> ptr
(export "agos_plugin_shutdown" (func $shutdown))   ;; (env) -> void
(export "agos_plugin_memory_required" (func $mem))  ;; () -> i32 (pages)

;; RuleSetPlugin additionally exports:
(export "agos_rule_count" (func $rule_count))      ;; () -> i32
(export "agos_rule_school" (func $rule_school))    ;; (ptr, len) -> void
```

### 5.4 WASM Memory Model

```
Plugin WASM Memory Layout:
┌──────────────────────────────────────┐
│  0x00000000 - 0x0000FFFF: Reserved   │
│  (AGOS runtime communication area)   │
├──────────────────────────────────────┤
│  0x00010000 - 0x0001FFFF: Input      │
│  (Serialized plugin input data)      │
├──────────────────────────────────────┤
│  0x00020000 - 0x0002FFFF: Output     │
│  (Serialized plugin output data)     │
├──────────────────────────────────────┤
│  0x00030000 - 0x0003FFFF: Config     │
│  (Plugin configuration JSON)         │
├──────────────────────────────────────┤
│  0x00040000 - 0xFFFFFFFF: Heap       │
│  (Plugin's own allocations)          │
└──────────────────────────────────────┘
```

Communication between AGOS and the plugin uses **shared memory**:
1. AGOS writes serialized input to the Input region.
2. AGOS calls `agos_plugin_process(input_ptr, input_len)`.
3. Plugin reads input, processes, writes output to Output region.
4. Plugin returns pointer to output data.
5. AGOS reads output from the returned pointer.

---

## 6. Sandboxing & Security

### 6.1 Sandbox Architecture

Plugins execute in a sandboxed environment with the following restrictions:

```
┌──────────────────────────────────────────────┐
│                 Host Process                  │
│                                               │
│  ┌────────────────────────────────────────┐   │
│  │           Plugin Sandbox              │   │
│  │                                        │   │
│  │  ┌────────────────────────────────┐    │   │
│  │  │  Plugin WASM Instance          │    │   │
│  │  │  - No file I/O                │    │   │
│  │  │  - No network access          │    │   │
│  │  │  - No system calls            │    │   │
│  │  │  - No process spawning        │    │   │
│  │  │  - Limited memory (64-256 MB) │    │   │
│  │  │  - Bounded execution (100ms)  │    │   │
│  │  └────────────────────────────────┘    │   │
│  │                                        │   │
│  │  Allowed API Surface:                  │   │
│  │  - agos:plugin/log                     │   │
│  │  - agos:plugin/config                 │   │
│  │  - agos:plugin/kb_read                │   │
│  │  - agos:plugin/cache_get/set           │   │
│  └────────────────────────────────────────┘   │
└──────────────────────────────────────────────┘
```

### 6.2 WASM Sandbox (Primary)

For WASM-based plugins, the sandbox is enforced by the WASM runtime itself:

| Restriction | Enforcement |
|-------------|-------------|
| **No file I/O** | WASM has no file system access unless WASI is explicitly enabled |
| **No network access** | WASM cannot make network calls |
| **No system calls** | WASM has no syscall interface |
| **Memory limits** | WASM memory is pre-allocated and bounded (configurable: 64–256 MB) |
| **Execution limits** | WASM execution is bounded by instruction count (configurable) |
| **No infinite loops** | WASM runtime detects and terminates non-terminating executions |

### 6.3 Native Plugin Sandbox (Secondary)

For native plugins (`.so`/`.dylib`), a process-level sandbox is used:

| Restriction | Enforcement |
|-------------|-------------|
| **No file I/O** | seccomp (Linux) or sandbox_init (macOS) blocks file-related syscalls |
| **No network access** | seccomp blocks socket-related syscalls |
| **Memory limits** | setrlimit RLIMIT_AS to limit virtual memory |
| **Execution limits** | Signal-based timeout (SIGALRM / SIGXCPU) |
| **No process spawning** | seccomp blocks fork/clone/execve |

### 6.4 Permission Model

Each plugin declares required permissions in its manifest:

| Permission | Description | Risk Level |
|------------|-------------|------------|
| `read_kb` | Read knowledge base data | Low |
| `write_kb` | Modify knowledge base data | High |
| `read_cache` | Read cache entries | Low |
| `write_cache` | Write cache entries | Medium |
| `network_access` | Make network requests | Critical |
| `file_read` | Read files from plugin directory | Medium |
| `file_write` | Write files to plugin directory | High |
| `process_spawn` | Spawn subprocesses | Critical |

Permissions not requested in the manifest are denied at the sandbox level.

### 6.5 Plugin Verification

Before loading, plugins are verified:

```
Algorithm: verify_plugin
Input: plugin_binary (bytes), manifest (PluginManifest)

1. Integrity Check:
   - Verify SHA-256 checksum (if provided in manifest)
   - WASM: verify valid WASM module structure (magic + version)

2. Capability Analysis (WASM only):
   - Scan WASM imports to verify they match declared permissions
   - If plugin imports "wasi_unstable" but didn't declare file_read → reject
   - If plugin imports "env:execve" → reject (not in allowed imports)

3. Size Check:
   - Verify plugin binary size < configured maximum (default: 20 MB)

4. Test Execution:
   - Execute plugin with a minimal test input
   - Verify it returns within timeout (default: 100 ms)
   - Verify output type matches expected type

Return: verified = true | verified = false with reason
```

---

## 7. Grammar DSL Overview

### 7.1 Purpose

The Grammar DSL (Domain-Specific Language) is the language in which grammatical rules are authored. It is defined in full in RFC-0001. This section provides an overview of its design and capabilities.

### 7.2 Design Goals

1. **Linguist-friendly syntax.** Rules read like natural language grammatical descriptions, not programming code.
2. **Deterministic semantics.** Every DSL construct has a well-defined, deterministic meaning.
3. **School-agnostic.** The same DSL is used for all grammar schools; school-specific behavior comes from different rule sets.
4. **Testable.** Each rule can be tested in isolation with known inputs and expected outputs.
5. **Versionable.** Rule sets are plain text files that can be version-controlled.

### 7.3 Rule Structure

A rule is composed of three parts: **condition**, **action**, and **metadata**.

```
rule "basra-0103: Subject-Verb Person Agreement" {
    metadata {
        id: "basra-0103"
        school: "basra"
        version: "1.2.0"
        priority: 50
        description: "In verbal sentences, the verb must agree with the subject in person."
        source: "Sibawayh, Al-Kitab, Vol. 1, p. 234"
        tags: ["agreement", "fi'l", "fa'il", "verbal-sentence"]
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        fi'l.person != fa'il.person
    }

    action {
        reject("Subject-verb person disagreement")
        flag("error", "SUBJECT_VERB_PERSON_MISMATCH", fi'l, fa'il)
    }
}
```

### 7.4 DSL Constructs

| Construct | Example | Description |
|-----------|---------|-------------|
| `rule` | `rule "id: Title" { ... }` | Define a rule with metadata, condition, and action |
| `metadata` | `metadata { id: "...", priority: 50 }` | Rule metadata block |
| `condition` | `condition { ... }` | Logical expression that triggers the rule |
| `action` | `action { ... }` | What the rule does when condition is met |
| `confirm` | `confirm(analysis_id)` | Mark an ambiguous analysis as correct |
| `reject` | `reject("reason")` | Remove an ambiguous analysis |
| `modify` | `modify(token.features.case, "accusative")` | Change a feature value |
| `flag` | `flag("error", "CODE", ...)` | Add a grammatical flag |
| `resolve` | `resolve(pronoun, antecedent)` | Resolve anaphora |
| `if` | `if (expr) { ... }` | Conditional branching within action |
| `forall` | `forall(token in sentence.tokens) { ... }` | Iteration over tokens/constituents |
| `exists` | `exists(role == "fa'il")` | Existence check |
| `matches` | `token.root matches "كتب"` | Pattern/regex matching |
| `import` | `import "basra-core"` | Import rules from another file |

### 7.5 Built-in Variables

The DSL provides access to the GIR state:

| Variable | Type | Description |
|----------|------|-------------|
| `sentence` | `Sentence` | The current sentence being analyzed |
| `token` | `Token` | A token in the sentence |
| `fi'l` | `Token` | The verb (if any) |
| `fa'il` | `Token` | The subject (if any) |
| `mubtada'` | `Token` | The topic in nominal sentences |
| `khabar` | `Token` | The comment in nominal sentences |
| `constituent` | `Constituent` | A syntactic constituent |
| `sentence.type` | `string` | Sentence type |
| `token.features` | `FeatureMap` | Morphological features |
| `token.role` | `string` | Syntactic role |

### 7.6 Example Rule Sets

#### Basra School: Preposition Governs Genitive

```
rule "basra-0201: Preposition governs genitive case" {
    metadata {
        id: "basra-0201"
        school: "basra"
        priority: 30
    }

    condition {
        token.role == "majrur"
        governing_particle.type == "harf_jarr"
        token.features.case != "genitive"
    }

    action {
        modify(token.features.case, "genitive")
        confirm(token)
    }
}
```

#### Kufa School: Different Agreement Rule

```
rule "kufa-0104: Verb before plural non-human subject" {
    metadata {
        id: "kufa-0104"
        school: "kufa"
        priority: 45
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        fa'il.features.number == "plural"
        fa'il.features.gender == "feminine"  // or non-human masculine
        fi'l.position < fa'il.position       // verb before subject
    }

    action {
        // Kufa school: verb takes feminine singular with non-human plural subject
        modify(fi'l.features.gender, "feminine")
        modify(fi'l.features.number, "singular")
        confirm()
    }
}
```

---

## 8. School-Specific Rule Sets

### 8.1 School Plugin Architecture

Each grammar school is a `rule_set` plugin containing:
- A set of DSL rule files
- A rule priority configuration
- School-specific morphological preferences
- School-specific syntactic rules

### 8.2 Supported Schools (Default)

| School | Plugin ID | Rule Count | Status |
|--------|-----------|------------|--------|
| **Basra** (البصرة) | `rules-basra` | ~1,250 | Planned |
| **Kufa** (الكوفة) | `rules-kufa` | ~1,100 | Planned |
| **Baghdad** (بغداد) | `rules-baghdad` | ~800 | Planned |
| **Andalus** (الأندلس) | `rules-andalus` | ~600 | Planned |
| **Modern** (حديث) | `rules-modern` | ~400 | Planned |

### 8.3 School Comparison Matrix

| Feature | Basra | Kufa | Baghdad | Andalus |
|---------|-------|------|---------|---------|
| **Founder** | Abu al-Aswad al-Du'ali | Al-Kisa'i | Ibn al-Sarraj | Ibn Malik |
| **Analogical method** | Strict qiyas | Broader qiyas | Synthesist | Grammatical |
| **Rare constructions** | Reject | Accept | Case-by-case | Document |
| **Quranic readings** | Some accepted | Widely accepted | Accepted | Accepted |
| **Poetic evidence** | Limited | Extensive | Moderate | Extensive |
| **Verb agreement** | Strict | Flexible (with order) | Mixed | Mixed |

### 8.4 Creating a New School Plugin

To create a new grammar school as a plugin:

1. **Author rules** in Grammar DSL (one `.agosrule` file per logical category).
2. **Define priority** order for rule application.
3. **Package** as a WASM module with the `rule_set` plugin interface.
4. **Publish** to the AGOS plugin registry.
5. **Test** against the AGOS conformance test suite.

---

## 9. Custom Knowledge Base Plugins

### 9.1 KB Plugin Types

Custom KB plugins extend the knowledge bases that the pipeline can resolve against:

| KB Plugin Type | Extends | Example Use Case |
|----------------|---------|------------------|
| `dictionary` | KB definitions | Hans Wehr dictionary integration |
| `etymology` | Root origins | Quranic Aramaic etymology lookup |
| `semantic_network` | Semantic relations | WordNet-style semantic relations |
| `loanword` | Loanword detection | Persian/Turkish loanwords in Arabic |
| `corpus` | Usage examples | Quranic verse examples, Hadith citations |
| `translation` | Word translations | Multi-language word translations |

### 9.2 KB Plugin Interface

```
trait KBPlugin {
    fn plugin_id() -> string;
    fn plugin_type() -> "kb_resolver";
    fn supported_kb_types() -> string[];

    fn lookup(input: KBLookupRequest, context: PipelineContext)
        -> Result<KBLookupResponse, PluginError>;
}

type KBLookupRequest = {
    query_type: "root" | "word" | "pattern" | "semantic",
    query_value: string,                         // e.g., root text, word text
    max_results: integer,
    language: string | null,                     // Preferred language
}

type KBLookupResponse = {
    entries: KBEntry[],
    total_found: integer,
    query_time_ms: float,
}

type KBEntry = {
    id: string,
    source: string,                               // Plugin ID
    relevance_score: float,
    data: { [key: string]: any },                 // Plugin-specific data
}
```

### 9.3 KB Plugin Example: Hans Wehr Dictionary

A Hans Wehr dictionary plugin would:
1. Load the dictionary data as a compressed binary index at plugin init time.
2. On each lookup request, search the index for the queried root or word.
3. Return dictionary entries with definitions, root references, and examples.
4. Cache frequently accessed entries in the shared cache.

---

## 10. Explanation & Application Plugins

### 10.1 Explanation Plugin Types

| Plugin Type | Use Case | Examples |
|-------------|----------|----------|
| **Language template** | Localized explanations | Arabic, English, Urdu, Malay, French |
| **Pedagogical style** | Age/level-appropriate | Beginner, intermediate, advanced |
| **Gamification** | Educational games | Quiz generator, fill-in-the-blank |
| **Accessibility** | Special needs support | Screen-reader optimized, simplified text |
| **LLM prompt** | Custom LLM prompting | Domain-specific explanation prompts |

### 10.2 Creating an Explanation Plugin

```
// Example: Urdu explanation plugin (conceptual)
impl ExplanationPlugin for UrduExplanationPlugin {
    fn plugin_id() -> string { "explanation-urdu" }
    fn supported_languages() -> string[] { ["ur"] }
    fn supported_formats() -> string[] { ["text", "html"] }

    fn explain(analysis: AnalysisResult, context: PipelineContext)
        -> Result<ExplanationOutput, PluginError>
    {
        // 1. Load Urdu templates
        // 2. Generate I'rab breakdown in Urdu script
        // 3. Translate grammatical terms to Urdu
        // 4. Generate overview in Urdu
        // 5. Return ExplanationOutput with urdu text
    }
}
```

### 10.3 API Middleware Plugins

API middleware plugins allow customization of the API layer:

| Middleware | Purpose | Example |
|-----------|---------|---------|
| **Authentication** | Custom auth provider | OAuth, SAML, custom JWT |
| **Rate limiting** | Custom rate limiting | Per-user, per-tier, per-endpoint |
| **Audit logging** | Request/response logging | Compliance recording |
| **Caching strategy** | Custom cache behavior | Pre-warm cache for common phrases |
| **Response transformation** | Output modification | Custom JSON schema, XML wrapper |

---

## 11. Plugin Distribution & Registry

### 11.1 AGOS Plugin Registry

An optional centralized plugin registry MAY be maintained at `https://registry.agos.org`:

```
https://registry.agos.org/
├── /v1/
│   ├── /plugins                  # List all plugins
│   ├── /plugins/{id}            # Plugin details
│   ├── /plugins/{id}/download   # Plugin binary download
│   ├── /search?q={query}        # Search plugins
│   ├── /schools                 # List available grammar schools
│   └── /verify                  # Verify plugin checksum
```

### 11.2 Plugin Installation

```bash
# Install from registry
agos plugin install basra-rules-v2
# Output: Installed basra-rules-v2 v2.3.1 (1250 rules)

# Install from local file
agos plugin install ./my-plugin.agos-plugin.yaml

# List installed plugins
agos plugin list
# Output:
# basra-rules-v2      v2.3.1  ACTIVE   rule_set   1250 rules
# explanation-urdu    v1.0.0  ACTIVE   explanation  ur, text+html
# dictionary-hans-wehr v3.2.1 ACTIVE   kb_resolver 150K entries

# Update a plugin
agos plugin update basra-rules-v2

# Remove a plugin
agos plugin remove basra-rules-v2
```

### 11.3 Plugin Publishing

Plugin authors can publish to the registry:

```bash
# Package plugin for distribution
agos plugin package ./basra-rules/
# Output: basra-rules-v2.3.1.agos-plugin

# Validate plugin package
agos plugin validate basra-rules-v2.3.1.agos-plugin
# Output: Valid: yes (1250 rules, no errors)

# Publish to registry
agos plugin publish basra-rules-v2.3.1.agos-plugin --registry=https://registry.agos.org
# Output: Published basra-rules-v2 v2.3.1
```

### 11.4 Plugin Directory Structure

```
my-plugin/
├── manifest.yaml              # Plugin manifest
├── rules/                     # DSL rule files (for rule_set plugins)
│   ├── 01-verbal-sentences.agosrule
│   ├── 02-nominal-sentences.agosrule
│   ├── 03-idafa.agosrule
│   ├── 04-agreement.agosrule
│   └── 05-exceptions.agosrule
├── src/                       # Source code (for compiled plugins)
│   ├── lib.rs                 # Rust source
│   └── Cargo.toml
├── tests/                     # Test cases
│   ├── test_sentences.txt
│   └── expected_outputs.json
├── dist/                      # Compiled output
│   ├── plugin.wasm
│   └── plugin.so
├── README.md
└── LICENSE
```

---

## 12. Plugin Testing & Validation

### 12.1 Plugin Test Harness

Every plugin SHOULD be tested using the AGOS Plugin Test Harness:

```
trait PluginTestHarness {
    /// Load the plugin in isolation
    fn load_plugin(manifest_path: string) -> PluginInstance;

    /// Run a test case through the plugin
    fn run_test(test_case: PluginTestCase) -> TestResult;

    /// Run the plugin against the AGOS conformance suite
    fn run_conformance(plugin: PluginInstance) -> ConformanceReport;
}

type PluginTestCase = {
    name: string,
    description: string,
    input: any,                  // Plugin-specific input type
    expected_output: any,        // Expected output type
    expected_flags: GrammaticalFlag[],  // Expected flags (rule_set only)
}

type TestResult = {
    passed: boolean,
    actual_output: any,
    errors: string[],
    execution_time_ms: float,
}
```

### 12.2 Conformance Tests

All `rule_set` plugins MUST pass the AGOS Grammar Conformance Suite:

| Test Suite | Description | Test Count |
|------------|-------------|------------|
| **Basic verbal sentences** | Verb + subject + object | 50 |
| **Basic nominal sentences** | Topic + comment | 50 |
| **Idafa constructions** | Construct state | 30 |
| **Adjective agreement** | Wasf (na'at) | 30 |
| **Prepositional phrases** | Harf jarr + majrur | 20 |
| **Kana and sisters** | Kana & similar verbs | 20 |
| **Inna and sisters** | Inna & similar particles | 20 |
| **Conditional sentences** | In, law, lawla | 15 |
| **Exceptional constructions** | Istithna | 10 |
| **Quranic examples** | Classical Quranic grammar | 25 |
| **Poetic examples** | Classical poetry | 15 |
| **Ambiguous cases** | Multiple valid analyses | 20 |

Total: ~305 conformance tests per school.

### 12.3 Plugin Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Conformance pass rate** | > 99% | Tests passed / total tests |
| **Rule coverage** | > 90% | % of conformance cases where at least one rule fired |
| **False positive rate** | < 1% | Rules firing on non-matching sentences |
| **Average execution time** | < 100 μs per rule | Per-rule timing |
| **Memory usage** | < 10 MB per sentence | Peak memory during analysis |
| **Binary size** | < 5 MB (WASM) | Plugin binary size |

---

## 13. Cross-References

### 13.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | Plugin injection points at each pipeline stage |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | Detailed stage algorithms that plugins can extend |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | PluginLoader (MOD-12) interface definition |
| SPEC-0001-C6 | Deployment & Runtime Considerations | Sandboxing at deployment level |
| SPEC-0601 | Plugin System | Detailed plugin specification (future) |
| RFC-0001 | Grammar DSL | Full DSL specification for rule_set plugins |
| ADR-0005 | Why Plugin Architecture | Rationale for the plugin architecture decision |

### 13.2 External References

| Reference | Relevance |
|-----------|-----------|
| WebAssembly Specification | Plugin binary format |
| WASI (WebAssembly System Interface) | Optional OS access for plugins |
| JSON Schema | Plugin manifest validation |
| SemVer 2.0.0 | Plugin versioning scheme |
| Rust `wasm-pack` | Plugin SDK build tooling reference |
| Lua Sandboxing | Inspiration for plugin security model |

---

## Progress Summary

**SPEC-0001: Platform Architecture**

| Chapter | Title | Status |
|---------|-------|--------|
| Chapter 1 | Introduction and Scope | ✓ COMPLETE |
| Chapter 2 | System Architecture Overview | ✓ COMPLETE |
| Chapter 3 | Compilation Pipeline — Stage-by-Stage | ✓ COMPLETE |
| Chapter 4 | Module Responsibilities & Interfaces | ✓ COMPLETE |
| Chapter 5 | Data Flow & Intermediate Representations | ✓ COMPLETE |
| Chapter 6 | Deployment & Runtime Considerations | ✓ COMPLETE |
| **Chapter 7** | **Extensibility & Plugin Architecture** | **✓ COMPLETE (this document)** |
| Chapter 8 | Security, Validation & Error Handling | Pending |
| Chapter 9 | Performance Targets & Constraints | Pending |

**Dependencies:** Chapters 1–6, SPEC-0601, RFC-0001, ADR-0005.

**Recommended Next Chapter:** Chapter 8 — Security, Validation & Error Handling, which will define the security model, input validation framework, error classification, and the complete error handling strategy across all pipeline stages.
