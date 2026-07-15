---
kb_id: KB-0004
title: Noun Patterns — Derived Noun Specifications
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0004)
  - SPEC-0001-C3: Compilation Pipeline (MOD-04 MorphologicalParser)
  - SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-04, MOD-08)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-4, IR-8)
  - SPEC-0001-C6: Deployment & Runtime Considerations (KB Bundling)
  - SPEC-0001-C8: Security, Validation & Error Handling (KB Integrity)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - SPEC-0101: Morphology Engine (planned)
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---

# KB-0004: Noun Patterns — Derived Noun Specifications

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Nouns in Arabic Morphology](#2-nouns-in-arabic-morphology)
3. [Data Model](#3-data-model)
4. [Noun Pattern Entry Schema](#4-noun-pattern-entry-schema)
5. [Verbal Noun (Masdar) Patterns — Regular](#5-verbal-noun-masdar-patterns--regular)
6. [Verbal Noun (Masdar) Patterns — Form I Unpredictable](#6-verbal-noun-masdar-patterns--form-i-unpredictable)
7. [Active Participle (Ism al-Faʿil) Patterns](#7-active-participle-ism-al-fail-patterns)
8. [Passive Participle (Ism al-Mafʿul) Patterns](#8-passive-participle-ism-al-maful-patterns)
9. [Noun of Place & Time (Ism al-Makan / al-Zaman)](#9-noun-of-place--time-ism-al-makan--al-zaman)
10. [Noun of Instrument (Ism al-Ālah)](#10-noun-of-instrument-ism-al-alah)
11. [Resembling Adjective (Sifah Mushabbahah)](#11-resembling-adjective-sifah-mushabbahah)
12. [Elative (Ism al-Tafḍīl)](#12-elative-ism-al-tafdil)
13. [Relative Adjective (Nisbah)](#13-relative-adjective-nisbah)
14. [Broken Plural (Jamʿ al-Taksir) Patterns](#14-broken-plural-jam-al-taksir-patterns)
15. [Other Noun Patterns](#15-other-noun-patterns)
16. [Weak Root Variants for Noun Patterns](#16-weak-root-variants-for-noun-patterns)
17. [Noun Pattern Matching Algorithm](#17-noun-pattern-matching-algorithm)
18. [Serialization & Storage](#18-serialization--storage)
19. [Versioning & Evolution](#19-versioning--evolution)
20. [Quality Requirements](#20-quality-requirements)
21. [Example Entries](#21-example-entries)
22. [Cross-References](#22-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0004 is the **authoritative specification of Arabic derived noun patterns** (الأسماء المشتقة, `al-asmāʾ al-mushtaqqa`) used by the AGOS platform. While KB-0002 (Wazan Database) defines the **stem-level wazan templates** for noun patterns, KB-0004 provides the **detailed, full entry-level specifications** for each noun pattern type, including:

- Full inflectional behavior (gender, number, definiteness, case)
- Semantic and syntactic properties
- Weak root variant tables for each noun type
- Broken plural associations
- Case ending patterns (declension class)

KB-0004 answers: **\"What kind of noun is this derived form? How does it inflect? What patterns can its broken plural take?\"**

### 1.2 Scope

KB-0004 covers:

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Derived nouns** | Masdar, ism fāʿil, ism mafʿūl, ism makān/zamān, ism ālah, ṣifah, tafḍīl, nisbah | Primary/primitive nouns (non-derived, covered per-root lookups) |
| **Broken plurals** | Jamʿ taksīr patterns (~30+ templates) | Sound masculine/feminine plurals (regular suffixation) |
| **Root types** | Sound, Weak (all types), Hamzated, Doubled, Quadriliteral | N/A |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal patterns |
| **Inflection** | Gender assignment, declension class, number patterns, broken plural mapping | Full case endings (covered by MOD-03 iʿrāb) |
| **Semantics** | Semantic categories per pattern type (e.g., masdar = event/action, ism makan = location) | Lexical semantics (covered by KB-0001) |

### 1.3 Target Audience

- **AGOS Pipeline:** MOD-04 (MorphologicalParser) reads KB-0004 during noun pattern identification. MOD-08 (KnowledgeGraphResolver) reads KB-0004 during feature resolution. MOD-09 (BytecodeGenerator) reads KB-0004 for noun form generation. MOD-11 (ExplanationEngine) uses KB-0004 for pedagogical explanations.
- **Linguists & Data Maintainers:** Edit and extend KB-0004 with additional noun pattern types or weak root variants.
- **Grammar Tool Authors:** KB-0004 serves as a reference for Arabic noun morphology.

### 1.4 Relationship to Other KBs

```diff
  KB-0001: Roots (جذور)                 — The consonants that carry lexical meaning
    │
    ├──► KB-0002: Wazan (أوزان)         — Stem-level patterns (verb & noun templates)
    │         │
    │         └──► KB-0004: Noun Patterns ◄── This document (Detailed noun specs)
    │                 │
    │                 ├──► Masdar (مصدر)          — Verbal noun of action
    │                 ├──► Ism Faʿil (اسم فاعل)   — Active participle
    │                 ├──► Ism Mafʿul (اسم مفعول) — Passive participle
    │                 ├──► Ism Makan/Zaman        — Noun of place/time
    │                 ├──► Ism Ālah (اسم آلة)     — Noun of instrument
    │                 ├──► Sifah Mushabbahah      — Resembling adjective
    │                 ├──► Ism Tafḍīl (اسم تفضيل) — Comparative/elative
    │                 ├──► Nisbah (نسبة)          — Relative adjective
    │                 └──► Jamʿ Taksir (جمع تكسير) — Broken plural
    │
    ├──► KB-0003: Verb Forms (تصريف)   — Verb conjugation paradigms
    ├──► KB-0005: Particles             — Particles have no roots or wazans
    ├──► KB-0006: Pronouns              — Pronouns have no roots or wazans
    └──► KB-0007: Morphological Feat.   — Feature taxonomy for nouns
```

### 1.5 How KB-0004 Complements KB-0002

The boundary between KB-0002 and KB-0004 for noun patterns is:

| Aspect | KB-0002 (Wazan) | KB-0004 (Noun Patterns) |
|--------|-----------------|------------------------|
| **Granularity** | Stem-level wazan template | Full noun entry with inflection, semantics, broken plural mapping |
| **What it adds** | Phonological template, root-position mapping | Gender, declension, broken plural patterns, semantic categories |
| **Example (فَاعِل)** | `C₁āC₂iC₃` template with root map | Masculine, sound plural فَاعِلُونَ, feminine فَاعِلَة, broken plural فُعَّال/فَعَلَة |
| **Weak root handling** | Phonological variants of stems | Full weak root paradigm for each noun type |
| **Output for MOD-09** | How to generate the noun stem | How to generate all inflected forms of the noun |
| **Broken plurals** | Not covered | ~40+ broken plural templates with pattern mapping |
| **Semantic categories** | `semantic_modification` field | Detailed semantic roles (agent, patient, instrument, location, etc.) |

---

## 2. Nouns in Arabic Morphology

### 2.1 Derivation vs. Primitive Nouns

Arabic nouns fall into two broad categories:

1. **Derived nouns** (اسم مشتق, `ism mushtaqq`): Formed by applying a noun pattern (wazan) to a root. These are the focus of KB-0004.
2. **Primitive nouns** (اسم جامد, `ism jāmid`): Not derived from a verb root. These include basic vocabulary items (e.g., كِتَاب 'book' which is synchronically a noun with a meaning beyond its derivation) and foreign loanwords.

The line between derived and primitive is sometimes blurred — many nouns that are historically derived are treated as lexical entries in KB-0001. KB-0004 focuses on the **productive patterns** used to generate new nouns.

### 2.2 The Noun Pattern Taxonomy

```diff
  Arabic Derived Nouns (الأسماء المشتقة)
  │
  ├── 1. VERBAL NOUNS (مصادر)
  │     ├── Form I masdar (unpredictable, ~40+ patterns)
  │     ├── Form II masdar (تَفْعِيل)
  │     ├── Form III masdar (فِعَال / مُفَاعَلَة)
  │     ├── Form IV masdar (إِفْعَال)
  │     ├── Form V masdar (تَفَعُّل)
  │     ├── Form VI masdar (تَفَاعُل)
  │     ├── Form VII masdar (اِنْفِعَال)
  │     ├── Form VIII masdar (اِفْتِعَال)
  │     ├── Form IX masdar (اِفْعِلَال)
  │     ├── Form X masdar (اِسْتِفْعَال)
  │     ├── Masdar marrah (instance noun)
  │     └── Masdar hay'ah (manner noun)
  │
  ├── 2. PARTICIPLES (المشتقات الصرفية)
  │     ├── Active participle (اسم فاعل)
  │     ├── Passive participle (اسم مفعول)
  │     └── Resembling adjective (صفة مشبهة)
  │
  ├── 3. PLACE / TIME NOUNS (أسماء المكان والزمان)
  │
  ├── 4. INSTRUMENT NOUNS (أسماء الآلة)
  │
  ├── 5. COMPARATIVE / ELATIVE (اسم التفضيل)
  │
  ├── 6. RELATIVE ADJECTIVE (النسبة)
  │
  └── 7. BROKEN PLURALS (جموع التكسير)
        ├── Plural of paucity (جمع قلة)
        ├── Plural of multitude (جمع كثرة)
        └── Pattern-based plurals (~30+ templates)
```

### 2.3 Noun Inflection Categories

Arabic nouns inflect for:

| Category | Values | Notes |
|----------|--------|-------|
| **Gender** | Masculine (مذكر), Feminine (مؤنث) | Feminine usually marked by tāʾ marbūṭa (ة) |
| **Number** | Singular, Dual, Plural (sound or broken) | Dual is regular +ān/+ayn; plural can be sound or broken |
| **Definiteness** | Definite (ال prefix), Indefinite (tanwīn) | Marked by article or nunation |
| **Case** | Nominative (الرفع), Accusative (النصب), Genitive (الجر) | Marked by short vowels and suffixes |
| **Declension** | Triptote (مصرف), Diptote (ممنوع من الصرف) | Determines case ending behavior |

KB-0004 catalogs the **default** gender, number, and declension class for each noun pattern. These defaults can be overridden per-root in KB-0001.

### 2.4 Sound vs. Broken Plurals

Arabic has two plural formation strategies:

1. **Sound plural** (جمع سالم `jamʿ sālim`): Regular suffixation. Masculine: -ūn(a)/-īn(a). Feminine: -āt(un).
2. **Broken plural** (جمع تكسير `jamʿ taksīr`): Internal vowel and consonant changes to the stem. Unpredictable from the singular — must be stored per pattern or per root.

KB-0004 includes ~30+ broken plural templates and maps each noun pattern to its **most common broken plural pattern(s)**.

---

## 3. Data Model

### 3.1 Logical Data Model

```yaml
Noun Patterns Database (KB-0004)
├── Metadata
│   ├── kb_id: "KB-0004"
│   ├── version: "1.0.0"
│   ├── noun_pattern_count: integer
│   ├── broken_plural_count: integer
│   ├── created_at: timestamp
│   ├── sources: string[]
│   └── checksum_sha256: string
│
├── NounPatterns: NounPatternEntry[]
│   ├── Masdar patterns (regular + Form I unpredictable)
│   ├── Participle patterns (active, passive)
│   ├── Place/Time noun patterns
│   ├── Instrument noun patterns
│   ├── Adjective patterns (resembling, elative, relative)
│   └── Other patterns
│
├── BrokenPluralTemplates: BrokenPluralTemplate[]
│   └── ~30+ broken plural pattern definitions
│
└── NounClassMappings: ClassMapping[]
    └── Maps each (noun_type, pattern) → inflection class
```

### 3.2 Storage Model

KB-0004 is stored in two formats:

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~30–50 MB uncompressed | Human-readable |
| **Compiled (Table Binary)** | Production pipeline | ~10–30 MB | Memory-mapped O(1) lookup |

### 3.3 Pattern Count Target

| Category | Estimated Patterns | Notes |
|----------|-------------------|-------|
| Masdar (Form I, unpredictable) | ~40–50 | Core patterns covering ~90% of attested Form I masdars |
| Masdar (Forms II–X, regular) | ~9 | One per derived form |
| Active Participle | ~12 | Per verb form (I–X) + weak root variants |
| Passive Participle | ~10 | Per verb form (I–X, except IX) + weak root variants |
| Noun of Place/Time | ~6 | مَفْعَل, مَفْعِل, variants for weak roots |
| Noun of Instrument | ~6 | مِفْعَل, مِفْعَال, مِفْعَلَة + variants |
| Resembling Adjective | ~12 | فَعِيل, فَعْل, فَعِل, أَفْعَل, فَعْلَان + variants |
| Elative (Tafḍīl) | ~4 | أَفْعَل pattern + weak/hamzated variants |
| Nisbah (Relative) | ~6 | فَعْلِيّ, فِعْلِيّ, variants |
| Broken Plural Templates | ~30–40 | All major jamʿ taksīr patterns |
| Other | ~6 | Masdar marrah, masdar hay'ah, etc. |
| **Total** | **~135–180** | Covers all productive patterns for Classical Arabic and MSA |

---

## 4. Noun Pattern Entry Schema

### 4.1 Schema Definition

```yaml
NounPatternEntry:
  # --- Identity ---
  id: string                           # "KB-0004:{noun_type}:{canonical_pattern}"
                                       # e.g., "KB-0004:masdar:form_I:فَعْل"
  noun_type: NounType                  # From NounType enum
  canonical_pattern: string            # Wazan with ف-ع-ل, e.g., "فَاعِل"
  pattern_variant: string | null       # e.g., "basic", "hollow_variant"

  # --- Source ---
  verb_form: integer | null            # I–XV (which form this pattern belongs to, null if noun-only)
  derived_from_form: integer | null    # For masdars: which verb form this is the masdar of

  # --- Phonological Template ---
  template: PhonologicalTemplate       # Segments, root positions, vowel pattern

  # --- Inflection ---
  gender: "masculine" | "feminine" | "both"
  declension: "triptote" | "diptote" | "indeclinable"
  feminine_form: string | null         # How to form the feminine (e.g., "add_ta_marbuta")
  feminine_template: string | null     # Feminine wazan (e.g., "فَاعِلَة" for "فَاعِل")
  sound_plural_masculine: string | null  # Sound masculine plural template
  sound_plural_feminine: string | null   # Sound feminine plural template

  # --- Broken Plurals ---
  broken_plurals: BrokenPluralLink[]   # Common broken plural patterns for this noun pattern

  # --- Semantics ---
  semantic_role: string                # e.g., "agent", "patient", "action", "instrument",
                                       # "location", "quality", "comparative", "relational"
  core_meaning: string                 # Pattern meaning in English
  core_meaning_ar: string              # Pattern meaning in Arabic

  # --- Syntactic Properties ---
  syntactic_notes: string | null       # e.g., "can govern object like verb"

  # --- Attestation ---
  attestation: Attestation

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string
```

### 4.2 Supporting Types

```yaml
NounType:
  "masdar" | "ism_fail" | "ism_maful" | "ism_makan" | "ism_zaman" |
  "ism_alah" | "sifah_mushabbahah" | "tafdil" | "nisbah" |
  "jam_taksir" | "ism_marrati" | "ism_hayati" | "ism_jins" |
  "ism_zarf" | "ism_tasghir"

PhonologicalTemplate:
  segments: Segment[]
  vowel_pattern: string[]
  consonant_count: integer
  root_position_map: RootPositionMap
  template_script: string              # e.g., "C₁āC₂iC₃"

Segment:
  type: "root_consonant" | "prefix" | "suffix" | "infix" | "vowel" |
        "gemination_marker" | "long_vowel_marker"
  position: integer | null
  character: string | null
  slot_label: string | null

RootPositionMap:
  mappings: RootPositionMapping[]

RootPositionMapping:
  template_slot: integer
  root_position: integer | 0           # 0 = affix, not a root consonant
  is_geminated: boolean

BrokenPluralLink:
  plural_pattern_id: string            # Reference to KB-0004 broken plural template
  plural_pattern_wazan: string         # e.g., "فُعَّال", "فَعَلَة"
  frequency: "primary" | "common" | "secondary" | "rare"
  notes: string | null

BrokenPluralTemplate:
  id: string                           # "KB-0004:jam_taksir:{canonical}"
  canonical_pattern: string            # e.g., "فُعَّال"
  template: PhonologicalTemplate
  gender: "masculine" | "feminine" | "both"
  declension: "triptote" | "diptote" | "indeclinable"
  applies_to: string[]                 # Which singular patterns use this plural
  nominal_pattern: string              # e.g., "C₁uC₂C₂āC₃"
  frequency: "very_common" | "common" | "moderate" | "rare"

DeclensionClass:
  type: "triptote" | "diptote" | "indeclinable"
  nominative_suffix: string
  accusative_suffix: string
  genitive_suffix: string
  definite_prefix: string              # "ال" standard

WeakRootVariant:
  applies_to: RootType[]               # e.g., ["ajwaf_wawi", "naqis_yai"]
  pattern_change: string               # Description of how pattern changes
  surface_pattern: string              # The resulting surface wazan
  examples: Example[]

Attestation:
  confidence: "certain" | "well_attested" | "attested" | "disputed"
  primary_sources: string[]
  classical_references: string[]
  notes: string | null

Example:
  word: string
  transliteration: string
  meaning: string
  root: string
  context: string | null
```

### 4.3 JSON Example (Active Participle — فَاعِل)

```json
{
  "id": "KB-0004:ism_fail:form_I:فَاعِل",
  "noun_type": "ism_fail",
  "canonical_pattern": "فَاعِل",
  "pattern_variant": "basic",
  "verb_form": 1,
  "derived_from_form": null,
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "long_vowel_marker", "character": "ā", "slot_label": "LONG_A" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "vowel", "character": "i" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "vowel_pattern": ["ā", "i"],
    "consonant_count": 3,
    "root_position_map": {
      "mappings": [
        { "template_slot": 1, "root_position": 1, "is_geminated": false },
        { "template_slot": 2, "root_position": 2, "is_geminated": false },
        { "template_slot": 3, "root_position": 3, "is_geminated": false }
      ]
    },
    "template_script": "C₁āC₂iC₃"
  },
  "gender": "masculine",
  "declension": "triptote",
  "feminine_form": "add_ta_marbuta",
  "feminine_template": "فَاعِلَة",
  "sound_plural_masculine": "فَاعِلُونَ",
  "sound_plural_feminine": "فَاعِلَاتٌ",
  "broken_plurals": [
    { "plural_pattern_id": "KB-0004:jam_taksir:فُعَّال", "plural_pattern_wazan": "فُعَّال", "frequency": "primary" },
    { "plural_pattern_id": "KB-0004:jam_taksir:فَعَلَة", "plural_pattern_wazan": "فَعَلَة", "frequency": "common" },
    { "plural_pattern_id": "KB-0004:jam_taksir:فُعَّل", "plural_pattern_wazan": "فُعَّل", "frequency": "secondary" }
  ],
  "semantic_role": "agent",
  "core_meaning": "Active participle — one who performs the action",
  "core_meaning_ar": "اسم الفاعل — من قام بالفعل",
  "syntactic_notes": "Can govern an object like its source verb; may have active meaning",
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab, Vol. I"],
    "classical_references": ["Al-Kitab", "Sharh al-Ashmuni", "Qatr al-Nada"]
  },
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

---

## 5. Verbal Noun (Masdar) Patterns — Regular

The masdar (مصدر, pl. مصادر) is the verbal noun — it names the action or state expressed by the verb. For Forms II–XV, the masdar pattern is regular and predictable. For Form I, it is largely unpredictable (~40+ patterns).

### 5.1 Form II Masdar: تَفْعِيل

| Property | Value |
|----------|-------|
| **Wazan** | تَفْعِيل |
| **Template** | taC₁C₂īC₃ |
| **Gender** | Masculine |
| **Declension** | Triptote |
| **Sound plural** | تَفْعِيلَات |
| **Broken plurals** | تَفَاعِيل (primary) |
| **Semantic role** | Action/event of Form II verb |
| **Weak root note** | For ajwaf: تَقْوِيل; for naqis: تَدْعِيَة |
| **Example** | تَكْتِيب (taktīb) — \"dictation\" from ك-ت-ب |

### 5.2 Form III Masdar: فِعَال / مُفَاعَلَة

Form III has two masdar patterns, often used interchangeably:

| Property | فِعَال | مُفَاعَلَة |
|----------|--------|-----------|
| **Wazan** | فِعَال | مُفَاعَلَة |
| **Template** | C₁iC₂āC₃ | muC₁āC₂aC₃a |
| **Gender** | Masculine | Feminine |
| **Declension** | Triptote | Triptote |
| **Broken plural** | أَفْعِلَة (rare) | مُفَاعَلَات |
| **Example** | كِتَاب (kitāb) — \"correspondence\" | مُكَاتَبَة (mukātaba) — \"correspondence\" |

### 5.3 Form IV Masdar: إِفْعَال

| Property | Value |
|----------|-------|
| **Wazan** | إِفْعَال |
| **Template** | iC₁C₂āC₃ |
| **Gender** | Masculine |
| **Declension** | Triptote or Diptote |
| **Sound plural** | إِفْعَالَات |
| **Broken plurals** | أَفَاعِيل (primary) |
| **Semantic role** | Causative action |
| **Example** | إِكْتَاب (iktāb) — \"dictation (causing to write)\" |

### 5.4 Form V Masdar: تَفَعُّل

| Property | Value |
|----------|-------|
| **Wazan** | تَفَعُّل |
| **Template** | taC₁aC₂C₂uC₃ |
| **Gender** | Masculine |
| **Declension** | Triptote |
| **Sound plural** | تَفَعُّلَات |
| **Semantic role** | Reflexive/intensive action |
| **Example** | تَكَتُّب (takattub) — \"registration, enrollment\" |

### 5.5 Form VI Masdar: تَفَاعُل

| Property | Value |
|----------|-------|
| **Wazan** | تَفَاعُل |
| **Template** | taC₁āC₂uC₃ |
| **Gender** | Masculine |
| **Declension** | Triptote |
| **Sound plural** | تَفَاعُلَات |
| **Semantic role** | Reciprocal action |
| **Example** | تَكَاتُب (takātub) — \"correspondence (mutual)\" |

### 5.6 Form VII–X Masdar Patterns

| Form | Wazan | Template | Gender | Example |
|------|-------|----------|--------|---------|
| **VII** | اِنْفِعَال | inC₁C₂āC₃ | Masculine | اِنْكِتَاب (inkitāb) — \"subscription\" |
| **VIII** | اِفْتِعَال | iC₁C₂C₂āC₃ | Masculine | اِكْتِتَاب (iktitāb) — \"copying\" |
| **IX** | اِفْعِلَال | iC₁C₂C₂āC₃ | Masculine | اِحْمِرَار (iḥmirār) — \"redness\" |
| **X** | اِسْتِفْعَال | istiC₁C₂āC₃ | Masculine | اِسْتِكْتَاب (istiktāb) — \"request to write\" |

### 5.7 Regular Masdar Summary Table

| Form | Canonical Wazan | Template | Gender | Broken Plurals | Productivity |
|------|----------------|----------|--------|----------------|-------------|
| **II** | تَفْعِيل | taC₁C₂īC₃ | Masc | تَفَاعِيل | 100% regular |
| **III** | فِعَال | C₁iC₂āC₃ | Masc | — | 100% regular |
| **III** | مُفَاعَلَة | muC₁āC₂aC₃a | Fem | — | Common variant |
| **IV** | إِفْعَال | iC₁C₂āC₃ | Masc | أَفَاعِيل | 100% regular |
| **V** | تَفَعُّل | taC₁aC₂C₂uC₃ | Masc | — | 100% regular |
| **VI** | تَفَاعُل | taC₁āC₂uC₃ | Masc | — | 100% regular |
| **VII** | اِنْفِعَال | inC₁C₂āC₃ | Masc | — | 100% regular |
| **VIII** | اِفْتِعَال | iC₁C₂C₂āC₃ | Masc | — | 100% regular |
| **IX** | اِفْعِلَال | iC₁C₂C₂āC₃ | Masc | — | 100% regular |
| **X** | اِسْتِفْعَال | istiC₁C₂āC₃ | Masc | — | 100% regular |

---

## 6. Verbal Noun (Masdar) Patterns — Form I Unpredictable

Form I masdar patterns are the **most complex** aspect of Arabic noun morphology. The pattern cannot be predicted from the verb form alone (unlike Forms II–X). KB-0004 catalogs ~40+ patterns with frequency and root-type applicability data.

### 6.1 High-Frequency Patterns (Very Common)

These patterns cover ~60% of all Form I masdars:

| Pattern | Template | Example | Verb Type | Root Examples |
|---------|----------|---------|-----------|---------------|
| **فَعْل** | C₁aC₂C₃ | ضَرْب (ḍarb) \"hitting\" | Transitive | ض-ر-ب, ق-ت-ل, ض-ر-ب |
| **فِعْل** | C₁iC₂C₃ | عِلْم (ʿilm) \"knowledge\" | Stative | ع-ل-م, ف-ه-م, ف-ك-ر |
| **فُعْل** | C₁uC₂C₃ | حُسْن (ḥusn) \"beauty\" | Quality | ح-س-ن, ك-ر-م, ص-ع-ب |
| **فَعَل** | C₁aC₂aC₃ | طَلَب (ṭalab) \"request\" | Various | ط-ل-ب, ج-م-ع, ن-ظ-ر |
| **فَعَال** | C₁aC₂āC₃ | سَلَام (salām) \"peace\" | Intransitive | س-ل-م, ر-ح-م, ع-ف-و |
| **فِعَال** | C₁iC₂āC₃ | كِتَاب (kitāb) \"writing\" | Action | ك-ت-ب, د-ر-س, ر-س-ل |

### 6.2 Medium-Frequency Patterns

| Pattern | Template | Example | Notes |
|---------|----------|---------|-------|
| **فُعَال** | C₁uC₂āC₃ | غُسَال (ghusāl) \"washing\" | Often indicates process |
| **فَعِيل** | C₁aC₂īC₃ | كَثِير (kathīr) \"multitude\" | Also used for adjectives |
| **فُعُول** | C₁uC₂ūC₃ | جُلُوس (julūs) \"sitting\" | Common for intransitives |
| **فَعَالَة** | C₁aC₂āC₃a | دَرَاسَة (dirāsa) \"study\" | Often feminine |
| **فِعَالَة** | C₁iC₂āC₃a | كِتَابَة (kitāba) \"writing\" | Often indicates profession |
| **فَعُول** | C₁aC₂ūC₃ | رَكُوب (rakūb) \"riding\" | Less common |
| **فَعِيلَة** | C₁aC₂īC₃a | بَقِيعَة (baqīʿa) \"remnant\" | Rare |

### 6.3 Low-Frequency and Rare Patterns

| Pattern | Template | Example | Notes |
|---------|----------|---------|-------|
| **فَعْلَة** | C₁aC₂C₃a | جَلْسَة (jalsa) \"sitting\" | Instance noun (masdar marrah) |
| **فُعْلَة** | C₁uC₂C₃a | رُكْبَة (rukba) \"knee\" | Body part / state |
| **فِعْلَة** | C₁iC₂C₃a | جِلْسَة (jilsa) \"sitting posture\" | Manner noun (masdar hay'ah) |
| **فِعْلَى** | C₁iC₂C₃ā | ذِكْرَى (dhikrā) \"remembrance\" | Feminine |
| **فُعْلَى** | C₁uC₂C₃ā | رُجْعَى (rujʿā) \"return\" | Feminine |
| **فَعَلَان** | C₁aC₂aC₃ān | غَلَيَان (ghalyān) \"boiling\" | Dynamic process |
| **فُعْلَان** | C₁uC₂C₃ān | نُقْصَان (nuqṣān) \"deficiency\" | State/process |
| **فِعْلَان** | C₁iC₂C₃ān | شِبْعَان (shibʿān) \"satiety\" | Rare |
| **فُعُولَة** | C₁uC₂ūC₃a | وُقُوعَة (wuqūʿa) \"occurrence\" | Very rare |
| **فَعَوْلَلَة** | C₁aC₂awC₃aC₄a | جَلْوَلَة (jalwala) \"movement\" | Quadriliteral masdar |

### 6.4 Masdar Pattern Selection Heuristic

For MOD-04: When the Form I masdar pattern is not stored per-root (fallback path):

```pseudo
Algorithm: estimate_form_I_masdar_pattern
Input: root_type (string), verb_semantics (string), transitivity (string)
Output: masdar_pattern_rankings (ordered list of candidate patterns)

1. If root_type in ["ajwaf_wawi", "ajwaf_yai"]:
   → prefer فَعْل (e.g., قَوْل from ق-و-ل, سَيْر from س-ي-ر)
2. If transitivity == "transitive" and semantic_field in ["combat", "physical"]:
   → prefer فَعْل (ضَرْب, قَتْل)
3. If root_type == "doubled":
   → prefer فَعّ (e.g., مَدّ from م-د-د)
4. If transitivity == "intransitive" and semantic_field in ["motion", "state"]:
   → prefer فُعُول (جُلُوس, رُكُوب)
5. If semantic_field in ["emotion", "quality"]:
   → prefer فِعْل or فَعَل (حُزْن, فَرَح)
6. Return ranked list (up to 3 most likely patterns).
accuracy: ~70-80% for top-1, ~90% for top-3.
```

---

## 7. Active Participle (Ism al-Faʿil) Patterns

The active participle (اسم الفاعل, `ism al-fāʿil`) denotes the performer of the action. It is formed regularly from all verb forms.

### 7.1 Form I Active Participle: فَاعِل

| Property | Value |
|----------|-------|
| **Wazan** | فَاعِل |
| **Template** | C₁āC₂iC₃ |
| **Gender** | Masculine (default); feminine: فَاعِلَة |
| **Declension** | Triptote |
| **Sound plural (masc)** | فَاعِلُونَ |
| **Sound plural (fem)** | فَاعِلَاتٌ |
| **Broken plurals** | فُعَّال (primary), فَعَلَة (common), فُعَّل (secondary) |
| **Semantic role** | Agent — the one who performs the action |
| **Syntactic** | May govern object like source verb; active meaning |

### 7.2 Forms II–X Active Participle Patterns

| Form | Wazan | Template | Feminine | Sound Plural (Masc) | Example |
|------|-------|----------|----------|-------------------|---------|
| **II** | مُفَعِّل | muC₁aC₂C₂iC₃ | مُفَعِّلَة | مُفَعِّلُونَ | مُدَرِّس (mudarris) \"teacher\" |
| **III** | مُفَاعِل | muC₁āC₂iC₃ | مُفَاعِلَة | مُفَاعِلُونَ | مُكَاتِب (mukātib) \"correspondent\" |
| **IV** | مُفْعِل | muC₁C₂iC₃ | مُفْعِلَة | مُفْعِلُونَ | مُكْتِب (muktib) \"dictator\" |
| **V** | مُتَفَعِّل | mutaC₁aC₂C₂iC₃ | مُتَفَعِّلَة | مُتَفَعِّلُونَ | مُتَكَاتِب (mutakātib) \"registrant\" |
| **VI** | مُتَفَاعِل | mutaC₁āC₂iC₃ | مُتَفَاعِلَة | مُتَفَاعِلُونَ | مُتَدَرِّس (mutadarras) \"learner\" |
| **VII** | مُنْفَعِل | munC₁aC₂iC₃ | مُنْفَعِلَة | مُنْفَعِلُونَ | مُنْكَاتِب (munkātib) \"subscriber\" |
| **VIII** | مُفْتَعِل | muC₁taC₂iC₃ | مُفْتَعِلَة | مُفْتَعِلُونَ | مُكْتَتِب (muktātib) \"copyist\" |
| **IX** | مُفْعَلّ | muC₁C₂aC₃C₃ | مُفْعَلَّة | مُفْعَلُّونَ | مُحْمَرّ (muḥmarr) \"red one\" |
| **X** | مُسْتَفْعِل | mustaC₁C₂iC₃ | مُسْتَفْعِلَة | مُسْتَفْعِلُونَ | مُسْتَكْتِب (mustaktib) \"requester\" |

### 7.3 Weak Root Variants (Active Participle, Form I)

| Root Type | Surface Pattern | Example | Rule |
|-----------|----------------|---------|------|
| **Sound** | C₁āC₂iC₃ | كَاتِب (kātib) | Regular |
| **Ajwaf wawi** | C₁āʾiC₃ | قَائِل (qāʾil) | C₂ (w) → hamza when between ā and i |
| **Ajwaf yai** | C₁āʾiC₃ | سَائِر (sāʾir) | C₂ (y) → hamza when between ā and i |
| **Naqis wawi** | C₁āC₂in | دَاعٍ (dāʿin) | C₃ (w) drops; noun is diptote in indefinite |
| **Naqis yai** | C₁āC₂in | رَامٍ (rāmin) | C₃ (y) drops; noun is diptote in indefinite |
| **Doubled** | C₁āC₂C₂ | مَادّ (mādd) | C₂ and C₃ merge with shadda |
| **Mithal wawi** | C₁āC₂iC₃ | وَاجِد (wājid) | Regular (waw does NOT drop) |
| **Hamzated (middle)** | C₁āʾiC₃ | سَائِل (sāʾil) | C₂ hamza seat based on vowels |

---

## 8. Passive Participle (Ism al-Mafʿul) Patterns

The passive participle (اسم المفعول, `ism al-mafʿūl`) denotes the recipient of the action.

### 8.1 Form I Passive Participle: مَفْعُول

| Property | Value |
|----------|-------|
| **Wazan** | مَفْعُول |
| **Template** | maC₁C₂ūC₃ |
| **Gender** | Masculine (default); feminine: مَفْعُولَة |
| **Declension** | Triptote |
| **Sound plural (masc)** | مَفْعُولُونَ |
| **Sound plural (fem)** | مَفْعُولَاتٌ |
| **Broken plurals** | مَفَاعِيل (primary), مَفَاعِل (common) |
| **Semantic role** | Patient — the one that receives the action |

### 8.2 Forms II–X Passive Participle Patterns

| Form | Wazan | Template | Feminine | Example |
|------|-------|----------|----------|---------|
| **II** | مُفَعَّل | muC₁aC₂C₂aC₃ | مُفَعَّلَة | مُدَرَّس (mudarras) \"taught\" |
| **III** | مُفَاعَل | muC₁āC₂aC₃ | مُفَاعَلَة | مُكَاتَب (mukātab) \"corresponded\" |
| **IV** | مُفْعَل | muC₁C₂aC₃ | مُفْعَلَة | مُكْتَب (muktab) \"dictated\" |
| **V** | مُتَفَعَّل | mutaC₁aC₂C₂aC₃ | مُتَفَعَّلَة | مُتَكَاتَب (mutakātab) \"registered\" |
| **VI** | مُتَفَاعَل | mutaC₁āC₂aC₃ | مُتَفَاعَلَة | مُتَقَاتَل (mutaqātal) \"combated\" |
| **VII** | مُنْفَعَل | munC₁aC₂aC₃ | مُنْفَعَلَة | مُنْكَتَب (munkatab) \"subscribed to\" |
| **VIII** | مُفْتَعَل | muC₁taC₂aC₃ | مُفْتَعَلَة | مُكْتَتَب (muktatab) \"copied\" |
| **IX** | Not used | — | — | — |
| **X** | مُسْتَفْعَل | mustaC₁C₂aC₃ | مُسْتَفْعَلَة | مُسْتَكْتَب (mustaktab) \"requested\" |

### 8.3 Weak Root Variants (Passive Participle, Form I)

| Root Type | Surface Pattern | Example | Rule |
|-----------|----------------|---------|------|
| **Sound** | maC₁C₂ūC₃ | مَكْتُوب (maktūb) | Regular |
| **Ajwaf wawi** | maC₁ūC₃ | مَقُول (maqūl) | C₂ (w) → ū |
| **Ajwaf yai** | maC₁īC₃ | مَسِير (masīr) | C₂ (y) → ī |
| **Naqis wawi** | maC₁C₂iyy | مَدْعِيّ (madʿiyy) | C₃ (w) → y with shadda |
| **Naqis yai** | maC₁C₂iyy | مَرْمِيّ (marmiyy) | C₃ (y) merges with suffix |
| **Doubled** | maC₁CūC₃ | مَمْدُود (mamdūd) | C₂ and C₃ separate (doubled) |

---

## 9. Noun of Place & Time (Ism al-Makan / al-Zaman)

Nouns of place (اسم المكان, `ism al-makān`) and time (اسم الزمان, `ism al-zamān`) indicate where or when the action occurs.

### 9.1 Primary Patterns

| Pattern | Wazan | Template | Usage | Example |
|---------|-------|----------|-------|---------|
| **مَفْعَل** | مَفْعَل | maC₁C₂aC₃ | Place (Form I, sound) | مَكْتَب (maktab) \"desk/office\" |
| **مَفْعِل** | مَفْعِل | maC₁C₂iC₃ | Time (Form I, sound) | مَوْعِد (mawʿid) \"appointment\" |
| **مَفْعَل** | مَفْعَل | maC₁C₂aC₃ | Place/Time (Form I, ajwaf) | مَقَال (maqāl) \"article/speech\" |

Note: The distinction between مَفْعَل (place) and مَفْعِل (time) is a Classical rule. In practice, مَفْعَل serves both functions for most roots, and the specific meaning depends on the lexical item.

### 9.2 Derived Form Patterns (Forms II–X)

For derived forms, the noun of place/time is formed from the active participle by vowel change or by the masdar with a place suffix:

| Form | Pattern | Template | Example |
|------|---------|----------|---------|
| **II** | مُفَعَّل | muC₁aC₂C₂aC₃ | مُدَرَّس (mudarras) \"school\" (place of study) |
| **III** | مُفَاعَل | muC₁āC₂aC₃ | مُقَاوَل (muqāwal) \"contract office\" |
| **IV** | مُفْعَل | muC₁C₂aC₃ | مُدْخَل (mudkhal) \"entrance\" |
| **V** | مُتَفَعَّل | mutaC₁aC₂C₂aC₃ | مُتَكَاتَب (mutakātab) \"registry\" |
| **VI** | مُتَفَاعَل | mutaC₁āC₂aC₃ | مُتَقَاتَل (mutaqātal) \"battleground\" |
| **VII** | مُنْفَعَل | munC₁aC₂aC₃ | مُنْقَلَب (munqalab) \"turning point\" |
| **VIII** | مُفْتَعَل | muC₁taC₂aC₃ | مُجْتَمَع (mujtamaʿ) \"society\" (place of gathering) |
| **X** | مُسْتَفْعَل | mustaC₁C₂aC₃ | مُسْتَقْبَل (mustaqbal) \"future\" (time) |

### 9.3 Noun of Place/Time Inflection

| Property | مَفْعَل | مَفْعِل |
|----------|---------|---------|
| **Gender** | Masculine | Masculine |
| **Declension** | Triptote | Triptote |
| **Feminine form** | مَفْعَلَة | مَفْعِلَة |
| **Broken plurals** | مَفَاعِل (primary) | مَفَاعِل (primary) |

### 9.4 Weak Root Variants (Noun of Place/Time, Form I)

| Root Type | Pattern | Example | Rule |
|-----------|---------|---------|------|
| **Ajwaf wawi** | مَفْعَل → مَفَال | مَقَال (maqāl) | C₂ → ā between C₁ and C₃ |
| **Naqis yai** | مَفْعَل → مَفْعًى | مَرْمًى (marman) | C₃ (y) → ā alif maqsura |
| **Naqis wawi** | مَفْعَل → مَفْعًى | مَدْعًى (madʿan) | C₃ (w) → ā alif maqsura |
| **Doubled** | مَفْعَل → مَفَلّ | مَحَلّ (maḥall) | C₂=C₃ merge with shadda |

---

## 10. Noun of Instrument (Ism al-Ālah)

The noun of instrument (اسم الآلة, `ism al-ālah`) denotes the tool or instrument used to perform the action.

### 10.1 Instrument Patterns

| Pattern | Wazan | Template | Gender | Example |
|---------|-------|----------|--------|---------|
| **مِفْعَل** | مِفْعَل | miC₁C₂aC₃ | Masculine | مِنْجَل (minjal) \"sickle\" |
| **مِفْعَال** | مِفْعَال | miC₁C₂āC₃ | Masculine | مِفْتَاح (miftāḥ) \"key\" |
| **مِفْعَلَة** | مِفْعَلَة | miC₁C₂aC₃a | Feminine | مِكْنَسَة (miknasa) \"broom\" |
| **مُفْعَل** | مُفْعَل | muC₁C₂aC₃ | Masculine | مُنْخُل (munkhul) \"sieve\" (rare) |
| **فَعَّالَة** | فَعَّالَة | C₁aC₂C₂āC₃a | Feminine | غَسَّالَة (ghassāla) \"washing machine\" (modern) |
| **فَعَّال** | فَعَّال | C₁aC₂C₂āC₃ | Masculine | حَسَّاب (ḥassāb) \"computer\" (modern) |

### 10.2 Instrument Inflection

| Property | مِفْعَل | مِفْعَال | مِفْعَلَة |
|----------|---------|---------|-----------|
| **Gender** | Masculine | Masculine | Feminine |
| **Declension** | Triptote | Triptote | Triptote |
| **Sound plural** | مِفْعَلُونَ | مَفَاعِيل | مِفْعَلَات |
| **Broken plural** | مَفَاعِل | مَفَاعِيل | مَفَاعِل |

### 10.3 Instrument Weak Root Variants

| Root Type | مِفْعَال Variant | Example |
|-----------|-----------------|---------|
| **Ajwaf** | مِفْعَال → مِفْيَال | مِقْيَال (miqyāl) \"measure\" |
| **Naqis** | مِفْعَال → مِفْعَاء | مِرْمَاء (mirmāʾ) \"projectile\" |
| **Doubled** | مِفْعَال → مِفَالّ | مِظَلَّة (miẓalla) \"umbrella\" |

---

## 11. Resembling Adjective (Sifah Mushabbahah)

The resembling adjective (الصفة المشبهة, `al-ṣifah al-mushabbahah`) is a permanent or inherent quality, resembling the active participle in meaning but emphasizing **stability** rather than occurrence.

### 11.1 Resembling Adjective Patterns

| Pattern | Wazan | Template | Gender | Meaning | Example |
|---------|-------|----------|--------|---------|---------|
| **فَعِيل** | فَعِيل | C₁aC₂īC₃ | Both | Permanent quality | كَرِيم (karīm) \"generous\" |
| **فَعْل** | فَعْل | C₁aC₂C₃ | Masc | Quality | ضَخْم (ḍakhm) \"huge\" |
| **فَعِل** | فَعِل | C₁aC₂iC₃ | Masc | State/quality | فَرِح (fariḥ) \"happy\" |
| **فَعْلَان** | فَعْلَان | C₁aC₂C₃ān | Both | Color/emotion | غَضْبَان (ghaḍbān) \"angry\" |
| **أَفْعَل** | أَفْعَل | aC₁C₂aC₃ | Masc (fem: فَعْلَاء) | Color/defect | أَحْمَر (aḥmar) \"red\" |
| **فَعَال** | فَعَال | C₁aC₂āC₃ | Both | Disease/state | صُدَاع (ṣudāʿ) \"headache\" (noun) |
| **فُعَال** | فُعَال | C₁uC₂āC₃ | Both | Disease | زُكَام (zukām) \"cold\" |

### 11.2 Special: أَفْعَل (Elative-Adjective)

The pattern أَفْعَل has two functions:
1. **Resembling adjective** for colors and defects: أَحْمَر (aḥmar) \"red\", أَعْمَى (aʿmā) \"blind\"
2. **Elative** (see Section 12): أَكْبَر (akbar) \"greater\"

For colors/defects, أَفْعَل has a special feminine form فَعْلَاء and broken plural فُعْل:

| Gender | Pattern | Example |
|--------|---------|---------|
| Masculine | أَفْعَل | أَحْمَر (aḥmar) |
| Feminine | فَعْلَاء | حَمْرَاء (ḥamrāʾ) |
| Plural | فُعْل | حُمْر (ḥumr) |

### 11.3 Sifah Mushabbahah Broken Plurals

| Singular Pattern | Broken Plural Patterns | Examples |
|-----------------|----------------------|----------|
| فَعِيل | فِعَال, فُعَلَاء, أَفْعِلَاء | كِرَام (kirām), كُرَمَاء (kuraamāʾ) |
| فَعْلَان | فَعَالَى, فِعَال | غَضَابَى (ghaḍābā), غِضَاب (ghiḍāb) |
| فَعِل | فَعَالَى, فِعَال | فَرَاحَى (farāḥā) — rare |

---

## 12. Elative (Ism al-Tafḍīl)

The elative (اسم التفضيل, `ism al-tafḍīl`) expresses the comparative (\"more X\") or superlative (\"most X\") degree.

### 12.1 Elative Pattern: أَفْعَل

| Property | Value |
|----------|-------|
| **Wazan** | أَفْعَل |
| **Template** | aC₁C₂aC₃ |
| **Gender (masc)** | Masculine: أَفْعَل |
| **Gender (fem)** | Feminine: فُعْلَى (special pattern) |
| **Declension** | Diptote (indefinite) / Triptote (definite) |
| **Broken plural** | أَفَاعِل (rare) |
| **Sound plural (masc)** | أَفْعَلُونَ |
| **Sound plural (fem)** | فُعْلَيَات |
| **Semantic role** | Comparative/superlative |
| **Example** | أَكْبَر (akbar) \"greater/greatest\" |

### 12.2 Elative Usage Distinctions

| Usage | Form | Case/Gender Agreement | Example |
|-------|------|-----------------------|---------|
| **Comparative** (indefinite) | Masculine أَفْعَل | Invariable | هُوَ أَكْبَرُ مِنْهُ (huwa akbaru minhu) |
| **Superlative** (definite) | أَفْعَل + ال | Agrees | هُوَ الْأَكْبَرُ (huwa al-akbaru) |
| **Superlative** (construct) | أَفْعَل + noun | Agrees | أَكْبَرُ الْمُدُنِ (akbaru al-muduni) |
| **Feminine comparative** | فُعْلَى | Full agreement | هِيَ الْكُبْرَى (hiya al-kubrā) |

### 12.3 Weak Root Variants (Elative)

| Root Type | Surface Pattern | Example | Rule |
|-----------|----------------|---------|------|
| **Sound** | aC₁C₂aC₃ | أَكْبَر (akbar) | Regular |
| **Ajwaf wawi** | aC₁waC₃ → أَطْوَل | أَطْوَل (aṭwal) \"longer\" | C₂ surfaces as waw |
| **Ajwaf yai** | aC₁yaC₃ → أَطْيَب | أَطْيَب (aṭyab) \"better\" | C₂ surfaces as ya |
| **Naqis yai** | aC₁Cā | أَدْنَى (adnā) \"nearer\" | C₃ (y) → ā alif maqsura |
| **Naqis wawi** | aC₁Cā | أَعْلَى (aʿlā) \"higher\" | C₃ (w) → ā alif maqsura |
| **Doubled** | aC₁aC₃C₃ | أَشَدّ (ashadd) \"stronger\" | C₂=C₃ merge |
| **Hamzated (first)** | aC₁C₂aC₃ | أَأْكَل (aʾkal) → آكَل (ākal) | Hamza assimilation |

---

## 13. Relative Adjective (Nisbah)

The relative adjective (النسبة, `al-nisbah`) forms adjectives indicating relation or origin, similar to English \"-ic\", \"-ian\", or \"-ese\".

### 13.1 Nisbah Patterns

| Pattern | Wazan | Template | Notes | Example |
|---------|-------|----------|-------|---------|
| **فَعْلِيّ** | فَعْلِيّ | C₁aC₂C₃iyy | Standard masculine | عَرَبِيّ (ʿarabiyy) \"Arabic\" |
| **فِعْلِيّ** | فِعْلِيّ | C₁iC₂C₃iyy | Variant | عِلْمِيّ (ʿilmiyy) \"scientific\" |
| **فُعْلِيّ** | فُعْلِيّ | C₁uC₂C₃iyy | Less common | صُنْعِيّ (ṣunʿiyy) \"artificial\" |
| **فَعَلِيّ** | فَعَلِيّ | C₁aC₂aC₃iyy | Extended | جَبَلِيّ (jabaliyy) \"mountainous\" |

### 13.2 Nisbah Inflection

| Property | Value |
|----------|-------|
| **Gender (masc)** | فَعْلِيّ (with shadda on yā) |
| **Gender (fem)** | فَعْلِيَّة (add tāʾ marbūṭa) |
| **Sound plural (masc)** | فَعْلِيُّونَ |
| **Sound plural (fem)** | فَعْلِيَّات |
| **Declension** | Triptote |

### 13.3 Nisbah Formation Rules (Orthographic)

Nisbah formation involves specific orthographic changes to the base noun:

| Base Noun Ending | Nisbah Rule | Example |
|-----------------|-------------|---------|
| **تاء مربوطة (ة)** | Drop ة, add يّ | مَكَّة (makka) → مَكِّيّ (makkiyy) |
| **Alif maqsura (ى)** | Change to و before يّ | فَتًى (fatan) → فَتَوِيّ (fatawiyy) |
| **Alif mamduda (اء)** | Change to ائ or أو before يّ | صَحْرَاء (ṣaḥrāʾ) → صَحْرَاوِيّ (ṣaḥrāwiyy) |
| **Long ā (ا)** | Drop ا if middle, keep و | كِتَاب (kitāb) → كِتَابِيّ (kitābiyy) |
| **Diptote -ān** | Add يّ with adjustments | عُمْان (ʿumān) → عُمَانِيّ (ʿumāniyy) |

These are orthographic rules applied by MOD-03, not stored as pattern variants in KB-0004. KB-0004 stores the underlying nisbah pattern templates.

---

## 14. Broken Plural (Jamʿ al-Taksir) Patterns

Broken plurals (جموع التكسير, `jumūʿ al-taksīr`) are formed by **internal** modification of the singular stem, unlike sound plurals which add suffixes. KB-0004 catalogs ~30+ broken plural templates.

### 14.1 Plural of Paucity (جمع قلة)

The plural of paucity is used for counts of 3–10:

| Pattern | Wazan | Template | Example | Notes |
|---------|-------|----------|---------|-------|
| **أَفْعُل** | أَفْعُل | aC₁C₂uC₃ | أَكْلُب (aklub) \"dogs\" | Least common paucity pattern |
| **أَفْعَال** | أَفْعَال | aC₁C₂āC₃ | أَجْوِبَة (ajwiba) \"answers\" | Very common |
| **أَفْعِلَة** | أَفْعِلَة | aC₁C₂iC₃a | أَسْئِلَة (asʾila) \"questions\" | Common for non-human plurals |
| **فِعْلَة** | فِعْلَة | C₁iC₂C₃a | فِتْيَة (fitya) \"youths\" | Rare |

### 14.2 Plural of Multitude (جمع كثرة)

The plural of multitude is for counts of 10+ or indefinite large numbers:

| Pattern | Wazan | Template | Example | Notes |
|---------|-------|----------|---------|-------|
| **فُعْل** | فُعْل | C₁uC₂C₃ | حُمْر (ḥumr) \"red ones\" | Colors/defects |
| **فِعَل** | فِعَل | C₁iC₂aC₃ | قِطَع (qiṭaʿ) \"pieces\" | Common |
| **فُعَل** | فُعَل | C₁uC₂aC₃ | دُرَر (durar) \"pearls\" | Doubled roots |
| **فُعَلَاء** | فُعَلَاء | C₁uC₂aC₃āʾ | كُرَمَاء (kuraamāʾ) \"generous\" | فَعِيل pattern |
| **فُعَّال** | فُعَّال | C₁uC₂C₂āC₃ | كُتَّاب (kuttāb) \"writers\" | فَاعِل pattern |
| **فَعَلَة** | فَعَلَة | C₁aC₂aC₃a | كَتَبَة (kataba) \"scribes\" | Professions |
| **فِعَال** | فِعَال | C₁iC₂āC₃ | كِرَام (kirām) \"noble ones\" | فَعِيل pattern |
| **فُعُول** | فُعُول | C₁uC₂ūC₃ | جُلُوس (julūs) \"assembly\" | Collective |
| **فَوَاعِل** | فَوَاعِل | C₁awāC₂iC₃ | جَوَارِب (jawārib) \"socks\" | Quadriliteral |
| **فَعَالِل** | فَعَالِل | C₁aC₂āC₃iC₄ | زَلازِل (zalāzil) \"earthquakes\" | Quadriliteral plurals |
| **مَفَاعِل** | مَفَاعِل | maC₁āC₂iC₃ | مَكَاتِب (makātib) \"offices\" | مَفْعَل pattern |
| **مَفَاعِيل** | مَفَاعِيل | maC₁āC₂īC₃ | مَفَاتِيح (mafātīḥ) \"keys\" | مِفْعَال pattern |

### 14.3 Broken Plural to Singular Mapping

KB-0004 stores the **reverse mapping** — which singular patterns map to which broken plurals:

```yaml
BrokenPluralMapping:
  singular_pattern: "فَاعِل"
  plural_patterns:
    - pattern: "فُعَّال"
      frequency: "primary"
      notes: "For human agents (kuttāb)"
    - pattern: "فَعَلَة"
      frequency: "common"
      notes: "For professions (kataba)"
    - pattern: "فُعَّل"
      frequency: "secondary"
      notes: "For participles with shadda on C₂ (ḥujjār)"
    - pattern: "فِعَال"
      frequency: "secondary"
      notes: "For non-human"
    - pattern: "أَفْعِلَة"
      frequency: "rare"
      notes: "دَواب (dawābb) — four-footed animals"
```

### 14.4 Broken Plural Concordance Table (Selected)

| Singular Pattern | Primary BP | Secondary BP | Tertiary BP |
|-----------------|-----------|-------------|------------|
| فَاعِل | فُعَّال | فَعَلَة | فُعَّل |
| فَعِيل | فِعَال | فُعَلَاء | أَفْعِلَاء |
| فَعْل | فُعُول | أَفْعَال | فِعَال |
| فِعْل | فُعُول | أَفْعَال | فِعَل |
| فُعْل | فِعَال | أَفْعَال | — |
| فَعَل | أَفْعَال | فِعَال | — |
| مَفْعَل | مَفَاعِل | — | — |
| مِفْعَال | مَفَاعِيل | — | — |
| فَعْلَان | فِعَال | فَعَالَى | — |
| أَفْعَل | فُعْل | — | — |

---

## 15. Other Noun Patterns

### 15.1 Instance Noun (Ism Marrah — اسم المرة)

The instance noun indicates a **single occurrence** of the action.

| Property | Value |
|----------|-------|
| **Wazan** | فَعْلَة (Form I) |
| **Template** | C₁aC₂C₃a |
| **Gender** | Feminine (by tāʾ marbūṭa) |
| **Declension** | Triptote |
| **Sound plural** | فَعْلَات |
| **Example** | ضَرْبَة (ḍarba) \"a hit/strike\" |

For derived forms (II–X), the instance noun is formed by adding tāʾ marbūṭa to the regular masdar:

| Form | Instance Noun | Example |
|------|--------------|---------|
| II | تَفْعِيلَة | تَكْتِيبَة (taktība) \"a dictation\" |
| III | فِعَالَة | كِتَابَة (kitāba) \"a writing\" |
| VIII | اِفْتِعَالَة | اِكْتِتَابَة (iktitāba) \"a copying\" |

### 15.2 Manner Noun (Ism Hay'ah — اسم الهيئة)

The manner noun indicates the **way/manner** in which an action is performed.

| Property | Value |
|----------|-------|
| **Wazan** | فِعْلَة (Form I) |
| **Template** | C₁iC₂C₃a |
| **Gender** | Feminine |
| **Declension** | Triptote |
| **Example** | جِلْسَة (jilsa) \"sitting posture\" |

### 15.3 Diminutive (Ism Tasghir — اسم التصغير)

The diminutive indicates smallness or endearment, formed by a fixed pattern:

| Pattern | Template | Example |
|---------|----------|---------|
| **فُعَيْل** | C₁uC₂ayC₃ | كُتَيْب (kutayb) \"little book\" |
| **فُعَيْعِل** | C₁uC₂ayC₃iC₄ | دُحَيْرِج (duḥayrij) \"little ball\" (quadriliteral) |

The diminutive is **semi-productive** in MSA and **fully productive** in Classical Arabic. KB-0004 defines the pattern templates but actual diminutive forms are computed by rule rather than stored individually.

### 15.4 NomenVerbi (Ism Jins — اسم الجنس)

The generic noun indicates the action as a **generic concept** rather than a specific event:

| Pattern | Template | Example | Notes |
|---------|----------|---------|-------|
| **فَعَل** | C₁aC₂aC₃ | جَمَل (jamal) \"generosity\" | Often synonymous with masdar |
| **فِعَال** | C₁iC₂āC₃ | جَمَال (jamāl) \"beauty\" | Common for abstract concepts |

---

## 16. Weak Root Variants for Noun Patterns

### 16.1 Active Participle Weak Variants (Full Table)

| Verb Form | Root Type | Surface Pattern | Example |
|-----------|-----------|----------------|---------|
| **I** | Ajwaf wawi | C₁āʾiC₃ | قَائِل (qāʾil) |
| **I** | Ajwaf yai | C₁āʾiC₃ | سَائِر (sāʾir) |
| **I** | Naqis wawi | C₁āC₂in (diptote) | دَاعٍ (dāʿin) |
| **I** | Naqis yai | C₁āC₂in (diptote) | رَامٍ (rāmin) |
| **I** | Doubled | C₁āC₂C₂ | مَادّ (mādd) |
| **II** | Ajwaf | muC₁awC₂iC₃ | مُقَوِّل (muqawwil) |
| **II** | Naqis | muC₁aC₂C₂i | مُرَمٍّ (murammin) |
| **III** | Ajwaf | muC₁āwC₂iC₃ | — |
| **IV** | Ajwaf | muC₁īC₂iC₃ | مُقِيل (muqīl) |
| **X** | Ajwaf | mustaC₁īC₂iC₃ | مُسْتَقِيل (mustaqīl) |

### 16.2 Passive Participle Weak Variants (Form I)

| Root Type | Surface Pattern | Example |
|-----------|----------------|---------|
| Sound | maC₁C₂ūC₃ | مَكْتُوب (maktūb) |
| Ajwaf wawi | maC₁ūC₃ | مَقُول (maqūl) |
| Ajwaf yai | maC₁īC₃ | مَسِير (masīr) |
| Naqis wawi | maC₁C₂iyy | مَدْعِيّ (madʿiyy) |
| Naqis yai | maC₁C₂iyy | مَرْمِيّ (marmiyy) |
| Doubled | maC₁dūd | مَمْدُود (mamdūd) |

### 16.3 Masdar Weak Variants (Derived Forms)

| Form | Root Type | Surface Masdar | Rule |
|------|-----------|---------------|------|
| **II** | Ajwaf wawi | تَقْوِيل (taqwīl) | C₂ (w) surfaces as waw |
| **II** | Naqis yai | تَدْعِيَة (tadʿiya) | C₃ (y) → y + tāʾ marbūṭa |
| **IV** | Ajwaf wawi | إِقَالَة (iqāla) | C₂ elided, tāʾ marbūṭa added |
| **VIII** | Ajwaf wawi | اِقْتِيَال (iqtiyāl) | C₂ → y |
| **X** | Ajwaf wawi | اِسْتِقَالَة (istiqāla) | C₂ elided, tāʾ marbūṭa added |
| **X** | Naqis yai | اِسْتِدْعَاء (istidʿāʾ) | C₃ → hamza |

### 16.4 Broken Plural Weak Variants

| Singular Pattern | Root Type | Broken Plural | Rule |
|-----------------|-----------|---------------|------|
| فَاعِل | Ajwaf | فُوَّال | C₂ (w) → w w/ shadda |
| فَعِيل | Naqis | فُعَلَاء → فُعَل | C₃ (y) drops |
| مَفْعَل | Ajwaf | مَفَاعِل → مَفَائِل | C₂ → hamza |

---

## 17. Noun Pattern Matching Algorithm

### 17.1 Primary Algorithm: Identify Noun Pattern

```pseudo
Algorithm: identify_noun_pattern
Input: noun_stem (string), root (RootEntry), pattern_hint (NounType | null)
Output: NounPatternMatch[]

1. Preprocessing:
   a. Strip any tāʾ marbūṭa (ـة) if present and note gender.
   b. Strip any feminine alif (ـاء) or alif maqsura (ـى).
   c. Record the stripped form as the candidate stem.
   d. Identify root consonants from KB-0001.

2. Determine candidate noun types:
   a. If pattern_hint is provided:
      i.   Load noun patterns for that specific noun type.
   b. If no pattern_hint:
      i.   Try all noun patterns (masdar → participle → place → instrument → etc.).
      ii.  Order by probability: active participle > passive participle >
           masdar > adjective > place > elative > instrument.

3. For each candidate noun pattern p:
   a. Retrieve the phonological template from the pattern entry.
   b. Map root consonants to template slots.
   c. Fill the template.
   d. Compare against the input noun stem.
   e. If match → record match:
      i.   Pattern ID
      ii.  Noun type
      iii. Gender (infer from form)
      iv.  Broken plural candidates
      v.   Confidence score

4. For weak roots:
   a. Try weak root variant patterns.
   b. Apply phonological rules (hamza seat, elision, etc.).

5. Score matches by:
   a. Template match accuracy (exact > partial).
   b. Root type compatibility.
   c. Pattern productivity (common > rare).
   d. Semantic congruity.

6. Return ordered NounPatternMatch[].
```

### 17.2 Secondary Algorithm: Generate Noun Plural

```pseudo
Algorithm: generate_noun_plural
Input: noun_stem (string), noun_pattern (NounPatternEntry),
       count_category ("paucity" | "multitude" | "sound")
Output: string[] (candidate plural forms)

1. Determine available plural strategies:
   a. From noun_pattern.broken_plurals → list broken plural templates.
   b. From noun_pattern.sound_plural_masculine/feminine → sound plurals.

2. If count_category == "sound":
   a. Apply sound masculine plural template (if applicable).
   b. Apply sound feminine plural template (if applicable).
   c. Return sound plural forms.

3. If count_category in ["paucity", "multitude"]:
   a. For each broken plural template (ordered by frequency):
      i.   Apply the broken plural pattern to the stem.
      ii.  Handle weak root variations.
      iii. Record the resulting plural form.
   b. Return up to 3 top-frequency broken plural forms.

4. Apply gender and declension:
   a. Set case endings based on declension class.
   b. Apply definiteness markers if applicable.

5. Return string[] of candidate plural forms.
```

---

## 18. Serialization & Storage

### 18.1 Source Format

```diff
  /knowledge/KB-0004/
  ├── metadata.yaml                     # KB metadata (version, counts)
  ├── masdars/
  │   ├── form-I.yaml                   # Form I masdar patterns (~40+)
  │   ├── form-II.yaml                  # تَفْعِيل
  │   ├── form-III.yaml                 # فِعَال / مُفَاعَلَة
  │   ├── form-IV.yaml                  # إِفْعَال
  │   ... (V–X)
  │   └── other-masdars.yaml            # Masdar marrah, hay'ah, jins
  ├── participles/
  │   ├── active-participle.yaml        # فَاعِل + derived form patterns
  │   ├── passive-participle.yaml       # مَفْعُول + derived form patterns
  │   └── resembling-adjective.yaml     # Sifah mushabbahah patterns
  ├── place-time.yaml                   # Ism makan/zaman patterns
  ├── instrument.yaml                   # Ism ālah patterns
  ├── elative.yaml                      # Tafḍīl patterns
  ├── nisbah.yaml                       # Relative adjective patterns
  ├── broken-plurals/
  │   ├── plural-of-paucity.yaml        # أَفْعَال, أَفْعِلَة, etc.
  │   ├── plural-of-multitude.yaml      # فُعُول, فِعَال, etc.
  │   └── pattern-mappings.yaml         # Map singular → broken plural patterns
  ├── weak-variants/
  │   ├── active-participle.yaml        # Weak root variants for ism fāʿil
  │   ├── passive-participle.yaml       # Weak root variants for ism mafʿūl
  │   ├── masdars.yaml                  # Weak root variants for masdars
  │   └── elative.yaml                  # Weak root variants for tafḍīl
  └── other.yaml                        # Diminutive, nomen verbi, etc.
```

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0004"
title: "Noun Patterns — Derived Noun Specifications"
version: "1.0.0"
status: "draft" | "review" | "published"

noun_pattern_count: 156
broken_plural_count: 34

created_at: "2026-07-15T00:00:00Z"
updated_at: "2026-07-15T00:00:00Z"

sources:
  - name: "Sibawayh, Al-Kitab"
    version: "critical_1988"
  - name: "Al-Muqtadab (Al-Mubarrad)"
    version: "print_1979"
  - name: "Sharh al-Ashmuni"
    version: "print_1963"
  - name: "Wright's Arabic Grammar"
    version: "3rd_edition"

checksum_sha256: "d4e5f6a7b8c9..."
maintainers:
  - name: "Dr. [Name]"
    email: "[email]"
    role: "noun_morphology_editor"
```

### 18.2 Compiled Format (Table Binary)

```diff
  Compiled Noun Pattern Binary:
  ┌──────────────────────────────────────────────────────────────┐
  │ HEADER                                                       │
  │ ├── magic: "AGOSKB04" (8 bytes)                             │
  │ ├── version: major(2B) + minor(2B) + patch(2B)              │
  │ ├── pattern_count: u32 (4 bytes)                            │
  │ ├── plural_template_count: u32 (4 bytes)                    │
  │ ├── weak_variant_count: u32 (4 bytes)                       │
  │ ├── pattern_index_offset: u32 (4 bytes)                     │
  │ ├── plural_index_offset: u32 (4 bytes)                      │
  │ ├── string_table_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ PATTERN INDEX                                                │
  │ ├── Fixed-size entries (96 bytes each)                      │
  │ │   ├── noun_type: u8                                       │
  │ │   ├── verb_form: u8 | 0xFF                                │
  │ │   ├── gender: u8 (0=masc, 1=fem, 2=both)                 │
  │ │   ├── declension: u8 (0=triptote, 1=diptote, 2=indecl)   │
  │ │   ├── template_signature: u64                             │
  │ │   ├── feminine_template_offset: u32 (→ string table)      │
  │ │   ├── sound_plural_m_offset: u32                          │
  │ │   ├── sound_plural_f_offset: u32                          │
  │ │   ├── broken_plural_count: u8                             │
  │ │   ├── broken_plural_refs: u32 × 4 (max)                  │
  │ │   ├── weak_variant_offset: u32                            │
  │ │   └── feature_bitmask: u32                                │
  │ └── ... (pattern_count entries)                             │
  ├──────────────────────────────────────────────────────────────┤
  │ BROKEN PLURAL INDEX                                          │
  │ ├── Plural templates with pattern signatures                │
  │ ├── Mapping entries singular_pattern_ref → plural_id[]      │
  │ └── Frequency rankings for disambiguation                   │
  ├──────────────────────────────────────────────────────────────┤
  │ WEAK VARIANT TABLE                                           │
  │ ├── Root-type indexed variant rules                         │
  │ ├── Per (noun_type, root_type) override rules               │
  │ └── Surface pattern references                              │
  ├──────────────────────────────────────────────────────────────┤
  │ STRING TABLE                                                 │
  │ ├── Length-prefixed UTF-8 strings                           │
  │ ├── Pattern names, templates, examples                      │
  │ └── Referenced by offsets from all tables                   │
  └──────────────────────────────────────────────────────────────┘
```

#### C Struct: Noun Pattern Index Entry

```c
struct NounPatternEntry {
    uint8_t  noun_type;                    // From NounType enum
    uint8_t  verb_form;                    // I–XV, or 0xFF if none
    uint8_t  gender;                       // 0=masc, 1=fem, 2=both
    uint8_t  declension;                   // 0=triptote, 1=diptote
    uint64_t template_signature;           // Compact pattern signature
    uint32_t canonical_pattern_offset;     // → string table
    uint32_t feminine_template_offset;     // → string table (0 = none)
    uint32_t broken_plural_refs[4];        // → broken plural templates
    uint8_t  broken_plural_count;          // How many are valid
    uint8_t  weak_variant_count;           // Number of weak root variants
    uint16_t weak_variant_offset;          // → weak variant table
    uint16_t semantic_role;                // Mapped from semantic role enum
    uint8_t  padding[3];                  // Alignment
};
```

### 18.3 File Packaging

```diff
  KB-0004-v1.0.0.agos-kb              # Compiled pattern binary
  KB-0004-v1.0.0.agos-kb.sig          # Ed25519 signature
  KB-0004-v1.0.0.agos-kb.sha256       # SHA-256 checksum
  KB-0004-v1.0.0.source.tar.gz        # Source YAML files (optional)
```

### 18.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Pattern index | 2 MB | 3 MB | ~160 entries × 96 bytes |
| Broken plural index | 1 MB | 2 MB | ~35 templates + mapping table |
| Weak variant table | 2 MB | 6 MB | Weak root variant descriptions |
| String table | 3 MB | 8 MB | Pattern names, templates, examples |
| Mapping data | 1 MB | 5 MB | Singular→plural mappings |
| Plural paradigm cache | 1 MB | 6 MB | Pre-computed plural forms |
| **Total** | **~10 MB** | **~30 MB** | Memory-mapped load |

---

## 19. Versioning & Evolution

### 19.1 Versioning Scheme

KB-0004 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to noun pattern schema, format change | `1.0.0` → `2.0.0` | Requires KB conversion tool, invalidates all caches |
| **MINOR** | Addition of new noun patterns, new broken plural templates, new optional fields | `1.0.0` → `1.1.0` | Backward-compatible; existing pattern IDs remain valid |
| **PATCH** | Corrections to pattern definitions, improved plural mappings, typo fixes | `1.0.0` → `1.0.1` | Backward-compatible; no schema changes |

### 19.2 Cross-KB Compatibility

```yaml
cross_kb_compatibility:
  KB-0001: ">= 1.0.0"       # Noun patterns reference root types and derived_nouns
  KB-0002: ">= 1.0.0"       # Noun pattern wazans link from KB-0002
  KB-0003: ">= 1.0.0"       # Shared paradigm patterns
  KB-0005: ">= 1.0.0"       # Independent (no noun pattern dependency)
  KB-0006: ">= 1.0.0"       # Independent (no noun pattern dependency)
  KB-0007: ">= 1.0.0"       # noun_type, case, state, gender features
```

### 19.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add new noun pattern type | MINOR | Add pattern definitions, update index |
| Correct broken plural mapping | PATCH | Edit mapping entry, regenerate index |
| Add weak root variant | MINOR | Add variant entry, link to base pattern |
| Add new broken plural template | MINOR | Add template definition, update mapping |
| Remove noun pattern | MAJOR | Only for demonstrably incorrect entries |
| Add new optional field | MINOR | Add field, existing entries remain valid |

---

## 20. Quality Requirements

### 20.1 Completeness Targets

| Category | Minimum | Target | Stretch |
|----------|---------|--------|---------|
| Masdar patterns (Form I, common ~20) | 90% | 95% | 100% |
| Masdar patterns (Form I, all ~40) | 75% | 85% | 95% |
| Regular masdar patterns (II–X) | 100% | 100% | 100% |
| Active participle patterns (I–X) | 100% | 100% | 100% |
| Passive participle patterns (I–X, exc IX) | 100% | 100% | 100% |
| Noun of place/time | 100% | 100% | 100% |
| Noun of instrument | 100% | 100% | 100% |
| Resembling adjective patterns | 90% | 95% | 100% |
| Elative (tafḍīl) | 100% | 100% | 100% |
| Nisbah patterns | 100% | 100% | 100% |
| Broken plural templates (paucity) | 100% | 100% | 100% |
| Broken plural templates (multitude) | 80% | 90% | 100% |
| Weak root variants (active participle) | 90% | 95% | 100% |
| Weak root variants (passive participle) | 85% | 90% | 95% |
| Weak root variants (masdar) | 80% | 90% | 95% |
| Instance/manner noun patterns | 100% | 100% | 100% |

### 20.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Pattern-to-template consistency | 100% — template must match canonical pattern | Automated cross-check |
| Gender assignment | 100% — default gender must be correct | Automated validation |
| Broken plural mapping | ≥ 95% — plural patterns must match attested usage | Comparison with reference |
| Weak root variant correctness | 100% — weak transformations produce correct forms | Per-class regression |
| Cross-KB consistency (KB-0002) | 100% — referenced wazans must exist | Automated cross-KB check |
| Cross-KB consistency (KB-0001) | 100% — noun types referenced must exist | Automated cross-KB check |
| Unicode normalization | 100% — all Arabic text valid NFC-normalized UTF-8 | Automated encoding check |

### 20.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate YAML structure
  ├── schema: validate against KB-0004 JSON Schema
  ├── template_check: verify each pattern's template matches its canonical form
  ├── gender_check: verify feminine forms are consistent
  ├── broken_plural_check: verify plural mappings reference valid templates
  └── lint: field presence, Arabic-only text

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── cross_kb: verify noun pattern IDs exist in KB-0002
  ├── cross_kb: verify noun types match KB-0001 derived_nouns
  ├── weak_regression: verify known noun patterns with weak roots
  ├── broken_plural_coverage: verify plural mappings for all patterns
  ├── compilation: verify table binary compiles without error
  ├── size_budget: verify compiled size ≤ 30 MB
  └── regression: verify 100+ known nouns produce correct patterns

  Review (manual, per release):
  ├── sample_check: linguist reviews 3% random pattern sample
  ├── hotspot_check: review patterns modified since last version
  ├── broken_plural_audit: verify plural mappings against Wright's Grammar
  └── changelog: verify changelog accuracy
```

### 20.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Noun pattern lookup (template match) | < 2 μs | Per lookup, average |
| Noun pattern lookup (p99) | < 5 μs | Per lookup, 99th percentile |
| Broken plural generation | < 5 μs | Per noun pattern, average |
| Weak root variant lookup | < 3 μs | Per variant, average |
| Full noun analysis (10 candidates) | < 30 μs | Per stem, average |
| KB load time (compact) | < 25 ms | mmap + verify checksum |
| KB load time (full) | < 50 ms | mmap + verify checksum |
| Memory (compact) | ~10 MB | RSS |
| Memory (full) | ~30 MB | RSS |

---

## 21. Example Entries

### 21.1 Active Participle — Form I (فَاعِل)

```json
{
  "id": "KB-0004:ism_fail:form_I:فَاعِل",
  "noun_type": "ism_fail",
  "canonical_pattern": "فَاعِل",
  "pattern_variant": "basic",
  "verb_form": 1,
  "template": {
    "segments": [
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "long_vowel_marker", "character": "ā" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "vowel", "character": "i" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "template_script": "C₁āC₂iC₃"
  },
  "gender": "masculine",
  "declension": "triptote",
  "feminine_form": "add_ta_marbuta",
  "feminine_template": "فَاعِلَة",
  "sound_plural_masculine": "فَاعِلُونَ",
  "sound_plural_feminine": "فَاعِلَاتٌ",
  "broken_plurals": [
    { "plural_pattern_id": "KB-0004:jam_taksir:فُعَّال", "plural_pattern_wazan": "فُعَّال", "frequency": "primary" },
    { "plural_pattern_id": "KB-0004:jam_taksir:فَعَلَة", "plural_pattern_wazan": "فَعَلَة", "frequency": "common" },
    { "plural_pattern_id": "KB-0004:jam_taksir:فُعَّل", "plural_pattern_wazan": "فُعَّل", "frequency": "secondary" }
  ],
  "semantic_role": "agent",
  "core_meaning": "Active participle — one who performs the action",
  "core_meaning_ar": "اسم الفاعل",
  "attestation": { "confidence": "certain" }
}
```

### 21.2 Passive Participle — Form I (مَفْعُول)

```json
{
  "id": "KB-0004:ism_maful:form_I:مَفْعُول",
  "noun_type": "ism_maful",
  "canonical_pattern": "مَفْعُول",
  "pattern_variant": "basic",
  "verb_form": 1,
  "template": {
    "segments": [
      { "type": "prefix", "character": "م", "slot_label": "PREF_M" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "long_vowel_marker", "character": "ū" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "template_script": "maC₁C₂ūC₃"
  },
  "gender": "masculine",
  "declension": "triptote",
  "feminine_form": "add_ta_marbuta",
  "feminine_template": "مَفْعُولَة",
  "sound_plural_masculine": "مَفْعُولُونَ",
  "sound_plural_feminine": "مَفْعُولَاتٌ",
  "broken_plurals": [
    { "plural_pattern_id": "KB-0004:jam_taksir:مَفَاعِيل", "plural_pattern_wazan": "مَفَاعِيل", "frequency": "primary" },
    { "plural_pattern_id": "KB-0004:jam_taksir:مَفَاعِل", "plural_pattern_wazan": "مَفَاعِل", "frequency": "common" }
  ],
  "semantic_role": "patient",
  "core_meaning": "Passive participle — one that receives the action",
  "core_meaning_ar": "اسم المفعول",
  "attestation": { "confidence": "certain" }
}
```

### 21.3 Masdar — Form II Regular (تَفْعِيل)

```json
{
  "id": "KB-0004:masdar:form_II:تَفْعِيل",
  "noun_type": "masdar",
  "canonical_pattern": "تَفْعِيل",
  "verb_form": 2,
  "derived_from_form": 2,
  "template": {
    "segments": [
      { "type": "prefix", "character": "ت", "slot_label": "PREF_T" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "long_vowel_marker", "character": "ī" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "template_script": "taC₁C₂īC₃"
  },
  "gender": "masculine",
  "declension": "triptote",
  "broken_plurals": [
    { "plural_pattern_id": "KB-0004:jam_taksir:تَفَاعِيل", "plural_pattern_wazan": "تَفَاعِيل", "frequency": "primary" }
  ],
  "semantic_role": "action",
  "core_meaning": "Verbal noun of Form II — the act of {verb}",
  "core_meaning_ar": "مصدر الفعل الثلاثي المزيد (فَعَّلَ)",
  "attestation": { "confidence": "certain" }
}
```

### 21.4 Noun of Place — Form I (مَفْعَل)

```json
{
  "id": "KB-0004:ism_makan:form_I:مَفْعَل",
  "noun_type": "ism_makan",
  "canonical_pattern": "مَفْعَل",
  "pattern_variant": "basic",
  "verb_form": 1,
  "template": {
    "segments": [
      { "type": "prefix", "character": "م", "slot_label": "PREF_M" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "template_script": "maC₁C₂aC₃"
  },
  "gender": "masculine",
  "declension": "triptote",
  "sound_plural_feminine": "مَفْعَلَات",
  "broken_plurals": [
    { "plural_pattern_id": "KB-0004:jam_taksir:مَفَاعِل", "plural_pattern_wazan": "مَفَاعِل", "frequency": "primary" }
  ],
  "semantic_role": "location",
  "core_meaning": "Noun of place — where the action occurs",
  "core_meaning_ar": "اسم المكان",
  "attestation": { "confidence": "certain" }
}
```

### 21.5 Elative — Form I (أَفْعَل)

```json
{
  "id": "KB-0004:tafdil:أَفْعَل",
  "noun_type": "tafdil",
  "canonical_pattern": "أَفْعَل",
  "verb_form": null,
  "template": {
    "segments": [
      { "type": "prefix", "character": "أ", "slot_label": "PREF_ALIF" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 1, "slot_label": "C₁" },
      { "type": "root_consonant", "position": 2, "slot_label": "C₂" },
      { "type": "vowel", "character": "a" },
      { "type": "root_consonant", "position": 3, "slot_label": "C₃" }
    ],
    "template_script": "aC₁C₂aC₃"
  },
  "gender": "masculine",
  "feminine_form": "special_فُعْلَى",
  "feminine_template": "فُعْلَى",
  "declension": "diptote",
  "sound_plural_masculine": "أَفْعَلُونَ",
  "sound_plural_feminine": "فُعْلَيَات",
  "semantic_role": "comparative",
  "core_meaning": "Elative/comparative — more X, most X",
  "core_meaning_ar": "اسم التفضيل",
  "attestation": { "confidence": "certain" }
}
```

---

## 22. Cross-References

### 22.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0004 in module catalog |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Noun pattern analysis using KB-0004 |
| SPEC-0001-C3 | Compilation Pipeline (MOD-08) | Noun feature resolution using KB-0004 |
| SPEC-0001-C3 | Compilation Pipeline (MOD-09) | Noun form generation using KB-0004 |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Noun pattern interface in MOD-04/MOD-08 |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | Noun features in IR-4/IR-8 |
| SPEC-0001-C6 | Deployment & Runtime Considerations | KB bundling, size budget |
| SPEC-0001-C8 | Security, Validation & Error Handling | KB integrity verification |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0004 size, pattern lookup performance |
| KB-0001 | Roots Database | Root types and derived_nouns referencing noun patterns |
| KB-0002 | Wazan Database | Noun pattern wazans that KB-0004 extends |
| KB-0003 | Verb Forms | Verb form paradigms linking to masdar patterns |
| KB-0007 | Morphological Features | Feature taxonomy for nouns |

### 22.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (8th C. CE) | Foundational grammar; defines the noun derivation system |
| Al-Mubarrad, Al-Muqtadab (9th C. CE) | Analysis of derived noun patterns |
| Ibn Hisham, Qatr al-Nada (14th C. CE) | I'rab-based noun analysis |
| Al-Ashmuni, Sharh al-Ashmuni (15th C. CE) | Comprehensive grammatical commentary |
| Wright's Arabic Grammar (1859) | Western reference for Arabic noun morphology |
| Lane's Arabic-English Lexicon | Reference for noun pattern meanings |
| Buckwalter Arabic Morphological Analyzer | Reference for computational noun pattern matching |
| Haywood & Nahmad, A New Arabic Grammar | Modern pedagogical reference |

---

## Progress Summary

**KB-0004: Noun Patterns — Derived Noun Specifications**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Nouns in Arabic Morphology | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Noun Pattern Entry Schema | ✓ COMPLETE |
| Section 5 | Verbal Noun (Masdar) — Regular | ✓ COMPLETE |
| Section 6 | Verbal Noun (Masdar) — Form I Unpredictable | ✓ COMPLETE |
| Section 7 | Active Participle Patterns | ✓ COMPLETE |
| Section 8 | Passive Participle Patterns | ✓ COMPLETE |
| Section 9 | Noun of Place & Time | ✓ COMPLETE |
| Section 10 | Noun of Instrument | ✓ COMPLETE |
| Section 11 | Resembling Adjective | ✓ COMPLETE |
| Section 12 | Elative (Tafḍīl) | ✓ COMPLETE |
| Section 13 | Relative Adjective (Nisbah) | ✓ COMPLETE |
| Section 14 | Broken Plural Patterns | ✓ COMPLETE |
| Section 15 | Other Noun Patterns | ✓ COMPLETE |
| Section 16 | Weak Root Variants | ✓ COMPLETE |
| Section 17 | Noun Pattern Matching Algorithm | ✓ COMPLETE |
| Section 18 | Serialization & Storage | ✓ COMPLETE |
| Section 19 | Versioning & Evolution | ✓ COMPLETE |
| Section 20 | Quality Requirements | ✓ COMPLETE |
| Section 21 | Example Entries | ✓ COMPLETE |
| Section 22 | Cross-References | ✓ COMPLETE |

**Dependencies:** KB-0001 (Roots Database), KB-0002 (Wazan Database), KB-0003 (Verb Forms), SPEC-0001 (Chapters 1–9).

**Recommended next document:** KB-0005 (Particles) — the linguistic knowledge base for Arabic particles.
