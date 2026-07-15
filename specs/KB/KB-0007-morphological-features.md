---
kb_id: KB-0007
title: Morphological Features — Taxonomy, Bitfield Encoding & Inference Rules
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0007)
  - SPEC-0001-C3: Compilation Pipeline (MOD-04 — Feature Extraction Step 1.7)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-4 — Feature Taxonomy)
  - SPEC-0001-C8: Security, Validation & Error Handling (Feature Validation)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - RFC-0001: Grammar DSL (Feature Names)
  - RFC-0002: Grammar Bytecode Format (Feature Bitfield — 64-bit)
  - RFC-0003: Grammar Virtual Machine (Feature Extraction Instruction)
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
---

# KB-0007: Morphological Features — Taxonomy, Bitfield Encoding & Inference Rules

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Morphological Features in Arabic](#2-morphological-features-in-arabic)
3. [Data Model](#3-data-model)
4. [Feature Entry Schema](#4-feature-entry-schema)
5. [Part of Speech (POS)](#5-part-of-speech-pos)
6. [Inflectional Features](#6-inflectional-features)
7. [Derivational Features](#7-derivational-features)
8. [Prosodic Features](#8-prosodic-features)
9. [Orthographic Features](#9-orthographic-features)
10. [Feature Bitfield Encoding (RFC-0002)](#10-feature-bitfield-encoding-rfc-0002)
11. [Feature Agreement Rules](#11-feature-agreement-rules)
12. [Feature Inference Rules](#12-feature-inference-rules)
13. [Feature Validation](#13-feature-validation)
14. [Cross-Feature Constraints](#14-cross-feature-constraints)
15. [Feature Matching Algorithm](#15-feature-matching-algorithm)
16. [Serialization & Storage](#16-serialization--storage)
17. [Versioning & Evolution](#17-versioning--evolution)
18. [Quality Requirements](#18-quality-requirements)
19. [Example Entries](#19-example-entries)
20. [Cross-References](#20-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0007 is the **authoritative feature taxonomy** for the AGOS platform. It defines:

1. **The complete set of morphological features** recognized by AGOS — their names, allowed values, and semantics.
2. **The bitfield encoding** (64-bit) used to pack features into the compact representation defined by RFC-0002.
3. **Agreement, inference, and validation rules** that govern how features interact across tokens, modules, and knowledge bases.

KB-0007 answers: **"What features does this token have? Which values are valid for each feature? How do features constrain each other? How are features packed into the 64-bit bytecode format?"**

### 1.2 Scope

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Inflectional features** | Gender, number, person, tense, mood, voice, case, state, aspect | Phonological features (covered by MOD-02) |
| **Derivational features** | Verb form (I–XV), noun type, pronoun type, transitivity, root type | — |
| **Prosodic features** | Stress pattern, syllable count | Metrical/prosodic analysis (covered by MOD-07) |
| **Orthographic features** | Shadda, madd, hamza presence | Orthographic normalization (covered by MOD-02) |
| **Part of speech** | 10 POS types (verb, noun, particle, pronoun, adjective, adverb, preposition, conjunction, proper_noun, interrogative) | Sub-categorization (covered by KB-0005, KB-0006, MOD-05) |
| **Bitfield encoding** | 64-bit packed feature representation per RFC-0002 | Custom/plugin extensions (bits 48–63) |
| **Agreement rules** | Arabic agreement constraints (gender, number, person, case, state) | Semantic agreement (covered by MOD-07) |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal feature systems |

### 1.3 Feature Categories Overview

```yaml
Feature Taxonomy (KB-0007)
├── Part of Speech (1 feature, 10 values)
│   └── pos
├── Inflectional (8 features, 34 values total)
│   ├── gender: masculine | feminine | common | unspecified
│   ├── number: singular | dual | plural | unspecified
│   ├── person: first | second | third | unspecified
│   ├── tense: past | present | imperative | unspecified
│   ├── mood: indicative | subjunctive | jussive | energetic | unspecified
│   ├── voice: active | passive
│   ├── case: nominative | accusative | genitive | unspecified
│   └── state: definite | indefinite
├── Derivational (5 features, 47 values total)
│   ├── verb_form: I | II | III | IV | V | VI | VII | VIII | IX | X | XI | XII | XIII | XIV | XV | not_a_verb
│   ├── noun_type: masdar | ism_fail | ism_maful | ism_makan | ism_zaman | ism_alah | sifah_mushabbahah | tafdil | nisbah | jam_taksir | ism_marrati | ism_hayati | jins | ism_tasghir | not_a_noun
│   ├── pronoun_type: personal_attached | personal_detached | demonstrative | relative | interrogative | conditional | compound | not_a_pronoun
│   ├── transitivity: intransitive | transitive_1 | transitive_2 | ditransitive | unspecified
│   └── root_type: sound | weak_initial | weak_middle | weak_final | hamzated | doubled | sound_quadriliteral | weak_quadriliteral | unspecified
├── Prosodic (2 features, ~10 values total)
│   ├── stress_pattern: final | penultimate | antepenultimate | unspecified
│   └── syllable_count: integer (1–8)
└── Orthographic (3 features, 8 values total)
    ├── has_shadda: true | false
    ├── has_madd: true | false
    └── has_hamza: true | false
```

### 1.4 Role in the AGOS Pipeline

```diff
  MOD-03: Preprocessor (Clitic Stripping)
    │
    ▼
  MOD-04: MorphologicalParser
    │
    ├── Step 1.7: Load KB-0007 (Feature Taxonomy)   ◄── THIS KB
    │
    ├── Steps 3–5: Parse & extract morphological features
    │   ├── Features inferred from KB-0001 (root properties)
    │   ├── Features inferred from KB-0002 (wazan patterns)
    │   ├── Features inferred from KB-0003/4 (conjugation/patterns)
    │   ├── Features from KB-0005/6 (particles/pronouns)
    │   └── Features validated against KB-0007 taxonomy
    │
    ├── Step 6: Pack features into 64-bit bitfield (RFC-0002)
    │
    ▼
  MOD-05: SyntacticParser
    │
    └── Agreement checking using KB-0007 feature relationships
```

### 1.5 Relationship to Other KBs

```diff
  KB-0007: Morphological Features         ◄── This document (Feature Taxonomy)
    │
    ├── Referenced by ALL other KBs (0001–0006)
    ├── Defines valid feature values used across the AGOS pipeline
    ├── Maps features to 64-bit bitfield positions (RFC-0002)
    │
    ├──► KB-0001: Roots                   — root_type, transitivity features
    ├──► KB-0002: Wazan                  — verb_form, noun_type features
    ├──► KB-0003: Verb Forms             — tense, mood, voice, person, number, gender
    ├──► KB-0004: Noun Patterns          — noun_type, case, state, gender, number
    ├──► KB-0005: Particles              — pos (particle), grammatical function
    └──► KB-0006: Pronouns               — pos (pronoun), pronoun_type, person, number, gender
```

### 1.6 Document Conventions

- **Feature Names:** Lowercase with underscores (e.g., `verb_form`, `root_type`).
- **Feature Values:** Lowercase for values (e.g., `masculine`, `indicative`, `passive`).
- **Bitfield Positions:** Zero-indexed from the least significant bit.
- **RFC-0002 Alignment:** All feature positions and values align with RFC-0002 §Feature Bitfield.

---

## 2. Morphological Features in Arabic

### 2.1 Definition

A **morphological feature** is a discrete linguistic property that characterizes a word form. Arabic, as a highly inflected Semitic language, encodes multiple features simultaneously in a single word through root-pattern morphology, prefix-suffix inflection, and stem-internal changes.

### 2.2 Feature Types

| Type | Description | Persistence | Example |
|------|-------------|-------------|---------|
| **Inherent** | Intrinsic property of a lexical item | Fixed across all forms | root_type for a given root |
| **Inflectional** | Grammatical property marked by inflection | Varies by context | gender, number, case |
| **Derivational** | Property of the derivational pattern | Fixed per derived form | verb_form (II, III, etc.) |
| **Prosodic** | Stress and syllable properties of a form | Varies by surface form | stress_pattern, syllable_count |
| **Orthographic** | Orthographic features of the written form | Varies by surface form | has_shadda, has_hamza |

### 2.3 Feature Interaction in Arabic

Arabic features interact in complex ways:

1. **Agreement:** A verb must agree with its subject in person, number, and gender.
2. **Government:** A preposition governs the genitive case on its object.
3. **Mood:** Conditional particles govern the jussive mood on the verb.
4. **State:** Definiteness spreads from a definite noun to its adjective modifier.

KB-0007 defines the **feature values** used in these interactions. The actual agreement rules are implemented by MOD-05 (SyntacticParser).

### 2.4 Feature Count Target

| Category | Feature Count | Value Count | Notes |
|----------|--------------|-------------|-------|
| Part of Speech | 1 | 10 | Enumerated 4-bit value (0–15) |
| Inflectional | 8 | 34 | 2-bit or 1-bit per feature |
| Derivational | 5 | 47 | 4-bit or 5-bit per feature |
| Prosodic | 2 | ~10 | Integer + enumeration |
| Orthographic | 3 | 6 | 3 × boolean (1-bit each) |
| **Total** | **19** | **~107** | Packed into 64-bit bitfield |

---

## 3. Data Model

### 3.1 Logical Data Model

```yaml
Morphological Features Database (KB-0007)
├── Metadata
│   ├── kb_id: "KB-0007"
│   ├── version: "1.0.0"
│   ├── feature_count: integer
│   ├── bitfield_width: 64
│   ├── created_at: timestamp
│   └── checksum_sha256: string
│
├── FeatureDefinitions: FeatureEntry[]
│   ├── pos (part of speech)
│   ├── gender, number, person, tense, mood, voice, case, state
│   ├── verb_form, noun_type, pronoun_type, transitivity, root_type
│   ├── stress_pattern, syllable_count
│   └── has_shadda, has_madd, has_hamza
│
├── AgreementRules: AgreementRule[]
│   ├── Subject-verb agreement constraints
│   ├── Noun-adjective agreement constraints
│   ├── Preposition-case government rules
│   └── Mood-government rules
│
├── InferenceRules: InferenceRule[]
│   ├── Feature inference from POS
│   ├── Feature inference from verb_form
│   ├── Feature inference from noun_type
│   └── Cross-feature defaults
│
└── Constraints: FeatureConstraint[]
    ├── Valid feature combinations
    ├── Mutually exclusive features
    └── Dependency constraints
```

### 3.2 Storage Model

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~0.5–1 MB | Human-readable |
| **Compiled (Bitfield Map + Rule Table)** | Production pipeline | ~1–2 MB | O(1) feature lookup + O(log n) rule matching |

### 3.3 Feature Graph Model

Features form a directed acyclic graph (DAG) where edges represent dependency/constraint relationships:

```diff
  POS (root node)
  │
  ├──→ verb
  │     ├──→ verb_form (I–XV)          ──→ transitivity
  │     ├──→ tense                      ──→ mood
  │     ├──→ voice
  │     └──→ person · number · gender   (agreement features)
  │
  ├──→ noun / adjective
  │     ├──→ noun_type
  │     ├──→ case · state
  │     └──→ gender · number
  │
  ├──→ particle
  │     └──→ (grammatical function from KB-0005)
  │
  └──→ pronoun
        └──→ pronoun_type
              └──→ person · number · gender
```

---

## 4. Feature Entry Schema

### 4.1 Schema Definition

```yaml
FeatureEntry:
  # --- Identity ---
  id: string                           # "KB-0007:{feature_name}"
                                       # e.g., "KB-0007:gender"
  feature_name: string                 # Canonical name (e.g., "gender")

  # --- Classification ---
  category: FeatureCategory            # "pos" | "inflectional" | "derivational" |
                                       # "prosodic" | "orthographic"
  bitfield_position: integer           # Starting bit position in the 64-bit field
  bitfield_width: integer              # Number of bits used (1, 2, 3, 4, or 5)

  # --- Allowed Values ---
  values: FeatureValue[]               # All valid values for this feature
  default_value: string | null         # Default when feature is unspecified

  # --- Semantics ---
  description: string                  # Description of what this feature encodes
  linguistic_relevance: string         # Why this feature matters for Arabic

  # --- Constraints ---
  applies_to_pos: string[]             # Which POS types this feature applies to
  depends_on: string | null            # Parent/conditional feature (e.g., mood depends on tense)
  mutually_exclusive_with: string[]    # Features that cannot co-occur
  requires_feature: string[]           # Features that must also be present

  # --- Encoding ---
  encoding: ValueEncoding[]            # Bit encoding for each value

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string

FeatureValue:
  name: string                         # Value name (e.g., "masculine")
  code: integer                        # Numeric code in bitfield
  label: string                        # Human-readable label
  description: string | null           # Description of this value
  arabic_term: string | null           # Arabic grammatical term (if applicable)

FeatureCategory:
  "pos" | "inflectional" | "derivational" | "prosodic" | "orthographic"

ValueEncoding:
  value_name: string                   # Reference to FeatureValue.name
  binary_code: integer                 # Bit pattern (0, 1, 2, ... 2^width - 1)
  notes: string | null

AgreementRule:
  id: string
  rule_type: "subject_verb" | "noun_adjective" | "government" | "mood_government"
  description: string
  source_feature: string
  target_feature: string
  constraint: string                   # Description of the agreement constraint
  exceptions: string | null
  applies_to: string[]                 # POS types this rule applies to

InferenceRule:
  id: string
  input_feature: string
  input_value: string
  inferred_feature: string
  inferred_value: string
  priority: integer                    # Higher priority = override later inference
  condition: string | null             # Optional condition
  notes: string | null

FeatureConstraint:
  id: string
  constraint_type: "valid_combination" | "mutual_exclusion" | "dependency" | "conditional_required"
  description: string
  features: string[]                   # Features involved in this constraint
  condition: string | null             # When this constraint applies
  allowed_combinations: string[][] | null  # For valid_combination type
```

### 4.2 JSON Example (Gender Feature)

```json
{
  "id": "KB-0007:gender",
  "feature_name": "gender",
  "category": "inflectional",
  "bitfield_position": 4,
  "bitfield_width": 2,
  "values": [
    { "name": "masculine", "code": 0, "label": "Masculine",
      "description": "Masculine gender", "arabic_term": "مذكر" },
    { "name": "feminine", "code": 1, "label": "Feminine",
      "description": "Feminine gender", "arabic_term": "مؤنث" },
    { "name": "common", "code": 2, "label": "Common",
      "description": "Common gender (same form for both)", "arabic_term": "مشترك" },
    { "name": "unspecified", "code": 3, "label": "Unspecified",
      "description": "Gender not applicable or unknown" }
  ],
  "default_value": "unspecified",
  "description": "Grammatical gender of a noun, pronoun, or verb agreement",
  "linguistic_relevance": "Arabic has two grammatical genders (masculine/feminine) with agreement in verbs, adjectives, and pronouns",
  "applies_to_pos": ["verb", "noun", "adjective", "pronoun", "proper_noun"],
  "depends_on": null,
  "mutually_exclusive_with": [],
  "requires_feature": [],
  "encoding": [
    { "value_name": "masculine", "binary_code": 0, "notes": "Default for most nouns" },
    { "value_name": "feminine", "binary_code": 1, "notes": "Often marked by ة suffix" },
    { "value_name": "common", "binary_code": 2, "notes": "1st person pronouns, some nouns" },
    { "value_name": "unspecified", "binary_code": 3, "notes": "Fallback value" }
  ],
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

---

## 5. Part of Speech (POS)

### 5.1 POS Definition

The **part of speech** is the root feature that determines which other features apply to a token. Every analyzed token MUST have a POS value.

### 5.2 POS Table

| Code | Name | Arabic | Applies To | Parent KB |
|------|------|--------|------------|-----------|
| 0 | `verb` | فعل | All verb features | KB-0001, KB-0002, KB-0003 |
| 1 | `noun` | اسم | All noun features | KB-0001, KB-0002, KB-0004 |
| 2 | `particle` | حرف | Minimal features | KB-0005 |
| 3 | `pronoun` | ضمير | Pronoun features | KB-0006 |
| 4 | `adjective` | صفة | Noun-like features | KB-0004 |
| 5 | `adverb` | ظرف | Minimal features | — |
| 6 | `preposition` | حرف جر | Government features | KB-0005 |
| 7 | `conjunction` | حرف عطف | Minimal features | KB-0005 |
| 8 | `proper_noun` | اسم علم | Noun-like features | KB-0001 |
| 9 | `interrogative` | اسم استفهام | Variable features | KB-0005, KB-0006 |
| 10–15 | *reserved* | — | — | — |

### 5.3 POS Bitfield Encoding

```text
Bits 0–3: pos (4 bits)
  0  → verb
  1  → noun
  2  → particle
  3  → pronoun
  4  → adjective
  5  → adverb
  6  → preposition
  7  → conjunction
  8  → proper_noun
  9  → interrogative
  10 → reserved (must be 0)
  ...
  15 → reserved (must be 0)
```

### 5.4 Feature Applicability by POS

| Feature | verb | noun | adj | pronoun | particle | adv | prep | conj | prop_noun | interr |
|---------|------|------|-----|---------|----------|-----|------|------|-----------|--------|
| gender | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | — |
| number | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | — |
| person | ✓ | — | — | ✓ | — | — | — | — | — | — |
| tense | ✓ | — | — | — | — | — | — | — | — | — |
| mood | ✓ | — | — | — | — | — | — | — | — | — |
| voice | ✓ | — | — | — | — | — | — | — | — | — |
| case | — | ✓ | ✓ | — | — | ✓ | — | — | ✓ | — |
| state | — | ✓ | ✓ | — | — | — | — | — | ✓ | — |
| verb_form | ✓ | — | — | — | — | — | — | — | — | — |
| noun_type | — | ✓ | ✓ | — | — | — | — | — | — | — |
| pronoun_type | — | — | — | ✓ | — | — | — | — | — | — |
| transitivity | ✓ | — | — | — | — | — | — | — | — | — |
| root_type | ✓ | ✓ | ✓ | — | — | — | — | — | ✓ | — |
| stress_pattern | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| syllable_count | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| has_shadda | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| has_madd | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | — |
| has_hamza | ✓ | ✓ | ✓ | — | ✓ | — | — | — | ✓ | ✓ |

---

## 6. Inflectional Features

### 6.1 Gender

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `masculine` | مذكر | Masculine gender (default for most nouns) | Bits 4–5 (2 bits) |
| 1 | `feminine` | مؤنث | Feminine gender (often marked by ة suffix) | |
| 2 | `common` | مشترك | Same form for both genders (e.g., 1st person) | |
| 3 | `unspecified` | — | Not applicable or unknown | |

**Applicable POS:** verb, noun, adjective, pronoun, proper_noun.

**Arabic Notes:**
- Feminine nouns are often marked by the suffix ة (tāʾ marbūṭa), but many feminine nouns have no overt marker (e.g., شَمْس `sun`).
- Masculine is the default gender for nouns without feminine marking.
- Common gender applies to 1st person pronouns and some invariable nouns.
- Verbs agree with their subject in gender (3rd person singular/dual/plural).

### 6.2 Number

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `singular` | مفرد | One entity | Bits 6–7 (2 bits) |
| 1 | `dual` | مثنى | Two entities | |
| 2 | `plural` | جمع | Three or more entities | |
| 3 | `unspecified` | — | Not applicable or unknown | |

**Applicable POS:** verb, noun, adjective, pronoun, proper_noun.

**Arabic Notes:**
- Dual is a distinct category in Arabic (not just "plural of two").
- Broken plurals (جمع تكسير) still have the feature value `plural`, with `noun_type = jam_taksir`.
- Verbs in 3rd person dual/feminine plural have special forms.

### 6.3 Person

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `first` | متكلم | Speaker | Bits 8–9 (2 bits) |
| 1 | `second` | مخاطب | Addressee | |
| 2 | `third` | غائب | Referent not speaker/addressee | |
| 3 | `unspecified` | — | Not applicable | |

**Applicable POS:** verb, pronoun.

**Arabic Notes:**
- 1st person singular: أَفْعَلُ/فَعَلْتُ (imperfect/perfect)
- 2nd person distinguishes masculine and feminine in singular/plural
- 3rd person distinguishes masculine and feminine in singular/dual/plural
- The "person of separation" (ضمير الفصل) is always 3rd person.

### 6.4 Tense

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `past` | ماض | Completed action (perfect) | Bits 10–11 (2 bits) |
| 1 | `present` | مضارع | Ongoing/future action (imperfect) | |
| 2 | `imperative` | أمر | Command | |
| 3 | `unspecified` | — | Not applicable | |

**Applicable POS:** verb.

**Arabic Notes:**
- Arabic has two base tenses: past (perfect) and present (imperfect).
- The imperative is derived from the present tense jussive form.
- Future can be expressed by present tense with prefix سَـ or word سَوْفَ.
- The perfect does NOT indicate mood (it has no subjunctive/jussive distinction).
- The imperfect DOES indicate mood (indicative vs. subjunctive vs. jussive).

### 6.5 Mood

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `indicative` | مرفوع | Default mood (present tense) | Bits 12–13 (2 bits) |
| 1 | `subjunctive` | منصوب | After subjunctive particles (أَنْ, لَنْ, etc.) | |
| 2 | `jussive` | مجزوم | After jussive particles (لَمْ, etc.) | |
| 3 | `energetic` | مؤكد | Emphatic/energetic mood (نَ التوكيد) | |

**Applicable POS:** verb.

**Dependency:** Only applies when `tense = present`. Mood is not encoded for past or imperative verbs.

| Tense | Mood Allowed | Notes |
|-------|-------------|-------|
| past | `unspecified` (3) | Past tense has no mood distinction |
| present | `indicative`, `subjunctive`, `jussive`, `energetic` | All four moods possible |
| imperative | `unspecified` (3) or `jussive` | Imperative derived from jussive |

### 6.6 Voice

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `active` | مبني للمعلوم | Subject performs the action | Bit 14 (1 bit) |
| 1 | `passive` | مبني للمجهول | Subject receives the action | |

**Applicable POS:** verb.

**Arabic Notes:**
- Passive is formed by changing internal vowels (ضُرِبَ vs. ضَرَبَ).
- Passive only applies to transitive verbs (but can be formed from intransitive in some cases).
- In passive constructions, the agent is omitted; the object takes nominative case.

### 6.7 Case

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `nominative` | مرفوع | Subject, predicate of nominal sentence | Bits 15–16 (2 bits) |
| 1 | `accusative` | منصوب | Object, after inna and sisters, circumstantial accusative | |
| 2 | `genitive` | مجرور | After prepositions, construct state possessor | |
| 3 | `unspecified` | — | Not applicable or invariable word | |

**Applicable POS:** noun, adjective, proper_noun, adverb (some).

**Arabic Notes:**
- Case is marked on nouns, adjectives, and some adverbs (ظروف).
- Triptote nouns have all three cases with distinct endings (-u, -a, -i).
- Diptote nouns have only two case endings (nominative -u, oblique -a).
- Invariable nouns (مبني) have case value `unspecified`.

### 6.8 State

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `definite` | معرفة | Definite (with الـ, proper noun, or construct) | Bit 17 (1 bit) |
| 1 | `indefinite` | نكرة | Indefinite (without الـ, with nunation) | |

**Applicable POS:** noun, adjective, proper_noun.

**Bit-width note:** State is 1 bit (values 0 or 1 only). There is no `unspecified` code — state is simply not set (bit = 0, `definite`) for POS types where state does not apply. Bits for inapplicable features MUST be set to 0 to avoid leaking into adjacent bitfields.

**Arabic Notes:**
- Definiteness is marked by the prefix الـ (al-) or by being a proper noun or by being in the construct state with a definite possessor.
- Indefinite nouns typically have nunation (تنوين) on the final vowel.
- The nominal sentence's subject is typically definite, and the predicate is indefinite.

---

## 7. Derivational Features

### 7.1 Verb Form

| Code | Value | Arabic | Example | Meaning Domain | Bit Position |
|------|-------|--------|---------|----------------|--------------|
| 0 | `not_a_verb` | ليس فعلاً | — | Default for non-verbs | Bits 18–22 (5 bits) |
| 1 | `I` | فَعَلَ | كَتَبَ | Base meaning | |
| 2 | `II` | فَعَّلَ | عَلَّمَ | Intensive/causative | |
| 3 | `III` | فَاعَلَ | كَاتَبَ | Reciprocal/attemptive | |
| 4 | `IV` | أَفْعَلَ | أَكْرَمَ | Causative/declarative | |
| 5 | `V` | تَفَعَّلَ | تَعَلَّمَ | Reflexive of II | |
| 6 | `VI` | تَفَاعَلَ | تَكَاتَبَ | Reciprocal of III | |
| 7 | `VII` | اِنْفَعَلَ | اِنْكَتَبَ | Passive/reflexive of I | |
| 8 | `VIII` | اِفْتَعَلَ | اِكْتَتَبَ | Reflexive of I | |
| 9 | `IX` | اِفْعَلَّ | اِحْمَرَّ | Colors/defects | |
| 10 | `X` | اِسْتَفْعَلَ | اِسْتَكْتَبَ | Requestive/deemed | |
| 11 | `XI` | اِفْعَالَّ | اِحْمَارَّ | Intensive colors | |
| 12 | `XII` | اِفْعَوْعَلَ | اِحْدَوْدَبَ | Intensive XX | |
| 13 | `XIII` | اِفْعَوَّلَ | اِعْلَوَّطَ | Rare pattern | |
| 14 | `XV` | اِفْعَنْلَلَ | اِسْحَنْكَفَ | Very rare | |
| 15 | – | (Form XIV) | — | Reserved (rare) | |

**Applicable POS:** verb.

**Note:** Forms XI–XV are rare in MSA but appear in Classical Arabic.
Forms are numbered with Roman numerals per standard Arabic grammar convention.
The bitfield encodes the numeric value directly (I=1, II=2, ..., XV=15).

### 7.2 Noun Type

| Code | Value | Arabic | Example | Description | Bit Position |
|------|-------|--------|---------|-------------|--------------|
| 0 | `not_a_noun` | ليس اسماً | — | Default for non-nouns | Bits 23–27 (5 bits) |
| 1 | `masdar` | مصدر | كِتَابَة | Verbal noun | |
| 2 | `ism_fail` | اسم فاعل | كَاتِب | Active participle | |
| 3 | `ism_maful` | اسم مفعول | مَكْتُوب | Passive participle | |
| 4 | `ism_makan` | اسم مكان | مَكْتَب | Noun of place | |
| 5 | `ism_zaman` | اسم زمان | مَوْعِد | Noun of time | |
| 6 | `ism_alah` | اسم آلة | مِفْتَاح | Instrument noun | |
| 7 | `sifah_mushabbahah` | صفة مشبهة | حَسَن | Resembling adjective | |
| 8 | `tafdil` | تفضيل | أَكْبَر | Elative (comparative/superlative) | |
| 9 | `nisbah` | نسبة | عَرَبِيّ | Relative adjective (nisbah) | |
| 10 | `jam_taksir` | جمع تكسير | كُتُب | Broken plural | |
| 11 | `ism_marrati` | اسم مرة | ضَرْبَة | Instance noun | |
| 12 | `ism_hayati` | اسم هيئة | جِلْسَة | Manner noun | |
| 13 | `jins` | اسم جنس | مَاء | Generic noun | |
| 14 | `ism_tasghir` | اسم تصغير | كُتَيِّب | Diminutive | |
| 15–31 | *reserved* | — | — | — | |

**Applicable POS:** noun, adjective.

### 7.3 Pronoun Type

| Code | Value | Arabic | Example | Description | Bit Position |
|------|-------|--------|---------|-------------|--------------|
| 0 | `not_a_pronoun` | ليس ضميراً | — | Default for non-pronouns | Bits 28–31 (4 bits) |
| 1 | `personal_attached` | متصل | -تُ, -كَ | Attached pronoun (subject/object/possessive) | |
| 2 | `personal_detached` | منفصل | هُوَ, أَنَا | Detached pronoun | |
| 3 | `demonstrative` | إشارة | هٰذَا | Demonstrative pronoun | |
| 4 | `relative` | موصول | الَّذِي | Relative pronoun | |
| 5 | `interrogative` | استفهام | مَنْ, مَا | Interrogative pronoun | |
| 6 | `conditional` | شرط | مَنْ, مَهْمَا | Conditional pronoun | |
| 7 | `compound` | مركب | بِمَا, لِمَنْ | Fused particle+pronoun form | |
| 8–15 | *reserved* | — | — | — | |

**Applicable POS:** pronoun.

**Note:** Values 3–7 extend RFC-0002 reserved space (bits 28–31, values 3–15). RFC-0002 must be updated before KB-0006 v1.0.0 release.

### 7.4 Transitivity

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `unspecified` | — | Not specified or unknown | Bits 32–35 (4 bits) |
| 1 | `intransitive` | لازم | No direct object | |
| 2 | `transitive_1` | متعدٍ (1) | Takes one object | |
| 3 | `transitive_2` | متعدٍ (2) | Takes two objects (e.g., أَعْطَى) | |
| 4 | `ditransitive` | متعدٍ لثلاثة | Takes three objects (rare) | |
| 5–15 | *reserved* | — | — | |

**Applicable POS:** verb.

**Notes:**
- Transitivity is an **inherent** property of a root+form combination (stored in KB-0001).
- Form II and IV often make intransitive roots transitive (e.g., جَلَسَ → جَلَّسَ).
- Form VII and VIII are often intransitive or reflexive even from transitive roots.

### 7.5 Root Type

| Code | Value | Arabic | Example | Description | Bit Position |
|------|-------|--------|---------|-------------|--------------|
| 0 | `unspecified` | — | — | Not specified | Bits 36–39 (4 bits) |
| 1 | `sound` | صحيح ساكن | ك ت ب | All consonants present and stable | |
| 2 | `weak_initial` | مثال | و ج د | First radical is wāw or yāʾ (mithāl) | |
| 3 | `weak_middle` | أجوف | ق و ل | Second radical is wāw or yāʾ (ajwaf) | |
| 4 | `weak_final` | ناقص | ر م ي | Third radical is wāw or yāʾ (nāqiṣ) | |
| 5 | `hamzated` | مهموز | س أ ل | One or more radicals is hamza | |
| 6 | `doubled` | مضاعف | م د د | Second and third radicals are identical | |
| 7 | `sound_quadriliteral` | رباعي | د ح ر ج | Four consonants (triliteral) | |
| 8 | `weak_quadriliteral` | رباعي معتل | س ب ع ل | Quadriliteral with a weak consonant | |
| 9–15 | *reserved* | — | — | — | |

**Applicable POS:** verb, noun, adjective, proper_noun.

**Notes:**
- root_type affects conjugation paradigms (KB-0003) and noun patterns (KB-0004).
- For triliteral roots: the weak root subtypes are `weak_initial` (مثال), `weak_middle` (أجوف), `weak_final` (ناقص).
- For quadriliteral roots: `sound_quadriliteral` or `weak_quadriliteral`.
- `hamzated` overrides the weak subtype when hamza is part of the root.
- `doubled` (مضاعف) is for roots with identical C₂ and C₃ (e.g., مَدَدَ).

---

## 8. Prosodic Features

### 8.1 Stress Pattern

| Code | Value | Arabic | Description | Bit Position |
|------|-------|--------|-------------|--------------|
| 0 | `unspecified` | — | Not determined | Bits 40–42 (3 bits) |
| 1 | `final` | — | Stress on the final syllable | |
| 2 | `penultimate` | — | Stress on the second-to-last syllable | |
| 3 | `antepenultimate` | — | Stress on the third-to-last syllable | |
| 4 | `pre_antepenultimate` | — | Stress on the fourth-to-last syllable (rare) | |
| 5–7 | *reserved* | — | — | |

**Applicable POS:** All.

**Arabic Notes:**
- Arabic stress is predictable based on syllable weight:
  - Final super-heavy syllable (CVVC or CVCC) → final stress.
  - Penultimate heavy syllable (CVV or CVC) → penultimate stress.
  - Otherwise → antepenultimate stress.
- Unlike other features, stress is **not stored** in the lexicon but computed by MOD-02.

### 8.2 Syllable Count

| Field | Description | Bit Position |
|-------|-------------|--------------|
| Value | Number of syllables (1–8) | Bits 43–46 (4 bits) |
| 0 | Unspecified | |
| 1–8 | Syllable count | |
| 9–15 | Reserved | |

**Applicable POS:** All.

---

## 9. Orthographic Features

### 9.1 has_shadda

| Code | Value | Description | Bit Position |
|------|-------|-------------|--------------|
| 0 | `false` | No shadda (ـّ) present | Bit 47 (1 bit) |
| 1 | `true` | Shadda present, indicating gemination | |

**Applicable POS:** All.

**Note:** Shadda (ـّ) indicates consonant gemination (doubling). It affects syllabification and stress.

### 9.2 has_madd

| Code | Value | Description | Bit Position |
|------|-------|-------------|--------------|
| 0 | `false` | No madd (آ, ـٓ) present | Bit 48 (1 bit) |
| 1 | `true` | Madda present, indicating long vowel prolongation | |

**Applicable POS:** verb, noun, adjective, proper_noun, pronoun.

**Note:** Madd is usually found in proper nouns (آدَم, قرآن) or in words with dagger alif.

### 9.3 has_hamza

| Code | Value | Description | Bit Position |
|------|-------|-------------|--------------|
| 0 | `false` | No hamza present | Bit 49 (1 bit) |
| 1 | `true` | Hamza present (any seat: ء, أ, إ, ؤ, ئ) | |

**Applicable POS:** verb, noun, adjective, particle, proper_noun, interrogative.

**Note:** This is a surface feature — the presence of hamza in the written form. It is distinct from `root_type = hamzated`, which is a lexical property of the root itself.

### 9.4 Orthographic Feature Summary

| Feature | Bit Position | Bit Width | Values | Applies To |
|---------|-------------|-----------|--------|------------|
| has_shadda | 47 | 1 | false (0), true (1) | All |
| has_madd | 48 | 1 | false (0), true (1) | verb, noun, adj, pronoun, prop_noun |
| has_hamza | 49 | 1 | false (0), true (1) | verb, noun, adj, particle, prop_noun, interr |

---

## 10. Feature Bitfield Encoding (RFC-0002)

### 10.1 Complete Bitfield Layout

```text
Bitfield: 64 bits (8 bytes)> The bitfield diagram below is a high-level visual. Use the **Bitfield Summary Table (§10.2)** for precise bit positions.

```text
Bitfield: 64 bits (8 bytes)

     3         2         1         1
     1         6         2         8         4         0         6         4
┌─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┐
│ CUSTOM  │   ORTHO │   PROS  │    DERIVATIONAL    │ INFLECTIONAL  │   POS   │
│ EXT     │ H M S   │ S S     │ R T R N P V        │ C S M T P N G │         │
│ (48-63) │ M Z D   │ C S T   │ T P N T F          │ A O V S E U M │ (0-3)   │
│         │ Z   D   │ Y L R   │ .   . . .          │ S . E . N . . │         │
│         │         │ L N E   │                     │ E . . . D . . │         │
│         │         │ . C S   │                     │               │         │
├─────────┼─────────┼─────────┼─────────────────────┼───────────────┼─────────┤
│ 63   50 │ 49 48 47│ 46   40 │ 39   36 35   32 31  28 27   23 22   18 │ 17   15 14   12 11   10   9    8   7    6   5    4   3    0 │
│ Reserved│ H M S   │ S S S S │ R  R  T  T  P  P  N  N  V  V  V  V  V│ S  C  C  V  M  M  T  T  P  P  N  N  G  G  P  P  P  P│
│ (must 0)│ A Z D   │ Y C S T │ T  T  R  R  N  N  N  N  F  F  F  F  F│ T  A  A  O  D  D  N  N  S  S  U  U  E  E  O  O  O  O│
│         │ M D D   │ L . R E │ P  P  N  N  T  T  T  T  R  R  R  R  R│    S  S  I  D  D  S  S  E  E  M  M  N  N  S  S  S  S│
│         │   A A   │ .   S S │ E  E  S  S  Y  Y  Y  Y  M  M  M  M  M│    E  E  C  D  D  E  E  R  R  B  B  D  D        │
│         │         │      S S │                                       │         │
│         │         │         │                                       │         │
└─────────┴─────────┴─────────┴─────────────────────┴───────────────┴─────────┘

### 10.2 Bitfield Summary Table

| Bits | Field Name | Width | Feature | Values |
|------|-----------|-------|---------|--------|
| 0–3 | `pos` | 4 | Part of Speech | 0=verb, ..., 9=interrogative |
| 4–5 | `gender` | 2 | Gender | 0=masc, 1=fem, 2=common, 3=unspec |
| 6–7 | `number` | 2 | Number | 0=sg, 1=dual, 2=pl, 3=unspec |
| 8–9 | `person` | 2 | Person | 0=1st, 1=2nd, 2=3rd, 3=unspec |
| 10–11 | `tense` | 2 | Tense | 0=past, 1=pres, 2=impv, 3=unspec |
| 12–13 | `mood` | 2 | Mood | 0=ind, 1=subj, 2=juss, 3=energ |
| 14 | `voice` | 1 | Voice | 0=active, 1=passive |
| 15–16 | `case` | 2 | Case | 0=nom, 1=acc, 2=gen, 3=unspec |
| 17 | `state` | 1 | State | 0=def, 1=indef |
| 18–22 | `verb_form` | 5 | Verb Form | 0=not_a_verb, 1=I, ..., 15=XV |
| 23–27 | `noun_type` | 5 | Noun Type | 0=not_a_noun, 1=masdar, ..., 14=ism_tasghir |
| 28–31 | `pronoun_type` | 4 | Pronoun Type | 0=not_a_pronoun, 1=attached, ..., 7=compound |
| 32–35 | `transitivity` | 4 | Transitivity | 0=unspec, 1=intrans, ..., 4=ditrans |
| 36–39 | `root_type` | 4 | Root Type | 0=unspec, 1=sound, ..., 8=weak_quad |
| 40–42 | `stress_pattern` | 3 | Stress | 0=unspec, 1=final, ..., 4=pre_antepenult |
| 43–46 | `syllable_count` | 4 | Syllables | 0=unspec, 1–8=syllable count |
| 47 | `has_shadda` | 1 | Shadda | 0=false, 1=true |
| 48 | `has_madd` | 1 | Madd | 0=false, 1=true |
| 49 | `has_hamza` | 1 | Hamza | 0=false, 1=true |
| 50–63 | *reserved* | 14 | — | Must be zero for KB-defined features |

### 10.3 Feature Bitmask Encoding Functions

```pseudo
Function: pack_features(features) -> u64
Input: features (dictionary of feature name → value)
Output: 64-bit bitfield

1. Initialize bitfield = 0

2. Pack pos (bits 0–3):
   bitfield |= (features.pos_code << 0)

3. Pack gender (bits 4–5):
   bitfield |= (features.gender_code << 4)

4. Pack number (bits 6–7):
   bitfield |= (features.number_code << 6)

5. Pack person (bits 8–9):
   bitfield |= (features.person_code << 8)

6. Pack tense (bits 10–11):
   bitfield |= (features.tense_code << 10)

7. Pack mood (bits 12–13):
   bitfield |= (features.mood_code << 12)

8. Pack voice (bit 14):
   bitfield |= (features.voice_code << 14)

9. Pack case (bits 15–16):
   bitfield |= (features.case_code << 15)

10. Pack state (bit 17):
    bitfield |= (features.state_code << 17)

11. Pack verb_form (bits 18–22):
    bitfield |= (features.verb_form_code << 18)

12. Pack noun_type (bits 23–27):
    bitfield |= (features.noun_type_code << 23)

13. Pack pronoun_type (bits 28–31):
    bitfield |= (features.pronoun_type_code << 28)

14. Pack transitivity (bits 32–35):
    bitfield |= (features.transitivity_code << 32)

15. Pack root_type (bits 36–39):
    bitfield |= (features.root_type_code << 36)

16. Pack stress_pattern (bits 40–42):
    bitfield |= (features.stress_pattern_code << 40)

17. Pack syllable_count (bits 43–46):
    bitfield |= (features.syllable_count << 43)

18. Pack has_shadda (bit 47):
    bitfield |= (features.has_shadda << 47)

19. Pack has_madd (bit 48):
    bitfield |= (features.has_madd << 48)

20. Pack has_hamza (bit 49):
    bitfield |= (features.has_hamza << 49)

21. Ensure reserved bits (50–63) are 0:
    bitfield &= 0x0000FFFFFFFFFFFF

22. Return bitfield


Function: unpack_features(bitfield: u64) -> features
Input: 64-bit bitfield
Output: feature dictionary

1. features.pos = decode_pos((bitfield >> 0) & 0xF)
2. features.gender = decode_gender((bitfield >> 4) & 0x3)
3. features.number = decode_number((bitfield >> 6) & 0x3)
4. features.person = decode_person((bitfield >> 8) & 0x3)
5. features.tense = decode_tense((bitfield >> 10) & 0x3)
6. features.mood = decode_mood((bitfield >> 12) & 0x3)
7. features.voice = decode_voice((bitfield >> 14) & 0x1)
8. features.case = decode_case((bitfield >> 15) & 0x3)
9. features.state = decode_state((bitfield >> 17) & 0x1)
10. features.verb_form = decode_verb_form((bitfield >> 18) & 0x1F)
11. features.noun_type = decode_noun_type((bitfield >> 23) & 0x1F)
12. features.pronoun_type = decode_pronoun_type((bitfield >> 28) & 0xF)
13. features.transitivity = decode_transitivity((bitfield >> 32) & 0xF)
14. features.root_type = decode_root_type((bitfield >> 36) & 0xF)
15. features.stress_pattern = decode_stress_pattern((bitfield >> 40) & 0x7)
16. features.syllable_count = (bitfield >> 43) & 0xF
17. features.has_shadda = ((bitfield >> 47) & 0x1) == 1
18. features.has_madd = ((bitfield >> 48) & 0x1) == 1
19. features.has_hamza = ((bitfield >> 49) & 0x1) == 1

20. Return features
```

### 10.4 Reserved Bits Policy

Bits 50–63 are reserved for:
- **Plugins:** External AGOS plugins MAY use bits 50–63 for custom feature extensions.
- **Future KB versions:** New KB-0007 features MUST use reserved bits before using extension bits.
- **Conflicts:** If two plugins attempt to use the same reserved bit, the pipeline MUST flag an error at load time.

```yaml
reserved_bits_policy:
  range: [50, 63]
  default_value: 0
  validation: "MUST be zero unless explicitly assigned by a registered plugin"
  conflict_detection: "Enabled at KB load time"
```

---

## 11. Feature Agreement Rules

### 11.1 Subject-Verb Agreement

```yaml
agreement_rules:
  - id: "AGR-001"
    rule_type: "subject_verb"
    description: "Verb agrees with subject in person, number, and gender"
    source_feature: "subject.person, subject.number, subject.gender"
    target_feature: "verb.person, verb.number, verb.gender"
    constraint: |
      verb.person == subject.person AND
      verb.number == subject.number AND
      verb.gender == subject.gender
    exceptions: |
      In SVO order, 3fs verb can be used with feminine plural subjects
      In VSO order, verb is always singular, agrees only in gender
    applies_to: ["verb"]
```

**Agreement Table:**

| Subject | Verb (VSO) | Verb (SVO) |
|---------|------------|------------|
| 3ms | 3ms | 3ms |
| 3fs | 3fs | 3fs |
| 3md | 3ms | 3md |
| 3fd | 3fs | 3fd |
| 3mp | 3ms | 3mp |
| 3fp | 3fs | 3fp |
| 2ms | 2ms | 2ms |
| 2fs | 2fs | 2fs |
| ... | ... | ... |

### 11.2 Noun-Adjective Agreement

```yaml
  - id: "AGR-002"
    rule_type: "noun_adjective"
    description: "Adjective agrees with noun in gender, number, case, and state"
    source_feature: "noun.gender, noun.number, noun.case, noun.state"
    target_feature: "adjective.gender, adjective.number, adjective.case, adjective.state"
    constraint: |
      adjective.gender == noun.gender AND
      adjective.case == noun.case AND
      adjective.state == noun.state AND
      (noun.number != "plural" OR adjective.number == noun.number)
    exceptions: |
      For non-human plurals (جمع غير عاقل), adjective is feminine singular
      Broken plural nouns often take feminine singular adjective agreement
    applies_to: ["adjective"]
```

### 11.3 Government Rules

```yaml
  - id: "AGR-003"
    rule_type: "government"
    description: "Prepositions govern the genitive case on their object"
    source_feature: "preposition"
    target_feature: "object.case"
    constraint: "object.case == genitive"
    exceptions: "None for standard prepositions"
    applies_to: ["preposition"]

  - id: "AGR-004"
    rule_type: "government"
    description: "Inna and its sisters govern the accusative case on the subject"
    source_feature: "particle (inna group)"
    target_feature: "subject.case"
    constraint: "subject.case == accusative"
    exceptions: "None"
    applies_to: ["particle"]
```

### 11.4 Mood Government Rules

```yaml
  - id: "AGR-005"
    rule_type: "mood_government"
    description: "Subjunctive particles govern subjunctive mood on the verb"
    source_feature: "governing_particle (أَنْ, لَنْ, etc.)"
    target_feature: "verb.mood"
    constraint: "verb.mood == subjunctive"
    exceptions: "None"
    applies_to: ["verb"]

  - id: "AGR-006"
    rule_type: "mood_government"
    description: "Jussive particles govern jussive mood on the verb"
    source_feature: "governing_particle (لَمْ, لَمَّا, etc.)"
    target_feature: "verb.mood"
    constraint: "verb.mood == jussive"
    exceptions: "None"
    applies_to: ["verb"]
```

---

## 12. Feature Inference Rules

### 12.1 Default Value Inference

When a feature value is not explicitly extracted, KB-0007 provides default values:

```yaml
inference_rules:
  # POS-specific defaults
  - id: "INF-001"
    description: "Verb defaults"
    condition: "pos == verb"
    defaults:
      gender: "masculine"
      number: "singular"
      person: "third"
      tense: "present"
      mood: "indicative"
      voice: "active"
      verb_form: "I"
      transitivity: "unspecified"

  - id: "INF-002"
    description: "Noun defaults"
    condition: "pos == noun"
    defaults:
      gender: "masculine"
      number: "singular"
      case: "nominative"
      state: "indefinite"
      noun_type: "not_a_noun"

  - id: "INF-003"
    description: "Adjective defaults"
    condition: "pos == adjective"
    defaults:
      gender: "masculine"
      number: "singular"
      case: "nominative"
      state: "indefinite"
      noun_type: "sifah_mushabbahah"

  - id: "INF-004"
    description: "Pronoun defaults: person = third for demonstratives/relatives"
    condition: "pos == pronoun AND pronoun_type in [demonstrative, relative]"
    defaults:
      person: "third"

  - id: "INF-005"
    description: "Particle defaults: no inflectional features"
    condition: "pos == particle"
    defaults:
      gender: "unspecified"
      number: "unspecified"
      person: "unspecified"
      tense: "unspecified"
      mood: "unspecified"
      voice: "unspecified"
      case: "unspecified"
      state: "unspecified"
```

### 12.2 Cross-Feature Inference

```yaml
  - id: "INF-010"
    input_feature: "pos"
    input_value: "pronoun"
    inferred_feature: "pronoun_type"
    inferred_value: "personal_detached"
    priority: 1
    condition: "pronoun is a known detached pronoun from KB-0006"
    notes: "Overridden by explicit KB-0006 entry"

  - id: "INF-011"
    input_feature: "verb_form"
    input_value: "II"
    inferred_feature: "transitivity"
    inferred_value: "transitive_1"
    priority: 2
    condition: "Form II default (more specific inference from KB-0001 overrides)"
    notes: "Form II verbs are typically transitive, but some are intransitive"

  - id: "INF-012"
    input_feature: "noun_type"
    input_value: "ism_fail"
    inferred_feature: "gender"
    inferred_value: "masculine"
    priority: 1
    condition: "Default active participle gender"
    notes: "Overridden by explicit feminine marking (ة suffix)"

  - id: "INF-013"
    input_feature: "noun_type"
    input_value: "tafdil"
    inferred_feature: "gender"
    inferred_value: "masculine"
    priority: 1
    condition: "Default elative gender"
    notes: "Elatives are usually masculine singular even when modifying feminine nouns"

  - id: "INF-014"
    input_feature: "tense"
    input_value: "present"
    inferred_feature: "mood"
    inferred_value: "indicative"
    priority: 1
    condition: "Default present tense mood"
    notes: "Overridden by governing particles"

  - id: "INF-015"
    input_feature: "tense"
    input_value: "past"
    inferred_feature: "mood"
    inferred_value: "unspecified"
    priority: 1
    condition: "Past tense has no mood"
    notes: "Past verbs do not carry mood distinctions"
```

### 12.3 Inference Priority Chain

When multiple inference rules could apply, the following priority chain determines the final value:

```diff
  1. Explicit extraction from KB-0001/2/3/4/5/6    (highest priority, overrides all)
  2. Explicit grammar rule (agreement, government)   (second highest)
  3. Cross-feature inference (INF-010 through INF-015) (third)
  4. POS-specific defaults (INF-001 through INF-005)  (lowest priority)
```

---

## 13. Feature Validation

### 13.1 Validation Rules

KB-0007 defines validation rules that MOD-04 must enforce after feature extraction:

```yaml
validation_rules:
  # --- Feature Existence ---
  - id: "VAL-001"
    description: "Every token must have a POS value"
    check: "features.pos != undefined"
    severity: "error"

  - id: "VAL-002"
    description: "Token POS must be a valid value (0–9)"
    check: "features.pos_code in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"
    severity: "error"

  # --- Applicability checks ---
  - id: "VAL-003"
    description: "Tense only applies to verbs"
    check: "if features.tense != unspecified then features.pos == verb"
    severity: "warning"

  - id: "VAL-004"
    description: "Mood only applies to verbs with tense=present"
    check: "if features.mood not in [unspecified] then features.tense == present"
    severity: "error"

  - id: "VAL-005"
    description: "Verb form only applies to verbs"
    check: "if features.verb_form != not_a_verb then features.pos == verb"
    severity: "error"

  - id: "VAL-006"
    description: "Noun type only applies to nouns and adjectives"
    check: "if features.noun_type != not_a_noun then features.pos in [noun, adjective]"
    severity: "warning"

  - id: "VAL-007"
    description: "Pronoun type only applies to pronouns"
    check: "if features.pronoun_type != not_a_pronoun then features.pos == pronoun"
    severity: "error"

  - id: "VAL-008"
    description: "Case only applies to nouns, adjectives, and proper nouns"
    check: "if features.case != unspecified then features.pos in [noun, adjective, proper_noun]"
    severity: "warning"

  - id: "VAL-009"
    description: "State only applies to nouns, adjectives, and proper nouns"
    check: "if features.state != unspecified then features.pos in [noun, adjective, proper_noun]"
    severity: "warning"

  - id: "VAL-010"
    description: "Transitivity only applies to verbs"
    check: "if features.transitivity != unspecified then features.pos == verb"
    severity: "warning"

  - id: "VAL-011"
    description: "Root type only applies to verbs, nouns, adjectives, and proper nouns"
    check: "if features.root_type != unspecified then features.pos in [verb, noun, adjective, proper_noun]"
    severity: "warning"

  # --- Value range checks ---
  - id: "VAL-020"
    description: "Syllable count must be 0–8"
    check: "features.syllable_count in [0, 1, 2, 3, 4, 5, 6, 7, 8]"
    severity: "error"

  - id: "VAL-021"
    description: "Verb form must be 0–15"
    check: "features.verb_form_code in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]"
    severity: "error"

  # --- Reserved bits check ---
  - id: "VAL-030"
    description: "Reserved bits (50–63) must be zero unless plugin-registered"
    check: "bitfield & 0xFFFF000000000000 == 0"
    severity: "error"
```

### 13.2 Validation Pipeline

```diff
  MOD-04: Feature Validation Pipeline

  Input: raw token, extracted features, context

  1. POS Assignment Check:
     └── Every token MUST have a POS
         ├── If no POS → attempt inference from token form
         └── If still no POS → assign "noun" default

  2. Feature Applicability Check:
     └── For each feature, verify it is valid for this POS
         ├── Invalid features → set to "unspecified" with warning
         └── Log invalid feature warnings for debugging

  3. Value Range Check:
     └── For each present feature, verify value is in allowed range
         ├── Out-of-range values → set to "unspecified" with error
         └── Log value errors for debugging

  4. Cross-Feature Constraint Check:
     └── Verify no violated constraints
         ├── Constraint violations → attempt inference correction
         └── If uncorrectable → flag ambiguity in output

  5. Reserved Bits Check:
     └── Verify bits 50–63 are zero (or plugin-registered)
         ├── Non-zero reserved bits → log error
         └── If plugin-registered → pass through
```

---

## 14. Cross-Feature Constraints

### 14.1 Valid Feature Combinations

```yaml
constraints:
  - id: "CON-001"
    constraint_type: "valid_combination"
    description: "Present tense verbs must have a valid mood"
    features: ["tense", "mood"]
    allowed_combinations:
      - ["past", "unspecified"]
      - ["present", "indicative"]
      - ["present", "subjunctive"]
      - ["present", "jussive"]
      - ["present", "energetic"]
      - ["imperative", "jussive"]
      - ["imperative", "unspecified"]

  - id: "CON-002"
    constraint_type: "valid_combination"
    description: "Voice and verb form compatibility"
    features: ["voice", "verb_form"]
    allowed_combinations:
      - ["active", "I"]
      - ["passive", "I"]
      - ["active", "II"]
      - ["passive", "II"]
      - ...  # All forms allow both voices
      - ["active", "XV"]
      - ["passive", "XV"]

  - id: "CON-003"
    constraint_type: "valid_combination"
    description: "Case and state apply together to nouns"
    features: ["case", "state", "pos"]
    condition: "pos == noun"
    allowed_combinations:
      - ["nominative", "indefinite"]
      - ["nominative", "definite"]
      - ["accusative", "indefinite"]
      - ["accusative", "definite"]
      - ["genitive", "indefinite"]
      - ["genitive", "definite"]

  - id: "CON-004"
    constraint_type: "valid_combination"
    description: "Person and pronoun_type compatibility"
    features: ["person", "pronoun_type"]
    condition: "pos == pronoun"
    allowed_combinations:
      - ["first", "personal_attached"]
      - ["first", "personal_detached"]
      - ["second", "personal_attached"]
      - ["second", "personal_detached"]
      - ["third", "personal_attached"]
      - ["third", "personal_detached"]
      - ["third", "demonstrative"]
      - ["third", "relative"]
      - [null, "interrogative"]
      - [null, "conditional"]
```

### 14.2 Mutual Exclusions

```yaml
  - id: "CON-010"
    constraint_type: "mutual_exclusion"
    description: "Verb and noun features are mutually exclusive"
    features: ["verb_form", "noun_type"]
    condition: "Only one can be non-default"
    notes: "pos determines which is active"

  - id: "CON-011"
    constraint_type: "mutual_exclusion"
    description: "Verb form and pronoun type are mutually exclusive"
    features: ["verb_form", "pronoun_type"]
    condition: "Only one can be non-default"
    notes: "pos determines which is active"
```

### 14.3 Dependency Constraints

```yaml
  - id: "CON-020"
    constraint_type: "dependency"
    description: "Mood depends on tense = present"
    features: ["mood", "tense"]
    condition: "mood in [indicative, subjunctive, jussive, energetic] → tense == present"
    notes: "Mood only has meaning in present tense"

  - id: "CON-021"
    constraint_type: "conditional_required"
    description: "If verb_form is specified, transitivity should be specified"
    features: ["verb_form", "transitivity"]
    condition: "verb_form != not_a_verb"
    notes: "Transitivity may be unspecified if not stored"

  - id: "CON-022"
    constraint_type: "conditional_required"
    description: "If noun_type is specified, case and state may be relevant"
    features: ["noun_type", "case", "state"]
    condition: "noun_type != not_a_noun AND pos in [noun, adjective]"
    notes: "Some noun types (masdar, ism_fail) can function as verbs in some contexts"
```

---

## 15. Feature Matching Algorithm

### 15.1 Primary Algorithm: Feature Extraction from Token

```pseudo
Algorithm: extract_features
Input: token (string), context (TokenContext)
Output: FeatureSet (64-bit bitfield)

1. Determine Part of Speech:
   a. Check KB-0005 (Particles) fast path:
      If matched → pos = particle, goto step 7.
   b. Check KB-0006 (Pronouns) fast path:
      If matched → pos = pronoun, extract pronoun_type, goto step 7.
   c. Attempt root extraction (KB-0001):
      If root found:
         If root has verbal wazan → pos = verb
         If root has noun pattern → pos = noun/adjective
         Extract root_type from root entry.
      Else:
         pos = noun (default)

2. Extract inflectional features (verbs):
   If pos == verb:
      a. Extract perfect/imperfect/imperative markers.
      b. Determine tense from prefix/suffix pattern.
      c. Determine mood from suffix (present only).
      d. Determine voice from vowel pattern.
      e. Extract person from prefix (imperfect) or suffix (perfect).
      f. Extract number from suffix.
      g. Extract gender from suffix (3rd person only).

3. Extract inflectional features (nouns/adjectives):
   If pos in [noun, adjective]:
      a. Determine case from final vowel/nunation.
      b. Determine state from الـ prefix or nunation.
      c. Determine gender from morphological markers (ة, اء, etc.).
      d. Determine number from form (sound plural, broken plural, dual).

4. Extract derivational features:
   a. Determine verb_form from wazan pattern (KB-0002).
   b. Determine noun_type from pattern (KB-0004).
   c. Look up transitivity from KB-0001 root entry.
   d. Extract root_type from KB-0001.

5. Extract prosodic features (may be populated by MOD-02 instead of MOD-04):
   a. Syllable count may be precomputed and stored alongside the compiled KB, or computed by MOD-02 (PhonologicalProcessor) during phonology analysis.
   b. Stress pattern is determined by MOD-02 based on syllable weights. If MOD-02 has not yet run, stress defaults to `unspecified`.
   c. In the fast path, prosodic features may be omitted entirely to save time; they are filled in during later pipeline stages.

6. Extract orthographic features:
   a. Check for shadda presence (ـّ).
   b. Check for madd presence (آ).
   c. Check for hamza presence (any seat).

7. Apply default values for unspecified features:
   a. For each feature not explicitly extracted:
      Apply INF-001 through INF-005 defaults.

8. Validate feature set:
   a. Run VAL-001 through VAL-030 validation rules.
   b. Log warnings and errors as appropriate.
   c. Fix correctable issues.

9. Pack into 64-bit bitfield (pack_features).

10. Return FeatureSet (bitfield + confidence scores).
```

### 15.2 Secondary Algorithm: Feature Agreement Matching

```pseudo
Algorithm: check_feature_agreement
Input: feature_set_a (FeatureSet), feature_set_b (FeatureSet), rule_type (string)
Output: boolean (agreement holds), string[] (violations)

1. Select agreement rules by rule_type:
   a. If rule_type == "subject_verb" → use AGR-001.
   b. If rule_type == "noun_adjective" → use AGR-002.
   c. If rule_type == "government" → use AGR-003, AGR-004.
   d. If rule_type == "mood_government" → use AGR-005, AGR-006.

2. For each applicable rule:
   a. Extract source features from feature_set_a.
   b. Extract target features from feature_set_b.
   c. Compare feature values per constraint expression.
   d. If violation found → record violation.

3. Apply exceptions:
   a. Check for exception conditions.
   b. If exception applies → skip the violated rule.

4. Return:
   a. True if all rules pass (or exceptions apply).
   b. List of all violation strings if any fail.

Performance target: < 1 μs for 2-feature agreement check.
```

---

## 16. Serialization & Storage

### 16.1 Source Format

```diff
  /knowledge/KB-0007/
  ├── metadata.yaml                     # KB metadata (version, counts)
  ├── features/
  │   ├── pos.yaml                      # Part of speech definitions
  │   ├── inflectional/
  │   │   ├── gender.yaml               # Gender feature definition
  │   │   ├── number.yaml               # Number feature definition
  │   │   ├── person.yaml               # Person feature definition
  │   │   ├── tense.yaml                # Tense feature definition
  │   │   ├── mood.yaml                 # Mood feature definition
  │   │   ├── voice.yaml                # Voice feature definition
  │   │   ├── case.yaml                 # Case feature definition
  │   │   └── state.yaml                # State feature definition
  │   ├── derivational/
  │   │   ├── verb-form.yaml            # Verb form (I–XV) definitions
  │   │   ├── noun-type.yaml            # Noun type definitions
  │   │   ├── pronoun-type.yaml         # Pronoun type definitions
  │   │   ├── transitivity.yaml         # Transitivity definitions
  │   │   └── root-type.yaml            # Root type definitions
  │   ├── prosodic/
  │   │   ├── stress-pattern.yaml       # Stress pattern definitions
  │   │   └── syllable-count.yaml       # Syllable count definitions
  │   └── orthographic/
  │       ├── has-shadda.yaml           # Shadda feature
  │       ├── has-madd.yaml             # Madd feature
  │       └── has-hamza.yaml            # Hamza feature
  ├── rules/
  │   ├── agreement.yaml                # Agreement rules (AGR-*)
  │   ├── inference.yaml               # Inference rules (INF-*)
  │   └── constraints.yaml              # Cross-feature constraints (CON-*)
  └── validation/
      └── validation-rules.yaml         # Validation rules (VAL-*)
```

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0007"
title: "Morphological Features — Taxonomy, Bitfield Encoding & Inference Rules"
version: "1.0.0"
status: "draft" | "review" | "published"

feature_count: 19
bitfield_width: 64
feature_category_counts:
  pos: 1
  inflectional: 8
  derivational: 5
  prosodic: 2
  orthographic: 3

rule_counts:
  agreement: 6
  inference: 15
  constraints: 12
  validation: 15

created_at: "2026-07-15T00:00:00Z"
updated_at: "2026-07-15T00:00:00Z"

sources:
  - name: "Sibawayh, Al-Kitab"
    version: "critical_1988"
  - name: "Ibn Jinni, Al-Khasa'is"
    version: "print_1952"
  - name: "Wright's Arabic Grammar"
    version: "3rd_edition"

checksum_sha256: "a1b2c3d4e5f6..."
```

### 16.2 Compiled Format (Feature Map + Rule Table)

```diff
  Compiled Feature Binary:
  ┌──────────────────────────────────────────────────────────────┐
  │ HEADER                                                       │
  │ ├── magic: "AGOSKB07" (8 bytes)                             │
  │ ├── version: major(2B) + minor(2B) + patch(2B)              │
  │ ├── feature_count: u32 (4 bytes)                            │
  │ ├── rule_count: u32 (4 bytes)                               │
  │ ├── constraint_count: u32 (4 bytes)                         │
  │ ├── bitfield_map_offset: u32 (4 bytes)                      │
  │ ├── rule_table_offset: u32 (4 bytes)                        │
  │ ├── string_table_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ BITFIELD MAP (position → feature mapping)                   │
  │ ├── Array of FeatureMapping entries (32 bytes each)         │
  │ │   ├── feature_id: u16                                     │
  │ │   ├── bit_position: u8                                    │
  │ │   ├── bit_width: u8                                       │
  │ │   ├── default_code: u8                                    │
  │ │   ├── value_count: u8                                     │
  │ │   ├── applies_to_pos_mask: u16 (10 bits used)            │
  │ │   └── value_list: (uint8[])                               │
  │ └── ... (19 feature mappings)                               │
  ├──────────────────────────────────────────────────────────────┤
  │ RULE TABLE                                                   │
  │ ├── Array of RuleEntry (44 bytes each)                      │
  │ │   ├── rule_id: u8                                         │
  │ │   ├── rule_type: u8                                       │
  │ │   ├── input_feature_id: u16                               │
  │ │   ├── target_feature_id: u16                              │
  │ │   ├── constraint_data: u32                                │
  │ │   └── priority: u8                                        │
  │ └── ... (~33 rule entries)                                  │
  ├──────────────────────────────────────────────────────────────┤
  │ STRING TABLE                                                 │
  │ ├── Length-prefixed UTF-8 strings                           │
  │ ├── Feature names, value labels, rule descriptions          │
  │ └── Referenced by offsets from bitfield map & rule table    │
  └──────────────────────────────────────────────────────────────┘
```

#### C Struct: Feature Mapping

```c
struct FeatureMapping {
    uint16_t feature_id;                // Feature identifier
    uint8_t  bit_position;              // Starting bit position (0–49)
    uint8_t  bit_width;                 // 1, 2, 3, 4, or 5 bits
    uint8_t  default_code;              // Default value code
    uint8_t  value_count;               // Number of valid values
    uint16_t applies_to_pos_mask;       // Bitmask of applicable POS types
    uint8_t  reserved[2];               // Padding
};
```

#### C Struct: Rule Entry

```c
struct RuleEntry {
    uint8_t  rule_id;                   // Rule identifier (AGR=1-6, INF=7-21, CON=22-33)
    uint8_t  rule_type;                 // 0=agreement, 1=inference, 2=constraint, 3=validation
    uint16_t input_feature_id;          // Source/input feature
    uint16_t target_feature_id;         // Target/inferred feature
    uint32_t constraint_data;           // Encoded constraint expression
    uint8_t  priority;                  // Inference priority (1-255, higher = overrides)
    uint8_t  reserved[3];               // Padding to 16 bytes
};
```

### 16.3 File Packaging

```diff
  KB-0007-v1.0.0.agos-kb              # Compiled feature binary
  KB-0007-v1.0.0.agos-kb.sig          # Ed25519 signature
  KB-0007-v1.0.0.agos-kb.sha256       # SHA-256 checksum
  KB-0007-v1.0.0.source.tar.gz        # Source YAML files (optional)
```

### 16.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Bitfield map | 0.1 MB | 0.2 MB | 19 feature mappings × 32 bytes |
| Rule table | 0.1 MB | 0.2 MB | ~33 rules × 16 bytes |
| Validation rules | 0.05 MB | 0.1 MB | ~15 validation rules |
| String table | 0.4 MB | 0.8 MB | Feature names, value labels, descriptions |
| Agreement lookup index | 0.2 MB | 0.5 MB | Fast agreement matching tables |
| Reserved bit table | 0.05 MB | 0.1 MB | Plugin registration table |
| **Total** | **~1 MB** | **~2 MB** | Memory-mapped load |

---

## 17. Versioning & Evolution

### 17.1 Versioning Scheme

KB-0007 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to bitfield layout, feature removal, POS value renumbering | `1.0.0` → `2.0.0` | Requires RFC-0002 update, invalidates all compiled KBs |
| **MINOR** | New feature value, new feature, new agreement rule | `1.0.0` → `1.1.0` | Backward-compatible; existing bitfields remain valid |
| **PATCH** | Corrections to feature descriptions, improved examples, typo fixes | `1.0.0` → `1.0.1` | No schema or encoding changes |

### 17.2 Cross-KB Compatibility

```yaml
cross_kb_compatibility:
  KB-0001: ">= 1.0.0"       # root_type, transitivity features
  KB-0002: ">= 1.0.0"       # verb_form, noun_type features
  KB-0003: ">= 1.0.0"       # tense, mood, voice, person, number, gender
  KB-0004: ">= 1.0.0"       # noun_type, case, state, gender
  KB-0005: ">= 1.0.0"       # pos (particle)
  KB-0006: ">= 1.0.0"       # pos (pronoun), pronoun_type
  RFC-0002: ">= 1.0.0"      # Bitfield encoding MUST match exactly
```

### 17.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add new feature value | MINOR | Add value to feature, update encoding |
| Add new feature | MINOR | Find unused bit range, add feature mapping |
| Renumber feature values | MAJOR | Update all KBs and RFC-0002 |
| Change bitfield width | MAJOR | Update RFC-0002, all KBs, all compilers |
| Add agreement rule | MINOR | Add new AGR-ID to rules table |
| Mark feature deprecated | MINOR | Add deprecation note, keep feature working |
| Remove deprecated feature | MAJOR | Wait at least 2 MINOR versions |

### 17.4 Bitfield Extension Policy

When all 50 bits (0–49) are exhausted:

1. **Reserved bits (50–63)** can be allocated with a MINOR version bump.
2. If reserved bits are also exhausted, the **custom/plugin extension (bits 48–63)** may be reorganized with a MAJOR version.
3. A future MAJOR version may extend to a 128-bit feature field with a new RFC-0002 revision.

---

## 18. Quality Requirements

### 18.1 Completeness Targets

| Category | Minimum | Target | Stretch |
|----------|---------|--------|---------|
| Part of speech definitions | 100% | 100% | 100% |
| Inflectional features (all 8) | 100% | 100% | 100% |
| Derivational features (all 5) | 100% | 100% | 100% |
| Prosodic features (all 2) | 100% | 100% | 100% |
| Orthographic features (all 3) | 100% | 100% | 100% |
| Feature bitfield mapping (0–49) | 100% | 100% | 100% |
| POS applicability table | 100% | 100% | 100% |
| Agreement rules | 90% | 95% | 100% |
| Inference rules | 80% | 90% | 95% |
| Validation rules | 90% | 95% | 100% |
| Cross-feature constraints | 85% | 90% | 95% |

### 18.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Bitfield position accuracy | 100% — must match RFC-0002 | Automated comparison |
| Value code assignment | 100% — codes match encoding table | Automated check |
| POS applicability | 100% — each feature maps to correct POS | Automated check |
| Agreement rule correctness | 100% — rules match Arabic grammar | Manual linguistic review |
| Feature description accuracy | 100% — descriptions match linguistic reality | Manual review |
| Unicode normalization | 100% — all Arabic text valid NFC-normalized UTF-8 | Automated encoding check |

### 18.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate YAML structure
  ├── schema: validate against KB-0007 JSON Schema
  ├── bitfield_check: verify all bit positions unique and non-overlapping
  ├── value_check: verify codes are contiguous and within bit width
  └── lint: field presence, Arabic-only text for Arabic fields

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── bitfield_mapping: verify bytecode matches RFC-0002
  ├── agreement_regression: verify all 6 agreement rules
  ├── compilation: verify feature map compiles without error
  ├── size_budget: verify compiled size ≤ 2 MB
  └── regression: verify 50+ known feature combinations are correctly encoded

  Review (manual, per release):
  ├── bitfield_review: linguist + engineer review encoding
  ├── rule_review: verify rule correctness
  └── changelog: verify changelog accuracy
```

### 18.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Feature map lookup (by name) | < 200 ns | Per lookup, average |
| Feature map lookup (by position) | < 100 ns | Per lookup, average |
| Pack feature set to bitfield | < 500 ns | Average per token |
| Unpack bitfield to features | < 300 ns | Average per token |
| Agreement check (2 features) | < 1 μs | Average per pair |
| Full feature extraction + validation | < 5 μs | Average per token |
| KB load time | < 5 ms | mmap + verify checksum |
| Memory | ~1–2 MB | RSS |

---

## 19. Example Entries

### 19.1 POS Feature: Verb

```json
{
  "id": "KB-0007:pos",
  "feature_name": "pos",
  "category": "pos",
  "bitfield_position": 0,
  "bitfield_width": 4,
  "values": [
    { "name": "verb", "code": 0, "label": "Verb",
      "description": "Action or state word", "arabic_term": "فعل" }
  ],
  "default_value": "noun",
  "description": "Part of speech — the fundamental grammatical category of a token",
  "linguistic_relevance": "POS determines which other features apply to a token",
  "applies_to_pos": ["verb", "noun", "particle", "pronoun", "adjective",
                      "adverb", "preposition", "conjunction", "proper_noun", "interrogative"],
  "depends_on": null,
  "mutually_exclusive_with": [],
  "requires_feature": [],
  "encoding": [
    { "value_name": "verb", "binary_code": 0, "notes": "Covers all verb forms I–XV" },
    { "value_name": "noun", "binary_code": 1, "notes": "Covers all noun types" },
    { "value_name": "particle", "binary_code": 2, "notes": "Invariable function words" },
    { "value_name": "pronoun", "binary_code": 3, "notes": "Personal, demonstrative, relative, etc." },
    { "value_name": "adjective", "binary_code": 4, "notes": "Descriptive words, agreeing with nouns" },
    { "value_name": "adverb", "binary_code": 5, "notes": "Manner, time, place adverbs" },
    { "value_name": "preposition", "binary_code": 6, "notes": "Governing genitive case" },
    { "value_name": "conjunction", "binary_code": 7, "notes": "Coordinating and subordinating" },
    { "value_name": "proper_noun", "binary_code": 8, "notes": "Proper names, no article needed" },
    { "value_name": "interrogative", "binary_code": 9, "notes": "Question words (may overlap with particle/pronoun)" }
  ],
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

### 19.2 Tense Feature: Full Entry

```json
{
  "id": "KB-0007:tense",
  "feature_name": "tense",
  "category": "inflectional",
  "bitfield_position": 10,
  "bitfield_width": 2,
  "values": [
    { "name": "past", "code": 0, "label": "Past",
      "description": "Completed action (perfect tense)", "arabic_term": "ماض" },
    { "name": "present", "code": 1, "label": "Present",
      "description": "Ongoing or future action (imperfect tense)", "arabic_term": "مضارع" },
    { "name": "imperative", "code": 2, "label": "Imperative",
      "description": "Command form", "arabic_term": "أمر" },
    { "name": "unspecified", "code": 3, "label": "Unspecified",
      "description": "Tense not applicable or unknown" }
  ],
  "default_value": "unspecified",
  "description": "Temporal reference of a verb form",
  "linguistic_relevance": "Arabic has two base tenses (past/present) plus imperative. The present/imperfect has mood distinctions.",
  "applies_to_pos": ["verb"],
  "depends_on": "pos == verb",
  "mutually_exclusive_with": [],
  "requires_feature": ["pos"],
  "encoding": [
    { "value_name": "past", "binary_code": 0, "notes": "Perfect tense; no mood distinction" },
    { "value_name": "present", "binary_code": 1, "notes": "Imperfect tense; can have mood" },
    { "value_name": "imperative", "binary_code": 2, "notes": "Derived from jussive" },
    { "value_name": "unspecified", "binary_code": 3, "notes": "Fallback value" }
  ],
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

### 19.3 Verb Form Feature: Form II

```json
{
  "id": "KB-0007:verb_form:II",
  "feature_name": "verb_form",
  "category": "derivational",
  "bitfield_position": 18,
  "bitfield_width": 5,
  "values": [
    {
      "name": "II",
      "code": 2,
      "label": "Form II",
      "description": "Intensive/causative: فَعَّلَ pattern",
      "arabic_term": "باب التفعيل"
    }
  ],
  "default_value": "not_a_verb",
  "description": "Verb form (measure/وزن) — the derivational pattern of a verb stem",
  "linguistic_relevance": "Each verb form has characteristic meaning modifications: causative (II, IV), reciprocal (III, VI), reflexive (V, VII, VIII), etc.",
  "applies_to_pos": ["verb"],
  "depends_on": "pos == verb",
  "mutually_exclusive_with": ["noun_type", "pronoun_type"],
  "requires_feature": ["pos"],
  "encoding": [
    { "value_name": "not_a_verb", "binary_code": 0, "notes": "Default for non-verbs" },
    { "value_name": "I", "binary_code": 1, "notes": "Base form" },
    { "value_name": "II", "binary_code": 2, "notes": "فَعَّلَ — intensive/causative" },
    { "value_name": "III", "binary_code": 3, "notes": "فَاعَلَ — reciprocal/attemptive" },
    { "value_name": "IV", "binary_code": 4, "notes": "أَفْعَلَ — causative" },
    { "value_name": "V", "binary_code": 5, "notes": "تَفَعَّلَ — reflexive of II" },
    { "value_name": "VI", "binary_code": 6, "notes": "تَفَاعَلَ — reciprocal of III" },
    { "value_name": "VII", "binary_code": 7, "notes": "اِنْفَعَلَ — passive/reflexive of I" },
    { "value_name": "VIII", "binary_code": 8, "notes": "اِفْتَعَلَ — reflexive of I" },
    { "value_name": "IX", "binary_code": 9, "notes": "اِفْعَلَّ — colors/defects" },
    { "value_name": "X", "binary_code": 10, "notes": "اِسْتَفْعَلَ — requestive" },
    { "value_name": "XI", "binary_code": 11, "notes": "اِفْعَالَّ — intensive colors" },
    { "value_name": "XII", "binary_code": 12, "notes": "اِفْعَوْعَلَ — intensive" },
    { "value_name": "XIII", "binary_code": 13, "notes": "اِفْعَوَّلَ — rare" },
    { "value_name": "XIV", "binary_code": 14, "notes": "Reserved (very rare)" },
    { "value_name": "XV", "binary_code": 15, "notes": "اِفْعَنْلَلَ — very rare" }
  ],
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

### 19.4 Agreement Rule: Subject-Verb

```json
{
  "id": "AGR-001",
  "rule_type": "subject_verb",
  "description": "Verb agrees with subject in person, number, and gender",
  "source_feature": "subject.person, subject.number, subject.gender",
  "target_feature": "verb.person, verb.number, verb.gender",
  "constraint": "verb.person == subject.person AND verb.number == subject.number AND verb.gender == subject.gender",
  "exceptions": "In SVO order, 3fs verb can be used with feminine plural subjects. In VSO order, verb is always singular, agrees only in gender.",
  "applies_to": ["verb"]
}
```

### 19.5 Full Feature Bitfield Example: كَتَبَ (kataba, "he wrote")

```yaml
Token: كَتَبَ (kataba)
Features:
  pos: verb (0)
  gender: masculine (0)
  number: singular (0)
  person: third (2)
  tense: past (0)
  mood: unspecified (3)      # past tense has no mood; 2-bit field allows code 3
  voice: active (0)
  case: unspecified (0)      # verbs don't have case; bit unused, set to 0
  state: definite (0)         # verbs don't have state; bit unused, set to 0
  verb_form: I (1)
  noun_type: not_a_noun (0)
  pronoun_type: not_a_pronoun (0)
  transitivity: transitive_1 (2)
  root_type: sound (1)
  stress_pattern: penultimate (2)  # ka-TA-ba
  syllable_count: 3
  has_shadda: false (0)
  has_madd: false (0)
  has_hamza: false (0)

Bitfield computation:
  pos(0)       | gender(0)<<4  | number(0)<<6  | person(2)<<8   |
  tense(0)<<10 | mood(3)<<12   | voice(0)<<14  | case(0)<<15    |
  state(0)<<17 | verb_form(1)<<18                               |
  noun_type(0)<<23 | pronoun_type(0)<<28                        |
  transitivity(2)<<32 | root_type(1)<<36                        |
  stress(2)<<40 | syllable_count(3)<<43 | has_shadda(0)<<47     |
  has_madd(0)<<48 | has_hamza(0)<<49                            |

Result (by byte, little-endian):
  Byte 0 (bits 0–7):  0000 0010  = person(2) @ bit 8 spills to byte 1 → 0b0000_0010
  Byte 1 (bits 8–15): 0000 0011  = person(2)<<0 + tense(0)<<2 + mood(3)<<4 + voice(0)<<6 + case(0)<<7 = 0b0000_0011  (3)
  Byte 2 (bits 16–23): 0000 0001  = state(0)<<1 + verb_form(1)<<2 = 0b0000_0001  (4)
  Byte 3 (bits 24–31): 0000 0000  = 0
  Byte 4 (bits 32–39): 0000 0001  = transitivity(2) + root_type(1)<<4 = 0b0001_0010  (0x12)
  Byte 5 (bits 40–47): 0000 0010  = stress(2)<<0 + syllable_count(3)<<3 + has_shadda(0)<<7 = 0b0001_1010  (0x1A)
  Byte 6 (bits 48–55): 0000 0000  = has_madd(0) + has_hamza(0)<<1 = 0
  Byte 7 (bits 56–63): 0000 0000  = reserved, must be 0

  Packed u64: 0x0000_0000_001A_1203
```

---

## 20. Cross-References

### 20.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0007 in module catalog; feature taxonomy |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Feature loading (Step 1.7), feature validation |
| SPEC-0001-C5 | Data Flow & IR (IR-4) | Features in intermediate representation |
| SPEC-0001-C8 | Security, Validation & Error Handling | Feature validation during parsing |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0007 size (1–2 MB), lookup performance |
| RFC-0001 | Grammar DSL | Feature names used in DSL expressions |
| RFC-0002 | Grammar Bytecode Format | 64-bit feature bitfield (entire Section 10 aligned) |
| RFC-0003 | Grammar Virtual Machine | FEATURE_EXTRACT instruction |
| KB-0001 | Roots Database | root_type, transitivity features |
| KB-0002 | Wazan Database | verb_form, noun_type features |
| KB-0003 | Verb Forms | tense, mood, voice, person, number, gender |
| KB-0004 | Noun Patterns | noun_type, case, state, gender, number |
| KB-0005 | Particles | pos (particle), grammatical function |
| KB-0006 | Pronouns | pos (pronoun), pronoun_type, person, number, gender |

### 20.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (8th C. CE) | Foundational Arabic grammar; defines the feature system indirectly |
| Ibn Jinni, Al-Khasa'is (10th C. CE) | Phonological and morphological feature theory |
| Wright's Arabic Grammar (1859) | Western reference for Arabic feature system |
| Ryding, A Reference Grammar of MSA (2005) | Contemporary reference for MSA feature usage |
| UNICODE Standard, Arabic Block (U+0600–U+06FF) | Orthographic feature reference |

### 20.3 RFC Compliance Summary

```yaml
rfc_compliance:
  RFC-0002: "§Feature Bitfield"
    status: "aligned"
    notes: |
      All bit positions and value encodings match RFC-0002 exactly.
      RFC-0002 must be updated to reflect KB-0007's pronoun_type extension
      (values 3–7) before KB-0006 v1.0.0 release.

  RFC-0003: "§Feature Extraction"
    status: "referenced"
    notes: |
      Feature IDs used in FEATURE_EXTRACT instruction must match
      KB-0007 feature_id values.

  RFC-0001: "§Morphological Features"
    status: "referenced"
    notes: |
      Feature names in DSL expressions (e.g., `$features.gender`)
      must match KB-0007 feature_name values.
```

---

## Progress Summary

**KB-0007: Morphological Features — Taxonomy, Bitfield Encoding & Inference Rules**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Morphological Features in Arabic | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Feature Entry Schema | ✓ COMPLETE |
| Section 5 | Part of Speech (POS) | ✓ COMPLETE |
| Section 6 | Inflectional Features | ✓ COMPLETE |
| Section 7 | Derivational Features | ✓ COMPLETE |
| Section 8 | Prosodic Features | ✓ COMPLETE |
| Section 9 | Orthographic Features | ✓ COMPLETE |
| Section 10 | Feature Bitfield Encoding (RFC-0002) | ✓ COMPLETE |
| Section 11 | Feature Agreement Rules | ✓ COMPLETE |
| Section 12 | Feature Inference Rules | ✓ COMPLETE |
| Section 13 | Feature Validation | ✓ COMPLETE |
| Section 14 | Cross-Feature Constraints | ✓ COMPLETE |
| Section 15 | Feature Matching Algorithm | ✓ COMPLETE |
| Section 16 | Serialization & Storage | ✓ COMPLETE |
| Section 17 | Versioning & Evolution | ✓ COMPLETE |
| Section 18 | Quality Requirements | ✓ COMPLETE |
| Section 19 | Example Entries | ✓ COMPLETE |
| Section 20 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–9), RFC-0001, RFC-0002, RFC-0003, KB-0001 through KB-0006.

**Recommended next document:** KB-0007 is the final KB in the AGOS core specification. The next phase is RFC-0003 (Grammar Virtual Machine) which builds on the feature bitfield encoding defined here.
