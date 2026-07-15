# AGOS Implementation Task Plan

## Overview

This plan breaks down the AGOS implementation into **8 phases** with **38 tasks**, ordered to produce a working end-to-end minimal path as early as possible. Each phase builds on the previous one.

**Total estimated effort:** ~6-9 months (single full-time engineer)

**Primary language:** Rust (see `docs/TECH-STACK-RECOMMENDATION.md`)

**Spec references:** Each task links to the relevant spec(s).

## Phase 0: Project Scaffolding (Week 1)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 0.1 | Initialize Rust workspace with crate structure | TECH-STACK-RECOMMENDATION.md | `Cargo.toml` workspace with 16 empty crate stubs |
| 0.2 | Set up CI (GitHub Actions) | SPEC-0001-C8 §6.2.1 | CI with `cargo test`, `cargo clippy`, `cargo fmt`, `cargo audit` |
| 0.3 | Define shared types crate (`agos-core`) | SPEC-0001-C5 (IR-1..IR-11), SPEC-0001-C8 (error codes) | `Token`, `MorphologicalAnalysis`, `GIR`, `Bytecode`, `Explanation`, `Error` types, `Result<T>` |
| 0.4 | Define error types and error codes (~50+) | SPEC-0001-C8 §3 | `ErrorKind` enum with all error codes, `Display + Debug` impl, error chain |
| 0.5 | Define pipeline trait (`PipelineStage<I, O>`) | SPEC-0001-C4 | `trait PipelineStage` with `fn process(&self, input: I) -> Result<O>` |
| 0.6 | Set up logging/tracing infrastructure | SPEC-0001-C6 §4 | `tracing` subscriber, structured JSON logging, span per stage |

## Phase 1: Knowledge Base Compiler + Data (Weeks 2-6)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 1.1 | KB compiler CLI (`agos-kb-compile`) | KB-OVERVIEW, KB-0001..KB-0008 | CLI that reads YAML source → compiled binary |
| 1.2 | KB-0001: Roots database schema + compile | KB-0001 | YAML spec → binary trie loader. ~15K-20K root entries |
| 1.3 | KB-0002: Wazan database schema + compile | KB-0002 | YAML spec → hash index loader. ~300-450 wazan |
| 1.4 | KB-0003: Verb forms table + compile | KB-0003 | YAML spec → table binary. ~180-250 paradigms |
| 1.5 | KB-0004: Noun patterns table + compile | KB-0004 | YAML spec → table binary. ~135-180 patterns |
| 1.6 | KB-0005: Particles database + compile | KB-0005 | YAML spec → hash index. ~120-200 particles |
| 1.7 | KB-0006: Pronouns database + compile | KB-0006 | YAML spec → hash index. ~60-80 pronouns |
| 1.8 | KB-0007: Feature bitfield definitions | KB-0007, SPEC-0102 | 64-bit bitfield layout, serde support, validation |
| 1.9 | KB runtime loader (`agos-kb` crate) | SPEC-0401, KB-OVERVIEW | `mmap`-based KB loader, trait `KbReader`, per-KB accessor structs |
| 1.10 | KB conformance tests | KB-0008 §8 | Verify all binary formats round-trip correctly, benchmark lookups meet budget |

## Phase 2: Pipeline Frontend — Text Input to GIR (Weeks 7-10)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 2.1 | MOD-01: UnicodeValidator | SPEC-0001-C3, SPEC-0001-C8 | Validate Arabic Unicode block range, normalize (NFKC), detect invalid sequences |
| 2.2 | MOD-02: Lexer | SPEC-0001-C3 | Segment input into word boundaries (whitespace, punctuation handling) |
| 2.3 | MOD-03: Tokenizer | SPEC-0001-C3, SPEC-0101 | Tokenize each word: identify particles (KB-0005), pronouns (KB-0006), potential roots |
| 2.4 | MOD-04: MorphologicalParser (core) | SPEC-0101 §3-4 | Root extraction (trie lookup in KB-0001), wazan identification (KB-0002), feature extraction (KB-0007) |
| 2.5 | MOD-04: MorphologicalParser (advanced) | SPEC-0101 §5-7 | Weak root handling, geminate handling, hamza variants, school-specific behavior hooks |
| 2.6 | MOD-05: SyntaxParser | SPEC-0101 §8 | POS disambiguation, phrase boundary detection using KB-0005/0006/0007 |
| 2.7 | MOD-06: GIRConstructor | SPEC-0001-C5 (IR-6) | Construct Grammar Intermediate Representation from morphological + syntactic analysis |
| 2.8 | Early integration test: Text → GIR | — | Pipe a few Arabic sentences through MOD-01..MOD-06, verify GIR output |

## Phase 3: Rule Engine + DSL Compiler (Weeks 11-14)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 3.1 | Grammar DSL lexer + parser (RFC-0001) | RFC-0001 §3-5 | Parse `.agosrule` files into AST: rule structure, conditions, actions |
| 3.2 | DSL semantic analysis + validation | RFC-0001 §6-7 | Type-checking, reference resolution, `matches` operator validation |
| 3.3 | DSL standard library (built-in predicates) | RFC-0004 §6 | Implement built-in functions: `is_nominative`, `is_definite`, `has_feature`, etc. |
| 3.4 | DSL → Bytecode compiler | RFC-0001 §8, RFC-0002 | Compile rule AST → Grammar Bytecode (.agos format) |
| 3.5 | MOD-07: RuleEngine | SPEC-0201 | Load compiled rule sets, apply rules to GIR, produce annotated GIR with evidence |
| 3.6 | MOD-08: KnowledgeGraphResolver | SPEC-0401 | Entity resolution, link analysis, cross-KB queries on annotated GIR |
| 3.7 | School-specific rule sets (Basra, Kufa) | RFC-0004 §4 | Write initial rule sets in Grammar DSL for Basra and Kufa schools |
| 3.8 | Rule engine test suite | RFC-0004 §9, SPEC-0201 §7 | JSON fixture-based tests for each rule with expected evidence output |

## Phase 4: Grammar Bytecode + Bytecode Generator (Weeks 15-16)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 4.1 | Bytecode binary format (de)serialization | RFC-0002 §3-15 | Encode/decode: magic bytes, header, section table, string table, instruction encoding |
| 4.2 | LEB128 varint + CRC32C support | RFC-0002 §4 | Variable-length integer encoding, section checksums |
| 4.3 | Feature bitfield section encoding | RFC-0002 §9.2, KB-0007 | 64-bit feature bitfield serialization in bytecode |
| 4.4 | MOD-09: BytecodeGenerator | RFC-0002 | Compile GIR + rule engine output → bytecode (.agos) |
| 4.5 | Bytecode conformance tests (~275) | RFC-0002 §18 | Verify all instructions round-trip, all sections encode/decode correctly |

## Phase 5: Grammar Virtual Machine (Weeks 17-22)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 5.1 | GVM core: register file + 2 stacks | SPEC-0304, RFC-0003 §4-5 | Value stack + call stack, 64-bit registers, operand stack |
| 5.2 | GVM memory model (7 regions) | SPEC-0304 §3-5 | Token region, feature region, constituent region, string region, rule region, evidence region, scratch region + bump allocator |
| 5.3 | GVM instruction dispatch (~50 opcodes) | SPEC-0302, RFC-0003 §6-8 | Decode + execute all instructions across 9 categories |
| 5.4 | GVM verification (12 checks) | SPEC-0301 §4 | Bytecode validation: stack balance, type checking, bounds checking, etc. |
| 5.5 | GVM tracing + step counting | SPEC-0301 §6 | Instruction tracing, step counter, debug output, budget enforcement |
| 5.6 | GVM bytecode loader | RFC-0002, SPEC-0301 | Load `.agos` bytecode, verify sections, prepare for execution |
| 5.7 | GVM conformance tests (~163) | RFC-0003 §11 | Verify each opcode, each memory region, verification checks |
| 5.8 | Memory model conformance tests (~215) | SPEC-0304 §9 | Budget calculation, allocation patterns, OOM handling, region isolation |

## Phase 6: Explanation Engine (Weeks 23-25)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 6.1 | MOD-11: Evidence-to-Explanation mapping | SPEC-0501 §3 | Transform GVM evidence trail into structured explanation data |
| 6.2 | Template system (Handlebars-based) | SPEC-0501 §4 | Template loading, rendering, Arabic-aware helpers |
| 6.3 | I'rab generation (5 breakdown types) | SPEC-0501 §5 | Generate Arabic grammatical breakdown: morphological, syntactic, full |
| 6.4 | Output formatters (JSON, Text, HTML) | SPEC-0501 §7 | Format explanations in multiple output formats |
| 6.5 | LLM integration provider interface | SPEC-0501 §6 | Optional LLM augmentation for explanation/tutoring (strictly non-grammatical) |
| 6.6 | Explanation engine test suite | SPEC-0501 | Verify template output, I'rab correctness, format fidelity |

## Phase 7: CLI + Server + Integration (Weeks 26-28)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 7.1 | CLI tool (`agos`) | SPEC-0001-C6 | `agos analyze`, `agos compile`, `agos run`, `agos kb-compile`, `agos plugin` commands |
| 7.2 | REST API server | SPEC-0001-C6 §1.2 | `agos-server` with analysis endpoint, plugin management, health check |
| 7.3 | MOD-13: CacheManager | SPEC-0001-C5, SPEC-0103 | LRU cache + Redis backend, cache key design (IR hashing) |
| 7.4 | MOD-14: APIGateway | SPEC-0001-C4, SPEC-0001-C6 | Request routing, rate limiting, API versioning |
| 7.5 | End-to-end integration tests | — | Full pipeline from Arabic text → explanation output, CI-gated |
| 7.6 | Benchmark suite | SPEC-0001-C9, SPEC-0103 | Per-stage latency budgets, throughput targets, regression tracking with `criterion` |

## Phase 8: Plugin System + Advanced (Weeks 29-34)

| # | Task | Spec Ref | Deliverable |
|---|------|----------|-------------|
| 8.1 | MOD-12: PluginLoader core | SPEC-0601 §3-4 | Discover, load, verify plugins via WASM |
| 8.2 | WASM sandbox (wasmtime integration) | SPEC-0601 §6, ADR-0005 | WASM runtime with capability-based security, resource limits |
| 8.3 | Plugin manifest + signing | SPEC-0601 §5 | Manifest format (YAML), ed25519 signing, SHA-256 verification |
| 8.4 | Plugin registry (SQLite) | SPEC-0601 §4.4 | Local plugin database: install, update, remove, list |
| 8.5 | Plugin SDK crate (`agos-plugin-sdk`) | SPEC-0601 §7 | Rust crate with derive macros, host function bindings, MessagePack serialization |
| 8.6 | School-specific rule packages | RFC-0004, SPEC-0601 | Package Basra/Kufa/Baghdad/Andalus/Modern rule sets as distributable plugins |
| 8.7 | Performance optimization pass | SPEC-0103, SPEC-0001-C9 | Hot path profiling, fast-path optimization, known words index, KB caching |
| 8.8 | Fuzz testing + security audit | SPEC-0601 §6.5, SPEC-0001-C8 | WASM parsing fuzz, input validation fuzz, dependency audit |

## Dependencies Between Phases

```
Phase 0 (Scaffolding)
    │
    ▼
Phase 1 (KB Data) ──────────────────────────────────────────┐
    │                                                       │
    ▼                                                       │
Phase 2 (Text → GIR)                                       │
    │                                                       │
    ▼                                                       │
Phase 3 (Rules + DSL) ─────┐                               │
    │                       │                               │
    ▼                       ▼                               │
Phase 4 (Bytecode) ◄────────┘                               │
    │                                                       │
    ▼                                                       │
Phase 5 (GVM) ◄─────────────────────────────────────────────┘
    │
    ▼
Phase 6 (Explanation)
    │
    ▼
Phase 7 (CLI + Server)
    │
    ▼
Phase 8 (Plugins + Optimization)
```

Phases 1 (KB) and 3 (Rule Engine) can partially overlap: KB-0005 (particles) and KB-0006 (pronouns) are needed early for MOD-03 (Tokenizer), but KB-0001/0002/0003/0004 (roots/wazan/verbs/nouns) are needed later for MOD-04/05.

## Key Milestones

| Milestone | Phase | What Works End-to-End |
|-----------|-------|----------------------|
| **M1: KB Foundation** | P1 | KB compiler produces binary trie/hash/table from YAML |
| **M2: Text → GIR** | P2 | Arabic sentence → GIR (cached KB data) |
| **M3: Rule Application** | P3 | GIR → annotated GIR with evidence (Basra school) |
| **M4: Bytecode Working** | P4 | GIR → bytecode binary → load + verify |
| **M5: GVM Working** | P5 | Bytecode → GVM execution → evidence trail |
| **M6: Full Pipeline Alpha** | P6-P7 | Arabic text → CLI output: JSON + HTML explanation |
| **M7: Production Ready** | P8 | WASM plugins, performance budgets met, fuzz-tested |

## Spec Documents Created Per Phase

Each implementation phase should also produce/update:
- **ADR** decisions encountered during implementation
- **KB YAML source data** (the actual Arabic linguistic entries)
- **Conformance test specs** (JSON fixtures for rule tests, GVM tests)
- **Implementation notes** in `docs/`
