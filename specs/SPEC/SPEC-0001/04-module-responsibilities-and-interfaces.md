---
spec_id: SPEC-0001
chapter: 4
title: Module Responsibilities & Interfaces
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0301: Grammar Runtime
  - SPEC-0401: Knowledge Graph Engine
  - SPEC-0501: Explanation Engine
  - SPEC-0601: Plugin System
  - RFC-0001: Grammar DSL (proposed)
  - RFC-0002: Grammar Bytecode (proposed)
  - RFC-0003: Grammar Virtual Machine (proposed)
  - ADR-0001: Compiler Architecture Rationale
---

# Chapter 4: Module Responsibilities & Interfaces

## Table of Contents

1. [Interface Design Principles](#1-interface-design-principles)
2. [Common Types & Error Handling](#2-common-types--error-handling)
3. [MOD-01: UnicodeValidator Interface](#3-mod-01-unicodevalidator-interface)
4. [MOD-02: Lexer Interface](#4-mod-02-lexer-interface)
5. [MOD-03: Tokenizer Interface](#5-mod-03-tokenizer-interface)
6. [MOD-04: MorphologicalParser Interface](#6-mod-04-morphologicalparser-interface)
7. [MOD-05: SyntaxParser Interface](#7-mod-05-syntaxparser-interface)
8. [MOD-06: GIRConstructor Interface](#8-mod-06-girconstructor-interface)
9. [MOD-07: RuleEngine Interface](#9-mod-07-ruleengine-interface)
10. [MOD-08: KnowledgeGraphResolver Interface](#10-mod-08-knowledgegraphresolver-interface)
11. [MOD-09: BytecodeGenerator Interface](#11-mod-09-bytecodegenerator-interface)
12. [MOD-10: GVM Interface](#12-mod-10-gvm-interface)
13. [MOD-11: ExplanationEngine Interface](#13-mod-11-explanationengine-interface)
14. [MOD-12: PluginLoader Interface](#14-mod-12-pluginloader-interface)
15. [MOD-13: CacheManager Interface](#15-mod-13-cachemanager-interface)
16. [MOD-14: APIGateway Interface](#16-mod-14-apigateway-interface)
17. [Pipeline Orchestrator Interface](#17-pipeline-orchestrator-interface)
18. [Versioning & Compatibility Policy](#18-versioning--compatibility-policy)
19. [Cross-References](#19-cross-references)

---

## 1. Interface Design Principles

All AGOS module interfaces conform to the following principles:

### 1.1 Principle 1: Strict Input/Output Contracts

Every module defines exactly one input type and one output type. The input is the **complete** set of data the module requires. The output is the **complete** set of data the module produces. No module reads from global state, environment variables, or shared memory.

### 1.2 Principle 2: Stateless by Default

Modules are stateless functions: given the same input, they always produce the same output. State (caching, configuration) is managed externally by the Pipeline Orchestrator or CacheManager.

### 1.3 Principle 3: Error Handling via Union Types

Every module returns either a success value or a structured error value. The calling code MUST handle both cases. Errors are never returned as exceptions, panics, or side effects.

```
Result<T, E> = Ok(T) | Err(E)
```

### 1.4 Principle 4: Serialization Independence

Interface types are defined abstractly. Implementations MAY serialize data between stages or pass references in-process. The interface contract is the **logical type**, not the wire format.

### 1.5 Principle 5: Backward Compatibility

All interface changes MUST be backward-compatible at the minor version level. Breaking changes require a major version bump and a migration period defined in the deprecation policy (see Section 18).

### 1.6 Notation Used

Throughout this chapter, interfaces are expressed in a language-neutral pseudo-interface notation:

```
trait ModuleName {
    /// Human-readable description
    fn process(input: InputType) -> Result<OutputType, ErrorType>;
}
```

Language-specific implementations (Rust traits, Go interfaces, C++ abstract classes, TypeScript types) MUST conform to the logical interface defined here.

---

## 2. Common Types & Error Handling

### 2.1 Result Type

```
type Result<T, E> = {
    success: true,
    value: T,
} | {
    success: false,
    error: E,
}
```

### 2.2 PipelineError (Common Error Envelope)

All modules return errors that conform to this envelope:

```
type PipelineError = {
    code: string,                    // Machine-readable error code
    message: string,                 // Human-readable description
    stage: string,                   // Stage ID (e.g., "MOD-04")
    is_fatal: boolean,               // Whether pipeline should stop
    recovery_hint: string | null,    // Optional recovery suggestion
    inner: Error | null,             // Optional wrapped error (for debugging)
}
```

### 2.3 EvidenceEntry (Common Evidence Type)

```
type EvidenceEntry = {
    stage: string,                   // Stage that produced this evidence
    rule_or_algorithm: string,       // Rule ID or algorithm name
    input_hash: string,              // Hash of input state
    output_description: string,      // What was decided
    confidence: float,               // 0.0 to 1.0
    timestamp: string,               // ISO 8601
}
```

### 2.4 KnowledgeVersionMap

```
type KnowledgeVersionMap = {
    [kb_id: string]: string,         // e.g., "KB-0001": "1.2.3"
}
```

### 2.5 StageConfig (Base Configuration)

Every stage accepts a configuration object. The base configuration fields are:

```
type StageConfig = {
    stage_id: string,                // e.g., "MOD-01"
    knowledge_versions: KnowledgeVersionMap,
    log_level: "debug" | "info" | "warn" | "error",
}
```

Individual stages extend this with stage-specific fields (documented in each section below).

---

## 3. MOD-01: UnicodeValidator Interface

### 3.1 Trait Definition

```
trait UnicodeValidator {
    /// Validate and normalize Arabic text.
    /// Returns normalized text suitable for downstream processing.
    fn validate(
        input: UnicodeValidatorInput
    ) -> Result<UnicodeValidatorOutput, UnicodeValidatorError>;
}
```

### 3.2 Input Type

```
type UnicodeValidatorInput = {
    raw_text: string,                              // Raw UTF-8 Arabic text
    config: {
        normalize_tashkeel: boolean,               // default: false
        strip_tatweel: boolean,                    // default: true
        strict_arabic_only: boolean,               // default: false
        allowed_unicode_ranges: string[],          // default: ["0600-06FF", "0750-077F", "08A0-08FF"]
        max_input_size: integer,                   // default: 1048576 (1 MiB)
    },
}
```

### 3.3 Output Type

```
type UnicodeValidatorOutput = {
    normalized_text: string,
    original_text: string,
    metadata: {
        char_count: integer,
        word_count_estimate: integer,
        has_tashkeel: boolean,
        has_tatweel: boolean,
        has_quranic_symbols: boolean,
        normalization_applied: string[],
    },
}
```

### 3.4 Error Codes

| Code | HTTP Analogue | Fatal | Description |
|------|---------------|-------|-------------|
| `INVALID_ENCODING` | 400 | Yes | Malformed UTF-8 bytes |
| `EMPTY_INPUT` | 400 | Yes | Empty string provided |
| `MAX_LENGTH_EXCEEDED` | 413 | Yes | Input exceeds maximum size |
| `NON_ARABIC_CHAR` | 400 | Config-dependent | Non-Arabic character in strict mode |
| `UNSUPPORTED_CHAR` | 400 | Config-dependent | Character in recognized but unsupported range |

### 3.5 Implementation Contract

1. **Idempotent:** `validate(validate(input)) == validate(input)` — applying validation twice produces the same result.
2. **Preserves input:** The `original_text` field MUST contain the exact input string before any normalization.
3. **Zero knowledge dependencies:** No KBs are required. Unicode tables are compiled into the implementation.

---

## 4. MOD-02: Lexer Interface

### 4.1 Trait Definition

```
trait Lexer {
    /// Tokenize normalized Arabic text into a stream of raw tokens.
    fn lex(
        input: LexerInput
    ) -> Result<TokenStream, LexerError>;
}
```

### 4.2 Input Type

```
type LexerInput = {
    normalized_text: string,                       // From MOD-01
    config: {
        skip_whitespace_tokens: boolean,           // default: false
        include_offsets: boolean,                  // default: true
    },
}
```

### 4.3 Output Type

```
type RawToken = {
    id: integer,                                   // 0-based sequential ID
    text: string,
    token_type: "word" | "punctuation" | "number" | "whitespace" | "symbol" | "unknown",
    start_offset: integer,                         // Byte offset in normalized text
    end_offset: integer,                           // Exclusive byte offset
}

type TokenStream = {
    text: string,                                  // Reference to input text
    tokens: RawToken[],
    metadata: {
        token_count: integer,
        word_count: integer,
        has_tokens: boolean,
    },
}
```

### 4.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `INTERNAL_ERROR` | Yes | Unexpected algorithm failure |

### 4.5 Implementation Contract

1. **Monotonic token IDs:** Token IDs MUST be sequential, starting from 0, without gaps.
2. **Non-overlapping spans:** Token offsets MUST NOT overlap. Each byte of the input belongs to exactly one token.
3. **Coverage:** The concatenation of all token texts (in order) MUST equal the input text (whitespace tokens included).

---

## 5. MOD-03: Tokenizer Interface

### 5.1 Trait Definition

```
trait Tokenizer {
    /// Segment raw tokens into morphemes using clitic tables.
    fn tokenize(
        input: TokenizerInput
    ) -> Result<SegmentedTokenStream, TokenizerError>;
}
```

### 5.2 Input Type

```
type TokenizerInput = {
    token_stream: TokenStream,                     // From MOD-02
    config: {
        max_segmentations: integer,                // default: 16
        known_clitics_path: string | null,         // default: null (use built-in)
    },
}
```

### 5.3 Output Type

```
type Morpheme = {
    text: string,
    morpheme_type: "prefix" | "stem" | "suffix" | "clitic" | "particle",
    original_offset: integer,                      // Offset within the raw token
    length: integer,
}

type Segmentation = {
    morphemes: Morpheme[],                         // Ordered prefix → stem → suffix
    confidence: float,                             // 0.0 to 1.0
    source: "builtin" | "custom_rule",
}

type SegmentedToken = {
    raw_token: RawToken,
    segmentations: Segmentation[],                 // Ordered by confidence
}

type SegmentedTokenStream = {
    tokens: SegmentedToken[],
    metadata: {
        total_tokens: integer,
        segmentable_tokens: integer,
        ambiguous_tokens: integer,
        total_ambiguity: float,                    // Average per segmentable token
    },
}
```

### 5.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `MAX_SEGMENTATIONS_EXCEEDED` | Yes | Number of ambiguity alternatives exceeds limit |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 5.5 Implementation Contract

1. **Default segmentation always included:** The "no clitics" segmentation (entire token is the stem) MUST always be present in the alternatives list.
2. **Alif-lam (ال) is NOT segmented:** The definite article is part of the stem, handled by MorphologicalParser.
3. **Ta-marbuta (ة) is NOT segmented:** It is a stem character, not a suffix.
4. **Empty stem rejection:** Any segmentation that would leave an empty stem MUST be rejected.

---

## 6. MOD-04: MorphologicalParser Interface

### 6.1 Trait Definition

```
trait MorphologicalParser {
    /// Perform morphological analysis on segmented tokens.
    fn analyze_morphology(
        input: MorphologicalParserInput
    ) -> Result<MorphologicalAnalysisOutput, MorphologicalParserError>;
}
```

### 6.2 Input Type

```
type MorphologicalParserInput = {
    segmented_stream: SegmentedTokenStream,         // From MOD-03
    config: {
        school: "basra" | "kufa" | "baghdad" | "andalus" | "modern",
        max_morphological_analyses: integer,        // default: 32
        enable_guess: boolean,                      // default: false
        known_words_path: string | null,            // default: null
    },
}
```

### 6.3 Output Type

```
type MorphologicalFeature = {
    name: string,                                   // e.g., "gender", "number"
    value: string,                                  // e.g., "masculine", "plural"
    confidence: float,
    source: string,                                 // KB entry or rule ID
}

type MorphologicalAnalysis = {
    stem: string,
    root: {
        text: string,
        source: string,
        confidence: float,
    } | null,
    wazan: {
        text: string,
        source: string,
        form: integer | null,
        confidence: float,
    } | null,
    pos: PartOfSpeech,
    features: MorphologicalFeature[],
    is_ambiguous: boolean,
    alternatives: MorphologicalAnalysis[],
    evidence: EvidenceEntry[],
}

type PartOfSpeech =
    "verb" | "noun" | "particle" | "proper_noun" | "pronoun" |
    "adjective" | "adverb" | "preposition" | "conjunction" |
    "interrogative" | "unknown"

type TokenAnalysis = {
    token_id: integer,
    stem_analyses: MorphologicalAnalysis[],
}

type MorphologicalAnalysisOutput = {
    token_analyses: TokenAnalysis[],
    metadata: {
        total_tokens: integer,
        analyzed_tokens: integer,
        ambiguous_tokens: integer,
        unknown_tokens: integer,
        unknown_stems: string[],
    },
}
```

### 6.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `MAX_ANALYSES_EXCEEDED` | Yes | Per-stem ambiguity exceeds limit |
| `KB_MISSING` | Yes | Required knowledge base not found |
| `KB_VERSION_MISMATCH` | Yes | KB version incompatible with school |
| `INTERNAL_ERROR` | Yes | Unexpected algorithm failure |

### 6.5 Implementation Contract

1. **Fast path for particles and pronouns:** KB-0005 (Particles) and KB-0006 (Pronouns) MUST be checked first, before root extraction. If a match is found, skip root extraction entirely.
2. **All 15 verb forms supported:** The implementation MUST recognize at least Forms I through X. Forms XI–XV are RECOMMENDED.
3. **Weak root handling:** Verbs with weak radicals (أجوف, ناقص, مثال) MUST be handled with special root extraction rules.
4. **Confidence ordering:** Alternatives MUST be ordered by descending confidence. Confidence ties are broken by source priority (known word > extraction > guess).

---

## 7. MOD-05: SyntaxParser Interface

### 7.1 Trait Definition

```
trait SyntaxParser {
    /// Parse the syntactic structure of the sentence(s).
    fn parse_syntax(
        input: SyntaxParserInput
    ) -> Result<SyntaxOutput, SyntaxParserError>;
}
```

### 7.2 Input Type

```
type SyntaxParserInput = {
    morphology: MorphologicalAnalysisOutput,         // From MOD-04
    config: {
        max_parse_trees: integer,                    // default: 8
        max_sentence_length: integer,                // default: 200
    },
}
```

### 7.3 Output Type

```
type SyntacticRole =
    "mubtada" | "khabar" | "fi'l" | "fa'il" |
    "maf'ul_bi-hi" | "maf'ul_mutlaq" | "maf'ul_fih" |
    "maf'ul_lahu" | "maf'ul_ma'ahu" |
    "hal" | "tamyiz" | "na'at" | "idafa" |
    "mudaf" | "mudaf_ilayh" |
    "harf_jarr" | "majrur" | "harf_nasb" | "harf_jazm" |
    "zarf" | "qayd" | "ta'kid" | "badal" | "atasf" |
    "istithna" | "nida" | "jawab" | "shart" | "jaza" |
    "sila" | "rabit" | "unknown"

type Constituent = {
    node_type: "word" | "phrase" | "clause",
    role: SyntacticRole,
    token_ids: integer[],
    children: Constituent[],
    features: { [key: string]: string },
}

type SyntaxTree = {
    tree_type:
        "jumlah_ismiyyah" | "jumlah_fi'liyyah" |
        "jumlah_shartiyyah" | "jumlah_zar_fiyyah" |
        "phrase" | "incomplete" | "unknown",
    root: Constituent,
    confidence: float,
}

type SyntaxOutput = {
    trees: SyntaxTree[],                             // Ordered by confidence
    metadata: {
        sentence_count: integer,
        tokens_parsed: integer,
        ambiguity_count: integer,
        parse_time_ms: float,
    },
}
```

### 7.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `MAX_TREES_EXCEEDED` | No | Too many parse trees; truncated |
| `SENTENCE_TOO_LONG` | No | Input exceeds max sentence length |
| `PARSE_FAILURE` | No | No valid parse tree found |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 7.5 Implementation Contract

1. **Partial parse on failure:** If `PARSE_FAILURE` occurs, the parser MUST still return a partial parse identifying any recognizable constituents (prepositional phrases, known verb-subject pairs, etc.).
2. **Ambiguity propagation:** Every ambiguity in the morphological input MUST be explored in the syntactic parse. A valid parse tree exists for each (morphological_analysis × syntax_rule) combination that produces a valid parse.
3. **Ellipsis marking:** Omitted elements (hadhf) MUST be marked as implicit constituents with the `implicit: true` feature flag, not silently filled in.

---

## 8. MOD-06: GIRConstructor Interface

### 8.1 Trait Definition

```
trait GIRConstructor {
    /// Combine morphology and syntax into a unified Grammar Intermediate Representation.
    fn construct_gir(
        input: GIRConstructorInput
    ) -> Result<GrammarIR, GIRConstructorError>;
}
```

### 8.2 Input Type

```
type GIRConstructorInput = {
    morphology: MorphologicalAnalysisOutput,         // From MOD-04
    syntax: SyntaxOutput,                             // From MOD-05
    original_text: string,                            // From MOD-01
    config: {
        gir_version: string,                          // e.g., "1.0"
    },
}
```

### 8.3 Output Type

```
type GIRToken = {
    index: integer,
    original_text: string,
    normalized_text: string,
    start_offset: integer,
    end_offset: integer,
    morphology: MorphologicalAnalysis | MorphologicalAnalysis[],
    clitics: {
        prefix: string[],
        suffix: string[],
    },
}

type GIRConstituent = {
    node_type: "token" | "phrase" | "clause",
    role: SyntacticRole,
    token_indices: integer[],
    children: GIRConstituent[],
    features: { [key: string]: string },
    confidence: float,
}

type GIRTree = {
    id: string,
    sentence_type: string,
    root: GIRConstituent,
    confidence: float,
    source: string,                                    // Grammar school
}

type GrammarIR = {
    version: string,
    spec_id: "SPEC-0001",
    metadata: {
        created_at: string,
        pipeline_version: string,
        knowledge_versions: KnowledgeVersionMap,
        school: string,
    },
    text: string,
    tokens: GIRToken[],
    trees: GIRTree[],
    evidence: EvidenceEntry[],
}
```

### 8.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `TOKEN_MISMATCH` | Yes | MOD-04 and MOD-05 token counts disagree |
| `GIR_VERSION_INCOMPATIBLE` | Yes | Requested GIR version is not supported |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 8.5 Implementation Contract

1. **Pure transformation:** The GIRConstructor performs no analysis. It is a pure structural transformation that merges inputs.
2. **Ambiguity forest:** For each valid (morphology × syntax) combination, exactly one GIRTree MUST be produced. Invalid combinations (e.g., verb morphology in a noun syntactic role) MUST be pruned.
3. **Evidence collection:** All evidence entries from MOD-03, MOD-04, and MOD-05 MUST be collected and included in the GIR.

---

## 9. MOD-07: RuleEngine Interface

### 9.1 Trait Definition

```
trait RuleEngine {
    /// Apply grammatical rules to the GIR.
    fn apply_rules(
        input: RuleEngineInput
    ) -> Result<AnnotatedGIR, RuleEngineError>;
}
```

### 9.2 Input Type

```
type RuleEngineInput = {
    gir: GrammarIR,                                    // From MOD-06
    config: {
        school: string,                                // e.g., "basra"
        rule_set_version: string | null,               // default: latest
        max_rule_applications: integer,                // default: 1000
        strict_mode: boolean,                          // default: false
    },
}
```

### 9.3 Output Type

```
type RuleApplication = {
    rule_id: string,
    rule_name: string,
    school: string,
    version: string,
    applies_to: {
        token_indices: integer[],
        constituent_path: string[],
    },
    condition: string,
    action: string,
    result: {
        confirmed: string[],
        rejected: string[],
        modified: { feature: string, from: string, to: string }[],
        flag: GrammaticalFlag | null,
    },
    evidence: EvidenceEntry,
}

type GrammaticalFlag = {
    flag_type: "error" | "warning" | "info",
    code: string,
    message: string,
    token_indices: integer[],
    rule_id: string,
}

type AnnotatedGIR = GrammarIR & {
    rule_applications: RuleApplication[],
    flags: GrammaticalFlag[],
    rule_set_version: string,
    school: string,
}
```

### 9.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `RULE_SET_NOT_FOUND` | Yes | Rule set for configured school not available |
| `RULE_VERSION_MISMATCH` | Yes | Rule set version is incompatible |
| `RULE_APPLICATION_LIMIT` | No | Too many rule applications; truncated |
| `RULE_CONFLICT` | Config-dependent | Conflicting rule applications (only in strict mode) |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 9.5 Implementation Contract

1. **Deterministic rule ordering:** Rules MUST be applied in a fixed, deterministic order (by priority descending, then by rule ID alphabetically for ties).
2. **Workflow isolation:** The RuleEngine operates on a mutable copy of the GIR. The original GIR is preserved for debugging and comparison.
3. **Circular rule detection:** If a full pass through all rules produces no state change, the engine MUST terminate (fixpoint reached). This prevents infinite loops.
4. **Conflict resolution priority:**
   1. Higher priority rules override lower priority rules.
   2. Within same priority: school default rules > plugin rules.
   3. If still conflicting: strict_mode determines behavior (fail vs. preserve ambiguity).

---

## 10. MOD-08: KnowledgeGraphResolver Interface

### 10.1 Trait Definition

```
trait KnowledgeGraphResolver {
    /// Resolve GIR references against knowledge bases.
    fn resolve(
        input: KnowledgeGraphResolverInput
    ) -> Result<ResolvedGIR, KnowledgeGraphResolverError>;
}
```

### 10.2 Input Type

```
type KnowledgeGraphResolverInput = {
    gir: AnnotatedGIR,                                 // From MOD-07
    config: {
        resolve_depth: integer,                        // default: 3
        enable_semantic: boolean,                      // default: true
        enable_etymology: boolean,                     // default: false
        max_entries_per_reference: integer,            // default: 5
    },
}
```

### 10.3 Output Type

```
type RootEntry = {
    id: string,
    root: string,
    meaning: string,
    forms: string[],
    derived_nouns: string[],
    cognates: string[],
    semantic_field: string,
    cross_references: {
        related_roots: string[],
        antonyms: string[],
        synonyms: string[],
    },
}

type WazanEntry = {
    id: string,
    pattern: string,
    meaning: string,
    form: integer | null,
    example: string,
}

type ResolvedToken = GIRToken & {
    root_entry: RootEntry | null,
    wazan_entry: WazanEntry | null,
    dictionary_entry: { [key: string]: any } | null,
    semantic_tags: string[],
}

type ResolvedGIR = AnnotatedGIR & {
    tokens: ResolvedToken[],
    knowledge_versions: KnowledgeVersionMap,
    resolution_stats: {
        roots_resolved: integer,
        patterns_resolved: integer,
        unresolved_references: integer,
        resolution_time_ms: float,
    },
}
```

### 10.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `KB_LOAD_FAILURE` | Yes | Knowledge base cannot be loaded |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 10.5 Implementation Contract

1. **Graceful degradation:** If a KB is unavailable, the resolver continues with available KBs and records which KBs were missing in resolution_stats.
2. **No hallucination:** The resolver MUST NOT fabricate root meanings or semantic tags. Unresolved references MUST be explicitly marked as `null`.
3. **O(1) lookup target:** All KB lookups MUST target O(1) average time complexity using hash-based indices. KBs SHOULD be memory-mapped for fast access.

---

## 11. MOD-09: BytecodeGenerator Interface

### 11.1 Trait Definition

```
trait BytecodeGenerator {
    /// Compile resolved GIR into executable Grammar Bytecode.
    fn generate_bytecode(
        input: BytecodeGeneratorInput
    ) -> Result<GrammarBytecode, BytecodeGeneratorError>;
}
```

### 11.2 Input Type

```
type BytecodeGeneratorInput = {
    gir: ResolvedGIR,                                  // From MOD-08
    config: {
        bytecode_version: string,                      // default: latest supported
        optimization_level: 0 | 1 | 2,                 // default: 1
        embed_text: boolean,                            // default: true
        embed_evidence: boolean,                        // default: true
        max_bytecode_size: integer,                     // default: 10485760 (10 MiB)
    },
}
```

### 11.3 Output Type

```
type BytecodeSection = {
    section_type:
        "header" | "metadata" | "tokens" | "morphology" |
        "syntax" | "rules" | "evidence" | "strings" | "end",
    data: Uint8Array,                                   // Raw section bytes
    offset: integer,                                    // Byte offset in raw bytecode
    size: integer,                                      // Section byte size
}

type GrammarBytecode = {
    raw: Uint8Array,                                    // Complete serialized bytecode
    sections: BytecodeSection[],
    size: integer,
    metadata: {
        input_text_hash: string,
        token_count: integer,
        tree_count: integer,
        rule_count: integer,
        compression_ratio: float,
        gir_json_size_bytes: integer,
    },
}
```

### 11.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `BYTECODE_VERSION_UNSUPPORTED` | Yes | Requested bytecode version cannot be generated |
| `GIR_VALIDATION_FAILED` | Yes | Input GIR is structurally invalid |
| `BYTECODE_TOO_LARGE` | Yes | Exceeds maximum bytecode size |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 11.5 Implementation Contract

1. **Byte-for-byte determinism:** Given the same input and configuration, `generate_bytecode` MUST produce identical bytecode (byte-for-byte) across all runs and all compliant implementations.
2. **Self-contained output:** The bytecode MUST contain all information needed for execution. No external KB references, file paths, or runtime configuration are required by the GVM.
3. **Version compatibility:** The bytecode header MUST include a version field. The GVM MUST reject bytecode with a version higher than its own.

---

## 12. MOD-10: GVM Interface

### 12.1 Trait Definition

```
trait GrammarVirtualMachine {
    /// Execute Grammar Bytecode to produce the final analysis result.
    fn execute(
        input: GVMInput
    ) -> Result<AnalysisResult, GVMError>;

    /// Verify bytecode validity without executing.
    fn verify(
        bytecode: GrammarBytecode
    ) -> Result<VerificationResult, GVMError>;

    /// Get GVM version information.
    fn version() -> GVMVersion;
}
```

### 12.2 Input Type

```
type GVMInput = {
    bytecode: GrammarBytecode,                          // From MOD-09
    config: {
        max_execution_steps: integer,                   // default: 100000
        max_memory: integer,                            // default: 67108864 (64 MiB)
        sandbox_mode: boolean,                          // default: true
    },
}
```

### 12.3 Output Types

```
type GVMVersion = {
    major: integer,
    minor: integer,
    patch: integer,
    supported_bytecode_versions: string[],               // e.g., ["1.0", "1.1"]
}

type VerificationResult = {
    valid: boolean,
    issues: {
        severity: "error" | "warning",
        code: string,
        message: string,
        offset: integer | null,
    }[],
}

type AnalysisFeature = {
    name: string,
    value: string,
    confidence: float,
    evidence: EvidenceEntry[],
}

type AnalysisToken = {
    index: integer,
    text: string,
    features: AnalysisFeature[],
    syntactic_role: SyntacticRole | null,
}

type AnalysisTree = {
    id: string,
    type: string,
    tokens: AnalysisToken[],
    constituents: GIRConstituent[],
    flags: GrammaticalFlag[],
    confidence: float,
}

type AnalysisResult = {
    version: string,
    metadata: {
        executed_at: string,
        execution_time_ms: float,
        steps_executed: integer,
        memory_used: integer,
        bytecode_size: integer,
    },
    input_text: string,
    input_text_hash: string,
    trees: AnalysisTree[],                             // One per successful parse
    flags: GrammaticalFlag[],
    evidence: EvidenceEntry[],
}
```

### 12.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `UNSUPPORTED_BYTECODE_VERSION` | Yes | Bytecode version exceeds GVM version |
| `BYTECODE_CORRUPTED` | Yes | Bytecode failed integrity check |
| `MAX_STEPS_EXCEEDED` | Yes | Execution exceeded step limit |
| `MAX_MEMORY_EXCEEDED` | Yes | Execution exceeded memory limit |
| `EXECUTION_FAILURE` | Yes | Unrecoverable execution error |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 12.5 Implementation Contract

1. **Bounded execution:** The GVM MUST guarantee that execution terminates within `max_execution_steps`. Infinite loops in bytecode MUST be detected and terminated.
2. **Memory safety:** The GVM MUST enforce memory boundaries. Bytecode that attempts to access memory outside its allocated region MUST fail with `MAX_MEMORY_EXCEEDED`.
3. **Deterministic replay:** Given the same bytecode and the same configuration, `execute()` MUST produce the same `AnalysisResult` across all runs.
4. **Side-effect free:** The GVM MUST NOT produce any side effects (file I/O, network calls, etc.) during execution. All output is through the returned `AnalysisResult`.

---

## 13. MOD-11: ExplanationEngine Interface

### 13.1 Trait Definition

```
trait ExplanationEngine {
    /// Transform an AnalysisResult into human-readable explanations.
    fn explain(
        input: ExplanationEngineInput
    ) -> Result<ExplanationOutput, ExplanationEngineError>;

    /// Get supported output formats.
    fn supported_formats() -> string[];

    /// Get supported languages.
    fn supported_languages() -> string[];
}
```

### 13.2 Input Type

```
type ExplanationEngineInput = {
    analysis: AnalysisResult,                            // From MOD-10
    config: {
        language: string,                                // default: "en"
        format: "text" | "html" | "json" | "pdf",       // default: "json"
        include_evidence: boolean,                       // default: false
        include_flags: boolean,                          // default: true
        enable_llm: boolean,                             // default: false
        llm_config: {
            model: string | null,
            temperature: float | null,
            max_tokens: integer | null,
        } | null,
    },
}
```

### 13.3 Output Type

```
type IrabEntry = {
    token: string,
    root: string | null,
    pos: string,
    features: { name: string, value: string }[],
    syntactic_role: string | null,
    explanation: string,                                 // Localized
}

type ExplanationOutput = {
    metadata: {
        generated_at: string,
        language: string,
        format: string,
        llm_enhanced: boolean,
        generation_time_ms: float,
    },
    input_text: string,
    overview: string,                                    // Summary of the grammatical analysis
    sentence_type: string | null,
    irab_breakdown: IrabEntry[],                         // Word-by-word I'rab
    constructions: {                                     // Notable grammatical constructions
        name: string,
        description: string,
        tokens: integer[],
    }[],
    flags: {
        type: string,
        message: string,
        tokens: integer[],
    }[],
    evidence: EvidenceEntry[],                           // If include_evidence == true
    raw: string,                                         // Formatted according to 'format'
}
```

### 13.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `UNSUPPORTED_LANGUAGE` | Yes | Requested language is not supported |
| `UNSUPPORTED_FORMAT` | Yes | Requested output format is not supported |
| `LLM_SERVICE_UNAVAILABLE` | No | LLM enhancement requested but service unavailable |
| `INTERNAL_ERROR` | Yes | Unexpected failure |

### 13.5 Implementation Contract

1. **LLM NEVER modifies analysis:** When LLM enhancement is enabled, the LLM receives the `AnalysisResult` and generates explanatory text. The LLM MUST NOT modify or override any part of the deterministic analysis.
2. **Template-based fallback:** If LLM enhancement is unavailable, the engine MUST produce explanations using built-in templates. LLM enhancement is additive, not required.
3. **I'rab completeness:** Every token MUST have an `IrabEntry` in the `irab_breakdown` array, even if the analysis is "unknown."

---

## 14. MOD-12: PluginLoader Interface

### 14.1 Trait Definition

```
trait PluginLoader {
    /// Discover plugins from configured directories.
    fn discover_plugins(
        config: PluginLoaderConfig
    ) -> Result<PluginManifest[], PluginLoaderError>;

    /// Load a specific plugin by its manifest.
    fn load_plugin(
        manifest: PluginManifest
    ) -> Result<PluginInstance, PluginLoaderError>;

    /// Unload a plugin, releasing its resources.
    fn unload_plugin(
        plugin_id: string
    ) -> Result<void, PluginLoaderError>;

    /// Get loaded plugin information.
    fn loaded_plugins() -> PluginManifest[];
}

trait PluginInstance {
    fn plugin_id() -> string;
    fn plugin_type() -> PluginType;
    fn process(input: any, context: PipelineContext) -> Result<any, PipelineError>;
}

type PluginType =
    "normalizer" | "token_classifier" | "segmenter" |
    "morphology_engine" | "syntax_engine" |
    "rule_set" | "kb_resolver" | "explanation" | "api_middleware"

type PipelineContext = {
    stage_id: string,
    text: string,
    knowledge_versions: KnowledgeVersionMap,
    config: { [key: string]: any },
}
```

### 14.2 Input Type

```
type PluginLoaderConfig = {
    plugin_directories: string[],                        // default: ["./plugins"]
    allowed_plugin_types: PluginType[],                  // default: all
    sandbox_enabled: boolean,                            // default: true
    max_plugins_per_type: integer,                       // default: 10
}
```

### 14.3 Output Type

```
type PluginManifest = {
    id: string,                                          // Unique plugin ID
    name: string,                                        // Human-readable name
    version: string,                                     // Semver
    plugin_type: PluginType,
    author: string,
    description: string,
    homepage: string | null,
    entry_point: string,                                 // File path to plugin binary/script
    api_version: string,                                 // AGOS API version required
    dependencies: string[],                              // Other plugin IDs
    permissions: string[],                               // e.g., ["read_kb", "write_cache"]
    config_schema: { [key: string]: any } | null,        // JSON Schema for plugin config
}
```

### 14.4 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `PLUGIN_NOT_FOUND` | Yes | Plugin manifest file not found |
| `PLUGIN_INVALID_MANIFEST` | Yes | Manifest failed schema validation |
| `PLUGIN_VERSION_MISMATCH` | Yes | Plugin requires a different AGOS API version |
| `PLUGIN_DEPENDENCY_MISSING` | Yes | Plugin dependency not satisfied |
| `PLUGIN_LOAD_FAILED` | Yes | Plugin failed to load |
| `PLUGIN_SANDBOX_VIOLATION` | Yes | Plugin attempted a disallowed operation |

### 14.5 Implementation Contract

1. **Sandbox isolation:** All plugins MUST run in a sandboxed environment with limited system access. The sandbox prevents file I/O, network access, and system call execution unless explicitly permitted in the manifest.
2. **Lifecycle management:** The PluginLoader manages the full plugin lifecycle: discover → validate → load → inject → unload. Plugins can be hot-reloaded without restarting the pipeline.
3. **Dependency resolution:** Plugins MAY declare dependencies on other plugins. The PluginLoader MUST resolve dependencies before loading and detect circular dependencies.

---

## 15. MOD-13: CacheManager Interface

### 15.1 Trait Definition

```
trait CacheManager {
    /// Get cached output for a stage.
    fn get(
        key: CacheKey
    ) -> Result<CacheEntry | null, CacheManagerError>;

    /// Store output for a stage.
    fn set(
        key: CacheKey,
        value: CacheEntry,
        ttl_seconds: integer | null
    ) -> Result<void, CacheManagerError>;

    /// Invalidate cache entries matching a filter.
    fn invalidate(
        filter: CacheInvalidationFilter
    ) -> Result<integer, CacheManagerError>;              // Returns count invalidated

    /// Get cache statistics.
    fn stats() -> CacheStats;

    /// Clear all cache entries.
    fn clear() -> Result<void, CacheManagerError>;
}
```

### 15.2 Types

```
type CacheKey = {
    input_hash: string,                                  // SHA-256 of stage input
    stage_id: string,                                    // e.g., "MOD-04"
    pipeline_config_hash: string,                        // Hash of pipeline configuration
    knowledge_version_hash: string,                      // Hash of KB versions used
}

type CacheEntry = {
    output: Uint8Array,                                  // Serialized stage output
    created_at: string,                                  // ISO 8601
    expires_at: string | null,                           // ISO 8601 (null = no expiry)
    size_bytes: integer,
    access_count: integer,
}

type CacheInvalidationFilter = {
    stage_id: string | null,                             // Invalidate specific stage
    knowledge_base_id: string | null,                    // Invalidate if KB version changed
    created_before: string | null,                       // Invalidate entries older than this
}

type CacheStats = {
    total_entries: integer,
    total_size_bytes: integer,
    hit_count: integer,
    miss_count: integer,
    hit_rate: float,                                     // 0.0 to 1.0
    eviction_count: integer,
    backend_type: "memory" | "redis" | "filesystem" | "database",
}
```

### 15.3 Error Codes

| Code | Fatal | Description |
|------|-------|-------------|
| `BACKEND_UNAVAILABLE` | No | Cache backend is temporarily unavailable |
| `SERIALIZATION_FAILED` | Yes | Failed to serialize or deserialize cache entry |
| `STORAGE_FULL` | No | Cache backend storage is full |

### 15.4 Implementation Contract

1. **TTL-based expiration:** Every cache entry MAY have a TTL. Expired entries MUST be treated as cache misses and eventually evicted.
2. **Automatic invalidation:** When a knowledge base version changes, all cache entries that depend on that KB MUST be automatically invalidated.
3. **Thread safety:** The CacheManager MUST be thread-safe. Concurrent reads and writes MUST not produce race conditions.
4. **Backend abstraction:** Multiple backends are supported: in-memory (default), Redis, filesystem, database. The backend is selected at configuration time and MUST NOT affect cache semantics.

---

## 16. MOD-14: APIGateway Interface

### 16.1 Trait Definition

```
trait APIGateway {
    /// Analyze a single text input.
    fn analyze(
        request: AnalyzeRequest
    ) -> Result<AnalyzeResponse, APIError>;

    /// Analyze multiple texts in batch.
    fn analyze_batch(
        request: BatchAnalyzeRequest
    ) -> Result<BatchAnalyzeResponse, APIError>;

    /// Query knowledge bases.
    fn query_knowledge_base(
        request: KBQueryRequest
    ) -> Result<KBQueryResponse, APIError>;

    /// Get pipeline health and status.
    fn health() -> HealthResponse;
}
```

### 16.2 API Types

```
type AnalyzeRequest = {
    text: string,                                        // Arabic text to analyze
    config: {
        school: string | null,                           // default: "basra"
        pipeline_mode: "full" | "morphology-only" | "tokenization-only" | null,
        explanation_language: string | null,              // default: "en"
        explanation_format: "text" | "html" | "json" | null,
        include_evidence: boolean | null,                 // default: false
        enable_llm: boolean | null,                       // default: false
    },
}

type AnalyzeResponse = {
    request_id: string,
    status: "completed" | "partial" | "error",
    result: ExplanationOutput | null,
    error: APIError | null,
    timing_ms: {
        total: float,
        validation: float,
        lexing: float,
        tokenization: float,
        morphology: float,
        syntax: float,
        gir_construction: float,
        rule_engine: float,
        kg_resolution: float,
        bytecode_generation: float,
        gvm_execution: float,
        explanation: float,
    },
}

type BatchAnalyzeRequest = {
    texts: string[],
    config: AnalyzeRequest.config,                       // Shared across all texts
}

type BatchAnalyzeResponse = {
    request_id: string,
    status: "completed" | "partial" | "error",
    results: AnalyzeResponse[],
    summary: {
        total: integer,
        succeeded: integer,
        failed: integer,
        total_time_ms: float,
    },
}

type HealthResponse = {
    status: "healthy" | "degraded" | "unhealthy",
    version: string,
    uptime_seconds: float,
    active_requests: integer,
    kb_loaded: string[],
    schools_available: string[],
    cache_stats: CacheStats | null,
}
```

### 16.3 HTTP API Endpoints

The following REST endpoints MUST be implemented:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/v1/analyze` | Single text analysis |
| `POST` | `/v1/analyze/batch` | Batch text analysis |
| `GET` | `/v1/kb/{kb_id}/query` | Knowledge base query |
| `GET` | `/v1/health` | Health check |
| `GET` | `/v1/version` | API version information |

Additionally, gRPC equivalents SHOULD be provided:

| RPC | Request Type | Response Type |
|-----|-------------|---------------|
| `Analyze` | `AnalyzeRequest` | `AnalyzeResponse` |
| `AnalyzeBatch` | `BatchAnalyzeRequest` | `BatchAnalyzeResponse` |
| `QueryKB` | `KBQueryRequest` | `KBQueryResponse` |
| `Health` | `Empty` | `HealthResponse` |

### 16.4 Error Codes

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `INVALID_REQUEST` | Malformed request body |
| 400 | `UNSUPPORTED_SCHOOL` | Requested grammar school not available |
| 400 | `UNSUPPORTED_LANGUAGE` | Requested explanation language not available |
| 413 | `TEXT_TOO_LONG` | Input text exceeds maximum length |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Unexpected server error |
| 503 | `SERVICE_UNAVAILABLE` | Pipeline not ready |

### 16.5 Implementation Contract

1. **Idempotent POST:** POST `/v1/analyze` with the same request body MUST produce the same response (assuming the same pipeline configuration). This allows safe retries.
2. **Request ID tracing:** Every API response MUST include a unique `request_id` that can be used for tracing and debugging across all pipeline stages.
3. **Per-request configuration override:** API requests MAY override the default pipeline configuration on a per-request basis (school, pipeline mode, etc.).

---

## 17. Pipeline Orchestrator Interface

### 17.1 Purpose

The Pipeline Orchestrator is not a separate module, but a coordination layer that composes all 14 modules into a working pipeline. It is the **public API** of the compilation and runtime layers.

### 17.2 Trait Definition

```
trait PipelineOrchestrator {
    /// Execute the full analysis pipeline.
    fn analyze(
        request: AnalyzeRequest
    ) -> Result<AnalyzeResponse, PipelineError>;

    /// Execute a mini-pipeline (short circuit).
    fn analyze_partial(
        request: AnalyzeRequest,
        mode: PipelineMode
    ) -> Result<any, PipelineError>;

    /// Initialize the pipeline (load KBs, plugins, etc.).
    fn initialize(
        config: OrchestratorConfig
    ) -> Result<void, PipelineError>;

    /// Reload knowledge bases and invalidate cache.
    fn reload_knowledge() -> Result<void, PipelineError>;

    /// Gracefully shut down the pipeline.
    fn shutdown() -> Result<void, PipelineError>;
}
```

### 17.3 Pipeline Wiring

The Orchestrator wires modules together in the following sequence. Each module's output is fed as input to the next:

```
function execute_full_pipeline(request: AnalyzeRequest):
    // Stage 1: UnicodeValidator
    validated = MOD-01.validate({
        raw_text: request.text,
        config: request.config,
    })
    if validated.is_error: return error

    // Stage 2: Lexer
    tokens = MOD-02.lex({ normalized_text: validated.value.normalized_text })
    if tokens.is_error: return error

    // Stage 3: Tokenizer (check cache first)
    segmented = cache.get_or_compute(
        key = cache_key(validated.value, tokens.value),
        compute_fn = () => MOD-03.tokenize({ token_stream: tokens.value })
    )

    // Stage 4: MorphologicalParser
    morphology = MOD-04.analyze_morphology({
        segmented_stream: segmented.value,
        config: { school: request.config.school }
    })

    // Stage 5: SyntaxParser
    syntax = MOD-05.parse_syntax({ morphology: morphology.value })

    // Stage 6: GIRConstructor
    gir = MOD-06.construct_gir({
        morphology: morphology.value,
        syntax: syntax.value,
        original_text: request.text,
    })

    // Stage 7: RuleEngine
    annotated_gir = MOD-07.apply_rules({
        gir: gir.value,
        config: { school: request.config.school }
    })

    // Stage 8: KnowledgeGraphResolver
    resolved_gir = MOD-08.resolve({ gir: annotated_gir.value })

    // Stage 9: BytecodeGenerator
    bytecode = MOD-09.generate_bytecode({ gir: resolved_gir.value })

    // Stage 10: GVM
    analysis = MOD-10.execute({ bytecode: bytecode.value })

    // Stage 11: ExplanationEngine
    explanation = MOD-11.explain({
        analysis: analysis.value,
        config: { language: request.config.explanation_language }
    })

    return explanation
```

### 17.4 Error Propagation Logic

```
function handle_stage_error(error: PipelineError, stage: string):
    if error.is_fatal:
        abort pipeline
        return error
    else:
        annotate result with error
        continue pipeline with degraded state
```

---

## 18. Versioning & Compatibility Policy

### 18.1 Semantic Versioning for All Interfaces

All AGOS interfaces use [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version increment: Breaking interface changes (field removed, type changed, required field added).
- **MINOR** version increment: Backward-compatible additions (new optional field, new error code, new output field).
- **PATCH** version increment: Bug fixes, documentation changes, performance improvements.

### 18.2 Interface Compatibility Rules

| Change Type | Examples | Semver Impact |
|-------------|----------|---------------|
| **Breaking** | Removing a field, changing a field type, adding a required field, removing an error code | MAJOR |
| **Additive** | Adding an optional field, adding a new error code, adding a new output field | MINOR |
| **Fixing** | Fixing a bug in implementation, updating documentation | PATCH |

### 18.3 Deprecation Policy

1. A deprecated interface element MUST be announced at least one MAJOR version before removal.
2. During the deprecation period, the element MUST continue to function.
3. After the deprecation period, the element MAY be removed in the next MAJOR version.
4. All deprecation notices MUST include: what is deprecated, what replaces it, and the timeline for removal.

### 18.4 Module Version Table

When all 14 modules are implemented, each module MUST report:

```
type ModuleVersion = {
    module_id: string,                  // e.g., "MOD-04"
    version: string,                    // Semver
    api_version: string,                // Interface version this implements
    dependencies: {                     // Versioned dependencies
        module_id: string,
        min_version: string,
    }[],
}
```

---

## 19. Cross-References

### 19.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | Module catalog with IDs, layers, and high-level responsibilities |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | Detailed algorithm specifications for MOD-01 through MOD-09 |
| SPEC-0101 | Morphology Engine | Detailed implementation of MOD-04 and MOD-05 |
| SPEC-0201 | Rule Engine | Detailed implementation of MOD-07 and DSL semantics |
| SPEC-0301 | Grammar Runtime | Detailed implementation of MOD-10 and MOD-11 |
| SPEC-0401 | Knowledge Graph Engine | Detailed implementation of MOD-08 |
| SPEC-0501 | Explanation Engine | Detailed implementation of MOD-11 |
| SPEC-0601 | Plugin System | Detailed implementation of MOD-12 |
| RFC-0002 | Grammar Bytecode | Bytecode format and opcode definitions |
| RFC-0003 | Grammar Virtual Machine | GVM execution model and instruction semantics |

### 19.2 External References

| Reference | Relevance |
|-----------|-----------|
| Rust Traits | Inspiration for trait-based interface design |
| Go Interfaces | Inspiration for minimal interface definitions |
| OpenAPI 3.0 | REST API documentation standard |
| gRPC Service Definitions | RPC interface standard |
| WebAssembly Interface Types | Component model inspiration for plugins |
| SemVer 2.0.0 | Versioning policy reference |

---

## Progress Summary

**SPEC-0001: Platform Architecture**

| Chapter | Title | Status |
|---------|-------|--------|
| Chapter 1 | Introduction and Scope | ✓ COMPLETE |
| Chapter 2 | System Architecture Overview | ✓ COMPLETE |
| Chapter 3 | Compilation Pipeline — Stage-by-Stage | ✓ COMPLETE |
| **Chapter 4** | **Module Responsibilities & Interfaces** | **✓ COMPLETE (this document)** |
| Chapter 5 | Data Flow & Intermediate Representations | Pending |
| Chapter 6 | Deployment & Runtime Considerations | Pending |
| Chapter 7 | Extensibility & Plugin Architecture | Pending |
| Chapter 8 | Security, Validation & Error Handling | Pending |
| Chapter 9 | Performance Targets & Constraints | Pending |

**Dependencies:** Chapters 1–3, ADR-0001, RFC-0001–0003, KB-0001–0007.

**Recommended Next Chapter:** Chapter 5 — Data Flow & Intermediate Representations, which will define the precise schemas for all IRs (TokenStream, GIR, Bytecode, AnalysisResult) and the data transformation that occurs at each pipeline boundary.
