---
kb_id: KB-0002
title: Wazan Database — Morphological Patterns
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-14
updated: 2026-07-14
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0002)
  - SPEC-0001-C3: Compilation Pipeline (MOD-04 MorphologicalParser)
  - SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-04, MOD-08)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-4, IR-8)
  - SPEC-0001-C6: Deployment & Runtime Considerations (KB Bundling)
  - SPEC-0001-C8: Security, Validation & Error Handling (KB Integrity)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0401: Knowledge Graph Engine (planned)
  - KB-0001: Roots Database
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---


# KB-0002: Wazan Database — Morphological Patterns

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Wazan in Arabic Morphology](#2-wazan-in-arabic-morphology)
3. [Data Model](#3-data-model)
4. [Wazan Entry Schema](#4-wazan-entry-schema)
5. [Verb Form Wazan (I–XV)](#5-verb-form-wazan-i-xv)
6. [Derived Noun Wazan](#6-derived-noun-wazan)
7. [Quadriliteral Wazan](#7-quadriliteral-wazan)
8. [Weak Root Pattern Variants](#8-weak-root-pattern-variants)
9. [Wazan Matching Algorithm](#9-wazan-matching-algorithm)
10. [Serialization & Storage](#10-serialization--storage)
11. [Versioning & Evolution](#11-versioning--evolution)
12. [Quality Requirements](#12-quality-requirements)
13. [Example Entries](#13-example-entries)
14. [Cross-References](#14-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0002 is the **authoritative register of Arabic morphological patterns** (أوزان, `awzān` — sing. وزن, `wazn`) used by the AGOS platform. It provides the pattern-matching data that powers morphological analysis (MOD-04) and knowledge graph resolution (MOD-08).

Every Arabic word form is the result of applying a **wazan** (morphological pattern) to a **root** (jadhr). The wazan specifies:

- The sequence of consonants and vowels that make up the pattern.
- The positions where root consonants are inserted.
- The morphological features (part of speech, transitivity, etc.) associated with the pattern.
- The conjugation class and behavior of the resulting word.

KB-0002 answers: **"What pattern produced this word form? What are its linguistic properties?"**

### 1.2 Scope

KB-0002 covers:

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Verb patterns** | Forms I–XV (triliteral), Quadriliteral Forms I–III | Rare/archaic forms beyond standard 15 (covered by KB plugins) |
| **Noun patterns** | Masdar, ism fāʿil, ism mafʿūl, ism makān, ism zamān, ism ālah, ṣifah, tafḍīl, nisbah | Dialectal patterns (covered by KB plugins) |
| **Root types** | Sound, Weak, Hamzated, Doubled (with pattern variants for each) | N/A |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal Arabic |
| **Conjugation** | Pattern-to-form mapping (vowel templates, affix positions) | Full conjugation tables (covered by KB-0003) |

### 1.3 Target Audience

- **AGOS Pipeline:** MOD-04 (MorphologicalParser) reads KB-0002 during wazan identification. MOD-08 (KnowledgeGraphResolver) reads KB-0002 during pattern resolution.
- **Linguists & Data Maintainers:** Edit and extend KB-0002 with new patterns or pattern variants.
- **Plugin Authors:** KB-0002 serves as the data source for `morphology_engine` plugins that extend or modify pattern matching (e.g., alternative wazan matching for specific schools).

### 1.4 Relationship to Other KBs

```diff
  KB-0001: Roots (جذور)               — The consonants that carry lexical meaning
    │
    ├──► KB-0002: Wazan (أوزان)       ◄── This document (Patterns that specify form)
    │         │
    │         ├──► Verb Forms (I–XV)      — Defined by verb wazan patterns
    │         └──► Noun Patterns          — Defined by noun wazan patterns
    │
    ├──► KB-0003: Verb Forms (تصريف)   — Full conjugation paradigms for each form
    ├──► KB-0004: Noun Patterns (أوزان) — Detailed noun pattern specifications
    ├──► KB-0005: Particles             — Particles have no roots or wazans
    ├──► KB-0006: Pronouns              — Pronouns have no roots or wazans
    └──► KB-0007: Morphological Feat.   — Feature values inferred from wazan
```

The relationship between root and wazan is the **central generative mechanism** of Arabic morphology:

```
Root (ك-ت-ب) + Wazan (فَاعَلَ) = Word (كَاتَبَ — "he corresponded")
Root (د-ر-س) + Wazan (مَفْعَل) = Word (مَدْرَس — "school")
```

---

## 2. Wazan in Arabic Morphology

### 2.1 Definition

A **wazan** (وزن, pl. أوزان, `awzān`) is a morphological template that specifies the phonological shape of a word derived from a root. The term literally means "weight" or "measure" in Arabic — the wazan is the pattern that the root is "weighed against."

The prototypical triliteral wazan uses the consonants **ف-ع-ل** (f-ʿ-l, from the root فَعَلَ meaning "to do") as placeholder consonants representing the three root positions:

| Position | Placeholder | Meaning |
|----------|-------------|---------|
| First radical | ف (f) | C₁ — first consonant of any root |
| Second radical | ع (ʿ) | C₂ — second consonant |
| Third radical | ل (l) | C₃ — third consonant |

For quadriliteral patterns, the wazan uses **ف-ع-ل-ل** (f-ʿ-l-l) as placeholders, or sometimes **ف-عْلَل** for patterns where the fourth radical behaves differently.

### 2.2 How Wazan Works

A wazan specifies:

1. **The consonantal skeleton** — which positions take root consonants and which take pattern-specific consonants (e.g., prefixes like ت, suffixes like ي, infixes like ا).
2. **The vowel template** — which short vowels (a, i, u) appear between consonants and in what order.
3. **Gemination** — which consonants are doubled (carry shadda).
4. **Elongation** — which vowels are long (ā, ī, ū).
5. **Affix positions** — where additional morphemes (prefixes, suffixes, infixes) appear beyond the root consonants.

#### Example: Form I Active Verb (فَعَلَ)

```
Pattern:  فَعَلَ
          C₁ a C₂ a C₃ a

Applied to root ك-ت-ب (k-t-b):
          كَتَبَ
          kataba — "he wrote"
```

#### Example: Form VIII Reflexive Verb (اِفْتَعَلَ)

```
Pattern:  اِفْتَعَلَ
          i C₁ t a C₂ a C₃ a

Applied to root ك-ت-ب (k-t-b):
          اِكْتَتَبَ
          iktataba — "he copied/recorded"
          
Note: The infix ت (t) is inserted after the first consonant.
If C₁ is a coronal consonant, the ت may assimilate (e.g., اِصْطَبَرَ from ص-ب-ر).
```

#### Example: Noun of Place (مَفْعَل)

```
Pattern:  مَفْعَل
          ma C₁ C₂ a C₃

Applied to root ك-ت-ب (k-t-b):
          مَكْتَب
          maktab — "desk/office"
          
Note: The prefix م (m) indicates a noun of place/instrument.
```

### 2.3 The Wazan Taxonomy

Arabic wazans can be organized hierarchically:

```
All Wazans
├── Verb Wazans (أوزان الأفعال)
│   ├── Triliteral (I–XV)
│   │   ├── Form I:     فَعَلَ, فَعِلَ, فَعُلَ
│   │   ├── Form II:    فَعَّلَ
│   │   ├── Form III:   فَاعَلَ
│   │   ├── Form IV:    أَفْعَلَ
│   │   ├── Form V:     تَفَعَّلَ
│   │   ├── Form VI:    تَفَاعَلَ
│   │   ├── Form VII:   اِنْفَعَلَ
│   │   ├── Form VIII:  اِفْتَعَلَ
│   │   ├── Form IX:    اِفْعَلَّ
│   │   ├── Form X:     اِسْتَفْعَلَ
│   │   ├── Form XI:    اِفْعَالَّ
│   │   ├── Form XII:   اِفْعَوْعَلَ
│   │   ├── Form XIII:  اِفْعَوَّلَ
│   │   ├── Form XIV:   اِفْعَنْلَلَ
│   │   └── Form XV:    اِفْعَنْلَى
│   └── Quadriliteral
│       ├── Form I:     فَعْلَلَ
│       ├── Form II:    تَفَعْلَلَ
│       └── Form III:   اِفْعَنْلَلَ
│
├── Noun Wazans (أوزان الأسماء)
│   ├── Verbal Noun (مصدر)
│   │   ├── Form I:     فَعْل, فِعْل, فُعْل, فَعَل, فَعَال, فِعَال, etc. (~40+ patterns)
│   │   ├── Form II:    تَفْعِيل
│   │   ├── Form III:   فِعَال, مُفَاعَلَة
│   │   ├── Form IV:    إِفْعَال
│   │   ├── Form V:     تَفَعُّل
│   │   ├── Form VI:    تَفَاعُل
│   │   ├── Form VII:   اِنْفِعَال
│   │   ├── Form VIII:  اِفْتِعَال
│   │   ├── Form IX:    اِفْعِلَال
│   │   └── Form X:     اِسْتِفْعَال
│   ├── Active Participle (اسم فاعل)
│   │   ├── Triliteral: فَاعِل
│   │   └── Nontriliteral: مُ{form_prefix}عِل
│   ├── Passive Participle (اسم مفعول)
│   │   ├── Triliteral: مَفْعُول
│   │   └── Nontriliteral: مُ{form_prefix}عَل
│   ├── Noun of Place/Time (اسم مكان/زمان): مَفْعَل, مَفْعِل
│   ├── Noun of Instrument (اسم آلة): مِفْعَل, مِفْعَال, مِفْعَلَة
│   ├── Adjective (صفة مشبهة): فَعِيل, فَعْل, فَعِل, أَفْعَل, فَعْلَان
│   ├── Elative (تفضيل): أَفْعَل
│   ├── Relative Adjective (نسبة): فَعْلِيّ, فِعْلِيّ
│   └── ... (additional rare patterns)
│
└── Particle Wazans (أوزان الحروف)
    └── (Particles are fixed forms, not derived from roots — covered by KB-0005)
```

### 2.4 Why Wazan Is Central

The wazan system is the generative engine of Arabic morphology:

1. **Productivity.** Every root can potentially be used with every verb form wazan (though not all are attested). Understanding the wazan system allows AGOS to analyze novel combinations.

2. **Semantic derivation.** Each wazan carries a core semantic modification: Form II adds intensification/causation, Form III adds reciprocity, Form X adds request, etc. This enables semantic inference.

3. **Pattern-based parsing.** Rather than listing every possible word form, AGOS uses wazans to analyze stems algorithmically. This is both more maintainable (fewer entries) and more powerful (can analyze unseen words).

4. **Pedagogical value.** Arabic grammar pedagogy is organized around the wazan system. The Explanation Engine (MOD-11) can teach morphology through wazan analysis.

---

## 3. Data Model

### 3.1 Logical Data Model

```yaml
Wazan Database (KB-0002)
├── Metadata
│   ├── kb_id: "KB-0002"
│   ├── version: "2.0.1"
│   ├── pattern_count: integer
│   ├── verb_pattern_count: integer
│   ├── noun_pattern_count: integer
│   ├── created_at: timestamp
│   ├── sources: string[]
│   └── checksum_sha256: string
│
├── Verb Patterns: WazanEntry[]     # Verb form wazans
├── Noun Patterns: WazanEntry[]     # Derived noun wazans
└── Pattern Classes: PatternClass[] # Named classes (sound, hollow, defective, etc.)
```

### 3.2 Storage Model

KB-0002 is stored in two formats:

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~50 MB uncompressed | Human-readable |
| **Compiled (Hash Index)** | Production pipeline | ~10–40 MB | Memory-mapped O(1) lookup |

The **source format** is the canonical representation. The **compiled format** is an optimized hash index of pattern signatures, generated by `agos kb compile`.

### 3.3 Pattern Count Target

| Category | Estimated Count | Notes |
|----------|----------------|-------|
| Verb form patterns (I–XV) | ~30–40 | Base patterns + vowel variants (Form I has 3 vowel templates) |
| Derived noun patterns | ~80–120 | Masdar patterns (40+), participle patterns (10+), place/time (5+), instrument (5+), adjective (20+), etc. |
| Weak root variants | ~150–200 | Adjusted patterns for ajwaf, naqis, mithal, doubled, hamzated roots |
| Quadriliteral patterns | ~15–25 | Verb forms I–III + noun patterns |
| Pattern class templates | ~20–30 | Generalized templates with substitution rules |
| **Total (Version 1.0)** | **~300–450** | Covers all productive patterns for Classical Arabic and MSA |

---

## 4. Wazan Entry Schema

### 4.1 Schema Definition

```yaml
WazanEntry:
  # --- Identity ---
  id: string                           # Unique ID: "KB-0002:{pattern_type}:{canonical_form}"
                                       # e.g., "KB-0002:verb:I:فَعَلَ"
  pattern_type: PatternType            # Verb, noun, or particle
  canonical_pattern: string            # Canonical wazan form with ف-ع-ل placeholders
                                       # e.g., "فَعَلَ" for Form I
  pattern_script: string               # Full Arabic script pattern
                                       # e.g., "فَعَلَ" (same as canonical for basic)

  # --- Classification ---
  verb_form: integer | null            # I–XV for verb patterns, null for noun patterns
  noun_type: NounType | null           # For noun patterns (masdar, ism_fail, etc.)
  subclass: string | null              # e.g., "basic", "variant_a", "hollow"
  root_type_applicability: RootType[]  # Which root types this pattern applies to
                                       # e.g., ["sound_triliteral", "ajwaf_wawi"]

  # --- Phonological Template ---
  template: PhonologicalTemplate       # The detailed phonological structure

  # --- Morphological Features ---
  inherent_features: FeatureMap        # Features this pattern inherently assigns
                                       # e.g., for فَاعِل (ism fāʿil): POS=noun,
                                       #   noun_type=ism_fail, voice=active

  # --- Morphological Behavior ---
  conjugation_class: string | null     # For verb patterns: "sound", "hollow",
                                       # "defective", "doubled", "hamzated"
  transitivity_default: string | null  # "transitive" | "intransitive" | "ditransitive" |
                                       # "both" | null (inherits from KB-0001)

  # --- Semantics ---
  core_meaning: string                 # Pattern meaning in English
  core_meaning_ar: string              # Pattern meaning in Arabic
  semantic_modification: string        # How this pattern modifies the root's meaning
                                       # e.g., "causative", "intensive", "reflexive"

  # --- Examples ---
  examples: Example[]                  # Concrete examples of this pattern

  # --- Attestation ---
  attestation: Attestation

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string
```

### 4.2 Supporting Types

```yaml
PatternType:
  "verb" | "noun" | "particle"

NounType:
  "masdar" | "ism_fail" | "ism_maful" | "ism_makan" | "ism_zaman" |
  "ism_alah" | "sifah_mushabbahah" | "tafdil" | "nisbah" | "jam_taksir" |
  "ism_marrati" | "ism_hayati"

PhonologicalTemplate:
  segments: Segment[]                  # Ordered list of phonological segments
  vowel_pattern: string[]              # Vowel sequence (a, i, u, ā, ī, ū)
  consonant_count: integer             # Total consonant positions
  root_position_map: RootPositionMap   # Maps wazan slots to root consonant positions
  template_script: string              # Full template in Arabic, e.g., "C₁aC₂aC₃a"

Segment:
  type: "root_consonant" | "prefix" | "suffix" | "infix" | "vowel" |
        "gemination_marker" | "long_vowel_marker"
  position: integer | null             # Slot position within the pattern
  character: string | null             # Fixed character (for affixes)
  slot_label: string | null            # e.g., "C₁", "C₂", "C₃", "PREF_T", "SUFF_W"

RootPositionMap:
  mappings: RootPositionMapping[]      # How template slots map to root consonants

RootPositionMapping:
  template_slot: integer               # Position in template (1-based)
  root_position: integer               # Position in root (1-based: 1, 2, 3 for triliteral)
  is_geminated: boolean                # Whether this consonant is doubled (shadda)

Example:
  word: string                         # Example word in Arabic
  transliteration: string              # Latin transliteration
  meaning: string                      # English meaning
  root: string                         # Root used in this example
  context: string | null               # Usage context or citation

FeatureMap:
  features: Feature[]                  # Key-value feature pairs

Feature:
  name: string                         # e.g., "pos", "voice", "noun_type"
  value: string                        # e.g., "verb", "active", "ism_fail"

Attestation:
  confidence: "certain" | "well_attested" | "attested" | "disputed"
  primary_sources: string[]            # Source references
  classical_references: string[]       # Grammar book references
  notes: string | null

PatternClass:
  id: string                           # e.g., "hollow_wawi"
  name: string                         # e.g., "Hollow (Ajwaf Wawi)"
  description: string                  # Behavioral description
  applies_to: RootType[]               # Which root types use this class
  substitution_rules: SubstitutionRule[] # How the base pattern changes

SubstitutionRule:
  condition: string                    # When this rule applies
  operation: string                    # What changes (e.g., "medial_waw→alif")
  result_template: string              # The resulting pattern form
```

### 4.3 JSON Example (Verb Form I Active, Sound)

```json
{
  "id": "KB-0002:verb:I:فَعَلَ",
  "pattern_type": "verb",
  "canonical_pattern": "فَعَلَ",
  "pattern_script": "فَعَلَ",
  "verb_form": 1,
  "noun_type": null,
  "subclass": "basic_a",
  "root_type_applicability": ["sound_triliteral", "mahmuz_al_ayn", "mahmuz_al_lam"],
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" },
      { "type": "vowel", "character": "a" }
    ],
    "vowel_pattern": ["a", "a", "a"],
    "consonant_count": 3,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 2, "root_position": 2, "is_geminated": false },
        { "template_slot": 3, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "C₁aC₂aC₃a"
  },
  "inherent_features": {
    "features": [
      { "name": "pos", "value": "verb" },
      { "name": "voice", "value": "active" },
      { "name": "tense", "value": "past" },
      { "name": "form", "value": "I" }
    ]
  },
  "conjugation_class": "sound",
  "transitivity_default": null,
  "core_meaning": "Basic action (active, past tense)",
  "core_meaning_ar": "الفعل الماضي المبني للمعلوم",
  "semantic_modification": "No modification — this is the base form from which other forms derive",
  "examples": [
    {
      "word": "كَتَبَ",
      "transliteration": "kataba",
      "meaning": "he wrote",
      "root": "كتب",
      "context": "Quran 96:4-5"
    },
    {
      "word": "جَلَسَ",
      "transliteration": "jalasa",
      "meaning": "he sat",
      "root": "جلس"
    },
    {
      "word": "دَخَلَ",
      "transliteration": "dakhala",
      "meaning": "he entered",
      "root": "دخل"
    }
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": [
      "Al-Kitab (Sibawayh)",
      "Sharh al-Ashmuni",
      "Qatr al-Nada (Ibn Hisham)"
    ]
  },
  "created_at": "2026-07-14T00:00:00Z",
  "updated_at": "2026-07-14T00:00:00Z",
  "version_added": "1.0.0"
}
```

---

## 5. Verb Form Wazan (I–XV)

### 5.1 Complete Verb Form Table

The following table lists all 15 verb forms with their canonical wazans for **sound triliteral roots**. Each form is defined by its characteristic prefix, infix, suffix, and vowel pattern.

| Form | Wazan | Template (Sound) | Prefix | Stem Pattern | Meaning Category |
|------|-------|-------------------|--------|--------------|------------------|
| **I** | فَعَلَ | C₁aC₂aC₃a | — | C₁aC₂aC₃a | Basic action |
| **I** | فَعِلَ | C₁aC₂iC₃a | — | C₁aC₂iC₃a | Basic action (vowel variant) |
| **I** | فَعُلَ | C₁aC₂uC₃a | — | C₁aC₂uC₃a | Quality/stative (vowel variant) |
| **II** | فَعَّلَ | C₁aC₂C₂aC₃a | — | C₁aC₂C₂aC₃a | Intensive/causative |
| **III** | فَاعَلَ | C₁āC₂aC₃a | — | C₁āC₂aC₃a | Attemptive/reciprocal |
| **IV** | أَفْعَلَ | aC₁C₂aC₃a | أَ | aC₁C₂aC₃a | Causative/declarative |
| **V** | تَفَعَّلَ | taC₁aC₂C₂aC₃a | تَ | taC₁aC₂C₂aC₃a | Reflexive of II |
| **VI** | تَفَاعَلَ | taC₁āC₂aC₃a | تَ | taC₁āC₂aC₃a | Reciprocal/reflexive of III |
| **VII** | اِنْفَعَلَ | inC₁aC₂aC₃a | اِنْ | inC₁aC₂aC₃a | Passive/reflexive |
| **VIII** | اِفْتَعَلَ | iC₁taC₂aC₃a | اِ | C₁taC₂aC₃a | Reflexive (with infix ت) |
| **IX** | اِفْعَلَّ | iC₁C₂aC₃C₃a | اِ | iC₁C₂aC₃C₃a | Colors/defects |
| **X** | اِسْتَفْعَلَ | istaC₁C₂aC₃a | اِسْتَ | istaC₁C₂aC₃a | Requestive/deemed |
| **XI** | اِفْعَالَّ | iC₁C₂āC₃C₃a | اِ | iC₁C₂āC₃C₃a | Intensive color (rare) |
| **XII** | اِفْعَوْعَلَ | iC₁C₂awC₃aC₃a | اِ | iC₁C₂awC₃aC₃a | Intensive (rare) |
| **XIII** | اِفْعَوَّلَ | iC₁C₂awwaC₃a | اِ | iC₁C₂awwaC₃a | Very rare |
| **XIV** | اِفْعَنْلَلَ | iC₁C₂anC₃aC₃a | اِ | iC₁C₂anC₃aC₃a | Very rare |
| **XV** | اِفْعَنْلَى | iC₁C₂anC₃ā | اِ | iC₁C₂anC₃ā | Very rare |

### 5.2 Form I Vowel Variants

Form I has **three vowel patterns** for the perfect stem:

| Variant | Perfect Template | Imperfect Vowel | Semantic Tendency | Example |
|---------|-----------------|-----------------|-------------------|---------|
| فَعَلَ | C₁aC₂aC₃a | u / i | Transitive action | كَتَبَ (yaktubu) — to write |
| فَعِلَ | C₁aC₂iC₃a | a | Intransitive/state | جَلِسَ (yajlasu) — to sit |
| فَعُلَ | C₁aC₂uC₃a | u | Quality/stative | حَسُنَ (yaḥsunu) — to be beautiful |

Each vowel variant is a **distinct sub-entry** in KB-0002 (e.g., `KB-0002:verb:I:فَعَلَ`, `KB-0002:verb:I:فَعِلَ`, `KB-0002:verb:I:فَعُلَ`).

### 5.3 Form-by-Form Details

#### Form I: فَعَلَ / فَعِلَ / فَعُلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | فَعَلَ (primary), فَعِلَ, فَعُلَ (variants) |
| **Template** | C₁aC₂(a/i/u)C₃a |
| **Prefix** | None |
| **Suffix in perfect** | (none) — the pattern is already the 3ms perfect |
| **Imperfect stem** | C₁C₂(u/i/a)C₃ (vowel unpredictable; stored in KB-0001 verb_form_details) |
| **Masdar pattern** | Variable (~40+ patterns, e.g., فَعْل, فِعْل, فُعْل, فَعَل, فِعَال) |
| **Active participle** | فَاعِل |
| **Passive participle** | مَفْعُول |
| **Semantic range** | Broad — covers basic actions, states, and qualities |
| **Frequency** | Most common form (~60% of Arabic verbs) |

#### Form II: فَعَّلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | فَعَّلَ |
| **Template** | C₁aC₂C₂aC₃a |
| **Distinguishing feature** | Geminated middle radical (C₂ carries shadda) |
| **Perfect marker** | C₂ gemination |
| **Imperfect** | يُفَعِّلُ (yufaʿʿilu) |
| **Masdar** | تَفْعِيل |
| **Semantic range** | Causative, intensive, denominative, declarative |
| **Example** | كَتَّبَ (kattaba) — "he caused (someone) to write" |

#### Form III: فَاعَلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | فَاعَلَ |
| **Template** | C₁āC₂aC₃a |
| **Distinguishing feature** | Long ā after C₁ |
| **Perfect marker** | Medial alif |
| **Imperfect** | يُفَاعِلُ (yufāʿilu) |
| **Masdar** | فِعَال or مُفَاعَلَة |
| **Semantic range** | Reciprocal, attemptive, conative |
| **Example** | كَاتَبَ (kātaba) — "he corresponded (with someone)" |

#### Form IV: أَفْعَلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | أَفْعَلَ |
| **Template** | aC₁C₂aC₃a |
| **Distinguishing feature** | Prefix أ (alif with hamza) |
| **Perfect marker** | Initial hamza |
| **Imperfect** | يُفْعِلُ (yufʿilu) |
| **Masdar** | إِفْعَال |
| **Semantic range** | Causative, declarative, estimative |
| **Example** | أَكْتَبَ (aktaba) — "he dictated" |

#### Form V: تَفَعَّلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | تَفَعَّلَ |
| **Template** | taC₁aC₂C₂aC₃a |
| **Distinguishing feature** | Prefix ت + geminated C₂ |
| **Perfect marker** | Initial ت |
| **Imperfect** | يَتَفَعَّلُ (yatafaʿʿalu) |
| **Masdar** | تَفَعُّل |
| **Semantic range** | Reflexive of Form II; gradual action; assumption |
| **Example** | تَكَتَّبَ (takattaba) — "he enrolled/registered" |

#### Form VI: تَفَاعَلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | تَفَاعَلَ |
| **Template** | taC₁āC₂aC₃a |
| **Distinguishing feature** | Prefix ت + long ā after C₁ |
| **Imperfect** | يَتَفَاعَلُ (yatafāʿalu) |
| **Masdar** | تَفَاعُل |
| **Semantic range** | Reciprocal of Form III; affected action |
| **Example** | تَكَاتَبَ (takātaba) — "they corresponded (with each other)" |

#### Form VII: اِنْفَعَلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | اِنْفَعَلَ |
| **Template** | inC₁aC₂aC₃a |
| **Distinguishing feature** | Prefix اِنْ (alif + nūn) |
| **Imperfect** | يَنْفَعِلُ (yanfaʿilu) |
| **Masdar** | اِنْفِعَال |
| **Semantic range** | Passive/reflexive; involuntary action |
| **Example** | اِنْكَتَبَ (inkataba) — "he subscribed" |

#### Form VIII: اِفْتَعَلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | اِفْتَعَلَ |
| **Template** | iC₁taC₂aC₃a |
| **Distinguishing feature** | Infix ت (t) after C₁; assimilates with coronal C₁ |
| **Imperfect** | يَفْتَعِلُ (yaftaʿilu) |
| **Masdar** | اِفْتِعَال |
| **Semantic range** | Reflexive; action for oneself |
| **Coronal assimilation** | If C₁ ∈ {ت, ث, د, ذ, ز, ص, ض, ط, ظ}, the ت assimilates to C₁ with gemination (e.g., اِصْطَبَرَ from ص-ب-ر) |
| **Example** | اِكْتَتَبَ (iktataba) — "he copied" |

#### Form IX: اِفْعَلَّ

| Property | Value |
|----------|-------|
| **Canonical wazan** | اِفْعَلَّ |
| **Template** | iC₁C₂aC₃C₃a |
| **Distinguishing feature** | Geminated C₃; no medial vowel |
| **Imperfect** | يَفْعَلُّ (yafʿallu) |
| **Masdar** | اِفْعِلَال |
| **Semantic range** | Colors, defects, physical qualities |
| **Note** | Only used with roots whose base meaning involves colors or defects |
| **Example** | اِحْمَرَّ (iḥmarra) — "it became red" |

#### Form X: اِسْتَفْعَلَ

| Property | Value |
|----------|-------|
| **Canonical wazan** | اِسْتَفْعَلَ |
| **Template** | istaC₁C₂aC₃a |
| **Distinguishing feature** | Prefix اِسْتَ (alif + sīn + tā) |
| **Imperfect** | يَسْتَفْعِلُ (yastafʿilu) |
| **Masdar** | اِسْتِفْعَال |
| **Semantic range** | Requestive, estimative, transformative |
| **Example** | اِسْتَكْتَبَ (istaktaba) — "he asked (someone) to write" |

#### Forms XI–XV (Rare)

| Form | Wazan | Template | Semantic | Frequency |
|------|-------|----------|----------|-----------|
| **XI** | اِفْعَالَّ | iC₁C₂āC₃C₃a | Intensive color/quality | Rare |
| **XII** | اِفْعَوْعَلَ | iC₁C₂awC₃aC₃a | Intensive | Very rare |
| **XIII** | اِفْعَوَّلَ | iC₁C₂awwaC₃a | Intensive | Very rare |
| **XIV** | اِفْعَنْلَلَ | iC₁C₂anC₃aC₃a | Intensive | Very rare |
| **XV** | اِفْعَنْلَى | iC₁C₂anC₃ā | Intensive | Very rare |

Forms XI–XV are included in KB-0002 for completeness but will have few attestations. They are marked with `frequency: "rare"` and may be skipped in compact KB builds.

---

## 6. Derived Noun Wazan

### 6.1 Verbal Noun (Masdar) Wazan

Each verb form has one or more associated masdar patterns. For Form I, the masdar pattern is unpredictable and must be stored per-root. For Forms II–XV, the masdar pattern is regular.

#### Regular Masdar Patterns (Forms II–XV)

| Verb Form | Masdar Wazan | Template | Example |
|-----------|--------------|----------|---------|
| **II** | تَفْعِيل | taC₁C₂īC₃ | تَكْتِيب (taktīb) |
| **III** | فِعَال / مُفَاعَلَة | C₁C₂āC₃ / muC₁āC₂aC₃a | كِتَاب (kitāb) / مُكَاتَبَة (mukātaba) |
| **IV** | إِفْعَال | iC₁C₂āC₃ | إِكْتَاب (iktāb) |
| **V** | تَفَعُّل | taC₁aC₂C₂uC₃ | تَكَتُّب (takattub) |
| **VI** | تَفَاعُل | taC₁āC₂uC₃ | تَكَاتُب (takātub) |
| **VII** | اِنْفِعَال | inC₁C₂āC₃ | اِنْكِتَاب (inkitāb) |
| **VIII** | اِفْتِعَال | iC₁C₂C₂āC₃ | اِكْتِتَاب (iktitāb) |
| **IX** | اِفْعِلَال | iC₁C₂C₂āC₃ | اِحْمِرَار (iḥmirār) |
| **X** | اِسْتِفْعَال | istiC₁C₂āC₃ | اِسْتِكْتَاب (istiktāb) |

#### Form I Masdar Patterns (Unpredictable)

Form I masdar patterns are numerous and largely unpredictable from the verb form alone. KB-0002 catalogs ~40+ attested patterns:

| Pattern | Template | Example | Frequency |
|---------|----------|---------|-----------|
| فَعْل | C₁aC₂C₃ | ضَرْب (ḍarb) — "hitting" | Very common |
| فِعْل | C₁iC₂C₃ | عِلْم (ʿilm) — "knowledge" | Common |
| فُعْل | C₁uC₂C₃ | حُسْن (ḥusn) — "beauty" | Common |
| فَعَل | C₁aC₂aC₃ | طَلَب (ṭalab) — "request" | Common |
| فَعَال | C₁aC₂āC₃ | سَلَام (salām) — "peace" | Common |
| فِعَال | C₁iC₂āC₃ | كِتَاب (kitāb) — "writing" | Common |
| فُعَال | C₁uC₂āC₃ | غُسَال (ghusāl) — "washing" | Less common |
| فَعِيل | C₁aC₂īC₃ | كَثِير (kathīr) — "much" | Less common |
| فُعُول | C₁uC₂ūC₃ | جُلُوس (julūs) — "sitting" | Common |
| فَعَالَة | C₁aC₂āC₃a | دَرَاسَة (dirāsa) — "study" | Common |
| فِعَالَة | C₁iC₂āC₃a | كِتَابَة (kitāba) — "writing" | Common |
| فَعُول | C₁aC₂ūC₃ | رَكُوب (rakūb) — "riding" | Less common |
| فَعِيلَة | C₁aC₂īC₃a | بَقِيع (baqīʿa) — "remnant" | Less common |
| ... | ... | ~25+ additional rare patterns | Rare |

The per-root masdar pattern is stored in KB-0001 (`derived_nouns`), while the pattern definitions are stored in KB-0002.

### 6.2 Active Participle (ʾIsm al-Fāʿil) Wazan

| Verb Form | Active Participle Wazan | Template | Example |
|-----------|------------------------|----------|---------|
| **I** (sound) | فَاعِل | C₁āC₂iC₃ | كَاتِب (kātib) |
| **II** | مُفَعِّل | muC₁aC₂C₂iC₃ | مُكَاتِب (mukātib) |
| **III** | مُفَاعِل | muC₁āC₂iC₃ | مُدَرِّس (mudarris) |
| **IV** | مُفْعِل | muC₁C₂iC₃ | مُكْتِب (muktib) |
| **V** | مُتَفَعِّل | mutaC₁aC₂C₂iC₃ | مُتَكَاتِب (mutakātib) |
| **VI** | مُتَفَاعِل | mutaC₁āC₂iC₃ | مُتَدَرِّس (mutadarras) |
| **VII** | مُنْفَعِل | munC₁aC₂iC₃ | مُنْكَاتِب (munkātib) |
| **VIII** | مُفْتَعِل | muC₁taC₂iC₃ | مُكْتَتِب (muktātib) |
| **IX** | مُفْعَل | muC₁C₂aC₃C₃ | مُحْمَر (muḥmar) |
| **X** | مُسْتَفْعِل | mustaC₁C₂iC₃ | مُسْتَكْتِب (mustaktib) |

### 6.3 Passive Participle (ʾIsm al-Mafʿūl) Wazan

| Verb Form | Passive Participle Wazan | Template | Example |
|-----------|-------------------------|----------|---------|
| **I** (sound) | مَفْعُول | maC₁C₂ūC₃ | مَكْتُوب (maktūb) |
| **II** | مُفَعَّل | muC₁aC₂C₂aC₃ | مُدَرَّس (mudarras) |
| **III** | مُفَاعَل | muC₁āC₂aC₃ | مُكَاتَب (mukātab) |
| **IV** | مُفْعَل | muC₁C₂aC₃ | مُكْتَب (muktab) |
| **V** | مُتَفَعَّل | mutaC₁aC₂C₂aC₃ | مُتَكَاتَب (mutakātab) |
| **VI** | مُتَفَاعَل | mutaC₁āC₂aC₃ | مُتَعَالَم (mutaʿālam) |
| **VII** | مُنْفَعَل | munC₁aC₂aC₃ | مُنْكَتَب (munkatab) |
| **VIII** | مُفْتَعَل | muC₁taC₂aC₃ | مُكْتَتَب (muktatab) |
| **IX** | (not used) | — | — |
| **X** | مُسْتَفْعَل | mustaC₁C₂aC₃ | مُسْتَكْتَب (mustaktab) |

### 6.4 Other Noun Wazan Patterns

| Noun Type | Wazan | Template | Meaning | Example |
|-----------|-------|----------|---------|---------|
| **Noun of Place** (ظرف مكان) | مَفْعَل | maC₁C₂aC₃ | Place where action occurs | مَكْتَب (maktab) — "office" |
| **Noun of Time** (ظرف زمان) | مَفْعِل | maC₁C₂iC₃ | Time when action occurs | مَوْعِد (mawʿid) — "appointment" |
| **Noun of Instrument** (اسم آلة) | مِفْعَل | miC₁C₂aC₃ | Tool for action | مِنْجَل (minjal) — "sickle" |
| **Noun of Instrument** | مِفْعَال | miC₁C₂āC₃ | Tool (alternative) | مِفْتَاح (miftāḥ) — "key" |
| **Noun of Instrument** | مِفْعَلَة | miC₁C₂aC₃a | Tool (alternative) | مِكْنَسَة (miknasa) — "broom" |
| **Resembling Adjective** (صفة مشبهة) | فَعِيل | C₁aC₂īC₃ | Permanent quality | كَرِيم (karīm) — "generous" |
| **Resembling Adjective** | فَعْل | C₁aC₂C₃ | Quality | ضَخْم (ḍakhm) — "huge" |
| **Resembling Adjective** | فَعْلَان | C₁aC₂C₃ān | Color/emotion | غَضْبَان (ghaḍbān) — "angry" |
| **Elative** (اسم تفضيل) | أَفْعَل | aC₁C₂aC₃ | Comparative/superlative | أَكْبَر (akbar) — "greater" |
| **Relative Adjective** (نسبة) | فَعْلِيّ | C₁aC₂C₃iyy | Relational adjective | عَرَبِيّ (ʿarabiyy) — "Arabic" |
| **Relative Adjective** | فِعْلِيّ | C₁iC₂C₃iyy | Relational (variant) | عِلْمِيّ (ʿilmiyy) — "scientific" |

---

## 7. Quadriliteral Wazan

### 7.1 Quadriliteral Verb Forms

Quadriliteral roots follow their own (simpler) wazan system with three main verb forms:

| Form | Wazan | Template | Meaning | Example |
|------|-------|----------|---------|---------|
| **QI** | فَعْلَلَ | C₁aC₂C₃aC₄a | Basic quadriliteral | زَلْزَلَ (zalzala) — "to shake" |
| **QII** | تَفَعْلَلَ | taC₁aC₂C₃aC₄a | Reflexive of QI | تَدَحْرَجَ (tadaḥraja) — "to roll" |
| **QIII** | اِفْعَنْلَلَ | iC₁C₂anC₃aC₄a | Reflexive/inchoative | اِحْرَنْجَمَ (iḥranjama) — "to crowd" |

### 7.2 Quadriliteral Noun Patterns

| Noun Type | Wazan | Template | Example |
|-----------|-------|----------|---------|
| Masdar (QI) | فَعْلَلَة | C₁aC₂C₃aC₄a | زَلْزَلَة (zalzala) — "earthquake" |
| Masdar (QII) | تَفَعْلُل | taC₁aC₂C₃uC₄ | تَمَعْدُن (tamaʿdun) — "mineralization" |
| Active participle | مُفَعْلِل | muC₁aC₂C₃iC₄ | مُزَلْزِل (muzalzil) — "shaker" |
| Passive participle | مُفَعْلَل | muC₁aC₂C₃aC₄ | مُزَلْزَل (muzalzal) — "shaken" |

### 7.3 Quadriliteral Pattern Notes

1. The placeholder **ف-ع-ل-ل** (f-ʿ-l-l) represents the four root consonants, with the fourth radical represented by the final ل (l).
2. Alternative placeholder: **ف-عْلَل** (f-ʿl-l) is sometimes used to emphasize that the pattern has a C₂+C₃ cluster.
3. Quadriliteral forms are less productive than triliteral forms. Many quadriliteral roots exist in only one verb form.
4. The masdar of QI typically is identical to the perfect stem: فَعْلَلَة (not فَعْلَل).

---

## 8. Weak Root Pattern Variants

### 8.1 Why Weak Roots Need Pattern Variants

Weak roots (those containing و, ي, or ا) undergo systematic phonological changes when combined with wazans. These changes include:

- **Assimilation:** Weak letters merge with adjacent vowels (e.g., medial و → ū).
- **Elision:** Weak letters drop in certain positions (e.g., final ي drops in jussive).
- **Substitution:** Weak letters change quality (e.g., و → ا in certain environments).

KB-0002 must store **pattern variants** for each weak root category, so MOD-04 can match input stems against the correct pattern variant.

### 8.2 Weak Pattern Variant Table

| Root Type | Base Pattern (Form I) | Actual Surface | Rule | Example |
|-----------|----------------------|----------------|------|---------|
| **Sound** | فَعَلَ → C₁aC₂aC₃a | كَتَبَ | No change | ك-ت-ب → كَتَبَ |
| **Mithal Wawi** (و-ج-د) | فَعَلَ → C₁aC₂aC₃a | وَجَدَ | No change in perfect | و-ج-د → وَجَدَ |
| **Mithal Wawi** (imperfect) | يَفْعُلُ → yaC₁C₂uC₃u | يَجِدُ | C₁ assimilates in imperfect | و-ج-د → يَجِدُ (not يَوْجِدُ) |
| **Ajwaf Wawi** (ق-و-ل) | فَعَلَ → C₁aC₂aC₃a | قَالَ | C₂ (w) → ā in perfect | ق-و-ل → قَالَ (not قَوَلَ) |
| **Ajwaf Wawi** (imperfect) | يَفْعُلُ → yaC₁C₂uC₃u | يَقُولُ | C₂ (w) → ū in imperfect | ق-و-ل → يَقُولُ |
| **Ajwaf Yai** (س-ي-ر) | فَعَلَ → C₁aC₂aC₃a | سَارَ | C₂ (y) → ā in perfect | س-ي-ر → سَارَ (not سَيَرَ) |
| **Naqis Wawi** (د-ع-و) | فَعَلَ → C₁aC₂aC₃a | دَعَا | C₃ (w) → ā word-final | د-ع-و → دَعَا (not دَعَوَ) |
| **Naqis Yai** (ر-م-ي) | فَعَلَ → C₁aC₂aC₃a | رَمَى | C₃ (y) → ā word-final | ر-م-ي → رَمَى (not رَمَيَ) |
| **Doubled** (م-د-د) | فَعَلَ → C₁aC₂aC₃a | مَدَّ | C₂=C₃ merge with shadda | م-د-د → مَدَّ (not مَدَدَ) |
| **Hamzated** (مهموز) | فَعَلَ → C₁aC₂aC₃a | أَكَلَ / سَأَلَ | Stable (hamzat al-qatʿ) OR seat change based on adjacent vowels | أ-ك-ل → أَكَلَ (stable) / س-أ-ل → سَأَلَ (seat on alif) / س-ؤ-ل → سُئِلَ (seat on waw) |

**Hamzated root note:** Hamzated roots are classified into two behaviors: (1) **hamzat al-qatʿ** (stable hamza, e.g., أ-ك-ل) where the hamza retains its shape regardless of phonological context; (2) **fluctuating hamza** (e.g., س-أ-ل) where the hamza's seat (أ, إ, ؤ, ئ) changes based on adjacent vowels. KB-0002 stores both behaviors as variant entries under a unified "hamzated" category, with the specific seat rule determined by surrounding vowels during template matching.

### 8.3 Pattern Variant Entry Schema

For each wazan that has weak root variants, KB-0002 stores variant records:

```yaml
PatternVariant:
  variant_name: string                   # e.g., "Form I Ajwaf Wawi (perfect)"
  base_pattern_id: string                # Reference to base pattern (e.g., "KB-0002:verb:I:فَعَلَ")
  applies_to: RootType[]                 # Which root types use this variant
  template: PhonologicalTemplate         # The modified template
  transformation: string                 # Description of the transformation
  transformation_rule: SubstitutionRule  # Formal rule definition
  examples: Example[]                    # Examples showing this variant
```

### 8.4 Example: Form I Ajwaf Wawi Variant

```json
{
  "variant_name": "Form I Ajwaf Wawi (perfect)",
  "base_pattern_id": "KB-0002:verb:I:فَعَلَ",
  "applies_to": ["ajwaf_wawi"],
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "long_vowel_marker", "character": "ā" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "vowel_pattern": ["ā"],
    "consonant_count": 2,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 2, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "C₁āC₃"
  },
  "transformation": "Medial waw (C₂) lengthens to ā; C₂ slot absorbed into vowel length",
  "transformation_rule": {
    "condition": "root_type == ajwaf_wawi AND pattern == Form I perfect",
    "operation": "C₂_waw → ā (medial vowel lengthening)",
    "result_template": "C₁āC₃"
  },
  "examples": [
    { "word": "قَالَ", "meaning": "he said", "root": "قول" },
    { "word": "كَانَ", "meaning": "he was", "root": "كون" },
    { "word": "زَارَ", "meaning": "he visited", "root": "زور" }
  ]
}
```

### 8.5 Weak Variant Coverage Requirements

| Root Type | Verbs Affected | KB-0002 Variant Entries Required |
|-----------|---------------|----------------------------------|
| Sound | Regular — no variants needed | 0 |
| Mithal (first weak) | ~200 roots | ~5 variant entries per verb form |
| Ajwaf (second weak) | ~500 roots | ~8 variant entries per verb form |
| Naqis (third weak) | ~400 roots | ~6 variant entries per verb form |
| Lafif (two weak) | ~50 roots | ~10 variant entries per verb form |
| Doubled | ~200 roots | ~3 variant entries per verb form |
| Hamzated (any position) | ~300 roots | ~4 variant entries per verb form |

**Total weak variant patterns needed: ~150–200 entries.**

---

## 9. Wazan Matching Algorithm

### 9.1 Core Matching Algorithm

The primary algorithm used by MOD-04 to determine which wazan produced a given stem:

```pseudo
Algorithm: match_wazan
Input: stem (string), root (RootEntry), verb_form_hint (integer | null)
Output: WazanMatch[]

1. Preprocessing:
   a. Strip any remaining clitics from the stem (should already be done by MOD-03).
   b. Identify the root consonants from KB-0001.
   c. Record stem length in consonants.

2. Determine candidate wazans:
   a. If verb_form_hint is provided:
      i.   Look up the specific form's wazan.
      ii.  If weak root variant exists → include both base and variant.
   b. If no verb_form_hint:
      i.   Check all verb wazans (I–XV) as candidates.
      ii.  Also check noun wazans as candidates.
      iii. Organize by probability: verb forms first, then common noun patterns.

3. For each candidate wazan w:
   a. Retrieve the phonological template.
   b. Map root consonants to template slots using root_position_map.
   c. Fill the template:
      i.   Insert root consonants at their mapped positions.
      ii.  Insert fixed pattern characters (prefixes, infixes, suffixes).
      iii. Apply vowel pattern.
      iv.  Apply gemination markers.
   d. Compare the filled template against the input stem.
   e. If match → record match details:
      i.   Wazan ID
      ii.  Pattern matching confidence
      iii. Which segments matched, which deviated
      iv.  Any phonological adjustments needed

4. If weak root variant exists for this wazan:
   a. Apply the variant's transformation rules.
   b. Generate the expected surface form.
   c. Compare against the stem.
   d. If match → record with higher confidence than base pattern
      (weak root patterns are more specific and thus more reliable).

5. Order matches by:
   a. Exact match (full template matches) → higher than partial.
   b. Specificity (weak root variant > generic pattern).
   c. Verb form frequency (Form I > Forms II–X > Forms XI–XV).
   d. Semantic congruity (does the pattern's semantic modification
      align with the root's semantic field).

6. Return ordered WazanMatch[] array (may be empty).
```

### 9.2 Pattern Signature Hashing

For fast lookup, KB-0002 uses a **pattern signature** — a compact numerical representation of the phonological template:

```pseudo
Algorithm: compute_pattern_signature
Input: wazan (WazanEntry)
Output: signature (u64)

1. consonant_positions = bitmap of consonant positions (prefix, infix, suffix, root)
   Bits 0–15:   Root consonant mapping (C₁, C₂, C₃, C₄ positions)
   Bits 16–23:  Affix presence (prefix_alif, prefix_ta, prefix_sin, etc.)
   Bits 24–31:  Vowel pattern (a=00, i=01, u=10, long=11 for each vowel position)

2. Return u64 signature.

Algorithm: match_by_signature
Input: stem (string), target_signature (u64)
Output: WazanEntry[] (candidates)

1. Extract stem characteristics:
   a. Consonantal skeleton (remove vowels, keep consonants)
   b. Position of each consonant
   c. Vowel sequence

2. Compare against KB-0002 signature index:
   a. Hash-indexed lookup by consonant count and affix presence.
   b. Returns all wazans with matching structural properties.

3. For each candidate, run detailed template matching (Section 9.1, Step 3).

4. Return ordered candidates.
```

**Complexity:** O(1) for hash-indexed lookup; O(c × t) for template matching where c = candidate count, t = template length.

### 9.3 Handling Ambiguity

Some stems may match multiple wazans:

| Stem | Possible Wazan(s) | Root | Resolution |
|------|-------------------|------|------------|
| سَامَ | Form I: سَامَ (sāma — "he negotiated") | س-و-م | Weak root (ajwaf wawi) variant of Form I |
| سَامَ | Form III: سَامَ (sāma — "he tried/resisted") ? | س-ي-د? | Possible if root س-ي-د; less likely |
| مَدَّ | Form I: مَدَّ (madda — "he extended") | م-د-د | Doubled root — C₂ and C₃ merge |
| مَدَّ | Form I: (if root م-د-و — "to praise")? | م-د-و | Rare root; low probability |

**Note on triliteral vs. quadriliteral disambiguation:** A stem matching a pattern like `C₁aC₂aC₃a` could potentially correspond to both Form I perfect (triliteral, 3 root consonants) and QI perfect (quadriliteral, 4 root consonants). The disambiguating factor is the **known root's letter count** retrieved from KB-0001: if the root is triliteral (3 consonants), only verb form candidates compatible with 3-consonant roots are considered; if quadriliteral (4 consonants), only 4-consonant-compatible patterns are tried. This filtering happens at Step 2 of the matching algorithm (candidate selection), before template comparison.

In such cases, MOD-04 includes all valid matches in the ambiguity set, with confidence scores based on root attestation frequency and semantic congruity.

### 9.4 Stem-to-Wazan Reverse Index

For fast lookup during morphological analysis, KB-0002 maintains a **reverse index** that maps common stem patterns to their wazans:

```pseudo
ReverseIndex:
  # Sound verb stems
  "C₁aC₂aC₃a"      → [Form I perfect active, QI perfect]        # 3-consonant
  "C₁aC₂C₂aC₃a"    → [Form II perfect active]                   # Geminated C₂
  "C₁āC₂aC₃a"      → [Form III perfect active]                  # Long ā after C₁
  "aC₁C₂aC₃a"      → [Form IV perfect active]                   # Prefix alif
  "taC₁aC₂C₂aC₃a"  → [Form V perfect active]                    # Prefix ta + geminated C₂

  # Common noun patterns
  "C₁āC₂iC₃"       → [Active participle (ism fāʿil)]            # فَاعِل
  "maC₁C₂ūC₃"      → [Passive participle (ism mafʿūl)]          # مَفْعُول
  "maC₁C₂aC₃"      → [Noun of place]                            # مَفْعَل
  "miC₁C₂āC₃"      → [Noun of instrument]                       # مِفْعَال

  # Weak root variants
  "C₁āC₃"          → [Form I ajwaf perfect active]              # قَالَ (root: ق-و-ل)
  "C₁āC₃ā"         → [Form I naqis perfect active]              # رَمَى → C₁āC₃ā (root: ر-م-ي)
  "yaC₁C₂ūC₃u"     → [Form I ajwaf imperfect indicative]        # يَقُولُ
  ...
```

---

## 10. Serialization & Storage

### 10.1 Source Format

```diff
  /knowledge/KB-0002/
  ├── metadata.yaml                     # KB metadata (version, counts, sources)
  ├── verb-forms/
  │   ├── form-I.yaml                   # All Form I patterns (base + vowel variants)
  │   ├── form-II.yaml                  # All Form II patterns
  │   ├── form-III.yaml
  │   ├── ...
  │   └── form-XV.yaml
  ├── noun-patterns/
  │   ├── masdar.yaml                   # All verbal noun patterns
  │   ├── active-participle.yaml        # Ism fāʿil patterns
  │   ├── passive-participle.yaml       # Ism mafʿūl patterns
  │   ├── place-time.yaml               # Ism makān/zamān patterns
  │   ├── instrument.yaml               # Ism ālah patterns
  │   ├── adjective.yaml                # Sifah mushabbahah patterns
  │   ├── elative.yaml                  # Tafḍīl patterns
  │   └── nisbah.yaml                   # Nisbah patterns
  ├── weak-variants/
  │   ├── ajwaf.yaml                    # Hollow root pattern variants
  │   ├── naqis.yaml                    # Defective root pattern variants
  │   ├── mithal.yaml                   # Assimilated root pattern variants
  │   ├── doubled.yaml                  # Doubled root pattern variants
  │   ├── hamzated.yaml                 # Hamzated root pattern variants
  │   └── lafif.yaml                    # Double-weak root pattern variants
  ├── quadriliteral/
  │   ├── form-QI.yaml                  # Basic quadriliteral verb patterns
  │   ├── form-QII.yaml
  │   ├── form-QIII.yaml
  │   └── noun-patterns.yaml
  └── classes/
      ├── sound.yaml                    # Sound triliteral class definition
      ├── hollow.yaml                   # Hollow verb class rules
      ├── defective.yaml                # Defective verb class rules
      └── doubled.yaml                  # Doubled verb class rules
```

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0002"
title: "Wazan Database — Morphological Patterns"
version: "2.0.1"
status: "draft" | "review" | "published"

pattern_count: 385
verb_pattern_count: 42
noun_pattern_count: 112
weak_variant_count: 180
quadriliteral_count: 18
class_definitions: 23

created_at: "2026-07-14T00:00:00Z"
updated_at: "2026-07-14T00:00:00Z"

sources:
  - name: "Sibawayh, Al-Kitab"
    version: "critical_1988"
  - name: "Al-Muqtadab (Al-Mubarrad)"
    version: "print_1979"
  - name: "Sharh al-Ashmuni"
    version: "print_1963"

checksum_sha256: "b2c3d4e5f6a7..."
maintainers:
  - name: "Dr. [Name]"
    email: "[email]"
    role: "morphology_editor"
```

### 10.2 Compiled Format (Hash Index)

The production format is a **hash-indexed pattern database** optimized for fast wazan matching:

```diff
  Compiled Hash Index Layout:
  ┌──────────────────────────────────────────────────────────────┐
  │ HEADER                                                       │
  │ ├── magic: "AGOSKB02" (8 bytes)                             │
  │ ├── version: major(2B) + minor(2B) + patch(2B)              │
  │ ├── pattern_count: u32 (4 bytes)                            │
  │ ├── signature_index_offset: u32 (4 bytes)                   │
  │ ├── pattern_data_offset: u32 (4 bytes)                      │
  │ ├── string_table_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ SIGNATURE INDEX                                              │
  │ ├── Hash table mapping u64 signatures → pattern_id[]        │
  │ ├── Buckets: 1024 entries, chained                           │
  │ └── O(1) average lookup                                     │
  ├──────────────────────────────────────────────────────────────┤
  │ PATTERN DATA                                                 │
  │ ├── Fixed-size pattern records (128 bytes each)             │
  │ │   ├── pattern_type: u8                                    │
  │ │   ├── verb_form: u8 | 0xFF                                │
  │ │   ├── root_type_applicability: u32 bitmask                │
  │ │   ├── template_signature: u64                             │
  │ │   ├── inherent_feature_bitmask: u64                       │
  │ │   ├── canonical_pattern_offset: u32 (→ string table)     │
  │ │   ├── template_script_offset: u32 (→ string table)       │
  │ │   └── example_count: u16                                  │
  │ └── ... (pattern_count entries)                             │
  ├──────────────────────────────────────────────────────────────┤
  │ VARIANT DATA                                                 │
  │ ├── Pattern variant links                                   │
  │ ├── Weak root transformation rules                          │
  │ └── Substitution rule bytecodes                             │
  ├──────────────────────────────────────────────────────────────┤
  │ STRING TABLE                                                 │
  │ ├── Length-prefixed UTF-8 strings                           │
  │ ├── Canonical patterns, template scripts, examples          │
  │ └── Referenced by offsets from pattern data                 │
  └──────────────────────────────────────────────────────────────┘
```

#### Entry Record Structure

```c
struct PatternRecord {
    uint8_t pattern_type;                     // 0=verb, 1=noun, 2=variant
    uint8_t verb_form;                        // 1–15, or 0xFF if noun
    uint8_t noun_type;                        // From NounType enum
    uint16_t subclass_id;                     // Pattern subclass identifier
    uint32_t root_type_bitmask;               // Which root types this applies to
    uint64_t template_signature;              // Compact pattern signature
    uint64_t feature_bitmask;                 // Inherent features
    uint32_t canonical_pattern_offset;        // → string table
    uint32_t template_script_offset;          // → string table
    uint32_t variant_link_offset;             // → weak variant data (if any)
    uint16_t example_count;                   // Number of examples
    uint16_t example_offset;                  // → example array
};
```

### 10.3 File Packaging

```diff
  KB-0002-v2.0.1.agos-kb              # Compiled hash index binary
  KB-0002-v2.0.1.agos-kb.sig          # Ed25519 signature
  KB-0002-v2.0.1.agos-kb.sha256       # SHA-256 checksum
  KB-0002-v2.0.1.source.tar.gz        # Source YAML files (optional)
```

### 10.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Signature index | 1 MB | 2 MB | Hash table with 1024 buckets |
| Pattern data | 2 MB | 5 MB | ~400 records at 128 bytes each |
| Variant data | 2 MB | 8 MB | Weak root transformation rules |
| String table | 3 MB | 13 MB | Pattern names, template scripts, examples |
| Example data | 1 MB | 6 MB | Concrete examples with citations |
| Class definitions | 1 MB | 6 MB | 20+ pattern class definitions with rules |
| **Total** | **~10 MB** | **~40 MB** | Memory-mapped load |

The **Compact** format drops rare pattern variants (Forms XI–XV), less common noun patterns, and verbose example citations. The **Full** format includes all data.

---

## 11. Versioning & Evolution

### 11.1 Versioning Scheme

KB-0002 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to wazan schema, removal of pattern entries, format change | `1.0.0` → `2.0.0` | Requires KB conversion tool, invalidates all caches |
| **MINOR** | Addition of new patterns, new variant entries, new optional fields | `2.0.1` → `2.1.0` | Backward-compatible; existing pattern IDs remain valid |
| **PATCH** | Corrections to pattern definitions, improved examples, typo fixes | `2.0.1` → `2.0.2` | Backward-compatible; no schema changes |

### 11.2 Cross-KB Compatibility

```yaml
cross_kb_compatibility:
  KB-0001: ">= 1.0.0"       # Wazans reference root types from KB-0001
  KB-0003: ">= 1.0.0"       # Verb form wazans link to conjugation paradigms
  KB-0004: ">= 1.0.0"       # Noun pattern wazans link to detailed noun specs
  KB-0005: ">= 1.0.0"       # Independent (no wazan dependency)
  KB-0006: ">= 1.0.0"       # Independent (no wazan dependency)
  KB-0007: ">= 1.0.0"       # verb_form, noun_type features
```

### 11.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add a new wazan pattern | MINOR | Add YAML file, update signature index, regenerate compiled KB |
| Correct pattern template | PATCH | Edit pattern definition, update examples |
| Add weak root variant | MINOR | Add variant entry, link to base pattern |
| Remove a pattern | MAJOR | Only for demonstrably incorrect entries; document in changelog |
| Add new optional field | MINOR | Add field, existing entries remain valid |
| Pattern merger | MAJOR | Merge two patterns, add cross-references, deprecate one |

---

## 12. Quality Requirements

### 12.1 Completeness Targets

| Category | Minimum | Target | Stretch |
|----------|---------|--------|---------|
| Verb forms I–X (base patterns) | 100% | 100% | 100% |
| Verb forms XI–XV (rare) | 80% | 90% | 100% |
| Form I vowel variants | 100% | 100% | 100% |
| Active/Passive participle patterns | 100% | 100% | 100% |
| Regular masdar patterns (II–X) | 100% | 100% | 100% |
| Form I masdar patterns (common ~20) | 90% | 95% | 100% |
| Noun of place/time/instrument | 100% | 100% | 100% |
| Adjective patterns | 90% | 95% | 100% |
| Weak root variants (Ajwaf) | 95% | 100% | 100% |
| Weak root variants (Naqis) | 90% | 95% | 100% |
| Weak root variants (Mithal) | 85% | 90% | 95% |
| Weak root variants (Doubled) | 90% | 95% | 100% |
| Quadriliteral patterns | 90% | 95% | 100% |

### 12.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Pattern-to-template consistency | 100% — template must match canonical pattern | Automated cross-check |
| Root position mapping | 100% — all root consonants mapped to valid positions | Automated validation |
| Example correctness | 100% — each example must match its pattern template | Automated stem-pattern matching |
| Weak variant completeness | ≥ 95% — all attested weak variants for core patterns | Comparison with reference grammars |
| Cross-KB consistency (KB-0001) | 100% — root types referenced must exist in KB-0001 | Automated cross-KB check |

### 12.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate YAML structure
  ├── schema: validate against KB-0002 JSON Schema
  ├── template_check: verify each pattern's template matches its canonical form
  ├── example_check: verify each example matches the pattern template
  └── lint: field presence, Arabic-only text

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── cross_kb: verify root type references exist in KB-0001
  ├── cross_kb: verify masdar patterns are listed in KB-0001 derived_nouns
  ├── variant_links: verify weak variants link to valid base patterns
  ├── signature_uniqueness: verify pattern signatures don't collide
  ├── compilation: verify hash index compiles without error
  ├── size_budget: verify compiled size ≤ 40 MB
  └── regression: verify known stems still match correct patterns

  Review (manual, per release):
  ├── sample_check: linguist reviews 5% random pattern sample
  ├── hotspot_check: review patterns modified since last version
  ├── weak_variant_audit: verify weak root variants against reference grammar
  └── changelog: verify changelog accuracy
```

### 12.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Pattern signature lookup | < 500 ns | Per lookup, average |
| Pattern signature lookup (p99) | < 2 μs | Per lookup, 99th percentile |
| Template matching (candidate stem) | < 2 μs | Per candidate, average |
| Full wazan identification (10 candidates) | < 20 μs | Per stem, average |
| KB load time (compact) | < 25 ms | mmap + verify checksum |
| KB load time (full) | < 50 ms | mmap + verify checksum |
| Memory (compact) | ~10 MB | RSS |
| Memory (full) | ~40 MB | RSS |

---

## 13. Example Entries

### 13.1 Verb Form II (Intensive/Causative): فَعَّلَ

```json
{
  "id": "KB-0002:verb:II:فَعَّلَ",
  "pattern_type": "verb",
  "canonical_pattern": "فَعَّلَ",
  "pattern_script": "فَعَّلَ",
  "verb_form": 2,
  "noun_type": null,
  "subclass": "basic",
  "root_type_applicability": ["sound_triliteral", "ajwaf_wawi", "ajwaf_yai",
                               "naqis_wawi", "naqis_yai", "doubled",
                               "mahmuz_al_fa", "mahmuz_al_ayn", "mahmuz_al_lam"],
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂", "is_geminated": true },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂_gem", "is_geminated": true },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" },
      { "type": "vowel", "character": "a" }
    ],
    "vowel_pattern": ["a", "a"],
    "consonant_count": 3,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 2, "root_position": 2, "is_geminated": true },
        { "template_slot": 3, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "C₁aC₂C₂aC₃a"
  },
  "inherent_features": {
    "features": [
      { "name": "pos", "value": "verb" },
      { "name": "voice", "value": "active" },
      { "name": "tense", "value": "past" },
      { "name": "form", "value": "II" }
    ]
  },
  "conjugation_class": "sound",
  "transitivity_default": "transitive",
  "core_meaning": "Causative, intensive, denominative",
  "core_meaning_ar": "التكثير والتقوية والتعدية",
  "semantic_modification": "Strengthens the action, often adds causation or shows intensity",
  "examples": [
    { "word": "كَتَّبَ", "transliteration": "kattaba", "meaning": "he caused (someone) to write", "root": "كتب" },
    { "word": "عَلَّمَ", "transliteration": "ʿallama", "meaning": "he taught", "root": "علم" },
    { "word": "قَدَّمَ", "transliteration": "qaddama", "meaning": "he presented forward", "root": "قدم" },
    { "word": "كَسَّرَ", "transliteration": "kassara", "meaning": "he broke into pieces", "root": "كسر" }
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab, Vol. IV, p. 125"],
    "classical_references": ["Al-Kitab", "Al-Muqtadab", "Sharh al-Ashmuni"]
  }
}
```

### 13.2 Noun of Place: مَفْعَل

```json
{
  "id": "KB-0002:noun:ism_makan:مَفْعَل",
  "pattern_type": "noun",
  "canonical_pattern": "مَفْعَل",
  "pattern_script": "مَفْعَل",
  "verb_form": null,
  "noun_type": "ism_makan",
  "subclass": "basic",
  "root_type_applicability": ["sound_triliteral"],
  "template": {
    "segments": [
      { "type": "prefix", "character": "م", "slot_label": "PREF_M" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "vowel_pattern": ["a", "a"],
    "consonant_count": 4,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 0, "is_geminated": false },
        { "template_slot": 2, "root_position": 1, "is_geminated": false },
        { "template_slot": 3, "root_position": 2, "is_geminated": false },
        { "template_slot": 4, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "maC₁C₂aC₃"
  },
  "note": "Template slots 1–4 correspond to the four template positions (prefix M + C₁ + C₂ + C₃). The prefix (م) at slot 1 maps to root_position 0 because it is not a root consonant."
  },
  "inherent_features": {
    "features": [
      { "name": "pos", "value": "noun" },
      { "name": "noun_type", "value": "ism_makan" },
      { "name": "gender", "value": "masculine" }
    ]
  },
  "core_meaning": "Noun of place — indicates where an action occurs",
  "core_meaning_ar": "اسْم المَكَان — موضع الفعل",
  "semantic_modification": "A location where the root's action is performed",
  "examples": [
    { "word": "مَكْتَب", "transliteration": "maktab", "meaning": "desk, office (place of writing)", "root": "كتب" },
    { "word": "مَلْعَب", "transliteration": "malʿab", "meaning": "playground (place of playing)", "root": "لعب" },
    { "word": "مَذْهَب", "transliteration": "madhhab", "meaning": "school of thought (place of going)", "root": "ذهب" },
    { "word": "مَطْبَخ", "transliteration": "maṭbakh", "meaning": "kitchen (place of cooking)", "root": "طبخ" }
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Al-Muqtadab (Al-Mubarrad), Vol. II, p. 45"],
    "classical_references": ["Al-Muqtadab", "Sharh al-Ashmuni"]
  }
}
```

### 13.3 Weak Variant: Form I Ajwaf Wawi (Perfect Active)

```json
{
  "id": "KB-0002:variant:form_I:ajwaf_wawi_perfect",
  "pattern_type": "verb",
  "canonical_pattern": "فَالَ",
  "pattern_script": "فَالَ",
  "verb_form": 1,
  "noun_type": null,
  "subclass": "ajwaf_wawi",
  "root_type_applicability": ["ajwaf_wawi"],
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "long_vowel_marker", "character": "ā" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "vowel_pattern": ["ā"],
    "consonant_count": 2,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 3, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "C₁āC₃"
  },
  "inherent_features": {
    "features": [
      { "name": "pos", "value": "verb" },
      { "name": "voice", "value": "active" },
      { "name": "tense", "value": "past" },
      { "name": "form", "value": "I" },
      { "name": "root_type", "value": "ajwaf_wawi" }
    ]
  },
  "core_meaning": "Form I ajwaf wawi perfect active — medial waw becomes ā",
  "core_meaning_ar": "الفعل الأجوف الواوي في الماضي",
  "semantic_modification": "No semantic modification; this is a phonological variant of Form I",
  "examples": [
    { "word": "قَالَ", "transliteration": "qāla", "meaning": "he said", "root": "قول" },
    { "word": "كَانَ", "transliteration": "kāna", "meaning": "he was", "root": "كون" },
    { "word": "زَارَ", "transliteration": "zāra", "meaning": "he visited", "root": "زور" },
    { "word": "سَالَ", "transliteration": "sāla", "meaning": "it flowed", "root": "سيل" }
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab — Chapter on Hollow Verbs (باب الأجوف)"]
  }
}
```

### 13.4 Quadriliteral Verb: Form I (فَعْلَلَ)

```json
{
  "id": "KB-0002:verb:QI:فَعْلَلَ",
  "pattern_type": "verb",
  "canonical_pattern": "فَعْلَلَ",
  "pattern_script": "فَعْلَلَ",
  "verb_form": 1,
  "noun_type": null,
  "subclass": "quadriliteral",
  "root_type_applicability": ["sound_quadriliteral"],
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" },
      { "type": "root_consonant", "position": 4, "slot_label": "C₄" },
      { "type": "vowel", "character": "a" }
    ],
    "vowel_pattern": ["a", "a"],
    "consonant_count": 4,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 2, "root_position": 2, "is_geminated": false },
        { "template_slot": 3, "root_position": 3, "is_geminated": false },
        { "template_slot": 4, "root_position": 4, "is_geminated": false }
      ]
    },
    "template_script": "C₁aC₂C₃aC₄a"
  },
  "inherent_features": {
    "features": [
      { "name": "pos", "value": "verb" },
      { "name": "voice", "value": "active" },
      { "name": "tense", "value": "past" },
      { "name": "form", "value": "QI" }
    ]
  },
  "conjugation_class": "sound_quad",
  "core_meaning": "Basic quadriliteral verb",
  "core_meaning_ar": "الفعل الرباعي المجرد",
  "semantic_modification": "Basic form for quadriliteral roots; often intensitive, onomatopoeic, or denominative",
  "examples": [
    { "word": "زَلْزَلَ", "transliteration": "zalzala", "meaning": "to shake (violently)", "root": "زلزل" },
    { "word": "وَسْوَسَ", "transliteration": "waswasa", "meaning": "to whisper (insistently)", "root": "وسوس" },
    { "word": "دَحْرَجَ", "transliteration": "daḥraja", "meaning": "to roll", "root": "دحرج" }
  ],
  "attestation": {
    "confidence": "certain",
    "sources": ["Sibawayh, Al-Kitab"],
    "notes": "Quadriliteral roots are fully productive in this form"
  }
}
```

### 13.5 Masdar Pattern: Form II (تَفْعِيل)

```json
{
  "id": "KB-0002:noun:masdar:form_II:تَفْعِيل",
  "pattern_type": "noun",
  "canonical_pattern": "تَفْعِيل",
  "pattern_script": "تَفْعِيل",
  "verb_form": 2,
  "noun_type": "masdar",
  "subclass": "form_II_regular",
  "root_type_applicability": ["sound_triliteral", "ajwaf_wawi", "naqis_yai", "doubled"],
  "template": {
    "segments": [
      { "type": "prefix", "character": "ت", "slot_label": "PREF_T" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "long_vowel_marker", "character": "ī" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "vowel_pattern": ["a", "ī"],
    "consonant_count": 4,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 2, "root_position": 2, "is_geminated": false },
        { "template_slot": 3, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "taC₁C₂īC₃"
  },
  "inherent_features": {
    "features": [
      { "name": "pos", "value": "noun" },
      { "name": "noun_type", "value": "masdar" },
      { "name": "gender", "value": "masculine" }
    ]
  },
  "core_meaning": "Verbal noun of Form II — the act of {verb meaning}",
  "core_meaning_ar": "مصدر الفعل الثلاثي المزيد (فَعَّلَ)",
  "examples": [
    { "word": "تَكْتِيب", "transliteration": "taktīb", "meaning": "dictation (act of causing to write)", "root": "كتب" },
    { "word": "تَعْلِيم", "transliteration": "taʿlīm", "meaning": "teaching (act of teaching)", "root": "علم" },
    { "word": "تَقْدِيم", "transliteration": "taqdīm", "meaning": "presentation (act of presenting)", "root": "قدم" },
    { "word": "تَكْسِير", "transliteration": "taksīr", "meaning": "breaking (act of breaking into pieces)", "root": "كسر" }
  ],
  "attestation": {
    "confidence": "certain"
  }
}
```

---

## 14. Cross-References

### 14.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0002 in module catalog |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Wazan identification algorithm using KB-0002 |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Wazan schema in MOD-08 interface |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-4) | Wazan field in morphological analysis |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-8) | Resolved wazan entry in ResolvedGIR |
| SPEC-0001-C6 | Deployment & Runtime Considerations | KB bundling, size budget |
| SPEC-0001-C8 | Security, Validation & Error Handling | KB integrity verification |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0002 size, hash index lookup |
| KB-0001 | Roots Database | Root types and classifications referenced by wazans |
| KB-0003 | Verb Forms | Full conjugation tables for each verb form |
| KB-0004 | Noun Patterns | Detailed noun pattern specifications |
| KB-0007 | Morphological Features | Feature taxonomy for inherent features |

### 14.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (8th C. CE) | Foundational grammar; defines the wazan system |
| Al-Mubarrad, Al-Muqtadab (9th C. CE) | Morphological analysis of Arabic verbs |
| Ibn Hisham, Qatr al-Nada (14th C. CE) | I'rab-based pattern analysis |
| Al-Ashmuni, Sharh al-Ashmuni (15th C. CE) | Comprehensive grammatical commentary |
| Wright's Arabic Grammar (1859) | Western reference for Arabic morphology |
| Buckwalter Arabic Morphological Analyzer | Reference for computational pattern matching |

---

## Progress Summary

**KB-0002: Wazan Database — Morphological Patterns**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Wazan in Arabic Morphology | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Wazan Entry Schema | ✓ COMPLETE |
| Section 5 | Verb Form Wazan (I–XV) | ✓ COMPLETE |
| Section 6 | Derived Noun Wazan | ✓ COMPLETE |
| Section 7 | Quadriliteral Wazan | ✓ COMPLETE |
| Section 8 | Weak Root Pattern Variants | ✓ COMPLETE |
| Section 9 | Wazan Matching Algorithm | ✓ COMPLETE |
| Section 10 | Serialization & Storage | ✓ COMPLETE |
| Section 11 | Versioning & Evolution | ✓ COMPLETE |
| Section 12 | Quality Requirements | ✓ COMPLETE |
| Section 13 | Example Entries | ✓ COMPLETE |
| Section 14 | Cross-References | ✓ COMPLETE |

**Dependencies:** KB-0001 (Roots Database), SPEC-0001 (Chapters 1–5, 8, 9).

**Recommended next document:** KB-0003 (Verb Forms) — the full conjugation paradigms for each verb form.
