# SPEC-0302: GVM Instruction Set — Instruction-Level Reference

| **Field** | **Value** |
|---|---|
| **Spec ID** | SPEC-0302 |
| **Title** | GVM Instruction Set — Instruction-Level Reference for Implementers |
| **Version** | 1.0.0 |
| **Status** | Draft |
| **Depends on** | RFC-0003 (GVM Architecture & Execution Model), RFC-0002 (Bytecode Format), KB-0007 (Feature Taxonomy), SPEC-0301 (Grammar Runtime) |
| **Related SPECs** | SPEC-0102 (Feature Taxonomy Reference), SPEC-0201 (Rule Engine), SPEC-0501 (Explanation Engine) |
| **Related RFCs** | RFC-0001 (Grammar DSL), RFC-0004 (Arabic Grammar Rule DSL) |
| **License** | AGOS Specification License v1.0 |

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Instruction Encoding Reference](#2-instruction-encoding-reference)
3. [Opcode Table & Dispatch Reference](#3-opcode-table--dispatch-reference)
4. [Flow Control Instructions (0x00–0x0F)](#4-flow-control-instructions-0x00-0x0f)
5. [Stack Operations (0x10–0x1F)](#5-stack-operations-0x10-0x1f)
6. [Token Operations (0x20–0x2F)](#6-token-operations-0x20-0x2f)
7. [Feature Operations (0x30–0x3F)](#7-feature-operations-0x30-0x3f)
8. [Constituent Operations (0x40–0x4F)](#8-constituent-operations-0x40-0x4f)
9. [Rule Operations (0x50–0x5F)](#9-rule-operations-0x50-0x5f)
10. [Evidence Operations (0x60–0x6F)](#10-evidence-operations-0x60-0x6f)
11. [Output Operations (0x70–0x7F)](#11-output-operations-0x70-0x7f)
12. [Reserved Opcodes & Extension (0x80–0xFF)](#12-reserved-opcodes--extension-0x80-0xff)
13. [Common Instruction Sequences](#13-common-instruction-sequences)
14. [Instruction Cost Model](#14-instruction-cost-model)
15. [Error Code Reference](#15-error-code-reference)
16. [Feature ID Reference for Instructions](#16-feature-id-reference-for-instructions)
17. [Cross-References](#17-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0302 provides the **instruction-level reference** for the Grammar Virtual Machine (GVM). Where RFC-0003 defines the architecture, execution model, and design philosophy of the GVM, this specification focuses exclusively on the instructions themselves — their encoding, semantics, stack effects, error conditions, and usage patterns.

This document serves as:

- **The canonical reference** for every GVM instruction — opcode, mnemonic, encoding, and behavior.
- **The implementer's handbook** for building a GVM — covering the instruction dispatch table, operand decoding, stack management, and error handling.
- **The bytecode generator's guide** for MOD-09 developers — showing how to emit correct instruction sequences for common patterns.
- **The debugger's companion** for understanding disassembly output and execution traces.

### 1.2 Relationship to RFC-0003

| Aspect | RFC-0003 | SPEC-0302 (this document) |
|--------|----------|--------------------------|
| **Focus** | Architecture, execution model, memory model, verification | Instruction encoding, stack effects, error conditions, sequences |
| **Audience** | System architects, VM designers | VM implementers, bytecode generator developers, debuggers |
| **Content** | 57 instructions described in ~10 pages | Each instruction detailed with encoding, pseudocode, errors, and examples |
| **Format** | High-level tables with effect descriptions | Per-instruction reference cards + categorized listings |
| **Extension** | Reserved opcodes mentioned briefly | Full extension strategy with opcode slot allocation plan |

### 1.3 Scope

**In scope:**

| Category | Coverage |
|----------|----------|
| **Instruction encoding** | Opcode byte, flags byte, operand encoding per instruction |
| **Instruction semantics** | Precise stack effects, state changes, memory region modifications |
| **Error conditions** | Every error code each instruction can produce, with preconditions |
| **Feature IDs** | Complete mapping of feature IDs used in FEATURE_GET/SET/COMPARE instructions |
| **Instruction sequences** | Common patterns (token loop, feature comparison, tree building) |
| **Cost model** | Step counts, estimated latency, and memory impact per instruction |
| **Opcode table** | Complete 256-entry dispatch table indexed by opcode |
| **Extension slots** | Reserved opcodes with planned allocation for future instruction sets |

**Out of scope:**

| Topic | Covered By |
|-------|-----------|
| Memory region layout and management | RFC-0003 §6, SPEC-0301 §6 |
| Bytecode binary format and section layout | RFC-0002 |
| GVM lifecycle and configuration | RFC-0003 §8, SPEC-0301 §4 |
| Execution tracing and diagnostics | RFC-0003 §9, SPEC-0301 §7 |
| Feature taxonomy definitions | KB-0007, SPEC-0102 |
| Bytecode generation algorithms | SPEC-0001-C3 (MOD-09) |

### 1.4 Conventions

- **Opcodes** are shown in hexadecimal: `0x00`, `0x30`, etc.
- **Stack notation**: `[before] → [after]`. Top of stack is rightmost.
- **`u32`**: unsigned 32-bit integer. **`i32`**: signed 32-bit integer.
- **PC**: Program counter (byte offset in instruction stream).
- **Feature IDs** are `u32` constants that map to KB-0007 feature positions.
- **String indices** are `u32` references into the bytecode's string table.
- **Token indices** are `u32` references into the token region.

---

## 2. Instruction Encoding Reference

### 2.1 Instruction Format

Every instruction is encoded as a variable-length sequence:

```
┌──────────┬──────────┬─────────────────────────────────────────┐
│ Opcode   │ Flags    │ Operands (variable, per opcode)         │
│ 1 byte   │ 1 byte   │ N bytes (0–12 typical, up to 32 max)   │
└──────────┴──────────┴─────────────────────────────────────────┘
```

**Minimum instruction size:** 2 bytes (opcode + flags, no operands — e.g., `HALT`, `POP`).

**Maximum instruction size:** 34 bytes (opcode + flags + 32 bytes of operands — e.g., `PUSH_F64` with an 8-byte float plus `FEATURE_PACK` with many pairs).

### 2.2 Flags Byte

```
Bit    Name             Description
───    ────             ───────────
0      HAS_IMMEDIATE    Operand contains an immediate value
1      HAS_STRING_REF   Operand references the string table
2      HAS_TOKEN_REF    Operand references a token index
3      HAS_REGION_REF   Operand references a memory region
4–7    RESERVED         Must be zero (rejected by verifier if set)
```

The flags byte is used by the GVM decoder to optimize operand parsing. For example, if `HAS_STRING_REF` is set, the decoder knows to parse the next varint as a string table index and validate it accordingly.

### 2.3 Operand Encoding Types

| Encoding ID | Type | Size | Encoding Method | Instructions Using It |
|-------------|------|------|-----------------|----------------------|
| `u32` | Unsigned varint | 1–5 bytes | LEB128 | JUMP offset, PUSH_I32, LOAD_TOKEN, TOKEN_ITERATE |
| `i32` | Signed varint | 1–5 bytes | Zigzag LEB128 | Jump offsets, counts |
| `u64` | Fixed | 8 bytes | Little-endian u64 | FEATURE_COMPARE_MASK (mask) |
| `f64` | Fixed | 8 bytes | IEEE 754, LE | PUSH_F64 |
| `u8` | Fixed | 1 byte | Raw byte | PUSH_BOOL, token type |
| `string` | Varint-prefixed | Variable | Length + UTF-8 | Metadata entries |
| `string_index` | Varint (u32) | 1–5 bytes | LEB128 | PUSH_STRING, RULE_APPLY, EVIDENCE_PUSH |

### 2.4 Operand Validation Rules

Every operand MUST be validated before instruction execution:

| Operand Type | Validation | Error on Violation |
|-------------|------------|-------------------|
| `string_index` | `< string_region.size` | `STRING_INDEX_OUT_OF_BOUNDS` |
| `token_index` | `< token_region.size` | `TOKEN_INDEX_OUT_OF_BOUNDS` |
| `feature_id` | `≤ 18` (0–18 are defined) | `INVALID_FEATURE_ID` |
| `constituent_ptr` | `< constituent_region.size` | `CONSTITUENT_INDEX_OUT_OF_BOUNDS` |
| `confidence (f64)` | `0.0 ≤ value ≤ 1.0` | `TYPE_ERROR` |

---

## 3. Opcode Table & Dispatch Reference

### 3.1 Complete Opcode Map

```
Range       Category            Defined   Reserved   Total Slots
─────       ────────            ───────   ────────   ──────────
0x00–0x0F   Flow Control        7         9          16
0x10–0x1F   Stack Operations    9         7          16
0x20–0x2F   Token Operations    7         9          16
0x30–0x3F   Feature Operations  6         10         16
0x40–0x4F   Constituent Ops     7         9          16
0x50–0x5F   Rule Operations     6         10         16
0x60–0x6F   Evidence Operations 3         13         16
0x70–0x7F   Output Operations   5         11         16
0x80–0xFF   Reserved/Extension  0         128        128
───────     ────────            ───────   ────────   ──────────
Total                           50        206        256
```

**Total defined instructions:** 50 (within the ~50+ target from RFC-0003).
**Available for future extension:** 206 opcode slots (128 in reserved range + 78 in category ranges).

### 3.2 Dispatch Table (256 entries)

The GVM dispatch table maps opcodes to handler functions. Unused entries MUST return `INVALID_OPCODE`.

```rust
/// Opcode dispatch table. 256 entries indexed by opcode.
/// None = invalid opcode; returns INVALID_OPCODE error.
const OPCODE_TABLE: [Option<InstructionDef>; 256] = {
    // ── Flow Control (0x00–0x0F) ──────────────────────────
    0x00: Some(instruction("HALT",           0, &[],          handle_halt)),
    0x01: Some(instruction("JUMP",           1, &[Operand::I32], handle_jump)),
    0x02: Some(instruction("JUMP_IF_TRUE",   1, &[Operand::I32], handle_jump_if_true)),
    0x03: Some(instruction("JUMP_IF_FALSE",  1, &[Operand::I32], handle_jump_if_false)),
    0x04: Some(instruction("CALL",           1, &[Operand::I32], handle_call)),
    0x05: Some(instruction("RETURN",         0, &[],          handle_return)),
    0x06: Some(instruction("DIE",            1, &[Operand::U32], handle_die)),
    0x07–0x0F: None,  // Reserved

    // ── Stack Operations (0x10–0x1F) ──────────────────────
    0x10: Some(instruction("PUSH_I32",       1, &[Operand::I32], handle_push_i32)),
    0x11: Some(instruction("PUSH_I64",       1, &[Operand::I64], handle_push_i64)),
    0x12: Some(instruction("PUSH_F64",       1, &[Operand::F64], handle_push_f64)),
    0x13: Some(instruction("PUSH_BOOL",      1, &[Operand::U8],  handle_push_bool)),
    0x14: Some(instruction("PUSH_STRING",    1, &[Operand::U32], handle_push_string)),
    0x15: Some(instruction("PUSH_NULL",      0, &[],           handle_push_null)),
    0x16: Some(instruction("POP",            0, &[],           handle_pop)),
    0x17: Some(instruction("DUP",            0, &[],           handle_dup)),
    0x18: Some(instruction("SWAP",           0, &[],           handle_swap)),
    0x19–0x1F: None,  // Reserved

    // ── Token Operations (0x20–0x2F) ──────────────────────
    0x20: Some(instruction("LOAD_TOKEN",     1, &[Operand::U32], handle_load_token)),
    0x21: Some(instruction("TOKEN_GET_TEXT", 0, &[],           handle_token_get_text)),
    0x22: Some(instruction("TOKEN_GET_OFFSET",0, &[],          handle_token_get_offset)),
    0x23: Some(instruction("TOKEN_GET_TYPE", 0, &[],           handle_token_get_type)),
    0x24: Some(instruction("TOKEN_GET_FEATURES", 0, &[],       handle_token_get_features)),
    0x25: Some(instruction("TOKEN_ITERATE",  1, &[Operand::I32], handle_token_iterate)),
    0x26: Some(instruction("TOKEN_COUNT",    0, &[],           handle_token_count)),
    0x27–0x2F: None,  // Reserved

    // ── Feature Operations (0x30–0x3F) ────────────────────
    0x30: Some(instruction("FEATURE_GET",          1, &[Operand::U32], handle_feature_get)),
    0x31: Some(instruction("FEATURE_SET",          1, &[Operand::U32], handle_feature_set)),
    0x32: Some(instruction("FEATURE_HAS",          1, &[Operand::U32], handle_feature_has)),
    0x33: Some(instruction("FEATURE_COMPARE_EQ",   1, &[Operand::U32], handle_feature_compare_eq)),
    0x34: Some(instruction("FEATURE_COMPARE_MASK", 1, &[Operand::U64], handle_feature_compare_mask)),
    0x35: Some(instruction("FEATURE_PACK",         1, &[Operand::U32], handle_feature_pack)),
    0x36–0x3F: None,  // Reserved

    // ── Constituent Operations (0x40–0x4F) ────────────────
    0x40: Some(instruction("CONST_MAKE",         1, &[Operand::U32], handle_const_make)),
    0x41: Some(instruction("CONST_ADD_CHILD",    0, &[],           handle_const_add_child)),
    0x42: Some(instruction("CONST_GET_CHILD",    1, &[Operand::U32], handle_const_get_child)),
    0x43: Some(instruction("CONST_GET_ROLE",     0, &[],           handle_const_get_role)),
    0x44: Some(instruction("CONST_SET_ROLE",     0, &[],           handle_const_set_role)),
    0x45: Some(instruction("CONST_ATTACH_TOKENS",1, &[Operand::U32], handle_const_attach_tokens)),
    0x46: Some(instruction("CONST_TRAVERSE",     1, &[Operand::I32], handle_const_traverse)),
    0x47–0x4F: None,  // Reserved

    // ── Rule Operations (0x50–0x5F) ───────────────────────
    0x50: Some(instruction("RULE_APPLY",      2, &[Operand::U32, Operand::U32], handle_rule_apply)),
    0x51: Some(instruction("RULE_CONFIRM",    1, &[Operand::U32],              handle_rule_confirm)),
    0x52: Some(instruction("RULE_REJECT",     1, &[Operand::U32],              handle_rule_reject)),
    0x53: Some(instruction("RULE_MODIFY",     1, &[Operand::U32],              handle_rule_modify)),
    0x54: Some(instruction("RULE_FLAG",       2, &[Operand::U32, Operand::U32], handle_rule_flag)),
    0x55: Some(instruction("RULE_RESOLVE",    0, &[],                          handle_rule_resolve)),
    0x56–0x5F: None,  // Reserved

    // ── Evidence Operations (0x60–0x6F) ───────────────────
    0x60: Some(instruction("EVIDENCE_PUSH",   2, &[Operand::U32, Operand::U32], handle_evidence_push)),
    0x61: Some(instruction("EVIDENCE_QUERY",  0, &[],           handle_evidence_query)),
    0x62: Some(instruction("EVIDENCE_EMIT",   0, &[],           handle_evidence_emit)),
    0x63–0x6F: None,  // Reserved

    // ── Output Operations (0x70–0x7F) ─────────────────────
    0x70: Some(instruction("OUTPUT_SET_METADATA", 2, &[Operand::U32, Operand::U32], handle_output_set_metadata)),
    0x71: Some(instruction("OUTPUT_ADD_TREE",     1, &[Operand::U32],              handle_output_add_tree)),
    0x72: Some(instruction("OUTPUT_ADD_TOKEN",    0, &[],                           handle_output_add_token)),
    0x73: Some(instruction("OUTPUT_SET_INPUT",    2, &[Operand::U32, Operand::U32], handle_output_set_input)),
    0x74: Some(instruction("OUTPUT_FINALIZE",     0, &[],                           handle_output_finalize)),
    0x75–0x7F: None,  // Reserved

    // ── Reserved/Extension (0x80–0xFF): all None ──────────
};
```

### 3.3 Mnemonic Aliases

For disassembly and diagnostics, each instruction has a unique mnemonic:

| Opcode | Mnemonic | Category |
|--------|----------|----------|
| `0x00` | `HALT` | Flow Control |
| `0x01` | `JUMP` | Flow Control |
| `0x02` | `JEQ` | Flow Control (jump-if-true) |
| `0x03` | `JNE` | Flow Control (jump-if-false) |
| `0x04` | `CALL` | Flow Control |
| `0x05` | `RET` | Flow Control |
| `0x06` | `DIE` | Flow Control |
| `0x10` | `PUSH` | Stack (32-bit) |
| `0x11` | `PUSH64` | Stack (64-bit) |
| `0x12` | `PUSHF` | Stack (float) |
| `0x13` | `PUSHB` | Stack (boolean) |
| `0x14` | `PUSHS` | Stack (string index) |
| `0x15` | `PUSHN` | Stack (null) |
| `0x16` | `POP` | Stack |
| `0x17` | `DUP` | Stack |
| `0x18` | `SWAP` | Stack |
| `0x20` | `LDTOK` | Token |
| `0x21` | `TOKTXT` | Token |
| `0x22` | `TOKOFF` | Token |
| `0x23` | `TOKTYP` | Token |
| `0x24` | `TOKFEAT` | Token |
| `0x25` | `TOKITR` | Token |
| `0x26` | `TOKCNT` | Token |
| `0x30` | `FGET` | Feature |
| `0x31` | `FSET` | Feature |
| `0x32` | `FHAS` | Feature |
| `0x33` | `FCMP` | Feature |
| `0x34` | `FCMPM` | Feature |
| `0x35` | `FPACK` | Feature |
| `0x40` | `CMAKE` | Constituent |
| `0x41` | `CADD` | Constituent |
| `0x42` | `CGET` | Constituent |
| `0x43` | `CROLE` | Constituent |
| `0x44` | `CSROLE` | Constituent |
| `0x45` | `CATOK` | Constituent |
| `0x46` | `CTRAV` | Constituent |
| `0x50` | `RAPPLY` | Rule |
| `0x51` | `RCONF` | Rule |
| `0x52` | `RREJ` | Rule |
| `0x53` | `RMOD` | Rule |
| `0x54` | `RFLAG` | Rule |
| `0x55` | `RRES` | Rule |
| `0x60` | `EPUSH` | Evidence |
| `0x61` | `EQUERY` | Evidence |
| `0x62` | `EEMIT` | Evidence |
| `0x70` | `OSMETA` | Output |
| `0x71` | `OATREE` | Output |
| `0x72` | `OATOK` | Output |
| `0x73` | `OSIN` | Output |
| `0x74` | `OFIN` | Output |

---

## 4. Flow Control Instructions (0x00–0x0F)

### 4.1 HALT (0x00)

```
HALT — Halt execution

Encoding:  0x00 0x00
Size:      2 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    state.halted = true
    // PC is not advanced; execution stops immediately
    // after this instruction is processed

Errors:    None
```

**Usage:** Every bytecode MUST end with HALT (or DIE). The GVM verifier checks that the last instruction in the instruction section is HALT or DIE.

**Example (disassembly):**
```
0x002A  HALT
```

### 4.2 JUMP (0x01)

```
JUMP — Unconditional jump

Encoding:  0x01 0x00 <offset: i32 (zigzag)>
Size:      2–6 bytes (2 + varint size)
Stack:     [] → []
Steps:     1

Pseudocode:
    state.pc = state.pc + offset
    // offset is relative to the START of this instruction

Errors:
    JUMP_OUT_OF_BOUNDS — if new PC is outside instruction section bounds

Validation:
    The verifier MUST check that the target PC points to a valid
    instruction boundary. Jumping into the middle of an operand
    or past the end of the instruction section is invalid.
```

**Usage:** Used for loop back-edges, forward skips, and else-branches in if-then-else patterns.

**Example (disassembly):**
```
0x0010  JUMP +20          # Jump forward 20 bytes to 0x0024
0x0012  ...               # Skipped code
0x0024  HALT

0x0100  JUMP -30          # Jump backward 30 bytes (loop)
```

### 4.3 JUMP_IF_TRUE (0x02) / JEQ

```
JUMP_IF_TRUE — Conditional jump if true

Encoding:  0x02 0x00 <offset: i32 (zigzag)>
Size:      2–6 bytes
Stack:     [bool] → []
Steps:     1

Pseudocode:
    let condition = state.operand_stack.pop()
    if condition.type != Bool:
        error(TYPE_ERROR)
    if condition.value == true:
        state.pc = state.pc + offset
    // else: fall through to next instruction

Errors:
    TYPE_ERROR — if top of stack is not boolean
    JUMP_OUT_OF_BOUNDS — if target PC is outside instruction section
```

**Mnemonic:** `JEQ` (jump-if-equal-to-true).

### 4.4 JUMP_IF_FALSE (0x03) / JNE

```
JUMP_IF_FALSE — Conditional jump if false

Encoding:  0x03 0x00 <offset: i32 (zigzag)>
Size:      2–6 bytes
Stack:     [bool] → []
Steps:     1

Pseudocode:
    let condition = state.operand_stack.pop()
    if condition.type != Bool:
        error(TYPE_ERROR)
    if condition.value == false:
        state.pc = state.pc + offset
    // else: fall through

Errors:
    TYPE_ERROR — if top of stack is not boolean
    JUMP_OUT_OF_BOUNDS — if target PC is outside instruction section
```

**Mnemonic:** `JNE` (jump-if-not-equal-to-true, i.e., jump-if-false).

### 4.5 CALL (0x04)

```
CALL — Call subroutine

Encoding:  0x04 0x00 <offset: i32 (zigzag)>
Size:      2–6 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    let return_addr = state.pc + instruction_length
    state.call_stack.push(return_addr)
    if state.call_stack.size > state.config.max_call_depth:
        error(CALL_STACK_OVERFLOW)
    state.pc = state.pc + offset

Errors:
    CALL_STACK_OVERFLOW — if call stack exceeds max depth (default: 64)
    JUMP_OUT_OF_BOUNDS — if target PC is outside instruction section
```

### 4.6 RETURN (0x05) / RET

```
RETURN — Return from subroutine

Encoding:  0x05 0x00
Size:      2 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    if state.call_stack.is_empty():
        error(CALL_STACK_UNDERFLOW)
    state.pc = state.call_stack.pop()

Errors:
    CALL_STACK_UNDERFLOW — if call stack is empty
```

### 4.7 DIE (0x06)

```
DIE — Fatal error termination

Encoding:  0x06 0x00 <error_code: u32 (varint)>
Size:      2–6 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    state.halted = true
    state.error = decode_error_code(error_code)
    // Execution stops immediately

Errors:    None (intentionally terminates execution)

Used for: Flagging unreachable code, explicit error handling
```

---

## 5. Stack Operations (0x10–0x1F)

### 5.1 PUSH_I32 (0x10) / PUSH

```
PUSH_I32 — Push 32-bit signed integer

Encoding:  0x10 0x00 <value: i32 (zigzag)>
Size:      2–6 bytes
Stack:     [] → [i32]
Steps:     1

Pseudocode:
    if state.operand_stack.size >= state.config.max_stack_depth:
        error(STACK_OVERFLOW)
    state.operand_stack.push(Value::I32(value))

Errors:
    STACK_OVERFLOW — if stack depth exceeds max (default: 1024)
```

**Mnemonic:** `PUSH` (the default for 32-bit integer pushes).

### 5.2 PUSH_I64 (0x11) / PUSH64

```
PUSH_I64 — Push 64-bit signed integer

Encoding:  0x11 0x00 <value: i64 (8 bytes, LE)>
Size:      10 bytes (fixed)
Stack:     [] → [i64]
Steps:     1

Errors:
    STACK_OVERFLOW — if stack is full
```

**Mnemonic:** `PUSH64`.

### 5.3 PUSH_F64 (0x12) / PUSHF

```
PUSH_F64 — Push 64-bit IEEE 754 float

Encoding:  0x12 0x00 <value: f64 (8 bytes, LE)>
Size:      10 bytes (fixed)
Stack:     [] → [f64]
Steps:     1

Errors:
    STACK_OVERFLOW — if stack is full
```

**Mnemonic:** `PUSHF`.

### 5.4 PUSH_BOOL (0x13) / PUSHB

```
PUSH_BOOL — Push boolean value

Encoding:  0x13 0x00 <value: u8>
Size:      3 bytes (fixed)
Stack:     [] → [bool]
Steps:     1

Value encoding:
    0x00 → false
    0x01 → true
    Any other value → TYPE_ERROR

Errors:
    STACK_OVERFLOW — if stack is full
    TYPE_ERROR — if value is not 0x00 or 0x01
```

**Mnemonic:** `PUSHB`.

### 5.5 PUSH_STRING (0x14) / PUSHS

```
PUSH_STRING — Push string table index

Encoding:  0x14 0x00 <index: u32 (varint)>
Size:      2–6 bytes
Stack:     [] → [string_index]
Steps:     1

Validation:
    index < state.string_region.size
    Otherwise: STRING_INDEX_OUT_OF_BOUNDS

Errors:
    STACK_OVERFLOW — if stack is full
    STRING_INDEX_OUT_OF_BOUNDS — if index >= string table size
```

**Mnemonic:** `PUSHS`.

### 5.6 PUSH_NULL (0x15) / PUSHN

```
PUSH_NULL — Push null value

Encoding:  0x15 0x00
Size:      2 bytes
Stack:     [] → [null]
Steps:     1

Errors:
    STACK_OVERFLOW — if stack is full
```

**Mnemonic:** `PUSHN`.

### 5.7 POP (0x16)

```
POP — Discard top of stack

Encoding:  0x16 0x00
Size:      2 bytes
Stack:     [any] → []
Steps:     1

Errors:
    STACK_UNDERFLOW — if stack is empty
```

### 5.8 DUP (0x17)

```
DUP — Duplicate top of stack

Encoding:  0x17 0x00
Size:      2 bytes
Stack:     [a] → [a, a]
Steps:     1

Errors:
    STACK_UNDERFLOW — if stack is empty
    STACK_OVERFLOW — if stack is full after duplication
```

### 5.9 SWAP (0x18)

```
SWAP — Swap top two values

Encoding:  0x18 0x00
Size:      2 bytes
Stack:     [a, b] → [b, a]
Steps:     1

Errors:
    STACK_UNDERFLOW — if stack has < 2 elements
```

---

## 6. Token Operations (0x20–0x2F)

### 6.1 LOAD_TOKEN (0x20) / LDTOK

```
LOAD_TOKEN — Load a token by index

Encoding:  0x20 0x00 <token_index: u32 (varint)>
Size:      2–6 bytes
Stack:     [] → [token_index]
Steps:     1

Pseudocode:
    let idx = decode_varint(operands[0])
    if idx >= state.token_region.size:
        error(TOKEN_INDEX_OUT_OF_BOUNDS)
    state.current_token_index = idx
    state.operand_stack.push(Value::TokenIndex(idx))

Errors:
    TOKEN_INDEX_OUT_OF_BOUNDS — if token_index >= token_region.size
    STACK_OVERFLOW — if stack is full

Side effects:
    Sets state.current_token_index for use by subsequent token operations.
```

**Mnemonic:** `LDTOK`.

### 6.2 TOKEN_GET_TEXT (0x21) / TOKTXT

```
TOKEN_GET_TEXT — Get token text as string index

Encoding:  0x21 0x00
Size:      2 bytes
Stack:     [token_index] → [string_index]
Steps:     1

Pseudocode:
    let tok_idx = state.operand_stack.pop()
    if tok_idx.type != TokenIndex:
        error(TYPE_ERROR)
    let token = state.token_region.get(tok_idx.value)
    state.operand_stack.push(Value::StringIndex(token.text_index))

Errors:
    TYPE_ERROR — if top is not a token_index
    STRING_INDEX_OUT_OF_BOUNDS — if token's text_index is invalid
```

**Mnemonic:** `TOKTXT`.

### 6.3 TOKEN_GET_OFFSET (0x22) / TOKOFF

```
TOKEN_GET_OFFSET — Get token byte offsets

Encoding:  0x22 0x00
Size:      2 bytes
Stack:     [token_index] → [u32, u32]
Steps:     1

Pseudocode:
    let tok_idx = state.operand_stack.pop()
    if tok_idx.type != TokenIndex:
        error(TYPE_ERROR)
    let token = state.token_region.get(tok_idx.value)
    state.operand_stack.push(Value::I32(token.start_offset as i32))
    state.operand_stack.push(Value::I32(token.end_offset as i32))

Errors:
    TYPE_ERROR — if top is not a token_index
```

**Mnemonic:** `TOKOFF`.

### 6.4 TOKEN_GET_TYPE (0x23) / TOKTYP

```
TOKEN_GET_TYPE — Get token type code

Encoding:  0x23 0x00
Size:      2 bytes
Stack:     [token_index] → [u32]
Steps:     1

Token types:
    0 = word         1 = punctuation   2 = number
    3 = whitespace   4 = symbol        5 = unknown

Errors:
    TYPE_ERROR — if top is not a token_index
```

**Mnemonic:** `TOKTYP`.

### 6.5 TOKEN_GET_FEATURES (0x24) / TOKFEAT

```
TOKEN_GET_FEATURES — Get token's packed feature bitfield

Encoding:  0x24 0x00
Size:      2 bytes
Stack:     [token_index] → [feature_bits]
Steps:     1

Pseudocode:
    let tok_idx = state.operand_stack.pop()
    if tok_idx.type != TokenIndex:
        error(TYPE_ERROR)
    let token = state.token_region.get(tok_idx.value)
    if token.feature_index == NULL_FEATURE_REF:
        // No features — push 0 (all-unspecified bitfield)
        state.operand_stack.push(Value::FeatureBits(0))
    else:
        let bits = state.feature_region.get(token.feature_index)
        state.operand_stack.push(Value::FeatureBits(bits))

Errors:
    TYPE_ERROR — if top is not a token_index
    FEATURE_INDEX_OUT_OF_BOUNDS — if token's feature ref is invalid
```

**Mnemonic:** `TOKFEAT`.

### 6.6 TOKEN_ITERATE (0x25) / TOKITR

```
TOKEN_ITERATE — Loop over token range

Encoding:  0x25 0x00 <offset: i32 (zigzag)>
Size:      2–6 bytes
Stack:     [u32, u32] → [u32, u32]
Steps:     2

Pre-stack:  [current_index, total_count] (current on top)
Post-stack: [current_index+1, total_count] (if looping)
            OR [current_index, total_count] (if falling through)

Pseudocode:
    let total = state.operand_stack.pop()     // total_count
    let current = state.operand_stack.pop()   // current_index
    if current.type != I32 || total.type != I32:
        error(TYPE_ERROR)
    if current.value < total.value:
        // Continue loop: push incremented index and jump back
        state.operand_stack.push(Value::I32(current.value + 1))
        state.operand_stack.push(Value::I32(total.value))
        state.pc = state.pc + offset
    else:
        // Fall through
        state.operand_stack.push(Value::I32(current.value))
        state.operand_stack.push(Value::I32(total.value))

Errors:
    TYPE_ERROR — if top two values are not i32
    JUMP_OUT_OF_BOUNDS — if target PC is outside instruction section
```

**Mnemonic:** `TOKITR`.

**Common token loop pattern:**
```
// Setup
PUSH_I32 0          // current = 0
TOKEN_COUNT         // push total count
                    // stack: [0, total]

// Loop header
loop_start:
  DUP               // total (for comparison)
  DUP               // total (for comparison)
  TOKITR loop_end   // if current >= total, fall through

  // Loop body — stack: [current, total]
  LOAD_TOKEN        // load current token
  // ... process token ...

  JUMP loop_start   // back to loop header

loop_end:
  POP               // discard total
  POP               // discard current
```

### 6.7 TOKEN_COUNT (0x26) / TOKCNT

```
TOKEN_COUNT — Get total token count

Encoding:  0x26 0x00
Size:      2 bytes
Stack:     [] → [u32]
Steps:     1

Pseudocode:
    state.operand_stack.push(
        Value::I32(state.token_region.size as i32))

Errors:
    STACK_OVERFLOW — if stack is full
```

**Mnemonic:** `TOKCNT`.

---

## 7. Feature Operations (0x30–0x3F)

### 7.1 FEATURE_GET (0x30) / FGET

```
FEATURE_GET — Extract a feature value from bitfield

Encoding:  0x30 <feature_id: u32 (varint)>
Size:      2–6 bytes
Stack:     [feature_bits] → [value: u32]
Steps:     2

feature_id mapping (from KB-0007):
    0  = pos             10 = verb_form
    1  = gender          11 = noun_type
    2  = number          12 = pronoun_type
    3  = person          13 = transitivity
    4  = tense           14 = root_type
    5  = mood            15 = stress_pattern
    6  = voice           16 = syllable_count
    7  = case            17 = has_shadda
    8  = state           18 = has_madd
    9  = (reserved)      19 = has_hamza

Pseudocode:
    let bits = state.operand_stack.pop()
    if bits.type != FeatureBits:
        error(TYPE_ERROR)
    let fid = decode_feature_id(operands[0])
    if fid > 19:
        error(INVALID_FEATURE_ID)
    let value = extract_feature(bits.value, fid)
    state.operand_stack.push(Value::I32(value as i32))

Errors:
    TYPE_ERROR — if top is not a feature_bits
    INVALID_FEATURE_ID — if feature_id > 19 (or otherwise unknown)
```

**Mnemonic:** `FGET`.

**Feature extraction implementation (internal):**
```rust
fn extract_feature(bits: u64, feature_id: u32) -> u32 {
    match feature_id {
        0  => ((bits >> 0)  & 0xF) as u32,    // pos
        1  => ((bits >> 4)  & 0x3) as u32,    // gender
        2  => ((bits >> 6)  & 0x3) as u32,    // number
        3  => ((bits >> 8)  & 0x3) as u32,    // person
        4  => ((bits >> 10) & 0x3) as u32,    // tense
        5  => ((bits >> 12) & 0x3) as u32,    // mood
        6  => ((bits >> 14) & 0x1) as u32,    // voice
        7  => ((bits >> 15) & 0x3) as u32,    // case
        8  => ((bits >> 17) & 0x1) as u32,    // state
        10 => ((bits >> 18) & 0x1F) as u32,   // verb_form
        11 => ((bits >> 23) & 0x1F) as u32,   // noun_type
        12 => ((bits >> 28) & 0xF) as u32,    // pronoun_type
        13 => ((bits >> 32) & 0xF) as u32,    // transitivity
        14 => ((bits >> 36) & 0xF) as u32,    // root_type
        15 => ((bits >> 40) & 0x7) as u32,    // stress_pattern
        16 => ((bits >> 43) & 0xF) as u32,    // syllable_count
        17 => ((bits >> 47) & 0x1) as u32,    // has_shadda
        18 => ((bits >> 48) & 0x1) as u32,    // has_madd
        19 => ((bits >> 49) & 0x1) as u32,    // has_hamza
        _  => 0,  // Reserved/unknown — should not be reached
    }
}
```

### 7.2 FEATURE_SET (0x31) / FSET

```
FEATURE_SET — Set a feature value in bitfield

Encoding:  0x31 <feature_id: u32 (varint)>
Size:      2–6 bytes
Stack:     [feature_bits, value: u32] → [feature_bits]
Steps:     1

Pseudocode:
    let new_val = state.operand_stack.pop()
    let bits = state.operand_stack.pop()
    if bits.type != FeatureBits || new_val.type != I32:
        error(TYPE_ERROR)
    let fid = decode_feature_id(operands[0])
    if fid > 19:
        error(INVALID_FEATURE_ID)
    let new_bits = set_feature(bits.value, fid, new_val.value as u64)
    state.operand_stack.push(Value::FeatureBits(new_bits))

Errors:
    TYPE_ERROR — if top is not feature_bits or value is not i32
    INVALID_FEATURE_ID — if feature_id > 19
```

**Feature set implementation (internal):**
```rust
fn set_feature(bits: u64, feature_id: u32, value: u64) -> u64 {
    let (shift, mask) = FEATURE_MASK[feature_id as usize];
    (bits & !(mask << shift)) | ((value & mask) << shift)
}
```

### 7.3 FEATURE_HAS (0x32) / FHAS

```
FEATURE_HAS — Check if a feature has a non-default value

Encoding:  0x32 <feature_id: u32 (varint)>
Size:      2–6 bytes
Stack:     [feature_bits] → [bool]
Steps:     1

Pseudocode:
    let bits = state.operand_stack.pop()
    if bits.type != FeatureBits:
        error(TYPE_ERROR)
    let fid = decode_feature_id(operands[0])
    if fid > 19:
        error(INVALID_FEATURE_ID)
    let value = extract_feature(bits.value, fid)
    let is_present = !is_default_value(fid, value)
    state.operand_stack.push(Value::Bool(is_present))

Errors:
    TYPE_ERROR — if top is not feature_bits
    INVALID_FEATURE_ID — if feature_id > 19
```

**Mnemonic:** `FHAS`.

### 7.4 FEATURE_COMPARE_EQ (0x33) / FCMP

```
FEATURE_COMPARE_EQ — Compare a specific feature between two bitfields

Encoding:  0x33 <feature_id: u32 (varint)>
Size:      2–6 bytes
Stack:     [feature_bits_a, feature_bits_b] → [bool]
Steps:     1

Pseudocode:
    let bits_b = state.operand_stack.pop()
    let bits_a = state.operand_stack.pop()
    if bits_a.type != FeatureBits || bits_b.type != FeatureBits:
        error(TYPE_ERROR)
    let fid = decode_feature_id(operands[0])
    if fid > 19:
        error(INVALID_FEATURE_ID)
    let val_a = extract_feature(bits_a.value, fid)
    let val_b = extract_feature(bits_b.value, fid)
    state.operand_stack.push(Value::Bool(val_a == val_b))

Errors:
    TYPE_ERROR — if top values are not feature_bits
    INVALID_FEATURE_ID — if feature_id > 19
```

**Mnemonic:** `FCMP`.

### 7.5 FEATURE_COMPARE_MASK (0x34) / FCMPM

```
FEATURE_COMPARE_MASK — Compare multiple features via bitmask

Encoding:  0x34 <mask: u64 (8 bytes, LE)>
Size:      10 bytes (fixed)
Stack:     [feature_bits_a, feature_bits_b] → [bool]
Steps:     2

Pseudocode:
    let bits_b = state.operand_stack.pop()
    let bits_a = state.operand_stack.pop()
    if bits_a.type != FeatureBits || bits_b.type != FeatureBits:
        error(TYPE_ERROR)
    let mask = decode_u64(operands[0])
    let masked_a = bits_a.value & mask
    let masked_b = bits_b.value & mask
    state.operand_stack.push(Value::Bool(masked_a == masked_b))

Mask construction:
    // To compare gender + number:
    // mask = (0x3 << 4) | (0x3 << 6) = 0x30 | 0xC0 = 0xF0
    //
    // To compare case + state:
    // mask = (0x3 << 15) | (0x1 << 17) = 0x18000 | 0x20000 = 0x38000

Errors:
    TYPE_ERROR — if top values are not feature_bits
```

**Mnemonic:** `FCMPM`.

### 7.6 FEATURE_PACK (0x35) / FPACK

```
FEATURE_PACK — Pack multiple (feature_id, value) pairs into bitfield

Encoding:  0x35 <count: u32 (varint)>
Size:      2–6 bytes
Stack:     [fid_0, val_0, fid_1, val_1, ..., fid_n-1, val_n-1] → [feature_bits]
Steps:     count (variable, one per pair)

Pseudocode:
    let count = decode_varint(operands[0])
    let mut bits: u64 = 0
    for i in 0..count:
        let val = state.operand_stack.pop()
        let fid = state.operand_stack.pop()
        if fid.type != I32 || val.type != I32:
            error(TYPE_ERROR)
        if fid.value > 19:
            error(INVALID_FEATURE_ID)
        bits = set_feature(bits, fid.value as u32, val.value as u64)
    state.operand_stack.push(Value::FeatureBits(bits))

Errors:
    TYPE_ERROR — if feature IDs or values are not i32
    INVALID_FEATURE_ID — if any feature_id > 19
    STACK_UNDERFLOW — if stack has fewer than 2*count values
```

**Mnemonic:** `FPACK`.

---

## 8. Constituent Operations (0x40–0x4F)

### 8.1 CONST_MAKE (0x40) / CMAKE

```
CONST_MAKE — Create a new constituent node

Encoding:  0x40 <role_id: u32 (varint)>
Size:      2–6 bytes
Stack:     [u32, child_ptr_0, ..., child_ptr_n-1] → [constituent_ptr]
Steps:     2 + n (n = number of children)

Pseudocode:
    let role_id = decode_varint(operands[0])
    // Children are on the stack, preceded by their count
    let child_count = state.operand_stack.pop()
    if child_count.type != I32:
        error(TYPE_ERROR)
    let mut children = Vec::new()
    for i in 0..child_count.value:
        let child = state.operand_stack.pop()
        if child.type != ConstituentPtr:
            error(TYPE_ERROR)
        children.push(child.value)
    // Ensure region has capacity
    if state.constituent_region.size >= state.constituent_region.capacity:
        error(CONSTITUENT_REGION_FULL)
    let node = ConstituentNode {
        role: role_id,
        children: children,
        token_indices: Vec::new(),
    }
    let ptr = state.constituent_region.alloc(node)
    state.operand_stack.push(Value::ConstituentPtr(ptr))

Errors:
    CONSTITUENT_REGION_FULL — if constituent region is at capacity
    TYPE_ERROR — if child pointer or count types are wrong
```

**Mnemonic:** `CMAKE`.

### 8.2 CONST_ADD_CHILD (0x41) / CADD

```
CONST_ADD_CHILD — Add a child constituent to a parent

Encoding:  0x41 0x00
Size:      2 bytes
Stack:     [constituent_ptr, child_ptr] → [constituent_ptr]
Steps:     1

Pseudocode:
    let child = state.operand_stack.pop()
    let parent = state.operand_stack.pop()
    if parent.type != ConstituentPtr || child.type != ConstituentPtr:
        error(TYPE_ERROR)
    // Bounds check both pointers
    if parent.value >= state.constituent_region.size:
        error(INVALID_CONSTITUENT_PTR)
    if child.value >= state.constituent_region.size:
        error(INVALID_CONSTITUENT_PTR)
    state.constituent_region.add_child(parent.value, child.value)
    state.operand_stack.push(parent)  // Parent stays on stack

Errors:
    INVALID_CONSTITUENT_PTR — if either pointer is invalid
```

**Mnemonic:** `CADD`.

### 8.3 CONST_GET_CHILD (0x42) / CGET

```
CONST_GET_CHILD — Get a child by index

Encoding:  0x42 <child_index: u32 (varint)>
Size:      2–6 bytes
Stack:     [constituent_ptr] → [child_ptr]
Steps:     1

Pseudocode:
    let parent = state.operand_stack.pop()
    let child_idx = decode_varint(operands[0])
    if parent.type != ConstituentPtr:
        error(TYPE_ERROR)
    let node = state.constituent_region.get(parent.value)
    if child_idx >= node.children.len():
        error(CHILD_INDEX_OUT_OF_BOUNDS)
    state.operand_stack.push(
        Value::ConstituentPtr(node.children[child_idx]))

Errors:
    TYPE_ERROR — if top is not constituent_ptr
    CHILD_INDEX_OUT_OF_BOUNDS — if child_index >= child count
```

**Mnemonic:** `CGET`.

### 8.4 CONST_GET_ROLE (0x43) / CROLE

```
CONST_GET_ROLE — Get constituent's role ID

Encoding:  0x43 0x00
Size:      2 bytes
Stack:     [constituent_ptr] → [u32]
Steps:     1

Pseudocode:
    let ptr = state.operand_stack.pop()
    if ptr.type != ConstituentPtr:
        error(TYPE_ERROR)
    let node = state.constituent_region.get(ptr.value)
    state.operand_stack.push(Value::I32(node.role as i32))

Errors:
    TYPE_ERROR — if top is not constituent_ptr
```

**Mnemonic:** `CROLE`.

### 8.5 CONST_SET_ROLE (0x44) / CSROLE

```
CONST_SET_ROLE — Set constituent's role ID

Encoding:  0x44 0x00
Size:      2 bytes
Stack:     [constituent_ptr, u32] → [constituent_ptr]
Steps:     1

Pseudocode:
    let new_role = state.operand_stack.pop()
    let ptr = state.operand_stack.pop()
    if ptr.type != ConstituentPtr || new_role.type != I32:
        error(TYPE_ERROR)
    state.constituent_region.get_mut(ptr.value).role = new_role.value as u32
    state.operand_stack.push(ptr)

Errors:
    TYPE_ERROR — if pointer or role value types are wrong
```

**Mnemonic:** `CSROLE`.

### 8.6 CONST_ATTACH_TOKENS (0x45) / CATOK

```
CONST_ATTACH_TOKENS — Attach token indices to a constituent

Encoding:  0x45 <count: u32 (varint)>
Size:      2–6 bytes
Stack:     [constituent_ptr, tok_idx_0, ..., tok_idx_n-1] → [constituent_ptr]
Steps:     count

Pseudocode:
    let count = decode_varint(operands[0])
    let mut token_indices = Vec::new()
    for i in 0..count:
        let tok = state.operand_stack.pop()
        if tok.type != I32 && tok.type != TokenIndex:
            error(TYPE_ERROR)
        token_indices.push(tok.value as u32)
    let ptr = state.operand_stack.pop()
    if ptr.type != ConstituentPtr:
        error(TYPE_ERROR)
    state.constituent_region.get_mut(ptr.value).token_indices = token_indices
    state.operand_stack.push(ptr)

Errors:
    TYPE_ERROR — if token indices or pointer types are wrong
```

**Mnemonic:** `CATOK`.

### 8.7 CONST_TRAVERSE (0x46) / CTRAV

```
CONST_TRAVERSE — Depth-first traversal with callback

Encoding:  0x46 <offset: i32 (zigzag)>
Size:      2–6 bytes
Stack:     [constituent_ptr] → [constituent_ptr]
Steps:     Varies (proportional to tree depth)

Pseudocode:
    fn traverse(node_ptr, state, callback_offset, pc):
        // Push current node for callback
        state.operand_stack.push(Value::ConstituentPtr(node_ptr))
        // Jump to callback (uses temporary PC modification)
        let saved_pc = state.pc
        state.pc = callback_offset
        // Callback processes node and should RETURN
        // After RETURN, continue traversal
        state.pc = saved_pc + instruction_length
        let node = state.constituent_region.get(node_ptr)
        for child_ptr in &node.children:
            traverse(*child_ptr, state, callback_offset, pc)
    // Start traversal from root
    let root = state.operand_stack.pop()
    if root.type != ConstituentPtr:
        error(TYPE_ERROR)
    traverse(root.value, state, offset, state.pc)
    state.operand_stack.push(root)

Errors:
    TYPE_ERROR — if top is not constituent_ptr
    CALL_STACK_OVERFLOW — if tree is too deep (> max_call_depth)
```

**Mnemonic:** `CTRAV`.

---

## 9. Rule Operations (0x50–0x5F)

### 9.1 RULE_APPLY (0x50) / RAPPLY

```
RULE_APPLY — Record a rule application

Encoding:  0x50 <rule_id: u32 (varint)>, <rule_name_index: u32 (varint)>
Size:      2–10 bytes (2 + 2× varint)
Stack:     [] → []
Steps:     1

Pseudocode:
    let rule_name_idx = decode_varint(operands[1])
    let rule_id = decode_varint(operands[0])
    if rule_name_idx >= state.string_region.size:
        error(STRING_INDEX_OUT_OF_BOUNDS)
    if state.rule_region.size >= state.rule_region.capacity:
        error(RULE_REGION_FULL)
    let record = RuleApplicationRecord {
        rule_id: rule_id as u32,
        rule_name_index: rule_name_idx as u32,
        pc: state.pc,
        token_index: state.current_token_index,
        timestamp: state.step_count,
    }
    state.rule_region.alloc(record)
    state.last_rule_index = state.rule_region.size - 1

Errors:
    STRING_INDEX_OUT_OF_BOUNDS — if rule_name_index is invalid
    RULE_REGION_FULL — if rule region capacity exceeded
```

**Mnemonic:** `RAPPLY`.

### 9.2 RULE_CONFIRM (0x51) / RCONF

```
RULE_CONFIRM — Confirm an analysis

Encoding:  0x51 <analysis_index: u32 (varint)>
Size:      2–6 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    let analysis_idx = decode_varint(operands[0])
    // Associates confirmation with most recent RULE_APPLY
    if state.last_rule_index == None:
        error(NO_RULE_IN_CONTEXT)
    state.rule_region.get_mut(state.last_rule_index)
        .add_confirmation(analysis_idx as u32)

Errors:
    NO_RULE_IN_CONTEXT — if no RULE_APPLY has been executed
```

**Mnemonic:** `RCONF`.

### 9.3 RULE_REJECT (0x52) / RREJ

```
RULE_REJECT — Reject an analysis

Encoding:  0x52 <analysis_index: u32 (varint)>
Size:      2–6 bytes
Stack:     [] → []
Steps:     1

Errors:
    NO_RULE_IN_CONTEXT — if no RULE_APPLY has been executed

(Same format as RULE_CONFIRM, but marks the analysis as rejected.)
```

**Mnemonic:** `RREJ`.

### 9.4 RULE_MODIFY (0x53) / RMOD

```
RULE_MODIFY — Modify a feature on a bitfield

Encoding:  0x53 <feature_id: u32 (varint)>
Size:      2–6 bytes
Stack:     [feature_bits, value: u32] → [feature_bits]
Steps:     1

Pseudocode:
    // (same as FEATURE_SET, but also records modification in rule region)
    let new_val = state.operand_stack.pop()
    let bits = state.operand_stack.pop()
    if bits.type != FeatureBits || new_val.type != I32:
        error(TYPE_ERROR)
    let fid = decode_feature_id(operands[0])
    if fid > 19:
        error(INVALID_FEATURE_ID)
    let old_val = extract_feature(bits.value, fid)
    let new_bits = set_feature(bits.value, fid, new_val.value as u64)
    // Record modification if a rule is active
    if state.last_rule_index != None:
        state.rule_region.get_mut(state.last_rule_index)
            .add_modification(fid, old_val, new_val.value as u32)
    state.operand_stack.push(Value::FeatureBits(new_bits))

Errors:
    TYPE_ERROR — if top is not feature_bits or value is not i32
    INVALID_FEATURE_ID — if feature_id > 19
```

**Mnemonic:** `RMOD`.

### 9.5 RULE_FLAG (0x54) / RFLAG

```
RULE_FLAG — Raise a grammatical flag

Encoding:  0x54 <flag_type: u32 (varint)>, <flag_code_index: u32 (varint)>
Size:      2–10 bytes
Stack:     [] → []
Steps:     1

Flag types:
    0 = error
    1 = warning
    2 = info

Pseudocode:
    let flag_code_idx = decode_varint(operands[1])
    let flag_type = decode_varint(operands[0])
    if flag_code_idx >= state.string_region.size:
        error(STRING_INDEX_OUT_OF_BOUNDS)
    if flag_type > 2:
        error(TYPE_ERROR)
    state.flags.push(GrammaticalFlag {
        flag_type,
        code_index: flag_code_idx as u32,
        token_index: state.current_token_index,
        rule_id: state.last_rule_index.map(|i| state.rule_region.get(i).rule_id),
        pc: state.pc,
    })

Errors:
    STRING_INDEX_OUT_OF_BOUNDS — if flag_code_index is invalid
    TYPE_ERROR — if flag_type > 2
```

**Mnemonic:** `RFLAG`.

### 9.6 RULE_RESOLVE (0x55) / RRES

```
RULE_RESOLVE — Record anaphora resolution

Encoding:  0x55 0x00
Size:      2 bytes
Stack:     [token_index_antecedent, token_index_pronoun] → []
Steps:     1

Pseudocode:
    let pronoun_idx = state.operand_stack.pop()
    let antecedent_idx = state.operand_stack.pop()
    if pronoun_idx.type != TokenIndex && pronoun_idx.type != I32:
        error(TYPE_ERROR)
    if antecedent_idx.type != TokenIndex && antecedent_idx.type != I32:
        error(TYPE_ERROR)
    if antecedent_idx.value >= state.token_region.size
        || pronoun_idx.value >= state.token_region.size:
        error(TOKEN_INDEX_OUT_OF_BOUNDS)
    state.anaphora_resolutions.push(AnaphoraResolution {
        antecedent_index: antecedent_idx.value as u32,
        pronoun_index: pronoun_idx.value as u32,
    })

Errors:
    TYPE_ERROR — if stack values are not token indices
    TOKEN_INDEX_OUT_OF_BOUNDS — if either index is out of bounds
```

**Mnemonic:** `RRES`.

---

## 10. Evidence Operations (0x60–0x6F)

### 10.1 EVIDENCE_PUSH (0x60) / EPUSH

```
EVIDENCE_PUSH — Create an evidence entry

Encoding:  0x60 <stage_name_index: u32 (varint)>, <algorithm_index: u32 (varint)>
Size:      2–10 bytes
Stack:     [confidence: f64] → []
Steps:     2

Pseudocode:
    let confidence = state.operand_stack.pop()
    if confidence.type != F64:
        error(TYPE_ERROR)
    if confidence.f64_value < 0.0 || confidence.f64_value > 1.0:
        error(TYPE_ERROR)
    let alg_idx = decode_varint(operands[1])
    let stage_idx = decode_varint(operands[0])
    if stage_idx >= state.string_region.size
        || alg_idx >= state.string_region.size:
        error(STRING_INDEX_OUT_OF_BOUNDS)
    if state.evidence_region.size >= state.evidence_region.capacity:
        error(EVIDENCE_REGION_FULL)
    state.evidence_region.alloc(EvidenceEntry {
        stage_name_index: stage_idx as u32,
        algorithm_index: alg_idx as u32,
        confidence: confidence.f64_value,
        token_index: state.current_token_index,
        step: state.step_count,
    })

Errors:
    TYPE_ERROR — if confidence is not f64, or confidence out of [0, 1]
    STRING_INDEX_OUT_OF_BOUNDS — if stage or algorithm index is invalid
    EVIDENCE_REGION_FULL — if evidence region capacity exceeded
```

**Mnemonic:** `EPUSH`.

### 10.2 EVIDENCE_QUERY (0x61) / EQUERY

```
EVIDENCE_QUERY — Get evidence count

Encoding:  0x61 0x00
Size:      2 bytes
Stack:     [] → [u32]
Steps:     1

Pseudocode:
    state.operand_stack.push(
        Value::I32(state.evidence_region.size as i32))

Errors:
    STACK_OVERFLOW — if stack is full
```

**Mnemonic:** `EQUERY`.

### 10.3 EVIDENCE_EMIT (0x62) / EEMIT

```
EVIDENCE_EMIT — Mark evidence for inclusion in output

Encoding:  0x62 0x00
Size:      2 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    state.evidence_emitted = true
    // Marks all current evidence entries for inclusion
    // in the AnalysisResult

Errors:    None
```

**Mnemonic:** `EEMIT`.

---

## 11. Output Operations (0x70–0x7F)

### 11.1 OUTPUT_SET_METADATA (0x70) / OSMETA

```
OUTPUT_SET_METADATA — Set output metadata key-value pair

Encoding:  0x70 <key_index: u32 (varint)>, <value_index: u32 (varint)>
Size:      2–10 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    let val_idx = decode_varint(operands[1])
    let key_idx = decode_varint(operands[0])
    if key_idx >= state.string_region.size
        || val_idx >= state.string_region.size:
        error(STRING_INDEX_OUT_OF_BOUNDS)
    state.output_metadata.insert(key_idx, val_idx)

Errors:
    STRING_INDEX_OUT_OF_BOUNDS — if either index is invalid
```

**Mnemonic:** `OSMETA`.

### 11.2 OUTPUT_ADD_TREE (0x71) / OATREE

```
OUTPUT_ADD_TREE — Add an analysis tree to output

Encoding:  0x71 <tree_type_index: u32 (varint)>
Size:      2–6 bytes
Stack:     [constituent_ptr, confidence: f64] → []
Steps:     2

Pseudocode:
    let confidence = state.operand_stack.pop()
    let tree_root = state.operand_stack.pop()
    if tree_root.type != ConstituentPtr:
        error(TYPE_ERROR)
    if confidence.type != F64:
        error(TYPE_ERROR)
    if confidence.f64_value < 0.0 || confidence.f64_value > 1.0:
        error(TYPE_ERROR)
    let tree_type_idx = decode_varint(operands[0])
    if tree_type_idx >= state.string_region.size:
        error(STRING_INDEX_OUT_OF_BOUNDS)
    state.output_trees.push(OutputTree {
        root: tree_root.value as u32,
        tree_type_index: tree_type_idx as u32,
        confidence: confidence.f64_value,
    })

Errors:
    TYPE_ERROR — if pointer or confidence types are wrong
    STRING_INDEX_OUT_OF_BOUNDS — if tree_type_index is invalid
```

**Mnemonic:** `OATREE`.

### 11.3 OUTPUT_ADD_TOKEN (0x72) / OATOK

```
OUTPUT_ADD_TOKEN — Add a token to the output

Encoding:  0x72 0x00
Size:      2 bytes
Stack:     [token_index, feature_bits, role_id: u32] → []
Steps:     1

Pseudocode:
    let role_id = state.operand_stack.pop()
    let features = state.operand_stack.pop()
    let tok_idx = state.operand_stack.pop()
    if tok_idx.type != TokenIndex
        && features.type != FeatureBits
        && role_id.type != I32:
        error(TYPE_ERROR)
    state.output_tokens.push(OutputToken {
        token_index: tok_idx.value as u32,
        feature_bits: features.u64_value,
        role_id: role_id.value as u32,
    })

Errors:
    TYPE_ERROR — if stack value types are wrong
```

**Mnemonic:** `OATOK`.

### 11.4 OUTPUT_SET_INPUT (0x73) / OSIN

```
OUTPUT_SET_INPUT — Set input text and hash

Encoding:  0x73 <text_index: u32 (varint)>, <hash_index: u32 (varint)>
Size:      2–10 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    let hash_idx = decode_varint(operands[1])
    let text_idx = decode_varint(operands[0])
    if text_idx >= state.string_region.size
        || hash_idx >= state.string_region.size:
        error(STRING_INDEX_OUT_OF_BOUNDS)
    state.input_text_index = text_idx as u32
    state.input_text_hash_index = hash_idx as u32

Errors:
    STRING_INDEX_OUT_OF_BOUNDS — if either string index is invalid
```

**Mnemonic:** `OSIN`.

### 11.5 OUTPUT_FINALIZE (0x74) / OFIN

```
OUTPUT_FINALIZE — Finalize the AnalysisResult

Encoding:  0x74 0x00
Size:      2 bytes
Stack:     [] → []
Steps:     1

Pseudocode:
    // Sort trees by confidence descending
    state.output_trees.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence))
    // Collect all flags
    state.output_flags = state.flags.clone()
    // After this instruction, no more output operations are allowed.
    // If any output instruction is executed after FINALIZE, it is ignored
    // (no effect, no error).
    state.output_finalized = true

Errors:    None
```

**Mnemonic:** `OFIN`.

---

## 12. Reserved Opcodes & Extension (0x80–0xFF)

### 12.1 Planned Extension Allocation

The 128 reserved opcode slots (0x80–0xFF) are pre-allocated for future instruction sets:

| Range | Size | Planned Extension | Priority |
|-------|------|-------------------|----------|
| 0x80–0x8F | 16 | Serialization instructions (direct read/write of serialized data) | High |
| 0x90–0x9F | 16 | Statistical analysis (count, sum, mean per feature) | Medium |
| 0xA0–0xA7 | 8 | Corpus comparison (diff/similarity between analyses) | Low |
| 0xA8–0xAF | 8 | Plugin call interface (invoke external plugin functions) | High |
| 0xB0–0xBF | 16 | Dynamic string operations (concat, substring at runtime) | Medium |
| 0xC0–0xC7 | 8 | Advanced traversal (BFS, post-order constituent traversal) | Low |
| 0xC8–0xCF | 8 | Debugging instructions (breakpoint, step-over markers) | Medium |
| 0xD0–0xFF | 48 | Unallocated (available for future use) | — |

### 12.2 Extension Policy

| Policy | Rule |
|--------|------|
| **Registration** | New opcodes MUST be registered by adding them to the official opcode table. |
| **Versioning** | New opcodes are introduced with a MINOR bytecode version bump. |
| **GVM compatibility** | An older GVM encountering an unknown opcode MUST return `INVALID_OPCODE`. |
| **Deprecation** | Deprecated opcodes MUST remain in the table for at least 2 major versions. |
| **Custom/plugin** | Plugins MAY define custom opcodes, but they MUST use a registered opcode range for their plugin ID. |

---

## 13. Common Instruction Sequences

### 13.1 Token Feature Comparison Loop

The most common pattern: iterate tokens, compare features, apply rules.

```
// Setup: load token count, initialize counter
TOKEN_COUNT             // push total
PUSH_I32 0              // push current = 0
                        // stack: [0, total]

loop_header:            // ← loop start target
  DUP                   // total
  DUP                   // total (for TOKITR)
  TOKITR loop_end       // if current >= total, exit

  // current, total still on stack (TOKITR restores them)
  DUP                   // duplicate total for safety
  LOAD_TOKEN            // load token at current index
  // stack: [current, total, tok_idx]

  TOKEN_GET_FEATURES    // get feature bitfield
  // stack: [current, total, feature_bits]

  DUP                   // duplicate features for comparison
  FEATURE_HAS 1         // check gender (feature_id=1)
  // stack: [current, total, feature_bits, has_gender?]

  JUMP_IF_FALSE skip_gender

  // Compare gender with another bitfield
  // (second bitfield would be pushed here)
  FCMP 1                // FEATURE_COMPARE_EQ gender
  // stack: [current, total, feature_bits, gender_match?]

  JUMP_IF_TRUE apply_rule
  // ... or continue with more comparisons

skip_gender:
  POP                   // discard feature_bits
  JUMP loop_header      // next token

apply_rule:
  RULE_APPLY 100, 5     // rule_id=100, name_index=5
  RULE_CONFIRM 0        // confirm analysis 0
  POP                   // discard feature_bits
  JUMP loop_header

loop_end:
  POP                   // discard total
  POP                   // discard current
```

### 13.2 Case Assignment Pattern

Modify a token's case feature based on a government rule:

```
// 1. Load the governed token (e.g., noun after preposition)
PUSH_I32 1              // token index 1 (the noun)
LOAD_TOKEN              // load it
TOKEN_GET_FEATURES      // get features

// 2. Set the case to genitive
PUSH_I32 2              // value = 2 (genitive)
FSET 7                  // set case (feature_id=7)
                        // stack: [new_feature_bits]

// 3. Apply the rule
RULE_APPLY 201, 6       // rule_id=201, "preposition-governs-genitive"
RULE_MODIFY 7           // record modification of case (feature_id=7)
```

### 13.3 Constituent Tree Building

Build a parse tree for a verbal sentence:

```
// Verb constituent
PUSH_I32 0              // child count = 0
CMAKE 10                // role_id=10 (fi'l)
                        // stack: [verb_constituent]

// Subject constituent
PUSH_I32 0              // child count = 0
CMAKE 11                // role_id=11 (fa'il)
                        // stack: [verb_constituent, subject_constituent]

// Attach tokens to verb
DUP                     // duplicate subject pointer
PUSH_I32 0              // token index 0
CATOK 1                 // attach 1 token
                        // stack: [verb_constituent, subject_constituent]

// Attach tokens to subject
DUP                     // duplicate subject pointer
PUSH_I32 1              // token index 1
CATOK 1                 // attach 1 token
                        // stack: [verb_constituent, subject_constituent]

// Create clause with 2 children
PUSH_I32 2              // child count = 2
SWAP                    // ... subject, count → stack: [verb, 2, subject]
SWAP                    // stack: [verb, subject, 2]
CMAKE 20                // role_id=20 (jumlah_fi'liyyah)
                        // stack: [clause_constituent]

// Add to output with confidence
PUSH_F64 0.95           // confidence
OATREE 3                // tree_type_index=3 (verbal sentence)
```

### 13.4 Evidence Push Pattern

```
// Load current token
LOAD_TOKEN              // load token at current index
// (stack: [tok_idx])

// Before processing, push evidence
PUSH_F64 0.85           // confidence
EVIDENCE_PUSH 7, 8      // stage="MOD-04", algorithm="root-extraction-v2"
```

### 13.5 Mood Government Pattern

Check and modify verb mood based on governing particle:

```
// Load verb token
PUSH_I32 verb_idx       // verb token index
LOAD_TOKEN
TOKEN_GET_FEATURES      // features

// Check current mood is not already jussive
DUP                     // duplicate features
FGET 5                  // get mood (feature_id=5)
PUSH_I32 2              // jussive mood = 2
FCMP 5                  // compare mood
// stack: [features, is_already_jussive?]

JUMP_IF_TRUE already_jussive

// Set mood to jussive
PUSH_I32 2              // jussive = 2
FSET 5                  // set mood
RULE_APPLY 301, 9       // "lam-governs-jussive"
RULE_MODIFY 5           // record modification

already_jussive:
  POP                   // discard features
```

---

## 14. Instruction Cost Model

### 14.1 Step Count by Category

Each instruction has a fixed step cost. The total step count determines execution budget.

| Category | Instructions | Steps per Instr | Typical Count in Bytecode |
|----------|-------------|-----------------|--------------------------|
| Flow Control | HALT, JUMP, CALL, RETURN, DIE | 1 | 5–20 |
| Stack | PUSH, POP, DUP, SWAP | 1 | 40–200 |
| Token | LOAD_TOKEN, TOKEN_GET_FEATURES, etc. | 1–2 | 60–300 |
| Feature | FEATURE_GET, FEATURE_SET, FCMP | 1–2 | 50–250 |
| Constituent | CONST_MAKE, CONST_ADD_CHILD, TRAVERSE | 1–3 | 30–150 |
| Rule | RULE_APPLY, RULE_CONFIRM, MODIFY | 1 | 10–100 |
| Evidence | EVIDENCE_PUSH, QUERY, EMIT | 1–2 | 5–50 |
| Output | OUTPUT_ADD_TREE, OUTPUT_FINALIZE | 1–2 | 5–20 |

**Typical total steps:** 200–2,000 instructions (varying with ambiguity and sentence length).

### 14.2 Estimated Wall-Clock Latency

| Instruction | Approx Time (ns) | Notes |
|-------------|------------------|-------|
| HALT | 5 | Simple flag set |
| JUMP | 10 | Branch + bounds check |
| PUSH (any type) | 5–15 | Stack push + optional bounds check |
| POP | 5 | Stack pop only |
| LOAD_TOKEN | 15 | Bounds check + index push |
| TOKEN_GET_FEATURES | 10 | Region read + type check |
| FEATURE_GET | 15 | Extract bitfield + mask/shift |
| FEATURE_SET | 15 | Mask/set bitfield + validate |
| FEATURE_COMPARE_EQ | 15 | Two extracts + comparison |
| FEATURE_COMPARE_MASK | 20 | Two ANDs + comparison |
| CONST_MAKE | 30–50 | Allocation + config (varies with children) |
| CONST_ADD_CHILD | 10 | Bounds checks + push |
| RULE_APPLY | 15 | Write to rule region |
| EVIDENCE_PUSH | 25 | Write to evidence region + validate confidence |
| OUTPUT_ADD_TREE | 30 | Push to output + validate |

**Profile breakdown (10-word sentence, ~500 instructions):**

| Profile | Instructions | Estimated Time |
|---------|-------------|----------------|
| interactive | 500 | ~5 μs (5,000 ns / 500) |
| server | 500 | ~5 μs |
| batch | 500 | ~5 μs |
| debug | 500 + tracing | ~50–100 μs (tracing overhead ~10×) |

### 14.3 Peak Memory per Instruction

| Operation | Temporary Memory | Persistent Memory |
|-----------|-----------------|-------------------|
| Stack operations | 16 bytes (Value) | 0 |
| Token load | 0 | 0 (already allocated) |
| Feature operations | 8 bytes (u64) | 0 |
| CONST_MAKE | 0 | ~48 bytes (ConstituentNode) |
| RULE_APPLY | 0 | ~32 bytes (RuleRecord) |
| EVIDENCE_PUSH | 0 | ~64 bytes (EvidenceEntry) |
| OUTPUT_ADD_TREE | 0 | ~32 bytes |

---

## 15. Error Code Reference

### 15.1 Complete Error Code Table

| Error Code | Value | Raised By | Description | Recovery |
|------------|-------|-----------|-------------|----------|
| `UNSUPPORTED_BYTECODE_VERSION` | 1 | Verifier | Bytecode version exceeds GVM | Update GVM or regenerate bytecode |
| `BYTECODE_CORRUPTED` | 2 | Verifier | CRC32C checksum mismatch | Regenerate bytecode |
| `MAX_STEPS_EXCEEDED` | 3 | Executor | Step limit reached | Increase limit or simplify input |
| `MAX_MEMORY_EXCEEDED` | 4 | Executor | Memory limit reached | Increase limit or simplify input |
| `EXECUTION_FAILURE` | 5 | Executor | Unrecoverable execution error | Report bug |
| `STACK_OVERFLOW` | 10 | All stack ops | Stack depth exceeded max | Reduce expression complexity |
| `STACK_UNDERFLOW` | 11 | POP, DUP, SWAP | Stack empty when value needed | Bug in bytecode generator |
| `TYPE_ERROR` | 12 | Many | Wrong value type on stack | Bug in bytecode generator |
| `INVALID_OPCODE` | 13 | Executor | Unknown opcode encountered | Update GVM or regenerate bytecode |
| `JUMP_OUT_OF_BOUNDS` | 14 | JUMP, CALL | Jump target outside instruction section | Bug in bytecode generator |
| `TOKEN_INDEX_OUT_OF_BOUNDS` | 20 | LOAD_TOKEN | Token index exceeds region | Bug in bytecode generator |
| `STRING_INDEX_OUT_OF_BOUNDS` | 21 | PUSH_STRING, RULE_APPLY, etc. | String index exceeds table | Bug in bytecode generator |
| `FEATURE_INDEX_OUT_OF_BOUNDS` | 22 | TOKEN_GET_FEATURES | Feature index exceeds region | Bug in bytecode generator |
| `CONSTITUENT_INDEX_OUT_OF_BOUNDS` | 23 | CONST_ADD_CHILD | Constituent index exceeds region | Bug in bytecode generator |
| `CONSTITUENT_REGION_FULL` | 24 | CONST_MAKE | Constituent region at capacity | Increase capacity in bytecode |
| `CHILD_INDEX_OUT_OF_BOUNDS` | 25 | CONST_GET_CHILD | Child index out of range | Bug in bytecode generator |
| `CALL_STACK_OVERFLOW` | 30 | CALL | Call stack max depth exceeded | Reduce call nesting |
| `CALL_STACK_UNDERFLOW` | 31 | RETURN | Return with empty call stack | Bug in bytecode generator |
| `INVALID_FEATURE_ID` | 40 | FEATURE_GET, SET, HAS, FCMP | Feature ID not in taxonomy | Update bytecode generator |
| `RULE_REGION_FULL` | 50 | RULE_APPLY | Rule region at capacity | Increase capacity |
| `NO_RULE_IN_CONTEXT` | 51 | RULE_CONFIRM, REJECT, MODIFY | No RULE_APPLY preceding | Bug in bytecode generator |
| `EVIDENCE_REGION_FULL` | 60 | EVIDENCE_PUSH | Evidence region at capacity | Increase capacity |
| `SCRATCH_OVERFLOW` | 70 | (future) | Scratch buffer capacity exceeded | Increase capacity |
| `INTERNAL_ERROR` | 255 | Any | Unexpected GVM implementation failure | Report bug |

### 15.2 Error Recovery by Phase

| Phase | Non-Fatal | Fatal | Recovery |
|-------|-----------|-------|----------|
| **Loading** | Version warning (minor mismatch) | Magic mismatch, corrupted header | Regenerate bytecode |
| **Verification** | Unknown feature ID warning | Checksum mismatch, invalid jump target | Regenerate bytecode |
| **Execution** | — | All runtime errors | Fix bytecode generator; retry |
| **Assembly** | — | Missing required output fields | Bug in GVM implementation |

---

## 16. Feature ID Reference for Instructions

### 16.1 Feature ID Constants

These constants are used in the FEATURE_GET (0x30), FEATURE_SET (0x31), FEATURE_HAS (0x32), FEATURE_COMPARE_EQ (0x33), FEATURE_PACK (0x35), and RULE_MODIFY (0x53) instructions.

```rust
// Feature IDs for GVM instructions (maps to KB-0007 bitfield positions)
const FEATURE_POS:              u32 = 0;   // bits 0–3
const FEATURE_GENDER:           u32 = 1;   // bits 4–5
const FEATURE_NUMBER:           u32 = 2;   // bits 6–7
const FEATURE_PERSON:           u32 = 3;   // bits 8–9
const FEATURE_TENSE:            u32 = 4;   // bits 10–11
const FEATURE_MOOD:             u32 = 5;   // bits 12–13
const FEATURE_VOICE:            u32 = 6;   // bit  14
const FEATURE_CASE:             u32 = 7;   // bits 15–16
const FEATURE_STATE:            u32 = 8;   // bit  17
const FEATURE_VERB_FORM:        u32 = 10;  // bits 18–22
const FEATURE_NOUN_TYPE:        u32 = 11;  // bits 23–27
const FEATURE_PRONOUN_TYPE:     u32 = 12;  // bits 28–31
const FEATURE_TRANSITIVITY:     u32 = 13;  // bits 32–35
const FEATURE_ROOT_TYPE:        u32 = 14;  // bits 36–39
const FEATURE_STRESS_PATTERN:   u32 = 15;  // bits 40–42
const FEATURE_SYLLABLE_COUNT:   u32 = 16;  // bits 43–46
const FEATURE_HAS_SHADDA:       u32 = 17;  // bit  47
const FEATURE_HAS_MADD:         u32 = 18;  // bit  48
const FEATURE_HAS_HAMZA:        u32 = 19;  // bit  49
```

### 16.2 Feature Value Constants

These constants are used as comparison values with FEATURE_GET, FEATURE_SET, FEATURE_PACK, etc.

```rust
// POS values (feature_id = 0)
const POS_VERB:          u32 = 0;  // فعل
const POS_NOUN:          u32 = 1;  // اسم
const POS_PARTICLE:      u32 = 2;  // حرف
const POS_PRONOUN:       u32 = 3;  // ضمير
const POS_ADJECTIVE:     u32 = 4;  // صفة
const POS_ADVERB:        u32 = 5;  // ظرف
const POS_PREPOSITION:   u32 = 6;  // حرف جر
const POS_CONJUNCTION:   u32 = 7;  // حرف عطف
const POS_PROPER_NOUN:   u32 = 8;  // اسم علم
const POS_INTERROGATIVE: u32 = 9;  // اسم استفهام

// Gender values (feature_id = 1)
const GENDER_MASCULINE:  u32 = 0;  // مذكر
const GENDER_FEMININE:   u32 = 1;  // مؤنث
const GENDER_COMMON:     u32 = 2;  // مشترك

// Number values (feature_id = 2)
const NUMBER_SINGULAR:   u32 = 0;  // مفرد
const NUMBER_DUAL:       u32 = 1;  // مثنى
const NUMBER_PLURAL:     u32 = 2;  // جمع

// Person values (feature_id = 3)
const PERSON_FIRST:      u32 = 0;  // متكلم
const PERSON_SECOND:     u32 = 1;  // مخاطب
const PERSON_THIRD:      u32 = 2;  // غائب

// Tense values (feature_id = 4)
const TENSE_PAST:        u32 = 0;  // ماض
const TENSE_PRESENT:     u32 = 1;  // مضارع
const TENSE_IMPERATIVE:  u32 = 2;  // أمر

// Mood values (feature_id = 5)
const MOOD_INDICATIVE:   u32 = 0;  // مرفوع
const MOOD_SUBJUNCTIVE:  u32 = 1;  // منصوب
const MOOD_JUSSIVE:      u32 = 2;  // مجزوم
const MOOD_ENERGETIC:    u32 = 3;  // مؤكد

// Voice values (feature_id = 6)
const VOICE_ACTIVE:      u32 = 0;  // مبني للمعلوم
const VOICE_PASSIVE:     u32 = 1;  // مبني للمجهول

// Case values (feature_id = 7)
const CASE_NOMINATIVE:   u32 = 0;  // مرفوع
const CASE_ACCUSATIVE:   u32 = 1;  // منصوب
const CASE_GENITIVE:     u32 = 2;  // مجرور

// State values (feature_id = 8)
const STATE_DEFINITE:    u32 = 0;  // معرفة
const STATE_INDEFINITE:  u32 = 1;  // نكرة

// Verb form values (feature_id = 10)
const VERB_FORM_NOT_A_VERB: u32 = 0;  // ليس فعلاً
const VERB_FORM_I:          u32 = 1;  // فَعَلَ
const VERB_FORM_II:         u32 = 2;  // فَعَّلَ
const VERB_FORM_III:        u32 = 3;  // فَاعَلَ
const VERB_FORM_IV:         u32 = 4;  // أَفْعَلَ
const VERB_FORM_V:          u32 = 5;  // تَفَعَّلَ
const VERB_FORM_VI:         u32 = 6;  // تَفَاعَلَ
const VERB_FORM_VII:        u32 = 7;  // اِنْفَعَلَ
const VERB_FORM_VIII:       u32 = 8;  // اِفْتَعَلَ
const VERB_FORM_IX:         u32 = 9;  // اِفْعَلَّ
const VERB_FORM_X:          u32 = 10; // اِسْتَفْعَلَ
```

### 16.3 Feature Mask Constants (for FEATURE_COMPARE_MASK)

```rust
// Pre-computed bitmasks for common feature comparison patterns
const MASK_GENDER_NUMBER:  u64 = 0x00F0;       // bits 4–7
const MASK_PERSON_GENDER:  u64 = 0x0330;       // bits 4–5, 8–9
const MASK_TENSE_MOOD:     u64 = 0x3C00;       // bits 10–13
const MASK_CASE_STATE:     u64 = 0x038000;     // bits 15–17
const MASK_ALL_INFLECTION: u64 = 0x0003FFFF;   // bits 0–17
const MASK_ALL_DERIVATION: u64 = 0x000FFFC0000; // bits 18–39
const MASK_FULL_FEATURES:  u64 = 0x0003FFFFFFFF; // bits 0–49
```

---

## 17. Cross-References

### 17.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| RFC-0003 §5 | Instruction Set Reference | Architecture-level instruction definitions (SPEC-0302 provides the detailed implementation reference) |
| RFC-0003 §4 | Instruction Encoding | Opcode/flags/operand encoding specification |
| RFC-0003 §6 | Memory Model | Region layout used by all instructions |
| RFC-0003 §7 | Execution Model | Fetch-decode-execute cycle |
| RFC-0003 §8 | Configuration & Tuning | Step limits, stack depths |
| RFC-0002 §11 | Instruction Section | Bytecode binary layout for instructions |
| RFC-0002 §9 | Feature Section | 64-bit bitfield encoding |
| SPEC-0301 §5 | GVM Execution Pipeline | Instruction dispatch, cycle, cost model |
| SPEC-0301 §7 | Diagnostics & Verification | Tracing, disassembler, CLI |
| SPEC-0102 §8 | 64-Bit Bitfield Reference | Complete bitfield layout, pack/unpack |
| SPEC-0102 §4 | Inflectional Features | Feature value references |
| SPEC-0102 §5 | Derivational Features | Verb form, noun type, pronoun type values |
| KB-0007 §10 | Feature Bitfield Encoding | Authoritative bitfield layout |

### 17.2 External References

| Reference | Relevance |
|-----------|-----------|
| JVM Specification (Instruction Set) | Stack-based VM instruction encoding patterns |
| WebAssembly Binary Format | Opcode space allocation, extension strategy |
| Lua 5.4 VM Instructions | Minimal instruction set design patterns |
| LEB128 Encoding (DWARF) | Variable-length integer encoding |

---

## Progress Summary

**SPEC-0302: GVM Instruction Set — Instruction-Level Reference**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Instruction Encoding Reference | ✓ COMPLETE |
| 3 | Opcode Table & Dispatch Reference | ✓ COMPLETE |
| 4 | Flow Control Instructions (0x00–0x0F) | ✓ COMPLETE (7 instructions) |
| 5 | Stack Operations (0x10–0x1F) | ✓ COMPLETE (9 instructions) |
| 6 | Token Operations (0x20–0x2F) | ✓ COMPLETE (7 instructions) |
| 7 | Feature Operations (0x30–0x3F) | ✓ COMPLETE (6 instructions) |
| 8 | Constituent Operations (0x40–0x4F) | ✓ COMPLETE (7 instructions) |
| 9 | Rule Operations (0x50–0x5F) | ✓ COMPLETE (6 instructions) |
| 10 | Evidence Operations (0x60–0x6F) | ✓ COMPLETE (3 instructions) |
| 11 | Output Operations (0x70–0x7F) | ✓ COMPLETE (5 instructions) |
| 12 | Reserved Opcodes & Extension | ✓ COMPLETE |
| 13 | Common Instruction Sequences | ✓ COMPLETE (5 sequences) |
| 14 | Instruction Cost Model | ✓ COMPLETE |
| 15 | Error Code Reference | ✓ COMPLETE |
| 16 | Feature ID Reference for Instructions | ✓ COMPLETE |
| 17 | Cross-References | ✓ COMPLETE |

**Total defined instructions:** 50 (across 8 categories).
**Available extension slots:** 206 opcodes (128 reserved + 78 category gaps).

---

*End of SPEC-0302*
