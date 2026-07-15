---
title: AGOS Knowledge Base Suite — Overview & Architecture
version: 1.0.0
status: Draft
author: AGOS Linguistics Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0001-C9: Performance Targets & Constraints
  - KB-0001: Roots Database
  - KB-0002: Wazan Database
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
---

# AGOS Knowledge Base Suite — Overview & Architecture

## 1. Introduction

This document provides an **architectural overview** of the AGOS Knowledge Base (KB) suite — seven interconnected linguistic databases that collectively model the Arabic morphological system. The KB suite is the foundational data layer of the AGOS platform, consumed by morphological analysis (MOD-04), syntactic parsing (MOD-05), bytecode generation (MOD-09), knowledge graph resolution (MOD-08), and the explanation engine (MOD-11).

### 1.1 Purpose

The KB overview answers:

- **What does each KB cover?** — Scope, content, and key metrics.
- **How do the KBs relate?** — Dependency graph, data flow, and cross-KB interfaces.
- **What are the combined resource budgets?** — Total size, lookup performance, and compilation targets.
- **How does the suite integrate with the AGOS pipeline?** — Which modules load which KBs, and in what order.

### 1.2 KB Suite Bill of Materials

| KB | ID | Title | Sections | Lines | Scope |\n|-----|----|-------|----------|-------|-------|\n| **KB-0001** | Roots | Roots Database | 14 | ~1,530 | ~15,000–20,000 Arabic roots\n| **KB-0002** | Wazan | Wazan Database — Morphological Patterns | 14 | ~1,640 | ~300–450 morphological patterns\n| **KB-0003** | Verb Forms | Verb Forms — Conjugation Paradigms | 18 | ~1,820 | ~180–250 conjugation tables\n| **KB-0004** | Nouns | Noun Patterns — Derived Noun Specifications | 22 | ~1,670 | ~135–180 noun patterns\n| **KB-0005** | Particles | Particles — Grammatical & Functional Words | 22 | ~1,560 | ~120–200 particle entries\n| **KB-0006** | Pronouns | Pronouns — Personal, Demonstrative & Relative | 19 | ~1,290 | ~60–80 pronoun entries\n| **KB-0007** | Features | Morphological Features — Taxonomy & Encoding | 20 | ~2,090 | 19 features, ~107 values, ~33 rules\n| **Total** | — | — | **129** | **~11,600** | Comprehensive Arabic morphology model\n\n---

## 2. KB Dependency Architecture

### 2.1 Dependency Graph

The KB suite forms a **directed acyclic graph (DAG)** with clear dependency relationships:

```diff
  KB-0001: Roots (جذور)
  │
  ├──► KB-0002: Wazan (أوزان)        — Roots + Wazan = Stems (central generative mechanism)
  │     │
  │     ├──► KB-0003: Verb Forms (تصريف)  — Stem × Form × Class = Conjugated verb
  │     └──► KB-0004: Noun Patterns (أوزان) — Stem × Pattern = Derived noun
  │
  ├──► KB-0005: Particles (حروف)     — Fast-path, no root dependency ✓
  ├──► KB-0006: Pronouns (ضمائر)     — Fast-path, no root dependency ✓
  └──► KB-0007: Features (خصائص)     — Cross-cutting feature taxonomy ✓
        │
        └── Referenced by ALL KBs (shared feature vocabulary)
```

### 2.2 Dependency Matrix

| KB | Depends On | Consumed By | Files |\n|-----|-----------|-------------|-------|\n| **KB-0001** | — | KB-0002, KB-0003, KB-0004, KB-0007 | `KB-0001-roots-database.md`\n| **KB-0002** | KB-0001 | KB-0003, KB-0004 | `KB-0002-wazan-database.md`\n| **KB-0003** | KB-0001, KB-0002 | KB-0007 | `KB-0003-verb-forms.md`\n| **KB-0004** | KB-0001, KB-0002, KB-0003 | KB-0007 | `KB-0004-noun-patterns.md`\n| **KB-0005** | — (independent) | — | `KB-0005-particles.md`\n| **KB-0006** | — (independent) | — | `KB-0006-pronouns.md`\n| **KB-0007** | KB-0001, KB-0002, KB-0003, KB-0004, KB-0005, KB-0006 | All | `KB-0007-morphological-features.md`\n\n### 2.3 Independence vs. Dependency

Three KBs are **fully independent** of root derivation:

| KB | Reason |\n|-----|--------|\n| **KB-0005** (Particles) | Particles have no root — they are fixed, invariable function words. Checked **first** in the MOD-04 fast path.\n| **KB-0006** (Pronouns) | Pronouns have no root — they are a closed class of ~60–80 entries. Checked **second** in the MOD-04 fast path.\n| **KB-0007** (Features) | The feature taxonomy is a **cross-cutting vocabulary** referenced by all KBs. It depends on other KBs for valid feature values but is logically self-contained.\n\nFour KBs form the **core morphological chain**:

| KB | Dependency | Role |\n|-----|-----------|------|\n| **KB-0001** (Roots) | Foundation | Provides the consonantal skeleton\n| **KB-0002** (Wazan) | KB-0001 → KB-0002 | Provides the pattern template\n| **KB-0003** (Verb Forms) | KB-0001 + KB-0002 → KB-0003 | Provides full conjugation tables\n| **KB-0004** (Nouns) | KB-0001 + KB-0002 → KB-0004 | Provides derived noun specifications\n\n---

## 3. Per-KB Summary

### 3.1 KB-0001: Roots Database

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | Authoritative register of Arabic roots (جذور). Every root extracted from a stem must be matched against KB-0001.\n| **Scope** | ~15,000–20,000 roots (Classical Arabic + MSA). Triliteral (~85%) and quadriliteral (~12–15%).\n| **Root types** | Sound (صحيح سالم), Weak (معتل — mithal/ajwaf/naqis/lafif), Hamzated (مهموز), Doubled (مضاعف)\n| **Entries per root** | Verb forms (I–XV), derived nouns, semantic fields, cross-references, attestations, weak root variants\n| **Storage** | Source: ~200 MB YAML/JSON → Compiled: ~20–80 MB binary trie\n| **Lookup** | O(k) trie traversal (k = 3–4 characters). Exact lookup < 1 μs. Fuzzy lookup < 10 ms.\n| **Key consumers** | MOD-04 (root extraction), MOD-08 (knowledge graph resolution)\n\n### 3.2 KB-0002: Wazan Database

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | Authoritative register of Arabic morphological patterns (أوزان). Every word form is the result of applying a wazan to a root.\n| **Scope** | ~300–450 patterns: verb forms I–XV (~30–40), derived noun patterns (~80–120), weak root variants (~150–200), quadriliteral (~15–25)\n| **Pattern types** | Verb (I–XV + QI–QIII), Noun (masdar, participle, place, instrument, adjective, elative, nisbah), Weak root variants\n| **Key innovation** | Pattern signature hashing (u64) for O(1) template matching. Reverse index mapping stem patterns to wazans.\n| **Storage** | Source: ~50 MB YAML → Compiled: ~10–40 MB hash index\n| **Performance** | Pattern signature lookup < 500 ns. Full wazan identification < 20 μs.\n| **Key consumers** | MOD-04 (wazan identification), MOD-08 (pattern resolution)\n\n### 3.3 KB-0003: Verb Forms

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | Full inflectional paradigms for all verb forms. KB-0002 defines the **stem**; KB-0003 defines the **complete set of inflected forms**.\n| **Scope** | ~180–250 paradigm tables across 15 conjugation classes: sound, hollow (wawi/yai), defective (wawi/yai), assimilated (wawi/yai), doubled, hamzated (first/mid/last), lafif (mafruq/makrun), sound quadrilateral, weak quadrilateral\n| **Paradigm slots** | 13 per table: 3ms, 3fs, 2ms, 2fs, 1s, 3md, 3fd, 2d, 3mp, 3fp, 2mp, 2fp, 1p\n| **Tenses & moods** | Perfect (past), Imperfect (present/future) × 4 moods (indicative, subjunctive, jussive, energetic + I/II)\n| **Storage** | Source: ~50–80 MB YAML → Compiled: ~15–30 MB table binary\n| **Performance** | Single slot conjugation < 1 μs (pre-computed). Full 13-slot paradigm < 30 μs.\n| **Key consumers** | MOD-04 (verb analysis), MOD-09 (verb generation), MOD-11 (pedagogical tables)\n\n### 3.4 KB-0004: Noun Patterns

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | Detailed specifications for derived noun patterns. Extends KB-0002's stem-level wazan templates with full inflectional behavior, broken plural mappings, and semantic roles.\n| **Scope** | ~135–180 patterns: masdar (Form I unpredictable ~40+ + Forms II–X regular ~9), participles (~22), place/time (~6), instrument (~6), adjectives (~14), elative (~4), nisbah (~4), broken plurals (~30–40), other (~6)\n| **Inflectional data** | Gender, declension class (triptote/diptote/indeclinable), feminine form, sound plural, broken plural mappings (with frequency rankings)\n| **Broken plurals** | ~30+ templates: paucity (4) + multitude (12+) with concordance table mapping singular patterns to plural patterns\n| **Weak variants** | Full tables for active participle (11 root types), passive participle (6), masdar (6), broken plural (3)\n| **Storage** | Source: ~30–50 MB YAML → Compiled: ~10–30 MB table binary\n| **Performance** | Noun pattern lookup < 2 μs. Broken plural generation < 5 μs.\n| **Key consumers** | MOD-04 (noun analysis), MOD-08 (feature resolution), MOD-09 (noun generation)\n\n### 3.5 KB-0005: Particles

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | Authoritative register of Arabic particles (حروف المعاني). Fast-path KB checked **first** in MOD-04 before root extraction.\n| **Scope** | ~120–200 entries across 13+ functional categories: prepositions (17 primary + compounds), conjunctions (10 coordinating + subordinating), subjunctive particles (6), jussive particles (5), conditional particles (12), interrogative particles (6), negative particles (8), vocative particles (6), inna & sisters (7), kāna & sisters (14), answer/exception (8+), masdar-forming (4), other (10+)\n| **Governance rules** | Each particle specifies its grammatical effect: case government (genitive/accusative/nominative), mood government (subjunctive/jussive), and government type (independent/requires complement)\n| **Disambiguation** | Homograph resolution algorithm for shared forms (e.g., مَا has 5+ interpretations). Scoring system based on syntactic context.\n| **Storage** | Source: ~1–2 MB YAML → Compiled: ~2–5 MB hash index (smallest KB)\n| **Performance** | Fast-path lookup < 500 ns. Homograph disambiguation < 1 μs.\n| **Key consumers** | MOD-04 (fast path, Step 3.1), MOD-05 (syntactic governance), MOD-11 (explanations)\n\n### 3.6 KB-0006: Pronouns

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | Authoritative register of Arabic pronouns (الضمائر). Fast-path KB checked **second** in MOD-04.\n| **Scope** | ~60–80 entries: attached personal pronouns (15–20: subject, object, possessive), detached personal pronouns (15: nominative + accusative إِيَّايَ series), demonstratives (12–15: near/mid/far × gender/number), relative pronouns (10–12: definite + indefinite), interrogative pronouns (8–12), conditional pronouns (8–10)\n| **Features per entry** | Pronoun type, person (1st/2nd/3rd), number (singular/dual/plural), gender (masculine/feminine/common), attachment type (standalone/suffix/prefix), phonetic variants, script forms\n| **Phonological variants** | Vowel-dependent form changes (e.g., 1s object -نِي after consonant vs. -ي after vowel; 3ms -هُ vs. -هِ)\n| **Anaphora resolution** | Scoring algorithm for antecedent matching: +3 for full person/number/gender match, +2 for person+number, +1 for person-only\n| **Storage** | Source: ~0.5–1 MB YAML → Compiled: ~1–2 MB hash index\n| **Performance** | Fast-path lookup < 500 ns. Anaphora resolution < 5 μs (10 candidate search).\n| **Key consumers** | MOD-04 (fast path, Step 3.2), MOD-05 (anaphora resolution), MOD-11 (reference explanations)\n\n### 3.7 KB-0007: Morphological Features

| Aspect | Detail |\n|--------|--------|\n| **Purpose** | The authoritative feature taxonomy for AGOS. Defines 19 morphological features, their allowed values, bitfield encoding (64-bit per RFC-0002), agreement rules, inference rules, and cross-feature constraints.\n| **Feature count** | 19 features: 1 POS, 8 inflectional (gender, number, person, tense, mood, voice, case, state), 5 derivational (verb_form, noun_type, pronoun_type, transitivity, root_type), 2 prosodic (stress_pattern, syllable_count), 3 orthographic (has_shadda, has_madd, has_hamza)\n| **Bitfield layout** | Bits 0–49 encode all 19 features. Bits 50–63 reserved for plugins. Complete `pack_features` and `unpack_features` functions provided.\n| **Rules** | 6 agreement rules (subject-verb, noun-adjective, government, mood-government), 15 inference rules (default values + cross-feature inference), 12+ constraints (valid combinations, mutual exclusions, dependencies)\n| **Validation** | 15+ validation rules (VAL-001 through VAL-030) ensuring feature correctness at parse time.\n| **Storage** | Source: ~0.5–1 MB YAML → Compiled: ~1–2 MB feature map + rule table\n| **Performance** | Feature pack < 500 ns. Feature unpack < 300 ns. Agreement check < 1 μs.\n| **Key consumers** | ALL modules that deal with morphological features. Central reference for the 64-bit feature bitfield.\n\n---

## 4. Combined Resource Budgets

### 4.1 Size Budgets (Compiled)

| KB | Compact (Level 1) | Full (Level 2) | Format |\n|-----|-------------------|----------------|--------|\n| **KB-0001** (Roots) | ~20 MB | ~80 MB | Binary trie\n| **KB-0002** (Wazan) | ~10 MB | ~40 MB | Hash index\n| **KB-0003** (Verb Forms) | ~15 MB | ~30 MB | Table binary\n| **KB-0004** (Noun Patterns) | ~10 MB | ~30 MB | Table binary\n| **KB-0005** (Particles) | ~2 MB | ~5 MB | Hash index\n| **KB-0006** (Pronouns) | ~1 MB | ~2 MB | Hash index\n| **KB-0007** (Features) | ~1 MB | ~2 MB | Feature map + rule table\n| **Total** | **~59 MB** | **~189 MB** | All KBs combined\n\n### 4.2 Performance Budgets

| Operation | KB-0001 | KB-0002 | KB-0003 | KB-0004 | KB-0005 | KB-0006 | KB-0007 |\n|-----------|---------|---------|---------|---------|---------|---------|---------|\n| **Primary lookup** | < 1 μs | < 500 ns | < 1 μs | < 2 μs | < 500 ns | < 500 ns | < 200 ns\n| **Full analysis** | < 10 ms | < 20 μs | < 30 μs | < 30 μs | < 1 μs | < 3 μs | < 5 μs\n| **KB load (compact)** | < 50 ms | < 25 ms | < 25 ms | < 25 ms | < 10 ms | < 5 ms | < 5 ms\n| **KB load (full)** | < 100 ms | < 50 ms | < 50 ms | < 50 ms | < 10 ms | < 5 ms | < 5 ms\n| **Memory (compact)** | ~20 MB | ~10 MB | ~15 MB | ~10 MB | ~2–5 MB | ~1–2 MB | ~1–2 MB |\n| **Memory (full)** | ~80 MB | ~40 MB | ~30 MB | ~30 MB | ~2–5 MB | ~1–2 MB | ~1–2 MB |\n\n### 4.3 Combined Pipeline Latency

Under realistic conditions (parsing a typical Arabic sentence of ~15 tokens):

```diff
  Token preprocessing & clitic stripping:    ~1 μs per token
  Fast-path check (KB-0005 + KB-0006):      ~1 μs per token
  Root extraction (KB-0001):                ~5 μs per non-particle token
  Wazan identification (KB-0002):           ~10 μs per stem
  Conjugation/noun analysis (KB-0003/4):    ~15 μs per analyzed token
  Feature extraction & validation (KB-0007): ~3 μs per token
  ────────────────────────────────────────────────────────
  Estimated total per token:                ~35 μs
  Estimated total per 15-token sentence:    ~525 μs (< 1 ms)
```

### 4.4 Source Line Counts

| KB | Sections | Lines (approx) | File |\n|-----|----------|----------------|------|\n| KB-0001 | 14 | 1,530 | `KB-0001-roots-database.md`\n| KB-0002 | 14 | 1,640 | `KB-0002-wazan-database.md`\n| KB-0003 | 18 | 1,820 | `KB-0003-verb-forms.md`\n| KB-0004 | 22 | 1,670 | `KB-0004-noun-patterns.md`\n| KB-0005 | 22 | 1,560 | `KB-0005-particles.md`\n| KB-0006 | 19 | 1,290 | `KB-0006-pronouns.md`\n| KB-0007 | 20 | 2,090 | `KB-0007-morphological-features.md`\n| **Total** | **129** | **~11,600** | **7 files**\n\n---

## 5. Pipeline Integration

### 5.1 KB Loading Sequence (MOD-04 Startup)

```diff
  MOD-04: MorphologicalParser — Initialization
  ┌─────────────────────────────────────────────────────┐
  │ Step 1.1: Load KB-0001 (Roots)     — Binary trie    │
  │ Step 1.2: Load KB-0002 (Wazan)     — Hash index     │
  │ Step 1.3: Load KB-0003 (Verb Forms) — Table binary  │
  │ Step 1.4: Load KB-0004 (Noun Patterns) — Table bin  │
  │ Step 1.5: Load KB-0005 (Particles) — Hash index     │
  │ Step 1.6: Load KB-0006 (Pronouns)  — Hash index     │
  │ Step 1.7: Load KB-0007 (Features)  — Feature map    │
  └─────────────────────────────────────────────────────┘
```

### 5.2 KB Usage During Token Analysis

```diff
  For each token:
  │
  ├── Step 3.1: Fast path — KB-0005 (Particles)
  │   └── If particle found → skip to next token
  │
  ├── Step 3.2: Fast path — KB-0006 (Pronouns)
  │   └── If pronoun found → skip to next token
  │
  ├── Step 3.3+: Main path — KB-0001 (Roots)
  │   ├── Extract root candidates from token stem
  │   ├── Look up each candidate in KB-0001 trie
  │   └── Retrieve root type, verb forms, derived nouns
  │
  ├── Step 4+: Pattern matching — KB-0002 (Wazan)
  │   ├── Match stem against pattern signatures
  │   ├── Identify verb form (I–XV) or noun pattern
  │   └── Retrieve phonological template and features
  │
  ├── Step 5+: Conjugation — KB-0003 (Verb Forms) / KB-0004 (Nouns)
  │   ├── If verb: look up conjugation table in KB-0003
  │   ├── If noun: look up noun pattern in KB-0004
  │   └── Generate/verify all 13 inflectional slots
  │
  └── Step 6+: Feature extraction — KB-0007 (Features)
      ├── Pack extracted features into 64-bit bitfield
      ├── Validate against KB-0007 rules
      └── Apply defaults and inferences
```

### 5.3 Module-to-KB Mapping

| Module | KBs Read | KBs Written | Purpose |\n|--------|----------|-------------|---------|\n| **MOD-03** Preprocessor | KB-0005, KB-0006 | — | Clitic stripping, initial token classification\n| **MOD-04** MorphologicalParser | KB-0001, KB-0002, KB-0003, KB-0004, KB-0005, KB-0006, KB-0007 | — | Full morphological analysis\n| **MOD-05** SyntacticParser | KB-0005, KB-0006, KB-0007 | — | Agreement checking, anaphora resolution\n| **MOD-08** KnowledgeGraphResolver | KB-0001, KB-0002, KB-0004 | — | Cross-referencing and resolution\n| **MOD-09** BytecodeGenerator | KB-0003, KB-0004 | — | Inflected form generation\n| **MOD-11** ExplanationEngine | All KBs | — | Pedagogical explanations\n| **KB Compiler** (`agos kb compile`) | — | All KBs | Source → compiled format\n\n---

## 6. Cross-KB Data Model Summary

### 6.1 Shared Vocabulary

All KBs use a shared vocabulary defined across the suite:

| Concept | Defined In | Used By |\n|---------|-----------|--------|\n| Root (جذر) | KB-0001 | KB-0002, KB-0003, KB-0004 |\n| Root type | KB-0001 (§5) + KB-0007 (§7.5) | KB-0002, KB-0003, KB-0004 |\n| Wazan (وزن) | KB-0002 | KB-0003, KB-0004 |\n| Verb form (I–XV) | KB-0002 (§5) + KB-0007 (§7.1) | KB-0003, KB-0004 |\n| Noun type | KB-0004 (§2) + KB-0007 (§7.2) | KB-0002 |\n| Conjugation class | KB-0003 (§3) + KB-0007 (§7.5) | KB-0001 |\n| Part of speech | KB-0007 (§5) | ALL KBs |\n| Feature bitfield (64-bit) | KB-0007 (§10) | RFC-0002, ALL modules |\n| Person/number/gender | KB-0006, KB-0007 (§6) | KB-0003, KB-0006 |\n| Case/state | KB-0004, KB-0007 (§6.7–6.8) | KB-0003 (passive) |\n\n### 6.2 Cross-KB Version Compatibility

```yaml\ncross_kb_compatibility_matrix:\n  KB-0001:\n    KB-0002: \">= 1.0.0\"\n    KB-0003: \">= 1.0.0\"\n    KB-0004: \">= 1.0.0\"\n    KB-0005: \">= 1.0.0\"       # Independent\n    KB-0006: \">= 1.0.0\"       # Independent\n    KB-0007: \">= 1.0.0\"       # root_type, transitivity features\n\n  KB-0002:\n    KB-0001: \">= 1.0.0\"\n    KB-0003: \">= 1.0.0\"\n    KB-0004: \">= 1.0.0\"\n    KB-0005: \">= 1.0.0\"       # Independent\n    KB-0006: \">= 1.0.0\"       # Independent\n    KB-0007: \">= 1.0.0\"       # verb_form, noun_type features\n\n  KB-0003:\n    KB-0001: \">= 1.0.0\"\n    KB-0002: \">= 1.0.0\"\n    KB-0004: \">= 1.0.0\"       # Shared paradigm patterns\n    KB-0005: \">= 1.0.0\"       # Independent\n    KB-0006: \">= 1.0.0\"       # Independent\n    KB-0007: \">= 1.0.0\"       # tense, mood, voice, person, number, gender\n\n  KB-0004:\n    KB-0001: \">= 1.0.0\"\n    KB-0002: \">= 1.0.0\"\n    KB-0003: \">= 1.0.0\"       # Shared paradigm patterns\n    KB-0005: \">= 1.0.0\"       # Independent\n    KB-0006: \">= 1.0.0\"       # Independent\n    KB-0007: \">= 1.0.0\"       # noun_type, case, state, gender\n\n  KB-0005:\n    KB-0001: \">= 1.0.0\"       # Independent\n    KB-0002: \">= 1.0.0\"       # Independent\n    KB-0003: \">= 1.0.0\"       # Independent\n    KB-0004: \">= 1.0.0\"       # Independent\n    KB-0006: \">= 1.0.0\"       # Shared fast-path\n    KB-0007: \">= 1.0.0\"       # Particle-type features\n\n  KB-0006:\n    KB-0001: \">= 1.0.0\"       # Independent\n    KB-0002: \">= 1.0.0\"       # Independent\n    KB-0003: \">= 1.0.0\"       # Independent\n    KB-0004: \">= 1.0.0\"       # Independent\n    KB-0005: \">= 1.0.0\"       # Shared fast-path\n    KB-0007: \">= 1.0.0\"       # Pronoun features\n\n  KB-0007:\n    KB-0001: \">= 1.0.0\"       # root_type, transitivity\n    KB-0002: \">= 1.0.0\"       # verb_form, noun_type\n    KB-0003: \">= 1.0.0\"       # tense, mood, voice, person, number, gender\n    KB-0004: \">= 1.0.0\"       # noun_type, case, state, gender\n    KB-0005: \">= 1.0.0\"       # pos (particle)\n    KB-0006: \">= 1.0.0\"       # pos (pronoun), pronoun_type\n```

---

## 7. RFC Compliance

| RFC | KB Relationship | Status |\n|-----|----------------|--------|\n| **RFC-0001** (Grammar DSL) | Feature names used in DSL expressions must match KB-0007 `feature_name` values | Referenced |\n| **RFC-0002** (Bytecode Format) | 64-bit feature bitfield layout defined by KB-0007 (§10). Bits 0–49 encode 19 features. Bits 50–63 reserved for plugins. | Aligned — KB-0006 extends pronoun_type values 3–7 (requires RFC update) |\n| **RFC-0003** (Grammar VM) | FEATURE_EXTRACT instruction uses KB-0007 feature IDs (u32) | Referenced |\n| **SPEC-0001** (System Spec) | All 7 KBs referenced in module catalog (C2), compilation pipeline (C3), data flow (C5), and performance targets (C9) | Aligned |\n

---

## 8. File Organization

```diff\n  /knowledge/\n  ├── KB-0001/\n  │   ├── metadata.yaml\n  │   ├── ayn/          # Roots by first radical (Arabic alphabet)\n  │   ├── kaf/\n  │   └── ...           # All 28 Arabic letter directories\n  │\n  ├── KB-0002/\n  │   ├── metadata.yaml\n  │   ├── verb-forms/    # Form I–XV YAML files\n  │   ├── noun-patterns/ # Masdar, participle, etc.\n  │   ├── weak-variants/ # Ajwaf, naqis, mithal, etc.\n  │   ├── quadriliteral/ # QI–QIII\n  │   └── classes/       # Pattern class definitions\n  │\n  ├── KB-0003/\n  │   ├── metadata.yaml\n  │   ├── affix-rules/   # Perfect/imperfect/mood affixes\n  │   ├── sound/         # Sound triliteral paradigms\n  │   ├── weak/          # Hollow, defective, assimilated, etc.\n  │   └── quadriliteral/ # QI–QIII paradigms\n  │\n  ├── KB-0004/\n  │   ├── metadata.yaml\n  │   ├── masdars/       # Form I + II–X masdar patterns\n  │   ├── participles/   # Active, passive, resembling\n  │   ├── place-time.yaml\n  │   ├── instrument.yaml\n  │   ├── elative.yaml\n  │   ├── nisbah.yaml\n  │   ├── broken-plurals/\n  │   ├── weak-variants/\n  │   └── other.yaml\n  │\n  ├── KB-0005/\n  │   ├── metadata.yaml\n  │   ├── prepositions.yaml\n  │   ├── conjunctions.yaml\n  │   ├── subjunctive.yaml\n  │   ├── jussive.yaml\n  │   ├── conditional.yaml\n  │   ├── interrogative.yaml\n  │   ├── negative.yaml\n  │   ├── vocative.yaml\n  │   ├── inna-sisters.yaml\n  │   ├── kana-sisters.yaml\n  │   ├── answer-exception.yaml\n  │   ├── masdar-forming.yaml\n  │   └── other.yaml\n  │\n  ├── KB-0006/\n  │   ├── metadata.yaml\n  │   ├── personal-attached.yaml\n  │   ├── personal-detached.yaml\n  │   ├── demonstratives.yaml\n  │   ├── relatives.yaml\n  │   ├── interrogatives.yaml\n  │   └── conditionals.yaml\n  │\n  └── KB-0007/\n      ├── metadata.yaml\n      ├── features/\n      │   ├── pos.yaml\n      │   ├── inflectional/    # gender, number, person, tense, etc.\n      │   ├── derivational/    # verb_form, noun_type, pronoun_type, etc.\n      │   ├── prosodic/        # stress_pattern, syllable_count\n      │   └── orthographic/    # has_shadda, has_madd, has_hamza\n      ├── rules/\n      │   ├── agreement.yaml\n      │   ├── inference.yaml\n      │   └── constraints.yaml\n      └── validation/\n          └── validation-rules.yaml\n```

---

## 9. KB Suite Evolution

### 9.1 Versioning Strategy

All KBs follow **Semantic Versioning 2.0.0** independently. The version compatibility matrix (§6.2) defines which KB versions work together. Key principles:

- **MAJOR**: Breaking schema changes, format changes, or removals. Requires coordinated updates across dependent KBs and a KB compiler update.
- **MINOR**: Additions (new entries, patterns, features). Backward-compatible; compiled KBs remain valid until rebuild.
- **PATCH**: Corrections, improved descriptions, typo fixes. Fully backward-compatible; no recompilation needed if schema unchanged.

### 9.2 Suggested Release Sequence

| Phase | KBs | Version | Milestone |\n|-------|-----|---------|-----------|\n| **1** | KB-0005, KB-0006 | v1.0.0 | Fast-path KBs (smallest, simplest, no dependencies)\n| **2** | KB-0007 | v1.0.0 | Feature taxonomy (cross-cutting, enables validation)\n| **3** | KB-0001, KB-0002 | v1.0.0 | Core chain (roots + patterns — the generative engine)\n| **4** | KB-0003, KB-0004 | v1.0.0 | Paradigm KBs (depend on KB-0001 + KB-0002)\n

---

## 10. Quick Reference

### 10.1 Which KB to Consult

| If you need... | Consult... |\n|----------------|-----------|\n| Root existence, meaning, or verb forms | KB-0001 (§4, §13) |\n| Morphological pattern (wazan) templates | KB-0002 (§5, §6, §8) |\n| Full conjugation tables for a verb form | KB-0003 (§5–§12) |\n| Derived noun patterns (masdar, participle, etc.) | KB-0004 (§5–§16) |\n| Particle meanings, types, or governance rules | KB-0005 (§5–§16) |\n| Pronoun forms, types, or anaphora rules | KB-0006 (§5–§14) |\n| Feature bitfield layout, values, or validation | KB-0007 (§5–§15) |\n| Cross-KB version compatibility | KB-OVERVIEW (§6.2) |\n| Combined size/performance budgets | KB-OVERVIEW (§4) |\n| Pipeline integration details | KB-OVERVIEW (§5) |\n\n### 10.2 Recommended Reading Order

For newcomers to the AGOS KB suite:

```diff
  1. KB-0007: Features    ← Start here (taxonomy & bitfield: the shared vocabulary)
  2. KB-0001: Roots       ← Foundation (consonantal skeleton)
  3. KB-0002: Wazan       ← Mechanism (pattern templates)
  4. KB-0003: Verb Forms  ← Verbs (conjugation paradigms)
  5. KB-0004: Nouns       ← Nouns (derived patterns + broken plurals)
  6. KB-0005: Particles   ← Fast-path (function words)
  7. KB-0006: Pronouns    ← Fast-path (referential words)
```

---

## Progress Summary

| Section | Title | Status |\n|---------|-------|--------|\n| Section 1 | Introduction | ✓ COMPLETE |\n| Section 2 | KB Dependency Architecture | ✓ COMPLETE |\n| Section 3 | Per-KB Summary | ✓ COMPLETE |\n| Section 4 | Combined Resource Budgets | ✓ COMPLETE |\n| Section 5 | Pipeline Integration | ✓ COMPLETE |\n| Section 6 | Cross-KB Data Model Summary | ✓ COMPLETE |\n| Section 7 | RFC Compliance | ✓ COMPLETE |\n| Section 8 | File Organization | ✓ COMPLETE |\n| Section 9 | KB Suite Evolution | ✓ COMPLETE |\n| Section 10 | Quick Reference | ✓ COMPLETE |\n\n**Dependencies:** KB-0001 through KB-0007 (all 7 KBs), SPEC-0001 (Chapters 1–9), RFC-0001, RFC-0002, RFC-0003.\n\n**Recommended next step:** RFC-0003 (Grammar Virtual Machine) — the next major architectural component building on the feature bitfield encoding defined in KB-0007.
