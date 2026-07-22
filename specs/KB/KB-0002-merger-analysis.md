---
kb_id: KB-0002 (Revision)
title: KB Merger Analysis — Wazan Structural Database (KB-0002) vs Pattern POS Database (KB-0004)
version: 1.0.0-draft
status: Proposal
author: AGOS Linguistics Committee
created: 2026-07-22
updated: 2026-07-22
references:
  - KB-0002: Wazan Database (original wazan morphological patterns spec)
  - KB-0004: Noun Patterns (original derived noun specifications spec)
  - KB-0004: Wazan & Noun Pattern Database (proposed pattern-to-POS repository)
supersedes: KB-0002-merger-analysis
---

# KB-0002 Revision: Merger Analysis
## Unifying KB-0002 (Wazan Database), KB-0004 (Noun Patterns), and KB-0004 (Pattern POS Proposal)

---

## 1. Problem Statement

Three documents currently cover overlapping aspects of Arabic morphological patterns:

| Document | ID | Focus | Sections | Status |
|----------|-----|-------|---------|--------|
| Wazan Database | KB-0002 | Phonological templates, verb forms I–XV, weak root variants | 14 | Draft |
| Noun Patterns | KB-0004 (original) | Derived noun specifications, inflection, broken plurals | 22 | Draft |
| Pattern POS Repository | KB-0004 (proposed) | Pattern-to-POS mapping, stem overrides, MOD-04 integration | 15 | Proposal |

**Overlap**: All three define wazan patterns. KB-0002 and the old KB-0004 both list the same noun pattern templates (فَاعِل, مَفْعُول, فَعِيل, etc.) with near-identical examples. The proposed KB-0004 replicates some verb form definitions already in KB-0002.

**Gaps**: No document yet has a concrete JSON implementation. No runtime Rust trait exists. No compiled binary format has been tested.

---

## 2. Merger Decision: Keep Two KBs

Rather than merging all three into one monolithic KB, we **keep two separate KBs with clean boundaries**:

```
KB-0002: Wazan Structural Database
  └─ Answers: "What does this pattern LOOK like?"
  └─ Contains: phonological templates, verb form definitions,
               derived noun templates, weak root variants,
               quadriliteral patterns, root-position maps
  └─ Consumer: MOD-04 (pattern matching), MOD-09 (stem generation)

KB-0004: Noun & Pattern POS Database
  └─ Answers: "What IS this pattern? What POS does it indicate?"
  └─ Contains: POS profiles per pattern, confidence scores,
               stem-level POS overrides, inflection properties,
               broken plural tables, semantic roles
  └─ Consumer: MOD-04 (POS assignment), MOD-08 (knowledge graph),
               MOD-11 (explanation engine)
```

---

## 3. Boundary Definition

| Aspect | KB-0002 (Structural) | KB-0004 (POS & Noun Details) |
|--------|---------------------|------------------------------|
| **Granularity** | Phonological template | Lexico-semantic profile |
| **Primary data** | Wazan templates (ف-ع-ل placeholders) | POS, confidence, inflection classes |
| **Verb forms I–XV** | Templates, stem signatures | POS=Verb, confidence per form |
| **Noun templates** | Templates only (فَاعِل, مَفْعُول, etc.) | Full noun profiles: gender, plural, semantic role |
| **Weak root variants** | Surface-form templates for each root type | Confidence adjustments for weak variants |
| **Root-position mapping** | ✅ Yes (C₁, C₂, C₃ positions) | ❌ Referenced but not stored |
| **POS assignment** | ❌ (has `inherent_features` as generic key-value) | ✅ Core function (definitive POS per pattern) |
| **Confidence scoring** | ❌ | ✅ 0.0–1.0 per pattern + stem overrides |
| **3-letter stem disambiguation** | ❌ | ✅ (replaces COMMON_NOUNS_3L / VERBS_3L) |
| **Inflection details** | ❌ | ✅ Gender, declension, broken plural links |
| **Broken plural templates** | ❌ | ✅ ~35+ patterns with singular→plural mapping |
| **Nisbah (relative adj) rules** | ❌ | ✅ ya-nisba attachment rules |
| **Semantic roles** | ❌ | ✅ agent, patient, instrument, location, etc. |
| **Runtime lookup API** | ❌ | ✅ Rust trait `WazanPatternLookup` |
| **MOD-04 integration** | Referenced but not specified | ✅ Detailed integration plan + migration |
| **Pattern signature hashing** | ✅ (u64 signature for fast matching) | ❌ |

---

## 4. Content Migration Plan

### 4.1 Content Moving from Old KB-0004 (Noun Patterns) → KB-0002 (Structural)

These sections from the old KB-0004 are **phonological/structural** and belong in KB-0002:

| Old KB-0004 Section | Destination in KB-0002 | Reason |
|---------------------|----------------------|--------|
| 5. Verbal Noun (Masdar) Patterns | §6 (Derived Noun Wazan) — expand | Masdar templates are structural patterns |
| 6. Form I Unpredictable Patterns | §6 (new sub-section) | Same type: phonological templates for Form I masdars |
| 7. Active Participle Patterns | §6.2 (Active Participle) — expand | Participle wazans are structural |
| 8. Passive Participle Patterns | §6.3 (Passive Participle) — expand | Same as above |
| 9. Noun of Place/Time | §6.4 (Other Noun Wazan) — expand | Structural patterns |
| 10. Noun of Instrument | §6.4 — expand | Structural patterns |
| 11. Sifah Mushabbahah | §6.4 — expand | Adjective wazan templates |
| 12. Elative (Tafdil) | §6.4 — expand | Template only |
| 13. Nisbah | §6.4 — expand | Template only |
| 16. Weak Root Variants for Noun Patterns | §8 (Weak Root Pattern Variants) — expand | Same variant system applies to nouns |

### 4.2 Content Staying in KB-0004 (Noun & Pattern POS Database)

These sections are **lexico-semantic/inflectional** and belong in KB-0004:

| Old KB-0004 Section | Status | Reason |
|---------------------|--------|--------|
| 17. Noun Pattern Matching Algorithm | Merge with KB-0004 proposal §9 | Both describe how MOD-04 uses patterns |
| 14. Broken Plural (Jamʿ Taksir) Patterns | Keep in KB-0004 | Inflectional data — not structural |
| N/A — Gender, declension per noun type | Add to KB-0004 | New section needed |
| N/A — Pluralization (sound + broken) per noun type | Add to KB-0004 | New section needed |
| 4.1 — `NounPatternEntry` sections on gender, declension, broken_plurals | Keep in KB-0004 | Inflection data |

### 4.3 Content from KB-0004 (Proposal) — Where It Goes

| Proposed Section | Destination | Reason |
|-----------------|-------------|--------|
| 5. WazanPatternEntry | KB-0004 §4 (Schema) | POS + confidence is KB-0004's core |
| 6. Supported Pattern Types | KB-0004 §6 | Pattern catalog with POS profiles |
| 7. Weak Root & Variant Handling | KB-0002 §8 (move the template parts) | Templates stay in KB-0002; confidence adjustments stay in KB-0004 |
| 8. Runtime Lookup API | KB-0004 §8 | MOD-04 integration is KB-0004's job |
| 9. Integration with MOD-04 | KB-0004 §9 | Migration plan |
| 10. Integration with KB-0001/0002 | KB-0004 §10 | Cross-KB references |
| 11. Migration Path | KB-0004 §11 | Phase plan |
| 13. Performance Budget | KB-0004 §13 | Validation targets |

---

## 5. Updated KB-0002: Wazan Structural Database (Revised Scope)

### 5.1 Purpose

The **authoritative register of Arabic morphological patterns** (أوزان, `awzān`). Provides the phonological templates, root-position maps, and weak-root surface variants that MOD-04 uses to **match input stems against morphological patterns**.

### 5.2 Contents (Updated)

| Section | Content | Source |
|---------|---------|--------|
| 1–4 | Purpose, scope, data model, schema | KB-0002 original |
| 5 | Verb form wazans I–XV (templates, prefixes, stem signatures) | KB-0002 original |
| 6 | Derived noun wazans: masdar, participles, place/time, instrument, adjectives, nisbah | KB-0002 original + expanded from old KB-0004 §§5–13 |
| 7 | Quadriliteral patterns | KB-0002 original |
| 8 | Weak root pattern variants (sound, mithal, ajwaf, naqis, doubled, hamzated) | KB-0002 original + expanded from old KB-0004 §16 |
| 9 | Pattern signature hashing (u64 algorithm) | KB-0002 original |
| 10 | Serialization & storage | KB-0002 original |

### 5.3 Schema Changes

The `WazanEntry` in KB-0002 **loses** the `inherent_features` field (moves to KB-0004) but **gains** expanded weak-variant coverage for derived noun patterns:

```diff
 WazanEntry:
   # --- Identity --- (unchanged)
   # --- Classification --- (unchanged)
   # --- Phonological Template --- (unchanged)
-  # --- Morphological Features --- (REMOVED — moved to KB-0004)
-  inherent_features: FeatureMap
   # --- Morphological Behavior --- (unchanged)
-  # --- Semantics --- (REMOVED — moved to KB-0004)
-  core_meaning, core_meaning_ar, semantic_modification
+  # --- Weak Noun Variants --- (NEW — expanded from old KB-0004)
+  weak_noun_variants: NounPatternVariant[]
   # --- Examples --- (unchanged, but expanded with noun examples)
   # --- Attestation --- (unchanged)
```

---

## 6. Updated KB-0004: Pattern POS Database (Revised Scope)

### 6.1 Purpose

The **definitive POS and inflection database for Arabic morphological patterns**. Assigns part-of-speech, confidence scores, inflection properties, and stem-level overrides to every wazan. This is the authoritative replacement for `COMMON_NOUNS_3L`, `COMMON_VERBS_3L`, and the `determine_pos()` function.

### 6.2 Contents (Updated)

| Section | Content | Source |
|---------|---------|--------|
| 1 | Motivation & problem statement | KB-0004 proposal §1 |
| 2 | Design goals | KB-0004 proposal §2 |
| 3 | Architecture overview | KB-0004 proposal §3 |
| 4 | Data schema (`PatternProfileEntry`, `StemOverrideEntry`, `BrokenPluralEntry`) | Merged from KB-0004 proposal §4 + old KB-0004 §4 |
| 5 | POS profiles for verb form patterns | KB-0004 proposal §5.3 (table) |
| 6 | POS profiles for noun patterns (35+ templates with confidence) | KB-0004 proposal §6.2 (expanded table) |
| 7 | Inflection properties (gender, declension, pluralization per noun type) | New — from old KB-0004 |
| 8 | Broken plural templates (30+ patterns with singular→plural mapping) | Old KB-0004 §14 |
| 9 | Stem-level POS overrides (replaces the heuristic lists) | KB-0004 proposal §5.3 |
| 10 | Weak root confidence adjustments | KB-0004 proposal §7.1 |
| 11 | Runtime lookup API (Rust trait) | KB-0004 proposal §8 |
| 12 | Integration with MOD-04 (replacing heuristic code paths) | KB-0004 proposal §9 |
| 13 | Integration with KB-0001 & KB-0002 | KB-0004 proposal §10 |
| 14 | Migration path (3 phases) | KB-0004 proposal §11 |
| 15 | Serialization & storage | KB-0004 proposal §12 |
| 16 | Performance budget | KB-0004 proposal §13 |
| 17 | Example entries | KB-0004 proposal §14 |

### 6.3 Schema (Merged)

```yaml
# KB-0004: Pattern POS Database
# Core entity: PatternProfileEntry (assigns POS + confidence to a KB-0002 wazan)

PatternProfileEntry:
  # --- Identity ---
  id: string                              # "KB-0004:verb:form_I:basic_a"
  wazan_id: string | null                 # Cross-reference to KB-0002 wazan entry
                                           # e.g., "KB-0002:verb:I:فَعَلَ"

  # --- POS & Confidence ---
  pattern_family: "verb" | "noun" | "particle"
  verb_form: integer | null               # 1–15
  noun_type: NounType | null              # "masdar", "ism_fail", "ism_maful", etc.
  canonical_pattern: string               # e.g., "فَعَلَ", "فَاعِل", "مَفْعُول"

  default_pos: PartOfSpeech               # The POS this pattern indicates
  default_confidence: float                # Base confidence (0.0–1.0)
  boosted_confidence: float | null         # For stems with known verb-form attestation
  root_type_confidence: RootTypeConfidence[]  # Per-root-type confidence adjustments

  # --- Inflection (for noun patterns) ---
  gender: "masculine" | "feminine" | "both" | null
  declension: "triptote" | "diptote" | "indeclinable" | null
  feminine_form_pattern: string | null     # e.g., "فَاعِلَة" for "فَاعِل"
  sound_plural_masculine: string | null
  sound_plural_feminine: string | null
  broken_plural_links: BrokenPluralLink[]  # Common broken plural patterns

  # --- Semantics ---
  semantic_role: string | null             # "agent", "patient", "action", "instrument", etc.
  semantic_modification: string | null     # How this pattern modifies root meaning

  # --- Cross-Reference ---
  attestation: Attestation
  examples: Example[]

RootTypeConfidence:
  root_type: RootType                      # e.g., "ajwaf_wawi", "naqis_yai"
  confidence_adjustment: float             # +/- adjustment from base
  notes: string | null

NounType:
  "masdar" | "ism_fail" | "ism_maful" | "ism_makan" | "ism_zaman" |
  "ism_alah" | "sifah_mushabbahah" | "tafdil" | "nisbah" | "jam_taksir" |
  "ism_marrati" | "ism_hayati" | "ism_jins" | "noun_other"

StemOverrideEntry:                         # (unchanged from proposal)
  stem_text: string
  pos: PartOfSpeech
  confidence: float
  secondary_pos: PartOfSpeech | null
  pattern_candidates: PatternCandidate[]

BrokenPluralEntry:                         # (new — from old KB-0004)
  id: string                               # "KB-0004:jam_taksir:فُعَّال"
  canonical_pattern: string                # "فُعَّال"
  template_script: string                  # "C₁uC₂C₂āC₃"
  gender: "masculine" | "feminine" | "both"
  declension: "triptote" | "diptote"
  applies_to_noun_types: NounType[]
  applies_to_singular_patterns: string[]   # Which singular patterns use this plural
  frequency: "very_common" | "common" | "moderate" | "rare"
  examples: Example[]
```

---

## 7. Cross-KB Data Flow

```
                    ┌─────────────────────────────────────────┐
                    │               MOD-04                     │
                    │         MorphologicalParser              │
                    └─────────────────────────────────────────┘
                         │                    │
                    ┌────▼─────┐         ┌────▼──────┐
                    │ KB-0002  │         │ KB-0004   │
                    │Structural│         │ POS &     │
                    │Templates │         │Inflection │
                    └──────────┘         └───────────┘
                         │                    │
                         │ Provides:          │ Provides:
                         │ • C₁aC₂aC₃a       │ • Verb → confidence 0.30
                         │ • C₁āC₃ (ajwaf)   │ • فَاعِل → Noun, 0.30
                         │ • prefix/infix     │ • فَعْل → Noun, 0.25
                         │ • root_position    │ • Stem "رجل" → Noun, 0.85
                         │ • weak variants    │ • فَاعِل → gender=masc
                         │                    │ • فَاعِل → broken plural فُعَّال
                         │                    │ • مَفْعَل → type=place/time
                         ▼                    ▼
                    ┌─────────────────────────────────────────┐
                    │           KB-0001 (Roots)                │
                    │   Validates: is this verb form attested?  │
                    └─────────────────────────────────────────┘
```

### Lookup Path

```pseudo
1. stem = "رجل" (3-letter, no definite article)

2. MOD-04 → KB-0004: StemOverride for "رجل"?
   → YES: pos=Noun, confidence=0.85
   → Return Noun (skip pattern matching)

3. If NO StemOverride:
   a. MOD-04 → KB-0002: Which patterns match this stem length?
      → Form I (C₁aC₂aC₃a): length=3, prefix=none → candidate
      → Form I (ajwaf variant C₁āC₃): length=2 → no match (stem_len=3)
      → Noun pattern فَعْل (C₁aC₂C₃): length=3 → candidate
      → Noun pattern فِعْل (C₁iC₂C₃): length=3 → candidate
      → ... all 35+ noun patterns checked

   b. MOD-04 → KB-0004: What POS + confidence for each matched pattern?
      → Form I (فَعَلَ): pos=Verb, confidence=0.30
      → فَعْل pattern: pos=Noun, confidence=0.25
      → فِعْل pattern: pos=Noun, confidence=0.25
      → Highest: Verb 0.30 > Noun 0.25 → Verb (default heuristic)

   c. MOD-04 → KB-0004: Any per-root-type confidence adjustment?
      → Root "رجل" is sound type
      → No adjustment → Verb stays at 0.30

   d. Without StemOverride → Verb wins (0.30 > 0.25)
      With StemOverride → Noun wins (0.85 > 0.30)
      → This is exactly why StemOverride is needed!
```

---

## 8. File Organization After Revision

### KB-0002 (Structural)

```
KB-0002/
├── metadata.json
├── patterns/
│   ├── verbs/
│   │   ├── form-I.json           # فَعَلَ + variants
│   │   ├── form-II.json          # فَعَّلَ
│   │   ├── form-III.json         # فَاعَلَ
│   │   ├── form-IV.json          # أَفْعَلَ
│   │   ├── form-V.json           # تَفَعَّلَ
│   │   ├── form-VI.json          # تَفَاعَلَ
│   │   ├── form-VII.json         # اِنْفَعَلَ
│   │   ├── form-VIII.json        # اِفْتَعَلَ
│   │   ├── form-IX.json          # اِفْعَلَّ
│   │   ├── form-X.json           # اِسْتَفْعَلَ
│   │   ├── form-XI-XV.json       # Rare forms (combined)
│   │   └── quadriliteral.json    # فَعْلَلَ + QII, QIII
│   ├── nouns/
│   │   ├── masdar-form-I.json    # ~40+ Form I masdar patterns
│   │   ├── masdar-regular.json   # Forms II–X masdar templates
│   │   ├── participle-active.json    # فَاعِل, مُفَعِّل, etc.
│   │   ├── participle-passive.json   # مَفْعُول, مُفَعَّل, etc.
│   │   ├── place-time.json           # مَفْعَل, مَفْعِل
│   │   ├── instrument.json           # مِفْعَل, مِفْعَال, مِفْعَلَة
│   │   ├── adjective.json            # فَعِيل, فَعْلَان, أَفْعَل
│   │   └── nisbah.json               # فَعْلِيّ, فِعْلِيّ
│   └── weak-variants/
│       ├── ajwaf.json             # Hollow root variants
│       ├── naqis.json             # Defective root variants
│       ├── mithal.json            # Assimilated root variants
│       ├── doubled.json           # Geminate root variants
│       └── hamzated.json          # Hamzated root variants
└── signatures/
    └── pattern-hash-table.json    # u64 signature → pattern_id map
```

### KB-0004 (POS & Inflection)

```
KB-0004/
├── metadata.json
├── profiles/
│   ├── verb-pos-profiles.json        # POS + confidence per verb form
│   ├── noun-pos-profiles.json        # POS + confidence per noun pattern
│   └── root-type-adjustments.json    # Confidence adjustments per root type
├── overrides/
│   └── stem-overrides.json           # ~2K–5K stem POS overrides
├── inflection/
│   ├── noun-gender.json              # Default gender per noun pattern
│   ├── noun-declension.json          # Declension class per noun pattern
│   ├── sound-plurals.json            # Sound plural templates
│   └── broken-plurals.json           # ~35+ broken plural templates
└── examples/
    └── pattern-examples.json         # Curated examples per pattern
```

---

## 9. Summary of Changes

### What happens to each existing document?

| Document | Action | Result |
|----------|--------|--------|
| **KB-0002** (Wazan Database) | **Keep + expand** — add noun wazan templates from old KB-0004, expand weak variants, remove `inherent_features` | Updated KB-0002 with cleaner scope |
| **KB-0004** (Noun Patterns — old) | **Deprecate** — split content: structural templates → KB-0002, inflection/POS → new KB-0004 | Old KB-0004 marked as superseded |
| **KB-0004** (Pattern POS proposal) | **Adopt + merge** with old KB-0004 inflection content → becomes the new KB-0004 | New KB-0004 with POS + inflection |

### Cross-reference updates

All documents that reference KB-0002 or KB-0004 must be updated:

| Referencing Document | Current Reference | Updated Reference |
|---------------------|-------------------|-------------------|
| KB-0001: Roots Database | References both KB-0002 (wazan) and KB-0004 (noun patterns) | Unchanged — both still exist |
| KB-0003: Verb Forms | References KB-0002 (patterns) | Unchanged |
| KB-0005: Particles | Independent | Unchanged |
| KB-0006: Pronouns | Independent | Unchanged |
| KB-0007: Morphological Features | References KB-0004 (noun features) | KB-0004 (new) — same reference, different content |
| SPEC-0101: Morphology Engine | References KB-0002 + KB-0004 | Unchanged |

### Next Steps

1. **Update KB-0002 spec**: Add noun wazan templates from old KB-0004, remove `inherent_features`, expand weak variants section with noun patterns
2. **Rewrite KB-0004 spec**: Merge the proposal's POS content with the old KB-0004's inflection content  
3. **Mark old KB-0004 as superseded**: Add deprecation notice to `specs/KB/KB-0004-noun-patterns.md`
4. **Update KB-OVERVIEW.md**: Reflect the new scope of KB-0002 vs KB-0004
5. **Begin Phase 1 implementation**: Seed JSON files from heuristic lists
