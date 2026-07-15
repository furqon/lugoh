---
rfc_id: RFC-0003
title: Grammar Virtual Machine (GVM) — Instruction Set & Execution Model
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
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-9, IR-10)
  - SPEC-0001-C9: Performance Targets & Constraints
  - RFC-0002: Grammar Bytecode Format (proposed)
  - SPEC-0301: Grammar Runtime (planned)
supersedes: None
---

# RFC-0003: Grammar Virtual Machine (GVM) — Instruction Set & Execution Model

## Table of Contents

1. [Introduction](#1-introduction)
2. [Design Philosophy](#2-design-philosophy)
3. [GVM Architecture Overview](#3-gvm-architecture-overview)
4. [Instruction Encoding](#4-instruction-encoding)
5. [Instruction Set Reference](#5-instruction-set-reference)
6. [Memory Model](#6-memory-model)
7. [Execution Model](#7-execution-model)
8. [Configuration & Tuning](#8-configuration--tuning)
9. [Verification & Diagnostics](#9-verification--diagnostics)
10. [Implementation Guidance](#10-implementation-guidance)
11. [Conformance Test Suite](#11-conformance-test-suite)
12. [Future Extensions](#12-future-extensions)
13. [Cross-References](#13-cross-references)
14. [Progress Summary](#14-progress-summary)

---

## 1. Introduction

### 1.1 Purpose

This RFC defines the **Grammar Virtual Machine (GVM)** — the runtime execution engine for AGOS Grammar Bytecode. The GVM takes binary bytecode (produced by MOD-09 BytecodeGenerator) and executes it deterministically to produce the final `AnalysisResult`.

The GVM is the boundary between the **Compilation Layer** (MOD-01 through MOD-09) and the **Runtime Layer** (MOD-10 and MOD-11). It is designed to be:

- **Deterministic:** Same bytecode + same config = identical output everywhere.
- **Sandboxed:** Bounded steps, bounded memory, no side effects.
- **Language-independent:** Any language can implement a GVM from this spec.
- **Minimal:** ~50 domain-specific instructions, no general-purpose computation.

### 1.2 Relationship to Other Documents

| Document | Relationship |
|----------|-------------|
| **ADR-0002** | Architectural rationale for bytecode + GVM |
| **RFC-0002** | Binary format of the bytecode consumed by the GVM |
| **SPEC-0001-C3** | MOD-10 GVM within the pipeline |
| **SPEC-0001-C4** | GVM interface (execute, verify, version) |
| **SPEC-0001-C5** | IR-9 (bytecode) → IR-10 (AnalysisResult) transformation |
| **SPEC-0001-C9** | Performance targets for GVM execution |
| **SPEC-0301** | Full Grammar Runtime specification (planned) |

### 1.3 Terms and Definitions

| Term | Definition |
|------|------------|
| **Bytecode** | The binary Grammar Bytecode (RFC-0002 format) consumed by the GVM |
| **Instruction** | A single operation in the GVM instruction set |
| **Opcode** | The numeric code identifying an instruction (0–255) |
| **GVM State** | The runtime state of the GVM during execution: registers, memory, stack, program counter |
| **AnalysisResult** | The final output produced after bytecode execution |
| **Step** | A single instruction execution cycle (fetch → decode → execute) |
| **Sandbox** | The bounded environment in which bytecode executes (no file I/O, no network, no system calls) |

---

## 2. Design Philosophy

### 2.1 Core Principles

1. **Domain-Specific, Not General-Purpose.** The GVM is designed exclusively for grammatical analysis. It has no general-purpose computation instructions (no arbitrary arithmetic, no complex control flow). Its instruction set mirrors the structure of Arabic grammatical analysis: tokens, features, constituents, rules, evidence.

2. **Determinism by Construction.** Every instruction has a well-defined effect on GVM state. There is no randomness, no external input during execution, no concurrency, no time-dependent behavior. The same bytecode + same configuration = identical output on every run, every platform, every implementation.

3. **Safety Above Performance.** The GVM enforces strict bounds on steps and memory before executing the first instruction. No instruction can crash the host process, access memory outside its allocated region, or perform I/O. Performance is optimized within these safety constraints.

4. **Verifiable at Load Time.** Before executing any instruction, the GVM verifies bytecode integrity (checksums), version compatibility, structural validity, and resource bounds. Invalid bytecode is rejected before execution begins.

5. **Traceable Execution.** The GVM can optionally produce a detailed execution trace — every instruction, its operands, its effect on state, and the time it took. This supports debugging, education, and performance profiling.

### 2.2 Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Execution model** | Stack-based interpreter | Simpler to implement correctly than register-based; easier to verify; sufficient for the instruction count and complexity |
| **Memory model** | Pre-allocated typed regions | No garbage collection needed; no memory management overhead during execution; bounds-checked at every access |
| **Instruction encoding** | Fixed 2-byte opcode + variable operands | Balance between compactness and decode speed; operands use varint encoding |
| **Step counting** | Every instruction is 1 step | Simple, predictable; fine-grained enough for profiling |
| **Tracing** | Optional compile-time feature | Zero overhead when disabled; full detail when enabled |
| **JIT compilation** | Deferred (post-v1.0) | Interpreter is simpler and sufficient for the domain; JIT can be added if profiling shows need |

---

## 3. GVM Architecture Overview

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     GVM Runtime                              │
│                                                              │
│  ┌──────────┐    ┌──────────┐    ┌───────────────────┐      │
│  │ Loader   │───►│ Verifier │───►│ Execution Engine   │      │
│  │          │    │          │    │                    │      │
│  │ • Parse  │    │ • Check  │    │ • Fetch/Decode     │      │
│  │   header │    │   version│    │ • Execute instrs   │      │
│  │ • Read   │    │ • Verify │    │ • Step counting    │      │
│  │   sections│   │   checksum│   │ • Bounds checking  │      │
│  │ • Build  │    │ • Validate│   │ • Tracing (opt)    │      │
│  │   section│    │   structure│  │                    │      │
│  │   table  │    │ • Check   │    │                    │      │
│  │          │    │   resource│    │                    │      │
│  └──────────┘    │   bounds  │    └────────┬──────────┘      │
│                  └──────────┘             │                   │
│                                           ▼                   │
│  ┌───────────────────────────────────────────────────────┐    │
│  │                  GVM State                             │    │
│  │  ┌─────────┐ ┌──────────┐ ┌────────┐ ┌────────────┐  │    │
│  │  │ Program │ │ Operand  │ │ Call    │ │ Memory     │  │    │
│  │  │ Counter │ │ Stack    │ │ Stack   │ │ Regions    │  │    │
│  │  └─────────┘ └──────────┘ └────────┘ └────────────┘  │    │
│  └───────────────────────────────────────────────────────┘    │
│                                                              │
│  Output: AnalysisResult                                       │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Input → Processing → Output

```
Input: GrammarBytecode    (RFC-0002 binary format)
    │
    ▼
┌─────────────────────────────────────────────┐
│  1. LOAD BYTECODE                           │
│     • Read header (magic, version, flags)   │
│     • Build section table                    │
│     • Map sections into memory               │
│                                             │
│  2. VERIFY                                   │
│     • Check magic == "AGOS"                  │
│     • Check version compatibility            │
│     • Verify CRC32C checksums                │
│     • Validate structural integrity          │
│     • Check resource bounds (steps, memory)  │
│                                             │
│  3. EXECUTE                                  │
│  3.1  Initialize GVM state:                  │
│       • PC = 0                               │
│       • operand_stack = empty                │
│       • call_stack = empty                   │
│       • memory regions = pre-allocated       │
│       • step_count = 0                       │
│  3.2  Instruction loop:                      │
│       • Fetch instruction at PC              │
│       • Decode opcode and operands           │
│       • Validate operands                    │
│       • Execute instruction                  │
│       • Update PC                            │
│       • Increment step_count                 │
│       • Check step_count < max_steps         │
│       • If halted → break                    │
│                                             │
│  4. PRODUCE OUTPUT                           │
│     • Collect state into AnalysisResult      │
│     • Include timing, steps, memory stats    │
└─────────────────────────────────────────────┘
    │
    ▼
Output: AnalysisResult    (IR-10)
```

### 3.3 GVM State

The GVM maintains the following state during execution:

```
GVMState = {
    // Execution control
    pc: integer,                        // Program counter (byte offset in bytecode)
    halted: boolean,                    // Execution completed
    error: GVMError | null,            // Execution error (if any)

    // Stacks
    operand_stack: Value[],             // Stack-based operand passing
    call_stack: CallFrame[],            // Call/return frames

    // Typed Memory Regions
    token_region: TokenRegion,          // Token data
    feature_region: FeatureRegion,      // Feature bitfields
    constituent_region: ConstituentRegion, // Tree structures
    string_region: StringRegion,        // Interned strings
    rule_region: RuleRegion,            // Rule application data
    evidence_region: EvidenceRegion,    // Evidence trail
    scratch_region: Bytes,             // Temporary workspace

    // Metrics
    step_count: integer,                // Instructions executed
    peak_memory_used: integer,          // Max memory used during execution
    started_at: Timestamp,              // Execution start time
}
```

### 3.4 Value Types

Values on the operand stack can be:

```
Value = integer (i32)          // 32-bit signed integer
      | integer (i64)          // 64-bit signed integer
      | float (f64)            // 64-bit IEEE 754 float
      | string_index (u32)     // Index into string table
      | token_index (u32)      // Index into token region
      | feature_bits (u64)     // Packed morphological feature bits
      | constituent_ptr (u32)  // Pointer into constituent region
      | bool                   // true or false
      | null                   // Absence of value
```

---

## 4. Instruction Encoding

### 4.1 Instruction Format

```
┌────────┬──────────┬────────────────────────────────────┐
│ Opcode │  Flags   │  Operands (variable, per opcode)   │
│ 1 byte │  1 byte  │  N bytes                            │
└────────┴──────────┴────────────────────────────────────┘
```

**Flags byte:**

```
Bit 0: HAS_IMMEDIATE   // Operand contains an immediate value
Bit 1: HAS_STRING_REF  // Operand references string table
Bit 2: HAS_TOKEN_REF   // Operand references token index
Bit 3: HAS_REGION_REF  // Operand references memory region
Bits 4-7: Reserved (must be 0)
```

### 4.2 Operand Encoding

Operands use a variable-length encoding:

| Type | Encoding | Size | Description |
|------|----------|------|-------------|
| `u32` | Varint (LEB128) | 1–5 bytes | Unsigned 32-bit integer |
| `i32` | Varint (zigzag) | 1–5 bytes | Signed 32-bit integer |
| `u64` | Varint (LEB128) | 1–10 bytes | Unsigned 64-bit integer |
| `f64` | Fixed (IEEE 754) | 8 bytes | 64-bit float |
| `string_index` | Varint | 1–5 bytes | Index into string table |
| `feature_bits` | Fixed | 8 bytes | Packed 64-bit feature field |

### 4.3 Instruction Categories

Instructions are organized into 9 categories by opcode range:

| Category | Opcode Range | Count | Description |
|----------|-------------|-------|-------------|
| **Flow Control** | 0x00–0x0F | 10 | Halt, jump, call, return, branch |
| **Stack Operations** | 0x10–0x1F | 8 | Push, pop, dup, swap, drop |
| **Token Operations** | 0x20–0x2F | 10 | Load token, token features, token index |
| **Feature Operations** | 0x30–0x3F | 10 | Feature access, comparison, bitfield ops |
| **Constituent Operations** | 0x40–0x4F | 8 | Tree loading, traversal, role assignment |
| **Rule Operations** | 0x50–0x5F | 6 | Rule application, confirmation, rejection |
| **Evidence Operations** | 0x60–0x6F | 4 | Evidence push, query, emit |
| **Output Operations** | 0x70–0x7F | 6 | Build AnalysisResult, set metadata |
| **Reserved / Extension** | 0x80–0xFF | 128 | Future instruction sets |

**Total defined:** ~62 instructions (within the ~50+ target).

---

## 5. Instruction Set Reference

### 5.1 Flow Control Instructions

#### HALT — Halt Execution

| Field | Value |
|-------|-------|
| **Opcode** | `0x00` |
| **Operands** | None |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Sets `halted = true`. Execution stops after this instruction. |
| **Errors** | None |

```
HALT
  PC → PC + 1
  halted = true
```

#### JUMP — Unconditional Jump

| Field | Value |
|-------|-------|
| **Opcode** | `0x01` |
| **Operands** | `offset: i32` — signed byte offset relative to current PC |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Sets `PC = PC + offset`. |

```
JUMP offset:
  PC = PC + offset
```

#### JUMP_IF_TRUE — Conditional Jump (true)

| Field | Value |
|-------|-------|
| **Opcode** | `0x02` |
| **Operands** | `offset: i32` — signed byte offset relative to current PC |
| **Stack** | [bool] → [] |
| **Steps** | 1 |
| **Effect** | Pops a boolean. If true, jumps to `PC + offset`. Otherwise continues to next instruction. |
| **Errors** | `TYPE_ERROR` if top of stack is not boolean |

#### JUMP_IF_FALSE — Conditional Jump (false)

| Field | Value |
|-------|-------|
| **Opcode** | `0x03` |
| **Operands** | `offset: i32` |
| **Stack** | [bool] → [] |
| **Steps** | 1 |
| **Effect** | Pops a boolean. If false, jumps to `PC + offset`. Otherwise continues. |
| **Errors** | `TYPE_ERROR` if top of stack is not boolean |

#### CALL — Call Subroutine

| Field | Value |
|-------|-------|
| **Opcode** | `0x04` |
| **Operands** | `offset: i32` — offset to target instruction |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Pushes current PC + instruction_length onto call_stack. Sets PC = offset. |
| **Errors** | `CALL_STACK_OVERFLOW` if call stack exceeds max depth (default: 64) |

#### RETURN — Return from Subroutine

| Field | Value |
|-------|-------|
| **Opcode** | `0x05` |
| **Operands** | None |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Pops return address from call_stack. Sets PC = return address. |
| **Errors** | `CALL_STACK_UNDERFLOW` if call stack is empty |

#### DIE — Fatal Error

| Field | Value |
|-------|-------|
| **Opcode** | `0x06` |
| **Operands** | `error_code: u32` — error code (from SPEC-0001-C4 GVMError codes) |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Sets `halted = true`, `error = error_code`. Execution stops immediately. |
| **Errors** | None (intentionally terminates execution) |

---

### 5.2 Stack Operations

#### PUSH_I32 — Push 32-bit Integer

| Field | Value |
|-------|-------|
| **Opcode** | `0x10` |
| **Operands** | `value: i32` — signed 32-bit integer |
| **Stack** | [] → [i32] |
| **Steps** | 1 |
| **Effect** | Pushes `value` onto operand stack. |
| **Errors** | `STACK_OVERFLOW` if stack exceeds max depth (default: 1024) |

#### PUSH_I64 — Push 64-bit Integer

| Field | Value |
|-------|-------|
| **Opcode** | `0x11` |
| **Operands** | `value: i64` — signed 64-bit integer |
| **Stack** | [] → [i64] |
| **Steps** | 1 |

#### PUSH_F64 — Push 64-bit Float

| Field | Value |
|-------|-------|
| **Opcode** | `0x12` |
| **Operands** | `value: f64` — IEEE 754 double |
| **Stack** | [] → [f64] |
| **Steps** | 1 |

#### PUSH_BOOL — Push Boolean

| Field | Value |
|-------|-------|
| **Opcode** | `0x13` |
| **Operands** | `value: u8` — 0 = false, 1 = true |
| **Stack** | [] → [bool] |
| **Steps** | 1 |

#### PUSH_STRING — Push String Reference

| Field | Value |
|-------|-------|
| **Opcode** | `0x14` |
| **Operands** | `index: u32` — index into string table |
| **Stack** | [] → [string_index] |
| **Steps** | 1 |
| **Effect** | Validates that index is within string table bounds. |
| **Errors** | `STRING_INDEX_OUT_OF_BOUNDS` if index >= string table length |

#### PUSH_NULL — Push Null

| Field | Value |
|-------|-------|
| **Opcode** | `0x15` |
| **Operands** | None |
| **Stack** | [] → [null] |
| **Steps** | 1 |

#### POP — Discard Top of Stack

| Field | Value |
|-------|-------|
| **Opcode** | `0x16` |
| **Operands** | None |
| **Stack** | [any] → [] |
| **Steps** | 1 |
| **Errors** | `STACK_UNDERFLOW` if stack is empty |

#### DUP — Duplicate Top of Stack

| Field | Value |
|-------|-------|
| **Opcode** | `0x17` |
| **Operands** | None |
| **Stack** | [a] → [a, a] |
| **Steps** | 1 |
| **Errors** | `STACK_OVERFLOW` if stack is full |

#### SWAP — Swap Top Two Values

| Field | Value |
|-------|-------|
| **Opcode** | `0x18` |
| **Operands** | None |
| **Stack** | [a, b] → [b, a] |
| **Steps** | 1 |
| **Errors** | `STACK_UNDERFLOW` if stack has < 2 elements |

---

### 5.3 Token Operations

#### LOAD_TOKEN — Load Token Data

| Field | Value |
|-------|-------|
| **Opcode** | `0x20` |
| **Operands** | `token_index: u32` — index of token in token region |
| **Stack** | [] → [token_index] |
| **Steps** | 1 |
| **Effect** | Validates token index. Pushes the token reference onto the stack. |
| **Errors** | `TOKEN_INDEX_OUT_OF_BOUNDS` if token_index >= token_region.length |

#### TOKEN_GET_TEXT — Get Token Text

| Field | Value |
|-------|-------|
| **Opcode** | `0x21` |
| **Operands** | None |
| **Stack** | [token_index] → [string_index] |
| **Steps** | 1 |
| **Effect** | Pops token_index. Pushes the string index of the token's text. |
| **Errors** | `TYPE_ERROR` if top is not a token_index |

#### TOKEN_GET_OFFSET — Get Token Offsets

| Field | Value |
|-------|-------|
| **Opcode** | `0x22` |
| **Operands** | None |
| **Stack** | [token_index] → [u32, u32] |
| **Steps** | 1 |
| **Effect** | Pushes start_offset and end_offset (byte offsets in original text). |

#### TOKEN_GET_TYPE — Get Token Type

| Field | Value |
|-------|-------|
| **Opcode** | `0x23` |
| **Operands** | None |
| **Stack** | [token_index] → [u32] |
| **Steps** | 1 |
| **Effect** | Pushes numeric token type ID (0=word, 1=punctuation, 2=number, 3=whitespace, 4=symbol, 5=unknown). |

#### TOKEN_GET_FEATURES — Get Token Features

| Field | Value |
|-------|-------|
| **Opcode** | `0x24` |
| **Operands** | None |
| **Stack** | [token_index] → [feature_bits] |
| **Steps** | 1 |
| **Effect** | Pops token_index. Pushes the packed feature bitfield for this token. |

#### TOKEN_ITERATE — Iterate Over Tokens

| Field | Value |
|-------|-------|
| **Opcode** | `0x25` |
| **Operands** | `offset: i32` — offset to loop body (backward jump target) |
| **Stack** | [u32, u32] → [u32, u32] |
| **Steps** | 2 |
| **Effect** | Expects [current_index, total_count] on stack. If current_index < total_count, pushes [current_index+1, total_count] and continues. Otherwise, falls through. The `offset` points to the instruction after the loop body (backward jump). |
| **Errors** | `TYPE_ERROR` if top values are not integers |

Example loop:

```
// Setup: push start and end indices
PUSH_I32 0           // current = 0
PUSH_I32 N           // total = N
// Loop header
DUP                  // ... total total
DUP                  // ... total total total
TOKEN_ITERATE loop_end
  // loop body — stack is [current, total]
  ...
  JUMP loop_header
loop_end:
POP                  // discard total
POP                  // discard current
```

#### TOKEN_COUNT — Get Token Count

| Field | Value |
|-------|-------|
| **Opcode** | `0x26` |
| **Operands** | None |
| **Stack** | [] → [u32] |
| **Steps** | 1 |
| **Effect** | Pushes the total number of tokens in the token region. |

---

### 5.4 Feature Operations

#### FEATURE_GET — Get Feature Value

| Field | Value |
|-------|-------|
| **Opcode** | `0x30` |
| **Operands** | `feature_id: u32` — feature ID from KB-0007 taxonomy |
| **Stack** | [feature_bits] → [value: u32] |
| **Steps** | 2 |
| **Effect** | Pops feature_bits. Extracts and pushes the value for the specified feature ID. Feature IDs and their bit positions are defined in the feature taxonomy (KB-0007). |
| **Errors** | `INVALID_FEATURE_ID` if feature_id is not in the taxonomy |

#### FEATURE_SET — Set Feature Value

| Field | Value |
|-------|-------|
| **Opcode** | `0x31` |
| **Operands** | `feature_id: u32` |
| **Stack** | [feature_bits, value: u32] → [feature_bits] |
| **Steps** | 1 |
| **Effect** | Pops value and feature_bits. Returns new feature_bits with the specified feature set to the given value. |

#### FEATURE_HAS — Check Feature Presence

| Field | Value |
|-------|-------|
| **Opcode** | `0x32` |
| **Operands** | `feature_id: u32` |
| **Stack** | [feature_bits] → [bool] |
| **Steps** | 1 |
| **Effect** | Pops feature_bits. Returns true if the feature has a non-null value. |

#### FEATURE_COMPARE_EQ — Feature Equality

| Field | Value |
|-------|-------|
| **Opcode** | `0x33` |
| **Operands** | `feature_id: u32` |
| **Stack** | [feature_bits_a, feature_bits_b] → [bool] |
| **Steps** | 1 |
| **Effect** | Compares the specified feature on two feature bitfields. Returns true if both have the same value. |

#### FEATURE_COMPARE_MASK — Feature Mask Comparison

| Field | Value |
|-------|-------|
| **Opcode** | `0x34` |
| **Operands** | `mask: u64` — bitmask of features to compare |
| **Stack** | [feature_bits_a, feature_bits_b] → [bool] |
| **Steps** | 2 |
| **Effect** | Compares multiple features at once using a mask. Returns true if all masked bits match. |

#### FEATURE_PACK — Pack Features from Stack

| Field | Value |
|-------|-------|
| **Opcode** | `0x35` |
| **Operands** | `count: u32` — number of (feature_id, value) pairs to pop |
| **Stack** | [fid_0, val_0, fid_1, val_1, ..., fid_n-1, val_n-1] → [feature_bits] |
| **Steps** | count |
| **Effect** | Pops `count` pairs of (feature_id, value) and packs them into a single feature_bits value. |

---

### 5.5 Constituent Operations

#### CONST_MAKE — Create Constituent

| Field | Value |
|-------|-------|
| **Opcode** | `0x40` |
| **Operands** | `role_id: u32` — syntactic role ID from SPEC-0001-C3 role table |
| **Stack** | [u32, child_ptr_0, child_ptr_1, ..., child_ptr_n-1] → [constituent_ptr] |
| **Steps** | 2 + n (n = number of children) |
| **Effect** | Pops a count `n`, then pops `n` child constituent pointers. Creates a new constituent node with the specified role and children. Pushes pointer to the new constituent. |
| **Errors** | `CONSTITUENT_REGION_FULL` if region capacity exceeded |

#### CONST_ADD_CHILD — Add Child Constituent

| Field | Value |
|-------|-------|
| **Opcode** | `0x41` |
| **Operands** | None |
| **Stack** | [constituent_ptr, child_ptr] → [constituent_ptr] |
| **Steps** | 1 |
| **Effect** | Adds child constituent to parent. Parent pointer remains on stack. |
| **Errors** | `INVALID_CONSTITUENT_PTR` if either pointer is invalid |

#### CONST_GET_CHILD — Get Child Constituent

| Field | Value |
|-------|-------|
| **Opcode** | `0x42` |
| **Operands** | `child_index: u32` |
| **Stack** | [constituent_ptr] → [child_ptr] |
| **Steps** | 1 |
| **Effect** | Gets the child at the specified index. |
| **Errors** | `CHILD_INDEX_OUT_OF_BOUNDS` if child_index >= child count |

#### CONST_GET_ROLE — Get Constituent Role

| Field | Value |
|-------|-------|
| **Opcode** | `0x43` |
| **Operands** | None |
| **Stack** | [constituent_ptr] → [u32] |
| **Steps** | 1 |
| **Effect** | Pushes the numeric role ID of the constituent. |

#### CONST_SET_ROLE — Set Constituent Role

| Field | Value |
|-------|-------|
| **Opcode** | `0x44` |
| **Operands** | None |
| **Stack** | [constituent_ptr, u32] → [constituent_ptr] |
| **Steps** | 1 |
| **Effect** | Sets the role of the constituent to the specified role ID. |

#### CONST_ATTACH_TOKENS — Attach Token Indices

| Field | Value |
|-------|-------|
| **Opcode** | `0x45` |
| **Operands** | `count: u32` — number of token indices to attach |
| **Stack** | [constituent_ptr, u32, ..., u32] → [constituent_ptr] |
| **Steps** | count |
| **Effect** | Pops `count` token indices and attaches them to the constituent. |

#### CONST_TRAVERSE — Traverse Constituent Tree

| Field | Value |
|-------|-------|
| **Opcode** | `0x46` |
| **Operands** | `offset: i32` — offset to traversal callback |
| **Stack** | [constituent_ptr] → [constituent_ptr] |
| **Steps** | Varies (depth of tree) |
| **Effect** | Performs a depth-first traversal of the constituent tree. For each node, pushes the node pointer and jumps to the callback. The callback should process the node and RETURN. After the callback returns, traversal continues to the next node. |

---

### 5.6 Rule Operations

#### RULE_APPLY — Apply Rule

| Field | Value |
|-------|-------|
| **Opcode** | `0x50` |
| **Operands** | `rule_id: u32`, `rule_name_index: u32` (string table index) |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Records a rule application in the rule region. The rule is identified by its numeric ID. The rule name is a string table reference. The current PC and the currently-active token index (tracked internally by the GVM via LOAD_TOKEN) are associated with the rule. The stack is not modified — rule context is drawn from GVM internal state. |

#### RULE_CONFIRM — Confirm Analysis

| Field | Value |
|-------|-------|
| **Opcode** | `0x51` |
| **Operands** | `analysis_index: u32` |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Marks the analysis at the given index as confirmed by the most recent RULE_APPLY. |

#### RULE_REJECT — Reject Analysis

| Field | Value |
|-------|-------|
| **Opcode** | `0x52` |
| **Operands** | `analysis_index: u32` |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Marks the analysis at the given index as rejected by the most recent RULE_APPLY. |

#### RULE_MODIFY — Modify Feature

| Field | Value |
|-------|-------|
| **Opcode** | `0x53` |
| **Operands** | `feature_id: u32` |
| **Stack** | [feature_bits, value: u32] → [feature_bits] |
| **Steps** | 1 |
| **Effect** | Modifies the specified feature on the feature bitfield and records the modification in the rule region. The modification is associated with the most recent RULE_APPLY. |
| **Errors** | `INVALID_FEATURE_ID` if feature_id is not in the taxonomy |

#### RULE_FLAG — Raise Grammatical Flag

| Field | Value |
|-------|-------|
| **Opcode** | `0x54` |
| **Operands** | `flag_type: u32` (0=error, 1=warning, 2=info), `flag_code_index: u32` (string table index) |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Raises a grammatical flag associated with the most recent RULE_APPLY. Flags are collected and returned in the AnalysisResult. |

#### RULE_RESOLVE — Resolve Anaphora

| Field | Value |
|-------|-------|
| **Opcode** | `0x55` |
| **Operands** | None |
| **Stack** | [token_index_antecedent, token_index_pronoun] → [] |
| **Steps** | 1 |
| **Effect** | Records a pronoun resolution: the pronoun at token_index_pronoun refers to the antecedent at token_index_antecedent. |

---

### 5.7 Evidence Operations

#### EVIDENCE_PUSH — Add Evidence Entry

| Field | Value |
|-------|-------|
| **Opcode** | `0x60` |
| **Operands** | `stage_name_index: u32` (string table), `algorithm_index: u32` (string table) |
| **Stack** | [confidence: f64] → [] |
| **Steps** | 2 |
| **Effect** | Creates a new evidence entry with the given stage name and algorithm name. The current token index (from context) and confidence (from stack) are recorded. |

#### EVIDENCE_QUERY — Query Evidence Count

| Field | Value |
|-------|-------|
| **Opcode** | `0x61` |
| **Operands** | None |
| **Stack** | [] → [u32] |
| **Steps** | 1 |
| **Effect** | Pushes the total number of evidence entries accumulated so far. |

#### EVIDENCE_EMIT — Emit Evidence to Output

| Field | Value |
|-------|-------|
| **Opcode** | `0x62` |
| **Operands** | None |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Marks all accumulated evidence for inclusion in the AnalysisResult. |

---

### 5.8 Output Operations

#### OUTPUT_SET_METADATA — Set Output Metadata

| Field | Value |
|-------|-------|
| **Opcode** | `0x70` |
| **Operands** | `key_index: u32` (string table index), `value_index: u32` (string table index) |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Sets a metadata key-value pair on the output. E.g., sentence_type, school, confidence. |

#### OUTPUT_ADD_TREE — Add Analysis Tree

| Field | Value |
|-------|-------|
| **Opcode** | `0x71` |
| **Operands** | `tree_type_index: u32` (string table index) |
| **Stack** | [constituent_ptr, confidence: f64] → [] |
| **Steps** | 2 |
| **Effect** | Adds a new analysis tree (from the constituent pointer) to the output. The tree type and confidence are recorded. |

#### OUTPUT_ADD_TOKEN — Add Token to Output

| Field | Value |
|-------|-------|
| **Opcode** | `0x72` |
| **Operands** | None |
| **Stack** | [token_index, feature_bits, role_id: u32] → [] |
| **Steps** | 1 |
| **Effect** | Adds a token (with its features and syntactic role) to the output tree's token list. |

#### OUTPUT_SET_INPUT — Set Input Text

| Field | Value |
|-------|-------|
| **Opcode** | `0x73` |
| **Operands** | `text_index: u32` (string table index), `hash_index: u32` (string table index for SHA-256) |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Sets the input text and its SHA-256 hash on the AnalysisResult. |

#### OUTPUT_FINALIZE — Finalize Output

| Field | Value |
|-------|-------|
| **Opcode** | `0x74` |
| **Operands** | None |
| **Stack** | [] → [] |
| **Steps** | 1 |
| **Effect** | Finalizes the AnalysisResult: computes totals, sorts trees by confidence, collects all flags and evidence. After this instruction, the output is complete and cannot be modified. |

---

### 5.9 Reserved Opcodes

Opcodes `0x80` through `0xFF` are reserved for future extensions. Any GVM implementation MUST reject bytecode containing reserved opcodes with `UNSUPPORTED_OPCODE` unless the implementation explicitly supports the extension (and documents which extended opcodes it supports).

---

## 6. Memory Model

### 6.1 Memory Regions

The GVM uses pre-allocated typed memory regions. Each region has a fixed capacity declared in the bytecode header:

```
GVM_Memory = {
    token_region: {
        capacity: u32,              // Max tokens
        size: u32,                   // Current token count
        tokens: Token[],             // Token data
    },
    feature_region: {
        capacity: u32,              // Max feature bitfields
        size: u32,                   // Current count
        features: u64[],            // Packed feature bitfields
    },
    constituent_region: {
        capacity: u32,              // Max constituent nodes
        size: u32,                   // Current count
        nodes: ConstituentNode[],   // Constituent tree nodes
    },
    string_region: {
        strings: String[],           // Immutable string table (from bytecode)
    },
    rule_region: {
        capacity: u32,              // Max rule applications
        size: u32,                   // Current count
        applications: RuleApplication[],
    },
    evidence_region: {
        capacity: u32,              // Max evidence entries
        size: u32,                   // Current count
        entries: EvidenceEntry[],
    },
    scratch_region: {
        capacity: u32,              // Scratch buffer size
        data: u8[],                  // Raw byte buffer
        cursor: u32,                 // Current write position
    },
}
```

### 6.2 Memory Allocation

Memory regions are allocated at GVM initialization time based on the bytecode header's declared capacities:

| Region | Default Capacity | Size per Entry | Notes |
|--------|-----------------|----------------|-------|
| Token | 256 | 32–64 bytes | Grows with document size |
| Feature | 512 | 8 bytes | 64-bit bitfield per feature set |
| Constituent | 1024 | 48 bytes | Tree node structure |
| String | — | Variable | Immutable; from bytecode string table |
| Rule | 512 | 32 bytes | Rule application record |
| Evidence | 1024 | 64 bytes | Evidence entry |
| Scratch | 4096 | 1 byte | Temporary workspace |

### 6.3 Bounds Checking

Every memory access is bounds-checked:

| Access Type | Check | Error on Violation |
|-------------|-------|-------------------|
| Token read/write | `index < token_region.size` | `TOKEN_INDEX_OUT_OF_BOUNDS` |
| Feature read/write | `index < feature_region.size` | `FEATURE_INDEX_OUT_OF_BOUNDS` |
| Constituent read/write | `index < constituent_region.size` | `CONSTITUENT_INDEX_OUT_OF_BOUNDS` |
| String read | `index < string_region.strings.length` | `STRING_INDEX_OUT_OF_BOUNDS` |
| Scratch read/write | `cursor + length < scratch_region.capacity` | `SCRATCH_OVERFLOW` |

### 6.4 Memory Safety Guarantees

1. **No pointer arithmetic.** All memory access is through typed indices (u32), not raw pointers. There is no way to construct an arbitrary memory address.
2. **No dynamic allocation during execution.** All memory is pre-allocated. The "allocation" operations just increment counters within capacity limits.
3. **No out-of-bounds writes.** Every write is bounds-checked.
4. **No use-after-free.** There is no deallocation during execution. Memory is only freed when the GVM instance is destroyed.

---

## 7. Execution Model

### 7.1 Instruction Cycle

The GVM uses a classic fetch-decode-execute cycle:

```
function execute_cycle():
    while not halted and step_count < max_steps:
        // 1. FETCH
        instruction_bytes = bytecode[pc:pc+max_instruction_length]
        opcode = instruction_bytes[0]
        flags = instruction_bytes[1]

        // 2. DECODE
        instruction = instruction_table[opcode]
        operands = decode_operands(instruction_bytes[2:], instruction.operand_types)

        // 3. VALIDATE
        validate_operand_types(operands, instruction)
        validate_operand_bounds(operands, memory_regions)

        // 4. EXECUTE
        execute_instruction(opcode, operands)

        // 5. UPDATE STATE
        step_count += 1
        if tracing_enabled:
            record_trace(pc, opcode, operands, stack_depth, memory_usage)

    if step_count >= max_steps:
        error = MAX_STEPS_EXCEEDED
```

### 7.2 Execution Guarantees

| Guarantee | Mechanism | Configuration |
|-----------|-----------|---------------|
| **Bounded steps** | Step counter incremented per instruction; halted when >= limit | `max_execution_steps` (default: 100,000) |
| **Bounded memory** | Pre-allocated regions with fixed capacities; no dynamic allocation | `max_memory` (default: 64 MB) |
| **Deterministic output** | No randomness, no external state, fixed instruction order | Not configurable |
| **Side-effect free** | No file I/O, network, system calls, or external process interaction | Not configurable |
| **No infinite loops** | Step limit ensures termination; bytecode cannot disable the step counter | Not configurable |

### 7.3 Error States

When an error is encountered during execution, the GVM transitions to the error state:

```
Execution States:
┌─────────┐     ┌──────────┐     ┌──────────┐
│  LOADED │────►│ VERIFIED │────►│ EXECUTING│
└─────────┘     └──────────┘     └────┬─────┘
                                       │
                          ┌────────────┼────────────┐
                          ▼            ▼            ▼
                     ┌────────┐  ┌──────────┐  ┌────────┐
                     │ HALTED │  │  ERROR   │  │ TIMEOUT│
                     │ (ok)   │  │          │  │        │
                     └────────┘  └──────────┘  └────────┘
```

| State | Description |
|-------|-------------|
| **LOADED** | Bytecode loaded but not yet verified |
| **VERIFIED** | Bytecode passed all verification checks |
| **EXECUTING** | Instructions are being executed |
| **HALTED** | Execution completed successfully (HALT instruction reached) |
| **ERROR** | Execution terminated due to an error (DIE or runtime error) |
| **TIMEOUT** | Execution terminated because step limit was reached |

### 7.4 Error Codes

The following error codes can be returned by the GVM (from SPEC-0001-C4 and extended):

| Code | Description | Fatal | Recovery |
|------|-------------|-------|----------|
| `UNSUPPORTED_BYTECODE_VERSION` | Bytecode version exceeds GVM version | Yes | Update GVM or regenerate bytecode |
| `BYTECODE_CORRUPTED` | Bytecode failed integrity check (CRC32C) | Yes | Regenerate bytecode |
| `MAX_STEPS_EXCEEDED` | Execution exceeded step limit | Yes | Increase `max_execution_steps` |
| `MAX_MEMORY_EXCEEDED` | Execution exceeded memory limit | Yes | Increase `max_memory` or simplify input |
| `EXECUTION_FAILURE` | Unrecoverable execution error | Yes | Report bug |
| `STACK_OVERFLOW` | Operand stack exceeded max depth | Yes | Simplify analysis; report to bytecode generator |
| `STACK_UNDERFLOW` | Operand stack empty when a value was needed | Yes | Bug in bytecode generator |
| `TYPE_ERROR` | Value on stack has unexpected type | Yes | Bug in bytecode generator |
| `INVALID_OPCODE` | Unknown opcode encountered | Yes | Update GVM or regenerate bytecode |
| `TOKEN_INDEX_OUT_OF_BOUNDS` | Token index exceeds token region | Yes | Bug in bytecode generator |
| `STRING_INDEX_OUT_OF_BOUNDS` | String index exceeds string table | Yes | Bug in bytecode generator |
| `FEATURE_INDEX_OUT_OF_BOUNDS` | Feature index exceeds feature region | Yes | Bug in bytecode generator |
| `CONSTITUENT_INDEX_OUT_OF_BOUNDS` | Constituent index exceeds region | Yes | Bug in bytecode generator |
| `CALL_STACK_OVERFLOW` | Call stack exceeded max depth (64) | Yes | Simplify rule nesting |
| `CALL_STACK_UNDERFLOW` | Return with empty call stack | Yes | Bug in bytecode generator |
| `INVALID_FEATURE_ID` | Feature ID not in taxonomy | Yes | Update bytecode generator |
| `SCRATCH_OVERFLOW` | Scratch buffer write exceeds capacity | Yes | Increase scratch capacity in bytecode |
| `DIVISION_BY_ZERO` | Division by zero (reserved, not used in initial set) | Yes | Bug in bytecode generator |
| `INTERNAL_ERROR` | Unexpected GVM implementation failure | Yes | Report bug |

---

## 8. Configuration & Tuning

### 8.1 Execution Configuration

The GVM accepts the following configuration parameters:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `max_execution_steps` | u32 | 100,000 | Maximum number of instructions to execute |
| `max_memory` | u32 | 67,108,864 (64 MiB) | Maximum memory allocation (bytes) |
| `sandbox_mode` | bool | true | Enforce sandbox (must be true in production) |
| `tracing_enabled` | bool | false | Collect execution trace |
| `max_call_depth` | u32 | 64 | Maximum call/return nesting depth |
| `max_stack_depth` | u32 | 1024 | Maximum operand stack depth |

### 8.2 Performance Profiles

| Profile | Description | Use Case |
|---------|-------------|----------|
| **interactive** | Strict latency budget (< 10 ms). Step limit: 10,000. No tracing. | Real-time analysis |
| **server** | Balanced throughput. Step limit: 100,000. No tracing. | Web API |
| **batch** | Higher throughput, no latency constraints. Step limit: 1,000,000. No tracing. | Corpus analysis |
| **debug** | Full tracing enabled. Step limit: 10,000,000. Trace output captures all instructions. | Debugging rule sets |
| **educational** | Tracing enabled with human-readable output. Step limit: 50,000. | Teaching Arabic grammar |

### 8.3 Performance Targets

From SPEC-0001-C9:

| Metric | Target | Profile |
|--------|--------|---------|
| Latency (p50) | < 1 ms per sentence | Interactive, Server |
| Latency (p99) | < 5 ms per sentence | Interactive, Server |
| Throughput | > 2K sentences/s (single core) | Server |
| Execution speed | > 100K instructions/s | All profiles |
| Cache hit latency | < 0.5 μs | All profiles |

---

## 9. Verification & Diagnostics

### 9.1 Bytecode Verification

Before executing, the GVM verifies the bytecode:

```
function verify(bytecode: GrammarBytecode) -> VerificationResult:
    issues = []

    // 1. Magic bytes
    if bytecode.magic != "AGOS":
        issues.add({ severity: "error", code: "INVALID_MAGIC", ... })

    // 2. Version compatibility
    if bytecode.version.major > GVM_VERSION.major:
        issues.add({ severity: "error", code: "VERSION_TOO_HIGH", ... })
    if bytecode.version.major == GVM_VERSION.major && bytecode.version.minor > GVM_VERSION.minor:
        issues.add({ severity: "warning", code: "VERSION_HIGHER_MINOR", ... })

    // 3. Checksum verification
    for each section in bytecode.sections:
        expected_crc = section.header.crc32c
        actual_crc = crc32c(section.data)
        if expected_crc != actual_crc:
            issues.add({ severity: "error", code: "CHECKSUM_MISMATCH", ... })

    // 4. Structural validation
    validate_section_ordering(bytecode.sections)        // Must be: header, metadata, tokens, ..., end
    validate_instruction_stream(bytecode.instructions)   // All opcodes known, operands valid
    validate_jump_targets(bytecode.instructions)         // All jump offsets point to valid instructions
    validate_string_table(bytecode.string_table)         // All strings valid UTF-8

    // 5. Resource bounds check
    token_count = bytecode.header.token_count
    tree_count = bytecode.header.tree_count
    rule_count = bytecode.header.rule_count
    // Check against configured limits
    if token_count * TOKEN_SIZE > max_memory:
        issues.add({ severity: "error", code: "MEMORY_BUDGET_EXCEEDED", ... })

    // 6. Feature ID validation
    for each feature_id referenced in instructions:
        if not in_feature_taxonomy(feature_id):
            issues.add({ severity: "warning", code: "UNKNOWN_FEATURE_ID", ... })

    return { valid: issues.has_no_errors, issues: issues }
```

### 9.2 Tracing Output

When tracing is enabled, the GVM produces a trace for every instruction:

```
TraceEntry = {
    instruction_number: u32,             // Sequential instruction number
    pc: u32,                              // Program counter at time of execution
    opcode: u8,                           // Instruction opcode
    opcode_name: string,                  // Human-readable instruction name
    operands: string[],                   // Operand values (human-readable)
    stack_before: Value[],                // Operand stack before execution (top 5)
    stack_after: Value[],                 // Operand stack after execution (top 5)
    memory_delta: {                       // Memory changes from this instruction
        region: string,
        index: u32,
        old_value: string,
        new_value: string,
    } | null,
    wall_time_ns: u64,                    // Execution time in nanoseconds
}
```

### 9.3 Disassembler

The GVM SHOULD include a disassembler that converts bytecode to human-readable instruction listings:

```
# Example disassembly of a simple rule application bytecode
0x0000  PUSH_I32        0                # current = 0
0x0002  PUSH_I32        N                # total = N
0x0004  TOKEN_ITERATE   0x0014           # loop over tokens
0x0006  LOAD_TOKEN      stack[-1]        # load current token
0x0008  TOKEN_GET_FEATURES               # get token features
0x000A  FEATURE_HAS     FEATURE_GENDER   # does gender exist?
0x000C  JUMP_IF_FALSE   0x0012           # skip if no gender
0x000E  FEATURE_COMPARE_EQ FEATURE_GENDER # masculine?
0x0010  RULE_APPLY      RULE_100, "subject-verb-agreement"
0x0012  POP                              # discard token index
0x0014  JUMP            0x0002           # loop back
0x0016  POP                              # discard total
0x0018  POP                              # discard current
0x001A  OUTPUT_FINALIZE
0x001C  HALT
```

### 9.4 Command-Line Interface

The GVM implementation SHOULD expose the following CLI commands:

```
# Execute bytecode
agos gvm run --bytecode=analysis.agos --output=result.json

# Verify bytecode without executing
agos gvm verify --bytecode=analysis.agos

# Disassemble bytecode to human-readable listing
agos gvm disassemble --bytecode=analysis.agos

# Execute with tracing
agos gvm run --bytecode=analysis.agos --trace=trace.json

# Run conformance tests
agos gvm test --suite=conformance-v1
```

---

## 10. Implementation Guidance

### 10.1 Recommended Implementation Order

1. **Implement the GVM loader and verifier first.** These have no complex logic and establish the foundation. Parse the bytecode, verify checksums, validate structure.

2. **Implement the stack and memory model.** Create the operand stack, call stack, and typed memory regions. Implement bounds checking.

3. **Implement the instruction decoder.** Create the opcode dispatch table. Start with the simplest instructions: HALT, PUSH_I32, POP, DUP.

4. **Implement flow control instructions.** JUMP, JUMP_IF_TRUE, JUMP_IF_FALSE, CALL, RETURN. These enable basic program structure.

5. **Implement token and feature operations.** LOAD_TOKEN, TOKEN_GET_FEATURES, FEATURE_GET, FEATURE_COMPARE_EQ. These are the core data access instructions.

6. **Implement constituent operations.** CONST_MAKE, CONST_ADD_CHILD, CONST_TRAVERSE. These build the analysis trees.

7. **Implement rule and evidence operations.** RULE_APPLY, RULE_CONFIRM, EVIDENCE_PUSH.

8. **Implement output operations.** OUTPUT_ADD_TREE, OUTPUT_ADD_TOKEN, OUTPUT_FINALIZE. These produce the AnalysisResult.

9. **Implement the tracer and disassembler.** These are non-essential but invaluable for debugging.

10. **Write the conformance test suite.** Test every instruction, every error code, and every edge case.

### 10.2 Language-Specific Considerations

#### Rust (Primary Implementation)

```rust
// Core GVM trait (simplified)
pub trait GrammarVirtualMachine {
    fn execute(&mut self, bytecode: &GrammarBytecode, config: &GVMConfig) -> Result<AnalysisResult, GVMError>;
    fn verify(&self, bytecode: &GrammarBytecode) -> VerificationResult;
    fn version(&self) -> GVMVersion;
}

// Memory-safe approach: use Rust's type system for region types
struct GVMState {
    pc: usize,
    operand_stack: Vec<Value>,
    call_stack: Vec<usize>,
    token_region: Vec<Token>,
    feature_region: Vec<u64>,
    constituent_region: Vec<ConstituentNode>,
    // ... etc
}
```

#### C (Secondary Implementation)

- Use tagged unions for `Value` type.
- Use fixed-size arrays (not dynamic allocation) for memory regions.
- Implement bounds checking with `assert()` or explicit `if` checks.
- Use `setjmp`/`longjmp` for error handling (or explicit error propagation).

#### Python (Ecosystem Implementation)

- Use `dataclasses` for GVM state and `Value` types.
- Python's dynamic typing means type errors are detected at runtime.
- Performance will be 10–100× slower than Rust; suitable for prototyping and education.

#### JavaScript/TypeScript (Ecosystem Implementation)

- Use TypedArrays (`Uint8Array`, `BigUint64Array`) for memory regions.
- The event loop can yield between instruction cycles for responsive tracing UIs.
- WASM-based GVM can be compiled from Rust for browser use.

### 10.3 Common Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| **Off-by-one in jump offsets** | Unit-test every JUMP instruction; validate all jump targets during verification |
| **Stack underflow in complex expressions** | Clearly document stack effects for every instruction; test with minimal examples |
| **Endianness mismatch** | Encode bytecode as little-endian; test cross-platform (x86 vs ARM vs WASM) |
| **Varint decoding errors** | Use a well-tested LEB128 library; test with boundary values (0, 127, 128, 16383, 16384) |
| **Infinite loop detection** | Always check step counter before every instruction fetch, even for single-cycle instructions |
| **Race conditions in multi-threaded use** | Each GVM instance is single-threaded. For concurrency, use one GVM per thread. |

---

## 11. Conformance Test Suite

### 11.1 Overview

Every GVM implementation MUST pass the AGOS GVM Conformance Test Suite. The test suite consists of:

- **Test bytecodes** (`.agos` files) with known expected outputs (`.json` files).
- **Test runner** that executes each bytecode and compares the output against the expected result.
- **Test categories** covering every instruction, error condition, and edge case.

### 11.2 Test Categories

| Category | Tests | Coverage |
|----------|-------|----------|
| **Flow Control** | 20 | HALT, JUMP forward/backward, conditional jumps, CALL/RETURN, DIE |
| **Stack Operations** | 15 | PUSH all types, POP, DUP, SWAP, PUSH_NULL |
| **Token Operations** | 15 | LOAD_TOKEN (valid/invalid), TOKEN_COUNT, TOKEN_ITERATE, TOKEN_GET_TEXT |
| **Feature Operations** | 20 | FEATURE_GET/SET/HAS, FEATURE_COMPARE_EQ/MASK, FEATURE_PACK |
| **Constituent Operations** | 15 | CONST_MAKE, CONST_ADD_CHILD, CONST_GET_CHILD, CONST_TRAVERSE |
| **Rule Operations** | 10 | RULE_APPLY, RULE_CONFIRM, RULE_REJECT, RULE_MODIFY |
| **Evidence Operations** | 8 | EVIDENCE_PUSH, EVIDENCE_QUERY, EVIDENCE_EMIT |
| **Output Operations** | 10 | OUTPUT_SET_METADATA, OUTPUT_ADD_TREE, OUTPUT_FINALIZE |
| **Bounds Checking** | 25 | Every bounds-check error condition |
| **Error Handling** | 20 | Every error code |
| **Cross-Implementation** | 5 | Same bytecode produces identical output across Rust, C, Python, JS, Swift |

**Total:** ~163 conformance tests.

### 11.3 Test Bytecode Format

Each test is a minimal bytecode file with a paired expected output:

```
# tests/conformance/stack/push_i32.agos  (binary bytecode)
# tests/conformance/stack/push_i32.json  (expected output)

{
    "test_name": "push_i32",
    "description": "PUSH_I32 with value 42, then HALT",
    "config": { "max_execution_steps": 100 },
    "expected": {
        "status": "completed",
        "metadata": {
            "steps_executed": 2
        }
    }
}
```

### 11.4 Test Runner

```bash
# Run all conformance tests
agos gvm test --suite=conformance-v1

# Run specific category
agos gvm test --suite=conformance-v1 --category=stack

# Run a single test
agos gvm test --bytecode=tests/conformance/stack/push_i32.agos

# Generate conformance report
agos gvm test --suite=conformance-v1 --report=conformance.html

# Check cross-implementation consistency
agos gvm test --cross-impl --impls=rust,c,python,js,swift
```

---

## 12. Future Extensions

### 12.1 Planned Extensions (Post-v1.0)

| Extension | Description | Opcode Range |
|-----------|-------------|-------------|
| **JIT Compilation** | Compile hot instruction sequences to native code | No new opcodes |
| **Serialization Instructions** | Direct read/write of serialized data structures | 0x80–0x8F |
| **Statistical Analysis** | Aggregation operations (count, sum, mean per feature) | 0x90–0x9F |
| **Corpus Comparison** | Compare two AnalysisResults for diff/similarity | 0xA0–0xA7 |
| **Plugin Call Interface** | Invoke external plugin functions from bytecode | 0xA8–0xAF |
| **Dynamic String Operations** | String concatenation, substring at runtime | 0xB0–0xBF |
| **Advanced Traversal** | Breadth-first, post-order constituent traversal | 0xC0–0xC7 |
| **Debugging Instructions** | Breakpoint, step-over markers for educational tools | 0xC8–0xCF |

### 12.2 Version Compatibility Policy

| Bytecode Version | GVM Version | Compatibility |
|------------------|-------------|---------------|
| 0.x (experimental) | 0.x | May break at any time |
| 1.0 (stable) | 1.0.x | Full forward/backward compatible within major version |
| 1.x | 1.y (y >= x) | Backward compatible (newer GVM runs older bytecode) |
| 2.0 | 2.0+ | Breaking changes documented in migration guide |

### 12.3 Backward Compatibility Requirements

1. A GVM version `M.x` MUST execute any bytecode version `M.y` where `y <= x`.
2. When executing older bytecode, the GVM MUST produce the same output as the older GVM version would have.
3. New instructions in newer bytecode versions MUST NOT change the semantics of existing instructions.
4. Deprecated instructions MUST continue to be supported for at least 2 major versions.

---

## 13. Cross-References

### 13.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| ADR-0002 | Why Grammar Bytecode | Architectural rationale for GVM |
| SPEC-0001-C3 | Compilation Pipeline (MOD-09, MOD-10) | GVM within the pipeline |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | GVM interface (execute, verify, version) |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | IR-9 → GVM → IR-10 |
| SPEC-0001-C9 | Performance Targets | GVM latency/throughput targets |
| RFC-0002 | Grammar Bytecode Format | Binary format consumed by GVM |
| SPEC-0301 | Grammar Runtime | Full runtime specification (planned) |

### 13.2 External References

| Reference | Relevance |
|-----------|-----------|
| JVM Specification (Java 8) | Inspiration for stack-based VM design and verification |
| WebAssembly Specification | Inspiration for sandboxing and deterministic execution |
| Lua 5.4 VM | Inspiration for minimal instruction set and register-based design |
| CPython Bytecode | Inspiration for instruction set design patterns |
| LEB128 Encoding (DWARF) | Variable-length integer encoding used for operands |
| IEEE 754 | Floating-point representation for confidence values |

---

## 14. Progress Summary

**RFC-0003: Grammar Virtual Machine (GVM) — Instruction Set & Execution Model**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction | ✓ COMPLETE |
| 2 | Design Philosophy | ✓ COMPLETE (5 principles, 7 decisions) |
| 3 | GVM Architecture Overview | ✓ COMPLETE |
| 4 | Instruction Encoding | ✓ COMPLETE |
| 5 | Instruction Set Reference | ✓ COMPLETE (~57 instructions defined) |
| 6 | Memory Model | ✓ COMPLETE |
| 7 | Execution Model | ✓ COMPLETE |
| 8 | Configuration & Tuning | ✓ COMPLETE |
| 9 | Verification & Diagnostics | ✓ COMPLETE |
| 10 | Implementation Guidance | ✓ COMPLETE |
| 11 | Conformance Test Suite | ✓ COMPLETE (~163 tests defined) |
| 12 | Future Extensions | ✓ COMPLETE |
| 13 | Cross-References | ✓ COMPLETE |

**Dependencies:** ADR-0002, RFC-0002, SPEC-0001-C3, SPEC-0001-C4, SPEC-0001-C5, SPEC-0001-C9.

**Recommended next document:** RFC-0002 — Grammar Bytecode Format (the binary format specification that the GVM consumes), or ADR-0003 — Why Grammar IR.
