---
spec_id: SPEC-0001
chapter: 2
title: System Architecture Overview
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - ADR-0001: Compiler Architecture Rationale
  - ADR-0002: Why Grammar Bytecode (planned)
  - ADR-0003: Why Grammar IR (planned)
  - RFC-0001: Grammar DSL (proposed)
  - RFC-0002: Grammar Bytecode (proposed)
  - RFC-0003: Grammar Virtual Machine (proposed)
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0301: Grammar Runtime
  - SPEC-0401: Knowledge Graph Engine
  - SPEC-0501: Explanation Engine
---

# Chapter 2: System Architecture Overview

## Table of Contents

1. [Architectural Layers](#1-architectural-layers)
2. [System Decomposition](#2-system-decomposition)
3. [Pipeline Architecture](#3-pipeline-architecture)
4. [Module Catalog](#4-module-catalog)
5. [Layer Responsibilities](#5-layer-responsibilities)
6. [Module Interaction Patterns](#6-module-interaction-patterns)
7. [Dependency Graph](#7-dependency-graph)
8. [Configuration & Lifecycle](#8-configuration--lifecycle)
9. [Scope Boundaries](#9-scope-boundaries)
10. [Cross-References](#10-cross-references)

---

## 1. Architectural Layers

AGOS is organized into **four architectural layers**. Each layer has a distinct responsibility, lifecycle, and dependency direction. Dependencies flow downward: higher layers depend on lower layers. No layer may depend on a layer above it.

```
┌─────────────────────────────────────────────────────────────┐
│                     APPLICATION LAYER                       │
│  I'rab Analyzer │ Nahwu Tutor │ Quran Explorer │ API ...   │
│  Built on top of AGOS. Not part of the core platform.      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     RUNTIME LAYER                           │
│  Grammar Virtual Machine (GVM)                              │
│  Explanation Engine                                         │
│  Plugin Loader                                              │
│  Cache Manager                                              │
│  API Gateway                                                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    COMPILATION LAYER                        │
│  Unicode Validator                                          │
│  Lexer                                                      │
│  Tokenizer                                                  │
│  Morphological Parser                                       │
│  Syntax Parser                                              │
│  GIR Constructor                                            │
│  Rule Engine                                                │
│  Knowledge Graph Resolver                                   │
│  Bytecode Generator                                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    KNOWLEDGE LAYER                          │
│  Grammar Rule Sets (versioned, school-specific)             │
│  Knowledge Bases (roots, wazan, verb forms, etc.)          │
│  Morphological Dictionaries                                 │
│  Corpus / Reference Data                                    │
│  Plugin Manifests                                           │
└─────────────────────────────────────────────────────────────┘
```

### 1.1 Layer Definitions

| Layer | Responsibility | Technology Independence |
|-------|---------------|------------------------|
| **Knowledge Layer** | Stores all versioned linguistic knowledge. Pure data — no executable logic. Rules are authored in Grammar DSL (RFC-0001) and stored as versioned files or datasets. | Language-agnostic. May be files, databases, or content-addressable storage. |
| **Compilation Layer** | Transforms Arabic text through successive stages into Grammar Bytecode. This is the core pipeline — the "compiler." Each stage is an independent module. | Best implemented in systems languages (Rust, C++, Go) for performance. Stages communicate through serialized GIR. |
| **Runtime Layer** | Executes Grammar Bytecode via the GVM, produces explanations, manages plugins, and provides the API surface for applications. | GVM can be implemented in any language. May run as a standalone process or embedded library. |
| **Application Layer** | Consumer applications built on top of the platform API. Not part of the AGOS core platform. | Any language/framework. Communicates with AGOS via the public API. |

### 1.2 Layer Rules

1. **Strict downward dependency.** A layer MAY reference types, interfaces, and services from any layer below it. A layer MUST NOT reference types, interfaces, or services from any layer above it.
2. **Compilation Layer MAY depend on Knowledge Layer.** The compilation pipeline reads rule sets and KB data to perform analysis. It does not modify the Knowledge Layer.
3. **Runtime Layer MUST NOT depend on Compilation Layer internals.** The Runtime Layer consumes the *output* of the Compilation Layer (Grammar Bytecode) but MUST NOT depend on any internal stage of the compilation pipeline. This enables the Compilation Layer to be replaced, optimized, or distributed independently.
4. **Knowledge Layer has zero dependencies.** It is pure data with no code dependencies.

---

## 2. System Decomposition

The AGOS platform decomposes into **14 core modules** across the Compilation and Runtime layers.

```
AGOS Platform
│
├── COMPILATION LAYER
│   ├── Module 01: UnicodeValidator
│   ├── Module 02: Lexer
│   ├── Module 03: Tokenizer
│   ├── Module 04: MorphologicalParser
│   ├── Module 05: SyntaxParser
│   ├── Module 06: GIRConstructor
│   ├── Module 07: RuleEngine
│   ├── Module 08: KnowledgeGraphResolver
│   └── Module 09: BytecodeGenerator
│
├── RUNTIME LAYER
│   ├── Module 10: GrammarVirtualMachine (GVM)
│   ├── Module 11: ExplanationEngine
│   ├── Module 12: PluginLoader
│   ├── Module 13: CacheManager
│   └── Module 14: APIGateway
│
└── KNOWLEDGE LAYER (data, not code)
    ├── KB-0001: Roots
    ├── KB-0002: Wazan
    ├── KB-0003: Verb Forms
    ├── KB-0004: Noun Patterns
    ├── KB-0005: Particles
    ├── KB-0006: Pronouns
    ├── KB-0007: Morphological Features
    ├── School-Specific Rule Sets
    └── Corpus / Reference Data
```

### 2.1 Module Naming Convention

Each module SHALL be identified by:
- **Module ID:** `AGOS-MOD-NN` where NN is a two-digit number (01–99).
- **Canonical name:** CamelCase, e.g., `MorphologicalParser`.
- **Short name:** lowercase_with_underscores, e.g., `morphological_parser`.

These identifiers are used in all specification documents, API contracts, configuration files, and source code.

---

## 3. Pipeline Architecture

### 3.1 Standard Pipeline

The standard compilation pipeline executes all Compilation Layer stages in sequence:

```
Input: Arabic Text (UTF-8 string)
  │
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-01 │ UnicodeValidator          │
  │  └─────────────────────────────────────────┘
  │  Output: NormalizedText
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-02 │ Lexer                     │
  │  └─────────────────────────────────────────┘
  │  Output: TokenStream
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-03 │ Tokenizer                 │
  │  └─────────────────────────────────────────┘
  │  Output: SegmentedTokenStream
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-04 │ MorphologicalParser       │
  │  └─────────────────────────────────────────┘
  │  Output: MorphologicalAnalysis
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-05 │ SyntaxParser              │
  │  └─────────────────────────────────────────┘
  │  Output: SyntaxTree
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-06 │ GIRConstructor            │
  │  └─────────────────────────────────────────┘
  │  Output: GrammarIR (GIR)
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-07 │ RuleEngine                │
  │  └─────────────────────────────────────────┘
  │  Output: AnnotatedGIR + EvidenceTrail
  ▼
  │  ┌─────────────────────────────────────────┐
  ├──│ AGOS-MOD-08 │ KnowledgeGraphResolver    │
  │  └─────────────────────────────────────────┘
  │  Output: ResolvedGIR + EvidenceTrail
  ▼
  │  ┌─────────────────────────────────────────┐
  └──│ AGOS-MOD-09 │ BytecodeGenerator         │
     └─────────────────────────────────────────┘
  Output: GrammarBytecode

Output: GrammarBytecode → Runtime Layer
```

### 3.2 Mini Pipelines (Short Circuits)

The standard pipeline may be shortened for specific use cases. Mini pipelines bypass unnecessary stages:

```
Morphology-Only Pipeline:
  Arabic Text → Validator → Lexer → Tokenizer → MorphologicalParser
  Output: MorphologicalAnalysis (no syntax, no rules, no bytecode)

Tokenization-Only Pipeline:
  Arabic Text → Validator → Lexer → Tokenizer
  Output: SegmentedTokenStream

Syntax-Only Pipeline (pre-tokenized input):
  SegmentedTokenStream → MorphologicalParser → SyntaxParser
  Output: SyntaxTree
```

Mini pipelines MUST produce output that conforms to the same GIR schema as the equivalent segment of the full pipeline. This ensures that any consumer expecting full pipeline output can also consume mini pipeline output.

### 3.3 Pipeline Execution Model

| Property | Specification |
|----------|--------------|
| **Execution order** | Strictly sequential within a single pipeline. Stage N+1 receives the output of Stage N. |
| **Stage coupling** | Loose. Stages communicate only through serialized IR. No shared state. No stage calls another stage's internal methods. |
| **Concurrency model** | The pipeline MAY process multiple input texts concurrently. Within a single input's processing, stages MUST execute sequentially. |
| **Error propagation** | A stage MUST either succeed (producing valid output) or fail with a structured error. Downstream stages MUST NOT proceed if an upstream stage fails. |
| **Caching** | Any stage's output MAY be cached by the CacheManager. If cached output exists for a given input + stage + knowledge version, the stage MAY be skipped. |

---

## 4. Module Catalog

### 4.1 UnicodeValidator (AGOS-MOD-01)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | Raw UTF-8 string (Arabic text) |
| **Output** | Normalized Unicode string |
| **Deterministic** | Yes |
| **Knowledge dependencies** | Unicode character tables (built-in) |
| **Specification** | SPEC-0001 (this document) — detailed in Chapter 3 |

**Responsibility:** Validate and normalize the input text. Handle:
- Unicode normalization (NFKC for Arabic).
- Character range validation (Arabic block U+0600–U+06FF, U+0750–U+077F, etc.).
- Detection and rejection of non-Arabic characters (based on configuration).
- Diacritic (tashkeel) normalization: optional stripping or canonicalization.
- Tatweel (kashida) handling: stripping or normalization.
- BOM (Byte Order Mark) handling.

### 4.2 Lexer (AGOS-MOD-02)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | Normalized Unicode string |
| **Output** | `TokenStream` — ordered list of `RawToken` objects |
| **Deterministic** | Yes |
| **Knowledge dependencies** | Character class tables (letter, digit, punctuation, whitespace) |
| **Specification** | SPEC-0001 (this document) — detailed in Chapter 3 |

**Responsibility:** Perform lexical analysis of the normalized text. Produce a stream of raw tokens:
- Identify word boundaries (whitespace-separated tokens).
- Identify punctuation, digits, and special characters.
- Preserve original positions (start offset, end offset) for each token.
- Handle special cases: Quranic annotation symbols, poetic meter markers, etc.

### 4.3 Tokenizer (AGOS-MOD-03)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `TokenStream` |
| **Output** | `SegmentedTokenStream` — tokens with morpheme boundaries |
| **Deterministic** | Yes |
| **Knowledge dependencies** | Prefix/suffix tables, clitic tables, morphological boundary rules |
| **Specification** | SPEC-0001 (this document) — detailed in Chapter 3 |

**Responsibility:** Segment each raw token into its constituent morphemes:
- Identify and separate proclitics (conjunctions: وَ, فَ; prepositions: بِ, لِ, كَ; future marker: سَ).
- Identify and separate enclitics (object pronouns: هُ, هَا, هُمْ, etc.).
- Identify the core stem (which will be analyzed morphologically).
- Handle ambiguous segmentations (where multiple segmentations are possible, produce all alternatives as an ambiguity set).

### 4.4 MorphologicalParser (AGOS-MOD-04)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `SegmentedTokenStream` |
| **Output** | `MorphologicalAnalysis` — per-stem: root, wazan, POS, features |
| **Deterministic** | Yes |
| **Knowledge dependencies** | KB-0001 (Roots), KB-0002 (Wazan), KB-0003 (Verb Forms), KB-0004 (Noun Patterns), KB-0005 (Particles), KB-0006 (Pronouns), KB-0007 (Morphological Features) |
| **Specification** | SPEC-0101 (Morphology Engine) |

**Responsibility:** Perform morphological analysis on each stem:
- Identify the root (jadhr) by matching against the root KB.
- Identify the morphological pattern (wazan) by matching against the pattern KB.
- Determine part of speech (noun, verb, particle).
- Extract morphological features: gender, number, person, tense, mood, voice, case, state (definiteness).
- Handle verb form identification (Form I, Form II, ..., Form XV).
- Handle broken plurals and other irregular forms.
- When multiple analyses are possible, produce all alternatives as ambiguity sets.

### 4.5 SyntaxParser (AGOS-MOD-05)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `MorphologicalAnalysis` |
| **Output** | `SyntaxTree` — constituent structure with grammatical roles |
| **Deterministic** | Yes |
| **Knowledge dependencies** | Morphological features from MOD-04, sentence structure rules |
| **Specification** | SPEC-0101 (Morphology Engine) — Chapter on Syntax |

**Responsibility:** Parse the syntactic structure of the sentence:
- Identify sentence type: nominal (jumlah ismiyyah) or verbal (jumlah fi'liyyah).
- Identify sentence constituents: mubtada' (topic), khabar (comment), fi'l (verb), fa'il (subject), maf'ul (object), etc.
- Build a constituency tree or dependency structure that represents the grammatical relationships between tokens.
- Handle multi-word constructions: idafa (construct state), wasf (adjective agreement), etc.
- Handle ambiguity: multiple possible syntactic structures are represented as an ambiguity forest.

### 4.6 GIRConstructor (AGOS-MOD-06)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `SyntaxTree` + `MorphologicalAnalysis` |
| **Output** | `GrammarIR` (GIR) — unified grammatical intermediate representation |
| **Deterministic** | Yes |
| **Knowledge dependencies** | None (purely structural transformation) |
| **Specification** | ADR-0003 (Why Grammar IR) and RFC-0003 (Grammar Virtual Machine) |

**Responsibility:** Combine morphology and syntax into a single unified intermediate representation:
- Transform the syntax tree and morphological annotations into the GIR format.
- The GIR captures the complete grammatical state at this point in the pipeline: every token's morphology, every syntactic relationship, and every ambiguity set.
- The GIR is a data structure (not yet bytecode) that can be serialized, inspected, cached, and passed to the Rule Engine.
- GIR format is versioned. All stages in the pipeline must agree on the GIR version.

### 4.7 RuleEngine (AGOS-MOD-07)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `GrammarIR` |
| **Output** | `AnnotatedGIR` + `EvidenceTrail` |
| **Deterministic** | Yes |
| **Knowledge dependencies** | School-specific Grammar DSL rule sets (RFC-0001) |
| **Specification** | SPEC-0201 (Rule Engine) |

**Responsibility:** Apply grammatical rules to the GIR:
- Load the rule set for the configured grammar school (Basra, Kufa, etc.).
- Apply each rule to the GIR in the defined order.
- Rules may:
  - Confirm or reject ambiguous analyses.
  - Assign or modify grammatical features (case, mood, etc.).
  - Detect and flag grammatical violations.
  - Resolve anaphora and other cross-sentence references.
- Record every rule application in the Evidence Trail: which rule fired, with what inputs, producing what conclusion.
- When multiple rules conflict, apply conflict resolution strategy: rule priority ordering, school-specific tiebreakers, or preservation of ambiguity for downstream handling.

### 4.8 KnowledgeGraphResolver (AGOS-MOD-08)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `AnnotatedGIR` + `EvidenceTrail` |
| **Output** | `ResolvedGIR` + `ResolvedEvidenceTrail` |
| **Deterministic** | Yes |
| **Knowledge dependencies** | KB-0001 through KB-0007, knowledge graph indices |
| **Specification** | SPEC-0401 (Knowledge Graph Engine) |

**Responsibility:** Resolve all knowledge graph references in the GIR:
- Link each root reference to its full KB entry (definitions, cognates, semantic fields).
- Link each wazan reference to its morphological paradigm.
- Link each word form to its dictionary entry (if available).
- Resolve cross-references: synonyms, antonyms, related roots.
- Enrich the GIR with resolved metadata for use by the Explanation Engine.
- The resolved GIR contains both the structural grammatical analysis and the linked linguistic knowledge.

### 4.9 BytecodeGenerator (AGOS-MOD-09)

| Field | Value |
|-------|-------|
| **Layer** | Compilation |
| **Input** | `ResolvedGIR` + `ResolvedEvidenceTrail` |
| **Output** | `GrammarBytecode` (serialized binary) |
| **Deterministic** | Yes |
| **Knowledge dependencies** | GVM instruction set specification (RFC-0002) |
| **Specification** | RFC-0002 (Grammar Bytecode) |

**Responsibility:** Compile the resolved GIR into executable bytecode:
- Translate the GIR structure into GVM instructions.
- Emit bytecode that, when executed by the GVM, reproduces the grammatical analysis and evidence trail.
- Optimize the bytecode for execution: eliminate redundant instructions, compact repeated patterns, encode strings efficiently.
- The bytecode is self-contained: it includes the input text, the analysis, and the evidence trail. No external references are needed for execution.
- Bytecode format is versioned. The GVM version must match or exceed the bytecode version.

### 4.10 GrammarVirtualMachine (AGOS-MOD-10)

| Field | Value |
|-------|-------|
| **Layer** | Runtime |
| **Input** | `GrammarBytecode` |
| **Output** | `AnalysisResult` — executed analysis with full state |
| **Deterministic** | Yes (given same bytecode) |
| **Knowledge dependencies** | None (all knowledge is encoded in the bytecode) |
| **Specification** | RFC-0003 (Grammar Virtual Machine) |

**Responsibility:** Execute Grammar Bytecode to produce the final grammatical analysis:
- Interpret or JIT-compile the bytecode instructions.
- Produce a structured AnalysisResult: the complete grammatical analysis with all features, relationships, and evidence.
- The AnalysisResult is the canonical output of the AGOS pipeline — all downstream consumers (Explanation Engine, applications, APIs) work from this result.
- The GVM provides execution guarantees: bounded execution time, memory safety, deterministic output.

### 4.11 ExplanationEngine (AGOS-MOD-11)

| Field | Value |
|-------|-------|
| **Layer** | Runtime |
| **Input** | `AnalysisResult` |
| **Output** | `HumanReadableExplanation` (localized text, multiple formats) |
| **Deterministic** | Yes (template-based) / Optional LLM enhancement |
| **Knowledge dependencies** | Explanation templates (built-in or plugin), optional LLM service |
| **Specification** | SPEC-0501 (Explanation Engine) |

**Responsibility:** Transform the AnalysisResult into human-readable explanations:
- Generate I'rab-style breakdowns (word-by-word grammatical analysis).
- Generate natural language explanations of grammatical constructions.
- Support multiple languages for explanations (Arabic, English, Urdu, Malay, etc.).
- Support multiple output formats: plain text, HTML, JSON, PDF.
- Optionally, interface with an LLM for enhanced, conversational explanations. When LLM enhancement is used, the LLM receives the AnalysisResult (not the raw text) and produces explanations based on the deterministic analysis. The LLM NEVER modifies or overrides the AnalysisResult.

### 4.12 PluginLoader (AGOS-MOD-12)

| Field | Value |
|-------|-------|
| **Layer** | Runtime |
| **Input** | Plugin manifest (YAML/JSON) |
| **Output** | Loaded plugin instances |
| **Deterministic** | Yes |
| **Knowledge dependencies** | Plugin API specification |
| **Specification** | SPEC-0601 (Plugin System) |

**Responsibility:** Discover, validate, and load plugins:
- Scan plugin directories for plugin manifests.
- Validate plugin manifests against the plugin schema.
- Load plugins into the appropriate pipeline stages.
- Manage plugin lifecycle: load, unload, reload.
- Verify plugin compatibility with the current platform version.
- Isolate plugins for security: plugins run in a sandboxed environment with restricted system access.

### 4.13 CacheManager (AGOS-MOD-13)

| Field | Value |
|-------|-------|
| **Layer** | Runtime |
| **Input** | Cache key (input hash + pipeline configuration + knowledge versions) |
| **Output** | Cached stage output (if hit) or cache miss signal |
| **Deterministic** | Yes |
| **Knowledge dependencies** | None (generic caching infrastructure) |
| **Specification** | SPEC-0001 (this document) — Section 6 |

**Responsibility:** Manage caching of pipeline stage outputs:
- Cache the output of any pipeline stage based on a cache key.
- Cache key includes: input hash, pipeline configuration hash, knowledge version hashes.
- Support configurable cache backends: in-memory (default), Redis, filesystem, database.
- Support configurable TTL and eviction policies: LRU, LFU, time-based.
- Automatic cache invalidation when knowledge versions change.
- Cache statistics: hit rate, miss rate, storage usage.

### 4.14 APIGateway (AGOS-MOD-14)

| Field | Value |
|-------|-------|
| **Layer** | Runtime |
| **Input** | API request (REST, gRPC, or GraphQL) |
| **Output** | API response |
| **Deterministic** | Yes |
| **Knowledge dependencies** | None (generic API infrastructure) |
| **Specification** | SPEC-0001 (this document) — Section 6 |

**Responsibility:** Provide the external API surface for AGOS:
- Expose REST/gRPC/GraphQL APIs for text analysis, batch processing, knowledge base queries.
- Authentication and authorization.
- Rate limiting and quota management.
- Request validation and sanitization.
- Response formatting and serialization (JSON, XML, proto).
- API versioning.
- Documentation generation (OpenAPI/Swagger for REST, proto docs for gRPC).

---

## 5. Layer Responsibilities

### 5.1 Knowledge Layer

**Governance:** The Knowledge Layer is governed by the AGOS Knowledge Committee (linguists, domain experts). Changes to knowledge bases follow a review process analogous to RFCs: Draft → Review → Accepted → Published.

**Versioning:** Every knowledge base has a semantic version (MAJOR.MINOR.PATCH). All pipeline stages that depend on knowledge bases record the versions they used, ensuring reproducibility.

**Storage:** Knowledge bases MAY be stored as:
- Version-controlled files (Git) for development and review.
- Compiled binary formats for production deployment.
- Content-addressable storage for distribution and caching.

### 5.2 Compilation Layer

**Governance:** The Compilation Layer is governed by the AGOS Engineering Committee. Changes to pipeline stages follow standard software engineering practices: design review, implementation, code review, testing, release.

**Performance requirements:** The Compilation Layer MUST process text at a rate sufficient for interactive use. Target: < 100ms per sentence for the full pipeline. Morphology-only pipeline: < 10ms per token.

**Language recommendations:** Rust (preferred) or C++ for performance-critical stages. Go for orchestration and pipeline management.

### 5.3 Runtime Layer

**Governance:** Same as Compilation Layer. The Runtime Layer is the interface between the compilation pipeline and the outside world.

**Performance requirements:** The GVM MUST execute bytecode at a rate sufficient for interactive use. Target: < 50ms per sentence for full execution. Cache hit: < 1ms.

**Portability:** The GVM MUST support at least: Linux (x86_64, aarch64), macOS (x86_64, arm64), Windows (x86_64). WASM target is a future extension.

### 5.4 Application Layer

**Governance:** Not governed by AGOS core. Applications are built by third parties or by the AGOS team as separate projects. The Application Layer communicates with AGOS through the Runtime Layer's public API only.

---

## 6. Module Interaction Patterns

### 6.1 Synchronous Pipeline (Default)

```
Application
    │
    ▼
APIGateway ──► CacheManager (check cache)
    │
    ▼
UnicodeValidator ──► Lexer ──► Tokenizer ──► ... ──► BytecodeGenerator
    │
    ▼
GVM ──► ExplanationEngine
    │
    ▼
CacheManager (store result)
    │
    ▼
APIGateway ──► Application
```

Most analysis requests follow this pattern: synchronous, request-response.

### 6.2 Batch Pipeline

```
Application
    │
    ▼
APIGateway ──► Batch Queue
                  │
                  ▼
            Batch Processor
            (pipeline × N texts)
                  │
                  ▼
            Result Store
                  │
                  ▼
Application ◄── Callback / Polling
```

For batch processing of large corpora (Quran, Hadith collections, etc.), the pipeline runs asynchronously. Each text is processed independently, and results are collected for bulk retrieval.

### 6.3 Interactive Pipeline (with ambiguity resolution)

```
Application (user interface)
    │
    ▼
APIGateway ──► Pipeline (up to GIR)
    │
    ▼
Multiple GIRs (ambiguity forest)
    │
    ▼
Rule Engine (apply rules, reduce ambiguity)
    │
    ▼
If ambiguity remains → Application prompts user
    │
    ▼
User selects preferred analysis
    │
    ▼
Pipeline continues → BytecodeGenerator → GVM → Explanation
```

For educational applications, the user may be prompted to resolve ambiguity interactively. This teaches grammatical reasoning while improving analysis accuracy.

### 6.4 Plugin Injection Points

Plugins can be injected at the following points in the pipeline:

```
Pipeline Stage        │ Plugin Type              │ Example
──────────────────────┼──────────────────────────┼─────────────────────────
UnicodeValidator      │ Custom normalizer        │ Quranic symbol handler
Lexer                 │ Custom token classifier  │ Poetic meter tokenizer
Tokenizer             │ Custom segmenter         │ Special clitic rules for a dialect
MorphologicalParser   │ Custom morphology engine │ Alternative wazan matching
SyntaxParser          │ Custom syntax engine     │ Dependency grammar parser
RuleEngine            │ School rule set          │ Basra, Kufa, Andalus
KnowledgeGraphResolver│ Custom KB resolver       │ Additional dictionary source
ExplanationEngine     │ Custom explanation       │ Gamified explanation for children
APIGateway            │ Custom API middleware    │ Authentication provider
```

---

## 7. Dependency Graph

### 7.1 Module Dependency Table

```
Module                    │ Depends On              │ Used By
──────────────────────────┼─────────────────────────┼─────────────────────────
UnicodeValidator          │ (none)                  │ Lexer
Lexer                     │ UnicodeValidator        │ Tokenizer
Tokenizer                 │ Lexer                   │ MorphologicalParser
MorphologicalParser       │ Tokenizer, KB-*        │ SyntaxParser, GIRConstructor
SyntaxParser              │ MorphologicalParser     │ GIRConstructor
GIRConstructor            │ MorphologicalParser,    │ RuleEngine
                          │ SyntaxParser            │
RuleEngine                │ GIRConstructor, DSL     │ KnowledgeGraphResolver
                          │ rule sets               │
KnowledgeGraphResolver    │ RuleEngine, KB-*        │ BytecodeGenerator
BytecodeGenerator         │ KnowledgeGraphResolver  │ GVM
GVM                       │ BytecodeGenerator       │ ExplanationEngine
ExplanationEngine         │ GVM                     │ APIGateway (indirect)
PluginLoader              │ (none)                  │ All stages (via injection)
CacheManager              │ (none)                  │ All stages
APIGateway                │ GVM, ExplanationEngine  │ Applications
```

### 7.2 Dependency Graph (ASCII)

```
KB-* ──────────────────────────────────────────────────────────┐
  │                                                             │
  ▼                                                             │
MOD-01 (UnicodeValidator)                                       │
  │                                                             │
  ▼                                                             │
MOD-02 (Lexer)                                                  │
  │                                                             │
  ▼                                                             │
MOD-03 (Tokenizer)                                              │
  │                                                             │
  ├─────────────────────────────────────────────────────────┐   │
  ▼                                                         │   │
MOD-04 (MorphologicalParser) ◄────────────── KB-*, DSL -----┼───┤
  │                                                         │   │
  ├─────────────────────────────────────┐                   │   │
  ▼                                     ▼                   │   │
MOD-05 (SyntaxParser)         MOD-06 (GIRConstructor)       │   │
  │                                     │                   │   │
  └─────────────────────────────────────┘                   │   │
                      │                                     │   │
                      ▼                                     │   │
              MOD-07 (RuleEngine) ◄────── DSL rule sets ----┼───┘
                      │                                     │
                      ▼                                     │
              MOD-08 (KnowledgeGraphResolver) ◄──────── KB-*│
                      │                                     │
                      ▼                                     │
              MOD-09 (BytecodeGenerator)                     │
                      │                                     │
                      ▼                                     │
              MOD-10 (GVM)                                   │
                      │                                     │
                      ▼                                     │
              MOD-11 (ExplanationEngine)                     │
                      │                                     │
                      ▼                                     │
              MOD-14 (APIGateway)                            │
                      │                                     │
                      ▼                                     │
              Applications                                   │
                                                            │
MOD-12 (PluginLoader) ──► Injects into MOD-01..11            │
MOD-13 (CacheManager) ──► Caches MOD-01..09 output            │
KB-* ────────────────────────────────────────────────────────┘
```

---

## 8. Configuration & Lifecycle

### 8.1 Pipeline Configuration

The AGOS platform MUST support the following configuration dimensions:

| Dimension | Values | Default |
|-----------|--------|---------|
| **Pipeline mode** | `full`, `morphology-only`, `tokenization-only`, `syntax-only` | `full` |
| **Grammar school** | `basra`, `kufa`, `baghdad`, `andalus`, `modern` | `basra` |
| **Language variant** | `classical`, `msa`, `quranic` | `classical` |
| **Explanation language** | `ar`, `en`, `ur`, `ms`, `id`, `fr`, `tr` | `en` |
| **Explanation format** | `text`, `html`, `json`, `pdf` | `json` |
| **Enable caching** | `true`, `false` | `true` |
| **Enable LLM enhancement** | `true`, `false` | `false` |
| **Plugin directory** | Path string | `./plugins` |
| **Knowledge base directory** | Path string | `./knowledge` |

### 8.2 Pipeline Lifecycle

```
1. LOAD
   ├── Load configuration
   ├── Load knowledge bases (version check)
   ├── Load rule sets (version check)
   ├── Discover and load plugins
   └── Initialize CacheManager

2. READY
   ├── Pipeline is ready to accept requests
   ├── All stages initialized
   ├── Warm cache (optional, pre-compute common phrases)
   └── Health check endpoint responds 200

3. PROCESSING
   ├── Accept request
   ├── Execute pipeline
   ├── Return result
   └── (Repeat)

4. RELOAD
   ├── Detect knowledge base version change
   ├── Re-initialize affected stages
   ├── Invalidate affected cache entries
   └── Return to READY

5. SHUTDOWN
   ├── Stop accepting new requests
   ├── Complete in-flight requests (graceful timeout)
   ├── Flush cache (if persistent)
   ├── Unload plugins
   └── Exit
```

### 8.3 Configuration Sources

Configuration is resolved in the following order (later sources override earlier ones):

1. **Built-in defaults** (compiled into each module).
2. **Configuration file** (`agos.yaml` or `agos.json` in the working directory or `AGOS_CONFIG_PATH`).
3. **Environment variables** (prefix `AGOS_`, e.g., `AGOS_SCHOOL=basra`).
4. **Command-line arguments** (for CLI usage).
5. **API request parameters** (for server usage, per-request overrides).

---

## 9. Scope Boundaries

### 9.1 AGOS Core (in scope for SPEC-0001)

```
AGOS Core
├── Compilation Pipeline (MOD-01 through MOD-09)
├── GVM (MOD-10)
├── Explanation Engine (MOD-11)
├── Plugin System (MOD-12)
├── Cache Manager (MOD-13)
├── API Gateway (MOD-14)
├── Knowledge Base format specifications
└── Grammar DSL specification
```

### 9.2 NOT in scope for AGOS Core (covered elsewhere)

```
Not Core (separate specifications or external)
├── SPEC-0101: Morphology Engine implementation details
├── SPEC-0201: Rule Engine implementation details
├── SPEC-0301: GVM implementation details
├── SPEC-0401: Knowledge Graph implementation details
├── SPEC-0501: Explanation Engine implementation details
├── SPEC-0601: Plugin System implementation details
├── KB-0001..0007: Knowledge base contents (linguistic data)
├── Application-specific concerns (UI, LMS, mobile, etc.)
└── LLM service (external dependency, no specification needed)
```

### 9.3 Boundary Rules

1. **No circular dependencies.** The module dependency graph MUST remain a directed acyclic graph (DAG).
2. **No module may access another module's internal state.** All communication is through defined IRs and APIs.
3. **No module may bypass the pipeline order.** A module cannot consume output from a non-adjacent upstream module without going through intermediate stages (exception: CacheManager may serve cached output from any stage).
4. **No application may import core module internals.** Applications consume the public API only.

---

## 10. Cross-References

### 10.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| ADR-0001 | Compiler Architecture Rationale | Justifies the pipeline decomposition |
| ADR-0002 | Why Grammar Bytecode | Justifies MOD-09 and MOD-10 |
| ADR-0003 | Why Grammar IR | Justifies MOD-06 |
| SPEC-0101 | Morphology Engine | Detailed specification for MOD-04, MOD-05 |
| SPEC-0201 | Rule Engine | Detailed specification for MOD-07 |
| SPEC-0301 | Grammar Runtime | Detailed specification for MOD-10, MOD-11 |
| SPEC-0401 | Knowledge Graph Engine | Detailed specification for MOD-08 |
| SPEC-0501 | Explanation Engine | Detailed specification for MOD-11 |
| SPEC-0601 | Plugin System | Detailed specification for MOD-12 |
| RFC-0001 | Grammar DSL | Defines the rule authoring language |
| RFC-0002 | Grammar Bytecode | Defines MOD-09 output format |
| RFC-0003 | Grammar Virtual Machine | Defines MOD-10 execution model |

### 10.2 External References

| Reference | Relevance |
|-----------|-----------|
| LLVM Language Reference | Stage decomposition pattern |
| MLIR (Multi-Level Intermediate Representation) | Multi-stage IR design inspiration |
| WebAssembly Specification | Portable bytecode execution model |
| Unix Pipeline Philosophy | "Do one thing and do it well" — stage design |
| Clean Architecture (R. Martin) | Layer dependency rules |

---

## Progress Summary

**SPEC-0001: Platform Architecture**

| Chapter | Title | Status |
|---------|-------|--------|
| Chapter 1 | Introduction and Scope | ✓ COMPLETE |
| **Chapter 2** | **System Architecture Overview** | **✓ COMPLETE (this document)** |
| Chapter 3 | Compilation Pipeline — Stage-by-Stage | Pending |
| Chapter 4 | Module Responsibilities & Interfaces | Pending |
| Chapter 5 | Data Flow & Intermediate Representations | Pending |
| Chapter 6 | Deployment & Runtime Considerations | Pending |
| Chapter 7 | Extensibility & Plugin Architecture | Pending |
| Chapter 8 | Security, Validation & Error Handling | Pending |
| Chapter 9 | Performance Targets & Constraints | Pending |

**Dependencies:** Chapter 1 (Introduction and Scope), ADR-0001 (Compiler Architecture Rationale).

**Recommended Next Chapter:** Chapter 3 — Compilation Pipeline — Stage-by-Stage, which will provide detailed specifications for each of the 9 compilation stages (MOD-01 through MOD-09), including input/output schemas, algorithms, edge cases, and error handling.
