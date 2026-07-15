# AGOS Specification Suite — Master Index

| **Field** | **Value** |
|---|---|
| **Document** | Index & Navigation Guide |
| **Version** | 1.0.0 |
| **Status** | Published |
| **Last Updated** | 2026-07-15 |
| **Document Count** | 39 specification documents |
| **Total Line Count** | ~81,861 lines of specification |

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [How to Use This Index](#2-how-to-use-this-index)
3. [Document Types](#3-document-types)
4. [Recommended Reading Order](#4-recommended-reading-order)
5. [Quick Reference by Topic](#5-quick-reference-by-topic)
6. [RFCs — Requests for Comments](#6-rfcs--requests-for-comments)
7. [SPEC-0001 — Platform Architecture (9 Chapters)](#7-spec-0001--platform-architecture-9-chapters)
8. [Module Specifications](#8-module-specifications)
9. [ADRs — Architecture Decision Records](#9-adrs--architecture-decision-records)
10. [KBs — Knowledge Base Specifications](#10-kbs--knowledge-base-specifications)
11. [Pipeline-to-Spec Mapping](#11-pipeline-to-spec-mapping)
12. [Cross-Reference Directory](#12-cross-reference-directory)
13. [Specification Statistics](#13-specification-statistics)
14. [Glossary of ID Prefixes](#14-glossary-of-id-prefixes)

---

## 1. Introduction

This document is the **master index and navigation guide** for the AGOS (Arabic Grammar Operating System) specification suite — an engineering standards repository covering the architecture, design, implementation, and linguistic knowledge of a complete computational platform for Arabic grammar.

The specification suite is organized into five document categories, mirroring the structure of major engineering standards (IETF RFCs, W3C specifications, programming language specifications):

| Category | Prefix | Count | Purpose |
|----------|--------|-------|---------|
| **Request for Comments** | RFC- | 4 | Proposals and newly accepted standards |
| **Specification** | SPEC- | 20 | Normative implementation specifications (9 architecture chapters + 11 module specs) |
| **Architecture Decision Record** | ADR- | 5 | Rationale for major architectural decisions |
| **Knowledge Base** | KB- | 9 | Versioned linguistic datasets and taxonomies |
| **Deprecation Proposal** | DEP- | 0 | Documents for deprecated designs (reserved for future use) |
| **Master Index** | — | 1 | This document — navigation and cross-reference guide |
| **Total** | | **39** | |

The AGOS pipeline transforms Arabic text through a compiler-inspired multi-stage pipeline:

```
Input Text → Unicode Validation → Lexer → Tokenizer →
Morphological Parser → Syntax Parser → GIR Construction →
Rule Engine → Knowledge Graph Resolver → Bytecode Generator →
Grammar Virtual Machine → Explanation Engine → Applications
```

Each stage is documented in one or more specifications, linked through cross-references and shared intermediate representation (IR) schemas.

---

## 2. How to Use This Index

### For New Readers

Follow the recommended reading order in [Section 4](#4-recommended-reading-order). Start with the ADRs (why), then the RFCs (what), then the SPECs (how), and finally the KBs (data).

### For Implementers

Use these entry points based on what you are implementing:

| I'm implementing... | Start with |
|--------------------|------------|
| The compilation pipeline | SPEC-0001-C3 (§7) |
| Morphological analysis | SPEC-0101 (§8), KB-0001/2/3/4/5/6 |
| Syntactic parsing | SPEC-0101 (§8), SPEC-0201 |
| The rule engine | SPEC-0201 (§8), RFC-0001, RFC-0004 |
| The Grammar Virtual Machine | SPEC-0301, RFC-0003, SPEC-0302, SPEC-0303, SPEC-0304 |
| Knowledge graph resolution | SPEC-0401 (§8) |
| Explanation generation | SPEC-0501 (§8) |
| The plugin system | SPEC-0601 (§8), SPEC-0001-C7 |
| Feature bitfield encoding | SPEC-0102 (§8), KB-0007 |
| Performance optimization | SPEC-0103, SPEC-0001-C9 |
| A GVM in a new language | SPEC-0303 (porting guide), SPEC-0302 (instruction set), SPEC-0304 (memory) |
| A new grammar school | ADR-0005, RFC-0001, RFC-0004, SPEC-0601 |
| A new explanation language | SPEC-0501, SPEC-0601 |

### For Linguists and Data Maintainers

| I need to... | Consult |
|--------------|---------|
| Understand the KB suite architecture | KB-OVERVIEW (§10) |
| Add/edit roots | KB-0001 (§10) |
| Add/edit morphological patterns | KB-0002 (§10) |
| Add/edit verb conjugations | KB-0003 (§10) |
| Add/edit noun patterns | KB-0004 (§10) |
| Add/edit particles | KB-0005, KB-0008 (§10) |
| Add/edit pronouns | KB-0006 (§10) |
| Understand the feature taxonomy | KB-0007, SPEC-0102 (§10) |
| Understand particle lookup internals | KB-0008 (§10) |

---

## 3. Document Types

### 3.1 RFC — Request for Comments

RFCs define new proposals before formal acceptance. They follow an RFC lifecycle:

```
Draft → Review → Accepted → Implemented → Converted to SPEC
```

RFCs may change significantly during review. Once accepted and stable, they may be converted to SPEC documents.

**Current RFC status:** All 4 RFCs are in Draft status, accepted as foundational designs.

### 3.2 SPEC — Official Specification

SPEC documents are **normative** — all implementations MUST follow them. Every SPEC describes a component, interface, or algorithm of the AGOS platform with sufficient detail to be implemented independently.

SPECs are subdivided by module numbering:
- **SPEC-0001:** Platform architecture (cross-cutting, 9 chapters)
- **SPEC-01xx:** Morphology engine family
- **SPEC-02xx:** Rule engine
- **SPEC-03xx:** Grammar Runtime / GVM family
- **SPEC-04xx:** Knowledge graph engine
- **SPEC-05xx:** Explanation engine
- **SPEC-06xx:** Plugin system

### 3.3 ADR — Architecture Decision Record

ADRs document the **rationale** behind major architectural decisions. Each ADR records:

- **Context:** The problem or question that prompted the decision
- **Alternatives Considered:** Other approaches that were evaluated and rejected
- **Decision:** The chosen approach
- **Consequences:** Positive and negative effects of the decision

ADRs are **Accepted** (binding) or **Proposed** (under consideration). All 5 current ADRs are Accepted.

### 3.4 KB — Knowledge Base Specification

KBs define **versioned linguistic data** — the empirical knowledge that AGOS uses to analyze Arabic text. KBs are treated as versioned datasets rather than platform code. Each KB:

- Has independent semantic versioning (MAJOR.MINOR.PATCH)
- Has a declared dependency matrix specifying compatibility with other KBs
- Is compiled from human-readable YAML source into a compact binary format
- Is loaded (typically memory-mapped) at runtime by the relevant pipeline module

### 3.5 DEP — Deprecation Proposal

DEPs document **obsolete designs** that are being phased out. No DEPs currently exist (the platform has not yet shipped a version with legacy features to deprecate). Reserved for future use.

---

## 4. Recommended Reading Order

For newcomers to the AGOS specification suite, the following order builds understanding from first principles to implementation detail:

### Phase 1: Architectural Foundation (Start Here)

```
 1. ADR-0001  Compiler Architecture Rationale
 2. ADR-0002  Grammar Bytecode Rationale
 3. ADR-0003  Grammar IR Rationale
 4. ADR-0004  Offline-First Architecture
 5. ADR-0005  Plugin Architecture
```

**Why start here:** The five ADRs establish the "why" behind every design decision in the platform. Understanding these decisions first makes the rest of the spec suite self-explanatory.

### Phase 2: Platform Architecture (The Big Picture)

```
 6. SPEC-0001-C1  Introduction and Scope
 7. SPEC-0001-C2  System Architecture Overview
 8. SPEC-0001-C3  Compilation Pipeline — Stage-by-Stage
 9. SPEC-0001-C4  Module Responsibilities & Interfaces
10. SPEC-0001-C5  Data Flow & Intermediate Representations
```

**What you'll learn:** The complete pipeline architecture, all 11 pipeline stages, the 11 intermediate representations (IR-1 through IR-11), and the module interface contracts.

### Phase 3: Linguistic Knowledge (The Data)

```
11. KB-OVERVIEW    KB Suite Overview & Architecture
12. KB-0007        Morphological Features Taxonomy
13. KB-0001        Roots Database
14. KB-0002        Wazan Database
15. KB-0003        Verb Forms
16. KB-0004        Noun Patterns
17. KB-0005        Particles
18. KB-0006        Pronouns
```

**What you'll learn:** The complete Arabic morphological model — all linguistic data that the pipeline uses for analysis. KB-0007 (features) is first because it defines the shared vocabulary used by all other KBs.

### Phase 4: Core Implementation (The How)

```
19. RFC-0001       Grammar DSL
20. SPEC-0101      Morphology Engine
21. SPEC-0102      Morphological Features Encoding
22. SPEC-0201      Rule Engine
23. SPEC-0401      Knowledge Graph Engine
24. RFC-0002       Grammar Bytecode Format
25. RFC-0003       Grammar Virtual Machine
26. SPEC-0301      Grammar Runtime
27. SPEC-0302      GVM Instruction Set
28. SPEC-0303      GVM Implementation Guide
29. SPEC-0304      GVM Memory Model
30. SPEC-0501      Explanation Engine
```

**What you'll learn:** How each pipeline stage works internally — algorithms, data structures, interfaces, and error handling.

### Phase 5: Advanced Topics

```
31. RFC-0004       Arabic Grammar Rule DSL
32. SPEC-0103      Performance Optimization Guide
33. SPEC-0601      Plugin System
35. SPEC-0001-C6   Deployment & Runtime Considerations
36. SPEC-0001-C7   Extensibility & Plugin Architecture
37. SPEC-0001-C8   Security, Validation & Error Handling
38. SPEC-0001-C9   Performance Targets & Constraints
39. KB-0008        Particles Developer Reference
```

**What you'll learn:** Performance tuning, plugin development, deployment configuration, security model, and the low-level particle lookup module.

---

## 5. Quick Reference by Topic

| Topic | Primary Document | Related Documents |
|-------|-----------------|-------------------|
| **Architectural rationale** | ADR-0001 | ADR-0002, ADR-0003, ADR-0004, ADR-0005 |
| **Pipeline architecture** | SPEC-0001-C2 | SPEC-0001-C3, SPEC-0001-C4, SPEC-0001-C5 |
| **Morphological analysis** | SPEC-0101 | KB-0001, KB-0002, KB-0003, KB-0004, KB-0005, KB-0006, KB-0007 |
| **Feature system** | SPEC-0102 | KB-0007 |
| **Performance optimization** | SPEC-0103 | SPEC-0001-C9 |
| **Rule engine** | SPEC-0201 | RFC-0001, RFC-0004 |
| **Grammar DSL** | RFC-0001 | RFC-0004, SPEC-0201 |
| **Grammar Runtime / GVM** | SPEC-0301 | RFC-0003, SPEC-0302, SPEC-0303, SPEC-0304 |
| **GVM instruction set** | SPEC-0302 | RFC-0003, SPEC-0301 |
| **GVM implementation** | SPEC-0303 | SPEC-0302, SPEC-0304, RFC-0003 |
| **GVM memory model** | SPEC-0304 | RFC-0003, SPEC-0301 |
| **Bytecode format** | RFC-0002 | SPEC-0301, ADR-0002 |
| **Knowledge graph** | SPEC-0401 | KB-0001, KB-0002, KB-0004 |
| **Explanation engine** | SPEC-0501 | SPEC-0301 |
| **Plugin system** | SPEC-0601 | SPEC-0001-C7, ADR-0005 |
| **Deployment** | SPEC-0001-C6 | ADR-0004 |
| **Security** | SPEC-0001-C8 | SPEC-0601 |
| **Performance targets** | SPEC-0001-C9 | SPEC-0103 |
| **KB overview** | KB-OVERVIEW | All KBs |
| **Particles (linguistic)** | KB-0005 | KB-0008 |
| **Particles (developer)** | KB-0008 | KB-0005, SPEC-0101 |

---

## 6. RFCs — Requests for Comments

| RFC | Title | Version | Status | Lines | Description |
|-----|-------|---------|--------|-------|-------------|
| RFC-0001 | Grammar DSL — Domain-Specific Language for Grammatical Rules | 0.1.0 | Draft | 1,224 | Defines the DSL syntax, rule structure, condition/action semantics, and built-in functions for authoring grammatical rules |
| RFC-0002 | Grammar Bytecode Format — Binary Container Specification | 0.1.0 | Draft | 1,339 | Defines the compiled binary format: magic bytes, header, section table, feature bitfields, string table, and instruction encoding |
| RFC-0003 | Grammar Virtual Machine — Instruction Set & Execution Model | 0.1.0 | Draft | 1,438 | Defines the GVM architecture: registers, stacks, memory regions, ~50 instruction opcodes, execution lifecycle, and verification |
| RFC-0004 | Arabic Grammar Rule DSL — School-Specific Rule Sets, Standard Library & Authoring Conventions | 0.1.0 | Draft | 2,016 | Extends RFC-0001 with school-specific rule sets, the standard library of built-in predicates/functions, and authoring conventions |

### RFC Dependency Graph

```
RFC-0001 (Grammar DSL)
    │
    ├──► RFC-0004 (Arabic Grammar Rule DSL)
    │         Extends RFC-0001 with school-specific rules
    │
    └──► SPEC-0201 (Rule Engine)
              Consumes DSL rules (RFC-0001 / RFC-0004)

RFC-0002 (Bytecode Format)
    │
    ├──► SPEC-0301 (Grammar Runtime)
    │         GVM executes bytecode (RFC-0002)
    ├──► SPEC-0302 (GVM Instruction Set)
    │         Instructions encoded in bytecode format
    └──► ADR-0002 (Why Bytecode)
              Architectural rationale

RFC-0003 (GVM Architecture)
    │
    ├──► SPEC-0301 (Grammar Runtime)
    │         Runtime implementation of GVM
    ├──► SPEC-0302 (GVM Instruction Set)
    │         Detailed instruction specifications
    ├──► SPEC-0303 (GVM Implementation Guide)
    │         Porting guide for implementers
    └──► SPEC-0304 (GVM Memory Model)
              Memory arena specifications
```

---

## 7. SPEC-0001 — Platform Architecture (9 Chapters)

SPEC-0001 is the foundational platform specification, organized into 9 independently versioned chapters.

| Chapter | Title | Version | Status | Lines | Key Content |
|---------|-------|---------|--------|-------|-------------|
| C1 | Introduction and Scope | 0.1.0 | Draft | ~40 | Platform vision, architectural philosophy, 12 core principles, target applications |
| C2 | System Architecture Overview | 0.1.0 | Draft | ~80 | Layered architecture (3 layers × 4 tiers), module catalog (MOD-01 through MOD-14), design precepts |
| C3 | Compilation Pipeline — Stage-by-Stage | 0.1.0 | Draft | ~120 | Complete pipeline walkthrough, 11 stage algorithms with pseudocode, IR contracts between stages |
| C4 | Module Responsibilities & Interfaces | 0.1.0 | Draft | ~100 | Formal interface definitions for all 14 modules, API contracts, function signatures |
| C5 | Data Flow & Intermediate Representations | 0.1.0 | Draft | ~80 | 11 IR schemas (IR-1 through IR-11), serialization formats, cache key design |
| C6 | Deployment & Runtime Considerations | 0.1.0 | Draft | ~100 | 3 deployment topologies (embedded, server, distributed), packaging, configuration, monitoring |
| C7 | Extensibility & Plugin Architecture | 0.1.0 | Draft | ~100 | Plugin types, lifecycle, WASM sandboxing, DSL overview, plugin distribution |
| C8 | Security, Validation & Error Handling | 0.1.0 | Draft | ~100 | Error classification, 50+ error codes, input validation, plugin security, audit trails |
| C9 | Performance Targets & Constraints | 0.1.0 | Draft | ~80 | Per-stage latency budgets, throughput targets, memory budgets, benchmarking methodology |

**Total SPEC-0001:** ~7,840 lines across 9 chapters.

### Reading Order Within SPEC-0001

```
C1 (Introduction) → C2 (Architecture) → C3 (Pipeline) → C4 (Interfaces) →
C5 (Data Flow) → C6 (Deployment) → C7 (Plugins) → C8 (Security) → C9 (Performance)
```

### Chapter Dependency Graph

```
C1 ──► C2 ──► C3 ──► C4 ──► C5
                               │
                  ┌────────────┼────────────┐
                  ▼            ▼            ▼
              C6 (Deploy)  C7 (Plugins)  C8 (Security)
                  │            │            │
                  └────────────┼────────────┘
                               ▼
                           C9 (Performance)
```

---

## 8. Module Specifications

The following SPEC documents define individual pipeline modules or component families in detail.

### 8.1 Morphology Engine Family (SPEC-01xx)

| Spec | Title | Version | Status | Lines | Covers |
|------|-------|---------|--------|-------|--------|
| SPEC-0101 | Morphology Engine — Detailed Implementation Specification | 1.0.0 | Draft | 2,465 | MOD-04 (MorphologicalParser) and MOD-05 (SyntaxParser): internal architecture, root extraction, wazan identification, feature extraction, syntactic parsing, school-specific behavior |
| SPEC-0102 | Morphological Features — Encoding, Validation & Resolution | 1.0.0 | Draft | 1,898 | Complete feature system: POS taxonomy (9 categories), 19 features across 4 categories, bitfield layout (64-bit), validation rules, inference rules, resolution system |
| SPEC-0103 | Morphology Engine Performance Optimization Guide | 1.0.0 | Draft | 2,040 | Hot path analysis, fast-path optimization, known words index, root extraction optimization, wazan caching, bitfield packing, benchmarking methodology |

### 8.2 Rule Engine (SPEC-02xx)

| Spec | Title | Version | Status | Lines | Covers |
|------|-------|---------|--------|-------|--------|
| SPEC-0201 | Rule Engine — Detailed Implementation Specification | 1.0.0 | Draft | 1,888 | MOD-07: rule lifecycle, DSL compilation, rule application engine (7 algorithms), evidence generation (5 types), school-specific behavior, 12+ construction recognizers |

### 8.3 Grammar Runtime / GVM Family (SPEC-03xx)

| Spec | Title | Version | Status | Lines | Covers |
|------|-------|---------|--------|-------|--------|
| SPEC-0301 | Grammar Runtime — GVM Execution & Explanation Generation | 0.1.0 | Draft | 3,165 | GVM architecture, verification (12 checks), instruction dispatch, step counting, tracing, 9-module architecture, cache/lookup/security modules |
| SPEC-0302 | GVM Instruction Set | 1.0.0 | Draft | 2,178 | Complete instruction catalog (~50 instructions across 9 categories: flow control, stack, token, feature, constituent, rule, evidence, output, extension) |
| SPEC-0303 | GVM Implementation Guide | 1.0.0 | Draft | 2,558 | Porting guide with patterns for 6 languages (Rust, C, Python, TypeScript, Go, Zig), SSA form, diagnostic tooling, optimization techniques |
| SPEC-0304 | GVM Memory Model | 1.0.0 | Draft | 2,171 | 7 memory regions (token, feature, constituent, string, rule, evidence, scratch) + 2 stacks, bump allocator, budget calculation, 215 conformance tests |

### 8.4 Knowledge Graph Engine (SPEC-04xx)

| Spec | Title | Version | Status | Lines | Covers |
|------|-------|---------|--------|-------|--------|
| SPEC-0401 | Knowledge Graph Engine | 0.1.0 | Draft | 1,980 | MOD-08: query system (3 query types), entity resolution, KB reader (memory-mapped, 7 KBs), link analysis, cross-referencing, 6 resolution strategies |

### 8.5 Explanation Engine (SPEC-05xx)

| Spec | Title | Version | Status | Lines | Covers |
|------|-------|---------|--------|-------|--------|
| SPEC-0501 | Explanation Engine — Template System, I'rab Generation, LLM Integration & Output Formatting | 0.1.0 | Draft | 4,412 | MOD-11: template system (Handlebars-based), I'rab generation (5 breakdown types), LLM integration (3 provider interfaces), 6 output formats, educational features |

### 8.6 Plugin System (SPEC-06xx)

| Spec | Title | Version | Status | Lines | Covers |
|------|-------|---------|--------|-------|--------|
| SPEC-0601 | Plugin System — Deep-Dive Specification for MOD-12 | 1.0.0 | Draft | 2,539 | MOD-12: 9 plugin types, lifecycle management, manifest format, WASM sandboxing, capability-based security, distribution, registry, SDK guide |

---

## 9. ADRs — Architecture Decision Records

| ADR | Title | Version | Status | Lines | Key Decision |
|-----|-------|---------|--------|-------|-------------|
| ADR-0001 | Why AGOS Uses a Compiler Architecture Instead of Direct NLP | 1.0.0 | Accepted | 460 | Multi-stage pipeline (like LLVM/GCC) rather than neural/statistical NLP; 5 alternatives considered and rejected |
| ADR-0002 | Why AGOS Compiles Grammar Analysis into Bytecode | 1.0.0 | Accepted | 493 | Custom Grammar Bytecode (~50 instructions) rather than WASM, JSON, or direct GIR output; 6 alternatives considered |
| ADR-0003 | Why AGOS Introduces a Dedicated Grammar Intermediate Representation (GIR) | 1.0.0 | Accepted | 599 | Formal, versioned, serializable GIR (IR-6/7/8) as front-end/back-end boundary; 5 alternatives considered |
| ADR-0004 | Why AGOS Adopts an Offline-First Architecture | 1.0.0 | Accepted | 581 | Core analysis fully local; network optional for updates/plugins only; 375× cost savings over cloud-only |
| ADR-0005 | Why AGOS Adopts a Plugin Architecture | 1.0.0 | Accepted | 739 | WASM-based plugins with capability-based security; 9 plugin types; 6 alternatives considered |

### ADR Dependency Graph

```
ADR-0001 (Compiler Architecture)
    ├──► ADR-0002 (Bytecode)
    │         └──► ADR-0003 (GIR)
    ├──► ADR-0004 (Offline-First)
    └──► ADR-0005 (Plugin Architecture)

The ADRs form a tree: ADR-0001 is the root decision,
and ADRs 2–5 refine specific aspects of it.
```

---

## 10. KBs — Knowledge Base Specifications

### 10.1 KB Suite Overview

| KB | Title | Version | Status | Lines | Scope | Format |
|----|-------|---------|--------|-------|-------|--------|
| KB-OVERVIEW | AGOS Knowledge Base Suite — Overview & Architecture | 1.0.0 | Draft | 231 | Cross-KB architecture, dependency graph, combined resource budgets, pipeline integration, version compatibility matrix | — |
| KB-0001 | Roots Database | 1.0.0 | Draft | 1,536 | ~15,000–20,000 Arabic roots: triliteral (~85%), quadriliteral (~12–15%), with root types, verb forms, derived nouns, semantic fields | Binary trie (~20–80 MB) |
| KB-0002 | Wazan Database — Morphological Patterns | 1.0.0 | Draft | 1,644 | ~300–450 patterns: verb forms I–XV, derived noun patterns, weak root variants, quadriliteral patterns | Hash index (~10–40 MB) |
| KB-0003 | Verb Forms — Conjugation Paradigms | 1.0.0 | Draft | 1,825 | ~180–250 paradigm tables across 15 conjugation classes, 13 slots per table, 4 moods × 2 tenses | Table binary (~15–30 MB) |
| KB-0004 | Noun Patterns — Derived Noun Specifications | 1.0.0 | Draft | 1,677 | ~135–180 patterns: masdars, participles, place/time, instrument, adjectives, elative, nisbah, broken plurals (~30+ templates) | Table binary (~10–30 MB) |
| KB-0005 | Particles — Grammatical & Functional Words | 1.0.0 | Draft | 1,564 | ~120–200 particle entries across 13+ categories: prepositions, conjunctions, subjunctive, jussive, conditional, interrogative, negative, vocative, inna/kana sisters | Hash index (~2–5 MB) |
| KB-0006 | Pronouns — Personal, Demonstrative & Relative | 1.0.0 | Draft | 1,296 | ~60–80 pronoun entries: personal (attached + detached), demonstrative (near/mid/far), relative (definite + indefinite), interrogative, conditional | Hash index (~1–2 MB) |
| KB-0007 | Morphological Features — Taxonomy, Bitfield Encoding & Inference Rules | 1.0.0 | Draft | 2,089 | 19 features (1 POS + 8 inflectional + 5 derivational + 2 prosodic + 3 orthographic), 64-bit bitfield layout, 6 agreement rules, 15 inference rules, 12+ constraints | Feature map (~1–2 MB) |
| KB-0008 | Particles Database — Developer Reference & Compiled Module | 1.0.0 | Draft | 2,132 | Compiled binary format, perfect hash index, lookup API, governance resolution, MOD-04/05 integration, homograph disambiguation engine (~255 tests) | Compiled `.agos-kb` (~190–520 KB) |

### 10.2 Cross-KB Dependency Graph

```
KB-0001 (Roots) — Foundation (no dependencies)
    │
    ├──► KB-0002 (Wazan) — Depends on KB-0001 for root types
    │     │
    │     ├──► KB-0003 (Verb Forms) — Depends on KB-0001 + KB-0002
    │     └──► KB-0004 (Noun Patterns) — Depends on KB-0001 + KB-0002
    │
    ├──► KB-0005 (Particles) — Independent (no root dependency) [Fast-path]
    ├──► KB-0006 (Pronouns) — Independent (no root dependency) [Fast-path]
    ├──► KB-0007 (Features) — Cross-cutting (referenced by ALL KBs)
    │
    └──► KB-0008 (Particles Dev) — Depends on KB-0005 (linguistic content)
    │
    └──► KB-OVERVIEW — References all KBs
```

### 10.3 KB Compilation Targets

| KB | Source Format | Source Size | Compiled Format | Compiled Size (Compact) | Compiled Size (Full) |
|----|--------------|-------------|-----------------|------------------------|---------------------|
| KB-0001 | YAML/JSON | ~200 MB | Binary trie | ~20 MB | ~80 MB |
| KB-0002 | YAML/JSON | ~50 MB | Hash index | ~10 MB | ~40 MB |
| KB-0003 | YAML/JSON | ~50–80 MB | Table binary | ~15 MB | ~30 MB |
| KB-0004 | YAML/JSON | ~30–50 MB | Table binary | ~10 MB | ~30 MB |
| KB-0005 | YAML/JSON | ~1–2 MB | Hash index | ~2 MB | ~5 MB |
| KB-0006 | YAML/JSON | ~0.5–1 MB | Hash index | ~1 MB | ~2 MB |
| KB-0007 | YAML/JSON | ~0.5–1 MB | Feature map | ~1 MB | ~2 MB |
| KB-0008 | Compiled from KB-0005 | — | `.agos-kb` binary | ~190 KB | ~520 KB |
| **Total** | | **~332–382 MB** | | **~59 MB** | **~189 MB** |

### 10.4 KB Performance Budgets

| Operation | KB-0001 | KB-0002 | KB-0003 | KB-0004 | KB-0005 | KB-0006 | KB-0007 |
|-----------|---------|---------|---------|---------|---------|---------|---------|
| **Primary lookup** | < 1 μs | < 500 ns | < 1 μs | < 2 μs | < 500 ns | < 500 ns | < 200 ns |
| **Full analysis** | < 10 ms | < 20 μs | < 30 μs | < 30 μs | < 1 μs | < 3 μs | < 5 μs |
| **KB load (compact)** | < 50 ms | < 25 ms | < 25 ms | < 25 ms | < 10 ms | < 5 ms | < 5 ms |

---

## 11. Pipeline-to-Spec Mapping

The following table maps each pipeline stage (MOD-) to the specifications that define it.

| Stage | Module | Spec | KB Inputs | ADR References |
|-------|--------|------|-----------|----------------|
| MOD-01 | UnicodeValidator | SPEC-0001-C3, SPEC-0001-C8 | — | ADR-0001 |
| MOD-02 | Lexer | SPEC-0001-C3 | — | ADR-0001 |
| MOD-03 | Tokenizer | SPEC-0001-C3, SPEC-0101 | KB-0005, KB-0006 | ADR-0001 |
| MOD-04 | MorphologicalParser | SPEC-0101, SPEC-0102, SPEC-0103 | KB-0001–KB-0007 | ADR-0001 |
| MOD-05 | SyntaxParser | SPEC-0101 | KB-0005, KB-0006, KB-0007 | ADR-0001 |
| MOD-06 | GIRConstructor | SPEC-0001-C5 | — | ADR-0003 |
| MOD-07 | RuleEngine | SPEC-0201, RFC-0001, RFC-0004 | KB-0007 | ADR-0001, ADR-0005 |
| MOD-08 | KnowledgeGraphResolver | SPEC-0401 | KB-0001, KB-0002, KB-0004 | ADR-0001 |
| MOD-09 | BytecodeGenerator | RFC-0002 | — | ADR-0002 |
| MOD-10 | GVM | SPEC-0301, RFC-0003, SPEC-0302, SPEC-0303, SPEC-0304 | — | ADR-0002 |
| MOD-11 | ExplanationEngine | SPEC-0501 | All KBs | ADR-0001 |
| MOD-12 | PluginLoader | SPEC-0601, SPEC-0001-C7 | — | ADR-0005 |
| MOD-13 | CacheManager | SPEC-0001-C5, SPEC-0103 | — | ADR-0004 |
| MOD-14 | APIGateway | SPEC-0001-C4, SPEC-0001-C6 | — | ADR-0004 |

### Pipeline Visualization

```
Input Text
    │
    ▼
MOD-01: UnicodeValidator
    │  SPEC-0001-C3, SPEC-0001-C8
    ▼
MOD-02: Lexer
    │  SPEC-0001-C3
    ▼
MOD-03: Tokenizer
    │  SPEC-0001-C3, SPEC-0101  ◄── KB-0005, KB-0006
    ▼
MOD-04: MorphologicalParser
    │  SPEC-0101, SPEC-0102, SPEC-0103  ◄── KB-0001..KB-0007
    ▼
MOD-05: SyntaxParser
    │  SPEC-0101  ◄── KB-0005, KB-0006, KB-0007
    ▼
MOD-06: GIRConstructor
    │  SPEC-0001-C5
    ▼
┌─────────────────────────────────────────────────────┐
│                  GIR Boundary                         │
│              (Caching / Distribution)                  │
│              ADR-0003, SPEC-0001-C5                   │
└─────────────────────────────────────────────────────┘
    │
    ▼
MOD-07: RuleEngine
    │  SPEC-0201, RFC-0001, RFC-0004  ◄── KB-0007
    ▼
MOD-08: KnowledgeGraphResolver
    │  SPEC-0401  ◄── KB-0001, KB-0002, KB-0004
    ▼
MOD-09: BytecodeGenerator
    │  RFC-0002
    ▼
MOD-10: GVM
    │  SPEC-0301, RFC-0003, SPEC-0302, SPEC-0303, SPEC-0304
    ▼
MOD-11: ExplanationEngine
    │  SPEC-0501
    ▼
Output
```

---

## 12. Cross-Reference Directory

### 12.1 Specs Referenced by Others (Most Referenced First)

| Spec ID | Title | Referenced By |
|---------|-------|---------------|
| SPEC-0001 | Platform Architecture (all chapters) | Every other spec — foundational architecture |
| KB-0007 | Morphological Features | Every KB and most SPECs — shared feature vocabulary |
| ADR-0001 | Compiler Architecture Rationale | ADR-0002, ADR-0003, ADR-0004, ADR-0005 |
| KB-0005 | Particles | SPEC-0101, KB-0008, SPEC-0201, SPEC-0501 |
| KB-0001 | Roots Database | KB-0002, KB-0003, KB-0004, KB-0007, SPEC-0101 |
| RFC-0002 | Bytecode Format | SPEC-0301, SPEC-0302, ADR-0002 |
| RFC-0003 | Grammar Virtual Machine | SPEC-0301, SPEC-0302, SPEC-0303, SPEC-0304 |

### 12.2 Cross-Reference Index

Use this table to find which documents reference each spec:

| Spec | Referenced in |
|------|--------------|
| ADR-0001 | ADR-0002, ADR-0003, ADR-0004, ADR-0005, SPEC-0001 (all chapters) |
| ADR-0002 | ADR-0003, SPEC-0001-C3/C5/C6/C9, RFC-0002, RFC-0003 |
| ADR-0003 | SPEC-0001-C2/C3/C5/C7, SPEC-0201, SPEC-0401, RFC-0001 |
| ADR-0004 | SPEC-0001-C6/C9, SPEC-0301, SPEC-0601 |
| ADR-0005 | SPEC-0001-C4/C7, SPEC-0601, RFC-0001 |
| RFC-0001 | SPEC-0001-C7, SPEC-0201, SPEC-0601, ADR-0005 |
| RFC-0002 | SPEC-0001-C3/C5/C6/C9, SPEC-0301, SPEC-0302, ADR-0002 |
| RFC-0003 | SPEC-0001-C6, SPEC-0301, SPEC-0302, SPEC-0303, SPEC-0304, ADR-0002 |
| RFC-0004 | SPEC-0201, SPEC-0601 |
| SPEC-0001 (all ch.) | Every other spec — foundational architecture reference |
| SPEC-0101 | SPEC-0001-C9, SPEC-0102, SPEC-0103 |
| SPEC-0102 | KB-0007, SPEC-0001-C9 |
| SPEC-0103 | SPEC-0001-C9, SPEC-0101 |
| SPEC-0201 | SPEC-0001-C3/C9 |
| SPEC-0301 | SPEC-0001-C6/C9, RFC-0003 |
| SPEC-0302 | SPEC-0301, RFC-0003 |
| SPEC-0303 | SPEC-0301, SPEC-0302, SPEC-0304 |
| SPEC-0304 | SPEC-0301, RFC-0003 |
| SPEC-0401 | SPEC-0001-C9 |
| SPEC-0501 | SPEC-0001-C9 |
| SPEC-0601 | SPEC-0001-C2/C4/C7/C8/C9 |
| KB-0001 | KB-0002, KB-0003, KB-0004, KB-0007, SPEC-0101 |
| KB-0002 | KB-0003, KB-0004, KB-0007, SPEC-0101 |
| KB-0003 | KB-0007, SPEC-0101 |
| KB-0004 | KB-0007, SPEC-0101 |
| KB-0005 | KB-0007, KB-0008, SPEC-0101, SPEC-0501 |
| KB-0006 | KB-0007, SPEC-0101, SPEC-0501 |
| KB-0007 | ALL KBs, ALL SPECs (except SPEC-0001-C1), RFC-0002 |
| KB-0008 | SPEC-0101, KB-0005, SPEC-0102 |
| KB-OVERVIEW | All KBs |

---

## 13. Specification Statistics

### 13.1 Size by Category

| Category | Documents | Total Lines | Average Lines | Largest |
|----------|-----------|-------------|---------------|---------|
| **RFCs** | 4 | 6,017 | 1,504 | RFC-0004 (2,016) |
| **SPECs** | 20 | 58,222 | 2,911 | SPEC-0501 (4,412) |
| **ADRs** | 5 | 2,872 | 574 | ADR-0005 (739) |
| **KBs** | 9 | 14,250 | 1,583 | KB-0008 (2,132) |
| **DEPs** | 0 | 0 | — | — |
| **Index** | 1 | ~500 | 500 | — |
| **Total** | **39** | **~81,861** | **~2,099** | **SPEC-0501 (4,412)** |

### 13.2 Size by Status

| Status | Documents | Lines |
|--------|-----------|-------|
| **Accepted** | 5 (all ADRs) | 2,872 |
| **Draft** | 33 (all RFCs + SPECs + KBs) | 58,222 |
| **Published** | 1 (this index) | ~500 |

### 13.3 Module Coverage

| Module | Specified In | Lines Dedicated |
|--------|-------------|-----------------|
| MOD-01 UnicodeValidator | SPEC-0001-C3, SPEC-0001-C8 | ~100 |
| MOD-02 Lexer | SPEC-0001-C3 | ~50 |
| MOD-03 Tokenizer | SPEC-0001-C3 | ~50 |
| MOD-04 MorphologicalParser | SPEC-0101, SPEC-0102, SPEC-0103 | ~6,400 |
| MOD-05 SyntaxParser | SPEC-0101 | ~800 |
| MOD-06 GIRConstructor | SPEC-0001-C5 | ~80 |
| MOD-07 RuleEngine | SPEC-0201 | ~1,900 |
| MOD-08 KnowledgeGraphResolver | SPEC-0401 | ~2,000 |
| MOD-09 BytecodeGenerator | RFC-0002 | ~1,300 |
| MOD-10 GVM | SPEC-0301, RFC-0003, SPEC-0302, SPEC-0303, SPEC-0304 | ~11,500 |
| MOD-11 ExplanationEngine | SPEC-0501 | ~4,400 |
| MOD-12 PluginLoader | SPEC-0601, SPEC-0001-C7 | ~2,600 |
| MOD-13 CacheManager | SPEC-0001-C5, SPEC-0103 | ~80 |
| MOD-14 APIGateway | SPEC-0001-C4, SPEC-0001-C6 | ~60 |

### 13.4 Top 5 Largest Documents

| Rank | Document | Lines | % of Total | Primary Topic |
|------|----------|-------|------------|---------------|
| 1 | SPEC-0501 | 4,412 | 7.1% | Explanation Engine |
| 2 | SPEC-0301 | 3,165 | 5.1% | Grammar Runtime |
| 3 | SPEC-0303 | 2,558 | 4.1% | GVM Implementation Guide |
| 4 | SPEC-0601 | 2,539 | 4.1% | Plugin System |
| 5 | SPEC-0101 | 2,465 | 4.0% | Morphology Engine |

---

## 14. Glossary of ID Prefixes

| Prefix | Document Type | Numbering | Example |
|--------|--------------|-----------|---------|
| **RFC-** | Request for Comments | Sequential: 0001+ | RFC-0001: Grammar DSL |
| **SPEC-** | Official Specification | Modular: 0001 (architecture), 01xx (morphology), 02xx (rules), 03xx (GVM), 04xx (KG), 05xx (explanation), 06xx (plugins) | SPEC-0101: Morphology Engine |
| **SPEC-0001-C** | SPEC Chapter | Chapter number: 1–9 | SPEC-0001-C3: Compilation Pipeline |
| **ADR-** | Architecture Decision Record | Sequential: 0001+ | ADR-0003: Why Grammar IR |
| **DEP-** | Deprecation Proposal | Sequential: 0001+ | DEP-0001: (future) |
| **KB-** | Knowledge Base | Sequential: 0001+ with functional grouping | KB-0007: Morphological Features |
| **KB-OVERVIEW** | KB Overview | Special | KB-OVERVIEW |
| **MOD-** | Pipeline Module | Sequential: 01–14 | MOD-04: MorphologicalParser |
| **IR-** | Intermediate Representation | Sequential: 1–11 | IR-4: MorphologicalAnalysis |

---

*End of AGOS Specification Suite — Master Index*

---

**Maintained by:** AGOS Architecture Committee

**Purpose:** This index is a living document that SHOULD be updated whenever new specifications are added, removed, or significantly revised. It is the canonical starting point for navigating the AGOS specification suite.

**Source of truth:** The specification files themselves (under `specs/`). This index is a navigation aid and may not reflect every detail of each specification.
