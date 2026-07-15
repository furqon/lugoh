---
kb_id: KB-0006
title: Pronouns — Personal, Demonstrative & Relative
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview (Module Catalog — KB-0006)
  - SPEC-0001-C3: Compilation Pipeline (MOD-03, MOD-04 — Fast Path)
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-2, IR-4)
  - SPEC-0001-C6: Deployment & Runtime Considerations (KB Bundling)
  - SPEC-0001-C8: Security, Validation & Error Handling (KB Integrity)
  - SPEC-0001-C9: Performance Targets & Constraints (KB Size & Lookup)
  - RFC-0002: Grammar Bytecode Format (Pronoun Feature Bitmask)
  - KB-0005: Particles
  - KB-0007: Morphological Features
---

# KB-0006: Pronouns — Personal, Demonstrative & Relative

## Table of Contents

1. [Purpose & Scope](#1-purpose--scope)
2. [Pronouns in Arabic Grammar](#2-pronouns-in-arabic-grammar)
3. [Data Model](#3-data-model)
4. [Pronoun Entry Schema](#4-pronoun-entry-schema)
5. [Attached Pronouns (al-Ḍamāʾir al-Muttaṣila)](#5-attached-pronouns-al-damair-al-muttasila)
6. [Detached Pronouns (al-Ḍamāʾir al-Munfaṣila)](#6-detached-pronouns-al-damair-al-munfasila)
7. [Demonstrative Pronouns (Asmāʾ al-Ishāra)](#7-demonstrative-pronouns-asma-al-ishara)
8. [Relative Pronouns (al-Asmāʾ al-Mawṣūla)](#8-relative-pronouns-al-asma-al-mawsula)
9. [Interrogative Pronouns (Asmāʾ al-Istifhām)](#9-interrogative-pronouns-asma-al-istifham)
10. [Conditional Pronouns (Asmāʾ al-Sharṭ)](#10-conditional-pronouns-asma-al-shart)
11. [Pronoun Inflection Features](#11-pronoun-inflection-features)
12. [Pronoun Attachment & Clitic Behavior](#12-pronoun-attachment--clitic-behavior)
13. [Anaphora & Pronoun Resolution](#13-anaphora--pronoun-resolution)
14. [Pronoun Matching Algorithm](#14-pronoun-matching-algorithm)
15. [Serialization & Storage](#15-serialization--storage)
16. [Versioning & Evolution](#16-versioning--evolution)
17. [Quality Requirements](#17-quality-requirements)
18. [Example Entries](#18-example-entries)
19. [Cross-References](#19-cross-references)

---

## 1. Purpose & Scope

### 1.1 Purpose

KB-0006 is the **authoritative register of Arabic pronouns** (الضمائر, `al-ḍamāʾir`) used by the AGOS platform. Pronouns, like particles, are part of the **fast path** — checked immediately after particles and before any root extraction, because they have no morphological derivation.

KB-0006 answers: **\"Is this token a pronoun? What person, number, and gender does it encode? How does it attach to the surrounding context?\"**

### 1.2 Scope

KB-0006 covers:

| Category | In Scope | Out of Scope |
|----------|----------|--------------|
| **Personal pronouns** | Attached (متصل) and Detached (منفصل) | Archaic/rare forms (covered by KB plugins) |
| **Demonstratives** | هٰذَا, ذٰلِكَ, etc. with gender/number forms | — |
| **Relative pronouns** | الَّذِي, الَّتِي, etc. | — |
| **Interrogative pronouns** | مَنْ, مَا, etc. | — |
| **Conditional pronouns** | مَنْ, مَا, etc. (when used as sharṭ) | — |
| **Pronoun suffixes** | Object and possessive suffixes attached to verbs/nouns | Enclitic analysis (done by MOD-03) |
| **Inflection features** | Person, number, gender, pronoun type, attachment type | Semantic analysis (done by MOD-05/07) |
| **Language** | Classical Arabic, Modern Standard Arabic | Dialectal pronoun systems |

### 1.3 Target Audience

- **AGOS Pipeline:** MOD-04 (MorphologicalParser) reads KB-0006 during the fast-path pronoun check (Step 3.2). MOD-05 (SyntacticParser) uses KB-0006 for anaphora resolution assignments. MOD-11 (ExplanationEngine) uses KB-0006 for pronoun reference explanations.
- **Linguists & Data Maintainers:** Edit and extend KB-0006 with additional pronoun forms or grammatical notes.
- **Anaphora Resolution Module:** KB-0006 provides the pronoun feature sets needed for antecedent matching.

### 1.4 Role in the AGOS Pipeline

```diff
  MOD-04: MorphologicalParser
    │
    ├── Step 3.1: Check KB-0005 (Particles)
    ├── Step 3.2: Check KB-0006 (Pronouns)   ◄── THIS KB
    │   ├── Is it a known pronoun form?
    │   ├── Record person/number/gender features
    │   └── If yes → skip root extraction!
    │
    ├── Step 4+: Root extraction (only for non-pronoun tokens)
    │
    ▼
  MOD-05: SyntacticParser
    │
    └── Anaphora resolution: match pronouns to antecedents
        using person/number/gender features from KB-0006
```

### 1.5 Relationship to Other KBs

```diff
  KB-0006: Pronouns                     ◄── This document (Pronoun reference data)
    │
    ├── Fast-path (like KB-0005): checked before root extraction
    ├── No root derivation (independent of KB-0001/2/3/4)
    ├── Provides person/number/gender features for anaphora (MOD-05)
    │
    ├──► KB-0005: Particles             — Also fast-path, also invariable
    └──► KB-0007: Morphological Feat.   — Feature taxonomy for pronoun features
```

---

## 2. Pronouns in Arabic Grammar

### 2.1 Definition

A pronoun (ضمير, `ḍamīr` — pl. ضمائر, `ḍamāʾir`) is a word that substitutes for a noun (antecedent). Arabic pronouns are:

1. **Invariable** (مبني) — they have fixed vocalization (except for some demonstratives).
2. **No root derivation** — they cannot be analyzed by root extraction.
3. **Closed class** — the set of Arabic pronouns is small and well-cataloged (~40–60 entries).
4. **Feature-rich** — pronouns encode person (1st/2nd/3rd), number (singular/dual/plural), and gender (masculine/feminine).

### 2.2 Pronoun vs. Other Categories

| Feature | Pronoun (ضمير) | Noun (اسم) | Particle (حرف) |
|---------|---------------|------------|----------------|
| **Root derivation** | None | Yes (usually) | None |
| **Inflection** | Fixed (mabni) | Case/number/gender | Invariable |
| **Person encoding** | Yes (1st/2nd/3rd) | No | No |
| **Anaphoric reference** | Yes | No (antecedent) | No |
| **Closed class** | Yes (~50) | No | Yes (~200) |
| **Fast-path in MOD-04** | Yes (checked 2nd) | No | Yes (checked 1st) |
| **Can attach as suffix** | Yes (object/possessive) | No | Some |

### 2.3 Pronoun Taxonomy

```diff
  Arabic Pronouns (الضمائر العربية)
  │
  ├── 1. PERSONAL PRONOUNS (الضمائر الشخصية)
  │     ├── Attached (متصل) — Suffixes: تُ, تَ, وا, ي, ك, ه, ... (13 forms)
  │     │    ├── Subject pronouns (on verbs)
  │     │    ├── Object pronouns (on verbs)
  │     │    └── Possessive pronouns (on nouns)
  │     │
  │     └── Detached (منفصل) — Separate words: أَنَا, أَنْتَ, هُوَ, ... (13 forms)
  │          ├── Nominative (الرفع): أَنَا, نَحْنُ, أَنْتَ, ...
  │          └── Accusative (النصب): إِيَّايَ, إِيَّانَا, ... (rare)
  │
  ├── 2. DEMONSTRATIVES (أسماء الإشارة)
  │     ├── Near (القريب): هٰذَا, هٰذِهِ, هٰؤُلَاءِ
  │     ├── Mid (المتوسط): ذٰلِكَ, تِلْكَ, أُولٰئِكَ
  │     └── Far (البعيد): ذَاكَ, تَاكَ, أُلَاكَ
  │
  ├── 3. RELATIVE PRONOUNS (الأسماء الموصولة)
  │     ├── Definite (المعرفة): الَّذِي, الَّتِي, الَّذِينَ, ...
  │     ├── Indefinite (النكرة): مَنْ (whoever), مَا (whatever)
  │     └── Compound: الَّذِي + particle (e.g., بِمَا, لِمَنْ)
  │
  ├── 4. INTERROGATIVE PRONOUNS (أسماء الاستفهام)
  │     ├── مَنْ (who?), مَا/مَاذَا (what?), أَيُّ (which?)
  │     ├── كَمْ (how many?), كَيْفَ (how?)
  │     └── أَيْنَ (where?), مَتَى (when?)
  │
  └── 5. CONDITIONAL PRONOUNS (أسماء الشرط)
        ├── مَنْ (whoever), مَا (whatever)
        ├── مَهْمَا (whatever), any/however
        └── حَيْثُمَا (wherever), etc.
```

### 2.4 Person/Number/Gender Paradigm

The personal pronoun paradigm has 13 slots (5 singular, 3 dual, 5 plural):

| # | Label | Person | Number | Gender | Detached | Attached (Nom) | Attached (Obj/Poss) |
|---|-------|--------|--------|--------|----------|---------------|-------------------|
| 1 | **1s** | 1st | Singular | Common | أَنَا | -تُ | -نِي / -ي |
| 2 | **1p** | 1st | Plural | Common | نَحْنُ | -نَا | -نَا |
| 3 | **2ms** | 2nd | Singular | Masculine | أَنْتَ | -تَ | -كَ |
| 4 | **2fs** | 2nd | Singular | Feminine | أَنْتِ | -تِ | -كِ |
| 5 | **2md** | 2nd | Dual | Common | أَنْتُمَا | -تُمَا | -كُمَا |
| 6 | **2mp** | 2nd | Plural | Masculine | أَنْتُمْ | -تُمْ | -كُمْ |
| 7 | **2fp** | 2nd | Plural | Feminine | أَنْتُنَّ | -تُنَّ | -كُنَّ |
| 8 | **3ms** | 3rd | Singular | Masculine | هُوَ | -َ | -هُ / -هِ |
| 9 | **3fs** | 3rd | Singular | Feminine | هِيَ | -َتْ | -هَا |
| 10 | **3md** | 3rd | Dual | Masculine | هُمَا | -َا | -هُمَا |
| 11 | **3fd** | 3rd | Dual | Feminine | هُمَا | -َتَا | -هُمَا |
| 12 | **3mp** | 3rd | Plural | Masculine | هُمْ | -ُوا / -ُو | -هُمْ |
| 13 | **3fp** | 3rd | Plural | Feminine | هُنَّ | -ْنَ | -هُنَّ |

### 2.5 Pronoun Count Target

| Category | Estimated Count | Notes |
|----------|----------------|-------|
| Attached personal pronouns | ~15–20 | Subject, object, possessive forms (some overlap) |
| Detached personal pronouns | ~15 | Nominative + accusative (إِيَّايَ series) |
| Demonstratives | ~12–15 | Near/mid/far × gender/number |
| Relative pronouns | ~10–12 | Definite + indefinite forms |
| Interrogative pronouns | ~8–12 | Including adverbial interrogatives |
| Conditional pronouns | ~8–10 | |
| **Total (Version 1.0)** | **~60–80** | Including all attested grammatical forms |

---

## 3. Data Model

### 3.1 Logical Data Model

```yaml
Pronouns Database (KB-0006)
├── Metadata
│   ├── kb_id: "KB-0006"
│   ├── version: "1.0.0"
│   ├── pronoun_count: integer
│   ├── pronoun_type_counts: map
│   ├── created_at: timestamp
│   ├── sources: string[]
│   └── checksum_sha256: string
│
└── Pronouns: PronounEntry[]
    ├── Attached personal pronouns
    ├── Detached personal pronouns
    ├── Demonstratives
    ├── Relatives
    ├── Interrogatives
    └── Conditionals
```

### 3.2 Storage Model

KB-0006 is stored in two formats:

| Format | Use Case | Size | Access Pattern |
|--------|----------|------|----------------|
| **Source (YAML/JSON)** | Authoring, review, diff tracking | ~0.5–1 MB | Human-readable |
| **Compiled (Hash Index)** | Production pipeline | ~1–2 MB | Memory-mapped O(1) lookup |

KB-0006 is the **smallest** AGOS KB alongside KB-0005.

### 3.3 Pronoun Attachment Model

Pronouns can attach to words in three ways:

1. **As standalone tokens** — Detached pronouns, demonstratives, relatives: checked as whole words.
2. **As suffixes on verbs** — Subject pronouns (-تُ, -تَ, -تِ, -َا, -ُوا, etc.) and object pronouns (-نِي, -نَا, -كَ, -كِ, -هُ, -هَا, -هُمْ, etc.).
3. **As suffixes on nouns** — Possessive pronouns (same forms as object pronouns: -ي, -كَ, -ه, -هَا, etc.).
4. **As suffixes on particles** — Some particles take pronominal suffixes (إِنَّ + ي = إِنِّي, لِ + ه = لَهُ, بِ + ك = بِكَ).

The attached pronoun forms are identified by MOD-03 (Preprocessor) during **clitic stripping** and then matched against KB-0006 for feature resolution.

---

## 4. Pronoun Entry Schema

### 4.1 Schema Definition

```yaml
PronounEntry:
  # --- Identity ---
  id: string                           # "KB-0006:{pronoun_type}:{pronoun_form}"
                                       # e.g., "KB-0006:detached:أَنَا"
  pronoun: string                      # The pronoun in Arabic script
  transliteration: string              # Latin transliteration

  # --- Classification ---
  pronoun_type: PronounType            # detached / attached / demonstrative / relative /
                                       # interrogative / conditional
  sub_type: string | null              # e.g., "subject", "object", "possessive",
                                       # "near", "far", "definite", "indefinite"

  # --- Grammatical Features ---
  person: 1 | 2 | 3 | null            # 1st, 2nd, 3rd person
  number: "singular" | "dual" | "plural" | null
  gender: "masculine" | "feminine" | "common" | null

  # --- Attachment ---
  attachment_type: "prefix" | "suffix" | "infix" | "standalone" | "both"
  attaches_to: "verb" | "noun" | "particle" | "any" | null

  # --- Orthography ---
  script_forms: ScriptForm[]           # Alternative orthographic realizations
  aliases: string[]                    # Alternative forms (e.g., after vowels: ه → ـهِ)

  # --- Semantics ---
  meaning: string                      # English gloss
  meaning_ar: string                   # Arabic description
  reference_notes: string | null       # Anaphoric reference notes

  # --- Examples ---
  examples: Example[]

  # --- Attestation ---
  attestation: Attestation

  # --- Metadata ---
  created_at: timestamp
  updated_at: timestamp
  version_added: string
```

### 4.2 Supporting Types

```yaml
PronounType:
  "personal_attached" | "personal_detached" | "demonstrative" |
  "relative" | "interrogative" | "conditional" | "compound"

ScriptForm:
  form: string                         # The orthographic form
  context: string                      # When this form is used (phonological context)
  example: string | null               # Example usage

Example:
  phrase: string                       # Example phrase or sentence
  transliteration: string
  translation: string
  source: string | null                # Source citation

Attestation:
  confidence: "certain" | "well_attested" | "attested" | "disputed"
  primary_sources: string[]
  classical_references: string[]
  notes: string | null

# Reserved for future expansion:
# AttachmentRule would define phonological context → form mappings
#   host_type: "verb" | "noun" | "particle" | "any"
#   position: "after_vowel" | "after_consonant" | ...
#   form_variant: string | null          # Different form depending on host
```

### 4.3 JSON Example (Detached Pronoun — هُوَ)

```json
{
  "id": "KB-0006:personal_detached:3ms:هُوَ",
  "pronoun": "هُوَ",
  "transliteration": "huwa",
  "pronoun_type": "personal_detached",
  "sub_type": "nominative",
  "person": 3,
  "number": "singular",
  "gender": "masculine",
  "attachment_type": "standalone",
  "attaches_to": null,
  "script_forms": [
    { "form": "هُوَ", "context": "standard", "example": "هُوَ كَاتِبٌ" }
  ],
  "aliases": [],
  "meaning": "he",
  "meaning_ar": "ضمير الغائب المفرد المذكر",
  "reference_notes": "3rd person masculine singular; used for both humans and non-humans",
  "examples": [
    { "phrase": "هُوَ كَاتِبٌ", "transliteration": "huwa kātibun", "translation": "He is a writer" },
    { "phrase": "هُوَ اللَّهُ", "transliteration": "huwa llāhu", "translation": "He is God", "source": "Quran 59:22" }
  ],
  "attestation": {
    "confidence": "certain",
    "primary_sources": ["Sibawayh, Al-Kitab"],
    "classical_references": ["Al-Kitab", "Mughni al-Labib"]
  },
  "created_at": "2026-07-15T00:00:00Z",
  "updated_at": "2026-07-15T00:00:00Z",
  "version_added": "1.0.0"
}
```

---

## 5. Attached Pronouns (al-Ḍamāʾir al-Muttaṣila)

Attached pronouns (الضمائر المتصلة, `al-ḍamāʾir al-muttaṣila`) are **suffixes** that attach to verbs, nouns, or particles. They serve three roles depending on the host:

### 5.1 Subject Pronouns (Attached to Verbs)

These suffix pronouns mark the **subject** of a perfect verb:

| Suffix | Pronunciation | Person | Number | Gender | Example (كَتَبَ) | Role |
|--------|--------------|--------|--------|--------|----------------|------|
| **-تُ** | -tu | 1st | Singular | Common | كَتَبْتُ (katabtu) | I wrote |
| **-نَا** | -nā | 1st | Plural | Common | كَتَبْنَا (katabnā) | we wrote |
| **-تَ** | -ta | 2nd | Singular | Masc | كَتَبْتَ (katabta) | you (ms) wrote |
| **-تِ** | -ti | 2nd | Singular | Fem | كَتَبْتِ (katabti) | you (fs) wrote |
| **-تُمَا** | -tumā | 2nd | Dual | Common | كَتَبْتُمَا (katabtumā) | you (d) wrote |
| **-تُمْ** | -tum | 2nd | Plural | Masc | كَتَبْتُمْ (katabtum) | you (mp) wrote |
| **-تُنَّ** | -tunna | 2nd | Plural | Fem | كَتَبْتُنَّ (katabtunna) | you (fp) wrote |
| **-َ (zero)** | -a | 3rd | Singular | Masc | كَتَبَ (kataba) | he wrote |
| **-َتْ** | -at | 3rd | Singular | Fem | كَتَبَتْ (katabat) | she wrote |
| **-َا** | -ā | 3rd | Dual | Masc | كَتَبَا (katabā) | they (md) wrote |
| **-َتَا** | -atā | 3rd | Dual | Fem | كَتَبَتَا (katabatā) | they (fd) wrote |
| **-ُوا** | -ū | 3rd | Plural | Masc | كَتَبُوا (katabū) | they (mp) wrote |
| **-ْنَ** | -na | 3rd | Plural | Fem | كَتَبْنَ (katabna) | they (fp) wrote |

Note: For the imperfect, subject marking uses **prefixes** (أَ, نَ, يَ, تَ) with optional suffixes. These prefix markers are part of the verb conjugation system (KB-0003), not stored in KB-0006.

### 5.2 Object Pronouns (Attached to Verbs)

These suffix pronouns mark the **direct or indirect object** of a verb:

| Suffix | Pronunciation | Person | Number | Gender | Example (كَتَبَ +) | Notes |
|--------|--------------|--------|--------|--------|-------------------|-------|
| **-نِي** | -nī | 1st | Singular | Common | كَتَبَنِي (katabanī) | he wrote me |
| **-نَا** | -nā | 1st | Plural | Common | كَتَبَنَا (katabanā) | he wrote us |
| **-كَ** | -ka | 2nd | Singular | Masc | كَتَبَكَ (katabaka) | he wrote you (ms) |
| **-كِ** | -ki | 2nd | Singular | Fem | كَتَبَكِ (katabaki) | he wrote you (fs) |
| **-كُمَا** | -kumā | 2nd | Dual | Common | كَتَبَكُمَا (katabakumā) | he wrote you (d) |
| **-كُمْ** | -kum | 2nd | Plural | Masc | كَتَبَكُمْ (katabakum) | he wrote you (mp) |
| **-كُنَّ** | -kunna | 2nd | Plural | Fem | كَتَبَكُنَّ (katabakunna) | he wrote you (fp) |
| **-هُ** | -hu | 3rd | Singular | Masc | كَتَبَهُ (katabahu) | he wrote him/it |
| **-هَا** | -hā | 3rd | Singular | Fem | كَتَبَهَا (katabahā) | he wrote her/it |
| **-هُمَا** | -humā | 3rd | Dual | Common | كَتَبَهُمَا (katabahumā) | he wrote them (d) |
| **-هُمْ** | -hum | 3rd | Plural | Masc | كَتَبَهُمْ (katabahum) | he wrote them (mp) |
| **-هُنَّ** | -hunna | 3rd | Plural | Fem | كَتَبَهُنَّ (katabahunna) | he wrote them (fp) |

### 5.3 Possessive Pronouns (Attached to Nouns)

The **same suffix forms** as object pronouns attach to nouns to indicate possession:

| Suffix | Example | Meaning | Suffix | Example | Meaning |
|--------|---------|---------|--------|---------|---------|
| **-ي** / **-ِي** | كِتَابِي (kitābī) | my book | **-نَا** | كِتَابُنَا (kitābunā) | our book |
| **-كَ** | كِتَابُكَ (kitābuka) | your (ms) book | **-كُمْ** | كِتَابُكُمْ (kitābukum) | your (mp) book |
| **-كِ** | كِتَابُكِ (kitābuki) | your (fs) book | **-كُنَّ** | كِتَابُكُنَّ (kitābukunna) | your (fp) book |
| **-هُ** | كِتَابُهُ (kitābuhu) | his book | **-هُمْ** | كِتَابُهُمْ (kitābuhum) | their (mp) book |
| **-هَا** | كِتَابُهَا (kitābuhā) | her book | **-هُنَّ** | كِتَابُهُنَّ (kitābuhunna) | their (fp) book |

### 5.4 Phonological Variants of Attached Pronouns

Some attached pronouns change form based on the **preceding vowel**:

| Pronoun | After Consonant/Vowel | Variant | Example |
|---------|----------------------|---------|---------|
| 1s object | After consonant | -نِي (nī) | ضَرَبَنِي (ḍarabanī) |
| 1s object | After vowel | -ي (ī) / -يَ | ضَرَبَنِيَ (ḍarabanīya) |
| 3ms object | After vowel (some contexts) | -هِ (hi) | كَتَبَهُ → after yā: عَلَيْهِ (ʿalayhi) |
| 3ms possessive | After -i/u vowel | -هِ | كِتَابِهِ (kitābihi) — \"his book\" |

These phonological variants are stored in KB-0006 as `script_forms` with appropriate context notes.

---

## 6. Detached Pronouns (al-Ḍamāʾir al-Munfaṣila)

Detached pronouns (الضمائر المنفصلة, `al-ḍamāʾir al-munfaṣila`) are **independent words** that serve as the subject of nominal sentences or as emphasized subjects.

### 6.1 Nominative Detached Pronouns

These are the standard standalone personal pronouns:

| Form | Translit. | Person | Number | Gender | Meaning |
|------|-----------|--------|--------|--------|---------|
| **أَنَا** | anā | 1st | Singular | Common | I |
| **نَحْنُ** | naḥnu | 1st | Plural | Common | we |
| **أَنْتَ** | anta | 2nd | Singular | Masc | you (masculine) |
| **أَنْتِ** | anti | 2nd | Singular | Fem | you (feminine) |
| **أَنْتُمَا** | antumā | 2nd | Dual | Common | you (dual) |
| **أَنْتُمْ** | antum | 2nd | Plural | Masc | you (masculine plural) |
| **أَنْتُنَّ** | antunna | 2nd | Plural | Fem | you (feminine plural) |
| **هُوَ** | huwa | 3rd | Singular | Masc | he |
| **هِيَ** | hiya | 3rd | Singular | Fem | she |
| **هُمَا** | humā | 3rd | Dual | Common | they (dual) |
| **هُمْ** | hum | 3rd | Plural | Masc | they (masculine) |
| **هُنَّ** | hunna | 3rd | Plural | Fem | they (feminine) |

### 6.2 Accusative Detached Pronouns (إِيَّايَ Series)

These are used for emphasis or when the pronoun cannot attach:

| Form | Translit. | Person | Number | Gender |
|------|-----------|--------|--------|--------|
| **إِيَّايَ** | iyyāya | 1st | Singular | Common |
| **إِيَّانَا** | iyyānā | 1st | Plural | Common |
| **إِيَّاكَ** | iyyāka | 2nd | Singular | Masc |
| **إِيَّاكِ** | iyyāki | 2nd | Singular | Fem |
| **إِيَّاكُمَا** | iyyākumā | 2nd | Dual | Common |
| **إِيَّاكُمْ** | iyyākum | 2nd | Plural | Masc |
| **إِيَّاكُنَّ** | iyyākunna | 2nd | Plural | Fem |
| **إِيَّاهُ** | iyyāhu | 3rd | Singular | Masc |
| **إِيَّاهَا** | iyyāhā | 3rd | Singular | Fem |
| **إِيَّاهُمَا** | iyyāhumā | 3rd | Dual | Common |
| **إِيَّاهُمْ** | iyyāhum | 3rd | Plural | Masc |
| **إِيَّاهُنَّ** | iyyāhunna | 3rd | Plural | Fem |

These are used in constructions like: إِيَّاكَ نَعْبُدُ (iyyāka naʿbudu) \"You alone we worship\" — the detached accusative pronoun إِيَّاكَ is used for emphasis.

---

## 7. Demonstrative Pronouns (Asmāʾ al-Ishāra)

Demonstrative pronouns (أسماء الإشارة, `asmāʾ al-ishāra`) indicate **proximity** or **distance**.

### 7.1 Demonstrative Table

| Form | Translit. | Gender | Number | Distance | Meaning |
|------|-----------|--------|--------|----------|---------|
| **هٰذَا** | hādhā | Masc | Singular | Near | this |
| **هٰذِهِ** | hādhihi | Fem | Singular | Near | this |
| **هٰذَانِ** | hādhāni | Masc | Dual (Nom) | Near | these two |
| **هٰذَيْنِ** | hādhayni | Masc | Dual (Acc/Gen) | Near | these two |
| **هَاتَانِ** | hātāni | Fem | Dual (Nom) | Near | these two |
| **هَاتَيْنِ** | hātayni | Fem | Dual (Acc/Gen) | Near | these two |
| **هٰؤُلَاءِ** | hāʾulāʾi | Common | Plural | Near | these |
| **ذٰلِكَ** | dhālika | Masc | Singular | Far (mid) | that |
| **تِلْكَ** | tilka | Fem | Singular | Far (mid) | that |
| **ذَانِكَ** | dhānika | Masc | Dual | Far (mid) | those two |
| **أُولٰئِكَ** | ulāʾika | Common | Plural | Far (mid) | those |
| **ذَاكَ** | dhāka | Masc | Singular | Far | that (far) |
| **تَاكَ** | tāka | Fem | Singular | Far | that (far) |

### 7.2 Demonstrative Grammar Notes

- Demonstratives precede the noun they modify: **هٰذَا كِتَابٌ** (this is a book).
- When followed by a definite noun, they form a definite phrase: **هٰذَا الْكِتَابُ** (this book).
- The far demonstratives (ذاك, تاك) are less common in MSA but appear in Classical Arabic.
- The dual forms inflect for case (nominative vs. accusative/genitive).

---

## 8. Relative Pronouns (al-Asmāʾ al-Mawṣūla)

Relative pronouns (الأسماء الموصولة, `al-asmāʾ al-mawṣūla`) introduce **relative clauses**.

### 8.1 Definite Relative Pronouns

| Form | Translit. | Gender | Number | Usage |
|------|-----------|--------|--------|-------|
| **الَّذِي** | alladhī | Masc | Singular | who, which, that |
| **الَّتِي** | allatī | Fem | Singular | who, which, that |
| **اللَّذَانِ** | alladhāni | Masc | Dual (Nom) | who, which (two) |
| **اللَّذَيْنِ** | alladhayni | Masc | Dual (Acc/Gen) | who, which (two) |
| **اللَّتَانِ** | allatāni | Fem | Dual (Nom) | who, which (two) |
| **اللَّتَيْنِ** | allatayni | Fem | Dual (Acc/Gen) | who, which (two) |
| **الَّذِينَ** | alladhīna | Masc | Plural | who, which |
| **اللَّاتِي** | allātī | Fem | Plural | who, which |
| **اللَّوَاتِي** | allawātī | Fem | Plural | who, which (variant) |

### 8.2 Indefinite Relative Pronouns

| Form | Translit. | Meaning | Notes |
|------|-----------|---------|-------|
| **مَنْ** | man | whoever, the one who | Used for humans |
| **مَا** | mā | whatever, that which | Used for non-humans |
| **أَيُّ** | ayyu | whichever | With construct; agrees in gender |
| **ذُو** | dhū | the one who | Archaic; used in construct, case-inflected |

### 8.3 Relative Pronoun Grammar

```diff
  Relative clause structure:

  الَّذِي + Verb clause / Noun clause

  Examples:
  جَاءَ الَّذِي كَتَبَ
  (jāʾa alladhī kataba)
  \"The one who wrote came\"

  الْكِتَابُ الَّذِي قَرَأْتُهُ
  (al-kitābu alladhī qaraʾtuhu)
  \"The book that I read\"
  (Note: resumptive pronoun -hu refers back to antecedent!)
```

---

## 9. Interrogative Pronouns (Asmāʾ al-Istifhām)

Interrogative pronouns (أسماء الاستفهام, `asmāʾ al-istifhām`) are used to ask **questions**.

### 9.1 Interrogative Pronoun Table

| Form | Translit. | Meaning | Category | Example |
|------|-----------|---------|----------|---------|
| **مَنْ** | man | who? | Person | مَنْ كَتَبَ؟ (who wrote?) |
| **مَا** | mā | what? | Thing | مَا هٰذَا؟ (what is this?) |
| **مَاذَا** | mādhā | what? | Thing | مَاذَا تَفْعَلُ؟ (what do you do?) |
| **أَيُّ** | ayyu | which? | Selection | أَيُّ كِتَابٍ؟ (which book?) |
| **كَمْ** | kam | how many/much? | Quantity | كَمْ كِتَابًا؟ (how many books?) |
| **كَيْفَ** | kayfa | how? | Manner | كَيْفَ أَنْتَ؟ (how are you?) |
| **أَيْنَ** | ayna | where? | Place | أَيْنَ الْكِتَابُ؟ (where is the book?) |
| **أَنَّى** | annā | how/where? | Manner/place | أَنَّى لَكَ هٰذَا؟ (how did you get this?) |
| **مَتَى** | matā | when? | Time | مَتَى جِئْتَ؟ (when did you come?) |
| **أَيَّانَ** | ayyāna | when? | Time (future) | أَيَّانَ نَصْرُ اللَّهِ؟ (when is God's victory?) |

### 9.2 Interrogative vs. Other Functions

Some interrogative pronouns overlap with other pronoun categories:

| Form | Interrogative | Relative | Conditional | Other |
|------|--------------|----------|-------------|-------|
| **مَنْ** | who? | whoever | whoever | — |
| **مَا** | what? | that which | whatever | Negative particle, masdar-forming |
| **أَيُّ** | which? | whichever | whichever | — |

Disambiguation is handled by MOD-05 based on sentence type (question vs. statement) and syntactic position.

---

## 10. Conditional Pronouns (Asmāʾ al-Sharṭ)

Conditional pronouns (أسماء الشرط, `asmāʾ al-sharṭ`) are used in **conditional sentences** as the condition word. They govern the **jussive mood** on both verbs.

### 10.1 Conditional Pronoun Table

| Form | Translit. | Meaning | Notes |
|------|-----------|---------|-------|
| **مَنْ** | man | whoever | For people |
| **مَا** | mā | whatever | For things |
| **مَهْمَا** | mahnā | whatever, no matter what | Emphatic |
| **أَيُّ** | ayyu | whichever | Agrees in gender; used in construct |
| **مَتَى** | matā | whenever | Time |
| **أَيَّانَ** | ayyāna | whenever | Time (future) |
| **أَيْنَ** | ayna | wherever | Place |
| **أَنَّى** | annā | wherever/however | Place/manner |
| **حَيْثُمَا** | ḥaythumā | wherever | Place (with -مَا suffix) |
| **كَيْفَمَا** | kayfama | however | Manner (with -مَا suffix) |

### 10.2 Conditional Pronoun Grammar

```diff
  Conditional pronoun structure:

  مَنْ + Verb₁ (jussive) + Verb₂ (jussive)

  Example:
  مَنْ يَكْتُبْ يَنْجَحْ
  (man yaktub yanjaḥ)
  \"Whoever writes succeeds\"

  Both verbs are in the jussive mood.
```

---

## 11. Pronoun Inflection Features

### 11.1 Feature Encoding for MOD-04/MOD-05

When a pronoun is identified, KB-0006 provides the following feature set for downstream modules:

```yaml
PronounFeatures:
  is_pronoun: true                      # Flag for downstream use
  pronoun_type: string                  # From PronounType enum
  person: 1 | 2 | 3 | null
  number: "singular" | "dual" | "plural" | null
  gender: "masculine" | "feminine" | "common" | null
  attachment: "standalone" | "suffix" | "prefix"
  host_type: "verb" | "noun" | "particle" | null  # What the pronoun attaches to
  case_role: "subject" | "object" | "possessive" | "independent" | null
```

### 11.2 Feature Bitmask Mapping (RFC-0002 Compatible)

KB-0006 features map to the RFC-0002 feature bitmask:

```text
Bits 28–31: pronoun_type
  0     → not_a_pronoun
  1     → personal_attached
  2     → personal_detached
  3     → relative
  4     → demonstrative

Bits 20–22: person (encoded with verb form)
Bits 13–15: number (shared field)
Bits 16–17: gender (shared field)

Note: Values 3 (relative) and 4 (demonstrative) extend RFC-0002's reserved space
(bits 28–31, values 3–15 are reserved). RFC-0002 must be updated to reflect
KB-0006's extended pronoun_type encoding before KB-0006 v1.0.0 release.
```

### 11.3 Anaphora Resolution Features

For anaphora resolution, KB-0006 entries provide:

| Feature | Values | Used For |
|---------|--------|----------|
| Person | 1, 2, 3 | Matching pronoun → antecedent person agreement |
| Number | sg, dl, pl | Matching pronoun → antecedent number agreement |
| Gender | masc, fem, common | Matching pronoun → antecedent gender agreement |
| Animacy | human, non-human, abstract | Refining antecedent search (from KB-0001) |
| Prominence | subject, object, topic | Determining likely antecedent (from MOD-05) |

---

## 12. Pronoun Attachment & Clitic Behavior

### 12.1 Pronoun Attachment Rules

When a pronoun attaches as a suffix, MOD-03 must identify and strip it before KB-0006 lookup:

```pseudo
Algorithm: identify_attached_pronoun
Input: inflected_word (string), host_type ("verb" | "noun" | "particle")
Output: (stem, PronounEntry | null)

1. Establish candidate pronoun suffixes:
   a. If host_type == "verb":
      - Check subject pronoun suffixes first (14 forms):
        -تُ, -نَا, -تَ, -تِ, -تُمَا, -تُمْ, -تُنَّ,
        -َتْ, -َا, -َتَا, -ُوا, -ْنَ
      - Check object pronoun suffixes second (12 forms):
        -نِي, -نَا, -كَ, -كِ, -كُمَا, -كُمْ, -كُنَّ,
        -هُ, -هَا, -هُمَا, -هُمْ, -هُنَّ
   b. If host_type == "noun":
      - Check possessive pronoun suffixes (same 12 as object):
        -ي (for 1s), -نَا, -كَ, -كِ, etc.
   c. If host_type == "particle":
      - Check fused particle+pronoun forms:
        إِنِّي (inna + y), إِنَّنِي (inna + nī),
        لَهُ (li + hu), فِيهِ (fī + hi), etc.

2. For each candidate suffix:
   a. Try stripping it from the word tail.
   b. Verify the remaining stem is a valid word form.
   c. If valid → return (stem, PronounEntry).

3. Return (inflected_word, null) if no pronoun found.
```

### 12.2 Attachment Order (Stacking)

Multiple pronouns can stack on a single verb:

```diff
  Single:       كَتَبْتُ (katabtu)         — verb + subject pronoun
  Double:       كَتَبْتُهُ (katabtuhu)      — verb + subject + object
  Triple:       أَعْطَيْتُكُمُوهُ (aʿṭaytukumūhu) — verb + subj + indir obj + dir obj

  Attachment order:
  [Verb Stem] + [Subject Pronoun] + [Indirect Object] + [Direct Object]
```

KB-0006 stores each individual pronoun form; the stacking order is handled by MOD-03 clitic analysis.

---

## 13. Anaphora & Pronoun Resolution

### 13.1 Resolution Algorithm

```pseudo
Algorithm: resolve_pronoun_antecedent
Input: pronoun (PronounEntry), tokens (TokenInfo[]), context (syntactic_context)
Output: TokenInfo (antecedent token) | null

1. Extract pronoun features:
   a. person = pronoun.person
   b. number = pronoun.number
   c. gender = pronoun.gender

2. Search backwards through preceding tokens for candidate antecedents:
   a. For each preceding noun, noun phrase, or clause:
      i.   Check person agreement:
           - 1st person pronoun → must match 1st person antecedent
           - 2nd person pronoun → must match 2nd person antecedent
           - 3rd person pronoun → any preceding noun/noun phrase
      ii.  Check number agreement:
           - Singular pronoun → singular noun
           - Dual pronoun → dual noun or two nouns
           - Plural pronoun → plural noun or collective noun
      iii. Check gender agreement:
           - Masculine pronoun → masculine noun or common noun
           - Feminine pronoun → feminine noun
           - Common pronoun → any
      iv.  Score the candidate:
           +3 if person, number, and gender all match
           +2 if person and number match (gender compatible)
           +1 if only person matches
           +0 if person does not match

3. Return highest-scoring antecedent (or null if none found).
```

### 13.2 Special Cases

| Case | Rule | Example |
|------|------|---------|
| **Impersonal pronoun** | Subject included in verb form | يَكْتُبُ (yaktubu) = he writes (implicit 3ms) |
| **Generic subject** | 2nd person can be generic | يَا أَيُّهَا الْإِنْسَانُ (O humankind — addressing all) |
| **Dual ambiguity** | هُمَا can be 3md or 3fd | Context determines gender |
| **Pronoun of separation** | ضمير الفصل between subject and predicate | كَانَ اللَّهُ هُوَ الْغَفُورُ |

---

## 14. Pronoun Matching Algorithm

### 14.1 Primary Algorithm: Fast Path Pronoun Check

```pseudo
Algorithm: fast_path_pronoun_check
Input: token (string), context (syntactic_context)
Output: (boolean, PronounEntry[])

1. Normalize token:
   a. Remove tatweel/kashida if present.
   b. Apply NFKC normalization.
   c. Strip any remaining diacritics for lookup.

2. Hash-index lookup:
   a. Query KB-0006 by pronoun text (detached, demonstrative, relative, etc.).
   b. If found → return (true, PronounEntry).

3. If not found as standalone:
   a. Check if token contains a suffix pronoun:
      i.   Try stripping known pronoun suffixes from the end.
      ii.  If the remaining stem is valid and the suffix matches KB-0006 →
           return (true, PronounEntry).
   b. Check if token is a fused particle+pronoun (e.g., إِنِّي, لَهُ):
      i.   Known compound forms stored in KB-0006.
      ii.  If matched → return (true, PronounEntry).

4. Return (false, []).

Performance target: < 1 μs for common case (standalone pronoun lookup).
```

### 14.2 Secondary Algorithm: Classify Pronoun

```pseudo
Algorithm: classify_pronoun
Input: pronoun (PronounEntry), context (syntactic_context)
Output: PronounClassification

1. Determine role:
   a. If pronoun is attached to a verb → classify as subject or object.
   b. If pronoun is attached to a noun → classify as possessive.
   c. If pronoun is detached → classify as independent/subject.
   d. If pronoun is demonstrative → identify proximity (near/far).
   e. If pronoun is relative → classify as definite/indefinite.

2. Determine grammatical function:
   a. Check sentence type (nominal, verbal, conditional, interrogative).
   b. For relative pronouns: mark start of relative clause.
   c. For interrogative pronouns: mark question type (yes/no, wh-).
   d. For conditional pronouns: mark condition-result structure.

3. Prepare feature vector for downstream modules:
   {
     pronoun_type: ...,
     person: ...,
     number: ...,
     gender: ...,
     role: ...,
     grammatical_function: ...
   }

4. Return PronounClassification.
```

---

## 15. Serialization & Storage

### 15.1 Source Format

```diff
  /knowledge/KB-0006/
  ├── metadata.yaml                     # KB metadata (version, counts)
  ├── personal-attached.yaml            # Attached pronouns (subject, object, possessive)
  ├── personal-detached.yaml            # Detached pronouns (nominative + iyyā series)
  ├── demonstratives.yaml               # Asma al-ishara
  ├── relatives.yaml                    # Definite + indefinite relative pronouns
  ├── interrogatives.yaml               # Interrogative pronouns
  └── conditionals.yaml                 # Conditional pronouns
```

#### Metadata File

```yaml
# metadata.yaml
kb_id: "KB-0006"
title: "Pronouns — Personal, Demonstrative & Relative"
version: "1.0.0"
status: "draft" | "review" | "published"

pronoun_count: 68
pronoun_type_counts:
  personal_attached: 14
  personal_detached: 13
  demonstrative: 13
  relative: 12
  interrogative: 10
  conditional: 6

created_at: "2026-07-15T00:00:00Z"
updated_at: "2026-07-15T00:00:00Z"

sources:
  - name: "Sibawayh, Al-Kitab"
    version: "critical_1988"
  - name: "Ibn Hisham, Mughni al-Labib"
    version: "print_1964"
  - name: "Wright's Arabic Grammar"
    version: "3rd_edition"

checksum_sha256: "f6a7b8c9d0e1..."
maintainers:
  - name: "Dr. [Name]"
    email: "[email]"
    role: "pronoun_editor"
```

### 15.2 Compiled Format (Hash Index)

```diff
  Compiled Pronoun Binary:
  ┌──────────────────────────────────────────────────────────────┐
  │ HEADER                                                       │
  │ ├── magic: "AGOSKB06" (8 bytes)                             │
  │ ├── version: major(2B) + minor(2B) + patch(2B)              │
  │ ├── pronoun_count: u32 (4 bytes)                            │
  │ ├── hot_size: u16 (for hot/cold split)                      │
  │ ├── hash_index_offset: u32 (4 bytes)                        │
  │ ├── entry_table_offset: u32 (4 bytes)                       │
  │ ├── string_table_offset: u32 (4 bytes)                      │
  │ └── checksum: SHA-256 (32 bytes)                            │
  ├──────────────────────────────────────────────────────────────┤
  │ HASH INDEX (O(1) lookup)                                    │
  │ ├── Perfect hash over pronoun text strings                  │
  │ ├── Maps pronoun text → pronoun_id                          │
  │ └── 128 entry bucket table (smaller than KB-0005)           │
  ├──────────────────────────────────────────────────────────────┤
  │ ENTRY TABLE                                                  │
  │ ├── Fixed-size entries (48 bytes each)                      │
  │ │   ├── pronoun_type: u8                                    │
  │ │   ├── person: u8 (0=none, 1=1st, 2=2nd, 3=3rd)          │
  │ │   ├── number: u8 (0=none, 1=sing, 2=dual, 3=plur)       │
  │ │   ├── gender: u8 (0=none, 1=mas, 2=fem, 3=common)       │
  │ │   ├── attachment_type: u8 (0=standalone, 1=suffix)       │
  │ │   ├── attaches_to: u8 (0=any, 1=verb, 2=noun, 3=part)   │
  │ │   ├── sub_type: u8                                       │
  │ │   ├── meaning_offset: u32 (→ string table)               │
  │ │   ├── example_count: u8                                  │
  │ │   └── filler: 4 bytes                                    │
  │ └── ... (pronoun_count entries)                             │
  ├──────────────────────────────────────────────────────────────┤
  │ STRING TABLE                                                 │
  │ ├── Length-prefixed UTF-8 strings                           │
  │ ├── Pronoun forms, meanings, grammar notes, examples        │
  │ └── Referenced by offsets from entry table                  │
  └──────────────────────────────────────────────────────────────┘
```

#### C Struct: Pronoun Entry

```c
struct PronounEntry {
    uint8_t  pronoun_type;               // From PronounType enum
    uint8_t  person;                     // 0=none, 1=1st, 2=2nd, 3=3rd
    uint8_t  number;                     // 0=none, 1=singular, 2=dual, 3=plural
    uint8_t  gender;                     // 0=none, 1=masc, 2=fem, 3=common
    uint8_t  attachment_type;            // 0=standalone, 1=suffix, 2=prefix
    uint8_t  attaches_to;                // 0=any, 1=verb, 2=noun, 3=particle
    uint8_t  sub_type;                   // Sub-category identifier
    uint32_t meaning_offset;             // → string table
    uint8_t  example_count;              // Number of examples
    uint8_t  filler[4];                 // Padding to 48 bytes
};
```

### 15.3 File Packaging

```diff
  KB-0006-v1.0.0.agos-kb              # Compiled pronoun binary
  KB-0006-v1.0.0.agos-kb.sig          # Ed25519 signature
  KB-0006-v1.0.0.agos-kb.sha256       # SHA-256 checksum
  KB-0006-v1.0.0.source.tar.gz        # Source YAML files (optional)
```

### 15.4 Size Budget

| Component | Compact (Level 1) | Full (Level 2) | Notes |
|-----------|-------------------|----------------|-------|
| Hash index | 0.2 MB | 0.3 MB | 128-bucket perfect hash |
| Entry table | 0.05 MB | 0.1 MB | ~80 entries × 48 bytes |
| String table | 0.5 MB | 1.0 MB | Pronoun forms, meanings, examples |
| Example data | 0.15 MB | 0.4 MB | Usage examples |
| Fused form table | 0.1 MB | 0.2 MB | Particle+pronoun compound forms |
| **Total** | **~1 MB** | **~2 MB** | Memory-mapped load |

---

## 16. Versioning & Evolution

### 16.1 Versioning Scheme

KB-0006 follows **Semantic Versioning 2.0.0** (MAJOR.MINOR.PATCH):

| Bump | Criteria | Example | Impact |
|------|----------|---------|--------|
| **MAJOR** | Breaking change to pronoun schema, format change | `1.0.0` → `2.0.0` | Requires KB conversion tool, invalidates caches |
| **MINOR** | Addition of new pronoun forms, new pronoun types, new optional fields | `1.0.0` → `1.1.0` | Backward-compatible; existing IDs remain valid |
| **PATCH** | Corrections to pronoun features, improved examples, typo fixes | `1.0.0` → `1.0.1` | Backward-compatible; no schema changes |

### 16.2 Cross-KB Compatibility

```yaml
cross_kb_compatibility:
  KB-0001: ">= 1.0.0"       # Independent (no root dependency)
  KB-0002: ">= 1.0.0"       # Independent (no wazan dependency)
  KB-0003: ">= 1.0.0"       # Independent (no verb conjugation dependency)
  KB-0004: ">= 1.0.0"       # Independent (no noun pattern dependency)
  KB-0005: ">= 1.0.0"       # Shared fast-path in MOD-04
  KB-0007: ">= 1.0.0"       # Pronoun features referenced
```

### 16.3 Evolution Guidelines

| Operation | Version Bump | Procedure |
|-----------|-------------|-----------|
| Add new pronoun form | MINOR | Add pronoun entry, regenerate hash index |
| Correct pronoun features | PATCH | Edit pronoun features (person/number/gender) |
| Add fused form | MINOR | Add fused particle+pronoun compound entry |
| Remove pronoun | MAJOR | Only for demonstrably incorrect entries |

---

## 17. Quality Requirements

### 17.1 Completeness Targets

| Category | Minimum | Target | Stretch |
|----------|---------|--------|---------|
| Attached pronouns (subject) | 100% | 100% | 100% |
| Attached pronouns (object/possessive) | 100% | 100% | 100% |
| Detached pronouns (nominative) | 100% | 100% | 100% |
| Detached pronouns (accusative/إِيَّايَ series) | 100% | 100% | 100% |
| Demonstratives (all forms) | 100% | 100% | 100% |
| Relative pronouns (definite) | 100% | 100% | 100% |
| Relative pronouns (indefinite) | 90% | 95% | 100% |
| Interrogative pronouns | 100% | 100% | 100% |
| Conditional pronouns | 90% | 95% | 100% |
| Phonological variants | 90% | 95% | 100% |
| Fused particle+pronoun forms | 85% | 90% | 95% |

### 17.2 Accuracy Requirements

| Metric | Requirement | Enforcement |
|--------|-------------|-------------|
| Person assignment | 100% — each pronoun must have correct person | Manual verification |
| Number assignment | 100% — singular/dual/plural must be correct | Manual verification |
| Gender assignment | 100% — masculine/feminine must be correct | Manual verification |
| Attachment type | 100% — standalone/suffix must be correct | Automated check |
| Script form variants | 100% — phonological variants must be correct | Unicode check |
| Unicode normalization | 100% — all Arabic text valid NFC-normalized UTF-8 | Automated encoding check |

### 17.3 Validation Pipeline

```diff
  Pre-commit (local):
  ├── syntax: validate YAML structure
  ├── schema: validate against KB-0006 JSON Schema
  ├── feature_check: verify person/number/gender are valid values
  └── lint: field presence, Arabic-only text for Arabic fields

  CI (automated, per commit):
  ├── structure: file tree matches expected layout
  ├── hash_uniqueness: verify no duplicate pronoun entries
  ├── feature_regression: verify pronoun features produce correct outputs
  ├── compilation: verify hash index compiles without error
  ├── size_budget: verify compiled size ≤ 2 MB
  └── regression: verify 30+ known pronoun usages are correctly classified

  Review (manual, per release):
  ├── sample_check: linguist reviews 10% random pronoun sample
  ├── hotspot_check: review pronouns modified since last version
  └── changelog: verify changelog accuracy
```

### 17.4 Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Fast-path pronoun lookup (standalone) | < 500 ns | Per lookup, average |
| Fast-path pronoun lookup (standalone, p99) | < 2 μs | Per lookup, 99th percentile |
| Attached pronoun suffix identification | < 3 μs | Per suffix check, average |
| Anaphora resolution (10 candidate search) | < 5 μs | With feature matching |
| KB load time | < 5 ms | mmap + verify checksum |
| Memory | ~1–2 MB | RSS |

---

## 18. Example Entries

### 18.1 Attached Subject Pronoun: -نَا (-nā, \"we\")

```json
{
  "id": "KB-0006:personal_attached:1p:نَا",
  "pronoun": "نَا",
  "transliteration": "nā",
  "pronoun_type": "personal_attached",
  "sub_type": "subject",
  "person": 1,
  "number": "plural",
  "gender": "common",
  "attachment_type": "suffix",
  "attaches_to": "verb",
  "script_forms": [
    { "form": "نَا", "context": "after consonant", "example": "كَتَبْنَا" },
    { "form": "نَا", "context": "after vowel (same form)", "example": "كُنَّا" }
  ],
  "aliases": [],
  "meaning": "we (subject)",
  "meaning_ar": "ضمير الرفع المتحرك — ضمير المتكلمين",
  "reference_notes": "Attached to perfect verbs as subject. Also used as possessive on nouns (same form).",
  "examples": [
    { "phrase": "كَتَبْنَا", "transliteration": "katabnā", "translation": "we wrote" },
    { "phrase": "كُنَّا", "transliteration": "kunnā", "translation": "we were" },
    { "phrase": "رَبَّنَا", "transliteration": "rabbanā", "translation": "our Lord (possessive on noun)" }
  ],
  "attestation": { "confidence": "certain" }
}
```

### 18.2 Detached Pronoun: نَحْنُ (naḥnu, \"we\")

```json
{
  "id": "KB-0006:personal_detached:1p:نَحْنُ",
  "pronoun": "نَحْنُ",
  "transliteration": "naḥnu",
  "pronoun_type": "personal_detached",
  "sub_type": "nominative",
  "person": 1,
  "number": "plural",
  "gender": "common",
  "attachment_type": "standalone",
  "attaches_to": null,
  "script_forms": [
    { "form": "نَحْنُ", "context": "standard", "example": "نَحْنُ نَكْتُبُ" }
  ],
  "aliases": [],
  "meaning": "we",
  "meaning_ar": "ضمير الرفع المنفصل — ضمير المتكلمين",
  "reference_notes": "1st person plural detached pronoun; used as subject of nominal sentences or for emphasis",
  "examples": [
    { "phrase": "نَحْنُ نَكْتُبُ", "transliteration": "naḥnu naktubu", "translation": "we write" },
    { "phrase": "نَحْنُ اللَّهُ", "transliteration": "naḥnu llāhu", "translation": "We are God (royal 'we')" }
  ],
  "attestation": { "confidence": "certain" }
}
```

### 18.3 Demonstrative: هٰذَا (hādhā, \"this\")

```json
{
  "id": "KB-0006:demonstrative:ms:هٰذَا",
  "pronoun": "هٰذَا",
  "transliteration": "hādhā",
  "pronoun_type": "demonstrative",
  "sub_type": "near",
  "person": 3,
  "number": "singular",
  "gender": "masculine",
  "attachment_type": "standalone",
  "attaches_to": null,
  "script_forms": [
    { "form": "هٰذَا", "context": "standard (masculine singular near)", "example": "هٰذَا كِتَابٌ" },
    { "form": "هَذَا", "context": "MSA simplified (without dagger alif)", "example": "هَذَا كِتَابٌ" }
  ],
  "aliases": ["هَذَا"],
  "meaning": "this (masculine singular, near)",
  "meaning_ar": "اسم إشارة للمفرد المذكر القريب",
  "reference_notes": "The most common demonstrative in MSA. Used for nearby objects, present people, or abstract concepts being introduced.",
  "examples": [
    { "phrase": "هٰذَا كِتَابٌ", "transliteration": "hādhā kitābun", "translation": "This is a book" },
    { "phrase": "هٰذَا الْكِتَابُ", "transliteration": "hādhā l-kitābu", "translation": "This book" }
  ],
  "attestation": { "confidence": "certain" }
}
```

### 18.4 Relative Pronoun: الَّذِي (alladhī, \"who/which\")

```json
{
  "id": "KB-0006:relative:ms:الَّذِي",
  "pronoun": "الَّذِي",
  "transliteration": "alladhī",
  "pronoun_type": "relative",
  "sub_type": "definite",
  "person": 3,
  "number": "singular",
  "gender": "masculine",
  "attachment_type": "standalone",
  "attaches_to": null,
  "script_forms": [
    { "form": "الَّذِي", "context": "standard", "example": "الَّذِي كَتَبَ" }
  ],
  "aliases": [],
  "meaning": "who, which, that (masculine singular)",
  "meaning_ar": "الاسم الموصول المفرد المذكر",
  "reference_notes": "Must agree with antecedent in gender and number. The relative clause after it contains a resumptive pronoun when the antecedent is the object.",
  "examples": [
    { "phrase": "جَاءَ الَّذِي كَتَبَ", "transliteration": "jāʾa alladhī kataba", "translation": "The one who wrote came" },
    { "phrase": "الْكِتَابُ الَّذِي قَرَأْتُهُ", "transliteration": "al-kitābu alladhī qaraʾtuhu", "translation": "The book that I read (with resumptive -hu)" }
  ],
  "attestation": { "confidence": "certain" }
}
```

### 18.5 Interrogative Pronoun: مَنْ (man, \"who\")

```json
{
  "id": "KB-0006:interrogative:مَنْ",
  "pronoun": "مَنْ",
  "transliteration": "man",
  "pronoun_type": "interrogative",
  "sub_type": "person",
  "person": null,
  "number": null,
  "gender": null,
  "attachment_type": "standalone",
  "attaches_to": null,
  "script_forms": [
    { "form": "مَنْ", "context": "standard", "example": "مَنْ كَتَبَ؟" }
  ],
  "aliases": [],
  "meaning": "who?",
  "meaning_ar": "اسم استفهام — للعاقل",
  "reference_notes": "Interrogative for humans. Also functions as conditional and relative pronoun. Disambiguated by context in MOD-05.",
  "examples": [
    { "phrase": "مَنْ كَتَبَ هٰذَا؟", "transliteration": "man kataba hādhā?", "translation": "Who wrote this?" }
  ],
  "attestation": { "confidence": "certain" }
}
```

---

## 19. Cross-References

### 19.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | KB-0006 in module catalog; fast path |
| SPEC-0001-C3 | Compilation Pipeline (MOD-04) | Fast path pronoun check (Step 3.2) |
| SPEC-0001-C3 | Compilation Pipeline (MOD-05) | Anaphora resolution using pronoun features |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Pronoun features in MOD-04 output |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-2) | Pronoun morpheme types |
| SPEC-0001-C5 | Data Flow & Intermediate Representations (IR-4) | POS features including pronoun types |
| SPEC-0001-C6 | Deployment & Runtime Considerations | KB bundling, size budget |
| SPEC-0001-C8 | Security, Validation & Error Handling | KB integrity verification |
| SPEC-0001-C9 | Performance Targets & Constraints | KB-0006 size (1–2 MB), lookup performance |
| RFC-0002 | Grammar Bytecode Format | Pronoun feature bitmask (bits 28–31) |
| KB-0001 | Roots Database | Independent (pronouns have no roots) |
| KB-0005 | Particles | Shared fast-path; particle+pronoun fused forms |
| KB-0007 | Morphological Features | Feature taxonomy referenced by pronoun entries |

### 19.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (8th C. CE) | Foundational grammar; defines the pronoun classification system |
| Ibn Hisham, Mughni al-Labib (14th C. CE) | Definitive reference on Arabic particles and pronouns |
| Wright's Arabic Grammar (1859) | Western reference for Arabic pronoun grammar |
| Ryding, A Reference Grammar of MSA (2005) | Contemporary reference for pronoun usage in MSA |

---

## Progress Summary

**KB-0006: Pronouns — Personal, Demonstrative & Relative**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Purpose & Scope | ✓ COMPLETE |
| Section 2 | Pronouns in Arabic Grammar | ✓ COMPLETE |
| Section 3 | Data Model | ✓ COMPLETE |
| Section 4 | Pronoun Entry Schema | ✓ COMPLETE |
| Section 5 | Attached Pronouns | ✓ COMPLETE |
| Section 6 | Detached Pronouns | ✓ COMPLETE |
| Section 7 | Demonstrative Pronouns | ✓ COMPLETE |
| Section 8 | Relative Pronouns | ✓ COMPLETE |
| Section 9 | Interrogative Pronouns | ✓ COMPLETE |
| Section 10 | Conditional Pronouns | ✓ COMPLETE |
| Section 11 | Pronoun Inflection Features | ✓ COMPLETE |
| Section 12 | Pronoun Attachment & Clitic Behavior | ✓ COMPLETE |
| Section 13 | Anaphora & Pronoun Resolution | ✓ COMPLETE |
| Section 14 | Pronoun Matching Algorithm | ✓ COMPLETE |
| Section 15 | Serialization & Storage | ✓ COMPLETE |
| Section 16 | Versioning & Evolution | ✓ COMPLETE |
| Section 17 | Quality Requirements | ✓ COMPLETE |
| Section 18 | Example Entries | ✓ COMPLETE |
| Section 19 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–9), RFC-0002, KB-0005, KB-0007.

**Recommended next document:** KB-0007 (Morphological Features) — the feature taxonomy for AGOS morphological analysis.
