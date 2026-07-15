---
spec_id: SPEC-0001
chapter: 5
title: Data Flow & Intermediate Representations
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0301: Grammar Runtime
  - SPEC-0401: Knowledge Graph Engine
  - SPEC-0501: Explanation Engine
  - RFC-0002: Grammar Bytecode (proposed)
  - RFC-0003: Grammar Virtual Machine (proposed)
  - KB-0007: Morphological Features Taxonomy (planned)
---

# Chapter 5: Data Flow & Intermediate Representations

## Table of Contents

1. [Data Flow Architecture](#1-data-flow-architecture)
2. [IR-1: NormalizedText](#2-ir-1-normalizedtext)
3. [IR-2: TokenStream](#3-ir-2-tokenstream)
4. [IR-3: SegmentedTokenStream](#4-ir-3-segmentedtokenstream)
5. [IR-4: MorphologicalAnalysis](#5-ir-4-morphologicalanalysis)
6. [IR-5: SyntaxTree](#6-ir-5-syntaxtree)
7. [IR-6: GrammarIR (GIR)](#7-ir-6-grammarir-gir)
8. [IR-7: AnnotatedGIR](#8-ir-7-annotatedgir)
9. [IR-8: ResolvedGIR](#9-ir-8-resolvedgir)
10. [IR-9: GrammarBytecode](#10-ir-9-grammarbytecode)
11. [IR-10: AnalysisResult](#11-ir-10-analysisresult)
12. [IR-11: ExplanationOutput](#12-ir-11-explanationoutput)
13. [Evidence Trail Data Model](#13-evidence-trail-data-model)
14. [Ambiguity Forest Data Model](#14-ambiguity-forest-data-model)
15. [Serialization & Wire Formats](#15-serialization--wire-formats)
16. [Data Flow with Caching](#16-data-flow-with-caching)
17. [Cross-References](#17-cross-references)

---

## 1. Data Flow Architecture

### 1.1 Overview

AGOS processes Arabic text through a series of transformations, each producing an **intermediate representation (IR)** that is consumed by the next stage. Data flows strictly forward — no stage reads from or writes to a downstream IR.

```
INPUT: Raw Arabic Text (UTF-8 string)
  │
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-01 │ UnicodeValidator                                │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-1 │ NormalizedText
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-02 │ Lexer                                           │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-2 │ TokenStream
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-03 │ Tokenizer                                       │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-3 │ SegmentedTokenStream
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-04 │ MorphologicalParser                             │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-4 │ MorphologicalAnalysis
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-05 │ SyntaxParser                                    │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-5 │ SyntaxTree
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-06 │ GIRConstructor                                  │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-6 │ GrammarIR (GIR)
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-07 │ RuleEngine                                      │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-7 │ AnnotatedGIR
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-08 │ KnowledgeGraphResolver                          │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-8 │ ResolvedGIR
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-09 │ BytecodeGenerator                               │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-9 │ GrammarBytecode
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  ├─│ MOD-10 │ GVM                                             │
  │ └──────────────────────────────────────────────────────────┘
  │ OUTPUT: IR-10 │ AnalysisResult
  ▼
  │ ┌──────────────────────────────────────────────────────────┐
  └─│ MOD-11 │ ExplanationEngine                               │
    └──────────────────────────────────────────────────────────┘
  OUTPUT: IR-11 │ ExplanationOutput
```

### 1.2 Data Flow Invariants

1. **Monotonic growth:** Information content increases monotonically through the pipeline. Each IR adds new information to the previous IR. No stage discards information from a previous stage.

2. **Forward-only:** A stage MAY read outputs from any previous (upstream) stage. A stage MUST NOT read outputs from any later (downstream) stage. This ensures the pipeline can be parallelized and cached at any boundary.

3. **Deterministic transformation:** Each IR is a deterministic function of the previous IR(s) and the stage's configuration + knowledge dependencies. Given the same inputs, the same IR is always produced.

4. **Serializability:** Every IR MUST be serializable to a byte format (JSON, binary, or both). This enables caching, inspection, debugging, and replay.

5. **Versioned schema:** Every IR type has a version number. IR consumers MUST check version compatibility before processing.

### 1.3 IR Version Table

| IR # | Name | Produced By | Version | Serialization |
|------|------|-------------|---------|---------------|
| 1 | NormalizedText | MOD-01 | 1.0 | JSON / UTF-8 |
| 2 | TokenStream | MOD-02 | 1.0 | JSON |
| 3 | SegmentedTokenStream | MOD-03 | 1.0 | JSON |
| 4 | MorphologicalAnalysis | MOD-04 | 1.0 | JSON |
| 5 | SyntaxTree | MOD-05 | 1.0 | JSON |
| 6 | GrammarIR (GIR) | MOD-06 | 1.0 | JSON / Binary |
| 7 | AnnotatedGIR | MOD-07 | 1.0 | JSON / Binary |
| 8 | ResolvedGIR | MOD-08 | 1.0 | JSON / Binary |
| 9 | GrammarBytecode | MOD-09 | 1.0 | Binary only |
| 10 | AnalysisResult | MOD-10 | 1.0 | JSON / Binary |
| 11 | ExplanationOutput | MOD-11 | 1.0 | JSON / HTML / Text / PDF |

---

## 2. IR-1: NormalizedText

### 2.1 Produced By

**MOD-01:** UnicodeValidator → `validate()`

### 2.2 Consumed By

**MOD-02:** Lexer → `lex()`

### 2.3 Schema

```
IR-1: NormalizedText = {
    spec: "SPEC-0001/IR-1",
    version: "1.0",

    normalized_text: string,               // NFC-normalized, validated UTF-8 Arabic text
    original_text: string,                 // Exact original input (unchanged)

    metadata: {
        char_count: integer,                // Length in Unicode code points (after normalization)
        byte_count: integer,                // Length in UTF-8 bytes (after normalization)
        word_count_estimate: integer,       // Approximate word count (whitespace-delimited)
        has_tashkeel: boolean,
        has_tatweel: boolean,
        has_quranic_symbols: boolean,
        has_non_arabic: boolean,
        normalization_applied: string[],    // e.g., ["nfkc", "strip_tatweel"]
    },

    config_snapshot: {
        normalize_tashkeel: boolean,
        strip_tatweel: boolean,
        strict_arabic_only: boolean,
        max_input_size: integer,
    },
}
```

### 2.4 Data Flow Diagram

```
Raw Arabic String
    │
    ▼
┌─────────────────────────────┐
│  MOD-01 UnicodeValidator    │
│                             │
│  1. Validate UTF-8 encoding │
│  2. Check character ranges  │
│  3. Apply NFKC normal.      │
│  4. Strip tatweel (opt)     │
│  5. Handle tashkeel (opt)   │
│  6. Collect metadata        │
└─────────────────────────────┘
    │
    ▼
IR-1: NormalizedText
    ├── normalized_text     (to MOD-02)
    ├── original_text       (preserved for reference)
    └── metadata            (for diagnostics)
```

### 2.5 Example

```json
{
    "spec": "SPEC-0001/IR-1",
    "version": "1.0",
    "normalized_text": "السَّلَامُ عَلَيْكُمْ وَرَحْمَةُ اللَّهِ وَبَرَكَاتُهُ",
    "original_text": "السَّلَامُ عَلَيْكُمْ وَرَحْمَةُ اللَّهِ وَبَرَكَاتُهُ",
    "metadata": {
        "char_count": 43,
        "byte_count": 129,
        "word_count_estimate": 5,
        "has_tashkeel": true,
        "has_tatweel": false,
        "has_quranic_symbols": false,
        "has_non_arabic": false,
        "normalization_applied": ["nfkc"]
    },
    "config_snapshot": {
        "normalize_tashkeel": false,
        "strip_tatweel": true,
        "strict_arabic_only": false,
        "max_input_size": 1048576
    }
}
```

### 2.6 Transformation Summary

| Aspect | Input | Output |
|--------|-------|--------|
| **Encoding** | Raw UTF-8 (possibly invalid) | Validated UTF-8 |
| **Normalization** | Arbitrary Unicode | NFKC-normalized |
| **Arabic characters** | Original | Preserved |
| **Non-Arabic characters** | Present (if allowed) | Preserved or removed |
| **Tashkeel** | Present (if any) | Preserved or stripped |
| **Tatweel** | Present (if any) | Preserved or stripped |

---

## 3. IR-2: TokenStream

### 3.1 Produced By

**MOD-02:** Lexer → `lex()`

### 3.2 Consumed By

**MOD-03:** Tokenizer → `tokenize()`

### 3.3 Schema

```
IR-2: TokenStream = {
    spec: "SPEC-0001/IR-2",
    version: "1.0",

    text: string,                            // Reference to IR-1.normalized_text

    tokens: [
        {
            id: integer,                     // 0-based sequential ID
            text: string,                    // Token text (substring of `text`)
            token_type: TokenType,
            start_offset: integer,           // Byte offset in `text`
            end_offset: integer,             // Byte offset (exclusive)
        },
        ...
    ],

    metadata: {
        token_count: integer,
        word_count: integer,                 // Count of type="word" tokens
        has_tokens: boolean,
    },
}

TokenType = "word" | "punctuation" | "number" | "whitespace" | "symbol" | "unknown"
```

### 3.4 Data Flow Diagram

```
IR-1: NormalizedText
    │
    ▼
┌─────────────────────────────┐
│  MOD-02 Lexer               │
│                             │
│  1. Scan characters         │
│  2. Classify each char      │
│  3. Group into tokens       │
│  4. Assign IDs & offsets    │
│  5. Collect metadata        │
└─────────────────────────────┘
    │
    ▼
IR-2: TokenStream
    ├── text           (reference to IR-1)
    ├── tokens[]       (ordered list)
    │   ├── id
    │   ├── text
    │   ├── token_type
    │   ├── start_offset
    │   └── end_offset
    └── metadata
```

### 3.5 Example

```json
{
    "spec": "SPEC-0001/IR-2",
    "version": "1.0",
    "text": "السَّلَامُ عَلَيْكُمْ",
    "tokens": [
        {
            "id": 0,
            "text": "السَّلَامُ",
            "token_type": "word",
            "start_offset": 0,
            "end_offset": 16
        },
        {
            "id": 1,
            "text": " ",
            "token_type": "whitespace",
            "start_offset": 16,
            "end_offset": 17
        },
        {
            "id": 2,
            "text": "عَلَيْكُمْ",
            "token_type": "word",
            "start_offset": 17,
            "end_offset": 32
        }
    ],
    "metadata": {
        "token_count": 3,
        "word_count": 2,
        "has_tokens": true
    }
}
```

### 3.6 Transformation Summary

| Aspect | Input (IR-1) | Output (IR-2) |
|--------|-------------|---------------|
| **Representation** | Flat Unicode string | Structured token list |
| **Granularity** | Characters | Words, punctuation, numbers, etc. |
| **Position info** | Not available | Byte offset start/end for each token |
| **Coverage** | n/a | Concatenation of token texts = input text |

---

## 4. IR-3: SegmentedTokenStream

### 4.1 Produced By

**MOD-03:** Tokenizer → `tokenize()`

### 4.2 Consumed By

**MOD-04:** MorphologicalParser → `analyze_morphology()`

### 4.3 Schema

```
IR-3: SegmentedTokenStream = {
    spec: "SPEC-0001/IR-3",
    version: "1.0",

    tokens: [
        {
            raw_token: RawToken,                 // From IR-2

            segmentations: [                     // Ordered by confidence
                {
                    id: string,                  // Unique segmentation ID
                    morphemes: [
                        {
                            text: string,
                            morpheme_type: "prefix" | "stem" | "suffix" | "clitic" | "particle",
                            original_offset: integer,   // Offset within raw_token.text
                            length: integer,
                        },
                        ...
                    ],
                    confidence: float,           // 0.0 to 1.0
                    source: "builtin" | "custom_rule",
                },
                ...
            ],
        },
        ...
    ],

    metadata: {
        total_tokens: integer,
        segmentable_tokens: integer,
        ambiguous_tokens: integer,
        total_ambiguity: float,
    },
}
```

### 4.4 Data Flow Diagram

```
IR-2: TokenStream
    │
    ▼
┌──────────────────────────────────┐
│  MOD-03 Tokenizer                │
│                                  │
│  1. For each "word" token:       │
│   a. Match known proclitics      │
│   b. Match known enclitics       │
│   c. Generate candidate segs     │
│   d. Deduplicate & order         │
│  2. For non-word tokens:         │
│   a. Single segmentation (whole) │
└──────────────────────────────────┘
    │
    ▼
IR-3: SegmentedTokenStream
    ├── tokens[]
    │   ├── raw_token         (from IR-2)
    │   ├── segmentations[]   (ambiguity set)
    │   │   ├── morphemes[]   (prefix → stem → suffix)
    │   │   └── confidence
    │   └── ...
    └── metadata
```

### 4.5 Example

For the token `وَبِالْحَقِّ` (wa-bi-l-ḥaqq):

```json
{
    "tokens": [
        {
            "raw_token": {
                "id": 0,
                "text": "وَبِالْحَقِّ",
                "token_type": "word",
                "start_offset": 0,
                "end_offset": 12
            },
            "segmentations": [
                {
                    "id": "seg-0-0",
                    "morphemes": [
                        { "text": "وَ", "morpheme_type": "prefix", "offset": 0, "length": 2 },
                        { "text": "بِ", "morpheme_type": "prefix", "offset": 2, "length": 2 },
                        { "text": "الْحَقِّ", "morpheme_type": "stem", "offset": 4, "length": 8 }
                    ],
                    "confidence": 0.9,
                    "source": "builtin"
                },
                {
                    "id": "seg-0-1",
                    "morphemes": [
                        { "text": "وَبِالْحَقِّ", "morpheme_type": "stem", "offset": 0, "length": 12 }
                    ],
                    "confidence": 0.3,
                    "source": "builtin"
                }
            ]
        }
    ]
}
```

### 4.6 Transformation Summary

| Aspect | Input (IR-2) | Output (IR-3) |
|--------|-------------|---------------|
| **Word tokens** | Atomic strings | Decomposed into morphemes |
| **Ambiguity** | None | Per-token ambiguity sets |
| **Non-word tokens** | Classified | Single morpheme (type="particle") |
| **Confidence** | Not available | Per-segmentation confidence score |

---

## 5. IR-4: MorphologicalAnalysis

### 5.1 Produced By

**MOD-04:** MorphologicalParser → `analyze_morphology()`

### 5.2 Consumed By

**MOD-05:** SyntaxParser → `parse_syntax()`
**MOD-06:** GIRConstructor → `construct_gir()`

### 5.3 Schema

```
IR-4: MorphologicalAnalysis = {
    spec: "SPEC-0001/IR-4",
    version: "1.0",

    token_analyses: [
        {
            token_id: integer,

            stem_analyses: [                     // One per segmentation
                {
                    analysis_id: string,

                    segmentation_id: string,     // Reference to IR-3 segmentation
                    stem: string,                // The core stem (after clitic removal)

                    root: {                      // From KB-0001
                        text: string,            // e.g., "كتب"
                        source: string,          // KB-0001 entry ID
                        confidence: float,
                    } | null,

                    wazan: {                     // From KB-0002
                        text: string,            // e.g., "فَعَلَ"
                        source: string,
                        form: integer | null,    // I-XV, or null for nouns
                        confidence: float,
                    } | null,

                    pos: PartOfSpeech,

                    features: [                  // Extracted morphological features
                        {
                            name: string,        // e.g., "gender"
                            value: string,       // e.g., "masculine"
                            category: string,    // e.g., "inflectional"
                            confidence: float,
                            source: string,      // KB entry or rule ID
                        },
                        ...
                    ],

                    is_ambiguous: boolean,       // True if alternatives.length > 0
                    alternatives: MorphologicalAnalysis[],  // Recursive

                    evidence: EvidenceEntry[],
                },
                ...
            ],
        },
        ...
    ],

    metadata: {
        total_tokens: integer,
        analyzed_tokens: integer,
        ambiguous_tokens: integer,
        unknown_tokens: integer,
        unknown_stems: string[],
    },
}

PartOfSpeech = "verb" | "noun" | "particle" | "proper_noun" | "pronoun" |
               "adjective" | "adverb" | "preposition" | "conjunction" |
               "interrogative" | "unknown"
```

### 5.4 Morphological Feature Taxonomy (KB-0007 Reference)

Features are categorized by their linguistic domain:

| Category | Features | Example Values |
|----------|----------|----------------|
| **Inflectional** | gender, number, person, tense, mood, voice, case, state | masculine, plural, third, past, indicative, active, accusative, definite |
| **Derivational** | form, transitivity, root_type, noun_type | I, transitive, triliteral_weak, masdar |
| **Prosodic** | stress_pattern, syllable_count | CVCVCV, 3 |
| **Orthographic** | has_shadda, has_madd, has_hamza | true, false |

A complete taxonomy is defined in KB-0007.

### 5.5 Data Flow Diagram

```
IR-3: SegmentedTokenStream
    │
    ▼
┌────────────────────────────────────────────┐
│  MOD-04 MorphologicalParser                │
│                                            │
│  For each stem (from each segmentation):   │
│  1. Fast path: check particles (KB-0005)   │
│  2. Fast path: check pronouns (KB-0006)    │
│  3. Known word lookup (KB-0001-0004)       │
│  4. Root extraction algorithm              │
│  5. Wazan matching                         │
│  6. Feature extraction                     │
│  7. Collect evidence                       │
└────────────────────────────────────────────┘
    │
    ▼
IR-4: MorphologicalAnalysis
    ├── token_analyses[]
    │   ├── token_id
    │   ├── stem_analyses[]
    │   │   ├── root (KB-0001)
    │   │   ├── wazan (KB-0002)
    │   │   ├── pos
    │   │   ├── features[]
    │   │   ├── alternatives[]
    │   │   └── evidence[]
    │   └── ...
    └── metadata
```

### 5.6 Example

For the stem `الْحَقِّ` (al-ḥaqq — "the truth"):

```json
{
    "token_analyses": [
        {
            "token_id": 0,
            "stem_analyses": [
                {
                    "analysis_id": "anl-0-0",
                    "segmentation_id": "seg-0-0",
                    "stem": "الحق",
                    "root": {
                        "text": "ح ق ق",
                        "source": "KB-0001:1423",
                        "confidence": 0.98
                    },
                    "wazan": {
                        "text": "فَعْل",
                        "source": "KB-0002:89",
                        "form": null,
                        "confidence": 0.95
                    },
                    "pos": "noun",
                    "features": [
                        { "name": "gender", "value": "masculine", "category": "inflectional", "confidence": 1.0, "source": "KB-0007" },
                        { "name": "number", "value": "singular", "category": "inflectional", "confidence": 1.0, "source": "KB-0007" },
                        { "name": "state", "value": "definite", "category": "inflectional", "confidence": 1.0, "source": "KB-0007" },
                        { "name": "case", "value": "genitive", "category": "inflectional", "confidence": 0.7, "source": "root_pattern" },
                        { "name": "noun_type", "value": "masdar", "category": "derivational", "confidence": 0.6, "source": "KB-0002:89" }
                    ],
                    "is_ambiguous": false,
                    "alternatives": [],
                    "evidence": []
                }
            ]
        }
    ]
}
```

### 5.7 Transformation Summary

| Aspect | Input (IR-3) | Output (IR-4) |
|--------|-------------|---------------|
| **Morphemes** | Surface form only | Analyzed: root, pattern, POS |
| **Knowledge** | None | Linked to KB-0001–0007 |
| **Features** | None | Morphological feature set |
| **Ambiguity** | Segmentation alternatives | Morphological alternatives |
| **Evidence** | None | Per-analysis evidence trail |

---

## 6. IR-5: SyntaxTree

### 6.1 Produced By

**MOD-05:** SyntaxParser → `parse_syntax()`

### 6.2 Consumed By

**MOD-06:** GIRConstructor → `construct_gir()`

### 6.3 Schema

```
IR-5: SyntaxTree = {
    spec: "SPEC-0001/IR-5",
    version: "1.0",

    trees: [
        {
            id: string,                          // Unique tree ID
            tree_type: SentenceType,
            root: Constituent,
            confidence: float,
            source: string,                      // Grammar school
        },
        ...
    ],

    metadata: {
        sentence_count: integer,
        tokens_parsed: integer,
        ambiguity_count: integer,
        parse_time_ms: float,
    },
}

SentenceType = "jumlah_ismiyyah" | "jumlah_fi'liyyah" | "jumlah_shartiyyah" |
               "jumlah_zarfiyyah" | "phrase" | "incomplete" | "unknown"

Constituent = {
    node_type: "word" | "phrase" | "clause",
    role: SyntacticRole,
    token_ids: integer[],                        // Token IDs from IR-2/IR-3
    children: Constituent[],
    features: {
        [feature_name: string]: string,           // e.g., "case": "nominative"
    },
    implicit: boolean,                            // True if this constituent is implied (hadhf)
}

SyntacticRole = "mubtada" | "khabar" | "fi'l" | "fa'il" |
                "maf'ul_bi-hi" | "maf'ul_mutlaq" | "maf'ul_fih" |
                "maf'ul_lahu" | "maf'ul_ma'ahu" |
                "hal" | "tamyiz" | "na'at" | "idafa" |
                "mudaf" | "mudaf_ilayh" |
                "harf_jarr" | "majrur" | "harf_nasb" | "harf_jazm" |
                "zarf" | "qayd" | "ta'kid" | "badal" | "atasf" |
                "istithna" | "nida" | "jawab" | "shart" | "jaza" |
                "sila" | "rabit" | "unknown"
```

### 6.4 Data Flow Diagram

```
IR-4: MorphologicalAnalysis
    │
    ▼
┌────────────────────────────────────────────┐
│  MOD-05 SyntaxParser                       │
│                                            │
│  1. Sentence segmentation                  │
│  2. Identify sentence type                 │
│  3. Parse verbal sentence (fi'l → fa'il)   │
│  4. Parse nominal sentence (mubtada' →     │
│     khabar)                                │
│  5. Identify constructions (idafa, wasf,   │
│     tawkid, badal)                         │
│  6. Handle ambiguity × morphology          │
│  7. Partial parse on failure               │
└────────────────────────────────────────────┘
    │
    ▼
IR-5: SyntaxTree
    ├── trees[]
    │   ├── tree_type
    │   ├── root (recursive Constituent)
    │   │   ├── node_type
    │   │   ├── role
    │   │   ├── token_ids
    │   │   ├── children[]
    │   │   └── features
    │   ├── confidence
    │   └── source (school)
    └── metadata
```

### 6.5 Example

For the sentence `كَتَبَ مُحَمَّدٌ رِسَالَةً` (Muhammad wrote a letter):

```json
{
    "spec": "SPEC-0001/IR-5",
    "version": "1.0",
    "trees": [
        {
            "id": "tree-0",
            "tree_type": "jumlah_fi'liyyah",
            "root": {
                "node_type": "clause",
                "role": "jumlah_fi'liyyah",
                "token_ids": [0, 1, 2],
                "children": [
                    {
                        "node_type": "word",
                        "role": "fi'l",
                        "token_ids": [0],
                        "children": [],
                        "features": {
                            "tense": "past",
                            "person": "third",
                            "gender": "masculine",
                            "number": "singular",
                            "voice": "active"
                        },
                        "implicit": false
                    },
                    {
                        "node_type": "word",
                        "role": "fa'il",
                        "token_ids": [1],
                        "children": [],
                        "features": {
                            "case": "nominative"
                        },
                        "implicit": false
                    },
                    {
                        "node_type": "word",
                        "role": "maf'ul_bi-hi",
                        "token_ids": [2],
                        "children": [],
                        "features": {
                            "case": "accusative"
                        },
                        "implicit": false
                    }
                ],
                "features": {},
                "implicit": false
            },
            "confidence": 0.95,
            "source": "basra"
        }
    ],
    "metadata": {
        "sentence_count": 1,
        "tokens_parsed": 3,
        "ambiguity_count": 0,
        "parse_time_ms": 1.2
    }
}
```

### 6.6 Transformation Summary

| Aspect | Input (IR-4) | Output (IR-5) |
|--------|-------------|---------------|
| **Per-token** | Morphological features | Plus syntactic role assignment |
| **Structure** | Flat list of token analyses | Hierarchical tree with constituents |
| **Relationships** | None between tokens | Subject-verb, modifier-modified, etc. |
| **Ambiguity** | Per-token morphological | Per-sentence parse trees |
| **Implicit elements** | Not present | Marked with `implicit: true` |

---

## 7. IR-6: GrammarIR (GIR)

### 7.1 Produced By

**MOD-06:** GIRConstructor → `construct_gir()`

### 7.2 Consumed By

**MOD-07:** RuleEngine → `apply_rules()`

### 7.3 Schema

```
IR-6: GrammarIR = {
    spec: "SPEC-0001/IR-6",
    version: "1.0",

    metadata: {
        created_at: string,                        // ISO 8601
        pipeline_version: string,                  // AGOS platform version
        knowledge_versions: {
            [kb_id: string]: string,               // e.g., "KB-0001": "1.2.3"
        },
        school: string,                            // Grammar school
    },

    text: string,                                  // Original input text (from IR-1)

    tokens: [
        {
            index: integer,                         // 0-based token index
            original_text: string,                  // Token text from original input
            normalized_text: string,                // Token text from normalized input
            start_offset: integer,                  // Byte offset in original text
            end_offset: integer,

            clitics: {
                prefixes: string[],                 // Separated prefix texts
                suffixes: string[],                 // Separated suffix texts
            },

            morphology: MorphologicalAnalysis | MorphologicalAnalysis[],
                                                    // Single or ambiguous
        },
        ...
    ],

    trees: [                                        // Ambiguity forest
        {
            id: string,
            sentence_type: string,
            root: GIRConstituent,
            confidence: float,
            source: string,
        },
        ...
    ],

    evidence: EvidenceEntry[],                      // All evidence from MOD-03 through MOD-06
}

GIRConstituent = {
    node_type: "token" | "phrase" | "clause",
    role: SyntacticRole,
    token_indices: integer[],
    children: GIRConstituent[],
    features: { [key: string]: string },
    confidence: float,
}
```

### 7.4 Data Flow Diagram

```
IR-4: MorphologicalAnalysis    IR-5: SyntaxTree
    │                               │
    └───────────────┬───────────────┘
                    │
                    ▼
┌──────────────────────────────────────────┐
│  MOD-06 GIRConstructor                   │
│                                          │
│  1. Align tokens from IR-4 and IR-5      │
│  2. Build token-morphology mapping       │
│  3. Build token-syntax mapping           │
│  4. Create ambiguity forest              │
│     (morphology × syntax combinations)   │
│  5. Prune invalid combinations           │
│  6. Collect evidence from all stages     │
└──────────────────────────────────────────┘
                    │
                    ▼
IR-6: GrammarIR (GIR)
    ├── metadata
    ├── text
    ├── tokens[]        (morphology + syntax merged)
    │   ├── index, text, offsets
    │   ├── clitics
    │   ├── morphology  (from IR-4)
    │   └── ...
    ├── trees[]         (from IR-5, enriched)
    │   ├── id, type
    │   ├── root (constituent tree)
    │   ├── confidence
    │   └── source
    └── evidence[]
```

### 7.5 Transformation Summary

| Aspect | Input (IR-4 + IR-5) | Output (IR-6) |
|--------|---------------------|---------------|
| **Unification** | Separate morphology + syntax | Single unified representation |
| **Token alignment** | Separate token spaces | Single token list with both analyses |
| **Ambiguity** | Independent per-stage | Combined: morphology × syntax combinations |
| **Evidence** | Per-stage evidence | Consolidated evidence trail |

---

## 8. IR-7: AnnotatedGIR

### 8.1 Produced By

**MOD-07:** RuleEngine → `apply_rules()`

### 8.2 Consumed By

**MOD-08:** KnowledgeGraphResolver → `resolve()`

### 8.3 Schema

```
IR-7: AnnotatedGIR = {
    // Inherits all fields from IR-6
    ...IR-6,

    // Additional fields
    rule_applications: [
        {
            rule_id: string,
            rule_name: string,
            school: string,
            version: string,

            applies_to: {
                token_indices: integer[],
                constituent_path: string[],         // Path in syntax tree
            },

            condition: string,                      // Human-readable condition
            action: string,                         // Human-readable action

            result: {
                confirmed: string[],                 // Analysis IDs that were confirmed
                rejected: string[],                  // Analysis IDs that were rejected
                modified: [
                    {
                        feature: string,
                        from: string,
                        to: string,
                    },
                    ...
                ],
                flag: GrammaticalFlag | null,
            },

            evidence: EvidenceEntry,
        },
        ...
    ],

    flags: [
        {
            flag_type: "error" | "warning" | "info",
            code: string,                            // e.g., "SUBJECT_VERB_AGREEMENT"
            message: string,                         // Human-readable
            token_indices: integer[],
            rule_id: string,
        },
        ...
    ],

    rule_set_version: string,
    school: string,
}
```

### 8.4 Data Flow Diagram

```
IR-6: GrammarIR
    │
    ▼
┌──────────────────────────────────────────┐
│  MOD-07 RuleEngine                       │
│                                          │
│  1. Load rule set for school + version   │
│  2. For each rule (priority order):      │
│   a. Evaluate condition against GIR      │
│   b. If match → apply action             │
│      (confirm / reject / modify / flag)  │
│   c. Record rule application             │
│  3. Detect conflicts                     │
│  4. Sort remaining alternatives          │
│  5. Detect circular rule fixpoint        │
└──────────────────────────────────────────┘
    │
    ▼
IR-7: AnnotatedGIR
    ├── ... (all IR-6 fields)
    ├── rule_applications[]
    │   ├── rule_id, rule_name, school
    │   ├── condition, action
    │   ├── result (confirmed / rejected / modified)
    │   └── evidence
    ├── flags[]
    │   ├── type (error / warning / info)
    │   ├── code, message
    │   └── token_indices
    ├── rule_set_version
    └── school
```

### 8.5 Transformation Summary

| Aspect | Input (IR-6) | Output (IR-7) |
|--------|-------------|---------------|
| **Ambiguity** | Full forest (all alternatives) | Pruned: confirmed/rejected/ modified |
| **Features** | Initial assignment | Updated per school-specific rules |
| **Evidence** | Stage evidence | Plus rule application evidence |
| **Flags** | None | Grammatical flags (errors, warnings) |
| **Rule trace** | None | Complete rule application history |

---

## 9. IR-8: ResolvedGIR

### 9.1 Produced By

**MOD-08:** KnowledgeGraphResolver → `resolve()`

### 9.2 Consumed By

**MOD-09:** BytecodeGenerator → `generate_bytecode()`

### 9.3 Schema

```
IR-8: ResolvedGIR = {
    // Inherits all fields from IR-7
    ...IR-7,

    // Override tokens with resolved data
    tokens: [
        {
            // Inherits all GIRToken fields
            ...GIRToken,

            // Additional fields
            root_entry: {
                id: string,                          // KB-0001 entry ID
                root: string,
                meaning: string,
                forms: string[],                     // Verb forms I-XV
                derived_nouns: string[],
                cognates: string[],
                semantic_field: string,
                cross_references: {
                    related_roots: string[],
                    antonyms: string[],
                    synonyms: string[],
                },
            } | null,

            wazan_entry: {
                id: string,
                pattern: string,
                meaning: string,
                form: integer | null,
                example: string,
            } | null,

            dictionary_entry: {
                id: string,
                word: string,
                definition: string,
                translations: {
                    [language: string]: string,
                },
                usage_examples: string[],
            } | null,

            semantic_tags: string[],                 // e.g., ["human", "action"]
        },
        ...
    ],

    knowledge_versions: {
        [kb_id: string]: string,
    },

    resolution_stats: {
        roots_resolved: integer,
        patterns_resolved: integer,
        unresolved_references: integer,
        resolution_time_ms: float,
    },
}
```

### 9.4 Data Flow Diagram

```
IR-7: AnnotatedGIR       KB-0001..0007
    │                         │
    └──────────┬──────────────┘
               │
               ▼
┌──────────────────────────────────────────┐
│  MOD-08 KnowledgeGraphResolver           │
│                                          │
│  For each token with a root reference:   │
│  1. Look up root in KB-0001             │
│  2. Attach root_entry (definition, etc.)│
│  3. Look up wazan in KB-0002            │
│  4. Look up dictionary entry (optional) │
│  5. Look up semantic tags               │
│  6. Count resolved / unresolved          │
└──────────────────────────────────────────┘
               │
               ▼
IR-8: ResolvedGIR
    ├── ... (all IR-7 fields)
    ├── tokens[]
    │   └── (enriched with root_entry,
    │         wazan_entry,
    │         dictionary_entry,
    │         semantic_tags)
    ├── knowledge_versions
    └── resolution_stats
```

### 9.5 Transformation Summary

| Aspect | Input (IR-7) | Output (IR-8) |
|--------|-------------|---------------|
| **Root references** | Abstract (root text only) | Resolved (full KB entry) |
| **Wazan references** | Abstract (pattern text only) | Resolved (full pattern description) |
| **Dictionary** | Not available | Optional word definitions |
| **Semantic tags** | Not available | Semantic categorization |
| **Unresolved refs** | Not tracked | Explicitly counted in stats |

---

## 10. IR-9: GrammarBytecode

### 10.1 Produced By

**MOD-09:** BytecodeGenerator → `generate_bytecode()`

### 10.2 Consumed By

**MOD-10:** GVM → `execute()`

### 10.3 Schema

```
IR-9: GrammarBytecode = {
    spec: "SPEC-0001/IR-9",
    version: "1.0",

    raw: Uint8Array,                              // Complete serialized bytecode

    sections: [                                    // Logical sections (for debugging)
        {
            section_type: SectionType,
            data: Uint8Array,
            offset: integer,                       // Byte offset in raw
            size: integer,                         // Byte size
        },
        ...
    ],

    size: integer,                                 // Total bytecode size in bytes

    metadata: {
        input_text_hash: string,                   // SHA-256 of original text
        token_count: integer,
        tree_count: integer,
        rule_count: integer,
        compression_ratio: float,                  // bytecode_size / gir_json_size
        gir_json_size_bytes: integer,
    },
}

SectionType = "header" | "metadata" | "tokens" | "morphology" |
              "syntax" | "rules" | "evidence" | "strings" | "end"
```

### 10.4 Binary Layout

The bytecode binary layout is as follows (detailed specification in RFC-0002):

```
Offset  Size    Field
──────  ────    ─────
0       4       Magic: "AGOS" (0x41474F53)
4       2       Version major
6       2       Version minor
8       2       Version patch
10      2       Flags (bitmask)
12      4       Total bytecode size
16      4       Section count
20      N       Section table:
                ┌────────────┬───────┬──────┐
                │ section_id │ offset│ size │
                ├────────────┼───────┼──────┤
                │ 1          │   X   │   Y  │  → metadata section
                │ 2          │   X+Y │   Z  │  → tokens section
                │ ...        │  ...  │  ... │
                └────────────┴───────┴──────┘
...     ...     Section data (variable):
                │ METADATA: school, KB versions, timestamp
                │ TOKENS: token data as compact bitfields
                │ MORPHOLOGY: root IDs, wazan IDs, feature bitfields
                │ SYNTAX: tree structures, roles
                │ RULES: rule applications
                │ EVIDENCE: evidence trail entries
                │ STRINGS: string table (UTF-8)
...     4       End marker (0x454E4444 = "ENDD")
```

### 10.5 Data Flow Diagram

```
IR-8: ResolvedGIR
    │
    ▼
┌──────────────────────────────────────────┐
│  MOD-09 BytecodeGenerator               │
│                                          │
│  1. Emit header (magic, version, flags)  │
│  2. Emit metadata section                │
│  3. Emit tokens section (compact)        │
│  4. Emit morphology section              │
│  5. Emit syntax section                  │
│  6. Emit rules section                   │
│  7. Emit evidence section                │
│  8. Emit string table                    │
│  9. Apply optimizations (opt level)      │
│  10. Finalize (fill size, end marker)    │
└──────────────────────────────────────────┘
    │
    ▼
IR-9: GrammarBytecode
    ├── raw (Uint8Array)
    ├── sections[]
    ├── size
    └── metadata
```

### 10.6 Transformation Summary

| Aspect | Input (IR-8) | Output (IR-9) |
|--------|-------------|---------------|
| **Format** | Human-readable (JSON) | Machine-readable (binary) |
| **Size** | Large (verbose) | Compact (~20% of JSON) |
| **Execution** | Cannot be executed | Executable by GVM |
| **Self-contained** | References KBs | Embeds all analysis data |
| **Portability** | Platform-independent | Platform-independent + versioned |

---

## 11. IR-10: AnalysisResult

### 11.1 Produced By

**MOD-10:** GVM → `execute()`

### 11.2 Consumed By

**MOD-11:** ExplanationEngine → `explain()`

### 11.3 Schema

```
IR-10: AnalysisResult = {
    spec: "SPEC-0001/IR-10",
    version: "1.0",

    metadata: {
        executed_at: string,                         // ISO 8601
        execution_time_ms: float,
        steps_executed: integer,                     // GVM instruction steps
        memory_used: integer,                        // Bytes used during execution
        bytecode_size: integer,                      // Input bytecode size
    },

    input_text: string,                              // Original input text
    input_text_hash: string,                         // SHA-256 of input text

    trees: [                                         // One tree per successful parse
        {
            id: string,
            type: string,
            tokens: [
                {
                    index: integer,
                    text: string,
                    features: {
                        morphological: {
                            root: string | null,
                            wazan: string | null,
                            pos: string,
                            gender: string | null,
                            number: string | null,
                            person: string | null,
                            tense: string | null,
                            mood: string | null,
                            voice: string | null,
                            case: string | null,
                            state: string | null,
                        },
                        syntactic: {
                            role: string | null,
                            governor: integer | null,     // Token index of governing word
                        },
                        semantic: {
                            tags: string[],
                            definition: string | null,
                            root_meaning: string | null,
                        },
                    },
                    evidence: EvidenceEntry[],
                },
                ...
            ],
            constituents: GIRConstituent[],
            flags: GrammaticalFlag[],
            confidence: float,
        },
        ...
    ],

    flags: GrammaticalFlag[],

    evidence: EvidenceEntry[],                       // Complete evidence trail
}
```

### 11.4 Data Flow Diagram

```
IR-9: GrammarBytecode
    │
    ▼
┌──────────────────────────────────────────┐
│  MOD-10 GrammarVirtualMachine            │
│                                          │
│  1. Verify bytecode version              │
│  2. Verify bytecode integrity            │
│  3. Allocate execution context           │
│  4. Interpret or JIT instructions        │
│  5. Enforce step limit                   │
│  6. Produce AnalysisResult               │
└──────────────────────────────────────────┘
    │
    ▼
IR-10: AnalysisResult
    ├── metadata (timing, steps, memory)
    ├── input_text, input_text_hash
    ├── trees[]
    │   ├── tokens[] (with combined features)
    │   │   ├── morphological features
    │   │   ├── syntactic features
    │   │   ├── semantic features
    │   │   └── evidence
    │   ├── constituents
    │   ├── flags
    │   └── confidence
    ├── flags[]
    └── evidence[]
```

### 11.5 Transformation Summary

| Aspect | Input (IR-9) | Output (IR-10) |
|--------|-------------|---------------|
| **Format** | Binary bytecode | Structured analysis object |
| **Enrichment** | Compressed data | Fully expanded feature set |
| **Execution** | Potential energy | Kinetic: executed result |
| **Features** | Encoded bitfields | Organized: morphological / syntactic / semantic |

---

## 12. IR-11: ExplanationOutput

### 12.1 Produced By

**MOD-11:** ExplanationEngine → `explain()`

### 12.2 Consumed By

**MOD-14:** APIGateway → `analyze()` (returned to caller)

### 12.3 Schema

```
IR-11: ExplanationOutput = {
    spec: "SPEC-0001/IR-11",
    version: "1.0",

    metadata: {
        generated_at: string,
        language: string,
        format: string,
        llm_enhanced: boolean,
        generation_time_ms: float,
        pipeline_timing_ms: {                      // Per-stage timing
            total: float,
            validation: float,
            lexing: float,
            tokenization: float,
            morphology: float,
            syntax: float,
            gir_construction: float,
            rule_engine: float,
            kg_resolution: float,
            bytecode_generation: float,
            gvm_execution: float,
            explanation: float,
        },
    },

    input_text: string,

    overview: string,                               // Summary of the grammatical analysis

    sentence_type: string | null,

    irab_breakdown: [
        {
            token: string,
            root: string | null,
            pos: string,
            features: [
                {
                    name: string,
                    value: string,
                },
                ...
            ],
            syntactic_role: string | null,
            explanation: string,                    // Localized natural language
        },
        ...
    ],

    constructions: [                                // Notable grammatical constructions
        {
            name: string,                           // e.g., "Idafa (Construct State)"
            description: string,
            tokens: integer[],
        },
        ...
    ],

    flags: [                                        // Grammatical flags
        {
            flag_type: string,
            message: string,
            tokens: integer[],
        },
        ...
    ],

    evidence: EvidenceEntry[],                      // If include_evidence == true

    raw: string,                                    // Formatted according to 'format',
}                                                   //   e.g., HTML string, plain text
```

### 12.4 Data Flow Diagram

```
IR-10: AnalysisResult
    │
    ▼
┌──────────────────────────────────────────┐
│  MOD-11 ExplanationEngine                │
│                                          │
│  1. Select language template             │
│  2. Generate I'rab breakdown             │
│  3. Identify notable constructions       │
│  4. Generate overview summary            │
│  5. Format output (text/html/json/pdf)   │
│  6. LLM enhancement (optional)           │
│     (NEVER modifies analysis)            │
└──────────────────────────────────────────┘
    │
    ▼
IR-11: ExplanationOutput
    ├── metadata
    ├── input_text
    ├── overview
    ├── sentence_type
    ├── irab_breakdown[]
    │   ├── token, root, pos
    │   ├── features[]
    │   ├── syntactic_role
    │   └── explanation (localized)
    ├── constructions[]
    ├── flags[]
    ├── evidence[]
    └── raw (formatted output)
```

### 12.5 Transformation Summary

| Aspect | Input (IR-10) | Output (IR-11) |
|--------|-------------|---------------|
| **Audience** | Machine (structured data) | Human (natural language) |
| **Format** | JSON/Binary | Text / HTML / JSON / PDF |
| **Language** | English identifiers | Localized (ar, en, ur, etc.) |
| **Detail** | Complete feature set | Selective, educational presentation |
| **LLM enhancement** | Optional | Additive, analysis-preserving |

---

## 13. Evidence Trail Data Model

### 13.1 Purpose

The evidence trail is the **complete, immutable record** of every decision made during the analysis of a text. It is the mechanism that satisfies Core Principles 3 (Explainability by Design) and 12 (Evidence Trail Completeness).

### 13.2 Schema

```
EvidenceTrail = {
    spec: "SPEC-0001/evidence",
    version: "1.0",

    pipeline: {
        started_at: string,
        completed_at: string,
        pipeline_version: string,
        school: string,
        knowledge_versions: KnowledgeVersionMap,
    },

    entries: EvidenceEntry[],

    summary: {
        total_entries: integer,
        stages_involved: string[],                    // e.g., ["MOD-03", "MOD-04", ...]
        rules_applied: integer,                       // Count of rule applications
        ambiguities_resolved: integer,
        flags_raised: integer,
    },
}

EvidenceEntry = {
    id: string,                                        // Unique entry ID
    timestamp: string,                                 // ISO 8601
    stage: string,                                     // e.g., "MOD-04"
    stage_iteration: integer,                          // If stage ran multiple passes

    category: "segmentation" | "morphology" | "syntax" |
              "rule_application" | "knowledge_resolution" | "bytecode" | "execution",

    rule_or_algorithm: string,                         // Rule ID or algorithm name
    version: string,                                   // Version of rule/algorithm

    input: {
        description: string,                           // Human-readable input summary
        state_hash: string,                            // Hash of input state
    },

    output: {
        description: string,                           // Human-readable output summary
        delta: string,                                 // What changed (e.g., "confirmed analysis 2")
    },

    confidence: float,                                 // 0.0 to 1.0
    token_indices: integer[],                          // Affected tokens
}
```

### 13.3 Accumulation Flow

The evidence trail is built incrementally across the pipeline:

```
Stage         │ Evidence Added
──────────────┼───────────────────────────────────────────
MOD-03        │ Each segmentation decision
MOD-04        │ Each root extraction, pattern match, feature assignment
MOD-05        │ Each syntactic role assignment, parse tree construction
MOD-06        │ Each token alignment, ambiguity combination
MOD-07        │ Each rule application (confirm, reject, modify, flag)
MOD-08        │ Each KB lookup result
MOD-09        │ Each bytecode encoding decision
MOD-10        │ Each GVM execution step (summary level)
```

### 13.4 Pathological Case: Full Evidence for a 10-Word Sentence

A typical 10-word sentence with moderate ambiguity generates approximately:

- **MOD-03:** 15–30 segmentation entries (1.5–3× ambiguity)
- **MOD-04:** 20–50 morphological entries (2–5 analyses per word)
- **MOD-05:** 5–10 syntax entries (1–2 parse trees)
- **MOD-07:** 50–200 rule application entries
- **MOD-08:** 10–20 KB resolution entries

**Total:** ~100–310 evidence entries per sentence. For educational applications, this level of detail enables:
- Step-by-step explanation of grammatical reasoning
- Debugging incorrect analyses
- Audit trails for research use
- Comparative analysis across grammar schools

---

## 14. Ambiguity Forest Data Model

### 14.1 Purpose

The ambiguity forest represents the **space of all valid grammatical analyses** for a given input. Rather than committing to a single analysis early, AGOS preserves ambiguity as a first-class concept throughout the pipeline.

### 14.2 Structure

```
AmbiguityForest = {
    version: "1.0",

    tokens: [
        {
            index: integer,

            // Per-token ambiguity
            morphological_alternatives: [
                {
                    analysis_id: string,
                    probability: float,               // 0.0 to 1.0
                    features: { ... },
                },
                ...
            ],
        },
        ...
    ],

    // Cross-token combinations
    parse_paths: [                                    // Ordered by confidence
        {
            path_id: string,
            probabilities: {                           // Per-token analysis selection
                [token_index: integer]: string,        // analysis_id
            },
            tree: {
                id: string,
                tree_type: string,
                root: GIRConstituent,
            },
            confidence: float,                         // Aggregate across all tokens
            source: string,                            // Grammar school
        },
        ...
    ],
}
```

### 14.3 Path Counting

The total number of possible parse paths is the product of all token-level ambiguities:

```
total_paths = ∏(token.morphological_alternatives.length)
               × trees.length
```

For a 5-word sentence where each word has 2 morphological alternatives and there are 2 parse trees: **2⁵ × 2 = 64 paths**.

### 14.4 Pruning by Rules

The Rule Engine prunes the ambiguity forest by rejecting paths that violate grammatical rules:

```
Before MOD-07:    64 paths (full forest)
After  MOD-07:    2–8 paths (rule-pruned forest)
After  MOD-08:    2–8 paths (knowledge enriches, doesn't prune)
After  MOD-10:    1–3 paths (final execution order)
```

### 14.5 Path Selection for Final Output

The final `AnalysisResult` contains all surviving parse paths, ordered by confidence. The `ExplanationEngine` may:
- Present the highest-confidence path as the primary analysis.
- List alternative paths with explanations of why they differ.
- Highlight unresolved ambiguities for educational purposes.

---

## 15. Serialization & Wire Formats

### 15.1 Serialization Matrix

| IR | JSON | Binary (CBOR) | Custom Binary | ProtoBuf |
|----|------|---------------|---------------|----------|
| IR-1 | ✓ | — | — | RECOMMENDED |
| IR-2 | ✓ | — | — | RECOMMENDED |
| IR-3 | ✓ | — | — | RECOMMENDED |
| IR-4 | ✓ | OPTIONAL | — | RECOMMENDED |
| IR-5 | ✓ | OPTIONAL | — | RECOMMENDED |
| IR-6 | ✓ | OPTIONAL | — | RECOMMENDED |
| IR-7 | ✓ | OPTIONAL | — | RECOMMENDED |
| IR-8 | ✓ | OPTIONAL | — | RECOMMENDED |
| IR-9 | — | — | ✓ (RFC-0002) | — |
| IR-10 | ✓ | OPTIONAL | — | RECOMMENDED |
| IR-11 | ✓ | — | — | OPTIONAL |

### 15.2 JSON Serialization Rules

1. Field names use `snake_case`.
2. All timestamps use ISO 8601 with timezone (RFC 3339).
3. All hashes use lowercase hex-encoded SHA-256.
4. `null` values are omitted from output (not included as `"field": null`).
5. Empty arrays are included as `"field": []`.
6. Numbers that represent byte sizes or offsets are integers.
7. Numbers that represent confidence or time are floats with up to 6 decimal places.

### 15.3 Binary Serialization (IR-9)

Grammar Bytecode (IR-9) uses a custom binary format specified in RFC-0002. Key properties:

| Property | Value |
|----------|-------|
| **Encoding** | Little-endian |
| **Integers** | Fixed-size (u16, u32, u64) and varint for compactness |
| **Strings** | UTF-8 with length prefix (varint) |
| **Feature bitfields** | Custom bit packing per KB-0007 taxonomy |
| **Checksum** | CRC32C at end of each section |
| **Magic bytes** | 0x41474F53 ("AGOS") at start |

---

## 16. Data Flow with Caching

### 16.1 Cache Points

The CacheManager (MOD-13) can cache at any pipeline boundary. Each cache point uses a composite key:

```
Cache Key = hash(
    stage_input_hash +
    stage_config_hash +
    knowledge_version_hash
)
```

| Cache Point | Caches Output Of | Typical Cache Hit Time | Typical Cache Miss Time |
|-------------|------------------|----------------------|------------------------|
| After MOD-01 | IR-1 | < 1 μs | ~10 μs |
| After MOD-02 | IR-2 | < 1 μs | ~5 μs |
| After MOD-03 | IR-3 | < 1 μs | ~20 μs |
| After MOD-04 | IR-4 | < 1 μs | ~500 μs |
| After MOD-05 | IR-5 | < 1 μs | ~1 ms |
| After MOD-06 | IR-6 | < 1 μs | ~100 μs |
| After MOD-07 | IR-7 | < 1 μs | ~500 μs |
| After MOD-08 | IR-8 | < 1 μs | ~100 μs |
| After MOD-09 | IR-9 | < 1 μs | ~200 μs |
| After MOD-10 | IR-10 | < 1 μs | ~1 ms |
| After MOD-11 | IR-11 | < 1 μs | ~5 ms |

### 16.2 Cache Invalidation Flow

```
Knowledge Base Update
    │
    ▼
CacheManager.invalidate({ knowledge_base_id: "KB-0001" })
    │
    ▼
All cache entries that used KB-0001 v1.2.3 are invalidated
    │
    ├── MOD-04 output caches (morphology depends on KB-0001)
    ├── MOD-05 output caches (syntax depends on morphology)
    ├── MOD-06 output caches (GIR depends on syntax)
    ├── MOD-07 output caches (rules depend on GIR)
    ├── MOD-08 output caches (KG depends on KB-0001)
    ├── MOD-09 output caches (bytecode depends on KG)
    ├── MOD-10 output caches (GVM depends on bytecode)
    └── MOD-11 output caches (explanation depends on GVM)
```

### 16.3 Cache-Aware Pipeline

```
function analyze_with_cache(request):
    key = build_cache_key(request)

    // Check if the final result is cached
    cached = cache.get(key)
    if cached is hit:
        return cached.value

    // Check partial cache (e.g., morphology-only)
    morphology_key = build_cache_key_partial(request, "MOD-04")
    cached_morphology = cache.get(morphology_key)

    if cached_morphology is hit:
        // Resume pipeline from MOD-05
        return execute_from(MOD-05, cached_morphology.value)
    else:
        // Full pipeline execution
        return execute_full_pipeline(request)
```

---

## 17. Cross-References

### 17.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | Module boundaries and pipeline structure |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | Generation algorithms for each IR |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | Formal interface definitions for IR producers/consumers |
| SPEC-0101 | Morphology Engine | Detailed specification for IR-4 and IR-5 generation |
| SPEC-0201 | Rule Engine | Detailed specification for IR-7 generation |
| SPEC-0301 | Grammar Runtime | Detailed specification for IR-10 generation |
| SPEC-0401 | Knowledge Graph Engine | Detailed specification for IR-8 generation |
| SPEC-0501 | Explanation Engine | Detailed specification for IR-11 generation |
| RFC-0002 | Grammar Bytecode | Binary layout for IR-9 |
| RFC-0003 | Grammar Virtual Machine | Execution model that consumes IR-9 and produces IR-10 |

### 17.2 External References

| Reference | Relevance |
|-----------|-----------|
| JSON Schema | Schema validation for IR-1 through IR-8, IR-10, IR-11 |
| CBOR (RFC 7049) | Compact binary JSON alternative for IR serialization |
| Protocol Buffers | Efficient schema-based serialization |
| Unicode Standard | Character encoding throughout all IRs |
| RFC 3339 (Date/Time) | Timestamp format for all IRs |

---

## Progress Summary

**SPEC-0001: Platform Architecture**

| Chapter | Title | Status |
|---------|-------|--------|
| Chapter 1 | Introduction and Scope | ✓ COMPLETE |
| Chapter 2 | System Architecture Overview | ✓ COMPLETE |
| Chapter 3 | Compilation Pipeline — Stage-by-Stage | ✓ COMPLETE |
| Chapter 4 | Module Responsibilities & Interfaces | ✓ COMPLETE |
| **Chapter 5** | **Data Flow & Intermediate Representations** | **✓ COMPLETE (this document)** |
| Chapter 6 | Deployment & Runtime Considerations | Pending |
| Chapter 7 | Extensibility & Plugin Architecture | Pending |
| Chapter 8 | Security, Validation & Error Handling | Pending |
| Chapter 9 | Performance Targets & Constraints | Pending |

**Dependencies:** Chapters 1–4, RFC-0002, RFC-0003, KB-0007.

**Recommended Next Chapter:** Chapter 6 — Deployment & Runtime Considerations, which will define deployment topologies, runtime environments, packaging, distribution, and operational concerns.
