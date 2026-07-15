---
adr_id: ADR-0003
title: Why AGOS Introduces a Dedicated Grammar Intermediate Representation
version: 1.0.0
status: Accepted
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
decided: 2026-07-13
references:
  - ADR-0001: Compiler Architecture Rationale
  - ADR-0002: Why Grammar Bytecode
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline (MOD-06 GIRConstructor)
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-6, IR-7, IR-8)
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0401: Knowledge Graph Engine (planned)
  - RFC-0001: Grammar DSL (proposed)
supersedes: None
---

# ADR-0003: Why AGOS Introduces a Dedicated Grammar Intermediate Representation (GIR)

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

ADR-0001 established that AGOS uses a compiler architecture with a multi-stage pipeline. ADR-0002 established that the pipeline culminates in bytecode execution via the GVM. Between these two decisions lies a critical architectural question: **what intermediate representation should the pipeline use, and where should its boundaries be drawn?**

The pipeline currently defines **11 distinct intermediate representations** (IR-1 through IR-11), but three of them — IR-6, IR-7, and IR-8 — form a family collectively called the **Grammar Intermediate Representation (GIR)**:

| IR | Produced By | Content | Role |
|----|-------------|---------|------|
| **IR-6: GrammarIR** | MOD-06 GIRConstructor | Unified morphology + syntax + ambiguity forest | The canonical parsed state |
| **IR-7: AnnotatedGIR** | MOD-07 RuleEngine | IR-6 + rule applications + evidence trail + flags | The rule-annotated state |
| **IR-8: ResolvedGIR** | MOD-08 KnowledgeGraphResolver | IR-7 + resolved KB entries + semantic tags | The fully enriched state |

The GIR occupies the central position in the pipeline — it is the boundary between the **front-end stages** (MOD-01 through MOD-06: validation, lexing, tokenization, morphology, syntax, unification) and the **back-end stages** (MOD-07 through MOD-09: rule application, KB resolution, bytecode generation).

The decision to designate a formal, versioned, serializable intermediate representation at this boundary — rather than passing ad-hoc data structures between stages — requires its own architectural rationale.

The central question is: **why does AGOS need a dedicated, versioned intermediate representation that separates the front-end (morphology + syntax) from the back-end (rules + bytecode)?**

---

## 2. Problem Statement

### 2.1 The Naive Approach: Direct Stage-to-Stage Data Passing

In the simplest architecture, each pipeline stage would pass its output directly to the next stage using in-memory data structures. MorphologicalParser would construct an object and pass it to SyntaxParser, which would extend it and pass it to the RuleEngine, and so on. There would be no formal IR boundary — just a chain of function calls with shared data structures.

### 2.2 Why Direct Data Passing Is Insufficient

The naive approach fails on several critical requirements:

#### 2.2.1 No Canonical Representation

Without a formal GIR, there is no single, authoritative representation of "the grammatical state of the text at this point in the pipeline." Each stage defines its own output format. This means:

- **No cross-stage contracts.** If MOD-04 changes its output format, MOD-05 must be updated — but there is no schema to check against.
- **No caching boundary.** Without a canonical, serializable representation, output from MOD-06 cannot be cached independently of MOD-07. Re-running the rule engine requires re-running MOD-01 through MOD-06.
- **No inspection point.** Developers and tools cannot easily inspect the state of the analysis at the boundary between front-end and back-end.

#### 2.2.2 Tight Coupling Between Stages

When stages pass raw data structures, they become coupled to each other's internal representations:

- MOD-05 (SyntaxParser) must understand the specific data structures produced by MOD-04 (MorphologicalParser).
- MOD-07 (RuleEngine) must understand both MOD-04's and MOD-05's data structures.
- A change to MOD-04's output format cascades through MOD-05, MOD-06, MOD-07, MOD-08, and MOD-09.

This coupling makes the pipeline brittle. A single change ripples through the entire system, requiring coordinated updates across all dependent stages.

#### 2.2.3 Ambiguity Is Not First-Class

Morphological and syntactic ambiguity is a fundamental property of Arabic grammar. Without a formal IR, ambiguity must be represented as an ad-hoc collection of alternative outputs — lists of possibilities that grow combinatorially as they pass through stages:

- MOD-03 produces up to 16 segmentations per token.
- MOD-04 produces up to 32 morphological analyses per stem.
- MOD-05 produces up to 8 parse trees per sentence.

Without a formal GIR that explicitly models the ambiguity forest as a first-class structure, the combinatorial explosion of (16 × 32 × 8 = 4,096+) paths becomes unmanageable. Each stage must manually track which combinations are valid, which have been pruned, and why.

#### 2.2.4 No Multi-School Foundation

The GIR must be school-agnostic because it is the input to the Rule Engine, which applies school-specific rules. If the GIR were school-specific, the entire front-end (MOD-01 through MOD-06) would need to run once per school, multiplying computational cost by the number of schools.

Without a school-agnostic GIR, supporting N grammar schools requires running the first 6 stages N times — one per school.

#### 2.2.5 No Serialization for Distributed Execution

The pipeline stages may run on different machines in a distributed deployment (see SPEC-0001 Chapter 6 — Deployment Topologies). Direct data passing through function calls only works for in-process pipelines. A serialized intermediate representation is required for:

- **Remote stage execution:** MOD-06 runs on server A, MOD-07 runs on server B.
- **Pipeline debugging:** Inspect GIR at each stage boundary without running the downstream stages.
- **Hydrated analysis inspection:** Send a GIR to a colleague or a debugging tool for analysis.
- **Regression testing:** Store GIR snapshots and compare across pipeline versions.

---

## 3. Decision

**AGOS SHALL define a formal, versioned, serializable Grammar Intermediate Representation (GIR)** as the canonical boundary between the front-end stages (MOD-01 through MOD-06: validation through unification) and the back-end stages (MOD-07 through MOD-09: rule application through bytecode generation).

```diff
- Naive Pipeline:
  MOD-04 (in-memory structs) → MOD-05 (in-memory structs) → MOD-06 → MOD-07 → MOD-08 → MOD-09

+ GIR-Based Pipeline:
  MOD-04 → MOD-05 → MOD-06 → [GIR: versioned, serializable IR-6]
                                ↓
                            MOD-07 → [AnnotatedGIR: versioned, serializable IR-7]
                                      ↓
                                  MOD-08 → [ResolvedGIR: versioned, serializable IR-8]
                                              ↓
                                          MOD-09 → GrammarBytecode
```

The GIR:

1. **Is versioned** with semantic versioning (major.minor.patch). A GIR consumer checks version compatibility before processing.

2. **Is serializable** to JSON (for debugging and development) and binary CBOR/Protobuf (for production and distributed execution).

3. **Explicitly models ambiguity** as a first-class forest structure, not as ad-hoc alternatives.

4. **Is school-agnostic.** The GIR represents the grammatical state of the text without reference to any specific grammar school. School-specific annotations are added by the Rule Engine (MOD-07) and recorded in the AnnotatedGIR (IR-7).

5. **Has three variants** that represent the state of analysis at progressive pipeline stages:
   - IR-6 (GrammarIR): the raw unified state, before rule application.
   - IR-7 (AnnotatedGIR): after rule application, with evidence trail.
   - IR-8 (ResolvedGIR): after knowledge graph resolution, with KB enrichment.

6. **Is the caching boundary.** Output from MOD-06 (the GIR) can be cached independently of the rule engine and bytecode generation stages. If the same text is re-analyzed with a different rule set, only MOD-07 through MOD-09 must re-run.

7. **Is the distributed execution boundary.** Stages before and after the GIR can run on different machines or in different processes, communicating through serialized GIR.

---

## 4. Alternatives Considered

### 4.1 Alternative A: No Formal IR (Ad-Hoc Data Passing)

**Description:** Each stage defines its own output format. Stages pass in-memory data structures directly. No versioned, serializable intermediate representation.

**Advantages:**
- Simplest implementation — no IR schema to design, version, or maintain.
- Lowest latency (no serialization/deserialization overhead).
- No IR format learning curve for developers.

**Disadvantages (why rejected):**
- **No cross-stage contracts.** There is no schema to validate stage outputs against. A change in one stage's output format silently breaks downstream stages, caught only by runtime errors or test failures.
- **No caching boundary.** Without a serializable IR, the output of MOD-06 cannot be cached. Re-analyzing the same text with a different rule set requires re-running MOD-01 through MOD-06.
- **No distributed execution.** Stages cannot run on different machines because there is no serialization format for communication.
- **No debugging/inspection point.** Developers cannot easily inspect the unified grammatical state at the front-end/back-end boundary.
- **Ambiguity representation is ad-hoc.** Each stage manually tracks ambiguity using its own conventions. There is no guarantee that MOD-04's ambiguity representation is compatible with MOD-07's expectations.
- **School coupling.** The front-end stages must produce school-specific output if the back-end requires it, meaning the front-end must run N times for N schools.

**Verdict: REJECTED.** The lack of contracts, caching, serialization, and formal ambiguity representation makes this approach unsuitable for a platform that must support multiple schools, distributed deployment, and incremental development.

### 4.2 Alternative B: Use the Existing TokenStream as the Universal IR

**Description:** Rather than defining a new GIR, extend the existing IR-2 (TokenStream) or IR-3 (SegmentedTokenStream) to carry all information needed by downstream stages — morphological features, syntactic roles, rule applications, and KB data. The token stream becomes the universal IR.

**Advantages:**
- Reuses an existing IR (TokenStream) that is already defined and implemented.
- Minimal additional IR design work.
- The token stream is a natural organizing structure (one entry per token).

**Disadvantages (why rejected):**
- **Token-centric, not syntax-centric.** A token stream is a flat list. Grammatical analysis requires hierarchical structure (syntax trees, constituent relationships, idafa constructions). Fitting hierarchical syntax into a flat token stream requires awkward encoding (parent pointers, role annotations on tokens, etc.).
- **No ambiguity forest representation.** Token streams do not naturally model cross-token ambiguity combinations (morphology × syntax). The ambiguity forest requires a higher-level structure that spans tokens.
- **No evidence trail attachment point.** The evidence trail is attached to rule applications and analyses, not to individual tokens. A token stream cannot naturally group evidence by rule, tree, or analysis path.
- **No tree-level metadata.** Properties like sentence type (jumlah fi'liyyah vs. ismiyyah), confidence score per parse tree, and source school apply to the entire tree, not to individual tokens.
- **Extensibility burden.** Every new piece of analysis data (semantic tags, etymology, cross-references) must be shoehorned into token annotations, bloating the token structure with fields irrelevant to token-level concerns.

**Verdict: REJECTED.** The token stream is the right representation for IR-2 and IR-3, but it is fundamentally inadequate for the unified, hierarchical, ambiguity-aware representation required by the GIR.

### 4.3 Alternative C: Use the SyntaxTree as the Universal IR

**Description:** Rather than defining a new GIR, extend the existing IR-5 (SyntaxTree) to carry morphological features on each node. The syntax tree becomes the universal IR — every token's morphology is embedded in the tree node that represents it.

**Advantages:**
- Reuses the existing SyntaxTree structure.
- Hierarchical by nature — naturally represents constituent relationships.
- Tree nodes can carry feature annotations.

**Disadvantages (why rejected):**
- **Tree-centric, not ambiguity-aware.** A syntax tree is a single parse. The GIR must represent an ambiguity *forest* — multiple possible trees, each with different morphological analyses underlying them. Extending a single tree to represent a forest requires a forest-of-trees structure, which is essentially a new representation anyway.
- **No independent token list.** By embedding morphology in tree nodes, the morphology is tied to a specific parse. When there are multiple parses, the same token's morphology must be duplicated in each tree node — wasting space and creating manual synchronization requirements.
- **No cross-tree evidence grouping.** Evidence entries span morphology, syntax, rules, and KB resolution. Attaching them to tree nodes scatters evidence across trees, making it difficult to present a unified evidence trail.
- **Token-to-tree mapping complexity.** The SyntaxParser produces trees that reference token IDs. If the token list is embedded in the tree, token ordering and indexing must be reconstructed from tree traversal, which is fragile.

**Verdict: REJECTED.** The syntax tree is the right representation for IR-5, but extending it to serve as the universal GIR creates more problems than it solves. The GIR needs both a flat token list (for morphology) and a forest of trees (for syntax) — a hybrid structure that is neither pure token stream nor pure syntax tree.

### 4.4 Alternative D: Multiple Independent IRs (Status Quo Without a Dominant Boundary)

**Description:** Keep all IRs (IR-1 through IR-8) as independent, versioned representations with no designated "main" IR. Each stage-to-stage boundary is equally important. There is no central GIR — just a chain of IR transformations.

**Advantages:**
- All IRs are already defined and versioned independently.
- No need to designate one IR as "more important" than others.
- Maximum flexibility — any IR can be cached, inspected, or serialized.

**Disadvantages (why rejected):**
- **No operational boundary.** Without designating the GIR as the caching and distributed execution boundary, every stage boundary is *potential* — but none is *operationalized*. A distributed pipeline needs a clear handoff point. The GIR is the natural choice because it is the first IR that unifies all front-end analysis.
- **No architectural clarity.** Developers must understand all 8 IRs to understand where the front-end ends and the back-end begins. Designating the GIR as the boundary makes the architecture self-documenting.
- **Caching granularity without policy.** Multiple independent caching points require complex cache invalidation logic. Designating the GIR as the primary caching boundary simplifies the caching strategy: cache at GIR, re-run back-end when rules or KBs change.
- **School switching is unclear.** Without a designated GIR, it is ambiguous whether MOD-01 through MOD-05 should be re-run when switching schools, or whether only MOD-07 should be re-run. The GIR boundary makes this explicit: the GIR is school-agnostic; only back-end stages need to re-run.

**Verdict: REJECTED.** Multiple independent IRs are the right approach for *defining* the pipeline transformations, but a designated boundary IR (the GIR) is needed for *operationalizing* caching, distribution, and school switching.

### 4.5 Alternative E: Grammar Intermediate Representation (GIR) — Chosen

**Description:** Designate a formal, versioned, serializable GIR (comprising IR-6, IR-7, and IR-8) as the canonical boundary between front-end and back-end stages. The GIR explicitly models ambiguity forests, supports school-agnostic analysis, and serves as the caching and distributed execution boundary.

**Advantages:**
- See [Section 5 — Detailed Rationale](#5-detailed-rationale).

**Disadvantages (addressed):**
- **Design and maintenance cost.** The GIR schema must be designed, documented, versioned, and migrated. Mitigated by the fact that the GIR is the *only* new IR needed at this boundary — the front-end and back-end IRs (IR-1 through IR-5, IR-9 through IR-11) are already defined.
- **Serialization overhead.** JSON serialization of the GIR adds ~100 μs per sentence. Mitigated by using binary serialization (CBOR/Protobuf) in production and caching to eliminate repeated serialization of the same GIR.
- **Version migration effort.** When the GIR schema changes, all stages that produce or consume the GIR must be updated. Mitigated by backward-compatible schema evolution (additive fields only; no breaking changes without a major version bump and migration tooling).

**Verdict: ACCEPTED.** The GIR is the only approach that simultaneously satisfies all requirements: canonical representation, caching boundary, distributed execution, formal ambiguity management, school-agnosticism, and architectural clarity.

---

## 5. Detailed Rationale

### 5.1 Six Reasons for the GIR

#### Reason 1: Canonical Representation at the Architecture Boundary

The GIR is the first point in the pipeline where all analytical threads converge:

```diff
  MOD-01 (Unicode Validation)      │
  MOD-02 (Lexer)                    │  Front-End
  MOD-03 (Tokenizer)               │  (What the text IS)
  MOD-04 (Morphological Parser)    │
  MOD-05 (Syntax Parser)           │
  MOD-06 (GIR Constructor)         │
              ↓                     ▼
        [GIR Boundary]          ════════════════
              ↓                     ▲
  MOD-07 (Rule Engine)              │  Back-End
  MOD-08 (KG Resolver)              │  (What the rules SAY)
  MOD-09 (Bytecode Generator)      │
```

Before the GIR, each stage produces partial, independent analyses:
- MOD-02 produces tokens (no morphology, no syntax).
- MOD-03 produces segmented tokens (still no morphology or syntax).
- MOD-04 produces morphological features (but no syntax).
- MOD-05 produces syntax trees (but the morphology is in a separate structure).

The GIR is the first representation that unifies all of these into a single, self-consistent whole. A developer or tool inspecting the GIR sees the complete grammatical state of the text at the front-end/back-end boundary — every token's morphology, every syntactic relationship, every ambiguity combination.

#### Reason 2: Caching Boundary

The GIR is the optimal caching boundary for several reasons:

| Cache Point | Cache Hit Saves | Typical Reanalysis Cost After KB/Rule Change |
|-------------|-----------------|----------------------------------------------|
| Before MOD-01 | Nothing | Full pipeline re-run |
| After MOD-03 | ~30 μs | Full pipeline minus lexing |
| After MOD-04 | ~500 μs | Full pipeline minus morphology |
| After MOD-05 | ~1.5 ms | Full pipeline minus syntax |
| **After MOD-06 (GIR)** | **~2 ms** | **Only rule engine + KG + bytecode** |
| After MOD-07 | ~2.5 ms | Only KG + bytecode |
| After MOD-08 | ~2.6 ms | Only bytecode |
| After MOD-09 | ~2.8 ms | Only GVM execution |

The GIR boundary provides the best **benefit-to-cost ratio** for caching:

- **Caching at MOD-06 saves ~70% of pipeline time** (the front-end stages), while requiring only one cache tier.
- **Caching earlier (before MOD-06) saves less** and requires more cache tiers with complex invalidation logic.
- **Caching later (after MOD-06) saves more per tier** but requires running the back-end (MOD-07 through MOD-09) redundantly for different school/KB configurations.

**Use case example:** A user analyzes the same Quranic verse with Basra rules, then switches to Kufa rules. With GIR caching:

```
First analysis (Basra):
  1. Run MOD-01 through MOD-06: ~2 ms
  2. Cache GIR
  3. Run MOD-07 through MOD-09 (Basra): ~1 ms
  4. Run GVM: ~1 ms
  Total: ~4 ms

Second analysis (Kufa) — CACHE HIT:
  1. Load cached GIR: <1 μs
  2. Run MOD-07 through MOD-09 (Kufa): ~1 ms
  3. Run GVM: ~1 ms
  Total: ~2 ms (50% reduction)
```

Without GIR caching, the second analysis would re-run MOD-01 through MOD-06, costing an additional ~2 ms.

#### Reason 3: Distributed Execution Boundary

The GIR is the natural handoff point for distributed pipeline execution:

```diff
  ┌──────────────────────┐     ┌──────────────────────┐
  │  Compilation Server   │     │  Analysis Server     │
  │  (MOD-01 through     │     │  (MOD-07 through     │
  │   MOD-06)            │     │   MOD-10)            │
  │                      │     │                      │
  │  Input: Arabic text   │     │  Input: GIR           │
  │  Output: GIR (IR-6)  │ ──► │  Output: Analysis     │
  └──────────────────────┘     └──────────────────────┘
```

In a distributed topology:

- **Compilation servers** handle the heavy, KB-dependent front-end stages. They can be scaled independently based on input volume.
- **Analysis servers** handle the rule engine and GVM execution. They can be scaled independently based on rule complexity and request volume.
- **The GIR** is the transfer format between them — serialized, versioned, self-contained (modulo KB references that are resolved later by IR-8).

This architecture enables:
- **Geographic distribution:** A compilation server in Cairo generates GIR; an analysis server in Jakarta executes it with local rule sets.
- **Language-specific processing:** Compilation servers run Rust (for performance); analysis servers can run Python (for research) or JavaScript (for browser).
- **Load balancing:** Compilation servers are CPU-bound (morphology, syntax); analysis servers are memory-bound (KB lookups, rule application). Each can be scaled independently.

#### Reason 4: Formal Ambiguity Management

Arabic text is inherently ambiguous. The GIR makes ambiguity a first-class concept with an explicit data model:

```diff
- Without GIR:
  Each stage produces a list of "maybe this, maybe that" alternatives.
  No stage knows how alternatives from different stages combine.
  The combinatorial explosion (morphology × syntax) is unmanaged.

+ With GIR:
  MOD-06 explicitly constructs an AmbiguityForest:
    - Each token has a set of morphological alternatives.
    - Each parse tree represents a valid (morphology × syntax) combination.
    - Paths through the forest are counted, ordered, and indexed.
  MOD-07 prunes the forest by applying rules:
    - Rules confirm paths (keep), reject paths (remove), or modify paths.
    - The pruned forest explicitly shows which paths survived and why.
  MOD-08 enriches the surviving paths with KB data.
```

The GIR's ambiguity forest model provides:

1. **Explicit path counting.** The total number of valid parse paths is computed as `∏(token.alternatives) × syntax_trees`. This makes ambiguity measurable and comparable across inputs.

2. **Deterministic pruning.** Rules prune the forest by rejection — they remove invalid paths. The remaining paths are tracked with their confidence scores and supporting evidence.

3. **Transparency.** Ambiguity is never silently discarded. Every path that was considered is either retained in the forest or recorded in the evidence trail with a reason for rejection.

4. **Educational value.** For AGOS's educational use case, the ambiguity forest can be presented to the user as "here are N possible analyses of this sentence, here is why each is possible, and here is why the primary analysis was chosen."

#### Reason 5: School-Agnostic Foundation

The GIR is school-agnostic by design. This is a critical architectural property:

```diff
  ┌──────────────────────────────────────────────────────────────┐
  │                    GIR (School-Agnostic)                     │
  │                                                              │
  │  Tokens: [masculine, singular, nominative, ...]              │
  │  Trees: [fi'l (index 0), fa'il (index 1), ...]             │
  │  Features: [gender, number, person, case, mood, ...]        │
  │  Ambiguity: [path_A, path_B, path_C]                       │
  │                                                              │
  │  NO reference to:                                            │
  │    - Any specific grammar school                             │
  │    - School-specific rule interpretations                    │
  │    - School-specific feature assignments                     │
  └──────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  Basra Rules  │  │  Kufa Rules  │  │  Andalus R.  │
    │  (MOD-07)     │  │  (MOD-07)    │  │  (MOD-07)    │
    └──────┬───────┘  └──────┬───────┘  └──────┬───────┘
           ▼                 ▼                 ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │AnnotatedGIR  │  │AnnotatedGIR  │  │AnnotatedGIR  │
    │(Basra)       │  │(Kufa)        │  │(Andalus)     │
    └──────────────┘  └──────────────┘  └──────────────┘
```

This architecture has several advantages:

1. **Single front-end pass.** The front-end stages run once, regardless of the number of schools. Only the back-end (rule engine, MOD-07) runs per school.

2. **Fair comparison.** Because the GIR is school-agnostic, analyses from different schools can be compared directly — they start from the same baseline (the GIR) and diverge only in rule application.

3. **Plugin-friendly.** A new grammar school plugin only needs to provide a rule set for MOD-07. It does not need to reimplement morphology, syntax, or any front-end stage.

4. **Future-proof.** A new grammar school discovered or reconstructed from historical texts can be added as a rule set plugin without modifying any front-end code. The GIR serves as the invariant interface between the universal front-end and the school-specific back-end.

#### Reason 6: Architectural Clarity

Designating the GIR as the central boundary provides architectural clarity that benefits developers, operators, and users:

| Stakeholder | What the GIR Boundary Clarifies |
|-------------|---------------------------------|
| **Front-end developer** | "My job is to produce a correct GIR from Arabic text. I don't need to know about rules or schools." |
| **Back-end developer** | "My job is to consume a GIR and produce annotated/resolved bytecode. I don't need to know about lexing or morphology." |
| **Linguist / rule author** | "I write rules that operate on the GIR. The GIR is my API — I don't need to understand how the text was parsed." |
| **Operator** | "I cache the GIR to avoid re-running the front-end. I can scale compilation servers and analysis servers independently." |
| **Tooling developer** | "I can inspect the GIR at the pipeline boundary. The GIR schema is my contract for building debuggers, visualizers, and editors." |

### 5.2 GIR vs. Bytecode: Different Purposes

The GIR and Grammar Bytecode (RFC-0002) serve different purposes and are not redundant:

| Property | GIR (IR-6, IR-7, IR-8) | Grammar Bytecode (IR-9) |
|----------|------------------------|------------------------|
| **Purpose** | Analysis representation | Execution plan |
| **Audience** | Stages, developers, tools | GVM (Machine) |
| **Format** | JSON/Binary (human-readable option) | Binary only (optimized for execution) |
| **Granularity** | Feature-level (morphological, syntactic) | Instruction-level (opcodes, operands) |
| **Ambiguity** | Explicit forest | Implicit (resolved during generation) |
| **Self-contained** | No (references KBs by ID) | Yes (all data inlined) |
| **Caching value** | Caches front-end analysis | Caches execution plan |
| **Versioning** | GIR schema version | Bytecode format version |

The GIR answers: "What is the grammatical state of this text?" Bytecode answers: "What instructions should the GVM execute to verify and explain this analysis?" They are complementary: bytecode is generated *from* the GIR (by MOD-09), and the GIR is the human-inspectable source of truth.

### 5.3 Comparison with Compiler IRs

| Property | LLVM IR | GCC GIMPLE | JVM Bytecode | AGOS GIR |
|----------|---------|------------|--------------|----------|
| **Source** | C/C++/etc. | C/C++/etc. | Java | Arabic text |
| **Form** | SSA form | Three-address code | Bytecode stack | Structured JSON/binary |
| **Purpose** | Optimization target | Middle-end IR | Execution | Unified analysis state |
| **S-dependence** | Language-independent | Language-independent | Lang-independent | School-agnostic |
| **Optimization** | ~200 passes | ~150 passes | JIT compilation | Rule application + KB resolution |
| **Lifetime** | Pipeline middle | Pipeline middle | Execution target | Front-end/back-end boundary |

The GIR fills the same architectural role as LLVM IR — it is the *lingua franca* that enables independent development of front-end and back-end stages. Just as LLVM IR enables any language to target any architecture, the GIR enables any Arabic text to be analyzed by any grammar school.

---

## 6. Consequences

### 6.1 Positive Consequences

1. **Caching boundary.** The GIR is the optimal point for caching front-end analysis results. Repeated analysis of the same text with different rule sets saves ~70% of pipeline execution time.

2. **Distributed execution.** The GIR serialization enables compilation servers and analysis servers to run independently, in different locations, and at different scales.

3. **School-agnostic analysis.** A single front-end pass serves all grammar schools. New schools can be added without modifying front-end stages.

4. **Formal ambiguity management.** Ambiguity is a first-class concept with explicit modeling, computation, and pruning. No stage silently discards alternatives.

5. **Architectural clarity.** The GIR boundary cleanly separates front-end concerns (what the text is) from back-end concerns (what the rules say). Developers, linguists, and operators have clear contracts.

6. **Inspectability.** The GIR can be serialized to JSON for debugging, visualization, and educational tools. It is the authoritative snapshot of the analysis at the front-end/back-end boundary.

7. **Versioning independence.** The GIR schema is versioned independently of the bytecode format and the pipeline stages. A GIR schema change does not necessarily require bytecode format changes, and vice versa.

8. **Plugin foundation.** The GIR is the input to rule set plugins (MOD-07) and KB resolver plugins (MOD-08). Plugin authors only need to understand the GIR schema, not the entire pipeline.

### 6.2 Negative Consequences

1. **Design and maintenance cost.** The GIR schema must be designed, documented, versioned, and migrated. Schema changes require coordinated updates across all stages that produce or consume the GIR.

2. **Serialization overhead.** JSON serialization of the GIR adds ~100 μs per sentence. Binary serialization reduces this to ~10 μs but adds complexity.

3. **Indirection.** Developers must understand the GIR schema in addition to the specific IRs (IR-1 through IR-5) that feed into it. This adds to the learning curve.

4. **Schema evolution effort.** Every GIR field is a contract between front-end and back-end stages. Adding, removing, or changing a field requires updating the schema definition, all producers, and all consumers.

5. **Tooling dependency.** The GIR is only as useful as the tools that support it — schema validators, serializers, deserializers, inspectors, and debuggers must be built and maintained.

### 6.3 Risk Mitigation

| Risk | Mitigation |
|------|------------|
| GIR schema churns during early development | Start with version 0.x (experimental). Add fields liberally. Stabilize at 1.0 when the pipeline is mature. |
| Serialization overhead impacts latency | Use binary serialization in production; JSON only for debugging. Cache GIR to avoid repeated serialization. |
| Schema evolution is costly | Design for extensibility from day one: optional fields, tag-union types, reserved field numbers (for Protobuf). |
| Developers ignore the GIR and couple stages directly | Enforce that MOD-06 always produces serialized GIR and MOD-07 always reads serialized GIR — even in-process. No direct data sharing allowed. |
| GIR becomes a dumping ground for every data point | Keep the GIR minimal. Include only data that is needed by downstream stages. Application-specific data should be added by plugins, not to the core GIR schema. |

---

## 7. Implementation Guidance

### 7.1 Recommended Implementation Order

1. **Define IR-6 (GrammarIR) schema.** Start with the minimum viable GIR: tokens with morphological features, syntax trees with constituent roles, and the ambiguity forest structure. Version 0.1.0.

2. **Implement MOD-06 (GIRConstructor).** This stage is the GIR producer. Implement it early to validate the GIR schema against real pipeline output.

3. **Define IR-7 (AnnotatedGIR) schema.** Extend IR-6 with rule application records, evidence trail entries, and flags. Version 0.1.0.

4. **Implement MOD-07 (RuleEngine).** The rule engine is the GIR consumer. Implement it against the IR-6/IR-7 schemas to validate that the GIR contains everything rules need.

5. **Define IR-8 (ResolvedGIR) schema.** Extend IR-7 with KB entry references, semantic tags, and resolution metadata. Version 0.1.0.

6. **Implement MOD-08 (KnowledgeGraphResolver).** This stage enriches the GIR with KB data. It is the last GIR consumer before bytecode generation.

7. **Build GIR tooling.** Schema validator, JSON serializer/deserializer, binary serializer/deserializer, GIR inspector CLI.

8. **Integrate caching at GIR boundary.** Implement the CacheManager (MOD-13) to cache IR-6 output. Verify cache invalidation works with KB version changes.

9. **Stabilize schema.** After the pipeline is working end-to-end, stabilize the GIR schema at version 1.0. Document all fields, invariants, and compatibility rules.

### 7.2 GIR Schema Design Principles

1. **Minimal but sufficient.** A field belongs in the GIR if (a) it is produced by a front-end stage and consumed by a back-end stage, or (b) it is needed for inspection/debugging at the pipeline boundary. If a field is only used within a single stage, it does not belong in the GIR.

2. **Forward-compatible.** New fields MUST be additive. Renaming, removing, or changing the type of an existing field is a breaking change that requires a major version bump.

3. **Explicit over implicit.** All relationships between tokens, morphology, and syntax are explicitly represented. No relationships are derived from convention or position.

4. **Self-describing.** The GIR metadata includes the GIR version, the pipeline version that produced it, and the KB versions used. This enables consumers to detect and handle version mismatches.

5. **Independent of serialization format.** The GIR schema is defined in JSON Schema (or Protobuf IDL), not in any implementation language. The schema is the canonical definition; language-specific types are generated from it.

### 7.3 GIR as a Service (Advanced)

In advanced deployments, the GIR boundary can be operationalized as a service boundary:

```diff
  ┌──────────────────────┐     ┌──────────────────────┐
  │  GIR Service          │     │  Analysis Service    │
  │                      │     │                      │
  │  POST /v1/gir         │     │  POST /v1/analyze    │
  │  Input: Arabic text   │     │  Input: GIR blob    │
  │  Output: GIR (IR-6)  │ ──► │  Output: Analysis    │
  │                      │     │                      │
  │  GET /v1/gir/{hash}  │     │  GET /v1/cached      │
  │  Output: cached GIR  │     │  Output: cached      │
  └──────────────────────┘     │  analysis            │
                               └──────────────────────┘
```

This is an extension for microservice deployments, not required for the core pipeline.

### 7.4 Validation Strategy

1. **Schema conformance.** Every GIR produced by MOD-06 MUST validate against the IR-6 JSON Schema. Validation is built into the stage output (fail fast on invalid GIR).

2. **Round-trip equality.** Serialize a GIR to JSON, deserialize it, and verify that the deserialized GIR is structurally identical to the original. This tests both serializer and deserializer.

3. **Forward compatibility.** Test that a GIR produced by an older schema version can be consumed by a newer stage (and vice versa, within compatibility guarantees).

4. **Boundary testing.** Verify that the GIR carries all information needed by consumering stages: MOD-07 must be able to apply rules using only the GIR; MOD-08 must be able to resolve KB references using only the GIR.

5. **Regression GIR snapshots.** Store a corpus of GIR snapshots with known analyses. Verify that pipeline changes do not produce regression in GIR structure or content for known inputs.

---

## 8. Status

**Accepted.** This decision is binding on all AGOS architecture and implementation work.

This ADR supersedes no prior decision.

This ADR is referenced by:
- SPEC-0001: Platform Architecture (Chapters 2, 3, 5, 7)
- SPEC-0201: Rule Engine (planned)
- SPEC-0401: Knowledge Graph Engine (planned)
- RFC-0001: Grammar DSL (proposed)

---

## Progress Summary

**ADR-0003: Why Grammar Intermediate Representation (GIR)**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Context | ✓ COMPLETE |
| Section 2 | Problem Statement | ✓ COMPLETE (5 limitations of direct data passing) |
| Section 3 | Decision | ✓ COMPLETE (7 properties of the GIR) |
| Section 4 | Alternatives Considered | ✓ COMPLETE (5 alternatives analyzed) |
| Section 5 | Detailed Rationale | ✓ COMPLETE (6 reasons + comparisons) |
| Section 6 | Consequences | ✓ COMPLETE (8 positive, 5 negative) |
| Section 7 | Implementation Guidance | ✓ COMPLETE (9-step order, 5 design principles) |
| Section 8 | Status | ✓ COMPLETE |

**Dependencies:** ADR-0001, ADR-0002, SPEC-0001 (Chapters 2, 3, 5, 7).

**Recommended next document:** RFC-0001 (Grammar DSL) — the domain-specific language for authoring grammatical rules, or continue with KB-0001 (Roots) — the first knowledge base specification.
