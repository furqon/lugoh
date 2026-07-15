---
kb_id: KB-0003
title: Verb Forms — Conjugation Paradigms
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0003)
  - SPEC-0001-C3: Compilation Pipeline (MOD-04 MorphologicalParser)
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-4, IR-8)
  - SPEC-0001-C6: Deployment & Runtime Considerations (KB Bundling)
  - SPEC-0001-C8: Security, Validation & Error Handling (KB Integrity)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - SPEC-0101: Morphology Engine (planned)
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---

# KB-0003: Verb Forms — Conjugation Paradigms

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Conjugation in Arabic Grammar](#2-conjugation-in-arabic-grammar)
3. [Data Model](#3-data-model)
4. [Paradigm Entry Schema](#4-paradigm-entry-schema)
5. [Sound Triliteral Conjugation (صحيح سالم)](#5-sound-triliteral-conjugation-صحيح-سالم)
6. [Weak Root Conjugations (المعتل)](#6-weak-root-conjugations-المعتل)
7. [Doubled Root Conjugation (المضاعف)](#7-doubled-root-conjugation-المضاعف)
8. [Hamzated Root Conjugation (المهموز)](#8-hamzated-root-conjugation-المهموز)
9. [Quadriliteral Verb Conjugation](#9-quadriliteral-verb-conjugation)
10. [Passive Voice Conjugation](#10-passive-voice-conjugation)
11. [Imperative Mood](#11-imperative-mood)
12. [Verb Form Paradigm Summary (I–XV)](#12-verb-form-paradigm-summary-i-xv)
13. [Paradigm Lookup Algorithms](#13-paradigm-lookup-algorithms)
14. [Serialization & Storage](#14-serialization--storage)
15. [Versioning & Evolution](#15-versioning--evolution)
16. [Quality Requirements](#16-quality-requirements)
17. [Example Entries](#17-example-entries)
18. [Cross-References](#18-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0003 is the **authoritative register of Arabic verb conjugation paradigms** (تصريف الأفعال, `taṣrīf al-afʿāl`) used by the AGOS platform. It provides the full inflectional tables that power verb form generation (MOD-09 BytecodeGenerator), verb form analysis (MOD-04 MorphologicalParser), and educational explanations (MOD-11 ExplanationEngine).

While KB-0002 (Wazan Database) defines the **patterns** that combine with roots to produce stems, KB-0003 defines the **full inflectional paradigms** — the complete set of inflected forms for each combination of verb form and conjugation class.

KB-0003 answers: **"Given a root, a verb form, and a conjugation class, what are all the inflected forms (person, number, gender, tense, mood)?"**

### 1.2 Scope

KB-0003 covers:

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Tenses** | Perfect (past, الماضي), Imperfect (present/future, المضارع) | Compound tenses (covered by MOD-03 Preprocessor) |
| **Moods (Imperfect)** | Indicative (الرفع), Subjunctive (النصب), Jussive (الجزم), Energetic I/II (التوكيد) | — |
| **Imperative** | Positive imperative (الأمر), Negative imperative (النهي) | — |
| **Voices** | Active (مبني للمعلوم), Passive (مبني للمجهول) | — |
| **Person/Gender/Number** | 1s, 1p, 2ms, 2fs, 2p(f/m), 3ms, 3fs, 3p(f/m), 3d(f/m) (see Section 2.3) | Dialectal gender/number distinctions |
| **Verb Forms** | Forms I–XV (triliteral), Forms QI–QIII (quadriliteral) | Archaic forms beyond standard 18 |
| **Conjugation Classes** | Sound, Hollow, Defective, Assimilated, Doubled, Hamzated, Sound Quadriliteral | Dialectal conjugation variants |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal Arabic |

### 1.3 Target Audience

- **AGOS Pipeline:** MOD-04 (MorphologicalParser) reads KB-0003 to verify candidate verb forms against known paradigms. MOD-09 (BytecodeGenerator) reads KB-0003 to generate inflected forms as runtime bytecode. MOD-11 (ExplanationEngine) uses KB-0003 for pedagogical conjugation tables.
- **Linguists & Data Maintainers:** Edit and extend KB-0003 with additional paradigm patterns or weak root variants.
- **Grammar Tool Authors:** KB-0003 serves as input for conjugation lookup tools and educational apps.

### 1.4 Relationship to Other KBs

```diff
  KB-0001: Roots (جذور)                 — The consonants that carry lexical meaning
    │
    ├──► KB-0002: Wazan (أوزان)         — Patterns that combine with roots to form stems
    │         │
    │         └──► KB-0003: Verb Forms  ◄── This document (Full conjugation paradigms)
    │                 │
    │                 ├──► Sound Triliteral Conjugation (13 paradigm tables)
    │                 ├──► Hollow Conjugation (Ajwaf — ~9 tables)
    │                 ├──► Defective Conjugation (Naqis — ~9 tables)
    │                 ├──► Assimilated Conjugation (Mithal — ~5 tables)
    │                 ├──► Doubled Conjugation (Muda'af — ~9 tables)
    │                 ├──► Hamzated Conjugation (Mahmuz — ~5 tables)
    │                 └──► Quadriliteral Conjugation (~5 tables)
    │
    ├──► KB-0004: Noun Patterns (أوزان) — Detailed noun pattern specifications
    ├──► KB-0005: Particles             — Particles have no roots or wazans
    ├──► KB-0006: Pronouns              — Pronouns have no roots or wazans
    └──► KB-0007: Morphological Feat.   — Feature values enriched by conjugation
```

### 1.5 How KB-0003 Complements KB-0002

The boundary between KB-0002 and KB-0003 is:

| Aspect | KB-0002 (Wazan) | KB-0003 (Verb Forms) |
|--------|-----------------|---------------------|
| **What it defines** | Stem-level patterns (phonological templates) | Full inflectional tables |
| **Granularity** | One pattern per verb form × conjugation class | 13 inflected forms per verb form × conjugation class |
| **Example** | Form I active perfect: فَعَلَ | All 13 person/number/gender forms for each tense/mood |
| **Output for MOD-09** | How to generate a stem from root + form | How to generate all surface forms from root + form + features |
| **Weak root handling** | Phonological variants of stems | Conjugation-specific inflection rules per weak class |

---

## 2. Conjugation in Arabic Grammar

### 2.1 The Conjugation System

Arabic verbs are inflected for **person**, **gender**, **number**, **tense**, **mood**, and **voice**. The inflectional system is suffix-dominated in the perfect and prefix + suffix in the imperfect.

The two primary tenses are:

1. **Perfect** (الماضي, `al-māḍī`): Completed actions, past tense. Marked by **suffixes** on the stem.
2. **Imperfect** (المضارع, `al-muḍāriʿ`): Incomplete actions, present/future tense. Marked by **prefixes** (for person) and **suffixes** (for number/gender/mood).

The imperfect has four moods:

| Mood | Arabic Term | Marker | Usage |
|------|-------------|--------|-------|
| **Indicative** | الرَّفْع (`ar-rafʿ`) | Suffix -u | Default mood; statements of fact |
| **Subjunctive** | النَّصْب (`an-naṣb`) | Suffix -a | After subordinating conjunctions (أَنْ, لَنْ) |
| **Jussive** | الْجَزْم (`al-jazm`) | Suffix -∅ (sukun) | After conditional particles (لَمْ, لَمَّا) |
| **Energetic I** | التَّوْكِيد الثقيل | Suffix -anna(n) | Emphatic assertion |
| **Energetic II** | التَّوْكِيد الخفيف | Suffix -an | Emphatic assertion (lighter) |

### 2.2 Stem Types

Arabic verb forms have two principal stems:

1. **Perfect stem** (أساس الماضي): Used as the base for perfect tense suffixes.
2. **Imperfect stem** (أساس المضارع): Used as the base for imperfect prefixes and suffixes.

For sound triliteral Form I, the imperfect stem vowel is unpredictable and must be stored per-root in KB-0001. For derived forms (II–XV), the imperfect stem is predictable from the form.

### 2.3 Person/Number/Gender Paradigm

The standard Arabic verb paradigm has 13 slots (5 singular, 3 dual, 5 plural for Classical Arabic):

| # | Label | Person | Number | Gender | Perfect Suffix | Imperfect Prefix | Imperfect Suffix |
|---|-------|--------|--------|--------|----------------|------------------|------------------|
| 1 | **3ms** | 3rd | Singular | Masculine | -a | ya- | -u (indicative) |
| 2 | **3fs** | 3rd | Singular | Feminine | -at | ta- | -u |
| 3 | **2ms** | 2nd | Singular | Masculine | -ta | ta- | -u |
| 4 | **2fs** | 2nd | Singular | Feminine | -ti | ta- | -īna (indicative) |
| 5 | **1s** | 1st | Singular | Common | -tu | a- | -u |
| 6 | **3md** | 3rd | Dual | Masculine | -ā | ya- | -āni (indicative) |
| 7 | **3fd** | 3rd | Dual | Feminine | -atā | ta- | -āni |
| 8 | **2d** | 2nd | Dual | Common | -tumā | ta- | -āni |
| 9 | **3mp** | 3rd | Plural | Masculine | -ū | ya- | -ūna (indicative) |
| 10 | **3fp** | 3rd | Plural | Feminine | -na | ya- | -na |
| 11 | **2mp** | 2nd | Plural | Masculine | -tum | ta- | -ūna |
| 12 | **2fp** | 2nd | Plural | Feminine | -tunna | ta- | -na |
| 13 | **1p** | 1st | Plural | Common | -nā | na- | -u |

Note: Dual forms (3md, 3fd, 2d) are part of Classical Arabic. In MSA, dual is used for number/agreement but verb paradigms are often simplified to singular + plural. KB-0003 includes dual forms as optional (configurable at KB compile time).

---

## 3. Data Model

### 3.1 Logical Data Model

```yaml
Verb Forms Database (KB-0003)
├── Metadata
│   ├── kb_id: "KB-0003"
│   ├── version: "1.0.0"
│   ├── paradigm_count: integer
│   ├── conjugation_classes: string[]
│   ├── verb_forms: integer[]          # [I, II, III, ..., XV, QI, QII, QIII]
│   ├── created_at: timestamp
│   ├── sources: string[]
│   └── checksum_sha256: string
│
├── ConjugationClasses: ConjugationClass[]
│   ├── sound_triliteral
│   ├── hollow_wawi (ajwaf wawi)
│   ├── hollow_yai (ajwaf yai)
│   ├── defective_wawi (naqis wawi)
│   ├── defective_yai (naqis yai)
│   ├── assimilated_wawi (mithal wawi)
│   ├── assimilated_yai (mithal yai)
│   ├── doubled (muda'af)
│   ├── hamzated_first (mahmuz al-fa)
│   ├── hamzated_middle (mahmuz al-ayn)
│   ├── hamzated_last (mahmuz al-lam)
│   ├── sound_quadriliteral
│   ├── weak_quadriliteral
│   └── lafif (double-weak)
│
└── ParadigmTables: ParadigmTable[]
    ├── One table per (verb_form, conjugation_class, voice) combination
    └── ~180–250 total tables
```

### 3.2 Storage Model

KB-0003 is stored in two formats:

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~50–80 MB uncompressed | Human-readable paradigm tables |
| **Compiled (Table Binary)** | Production pipeline | ~15–30 MB | Memory-mapped O(1) table lookup |

The **source format** stores each paradigm as a named table with explicit affix rules. The **compiled format** pre-computes all 13 × ~18 × ~6 = ~1,400 inflected slots and indexes them by (verb_form, conjugation_class, voice, tense, mood, person, number, gender).

### 3.3 Paradigm Count Target

| Category | Conjugation Classes | Forms per Class | Total Tables | Infl. Slots |
|----------|-------------------|------------------|--------------|-------------|
| Sound triliteral | 1 | 15 (I–XV) × 2 voices | 30 | 390 |
| Hollow (Ajwaf) | 2 (wawi/yai) | 15 × 2 voices | 60 | 780 |
| Defective (Naqis) | 2 (wawi/yai) | 15 × 2 voices | 60 | 780 |
| Assimilated (Mithal) | 2 (wawi/yai) | 15 × 2 voices | 60 | 780 |
| Doubled | 1 | 15 × 2 voices | 30 | 390 |
| Hamzated | 3 (first/mid/last) | 15 × 2 voices | 90 | 1,170 |
| Lafif (double-weak) | 2 (mafruq/makrun) | 15 × 2 voices | 60 | 780 |
| Quadriliteral (sound) | 1 | 3 (QI–QIII) × 2 voices | 6 | 78 |
| Quadriliteral (weak) | 1 | 3 × 2 voices | 6 | 78 |
| **Total** | **15** | — | **~252–402** | **~3,276–5,226** |

Note: Not all conjugation classes × verb form combinations are attested. For example, Forms IX and XI–XV are rarely used with weak roots. The actual paradigm count in Version 1.0 will be ~180–250 tables covering attested combinations. Unattested combinations return an empty result.

---

## 4. Paradigm Entry Schema

### 4.1 Schema Definition

```yaml
ParadigmTable:
  # --- Identity ---
  id: string                           # "KB-0003:{verb_form}:{conjugation_class}:{voice}:{tense}"
                                       # e.g., "KB-0003:I:sound:active:perfect"
  verb_form: integer | string          # I–XV or QI–QIII
  conjugation_class: string            # "sound", "hollow_wawi", "hollow_yai",
                                       # "defective_wawi", "defective_yai",
                                       # "assimilated_wawi", "assimilated_yai",
                                       # "doubled", "hamzated_first",
                                       # "hamzated_middle", "hamzated_last",
                                       # "lafif_mafruq", "lafif_makrun",
                                       # "sound_quadriliteral", "weak_quadriliteral"
  voice: "active" | "passive"

  # --- Stem Definition ---
  perfect_stem: StemDefinition         # The base perfect stem (3ms form)
  imperfect_stem: StemDefinition       # The base imperfect stem

  # --- Paradigm Rules ---
  perfect_conjugation: ParadigmRule[]  # Rules for perfect tense affixation
  imperfect_conjugation: ParadigmRule[] # Rules for imperfect tense affixation
  imperative: ImperativeRule | null    # Imperative formation (active only)

  # --- Mood Variation ---
  indicative: MoodRule                 # Indicative affix adjustments
  subjunctive: MoodRule                # Subjunctive affix adjustments
  jussive: MoodRule                    # Jussive affix adjustments
  energetic_i: MoodRule | null         # Energetic I (heavy) adjustments
  energetic_ii: MoodRule | null        # Energetic II (light) adjustments

  # --- Phonological Notes ---
  phonological_rules: PhonologicalRule[] # Any stem changes during conjugation

  # --- Attestation ---
  attestation: Attestation

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string
```

### 4.2 Supporting Types

```yaml
StemDefinition:
  stem_form: string                    # The stem itself (e.g., "فَعَلَ" for perfect)
  stem_template: string                # Template with placeholders (e.g., "C₁aC₂aC₃")
  stem_note: string | null             # Notes about stem formation

ParadigmRule:
  slot: string                         # Paradigm slot label (see Section 2.3)
                                       # "3ms", "3fs", "2ms", "2fs", "1s",
                                       # "3md", "3fd", "2d",
                                       # "3mp", "3fp", "2mp", "2fp", "1p"
  prefix: string | null                # Prefix to add before the stem
  suffix: string | null                # Suffix to add after the stem
  stem_vowel: string | null            # Any stem vowel change (for weak roots)
  stem_change: string | null           # Any other stem modification
  notes: string | null                 # Special notes about this slot

MoodRule:
  suffix_adjustment: string            # How mood changes the imperfect suffix
  vowel_changes: string[] | null       # Any vowel changes from mood
  notes: string | null

ImperativeRule:
  derived_from: string                 # Which imperfect form the imperative derives from
  prefix_removal: string               # The imperfect prefix to replace/remove
  prefix_insert: string | null         # New prefix for imperative (alif wasl for 2ms/2fs)
  pattern_template: string             # Template showing imperative formation
  slots: ParadigmRule[]                # Per-slot imperative rules (2ms, 2fs, 2mp, 2fp, 2d)

PhonologicalRule:
  condition: string                    # When this rule is triggered
  operation: string                    # What happens to the stem/affixes
  affected_slots: string[]             # Which slots are affected
  rule_type: "assimilation" | "elision" | "substitution" | "gemination" | "stress"

Attestation:
  confidence: "certain" | "well_attested" | "attested" | "disputed"
  primary_sources: string[]
  classical_references: string[]
  notes: string | null

PatternExample:
  slot: string                         # Paradigm slot
  word: string                         # The inflected word form
  transliteration: string
  meaning: string                      # Translation
  context: string | null
```

### 4.3 JSON Example (Form I Sound Active Perfect)

```json
{
  "id": "KB-0003:I:sound:active:perfect",
  "verb_form": "I",
  "conjugation_class": "sound",
  "voice": "active",
  "tense": "perfect",
  "perfect_stem": {
    "stem_form": "فَعَلَ",
    "stem_template": "C₁aC₂aC₃",
    "stem_note": "The 3ms perfect form itself (kataba, jalasa, etc.)"
  },
  "perfect_conjugation": [
    { "slot": "3ms", "suffix": "" },
    { "slot": "3fs", "suffix": "َت" },
    { "slot": "2ms", "suffix": "تَ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "2fs", "suffix": "تِ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "1s", "suffix": "تُ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "3md", "suffix": "َا" },
    { "slot": "3fd", "suffix": "َتَا" },
    { "slot": "2d", "suffix": "تُمَا", "stem_vowel": "sukun_on_C₃" },
    { "slot": "3mp", "suffix": "وا", "stem_vowel": "dhamma_on_C₃" },
    { "slot": "3fp", "suffix": "نَ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "2mp", "suffix": "تُمْ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "2fp", "suffix": "تُنَّ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "1p", "suffix": "نَا", "stem_vowel": "sukun_on_C₃" }
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab, Vol. IV"],
    "classical_references": ["Al-Kitab", "Sharh al-Ashmuni"]
  },
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

---

## 5. Sound Triliteral Conjugation (صحيح سالم)

Sound triliteral roots (صحيح سالم) have no weak letters, hamza, or doubled radicals. They follow **regular** conjugation patterns without assimilation, elision, or substitution. All 15 verb forms are fully productive for sound roots.

### 5.1 Form I — Perfect Active (Sound)

```
Stem: فَعَلَ (C₁aC₂aC₃)
Imperfect stem vowel: unpredictable (stored in KB-0001 per root)

Perfect Active — Sound Triliteral Form I
┌──────────┬────────────────┬───────────────┐
│   Slot   │  Suffix        │  Example (ك-ت-ب) │
├──────────┼────────────────┼───────────────┤
│ 3ms      │  —             │  كَتَبَ        │
│ 3fs      │  َت            │  كَتَبَت       │
│ 2ms      │  َت            │  كَتَبْتَ      │
│ 2fs      │  ِت            │  كَتَبْتِ      │
│ 1s       │  ُت            │  كَتَبْتُ      │
│ 3md      │  ا             │  كَتَبَا       │
│ 3fd      │ َتَا            │  كَتَبَتَا     │
│ 2d       │  تُّمَا         │  كَتَبْتُمَا   │
│ 3mp      │  وا             │  كَتَبُوا     │
│ 3fp      │  نَ             │  كَتَبْنَ      │
│ 2mp      │  تُمْ           │  كَتَبْتُمْ    │
│ 2fp      │  تُنَّ          │  كَتَبْتُنَّ   │
│ 1p       │  نَا            │  كَتَبْنَا     │
└──────────┴────────────────┴───────────────┘

Note: The 2fs, 1s, and 2mp/md forms place a sukun on C₃ before the suffix.
```

### 5.2 Form I — Perfect Passive (Sound)

```
Stem: فُعِلَ (C₁uC₂iC₃)  — vowel pattern changes from active

Perfect Passive — Sound Triliteral Form I
┌──────────┬────────────────┬────────────────┐
│   Slot   │  Suffix        │  Example (ك-ت-ب)  │
├──────────┼────────────────┼────────────────┤
│ 3ms      │  —             │  كُتِبَ         │
│ 3fs      │  َت            │  كُتِبَت        │
│ 2ms      │  َت            │  كُتِبْتَ       │
│ 2fs      │  ِت            │  كُتِبْتِ       │
│ 1s       │  ُت            │  كُتِبْتُ       │
│ 3md      │  ا             │  كُتِبَا        │
│ 3fd      │ َتَا            │  كُتِبَتَا      │
│ 2d       │  تُّمَا         │  كُتِبْتُمَا    │
│ 3mp      │  وا             │  كُتِبُوا      │
│ 3fp      │  نَ             │  كُتِبْنَ       │
│ 2mp      │  تُمْ           │  كُتِبْتُمْ     │
│ 2fp      │  تُنَّ          │  كُتِبْتُنَّ    │
│ 1p       │  نَا            │  كُتِبْنَا      │
└──────────┴────────────────┴────────────────┘

Passive marker: C₁ vowel → u, C₂ vowel → i throughout.
```

### 5.3 Form I — Imperfect Indicative Active (Sound)

```
Stem: (imperfect stem — vowel varies)
       Pattern: yaC₁C₂(u/i/a)C₃u for 3ms
       The imperfect stem vowel (C₂ vowel) is root-specific.

Imperfect Indicative Active — Sound Triliteral Form I
┌──────────┬──────────┬────────────┬───────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (ك-ت-ب) │
├──────────┼──────────┼────────────┼───────────────┤
│ 3ms      │  يَ      │  ُ         │  يَكْتُبُ      │
│ 3fs      │  تَ      │  ُ         │  تَكْتُبُ      │
│ 2ms      │  تَ      │  ُ         │  تَكْتُبُ      │
│ 2fs      │  تَ      │ ِينَ       │  تَكْتُبِينَ   │
│ 1s       │  أَ      │  ُ         │  أَكْتُبُ      │
│ 3md      │  يَ      │ َانِ        │  يَكْتُبَانِ   │
│ 3fd      │  تَ      │ َانِ        │  تَكْتُبَانِ   │
│ 2d       │  تَ      │ َانِ        │  تَكْتُبَانِ   │
│ 3mp      │  يَ      │ ُونَ        │  يَكْتُبُونَ   │
│ 3fp      │  يَ      │  نَ         │  يَكْتُبْنَ     │
│ 2mp      │  تَ      │ ُونَ        │  تَكْتُبُونَ   │
│ 2fp      │  تَ      │  نَ         │  تَكْتُبْنَ     │
│ 1p       │  نَ      │  ُ          │  نَكْتُبُ      │
└──────────┴──────────┴────────────┴───────────────┘

Mood variation for imperfect:
  Indicative:  suffix -u (or -na/-ni for 2fs/3mp/2mp)
  Subjunctive: suffix -a (or -∅ from -na → -∅ for 2fs/3mp/2mp)
  Jussive:     suffix -∅ (or drop -na/-ni for 2fs/3mp/2mp)
```

### 5.4 Imperfect Mood Variation (Form I Sound)

```
Mood Variation — 3ms
┌──────────────┬──────────┬────────┬───────────────┐
│    Mood      │  Prefix  │ Suffix │  Example        │
├──────────────┼──────────┼────────┼───────────────┤
│ Indicative   │  يَ      │  ُ     │  يَكْتُبُ       │
│ Subjunctive  │  يَ      │  َ     │  يَكْتُبَ       │
│ Jussive      │  يَ      │  —     │  يَكْتُبْ (sukun)│
│ Energetic I  │  يَ      │  َنَّ  │  يَكْتُبَنَّ    │
│ Energetic II │  يَ      │  َْن  │  يَكْتُبَنْ     │
└──────────────┴──────────┴────────┴───────────────┘

Mood Variation — 3mp (strong affix -ūna)
┌──────────────┬──────────┬──────────┬────────────────┐
│    Mood      │  Prefix  │  Suffix  │  Example         │
├──────────────┼──────────┼──────────┼────────────────┤
│ Indicative   │  يَ      │ ُونَ     │  يَكْتُبُونَ     │
│ Subjunctive  │  يَ      │  وا      │  يَكْتُبُوا      │
│ Jussive      │  يَ      │  وا      │  يَكْتُبُوا      │
│ Energetic I  │  يَ      │  ونَّ    │  يَكْتُبُونَّ    │
│ Energetic II │  يَ      │  ونْ    │  يَكْتُبُونْ     │
└──────────────┴──────────┴──────────┴────────────────┘

Mood Variation — 2fs (strong affix -īna)
┌──────────────┬──────────┬──────────┬────────────────┐
│    Mood      │  Prefix  │  Suffix  │  Example         │
├──────────────┼──────────┼──────────┼────────────────┤
│ Indicative   │  تَ      │ ِينَ     │  تَكْتُبِينَ     │
│ Subjunctive  │  تَ      │  ي       │  تَكْتُبِي       │
│ Jussive      │  تَ      │  ي       │  تَكْتُبِي       │
│ Energetic I  │  تَ      │  ينَّ    │  تَكْتُبِينَّ    │
│ Energetic II │  تَ      │  ينْ    │  تَكْتُبِينْ     │
└──────────────┴──────────┴──────────┴────────────────┘
```

### 5.5 Summary: Regular Sound Affix Rules

For sound triliteral roots, the affixes for all 15 verb forms follow the same pattern. The only differences between forms are:

1. **Perfect stem** (e.g., فَعَلَ, فَعَّلَ, فَاعَلَ, أَفْعَلَ, etc.)
2. **Imperfect stem** (e.g., يَفْعُلُ, يُفَعِّلُ, يُفَاعِلُ, يُفْعِلُ, etc.)
3. **Imperfect stem vowel** (unpredictable for Form I only; fixed for Forms II–XV)

Thus, KB-0003 stores one set of affix tables (suffixes for perfect, prefixes + suffixes for imperfect) and refers to the form-specific stem from the wazan definition in KB-0002.

### 5.6 Form I — Perfect Active Summary Table

| Slot | Suffix | Slot | Suffix |
|------|--------|------|--------|
| 3ms | — (stem) | 3md | -ā |
| 3fs | -at | 3fd | -atā |
| 2ms | -ta | 2d | -tumā |
| 2fs | -ti | 3mp | -ū |
| 1s | -tu | 3fp | -na |
| — | — | 2mp | -tum |
| — | — | 2fp | -tunna |
| — | — | 1p | -nā |

### 5.7 Form I — Imperfect Indicative Active Summary Table

| Slot | Prefix | Suffix | Slot | Prefix | Suffix |
|------|--------|--------|------|--------|--------|
| 3ms | ya- | -u | 3md | ya- | -āni |
| 3fs | ta- | -u | 3fd | ta- | -āni |
| 2ms | ta- | -u | 2d | ta- | -āni |
| 2fs | ta- | -īna | 3mp | ya- | -ūna |
| 1s | a- | -u | 3fp | ya- | -na |
| 1p | na- | -u | 2mp | ta- | -ūna |
| — | — | — | 2fp | ta- | -na |

### 5.8 Derived Form Conjugation (Forms II–X)

For Forms II–X, the affix tables are identical to Form I sound. What changes is:
- The **stem** itself (defined by the wazan in KB-0002)
- The **imperfect prefix vowel** (for derived forms, the imperfect prefix vowel is always u/damma, unlike Form I where it can be a/i/u)

| Form | Perfect Stem | Imperfect 3ms | Imperfect Prefix Vowel |
|------|-------------|----------------|----------------------|
| **I** | فَعَلَ | يَفْعُلُ / يَفْعِلُ / يَفْعَلُ | a (variable) |
| **II** | فَعَّلَ | يُفَعِّلُ | u |
| **III** | فَاعَلَ | يُفَاعِلُ | u |
| **IV** | أَفْعَلَ | يُفْعِلُ | u |
| **V** | تَفَعَّلَ | يَتَفَعَّلُ | a (from ت prefix) |
| **VI** | تَفَاعَلَ | يَتَفَاعَلُ | a (from ت prefix) |
| **VII** | اِنْفَعَلَ | يَنْفَعِلُ | a |
| **VIII** | اِفْتَعَلَ | يَفْتَعِلُ | a |
| **IX** | اِفْعَلَّ | يَفْعَلُّ | a |
| **X** | اِسْتَفْعَلَ | يَسْتَفْعِلُ | a |

**Key insight for MOD-09:** The affixation rules are constant across forms for sound roots. KB-0003 stores **18 affix rules** (13 perfect suffixes + 5 imperfect affix slots), and each (form × conjugation class) combination specifies which stem they apply to.

---

## 6. Weak Root Conjugations (المعتل)

Weak roots contain one or more weak letters (و, ي, ا) that trigger phonological changes during conjugation. KB-0003 stores these as **paradigm variants** — modified versions of the sound paradigm with weak-specific affixation rules.

### 6.1 Assimilated Roots (Mithal — مثال)

Assimilated roots have **wāw (و) or yā (ي) as the first radical** (C₁). In Form I imperfect, C₁ (waw) assimilates with the prefix vowel. In the perfect, mithal roots conjugate like sound roots.

#### Form I Perfect (Mithal Wawi / W-ج-د)

```
Perfect Active — Mithal Wawi (Form I)
┌──────────┬────────────────┬───────────────┐
│   Slot   │  Suffix        │  Example (و-ج-د) │
├──────────┼────────────────┼───────────────┤
│ 3ms      │  —             │  وَجَدَ        │
│ 3fs      │  َت            │  وَجَدَت       │
│ 2ms      │  َت            │  وَجَدْتَ      │
│ ...      │  (same as sound) │  ...          │
└──────────┴────────────────┴───────────────┘

Perfect is identical to sound. Assimilation only affects imperfect.
```

#### Form I Imperfect Indicative (Mithal Wawi)

```
Imperfect Indicative Active — Mithal Wawi (Form I)
┌──────────┬──────────┬────────────┬───────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (و-ج-د) │
├──────────┼──────────┼────────────┼───────────────┤
│ 3ms      │  يَ      │  ُ         │  يَجِدُ        │ ← C₁ (w) drops!
│ 3fs      │  تَ      │  ُ         │  تَجِدُ        │
│ 2ms      │  تَ      │  ُ         │  تَجِدُ        │
│ 2fs      │  تَ      │ ِينَ       │  تَجِدِينَ     │
│ 1s       │  أَ      │  ُ         │  أَجِدُ        │
│ 3md      │  يَ      │ َانِ        │  يَجِدَانِ     │
│ ...      │  ...    │  ...       │  ...           │
└──────────┴──────────┴────────────┴───────────────┘

Rule: In the imperfect, C₁ (waw) assimilates into the prefix vowel,
resulting in C₁ being dropped. The paradigm then follows sound
conjugation on the remaining C₂ and C₃.
```

#### Assimilation Rule Summary (Mithal Wawi)

| Verb Form | Perfect | Imperfect Rule | Example |
|-----------|---------|---------------|---------|
| Form I | Sound-like (وَجَدَ) | C₁ (w) drops; C₂ vowel becomes i | يَجِدُ (yajidu) |
| Form II | Sound-like (وَلَّدَ) | Regular (geminated C₂ overrides) | يُوَلِّدُ (yuwallidu) |
| Form IV | Sound-like (أَوْجَدَ) | Regular (prefix أَ before C₁) | يُوجِدُ (yūjidu) |
| Form VII | Sound-like (اِنْوَجَدَ) | Not formed (rare) | — |
| Form VIII | Sound-like (اِتَّجَدَ) | C₁ (w) assimilates into infix ت | يَتَّجِدُ (yattajidu) |
| Form X | Sound-like (اِسْتَوْجَدَ) | Regular | يَسْتَوْجِدُ (yastawjidu) |

**Mithal Yai** (e.g., ي-ب-س — y-b-s, "to be dry"): Similar behavior, but with yā instead of waw. C₁ (y) does NOT assimilate in the imperfect for most forms: يَيْبَسُ (yaybasu). This is an important distinction between mithal wawi and mithal yai.

### 6.2 Hollow Roots (Ajwaf — أجوف)

Hollow roots have a **wāw (و) or yā (ي) as the second radical** (C₂). This is the most common weak root type (~500 roots). The medial weak letter undergoes systematic changes depending on tense and form.

#### Form I Perfect Active (Ajwaf Wawi — ق-و-ل)

```
Perfect Active — Ajwaf Wawi (Form I)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (ق-و-ل)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  قَالَ           │ ← C₂ (w) → ā
│ 3fs      │  َت            │  قَالَت          │
│ 2ms      │  َت            │  قُلْتَ          │ ← C₂ disappears!
│ 2fs      │  ِت            │  قُلْتِ          │
│ 1s       │  ُت            │  قُلْتُ          │
│ 3md      │  ا             │  قَالَا          │
│ 3fd      │ َتَا            │  قَالَتَا        │
│ 2d       │  تُّمَا         │  قُلْتُمَا       │
│ 3mp      │  وا             │  قَالُوا        │
│ 3fp      │  نَ             │  قُلْنَ          │
│ 2mp      │  تُمْ           │  قُلْتُمْ        │
│ 2fp      │  تُنَّ          │  قُلْتُنَّ       │
│ 1p       │  نَا            │  قُلْنَا         │
└──────────┴────────────────┴─────────────────┘

Key rule: When the suffix begins with a consonant (2ms/2fs/1s/2d/3fp/2mp/2fp/1p),
          C₂ (w) is completely elided and C₁ takes the vowel u.
          When the suffix begins with a vowel or is empty (3ms/3fs/3md/3fd/3mp),
          C₂ (w) becomes ā.
```

#### Form I Imperfect Indicative Active (Ajwaf Wawi)

```
Imperfect Indicative Active — Ajwaf Wawi (Form I)
┌──────────┬──────────┬────────────┬─────────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (ق-و-ل)   │
├──────────┼──────────┼────────────┼─────────────────┤
│ 3ms      │  يَ      │  ُ         │  يَقُولُ         │ ← C₂ (w) → ū
│ 3fs      │  تَ      │  ُ         │  تَقُولُ         │
│ 2ms      │  تَ      │  ُ         │  تَقُولُ         │
│ 2fs      │  تَ      │ ِينَ       │  تَقُولِينَ      │
│ 1s       │  أَ      │  ُ         │  أَقُولُ         │
│ 3md      │  يَ      │ َانِ        │  يَقُولَانِ      │
│ 3fd      │  تَ      │ َانِ        │  تَقُولَانِ      │
│ 2d       │  تَ      │ َانِ        │  تَقُولَانِ      │
│ 3mp      │  يَ      │ ُونَ        │  يَقُولُونَ      │
│ 3fp      │  يَ      │  نَ         │  يَقُلْنَ        │ ← C₂ drops before consonant
│ 2mp      │  تَ      │ ُونَ        │  تَقُولُونَ      │
│ 2fp      │  تَ      │  نَ         │  تَقُلْنَ        │ ← C₂ drops before consonant
│ 1p       │  نَ      │  ُ          │  نَقُولُ         │
└──────────┴──────────┴────────────┴─────────────────┘

Rule: In the imperfect, C₂ (w) → ū (long vowel) before vowel-initial suffixes.
      Before consonant-initial suffixes (3fp, 2fp), C₂ drops and C₁ vowel → u.
```

#### Full Paradigm: Ajwaf Jussive (Form I)

```
Imperfect Jussive Active — Ajwaf Wawi (Form I)
┌──────────┬──────────┬────────┬─────────────────┐
│   Slot   │  Prefix  │ Suffix │  Example (ق-و-ل)   │
├──────────┼──────────┼────────┼─────────────────┤
│ 3ms      │  يَ      │  —     │  يَقُلْ          │ ← C₂ drops entirely!
│ 3fs      │  تَ      │  —     │  تَقُلْ          │
│ 2ms      │  تَ      │  —     │  تَقُلْ          │
│ 2fs      │  تَ      │  ي     │  تَقُولِي        │ ← before -ī, C₂ surfaces as ū
│ 1s       │  أَ      │  —     │  أَقُلْ          │
│ 3md      │  يَ      │  ا     │  يَقُولَا        │
│ 3fd      │  تَ      │  ا     │  تَقُولَا        │
│ 2d       │  تَ      │  ا     │  تَقُولَا        │
│ 3mp      │  يَ      │  وا    │  يَقُولُوا       │
│ 3fp      │  يَ      │  نَ    │  يَقُلْنَ         │ ← C₂ drops before consonant
│ 2mp      │  تَ      │  وا    │  تَقُولُوا       │
│ 2fp      │  تَ      │  نَ    │  تَقُلْنَ         │
│ 1p       │  نَ      │  —     │  نَقُلْ          │
└──────────┴──────────┴────────┴─────────────────┘

Rule: In the jussive, C₂ drops when the suffix is empty (-∅).
      Before long vowel suffixes (-ī, -ā, -ū), C₂ surfaces as ū.
      Before consonant suffixes (-na), C₂ drops.
```

#### Hollow Root Summary Table

| Verb Form | Perfect 3ms | Imperfect 3ms | Behavior |
|-----------|-------------|---------------|----------|
| **I** (ajwaf wawi) | قَالَ | يَقُولُ | C₂ w → ā (perfect), ū (imperfect) |
| **I** (ajwaf yai) | سَارَ | يَسِيرُ | C₂ y → ā (perfect), ī (imperfect) |
| **II** (ajwaf) | قَوَّلَ | يُقَوِّلُ | Regular (geminated C₂ stabilizes) |
| **III** (ajwaf) | قَاوَلَ | يُقَاوِلُ | Regular (long ā after C₁) |
| **IV** (ajwaf) | أَقَالَ | يُقِيلُ | Complex: C₂ assimilates into prefix |
| **V** (ajwaf) | تَقَوَّلَ | يَتَقَوَّلُ | Regular (geminated C₂ stabilizes) |
| **VI** (ajwaf) | تَقَاوَلَ | يَتَقَاوَلُ | Regular (long ā after C₁) |
| **VII** (ajwaf) | اِنْقَالَ | يَنْقَالُ | C₂ becomes part of the long vowel |
| **VIII** (ajwaf) | اِقْتَالَ | يَقْتَالُ | C₂ assimilates (rare) |
| **X** (ajwaf) | اِسْتَقَالَ | يَسْتَقِيلُ | C₂ w → ā (perfect), ī (imperfect)

### 6.3 Defective Roots (Naqis — ناقص)

Defective roots have a **wāw (و) or yā (ي) as the third radical** (C₃). The final weak letter drops or changes in certain inflections.

#### Form I Perfect Active (Naqis Yai — ر-م-ي)

```
Perfect Active — Naqis Yai (Form I)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (ر-م-ي)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  رَمَى           │ ← C₃ (y) → ā (word-final)
│ 3fs      │  َت            │  رَمَت           │ ← C₃ drops before vowel suffix
│ 2ms      │  َت            │  رَمَيْتَ         │ ← C₃ (y) surfaces before consonant
│ 2fs      │  ِت            │  رَمَيْتِ         │
│ 1s       │  ُت            │  رَمَيْتُ         │
│ 3md      │  ا             │  رَمَيَا          │ ← before -ā, C₃ surfaces as y
│ 3fd      │ َتَا            │  رَمَتَا          │ ← C₃ drops at C₂-C₃ cluster with -atā
│ 2d       │  تُّمَا         │  رَمَيْتُمَا      │
│ 3mp      │  وا             │  رَمَوْا          │ ← C₃ (y) → w before suffix waw!
│ 3fp      │  نَ             │  رَمَيْنَ          │
│ 2mp      │  تُمْ           │  رَمَيْتُمْ       │
│ 2fp      │  تُنَّ          │  رَمَيْتُنَّ      │
│ 1p       │  نَا            │  رَمَيْنَا        │
└──────────┴────────────────┴─────────────────┘

Key rules:
  1. Word-final (3ms): C₃ (y/w) → ā (alif maqsura)
  2. Before vowel-initial suffixes (3fs -at, 3fd -atā): C₃ drops
  3. Before consonant-initial suffixes: C₃ surfaces (with vowel)
  4. Before suffix و (3mp): C₃ (y) → w (ي → و)
```

#### Form I Imperfect Indicative Active (Naqis Yai)

```
Imperfect Indicative Active — Naqis Yai (Form I)
┌──────────┬──────────┬────────────┬─────────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (ر-م-ي)   │
├──────────┼──────────┼────────────┼─────────────────┤
│ 3ms      │  يَ      │  ُ         │  يَرْمِي         │ ← C₃ (y) → ī (word-final)
│ 3fs      │  تَ      │  ُ         │  تَرْمِي         │
│ 2ms      │  تَ      │  ُ         │  تَرْمِي         │
│ 2fs      │  تَ      │ ِينَ       │  تَرْمِينَ       │ ← C₃ drops before īna!
│ 1s       │  أَ      │  ُ         │  أَرْمِي         │
│ 3md      │  يَ      │ َانِ        │  يَرْمِيَانِ     │
│ ...      │  ...    │  ...       │  ...             │
└──────────┴──────────┴────────────┴─────────────────┘

Imperfect Jussive Active — Naqis Yai (Form I)
┌──────────┬──────────┬────────┬─────────────────┐
│   Slot   │  Prefix  │ Suffix │  Example (ر-م-ي)   │
├──────────┼──────────┼────────┼─────────────────┤
│ 3ms      │  يَ      │  —     │  يَرْمِ          │ ← C₃ drops; C₂ vowel → i
│ 3fs      │  تَ      │  —     │  تَرْمِ          │
│ 2ms      │  تَ      │  —     │  تَرْمِ          │
│ 2fs      │  تَ      │  ي     │  تَرْمِي         │ ← before -ī, C₃ surfaces
│ ...      │  ...    │  ...   │  ...              │
└──────────┴──────────┴────────┴─────────────────┘

Key rule: In the jussive 3ms/3fs/2ms, C₃ drops and C₂ takes kasra.
           This is sometimes called "حذف حرف العلة" (deletion of the weak letter).
```

### 6.4 Lafif Roots (لفيف — Double-Weak)

Lafif roots contain **two** weak letters. They are divided into two subtypes:

| Subtype | Arabic Term | Pattern | Example |
|---------|-------------|---------|---------|
| **Lafif Mafruq** (لفيف مفروق) | Separated | C₁ is weak, C₃ is weak | و-ف-ي (w-f-y) |
| **Lafif Makrun** (لفيف مقرون) | Contiguous | C₂ is weak, C₃ is weak | ط-و-ي (ṭ-w-y) |

#### Lafif Mafruq — Form I (و-ف-ي, "to fulfill")

```
Perfect Active — Lafif Mafruq (Form I, root و-ف-ي)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (و-ف-ي)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  وَفَى           │ ← C₁ regular, C₃ → ā
│ 3fs      │  َت            │  وَفَت           │ ← C₃ drops before vowel
│ 2ms      │  َت            │  وَفَيْتَ         │ ← C₃ surfaces
│ ...      │  ...          │  ...              │
│ 3mp      │  وا             │  وَفَوْا         │ ← C₁ regular, C₃ → w
└──────────┴────────────────┴─────────────────┘

Imperfect Active — Lafif Mafruq (Form I)
┌──────────┬──────────┬────────────┬─────────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (و-ف-ي)   │
├──────────┼──────────┼────────────┼─────────────────┤
│ 3ms      │  يَ      │  ُ         │  يَفِي          │ ← C₁ drops (mithal assimilation)!
│ 3fs      │  تَ      │  ُ         │  تَفِي          │    + C₃ → ī (naqis)
│ 2ms      │  تَ      │  ُ         │  تَفِي          │
│ ...      │  ...    │  ...       │  ...             │
└──────────┴──────────┴────────────┴─────────────────┘

Lafif roots combine the rules of mithal + naqis:
  - Mithal rule: C₁ (w) drops in imperfect
  - Naqis rule: C₃ (y) → ī in imperfect word-final
```

---

## 7. Doubled Root Conjugation (المضاعف)

Doubled roots (مضاعف) have **identical C₂ and C₃** (e.g., م-د-د, ح-ب-ب). The two identical consonants merge with shadda in various forms.

### 7.1 Form I Perfect Active (م-د-د)

```
Perfect Active — Doubled (Form I, root م-د-د)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (م-د-د)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  مَدَّ           │ ← C₂=C₃ merge → madda
│ 3fs      │  َت            │  مَدَّت          │
│ 2ms      │  َت            │  مَدَدْتَ        │ ← merge broken before consonant
│ 2fs      │  ِت            │  مَدَدْتِ        │
│ 1s       │  ُت            │  مَدَدْتُ        │
│ 3md      │  ا             │  مَدَّا          │
│ 3fd      │ َتَا            │  مَدَّتَا        │
│ 2d       │  تُّمَا         │  مَدَدْتُمَا     │
│ 3mp      │  وا             │  مَدُّوا        │ ← merged before waw
│ 3fp      │  نَ             │  مَدَدْنَ        │ ← broken before consonant
│ 2mp      │  تُمْ           │  مَدَدْتُمْ      │
│ 2fp      │  تُنَّ          │  مَدَدْتُنَّ     │
│ 1p       │  نَا            │  مَدَدْنَا       │
└──────────┴────────────────┴─────────────────┘

Rule: C₂ and C₃ merge with shadda when the suffix begins with a vowel or is empty.
      When the suffix begins with a consonant, C₂ and C₃ separate: C₂aC₃.
```

### 7.2 Form I Imperfect Active (م-د-د)

```
Imperfect Indicative Active — Doubled (Form I)
┌──────────┬──────────┬────────────┬─────────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (م-د-د)   │
├──────────┼──────────┼────────────┼─────────────────┤
│ 3ms      │  يَ      │  ُ         │  يَمُدُّ         │ ← merged: yamuddu
│ 3fs      │  تَ      │  ُ         │  تَمُدُّ         │
│ 2ms      │  تَ      │  ُ         │  تَمُدُّ         │
│ 2fs      │  تَ      │ ِينَ       │  تَمُدِّينَ      │
│ 1s       │  أَ      │  ُ         │  أَمُدُّ         │
│ 3md      │  يَ      │ َانِ        │  يَمُدَّانِ      │
│ 3fp      │  يَ      │  نَ         │  يَمْدُدْنَ      │ ← separated before consonant
│ 2fp      │  تَ      │  نَ         │  تَمْدُدْنَ      │
└──────────┴──────────┴────────────┴─────────────────┘

Same pattern: merge when suffix is vowel-initial or empty.
              Separate when suffix begins with a consonant.
```

### 7.3 Doubled Root Imperfect Jussive

```
Imperfect Jussive Active — Doubled (Form I)
┌──────────┬──────────┬────────┬─────────────────┐
│   Slot   │  Prefix  │ Suffix │  Example (م-د-د)   │
├──────────┼──────────┼────────┼─────────────────┤
│ 3ms      │  يَ      │  —     │  يَمُدَّ / يَمْدُدْ │ ← two forms exist!
│ 3fs      │  تَ      │  —     │  تَمُدَّ / تَمْدُدْ │
│ 2ms      │  تَ      │  —     │  تَمُدَّ / تَمْدُدْ │
│ ...      │  ...    │  ...   │  ...               │
└──────────┴──────────┴────────┴─────────────────┘

Note: Doubled roots in jussive have two acceptable forms:
  - The "heavy" form with shadda: يَمُدَّ (yamudda)
  - The "light" form with separated consonants: يَمْدُدْ (yamdud)
KB-0003 stores both as variant entries with equal priority.
```

---

## 8. Hamzated Root Conjugation (المهموز)

Hamzated roots contain **hamza (ء)** as one or more radicals. Hamza behaves as a "semi-weak" letter — it follows regular conjugation but its orthographic seat changes based on surrounding vowels. This is primarily a writing-system phenomenon rather than a true phonological change.

### 8.1 Hamzated First (Mahmuz al-Fa — أ-ك-ل)

```
Perfect Active — Hamzated First (Form I, root أ-ك-ل)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (أ-ك-ل)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  أَكَلَ          │ ← regular conjugation
│ 3fs      │  َت            │  أَكَلَت         │
│ 2ms      │  َت            │  أَكَلْتَ        │
│ ...      │  (same affixes as sound) │  ...    │
└──────────┴────────────────┴─────────────────┘

Conjugation is identical to sound. Only orthographic seat changes may occur:
  أَكَلَ (akala) — stable hamza on alif
  In some forms: كُلْ (imperative of أَكَلَ) — hamza drops!
```

### 8.2 Hamzated Middle (Mahmuz al-Ayn — س-أ-ل)

```
Perfect Active — Hamzated Middle (Form I, root س-أ-ل)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (س-أ-ل)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  سَأَلَ          │
│ 3fs      │  َت            │  سَأَلَت         │
│ 2ms      │  َت            │  سَأَلْتَ        │
│ ...      │  ...          │  ...              │
└──────────┴────────────────┴─────────────────┘

Seat changes:
  سَأَلَ (sa'ala) — hamza on alif (between two fatha vowels)
  سُئِلَ (su'ila) — perfect passive: hamza on waw (between u and i)
  يَسْأَلُ (yas'alu) — imperfect: hamza on alif (between sukun and fatha)
  يَسْتِسْئِلُ (yastas'ilu) — Form X: hamza on alif (between vowels)
```

### 8.3 Hamzated Last (Mahmuz al-Lam — ق-ر-أ)

```
Perfect Active — Hamzated Last (Form I, root ق-ر-أ)
┌──────────┬────────────────┬─────────────────┐
│   Slot   │  Suffix        │  Example (ق-ر-أ)   │
├──────────┼────────────────┼─────────────────┤
│ 3ms      │  —             │  قَرَأَ          │
│ 3fs      │  َت            │  قَرَأَت         │
│ 2ms      │  َت            │  قَرَأْتَ        │
│ 3mp      │  وا             │  قَرَؤُوا       │ ← hamza on waw before waw!
│ ...      │  ...          │  ...              │
└──────────┴────────────────┴─────────────────┘

Seat rule: C₃ (hamza) takes orthographic seat based on preceding vowel:
  After fatha (a): hamza on alif — قَرَأَ
  After damma (u): hamza on waw — يَقْرُؤُ
  After kasra (i): hamza on ya — يَقْرِئُ

This is an orthographic rule (MOD-03 or output formatter), not a true conjugation difference.
KB-0003 stores hamzated paradigms as "sound-like" with orthographic seat notes.
```

---

## 9. Quadriliteral Verb Conjugation

Quadriliteral verbs follow simpler conjugation patterns than triliteral verbs. They have only three verb forms (QI–QIII) and no weak-letter complications (most quadriliteral roots are sound).

### 9.1 Form QI — Perfect Active (Sound Quadriliteral)

```
Stem: فَعْلَلَ (C₁aC₂C₃aC₄)

Perfect Active — Sound Quadriliteral
┌──────────┬────────────────┬─────────────────────┐
│   Slot   │  Suffix        │  Example (ز-ل-ز-ل)     │
├──────────┼────────────────┼─────────────────────┤
│ 3ms      │  —             │  زَلْزَلَ            │
│ 3fs      │  َت            │  زَلْزَلَت           │
│ 2ms      │  َت            │  زَلْزَلْتَ          │
│ 2fs      │  ِت            │  زَلْزَلْتِ          │
│ 1s       │  ُت            │  زَلْزَلْتُ          │
│ 3md      │  ا             │  زَلْزَلَا           │
│ 3fd      │ َتَا            │  زَلْزَلَتَا         │
│ 2d       │  تُّمَا         │  زَلْزَلْتُمَا       │
│ 3mp      │  وا             │  زَلْزَلُوا         │
│ 3fp      │  نَ             │  زَلْزَلْنَ          │
│ 2mp      │  تُمْ           │  زَلْزَلْتُمْ        │
│ 2fp      │  تُنَّ          │  زَلْزَلْتُنَّ       │
│ 1p       │  نَا            │  زَلْزَلْنَا         │
└──────────┴────────────────┴─────────────────────┘

Quadriliteral affixes follow the same pattern as triliteral sound verbs.
The suffixes are identical; only the stem structure differs.
```

### 9.2 Form QI — Imperfect Active

```
Imperfect Indicative Active — Sound Quadriliteral
┌──────────┬──────────┬────────────┬─────────────────────┐
│   Slot   │  Prefix  │  Suffix    │  Example (ز-ل-ز-ل)     │
├──────────┼──────────┼────────────┼─────────────────────┤
│ 3ms      │  يَ      │  ُ         │  يُزَلْزِلُ          │ ← prefix = yu-, C₃ → i
│ 3fs      │  تَ      │  ُ         │  تُزَلْزِلُ          │
│ 2ms      │  تَ      │  ُ         │  تُزَلْزِلُ          │
│ ...      │  ...    │  ...       │  ...                 │
└──────────┴──────────┴────────────┴─────────────────────┘

Quadriliteral imperfect prefix: yu- (with damma, not fatha like triliteral sound).
Imperfect stem: yuzalzilu (C₃ vowel = i).
```

### 9.3 Quadriliteral Verb Form Summary

| Form | Perfect Stem | Imperfect 3ms | Active Participle | Passive Participle |
|------|-------------|---------------|-------------------|-------------------|
| **QI** | فَعْلَلَ | يُفَعْلِلُ | مُفَعْلِل | مُفَعْلَل |
| **QII** | تَفَعْلَلَ | يَتَفَعْلَلُ | مُتَفَعْلِل | مُتَفَعْلَل |
| **QIII** | اِفْعَنْلَلَ | يَفْعَنْلِلُ | مُفْعَنْلِل | مُفْعَنْلَل |

---

## 10. Passive Voice Conjugation

The passive voice (المبني للمجهول) in Arabic is formed by **vowel change**, not by auxiliary verbs. The consonant skeleton remains the same as the active; only the vowel pattern changes.

### 10.1 Perfect Passive Vowel Rules

For all verb forms, the perfect passive follows this vowel pattern:
- C₁ vowel: u (damma)
- C₂ vowel: i (kasra) or ī (for stems with medial vowel)
- Pre-suffix vowel (C₃): i

| Form | Active Perfect (3ms) | Passive Perfect (3ms) | Rule |
|------|---------------------|----------------------|------|
| **I** | فَعَلَ | فُعِلَ | C₁a → u, C₂a → i |
| **II** | فَعَّلَ | فُعِّلَ | C₁a → u (geminated C₂ takes kasra) |
| **III** | فَاعَلَ | فُوعِلَ | C₁ā → ū, C₂a → i |
| **IV** | أَفْعَلَ | أُفْعِلَ | a- → u- |
| **V** | تَفَعَّلَ | تُفُعِّلَ | ta- → tu-, C₂a → u |
| **VI** | تَفَاعَلَ | تُفُوعِلَ | ta- → tu-, C₁ā → ū |
| **VII** | اِنْفَعَلَ | اُنْفُعِلَ | in- → un-, C₁a → u |
| **VIII** | اِفْتَعَلَ | اُفْتُعِلَ | i- → u-, C₁a → u |
| **IX** | اِفْعَلَّ | اُفْعُلَّ | i- → u-, C₂a → u |
| **X** | اِسْتَفْعَلَ | اُسْتُفْعِلَ | i- → u-, C₁a → u |

### 10.2 Imperfect Passive Vowel Rules

For the imperfect passive:
- Prefix vowel: u (damma) for all persons
- C₂ vowel: a (fatha)

Form I example:
- Active: يَفْعَلُ (yafʿalu) → Passive: يُفْعَلُ (yufʿalu)
- Active: يَفْعِلُ (yafʿilu) → Passive: يُفْعَلُ (yufʿalu)
- Active: يَفْعُلُ (yafʿulu) → Passive: يُفْعَلُ (yufʿalu)

Note: In the imperfect passive, the C₂ vowel is ALWAYS a, regardless of the active vowel.

### 10.3 Passive Paradigm Table (Form I Sound)

```
Perfect Passive — Sound Triliteral Form I
All slots follow same affixes as Section 5.2.
Stem vowel pattern: C₁uC₂iC₃ (before suffixes)

Imperfect Passive — Sound Triliteral Form I
All slots follow same affixes as Section 5.3.
Prefix vowel: u (always)
Stem vowel pattern: C₁C₂aC₃
```

---

## 11. Imperative Mood

The imperative (الأمر) is derived from the imperfect jussive. It exists only in the **second person** (2ms, 2fs, 2d, 2mp, 2fp) and only in the **active voice**.

### 11.1 Imperative Formation Rule

```
General rule:
  1. Take the 2ms imperfect jussive form.
  2. Remove the person prefix (تَ).
  3. If the remaining form starts with a consonant cluster, add an
     initial alif (ا) with the appropriate connecting vowel (i or u).
  4. The base connecting vowel is i (kasra) unless the C₂ vowel is u
     (damma), in which case the connecting vowel is u.
```

### 11.2 Imperative Table (Form I Sound)

```
Imperative — Sound Triliteral Form I (root ك-ت-ب, imperfect stem vowel = u)
┌──────────┬─────────────────┬───────────────┐
│   Slot   │  Form            │  Example       │
├──────────┼─────────────────┼───────────────┤
│ 2ms      │  اُفْعُلْ        │  اُكْتُبْ      │ ← prefix اُ (u for u-stem)
│ 2fs      │  اُفْعُلِي       │  اُكْتُبِي     │ ← suffix -ī
│ 2d       │  اُفْعُلَا       │  اُكْتُبَا     │ ← suffix -ā
│ 2mp      │  اُفْعُلُوا     │  اُكْتُبُوا   │ ← suffix -ū
│ 2fp      │  اُفْعُلْنَ      │  اُكْتُبْنَ    │ ← suffix -na
└──────────┴─────────────────┴───────────────┘

Imperative — Sound Triliteral Form I (root ج-ل-س, imperfect stem vowel = i)
┌──────────┬─────────────────┬───────────────┐
│   Slot   │  Form            │  Example       │
├──────────┼─────────────────┼───────────────┤
│ 2ms      │  اِفْعِلْ        │  اِجْلِسْ      │ ← prefix اِ (i for i-stem)
│ 2fs      │  اِفْعِلِي       │  اِجْلِسِي     │
│ 2d       │  اِفْعِلَا       │  اِجْلِسَا     │
│ 2mp      │  اِفْعِلُوا     │  اِجْلِسُوا   │
│ 2fp      │  اِفْعِلْنَ      │  اِجْلِسْنَ    │
└──────────┴─────────────────┴───────────────┘

Imperative connecting vowel rule:
  - If imperfect C₂ vowel = u → connecting vowel = u (اُفْعُلْ)
  - If imperfect C₂ vowel = a or i → connecting vowel = i (اِفْعِلْ / اِفْعَلْ)
```

### 11.3 Negative Imperative (النهي)

The negative imperative is simply the **2ms imperfect jussive** preceded by the particle **لَا (lā)**:

```
لَا تَفْعَلْ (lā tafʿal) — "don't do!"
لَا تَكْتُبْ (lā taktub) — "don't write!"

This applies to all slots (2ms, 2fs, 2d, 2mp, 2fp) using the jussive forms.
```

KB-0003 does not store negative imperative as a separate table; it is derived from the imperfect jussive + لَا at the orthographic level.

---

## 12. Verb Form Paradigm Summary (I–XV)

This section provides a **complete reference table** for all 15 verb forms showing the 3ms perfect and 3ms imperfect indicative for each conjugation class.

### 12.1 Perfect Active 3ms Reference

| Form | Sound | Hollow (Wawi) | Hollow (Yai) | Defective (Wawi) | Defective (Yai) | Doubled |
|------|-------|---------------|--------------|-----------------|-----------------|---------|
| **I** | فَعَلَ | قَالَ | سَارَ | دَعَا | رَمَى | مَدَّ |
| **II** | فَعَّلَ | قَوَّلَ | سَيَّرَ | دَعَّى | رَمَّى | مَدَّدَ |
| **III** | فَاعَلَ | قَاوَلَ | سَايَرَ | دَاعَى | رَامَى | مَادَّ |
| **IV** | أَفْعَلَ | أَقَالَ | أَسَارَ | أَدْعَى | أَرْمَى | أَمَدَّ |
| **V** | تَفَعَّلَ | تَقَوَّلَ | تَسَيَّرَ | تَدَعَّى | تَرَمَّى | تَمَدَّدَ |
| **VI** | تَفَاعَلَ | تَقَاوَلَ | تَسَايَرَ | تَدَاعَى | تَرَامَى | تَمَادَّ |
| **VII** | اِنْفَعَلَ | اِنْقَالَ | اِنْسَارَ | اِنْدَعَى | اِنْرَمَى | اِنْمَدَّ |
| **VIII** | اِفْتَعَلَ | اِقْتَالَ | اِسْتَارَ | اِدَّعَى | اِرْتَمَى | اِمْتَدَّ |
| **IX** | اِفْعَلَّ | — | — | — | — | اِمْدَدَّ |
| **X** | اِسْتَفْعَلَ | اِسْتَقَالَ | اِسْتَسَارَ | اِسْتَدْعَى | اِسْتَرْمَى | اِسْتَمَدَّ |
| **XI** | اِفْعَالَّ | — | — | — | — | — |
| **XII** | اِفْعَوْعَلَ | — | — | — | — | — |
| **XIII** | اِفْعَوَّلَ | — | — | — | — | — |
| **XIV** | اِفْعَنْلَلَ | — | — | — | — | — |
| **XV** | اِفْعَنْلَى | — | — | — | — | — |

Note: "—" indicates the form is not attested or extremely rare for this conjugation class. Forms XI–XV are rare even for sound roots and are generally not used with weak roots.

### 12.2 Imperfect Active 3ms Reference

| Form | Sound | Hollow (Wawi) | Hollow (Yai) | Defective (Wawi) | Defective (Yai) | Doubled |
|------|-------|---------------|--------------|-----------------|-----------------|---------|
| **I** | يَفْعُلُ | يَقُولُ | يَسِيرُ | يَدْعُو | يَرْمِي | يَمُدُّ |
| **II** | يُفَعِّلُ | يُقَوِّلُ | يُسَيِّرُ | يُدَعِّي | يُرَمِّي | يُمَدِّدُ |
| **III** | يُفَاعِلُ | يُقَاوِلُ | يُسَايِرُ | يُدَاعِي | يُرَامِي | يُمَادُّ |
| **IV** | يُفْعِلُ | يُقِيلُ | يُسِيرُ | يُدْعِي | يُرْمِي | يُمِدُّ |
| **V** | يَتَفَعَّلُ | يَتَقَوَّلُ | يَتَسَيَّرُ | يَتَدَعَّى | يَتَرَمَّى | يَتَمَدَّدُ |
| **VI** | يَتَفَاعَلُ | يَتَقَاوَلُ | يَتَسَايَرُ | يَتَدَاعَى | يَتَرَامَى | يَتَمَادُّ |
| **VII** | يَنْفَعِلُ | يَنْقَالُ | يَنْسَارُ | يَنْدَعِي | يَنْرَمِي | يَنْمَدُّ |
| **VIII** | يَفْتَعِلُ | يَقْتَالُ | يَسْتَارُ | يَدَّعِي | يَرْتَمِي | يَمْتَدُّ |
| **IX** | يَفْعَلُّ | — | — | — | — | يَمْدَدُّ |
| **X** | يَسْتَفْعِلُ | يَسْتَقِيلُ | يَسْتَسِيرُ | يَسْتَدْعِي | يَسْتَرْمِي | يَسْتَمِدُّ |
| **XI** | يَفْعَالُّ | — | — | — | — | — |
| **XII–XV** | various | — | — | — | — | — |

### 12.3 Passive 3ms Reference

| Form | Sound Perfect | Sound Imperfect | Hollow Perf (W) | Hollow Imp (W) |
|------|---------------|-----------------|-----------------|----------------|
| **I** | فُعِلَ | يُفْعَلُ | قِيلَ | يُقَالُ |
| **II** | فُعِّلَ | يُفَعَّلُ | قُوِّلَ | يُقَوَّلُ |
| **III** | فُوعِلَ | يُفَاعَلُ | قُووِلَ | يُقَاوَلُ |
| **IV** | أُفْعِلَ | يُفْعَلُ | أُقِيلَ | يُقَالُ |
| **V** | تُفُعِّلَ | يُتَفَعَّلُ | تُقُوِّلَ | يُتَقَوَّلُ |
| **VI** | تُفُوعِلَ | يُتَفَاعَلُ | تُقُووِلَ | يُتَقَاوَلُ |
| **VII** | اُنْفُعِلَ | يُنْفَعَلُ | اُنْقِيلَ | يُنْقَالُ |
| **VIII** | اُفْتُعِلَ | يُفْتَعَلُ | اُقْتِيلَ | يُقْتَالُ |
| **IX** | اُفْعُلَّ | يُفْعَلُّ | — | — |
| **X** | اُسْتُفْعِلَ | يُسْتَفْعَلُ | اُسْتُقِيلَ | يُسْتَقَالُ |

### 12.4 Imperative 2ms Reference

| Form | Sound | Hollow (Wawi) | Defective (Yai) | Doubled |
|------|-------|--------------|-----------------|---------|
| **I** | اُفْعُلْ / اِفْعِلْ | قُلْ | اِرْمِ | مُدَّ / اُمْدُدْ |
| **II** | فَعِّلْ | قَوِّلْ | رَمِّ | مَدِّدْ |
| **III** | فَاعِلْ | قَاوِلْ | رَامِ | مَادَّ / مَادِدْ |
| **IV** | أَفْعِلْ | أَقِلْ | أَرْمِ | أَمِدَّ / أَمْدِدْ |
| **V** | تَفَعَّلْ | تَقَوَّلْ | تَرَمَّ | تَمَدَّدْ |
| **VI** | تَفَاعَلْ | تَقَاوَلْ | تَرَامَ | تَمَادَّ / تَمَادَدْ |
| **VII** | اِنْفَعِلْ | اِنْقَلْ | اِنْرَمِ | اِنْمَدَّ / اِنْمَدِدْ |
| **VIII** | اِفْتَعِلْ | اِقْتَلْ | اِرْتَمِ | اِمْتَدَّ / اِمْتَدِدْ |
| **IX** | اِفْعَلَلْ | — | — | اِمْدَدَدْ |
| **X** | اِسْتَفْعِلْ | اِسْتَقِلْ | اِسْتَرْمِ | اِسْتَمِدَّ / اِسْتَمْدِدْ |

---

## 13. Paradigm Lookup Algorithms

### 13.1 Primary Algorithm: Conjugate Verb

```pseudo
Algorithm: conjugate_verb
Input: root (RootEntry), verb_form (1–15), conjugation_class (string),
       tense ("perfect" | "imperfect"), mood ("indicative" | "subjunctive" |
       "jussive" | "energetic_I" | "energetic_II"), voice ("active" | "passive"),
       slot (string)  # e.g., "3ms", "1p"
Output: string (inflected word form) | null

1. Determine paradigm table key:
   a. table_key = "{verb_form}:{conjugation_class}:{voice}:{tense}"
   b. Look up ParadigmTable in KB-0003 index.
   c. If not found → return null (unattested combination).

2. Retrieve the paradigm rules:
   a. If tense == "perfect":
      i.   Get the perfect_stem (3ms form).
      ii.  Get the perfect_rule for the target slot.
      iii. If slot == "3ms" → return perfect_stem (no suffix).
   b. If tense == "imperfect":
      i.   Get the imperfect_stem.
      ii.  Get the imperfect_rule for the target slot.
      iii. Apply mood-specific suffix adjustments:
           - indicative: use standard suffix
           - subjunctive: apply subjunctive suffix changes
           - jussive: apply jussive suffix changes
           - energetic: apply energetic affix changes

3. Apply conjugation rules:
   a. Start with the stem (perfect or imperfect).
   b. Apply slot prefix (if any).
   c. Apply slot suffix (if any).
   d. Apply stem vowel changes (from ParadigmRule).
   e. Apply stem_change rules (for weak roots).
   f. Apply phonological rules (assimilation, elision, etc.).

4. For weak root conjugation classes:
   a. Look up the specific weak variant rules.
   b. Override or adjust the base sound conjugation rules.
   c. Apply weak-specific transformations (Section 6 rules).

5. Orthographic normalization:
   a. Apply hamza seat rules (for hamzated roots).
   b. Apply orthographic adjustments (e.g., alif maqsura → alif).
   c. Normalize Unicode representation.

6. Return the inflected word form.
```

### 13.2 Secondary Algorithm: Analyze Inflected Form

```pseudo
Algorithm: analyze_inflected_form
Input: word_form (string), root_candidates (RootEntry[]),
       wazan_candidates (WazanEntry[])
Output: ConjugationAnalysis[] (ranked)

1. For each (root, wazan) candidate pair:
   a. Determine the conjugation_class from:
      i.   KB-0001: root.root_type → maps to conjugation class
      ii.  KB-0002: wazan.verb_form and wazan.inherent_features
   b. Set voice: try active then passive.
   c. Set tense: try perfect then imperfect.

2. For each candidate configuration (root, wazan, voice, tense):
   a. Generate all 13 slot forms using conjugate_verb (Algorithm 13.1).
   b. Compare each generated form against the input word_form.
   c. If exact match → high confidence.

3. If no exact match:
   a. For imperfect: try all 4 moods (indicative, subjunctive, jussive, energetic).
   b. For weak roots: try weak-specific slot patterns.
   c. Check imperative separately (try the 5 second-person forms).

4. Score matches by:
   a. Exactness of conjugation match.
   b. Agreement with root's attested verb forms (KB-0001).
   c. Agreement with wazan's inherent features.

5. Return ordered ConjugationAnalysis[] (may be empty).
```

### 13.3 Lookup Key Structure

For O(1) lookup, the compiled KB-0003 index uses a composite key:

```
LookupKey = {
  verb_form:     4 bits  (I–XV → 0–14, QI–QIII → 15–17)
  conj_class:    4 bits  (sound=0, hollow_w=1, hollow_y=2, ...)
  voice:         1 bit   (active=0, passive=1)
  tense:         1 bit   (perfect=0, imperfect=1)
  mood:          3 bits  (ind=0, sub=1, jus=2, en1=3, en2=4)
  person:        2 bits  (1st=0, 2nd=1, 3rd=2)
  number:        2 bits  (sg=0, dl=1, pl=2)
  gender:        1 bit   (masc=0, fem=1)
  ---
  Total:         18 bits → fits in u32
}

LookupKey encoding: (verb_form << 14) | (conj_class << 10) | (voice << 9) |
                    (tense << 8) | (mood << 5) | (person << 3) |
                    (number << 1) | gender
```

This key directly indexes into a flat array of u32 offsets (pointing to string table entries), enabling **O(1) conjugation form lookup**.

---

## 14. Serialization & Storage

### 14.1 Source Format

```diff
  /knowledge/KB-0003/
  ├── metadata.yaml                     # KB metadata (version, counts, classes)
  ├── affix-rules/
  │   ├── perfect-suffixes.yaml         # 13 perfect suffix definitions (sound base)
  │   ├── imperfect-prefixes.yaml       # 5 imperfect prefix definitions
  │   ├── imperfect-suffixes.yaml       # 13 imperfect suffix definitions
  │   ├── mood-variations.yaml          # Indicative/subjunctive/jussive/energetic rules
  │   └── imperative-rules.yaml         # Imperative formation from jussive
  ├── sound/
  │   ├── form-I.yaml                   # Sound Form I full paradigm
  │   ├── form-II.yaml                  # Sound Form II full paradigm
  │   ├── ...
  │   └── form-XV.yaml                  # Sound Form XV full paradigm
  ├── weak/
  │   ├── hollow-wawi/
  │   │   ├── form-I.yaml
  │   │   ├── ...
  │   ├── hollow-yai/
  │   │   └── ...
  │   ├── defective-wawi/
  │   │   └── ...
  │   ├── defective-yai/
  │   │   └── ...
  │   ├── assimilated-wawi/
  │   │   └── ...
  │   ├── assimilated-yai/
  │   │   └── ...
  │   ├── doubled/
  │   │   └── ...
  │   ├── hamzated/
  │   │   └── ...
  │   └── lafif/
  │       └── ...
  └── quadriliteral/
      ├── form-QI.yaml
      ├── form-QII.yaml
      └── form-QIII.yaml
```

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0003"
title: "Verb Forms — Conjugation Paradigms"
version: "1.0.0"
status: "draft" | "review" | "published"

paradigm_count: 218
conjugation_classes: 15
verb_forms: ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X",
            "XI", "XII", "XIII", "XIV", "XV", "QI", "QII", "QIII"]

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

checksum_sha256: "c3d4e5f6a7b8..."
maintainers:
  - name: "Dr. [Name]"
    email: "[email]"
    role: "morphology_editor"
```

### 14.2 Compiled Format (Table Binary)

The production format is a **pre-computed table binary** with all inflected forms indexed by composite key:

```diff
  Compiled Table Binary Layout:
  ┌──────────────────────────────────────────────────────────────┐
  │ HEADER                                                       │
  │ ├── magic: "AGOSKB03" (8 bytes)                             │
  │ ├── version: major(2B) + minor(2B) + patch(2B)              │
  │ ├── paradigm_count: u32 (4 bytes)                           │
  │ ├── affix_table_offset: u32 (4 bytes)                       │
  │ ├── form_table_offset: u32 (4 bytes)                        │
  │ ├── string_table_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ AFFIX TABLE                                                  │
  │ ├── Base suffix table (13 entries × fixed size)             │
  │ │   ├── suffix_utf8: u32 → string table                    │
  │ │   ├── prefix_utf8: u32 → string table                    │
  │ │   └── flags: u8 (stem_vowel_change, elision, etc.)       │
  │ ├── Base prefix table (5 entries × fixed size)              │
  │ └── Mood variation rules                                    │
  ├──────────────────────────────────────────────────────────────┤
  │ FORM INDEX                                                   │
  │ ├── Per (verb_form, conj_class, voice) entry:               │
  │ │   ├── perfect_stem_offset: u32 (→ string table)          │
  │ │   ├── imperfect_stem_offset: u32                         │
  │ │   ├── base_affix_set_id: u16                             │
  │ │   ├── weak_rule_bitmask: u16                              │
  │ │   └── slot_overrides: u16 × 13 (index into override arr) │
  │ └── ... (paradigm_count entries)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ COMPUTED PARADIGM CACHE                                      │
  │ ├── Pre-computed forms for common lookups                   │
  │ ├── Key: u32 (LookupKey encoding) → form u32 (→ string)    │
  │ └── ~1,400 entries                                          │
  ├──────────────────────────────────────────────────────────────┤
  │ STRING TABLE                                                 │
  │ ├── Length-prefixed UTF-8 strings                           │
  │ ├── All inflected word forms, stems, affixes                │
  │ └── Referenced by offsets from all tables                   │
  └──────────────────────────────────────────────────────────────┘
```

#### C Struct: Paradigm Index Entry

```c
struct ParadigmEntry {
    uint32_t perfect_stem_offset;         // → string table
    uint32_t imperfect_stem_offset;       // → string table
    uint8_t  base_affix_set;              // Which affix set to use (0=sound)
    uint8_t  weak_rule_set;              // Which weak root rule set to apply
    uint16_t slot_override_bitmask;       // Bits: which slots have overrides
    uint16_t slot_override_offsets[13];   // → override rules for each slot
    uint8_t  imperfect_prefix_vowel;      // 0=a, 1=u, 2=i
    uint8_t  reserved[3];                // Alignment padding
};
```

### 14.3 File Packaging

```diff
  KB-0003-v1.0.0.agos-kb              # Compiled table binary
  KB-0003-v1.0.0.agos-kb.sig          # Ed25519 signature
  KB-0003-v1.0.0.agos-kb.sha256       # SHA-256 checksum
  KB-0003-v1.0.0.source.tar.gz        # Source YAML files (optional)
```

### 14.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Form index | 2 MB | 4 MB | ~250 entries × 64 bytes |
| Affix tables | 1 MB | 2 MB | Suffix/prefix definitions |
| Computed paradigm cache | 3 MB | 8 MB | ~1,400 pre-computed forms |
| Slot overrides | 3 MB | 6 MB | Weak root slot-level adjustments |
| String table | 4 MB | 7 MB | All inflected word forms |
| Phonological rules | 2 MB | 3 MB | Weak root transformation rules |
| **Total** | **~15 MB** | **~30 MB** | Memory-mapped load |

The **Compact** format drops rare forms (XI–XV, dual paradigms for all slots) and computes them on-the-fly from rules. The **Full** format pre-computes all 13 × N forms.

---

## 15. Versioning & Evolution

### 15.1 Versioning Scheme

KB-0003 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to paradigm schema, removal of paradigm classes, format change | `1.0.0` → `2.0.0` | Requires KB conversion tool, invalidates all caches |
| **MINOR** | Addition of new conjugation classes, new paradigm variants, new optional fields | `1.0.0` → `1.1.0` | Backward-compatible; existing paradigm IDs remain valid |
| **PATCH** | Corrections to affixation rules, improved weak root handling, typo fixes | `1.0.0` → `1.0.1` | Backward-compatible; no schema changes |

### 15.2 Cross-KB Compatibility

```yaml
cross_kb_compatibility:
  KB-0001: ">= 1.0.0"       # Conjugation classes reference root types
  KB-0002: ">= 1.0.0"       # Verb form wazans link to conjugation paradigms
  KB-0004: ">= 1.0.0"       # Shared paradigm patterns
  KB-0005: ">= 1.0.0"       # Independent (no verb conjugation dependency)
  KB-0006: ">= 1.0.0"       # Independent (no verb conjugation dependency)
  KB-0007: ">= 1.0.0"       # tense, mood, voice, person, number, gender features
```

### 15.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add new conjugation class | MINOR | Add paradigm tables, update index |
| Correct affix rule | PATCH | Edit rule definition, regenerate tables |
| Add weak root variant | MINOR | Add variant entry, link to base paradigm |
| Add new verb form (XI–XV) | MINOR | Add form × class tables where attested |
| Remove paradigm class | MAJOR | Only for demonstrably incorrect entries |
| Change pre-computed forms | PATCH | Recompute paradigm cache |

---

## 16. Quality Requirements

### 16.1 Completeness Targets

| Category | Minimum | Target | Stretch |
|----------|---------|--------|---------|
| Sound triliteral (Forms I–X) | 100% | 100% | 100% |
| Sound triliteral (Forms XI–XV) | 60% | 80% | 100% |
| Hollow wawi (Forms I–X) | 90% | 95% | 100% |
| Hollow yai (Forms I–X) | 85% | 90% | 95% |
| Defective wawi (Forms I–X) | 85% | 90% | 95% |
| Defective yai (Forms I–X) | 85% | 90% | 95% |
| Doubled (Forms I–X) | 90% | 95% | 100% |
| Assimilated wawi (Forms I, IV, VIII, X) | 80% | 90% | 95% |
| Hamzated (Forms I–X) | 80% | 90% | 95% |
| Quadriliteral (QI–QIII) | 90% | 95% | 100% |
| Passive voice (sound I–X) | 100% | 100% | 100% |
| Mood variations (indicative + subjunctive + jussive) | 100% | 100% | 100% |
| Energetic moods | 50% | 70% | 90% |
| Imperative (active, sound) | 100% | 100% | 100% |

### 16.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Paradigm slot correctness | 100% — each inflected form must match reference grammar | Automated cross-check with reference corpora |
| Weak root rule correctness | 100% — weak transformations must produce correct forms | Per-class regression tests |
| Stem-affix combination | 100% — no illegal affix-stem combinations | Automated validation |
| Cross-KB consistency (KB-0001) | 100% — conjugation classes referenced must exist | Automated cross-KB check |
| Cross-KB consistency (KB-0002) | 100% — verb forms referenced must exist | Automated cross-KB check |
| Passive voice consistency | 100% — all active forms must have passive counterparts | Automated check |
| Unicode normalization | 100% — all Arabic text valid NFC-normalized UTF-8 | Automated encoding check |

### 16.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate YAML structure
  ├── schema: validate against KB-0003 JSON Schema
  ├── affix_check: verify all slots have valid affixes
  ├── stem_check: verify stems match KB-0002 wazan templates
  └── lint: field presence, Arabic-only text

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── cross_kb: verify verb form IDs exist in KB-0002
  ├── cross_kb: verify conjugation classes exist in KB-0001
  ├── paradigm_completeness: verify all expected slots are filled
  ├── weak_regression: verify known weak root conjugations produce correct forms
  ├── compilation: verify table binary compiles without error
  ├── size_budget: verify compiled size ≤ 30 MB
  └── regression: verify 100+ known verb forms still conjugate correctly

  Review (manual, per release):
  ├── sample_check: linguist reviews 2% random paradigm sample
  ├── hotspot_check: review paradigms modified since last version
  ├── weak_paradigm_audit: verify weak root paradigms against Wright's Grammar
  └── changelog: verify changelog accuracy
```

### 16.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Single slot conjugation (lookup) | < 1 μs | Pre-computed table lookup |
| Single slot conjugation (computed) | < 5 μs | Rule-based derivation |
| Full 13-slot paradigm generation | < 30 μs | Per verb form, average |
| Weak root conjugation | < 10 μs | Additional rule overhead |
| KB load time (compact) | < 25 ms | mmap + verify checksum |
| KB load time (full) | < 50 ms | mmap + verify checksum |
| Memory (compact) | ~15 MB | RSS |
| Memory (full) | ~30 MB | RSS |

---

## 17. Example Entries

### 17.1 Sound Triliteral — Form I Complete (ك-ت-ب)

```json
{
  "id": "KB-0003:I:sound:active:perfect",
  "verb_form": "I",
  "conjugation_class": "sound",
  "voice": "active",
  "tense": "perfect",
  "perfect_stem": {
    "stem_form": "فَعَلَ",
    "stem_template": "C₁aC₂aC₃",
    "stem_note": "Apply to any sound triliteral root"
  },
  "perfect_conjugation": [
    { "slot": "3ms", "suffix": "" },
    { "slot": "3fs", "suffix": "َت" },
    { "slot": "2ms", "suffix": "َتَ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "2fs", "suffix": "َتِ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "1s", "suffix": "َتُ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "3md", "suffix": "َا" },
    { "slot": "3fd", "suffix": "َتَا" },
    { "slot": "2d", "suffix": "َتُّمَا" },
    { "slot": "3mp", "suffix": "ُو" },
    { "slot": "3fp", "suffix": "ْنَ", "stem_vowel": "sukun_on_C₂" },
    { "slot": "2mp", "suffix": "َتُّمْ" },
    { "slot": "2fp", "suffix": "َتُنَّ" },
    { "slot": "1p", "suffix": "ْنَا", "stem_vowel": "sukun_on_C₂" }
  ]
}
```

### 17.2 Hollow Wawi — Form I Perfect (ق-و-ل)

```json
{
  "id": "KB-0003:I:hollow_wawi:active:perfect",
  "verb_form": "I",
  "conjugation_class": "hollow_wawi",
  "voice": "active",
  "tense": "perfect",
  "perfect_stem": {
    "stem_form": "فَالَ",
    "stem_template": "C₁āC₃",
    "stem_note": "C₂ (w) → ā in 3ms/3fs/3md/3fd/3mp; C₂ drops in 2ms/2fs/1s/2d/3fp/2mp/2fp/1p"
  },
  "perfect_conjugation": [
    { "slot": "3ms", "suffix": "" },
    { "slot": "3fs", "suffix": "َت" },
    { "slot": "2ms", "suffix": "َتَ", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "2fs", "suffix": "َتِ", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "1s", "suffix": "َتُ", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "3md", "suffix": "َا" },
    { "slot": "3fd", "suffix": "َتَا" },
    { "slot": "2d", "suffix": "َتُّمَا", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "3mp", "suffix": "ُو" },
    { "slot": "3fp", "suffix": "ْنَ", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "2mp", "suffix": "َتُّمْ", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "2fp", "suffix": "َتُنَّ", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" },
    { "slot": "1p", "suffix": "ْنَا", "stem_change": "C₂_drop", "stem_vowel": "u_on_C₁" }
  ],
  "phonological_rules": [
    {
      "condition": "slot in [2ms, 2fs, 1s, 2d, 3fp, 2mp, 2fp, 1p]",
      "operation": "elide C₂ (weak letter) and assign C₁ vowel u",
      "affected_slots": ["2ms", "2fs", "1s", "2d", "3fp", "2mp", "2fp", "1p"],
      "rule_type": "elision"
    },
    {
      "condition": "slot in [3ms, 3fs, 3md, 3fd, 3mp]",
      "operation": "change C₂ (waw) to long ā",
      "affected_slots": ["3ms", "3fs", "3md", "3fd", "3mp"],
      "rule_type": "substitution"
    }
  ]
}
```

### 17.3 Defective Yai — Form I Jussive (ر-م-ي)

```json
{
  "id": "KB-0003:I:defective_yai:active:jussive",
  "verb_form": "I",
  "conjugation_class": "defective_yai",
  "voice": "active",
  "tense": "imperfect",
  "mood": "jussive",
  "imperfect_stem": {
    "stem_form": "C₁C₂ī",
    "stem_template": "C₁C₂ī",
    "stem_note": "Imperfect indicative 3ms: yarmī; jussive: yarmi (C₃ drops)"
  },
  "imperfect_conjugation": [
    { "slot": "3ms", "prefix": "يَ", "suffix": "", "stem_change": "C₃_drop" },
    { "slot": "3fs", "prefix": "تَ", "suffix": "", "stem_change": "C₃_drop" },
    { "slot": "2ms", "prefix": "تَ", "suffix": "", "stem_change": "C₃_drop" },
    { "slot": "2fs", "prefix": "تَ", "suffix": "ي" },
    { "slot": "1s", "prefix": "أَ", "suffix": "", "stem_change": "C₃_drop" },
    { "slot": "3md", "prefix": "يَ", "suffix": "ا" },
    { "slot": "3fd", "prefix": "تَ", "suffix": "ا" },
    { "slot": "2d", "prefix": "تَ", "suffix": "ا" },
    { "slot": "3mp", "prefix": "يَ", "suffix": "وا" },
    { "slot": "3fp", "prefix": "يَ", "suffix": "نَ", "stem_change": "C₃_drop" },
    { "slot": "2mp", "prefix": "تَ", "suffix": "وا" },
    { "slot": "2fp", "prefix": "تَ", "suffix": "نَ", "stem_change": "C₃_drop" },
    { "slot": "1p", "prefix": "نَ", "suffix": "", "stem_change": "C₃_drop" }
  ]
}
```

### 17.4 Sound Quadriliteral — Form QI Perfect (ز-ل-ز-ل)

```json
{
  "id": "KB-0003:QI:sound_quadriliteral:active:perfect",
  "verb_form": "QI",
  "conjugation_class": "sound_quadriliteral",
  "voice": "active",
  "tense": "perfect",
  "perfect_stem": {
    "stem_form": "فَعْلَلَ",
    "stem_template": "C₁aC₂C₃aC₄",
    "stem_note": "Quadriliteral perfect stem; suffixes same as sound triliteral"
  },
  "perfect_conjugation": [
    { "slot": "3ms", "suffix": "" },
    { "slot": "3fs", "suffix": "َت" },
    { "slot": "2ms", "suffix": "َتَ", "stem_vowel": "sukun_on_C₄" },
    { "slot": "2fs", "suffix": "َتِ", "stem_vowel": "sukun_on_C₄" },
    { "slot": "1s", "suffix": "َتُ", "stem_vowel": "sukun_on_C₄" },
    { "slot": "3md", "suffix": "َا" },
    { "slot": "3fd", "suffix": "َتَا" },
    { "slot": "2d", "suffix": "َتُّمَا" },
    { "slot": "3mp", "suffix": "ُو" },
    { "slot": "3fp", "suffix": "ْنَ", "stem_vowel": "sukun_on_C₃" },
    { "slot": "2mp", "suffix": "َتُّمْ" },
    { "slot": "2fp", "suffix": "َتُنَّ" },
    { "slot": "1p", "suffix": "ْنَا", "stem_vowel": "sukun_on_C₃" }
  ],
  "phonological_rules": [
    {
      "condition": "suffix begins with a consonant",
      "operation": "C₄ takes sukun before consonant-initial suffix",
      "affected_slots": ["2ms", "2fs", "1s", "2d", "2mp", "2fp"],
      "rule_type": "elision"
    },
    {
      "condition": "suffix begins with ن or begins with consonant after C₃",
      "operation": "C₃ takes sukun before ن suffix",
      "affected_slots": ["3fp", "2fp", "1p"],
      "rule_type": "elision"
    }
  ]
}
```

---

## 18. Cross-References

### 18.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0003 in module catalog |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Verb form analysis using KB-0003 |
| SPEC-0001-C3 | Compilation Pipeline (MOD-09) | Verb form generation using KB-0003 |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Conjugation interface in MOD-04/MOD-09 |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | Conjugation features in IR-4/IR-8 |
| SPEC-0001-C6 | Deployment & Runtime Considerations | KB bundling, size budget |
| SPEC-0001-C8 | Security, Validation & Error Handling | KB integrity verification |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0003 size, table lookup performance |
| KB-0001 | Roots Database | Root types and attestations informing conjugation |
| KB-0002 | Wazan Database | Verb form patterns that KB-0003 conjugates |
| KB-0004 | Noun Patterns | Shared paradigm patterns |
| KB-0007 | Morphological Features | Feature taxonomy for conjugation features |

### 18.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (8th C. CE) | Foundational grammar for Arabic verb conjugation |
| Al-Mubarrad, Al-Muqtadab (9th C. CE) | Detailed analysis of weak verb conjugation |
| Ibn Hisham, Qatr al-Nada (14th C. CE) | I'rab-based conjugation analysis |
| Al-Ashmuni, Sharh al-Ashmuni (15th C. CE) | Comprehensive grammatical commentary |
| Wright's Arabic Grammar (1859) | Western reference for Arabic conjugation tables |
| Haywood & Nahmad, A New Arabic Grammar (1962) | Modern pedagogical reference |
| Buckwalter Arabic Morphological Analyzer (BAMA) | Reference for computational conjugation |
| Standard Arabic: An Elementary-Intermediate Course (Brustad, Al-Batal, Al-Tonsi) | Modern pedagogical paradigms |

---

## Progress Summary

**KB-0003: Verb Forms — Conjugation Paradigms**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Conjugation in Arabic Grammar | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Paradigm Entry Schema | ✓ COMPLETE |
| Section 5 | Sound Triliteral Conjugation | ✓ COMPLETE |
| Section 6 | Weak Root Conjugations | ✓ COMPLETE |
| Section 7 | Doubled Root Conjugation | ✓ COMPLETE |
| Section 8 | Hamzated Root Conjugation | ✓ COMPLETE |
| Section 9 | Quadriliteral Verb Conjugation | ✓ COMPLETE |
| Section 10 | Passive Voice Conjugation | ✓ COMPLETE |
| Section 11 | Imperative Mood | ✓ COMPLETE |
| Section 12 | Verb Form Paradigm Summary (I–XV) | ✓ COMPLETE |
| Section 13 | Paradigm Lookup Algorithms | ✓ COMPLETE |
| Section 14 | Serialization & Storage | ✓ COMPLETE |
| Section 15 | Versioning & Evolution | ✓ COMPLETE |
| Section 16 | Quality Requirements | ✓ COMPLETE |
| Section 17 | Example Entries | ✓ COMPLETE |
| Section 18 | Cross-References | ✓ COMPLETE |

**Dependencies:** KB-0001 (Roots Database), KB-0002 (Wazan Database), SPEC-0001 (Chapters 1–9).

**Recommended next document:** KB-0004 (Noun Patterns) — the detailed noun pattern specifications for AGOS.
```

