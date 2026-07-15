# SPEC-0303: GVM Implementation Guide — Porting Guide for Implementers

| **Field** | **Value** |
|---|---|
| **Spec ID** | SPEC-0303 |
| **Title** | GVM Implementation Guide — Porting Guide for Implementers |
| **Version** | 1.0.0 |
| **Status** | Draft |
| **Depends on** | RFC-0003 (GVM Architecture), RFC-0002 (Bytecode Format), SPEC-0302 (Instruction Set), SPEC-0301 (Grammar Runtime) |
| **Related SPECs** | SPEC-0102 (Feature Taxonomy), SPEC-0001-C8 (Security), SPEC-0001-C9 (Performance) |
| **License** | AGOS Specification License v1.0 |

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Implementation Strategy & Ordering](#2-implementation-strategy--ordering)
3. [Language-Specific Implementation Guides](#3-language-specific-implementation-guides)
4. [Core Data Structures](#4-core-data-structures)
5. [The Instruction Dispatch Loop](#5-the-instruction-dispatch-loop)
6. [Memory Region Management](#6-memory-region-management)
7. [Bytecode Loader & Verifier](#7-bytecode-loader--verifier)
8. [Instruction Handler Implementations](#8-instruction-handler-implementations)
9. [Conformance Testing](#9-conformance-testing)
10. [Optimization Patterns](#10-optimization-patterns)
11. [Error Handling & Diagnostics](#11-error-handling--diagnostics)
12. [Common Pitfalls & Debugging](#12-common-pitfalls--debugging)
13. [Cross-Implementation Consistency](#13-cross-implementation-consistency)
14. [Performance Tuning](#14-performance-tuning)
15. [Cross-References](#15-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0303 provides the **practical implementation guide** for the Grammar Virtual Machine (GVM). Where RFC-0003 defines *what* the GVM does and SPEC-0302 defines *how each instruction works*, this document defines *how to build one*.

This specification is written for:

- **Primary implementers** building the canonical Rust GVM.
- **Porting implementers** building GVMs in C, Python, JavaScript/TypeScript, Go, Java, Swift, or any other language.
- **Bytecode generator developers** (MOD-09) who need to test their output against a real GVM.
- **Quality engineers** writing conformance tests and integration suites.

### 1.2 Relationship to Other Specifications

| Document | Provides | Role in Implementation |
|----------|----------|----------------------|
| **RFC-0003** | Architecture, design philosophy, state model | Foundation — read this first |
| **RFC-0002** | Binary bytecode format, section layout | Parser implementation target |
| **SPEC-0302** | Per-instruction encoding, stack effects, error codes | Instruction handler reference |
| **SPEC-0301** | Runtime integration, instance pooling, caching | Runtime environment |
| **SPEC-0001-C9** | Latency/memory/throughput targets | Performance goals |
| **SPEC-0001-C8** | Security model, sandbox requirements | Safety constraints |
| **SPEC-0303** (this) | Implementation order, code templates, testing | Implementation methodology |

### 1.3 Scope

**In scope:**

| Category | Coverage |
|----------|----------|
| **Implementation ordering** | Recommended build order with dependency justification |
| **Language-specific guides** | Rust (primary), C, Python, TypeScript, Go, Java — with code templates |
| **Core data structures** | Complete Rust struct definitions with explanations for porting |
| **Instruction dispatch** | Flat opcode table, handler signatures, macro expansions |
| **Bytecode parsing** | Section-by-section parsing pipeline with validation |
| **Memory management** | Region allocation, bounds checking, safety patterns |
| **Conformance testing** | Test structure, test data formats, CI integration |
| **Optimization patterns** | Hot-path inlining, instance pooling, caching |
| **Cross-implementation consistency** | Determinism requirements, test vectors, verification |

**Out of scope:**

| Topic | Covered By |
|-------|-----------|
| Instruction encoding details | SPEC-0302 |
| Bytecode binary format reference | RFC-0002 |
| Runtime integration (pooling, caching) | SPEC-0301 |
| Performance targets (latency, throughput) | SPEC-0001-C9 |

### 1.4 Implementation Principles

1. **Correctness before performance.** Get the instruction semantics right first. Optimize only after conformance tests pass. A fast but incorrect GVM is worse than no GVM.

2. **Determinism is a design constraint.** Every choice — hash function, sort order, iteration order, concurrent access pattern — must produce identical output for identical input. Prefer stable sorts, deterministic hash maps, and strict left-to-right evaluation.

3. **Test with byte-for-byte output comparisons.** The conformance test suite compares GVM output against canonical expected output. Use JSON serialization with deterministic key ordering for output comparison.

4. **Implement the verifier first.** A correct verifier that rejects invalid bytecode is simpler than a correct executor that must handle all edge cases. Verify before execute.

5. **One instruction at a time.** Implement each instruction in opcode order. Test each instruction in isolation before combining them. Use the conformance test's per-instruction tests.

---

## 2. Implementation Strategy & Ordering

### 2.1 Recommended Build Order

The GVM implementation is organized into 10 phases. Each phase builds on the previous one and has a clear completion milestone.

```
Phase 1: Foundation
    Build the bytecode parser, verifier, and data structures
    Milestone: Can parse and fully verify a valid .agos file

Phase 2: Stack & Memory
    Build the operand stack, call stack, and typed memory regions
    Milestone: Can push/pop values, allocate regions, bounds-check

Phase 3: Flow Control
    Implement HALT, JUMP, JUMP_IF_TRUE, JUMP_IF_FALSE, CALL, RETURN, DIE
    Milestone: Can execute a simple branching instruction sequence

Phase 4: Stack Operations
    Implement PUSH_I32, PUSH_I64, PUSH_F64, PUSH_BOOL, PUSH_STRING,
    PUSH_NULL, POP, DUP, SWAP
    Milestone: Can push values of all types and manipulate the stack

Phase 5: Token Operations
    Implement LOAD_TOKEN, TOKEN_GET_TEXT, TOKEN_GET_OFFSET,
    TOKEN_GET_TYPE, TOKEN_GET_FEATURES, TOKEN_ITERATE, TOKEN_COUNT
    Milestone: Can iterate over tokens and access their data

Phase 6: Feature Operations
    Implement FEATURE_GET, FEATURE_SET, FEATURE_HAS,
    FEATURE_COMPARE_EQ, FEATURE_COMPARE_MASK, FEATURE_PACK
    Milestone: Can extract, set, compare, and pack feature bitfields

Phase 7: Constituent Operations
    Implement CONST_MAKE, CONST_ADD_CHILD, CONST_GET_CHILD,
    CONST_GET_ROLE, CONST_SET_ROLE, CONST_ATTACH_TOKENS, CONST_TRAVERSE
    Milestone: Can build and traverse constituent trees

Phase 8: Rule & Evidence Operations
    Implement RULE_APPLY, RULE_CONFIRM, RULE_REJECT, RULE_MODIFY,
    RULE_FLAG, RULE_RESOLVE, EVIDENCE_PUSH, EVIDENCE_QUERY, EVIDENCE_EMIT
    Milestone: Can record rule applications and build evidence trails

Phase 9: Output Operations
    Implement OUTPUT_SET_METADATA, OUTPUT_ADD_TREE, OUTPUT_ADD_TOKEN,
    OUTPUT_SET_INPUT, OUTPUT_FINALIZE
    Milestone: Can produce a complete AnalysisResult

Phase 10: Integration & Optimization
    Assemble the complete pipeline, add the disassembler,
    add the tracer, implement instance pooling, profile and optimize
    Milestone: Passes all conformance tests; meets performance targets
```

### 2.2 Dependency Graph

```
Phase 1 (Parser/Verifier)
    │
    ▼
Phase 2 (Stack & Memory)
    │
    ├──► Phase 3 (Flow Control) ────┐
    │                               │
    ├──► Phase 4 (Stack Ops) ───────┤
    │                               │
    ├──► Phase 5 (Token Ops) ───────┤
    │                               │
    ├──► Phase 6 (Feature Ops) ─────┤
    │                               │
    ├──► Phase 7 (Constituent Ops) ──┤
    │                               │
    ├──► Phase 8 (Rule & Evidence) ──┤
    │                               │
    └──► Phase 9 (Output Ops) ──────┤
                                    ▼
                          Phase 10 (Integration)
```

### 2.3 Milestone Verification

| Phase | Verification | Criteria |
|-------|-------------|----------|
| 1 | `agos gvm verify --bytecode=test.agos` | Returns `valid: true` for valid files |
| 2 | `agos gvm run --bytecode=stack-test.agos` | Stack operations produce correct values |
| 3 | `agos gvm run --bytecode=flow-test.agos` | Correct branching and subroutines |
| 4 | `agos gvm run --bytecode=push-pop-test.agos` | All push/pop/dup/swap correct |
| 5a | `agos gvm run --bytecode=token-test.agos` | Correct token iteration and access |
| 5b | `agos gvm run --bytecode=loop-test.agos` | Token loop with feature extraction works |
| 6 | `agos gvm run --bytecode=feature-test.agos` | Feature get/set/compare correct |
| 7 | `agos gvm run --bytecode=constituent-test.agos` | Tree building and traversal correct |
| 8 | `agos gvm run --bytecode=rule-test.agos` | Rule recording and evidence correct |
| 9 | `agos gvm run --bytecode=full-analysis.agos` | Complete AnalysisResult generated |
| 10 | `agos gvm test --suite=conformance-v1` | All 163+ conformance tests pass |

---

## 3. Language-Specific Implementation Guides

### 3.1 Rust (Primary Implementation)

Rust is the **primary implementation language** for the GVM. The Rust GVM serves as the reference implementation against which all other implementations are tested.

#### 3.1.1 Project Structure

```
agos-gvm/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API (execute, verify, version)
│   ├── types.rs                  # Core types: Value, FeatureSet, etc.
│   ├── state.rs                  # GVMState, Stack, Region
│   ├── bytecode.rs               # Bytecode parser and section types
│   ├── verifier.rs               # Verification pipeline
│   ├── decoder.rs                # Instruction decoder (varint, operands)
│   ├── dispatch.rs               # Opcode dispatch table
│   ├── handlers/
│   │   ├── mod.rs                # Handler module
│   │   ├── flow.rs               # Flow control handlers
│   │   ├── stack.rs              # Stack operation handlers
│   │   ├── token.rs              # Token operation handlers
│   │   ├── feature.rs            # Feature operation handlers
│   │   ├── constituent.rs        # Constituent operation handlers
│   │   ├── rule.rs               # Rule operation handlers
│   │   ├── evidence.rs           # Evidence operation handlers
│   │   └── output.rs             # Output operation handlers
│   ├── memory.rs                 # Memory region implementation
│   ├── tracer.rs                 # Execution tracing (optional)
│   ├── disassembler.rs           # Bytecode disassembler
│   ├── error.rs                  # GVMError and error codes
│   ├── config.rs                 # GVMConfig, Profile
│   ├── pool.rs                   # GVMInstancePool
│   └── output.rs                 # AnalysisResult assembly
├── tests/
│   ├── conformance/              # Conformance test bytecodes
│   │   ├── flow/
│   │   ├── stack/
│   │   ├── token/
│   │   ├── feature/
│   │   ├── constituent/
│   │   ├── rule/
│   │   ├── evidence/
│   │   ├── output/
│   │   └── error/
│   └── fixtures/                 # Test fixture data
├── benches/
│   └── gvm_bench.rs              # Criterion benchmarks
└── bytecode-samples/             # Sample .agos files
```

#### 3.1.2 Key Type Definitions

```rust
// ── Core Value Type ──────────────────────────────────────────
/// All values that can appear on the operand stack.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F64(f64),
    StringIndex(u32),      // Index into string table
    TokenIndex(u32),       // Index into token region
    FeatureBits(u64),      // Packed morphological features
    ConstituentPtr(u32),   // Index into constituent region
    Bool(bool),
    Null,
}

impl Value {
    /// Type tag for runtime type checking.
    pub fn type_tag(&self) -> &'static str {
        match self {
            Value::I32(_)           => "i32",
            Value::I64(_)           => "i64",
            Value::F64(_)           => "f64",
            Value::StringIndex(_)   => "string_index",
            Value::TokenIndex(_)    => "token_index",
            Value::FeatureBits(_)   => "feature_bits",
            Value::ConstituentPtr(_) => "constituent_ptr",
            Value::Bool(_)          => "bool",
            Value::Null             => "null",
        }
    }
}

// ── Operand Stack ────────────────────────────────────────────
/// Bounded stack for instruction operands.
#[derive(Debug, Clone)]
pub struct Stack {
    data: Vec<Value>,
    max_depth: u32,
}

impl Stack {
    pub fn new(max_depth: u32) -> Self {
        Self { data: Vec::with_capacity(max_depth as usize), max_depth }
    }

    pub fn push(&mut self, value: Value) -> Result<(), GVMError> {
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

    pub fn pop(&mut self) -> Result<Value, GVMError> {
        self.data.pop().ok_or(GVMError::StackUnderflow)
    }

    pub fn peek(&self) -> Result<&Value, GVMError> {
        self.data.last().ok_or(GVMError::StackUnderflow)
    }

    pub fn peek_mut(&mut self) -> Result<&mut Value, GVMError> {
        self.data.last_mut().ok_or(GVMError::StackUnderflow)
    }

    pub fn dup(&mut self) -> Result<(), GVMError> {
        let val = self.peek()?.clone();
        self.push(val)
    }

    pub fn swap(&mut self) -> Result<(), GVMError> {
        let len = self.data.len();
        if len < 2 {
            return Err(GVMError::StackUnderflow);
        }
        self.data.swap(len - 1, len - 2);
        Ok(())
    }

    pub fn len(&self) -> usize { self.data.len() }
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    /// Pop a value and check its type tag.
    pub fn pop_typed(&mut self, expected: &str) -> Result<Value, GVMError> {
        let val = self.pop()?;
        if val.type_tag() != expected {
            return Err(GVMError::TypeError {
                expected: expected.to_string(),
                got: val.type_tag().to_string(),
                pc: 0, // Caller should fill this in
            });
        }
        Ok(val)
    }
}

// ── Typed Memory Region ──────────────────────────────────────
/// A pre-allocated, bounds-checked memory region for typed data.
#[derive(Debug, Clone)]
pub struct Region<T: Clone> {
    data: Vec<T>,
    capacity: u32,
}

impl<T: Clone> Region<T> {
    pub fn new(capacity: u32, default: T) -> Self {
        Self {
            data: Vec::with_capacity(capacity as usize),
            capacity,
        }
    }

    pub fn alloc(&mut self, value: T) -> Result<u32, GVMError> {
        if self.data.len() >= self.capacity as usize {
            return Err(GVMError::RegionFull {
                region: std::any::type_name::<T>().to_string(),
                size: self.data.len() as u32,
                capacity: self.capacity,
            });
        }
        let idx = self.data.len() as u32;
        self.data.push(value);
        Ok(idx)
    }

    pub fn get(&self, index: u32) -> Result<&T, GVMError> {
        self.data.get(index as usize).ok_or(GVMError::IndexOutOfBounds {
            index,
            count: self.data.len() as u32,
        })
    }

    pub fn get_mut(&mut self, index: u32) -> Result<&mut T, GVMError> {
        self.data.get_mut(index as usize).ok_or(GVMError::IndexOutOfBounds {
            index,
            count: self.data.len() as u32,
        })
    }

    pub fn size(&self) -> u32 { self.data.len() as u32 }
    pub fn capacity(&self) -> u32 { self.capacity }
    pub fn clear(&mut self) { self.data.clear(); }
}
```

#### 3.1.3 Handler Signature

```rust
/// Type alias for instruction handler functions.
/// Each handler takes flags, decoded operands, and mutable state.
type InstructionHandler = fn(
    flags: u8,
    operands: &[Operand],
    state: &mut GVMState,
) -> Result<(), GVMError>;
```

#### 3.1.4 Instruction Definition

```rust
/// Metadata for a single instruction in the dispatch table.
#[derive(Debug, Clone)]
pub struct InstructionDef {
    /// Human-readable mnemonic.
    pub mnemonic: &'static str,
    /// Number of operands.
    pub operand_count: u8,
    /// Types of each operand.
    pub operand_types: &'static [OperandType],
    /// Handler function.
    pub handler: InstructionHandler,
    /// Default step cost.
    pub steps: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperandType {
    U32,  // Unsigned varint (LEB128)
    I32,  // Signed varint (zigzag)
    U64,  // Fixed 8 bytes
    F64,  // Fixed 8 bytes, IEEE 754
    U8,   // Fixed 1 byte
}

/// Parsed operand value.
#[derive(Debug, Clone)]
pub enum Operand {
    U32(u32),
    I32(i32),
    U64(u64),
    F64(f64),
    U8(u8),
}

// ── Macro for defining instructions ──────────────────────────
macro_rules! instruction {
    ($mnemonic:expr, $count:expr, $types:expr, $handler:expr) => {
        InstructionDef {
            mnemonic: $mnemonic,
            operand_count: $count,
            operand_types: $types,
            handler: $handler,
            steps: 1,
        }
    };
}
```

### 3.2 C (Secondary Implementation)

C is the **C-compatible ecosystem implementation**. It targets embedded systems, FFI boundaries, and language runtimes that can call C.

#### 3.2.1 Core Types

```c
// ── Core Value Type ──────────────────────────────────────────
// Tagged union for operand stack values.
typedef struct {
    uint8_t type;  // 0=i32, 1=i64, 2=f64, 3=string_idx, 4=token_idx,
                   // 5=feature_bits, 6=const_ptr, 7=bool, 8=null
    union {
        int32_t i32;
        int64_t i64;
        double f64;
        uint32_t u32;    // string index, token index, constituent ptr
        uint64_t u64;    // feature bits
        uint8_t bool_val;
    } data;
} Value;

// ── Bounded Stack ────────────────────────────────────────────
typedef struct {
    Value* data;
    uint32_t size;
    uint32_t max_depth;
} Stack;

// ── Typed Region ─────────────────────────────────────────────
typedef struct {
    void* data;          // Pre-allocated block
    uint32_t entry_size; // Size of each entry in bytes
    uint32_t size;       // Current count
    uint32_t capacity;   // Maximum count
} Region;

// ── GVM State ────────────────────────────────────────────────
typedef struct {
    uint32_t pc;
    bool halted;
    GVMError error;

    Stack operand_stack;
    Stack call_stack;    // Stores return addresses as Value::I32

    Region token_region;
    Region feature_region;
    Region constituent_region;
    Region rule_region;
    Region evidence_region;

    uint8_t* string_table_data;  // Raw string table
    uint32_t* string_offsets;    // Offset of each string in data
    uint32_t string_count;

    uint64_t step_count;
    uint64_t max_steps;
    bool tracing;

    // Internal tracking
    int32_t current_token_index;
    int32_t last_rule_index;
} GVMState;

// ── Instruction Handler ─────────────────────────────────────
typedef int (*InstructionHandler)(GVMState* state, const Operand* operands,
                                   uint8_t operand_count);
```

#### 3.2.2 Thread Safety

```c
// The C GVM is NOT thread-safe by default. Each GVMState must
// be used by exactly one thread at a time. For concurrent use,
// create separate GVMState instances per thread.
//
// Thread safety is achieved by:
// 1. No shared mutable state between GVM instances
// 2. Each GVMState has its own memory regions
// 3. Bytecode is shared read-only (const uint8_t*)
```

#### 3.2.3 Memory Management

```c
// Pre-allocate all memory regions at initialization.
// No malloc/free during execution.

void gvm_init(GVMState* state, const BytecodeHeader* header) {
    // Token region: T token entries
    state->token_region.entry_size = sizeof(Token);
    state->token_region.capacity = header->token_count;
    state->token_region.size = 0;
    state->token_region.data = calloc(header->token_count, sizeof(Token));

    // Feature region: 8 bytes each (u64 bitfield)
    state->feature_region.entry_size = 8;
    state->feature_region.capacity = header->feature_count;
    state->feature_region.size = 0;
    state->feature_region.data = calloc(header->feature_count, 8);

    // Constituent region: ~48 bytes each
    state->constituent_region.entry_size = sizeof(ConstituentNode);
    state->constituent_region.capacity = header->constituent_count;
    state->constituent_region.size = 0;
    state->constituent_region.data = calloc(header->constituent_count,
                                            sizeof(ConstituentNode));

    // ... (remaining regions follow the same pattern)
}

void gvm_destroy(GVMState* state) {
    free(state->token_region.data);
    free(state->feature_region.data);
    free(state->constituent_region.data);
    free(state->rule_region.data);
    free(state->evidence_region.data);
    free(state->operand_stack.data);
    free(state->call_stack.data);
    free(state->string_offsets);
    free(state->string_table_data);
}
```

### 3.3 Python (Ecosystem Implementation)

Python is the **prototyping and education implementation**. Performance is 10–100× slower than Rust, but it enables rapid experimentation and teaching.

#### 3.3.1 Core Types

```python
from dataclasses import dataclass
from enum import Enum, auto
from typing import Any, Optional
from array import array

# ── Value Type ────────────────────────────────────────────────
@dataclass
class Value:
    class Type(Enum):
        I32 = auto(); I64 = auto(); F64 = auto()
        STRING_INDEX = auto(); TOKEN_INDEX = auto()
        FEATURE_BITS = auto(); CONSTITUENT_PTR = auto()
        BOOL = auto(); NULL = auto()

    type: Type
    data: Any  # int, float, bool, or None

    @classmethod
    def i32(cls, v: int) -> 'Value':
        return cls(cls.Type.I32, v)
    @classmethod
    def f64(cls, v: float) -> 'Value':
        return cls(cls.Type.F64, v)
    @classmethod
    def bool(cls, v: bool) -> 'Value':
        return cls(cls.Type.BOOL, v)
    @classmethod
    def null(cls) -> 'Value':
        return cls(cls.Type.NULL, None)
    @classmethod
    def feature_bits(cls, v: int) -> 'Value':
        return cls(cls.Type.FEATURE_BITS, v)
    @classmethod
    def token_index(cls, v: int) -> 'Value':
        return cls(cls.Type.TOKEN_INDEX, v)

# ── Stack ────────────────────────────────────────────────────
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

    def dup(self) -> None: self.push(self.peek())
    def swap(self) -> None:
        if len(self.data) < 2:
            raise GVMError.stack_underflow()
        self.data[-1], self.data[-2] = self.data[-2], self.data[-1]

# ── Region ───────────────────────────────────────────────────
class Region:
    """A typed, pre-allocated memory region."""
    def __init__(self, capacity: int):
        self.data: list[Any] = []
        self.capacity = capacity

    def alloc(self, item: Any) -> int:
        if len(self.data) >= self.capacity:
            raise GVMError(f"Region full ({len(self.data)}/{self.capacity})")
        idx = len(self.data)
        self.data.append(item)
        return idx

    def __getitem__(self, idx):
        if idx < 0 or idx >= len(self.data):
            raise GVMError(f"Index {idx} out of bounds ({len(self.data)})")
        return self.data[idx]

    def __setitem__(self, idx, value):
        if idx < 0 or idx >= len(self.data):
            raise GVMError(f"Index {idx} out of bounds ({len(self.data)})")
        self.data[idx] = value

    @property
    def size(self) -> int: return len(self.data)
```

#### 3.3.2 Instruction Decoder

```python
def decode_uleb128(data: bytes, offset: int) -> tuple[int, int]:
    """Decode an unsigned LEB128 varint. Returns (value, new_offset)."""
    result = 0
    shift = 0
    while True:
        byte = data[offset]
        result |= (byte & 0x7F) << shift
        offset += 1
        if (byte & 0x80) == 0:
            break
        shift += 7
    return result, offset

def decode_zigzag(value: int) -> int:
    """Decode a zigzag-encoded signed integer."""
    return (value >> 1) ^ -(value & 1)

def decode_instruction(data: bytes, pc: int) -> tuple:
    """Decode a single instruction. Returns (opcode, flags, operands, next_pc)."""
    opcode = data[pc]
    flags = data[pc + 1]
    offset = pc + 2
    operands = []

    # Get expected operand types from the opcode table
    op_types = OPCODE_TABLE[opcode]['operand_types']
    for op_type in op_types:
        if op_type == 'u32':
            val, offset = decode_uleb128(data, offset)
            operands.append(val)
        elif op_type == 'i32':
            val, offset = decode_uleb128(data, offset)
            operands.append(decode_zigzag(val))
        elif op_type == 'u64':
            val = int.from_bytes(data[offset:offset+8], 'little')
            operands.append(val); offset += 8
        elif op_type == 'f64':
            val = struct.unpack('<d', data[offset:offset+8])[0]
            operands.append(val); offset += 8
        elif op_type == 'u8':
            operands.append(data[offset]); offset += 1

    return opcode, flags, operands, offset
```

### 3.4 TypeScript/JavaScript (Ecosystem Implementation)

TypeScript is the **browser and Node.js implementation**. It uses TypedArrays for memory regions.

#### 3.4.1 Core Types

```typescript
// ── Value Type ────────────────────────────────────────────────
type Value =
    | { type: 'i32'; data: number }
    | { type: 'i64'; data: bigint }
    | { type: 'f64'; data: number }
    | { type: 'string_index'; data: number }
    | { type: 'token_index'; data: number }
    | { type: 'feature_bits'; data: bigint }
    | { type: 'constituent_ptr'; data: number }
    | { type: 'bool'; data: boolean }
    | { type: 'null' };

// ── Stack ────────────────────────────────────────────────────
class Stack {
    private data: Value[] = [];
    constructor(private maxDepth: number = 1024) {}

    push(v: Value): void {
        if (this.data.length >= this.maxDepth)
            throw new GVMError(`Stack overflow: ${this.data.length}/${this.maxDepth}`);
        this.data.push(v);
    }

    pop(): Value {
        const v = this.data.pop();
        if (v === undefined) throw new GVMError('Stack underflow');
        return v;
    }

    peek(): Value {
        const v = this.data[this.data.length - 1];
        if (v === undefined) throw new GVMError('Stack underflow');
        return v;
    }

    dup(): void { const v = this.peek(); this.push({...v}); }
    swap(): void {
        if (this.data.length < 2) throw new GVMError('Stack underflow');
        [this.data[this.data.length - 1], this.data[this.data.length - 2]] =
        [this.data[this.data.length - 2], this.data[this.data.length - 1]];
    }
}

// ── Typed Memory Region ─────────────────────────────────────
class Region<T> {
    private data: T[] = [];
    constructor(private capacity: number) {}

    alloc(item: T): number {
        if (this.data.length >= this.capacity)
            throw new GVMError(`Region full: ${this.data.length}/${this.capacity}`);
        const idx = this.data.length;
        this.data.push(item);
        return idx;
    }

    get(index: number): T {
        const item = this.data[index];
        if (item === undefined) throw new GVMError(`Index ${index} out of bounds`);
        return item;
    }

    set(index: number, value: T): void {
        if (index < 0 || index >= this.data.length)
            throw new GVMError(`Index ${index} out of bounds`);
        this.data[index] = value;
    }

    get size(): number { return this.data.length; }
}
```

#### 3.4.2 TypedArray Memory Regions

```typescript
// For performance-critical regions, use TypedArrays instead of arrays.
// This provides guaranteed O(1) access and memory efficiency.

class FeatureRegion {
    private data: BigUint64Array;
    private _size: number = 0;

    constructor(capacity: number) {
        this.data = new BigUint64Array(capacity);
    }

    alloc(bits: bigint): number {
        if (this._size >= this.data.length)
            throw new GVMError(`Feature region full`);
        const idx = this._size;
        this.data[idx] = bits;
        this._size++;
        return idx;
    }

    get(index: number): bigint {
        if (index < 0 || index >= this._size)
            throw new GVMError(`Feature index ${index} out of bounds`);
        return this.data[index];
    }

    set(index: number, bits: bigint): void {
        if (index < 0 || index >= this._size)
            throw new GVMError(`Feature index ${index} out of bounds`);
        this.data[index] = bits;
    }

    get size(): number { return this._size; }
}
```

### 3.5 Go (Ecosystem Implementation)

Go is the **cloud-native implementation**. It targets microservices and serverless deployments.

#### 3.5.1 Core Types

```go
// ── Value Type ────────────────────────────────────────────────
type ValueType uint8

const (
    TypeI32 ValueType = iota
    TypeI64
    TypeF64
    TypeStringIndex
    TypeTokenIndex
    TypeFeatureBits
    TypeConstituentPtr
    TypeBool
    TypeNull
)

type Value struct {
    Type ValueType
    I32  int32
    I64  int64
    F64  float64
    U32  uint32
    U64  uint64
    Bool bool
}

// ── Stack ────────────────────────────────────────────────────
type Stack struct {
    data     []Value
    maxDepth uint32
}

func NewStack(maxDepth uint32) *Stack {
    return &Stack{
        data:     make([]Value, 0, maxDepth),
        maxDepth: maxDepth,
    }
}

func (s *Stack) Push(v Value) error {
    if uint32(len(s.data)) >= s.maxDepth {
        return fmt.Errorf("stack overflow: %d/%d", len(s.data), s.maxDepth)
    }
    s.data = append(s.data, v)
    return nil
}

func (s *Stack) Pop() (Value, error) {
    if len(s.data) == 0 {
        return Value{}, fmt.Errorf("stack underflow")
    }
    v := s.data[len(s.data)-1]
    s.data = s.data[:len(s.data)-1]
    return v, nil
}

func (s *Stack) Dup() error {
    v, err := s.Peek()
    if err != nil { return err }
    return s.Push(v)
}

func (s *Stack) Swap() error {
    if len(s.data) < 2 {
        return fmt.Errorf("stack underflow: need 2 elements, have %d", len(s.data))
    }
    s.data[len(s.data)-1], s.data[len(s.data)-2] =
        s.data[len(s.data)-2], s.data[len(s.data)-1]
    return nil
}
```

#### 3.5.2 Instruction Dispatch Pattern

```go
// Go's type switch makes instruction dispatch clean and fast.
func (vm *GVM) executeInstruction(opcode uint8, operands []Operand) error {
    switch opcode {
    case 0x00: return vm.handleHalt()
    case 0x01: return vm.handleJump(operands[0].(int32))
    case 0x02: return vm.handleJumpIfTrue(operands[0].(int32))
    case 0x03: return vm.handleJumpIfFalse(operands[0].(int32))
    case 0x04: return vm.handleCall(operands[0].(int32))
    case 0x05: return vm.handleReturn()
    case 0x06: return vm.handleDie(operands[0].(uint32))
    case 0x10: return vm.handlePushI32(operands[0].(int32))
    // ... remaining opcodes
    default:
        return fmt.Errorf("invalid opcode: 0x%02X", opcode)
    }
}
```

### 3.6 Java (Ecosystem Implementation)

Java is the **Android and enterprise implementation**.

#### 3.6.1 Core Types

```java
// ── Value Type (Boxed) ───────────────────────────────────────
public sealed interface Value {
    record I32(int value) implements Value {}
    record I64(long value) implements Value {}
    record F64(double value) implements Value {}
    record StringIndex(int index) implements Value {}
    record TokenIndex(int index) implements Value {}
    record FeatureBits(long bits) implements Value {}
    record ConstituentPtr(int index) implements Value {}
    record Bool(boolean value) implements Value {}
    record Null() implements Value {}

    default String typeTag() {
        return switch (this) {
            case I32 _ -> "i32"; case I64 _ -> "i64";
            case F64 _ -> "f64"; case StringIndex _ -> "string_index";
            case TokenIndex _ -> "token_index"; case FeatureBits _ -> "feature_bits";
            case ConstituentPtr _ -> "constituent_ptr"; case Bool _ -> "bool";
            case Null _ -> "null";
        };
    }
}

// ── Stack ────────────────────────────────────────────────────
public final class Stack {
    private final ArrayList<Value> data = new ArrayList<>();
    private final int maxDepth;

    public Stack(int maxDepth) { this.maxDepth = maxDepth; }

    public void push(Value v) {
        if (data.size() >= maxDepth)
            throw new GVMError("Stack overflow: " + data.size() + "/" + maxDepth);
        data.add(v);
    }

    public Value pop() {
        if (data.isEmpty()) throw new GVMError("Stack underflow");
        return data.remove(data.size() - 1);
    }

    public Value peek() {
        if (data.isEmpty()) throw new GVMError("Stack underflow");
        return data.get(data.size() - 1);
    }

    public void dup() { push(peek()); }
    public void swap() {
        if (data.size() < 2) throw new GVMError("Stack underflow");
        Collections.swap(data, data.size() - 1, data.size() - 2);
    }
}
```

#### 3.6.2 Switch Expression Dispatch

```java
// Java 21+ switch expressions provide clean dispatch.
Result executeInstruction(int opcode, Operand[] operands) {
    return switch (opcode) {
        case 0x00 -> handleHalt();
        case 0x01 -> handleJump(operands[0].asI32());
        case 0x02 -> handleJumpIfTrue(operands[0].asI32());
        case 0x03 -> handleJumpIfFalse(operands[0].asI32());
        // ... remaining opcodes
        default -> Result.err("Invalid opcode: 0x" + Integer.toHexString(opcode));
    };
}
```

---

## 4. Core Data Structures

### 4.1 GVMState (Complete Definition)

```rust
/// Complete runtime state of a GVM instance.
/// All memory is pre-allocated at creation time.
pub struct GVMState {
    // ── Execution Control ──
    pub pc: u32,                            // Program counter
    pub halted: bool,                       // Execution completed
    pub error: Option<GVMError>,            // Execution error

    // ── Stacks ──
    pub operand_stack: Stack,               // Max depth: 1024
    pub call_stack: Stack,                  // Call/return frames (stored as I32)

    // ── Typed Memory Regions ──
    pub token_region: Region<Token>,
    pub feature_region: Region<u64>,
    pub constituent_region: Region<ConstituentNode>,
    pub rule_region: Region<RuleRecord>,
    pub evidence_region: Region<EvidenceRecord>,
    pub scratch_region: ScratchBuffer,

    // ── String Table (immutable, from bytecode) ──
    pub strings: Vec<String>,               // Indexed by string_index

    // ── Metrics ──
    pub step_count: u64,
    pub max_steps: u64,
    pub started_at: Instant,

    // ── Internal Tracking ──
    pub current_token_index: i32,           // Set by LOAD_TOKEN, read by RULE ops
    pub last_rule_index: Option<u32>,       // Index into rule_region for RULE ops

    // ── Output Buffer ──
    pub output_metadata: Vec<(u32, u32)>,   // (key_index, value_index)
    pub output_trees: Vec<OutputTreeDef>,
    pub output_flags: Vec<FlagRecord>,
    pub output_finalized: bool,

    // ── Anaphora ──
    pub anaphora_resolutions: Vec<(u32, u32)>, // (antecedent, pronoun)
}
```

### 4.2 Porting GVMState to Other Languages

| Component | Rust | C | Python | TypeScript | Go | Java |
|-----------|------|---|--------|------------|----|------|
| Region | `Region<T>` (generic) | `struct Region` (raw bytes + entry_size) | `class Region` (list) | `class Region<T>` (generic array) | `struct Region` (slice) | `class Region<T>` (generic ArrayList) |
| Stack | `Stack` (Vec wrapper) | `struct Stack` (array + size) | `class Stack` (list wrapper) | `class Stack` (array wrapper) | `struct Stack` (slice wrapper) | `class Stack` (ArrayList wrapper) |
| Value | `enum Value` (tagged union) | `struct Value` (tagged union) | `Value` (dataclass + Enum) | `type Value` (union) | `struct Value` (fields + type) | `sealed interface Value` |
| Strings | `Vec<String>` | `uint8_t* + uint32_t*` offsets | `list[str]` | `string[]` | `[]string` | `List<String>` |
| Error | `enum GVMError` | `int error_code` + `char* message` | `class GVMError(Exception)` | `class GVMError extends Error` | `type GVMError struct` | `class GVMError extends RuntimeException` |

### 4.3 Porting Constants

```rust
/// Default capacities and limits — MUST match across implementations.
impl Default for GVMConfig {
    fn default() -> Self {
        Self {
            max_execution_steps: 100_000,
            max_memory_bytes: 64 * 1024 * 1024,    // 64 MiB
            sandbox_mode: true,
            tracing_enabled: false,
            max_call_depth: 64,
            max_stack_depth: 1024,
            performance_profile: Profile::Server,
        }
    }
}

/// Default memory region capacities — MUST match across implementations.
mod default_capacities {
    pub const TOKENS: u32        = 256;
    pub const FEATURES: u32      = 512;
    pub const CONSTITUENTS: u32  = 1024;
    pub const RULES: u32         = 512;
    pub const EVIDENCE: u32      = 1024;
    pub const SCRATCH: u32       = 4096;  // bytes
}
```

---

## 5. The Instruction Dispatch Loop

### 5.1 Canonical Execution Loop

```rust
/// The main execution loop: fetch → decode → validate → execute → update.
/// Every GVM implementation MUST implement this exact cycle.
pub fn execute_cycle(state: &mut GVMState, bytecode: &[u8]) -> Result<(), GVMError> {
    // Step limit check before each instruction
    if state.step_count >= state.max_steps {
        return Err(GVMError::MaxStepsExceeded {
            steps: state.step_count,
            limit: state.max_steps,
        });
    }

    // 1. FETCH — read opcode and flags
    let opcode = bytecode.get(state.pc as usize)
        .ok_or(GVMError::ProgramCounterOutOfBounds { pc: state.pc })?;
    let flags = bytecode.get(state.pc as usize + 1)
        .ok_or(GVMError::ProgramCounterOutOfBounds { pc: state.pc + 1 })?;

    // 2. LOOKUP — find instruction definition
    let instr = OPCODE_TABLE[opcode as usize]
        .ok_or(GVMError::InvalidOpcode { opcode: *opcode, pc: state.pc })?;

    // 3. DECODE — parse operands
    let (operands, consumed) = decode_operands(bytecode, state.pc + 2, instr)?;

    // 4. VALIDATE — type-check and bounds-check operands
    validate_operands(&operands, state)?;

    // 5. EXECUTE — dispatch to handler
    (instr.handler)(*flags, &operands, state)?;

    // 6. UPDATE — advance PC and step count
    state.pc += 2 + consumed;  // 2 for opcode + flags
    state.step_count += 1;

    // 7. TRACE (optional)
    if state.tracing_enabled {
        record_trace(state, *opcode, &operands);
    }

    Ok(())
}
```

### 5.2 Main Execution Loop (All Languages)

```python
# Python execution loop
def execute(self, bytecode: bytes) -> AnalysisResult:
    while not self.state.halted:
        if self.state.step_count >= self.state.max_steps:
            raise GVMError.max_steps_exceeded(
                self.state.step_count, self.state.max_steps)

        opcode, flags, operands, next_pc = decode_instruction(
            bytecode, self.state.pc)

        if opcode not in self.INSTRUCTION_TABLE:
            raise GVMError.invalid_opcode(opcode, self.state.pc)

        handler = self.INSTRUCTION_TABLE[opcode]
        handler(self.state, flags, operands)

        self.state.pc = next_pc
        self.state.step_count += 1

        if self.state.tracing:
            self.record_trace(opcode, operands)

    return self.assemble_output()
```

```go
// Go execution loop
func (vm *GVM) Execute(bytecode []byte) (*AnalysisResult, error) {
    for !vm.state.Halted {
        if vm.state.StepCount >= vm.state.MaxSteps {
            return nil, fmt.Errorf("max steps exceeded: %d/%d",
                vm.state.StepCount, vm.state.MaxSteps)
        }

        opcode := bytecode[vm.state.PC]
        flags := bytecode[vm.state.PC+1]
        operands, consumed, err := decodeOperands(bytecode, vm.state.PC+2, opcode)
        if err != nil { return nil, err }

        if err := vm.executeInstruction(opcode, operands); err != nil {
            return nil, err
        }

        vm.state.PC += 2 + uint32(consumed)
        vm.state.StepCount++

        if vm.config.TracingEnabled {
            vm.recordTrace(opcode, operands)
        }
    }

    return vm.assembleOutput(), nil
}
```

### 5.3 Decode Operands

```rust
/// Decode instruction operands from the bytecode stream.
/// Returns (operands, bytes_consumed).
fn decode_operands(
    bytecode: &[u8],
    start: usize,
    instr: &InstructionDef,
) -> Result<(Vec<Operand>, u32), GVMError> {
    let mut operands = Vec::with_capacity(instr.operand_count as usize);
    let mut offset = start;

    for op_type in instr.operand_types {
        let (op, consumed) = match op_type {
            OperandType::U32 => {
                let (val, new_off) = decode_uleb128(bytecode, offset)?;
                (Operand::U32(val), new_off - offset)
            }
            OperandType::I32 => {
                let (val, new_off) = decode_uleb128(bytecode, offset)?;
                (Operand::I32(decode_zigzag(val)), new_off - offset)
            }
            OperandType::U64 => {
                let val = u64::from_le_bytes(
                    bytecode[offset..offset+8].try_into().unwrap());
                (Operand::U64(val), 8)
            }
            OperandType::F64 => {
                let val = f64::from_le_bytes(
                    bytecode[offset..offset+8].try_into().unwrap());
                (Operand::F64(val), 8)
            }
            OperandType::U8 => {
                (Operand::U8(bytecode[offset]), 1)
            }
        };
        operands.push(op);
        offset += consumed;
    }

    Ok((operands, (offset - start) as u32))
}
```

### 5.4 Validate Operands

```rust
/// Validate decoded operands against current state bounds.
fn validate_operands(operands: &[Operand], state: &GVMState) -> Result<(), GVMError> {
    for op in operands {
        match op {
            Operand::U32(val) if *val >= state.token_region.size() => {
                // Only validate token indices; other U32 values are validated
                // by their specific handlers
            }
            _ => {}  // Type validation is done by handlers
        }
    }
    Ok(())
}
```

---

## 6. Memory Region Management

### 6.1 Region Initialization

All memory regions are initialized once when a GVM instance is created. There is NO dynamic allocation during execution.

```rust
/// Initialize all memory regions for a GVM instance.
pub fn init_regions(state: &mut GVMState, header: &BytecodeHeader) {
    state.token_region = Region::new(
        header.token_count.max(default_capacities::TOKENS));
    state.feature_region = Region::new(
        header.feature_count.max(default_capacities::FEATURES));
    state.constituent_region = Region::new(
        header.constituent_count.max(default_capacities::CONSTITUENTS));
    state.rule_region = Region::new(
        header.rule_count.max(default_capacities::RULES));
    state.evidence_region = Region::new(
        default_capacities::EVIDENCE);
    state.scratch_region = ScratchBuffer::new(
        default_capacities::SCRATCH);
}
```

### 6.2 Region Sizing Calculation

```rust
/// Calculate the total memory budget for a GVM instance.
pub fn calculate_memory_budget(header: &BytecodeHeader) -> u64 {
    let token_size      = (header.token_count.max(default_capacities::TOKENS) as u64) * TOKEN_SIZE;
    let feature_size    = (header.feature_count.max(default_capacities::FEATURES) as u64) * 8;
    let constituent_size = (header.constituent_count.max(default_capacities::CONSTITUENTS) as u64) * CONSTITUENT_SIZE;
    let rule_size       = (header.rule_count.max(default_capacities::RULES) as u64) * RULE_SIZE;
    let evidence_size   = (default_capacities::EVIDENCE as u64) * EVIDENCE_SIZE;
    let stack_size      = (1024) * 16;  // 1024 values × 16 bytes
    let call_stack_size = (64) * 4;     // 64 return addresses × 4 bytes

    token_size + feature_size + constituent_size + rule_size +
    evidence_size + default_capacities::SCRATCH as u64 +
    stack_size + call_stack_size + 4096 /* overhead */
}

const TOKEN_SIZE: u64       = 48;   // Token struct
const CONSTITUENT_SIZE: u64 = 48;   // ConstituentNode struct
const RULE_SIZE: u64        = 32;   // RuleRecord struct
const EVIDENCE_SIZE: u64    = 64;   // EvidenceRecord struct
```

### 6.3 Bounds Checking Pattern

Every memory access in every implementation MUST be bounds-checked:

```rust
// Rust: Region::get() always checks bounds
let token = state.token_region.get(index)?;

// C: explicit bounds check
if (index >= state->token_region.size) return GVM_ERROR_TOKEN_INDEX_OUT_OF_BOUNDS;

// Python: Region.__getitem__ checks
token = self.state.token_region[index]

// TypeScript: explicit check
if (index >= this.tokenRegion.size) throw new GVMError('Token index out of bounds');

// Go: explicit check
if index >= len(vm.state.tokens) { return fmt.Errorf("token index %d out of bounds", index) }

// Java: explicit check
if (index >= tokenRegion.size()) throw new GVMError("Token index out of bounds: " + index);
```

---

## 7. Bytecode Loader & Verifier

### 7.1 Section Parsing Pipeline

```rust
/// Parse a bytecode file into its constituent sections.
pub fn parse_bytecode(data: &[u8]) -> Result<ParsedBytecode, GVMError> {
    // 1. Parse the fixed-size header (32+ bytes)
    let header = parse_header(data)?;

    // 2. Parse the section table (N × 10 bytes)
    let sections = parse_section_table(data, header.section_count)?;

    // 3. Parse each section by type
    let metadata = parse_metadata_section(data, &sections[0])?;
    let string_table = parse_string_table(data, &sections[1])?;
    let tokens = parse_token_section(data, &sections[2], &string_table)?;
    let features = parse_feature_section(data, &sections[3])?;
    let constituents = parse_constituent_section(data, &sections[4])?;
    let instructions = parse_instruction_section(data, &sections[5])?;

    // 4. Parse optional sections
    let rules = sections.get(6)
        .map(|s| parse_rule_section(data, s, &string_table)).transpose()?;
    let evidence = sections.get(7)
        .map(|s| parse_evidence_section(data, s, &string_table)).transpose()?;

    Ok(ParsedBytecode {
        header, metadata, string_table,
        tokens, features, constituents,
        instructions, rules, evidence,
    })
}
```

### 7.2 Verification Pipeline

```rust
/// Full bytecode verification. All checks MUST pass before execution.
pub fn verify_bytecode(bytecode: &ParsedBytecode) -> VerificationResult {
    let mut issues = Vec::new();

    // 1. Magic bytes
    verify_magic(&bytecode.header, &mut issues);

    // 2. Version compatibility
    verify_version(&bytecode.header, &mut issues);

    // 3. CRC32C checksums (per section)
    verify_checksums(&bytecode, &mut issues);

    // 4. Section ordering
    verify_section_order(&bytecode.header, &mut issues);

    // 5. Instruction stream validity
    verify_instructions(&bytecode.instructions, &mut issues);

    // 6. Jump target validation
    verify_jump_targets(&bytecode.instructions, &mut issues);

    // 7. String table integrity
    verify_string_table(&bytecode.string_table, &mut issues);

    // 8. Feature ID validation
    verify_feature_ids(&bytecode.instructions, &mut issues);

    // 9. Resource bounds
    verify_resource_bounds(&bytecode, &mut issues);

    VerificationResult {
        valid: issues.iter().all(|i| i.severity != Severity::Error),
        issues,
    }
}
```

### 7.3 Instruction Stream Validation

```rust
/// Validate the instruction stream for structural correctness.
fn verify_instruction_stream(
    instructions: &[u8],
    string_count: u32,
    token_count: u32,
) -> Vec<VerificationIssue> {
    let mut issues = Vec::new();
    let mut pc = 0;

    // Walk through all instructions
    while pc < instructions.len() {
        let opcode = instructions[pc];

        // 1. Check opcode is valid
        let instr = match OPCODE_TABLE.get(opcode as usize) {
            Some(i) => i,
            None => {
                issues.push(VerificationIssue::error(
                    "INVALID_OPCODE",
                    format!("Unknown opcode 0x{opcode:02X} at offset {pc}"),
                    Some(pc as u32),
                ));
                // Skip 2 bytes and try to continue
                pc += 2;
                continue;
            }
        };

        // 2. Decode operands to validate them
        let mut offset = pc + 2;
        for op_type in instr.operand_types {
            match op_type {
                OperandType::U32 | OperandType::I32 => {
                    let (val, new_off) = decode_uleb128(instructions, offset).unwrap_or((0, offset + 1));
                    offset = new_off;

                    // Validate string references
                    if instr.mnemonic == "PUSH_STRING" || instr.mnemonic == "RULE_APPLY" {
                        if val >= string_count {
                            issues.push(VerificationIssue::error(
                                "STRING_INDEX_OUT_OF_BOUNDS",
                                format!("String index {val} out of bounds (count: {string_count}) at offset {pc}"),
                                Some(pc as u32),
                            ));
                        }
                    }

                    // Validate token references
                    if instr.mnemonic == "LOAD_TOKEN" {
                        if val >= token_count {
                            issues.push(VerificationIssue::error(
                                "TOKEN_INDEX_OUT_OF_BOUNDS",
                                format!("Token index {val} out of bounds (count: {token_count}) at offset {pc}"),
                                Some(pc as u32),
                            ));
                        }
                    }
                }
                OperandType::U64 => offset += 8,
                OperandType::F64 => offset += 8,
                OperandType::U8  => offset += 1,
            }
        }

        pc = offset;
    }

    // 3. Check last instruction is HALT or DIE
    if instructions.len() >= 2 {
        let last_opcode = instructions[instructions.len() - 2];
        if last_opcode != 0x00 && last_opcode != 0x06 {
            issues.push(VerificationIssue::warning(
                "MISSING_TERMINATOR",
                "Last instruction is not HALT or DIE".into(),
                Some((instructions.len() - 2) as u32),
            ));
        }
    }

    issues
}
```

### 7.4 Varint Decoding (All Languages)

```rust
// ── Rust ─────────────────────────────────────────────────────
fn decode_uleb128(data: &[u8], offset: usize) -> Result<(u64, usize), GVMError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut pos = offset;

    loop {
        let byte = *data.get(pos).ok_or(GVMError::BytecodeCorrupted {
            issues: vec![VerificationIssue::error(
                "TRUNCATED_VARINT", "Unexpected end of bytecode".into(), None)]
        })?;
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift > 63 {
            return Err(GVMError::BytecodeCorrupted {
                issues: vec![VerificationIssue::error(
                    "VARINT_TOO_LONG", "Varint exceeds 64 bits".into(), None)]
            });
        }
    }

    Ok((result, pos))
}

fn decode_zigzag(val: u64) -> i64 {
    ((val >> 1) as i64) ^ -((val & 1) as i64)
}
```

```c
// ── C ────────────────────────────────────────────────────────
uint64_t decode_uleb128(const uint8_t* data, uint32_t* offset) {
    uint64_t result = 0;
    int shift = 0;
    uint8_t byte;
    do {
        byte = data[(*offset)++];
        result |= (uint64_t)(byte & 0x7F) << shift;
        shift += 7;
    } while (byte & 0x80);
    return result;
}
```

```python
# ── Python ────────────────────────────────────────────────────
def decode_uleb128(data: bytes, offset: int) -> tuple[int, int]:
    result = 0
    shift = 0
    while True:
        byte = data[offset]
        result |= (byte & 0x7F) << shift
        offset += 1
        if (byte & 0x80) == 0:
            break
        shift += 7
    return result, offset
```

---

## 8. Instruction Handler Implementations

### 8.1 Handler Template (Rust)

Every instruction handler follows the same pattern:

```rust
/// Template for an instruction handler.
pub fn handle_instruction_example(
    flags: u8,
    operands: &[Operand],
    state: &mut GVMState,
) -> Result<(), GVMError> {
    // 1. Extract operands
    let operand = operands[0].as_u32()?;

    // 2. Pop values from operand stack (check types)
    let val_a = state.operand_stack.pop_typed("i32")?;
    let val_b = state.operand_stack.pop_typed("feature_bits")?;

    // 3. Perform operation (bounds-check all accesses)
    let result = do_something(val_b.as_u64(), val_a.as_i32());

    // 4. Push result back
    state.operand_stack.push(Value::FeatureBits(result))?;

    // 5. Return Ok or error
    Ok(())
}
```

### 8.2 Handler: FEATURE_GET (Rust)

```rust
/// 0x30 — FEATURE_GET: Extract a feature value from bitfield.
pub fn handle_feature_get(
    flags: u8,
    operands: &[Operand],
    state: &mut GVMState,
) -> Result<(), GVMError> {
    // Operand 0: feature_id (u32)
    let feature_id = operands[0].as_u32()?;

    // Validate feature ID
    if feature_id > 19 {
        return Err(GVMError::InvalidFeatureId { feature_id });
    }

    // Pop feature bitfield from stack
    let bits = state.operand_stack.pop_typed("feature_bits")?;
    let bits_u64 = match bits {
        Value::FeatureBits(b) => b,
        _ => unreachable!(), // pop_typed ensures this
    };

    // Extract using bitmask + shift
    let value = extract_feature_from_bitfield(bits_u64, feature_id);

    // Push result as i32
    state.operand_stack.push(Value::I32(value as i32))?;

    Ok(())
}
```

### 8.3 Handler: FEATURE_SET (Rust)

```rust
/// 0x31 — FEATURE_SET: Set a feature value in bitfield.
pub fn handle_feature_set(
    flags: u8,
    operands: &[Operand],
    state: &mut GVMState,
) -> Result<(), GVMError> {
    let feature_id = operands[0].as_u32()?;
    if feature_id > 19 {
        return Err(GVMError::InvalidFeatureId { feature_id });
    }

    let new_val = state.operand_stack.pop_typed("i32")?;
    let bits = state.operand_stack.pop_typed("feature_bits")?;

    let bits_val = match bits { Value::FeatureBits(b) => b, _ => unreachable!() };
    let val_val = match new_val { Value::I32(v) => v as u64, _ => unreachable!() };

    let (shift, mask) = FEATURE_BIT_MASK[feature_id as usize];
    let result = (bits_val & !(mask << shift)) | ((val_val & mask) << shift);

    state.operand_stack.push(Value::FeatureBits(result))?;
    Ok(())
}
```

### 8.4 Handler: CONST_MAKE (Rust)

```rust
/// 0x40 — CONST_MAKE: Create a new constituent node.
pub fn handle_const_make(
    flags: u8,
    operands: &[Operand],
    state: &mut GVMState,
) -> Result<(), GVMError> {
    let role_id = operands[0].as_u32()?;

    // Pop child count, then children
    let child_count_val = state.operand_stack.pop_typed("i32")?;
    let child_count = match child_count_val { Value::I32(c) => c as usize, _ => unreachable!() };

    let mut children = Vec::with_capacity(child_count);
    for _ in 0..child_count {
        let child = state.operand_stack.pop_typed("constituent_ptr")?;
        match child {
            Value::ConstituentPtr(p) => {
                // Validate child pointer
                if p >= state.constituent_region.size() {
                    return Err(GVMError::IndexOutOfBounds {
                        index: p, count: state.constituent_region.size()
                    });
                }
                children.push(p);
            }
            _ => unreachable!()
        }
    }

    // Allocate new node
    let node = ConstituentNode {
        role: role_id,
        children,
        token_indices: Vec::new(),
    };
    let ptr = state.constituent_region.alloc(node)?;

    state.operand_stack.push(Value::ConstituentPtr(ptr))?;
    Ok(())
}
```

### 8.5 Feature Bit Mask Table

```rust
/// Feature bit mask table used by FEATURE_GET, FEATURE_SET, FEATURE_PACK.
/// Indexed by feature_id. Returns (shift, mask).
pub const FEATURE_BIT_MASK: [(u8, u64); 20] = [
    (0,  0xF),      // 0:  pos           bits 0-3
    (4,  0x3),      // 1:  gender        bits 4-5
    (6,  0x3),      // 2:  number        bits 6-7
    (8,  0x3),      // 3:  person        bits 8-9
    (10, 0x3),      // 4:  tense         bits 10-11
    (12, 0x3),      // 5:  mood          bits 12-13
    (14, 0x1),      // 6:  voice         bit  14
    (15, 0x3),      // 7:  case          bits 15-16
    (17, 0x1),      // 8:  state         bit  17
    (0,  0),        // 9:  (reserved)
    (18, 0x1F),     // 10: verb_form     bits 18-22
    (23, 0x1F),     // 11: noun_type     bits 23-27
    (28, 0xF),      // 12: pronoun_type  bits 28-31
    (32, 0xF),      // 13: transitivity  bits 32-35
    (36, 0xF),      // 14: root_type     bits 36-39
    (40, 0x7),      // 15: stress        bits 40-42
    (43, 0xF),      // 16: syllable_cnt  bits 43-46
    (47, 0x1),      // 17: has_shadda    bit  47
    (48, 0x1),      // 18: has_madd      bit  48
    (49, 0x1),      // 19: has_hamza     bit  49
];
```

---

## 9. Conformance Testing

### 9.1 Test Structure

Each conformance test consists of:

1. **A `.agos` bytecode file** — a minimal, self-contained bytecode for a single test scenario.
2. **A `.json` expected output file** — the expected AnalysisResult or error.
3. **A test runner** — executes the bytecode and compares the output.

```
tests/conformance/
├── flow/
│   ├── halt.agos              halt.json
│   ├── jump_forward.agos      jump_forward.json
│   ├── jump_backward.agos     jump_backward.json
│   ├── jump_if_true.agos      jump_if_true.json
│   ├── jump_if_false.agos     jump_if_false.json
│   ├── call_return.agos       call_return.json
│   └── die.agos               die.json
├── stack/
│   ├── push_i32.agos          push_i32.json
│   ├── push_i64.agos          push_i64.json
│   ├── push_f64.agos          push_f64.json
│   ├── push_bool.agos         push_bool.json
│   ├── push_string.agos       push_string.json
│   ├── push_null.agos         push_null.json
│   ├── pop.agos               pop.json
│   ├── dup.agos               dup.json
│   └── swap.agos              swap.json
├── token/
│   ├── load_token.agos        load_token.json
│   ├── token_count.agos       token_count.json
│   ├── token_iterate.agos     token_iterate.json
│   ├── token_get_text.agos    token_get_text.json
│   └── token_get_features.agos token_get_features.json
├── feature/
│   ├── feature_get.agos       feature_get.json
│   ├── feature_set.agos       feature_set.json
│   ├── feature_has.agos       feature_has.json
│   ├── feature_compare_eq.agos feature_compare_eq.json
│   ├── feature_compare_mask.agos feature_compare_mask.json
│   └── feature_pack.agos      feature_pack.json
├── constituent/
│   ├── const_make.agos        const_make.json
│   ├── const_add_child.agos   const_add_child.json
│   ├── const_get_child.agos   const_get_child.json
│   ├── const_get_role.agos    const_get_role.json
│   ├── const_set_role.agos    const_set_role.json
│   ├── const_attach.agos      const_attach.json
│   └── const_traverse.agos    const_traverse.json
├── rule/
│   ├── rule_apply.agos        rule_apply.json
│   ├── rule_confirm.agos      rule_confirm.json
│   ├── rule_reject.agos       rule_reject.json
│   ├── rule_modify.agos       rule_modify.json
│   ├── rule_flag.agos         rule_flag.json
│   └── rule_resolve.agos      rule_resolve.json
├── evidence/
│   ├── evidence_push.agos     evidence_push.json
│   ├── evidence_query.agos    evidence_query.json
│   └── evidence_emit.agos     evidence_emit.json
├── output/
│   ├── output_set_metadata.agos output_set_metadata.json
│   ├── output_add_tree.agos   output_add_tree.json
│   ├── output_add_token.agos  output_add_token.json
│   ├── output_set_input.agos  output_set_input.json
│   └── output_finalize.agos   output_finalize.json
├── error/
│   ├── stack_underflow.agos   stack_underflow.json
│   ├── type_error.agos        type_error.json
│   ├── invalid_opcode.agos    invalid_opcode.json
│   ├── token_oob.agos         token_oob.json
│   ├── string_oob.agos        string_oob.json
│   ├── call_stack_overflow.agos call_stack_overflow.json
│   └── invalid_feature_id.agos invalid_feature_id.json
└── integration/
    ├── full_analysis.agos      full_analysis.json
    └── complex_tree.agos       complex_tree.json
```

### 9.2 Expected Output Format

```jsonc
{
    "spec": "SPEC-0303/conformance-test",
    "version": "1.0.0",
    "test_name": "feature_get_gender",
    "description": "FEATURE_GET with feature_id=1 (gender) on a masculine bitfield",

    "config": {
        "max_execution_steps": 100,
        "max_stack_depth": 1024
    },

    "expected": {
        "status": "completed",          // "completed" | "error" | "timeout"
        "error_code": null,             // null if completed, error code string if error

        "metadata": {
            "steps_executed": 3,        // PUSH + FEATURE_GET + HALT
            "execution_success": true
        },

        // Stack state after execution (top first)
        "final_stack": [
            { "type": "i32", "value": 0 }   // masculine = 0
        ],

        // Output trees (if any)
        "output_trees": [],

        // Flags (if any)
        "flags": []
    }
}
```

### 9.3 Test Runner (Rust)

```rust
/// Generic conformance test runner.
fn run_conformance_test(test_path: &Path) -> Result<(), TestingError> {
    // 1. Load bytecode
    let bytecode_bytes = std::fs::read(test_path.join("test.agos"))?;

    // 2. Load expected output
    let expected: ExpectedOutput =
        serde_json::from_str(&std::fs::read_to_string(test_path.join("expected.json"))?)?;

    // 3. Create GVM and execute
    let config = GVMConfig {
        max_execution_steps: expected.config.max_execution_steps,
        ..Default::default()
    };
    let mut gvm = GVM::new(config);
    let result = gvm.execute(&bytecode_bytes);

    // 4. Compare against expected
    match (&result, &expected.expected.status) {
        (Ok(analysis), Status::Completed) => {
            assert_eq!(analysis.metadata.steps_executed,
                       expected.expected.metadata.steps_executed);
            // Compare stack, trees, flags...
        }
        (Err(error), Status::Error) => {
            assert_eq!(error.code(), expected.expected.error_code.as_ref().unwrap());
        }
        (Ok(_), Status::Error) => {
            return Err(TestingError::ExpectedErrorButGotSuccess(test_path));
        }
        (Err(_), Status::Completed) => {
            return Err(TestingError::ExpectedSuccessButGotError(test_path));
        }
    }

    Ok(())
}

/// Run all conformance tests.
#[test]
fn conformance_test_suite() {
    let test_dir = Path::new("tests/conformance");
    let mut passed = 0;
    let mut failed = 0;

    for category in std::fs::read_dir(test_dir).unwrap() {
        let category_path = category.unwrap().path();
        if !category_path.is_dir() { continue; }

        for test in std::fs::read_dir(&category_path).unwrap() {
            let test_path = test.unwrap().path();
            if !test_path.is_dir() { continue; }

            match run_conformance_test(&test_path) {
                Ok(()) => passed += 1,
                Err(e) => {
                    eprintln!("FAIL: {} — {}", test_path.display(), e);
                    failed += 1;
                }
            }
        }
    }

    eprintln!("Conformance: {passed} passed, {failed} failed");
    assert_eq!(failed, 0, "{failed} conformance tests failed");
}
```

### 9.4 Cross-Implementation Test Vectors

To ensure cross-implementation consistency, the conformance suite includes **test vectors** — byte-for-byte identical `.agos` files paired with expected outputs.

```
Test vectors are generated by the reference Rust implementation and
MUST produce identical outputs in all other implementations.

Test vector directory structure:
tests/vectors/
├── v1.0.0/
│   ├── manifest.json           # List of all test vectors with descriptions
│   ├── flow/
│   │   ├── halt.agos           # Binary bytecode
│   │   └── halt.json           # Expected output
│   ├── stack/
│   │   └── ...
│   ├── integration/
│   │   ├── full_analysis.agos  # 10-word sentence analysis
│   │   └── full_analysis.json
│   └── ...
```

---

## 10. Optimization Patterns

### 10.1 Hot-Path Inlining

The most common instruction sequence is token iteration + feature extraction. Optimize by fusing common patterns:

```rust
// ── Instruction Fusion: TOKEN_ITERATE + LOAD_TOKEN ──────────
// Instead of:
//   TOKEN_ITERATE offset
//   ... (loop body)
//   LOAD_TOKEN stack[-3]  // load current token
//
// Fuse into:
//   TOKEN_ITERATE_LOAD offset  // loads current token automatically

// ── Pre-computed Feature Extractors ─────────────────────────
// Instead of:
//   FEATURE_GET 1    // gender (mask+shift)
//   FEATURE_GET 2    // number (mask+shift)
// Use pre-computed combined extractors:
fn extract_gender_and_number(bits: u64) -> (u32, u32) {
    let combined = (bits >> 4) & 0xF;  // Extract bits 4-7 at once
    (combined & 0x3, (combined >> 2) & 0x3)
}
```

### 10.2 Opcode Dispatch Optimization

```rust
// ── Indirect Dispatch Table (fast) ──────────────────────────
// Use a flat array of function pointers indexed by opcode.
// This is O(1) and branch-predictor-friendly.

// Rust: static array of function pointers
type Handler = fn(u8, &[Operand], &mut GVMState) -> Result<(), GVMError>;
const DISPATCH: [Option<Handler>; 256] = { ... };

// C: array of function pointers
typedef GVMError (*Handler)(GVMState*, const Operand*, uint8_t);
static const Handler DISPATCH[256] = { ... };

// ── Jump Table (alternative) ────────────────────────────────
// For languages without function pointers (or where they are slow),
// use a computed goto (C) or labeled goto (GCC extension).

// C (GCC computed goto):
static const void* dispatch[] = {
    [0x00] = &&HALT,
    [0x01] = &&JUMP,
    // ...
};
goto *dispatch[opcode];
```

### 10.3 Memory Region Optimization

```rust
// ── Contiguous Memory Layout ────────────────────────────────
// Allocate regions as contiguous blocks. Use pointer arithmetic
// within each region rather than Vec/ArrayList overhead.

// Rust: Vec<T> is fine (memory-contiguous for T)
// C: calloc() for each region
// Python: list is NOT contiguous (PyObject pointers). Use array('Q') for features.

// ── Scratch Buffer as Stack ─────────────────────────────────
// Use the scratch buffer as a secondary stack for intermediate
// values instead of pushing/popping the operand stack.

// ── Lazy String Table Access ────────────────────────────────
// Don't decode all strings at load time. Only decode when
// PUSH_STRING is executed. Cache decoded strings.

type LazyStringTable = Vec<Option<String>>;  // None = not yet decoded

fn get_string(table: &mut LazyStringTable, raw_data: &[u8],
              offsets: &[u32], index: u32) -> &str {
    if table[index as usize].is_none() {
        let start = offsets[index as usize] as usize;
        let end = offsets[(index + 1) as usize] as usize;
        table[index as usize] = Some(
            String::from_utf8_lossy(&raw_data[start..end]).to_string()
        );
    }
    table[index as usize].as_ref().unwrap()
}
```

### 10.4 Instance Pooling

```rust
/// Thread-safe pool of pre-initialized GVM instances.
/// Avoids allocation overhead per execution request.
pub struct GVMInstancePool {
    instances: Mutex<Vec<GVMInstance>>,
    max_pool_size: u32,
    bytecode_header: BytecodeHeader,  // Shared among all instances
}

impl GVMInstancePool {
    pub fn new(config: &GVMConfig, header: &BytecodeHeader, size: u32) -> Self {
        let mut instances = Vec::with_capacity(size as usize);
        for _ in 0..size {
            let mut state = GVMState::new(config);
            init_regions(&mut state, header);
            instances.push(GVMInstance { state, in_use: false });
        }
        Self {
            instances: Mutex::new(instances),
            max_pool_size: size,
            bytecode_header: header.clone(),
        }
    }

    pub fn acquire(&self) -> GVMInstance {
        loop {
            let mut guard = self.instances.lock().unwrap();
            if let Some(inst) = guard.iter_mut().find(|i| !i.in_use) {
                inst.in_use = true;
                inst.state.reset_regions();
                return inst.clone();  // Actually return a RAII guard
            }
            // Pool empty — create new or wait
            if guard.len() < self.max_pool_size as usize {
                let mut state = GVMState::new(&GVMConfig::default());
                init_regions(&mut state, &self.bytecode_header);
                guard.push(GVMInstance { state, in_use: true });
                return guard.last().unwrap().clone();
            }
            // All instances busy — spin or use parking_lot::Cvar
        }
    }
}
```

---

## 11. Error Handling & Diagnostics

### 11.1 Error Propagation Pattern

```rust
/// Result type used throughout the GVM.
pub type GVMResult<T> = Result<T, GVMError>;

/// All possible GVM errors.
#[derive(Debug, Clone)]
pub enum GVMError {
    // Bytecode errors (detected at load/verify time)
    UnsupportedBytecodeVersion { bytecode: (u16, u16, u16), gvm: (u16, u16, u16) },
    BytecodeCorrupted { issues: Vec<VerificationIssue> },
    InvalidMagic,
    SectionMissing { section_id: u8 },
    SectionOrderingViolation,

    // Execution errors
    MaxStepsExceeded { steps: u64, limit: u64 },
    MaxMemoryExceeded { memory: u64, limit: u64 },
    ProgramCounterOutOfBounds { pc: u32 },

    // Stack errors
    StackOverflow { stack_type: String, depth: u32, max: u32 },
    StackUnderflow,
    TypeError { expected: String, got: String, pc: u32 },

    // Instruction errors
    InvalidOpcode { opcode: u8, pc: u32 },
    JumpOutOfBounds { target_pc: u32, max_pc: u32 },

    // Region errors
    IndexOutOfBounds { index: u32, count: u32 },  // Generalized
    RegionFull { region: String, size: u32, capacity: u32 },

    // Specific region errors
    InvalidFeatureId { feature_id: u32 },
    ChildIndexOutOfBounds { index: u32, count: u32 },
    InvalidConstituentPtr { ptr: u32 },

    // Context errors
    NoRuleInContext,
    ScratchOverflow { cursor: u32, capacity: u32 },

    // Internal
    InternalError { description: String },
}

impl GVMError {
    /// Human-readable error code for diagnostics.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedBytecodeVersion { .. } => "UNSUPPORTED_BYTECODE_VERSION",
            Self::BytecodeCorrupted { .. }          => "BYTECODE_CORRUPTED",
            Self::InvalidMagic                      => "INVALID_MAGIC",
            Self::MaxStepsExceeded { .. }           => "MAX_STEPS_EXCEEDED",
            Self::StackOverflow { .. }              => "STACK_OVERFLOW",
            Self::StackUnderflow                    => "STACK_UNDERFLOW",
            Self::TypeError { .. }                  => "TYPE_ERROR",
            Self::InvalidOpcode { .. }              => "INVALID_OPCODE",
            Self::JumpOutOfBounds { .. }            => "JUMP_OUT_OF_BOUNDS",
            Self::IndexOutOfBounds { .. }           => "INDEX_OUT_OF_BOUNDS",
            Self::RegionFull { .. }                 => "REGION_FULL",
            Self::InvalidFeatureId { .. }           => "INVALID_FEATURE_ID",
            Self::InternalError { .. }              => "INTERNAL_ERROR",
            _ => "UNKNOWN_ERROR",
        }
    }

    /// Whether the error is recoverable (non-fatal).
    pub fn is_fatal(&self) -> bool {
        match self {
            Self::MaxStepsExceeded { .. }
            | Self::StackOverflow { .. }
            | Self::TypeError { .. }
            | Self::InvalidOpcode { .. }
            | Self::InternalError { .. } => true,
            _ => true, // All GVM errors are currently fatal
        }
    }
}
```

### 11.2 Tracing Support

```rust
/// Execution trace entry enabled when tracing is on.
#[derive(Debug, Clone, Serialize)]
pub struct TraceEntry {
    pub instruction_number: u64,
    pub pc: u32,
    pub opcode: u8,
    pub mnemonic: String,
    pub operands: Vec<String>,
    pub stack_before: Vec<String>,
    pub stack_after: Vec<String>,
    pub wall_time_ns: u64,
}

/// Record a trace entry for the current instruction.
fn record_trace(
    state: &mut GVMState,
    opcode: u8,
    operands: &[Operand],
) {
    let instr = OPCODE_TABLE[opcode as usize].unwrap();

    let stack_before = state.operand_stack.data
        .iter().rev().take(5).map(|v| format!("{:?}", v)).collect();

    // Execute instruction
    // (trace must be called AFTER execution so we capture after-state)

    let stack_after = state.operand_stack.data
        .iter().rev().take(5).map(|v| format!("{:?}", v)).collect();

    state.trace.push(TraceEntry {
        instruction_number: state.step_count,
        pc: state.pc,
        opcode,
        mnemonic: instr.mnemonic.to_string(),
        operands: operands.iter().map(|o| format!("{:?}", o)).collect(),
        stack_before,
        stack_after,
        wall_time_ns: 0, // Measure with Instant::now() deltas
    });
}
```

---

## 12. Common Pitfalls & Debugging

### 12.1 Pitfall Table

| # | Pitfall | Symptom | Root Cause | Prevention |
|---|---------|---------|------------|------------|
| 1 | **Off-by-one jump target** | Wrong branch taken | Jump offset calculation error | Unit-test every JUMP offset; verify targets in conformance tests |
| 2 | **Stack underflow in complex expressions** | GVM crashes with "stack underflow" | Operation pops more values than available | Trace execution with `--trace` flag; check stack balance |
| 3 | **Endianness mismatch** | Feature values interpreted incorrectly | Multi-byte values (u32, u64, f64) read with wrong byte order | Always use little-endian; test on big-endian if possible |
| 4 | **Varint overflow (>64 bits)** | Varint decoder reads >10 bytes | Malformed bytecode or infinite loop in decoder | Limit varint to 10 bytes; reject overflow |
| 5 | **Infinite loop** | GVM step limit hit | Loop condition never becomes false | Always check step counter; never skip step count increment |
| 6 | **Type confusion on stack** | Wrong type tag on Value | Handler pushes wrong type or pops without checking | Use `pop_typed()` instead of `pop()`; check types in all handlers |
| 7 | **Forgetting to update PC** | Infinite loop on same instruction | Handler returns Ok but PC not advanced | PC is advanced in execute_cycle, not in handler |
| 8 | **Double-PC update** | Skipped instructions | Handler also advances PC | Never modify PC inside a handler (except JUMP/CALL/RETURN) |
| 9 | **Not resetting regions on pool reuse** | Stale data from previous execution | Region data persists between executions | Call `clear()` on all regions before reuse |
| 10 | **Mismatch with RFC-0002 reserved bits** | Feature validation fails | Bits 40–63 not zeroed | Always mask bitfield to 40 bits after set/pack operations |

### 12.2 Debugging Checklist

When a GVM implementation fails a conformance test:

```
1. VERIFY the bytecode file
   □ Does `agos gvm verify --bytecode=test.agos` pass?
   □ Are all section checksums valid?
   □ Is the version compatible?

2. DISASSEMBLE the bytecode
   □ Does `agos gvm disassemble --bytecode=test.agos` produce expected output?
   □ Are all opcodes recognized?
   □ Are jump targets correct?

3. TRACE execution
   □ Run with `--trace=trace.json`
   □ For each instruction:
     □ Is the opcode decoded correctly?
     □ Are operands decoded correctly?
     □ Is the stack state before/after correct?
     □ Is the PC advancing correctly?

4. CHECK output
   □ Does the final stack match expected?
   □ Does the AnalysisResult structure match?
   □ Are all expected output trees present?

5. COMPARE with reference implementation
   □ Run the same bytecode through the Rust reference GVM
   □ Compare output byte-for-byte (JSON with sorted keys)
   □ If different: check the trace differences
```

### 12.3 Common Debugging Commands

```bash
# Verify bytecode
agos gvm verify --bytecode=test.agos

# Disassemble to text
agos gvm disassemble --bytecode=test.agos

# Execute with tracing
agos gvm run --bytecode=test.agos --trace=trace.json

# Show trace summary
agos gvm trace-summary --trace=trace.json

# Run single conformance test
agos gvm test --bytecode=tests/conformance/flow/halt.agos

# Compare two trace files
agos gvm trace-diff --trace1=ref.json --trace2=impl.json

# Profile execution
agos gvm profile --bytecode=test.agos
```

---

## 13. Cross-Implementation Consistency

### 13.1 Determinism Requirements

Every GVM implementation MUST produce **byte-for-byte identical output** for identical bytecode input. This means:

| Aspect | Requirement | Reason |
|--------|-------------|--------|
| **Hash maps** | Use deterministic (not randomized) hash maps | Same iteration order every run |
| **Sorting** | Use stable sorts | Equal elements maintain insertion order |
| **Randomness** | No random values, no seed-based operations | Same input → same output |
| **Floating point** | Same IEEE 754 operations (no FMA contractions) | Same confidence values everywhere |
| **Concurrency** | No concurrent instruction execution | Deterministic instruction order |
| **Time-dependent behavior** | No wall-clock-based decisions | Execution is CPU-time-independent |

### 13.2 Canonical Test Vectors

The conformance test suite provides canonical test vectors generated by the Rust reference implementation:

```
tests/vectors/v1.0.0/
├── manifest.json                # All test vectors with SHA-256 of expected output
├── flow/
│   ├── halt.agos                # Bytecode
│   ├── halt.json                # Expected output (from Rust impl)
│   └── halt.json.sha256         # Checksum of expected output
├── stack/
│   ├── push_i32.agos
│   ├── push_i32.json
│   └── push_i32.json.sha256
├── feature/
│   ├── feature_pack.agos
│   ├── feature_pack.json
│   └── feature_pack.json.sha256
└── integration/
    ├── analysis_result.agos
    ├── analysis_result.json
    └── analysis_result.json.sha256
```

### 13.3 Cross-Implementation Test Runner

```bash
# Run the cross-implementation test suite
agos-gvm-test --impl=<path-to-gvm-binary> --suite=vectors-v1.0.0

# Compare against reference (Rust) output
agos-gvm-test --cmp-ref --impl=<path-to-gvm-binary>

# Generate cross-implementation report
agos-gvm-test --report=consistency.html
```

### 13.4 Implementation Status Matrix

| Feature | Rust | C | Python | TypeScript | Go | Java |
|---------|------|---|--------|------------|----|------|
| Bytecode parser | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Verifier | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Flow control (7) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Stack operations (9) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Token operations (7) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Feature operations (6) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Constituent ops (7) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Rule operations (6) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Evidence ops (3) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Output operations (5) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| All 163 conformance tests | ✅ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| Performance targets | ✅ | ⬜ | N/A | N/A | ⬜ | ⬜ |

---

## 14. Performance Tuning

### 14.1 Profiling Guide

```bash
# 1. Profile overall GVM execution
agos gvm profile --bytecode=benchmark.agos --profile=server

# 2. Profile per-instruction latency
agos gvm profile --bytecode=benchmark.agos --per-instruction

# 3. Profile memory usage
agos gvm profile --bytecode=benchmark.agos --memory

# 4. Compare profiles across versions
agos gvm profile-diff --profile1=v1.profile.json --profile2=v2.profile.json
```

### 14.2 Bottleneck Identification

| Bottleneck | Symptom | Typical Cause | Solution |
|------------|---------|---------------|----------|
| **Instruction dispatch** | High per-instruction latency | `match` or `if-else` chain | Use flat dispatch table (function pointer array) |
| **Varint decoding** | High decode time | Repeated bounds checks | Batch-decode at load time; cache decoded varints |
| **Feature extraction** | High time in FEATURE_GET | Repeated mask+shift | Pre-compute extractors; use combined extract |
| **String access** | Time in PUSH_STRING | All strings decoded at load | Lazy string decoding |
| **Memory allocation** | Time in `malloc`/`free` | Per-execution allocation | Pre-allocate and pool |
| **Stack bounds check** | Branches in hot path | Bounds check on every push/pop | Use unchecked access for trusted internal use |

### 14.3 Optimization Targets

| Optimization | Expected Gain | Implementation Effort | Priority |
|-------------|---------------|----------------------|----------|
| Flat dispatch table | 5–15% | Low (change dispatch structure) | High |
| Instance pooling | 5–10× cold start | Medium (add pool management) | High |
| Lazy string decoding | 10–30% load time | Low (change string table logic) | High |
| Inline feature extractors | 10–20% feature ops | Medium (pre-compute shift/mask) | Medium |
| Fused token+feature ops | 10–20% fewer instr | High (new fused opcodes) | Low |
| JIT compilation (future) | 10–50× | Very high (post-v1.0) | Future |

### 14.4 Benchmark Scenarios

```rust
// Criterion benchmark scenarios
fn bench_simple_sentence(c: &mut Criterion) {
    let bytecode = load_bytecode("benches/fixtures/simple.agos");
    let config = GVMConfig::default();

    c.bench_function("gvm_simple_sentence", |b| {
        b.iter(|| {
            let mut gvm = GVM::new(config.clone());
            gvm.execute(&bytecode).unwrap();
        });
    });
}

fn bench_complex_sentence(c: &mut Criterion) {
    let bytecode = load_bytecode("benches/fixtures/complex.agos");
    let config = GVMConfig {
        performance_profile: Profile::Server,
        ..Default::default()
    };

    c.bench_function("gvm_complex_sentence", |b| {
        b.iter(|| {
            let mut gvm = GVM::new(config.clone());
            gvm.execute(&bytecode).unwrap();
        });
    });
}

fn bench_instance_pool(c: &mut Criterion) {
    let bytecode = load_bytecode("benches/fixtures/simple.agos");
    let config = GVMConfig::default();

    let mut group = c.benchmark_group("gvm_pool");
    group.throughput(Throughput::Elements(1));

    // Without pooling
    group.bench_function("no_pool", |b| {
        b.iter(|| {
            let mut gvm = GVM::new(config.clone());
            gvm.execute(&bytecode).unwrap();
        });
    });

    // With pooling
    let pool = GVMInstancePool::new(&config, &bytecode.header(), 16);
    group.bench_function("with_pool", |b| {
        b.iter(|| {
            let mut inst = pool.acquire();
            inst.execute(&bytecode).unwrap();
        });
    });

    group.finish();
}
```

---

## 15. Cross-References

### 15.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| RFC-0003 | GVM Architecture & Execution Model | Foundation that this guide implements |
| RFC-0003 §10 | Implementation Guidance | Recommended ordering, language-specific notes, pitfalls |
| RFC-0003 §11 | Conformance Test Suite | 163+ tests that every GVM must pass |
| RFC-0002 | Bytecode Format | Binary format parsed by the loader |
| RFC-0002 §4 | Primitive Encoding | Varint, zigzag, fixed int encoding |
| SPEC-0301 §4 | GVM Integration & Lifecycle | Instance management, pooling, config |
| SPEC-0301 §5 | GVM Execution Pipeline | Instruction cycle, dispatch, cost model |
| SPEC-0301 §7 | Diagnostics & Verification | Tracing, disassembler, CLI |
| SPEC-0302 | GVM Instruction Set | Per-instruction details, cost model, error codes |
| SPEC-0302 §15 | Error Code Reference | Complete error code table |
| SPEC-0102 §8 | 64-Bit Bitfield Reference | Bitfield layout for feature operations |
| SPEC-0001-C9 | Performance Targets | Latency/memory/throughput goals |
| SPEC-0001-C8 | Security Model | Sandboxing, bounds checking requirements |

### 15.2 External References

| Reference | Relevance |
|-----------|-----------|
| JVM Specification | Stack-based VM implementation patterns, class file verification |
| WebAssembly Specification | Sandboxed VM design, module structure |
| Lua 5.4 Implementation | Minimal VM in C, instruction dispatch patterns |
| CPython Bytecode Interpreter | Stack-based VM in C, evaluation loop |
| LEB128 (DWARF Standard) | Variable-length integer encoding |
| IEEE 754 | Floating-point representation |
| criterion.rs | Rust benchmarking library |

---

## Progress Summary

**SPEC-0303: GVM Implementation Guide — Porting Guide for Implementers**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Implementation Strategy & Ordering | ✓ COMPLETE (10 phases with milestones) |
| 3 | Language-Specific Implementation Guides | ✓ COMPLETE (Rust, C, Python, TS, Go, Java) |
| 4 | Core Data Structures | ✓ COMPLETE (cross-language porting table) |
| 5 | The Instruction Dispatch Loop | ✓ COMPLETE (Rust, Python, Go loops) |
| 6 | Memory Region Management | ✓ COMPLETE (initialization, sizing, bounds) |
| 7 | Bytecode Loader & Verifier | ✓ COMPLETE (parsing, verification pipeline) |
| 8 | Instruction Handler Implementations | ✓ COMPLETE (template, 4 handler examples) |
| 9 | Conformance Testing | ✓ COMPLETE (test structure, format, runner) |
| 10 | Optimization Patterns | ✓ COMPLETE (fusion, dispatch, memory, pooling) |
| 11 | Error Handling & Diagnostics | ✓ COMPLETE (error types, tracing) |
| 12 | Common Pitfalls & Debugging | ✓ COMPLETE (10 pitfalls, checklist, commands) |
| 13 | Cross-Implementation Consistency | ✓ COMPLETE (determinism, vectors, runner) |
| 14 | Performance Tuning | ✓ COMPLETE (profiling, bottlenecks, benchmarks) |
| 15 | Cross-References | ✓ COMPLETE |

---

*End of SPEC-0303*
