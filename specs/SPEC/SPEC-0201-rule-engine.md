---
spec_id: SPEC-0201
title: Rule Engine — Detailed Implementation Specification
version: 1.0.0
status: Draft
author: AGOS Rule Engine Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0001-C8: Security, Validation & Error Handling
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0101: Morphology Engine
  - SPEC-0401: Knowledge Graph Engine
  - RFC-0001: Grammar DSL
  - RFC-0002: Grammar Bytecode Format
  - ADR-0003: Why Grammar IR
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---

# SPEC-0201: Rule Engine — Detailed Implementation Specification

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Internal Component Model](#3-internal-component-model)
4. [Rule Loading & Compilation](#4-rule-loading--compilation)
5. [Rule Execution Engine](#5-rule-execution-engine)
6. [Ambiguity Pruning & Management](#6-ambiguity-pruning--management)
7. [Anaphora Resolution](#7-anaphora-resolution)
8. [School-Specific Rule Sets](#8-school-specific-rule-sets)
9. [Conflict Detection & Resolution](#9-conflict-detection--resolution)
10. [Evidence & Flag Generation](#10-evidence--flag-generation)
11. [Performance & Optimization](#11-performance--optimization)
12. [Plugin Integration](#12-plugin-integration)
13. [Testing & Validation Strategy](#13-testing--validation-strategy)
14. [Implementation Guidance](#14-implementation-guidance)
15. [Cross-References](#15-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0201 provides the **detailed implementation specification** for the AGOS Rule Engine (MOD-07), which applies school-specific grammatical rules to the Grammar Intermediate Representation (GIR, IR-6) to produce the AnnotatedGIR (IR-7).

The Rule Engine is the **central disambiguation and verification stage** in the AGOS pipeline. It receives a GIR containing the full ambiguity forest (all possible morphological and syntactic analyses), applies the configured grammar school's rule set, and produces an annotated GIR with:

- **Confirmed analyses** — paths that satisfy grammatical rules
- **Rejected analyses** — paths that violate grammatical rules
- **Modified features** — features corrected by rule application (e.g., case assignment, mood government)
- **Grammatical flags** — errors, warnings, and informational notices
- **Complete evidence trail** — every rule application recorded for auditability

### 1.2 Scope

| In Scope | Out of Scope |
|----------|-------------|
| MOD-07 internal architecture (4 subsystems) | MOD-06 (GIRConstructor) specification |
| Rule loading, parsing, and compilation | MOD-08 (KnowledgeGraphResolver) |
| DSL rule execution (interpreter) | MOD-09 (BytecodeGenerator) |
| Ambiguity pruning algorithm | KB-0001 through KB-0007 content |
| Conflict detection and resolution | RFC-0001 Grammar DSL syntax design |
| Anaphora resolution algorithm | School-specific rule content (authored as DSL files) |
| Evidence trail generation from rules | GVM execution model (RFC-0003) |
| Plugin integration for rule sets | Deployment and operational concerns |
| Performance budget for MOD-07 | Cache management internals |

### 1.3 Relationship to Other Specifications

```diff
  Pipeline Position:
    MOD-06 (GIRConstructor) → MOD-07 (RuleEngine) → MOD-08 (KnowledgeGraphResolver)
                                    │
                                    ▼
                              IR-7 (AnnotatedGIR)

  Specification Hierarchy:
    SPEC-0001 ──── Architectural foundation
      │
      ├── SPEC-0101: Morphology Engine (MOD-04, MOD-05)
      ├── SPEC-0201: Rule Engine (this document — MOD-07)
      ├── SPEC-0301: Grammar Runtime (MOD-10, MOD-11)
      ├── SPEC-0401: Knowledge Graph Engine (MOD-08)
      ├── SPEC-0501: Explanation Engine (MOD-11)
      └── SPEC-0601: Plugin System (MOD-12)
```

### 1.4 Knowledge Dependencies

| KB | Content | Purpose in MOD-07 |
|----|---------|-------------------|
| KB-0005 | Particles | Particle governance rules (case assignment, mood government) |
| KB-0006 | Pronouns | Pronoun types and anaphora resolution rules |
| KB-0007 | Morphological Features | Feature agreement rules, cross-feature constraints |

### 1.5 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Rule-first disambiguation** | The Rule Engine is the ONLY stage that may prune ambiguity. No other stage discards alternative analyses. This ensures the evidence trail is complete and auditable. |
| **Deterministic rule execution** | Rules are applied in a fixed, priority-ordered sequence. No randomness, no ML-based ordering. Same input + same rule set = same output always. |
| **Mutable copy, immutable original** | The Rule Engine operates on a mutable copy of the GIR. The original GIR is preserved for debugging and comparison. |
| **Fixpoint detection** | If a full pass through all rules produces no state change, the engine terminates. This prevents infinite loops from circular rule dependencies. |
| **Data-driven school config** | School-specific behavior is entirely data-driven via rule set files. No hard-coded school logic in the engine. Adding a new school = adding a new rule set. |
| **Strict mode for fail-fast** | In strict mode, any rule conflict causes an error. In normal mode, conflicts preserve ambiguity. This supports development/debugging while allowing graceful degradation in production. |

---

## 2. Architecture Overview

### 2.1 MOD-07 Internal Architecture

```diff
  MOD-07: RuleEngine — Internal Architecture

  Input: GrammarIR (IR-6 from MOD-06)
  ┌────────────────────────────────────────────────────────────────────┐
  │                                                                    │
  │  ┌─────────────────────────────────────────────────────────────┐   │
  │  │ Subsystem 1: Rule Loader & Compiler                         │   │
  │  │                                                             │   │
  │  │  1a. Locate rule set for configured school + version       │   │
  │  │  1b. Parse .agosrule files into AST                        │   │
  │  │  1c. Compile AST into executable Rule objects              │   │
  │  │  1d. Resolve imports and deduplicate                      │   │
  │  │  1e. Sort rules by priority (descending)                  │   │
  │  │  1f. Cache compiled rule set for reuse                     │   │
  │  └─────────────────────────────────────────────────────────────┘   │
  │                              │                                     │
  │                              ▼                                     │
  │  ┌─────────────────────────────────────────────────────────────┐   │
  │  │ Subsystem 2: Rule Execution Engine                          │   │
  │  │                                                             │   │
  │  │  For each rule (priority order):                            │   │
  │  │    2a. Create mutable copy of GIR (workflow)               │   │
  │  │    2b. Evaluate rule condition against workflow            │   │
  │  │    2c. If condition matches → execute rule action          │   │
  │  │    2d. Record RuleApplication in evidence trail            │   │
  │  │    2e. Increment application counter                       │   │
  │  │    2f. Detect fixpoint (no state change → terminate)       │   │
  │  └─────────────────────────────────────────────────────────────┘   │
  │                              │                                     │
  │                              ▼                                     │
  │  ┌─────────────────────────────────────────────────────────────┐   │
  │  │ Subsystem 3: Ambiguity Manager                              │   │
  │  │                                                             │   │
  │  │  3a. Track confirmed/rejected analyses per rule             │   │
  │  │  3b. Prune rejected analyses from workflow                  │   │
  │  │  3c. Sort remaining alternatives by confidence             │   │
  │  │  3d. Detect unresolvable ambiguity                         │   │
  │  │  3e. Detect and resolve conflicts                          │   │
  │  └─────────────────────────────────────────────────────────────┘   │
  │                              │                                     │
  │                              ▼                                     │
  │  ┌─────────────────────────────────────────────────────────────┐   │
  │  │ Subsystem 4: Output Assembler                               │   │
  │  │                                                             │   │
  │  │  4a. Build AnnotatedGIR from workflow + rule applications   │   │
  │  │  4b. Generate grammatical flags                             │   │
  │  │  4c. Compile evidence trail                                 │   │
  │  │  4d. Validate output structure                              │   │
  │  └─────────────────────────────────────────────────────────────┘   │
  │                              │                                     │
  │                              ▼                                     │
  │  Output: AnnotatedGIR (IR-7 to MOD-08)                            │
  └────────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow Through MOD-07

```diff
  IR-6 (GrammarIR)      +      School Config (school, version, strict_mode)
    │                                    │
    └──────────────┬─────────────────────┘
                   │
                   ▼
  ┌───────────────────────────────────────────────┐
  │  1. Load Rule Set                              │
  │     ├── Resolve school + version               │
  │     ├── Parse .agosrule files                  │
  │     ├── Compile to Rule[]                      │
  │     ├── Sort by priority                       │
  │     └── Cache for reuse                        │
  └───────────────────────────────────────────────┘
                   │
                   ▼
  ┌───────────────────────────────────────────────┐
  │  2. Execute Rules (in priority order)          │
  │                                                │
  │  For each rule r in rules:                     │
  │    ├── Copy state (workflow = clone(GIR))      │
  │    ├── Evaluate condition against workflow     │
  │    ├── If true: apply action                  │
  │    │   ├── confirm / reject / modify / flag   │
  │    │   ├── record RuleApplication             │
  │    │   ├── check application limit            │
  │    │   └── check all-rejected → flag          │
  │    ├── If false: skip (continue to next)      │
  │    └── Check fixpoint (no change → break)     │
  └───────────────────────────────────────────────┘
                   │
                   ▼
  ┌───────────────────────────────────────────────┐
  │  3. Assemble Output                            │
  │     ├── Build AnnotatedGIR (workflow + flags)  │
  │     ├── Collect all rule applications          │
  │     ├── Generate grammatical flags             │
  │     └── Return IR-7                            │
  └───────────────────────────────────────────────┘
                   │
                   ▼
  IR-7 (AnnotatedGIR) → MOD-08 (KnowledgeGraphResolver)
```

### 2.3 Ambiguity Forest Transformation

The primary function of MOD-07 is to transform the ambiguity forest:

```diff
  Before MOD-07 (IR-6):
  ┌──────────────────────────────────────────────┐
  │  Ambiguity Forest: Full                      │
  │                                              │
  │  Token 0: 2 morphological analyses           │
  │  Token 1: 3 morphological analyses           │
  │  Token 2: 1 morphological analysis           │
  │  Syntax: 2 parse trees                       │
  │  ─────────────────────                       │
  │  Total paths: 2 × 3 × 1 × 2 = 12 paths      │
  └──────────────────────────────────────────────┘
                    │
                    ▼
  After MOD-07 (IR-7):
  ┌──────────────────────────────────────────────┐
  │  Annotated GIR: Pruned by Rules              │
  │                                              │
  │  Token 0: confirmed analysis A, rejected B   │
  │  Token 1: confirmed analysis C, rejected D,E │
  │  Token 2: confirmed analysis F               │
  │  Syntax: confirmed tree 1, rejected tree 2   │
  │  ─────────────────────                       │
  │  Total paths: 1 × 1 × 1 × 1 = 1 path        │
  │                                              │
  │  Rule applications: 8                        │
  │  Flags: 2 (1 error, 1 info)                 │
  └──────────────────────────────────────────────┘
```

---

## 3. Internal Component Model

### 3.1 Core Data Structures

#### 3.1.1 CompiledRule

```yaml
CompiledRule:
  description: >
    A grammatical rule that has been parsed, validated, and compiled
    into an executable form. This is the runtime representation of
    an RFC-0001 .agosrule file's rule definition.

  fields:
    # Metadata (from RFC-0001 §8)
    id: string                            # Unique rule ID (e.g., "basra-0103")
    school: string                        # Grammar school
    version: string                       # Semver rule version
    priority: integer                     # Application priority (higher = earlier)
    description: string                   # Human-readable description
    enabled: boolean                      # Whether this rule is active

    # Compiled condition
    condition_ast: ASTNode                # Compiled condition expression tree
    condition_source: string              # Original condition DSL text

    # Compiled actions
    actions: CompiledAction[]             # Ordered list of actions to execute

    # Optimization data
    required_features: string[]           # Features referenced by this rule
    token_role_filters: string[] | null   # If set, rule only applies to tokens with these roles
    sentence_type_filter: string | null   # If set, rule only applies to this sentence type

CompiledAction:
  type: "confirm" | "reject" | "modify" | "flag" | "resolve"
  target: ExpressionPath | null           # Target of the action
  value: Expression | null                # Value (for modify)
  reason: string | null                   # For reject actions
  severity: string | null                 # For flag actions (error/warning/info)
  code: string | null                     # For flag actions
```

#### 3.1.2 RuleWorkflow

```yaml
RuleWorkflow:
  description: >
    Mutable state maintained during rule application.
    A clone of the GIR that rules modify.

  fields:
    # The working GIR (mutable copy)
    gir: AnnotatedGIR                    # Clone of input IR-6

    # Rule application state
    rule_applications: RuleApplication[]
    flags: GrammaticalFlag[]
    applied_count: integer               # Counter for limit checking
    last_modified_at_step: integer       # Step tracking for fixpoint detection

    # Ambiguity state
    confirmed_analyses: Map<string, boolean>     # analysis_id → confirmed
    rejected_analyses: Map<string, boolean>      # analysis_id → rejected
    ambiguity_paths: AmbiguityPath[]

    # Index structures for fast rule evaluation
    token_by_role: Map<string, Token>    # role → token (e.g., "fi'l" → token)
    token_by_index: Map<integer, Token>  # index → token
    sentence_type_cache: string          # Cached sentence type
```

#### 3.1.3 RuleApplicationRecord

```yaml
RuleApplicationRecord:
  description: >
    A complete record of one rule application, stored in the evidence trail.

  fields:
    rule_id: string                      # Rule that fired
    rule_name: string                    # Human-readable rule name
    school: string                       # School
    version: string                      # Rule version

    applies_to:
      token_indices: integer[]
      constituent_path: string[]

    condition: string                    # Condition that matched
    action: string                       # Action that was executed

    result:
      confirmed: string[]                # Analysis IDs confirmed
      rejected: string[]                 # Analysis IDs rejected
      modified: ModifyRecord[]           # Feature modifications
      flag: GrammaticalFlag | null       # Flag raised (if any)

    evidence: EvidenceEntry              # Standard AGOS evidence format

ModifyRecord:
  feature: string                        # e.g., "case"
  from: string                           # e.g., "nominative"
  to: string                             # e.g., "accusative"
  token_index: integer                   # Token affected
```

#### 3.1.4 AmbiguityPath

```yaml
AmbiguityPath:
  description: >
    One path through the ambiguity forest. Each path is a unique
    combination of per-token morphological analyses × syntactic parse trees.

  fields:
    path_id: string                      # Unique path identifier
    analysis_selections:
      [token_index: integer]: string     # token_index → analysis_id
    tree_id: string                      # Selected parse tree
    confidence: float                    # Aggregate confidence
    source: string                       # Grammar school
    is_rejected: boolean                 # Whether rules have rejected this path
    rejection_reason: string | null      # Why it was rejected
    is_confirmed: boolean                # Whether rules have confirmed this path
```

### 3.2 Configuration Model

```yaml
RuleEngineConfig:
  extends: StageConfig

  fields:
    school: string                       # e.g., "basra", "kufa"
    rule_set_version: string | null      # default: latest
    max_rule_applications: integer       # default: 1000
    strict_mode: boolean                 # default: false
    enable_fixpoint_detection: boolean   # default: true
    max_fixpoint_iterations: integer     # default: 5
    enable_anaphora_resolution: boolean  # default: true
    anaphora_window: integer             # default: 5 (tokens before pronoun)
    rule_set_path: string | null         # Custom rule set path

InternalRuleEngineConfig:
  debug_mode: boolean                    # default: false (extra logging)
  profile_mode: boolean                  # default: false (perf counters)
  skip_validation: boolean               # default: false (skip output validation)
```

### 3.3 Rule Set Catalog

Each grammar school has a versioned rule set. The engine supports:

```yaml
RuleSetCatalog:
  basra:
    versions: ["1.0.0", "1.1.0", "2.0.0"]
    latest: "2.0.0"
    rule_count: ~850
    file: "rules/basra/rule-set.agosrule"

  kufa:
    versions: ["1.0.0", "1.1.0"]
    latest: "1.1.0"
    rule_count: ~720
    file: "rules/kufa/rule-set.agosrule"

  baghdad:
    versions: ["1.0.0"]
    latest: "1.0.0"
    rule_count: ~780
    file: "rules/baghdad/rule-set.agosrule"

  andalus:
    versions: ["1.0.0"]
    latest: "1.0.0"
    rule_count: ~680
    file: "rules/andalus/rule-set.agosrule"

  modern:
    versions: ["1.0.0"]
    latest: "1.0.0"
    rule_count: ~550
    file: "rules/modern/rule-set.agosrule"
```

---

## 4. Rule Loading & Compilation

### 4.1 Rule Set Resolution

```pseudocode
Algorithm: resolve_rule_set
Input: school (string), version (string | null)
Output: CompiledRule[]

Step 1: Determine Rule Set Path
  1.1  If custom rule_set_path provided:
       - Use the provided path.
       - Skip standard resolution.
  1.2  Else:
       - Resolve school to standard path: "rules/{school}/rule-set.agosrule"
       - If version is null → use latest version from catalog.

Step 2: Load Rule Set File
  2.1  Check file exists → if not: RULE_SET_NOT_FOUND
  2.2  Check file version compatibility → if incompatible: RULE_VERSION_MISMATCH
  2.3  Read file as UTF-8 text.

Step 3: Parse Imports
  3.1  Extract all import statements from the file.
  3.2  Resolve each import path:
       - Relative imports: resolved relative to importing file's directory.
       - Standard library imports: resolved from AGOS_DSL_PATH.
  3.3  Check for circular imports → if circular: DSL_CIRCULAR_IMPORT
  3.4  Recursively load imported files.

Step 4: Parse Rules
  4.1  For each rule block in all loaded files:
  4.1.1  Parse metadata block → extract id, school, version, priority, etc.
  4.1.2  Parse condition block → build AST.
  4.1.3  Parse action block → build CompiledAction[].
  4.1.4  If rule is disabled (enabled: false) → skip.
  4.1.5  Validate: check for duplicate rule IDs → if duplicate: DSL_DUPLICATE_RULE_ID
  4.1.6  Create CompiledRule object.

Step 5: Sort Rules
  5.1  Sort by priority descending (higher priority first).
  5.2  Ties broken by rule ID alphabetically.
  5.3  Result: ordered list of CompiledRule.

Step 6: Cache
  6.1  Cache compiled rule set by (school, version) key.
  6.2  Subsequent requests with same key → return cached version.
  6.3  Cache invalidation: when rule set files change (watch file mtime).

Step 7: Return CompiledRule[]
```

### 4.2 DSL Compilation Pipeline

```pseudocode
Algorithm: compile_rule
Input: dsl_source (string), rule_metadata (Metadata)
Output: CompiledRule

Step 1: Lexical Analysis
  1.1  Tokenize DSL source into tokens:
       - Keywords: rule, metadata, condition, action, import, etc.
       - Identifiers: role names, feature names, variable names
       - Literals: strings, integers, booleans
       - Operators: ==, !=, and, or, not, in, matches
       - Punctuation: { } ( ) . , :
  1.2  Handle comments: strip //, /* */, (* *) comments.
  1.3  If tokenization fails → DSL_PARSE_ERROR.

Step 2: Syntactic Analysis (Parse AST)
  2.1  Parse top-level rule structure: rule "id" { metadata { } condition { } action { } }
  2.2  Parse metadata block into key-value pairs.
  2.3  Parse condition expression into AST:
       - Logical operators: and, or, not → BinaryOp / UnaryOp nodes
       - Comparisons: ==, !=, <, >, matches, in → Comparison nodes
       - Quantifications: forall, exists → Quantification nodes
       - Path expressions: sentence.tokens[0].features.gender → Path nodes
       - Shorthands: fi'l, fa'il, mubtada' → resolve to full paths
  2.4  Parse action block into CompiledAction[]:
       - confirm, reject, modify, flag, resolve → Action nodes
       - Conditional: if (expr) { actions } else { actions }
       - Quantified: forall (var in collection) { actions }
  2.5  If parsing fails → DSL_PARSE_ERROR with position.

Step 3: Semantic Analysis (Type Checking)
  3.1  Verify all referenced variables exist:
       - Check built-in variables: sentence, token, global
       - Check role shorthands: fi'l, fa'il, mubtada', khabar, etc.
  3.2  Verify all referenced feature names match KB-0007 taxonomy:
       - e.g., gender, number, person, tense, mood, voice, case, state
  3.3  Verify all referenced syntactic roles are valid:
       - e.g., fi'l, fa'il, mubtada', khabar, maf'ul_bi-hi
  3.4  Type check comparisons:
       - String vs. string: OK
       - Integer vs. integer: OK
       - Feature comparison: feature value is always string
  3.5  If type checking fails → DSL_TYPE_MISMATCH.

Step 4: Optimization
  4.1  Extract required_features from condition:
       - Scan AST for all feature paths (e.g., token.features.gender)
       - Store as CompiledRule.required_features
  4.2  Extract token_role_filters:
       - If condition checks specific role (e.g., token.role == "fi'l")
       → set token_role_filters
  4.3  Extract sentence_type_filter:
       - If condition checks sentence.type → set sentence_type_filter
  4.4  Constant-fold comparisons where possible.

Step 5: Build CompiledRule
  5.1  Create CompiledRule with:
       - Metadata fields from parsed metadata
       - Compiled condition AST
       - Compiled actions
       - Optimization hints (required_features, filters)

Step 6: Return CompiledRule
```

### 4.3 Rule Caching

```pseudocode
Algorithm: manage_rule_cache
Input: school (string), version (string)
Output: CompiledRule[] (from cache or compiler)

Internal State:
  rule_cache: Map<(school, version), CompiledRule[]>
  cache_mtime: Map<(school, version), timestamp>

Step 1: Check Cache
  1.1  key = (school, version)
  1.2  If key in rule_cache:
  1.2.1  Check if rule set file has been modified since cached:
         - Compare file mtime with cache_mtime[key]
         - If file is newer → invalidate cache entry
         - If file same age → return cached CompiledRule[]
  1.3  If key not in cache (or invalidated):
  1.3.1  Load and compile rule set → CompiledRule[]
  1.3.2  rule_cache[key] = CompiledRule[]
  1.3.3  cache_mtime[key] = current file mtime
  1.3.4  Return CompiledRule[]

Step 2: Handle Multiple Schools
  2.1  If pipeline is configured for multiple schools:
  2.1.1  Load each school's rule set independently
  2.1.2  Each school's rules maintain separate cache entries
  2.1.3  School switching is O(1) (cache hit) after initial load
```

---

## 5. Rule Execution Engine

### 5.1 Main Execution Loop

```pseudocode
Algorithm: apply_rules
Input: input (RuleEngineInput)
Output: AnnotatedGIR or RuleEngineError

Step 1: Validate Input
  1.1  Verify input.gir is non-null and structurally valid.
  1.2  Verify input.config.school is a recognized school.
  1.3  Verify input.config.rule_set_version is compatible.
  1.4  If validation fails → return appropriate error.

Step 2: Load Rule Set
  2.1  rules = resolve_rule_set(input.config.school, input.config.rule_set_version)
  2.2  If rules empty → return RULE_SET_NOT_FOUND.

Step 3: Initialize Workflow
  3.1  workflow = RuleWorkflow {
           gir = deep_copy(input.gir),
           rule_applications = [],
           flags = [],
           applied_count = 0,
           confirmed_analyses = {},
           rejected_analyses = {},
           last_modified_at_step = 0
       }
  3.2  Build index structures:
       - workflow.token_by_role: scan tokens, map role → token
       - workflow.token_by_index: map index → token
  3.3  Build initial ambiguity paths:
       - For each combination of (token_analyses × trees):
           Create AmbiguityPath with confidence

Step 4: Execute Rules (Main Loop)
  4.1  iteration = 0
  4.2  While applied_count < max_rule_applications AND iteration < max_fixpoint_iterations:
  4.2.1  iteration += 1
  4.2.2  state_changed = false
  4.2.3  For each rule r in rules (priority order):
  4.2.3.1  If not r.enabled → skip
  4.2.4  state_changed |= execute_single_rule(workflow, r)
  4.2.5  If not state_changed → fixpoint reached, break
  4.2.6  Check for all-rejected state:
         - If all ambiguity paths are rejected → flag UNRESOLVABLE_AMBIGUITY
         - Continue (partial analysis still valid)

Step 5: Post-Processing
  5.1  Sort remaining ambiguity paths by:
       - Primary: number of confirming rules (more = higher)
       - Secondary: rule priority weighted (higher priority rules' confirmations weigh more)
       - Tertiary: aggregate morphological confidence
       - Quaternary: school preference
  5.2  Generate summary flags:
       - If ambiguity remains → flag AMBIGUITY_REMAINING (info)
       - If known violations detected → appropriate error/warning flags

Step 6: Build Output
  6.1  annotated_gir = assemble_output(workflow)
  6.2  If debug_mode: log rule execution summary
  6.3  If profile_mode: log performance counters

Step 7: Return
  7.1  Return annotated_gir
```

### 5.2 Single Rule Execution

```pseudocode
Algorithm: execute_single_rule
Input: workflow (RuleWorkflow), rule (CompiledRule)
Output: boolean (whether state changed)

Step 1: Check Rule Applicability (Fast Filters)
  1.1  If rule.token_role_filters is set:
       - Check if any token has matching role
       - If none match → skip (return false, no state change)
  1.2  If rule.sentence_type_filter is set:
       - Check if sentence type matches
       - If not match → skip

Step 2: Evaluate Condition
  2.1  condition_result = evaluate_expression(workflow, rule.condition_ast)
  2.2  If condition_result is Error:
       - Log RULE_CONDITION_EVAL_FAILED
       - Return false (skip rule, no state change)

Step 3: If Condition is True → Execute Actions
  3.1  If condition_result == true:
  3.1.1  For each action in rule.actions:
  3.1.1.1  Execute action on workflow:
           - confirm(target) → mark analyses as confirmed
           - reject(target, reason) → mark analyses as rejected
           - modify(target, value) → change feature value
           - flag(severity, code, targets) → add grammatical flag
           - resolve(pronoun_index, antecedent) → link anaphora
  3.1.1.2  If state changed → record state_changed = true
  3.1.2  Record RuleApplication in workflow.rule_applications
  3.1.3  workflow.applied_count += 1
  3.2  Else (condition false):
  3.2.1  Return false (no state change)

Step 4: Check Application Limit
  4.1  If workflow.applied_count >= max_rule_applications:
  4.1.1  Log RULE_APPLICATION_LIMIT warning
  4.1.2  Return state_changed (execution will stop at main loop)

Step 5: Return state_changed
```

### 5.3 Condition Evaluation

```pseudocode
Algorithm: evaluate_expression
Input: workflow (RuleWorkflow), node (ASTNode)
Output: boolean | Error

Dispatch on node.type:

  case "boolean_literal":
    return node.value (true or false)

  case "comparison":
    left = evaluate_expression(workflow, node.left)
    right = evaluate_expression(workflow, node.right)

    switch node.operator:
      case "==": return left == right
      case "!=": return left != right
      case "<":  return left < right
      case ">":  return left > right
      case "matches":
        return regex_match(left, right)  // RE2-compatible
      case "in":
        return left in right

  case "logical_and":
    left = evaluate_expression(workflow, node.left)
    if not left: return false  // short-circuit
    return evaluate_expression(workflow, node.right)

  case "logical_or":
    left = evaluate_expression(workflow, node.left)
    if left: return true  // short-circuit
    return evaluate_expression(workflow, node.right)

  case "logical_not":
    return not evaluate_expression(workflow, node.operand)

  case "path_expression":
    return resolve_path(workflow, node.path)

  case "quantified_forall":
    collection = evaluate_expression(workflow, node.collection)
    for each element in collection:
      workflow.scope.set(node.variable, element)
      result = evaluate_expression(workflow, node.body)
      if not result: return false
    return true

  case "quantified_exists":
    collection = evaluate_expression(workflow, node.collection)
    for each element in collection:
      workflow.scope.set(node.variable, element)
      result = evaluate_expression(workflow, node.body)
      if result: return true
    return false

  case "function_call":
    return evaluate_builtin(workflow, node.function_name, node.args)

  default:
    return Error("Unknown AST node type")
```

### 5.4 Path Resolution

```pseudocode
Algorithm: resolve_path
Input: workflow (RuleWorkflow), path (string[])
Output: any (value at path)

Step 1: Start from root
  1.1  path_segments = split path by "."
  1.2  current = workflow

Step 2: Resolve Each Segment
  2.1  For segment in path_segments:
  2.1.1  If segment is a role shorthand:
         - fi'l → workflow.token_by_role["fi'l"]
         - fa'il → workflow.token_by_role["fa'il"]
         - mubtada' → workflow.token_by_role["mubtada"]
         - etc.
  2.1.2  If segment is a built-in variable:
         - sentence → workflow.gir
         - token → workflow.current_token
         - global → workflow.gir.metadata
  2.1.3  If segment is an index access (e.g., tokens[0]):
         - Parse index from brackets
         - Access collection at index
  2.1.4  If segment is a feature access (e.g., features.gender):
         - Access feature map via .features
         - Return feature value
  2.1.5  If segment is a direct field access (e.g., role, type, text):
         - Access object field directly
  2.1.6  If current is null → return null

Step 3: Handle Shorthand Resolution
  3.1  Role shorthands resolve to FIRST token with that role:
       - For fi'l: find first token where token.role == "fi'l"
       - This is deterministic (tokens are ordered)
  3.2  If no token has the role → return null (condition will evaluate to false)

Step 4: Return resolved value
```

### 5.5 Action Execution

```pseudocode
Algorithm: execute_action
Input: workflow (RuleWorkflow), action (CompiledAction)
Output: boolean (state changed)

switch action.type:

  case "confirm":
    target = resolve_action_target(workflow, action.target)
    if target is a specific analysis_id:
      workflow.confirmed_analyses[target] = true
      update_path_confidence(target, +0.2)
      return true
    else:
      // Confirm current path
      for each path in workflow.ambiguity_paths:
        if not path.is_rejected:
          path.is_confirmed = true
          path.confidence += 0.1
      return true

  case "reject":
    target = resolve_action_target(workflow, action.target)
    if target is a specific analysis_id:
      workflow.rejected_analyses[target] = true
      mark_paths_with_analysis_rejected(target, action.reason)
      return true
    else:
      // Reject all paths matching current condition context
      for each relevant path:
        path.is_rejected = true
        path.rejection_reason = action.reason
      return true

  case "modify":
    target_path = action.target  // e.g., token.features.case
    new_value = evaluate_expression(workflow, action.value)

    resolved = resolve_path(workflow, target_path)
    old_value = resolved

    if old_value != new_value:
      modify_in_workflow(workflow, target_path, new_value)
      record_modification(target_path, old_value, new_value)
      return true
    return false

  case "flag":
    severity = action.severity    // "error" | "warning" | "info"
    code = action.code             // e.g., "SUBJECT_VERB_AGREEMENT"
    targets = resolve_targets(workflow, action.targets)

    flag = GrammaticalFlag {
      flag_type: severity,
      code: code,
      message: format_flag_message(code, targets),
      token_indices: extract_indices(targets),
      rule_id: action.rule_id
    }

    workflow.flags.append(flag)
    return true

  case "resolve":
    pronoun_index = evaluate_expression(workflow, action.pronoun_index)
    antecedent_index = evaluate_expression(workflow, action.antecedent_index)

    // Record resolution link
    workflow.gir.anaphora_resolutions.append({
      pronoun_token_index: pronoun_index,
      antecedent_token_index: antecedent_index
    })
    return true
```

---

## 6. Ambiguity Pruning & Management

### 6.1 Ambiguity Pruning Algorithm

```pseudocode
Algorithm: prune_ambiguity
Input: workflow (RuleWorkflow)
Output: void (modifies workflow)

Step 1: Process Rejections
  1.1  For each rejected_analysis_id in workflow.rejected_analyses:
  1.1.1  For each ambiguity_path in workflow.ambiguity_paths:
         - If ambiguity_path includes rejected_analysis_id:
             path.is_rejected = true
             path.rejection_reason = workflow.rejected_analyses[analysis_id]
  1.1.2  Remove rejected analysis from token's alternatives list

Step 2: Process Confirmations
  2.1  For each confirmed_analysis_id in workflow.confirmed_analyses:
  2.1.1  For each ambiguity_path in workflow.ambiguity_paths:
         - If ambiguity_path includes confirmed_analysis_id:
             path.is_confirmed = true
             path.confidence += 0.2

Step 3: Remove Fully Rejected Paths
  3.1  Remove all paths where is_rejected == true from active consideration
  3.2  Keep them in the evidence trail (they document why the path was rejected)

Step 4: Check for Complete Rejection
  4.1  If workflow.ambiguity_paths.length == 0:
  4.1.1  All analyses have been rejected
  4.1.2  Flag: UNRESOLVABLE_AMBIGUITY (error)
  4.1.3  Return last remaining (non-rejected) path with confidence = 0.0
         as fallback

Step 5: Sort Remaining Paths
  5.1  Sort by:
       - is_confirmed (confirmed paths first)
       - confidence (descending)
       - number of confirming rules (descending)
```

### 6.2 Ambiguity Sources and Rule Effects

| Source of Ambiguity | Example | Rule Effect | Output |
|---------------------|---------|-------------|--------|
| **Morphological homograph** | عين = eye/spring/to appoint | Confirm one, reject others | Single analysis |
| **Unvocalized text** | كتب = kataba/kutiba/kutub | Reject by syntactic context | Single POS + features |
| **Dual POS** | ضرب = verb or noun | Confirm by positional role | Correct POS |
| **Syntax attachment** | PP attachment ambiguity | Reject by school-specific rules | Correct tree |
| **Case assignment** | Unvocalized noun case | Modify by government rule | Correct case |
| **Anaphora** | Pronoun antecedent | Resolve by agreement | Antecedent link |
| **Mood** | Present tense verb | Modify by governing particle | Correct mood |

### 6.3 Ambiguity Metrics

```yaml
AmbiguityMetrics:
  typical_input: "5-word sentence, moderate ambiguity"
  before_rules:
    total_paths: 12-64
    per_token_alternatives: 1-4
    parse_trees: 1-4
  after_rules:
    total_paths: 1-4
    paths_with_full_confidence: 1-2
    flags_raised: 0-3
    rules_applied: 5-50
```

---

## 7. Anaphora Resolution

### 7.1 Anaphora Resolution Algorithm

```pseudocode
Algorithm: resolve_anaphora
Input: workflow (RuleWorkflow), pronoun_token (Token)
Output: integer | null (antecedent token index)

Step 1: Identify Pronoun Type
  1.1  features = pronoun_token.features
  1.2  If features.pronoun_type not in ["personal_attached", "personal_detached"]:
       - Return null (only personal pronouns need resolution)

Step 2: Search for Antecedent
  2.1  Search window: tokens BEFORE the pronoun (up to anaphora_window tokens)
  2.2  For each candidate_token in search window:
  2.2.1  If candidate_token.POS not in ["noun", "proper_noun", "pronoun"]:
         - Skip (only nouns/pro nouns can be antecedents)
  2.2.2  Score candidate:
         - +3 if person, number, and gender ALL match
         - +2 if person and number match (gender differs)
         - +1 if only person matches
         - -1 if person differs (strong demotion)
         - -2 if candidate is a pronoun itself (prefer full noun antecedents)

Step 3: Select Best Antecedent
  3.1  If any candidate scores +3:
       - Select the closest +3 candidate (highest confidence)
  3.2  Else if any candidate scores +2:
       - Select the closest +2 candidate
  3.3  Else if any candidate scores +1:
       - Select the closest +1 candidate (low confidence)
  3.4  Else:
       - Return null (no antecedent found)

Step 4: Record Resolution
  4.1  Record resolve action in evidence trail
  4.2  Link pronoun to antecedent in AnnotatedGIR
  4.3  Return antecedent token index

Scoring Example:
  Sentence: "رأيت زيدًا وهو يقرأ" (I saw Zayd while he was reading)
  Pronoun: "هو" (he) - 3ms
  Candidates:
    "زيدًا" (Zayd) - 3ms → +3 match (perfect)
    "رأيت" (I saw) - 1s → -1 no match
  Result: "زيدًا" at index 1 is the antecedent
```

### 7.2 Special Cases

```yaml
AnaphoraSpecialCases:
  impersonal_pronouns:
    description: "Pronouns without clear antecedent"
    example: "يُقَالُ إِنَّ... (it is said that...)"
    behavior: >
      Leave unresolved. Mark as IMPERSONAL_PRONOUN flag.
      The Explanation Engine will explain as impersonal construction.

  generic_antecedent:
    description: "Pronoun referring to a generic statement"
    example: "مَنْ يَجْتَهِدْ يَنْجَحْ (whoever strives succeeds)"
    behavior: >
      The conditional pronoun مَنْ serves as both conditional particle and
      antecedent. Resolve to the conditional clause head.

  dual_ambiguity:
    description: "Two equally plausible antecedents"
    example: "ضَرَبَ زَيْدٌ عَمْرًا وَهُوَ... (Zayd hit Amr while he...)"
    behavior: >
      Both زَيْد and عَمْر are plausible antecedents for هُوَ.
      Preserve both resolutions as ambiguity. Flag ANTHROPIC_AMBIGUITY.
```

---

## 8. School-Specific Rule Sets

### 8.1 Rule Categorization by School

Each school's rule set covers the following categories:

```yaml
RuleCategories:
  agreement:
    description: "Subject-verb, noun-adjective, and other agreement rules"
    rule_count_range: [80, 150]
    examples:
      - "basra-0103: Subject-Verb Person Agreement"
      - "basra-0104: Subject-Verb Number Agreement (VSO exception)"
      - "kufa-0104: Verb before plural non-human subject"

  case_assignment:
    description: "Rules that assign or modify case features"
    rule_count_range: [60, 120]
    examples:
      - "basra-0201: Preposition governs genitive"
      - "basra-0202: Subject takes nominative"
      - "basra-0203: Direct object takes accusative"

  mood_government:
    description: "Rules that assign mood based on governing particles"
    rule_count_range: [30, 60]
    examples:
      - "basra-0301: أَنْ governs subjunctive"
      - "basra-0302: لَمْ governs jussive"
      - "basra-0303: لَنْ governs subjunctive"

  special_verbs:
    description: "Rules for kana and sisters, inna and sisters, etc."
    rule_count_range: [40, 80]
    examples:
      - "basra-0305: Kana and her sisters case reassignment"
      - "basra-0310: Inna and her sisters case reassignment"

  construction:
    description: "Rules for idafa, wasf, tawkid, badal, etc."
    rule_count_range: [50, 100]
    examples:
      - "basra-0401: Idafa case and state constraints"
      - "basra-0402: Wasf four-way agreement"

  conditional:
    description: "Rules for conditional sentence structure"
    rule_count_range: [20, 40]
    examples:
      - "basra-0501: إِنْ governs jussive on both verbs"
      - "basra-0502: لَوْ takes perfect tense (no jussive)"

  exception:
    description: "Rules for istithna and other exceptions"
    rule_count_range: [15, 30]
    examples:
      - "basra-0601: إِلَّا accusative case"
      - "basra-0602: Negative إِلَّا follows negated case"

  anaphora:
    description: "Rules for pronoun-antecedent resolution"
    rule_count_range: [20, 40]
    examples:
      - "basra-0701: Pronoun agreement with antecedent"
      - "basra-0702: Implicit subject resolution"

  validation:
    description: "Rules that detect ungrammatical constructions"
    rule_count_range: [40, 80]
    examples:
      - "basra-0801: Missing subject on active verb"
      - "basra-0802: Preposition without object"
```

### 8.2 School-Specific Rule Differences

The following table highlights key differences between schools:

| Rule Category | Basra | Kufa | Baghdad | Andalus | Modern |
|---------------|-------|------|---------|---------|--------|
| **إِنْ mood** | Jussive on both verbs | Jussive on both verbs | Jussive on both verbs | Jussive on both verbs | Perfect tense allowed |
| **إِنَّ vs. أَنَّ** | Strict distinction | Some overlap allowed | Moderate | Strict | Relaxed |
| **لَوْ usage** | Hypothetical only | Broader conditional | Moderate | Typical | Flexible |
| **VSO number** | Verb always singular | Verb can be plural | Verb always singular | Verb always singular | Verb can be plural |
| **SVO number** | Verb agrees fully | Verb agrees fully | Verb agrees fully | Verb agrees fully | Verb agrees fully |
| **مبتدأ definiteness** | Must be definite | Can be indefinite | Must be definite | Must be definite | Can be indefinite |
| **خبر before مبتدأ** | Not allowed | Allowed | Allowed | Not allowed | Allowed |
| **إِلَّا case** | Always accusative | Flexible | Mostly accusative | Always accusative | Flexible |
| **Idafa strictness** | Strict | Relaxed | Moderate | Strict | Relaxed |
| **Non-human plural** | Fem sg verb | Fem sg verb | Fem sg verb | Fem sg verb | Fem sg verb preferred |

### 8.3 Rule Priority Ranges

```yaml
PriorityRanges:
  description: "Rules are assigned priorities in these ranges"

  high_priority: [70, 100]
  categories:
    - "fundamental_agreement"    # Basic subject-verb agreement
    - "essential_case_rules"     # Preposition genitive, subject nominative
    - "mood_government"          # Particle mood assignment
    - "construction_foundation"  # Idafa, wasf fundamentals

  medium_priority: [30, 69]
  categories:
    - "secondary_agreement"      # Number agreement with non-human plurals
    - "special_constructions"    # Kana, inna, conditional
    - "anaphora_resolution"      # Pronoun-antecedent linking
    - "ellipsis_detection"       # Implicit subject/predicate

  low_priority: [1, 29]
  categories:
    - "validation"               # Ungrammatical construction detection
    - "educational_flags"        # Informational notices
    - "school_specific_nuance"   # Minor school-specific variations
```

---

## 9. Conflict Detection & Resolution

### 9.1 Conflict Types

```yaml
ConflictTypes:
  direct_contradiction:
    description: "Two rules modify the same feature to different values"
    example:
      - Rule A: modify(maf'ul_bi-hi.case, "accusative")
      - Rule B: modify(maf'ul_bi-hi.case, "genitive")
    resolution: >
      Higher priority rule wins. If same priority: strict_mode = fail,
      normal mode = preserve both alternatives as ambiguity.

  confirm_vs_reject:
    description: "One rule confirms an analysis, another rejects it"
    example:
      - Rule A: confirm(analysis_1)
      - Rule B: reject("reason", analysis_1)
    resolution: >
      Higher priority rule wins. If same priority: REJECT takes precedence
      (safer to remove an incorrect analysis than to keep it).

  scope_overlap:
    description: "Two rules affect overlapping token sets"
    example:
      - Rule A: modifies all tokens in sentence
      - Rule B: modifies specific token
    resolution: >
      Priority-based. Specific rules should have higher priority than
      general rules (convention, not enforced).

  school_overlap:
    description: "Rules from different schools when multiple schools configured"
    example:
      - Basra rule: mubtada' must be definite
      - Kufa rule: mubtada' can be indefinite
    resolution: >
      Primary school's rules take precedence. Other schools' rules are
      applied with lower weight.
```

### 9.2 Conflict Detection Algorithm

```pseudocode
Algorithm: detect_conflicts
Input: workflow (RuleWorkflow), existing_modifications (ModifyRecord[]),
       new_action (CompiledAction)
Output: Conflict | null

Step 1: Check for Direct Contradiction
  1.1  If new_action.type == "modify":
  1.1.1  For each existing_modification:
  1.1.1.1  If existing_modification.feature == new_action.target.feature
            AND existing_modification.token_index == new_action.target.token_index
            AND existing_modification.to != new_action.value:
            → Direct contradiction detected
  1.1.2  Return Conflict(direct_contradiction, existing, new)

Step 2: Check for Confirm vs. Reject
  2.1  If new_action.type in ["confirm", "reject"]:
  2.1.1  For each existing RuleApplication:
  2.1.1.1  If existing.rejected contains new_action.target
            AND new_action.type == "confirm":
            → Confirm vs. reject conflict
  2.1.1.2  If existing.confirmed contains new_action.target
            AND new_action.type == "reject":
            → Confirm vs. reject conflict

Step 3: Return null (no conflict)
```

### 9.3 Conflict Resolution Algorithm

```pseudocode
Algorithm: resolve_conflict
Input: conflict (Conflict), strict_mode (boolean)
Output: Resolution

Step 1: Check Priorities
  1.1  If conflict.rule_a.priority > conflict.rule_b.priority:
       → Rule A wins, Rule B is overridden
  1.2  If conflict.rule_b.priority > conflict.rule_a.priority:
       → Rule B wins, Rule A is overridden
  1.3  If equal priority:

Step 2: School Precedence (Same Priority)
  2.1  If conflict.rule_a.school == primary_school
        AND conflict.rule_b.school != primary_school:
       → Rule A wins (primary school rules take precedence)
  2.2  If both rules from same school:

Step 3: Strict Mode Behavior
  3.1  If strict_mode == true:
  3.1.1  Return RULE_CONFLICT error with details
  3.1.2  Pipeline stops (fatal error)
  3.2  If strict_mode == false:
  3.2.1  Preserve both alternatives as ambiguity
  3.2.2  Record conflict in evidence trail
  3.2.3  Return Resolution(preserve_ambiguity)
```

---

## 10. Evidence & Flag Generation

### 10.1 Rule Application Evidence

Each rule application generates an `EvidenceEntry` and a `RuleApplicationRecord`:

```pseudocode
Algorithm: record_rule_application
Input: workflow (RuleWorkflow), rule (CompiledRule), result (ActionResult)
Output: void

1.  evidence_entry = EvidenceEntry {
        id: generate_uuid(),
        timestamp: now(),
        stage: "MOD-07",
        stage_iteration: current_iteration,
        category: "rule_application",
        rule_or_algorithm: rule.id,
        version: rule.version,
        input: {
            description: rule.condition_source,
            state_hash: hash(workflow_state)
        },
        output: {
            description: format_action_description(result),
            delta: format_action_delta(result)
        },
        confidence: calculate_confidence(result),
        token_indices: extract_token_indices(result)
    }

2.  rule_application = RuleApplication {
        rule_id: rule.id,
        rule_name: rule.description,
        school: rule.school,
        version: rule.version,
        applies_to: {
            token_indices: extract_token_indices(result),
            constituent_path: extract_constituent_path(result)
        },
        condition: rule.condition_source,
        action: format_action_source(result),
        result: {
            confirmed: result.confirmed_analyses,
            rejected: result.rejected_analyses,
            modified: result.modifications,
            flag: result.flag
        },
        evidence: evidence_entry
    }

3.  workflow.rule_applications.append(rule_application)
```

### 10.2 Grammatical Flag Types

```yaml
GrammaticalFlags:
  error_flags:
    description: "Definite grammatical violations"
    standard_codes:
      - "SUBJECT_VERB_PERSON_MISMATCH"
      - "SUBJECT_VERB_NUMBER_MISMATCH"
      - "SUBJECT_VERB_GENDER_MISMATCH"
      - "PREPOSITION_CASE_MISMATCH"
      - "INNA_CASE_MISMATCH"
      - "ADJECTIVE_AGREEMENT_MISMATCH"
      - "MOOD_GOVERNMENT_VIOLATION"
      - "UNRESOLVABLE_AMBIGUITY"

  warning_flags:
    description: "Unusual but potentially grammatical"
    standard_codes:
      - "MISSING_FA_IL"
      - "MISSING_MUBTADA"
      - "VERB_BEFORE_PLURAL_SUBJECT"
      - "NON_HUMAN_PLURAL_SINGULAR_VERB"
      - "ANTHROPIC_AMBIGUITY"
      - "AMBIGUITY_REMAINING"
      - "RULE_APPLICATION_LIMIT_REACHED"

  info_flags:
    description: "Informational or educational notices"
    standard_codes:
      - "KANA_CONSTRUCTION"
      - "INNA_CONSTRUCTION"
      - "CONDITIONAL_SENTENCE"
      - "ELLIPSIS_DETECTED"
      - "IMPLICIT_SUBJECT"
      - "BROKEN_PLURAL"
      - "DIPTOTE_NOUN"
      - "IMPERATIVE_FORM"
      - "RESOLVED_ANAPHORA"
```

### 10.3 Flag Generation Rules

```pseudocode
Algorithm: generate_flags
Input: workflow (RuleWorkflow)
Output: GrammaticalFlag[]

1.  flags = []

2.  Check for remaining ambiguity:
    2.1  If workflow.ambiguity_paths.length > 1:
         - flag: AMBIGUITY_REMAINING (info)

3.  Check for unresolved anaphora:
    3.1  For each pronoun with no antecedent:
         - flag: UNRESOLVED_ANAPHORA (info)

4.  Check for implicit constructions:
    4.1  If implicit subject detected:
         - flag: IMPLICIT_SUBJECT (info)
    4.2  If ellipsis detected:
         - flag: ELLIPSIS_DETECTED (info)

5.  Aggregate flags from rule applications:
    5.1  Collect all flags from each RuleApplicationRecord
    5.2  Deduplicate (same code + same token indices)

6.  Return flags
```

---

## 11. Performance & Optimization

### 11.1 Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Rule compilation (1,000 rules) | < 100 ms | At initialization, one-time cost |
| Rule loading (from cache) | < 10 ms | Cache hit path |
| Single rule condition evaluation | < 5 μs | Simple comparison |
| Single rule action execution | < 2 μs | Confirm, reject, or modify |
| Full pipeline (10 rules, 10-word sentence) | < 500 μs p50 | Target for production |
| Full pipeline (200 rules, 30-word sentence) | < 5 ms p99 | Worst case |
| Anaphora resolution (per pronoun) | < 5 μs | 10-candidate search |
| Conflict detection (per rule) | < 1 μs | Hash-based lookup |

### 11.2 Optimization Strategies

```yaml
Optimizations:
  - strategy: "rule_indexing"
    description: >
      Pre-index rules by required_features and token_role_filters
      so that most rules can be quickly filtered before condition
      evaluation.
    impact: >
      60-80% of rules can be skipped for any given sentence,
      reducing effective rule count from ~800 to ~100-200.

  - strategy: "condition_short_circuit"
    description: >
      Logical AND/OR expressions use short-circuit evaluation.
      Expensive operations (quantifications, regex matching) are
      placed at the end of AND chains.
    impact: >
      Up to 50% reduction in condition evaluation time for
      rules that don't match.

  - strategy: "path_result_caching"
    description: >
      Cache resolved path expressions. If two rules reference
      the same path (e.g., sentence.type), the result is cached
      after the first evaluation.
    impact: >
      30-50% reduction in repeated path resolution.

  - strategy: "batch_rejection"
    description: >
      When a rule rejects an analysis, all paths that depend on
      that analysis are rejected simultaneously. No need to
      re-evaluate per-path.
    impact: >
      O(n) rejection instead of O(n × p) where n = analyses,
      p = paths.

  - strategy: "parallel_rule_evaluation"
    description: >
      Rules with no overlapping required_features can be evaluated
      in parallel. Dependency graph ensures sequential evaluation
      only where needed.
    impact: >
      2-4× throughput improvement on multi-core systems.
    note: >
      Output must be serialized deterministically — parallel
      evaluation order does not affect final result.
```

### 11.3 Memory Budget

```yaml
MemoryBudget:
  static:
    - Compiled rules (1 school, ~1,000 rules): ~5-10 MB
    - Rule index structures: ~2-5 MB
    - DSL parser: ~1-2 MB
    Total static: ~10-20 MB

  dynamic (per request):
    - GIR copy (10-word sentence): ~20 KB
    - Ambiguity paths: ~10 KB
    - Rule applications log: ~5 KB
    - Working memory: ~5 KB
    Total per request: ~40 KB
```

---

## 12. Plugin Integration

### 12.1 Rule Set Plugin Type

MOD-07 supports `rule_set` plugins as defined in SPEC-0001-C7:

```yaml
RuleSetPlugin:
  plugin_type: "rule_set"
  injection_point: "MOD-07 RuleEngine"

  interface:
    load_rules(school: string, version: string) -> CompiledRule[]
    validate_rules(rules: CompiledRule[]) -> ValidationResult

  lifecycle:
    - Discover: PluginLoader discovers .agosrule files in plugin directory
    - Validate: PluginLoader validates manifest against schema
    - Load: RuleEngine loads rules via plugin's load_rules()
    - Apply: Rules are applied in normal priority order
    - Unload: Remove plugin's rules from cache

  sandboxing:
    - Rule evaluation is deterministic (no sandbox violations possible)
    - Rule files are read-only (no write permissions needed)
    - Plugin can only provide rules, not modify engine behavior
```

### 12.2 Multiple School Configuration

```pseudocode
Algorithm: apply_multiple_schools
Input: gir (GrammarIR), schools (string[])
Output: Map<school, AnnotatedGIR>

1.  results = {}

2.  For each school in schools:
    2.1  config = { school: school, strict_mode: false }
    2.2  result = apply_rules({ gir: gir, config: config })
    2.3  results[school] = result

3.  Return results

Note: The GIR is shared (not copied per school) because it is
school-agnostic. Only the rule engine runs per school.
```

---

## 13. Testing & Validation Strategy

### 13.1 Test Categories

```yaml
TestCategories:
  unit_tests:
    description: "Test individual components in isolation"
    coverage_target: "95%+ of code paths"

    test_modules:
      - "Rule parser (DSL → AST)"
      - "Rule compiler (AST → CompiledRule)"
      - "Condition evaluator"
      - "Action executor (confirm, reject, modify, flag, resolve)"
      - "Path resolver (role shorthands, feature access)"
      - "Anaphora resolution algorithm"
      - "Conflict detection"
      - "Conflict resolution (strict mode, normal mode)"

  integration_tests:
    description: "Test full MOD-07 pipeline"
    coverage_target: "All major rule categories"

    test_scenarios:
      - "Verbal sentence with full agreement → no flags"
      - "Verbal sentence with person mismatch → error flag"
      - "Nominal sentence with inna → case reassignment"
      - "Conditional sentence with إِنْ → jussive mood"
      - "Prepositional phrase → genitive case"
      - "Idafa construction → no tanwin on first noun"
      - "Adjective agreement → case/state/number/gender matching"
      - "Kana sentence → subject accusative, predicate nominative"
      - "Anaphora resolution → correct antecedent"
      - "Ellipsis detection → implicit subject marked"

  school_comparison_tests:
    description: "Verify school-specific differences"
    tests:
      - "Basra: مبدأ must be definite → reject indefinite mubtada"
      - "Kufa: مبدأ can be indefinite → accept indefinite mubtada"
      - "Basra: خبر before مبتدأ → reject"
      - "Kufa: خبر before مبتدأ → accept"

  ambiguity_tests:
    description: "Verify ambiguity preservation and pruning"
    tests:
      - "Unvocalized text: all morphological alternatives preserved"
      - "Homograph: all interpretations present as alternatives"
      - "Rule rejects one analysis → remaining alternatives preserved"
      - "All analyses rejected → flag UNRESOLVABLE_AMBIGUITY"
      - "Strict mode: conflict → error"
      - "Normal mode: conflict → ambiguity preserved"

  conformance_tests:
    description: "Verify against canonical Arabic grammar references"
    test_sources:
      - "Sibawayh's Al-Kitab example sentences"
      - "Quranic Arabic Corpus (verified interpretations)"
      - "Standard MSA grammar textbook examples"
      - "Known problematic constructions"
      - "Edge cases from Arabic linguistics literature"

  regression_tests:
    description: "Ensure rule updates don't break existing analyses"
    test_suite: "10,000+ sentence corpus with known-expected analyses"
    acceptance_criteria:
      - "No regression in known-grammatical sentences"
      - "No regression in known-ungrammatical sentences (flags unchanged)"
      - "New rules only affect intended constructions"
```

### 13.2 Test Fixture Format

```yaml
RuleEngineTestFixture:
  description: "Input-output pair for testing the Rule Engine"

  input:
    gir: GrammarIR                         # The GIR before rules
    school: string                         # Grammar school
    rule_set_version: string | null        # Specific rule set version
    strict_mode: boolean

  expected:
    confirmed_analysis: string | null      # Analysis that should be confirmed
    rejected_analyses: string[]            # Analyses that should be rejected
    feature_modifications:                 # Expected modifications
      - token_index: integer
        feature: string
        expected_value: string

    flags:                                 # Expected flags
      - code: string
        severity: "error" | "warning" | "info"
        token_indices: integer[]

    rule_applications_min: integer         # Minimum number of rules applied
    rule_applications_max: integer         # Maximum number of rules applied

    remaining_ambiguity: integer           # Expected number of surviving paths
    all_rejected: boolean                  # Whether all analyses rejected

  metadata:
    rule_ids_fired: string[]               # Rules that MUST fire
    rule_ids_blocked: string[]             # Rules that MUST NOT fire
    description: string                    # What this test verifies
```

### 13.3 Example Test Cases

```yaml
# Test 1: Subject-Verb Person Agreement
Test_SubjectVerbAgreement_Person:
  input:
    school: "basra"
    gir:
      sentence_type: "jumlah_fi'liyyah"
      tokens:
        - text: "يَكْتُبُ"
          role: "fi'l"
          features:
            person: "third"
            number: "singular"
            gender: "masculine"
        - text: "أَنْتَ"
          role: "fa'il"
          features:
            person: "second"
            number: "singular"
            gender: "masculine"
  expected:
    rejected_analyses: ["1"]              # Second-person subject not matching third-person verb
    flags:
      - code: "SUBJECT_VERB_PERSON_MISMATCH"
        severity: "error"
        token_indices: [0, 1]
    rule_applications_min: 1
    remaining_ambiguity: 0                # Only one possible path: reject

# Test 2: Preposition Governs Genitive
Test_PrepositionCase_Genitive:
  input:
    school: "basra"
    gir:
      sentence_type: "jumlah_fi'liyyah"
      tokens:
        - text: "فِي"
          role: "harf_jarr"
        - text: "بَيْتٌ"
          role: "majrur"
          features:
            case: "nominative"            # Incorrect case
  expected:
    feature_modifications:
      - token_index: 1
        feature: "case"
        expected_value: "genitive"
    rule_applications_min: 1

# Test 3: Inna Case Reassignment
Test_Inna_CaseReassignment:
  input:
    school: "basra"
    gir:
      sentence_type: "jumlah_ismiyyah"
      tokens:
        - text: "إِنَّ"
          role: "harf_nasb"
        - text: "الطَّالِبُ"
          role: "mubtada'"
          features:
            case: "nominative"            # Needs to change to accusative
        - text: "مُجْتَهِدٌ"
          role: "khabar"
          features:
            case: "nominative"
  expected:
    feature_modifications:
      - token_index: 1
        feature: "case"
        expected_value: "accusative"      # Mubtada' becomes ism inna → accusative
    rule_applications_min: 1
    flags:
      - code: "INNA_CONSTRUCTION"
        severity: "info"
```

---

## 14. Implementation Guidance

### 14.1 Recommended Implementation Languages

| Component | Recommended Language | Rationale |
|-----------|-------------------|-----------|
| **Rule compiler** | Rust / C++ | Performance-critical, parsing + compilation |
| **Condition evaluator** | Rust / C++ | Tree-walking interpreter, must be fast |
| **Action executor** | Rust / C++ | Tight loop, modifies workflow state |
| **Anaphora resolution** | Rust / C++ | Scoring algorithm, integer operations |
| **Plugin interface** | Rust + WASM | Sandboxed rule set loading |
| **Test framework** | Rust (cargo test) | Native integration |
| **Rule authoring tools** | TypeScript / Python | Developer experience tools |

### 14.2 Key Implementation Risks

| Risk | Mitigation |
|------|-----------|
| **Circular rule dependencies** | Fixpoint detection with max iterations; detect no-state-change after full pass |
| **Slow condition evaluation** | Rule indexing by required_features; path caching; short-circuit evaluation |
| **Memory explosion from deep copies** | Per-request arena allocation; shallow copy where safe; copy-on-write |
| **Non-deterministic rule ordering** | Strict priority sort; tie-breaking by rule ID alphabetically; no parallel evaluation ambiguity |
| **Rule conflict deadlocks** | Clear priority-based resolution; strict_mode for fail-fast; normal mode preserves ambiguity |
| **Large rule sets (10,000+)** | Rule indexing reduces effective rules to ~200; lazy loading per school |

### 14.3 Data Flow Through the Engine

```diff
  IR-6 (GrammarIR from MOD-06)
    │
    │  MOD-07 — Subsystem 1: Rule Loader & Compiler
    ▼
  ┌──────────────────────────────────────┐
  │  resolve_rule_set(school, version)    │
  │  → parse .agosrule files             │
  │  → compile to CompiledRule[]         │
  │  → sort by priority                  │
  │  → cache for reuse                   │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  initialize_workflow(IR-6)           │
  │  → clone GIR                         │
  │  → build token indices               │
  │  → build ambiguity paths             │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 2: Rule Execution Loop    │
  │                                      │
  │  iteration = 0                       │
  │  while not fixpoint:                 │
  │    iteration += 1                    │
  │    for rule in rules:                │
  │      │                              │
  │      ├── check rule applicability   │
  │      │   (fast filter by features/  │
  │      │    roles/sentence type)      │
  │      │                              │
  │      ├── evaluate condition         │
  │      │   (walk AST, resolve paths)  │
  │      │                              │
  │      ├── if true: execute actions   │
  │      │   (confirm/reject/modify/    │
  │      │    flag/resolve)             │
  │      │                              │
  │      └── record application         │
  │          (evidence + RuleRecord)    │
  │                                      │
  │    check all-rejected → flag        │
  │    check application limit → warn   │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 3: Ambiguity Management   │
  │                                      │
  │  → prune rejected paths             │
  │  → sort remaining by confidence     │
  │  → detect unresolvable ambiguity    │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 4: Output Assembly        │
  │                                      │
  │  → build AnnotatedGIR               │
  │  → collect rule applications         │
  │  → generate flags                    │
  │  → validate output structure         │
  └──────────────────────────────────────┘
    │
    │  IR-7 (AnnotatedGIR)
    ▼
  MOD-08 (KnowledgeGraphResolver)
```

---

## 15. Cross-References

### 15.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | MOD-07 module definition, pipeline position |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | MOD-07 RuleEngine algorithm (foundational) |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Public interface definition for MOD-07 |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | IR-6 input schema, IR-7 output schema |
| SPEC-0001-C7 | Extensibility & Plugin Architecture | rule_set plugin type, RuleSetPlugin interface |
| SPEC-0001-C8 | Security, Validation & Error Handling | Error codes, rule validation, evidence integrity |
| SPEC-0001-C9 | Performance Targets & Constraints | MOD-07 latency/memory/throughput targets |
| SPEC-0101 | Morphology Engine | Consumes IR-4 (morphology) indirectly via IR-6 |
| SPEC-0401 | Knowledge Graph Engine | Consumes IR-7 (AnnotatedGIR) |
| RFC-0001 | Grammar DSL | Rule authoring language interpreted by MOD-07 |
| ADR-0003 | Why Grammar IR | Justifies MOD-06/MOD-07 boundary |

### 15.2 Knowledge Base References

| KB | Title | Relationship |
|----|-------|--------------|
| KB-0005 | Particles | Particle governance rules (preposition case, mood government) |
| KB-0006 | Pronouns | Pronoun types, anaphora resolution rules |
| KB-0007 | Morphological Features | Feature agreement rules, cross-feature constraints |

### 15.3 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh's Al-Kitab | Foundational Arabic grammar rules |
| Wright's Arabic Grammar | Standard reference for school-specific rules |
| Ryding, A Reference Grammar of MSA | Contemporary MSA rules |
| Drools Rule Language | Inspiration for declarative rule engine design |
| Roslyn Analyzer Rules | Inspiration for condition + action rule structure |

---

## Progress Summary

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Introduction & Scope | ✓ COMPLETE |
| Section 2 | Architecture Overview | ✓ COMPLETE |
| Section 3 | Internal Component Model | ✓ COMPLETE |
| Section 4 | Rule Loading & Compilation | ✓ COMPLETE |
| Section 5 | Rule Execution Engine | ✓ COMPLETE |
| Section 6 | Ambiguity Pruning & Management | ✓ COMPLETE |
| Section 7 | Anaphora Resolution | ✓ COMPLETE |
| Section 8 | School-Specific Rule Sets | ✓ COMPLETE |
| Section 9 | Conflict Detection & Resolution | ✓ COMPLETE |
| Section 10 | Evidence & Flag Generation | ✓ COMPLETE |
| Section 11 | Performance & Optimization | ✓ COMPLETE |
| Section 12 | Plugin Integration | ✓ COMPLETE |
| Section 13 | Testing & Validation Strategy | ✓ COMPLETE |
| Section 14 | Implementation Guidance | ✓ COMPLETE |
| Section 15 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–9), SPEC-0101, RFC-0001, KB-0005, KB-0006, KB-0007, ADR-0003.

**Recommended next step:** SPEC-0401 (Knowledge Graph Engine) — the next downstream component consuming MOD-07's AnnotatedGIR output.
