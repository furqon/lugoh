# AGOS Implementation Plan — Step by Step

**Status:** In Progress | **Updated:** 2026-07-21

This plan breaks the AGOS implementation into **actionable steps** organized into **8 phases** with **~38 tasks**. Each step specifies exactly which files to create/modify, which specs to follow, and the deliverable.

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ DONE | Completed and compiling |
| 🔧 IN PROGRESS | Being worked on |
| ⬜ PENDING | Not yet started |

---

## Phase 0: Project Scaffolding

### 0.1 ✅ Initialize Rust workspace with crate structure

**Delivered ✅** — Root `Cargo.toml` workspace with `agos-core` and `agos-kb`.

**Remaining crate stubs to create (14 more):**
- `agos-lexer` — MOD-02: Lexer
- `agos-tokenizer` — MOD-03: Tokenizer  
- `agos-morph` — MOD-04/05: MorphologicalParser + SyntaxParser
- `agos-gir` — MOD-06: GIRConstructor
- `agos-rule-engine` — MOD-07: RuleEngine + DSL compiler
- `agos-bytecode` — MOD-09: BytecodeGenerator
- `agos-gvm` — MOD-10: Grammar Virtual Machine
- `agos-explanation` — MOD-11: ExplanationEngine
- `agos-plugin` — MOD-12: PluginLoader
- `agos-cache` — MOD-13: CacheManager
- `agos-api` — MOD-14: APIGateway
- `agos-cli` — CLI binary
- `agos-server` — Server binary
- `agos-kb-compile` — KB compiler CLI

### 0.2 ⬜ Set up CI (GitHub Actions)

**What:** Create `.github/workflows/ci.yml` with:
- `cargo check` on push/PR
- `cargo test` on all crates
- `cargo clippy` with `-D warnings`
- `cargo fmt --check`
- `cargo audit` for dependency vulnerabilities

**Spec:** SPEC-0001-C8 §6.2.1
**Files to create:** `.github/workflows/ci.yml`

### 0.3 ✅ Define shared types crate (`agos-core`)

**Delivered ✅** — Covers:
- `error.rs` — PipelineError + 30+ error codes for all 14 modules
- `types.rs` — Common enums (POS, SyntacticRole, GrammarSchool, etc.)
- `version.rs` — SemVer, KnowledgeVersionMap, ModuleVersion
- `evidence.rs` — EvidenceEntry, EvidenceTrail (SPEC-0001-C5 §13)
- `feature.rs` — 64-bit FeatureBitfield (KB-0007 §10)
- `ir.rs` — All 11 Intermediate Representations (IR-1 through IR-11)

### 0.4 ✅ Define error types (~40 codes)

**Delivered ✅** — `PipelineError` struct + `codes` module with error constants for all 14 modules.

### 0.5 ⬜ Define pipeline trait (`PipelineStage`)

**What:** Create a generic pipeline stage trait that all modules implement.

```rust
/// A single stage in the AGOS compilation pipeline.
pub trait PipelineStage<I, O> {
    /// The unique identifier for this stage (e.g., "MOD-01").
    fn stage_id(&self) -> &'static str;

    /// Process the input and produce output, or return an error.
    fn process(&self, input: I) -> PipelineResult<O>;
}
```

**Spec:** SPEC-0001-C4 §1-2 — Interface Design Principles
**Files:** `agos-core/src/pipeline.rs` + add module to lib.rs

### 0.6 ⬜ Set up logging/tracing infrastructure

**What:** Add `tracing` crate dependency, create a tracing subscriber setup, structured JSON logging with spans per stage.

```rust
// agos-core/src/tracing.rs
pub fn init_tracing(level: LogLevel, json: bool) { ... }
```

**Spec:** SPEC-0001-C6 §4
**Dependencies:** `tracing`, `tracing-subscriber`
**Files:** `agos-core/src/tracing.rs`

---

## Phase 1: Knowledge Base Implementation

### 1.1 ⬜ KB compiler CLI (`agos-kb-compile`)

**What:** Create a standalone binary crate `agos-kb-compile` that:
1. Reads YAML source files from knowledge/ directory
2. Validates schemas and cross-references
3. Compiles to binary `.agos-kb` format
4. Outputs metadata and validation report

```bash
agos-kb-compile compile knowledge/ output/
agos-kb-compile verify output/kb-0001.agos-kb
```

**Spec:** KB-OVERVIEW, KB-0001..KB-0008
**Files:** `agos-kb-compile/Cargo.toml`, `agos-kb-compile/src/main.rs`
**Dependencies:** `serde_yaml`, `clap` (for CLI arg parsing)

### 1.2 ⬜ KB-0001: Roots data + compiler

**What:** 
1. Create YAML source files for Arabic roots organized by first radical (28 Arabic letters)
2. Implement binary trie compilation
3. Target: ~15K-20K root entries, lookup < 1μs

**Spec:** KB-0001
**Files:** `knowledge/KB-0001/{ayn,ba,ta,...}.yaml`, `agos-kb-compile/src/compile_roots.rs`

### 1.3 ⬜ KB-0002: Wazan data + compiler

**What:**
1. Create YAML source files for morphological patterns (verb forms I–XV + noun patterns)
2. Implement hash index compilation with pattern signature hashing
3. Target: ~300-450 patterns, lookup < 500ns

**Spec:** KB-0002
**Files:** `knowledge/KB-0002/{verb-forms,noun-patterns,weak-variants,...}.yaml`

### 1.4 ⬜ KB-0003: Verb forms data + compiler

**What:**
1. Create YAML source files for conjugation paradigms (13 slots × 5 moods × 15 forms)
2. Implement table binary compilation
3. Target: ~180-250 paradigms, lookup < 1μs

**Spec:** KB-0003
**Files:** `knowledge/KB-0003/{sound,weak,quadriliteral,...}.yaml`

### 1.5 ⬜ KB-0004: Noun patterns data + compiler

**What:**
1. Create YAML source files for derived noun patterns
2. Implement table binary compilation
3. Target: ~135-180 patterns, lookup < 2μs

**Spec:** KB-0004
**Files:** `knowledge/KB-0004/{masdars,participles,broken-plurals,...}.yaml`

### 1.6 ⬜ KB-0005: Particles data + compiler

**What:**
1. Create YAML source files for ~120-200 particles (13+ categories)
2. Implement hash index compilation with governance rules
3. Target: lookup < 500ns

**Spec:** KB-0005
**Files:** `knowledge/KB-0005/{prepositions,conjunctions,...}.yaml`

### 1.7 ⬜ KB-0006: Pronouns data + compiler

**What:**
1. Create YAML source files for ~60-80 pronouns (6 categories)
2. Implement hash index compilation
3. Target: lookup < 500ns

**Spec:** KB-0006
**Files:** `knowledge/KB-0006/{personal-attached,personal-detached,...}.yaml`

### 1.8 ✅ KB-0007: Feature bitfield definitions

**Delivered ✅** — `agos-core/src/feature.rs` defines:
- `FeatureBitfield` (64-bit) with all 19 features
- `FeatureDefinition`, `FeatureValue`, `AgreementRule`, `InferenceRule`, `FeatureConstraint`
- Bit positions matching KB-0007 §10

**Remaining:** Create YAML source data for the feature taxonomy itself.

### 1.9 ✅ KB runtime loader (`agos-kb` crate)

**Delivered ✅** — `agos-kb` crate with:
- `types.rs` — All KB entry types (RootEntry, WazanEntry, VerbParadigm, etc.)
- `traits.rs` — `KbLoader`, `KbReader`, `KbSuite` with `KbReader` impl
- `loader.rs` — `DefaultKbLoader` with JSON deserialization
- `error.rs` — `KbError` wrapper

**Remaining:** 
- Replace JSON deserialization with binary `.agos-kb` format parsing
- Add mmap-based loading
- Benchmark to verify lookup budgets

### 1.10 ⬜ KB conformance tests

**What:** For each KB, verify:
- Round-trip: YAML → compiled binary → loaded structs → same data
- Lookup performance meets budgets
- Cross-KB version compatibility checks

**Spec:** KB-0008 §8
**Files:** `agos-kb/tests/conformance.rs`

---

## Phase 2: Pipeline Frontend — Text to GIR

### 2.1 ⬜ MOD-01: UnicodeValidator

**What:** Create `agos-unicode` (or add to `agos-morph`) with:
- Validate Arabic Unicode block ranges (U+0600–U+06FF, U+0750–U+077F, U+08A0–U+08FF)
- NFKC normalization
- Tatweel stripping (optional)
- Tashkeel normalization (optional)
- Non-Arabic character handling
- Max input size enforcement

```rust
pub struct UnicodeValidator;
impl PipelineStage<UnicodeValidatorInput, NormalizedText> for UnicodeValidator { ... }
```

**Spec:** SPEC-0001-C3 §2, SPEC-0001-C8
**Dependencies:** `unicode-normalization`, `unicode-segmentation`
**Files:** `agos-morph/src/unicode_validator.rs` (or new `agos-unicode` crate)

### 2.2 ⬜ MOD-02: Lexer

**What:** Create lexical analyzer that:
- Segments normalized text into tokens (words, punctuation, whitespace, numbers, symbols)
- Assigns token IDs and byte offsets
- Produces `TokenStream` (IR-2)

```rust
pub struct Lexer;
impl PipelineStage<NormalizedText, TokenStream> for Lexer { ... }
```

**Spec:** SPEC-0001-C3 §3
**Files:** `agos-morph/src/lexer.rs`

### 2.3 ⬜ MOD-03: Tokenizer

**What:** Create tokenizer that:
- Identifies proclitics (وَ, فَ, بِ, لِ, كَ, سَ)
- Identifies enclitics (object pronouns: هُ, هَا, هُمْ, etc.)
- Uses KB-0005 (Particles) and KB-0006 (Pronouns) for fast-path matching
- Generates ambiguity sets for multiple possible segmentations
- Produces `SegmentedTokenStream` (IR-3)

**Spec:** SPEC-0001-C3 §4, SPEC-0101
**Files:** `agos-morph/src/tokenizer.rs`

### 2.4 ⬜ MOD-04: MorphologicalParser (core)

**What:** Create morphological analyzer that:
- Fast path: check KB-0005 (Particles) → if match, skip
- Fast path: check KB-0006 (Pronouns) → if match, skip
- Root extraction: trie lookup in KB-0001
- Wazan identification: signature matching in KB-0002
- Feature extraction: pack into 64-bit bitfield (KB-0007)
- Produces `MorphologicalAnalysis` (IR-4)

**Spec:** SPEC-0101 §3-4
**Files:** `agos-morph/src/morphological_parser.rs`, `agos-morph/src/root_extraction.rs`, `agos-morph/src/wazan_matching.rs`

### 2.5 ⬜ MOD-04: MorphologicalParser (advanced)

**What:** Handle complex cases:
- Weak root handling (mithal, ajwaf, naqis, lafif)
- Doubled/geminate root handling
- Hamza variant handling
- Broken plural identification via KB-0004
- School-specific behavior hooks
- Verb form I–XV recognition via KB-0003

**Spec:** SPEC-0101 §5-7
**Files:** `agos-morph/src/weak_roots.rs`, `agos-morph/src/hamza.rs`, `agos-morph/src/verb_forms.rs`

### 2.6 ⬜ MOD-05: SyntaxParser

**What:** Create syntactic parser that:
- Identifies sentence type (jumlah ismiyyah, jumlah fi'liyyah, etc.)
- Assigns syntactic roles (mubtada', khabar, fi'l, fa'il, etc.)
- Builds constituency tree
- Handles ambiguity × morphology (ambiguity forest)
- Partial parse on failure

**Spec:** SPEC-0101 §8
**Files:** `agos-morph/src/syntax_parser.rs`, `agos-morph/src/sentence_type.rs`, `agos-morph/src/constituent_builder.rs`

### 2.7 ⬜ MOD-06: GIRConstructor

**What:** Create GIR constructor that:
- Aligns token data from MOD-04 and MOD-05
- Merges morphological and syntactic analyses
- Creates unified GrammarIR (IR-6)
- Handles ambiguity forest (morphology × syntax combinations)
- Collects evidence from all upstream stages

**Spec:** SPEC-0001-C5 §7 (IR-6)
**Files:** `agos-gir/src/lib.rs`

### 2.8 ⬜ Early integration test: Text → GIR

**What:** Pipe Arabic sentences through MOD-01 → MOD-06 and verify GIR output.

```rust
#[test]
fn test_text_to_gir() {
    let text = "السَّلَامُ عَلَيْكُمْ";
    let suite = load_kb_suite();
    let gir = run_pipeline(&text, &suite)?;
    assert_eq!(gir.tokens.len(), 2);
    assert_eq!(gir.trees.len(), 1);
}
```

**Files:** `agos-gir/tests/integration.rs`

---

## Phase 3: Rule Engine + DSL Compiler

### 3.1 ⬜ Grammar DSL lexer + parser

**What:** Create a Rust `nom`-based parser for `.agosrule` files that produces an AST:
- Rule structure: `rule <name> [priority] { condition => action }`
- Conditions: `matches`, `has_feature`, `is_nominative`, `is_definite`, etc.
- Actions: `confirm`, `reject`, `assign_feature`, `flag`

**Spec:** RFC-0001 §3-5
**Dependencies:** `nom`
**Files:** `agos-rule-engine/src/dsl/{lexer.rs, parser.rs, ast.rs}`

### 3.2 ⬜ DSL semantic analysis + validation

**What:** Type-check and validate DSL AST:
- Reference resolution (rule-to-rule, feature-to-KB)
- `matches` operator validation against known patterns
- School compatibility checking
- Circular dependency detection

**Spec:** RFC-0001 §6-7
**Files:** `agos-rule-engine/src/dsl/{semantic.rs, validation.rs}`

### 3.3 ⬜ DSL standard library

**What:** Implement built-in predicate functions:
- `is_nominative(token)`, `is_accusative(token)`, `is_genitive(token)`
- `is_definite(token)`, `is_indefinite(token)`
- `has_feature(token, name, value)`
- `agrees_with(token_a, token_b, features)`
- `governs_case(particle, target)`

**Spec:** RFC-0004 §6
**Files:** `agos-rule-engine/src/dsl/stdlib.rs`

### 3.4 ⬜ DSL → Bytecode compiler

**What:** Compile validated DSL AST into Grammar Bytecode:
- Rule → bytecode instruction sequence
- Conditions → conditional branch instructions
- Actions → feature manipulation instructions
- Emit section data for the `.agos` binary format

**Spec:** RFC-0001 §8, RFC-0002
**Files:** `agos-rule-engine/src/dsl/{compiler.rs, codegen.rs}`

### 3.5 ⬜ MOD-07: RuleEngine

**What:** Create rule engine that:
- Loads compiled rule sets (per grammar school)
- Applies rules in priority order to the GIR
- Confirms/rejects ambiguous analyses
- Assigns/modifies grammatical features
- Raises grammatical flags
- Produces AnnotatedGIR (IR-7) with complete rule application evidence
- Detects fixpoint (no more state changes) to prevent infinite loops

**Spec:** SPEC-0201
**Files:** `agos-rule-engine/src/{lib.rs, engine.rs, evidence.rs}`

### 3.6 ⬜ MOD-08: KnowledgeGraphResolver

**What:** Create KG resolver that:
- Looks up root entries from KB-0001
- Looks up wazan entries from KB-0002
- Links dictionary definitions
- Resolves cross-references (cognates, synonyms, antonyms)
- Attaches semantic tags
- Produces ResolvedGIR (IR-8) with resolution statistics

**Spec:** SPEC-0401
**Files:** `agos-rule-engine/src/kg_resolver.rs`

### 3.7 ⬜ School-specific rule sets (Basra, Kufa)

**What:** Write initial Grammar DSL rule files for:
- Basra school: default rules for nominal/verbal sentences
- Kufa school: alternative rules for key constructions
- ~50-100 rules per school initially

**Spec:** RFC-0004 §4
**Files:** `rules/basra/*.agosrule`, `rules/kufa/*.agosrule`

### 3.8 ⬜ Rule engine test suite

**What:** JSON fixture-based tests:
- Each test: GIR input + expected AnnotatedGIR output
- Verify rule firing, evidence entries, and feature modifications
- ~50+ tests per school

**Spec:** RFC-0004 §9, SPEC-0201 §7
**Files:** `agos-rule-engine/tests/{fixtures,conformance}.rs`

---

## Phase 4: Grammar Bytecode

### 4.1 ⬜ Bytecode binary format (de)serialization

**What:** Implement the `.agos` binary format:
- Magic bytes: `0x41474F53` ("AGOS")
- Header with version, flags, section count
- Section table with type, offset, size
- String table with UTF-8 string encoding
- End marker: `0x454E4444` ("ENDD")

```rust
pub struct BytecodeReader;
impl BytecodeReader {
    pub fn read(bytes: &[u8]) -> Result<GrammarBytecode, BytecodeError>;
    pub fn section(&self, section_type: SectionType) -> Option<&[u8]>;
}
```

**Spec:** RFC-0002 §3-15
**Files:** `agos-bytecode/src/{lib.rs, reader.rs, writer.rs, sections.rs}`

### 4.2 ⬜ LEB128 varint + CRC32C support

**What:** Implement variable-length integer encoding and section checksums:
- LEB128 encoding for compact integers
- CRC32C checksums per section
- Integrity verification on load

**Spec:** RFC-0002 §4
**Files:** `agos-bytecode/src/{leb128.rs, checksum.rs}`

### 4.3 ⬜ Feature bitfield section encoding

**What:** Implement 64-bit feature bitfield serialization:
- Pack/unpack features to/from the bitfield format
- Encode in bytecode's morphology section
- Support plugin extension bits (48-63)

**Spec:** RFC-0002 §9.2, KB-0007
**Files:** `agos-bytecode/src/features.rs`

### 4.4 ⬜ MOD-09: BytecodeGenerator

**What:** Compile ResolvedGIR (IR-8) into `.agos` bytecode:
- Emit header
- Emit metadata section (school, KB versions, timestamp)
- Emit tokens section (compact bitfield encoding)
- Emit morphology section (root/wazan IDs, feature bitfields)
- Emit syntax section (tree structures, roles)
- Emit rules section (rule applications)
- Emit evidence section
- Emit string table
- Apply optimizations (level 0/1/2)

**Spec:** RFC-0002
**Files:** `agos-bytecode/src/generator.rs`

### 4.5 ⬜ Bytecode conformance tests (~275)

**What:** Verify all bytecode operations:
- Every instruction round-trips correctly
- All section types encode/decode
- String table deduplication works
- Edge cases: empty, max size, version mismatch

**Spec:** RFC-0002 §18
**Files:** `agos-bytecode/tests/conformance.rs`

---

## Phase 5: Grammar Virtual Machine

### 5.1 ⬜ GVM core: register file + 2 stacks

**What:** Create GVM execution context with:
- 64-bit general-purpose registers
- Value stack (for operand evaluation)
- Call stack (for rule function calls)
- Program counter
- Step counter

**Spec:** SPEC-0304, RFC-0003 §4-5
**Files:** `agos-gvm/src/{core.rs, registers.rs, stack.rs}`

### 5.2 ⬜ GVM memory model (7 regions)

**What:** Implement 7 memory regions with bump allocator:
- Token region: token data from bytecode
- Feature region: feature bitfields
- Constituent region: syntax tree nodes
- String region: interred UTF-8 strings
- Rule region: rule application records
- Evidence region: evidence trail entries
- Scratch region: temporary computation workspace
- Budget: 64 MiB total, configurable

**Spec:** SPEC-0304 §3-5
**Files:** `agos-gvm/src/memory/{regions.rs, allocator.rs, budget.rs}`

### 5.3 ⬜ GVM instruction dispatch (~50 opcodes)

**What:** Implement decode + execute for ~50 instructions across 9 categories:
- Flow control (JUMP, CALL, RET, HALT)
- Stack (PUSH, POP, DUP, SWAP)
- Token (LOAD_TOKEN, LOAD_FEATURE)
- Feature (EXTRACT, MODIFY, COMPARE)
- Constituent (LOAD_CONSTITUENT, WALK)
- Rule (APPLY_RULE, CONFIRM, REJECT)
- Evidence (RECORD, EMIT)
- Output (OUTPUT_JSON, OUTPUT_TEXT)
- Extension (PLUGIN_CALL)

**Spec:** SPEC-0302, RFC-0003 §6-8
**Files:** `agos-gvm/src/instructions/{mod.rs, flow.rs, stack.rs, token.rs, feature.rs, constituent.rs, rule.rs, evidence.rs, output.rs, extension.rs}`

### 5.4 ⬜ GVM verification (12 checks)

**What:** Pre-execution bytecode verification:
- Magic bytes validity
- Version compatibility
- Section integrity (CRC32C)
- Stack balance analysis
- Type checking
- Bounds checking (token indices, register indices)
- Memory budget check
- Instruction alignment
- String table coherence
- Feature bitfield validation
- Evidence completeness
- Plugin capability validation

**Spec:** SPEC-0301 §4
**Files:** `agos-gvm/src/verifier.rs`

### 5.5 ⬜ GVM tracing + step counting

**What:** Debug and monitoring infrastructure:
- Instruction-level tracing per step
- Step counter with configurable max (default 100K)
- Memory usage tracking
- Execution time measurement
- Debug output on HALT
- Budget enforcement and graceful termination

**Spec:** SPEC-0301 §6
**Files:** `agos-gvm/src/{tracer.rs, budget.rs}`

### 5.6 ⬜ GVM bytecode loader

**What:** Load and prepare bytecode for execution:
- Read `.agos` file
- Verify sections (using verifier)
- Build section index for O(1) access
- Pre-compute string table offsets
- Allocate memory regions based on bytecode size

**Spec:** RFC-0002, SPEC-0301
**Files:** `agos-gvm/src/loader.rs`

### 5.7 ⬜ GVM conformance tests (~163)

**What:** Test each opcode, each memory region, verification checks:
- Per-opcode unit tests
- Memory allocation/deallocation patterns
- Verification edge cases
- Budget enforcement

**Spec:** RFC-0003 §11
**Files:** `agos-gvm/tests/{opcodes.rs, memory.rs, verification.rs}`

### 5.8 ⬜ Memory model conformance tests (~215)

**What:** Comprehensive memory model tests:
- Budget calculation for each region
- Allocation patterns (sequential, interleaved)
- Out-of-memory handling
- Region isolation (cross-region access prevention)
- Bump allocator reset/reuse

**Spec:** SPEC-0304 §9
**Files:** `agos-gvm/tests/memory_model.rs`

---

## Phase 6: Explanation Engine

### 6.1 ⬜ MOD-11: Evidence-to-Explanation mapping

**What:** Transform GVM's `AnalysisResult` (IR-10) into structured explanation data:
- Map evidence entries to natural language text
- Extract key grammatical decisions
- Organize by token and by construction

**Spec:** SPEC-0501 §3
**Files:** `agos-explanation/src/{mapping.rs, evidence_reader.rs}`

### 6.2 ⬜ Template system (Handlebars)

**What:** Implement template-based explanation generation:
- `handlebars`-compatible templates
- Arabic-aware helpers (right-to-left, pluralization, gender agreement)
- Template loading from embedded or file-based sources
- Multi-language templates (Arabic, English, Urdu, Malay)

**Spec:** SPEC-0501 §4
**Dependencies:** `handlebars`
**Files:** `agos-explanation/src/{templates.rs, helpers.rs}`, `agos-explanation/templates/{ar,en,ur}/**/*.hbs`

### 6.3 ⬜ I'rab generation (5 breakdown types)

**What:** Generate Arabic grammatical breakdowns:
- Per-word morphological breakdown (root, wazan, POS, features)
- Per-word syntactic breakdown (role, governor)
- Construction-level breakdown (idafa, wasf, tawkid)
- Full sentence I'rab
- Educational annotation (with color-coding markers)

**Spec:** SPEC-0501 §5
**Files:** `agos-explanation/src/{irab.rs, constructions.rs}`

### 6.4 ⬜ Output formatters (JSON, Text, HTML)

**What:** Format explanations for output:
- JSON: structured `ExplanationOutput` (IR-11)
- Text: plain text with ANSI formatting
- HTML: styled web output with CSS classes for Arabic text

**Spec:** SPEC-0501 §7
**Files:** `agos-explanation/src/formatters/{json.rs, text.rs, html.rs}`

### 6.5 ⬜ LLM integration provider interface

**What:** Optional LLM augmentation for explanations:
- Provider trait: `LlmProvider { fn explain(&self, analysis: &AnalysisResult, config: &LlmConfig) -> LlmResult }`
- Built-in: API-based (OpenAI-compatible)
- Strictly: LLM receives `AnalysisResult`, NEVER modifies it
- Fallback: template-based when LLM unavailable

**Spec:** SPEC-0501 §6
**Files:** `agos-explanation/src/llm/{provider.rs, openai.rs, fallback.rs}`

### 6.6 ⬜ Explanation engine test suite

**What:** Test explanation output:
- Template rendering correctness
- I'rab breakdown accuracy
- Format fidelity (JSON schema, HTML structure)
- Multi-language output consistency
- LLM integration edge cases (timeout, unavailability, malformed response)

**Spec:** SPEC-0501
**Files:** `agos-explanation/tests/{templates.rs, irab.rs, formats.rs}`

---

## Phase 7: CLI + Server + Integration

### 7.1 ⬜ CLI tool (`agos`)

**What:** Create the main `agos` CLI binary with subcommands:
```bash
agos analyze "السَّلَامُ عَلَيْكُمْ"        # Analyze text
agos compile rules/ output/                  # Compile DSL rules
agos run bytecode.agos                       # Run bytecode in GVM
agos kb-compile knowledge/ output/           # Compile KB sources
agos plugin list                             # Manage plugins
agos serve                                   # Start server
```

**Subcommands:**
- `analyze`: Full pipeline from text to explanation
- `compile`: Compile DSL rule files to bytecode
- `run`: Execute pre-compiled bytecode
- `kb-compile`: Compile YAML KB sources to binary
- `plugin`: List/install/remove plugins
- `serve`: Start REST API server

**Spec:** SPEC-0001-C6
**Dependencies:** `clap`
**Files:** `agos-cli/src/{main.rs, commands/{analyze.rs, compile.rs, run.rs, kb_compile.rs, plugin.rs, serve.rs}}`

### 7.2 ⬜ REST API server

**What:** Create `agos-server` with REST endpoints:
- `POST /v1/analyze` — single text analysis
- `POST /v1/analyze/batch` — batch analysis
- `GET /v1/kb/{kb_id}/query` — KB queries
- `GET /v1/health` — health check
- `GET /v1/version` — version info

**Spec:** SPEC-0001-C6 §1.2
**Dependencies:** `axum`, `tokio`
**Files:** `agos-server/src/{main.rs, routes.rs, middleware.rs}`

### 7.3 ⬜ MOD-13: CacheManager

**What:** Implement pipeline output caching:
- LRU cache for hot paths
- Redis backend for distributed caching
- Cache key design: hash(input + stage + config + KB versions)
- Automatic invalidation on KB version changes

```rust
pub trait CacheBackend {
    fn get(&self, key: &CacheKey) -> Option<Vec<u8>>;
    fn set(&self, key: &CacheKey, value: &[u8], ttl: Duration);
    fn invalidate(&self, filter: &InvalidationFilter);
}
```

**Spec:** SPEC-0001-C5, SPEC-0103
**Dependencies:** optional `redis`
**Files:** `agos-cache/src/{lib.rs, lru.rs, redis.rs, key.rs}`

### 7.4 ⬜ MOD-14: APIGateway

**What:** API gateway layer:
- Request validation
- Pipeline configuration per request (school, mode, language)
- Rate limiting (token bucket)
- API versioning (`/v1/analyze`, `/v2/analyze`)
- Response formatting (JSON, XML)
- Request ID tracing

**Spec:** SPEC-0001-C4, SPEC-0001-C6
**Files:** `agos-api/src/{lib.rs, router.rs, rate_limiter.rs, versioning.rs}`

### 7.5 ⬜ End-to-end integration tests

**What:** Full pipeline integration tests:
- Arabic text → Explanation output
- All mini pipelines (morphology-only, syntax-only)
- All grammar schools (Basra, Kufa, Baghdad)
- Error handling (invalid input, missing KBs, corrupted bytecode)
- Cache integration (hit/miss verification)
- Multi-language output

**Files:** `tests/e2e/{basic.rs, schools.rs, errors.rs, cache.rs}`

### 7.6 ⬜ Benchmark suite

**What:** Performance benchmarks using `criterion`:
- Per-stage latency budgets (SPEC-0001-C9)
- Throughput targets (sentences/second)
- Cache hit/miss performance
- KB lookup benchmarks
- GVM execution benchmarks
- Regression tracking in CI

**Spec:** SPEC-0001-C9, SPEC-0103
**Dependencies:** `criterion`
**Files:** `benches/{pipeline.rs, kb_lookup.rs, gvm.rs}`

---

## Phase 8: Plugin System + Advanced

### 8.1 ⬜ MOD-12: PluginLoader core

**What:** Core plugin loading infrastructure:
- Plugin discovery (scan directories for manifests)
- Manifest validation (YAML/JSON schema)
- Plugin lifecycle: discover → validate → load → inject → unload
- Hot-reload support
- 9 plugin types: normalizer, token_classifier, segmenter, morphology_engine, syntax_engine, rule_set, kb_resolver, explanation, api_middleware

**Spec:** SPEC-0601 §3-4
**Files:** `agos-plugin/src/{lib.rs, loader.rs, lifecycle.rs, manifest.rs}`

### 8.2 ⬜ WASM sandbox (wasmtime)

**What:** WASM runtime for plugin isolation:
- `wasmtime` integration
- Capability-based security (plugins declare required permissions)
- Resource limits (memory, CPU time)
- Host function bindings (AGOS API exposed to WASM)
- Sandbox: no file I/O, no network, no system calls unless explicitly permitted

**Spec:** SPEC-0601 §6, ADR-0005
**Dependencies:** `wasmtime`
**Files:** `agos-plugin/src/sandbox/{runtime.rs, host_functions.rs, permissions.rs}`

### 8.3 ⬜ Plugin manifest + signing

**What:** Plugin distribution infrastructure:
- Manifest format (YAML): name, version, author, entry point, dependencies, permissions
- ed25519 cryptographic signing
- SHA-256 integrity verification
- Manifest schema validation

**Spec:** SPEC-0601 §5
**Dependencies:** `ed25519-dalek`, `sha2`
**Files:** `agos-plugin/src/{manifest.rs, signing.rs, verification.rs}`

### 8.4 ⬜ Plugin registry (SQLite)

**What:** Local plugin database:
- SQLite-backed registry
- CRUD: install, update, remove, list
- Dependency resolution (detect circular deps)
- Version compatibility checking

**Spec:** SPEC-0601 §4.4
**Dependencies:** `rusqlite`
**Files:** `agos-plugin/src/registry.rs`

### 8.5 ⬜ Plugin SDK crate (`agos-plugin-sdk`)

**What:** Rust SDK for plugin authors:
- Derive macros: `#[agos_plugin]` for automatic manifest generation
- Host function bindings (safe wrappers around WASM imports)
- MessagePack serialization for plugin boundary
- Example plugin templates
- Documentation and migration guide

**Spec:** SPEC-0601 §7
**Dependencies:** `proc-macro2`, `quote`, `syn`, `rmp-serde`
**Files:** `agos-plugin-sdk/src/{lib.rs, macros.rs, host.rs}`

### 8.6 ⬜ School-specific rule packages

**What:** Package grammar school rule sets as distributable plugins:
- Basra school (~200 rules)
- Kufa school (~200 rules)
- Baghdad school (~150 rules)
- Andalus school (~150 rules)
- Modern school (~100 rules)
- Each as a WASM-compiled plugin with manifest

**Spec:** RFC-0004, SPEC-0601
**Files:** `plugins/basra/{manifest.yaml, rules/*.agosrule, src/lib.rs}`

### 8.7 ⬜ Performance optimization pass

**What:** Hot path profiling and optimization:
- Profile end-to-end pipeline with `criterion` + `perf`/`flamegraph`
- Optimize KB lookups: known words index, prefix caching
- Optimize root extraction: trie with fallback heuristics
- Optimize GVM: instruction fusion, JIT compilation (future)
- Fast-path optimization: skip stages for known/common patterns

**Spec:** SPEC-0103, SPEC-0001-C9
**Files:** `agos-core/src/optimization.rs`, benchmarks in `benches/`

### 8.8 ⬜ Fuzz testing + security audit

**What:** Security hardening:
- WASM parsing fuzz with `cargo-fuzz` / `afl`
- Input validation fuzz (malformed Arabic text, edge case Unicode)
- Bytecode fuzz (corrupted sections, invalid instructions)
- Dependency audit with `cargo audit` + `trivy`
- Static analysis with `cargo clippy` + `cargo deny`
- Documentation of threat model and security boundaries

**Spec:** SPEC-0601 §6.5, SPEC-0001-C8
**Files:** `fuzz/{fuzz_targets/{wasm_parse.rs, bytecode_parse.rs, input_validate.rs}}`

---

## Implementation Order Summary

```
NOW → Phase 0 (finish scaffolding: CI, PipelineStage trait, tracing)
    → Phase 1 (KB data: YAML sources + binary compiler)
    → Phase 2 (pipeline frontend: Text → GIR, with integration test)
    → Phase 3 (rules + DSL: parser, compiler, rule engine)
    → Phase 4 (bytecode: binary format, generator)
    → Phase 5 (GVM: execution engine)
    → Phase 6 (explanation: templates, I'rab, output)
    → Phase 7 (CLI + server: agos binary, REST API)
    → Phase 8 (plugins: WASM sandbox, SDK, optimization)
```

Each phase builds on the previous. Phases can overlap at granular task level (e.g., KB-0005 data can be authored while Phase 2 work starts on the pipeline).

---

## Current Status (2026-07-21)

| Phase | Progress | Key Milestones |
|-------|----------|----------------|
| **Phase 0: Scaffolding** | ~40% | ✅ agos-core, agos-kb, workspace |
| **Phase 1: KB Data** | ~10% | ✅ Rust types + loader; ⬜ YAML sources + compiler |
| **Phase 2: Text → GIR** | 0% | ⬜ Not started |
| **Phase 3: Rules + DSL** | 0% | ⬜ Not started |
| **Phase 4: Bytecode** | 0% | ⬜ Not started |
| **Phase 5: GVM** | 0% | ⬜ Not started |
| **Phase 6: Explanation** | 0% | ⬜ Not started |
| **Phase 7: CLI + Server** | 0% | ⬜ Not started |
| **Phase 8: Plugins** | 0% | ⬜ Not started |
