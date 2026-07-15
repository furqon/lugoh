# SPEC-0102: Morphological Features — Reference Taxonomy

| **Field** | **Value** |
|---|---|
| **Spec ID** | SPEC-0102 |
| **Title** | Morphological Features — Reference Taxonomy for Pipeline Modules |
| **Version** | 1.0.0 |
| **Status** | Draft |
| **Depends on** | SPEC-0001-C2 (Architecture), C3 (Pipeline), C4 (Interfaces), C5 (Data Flow), C9 (Performance); KB-0007 (Feature Taxonomy); RFC-0002 (Bytecode Format) |
| **Related SPECs** | SPEC-0101 (Morphology Engine), SPEC-0201 (Rule Engine), SPEC-0301 (Grammar Runtime), SPEC-0401 (KG Engine), SPEC-0501 (Explanation Engine) |
| **Related RFCs** | RFC-0001 (Grammar DSL), RFC-0002 (Grammar Bytecode Format), RFC-0003 (GVM), RFC-0004 (Arabic Grammar Rule DSL) |
| **License** | AGOS Specification License v1.0 |

---

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Feature Taxonomy Overview](#2-feature-taxonomy-overview)
3. [Part of Speech (POS) Reference](#3-part-of-speech-pos-reference)
4. [Inflectional Features Reference](#4-inflectional-features-reference)
5. [Derivational Features Reference](#5-derivational-features-reference)
6. [Prosodic Features Reference](#6-prosodic-features-reference)
7. [Orthographic Features Reference](#7-orthographic-features-reference)
8. [64-Bit Bitfield Reference](#8-64-bit-bitfield-reference)
9. [Feature Usage by Pipeline Module](#9-feature-usage-by-pipeline-module)
10. [Feature Query Patterns in the Rule Engine DSL](#10-feature-query-patterns-in-the-rule-engine-dsl)
11. [Validation Rules & Constraints Reference](#11-validation-rules--constraints-reference)
12. [Feature Agreement Reference](#12-feature-agreement-reference)
13. [Cross-Feature Interaction Reference](#13-cross-feature-interaction-reference)
14. [Plugin Extension Mechanism](#14-plugin-extension-mechanism)
15. [Performance & Memory Reference](#15-performance--memory-reference)
16. [Testing & Quality Reference](#16-testing--quality-reference)
17. [Cross-References](#17-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0102 provides the **developer-oriented reference taxonomy** for AGOS morphological features. Unlike KB-0007 (the authoritative knowledge base that defines *what* the features are and *why* they exist), this specification focuses on *how* features are used across all pipeline modules.

This document serves as the single reference for:

- **Module developers** implementing MOD-04 (feature extraction), MOD-05 (agreement checking), MOD-07 (rule conditions), MOD-10 (GVM feature operations), and MOD-11 (feature display).
- **DSL rule authors** writing conditions that reference features.
- **Plugin developers** extending the feature system with custom features.
- **Quality engineers** writing validation and conformance tests.

### 1.2 Relationship to KB-0007

| Aspect | KB-0007 | SPEC-0102 (this document) |
|--------|---------|--------------------------|
| **Purpose** | Authoritative linguistic definition | Developer usage reference |
| **Audience** | Linguists, KB curators | Module developers, rule authors |
| **Content** | Full linguistic detail, sources, Arabic terminology | Bitfield positions, APIs, query patterns, module usage |
| **Format** | KB entry schema, YAML/JSON examples | Reference tables, pseudocode, DSL patterns |
| **Depth** | Exhaustive per-feature linguistic analysis | Cross-module integration patterns |
| **Versioning** | Semantic versioning as a KB | Aligned with KB-0007 version |
| **Authority** | Source of truth | Derived reference (must match KB-0007) |

### 1.3 Scope

**In scope:**

| Topic | Coverage |
|-------|----------|
| **Feature reference** | All 19 features with canonical names, bit positions, value codes, applicable POS |
| **Bitfield reference** | Complete 64-bit layout, pack/unpack APIs, encoding functions |
| **Module usage patterns** | How each pipeline module reads, writes, transforms, and validates features |
| **DSL query patterns** | How rule conditions reference features (feature paths, operators, combinators) |
| **Agreement rules** | Cross-feature agreement constraints between tokens |
| **Validation rules** | Feature validity checks enforced at each pipeline stage |
| **Plugin extension** | Reserved bits (50–63), custom feature registration, conflict detection |
| **Expression constants** | Pre-defined feature value constants for use in DSL and code |

**Out of scope:**

| Topic | Covered By |
|-------|-----------|
| Linguistic analysis of Arabic features | KB-0007 |
| Feature extraction algorithms (MOD-04) | SPEC-0101 |
| Rule engine internals (MOD-07) | SPEC-0201 |
| GVM feature instructions (MOD-10) | RFC-0003, SPEC-0301 |
| Explanation template patterns (MOD-11) | SPEC-0501 |
| Arabic terminology mapping | RFC-0004 §5 |

### 1.4 Conventions

- **Feature names** are lowercase with underscores: `pos`, `verb_form`, `root_type`
- **Feature values** are lowercase: `masculine`, `indicative`, `past`
- **Bit positions** are zero-indexed from LSB (least significant bit)
- **Path notation** uses dot-separated access: `token.features.gender`
- **C-like pseudocode** for API examples
- **Rust/TypeScript** for code examples (primary implementation languages)

---

## 2. Feature Taxonomy Overview

### 2.1 Complete Feature Inventory

AGOS defines **19 morphological features** organized into 5 categories:

| # | Feature Name | Category | Bit Width | Bit Position | Values | Applies To |
|---|-------------|----------|-----------|-------------|--------|------------|
| 1 | `pos` | POS | 4 | 0–3 | 10 | All |
| 2 | `gender` | Inflectional | 2 | 4–5 | 4 | verb, noun, adj, pronoun, prop_noun |
| 3 | `number` | Inflectional | 2 | 6–7 | 4 | verb, noun, adj, pronoun, prop_noun |
| 4 | `person` | Inflectional | 2 | 8–9 | 4 | verb, pronoun |
| 5 | `tense` | Inflectional | 2 | 10–11 | 4 | verb |
| 6 | `mood` | Inflectional | 2 | 12–13 | 5 | verb |
| 7 | `voice` | Inflectional | 1 | 14 | 2 | verb |
| 8 | `case` | Inflectional | 2 | 15–16 | 4 | noun, adj, prop_noun, adverb |
| 9 | `state` | Inflectional | 1 | 17 | 2 | noun, adj, prop_noun |
| 10 | `verb_form` | Derivational | 5 | 18–22 | 17 | verb |
| 11 | `noun_type` | Derivational | 5 | 23–27 | 16 | noun, adj |
| 12 | `pronoun_type` | Derivational | 4 | 28–31 | 9 | pronoun |
| 13 | `transitivity` | Derivational | 4 | 32–35 | 6 | verb |
| 14 | `root_type` | Derivational | 4 | 36–39 | 10 | verb, noun, adj, prop_noun |
| 15 | `stress_pattern` | Prosodic | 3 | 40–42 | 6 | All |
| 16 | `syllable_count` | Prosodic | 4 | 43–46 | 9 | All |
| 17 | `has_shadda` | Orthographic | 1 | 47 | 2 | All |
| 18 | `has_madd` | Orthographic | 1 | 48 | 2 | verb, noun, adj, pronoun, prop_noun |
| 19 | `has_hamza` | Orthographic | 1 | 49 | 2 | verb, noun, adj, particle, prop_noun, interr |
| — | *reserved* | — | 14 | 50–63 | — | — |

### 2.2 Feature Category Summary

| Category | Count | Bits Used | Purpose | Extraction Source |
|----------|-------|-----------|---------|-------------------|
| POS | 1 | 4 | Grammatical category of each token | MOD-04 (via KB-0005/6 fast-path or KB-0001/2 root+pattern) |
| Inflectional | 8 | 14 | Grammatical properties marked by inflection | MOD-04 (from affixes, vowel patterns, context) |
| Derivational | 5 | 22 | Properties of the derivational pattern | MOD-04 (from KB-0002 wazan, KB-0001 root) |
| Prosodic | 2 | 7 | Stress and syllable properties | MOD-02 (phonological processor, future) |
| Orthographic | 3 | 3 | Written form characteristics | MOD-01 (from Unicode character scanning) |
| **Total** | **19** | **50** | — | — |

### 2.3 Feature DAG (Dependency Graph)

Features have a directed dependency structure. A feature's validity or value may depend on another feature:

```
pos (root)
├──→ verb
│     ├──→ verb_form (I–XV)       ──→ transitivity (inherent or form-modified)
│     ├──→ tense                   ──→ mood (mood only applies when tense=present)
│     ├──→ voice
│     ├──→ stress_pattern
│     ├──→ syllable_count
│     └──→ person · number · gender  (agreement features)
│
├──→ noun / adjective
│     ├──→ noun_type               (masdar, ism_fail, etc.)
│     ├──→ case · state
│     ├──→ gender · number
│     ├──→ root_type
│     ├──→ stress_pattern
│     └──→ syllable_count
│
├──→ pronoun
│     ├──→ pronoun_type
│     ├──→ person · number · gender
│     ├──→ stress_pattern
│     └──→ syllable_count
│
├──→ particle
│     ├──→ has_shadda
│     ├──→ has_hamza
│     ├──→ stress_pattern
│     └──→ syllable_count
│
└──→ (adverb, preposition, conjunction, proper_noun, interrogative)
      ├──→ case (selected)
      ├──→ stress_pattern
      ├──→ syllable_count
      └──→ (orthographic features as applicable)
```

### 2.4 Feature Applicability Matrix

Full POS-to-feature applicability:

```
      Feature         │ vrb │ nou │ adj │ pro │ prt │ adv │ prp │ cnj │ pno │ int │
──────────────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
pos                   │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │
gender                │  ✓  │  ✓  │  ✓  │  ✓  │  —  │  —  │  —  │  —  │  ✓  │  —  │
number                │  ✓  │  ✓  │  ✓  │  ✓  │  —  │  —  │  —  │  —  │  ✓  │  —  │
person                │  ✓  │  —  │  —  │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │
tense                 │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │
mood                  │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │
voice                 │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │
case                  │  —  │  ✓  │  ✓  │  —  │  —  │  ✓  │  —  │  —  │  ✓  │  —  │
state                 │  —  │  ✓  │  ✓  │  —  │  —  │  —  │  —  │  —  │  ✓  │  —  │
verb_form             │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │
noun_type             │  —  │  ✓  │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │
pronoun_type          │  —  │  —  │  —  │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │
transitivity          │  ✓  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │  —  │
root_type             │  ✓  │  ✓  │  ✓  │  —  │  —  │  —  │  —  │  —  │  ✓  │  —  │
stress_pattern        │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │
syllable_count        │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │
has_shadda            │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │  ✓  │
has_madd              │  ✓  │  ✓  │  ✓  │  ✓  │  —  │  —  │  —  │  —  │  ✓  │  —  │
has_hamza             │  ✓  │  ✓  │  ✓  │  —  │  ✓  │  —  │  —  │  —  │  ✓  │  ✓  │

Key:  vrb=verb,  nou=noun,  adj=adjective,  pro=pronoun,  prt=particle,
      adv=adverb,  prp=preposition,  cnj=conjunction,  pno=proper_noun,  int=interrogative
```

### 2.5 POS Codes

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `verb` | فعل | All verb forms I–XV |
| 1 | `noun` | اسم | All noun types |
| 2 | `particle` | حرف | Invariable function words |
| 3 | `pronoun` | ضمير | Personal, demonstrative, relative, etc. |
| 4 | `adjective` | صفة | Descriptive words agreeing with nouns |
| 5 | `adverb` | ظرف | Manner, time, place |
| 6 | `preposition` | حرف جر | Governing genitive case |
| 7 | `conjunction` | حرف عطف | Coordinating and subordinating |
| 8 | `proper_noun` | اسم علم | Proper names |
| 9 | `interrogative` | اسم استفهام | Question words |
| 10–15 | *reserved* | — | — |

---

## 3. Part of Speech (POS) Reference

### 3.1 POS as Root Feature

`pos` is the **root feature** — every analyzed token MUST have a POS value. POS determines which other features are applicable and how they are interpreted.

```rust
/// POS is always the first 4 bits of the 64-bit feature bitfield.
const POS_BIT_OFFSET: u8 = 0;
const POS_BIT_WIDTH: u8 = 4;

/// All valid POS values.
enum PartOfSpeech: u8 {
    Verb          = 0,
    Noun          = 1,
    Particle      = 2,
    Pronoun       = 3,
    Adjective     = 4,
    Adverb        = 5,
    Preposition   = 6,
    Conjunction   = 7,
    ProperNoun    = 8,
    Interrogative = 9,
}
```

### 3.2 POS in Pipeline Modules

| Module | Usage |
|--------|-------|
| **MOD-01** | Not used (pre-feature extraction) |
| **MOD-02** | Not used directly |
| **MOD-03** | Not used directly |
| **MOD-04** | **Populates POS** during morphological analysis (fast-path or root+pattern) |
| **MOD-05** | **Reads POS** for sentence type identification and parse role assignment |
| **MOD-06** | **Propagates POS** through GIR construction |
| **MOD-07** | **Queries POS** in rule conditions (e.g., `token.pos == "verb"`) |
| **MOD-08** | Uses POS for KG entity resolution |
| **MOD-09** | Encodes POS in bytecode |
| **MOD-10** | Reads POS during GVM execution |
| **MOD-11** | Uses POS for I'rab generation and explanation display |

### 3.3 POS in Rule Engine DSL

```ebnf
// POS is accessed via the `pos` field (shorthand for `features.pos`)
// or explicitly via `token.features.pos`.

// Standard school-specific mappings:
// verb     = 0  ("fi'l" in Arabic rule terminology)
// noun     = 1  ("ism")
// particle = 2  ("harf")
// pronoun  = 3  ("damir")
// adjective= 4  ("sifah")
// adverb   = 5  ("zarf")

// Common query patterns:
// Check POS directly:
condition { token.pos == "verb" }
condition { token.pos in ["noun", "pronoun", "proper_noun"] }
condition { token.pos != "particle" }

// Role-based queries (preferred for readability):
condition { token.role == "fi'l" }     // Implies pos == "verb"
condition { token.role == "fa'il" }    // Implies pos in ["noun", "pronoun"]
condition { token.role == "harf_jarr" } // Implies pos == "preposition"
```

---

## 4. Inflectional Features Reference

### 4.1 Gender

| Field | Value |
|-------|-------|
| **Feature name** | `gender` |
| **Category** | Inflectional |
| **Bit position** | 4–5 (2 bits) |
| **Default value** | `unspecified` (code 3) |
| **Applies to POS** | verb, noun, adjective, pronoun, proper_noun |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `masculine` | مذكر | Default for most nouns |
| 1 | `feminine` | مؤنث | Often marked by ة suffix |
| 2 | `common` | مشترك | Same form for both genders |
| 3 | `unspecified` | — | Not applicable or unknown |

**Pack/unpack:**

```rust
// Extract gender from bitfield
fn gender_from_bitfield(bitfield: u64) -> u8 {
    ((bitfield >> 4) & 0x3) as u8
}

// Pack gender into bitfield
fn pack_gender(bitfield: &mut u64, gender: u8) {
    *bitfield |= (gender as u64) << 4;
}
```

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From noun form (ة suffix → feminine), verb suffix (3rd person) |
| MOD-05 | Check | Subject-verb gender agreement |
| MOD-07 | Query | `fi'l.gender == fa'il.gender` |
| MOD-07 | Modify | Gender reassignment for non-human plurals (→ feminine) |
| MOD-10 | Read | Feature extraction instruction |
| MOD-11 | Display | "مذكر" (masculine) / "مؤنث" (feminine) in I'rab |

### 4.2 Number

| Field | Value |
|-------|-------|
| **Feature name** | `number` |
| **Category** | Inflectional |
| **Bit position** | 6–7 (2 bits) |
| **Default value** | `unspecified` (code 3) |
| **Applies to POS** | verb, noun, adjective, pronoun, proper_noun |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `singular` | مفرد | One entity |
| 1 | `dual` | مثنى | Two entities |
| 2 | `plural` | جمع | Three or more |
| 3 | `unspecified` | — | Not applicable |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From noun suffix (ان → dual, ات → fpl, ون → mpl) |
| MOD-05 | Check | Subject-verb number agreement; noun-adjective agreement |
| MOD-07 | Query | `fi'l.number == fa'il.number`; Basra: verb-before-subject → singular |
| MOD-07 | Modify | Force singular for verb-before-plural-subject (Basra) |
| MOD-10 | Read | Stack operations |
| MOD-11 | Display | "مفرد" (singular) / "مثنى" (dual) / "جمع" (plural) |

### 4.3 Person

| Field | Value |
|-------|-------|
| **Feature name** | `person` |
| **Category** | Inflectional |
| **Bit position** | 8–9 (2 bits) |
| **Default value** | `unspecified` (code 3) |
| **Applies to POS** | verb, pronoun |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `first` | متكلم | Speaker |
| 1 | `second` | مخاطب | Addressee |
| 2 | `third` | غائب | Referent not speaker/addressee |
| 3 | `unspecified` | — | Not applicable |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From verb prefix (أ→1st, ت→2nd, ي→3rd) or suffix (تُ→1s) |
| MOD-05 | Check | Subject-verb person agreement |
| MOD-07 | Query | `fi'l.person == fa'il.person` |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "متكلم" (1st) / "مخاطب" (2nd) / "غائب" (3rd) |

### 4.4 Tense

| Field | Value |
|-------|-------|
| **Feature name** | `tense` |
| **Category** | Inflectional |
| **Bit position** | 10–11 (2 bits) |
| **Default value** | `unspecified` (code 3) |
| **Applies to POS** | verb |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `past` | ماض | Completed action (perfect) |
| 1 | `present` | مضارع | Ongoing/future action (imperfect) |
| 2 | `imperative` | أمر | Command |
| 3 | `unspecified` | — | Not applicable |

**Dependency:** When `tense != present`, `mood` MUST be `unspecified` (code 3).
When `tense == present`, `mood` MAY be `indicative`, `subjunctive`, `jussive`, or `energetic`.

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From verb prefix/suffix pattern (past=no prefix, present=prefix) |
| MOD-05 | Read | Determines sentence time reference |
| MOD-07 | Query | `fi'l.tense == "present"`, gateway to mood rules |
| MOD-07 | Modify | Tense-conditional: lam + past → jussive meaning |
| MOD-10 | Read | GVM execution |
| MOD-11 | Display | "ماض" (past) / "مضارع" (present) / "أمر" (imperative) |

### 4.5 Mood

| Field | Value |
|-------|-------|
| **Feature name** | `mood` |
| **Category** | Inflectional |
| **Bit position** | 12–13 (2 bits) |
| **Default value** | `unspecified` (code 3) |
| **Applies to POS** | verb |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `indicative` | مرفوع | Default mood (present tense) |
| 1 | `subjunctive` | منصوب | After subjunctive particles (أَنْ, لَنْ, etc.) |
| 2 | `jussive` | مجزوم | After jussive particles (لَمْ, etc.) |
| 3 | `energetic` | مؤكد | Emphatic/energetic (نَ التوكيد) |

**Dependency:** `mood` only applies when `tense == present`. Past tense and imperative have no mood distinction.

| Tense | Valid Moods |
|-------|-------------|
| past | `unspecified` only |
| present | `indicative`, `subjunctive`, `jussive`, `energetic` |
| imperative | `unspecified` or `jussive` (imperative derived from jussive) |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From vowel ending (u→indicative, a→subjunctive, sukun→jussive) |
| MOD-05 | Pass-through | Mood is set by MOD-07, not MOD-05 |
| MOD-07 | **Assign** | **Primary assignment of mood** via governing particles (SPEC-0201 §5) |
| MOD-07 | Query | `fi'l.mood == "jussive"` (check after government) |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "مرفوع" (indicative) / "منصوب" (subjunctive) / "مجزوم" (jussive) |

### 4.6 Voice

| Field | Value |
|-------|-------|
| **Feature name** | `voice` |
| **Category** | Inflectional |
| **Bit position** | 14 (1 bit) |
| **Default value** | `active` (code 0) |
| **Applies to POS** | verb |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `active` | مبني للمعلوم | Subject performs the action |
| 1 | `passive` | مبني للمجهول | Subject receives the action |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From vowel pattern (fatha→active, damma→passive for perfect) |
| MOD-05 | Read | Determines whether agent is expressed |
| MOD-07 | Query | `fi'l.voice == "passive"` → trigger na'ib al-fa'il rules |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "مبني للمعلوم" (active) / "مبني للمجهول" (passive) |

### 4.7 Case

| Field | Value |
|-------|-------|
| **Feature name** | `case` |
| **Category** | Inflectional |
| **Bit position** | 15–16 (2 bits) |
| **Default value** | `unspecified` (code 3) |
| **Applies to POS** | noun, adjective, proper_noun, adverb (some) |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `nominative` | مرفوع | Subject, predicate of nominal sentence |
| 1 | `accusative` | منصوب | Object, after inna, circumstantial accusative |
| 2 | `genitive` | مجرور | After prepositions, construct state possessor |
| 3 | `unspecified` | — | Invariable noun (مبني) |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From final vowel/nunation (u→nom, a→acc, i→gen) |
| MOD-04 | Default | Without tashkeel: default to `nominative` (low confidence) |
| MOD-05 | Read | Used in parse tree construction |
| MOD-07 | **Assign** | **Primary assignment of case** via government rules |
| MOD-07 | Modify | `modify(maf'ul_bi-hi.case, "accusative")` |
| MOD-07 | Modify | `modify(majrur.case, "genitive")` (preposition) |
| MOD-07 | Modify | `modify(mubtada'.case, "accusative")` (inna construction) |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "مرفوع" (nominative) / "منصوب" (accusative) / "مجرور" (genitive) |

### 4.8 State

| Field | Value |
|-------|-------|
| **Feature name** | `state` |
| **Category** | Inflectional |
| **Bit position** | 17 (1 bit) |
| **Default value** | `definite` (code 0) |
| **Applies to POS** | noun, adjective, proper_noun |

**Value table:**

| Code | Value | Arabic | Notes |
|------|-------|--------|-------|
| 0 | `definite` | معرفة | With الـ, proper noun, or construct state |
| 1 | `indefinite` | نكرة | Without الـ, with nunation |

**Note:** State has no `unspecified` code. When state is not applicable, the bit MUST be 0 (definite).

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From الـ prefix → definite; tanwin → indefinite |
| MOD-05 | Check | Definiteness spread in idafa; mubtada' definiteness |
| MOD-07 | Query | `token.state == "definite"` |
| MOD-07 | Modify | Idafa: second term becomes definite (by possession) |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "معرفة" (definite) / "نكرة" (indefinite) |

---

## 5. Derivational Features Reference

### 5.1 Verb Form

| Field | Value |
|-------|-------|
| **Feature name** | `verb_form` |
| **Category** | Derivational |
| **Bit position** | 18–22 (5 bits) |
| **Default value** | `not_a_verb` (code 0) |
| **Applies to POS** | verb |

**Value table:**

| Code | Value | Arabic | Example | Meaning Domain |
|------|-------|--------|---------|----------------|
| 0 | `not_a_verb` | ليس فعلاً | — | Default for non-verbs |
| 1 | `I` | فَعَلَ | كَتَبَ | Base meaning |
| 2 | `II` | فَعَّلَ | عَلَّمَ | Intensive/causative |
| 3 | `III` | فَاعَلَ | كَاتَبَ | Reciprocal/attemptive |
| 4 | `IV` | أَفْعَلَ | أَكْرَمَ | Causative/declarative |
| 5 | `V` | تَفَعَّلَ | تَعَلَّمَ | Reflexive of II |
| 6 | `VI` | تَفَاعَلَ | تَكَاتَبَ | Reciprocal of III |
| 7 | `VII` | اِنْفَعَلَ | اِنْكَتَبَ | Passive/reflexive of I |
| 8 | `VIII` | اِفْتَعَلَ | اِكْتَتَبَ | Reflexive of I |
| 9 | `IX` | اِفْعَلَّ | اِحْمَرَّ | Colors/defects |
| 10 | `X` | اِسْتَفْعَلَ | اِسْتَكْتَبَ | Requestive/deemed |
| 11 | `XI` | اِفْعَالَّ | اِحْمَارَّ | Intensive colors |
| 12 | `XII` | اِفْعَوْعَلَ | اِحْدَوْدَبَ | Intensive |
| 13 | `XIII` | اِفْعَوَّلَ | اِعْلَوَّطَ | Rare |
| 14 | `XIV` | Reserved | — | Reserved |
| 15 | `XV` | اِفْعَنْلَلَ | اِسْحَنْكَفَ | Very rare |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From wazan signature (KB-0002); primary output of Subsystem 3 |
| MOD-05 | Read | Verb form affects available conjugation patterns |
| MOD-07 | Query | `fi'l.verb_form == "II"` (form II → typically transitive) |
| MOD-07 | Query | `fi'l.verb_form in ["I", "II", "III"]` |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | Roman numeral: "I", "II", ..., "XV" |

### 5.2 Noun Type

| Field | Value |
|-------|-------|
| **Feature name** | `noun_type` |
| **Category** | Derivational |
| **Bit position** | 23–27 (5 bits) |
| **Default value** | `not_a_noun` (code 0) |
| **Applies to POS** | noun, adjective |

**Value table:**

| Code | Value | Arabic | Description |
|------|-------|--------|-------------|
| 0 | `not_a_noun` | ليس اسماً | Default for non-nouns |
| 1 | `masdar` | مصدر | Verbal noun |
| 2 | `ism_fail` | اسم فاعل | Active participle |
| 3 | `ism_maful` | اسم مفعول | Passive participle |
| 4 | `ism_makan` | اسم مكان | Noun of place |
| 5 | `ism_zaman` | اسم زمان | Noun of time |
| 6 | `ism_alah` | اسم آلة | Instrument noun |
| 7 | `sifah_mushabbahah` | صفة مشبهة | Resembling adjective |
| 8 | `tafdil` | تفضيل | Elative (comparative/superlative) |
| 9 | `nisbah` | نسبة | Relative adjective |
| 10 | `jam_taksir` | جمع تكسير | Broken plural |
| 11 | `ism_marrati` | اسم مرة | Instance noun |
| 12 | `ism_hayati` | اسم هيئة | Manner noun |
| 13 | `jins` | اسم جنس | Generic noun |
| 14 | `ism_tasghir` | اسم تصغير | Diminutive |
| 15–31 | *reserved* | — | — |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From wazan signature (KB-0002) |
| MOD-05 | Read | Noun type affects syntactic behavior (masdar can function as verb, etc.) |
| MOD-07 | Query | `token.noun_type == "masdar"` |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | Arabic term for noun type in I'rab |

### 5.3 Pronoun Type

| Field | Value |
|-------|-------|
| **Feature name** | `pronoun_type` |
| **Category** | Derivational |
| **Bit position** | 28–31 (4 bits) |
| **Default value** | `not_a_pronoun` (code 0) |
| **Applies to POS** | pronoun |

**Value table:**

| Code | Value | Arabic | Example |
|------|-------|--------|---------|
| 0 | `not_a_pronoun` | ليس ضميراً | — |
| 1 | `personal_attached` | متصل | -تُ, -كَ |
| 2 | `personal_detached` | منفصل | هُوَ, أَنَا |
| 3 | `demonstrative` | إشارة | هٰذَا |
| 4 | `relative` | موصول | الَّذِي |
| 5 | `interrogative` | استفهام | مَنْ, مَا |
| 6 | `conditional` | شرط | مَنْ, مَهْمَا |
| 7 | `compound` | مركب | بِمَا, لِمَنْ |
| 8–15 | *reserved* | — | — |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From KB-0006 entry |
| MOD-05 | Read | Pronoun type determines syntactic role (subject, object, possessive) |
| MOD-07 | Query | `token.pronoun_type == "personal_detached"` |
| MOD-07 | Resolve | Anaphora resolution uses person/number/gender from pronoun features |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "ضمير متصل" (attached) / "ضمير منفصل" (detached) |

### 5.4 Transitivity

| Field | Value |
|-------|-------|
| **Feature name** | `transitivity` |
| **Category** | Derivational |
| **Bit position** | 32–35 (4 bits) |
| **Default value** | `unspecified` (code 0) |
| **Applies to POS** | verb |

**Value table:**

| Code | Value | Arabic | Description |
|------|-------|--------|-------------|
| 0 | `unspecified` | — | Not specified or unknown |
| 1 | `intransitive` | لازم | No direct object |
| 2 | `transitive_1` | متعدٍ (1) | Takes one object |
| 3 | `transitive_2` | متعدٍ (2) | Takes two objects |
| 4 | `ditransitive` | متعدٍ لثلاثة | Takes three objects |
| 5–15 | *reserved* | — | — |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From KB-0001 (inherent trait); verb form may modify (II → more transitive) |
| MOD-05 | Read | Determines how many objects to expect |
| MOD-07 | Query | `is_transitive_verb()` — checked before object rules |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | "لازم" (intransitive) / "متعدٍ" (transitive) |

### 5.5 Root Type

| Field | Value |
|-------|-------|
| **Feature name** | `root_type` |
| **Category** | Derivational |
| **Bit position** | 36–39 (4 bits) |
| **Default value** | `unspecified` (code 0) |
| **Applies to POS** | verb, noun, adjective, proper_noun |

**Value table:**

| Code | Value | Arabic | Example | Description |
|------|-------|--------|---------|-------------|
| 0 | `unspecified` | — | — | Not specified |
| 1 | `sound` | صحيح ساكن | ك ت ب | All radicals present and stable |
| 2 | `weak_initial` | مثال | و ج د | First radical is wāw or yāʾ |
| 3 | `weak_middle` | أجوف | ق و ل | Second radical is wāw or yāʾ |
| 4 | `weak_final` | ناقص | ر م ي | Third radical is wāw or yāʾ |
| 5 | `hamzated` | مهموز | س أ ل | One or more radicals is hamza |
| 6 | `doubled` | مضاعف | م د د | C₂ and C₃ are identical |
| 7 | `sound_quadriliteral` | رباعي | د ح ر ج | Four-consonant root |
| 8 | `weak_quadriliteral` | رباعي معتل | س ب ع ل | Quadriliteral with weak consonant |
| 9–15 | *reserved* | — | — | — |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-04 | Extract | From KB-0001 root entry or determined during extraction |
| MOD-04 | Guide | Root type affects extraction algorithm (hollow→restore, etc.) |
| MOD-05 | Read | Affects conjugation pattern selection |
| MOD-07 | Query | `token.root_type in ["weak_middle", "weak_final"]` |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | Arabic term for root type |

---

## 6. Prosodic Features Reference

### 6.1 Stress Pattern

| Field | Value |
|-------|-------|
| **Feature name** | `stress_pattern` |
| **Category** | Prosodic |
| **Bit position** | 40–42 (3 bits) |
| **Default value** | `unspecified` (code 0) |
| **Applies to POS** | All |

**Value table:**

| Code | Value | Notes |
|------|-------|-------|
| 0 | `unspecified` | Not determined yet |
| 1 | `final` | Stress on final syllable |
| 2 | `penultimate` | Stress on second-to-last |
| 3 | `antepenultimate` | Stress on third-to-last |
| 4 | `pre_antepenultimate` | Rare, stress on fourth-to-last |
| 5–7 | *reserved* | — |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-02 | Compute | (future) Determined by syllable weight |
| MOD-04 | Pass-through | Not extracted by MOD-04; read from upstream |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | Used in phonetic explanations (future) |

### 6.2 Syllable Count

| Field | Value |
|-------|-------|
| **Feature name** | `syllable_count` |
| **Category** | Prosodic |
| **Bit position** | 43–46 (4 bits) |
| **Default value** | 0 (unspecified) |
| **Applies to POS** | All |

**Value table:**

| Code | Value | Notes |
|------|-------|-------|
| 0 | unspecified | Not determined |
| 1–8 | syllable count | Number of syllables |
| 9–15 | *reserved* | — |

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-02 | Compute | (future) From vowel sequences |
| MOD-04 | Pass-through | Not extracted by MOD-04 |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | Used in phonetic explanations |

---

## 7. Orthographic Features Reference

### 7.1 has_shadda

| Field | Value |
|-------|-------|
| **Feature name** | `has_shadda` |
| **Category** | Orthographic |
| **Bit position** | 47 (1 bit) |
| **Default value** | `false` (code 0) |
| **Applies to POS** | All |

**Value table:** `false` (0) | `true` (1)

**Module usage:**

| Module | Operation | Details |
|--------|-----------|---------|
| MOD-01 | Extract | Detects U+0651 (shadda) in the text |
| MOD-04 | Pass-through | Passes through from MOD-01 |
| MOD-10 | Read | GVM feature instruction |
| MOD-11 | Display | Shows shadda in vocalized output |

### 7.2 has_madd

| Field | Value |
|-------|-------|
| **Feature name** | `has_madd` |
| **Category** | Orthographic |
| **Bit position** | 48 (1 bit) |
| **Default value** | `false` (code 0) |
| **Applies to POS** | verb, noun, adjective, pronoun, proper_noun |

**Value table:** `false` (0) | `true` (1)

### 7.3 has_hamza

| Field | Value |
|-------|-------|
| **Feature name** | `has_hamza` |
| **Category** | Orthographic |
| **Bit position** | 49 (1 bit) |
| **Default value** | `false` (code 0) |
| **Applies to POS** | verb, noun, adjective, particle, proper_noun, interrogative |

**Value table:** `false` (0) | `true` (1)

**Note:** `has_hamza` is a surface feature (hamza in the written form). It is distinct from `root_type = hamzated`, which is a lexical property of the root.

---

## 8. 64-Bit Bitfield Reference

### 8.1 Complete Bitfield Layout

```
Bit:      63  62  61  60  59  58  57  56  55  54  53  52  51  50
         ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
Reserved  │                                            │
         └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘

Bit:      49  48  47  46  45  44  43  42  41  40  39  38  37  36
         ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
Ortho    │ H │ M │ S │                                       │
         │ A │ A │ H │   Syllable Count    │  Stress Pattern  │
         │ M │ D │ D │        (43-46)      │     (40-42)      │
         │ Z │ D │ D │                     │                  │
         │ A │   │ A │                     │                  │
         ├───┼───┼───┼─────────────────────┼──────────────────┼
Deriv    │                                                       │
         │    Root Type        │   Transitivity  │              │
         │      (36-39)        │     (32-35)     │              │
         ├─────────────────────┴─────────────────┤              │
         │                                       │              │
         └───────────────────────────────────────────────────────┘

Bit:      35  34  33  32  31  30  29  28  27  26  25  24  23  22
         ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
         │                       │                               │
         │    Pronoun Type      │           Noun Type            │
         │      (28-31)         │          (23-27)               │
         ├──────────────────────┴────────────────────────────────┤
         │                                                       │
         └───────────────────────────────────────────────────────┘

Bit:      21  20  19  18  17  16  15  14  13  12  11  10   9   8
         ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
         │                       │ S │       │   │           │   │
         │    Verb Form          │ T │ Case  │ V │  Mood     │ T │
         │     (18-22)           │ A │(15-16)│ O │ (12-13)   │ E │
         │                       │ T │       │ I │           │ N │
         │                       │ E │       │ C │           │   │
         ├───────────────────────┴───┴───────┴───┴───────────┴───┤
         │                                                       │
         └───────────────────────────────────────────────────────┘

Bit:       7   6   5   4   3   2   1   0
         ┌───┬───┬───┬───┬───┬───┬───┬───┐
         │       │       │               │
         │ Person│Gender │     POS       │
         │ (8-9) │ (6-7) │    (0-3)     │
         │       │       │               │
         ├───────┴───────┴───────────────┤
         │   Number + Person + Gender    │
         └───────────────────────────────┘
```

### 8.2 Bitfield Summary Table

| Bits | Feature | Width | Values | Shift Mask |
|------|---------|-------|--------|------------|
| 0–3 | `pos` | 4 | 0–9 (verb through interrogative) | `0xF << 0` |
| 4–5 | `gender` | 2 | 0=masc, 1=fem, 2=common, 3=unspec | `0x3 << 4` |
| 6–7 | `number` | 2 | 0=sg, 1=dual, 2=pl, 3=unspec | `0x3 << 6` |
| 8–9 | `person` | 2 | 0=1st, 1=2nd, 2=3rd, 3=unspec | `0x3 << 8` |
| 10–11 | `tense` | 2 | 0=past, 1=pres, 2=impv, 3=unspec | `0x3 << 10` |
| 12–13 | `mood` | 2 | 0=ind, 1=subj, 2=juss, 3=energ | `0x3 << 12` |
| 14 | `voice` | 1 | 0=active, 1=passive | `0x1 << 14` |
| 15–16 | `case` | 2 | 0=nom, 1=acc, 2=gen, 3=unspec | `0x3 << 15` |
| 17 | `state` | 1 | 0=def, 1=indef | `0x1 << 17` |
| 18–22 | `verb_form` | 5 | 0=not_verb, 1=I, ..., 15=XV | `0x1F << 18` |
| 23–27 | `noun_type` | 5 | 0=not_noun, 1=masdar, ..., 14=tasghir | `0x1F << 23` |
| 28–31 | `pronoun_type` | 4 | 0=not_pron, 1=attached, ..., 7=compound | `0xF << 28` |
| 32–35 | `transitivity` | 4 | 0=unspec, 1=intrans, ..., 4=ditrans | `0xF << 32` |
| 36–39 | `root_type` | 4 | 0=unspec, 1=sound, ..., 8=weak_quad | `0xF << 36` |
| 40–42 | `stress_pattern` | 3 | 0=unspec, 1=final, ..., 4=pre_antepen | `0x7 << 40` |
| 43–46 | `syllable_count` | 4 | 0=unspec, 1–8=count | `0xF << 43` |
| 47 | `has_shadda` | 1 | 0=false, 1=true | `0x1 << 47` |
| 48 | `has_madd` | 1 | 0=false, 1=true | `0x1 << 48` |
| 49 | `has_hamza` | 1 | 0=false, 1=true | `0x1 << 49` |
| 50–63 | *reserved* | 14 | Must be zero for KB-defined features | `0x3FFF << 50` |

### 8.3 Pack Features (Reference Implementation)

```rust
/// Pack a complete feature set into a 64-bit bitfield.
/// Returns the packed bitfield.
fn pack_features(features: &FeatureSet) -> u64 {
    let mut bf: u64 = 0;

    // POS (bits 0–3)
    bf |= (features.pos as u64) << 0;

    // Inflectional (bits 4–17)
    bf |= (features.gender as u64) << 4;
    bf |= (features.number as u64) << 6;
    bf |= (features.person as u64) << 8;
    bf |= (features.tense as u64) << 10;
    bf |= (features.mood as u64) << 12;
    bf |= (features.voice as u64) << 14;
    bf |= (features.case as u64) << 15;
    bf |= (features.state as u64) << 17;

    // Derivational (bits 18–39)
    bf |= (features.verb_form as u64) << 18;
    bf |= (features.noun_type as u64) << 23;
    bf |= (features.pronoun_type as u64) << 28;
    bf |= (features.transitivity as u64) << 32;
    bf |= (features.root_type as u64) << 36;

    // Prosodic (bits 40–46)
    bf |= (features.stress_pattern as u64) << 40;
    bf |= (features.syllable_count as u64) << 43;

    // Orthographic (bits 47–49)
    bf |= (features.has_shadda as u64) << 47;
    bf |= (features.has_madd as u64) << 48;
    bf |= (features.has_hamza as u64) << 49;

    // Reserved bits (50–63) are implicitly 0

    bf
}
```

### 8.4 Unpack Features (Reference Implementation)

```rust
/// Unpack a 64-bit bitfield into a complete feature set.
fn unpack_features(bf: u64) -> FeatureSet {
    FeatureSet {
        pos:             ((bf >> 0)  & 0xF) as u8,
        gender:          ((bf >> 4)  & 0x3) as u8,
        number:          ((bf >> 6)  & 0x3) as u8,
        person:          ((bf >> 8)  & 0x3) as u8,
        tense:           ((bf >> 10) & 0x3) as u8,
        mood:            ((bf >> 12) & 0x3) as u8,
        voice:           ((bf >> 14) & 0x1) as u8,
        case:            ((bf >> 15) & 0x3) as u8,
        state:           ((bf >> 17) & 0x1) as u8,
        verb_form:       ((bf >> 18) & 0x1F) as u8,
        noun_type:       ((bf >> 23) & 0x1F) as u8,
        pronoun_type:    ((bf >> 28) & 0xF) as u8,
        transitivity:    ((bf >> 32) & 0xF) as u8,
        root_type:       ((bf >> 36) & 0xF) as u8,
        stress_pattern:  ((bf >> 40) & 0x7) as u8,
        syllable_count:  ((bf >> 43) & 0xF) as u8,
        has_shadda:      ((bf >> 47) & 0x1) != 0,
        has_madd:        ((bf >> 48) & 0x1) != 0,
        has_hamza:       ((bf >> 49) & 0x1) != 0,
    }
}
```

### 8.5 Feature Value to Code Mapping Functions

```rust
/// Map feature name + value string to bitfield code.
fn feature_value_to_code(feature: &str, value: &str) -> Result<u8, FeatureError> {
    match feature {
        "pos" => match value {
            "verb"          => Ok(0),  "noun"        => Ok(1),
            "particle"      => Ok(2),  "pronoun"     => Ok(3),
            "adjective"     => Ok(4),  "adverb"      => Ok(5),
            "preposition"   => Ok(6),  "conjunction" => Ok(7),
            "proper_noun"   => Ok(8),  "interrogative" => Ok(9),
            _ => Err(FeatureError::InvalidValue),
        },
        "gender" => match value {
            "masculine" => Ok(0), "feminine" => Ok(1),
            "common"    => Ok(2), "unspecified" => Ok(3),
            _ => Err(FeatureError::InvalidValue),
        },
        "number" => match value {
            "singular" => Ok(0), "dual"   => Ok(1),
            "plural"   => Ok(2), "unspecified" => Ok(3),
            _ => Err(FeatureError::InvalidValue),
        },
        "person" => match value {
            "first"  => Ok(0), "second"  => Ok(1),
            "third"  => Ok(2), "unspecified" => Ok(3),
            _ => Err(FeatureError::InvalidValue),
        },
        "tense" => match value {
            "past"      => Ok(0), "present"    => Ok(1),
            "imperative" => Ok(2), "unspecified" => Ok(3),
            _ => Err(FeatureError::InvalidValue),
        },
        "mood" => match value {
            "indicative"   => Ok(0), "subjunctive" => Ok(1),
            "jussive"      => Ok(2), "energetic"   => Ok(3),
            _ => Err(FeatureError::InvalidValue),
        },
        // ... (remaining features follow the same pattern)
        _ => Err(FeatureError::UnknownFeature),
    }
}
```

### 8.6 Bitfield Example: كَتَبَ (kataba, "he wrote")

```rust
let mut bf = 0u64;

bf = pack_features(&FeatureSet {
    pos:             0,      // verb
    gender:          0,      // masculine
    number:          0,      // singular
    person:          2,      // third
    tense:           0,      // past
    mood:            3,      // unspecified (past has no mood)
    voice:           0,      // active
    case:            0,      // nominative (default, not used for verbs)
    state:           0,      // definite (default, not used for verbs)
    verb_form:       1,      // form I
    noun_type:       0,      // not_a_noun
    pronoun_type:    0,      // not_a_pronoun
    transitivity:    2,      // transitive_1
    root_type:       1,      // sound
    stress_pattern:  2,      // penultimate
    syllable_count:  3,      // 3 syllables
    has_shadda:      false,
    has_madd:        false,
    has_hamza:       false,
});

// Result: 0x0000_0000_001A_1203
// Binary: 0000 0000 0000 0000 0000 0000 0001 1010 0001 0010 0000 0011
```

---

## 9. Feature Usage by Pipeline Module

### 9.1 MOD-04 (MorphologicalParser)

| Operation | Features Involved | Details |
|-----------|------------------|---------|
| **Extract** | All 19 features | Primary origin of most features |
| **Pack** | All 19 | Pack into 64-bit bitfield |
| **Validate** | All (per POS applicability) | VAL-001 through VAL-030 |
| **Default** | All unspecified | INF-001 through INF-005 |
| **Infer** | Cross-feature | INF-010 through INF-015 |

**Pseudocode entry point:**
```rust
fn extract_features(token: &Token) -> FeatureSet {
    // 1. Determine POS
    let pos = determine_pos(&token);

    // 2. Extract inflectional features based on POS
    let (tense, mood, voice, person, number, gender) = match pos {
        PartOfSpeech::Verb => extract_verb_features(&token),
        PartOfSpeech::Noun | PartOfSpeech::Adjective
            | PartOfSpeech::ProperNoun => extract_noun_features(&token),
        PartOfSpeech::Pronoun => extract_pronoun_features(&token),
        _ => Default::default(), // particles, etc.
    };

    // 3. Extract derivational features
    let (verb_form, noun_type, pronoun_type, transitivity, root_type)
        = extract_derivational(&token, pos);

    // 4. Pass-through prosodic/orthographic from upstream
    let (stress, syllables, shadda, madd, hamza)
        = token.upstream_features;

    // 5. Build and return FeatureSet
    FeatureSet { pos, gender, number, person, tense, mood, voice,
                 case, state, verb_form, noun_type, pronoun_type,
                 transitivity, root_type, stress_pattern: stress,
                 syllable_count: syllables, has_shadda: shadda,
                 has_madd: madd, has_hamza: hamza }
}
```

### 9.2 MOD-05 (SyntaxParser)

| Operation | Features Involved | Details |
|-----------|------------------|---------|
| **Read** | pos, case, state, gender, number, person | Parse tree construction |
| **Check agreement** | gender, number, person | Subject-verb agreement |
| **Set role** | (none — assigned by parser) | Token role (fi'l, fa'il, etc.) |

### 9.3 MOD-07 (RuleEngine)

| Operation | Features Involved | Details |
|-----------|------------------|---------|
| **Query** | All (most commonly pos, case, mood, gender, number, state) | Rule condition evaluation |
| **Modify** | case, mood, state | Government-based reassignments |
| **Check** | gender, number, person | Agreement rule validation |
| **Flag violations** | All | Grammar error detection |

**Query patterns by category:**

```ebnf
// Agreement rules → gender, number, person
condition { fi'l.gender == fa'il.gender }
condition { fi'l.number == fa'il.number }

// Case rules → case
condition { token.role == "majrur" and token.case != "genitive" }

// Mood rules → tense, mood
condition { fi'l.tense == "present" and fi'l.mood == "jussive" }

// State rules → state
condition { token.role == "mubtada" and token.state == "indefinite" }
```

### 9.4 MOD-10 (GVM)

| Operation | Features Involved | Details |
|-----------|------------------|---------|
| **Extract** | All | `FEATURE_EXTRACT` instruction |
| **Compare** | All | `FEATURE_CMP` instruction |
| **Check** | All | Agreement checking within GVM |

### 9.5 MOD-11 (ExplanationEngine)

| Operation | Features Involved | Details |
|-----------|------------------|---------|
| **Display** | All | I'rab generation template variables |
| **Format** | All | Localized display (`{{gender}}` → "مذكر") |
| **Filter** | pos, educational_level | Content adaptation by level |

---

## 10. Feature Query Patterns in the Rule Engine DSL

### 10.1 Direct Feature Access

```ebnf
// Via shorthand (token-level):
token.gender
token.number
token.person

// Via features path:
token.features.gender
token.features.case
token.features.mood

// Via role shorthand (for specific roles):
fi'l.gender               // → resolves to token with role "fi'l", then gender
fa'il.number              // → resolves to token with role "fa'il", then number
mubtada'.state            // → resolves to token with role "mubtada", then state
maf'ul_bi-hi.case         // → resolves to token with role "maf'ul_bi-hi", then case

// Via sentence-level (for sentence type properties not in token features):
sentence.type             // → "jumlah_fi'liyyah", etc.
```

### 10.2 Comparison Operators

```ebnf
// Equality (value comparison):
token.features.gender == "masculine"
token.features.case == "accusative"
fi'l.mood == "jussive"

// Inequality:
token.features.gender != "feminine"
token.features.case != "nominative"

// Set membership:
token.features.pos in ["noun", "adjective", "proper_noun"]
token.features.gender in ["masculine", "common"]

// Pattern matching (regex):
token.text matches ".*ة$"          // Ends with ta-marbuta → likely feminine
token.features.state matches "def.*"

// Negation:
not (token.features.case == "nominative")
not (mood_agrees())
```

### 10.3 Cross-Token Comparison

```ebnf
// Subject-verb agreement:
fi'l.gender == fa'il.gender      // Gender agreement
fi'l.number == fa'il.number      // Number agreement
fi'l.person == fa'il.person      // Person agreement

// Noun-adjective agreement:
token.gender == preceding_noun.gender
token.number == preceding_noun.number
token.case   == preceding_noun.case
token.state  == preceding_noun.state

// Agreement with exceptions:
fi'l.number == fa'il.number
or (verb_before_subject() and fi'l.number == "singular")

// Non-human plural exception:
not (fa'il.gender == "masculine" and fa'il.semantic_type == "human")
and fi'l.gender == "feminine"
and fi'l.number == "singular"
```

### 10.4 Quantified Queries

```ebnf
// Check all tokens:
forall (t in sentence.tokens) {
    t.features.case != null
}

// Check existence:
exists (t in sentence.tokens) {
    t.features.pos == "verb"
}

// Check with conditions:
forall (t in sentence.tokens where t.role == "na'at") {
    t.features.gender == preceding(t).features.gender
}
```

### 10.5 Standard Library Query Predicates

```ebnf
// From agos-core.agosrule:
person_agrees()          // fi'l.person == fa'il.person
number_agrees()          // fi'l.number == fa'il.number
gender_agrees()          // fi'l.gender == fa'il.gender
is_transitive_verb()     // token.transitivity in [transitive_1, transitive_2, ditransitive]
is_intransitive_verb()   // token.transitivity == intransitive
is_passive()             // token.voice == passive
is_active()              // token.voice == active
is_definite()            // token.state == definite
is_indefinite()          // token.state == indefinite
is_nominative()          // token.case == nominative
is_accusative()          // token.case == accusative
is_genitive()            // token.case == genitive

// From agreement.agosrule:
verbal_agreement_complete()      // person, number, gender all match
non_human_plural_agreement()     // non-human plural → feminine singular verb
mubtada_khabar_agreement()       // topic-comment agreement
adjective_agreement_complete()   // 4-way agreement: gender, number, case, state
```

### 10.6 DSL-to-Feature Mapping Quick Reference

| DSL Expression | Feature | Values |
|----------------|---------|--------|
| `token.pos` | POS | `verb`, `noun`, `particle`, `pronoun`, `adjective`, `adverb`, `preposition`, `conjunction`, `proper_noun`, `interrogative` |
| `token.gender` | Gender | `masculine`, `feminine`, `common` |
| `token.number` | Number | `singular`, `dual`, `plural` |
| `token.person` | Person | `first`, `second`, `third` |
| `token.tense` | Tense | `past`, `present`, `imperative` |
| `token.mood` | Mood | `indicative`, `subjunctive`, `jussive`, `energetic` |
| `token.voice` | Voice | `active`, `passive` |
| `token.case` | Case | `nominative`, `accusative`, `genitive` |
| `token.state` | State | `definite`, `indefinite` |
| `token.verb_form` | Verb Form | `I`, `II`, `III`, `IV`, `V`, `VI`, `VII`, `VIII`, `IX`, `X`, `XI`, `XII`, `XIII`, `XIV`, `XV` |
| `token.noun_type` | Noun Type | `masdar`, `ism_fail`, `ism_maful`, `ism_makan`, `ism_zaman`, `ism_alah`, `sifah_mushabbahah`, `tafdil`, `nisbah`, `jam_taksir`, `ism_marrati`, `ism_hayati`, `jins`, `ism_tasghir` |
| `token.pronoun_type` | Pronoun Type | `personal_attached`, `personal_detached`, `demonstrative`, `relative`, `interrogative`, `conditional`, `compound` |
| `token.transitivity` | Transitivity | `intransitive`, `transitive_1`, `transitive_2`, `ditransitive` |
| `token.root_type` | Root Type | `sound`, `weak_initial`, `weak_middle`, `weak_final`, `hamzated`, `doubled`, `sound_quadriliteral`, `weak_quadriliteral` |

---

## 11. Validation Rules & Constraints Reference

### 11.1 Validation Rules (VAL-001 through VAL-030)

KB-0007 defines 15+ validation rules that MUST be enforced by MOD-04 after feature extraction and MAY be re-checked by MOD-07 or MOD-10.

| ID | Description | Severity | Enforced By |
|----|-------------|----------|-------------|
| VAL-001 | Every token must have a POS value | error | MOD-04 |
| VAL-002 | POS must be a valid value (0–9) | error | MOD-04 |
| VAL-003 | Tense only applies to verbs | warning | MOD-04 |
| VAL-004 | Mood only applies when tense=present | error | MOD-04, MOD-07 |
| VAL-005 | Verb form only applies to verbs | error | MOD-04 |
| VAL-006 | Noun type only applies to nouns/adjectives | warning | MOD-04 |
| VAL-007 | Pronoun type only applies to pronouns | error | MOD-04 |
| VAL-008 | Case only applies to nouns/adjectives/proper_nouns | warning | MOD-04 |
| VAL-009 | State only applies to nouns/adjectives/proper_nouns | warning | MOD-04 |
| VAL-010 | Transitivity only applies to verbs | warning | MOD-04 |
| VAL-011 | Root type only applies to verbs/nouns/adjectives/proper_nouns | warning | MOD-04 |
| VAL-020 | Syllable count must be 0–8 | error | MOD-04 |
| VAL-021 | Verb form must be 0–15 | error | MOD-04 |
| VAL-030 | Reserved bits (50–63) must be zero | error | MOD-04, MOD-10 |

### 11.2 Feature Applicability Constraints

```rust
/// Check if a feature value is applicable for the given POS.
/// Returns Ok(()) if valid, Err with message if invalid.
fn check_feature_applicability(pos: PartOfSpeech, feature: &str, value: u8)
    -> Result<(), ValidationError>
{
    match feature {
        "tense" if pos != PartOfSpeech::Verb && value != 3 => {
            Err(ValidationError::new(VAL_003, "Tense only applies to verbs"))
        }
        "mood" if pos != PartOfSpeech::Verb && value != 3 => {
            Err(ValidationError::new(VAL_004, "Mood only applies to verbs"))
        }
        "verb_form" if pos != PartOfSpeech::Verb && value != 0 => {
            Err(ValidationError::new(VAL_005, "Verb form only applies to verbs"))
        }
        "noun_type" if pos != PartOfSpeech::Noun
                       && pos != PartOfSpeech::Adjective && value != 0 => {
            Err(ValidationError::new(VAL_006, "Noun type only applies to nouns/adjectives"))
        }
        // ... (remaining features)
        _ => Ok(())
    }
}
```

### 11.3 Constraint Checking Functions

```rust
/// Validate a fully assembled FeatureSet.
fn validate_feature_set(fs: &FeatureSet) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // VAL-001: POS must be assigned
    if fs.pos > 9 {
        errors.push(ValidationError::new(VAL_002, "Invalid POS value"));
    }

    // VAL-004: Mood dependency on tense
    if fs.mood != 3 && fs.tense != 1 {  // mood is set but tense != present
        errors.push(ValidationError::new(VAL_004,
            "Mood only applies when tense=present"));
    }

    // VAL-030: Reserved bits check
    // (handled by pack_features which masks bits 50-63)

    // Mutual exclusion: verb_form and noun_type
    if fs.verb_form != 0 && fs.noun_type != 0 {
        errors.push(ValidationError::new(CON_010,
            "Verb form and noun type are mutually exclusive"));
    }

    errors
}
```

---

## 12. Feature Agreement Reference

### 12.1 Subject-Verb Agreement Rules

| Rule ID | Condition | Constraint | Exception |
|---------|-----------|------------|-----------|
| AGR-001 | Subject-verb | `verb.person == subject.person AND verb.number == subject.number AND verb.gender == subject.gender` | VSO: verb singular, agrees only in gender |
| AGR-002 | Noun-adjective | `adj.gender == noun.gender AND adj.case == noun.case AND adj.state == noun.state AND (noun.number != "plural" OR adj.number == noun.number)` | Non-human plurals → feminine singular |
| AGR-003 | Preposition government | `object.case == genitive` | None |
| AGR-004 | Inna & sisters | `subject.case == accusative` | None |
| AGR-005 | Subjunctive particles | `verb.mood == subjunctive` | None |
| AGR-006 | Jussive particles | `verb.mood == jussive` | None |

### 12.2 Agreement Check Implementation

```rust
/// Check subject-verb agreement.
/// Returns true if agreement holds, false if violated.
fn check_subject_verb_agreement(
    verb_features: &FeatureSet,
    subject_features: &FeatureSet,
) -> AgreementResult {
    // Basic agreement: person, number, gender must match
    let person_match = verb_features.person == subject_features.person;
    let number_match = verb_features.number == subject_features.number;
    let gender_match = verb_features.gender == subject_features.gender;

    if person_match && number_match && gender_match {
        return AgreementResult::Pass;
    }

    // Check VSO exception: verb before subject → verb is singular
    // (agrees only in gender)
    if verb_position < subject_position {
        if person_match && gender_match && verb_features.number == 0 /* singular */ {
            return AgreementResult::PassWithException("VSO order");
        }
    }

    // Collect specific violations
    let mut violations = Vec::new();
    if !person_match { violations.push("person"); }
    if !number_match { violations.push("number"); }
    if !gender_match { violations.push("gender"); }

    AgreementResult::Fail(violations)
}
```

### 12.3 Agreement Feature Constants

```rust
/// Pre-defined agreement constants for use in DSL and code.
mod agreement_constants {
    // Agreement dimensions checked by rules
    pub const AGREEMENT_PERSON: &str = "person";
    pub const AGREEMENT_NUMBER: &str = "number";
    pub const AGREEMENT_GENDER: &str = "gender";
    pub const AGREEMENT_CASE:   &str = "case";
    pub const AGREEMENT_STATE:  &str = "state";

    // Full agreement: all applicable dimensions
    pub const FULL_VERBAL_AGREEMENT: &[&str] = &["person", "number", "gender"];
    pub const FULL_NOMINAL_AGREEMENT: &[&str] = &["gender", "number"];
    pub const FULL_ADJECTIVAL_AGREEMENT: &[&str] = &["gender", "number", "case", "state"];
}
```

---

## 13. Cross-Feature Interaction Reference

### 13.1 Tense ↔ Mood Interaction

```rust
/// Determine valid mood values based on tense.
fn valid_moods_for_tense(tense: u8) -> &'static [u8] {
    match tense {
        0 /* past */       => &[3],              // only unspecified
        1 /* present */    => &[0, 1, 2, 3],     // indicative, subjunctive, jussive, energetic
        2 /* imperative */ => &[2, 3],           // jussive or unspecified
        _                  => &[3],              // default: unspecified
    }
}
```

### 13.2 Verb Form ↔ Transitivity Interaction

```rust
/// Default transitivity by verb form.
fn default_transitivity_for_form(form: u8) -> u8 {
    match form {
        1 /* I */   => 2,  // transitive_1 (default, varies by root)
        2 /* II */  => 2,  // transitive_1 (often makes intransitive roots transitive)
        3 /* III */ => 2,  // transitive_1
        4 /* IV */  => 2,  // transitive_1 (causative)
        5 /* V */   => 1,  // intransitive (reflexive)
        6 /* VI */  => 1,  // intransitive (reciprocal)
        7 /* VII */ => 1,  // intransitive (passive/reflexive)
        8 /* VIII */=> 2,  // transitive_1 (reflexive, but often transitive)
        9 /* IX */  => 1,  // intransitive (colors/defects)
        10 /* X */  => 2,  // transitive_1 (requestive)
        _           => 0,  // unspecified
    }
}
```

### 13.3 POS ↔ Derivational Feature Mutual Exclusions

```rust
/// Check mutual exclusion between derivational features.
fn check_derivational_exclusion(fs: &FeatureSet) -> bool {
    let mut has_derivational = 0;

    if fs.verb_form != 0    { has_derivational += 1; }
    if fs.noun_type != 0    { has_derivational += 1; }
    if fs.pronoun_type != 0 { has_derivational += 1; }

    has_derivational <= 1  // At most one derivational feature can be non-default
}
```

### 13.4 Case ↔ State ↔ POS Interaction

```rust
/// Determine valid case/state combinations by POS.
fn valid_case_state_combinations(pos: u8) -> &'static [(u8, u8)] {
    match pos {
        0 /* verb */    => &[(0, 0)],                // (nominative, definite) — unused
        1 /* noun */    => &[(0,0),(0,1),(1,0),(1,1),(2,0),(2,1)],  // all 6 combos
        4 /* adj */     => &[(0,0),(0,1),(1,0),(1,1),(2,0),(2,1)],  // all 6 combos
        8 /* prop_noun */=> &[(0,0),(1,0),(2,0)],     // always definite
        _               => &[(3, 0)],                 // (unspecified, definite)
    }
}
```

---

## 14. Plugin Extension Mechanism

### 14.1 Reserved Bit Allocation

Bits 50–63 (14 bits total) are reserved for plugin-defined custom features.

```rust
/// Register a custom feature for plugin use.
/// Returns the allocated bit range.
fn register_custom_feature(
    plugin_id: &str,
    feature_name: &str,
    bit_width: u8,
) -> Result<BitRange, RegistrationError> {
    // Check if plugin is authorized to register features
    if !plugin_has_capability(plugin_id, Capability::FeatureRegister) {
        return Err(RegistrationError::NotAuthorized);
    }

    // Find available bits in the reserved range (50–63)
    let available = find_available_bits(50, 14, bit_width)?;

    // Register in the global feature registry
    FEATURE_REGISTRY.register(CustomFeature {
        plugin_id: plugin_id.to_string(),
        feature_name: feature_name.to_string(),
        bit_start: available.start,
        bit_width,
        registered_at: now(),
    });

    Ok(available)
}
```

### 14.2 Custom Feature Constraints

| Constraint | Description |
|------------|-------------|
| Max total custom bits | 14 bits (50–63) across all plugins |
| Max per plugin | 8 bits per plugin |
| Bit overlap detection | Two plugins cannot use overlapping bit ranges |
| Registration | Must be registered before plugin initialization |
| Validation | Custom feature values must be in valid range (0 to 2^width - 1) |
| Conflict resolution | First-registered plugin wins; second is rejected with error |

### 14.3 Reserved Bit Validation

```rust
/// Validate that reserved bits (50–63) are either zero or plugin-registered.
fn validate_reserved_bits(
    bitfield: u64,
    registered_plugins: &[RegisteredFeature],
) -> Result<(), ValidationError> {
    let reserved_mask = 0x3FFF_0000_0000_0000; // bits 50–63
    let reserved_value = bitfield & reserved_mask;

    if reserved_value == 0 {
        return Ok(());  // Reserved bits are zero — valid
    }

    // Check if non-zero bits are plugin-registered
    for plugin in registered_plugins {
        let plugin_mask = plugin.bitmask();
        if reserved_value & plugin_mask == reserved_value {
            return Ok(());  // All non-zero bits belong to registered plugins
        }
    }

    Err(ValidationError::new(VAL_030,
        "Reserved bits (50–63) are non-zero and not plugin-registered"))
}
```

---

## 15. Performance & Memory Reference

### 15.1 Feature Operations Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Pack features (all 19) | < 500 ns | Bitwise operations, no branching |
| Unpack features (all 19) | < 300 ns | Bitwise operations |
| Single feature extract | < 50 ns | Mask + shift |
| Single feature modify | < 50 ns | Mask + set |
| Feature value to code lookup | < 200 ns | Hash map for string→code |
| Code to feature value lookup | < 100 ns | Array index |
| Agreement check (2 features) | < 1 μs | Comparison + exception check |
| Full feature validation | < 5 μs | 15+ checks |

### 15.2 Memory Layout

```rust
/// Compact in-memory representation of a feature set (32 bytes).
#[repr(C, packed)]
struct FeatureSetPacked {
    /// 64-bit bitfield (8 bytes)
    bitfield: u64,

    /// Per-feature confidence scores packed into 64 bits (4 bits × 16 features)
    confidences: u64,

    /// Reserved for future use
    _reserved: [u8; 16],
}

/// Expanded feature set for DSL/API use (40 bytes).
struct FeatureSet {
    pos:             u8,   // 1 byte (+ 3 padding)
    gender:          u8,
    number:          u8,
    person:          u8,
    tense:           u8,
    mood:            u8,
    voice:           u8,
    case:            u8,
    state:           u8,
    verb_form:       u8,
    noun_type:       u8,
    pronoun_type:    u8,
    transitivity:    u8,
    root_type:       u8,
    stress_pattern:  u8,
    syllable_count:  u8,   // = 16 bytes
    has_shadda:      bool, // = 1 byte
    has_madd:        bool,
    has_hamza:       bool, // = 3 bytes
    _padding:        [u8; 13], // alignment padding
    // Total: 32 bytes with alignment
}
```

### 15.3 Size Budget by Module

| Module | Feature Memory | Notes |
|--------|---------------|-------|
| MOD-04 | ~60 MB | KB-0007 loaded + working memory |
| MOD-05 | ~5 MB | Per-sentence feature tracking |
| MOD-07 | ~10 MB | Feature indices for ~1,000 rules |
| MOD-10 | ~1 MB | GVM feature map |
| MOD-11 | ~5 MB | Template feature mappings |

---

## 16. Testing & Quality Reference

### 16.1 Test Categories

| Category | Description | Coverage |
|----------|-------------|----------|
| **Pack/unpack roundtrip** | Verify pack_features(unpack_features(bf)) == bf | 100% of valid combinations |
| **POS applicability** | Each feature maps to correct POS types | 100% |
| **Feature range validation** | All values within valid ranges | 100% |
| **Boundary values** | Min/max values for each feature | 100% |
| **Reserved bits** | Bits 50–63 remain zero after pack/unpack | 100% |
| **Agreement rules** | All 6 agreement rules pass test fixtures | 100% |
| **Mutual exclusion** | verb_form/noun_type/pronoun_type are mutually exclusive | 100% |
| **Tense↔mood dependency** | Mood is only valid when tense=present | 100% |
| **Cross-feature constraints** | CON-001 through CON-022 | 100% |

### 16.2 Test Fixture Format

```jsonc
{
    "spec": "SPEC-0102/test-fixture",
    "version": "1.0.0",
    "name": "feature-pack-unpack-verb-past",
    "description": "Verify pack/unpack for past tense verb features",

    "input_features": {
        "pos":             "verb",
        "gender":          "masculine",
        "number":          "singular",
        "person":          "third",
        "tense":           "past",
        "mood":            "unspecified",
        "voice":           "active",
        "case":            "nominative",     // default, not used for verbs
        "state":           "definite",       // default, not used for verbs
        "verb_form":       "I",
        "noun_type":       "not_a_noun",
        "pronoun_type":    "not_a_pronoun",
        "transitivity":    "transitive_1",
        "root_type":       "sound",
        "stress_pattern":  "unspecified",
        "syllable_count":  0,
        "has_shadda":      false,
        "has_madd":        false,
        "has_hamza":       false
    },

    "expected_bitfield": "0x0000_0000_001A_1203",

    "checks": [
        { "check": "pack_unpack_roundtrip" },
        { "check": "reserved_bits_zero" },
        { "check": "pos_applicability" },
        { "check": "tense_mood_dependency" }
    ]
}
```

### 16.3 Conformance Test Examples

```rust
#[test]
fn test_pack_unpack_roundtrip() {
    let original = FeatureSet {
        pos: 0, gender: 0, number: 0, person: 2,  // verb, masc, sg, 3rd
        tense: 0, mood: 3, voice: 0,               // past, no mood, active
        case: 0, state: 0,                          // nom, def (unused for verbs)
        verb_form: 1, noun_type: 0, pronoun_type: 0, // form I
        transitivity: 2, root_type: 1,              // transitive, sound
        stress_pattern: 2, syllable_count: 3,
        has_shadda: false, has_madd: false, has_hamza: false,
    };

    let bf = pack_features(&original);
    let unpacked = unpack_features(bf);

    assert_eq!(original.pos, unpacked.pos);
    assert_eq!(original.gender, unpacked.gender);
    assert_eq!(original.number, unpacked.number);
    assert_eq!(original.tense, unpacked.tense);
    assert_eq!(original.mood, unpacked.mood);
    assert_eq!(original.verb_form, unpacked.verb_form);
    assert_eq!(original.transitivity, unpacked.transitivity);
    // ... (assert all features match)
}

#[test]
fn test_reserved_bits_zero() {
    let bf = pack_features(&FeatureSet::default());
    let reserved = (bf >> 50) & 0x3FFF;
    assert_eq!(reserved, 0, "Reserved bits 50–63 must be zero");
}

#[test]
fn test_tense_mood_dependency() {
    // Mood cannot be set when tense is past
    let fs = FeatureSet {
        pos: 0, tense: 0 /* past */, mood: 0 /* indicative */,
        ..Default::default()
    };
    let errors = validate_feature_set(&fs);
    assert!(errors.iter().any(|e| e.code == "VAL-004"));
}

#[test]
fn test_present_tense_valid_moods() {
    // Present tense can have any mood
    for mood in 0..=3 {
        let fs = FeatureSet {
            pos: 0, tense: 1 /* present */, mood,
            ..Default::default()
        };
        let errors = validate_feature_set(&fs);
        assert!(errors.is_empty(), "Mood {} should be valid for present tense", mood);
    }
}
```

---

## 17. Cross-References

### 17.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 §4 | Module Catalog | Module feature responsibilities |
| SPEC-0001-C3 §4 | MOD-04 Pipeline | Feature extraction step |
| SPEC-0001-C3 §5 | MOD-05 Pipeline | Agreement checking step |
| SPEC-0001-C3 §8 | MOD-07 Pipeline | Rule evaluation using features |
| SPEC-0001-C4 §5 | MOD-04 Interface | Feature-related API methods |
| SPEC-0001-C5 §4 | IR-4 MorphologicalAnalysis | Feature field in IR |
| SPEC-0001-C5 §8 | IR-7 AnnotatedGIR | Feature modifications in rules |
| SPEC-0001-C9 §4.2 | KB-0007 Size Budget | Feature compilation size |
| SPEC-0101 §6 | MOD-04 Feature Extraction | How features are extracted |
| SPEC-0101 §11 | MOD-05 Agreement | How agreement uses features |
| SPEC-0201 §5 | MOD-07 Rule Execution | How rules query/modify features |
| SPEC-0301 §5.2 | GVM Feature Instructions | how GVM processes features |
| SPEC-0501 §5 | I'rab Generation Templates | How features are displayed |
| RFC-0002 §Feature Bitfield | Bytecode Format | 64-bit bitfield encoding |
| RFC-0003 §Feature Instructions | GVM Instructions | Feature extraction in GVM |
| RFC-0004 §5 | Arabic Terminology Mapping | Arabic terms for features |

### 17.2 Knowledge Base References

| KB | Title | Relationship |
|----|-------|--------------|
| KB-0001 | Roots Database | root_type, transitivity features |
| KB-0002 | Wazan Database | verb_form, noun_type features |
| KB-0003 | Verb Forms | tense, mood, voice, person, number, gender |
| KB-0004 | Noun Patterns | noun_type, case, state, gender |
| KB-0005 | Particles | pos (particle), grammatical function |
| KB-0006 | Pronouns | pos (pronoun), pronoun_type |
| KB-0007 | Morphological Features | Authoritative source for all 19 features |

### 17.3 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab | Foundational Arabic grammar feature system |
| Wright's Arabic Grammar | Western reference for Arabic features |
| Ryding, A Reference Grammar of MSA | Contemporary MSA feature usage |
| Unicode Standard, Arabic Block (U+0600–U+06FF) | Orthographic feature reference |

---

## Progress Summary

**SPEC-0102: Morphological Features — Reference Taxonomy**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Feature Taxonomy Overview | ✓ COMPLETE |
| 3 | Part of Speech (POS) Reference | ✓ COMPLETE |
| 4 | Inflectional Features Reference | ✓ COMPLETE |
| 5 | Derivational Features Reference | ✓ COMPLETE |
| 6 | Prosodic Features Reference | ✓ COMPLETE |
| 7 | Orthographic Features Reference | ✓ COMPLETE |
| 8 | 64-Bit Bitfield Reference | ✓ COMPLETE |
| 9 | Feature Usage by Pipeline Module | ✓ COMPLETE |
| 10 | Feature Query Patterns in the Rule Engine DSL | ✓ COMPLETE |
| 11 | Validation Rules & Constraints Reference | ✓ COMPLETE |
| 12 | Feature Agreement Reference | ✓ COMPLETE |
| 13 | Cross-Feature Interaction Reference | ✓ COMPLETE |
| 14 | Plugin Extension Mechanism | ✓ COMPLETE |
| 15 | Performance & Memory Reference | ✓ COMPLETE |
| 16 | Testing & Quality Reference | ✓ COMPLETE |
| 17 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–9), SPEC-0101, SPEC-0201, SPEC-0301, SPEC-0501, RFC-0002, RFC-0003, RFC-0004, KB-0007, KB-0001–0006.

**Recommended next step:** SPEC-0302 (GVM Instruction Set) — the instruction-level reference for the Grammar Virtual Machine, building on the feature bitfield operations defined here.

---

*End of SPEC-0102*
