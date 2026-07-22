---
kb_id: KB-0004
title: Wazan & Noun Pattern Database — Authoritative Pattern-to-POS Repository
version: 1.0.0-draft
status: Proposal
author: AGOS Linguistics Committee
created: 2026-07-22
updated: 2026-07-22
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline (MOD-04 MorphologicalParser)
  - SPEC-0101: Morphology Engine
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
supersedes:
  - agos-morph/src/morphological_parser.rs:COMMON_NOUNS_3L
  - agos-morph/src/morphological_parser.rs:COMMON_VERBS_3L
---

# KB-0004: Wazan & Noun Pattern Database
## Authoritative Pattern-to-POS Repository

## Table of Contents

1. [Motivation & Problem Statement](#1-motivation--problem-statement)
2. [Design Goals](#2-design-goals)
3. [Architecture Overview](#3-architecture-overview)
4. [Data Schema](#4-data-schema)
5. [WazanPatternEntry — Core Record](#5-wazanpatternentry--core-record)
6. [Supported Pattern Types](#6-supported-pattern-types)
7. [Weak Root & Variant Handling](#7-weak-root--variant-handling)
8. [Runtime Lookup API](#8-runtime-lookup-api)
9. [Integration with MOD-04](#9-integration-with-mod-04)
10. [Integration with KB-0001 & KB-0002](#10-integration-with-kb-0001--kb-0002)
11. [Migration Path from Heuristic Lists](#11-migration-path-from-heuristic-lists)
12. [Serialization & Storage](#12-serialization--storage)
13. [Performance Budget](#13-performance-budget)
14. [Example Entries](#14-example-entries)
15. [Open Questions & Future Work](#15-open-questions--future-work)

---

## 1. Motivation & Problem Statement

### 1.1 Current State

MOD-04 (MorphologicalParser) currently uses three hardcoded Rust `&[&str]` arrays to disambiguate 3-letter stems:

| List | Size | Purpose | Limitation |
|------|------|---------|------------|
| `COMMON_NOUNS_3L` | ~90 entries | Suppress Form I verb analysis for known nouns | Incomplete; no confidence scoring; manually maintained |
| `COMMON_VERBS_3L` | ~77 entries | Boost Form I verb confidence to 0.35 for known verbs | Incomplete; no weak-root awareness; manually maintained |
| Verb form matching | 15 arms in `check_verb_form()` | Pattern-match verb forms I–XV by length/prefix | Fragile heuristics; no root-type awareness; no weak-root variants |

### 1.2 Problems with the Heuristic Approach

1. **Incompleteness**: ~90 + ~77 = ~167 entries cannot cover the thousands of common Arabic 3-letter stems. Every stem not in either list defaults to Verb (Form I 0.3), causing systematic misclassification of nouns.

2. **No confidence scaling**: All noun-list entries suppress verb analysis to 0.0 uniformly. A clear noun (رجل, man) gets the same treatment as a borderline case (فكر, thought). There is no gradient based on how "noun-like" a stem is.

3. **No root-type awareness**: The verb form matching looks at `stem_len` and `stem_text.starts_with()` but doesn't consult the actual root type (sound, hollow, defective, doubled). This leads to brittle heuristics like checking `stem_text.chars().nth(1)` for Form III.

4. **No weak-root variants**: Hollow verbs (قال, كان) and defective verbs (رمى, دعا) are matched through general `stem_len == 3` checks rather than through pattern variant tables. This misses many edge cases.

5. **Maintenance burden**: Adding a word requires editing Rust source code, recompiling, and re-running all tests. There is no data-driven workflow.

### 1.3 Solution: KB-0004 as Authoritative Pattern Database

KB-0004 replaces the heuristic lists with a **data-driven pattern-to-POS repository** that stores:

1. **Morphological pattern definitions** — canonical templates for verb forms I–XV and noun patterns (فَعْل, فِعْل, فُعْل, فَعَل, etc.), each with an inherent POS classification and confidence score.

2. **Stem-level pattern variants** — weak-root surface forms for each pattern (e.g., the ajwaf variant of Form I: `C₁āC₃` instead of `C₁aC₂aC₃a`).

3. **Known-stem POS overrides** — for stems whose POS cannot be deduced from pattern alone (chiefly Form I masdar patterns where the same surface form is ambiguous between verb and noun).

---

## 2. Design Goals

| Goal | Priority | Rationale |
|------|----------|-----------|
| **Replace COMMON_NOUNS_3L + COMMON_VERBS_3L** | P0 | Remove hardcoded lists; serve ~5,000+ entries from first release |
| **Cover verb forms I–X with weak-root variants** | P0 | Replace `check_verb_form()` match arms with data-driven lookups |
| **Provide per-pattern confidence/priority scores** | P0 | Allow gradient noun/verb disambiguation instead of binary 0.0/0.3 |
| **Support both pattern-based and stem-based lookups** | P1 | MOD-04 needs: "what POS does this pattern have?" AND "what POS does this specific 3-letter stem have?" |
| **Integrate with KB-0001 roots** | P1 | Cross-reference root entries to verify verb-form attestation |
| **JSON source + compiled binary formats** | P1 | Follow existing KB conventions |
| **Backward compatibility during migration** | P2 | Allow gradual rollout — use KB-0004 when available, fall back to heuristics |

---

## 3. Architecture Overview

### 3.1 Data Flow

```
                        KB-0004
                    (JSON source)
                         │
                    agos kb compile
                         │
                    Compiled trie
                    (~5–15 MB)
                         │
                    Memory-mapped
                    at pipeline start
                         │
                    ┌────┴────┐
                    │         │
          Pattern Lookup   Stem Lookup
          (by wazan)       (by stem text)
                    │         │
                    └────┬────┘
                         │
                   MOD-04
              MorphologicalParser
```

### 3.2 Three-Layer Lookup Strategy

MOD-04 will query KB-0004 in three tiers, with decreasing priority:

```
TIER 1: Stem POS Override
  └─ Input: stem_text (e.g., "رجل", "كتب")
  └─ Output: PartOfSpeech + confidence (if override exists)
  └─ Covers: ~2,000–5,000 most common stems where
     pattern-based deduction is ambiguous

TIER 2: Pattern POS Assignment
  └─ Input: wazan_canonical (e.g., "فَعْل", "فَاعِل")
  └─ Output: inherent POS + confidence + inflection properties
  └─ Covers: All derived forms where pattern uniquely
     determines or strongly suggests POS

TIER 3: Root-Aware Backed-off Estimation
  └─ Input: root (from KB-0001) + pattern candidate
  └─ Output: weighted POS score based on:
     - Whether this root attests this verb form (KB-0001)
     - Whether the root's semantic field is nominal or verbal
     - Pattern frequency across all roots
```

---

## 4. Data Schema

### 4.1 Logical Data Model

```yaml
KB-0004 (Wazan & Noun Pattern Database)
├── Metadata
│   ├── kb_id: "KB-0004"
│   ├── version: "1.0.0"
│   ├── pattern_count: integer
│   ├── stem_override_count: integer
│   └── checksum_sha256: string
│
├── PatternCatalog: WazanPatternEntry[]
│   ├── Verb form patterns (I–X, with vowel variants)
│   ├── Noun pattern templates (فَعْل, فِعْل, فُعْل, فَعَل, etc.)
│   ├── Derived noun patterns (participles, place/time, instrument, etc.)
│   └── Weak root pattern variants for each base pattern
│
├── PosProfile: PosProfileEntry[]         # POS profiles per (pattern, root_type) pair
│
└── StemOverride: StemOverrideEntry[]     # Known-stem POS overrides
```

### 4.2 High-Level Structure

```
KB-0004/
├── metadata.json
├── patterns/
│   ├── verbs/
│   │   ├── form-I.json           # فَعَلَ, فَعِلَ, فَعُلَ
│   │   ├── form-II.json          # فَعَّلَ
│   │   ├── form-III.json         # فَاعَلَ
│   │   ├── form-IV.json          # أَفْعَلَ
│   │   ├── form-V.json           # تَفَعَّلَ
│   │   ├── form-VI.json          # تَفَاعَلَ
│   │   ├── form-VII.json         # اِنْفَعَلَ
│   │   ├── form-VIII.json        # اِفْتَعَلَ
│   │   ├── form-IX.json          # اِفْعَلَّ
│   │   └── form-X.json           # اِسْتَفْعَلَ
│   ├── nouns/
│   │   ├── masdar-form-I.json    # ~40+ Form I masdar patterns
│   │   ├── masdar-regular.json   # Forms II–X masdar patterns
│   │   ├── participle-active.json
│   │   ├── participle-passive.json
│   │   ├── place-time.json
│   │   ├── instrument.json
│   │   ├── adjective.json
│   │   └── broken-plural.json
│   └── weak-variants/
│       ├── ajwaf.json
│       ├── naqis.json
│       ├── mithal.json
│       ├── doubled.json
│       └── hamzated.json
├── profiles/
│   └── pos-profiles.json         # POS profiles for all (pattern, root_type) pairs
└── overrides/
    └── stem-overrides.json       # Known-stem POS overrides (~2K–5K entries)
```

---

## 5. WazanPatternEntry — Core Record

### 5.1 Schema

```yaml
WazanPatternEntry:
  # --- Identity ---
  id: string                              # "KB-0004:verb:form_I:basic_a"
                                          # "KB-0004:noun:masdar:form_I:فَعْل"
  pattern_family: "verb" | "noun" | "particle"
  verb_form: integer | null               # 1–15
  noun_type: string | null                # "masdar", "ism_fail", "ism_maful", etc.

  # --- Canonical Representation ---
  canonical_pattern: string               # With ف-ع-ل, e.g., "فَعَلَ", "فَاعِل", "فَعْل"
  template_script: string                 # e.g., "C₁aC₂aC₃a", "C₁āC₂iC₃", "C₁aC₂C₃"
  consonant_count: integer                # Count of root-consonant slots

  # --- Matching Heuristics ---
  length_constraint: LengthConstraint     # Stem length constraints for matching
  prefix_pattern: string | null           # e.g., "أ", "ت", "اِ", "اِنْ", "اِسْتَ"
  infix_pattern: string | null            # e.g., "ت" (Form VIII)
  suffix_pattern: string | null           # e.g., "ة", "ات"
  weak_letter_positions: integer[]        # Which root positions can be weak (1-based)
  structural_pattern: string              # Regex-like: "C a C a C a", "C ā C i C"

  # --- POS & Confidence ---
  default_pos: PartOfSpeech               # "Verb" | "Noun" | "Adjective" | etc.
  default_confidence: float               # 0.0–1.0, e.g., 0.30 for Form I
  boosted_confidence: float | null        # Higher confidence for known-form roots
  pos_profile_id: string                  # Key into PosProfile table for per-root-type variants

  # --- Source Form Mapping ---
  derived_from_verb_form: integer | null  # For noun patterns: which verb form they derive from
  applies_to_root_types: RootType[]       # Which root types this pattern applies to
                                           # ["sound", "ajwaf_wawi", "ajwaf_yai", "naqis_wawi",
                                           #  "naqis_yai", "mithal_wawi", "mithal_yai",
                                           #  "doubled", "hamzated_first", etc.]

  # --- Weak Root Variants ---
  weak_variants: PatternVariant[]         # Surface-form variants for weak root types

  # --- Semantic & Grammatical ---
  semantic_modification: string           # e.g., "causative", "reciprocal", "intensive"
  inherent_features: FeatureMap           # Features this pattern inherently assigns

  # --- Attestation ---
  attestation: Attestation

  # --- Examples ---
  examples: Example[]
```

### 5.2 Supporting Types

```yaml
LengthConstraint:
  min_letters: integer                    # Minimum character count for stem
  max_letters: integer                    # Maximum character count
  exact: integer | null                   # Exact count, if fixed

PatternVariant:
  variant_name: string                    # "ajwaf_wawi_perfect", "naqis_yai_imperative"
  applies_to_root_type: RootType
  template_script: string                 # e.g., "C₁āC₃", "C₁aC₂ī"
  condition: string                       # e.g., "past tense, active, perfect stem"
  confidence_adjustment: float            # +/- adjustment from base confidence

PosProfileEntry:
  profile_id: string                      # e.g., "form_I_sound", "form_I_hollow"
  root_type: RootType
  default_pos: PartOfSpeech
  confidence: float
  notes: string | null

PartOfSchool:
  "Verb" | "Noun" | "Adjective" | "Particle" | "Pronoun" | "Unknown"

RootType:
  "sound" | "mithal_wawi" | "mithal_yai" | "ajwaf_wawi" | "ajwaf_yai" |
  "naqis_wawi" | "naqis_yai" | "lafif_mafruq" | "lafif_makrun" |
  "hamzated_first" | "hamzated_middle" | "hamzated_last" | "doubled" |
  "quadriliteral_sound" | "quadriliteral_weak"

FeatureMap:
  features: NamedFeature[]

NamedFeature:
  name: string
  value: string
  confidence: float

Attestation:
  confidence: "certain" | "well_attested" | "attested" | "disputed"
  primary_sources: string[]

Example:
  word: string
  transliteration: string
  meaning: string
  root: string
```

### 5.3 3-Letter Stem POS Override Schema

This is the direct replacement for `COMMON_NOUNS_3L` and `COMMON_VERBS_3L`:

```yaml
StemOverrideEntry:
  # --- Identity ---
  stem_text: string                       # e.g., "رجل", "كتب", "قال"
  root: string | null                     # e.g., "رجل", "كتب", "قول"

  # --- POS Assignment ---
  pos: PartOfSpeech                       # Primary POS
  secondary_pos: PartOfSpeech | null      # Secondary POS (for ambiguous stems)
  confidence: float                       # How strongly this stem favors its primary POS
                                          # 0.7+ = strongly favors noun
                                          # 0.5–0.7 = moderately favors
                                          # 0.3–0.49 = slightly favors

  # --- Morphological Info ---
  pattern_candidates: PatternCandidate[]  # Which patterns this stem maps to
  root_type: RootType                     # For weak roots, the type

  # --- Source ---
  source: string                          # "KB-0001:root" | "classical_lexicon" | "corpus_frequency"
  frequency_in_corpus: string | null      # "high" | "medium" | "low" | "rare"
  notes: string | null

PatternCandidate:
  pattern_id: string                      # Reference to KB-0004 pattern entry
  canonical_pattern: string               # e.g., "فَعْل", "فَعَلَ"
  weight: float                           # How likely this pattern is for this stem (0.0–1.0)
```

---

## 6. Supported Pattern Types

### 6.1 Verb Form Patterns (I–XV)

Each verb form pattern stores:
- Canonical wazan with ف-ع-ل placeholders
- Length constraints (min/max letters)
- Prefix/infix/suffix patterns for matching
- Default POS: Verb
- Default confidence per form (matching current heuristic):
  - Form I: 0.30 (sound), 0.25 (imperfect prefix), 0.20 (imperative prefix)
  - Form II: 0.35 (doubled), 0.20 (geminated), 0.15 (weak middle)
  - Form III: 0.20
  - Form IV: 0.20
  - Form V: 0.20
  - Form VI: 0.15
  - Form VII: 0.15
  - Form VIII: 0.20
  - Form IX: 0.10
  - Form X: 0.10
  - Forms XI–XV: 0.05

### 6.2 Noun Pattern Templates

Each noun pattern stores inherent POS and confidence:

| Canonical | Template | Example | Default POS | Default Confidence |
|-----------|----------|---------|-------------|-------------------|
| فَعْل | C₁aC₂C₃ | ضَرْب (hit), رَجُل (man) | Noun | 0.25 |
| فِعْل | C₁iC₂C₃ | عِلْم (knowledge) | Noun | 0.25 |
| فُعْل | C₁uC₂C₃ | حُسْن (beauty) | Noun | 0.25 |
| فَعَل | C₁aC₂aC₃ | طَلَب (request), جَمَل (camel) | Noun | 0.25 |
| فِعَل | C₁iC₂aC₃ | قِرَد (monkeys) | Noun | 0.20 |
| فُعَل | C₁uC₂aC₃ | دُرَر (pearls) | Noun | 0.20 |
| فَعَال | C₁aC₂āC₃ | سَلَام (peace) | Noun | 0.25 |
| فِعَال | C₁iC₂āC₃ | كِتَاب (book) | Noun | 0.25 |
| فُعَال | C₁uC₂āC₃ | غُسَال (washing) | Noun | 0.20 |
| فَعِيل | C₁aC₂īC₃ | كَرِيم (generous) | Adjective | 0.25 |
| فَعُول | C₁aC₂ūC₃ | صَبُور (patient) | Adjective | 0.20 |
| فَعْلَة | C₁aC₂C₃a | جَلْسَة (session) | Noun | 0.30 |
| فِعْلَة | C₁iC₂C₃a | جِلْسَة (posture) | Noun | 0.20 |
| فُعْلَة | C₁uC₂C₃a | رُكْبَة (knee) | Noun | 0.25 |
| فَاعِل | C₁āC₂iC₃ | كَاتِب (writer) | Noun | 0.30 |
| فَعَّال | C₁aC₂C₂āC₃ | كَتَّاب (scribe) | Noun | 0.25 |
| فَعُولَة | C₁aC₂ūC₃a | رَحْمَة (mercy) | Noun | 0.20 |
| فَعَلَان | C₁aC₂aC₃ān | غَلَيَان (boiling) | Noun | 0.20 |
| فُعْلَان | C₁uC₂C₃ān | نُقْصَان (deficiency) | Noun | 0.20 |
| أَفْعَل | aC₁C₂aC₃ | أَكْبَر (greater) | Adjective | 0.30 |
| فَعْلَى | C₁aC₂C₃ā | مَجْرَى (course) | Noun | 0.20 |
| فِعْلَى | C₁iC₂C₃ā | ذِكْرَى (remembrance) | Noun | 0.20 |
| فُعْلَى | C₁uC₂C₃ā | رُجْعَى (return) | Noun | 0.20 |
| فَعْلَاء | C₁aC₂C₃āʾ | حَمْرَاء (red) | Adjective | 0.30 |
| فِعْلِيّ | C₁iC₂C₃iyy | عِلْمِيّ (scientific) | Adjective | 0.30 |
| فَعْلِيّ | C₁aC₂C₃iyy | عَرَبِيّ (Arabic) | Adjective | 0.30 |
| مَفْعَل | maC₁C₂aC₃ | مَكْتَب (office) | Noun | 0.30 |
| مَفْعِل | maC₁C₂iC₃ | مَوْعِد (appointment) | Noun | 0.25 |
| مَفْعُول | maC₁C₂ūC₃ | مَكْتُوب (written) | Adjective | 0.30 |
| مِفْعَل | miC₁C₂aC₃ | مِنْجَل (sickle) | Noun | 0.25 |
| مِفْعَال | miC₁C₂āC₃ | مِفْتَاح (key) | Noun | 0.30 |
| مِفْعَلَة | miC₁C₂aC₃a | مِكْنَسَة (broom) | Noun | 0.30 |
| تَفْعِيل | taC₁C₂īC₃ | تَكْتِيب (dictation) | Noun | 0.25 |

### 6.3 Pattern-to-POS Decision Logic

The core algorithm for determining POS from a matched pattern:

```pseudo
Algorithm: pos_from_pattern
Input: pattern (WazanPatternEntry), stem_text (str),
       root (RootCandidate | null), context (PipelineContext)
Output: (PartOfSpeech, confidence)

1. If pattern.pattern_family == "verb":
   → Return (Verb, pattern.default_confidence)

2. If pattern has a known POS in its noun_type:
   → Return (map_noun_type_to_pos(pattern.noun_type), pattern.default_confidence)

3. Else (noun patterns with ambiguous POS):
   a. Check StemOverrideEntry for this stem_text:
      → If found, return override value
   b. Check if stem_text.ends_with('ي') and not 'ة':
      → Return (Adjective, confidence * 0.8)
   c. Check pattern.default_pos:
      → Return (pattern.default_pos, pattern.default_confidence)
   d. Default:
      → Return (Noun, 0.2)

Helper: map_noun_type_to_pos(noun_type)
  "masdar" | "ism_makan" | "ism_zaman" | "ism_alah" |
  "ism_marrati" | "jam_taksir" | "noun_other" → Noun
  "ism_fail" | "ism_maful" | "sifah_mushabbahah" |
  "tafdil" | "nisbah" → Adjective (or Noun if contextually nominal)
```

---

## 7. Weak Root & Variant Handling

### 7.1 Base Patterns vs. Weak Variants

Each verb form and noun pattern has a **base pattern** (for sound roots) and **variants** for each weak root type. The variant table replaces the current `match` arms in `check_verb_form()` with data lookups:

| Pattern | Root Type | Variant Template | Length | Example |
|---------|-----------|-----------------|--------|---------|
| Form I (فَعَلَ) | sound | C₁aC₂aC₃a | 3 | كتب (kataba) |
| Form I | ajwaf_wawi | C₁āC₃ | 2 | قال (qāla) |
| Form I | ajwaf_yai | C₁āC₃ | 2 | سار (sāra) |
| Form I | naqis_wawi | C₁aC₂ā | 3 | دعا (daʿā) |
| Form I | naqis_yai | C₁aC₂ā | 3 | رمى (ramā) |
| Form I | doubled | C₁aC₃C₃ | 2–3 | مدّ (madda) |
| Form I | mithal_wawi | C₁aC₂aC₃a | 3 | وجد (wajada) |
| Form I | hamzated_first | āC₂aC₃a | 3 | أكل (akala) → آكل |
| Form VIII (اِفْتَعَلَ) | sound | iC₁taC₂aC₃a | 5–6 | اجتهد (ijtahada) |
| Form VIII | coronal_C1 | iC₁C₁aC₂aC₃a | 5–6 | اصطبر (iṣṭabara) |

### 7.2 Variant Matching Algorithm

```pseudo
Algorithm: match_with_variants
Input: stem_text (str), root_type (RootType), pattern (WazanPatternEntry)
Output: (matched_variant, confidence)

1. Determine the set of applicable variants:
   a. Start with the base pattern template
   b. Look up all PatternVariant entries where
      applies_to_root_type matches root_type
   c. Merge: base + applicable variants = candidate set

2. For each candidate (base or variant):
   a. Generate expected stem by filling template with root consonants
   b. Compare against stem_text
   c. Score the match:
      - Exact match: 1.0 weight
      - Partial match (one char diff): 0.7 weight
      - No match: 0.0 weight

3. Return best match with highest weight × confidence.
```

---

## 8. Runtime Lookup API

### 8.1 Rust Trait

```rust
/// API that MOD-04 uses to query KB-0004 at runtime.
pub trait WazanPatternLookup {
    /// Look up the default POS & confidence for a wazan pattern.
    fn pos_for_pattern(&self, canonical_pattern: &str, root_type: RootType)
        -> (PartOfSpeech, f64);

    /// Look up a specific stem's POS override.
    fn stem_pos_override(&self, stem_text: &str)
        -> Option<(PartOfSpeech, f64)>;

    /// Get all verb form patterns that could match a stem,
    /// considering length, prefix, and root type.
    fn matching_verb_patterns(&self, stem_text: &str, stem_len: usize, root_type: RootType)
        -> Vec<(u8, String, f64)>;  // (form_number, pattern_variant, confidence)

    /// Get all noun patterns that could match a stem.
    fn matching_noun_patterns(&self, stem_text: &str, stem_len: usize)
        -> Vec<(WazanCategory, String, f64)>;  // (category, canonical_pattern, confidence)

    /// Check if a 3-letter stem is primarily a noun (replaces COMMON_NOUNS_3L).
    fn is_primarily_noun(&self, stem_text: &str) -> Option<f64>;

    /// Check if a 3-letter stem is primarily a verb (replaces COMMON_VERBS_3L boost).
    fn is_primarily_verb(&self, stem_text: &str) -> Option<f64>;

    /// Get the best POS guess for a 3-letter stem (unified noun/verb disambiguation).
    fn best_pos_for_3l_stem(&self, stem_text: &str) -> (PartOfSpeech, f64, String);
}
```

### 8.2 Example Usage in MOD-04

```rust
// Current heuristic approach (to be replaced):
fn check_verb_form(&self, stem_text: &str, _root: &RootCandidate, form: u8) -> f64 {
    // ... 15 match arms with hardcoded patterns ...
}

// KB-0004 driven approach:
fn check_verb_form_kb(&self, stem_text: &str, root: &RootCandidate, form: u8) -> f64 {
    let kb_0004 = self.kb_0004.as_ref()?;
    let stem_len = stem_text.chars().count();

    // 1. Check stem override first (fast path)
    if stem_len == 3 {
        if let Some((pos, conf)) = kb_0004.stem_pos_override(stem_text) {
            if pos != PartOfSpeech::Verb {
                return 0.0; // Known non-verb
            }
            return conf; // Known verb, use boosted confidence
        }
    }

    // 2. Look up pattern variants for this form + root type
    let candidates = kb_0004.matching_verb_patterns(
        stem_text, stem_len, root.root_type
    );
    candidates.iter()
        .find(|(f, _, _)| *f == form)
        .map(|(_, _, conf)| *conf)
        .unwrap_or(0.0)
}

// Current noun matching (to be replaced):
fn match_noun_patterns(...) {
    // Hardcoded push of WazanCandidate with fixed confidences
}

// KB-0004 driven noun matching:
fn match_noun_patterns_kb(...) {
    let kb_0004 = self.kb_0004.as_ref()?;
    let patterns = kb_0004.matching_noun_patterns(stem_text, stem_len);
    for (category, pattern, confidence) in patterns {
        candidates.push(WazanCandidate {
            text: pattern,
            category,
            confidence,
            source: "KB-0004".to_string(),
        });
    }
}
```

---

## 9. Integration with MOD-04

### 9.1 Current Architecture (Heuristic)

```
check_fast_path()
    ├── COMMON_PARTICLES lookup
    └── COMMON_PRONOUNS lookup
extract_roots()
    └── Heuristic extraction (triliteral, hollow, etc.)
identify_wazan()
    ├── match_verb_forms()
    │   └── check_verb_form()         ← 15 match arms
    └── match_noun_patterns()         ← Hardcoded WazanCandidate pushes
determine_pos()
    └── match on WazanCategory        ← Binary: Verb vs Noun/Adjective
```

### 9.2 KB-0004 Driven Architecture (Target)

```
check_fast_path()
    ├── KB-0005: Particles
    └── KB-0006: Pronouns
extract_roots()
    ├── KB-0001: Roots Database       ← Authoritative root validation
    └── Heuristic fallback (for unknown roots)
identify_wazan()
    ├── KB-0004.matching_verb_patterns()  ← Data-driven verb form matching
    │   ├── Form I–X base patterns
    │   ├── Weak root variants
    │   └── Per-form confidence from data
    └── KB-0004.matching_noun_patterns()  ← Data-driven noun pattern matching
        ├── ~35+ noun pattern templates
        ├── Per-template confidence from data
        └── Stem overlap resolution
determine_pos()
    ├── KB-0004.pos_for_pattern()     ← Data-driven POS assignment
    └── KB-0004.stem_pos_override()   ← Stem-level overrides for ambiguous cases
```

### 9.3 3-Letter Stem POS Resolution Algorithm

This is the core replacement for `COMMON_NOUNS_3L` + `COMMON_VERBS_3L`:

```pseudo
Algorithm: resolve_3l_stem_pos
Input: stem_text (3-letter Arabic stem)
Output: best_analysis (WazanCandidate[])

1. Check StemOverrideEntry for exact stem match:
   a. If found AND confidence > 0.6:
      → Return override POS with override confidence
   b. If found AND confidence > 0.3:
      → Add override as strong candidate, continue checking patterns

2. Collect all matching noun patterns for 3-letter stems:
   a. فَعْل (template C₁aC₂C₃) → Noun, 0.25
   b. فِعْل (template C₁iC₂C₃) → Noun, 0.25
   c. فُعْل (template C₁uC₂C₃) → Noun, 0.25
   d. فَعَل (template C₁aC₂aC₃) → Noun, 0.25
   → Score each by matching stem characters against template

3. Check verb form I for this stem:
   a. Look up Form I pattern with root type awareness
   b. Check if any weak-root variant matches
   c. Assign Form I confidence from KB-0004 data
   d. If StemOverride marks as verb → boost confidence to override

4. Select winner:
   a. Sort noun patterns by confidence descending
   b. Compare top noun vs verb confidence
   c. If noun confidence > verb confidence → Noun
   d. Else if verb confidence > noun confidence → Verb
   e. If tied → use StemOverride tiebreaker if available
   f. Otherwise → Noun (conservative default: prefer noun for ambiguous 3-letter stems)

5. Return ordered candidate list for MOD-04's wazan selection
```

---

## 10. Integration with KB-0001 & KB-0002

### 10.1 Cross-Reference Flow

```
                     KB-0001
                  (Roots DB)
                     │
                     │ Contains per-root:
                     │  • verb_forms[I–XV].attested
                     │  • verb_form_details[form].conjugation_class
                     │  • derived_nouns[].pattern
                     │  • root_type (sound/weak/hamzated/doubled)
                     ▼
              ┌──────────────┐
        ┌────►│   MOD-04     │◄────┐
        │     │   Pipeline   │     │
        │     └──────────────┘     │
        │           │              │
        │           ▼              │
        │     ┌──────────────┐     │
        │     │   KB-0004    │     │
        │     │  (Patterns)  │     │
        │     └──────────────┘     │
        │           │              │
        │           ▼              │
        │     ┌──────────────┐     │
        │     │   KB-0002    │─────┘
        │     │  (Wazan DB)  │  (Wazan definitions for
        │     └──────────────┘   verification/authoring)
        │
        └─────────────────────────┘
        KB-0004 queries KB-0001 for:
          • Is this form attested for this root?
          • What conjugation class does this root have?
          • What is the root's semantic field?
```

### 10.2 Root-Aware Confidence Adjustment

When KB-0004 finds a candidate pattern, it cross-references KB-0001 to adjust confidence:

```rust
fn adjust_confidence_for_root(
    base_confidence: f64,
    verb_form: u8,
    root_entry: Option<&RootEntry>,
) -> f64 {
    let root = match root_entry {
        Some(r) => r,
        None => return base_confidence * 0.5,  // Unknown root → halve confidence
    };

    // Check if this verb form is attested for this root
    let attested = root.verb_forms.iter()
        .find(|vf| vf.form == verb_form)
        .map(|vf| vf.attested)
        .unwrap_or(false);

    if attested {
        base_confidence * 1.2  // Boost by 20% for attested forms
    } else {
        base_confidence * 0.7  // Reduce by 30% for unattested forms
    }
}
```

---

## 11. Migration Path from Heuristic Lists

### 11.1 Phase 1: Seed KB-0004 from Heuristic Lists (Current → 1 week)

1. **Extract existing data**: The current `COMMON_NOUNS_3L` (~90 entries) and `COMMON_VERBS_3L` (~77 entries) become `StemOverrideEntry` records.

2. **Extract pattern definitions**: The 15 `match` arms in `check_verb_form()` and the `match_noun_patterns()` functions are transcribed into `WazanPatternEntry` JSON records.

3. **Build initial compiled KB**: `agos kb compile` generates the binary trie for KB-0004.

4. **Add dual code path**: MOD-04 checks `kb_0004.is_some()` — if the KB is loaded, use KB-0004; otherwise fall back to heuristic lists.

### 11.2 Phase 2: Expand Stem Overrides (1–4 weeks)

1. **Add ~2,000 additional stems** to `StemOverrideEntry`:
   - All roots with `root_count` fields from KB-0001 that are 3-letter stems
   - All nouns from a frequency list of MSA (top 5,000 words)
   - All Form I verbs from a standard Arabic verb frequency list

2. **Source data**:
   - Buckwalter Arabic Morphological Analyzer (BAMA) stem lists
   - Arabic WordNet noun/verb classifications
   - Quranic word frequency lists
   - MSA newspaper corpus frequency lists

### 11.3 Phase 3: Pattern-Based Matching (4–8 weeks)

1. **Replace verb form matching**: Remove the 15 `match` arms in `check_verb_form()` and use `KB-0004.matching_verb_patterns()`.

2. **Replace noun pattern matching**: Remove the hardcoded `WazanCandidate` pushes in `match_noun_patterns()` and use `KB-0004.matching_noun_patterns()`.

3. **Remove heuristic lists**: Delete `COMMON_NOUNS_3L` and `COMMON_VERBS_3L` constants.

4. **Performance validation**: Verify that KB-0004 lookups are faster or equal to the current heuristic matching.

### 11.4 Backward Compatibility

```rust
/// MOD-04 configuration option to control KB-0004 usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KbUsageMode {
    /// Only use heuristic lists (legacy behavior).
    HeuristicOnly,
    /// Prefer KB-0004; fall back to heuristics if KB not loaded.
    PreferKb,
    /// Require KB-0004; error if not loaded.
    KbRequired,
}

impl Default for KbUsageMode {
    fn default() -> Self { Self::PreferKb }
}
```

---

## 12. Serialization & Storage

### 12.1 Source Format (JSON)

Following existing KB conventions (`KB-0001` → JSON files):

```json
// patterns/verbs/form-I.json
{
  "kb_id": "KB-0004",
  "pattern_family": "verb",
  "entries": [
    {
      "id": "KB-0004:verb:form_I:basic_a",
      "verb_form": 1,
      "canonical_pattern": "فَعَلَ",
      "template_script": "C₁aC₂aC₃a",
      "consonant_count": 3,
      "default_pos": "Verb",
      "default_confidence": 0.30,
      "applies_to_root_types": ["sound", "mithal_wawi", "mithal_yai",
                                 "hamzated_first", "hamzated_middle", "hamzated_last"],
      "weak_variants": [
        {
          "variant_name": "ajwaf_wawi_perfect",
          "applies_to_root_type": "ajwaf_wawi",
          "template_script": "C₁āC₃",
          "condition": "perfect tense, active",
          "confidence_adjustment": 0.0
        },
        {
          "variant_name": "ajwaf_yai_perfect",
          "applies_to_root_type": "ajwaf_yai",
          "template_script": "C₁āC₃",
          "condition": "perfect tense, active",
          "confidence_adjustment": 0.0
        },
        {
          "variant_name": "naqis_wawi_perfect",
          "applies_to_root_type": "naqis_wawi",
          "template_script": "C₁aC₂ā",
          "condition": "perfect tense, 3ms",
          "confidence_adjustment": -0.05
        },
        {
          "variant_name": "naqis_yai_perfect",
          "applies_to_root_type": "naqis_yai",
          "template_script": "C₁aC₂ā",
          "condition": "perfect tense, 3ms",
          "confidence_adjustment": -0.05
        },
        {
          "variant_name": "doubled_perfect",
          "applies_to_root_type": "doubled",
          "template_script": "C₁aC₃C₃",
          "condition": "perfect tense, 3ms",
          "confidence_adjustment": 0.05
        }
      ],
      "examples": [
        {"word": "كَتَبَ", "meaning": "he wrote", "root": "كتب"},
        {"word": "جَلَسَ", "meaning": "he sat", "root": "جلس"},
        {"word": "قَالَ", "meaning": "he said", "root": "قول"},
        {"word": "رَمَى", "meaning": "he threw", "root": "رمي"},
        {"word": "مَدَّ", "meaning": "he extended", "root": "مدد"}
      ]
    },
    {
      "id": "KB-0004:verb:form_I:variant_i",
      "verb_form": 1,
      "canonical_pattern": "فَعِلَ",
      "template_script": "C₁aC₂iC₃a",
      "consonant_count": 3,
      "default_pos": "Verb",
      "default_confidence": 0.28,
      "applies_to_root_types": ["sound"],
      "examples": [
        {"word": "عَلِمَ", "meaning": "he knew", "root": "علم"},
        {"word": "فَهِمَ", "meaning": "he understood", "root": "فهم"}
      ]
    },
    {
      "id": "KB-0004:verb:form_I:variant_u",
      "verb_form": 1,
      "canonical_pattern": "فَعُلَ",
      "template_script": "C₁aC₂uC₃a",
      "consonant_count": 3,
      "default_pos": "Verb",
      "default_confidence": 0.28,
      "applies_to_root_types": ["sound"],
      "examples": [
        {"word": "حَسُنَ", "meaning": "he was beautiful", "root": "حسن"},
        {"word": "كَبُرَ", "meaning": "he grew large", "root": "كبر"}
      ]
    },
    {
      "id": "KB-0004:verb:form_I:imperfect_prefix",
      "verb_form": 1,
      "canonical_pattern": "يَفْعُلُ",
      "template_script": "yāC₁C₂uC₃u",
      "consonant_count": 3,
      "default_pos": "Verb",
      "default_confidence": 0.25,
      "applies_to_root_types": ["sound"]
    },
    {
      "id": "KB-0004:verb:form_I:imperative_prefix",
      "verb_form": 1,
      "canonical_pattern": "اِفْعَلْ",
      "template_script": "iC₁C₂aC₃",
      "consonant_count": 3,
      "default_pos": "Verb",
      "default_confidence": 0.20,
      "applies_to_root_types": ["sound"]
    }
  ]
}
```

```json
// overrides/stem-overrides.json
{
  "kb_id": "KB-0004",
  "entries": [
    {"stem_text": "رجل", "pos": "Noun", "confidence": 0.85,
     "pattern_candidates": [
       {"canonical_pattern": "فَعْل", "weight": 0.9}
     ],
     "source": "KB-0001:root", "frequency_in_corpus": "high"},
    {"stem_text": "كتب", "pos": "Verb", "confidence": 0.90,
     "pattern_candidates": [
       {"canonical_pattern": "فَعَلَ", "weight": 1.0}
     ],
     "source": "KB-0001:root", "frequency_in_corpus": "high"},
    {"stem_text": "جمل", "pos": "Noun", "confidence": 0.80,
     "pattern_candidates": [
       {"canonical_pattern": "فَعَل", "weight": 0.85}
     ],
     "source": "KB-0001:root", "frequency_in_corpus": "high"},
    {"stem_text": "قال", "pos": "Verb", "confidence": 0.90,
     "pattern_candidates": [
       {"canonical_pattern": "فَعَلَ", "weight": 1.0}
     ],
     "source": "KB-0001:root", "frequency_in_corpus": "high"},
    {"stem_text": "علم", "pos": "Noun", "confidence": 0.65,
     "secondary_pos": "Verb",  // Also a verb (he knew)
     "pattern_candidates": [
       {"canonical_pattern": "فِعْل", "weight": 0.6},
       {"canonical_pattern": "فَعَلَ", "weight": 0.4}
     ],
     "source": "KB-0001:root", "frequency_in_corpus": "high",
     "notes": "Noun (knowledge) and verb (he knew) readings are both common"},
    {"stem_text": "حكم", "pos": "Noun", "confidence": 0.60,
     "secondary_pos": "Verb",
     "source": "KB-0001:root", "frequency_in_corpus": "high",
     "notes": "Rule/judgment (noun) vs he ruled/judged (verb)"},
    {"stem_text": "نصر", "pos": "Verb", "confidence": 0.75,
     "source": "KB-0001:root", "frequency_in_corpus": "medium"}
  ]
}
```

### 12.2 Compiled Format

The compiled binary follows the same trie-based approach as KB-0001 (§9.2 in that spec), with two modifications:

1. **Dual trie structure**: Two separate tries for fast access:
   - **Pattern trie**: Keyed by canonical pattern (e.g., "فَعَلَ", "فَاعِل") → pattern metadata
   - **Stem trie**: Keyed by stem text (e.g., "رجل", "كتب") → POS override

2. **Pattern signature hash**: Each pattern gets a compact u64 signature for fast candidate selection:
   - Bits 0–3: consonant_count
   - Bits 4–7: prefix type (0=none, 1=hamza, 2=ta, 3=alif_nun, 4=sin_ta, etc.)
   - Bits 8–11: infix presence
   - Bits 12–15: suffix type
   - Bits 16–19: vowel pattern type (0=a/a, 1=a/i, 2=a/u, 3=ā/i, 4=ā/u, etc.)
   - Bits 20–23: gemination pattern
   - Bits 24–31: length constraints (min/max)

### 12.3 Size Budget

| Component | Estimated Size |
|-----------|---------------|
| Pattern entries (~150) | ~1.5 MB |
| Stem overrides (~5,000) | ~3 MB |
| Weak variants (~200) | ~1 MB |
| POS profiles (~50) | ~0.2 MB |
| Trie overhead | ~1 MB |
| **Total compiled** | **~6–8 MB** |

---

## 13. Performance Budget

| Operation | Target | Compared to Heuristic |
|-----------|--------|----------------------|
| `matching_verb_patterns()` (single form) | < 2 μs | Current: ~0.5 μs (match arm) → 4× slower, but still negligible |
| `matching_noun_patterns()` (all patterns) | < 10 μs | Current: ~2 μs (hardcoded pushes) → 5× slower, still < 0.1% of pipeline |
| `stem_pos_override()` (exact match) | < 0.5 μs | Current: O(n) scan of COMMON_NOUNS_3L (~90) → O(1) trie lookup → faster |
| `best_pos_for_3l_stem()` | < 15 μs | Current: O(n) scan + match arms → comparable |
| KB-0004 load time | < 100 ms | New: memory-mapped load |

---

## 14. Example Entries

### 14.1 Verb Form I — Sound Pattern

```json
{
  "id": "KB-0004:verb:form_I:basic_a",
  "pattern_family": "verb",
  "verb_form": 1,
  "canonical_pattern": "فَعَلَ",
  "template_script": "C₁aC₂aC₃a",
  "length_constraint": { "exact": 3 },
  "prefix_pattern": null,
  "structural_pattern": "C a C a C a",
  "default_pos": "Verb",
  "default_confidence": 0.30,
  "boosted_confidence": 0.35,
  "applies_to_root_types": ["sound", "mithal_wawi", "mithal_yai", "hamzated_first", "hamzated_middle", "hamzated_last"],
  "weak_variants": [
    {
      "variant_name": "ajwaf_perfect",
      "applies_to_root_type": "ajwaf_wawi",
      "template_script": "C₁āC₃",
      "condition": "perfect, active, 3ms"
    },
    {
      "variant_name": "ajwaf_yai_perfect",
      "applies_to_root_type": "ajwaf_yai",
      "template_script": "C₁āC₃",
      "condition": "perfect, active, 3ms"
    },
    {
      "variant_name": "naqis_perfect",
      "applies_to_root_type": "naqis_wawi",
      "template_script": "C₁aC₂ā",
      "condition": "perfect, active, 3ms"
    },
    {
      "variant_name": "naqis_yai_perfect",
      "applies_to_root_type": "naqis_yai",
      "template_script": "C₁aC₂ā",
      "condition": "perfect, active, 3ms"
    },
    {
      "variant_name": "doubled_perfect",
      "applies_to_root_type": "doubled",
      "template_script": "C₁aC₃C₃",
      "condition": "perfect, active, 3ms"
    }
  ],
  "semantic_modification": "Basic action (base form)",
  "examples": [
    {"word": "كَتَبَ", "meaning": "he wrote", "root": "كتب"},
    {"word": "قَالَ", "meaning": "he said", "root": "قول"},
    {"word": "مَدَّ", "meaning": "he extended", "root": "مدد"}
  ]
}
```

### 14.2 Noun Pattern — فَعْل (Masdar/Primary Noun)

```json
{
  "id": "KB-0004:noun:masdar:form_I:فَعْل",
  "pattern_family": "noun",
  "noun_type": "masdar",
  "canonical_pattern": "فَعْل",
  "template_script": "C₁aC₂C₃",
  "length_constraint": { "exact": 3 },
  "default_pos": "Noun",
  "default_confidence": 0.25,
  "derived_from_verb_form": 1,
  "applies_to_root_types": ["sound", "ajwaf_wawi", "ajwaf_yai", "doubled"],
  "weak_variants": [
    {
      "variant_name": "ajwaf",
      "applies_to_root_type": "ajwaf_wawi",
      "template_script": "C₁awC₃",
      "condition": "masdar, Form I"
    }
  ],
  "semantic_modification": "Verbal noun of Form I action",
  "examples": [
    {"word": "ضَرْب", "meaning": "hitting", "root": "ضرب"},
    {"word": "رَجُل", "meaning": "man", "root": "رجل"},
    {"word": "قَوْل", "meaning": "saying", "root": "قول"}
  ]
}
```

### 14.3 Stem Override — Ambiguous Case

```json
{
  "stem_text": "علم",
  "root": "علم",
  "pos": "Noun",
  "secondary_pos": "Verb",
  "confidence": 0.60,
  "pattern_candidates": [
    {"canonical_pattern": "فِعْل", "weight": 0.55},
    {"canonical_pattern": "فَعَلَ", "weight": 0.40},
    {"canonical_pattern": "فَعْل", "weight": 0.05}
  ],
  "root_type": "sound",
  "source": "KB-0001:root",
  "frequency_in_corpus": "high",
  "notes": "Noun (knowledge/ علم ) and verb (he knew / ‏علمَ‎) are both frequent. Noun slightly more common in MSA texts."
}
```

---

## 15. Open Questions & Future Work

### 15.1 Open Questions

1. **Should KB-0004 be a separate file from KB-0002 (Wazan Database)?**
   KB-0002 defines stem-level wazan templates with root-position mapping. KB-0004 would add POS profiles, confidence scores, and stem overrides. Decision: Keep separate because KB-0002 is phonological/structural while KB-0004 is lexico-semantic (POS). But cross-reference KB-0004 → KB-0002 via `canonical_pattern`.

2. **What is the initial seed size for stem overrides?**
   Current heuristics: ~167 entries (90 nouns + 77 verbs). Target for Phase 1: ~500 entries (all unique stems from the current test suite + Quranic vocabulary). Target for Phase 2: ~5,000 entries (merge with a standard Arabic word frequency list).

3. **How to handle stems with strong verb AND noun readings (e.g., علم)?**
   Use `secondary_pos` with lower confidence. The final POS is determined by the syntax parser (MOD-05) which can disambiguate based on sentential context. KB-0004 provides a probabilistic prior, not a definitive answer.

4. **Should quadriliteral roots be included?**
   Yes, but Phase 1 focuses on triliteral (3-letter) stems where the ambiguity is most acute. Quadriliteral roots are rarely ambiguous (they're almost always nouns or quadriliteral verbs, easily distinguished by other means).

### 15.2 Future Work

1. **Frequency-weighted confidence**: Use corpus frequency data to adjust confidence scores automatically. A stem that appears as a noun 90% of the time in a large corpus should get higher noun confidence.

2. **Contextual disambiguation rules**: Store n-gram patterns that help disambiguate — e.g., if a 3-letter stem follows the definite article ال, it's more likely to be a noun regardless of its baseline POS.

3. **Machine learning integration**: The stem override confidence scores could be learned from a POS-tagged corpus rather than hand-crafted.

4. **Dialectal variants**: Add dialectal POS profiles for Egyptian, Levantine, Gulf, and Maghrebi Arabic as KB plugins.

5. **Learner's edition**: A simplified KB-0004 focused on the top 3,000 stems for Arabic language learning applications.

---

## Appendix A: Current Heuristic → KB-0004 Mapping

| Current Code | KB-0004 Equivalent |
|-------------|-------------------|
| `COMMON_NOUNS_3L.contains(&stem_text)` → `return 0.0` | `stem_pos_override(stem_text)` where `pos != Verb` → `return 0.0` |
| `COMMON_VERBS_3L.contains(&stem_text)` → `return 0.35` | `stem_pos_override(stem_text)` where `pos == Verb` → `return boosted_confidence` |
| `match form { 1 if stem_len == 3 => 0.3, ... }` | `matching_verb_patterns(stem_text, stem_len, root_type)` → filter by `form` |
| `WazanCandidate { text: "فَاعِل", confidence: 0.2 }` | `matching_noun_patterns(stem_text, stem_len)` → find `فَاعِل` with its KB-0004 confidence |
| `determine_pos()` → `match wazan.category` | `pos_for_pattern(canonical_pattern, root_type)` |
| `if stem_text.ends_with('ة') → return 0.0` | Keep as fast-path guard (cheaper than KB lookup) |
| `if stem_text.ends_with("ون/ين/ات") → noun pattern` | Keep as fast-path guard |

## Appendix B: Rust Integration Types

```rust
/// KB-0004 runtime representation (loaded from compiled binary).
pub struct Kb0004 {
    // Compiled trie for patterns
    pattern_trie: PatternTrie,
    // Compiled trie for stem overrides
    stem_trie: StemTrie,
    // Loaded at pipeline start
}

// ── Pattern Types ──

#[derive(Debug, Clone)]
pub struct PatternEntry {
    pub id: String,
    pub pattern_family: PatternFamily,
    pub verb_form: Option<u8>,
    pub noun_type: Option<NounType>,
    pub canonical_pattern: String,
    pub template_script: String,
    pub default_pos: PartOfSpeech,
    pub default_confidence: f64,
    pub boosted_confidence: Option<f64>,
    pub length_constraint: LengthConstraint,
    pub structural_pattern: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternFamily {
    Verb,
    Noun,
    Particle,
}

#[derive(Debug, Clone)]
pub struct LengthConstraint {
    pub exact: Option<usize>,
    pub min: Option<usize>,
    pub max: Option<usize>,
}

// ── Stem Override Types ──

#[derive(Debug, Clone)]
pub struct StemOverride {
    pub stem_text: String,
    pub pos: PartOfSpeech,
    pub confidence: f64,
    pub secondary_pos: Option<PartOfSpeech>,
}

// ── Lookup Result ──

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub canonical_pattern: String,
    pub verb_form: Option<u8>,
    pub category: WazanCategory,
    pub pos: PartOfSpeech,
    pub confidence: f64,
    pub variant: Option<String>,
}
```
