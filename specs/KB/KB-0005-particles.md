---
kb_id: KB-0005
title: Particles — Grammatical & Functional Words
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0005)
  - SPEC-0001-C3: Compilation Pipeline (MOD-03, MOD-04 — Fast Path)
  - SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-03 LexicalAnalysis)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-2, IR-4)
  - SPEC-0001-C6: Deployment & Runtime Considerations (KB Bundling)
  - SPEC-0001-C8: Security, Validation & Error Handling (KB Integrity)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - SPEC-0401: Knowledge Graph Engine
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---

# KB-0005: Particles — Grammatical & Functional Words

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Particles in Arabic Grammar](#2-particles-in-arabic-grammar)
3. [Data Model](#3-data-model)
4. [Particle Entry Schema](#4-particle-entry-schema)
5. [Prepositions (Huruf al-Jarr)](#5-prepositions-huruf-al-jarr)
6. [Conjunctions (Huruf al-ʿAṭf)](#6-conjunctions-huruf-al-atf)
7. [Subjunctive Particles (Huruf al-Naṣb)](#7-subjunctive-particles-huruf-al-nasb)
8. [Jussive Particles (Huruf al-Jazm)](#8-jussive-particles-huruf-al-jazm)
9. [Conditional Particles (Huruf al-Sharṭ)](#9-conditional-particles-huruf-al-shart)
10. [Interrogative Particles (Huruf al-Istifhām)](#10-interrogative-particles-huruf-al-istifham)
11. [Negative Particles (Huruf al-Nafy)](#11-negative-particles-huruf-al-nafy)
12. [Vocative Particles (Huruf al-Nidāʾ)](#12-vocative-particles-huruf-al-nida)
13. [Inna & Sisters (Inna wa Akhawātuhā)](#13-inna--sisters-inna-wa-akhawatiha)
14. [Kāna & Sisters (Kāna wa Akhawātuhā)](#14-kana--sisters-kana-wa-akhawatiha)
15. [Other Functional Particles](#15-other-functional-particles)
16. [Particle Ambiguity Resolution](#16-particle-ambiguity-resolution)
17. [Particle Matching Algorithm](#17-particle-matching-algorithm)
18. [Serialization & Storage](#18-serialization--storage)
19. [Versioning & Evolution](#19-versioning--evolution)
20. [Quality Requirements](#20-quality-requirements)
21. [Example Entries](#21-example-entries)
22. [Cross-References](#22-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0005 is the **authoritative register of Arabic particles** (حروف المعاني, `ḥurūf al-maʿānī`) used by the AGOS platform. Particles are the grammatical and functional words that govern grammatical relationships, express logical connections, and carry no lexical root derivation.

KB-0005 is critical to the AGOS pipeline because particles are checked **first** (before root extraction) in the **fast path** (MOD-04 Step 3.1). This short-circuits morphological analysis for particles, which cannot be analyzed by root extraction or wazan matching.

KB-0005 answers: **\"Is this token a particle? What is its grammatical function? How does it govern the surrounding context?\"**

### 1.2 Scope

KB-0005 covers:

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Particle types** | All 15+ functional categories of Arabic particles | Verb-based operators (covered by KB-0001/0003) |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal particles (covered by KB plugins) |
| **Inflection** | Particles are invariable (مبني) — store their fixed vocalization | Case endings; particles have no declension |
| **Grammatical effect** | How each particle governs surrounding words (case/mood) | Semantic analysis (covered by MOD-05/07) |
| **Multi-word particles** | Common compound particles (e.g., مِنْ بَعْدُ) | Full idiomatic expressions (covered by lexicons) |

### 1.3 Target Audience

- **AGOS Pipeline:** MOD-04 (MorphologicalParser) reads KB-0005 during the fast-path particle check. MOD-05 (SyntacticParser) reads KB-0005 for particle governance rules. MOD-11 (ExplanationEngine) uses KB-0005 for particle explanations.
- **Linguists & Data Maintainers:** Edit and extend KB-0005 with additional particles or grammatical notes.
- **Plugin Authors:** KB-0005 serves as the base particle set that dialectal or domain-specific particle plugins can extend.

### 1.4 Role in the AGOS Pipeline

```diff
  Text Input
    │
    ▼
  MOD-03: Preprocessor
    │
    ▼
  MOD-04: MorphologicalParser
    │
    ├── Step 3.1: Fast Path
    │   ├── Check if token ∈ KB-0005 (Particles)  ◄── THIS KB
    │   ├── Check if token ∈ KB-0006 (Pronouns)
    │   └── If match → skip root extraction entirely!
    │
    ├── Step 3.3+: Root extraction & wazan matching
    │   (only for non-particle, non-pronoun tokens)
    │
    ▼
  MOD-05: SyntacticParser (uses particle governance rules)
```

### 1.5 Relationship to Other KBs

```diff
  KB-0005: Particles                   ◄── This document (Fixed-function words)
    │
    ├── Independent of root derivation (no root extraction)
    ├── Checked FIRST in MOD-04 fast path
    ├── Provides grammatical governance rules for MOD-05
    │
    ├──► KB-0006: Pronouns              — Also fast-path, also invariable
    ├──► KB-0007: Morphological Feat.   — Feature taxonomy for particle features
    └──► KB-0001: Roots                — Checked only after particle/pronoun fast path
```

KB-0005 is **independent** from KB-0001 (Roots), KB-0002 (Wazan), KB-0003 (Verb Forms), and KB-0004 (Noun Patterns) because particles have no morphological derivation. It has the simplest data model of all KBs.

---

## 2. Particles in Arabic Grammar

### 2.1 Definition

A particle (حرف, `ḥarf` — pl. حروف, `ḥurūf`) in Arabic is a word that:

1. Has **no lexical root** — it cannot be analyzed by root extraction or pattern matching.
2. Is **invariable** (مبني, `mabnī`) — it has a fixed vocalization and no case/number/gender inflection.
3. Expresses **grammatical meaning** — relationships between words, logical connections, mood, negation, interrogation, etc.
4. Is a **closed class** — the set of Arabic particles is finite and well-cataloged (~150–250 entries depending on classification granularity).

### 2.2 Particle vs. Noun vs. Verb Distinction

| Feature | Particle (حرف) | Noun (اسم) | Verb (فعل) |
|---------|---------------|------------|------------|
| **Root derivation** | None | Yes (usually) | Yes |
| **Inflection** | Invariable | Case, number, gender | Tense, mood, person |
| **Closed class?** | Yes (~200) | No (open) | No (open) |
| **Can govern?** | Yes (case/mood) | Yes (iḍāfa) | Yes (object) |
| **Fast-path in MOD-04?** | Yes (checked 1st) | No | No |
| **Can carry clitics?** | Some can (e.g., لِ + تَ = لِتَ) | Yes | Yes |

### 2.3 Hybrid Cases: Multi-category Words

Some Arabic words function as **both particles and nouns/verbs** depending on context. KB-0005 handles these through **homograph entries**:

| Word | As Particle | As Verb/Noun |
|------|------------|--------------|
| **عَلَى** | Preposition \"on\" | Noun: elevation |
| **حَتَّى** | Preposition/conjunction \"until\" | Noun: purpose |
| **ثُمَّ** | Conjunction \"then\" | — (particle only) |
| **لَوْ** | Conditional \"if\" | — (particle only) |
| **لَا** | Negative \"not\" | Noun: \"no\" (in counting) |

Each multi-category word has multiple entries disambiguated by context in MOD-05.

### 2.4 Particle Taxonomy

```diff
  Arabic Particles (حروف المعاني)
  │
  ├── 1. PREPOSITIONS (حروف الجر) — Govern the genitive case
  │     ├── Primary: بِ, تَ, كَ, لِ, مِنْ, عَنْ, عَلَى, إِلَى, فِي, حَتَّى, ...
  │     ├── Redundant/Emphatic: بِ الزائدة
  │     └── Compound: مِنْ بَعْدُ, مِنْ قَبْلُ, ...
  │
  ├── 2. CONJUNCTIONS (حروف العطف) — Coordinate or subordinate
  │     ├── Coordinating: وَ, فَ, ثُمَّ, أَوْ, أَمْ, لَكِنْ, لَا, بَلْ, حَتَّى
  │     └── Subordinating: أَنْ, إِنْ, لَوْ, لَوْلَا, لَوْمَا
  │
  ├── 3. SUBJUNCTIVE PARTICLES (حروف النصب) — Govern the subjunctive mood
  │     ├── أَنْ, لَنْ, إِذَنْ, كَيْ, لِ, حَتَّى
  │     └── After these: imperfect verb takes subjunctive (-a)
  │
  ├── 4. JUSSIVE PARTICLES (حروف الجزم) — Govern the jussive mood
  │     ├── لَمْ, لَمَّا, لَامُ الْأَمْرِ, لَا النَّاهِيَة
  │     └── After these: imperfect verb takes jussive (-∅)
  │
  ├── 5. CONDITIONAL PARTICLES (حروف الشرط)
  │     ├── إِنْ, إِذَا, لَوْ, لَوْلَا, لَوْمَا
  │     └── Grammatical: condition + result structure
  │
  ├── 6. INTERROGATIVES (حروف الاستفهام)
  │     ├── هَلْ, أَ (hamzat al-istifhām), أَمْ
  │     └── Mark yes/no and alternative questions
  │
  ├── 7. NEGATIVES (حروف النفي)
  │     ├── لَا, مَا, لَمْ, لَنْ, لَيْسَ, إِنْ
  │     └── Each has different scope and grammatical effect
  │
  ├── 8. VOCATIVES (حروف النداء)
  │     ├── يَا, أَ, أَيْ, أَيُّهَا, أَيَّتُهَا
  │     └── Precede the noun being called
  │
  ├── 9. INNA & SISTERS (إن وأخواتها)
  │     ├── إِنَّ, أَنَّ, لَكِنَّ, كَأَنَّ, لَيْتَ, لَعَلَّ
  │     └── Govern accusative case on subject (nasb of ism inna)
  │
  ├── 10. KĀNA & SISTERS (كان وأخواتها)
  │     ├── كَانَ, صَارَ, لَيْسَ, أَصْبَحَ, أَمْسَى, ظَلَّ, مَا زَالَ, ...
  │     └── Semi-particles (defective verbs) governing nominative/accusative
  │
  ├── 11. ANSWER & EXCEPTION
  │     ├── Answer: نَعَمْ, بَلَى, لَا, أَجَلْ, إِي
  │     ├── Exception: إِلَّا, غَيْرُ (nominal), سِوَى (nominal)
  │     └── Prohibition: لَا النَّاهِيَة
  │
  ├── 12. MASDAR-FORMING (حروف المصدر)
  │     ├── أَنْ, مَا, أَنَّ, لَوْ, كَيْ
  │     └── Create masdar-like clauses (مَصْدَر تَأْوِيلِيّ)
  │
  └── 13. OTHER
        ├── Exhortation/encouragement: لَوْلَا, لَوْمَا, هَلَّا, أَلَا
        ├── Interjection: يَا (for calling), أَيْ, أَلَا (attention)
        ├── Futurity: سَ (prefix), سَوْفَ
        └── Comparison: كَ (as/like), كَأَنَّ (as if)
```

### 2.5 Particle Count Target

| Category | Estimated Count | Notes |
|----------|----------------|-------|
| Prepositions (حروف الجر) | ~20 | 17 primary + compound forms |
| Conjunctions (حروف العطف) | ~12 | Coordinating + subordinating |
| Subjunctive particles | ~6 | أَنْ, لَنْ, كَيْ, etc. |
| Jussive particles | ~5 | لَمْ, لَمَّا, etc. |
| Conditional particles | ~10 | إِنْ, لَوْ, إِذَا, etc. |
| Interrogative particles | ~6 | هَلْ, أَجَلْ, etc. |
| Negative particles | ~8 | لَا, مَا, لَمْ, etc. |
| Vocative particles | ~6 | يَا, أَيْ, etc. |
| Inna & sisters | ~8 | إِنَّ, أَنَّ, etc. |
| Kāna & sisters | ~15 | Also defective verbs |
| Answer particles | ~8 | نَعَمْ, لَا, etc. |
| Masdar-forming | ~6 | أَنْ, مَا, etc. |
| Other | ~10 | Comparison, future, etc. |
| **Total (Version 1.0)** | **~120–200** | Including variants and homographs |

---

## 3. Data Model

### 3.1 Logical Data Model

```yaml
Particles Database (KB-0005)
├── Metadata
│   ├── kb_id: "KB-0005"
│   ├── version: "1.0.0"
│   ├── particle_count: integer
│   ├── particle_types: string[]
│   ├── created_at: timestamp
│   ├── sources: string[]
│   └── checksum_sha256: string
│
└── Particles: ParticleEntry[]
    ├── Prepositions
    ├── Conjunctions
    ├── Subjunctive particles
    ├── Jussive particles
    ├── Conditional particles
    ├── Interrogative particles
    ├── Negative particles
    ├── Vocative particles
    ├── Inna & sisters
    ├── Kāna & sisters
    ├── Answer/Exception particles
    ├── Masdar-forming particles
    └── Other particles
```

### 3.2 Storage Model

KB-0005 is stored in two formats:

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~1–2 MB | Human-readable |
| **Compiled (Hash Index)** | Production pipeline | ~2–5 MB | Memory-mapped O(1) lookup |

The compiled format is the **smallest** of all AGOS KBs because particles are a small, closed class.

### 3.3 Particle vs. Homograph Distinction

A single Arabic orthographic form (e.g., `مَا`) may map to **multiple particle entries**:

```yaml
Homograph: ما
├── Entry 1: Particle | harf nafy | Negative (mā nāfiya)
├── Entry 2: Particle | harf istifham | Interrogative (mā istifhāmiyya)
├── Entry 3: Particle | harf masdariyya | Masdar-forming (mā maṣdariyya)
├── Entry 4: Pronoun | ism mawsul | Relative pronoun (mā mawṣūla)
└── Entry 5: Particle | harf sharṭ | Conditional (mā sharṭiyya) — rare
```

Disambiguation is performed by MOD-05 (SyntacticParser) using context. KB-0005 stores all attested interpretations as distinct entries with disambiguation hints.

---

## 4. Particle Entry Schema

### 4.1 Schema Definition

```yaml
ParticleEntry:
  # --- Identity ---
  id: string                           # "KB-0005:{particle_type}:{particle_text}"
                                       # e.g., "KB-0005:harf_jarr:مِن"
  particle: string                     # The particle in Arabic script, e.g., "مِن"
  transliteration: string              # Latin transliteration, e.g., "min"

  # --- Classification ---
  particle_type: ParticleType          # From the type taxonomy
  sub_type: string | null              # e.g., "primary", "redundant", "compound"
  category: string[]                   # e.g., ["preposition", "genitive-governing"]
  usage_rank: integer                  # 1 (most common) — for frequency ordering

  # --- Orthography ---
  script_forms: ScriptForm[]           # Different orthographic forms (attached/detached)
  attaches_to: "next_word" | "previous_word" | "standalone" | "both"
                                       # e.g., بِ attaches to next word

  # --- Grammatical Effect ---
  governs_case: "nominative" | "accusative" | "genitive" | "jussive" | "subjunctive" | null
  governs_mood: "indicative" | "subjunctive" | "jussive" | null
  government_type: "independent" | "requires_sister" | "requires_complement"
  grammatical_function: string         # Description of what this particle does

  # --- Semantic/Syntactic Properties ---
  meaning: string                      # English meaning
  meaning_ar: string                   # Arabic explanation
  usage_notes: string | null           # Usage constraints, exceptions

  # --- Examples ---
  examples: Example[]

  # --- Homograph Disambiguation ---
  homograph_group: string | null       # Group ID for homographs (e.g., "ما_group")
  disambiguation_hints: string[]       # Contextual clues for disambiguation

  # --- Attestation ---
  attestation: Attestation

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string
```

### 4.2 Supporting Types

```yaml
ParticleType:
  "harf_jarr" | "harf_atf" | "harf_nasb" | "harf_jazm" |
  "harf_shart" | "harf_istifham" | "harf_nafy" | "harf_nida" |
  "harf_tanbih" | "harf_tahdid" | "harf_tafsil" | "harf_jawab" |
  "harf_masdari" | "harf_tashbih" | "harf_istithna" |
  "harf_tawqit" | "harf_rad" | "harf_zarf" | "semi_verb" |
  "harf_maani" | "harf_nasikh" | "mushtaqq" | "other"

ScriptForm:
  form: string                         # The orthographic form
  context: string                      # When this form is used
  example: string | null               # Example usage

Example:
  phrase: string                       # Example phrase or sentence
  transliteration: string              # Latin transliteration
  translation: string                  # English translation
  source: string | null                # Source citation

Attestation:
  confidence: "certain" | "well_attested" | "attested" | "disputed"
  primary_sources: string[]
  classical_references: string[]
  notes: string | null

GovernanceRule:
  governs: string                      # What it governs (case, mood)
  condition: string                    # Under what conditions
  exceptions: string[]                 # Exceptions to this rule
```

### 4.3 JSON Example (Preposition — مِن)

```json
{
  "id": "KB-0005:harf_jarr:مِن",
  "particle": "مِن",
  "transliteration": "min",
  "particle_type": "harf_jarr",
  "sub_type": "primary",
  "category": ["preposition", "genitive-governing", "partitive"],
  "usage_rank": 1,
  "script_forms": [
    { "form": "مِنْ", "context": "standalone (with sukun)", "example": "مِنَ الْبَيْتِ" },
    { "form": "مِنَ", "context": "before alif lam (assimilation)", "example": "مِنَ الْبَيْتِ" }
  ],
  "attaches_to": "next_word",
  "governs_case": "genitive",
  "governs_mood": null,
  "government_type": "independent",
  "grammatical_function": "Initiates a genitive prepositional phrase indicating origin, source, partitivity, or comparison",
  "meaning": "from, of, some of, than (comparative)",
  "meaning_ar": "ابتداء الغاية أو التبعيض أو البيان أو التعليل",
  "usage_notes": "When followed by a definite article, changes to مِنَ (min + al = mina). In comparative constructions (أَفْعَلُ مِنْ), governs the noun of comparison.",
  "examples": [
    { "phrase": "مِنَ الْبَيْتِ", "transliteration": "mina l-bayti", "translation": "from the house", "source": null },
    { "phrase": "خَيْرٌ مِنْهُ", "transliteration": "khayrun minhu", "translation": "better than him", "source": null },
    { "phrase": "مِنَ الْمُؤْمِنِينَ", "transliteration": "mina l-muʾminīna", "translation": "some of the believers", "source": "Quran 9:99" }
  ],
  "homograph_group": null,
  "disambiguation_hints": [],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": ["Al-Kitab", "Mughni al-Labib", "Sharh al-Ashmuni"]
  },
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

---

## 5. Prepositions (Huruf al-Jarr)

Prepositions (حروف الجر, `ḥurūf al-jarr`) govern the **genitive case** on the nouns that follow them. They are the largest category of particles.

### 5.1 Primary Prepositions

These are the 17 foundational prepositions of Arabic:

| # | Particle | Translit. | Meaning | Notes |
|---|----------|-----------|---------|-------|
| 1 | **بِ** | bi | by, with, in, at | Attaches directly; vowel i under ب |
| 2 | **تَ** | ta | by (oath) | Archaic; used only in oaths |
| 3 | **لِ** | li | for, to, belonging to | Attaches directly |
| 4 | **كَ** | ka | like, as | Attaches directly |
| 5 | **مِنْ** | min | from, of, some of | See Section 4.3 example |
| 6 | **إِلَى** | ilā | to, toward, until | Usually written with alif maqsura |
| 7 | **عَنْ** | ʿan | from, about, away from | With sukun on ن |
| 8 | **عَلَى** | ʿalā | on, upon, over | With alif maqsura |
| 9 | **فِي** | fī | in, within, among | Written with ي |
| 10 | **مَعَ** | maʿa | with, together with | Nominal origin (ظرف) but functions as prep |
| 11 | **حَتَّى** | ḥattā | until, up to, even | Also conjunction; governs genitive only sometimes |
| 12 | **رُبَّ** | rubba | perhaps, many a | Rare; introduces indefinite nouns |
| 13 | **مُنْذُ** | mundhu | since, ago | Time reference; governs genitive or nominative |
| 14 | **مُذْ** | mudh | since, ago | Shortened form of مُنْذُ |
| 15 | **و** | wa | by (oath) | Not to be confused with و conjunction |
| 16 | **خَلَا** | khalā | except, excluding | Governs genitive or accusative |
| 17 | **عَدَا** | ʿadā | except, excluding | Governs genitive or accusative |

### 5.2 Preposition Grammar Rules

| Property | Rule |
|----------|------|
| **Effect** | Noun following a preposition takes genitive case (الْجَرّ) |
| **Attached form** | بِ, تَ, لِ, كَ, و (oath) attach as prefixes to the next word |
| **Detached form** | All other prepositions are written as separate words |
| **Before ال** | Some prepositions change: مِنْ + ال = مِنَ; عَنْ + ال = عَنِ; etc. |
| **Suffix pronouns** | Prepositions can take pronominal suffixes: بِهِ (bihi) \"with him\" |
| **مِنْ for comparison** | Used after elative (أَفْعَل مِنْ) for the standard of comparison |

### 5.3 Compound Prepositions

Some prepositions are formed by combining a primary preposition with a noun:

| Compound | Components | Meaning | Example |
|----------|-----------|---------|---------|
| **مِنْ بَعْدُ** | مِن + بَعْد | after | مِنْ بَعْدِ الْحَرْبِ |
| **مِنْ قَبْلُ** | مِن + قَبْل | before | مِنْ قَبْلِ الْيَوْمِ |
| **مِنْ دُونِ** | مِن + دُون | without, besides | مِنْ دُونِ شَكٍّ |
| **فَوْقَ** | (noun) | above | فَوْقَ الْبَيْتِ |
| **تَحْتَ** | (noun) | under | تَحْتَ الْكُرْسِيِّ |
| **أَمَامَ** | (noun) | in front of | أَمَامَ الْمَسْجِدِ |
| **وَرَاءَ** | (noun) | behind | وَرَاءَ الْبَابِ |
| **بَيْنَ** | (noun) | between | بَيْنَ الْيَدَيْنِ |
| **حَوْلَ** | (noun) | around | حَوْلَ الْمَدِينَةِ |
| **ضِدَّ** | (noun) | against | ضِدَّ الْعَدُوِّ |

Note: These compound prepositions are grammatically **nouns** (ظروف مكان/زمان) that function as prepositions. They are included in KB-0005 for completeness with their particle-like behavior documented.

### 5.4 Redundant/Emphatic Prepositions

| Particle | Usage | Example |
|----------|-------|---------|
| **بِ الزائدة** | Redundant ب — adds emphasis, no grammatical effect | كَفَى بِاللَّهِ (enough with God) |

---

## 6. Conjunctions (Huruf al-ʿAṭf)

Conjunctions (حروف العطف, `ḥurūf al-ʿaṭf`) connect words or phrases. In Arabic grammar, they coordinate nouns, verbs, or entire clauses.

### 6.1 Coordinating Conjunctions

| # | Particle | Translit. | Meaning | Function |
|---|----------|-----------|---------|----------|
| 1 | **وَ** | wa | and | Simple conjunction |
| 2 | **فَ** | fa | and so, then | Sequential/consequential |
| 3 | **ثُمَّ** | thumma | then, moreover | Sequential with delay |
| 4 | **أَوْ** | aw | or | Alternative |
| 5 | **أَمْ** | am | or (in questions) | Alternative question |
| 6 | **لَكِنْ** | lākin | but, however | Adversative |
| 7 | **لَا** | lā | not (disjunctive) | Negation in coordination |
| 8 | **بَلْ** | bal | but rather, nay | Correction/addition |
| 9 | **حَتَّى** | ḥattā | even, including | Inclusive |
| 10 | **إِمَّا** | immā | either...or | Correlative |

### 6.2 Conjunction Grammar

| Property | Rule |
|----------|------|
| **Effect on case** | The conjoined noun takes the same case as the noun before the conjunction |
| **و vs. ف** | و indicates simple addition; ف indicates sequence or consequence |
| **أَو vs. أَم** | أَو is used in statements; أَم is used in questions (alternative) |
| **بَلْ** | Indicates correction of a prior statement (بَلْ) or addition (بَلْ) |

### 6.3 Subordinating Conjunctions

| Particle | Translit. | Meaning | Grammatical Effect |
|----------|-----------|---------|-------------------|
| **أَنْ** | an | that | Governs subjunctive on following verb |
| **لَوْ** | law | if (hypothetical) | No case/mood effect on verb |
| **لَوْلَا** | lawlā | if not, were it not for | Introduces conditional clause |
| **لَوْمَا** | lawmā | if not | Rare variant of لَوْلَا |
| **كَيْ** | kay | in order to | Governs subjunctive on following verb |

---

## 7. Subjunctive Particles (Huruf al-Naṣb)

Subjunctive particles (حروف النصب, `ḥurūf al-naṣb`) govern the **subjunctive mood** (النصب) on the imperfect verb that follows them.

### 7.1 Subjunctive Particle Table

| # | Particle | Translit. | Meaning | Verb Form After | Notes |
|---|----------|-----------|---------|-----------------|-------|
| 1 | **أَنْ** | an | that, to | يَفْعَلَ (subjunctive) | Most common; masdar-forming |
| 2 | **لَنْ** | lan | will not (emphatic) | يَفْعَلَ (subjunctive) | Definite future negation |
| 3 | **إِذَنْ** | idhan | then, in that case | يَفْعَلَ (subjunctive) | Response particle |
| 4 | **كَيْ** | kay | in order to, so that | يَفْعَلَ (subjunctive) | Purpose |
| 5 | **لِ** | li | in order to, so that | يَفْعَلَ (subjunctive) | Purpose (attached form of كَيْ) |
| 6 | **حَتَّى** | ḥattā | so that, until | يَفْعَلَ (subjunctive) | Purpose or limit |

### 7.2 Subjunctive Mood Transformation

```diff
  Before subjunctive particle:
    يَكْتُبُ (yaktubu) — indicative (rafʿ)

  After subjunctive particle:
    أَنْ يَكْتُبَ (an yaktuba) — subjunctive (naṣb)
    لَنْ يَكْتُبَ (lan yaktuba) — will not write
```

### 7.3 Special: أَنْ vs. إِنْ Distinction

| Particle | Function | Notes |
|----------|----------|-------|
| **أَنْ** (with fatḥa) | Masdar-forming, subjunctive-governing | written with full alif |
| **إِنْ** (with kasra) | Conditional \"if\" | jussive-governing, written with broken alif |

These are homographs only in pronunciation; in writing they are distinguished by the alif form. KB-0005 stores separate entries.

---

## 8. Jussive Particles (Huruf al-Jazm)

Jussive particles (حروف الجزم, `ḥurūf al-jazm`) govern the **jussive mood** (الجزم) on the imperfect verb that follows them.

### 8.1 Jussive Particle Table

| # | Particle | Translit. | Meaning | Verb Form After | Notes |
|---|----------|-----------|---------|-----------------|-------|
| 1 | **لَمْ** | lam | not (past negation) | يَفْعَلْ (jussive) | Negates past; most common |
| 2 | **لَمَّا** | lammā | not yet, not until | يَفْعَلْ (jussive) | Implies expectation |
| 3 | **لَامُ الْأَمْرِ** | lāmu l-amr | let...! (imperative) | يَفْعَلْ (jussive) | Commands 3rd person |
| 4 | **لَا النَّاهِيَة** | lā n-nāhiya | don't! (prohibition) | يَفْعَلْ (jussive) | Negative imperative |
| 5 | **إِنْ** | in | if (conditional) | يَفْعَلْ (jussive) | Conditional jussive |

### 8.2 Jussive Mood Transformation

```diff
  Before jussive particle:
    يَكْتُبُ (yaktubu) — indicative (rafʿ)

  After jussive particle:
    لَمْ يَكْتُبْ (lam yaktub) — jussive (jazm) with sukun
    لَمَّا يَكْتُبْ (lammā yaktub) — not yet written
    لِيَكْتُبْ (li-yaktub) — let him write
```

### 8.3 لَمْ vs. مَا for Past Negation

| Particle | Effect | Example |
|----------|--------|---------|
| **لَمْ** | Jussive; implies denial of completed action | لَمْ يَكْتُبْ (he did not write) |
| **مَا** | Indicative; factual negation of past | مَا كَتَبَ (he did not write) |

---

## 9. Conditional Particles (Huruf al-Sharṭ)

Conditional particles (حروف الشرط, `ḥurūf al-sharṭ`) introduce **conditional sentences** consisting of a condition (شَرْط) and a result (جَوَاب).

### 9.1 Conditional Particle Table

| # | Particle | Translit. | Meaning | Verb Mood | Type |
|---|----------|-----------|---------|-----------|------|
| 1 | **إِنْ** | in | if | Jussive (both verbs) | Real/general |
| 2 | **لَوْ** | law | if (hypothetical) | Past tense | Unreal |
| 3 | **لَوْلَا** | lawlā | if not, were it not for | Past tense | Unreal |
| 4 | **لَوْمَا** | lawmā | if not | Past tense | Unreal (rare) |
| 5 | **إِذَا** | idhā | when, if (definite) | Perfect or imperfect | Temporal/real |
| 6 | **مَنْ** | man | whoever (conditional) | Jussive | Conditional pronoun |
| 7 | **مَا** | mā | whatever (conditional) | Jussive | Conditional pronoun |
| 8 | **مَهْمَا** | mahnā | whatever, whenever | Jussive | Conditional pronoun |
| 9 | **أَيّانَ** | ayyāna | whenever | Jussive | Time conditional |
| 10 | **أَيْنَ** | ayna | wherever | Jussive | Place conditional |
| 11 | **أَنَّى** | annā | wherever, however | Jussive | Manner/place conditional |
| 12 | **حَيْثُمَا** | ḥaythumā | wherever | Jussive | Place conditional |

### 9.2 Conditional Structure

```diff
  Conditional sentence structure:

  إِنْ + Condition (فِعْلُ الشَّرْطِ) + Result (جَوَابُ الشَّرْطِ)

  Example:
  إِنْ تَكْتُبْ تَنْجَحْ
  (in taktub tanjaḥ)
  \"If you write, you succeed\"

  Both verbs are in the jussive mood (جزم).
```

### 9.3 إِنْ vs. لَوْ Distinction

| Aspect | إِنْ (in) | لَوْ (law) |
|--------|----------|------------|
| **Type** | Real/general condition | Unreal/hypothetical |
| **Likelihood** | Possible, general | Impossible or unlikely |
| **Verb mood** | Jussive | Past tense |
| **Example** | إِنْ تَدْرُسْ تَنْجَحْ (if you study, you succeed) | لَوْ دَرَسْتَ نَجَحْتَ (if you had studied, you would have succeeded) |

---

## 10. Interrogative Particles (Huruf al-Istifhām)

Interrogative particles (حروف الاستفهام, `ḥurūf al-istifhām`) mark **questions**.

### 10.1 Interrogative Particle Table

| # | Particle | Translit. | Meaning | Type | Notes |
|---|----------|-----------|---------|------|-------|
| 1 | **هَلْ** | hal | is/are/do? | Yes/no question | Standard |
| 2 | **أَ** | a- | is/are/do? | Yes/no question | Attaches as prefix (hamzat al-istifhām) |
| 3 | **أَمْ** | am | or? | Alternative question | Follows a question |
| 4 | **أَجَلْ** | ajal | indeed? | Confirmatory | Rare as interrogative |
| 5 | **أَلَا** | alā | is it not that? | Rhetorical | Expects affirmative answer |
| 6 | **أَلَيْسَ** | alaysa | is it not? | Rhetorical | Negated question |

### 10.2 Interrogative vs. Other Functions

The prefix **أَ** (hamzat al-istifhām) is homographic with other hamza prefixes:

| Function | Example | Meaning |
|----------|---------|---------|
| Interrogative | أَكَتَبْتَ؟ (a-katabta?) | Did you write? |
| Vocative | أَخَالِدُ (a-khālidu) | O Khalid! |
| First person | أَكْتُبُ (aktubu) | I write (verb prefix) |

Disambiguation: MOD-05 checks whether the hamza is followed by a noun (vocative or interrogative) or a verb (interrogative or 1st person). Context determines the function.

### 10.3 هَلْ vs. أَ Distinction

| Aspect | هَلْ (hal) | أَ (a-) |
|--------|-----------|---------|
| **Form** | Separate word | Prefix (attaches to next word) |
| **Scope** | Questions only | Questions, vocative, more |
| **Negation after** | هَلْ + verb | أَ + لَمْ / أَ + مَا |
| **Example** | هَلْ كَتَبْتَ؟ | أَكَتَبْتَ؟ |

---

## 11. Negative Particles (Huruf al-Nafy)

Negative particles (حروف النفي, `ḥurūf al-nafy`) express negation with different **temporal scopes** and **grammatical effects**.

### 11.1 Negative Particle Table

| # | Particle | Translit. | Meaning | Tense | Effect | Notes |
|---|----------|-----------|---------|-------|--------|-------|
| 1 | **لَا** | lā | not | Present/general | Indicative | General negation |
| 2 | **مَا** | mā | not | Past | Perfect/indicative | Common classical negation |
| 3 | **لَمْ** | lam | did not | Past | Jussive | See Section 8 |
| 4 | **لَنْ** | lan | will not | Future | Subjunctive | See Section 7 |
| 5 | **لَيْسَ** | laysa | is not | Present | Kāna-type | Semi-verb, agrees in person |
| 6 | **إِنْ** | in | not (emphatic) | Present | — | Emphatic negation (rare) |
| 7 | **لَاتَ** | lāta | not | Past (time) | — | Very rare; used with time words |
| 8 | **غَيْرُ** | ghayru | other than, not | Nominal | Genitive | Noun, not truly a particle |

### 11.2 Negation Scope & Effect

| Particle | Scope | Verbal Effect | Nominal Effect |
|----------|-------|---------------|----------------|
| **لَا** | Verb negation | No change (indicative) | لَا النَّافِيَة لِلْجِنْس (generic no) |
| **مَا** | Verb negation | Perfect verb, no change | — |
| **لَيْسَ** | Copular negation | Agrees in person/gender | Nominative subject, accusative predicate |
| **لَمْ** | Past negation | Jussive | — |
| **لَنْ** | Future negation | Subjunctive | — |

### 11.3 لَا al-Nāfiya vs. لَا al-Nāhiya

| Variant | Function | Grammatical Effect | Example |
|---------|----------|-------------------|---------|
| **لَا النَّافِيَة** | Negative (indicative) | No mood change | لَا يَكْتُبُ (he does not write) |
| **لَا النَّاهِيَة** | Prohibition (jussive) | Jussive mood | لَا تَكْتُبْ (don't write!) |
| **لَا النَّافِيَة لِلْجِنْس** | Generic negation | Accusative noun | لَا إِلَهَ إِلَّا اللَّهُ |

---

## 12. Vocative Particles (Huruf al-Nidāʾ)

Vocative particles (حروف النداء, `ḥurūf al-nidāʾ`) introduce the **person or thing being addressed**.

### 12.1 Vocative Particle Table

| # | Particle | Translit. | Meaning | Noun Case | Notes |
|---|----------|-----------|---------|-----------|-------|
| 1 | **يَا** | yā | O! | Nominative (definite) | Most common; universal |
| 2 | **أَ** | a- | O! | Nominative | For close distance |
| 3 | **أَيْ** | ay | O! | Nominative | For close distance |
| 4 | **أَيُّهَا** | ayyuhā | O! (masculine) | Nominative | Followed by definite noun |
| 5 | **أَيَّتُهَا** | ayyatuhā | O! (feminine) | Nominative | Followed by definite noun |
| 6 | **يَا أَيُّهَا** | yā ayyuhā | O! | Nominative | Combined form |

### 12.2 Vocative Grammar

```diff
  Vocative structure:

  يَا + Noun (nominative, definite or construct)

  Examples:
  يَا مُحَمَّدُ           — O Muhammad (proper noun → nominative, damma)
  يَا رَجُلُ             — O man (indefinite intended → nominative)
  يَا عَبْدَ اللَّهِ     — O servant of God (construct → accusative!)
```

Note: The noun after a vocative takes **nominative** if it is a proper noun or definite. It takes **accusative** if it is a construct (مُضَاف).

---

## 13. Inna & Sisters (Inna wa Akhawātuhā)

Inna and its sisters (إِنَّ وَأَخَوَاتُها) are particles that govern the **accusative case** on the **subject** of a nominal sentence.

### 13.1 Complete Table

| # | Particle | Translit. | Meaning | Notes |
|---|----------|-----------|---------|-------|
| 1 | **إِنَّ** | inna | indeed, verily | Emphasis/affirmation |
| 2 | **أَنَّ** | anna | that (certainty) | Masdar-forming (certain) |
| 3 | **لَكِنَّ** | lākinna | but, however | Adversative; differs from لٰكِنْ |
| 4 | **كَأَنَّ** | ka-anna | as if, as though | Comparison/semblance |
| 5 | **لَيْتَ** | layta | if only, would that | Wish/hope |
| 6 | **لَعَلَّ** | laʿalla | perhaps, maybe | Expectation/hope |
| 7 | **لَ** | la- (prefix) | indeed, surely | Emphatic prefix |

### 13.2 Inna Grammar

```diff
  Without Inna:
    الْمُعَلِّمُ جَالِسٌ
    al-muʿallimu jālisun
    \"The teacher is sitting\"
    (Subject: nominative, Predicate: nominative)

  With Inna:
    إِنَّ الْمُعَلِّمَ جَالِسٌ
    inna l-muʿallima jālisun
    \"Indeed, the teacher is sitting\"
    (Ism inna: accusative! Khabar inna: nominative)
```

### 13.3 Inna vs. Anna Distinction

| Particle | Function | Used After |
|----------|----------|------------|
| **إِنَّ** (with kasra) | Independent assertion | Beginning of speech, after القَوْل |
| **أَنَّ** (with fatḥa) | Subordinate clause | After verbs of knowing, thinking, saying |

---

## 14. Kāna & Sisters (Kāna wa Akhawātuhā)

Kāna and its sisters (كَانَ وَأَخَوَاتُها) are **defective verbs** (أَفْعَال نَاقِصَة) that govern the **nominative** case on the subject and **accusative** on the predicate. While technically verbs, they function as grammatical operators and are included in KB-0005 as **semi-particles** for completeness.

### 14.1 Complete Table

| # | Particle | Translit. | Meaning | Type |
|---|----------|-----------|---------|------|
| 1 | **كَانَ** | kāna | was, existed | Existence/time |
| 2 | **لَيْسَ** | laysa | is not | Negation |
| 3 | **صَارَ** | ṣāra | became | Transformation |
| 4 | **أَصْبَحَ** | aṣbaḥa | became, entered morning | Time (morning) |
| 5 | **أَمْسَى** | amsā | became, entered evening | Time (evening) |
| 6 | **أَضْحَى** | aḍḥā | became, entered forenoon | Time (forenoon) |
| 7 | **ظَلَّ** | ẓalla | continued, remained | Continuation |
| 8 | **بَاتَ** | bāta | spent the night | Time (night) |
| 9 | **مَا زَالَ** | mā zāla | still, continues | Continuation (negated) |
| 10 | **مَا انْفَكَّ** | mā nfakka | still, continues | Continuation (negated) |
| 11 | **مَا فَتِئَ** | mā fatiʾa | still, continues | Continuation (negated) |
| 12 | **مَا بَرِحَ** | mā bariḥa | still, continues | Continuation (negated) |
| 13 | **مَا دَامَ** | mā dāma | as long as | Duration |
| 14 | **لَعَلَّ** | laʿalla | perhaps | (Also in inna-group, some grammarians place here for meccan dialect) |

### 14.2 Kāna Grammar

```diff
  Without Kāna:
    الْجَوُّ جَمِيلٌ
    al-jawwu jamīlun
    \"The weather is beautiful\"

  With Kāna:
    كَانَ الْجَوُّ جَمِيلًا
    kāna l-jawwu jamīlan
    \"The weather was beautiful\"
    (Subject: nominative, Predicate: accusative!)
```

### 14.3 ما دامَ Usage

ما دامَ (mā dāma) has a special meaning \"as long as\" and is unique in that the entire clause after it is in the accusative:

```diff
  أَعْبُدُ اللَّهَ مَا دُمْتُ حَيًّا
  aʿbudu llāha mā dumtu ḥayyan
  \"I worship God as long as I live\"
```

---

## 15. Other Functional Particles

### 15.1 Answer Particles (حروف الجواب)

| Particle | Translit. | Meaning | Usage |
|----------|-----------|---------|-------|
| **نَعَمْ** | naʿam | yes | General affirmative answer |
| **بَلَى** | balā | yes (contradicting negation) | Affirms after a negative question |
| **أَجَلْ** | ajal | yes, indeed | Emphatic affirmative |
| **إِي** | ī | yes, indeed | Oath/swear response |
| **لَا** | lā | no | General negative answer |
| **جَزَاكَ اللَّهُ** | jazāka llāh | thank you | Idiomatic (not a particle) |

### 15.2 Exception Particles (حروف الاستثناء)

| Particle | Translit. | Meaning | Case Effect | Example |
|----------|-----------|---------|-------------|---------|
| **إِلَّا** | illā | except, but | Accusative (or following the excepted noun's case) | جَاءَ الطُّلَّابُ إِلَّا زَيْدًا (the students came except Zayd) |
| **غَيْرُ** | ghayru | other than | Genitive (as a noun in construct) | جَاءَ غَيْرُ زَيْدٍ (other than Zayd came) |
| **سِوَى** | siwā | besides, except | Genitive | جَاءَ سِوَى زَيْدٍ |

### 15.3 Exhortation/Encouragement Particles

| Particle | Translit. | Meaning | Notes |
|----------|-----------|---------|-------|
| **لَوْلَا** | lawlā | if only, why not? | Encouragement to do something |
| **هَلَّا** | hallā | why not? | Encouragement |
| **أَلَا** | alā | will you not? | Rhetorical encouragement |
| **لَوْمَا** | lawmā | if only | Rare |

### 15.4 Masdar-Forming Particles (حروف المصدر)

These particles create a **masdar ta'wili** (interpretive verbal noun) clause:

| Particle | Translit. | Meaning | Verb Mood | Example |
|----------|-----------|---------|-----------|---------|
| **أَنْ** | an | that, to | Subjunctive | أَنْ تَكْتُبَ (that you write) |
| **مَا** | mā | that | Perfect/indicative | مَا تَفْعَلُ (what you do) |
| **أَنَّ** | anna | that (assumed) | Nominal clause | أَنَّكَ كَتَبْتَ (that you wrote) |
| **لَوْ** | law | that | Hypothetical | لَوْ كَتَبْتَ (that you would write) |

### 15.5 Future Particles

| Particle | Translit. | Meaning | Usage |
|----------|-----------|---------|-------|
| **سَ** | sa- | will (near future) | Prefix; attaches to imperfect verb |
| **سَوْفَ** | sawfa | will (distant future) | Separate word before imperfect verb |

```diff
  سَأَكْتُبُ (sa-aktubu) — I will write (soon/soon-ish)
  سَوْفَ أَكْتُبُ (sawfa aktubu) — I will write (eventually)
```

### 15.6 Prohibition: لَا النَّاهِيَة

```diff
  لَا النَّاهِيَة + Imperfect verb (jussive) = Prohibition

  لَا تَكْتُبْ (lā taktub) — Don't write!
  لَا تَقُلْ (lā taqul) — Don't say!
```

### 15.7 Negation of Generic: لَا النَّافِيَة لِلْجِنْس

```diff
  لَا + Indefinite noun (accusative) = Negation of an entire class

  لَا إِلَهَ إِلَّا اللَّهُ
  lā ilāha illā llāh
  \"There is no god but God\"
  (Noun \"ilāha\" is in accusative, not nominative!)
```

---

## 16. Particle Ambiguity Resolution

### 16.1 Common Homographs

Many Arabic particles are **homographic** — the same written form maps to multiple particle functions:

| Form | Possible Functions | Disambiguation |
|------|-------------------|----------------|
| **مَا** | Negative, interrogative, relative pronoun, conditional, masdar-forming, exclamative | Context: verb tense, surrounding particles, clause structure |
| **لَا** | Negation, prohibition, generic negation | Context: verb mood (indicative vs. jussive), noun case |
| **إِنْ** | Conditional \"if\", negative \"not\" | Context: verb mood (jussive vs. indicative) |
| **أَنْ** | Subjunctive-forming, masdar-forming | Context: following verb mood (always subjunctive) |
| **وَ** | Conjunction \"and\", oath \"by\" | Context: semantic; oath with Allah or abstract nouns |
| **فِي** | Preposition \"in\", noun \"mouth\" (فَم with ي suffix) | Context: syntactic position |
| **عَلَى** | Preposition \"on\", noun \"elevation\" | Context: syntactic position |

### 16.2 Disambiguation Algorithm

```pseudo
Algorithm: disambiguate_particle
Input: token (string), context (syntactic_context)
Output: ParticleEntry (most likely interpretation)

1. Look up token in KB-0005 hash index.
2. If only one entry → return that entry.
3. If multiple entries (homograph):
   a. Check verb mood in context:
      - If following verb is jussive → likely جازم (لم, لا ناهية, إن شرطية)
      - If following verb is subjunctive → likely ناصب (أن, لن, كي)
   b. Check noun case in context:
      - If following noun is genitive → likely جار (حرف جر)
      - If following noun is accusative while subject → likely إنّ or sisters
   c. Check negation scope:
      - If clause is question → likely interrogative
      - If clause is conditional → likely conditional
      - If clause is negative → likely negative
   d. Score each interpretation:
      - +3 for exact syntactic match
      - +2 for compatible semantic domain
      - +1 for frequency (usage_rank)
4. Return highest-scoring entry with confidence score.

Note: If multiple interpretations have similar confidence, all are
passed forward as ambiguity candidates. MOD-05 handles the final
disambiguation during syntactic parsing.
```

### 16.3 Ambiguity Handling in the Pipeline

```diff
  MOD-04 (MorphologicalParser):
    Identifies all possible particle interpretations for a token.
    Passes ambiguity set to MOD-05.

  MOD-05 (SyntacticParser):
    Uses clause structure and case/mood marking to resolve ambiguity.
    For example:
    - مَا + perfect verb → likely negative (مَا كَتَبَ = he did not write)
    - مَا + imperfect verb → likely negative or relative
    - مَا + after preposition → likely relative pronoun
    - مَا before a noun → likely interrogative or exclamative
```

---

## 17. Particle Matching Algorithm

### 17.1 Primary Algorithm: Fast Path Particle Check

```pseudo
Algorithm: fast_path_particle_check
Input: token (string), context (syntactic_context)
Output: (boolean, ParticleEntry[])

1. Normalize token:
   a. Remove tatweel/kashida if present.
   b. Apply NFKC Unicode normalization (handles lam-alef ligatures).
   c. Strip any remaining diacritics for lookup (optional, toggleable).
   d. Check for attached clitics (prefixes: wa-, fa-, bi-, li-, ka-, sa-, a-).

2. Hash-index lookup:
   a. Query KB-0005 by particle text.
   b. If not found with diacritics → try without diacritics.
   c. If still not found → return (false, []).

3. If found:
   a. Filter by particle_type if context provides a type hint.
   b. For multiple matches, run disambiguate_particle (Section 16.2).
   c. Record particle features:
      i.   Particle type (harf jarr, nasb, jazm, etc.)
      ii.  Grammatical governance (case/mood effect)
      iii. Anchoring (precedes/follows what)
   d. Return (true, ranked ParticleEntry[]).

4. Performance note:
   a. This function MUST complete in < 1 μs for the common case.
   b. The compiled KB-0005 is designed for O(1) hash lookup.
   c. The fast path is designed to short-circuit ~90% of particle tokens.
```

### 17.2 Secondary Algorithm: Resolve Particle Governance

```pseudo
Algorithm: resolve_particle_governance
Input: particle (ParticleEntry), following_word (TokenInfo),
       context (syntactic_context)
Output: GovernanceEffect

1. Determine governance type:
   a. If particle.governs_case is set:
      - Apply case to the following noun.
      - Update IR-4 token features with case assignment.
   b. If particle.governs_mood is set:
      - Apply mood to the following imperfect verb.
      - Update IR-4 token features with mood assignment.

2. Apply morphological adjustments:
   a. For prepositions with special vowel rules:
      - If word starts with ال and particle == مِنْ → use مِنَ
      - If word starts with ال and particle == عَنْ → use عَنِ
   b. For attached particles (prefixes):
      - Remove the particle from the word form for remainder analysis.
      - Pass the remainder to MOD-04 main path.
   c. For inna/kāna particles:
      - Mark the subject as accusative (for inna) or nominative (for kāna).
      - Mark the predicate as nominative (for inna) or accusative (for kāna).

3. Return GovernanceEffect with:
   a. Particle ID
   b. Case/mood assignment(s)
   c. Confidence score
   d. Any morphological adjustments applied
```

---

## 18. Serialization & Storage

### 18.1 Source Format

```diff
  /knowledge/KB-0005/
  ├── metadata.yaml                     # KB metadata (version, counts)
  ├── prepositions.yaml                 # حروف الجر
  ├── conjunctions.yaml                 # حروف العطف
  ├── subjunctive.yaml                  # حروف النصب
  ├── jussive.yaml                      # حروف الجزم
  ├── conditional.yaml                  # حروف الشرط
  ├── interrogative.yaml                # حروف الاستفهام
  ├── negative.yaml                     # حروف النفي
  ├── vocative.yaml                     # حروف النداء
  ├── inna-sisters.yaml                 # إن وأخواتها
  ├── kana-sisters.yaml                 # كان وأخواتها
  ├── answer-exception.yaml             # الجواب والاستثناء
  ├── masdar-forming.yaml               # حروف المصدر
  └── other.yaml                        # Future, prohibition, comparison, etc.
```

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0005"
title: "Particles — Grammatical & Functional Words"
version: "1.0.0"
status: "draft" | "review" | "published"

particle_count: 172
particle_types: [
  "harf_jarr", "harf_atf", "harf_nasb", "harf_jazm",
  "harf_shart", "harf_istifham", "harf_nafy", "harf_nida",
  "harf_nasikh", "harf_jawab", "harf_masdari", "other"
]

created_at: "2026-07-15T00:00:00Z"
updated_at: "2026-07-15T00:00:00Z"

sources:
  - name: "Sibawayh, Al-Kitab"
    version: "critical_1988"
  - name: "Ibn Hisham, Mughni al-Labib"
    version: "print_1964"
  - name: "Al-Ashmuni, Sharh al-Ashmuni"
    version: "print_1963"
  - name: "Wright's Arabic Grammar"
    version: "3rd_edition"

checksum_sha256: "e5f6a7b8c9d0..."
maintainers:
  - name: "Dr. [Name]"
    email: "[email]"
    role: "particle_editor"
```

### 18.2 Compiled Format (Hash Index)

The compiled format is a **flat hash-indexed array** — the simplest of all AGOS KBs:

```diff
  Compiled Particle Binary:
  ┌──────────────────────────────────────────────────────────────┐
  │ HEADER                                                       │
  │ ├── magic: "AGOSKB05" (8 bytes)                             │
  │ ├── version: major(2B) + minor(2B) + patch(2B)              │
  │ ├── particle_count: u32 (4 bytes)                           │
  │ ├── hash_index_offset: u32 (4 bytes)                        │
  │ ├── entry_table_offset: u32 (4 bytes)                       │
  │ ├── string_table_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ HASH INDEX (O(1) lookup)                                    │
  │ ├── Perfect hash over particle text strings                 │
  │ ├── Maps particle text → particle_id[] (for homographs)    │
  │ └── 256 entry bucket table                                  │
  ├──────────────────────────────────────────────────────────────┤
  │ ENTRY TABLE                                                  │
  │ ├── Fixed-size entries (64 bytes each)                      │
  │ │   ├── particle_type: u8                                   │
  │ │   ├── sub_type: u8                                        │
  │ │   ├── usage_rank: u8                                      │
  │ │   ├── governs_case: u8 (0=none, 1=nom, 2=acc, 3=gen)    │
  │ │   ├── governs_mood: u8 (0=none, 1=ind, 2=sub, 3=jus)    │
  │ │   ├── attaches_to: u8 (0=standalone, 1=next, 2=prev)    │
  │ │   ├── meaning_offset: u32 (→ string table)               │
  │ │   ├── grammar_offset: u32 (→ string table)               │
  │ │   ├── example_count: u8                                  │
  │ │   ├── disambiguation_flags: u32                          │
  │ │   └── filler: 3 bytes                                    │
  │ └── ... (particle_count entries)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ STRING TABLE                                                 │
  │ ├── Length-prefixed UTF-8 strings                           │
  │ ├── Particle text, meanings, grammar notes, examples        │
  │ └── Referenced by offsets from entry table                  │
  └──────────────────────────────────────────────────────────────┘
```

#### C Struct: Particle Entry

```c
struct ParticleEntry {
    uint8_t  particle_type;              // From ParticleType enum
    uint8_t  sub_type;                   // Sub-category
    uint8_t  usage_rank;                 // 1=most common
    uint8_t  governs_case;               // 0=none, 1=nom, 2=acc, 3=gen, 4=jus, 5=sub
    uint8_t  governs_mood;               // 0=none, 1=indicative, 2=subjunctive, 3=jussive
    uint8_t  attaches_to;                // 0=standalone, 1=next_word, 2=previous_word
    uint32_t meaning_offset;             // → string table
    uint32_t grammar_offset;             // → string table
    uint8_t  example_count;              // Number of examples
    uint32_t disambiguation_flags;       // Homograph disambiguation hints
    uint8_t  filler[3];                 // Padding to 64 bytes
};
```

### 18.3 File Packaging

```diff
  KB-0005-v1.0.0.agos-kb              # Compiled particle binary
  KB-0005-v1.0.0.agos-kb.sig          # Ed25519 signature
  KB-0005-v1.0.0.agos-kb.sha256       # SHA-256 checksum
  KB-0005-v1.0.0.source.tar.gz        # Source YAML files (optional)
```

### 18.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Hash index | 0.5 MB | 1 MB | 256-bucket perfect hash |
| Entry table | 0.1 MB | 0.2 MB | ~200 entries × 64 bytes |
| String table | 1 MB | 2 MB | Meanings, grammar notes, examples |
| Example data | 0.2 MB | 1.5 MB | Usage examples |
| Disambiguation data | 0.2 MB | 0.3 MB | Homograph resolution hints |
| **Total** | **~2 MB** | **~5 MB** | Memory-mapped load |

---

## 19. Versioning & Evolution

### 19.1 Versioning Scheme

KB-0005 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to particle schema, removal of particle types, format change | `1.0.0` → `2.0.0` | Requires KB conversion tool, invalidates all caches |
| **MINOR** | Addition of new particles, new particle types, new optional fields | `1.0.0` → `1.1.0` | Backward-compatible; existing particle IDs remain valid |
| **PATCH** | Corrections to particle grammar rules, improved examples, typo fixes | `1.0.0` → `1.0.1` | Backward-compatible; no schema changes |

### 19.2 Cross-KB Compatibility

```yaml
cross_kb_compatibility:
  KB-0001: ">= 1.0.0"       # Independent (no root dependency)
  KB-0002: ">= 1.0.0"       # Independent (no wazan dependency)
  KB-0003: ">= 1.0.0"       # Independent (no verb conjugation dependency)
  KB-0004: ">= 1.0.0"       # Independent (no noun pattern dependency)
  KB-0006: ">= 1.0.0"       # Shared fast-path in MOD-04
  KB-0007: ">= 1.0.0"       # Particle-type features referenced
```

### 19.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add new particle | MINOR | Add YAML file, regenerate hash index |
| Correct particle grammar | PATCH | Edit particle definition, update grammar notes |
| Add new particle type | MINOR | Create new YAML file, update type taxonomy |
| Remove particle | MAJOR | Only for demonstrably incorrect entries |
| Add new variant form | MINOR | Add script form variant to existing entry |

---

## 20. Quality Requirements

### 20.1 Completeness Targets

| Category | Minimum | Target | Stretch |
|----------|---------|--------|---------|
| Primary prepositions (حروف الجر) | 100% | 100% | 100% |
| Coordinating conjunctions | 100% | 100% | 100% |
| Subjunctive particles (النصب) | 100% | 100% | 100% |
| Jussive particles (الجزم) | 100% | 100% | 100% |
| Conditional particles (الشرط) | 90% | 95% | 100% |
| Interrogative particles | 100% | 100% | 100% |
| Negative particles | 100% | 100% | 100% |
| Vocative particles | 100% | 100% | 100% |
| Inna & sisters | 100% | 100% | 100% |
| Kāna & sisters | 90% | 95% | 100% |
| Answer/exception particles | 90% | 95% | 100% |
| Masdar-forming particles | 100% | 100% | 100% |
| Other functional particles | 80% | 90% | 95% |
| Homographs (all interpretations) | 85% | 90% | 95% |

### 20.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Particle type classification | 100% — each particle must belong to correct category | Manual verification by linguist |
| Governance rules | 100% — case/mood effects must match reference grammar | Automated test with known examples |
| Attached/detached form | 100% — script forms must be correct | Unicode check |
| Homograph disambiguation hints | ≥ 90% — hints must correctly prioritize interpretations | Manual review |
| Unicode normalization | 100% — all Arabic text valid NFC-normalized UTF-8 | Automated encoding check |

### 20.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate YAML structure
  ├── schema: validate against KB-0005 JSON Schema
  ├── type_check: verify particle_type is valid from taxonomy
  ├── governance_check: verify case/mood values are consistent
  └── lint: field presence, Arabic-only text for Arabic fields

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── hash_uniqueness: verify no particle text maps to wrong interpretation
  ├── grammar_regression: verify known particles produce correct governance rules
  ├── compilation: verify hash index compiles without error
  ├── size_budget: verify compiled size ≤ 5 MB
  └── regression: verify 50+ known particle usages are correctly classified

  Review (manual, per release):
  ├── sample_check: linguist reviews 10% random particle sample
  ├── hotspot_check: review particles modified since last version
  ├── homograph_audit: verify all homograph ambiguity entries are correct
  └── changelog: verify changelog accuracy
```

### 20.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Fast-path particle lookup | < 500 ns | Per lookup, average |
| Fast-path particle lookup (p99) | < 2 μs | Per lookup, 99th percentile |
| Homograph disambiguation | < 1 μs | With 2–3 candidates |
| Governance rule resolution | < 1 μs | Per particle |
| KB load time | < 10 ms | mmap + verify checksum |
| Memory | ~2–5 MB | RSS |

---

## 21. Example Entries

### 21.1 Preposition: فِي (fī, \"in\")

```json
{
  "id": "KB-0005:harf_jarr:فِي",
  "particle": "فِي",
  "transliteration": "fī",
  "particle_type": "harf_jarr",
  "sub_type": "primary",
  "category": ["preposition", "genitive-governing", "locative", "temporal"],
  "usage_rank": 2,
  "script_forms": [
    { "form": "فِي", "context": "standalone", "example": "فِي الْبَيْتِ" }
  ],
  "attaches_to": "next_word",
  "governs_case": "genitive",
  "governs_mood": null,
  "government_type": "independent",
  "grammatical_function": "Indicates location, time, or circumstance",
  "meaning": "in, within, among, at, during, concerning",
  "meaning_ar": "ظرفية مكانية أو زمانية أو مجازية",
  "usage_notes": "فِي is always written with a ي (dotless final form). It is one of the most common prepositions in Arabic, occurring thousands of times in the Quran alone.",
  "examples": [
    { "phrase": "فِي الْبَيْتِ", "transliteration": "fī l-bayti", "translation": "in the house" },
    { "phrase": "فِي الصَّيْفِ", "transliteration": "fī ṣ-ṣayfi", "translation": "in the summer" },
    { "phrase": "فِي الْحَقِيقَةِ", "transliteration": "fī l-ḥaqīqati", "translation": "in truth" }
  ],
  "homograph_group": null,
  "disambiguation_hints": [],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": ["Al-Kitab", "Mughni al-Labib"]
  }
}
```

### 21.2 Subjunctive Particle: أَنْ (an, \"that/to\")

```json
{
  "id": "KB-0005:harf_nasb:أَنْ",
  "particle": "أَنْ",
  "transliteration": "an",
  "particle_type": "harf_nasb",
  "sub_type": "primary",
  "category": ["subjunctive-governing", "masdar-forming", "subordinating"],
  "usage_rank": 1,
  "script_forms": [
    { "form": "أَنْ", "context": "standalone", "example": "أَنْ يَكْتُبَ" }
  ],
  "attaches_to": "next_word",
  "governs_case": null,
  "governs_mood": "subjunctive",
  "government_type": "independent",
  "grammatical_function": "Governs subjunctive mood on imperfect verb; creates masdar ta'wili",
  "meaning": "that, to (masdar-forming particle)",
  "meaning_ar": "حرف مصدر ونصب — يفيد معنى المصدر مع الفعل المضارع",
  "usage_notes": "Not to be confused with إِنْ (conditional). أَنْ always governs the subjunctive. When combined with لِ becomes لِكَيْ or simply لِ.",
  "examples": [
    { "phrase": "يُرِيدُ أَنْ يَكْتُبَ", "transliteration": "yurīdu an yaktuba", "translation": "he wants to write" },
    { "phrase": "أَنْ تَقُولَ الْحَقَّ", "transliteration": "an taqūla l-ḥaqqa", "translation": "that you speak the truth" }
  ],
  "homograph_group": "أن_group",
  "disambiguation_hints": [
    "Followed by imperfect verb → likely أَنْ masdariyya/nasb",
    "Followed by past verb → likely أَنْ (rare) or إِنَّ + lām"
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": ["Al-Kitab", "Mughni al-Labib"]
  }
}
```

### 21.3 Conditional: إِنْ (in, \"if\")

```json
{
  "id": "KB-0005:harf_shart:إِنْ",
  "particle": "إِنْ",
  "transliteration": "in",
  "particle_type": "harf_shart",
  "sub_type": "real_conditional",
  "category": ["conditional", "jussive-governing", "subordinating"],
  "usage_rank": 1,
  "script_forms": [
    { "form": "إِنْ", "context": "standalone", "example": "إِنْ تَكْتُبْ تَنْجَحْ" }
  ],
  "attaches_to": "next_word",
  "governs_case": null,
  "governs_mood": "jussive",
  "government_type": "requires_complement",
  "grammatical_function": "Governs jussive mood on both condition and result verbs",
  "meaning": "if (real conditional)",
  "meaning_ar": "حرف شرط جازم — يجزم فعلين: فعل الشرط وجوابه",
  "usage_notes": "إِنْ is the most common conditional particle. It governs the jussive on BOTH the condition and the result verb. Not to be confused with أَنْ (subjunctive) or إِنَّ (emphatic inna).",
  "examples": [
    { "phrase": "إِنْ تَكْتُبْ تَنْجَحْ", "transliteration": "in taktub tanjaḥ", "translation": "if you write, you succeed" },
    { "phrase": "إِنْ جِئْتَ أَكْرَمْتُكَ", "transliteration": "in jiʾta akramtuka", "translation": "if you came, I honored you" }
  ],
  "homograph_group": "إن_group",
  "disambiguation_hints": [
    "Followed by two verbs in jussive → conditional",
    "Followed by lām → إِنَّ (not إِنْ)",
    "In negative context → negative إِنْ"
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": ["Al-Kitab", "Mughni al-Labib", "Al-Muqtadab"]
  }
}
```

### 21.4 Interrogative: هَلْ (hal, question marker)

```json
{
  "id": "KB-0005:harf_istifham:هَلْ",
  "particle": "هَلْ",
  "transliteration": "hal",
  "particle_type": "harf_istifham",
  "sub_type": "yes_no_question",
  "category": ["interrogative", "question-marker"],
  "usage_rank": 1,
  "script_forms": [
    { "form": "هَلْ", "context": "standalone", "example": "هَلْ كَتَبْتَ؟" }
  ],
  "attaches_to": "next_word",
  "governs_case": null,
  "governs_mood": null,
  "government_type": "independent",
  "grammatical_function": "Marks a yes/no question; no grammatical effect on following words",
  "meaning": "does/is/are? (question marker)",
  "meaning_ar": "حرف استفهام — يُسأل به عن مضمون الجملة",
  "usage_notes": "هَلْ always expects a yes/no answer. It does not change the case or mood of the following word (unlike أَ which can precede different constructions).",
  "examples": [
    { "phrase": "هَلْ كَتَبْتَ الدَّرْسَ؟", "transliteration": "hal katabta d-darsa?", "translation": "Did you write the lesson?" },
    { "phrase": "هَلْ أَنْتَ مُسَافِرٌ؟", "transliteration": "hal anta musāfirun?", "translation": "Are you traveling?" }
  ],
  "homograph_group": null,
  "disambiguation_hints": [],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"]
  }
}
```

### 21.5 Inna & Sisters: إِنَّ (inna, \"indeed\")

```json
{
  "id": "KB-0005:harf_nasikh:إِنَّ",
  "particle": "إِنَّ",
  "transliteration": "inna",
  "particle_type": "harf_nasikh",
  "sub_type": "inna_sisters",
  "category": ["emphatic", "accusative-governing", "sentence-initial"],
  "usage_rank": 1,
  "script_forms": [
    { "form": "إِنَّ", "context": "standalone (with shadda)", "example": "إِنَّ اللَّهَ" },
    { "form": "إِنَّ", "context": "with suffix pronouns", "example": "إِنِّي, إِنَّكَ, إِنَّهُ" }
  ],
  "attaches_to": "next_word",
  "governs_case": "accusative",
  "governs_mood": null,
  "government_type": "independent",
  "grammatical_function": "Governs accusative case on the subject (ism inna) of a nominal sentence; predicate (khabar inna) remains nominative",
  "meaning": "indeed, verily, truly",
  "meaning_ar": "حرف توكيد ونصب — ينصب الاسم ويرفع الخبر",
  "usage_notes": "إِنَّ is one of the most common Quranic particles. It carries shadda (gemination) on the ن. When followed by لَام التوكيد, it forms an emphatic construction: إِنَّ ... لَ (inna ... la-). Not to be confused with أَنَّ (fatḥa + shadda) or إِنْ (conditional).",
  "examples": [
    { "phrase": "إِنَّ اللَّهَ عَلِيمٌ", "transliteration": "inna llāha ʿalīmun", "translation": "Indeed, God is all-knowing", "source": "Quran 2:282" },
    { "phrase": "إِنَّ فِي ذَلِكَ لَعِبْرَةً", "transliteration": "inna fī dhālika la-ʿibratan", "translation": "Indeed, in that is a lesson" }
  ],
  "homograph_group": "إن_group",
  "disambiguation_hints": [
    "Followed by noun/pronoun in accusative → likely إِنَّ",
    "Followed by لَام + noun/pronoun → إِنَّ with لَام التوكيد",
    "Shadda on ن distinguishes from إِنْ"
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": ["Al-Kitab", "Mughni al-Labib"]
  }
}
```

---

## 22. Cross-References

### 22.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0005 in module catalog; fast path |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Fast path particle check before root extraction |
| SPEC-0001-C3 | Compilation Pipeline (MOD-05) | Particle governance during syntactic parsing |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Particle entry in MOD-03 tokenization |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-2) | Particle morpheme types in token structure |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-4) | POS features including particle classification |
| SPEC-0001-C6 | Deployment & Runtime Considerations | KB bundling, size budget |
| SPEC-0001-C8 | Security, Validation & Error Handling | KB integrity verification |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0005 size (2–5 MB), lookup performance |
| KB-0001 | Roots Database | Particles have NO root derivation (checked before root extraction) |
| KB-0006 | Pronouns | Shared fast-path in MOD-04 |
| KB-0007 | Morphological Features | Feature taxonomy referenced by particle entries |

### 22.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (8th C. CE) | Foundational grammar; defines the particle classification system |
| Ibn Hisham, Mughni al-Labib (14th C. CE) | Definitive reference on Arabic particles and their meanings |
| Al-Ashmuni, Sharh al-Ashmuni (15th C. CE) | Comprehensive grammatical commentary |
| Wright's Arabic Grammar (1859) | Western reference for Arabic particle grammar |
| Badawi, Carter & Gully, Modern Written Arabic (2004) | Modern particle usage in MSA |
| Ryding, A Reference Grammar of Modern Standard Arabic (2005) | Contemporary reference for particle functions |

---

## Progress Summary

**KB-0005: Particles — Grammatical & Functional Words**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Particles in Arabic Grammar | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Particle Entry Schema | ✓ COMPLETE |
| Section 5 | Prepositions (Huruf al-Jarr) | ✓ COMPLETE |
| Section 6 | Conjunctions (Huruf al-ʿAṭf) | ✓ COMPLETE |
| Section 7 | Subjunctive Particles (Huruf al-Naṣb) | ✓ COMPLETE |
| Section 8 | Jussive Particles (Huruf al-Jazm) | ✓ COMPLETE |
| Section 9 | Conditional Particles (Huruf al-Sharṭ) | ✓ COMPLETE |
| Section 10 | Interrogative Particles (Huruf al-Istifhām) | ✓ COMPLETE |
| Section 11 | Negative Particles (Huruf al-Nafy) | ✓ COMPLETE |
| Section 12 | Vocative Particles (Huruf al-Nidāʾ) | ✓ COMPLETE |
| Section 13 | Inna & Sisters (Inna wa Akhawātuhā) | ✓ COMPLETE |
| Section 14 | Kāna & Sisters (Kāna wa Akhawātuhā) | ✓ COMPLETE |
| Section 15 | Other Functional Particles | ✓ COMPLETE |
| Section 16 | Particle Ambiguity Resolution | ✓ COMPLETE |
| Section 17 | Particle Matching Algorithm | ✓ COMPLETE |
| Section 18 | Serialization & Storage | ✓ COMPLETE |
| Section 19 | Versioning & Evolution | ✓ COMPLETE |
| Section 20 | Quality Requirements | ✓ COMPLETE |
| Section 21 | Example Entries | ✓ COMPLETE |
| Section 22 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–9), KB-0006, KB-0007.

**Recommended next document:** KB-0006 (Pronouns) — the linguistic knowledge base for Arabic pronouns.
