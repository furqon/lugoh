---
spec_id: SPEC-0401
title: Knowledge Graph Engine
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage (MOD-08)
  - SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-08)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-8)
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0001-C8: Security, Validation & Error Handling
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0101: Morphology Engine
  - SPEC-0201: Rule Engine
  - ADR-0003: Why Grammar Intermediate Representation (GIR)
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features Taxonomy
---

# SPEC-0401: Knowledge Graph Engine

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Internal Component Model](#3-internal-component-model)
4. [KB Index Loading & Lifecycle](#4-kb-index-loading--lifecycle)
5. [Root Resolution](#5-root-resolution)
6. [Wazan & Pattern Resolution](#6-wazan--pattern-resolution)
7. [Dictionary & Lexical Enrichment](#7-dictionary--lexical-enrichment)
8. [Semantic & Etymological Enrichment](#8-semantic--etymological-enrichment)
9. [Cross-Referencing & Relationship Resolution](#9-cross-referencing--relationship-resolution)
10. [Resolution Aggregation & Statistics](#10-resolution-aggregation--statistics)
11. [Performance & Optimization](#11-performance--optimization)
12. [Plugin Integration](#12-plugin-integration)
13. [Testing Strategy](#13-testing-strategy)
14. [Implementation Guidance](#14-implementation-guidance)
15. [Cross-References](#15-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0401 defines the **Knowledge Graph Engine (AGOS-MOD-08)** — the module responsible for resolving all references in the annotated Grammar Intermediate Representation (GIR) against the AGOS knowledge bases. It transforms an `AnnotatedGIR` (IR-7) into a `ResolvedGIR` (IR-8) by enriching every token with linked linguistic data: full root definitions, wazan paradigms, dictionary entries, semantic tags, and cross-references.

The Knowledge Graph Engine is the **bridge between abstract grammatical analysis and concrete lexical knowledge**. Where MOD-04 (MorphologicalParser) identifies roots and patterns by their identifiers, MOD-08 resolves those identifiers into the full KB entries, enabling downstream stages (MOD-09 BytecodeGenerator, MOD-11 ExplanationEngine) to produce rich, contextualized output.

### 1.2 Scope

**In scope:**
- KB index loading, caching, and version management
- Root entry resolution against KB-0001
- Wazan and pattern resolution against KB-0002
- Noun pattern resolution against KB-0004
- Dictionary lookups (optional KB integration)
- Semantic tag assignment from KB-0007 and root semantic fields
- Etymological enrichment (optional)
- Cross-reference resolution (synonyms, antonyms, related roots)
- Resolution statistics and reporting
- Plugin interfaces for custom KB resolvers
- Graceful degradation when KBs are unavailable

**Out of scope:**
- Root extraction and wazan identification (handled by MOD-04 / SPEC-0101)
- Grammatical rule application (handled by MOD-07 / SPEC-0201)
- Bytecode generation (handled by MOD-09 / RFC-0002)
- KB source data compilation (handled by KB Compiler tooling)
- Natural language explanation generation (handled by MOD-11 / SPEC-0501)

### 1.3 Relationship to Other Specifications

| Specification | Relationship |
|---------------|--------------|
| SPEC-0001-C3 | Defines the high-level MOD-08 pipeline algorithm (§9) which SPEC-0401 implements in detail |
| SPEC-0001-C4 | Defines the MOD-08 public interface (`KnowledgeGraphResolver.resolve()`) |
| SPEC-0001-C5 | Defines the IR-8 (ResolvedGIR) schema that MOD-08 produces |
| SPEC-0001-C7 | Defines the `kb_resolver` plugin type that MOD-08 hosts |
| SPEC-0001-C8 | Defines error handling and validation policies that MOD-08 must follow |
| SPEC-0001-C9 | Defines MOD-08 performance targets (p50 < 50 μs/token, p99 < 500 μs/token) |
| SPEC-0101 | Defines MOD-04 which produces the root/wazan references MOD-08 resolves |
| SPEC-0201 | Defines MOD-07 which produces the AnnotatedGIR that MOD-08 consumes |
| ADR-0003 | Justifies the GIR boundary that makes MOD-08's role possible |
| KB-0001–0007 | Authoritative knowledge data that MOD-08 resolves against |

### 1.4 Knowledge Dependencies

| Knowledge Base | Versioned | Resolves | Primary Use |
|----------------|-----------|----------|-------------|
| KB-0001: Roots | Yes (semver) | Root references | Root definitions, semantic fields, cross-references |
| KB-0002: Wazan | Yes (semver) | Pattern references | Pattern descriptions, verb forms, noun types |
| KB-0003: Verb Forms | Yes (semver) | Verb paradigms | Conjugation templates, weak root variants |
| KB-0004: Noun Patterns | Yes (semver) | Noun pattern refs | Inflection tables, broken plural mappings |
| KB-0005: Particles | Yes (semver) | Particle references | Particle meanings, grammatical effects |
| KB-0006: Pronouns | Yes (semver) | Pronoun references | Pronoun feature tables, attachment rules |
| KB-0007: Features | Yes (semver) | Feature values | Feature descriptions, allowed values |

### 1.5 Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **KB access pattern** | Hash-based index, memory-mapped files | O(1) average lookup meets performance targets; memory-mapping avoids loading entire KBs into process memory |
| **Resolution depth** | Configurable (default: 3 levels) | Controls how deep cross-references are followed (level 1 = direct, level 2 = direct + immediate relatives, level 3 = full transitive closure) |
| **Graceful degradation** | Continue with available KBs, record missing | Pipeline must not fail entirely when optional KBs are unavailable |
| **No hallucination** | Unresolved references are explicitly null | Core principle: the resolver must never fabricate data |
| **Plugin extension** | Custom KB resolvers via plugin system | Enables third-party dictionaries, specialized corpora, and research data |
| **Semantic enrichment** | Optional (configurable) | Semantic tagging adds value for educational use but is not required for core grammatical analysis |

---

## 2. Architecture Overview

### 2.1 Module Position in Pipeline

```
Input: AnnotatedGIR (IR-7) from MOD-07
    │
    ▼
┌─────────────────────────────────────────────┐
│        MOD-08: KnowledgeGraphResolver        │
│                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  KB      │  │ Reference│  │ Semantic │  │
│  │  Index   │→│ Resolver │→│ Enricher │  │
│  │  Loader  │  │ Engine   │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│                      │                      │
│  ┌──────────────────────────────────────┐   │
│  │       Cross-Reference Engine         │   │
│  │  (synonyms, antonyms, related roots) │   │
│  └──────────────────────────────────────┘   │
│                      │                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Stats   │  │  Plugin  │  │  Output  │  │
│  │  Collector│→│ Host     │→│ Assembler│  │
│  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────┘
    │
    ▼
Output: ResolvedGIR (IR-8) to MOD-09
```

### 2.2 Internal Architecture

The KnowledgeGraphResolver comprises **four internal subsystems**:

#### 2.2.1 KB Index Loader

Responsible for loading, validating, and caching KB indices at initialization time:

| Component | Responsibility |
|-----------|---------------|
| **Index Builder** | Loads compiled KB binaries into memory-mapped hash indices |
| **Version Validator** | Verifies KB versions are compatible with the configured school |
| **Cache Manager** | Maintains hot/cold cache for frequently accessed KB entries |
| **Lazy Loader** | Loads optional KBs (dictionary, etymology) on first access |

#### 2.2.2 Reference Resolver Engine

The core resolution logic that walks the AnnotatedGIR and resolves each reference:

| Component | Responsibility |
|-----------|---------------|
| **Root Resolver** | Resolves root text → KB-0001 entry (definition, forms, cognates) |
| **Wazan Resolver** | Resolves pattern ID → KB-0002 entry (pattern description, examples) |
| **Noun Pattern Resolver** | Resolves noun pattern → KB-0004 entry (inflection, broken plurals) |
| **Particle/Pronoun Resolver** | Resolves particle/pronoun → KB-0005/0006 entries |
| **Feature Resolver** | Resolves feature values → KB-0007 descriptions |

#### 2.2.3 Semantic Enricher

Optional subsystem that adds semantic and etymological layers:

| Component | Responsibility |
|-----------|---------------|
| **Semantic Tagger** | Assigns semantic tags from root properties and derived noun categories |
| **Etymology Engine** | Resolves etymological origins (optional, future: Quranic Aramaic) |
| **Field Classifier** | Groups entries by semantic field (e.g., "writing", "religion", "commerce") |

#### 2.2.4 Cross-Reference Engine

Resolves relationships between KB entries:

| Component | Responsibility |
|-----------|---------------|
| **Synonym Resolver** | Looks up synonym references in KB-0001 |
| **Antonym Resolver** | Looks up antonym references |
| **Related Root Resolver** | Resolves cross-referenced related roots |
| **Cognate Resolver** | Resolves cognate relationships across root entries |

### 2.3 Data Flow Diagram

```
IR-7: AnnotatedGIR                    KB-0001..0007 (compiled binaries)
    │                                      │
    │  1. Extract token references         │
    │     (root, wazan, feature IDs)       │
    │                                      │
    ▼                                      ▼
┌──────────────────────────────────────────────┐
│          Reference Resolver Engine            │
│                                              │
│  For each token[analysis] in GIR:            │
│                                              │
│  1. If root.text is present:                 │
│     → hash_lookup(KB-0001, root.text)        │
│     → attach root_entry                      │
│                                              │
│  2. If wazan.text or wazan.form is present:  │
│     → hash_lookup(KB-0002, wazan_id)         │
│     → attach wazan_entry                     │
│                                              │
│  3. If noun pattern is present:              │
│     → hash_lookup(KB-0004, pattern_id)       │
│     → attach noun_entry                      │
│                                              │
│  4. If particle_id is present:               │
│     → hash_lookup(KB-0005, particle_id)      │
│     → attach particle_entry                  │
│                                              │
│  5. If pronoun_id is present:                │
│     → hash_lookup(KB-0006, pronoun_id)       │
│     → attach pronoun_entry                   │
│                                              │
│  6. For each feature[analysis]:             │
│     → hash_lookup(KB-0007, feature_name)     │
│     → attach feature_description             │
└──────────────────────────────────────────────┘
    │
    ▼
IR-8: ResolvedGIR
    ├── tokens[].root_entry       (KB-0001 data)
    ├── tokens[].wazan_entry      (KB-0002 data)
    ├── tokens[].noun_entry       (KB-0004 data)
    ├── tokens[].particle_entry   (KB-0005 data)
    ├── tokens[].pronoun_entry    (KB-0006 data)
    ├── tokens[].feature_metadata (KB-0007 data)
    ├── tokens[].semantic_tags    (derived)
    ├── tokens[].cross_references (resolved)
    └── resolution_stats          (aggregate metrics)
```

### 2.4 Resolution Depth Model

The resolver supports configurable resolution depth that controls how deep cross-references are followed:

```yaml
Depth Levels:
  Level 0: "No resolution" — only direct ID-to-entry mapping (fastest)
    - root.text → entry.id, root.meaning only
  
  Level 1: "Direct resolution" (default minimum)
    - Full root_entry: meaning, forms, derived_nouns
    - Full wazan_entry: pattern, meaning, form, example
  
  Level 2: "Full resolution" (default)
    - All Level 1 data
    - Plus: cognates, semantic_field, cross_references
    - Plus: feature descriptions from KB-0007
  
  Level 3: "Deep resolution" (max)
    - All Level 2 data
    - Plus: resolve cross-references transitively (up to 1 hop)
    - Plus: optional etymological data
    - Plus: dictionary entries (if available)
```

### 2.5 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Hash-based indexing (primary)** | O(1) average lookup time; minimal CPU overhead per token |
| **Memory-mapped KB binaries** | KBs are shared across processes; no loading overhead; pages are demand-paged |
| **Configurable resolution depth** | Depth 1 is sufficient for bytecode generation; Depth 3 is needed for explanation engine |
| **Stateless resolution** | Given the same GIR and KB versions, resolution is fully deterministic |
| **Plugin-first dictionary** | Dictionary sources are plugins, not core KBs — enabling community contributions |
| **Aggregated stats** | Resolution statistics enable monitoring of KB coverage and quality |

---

## 3. Internal Component Model

### 3.1 Core Data Structures

#### 3.1.1 KBIndex (Internal)

```rust
type KBIndex<T> = {
    /// Hash table mapping key → entry
    table: HashMap<u64, Vec<KBEntryRef>>,
    /// Memory-mapped region backing the index
    mmap_region: MmapRegion,
    /// Entry count
    entry_count: u32,
    /// Bit flags for entry types present
    flags: u32,
    /// KB version this index was built from
    kb_version: String,
}

type KBEntryRef = {
    /// Absolute offset in mmap_region
    offset: u32,
    /// Entry byte length
    length: u32,
    /// Entry type discriminator (e.g., root, wazan, particle)
    entry_type: u16,
}
```

#### 3.1.2 ResolvedTokenData (Internal enrichment)

> **IR-8 mapping note:** The fields below extend the IR-8 `ResolvedToken` schema (C5 §9.3). The IR-8 schema defines `root_entry`, `wazan_entry`, `dictionary_entry`, and `semantic_tags` as standard fields. Additional fields (`noun_entry`, `particle_entry`, `pronoun_entry`, `feature_metadata`, `cross_references`) are internal enrichment structures that get mapped to the ResolvedGIR output as follows:
> - `noun_entry` → subsumed under `wazan_entry` (noun pattern data is part of the wazan record)
> - `particle_entry` → stored as a typed extension of `dictionary_entry`
> - `pronoun_entry` → stored as a typed extension of `dictionary_entry`
> - `feature_metadata` → embedded in each token's feature array as `description` fields
> - `cross_references` → stored in `root_entry.cross_references`

```rust
type ResolvedTokenData = {
    /// KB-0001 root entry (if root was resolved)
    root_entry: ResolvedRootEntry | null,
    /// KB-0002 wazan entry (if wazan was resolved)
    wazan_entry: ResolvedWazanEntry | null,
    /// KB-0004 noun pattern entry (if noun type was resolved)
    noun_entry: ResolvedNounEntry | null,
    /// KB-0005 particle entry (if particle was resolved)
    particle_entry: ResolvedParticleEntry | null,
    /// KB-0006 pronoun entry (if pronoun was resolved)
    pronoun_entry: ResolvedPronounEntry | null,
    /// KB-0007 feature metadata per feature
    feature_metadata: HashMap<String, FeatureMeta>,
    /// Semantic tags (from root properties or KB-0007)
    semantic_tags: Vec<String>,
    /// Dictionary entry (optional, from plugin)
    dictionary_entry: DictionaryEntry | null,
    /// Cross-references (resolved from KB-0001)
    cross_references: CrossReferences | null,
    /// Whether resolution was successful
    resolved: bool,
    /// Resolution timestamp
    resolved_at: u64,
}
```

#### 3.1.3 ResolvedRootEntry

```rust
type ResolvedRootEntry = {
    /// KB-0001 entry identifier
    id: String,
    /// The root consonants (e.g., "كتب")
    root: String,
    /// Core meaning (e.g., "to write")
    meaning: String,
    /// Root type: sound, weak, hamzated, doubled, quadriliteral
    root_type: String,
    /// Verb forms this root appears in (I–XV)
    forms: Vec<u8>,
    /// Derived noun patterns from this root
    derived_nouns: Vec<DerivedNounRef>,
    /// Cognate roots
    cognates: Vec<CognateRef>,
    /// Semantic field classification
    semantic_field: String | null,
    /// Transitivity: transitive / intransitive / both
    transitivity: String | null,
    /// Cross-references
    cross_references: RefSet,
    /// Attestation level
    attestation: "classical" | "quranic" | "msa" | "rare" | "disputed",
    /// Alternative root representations
    variants: Vec<String>,
    /// Part of speech of derived nouns from this root
    derived_meanings: Vec<String>,
}

type DerivedNounRef = {
    /// Pattern identifier (KB-0002 reference)
    wazan_id: String,
    /// Noun type (masdar, ism fa'il, ism maf'ul, etc.)
    noun_type: String,
    /// Meaning of this derived form
    meaning: String,
}

type CognateRef = {
    /// Related root text
    root: String,
    /// Relationship type
    relationship: "meaning_overlap" | "etymological" | "morphological",
    /// Strength of relationship (0.0–1.0)
    strength: f32,
}

type RefSet = {
    related_roots: Vec<RelatedRoot>,
    antonyms: Vec<String>,
    synonyms: Vec<String>,
}

type RelatedRoot = {
    root: String,
    relationship: String,     // e.g., "semantic_field", "derivational"
    description: String | null,
}
```

#### 3.1.4 ResolvedWazanEntry

```rust
type ResolvedWazanEntry = {
    /// KB-0002 entry identifier
    id: String,
    /// Pattern template (e.g., "فَعَلَ")
    pattern: String,
    /// Meaning/description of this pattern
    meaning: String,
    /// Verb form number (I–XV) or null for noun patterns
    form: u8 | null,
    /// Category: verb, noun, adjective, etc.
    category: String,
    /// Example word using this pattern
    example: String,
    /// Morphological features this pattern conveys
    features: HashMap<String, String>,
    /// Weak root variant template (if applicable)
    weak_variant: WeakVariantTemplate | null,
    /// Related patterns
    related_patterns: Vec<String>,

    /// Example inflections
    inflection_examples: Vec<InflectionExample> | null,
}

type WeakVariantTemplate = {
    /// Original root type this variant applies to
    root_type: String,
    /// Adjusted pattern
    adjusted_pattern: String,
    /// Rule description
    rule: String,
}

type InflectionExample = {
    /// Inflected form text
    form: String,
    /// Feature combination
    features: HashMap<String, String>,
    /// Example in a sentence
    example_sentence: String | null,
}
```

#### 3.1.5 ResolvedNounEntry

```rust
type ResolvedNounEntry = {
    /// KB-0004 entry identifier
    id: String,
    /// Pattern identifier
    pattern_id: String,
    /// Noun type: masdar, ism fa'il, ism makan, etc.
    noun_type: String,
    /// Gender: masculine / feminine / both
    gender: String,
    /// Declension class
    declension: "triptote" | "diptote" | "indeclinable",
    /// Sound plural forms
    sound_plural: SoundPlural | null,
    /// Broken plural mappings
    broken_plurals: Vec<BrokenPluralMapping>,
    /// Example
    example: String,
}

type SoundPlural = {
    masculine: String | null,    // e.g., "كاتبون"
    feminine: String | null,     // e.g., "كاتبات"
}

type BrokenPluralMapping = {
    /// Plural pattern
    pattern: String,
    /// Frequency: primary / secondary / rare
    frequency: String,
    /// Example plural form
    example: String,
}
```

#### 3.1.6 ResolvedParticleEntry & ResolvedPronounEntry

```rust
type ResolvedParticleEntry = {
    id: String,
    /// Particle text
    text: String,
    /// Particle category (harf_jarr, harf_nasb, harf_jazm, conjunction, etc.)
    category: String,
    /// Grammatical effect
    grammatical_effect: String | null,
    /// Meaning/translation
    meaning: String,
    /// Usage notes
    usage_notes: String | null,
}

type ResolvedPronounEntry = {
    id: String,
    /// Pronoun text
    text: String,
    /// Pronoun type: attached (mutasil) / detached (munfasil)
    pronoun_type: String,
    /// Person
    person: u8,
    /// Gender: masculine / feminine
    gender: String,
    /// Number: singular / dual / plural
    number: String,
    /// Case it takes (for detached pronouns)
    case: String | null,
    /// Attaches to (for attached pronouns): verb / noun / particle
    attaches_to: Vec<String>,
}
```

#### 3.1.7 KnowledgeGraphResolverConfig

```rust
type KnowledgeGraphResolverConfig = {
    /// Resolution depth level (0–3)
    resolve_depth: u8,                     // default: 3
    /// Enable semantic enrichment
    enable_semantic: bool,                 // default: true
    /// Enable etymological data
    enable_etymology: bool,                // default: false
    /// Max entries per reference type
    max_entries_per_reference: u16,        // default: 5
    /// Enable cross-reference resolution
    enable_cross_references: bool,         // default: true
    /// Max transitive depth for cross-references
    max_cross_ref_depth: u8,               // default: 1
    /// KB index cache size (bytes)
    kb_cache_size: u64,                    // default: 268435456 (256 MB)
    /// KB directories (overrides defaults)
    kb_directories: HashMap<String, String> | null,
    /// Plugin configuration
    plugin_config: PluginResolveConfig | null,
}

type PluginResolveConfig = {
    /// Ordered list of plugin IDs to invoke
    plugin_chain: Vec<String>,
    /// Timeout per plugin (ms)
    plugin_timeout_ms: u64,                // default: 50
    /// Fail on plugin error (vs. skip)
    strict_plugins: bool,                  // default: false
}
```

### 3.2 Index Architecture

#### 3.2.1 Primary Index Structure

Each compiled KB is loaded as an **immutable hash map** with the following properties:

```yaml
Index Properties:
  Hash function: xxHash3 (64-bit)
  Collision resolution: Robin Hood hashing with linear probing
  Load factor: < 0.7 (maintained during compilation)
  Key size: 8 bytes (u64 hash)
  Value: offset + length tuple into memory-mapped region
  Memory layout: Sequential, cache-line aligned entries
```

#### 3.2.2 KB Index Registry

```rust
type KBIndexRegistry = {
    /// Loaded indices, keyed by KB identifier
    indices: HashMap<String, KBIndex>,

    /// Version map of all loaded KBs
    versions: KnowledgeVersionMap,

    /// Configuration at load time
    config: LoadConfig,

    /// Whether all mandatory KBs are loaded
    mandatory_kbs_loaded: bool,

    /// List of missing optional KBs
    missing_kbs: Vec<String>,
}
```

#### 3.2.3 Index Key Derivation

Different reference types use different keys for hash lookups:

| Reference Type | Key Derivation | Example |
|----------------|----------------|---------|
| Root | Hash of root consonants (normalized) | `hash("كتب")` |
| Wazan | Hash of pattern ID | `hash("KB-0002:142")` |
| Pattern form | Hash of "form:" + form_number | `hash("form:IV")` |
| Noun pattern | Hash of pattern template | `hash("فَاعِل")` |
| Particle | Hash of particle text + type | `hash("مِن")` → `hash("مِن:preposition")` |
| Pronoun | Hash of pronoun text | `hash("أَنَا")` |
| Feature | Hash of "feature:" + feature_name | `hash("feature:gender")` |
| Feature value | Hash of feature_name + ":" + value | `hash("gender:masculine")` |

---

## 4. KB Index Loading & Lifecycle

### 4.1 Index Loading Algorithm

```rust
Algorithm: load_kb_indices
Input: config (KnowledgeGraphResolverConfig), kb_versions (KnowledgeVersionMap)
Output: KBIndexRegistry or KB_LOAD_FAILURE

Step 1: Resolve KB Paths
  1.1  For each mandatory KB (KB-0001 through KB-0007):
  1.1.1  Determine path from config.kb_directories or default paths
  1.1.2  Verify file exists and is readable
  1.1.3  If missing → KB_LOAD_FAILURE (mandatory KBs must exist)

  1.2  For each optional KB (dictionary, etymology, custom):
  1.2.1  If path is configured and file exists → load it
  1.2.2  If path is configured but file missing → log warning, mark as missing
  1.2.3  If path is not configured → skip silently

Step 2: Open Memory-Mapped Regions
  2.1  For each KB file:
  2.1.1  Open file descriptor
  2.1.2  Memory-map the file (read-only)
  2.1.3  Verify magic bytes match expected KB format
  2.1.4  Read header: version, entry count, index offset, index size
  2.1.5  Verify KB version compatibility with configured school

Step 3: Load Hash Index
  3.1  Locate the hash index section within the mmap region:
  3.1.1  Index base = mmap_base + header.index_offset
  3.1.2  Index entries = header.entry_count

  3.2  For each index entry:
  3.2.1  Read (hash_key, entry_offset, entry_length) tuple
  3.2.2  Store in in-memory HashMap<u64, Vec<KBEntryRef>>
         (Multiple entries may share the same hash key)

Step 4: Verify Index Integrity
  4.1  For a random sample of entries (configurable, default: 1%):
  4.1.1  Look up each sampled key
  4.1.2  Verify the returned entry parses correctly
  4.1.3  If > 5% of sampled entries fail → log warning

Step 5: Build Reverse Index (Optional)
  5.1  If configured, build reverse indices for:
  5.1.1  Derived nouns → root mapping
  5.1.2  Semantic field → root list
  5.1.3  Form number → wazan list

Step 6: Register Indices
  6.1  Store loaded indices in KBIndexRegistry
  6.2  Record KB versions
  6.3  Set mandatory_kbs_loaded = true

Step 7: Return
  7.1  Return KBIndexRegistry
```

### 4.2 Index Lifecycle

```yaml
Index Lifecycle States:
  UNLOADED     — KB not yet loaded
  LOADING      — File opened, mmap in progress
  ACTIVE       — Index loaded and ready for queries
  STALE        — KB version has changed; index needs reload
  ERROR        — Index corruption or version mismatch
  UNLOADED     — Resources released (terminal)
```

### 4.3 Hot Reload

When a KB version changes (detected by the Pipeline Orchestrator), indices are reloaded:

```rust
Algorithm: reload_kb_index
Input: kb_id (String), new_version (String)
Output: success / error

Step 1: Mark existing index as STALE
  1.1  In-flight queries continue using the stale index
  1.2  New queries block briefly during reload

Step 2: Load new index (parallel)
  2.1  Open new KB file
  2.2  Build new in-memory hash table
  2.3  Keep old mmap region alive until Step 3 completes

Step 3: Atomic Swap
  3.1  Replace old index in registry with new index
  3.2  Update version map
  3.3  Release old mmap region (OS will unmap on last reference)

Step 4: Invalidate Downstream Caches
  4.1  Notify CacheManager that KB-{kb_id} version changed
  4.2  Cache entries that depend on this KB are invalidated
```

### 4.4 Version Compatibility Check

```rust
Algorithm: check_kb_version_compatibility
Input: kb_version (String), school (String)
Output: compatible (bool), incompatibility_reason (String | null)

Step 1: Parse version
  1.1  Parse kb_version as semver (major.minor.patch)
  1.2  If parse fails → incompatible, "Invalid version string"

Step 2: Check against school requirements
  2.1  Each school declares required KB version ranges
  2.2  For the configured school:
  2.2.1  If major version differs from expected range → incompatible
  2.2.2  If minor version is lower than minimum → incompatible
  2.2.3  If minor version is higher than maximum → compatible (with warning)

Step 3: Return compatibility verdict
```

### 4.5 Mandatory vs. Optional KBs

| KB | Mandatory | Resolution If Missing |
|----|-----------|----------------------|
| KB-0001 (Roots) | Yes | Pipeline stops (FATAL) |
| KB-0002 (Wazan) | Yes | Pipeline stops (FATAL) |
| KB-0003 (Verb Forms) | Yes | Pipeline stops (FATAL) |
| KB-0004 (Noun Patterns) | Yes | Pipeline stops (FATAL) |
| KB-0005 (Particles) | Yes | Pipeline stops (FATAL) |
| KB-0006 (Pronouns) | Yes | Pipeline stops (FATAL) |
| KB-0007 (Features) | Yes | Pipeline stops (FATAL) |
| Dictionary | No | Root lookups still succeed; dictionary data is null |
| Etymology | No | Etymological data is omitted |
| Custom KB plugin | No | Plugin-specific data is unavailable |

---

## 5. Root Resolution

### 5.1 Purpose

Resolve every root reference in the AnnotatedGIR against KB-0001 (Roots Database). For each token that has a morphological analysis identifying a root, the Root Resolver retrieves the full root entry including its meaning, root type, derived forms, cognates, semantic field, and cross-references.

### 5.2 Resolution Algorithm

```rust
Algorithm: resolve_roots
Input:
  - tokens: Vec<GIRToken>           // From AnnotatedGIR
  - root_index: KBIndex             // KB-0001 loaded index
  - config: KnowledgeGraphResolverConfig
Output:
  - resolved_tokens: Vec<ResolvedTokenData>

Step 1: For each token t in tokens:
  1.1  If t does not have a root reference → skip, root_entry = null
  1.2  If t.root is null or empty → skip, root_entry = null

Step 2: Normalize root text
  2.1  Normalize root consonants:
  2.1.1  Remove any diacritics
  2.1.2  Normalize alif variants (آ, أ, إ → ا)
  2.1.3  Normalize alif maqsura (ى → ي) — roots stored with final ي in KB-0001
  2.1.4  Normalize ta-marbuta (ة → ه)
  2.1.5  Strip any trailing/leading non-consonant characters
  2.2  If normalized root is empty → root_entry = null, log warning

Step 3: Hash-based lookup
  3.1  Compute hash key:
  3.1.1  key = xxhash3(normalized_root)
  3.2  Look up key in root_index.table:
  3.2.1  If not found → return null (unresolved)
  3.2.2  If found → retrieve KBEntryRef(s) from the hash bucket
  3.2.3  (Normally only one entry per root, but some roots may have
         multiple entries for different senses)

Step 4: Deserialize Root Entry
  4.1  Read entry data from mmap_region at entry.offset
  4.2  Parse entry according to KB-0001 binary format:
  4.2.1  Entry header: id, root text, meaning, root_type
  4.2.2  Derived forms list (form numbers I–XV)
  4.2.3  Derived nouns list
  4.2.4  Cognates list
  4.2.5  Semantic field
  4.2.6  Cross-references (synonyms, antonyms, related roots)
  4.2.7  Attestation level
  4.2.8  Variant roots

Step 5: Build ResolvedRootEntry
  5.1  Construct ResolvedRootEntry from parsed data
  5.2  If multiple senses exist for the same root:
  5.2.1  Include all senses up to max_entries_per_reference
  5.2.2  Order by attestation level (quranic > classical > msa > rare)

Step 6: Resolution Depth Filtering
  6.1  If config.resolve_depth >= 2:
  6.1.1  Include cognates list
  6.1.2  Include cross_references
  6.1.3  Include derived_nouns with meanings
  6.2  If config.resolve_depth >= 3:
  6.2.1  Follow cross-references transitively (1 hop)
  6.2.2  Resolve referenced root entries
  6.2.3  Include etymological data (if config.enable_etymology)

Step 7: Assign to token
  7.1  Attach ResolvedRootEntry to token's resolved data
  7.2  Mark token as resolved

Step 8: Return resolved_tokens
```

### 5.3 Multiple Sense Handling

Some root entries in KB-0001 have multiple distinct meanings (polysemy):

```yaml
Example: Root "ع ي ن" (ayn)
  Sense 1: "eye" (body part)
    Semantic field: "body"
    Forms: I only
  Sense 2: "spring/water source"
    Semantic field: "geography"
    Forms: I only
  Sense 3: "to appoint/designate"
    Semantic field: "action/decision"
    Forms: I, II, III
```

When multiple senses match, all are included (up to `max_entries_per_reference`). The ambiguity is preserved for downstream stages.

### 5.4 Root Variation Handling

```yaml
Root Normalization Rules:
  Alif mapping:  آ → ا, أ → ا, إ → ا, ئ → ي (final), ؤ → و (final)
  Weak letter normalization: و stays و, ي stays ي (weak letters are meaningful)
  Shadda removal: شَدَّة → بدون shadda (doubled consonant kept)
  Alif maqsura: ى → ي (for searching), original preserved in entry
  Ta-marbuta: ة → ه (for searching), but original preserved in entry
```

### 5.5 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Root not found in KB-0001** | `root_entry = null`; count as `unresolved_references` |
| **Root with only deprecated entries** | Included with a deprecation warning flag if `resolve_depth >= 2` |
| **Weak root with multiple variant forms** | All variant forms are resolved and included in the entry |
| **Quadriliteral root** | Resolved identically to triliteral; root_type = "quadriliteral" |
| **Root with no semantic field** | `semantic_field = null`; no semantic tags assigned |
| **Root meaning in multiple languages** | English meaning is primary; Arabic definition included if available |

---

## 6. Wazan & Pattern Resolution

### 6.1 Purpose

Resolve every wazan/pattern reference in the AnnotatedGIR against KB-0002 (Wazan Database) and KB-0004 (Noun Patterns). For each token with an identified morphological pattern, the Pattern Resolver retrieves the full pattern description, verb form number (if applicable), example words, and inflection templates.

### 6.2 Wazan Resolution Algorithm

```rust
Algorithm: resolve_wazans
Input:
  - resolved_tokens: Vec<ResolvedTokenData>   // From root resolution
  - tokens: Vec<GIRToken>                     // Original tokens
  - wazan_index: KBIndex                      // KB-0002 loaded index
  - config: KnowledgeGraphResolverConfig
Output:
  - resolved_tokens: Vec<ResolvedTokenData>   // Enriched

Step 1: For each token t in tokens:
  1.1  If t does not have a wazan reference → skip
  1.2  Determine lookup key:
  1.2.1  If wazan.id (KB-0002 entry ID) is available → use it directly
  1.2.2  Otherwise, derive key from wazan.text (pattern template)
  1.2.3  Fallback: derive key from wazan.form (e.g., "form:IV")

Step 2: Hash-based lookup
  2.1  Look up key in wazan_index.table
  2.2  If not found → wazan_entry = null
  2.3  If found → deserialize entry

Step 3: Deserialize Wazan Entry
  3.1  Parse: id, pattern text, meaning, form number, category
  3.2  Parse example word and inflection examples
  3.3  Parse weak variant template (if applicable)
  3.4  Parse feature signature (what features this pattern implies)

Step 4: For Noun Patterns (KB-0004)
  4.1  If token.pos is noun or adjective, or wazan.category is noun:
  4.1.1  Look up the wazan_id in noun_pattern_index (KB-0004)
  4.1.2  Retrieve: gender, declension, broken plural mappings, sound plurals
  4.1.3  Attach as noun_entry

Step 5: Resolution Depth Filtering
  5.1  If resolve_depth >= 2:
  5.1.1  Include inflection examples
  5.1.2  Include related patterns
  5.1.3  Include weak variant template
  5.2  If resolve_depth >= 3:
  5.2.1  Resolve related patterns recursively (1 hop)
  5.2.2  Include example sentences for each inflection

Step 6: Attach to token
  6.1  Attach ResolvedWazanEntry to token's resolved data
  6.2  If noun pattern was resolved, attach ResolvedNounEntry
```

### 6.3 Verb Form Identification

When a wazan is linked to a verb form (I–XV), the resolver attaches additional metadata:

```yaml
Verb Form Data (from KB-0002 entry):
  Form number: I-XV
  Pattern template: فَعَلَ, فَعَّلَ, فَاعَلَ, etc.
  Semantic augment: "causative", "intensive", "reflexive", "requestive", etc.
  Transitivity effect: "may add object", "may remove object", etc.
  Weak root variant: adjusted pattern for ajwaf/naqis/mithal roots
  Example root + conjugation: "كَتَبَ → يَكْتُبُ" (Form I)
  Frequency: "very common" | "common" | "uncommon" | "rare"
```

### 6.4 Noun Type Resolution

For nouns, the resolver enriches with noun type data from KB-0004:

```yaml
Noun Type Examples (from KB-0004):
  Masdar (verbal noun):
    - Pattern examples: فَعْل, فِعَال, فُعُول, تَفْعِيل
    - Gender: varies by pattern
    - Broken plurals: often not pluralized
  
  Ism fa'il (active participle):
    - Pattern: فَاعِل
    - Gender: follows referent
    - Broken plurals: فُعَّال, فَعَلَة
  
  Ism maf'ul (passive participle):
    - Pattern: مَفْعُول
    - Follows object's gender/number
  
  Ism makan / zaman (place/time noun):
    - Pattern: مَفْعَل, مَفْعِل
    - Typically feminine
  
  Ism alah (instrument noun):
    - Pattern: مِفْعَال, مِفْعَل, مِفْعَلَة
    - Typically feminine
```

### 6.5 Verb Form Paradigm Resolution (KB-0003)

Verb form data from KB-0003 (Verb Forms) is accessed through the wazan index and attached as part of wazan resolution. Each wazan entry that corresponds to a verb form (I–XV) includes an `inflection_examples` field populated from KB-0003 data:

```yaml
Resolution Flow for Verb Forms:
  1. Token has wazan reference with form = IV (e.g., أَفْعَلَ)
  2. Wazan resolver looks up KB-0002 entry for "form:IV"
  3. KB-0002 entry includes a KB-0003 reference ID
  4. Resolver loads verb paradigm data from KB-0003 via the reference:
     - Conjugation templates (past, present, imperative)
     - Weak root variants (for ajwaf, naqis, mithal roots)
     - Example conjugations for a canonical root
  5. Data is attached to ResolvedWazanEntry.inflection_examples
  6. No separate KB-0003 resolution pass is needed — data is embedded
     in the wazan resolution path
```

There is no separate KB-0003 resolution subsystem. Verb paradigm data is embedded in the wazan entry resolution path as cross-linked data in the compiled KB indices. This avoids redundant lookups and keeps the resolution pipeline streamlined.

### 6.6 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Wazan ID available but pattern template missing** | Resolve by ID only; pattern template derived from KB-0002 entry |
| **Pattern template available but no form number** | Resolve by pattern template search in KB-0002 |
| **Ambiguous pattern (matches both verb and noun)** | Include both entries with confidence scores |
| **Wazan with weak variant** | Attach weak variant template; downstream can use for inflection |
| **Noun pattern not found in KB-0004** | noun_entry = null; wazan_entry still resolved from KB-0002 |
| **Pattern not found in any KB** | wazan_entry = null; count as unresolved |

---

## 7. Dictionary & Lexical Enrichment

### 7.1 Purpose

Optionally enrich each token with dictionary-style entries: word definitions, translations, usage examples, and cross-linguistic references. Dictionary data is provided by plugins (kb_resolver type), not by the core KBs.

### 7.2 Dictionary Plugin Interface

```rust
/// Interface for dictionary plugins
trait DictionaryPlugin {
    fn plugin_id(&self) -> String;
    fn plugin_type(&self) -> &str { "kb_resolver" }
    fn supported_languages(&self) -> Vec<String>;

    /// Look up a word or root in the dictionary
    fn lookup(
        &self,
        query: DictionaryQuery,
        context: &PipelineContext,
    ) -> Result<DictionaryResult, PluginError>;

    /// Batch lookup (for performance)
    fn batch_lookup(
        &self,
        queries: Vec<DictionaryQuery>,
        context: &PipelineContext,
    ) -> Result<Vec<Option<DictionaryResult>>, PluginError>;
}

type DictionaryQuery = {
    /// Query type
    query_type: "root" | "word" | "pattern" | "phrase",
    /// The query string (normalized)
    query: String,
    /// Language for definitions
    language: String,
    /// Maximum results
    max_results: u8,
}

type DictionaryResult = {
    /// Source plugin identifier
    source: String,
    /// Matched entries
    entries: Vec<DictionaryEntry>,
    /// Query execution time
    query_time_ms: f64,
}

type DictionaryEntry = {
    /// Entry identifier
    id: String,
    /// Headword
    headword: String,
    /// Vocalized form
    vocalized_form: String | null,
    /// Part of speech
    part_of_speech: String,
    /// Definitions (in requested language)
    definitions: Vec<String>,
    /// Usage examples
    examples: Vec<UsageExample>,
    /// Root reference (if applicable)
    root: String | null,
    /// Translations (optional)
    translations: HashMap<String, Vec<String>> | null,
    /// Register/domain labels
    labels: Vec<String>,     // e.g., ["classical", "poetic", "technical"]
}

type UsageExample = {
    /// Example text
    text: String,
    /// Source/reference
    source: String | null,
    /// Translation (if applicable)
    translation: String | null,
}
```

### 7.3 Dictionary Lookup Strategy

```rust
Algorithm: resolve_dictionary_entries
Input:
  - resolved_tokens: Vec<ResolvedTokenData>
  - tokens: Vec<GIRToken>
  - plugins: Vec<Box<dyn DictionaryPlugin>>
  - config: KnowledgeGraphResolverConfig
Output:
  - resolved_tokens // Enriched with dictionary_entry

Step 1: Build query batch
  1.1  For each token with stem text:
  1.1.1  Create DictionaryQuery for the stem
  1.1.2  If the token has a resolved root, create a second query
         for the root (dictionary may have root-level entries)
  1.1.3  Add to batch

Step 2: For each plugin in plugin chain:
  2.1  Call plugin.batch_lookup(queries, context)
  2.2  Merge results into token data:
  2.2.1  If plugin returns entries for a token → attach as dictionary_entry
  2.2.2  If multiple plugins return results → merge, deduplicate by headword

Step 3: Handle plugin timeouts
  3.1  If a plugin exceeds plugin_timeout_ms:
  3.1.1  Log warning
  3.1.2  If strict_plugins → return error
  3.1.3  If not strict → skip plugin, continue with available data

Step 4: Return enriched tokens
```

### 7.4 Built-in Dictionary Sources

AGOS may ship with a **compact built-in dictionary** containing:

| Data Set | Size | Coverage |
|----------|------|----------|
| Common word definitions | ~50,000 entries | MSA core vocabulary |
| Root meanings | ~5,000 entries | All common Arabic roots |
| Quranic word list | ~1,800 entries | Every word in the Quran (by lemma) |
| Common broken plurals | ~3,000 entries | Frequent irregular plural forms |

The built-in dictionary is optional and embedded as a plugin. Third-party dictionaries (e.g., Hans Wehr, Lane's Lexicon) are supported via custom plugins.

---

## 8. Semantic & Etymological Enrichment

### 8.1 Purpose

Assign semantic tags to tokens based on their resolved root properties, noun types, and contextual features. Optionally attach etymological information tracing word origins.

### 8.2 Semantic Tagging Algorithm

```rust
Algorithm: assign_semantic_tags
Input:
  - resolved_tokens: Vec<ResolvedTokenData>  // With root_entry resolved
  - config: KnowledgeGraphResolverConfig
Output:
  - resolved_tokens // Enriched with semantic_tags

Step 1: For each token with resolved root_entry:
  1.1  Derive semantic tags from root properties:

  1.2  From Root Type:
  1.2.1  "triliteral" | "quadriliteral" | "weak" | "hamzated" | "doubled"

  1.3  From Semantic Field:
  1.3.1  Map KB-0001 semantic_field to standard tags:
         - "writing" → ["communication", "written"]
         - "religion" → ["religion", "spiritual"]
         - "body" → ["body", "human"]
         - "nature" → ["nature", "environment"]
         - "emotion" → ["emotion", "psychological"]
         - "action" → ["action", "dynamic"]
         - "state" → ["state", "static"]
         - "quantity" → ["quantity", "measurement"]
         - "time" → ["time", "temporal"]
         - "space" → ["space", "location"]

  1.4  From Transitivity:
  1.4.1  "transitive" | "intransitive" | "ditransitive"

  1.5  From Verb Form:
  1.5.1  Map verb form to semantic augment tags:
         - Form II: ["intensive", "causative"]
         - Form III: ["attemptive", "reciprocal"]
         - Form IV: ["causative"]
         - Form V: ["reflexive"]
         - Form VI: ["reciprocal"]
         - Form VII: ["passive", "reflexive"]
         - Form VIII: ["reflexive", "mediopassive"]
         - Form IX: ["inchoative", "color"]
         - Form X: ["requestive"]

  1.6  From Noun Type:
  1.6.1  "verbal_noun" | "active_participle" | "passive_participle"
          "noun_of_place" | "noun_of_time" | "noun_of_instrument"
          "adjective" | "elative" | "nisbah"

Step 2: Deduplicate and filter tags
  2.1  Remove duplicate tags
  2.2  Limit to most specific tags (remove overly general ones
       if more specific ones are available)

Step 3: Assign confidence
  3.1  Tags derived from root_entry (direct match): confidence = 0.9
  3.2  Tags derived from form/noun type (indirect): confidence = 0.7
  3.3  Tags derived from inference: confidence = 0.5

Step 4: Update token resolved data
  4.1  Set token.semantic_tags = deduplicated tag list
```

### 8.3 Semantic Tag Taxonomy

Tags are drawn from a controlled taxonomy defined in KB-0007:

```yaml
Tag Categories (from KB-0007 §Semantic Features):
  Grammatical:  [verb, noun, particle, pronoun, adjective, adverb]
  Inflectional: [past, present, imperative, singular, dual, plural,
                 masculine, feminine, 1st_person, 2nd_person, 3rd_person]
  Derivational: [form_I, form_II, ..., form_XV, masdar, ism_fail,
                 ism_maful, ism_makan, ism_zaman, ism_alah]
  Lexical:      [human, animal, object, abstract, action, state,
                 emotion, cognition, communication, religion, nature,
                 body, time, space, quantity, quality]
  Discourse:    [main_clause, subordinate, conditional, emphatic]
  Register:     [classical, quranic, msa, poetic, colloquial]
```

### 8.4 Etymological Enrichment (Optional)

When `enable_etymology = true`, the resolver attempts to trace word origins:

```yaml
Etymology Data Sources (future):
  Arabic root origins:
    - Proto-Semitic reconstruction (where available)
    - Borrowed roots (Persian, Greek, Aramaic, Syriac, etc.)
    - Quranic Aramaic cognates
  
  Data included per entry:
    - Etymological origin: "arabic" | "semitic" | "borrowed:persian" | etc.
    - Proto-Semitic form (if reconstructable)
    - Cognates in other Semitic languages (Hebrew, Aramaic, Akkadian, etc.)
    - First attestation period: "pre-islamic" | "quranic" | "early_abbasid" | etc.
    - Source reference
```

Etymological enrichment is not available in the initial release. It is designed as an optional plugin extension.

---

## 9. Cross-Referencing & Relationship Resolution

### 9.1 Purpose

Resolve the cross-reference network between KB entries. KB-0001 defines synonym, antonym, and related-root relationships between root entries. The Cross-Reference Engine resolves these references into concrete entry references, enabling downstream stages to present rich relational data.

### 9.2 Cross-Reference Resolution Algorithm

```rust
Algorithm: resolve_cross_references
Input:
  - resolved_tokens: Vec<ResolvedTokenData>  // With root_entry resolved
  - root_index: KBIndex
  - config: KnowledgeGraphResolverConfig
Output:
  - resolved_tokens // Enriched with resolved cross_references

Step 1: For each token with resolved root_entry:
  1.1  If root_entry.cross_references is empty → skip

Step 2: Resolve Synonyms
  2.1  For each synonym reference in root_entry.cross_references.synonyms:
  2.1.1  Normalize the referenced root text
  2.1.2  Look up in root_index
  2.1.3  If found → include resolved synonym entry (meaning, root type)
  2.1.4  If not found → include as unresolved reference string
  2.1.5  Respect max_entries_per_reference limit

Step 3: Resolve Antonyms
  3.1  Same algorithm as synonyms but for antonym list

Step 4: Resolve Related Roots
  4.1  For each related_root reference:
  4.1.1  Look up in root_index
  4.1.2  Include relationship type (semantic_field, derivational, etc.)
  4.1.3  Include description if available

Step 5: Transitive Resolution (Depth >= 3)
  5.1  If config.resolve_depth >= 3 and config.max_cross_ref_depth >= 1:
  5.1.1  For each resolved synonym/antonym/related root:
  5.1.2  If they have their own cross_references, follow 1 hop
  5.1.3  Do NOT follow further (prevent infinite recursion)

Step 6: Build Resolved CrossReferences
  6.1  Package resolved references into CrossReferences struct
  6.2  Attach to token's resolved data
```

### 9.3 Resolved CrossReferences Structure

```rust
type ResolvedCrossReferences = {
    /// Resolved synonym entries (with meanings)
    synonyms: Vec<ResolvedReference>,
    /// Resolved antonym entries
    antonyms: Vec<ResolvedReference>,
    /// Resolved related roots with relationship types
    related_roots: Vec<ResolvedRelatedRoot>,
}

type ResolvedReference = {
    /// Root text
    root: String,
    /// Root meaning (from KB-0001)
    meaning: String,
    /// Attestation level
    attestation: String | null,
    /// Whether this reference was fully resolved
    resolved: bool,
}

type ResolvedRelatedRoot = {
    /// Root text
    root: String,
    /// Relationship type description
    relationship: String,
    /// Description of the relationship
    description: String | null,
    /// Whether fully resolved
    resolved: bool,
}
```

### 9.4 Semantic Field Grouping

Cross-reference resolution also groups related roots by semantic field:

```yaml
Example: Semantic Field "Writing" (field_id: "writing")
  Roots in this field:
    ك ت ب (kataba) — to write
    خط ط (khatta) — to draw/write
    رقم (raqama) — to number/write
    س ط ر (satara) — to write/line
    دو ن (dawwana) — to record/write
    حر ر (harara) — to write/free
    نسخ (nasakha) — to copy/write
  
  Each root in this field gets:
    semantic_tags += ["writing"]
    cross_references.related_roots += other roots in the same field
```

### 9.5 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Cross-reference cycle** (A → B → A) | Detected and terminated at max_cross_ref_depth |
| **Cross-reference to non-existent root** | Included as unresolved reference with `resolved = false` |
| **Large number of synonyms** | Truncated at max_entries_per_reference |
| **Same root in multiple semantic fields** | All field tags are assigned |

---

## 10. Resolution Aggregation & Statistics

### 10.1 Purpose

After all tokens have been resolved, the resolver aggregates statistics about the resolution process. These statistics are included in the ResolvedGIR for monitoring, debugging, and quality assessment.

### 10.2 Statistics Collection

```rust
Algorithm: collect_resolution_stats
Input:
  - resolved_tokens: Vec<ResolvedTokenData>
  - kb_registry: KBIndexRegistry
  - start_time: u64             // nanoseconds
  - config: KnowledgeGraphResolverConfig
Output:
  - resolution_stats: ResolutionStats

Step 1: Count resolved references
  1.1  For each token in resolved_tokens:
  1.1.1  If root_entry is not null → roots_resolved += 1
  1.1.2  If wazan_entry is not null → patterns_resolved += 1
  1.1.3  If noun_entry is not null → noun_patterns_resolved += 1
  1.1.4  If particle_entry is not null → particles_resolved += 1
  1.1.5  If pronoun_entry is not null → pronouns_resolved += 1

Step 2: Count unresolved references
  2.1  For each token in original tokens:
  2.1.1  If token has root reference but resolved root_entry is null →
         unresolved_references += 1
  2.1.2  If token has wazan reference but resolved wazan_entry is null →
         unresolved_references += 1
  2.1.3  (Same for noun patterns, particles, pronouns)

Step 3: Compute timing
  3.1  total_time = now() - start_time
  3.2  resolution_time_ms = total_time / 1_000_000

Step 4: Collect KB versions
  4.1  For each loaded KB in kb_registry:
  4.1.1  Record kb_id and kb_version

Step 5: Compute coverage
  5.1  If total references > 0:
  5.1.1  root_coverage = roots_resolved / (roots_resolved + root_unresolved)
  5.1.2  pattern_coverage = patterns_resolved / (patterns_resolved + pattern_unresolved)
  5.2  overall_coverage = total_resolved / (total_resolved + total_unresolved)

Step 6: Build ResolutionStats
  6.1  Return ResolutionStats struct
```

### 10.3 ResolutionStats Schema

```rust
type ResolutionStats = {
    /// Counts
    roots_resolved: u32,
    patterns_resolved: u32,
    noun_patterns_resolved: u32,
    particles_resolved: u32,
    pronouns_resolved: u32,
    dictionary_lookups_succeeded: u32,
    
    /// Unresolved
    unresolved_references: u32,
    unresolved_roots: Vec<String> | null,     // Only if > 0
    
    /// Timing
    resolution_time_ms: f64,
    kb_load_time_ms: f64,
    
    /// Coverage
    overall_coverage_pct: f64,                // 0.0–100.0
    
    /// KB versions used
    knowledge_versions: KnowledgeVersionMap,
    
    /// KB availability
    missing_kbs: Vec<String>,
    
    /// Plugin stats
    plugins_invoked: u32,
    plugin_errors: u32,
}
```

### 10.4 Resolution Metadata in IR-8

The ResolutionStats are included in the ResolvedGIR:

```json
{
    "resolution_stats": {
        "roots_resolved": 3,
        "patterns_resolved": 3,
        "noun_patterns_resolved": 1,
        "particles_resolved": 1,
        "pronouns_resolved": 1,
        "dictionary_lookups_succeeded": 2,
        "unresolved_references": 0,
        "unresolved_roots": [],
        "resolution_time_ms": 0.045,
        "kb_load_time_ms": 14.2,
        "overall_coverage_pct": 100.0,
        "knowledge_versions": {
            "KB-0001": "1.2.3",
            "KB-0002": "2.0.1",
            "KB-0003": "1.1.0",
            "KB-0004": "1.0.2",
            "KB-0005": "1.3.0",
            "KB-0006": "1.0.0",
            "KB-0007": "2.1.0"
        },
        "missing_kbs": [],
        "plugins_invoked": 1,
        "plugin_errors": 0
    }
}
```

---

## 11. Performance & Optimization

### 11.1 Performance Targets

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 50 μs per token | KBs memory-mapped, warm cache |
| Latency (p99) | < 500 μs per token | All KBs queried, cold cache |
| Throughput | > 10K tokens/s | Single core |
| KB lookup | < 1 μs per lookup | Hash-based index, no collisions |
| Index load time | < 100 ms per 100K entries | NVMe SSD |
| Memory per token | < 1 KB (resolved data) | Depth level 3 |
| Cache hit latency | < 0.5 μs per token | In-memory, hash lookup |

### 11.2 Optimization Strategies

#### Strategy 1: Hash-Based Direct Lookup

All KB lookups use O(1) average-time hash tables:

```yaml
Lookup Cost Breakdown:
  Hash computation:      ~50 ns  (xxHash3)
  Hash table lookup:     ~30 ns  (L1 cache hit)
  Mmap region read:      ~100 ns (L1/L2 cache)
  Entry deserialization: ~200 ns (lightweight binary parse)
  Total per lookup:      < 400 ns (typical)
```

#### Strategy 2: Memory-Mapped KB Indices

KBs are never fully loaded into process memory. Instead:

```yaml
Memory Mapping Benefits:
  - Zero copy: OS pages are mapped directly into process address space
  - Demand paging: Only accessed pages are loaded (hot entries stay cached by OS)
  - Shared across processes: Multiple resolver instances share the same physical pages
  - No (de)serialization: Binary format is parsed in-place from mmap region
  - Atomic reload: Old mmap region is released only after new one is active
```

#### Strategy 3: Batch Reference Extraction

Rather than walking the GIR tree for each token individually, the resolver extracts all references first, then performs batched lookups:

```rust
Algorithm: batch_resolve
Input: annotated_gir (AnnotatedGIR)
Output: resolved_gir (ResolvedGIR)

Step 1: Extract reference set
  1.1  Walk the GIR and collect all unique references:
  1.1.1  unique_roots = Set<String>     // All distinct root texts
  1.1.2  unique_patterns = Set<String>  // All distinct pattern IDs
  1.1.3  unique_particles = Set<String> // All distinct particle texts
  1.1.4  unique_pronouns = Set<String>  // All distinct pronoun texts
  1.1.5  unique_features = Set<String>  // All distinct feature names

Step 2: Batch KB lookups
  2.1  For each unique reference:
  2.1.1  Look up in the appropriate KB index
  2.1.2  Cache the result (for reuse across tokens)

Step 3: Distribute to tokens
  3.1  For each token in GIR:
  3.1.1  Look up resolved data from batch cache
  3.1.2  Attach to token
```

This strategy ensures each root is looked up exactly once, even if it appears in multiple tokens.

#### Strategy 4: Lazy Feature Metadata Resolution (Optional)

Feature metadata (KB-0007 descriptions) MAY be resolved on-demand rather than during batch resolution (see §3.2.2 which shows batch-time resolution as the default):

```yaml
Lazy Resolution (optional optimization):
  - At batch time: resolution of root, wazan, particle, pronoun (default behavior)
  - Alternative: at serialization time: resolution of feature metadata only
    * Feature descriptions are resolved when the ResolvedGIR is serialized
      for downstream consumption, not during batch resolution
    * In-memory representation stores feature names only
    * This avoids ~30μs of feature resolution for tokens not exported
    * Only beneficial if many tokens have resolution data that is never
      exported (e.g., during intermediate pipeline debugging)
    * Default implementation uses batch-time resolution for simplicity
```

#### Strategy 5: Parallel Token Resolution

For sentences with many tokens, resolution can be parallelized:

```yaml
Parallelization:
  Level: Token-level
  Granularity: per-token (each token is independent)
  Overhead: ~1 μs per token (spawn + join)
  Speedup: 2–4× on multi-core systems (limited by memory bandwidth)
  Default: sequential (single-threaded), parallel optional
```

#### Strategy 6: Depth-Based Short Circuit

Resolution stops early when configured depth is low:

```yaml
Depth vs. Time Tradeoff:
  Depth 0: ~5 μs per token (ID-to-name mapping only)
  Depth 1: ~15 μs per token (basic entry data)
  Depth 2: ~30 μs per token (full entry + cross-refs)
  Depth 3: ~50 μs per token (transitive + etymology)
```

### 11.3 Memory Budget

| Component | Size (Compact) | Size (Full) | Notes |
|-----------|---------------|-------------|-------|
| KB-0001 index | 20 MB | 80 MB | Memory-mapped |
| KB-0002 index | 10 MB | 40 MB | Memory-mapped |
| KB-0003 index | 15 MB | 30 MB | Memory-mapped |
| KB-0004 index | 10 MB | 30 MB | Memory-mapped |
| KB-0005 index | 2 MB | 5 MB | Memory-mapped |
| KB-0006 index | 1 MB | 2 MB | Memory-mapped |
| KB-0007 index | 1 MB | 2 MB | Memory-mapped |
| Hash tables (all KBs) | 10 MB | 40 MB | In-process memory |
| Resolver scratch space | 1 MB | 5 MB | Per-request (reusable) |
| **Total static** | **70 MB** | **234 MB** | |
| **Per-request dynamic** | **1 KB** | **50 KB** | Per token |

### 11.4 Cache Integration

```yaml
Cache Points:
  KB Index Cache:
    - What: Hash tables for each loaded KB
    - Lifetime: Until KB version changes (reload)
    - Eviction: None (all entries always loaded)
    - Size: ~10–40 MB total
  
  Resolved Entry Cache:
    - What: Recently looked up KB entries
    - Lifetime: TTL-based (configurable, default: 1 hour)
    - Eviction: LRU
    - Size: Configurable (default: 10,000 entries)
    - Purpose: Repeated lookups of common roots across requests
  
  GIR Resolution Cache (MOD-13):
    - What: Full ResolvedGIR output
    - Key: hash(AnnotatedGIR + KB versions + config)
    - Hit rate target: > 95%
    - Only invalidated when KB versions change or config changes
```

---

## 12. Plugin Integration

### 12.1 KB Resolver Plugin Interface

The `kb_resolver` plugin type (defined in SPEC-0001-C7) allows custom resolvers to extend the Knowledge Graph Engine:

```rust
/// KB Resolver Plugin trait
trait KBResolverPlugin {
    fn plugin_id(&self) -> String;
    fn plugin_type(&self) -> &str { "kb_resolver" }
    fn priority(&self) -> u32;

    /// Supported KB types this plugin can resolve
    fn supported_kb_types(&self) -> Vec<String>;

    /// Resolve a query against this plugin's data
    fn resolve(
        &self,
        input: KBResolverInput,
        context: &PipelineContext,
    ) -> Result<KBResolverOutput, PluginError>;
}

type KBResolverInput = {
    /// The AnnotatedGIR to resolve
    gir: AnnotatedGIR,
    /// Plugin-specific configuration
    plugin_config: HashMap<String, serde_json::Value>,
}

type KBResolverOutput = {
    /// Resolved token data
    resolved_tokens: Vec<ResolvedTokenDataExt>,
    /// Resolution statistics
    stats: PluginResolutionStats,
}
```

### 12.2 Plugin Chain Execution

```rust
Algorithm: execute_plugin_chain
Input:
  - gir: AnnotatedGIR
  - plugins: Vec<Box<dyn KBResolverPlugin>>   // Ordered by priority
  - config: KnowledgeGraphResolverConfig
Output:
  - enriched_entries: HashMap<String, Vec<ResolvedTokenDataExt>>

Step 1: For each plugin in plugins (ordered by priority descending):
  1.1  Start timer
  1.2  Call plugin.resolve(input, context)
  1.3  If plugin returns within timeout:
  1.3.1  Merge plugin's resolved data into the token enrichment
  1.3.2  Plugin data takes priority over lower-priority plugins
  1.4  If plugin times out:
  1.4.1  Log warning
  1.4.2  If strict_plugins → return error
  1.4.3  Continue to next plugin

Step 2: Merge plugin data
  2.1  For each token index:
  2.1.1  If plugin A resolved root → prefer plugin A's result
  2.1.2  If plugin B resolved dictionary → append to dictionary_entries
  2.1.3  Deduplicate: same headword from multiple plugins → keep highest
         priority

Step 3: Return merged results
```

### 12.3 Standard Plugin Examples

#### Example 1: Hans Wehr Dictionary Plugin

```yaml
Plugin Manifest:
  id: "dict-hans-wehr"
  name: "Hans Wehr Dictionary (4th ed.)"
  version: "1.2.0"
  plugin_type: "kb_resolver"
  priority: 50
  supported_kb_types: ["dictionary"]
  entry_point: "hans-wehr-resolver.wasm"
  data_size: "~45 MB compressed"
  coverage: "~50,000 entries"
```

#### Example 2: Quranic Word Plugin

```yaml
Plugin Manifest:
  id: "quranic-words"
  name: "Quranic Word-by-Word Data"
  version: "1.0.0"
  plugin_type: "kb_resolver"
  priority: 80
  supported_kb_types: ["dictionary", "etymology"]
  entry_point: "quranic-resolver.wasm"
  data_size: "~5 MB"
  coverage: "~1,800 lemmas, ~77,000 word occurrences"
```

### 12.4 Plugin Security

As defined in SPEC-0001-C7 §6, `kb_resolver` plugins operate in a sandboxed environment:

| Restriction | Enforcement |
|-------------|-------------|
| **KB data access** | Read-only memory-mapped regions |
| **Plugin data isolation** | Each plugin has its own data directory |
| **No modification** | Plugins cannot modify core KB data |
| **Execution limit** | Max 50 ms per plugin invocation |
| **Memory limit** | Max 64 MB per plugin instance |

---

## 13. Testing Strategy

### 13.1 Unit Tests

| Test Category | Description | Example |
|---------------|-------------|---------|
| **Root resolution** | Single root lookup against KB-0001 | `hash("كتب")` → root_entry with meaning "to write" |
| **Wazan resolution** | Single wazan lookup against KB-0002 | `pattern:"فَعَلَ"` → wazan_entry with form I |
| **Noun pattern resolution** | Noun pattern from KB-0004 | `pattern:"فَاعِل"` → noun_entry with type "ism fa'il" |
| **Resolution depth** | Verify data at each depth level | Depth 0: just IDs; Depth 3: transitive refs |
| **Missing reference** | Reference not in KB → null | `hash("XYZ")` → null |
| **Batch extraction** | Deduplication of references | Same root 3 times → 1 lookup |
| **Stats collection** | Correct counting | 3 roots resolved, 0 unresolved |
| **Cross-reference resolution** | Follow synonym links | Resolve A's synonym B, verify A's cross_refs contains B's meaning |

### 13.2 Integration Tests

| Test | Description |
|------|-------------|
| **Full pipeline integration** | MOD-04 → MOD-05 → MOD-06 → MOD-07 → MOD-08, verify ResolvedGIR completeness |
| **KB version mismatch** | Load KB-0001 with incompatible version → KB_VERSION_MISMATCH |
| **Missing optional KB** | Configure dictionary plugin absent → graceful degradation |
| **Plugin chain** | Multiple kb_resolver plugins → correct merge and priority |
| **Hot reload** | Reload KB-0001 while in-flight requests use old index, new requests use new |

### 13.3 Performance Tests

| Test | Target |
|------|--------|
| **Single token resolution** | < 50 μs, all KBs, depth 3 |
| **10-word sentence resolution** | < 500 μs, all KBs, depth 3 |
| **KB index load (all 7 KBs)** | < 500 ms on reference hardware |
| **Batch lookup (50 unique roots)** | < 2 ms |
| **Cross-reference resolution (10 refs)** | < 100 μs |
| **Memory-mapped KB access (random 1000 lookups)** | < 1 ms total |

### 13.4 Test Fixture Format

```json
{
    "test_name": "resolve_root_kataba",
    "description": "Resolve root 'كتب' against KB-0001",
    "config": {
        "resolve_depth": 2,
        "enable_semantic": true,
        "enable_cross_references": true
    },
    "input_token": {
        "root": { "text": "كتب", "confidence": 0.98 }
    },
    "expected_output": {
        "root_entry": {
            "root": "كتب",
            "meaning": "to write",
            "root_type": "sound",
            "forms": [1, 2, 3, 4, 5, 6, 7, 8, 10],
            "semantic_field": "writing"
        },
        "semantic_tags": [
            "triliteral", "communication", "written", "action"
        ],
        "resolved": true
    }
}
```

---

## 14. Implementation Guidance

### 14.1 Language Recommendations

| Component | Recommended Language | Rationale |
|-----------|---------------------|-----------|
| KB Index Loader | Rust | Memory-mapped I/O, zero-cost abstractions |
| Reference Resolver | Rust | Performance-critical hot path |
| Semantic Enricher | Rust / Python | Rust for performance; Python for research/prototyping |
| Plugin Host | Rust | WASM runtime integration |
| Statistics Collector | Any | Non-critical, light computation |

### 14.2 Implementation Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **KB index too large for memory-limited devices** | High memory usage (~240 MB full KBs) | Compact KB format (~70 MB); memory-mapped access; lazy loading of optional KBs |
| **Hash collision slowing lookups** | Degraded O(n) lookup time | Robin Hood hashing with good hash distribution; monitor collision rate; rebuild index if needed |
| **Plugin timeout blocking pipeline** | Latency spike | Strict timeout enforcement (50 ms default); sandboxed execution |
| **KB version mismatch detected at runtime** | Pipeline failure | Version check at load time; graceful error message; CI gate for KB compatibility |
| **Cross-reference cycles** | Infinite resolution | Hard depth limit (configurable, default 1 hop) |
| **Dictionary plugin crash** | Missing dictionary data | Graceful degradation: log error, skip plugin, continue with available KBs |

### 14.3 Resolution Pipeline Summary

```
INPUT: AnnotatedGIR (IR-7)
  │
  ▼
1. KB INDEX LOADING (at init time)
   ├── Memory-map all 7 KB binaries
   ├── Build/load hash indices
   └── Verify KB version compatibility
      │
      ▼
2. REFERENCE EXTRACTION (batch)
   ├── Walk GIR tokens
   ├── Collect unique root_refs
   ├── Collect unique pattern_refs
   ├── Collect unique particle_refs
   └── Collect unique pronoun_refs
      │
      ▼
3. BATCH KB LOOKUP
   ├── Resolve roots (KB-0001) → ResolvedRootEntry
   ├── Resolve patterns (KB-0002) → ResolvedWazanEntry
   ├── Resolve noun patterns (KB-0004) → ResolvedNounEntry
   ├── Resolve particles (KB-0005) → ResolvedParticleEntry
   ├── Resolve pronouns (KB-0006) → ResolvedPronounEntry
   └── Resolve features (KB-0007) → FeatureMeta
      │
      ▼
4. ENRICHMENT
   ├── Semantic tagging (from root properties)
   ├── Etymological lookup (optional)
   ├── Dictionary lookup (optional, via plugins)
   └── Cross-reference resolution
      │
      ▼
5. MERGE & ASSEMBLE
   ├── Distribute resolved data to each token
   ├── Merge plugin enrichment data
   ├── Collect resolution statistics
   └── Build ResolvedGIR (IR-8)
      │
      ▼
OUTPUT: ResolvedGIR (IR-8) → MOD-09 (BytecodeGenerator)
```

### 14.4 Key Implementation Rules

1. **Never fabricate data.** If a reference cannot be resolved, set it to `null`. Do not guess, infer, or hallucinate root meanings or semantic tags.

2. **Graceful degradation is mandatory.** The resolver must handle missing optional KBs, plugin timeouts, and partial data without crashing the pipeline.

3. **Deterministic output.** Given the same AnnotatedGIR and KB versions, the resolver must produce identical ResolvedGIR output, byte-for-byte at the serialization level.

4. **Batch before distribute.** Always extract unique references first, then batch-lookup, then distribute to tokens. This provides O(r) lookups instead of O(t × r) where r = distinct references and t = total tokens.

5. **Cache aggressively.** KB index lookups are the dominant cost (70%+ of resolution time). Cache resolved entries at the index level and the GIR level.

6. **Version everything.** Record which KB versions were used for resolution. This enables reproducibility and cache invalidation.

---

## 15. Cross-References

### 15.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | MOD-08 position in pipeline and layer architecture |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | MOD-08 pipeline algorithm (§9) |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | MOD-08 public interface (§10) |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | IR-8 (ResolvedGIR) schema (§9) |
| SPEC-0001-C7 | Extensibility & Plugin Architecture | KB resolver plugin type (§3, §9) |
| SPEC-0001-C8 | Security, Validation & Error Handling | Error handling policies |
| SPEC-0001-C9 | Performance Targets & Constraints | MOD-08 performance budgets (§3.2) |
| SPEC-0101 | Morphology Engine | Produces the root/wazan references MOD-08 resolves |
| SPEC-0201 | Rule Engine | Produces the AnnotatedGIR that MOD-08 consumes |
| ADR-0003 | Why Grammar Intermediate Representation | Justifies the GIR boundary and IR-8 design |

### 15.2 Knowledge Base References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| KB-0001 | Roots Database | Primary data source for root resolution |
| KB-0002 | Wazan Database | Primary data source for pattern resolution |
| KB-0003 | Verb Forms | Verb paradigm data referenced by patterns |
| KB-0004 | Noun Patterns | Noun pattern data for noun enrichment |
| KB-0005 | Particles | Particle definitions and grammatical effects |
| KB-0006 | Pronouns | Pronoun feature tables and attachment rules |
| KB-0007 | Morphological Features Taxonomy | Feature descriptions and allowed values |

### 15.3 External References

| Reference | Relevance |
|-----------|-----------|
| xxHash3 | Hash function for KB index lookups |
| Memory-Mapped Files (mmap) | KB binary loading strategy |
| WebAssembly (WASM) | Plugin runtime for KB resolver plugins |
| Robin Hood Hashing | Hash table collision resolution strategy |
| Semantic Field Theory (Louw & Nida) | Inspiration for semantic tag taxonomy |

---

## Progress Summary

| Section | Title | Status |
|---------|-------|--------|
| §1 | Introduction & Scope | ✓ COMPLETE |
| §2 | Architecture Overview | ✓ COMPLETE |
| §3 | Internal Component Model | ✓ COMPLETE |
| §4 | KB Index Loading & Lifecycle | ✓ COMPLETE |
| §5 | Root Resolution | ✓ COMPLETE |
| §6 | Wazan & Pattern Resolution | ✓ COMPLETE |
| §7 | Dictionary & Lexical Enrichment | ✓ COMPLETE |
| §8 | Semantic & Etymological Enrichment | ✓ COMPLETE |
| §9 | Cross-Referencing & Relationship Resolution | ✓ COMPLETE |
| §10 | Resolution Aggregation & Statistics | ✓ COMPLETE |
| §11 | Performance & Optimization | ✓ COMPLETE |
| §12 | Plugin Integration | ✓ COMPLETE |
| §13 | Testing Strategy | ✓ COMPLETE |
| §14 | Implementation Guidance | ✓ COMPLETE |
| §15 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 2–5, 7–9), SPEC-0101, SPEC-0201, ADR-0003, KB-0001–0007.

**Recommended next step:** RFC-0002 (Grammar Bytecode Format) — the next downstream component that consumes MOD-09 and produces grammar bytecode consumed by the GVM.
