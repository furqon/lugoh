---
spec_id: SPEC-0103
title: Morphology Engine Performance Optimization Guide
version: 1.0.0
status: Draft
author: AGOS Performance Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0101: Morphology Engine — Detailed Implementation Specification
  - SPEC-0102: Feature System — Encoding, Validation & Resolution
  - KB-OVERVIEW: KB Suite Overview & Architecture
  - KB-0005: Particles — Grammatical & Functional Words
  - KB-0006: Pronouns — Personal, Demonstrative & Relative
  - KB-0007: Morphological Features — Taxonomy & Encoding
  - KB-0008: Particles Database — Developer Reference & Compiled Module
  - RFC-0003: Grammar Virtual Machine
  - SPEC-0301: Grammar Runtime
---

# SPEC-0103: Morphology Engine Performance Optimization Guide

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Performance Philosophy & Principles](#2-performance-philosophy--principles)
3. [Hot Path Analysis](#3-hot-path-analysis)
4. [Fast-Path Optimization (Particles & Pronouns)](#4-fast-path-optimization-particles--pronouns)
5. [Known Words Index](#5-known-words-index)
6. [Root Extraction Optimization](#6-root-extraction-optimization)
7. [Wazan Identification Optimization](#7-wazan-identification-optimization)
8. [Feature Extraction & Bitfield Packing](#8-feature-extraction--bitfield-packing)
9. [Syntax Parsing Optimization](#9-syntax-parsing-optimization)
10. [KB Loading & Memory Management](#10-kb-loading--memory-management)
11. [Caching Architecture](#11-caching-architecture)
12. [Concurrency & Parallelism](#12-concurrency--parallelism)
13. [Benchmarking Methodology](#13-benchmarking-methodology)
14. [Performance Regression Prevention](#14-performance-regression-prevention)
15. [Profiling & Diagnostics](#15-profiling--diagnostics)
16. [Deployment-Specific Optimizations](#16-deployment-specific-optimizations)
17. [Cross-References](#17-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0103 is the **performance optimization guide** for the AGOS Morphology Engine (MOD-04 and MOD-05). It bridges the gap between the **performance targets** defined in SPEC-0001-C9 and the **implementation details** specified in SPEC-0101, providing concrete optimization strategies, code patterns, and benchmarking methodology for developers building or optimizing the morphology pipeline.

This guide answers:

- **"Which code paths account for 90% of execution time?"** — Hot path analysis with real-world token distribution.
- **"What are the highest-ROI optimizations?"** — Ranked optimization opportunities with expected impact.
- **"How do I implement the known words index?"** — Complete algorithm with size/latency tradeoffs.
- **"How do I optimize root extraction for weak/hamzated/doubled roots?"** — Short-circuit strategies and heuristic ordering.
- **"How do I ensure my changes don't regress performance?"** — Benchmark harness, CI gates, and regression detection.
- **"How do performance characteristics change across deployment profiles?"** — Mobile vs. server vs. batch considerations.

### 1.2 Scope

**In scope:**

| Category | Coverage |
|----------|----------|
| **Hot path identification** | Token frequency distribution, per-operation latency budgets, pipeline bottlenecks |
| **Fast-path optimization** | Particle/pronoun lookup (KB-0005/KB-0006), hash index design, normalization short-circuit |
| **Known words index** | Pre-computed stem→root mapping, size/performance tradeoffs, cache invalidation |
| **Root extraction** | Triliteral fast path, weak root short-circuit, heuristic ordering by probability |
| **Wazan identification** | Pattern signature caching, candidate pruning, verb form priority ordering |
| **Feature extraction** | Bitfield packing optimization, validation short-circuit, inference rule ordering |
| **Syntax parsing** | Chart parsing optimization, beam search, constituent caching |
| **KB loading** | Memory-mapped files, lazy loading, hot/cold region splitting |
| **Caching architecture** | Multi-level caching, cache key design, invalidation strategy |
| **Concurrency** | Pipeline parallelism, per-token parallelism, lock-free KB access |
| **Benchmarking** | Standard corpus, measurement methodology, CI integration |

**Out of scope:**

| Topic | Covered By |
|-------|-----------|
| Performance targets (latency, throughput, memory) | SPEC-0001-C9 |
| Basic implementation algorithms (root extraction, wazan matching) | SPEC-0101 §4–§7 |
| Feature bitfield encoding specifics | KB-0007, SPEC-0102 |
| Compiled KB binary format | KB-0008 (particles), KB-0006 (pronouns) |
| GVM execution optimization | RFC-0003 |
| Pipeline-level caching (cross-module) | SPEC-0001-C5 |

### 1.3 Relationship to Other Specifications

```diff
  SPEC-0001-C9: Performance Targets
    │  Defines WHAT: latency budgets, throughput targets, memory limits
    ▼
  SPEC-0103: Performance Optimization (THIS DOCUMENT)
    │  Defines HOW: optimization strategies, code patterns, benchmarking
    ▼
  SPEC-0101: Morphology Engine
    │  Defines WHAT/WHY: algorithms, data structures, interfaces
    │  Optimizations applied TO these algorithms
    ▼
  Individual KB modules (KB-0005, KB-0006, KB-0008)
    │  Compiled format optimizations, hash index design
    ▼
  Implementation (Rust/C/other)
    │  Profiling, tuning, validation
```

### 1.4 Target Audience

- **Performance Engineers:** Optimizing the morphology pipeline to meet SPEC-0001-C9 targets. Must understand hot paths, caching strategies, and benchmarking methodology.
- **MOD-04/MOD-05 Developers:** Implementing the morphology and syntax engines. Must understand where optimization effort should be concentrated.
- **KB Compiler Engineers:** Designing the compiled KB binary format. Must understand access patterns and memory layout requirements.
- **Platform Engineers:** Porting AGOS to mobile, server, or embedded environments. Must understand deployment-specific tradeoffs.

---

## 2. Performance Philosophy & Principles

### 2.1 Core Principles

The following principles guide all performance optimization work in the morphology engine:

1. **Determinism is non-negotiable.** No optimization may introduce non-determinism. Caching is deterministic (keyed by input + config + KB version). No randomized algorithms in the hot path.

2. **Measure first, optimize second.** All optimization must be guided by profiling data from the standard benchmark corpus. Never optimize based on intuition alone.

3. **Optimize the common case.** ~70% of Arabic tokens are non-particle, non-pronoun content words requiring full analysis. ~15–25% are particles or pronouns (fast path). The remaining ~5–15% are clitics or punctuation. Optimizations should prioritize the common case while ensuring worst-case bounds are met.

4. **Cache invalidation is harder than caching.** Every cache MUST have a clear invalidation strategy based on input + configuration + KB version. Caches with stale data produce incorrect results — this is worse than being slow.

5. **Bounds are better than speed.** If an optimization cannot guarantee bounded memory or bounded latency, it is not acceptable. The morphology engine must never consume unbounded memory or loop indefinitely.

6. **Optimize hot paths, not cold paths.** Profile to identify the top 3–5 operations consuming >80% of execution time. Focus optimization effort there. Do not micro-optimize code paths that account for <1% of execution time.

### 2.2 Performance Hierarchy

When choosing between optimization strategies, apply this priority:

```
Priority 1: Algorithmic improvements
  → Better data structures (hash maps → perfect hashing)
  → Better algorithms (O(n²) → O(n log n))
  → Value: 10–100× improvement

Priority 2: Caching and memoization
  → Pre-compute expensive operations
  → Avoid recomputation for repeated inputs
  → Value: 2–10× improvement

Priority 3: Memory optimization
  → Better memory layout (cache-friendly structs)
  → Reduced allocations (arena allocation)
  → Memory-mapped files instead of loading
  → Value: 1.2–3× improvement

Priority 4: Low-level tuning
  → Inline hot functions
  → Branch prediction hints (likely/unlikely)
  → SIMD for character operations
  → Value: 1.1–1.5× improvement
```

### 2.3 Token Type Distribution

Understanding the distribution of token types in real-world Arabic text is essential for prioritizing optimizations:

| Token Type | Typical Frequency | Fast Path? | Analysis Cost | Optimization Priority |
|-----------|------------------|------------|---------------|---------------------|
| Particle (حرف) | 15–25% | Yes (KB-0005) | < 1 μs | ✅ Already fast; ensure lookup stays fast |
| Pronoun (ضمير) | 3–8% | Yes (KB-0006) | < 1 μs | ✅ Already fast |
| Sound verb (فعل صحيح) | 15–25% | No | 5–15 μs | 🔥 **HIGH** — most common content word |
| Weak verb (فعل معتل) | 5–10% | No | 10–30 μs | 🔥 **HIGH** — expensive but common |
| Sound noun (اسم صحيح) | 20–35% | No | 5–15 μs | 🔥 **HIGH** — most common POS |
| Broken plural noun | 3–8% | No | 10–25 μs | 🔥 **HIGH** — expensive but moderately common |
| Hamzated verb (مهموز) | 2–5% | No | 10–25 μs | Medium — less common |
| Doubled verb (مضاعف) | 1–3% | No | 8–20 μs | Low — less common |
| Proper noun (علم) | 2–5% | No | 2–10 μs | Medium — variable cost |
| Punctuation/numbers | 5–10% | N/A | < 1 μs | ✅ Already fast |
| Unknown | 1–3% | No | 20–200 μs | Low — worst case, but rare |

**Key insight:** Sound verbs and nouns account for ~50% of tokens and cost 5–15 μs each. Optimizing these by even 20% (saving 1–3 μs per token) gives a ~10 μs improvement per 10-token sentence.

### 2.4 Optimization ROI Matrix

| Optimization | Effort | Impact (μs/token) | Impact (sentence) | Risk |
|-------------|--------|-------------------|-------------------|------|
| Known words index | Medium | -3 to -5 μs | -45 to -75 μs | Low — straightforward hash map |
| Fast-path hash tuning | Low | -0.1 to -0.3 μs | -1.5 to -4.5 μs | Low — well-understood |
| Weak root short-circuit | Low | -2 to -5 μs | -10 to -25 μs | Low — 70% of roots are sound |
| Pattern signature caching | Medium | -2 to -5 μs | -10 to -25 μs | Low — pre-computed table |
| Lazy KB loading | Medium | Cold start: -80 MB | Init: -80 ms | Low — feature toggle |
| Memory-mapped KBs | Low | Cold start: -90% time | Init: -450 ms | Low — standard technique |
| Chart parsing beam search | High | — | -50% on long sentences | Medium — may miss parses |
| Result caching (pipeline) | Medium | — | -85% on repeated input | Medium — cache invalidation |
| SIMD normalization | Medium | -0.05 μs | -0.75 μs | Low — well-understood |
| Arena allocation | High | -0.2 μs per alloc | — | Low — standard technique |

---

## 3. Hot Path Analysis

### 3.1 Per-Token Latency Breakdown

For a typical Arabic sentence of 10 tokens (7 content words requiring full analysis):

```diff
  For each token (typical 10-token sentence):

  ┌──────────────────────────────────────────────┐
  │  1. Normalization + preprocessing            │   1 μs (5%)
  │     ├── NFKC normalization                   │   0.3 μs
  │     ├── Tatweel removal                      │   0.1 μs
  │     └── Clitic detection/stripping           │   0.6 μs
  ├──────────────────────────────────────────────┤
  │  2. Fast-path particle check (KB-0005)       │   0.3 μs (1.5%)
  │     ├── Hash computation                     │   0.03 μs
  │     ├── Bucket lookup                        │   0.005 μs
  │     └── String comparison                    │   0.02 μs
  ├──────────────────────────────────────────────┤
  │  3. Fast-path pronoun check (KB-0006)        │   0.3 μs (1.5%)
  ├──────────────────────────────────────────────┤
  │  4. Known word lookup                        │   1 μs (5%)
  │     (pre-computed index, O(1) hash map)      │
  ├──────────────────────────────────────────────┤
  │  5. Root extraction (KB-0001)                │   5 μs (25%)
  │     ├── Triliteral extraction                │   1 μs
  │     ├── Weak root handling (if weak)         │   +5 μs
  │     ├── Hamza handling (if hamzated)         │   +3 μs
  │     ├── Doubled handling (if doubled)        │   +2 μs
  │     ├── KB-0001 trie lookup                  │   1 μs
  │     └── Confidence scoring                   │   0.5 μs
  ├──────────────────────────────────────────────┤
  │  6. Wazan identification (KB-0002)           │   8 μs (40%)
  │     ├── Pattern signature computation        │   1 μs
  │     ├── KB-0002 hash lookup                  │   0.5 μs
  │     ├── Verb form matching                   │   4 μs
  │     ├── Noun pattern matching                │   3 μs
  │     ├── Weak variant matching                │   +3 μs
  │     └── Candidate ordering                   │   0.5 μs
  ├──────────────────────────────────────────────┤
  │  7. Feature extraction (KB-0007)             │   3 μs (15%)
  │     ├── POS-specific extraction              │   1 μs
  │     ├── Feature bitfield packing             │   0.5 μs
  │     ├── KB-0007 validation                   │   1 μs
  │     └── Defaults and inference               │   0.5 μs
  ├──────────────────────────────────────────────┤
  │  8. Ambiguity set generation                 │   1 μs (5%)
  │     ├── Combination generation               │   0.5 μs
  │     └── Confidence aggregation               │   0.5 μs
  └──────────────────────────────────────────────┘
                     Total: ~20 μs per content token
                     Total: ~140 μs per 10-token sentence (7 content words)
```

### 3.2 Bottleneck Identification

Based on the hot path analysis:

| Rank | Operation | % of Time | Cumulative | Optimization Potential |
|------|-----------|-----------|------------|----------------------|
| 1 | **Wazan identification** | 40% | 40% | Pattern signature caching; verb form priority ordering |
| 2 | **Root extraction** | 25% | 65% | Known words index; weak root short-circuit |
| 3 | **Feature extraction** | 15% | 80% | Bitfield packing optimization; validation short-circuit |
| 4 | **Normalization** | 5% | 85% | SIMD; parallel normalization |
| 5 | **Known word lookup** | 5% | 90% | Perfect hashing; level-2 hot cache |
| 6 | **Ambiguity set generation** | 5% | 95% | Lazy generation; pruned combinations |

**Key finding:** Wazan identification (40%) + root extraction (25%) account for **65% of per-token analysis time**. These are the highest-ROI optimization targets.

### 3.3 Token-Type-Specific Costs

| Token Type | Time (μs) | % of Total | How Many per Sentence | Cumulative Time |
|-----------|-----------|------------|----------------------|-----------------|
| Particle (fast path) | 0.6 | 0.4% | 2 | 1.2 μs |
| Pronoun (fast path) | 0.6 | 0.4% | 0.5 | 0.3 μs |
| Sound verb | 12 | 8% | 1.5 | 18 μs |
| Sound noun | 12 | 8% | 2.5 | 30 μs |
| Weak verb | 22 | 15% | 0.7 | 15.4 μs |
| Broken plural noun | 20 | 14% | 0.5 | 10 μs |
| Proper noun | 5 | 3% | 0.3 | 1.5 μs |
| Punctuation/etc | 0.5 | 0.3% | 1 | 0.5 μs |
| **Total (10-token sentence)** | | | | **~77 μs** |

### 3.4 Cache Line Hot Spots

The following data structures must be cache-line optimized (64-byte alignment):

| Data Structure | Size | Cache Lines | Access Pattern | Optimization |
|---------------|------|-------------|----------------|-------------|
| KB-0005 hash bucket table | 2,048 B | 32 | Random (256 buckets) | Align to 64 B; store frequently-accessed buckets first |
| KB-0005 entry table | ~13 KB | ~200 | Random | 64-byte entries eliminate false sharing |
| KB-0006 entry table | ~4 KB | ~64 | Random | Same as above |
| Pattern signature cache | ~3 KB | ~48 | Hot — every wazan lookup | Keep in L1; align to 64 B |
| Known words index (hot entries) | ~60 KB | ~940 | Hot — most common stems | Memory-map; rely on OS page cache |
| KB-0001 root lookup | ~20–80 MB | ~300K–1.2M | Random trie traversal | Memory-map; page cache |

---

## 4. Fast-Path Optimization (Particles & Pronouns)

### 4.1 Current Performance

| Operation | Target | Current (Estimated) | Status |
|-----------|--------|-------------------|--------|
| KB-0005 particle lookup (hit) | < 500 ns | ~250 ns | ✅ Meets target |
| KB-0005 particle lookup (hit, p99) | < 2 μs | ~1 μs | ✅ Meets target |
| KB-0005 full miss | < 2 μs | ~700 ns | ✅ Meets target |
| KB-0006 pronoun lookup (hit) | < 500 ns | ~250 ns | ✅ Meets target |
| KB-0006 pronoun lookup (hit, p99) | < 2 μs | ~1 μs | ✅ Meets target |

**Current state:** Fast-path lookups already meet or exceed targets. Optimization effort should focus on **maintaining** this performance as KBs grow, not on making them faster.

### 4.2 Degradation Prevention

As the particle and pronoun KBs grow (new entries, more homographs), the following degradations could occur:

| Degradation | Cause | Prevention |
|-------------|-------|-----------|
| Hash collisions increase | More entries → more collisions within buckets | Expand bucket table from 256 to 512 buckets; use double hashing |
| Bucket chain length grows | Skewed distribution of particle texts | Use better hash function; manually tune seeds for distribution |
| Entry table exceeds L1 cache | More entries → entry table > 32 KB | Hot/cold splitting: frequently-used particles in first 64 entries |
| String comparison slows | Longer compound particle texts | Store first 8 bytes inline in entry table; compare pointer+offset |

### 4.3 Normalization Short-Circuit

The normalization step before fast-path lookup can be optimized:

```pseudo
Algorithm: fast_normalize_for_particle_lookup

Input:  raw_token (UTF-8 string)
Output: normalized (suitable for hash lookup)

Step 1: Quick Check (no normalization needed)
    ├── If token contains NO diacritics AND NO tatweel AND NO
    │   Arabic presentation forms:
    │   → Use raw token directly for hash computation
    │   → Skip normalization entirely (90% of tokens!)
    └── Only run full normalization for ~10% of tokens

Step 2: Hash-Only Normalization
    ├── If only tatweel present (U+0640):
    │   → Compute hash treating tatweel as ignored
    │   → No string copy needed
    └── If only diacritics present (FATHA, KASRA, DAMMA, etc.):
        → Compute hash skipping diacritic bytes
        → Use raw token with masked hash

Step 3: Full Normalization (fallback)
    ├── NFKC normalization
    ├── Diacritic stripping (if needed)
    ├── Clitic splitting (if needed)
    └── String copy to output buffer
```

**Expected improvement:** ~50% reduction in normalization time for the common case (no diacritics, no tatweel).

### 4.4 Clitic Splitting Optimization

Clitic splitting is required for tokens like `فَبِالْبَيْتِ` (fa-bi-l-bayti). The current algorithm tries prefixes one at a time. An optimized approach:

```pseudo
Algorithm: fast_clitic_split

Input:  token (UTF-8 string)
Output: (prefix_len, was_split)

Step 1: Fast path — check first byte only
    ├── Build a 256-entry lookup table indexed by first byte
    ├── If token[0] maps to a known clitic → return prefix length
    ├── If token[0] maps to NOT_A_CLITIC → return (0, false)
    └── Cost: 1 array lookup + 1 comparison (~2 ns)

Step 2: For two-byte clitics (ال)
    ├── Only reached if first byte is ل (U+0644)
    ├── Check second byte for ا (U+0627)
    └── Cost: 1 extra comparison (~1 ns)

Step 3: Compound clitics (wa + al = وال)
    ├── Check first 3 bytes for wal- prefix
    ├── Only reached for frequent compound forms
    └── Cost: rarely triggered
```

**Lookup table design:**

```c
// 256-entry lookup for single-byte clitic detection
// Indexed by first byte of UTF-8 token
// Value: 0 = not a clitic, 1+ = clitic length in bytes
static const uint8_t CLITIC_BYTE_TABLE[256] = {
    // 0x00–0xBF: Non-Arabic bytes → 0
    [0 ... 0xBF] = 0,

    // Arabic letters that are clitic prefixes:
    [0x88] = 1,  // و (U+0648) — wa-
    [0x81] = 1,  // ف (U+0641) — fa-
    [0x8A] = 1,  // ب (U+0628) — bi-
    [0x84] = 1,  // ل (U+0644) — li- (also al- if followed by ا)
    [0x8C] = 1,  // ك (U+0643) — ka-
    [0x8B] = 1,  // س (U+0633) — sa-
    [0x23] = 1,  // أ (U+0623) — a- (interrogative)

    // All other Arabic letters → 0
    [0x80 ... 0xFF] = 0,  // Simplified; real table has specific entries
};
```

### 4.5 Homograph Disambiguation Performance

For the common homograph cases (مَا, إِنْ, لَا, etc.):

```pseudo
Algorithm: fast_homograph_disambiguation

Design Principle:
  → Most homographs can be resolved with 2–3 context checks
  → Only run full scoring for the ~5% of tokens requiring it

Step 1: Pre-compute decision trees for each homograph group:
    ├── مَا (mā, 6 interpretations):
    │   ├── If next verb mood == jussive → conditional (HARF_SHART)
    │   ├── If sentence is interrogative → interrogative (HARF_ISTIFHAM)
    │   ├── If preceded by preposition → relative pronoun
    │   ├── If next verb tense == past → negative (HARF_NAFY)
    │   └── Else → masdar-forming (HARF_MASDARI) [default]
    │
    ├── إِنْ (in, 3 interpretations):
    │   ├── If next verb mood == jussive → conditional (HARF_SHART)
    │   ├── If used before لَام → إِنَّ (emphatic, different spelling)
    │   └── Else → negative (HARF_NAFY, rare)
    │
    └── لَا (lā, 3 interpretations):
        ├── If next verb mood == jussive → prohibition (HARF_NAHIYAH)
        ├── If next noun is indefinite accusative → generic negation
        └── Else → simple negative (HARF_NAFY) [default]

Step 2: Cache resolved homographs
    ├── Key: (token_text_hash, context_type, previous_token_pos)
    ├── Value: resolved particle_type
    └── Small LRU cache (64 entries) for repeated patterns
```

**Expected improvement:** Homograph resolution from ~1 μs to ~300 ns for common cases.

---

## 5. Known Words Index

### 5.1 Concept

The **known words index** is a pre-computed hash map mapping inflected stem forms to their root and pattern. It is the single highest-impact optimization in MOD-04 (estimated 3–5 μs saved per token).

### 5.2 Index Construction

The index is built at **KB compilation time** by enumerating all possible stem forms from KB-0001 through KB-0004:

```pseudo
Algorithm: build_known_words_index

Input:  KB-0001 (roots), KB-0002 (wazan), KB-0003 (verb forms),
        KB-0004 (noun patterns)
Output: hash_map<string, KnownWordEntry>

Step 1: Generate All Forms
    ├── For each root in KB-0001:
    │   ├── For each verb form I–XV in KB-0002:
    │   │   ├── For each conjugation slot in KB-0003:
    │   │   │   ├── Generate inflected form
    │   │   │   └── Index by (stem_text, root, form, slot_features)
    │   │   └── Generate weak-root variants
    │   └── For each noun pattern in KB-0004:
    │       ├── Generate derived noun forms
    │       ├── Generate broken plural forms
    │       └── Index each
    │
    ├── Handle weak roots separately:
    │   ├── Hollow (أجوف): generate forms with restored middle radical
    │   ├── Defective (ناقص): generate forms with restored final radical
    │   ├── Assimilated (مثال): generate forms with restored initial radical
    │   └── Doubled (مضاعف): generate both geminated and split forms
    │
    └── Handle particles and pronouns:
        ├── Include all KB-0005 entries (for fast-path verification)
        └── Include all KB-0006 entries (for fast-path verification)

Step 2: Deduplicate
    ├── Multiple roots may generate the same stem (homographs)
    ├── Store all (root, pattern) pairs for ambiguous stems
    └── Mark as ambiguous if count > 1

Step 3: Build Hash Map
    ├── Key: normalized stem text (NFKC, diacritics optional)
    ├── Value: KnownWordEntry (root, pattern, form, confidence)
    ├── Use perfect hashing if stems are known at compile time
    └── Otherwise use flat_hash_map (absl or equivalent)
```

### 5.3 Entry Format

```c
/// Entry in the known words index.
typedef struct KnownWordEntry {
    uint32_t root_id;                    // Index into KB-0001 trie
    uint32_t wazan_id;                   // Index into KB-0002 hash index
    uint8_t  pos;                        // Part of speech (from KB-0007)
    uint8_t  verb_form;                  // I–XV (0 if not a verb)
    uint8_t  noun_type;                  // masdar, participle, etc.
    uint8_t  root_type;                  // sound, hollow, defective, etc.
    uint16_t flags;                      // Ambiguity, weak variant, etc.
    uint16_t confidence;                 // 0–1000 (scaled for integer ops)
    uint32_t feature_bitfield;            // Pre-computed features (partial)
} KnownWordEntry;
// Total: 16 bytes per entry
```

### 5.4 Size vs. Performance Tradeoffs

| Level | Entries | Size | Lookup Time | Hit Rate | Memory |
|-------|---------|------|-------------|----------|--------|
| **Level 0 (None)** | 0 | 0 | N/A | 0% | Baseline |
| **Level 1 (Verb forms only)** | ~15,000 | ~240 KB | < 100 ns | ~30% | Negligible |
| **Level 2 (Verbs + common nouns)** | ~50,000 | ~800 KB | < 100 ns | ~55% | ~1 MB |
| **Level 3 (Full)** | ~100,000 | ~1.6 MB | < 100 ns | ~70% | ~2 MB |
| **Level 4 (Full + weak variants)** | ~200,000 | ~3.2 MB | < 150 ns | ~85% | ~4 MB |

**Recommended:** Level 2 (verbs + common nouns) for embedded/mobile. Level 3 (full) for server.

### 5.5 Cache Invalidation

The known words index must be invalidated when:

| Event | Action | Impact |
|-------|--------|--------|
| KB-0001 updated | Rebuild entire index | Full rebuild (~2–5 seconds) |
| KB-0002 updated | Rebuild entire index | Full rebuild |
| KB-0003 updated | Rebuild verb form section | Partial rebuild (~1 second) |
| KB-0004 updated | Rebuild noun section | Partial rebuild (~1 second) |
| School configuration changed | No rebuild needed | Index is school-agnostic |

### 5.6 Hot/Cold Splitting

Frequently accessed entries can be kept in a small hot cache:

```c
/// Small hot cache for most common stems.
/// Implemented as a fixed-size hash table with LRU eviction.
typedef struct HotWordCache {
    uint32_t entry_count;              // Current entries (max 1024)
    uint64_t keys[1024];              // Hash of stem text
    uint32_t values[1024];            // Index into known words index
    uint64_t access_counts[1024];     // For LRU eviction
    uint64_t total_accesses;           // Running counter for LRU
} HotWordCache;

// Expected hot cache hit rate: > 90% for common stems
// Hot cache size: 1024 × (8 + 4 + 8) ≈ 20 KB
```

---

## 6. Root Extraction Optimization

### 6.1 Current Performance

| Root Type | Current Time | Target | Gap |
|-----------|-------------|--------|-----|
| Sound triliteral | 4 μs | < 3 μs | ⚠️ 1 μs gap |
| Weak (hollow) | 15 μs | < 10 μs | ⚠️ 5 μs gap |
| Weak (defective) | 12 μs | < 10 μs | ⚠️ 2 μs gap |
| Weak (assimilated) | 10 μs | < 8 μs | ⚠️ 2 μs gap |
| Hamzated | 12 μs | < 10 μs | ⚠️ 2 μs gap |
| Doubled | 8 μs | < 8 μs | ✅ Meets target |
| Quadriliteral | 6 μs | < 5 μs | ⚠️ 1 μs gap |

### 6.2 Short-Circuit Strategy

The most impactful optimization is short-circuiting weak root handling when it is not needed:

```pseudo
Algorithm: fast_root_extraction

Input:  stem (string, normalized)
Output: RootCandidate[]

Step 1: Quick Classification
    ├── If stem contains NO weak letters (ا, و, ي, ى, ء):
    │   → Proceed directly to triliteral extraction
    │   → Skip all weak root heuristics
    │   → Covers ~70% of verb stems
    │   → Cost: 1 scan of stem (~0.1 μs)
    │
    ├── If stem contains weak letters:
    │   → Classify weakness type:
    │   │   ├── Medial ا/ى → hollow
    │   │   ├── Final ى/و/ي → defective
    │   │   ├── Initial و → assimilated
    │   │   ├── Contains ء → hamzated
    │   │   └── Shadda on C3 → doubled
    │   └── Apply specific handler for detected type
    │
    └── Cost: 1 additional scan (~0.15 μs)

Step 2: Extract Triliteral (fast path)
    ├── Step 2a: Try the 3 most common affix patterns FIRST
    │   ├── Most Arabic verbs follow Form I (فَعَلَ)
    │   ├── Strip common prefixes (ي, ت, أ, ن, س)
    │   └── Strip common suffixes (وا, ون, ين, ان, ات, ة, نا)
    │
    ├── Step 2b: Extract 3 consonants
    │   ├── Pick 3 consonants in order from the stem
    │   ├── C1 = first non-affix consonant
    │   ├── C2 = second non-affix consonant
    │   ├── C3 = third non-affix consonant
    │   └── Validate: C1, C2, C3 must be valid root letters
    │
    └── Step 2c: Look up in KB-0001 trie
        ├── Use iterative traversal (not recursive)
        ├── Pre-fetch child nodes
        └── If found: return with confidence = 0.9

Step 3: If not found (root not in KB-0001):
    ├── Try alternative affix analyses:
    │   ├── Different prefix split
    │   ├── Different suffix split
    │   └── Known edge cases (hamza seat ambiguity, etc.)
    └── If still not found: return empty
```

### 6.3 Heuristic Ordering

Root extraction heuristics should be ordered by probability of success:

```c
// Probability of each extraction method being correct
// (Estimated from Quranic + MSA corpus analysis)
typedef enum {
    EXTRACT_KNOWN_WORD      = 0,  // 70% — found in known words index
    EXTRACT_TRILITERAL      = 1,  // 15% — standard 3-consonant
    EXTRACT_WEAK_HOLLOW     = 2,  // 5%  — medial weak
    EXTRACT_WEAK_DEFECTIVE  = 3,  // 3%  — final weak
    EXTRACT_HAMZATED        = 4,  // 2%  — hamza as radical
    EXTRACT_DOUBLED         = 5,  // 2%  — geminate split
    EXTRACT_WEAK_ASSIMILATED = 6, // 1%  — initial weak
    EXTRACT_QUADRILITERAL   = 7,  // 1%  — 4-consonant
    EXTRACT_GUESS           = 8,  // 0.5% — fallback guess
} ExtractionMethod;

// Try in probability order, stop at first success
static const ExtractionMethod EXTRACTION_ORDER[] = {
    EXTRACT_KNOWN_WORD,
    EXTRACT_TRILITERAL,
    EXTRACT_WEAK_HOLLOW,
    EXTRACT_WEAK_DEFECTIVE,
    EXTRACT_HAMZATED,
    EXTRACT_DOUBLED,
    EXTRACT_WEAK_ASSIMILATED,
    EXTRACT_QUADRILITERAL,
    EXTRACT_GUESS,
};
```

### 6.4 KB-0001 Trie Optimization

The KB-0001 root trie is a critical data structure. Optimizations:

```c
/// Optimized trie node for KB-0001.
/// Packed into 8 bytes for cache efficiency.
typedef struct RootTrieNode {
    uint32_t children[2];    // Pair of (char, child_offset) entries
                             // Uses 16 bits per entry:
                             //   bits 0-7: character (Arabic letter index 0-35)
                             //   bits 8-31: child node offset
    uint16_t flags;          // 0=not_a_root, 1=is_root,
                             // 2=has_verb_forms, 4=has_nouns
    uint16_t root_id;        // Index into root data (0xFFFF if not a root)
} RootTrieNode;
// Total: 8 bytes per node

// Memory: ~2 MB for 250,000 nodes (covers ~20,000 roots)
```

**Optimization techniques:**

1. **Array-based (not pointer-based):** Nodes stored in a flat array indexed by offset. Eliminates pointer chasing overhead and improves cache locality.

2. **Child packing:** Two child entries per node (most nodes have 1–3 children). For nodes with >2 children, use an overflow pointer.

3. **Letter encoding:** Arabic letters mapped to 0–28 (ب=0, ت=1, ث=2, ..., ي=28). Hamza variants (أ, إ, ؤ, ئ) mapped to a single entry (29).

4. **Prefetching:** When traversing `C1 → C2`, prefetch `C3`'s node region.

### 6.5 Weak Root Short-Circuit Details

For the ~30% of verb stems containing weak letters:

```pseudo
Algorithm: detect_weakness_type

Input:  stem (string)
Output: WeaknessType (or NONE)

Step 1: Scan stem for weak letters
    ├── Scan left to right, single pass
    ├── Record positions of: ا, و, ي, ى, ء
    └── Cost: O(len) scan (~0.1 μs for 5-char stem)

Step 2: Classify weakness
    ├── If NO weak letters found:
    │   → return NONE
    │
    ├── If medial ا or ى (C1 + weak + C3 pattern):
    │   → return HOLLOW
    │
    ├── If final ي or ى (stem ends with weak):
    │   → return DEFECTIVE
    │
    ├── If final و (rare, archaic):
    │   → return DEFECTIVE_WAWI
    │
    ├── If initial و or ي:
    │   → return ASSIMILATED
    │
    ├── If contains ء at any position:
    │   → return HAMZATED
    │
    ├── If shadda on last consonant:
    │   → return DOUBLED
    │
    └── If shadda on middle consonant:
        → check if it's Form II (geminated middle radical)

Step 3: Apply type-specific handler
    ├── Map WeaknessType to handler function pointer
    ├── Execute handler (see SPEC-0101 §4.4–4.6 for algorithms)
    └── Return root candidates
```

---

## 7. Wazan Identification Optimization

### 7.1 Pattern Signature Caching

Pattern signatures are pre-computed for each (root_type, wazan_id) pair, eliminating repeated hash computation:

```pseudo
Algorithm: build_pattern_signature_cache

Input:  KB-0002 (wazan templates), KB-0007 (root types)
Output: cache[ROOT_TYPE_COUNT][WAZAN_COUNT] → u64

Step 1: Iterate over all root types (14 types from KB-0007 §7.5):
    ├── sound, mithal_wawi, mithal_yai, ajwaf_wawi, ajwaf_yai,
    │   naqis_wawi, naqis_yai, lafif_mafruq, lafif_makrun,
    │   hamzated_first, hamzated_middle, hamzated_last,
    │   doubled, quadriliteral_sound
    └── 14 root types × ~300–450 wazan entries

Step 2: For each (root_type, wazan) pair:
    ├── Generate a canonical stem by applying the wazan
    │   to a canonical root of the given type
    ├── Compute the pattern signature (u64)
    └── Store in cache array

Step 3: Collision resolution:
    ├── If two different (root_type, wazan) pairs produce
    │   the same signature:
    │   → Store both in a linked list at that cache slot
    │   → Linear search on collision (very rare)
    └── Expected collision rate: < 0.1%

Cache size: 14 × 450 × 8 bytes = ~50 KB
            (easily fits in L1/L2 cache)
```

### 7.2 Verb Form Priority Ordering

Verb forms should be tried in order of frequency:

```c
// Frequency-ordered verb forms for fast matching
// (Estimated from Quranic + MSA corpus analysis)
// Try in this order — stop at first match
static const uint8_t VERB_FORM_ORDER_BASRA[] = {
    1,   // Form I (فَعَلَ) — ~70% of all verb tokens
    2,   // Form II (فَعَّلَ) — ~8%
    3,   // Form III (فَاعَلَ) — ~5%
    4,   // Form IV (أَفْعَلَ) — ~4%
    5,   // Form V (تَفَعَّلَ) — ~3%
    6,   // Form VI (تَفَاعَلَ) — ~2%
    7,   // Form VII (اِنْفَعَلَ) — ~2%
    8,   // Form VIII (اِفْتَعَلَ) — ~3%
    9,   // Form IX (اِفْعَلَّ) — ~0.5%
    10,  // Form X (اِسْتَفْعَلَ) — ~2%
    // Forms XI–XV extremely rare (combined < 0.5%)
};

// School-specific orderings
static const uint8_t VERB_FORM_ORDER_KUFA[] = {
    1, 2, 3, 4, 5, 6, 8, 7, 9, 10  // Kufa prefers Form VIII before VII
};

static const uint8_t VERB_FORM_ORDER_ANDALUS[] = {
    1, 2, 4, 3, 5, 6, 8, 7, 9, 10  // Andalus prefers Form IV before III
};
```

**Expected improvement:** 70% of verb forms match on the first try (Form I), reducing average matching time from 4 μs to ~1 μs.

### 7.3 Noun Pattern Priority Ordering

```c
// Frequency-ordered noun patterns
static const uint8_t NOUN_PATTERN_ORDER[] = {
    // Masdars (verbal nouns) — most common derived noun type
    PATTERN_MASDAR_FORM_I,       // Form I masdar (~20% of derived nouns)

    // Active participles
    PATTERN_ISM_FAIL_FORM_I,     // فَاعِل  (~15%)
    PATTERN_ISM_FAIL_FORM_II,    // مُفَعِّل (~5%)

    // Passive participles
    PATTERN_ISM_MAFUL_FORM_I,    // مَفْعُول (~10%)
    PATTERN_ISM_MAFUL_FORM_II,   // مُفَعَّل (~3%)

    // Nouns of place/time
    PATTERN_ISM_MAKAN,           // مَفْعَل  (~8%)

    // Feminine masdar
    PATTERN_MASDAR_FEM,          // فِعَالَة (~5%)

    // Instrument nouns
    PATTERN_ISM_ALAH,            // مِفْعَال (~2%)

    // Broken plurals
    PATTERN_BROKEN_PLURAL_FI_AL,     // فِعَال   (~8%)
    PATTERN_BROKEN_PLURAL_FU_UL,     // فُعُول   (~5%)
    PATTERN_BROKEN_PLURAL_AF_AL,     // أَفْعَال (~5%)
    PATTERN_BROKEN_PLURAL_FU_AL,     // فُعَّل   (~2%)

    // Other patterns (descending frequency)
    // ...
};
```

### 7.4 Candidate Pruning

Before exhaustive matching, prune impossible candidates:

```pseudo
Algorithm: prune_wazan_candidates

Input:  stem (string), root (RootCandidate)
Output: pruned wazan candidates

Prune rules (in order, cheap checks first):

Rule 1: Length check
    ├── If stem length in consonants < 3 → no verb form matches
    ├── If stem length in consonants > 6 → no standard verb form matches
    └── Cost: O(1)

Rule 2: Prefix check
    ├── If stem starts with ي, ت, أ, ن → imperfect verb (Forms I–X)
    ├── If stem starts with ا → possible imperative or Form VII/VIII/IX
    ├── If stem starts with م → possible noun (participle, place, etc.)
    ├── If stem starts with ت → possible Form V/VI or noun
    └── Cost: 1 character lookup

Rule 3: Suffix check
    ├── If stem ends with ون → nominative masculine plural
    ├── If stem ends with ين → accusative/genitive masculine plural
    ├── If stem ends with ان → dual nominative
    ├── If stem ends with ين → dual accusative/genitive
    ├── If stem ends with ات → feminine plural
    ├── If stem ends with ة → feminine singular
    └── Cost: 1–2 character lookup

Rule 4: Root type filter
    ├── If root is hollow (أجوف):
    │   → Forms I, III, IV, VI, VII, VIII, X are possible
    │   → Forms II, V are modified (geminated middle radical)
    ├── If root is defective (ناقص):
    │   → All forms possible, but with different endings
    ├── If root is assimilated (مثال):
    │   → Initial و drops in Form I imperfect
    └── Cost: 1 lookup + bitmask check

Expected pruning: Eliminate 60–80% of candidates before signature matching
```

---

## 8. Feature Extraction & Bitfield Packing

### 8.1 Bitfield Packing Optimization

The feature bitfield is a critical hot-path operation that runs for every token:

```c
/// Optimized bitfield packing for features.
/// Pre-compute feature masks and shifts for each feature ID.
typedef struct FeatureField {
    uint64_t mask;          // Bitmask for this feature's field
    uint8_t  shift;         // Right shift to extract value
    uint8_t  width;         // Width in bits
    uint8_t  bit_offset;    // Starting bit position
    uint8_t  reserved;
} FeatureField;

// Pre-compute once at initialization
static FeatureField FEATURE_FIELDS[FEATURE_COUNT];

void init_feature_fields(void) {
    // From KB-0007 §10.1: 64-bit feature bitfield layout
    // See SPEC-0102 §3 for complete encoding

    // POS (bits 0-3)
    FEATURE_FIELDS[FEATURE_POS] = (FeatureField){
        .mask = 0x000000000000000FULL,
        .shift = 0,
        .width = 4,
        .bit_offset = 0,
    };

    // Gender (bits 4-5)
    FEATURE_FIELDS[FEATURE_GENDER] = (FeatureField){
        .mask = 0x0000000000000030ULL,  // bits 4-5
        .shift = 4,
        .width = 2,
        .bit_offset = 4,
    };

    // ... (all 19 features from KB-0007)

    // Reserved bits (50-63) for plugins
    // Not pre-computed — plugin features set via dedicated API
}

/// Pack a feature value into the bitfield.
/// Inline this function — it's called for every feature.
static inline uint64_t pack_feature(
    uint64_t bitfield,
    uint8_t  feature_id,
    uint32_t value
) {
    FeatureField* field = &FEATURE_FIELDS[feature_id];
    // Clear existing value
    bitfield &= ~field->mask;
    // Set new value
    bitfield |= ((uint64_t)value << field->bit_offset) & field->mask;
    return bitfield;
}

/// Unpack a feature value from the bitfield.
static inline uint32_t unpack_feature(
    uint64_t bitfield,
    uint8_t feature_id
) {
    FeatureField* field = &FEATURE_FIELDS[feature_id];
    return (uint32_t)((bitfield >> field->bit_offset) & ((1ULL << field->width) - 1));
}
```

**Optimization notes:**
- `pack_feature` and `unpack_feature` should be `inline` or `#[inline(always)]`.
- For the common case (packing 3–5 features per token), unroll the loop manually.
- Pre-compute `FEATURE_FIELDS` as a constant array at compile time if possible.

### 8.2 Validation Short-Circuit

Feature validation (KB-0007 rules) can be optimized:

```pseudo
Algorithm: fast_feature_validation

Step 1: Pre-compute valid feature combinations
    ├── At KB-0007 load time, build a set of all valid
    │   (pos, feature, value) triples
    ├── Store as a single 64-bit bloom filter for O(1) check
    └── False positives: rare, do full check only on bloom filter match

Step 2: POS-specific validation sets
    ├── For verbs: validate tense, mood, voice, person, number, gender, form
    ├── For nouns: validate case, state, number, gender, noun_type
    ├── For particles: validate particle_type, governance
    └── For pronouns: validate pronoun_type, person, number, gender

Step 3: Skip validation for inferred/default features
    ├── Features with confidence < 0.5 may have invalid combinations
    │   (that's expected — MOD-07 will prune them)
    └── Only validate features with confidence >= 0.5

Expected improvement: Validation time from ~1 μs to ~200 ns (80% reduction).
```

### 8.3 Default and Inference Optimization

```pseudo
Algorithm: apply_feature_defaults_fast

Step 1: Pre-compute defaults per POS
    ├── Build array: default_features[POS_COUNT] → FeatureSet
    ├── At init time, compute once from KB-0007 inference rules
    └── Apply as bitwise OR instead of individual field sets

Step 2: Inference rule ordering
    ├── Rules that apply to >50% of tokens should be checked first:
    │   ├── INF-001: Default verb tense = past (if no tense marker)
    │   ├── INF-002: Default noun state = indefinite (if no ال)
    │   ├── INF-003: Default noun case = nominative (if no marker)
    │   └── INF-004: Default gender = masculine (if no feminine marker)
    │
    ├── Rules that apply to <10% of tokens should be checked last:
    │   ├── INF-010: Broken plural → feminine singular agreement
    │   └── INF-012: Diptote → limited case system
    └── Expected improvement: 80% of inferences applied in 1 check
```

---

## 9. Syntax Parsing Optimization

### 9.1 Chart Parsing Optimization

MOD-05 uses a chart-parsing approach (CKY/Earley-inspired). The following optimizations apply:

```pseudo
Algorithm: optimize_chart_parsing

Optimization 1: Beam Search
    ├── Keep at most BEAM_WIDTH parse candidates at each chart cell
    ├── Default BEAM_WIDTH = 8 (configurable per deployment)
    ├── Prune: sort by confidence, keep top K
    └── Effect: O(n³) → O(n² × beam) in practice

Optimization 2: Pre-filter constituents
    ├── Before chart parsing, identify guaranteed constituents:
    │   ├── Prepositional phrases: harf_jarr + noun (genitive)
    │   ├── Idafa pairs: noun + noun (genitive)
    │   ├── Verb chunks: verb + subject pronoun
    │   └── Particle + verb: mood-governing particle + verb
    ├── These are O(n) to identify and reduce effective sentence length
    └── Effect: Effective n reduced by 20–40%

Optimization 3: Constituent caching
    ├── Cache results of sub-parse for each (start, end, type) triple
    ├── Many sentences share common sub-structures
    ├── Use a flat hash map keyed by (start << 16 | end << 8 | type)
    └── Effect: 30–50% reduction on repeated patterns

Optimization 4: Sentence length limit
    ├── Hard limit: MAX_SENTENCE_LENGTH = 200 tokens
    ├── Soft limit: if > 50 tokens, use greedy parse (no backtracking)
    └── Effect: Bounded worst-case time
```

### 9.2 Sentence Segmentation Optimization

```pseudo
Algorithm: fast_sentence_segmentation

Step 1: Boundary detection via byte-level scan
    ├── Scan for punctuation (., ?, !, ;, Quranic markers)
    ├── Use a 256-entry lookup table for O(1) boundary detection
    └── Cost: 1 scan of token array (~0.1 μs per token)

Step 2: Conjunction-based boundary detection (optional)
    ├── Only run if no explicit punctuation found
    ├── Check tokens against a small set (~10) of known sentence-initial
    │   conjunctions (وَ, فَ, ثُمَّ, بَل, لٰكِنَّ)
    └── Cost: O(n) token type check

Step 3: Sentence type classification
    ├── Check first content word's POS
    ├── Use a pre-computed decision tree:
    │   ├── If POS == verb → verbal sentence
    │   ├── If POS == noun/pronoun/adjective → nominal sentence
    │   ├── If POS == particle → check particle type:
    │   │   ├── HARF_SHART → conditional
    │   │   ├── HARF_ISTIFHAM → interrogative
    │   │   └── Other → check next word
    │   └── Default → ambiguous
    └── Cost: O(1) — single POS lookup
```

### 9.3 Candidate Pruning During Parse

```pseudo
Algorithm: prune_parse_candidates

Rule 1: Agreement filter
    ├── Subject-verb: person MUST match; gender SHOULD match
    ├── Noun-adjective: number, gender, case, state MUST match
    ├── Idafa (construct): N1 must be indefinite, N2 must be genitive
    └── If any MUST constraint violated → discard candidate

Rule 2: Case frame filter
    ├── Verb's transitivity determines how many objects exist
    ├── Intransitive verb → no object → discard candidates with objects
    ├── Transitive verb → need at least one accusative noun
    └── Ditransitive → need two accusative nouns

Rule 3: School-specific filter (applied last)
    ├── If school == Basra: reject indefinite mubtada'
    ├── If school == Kufa: accept indefinite mubtada'
    └── These filters are cheap bitmask checks

Expected pruning: 60–80% of candidates removed before full parse
```

---

## 10. KB Loading & Memory Management

### 10.1 Memory-Mapped KB Loading

All 7 KBs should be loaded via memory-mapped files rather than parsed into process memory:

```c
/// Memory-map a compiled KB file.
/// Returns a pointer to the mapped data, or NULL on error.
void* mmap_kb_file(const char* path, size_t* out_size) {
    int fd = open(path, O_RDONLY);
    if (fd < 0) return NULL;

    struct stat st;
    fstat(fd, &st);
    *out_size = (size_t)st.st_size;

    // Map entire file — the OS handles paging
    void* mapped = mmap(NULL, *out_size,
                        PROT_READ, MAP_PRIVATE,
                        fd, 0);
    close(fd);

    if (mapped == MAP_FAILED) return NULL;

    // Optional: advise kernel of access pattern
    // For KBs with regular access patterns:
    madvise(mapped, *out_size, MADV_RANDOM); // KB-0001 (random trie access)
    madvise(mapped, *out_size, MADV_SEQUENTIAL); // String tables

    return mapped;
}
```

**Benefits:**
- No parsing at load time — data is accessed in-place.
- The OS virtual memory manager handles paging — unused pages stay on disk.
- Cold start memory: only the header is loaded initially. Pages are demand-loaded.
- Multiple processes sharing the same KB file share physical memory pages.

### 10.2 KB Load Time Budget

| KB | Format | Size (Compact) | Load Method | Load Time |
|----|--------|---------------|-------------|-----------|
| KB-0001 | Trie binary | 20 MB | mmap + verify header | < 5 ms |
| KB-0002 | Hash index | 10 MB | mmap + verify header | < 3 ms |
| KB-0003 | Table binary | 15 MB | mmap + verify header | < 3 ms |
| KB-0004 | Table binary | 10 MB | mmap + verify header | < 3 ms |
| KB-0005 | Hash index | 2 MB | mmap + verify header | < 1 ms |
| KB-0006 | Hash index | 1 MB | mmap + verify header | < 1 ms |
| KB-0007 | Feature map | 1 MB | mmap + verify header | < 1 ms |
| **Total** | | **~59 MB** | **Sequential mmap** | **< 20 ms** |

### 10.3 Lazy KB Loading

Not all KBs are needed for every analysis:

```c
/// KB load strategy — configured per deployment.
typedef enum {
    KB_LOAD_EAGER,     // All KBs at init (server deployment)
    KB_LOAD_LAZY,      // Load on first use (mobile/embedded)
    KB_LOAD_DEMAND,    // Load only what's needed for current analysis
                        // (batch processing, well-known corpus)
} KBLoadStrategy;

/// KB demand loader — tracks which KBs are loaded.
typedef struct KBLoader {
    bool kb_loaded[7];           // Which KBs are loaded
    void* kb_data[7];            // mmap'd data pointers
    size_t kb_sizes[7];          // File sizes
    char kb_paths[7][256];       // File paths
    uint64_t load_timestamps[7]; // When each was loaded (ns resolution)
} KBLoader;

/// Ensure a KB is loaded before access.
/// Inline this function — called before every KB access.
static inline const void* kb_load_on_demand(
    KBLoader* loader,
    KbId kb_id
) {
    if (!loader->kb_loaded[kb_id]) {
        uint64_t start = get_time_ns();
        loader->kb_data[kb_id] = mmap_kb_file(
            loader->kb_paths[kb_id],
            &loader->kb_sizes[kb_id]
        );
        loader->kb_loaded[kb_id] = (loader->kb_data[kb_id] != NULL);
        loader->load_timestamps[kb_id] = get_time_ns() - start;
    }
    return loader->kb_data[kb_id];
}
```

**Memory savings with lazy loading:**

| Scenario | KBs Loaded | Memory | Savings |
|----------|-----------|--------|---------|
| Short text (particles/pronouns only) | KB-0005, KB-0006, KB-0007 | ~4 MB | ~55 MB (93%) |
| Particle/pronoun + known word hit | KB-0005, KB-0006, KB-0007, KWI | ~5 MB | ~54 MB (92%) |
| Verb analysis needed | All 7 KBs | ~59 MB | Baseline |
| Noun analysis needed | KB-0001, KB-0002, KB-0004, KB-0007 | ~41 MB | ~18 MB (31%) |

### 10.4 Hot/Cold Region Splitting

For KBs with uneven access patterns, split into hot and cold regions:

```c
/// Hot/cold split for KB entry tables.
/// Frequently accessed entries are stored first (hot region),
/// infrequently accessed entries are stored after (cold region).
typedef struct HotColdSplit {
    uint32_t hot_count;          // Number of hot entries
    uint32_t cold_count;         // Number of cold entries
    uint32_t hot_size_bytes;     // Size of hot region
    uint32_t cold_size_bytes;    // Size of cold region
    uint8_t  hot_data[];         // Hot region (aligned to 64 B)
    // Cold data follows immediately after hot region
} HotColdSplit;
```

**Which entries go in the hot region:**
- For KB-0005 (Particles): The 20 most common particles (فِي, مِنْ, لِ, بِ, إِنَّ, كَانَ, لَا, etc.).
- For KB-0006 (Pronouns): The 10 most common pronouns (هُوَ, هِيَ, هُمْ, أَنَا, أَنْتَ, etc.).
- For KB-0001 (Roots): The 1,000 most common roots (kept in a separate hot trie).
- For KB-0002 (Wazan): Form I patterns (the most common by far).

**Expected effect:** Hot region stays in L1/L2 cache (>90% hit rate), cold region only loaded on demand.

### 10.5 Arena Allocation for Per-Request Memory

Use arena allocation to avoid per-allocation overhead:

```c
/// Simple bump allocator for per-request memory.
/// Freed as a whole at the end of each request.
typedef struct Arena {
    uint8_t* base;              // Base of allocated memory
    uint8_t* current;           // Current allocation position
    size_t   capacity;          // Total capacity
    size_t   peak_used;         // Peak usage (for statistics)
} Arena;

/// Initialize an arena with a fixed capacity.
void arena_init(Arena* arena, uint8_t* buffer, size_t capacity) {
    arena->base = buffer;
    arena->current = buffer;
    arena->capacity = capacity;
    arena->peak_used = 0;
}

/// Allocate from the arena — O(1), no lock, no free.
static inline void* arena_alloc(Arena* arena, size_t size) {
    // Align to 8 bytes
    size_t aligned = (size + 7) & ~7;
    uint8_t* ptr = arena->current;
    arena->current += aligned;

    // Track peak usage
    size_t used = (size_t)(arena->current - arena->base);
    if (used > arena->peak_used) {
        arena->peak_used = used;
    }

    // Bounds check (optional, for debugging)
    assert(arena->current <= arena->base + arena->capacity);
    return ptr;
}

/// Reset the arena — O(1), frees everything at once.
static inline void arena_reset(Arena* arena) {
    arena->current = arena->base;
}
```

**Per-request arena size:** 6 KB (maximum per-token working memory × 10 tokens).

---

## 11. Caching Architecture

### 11.1 Cache Levels

The morphology engine uses a three-level cache hierarchy:

```diff
  Level 1: In-Memory Token Cache (L1)
  ├── Scope: Per-sentence (within single MOD-04/MOD-05 analysis)
  ├── Contents: Normalized tokens, extracted roots, matched wazans
  ├── Size: N tokens × ~200 bytes = ~2 KB per sentence
  ├── Eviction: End of sentence (arena reset)
  └── Purpose: Avoid recomputing the same root extraction twice

  Level 2: In-Process Result Cache (L2)
  ├── Scope: Across sentences within the same process
  ├── Contents: (stem_hash, context) → MorphologicalAnalysis
  ├── Size: Configurable (64 MB default, 256 MB server)
  ├── Eviction: LRU + KB version change
  └── Purpose: Cache results for repeated stems (common in Arabic)

  Level 3: Cross-Process Shared Cache (L3)
  ├── Scope: Across processes (server topology, Redis/memcached)
  ├── Contents: (text_hash + KB_versions + config_hash) → FullAnalysis
  ├── Size: Configurable (200 MB default)
  ├── Eviction: LRU + TTL (5 minutes default)
  └── Purpose: Share cache across server instances
```

### 11.2 Cache Key Design

```c
/// Cache key for morphological analysis results.
/// 24 bytes total — fits in a single cache line.
typedef struct MorphCacheKey {
    uint64_t text_hash;              // xxHash64 of normalized token text
    uint64_t context_hash;           // Hash of surrounding context (prev/next tokens)
    uint32_t config_hash;            // Hash of MOD-04 config (school, flags)
    uint8_t  kb_version_hash[4];     // Hash of KB versions (KB-0001 through KB-0007)
    uint16_t token_position_hint;    // Position in sentence (for context-dependent disambiguation)
    uint8_t  padding[2];             // Pad to 24 bytes
} MorphCacheKey;
// Total: 8 + 8 + 4 + 4 + 2 + 2 = 28 bytes (rounded to 32)

/// Cache value — compact morphological analysis result.
typedef struct MorphCacheValue {
    uint64_t feature_bitfield;        // Packed features (64-bit)
    uint32_t root_id;                // KB-0001 root index
    uint32_t wazan_id;               // KB-0002 wazan index
    uint8_t  pos;                    // Part of speech
    uint8_t  confidence;             // 0–255 (scaled)
    uint8_t  ambiguity_count;        // Number of alternative analyses
    uint8_t  flags;                  // Misc flags (was_fast_path, etc.)
    uint16_t analysis_time_ns;       // Time to compute (for statistics)
} MorphCacheValue;
// Total: 8 + 4 + 4 + 1 + 1 + 1 + 1 + 2 = 22 bytes (rounded to 24)
```

### 11.3 Cache Invalidation Strategy

| Event | Cache Action | Scope | Latency Impact |
|-------|-------------|-------|---------------|
| KB version change | Invalidate all | L2, L3 | ~1 second warm-up |
| Config change (school) | Invalidate by config_hash | L2 | ~100 ms warm-up |
| Process restart | L2 empty (cold start) | L2 | ~500 ms warm-up |
| TTL expiry | Invalidate individual entry | L3 | Minimal |
| LRU eviction | Evict oldest entry | L2, L3 | Minimal |

### 11.4 Estimated Cache Hit Rates

| Cache Level | Token-Type Hit Rate | Sentence-Level Hit Rate | Memory Savings |
|-------------|--------------------|------------------------|----------------|
| L1 (per-sentence) | 5% (duplicate stems) | 2% | ~2 KB |
| L2 (in-process) | 40% (common stems) | 20% | ~64 MB |
| L3 (cross-process) | 20% (shared patterns) | 10% | ~200 MB |
| **Combined** | **~65%** | **~32%** | **~266 MB** |

### 11.5 Cache Warm-Up

For server deployments, pre-warm caches during initialization:

```pseudo
Algorithm: warm_up_caches

Step 1: Warm the known words index
    ├── Load pre-computed hash map (already at init)
    ├── No additional action needed — index is always hot
    └── Cost: 0 (already loaded)

Step 2: Warm the pattern signature cache
    ├── Pre-compute all (root_type, wazan_id) → signature mappings
    ├── Store as a flat array (see §7.1)
    └── Cost: ~5 ms at init

Step 3: Warm KB-0001 hot root trie
    ├── Load the top 1,000 roots into the hot trie
    ├── These account for ~60% of root lookups
    └── Cost: ~200 KB loaded at init (negligible time)

Step 4: Warm the L2 result cache (optional)
    ├── Process the 1,000 most common Arabic stems
    ├── Store results in L2 cache
    └── Cost: ~10 ms, 100 KB cache entries
```

---

## 12. Concurrency & Parallelism

### 12.1 Pipeline Parallelism

The MOD-04 pipeline is sequential per token (each step depends on the previous), but different tokens are independent:

```pseudo
Algorithm: parallel_token_analysis

Step 1: Batch tokens into groups of 8–16
    ├── Each token in the batch is independent
    ├── Process tokens in parallel using a thread pool
    └── Synchronize after each pipeline stage

Step 2: Stage-level parallelism
    ├── Stage 1 (normalization): SIMD-friendly, 8 tokens at once
    ├── Stage 2 (fast path): Independent per token
    ├── Stage 3 (root extraction): Independent per token
    ├── Stage 4 (wazan matching): Independent per token
    ├── Stage 5 (feature extraction): Independent per token
    └── Stage 6 (ambiguity set): Requires all tokens done

Step 3: MOD-05 (syntax) parallelism
    ├── Sentences are independent within a document
    ├── Parse multiple sentences in parallel
    └── Each sentence parse is single-threaded (O(n³) chart parsing)
```

**Expected speedup:** 3–5× on 4-core CPU for a 10-token sentence.

### 12.2 Lock-Free KB Access

All KB data structures are read-only after initialization. Multiple threads can access them without locks:

```c
/// Thread-safe KB access — no locks needed.
/// All KB data is read-only (mmap'd, immutable).
///
/// Thread safety guarantee:
///   - KB data is loaded once at init (or lazily, with double-checked locking)
///   - No thread modifies KB data after initialization
///   - All threads can read simultaneously without synchronization
///
/// Thread-local state is used for per-request data:
///   - Arena allocator (per thread)
///   - Hot word cache (per thread, for better cache locality)
///   - Timing statistics (per thread, merged at request end)
```

### 12.3 Thread Pool Design

```c
/// Thread pool for parallel morphology analysis.
/// Fixed-size pool (N threads), work-stealing queue.
typedef struct MorphThreadPool {
    uint32_t thread_count;            // Number of worker threads
    uint32_t active_jobs;             // Currently executing jobs
    JobQueue  queue;                  // Lock-free work queue
    ThreadContext* contexts;          // Per-thread context (arena, stats)
} MorphThreadPool;

/// Per-thread context — no sharing between threads.
typedef struct ThreadContext {
    Arena     arena;                  // Per-thread arena (6 KB)
    HotWordCache hot_cache;           // Per-thread hot word cache
    uint64_t  stats_analysis_count;   // Per-thread statistics
    uint64_t  stats_total_time_ns;
    uint64_t  stats_cache_hits;
} ThreadContext;
```

### 12.4 SIMD Optimization Opportunities

| Operation | SIMD Width | Expected Speedup | Complexity |
|-----------|-----------|-----------------|------------|
| UTF-8 normalization (NFKC) | 16 bytes (SSE) | 2–3× | Low — byte scanning |
| Diacritic removal | 16 bytes (SSE) | 3–5× | Low — byte masking |
| Hash computation (CityHash) | Scalar (SSE for CRC) | 1.5× | Medium |
| Arabic character classification | 16 bytes (SSE) | 4–8× | Low — lookup table |
| Feature bitfield operations | Scalar (bitwise) | Already optimal | N/A |
| String comparison | 16 bytes (SSE) | 2–4× | Low — memcmp |

---

## 13. Benchmarking Methodology

### 13.1 Standard Benchmark Corpus

All performance measurements use the **AGOS Standard Benchmark Corpus** (ASBC-1.0), defined in SPEC-0001-C9 §2.1:

| Sub-corpus | Sentences | Tokens | Characteristic |
|------------|-----------|--------|----------------|
| Quranic | 500 | ~6,000 | Short, classical, highly structured |
| Hadith | 500 | ~8,000 | Medium, classical |
| Classical Poetry | 200 | ~4,000 | Long, complex, rare vocabulary |
| MSA News | 500 | ~10,000 | Modern, medium complexity |
| MSA Literature | 300 | ~6,000 | Varied length |
| Mixed Ambiguity | 100 | ~2,000 | Max morphological ambiguity |
| Edge Cases | 100 | ~500 | Edge cases, fragments |
| **Total** | **2,200** | **~36,500** | |

### 13.2 Microbenchmarks

```c
/// Microbenchmark for a specific operation.
/// Run N iterations, measure min/avg/max/p99.
typedef struct MicroBenchmark {
    const char* name;               // Benchmark name
    uint32_t iterations;            // Number of iterations (10,000+)
    uint64_t min_time_ns;           // Minimum observed time
    uint64_t max_time_ns;           // Maximum observed time
    uint64_t total_time_ns;         // Total time across all iterations
    uint64_t time_p99_ns;           // 99th percentile
    double   ops_per_second;        // Throughput
} MicroBenchmark;

// Required microbenchmark suite:
// 1. KB-0005 particle_lookup (hit, miss, homograph)
// 2. KB-0006 pronoun_lookup (hit, miss)
// 3. Known words index lookup (hit, miss)
// 4. Normalization (no change, diacritic stripping, NFKC)
// 5. Triliteral root extraction (sound, weak, hamzated, doubled)
// 6. Wazan pattern matching (Form I hit, multiple forms, noun)
// 7. Feature bitfield pack/unpack
// 8. Sentence segmentation (short, medium, long)
// 9. Chart parsing (5-token, 10-token, 20-token sentence)
```

### 13.3 End-to-End Benchmarks

```bash
# Run full end-to-end benchmark
agos benchmark run \
    --suite=full \
    --corpus=asbc-1.0 \
    --output=results.json \
    --warm-up=100     # Warm up with 100 sentences first

# Run morphology-only benchmark (MOD-04 only)
agos benchmark run \
    --suite=morphology-only \
    --corpus=asbc-1.0 \
    --output=morph-results.json

# Run syntax-only benchmark (MOD-05 only, with cached morphology)
agos benchmark run \
    --suite=syntax-only \
    --corpus=asbc-1.0 \
    --output=syntax-results.json

# Run with specific configuration
agos benchmark run \
    --suite=full \
    --corpus=asbc-1.0 \
    --school=kufa \
    --cache-level=l2 \
    --threads=4
```

### 13.4 Measurement Methodology

| Metric | Method | Aggregation |
|--------|--------|-------------|
| **p50 latency** | Median of 10,000 consecutive requests | Median |
| **p95 latency** | 95th percentile of 10,000 requests | Percentile |
| **p99 latency** | 99th percentile of 10,000 requests | Percentile |
| **Throughput** | Requests per second over 5-minute steady-state | Mean |
| **Memory (RSS)** | Peak RSS during benchmark | Maximum |
| **Cache hit rate** | Hits / (hits + misses) | Ratio |
| **Instruction count** | Hardware performance counters | Per operation |

All benchmarks:
- Cold cache: First run after process start.
- Warm cache: After 1,000 similar sentences (unless testing cold start).
- Single-threaded (unless testing parallelism).
- Reported with 95% confidence intervals.

### 13.5 Benchmark Output Format

```json
{
    "suite": "AGOS-Morph-Perf-v1",
    "version": "1.0.0",
    "timestamp": "2026-07-15T12:00:00Z",
    "pipeline_version": "1.0.0",
    "knowledge_versions": {
        "KB-0001": "1.0.0",
        "KB-0002": "1.0.0",
        "KB-0003": "1.0.0",
        "KB-0004": "1.0.0",
        "KB-0005": "1.0.0",
        "KB-0006": "1.0.0",
        "KB-0007": "1.0.0"
    },
    "hardware": {
        "cpu": "AMD EPYC 7763",
        "cores": 64,
        "ram_gb": 256,
        "disk": "NVMe SSD",
        "os": "Ubuntu 22.04"
    },
    "results": {
        "mod04_morphology": {
            "latency_p50_us": 12.3,
            "latency_p95_us": 45.6,
            "latency_p99_us": 89.1,
            "throughput_tokens_per_sec": 85000,
            "memory_mb": 120,
            "cache_hit_ratio": 0.65
        },
        "mod05_syntax": {
            "latency_p50_us": 120.0,
            "latency_p95_us": 450.0,
            "latency_p99_us": 890.0,
            "throughput_sentences_per_sec": 2500,
            "memory_mb": 50,
            "cache_hit_ratio": 0.30
        },
        "microbenchmarks": {
            "particle_lookup_hit_ns": 250,
            "pronoun_lookup_hit_ns": 240,
            "known_word_lookup_ns": 800,
            "triliteral_extraction_us": 3.5,
            "weak_root_extraction_us": 10.2,
            "wazan_signature_lookup_us": 0.8,
            "full_morph_analysis_simple_us": 8.5,
            "full_morph_analysis_complex_us": 35.0
        }
    }
}
```

---

## 14. Performance Regression Prevention

### 14.1 CI Performance Gates

```yaml
# .github/workflows/perf-morphology.yml
name: MOD-04/MOD-05 Performance Regression
on: [push, pull_request]
jobs:
  benchmark:
    runs-on: [self-hosted, benchmark-runner]
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release
      - name: Run microbenchmarks
        run: agos benchmark run --suite=micro --output=micro.json
      - name: Run end-to-end
        run: agos benchmark run --suite=e2e --output=e2e.json
      - name: Compare against baseline
        run: |
          agos benchmark compare micro.json baseline-micro.json
          agos benchmark compare e2e.json baseline-e2e.json
      - name: Check performance budget
        run: agos benchmark check-budget e2e.json .perf-budget.yaml
      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: perf-results
          path: "*.json"
```

### 14.2 Performance Budget

```yaml
# .perf-budget.yaml
# Per-operation latency budgets (hard limits)

mod04:
  particle_lookup_ns:
    soft: 500        # Warning if > 500 ns
    hard: 2000       # Fail if > 2 μs
  pronoun_lookup_ns:
    soft: 500
    hard: 2000
  triliteral_root_extraction_us:
    soft: 5
    hard: 15
  weak_root_extraction_us:
    soft: 10
    hard: 30
  wazan_signature_lookup_us:
    soft: 2
    hard: 5
  full_morph_analysis_simple_us:
    soft: 15
    hard: 50
  full_morph_analysis_complex_us:
    soft: 100
    hard: 300

mod05:
  sentence_segmentation_us:
    soft: 10
    hard: 50
  short_sentence_parse_us:     # 5 tokens
    soft: 50
    hard: 200
  medium_sentence_parse_us:    # 10 tokens
    soft: 200
    hard: 1000
  long_sentence_parse_us:      # 20 tokens
    soft: 1000
    hard: 5000

memory:
  mod04_rss_mb:
    soft: 150
    hard: 300
  mod05_rss_mb:
    soft: 60
    hard: 150
  combined_rss_mb:
    soft: 200
    hard: 400
```

### 14.3 Regression Detection Rules

```c
/// Compare benchmark results against baseline.
/// Returns FAIL if any hard limit is exceeded.
typedef enum {
    REGRESS_PASS,           // All within limits
    REGRESS_WARNING,        // Soft limit exceeded
    REGRESS_FAIL,           // Hard limit exceeded
} RegressionResult;

RegressionResult check_regression(
    const BenchmarkResults* current,
    const BenchmarkResults* baseline,
    const PerformanceBudget* budget
) {
    RegressionResult result = REGRESS_PASS;

    for (int i = 0; i < current->metric_count; i++) {
        const MetricResult* cur = &current->metrics[i];
        const MetricResult* base = &baseline->metrics[i];
        const MetricBudget* bud = &budget->metrics[i];

        // Check hard limit
        if (cur->p50 > bud->hard_limit) {
            log_error("HARD LIMIT EXCEEDED: %s = %.2f (limit = %.2f)",
                      cur->name, cur->p50, bud->hard_limit);
            result = REGRESS_FAIL;
        }

        // Check regression compared to baseline
        if (base->p50 > 0) {
            double regression_pct = ((double)cur->p50 - base->p50) / base->p50 * 100.0;

            if (regression_pct > 10.0) {
                // > 10% regression
                if (regression_pct > 20.0) {
                    log_error("REGRESSION > 20%%: %s increased %.1f%%",
                              cur->name, regression_pct);
                    result = REGRESS_FAIL;
                } else {
                    log_warn("REGRESSION > 10%%: %s increased %.1f%%",
                             cur->name, regression_pct);
                    result = (result == REGRESS_FAIL) ? REGRESS_FAIL : REGRESS_WARNING;
                }
            }
        }

        // Check soft limit
        if (cur->p50 > bud->soft_limit && result != REGRESS_FAIL) {
            log_warn("SOFT LIMIT EXCEEDED: %s = %.2f (limit = %.2f)",
                     cur->name, cur->p50, bud->soft_limit);
            result = (result == REGRESS_FAIL) ? REGRESS_FAIL : REGRESS_WARNING;
        }
    }

    return result;
}
```

### 14.4 Baseline Management

| Event | Action | New Baseline |
|-------|--------|-------------|
| Initial release (v1.0.0) | Run full benchmark → save as baseline | `baseline-v1.0.0.json` |
| Minor release (v1.1.0) | Compare against v1.0.0 baseline | Update if within budget |
| Major release (v2.0.0) | Run full benchmark on release branch | `baseline-v2.0.0.json` |
| KB update | Run microbenchmarks only | Update micro-baseline |
| Critical fix | Compare against last release baseline | No change |

---

## 15. Profiling & Diagnostics

### 15.1 Built-in Profiling

The morphology engine includes a built-in statistical profiler:

```c
/// Statistical profiler for MOD-04 operations.
/// Tracks time spent in each operation type.
/// Zero-overhead when disabled (compile-time flag).
typedef struct MorphProfiler {
    // Per-operation timing (nanosecond accumulation)
    uint64_t time_normalization_ns;
    uint64_t time_fast_path_particles_ns;
    uint64_t time_fast_path_pronouns_ns;
    uint64_t time_known_word_lookup_ns;
    uint64_t time_root_extraction_ns;
    uint64_t time_wazan_matching_ns;
    uint64_t time_feature_extraction_ns;
    uint64_t time_ambiguity_generation_ns;

    // Call counts
    uint64_t count_normalization;
    uint64_t count_fast_path_particles;
    uint64_t count_fast_path_pronouns;
    uint64_t count_known_word_lookup;
    uint64_t count_root_extraction;
    uint64_t count_wazan_matching;
    uint64_t count_feature_extraction;
    uint64_t count_ambiguity_generation;

    // Cache statistics
    uint64_t cache_hits;
    uint64_t cache_misses;

    // KB access statistics
    uint64_t kb_access_count[7];     // Per-KB access count
    uint64_t kb_page_faults[7];      // Per-KB OS page faults (Linux only)

    // Timing snapshots
    uint64_t last_checkpoint_ns;     // For delta measurements
} MorphProfiler;
```

### 15.2 Diagnostic Commands

```bash
# Print profile summary after analysis
agos analyze --profile --text="السلام عليكم"

# Profile output example:
# ┌──────────────────────────────────────────┬────────────┬──────────┐
# │ Operation                                │ Time (μs)  │   Count  │
# ├──────────────────────────────────────────┼────────────┼──────────┤
# │ Normalization                            │      1.2   │       4  │
# │ Fast-path particles (KB-0005)            │      0.8   │       4  │
# │ Fast-path pronouns (KB-0006)             │      0.5   │       4  │
# │ Known word lookup                        │      2.1   │       3  │
# │ Root extraction                          │     12.5   │       3  │
# │   ├─ Triliteral (sound)                  │      3.2   │       2  │
# │   ├─ Weak (hollow)                       │      8.1   │       1  │
# │   └─ KB-0001 trie lookup                 │      1.2   │       3  │
# │ Wazan matching                           │     15.3   │       3  │
# │ Feature extraction                       │      5.7   │       4  │
# │ Ambiguity generation                     │      1.8   │       4  │
# ├──────────────────────────────────────────┼────────────┼──────────┤
# │ Total (MOD-04)                           │     40.1   │       4  │
# └──────────────────────────────────────────┴────────────┴──────────┘

# Detailed cache statistics
agos analyze --cache-stats --text="السلام عليكم"

# Memory map visualization
agos analyze --memory-map

# KB access pattern heatmap
agos analyze --kb-heatmap --corpus=sample.txt
```

### 15.3 Performance Observation Points

The following points in the code MUST include timing instrumentation (disable in production, enable in debug):

```c
// Observation points (enabled with AGOS_PROFILE compile flag)

// 1. Token normalization start/end
PROFILE_CHECKPOINT("normalization_start");
// ... normalization code ...
PROFILE_CHECKPOINT("normalization_end");

// 2. KB-0005 lookup
PROFILE_CHECKPOINT("particle_lookup");
ParticleMatch match = particle_lookup(kb, text, len);
PROFILE_CHECKPOINT("particle_lookup_done");

// 3. KB-0001 trie traversal
PROFILE_CHECKPOINT("root_trie_start");
// ... trie traversal ...
PROFILE_CHECKPOINT("root_trie_end");

// 4. Wazan signature matching
PROFILE_CHECKPOINT("wazan_match_start");
// ... signature matching ...
PROFILE_CHECKPOINT("wazan_match_end");

// 5. Feature bitfield packing
PROFILE_CHECKPOINT("feature_pack_start");
bitfield = pack_features(features, count);
PROFILE_CHECKPOINT("feature_pack_end");

// 6. Sentence parse
PROFILE_CHECKPOINT("parse_start");
// ... chart parsing ...
PROFILE_CHECKPOINT("parse_end");
```

---

## 16. Deployment-Specific Optimizations

### 16.1 Mobile/Embedded (Class 1)

| Optimization | Impact | Configuration |
|-------------|--------|---------------|
| Compact KBs (Level 1) | ~59 MB total | `--kb-level=compact` |
| Lazy KB loading | ~4 MB baseline | `--kb-load=lazy` |
| No L3 cache | Skip Redis | `--cache-level=l2` |
| Single-threaded | No thread pool overhead | `--threads=1` |
| Minimal homograph disambiguation | Skip rare homographs | `--homograph-level=common` |
| Small hot word cache | 256-entry hot cache | `--hot-cache=256` |
| No profiling | Zero runtime overhead | `--no-profile` |

**Expected performance:** MOD-04 < 50 μs per token (worst case), total < 100 ms per sentence.

### 16.2 Server (Class 2)

| Optimization | Impact | Configuration |
|-------------|--------|---------------|
| Full KBs (Level 2) | ~189 MB | `--kb-level=full` |
| Eager KB loading | All at init | `--kb-load=eager` |
| L2 + L3 cache | Redis for shared cache | `--cache-level=l3` |
| 4 threads | ~3× throughput | `--threads=4` |
| Full homograph disambiguation | All homographs | `--homograph-level=full` |
| Large hot word cache | 4K-entry hot cache | `--hot-cache=4096` |
| Optional profiling | `--profile=sampling` | `--profile=sampling` |

**Expected performance:** MOD-04 < 20 μs per token (avg), throughput > 10K tokens/s.

### 16.3 Batch (Class 3)

| Optimization | Impact | Configuration |
|-------------|--------|---------------|
| Compact KBs (Level 1) | ~59 MB | `--kb-level=compact` |
| Demand KB loading | Per-corpus KB set | `--kb-load=demand` |
| No cache (unless repeated data) | Skip cache for linear throughput | `--cache-level=none` |
| 8+ threads | Max throughput | `--threads=8` |
| No homograph disambiguation | Pass all candidates | `--homograph-level=none` |
| Pre-computed results | Full corpus cache | `--corpus-cache=precompute` |

**Expected performance:** Throughput > 100K tokens/s per node.

### 16.4 Educational/Research (Class 4)

| Optimization | Impact | Configuration |
|-------------|--------|---------------|
| Full KBs (Level 2) | ~189 MB | `--kb-level=full` |
| Eager KB loading | All at init | `--kb-load=eager` |
| Full profiling | All diagnostics enabled | `--profile=full` |
| Debug tracing | Step-by-step analysis log | `--trace=verbose` |
| Example generation | Include example lookup | `--include-examples` |

**Expected performance:** ~2× slower than server profile (profiling overhead).

---

## 17. Cross-References

### 17.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C9 | Performance Targets & Constraints | Defines latency budgets, throughput targets, memory limits that this guide optimizes for |
| SPEC-0101 §4 | Root Extraction Subsystem | Algorithms optimized in §6 of this guide |
| SPEC-0101 §5 | Wazan Identification Subsystem | Algorithms optimized in §7 of this guide |
| SPEC-0101 §6 | Feature Extraction Subsystem | Algorithms optimized in §8 of this guide |
| SPEC-0101 §9 | MOD-04 Performance & Optimization | Performance targets for MOD-04 operations |
| SPEC-0101 §13 | Ambiguity Management Subsystem | Ambiguity set generation optimized in §8.5 |
| SPEC-0102 §3 | POS Feature Reference | Feature bitfield encoding optimized in §8.1 |
| SPEC-0102 §14 | Plugin Extension Mechanism | Reserved bit usage for particle features |
| KB-OVERVIEW §4 | Combined Resource Budgets | KB sizes and performance budgets |
| KB-0005 §20.4 | Particle Performance Requirements | Fast-path lookup targets (§4 of this guide) |
| KB-0006 §17.4 | Pronoun Performance Requirements | Fast-path lookup targets (§4 of this guide) |
| KB-0007 §10 | Feature Bitfield Layout | Bitfield packing specification (§8 of this guide) |
| KB-0008 §9 | MOD-04 Fast-Path Integration | Fast-path implementation (§4 of this guide) |
| KB-0008 §14 | Performance Model | Particle lookup latency budget |
| RFC-0003 | Grammar Virtual Machine | GVM execution optimization (related) |
| SPEC-0301 | Grammar Runtime | Runtime performance targets |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | Cache key design references IR schemas |

### 17.2 External References

| Reference | Relevance |
|-----------|-----------|
| **Google Abseil flat_hash_map** | Reference hash map implementation for known words index |
| **CityHash (Google)** | 64-bit hash function for hash index and cache keys |
| **xxHash** | Alternative hash function for cache keys |
| **CKY Algorithm** | Chart parsing foundation for MOD-05 |
| **Earley Algorithm** | Alternative chart parsing approach |
| **Google Benchmark** | C++ microbenchmarking library (reference methodology) |
| **Criterion.rs** | Rust microbenchmarking library |
| **Linux perf** | Performance counter analysis for hot path identification |
| **FlameGraph** | Visualization for profiling data |
| **Cachegrind/Valgrind** | Cache simulation for analyzing cache miss patterns |

---

## Progress Summary

**SPEC-0103: Morphology Engine Performance Optimization Guide**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Performance Philosophy & Principles | ✓ COMPLETE |
| 3 | Hot Path Analysis | ✓ COMPLETE |
| 4 | Fast-Path Optimization (Particles & Pronouns) | ✓ COMPLETE |
| 5 | Known Words Index | ✓ COMPLETE |
| 6 | Root Extraction Optimization | ✓ COMPLETE |
| 7 | Wazan Identification Optimization | ✓ COMPLETE |
| 8 | Feature Extraction & Bitfield Packing | ✓ COMPLETE |
| 9 | Syntax Parsing Optimization | ✓ COMPLETE |
| 10 | KB Loading & Memory Management | ✓ COMPLETE |
| 11 | Caching Architecture | ✓ COMPLETE |
| 12 | Concurrency & Parallelism | ✓ COMPLETE |
| 13 | Benchmarking Methodology | ✓ COMPLETE |
| 14 | Performance Regression Prevention | ✓ COMPLETE |
| 15 | Profiling & Diagnostics | ✓ COMPLETE |
| 16 | Deployment-Specific Optimizations | ✓ COMPLETE |
| 17 | Cross-References | ✓ COMPLETE |

---

*End of SPEC-0103*
