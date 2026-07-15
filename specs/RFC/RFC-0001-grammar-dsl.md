---
rfc_id: RFC-0001
title: Grammar DSL — Domain-Specific Language for Grammatical Rules
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage (MOD-07 RuleEngine)
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0201: Rule Engine (planned)
  - ADR-0001: Compiler Architecture Rationale
  - KB-0007: Morphological Features Taxonomy (planned)
supersedes: None
---

# RFC-0001: Grammar DSL

## Table of Contents

1. [Status](#1-status)
2. [Purpose & Scope](#2-purpose--scope)
3. [Design Philosophy](#3-design-philosophy)
4. [Lexical Structure](#4-lexical-structure)
5. [Data Types](#5-data-types)
6. [Expressions](#6-expressions)
7. [Rule Structure](#7-rule-structure)
8. [Metadata Block](#8-metadata-block)
9. [Condition Block](#9-condition-block)
10. [Action Block](#10-action-block)
11. [Built-in Functions](#11-built-in-functions)
12. [Built-in Variables](#12-built-in-variables)
13. [Import System](#13-import-system)
14. [Standard Library](#14-standard-library)
15. [Formal Grammar (EBNF)](#15-formal-grammar-ebnf)
16. [Complete Examples](#16-complete-examples)
17. [Error Handling](#17-error-handling)
18. [Performance Considerations](#18-performance-considerations)
19. [Cross-References](#19-cross-references)

---

## 1. Status

**Draft.** This RFC is open for review and comment. The DSL syntax and semantics are subject to change before acceptance.

**Status:** `Draft` → `Review` → `Accepted` → `Implemented` → `Converted to SPEC-0201`

---

## 2. Purpose & Scope

### 2.1 Purpose

The Grammar DSL (Domain-Specific Language) is the language in which grammatical rules are authored for the AGOS platform. It is designed to be used by:

- **Linguists and Arabic grammar experts** who author and maintain rule sets.
- **Compiler engineers** who implement the Rule Engine (MOD-07) that executes DSL rules.
- **Educators and researchers** who create custom rule sets for pedagogical or research purposes.

The DSL enables the expression of Arabic grammatical rules in a form that is:
- Human-readable and writable by domain experts (linguists), not just programmers.
- Deterministically executable by the AGOS Rule Engine.
- Versionable, testable, and composable into complete grammar school rule sets.

### 2.2 Scope

**In scope:**
- Rule definition syntax (metadata, condition, action).
- Expressions for accessing and comparing grammatical features.
- Built-in functions for rule effects (confirm, reject, modify, flag, resolve).
- Import system for composing rules across files.
- Standard library of common grammatical predicates.

**Out of scope:**
- The execution engine that interprets DSL rules (covered by SPEC-0201 Rule Engine).
- The GIR data model that the DSL queries (covered by Chapter 5 of SPEC-0001).
- Specific grammar schools' rule sets (authored in this DSL but distributed as plugins).
- Linguistic knowledge bases (covered by KB-0001–0007).

---

## 3. Design Philosophy

### 3.1 Principles

1. **Linguist-first syntax.** The DSL should be readable and writable by a linguist who knows Arabic grammar but has minimal programming experience. Keywords use Arabic grammatical terminology where appropriate.

2. **Declarative, not imperative.** Rules describe *what* to match and *what* to do, not *how* to find matches or *how* to apply changes. The execution engine handles the how.

3. **Deterministic by construction.** Every DSL construct has a well-defined, deterministic meaning. There is no randomness, no global state, and no time-dependent behavior.

4. **Stateless rules.** Rules operate on a read-only snapshot of the GIR and produce a list of effects (confirmations, rejections, modifications). They cannot have side effects beyond these declared effects.

5. **Composable.** Rules can be imported from other files, organized into categories, and layered to form complete grammar school rule sets.

6. **Testable.** Every rule can be tested in isolation with a known GIR input and expected effects output.

### 3.2 File Format

DSL rules are authored in `.agosrule` files. The recommended file extension is `.agosrule`.

Files are UTF-8 encoded plain text. The MIME type is `text/x-agos-rule`.

---

## 4. Lexical Structure

### 4.1 Character Set

The DSL uses the Unicode character set. The following Unicode blocks are supported:

| Block | Range | Usage |
|-------|-------|-------|
| Basic Latin | U+0000–U+007F | Keywords, operators, punctuation |
| Arabic | U+0600–U+06FF | Rule comments, string literals |
| Arabic Supplement | U+0750–U+077F | Extended Arabic in comments/strings |
| Arabic Extended-A | U+08A0–U+08FF | Extended Arabic in comments/strings |

### 4.2 Comments

```ebnf
(* Single-line comment *)
// Single-line comment (alternative)
/* Multi-line
   comment */
```

Comments are ignored by the parser. They are preserved in the AST for documentation generation tools.

The `//` style is the preferred convention throughout AGOS rule sets. The `(* *)` and `/* */` styles are supported for compatibility.

### 4.3 Whitespace

Whitespace (spaces, tabs, newlines) is ignored except where it separates tokens. The DSL is free-form within blocks.

### 4.4 Keywords

The following keywords are reserved:

```
rule        metadata    condition   action      import
if          else        forall      exists      matches
in          not         and         or          true
false       null        confirm     reject      modify
flag        resolve     let         return      with
```

### 4.5 Identifiers

```ebnf
identifier = (letter | "_") { letter | digit | "_" | "'" }
letter = "A".."Z" | "a".."z" | Arabic_letter
digit = "0".."9"
```

Identifiers are case-sensitive. Identifiers starting with uppercase are treated as **types**. Identifiers starting with lowercase or underscore are treated as **variables** or **field names**. Identifiers may contain hyphens (`-`) for multi-word names.

Examples: `sentence`, `token`, `fi'l`, `maf'ul_bi-hi`, `jumlah_fi'liyyah`, `SUBJECT_VERB_AGREEMENT`

### 4.6 Literals

```ebnf
string_literal = '"' { character } '"'
integer_literal = digit { digit }
float_literal = digit { digit } "." digit { digit }
boolean_literal = "true" | "false"
null_literal = "null"
```

Strings support escape sequences: `\"`, `\\`, `\n`, `\t`, `\uXXXX` (Unicode code point).

### 4.7 Operators

```ebnf
comparison_op = "==" | "!=" | "<" | ">" | "<=" | ">="
logical_op = "and" | "or" | "not"
assignment_op = ":="
member_op = "."
path_op = "->"
range_op = ".."
```

---

## 5. Data Types

### 5.1 Primitive Types

| Type | Description | Example Literal |
|------|-------------|-----------------|
| `string` | UTF-8 text | `"basra"` |
| `integer` | 64-bit signed integer | `50` |
| `float` | 64-bit floating point | `0.95` |
| `boolean` | True or false | `true` |
| `null` | Absence of value | `null` |

### 5.2 Grammatical Types

These types reflect the structure of the GIR (see SPEC-0001 Chapter 5).

#### Sentence

```ebnf
Sentence = {
    type: string,                      // "jumlah_fi'liyyah", "jumlah_ismiyyah", etc.
    tokens: Token[],                   // All tokens in the sentence
    constituents: Constituent[],       // Parse tree constituents
    flags: Flag[],                     // Grammatical flags on the sentence
}
```

#### Token

```ebnf
Token = {
    index: integer,                    // Position in sentence
    text: string,                      // Token text
    features: FeatureMap,              // Morphological features
    role: string | null,               // Syntactic role (e.g., "fi'l", "fa'il")
    clitics: {                         // Separated clitics
        prefixes: string[],
        suffixes: string[],
    },
    position: integer,                 // Token position within sentence
}
```

#### FeatureMap

```ebnf
FeatureMap = {
    root: string | null,               // Root text (e.g., "كتب")
    wazan: string | null,              // Pattern text (e.g., "فَعَلَ")
    pos: string,                       // Part of speech
    gender: string | null,             // "masculine" | "feminine"
    number: string | null,             // "singular" | "dual" | "plural"
    person: string | null,             // "first" | "second" | "third"
    tense: string | null,              // "past" | "present" | "future" | "imperative"
    mood: string | null,               // "indicative" | "subjunctive" | "jussive" | "energetic"
    voice: string | null,              // "active" | "passive"
    case: string | null,               // "nominative" | "accusative" | "genitive"
    state: string | null,              // "definite" | "indefinite"
    form: integer | null,              // Verb form (I-XV)
    // Additional features from KB-0007 are accessible as string values
    [key: string]: string | integer | boolean | null,
}
```

#### Constituent

```ebnf
Constituent = {
    type: "word" | "phrase" | "clause",
    role: string,                      // Syntactic role
    token_indices: integer[],          // Spanned token indices
    children: Constituent[],           // Sub-constituents
    features: { [key: string]: string },
}
```

### 5.3 Collection Types

```ebnf
Array<T> = T[]                         // Ordered list, 0-indexed
Map<K, V> = { [K]: V }                // Key-value map
```

Arrays support:

| Expression | Meaning |
|-----------|---------|
| `tokens[0]` | First element |
| `tokens[-1]` | Last element |
| `tokens[1..3]` | Slice (elements 1, 2) |
| `tokens.length` | Number of elements |

---

## 6. Expressions

### 6.1 Path Expressions

Path expressions navigate the GIR data model using dot notation:

```
sentence.type                                    → "jumlah_fi'liyyah"
sentence.tokens[0].features.gender              → "masculine"
sentence.tokens[-1].features.case               → "accusative"
sentence.constituents[0].role                   → "fi'l"
token.role                                      → "fa'il"
token.features.gender                           → "feminine"
token.features.number                           → "plural"
```

### 6.2 Shorthand Paths

Common paths have shorthands for readability:

| Shorthand | Full Path | Description |
|-----------|-----------|-------------|
| `sentence.type` | `sentence.type` | Sentence type |
| `sentence.tokens` | `sentence.tokens` | All tokens |
| `fi'l` | first token where role == "fi'l" | The verb |
| `fa'il` | first token where role == "fa'il" | The subject |
| `mubtada'` | first token where role == "mubtada" | Topic of nominal sentence |
| `khabar` | first token where role == "khabar" | Comment of nominal sentence |
| `token` | Current token in iteration | Current token (in `forall`) |
| `governing_particle` | The governing particle of current token | Governor |

Shorthands are resolved to full paths by the compiler before execution.

### 6.3 Comparison Expressions

```
expr == expr        // Equality
expr != expr        // Inequality
expr <  expr        // Less than
expr >  expr        // Greater than
expr <= expr        // Less than or equal
expr >= expr        // Greater than or equal
expr matches regex  // Regex match (string only, RE2-compatible)
expr in collection  // Membership (string in array, key in map)
```

**Regex flavor:** The `matches` operator uses the [RE2](https://github.com/google/re2) regular expression syntax (backtracking-free, O(n) worst-case). Nested quantifiers (e.g., `(a+)+`) and backreferences are **prohibited** and produce a compile-time error (DSL_PARSE_ERROR). This enforces the ReDoS prevention requirements defined in SPEC-0001 Chapter 8 Section 3.5.

### 6.4 Logical Expressions

```
not expr            // Logical NOT
expr and expr       // Logical AND
expr or expr        // Logical OR
```

Short-circuit evaluation: `and` and `or` use short-circuit evaluation. If the left operand determines the result, the right operand is not evaluated.

### 6.5 Quantified Expressions

```
forall (var in collection) { expr }   // Universal quantification
exists (var in collection) { expr }   // Existential quantification
```

Examples:

```
forall (token in sentence.tokens) {
    token.features.case == "nominative"
}

exists (constituent in sentence.constituents) {
    constituent.role == "fa'il"
}
```

### 6.6 Conditional Expression

```
if (expr) { block } [else { block }]
```

Conditional expressions can be used within action blocks for branching behavior.

### 6.7 Literal Expressions

```
"string literal"    // String
42                  // Integer
3.14                // Float
true                // Boolean
false               // Boolean
null                // Null
["a", "b", "c"]     // Array literal
{"key": "value"}    // Map literal
```

---

## 7. Rule Structure

### 7.1 Top-Level Syntax

A rule file is a sequence of rule definitions and imports:

```
// Comments
import "filename"

rule "id: Title" {
    metadata { ... }
    condition { ... }
    action { ... }
}

rule "id: Title" {
    metadata { ... }
    condition { ... }
    action { ... }
}
```

### 7.2 Rule Anatomy

Each rule has three mandatory blocks:

```
rule "unique-id: Human-readable title" {
    metadata {
        // Rule metadata (see Section 8)
    }

    condition {
        // Condition expression (see Section 9)
    }

    action {
        // Action statements (see Section 10)
    }
}
```

---

## 8. Metadata Block

### 8.1 Syntax

```
metadata {
    id: "basra-0103"
    school: "basra"
    version: "1.2.0"
    priority: 50
    description: "In verbal sentences, the verb agrees with the subject in person."
    source: "Sibawayh, Al-Kitab, Vol. 1, p. 234"
    tags: ["agreement", "fi'l", "fa'il", "verbal-sentence"]
    category: "agreement"
    enabled: true
}
```

### 8.2 Metadata Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | Yes | Unique rule identifier (school-specific prefix recommended) |
| `school` | `string` | Yes | Grammar school this rule belongs to |
| `version` | `string` | Yes | Rule version (semver) |
| `priority` | `integer` | Yes | Rule application priority (higher = earlier) |
| `description` | `string` | Yes | Human-readable rule description |
| `source` | `string` | No | Source reference (book, author, page) |
| `tags` | `string[]` | No | Categorization tags |
| `category` | `string` | No | Rule category (e.g., "agreement", "case-assignment") |
| `enabled` | `boolean` | No | Whether the rule is active (default: true) |

---

## 9. Condition Block

### 9.1 Syntax

```
condition {
    // One or more expressions
    // Multiple expressions are implicitly ANDed
    sentence.type == "jumlah_fi'liyyah"
    fi'l.person != fa'il.person
}
```

### 9.2 Semantics

The condition block contains one or more boolean expressions. If multiple expressions are present, they are implicitly combined with `and`. The condition is satisfied if and only if ALL expressions evaluate to `true`.

Explicit `and`, `or`, and `not` can be used for complex conditions:

```
condition {
    sentence.type == "jumlah_fi'liyyah"
    and (fi'l.person != fa'il.person or fi'l.number != fa'il.number)
    and not (fi'l.voice == "passive")
}
```

### 9.3 Accessing the GIR

Conditions can access any part of the GIR through the built-in variables:

```
// Sentence-level checks
sentence.type == "jumlah_fi'liyyah"
sentence.tokens.length > 2

// Token-level checks (using shorthand for specific roles)
fi'l.tense == "past"
fa'il.case == "nominative"
maf'ul_bi-hi.case == "accusative"

// Feature comparisons
fi'l.person == fa'il.person
fi'l.number == fa'il.number
fi'l.gender != fa'il.gender

// Existence checks
exists(fa'il)                                // Subject exists
forall(token in sentence.tokens) {          // All tokens have case
    token.features.case != null
}

// Pattern matching
fi'l.root matches "ف ع ل"                   // Specific root
fi'l.wazan matches "فَاعَلَ"                // Specific pattern
```

---

## 10. Action Block

### 10.1 Syntax

```
action {
    // Action statements
    confirm(analysis_id)
    reject("reason")
    modify(target, new_value)
    flag(severity, code, target...)
    resolve(pronoun_index, antecedent_index)

    // Conditional branching
    if (expr) {
        confirm(analysis_id)
    } else {
        flag("warning", "CODE", token)
    }

    // Quantified actions
    forall (token in sentence.tokens) {
        if (token.features.case == null) {
            flag("info", "MISSING_CASE", token)
        }
    }
}
```

### 10.2 Semantics

Actions are executed in order. Each action produces a `RuleApplication` record that is added to the evidence trail.

The action block can contain:
- Built-in function calls (see Section 11).
- `if`/`else` conditional branching.
- `forall` quantified actions (to apply actions to multiple tokens).

---

## 11. Built-in Functions

### 11.1 confirm

```
confirm([analysis_id: string])
```

Marks an ambiguous analysis as correct. If no `analysis_id` is provided, the current analysis path is confirmed.

**Effect:** The confirmed analysis is retained. Conflicting unconfirmed analyses may be deprioritized.

**Example:**
```
confirm()                                    // Confirm current path
confirm("anl-0-0")                           // Confirm specific analysis
```

### 11.2 reject

```
reject(reason: string, [analysis_id: string])
```

Removes an ambiguous analysis from consideration.

**Effect:** The rejected analysis is removed. The reason is recorded in the evidence trail.

**Example:**
```
reject("Verb cannot be passive with overt object")
reject("Person mismatch", "anl-0-1")
```

### 11.3 modify

```
modify(target: FeatureAccessor, value: any)
```

Changes a feature value on a token or constituent.

**Effect:** The specified feature is changed to the new value. The old value is recorded in the evidence trail for auditability.

**Example:**
```
modify(maf'ul_bi-hi.features.case, "accusative")
modify(fi'l.features.mood, "jussive")
modify(sentence.type, "jumlah_kāna")
```

### 11.4 flag

```
flag(severity: "error" | "warning" | "info", code: string, targets: TokenOrConstituent...)
```

Adds a grammatical flag (error, warning, or info) to the analysis.

**Effect:** A flag is added to the annotated GIR. Flags are visible to the Explanation Engine and API responses.

**Standard Flag Codes:**

| Code | Severity | Description |
|------|----------|-------------|
| `SUBJECT_VERB_PERSON_MISMATCH` | error | Verb subject disagree in person |
| `SUBJECT_VERB_NUMBER_MISMATCH` | error | Verb subject disagree in number |
| `PREPOSITION_CASE_MISMATCH` | error | Noun after preposition not in genitive |
| `ADJECTIVE_AGREEMENT_MISMATCH` | error | Adjective and noun disagree |
| `MISSING_FA_IL` | warning | Active verb without subject |
| `UNUSUAL_CONSTRUCTION` | info | Rare but grammatical construction |
| `UNKNOWN_TOKEN` | info | Token could not be analyzed |

**Example:**
```
flag("error", "SUBJECT_VERB_PERSON_MISMATCH", fi'l, fa'il)
flag("warning", "MISSING_FA_IL", sentence)
flag("info", "UNUSUAL_CONSTRUCTION", token)
```

### 11.5 resolve

```
resolve(pronoun_index: integer, antecedent_index: integer)
```

Resolves an anaphoric reference (e.g., pronoun → antecedent).

**Effect:** A link is established between the pronoun and its antecedent. This information is available to the Explanation Engine for generating clearer explanations.

**Example:**
```
// In sentence: "رأيت زيدًا وهو يقرأ" (I saw Zayd while he was reading)
resolve(pronoun_index=3, antecedent_index=1)
// Pronoun at index 3 refers to noun at index 1
```

---

## 12. Built-in Variables

### 12.1 Top-Level Variables

| Variable | Type | Description |
|----------|------|-------------|
| `sentence` | `Sentence` | The current sentence being analyzed |
| `global` | `Map` | Global analysis context (school, KB versions) |

### 12.2 Role Shorthands

These shorthands resolve to the first token in the sentence with the matching syntactic role:

| Shorthand | Resolves To | Returns |
|-----------|-------------|---------|
| `fi'l` | `role == "fi'l"` | `Token` |
| `fa'il` | `role == "fa'il"` | `Token` |
| `na'ib_fa'il` | `role == "na'ib_al-fa'il"` | `Token` |
| `mubtada'` | `role == "mubtada"` | `Token` |
| `khabar` | `role == "khabar"` | `Token` |
| `maf'ul_bi-hi` | `role == "maf'ul_bi-hi"` | `Token` |
| `maf'ul_mutlaq` | `role == "maf'ul_mutlaq"` | `Token` |
| `maf'ul_fih` | `role == "maf'ul_fih"` | `Token` |
| `hal` | `role == "hal"` | `Token` |
| `tamyiz` | `role == "tamyiz"` | `Token` |
| `mudaf` | `role == "mudaf"` | `Token` |
| `mudaf_ilayh` | `role == "mudaf_ilayh"` | `Token` |
| `governing_particle` | Particle governing current token | `Token` |

### 12.3 Token Feature Accessors

On any `Token`, the following feature paths are available via `.features`:

| Path | Type | Description |
|------|------|-------------|
| `.features.root` | `string \| null` | Root text |
| `.features.wazan` | `string \| null` | Pattern text |
| `.features.pos` | `string` | Part of speech |
| `.features.gender` | `string \| null` | Gender |
| `.features.number` | `string \| null` | Number |
| `.features.person` | `string \| null` | Person |
| `.features.tense` | `string \| null` | Tense |
| `.features.mood` | `string \| null` | Mood |
| `.features.voice` | `string \| null` | Voice |
| `.features.case` | `string \| null` | Case |
| `.features.state` | `string \| null` | State (definiteness) |
| `.features.form` | `integer \| null` | Verb form |

On any `Token`, the following direct properties are available:

| Path | Type | Description |
|------|------|-------------|
| `.index` | `integer` | Token index in sentence |
| `.text` | `string` | Token text |
| `.role` | `string \| null` | Syntactic role |
| `.position` | `integer` | Position (1-based index in sentence) |

---

## 13. Import System

### 13.1 Import Syntax

```
import "filename"                        // Relative import
import "basra-core"                      // Import from standard library
import "basra-core/agreement"            // Import specific module
```

### 13.2 Import Resolution

Imports are resolved relative to the importing file's directory. Standard library imports are resolved from the AGOS DSL standard library path.

```
// In: rules/basra/verbal-sentences.agosrule
import "../common/definitions.agosrule"
import "basra-core/agreement"

rule "basra-0103: ..." { ... }
```

### 13.3 Import Semantics

- Importing a file includes all rules defined in that file.
- Duplicate rule IDs across imports produce a compile-time error.
- Circular imports produce a compile-time error.
- Imported rules inherit their original priority; the priority is NOT adjusted by the import.

---

## 14. Standard Library

### 14.1 Purpose

The DSL standard library provides reusable predicates and helpers that simplify rule authoring. The standard library is versioned and distributed with the AGOS platform.

### 14.2 Standard Library Modules

| Module | File | Description |
|--------|------|-------------|
| `agos-core` | `agos-core.agosrule` | Core type definitions and utility predicates |
| `agos-core/agreement` | `agos-core/agreement.agosrule` | Subject-verb agreement rules |
| `agos-core/case` | `agos-core/case.agosrule` | Case assignment predicates |
| `agos-core/constructions` | `agos-core/constructions.agosrule` | Construction identification (idafa, wasf, etc.) |
| `agos-core/validation` | `agos-core/validation.agosrule` | Input validation helpers |

### 14.3 Standard Predicates

The standard library provides predefined predicates — named boolean expressions that encapsulate common grammatical checks. Predicates are defined using a `predicate` keyword (defined in the formal grammar) and are called as zero-argument functions in conditions.

#### Predicate Definition Syntax

```ebnf
predicate_definition = "predicate" identifier "=" expression
```

A predicate is a named, parameterless boolean expression. Predicates are resolved at compile time and expanded inline at call sites. They cannot have side effects.

#### Built-in Predicates

```ebnf
// In agos-core:
predicate "is_verbal" = sentence.type == "jumlah_fi'liyyah"
predicate "is_nominal" = sentence.type == "jumlah_ismiyyah"
predicate "is_conditional" = sentence.type == "jumlah_shartiyyah"
predicate "has_fa_il" = exists(fa'il)
predicate "has_maf_ul" = exists(maf'ul_bi-hi)
predicate "is_transitive" = token.features.form in [1, 2, 3, 4]  // simplified

// In agos-core/agreement:
predicate "person_agrees" = fi'l.person == fa'il.person
predicate "number_agrees" = fi'l.number == fa'il.number
predicate "gender_agrees" = fi'l.gender == fa'il.gender
```

### 14.4 Using Predicates

Predicates are included via imports and can be called as zero-argument functions in conditions:

```
import "agos-core/agreement"

condition {
    is_verbal()
    and not person_agrees()
}
```

Predicate calls are resolved at compile time: the predicate body replaces the call site, enabling the compiler to optimize the combined expression.

---

## 15. Formal Grammar (EBNF)

### 15.1 File Structure

```ebnf
rule_file = { import_statement | rule_definition }

import_statement = "import" string_literal

rule_definition = "rule" string_literal "{" metadata_block condition_block action_block "}"
```

### 15.2 Metadata Block

```ebnf
metadata_block = "metadata" "{" { metadata_field } "}"
metadata_field = identifier ":" (string_literal | integer_literal | boolean_literal | array_literal)
```

### 15.3 Condition Block

```ebnf
condition_block = "condition" "{" expression "}"

expression = logical_or_expression
logical_or_expression = logical_and_expression { "or" logical_and_expression }
logical_and_expression = negation_expression { "and" negation_expression }
negation_expression = [ "not" ] primary_expression

primary_expression = comparison_expression
                   | quantified_expression
                   | "(" expression ")"
                   | boolean_literal
                   | path_expression

comparison_expression = path_expression comparison_op path_expression
                     | path_expression "matches" regex_literal
                     | path_expression "in" collection_expression

quantified_expression = "forall" "(" identifier "in" collection_expression ")" "{" expression "}"
                      | "exists" "(" identifier "in" collection_expression ")" "{" expression "}"

path_expression = identifier { "." identifier }
                | identifier "[" integer_literal "]"
                | identifier "[" integer_literal ".." integer_literal "]"

collection_expression = path_expression | array_literal
regex_literal = "/" { character } "/"   (* RE2-compatible syntax, bounded backtracking *)
```

### 15.4 Predicate Definitions

```ebnf
predicate_definition = "predicate" identifier "=" expression
```

Predicates are named boolean expressions that are expanded inline at compile time (see Section 14.3).

### 15.5 Action Block

```ebnf
action_block = "action" "{" { action_statement } "}"
action_statement = confirm_call
                 | reject_call
                 | modify_call
                 | flag_call
                 | resolve_call
                 | if_statement
                 | forall_action

confirm_call = "confirm" "(" [ string_literal ] ")"
reject_call = "reject" "(" string_literal [ "," string_literal ] ")"
modify_call = "modify" "(" path_expression "," (string_literal | integer_literal | boolean_literal | path_expression) ")"
flag_call = "flag" "(" string_literal "," string_literal { "," path_expression } ")"
resolve_call = "resolve" "(" pronoun_index "," antecedent_index ")"

if_statement = "if" "(" expression ")" "{" { action_statement } "}" [ "else" "{" { action_statement } "}" ]
forall_action = "forall" "(" identifier "in" collection_expression ")" "{" { action_statement } "}"
```

---

## 16. Complete Examples

### 16.1 Basra School: Verbal Agreement

```ebnf
// File: rules/basra/agreement.agosrule
// Basra School: Subject-Verb Agreement Rules

import "agos-core"

// Rule 1: Basic person agreement
rule "basra-0103: Subject-Verb Person Agreement" {
    metadata {
        id: "basra-0103"
        school: "basra"
        version: "1.2.0"
        priority: 50
        description: "In verbal sentences, the verb must agree with the subject in person."
        source: "Sibawayh, Al-Kitab, Vol. 1, p. 234"
        tags: ["agreement", "fi'l", "fa'il"]
        category: "agreement"
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        and exists(fa'il)
        and fi'l.person != fa'il.person
    }

    action {
        reject("Subject-verb person disagreement")
        flag("error", "SUBJECT_VERB_PERSON_MISMATCH", fi'l, fa'il)
    }
}

// Rule 2: Number agreement (with exceptions for verb-before-subject)
rule "basra-0104: Subject-Verb Number Agreement" {
    metadata {
        id: "basra-0104"
        school: "basra"
        version: "1.1.0"
        priority: 45
        description: "In verbal sentences, verb number agrees with subject. "
                   "Exception: when verb precedes subject, verb is singular."
        source: "Sibawayh, Al-Kitab, Vol. 1, p. 240"
        tags: ["agreement", "number"]
        category: "agreement"
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        and exists(fa'il)
        and fi'l.position < fa'il.position       // Verb before subject
        and fi'l.number == "plural"
    }

    action {
        // Basra: verb before plural subject takes singular
        modify(fi'l.features.number, "singular")
        confirm()
    }
}
```

### 16.2 Basra School: Case Assignment

```ebnf
// File: rules/basra/case-assignment.agosrule
// Basra School: Case Assignment Rules

// Rule: Preposition governs genitive
rule "basra-0201: Preposition governs genitive case" {
    metadata {
        id: "basra-0201"
        school: "basra"
        version: "1.0.0"
        priority: 30
        description: "A noun governed by a preposition takes the genitive case."
        tags: ["case", "preposition"]
        category: "case-assignment"
    }

    condition {
        token.role == "majrur"
        and governing_particle.type == "harf_jarr"
        and token.features.case != "genitive"
    }

    action {
        modify(token.features.case, "genitive")
        confirm(token)
    }
}

// Rule: Subject takes nominative
rule "basra-0202: Subject (fa'il) takes nominative case" {
    metadata {
        id: "basra-0202"
        school: "basra"
        version: "1.0.0"
        priority: 35
        description: "The subject of a verb takes the nominative case."
        tags: ["case", "subject"]
        category: "case-assignment"
    }

    condition {
        token.role == "fa'il"
        and token.features.case != "nominative"
    }

    action {
        modify(token.features.case, "nominative")
        confirm()
    }
}

// Rule: Direct object takes accusative
rule "basra-0203: Direct object (maf'ul bi-hi) takes accusative case" {
    metadata {
        id: "basra-0203"
        school: "basra"
        version: "1.0.0"
        priority: 35
        description: "The direct object of a transitive verb takes the accusative case."
        tags: ["case", "object"]
        category: "case-assignment"
    }

    condition {
        token.role == "maf'ul_bi-hi"
        and token.features.case != "accusative"
    }

    action {
        modify(token.features.case, "accusative")
        confirm()
    }
}
```

### 16.3 Kufa School: Verb Agreement (School-Specific)

```ebnf
// File: rules/kufa/agreement.agosrule
// Kufa School: Subject-Verb Agreement Rules (differs from Basra)

// Rule: Verb before plural non-human subject -> feminine singular verb
rule "kufa-0104: Verb before plural non-human subject" {
    metadata {
        id: "kufa-0104"
        school: "kufa"
        version: "1.1.0"
        priority: 45
        description: "When the verb precedes a plural non-human subject, "
                   "the verb takes feminine singular form."
        source: "Al-Kisa'i, Grammar of the Kufa School"
        tags: ["agreement", "number", "gender"]
        category: "agreement"
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        and fa'il.features.number == "plural"
        and fi'l.position < fa'il.position
        and (fa'il.features.gender == "feminine"
             or fa'il.features.gender == "masculine")  // non-human masculine also takes feminine verb
    }

    action {
        modify(fi'l.features.gender, "feminine")
        modify(fi'l.features.number, "singular")
        confirm()
    }
}
```

### 16.4 Kana and Her Sisters

```ebnf
// File: rules/common/kana-and-sisters.agosrule
// Kana and her sisters (كان وأخواتها)

rule "basra-0305: Kana and her sisters - case reassignment" {
    metadata {
        id: "basra-0305"
        school: "basra"
        version: "1.0.0"
        priority: 40
        description: "Kana and its sister verbs (sara, laysa, asbaha, amsa, etc.) "
                   "raise the subject to accusative and keep the predicate in nominative."
        source: "Sibawayh, Al-Kitab, Vol. 2"
        tags: ["kana", "case", "nominal-sentence"]
        category: "special-verbs"
    }

    condition {
        exists(fi'l)
        and fi'l.features.root in ["كان", "صار", "ليس", "أصبح", "أمسى",
                                   "ظل", "بات", "ما زال", "ما انفك",
                                   "ما فتئ", "ما برح", "ما دام"]
        and sentence.type == "jumlah_fi'liyyah"
    }

    action {
        modify(sentence.type, "jumlah_kāna")
        if (exists(mubtada')) {
            modify(mubtada'.features.case, "accusative")     // Ism kana
        }
        if (exists(khabar)) {
            // Khabar kana remains nominative (or as-is)
            confirm()
        }
        flag("info", "KANA_CONSTRUCTION", sentence)
    }
}
```

---

## 17. Error Handling

### 17.1 Compile-Time Errors

Errors detected during DSL compilation:

| Error Code | Description | Example |
|------------|-------------|---------|
| `DSL_PARSE_ERROR` | Malformed syntax | Missing closing brace |
| `DSL_UNDEFINED_VARIABLE` | Reference to undefined variable | `nonexistent_var == true` |
| `DSL_TYPE_MISMATCH` | Type mismatch in expression | `"string" == 42` |
| `DSL_DUPLICATE_RULE_ID` | Duplicate rule ID across imports | Two rules with id "basra-0103" |
| `DSL_CIRCULAR_IMPORT` | Circular import detected | A imports B imports A |
| `DSL_IMPORT_NOT_FOUND` | Import target not found | `import "nonexistent"` |
| `DSL_INVALID_METADATA` | Missing required metadata field | No `id` field in metadata |

### 17.2 Runtime Errors

Errors that occur during rule execution:

| Error Code | Description | Recovery |
|------------|-------------|----------|
| `RULE_CONDITION_EVAL_FAILED` | Condition expression could not be evaluated | Rule skipped (logged as warning) |
| `RULE_MODIFY_INVALID_PATH` | Modify target path does not exist | Modification ignored (logged) |
| `RULE_FIXPOINT_DETECTED` | No state change after rule application | Rule engine terminates (see SPEC-0001 C3 Section 8) |

### 17.3 Debug Mode

Rules can be compiled and tested in debug mode, which produces detailed trace output:

```
agos-rule compile --debug rules/basra/agreement.agosrule
```

Debug output includes:
- Parsed AST (formatted tree)
- Variable resolution trace
- Type checking results
- Import resolution graph

---

## 18. Performance Considerations

### 18.1 Rule Complexity

| Operation | Approximate Cost | Notes |
|-----------|-----------------|-------|
| Path expression | < 1 μs | Direct field access |
| Comparison | < 1 μs | Value comparison |
| `forall` (10 tokens) | ~10 μs | Linear scan |
| `exists` (10 constituents) | ~5 μs | Short-circuit on first match |
| `matches` (regex) | ~5–50 μs | Depends on pattern complexity |
| `modify` | < 1 μs | Simple field update |
| `reject` | < 1 μs | Flag removal from ambiguity set |

### 18.2 Rule Profile Recommendations

| Aspect | Recommendation |
|--------|---------------|
| Rules per file | < 200 (keep files focused by category) |
| `forall` iterations | < 100 tokens per sentence |
| Regex complexity | Avoid nested quantifiers (ReDoS-safe) |
| Import depth | < 5 levels |
| Priority range | 0–1000 (higher = earlier execution) |

---

## 19. Cross-References

### 19.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | MOD-07 RuleEngine executes DSL rules |
| SPEC-0001-C7 | Extensibility & Plugin Architecture | DSL rules are authored as part of `rule_set` plugins |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | GIR data model that the DSL queries |
| SPEC-0201 | Rule Engine | Detailed specification of the rule execution engine |
| KB-0007 | Morphological Features Taxonomy | Defines valid feature names and values used in DSL |

### 19.2 External References

| Reference | Relevance |
|-----------|-----------|
| Roslyn Analyzer Rules | Inspiration for rule structure (condition + action) |
| Drools Rule Language | Inspiration for declarative rule syntax |
| EBNF (ISO 14977) | Formal grammar notation |
| ECMAScript Regular Expressions | Regex syntax used in `matches` |

---

## Progress Summary

**RFC-0001: Grammar DSL**

| Section | Title | Status |
|---------|-------|--------|
| Sections 1–3 | Status, Purpose, Philosophy | ✓ COMPLETE |
| Section 4 | Lexical Structure | ✓ COMPLETE |
| Section 5 | Data Types | ✓ COMPLETE (7 grammatical types defined) |
| Section 6 | Expressions | ✓ COMPLETE (7 expression types) |
| Sections 7–10 | Rule Structure (Metadata, Condition, Action) | ✓ COMPLETE |
| Section 11 | Built-in Functions | ✓ COMPLETE (5 functions) |
| Section 12 | Built-in Variables | ✓ COMPLETE (20 variables/shorthands) |
| Section 13 | Import System | ✓ COMPLETE |
| Section 14 | Standard Library | ✓ COMPLETE (5 modules) |
| Section 15 | Formal Grammar (EBNF) | ✓ COMPLETE |
| Section 16 | Complete Examples | ✓ COMPLETE (5 rule examples) |
| Sections 17–18 | Error Handling, Performance | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (particularly C3 RuleEngine, C5 GIR, C7 Plugin Architecture).

**Recommended next document:** RFC-0002 — Grammar Bytecode Format, which defines the binary instruction format consumed by the GVM.
