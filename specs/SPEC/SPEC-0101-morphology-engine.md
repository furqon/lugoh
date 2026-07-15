---
spec_id: SPEC-0101
title: Morphology Engine — Detailed Implementation Specification
version: 1.0.0
status: Draft
author: AGOS Morphology Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0401: Knowledge Graph Engine
  - RFC-0002: Grammar Bytecode Format
  - ADR-0001: Compiler Architecture Rationale
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---

# SPEC-0101: Morphology Engine — Detailed Implementation Specification

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Internal Component Model](#3-internal-component-model)
4. [MOD-04: MorphologicalParser — Root Extraction Subsystem](#4-mod-04-morphologicalparser--root-extraction-subsystem)
5. [MOD-04: Wazan Identification Subsystem](#5-mod-04-wazan-identification-subsystem)
6. [MOD-04: Feature Extraction Subsystem](#6-mod-04-feature-extraction-subsystem)
7. [MOD-04: Ambiguity Management Subsystem](#7-mod-04-ambiguity-management-subsystem)
8. [MOD-04: School-Specific Behavior](#8-mod-04-school-specific-behavior)
9. [MOD-04: Performance & Optimization](#9-mod-04-performance--optimization)
10. [MOD-05: SyntaxParser — Sentence Segmentation](#10-mod-05-syntaxparser--sentence-segmentation)
11. [MOD-05: Parse Algorithm](#11-mod-05-parse-algorithm)
12. [MOD-05: Syntactic Construction Recognition](#12-mod-05-syntactic-construction-recognition)
13. [MOD-05: Ambiguity & Partial Parsing](#13-mod-05-ambiguity--partial-parsing)
14. [MOD-05: School-Specific Syntax](#14-mod-05-school-specific-syntax)
15. [Cross-Module Interaction](#15-cross-module-interaction)
16. [Testing & Validation Strategy](#16-testing--validation-strategy)
17. [Implementation Guidance](#17-implementation-guidance)
18. [Cross-References](#18-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

SPEC-0101 provides the **detailed implementation specification** for the AGOS Morphology Engine, which comprises two pipeline modules:

- **MOD-04: MorphologicalParser** — Performs sarf (صرف, morphological) analysis on segmented Arabic tokens. Identifies roots, patterns (awzan), parts of speech, and morphological features.
- **MOD-05: SyntaxParser** — Performs nahw (نحو, syntactic) analysis on the morphological output. Identifies sentence structure, grammatical roles, and constituent relationships.

This specification bridges the gap between the **architectural definitions** in SPEC-0001 (Chapters 2–5) and the **actual implementation**. It provides:

- Internal subsystem architecture and component decomposition
- Detailed algorithm specifications beyond the pipeline descriptions in SPEC-0001-C3
- School-specific behavior (Basra, Kufa, Baghdad, Andalus, Modern)
- Implementation guidance for developers
- Testing and validation strategies

### 1.2 Scope

| In Scope | Out of Scope |
|----------|-------------|
| MOD-04 internal subsystem architecture | MOD-06 through MOD-14 specifications |
| MOD-05 internal subsystem architecture | KB content and curation (KB-0001–0007) |
| Root extraction algorithms | Grammar DSL rule authoring (RFC-0001) |
| Wazan matching strategies | Bytecode generation details (RFC-0002) |
| Feature extraction pipeline | GVM execution model (RFC-0003) |
| Arabic syntactic parsing algorithms | Plugin system internals (SPEC-0601) |
| School-specific morphology rules | Explanation engine templates (SPEC-0501) |
| School-specific syntax rules | API gateway concerns |
| Ambiguity representation and pruning | Deployment and operational concerns |
| Internal data structures (not IRs) | Cache management internals |

### 1.3 Relationship to Other Specifications

```diff
  SPEC-0001 ───── Architectural foundation (C2–C5)
    │
    ├── SPEC-0101: Morphology Engine (this document)
    │     ├── MOD-04: MorphologicalParser (detailed implementation)
    │     └── MOD-05: SyntaxParser (detailed implementation)
    │
    ├── SPEC-0201: Rule Engine (planned)
    ├── SPEC-0301: Grammar Runtime
    ├── SPEC-0401: Knowledge Graph Engine
    ├── SPEC-0501: Explanation Engine
    └── SPEC-0601: Plugin System
```

### 1.4 Knowledge Dependencies

| KB | Content | Loaded By | Purpose in MOD-04/05 |
|----|---------|-----------|---------------------|
| KB-0001 | Roots (جذور) | MOD-04 | Root dictionary for extraction and matching |
| KB-0002 | Wazan (أوزان) | MOD-04 | Morphological pattern templates |
| KB-0003 | Verb Forms (تصريف) | MOD-04 | Conjugation paradigms for feature verification |
| KB-0004 | Noun Patterns (أوزان) | MOD-04 | Derived noun specifications |
| KB-0005 | Particles (حروف) | MOD-04 | Fast-path particle identification |
| KB-0006 | Pronouns (ضمائر) | MOD-04 | Fast-path pronoun identification |
| KB-0007 | Features (خصائص) | MOD-04, MOD-05 | Feature taxonomy, agreement rules, constraints |

---

## 2. Architecture Overview

### 2.1 MOD-04 Internal Architecture

MOD-04 (MorphologicalParser) decomposes into **four internal subsystems** that operate sequentially on each stem:

```diff
  MOD-04: MorphologicalParser — Internal Architecture
  ┌────────────────────────────────────────────────────────────────┐
  │                                                               │
  │  Input: SegmentedTokenStream (IR-3)                           │
  │                                                               │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 1: Fast-Path Checker                             ││
  │  │                                                            ││
  │  │  For each stem:                                            ││
  │  │    1a. Hash lookup in KB-0005 (Particles) — O(1)           ││
  │  │    1b. Hash lookup in KB-0006 (Pronouns) — O(1)            ││
  │  │    1c. If found → skip to System 4 (Feature Extraction)    ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 2: Root Extraction (جذر)                         ││
  │  │                                                            ││
  │  │  For stems not identified as particles/pronouns:            ││
  │  │    2a. Known-word lookup (pre-computed index)               ││
  │  │    2b. Triliteral root extraction                           ││
  │  │    2c. Quadriliteral root extraction                        ││
  │  │    2d. Weak root handling (أجوف, ناقص, مثال)                 ││
  │  │    2e. Hamzated root handling (مهموز)                       ││
  │  │    2f. Doubled root handling (مضاعف)                        ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 3: Wazan Identification (وزن)                     ││
  │  │                                                            ││
  │  │    3a. Verb form matching (I–XV)                           ││
  │  │    3b. Noun pattern matching                                ││
  │  │    3c. Weak variant pattern matching                        ││
  │  │    3d. Pattern signature hashing (O(1))                     ││
  │  │    3e. Ambiguity set generation                             ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 4: Feature Extraction & Packaging                 ││
  │  │                                                            ││
  │  │    4a. Morphological feature extraction (per KB-0007)       ││
  │  │    4b. Feature bitfield packing (64-bit, per RFC-0002)      ││
  │  │    4c. Validation against KB-0007 rules                     ││
  │  │    4d. Default application and inference                    ││
  │  │    4e. Evidence trail generation                            ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  Output: MorphologicalAnalysis (IR-4)                          │
  └────────────────────────────────────────────────────────────────┘
```

### 2.2 MOD-05 Internal Architecture

MOD-05 (SyntaxParser) decomposes into **three internal subsystems**:

```diff
  MOD-05: SyntaxParser — Internal Architecture
  ┌────────────────────────────────────────────────────────────────┐
  │                                                               │
  │  Input: MorphologicalAnalysis (IR-4)                           │
  │                                                               │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 1: Sentence Segmentation                         ││
  │  │                                                            ││
  │  │    1a. Boundary detection (punctuation, conjunctions)       ││
  │  │    1b. Sentence type identification                        ││
  │  │    1c. Length validation                                   ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 2: Parse Engine                                  ││
  │  │                                                            ││
  │  │    2a. Verbal sentence parsing (jumlah fi'liyyah)           ││
  │  │    2b. Nominal sentence parsing (jumlah ismiyyah)           ││
  │  │    2c. Special constructions (shart, qasam, etc.)           ││
  │  │    2d. Idafa, wasf, tawkid, badal resolution               ││
  │  │    2e. Morphology × syntax ambiguity combination           ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  ┌────────────────────────────────────────────────────────────┐│
  │  │ Subsystem 3: Output Packaging                              ││
  │  │                                                            ││
  │  │    3a. Parse tree construction                              ││
  │  │    3b. Confidence scoring                                   ││
  │  │    3c. Partial parse generation (on failure)               ││
  │  │    3d. Evidence trail generation                            ││
  │  └────────────────────────────────────────────────────────────┘│
  │                            │                                   │
  │                            ▼                                   │
  │  Output: SyntaxTree (IR-5)                                     │
  └────────────────────────────────────────────────────────────────┘
```

### 2.3 Data Flow Between MOD-04 and MOD-05

```diff
  MOD-04 (MorphologicalParser)
    │
    │  Output: IR-4 (MorphologicalAnalysis)
    │  ├── token_analyses[]
    │  │   ├── token_id
    │  │   ├── stem_analyses[]
    │  │   │   ├── root (KB-0001 reference)
    │  │   │   ├── wazan (KB-0002 reference)
    │  │   │   ├── pos (PartOfSpeech)
    │  │   │   ├── features[] (per KB-0007 taxonomy)
    │  │   │   │   ├── name (e.g., "gender", "case")
    │  │   │   │   ├── value (e.g., "feminine", "accusative")
    │  │   │   │   ├── category (inflectional/derivational/prosodic/orthographic)
    │  │   │   │   ├── confidence (0.0–1.0)
    │  │   │   │   └── source (KB entry or rule ID)
    │  │   │   ├── is_ambiguous
    │  │   │   ├── alternatives[]
    │  │   │   └── evidence[]
    │  │   └── ...
    │  └── metadata
    │
    ▼
  MOD-05 (SyntaxParser) — Consumes IR-4, produces IR-5
    │
    │  Output: IR-5 (SyntaxTree)
    │  ├── trees[]
    │  │   ├── id, tree_type (jumlah_*)
    │  │   ├── root (recursive Constituent)
    │  │   │   ├── node_type, role
    │  │   │   ├── token_ids[]
    │  │   │   ├── children[]
    │  │   │   ├── features {}
    │  │   │   └── implicit (boolean)
    │  │   ├── confidence
    │  │   └── source (school)
    │  └── metadata
    │
    ▼
  MOD-06 (GIRConstructor) — Consumes IR-4 + IR-5
```

### 2.4 Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **KB-first architecture** | All linguistic knowledge is externalized to KBs. The engine contains no hard-coded lists of roots, patterns, or feature values. School-specific behavior is data-driven via KB version selection. |
| **Deterministic analysis** | No randomness, no machine learning, no heuristic ordering that could change between runs. Same input + same KB versions = same output always. |
| **Ambiguity as first-class** | Multiple valid analyses are preserved at every step. Pruning is done only by the Rule Engine (MOD-07) using school-specific rules. |
| **Fast-path before root extraction** | Particles and pronouns (which have no roots) are identified first via O(1) hash lookup. This handles ~15–25% of typical Arabic text tokens before expensive root extraction. |
| **Pattern signatures** | Wazan patterns are pre-computed as u64 hash signatures for O(1) template matching, avoiding expensive character-level alignment in the hot path. |

---

## 3. Internal Component Model

### 3.1 Core Data Structures

The Morphology Engine uses the following internal data structures that extend the IR schemas defined in SPEC-0001-C5:

#### 3.1.1 InternalTokenContext

```yaml
InternalTokenContext:
  description: >
    Extended token context used internally by both MOD-04 and MOD-05.
    Carries additional data beyond the public IR schemas for efficiency.

  fields:
    token_id: integer
    original_text: string
    normalized_text: string
    offsets:
      start: integer
      end: integer

    # From MOD-03
    segmentations: Segmentation[]       # All candidate segmentations

    # Populated by MOD-04 Subsystem 1 (Fast-Path Checker)
    is_particle: boolean
    is_pronoun: boolean
    particle_entry: ParticleEntry | null    # If is_particle
    pronoun_entry: PronounEntry | null      # If is_pronoun

    # Populated by MOD-04 Subsystems 2-4
    root_candidates: RootCandidate[]     # All candidate roots (ordered by confidence)
    wazan_candidates: WazanCandidate[]   # All candidate patterns
    features: FeatureSet[]               # All feature assignments
    feature_bitfield: u64                # Packed 64-bit feature bitfield

    # Populated by MOD-05
    syntactic_roles: SyntacticRole[]     # Possible syntactic roles
    sentence_type: SentenceType | null   # Assigned during parse
    governor: integer | null             # Token index of governing word
    dependents: integer[]                # Token indices of dependent words
```

#### 3.1.2 RootCandidate

```yaml
RootCandidate:
  description: >
    A candidate root extracted from a stem, with metadata about
    how it was derived and its confidence level.

  fields:
    text: string                           # e.g., "كتب"
    root_type: RootType                    # From KB-0001 §5
    radicals: string[]                     # Individual root consonants
    source: string                         # KB-0001 entry ID
    extraction_method: ExtractionMethod    # How this root was derived
    confidence: float                      # 0.0 to 1.0
    verb_forms: integer[]                  # Forms I–XV available for this root
    derived_nouns: string[]                # Available derived noun types

RootType:
  - sound              # صحيح سالم — all radicals are strong letters
  - mithal_wawi        # مثال واوي — initial radical is waw
  - mithal_yai         # مثال يائي — initial radical is ya
  - ajwaf_wawi         # أجوف واوي — middle radical is waw
  - ajwaf_yai          # أجوف يائي — middle radical is ya
  - naqis_wawi         # ناقص واوي — final radical is waw
  - naqis_yai          # ناقص يائي — final radical is ya
  - lafif_mafruq       # لفيف مفروق — first and last radicals are weak
  - lafif_makrun       # لفيف مقرون — middle and last radicals are weak
  - hamzated_first     # مهموز الفاء — first radical is hamza
  - hamzated_middle    # مهموز العين — middle radical is hamza (or medial hamza)
  - hamzated_last      # مهموز اللام — last radical is hamza
  - doubled            # مضاعف — second and third radicals are identical
  - quadriliteral_sound    # رباعي سالم
  - quadriliteral_weak     # رباعي معتل

ExtractionMethod:
  - exact_match        # Stem found in known-words index
  - triliteral         # Standard 3-consonant extraction
  - quadriliteral      # Standard 4-consonant extraction
  - weak_restoration   # Weak root with restored radical
  - weak_stripping     # Weak root with weak letter stripped
  - hamza_pattern      # Hamzated root by pattern
  - geminate_split     # Doubled root by consonant splitting
  - guess              # Low-confidence extraction (when enable_guess=true)
```

#### 3.1.3 WazanCandidate

```yaml
WazanCandidate:
  description: >
    A candidate morphological pattern (wazan) matched against a stem,
    with the specific form and pattern signature.

  fields:
    text: string                           # e.g., "فَعَلَ"
    pattern_signature: u64                 # Pre-computed hash signature
    form: integer | null                   # I–XV for verbs, null for nouns
    category: WazanCategory                # Verb or noun pattern type
    source: string                         # KB-0002 entry ID
    stem_alignment:
      root_slots: integer[]                # Which stem positions map to root consonants
      affix_slots: integer[]               # Which stem positions are affixes
    confidence: float
    conjugation_class: string | null       # For verbs: "sound", "hollow", etc.
    noun_type: string | null              # For nouns: "masdar", "ism_fail", etc.

WazanCategory:
  - verb_form_i
  - verb_form_ii
  - verb_form_iii
  - verb_form_iv
  - verb_form_v
  - verb_form_vi
  - verb_form_vii
  - verb_form_viii
  - verb_form_ix
  - verb_form_x
  - verb_form_xi
  - verb_form_xii
  - verb_form_xiii
  - verb_form_xiv
  - verb_form_xv
  - noun_masdar
  - noun_ism_fail          # Active participle
  - noun_ism_maful         # Passive participle
  - noun_sifat_musabbaha   # Resembling adjective
  - noun_ism_makan         # Noun of place
  - noun_ism_zaman         # Noun of time
  - noun_ism_alah          # Noun of instrument
  - noun_sighah_mubalaghah # Intensive form
  - noun_tafdil            # Elative (comparative/superlative)
  - noun_nisbah            # Relative adjective (nisba)
  - noun_broken_plural     # Broken plural template
  - noun_other
```

#### 3.1.4 FeatureSet

```yaml
FeatureSet:
  description: >
    A complete set of morphological features for one analysis,
    including the packed bitfield representation.

  fields:
    features: FeatureAssignment[]
    bitfield: u64                            # Packed per KB-0007 §10
    validated: boolean                       # Whether validated against KB-0007 rules
    validation_errors: ValidationError[]     # If validation failed
    confidence: float                        # Aggregate confidence across all features

FeatureAssignment:
  name: string                               # Feature name (per KB-0007)
  value: string                              # Feature value (per KB-0007)
  category: FeatureCategory                  # From KB-0007 taxonomy
  bit_position: integer                      # Position in 64-bit bitfield
  bit_width: integer                         # Width in bits
  confidence: float                          # Per-feature confidence
  source: string                             # KB entry, rule ID, or inference

ValidationError:
  code: string
  message: string
  feature_name: string
  severity: "error" | "warning"
```

### 3.2 Configuration Model

Both MOD-04 and MOD-05 accept configuration that extends the base StageConfig:

#### 3.2.1 MOD-04 Configuration

```yaml
MorphologicalParserConfig:
  extends: StageConfig

  fields:
    school: "basra" | "kufa" | "baghdad" | "andalus" | "modern"
    max_morphological_analyses: integer      # Default: 32
    enable_guess: boolean                    # Default: false (allow guess for unknown stems)
    enable_weak_heuristics: boolean          # Default: true
    enable_hamza_heuristics: boolean         # Default: true
    enable_geminate_heuristics: boolean      # Default: true
    max_root_candidates: integer             # Default: 8
    max_wazan_candidates: integer            # Default: 16
    known_words_path: string | null          # Custom dictionary path
    lazy_kb_loading: boolean                 # Default: false (load all KBs at init)
```

#### 3.2.2 MOD-05 Configuration

```yaml
SyntaxParserConfig:
  extends: StageConfig

  fields:
    school: "basra" | "kufa" | "baghdad" | "andalus" | "modern"
    max_parse_trees: integer                 # Default: 8
    max_sentence_length: integer             # Default: 200 (tokens)
    enable_partial_parse: boolean            # Default: true
    enable_ellipsis_detection: boolean       # Default: true
    enable_shart_parsing: boolean            # Default: true
    enable_qasam_parsing: boolean            # Default: true
    strict_idafa: boolean                    # Default: false (Kufa school often uses stricter idafa rules)
```

### 3.3 School Configuration Mapping

Each grammar school has specific settings that affect morphology and syntax analysis:

```yaml
SchoolConfig:
  basra:
    description: "البصرة — the dominant classical school, strictest rules"
    morphology:
      verb_form_priority: [I, II, III, IV, V, VI, VII, VIII, IX, X]
      prefer_assimilated_as_form_i: true
      hollow_verb_restoration: "default"      # Restore waw/ya in hollow roots
      naqis_vowel_harmony: true                # Apply vowel harmony for defective verbs
    syntax:
      default_sentence_type: "jumlah_fi'liyyah"
      verb_before_subject_required: false      # Nominal sentences can start with noun
      idafa_strictness: "strict"               # Must be indefinite + definite
      mubtada_must_be_definite: true
      khabar_may_precede_mubtada: false

  kufa:
    description: "الكوفة — more flexible, accepts more constructions"
    morphology:
      verb_form_priority: [I, II, III, IV, V, VI, VIII, VII, IX, X]
      prefer_assimilated_as_form_i: true
      hollow_verb_restoration: "default"
      naqis_vowel_harmony: true
    syntax:
      default_sentence_type: "jumlah_fi'liyyah"
      verb_before_subject_required: false
      idafa_strictness: "relaxed"              # Allows some exceptions
      mubtada_must_be_definite: false          # Allows indefinite mubtada'
      khabar_may_precede_mubtada: true

  baghdad:
    description: "بغداد — eclectic school, synthesizes Basra and Kufa"
    morphology:
      verb_form_priority: [I, II, III, IV, V, VI, VII, VIII, IX, X]
      prefer_assimilated_as_form_i: true
      hollow_verb_restoration: "default"
      naqis_vowel_harmony: true
    syntax:
      default_sentence_type: "jumlah_fi'liyyah"
      verb_before_subject_required: false
      idafa_strictness: "moderate"
      mubtada_must_be_definite: true
      khabar_may_precede_mubtada: true

  andalus:
    description: "الأندلس — Western school, influenced by Berber/Romance contact"
    morphology:
      verb_form_priority: [I, II, IV, III, V, VI, VIII, VII, IX, X]
      prefer_assimilated_as_form_i: true
      hollow_verb_restoration: "default"
      naqis_vowel_harmony: false               # Less vowel harmony in defective verbs
    syntax:
      default_sentence_type: "jumlah_fi'liyyah"
      verb_before_subject_required: true       # Verb-subject order preferred
      idafa_strictness: "moderate"
      mubtada_must_be_definite: true
      khabar_may_precede_mubtada: false

  modern:
    description: "Modern Standard Arabic (MSA) — relaxed, influenced by Western grammar"
    morphology:
      verb_form_priority: [I, II, III, IV, V, VI, VII, VIII, IX, X]
      prefer_assimilated_as_form_i: true
      hollow_verb_restoration: "default"
      naqis_vowel_harmony: true
    syntax:
      default_sentence_type: "jumlah_fi'liyyah"
      verb_before_subject_required: false
      idafa_strictness: "relaxed"
      mubtada_must_be_definite: false
      khabar_may_precede_mubtada: true
```

---

## 4. MOD-04: MorphologicalParser — Root Extraction Subsystem

### 4.1 Overview

The Root Extraction Subsystem (Subsystem 2) identifies the triliteral or quadriliteral consonantal root from a given stem. This is the core morphological operation in Arabic grammar — nearly every word is derived from a root by applying a pattern (wazan).

### 4.2 Known Word Lookup (Fast Path)

Before attempting algorithmic root extraction, the system checks a **known words index** — a pre-computed hash map of all stems in KB-0001 through KB-0004 mapped to their roots.

```pseudocode
Algorithm: known_word_lookup
Input: stem (string)
Output: RootCandidate[] or empty

1.  stem_normalized = normalize_stem(stem)
    // Normalization: strip definite article ال, normalize final ya/alif, etc.

2.  if stem_normalized in known_words_index:
        entry = known_words_index[stem_normalized]
        return [RootCandidate(
            text = entry.root,
            root_type = entry.root_type,
            source = entry.source,
            extraction_method = "exact_match",
            confidence = 1.0
        )]

3.  // Check with alternative normalizations
    for alt in generate_alternative_normalizations(stem_normalized):
        if alt in known_words_index:
            entry = known_words_index[alt]
            return [RootCandidate(
                text = entry.root,
                root_type = entry.root_type,
                source = entry.source,
                extraction_method = "exact_match",
                confidence = 0.95
            )]

4.  return empty  // Fall through to algorithmic extraction
```

### 4.3 Triliteral Root Extraction

For a stem that is not found in the known words index, the system applies algorithmic root extraction.

```pseudocode
Algorithm: extract_triliteral_root
Input: stem (string), root_type_hint (RootType | null)
Output: RootCandidate[]

Context:
  Arabic triliteral roots consist of 3 consonants (radicals).
  Non-root letters (حروف الزيادة) must be identified and removed.
  The ten extra letters are: سألتمونيها (sa-altamuuniyha):
    س, أ, ل, ت, م, و, ن, ي, ه, ا

Step 1: Identify Affixes
  1.1  Strip any remaining proclitics (should already be stripped by MOD-03)
  1.2  Identify possible prefixes:
       - أ (alif): Form IV prefix, interrogative, 1st person marker
       - ت (ta): Form V/VI prefix, 2nd/3rd feminine marker
       - ي (ya): 3rd masculine marker
       - ن (nun): 1st person plural marker
       - م (mim): Form VII/VIII prefix component
       - س (sin): Future marker (should have been stripped by MOD-03)
       - ا (alif): Form VII/VIII, IX prefix
  1.3  Identify possible suffixes:
       - وا (waw-alif): 3rd masculine plural perfect
       - ون (waw-nun): 3rd masculine plural imperfect, nominative
       - ين (ya-nun): 3rd masculine plural accusative/genitive
       - ان (alif-nun): Dual nominative
       - ين (ya-nun): Dual accusative/genitive
       - ات (alif-ta): Feminine plural
       - نا (nun-alif): 1st person plural
       - ت (ta): 2nd person / 3rd feminine singular perfect
       - ن (nun): Feminine plural / energetic
       - ة (ta-marbuta): Feminine marker
       - ي (ya): 1st person singular / nisba
  1.4  Strip identified affixes to produce a candidate bare stem

Step 2: Extract Root Consonants
  2.1  From the bare stem, extract 3 consonants in order.
  2.2  Apply the following rules for weak letters:
       - Initial و (waw) or ي (ya) → may be a radical (mithal) or an affix
       - Medial ا (alif) → indicates hollow root (أجوف), restore to و or ي
       - Final ا (alif), ى (alif maqsura), ي (ya) → indicate defective root (ناقص)
  2.3  For doubled verbs (مضاعف):
       - If C2 == C3 in the stem, geminate may be split
       - e.g., مَدَّ → م د د (m-d-d)

Step 3: Match Against KB-0001
  3.1  For each candidate root (up to max_root_candidates):
  3.1.1  Look up in KB-0001 trie
  3.1.2  If found: retrieve root type, verb forms, derived nouns
  3.1.3  If not found: try alternative extraction (different affix analysis)

Step 4: Return Candidates
  4.1  Return all valid root candidates ordered by confidence
  4.2  Confidence scoring:
       - exact_match (known words): 1.0
       - standard extraction, root in KB-0001: 0.9
       - standard extraction, root in KB-0001, different form: 0.7
       - heuristic extraction, root in KB-0001: 0.5
       - guess (enable_guess=true): 0.2
```

### 4.4 Weak Root Handling

Weak roots (roots containing و, ي, or ا) require special extraction rules:

#### 4.4.1 Hollow Roots (أجوف — Middle Radical Weak)

```pseudocode
Algorithm: handle_hollow_root
Input: stem (string), candidate_middle_vowel (char | null)
Output: RootCandidate | null

Context:
  Hollow roots have و (waw) or ي (ya) as the middle radical.
  In many conjugated forms, the middle radical is replaced by a long vowel.
  e.g., قال (qāla) < ق-و-ل (q-w-l, "to say")
        باع (bāʿa) < ب-ي-ع (b-y-ʿ, "to sell")

Step 1: Detect Hollow Pattern
  1.1  Check if stem contains medial ا (alif) or ى (alif maqsura)
  1.2  If C1 + alif + C3 pattern detected → likely hollow root
  1.3  Determine whether middle radical is waw or ya:
       - Check KB-0001 for both wawi and yai variants
       - If both exist: prefer based on verb semantics
       - Common defaults: قال ← ق-و-ل, باع ← ب-ي-ع

Step 2: Restore Middle Radical
  2.1  candidate_roots = [
           C1 + "و" + C3,    # Wawi variant
           C1 + "ي" + C3     # Yai variant
       ]
  2.2  For each candidate:
  2.2.1  Look up in KB-0001
  2.2.2  If found: create RootCandidate with root_type = ajwaf_wawi or ajwaf_yai

Step 3: Handle Imperfect Forms
  3.1  Imperfect forms may drop the middle radical entirely:
       e.g., يَقُولُ (yaqūlu) → ق-و-ل (restore waw)
       يَبِيعُ (yabīʿu) → ب-ي-ع (restore ya)
  3.2  Apply reverse transformation rules:
       - يُقَالُ (yuqālu, passive) ← ق-و-ل
       - يَخَافُ (yakhāfu) ← خ-و-ف (kh-w-f, "to fear")
```

#### 4.4.2 Defective Roots (ناقص — Final Radical Weak)

```pseudocode
Algorithm: handle_defective_root
Input: stem (string)
Output: RootCandidate | null

Context:
  Defective roots have و (waw) or ي (ya) as the final radical.
  The final radical may appear as ا (alif), ى (alif maqsura), و (waw), or ي (ya).
  e.g., دَعَا (daʿā) < د-ع-و (d-ʿ-w, "to call")
        رَمَى (ramā) < ر-م-ي (r-m-y, "to throw")

Step 1: Detect Defective Pattern
  1.1  Check if stem ends with ا, ى, و, or ي
  1.2  If yes → likely defective root

Step 2: Determine Final Radical
  2.1  -َى (alif maqsura) → typically ي (ya) radical
  2.2  -َا (alif) → typically و (waw) radical
  2.3  -َو (waw) → indicate و (waw) radical
  2.4  -َى or -َا may represent either → check KB-0001 for both

Step 3: Handle Imperfect/Nominative Forms
  3.1  Imperfect forms may preserve the final radical:
       يَدْعُو (yadʿū) ← د-ع-و
       يَرْمِي (yarmī) ← ر-م-ي
  3.2  Jussive forms drop the final radical:
       لَمْ يَدْعُ (lam yadʿu) ← لَم يَدْعُو → drop و
```

#### 4.4.3 Assimilated Roots (مثال — Initial Radical Weak)

```pseudocode
Algorithm: handle_assimilated_root
Input: stem (string)
Output: RootCandidate | null

Context:
  Assimilated roots have و (waw) or ي (ya) as the first radical.
  The initial weak letter may drop in certain forms.
  e.g., وَجَدَ (wajada) < و-ج-د (w-j-d, "to find")
        يَسَرَ (yasara) < ي-س-ر (y-s-r, "to be easy")

Step 1: Detect Assimilated Pattern
  1.1  Check if stem's first consonant could be the second radical
  1.2  Try prefixing و or ي as virtual first radical

Step 2: Generate Candidates
  2.1  candidate_roots = [
           "و" + C1 + C2,   # Wawi assimilated
           "ي" + C1 + C2    # Yai assimilated
       ]
  2.2  Also check Form IV (أفعل) where initial و drops:
       e.g., أَوْجَدَ (awjada) < و-ج-د (Form IV)
```

### 4.5 Hamzated Root Handling

```pseudocode
Algorithm: handle_hamzated_root
Input: stem (string)
Output: RootCandidate | null

Context:
  Hamzated roots contain hamza (ء) as one of the radicals.
  Hamza may appear as ء, أ, إ, ئ, or ؤ depending on context.

Step 1: Identify Hamza Position
  1.1  Scan stem for hamza variants (ء, أ, إ, ئ, ؤ)
  1.2  Determine position: first (فاء), middle (عين), or last (لام)

Step 2: First Radical Hamza (مهموز الفاء)
  2.1  Examples: أَخَذَ (akhadha) < أ-خ-ذ, أَكَلَ (akala) < أ-ك-ل
  2.2  Extract C1 = hamza, C2, C3 in order

Step 3: Middle Radical Hamza (مهموز العين)
  3.1  Examples: سَأَلَ (sa'ala) < س-أ-ل, قَرَأَ (qara'a) < ق-ر-أ
  3.2  The hamza may sit on a seat (alif, waw, ya): ئ, ؤ
  3.3  Extract: C1, hamza, C3

Step 4: Final Radical Hamza (مهموز اللام)
  4.1  Examples: قَرَأَ (qara'a) < ق-ر-أ, بَدَأَ (bada'a) < ب-د-أ
  4.2  Final hamza may appear as ء (bare hamza)
  4.3  Extract: C1, C2, hamza
```

### 4.6 Doubled Root Handling (Geminate)

```pseudocode
Algorithm: handle_doubled_root
Input: stem (string)
Output: RootCandidate | null

Context:
  Doubled roots (مضاعف) have identical C2 and C3.
  In many forms, the two identical consonants are written as one with shadda.
  e.g., مَدَّ (madda) < م-د-د (m-d-d, "to extend")
        حَبَّ (habba) < ح-ب-ب (h-b-b, "to love")

Step 1: Detect Doubled Pattern
  1.1  Check if C2 == C3 in the stem
  1.2  Check for shadda on final consonant (دَّ → د-د)
  1.3  Check for shadda on middle consonant in Form II patterns

Step 2: Split Geminate
  2.1  The last consonant with shadda → C2 + C3 (same letter)
  2.2  Example: مَدَّ → م-د-د (m-d-d)

Step 3: Handle Form II
  3.1  Form II (فعّل) geminates the middle radical:
       e.g., حَبَّبَ (habbaba) < ح-ب-ب (Form II, "to make beloved")
  3.2  The doubled middle is C2 + C2, not shared with C3
```

### 4.7 Quadriliteral Root Extraction

```pseudocode
Algorithm: extract_quadriliteral_root
Input: stem (string)
Output: RootCandidate[]

Context:
  Quadriliteral roots have 4 consonants. They follow different patterns
  (QI: فَعْلَلَ, QII: فَعْنَلَ, QIII: اِفْعَنْلَلَ).

Step 1: Identify Quadriliteral Pattern
  1.1  If stem has 4 or more consonants → candidate quadriliteral
  1.2  Standard quadriliteral form QI: C₁aC₂C₃aC₄a
  1.3  Lengthened QI: C₁āC₂C₃aC₄a (with long alif after C1)
  1.4  QII (فَعْنَلَ): C₁aC₂C₃C₄aC₅a — 5 consonants
  1.5  QIII (اِفْعَنْلَلَ): prefix + 4 consonants + suffix

Step 2: Extract 4 Consonants
  2.1  Strip known quadrilateral affixes (mostly the same as triliteral)
  2.2  Keep 4 consonants in order
  2.3  Look up in KB-0001 quadriliteral root index
```

### 4.8 Guessing (Low-Confidence Extraction)

When `enable_guess` is true and no root is found:

```pseudocode
Algorithm: guess_root
Input: stem (string)
Output: RootCandidate | null

1.  Extract all consonants from the stem (ignoring weak letters)
2.  If the stem has 3 consonants:
2.1    Propose them as candidate root
2.2    Set confidence = 0.2
2.3    Set extraction_method = "guess"
3.  If the stem has 4 consonants:
3.1    Propose as quadriliteral candidate
3.2    Set confidence = 0.15
4.  Mark evidence: "Stem could not be matched against KB-0001. Proposed root is a guess."
5.  Return candidate (or null if stem has < 3 consonants)
```

---

## 5. MOD-04: Wazan Identification Subsystem

### 5.1 Overview

The Wazan Identification Subsystem (Subsystem 3) determines which morphological pattern (wazan/وزن) applies to the stem given the extracted root. This identifies the verb form (I–XV) or noun pattern.

### 5.2 Pattern Signature Hashing

Patterns are identified using **pre-computed u64 hash signatures** for O(1) lookup:

```pseudocode
Algorithm: compute_pattern_signature
Input: stem (string), root (RootCandidate)
Output: u64

1.  Align root consonants with stem:
    - Map root C1, C2, C3 (or C1-C4) to their positions in the stem
    - The remaining stem characters are pattern affixes

2.  Create a position-annotated stem:
    - Mark each character as: ROOT_RADICAL | PATTERN_AFFIX
    - Record the type of each affix character (long vowel, short vowel, consonant)

3.  Compute 64-bit hash:
    - Bits 0-7:  Pattern length (stem length after root consonant extraction)
    - Bits 8-15: Affix type vector (which affix characters appear)
    - Bits 16-31: Affix position pattern
    - Bits 32-47: Vowel pattern (short vowels between consonants)
    - Bits 48-63: Verb form indicator / noun type indicator

4.  If noun pattern:
    - Set bit 63 = 1 (distinguishes from verb patterns)
    - Use bits 48-62 for noun type encoding

5.  Return u64 signature
```

### 5.3 Verb Form Matching

```pseudocode
Algorithm: match_verb_form
Input: stem (string), root (RootCandidate)
Output: WazanCandidate[]

Step 1: Compute Pattern Signature
  1.1  signature = compute_pattern_signature(stem, root)

Step 2: Look Up in KB-0002
  2.1  candidates = kb_0002.lookup_by_signature(signature)
  2.2  If candidates is not empty:
  2.2.1  Filter by root_type (e.g., hollow roots have different patterns)
  2.2.2  Filter by school (some schools recognize different forms)
  2.2.3  Return filtered candidates

Step 3: Pattern-by-Pattern Matching (Fallback)
  3.1  For each verb form I-XV (in school-specific priority order):
  3.1.1  Get pattern template from KB-0002
  3.1.2  Check if root consonants fit the template slots
  3.1.3  Check if affix characters match the pattern
  3.1.4  If match found: add to candidates

Step 4: Handle Weak Variants
  4.1  If root_type indicates weakness:
  4.1.1  Get weak variant patterns from KB-0002 §8
  4.1.2  Match stem against weak variant templates
  4.1.3  Example: hollow root in Form I → قَالَ pattern
            (C₁āC₃a instead of standard C₁aC₂aC₃a)

Step 5: Score & Order
  5.1  For each candidate wazan:
       - Exact pattern signature match: confidence = 0.95
       - Pattern match with root type adjustment: confidence = 0.85
       - Partial pattern match: confidence = 0.6
  5.2  Order by confidence descending
  5.3  Return ordered candidates (up to max_wazan_candidates)
```

### 5.4 Noun Pattern Matching

```pseudocode
Algorithm: match_noun_pattern
Input: stem (string), root (RootCandidate)
Output: WazanCandidate[]

Step 1: Check Noun Indicators
  1.1  Check for feminine markers: ة (ta-marbuta), اء (alif-mamduda)
  1.2  Check for dual/plural suffixes: ان, ين, ات, ون
  1.3  Check for definite article ال
  1.4  These indicators help narrow the pattern search

Step 2: Compute Pattern Signature
  2.1  signature = compute_pattern_signature(stem, root)
  2.2  Set noun flag (bit 63)

Step 3: Look Up in KB-0002 (Noun Patterns)
  3.1  candidates = kb_0002.lookup_noun_pattern(signature)
  3.2  If candidates not empty, filter by noun_type (masdar, participle, etc.)

Step 4: Masdar (Verbal Noun) Identification
  4.1  Form I masdars are unpredictable — there are ~40+ patterns
  4.2  Try each known Form I masdar pattern
  4.3  Forms II-X have regular masdar patterns:
       - Form II: تَفْعِيل (tafʿīl)
       - Form III: فِعَال (fiʿāl) or مُفَاعَلَة (mufāʿalah)
       - Form IV: إِفْعَال (ifʿāl)
       - Form V: تَفَعُّل (tafaʿʿul)
       - Form VI: تَفَاعُل (tafāʿul)
       - Form VII: اِنْفِعَال (infiʿāl)
       - Form VIII: اِفْتِعَال (iftiʿāl)
       - Form IX: اِفْعِلَال (ifʿilāl)
       - Form X: اِسْتِفْعَال (istifʿāl)

Step 5: Participle Identification
  5.1  Active Participle (اسم فاعل):
       - Form I: فَاعِل (fāʿil)
       - Forms II-X: مُفَعِّل, مُفَاعِل, مُفْعِل, etc.
  5.2  Passive Participle (اسم مفعول):
       - Form I: مَفْعُول (mafʿūl)
       - Forms II-X: مُفَعَّل, مُفَاعَل, مُفْعَل, etc.

Step 6: Other Noun Patterns
  6.1  Noun of Place (اسم مكان): مَفْعَل, مَفْعِل, etc.
  6.2  Noun of Time (اسم زمان): same patterns as place
  6.3  Noun of Instrument (اسم آلة): مِفْعَال, مِفْعَل, مِفْعَلَة, etc.
  6.4  Adjective (صفة): various patterns
  6.5  Elative (اسم تفضيل): أَفْعَل (afʿal)
  6.6  Nisba (نسبة): relative adjective ending in ي (نسْبي)

Step 7: Broken Plural Matching
  7.1  If the noun appears to be plural (context, agreement clues):
  7.1.1  Match against broken plural templates from KB-0004
  7.1.2  Common patterns: فِعَال, فُعُول, أَفْعَال, فَعْلَى, etc.
  7.1.3  Use the broken plural → singular reverse index

Step 8: Score & Order
  8.1  Same scoring approach as verb forms
  8.2  Return ordered candidates
```

### 5.5 Ambiguity in Wazan Identification

Many stems admit multiple wazan interpretations. The system MUST preserve all valid interpretations:

| Stem | Root | Possible Wazans | Reason for Ambiguity |
|------|------|-----------------|---------------------|
| يَضْرِبُ | ض-ر-ب | Form I (imperfect) | Could be indicative or subjunctive |
| يَكْتُبُ | ك-ت-ب | Form I vs. unknown | Without tashkeel, vowel pattern is ambiguous |
| مَكْتَب | ك-ت-ب | Noun of place vs. Form I passive participle | Same pattern for both |
| مُعَلِّم | ع-ل-م | Active participle of Form II | No ambiguity (only one match) |
| كَاتِب | ك-ت-ب | Active participle of Form I | No ambiguity |
| دَار | د-و-ر | Form I hollow verb vs. Form III | Hollow and Form III can appear similar |

---

## 6. MOD-04: Feature Extraction Subsystem

### 6.1 Overview

The Feature Extraction Subsystem (Subsystem 4) determines all morphological features for each analysis and packs them into the 64-bit RFC-0002 feature bitfield.

### 6.2 Feature Extraction Pipeline

```pseudocode
Algorithm: extract_features
Input: stem (string), root (RootCandidate), wazan (WazanCandidate), pos (PartOfSpeech)
Output: FeatureSet

Step 1: Determine Feature Category Based on POS
  1.1  If pos == "particle" or "pronoun":
       -> Fast path: retrieve features from KB-0005/KB-0006 entry
       -> Skip to Step 5

  1.2  If pos == "verb":
       -> Extract verb-specific features (tense, mood, voice, person, etc.)

  1.3  If pos == "noun" or "adjective":
       -> Extract noun-specific features (case, state, gender, number, etc.)

Step 2: Extract Verb Features
  2.1  Tense (تحديد الزمن):
  ┌────────────────────────────────────────────────────────────┐
  │  2.1.1  Compare stem against KB-0003 conjugation tables     │
  │  2.1.2  Perfect (ماضي):                                     │
  │         - Suffixes only (no prefixes)                       │
  │         - e.g., كَتَبَ, كَتَبْتُ, كَتَبْنَا                  │
  │  2.1.3  Imperfect (مضارع):                                  │
  │         - Prefixes + possible suffixes                      │
  │         - e.g., يَكْتُبُ, تَكْتُبِينَ, نَكْتُبُ               │
  │  2.1.4  Imperative (أمر):                                   │
  │         - No overt subject (2nd person)                     │
  │         - e.g., اُكْتُبْ, اُكْتُبُوا, اُكْتُبِي                │
  └────────────────────────────────────────────────────────────┘

  2.2  Person (الشخص):
  ┌────────────────────────────────────────────────────────────┐
  │  2.2.1  Identify person from verb affixes:                  │
  │  2.2.2  First person: تُ (perfect), أ-nun/نَ (imperfect)    │
  │  2.2.3  Second person: تَ/تِ/تُمْ (perfect), تَ/تِ prefix   │
  │  2.2.4  Third person: no overt marker (perfect), يَ prefix  │
  └────────────────────────────────────────────────────────────┘

  2.3  Gender (الجنس):
  ┌────────────────────────────────────────────────────────────┐
  │  2.3.1  Masculine: default (no overt feminine marker)       │
  │  2.3.2  Feminine: ت suffix (perfect), ي/ت prefix (imperfect) │
  └────────────────────────────────────────────────────────────┘

  2.4  Number (العدد):
  ┌────────────────────────────────────────────────────────────┐
  │  2.4.1  Singular: no dual/plural marker                     │
  │  2.4.2  Dual: ا (perfect suffix), ان (imperfect suffix)     │
  │  2.4.3  Plural: وا/تُنَّ (perfect), ون/ين (imperfect)       │
  └────────────────────────────────────────────────────────────┘

  2.5  Mood (الحال — imperfect only):
  ┌────────────────────────────────────────────────────────────┐
  │  2.5.1  Indicative (رفع): final u (ضمة)                    │
  │  2.5.2  Subjunctive (نصب): final a (فتحة)                  │
  │  2.5.3  Jussive (جزم): no final vowel (سكون)               │
  │  2.5.4  Energetic I (تأكيد): نّ or نَ suffix               │
  │  2.5.5  Energetic II (تأكيد مشدد): نَّ suffix              │
  │  Note: Without tashkeel, mood is ambiguous. The Rule Engine │
  │  (MOD-07) resolves mood based on governing particles.      │
  └────────────────────────────────────────────────────────────┘

  2.6  Voice (المبني للمعلوم / المجهول):
  ┌────────────────────────────────────────────────────────────┐
  │  2.6.1  Active (معلوم): fatha on penultimate (perfect)     │
  │  2.6.2  Passive (مجهول): damma on penultimate (perfect)    │
  │         e.g., كُتِبَ (kutiba, "it was written") vs.        │
  │                كَتَبَ (kataba, "he wrote")                  │
  │  2.6.3  Imperfect active: fatha on prefix vowel            │
  │  2.6.4  Imperfect passive: damma on prefix vowel           │
  │         e.g., يُكْتَبُ (yuktabu) vs. يَكْتُبُ (yaktubu)    │
  └────────────────────────────────────────────────────────────┘

  2.7  Verb Form (Form I–XV):
  ┌────────────────────────────────────────────────────────────┐
  │  2.7.1  Directly from wazan.form                            │
  └────────────────────────────────────────────────────────────┘

  2.8  Transitivity:
  ┌────────────────────────────────────────────────────────────┐
  │  2.8.1  Look up root in KB-0001 for inherent transitivity   │
  │  2.8.2  Verb form may change transitivity:                 │
  │         - Form II often makes intransitive → transitive     │
  │         - Form VII often makes transitive → intransitive    │
  │  2.8.3  Set confidence lower for form-induced changes       │
  └────────────────────────────────────────────────────────────┘

Step 3: Extract Noun Features
  3.1  Gender (الجنس):
  ┌────────────────────────────────────────────────────────────┐
  │  3.1.1  Masculine: default                                │
  │  3.1.2  Feminine markers: ة (ta-marbuta), اء (alif ext)   │
  │  3.1.3  Some nouns are inherently feminine (no marker):    │
  │         e.g., شمس (shams, sun), نار (nār, fire)           │
  │  3.1.4  Check KB-0004 noun pattern for inherent gender    │
  └────────────────────────────────────────────────────────────┘

  3.2  Number (العدد):
  ┌────────────────────────────────────────────────────────────┐
  │  3.2.1  Singular: no plural marker                        │
  │  3.2.2  Dual: ان (nominative), ين (accusative/genitive)   │
  │  3.2.3  Sound masculine plural: ون (nom), ين (acc/gen)    │
  │  3.2.4  Sound feminine plural: ات (āt)                    │
  │  3.2.5  Broken plural: check KB-0004 broken plural index  │
  └────────────────────────────────────────────────────────────┘

  3.3  State (الحالة — definiteness):
  ┌────────────────────────────────────────────────────────────┐
  │  3.3.1  Definite (معرفة): ال prefix → state = "definite"   │
  │  3.3.2  Indefinite (نكرة): tanwin (ـًـٌـٍ) → indefinite    │
  │  3.3.3  Construct (مضاف): no ال, no tanwin → "definite"   │
  │         (because the following genitive makes it definite) │
  │  3.3.4  Without tashkeel: default to "indefinite" (low     │
  │         confidence; syntax will determine)                 │
  └────────────────────────────────────────────────────────────┘

  3.4  Case (الإعراب):
  ┌────────────────────────────────────────────────────────────┐
  │  3.4.1  Nominative (رفع): u/damma/ـُ (or و/ون for plurals) │
  │  3.4.2  Accusative (نصب): a/fatha/ـَ (or ا/ين for plurals)│
  │  3.4.3  Genitive (جر): i/kasra/ـِ (or ين for plurals)     │
  │  3.4.4  Diptote nouns: limited to nominative + accusative │
  │         (no genitive kasra; genitive is fatha)             │
  │  3.4.5  Without tashkeel: default to nominative (low       │
  │         confidence; syntax will determine actual case)     │
  └────────────────────────────────────────────────────────────┘

  3.5  Noun Type:
  ┌────────────────────────────────────────────────────────────┐
  │  3.5.1  From wazan_category: masdar, ism fa'il, etc.      │
  └────────────────────────────────────────────────────────────┘

Step 4: Extract Particle/Pronoun Features
  4.1  For particles (from KB-0005):
       - particle_type (preposition, conjunction, subjunctive, etc.)
       - governance (case it assigns, mood it governs)
       - disambiguation_hint

  4.2  For pronouns (from KB-0006):
       - pronoun_type (personal_attached, personal_detached, demonstrative, etc.)
       - person, number, gender
       - attachment_type (standalone, suffix, prefix)

Step 5: Pack Features into Bitfield
  5.1  Apply pack_features algorithm (KB-0007 §10.3)
  5.2  For each feature, set its value in the appropriate bit range
  5.3  Result: 64-bit feature bitfield

Step 6: Validate Against KB-0007 Rules
  6.1  Apply validation rules (VAL-001 through VAL-030):
       - Feature existence check
       - Feature applicability check (is this feature valid for this POS?)
       - Value range check
       - Reserved bits check
  6.2  Record any validation errors

Step 6b: Pack Prosodic & Orthographic Features (from Upstream Modules)
  6b.1 Note: MOD-04 does NOT extract prosodic or orthographic features itself.
       These are extracted by upstream modules and passed through IR-3:

  6b.2  Orthographic features (has_shadda, has_madd, has_hamza):
        - Extracted by MOD-01 (UnicodeValidator) during character scanning
        - Shadda (U+0651) on any character → has_shadda = true
        - Madd (U+0653, U+0654, or madd on alif) → has_madd = true
        - Hamza (ء, أ, إ, ئ, ؤ) → has_hamza = true
        - MOD-04 reads these from the token's upstream metadata

  6b.3  Prosodic features (stress_pattern, syllable_count):
        - Extracted by MOD-02 (PhonologicalProcessor, future module)
        - Stress pattern is determined by syllable structure
        - Syllable count is computed from vowel sequences
        - MOD-04 reads these from the pre-computed token features
        - Until MOD-02 is implemented, default to stress_pattern = "unknown"
          and syllable_count = 0

  6b.4  MOD-04 packs these 5 features into the bitfield at the positions
        defined by KB-0007 §9–10 (bits 40–49) without modifying them

Step 7: Apply Defaults and Inferences
  7.1  Apply inference rules (INF-001 through INF-015):
       - Default values per POS
       - Cross-feature inference (e.g., broken plural → feminine singular)
  7.2  Set confidence for inferred features = 0.3 (modifiable by rules)

Step 8: Return FeatureSet
  8.1  Return FeatureSet with features, bitfield, validation status
```

### 6.3 Part-of-Speech Classification

The POS is determined based on which subsystem identified the stem:

```yaml
PartOfSpeech:
  classification_rules:
    - "particle":   Found in KB-0005 (Subsystem 1)
    - "pronoun":    Found in KB-0006 (Subsystem 1)
    - "verb":       Wazan category indicates verb form (Subsystem 3)
    - "noun":       Wazan category indicates noun pattern (Subsystem 3)
    - "adjective":  Wazan category indicates adjective/noun pattern
    - "adverb":     Specific noun patterns used adverbially (e.g., فَوْقَ, تَحْتَ)
    - "proper_noun": KB-0001 has proper_noun flag; or context (captialization in Latin transcription, etc.)
    - "preposition": Subtype of particle with harf_jarr governance
    - "conjunction": Subtype of particle (coordinating or subordinating)
    - "interrogative": Subtype of particle with interrogative flag
    - "unknown":     No classification possible
```

### 6.4 Feature Confidence Model

```yaml
ConfidenceModel:
  description: >
    Each feature assignment carries a confidence score.
    Scores aggregate from multiple signals:
    - KB entry authority
    - Extraction method reliability
    - Presence of overt morphological markers

  score_table:
    1.0: "Definitive — directly from KB entry (particles, pronouns, known words)"
    0.95: "Strong — morphological marker is explicit (e.g., dual suffix ان)"
    0.9: "High — pattern match with exact signature"
    0.85: "High — pattern match with root type adjustment"
    0.7: "Moderate — pattern match without tashkeel"
    0.6: "Moderate — pattern match with weak root heuristics"
    0.5: "Low — root extraction via heuristics"
    0.3: "Low — inferred by default/inference rule"
    0.2: "Very low — guess mode"
    0.1: "Minimal — fallback only"

  aggregation:
    method: "minimum_confidence"
    description: >
      The aggregate confidence for an entire analysis is the minimum
      confidence across all its feature assignments. This ensures
      that one weak feature doesn't get masked by several strong ones.
```

---

## 7. MOD-04: Ambiguity Management Subsystem

### 7.1 Ambiguity Sources

Morphological ambiguity arises from multiple sources:

| Source | Example | Ambiguity Count |
|--------|---------|----------------|
| **Homograph stems** | عين (ʿayn) = eye, spring, self, to appoint | 4+ |
| **Unvocalized text** | كتب (ktb) = kataba (he wrote), kutiba (it was written), kutub (books) | 3+ |
| **Dual POS** | ضرب (ḍrb) = ḍaraba (he hit, verb) or ḍarb (hitting, noun) | 2+ |
| **Weak root ambiguity** | قال (qāl) = q-w-l or q-y-l | 2 |
| **Multiple verb forms** | اكتب (ktb) = Form I imperative vs. Form VIII imperative | 2 |
| **Broken plural** | كتب (ktb) = books (plural of كتاب) or he wrote | 2+ |

### 7.2 Ambiguity Set Generation

```pseudocode
Algorithm: generate_ambiguity_set
Input: stem (string), root_candidates[], wazan_candidates[]
Output: MorphologicalAnalysis[]

1.  combinations = []

2.  For each root_candidate in root_candidates:
    2.1  For each wazan_candidate in wazan_candidates:
    2.1.1  If wazan_candidate is compatible with root_candidate.root_type:
    2.1.2    analysis = build_analysis(stem, root_candidate, wazan_candidate)
    2.1.3    combinations.append(analysis)

3.  // Also consider the case where the stem is a particle or pronoun
    // This was already handled in Subsystem 1, but add as alternative if
    // the stem could also be analyzed morphologically
    // e.g., مَا (mā) could be a particle ("what") or a noun ("thing")

4.  Order combinations by aggregate confidence descending

5.  If combinations.length > max_morphological_analyses:
    5.1  Truncate to max_morphological_analyses
    5.2  Mark token as ambiguous
    5.3  Record MAX_ANALYSES_EXCEEDED evidence

6.  If combinations.length == 0:
    6.1  Create a single "unknown" analysis
    6.2  Mark token as unknown

7.  Return combinations
```

### 7.3 Ambiguity Propagation to MOD-05

The ambiguity set is passed to MOD-05 through the IR-4 `stem_analyses[].alternatives[]` field. MOD-05 must consider each alternative morphology when constructing parse trees.

---

## 8. MOD-04: School-Specific Behavior

### 8.1 Morphological Differences Between Schools

While all schools agree on the core morphological system, there are minor differences in how certain forms are analyzed:

| Feature | Basra | Kufa | Baghdad | Andalus | Modern |
|---------|-------|------|---------|---------|--------|
| **Form I masdar predictability** | 40+ patterns | 35+ | 38+ | 40+ | 40+ |
| **Form VII formation** | Standard | Accepts more roots | Standard | Standard | Standard |
| **Form VIII assimilated verbs** | Strict rules | Relaxed | Moderate | Moderate | Relaxed |
| **Assimilated verb (waw drop)** | Standard | Standard | Standard | Standard | Standard |
| **إِنْ vs. أَنْ distinction** | Strict | Relaxed | Moderate | Strict | Relaxed |
| **Geminate splitting in Form II** | Standard | Standard | Standard | Standard | Standard |

### 8.2 School Configuration in Root Extraction

```pseudocode
Algorithm: apply_school_morphology_config
Input: school, root_candidates[], wazan_candidates[]
Output: filtered candidates

1.  Get school_config from SchoolConfig[school]

2.  Reorder wazan candidates based on school_config.verb_form_priority

3.  If school == "andalus":
    3.1  Reduce confidence for Form V/VI with weak roots
         (Andalus school is less strict about these)

4.  If school == "kufa":
    4.1  Allow إِنْ as subjunctive particle (Basra only allows أَنْ)
    4.2  Increase confidence for nominal sentence interpretations

5.  If school == "modern":
    5.1  Accept MSA-only forms and loan-word patterns
    5.2  Relax hamza orthography rules
```

---

## 9. MOD-04: Performance & Optimization

### 9.1 Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Fast-path particle check (KB-0005) | < 500 ns per token | O(1) hash lookup |
| Fast-path pronoun check (KB-0006) | < 500 ns per token | O(1) hash lookup |
| Known word lookup | < 1 μs per stem | O(1) hash map |
| Triliteral root extraction | < 5 μs per stem | Algorithmic + KB trie lookup |
| Weak root extraction | < 10 μs per stem | Additional restoration logic |
| Wazan signature lookup | < 1 μs per candidate | O(1) hash signature lookup |
| Full morphological analysis (simple) | < 10 μs per stem | Particle/pronoun/known word |
| Full morphological analysis (complex) | < 100 μs per stem | Weak root + multiple forms |
| Full morphological analysis (guess) | < 200 μs per stem | All heuristics exhausted |

### 9.2 Optimization Strategies

```yaml
Optimizations:
  - strategy: "lazy_kb_loading"
    description: >
      Load KBs on demand rather than at initialization.
      If a corpus contains no verbs, KB-0003 (Verb Forms)
      is never loaded. Reduces cold start memory by ~50%.

  - strategy: "pattern_signature_caching"
    description: >
      Pre-compute and cache pattern signatures for each
      (root_type, wazan_id) pair. Results in a lookup table
      of ~300-450 entries (~3 KB). Avoids computing signatures
      repeatedly for common root-type × wazan combinations.

  - strategy: "known_words_index"
    description: >
      Build a pre-computed hash map of all stems from
      KB-0001 through KB-0004 at compile time.
      Size: ~60,000-100,000 entries (~5-10 MB).
      This is the single highest-impact optimization in MOD-04.

  - strategy: "short_circuit_weak_roots"
    description: >
      If the stem contains no weak letters (ا, و, ي) and
      has exactly 3 consonants, skip all weak root heuristics.
      This handles ~70% of triliteral verb forms.

  - strategy: "trie_memory_mapping"
    description: >
      Memory-map the KB-0001 trie file rather than loading
      it into memory. Uses the OS virtual memory manager
      for paging. Reduces cold start time by ~90%.
```

### 9.3 Memory Budget

```yaml
MemoryBudget:
  MOD-04 only (morphology analysis, school = "basra"):
    - KB-0001 trie (memory-mapped): ~20-80 MB
    - KB-0002 hash index: ~10-40 MB
    - KB-0003 table binary: ~15-30 MB
    - KB-0004 table binary: ~10-30 MB
    - KB-0005 hash index: ~2-5 MB
    - KB-0006 hash index: ~1-2 MB
    - KB-0007 feature map: ~1-2 MB
    - Known words index: ~5-10 MB
    - Pattern signature cache: ~3 KB
    - Working memory (per token): ~1 KB
    -----------------------------------
    Total (compact): ~60 MB
    Total (full): ~200 MB

  MOD-05 only (syntax parsing):
    - School-specific rule tables: ~1-5 MB
    - Parse working memory: O(n²) where n = tokens
    - Constituent structure cache: ~500 KB per tree
```

---

## 10. MOD-05: SyntaxParser — Sentence Segmentation

### 10.1 Overview

The Sentence Segmentation Subsystem (MOD-05 Subsystem 1) divides the token stream into sentence boundaries and identifies each sentence's type.

### 10.2 Boundary Detection

```pseudocode
Algorithm: detect_sentence_boundaries
Input: morphology (MorphologicalAnalysis — IR-4)
Output: Sentence[]

Step 1: Identify Explicit Boundaries
  1.1  Scan tokens for sentence-ending punctuation:
       - Period (.) — standard sentence boundary
       - Question mark (?) — interrogative sentence
       - Exclamation mark (!) — exclamatory
       - Semicolon (;) — weaker boundary
       - Quranic verse markers (۝, ۞) — ayah boundaries

  1.2  Scan for conjunctions that introduce new clauses:
       - وَ (wa, "and") — may join sentences
       - فَ (fa, "and so") — often introduces result clause
       - ثُمَّ (thumma, "then") — sequential boundary
       - بَل (bal, "rather") — contrastive boundary
       - لٰكِنَّ (lākinna, "but") — contrastive boundary

Step 2: Handle Conjunction Ambiguity
  2.1  وَ can join two complete sentences or two elements within one sentence
  2.2  Ambiguity: both interpretations are valid
  2.3  Generate alternative segmentations:
       - Option A: One sentence with coordination
       - Option B: Two separate sentences

Step 3: Build Sentence Objects
  3.1  For each detected sentence:
       - start_token_index: integer
       - end_token_index: integer
       - tokens: TokenSpan
       - confidence: float (1.0 for explicit punctuation, 0.7 for conjunction-based)

Step 4: Return Sentences
  4.1  Return ordered list of Sentence objects
```

### 10.3 Sentence Type Identification

```pseudocode
Algorithm: identify_sentence_type
Input: sentence (Sentence)
Output: SentenceType

Step 1: Examine First Content Token
  1.1  Skip any introductory particles (wa, fa, inna, etc.)
  1.2  Examine the first content token's POS and features

Step 2: Type Classification
  2.1  jumlah_fi'liyyah (جملة فعلية — verbal sentence):
       First content word is a verb (POS == "verb")
       → SentenceType = "jumlah_fi'liyyah"

  2.2  jumlah_ismiyyah (جملة اسمية — nominal sentence):
       First content word is a noun, pronoun, or adjective
       (POS in ["noun", "pronoun", "adjective", "proper_noun"])
       → SentenceType = "jumlah_ismiyyah"

  2.3  jumlah_shartiyyah (جملة شرطية — conditional sentence):
       First word is a conditional particle (إِنْ, لَوْ, إِذَا, etc.)
       → SentenceType = "jumlah_shartiyyah"

  2.4  jumlah_zarfiyyah or other:
       Special constructions
       → SentenceType determined by context

  2.5  Default (for incomplete/informal input):
       → SentenceType = "incomplete" or "unknown"

Step 3: School-Specific Override
  3.1  If school == "kufa" and sentence could be either type:
       - Prefer nominal interpretation
  3.2  If school == "basra" and verb is initial:
       - Force verbal interpretation
```

---

## 11. MOD-05: Parse Algorithm

### 11.1 Overview

The Parse Engine (MOD-05 Subsystem 2) constructs syntactic parse trees from the morphological analysis. It uses a chart-parsing approach inspired by CKY and Earley algorithms, adapted for Arabic sentence structure.

### 11.2 Verbal Sentence Parsing

```pseudocode
Algorithm: parse_verbal_sentence
Input: sentence_tokens (MorphologicalAnalysis[])
Output: SyntaxTree | null

Context:
  Verbal sentence structure (jumlah fi'liyyah):
    [Optional: qad, sa, sawfa, negative particle]
    [Verb (fi'l)] [Subject (fa'il)] [Object(s) (maf'ul)] [Adjunct(s)]

Step 1: Locate the Verb
  1.1  Scan for the first token with POS == "verb"
  1.2  Check for preceding particles that modify the verb:
       - قَد (qad) — perfective/emphatic
       - سَ/سَوْفَ (sa/sawfa) — future
       - لَمْ/لَمَّا (lam/lamma) — past negative (governs jussive)
       - لَنْ (lan) — future negative (governs subjunctive)
       - مَا (mā) — can be negative
  1.3  Create verb constituent with features:
       - verb_features = verb.morphology.features
       - governing_particle = identified particle (or null)

Step 2: Locate the Subject
  2.1  Check if verb has an overt subject pronoun suffix:
       - تُ (tu) — 1st person singular
       - تَ (ta) — 2nd masculine singular
       - تِ (ti) — 2nd feminine singular
       - نَا (nā) — 1st person plural
       - وا (wā/ū) — 3rd masculine plural (suffix, not subject pronoun)
       - If yes → subject is implicit (ضمير مستتر)
  2.2  If no overt subject pronoun:
       - Look for next noun/pronoun in nominative case
       - This is the overt subject (فاعل ظاهر)
  2.3  Check subject-verb agreement:
       - Person: MUST match
       - Gender: SHOULD match (exceptions for non-human plurals → feminine)
       - Number: strict agreement for singular/dual; plural can take feminine singular
  2.4  Create subject constituent

Step 3: Locate Objects
  3.1  Check verb's transitivity:
       - Intransitive (لازم): no object
       - Transitive (متعدٍ): look for object(s)
       - Ditransitive: look for two objects

  3.2  Locate direct object (مفعول به):
       - Look for accusative case noun/pronoun
       - If verb is passive: the "subject" is actually نائب فاعل (deputy subject)
         and there is no direct object

  3.3  Locate other accusative constructions:
       - مَفْعُول مُطْلَق (absolute object): cognate accusative
         e.g., ضَرَبَ ضَرْبًا (daraba ḍarban, "he beat a beating")
       - مَفْعُول فِيه (adverbial): time or place in accusative
       - مَفْعُول لَهُ (reason): purpose in accusative
       - مَفْعُول مَعَهُ (accompaniment): "with" in accusative

Step 4: Build Parse Tree
  4.1  Create clause constituent:
       - type: "jumlah_fi'liyyah"
       - children: [verb, subject, object(s), adjuncts]
  4.2  Assign confidence based on agreement satisfaction

Step 5: Return SyntaxTree
  5.1  Return the built tree (or null if fundamental structure fails)
```

### 11.3 Nominal Sentence Parsing

```pseudocode
Algorithm: parse_nominal_sentence
Input: sentence_tokens (MorphologicalAnalysis[])
Output: SyntaxTree | null

Context:
  Nominal sentence structure (jumlah ismiyyah):
    [Introductory particle (inna, anna, etc.)]
    [Topic (mubtada')] [Comment (khabar)]
    OR:
    [Intro particle] [Noun 1] [Noun 2 / Prepositional Phrase / Verb Clause]

Step 1: Check for Inna and Sisters
  1.1  If sentence begins with إِنَّ, أَنَّ, كَأَنَّ, لٰكِنَّ, لَيْتَ, لَعَلَّ:
  1.1.1  The following noun is in accusative case (accusative subject of inna)
  1.1.2  This triggers case reassignment: mubtada' → accusative

Step 2: Check for Kana and Sisters
  2.1  If sentence begins with كَانَ, صَارَ, لَيْسَ, أَصْبَحَ, أَمْسَى, etc.:
  2.1.1  This transforms the nominal sentence
  2.1.2  Reclassify as verbal (kana is the verb)
  2.1.3  Mubtada' becomes اسم كان (accusative)
  2.1.4  Khabar becomes خبر كان (nominative)

Step 3: Identify Mubtada' (Topic)
  3.1  First noun/pronoun in nominative case
  3.2  Usually definite (ال prefix or proper noun)
  3.3  Check for school-specific rules:
       - Basra: mubtada' MUST be definite
       - Kufa: mubtada' MAY be indefinite (with certain conditions)
       - Modern: less strict

Step 4: Identify Khabar (Comment)
  4.1  The remainder of the sentence after mubtada'
  4.2  Khabar types:
       - Single noun (مفرد): a single word in nominative
         e.g., اَلطَّالِبُ مُجْتَهِدٌ (al-ṭālibu mujtahidun, "the student is diligent")
       - Prepositional phrase (شبه جملة):
         e.g., اَلطَّالِبُ فِي الْفَصْلِ (al-ṭālibu fī l-faṣli, "the student is in the class")
       - Verbal sentence (جملة فعلية): a full verbal clause
         e.g., اَلطَّالِبُ يَدْرُسُ (al-ṭālibu yadrusu, "the student is studying")
  4.3  Mubtada'-Khabar agreement:
       - Both in nominative case
       - Gender agreement (for single-word khabar)
       - Number agreement (exceptions apply)

Step 5: Build Parse Tree
  5.1  Create clause constituent:
       - type: "jumlah_ismiyyah"
       - children: [mubtada', khabar]
  5.2  If inna was present: mark as "jumlah_inna" subtype
```

### 11.4 Conditional Sentence Parsing

```pseudocode
Algorithm: parse_conditional_sentence
Input: sentence_tokens (MorphologicalAnalysis[])
Output: SyntaxTree | null

Context:
  Conditional sentence structure (jumlah shartiyyah):
    [Condition particle (أداة الشرط)]
    [Condition clause (جملة الشرط)]
    [Result clause (جملة الجواب)]
    The result clause is often marked with فَ (fa) or إِذَا (idha)

Step 1: Identify Conditional Particle
  1.1  Check first token for conditional particle:
       - إِنْ (in) — "if" (most common, governs jussive on both verbs)
       - لَوْ (law) — "if" (hypothetical, no jussive government)
       - إِذَا (idhā) — "when/if" (no jussive government)
       - مَنْ (man) — "whoever" (conditional relative)
       - مَا (mā) — "whatever"
       - مَهْمَا (mahmā) — "whatever"
       - مَتَى (matā) — "whenever"
       - أَيْنَ (ayna) — "wherever"
       - أَنَّى (annā) — "however/wherever"
       - كَيْفَمَا (kayfamā) — "however"
       - حَيْثُمَا (ḥaythumā) — "wherever"

Step 2: Parse Condition Clause
  2.1  The clause immediately following the conditional particle
  2.2  If particle is إِنْ: both verbs MUST be in jussive mood
  2.3  If particle is لَوْ: both verbs in perfect tense (no mood government)

Step 3: Parse Result Clause
  3.1  After the condition clause
  3.2  May be introduced by فَ (fa) — required if result is nominal
  3.3  If إِذَا: result clause may begin with فَ or directly

Step 4: Build Parse Tree
  4.1  Create clause constituent:
       - type: "jumlah_shartiyyah"
       - children: [condition_particle, condition_clause, result_clause]
```

---

## 12. MOD-05: Syntactic Construction Recognition

### 12.1 Idafa (Construct State — إضافة)

```pseudocode
Algorithm: recognize_idafa
Input: tokens (MorphologicalAnalysis[])
Output: IdafaPair[]

Context:
  Idafa is a two-noun construction where:
  - First noun (مضاف/mudaf): indefinite in meaning, no ال prefix, no tanwin
  - Second noun (مضاف إليه/mudaf ilayh): definite in genitive case
  - The combination means "X of Y" (e.g., كتاب الطالب = "the book of the student")

Step 1: Scan for Idafa Patterns
  1.1  For each adjacent pair of nouns (N1, N2):
  1.2  Check N1:
       - POS is "noun" or "adjective" used as noun
       - No ال prefix
       - No tanwin (ـًـٌـٍ)
       - If tashkeel available: final vowel omitted (construct state)
  1.3  Check N2:
       - POS is "noun" or "proper_noun"
       - In genitive case (جر)
       - Usually definite (ال prefix or proper noun)

Step 2: Extended Idafa
  2.1  Mudaf may be in construct with a pronoun suffix:
       - e.g., كِتَابُهُ (kitābuhu, "his book")
       - N1 has possessive pronoun suffix → no N2 needed
  2.2  Idafa may chain multiple nouns:
       - e.g., كِتَابُ طَالِبِ الْمَدْرَسَةِ
       - (kitābu ṭālibi l-madrasati, "the book of the student of the school")

Step 3: Record Idafa Relationship
  3.1  Create constituents:
       - mudaf: N1 with feature { construct: true }
       - mudaf_ilayh: N2 with role "mudaf_ilayh"
       - parent: phrase with type "idafa"
```

### 12.2 Adjective Agreement (Wasf — وصف)

```pseudocode
Algorithm: recognize_wasf
Input: tokens (MorphologicalAnalysis[])
Output: AdjectivePair[]

Context:
  An adjective (نعت/na'at) follows a noun (منعوت/man'ut) and agrees with it
  in gender, number, case, and state (definiteness).

Step 1: Scan for Adjective-Noun Pairs
  1.1  For each adjacent pair (N, Adj):
  1.2  Check:
       - N: POS is "noun"
       - Adj: POS is "adjective", appearing immediately after N
       - Agreement: gender, number, case, state ALL match
       - Note: broken plural nouns take feminine singular adjectives
             (e.g., كُتُبٌ كَبِيرَةٌ, kutubun kabīratun, "big books")

Step 2: Record Adjective Pair
  2.1  Create constituents:
       - man'ut: N with role "man'ut"
       - na'at: Adj with role "na'at"
       - parent: phrase with type "wasf"
```

### 12.3 Emphasis (Tawkid — توكيد)

```pseudocode
Algorithm: recognize_tawkid
Input: tokens (MorphologicalAnalysis[])
Output: EmphasisPair[]

Context:
  Emphasizers (ألفاظ التوكيد): نَفْس, عَيْن, كُلّ, جَمِيع, etc.
  These follow a noun and agree with it in case (not necessarily gender/number).

Step 1: Scan for Emphasizer Patterns
  1.1  For each noun-emphasizer pair:
  1.2  Check emphasizer word:
       - Is one of: نَفْس, عَيْن, كُلّ, جَمِيع, كِلَا, كِلْتَا
       - Agrees with preceding noun in case
       - Usually has a pronoun suffix referring to the emphasized noun

Step 2: Record Emphasis
  2.1  Create constituents:
       - mu'akkad: emphasized noun
       - tawkid: emphasizer
```

### 12.4 Apposition (Badal — بدل)

```pseudocode
Algorithm: recognize_badal
Input: tokens (MorphologicalAnalysis[])
Output: AppositionPair[]

Context:
  Badal is apposition: a second noun that explains or replaces the first.
  The badal follows the case of the first noun.

Step 1: Scan for Apposition
  1.1  For adjacent nouns (N1, N2) that are NOT in idafa:
  1.2  Check:
       - N1 and N2 have the same case
       - N2 provides a more specific identification of N1
       - Not separated by conjunction
  1.3  Example: جَاءَ الْخَلِيفَةُ عُمَرُ (jā'a l-khalīfatu ʿumaru,
       "the caliph Umar came") — عُمَر is badal of الْخَلِيفَة

Step 2: Record Apposition
  2.1  Create constituents:
       - mubdal_minhu: the first noun
       - badal: the second noun
```

### 12.5 Exception (Istithna — استثناء)

```pseudocode
Algorithm: recognize_istithna
Input: tokens (MorphologicalAnalysis[])
Output: ExceptionPhrase | null

Context:
  Exception particles: إِلَّا (illā), غَيْرُ (ghayru), سِوَى (siwā), خَلَا (khalā),
  حَاشَا (ḥāshā), عَدَا (ʿadā)

Step 1: Scan for Exception Particle
  1.1  Look for إِلَّا or other exception particles
  1.2  The excepted noun after إِلَّا is in accusative case
       (exception: in negative sentences, إِلَّا's noun may follow the same case)

Step 2: Parse Exception Structure
  2.1  Identify: [general_statement] + [إِلَّا] + [excepted_noun]
  2.2  Create construction with type "istithna"
```

### 12.6 Vocative (Nida — نداء)

```pseudocode
Algorithm: recognize_nida
Input: tokens (MorphologicalAnalysis[])
Output: VocativePhrase | null

Context:
  Vocative particles: يَا (yā), أَيْ (ay), أَيُّهَا (ayyuhā), أَيَّتُهَا (ayyatuhā)

Step 1: Scan for Vocative Particle
  1.1  Look for يَا, أَيْ, أَيُّهَا, أَيَّتُهَا
  1.2  The called noun is in accusative case (when definite) or
       in nominative (when indefinite construct)

Step 2: Parse Vocative
  2.1  Create construction with type "nida"
  2.2  Children: [vocative_particle, called_person]
```

---

## 13. MOD-05: Ambiguity & Partial Parsing

### 13.1 Ambiguity Combination

```pseudocode
Algorithm: combine_morphology_syntax_ambiguity
Input: morphology (MorphologicalAnalysis — IR-4)
Output: SyntaxTree[]

Step 1: For Each Token, Collect All Morphological Analyses
  1.1  For token_id 0 to N:
       - Stem analysis 0..M (from stem_analyses.stem_analysis)
       - Each stem's alternatives[]
       - Also each segmentation's morphological analysis

Step 2: For Each Ambiguity Combination
  2.1  Pick one analysis per token (Cartesian product across tokens)
  2.2  For each combination:
  2.2.1  Try all parse strategies (verbal, nominal, conditional, etc.)
  2.2.2  Some strategies will fail for specific combinations
  2.2.3  Collect all valid parse trees

Step 3: Filter Duplicate Trees
  3.1  Two trees are duplicates if they have the same:
       - Constituent structure (identical roles and relationships)
       - Feature assignments
  3.2  Deduplicate, keeping only one

Step 4: Order by Confidence
  4.1  Primary sort: number of confirming rule applications
  4.2  Secondary sort: aggregate morphological confidence
  4.3  Tertiary sort: school preference

Step 5: Limit to max_parse_trees
  5.1  If > max_parse_trees: keep the highest confidence ones
  5.2  Record MAX_TREES_EXCEEDED evidence

Step 6: Return Ordered Parse Trees
```

### 13.2 Partial Parse (Fallback)

When no complete parse tree can be constructed:

```pseudocode
Algorithm: partial_parse
Input: sentence_tokens (MorphologicalAnalysis[])
Output: SyntaxTree (partial)

1.  Initialize empty constituent tree

2.  Identify Recognizable Constituents:
    2.1  Prepositional phrases: [harf_jarr] + [majrur noun]
    2.2  Verb groups: [verb] + [overt subject] (if present)
    2.3  Known construction fragments:
         - Isolated idafa pairs
         - Known multi-word expressions
         - Particle + noun combinations

3.  For each identifiable constituent:
    3.1  Add as a child of the root
    3.2  Set role to the identified role
    3.3  Set confidence based on how solid the identification is

4.  Mark Remaining Tokens:
    4.1  Tokens that could not be assigned any role → mark as "unknown"
    4.2  Set confidence = 0.1 for these tokens

5.  Create Partial Parse Tree:
    5.1  tree_type = "incomplete"
    5.2  confidence = aggregate_confidence(identified_constituents)
    5.3  Add evidence: PARSE_FAILURE with description of what was found vs. not found

6.  Return partial SyntaxTree
```

### 13.3 Ellipsis Detection (Hadhf — حذف)

```pseudocode
Algorithm: detect_ellipsis
Input: partial_tree (SyntaxTree)
Output: SyntaxTree (with implicit constituents marked)

Step 1: Detect Implicit Subject
  1.1  If sentence_type == "jumlah_fi'liyyah" and no fa'il found:
  1.2  Check if verb suffix indicates implicit subject (ضمير مستتر):
       - 1s: تُ → "I" (implicit)
       - 2ms: تَ → "you" (implicit)
       - 2fs: تِ → "you" (implicit)
       - 1p: نَا → "we" (implicit)
  1.3  If yes: add implicit subject constituent
       - Set implicit = true
       - Set role = "fa'il"
       - Use person/number/gender from verb features

Step 2: Detect Implicit Predicate
  2.1  In nominal sentences where khabar is missing:
  2.2  Common with adverbial khabar:
       - e.g., اَلطَّالِبُ فِي الْفَصْلِ (implicit كَائِن/مَوْجُود)
  2.3  Add implicit predicate with role "khabar"

Step 3: Mark All Implicit Constituents
  3.1  Set implicit = true
  3.2  Add evidence describing why ellipsis was detected
```

---

## 14. MOD-05: School-Specific Syntax

### 14.1 Syntax Rule Differences Between Schools

| Rule | Basra | Kufa | Baghdad | Andalus | Modern |
|------|-------|------|---------|---------|--------|
| **مبتدأ must be definite** | Yes | No | Yes | Yes | No |
| **خبر may precede مبتدأ** | No | Yes | Yes | No | Yes |
| **إِنَّ vs. أَنَّ distinction** | Strict | Nuanced | Moderate | Strict | Relaxed |
| **Conditional إِنْ mood** | Jussive both | Jussive both | Jussive both | Jussive both | Can be perfect |
| **لَوْ as conditional** | Hypothetical only | Broader | Moderate | Standard | Standard |
| **فَ in conditional result** | Always if nominal | Always if nominal | Always if nominal | Always if nominal | Often optional |
| **Idafa with adjectives** | Strict | Relaxed | Moderate | Strict | Relaxed |
| **إِلَّا case assignment** | Accusative | Flexible | Accusative | Accusative | Flexible |
| **Vocative يَا case** | Accusative | Nominative allowed | Accusative | Accusative | Accusative |

### 14.2 School Application in Parsing

```pseudocode
Algorithm: apply_school_syntax_config
Input: school, trees (SyntaxTree[])
Output: SyntaxTree[] (reordered/filtered)

1.  Get school_config from SchoolConfig[school]

2.  Reorder trees:
    2.1  Trees from the configured school's rule set are preferred
    2.2  Trees from other schools are included with lower confidence

3.  Apply school-specific filters:
    3.1  If school == "basra":
         - Reject trees where mubtada' is indefinite
         - Reject trees where khabar precedes mubtada'
         - Require strict إِنَّ vs. أَنَّ distinction
    3.2  If school == "kufa":
         - Accept trees with indefinite mubtada'
         - Allow khabar to precede mubtada'
         - Allow إِنَّ to function as أَنَّ in certain contexts
    3.3  If school == "modern":
         - Accept relaxed idafa rules
         - Allow conditional إِنْ with perfect tense
         - Accept إِلَّا with flexible case assignment

4.  Return reordered (and optionally filtered) trees
```

---

## 15. Cross-Module Interaction

### 15.1 MOD-04 → MOD-05 Interface Contract

```yaml
InterfaceContract:
  producer: "MOD-04 (MorphologicalParser)"
  consumer: "MOD-05 (SyntaxParser)"
  ir_type: "IR-4 (MorphologicalAnalysis)"

  guarantees:
    - "Every token in the input has at least one morphological analysis"
    - "The 'unknown' POS is used only when no analysis is possible"
    - "Features are packed per KB-0007 taxonomy"
    - "Confidence scores are set for every analysis and feature"
    - "The alternatives chain is complete (no pruning by MOD-04)"
    - "Evidence entries document every root extraction and wazan match"

  requirements:
    - "MOD-05 MUST consider all morphological alternatives when parsing"
    - "MOD-05 MAY reorder analyses based on syntactic fit"
    - "MOD-05 MUST NOT discard morphological alternatives (pruning is MOD-07's job)"
```

### 15.2 MOD-05 → MOD-06 Interface Contract

```yaml
InterfaceContract:
  producer: "MOD-05 (SyntaxParser)"
  consumer: "MOD-06 (GIRConstructor)"
  ir_type: "IR-5 (SyntaxTree)"

  guarantees:
    - "Every parse tree references only tokens that exist in IR-4"
    - "Constituent token_indices are sequential and non-overlapping"
    - "Implicit constituents are marked with implicit: true"
    - "Partial parses are marked with tree_type = 'incomplete'"
    - "Evidence entries document every syntactic decision"

  requirements:
    - "MOD-06 MUST combine IR-4 and IR-5 into a unified GIR"
    - "MOD-06 MUST prune invalid (morphology × syntax) combinations"
    - "MOD-06 MUST include all evidence from both MOD-04 and MOD-05"
```

### 15.3 Plugin Injection Points

The Morphology Engine supports plugins at the following points:

```yaml
PluginInjectionPoints:
  MOD-04:
    - before_fast_path: "Custom token classifier (pre-check before KB-0005/6)"
    - root_extraction: "Custom root extraction algorithm"
    - wazan_matching: "Custom wazan matching strategy"
    - feature_extraction: "Custom feature extractor"
    - after_analysis: "Post-processing and enrichment"

  MOD-05:
    - before_parse: "Custom sentence segmentation"
    - parse_engine: "Custom parsing algorithm (e.g., dependency grammar)"
    - construction_recognition: "Custom construction recognizer"
    - after_parse: "Post-processing and enrichment"
```

---

## 16. Testing & Validation Strategy

### 16.1 Test Categories

```yaml
TestCategories:
  unit_tests:
    description: "Test individual algorithms in isolation"
    coverage_target: "90%+ of code paths"

    test_modules:
      - "Root extraction (triliteral, quadriliteral, weak, hamzated, doubled)"
      - "Wazan pattern signature computation and matching"
      - "Feature extraction per POS"
      - "Feature bitfield packing and unpacking"
      - "Sentence segmentation"
      - "Verbal sentence parsing"
      - "Nominal sentence parsing"
      - "Construction recognition (idafa, wasf, tawkid, badal)"

  integration_tests:
    description: "Test full MOD-04 and MOD-05 pipelines"
    coverage_target: "100% of KB-defined patterns"

    test_modules:
      - "Full MOD-04 pipeline on known words index"
      - "Full MOD-04 pipeline on all verb forms I-XV"
      - "Full MOD-04 pipeline on all noun patterns"
      - "Full MOD-04 + MOD-05 pipeline on canonical sentences"
      - "Full pipeline with all five schools"

  regression_tests:
    description: "Ensure KB updates don't break existing analyses"
    coverage_target: "Known problematic cases + diverse corpus"

    test_sources:
      - "Quranic Arabic Corpus (all chapters)"
      - "MSA news corpus (100,000+ sentences)"
      - "Classical Arabic poetry corpus"
      - "Hadith corpus"
      - "User-submitted ambiguous cases"

  performance_tests:
    description: "Verify performance targets"
    benchmarks:
      - "Fast-path lookup: 1M tokens/second minimum"
      - "Simple analysis: 100K tokens/second minimum"
      - "Complex analysis (weak root): 10K tokens/second minimum"
      - "Syntax parsing: 1K sentences/second minimum"
      - "Memory usage: within budget per school"
```

### 16.2 Test Fixture Format

All tests should use a common fixture format:

```yaml
TestFixture:
  input:
    arabic_text: string                 # Arabic text to analyze
    segmentation_hint: string | null    # Optional pre-segmented stem
    school: string                      # Grammar school to use

  expected:
    token_count: integer | null
    root: string | null                 # Expected root
    wazan: string | null                # Expected wazan pattern
    pos: string | null                  # Expected POS
    features:                          # Expected features (partial match OK)
      feature_name: feature_value
    sentence_type: string | null       # For MOD-05 tests
    parse_structure: string[] | null    # Expected constituent roles

  known_ambiguity:
    allowed_alternatives: integer       # Number of acceptable analyses
    includes: string[]                  # Analyses that MUST be in alternatives
    excludes: string[]                  # Analyses that MUST NOT be in alternatives

  metadata:
    source: string                      # Where this test case comes from
    grammatical_rule: string            # Which grammatical rule it tests
    notes: string
```

### 16.3 Example Test Cases

```yaml
Test_Verb_Form_I_Sound:
  input:
    arabic_text: "كَتَبَ"
    school: "basra"
  expected:
    root: "ك ت ب"
    wazan: "فَعَلَ"
    pos: "verb"
    features:
      tense: "past"
      person: "third"
      gender: "masculine"
      number: "singular"
      voice: "active"
      verb_form: "I"

Test_Noun_Masdar:
  input:
    arabic_text: "كِتَابَةً"
    school: "basra"
  expected:
    root: "ك ت ب"
    wazan: "فِعَالَة"
    pos: "noun"
    features:
      noun_type: "masdar"
      gender: "feminine"
      number: "singular"
      state: "indefinite"
      case: "accusative"

Test_Nominal_Sentence:
  input:
    arabic_text: "اَلْبَيْتُ كَبِيرٌ"
    school: "basra"
  expected:
    sentence_type: "jumlah_ismiyyah"
    parse_structure: ["mubtada'", "khabar"]
    features:
      al-baytu:
        pos: "noun"
        state: "definite"
        case: "nominative"
      kabirun:
        pos: "adjective"
        state: "indefinite"
        case: "nominative"

Test_Conditional_Sentence:
  input:
    arabic_text: "إِنْ تَدْرُسْ تَنْجَحْ"
    school: "basra"
  expected:
    sentence_type: "jumlah_shartiyyah"
    parse_structure: ["shart_particle", "fi'l_shart", "jawab"]
    features:
      particle:
        text: "إِن"
      tadrus:
        mood: "jussive"
      tanjah:
        mood: "jussive"

Test_Weak_Root_Hollow:
  input:
    arabic_text: "قَالَ"
    school: "basra"
  expected:
    root: "ق و ل"
    wazan: "فَعَلَ"
    pos: "verb"
    features:
      tense: "past"
      root_type: "ajwaf_wawi"

Test_Ambiguity_Homograph:
  input:
    arabic_text: "مَا"
    school: "basra"
  allowed_alternatives: 5                # مَا has 5+ interpretations
  includes:
    - "interrogative particle"
    - "negative particle"
    - "relative pronoun"
    - "indefinite noun (maa, 'thing')"
    - "conditional particle"
```

---

## 17. Implementation Guidance

### 17.1 Recommended Implementation Languages

| Component | Recommended Language | Rationale |
|-----------|-------------------|-----------|
| **Root extraction** | Rust / C++ | Performance-critical, tight loop |
| **Wazan matching** | Rust / C++ | Hash-heavy, needs O(1) guarantees |
| **Feature extraction** | Rust / C++ | KB access patterns, bitfield ops |
| **Syntax parser** | Rust / C++ | Chart parsing, complex algorithms |
| **School config** | Any (YAML/JSON) | Data-driven, loaded at init |
| **Test framework** | Rust (cargo test) | Native integration |
| **Plugin system** | Rust + WASM | Sandboxed plugins |

### 17.2 Key Implementation Risks

| Risk | Mitigation |
|------|-----------|
| **Weak root extraction accuracy** | Comprehensive test suite covering all root types; school-specific heuristics; fallback to guess mode |
| **Wazan signature collisions** | 64-bit hash space is large enough; add root_type to signature to avoid collisions between verb and noun patterns |
| **Chart parsing O(n³) complexity** | Limit max_sentence_length to 200 tokens; use beam search (keep top K candidates) for long sentences |
| **KB memory pressure** | Memory-map KB files; lazy loading of optional KBs; compact compilation (Level 1) for constrained environments |
| **School-specific rule conflicts** | Clear priority ordering; strict_mode for fail-fast; ambiguity preservation for non-strict mode |
| **Unvocalized text morph ambiguity** | Preserve all alternatives; let syntax (MOD-05) and rules (MOD-07) disambiguate |

### 17.3 Data Flow Through the Engine

The complete data flow through both modules, from segmented input to parsed output:

```diff
  IR-3 (SegmentedTokenStream)
    │
    │ MOD-04 — Subsystem 1: Fast-Path Checker
    ▼
  ┌──────────────────────────────────────┐
  │  KB-0005 lookup (O(1) hash)          │
  │  KB-0006 lookup (O(1) hash)          │
  │  If found: → Subsystem 4             │
  │  If not:   → Subsystem 2             │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 2: Root Extraction        │
  │                                      │
  │  1. Known word index (O(1) hash)     │
  │  2. Triliteral extraction            │
  │  3. Weak root detection & restore    │
  │  4. Hamzated root detection          │
  │  5. Geminate splitting               │
  │  6. KB-0001 trie lookup              │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 3: Wazan Identification   │
  │                                      │
  │  1. Pattern signature computation    │
  │  2. KB-0002 hash lookup              │
  │  3. Verb form matching (I-XV)        │
  │  4. Noun pattern matching            │
  │  5. Weak variant matching            │
  │  6. Ambiguity set generation         │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 4: Feature Extraction     │
  │                                      │
  │  1. POS-specific feature extraction  │
  │  2. KB-0007 feature bitfield pack    │
  │  3. KB-0007 validation rules         │
  │  4. Default/inference application    │
  │  5. Evidence trail generation        │
  └──────────────────────────────────────┘
    │
    │ IR-4 (MorphologicalAnalysis)
    ▼
  ┌──────────────────────────────────────┐
  │  MOD-05 — Subsystem 1: Segmentation  │
  │                                      │
  │  1. Sentence boundary detection      │
  │  2. Sentence type identification     │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 2: Parse Engine           │
  │                                      │
  │  1. Verbal sentence parsing          │
  │  2. Nominal sentence parsing         │
  │  3. Conditional sentence parsing     │
  │  4. Construction recognition          │
  │     (idafa, wasf, tawkid, badal)     │
  │  5. Ambiguity combination            │
  └──────────────────────────────────────┘
    │
    ▼
  ┌──────────────────────────────────────┐
  │  Subsystem 3: Output Packaging       │
  │                                      │
  │  1. Tree structure construction      │
  │  2. Confidence scoring               │
  │  3. Partial parse fallback           │
  │  4. Evidence trail generation        │
  └──────────────────────────────────────┘
    │
    │ IR-5 (SyntaxTree)
    ▼
  MOD-06 (GIRConstructor)
```

---

## 18. Cross-References

### 18.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | MOD-04 and MOD-05 module definitions, layer boundaries |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | Pipeline algorithms for MOD-04 and MOD-05 (foundational reference) |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Public interface definitions for MOD-04 and MOD-05 |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | IR-4 and IR-5 schemas, evidence trail model |
| SPEC-0001-C9 | Performance Targets & Constraints | Performance targets for morphology and syntax |
| SPEC-0201 | Rule Engine (planned) | Consumes MOD-05 output; school-specific rule application |
| SPEC-0401 | Knowledge Graph Engine | Consumes MOD-04/MOD-05 output for KG resolution |
| RFC-0002 | Grammar Bytecode Format | Feature bitfield layout (KB-0007 taxonomy) |
| ADR-0001 | Compiler Architecture Rationale | Pipeline decomposition justification |

### 18.2 Knowledge Base References

| KB | Title | Relationship |
|----|-------|--------------|
| KB-0001 | Roots Database | Root dictionary for extraction (MOD-04 Subsystem 2) |
| KB-0002 | Wazan Database | Pattern templates for matching (MOD-04 Subsystem 3) |
| KB-0003 | Verb Forms | Conjugation paradigms for feature verification (MOD-04 Subsystem 4) |
| KB-0004 | Noun Patterns | Noun specifications for feature extraction (MOD-04 Subsystem 4) |
| KB-0005 | Particles | Fast-path identification (MOD-04 Subsystem 1) |
| KB-0006 | Pronouns | Fast-path identification (MOD-04 Subsystem 1) |
| KB-0007 | Morphological Features | Feature taxonomy, bitfield layout, agreement rules, validation |

### 18.3 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh's Al-Kitab (الكتاب) | Foundational Arabic grammar reference for Basra school |
| Wright's Arabic Grammar | Standard reference for Arabic grammar in English |
| Buckwalter Arabic Morphological Analyzer (BAMA) | Reference morphological analysis approach |
| Quranic Arabic Corpus | Reference corpus for validation and testing |
| CKY Algorithm (Kasami-T. Younger) | Chart parsing foundation for MOD-05 |
| Earley Algorithm | Alternative chart parsing approach for MOD-05 |

---

## Progress Summary

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Introduction & Scope | ✓ COMPLETE |
| Section 2 | Architecture Overview | ✓ COMPLETE |
| Section 3 | Internal Component Model | ✓ COMPLETE |
| Section 4 | MOD-04: Root Extraction Subsystem | ✓ COMPLETE |
| Section 5 | MOD-04: Wazan Identification Subsystem | ✓ COMPLETE |
| Section 6 | MOD-04: Feature Extraction Subsystem | ✓ COMPLETE |
| Section 7 | MOD-04: Ambiguity Management Subsystem | ✓ COMPLETE |
| Section 8 | MOD-04: School-Specific Behavior | ✓ COMPLETE |
| Section 9 | MOD-04: Performance & Optimization | ✓ COMPLETE |
| Section 10 | MOD-05: Sentence Segmentation | ✓ COMPLETE |
| Section 11 | MOD-05: Parse Algorithm | ✓ COMPLETE |
| Section 12 | MOD-05: Syntactic Construction Recognition | ✓ COMPLETE |
| Section 13 | MOD-05: Ambiguity & Partial Parsing | ✓ COMPLETE |
| Section 14 | MOD-05: School-Specific Syntax | ✓ COMPLETE |
| Section 15 | Cross-Module Interaction | ✓ COMPLETE |
| Section 16 | Testing & Validation Strategy | ✓ COMPLETE |
| Section 17 | Implementation Guidance | ✓ COMPLETE |
| Section 18 | Cross-References | ✓ COMPLETE |

**Dependencies:** SPEC-0001 (Chapters 1–9), KB-0001 through KB-0007, RFC-0002, ADR-0001.

**Recommended next step:** SPEC-0201 (Rule Engine) — the next major component specification that details how school-specific grammatical rules are applied to the Morphology Engine's output.
