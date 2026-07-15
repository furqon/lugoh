---
adr_id: ADR-0002
title: Why AGOS Compiles Grammar Analysis into Bytecode
version: 1.0.0
status: Accepted
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
decided: 2026-07-13
references:
  - ADR-0001: Compiler Architecture Rationale
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-9)
  - SPEC-0001-C3: Compilation Pipeline (MOD-09 BytecodeGenerator)
  - SPEC-0001-C3: Compilation Pipeline (MOD-10 GVM)
  - SPEC-0001-C6: Deployment & Runtime Considerations
  - SPEC-0001-C9: Performance Targets (Bytecode size targets, GVM latency)
  - RFC-0002: Grammar Bytecode Format (proposed)
  - RFC-0003: Grammar Virtual Machine (proposed)
  - ADR-0003: Why Grammar IR (planned)
supersedes: None
---

# ADR-0002: Why AGOS Compiles Grammar Analysis into Bytecode

## Table of Contents

1. [Context](#1-context)
2. [Problem Statement](#2-problem-statement)
3. [Decision](#3-decision)
4. [Alternatives Considered](#4-alternatives-considered)
5. [Detailed Rationale](#5-detailed-rationale)
6. [Consequences](#6-consequences)
7. [Implementation Guidance](#7-implementation-guidance)
8. [Status](#8-status)

---

## 1. Context

ADR-0001 established that AGOS uses a compiler architecture — a multi-stage pipeline that transforms Arabic text through successive intermediate representations into a grammatical analysis. The pipeline culminates in two final stages:

- **MOD-09 (BytecodeGenerator):** Compiles the resolved Grammar Intermediate Representation (GIR) into a compact binary format called **Grammar Bytecode**.
- **MOD-10 (GVM — Grammar Virtual Machine):** Executes the Grammar Bytecode to produce the final `AnalysisResult`.

The decision to introduce bytecode and a virtual machine as the final compilation step — rather than directly outputting the AnalysisResult from the GIR — requires its own architectural rationale.

The key question is: **after all grammatical analysis is complete (rules applied, knowledge resolved), why compile the result into bytecode and execute it in a virtual machine, instead of outputting the analysis directly?**

---

## 2. Problem Statement

### 2.1 The Direct Output Approach

The simplest architecture would bypass MOD-09 and MOD-10 entirely:

```
GIR → RuleEngine → KnowledgeGraphResolver → ExplanationEngine → Output
```

In this approach, the Explanation Engine receives the ResolvedGIR directly and generates human-readable explanations. This is the path of least resistance — fewer components, less code, simpler reasoning.

### 2.2 Why Direct Output Is Insufficient

The direct output approach fails on several critical requirements:

#### 2.2.1 Serialization and Portability

Without bytecode, there is no canonical, self-contained representation of a completed grammatical analysis. The ResolvedGIR is a large JSON structure (~50–200 KB per sentence) that:

- References external knowledge bases (root IDs, wazan IDs, etc.) — the analysis is not interpretable without the KBs.
- Uses human-readable field names and string values — verbose and wasteful for storage.
- Is tied to the GIR schema version — any schema change requires re-analysis of stored results.

A portable, self-contained format is needed for:

- **Caching:** Storing analyses for reuse across pipeline restarts.
- **Transmission:** Sending analyses between distributed pipeline nodes.
- **Archival:** Persisting analyses for research, audit, and educational use.
- **Offline sharing:** Distributing analyses (e.g., pre-computed Quran grammar) to mobile and embedded devices.

#### 2.2.2 Execution Guarantees

Without a virtual machine, there is no mechanism to enforce:

- **Bounded execution time:** A complex GIR could cause the Explanation Engine to spend unbounded time processing. The GVM provides a configurable step limit.
- **memory safety:** A buggy or malicious rule set plugin could corrupt memory through the Explanation Engine. The GVM provides a sandboxed execution environment.
- **Deterministic replay:** Two different Explanation Engine implementations could produce different outputs from the same GIR. The GVM guarantees byte-for-byte identical execution across all compliant implementations.

#### 2.2.3 Language Independence

The pipeline stages (MOD-01 through MOD-09) are best implemented in a systems language like Rust for performance. But applications may be written in Python, JavaScript, Java, or Swift. Without bytecode:

- Each language would need its own implementation of the explanation generation logic.
- Analysis results could not be shared across language boundaries without a common format.

Bytecode provides a language-independent execution layer. A Python application can load a Python GVM, a Swift app can load a Swift GVM, and both execute the same bytecode identically.

#### 2.2.4 Optimization

The GIR is an **analysis** — it represents what was found. Bytecode is an **execution plan** — it represents what to do with what was found. This distinction enables optimizations:

- **Dead code elimination:** If certain features are not requested (e.g., semantic tags), the bytecode can skip resolution steps.
- **Pre-computation:** Frequently accessed paths in the explanation can be pre-computed during bytecode generation.
- **Instruction fusion:** Multiple small operations can be fused into single bytecode instructions for faster execution.

---

## 3. Decision

**AGOS SHALL compile the resolved GIR into Grammar Bytecode** — a compact, self-contained, versioned binary format — and **execute it in the Grammar Virtual Machine (GVM)** to produce the final analysis result.

```
ResolvedGIR (MOD-08 output)
    │
    ▼
MOD-09: BytecodeGenerator
    │  Compiles GIR → Grammar Bytecode
    │  Output is a self-contained binary (no external KB dependencies)
    │  Optimizations applied (level 0/1/2)
    ▼
Grammar Bytecode (RFC-0002 format)
    │
    ▼
MOD-10: Grammar Virtual Machine
    │  Executes bytecode deterministically
    │  Enforces bounds (steps, memory)
    │  Produces AnalysisResult
    ▼
AnalysisResult → MOD-11: ExplanationEngine
```

The separation between MOD-09 and MOD-10 is intentional and permanent. Bytecode generation is a **compilation** step. GVM execution is an **interpretation** step. They are separate concerns:

- MOD-09 transforms GIR → Bytecode. It is part of the Compilation Layer.
- MOD-10 transforms Bytecode → AnalysisResult. It is part of the Runtime Layer.

This separation enables the Runtime Layer to be completely independent of the Compilation Layer (per Layer Rule 3 from SPEC-0001 Chapter 2).

---

## 4. Alternatives Considered

### 4.1 Alternative A: Direct GIR → Explanation (No Bytecode)

**Description:** The Explanation Engine reads the ResolvedGIR directly and generates explanations without bytecode or a virtual machine.

**Advantages:**
- Simplest architecture — two fewer modules (MOD-09, MOD-10).
- Lowest latency (no compilation step).
- No bytecode format to design, version, and maintain.

**Disadvantages (why rejected):**
- **No portable format.** The GIR is not self-contained. It references KB IDs that require the full KB installation to interpret. A JSON export of the GIR is ~50–200 KB per sentence — too large for mobile apps, caching, or archival at scale.
- **No execution guarantees.** The Explanation Engine processes the GIR directly in-process with no sandbox, no step limits, and no memory bounds. A complex or malicious input could cause unbounded processing.
- **Language lock-in.** The Explanation Engine logic is tied to the implementation language (likely Rust). Applications in other languages would need to reimplement explanation generation.
- **No optimization potential.** Every explanation from the same GIR recomputes the same derived values. Bytecode enables pre-computation at generation time.
- **No deterministic replay guarantee.** Two Explanation Engine implementations (e.g., different versions) could produce different explanations from the same GIR. Bytecode + GVM guarantees identical output.

**Verdict: REJECTED.** The lack of portability, execution guarantees, and language independence makes this approach unsuitable for a platform.

### 4.2 Alternative B: JSON Serialization of AnalysisResult

**Description:** Instead of bytecode, serialize the AnalysisResult directly as a JSON/Protobuf document. Store, cache, and transmit this document.

**Advantages:**
- Human-readable format (JSON).
- No custom binary format to design.
- Protobuf provides schema evolution.

**Disadvantages (why rejected):**
- **No execution guarantees.** JSON is a passive data format. It cannot enforce bounded execution, memory safety, or deterministic processing. The consumer of the JSON still needs to implement the processing logic.
- **Verbose.** A JSON AnalysisResult for a 10-word sentence is typically ~50 KB. The bytecode equivalent is ~2–5 KB (95%+ reduction).
- **Schema coupling.** JSON representation is tightly coupled to the GIR schema. Schema changes require data migration or versioning at the application layer.
- **No optimization.** JSON is a literal representation of the analysis. There is no opportunity to optimize, fuse instructions, or pre-compute derived values.
- **No execution semantics.** JSON is data, not instructions. The consumer must re-implement the "execution" of the analysis. This duplicates the GVM's logic in every consumer.

**Verdict: REJECTED.** JSON/Protobuf serialization addresses the portability requirement but fails on execution guarantees, optimization, and size.

### 4.3 Alternative C: Pre-Computed Analysis Database

**Description:** Pre-compute analyses for all expected inputs (common Quranic phrases, Hadith, MSA sentences) and store them in a database. Look up analyses by input hash.

**Advantages:**
- Fastest possible retrieval (database lookup).
- No runtime bytecode generation or execution.
- Simple architecture for known inputs.

**Disadvantages (why rejected):**
- **Incomplete coverage.** Cannot pre-compute analyses for arbitrary user input. Novel sentences would require fallback to the full pipeline, creating two distinct code paths.
- **Storage explosion.** For a corpus of 1 million unique sentences: ~5 GB (bytecode) vs. ~50 GB (JSON). The bytecode format reduces this by 10×, but pre-computation at scale is still impractical.
- **Synchronization complexity.** Pre-computed analyses must be invalidated when KBs or rule sets change. This requires tracking which analyses depend on which KB versions.
- **Cold start problem.** New deployments would need to pre-compute analyses before serving traffic.

**Verdict: REJECTED.** Pre-computation is complementary to bytecode (caching), not a replacement for it.

### 4.4 Alternative D: AST Walking (No Bytecode Format)

**Description:** Traverse the ResolvedGIR as an abstract syntax tree (AST) using a visitor pattern. Extract data as needed without compiling to bytecode.

**Advantages:**
- No bytecode format to design.
- No compilation step.
- Low memory (no bytecode storage).

**Disadvantages (why rejected):**
- **AST coupling.** The visitor logic is tightly coupled to the GIR schema. Any GIR schema change requires updating all visitors across all language implementations.
- **No self-contained format.** The analysis cannot be serialized, cached, or transmitted without the full GIR structure.
- **Performance.** AST walking for every analysis recomputes the traversal from scratch. Bytecode generation pre-compiles the traversal.
- **No sandboxing.** AST walking runs in-process with full access to the host system. There is no security boundary.

**Verdict: REJECTED.** AST walking is an implementation technique, not an architectural solution to the portability, serialization, and security requirements.

### 4.5 Alternative E: Compile Grammar DSL to WebAssembly

**Description:** Instead of designing a custom bytecode format, compile the Grammar DSL directly to WebAssembly (WASM). Use existing WASM runtimes (wasmtime, wasmer, browser WASM engines) as the execution layer.

**Advantages:**
- **Mature infrastructure.** WASM has existing compilers, debuggers, profilers, and runtimes in every major language.
- **Sandboxing built-in.** WASM provides linear memory isolation, capability-based security, and bounded execution.
- **Language independence.** WASM runtimes exist for Rust, C, Python, JavaScript, Swift, Kotlin, Go — every target AGOS needs.
- **Standardized format.** WASM is a W3C standard with multiple implementations and a well-defined specification.

**Disadvantages (why rejected):**
- **General-purpose overhead.** WASM is designed for general computation. A grammar DSL compiled to WASM would carry the overhead of general-purpose control flow, stack management, and data structures irrelevant to grammatical analysis. AGOS bytecode can skip all of that and use ~50 domain-specific instructions instead of WASM's ~150+ MVP instructions.
- **No domain-specific optimizations.** WASM cannot natively represent morphological features, case markers, syntactic roles, or I'rab attributes. These would be encoded as opaque integers or strings, losing the domain semantics that enable AGOS bytecode's feature bitfields, string interning, and instruction fusion optimizations.
- **Memory model mismatch.** WASM's linear memory with manual management adds complexity that AGOS bytecode avoids with pre-allocated type-specific regions for tokens, features, and tree nodes.
- **Format size.** A WASM module representing the same analysis would be larger than dedicated bytecode because WASM requires explicit type definitions, function signatures, and module structure that are implicit in a domain-specific format.
- **Version coupling.** WASM's versioning is tied to the WASM specification, not to AGOS's grammar analysis needs. AGOS bytecode can version independently and evolve its instruction set as Arabic grammar analysis requirements change.

**Verdict: REJECTED.** WASM is the right choice for a general-purpose sandboxed execution environment, but AGOS needs a domain-specific format optimized for grammatical analysis. The custom bytecode format wins on size, optimization, and semantic expressiveness.

### 4.6 Alternative F: Grammar Bytecode + GVM (Chosen)

**Description:** Compile the resolved GIR into a compact, versioned binary bytecode format. Execute the bytecode in a sandboxed Grammar Virtual Machine.

**Advantages:**
- See [Section 5 — Detailed Rationale](#5-detailed-rationale).

**Disadvantages (addressed):**
- **Design cost.** A bytecode format requires design, documentation, versioning, and migration tooling. Mitigated by drawing on proven precedents (JVM bytecode, WebAssembly).
- **Compilation overhead.** The bytecode generation step adds ~200 μs per sentence (see SPEC-0001 Chapter 9). Mitigated by caching — common inputs skip compilation entirely.
- **Two formats to maintain.** Both GIR and bytecode must be versioned and maintained. Mitigated by the fact that GIR is the "source" and bytecode is the "compiled target" — only the GIR schema changes drive bytecode version changes.
- **Remote debugging complexity.** Debugging a GVM running in a different process or on a mobile device requires bytecode-level diagnostic tools. Mitigated by the GVM's built-in tracing and verification capabilities.

**Verdict: ACCEPTED.** Grammar Bytecode + GVM is the only approach that simultaneously satisfies all requirements: portability, execution guarantees, language independence, optimization, and security.

---

## 5. Detailed Rationale

### 5.1 Five Reasons for Bytecode

#### Reason 1: Self-Contained Portability

Grammar Bytecode is **self-contained** — it includes all information needed to produce an AnalysisResult:

- The input text (or its hash, for deduplication).
- The token stream and morphological features.
- The syntax trees and constituent structure.
- The rule applications and evidence trail.
- The KB references resolved to inline data.

A GVM can execute bytecode without access to the original KBs, rule sets, or pipeline configuration. This enables:

- **Pre-computed analyses** on mobile devices (download once, analyze offline).
- **Distributed pipeline** where bytecode is the transfer format between stages.
- **Archival** of analyses in a compact, versioned format.
- **Sharing** of analyses between users and applications.

#### Reason 2: Execution Guarantees

The GVM provides:

| Guarantee | Mechanism |
|-----------|-----------|
| **Bounded steps** | Configurable `max_execution_steps` (default: 100,000). Infinite loops are detected and terminated. |
| **Bounded memory** | Pre-allocated memory region (64–256 MB). Out-of-bounds access is detected and terminated. |
| **Deterministic output** | Same bytecode + same config = byte-for-byte identical output across all runs, all platforms, all implementations. |
| **Side-effect free** | GVM cannot access files, network, or system calls. It produces only an AnalysisResult. |

These guarantees are enforced by the GVM runtime, not by convention. Every bytecode instruction is validated before execution.

#### Reason 3: Language Independence

The bytecode format is defined at the binary level (RFC-0002). Any language can implement a GVM:

| Language | GVM Implementation | Use Case |
|----------|-------------------|----------|
| Rust | Primary | Server, embedded library |
| C | Secondary | Embedded systems |
| Python | Ecosystem | ML integration, research |
| JavaScript/TypeScript | Ecosystem | Browser-based AGOS (WASM) |
| Swift | Ecosystem | iOS app |
| Kotlin | Ecosystem | Android app |
| Go | Ecosystem | CLI tools, microservices |

Each GVM implementation accepts the same bytecode and produces the same AnalysisResult. This decouples analysis production (compilation pipeline, Rust) from analysis consumption (applications, any language).

#### Reason 4: Optimization at Compile Time

Bytecode generation (MOD-09) is a compilation step that can optimize the GIR before execution:

| Optimization | Description | Impact |
|-------------|-------------|--------|
| **String interning** | Repeated strings stored once in string table | 30–50% size reduction |
| **Feature bitfields** | Morphological features packed into compact bitfields | 10–20× size reduction vs. JSON |
| **Dead code elimination** | Skip features not requested in configuration | 5–15% execution time reduction |
| **Instruction fusion** | Common patterns fused into single instructions | 10–20% execution time reduction |
| **Delta encoding** | Sequential indices encoded as deltas | 10–30% size reduction |

These optimizations are applied once at bytecode generation time (MOD-09), not repeatedly at execution time (MOD-10).

#### Reason 5: Clear Architecture Boundary

The bytecode boundary cleanly separates the Compilation Layer from the Runtime Layer:

```
Compilation Layer (MOD-01..09)           Runtime Layer (MOD-10..11)
┌──────────────────────────────┐        ┌────────────────────────┐
│                              │        │                        │
│  Arabic Text                 │        │  GrammarBytecode       │
│      ↓                       │        │      ↓                 │
│  Unicode Validation          │        │  GVM Execution         │
│  Lexer                       │        │      ↓                 │
│  Tokenizer                   │        │  AnalysisResult        │
│  Morphology Parser           │        │      ↓                 │
│  Syntax Parser               │  ───►  │  Explanation Engine    │
│  GIR Constructor             │        │      ↓                 │
│  Rule Engine                 │        │  ExplanationOutput     │
│  Knowledge Graph Resolver    │        │                        │
│  Bytecode Generator          │        └────────────────────────┘
│      ↓                       │
│  Grammar Bytecode            │
└──────────────────────────────┘
```

This boundary provides:
- **Independent deployability.** The compilation pipeline can be updated without touching the GVM, and vice versa.
- **Caching granularity.** Bytecode is cached. If the same text is analyzed again, the compilation pipeline is skipped entirely.
- **Security isolation.** The GVM can run in a separate process or sandbox without access to the compilation pipeline's memory space.

### 5.2 Bytecode vs. GIR: A Comparison

| Property | GIR (JSON) | Grammar Bytecode | Advantage |
|----------|-----------|------------------|-----------|
| **Size (10-word sentence)** | ~50–200 KB | ~2–5 KB | Bytecode: 10–40× smaller |
| **Self-contained** | No (references KB IDs) | Yes (inline data) | Bytecode |
| **Execution guarantees** | None | Bounded steps, memory | Bytecode |
| **Language independent** | Yes (JSON) | Yes (custom binary) | Tie |
| **Optimization** | None | Multiple levels | Bytecode |
| **Human readable** | Yes | No (use GIR) | GIR |
| **Schema versioning** | JSON Schema | Binary version field | Tie |
| **Generation time** | 0 (already exists) | ~200 μs | GIR |

**Conclusion:** Use GIR for debugging, development, and intra-pipeline communication. Use bytecode for storage, transmission, execution, and cross-platform portability.

### 5.3 Comparison with Established Bytecode Formats

| Feature | JVM Bytecode | WebAssembly | AGOS Grammar Bytecode |
|---------|-------------|-------------|----------------------|
| **Purpose** | General computation | General computation | Grammatical analysis |
| **Instruction count** | ~200+ | ~30+ MVP | ~50 (estimated) |
| **Memory model** | Garbage-collected heap | Linear memory | Pre-allocated regions |
| **Type system** | Class-based | Value types | Feature-based |
| **Sandboxing** | Security manager | Capabilities | Step/memory bounds |
| **Format** | Binary class files | Binary WASM | Binary sections |
| **Versioning** | Major.minor | MVP+proposals | Major.minor.patch |

AGOS bytecode is **domain-specific** — it does not aim to be a general-purpose computation format. It is specialized for the domain of grammatical analysis, which allows:

- Smaller instruction set (~50 vs. ~200+ for JVM).
- Simpler memory model (pre-allocated regions vs. garbage collection).
- Domain-specific optimizations (feature bitfields, string interning).

---

## 6. Consequences

### 6.1 Positive Consequences

1. **Portable analysis format.** Grammar analyses can be stored, cached, transmitted, and executed anywhere a GVM implementation exists — from servers to mobile devices to browsers.

2. **Strong execution guarantees.** The GVM enforces bounded execution time, bounded memory usage, and side-effect-free execution. This prevents misbehaving rule sets or plugins from crashing or corrupting the host process.

3. **Language independence.** Applications in any language can consume AGOS analyses by implementing a GVM. No need to reimplement the compilation pipeline in each language.

4. **Optimization opportunities.** Compile-time optimizations (dead code elimination, instruction fusion, string interning) reduce bytecode size and execution time.

5. **Caching granularity.** Bytecode can be cached at a finer granularity than GIR. Pre-computed bytecode for common phrases (Quran, Hadith) can be distributed with the application.

6. **Versioning independence.** The bytecode format is versioned independently of the GIR format. A GIR schema change does not necessarily require a bytecode format change.

7. **Security boundary.** The GVM provides a sandboxed execution environment. Bytecode cannot access the host system's files, network, or processes.

### 6.2 Negative Consequences

1. **Design and maintenance cost.** A custom binary format requires design, documentation, parser implementation, and version migration tooling.

2. **Compilation overhead.** Every analysis requires a bytecode generation step (~200 μs per sentence). Mitigated by caching — the bytecode is generated once per unique input and reused.

3. **Two formats to maintain.** Both GIR and bytecode must be versioned, documented, and tested. Mitigated by the fact that bytecode is a compilation target of GIR — changes typically flow from GIR to bytecode, not the reverse.

4. **Debugging complexity.** Developers debugging analysis issues must understand both the GIR structure and the bytecode instruction set. Mitigated by the GVM's tracing mode and the bytecode disassembler.

5. **Binary format opacity.** Bytecode is not human-readable. Mitigated by the `agos bytecode disassemble` command that produces a human-readable instruction listing.

### 6.3 Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Bytecode format churns during early development | Start with version 0.x (experimental). Stabilize at 1.0 when the pipeline is mature. |
| GVM implementations drift across languages | Provide a conformance test suite (test bytecode + expected output). All GVM implementations must pass. |
| Compilation overhead impacts latency | Cache bytecode; generate in background; use optimization level 0 for latency-sensitive workloads. |
| Security vulnerabilities in GVM | Fuzz-test the GVM; keep the instruction set minimal; audit memory access patterns. |

---

## 7. Implementation Guidance

### 7.1 Recommended Order

1. **Define the instruction set** (RFC-0002). This is the most critical piece — all other implementation depends on the instruction set being stable.

2. **Implement BytecodeGenerator (MOD-09)** in Rust. Start with optimization level 0 (direct GIR→bytecode mapping with no optimizations). Add optimization levels 1 and 2 incrementally.

3. **Implement GVM (MOD-10)** in Rust. Start with an interpreter (simpler, easier to debug). Add a JIT compiler only if performance profiling shows it is needed.

4. **Write the conformance test suite.** A set of test bytecodes with known expected outputs. All GVM implementations must pass this suite.

5. **Implement GVM in other languages** (C, Python, JavaScript, Swift, Kotlin) based on demand.

### 7.2 Instruction Set Design Principles

- **Minimal.** Define the minimum instruction set needed to express grammatical analysis. Aim for ~50 instructions.
- **Verifiable.** Every instruction has a well-defined format, operand types, and effect on GVM state.
- **Safe.** No instruction can crash the GVM, access memory outside its allocated region, or produce non-deterministic output.
- **Extensible.** Reserve opcode space for future instructions. Use versioning to manage instruction set evolution.

### 7.3 GVM Implementation Requirements

1. **Deterministic execution.** The GVM MUST produce byte-for-byte identical output given the same bytecode and configuration.
2. **Step counting.** Every instruction increments a step counter. When `max_execution_steps` is reached, execution halts and returns a `GVM_STEPS_EXCEEDED` error.
3. **Memory bounds.** Memory access is bounds-checked. Out-of-bounds access returns a `GVM_MEMORY_EXCEEDED` error.
4. **No side effects.** The GVM MUST NOT perform file I/O, network access, system calls, or any operation that modifies external state.
5. **Tracing mode.** The GVM SHOULD support a tracing mode that logs every instruction executed, for debugging and educational use.

### 7.4 Bytecode Format Design Decisions

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| **Endianness** | Little-endian | Matches most modern hardware |
| **Integer encoding** | Varint for small values; fixed-size for offsets | Balances compactness and access speed |
| **String encoding** | UTF-8 with length prefix (varint) | Standard, efficient |
| **Sections** | Logical sections (metadata, tokens, morphology, syntax, rules, evidence, strings) | Enables lazy loading and streaming |
| **Magic bytes** | 0x41474F53 ("AGOS") | Identifies the format |
| **Checksum** | CRC32C per section | Integrity verification |
| **Version** | Major.minor.patch in header | Clear compatibility semantics |

---

## 8. Status

**Accepted.** This decision is binding on all AGOS architecture and implementation work.

This ADR supersedes no prior decision.

This ADR is referenced by:
- SPEC-0001: Platform Architecture (Chapters 3, 5, 6, 9)
- RFC-0002: Grammar Bytecode Format
- RFC-0003: Grammar Virtual Machine
- ADR-0003: Why Grammar IR (planned)

---

## Progress Summary

**ADR-0002: Why Grammar Bytecode**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Context | ✓ COMPLETE |
| Section 2 | Problem Statement | ✓ COMPLETE (4 limitations of direct output) |
| Section 3 | Decision | ✓ COMPLETE |
| Section 4 | Alternatives Considered | ✓ COMPLETE (6 alternatives analyzed) |
| Section 5 | Detailed Rationale | ✓ COMPLETE (5 reasons + comparisons) |
| Section 6 | Consequences | ✓ COMPLETE (7 positive, 5 negative) |
| Section 7 | Implementation Guidance | ✓ COMPLETE |
| Section 8 | Status | ✓ COMPLETE |

**Dependencies:** ADR-0001, SPEC-0001 (Chapters 3, 5, 6, 9).

**Recommended next document:** ADR-0003 — Why Grammar IR, or RFC-0002 — Grammar Bytecode Format.
