---
rfc_id: RFC-0002
title: Grammar Bytecode Format — Binary Container Specification
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - ADR-0002: Why Grammar Bytecode
  - SPEC-0001-C3: Compilation Pipeline (MOD-09 BytecodeGenerator)
  - SPEC-0001-C3: Compilation Pipeline (MOD-10 GVM)
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-9)
  - SPEC-0001-C9: Performance Targets & Constraints
  - RFC-0003: Grammar Virtual Machine (instruction set & execution model)
  - KB-0007: Morphological Features Taxonomy (planned)
supersedes: None
---

# RFC-0002: Grammar Bytecode Format — Binary Container Specification

## Table of Contents

1. [Introduction](#1-introduction)
2. [Design Principles](#2-design-principles)
3. [File Format Overview](#3-file-format-overview)
4. [Primitive Encoding](#4-primitive-encoding)
5. [Header Section](#5-header-section)
6. [Metadata Section](#6-metadata-section)
7. [String Table Section](#7-string-table-section)
8. [Token Section](#8-token-section)
9. [Feature Section](#9-feature-section)
10. [Constituent Section](#10-constituent-section)
11. [Instruction Section](#11-instruction-section)
12. [Rule Application Section](#12-rule-application-section)
13. [Evidence Section](#13-evidence-section)
14. [End Marker](#14-end-marker)
15. [Optimization Levels](#15-optimization-levels)
16. [Versioning & Compatibility](#16-versioning--compatibility)
17. [Implementation Guidance](#17-implementation-guidance)
18. [Conformance Test Suite](#18-conformance-test-suite)
19. [Cross-References](#19-cross-references)

---

## 1. Introduction

### 1.1 Purpose

This RFC defines the **Grammar Bytecode Format** — the binary container format for AGOS Grammar Bytecode. This format is the output of MOD-09 (BytecodeGenerator) and the input to MOD-10 (Grammar Virtual Machine). It is the canonical serialization format for completed grammatical analyses, designed for:

- **Compactness:** 10–40× smaller than equivalent JSON GIR (target: < 10 KB per typical sentence).
- **Self-contained execution:** All data required by the GVM is embedded in the bytecode; no external KB references.
- **Deterministic encoding:** Same GIR + same configuration = byte-for-byte identical bytecode.
- **Streaming verification:** CRC32C checksums per section enable incremental integrity checking.
- **Versioned evolution:** Major.minor.patch versioning with clear backward compatibility semantics.

### 1.2 Relationship to Other Documents

| Document | Relationship |
|----------|-------------|
| **ADR-0002** | Architectural rationale for bytecode format design decisions |
| **RFC-0003** | Instruction set and execution model of the GVM that consumes this format |
| **SPEC-0001-C3 (MOD-09)** | BytecodeGenerator that produces bytecode in this format |
| **SPEC-0001-C4** | Formal interface for MOD-09 and MOD-10 |
| **SPEC-0001-C5 (IR-9)** | Logical schema for Grammar Bytecode |
| **SPEC-0001-C9** | Performance targets: size (< 10 KB/sentence), compression (< 20% of JSON) |

### 1.3 Terms and Definitions

| Term | Definition |
|------|------------|
| **Bytecode** | The complete binary file (.agos extension) containing a serialized grammatical analysis |
| **Section** | A logical subdivision of the bytecode file with a specific type and CRC32C checksum |
| **Varint** | Variable-length integer encoding using LEB128 |
| **Offset** | A byte position relative to the start of the file or section |
| **Magic bytes** | The 4-byte sequence `0x41474F53` ("AGOS") identifying the format |
| **String table** | A deduplicated list of all strings referenced in the bytecode |
| **Feature bitfield** | A packed 64-bit representation of morphological features (per KB-0007 taxonomy) |

---

## 2. Design Principles

### 2.1 Core Principles

1. **Self-Describing.** The bytecode header contains all metadata needed to parse the file: version, section table, string count, token count, and resource bounds. No external configuration is required to parse or execute the bytecode.

2. **Compact by Design.** Every encoding choice favors compactness within the constraints of safe, deterministic parsing. Variable-length integers, feature bitfields, string interning, and delta encoding reduce size without sacrificing verifiability.

3. **Verifiable at Rest.** Every section carries a CRC32C checksum. The file can be fully verified without executing a single instruction. Structural constraints (section ordering, bounds) are checkable at load time.

4. **Streaming-Friendly.** Sections are ordered and sized so that a streaming parser can process tokens sequentially, build the constituent tree incrementally, and execute instructions without random access to earlier sections. The string table is placed early (after header/metadata) to enable lazy resolution.

5. **Deterministic Encoding.** Given the same GIR, the same bytecode version, and the same optimization level, the BytecodeGenerator MUST produce byte-for-byte identical output. All encoding decisions (varint padding, string ordering, section layout) are fully specified with no implementation freedom.

6. **Extensible via Versioning.** New instructions, section types, and feature encodings are introduced through version bumps. Backward compatibility within a major version is guaranteed. Forward compatibility (GVM vM.n runs bytecode vM.n-1) is guaranteed.

### 2.2 Byte-Level Conventions

| Convention | Value | Rationale |
|------------|-------|-----------|
| **Byte order** | Little-endian | Matches dominant hardware (x86, ARM) |
| **Text encoding** | UTF-8 | Universal, efficient for Arabic |
| **Integer encoding** | LEB128 varint for small values; fixed-size (u16, u32, u64) for offsets and sizes | Balances compactness with random-access speed |
| **Signed integers** | Zigzag LEB128 | Compact encoding of signed values (jump offsets) |
| **Floating point** | IEEE 754 binary64 | Universal, hardware-supported |
| **Checksum** | CRC32C (Castagnoli, polynomial 0x1EDC6F41) | Better error detection than CRC32, hardware-accelerated on modern CPUs |
| **Alignment** | No alignment padding | Every byte is significant; parsers handle unaligned access |
| **Reserved bytes** | Must be zero | Future use; parsers MUST reject non-zero reserved bytes |

### 2.3 File Extension and MIME Type

| Property | Value |
|----------|-------|
| **File extension** | `.agos` |
| **MIME type** | `application/x-agos-bytecode` |
| **Magic bytes (hex)** | `41 47 4F 53` |

---

## 3. File Format Overview

### 3.1 Top-Level Structure

```
┌──────────────────────────────────────────────────────────────┐
│                    HEADER SECTION (fixed size)                │
│  Magic (4B) │ Version (6B) │ Flags (2B) │ Total Size (4B)   │
│  Section Count (2B) │ Section Table (N × 10B)               │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐               │
│  │ SECTION 1 │  │ SECTION 2 │  │ SECTION 3 │  ...           │
│  │ METADATA  │  │ STRINGS   │  │ TOKENS    │               │
│  └───────────┘  └───────────┘  └───────────┘               │
│                                                              │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐               │
│  │ SECTION 4 │  │ SECTION 5 │  │ SECTION 6 │  ...           │
│  │ FEATURES  │  │ CONSTIT.  │  │ INSTRUCT. │               │
│  └───────────┘  └───────────┘  └───────────┘               │
│                                                              │
│  ┌───────────┐  ┌───────────┐                                │
│  │ SECTION 7 │  │ SECTION 8 │                                │
│  │ RULES     │  │ EVIDENCE  │                                │
│  └───────────┘  └───────────┘                                │
├──────────────────────────────────────────────────────────────┤
│                    END MARKER (4 bytes)                       │
└──────────────────────────────────────────────────────────────┘
```

### 3.2 Section Types

| ID | Section Name | Required | Description |
|----|-------------|----------|-------------|
| `0x01` | METADATA | Yes | Pipeline metadata, KB versions, school, config snapshot |
| `0x02` | STRINGS | Yes | Deduplicated UTF-8 string table |
| `0x03` | TOKENS | Yes | Token data (indices, text refs, types) |
| `0x04` | FEATURES | Yes | Packed morphological feature bitfields |
| `0x05` | CONSTITUENTS | Yes | Constituent tree structure |
| `0x06` | INSTRUCTIONS | Yes | Executable bytecode instruction stream |
| `0x07` | RULES | No | Rule application records (absent if no rules applied) |
| `0x08` | EVIDENCE | No | Evidence trail entries (absent if not embedded) |

### 3.3 Section Ordering

Sections MUST appear in the order specified above. This ordering enables:
- **Progressive parsing:** The string table is available before tokens are parsed. Features are available before constituents reference them.
- **Streaming verification:** Each section can be verified and passed downstream before the next section is fully read.
- **Lazy loading:** GVM implementations MAY skip evidence and rule sections during execution.

### 3.4 File Size Constraints

| Constraint | Minimum | Maximum | Rationale |
|------------|---------|---------|-----------|
| **Total file size** | 32 bytes | 10 MiB | 10 MiB covers extreme cases (100+ word sentences with full evidence) |
| **Section count** | 6 (required) | 16 (8 required + 8 reserved) | Room for future section types |
| **String table entries** | 0 | 65,535 | 2^16 unique strings per bytecode |
| **Tokens** | 0 | 4,096 | Max 4,096 tokens per bytecode (100+ word sentences with clitics) |
| **Constituents** | 0 | 8,192 | Tree nodes for complex sentences |
| **Instructions** | 1 (HALT) | 1,048,576 | 1M instruction limit (within GVM step budget) |
| **Rule applications** | 0 | 65,535 | Per-bytecode limit |

---

## 4. Primitive Encoding

### 4.1 Unsigned Varint (LEB128)

Unsigned integers use LEB128 encoding, where each byte uses 7 bits for data and the high bit (0x80) as a continuation flag:

```
Value      | Encoded Bytes
───────────┼─────────────────
0          | 0x00
1          | 0x01
127        | 0x7F
128        | 0x80 0x01
16383      | 0xFF 0x7F
16384      | 0x80 0x80 0x01
2097151    | 0xFF 0xFF 0x7F
```

**Decoding algorithm:**
```
function decode_uleb128(bytes, offset):
    result = 0
    shift = 0
    while True:
        byte = bytes[offset]
        result |= (byte & 0x7F) << shift
        if (byte & 0x80) == 0:
            break
        shift += 7
        offset += 1
    return (result, offset + 1)
```

**Redundant encoding:** LEB128 permits redundant encodings (e.g., 0x00 and 0x80 0x00 both encode zero). Bytecode generators MUST emit the shortest encoding. Bytecode parsers MUST accept any valid encoding (including redundant ones) for robustness.

### 4.2 Signed Varint (Zigzag LEB128)

Signed integers use Zigzag LEB128 encoding, where negative values are mapped to positive unsigned integers before LEB128 encoding:

```
n           | Zigzag(n)  | Encoded Bytes
────────────┼────────────┼─────────────────
0           | 0          | 0x00
-1          | 1          | 0x01
1           | 2          | 0x02
-2          | 3          | 0x03
2           | 4          | 0x04
-3          | 5          | 0x05
```

**Encoding:**
```
zigzag(n) = (n << 1) ^ (n >> 63)    // For i64
zigzag(n) = (n << 1) ^ (n >> 31)    // For i32
```

**Decoding:**
```
decode_zigzag(z) = (z >> 1) ^ -(z & 1)
```

### 4.3 Fixed-Size Integers

| Type | Size | Encoding |
|------|------|----------|
| `u16` | 2 bytes | Little-endian unsigned 16-bit integer |
| `u32` | 4 bytes | Little-endian unsigned 32-bit integer |
| `u64` | 8 bytes | Little-endian unsigned 64-bit integer |
| `i16` | 2 bytes | Little-endian signed 16-bit integer (two's complement) |
| `i32` | 4 bytes | Little-endian signed 32-bit integer (two's complement) |
| `i64` | 8 bytes | Little-endian signed 64-bit integer (two's complement) |

### 4.4 Float Encoding

| Type | Size | Encoding |
|------|------|----------|
| `f64` | 8 bytes | IEEE 754 binary64, little-endian byte order |

### 4.5 String Encoding

Strings are encoded as length-prefixed UTF-8 byte sequences:

```
┌────────────────────┬──────────────────────────────┐
│ Length (varint)    │ UTF-8 bytes (length bytes)   │
├────────────────────┼──────────────────────────────┤
│ N (1–5 bytes)      │ N bytes of UTF-8 text        │
└────────────────────┴──────────────────────────────┘
```

- Length is the byte count of the UTF-8 representation, not the character count.
- Empty strings are encoded as a single `0x00` byte (length = 0).
- Strings MUST be valid UTF-8. Invalid UTF-8 sequences MUST be rejected by the verifier.
- Strings are NOT null-terminated. The length prefix determines the boundary.

### 4.6 String Reference

When a section references a string from the string table, the reference is encoded as:

```
┌────────────────────┐
│ Index (varint)     │  Index into string table (0-based)
└────────────────────┘
```

The BytecodeGenerator ensures all string references are within the string table bounds. The GVM verifier checks this at load time.

---

## 5. Header Section

### 5.1 Layout

The header section is the only section with a fixed-size layout. It occupies the first 32+ bytes of the file and is **not** preceded by a section header (it IS the implicit section 0).

```
Offset  Size    Field                Value / Encoding
──────  ────    ─────                ───────────────
0       4       Magic bytes          0x41474F53 ("AGOS")
4       2       Version major        u16
6       2       Version minor        u16
8       2       Version patch        u16
10      2       Flags                Bitmask (u16)
12      4       Total file size      u32 (bytes from offset 0 to end marker inclusive)
16      2       Section count        u16 (number of sections in section table)
18      2       String count         u16 (number of entries in string table)
20      2       Token count          u16 (number of tokens in token section)
22      2       Feature count        u16 (number of feature bitfields)
24      2       Constituent count    u16 (number of constituent nodes)
26      2       Instruction count    u32 (number of instructions, 4 bytes)
30      2       Reserved             Must be zero
────────────────────────────────────────────────────────
32      N       Section table        Array of SectionEntry (10 bytes each)
```

**Total header size:** 32 + (10 × section_count) bytes.

### 5.2 Flags Bitmask

```
Bit    Name                Description
───    ────                ───────────
0      HAS_EVIDENCE        Evidence section is present
1      HAS_RULES           Rule application section is present
2      HAS_OPT_LEVEL_1     Optimization level 1 applied
3      HAS_OPT_LEVEL_2     Optimization level 2 applied
4      EMBEDDED_TEXT       Original input text is embedded in metadata
5      LARGE_OFFSETS       Use u32 (not u16) for section offsets
6–15   RESERVED            Must be zero
```

### 5.3 Section Entry

```
Offset  Size    Field                Encoding
──────  ────    ─────                ────────
0       1       Section ID           0x01–0x08 (see Section 3.2)
1       1       Reserved             Must be zero
2       4       Section offset       u32 (byte offset from start of file)
6       4       Section size         u32 (byte size of section data)
```

If flag bit 5 (LARGE_OFFSETS) is set, offsets and sizes use u32 (4 bytes each, as shown above). Without this flag, u16 (2 bytes) is used for offsets and sizes, limiting each section to 65,535 bytes and the file to ~1 MB. The large-offsets flag is RECOMMENDED for all production bytecode.

### 5.4 Header Verification

The GVM verifier MUST perform the following checks on the header:

1. **Magic bytes:** Bytes 0–3 MUST equal `0x41474F53`.
2. **Version:** `version.major` MUST be ≤ the GVM's supported major version.
3. **Section count:** MUST be between 6 (required sections) and 16 (max).
4. **Section table:** Section IDs MUST be unique, in ascending order, and contiguous from `0x01`.
5. **Total file size:** MUST equal the actual file size (parsers verify this after reading the end marker).
6. **Reserved bytes:** All reserved bytes MUST be zero. Non-zero reserved bytes MUST cause a verification error.
7. **Section bounds:** Every section's offset + size MUST NOT exceed `total_file_size - 4` (end marker).

---

## 6. Metadata Section

### 6.1 Layout

The metadata section contains structured pipeline metadata encoded as a sequence of key-value entries.

```
┌──────────────────────────────────────────────────────────────┐
│  ┌────────────┐  ┌────────────────┐  ┌────────────────────┐  │
│  │ Entry      │  │ Entry          │  │ Entry              │  │
│  │ Count      │  │ 1: school      │  │ 2: kb_versions     │  │
│  │ (varint)   │  │ (string)       │  │ (map: N pairs)     │  │
│  └────────────┘  └────────────────┘  └────────────────────┘  │
│  ...                                                         │
│  ┌────────────────────┐  ┌────────────────────────────────┐  │
│  │ Entry N-1:         │  │ Entry N:                      │  │
│  │ input_text         │  │ input_text_hash (32 bytes)    │  │
│  └────────────────────┘  └────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### 6.2 Metadata Entry Format

Each entry is a key-value pair:

```
┌────────────────────┬────────────────────┬──────────────────────┐
│ Key (string)       │ Value Type (u8)   │ Value (varies)       │
├────────────────────┼────────────────────┼──────────────────────┤
│ Varint-prefixed    │ 0x00 = string      │ Per type definition  │
│ UTF-8 string       │ 0x01 = integer     │                      │
│                    │ 0x02 = string[]    │                      │
│                    │ 0x03 = map         │                      │
│                    │ 0x04 = bytes       │                      │
└────────────────────┴────────────────────┴──────────────────────┘
```

### 6.3 Required Metadata Entries

| Key | Type | Description |
|-----|------|-------------|
| `spec` | string | Always `"SPEC-0001"` |
| `version` | string | Bytecode format version (e.g., `"1.0.0"`) |
| `created_at` | string | ISO 8601 timestamp with timezone |
| `school` | string | Grammar school (e.g., `"basra"`) |
| `pipeline_version` | string | AGOS platform version that generated this bytecode |
| `knowledge_versions` | map | KB version map (key=KB ID, value=semver) |
| `input_text_hash` | bytes | SHA-256 of original input text (32 bytes exactly) |
| `input_text` | string | Original input text (only if EMBEDDED_TEXT flag is set) |
| `bytecode_generator` | string | Implementer identification (e.g., `"agos-rust 0.1.0"`) |

### 6.4 Optional Metadata Entries

| Key | Type | Description |
|-----|------|-------------|
| `optimization_level` | integer | Optimization level applied (0, 1, or 2) |
| `compression_ratio` | float | Ratio of bytecode size to equivalent JSON GIR size |
| `gir_json_size_bytes` | integer | Size of equivalent JSON GIR in bytes |
| `notes` | string | Free-form generation notes |
| `rule_set_version` | string | Rule set version string |

### 6.5 Metadata Verification

1. All required metadata entries MUST be present.
2. The `input_text_hash` MUST be exactly 32 bytes (SHA-256).
3. If the EMBEDDED_TEXT flag is NOT set, the `input_text` entry MUST be absent.
4. Unknown metadata entries MUST be preserved (not rejected). They are treated as opaque data.

---

## 7. String Table Section

### 7.1 Purpose

The string table provides a deduplicated, compact representation of all strings referenced in the bytecode. Strings are stored once and referenced by index. This provides:

- **30–50% size reduction** compared to inline strings.
- **Fast equality checks** (compare indices, not string bytes).
- **Lazy loading** (strings are resolved on demand during GVM execution).

### 7.2 Layout

```
┌─────────────────────┬──────────────────────────┬──────────────────────────┐
│ Entry Count         │ Entry 0                  │ Entry 1                  │
│ (varint)            │ Length (varint) + UTF-8  │ Length (varint) + UTF-8  │
└─────────────────────┴──────────────────────────┴──────────────────────────┘
```

### 7.3 String Interning Rules

1. **Deduplication:** Every unique string value MUST appear exactly once in the string table. The BytecodeGenerator MUST deduplicate strings before writing the table.
2. **Ordering:** Strings are ordered by frequency (most-frequently-referenced first). This enables faster access for common strings and improves cache locality.
3. **Empty string:** The empty string (`""`) MUST be entry 0 if it is referenced anywhere. Otherwise, it SHOULD be omitted.
4. **Maximum index:** String indices are encoded as varints. The maximum index is `string_count - 1`.

### 7.4 String Table Capacity

| Property | Limit |
|----------|-------|
| Maximum entries | 65,535 (limited by header's u16 `string_count`) |
| Maximum bytes per string | 32,767 (1.5× a typical Quranic ayah) |
| Maximum total string data | 10 MiB (limited by total file size) |

### 7.5 String Table Verification

1. Every string entry's length prefix MUST be followed by exactly `length` bytes.
2. All string bytes MUST be valid UTF-8.
3. No two entries in the string table may contain identical byte sequences (deduplication invariant).

---

## 8. Token Section

### 8.1 Purpose

The token section encodes the token stream: each token's text (as a string table reference), its type, and its byte offsets in the original text.

### 8.2 Layout

```
┌────────────────┬──────────────────────┬──────────────────────┐
│ Token 0        │ Token 1              │ ... Token N-1        │
├────────────────┼──────────────────────┼──────────────────────┤
│ Variable per   │ Variable per token   │                      │
│ token (5-12 B) │                      │                      │
└────────────────┴──────────────────────┴──────────────────────┘
```

### 8.3 Token Record Format

Each token record has the following format:

```
┌─────────────┬────────────────┬────────────────┬────────────────┬──────────────┐
│ Text Ref    │ Token Type     │ Start Offset   │ End Offset     │ Feature Ref  │
│ (varint)    │ (u8)           │ (varint)       │ (varint)       │ (varint)     │
└─────────────┴────────────────┴────────────────┴────────────────┴──────────────┘
```

**Fields:**

| Field | Encoding | Description |
|-------|----------|-------------|
| **Text Ref** | varint (u32) | Index into string table for the token's text |
| **Token Type** | u8 (1 byte) | 0=word, 1=punctuation, 2=number, 3=whitespace, 4=symbol, 5=unknown |
| **Start Offset** | varint (u32) | Byte offset of token start in normalized input text |
| **End Offset** | varint (u32) | Byte offset of token end (exclusive) in normalized input text |
| **Feature Ref** | varint (u32) | Index into feature section for this token's feature bitfield (0xFFFFFFFF = no features; otherwise valid 0-based index) |

### 8.4 Delta Encoding

Start and end offsets within a token record are encoded as absolute values. However, when optimization level ≥ 1, the BytecodeGenerator MAY apply delta encoding to the start offsets of sequential tokens:

```
Token 0:  start_offset = S0, end_offset = E0
Token 1:  start_offset = delta(S0, E0) = S1 - E0   // Encoded as signed varint
Token 2:  start_offset = delta(E0, S1) = S2 - E1
...
```

The delta flag is indicated in the token type byte's high bit:
- Bit 7 = 0: Start offset is absolute.
- Bit 7 = 1: Start offset is a delta from the previous token's end offset.

### 8.5 Token Section Verification

1. All string references MUST be within bounds of the string table.
2. Feature references MUST be within bounds of the feature section (0xFFFFFFFF sentinel is also valid).
3. Start offset MUST be < end offset.
4. Token offsets MUST NOT overlap (each byte of the input belongs to exactly one token).
5. Token count MUST match the header's `token_count` field.

---

## 9. Feature Section

### 9.1 Purpose

The feature section stores packed morphological feature bitfields. Each bitfield is a 64-bit value where individual bits represent specific morphological features according to the KB-0007 taxonomy.

### 9.2 Feature Bitfield Layout (64 bits)

```
Bit    Feature            Values
───    ────────            ──────
0–3    pos (Part of Speech)  0=verb, 1=noun, 2=particle, 3=pronoun, 4=adjective,
                             5=adverb, 6=preposition, 7=conjunction, 8=proper_noun,
                             9=interrogative, 10–15=reserved
4–5    gender              0=masculine, 1=feminine, 2=common, 3=unspecified
6–7    number              0=singular, 1=dual, 2=plural, 3=unspecified
8–9    person              0=first, 1=second, 2=third, 3=unspecified
10–11  tense               0=past, 1=present, 2=imperative, 3=unspecified
12–13  mood                0=indicative, 1=subjunctive, 2=jussive, 3=energetic
14     voice               0=active, 1=passive
15–16  case                0=nominative, 1=accusative, 2=genitive, 3=unspecified
17     state               0=definite, 1=indefinite
18–22  verb_form           0=not_a_verb, 1=I, 2=II, ... 15=XV, 16–31=reserved
23–27  noun_type           0=not_a_noun, 1=masdar, 2=ism_fail, 3=ism_maful,
                           4=ism_makan, 5=ism_zaman, 6=ism_alah,
                           7=sifah, 8–31=reserved
28–31  pronoun_type        0=not_a_pronoun, 1=attached, 2=detached,
                           3=relative, 4–15=reserved
32–35  transitivity        0=unspecified, 1=intransitive, 2=transitive_1,
                           3=transitive_2, 4=ditransitive, 5–15=reserved
36–39  root_type           0=regular, 1=weak_initial, 2=weak_middle,
                           3=weak_final, 4=hamzated, 5=doubled,
                           6–15=reserved
40–47  Reserved             Must be zero
48–63  Custom/Plugin        Reserved for plugin-specific feature extensions
```

### 9.3 Packing and Unpacking

```
function pack_features(features) -> u64:
    bits = 0
    bits |= (features.pos << 0) & 0xF
    bits |= (features.gender << 4) & 0x30
    bits |= (features.number << 6) & 0xC0
    bits |= (features.person << 8) & 0x300
    bits |= (features.tense << 10) & 0xC00
    bits |= (features.mood << 12) & 0x3000
    bits |= (features.voice << 14) & 0x4000
    bits |= (features.case << 15) & 0x18000
    bits |= (features.state << 17) & 0x20000
    bits |= (features.verb_form << 18) & 0x7C0000
    bits |= (features.noun_type << 23) & 0xF800000
    bits |= (features.pronoun_type << 28) & 0xF0000000
    bits |= (features.transitivity << 32) & 0xF00000000
    bits |= (features.root_type << 36) & 0xF000000000
    // Reserved and custom bits remain zero
    return bits

function unpack_features(bits: u64) -> features:
    return {
        pos: (bits >> 0) & 0xF,
        gender: (bits >> 4) & 0x3,
        number: (bits >> 6) & 0x3,
        person: (bits >> 8) & 0x3,
        tense: (bits >> 10) & 0x3,
        mood: (bits >> 12) & 0x3,
        voice: (bits >> 14) & 0x1,
        case: (bits >> 15) & 0x3,
        state: (bits >> 17) & 0x1,
        verb_form: (bits >> 18) & 0x1F,
        noun_type: (bits >> 23) & 0x1F,
        pronoun_type: (bits >> 28) & 0xF,
        transitivity: (bits >> 32) & 0xF,
        root_type: (bits >> 36) & 0xF,
    }
```

### 9.4 Feature Section Layout

```
┌────────────────────┬──────────────────────┬──────────────────────┐
│ Feature Count      │ Feature Bitfield 0   │ Feature Bitfield 1   │
│ (varint)           │ (u64, 8 bytes)       │ (u64, 8 bytes)       │
└────────────────────┴──────────────────────┴──────────────────────┘
```

Each feature bitfield is exactly 8 bytes (64 bits), stored as a little-endian u64.

### 9.5 Feature Section Verification

1. Feature bitfields MUST NOT have reserved bits (40–47) set.
2. Custom/plugin bits (48–63) SHOULD NOT be set unless a plugin explicitly defines them.
3. The number of feature bitfields MUST match the header's `feature_count` field.

---

## 10. Constituent Section

### 10.1 Purpose

The constituent section encodes the syntactic parse tree(s) as a depth-first serialized sequence of nodes. Each node has a type, a syntactic role, feature references, and child node references.

### 10.2 Node Record Format

```
┌────────────────┬────────────────┬────────────────┬─────────────────┬──────────────┐
│ Node Type (u8) │ Role Ref       │ Feature Ref    │ Child Count      │ Children     │
│                │ (varint)       │ (varint)       │ (varint)         │ (varint[])   │
└────────────────┴────────────────┴────────────────┴─────────────────┴──────────────┘
```

**Node Type byte:**

```
Bits    Field       Values
───     ─────       ──────
0–1     node_type   0=word, 1=phrase, 2=clause, 3=root (sentence-level)
2       has_children 0=no children, 1=has children (implies child_count > 0)
3       implicit    0=explicit, 1=implicit (hadhf/ellipsis)
4–7     reserved    Must be zero
```

**Fields:**

| Field | Encoding | Description |
|-------|----------|-------------|
| **Node Type** | u8 (1 byte) | Encoded as above |
| **Role Ref** | varint (u32) | Index into string table for syntactic role (e.g., `"fi'l"`, `"fa'il"`) |
| **Feature Ref** | varint (u32) | Index into feature section (0 if no features) |
| **Child Count** | varint (u32) | Number of child nodes (0 for leaf nodes) |
| **Children** | varint[] (u32[]) | Array of child node indices (relative to this node's index) |

### 10.3 Child Index Encoding

Child indices are encoded as **relative offsets** from the current node's index, not absolute indices:

```
// Node at index 5 has children at indices 6, 7, 8
// Encoding: [1, 2, 3]  (relative: child_index - parent_index)

// Node at index 0 has children at indices 9, 10
// Encoding: [9, 10]  (absolute, since parent is at 0)
```

This encoding reduces the number of bytes needed for child references in deep trees, where children are typically adjacent to their parents.

### 10.4 Constituent Section Layout

```
┌────────────────────┬──────────────────────┬──────────────────────┐
│ Tree Count         │ Node 0               │ Node 1               │
│ (varint)           │ (variable)           │ (variable)           │
└────────────────────┴──────────────────────┴──────────────────────┘
```

The tree is serialized as a **flat array of nodes** in depth-first order. The first node (index 0) is the root of the first parse tree. If there are multiple parse trees (ambiguity), the roots are concatenated sequentially.

Each node's children must appear later in the node array (depth-first ensures this). The Child Count + Children fields reference nodes that appear later in the array.

### 10.5 Constituent Section Verification

1. Every node's children array MUST contain indices that are strictly greater than the node's own index (no backward references).
2. Every node's string reference MUST be within string table bounds.
3. Every node's feature reference MUST be within feature section bounds (or zero).
4. The number of nodes MUST match the header's `constituent_count` field.

---

## 11. Instruction Section

### 11.1 Purpose

The instruction section contains the executable bytecode instruction stream. This is the core payload that the GVM fetches, decodes, and executes.

### 11.2 Instruction Encoding

Each instruction is encoded as:

```
┌────────────┬──────────┬────────────────────────────────────┐
│ Opcode     │ Flags    │ Operands (variable, per opcode)    │
│ 1 byte     │ 1 byte   │ N bytes                            │
└────────────┴──────────┴────────────────────────────────────┘
```

**Opcode (1 byte):** 0x00–0xFF, as defined in RFC-0003 Section 5.

**Flags byte:**

```
Bit    Name                Description
───    ────                ───────────
0      HAS_IMMEDIATE       Operand contains an immediate value
1      HAS_STRING_REF      Operand references string table
2      HAS_TOKEN_REF       Operand references token index
3      HAS_REGION_REF      Operand references memory region
4–7    RESERVED            Must be zero
```

**Operands:** Variable-length, encoded per the instruction's operand type (see Section 4 — Primitive Encoding). The operand count and types for each opcode are defined in RFC-0003 Section 5.

### 11.3 Instruction Section Layout

```
┌────────────────────┬──────────────────────┬──────────────────────┐
│ Instruction Count  │ Instruction 0        │ Instruction 1        │
│ (varint)           │ (opcode + flags +    │ (opcode + flags +    │
│                    │  operands)           │  operands)           │
└────────────────────┴──────────────────────┴──────────────────────┘
```

### 11.4 Jump Offset Encoding

Jump offsets (in JUMP, JUMP_IF_TRUE, JUMP_IF_FALSE, CALL instructions) are encoded as **signed varints** (zigzag LEB128) representing the byte offset from the current instruction's start to the target instruction's start.

```
// Example: JUMP forward 20 bytes
// Current instruction starts at offset 100
// Target instruction starts at offset 120
// Offset encoding: zigzag(20) = 40 = 0x28

// Example: JUMP backward 15 bytes
// Current instruction starts at offset 200
// Target instruction starts at offset 185
// Offset encoding: zigzag(-15) = 29 = 0x1D
```

### 11.5 Instruction Section Verification

1. All opcodes MUST be known to the GVM version.
2. All operand types MUST match the instruction's operand specification.
3. All string references MUST be within string table bounds.
4. All jump offsets MUST point to valid instruction boundaries.
5. The final instruction MUST be HALT (opcode 0x00) or DIE (opcode 0x06).
6. Instruction count MUST match the header's `instruction_count` field.

---

## 12. Rule Application Section

### 12.1 Purpose

The rule application section records every grammatical rule that was applied during the pipeline's rule engine stage. This section is optional (absent if HAS_RULES flag is not set).

### 12.2 Rule Record Format

```
┌────────────────┬────────────────┬────────────────┬─────────────────┬──────────────┐
│ Rule ID Ref    │ Rule Name Ref │ School Ref     │ Action Type (u8)│ Token Count  │
│ (varint)       │ (varint)      │ (varint)       │                 │ (varint)     │
└────────────────┴────────────────┴────────────────┴─────────────────┴──────────────┘
┌─────────────────────────────────────────┐
│ Token Indices (varint[], token_count)   │
└─────────────────────────────────────────┘
```

**Action Type byte:**

```
Value   Action
─────   ──────
0       CONFIRM
1       REJECT
2       MODIFY
3       FLAG
4       RESOLVE
```

### 12.3 Rule Application Section Layout

```
┌────────────────────┬──────────────────────┬──────────────────────┐
│ Application Count  │ Application 0        │ Application 1        │
│ (varint)           │ (variable)           │ (variable)           │
└────────────────────┴──────────────────────┴──────────────────────┘
```

### 12.4 Rule Application Verification

1. All string references MUST be within string table bounds.
2. Token indices MUST be within bounds of the token section's token count.
3. The action type value MUST be 0–4.

---

## 13. Evidence Section

### 13.1 Purpose

The evidence section records the evidence trail: a complete log of every decision made during the analysis. This section is optional (absent if HAS_EVIDENCE flag is not set).

### 13.2 Evidence Entry Format

```
┌────────────────┬────────────────┬──────────────────┬────────────────┬──────────────┐
│ Stage Ref      │ Algorithm Ref  │ Confidence (f64) │ Token Count    │ Token Indices│
│ (varint)       │ (varint)       │ 8 bytes          │ (varint)       │ (varint[])   │
└────────────────┴────────────────┴──────────────────┴────────────────┴──────────────┘
```

**Fields:**

| Field | Encoding | Description |
|-------|----------|-------------|
| **Stage Ref** | varint (u32) | String table ref (e.g., `"MOD-04"`, `"MOD-07"`) |
| **Algorithm Ref** | varint (u32) | String table ref (e.g., `"basra-0103"`, `"root_extraction_v2"`) |
| **Confidence** | f64 | IEEE 754 binary64, 0.0 to 1.0 |
| **Token Count** | varint (u32) | Number of affected tokens |
| **Token Indices** | varint[] (u32[]) | Array of token indices |

### 13.3 Evidence Section Layout

```
┌────────────────────┬──────────────────────┬──────────────────────┐
│ Evidence Count     │ Entry 0              │ Entry 1              │
│ (varint)           │ (variable)           │ (variable)           │
└────────────────────┴──────────────────────┴──────────────────────┘
```

### 13.4 Evidence Section Verification

1. All string references MUST be within string table bounds.
2. Confidence values MUST be in the range [0.0, 1.0].
3. Token indices MUST be within bounds of the token section's token count.

---

## 14. End Marker

### 14.1 Format

The file MUST end with a 4-byte end marker:

```
Offset  Size    Field                Value
──────  ────    ─────                ─────
0       4       End marker           0x454E4444 ("ENDD")
```

### 14.2 Verification

1. The last 4 bytes of the file MUST equal `0x454E4444`.
2. There MUST be no bytes after the end marker.
3. The total file size (from header) MUST equal the byte position of the end marker + 4.

### 14.3 Relationship to SPEC-0001-C3

SPEC-0001 Chapter 3 Section 10.4 defines the end marker as `0x454E44` (3 bytes, "END"). RFC-0002 **refines** this to a 4-byte end marker `0x454E4444` ("ENDD") for alignment-friendly binary parsing. The 4-byte marker provides:
- Natural 4-byte alignment for the file size field (also u32).
- Improved truncation detection (a 4-byte sentinel is harder to accidentally match than 3 bytes).
- Consistency with the 4-byte magic at the start of the file.

All implementations MUST use the 4-byte `0x454E4444` marker per this RFC.

### 14.4 Purpose

The end marker serves as a **termination sentinel** that prevents truncated files from being processed. If a file is truncated during generation or transmission, the end marker will be absent or partial, and the GVM verifier will reject the bytecode.

---

## 15. Optimization Levels

### 15.1 Level 0: None

No optimizations applied. The bytecode is a direct serialization of the GIR with minimal processing:

- Strings are deduplicated (always required).
- Feature bitfields are packed normally.
- All offsets are absolute.
- All values use shortest LEB128 encoding.

**Typical size:** 100% of base (reference point).

### 15.2 Level 1: Basic

Standard optimizations:

| Optimization | Description | Size Impact |
|-------------|-------------|-------------|
| **String interning** | Strings are deduplicated within the entire bytecode (not just per-section) | 10–20% reduction |
| **Delta encoding** | Token offsets use delta encoding instead of absolute offsets | 5–10% reduction |
| **Feature deduplication** | Identical feature bitfields share a single entry | 5–15% reduction (many tokens share features) |
| **Empty string omission** | The empty string is omitted from the string table unless explicitly referenced | Negligible |
| **Dead string removal** | Strings not referenced by any section are omitted | 1–5% reduction |

**Typical size:** 70–85% of Level 0.

### 15.3 Level 2: Aggressive

All Level 1 optimizations plus:

| Optimization | Description | Size Impact |
|-------------|-------------|-------------|
| **Instruction fusion** | Common instruction sequences (LOAD_TOKEN followed by TOKEN_GET_FEATURES) are fused into single instructions | 10–20% execution time reduction |
| **Reordering** | Instructions are reordered for better GVM cache locality (hot paths first) | 5–10% execution time reduction |
| **String frequency ordering** | Strings are ordered by reference frequency (most common first) for better cache performance | Negligible size, 5–10% faster string access |
| **Constituent tree flattening** | Deep trees with single children are flattened (parent-child merge) | 5–15% reduction |

**Typical size:** 60–75% of Level 0.

**A note on determinism:** All optimizations at all levels MUST be deterministic. Given the same GIR and same configuration, the BytecodeGenerator MUST produce identical output. This means:
- String frequency ordering MUST use a stable sort.
- Instruction fusion patterns MUST be deterministic (first pattern match wins).
- Reordering MUST use a fixed algorithm, not runtime profiling.

---

## 16. Versioning & Compatibility

### 16.1 Version Scheme

The bytecode format uses Semantic Versioning (major.minor.patch):

| Component | Bump When | Example |
|-----------|-----------|---------|
| **Major** | Breaking changes (field removed, opcode removed, encoding changed) | 1.0.0 → 2.0.0 |
| **Minor** | Backward-compatible additions (new section type, new opcode, new metadata field) | 1.0.0 → 1.1.0 |
| **Patch** | Bug fixes, documentation, no encoding changes | 1.0.0 → 1.0.1 |

### 16.2 Compatibility Matrix

| Bytecode Version  | GVM Version       | Compatibility |
|-------------------|-------------------|---------------|
| 0.x (experimental) | 0.x              | May break at any time |
| 1.0               | 1.0.x            | Full compatibility |
| 1.x               | 1.y (y >= x)     | Backward compatible |
| 1.x               | 2.0+             | May break (major version mismatch) |
| 2.0               | 2.0+             | Full compatibility (new major cycle) |

### 16.3 Backward Compatibility Rules

1. A GVM version `M.x` MUST execute any bytecode version `M.y` where `y <= x`.
2. When executing older bytecode, the GVM MUST produce the same output as the older GVM version would have (determinism across versions).
3. New instructions in newer bytecode versions MUST NOT change the semantics of existing instructions.
4. Deprecated instructions MUST continue to be supported for at least 2 major versions.
5. New metadata entries MUST be optional. GVMs MUST ignore unknown metadata entries.
6. New section types MUST be optional. GVMs MUST skip unknown sections.

### 16.4 Version Upgrade Path

```
Bytecode v1.0 ──→ GVM v1.0 (native)
                     │
                     ├── GVM v1.1 (supports v1.0 + v1.1 bytecode)
                     │
                     └── GVM v2.0 (requires bytecode upgrade)
                              │
                              ├── Bytecode v2.0 (full compatibility)
                              │
                              └── Bytecode v1.0 → v2.0 converter tool
```

### 16.5 Format Evolution Principles

1. **No silent data loss.** Format changes that lose information MUST increment the major version. The old format MUST be clearly documented as deprecated.

2. **No silent semantic change.** If the same bytecode bytes would produce a different result in a newer GVM, the major version MUST be incremented.

3. **Progressive enhancement.** New features for new bytecode versions MUST NOT break old GVM's ability to parse old bytecode.

4. **Deprecation notice period.** A format feature targeted for removal MUST be marked as deprecated in a minor version at least 2 versions before removal.

---

## 17. Implementation Guidance

### 17.1 Recommended Implementation Order

1. **Implement varint encoding/decoding** — LEB128 and zigzag. These are the foundation of all other encoding. Unit-test boundary values: 0, 127, 128, 16383, 16384, u32 max, u64 max.

2. **Implement CRC32C** — Use a hardware-accelerated library (SSE 4.2 on x86, CRC extension on ARM). This is critical for verification performance.

3. **Implement the bytecode parser** — Read the header, parse the section table, verify magic/version/checksums. This is the core routine shared by GVM and diagnostics tools.

4. **Implement the string table** — Read string entries, build lookup index. This is needed before any other section can be fully parsed.

5. **Implement the token parser** — Parse tokens, resolve string references, build token region.

6. **Implement the feature parser** — Parse feature bitfields, make them accessible by index.

7. **Implement the constituent parser** — Build the constituent tree from the flat node array. Verify child references.

8. **Implement the instruction parser** — Parse the instruction stream. This is the GVM's instruction decoder.

9. **Implement the BytecodeGenerator (serialization)** — Start with Level 0 (direct mapping from GIR). Add Level 1 and Level 2 optimizations incrementally.

10. **Write the conformance test suite** — Small bytecode files with known properties, tested against both the parser and the generator.

### 17.2 Language-Specific Considerations

#### Rust (Primary Implementation)

```rust
// Example: Feature bitfield packing
#[repr(transparent)]
pub struct FeatureBitfield(u64);

impl FeatureBitfield {
    pub fn pos(&self) -> PartOfSpeech {
        PartOfSpeech::from_bits((self.0 >> 0) & 0xF)
    }

    pub fn gender(&self) -> Gender {
        Gender::from_bits((self.0 >> 4) & 0x3)
    }

    pub fn set_gender(&mut self, gender: Gender) {
        self.0 = (self.0 & !(0x3 << 4)) | ((gender.bits() as u64) << 4);
    }
}
```

#### C (Secondary Implementation)

```c
// Example: LEB128 decoding
uint64_t decode_uleb128(const uint8_t *buf, size_t *offset) {
    uint64_t result = 0;
    int shift = 0;
    while (1) {
        uint8_t byte = buf[(*offset)++];
        result |= (uint64_t)(byte & 0x7F) << shift;
        if ((byte & 0x80) == 0) break;
        shift += 7;
    }
    return result;
}
```

#### Python (Ecosystem Implementation)

```python
# Example: Feature bitfield unpacking to dict
FEATURE_MASKS = {
    'pos': (0, 0xF),
    'gender': (4, 0x3),
    'number': (6, 0x3),
    'person': (8, 0x3),
    'tense': (10, 0x3),
    'mood': (12, 0x3),
    'voice': (14, 0x1),
    'case': (15, 0x3),
    'state': (17, 0x1),
    'verb_form': (18, 0x1F),
    'noun_type': (23, 0x1F),
    'pronoun_type': (28, 0xF),
    'transitivity': (32, 0xF),
    'root_type': (36, 0xF),
}

def unpack_features(bits: int) -> dict:
    return {
        name: (bits >> shift) & mask
        for name, (shift, mask) in FEATURE_MASKS.items()
    }
```

### 17.3 End-to-End Bytecode Lifecycle

```
1. INPUT: Arabic text (string)
      │
2. PIPELINE (MOD-01 through MOD-08)
      │  Produces ResolvedGIR (JSON, ~50–200 KB for 10-word sentence)
      ▼
3. BYTECODE GENERATION (MOD-09)
      │  Reads ResolvedGIR
      │  Packs features into bitfields
      │  Builds string table (deduplication, frequency ordering)
      │  Generates instruction stream
      │  Computes CRC32C checksums per section
      │  Optimization Level 1/2 applied
      ▼
4. BYTECODE FILE (.agos, ~2–5 KB for 10-word sentence)
      │  Header + Metadata + Strings + Tokens + Features +
      │  Constituents + Instructions + Rules + Evidence + End
      │
5. Optional: VERIFICATION
      │  GVM verifier checks magic, version, checksums, structure
      ▼
6. EXECUTION (MOD-10 GVM)
      │  Reads bytecode, decodes instructions, executes
      ▼
7. OUTPUT: AnalysisResult (structured grammatical analysis)
```

### 17.4 Common Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| **Endianness mismatch** | Encode all multi-byte values as little-endian. Test on both x86 (little) and big-endian platforms (if available) using software byte-swap. |
| **Varint overflow** | LEB128 can encode values beyond 64 bits. Parse with a maximum of 10 bytes per varint. Reject values that overflow the target type. |
| **String table deduplication failure** | The BytecodeGenerator MUST guarantee no duplicate strings. An off-by-one in the deduplication hash can produce duplicates that cause GVM string resolution to find the wrong string. Use a well-tested hash set. |
| **CRC32C vs CRC32 confusion** | Use the Castagnoli polynomial (0x1EDC6F41), not the standard CRC-32 (0x04C11DB7). They produce different checksums. Hardware acceleration (SSE 4.2 CRC32) computes CRC32C. |
| **Instruction alignment** | Instructions are variable-length. The parser MUST compute the next instruction boundary correctly. Off-by-one errors here cause cascading parse failures. |
| **Constituent child reference ordering** | Children MUST appear after their parent in the flat node array. Violating this order makes tree construction impossible. |

---

## 18. Conformance Test Suite

### 18.1 Overview

Every AGOS bytecode implementation (both generators and parsers) MUST pass the AGOS Bytecode Conformance Test Suite. The test suite consists of:

- **Test bytecodes** (`.agos` files) with known binary representations.
- **Hex dumps** of small bytecodes for manual inspection.
- **Round-trip tests** (GIR → bytecode → parsed GIR → bytecode) verifying byte-for-byte equivalence.
- **Negative tests** (malformed bytecodes that must be rejected).

### 18.2 Test Categories

| Category | Tests | Coverage |
|----------|-------|----------|
| **Header** | 25 | Magic bytes, version, flags, section table, reserved bytes |
| **Varint** | 30 | LEB128 encoding/decoding boundaries, zigzag, redundant encoding |
| **String Table** | 20 | UTF-8 validation, deduplication, empty string, large strings |
| **Tokens** | 20 | Token encoding, delta offsets, string refs, feature refs |
| **Features** | 25 | Bitfield packing/unpacking, all POS values, reserved bits |
| **Constituents** | 20 | Tree serialization, child refs, depth-first ordering |
| **Instructions** | 30 | Opcode encoding, operand encoding, jump offsets |
| **Rules** | 15 | Rule record encoding, action types, token refs |
| **Evidence** | 10 | Evidence entry encoding, confidence range, token refs |
| **End Marker** | 5 | End marker presence, position, size verification |
| **Optimization** | 15 | Level 0/1/2 determinism, correctness of optimizations |
| **Negative** | 40 | Invalid headers, truncated files, bad checksums, invalid UTF-8 |
| **Round-Trip** | 20 | GIR → bytecode → parse: byte-for-byte equivalence |

**Total:** ~275 conformance tests.

### 18.3 Test Bytecode Format

Each test has:

```
# tests/conformance/header/valid_header.agos   (binary bytecode)
# tests/conformance/header/valid_header.json   (expected parse result or metadata)

{
    "test_name": "valid_header",
    "description": "Minimal valid bytecode with all required sections",
    "expected": {
        "valid": true,
        "version": { "major": 1, "minor": 0, "patch": 0 },
        "section_count": 6,
        "string_count": 0,
        "token_count": 0,
        "feature_count": 0,
        "constituent_count": 0,
        "instruction_count": 1,
        "file_size": 96
    }
}
```

### 18.4 Minimal Valid Bytecode (Hex Dump)

The smallest valid bytecode file contains only a HALT instruction and no tokens:

```
Offset  Bytes                              Description
──────  ─────                              ───────────
0x00    41 47 4F 53                        Magic: "AGOS"
0x04    01 00                              Version major: 1
0x06    00 00                              Version minor: 0
0x08    00 00                              Version patch: 0
0x0A    20 00                              Flags: 0x0020 (LARGE_OFFSETS set; no evidence, no rules, no opt)
0x0C    8A 00 00 00                        Total file size: 138 bytes
0x10    06 00                              Section count: 6
0x12    00 00                              String count: 0
0x14    00 00                              Token count: 0
0x16    00 00                              Feature count: 0
0x18    00 00                              Constituent count: 0
0x1A    01 00 00 00                        Instruction count: 1
0x1E    00 00                              Reserved
0x20                                    Section table (6 entries × 10 bytes):
0x20    01 00 5C 00 00 00 0C 00 00 00    Section 1 (METADATA): offset=0x5C, size=12
0x2A    02 00 68 00 00 00 02 00 00 00    Section 2 (STRINGS): offset=0x68, size=2
0x34    03 00 6A 00 00 00 02 00 00 00    Section 3 (TOKENS): offset=0x6A, size=2
0x3E    04 00 6C 00 00 00 02 00 00 00    Section 4 (FEATURES): offset=0x6C, size=2
0x48    05 00 6E 00 00 00 02 00 00 00    Section 5 (CONSTITUENTS): offset=0x6E, size=2
0x52    06 00 70 00 00 00 16 00 00 00    Section 6 (INSTRUCTIONS): offset=0x70, size=22
0x5C                                    --- Section data ---
0x5C                                     METADATA section (12 bytes):
0x5C    07                                Entry count: 7
0x5D    04 73 70 65 63                    Key: "spec" (4 bytes)
        01 00                              Type: integer(1)
        00                                 Value: 0 (SPEC-0001 enum index)
0x64    05 76 65 72 73 69 6F 6E          Key: "version" (5 bytes)
        00                                 Type: string
        05 31 2E 30 2E 30                  Value: "1.0.0"
...                                    (remaining metadata entries truncated)
0x68                                     STRINGS section (2 bytes):
0x68    00                                 Entry count: 0
0x69                                     
0x6A                                     TOKENS section (2 bytes):
0x6A    00                                 Token count: 0
0x6B                                     
0x6C                                     FEATURES section (2 bytes):
0x6C    00                                 Feature count: 0
0x6D                                     
0x6E                                     CONSTITUENTS section (2 bytes):
0x6E    00                                 Tree count: 0
0x6F                                     
0x70                                     INSTRUCTIONS section (22 bytes):
0x70    01                                 Instruction count: 1
0x71    00 00                              Instruction 0: HALT (opcode 0x00, flags 0x00)
0x73    00 00 00 00 00 00 00 00           Reserved (padding to 22 bytes)
        00 00 00 00 00 00 00 00
        00 00 00 00 00 00 00 00
0x86                                     End marker:
0x86    45 4E 44 44                        "ENDD" (0x454E4444)
```

### 18.5 Test Runner

```bash
# Run all conformance tests
agos bytecode test --suite=conformance-v1

# Run specific category
agos bytecode test --suite=conformance-v1 --category=feature

# Round-trip test: GIR JSON → bytecode → parse → compare
agos bytecode roundtrip --gir=sample.json --output=roundtrip.agos

# Verify a bytecode file
agos bytecode verify --file=analysis.agos

# Dump bytecode structure as human-readable JSON
agos bytecode dump --file=analysis.agos --format=json

# Compute bytecode statistics
agos bytecode stats --file=analysis.agos

# Compare two bytecode files (structural diff)
agos bytecode diff --file1=a.agos --file2=b.agos
```

---

## 19. Cross-References

### 19.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| ADR-0002 | Why Grammar Bytecode | Architectural rationale for the bytecode format |
| SPEC-0001-C3 (MOD-09) | BytecodeGenerator | Produces bytecode in this format |
| SPEC-0001-C3 (MOD-10) | GVM | Consumes bytecode in this format |
| SPEC-0001-C4 | Module Interfaces | Formal interface for bytecode generation and consumption |
| SPEC-0001-C5 (IR-9) | GrammarBytecode | Logical schema mapped to binary format |
| SPEC-0001-C9 | Performance Targets | Size targets: < 10 KB/sentence, < 20% of JSON |
| RFC-0003 | Grammar Virtual Machine | Instruction encoding and execution model |
| KB-0007 | Morphological Features Taxonomy | Feature bitfield layout based on KB-0007 |

### 19.2 External References

| Reference | Relevance |
|-----------|-----------|
| LEB128 Encoding (DWARF Debugging Standard) | Variable-length integer encoding used for operands and counts |
| CRC32C (Castagnoli) | Error detection; hardware-accelerated on SSE 4.2 and ARM |
| IEEE 754 | Floating-point representation for confidence values |
| WebAssembly Binary Format | Inspiration for section-based binary layout and versioning |
| JVM Class File Format | Inspiration for magic bytes, versioning, and constant pool (string table) |
| Protocol Buffers (varint encoding) | Reference for LEB128 implementation patterns |

---

## Progress Summary

**RFC-0002: Grammar Bytecode Format**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction | ✓ COMPLETE |
| 2 | Design Principles | ✓ COMPLETE (6 principles, byte-level conventions) |
| 3 | File Format Overview | ✓ COMPLETE (8 section types, ordering, constraints) |
| 4 | Primitive Encoding | ✓ COMPLETE (varint, zigzag, fixed, float, string) |
| 5 | Header Section | ✓ COMPLETE (fixed-size layout, flags bitmask, section table) |
| 6 | Metadata Section | ✓ COMPLETE (key-value entries, required/optional fields) |
| 7 | String Table Section | ✓ COMPLETE (deduplication, frequency ordering, capacity) |
| 8 | Token Section | ✓ COMPLETE (record format, delta encoding, verification) |
| 9 | Feature Section | ✓ COMPLETE (64-bit bitfield layout, pack/unpack algorithms) |
| 10 | Constituent Section | ✓ COMPLETE (node format, child index encoding, tree serialization) |
| 11 | Instruction Section | ✓ COMPLETE (opcode/flags/operands, jump offsets) |
| 12 | Rule Application Section | ✓ COMPLETE (rule record format, action types) |
| 13 | Evidence Section | ✓ COMPLETE (evidence entry format, confidence encoding) |
| 14 | End Marker | ✓ COMPLETE (sentinel, verification, truncation detection) |
| 15 | Optimization Levels | ✓ COMPLETE (Level 0/1/2 with size impacts) |
| 16 | Versioning & Compatibility | ✓ COMPLETE (semver, compatibility matrix, upgrade path) |
| 17 | Implementation Guidance | ✓ COMPLETE (10-step order, language-specific examples, pitfalls) |
| 18 | Conformance Test Suite | ✓ COMPLETE (~275 tests, hex dump, CLI commands) |

---

**Dependencies:** ADR-0002, RFC-0003 (for instruction encoding), SPEC-0001-C3/C4/C5/C9, KB-0007.

**Recommended next document:** ADR-0003 — Why Grammar Intermediate Representation (GIR).
