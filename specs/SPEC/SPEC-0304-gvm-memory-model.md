# SPEC-0304: GVM Memory Model — Memory Arena Specification

| **Field** | **Value** |
|---|---|
| **Spec ID** | SPEC-0304 |
| **Title** | GVM Memory Model — Memory Arena Specification |
| **Version** | 1.0.0 |
| **Status** | Draft |
| **Depends on** | RFC-0003 (GVM Architecture), RFC-0002 (Bytecode Format), SPEC-0302 (Instruction Set), SPEC-0301 (Grammar Runtime) |
| **Related SPECs** | SPEC-0303 (Implementation Guide), SPEC-0102 (Feature Taxonomy), SPEC-0001-C9 (Performance Targets) |
| **License** | AGOS Specification License v1.0 |

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Region Type Definitions & Layouts](#3-region-type-definitions--layouts)
4. [Region Lifecycle](#4-region-lifecycle)
5. [Bump Allocator Model](#5-bump-allocator-model)
6. [Token Region](#6-token-region)
7. [Feature Region](#7-feature-region)
8. [Constituent Region](#8-constituent-region)
9. [String Region](#9-string-region)
10. [Rule Region](#10-rule-region)
11. [Evidence Region](#11-evidence-region)
12. [Scratch Buffer](#12-scratch-buffer)
13. [Operand Stack](#13-operand-stack)
14. [Call Stack](#14-call-stack)
15. [Memory Budget Model](#15-memory-budget-model)
16. [Bounds Checking & Safety Guarantees](#16-bounds-checking--safety-guarantees)
17. [Cross-Language Porting Reference](#17-cross-language-porting-reference)
18. [Performance Considerations](#18-performance-considerations)
19. [Testing & Verification](#19-testing--verification)
20. [Cross-References](#20-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

This specification defines the **GVM Memory Model** — the detailed memory arena architecture used by the Grammar Virtual Machine (MOD-10). The GVM uses a **pre-allocated typed region model** where all memory is reserved at instance creation time and accessed through typed indices with mandatory bounds checking.

The memory model is designed around five core principles:

1. **Zero dynamic allocation during execution.** All memory regions are pre-allocated at GVM instance initialization. "Allocation" within a region is merely incrementing a bump pointer.

2. **No pointer arithmetic.** All memory access is through typed `u32` indices that are bounds-checked on every operation. There is no way to construct an arbitrary memory address.

3. **Deterministic layout.** The same bytecode header and GVM configuration produce byte-for-byte identical memory layouts across all implementations.

4. **Predictable performance.** With no garbage collection, no malloc/free during execution, and no cache-unfriendly pointer chasing, the memory model delivers consistent, predictable latency.

5. **Language-independent semantics.** The memory model is specified abstractly enough to be implemented in any language (Rust, C, Python, TypeScript, Go, Java, Swift) while preserving identical runtime behavior.

### 1.2 Scope

**In scope:**

| Category | Coverage |
|----------|----------|
| **Region architecture** | Full region hierarchy, type parameters, capacity model |
| **Per-region layout** | Memory layout of Token, FeatureBitfield, ConstituentNode, RuleRecord, EvidenceRecord, Stack values |
| **Bump allocator** | Allocation algorithm within each region, capacity enforcement, alignment |
| **String table** | Lazy/eager decoding trade-offs, offset-based indexing, interned string model |
| **Stacks** | Operand stack (Value type layout, max depth, overflow handling), call stack |
| **Scratch buffer** | Raw byte buffer, cursor-based allocation, overflow semantics |
| **Budget calculation** | Per-region sizing formulas, total memory budget, max memory enforcement |
| **Bounds checking** | Universal bounds check pattern, error codes, performance impact |
| **Safety model** | No-dynamic-allocation invariant, no-pointer-arithmetic invariant, type safety |
| **Porting reference** | Per-language region implementation patterns (Rust, C, Python, TS, Go, Java) |
| **Testing** | Memory model conformance tests, leak detection, bounds check coverage |

**Out of scope:**

| Topic | Covered By |
|-------|-----------|
| Instruction encoding and semantics | SPEC-0302 |
| Bytecode binary format | RFC-0002 |
| GVM execution loop and dispatch | RFC-0003 §7, SPEC-0301 §5 |
| Feature bitfield taxonomy | SPEC-0102, KB-0007 |
| GVM instance lifecycle and pooling | SPEC-0301 §4 |
| GVM implementation guide | SPEC-0303 |

### 1.3 Relationship to Source Documents

| Reference | Provides | Use in Memory Model |
|-----------|----------|---------------------|
| **RFC-0003 §6** | High-level memory region overview | Foundation — defines 7 regions and default capacities |
| **RFC-0003 §3** | GVMState structure | Region types within state |
| **RFC-0002 §7–13** | Section data formats | Determines how regions are populated from bytecode |
| **SPEC-0301 §6** | Runtime memory management | Capacity tuning, safety model, budget calculation |
| **SPEC-0302 §14** | Per-instruction memory impact | Memory cost per operation |
| **SPEC-0303 §6** | Implementation patterns | Region structs, initialization & sizing, bounds checking |
| **SPEC-0001-C9** | Performance targets | Latency, memory, throughput goals |

---

## 2. Architecture Overview

### 2.1 Region Hierarchy

The GVM memory model consists of **8 memory regions** plus **2 stacks**, organized by access pattern and lifetime:

```
┌──────────────────────────────────────────────────────────────────┐
│                     GVM Memory Instance                           │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    IMMUTABLE REGIONS                         │  │
│  │  (populated once from bytecode, read-only during execution) │  │
│  │                                                             │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐    │  │
│  │  │ String Region│  │ Token Region │  │ Feature Region │    │  │
│  │  │ (indexed by  │  │ (indexed by  │  │ (indexed by    │    │  │
│  │  │  string_idx) │  │  token_idx)  │  │  feature_idx)  │    │  │
│  │  └──────────────┘  └──────────────┘  └────────────────┘    │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    MUTABLE REGIONS                           │  │
│  │  (written during execution, grow monotonically)             │  │
│  │                                                             │  │
│  │  ┌──────────────────┐  ┌──────────────┐  ┌───────────────┐  │  │
│  │  │ Constituent Reg. │  │  Rule Region │  │ Evidence Reg. │  │  │
│  │  │ (tree nodes)     │  │  (rule apps) │  │ (evidence)    │  │  │
│  │  └──────────────────┘  └──────────────┘  └───────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    STACKS & BUFFERS                          │  │
│  │  (read-write, push/pop or cursor-based)                    │  │
│  │                                                             │  │
│  │  ┌────────────────┐  ┌────────────┐  ┌──────────────────┐  │  │
│  │  │ Operand Stack  │  │ Call Stack │  │ Scratch Buffer   │  │  │
│  │  │ (Value array)  │  │ (u32 addrs)│  │ (raw byte array) │  │  │
│  │  └────────────────┘  └────────────┘  └──────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 Capacity Model

Every region has two capacity parameters:

| Parameter | Source | Description |
|-----------|--------|-------------|
| **Declared capacity** | Bytecode header field | The minimum capacity needed for this bytecode. Derived from the bytecode generator's analysis of the input. |
| **Default capacity** | GVM configuration constant | A fallback minimum capacity used when the declared capacity is unreasonably small (e.g., zero). Ensures that even minimal bytecodes have enough workspace. |
| **Effective capacity** | `max(declared, default)` | The actual capacity allocated at initialization. |

```
effective_capacity = max(declared_capacity, DEFAULT_CAPACITY)
```

This model ensures:
- **Correctness:** The bytecode's declared capacities are always sufficient.
- **Robustness:** Even if declared capacity is 0 (edge case), the GVM doesn't fail on simple operations.
- **Determinism:** Same bytecode + same config = same effective capacities.

### 2.3 Default Capacities

These constants MUST be identical across all GVM implementations:

| Region | Default Capacity | Per-Entry Size | Default Total | Notes |
|--------|-----------------|---------------|---------------|-------|
| Token | 256 | 48 bytes | 12 KB | Enough for ~50-word sentence with clitics |
| Feature | 512 | 8 bytes | 4 KB | 64-bit bitfields; multiple analyses per token |
| Constituent | 1024 | 48 bytes | 48 KB | Complex parse trees |
| String | (from bytecode) | Variable | ~2–10 KB (in bytecode) | Immutable; loaded at init |
| Rule | 512 | 32 bytes | 16 KB | Rule application records |
| Evidence | 1024 | 64 bytes | 64 KB | Full evidence trail |
| Scratch | 4096 | 1 byte | 4 KB | Raw byte workspace |
| Operand Stack | 1024 | 16 bytes | 16 KB | Value entries |
| Call Stack | 64 | 4 bytes | 256 B | Return addresses |

**Default total memory per GVM instance:** ~162 KB (all regions at default capacities).

### 2.4 Memory Access Model

```
┌─────────────────────────────────────────────────────────────┐
│                  Memory Access Rules                          │
│                                                              │
│  Every memory access follows this pattern:                   │
│                                                              │
│  1. RECEIVE typed index (u32)                                │
│     - token_index, feature_idx, constituent_ptr,             │
│       string_index, rule_index, evidence_index               │
│                                                              │
│  2. VALIDATE bounds: index < region.size                    │
│     - If OOB → return GVMError::IndexOutOfBounds            │
│                                                              │
│  3. ACCESS region.data[index]                                │
│     - O(1) direct indexing (no pointer chasing)              │
│                                                              │
│  4. RETURN result or MODIFY in place                         │
│     - Immutable regions: read-only access                    │
│     - Mutable regions: read-write access                     │
│     - Stacks: push/pop/peek only                             │
└─────────────────────────────────────────────────────────────┘
```

### 2.5 Allocator Model Summary

| Region | Allocator | Growth Pattern | Used By |
|--------|-----------|---------------|---------|
| Token | Population at load | Bulk from bytecode section | LOAD_TOKEN at init |
| Feature | Population at load | Bulk from bytecode section | TOKEN_GET_FEATURES at init |
| Constituent | Bump allocator | Monotonic growth | CONST_MAKE during execution |
| Rule | Bump allocator | Monotonic growth | RULE_APPLY during execution |
| Evidence | Bump allocator | Monotonic growth | EVIDENCE_PUSH during execution |
| String | Loaded at init | Immutable | PUSH_STRING references |
| Scratch | Cursor-based | Sequential writes | Scratch operations |
| Operand Stack | Push/Pop | Full dynamic range | All instruction handlers |
| Call Stack | Push/Pop | Call/return only | CALL/RETURN |

---

## 3. Region Type Definitions & Layouts

### 3.1 Abstract Region Type

Every region is an instance of the following parameterized type:

```
Region<T> {
    // ── Metadata ──
    name: String,                          // Human-readable region name
    entry_size: u32,                       // Size of T in bytes
    capacity: u32,                         // Maximum number of T entries

    // ── Storage ──
    data: [T; capacity],                   // Contiguous array of T
    size: u32,                             // Current number of valid entries (0..capacity)

    // ── Allocation state ──
    bump: u32,                             // Next free slot index (= size)
    // Mutable regions only:
    last_alloc_index: Option<u32>,         // Index of most recent allocation
}
```

**Invariants:**
- `0 ≤ size ≤ capacity` — size never exceeds capacity.
- `size == bump` — the bump pointer always equals the count of allocated entries.
- `data[0..size]` — valid entries. `data[size..capacity]` — uninitialized (must not be read before write).

### 3.2 Token Layout

The `Token` struct stores all data from a single token in the bytecode token section:

```
Token (48 bytes on 8-byte boundary):
┌─────────┬──────────┬──────────────┬────────────┬──────────────────┐
│ text_   │ token_   │ start_offset │ end_offset │ feature_index    │
│ index   │ type     │ (u32, 4 B)   │ (u32, 4 B) │ (u32, 4 B)       │
│ (u32)   │ (u8, 1B) │              │            │                  │
├─────────┤          │              │            │                  │
│ 8 bytes │ 1 byte   │ 4 bytes      │ 4 bytes    │ 4 bytes          │
└─────────┴──────────┴──────────────┴────────────┴──────────────────┘
├─────────────────── 21 bytes data ───────────────────┤
├─────────────────── 48 bytes allocated ──────────────┤
                    (padding to 8-byte alignment)                     │
```

**Fields:**

| Field | Type | Bytes | Description |
|-------|------|-------|-------------|
| `text_index` | u32 | 4 | Index into string table for token text |
| `token_type` | u8 | 1 | 0=word, 1=punctuation, 2=number, 3=whitespace, 4=symbol, 5=unknown |
| `_padding1` | u8[3] | 3 | Alignment padding (to 8-byte boundary) |
| `start_offset` | u32 | 4 | Byte offset of token start in normalized input text |
| `end_offset` | u32 | 4 | Byte offset of token end (exclusive) |
| `feature_index` | u32 | 4 | Index into feature region (or `0xFFFFFFFF` for no features) |

**Total:** 20 bytes data → 24 bytes with 8-byte alignment padding → **48 bytes allocated** (padded to 48 to provide per-entry cache line isolation and room for future fields; each entry occupies ¾ of a 64-byte cache line regardless of base alignment).

### 3.3 Feature Bitfield Layout

A feature entry is a single 64-bit value:

```
FeatureEntry (8 bytes):
┌────────┬────────┬────────┬────────┬────────┬────────┬────────┬────────┐
│ byte 0 │ byte 1 │ byte 2 │ byte 3 │ byte 4 │ byte 5 │ byte 6 │ byte 7 │
└────────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘
├────────────── 8 bytes (u64, little-endian) ──────────────────────────┤
```

The bitfield layout follows KB-0007 / SPEC-0102 (see SPEC-0302 §16.1 for the complete mapping):

| Bits | Feature | Width |
|------|---------|-------|
| 0–3 | pos | 4 bits |
| 4–5 | gender | 2 bits |
| 6–7 | number | 2 bits |
| 8–9 | person | 2 bits |
| 10–11 | tense | 2 bits |
| 12–13 | mood | 2 bits |
| 14 | voice | 1 bit |
| 15–16 | case | 2 bits |
| 17 | state | 1 bit |
| 18–22 | verb_form | 5 bits |
| 23–27 | noun_type | 5 bits |
| 28–31 | pronoun_type | 4 bits |
| 32–35 | transitivity | 4 bits |
| 36–39 | root_type | 4 bits |
| 40–42 | stress_pattern | 3 bits |
| 43–46 | syllable_count | 4 bits |
| 47 | has_shadda | 1 bit |
| 48 | has_madd | 1 bit |
| 49 | has_hamza | 1 bit |
| 50–63 | reserved/plugin | 14 bits |

### 3.4 Constituent Node Layout

```
ConstituentNode (48 bytes):
┌──────────┬──────────────┬──────────────┬────────────────┐
│ role_id  │ child_count  │ token_count  │ flags (u8, 1B) │
│ (u32)    │ (u32)        │ (u32)        │                │
│ 4 bytes  │ 4 bytes      │ 4 bytes      │ 1 byte         │
├──────────┴──────────────┴──────────────┴────────────────┤
│ (continued)                                              │
├─────────────────── 48 bytes total ──────────────────────┤
```

**Fields:**

| Field | Type | Bytes | Description |
|-------|------|-------|-------------|
| `role_id` | u32 | 4 | Syntactic role ID (e.g., fi'l=10, fa'il=11) |
| `child_count` | u32 | 4 | Number of child constituents |
| `children` | u32[] | 4 × child_count | Child constituent indices (absolute) |
| `token_count` | u32 | 4 | Number of attached token indices |
| `token_indices` | u32[] | 4 × token_count | Attached token indices |
| `flags` | u8 | 1 | Bit flags: bit 0=implicit, bit 1=ambiguous |

**Size is variable:** `16 + 4 × child_count + 4 × token_count` bytes (with padding to 8-byte boundary). The struct uses heap-allocated slices (`Vec<u32>` in Rust) to accommodate a variable number of children and token indices.

**Typical allocation:** ~48 bytes (average case: ~4 children + ~4 tokens per node). Simple leaf nodes with 0 children and 1 token consume ~24 bytes. Complex nodes with 8 children and 8 tokens consume ~80 bytes. The constituent region entry size is dynamically computed per allocation.

### 3.5 Rule Record Layout

```
RuleRecord (32 bytes):
┌──────────┬────────────────┬──────────┬────────────┬──────────┐
│ rule_id  │ rule_name_     │ token_   │ pc_at_     │ modified │
│ (u32)    │ index (u32)    │ index    │ apply (u32)│ count    │
│ 4 bytes  │ 4 bytes        │ (u32)    │ 4 bytes    │ (u32)    │
│          │                │ 4 bytes │            │ 4 bytes  │
├──────────┴────────────────┴──────────┴────────────┴──────────┤
│ modifications: [(feature_id, old_val, new_val); mod_count]   │
├─────────────────── 32 bytes total ──────────────────────────┤
```

**Fields:**

| Field | Type | Bytes | Description |
|-------|------|-------|-------------|
| `rule_id` | u32 | 4 | Numeric rule identifier |
| `rule_name_index` | u32 | 4 | Index into string table for rule name |
| `token_index` | u32 | 4 | Token index active when rule applied |
| `pc_at_apply` | u32 | 4 | Program counter at time of application |
| `mod_count` | u32 | 4 | Number of feature modifications recorded |
| `modifications` | (u8,u8,u8)[] | 3 × mod_count | Feature ID, old value, new value |

### 3.6 Evidence Record Layout

```
EvidenceRecord (64 bytes):
┌───────────────┬───────────────┬──────────────┬──────────┐
│ stage_name_   │ algorithm_    │ confidence   │ token_   │
│ index (u32)   │ index (u32)   │ (f64, 8 B)   │ index    │
│ 4 bytes       │ 4 bytes       │              │ (u32)    │
│               │               │              │ 4 bytes  │
├───────────────┴───────────────┴──────────────┴──────────┤
│ step (u64, 8 B) / timestamp / reserved (28 B)          │
├─────────────────── 64 bytes total ─────────────────────┤
```

**Fields:**

| Field | Type | Bytes | Description |
|-------|------|-------|-------------|
| `stage_name_index` | u32 | 4 | String table index for stage (e.g., "MOD-04") |
| `algorithm_index` | u32 | 4 | String table index for algorithm |
| `confidence` | f64 | 8 | Confidence value [0.0, 1.0] |
| `token_index` | u32 | 4 | Token index at time of evidence |
| `step` | u64 | 8 | Step count at time of evidence |
| `reserved` | u8[36] | 36 | Reserved for future fields |

### 3.7 Value Type Layout (Operand Stack)

```
Value (16 bytes):
┌────────────┬──────────────────────────────────────────┐
│ type_tag   │ data (15 bytes)                           │
│ (u8, 1 B)  │                                          │
├────────────┴──────────────────────────────────────────┤
├─────────────────── 16 bytes total ────────────────────┤
```

**Type tags:**

| Tag | Type | Data Interpretation |
|-----|------|-------------------|
| `0x00` | I32 | `data[0..3]` as little-endian i32 |
| `0x01` | I64 | `data[0..7]` as little-endian i64 |
| `0x02` | F64 | `data[0..7]` as IEEE 754 binary64 |
| `0x03` | StringIndex | `data[0..3]` as u32 string table index |
| `0x04` | TokenIndex | `data[0..3]` as u32 token index |
| `0x05` | FeatureBits | `data[0..7]` as u64 bitfield |
| `0x06` | ConstituentPtr | `data[0..3]` as u32 constituent index |
| `0x07` | Bool | `data[0]` as u8 (0=false, 1=true) |
| `0x08` | Null | Data unused |
| `0x09–0xFF` | Reserved | MUST NOT occur |

### 3.8 Call Stack Entry Layout

```
CallStackEntry (4 bytes):
┌────────────────────────────────────────────────────────┐
│ return_address (u32, little-endian)                     │
├─────────────────── 4 bytes total ──────────────────────┤
```

The call stack stores return addresses as absolute byte offsets into the instruction stream. Each entry is a simple u32.

---

## 4. Region Lifecycle

### 4.1 Lifecycle States

Every region goes through the following states:

```
┌───────────┐     ┌───────────┐     ┌───────────┐     ┌───────────┐
│ UNINITIAL- │────►│ INITIAL-  │────►│ ACTIVE    │────►│ DESTROYED │
│ IZED       │     │ IZED      │     │           │     │           │
└───────────┘     └───────────┘     └───────────┘     └───────────┘
                        │                 │
                        │                 ▼
                        │            ┌──────────┐
                        └───────────►│  RESET   │
                                     │ (reuse)  │
                                     └──────────┘
```

| State | Description | Transitions |
|-------|-------------|-------------|
| **UNINITIALIZED** | Region memory not yet allocated | → INITIALIZED on `init()` |
| **INITIALIZED** | Memory allocated, `size=0`, `bump=0` | → ACTIVE on first allocation, → RESET on `clear()` |
| **ACTIVE** | Region is being read/written during execution | → RESET on `clear()`, → DESTROYED on `destroy()` |
| **RESET** | `size=0`, `bump=0`, data not zeroed (for performance) | → ACTIVE on next allocation |
| **DESTROYED** | Memory freed, region unusable | Terminal state |

### 4.2 Lifecycle in Context of GVM Instance

```
GVM Instance Lifecycle:
┌─────────────────────────────────────────────────────┐
│ 1. CREATE                                            │
│    pool.acquire() → GVMInstance                     │
│    • All memory regions are UNINITIALIZED            │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ 2. INITIALIZE REGIONS                                │
│    gvm_init_regions(header)                         │
│    • For each region: allocate data[capacity]        │
│    • size = 0, bump = 0                             │
│    • Populate immutable regions from bytecode:       │
│      - String region: load string table              │
│      - Token region: parse and load tokens           │
│      - Feature region: parse and load bitfields      │
│    • All regions → INITIALIZED                       │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ 3. EXECUTE                                           │
│    gvm_run(state, bytecode)                         │
│    • Regions → ACTIVE                                │
│    • Mutable regions grow via bump allocation        │
│    • Stacks grow/shrink via push/pop                 │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ 4. COLLECT OUTPUT                                    │
│    gvm_assemble_output(state) → AnalysisResult      │
│    • Read final state from all regions               │
│    • Regions still ACTIVE (not modified)             │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ 5. RESET & RELEASE (if pooling)                      │
│    gvm_reset_regions(state)                         │
│    • Set size = 0, bump = 0 for mutable regions      │
│    • Immutable regions: unchanged                    │
│      (they hold bytecode data; reused as-is)         │
│    • Stacks: clear to empty                         │
│    • Regions → RESET                                 │
│    pool.release(instance)                           │
└─────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────┐
│ 6. DESTROY (if pool full or shutdown)                │
│    gvm_destroy_regions(state)                       │
│    • Free all region data                            │
│    • Regions → DESTROYED                             │
└─────────────────────────────────────────────────────┘
```

### 4.3 Reset Semantics

The `reset()` operation is critical for instance pooling — it clears all mutable state without deallocating memory:

```rust
/// Reset all mutable regions for instance reuse.
/// Immutable regions (string, token, feature) are NOT cleared —
/// they are reused as-is for the next execution with the same bytecode.
fn reset_regions(state: &mut GVMState) {
    // Mutable regions: reset bump pointer
    state.constituent_region.clear();     // size = 0, bump = 0
    state.rule_region.clear();            // size = 0, bump = 0
    state.evidence_region.clear();        // size = 0, bump = 0
    state.scratch_region.clear();         // cursor = 0

    // Stacks: clear to empty
    state.operand_stack.clear();          // len = 0
    state.call_stack.clear();             // len = 0

    // Execution state
    state.pc = 0;
    state.halted = false;
    state.error = None;
    state.step_count = 0;
    state.current_token_index = -1;
    state.last_rule_index = None;
    state.output_finalized = false;
    state.output_metadata.clear();
    state.output_trees.clear();
    state.output_flags.clear();
    state.anaphora_resolutions.clear();
}
```

**Performance note:** Regions are NOT zeroed on reset. The old data remains in memory but is logically inaccessible because `size` is set to 0. This is safe because:
- All reads are bounds-checked against `size`.
- All writes overwrite old data before any read.
- No deallocation or free occurs.

---

## 5. Bump Allocator Model

### 5.1 Allocation Algorithm

All mutable regions (constituent, rule, evidence, scratch) use the same bump allocation algorithm:

```rust
/// Bump-allocate a new entry of type T in the region.
/// Returns the index of the newly allocated entry.
fn bump_alloc<T>(region: &mut Region<T>, value: T) -> Result<u32, GVMError> {
    // 1. Check capacity
    if region.size >= region.capacity {
        return Err(GVMError::RegionFull {
            region: region.name.clone(),
            size: region.size,
            capacity: region.capacity,
        });
    }

    // 2. Write value at current bump position
    let index = region.bump;
    region.data[index as usize] = value;

    // 3. Advance bump pointer
    region.size += 1;
    region.bump += 1;

    // 4. Return allocated index
    Ok(index)
}
```

### 5.2 Allocation Guarantees

| Property | Guarantee |
|----------|-----------|
| **Time complexity** | O(1) — constant time, no search |
| **Fragmentation** | Zero — regions are perfectly compact |
| **Alignment** | Natural alignment of T (8 bytes for u64/f64, 4 bytes for u32) |
| **Failure mode** | Deterministic error (`REGION_FULL`) — no OOM, no undefined behavior |
| **Concurrent access** | Not supported — single-threaded execution |
| **Rollback** | Not supported — allocations are monotonic |

### 5.3 Monotonic Growth Invariant

```
size(t+1) ≥ size(t) for all mutable regions during execution.
```

This invariant holds because:
- There are NO deallocation operations during execution.
- There is NO `free()` or `remove()` operation on regions.
- The only way to reduce `size` is via `reset()` between executions.

**Rationale:** This invariant eliminates use-after-free bugs, simplifies bounds checking (size only increases), and enables the bump allocator's O(1) performance.

### 5.4 Capacity Overrun Detection

If a bump allocation would exceed capacity, the GVM MUST:

1. **NOT** attempt to reallocate or grow the region.
2. **NOT** silently wrap around or overwrite existing data.
3. Return `GVMError::RegionFull` with the region name, current size, and capacity.
4. Halt execution (all region errors are fatal).

```rust
// Example: CONST_MAKE capacity check
fn handle_const_make(/* ... */) -> Result<(), GVMError> {
    if state.constituent_region.size >= state.constituent_region.capacity {
        return Err(GVMError::RegionFull {
            region: "constituent".into(),
            size: state.constituent_region.size,
            capacity: state.constituent_region.capacity,
        });
    }
    // ... allocate ...
}
```

---

## 6. Token Region

### 6.1 Purpose

The token region stores all parsed token data from the bytecode token section. It is **populated once at initialization** and is **read-only during execution**.

### 6.2 Initialization

```rust
fn populate_token_region(
    state: &mut GVMState,
    bytecode: &ParsedBytecode,
) -> Result<(), GVMError> {
    let count = bytecode.header.token_count as u32;
    let capacity = count.max(DEFAULT_TOKENS);

    state.token_region = Region::new("token", capacity);

    for i in 0..count {
        let token_data = &bytecode.tokens[i as usize];
        let token = Token {
            text_index: token_data.text_index,
            token_type: token_data.token_type,
            start_offset: token_data.start_offset,
            end_offset: token_data.end_offset,
            feature_index: token_data.feature_index,
        };
        state.token_region.bump_alloc(token)?;
    }

    Ok(())
}
```

### 6.3 Access Patterns

| Operation | Access Type | Method |
|-----------|-------------|--------|
| LOAD_TOKEN (0x20) | Read by index | `token_region.get(index)` |
| TOKEN_GET_TEXT (0x21) | Read field | `token.text_index` |
| TOKEN_GET_OFFSET (0x22) | Read fields | `token.start_offset`, `token.end_offset` |
| TOKEN_GET_TYPE (0x23) | Read field | `token.token_type` |
| TOKEN_GET_FEATURES (0x24) | Read via feature_index | `feature_region.get(token.feature_index)` |
| TOKEN_ITERATE (0x25) | Sequential scan | Index `0..size` |
| TOKEN_COUNT (0x26) | Size query | `token_region.size` |

### 6.4 Memory Layout

```
Token Region:
┌─────────┬─────────┬─────────┬─────┬─────────┐
│ Token 0 │ Token 1 │ Token 2 │ ... │ Token   │
│         │         │         │     │ N-1     │
│ 48 B    │ 48 B    │ 48 B    │     │ 48 B    │
└─────────┴─────────┴─────────┴─────┴─────────┘
├─────────────────── N × 48 bytes ─────────────────┤
```

The token region is a flat, contiguous array of `Token` structs.

---

## 7. Feature Region

### 7.1 Purpose

The feature region stores packed 64-bit morphological feature bitfields. Each entry corresponds to one token's complete feature set (POS, gender, number, person, tense, mood, voice, case, state, etc.).

### 7.2 Initialization

```rust
fn populate_feature_region(
    state: &mut GVMState,
    bytecode: &ParsedBytecode,
) -> Result<(), GVMError> {
    let count = bytecode.header.feature_count as u32;
    let capacity = count.max(DEFAULT_FEATURES);

    state.feature_region = Region::new("feature", capacity);

    for i in 0..count {
        let bits = bytecode.features[i as usize];
        // Validate: reserved bits (40–47) must be zero
        if bits & 0x0000FF0000000000 != 0 {
            return Err(GVMError::BytecodeCorrupted {
                issues: vec![VerificationIssue::error(
                    "RESERVED_FEATURE_BITS_SET",
                    format!("Feature bitfield {} has reserved bits set", i),
                    None,
                )],
            });
        }
        state.feature_region.bump_alloc(bits)?;
    }

    Ok(())
}
```

### 7.3 Access Patterns

| Operation | Access Type | Method |
|-----------|-------------|--------|
| TOKEN_GET_FEATURES (0x24) | Read by index | `feature_region.get(token.feature_index)` |
| FEATURE_GET (0x30) | Read + extract | Read u64, extract by shift/mask |
| FEATURE_SET (0x31) | Read + write | Read u64, modify, write back |
| FEATURE_HAS (0x32) | Read + check | Read u64, extract, compare to default |
| FEATURE_COMPARE_EQ (0x33) | Read + compare | Read u64, extract two, compare |
| FEATURE_COMPARE_MASK (0x34) | Read + compare | Read two u64, mask, compare |
| FEATURE_PACK (0x35) | Write new | Create new u64 from pairs |

### 7.4 Null Feature Reference

The sentinel `0xFFFFFFFF` in `Token.feature_index` indicates that a token has no associated features. When `TOKEN_GET_FEATURES` encounters this sentinel, it pushes `0` (the all-unspecified bitfield) instead of accessing the feature region.

### 7.5 Memory Layout

```
Feature Region:
┌──────────┬──────────┬──────────┬─────┬──────────┐
│ u64 0    │ u64 1    │ u64 2    │ ... │ u64 N-1  │
│ 8 B      │ 8 B      │ 8 B      │     │ 8 B      │
└──────────┴──────────┴──────────┴─────┴──────────┘
├─────────────────── N × 8 bytes ──────────────────┤
```

---

## 8. Constituent Region

### 8.1 Purpose

The constituent region stores syntactic parse tree nodes. During execution, `CONST_MAKE` allocates new nodes and `CONST_ADD_CHILD` links them into tree structures.

### 8.2 Allocation

```rust
fn handle_const_make(
    role_id: u32,
    children: Vec<u32>,
    state: &mut GVMState,
) -> Result<u32, GVMError> {
    let node = ConstituentNode {
        role_id,
        children,
        token_indices: Vec::new(),
        flags: 0,
    };
    let ptr = state.constituent_region.bump_alloc(node)?;
    Ok(ptr)
}
```

### 8.3 Child Index Encoding

Child constituent pointers are stored as **absolute indices** into the constituent region (not relative offsets). This simplifies tree traversal:

```
Root (index 0)
├── Child A (index 1)
│   ├── Grandchild A1 (index 3)
│   └── Grandchild A2 (index 4)
└── Child B (index 2)

Constituent region state after building this tree:
┌─────────┬─────────┬─────────┬───────────┬───────────┐
│ Node 0  │ Node 1  │ Node 2  │ Node 3    │ Node 4    │
│ root    │ child A │ child B │ grand A1  │ grand A2  │
│ children│ children│ children│ children  │ children  │
│ = [1,2] │ = [3,4] │ = []    │ = []      │ = []      │
└─────────┴─────────┴─────────┴───────────┴───────────┘
```

### 8.4 Access Patterns

| Operation | Access Type | Method |
|-----------|-------------|--------|
| CONST_MAKE (0x40) | Write (alloc) | `constituent_region.bump_alloc(node)` |
| CONST_ADD_CHILD (0x41) | Read + write parent | `get_mut(parent)`, push child |
| CONST_GET_CHILD (0x42) | Read parent + index | `get(parent).children[child_index]` |
| CONST_GET_ROLE (0x43) | Read | `get(ptr).role_id` |
| CONST_SET_ROLE (0x44) | Write | `get_mut(ptr).role_id = new_role` |
| CONST_ATTACH_TOKENS (0x45) | Write | `get_mut(ptr).token_indices = ...` |
| CONST_TRAVERSE (0x46) | Read (recursive) | DFS via `get(ptr).children` |

### 8.5 Traversal Depth Limit

The constituent tree depth is bounded by the maximum call stack depth (default: 64). This prevents stack overflow in recursive traversals like `CONST_TRAVERSE`.

---

## 9. String Region

### 9.1 Purpose

The string region stores all strings referenced by the bytecode. It is **immutable after initialization** and is **shared across all GVM instances** that execute the same bytecode.

### 9.2 Storage Model

The string region uses a **flat byte buffer with offset table** rather than a `Vec<String>` to minimize memory overhead:

```
String Region:
┌──────────────┬──────────────┬──────────────┬─────┬──────────────────────┐
│ Offset Table │ Raw String   │ Raw String   │ ... │ Raw String N-1       │
│              │ Data 0       │ Data 1       │     │                      │
│ N × 4 B      │ len0 B       │ len1 B       │     │ len(N-1) B           │
└──────────────┴──────────────┴──────────────┴─────┴──────────────────────┘
```

**Offset table:** `offset_table[i]` = byte offset of string `i` within the raw data buffer (0-based from start of data).
**String data:** Consecutive length-prefixed UTF-8 strings.

### 9.3 Initialization

```rust
fn populate_string_region(
    state: &mut GVMState,
    bytecode: &ParsedBytecode,
) -> Result<(), GVMError> {
    let count = bytecode.string_table.entries.len();
    let mut total_data_size = 0u32;
    let mut offsets = Vec::with_capacity(count);
    let mut raw_data = Vec::new();

    for entry in &bytecode.string_table.entries {
        offsets.push(total_data_size);
        // Encode as length-prefixed: length (varint) + UTF-8 bytes
        let len = entry.bytes.len() as u32;
        let mut buf = Vec::with_capacity(len + 5);
        encode_varint(len, &mut buf);
        buf.extend_from_slice(&entry.bytes);
        raw_data.extend_from_slice(&buf);
        total_data_size += buf.len() as u32;
    }

    state.string_region = StringRegion {
        count: count as u32,
        offsets,
        raw_data,
    };

    Ok(())
}
```

### 9.4 String Access

```rust
fn get_string(region: &StringRegion, index: u32) -> &str {
    assert!(index < region.count, "String index out of bounds");
    let start = region.offsets[index as usize] as usize;
    let end = if index + 1 < region.count {
        region.offsets[(index + 1) as usize] as usize
    } else {
        region.raw_data.len()
    };
    // raw_data contains length-prefixed strings; skip varint length prefix
    // to get to the UTF-8 bytes
    let (_, content_start) = decode_varint(&region.raw_data[start..]);
    std::str::from_utf8(&region.raw_data[start + content_start..end])
        .expect("String table MUST contain valid UTF-8")
}
```

### 9.5 Lazy vs Eager Decoding

Implementations MAY choose between two string access strategies:

| Strategy | Description | Memory | Speed | Use Case |
|----------|-------------|--------|-------|----------|
| **Eager** | All strings decoded to `String` at load time | Higher (heap-allocated Strings) | Faster (O(1) access) | Reference impl (Rust) |
| **Lazy** | Strings decoded on first access, cached | Lower (raw bytes only) | Slower (first access) | Memory-constrained impls (C, embedded) |

**Default:** Eager decoding for the primary Rust implementation.

### 9.6 Access Patterns

| Operation | Access Type | Method |
|-----------|-------------|--------|
| PUSH_STRING (0x14) | Read by index | `string_region.get(index)`, push as `StringIndex` |
| RULE_APPLY (0x50) | Read by index | `string_region.get(rule_name_index)` |
| RULE_FLAG (0x54) | Read by index | `string_region.get(flag_code_index)` |
| EVIDENCE_PUSH (0x60) | Read by index | `string_region.get(stage_name_index)` |
| OUTPUT operations | Read by index | Various metadata key/value lookups |

---

## 10. Rule Region

### 10.1 Purpose

The rule region stores records of every `RULE_APPLY` instruction executed during analysis. Each record captures the rule ID, name, token context, and any feature modifications made by `RULE_MODIFY`.

### 10.2 Allocation

```rust
fn handle_rule_apply(
    rule_id: u32,
    rule_name_index: u32,
    state: &mut GVMState,
) -> Result<(), GVMError> {
    // Validate string reference
    string_index_guard(rule_name_index, &state.string_region)?;

    let record = RuleRecord {
        rule_id,
        rule_name_index,
        token_index: state.current_token_index as u32,
        pc_at_apply: state.pc,
        mod_count: 0,
        modifications: Vec::with_capacity(4), // Pre-allocate small vec
    };

    let idx = state.rule_region.bump_alloc(record)?;
    state.last_rule_index = Some(idx);
    Ok(())
}
```

### 10.3 Modification Recording

When `RULE_MODIFY (0x53)` is executed after a `RULE_APPLY`, the modification is appended to the most recent rule record:

```rust
// Inside RULE_MODIFY handler:
if let Some(last_idx) = state.last_rule_index {
    let record = state.rule_region.get_mut(last_idx)?;
    record.modifications.push(Modification {
        feature_id: feature_id as u8,
        old_value: old_val as u8,
        new_value: new_val as u8,
    });
    record.mod_count += 1;
}
```

### 10.4 Access Patterns

| Operation | Access Type | Method |
|-----------|-------------|--------|
| RULE_APPLY (0x50) | Write (alloc) | `rule_region.bump_alloc(record)` |
| RULE_CONFIRM (0x51) | Read + write last | `get_mut(last_rule_index)` |
| RULE_REJECT (0x52) | Read + write last | `get_mut(last_rule_index)` |
| RULE_MODIFY (0x53) | Read + write last | `get_mut(last_rule_index)`, push mod |
| OUTPUT assembly | Read all | Iterate `0..size` |

---

## 11. Evidence Region

### 11.1 Purpose

The evidence region stores the complete evidence trail: every decision made by every pipeline stage, recorded as structured evidence entries.

### 11.2 Allocation

```rust
fn handle_evidence_push(
    stage_name_index: u32,
    algorithm_index: u32,
    confidence: f64,
    state: &mut GVMState,
) -> Result<(), GVMError> {
    // Validate
    string_index_guard(stage_name_index, &state.string_region)?;
    string_index_guard(algorithm_index, &state.string_region)?;
    if !(0.0..=1.0).contains(&confidence) {
        return Err(GVMError::TypeError {
            expected: "f64 in [0.0, 1.0]".into(),
            got: format!("{}", confidence),
            pc: state.pc,
        });
    }

    let entry = EvidenceRecord {
        stage_name_index,
        algorithm_index,
        confidence,
        token_index: state.current_token_index as u32,
        step: state.step_count,
        reserved: [0u8; 36],
    };

    state.evidence_region.bump_alloc(entry)?;
    Ok(())
}
```

### 11.3 Access Patterns

| Operation | Access Type | Method |
|-----------|-------------|--------|
| EVIDENCE_PUSH (0x60) | Write (alloc) | `evidence_region.bump_alloc(entry)` |
| EVIDENCE_QUERY (0x61) | Size query | `evidence_region.size` |
| EVIDENCE_EMIT (0x62) | Flag set | `state.evidence_emitted = true` |
| OUTPUT assembly | Read all | Iterate `0..size` |

---

## 12. Scratch Buffer

### 12.1 Purpose

The scratch buffer provides a temporary workspace for byte-level operations during execution. It is a **raw byte buffer with cursor-based allocation**.

### 12.2 Layout

```
Scratch Buffer:
┌─────────────────────────────────────────────────────┐
│ data[0..capacity-1]                                  │
│                                                      │
│ [written bytes (0..cursor-1)] [free space (cursor..)]│
│                                                      │
│ ▲                   ▲                                │
│ cursor = 0           cursor = next free position     │
└─────────────────────────────────────────────────────┘
```

### 12.3 Operations

```rust
struct ScratchBuffer {
    data: Vec<u8>,
    capacity: u32,
    cursor: u32,     // Next free position (also = bytes written)
}

impl ScratchBuffer {
    /// Allocate `size` bytes from the scratch buffer.
    /// Returns the starting offset of the allocated region.
    fn alloc(&mut self, size: u32) -> Result<u32, GVMError> {
        let start = self.cursor;
        if start + size > self.capacity {
            return Err(GVMError::ScratchOverflow {
                cursor: start,
                capacity: self.capacity,
            });
        }
        self.cursor += size;
        Ok(start)
    }

    /// Read a byte at the given offset.
    fn read_byte(&self, offset: u32) -> Result<u8, GVMError> {
        if offset >= self.cursor {
            return Err(GVMError::IndexOutOfBounds {
                index: offset,
                count: self.cursor,
            });
        }
        Ok(self.data[offset as usize])
    }

    /// Write a byte at the given offset.
    fn write_byte(&mut self, offset: u32, value: u8) -> Result<(), GVMError> {
        if offset >= self.cursor && offset >= self.capacity {
            // Writing beyond cursor is allowed IF within capacity
            // (this extends the written region)
        }
        if offset >= self.capacity {
            return Err(GVMError::ScratchOverflow {
                cursor: offset,
                capacity: self.capacity,
            });
        }
        self.data[offset as usize] = value;
        if offset >= self.cursor {
            self.cursor = offset + 1;
        }
        Ok(())
    }

    /// Reset the scratch buffer.
    fn clear(&mut self) {
        self.cursor = 0;
        // Data is NOT zeroed (performance)
    }
}
```

### 12.4 Default Capacity

The default scratch buffer capacity is **4096 bytes**. This is sufficient for all planned scratch operations, including temporary string building, serialization, and intermediate computation results.

### 12.5 Access Patterns

The scratch buffer is designed for future instruction extensions (see SPEC-0302 §12.1, opcodes 0xB0–0xBF for dynamic string operations). In the initial instruction set (0x00–0x7F), the scratch buffer is allocated but not used by any defined instruction.

---

## 13. Operand Stack

### 13.1 Purpose

The operand stack is the primary data structure for passing operands between instructions. All instructions that produce or consume values do so through the operand stack.

### 13.2 Layout

```
Operand Stack:
┌──────┬──────┬──────┬──────┬──────┬──────┬──────┬─────────┐
│ Val  │ Val  │ Val  │ Val  │ Val  │ Val  │ Val  │ ...     │
│ 0    │ 1    │ 2    │ 3    │ 4    │ 5    │ 6    │         │
├──────┴──────┴──────┴──────┴──────┴──────┴──────┴─────────┤
│ Top of stack is at index len-1                             │
├────────────────── max_depth × 16 bytes ───────────────────┤
```

### 13.3 Implementation

```rust
struct Stack {
    data: Vec<Value>,         // Contiguous array of Values
    max_depth: u32,           // Maximum capacity (default: 1024)
}

impl Stack {
    fn new(max_depth: u32) -> Self {
        Self {
            data: Vec::with_capacity(max_depth as usize),
            max_depth,
        }
    }

    fn push(&mut self, value: Value) -> Result<(), GVMError> {
        if self.data.len() >= self.max_depth as usize {
            return Err(GVMError::StackOverflow {
                stack_type: "operand".into(),
                depth: self.data.len() as u32,
                max: self.max_depth,
            });
        }
        self.data.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, GVMError> {
        self.data.pop().ok_or(GVMError::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value, GVMError> {
        self.data.last().ok_or(GVMError::StackUnderflow)
    }

    fn peek_mut(&mut self) -> Result<&mut Value, GVMError> {
        self.data.last_mut().ok_or(GVMError::StackUnderflow)
    }

    fn dup(&mut self) -> Result<(), GVMError> {
        let val = self.peek()?.clone();
        self.push(val)
    }

    fn swap(&mut self) -> Result<(), GVMError> {
        let len = self.data.len();
        if len < 2 {
            return Err(GVMError::StackUnderflow);
        }
        self.data.swap(len - 1, len - 2);
        Ok(())
    }

    fn pop_typed(&mut self, expected: &str) -> Result<Value, GVMError> {
        let val = self.pop()?;
        if val.type_tag() != expected {
            return Err(GVMError::TypeError {
                expected: expected.to_string(),
                got: val.type_tag().to_string(),
                pc: 0, // caller fills this in
            });
        }
        Ok(val)
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn len(&self) -> usize { self.data.len() }
    fn is_empty(&self) -> bool { self.data.is_empty() }
    fn depth(&self) -> u32 { self.data.len() as u32 }
}
```

### 13.4 Stack Depth Invariants

| Instruction | Stack Effect | Depth Change |
|-------------|-------------|--------------|
| PUSH (any) | [] → [val] | +1 |
| POP | [any] → [] | -1 |
| DUP | [a] → [a, a] | +1 |
| SWAP | [a, b] → [b, a] | 0 |
| FEATURE_GET | [fb] → [u32] | 0 |
| FEATURE_SET | [fb, u32] → [fb] | -1 |
| CONST_MAKE | [u32, ptr...] → [ptr] | -(n+1) |
| OUTPUT_ADD_TREE | [ptr, f64] → [] | -2 |

**Maximum stack depth reached during typical execution:** 10–50 entries (for a 10-word sentence).
**Safety limit:** Configurable via `max_stack_depth` (default: 1024).

### 13.5 Type Safety

Every `Value` on the operand stack carries a type tag. Before any instruction uses a value from the stack, it MUST check the type tag and return `TYPE_ERROR` if the type is unexpected.

```
Required type checks per instruction:

PUSH_I32:      No pop (push only)
PUSH_STRING:   No pop (push only)
POP:           No type check (discards any value)
DUP:           No type check (duplicates any value)
SWAP:          No type check (swaps any two values)
LOAD_TOKEN:    No pop (operand from instruction encoding)
TOKEN_GET_TEXT:  ✓ pop is TokenIndex
FEATURE_GET:     ✓ pop is FeatureBits
FEATURE_SET:     ✓ pop is FeatureBits, ✓ pop is I32
CONST_MAKE:      ✓ pop is I32 (count), ✓ pops are ConstituentPtr
JUMP_IF_TRUE:    ✓ pop is Bool
RULE_RESOLVE:    ✓ pops are TokenIndex or I32
OUTPUT_ADD_TREE: ✓ pop is ConstituentPtr, ✓ pop is F64
```

### 13.6 Stack Overflow Protection

Every push operation checks against `max_depth`:

```rust
fn push(&mut self, value: Value) -> Result<(), GVMError> {
    if self.data.len() >= self.max_depth as usize {
        return Err(GVMError::StackOverflow {
            stack_type: "operand".into(),
            depth: self.data.len() as u32,
            max: self.max_depth,
        });
    }
    // ...
}
```

---

## 14. Call Stack

### 14.1 Purpose

The call stack stores return addresses for `CALL`/`RETURN` subroutine calls. It is separate from the operand stack to prevent interference between call/return flow control and operand passing.

### 14.2 Layout

```
Call Stack:
┌──────────┬──────────┬──────────┬─────┬──────────┐
│ addr 0   │ addr 1   │ addr 2   │ ... │ addr N-1 │
│ u32      │ u32      │ u32      │     │ u32      │
│ 4 B      │ 4 B      │ 4 B      │     │ 4 B      │
└──────────┴──────────┴──────────┴─────┴──────────┘
├────────────────── N × 4 bytes ─────────────────┤

Top of stack = data[len-1]
```

### 14.3 Implementation

The call stack is a simple stack of `u32` values (return addresses):

```rust
struct CallStack {
    data: Vec<u32>,
    max_depth: u32,        // Default: 64
}

impl CallStack {
    fn new(max_depth: u32) -> Self {
        Self {
            data: Vec::with_capacity(max_depth as usize),
            max_depth,
        }
    }

    fn push(&mut self, addr: u32) -> Result<(), GVMError> {
        if self.data.len() >= self.max_depth as usize {
            return Err(GVMError::CallStackOverflow {
                depth: self.data.len() as u32,
                max: self.max_depth,
            });
        }
        self.data.push(addr);
        Ok(())
    }

    fn pop(&mut self) -> Result<u32, GVMError> {
        self.data.pop().ok_or(GVMError::CallStackUnderflow)
    }

    fn clear(&mut self) { self.data.clear(); }
    fn depth(&self) -> u32 { self.data.len() as u32 }
}
```

### 14.4 Typical Depth

| Pattern | Call Depth | Example |
|---------|-----------|---------|
| No calls | 0 | Simple linear bytecode |
| 1 level | 1 | Token loop with subroutine |
| 2 levels | 2 | Nested rule application |
| Deep recursion | ≤ 64 | Complex constituent traversal |

**Default limit:** 64 (sufficient for all planned use cases).

---

## 15. Memory Budget Model

### 15.1 Budget Calculation Formula

The total memory budget for a GVM instance is the sum of all region allocations plus overhead:

```rust
fn calculate_memory_budget(
    header: &BytecodeHeader,
    config: &GVMConfig,
) -> MemoryBudget {
    // ── Immutable regions (populated from bytecode) ──
    let token_region = effective_capacity(
        header.token_count, DEFAULT_TOKENS
    ) as u64 * TOKEN_SIZE;

    let feature_region = effective_capacity(
        header.feature_count, DEFAULT_FEATURES
    ) as u64 * FEATURE_SIZE;

    let constituent_region = effective_capacity(
        header.constituent_count, DEFAULT_CONSTITUENTS
    ) as u64 * CONSTITUENT_SIZE;

    let rule_region = effective_capacity(
        header.rule_count, DEFAULT_RULES
    ) as u64 * RULE_SIZE;

    // ── String region (from bytecode) ──
    let string_region = header.string_table_size as u64
        + (header.string_count as u64) * 4;  // offset table

    // ── Mutable regions (default capacity) ──
    let evidence_region = DEFAULT_EVIDENCE as u64 * EVIDENCE_SIZE;

    // ── Stacks ──
    let operand_stack = config.max_stack_depth as u64 * VALUE_SIZE;
    let call_stack = config.max_call_depth as u64 * 4;  // u32 addresses

    // ── Scratch ──
    let scratch = DEFAULT_SCRATCH as u64;

    // ── Overhead ──
    let overhead = 4096;  // VM structs, enums, metadata, etc.

    MemoryBudget {
        token_region,
        feature_region,
        constituent_region,
        string_region,
        rule_region,
        evidence_region,
        operand_stack,
        call_stack,
        scratch,
        overhead,

        total: token_region + feature_region + constituent_region
             + string_region + rule_region + evidence_region
             + operand_stack + call_stack + scratch + overhead,
    }
}
```

### 15.2 Size Constants

These constants MUST be identical across all GVM implementations:

| Constant | Value | Description |
|----------|-------|-------------|
| `TOKEN_SIZE` | 48 | Size of Token struct in bytes |
| `FEATURE_SIZE` | 8 | Size of u64 feature bitfield |
| `CONSTITUENT_SIZE` | 48 | Default allocation for ConstituentNode |
| `RULE_SIZE` | 32 | Default allocation for RuleRecord |
| `EVIDENCE_SIZE` | 64 | Size of EvidenceRecord |
| `VALUE_SIZE` | 16 | Size of Value on operand stack |
| `DEFAULT_TOKENS` | 256 | Default token region capacity |
| `DEFAULT_FEATURES` | 512 | Default feature region capacity |
| `DEFAULT_CONSTITUENTS` | 1024 | Default constituent region capacity |
| `DEFAULT_RULES` | 512 | Default rule region capacity |
| `DEFAULT_EVIDENCE` | 1024 | Default evidence region capacity |
| `DEFAULT_SCRATCH` | 4096 | Default scratch buffer size (bytes) |
| `DEFAULT_STACK_DEPTH` | 1024 | Default operand stack max depth |
| `DEFAULT_CALL_DEPTH` | 64 | Default call stack max depth |

### 15.3 Budget Example: Small Sentence (3 tokens)

| Region | Effective Capacity | Per-Entry | Total |
|--------|-------------------|-----------|-------|
| Token | max(3, 256) = 256 | 48 B | 12,288 B |
| Feature | max(3, 512) = 512 | 8 B | 4,096 B |
| Constituent | max(8, 1024) = 1024 | 48 B | 49,152 B |
| String | ~500 B (bytecode) | variable | ~500 B |
| Rule | max(5, 512) = 512 | 32 B | 16,384 B |
| Evidence | 1024 | 64 B | 65,536 B |
| Scratch | 4096 | 1 B | 4,096 B |
| Operand Stack | 1024 | 16 B | 16,384 B |
| Call Stack | 64 | 4 B | 256 B |
| Overhead | — | — | 4,096 B |
| **Total** | | | **~172 KB** |

### 15.4 Budget Example: Large Sentence (50 tokens)

| Region | Effective Capacity | Per-Entry | Total |
|--------|-------------------|-----------|-------|
| Token | max(50, 256) = 256 | 48 B | 12,288 B |
| Feature | max(100, 512) = 512 | 8 B | 4,096 B |
| Constituent | max(200, 1024) = 1024 | 48 B | 49,152 B |
| String | ~5 KB (bytecode) | variable | ~5,000 B |
| Rule | max(80, 512) = 512 | 32 B | 16,384 B |
| Evidence | 1024 | 64 B | 65,536 B |
| Scratch | 4096 | 1 B | 4,096 B |
| Operand Stack | 1024 | 16 B | 16,384 B |
| Call Stack | 64 | 4 B | 256 B |
| Overhead | — | — | 4,096 B |
| **Total** | | | **~177 KB** |

The total is similar because default capacities dominate for both small and medium sentences. This is by design: the fixed overhead of default capacities ensures that all bytecodes have sufficient workspace regardless of size.

### 15.5 Budget Validation

Before execution, the GVM verifier MUST check that the calculated budget does not exceed `config.max_memory_bytes`:

```rust
fn verify_memory_budget(
    header: &BytecodeHeader,
    config: &GVMConfig,
) -> Result<(), GVMError> {
    let budget = calculate_memory_budget(header, config);
    if budget.total > config.max_memory_bytes {
        return Err(GVMError::MaxMemoryExceeded {
            memory: budget.total,
            limit: config.max_memory_bytes,
        });
    }
    Ok(())
}
```

---

## 16. Bounds Checking & Safety Guarantees

### 16.1 Universal Bounds Check Pattern

Every memory access in the GVM MUST follow this pattern:

```
ALL reads:   index < region.size   → OK, access region.data[index]
                                     → ERROR, return IndexOutOfBounds

ALL writes:  index < region.size   → OK, write region.data[index]
             (mutable regions only)  → ERROR, return IndexOutOfBounds

ALL allocs:  region.size < region.capacity  → OK, bump alloc
                                             → ERROR, return RegionFull
```

### 16.2 Bounds Check Matrix

| Access Type | Check | Error Code |
|-------------|-------|------------|
| Token read | `index < token_region.size` | `TOKEN_INDEX_OUT_OF_BOUNDS` |
| Feature read | `index < feature_region.size` | `FEATURE_INDEX_OUT_OF_BOUNDS` |
| Constituent read | `index < constituent_region.size` | `CONSTITUENT_INDEX_OUT_OF_BOUNDS` |
| Rule read | `index < rule_region.size` | `INDEX_OUT_OF_BOUNDS` |
| Evidence read | `index < evidence_region.size` | `INDEX_OUT_OF_BOUNDS` |
| String read | `index < string_region.count` | `STRING_INDEX_OUT_OF_BOUNDS` |
| Scratch read | `offset < cursor` | `INDEX_OUT_OF_BOUNDS` |
| Scratch write | `offset < capacity` | `SCRATCH_OVERFLOW` |
| Constituent alloc | `size < capacity` | `CONSTITUENT_REGION_FULL` |
| Rule alloc | `size < capacity` | `REGION_FULL` |
| Evidence alloc | `size < capacity` | `EVIDENCE_REGION_FULL` |
| Stack push | `len < max_depth` | `STACK_OVERFLOW` |
| Stack pop | `len > 0` | `STACK_UNDERFLOW` |
| Stack swap | `len >= 2` | `STACK_UNDERFLOW` |
| Call stack push | `len < max_depth` | `CALL_STACK_OVERFLOW` |
| Call stack pop | `len > 0` | `CALL_STACK_UNDERFLOW` |

### 16.3 Bounds Check Performance Impact

Bounds checks are **inlined, predictable, and fast**:

```rust
// Rust: bounds check adds ~2-3 CPU instructions
fn get(&self, index: u32) -> Result<&T, GVMError> {
    // This compiles to:
    //   cmp index, self.size
    //   jae .oob
    //   lea result, [self.data + index * sizeof(T)]
    self.data.get(index as usize)
        .ok_or(GVMError::IndexOutOfBounds { index, count: self.size })
}
```

**Typical cost:** 1–3 ns per bounds check (branch-predictable for valid accesses).

**Optimization:** For hot paths where bounds have already been validated by the verifier (e.g., LOAD_TOKEN after verification), implementations MAY use unchecked access internally, but the verifier path MUST always check.

### 16.4 Safety Guarantees (Complete)

```
┌──────────────────────────────────────────────────────────────┐
│                 COMPLETE MEMORY SAFETY MODEL                  │
│                                                              │
│  1. NO RAW POINTERS                                           │
│     All memory access is through typed u32 indices.           │
│     No way to construct an arbitrary memory address.          │
│     No pointer arithmetic.                                    │
│                                                              │
│  2. NO DYNAMIC ALLOCATION DURING EXECUTION                    │
│     All memory pre-allocated at instance creation.            │
│     "Allocation" = incrementing bump pointer.                 │
│     No malloc, no free, no GC during execution.              │
│                                                              │
│  3. BOUNDS CHECKED ON EVERY ACCESS                            │
│     Every read/write validates: index < region.size.          │
│     OOB access returns structured error, not crash.           │
│     Deterministic behavior for all boundary conditions.       │
│                                                              │
│  4. NO USE-AFTER-FREE                                         │
│     No deallocation during execution.                         │
│     Memory freed only when GVM instance is destroyed.         │
│     Regions are reset (size=0, data preserved) between uses.  │
│                                                              │
│  5. TYPE SAFETY ON OPERAND STACK                              │
│     Every Value has a type tag (1 byte).                      │
│     Type mismatches detected before any operation.            │
│     pop_typed() enforces expected type.                       │
│                                                              │
│  6. NO DATA RACES                                             │
│     Single-threaded execution per instance.                   │
│     No shared mutable state between instances.                │
│     Bytecode is read-only (immutable after load).             │
│                                                              │
│  7. CAPACITY OVERRUN PREVENTED                                │
│     All regions have fixed capacities.                        │
│     Bump allocator checks capacity before writing.            │
│     Overrun returns REGION_FULL error (not silent corruption).│
│                                                              │
│  8. DETERMINISTIC LAYOUT                                      │
│     Same bytecode + same config = same memory layout.         │
│     No ASLR, no randomized layout within GVM.                 │
│     Iteration order is insertion order (deterministic).       │
└──────────────────────────────────────────────────────────────┘
```

---

## 17. Cross-Language Porting Reference

### 17.1 Region Porting Patterns

Each language should implement the abstract `Region<T>` and `Stack` types using native idioms:

| Language | Region Storage | Stack Storage | Value Type |
|----------|---------------|---------------|------------|
| **Rust** | `Vec<T>` (contiguous, bounds-checked by `get()`) | `Vec<Value>` | `enum Value` (tagged union) |
| **C** | `void*` + `entry_size` (raw bytes, manual offset calc) | `struct Value*` + `size` | `struct Value` (tagged union) |
| **Python** | `list[Any]` + capacity check (note: stores PyObject* pointers, not inline data; use `array('Q')` for numeric regions) | `list[Value]` | `@dataclass Value` |
| **TypeScript** | `T[]` or `TypedArray` | `Value[]` | `type Value = ...` (union) |
| **Go** | `[]T` (slice, pre-allocated with `make(T, cap)`) | `[]Value` | `struct Value` (fields + type) |
| **Java** | `ArrayList<T>` or `T[]` | `ArrayList<Value>` | `sealed interface Value` |

### 17.2 Rust Reference Implementation

```rust
// ── Generic Region ──
#[derive(Debug, Clone)]
pub struct Region<T: Clone + Default> {
    data: Vec<T>,
    capacity: u32,
}

impl<T: Clone + Default> Region<T> {
    pub fn new(name: &str, capacity: u32) -> Self {
        Self {
            data: Vec::with_capacity(capacity as usize),
            capacity,
        }
    }

    pub fn alloc(&mut self, value: T) -> Result<u32, GVMError> {
        let idx = self.data.len() as u32;
        if idx >= self.capacity {
            return Err(GVMError::RegionFull {
                region: std::any::type_name::<T>().into(),
                size: idx,
                capacity: self.capacity,
            });
        }
        self.data.push(value);
        Ok(idx)
    }

    pub fn get(&self, index: u32) -> Result<&T, GVMError> {
        self.data.get(index as usize).ok_or(
            GVMError::IndexOutOfBounds {
                index,
                count: self.data.len() as u32,
            }
        )
    }

    pub fn get_mut(&mut self, index: u32) -> Result<&mut T, GVMError> {
        self.data.get_mut(index as usize).ok_or(
            GVMError::IndexOutOfBounds {
                index,
                count: self.data.len() as u32,
            }
        )
    }

    pub fn clear(&mut self) { self.data.clear(); }
    pub fn size(&self) -> u32 { self.data.len() as u32 }
    pub fn capacity(&self) -> u32 { self.capacity }
}
```

### 17.3 C Implementation

```c
// ── Typed Region (raw bytes, manual indexing) ──
typedef struct {
    void* data;          // Pre-allocated block: calloc(capacity, entry_size)
    uint32_t entry_size; // Size of each entry in bytes
    uint32_t size;       // Current number of valid entries
    uint32_t capacity;   // Maximum number of entries
} Region;

// ── Value Type (tagged union) ──
typedef struct {
    uint8_t type;  // 0=I32, 1=I64, 2=F64, 3=StringIndex,
                   // 4=TokenIndex, 5=FeatureBits, 6=ConstituentPtr,
                   // 7=Bool, 8=Null
    union {
        int32_t i32;
        int64_t i64;
        double f64;
        uint32_t u32;
        uint64_t u64;
        uint8_t bool_val;
    } data;
} Value;

// ── Stack ──
typedef struct {
    Value* data;       // Pre-allocated: malloc(max_depth * sizeof(Value))
    uint32_t size;     // Current number of values
    uint32_t max_depth;
} Stack;

// ── Region allocation ──
uint32_t region_alloc(Region* r, const void* entry) {
    if (r->size >= r->capacity) return UINT32_MAX; // Full
    uint32_t idx = r->size;
    memcpy((uint8_t*)r->data + idx * r->entry_size, entry, r->entry_size);
    r->size++;
    return idx;
}

// ── Region read ──
void* region_get(Region* r, uint32_t index) {
    if (index >= r->size) return NULL; // OOB
    return (uint8_t*)r->data + index * r->entry_size;
}

// ── Region write ──
int region_set(Region* r, uint32_t index, const void* entry) {
    if (index >= r->size) return -1; // OOB
    memcpy((uint8_t*)r->data + index * r->entry_size, entry, r->entry_size);
    return 0;
}

// ── Bounds-checked stack operations ──
int stack_push(Stack* s, Value v) {
    if (s->size >= s->max_depth) return -1; // Overflow
    s->data[s->size++] = v;
    return 0;
}

int stack_pop(Stack* s, Value* out) {
    if (s->size == 0) return -1; // Underflow
    *out = s->data[--s->size];
    return 0;
}
```

### 17.4 Python Implementation

```python
# ── Region ──
class Region:
    """A typed, pre-allocated memory region."""
    def __init__(self, name: str, capacity: int):
        self.name = name
        self.data: list[Any] = []
        self.capacity = capacity

    def alloc(self, item: Any) -> int:
        if len(self.data) >= self.capacity:
            raise GVMError.region_full(self.name, len(self.data), self.capacity)
        idx = len(self.data)
        self.data.append(item)
        return idx

    def __getitem__(self, idx: int) -> Any:
        if idx < 0 or idx >= len(self.data):
            raise GVMError.index_out_of_bounds(idx, len(self.data))
        return self.data[idx]

    def __setitem__(self, idx: int, value: Any) -> None:
        if idx < 0 or idx >= len(self.data):
            raise GVMError.index_out_of_bounds(idx, len(self.data))
        self.data[idx] = value

    @property
    def size(self) -> int:
        return len(self.data)

    def clear(self) -> None:
        self.data.clear()


# ── Value Type ──
@dataclass
class Value:
    class Type(Enum):
        I32 = auto()
        I64 = auto()
        F64 = auto()
        STRING_INDEX = auto()
        TOKEN_INDEX = auto()
        FEATURE_BITS = auto()
        CONSTITUENT_PTR = auto()
        BOOL = auto()
        NULL = auto()

    type: Type
    data: Any  # int, float, bool, or None

    def type_tag(self) -> str:
        return self.type.name.lower()

    @classmethod
    def i32(cls, v: int) -> 'Value':
        return cls(cls.Type.I32, v)

    @classmethod
    def feature_bits(cls, v: int) -> 'Value':
        return cls(cls.Type.FEATURE_BITS, v)


# ── Stack ──
class Stack:
    def __init__(self, max_depth: int = 1024):
        self.data: list[Value] = []
        self.max_depth = max_depth

    def push(self, v: Value) -> None:
        if len(self.data) >= self.max_depth:
            raise GVMError.stack_overflow(len(self.data), self.max_depth)
        self.data.append(v)

    def pop(self) -> Value:
        if not self.data:
            raise GVMError.stack_underflow()
        return self.data.pop()

    def peek(self) -> Value:
        if not self.data:
            raise GVMError.stack_underflow()
        return self.data[-1]

    def dup(self) -> None:
        self.push(self.peek())

    def swap(self) -> None:
        if len(self.data) < 2:
            raise GVMError.stack_underflow()
        self.data[-1], self.data[-2] = self.data[-2], self.data[-1]

    def clear(self) -> None:
        self.data.clear()
```

### 17.5 TypeScript Implementation

```typescript
// ── Region ──
class Region<T> {
    private data: T[] = [];
    constructor(private capacity: number) {}

    alloc(item: T): number {
        if (this.data.length >= this.capacity) {
            throw new GVMError(`Region full: ${this.data.length}/${this.capacity}`);
        }
        const idx = this.data.length;
        this.data.push(item);
        return idx;
    }

    get(index: number): T {
        const item = this.data[index];
        if (item === undefined) {
            throw new GVMError(`Index ${index} out of bounds (size: ${this.data.length})`);
        }
        return item;
    }

    set(index: number, value: T): void {
        if (index < 0 || index >= this.data.length) {
            throw new GVMError(`Index ${index} out of bounds (size: ${this.data.length})`);
        }
        this.data[index] = value;
    }

    get size(): number { return this.data.length; }
    clear(): void { this.data.length = 0; }
}

// ── Feature Region (TypedArray) ──
class FeatureRegion {
    private data: BigUint64Array;
    private _size: number = 0;

    constructor(capacity: number) {
        this.data = new BigUint64Array(capacity);
    }

    alloc(bits: bigint): number {
        if (this._size >= this.data.length) {
            throw new GVMError(`Feature region full`);
        }
        const idx = this._size;
        this.data[idx] = bits;
        this._size++;
        return idx;
    }

    get(index: number): bigint {
        if (index < 0 || index >= this._size) {
            throw new GVMError(`Feature index ${index} out of bounds`);
        }
        return this.data[index];
    }

    set(index: number, bits: bigint): void {
        if (index < 0 || index >= this._size) {
            throw new GVMError(`Feature index ${index} out of bounds`);
        }
        this.data[index] = bits;
    }

    get size(): number { return this._size; }
    clear(): void { this._size = 0; }
}
```

---

## 18. Performance Considerations

### 18.1 Hot Paths

The following memory access patterns are **hot paths** — they account for the majority of GVM execution time:

| Hot Path | Instructions | Memory Access | Optimization |
|----------|-------------|---------------|--------------|
| Token iteration | TOKEN_ITERATE + LOAD_TOKEN | Token region read (sequential) | Prefetch next token; cache line alignment |
| Feature extraction | TOKEN_GET_FEATURES + FEATURE_GET | Feature region read + bit ops | Inline mask/shift; use pre-computed constants |
| Feature comparison | FEATURE_COMPARE_EQ / MASK | Feature region read × 2 | Combined mask compare; avoid branch |
| Constituent building | CONST_MAKE + CONST_ADD_CHILD | Constituent region write | Pre-allocate contiguous blocks |
| Rule application | RULE_APPLY + RULE_MODIFY | Rule region write | Batch writes |

### 18.2 Cache Line Considerations

Typical cache line size is 64 bytes on modern x86 and ARM processors:

| Region | Entry Size | Entries per Cache Line | Notes |
|--------|-----------|------------------------|-------|
| Token | 48 B | 1.33 | Fits in one cache line (occupies first 48 B of a 64 B line); may straddle 2 lines if region base is not 64 B-aligned — pad region start to 64 B boundary to avoid splits |
| Feature | 8 B | 8 | Excellent locality |
| Constituent | 48 B | 1.33 | Straddles 2 cache lines |
| Rule | 32 B | 2 | Good locality |
| Evidence | 64 B | 1 | Exactly 1 cache line |
| Value (stack) | 16 B | 4 | Good locality |

**Recommendation:** Align all region data to 64-byte cache line boundaries.

### 18.3 Allocation Patterns by Region

| Region | Allocation Pattern | Allocation Count (typical) | Growth |
|--------|-------------------|---------------------------|--------|
| Token | Bulk at load | token_count | None during execution |
| Feature | Bulk at load | feature_count | None during execution |
| Constituent | Incremental | ~3–10 per token | Monotonic |
| Rule | Incremental | ~2–8 per token | Monotonic |
| Evidence | Incremental | ~1–3 per token | Monotonic |
| Scratch | Sequential | ~5–20 writes | Cursor-based |
| Operand Stack | Push/Pop | 100–1000 ops | Full dynamic range |
| Call Stack | Push/Pop | 0–20 calls | Shallow |

### 18.4 Memory Bandwidth Estimate

For a typical 10-word sentence execution (~500 instructions):

| Operation | Memory Access | Data Moved |
|-----------|--------------|------------|
| Token region reads | 10 × 48 B | 480 B |
| Feature region reads | 10 × 8 B | 80 B |
| Constituent region writes | ~30 × 48 B | 1,440 B |
| Rule region writes | ~20 × 32 B | 640 B |
| Evidence region writes | ~10 × 64 B | 640 B |
| Stack operations | ~500 × 16 B | 8 KB |
| **Total data moved** | | **~11 KB** |

At DDR5 bandwidth (~50 GB/s), data movement takes ~0.2 μs, which is negligible compared to instruction dispatch overhead.

### 18.5 Optimization Recommendations

| Priority | Optimization | Expected Gain | Complexity |
|----------|-------------|---------------|------------|
| P0 | **Pre-allocate region data as contiguous blocks** | 10–20% fewer cache misses | Low |
| P0 | **Use language-native contiguous arrays** (Vec, array, TypedArray) | 5–15% faster access | Low |
| P1 | **Align regions to 64-byte cache lines** | 5–10% fewer cache misses | Low |
| P1 | **Use unchecked access after verifier validation** | 2–5% faster execution | Medium |
| P2 | **Prefetch next token during token iteration** | 5–15% faster token loops | Medium |
| P2 | **Batch-allocate constituent nodes** (pre-allocate in blocks) | 10–20% fewer alloc checks | Medium |
| P3 | **Use inline feature extractors** instead of generic mask/shift | 10–20% faster feature ops | Low |

---

## 19. Testing & Verification

### 19.1 Test Categories

| Category | Tests | Coverage |
|----------|-------|----------|
| **Region initialization** | 20 | Correct sizing, capacity from header, default fallback |
| **Bump allocation** | 25 | Monotonic growth, capacity limit, REGION_FULL error |
| **Bounds checking** | 40 | Every bounds check error code (16 regions × 2–3 tests each) |
| **Stack operations** | 30 | Push/pop/dup/swap, overflow, underflow, type errors |
| **Call stack** | 10 | Push/pop, overflow, underflow |
| **Region reset** | 15 | Clear all regions, reuse, data isolation between executions |
| **String access** | 15 | String index bounds, UTF-8 validation, lazy/eager |
| **Token region** | 10 | Load and access tokens by index, feature reference |
| **Constituent region** | 15 | Allocate nodes, link children, traverse, depth limit |
| **Rule region** | 10 | Rule record allocation, modification recording |
| **Evidence region** | 10 | Evidence push, confidence validation, step tracking |
| **Memory budget** | 15 | Budget calculation, max_memory enforcement, edge cases |
| **Cross-implementation** | 10 | Same bytecode → same memory layout across implementations |

**Total:** ~215 memory model conformance tests.

### 19.2 Test Fixture Format

```jsonc
{
    "spec": "SPEC-0304/memory-test",
    "version": "1.0.0",
    "test_name": "bump_alloc_region_full",
    "description": "Allocate beyond capacity should return REGION_FULL",

    "config": {
        "region_capacity": 3,
        "entry_size": 8
    },

    "operations": [
        { "op": "alloc", "expected": { "result": 0 } },
        { "op": "alloc", "expected": { "result": 1 } },
        { "op": "alloc", "expected": { "result": 2 } },
        { "op": "alloc", "expected": { "error": "REGION_FULL", "size": 3, "capacity": 3 } }
    ],

    "expected_final_state": {
        "size": 3,
        "entries": ["entry_0", "entry_1", "entry_2"]
    }
}
```

### 19.3 Cross-Implementation Test Vectors

Test vectors for memory model conformance use **byte-for-byte identical `.agos` files** paired with expected memory state:

```
tests/vectors/memory/
├── v1.0.0/
│   ├── token_region_access.agos       # LOAD_TOKEN at various indices
│   ├── token_region_access.json       # Expected memory state
│   ├── constituent_tree.agos          # Build and traverse tree
│   ├── constituent_tree.json
│   ├── stack_deep_push.agos           # Push to stack limit
│   ├── stack_deep_push.json
│   ├── region_full_error.agos         # Trigger REGION_FULL
│   ├── region_full_error.json
│   ├── region_reset_reuse.agos        # Reset and reuse
│   ├── region_reset_reuse.json
│   └── manifest.json                  # SHA-256 of all expected outputs
```

### 19.4 Memory Leak Detection

```rust
/// Leak detection: verify all regions are properly cleaned up.
#[cfg(test)]
fn test_no_memory_leaks() {
    let config = GVMConfig::default();
    let mut gvm = GVM::new(config);

    // Execute several times
    for i in 0..10 {
        let bytecode = generate_test_bytecode(format!("test_{}", i));
        gvm.execute(&bytecode).unwrap_or_else(|_| AnalysisResult::default());
    }

    // Verify total allocation hasn't grown
    assert!(gvm.memory_usage() < 2 * DEFAULT_MEMORY_BUDGET,
        "Memory leak detected: regions not properly reset");
}
```

### 19.5 Quality Gates

| Gate | Criteria | Blocking |
|------|----------|----------|
| **All bounds checks** | Every memory access in every instruction handler has a bounds check | Yes |
| **No unchecked array access** | 0 occurrences of `data[i]` without preceding bounds check | Yes |
| **Region capacity overflow** | All bump alloc paths return `REGION_FULL` on overflow | Yes |
| **Stack overflow/underflow** | All push/pop paths return appropriate errors | Yes |
| **Reset completeness** | All mutable regions and stacks cleared on reset | Yes |
| **Memory budget validation** | Budget checked before execution | Yes |
| **Feature alignment** | All feature bit shifts/masks match SPEC-0102 | Yes |

---

## 20. Cross-References

### 20.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| RFC-0003 §3 | GVM State (State Model) | GVMState struct that contains all memory regions |
| RFC-0003 §6 | Memory Model | High-level memory region overview (source for this spec) |
| RFC-0003 §6.2 | Memory Allocation | Default capacities table |
| RFC-0003 §6.3 | Bounds Checking | Bounds checking table |
| RFC-0003 §6.4 | Memory Safety Guarantees | 4 foundational safety guarantees |
| RFC-0003 §7.2 | Execution Guarantees | Bounded memory guarantee |
| RFC-0003 §8.1 | Execution Configuration | max_memory, max_stack_depth, max_call_depth |
| RFC-0002 §5 | Header Section | capacity fields in bytecode header |
| RFC-0002 §7 | String Table Section | String table format (source for StringRegion) |
| RFC-0002 §8 | Token Section | Token record format (source for Token) |
| RFC-0002 §9 | Feature Section | Feature bitfield layout (source for FeatureRegion) |
| RFC-0002 §10 | Constituent Section | Node record format (source for ConstituentNode) |
| SPEC-0301 §4.2 | Instance Pool Management | Region lifecycle in pooling context |
| SPEC-0301 §6.1 | Memory Region Architecture | Region hierarchy diagram |
| SPEC-0301 §6.2 | Capacity Tuning | Per-region capacity recommendations |
| SPEC-0301 §6.3 | Memory Safety Model | 6 safety guarantees |
| SPEC-0301 §6.4 | Memory Budget Calculation | Budget formula with constants |
| SPEC-0302 §14 | Instruction Cost Model | Per-instruction memory impact |
| SPEC-0302 §16 | Feature ID Reference | Bitfield shift/mask constants |
| SPEC-0303 §4 | Core Data Structures | Region, Stack, Value type definitions |
| SPEC-0303 §6 | Memory Region Management | Implementation patterns, sizing, bounds |
| SPEC-0102 §8 | 64-Bit Bitfield Reference | Complete bitfield layout with pack/unpack |
| SPEC-0001-C9 | Performance Targets | Latency, memory, throughput goals |
| SPEC-0001-C8 | Security Model | Sandboxing, bounds checking requirements |

### 20.2 External References

| Reference | Relevance |
|-----------|-----------|
| **Mimalloc (Microsoft)** | Reference for bump allocator design patterns |
| **Region-based Memory Management (Tofte, Talpin)** | Theoretical foundation for region-based memory management |
| **WebAssembly Linear Memory** | Similar pre-allocated contiguous memory model |
| **JVM Metaspace** | Pre-allocated memory region architecture |
| **slab allocator (Linux kernel)** | Object-caching allocator patterns (inspiration for per-type regions) |

---

## Progress Summary

**SPEC-0304: GVM Memory Model — Memory Arena Specification**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Architecture Overview | ✓ COMPLETE |
| 3 | Region Type Definitions & Layouts | ✓ COMPLETE |
| 4 | Region Lifecycle | ✓ COMPLETE |
| 5 | Bump Allocator Model | ✓ COMPLETE |
| 6 | Token Region | ✓ COMPLETE |
| 7 | Feature Region | ✓ COMPLETE |
| 8 | Constituent Region | ✓ COMPLETE |
| 9 | String Region | ✓ COMPLETE |
| 10 | Rule Region | ✓ COMPLETE |
| 11 | Evidence Region | ✓ COMPLETE |
| 12 | Scratch Buffer | ✓ COMPLETE |
| 13 | Operand Stack | ✓ COMPLETE |
| 14 | Call Stack | ✓ COMPLETE |
| 15 | Memory Budget Model | ✓ COMPLETE |
| 16 | Bounds Checking & Safety Guarantees | ✓ COMPLETE |
| 17 | Cross-Language Porting Reference | ✓ COMPLETE (Rust, C, Python, TypeScript) |
| 18 | Performance Considerations | ✓ COMPLETE |
| 19 | Testing & Verification | ✓ COMPLETE |
| 20 | Cross-References | ✓ COMPLETE |

---

*End of SPEC-0304*
