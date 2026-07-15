# KB-0008: Particles Database — Developer Reference & Compiled Module

| **Field** | **Value** |
|---|---|
| **KB ID** | KB-0008 |
| **Title** | Particles Database — Developer Reference & Compiled Module |
| **Version** | 1.0.0 |
| **Status** | Draft |
| **Depends on** | KB-0005 (Particles — Linguistic Content), SPEC-0101 (Morphology Engine), KB-0007 (Feature Taxonomy) |
| **Related KBs** | KB-0006 (Pronouns — companion fast-path KB), KB-0007 (Features — POS particle encoding) |
| **License** | AGOS Specification License v1.0 |

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Compiled Binary Format](#3-compiled-binary-format)
4. [Perfect Hash Index](#4-perfect-hash-index)
5. [Entry Table](#5-entry-table)
6. [String Table](#6-string-table)
7. [Particle Lookup API](#7-particle-lookup-api)
8. [Governance Resolution API](#8-governance-resolution-api)
9. [MOD-04 Fast-Path Integration](#9-mod-04-fast-path-integration)
10. [MOD-05 Syntactic Integration](#10-mod-05-syntactic-integration)
11. [Homograph Disambiguation Engine](#11-homograph-disambiguation-engine)
12. [Normalization & Preprocessing](#12-normalization--preprocessing)
13. [KB Compilation Pipeline](#13-kb-compilation-pipeline)
14. [Performance Model](#14-performance-model)
15. [Testing & Validation](#15-testing--validation)
16. [Cross-References](#16-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

KB-0008 is the **developer-oriented reference** for the AGOS Particle System — the compiled binary module and API that powers the **MOD-04 fast-path particle lookup**. Where KB-0005 defines the *linguistic content* (which particles exist, their grammatical functions, governance rules), KB-0008 defines the *implementation* (compiled binary format, lookup algorithms, API signatures, integration patterns).

KB-0008 answers:
- **"How is the particle KB compiled from source YAML into a binary hash index?"**
- **"What is the exact binary layout of the compiled particle module?"**
- **"What API does MOD-04 use for O(1) particle lookup?"**
- **"How does the governance resolution engine determine case/mood effects?"**
- **"How are homographs disambiguated with contextual scoring?"**
- **"What performance targets must the compiled module meet?"**

### 1.2 Scope

**In scope:**

| Category | Coverage |
|----------|----------|
| **Compiled binary format** | Complete header, hash index, entry table, string table layout |
| **Hash index** | Perfect hash construction, bucket table design, collision resolution |
| **Entry schema** | Compact binary entry format, all fields packed into 64 bytes |
| **Lookup API** | `lookup()` / `lookup_all()` / `resolve_governance()` signatures |
| **Fast-path integration** | MOD-04 Step 3.1 flow, clitic handling, early exit |
| **Homograph engine** | Contextual scoring system, disambiguation algorithm |
| **Normalization** | Unicode NFC, tatweel removal, diacritic stripping, clitic splitting |
| **Compilation pipeline** | YAML → binary hash index, validation steps |
| **Performance model** | Latency budgets, memory mapping, cache behavior |
| **Testing** | Conformance tests, regression suites, benchmark harness |

**Out of scope:**

| Topic | Covered By |
|-------|-----------|
| Linguistic particle content (meanings, types, examples) | KB-0005 (Particles — Linguistic Content) |
| Arabic grammar rules for particles | KB-0005 §5–15 |
| Full grammatical analysis algorithms | SPEC-0101 (Morphology Engine) |
| Rule engine particle predicates | RFC-0004 (Arabic Grammar Rule DSL) |
| Explanation engine particle descriptions | SPEC-0501 (Explanation Engine) |
| Feature bitfield encoding for particle POS | KB-0007, SPEC-0102 |

### 1.3 Relationship to KB-0005

| Aspect | KB-0005 (Linguistic) | KB-0008 (Developer Reference) |
|--------|----------------------|-------------------------------|
| **Purpose** | What particles exist and how they work grammatically | How to compile, load, and query particle data |
| **Audience** | Linguists, data maintainers, grammar authors | Pipeline developers, MOD-04/05 implementers |
| **Format** | Human-readable YAML schemas, grammar explanations | Binary layouts, C structs, API definitions |
| **Content** | 13 particle categories with full grammar tables | Hash index design, lookup algorithms, performance targets |
| **Output** | Source YAML files in `/knowledge/KB-0005/` | Compiled `.agos-kb` binary + `libparticles` API |

### 1.4 Target Audience

- **MOD-04 Developers:** Implementing the fast-path particle check (Step 3.1). Must understand the lookup API, normalization, and early exit logic.
- **KB Compiler Engineers:** Building the `agos kb compile` pipeline for particles. Must understand the binary format, perfect hash construction, and validation.
- **Plugin Authors:** Creating dialectal or domain-specific particle extensions. Must understand the plugin hook points in the lookup chain.
- **Performance Engineers:** Optimizing the fast path for < 500 ns lookup. Must understand the memory layout, cache behavior, and hot paths.

### 1.5 Design Principles

1. **O(1) lookup is non-negotiable.** The particle fast path is the first check for every token in MOD-04. Any slowdown there multiplies across the entire pipeline. The compiled module MUST provide worst-case O(1) lookup.

2. **Deterministic compilation.** Same source YAML + same compiler version = byte-for-byte identical binary. No randomization in hash construction, no non-deterministic ordering.

3. **Memory-map friendly.** The compiled binary is designed for direct `mmap()` loading — no parsing, no memory allocation at load time. The header contains all offsets needed for O(1) access into every section.

4. **Smallest possible footprint.** Particles are a closed class (~200 entries). The compiled binary SHOULD fit in ~2 MB (compact) to ~5 MB (full) — the smallest of all AGOS KBs.

5. **Graceful degradation.** If a particle is not found in the hash index, the fast path falls through to root extraction with zero additional overhead beyond the lookup miss itself.

---

## 2. Architecture Overview

### 2.1 System Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    AGOS Particle System                         │
│                                                                │
│  ┌─────────────────────┐       ┌───────────────────────────┐  │
│  │  KB-0005 Source     │──────►│  KB Compiler              │  │
│  │  (YAML files)       │       │  (agos kb compile)        │  │
│  │                     │       │  ┌─────────────────────┐  │  │
│  │  prepositions.yaml  │       │  │ 1. Parse YAML       │  │  │
│  │  conjunctions.yaml  │       │  │ 2. Validate schema  │  │  │
│  │  subjunctive.yaml   │       │  │ 3. Normalize text   │  │  │
│  │  jussive.yaml       │       │  │ 4. Build hash index │  │  │
│  │  conditional.yaml   │       │  │ 5. Serialize binary │  │  │
│  │  interrogative.yaml │       │  │ 6. Compute checksum │  │  │
│  │  negative.yaml      │       │  └─────────────────────┘  │  │
│  │  vocative.yaml      │       │                           │  │
│  │  inna-sisters.yaml  │       └───────────┬───────────────┘  │
│  │  kana-sisters.yaml  │                   │                  │
│  │  answer-exception.   │                   ▼                  │
│  │    yaml             │       ┌───────────────────────────┐  │
│  │  masdar-forming.yaml│       │  KB-0008 Compiled Binary  │  │
│  │  other.yaml         │       │  (.agos-kb file)          │  │
│  └─────────────────────┘       │  ┌─────────────────────┐  │  │
│                                │  │ HEADER (64 bytes)   │  │  │
│  ┌─────────────────────┐       │  │ HASH INDEX          │  │  │
│  │  Homograph Data     │──────►│  │ ENTRY TABLE         │  │  │
│  │  (KB-0005 §16)      │       │  │ STRING TABLE        │  │  │
│  └─────────────────────┘       │  └─────────────────────┘  │  │
│                                └───────────────────────────┘  │
│                                        │                      │
│                                        ▼                      │
│  ┌────────────────────────────────────────────────────────┐   │
│  │              libparticles API                           │   │
│  │                                                         │   │
│  │  ┌────────────────┐  ┌─────────────────────────────┐   │   │
│  │  │ lookup()       │  │ resolve_governance()        │   │   │
│  │  │ lookup_all()   │  │ disambiguate_homographs()   │   │   │
│  │  │ normalize()    │  │ split_clitics()             │   │   │
│  │  └────────────────┘  └─────────────────────────────┘   │   │
│  └────────────────────────────────────────────────────────┘   │
│                                        │                      │
│                     ┌──────────────────┼──────────────────┐   │
│                     ▼                  ▼                  ▼   │
│              ┌────────────┐    ┌────────────┐    ┌──────────┐ │
│              │ MOD-04     │    │ MOD-05     │    │ Plugins  │ │
│              │ Fast Path  │    │ Syntactic  │    │ (dialect │ │
│              │ (Step 3.1) │    │ Governance │    │  extend) │ │
│              └────────────┘    └────────────┘    └──────────┘ │
└────────────────────────────────────────────────────────────────┘
```

### 2.2 Module Architecture

The compiled particle module is organized into four contiguous sections:

```
Compiled Module (.agos-kb):
┌──────────────────────────────────────────────┐
│  HEADER (64 bytes, fixed size)                │
│  ├── Magic: "AGOSKB08" (8 bytes)             │
│  ├── Version (6 bytes)                       │
│  ├── Flags (2 bytes)                         │
│  ├── Particle count (4 bytes)                │
│  ├── Homograph group count (4 bytes)          │
│  ├── Offsets (4 × 4 = 16 bytes)              │
│  ├── Checksum (32 bytes)                     │
│  └── Reserved (8 bytes, zero)                │
├──────────────────────────────────────────────┤
│  HASH INDEX (variable)                       │
│  ├── Bucket table (256 × 4 = 1024 bytes)     │
│  ├── Hash entries (N × 8 bytes)              │
│  └── Overflow chain (if needed)              │
├──────────────────────────────────────────────┤
│  ENTRY TABLE (N × 64 bytes)                  │
│  ├── Entry 0: packed particle data           │
│  ├── Entry 1: ...                           │
│  ├── ...                                    │
│  └── Entry N-1: ...                         │
├──────────────────────────────────────────────┤
│  STRING TABLE (variable)                     │
│  ├── Length-prefixed UTF-8 strings           │
│  ├── Particle texts, meanings, usage notes  │
│  └── Example data, disambiguation hints      │
└──────────────────────────────────────────────┘
```

### 2.3 Data Flow: Token → Particle Match

```
Input: Token text (UTF-8 string)

1. NORMALIZE token text
   ├── Remove tatweel/kashida
   ├── Apply NFKC Unicode normalization
   ├── Strip leading clitic prefixes (optional toggle)
   └── Strip diacritics for fuzzy lookup (optional toggle)

2. HASH normalized text
   ├── Compute 64-bit city/xxHash of UTF-8 bytes
   ├── Map to bucket: hash % 256
   └── Probe bucket chain

3. MATCH against candidate entries
   ├── Compare full text (byte-for-byte)
   ├── If match → return particle entry
   ├── If no match → try normalized variant (no diacritics)
   └── If still no match → return NOT_FOUND (fall through)

4. DISAMBIGUATE (if homograph)
   ├── Collect all matching entries
   ├── Score each by syntactic context
   └── Return ranked candidates

5. RESOLVE GOVERNANCE
   ├── Read particle's governance type
   ├── Apply case/mood effect to following word
   └── Return GovernanceEffect
```

### 2.4 Fast-Path Position in MOD-04

```
MOD-04: MorphologicalParser — For Each Token
┌─────────────────────────────────────────────────────┐
│                                                     │
│  Step 3.1: FAST PATH — Particles (KB-0008)         │
│  ├── Normalize token                                │
│  ├── lookup(normalized_token)                       │
│  ├── If found:                                      │
│  │   ├── Record particle type + governance          │
│  │   ├── Set POS = particle in feature bitfield     │
│  │   ├── Skip root extraction (particles have none) │
│  │   └── Continue to next token                     │
│  │                                                  │
│  Step 3.2: FAST PATH — Pronouns (KB-0006)          │
│  ├── lookup(normalized_token)                       │
│  ├── If found: skip root extraction                 │
│  │                                                  │
│  Step 3.3+: MAIN PATH — Root Extraction            │
│  ├── Only reached for non-particle, non-pronoun     │
│  └── tokens                                         │
└─────────────────────────────────────────────────────┘
```

---

## 3. Compiled Binary Format

### 3.1 File Identification

| Property | Value |
|----------|-------|
| **File extension** | `.agos-kb` |
| **Magic bytes** | `0x41474F534B423038` = `"AGOSKB08"` (8 bytes) |
| **MIME type** | `application/x-agos-kb` |

### 3.2 Header Layout (64 bytes)

```c
struct ParticleKBHeader {
    // ── Identity (14 bytes) ──
    uint8_t  magic[8];              // "AGOSKB08"
    uint16_t version_major;         // Major version
    uint16_t version_minor;         // Minor version
    uint16_t version_patch;         // Patch version

    // ── Flags (2 bytes) ──
    uint16_t flags;                 // Bitmask (see below)

    // ── Counts (12 bytes) ──
    uint32_t particle_count;        // Total particle entries
    uint32_t homograph_group_count; // Number of homograph groups
    uint32_t string_count;          // Number of strings in string table

    // ── Section Offsets (16 bytes) ──
    uint32_t hash_index_offset;     // Byte offset to hash index section
    uint32_t entry_table_offset;    // Byte offset to entry table section
    uint32_t string_table_offset;   // Byte offset to string table section
    uint32_t total_file_size;       // Total file size in bytes

    // ── Integrity (32 bytes) ──
    uint8_t  checksum_sha256[32];   // SHA-256 of sections (not header)

    // ── Reserved (8 bytes) ──
    uint8_t  reserved[8];           // Must be zero
};
// Total: 64 bytes (8+6+2+4+4+4+4+4+4+4+32+8)
```

**Flags bitmask:**

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `HAS_HOMOGRAPH_DATA` | Homograph disambiguation data is present |
| 1 | `HAS_VERBOSE_STRINGS` | Full meaning strings included (Level 2) |
| 2 | `HAS_EXAMPLE_DATA` | Usage examples included |
| 3 | `HAS_DISAMBIGUATION_HINTS` | Disambiguation hint strings included |
| 4 | `HAS_BACKWARD_COMPAT_INDEX` | Legacy forward index for older KB-0005 schema |
| 5–15 | — | Reserved (must be zero) |

### 3.3 Section Offsets

All offsets are absolute byte positions from the start of the file.

| Section | Offset Field | Alignment |
|---------|-------------|-----------|
| Hash index | `hash_index_offset` | 8-byte |
| Entry table | `entry_table_offset` | 8-byte |
| String table | `string_table_offset` | 4-byte |

The sections MUST appear in this order: Header, Hash Index, Entry Table, String Table. Sections are contiguous (no gaps between `offset` and `offset + size`).

### 3.4 File Size Constraints

| Constraint | Minimum | Maximum | Rationale |
|------------|---------|---------|-----------|
| Total file size | 64 bytes (header only, empty KB) | 10 MB | 10 MB is extreme upper bound |
| Particle entries | 0 | 65,535 | Limited by u32 count field |
| Hash index size | 1,024 bytes (bucket table only) | 1 MB | 256 buckets × 4 bytes + entries × 8 bytes |
| Entry table | 0 | 4,194,240 (65,535 × 64 B) | Full entry capacity |
| String table | 0 | 5 MB | Meanings, notes, examples |
| String count | 0 | 131,072 | 2× particle count (meanings + notes) |

---

## 4. Perfect Hash Index

### 4.1 Hash Function Selection

The hash index uses a **64-bit non-cryptographic hash** optimized for short Arabic strings (2–5 characters typical). The hash function MUST be:

| Property | Requirement |
|----------|-------------|
| **Algorithm** | CityHash64 or xxHash64 |
| **Seed** | `0x4B42303038` (= `"KB008"` as u64) — deterministic |
| **Output** | u64 (64-bit, little-endian) |
| **Collision rate** | Zero collisions within the same bucket after perfect hash construction |
| **Avalanche** | Flipping any input bit flips ~50% of output bits |

**Why not SHA-256?** Performance. The fast path runs on every token (potentially thousands per second). SHA-256 is 100–1000× slower than CityHash/xxHash for short strings.

### 4.2 Bucket Table Design

The bucket table is a flat array of 256 entries, indexed by `hash % 256`:

```c
struct BucketEntry {
    uint32_t first_entry_index;    // Index into hash_entry array
    uint32_t entry_count;          // Number of entries in this bucket
};
// 4 + 4 = 8 bytes per bucket
// 256 buckets × 8 bytes = 2048 bytes total
```

**Bucket table layout:**

```
┌──────────┬──────────┬──────────┬─────┬──────────┐
│ Bucket 0 │ Bucket 1 │ Bucket 2 │ ... │ Bucket   │
│          │          │          │     │ 255      │
│ 8 bytes  │ 8 bytes  │ 8 bytes  │     │ 8 bytes  │
└──────────┴──────────┴──────────┴─────┴──────────┘
├──────────────────── 2048 bytes ──────────────────┤
```

Each `BucketEntry` points to a contiguous range of `HashEntry` records in the hash entries array. The entries within a bucket are sorted by the full hash value for binary search within the bucket.

### 4.3 Hash Entry Layout

```c
struct HashEntry {
    uint32_t particle_id;          // Index into entry table
    uint32_t hash_value;           // Upper 32 bits of full 64-bit hash
};
// 4 + 4 = 8 bytes
```

**Total hash entries:** `sum(bucket.entry_count for bucket in buckets)` = `particle_count` (one entry per particle, plus overflow entries for homographs).

### 4.4 Lookup Algorithm

```c
/// Look up a particle by its normalized text.
/// Returns the particle ID (index into entry table) or -1 if not found.
int32_t particle_lookup(
    const ParticleKB* kb,
    const uint8_t* text,        // Normalized UTF-8 text
    uint32_t text_len           // Byte length
) {
    // 1. Compute hash
    uint64_t hash = cityhash64(text, text_len, HASH_SEED);
    uint32_t bucket_index = hash % 256;
    uint32_t hash_upper = (hash >> 32) & 0xFFFFFFFF;

    // 2. Look up bucket
    BucketEntry* bucket = &kb->bucket_table[bucket_index];
    if (bucket->entry_count == 0) return -1;

    // 3. Binary search within bucket (sorted by hash_upper)
    HashEntry* entries = &kb->hash_entries[bucket->first_entry_index];
    int32_t lo = 0;
    int32_t hi = (int32_t)bucket->entry_count - 1;

    while (lo <= hi) {
        int32_t mid = lo + (hi - lo) / 2;
        if (entries[mid].hash_value < hash_upper) {
            lo = mid + 1;
        } else if (entries[mid].hash_value > hash_upper) {
            hi = mid - 1;
        } else {
            // 4. Hash match — verify full text
            int32_t pid = (int32_t)entries[mid].particle_id;
            ParticleEntry* entry = &kb->entry_table[pid];
            const char* entry_text = get_string(kb, entry->text_offset);

            if (string_equal(text, text_len, entry_text)) {
                return pid;  // Full match
            }

            // 5. Check adjacent entries with same hash (collision)
            int32_t check = mid - 1;
            while (check >= 0 && entries[check].hash_value == hash_upper) {
                pid = (int32_t)entries[check].particle_id;
                entry = &kb->entry_table[pid];
                entry_text = get_string(kb, entry->text_offset);
                if (string_equal(text, text_len, entry_text)) {
                    return pid;
                }
                check--;
            }
            check = mid + 1;
            while (check < (int32_t)bucket->entry_count
                   && entries[check].hash_value == hash_upper) {
                pid = (int32_t)entries[check].particle_id;
                entry = &kb->entry_table[pid];
                entry_text = get_string(kb, entry->text_offset);
                if (string_equal(text, text_len, entry_text)) {
                    return pid;
                }
                check++;
            }

            return -1;  // Hash collision but no text match
        }
    }

    return -1;  // Not found
}
```

**Complexity:** O(log N) per bucket where N = average entries per bucket (~1 for 200 particles / 256 buckets). Worst case: O(log k) where k = max entries in any bucket.

### 4.5 Perfect Hash Construction (Compiler)

The KB compiler constructs the hash index at build time:

```pseudo
Algorithm: build_perfect_hash

Input:  List of (particle_text, particle_entry) pairs
Output: Bucket table + hash entries

1. Group entries by (cityhash64(text) % 256):
   For each text in all_entries:
       hash = cityhash64(text, HASH_SEED)
       bucket_index = hash % 256
       Append (hash, entry) to bucket[bucket_index]

2. Sort each bucket by hash value (ascending):
   For each bucket:
       Sort by hash >> 32 (upper 32 bits)
       For entries with identical upper hash, sort by particle_id

3. Assign entry indices:
   current_index = 0
   For bucket_index = 0..255:
       bucket.first_entry_index = current_index
       bucket.entry_count = len(bucket.entries)
       For each (hash, entry) in bucket.entries:
           hash_entries[current_index] = { entry.id, hash >> 32 }
           current_index++

4. Verify no collisions within same bucket:
   For each bucket:
       For i = 0..len-2:
           Assert(hash_entries[i].hash_upper != hash_entries[i+1].hash_upper
                  OR entry texts differ)

5. Compute checksum:
   header.checksum_sha256 = SHA-256(hash_index || entry_table || string_table)
```

### 4.6 Hash Index Size Budget

| Component | Calculation | Size (200 entries) |
|-----------|------------|-------------------|
| Bucket table | 256 × 8 bytes | 2,048 B |
| Hash entries | 200 × 8 bytes | 1,600 B |
| **Total hash index** | | **~3.6 KB** |

This is negligible compared to the string table (~1–4 MB) and demonstrates the efficiency of the design for a small closed class.

---

## 5. Entry Table

### 5.1 Compact Entry Format (64 bytes)

Each particle entry is packed into exactly 64 bytes for cache-friendly sequential access:

```c
struct ParticleEntry {
    // ── Classification (4 bytes) ──
    uint8_t  particle_type;          // enum ParticleType (0–23)
    uint8_t  sub_type;               // Sub-category (e.g., "primary", "redundant")
    uint8_t  usage_rank;             // 1 = most common, 255 = least
    uint8_t  flags;                  // Bitmask: clitic, homograph, etc.

    // ── Governance (4 bytes) ──
    uint8_t  governs_case;           // 0=none, 1=nom, 2=acc, 3=gen, 4=jus, 5=sub
    uint8_t  governs_mood;           // 0=none, 1=ind, 2=sub, 3=jus
    uint8_t  government_type;        // 0=indep, 1=requires_sister, 2=requires_comp
    uint8_t  attaches_to;            // 0=standalone, 1=next_word, 2=prev_word, 3=both

    // ── String Table References (16 bytes) ──
    uint32_t text_offset;            // Offset into string table for particle text
    uint32_t meaning_offset;         // Offset for English meaning
    uint32_t grammar_offset;         // Offset for grammar description
    uint32_t usage_offset;           // Offset for usage notes (0 if none)

    // ── Homograph Data (8 bytes) ──
    uint32_t homograph_group_id;     // 0xFFFFFFFF if not a homograph
    uint32_t disambiguation_flags;   // Bitmask of contextual hints

    // ── Examples (4 bytes) ──
    uint32_t example_offset;         // Offset into string table for examples block

    // ── Entry Metadata (4 bytes) ──
    uint16_t script_form_count;      // Number of orthographic variants
    uint16_t category_flags;         // Bitmask of category memberships

    // ── Padding (24 bytes) ──
    uint8_t  filler[24];            // Reserved for future expansion

    // Total: 4 + 4 + 16 + 8 + 4 + 4 + 24 = 64 bytes
};
```

### 5.2 ParticleType Enum

```c
enum ParticleType : uint8_t {
    // Prepositions (جارة)
    HARF_JARR           = 0,   // حرف جر — preposition
    HARF_JARR_REDUNDANT = 1,   // حرف جر زائد — redundant preposition
    HARF_JARR_COMPOUND  = 2,   // حرف جر مركب — compound preposition

    // Conjunctions (عاطفة)
    HARF_ATF            = 3,   // حرف عطف — coordinating conjunction
    HARF_ATF_SUB        = 4,   // حرف عطف تابع — subordinating conjunction

    // Mood-Governance (ناصبة/جازمة)
    HARF_NASB           = 5,   // حرف نصب — subjunctive particle
    HARF_JAZM           = 6,   // حرف جزم — jussive particle

    // Conditional (شرطية)
    HARF_SHART          = 7,   // حرف شرط — conditional particle

    // Interrogative (استفهامية)
    HARF_ISTIFHAM       = 8,   // حرف استفهام — interrogative particle

    // Negative (نافية)
    HARF_NAFY           = 9,   // حرف نفي — negative particle

    // Vocative (ندائية)
    HARF_NIDA           = 10,  // حرف نداء — vocative particle

    // Inna & Sisters (ناسخة)
    HARF_NASIKH         = 11,  // حرف ناسخ — inna & sisters

    // Semi-verbs (فعل ناقص)
    SEMI_VERB           = 12,  // فعل ناقص — kana & sisters

    // Answer/Response (جوابية)
    HARF_JAWAB          = 13,  // حرف جواب — answer particle

    // Exception (استثنائية)
    HARF_ISTITHNA       = 14,  // حرف استثناء — exception particle

    // Masdar-Forming (مصدرية)
    HARF_MASDARI        = 15,  // حرف مصدر — masdar-forming particle

    // Comparison (تشبيهية)
    HARF_TASHBIH        = 16,  // حرف تشبيه — comparison particle

    // Exhortation (تحضيضية)
    HARF_TAHDID         = 17,  // حرف تحضيض — exhortation particle

    // Future (استقبالية)
    HARF_ISTIQBAL       = 18,  // حرف استقبال — future particle

    // Prohibition (ناهية)
    HARF_NAHIYAH        = 19,  // حرف ناهية — prohibition particle

    // Emphasis (توكيدية)
    HARF_TAWKID         = 20,  // حرف توكيد — emphatic particle

    // Attentional (تنبيهية)
    HARF_TANBIH         = 21,  // حرف تنبيه — attention particle

    // Other miscellaneous
    OTHER               = 22,  // Other functional particle

    // Reserved
    RESERVED            = 23   // Reserved for future expansion
};
```

### 5.3 Flags Byte

```c
enum ParticleFlags : uint8_t {
    FLAG_CLITIC_ATTACHED   = 0x01,  // Particle attaches as clitic (prefix/suffix)
    FLAG_HOMOGRAPH         = 0x02,  // Part of a homograph group
    FLAG_DISPUTED          = 0x04,  // Grammatical classification disputed
    FLAG_ARCHAIC           = 0x08,  // Archaic or rare usage
    FLAG_QURANIC           = 0x10,  // Occurs in the Quran
    FLAG_HAS_SCRIPT_VARIANTS = 0x20, // Has multiple orthographic forms
    FLAG_NASIKH_OPERATOR   = 0x40,  // Inna/Kana sister (governs subject case)
    FLAG_DEFECTIVE         = 0x80,  // Defective verb (kana-type semi-particle)
};
```

### 5.4 Category Flags

```c
enum CategoryFlag : uint16_t {
    CAT_PREPOSITION         = 0x0001,
    CAT_CONJUNCTION         = 0x0002,
    CAT_MOOD_GOVERNING      = 0x0004,
    CAT_CASE_GOVERNING      = 0x0008,
    CAT_CONDITIONAL         = 0x0010,
    CAT_INTERROGATIVE       = 0x0020,
    CAT_NEGATIVE            = 0x0040,
    CAT_VOCATIVE            = 0x0080,
    CAT_EMPHATIC            = 0x0100,
    CAT_MASDAR_FORMING      = 0x0200,
    CAT_EXCEPTION           = 0x0400,
    CAT_TIME_RELATED        = 0x0800,
    CAT_ANSWER              = 0x1000,
    CAT_EXHORTATION         = 0x2000,
    CAT_HOMONYM_NOUN        = 0x4000,  // Also functions as a noun
    CAT_HOMONYM_VERB        = 0x8000,  // Also functions as a verb
};
```

### 5.5 Governance Codes

```c
enum GovernsCase : uint8_t {
    CASE_NONE          = 0,   // No case governance
    CASE_NOMINATIVE    = 1,   // الرفع
    CASE_ACCUSATIVE    = 2,   // النصب
    CASE_GENITIVE      = 3,   // الجر
    CASE_JUSSIVE       = 4,   // الجزم (for mood, also used here for simplicity)
    CASE_SUBJUNCTIVE   = 5,   // النصب (for mood)
};

enum GovernsMood : uint8_t {
    MOOD_NONE         = 0,   // No mood governance
    MOOD_INDICATIVE   = 1,   // الرفع (indicative)
    MOOD_SUBJUNCTIVE  = 2,   // النصب (subjunctive)
    MOOD_JUSSIVE      = 3,   // الجزم (jussive)
};

enum GovernmentType : uint8_t {
    GOV_INDEPENDENT          = 0,   // Governs independently
    GOV_REQUIRES_SISTER      = 1,   // Requires sister particle (e.g., inna + la-)
    GOV_REQUIRES_COMPLEMENT  = 2,   // Requires complement clause (e.g., conditional)
};

enum AttachesTo : uint8_t {
    ATTACH_STANDALONE    = 0,   // Written as separate word
    ATTACH_NEXT_WORD     = 1,   // Attaches as prefix to next word
    ATTACH_PREV_WORD     = 2,   // Attaches as suffix to previous word
    ATTACH_BOTH          = 3,   // Can attach to either side
};
```

### 5.6 Disambiguation Flags

```c
enum DisambiguationFlag : uint32_t {
    // Context type hints
    DISAMBIG_FOLLOWED_BY_VERB          = 0x00000001,
    DISAMBIG_FOLLOWED_BY_NOUN          = 0x00000002,
    DISAMBIG_FOLLOWED_BY_PARTICLE      = 0x00000004,
    DISAMBIG_FOLLOWED_BY_JUSSIVE       = 0x00000008,
    DISAMBIG_FOLLOWED_BY_SUBJUNCTIVE   = 0x00000010,
    DISAMBIG_FOLLOWED_BY_GENITIVE      = 0x00000020,
    DISAMBIG_FOLLOWED_BY_ACCUSATIVE    = 0x00000040,
    DISAMBIG_FOLLOWED_BY_NOMINATIVE    = 0x00000080,

    // Structural hints
    DISAMBIG_SENTENCE_INITIAL          = 0x00000100,
    DISAMBIG_PRECEDED_BY_NEGATIVE      = 0x00000200,
    DISAMBIG_PRECEDED_BY_INTERROGATIVE = 0x00000400,
    DISAMBIG_IN_CONDITIONAL_CLAUSE     = 0x00000800,
    DISAMBIG_IN_EXCEPTION_CONSTRUCTION = 0x00001000,
    DISAMBIG_IN_VOCATIVE_CONSTRUCTION  = 0x00002000,

    // Semantic hints
    DISAMBIG_OATH_CONTEXT              = 0x00010000,
    DISAMBIG_EMPHATIC_CONTEXT          = 0x00020000,
    DISAMBIG_HYPOTHETICAL              = 0x00040000,
    DISAMBIG_TIME_REFERENCE            = 0x00080000,
    DISAMBIG_PURPOSE_CLAUSE            = 0x00100000,
    DISAMBIG_RESULT_CLAUSE             = 0x00200000,

    // Frequency hints
    DISAMBIG_MOST_COMMON               = 0x01000000,
    DISAMBIG_SECOND_COMMON             = 0x02000000,
    DISAMBIG_RARE                      = 0x04000000,
};

// Homograph group scoring weights
const int32_t SCORE_EXACT_SYNTACTIC_MATCH  = 3;
const int32_t SCORE_COMPATIBLE_DOMAIN      = 2;
const int32_t SCORE_FREQUENCY_BOOST        = 1;
```

### 5.7 Example Block Format

The example block is a length-prefixed sequence of example entries:

```c
struct ExampleBlock {
    uint32_t count;                      // Number of examples
    Example  examples[count];            // Array of examples
};

struct Example {
    uint8_t  source_type;                // 0=none, 1=quran, 2=classical, 3=modern, 4=synthetic
    uint32_t phrase_offset;              // String table offset for Arabic phrase
    uint32_t translation_offset;         // String table offset for English translation
    uint32_t source_offset;              // String table offset for source citation (0 if none)
};
```

---

## 6. String Table

### 6.1 Layout

The string table stores all human-readable text referenced by the entry table. It uses the same format as RFC-0002 §7:

```
┌────────────────┬──────────────────────┬──────────────────────┐
│ Entry Count    │ Entry 0              │ Entry 1              │
│ (varint)       │ Length (varint) +    │ Length (varint) +    │
│                │ UTF-8 bytes          │ UTF-8 bytes          │
└────────────────┴──────────────────────┴──────────────────────┘
```

### 6.2 String Content by Entry

| String Type | Count | Size per String | Total Size | Level 1 (Compact) | Level 2 (Full) |
|-------------|-------|----------------|------------|-------------------|----------------|
| Particle text | N_entries | ~5 B avg | ~1 KB | ✓ | ✓ |
| Meaning (English) | N_entries | ~50 B avg | ~10 KB | ✓ | ✓ |
| Grammar description | N_entries | ~100 B avg | ~20 KB | ✓ | ✓ |
| Usage notes | N_entries | ~80 B avg | ~16 KB | Optional | ✓ |
| Examples (phrases) | N_examples × 3 | ~150 B avg | ~90 KB | Optional | ✓ |
| Disambiguation hints | N_homographs × 2 | ~60 B avg | ~24 KB | Optional | ✓ |
| Script form variants | N_variants | ~40 B avg | ~8 KB | ✓ | ✓ |
| **Total (200 entries)** | | | **~170 KB (compact)** | **~500 KB (full)** | |

### 6.3 String Access Function

```c
/// Get a string from the string table by byte offset.
/// Returns a pointer to the UTF-8 data and sets the length.
const uint8_t* get_string(
    const ParticleKB* kb,
    uint32_t offset,
    uint32_t* out_len
) {
    // Strings are stored as: length (varint) + UTF-8 bytes
    const uint8_t* data = kb->string_table_data + offset;
    uint32_t pos = 0;

    // Decode varint length
    uint32_t len = 0;
    int shift = 0;
    while (true) {
        uint8_t byte = data[pos++];
        len |= (byte & 0x7F) << shift;
        if ((byte & 0x80) == 0) break;
        shift += 7;
    }

    *out_len = len;
    return data + pos;
}
```

---

## 7. Particle Lookup API

### 7.1 Core API

```c
/// Opaque handle to a compiled particle KB loaded in memory.
typedef struct ParticleKB ParticleKB;

/// Result of a particle lookup.
typedef struct {
    int32_t  particle_id;          // Index into entry table (-1 if not found)
    uint32_t match_type;           // 0=exact, 1=no_diacritics, 2=stem_only
    uint32_t confidence;           // 0–100 (for homograph disambiguation)
} ParticleMatch;

/// Load a compiled particle KB from memory.
/// @param data      Pointer to the .agos-kb file contents (mmap'd or loaded)
/// @param data_size Size of the data in bytes
/// @return          Initialized ParticleKB handle, or NULL on error
ParticleKB* particle_kb_load(const uint8_t* data, size_t data_size);

/// Free a loaded particle KB.
void particle_kb_free(ParticleKB* kb);

/// Look up a single particle by exact normalized text.
/// @param kb        Loaded particle KB
/// @param text      Normalized UTF-8 text
/// @param text_len  Byte length of text
/// @return          ParticleMatch with particle_id or -1 if not found
ParticleMatch particle_lookup(
    const ParticleKB* kb,
    const uint8_t* text,
    uint32_t text_len
);

/// Look up all homograph variants of a particle text.
/// @param kb        Loaded particle KB
/// @param text      Normalized UTF-8 text
/// @param text_len  Byte length of text
/// @param results   Output array (pre-allocated, up to max_results)
/// @param max_results Max entries in results array
/// @return          Number of results found (0 if none)
uint32_t particle_lookup_all(
    const ParticleKB* kb,
    const uint8_t* text,
    uint32_t text_len,
    ParticleMatch* results,
    uint32_t max_results
);

/// Get the entry data for a particle ID.
/// @param kb          Loaded particle KB
/// @param particle_id Valid particle ID from lookup
/// @return            Pointer to the packed entry, or NULL if invalid
const ParticleEntry* particle_get_entry(
    const ParticleKB* kb,
    uint32_t particle_id
);
```

### 7.2 Normalization API

```c
/// Normalize a token for particle lookup.
/// The normalization pipeline is:
///   1. Remove tatweel/kashida characters (U+0640)
///   2. Apply NFKC Unicode normalization
///   3. Strip leading clitic prefixes: wa-, fa-, bi-, li-, ka-, sa-, a-
///   4. Optionally strip diacritics
///
/// @param input        Raw UTF-8 token text
/// @param input_len    Byte length of input
/// @param output       Output buffer (must be >= input_len + 8 bytes)
/// @param output_len   Output byte length
/// @param flags        Normalization flags (see NormalizeFlags)
/// @return             0 on success, -1 on error
int normalize_token(
    const uint8_t* input,
    uint32_t input_len,
    uint8_t* output,
    uint32_t* output_len,
    uint32_t flags
);

enum NormalizeFlags {
    NORM_DEFAULT          = 0x00,        // Full normalization (no diacritic stripping)
    NORM_STRIP_DIACRITICS = 0x01,        // Strip fatha, kasra, damma, sukun, shadda
    NORM_KEEP_CLITICS     = 0x02,        // Don't strip leading clitic prefixes
    NORM_CLITIC_STRICT    = 0x04,        // Only strip known clitics (wa, fa, bi, li, etc.)
};

/// Split a token into a leading clitic prefix and the remaining stem.
/// Returns the prefix length in bytes (0 if no clitic detected).
uint32_t split_clitic_prefix(
    const uint8_t* token,
    uint32_t token_len,
    uint8_t* prefix_out,     // Output: clitic prefix text (at least 4 bytes)
    uint32_t* prefix_len_out // Output: length of prefix
);
```

### 7.3 Clitic Splitting Algorithm

```c
/// Known Arabic clitic prefixes (in UTF-8 byte form).
static const uint8_t* CLITIC_PREFIXES[] = {
    (uint8_t*)"و",    // wa- (and)
    (uint8_t*)"ف",    // fa- (and so, then)
    (uint8_t*)"ب",    // bi- (by, with, in)
    (uint8_t*)"ل",    // li- (for, to)
    (uint8_t*)"ك",    // ka- (like, as)
    (uint8_t*)"س",    // sa- (future tense)
    (uint8_t*)"أ",    // a- (interrogative/vocative prefix)
    (uint8_t*)"ال",   // al- (definite article) — stripped before noun analysis
};

/// Check if a token starts with a known clitic prefix.
/// Returns the number of bytes to strip from the front.
uint32_t detect_clitic_prefix(const uint8_t* token, uint32_t token_len) {
    if (token_len < 2) return 0;  // Minimum clitic is 1 byte + at least 1 byte stem

    // Check single-byte prefixes (wa, fa, bi, li, ka, sa, a)
    uint8_t first = token[0];
    if (token_len >= 2) {
        switch (first) {
            case 0x88:  // ب (U+0628)
            case 0x84:  // ت (U+062A) — rare, oath
            case 0x86:  // ث (U+062B) — rare
            case 0x8B:  // س (U+0633) — future
            case 0x83:  // ج (U+062C) — rare
            case 0x8F:  // ص (U+0635) — rare
                return 1;
        }
    }

    // Check و (wa-)
    if (first == 0x88 && token_len >= 2) { // U+0648 = و
        // Only strip if the remaining stem is >= 1 character
        return 1;
    }

    // Check ف (fa-)
    if (first == 0x81 && token_len >= 2) { // U+0641 = ف
        return 1;
    }

    // Check ل (li-)
    if (first == 0x84 && token_len >= 2) { // U+0644 = ل
        // Don't strip if the token is "لا" (laa, negative)
        if (token_len >= 2 && token[1] == 0x27) { // U+0627 = ا
            return 0;  // "لا" is a complete particle, not ل + ا
        }
        return 1;
    }

    // Check two-byte prefix: ال (al-)
    if (first == 0x84 && token_len >= 3) { // ل (U+0644)
        if (token[1] == 0x27) { // ا (U+0627)
            // Check for lam-alef ligature
            return 2;
        }
    }

    return 0;
}
```

---

## 8. Governance Resolution API

### 8.1 Governance Effect Structure

```c
/// The grammatical effect of a particle on surrounding words.
typedef struct {
    // ── Case Effect ──
    uint8_t  case_effect;            // CASE_NONE / NOMINATIVE / ACCUSATIVE / GENITIVE
    uint32_t case_target_token;     // Index of the token affected (or -1)
    uint32_t case_confidence;        // 0–100

    // ── Mood Effect ──
    uint8_t  mood_effect;            // MOOD_NONE / INDICATIVE / SUBJUNCTIVE / JUSSIVE
    uint32_t mood_target_token;     // Index of the verb affected (or -1)
    uint32_t mood_confidence;        // 0–100

    // ── Structural Effect ──
    uint8_t  structural_effect;      // 0=none, 1=conditional, 2=vocative,
                                     // 3=exception, 4=emphatic, 5=inna_clause
    uint32_t complement_token_start; // Start of complement clause (if applicable)
    uint32_t complement_token_end;   // End of complement clause (if applicable)

    // ── Modifications ──
    bool     requires_subject_case;   // e.g., inna: subject → accusative
    uint8_t  subject_case;           // Case for subject noun
    bool     requires_predicate_case; // e.g., kana: predicate → accusative
    uint8_t  predicate_case;         // Case for predicate

    // ── Morphological Adjustments ──
    uint32_t clitic_strip_bytes;     // Bytes to strip from following word (0 if standalone)
    uint32_t phonetic_change_type;   // 0=none, 1=min→mina (before al), etc.
} GovernanceEffect;
```

### 8.2 Resolution Function

```c
/// Resolve the governance effect of a particle on the following context.
///
/// @param kb              Loaded particle KB
/// @param particle_id     The matched particle entry ID
/// @param following_words Array of following word tokens (for context)
/// @param following_count Number of following words
/// @param sentence_type   Type of the current sentence (0=unknown, 1=verbal, 2=nominal)
/// @param effect          Output: governance effect
/// @return                0 on success, -1 on error
int resolve_governance(
    const ParticleKB* kb,
    uint32_t particle_id,
    const TokenInfo* following_words,
    uint32_t following_count,
    uint8_t sentence_type,
    GovernanceEffect* effect
) {
    const ParticleEntry* entry = particle_get_entry(kb, particle_id);
    if (!entry) return -1;

    // Clear effect
    memset(effect, 0, sizeof(GovernanceEffect));

    // 1. Case governance
    if (entry->governs_case != CASE_NONE) {
        effect->case_effect = entry->governs_case;
        effect->case_target_token = 0;  // First following noun
        effect->case_confidence = 95;

        // Prepositions: genitive on the noun
        if (entry->particle_type == HARF_JARR) {
            effect->structural_effect = 0;  // Prepositional phrase
            effect->phonetic_change_type = detect_phonetic_change(entry, following_words);
        }

        // Inna & sisters: accusative on subject
        if (entry->particle_type == HARF_NASIKH) {
            effect->requires_subject_case = true;
            effect->subject_case = CASE_ACCUSATIVE;
            effect->requires_predicate_case = true;
            effect->predicate_case = CASE_NOMINATIVE;
            effect->structural_effect = 4;  // Inna clause
        }
    }

    // 2. Mood governance
    if (entry->governs_mood != MOOD_NONE) {
        effect->mood_effect = entry->governs_mood;
        effect->mood_target_token = find_following_verb(following_words, following_count);
        effect->mood_confidence = 90;

        // جازم particles: jussive on verb
        if (entry->particle_type == HARF_JAZM) {
            effect->mood_effect = MOOD_JUSSIVE;
        }

        // ناصب particles: subjunctive on verb
        if (entry->particle_type == HARF_NASB) {
            effect->mood_effect = MOOD_SUBJUNCTIVE;
        }
    }

    // 3. Structural effects
    if (entry->particle_type == HARF_SHART) {
        effect->structural_effect = 1;  // Conditional
        // Find the condition and result clause boundaries
        effect->complement_token_start = 0;
        effect->complement_token_end = find_conditional_boundary(
            following_words, following_count);
    }

    if (entry->particle_type == HARF_NIDA) {
        effect->structural_effect = 2;  // Vocative
        effect->case_effect = CASE_NOMINATIVE;  // Default for vocative
        effect->case_target_token = 0;
    }

    // 4. Clitic handling
    effect->clitic_strip_bytes = 0;
    if (entry->attaches_to == ATTACH_NEXT_WORD) {
        effect->clitic_strip_bytes = get_particle_text_length(kb, entry);
    }

    return 0;
}
```

### 8.3 Phonetic Change Detection

```c
/// Detect phonetic changes required when a particle precedes certain words.
uint32_t detect_phonetic_change(
    const ParticleEntry* entry,
    const TokenInfo* following_words
) {
    if (following_words == NULL || following_words->text_len == 0) return 0;

    // مِنْ → مِنَ before definite article (ال)
    if (entry->particle_type == HARF_JARR) {
        const uint8_t* text = following_words->text;
        uint32_t len = following_words->text_len;

        // Check if next word starts with lam-alef (ال)
        if (len >= 2 && text[0] == 0x84 && text[1] == 0x27) { // ل + ا
            switch (entry->sub_type) {
                case 0: // مِنْ
                    return 1;  // MIN_TO_MINA
                case 1: // عَنْ
                    return 2;  // AN_TO_ANI
                case 2: // بِ
                    return 3;  // BI_no_change
                // ... other prepositions
            }
        }

        // مِنْ + identical consonant at start of next word → idgham
        // (detailed rules from tajweed/tajwid)
    }

    return 0;
}
```

---

## 9. MOD-04 Fast-Path Integration

### 9.1 Integration Flow

```c
/// MOD-04 Step 3.1: Fast-path particle check.
/// Returns true if the token was identified as a particle (analysis complete).
bool mod04_fast_path_particle_check(
    MorphContext* ctx,           // MOD-04 morphological context
    uint32_t token_index,        // Index of current token
    const uint8_t* token_text,   // Raw token text
    uint32_t token_len           // Token length
) {
    // 1. Normalize the token
    uint8_t normalized[256];
    uint32_t norm_len;
    if (normalize_token(token_text, token_len,
                        normalized, &norm_len,
                        NORM_DEFAULT) != 0) {
        return false;  // Fall through to main path
    }

    // 2. Look up in particle KB
    ParticleMatch match = particle_lookup(
        ctx->particle_kb,
        normalized,
        norm_len
    );

    if (match.particle_id < 0) {
        // 3. If not found, try stripping clitic prefixes
        uint8_t prefix[8];
        uint32_t prefix_len;
        uint32_t strip = split_clitic_prefix(
            normalized, norm_len,
            prefix, &prefix_len
        );

        if (strip > 0 && strip < norm_len) {
            // Retry with remainder
            match = particle_lookup(
                ctx->particle_kb,
                normalized + strip,
                norm_len - strip
            );
        }

        // 4. If still not found, try without diacritics
        if (match.particle_id < 0) {
            uint8_t no_diac[256];
            uint32_t no_diac_len;
            normalize_token(token_text, token_len,
                          no_diac, &no_diac_len,
                          NORM_STRIP_DIACRITICS);
            match = particle_lookup(
                ctx->particle_kb,
                no_diac, no_diac_len
            );

            if (match.particle_id < 0 && strip > 0 && strip < norm_len) {
                // Try clitic-stripped + no diacritics
                normalize_token(normalized + strip, norm_len - strip,
                              no_diac, &no_diac_len,
                              NORM_STRIP_DIACRITICS);
                match = particle_lookup(
                    ctx->particle_kb,
                    no_diac, no_diac_len
                );
            }
        }
    }

    // 5. If particle found, set up feature bitfield and exit fast
    if (match.particle_id >= 0) {
        const ParticleEntry* entry = particle_get_entry(
            ctx->particle_kb,
            (uint32_t)match.particle_id
        );

        // Set POS = particle (2) in the feature bitfield
        set_feature_bitfield(ctx, token_index, FEATURE_POS, POS_PARTICLE);

        // Set particle type feature
        set_feature_bitfield(ctx, token_index,
                             FEATURE_PARTICLE_TYPE,
                             entry->particle_type);

        // Record governance for syntactic analysis
        GovernanceEffect gov;
        resolve_governance(
            ctx->particle_kb,
            (uint32_t)match.particle_id,
            ctx->tokens + token_index + 1,     // Following words
            ctx->token_count - token_index - 1,
            ctx->sentence_type,
            &gov
        );
        ctx->governance_effects[token_index] = gov;

        // Skip root extraction — particles have no root
        ctx->token_analysis[token_index].status = ANALYSIS_PARTICLE;
        ctx->token_analysis[token_index].analysis_time_ns = measure_elapsed_ns();

        return true;
    }

    return false;  // Not a particle — fall through to main path
}
```

### 9.2 Performance Guarantees

| Path | Target | Scenario |
|------|--------|----------|
| **Hit (exact match)** | < 500 ns | Normalized text matches particle entry |
| **Hit (clitic-stripped)** | < 1 μs | Need to strip clitic prefix first |
| **Hit (no diacritics)** | < 1.5 μs | Need to strip diacritics and retry |
| **Full miss** | < 2 μs | Not a particle, exhausted all options |
| **Homograph lookup** | < 2 μs | Multiple entries match, need ranking |

### 9.3 Token Status After Fast Path

```c
enum TokenAnalysisStatus {
    ANALYSIS_PARTICLE      = 0,    // Identified as particle, analysis complete
    ANALYSIS_PRONOUN       = 1,    // Identified as pronoun (from KB-0006)
    ANALYSIS_ROOT_STARTED  = 2,    // Not a particle/pronoun, root extraction begins
    ANALYSIS_AMBIGUOUS     = 3,    // Multiple possible analyses
};
```

### 9.4 Feature Bitfield Values for Particles

When a particle is identified, the following features are set in the 64-bit feature bitfield:

| Feature ID | Field | Value | Meaning |
|-----------|-------|-------|---------|
| 0 | POS | `2` | POS_PARTICLE (حرف) |
| 20 | particle_type | `0–22` | From ParticleType enum |
| 21 | particle_sub_type | `0–255` | Sub-category |
| 22 | particle_usage_rank | `1–255` | Frequency rank |
| 23 | particle_governs_case | `0–5` | Case governance code |
| 24 | particle_governs_mood | `0–3` | Mood governance code |

These feature IDs (20–24) are in the **plugin/reserved zone** (bits 50–63 of the standard 64-bit feature bitfield). See SPEC-0102 §14 for the plugin extension mechanism.

**Bit assignment within reserved zone (bits 50–63):**

| Feature ID | Name | Bits | Width | Description |
|-----------|------|------|-------|-------------|
| 20 | particle_type | 50–55 | 6 | ParticleType enum (0–23, fits in 5 bits; 6 for future) |
| 21 | particle_sub_type | 56–59 | 4 | Sub-category within particle type |
| 22 | particle_usage_rank | 60–63 | 4 | Frequency rank (1–15; higher values reserved)

---

## 10. MOD-05 Syntactic Integration

### 10.1 Governance Application

MOD-05 (SyntacticParser) uses the governance effects recorded by MOD-04 to assign syntactic roles and ensure agreement:

```c
/// Apply particle governance effects during syntactic parsing.
/// Called after clause boundaries are determined.
int syntactics_apply_particle_governance(
    SyntaxContext* ctx,
    uint32_t particle_token_index
) {
    GovernanceEffect* gov = &ctx->governance_effects[particle_token_index];
    if (!gov->case_effect && !gov->mood_effect && !gov->structural_effect) {
        return 0;  // No governance to apply
    }

    // 1. Apply case effect to target noun
    if (gov->case_effect != CASE_NONE) {
        uint32_t target = particle_token_index + 1 + gov->case_target_token;

        // Scan forward to find the first noun
        while (target < ctx->token_count) {
            if (get_feature(ctx, target, FEATURE_POS) == POS_NOUN) {
                set_feature(ctx, target, FEATURE_CASE, gov->case_effect);

                // If preposition governs genitive, assign role
                if (gov->case_effect == CASE_GENITIVE) {
                    set_role(ctx, target, ROLE_MAJRUR);  // مجرور
                    set_role(ctx, particle_token_index, ROLE_HARF_JARR);  // حرف جر
                }
                break;
            }
            target++;
        }
    }

    // 2. Apply mood effect to target verb
    if (gov->mood_effect != MOOD_NONE) {
        uint32_t target = gov->mood_target_token;
        if (target < ctx->token_count) {
            if (get_feature(ctx, target, FEATURE_POS) == POS_VERB) {
                uint8_t current_mood = get_feature(ctx, target, FEATURE_MOOD);

                // Check compatibility: verb must be in present tense for mood government
                uint8_t tense = get_feature(ctx, target, FEATURE_TENSE);
                if (tense == TENSE_PRESENT) {
                    set_feature(ctx, target, FEATURE_MOOD, gov->mood_effect);

                    // Record the governing particle for evidence
                    add_governance_evidence(ctx,
                        particle_token_index, target,
                        gov->mood_effect, gov->case_effect);
                } else {
                    // Flag: mood government skipped (verb not in present tense)
                    add_grammatical_flag(ctx, FLAG_MOOD_GOVERNMENT_SKIPPED,
                        particle_token_index, target);
                }
            }
        }
    }

    // 3. Apply structural effects (conditional, vocative, exception)
    if (gov->structural_effect == 1) {  // Conditional
        // Build conditional clause structure
        // [shart_particle] + [condition_clause] + [result_clause]
        ctx->clause_stack.push(CLAUSE_CONDITIONAL);
        ctx->conditional_particle = particle_token_index;
    }

    if (gov->structural_effect == 2) {  // Vocative
        set_role(ctx, particle_token_index, ROLE_HARF_NIDA);  // حرف نداء
        uint32_t called = particle_token_index + 1;
        if (called < ctx->token_count) {
            set_role(ctx, called, ROLE_MUNADA);  // منادى
        }
    }

    if (gov->requires_subject_case) {
        // e.g., inna: subject → accusative
        uint32_t subject = particle_token_index + 1; // First noun after inna
        if (subject < ctx->token_count) {
            set_feature(ctx, subject, FEATURE_CASE, gov->subject_case);
            set_role(ctx, subject, ROLE_ISM_INNA);  // اسم إن
        }
    }

    if (gov->requires_predicate_case) {
        // e.g., inna: predicate → nominative
        uint32_t pred = particle_token_index + 2; // Second element after inna
        if (pred < ctx->token_count) {
            set_feature(ctx, pred, FEATURE_CASE, gov->predicate_case);
            set_role(ctx, pred, ROLE_KHABAR_INNA);  // خبر إن
        }
    }

    return 0;
}
```

### 10.2 Mood Government Compatibility Matrix

| Particle Type | Governs Mood | Compatible Verb Tenses | Mood After |
|--------------|-------------|----------------------|------------|
| HARF_NASB (أَنْ, لَنْ, etc.) | Subjunctive | Present only | Subjunctive (e.g., يَكْتُبَ) |
| HARF_JAZM (لَمْ, لَمَّا, etc.) | Jussive | Present only | Jussive (e.g., يَكْتُبْ) |
| HARF_SHART (إِنْ, etc.) | Jussive | Present only | Jussive on both verbs |
| HARF_SHART (لَوْ, etc.) | None | Past only | No mood change |
| HARF_NAFY (لَا, مَا) | None | Any | No mood change |
| HARF_NAHIYAH (لَا النَّاهِيَة) | Jussive | Present only | Jussive |
| HARF_ISTIQBAL (سَ, سَوْفَ) | None | Present only | Indicative (no change) |

---

## 11. Homograph Disambiguation Engine

### 11.1 Contextual Scoring System

When a particle text maps to multiple entries (homographs), the disambiguation engine scores each interpretation based on syntactic context:

```c
/// Score a single particle interpretation given syntactic context.
/// Higher score = more likely interpretation.
int32_t score_particle_interpretation(
    const ParticleEntry* entry,
    const SyntaxContext* ctx,
    uint32_t token_index
) {
    int32_t score = 0;

    // ── Base score from usage rank ──
    score += (256 - entry->usage_rank);  // rank 1 → 255 points, rank 255 → 1 point

    // ── Syntactic context matching ──
    TokenInfo* next_token = (token_index + 1 < ctx->token_count)
                          ? &ctx->tokens[token_index + 1]
                          : NULL;

    if (next_token) {
        uint8_t next_pos = next_token->features.pos;

        // Followed by verb
        if (next_pos == POS_VERB) {
            if (entry->disambiguation_flags & DISAMBIG_FOLLOWED_BY_VERB) {
                score += SCORE_EXACT_SYNTACTIC_MATCH * 10;
            }

            // Check verb mood
            uint8_t next_mood = next_token->features.mood;
            if (next_mood == MOOD_JUSSIVE
                && (entry->disambiguation_flags & DISAMBIG_FOLLOWED_BY_JUSSIVE)) {
                score += SCORE_EXACT_SYNTACTIC_MATCH;
            }
            if (next_mood == MOOD_SUBJUNCTIVE
                && (entry->disambiguation_flags & DISAMBIG_FOLLOWED_BY_SUBJUNCTIVE)) {
                score += SCORE_EXACT_SYNTACTIC_MATCH;
            }
        }

        // Followed by noun
        if (next_pos == POS_NOUN) {
            if (entry->disambiguation_flags & DISAMBIG_FOLLOWED_BY_NOUN) {
                score += SCORE_EXACT_SYNTACTIC_MATCH * 5;
            }

            // Check noun case
            uint8_t next_case = next_token->features.case_marker;
            if (next_case == CASE_GENITIVE
                && (entry->disambiguation_flags & DISAMBIG_FOLLOWED_BY_GENITIVE)) {
                score += SCORE_EXACT_SYNTACTIC_MATCH;
            }
            if (next_case == CASE_ACCUSATIVE
                && (entry->disambiguation_flags & DISAMBIG_FOLLOWED_BY_ACCUSATIVE)) {
                score += SCORE_EXACT_SYNTACTIC_MATCH;
            }
        }
    }

    // ── Sentence position ──
    if (token_index == 0) {
        if (entry->disambiguation_flags & DISAMBIG_SENTENCE_INITIAL) {
            score += SCORE_EXACT_SYNTACTIC_MATCH;
        }
    }

    // ── Conditional clause context ──
    if (ctx->clause_type == CLAUSE_CONDITIONAL) {
        if (entry->disambiguation_flags & DISAMBIG_IN_CONDITIONAL_CLAUSE) {
            score += SCORE_EXACT_SYNTACTIC_MATCH;
        }
    }

    // ── Negation context ──
    if (token_index > 0) {
        uint8_t prev_pos = ctx->tokens[token_index - 1].features.pos;
        if (prev_pos == POS_PARTICLE) {
            uint8_t prev_type = ctx->tokens[token_index - 1].features.particle_type;
            if (prev_type == HARF_NAFY
                && (entry->disambiguation_flags & DISAMBIG_PRECEDED_BY_NEGATIVE)) {
                score += SCORE_EXACT_SYNTACTIC_MATCH;
            }
        }
    }

    // ── Boost common interpretations ──
    if (entry->disambiguation_flags & DISAMBIG_MOST_COMMON) {
        score += SCORE_FREQUENCY_BOOST * 5;
    }

    return score;
}
```

### 11.2 Disambiguation Algorithm

```pseudo
Algorithm: disambiguate_particle

Input:  token_index, text, syntax_context
Output: (best_entry, confidence, all_candidates)

1. Look up all entries for this particle text:
       candidates = particle_lookup_all(kb, text)

2. If only 1 candidate → return with confidence=100

3. For each candidate:
       score = score_particle_interpretation(entry, ctx, token_index)

4. Sort candidates by score (descending)

5. If top score is >= 2× second score:
       return best = candidates[0], confidence = 90
   Else:
       // Ambiguous — pass both forward
       return best = candidates[0], confidence = 50,
              secondary = candidates[1]

6. Record ambiguity in AnalysisResult:
       flags.add({
           type: AMBIGUITY,
           code: "PARTICLE_AMBIGUITY",
           message: "Particle '{text}' has multiple possible interpretations",
           candidates: candidates[0..min(3, count)]
       })
```

### 11.3 Special Homograph: مَا (mā)

The most complex homograph in the Arabic particle system:

| # | Interpretation | Disambiguation Clues | Priority |
|---|---------------|---------------------|----------|
| 1 | **Negative** (مَا نَافِيَة) | Followed by perfect verb (كَتَبَ), or imperfect verb without jussive | High |
| 2 | **Interrogative** (مَا اسْتِفْهَامِيَّة) | Followed by noun, or at start of question; can have هَذَا after | High |
| 3 | **Relative pronoun** (مَا مَوْصُولَة) | Followed by verb clause; can be preceded by preposition | Medium |
| 4 | **Masdar-forming** (مَا مَصْدَرِيَّة) | Can be replaced by أَنْ + verb (masdar ta'wili) | Medium |
| 5 | **Conditional** (مَا شَرْطِيَّة) | Followed by two verbs in jussive; can be replaced by إِنْ | Low (rare) |
| 6 | **Exclamative** (مَا تَعَجُّبِيَّة) | Before elative pattern (أَفْعَلَ); مَا أَجْمَلَ | Low |

```c
// Specialized disambiguation for مَا
int32_t disambiguate_ma(
    const SyntaxContext* ctx,
    uint32_t token_index,
    const ParticleEntry* candidates[],
    uint32_t candidate_count
) {
    if (candidate_count == 0) return -1;
    if (candidate_count == 1) return 0;

    TokenInfo* next = (token_index + 1 < ctx->token_count)
                     ? &ctx->tokens[token_index + 1]
                     : NULL;
    TokenInfo* prev = (token_index > 0)
                     ? &ctx->tokens[token_index - 1]
                     : NULL;

    // Check 1: Conditional مَا — followed by two jussive verbs
    if (next && next->features.pos == POS_VERB
        && next->features.mood == MOOD_JUSSIVE
        && has_second_jussive_verb(ctx, token_index + 1)) {
        return find_entry_by_type(candidates, candidate_count, HARF_SHART);
    }

    // Check 2: Interrogative مَا — question context
    if (ctx->sentence_type == SENTENCE_INTERROGATIVE
        || (prev && prev->features.pos == POS_INTERROGATIVE)) {
        return find_entry_by_type(candidates, candidate_count, HARF_ISTIFHAM);
    }

    // Check 3: Relative pronoun مَا — preceded by preposition
    if (prev && prev->features.pos == POS_PARTICLE
        && prev->features.particle_type == HARF_JARR) {
        return find_entry_by_type(candidates, candidate_count, HARF_MAANI);  // relative
    }

    // Check 4: Masdar-forming مَا — can be replaced by أَنْ
    if (next && next->features.pos == POS_VERB) {
        // Check if the verb clause could be a masdar ta'wili
        if (is_masdar_ta_wili_candidate(ctx, token_index)) {
            return find_entry_by_type(candidates, candidate_count, HARF_MASDARI);
        }
    }

    // Check 5: Negative مَا — followed by perfect verb
    if (next && next->features.pos == POS_VERB
        && next->features.tense == TENSE_PAST) {
        return find_entry_by_type(candidates, candidate_count, HARF_NAFY);
    }

    // Default: most common usage (negative for مَا)
    return find_most_common(candidates, candidate_count);
}
```

---

## 12. Normalization & Preprocessing

### 12.1 Normalization Pipeline

```
Raw token text (UTF-8)
    │
    ▼
Step 1: Remove tatweel (kashida, U+0640)
    │  "مـــن" → "من"
    ▼
Step 2: NFKC Unicode normalization
    │  Handles lam-alef ligatures, presentation forms
    │  "ﷺ" → "صلى الله عليه وسلم" (if desired, optional)
    ▼
Step 3: Detect and strip leading clitic prefixes
    │  "فَبِالْبَيْتِ" → prefix="فَ", stem="بِالْبَيْتِ"
    │  "وَلِلَّهِ" → prefix="وَ", stem="لِلَّهِ"
    ▼
Step 4 (optional): Strip diacritics
    │  "مِنَ" → "من"
    │  "لَمْ" → "لم"
    ▼
Normalized token for lookup
```

### 12.2 Unicode Character Handling

```c
/// Arabic Unicode character ranges used in particle normalization.
/// All values in hexadecimal.

const uint32_t ARABIC_RANGE_START      = 0x0600;
const uint32_t ARABIC_RANGE_END        = 0x06FF;
const uint32_t ARABIC_SUPPLEMENT_START = 0x0750;
const uint32_t ARABIC_SUPPLEMENT_END   = 0x077F;

// Tatweel/kashida
const uint32_t TATWEEL                 = 0x0640;

// Diacritic marks (Arabic)
const uint32_t FATHA                   = 0x064E;
const uint32_t DAMMA                   = 0x064F;
const uint32_t KASRA                   = 0x0650;
const uint32_t FATHA_TANWIN            = 0x064B;
const uint32_t DAMMA_TANWIN            = 0x064C;
const uint32_t KASRA_TANWIN            = 0x064D;
const uint32_t SHADDA                  = 0x0651;
const uint32_t SUKUN                   = 0x0652;
const uint32_t MADDA                   = 0x0653;
const uint32_t HAMZA_ABOVE             = 0x0654;
const uint32_t HAMZA_BELOW             = 0x0655;

// Special characters
const uint32_t ALIF                    = 0x0627;
const uint32_t ALIF_WITH_HAMZA_ABOVE   = 0x0623;
const uint32_t ALIF_WITH_HAMZA_BELOW   = 0x0625;
const uint32_t ALIF_WITH_MADDA         = 0x0622;
const uint32_t LAM                     = 0x0644;
const uint32_t YEH                     = 0x064A;
const uint32_t ALEF_MAQSURA            = 0x0649;

// Normalization equivalence groups for Arabic
const uint32_t ALIF_GROUP[] = {
    ALIF, ALIF_WITH_HAMZA_ABOVE, ALIF_WITH_HAMZA_BELOW, ALIF_WITH_MADDA
};

const uint32_t YEH_GROUP[] = {
    YEH, ALEF_MAQSURA
};
```

### 12.3 Normalization Function

```c
/// Normalize Arabic text for particle lookup.
/// Returns the number of bytes written to output.
uint32_t normalize_arabic_text(
    const uint8_t* input,
    uint32_t input_len,
    uint8_t* output,
    uint32_t output_capacity,
    uint32_t flags
) {
    uint32_t in_pos = 0;
    uint32_t out_pos = 0;
    bool strip_diacritics = (flags & NORM_STRIP_DIACRITICS) != 0;

    while (in_pos < input_len && out_pos < output_capacity - 4) {
        uint32_t cp = decode_utf8(input, input_len, &in_pos);
        if (cp == 0) break;  // Invalid UTF-8

        // Step 1: Remove tatweel
        if (cp == TATWEEL) continue;

        // Step 2: NFKC normalization (simplified for Arabic)
        // For production: use ICU's UNormalizer2 or similar
        cp = nfkc_simplified(cp);

        // Step 3: Strip diacritics (if flag set)
        if (strip_diacritics) {
            if (is_arabic_diacritic(cp)) continue;

            // Normalize alif variants
            for (int i = 0; i < 4; i++) {
                if (cp == ALIF_GROUP[i]) {
                    cp = ALIF;  // Normalize all alifs to bare alif
                    break;
                }
            }

            // Normalize yeh/alef maqsura
            if (cp == ALEF_MAQSURA) cp = YEH;
        }

        // Encode back to UTF-8
        out_pos += encode_utf8(cp, output + out_pos, output_capacity - out_pos);
    }

    // Null-terminate (for C string compatibility)
    if (out_pos < output_capacity) {
        output[out_pos] = '\0';
    }

    return out_pos;
}

/// Check if a Unicode code point is an Arabic diacritic.
bool is_arabic_diacritic(uint32_t cp) {
    return (cp >= 0x064B && cp <= 0x0655)
        || cp == SHADDA
        || cp == SUKUN;
}
```

### 12.4 NFKC Simplification for Arabic

The following Arabic-specific normalization rules are applied during NFKC normalization:

| Input | Normalized | Reason |
|-------|-----------|--------|
| ﻛ (U+FEDB) | ك (U+0643) | Initial form → isolated |
| ﻜ (U+FEDC) | ك (U+0643) | Medial form → isolated |
| ﻚ (U+FEDA) | ك (U+0643) | Final form → isolated |
| ﻞ (U+FEDE) | ل (U+0644) | Initial form → isolated |
| ... (all presentation forms) | ... → isolated | All presentation forms normalize |
| ﻻ (U+FEFB) | لا (U+0644 U+0627) | Lam-alef ligature → two chars |
| ﻷ (U+FEF7) | لأ (U+0644 U+0623) | Lam-alef with hamza above |
| ﻹ (U+FEF9) | لإ (U+0644 U+0625) | Lam-alef with hamza below |

---

## 13. KB Compilation Pipeline

### 13.1 Compiler Command

```bash
# Compile particle YAML sources into binary .agos-kb
agos kb compile \
    --kb=KB-0005 \
    --source=/knowledge/KB-0005/ \
    --output=KB-0005-v1.0.0.agos-kb \
    --level=compact          # compact | full
    [--validate]              # Run validation suite
    [--stats]                 # Print compilation statistics
```

### 13.2 Compilation Steps

```pseudo
Algorithm: compile_particle_kb

Input:  /knowledge/KB-0005/*.yaml (13+ files)
Output: KB-0005-v1.0.0.agos-kb

Step 1: LOAD SOURCE FILES
    ├── Read metadata.yaml
    ├── Read all 13 category YAML files
    ├── Parse YAML → Vec<ParticleEntry>
    └── Validate against KB-0005 JSON Schema

Step 2: NORMALIZE TEXTS
    ├── Apply NFKC normalization to all particle texts
    ├── Check for duplicate normalized forms
    ├── Resolve homograph groups
    └── Assign homograph_group_id to matching entries

Step 3: BUILD STRING TABLE
    ├── Collect all strings: texts, meanings, grammar notes,
    │   usage notes, examples, disambiguation hints
    ├── Deduplicate identical strings
    ├── Frequency-sort (most common first)
    └── Build offset table

Step 4: PACK ENTRIES
    ├── For each ParticleEntry:
    │   ├── Map string offset references
    │   ├── Pack classification bytes
    │   ├── Pack governance bytes
    │   ├── Set disambiguation flags
    │   └── Zero-fill remaining bytes
    └── Write to EntryTable

Step 5: BUILD PERFECT HASH INDEX
    ├── For each entry's normalized text:
    │   ├── Compute cityhash64(text, HASH_SEED)
    │   ├── bucket = hash % 256
    │   └── Append (hash >> 32, particle_id) to bucket
    ├── Sort each bucket
    ├── Verify no duplicate hash values
    └── Write bucket table + hash entries

Step 6: SERIALIZE
    ├── Write header (64 bytes)
    ├── Write hash index
    ├── Write entry table
    ├── Write string table
    ├── Compute SHA-256 of sections
    ├── Write header again with checksum
    └── Write end marker

Step 7: VALIDATE
    ├── Load compiled file
    ├── Verify all particles → lookup → match
    ├── Verify all homographs → lookup_all → correct count
    ├── Verify checksum
    └── Print statistics
```

### 13.3 Validation During Compilation

```c
/// Validate compiled particle KB.
typedef struct CompileValidation {
    uint32_t total_entries;
    uint32_t missing_meanings;        // Entries without meaning strings
    uint32_t missing_grammar;         // Entries without grammar descriptions
    uint32_t homograph_groups;        // Number of homograph groups
    uint32_t max_homograph_group_size; // Largest homograph group
    uint32_t total_hash_collisions;   // Number of hash collisions resolved
    uint32_t max_bucket_size;         // Largest bucket size
    uint32_t min_bucket_size;         // Smallest bucket size
    uint32_t avg_bucket_size;         // Average bucket size

    // Normalization statistics
    uint32_t normalization_changes;   // Number of texts changed by NFKC
    uint32_t clitic_strippable;       // Number of particles that are clitics

    // Error counts
    uint32_t errors;                  // Fatal errors
    uint32_t warnings;                // Non-fatal issues
    char     error_messages[256][256]; // Error details
} CompileValidation;
```

---

## 14. Performance Model

### 14.1 Latency Budget Breakdown

| Operation | Budget (ns) | Cumulative (ns) | Notes |
|-----------|-------------|-----------------|-------|
| Normalize token | 100 | 100 | NFKC + tatweel removal |
| Strip diacritics (if needed) | 50 | 150 | Only when first lookup fails |
| Hash computation | 30 | 180 | CityHash64 for short strings |
| Bucket lookup | 5 | 185 | Direct array index |
| Binary search in bucket | 20 | 205 | ~3 iterations for ~8 entries |
| String comparison | 20 | 225 | Typically 2–5 byte comparison |
| **Total hit** | **~250** | **~250** | Under 500 ns target |
| Governance resolution | 50 | 300 | Struct fill + checks |
| **Total hit + gov** | **~300** | **~300** | Common case |
| Clitic retry (if needed) | +200 | ~500 | Second lookup |
| Diacritic retry (if needed) | +200 | ~700 | Third lookup |
| **Total miss** | **~700** | **~700** | All retries exhausted |

### 14.2 Memory Map Pattern

```
┌──────────────┐  ◄── mmap() base address
│              │
│   HEADER     │  offset 0
│   (64 B)     │
├──────────────┤
│              │
│  HASH INDEX  │  offset = header.hash_index_offset
│  (~4 KB)     │
├──────────────┤
│              │
│ ENTRY TABLE  │  offset = header.entry_table_offset
│  (~13 KB)    │  (200 entries × 64 B = 12,800 B)
├──────────────┤
│              │
│ STRING TABLE │  offset = header.string_table_offset
│  (~170 KB)   │  (compact level)
│              │
└──────────────┘
```

**Total mapped memory:** ~190 KB (compact) to ~520 KB (full). Well within the 2–5 MB budget.

### 14.3 Cache Behavior Analysis

| Section | Size | Cache Lines (64 B) | Access Pattern |
|---------|------|--------------------|----------------|
| Header | 64 B | 1 | Loaded once at init |
| Bucket table | 2,048 B | 32 | Random access (256 buckets) |
| Hash entries | ~1,600 B | 25 | Sequential within bucket (avg ~1 entry) |
| Entry table | ~12,800 B | 200 | Random access (one per hit) |
| String table | ~170,000 B | 2,656 | String comparison (first 8 bytes critical) |

**Hot path (on hit):**
1. Load bucket entry: 1 cache line (likely L1 hit for frequently-accessed particles)
2. Load hash entry: 1 cache line (same bucket, often same line as bucket entry)
3. Load entry table row: 1 cache line (64 B entry, full line)
4. Load first 8 bytes of string: 1 cache line (string data, potentially L2)

**Total cache lines per lookup:** 3–4 (L1/L2).
**Expected L1 hit rate:** > 90% for common particles (فِي, مِنْ, لِ, etc.).

### 14.4 Performance Targets by Deployment Profile

| Profile | Lookup Latency (p50) | Lookup Latency (p99) | Memory (compact) | Notes |
|---------|---------------------|---------------------|------------------|-------|
| **Interactive** | < 200 ns | < 500 ns | 2 MB | Tight latency, few homographs resolved |
| **Server** | < 300 ns | < 1 μs | 3 MB | Balanced, some disambiguation |
| **Batch** | < 500 ns | < 2 μs | 5 MB (full) | Full string data, all homographs |
| **Educational** | < 1 μs | < 5 μs | 5 MB (full) | Full data + description strings |
| **Debug** | < 2 μs | < 10 μs | 5 MB (full) | With tracing |

---

## 15. Testing & Validation

### 15.1 Test Categories

| Category | Tests | Coverage |
|----------|-------|----------|
| **Binary format** | 20 | Header parsing, section offsets, checksum verification |
| **Hash index** | 30 | Perfect hash correctness, bucket distribution, collision handling |
| **Lookup** | 50 | Exact match, no-diacritics match, clitic-stripped match, miss |
| **Homographs** | 25 | All common homographs (مَا, إِنْ, لَا, etc.), scoring, ranking |
| **Governance** | 30 | Case assignment, mood assignment, structural effects |
| **Normalization** | 20 | NFKC, tatweel removal, diacritic stripping, clitic splitting |
| **Compilation** | 15 | YAML → binary, round-trip, determinism |
| **Performance** | 10 | Latency budgets, memory budgets, cache miss rates |
| **Regression** | 50 | Known particle usages from Quran + classical texts |
| **Cross-implementation** | 5 | Same binary → same results across Rust, C, Python, TS |

**Total:** ~255 tests.

### 15.2 Test Fixture Format

```jsonc
{
    "spec": "KB-0008/particle-test",
    "version": "1.0.0",
    "test_name": "lookup_preposition_min",
    "description": "Look up the preposition مِنْ (min) with exact text",

    "input": {
        "text": "مِنْ",
        "normalization": "default"
    },

    "expected": {
        "found": true,
        "particle_id": ">= 0",
        "match_type": 0,           // exact match
        "entry_check": {
            "particle_type": 0,     // HARF_JARR
            "governs_case": 3,      // CASE_GENITIVE
            "governs_mood": 0,      // MOOD_NONE
            "attaches_to": 1,       // ATTACH_NEXT_WORD
            "category_flags": "CAT_PREPOSITION | CAT_CASE_GOVERNING"
        },
        "governance": {
            "case_effect": 3,       // GENITIVE
            "case_confidence": 95,
            "mood_effect": 0,
            "structural_effect": 0,
            "phonetic_change_type": 1  // MIN_TO_MINA
        }
    }
}
```

### 15.3 Regression Test Suite

The regression suite validates the particle system against known Arabic texts:

```c
/// Known particle usages for regression testing.
typedef struct RegressionTest {
    const char* text;               // Arabic text containing the particle
    const char* particle;           // Expected particle
    uint8_t     expected_type;      // HARF_JARR, HARF_NASB, etc.
    uint8_t     expected_case;      // CASE_GENITIVE, etc.
    uint8_t     expected_mood;      // MOOD_SUBJUNCTIVE, etc.
    const char* source;             // Source citation
} RegressionTest;

// Sample regression tests
RegressionTest REGRESSION_TESTS[] = {
    // Quranic examples
    { "بِسْمِ اللَّهِ", "بِ", HARF_JARR, CASE_GENITIVE, MOOD_NONE, "Quran 1:1" },
    { "مِنَ الْجِنَّةِ", "مِن", HARF_JARR, CASE_GENITIVE, MOOD_NONE, "Quran 114:5" },
    { "لَنْ تَنَالُوا", "لَنْ", HARF_NASB, CASE_NONE, MOOD_SUBJUNCTIVE, "Quran 3:92" },
    { "لَمْ يَلِدْ", "لَمْ", HARF_JAZM, CASE_NONE, MOOD_JUSSIVE, "Quran 112:3" },
    { "إِنَّ اللَّهَ", "إِنَّ", HARF_NASIKH, CASE_ACCUSATIVE, MOOD_NONE, "Quran 2:257" },

    // Classical Arabic examples
    { "إِنْ تَكْتُبْ تَنْجَحْ", "إِنْ", HARF_SHART, CASE_NONE, MOOD_JUSSIVE, "Classical" },
    { "هَلْ كَتَبْتَ؟", "هَلْ", HARF_ISTIFHAM, CASE_NONE, MOOD_NONE, "MSA" },
    { "يَا مُحَمَّدُ", "يَا", HARF_NIDA, CASE_NOMINATIVE, MOOD_NONE, "Classical" },
    { "لَا تَقْرَبُوا", "لَا", HARF_NAHIYAH, CASE_NONE, MOOD_JUSSIVE, "Quran 2:187" },

    // MSA examples
    { "سَوْفَ أَكْتُبُ", "سَوْفَ", HARF_ISTIQBAL, CASE_NONE, MOOD_NONE, "MSA" },
    { "مَا كَتَبْتُ", "مَا", HARF_NAFY, CASE_NONE, MOOD_NONE, "MSA" },

    // Terminators
    { NULL, NULL, 0, 0, 0, NULL }
};
```

### 15.4 Quality Gates

| Gate | Criteria | Blocking |
|------|----------|----------|
| **Binary integrity** | Magic, version, checksum all valid | Yes |
| **Hash completeness** | Every entry reachable via hash lookup | Yes |
| **Homograph detection** | All known homographs correctly grouped | Yes |
| **Governance correctness** | Case/mood effects match KB-0005 linguistic spec | Yes |
| **Normalization round-trip** | After normalize + lookup, entry text matches | Yes |
| **No missing strings** | Every entry has meaning and grammar strings | Warning |
| **Bucket distribution** | Max bucket < 10 entries (200 / 256 ≈ 0.78 avg) | Warning |
| **Compilation determinism** | Same source → byte-for-byte identical output | Yes |

---

## 16. Cross-References

### 16.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| KB-0005 | Particles — Linguistic Content | Source data for KB-0008 compiled module |
| KB-0005 §4 | Particle Entry Schema | Source schema mapped to binary entry format |
| KB-0005 §5–15 | Particle Categories (13 groups) | Each category corresponds to a ParticleType enum value |
| KB-0005 §16 | Particle Ambiguity Resolution | Disambiguation algorithm implemented in §11 |
| KB-0005 §17 | Particle Matching Algorithm | Fast-path algorithm implemented in §9 |
| KB-0005 §18 | Serialization & Storage | Compiled binary format refined in §3–§6 |
| KB-0006 | Pronouns Database | Companion fast-path KB (checked after particles) |
| KB-0007 | Morphological Features | POS encoding for particles (POS = 2) |
| SPEC-0101 §4.1 | Morphology Engine Fast Path | MOD-04 Step 3.1 integration (§9 of this doc) |
| SPEC-0101 §6 | Feature Extraction | Particle feature population in bitfield |
| SPEC-0101 §13 | Morphology Engine Subsystem 1 | Particle identification subsystem |
| KB-OVERVIEW §3.5 | KB-0005 Summary | Cross-KB architecture and budgets |
| SPEC-0102 §3 | POS Feature Reference | Particle POS code (2) in feature taxonomy |
| SPEC-0102 §14 | Plugin Extension Mechanism | Particle feature IDs (20–24) in reserved bits |
| RFC-0004 §7.6 | Particle Library (rule predicates) | DSL predicates for particle governance |
| RFC-0004 §8 | Particle Governance Rules | Rule engine integration of particle effects |
| SPEC-0001-C3 §3.1 | MOD-04 Fast Path | Pipeline position of particle check |

### 16.2 External References

| Reference | Relevance |
|-----------|-----------|
| **CityHash (Google)** | 64-bit non-cryptographic hash for hash index |
| **xxHash** | Alternative to CityHash for hashing short strings |
| **ICU Unicode Normalizer** | NFKC normalization for Arabic text |
| **Unicode Standard §9.2** | Arabic Unicode ranges, presentation forms, lam-alef |
| **Sibawayh, Al-Kitab** | Foundational grammar for particle classification |
| **Ibn Hisham, Mughni al-Labib** | Particle meaning and governance reference |

---

## Progress Summary

**KB-0008: Particles Database — Developer Reference & Compiled Module**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Architecture Overview | ✓ COMPLETE |
| 3 | Compiled Binary Format | ✓ COMPLETE |
| 4 | Perfect Hash Index | ✓ COMPLETE |
| 5 | Entry Table | ✓ COMPLETE |
| 6 | String Table | ✓ COMPLETE |
| 7 | Particle Lookup API | ✓ COMPLETE |
| 8 | Governance Resolution API | ✓ COMPLETE |
| 9 | MOD-04 Fast-Path Integration | ✓ COMPLETE |
| 10 | MOD-05 Syntactic Integration | ✓ COMPLETE |
| 11 | Homograph Disambiguation Engine | ✓ COMPLETE |
| 12 | Normalization & Preprocessing | ✓ COMPLETE |
| 13 | KB Compilation Pipeline | ✓ COMPLETE |
| 14 | Performance Model | ✓ COMPLETE |
| 15 | Testing & Validation | ✓ COMPLETE |
| 16 | Cross-References | ✓ COMPLETE |

---

*End of KB-0008*
