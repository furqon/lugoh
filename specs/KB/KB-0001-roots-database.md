---
kb_id: KB-0001
title: Roots Database
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-14
updated: 2026-07-14
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0001)
  - SPEC-0001-C3: Compilation Pipeline (MOD-04 MorphologicalParser)
  - SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-04, MOD-08)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-4, IR-8)
  - SPEC-0001-C6: Deployment & Runtime Considerations (KB Bundling)
  - SPEC-0001-C8: Security, Validation & Error Handling (KB Integrity)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0401: Knowledge Graph Engine
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---


# KB-0001: Roots Database

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Root in Arabic Grammar](#2-root-in-arabic-grammar)
3. [Data Model](#3-data-model)
4. [Root Entry Schema](#4-root-entry-schema)
5. [Root Classification](#5-root-classification)
6. [Weak Root Variants](#6-weak-root-variants)
7. [Semantic & Cross-Reference Model](#7-semantic--cross-reference-model)
8. [Data Sources & Authenticity](#8-data-sources--authenticity)
9. [Serialization & Storage](#9-serialization--storage)
10. [Lookup Algorithms](#10-lookup-algorithms)
11. [Versioning & Evolution](#11-versioning--evolution)
12. [Quality Requirements](#12-quality-requirements)
13. [Example Entries](#13-example-entries)
14. [Cross-References](#14-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0001 is the **authoritative register of Arabic roots** (جذور, `judhur`) used by the AGOS platform. It provides the foundational lexical data that powers morphological analysis (MOD-04) and knowledge graph resolution (MOD-08).

Every root extracted from an Arabic stem must be matched against KB-0001. The database answers: **"Is this a valid Arabic root? What does it mean? What forms does it have?"**

### 1.2 Scope

KB-0001 covers:

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Root types** | Triliteral (ثلاثي), Quadriliteral (رباعي) | Quinary roots (خماسي — extremely rare, handled as exceptions) |
| **Root strength** | Sound (صحيح), Weak (معتل), Hamzated (مهموز), Doubled (مضاعف) | N/A |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal Arabic (covered by KB plugins) |
| **Time period** | Pre-Islamic through Modern Standard | — |
| **Verbal roots** | All verbs with attestable roots | Non-derived nouns of unknown root (handled by KB-0004) |
| **Etymology** | Arabic-only roots | Loanwords from Persian, Aramaic, Greek, etc. (covered by KB plugins) |

### 1.3 Target Audience

- **AGOS Pipeline:** MOD-04 (MorphologicalParser) reads KB-0001 during root extraction. MOD-08 (KnowledgeGraphResolver) reads KB-0001 during knowledge resolution.
- **Linguists & Data Maintainers:** Edit and extend KB-0001 with new root entries.
- **Plugin Authors:** KB-0001 serves as the data source for `kb_resolver` plugins that extend the root database (e.g., dialectal root variants, etymological annotations).

### 1.4 Relationship to Other KBs

```diff
  KB-0001: Roots                    ◄── This document
    │
    ├──► KB-0002: Wazan (patterns)     — Roots combine with wazans to form stems
    ├──► KB-0003: Verb Forms           — Each root maps to verb conjugation paradigms
    ├──► KB-0004: Noun Patterns        — Roots combine with noun patterns
    ├──► KB-0005: Particles            — Particles have no roots (checked first)
    ├──► KB-0006: Pronouns             — Pronouns have no roots (checked first)
    └──► KB-0007: Morphological Feat.  — Feature values enriched by root properties
```

KB-0001 is the **linguistic foundation** of AGOS. Every other KB either depends on root data (KB-0002, KB-0003, KB-0004) or operates independently (KB-0005, KB-0006, KB-0007).

---

## 2. Root in Arabic Grammar

### 2.1 Definition

An Arabic **root** (جذر, `jadhr` — pl. جذور, `judhur`) is the set of consonants that carry the core lexical meaning of a word. Roots typically consist of three consonants (triliteral, ثلاثي) or four consonants (quadriliteral, رباعي). The root is abstract — it does not appear as a complete word in text. Instead, it is realized through morphological patterns (wazan, وزن).

In Arabic grammatical tradition, the root is the fundamental unit of lexical organization. Dictionaries are organized by root, and morphological analysis begins by identifying the root of a word.

### 2.2 Triliteral Roots (ثلاثي)

The vast majority of Arabic roots (~85%+) are triliteral — composed of three consonants (ف-ع-ل being the prototypical example used in grammar books).

```
Root: ك-ت-ب (k-t-b)
Core meaning: writing

Derived forms:
  كَتَبَ          (kataba)        — he wrote (Form I)
  كَتَّبَ          (kattaba)       — he dictated (Form II)
  كَاتَبَ         (kātaba)       — he corresponded (Form III)
  أَكْتَبَ         (aktaba)        — he made (someone) write (Form IV)
  تَكَتَّبَ        (takattaba)     — he enrolled (Form V)
  تَكَاتَبَ       (takātaba)     — they corresponded (Form VI)
  اِنْكَتَبَ       (inkataba)      — he subscribed (Form VII)
  اِكْتَتَبَ       (iktataba)      — he copied (Form VIII)
  اِسْتَكْتَبَ      (istaktaba)     — he asked (someone) to write (Form X)
  كِتَاب          (kitāb)         — book
  كَاتِب          (kātib)         — writer
  مَكْتُوب        (maktūb)        — written
  مَكْتَب         (maktab)        — desk / office
```

### 2.3 Quadriliteral Roots (رباعي)

Quadriliteral roots — composed of four consonants — form a smaller but significant class (~10–12%). They often represent onomatopoeic or intensive concepts.

```
Root: ز-ل-ز-ل (z-l-z-l)
Core meaning: to shake, to quake

Derived forms:
  زَلْزَلَ          (zalzala)       — he shook (Form I)
  زِلْزَال          (zilzāl)        — earthquake
  مُزَلْزِل         (muzalzil)      — shaker
```

### 2.4 Why the Root Is Central

The root is the organizing principle of Arabic grammar for several reasons:

1. **Morphological productivity.** A single root can generate dozens of words through systematic pattern application. KB-0001 captures this productivity by linking each root to its verb forms and derived nouns.

2. **Semantic coherence.** Words sharing a root generally share a core semantic field. This enables semantic enrichment, anaphora resolution, and educational explanations.

3. **Dictionary organization.** Classical Arabic dictionaries (Lisān al-ʿArab, Tāj al-ʿArūs, al-Qāmūs al-Muḥīṭ) are organized by root. KB-0001's root-based structure aligns with this scholarly tradition.

4. **Pedagogical value.** Language learners and native speakers alike use root-based reasoning to understand unfamiliar words. KB-0001 enables the Explanation Engine to teach root-based etymology.

---

## 3. Data Model

### 3.1 Logical Data Model

```
Root Database (KB-0001)
├── Metadata
│   ├── kb_id: "KB-0001"
│   ├── version: "1.2.3"
│   ├── root_count: integer
│   ├── created_at: timestamp
│   ├── source_attestation: string[]       # e.g., ["lisān-al-ʿarab", "tāj-al-ʿarūs", ...]
│   └── checksum_sha256: string
│
└── Entries: RootEntry[]
```

### 3.2 Storage Model

KB-0001 is stored in two formats:

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~200 MB uncompressed | Human-readable |
| **Compiled (Binary Trie)** | Production pipeline | ~20–80 MB | Memory-mapped O(k) lookup |

The **source format** is the canonical representation. The **compiled format** is an optimized trie generated by the AGOS KB compiler (`agos kb compile`).

### 3.3 Root Count Target

| Category | Estimated Count | Notes |
|----------|----------------|-------|
| Classical Arabic roots | ~8,000–10,000 | Core vocabulary of Quran, Hadith, classical poetry |
| Modern Standard Arabic additions | ~2,000–3,000 | 19th–20th century neologisms and borrowed concepts |
| Extended (rare, archaic, variant) | ~5,000–7,000 | Additional roots from Lisān al-ʿArab, Tāj al-ʿArūs |
| **Total (Version 1.0)** | **~15,000–20,000** | Sufficient for comprehensive Quranic and MSA analysis |

These counts are for **distinct roots** — each root is stored once regardless of how many words derive from it.

---

## 4. Root Entry Schema

### 4.1 Schema Definition

```yaml
RootEntry:
  # --- Identity ---
  id: string                          # Unique ID: "KB-0001:{root_text}" e.g., "KB-0001:كتب"
  root: string                        # Root consonants in canonical order, e.g., "كتب"
  root_transliteration: string        # Latin transliteration, e.g., "k-t-b"
  root_type: RootType                 # Classification (see Section 5)
  dialect: "classical" | "msa" | "both"

  # --- Semantics ---
  core_meaning: string                # Core semantic field in English, e.g., "writing"
  core_meaning_ar: string             # Core semantic field in Arabic, e.g., "الكتابة"
  definitions: Definition[]           # Nuanced definitions

  # --- Morphology ---
  letter_count: 3 | 4                 # Triliteral or quadriliteral
  radical_letters: string[]           # The root consonants as array, e.g., ["ك", "ت", "ب"]
  radical_types: RadicalType[]        # Sound, weak, hamzated, doubled per position

  # --- Verb Forms ---
  verb_forms: VerbForm[]              # Attested verb forms (I–XV)
  verb_form_details: {                # Details for each attested form
    [form_number: integer]: {
      meaning: string,                # Meaning of this specific form
      meaning_ar: string,
      transitivity: "transitive" | "intransitive" | "ditransitive" | "both" | null,
      conjugation_class: string,      # e.g., "sound", "hollow", "defective", "doubled"
      frequency: "high" | "medium" | "low" | "rare",
      attestations: string[],         # Source attestations for this form
    },
    ...
  }

  # --- Derived Nouns ---
  derived_nouns: DerivedNoun[]        # Attested derived forms
  derived_noun_count: integer

  # --- Semantic Relations ---
  semantic_field: string              # Primary semantic field, e.g., "communication"
  semantic_tags: string[]             # e.g., ["action", "human", "transitive"]
  cognates: Cognate[]                 # Related roots with shared semantic core

  # --- Cross-References ---
  cross_references: {
    related_roots: string[],          # Related root texts
    antonyms: string[],               # Opposing root texts
    synonyms: string[],               # Near-synonym root texts
    see_also: string[],               # Other relevant root texts
    variant_readings: string[],       # Alternative root forms (Quranic readings)
  }

  # --- Attestation ---
  attestation: {
    primary_sources: string[],        # Source references, e.g., "Quran 96:4-5"
    classical_dictionaries: string[], # e.g., ["Lisān al-ʿArab (Ibn Manẓūr)"]
    confidence: "certain" | "well_attested" | "attested" | "disputed" | "uncertain",
    notes: string | null,             # Linguistic notes
  }

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string               # KB-0001 version when this entry was added
  last_reviewed_at: timestamp | null
  reviewed_by: string | null
```

### 4.2 Supporting Types

```yaml
Definition:
  definition: string                  # Definition in English
  definition_ar: string               # Definition in Arabic
  usage_examples: string[]            # Example usages with citations
  context: string | null              # Semantic context/domain

VerbForm:
  form: integer                       # I–XV
  attested: boolean                   # Whether this form is attested
  meaning: string | null              # Meaning if attested

DerivedNoun:
  pattern: string                     # Morphological pattern, e.g., "فِعَال"
  word: string                        # The derived word, e.g., "كِتَاب"
  meaning: string                     # Meaning, e.g., "book"
  noun_type: NounType                 # Classification (masdar, ism fa'il, etc.)
  frequency: "high" | "medium" | "low" | "rare"

Cognate:
  root: string                        # Related root, e.g., "كتب"
  relationship: "shared_meaning" | "antonym" | "derived_from" | "derives"
  confidence: float                   # 0.0 to 1.0

RootType:
  "sound_triliteral" | "sound_quadriliteral" |
  "weak_first" | "weak_middle" | "weak_last" | "weak_double" |
  "hamzated_first" | "hamzated_middle" | "hamzated_last" |
  "doubled" | "assimilated"

RadicalType:
  "sound" | "waw" | "ya" | "alif" | "hamza" | "doubled"

NounType:
  "masdar" | "ism_fail" | "ism_maful" | "ism_makan" | "ism_zaman" |
  "ism_alah" | "sifah_mushabbahah" | "tafdil" | "nisbah" | "jam_taksir"

CollationPriority:
  "core_quranic" | "core_hadith" | "core_classical" |
  "msa_frequent" | "msa_standard" | "rare" | "archaic"
```

### 4.3 JSON Example (Canonical Source Format)

```json
{
  "id": "KB-0001:كتب",
  "root": "كتب",
  "root_transliteration": "k-t-b",
  "root_type": "sound_triliteral",
  "dialect": "both",
  "core_meaning": "writing",
  "core_meaning_ar": "الكتابة",
  "definitions": [
    {
      "definition": "To write, to inscribe, to record",
      "definition_ar": "خَطَّ، سَجَّلَ، دَوَّنَ",
      "usage_examples": [
        "Quran 96:4-5: الَّذِي عَلَّمَ بِالْقَلَمِ / عَلَّمَ الْإِنْسَانَ مَا لَمْ يَعْلَمْ",
        "كَتَبَ مُحَمَّدٌ رِسَالَةً — Muhammad wrote a letter"
      ],
      "context": "general"
    }
  ],
  "letter_count": 3,
  "radical_letters": ["ك", "ت", "ب"],
  "radical_types": ["sound", "sound", "sound"],
  "verb_forms": [
    { "form": 1, "attested": true },
    { "form": 2, "attested": true },
    { "form": 3, "attested": true },
    { "form": 4, "attested": true },
    { "form": 5, "attested": true },
    { "form": 6, "attested": true },
    { "form": 7, "attested": true },
    { "form": 8, "attested": true },
    { "form": 9, "attested": false },
    { "form": 10, "attested": true },
    { "form": 11, "attested": false },
    { "form": 12, "attested": false },
    { "form": 13, "attested": false },
    { "form": 14, "attested": false },
    { "form": 15, "attested": false }
  ],
  "verb_form_details": {
    "1": {
      "meaning": "to write",
      "meaning_ar": "خَطَّ",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "high",
      "attestations": ["Quran 96:4-5"]
    },
    "2": {
      "meaning": "to dictate, to cause to write",
      "meaning_ar": "أَمْلَى",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "medium"
    },
    "3": {
      "meaning": "to correspond with someone",
      "meaning_ar": "رَاسَلَ",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "medium"
    },
    "4": {
      "meaning": "to dictate (to someone)",
      "meaning_ar": "أَمْلَى عَلَى",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "medium"
    },
    "5": {
      "meaning": "to register, to enroll",
      "meaning_ar": "سَجَّلَ",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "medium"
    },
    "6": {
      "meaning": "to correspond (with each other)",
      "meaning_ar": "تَرَاسَلَ",
      "transitivity": "intransitive",
      "conjugation_class": "sound",
      "frequency": "medium"
    },
    "7": {
      "meaning": "to subscribe, to be recorded",
      "meaning_ar": "اِنْكَتِبَ",
      "transitivity": "intransitive",
      "conjugation_class": "sound",
      "frequency": "low"
    },
    "8": {
      "meaning": "to copy, to transcribe",
      "meaning_ar": "نَسَخَ",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "medium"
    },
    "10": {
      "meaning": "to ask (someone) to write, to dictate",
      "meaning_ar": "اسْتَكْتَبَ",
      "transitivity": "transitive",
      "conjugation_class": "sound",
      "frequency": "low"
    }
  },
  "derived_nouns": [
    {
      "pattern": "فِعَال",
      "word": "كِتَاب",
      "meaning": "book",
      "noun_type": "ism",
      "frequency": "high"
    },
    {
      "pattern": "فَاعِل",
      "word": "كَاتِب",
      "meaning": "writer, scribe",
      "noun_type": "ism_fail",
      "frequency": "high"
    },
    {
      "pattern": "مَفْعُول",
      "word": "مَكْتُوب",
      "meaning": "written, letter",
      "noun_type": "ism_maful",
      "frequency": "high"
    },
    {
      "pattern": "مَفْعَل",
      "word": "مَكْتَب",
      "meaning": "desk, office, bureau",
      "noun_type": "ism_makan",
      "frequency": "high"
    },
    {
      "pattern": "مَكْتَبَة",
      "word": "مَكْتَبَة",
      "meaning": "library, bookstore",
      "noun_type": "ism_makan",
      "frequency": "high"
    },
    {
      "pattern": "كِتَابَة",
      "word": "كِتَابَة",
      "meaning": "writing (gerund)",
      "noun_type": "masdar",
      "frequency": "high"
    },
    {
      "pattern": "مِفْتَعَل",
      "word": "مِكْتَب",
      "meaning": "typewriter (modern)",
      "noun_type": "ism_alah",
      "frequency": "medium"
    }
  ],
  "derived_noun_count": 7,
  "semantic_field": "communication",
  "semantic_tags": ["action", "transitive", "human", "information_transfer"],
  "cognates": [
    {
      "root": "ر ق م",
      "relationship": "shared_meaning",
      "confidence": 0.6
    },
    {
      "root": "خ ط ط",
      "relationship": "shared_meaning",
      "confidence": 0.5
    }
  ],
  "cross_references": {
    "related_roots": ["ر ق م", "خ ط ط", "س ج ل"],
    "antonyms": ["م ح ي", "م س ح"],
    "synonyms": ["خ ط ط"],
    "see_also": ["ق ر أ", "ع ل م"],
    "variant_readings": []
  },
  "attestation": {
    "primary_sources": ["Quran 96:4-5"],
    "classical_dictionaries": [
      "Lisān al-ʿArab (Ibn Manẓūr)",
      "Tāj al-ʿArūs (al-Zabīdī)",
      "Muʿjam al-Wasīṭ"
    ],
    "confidence": "certain",
    "notes": null
  },
  "created_at": "2026-07-14T00:00:00Z",
  "updated_at": "2026-07-14T00:00:00Z",
  "version_added": "1.0.0",
  "last_reviewed_at": null,
  "reviewed_by": null
}
```

---

## 5. Root Classification

### 5.1 By Length

| Type | Letter Count | Percentage | Example Root | Transliteration |
|------|-------------|------------|--------------|-----------------|
| **Triliteral** (ثلاثي) | 3 | ~85–90% | ف-ع-ل | f-ʿ-l |
| **Quadriliteral** (رباعي) | 4 | ~10–12% | ز-ل-ز-ل | z-l-z-l |
| **Quinquiliteral** (خماسي) | 5 | < 1% | س-ف-ر-ج-ل | s-f-r-j-l |

Note: Quinquiliteral roots are extremely rare and mostly represent loanwords or onomatopoeic formations. They MAY be included in KB-0001 but are not a priority for Version 1.0.

### 5.2 By Root Strength (صحة)

Arabic roots are classified by the "strength" or "health" of their constituent consonants:

#### Sound Roots (صحيح)

All three (or four) consonants are "sound" — neither weak letters nor hamza. These follow regular morphological patterns without assimilation or elision.

| Subtype | Description | Example Root | Transliteration |
|---------|-------------|--------------|-----------------|
| **Sound Salim** (سالم) | No weak letters, no hamza, no doubling | ك-ت-ب | k-t-b |
| **Sound Mahmuz** (مهموز) | One or more radicals is hamza (ء) | س-ء-ل | s-ʾ-l |
| **Sound Muda'af** (مضاعف) | Second and third radicals are the same letter | م-د-د | m-d-d |

#### Weak Roots (معتل)

One or more radicals is a "weak letter" (حرف علة): alif (ا), waw (و), or ya (ي). Weak letters undergo assimilation or elision in certain morphological forms.

| Subtype | Weak Position | Arabic Term | Example Root | Transliteration | Behavior |
|---------|---------------|-------------|--------------|-----------------|----------|
| **Mithal** (مثال) | First radical | مثال واوي / يائي | و-ج-د | w-j-d | Waw assimilates in Form I imperfect |
| **Ajwaf** (أجوف) | Second radical | أجوف واوي / يائي | ق-و-ل | q-w-l | Medial weak letter alternates (ū/ā) |
| **Naqis** (ناقص) | Third radical | ناقص واوي / يائي | د-ع-و | d-ʿ-w | Final weak letter drops in certain forms |
| **Lafif** (لفيف) | Two weak radicals | لفيف مفروق / مقرون | و-ف-ي | w-f-y | Two of the three radicals are weak |

#### Hamzated Roots (مهموز)

One or more radicals is hamza (ء). Hamza is treated specially because it can be written in various forms depending on context (أ, إ, ؤ, ئ).

| Subtype | Hamza Position | Arabic Term | Example Root | Transliteration |
|---------|---------------|-------------|--------------|-----------------|
| First radical | Fā' is hamza | مهموز الفاء | أ-ك-ل | ʾ-k-l |
| Second radical | ʿAyn is hamza | مهموز العين | س-أ-ل | s-ʾ-l |
| Third radical | Lām is hamza | مهموز اللام | ق-ر-أ | q-r-ʾ |
| Mixed | Multiple positions | — | أ-ث-ر | ʾ-th-r |

#### Doubled Roots (مضاعف)

The second and third radicals are the same consonant. This causes gemination in certain forms.

| Example Root | Transliteration | Behavior |
|--------------|-----------------|----------|
| م-د-د | m-d-d | Form I: مَدَّ (madda) — radicals merge with shadda |
| ح-ب-ب | ḥ-b-b | Form I: حَبَّ (ḥabba) — radicals merge |
| ش-د-د | sh-d-d | Form I: شَدَّ (shadda) — radicals merge |

### 5.3 Classification Inheritance

Root type classification determines:

1. **Conjugation paradigm** (which verb form tables apply)
2. **Root extraction algorithm** (how radicals are identified from a stem)
3. **Feature inference** (e.g., weak roots have specific phonological behaviors)
4. **Transitivity defaults** (certain root types tend toward specific transitivity patterns)

```yaml
RootTypeHierarchy:
  - triliteral
    - sound
      - sound_salim          # ك-ت-ب
      - sound_mahmuz         # س-أ-ل (hamzated)
      - sound_mudaaf         # م-د-د (doubled)
    - weak
      - mithal_wawi          # و-ج-د (first = waw)
      - mithal_yai           # ي-ق-ظ (first = ya)
      - ajwaf_wawi           # ق-و-ل (middle = waw)
      - ajwaf_yai            # س-ي-ر (middle = ya)
      - naqis_wawi           # د-ع-و (last = waw)
      - naqis_yai            # ر-م-ي (last = ya)
      - lafif_mafruq         # و-ف-ي (first + last)
      - lafif_makrun         # ط-و-ي (middle + last)
    - hamzated
      - mahmuz_al_fa         # أ-ك-ل (first)
      - mahmuz_al_ayn        # س-أ-ل (middle)
      - mahmuz_al_lam        # ق-ر-أ (last)
  - quadriliteral
    - sound_quad             # ز-ل-ز-ل
    - weak_quad              # د-ح-ر-ج (rare)
    - hamzated_quad          # ف-أ-س-ل (rare)
```

---

## 6. Weak Root Variants

### 6.1 Why Variants Matter

Weak roots require **variant tables** in KB-0001 because their surface forms differ systematically from sound roots. The morphological parser (MOD-04) needs to know:

1. Which weak letters the root contains and in what positions.
2. How the weak letters behave in each verb form and tense.
3. What alternate spellings are attested (especially in Quranic orthography).

### 6.2 Variant Entry Schema

For weak roots, KB-0001 stores **variant records** alongside the main entry:

```yaml
WeakRootVariant:
  variant_type: "imperfect_vowel" | "assimilated_form" | "elided_form" |
                "quranic_spelling" | "classical_variant"
  description: string                # What this variant represents
  form: string                       # The variant form
  context: string | null             # When this variant applies
  attestations: string[]             # Source references
  frequency: "high" | "medium" | "low" | "rare"
```

### 6.3 Weak Root Variant Examples

#### Ajwaf (Hollow) — ق-و-ل (q-w-l, "to say")

```yaml
root: "قول"
root_type: "ajwaf_wawi"

variants:
  - variant_type: "imperfect_vowel"
    description: "Imperfect stem vowel"
    form: "يقول"                   # yaqūlu — medial waw becomes ū
    context: "imperfect indicative"
    attestations: ["Quran 2:255"]

  - variant_type: "assimilated_form"
    description: "Imperfect jussive"
    form: "يقل"                     # yaqul — medial waw drops
    context: "imperfect jussive"
    attestations: ["Quran 112:1"]

  - variant_type: "imperfect_vowel"
    description: "Medial alif in Form III"
    form: "قاول"                    # qāwala — waw surfaces as alif in Form III
    context: "Form III"
    frequency: "low"
```

#### Naqis (Defective) — ر-م-ي (r-m-y, "to throw")

```yaml
root: "رمي"
root_type: "naqis_yai"

variants:
  - variant_type: "elided_form"
    description: "Third radical ya drops in past tense 3rd person masculine singular"
    form: "رمى"                     # ramā — final ya becomes alif
    context: "past tense, 3ms"
    attestations: ["Quran 3:102"]

  - variant_type: "imperfect_vowel"
    description: "Imperfect stem vowel"
    form: "يرمي"                    # yarmī — stem retains ya
    context: "imperfect indicative"
    attestations: ["Quran 8:17"]

  - variant_type: "quranic_spelling"
    description: "Quranic orthography without final alif"
    form: "ارموا"                   # colloquial/classical variant
    context: "Quranic reading variant"
    frequency: "rare"
```

### 6.4 Variant Storage in the Compiled Trie

In the compiled trie format, weak root variants are stored as **alternative paths** from the root node:

```
               root
                │
              [قول]
             /      \
        [يقول]     [يقل]
           │          │
    imperfect    jussive
    indicative
```

This allows the morphological parser to traverse from the input stem (e.g., `يقول`) upwards to the canonical root (`قول`), matching intermediate nodes for variant classification.

---

## 7. Semantic & Cross-Reference Model

### 7.1 Semantic Fields

KB-0001 categorizes roots into a **semantic field taxonomy** — a hierarchical classification system that enables semantic enrichment and educational explanations.

#### Top-Level Fields

| Field ID | Field Name (EN) | Field Name (AR) | Example Roots |
|----------|-----------------|-----------------|---------------|
| SEM-01 | Religion & Worship | الدين والعبادة | د-ع-و, ص-ل-و, ص-و-م, ح-ج-ج |
| SEM-02 | Knowledge & Communication | العلم والتواصل | ع-ل-م, ك-ت-ب, ق-ر-أ, ف-ه-م |
| SEM-03 | Action & Movement | العمل والحركة | ف-ع-ل, ع-م-ل, ذ-ه-ب, ج-ر-ي |
| SEM-04 | Creation & Making | الخلق والصنع | خ-ل-ق, ص-ن-ع, ب-ن-ي, ج-ع-ل |
| SEM-05 | Perception & Sensation | الإدراك والحس | ر-أ-ي, س-م-ع, ب-ص-ر, ش-م-م |
| SEM-06 | Emotion & Volition | العاطفة والإرادة | ح-ب-ب, ك-ر-ه, خ-و-ف, ر-غ-ب |
| SEM-07 | Social Relations | العلاقات الاجتماعية | ن-ك-ح, ز-و-ج, و-ل-ي, ح-ك-م |
| SEM-08 | Nature & Elements | الطبيعة والعناصر | ش-ر-ب, ط-ع-م, ن-ب-ت, م-ط-ر |
| SEM-09 | Time & Space | الزمان والمكان | و-ق-ت, ز-م-ن, م-ك-ن, ب-ع-د |
| SEM-10 | Quantity & Quality | الكمية والكيفية | ك-ث-ر, ق-ل-ل, ع-ظ-م, ص-غ-ر |
| SEM-11 | Possession & Exchange | الملكية والتبادل | م-ل-ك, ش-ر-ي, ب-ي-ع, و-ه-ب |
| SEM-12 | Abstract Relations | العلاقات المجردة | ك-و-ن, ك-ي-ف, ش-ب-ه, م-ث-ل |

Each root is assigned exactly one **primary** semantic field. Additional fields MAY be assigned as **secondary** tags.

#### Sub-Fields

Sub-fields provide finer-grained classification within each top-level field:

```yaml
SEM-02 (Knowledge & Communication):
  - SEM-02-01: Teaching & Learning (ع-ل-م, د-ر-س, ط-ل-ب)
  - SEM-02-02: Writing & Recording (ك-ت-ب, س-ج-ل, خ-ط-ط)
  - SEM-02-03: Speech & Utterance (ق-و-ل, ن-ط-ق, ل-ف-ظ)
  - SEM-02-04: Reading & Recitation (ق-ر-أ, ت-ل-ا, ت-م-ت-م)
  - SEM-02-05: Understanding & Intellect (ف-ه-م, ع-ق-ل, ف-ك-ر)
```

### 7.2 Semantic Tags

In addition to the hierarchical semantic field, each root carries **semantic tags** — flat, cross-cutting labels that enable queries and enrichment:

| Tag Category | Example Tags |
|--------------|--------------|
| **Action type** | `action`, `state`, `event`, `process`, `change_of_state` |
| **Transitivity** | `transitive`, `intransitive`, `ditransitive`, `ambitransitive` |
| **Animacy** | `human`, `animal`, `abstract`, `concrete`, `natural_phenomenon` |
| **Evaluation** | `positive`, `negative`, `neutral`, `intensive`, `diminutive` |
| **Domain** | `legal`, `religious`, `scientific`, `poetic`, `colloquial` |
| **Frequency** | `high_frequency`, `medium_frequency`, `low_frequency`, `rare` |

### 7.3 Cross-Reference Types

KB-0001 supports five cross-reference types to model semantic relationships between roots:

| Type | Description | Example |
|------|-------------|---------|
| **related_roots** | Roots sharing a semantic field or conceptual domain | ك-ت-ب ↔ ق-ر-أ (writing ↔ reading) |
| **synonyms** | Roots with closely related meanings | خ-ط-ط (to draw lines) ↔ ك-ت-ب (to write) |
| **antonyms** | Roots with opposing meanings | ح-ب-ب (to love) ↔ ب-غ-ض (to hate) |
| **see_also** | Roots that provide useful comparison or context | د-ر-س (to study) in entry for ع-ل-م (to know) |
| **variant_readings** | Alternative root forms in Quranic readings | ه-د-ي (standard) ↔ ه-د-ا (Quranic variant) |

Cross-references are **bidirectional** — adding A as a synonym of B implies B should also reference A. The KB compiler (`agos kb compile`) automatically validates bidirectionality and reports inconsistencies.

### 7.4 Cognate Model

Cognates are roots that share an etymological or phonetic-semantic connection. Unlike cross-references (which are semantic), cognates represent hypothesized historical relationships:

| Relationship | Description | Example | Confidence |
|--------------|-------------|---------|------------|
| **shared_meaning** | Roots with similar sound and meaning | ن-ف-خ (to blow) ↔ ن-ف-ث (to blow/spit) | Medium |
| **derived_from** | Root believed to derive from another | ك-ت-ب (to write) likely derived from ك-ت-ف (to collect/gather) ? | Speculative |
| **metathesis** | Roots where consonants are transposed | م-س-ح (to wipe) ↔ م-ح-س ~ س-م-ح ? | Speculative |
| **loan_cognate** | Root shared with another Semitic language | ك-ت-ב (Hebrew: k-t-v) ↔ ك-ت-ب (Arabic: k-t-b) | Certain |

Cognate relationships carry a **confidence score** (0.0–1.0) because many are hypothetical. Only entries with confidence ≥ 0.7 should be used in automated semantic enrichment; lower-confidence entries are for scholarly reference.

---

## 8. Data Sources & Authenticity

### 8.1 Primary Sources

KB-0001 is compiled from authoritative Arabic lexicographical sources:

| Source | Language | Period | Coverage | Authority |
|--------|----------|--------|----------|-----------|
| **al-Qurʾān al-Karīm** | Classical Arabic | 7th C. CE | Quranic vocabulary | Highest |
| **Lisān al-ʿArab** (Ibn Manẓūr) | Classical Arabic | 13th C. CE | Extensive | Highest |
| **Tāj al-ʿArūs** (al-Zabīdī) | Classical Arabic | 18th C. CE | Most comprehensive | Highest |
| **al-Qāmūs al-Muḥīṭ** (al-Fīrūzābādī) | Classical Arabic | 14th C. CE | Comprehensive | High |
| **Muʿjam al-Wasīṭ** (Arabic Language Academy) | Modern Standard | 20th C. CE | Modern usage | High |
| **al-Muʿjam al-ʿArabī al-Asāsī** | Modern Standard | 20th C. CE | Foundational | High |

### 8.2 Attestation Confidence Levels

Each root entry carries an attestation confidence level:

| Level | Meaning | Example Root |
|-------|---------|--------------|
| **certain** | Attested in Quran, authentic Hadith, or pre-Islamic poetry | ق-و-ل (to say) — Quran 112:1 |
| **well_attested** | Multiple attestations in classical sources | ف-ل-س (to become bankrupt) — Lisān al-ʿArab + Hadith |
| **attested** | Found in at least one major classical dictionary | خ-ن-ع-ب-س (a type of plant) — Tāj al-ʿArūs only |
| **disputed** | Scholars disagree on validity or root assignment | Various proposed roots for أُفّ (\"ugh!\") |
| **uncertain** | Likely but not confidently confirmed; may be a modern coinage | Some MSA technical terms |

### 8.3 Exclusion Criteria

The following entries are EXCLUDED from KB-0001:

1. **Unattested roots.** Any root proposed by analogy but not attested in any primary or secondary source.
2. **Metathetic roots that are not independently attested.** If A-B-C and B-A-C both appear, and B-A-C is not independently attested, it is not included.
3. **Hypothetical Proto-Semitic roots.** Only Arabic-attested roots are included. Proto-Semitic reconstructions are out of scope.
4. **Roots of non-Arabic origin** (loanwords) unless fully nativized into Arabic morphological patterns.
5. **Redundant root entries.** If multiple sources disagree on root extraction for the same word, the scholarly consensus root is used (with alternative readings noted in `variant_readings`).

### 8.4 Data Integrity

Every root entry in KB-0001 MUST include:

1. **At least one attestation** from a listed primary source (or a clear note explaining why attestation is unavailable).
2. **Explicit confidence level** ('certain', 'well_attested', 'attested', 'disputed', or 'uncertain').
3. **Version tracking** — the KB-0001 version in which the entry was added or last modified.

---

## 9. Serialization & Storage

### 9.1 Source Format

The canonical source format is **JSON** (or equivalently, **YAML**), with one file per root or one file containing all roots:

```diff
  /knowledge/KB-0001/
  ├── metadata.yaml              # KB metadata (version, counts, sources)
  ├── ayn/                       # Grouped by first radical letter
  │   ├── ع-ج-ل.json            # Root: عجل
  │   ├── ع-ل-م.json            # Root: علم
  │   ├── ع-م-ل.json            # Root: عمل
  │   └── ...
  ├── kaf/                       # Grouped by first radical letter
  │   ├── ك-ت-ب.json            # Root: كتب
  │   └── ...
  ├── lam/
  │   ├── ل-ع-ب.json            # Root: لعب
  │   └── ...
  └── ...
```

#### Directory Structure Rules

1. Files are grouped into directories by the **first radical letter** (Arabic alphabet order).
2. File names use the **Arabic root text** as the stem name (e.g., `ع-ل-م.json` for root `علم`).
3. Hyphens separate root consonants for readability.
4. A root with multiple entries (rare — all roots should be unique) uses numbered suffixes (`ع-ل-م-1.json`, `ع-ل-م-2.json`).

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0001"
title: "Roots Database"
version: "1.2.3"
status: "draft" | "review" | "published"

root_count: 18420
triliteral_count: 16235
quadriliteral_count: 2185

created_at: "2026-07-14T00:00:00Z"
updated_at: "2026-07-14T00:00:00Z"

sources:
  - name: "Lisān al-ʿArab"
    version: "print_1883"
    entries_derived: 14200
  - name: "Tāj al-ʿArūs"
    version: "print_1965"
    entries_derived: 3800
  - name: "Muʿjam al-Wasīṭ"
    version: "4th_edition_2004"
    entries_derived: 420

checksum_sha256: "a1b2c3d4e5f6..."
maintainers:
  - name: "Dr. [Name]"
    email: "[email]"
    role: "chief_linguist"
```

### 9.2 Compiled Format (Binary Trie)

The production format is a **compressed trie** (prefix tree) optimized for fast root lookup:

```diff
  Binary Trie Layout:
  ┌──────────────────────────────────────────────────────┐
  │ HEADER                                                │
  │ ├── magic: "AGOSKB01" (8 bytes)                      │
  │ ├── version: major(2B) + minor(2B) + patch(2B)       │
  │ ├── root_count: u32 (4 bytes)                        │
  │ ├── node_count: u32 (4 bytes)                        │
  │ ├── trie_depth: u16 (2 bytes)                        │
  │ ├── string_table_offset: u32 (4 bytes)               │
  │ ├── index_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                     │
  ├──────────────────────────────────────────────────────┤
  │ TRIE NODES                                            │
  │ ├── Node: { children[28]?, is_endpoint, entry_ptr }  │
  │ └── ... (node_count nodes, breadth-first order)      │
  ├──────────────────────────────────────────────────────┤
  │ STRING TABLE                                          │
  │ ├── Length-prefixed UTF-8 strings                    │
  │ ├── Core meanings, definitions, semantic tags         │
  │ └── Referenced by entry_ptr from nodes                │
  ├──────────────────────────────────────────────────────┤
  │ ENTRY DATA                                            │
  │ ├── Fixed-size entry records                          │
  │ │   ├── root_type: u8                                │
  │ │   ├── verb_form_bitmask: u16                        │
  │ │   ├── semantic_field: u16                          │
  │ │   ├── core_meaning_offset: u32 (→ string table)    │
  │ │   ├── derived_nouns_offset: u32 (→ derived array)  │
  │ │   ├── cross_refs_offset: u32 (→ cross-ref array)   │
  │ │   └── attestation_offset: u32 (→ attestation data) │
  │ └── ... (root_count entries)                         │
  ├──────────────────────────────────────────────────────┤
  │  VARIABLE DATA SECTIONS                               │
  │  ├── Derived noun arrays                              │
  │  ├── Cross-reference arrays                          │
  │  ├── Verb form details                               │
  │  └── Attestation data                                │
  └──────────────────────────────────────────────────────┘
```

#### Node Structure

Each trie node represents a consonant position in a root:

```c
struct TrieNode {
    uint8_t children[28];         // Index of child node for each Arabic letter
                                  // 0 = no child (Arabic letters mapped to 1–28)
                                  // 28 Arabic letter positions (ء through ي)
    bool is_endpoint;             // True if this node terminates a valid root
    uint32_t entry_ptr;           // Offset into Entry Data section (if endpoint)
    uint8_t depth;                // Depth in trie (1–4)
    uint8_t weak_flags;           // Bitmask: is_weak, is_hamzated, is_doubled
};
```

#### Memory-Mapped Access

The compiled trie is designed for **memory-mapped access** — it can be loaded into process memory without parsing:

```
1. mmap the .agos-kb file
2. Access HEADER at offset 0
3. Access TRIE NODES at offset HEADER_SIZE
4. Look up root by traversing nodes using Arabic consonant encoding
5. If endpoint found, read entry data from entry_ptr offset
6. Cross-reference strings are direct pointers into the string table
```

This enables O(k) lookup where k = root length (3 or 4 characters), with zero parsing overhead.

### 9.3 File Packaging

```diff
  KB-0001-v1.2.3.agos-kb              # Compiled trie binary
  KB-0001-v1.2.3.agos-kb.sig          # Ed25519 signature (for integrity verification)
  KB-0001-v1.2.3.agos-kb.sha256       # SHA-256 checksum file
  KB-0001-v1.2.3.source.tar.gz        # Source JSON files (optional distribution)
```

### 9.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Trie nodes | 4 MB | 12 MB | 20K–50K nodes at ~80 bytes each |
| String table | 6 MB | 28 MB | Core meanings, definitions, tags |
| Entry data | 5 MB | 20 MB | Fixed-size records for 15K–20K entries |
| Derived nouns | 3 MB | 12 MB | Variable-length arrays |
| Cross-references | 1 MB | 4 MB | Bidirectional link arrays |
| Verb form details | 1 MB | 4 MB | Conjugation metadata |
| **Total** | **~20 MB** | **~80 MB** | Memory-mapped load |

The **Compact** format drops derived noun examples, low-confidence cognates, and verbose attestation notes. The **Full** format includes all data.

---

## 10. Lookup Algorithms

### 10.1 Exact Root Lookup

The primary lookup operation — given a candidate root, verify it exists in KB-0001 and retrieve its entry:

```pseudo
Algorithm: lookup_root
Input: candidate_root (string, 3–4 Arabic characters)
Output: RootEntry | null

1. Normalize input:
   a. Remove tatweel (kashida) if present
   b. Apply NFKC normalization (handles lam-alef ligatures)
   c. Validate: length must be 3 or 4 characters
   d. Validate: each character must be an Arabic consonant
      (ء-ي range, including hamza variants)
   e. Map each consonant to its trie index (1–28)

2. Traverse trie:
   a. current_node = trie.root
   b. For each consonant in candidate_root:
      i.   child_index = trie_indices[consonant]
      ii.  if current_node.children[child_index] == 0 → return null
      iii. current_node = trie.nodes[current_node.children[child_index]]
   c. If current_node.is_endpoint == false → return null

3. Retrieve entry:
   a. entry_offset = current_node.entry_ptr
   b. Read entry data from ENTRY DATA section at entry_offset
   c. Resolve string references from STRING TABLE
   d. Return RootEntry

4. If not found:
   a. Attempt alternative normalization (see Section 10.3)
   b. Return null if still not found
```

**Complexity:** O(k) where k = root length (3 or 4). Constant-time per character.

### 10.2 Fuzzy Root Lookup

For unknown stems, the morphological parser may propose candidate roots that need fuzzy matching:

```pseudo
Algorithm: fuzzy_lookup_root
Input: candidate_root (string), max_distance (integer, default: 1)
Output: RootEntry[] (ordered by edit distance)

1. Compute edit distance (Levenshtein) between candidate_root and all
   roots in KB-0001 of the same letter count.

2. Filter: keep only roots with edit_distance ≤ max_distance.

3. Order by:
   a. Edit distance (ascending)
   b. Attestation confidence (certain > well_attested > attested > ...)
   c. Frequency of verb forms (high frequency first)

4. Return up to 5 best matches (or fewer if max_distance filters more).

5. If max_distance == 0 (exact match only), the algorithm short-circuits
   to Section 10.1.
```

**Complexity:** O(n × k) where n = root count, k = root length. This is a fallback path; the exact trie lookup is the primary path.

### 10.3 Weak Root Normalization

Before lookup, the input root may need normalization for weak letters:

```pseudo
Algorithm: normalize_weak_root
Input: candidate_root (string), morphological_context
Output: normal_forms (string[])

1. Check for weak letters (ا, و, ي) in the candidate root:

2. Substitution Rules:
   a. Alif (ا) may be a substitute for waw (و) or ya (ي):
      - If candidate_root contains alif in position 2 (Ajwaf):
        try both ق-و-ل and ق-ي-ل variants
      - If candidate_root contains alif in position 3 (Naqis):
        try both ر-م-ي and ر-م-و variants

   b. Hamza (ء) may be written on alif, waw, or ya seat:
      - س-أ-ل: seat is alif → normalize to س-ء-ل
      - س-ؤ-ل: seat is waw → normalize to س-ء-ل
      - Try all three seat variants

   c. Geminated (shadda) consonants may indicate doubled root:
      - م-دّ → split to م-د-د (doubled root)
      - ح-بّ → split to ح-ب-ب (doubled root)

3. For each normalization, produce a candidate and run exact lookup.

4. Return all valid matches (may be multiple).
```

### 10.4 Stem-to-Root Reverse Lookup

For the known-words index used by MOD-04 (Step 4.2.1 — known word lookup):

```pseudo
Algorithm: lookup_stem_to_root
Input: stem_text (string)
Output: (RootEntry, MorphologicalContext) | null

1. Hash stem_text against the known-words index
   (pre-computed from KB-0001–0004).

2. If found:
   a. Retrieve cached (root_id, wazan_id, feature_set)
   b. Return RootEntry + resolved context

3. If not found:
   a. Attempt root extraction (MOD-04 algorithm):
      - Strip expected affixes
      - Extract root consonants
      - Run exact or fuzzy root lookup
   b. If root found:
      i.   Return RootEntry
      ii.  Optionally, cache the stem→root mapping
   c. If not found:
      i.   Return null (unknown stem)
```

---

## 11. Versioning & Evolution

### 11.1 Versioning Scheme

KB-0001 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to root entry schema, removal of root entries, format change | `1.0.0` → `2.0.0` | Requires KB conversion tool, invalidates all caches |
| **MINOR** | Addition of new root entries, addition of new optional fields, new verb forms for existing roots | `1.2.3` → `1.3.0` | Backward-compatible; caches for unchanged roots remain valid |
| **PATCH** | Corrections to existing entries (typo fixes, updated attestations, confidence changes) | `1.2.3` → `1.2.4` | Backward-compatible; no schema changes |

### 11.2 Version Compatibility

```yaml
VersionCompatibility:
  # Pipeline version requirements
  pipeline_compatibility:
    - pipeline_version: ">= 0.1.0"
      kb_min_version: "1.0.0"
      kb_max_version: "1.x.x"
      notes: "Version 1.x only; pipeline will reject 2.x until updated"

  # Cross-KB compatibility
  cross_kb_compatibility:
    KB-0002: ">= 1.0.0"
    KB-0003: ">= 1.0.0"
    KB-0004: ">= 1.0.0"
    KB-0005: ">= 1.0.0"       # Independent (no root dependency)
    KB-0006: ">= 1.0.0"       # Independent (no root dependency)
    KB-0007: ">= 1.0.0"       # root_type, transitivity features
```

### 11.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add a new root | MINOR | Add JSON file to source directory. Regenerate compiled trie. |
| Correct root meaning | PATCH | Edit JSON, update `updated_at`, regenerate trie. |
| Add verb form to existing root | MINOR | Add form to `verb_forms` array, update `verb_form_details`. |
| Remove a root | MAJOR | Only for demonstrably incorrect entries. Document reason in changelog. |
| Change schema (add field) | MINOR | Add optional field. All existing entries remain valid. |
| Change schema (remove field) | MAJOR | Remove field. All existing entries must be updated. |
| Change schema (rename field) | MAJOR | Add new field with old name as alias. Deprecate old name. |
| Update source attestation | PATCH | Update attestation data. No semantic change. |
| Merge two roots | MAJOR | Merge data, add cross-references. Deprecate one entry. |

### 11.4 Deprecation Policy

Root entries MAY be deprecated (rather than removed) when they are found to be incorrect:

1. Deprecated entries are marked with `status: deprecated` and a `deprecated_reason` field.
2. Deprecated entries remain in the database for one MINOR version cycle.
3. After the deprecation cycle, deprecated entries are removed in the next MAJOR version.
4. The morphological parser (MOD-04) SHOULD NOT match against deprecated entries.
5. The knowledge graph resolver (MOD-08) MAY include deprecated entries with a warning flag.

### 11.5 Change Log

KB-0001 MUST maintain a human-readable change log in `CHANGELOG.md`:

```markdown
# KB-0001 Changelog

## [1.3.0] - 2026-09-15
### Added
- 142 new roots from Muʿjam al-Wasīṭ (4th edition)
- Semantic tags for 5,200 existing roots
- `variant_readings` field for 89 Quranic roots

### Fixed
- Corrected root for إستبرق (sundus → س-ن-د-س, was incorrectly س-ت-ب-ر-ق)

## [1.2.0] - 2026-08-01
### Added
- 890 MSA roots (modern technical terms, administrative vocabulary)
- `nisbah` derived noun type support
- `collation_priority` field for Quranic corpus roots

### Changed
- Expanded `derived_nouns` for 2,100 core Quranic roots with Quranic examples

## [1.1.0] - 2026-07-28
### Added
- Weak root variant tables for all ajwaf, naqis, and mithal roots
- `conjugation_class` field for all verb forms

## [1.0.0] - 2026-07-14
### Initial Release
- 15,842 roots (14,230 triliteral, 1,612 quadriliteral)
- All root types (sound, weak, hamzated, doubled)
- Verb forms I–X (forms XI–XV included for attested roots)
- Derived nouns with pattern matching data
- Semantic field classification
- Cross-references between related roots
```

---

## 12. Quality Requirements

### 12.1 Completeness Targets

| Metric | Minimum | Target | Stretch |
|--------|---------|--------|---------|
| Quranic roots | 100% | 100% | 100% |
| Hadith roots (Bukhari + Muslim) | 90% | 95% | 99% |
| Classical poetry roots (7 Mu'allaqat) | 80% | 90% | 95% |
| MSA frequent roots (top 3,000) | 95% | 99% | 100% |
| MSA standard roots (top 10,000) | 80% | 90% | 95% |
| Weak root variants (attested) | 80% | 95% | 99% |
| Cross-reference bidirectionality | 95% | 99% | 100% |

### 12.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Root validity | 100% — every entry must be a genuine Arabic root | Automated attestation check + manual review |
| Root uniqueness | 100% — no duplicate root entries | CI check on source directory |
| Cross-reference consistency | ≥ 99% — A→B implies B→A | Automated bidirectional check |
| Semantic field assignment | ≥ 95% accuracy | Manual sampling review |
| Verb form attestation | 100% — attested forms must have sources | Automated check |
| Derived noun pattern match | 100% — must match KB-0002 pattern | Automated cross-KB check |
| UTF-8 validity | 100% — all Arabic text valid UTF-8 | Automated encoding check |

### 12.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate JSON/YAML structure
  ├── encoding: verify UTF-8, NFC normalization
  ├── schema: validate against KB-0001 JSON Schema
  ├── uniqueness: no duplicate root entries
  └── lint: field presence, Arabic-only root text

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── cross_refs: verify bidirectional cross-references
  ├── cross_kb: verify derived noun patterns exist in KB-0002
  ├── attestations: verify source references are from allowed list
  ├── compilation: verify trie compiles without error
  ├── size_budget: verify compiled size ≤ 80 MB
  └── regression: verify known roots still resolve correctly

  Review (manual, per release):
  ├── sample_check: linguist reviews 1% random sample
  ├── hotspot_check: review roots modified since last version
  ├── exhaustive_check: for MINOR/MAJOR releases, full review
  └── changelog: verify changelog accuracy and completeness
```

### 12.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Exact root lookup (trie) | < 1 μs | Per lookup, average |
| Exact root lookup (trie, p99) | < 5 μs | Per lookup, 99th percentile |
| Fuzzy root lookup (edit dist 1) | < 10 ms | Per lookup, average |
| Fuzzy root lookup (edit dist 1, p99) | < 50 ms | Per lookup, 99th percentile |
| Stem→root reverse lookup | < 1 μs | Hash lookup |
| KB load time (compact) | < 50 ms | mmap + verify checksum |
| KB load time (full) | < 100 ms | mmap + verify checksum |
| Memory (compact) | ~20 MB | RSS |
| Memory (full) | ~80 MB | RSS |

---

## 13. Example Entries

### 13.1 Sound Triliteral: د-ر-س (d-r-s, "to study")

```json
{
  "id": "KB-0001:درس",
  "root": "درس",
  "root_transliteration": "d-r-s",
  "root_type": "sound_triliteral",
  "core_meaning": "studying, learning",
  "core_meaning_ar": "التعلُّم والدراسة",
  "verb_forms": [1, 2, 3, 5, 6],
  "derived_nouns": [
    {"word": "دَرْس", "meaning": "lesson", "noun_type": "masdar"},
    {"word": "مَدْرَسَة", "meaning": "school", "noun_type": "ism_makan"},
    {"word": "دَارِس", "meaning": "student (active participle)", "noun_type": "ism_fail"},
    {"word": "مَدْرُوس", "meaning": "studied (passive participle)", "noun_type": "ism_maful"},
    {"word": "دِرَاسَة", "meaning": "study (gerund)", "noun_type": "masdar"},
    {"word": "دِرَاسِيّ", "meaning": "academic", "noun_type": "nisbah"}
  ],
  "semantic_field": "SEM-02",
  "semantic_tags": ["action", "human", "transitive", "knowledge"],
  "cross_references": {
    "related_roots": ["ع-ل-م", "ق-ر-أ", "ف-ه-م"],
    "synonyms": ["ع-ل-م"],
    "see_also": ["ك-ت-ب"]
  },
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Quran 3:79", "Quran 68:1"]
  }
}
```

### 13.2 Weak Middle (Ajwaf): ق-و-ل (q-w-l, "to say")

```json
{
  "id": "KB-0001:قول",
  "root": "قول",
  "root_transliteration": "q-w-l",
  "root_type": "ajwaf_wawi",
  "core_meaning": "saying, speaking",
  "core_meaning_ar": "القول والكلام",
  "verb_forms": [1, 2, 3, 4, 5, 6, 8],
  "derived_nouns": [
    {"word": "قَوْل", "meaning": "saying, statement", "noun_type": "masdar"},
    {"word": "قَائِل", "meaning": "speaker", "noun_type": "ism_fail"},
    {"word": "مَقُول", "meaning": "that which is said", "noun_type": "ism_maful"},
    {"word": "مَقَال", "meaning": "article, discourse", "noun_type": "ism_makan"},
    {"word": "مَقَالَة", "meaning": "article, essay", "noun_type": "ism_makan"}
  ],
  "variants": [
    {
      "variant_type": "imperfect_vowel",
      "description": "Form I imperfect medial waw → ū",
      "form": "يقول",
      "context": "imperfect indicative active"
    },
    {
      "variant_type": "elided_form",
      "description": "Form I imperfect jussive drops medial waw",
      "form": "يقل",
      "context": "imperfect jussive active"
    }
  ],
  "semantic_field": "SEM-02-03",
  "semantic_tags": ["action", "human", "transitive", "communication", "high_frequency"],
  "cross_references": {
    "related_roots": ["ن-ط-ق", "ك-ل-م", "ل-ف-ظ"],
    "synonyms": ["ن-ط-ق"],
    "antonyms": ["ص-م-ت"]
  },
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Quran 112:1", "Quran 2:255", "Throughout Quran (>500 occurrences)"]
  }
}
```

### 13.3 Hamzated: س-أ-ل (s-ʾ-l, "to ask")

```json
{
  "id": "KB-0001:سأل",
  "root": "سأل",
  "root_transliteration": "s-ʾ-l",
  "root_type": "mahmuz_al_ayn",
  "core_meaning": "asking, questioning",
  "core_meaning_ar": "السؤال والاستفسار",
  "verb_forms": [1, 3, 4, 5, 6, 8, 10],
  "derived_nouns": [
    {"word": "سُؤَال", "meaning": "question", "noun_type": "masdar"},
    {"word": "سَائِل", "meaning": "questioner", "noun_type": "ism_fail"},
    {"word": "مَسْئُول", "meaning": "responsible, answerable", "noun_type": "ism_maful"},
    {"word": "مَسْأَلَة", "meaning": "issue, problem", "noun_type": "ism_makan"}
  ],
  "semantic_field": "SEM-02-03",
  "semantic_tags": ["action", "human", "transitive", "communication"],
  "cross_references": {
    "related_roots": ["س-و-ل", "س-ل-س-ل"],
    "see_also": ["ج-و-ب", "ط-ل-ب"]
  },
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Quran 5:101", "Quran 21:65"]
  }
}
```

### 13.4 Quadriliteral: ز-ل-ز-ل (z-l-z-l, "to shake")

```json
{
  "id": "KB-0001:زلزل",
  "root": "زلزل",
  "root_transliteration": "z-l-z-l",
  "root_type": "sound_quadriliteral",
  "core_meaning": "shaking, quaking",
  "core_meaning_ar": "الاهتزاز والاضطراب",
  "letter_count": 4,
  "radical_letters": ["ز", "ل", "ز", "ل"],
  "verb_forms": [
    { "form": 1, "attested": true }
  ],
  "verb_form_details": {
    "1": {
      "meaning": "to shake violently",
      "transitivity": "intransitive",
      "conjugation_class": "sound_quad",
      "frequency": "medium",
      "attestations": ["Quran 99:1"]
    }
  },
  "derived_nouns": [
    {"word": "زِلْزَال", "meaning": "earthquake", "noun_type": "masdar"},
    {"word": "مُزَلْزِل", "meaning": "shaker", "noun_type": "ism_fail"},
    {"word": "مُزَلْزَل", "meaning": "shaken", "noun_type": "ism_maful"}
  ],
  "semantic_field": "SEM-08",
  "semantic_tags": ["event", "intransitive", "nature", "intensive"],
  "cross_references": {
    "related_roots": ["ر-ج-ف", "ه-ز-ز"],
    "synonyms": ["ر-ج-ف", "ه-ز-ز"]
  },
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Quran 99:1 (Sūrat al-Zalzalah)"],
    "classical_dictionaries": ["Lisān al-ʿArab"]
  }
}
```

### 13.5 Doubled: م-د-د (m-d-d, "to extend")

```json
{
  "id": "KB-0001:مدد",
  "root": "مدد",
  "root_transliteration": "m-d-d",
  "root_type": "doubled",
  "core_meaning": "extending, stretching, supplying",
  "core_meaning_ar": "المد والتوسعة والإمداد",
  "verb_forms": [1, 4, 5, 7, 8],
  "derived_nouns": [
    {"word": "مَدّ", "meaning": "extension, supply", "noun_type": "masdar"},
    {"word": "مَادّ", "meaning": "extending (active participle)", "noun_type": "ism_fail"},
    {"word": "مَمْدُود", "meaning": "extended (passive participle)", "noun_type": "ism_maful"},
    {"word": "مِدَاد", "meaning": "ink", "noun_type": "ism_alah"},
    {"word": "مَادَّة", "meaning": "material, substance", "noun_type": "ism"}
  ],
  "semantic_field": "SEM-03",
  "semantic_tags": ["action", "transitive", "physical"],
  "cross_references": {
    "related_roots": ["ط-و-ل", "و-س-ع", "ز-و-د"],
    "see_also": ["د-و-م", "ك-ث-ر"]
  },
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Quran 19:75", "Quran 2:57"]
  }
}
```

---

## 14. Cross-References

### 14.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0001 in module catalog; Layers that consume KB-0001 |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Root extraction algorithm that uses KB-0001 |
| SPEC-0001-C3 | Compilation Pipeline (MOD-08) | Knowledge graph resolution that reads KB-0001 |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Root schema in MOD-08 interface |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-4) | Root field in morphological analysis |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-8) | Resolved root entry in ResolvedGIR |
| SPEC-0001-C6 | Deployment & Runtime Considerations | KB bundling, distribution, signing |
| SPEC-0001-C8 | Security, Validation & Error Handling | KB integrity verification (.sig, .sha256) |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0001 size budget, trie lookup performance |
| KB-0002 | Wazan Database | Morphological patterns that operate on roots |
| KB-0003 | Verb Forms | Conjugation paradigms indexed by root |
| KB-0004 | Noun Patterns | Noun patterns that combine with roots |
| KB-0007 | Morphological Features | Feature taxonomy enriched by root properties |

### 14.2 External References

| Reference | Relevance |
|-----------|-----------|
| Lisān al-ʿArab (Ibn Manẓūr, 1290 CE) | Primary lexical source for root attestation |
| Tāj al-ʿArūs (al-Zabīdī, 1774 CE) | Most comprehensive classical Arabic dictionary |
| al-Qāmūs al-Muḥīṭ (al-Fīrūzābādī, 1414 CE) | Foundational classical dictionary |
| Muʿjam al-Wasīṭ (Arabic Language Academy, 1960) | MSA lexical standard |
| Buckwalter Arabic Morphological Analyzer (BAMA) | Reference for computational root extraction |
| Quranic Arabic Corpus (Leeds University) | Arabic root annotation for Quranic vocabulary |
| Unicode Standard (Arabic block) | Character encoding for root text representation |
| SemVer 2.0.0 | KB versioning scheme |

---

## Progress Summary

**KB-0001: Roots Database**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Root in Arabic Grammar | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Root Entry Schema | ✓ COMPLETE |
| Section 5 | Root Classification | ✓ COMPLETE |
| Section 6 | Weak Root Variants | ✓ COMPLETE |
| Section 7 | Semantic & Cross-Reference Model | ✓ COMPLETE |
| Section 8 | Data Sources & Authenticity | ✓ COMPLETE |
| Section 9 | Serialization & Storage | ✓ COMPLETE |
| Section 10 | Lookup Algorithms | ✓ COMPLETE |
| Section 11 | Versioning & Evolution | ✓ COMPLETE |
| Section 12 | Quality Requirements | ✓ COMPLETE |
| Section 13 | Example Entries | ✓ COMPLETE |
| Section 14 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–5, 8, 9), KB-0002–0007.

**Recommended next document:** KB-0002 (Wazan Database) — the morphological patterns that combine with roots to form stems.
