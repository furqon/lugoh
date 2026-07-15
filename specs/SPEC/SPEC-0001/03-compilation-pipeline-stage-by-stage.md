---
spec_id: SPEC-0001
chapter: 3
title: Compilation Pipeline — Stage-by-Stage
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - ADR-0001: Compiler Architecture Rationale
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0401: Knowledge Graph Engine
  - RFC-0001: Grammar DSL (proposed)
  - RFC-0002: Grammar Bytecode (proposed)
  - KB-0001: Roots (planned)
  - KB-0002: Wazan (planned)
  - KB-0005: Particles (planned)
  - KB-0006: Pronouns (planned)
  - KB-0007: Morphological Features Taxonomy (planned)
---

# Chapter 3: Compilation Pipeline — Stage-by-Stage

## Table of Contents

1. [Stage Overview](#1-stage-overview)
2. [MOD-01: UnicodeValidator](#2-mod-01-unicodevalidator)
3. [MOD-02: Lexer](#3-mod-02-lexer)
4. [MOD-03: Tokenizer](#4-mod-03-tokenizer)
5. [MOD-04: MorphologicalParser](#5-mod-04-morphologicalparser)
6. [MOD-05: SyntaxParser](#6-mod-05-syntaxparser)
7. [MOD-06: GIRConstructor](#7-mod-06-girconstructor)
8. [MOD-07: RuleEngine](#8-mod-07-ruleengine)
9. [MOD-08: KnowledgeGraphResolver](#9-mod-08-knowledgegraphresolver)
10. [MOD-09: BytecodeGenerator](#10-mod-09-bytecodegenerator)
11. [Cross-Stage Concerns](#11-cross-stage-concerns)
12. [Cross-References](#12-cross-references)

---

## 1. Stage Overview

This chapter defines the **detailed processing specifications** for each of the 9 stages in the AGOS Compilation Layer. Each stage specification includes:

- **Data schema:** The formal input and output type definitions.
- **Processing algorithm:** The step-by-step logic that transforms input to output.
- **Determinism guarantees:** Why the stage is deterministic and what that means for reproducibility.
- **Knowledge dependencies:** Which versioned knowledge bases the stage reads.
- **Edge cases:** Known boundary conditions and how they are handled.
- **Error conditions:** All possible failure modes with structured error codes.
- **Ambiguity handling:** How the stage represents and propagates ambiguity.
- **Performance characteristics:** Time and space complexity, with targets.

### 1.1 Data Schema Notation

Throughout this chapter, data schemas are expressed in a language-neutral pseudo-type notation:

```
type_name = {
  field_name: field_type,         // Required field
  field_name?: field_type,        // Optional field
  field_name: field_type | null,  // Nullable field
  field_name: field_type[],       // Array of field_type
  field_name: { ... },            // Nested object
}
```

Primitive types: `string`, `integer`, `float`, `boolean`, `bytes`.

### 1.2 Ambiguity Representation

Ambiguity is a first-class concept in AGOS. Rather than committing to a single analysis at each stage, the pipeline supports **ambiguity sets** — ordered lists of alternative analyses.

```
ambiguity_set<T> = {
  alternatives: T[],           // Ordered list of alternative analyses
  confidence: "high" | "medium" | "low",  // Aggregate confidence
  source: string,              // Stage or rule that introduced ambiguity
  evidence: evidence_entry[],  // Evidence for each alternative
}
```

Stages SHOULD minimize ambiguity. When ambiguity cannot be resolved, it MUST be propagated to downstream stages (never silently discarded).

---

## 2. MOD-01: UnicodeValidator

### 2.1 Purpose

Validate that the input text is well-formed Arabic text, normalize it to a canonical Unicode representation, and reject text that cannot be processed.

### 2.2 Data Schemas

#### 2.2.1 Input Schema

```
validated_input = {
  raw_text: string,                          // Original input (UTF-8)
  config: {
    normalize_tashkeel: boolean,             // Strip or canonicalize diacritics
    strip_tatweel: boolean,                  // Remove kashida characters
    strict_arabic_only: boolean,             // Reject non-Arabic characters
    allowed_unicode_ranges: string[],        // e.g., ["0600-06FF", "0750-077F"]
  }
}
```

#### 2.2.2 Output Schema

```
normalized_output = {
  normalized_text: string,                   // Normalized UTF-8 string
  original_text: string,                     // Preserved original (for reference)
  metadata: {
    char_count: integer,                     // Character count (after normalization)
    word_count_estimate: integer,            // Approximate word count (whitespace-delimited)
    has_tashkeel: boolean,                   // Whether diacritics were present
    has_tatweel: boolean,                    // Whether tatweel was present
    has_quranic_symbols: boolean,            // Whether Quranic annotation symbols found
    normalization_applied: string[],         // List of normalizations performed
  }
}
```

#### 2.2.3 Error Output Schema

```
validation_error = {
  code: "INVALID_ENCODING" | "NON_ARABIC_CHAR" | "EMPTY_INPUT" |
        "MAX_LENGTH_EXCEEDED" | "UNSUPPORTED_CHAR",
  message: string,                           // Human-readable description
  position: integer | null,                  // Character position of error (if applicable)
  offending_char: string | null,             // The character that caused the error
  recovery_hint: string | null,              // Suggestion for recovery
}
```

### 2.3 Processing Algorithm

```
Algorithm: validate_and_normalize
Input: raw_text (UTF-8 string)
Output: normalized_output or validation_error

Step 1: Encoding Validation
  1.1  Verify raw_text is valid UTF-8.
       If invalid → return INVALID_ENCODING with position of first invalid byte.

Step 2: Length Check
  2.1  If raw_text is empty → return EMPTY_INPUT.
  2.2  If byte_length(raw_text) > MAX_INPUT_SIZE → return MAX_LENGTH_EXCEEDED.
       MAX_INPUT_SIZE = 1,048,576 bytes (1 MiB) by default, configurable.

Step 3: Character Validation (character-by-character)
  3.1  For each character c in raw_text:
  3.1.1  If c is in Arabic block (U+0600–U+06FF) or Arabic Supplement (U+0750–U+077F)
         or Arabic Extended-A (U+08A0–U+08FF) → continue.
  3.1.2  If c is a common character (space, newline, tab, carriage return) → continue.
  3.1.3  If c is a digit (Arabic-Indic U+0660–U+0669, Extended U+06F0–U+06F9) → continue.
  3.1.4  If c is punctuation (Arabic-specific or common) → continue.
  3.1.5  If c is a Quranic symbol (U+06D6–U+06ED, U+08D4–U+08E1) → continue, mark metadata.
  3.1.6  If strict_arabic_only is true → return NON_ARABIC_CHAR with position and char.
  3.1.7  Otherwise → continue (non-Arabic chars are preserved if not strict).

Step 4: Unicode Normalization
  4.1  Apply NFKC normalization to the text.
  4.2  Note: NFKC is preferred over NFD/NFC for Arabic because it handles
       compatibility decompositions (e.g., lam-alef ligatures → lam + alef).

Step 5: Tatweel (Kashida) Handling
  5.1  If strip_tatweel is true:
  5.1.1  Remove all U+0640 (TATWEEL) characters.
  5.1.2  Record "strip_tatweel" in normalization_applied.

Step 6: Tashkeel (Diacritic) Handling
  6.1  If normalize_tashkeel is true:
  6.1.1  Option A (strip): Remove all diacritical marks:
         - Fatha (U+064E), Damma (U+064F), Kasra (U+0650)
         - Fathatan (U+064B), Dammatan (U+064C), Kasratan (U+064D)
         - Shadda (U+0651), Sukun (U+0652)
         - Superscript Alef (U+0670)
  6.1.2  Option B (canonicalize): Normalize diacritic sequences to canonical order
         (e.g., Shadda + Fatha → canonical sequence).
  6.1.3  Record operation in normalization_applied.

Step 7: Detect Metadata
  7.1  Scan for presence of tashkeel, tatweel, Quranic symbols.
  7.2  Estimate word count by whitespace splitting.

Step 8: Return Output
  8.1  Return normalized_output with normalized_text and metadata.
```

### 2.4 Determinism

**Fully deterministic.** Given the same input text and the same configuration, `validate_and_normalize` MUST always produce identical output. No randomness, no external state, no time-dependent behavior.

### 2.5 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| Unicode Character Database | No (built into implementation) | Character classification and normalization rules |

The Unicode Character Database is not versioned as AGOS knowledge because it is a stable international standard. AGOS implementations SHOULD use the Unicode version that was current at the time of implementation, but MUST document which version they use.

### 2.6 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Empty string** | Returns EMPTY_INPUT error. |
| **Whitespace-only string** | Returns normalized_output with empty normalized_text (valid, but will produce empty token stream downstream). |
| **Very long input** | Returns MAX_LENGTH_EXCEEDED if over limit. Configurable. |
| **Mixed Arabic/English** | If strict_arabic_only=false, English characters are preserved. Downstream stages will handle them (likely as unrecognized tokens). |
| **Only non-Arabic** | If strict_arabic_only=false, passes through. Downstream stages will produce empty or minimal analysis. |
| **Quranic Uthmani script** | Characters in the Quranic-specific ranges (U+08D4–U+08FF) are recognized and preserved. |
| **Invisible characters** | Zero-width joiner (U+200D), zero-width non-joiner (U+200C), and BOM (U+FEFF) are handled: BOM is stripped; ZWJ/ZWNJ are preserved (they affect Arabic character shaping). |
| **Diacritic-only text** | A string consisting entirely of diacritics (no base letters) is preserved but downstream will likely produce no tokens. |

### 2.7 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| INVALID_ENCODING | Malformed UTF-8 bytes | Re-encode input as valid UTF-8 |
| EMPTY_INPUT | Empty string provided | Provide non-empty Arabic text |
| MAX_LENGTH_EXCEEDED | Input exceeds configured limit | Split input into smaller segments |
| NON_ARABIC_CHAR | Strict mode enabled and non-Arabic char found | Remove non-Arabic characters or disable strict mode |
| UNSUPPORTED_CHAR | Character in a recognized but unsupported range | Notify AGOS maintainers to add support |

### 2.8 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 100 MB/s on modern hardware |
| Latency (p50) | < 1 μs per KB of input |
| Latency (p99) | < 10 μs per KB of input |
| Memory | O(n) where n = input length |
| Allocations | Single allocation for output string |

---

## 3. MOD-02: Lexer

### 3.1 Purpose

Transform the normalized Unicode string into an ordered stream of raw tokens. A **token** is the smallest meaningful unit of text: a word, a punctuation mark, a number, or a special symbol.

### 3.2 Data Schemas

#### 3.2.1 Input Schema

```
lexer_input = normalized_output   // From MOD-01
```

#### 3.2.2 Output Schema

```
raw_token = {
  id: integer,                            // Sequential token ID (0-based)
  text: string,                           // The token's text (from input)
  type: "word" | "punctuation" | "number" | "whitespace" | "symbol" | "unknown",
  start_offset: integer,                  // Byte offset of start in normalized text
  end_offset: integer,                    // Byte offset of end (exclusive)
}

token_stream = {
  tokens: raw_token[],                    // Ordered list of tokens
  text: string,                           // Reference to input text
  metadata: {
    token_count: integer,
    word_count: integer,                  // Count of type="word" tokens
    has_tokens: boolean,
  }
}
```

#### 3.2.3 Error Output Schema

```
lexer_error = {
  code: "INTERNAL_ERROR",
  message: string,
  position: integer | null,
}
```

The Lexer is a simple stage with minimal error modes. Most validation errors are caught by MOD-01.

### 3.3 Processing Algorithm

```
Algorithm: lex
Input: normalized_output
Output: token_stream or lexer_error

Step 1: Initialize
  1.1  tokens = empty list
  1.2  current_position = 0
  1.3  text = normalized_output.normalized_text

Step 2: Scan Characters
  2.1  While current_position < length(text):

  2.2  Character Classification:
  2.2.1  Determine the class of text[current_position]:
         - ARABIC_LETTER: U+0600–U+06FF letter ranges
         - ARABIC_DIGIT: U+0660–U+0669, U+06F0–U+06F9
         - WHITESPACE: space (U+0020), newline (U+000A), tab (U+0009)
         - PUNCTUATION: . , ; : ! ? ( ) [ ] { } " « » — –
         - SYMBOL: @ # $ % ^ & * + = < > / \ | ~ `
         - OTHER: anything not classified above

  2.3  Token Extraction:
  2.3.1  If class is WHITESPACE:
         - Scan forward while class is WHITESPACE
         - Create token of type "whitespace"
         - Optionally: skip whitespace tokens in output (configurable)

  2.3.2  If class is ARABIC_LETTER:
         - Scan forward while class is ARABIC_LETTER or TASHKEEL
         - Tashkeel characters are INCLUDED in the word token (they modify the letter)
         - Create token of type "word"

  2.3.3  If class is ARABIC_DIGIT:
         - Scan forward while class is ARABIC_DIGIT
         - Include optional decimal separator and grouping marks
         - Create token of type "number"

  2.3.4  If class is PUNCTUATION:
         - Scan forward while class is PUNCTUATION and consecutive
           (e.g., "..." as multi-character punctuation)
         - Create token of type "punctuation"

  2.3.5  If class is SYMBOL:
         - Create single-character token of type "symbol"

  2.3.6  If class is OTHER:
         - Create single-character token of type "unknown"

  2.4  Advance current_position by the length of the extracted token.

Step 3: Post-Processing
  3.1  If configured to skip whitespace, filter whitespace tokens from the list.
  3.2  Validate token_count > 0; if 0 and text is non-empty, log warning.

Step 4: Build Metadata
  4.1  token_count = length(tokens)
  4.2  word_count = count(tokens where type == "word")

Step 5: Return
  5.1  Return token_stream { tokens, text, metadata }.
```

### 3.4 Determinism

**Fully deterministic.** Given the same normalized text, the Lexer MUST always produce the same token stream. Token order is strictly sequential — no parallel tokenization that could introduce non-deterministic ordering.

### 3.5 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| Character class tables | No | Classifying Arabic letters, digits, punctuation |
| Tashkeel character set | No | Identifying diacritics that should be attached to word tokens |

### 3.6 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Consecutive punctuation** | Grouped into a single multi-character punctuation token (e.g., "!!" → one token). |
| **Tashkeel at start of word** | Tashkeel characters preceding a letter are included in the word token. This is rare but valid (diacritic on first letter). |
| **Lone tashkeel** | A diacritic character not adjacent to a letter is treated as "unknown" type. |
| **Multiple spaces** | Whitespace tokens preserve the original whitespace characters; downstream stages can differentiate single space vs. multiple spaces vs. other whitespace. |
| **Hyphenated words** | The hyphen is a punctuation token between two word tokens. The Tokenizer (MOD-03) may recombine them. |
| **URLs and email addresses** | These will be tokenized into their constituent parts: word + symbol + word + ... This is acceptable — they are not valid Arabic grammatical inputs. |

### 3.7 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| INTERNAL_ERROR | Unexpected algorithm failure | Check input validity; report bug |

### 3.8 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 200 MB/s on modern hardware |
| Latency (p50) | < 0.5 μs per KB of input |
| Allocations | O(n) tokens, single pass over input |

---

## 4. MOD-03: Tokenizer

### 4.1 Purpose

Segment each raw word token into its constituent morphemes. This is the first stage that requires linguistic knowledge — it must recognize Arabic clitics (prefixes and suffixes) and separate them from the stem.

### 4.2 Data Schemas

#### 4.2.1 Input Schema

```
tokenizer_input = {
  token_stream: token_stream,              // From MOD-02
  config: {
    max_segmentations: integer,            // Maximum ambiguity alternatives (default: 16)
    known_clitics_path?: string,           // Optional custom clitic table
  }
}
```

#### 4.2.2 Output Schema

```
morpheme = {
  text: string,                            // The morpheme text
  type: "prefix" | "stem" | "suffix" | "clitic" | "particle",
  original_offset: integer,                // Offset in the original token
  length: integer,
}

segmented_token = {
  raw_token: raw_token,                    // Reference to the original raw token
  segmentations: segmentation[],           // Ordered list of alternative segmentations
}

segmentation = {
  morphemes: morpheme[],                   // Ordered from left to right (prefix→stem→suffix)
  confidence: float,                       // 0.0 to 1.0
  source: string,                          // "default" | "custom_rule"
}

segmented_token_stream = {
  tokens: segmented_token[],
  metadata: {
    total_tokens: integer,
    segmentable_tokens: integer,           // Count of tokens with at least one segmentation
    ambiguous_tokens: integer,             // Count of tokens with multiple segmentations
    total_ambiguity: float,                // Average ambiguity per segmentable token
  }
}
```

#### 4.2.3 Error Output Schema

```
tokenizer_error = {
  code: "MAX_SEGMENTATIONS_EXCEEDED" | "INTERNAL_ERROR",
  message: string,
  token_id: integer | null,
}
```

### 4.3 Processing Algorithm

```
Algorithm: tokenize
Input: tokenizer_input
Output: segmented_token_stream or tokenizer_error

Step 1: Initialize
  1.1  For each token t in token_stream.tokens where t.type == "word":

Step 2: Clitic Identification
  2.1  Build list of known clitics (from built-in tables + custom if configured):

  2.1.1  Proclitics (prefixes that attach to word beginnings):
         - Conjunctive: وَ (wa-), فَ (fa-)
         - Prepositional: بِ (bi-), لِ (li-), كَ (ka-)
         - Future marker: سَ (sa-), سَوْ (saw-)
         - Question: أَ (a-), هَلْ (hal — note: hal is a separate word, not a clitic)

  2.1.2  Enclitics (suffixes that attach to word endings):
         - Object pronouns: نِي (-nī), نَا (-nā), كَ (-ka), كِ (-ki),
           هُ (-hu), هَا (-hā), هُمْ (-hum), هُنَّ (-hunna), etc.
         - Possessive pronouns (same forms as object pronouns but attached to nouns)
         - Plural markers: ونَ (-ūna), ينَ (-īna), اتَ (-āta), etc.
         - Dual markers: انِ (-āni), ينِ (-ayni)
         - Feminine markers: ة (-ah / -at)
         - Verb subject markers: تُ (-tu), تَ (-ta), تِ (-ti), etc.

  2.1.3  Clitic Combination Rules:
         - Multiple proclitics can combine: وَبِ (wa-bi-), فَلِ (fa-li-)
         - Proclitic + stem + enclitic: standard Arabic word structure
         - Order: [CONJ] [PREP/FUT] STEM [PRON] [PLURAL]

Step 3: Generate Candidate Segmentations
  3.1  For each known proclitic combination that matches the beginning of t.text:
  3.1.1  Propose a segmentation with that proclitic.
  3.1.2  The remaining string after stripping the proclitic becomes the candidate stem.

  3.2  For each known enclitic combination that matches the end of the candidate stem:
  3.2.1  Propose a segmentation including that enclitic.
  3.2.2  The remaining middle is the core stem.

  3.3  The "empty segmentation" (no clitics, entire token is the stem) is always included
       as the default alternative.

  3.4  Ambiguity handling:
  3.4.1  Example: "ولهم" can be segmented as:
         - وَ + لَ + هُمْ (wa + la + hum = "and for them")
         - وَ + لَهُمْ (wa + lahum = "and lahum" — non-standard)
         Both are valid; both are included in the ambiguity set.
  3.4.2  If the number of segmentations exceeds max_segmentations → return
         MAX_SEGMENTATIONS_EXCEEDED (configurable; should be set high enough for all
         known ambiguous words).

Step 4: Assign Confidence
  4.1  Default segmentation (no clitics): confidence = 0.3 (low — rare for real Arabic text)
  4.2  Single clitic: confidence = 0.7
  4.3  Multiple clitics: confidence = 0.9 (most common for Arabic text)
  4.4  Strange/unusual segmentation: confidence = 0.1
  4.5  (Note: confidence here is heuristic only. The Rule Engine (MOD-07) will
       make the authoritative determination.)

Step 5: Handle Non-Word Tokens
  5.1  For tokens of type "punctuation", "number", "symbol", "whitespace", "unknown":
  5.1.1  Create a single segmentation with the entire token as a morpheme of type "particle".
  5.1.2  No segmentation ambiguity for these types.

Step 6: Build Metadata
  6.1  Count segmentable_tokens, ambiguous_tokens, total_ambiguity.

Step 7: Return
  7.1  Return segmented_token_stream.
```

### 4.4 Arabic Clitic Table (Reference)

The following table defines all known Arabic clitics that the Tokenizer MUST recognize.

#### Proclitics

| Clitic | Transliteration | Function | Examples |
|--------|-----------------|----------|----------|
| وَ | wa | Conjunction "and" | وَقَالَ (wa-qāla) |
| فَ | fa | Conjunction "and so/thus" | فَقَالَ (fa-qāla) |
| بِ | bi | Preposition "with/by" | بِسْمِ (bi-smi) |
| لِ | li | Preposition "for/to" | لِلَّهِ (li-llāhi) |
| كَ | ka | Preposition "like" | كَمَثَلِ (ka-mathali) |
| سَ | sa | Future marker | سَيَقُولُ (sa-yaqūlu) |
| سَوْفَ | sawfa | Future marker (separate word) | — |
| أَ | a | Interrogative prefix | أَتَعْلَمُ (a-taʿlamu) |

#### Combined Proclitics

| Clitic | Components | Example |
|--------|------------|---------|
| وَبِ | wa + bi | وَبِالْحَقِّ (wa-bi-l-ḥaqqi) |
| فَبِ | fa + bi | فَبِمَا (fa-bi-mā) |
| فَلِ | fa + li | فَلِلَّهِ (fa-li-llāhi) |
| وَال | wa + al | وَالَّذِي (wa-alladhī) |
| فَال | fa + al | فَالَّذِينَ (fa-alladhīna) |
| بِال | bi + al | بِالْحَقِّ (bi-l-ḥaqqi) |
| لِل | li + al | لِلَّهِ (li-llāhi) |

#### Enclitics (Object/Possessive Pronouns)

| Singular | Dual | Plural |
|----------|------|--------|
| ي (-ī) / نِي (-nī): me | نَا (-nā): us | نَا (-nā): us |
| كَ (-ka): you (masc) | كُمَا (-kumā): you two | كُمْ (-kum): you all (masc) |
| كِ (-ki): you (fem) | كُمَا (-kumā): you two | كُنَّ (-kunna): you all (fem) |
| هُ (-hu): him/it | هُمَا (-humā): them two | هُمْ (-hum): them (masc) |
| هَا (-hā): her/it | هُمَا (-humā): them two | هُنَّ (-hunna): them (fem) |

#### Verbal Enclitics (Subject Markers)

| Person | Singular | Dual | Plural |
|--------|----------|------|--------|
| 1st | تُ (-tu) | — | نَا (-nā) |
| 2nd masc | تَ (-ta) | تُمَا (-tumā) | تُمْ (-tum) |
| 2nd fem | تِ (-ti) | تُمَا (-tumā) | تُنَّ (-tunna) |
| 3rd masc | — | ا (-ā) | وا (-wā) / وا (-ū) |
| 3rd fem | تْ (-t) | تَا (-tā) | نَ (-na) |

### 4.5 Determinism

**Fully deterministic.** Given the same token stream and the same clitic tables, the Tokenizer MUST always produce the same segmentation output. Ambiguity alternatives are ordered deterministically (by confidence, then by segmentation length).

### 4.6 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| Clitic tables (built-in) | Yes | List of known proclitics, enclitics, and combination rules |
| Arabic particle set | Yes | Identifying standalone particles vs. clitics |

These tables are versioned because new clitic forms or dialect-specific clitics may be added over time.

### 4.7 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Token shorter than any clitic** | Only the default (no-segmentation) alternative is produced. |
| **Clitic matches part of a genuine stem** | Both alternatives are included in the ambiguity set. The morphological parser and rule engine will decide. |
| **Word starting with alif-lam (ال)** | The definite article ال is NOT segmented by the Tokenizer. It is part of the stem and handled by the Morphological Parser. Rationale: ال behaves morphologically as part of the noun, not as a free-standing clitic. |
| **Word ending with ta-marbuta (ة)** | Ta-marbuta is NOT segmented. It is part of the stem. The Morphological Parser will identify it as a feminine marker. |
| **All clitics consumed** | If stripping all possible clitics leaves an empty stem, that segmentation is rejected. |
| **Multiple identical segmentations** | Duplicate segmentations are deduplicated. |

### 4.8 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| MAX_SEGMENTATIONS_EXCEEDED | Number of ambiguity alternatives exceeds limit | Increase max_segmentations; simplify the input |
| INTERNAL_ERROR | Unexpected failure | Report bug |

### 4.9 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 500K tokens/second |
| Latency (p50) | < 2 μs per token |
| Memory | O(a) where a = ambiguity factor × token length |

---

## 5. MOD-04: MorphologicalParser

### 5.1 Purpose

Perform morphological (sarf) analysis on each stem: identify the root (jadhr), the morphological pattern (wazan), the part of speech, and all morphological features.

This is the most linguistically complex stage in the pipeline. It requires comprehensive knowledge bases and sophisticated matching algorithms.

### 5.2 Data Schemas

#### 5.2.1 Input Schema

```
morphological_input = {
  segmented_stream: segmented_token_stream,   // From MOD-03
  config: {
    school: "basra" | "kufa" | "baghdad" | "andalus" | "modern",
    max_morphological_analyses: integer,      // Per-stem limit (default: 32)
    enable_guess: boolean,                    // Allow guess for unknown roots
    known_words_path?: string,                // Custom dictionary
  }
}
```

#### 5.2.2 Output Schema

```
morphological_feature = {
  name: string,                              // e.g., "gender", "number", "person", "tense"
  value: string,                             // e.g., "masculine", "plural", "third", "past"
  confidence: float,                         // 0.0 to 1.0
  source: string,                            // KB entry or rule that determined this feature
}

morphological_analysis = {
  stem: string,                              // The stem text (after clitic removal)
  root: {                                    // The triliteral/quadriliteral root
    text: string,                            // e.g., "كتب" (k-t-b)
    source: string,                          // KB-0001 entry ID
    confidence: float,
  } | null,
  wazan: {                                   // The morphological pattern
    text: string,                            // e.g., "فَعَلَ" (faʿala)
    source: string,                          // KB-0002 entry ID
    form: integer | null,                    // Verb form I-XV, or null for nouns
    confidence: float,
  } | null,
  pos: "verb" | "noun" | "particle" | "proper_noun" | "pronoun" | "adjective" |
       "adverb" | "preposition" | "conjunction" | "interrogative" | "unknown",
  features: morphological_feature[],         // All extracted features
  is_ambiguous: boolean,                     // Whether multiple analyses exist
  alternatives: morphological_analysis[],    // Alternative analyses (ambiguity)
  evidence: evidence_entry[],                // Evidence for this analysis
}

morphological_analysis_output = {
  token_analyses: {
    token_id: integer,
    stem_analyses: morphological_analysis[],  // Per-stem (one stem per segmentation)
  }[],
  metadata: {
    total_tokens: integer,
    analyzed_tokens: integer,
    ambiguous_tokens: integer,
    unknown_tokens: integer,
    unknown_stems: string[],                 // Stems that could not be analyzed
  }
}
```

#### 5.2.3 Error Output Schema

```
morphological_error = {
  code: "MAX_ANALYSES_EXCEEDED" | "KB_MISSING" | "KB_VERSION_MISMATCH" | "INTERNAL_ERROR",
  message: string,
  token_id: integer | null,
  kb_reference: string | null,
}
```

### 5.3 Processing Algorithm

```
Algorithm: analyze_morphology
Input: morphological_input
Output: morphological_analysis_output or morphological_error

Step 1: Load Knowledge Bases
  1.1  Load KB-0001 (Roots) — version must match configured school.
  1.2  Load KB-0002 (Wazan/Patterns).
  1.3  Load KB-0003 (Verb Forms).
  1.4  Load KB-0004 (Noun Patterns).
  1.5  Load KB-0005 (Particles).
  1.6  Load KB-0006 (Pronouns).
  1.7  Load KB-0007 (Morphological Features).
  1.8  If any KB version is incompatible with the configured school → KB_VERSION_MISMATCH.

Step 2: For Each Token
  2.1  For each segmentation in token.segmentations:

Step 3: Identify Part of Speech (Fast Path)
  3.1  Check if stem ∈ KB-0005 (Particles):
  3.1.1  If yes → mark as "particle". Skip further morphological analysis.
  3.1.2  Record features from the particle's KB entry.

  3.2  Check if stem ∈ KB-0006 (Pronouns):
  3.2.1  If yes → mark as "pronoun". Skip further morphological analysis.
  3.2.2  Record features (person, number, gender) from the pronoun's KB entry.

Step 4: Root Extraction (for verbs and nouns)
  4.1  For stems not identified as particles or pronouns:

  4.2  Known Word Lookup:
  4.2.1  Look up the stem directly in the known words index (derived from KB-0001–0004).
  4.2.2  If found: retrieve root, wazan, and features directly. → Go to Step 6.

  4.3  Root Extraction Algorithm:
  4.3.1  Strip any remaining non-root characters (redundant after Tokenizer, but safe).
  4.3.2  Extract candidate root consonants:
         - For triliteral roots: extract 3 consonants from the stem by removing
           non-root letters (alif, waw, ya, ta, mim, nun when they serve as affixes).
         - For quadriliteral roots: extract 4 consonants.
  4.3.3  Match candidate root against KB-0001.
  4.3.4  If match found → proceed to wazan identification.
  4.3.5  If no match → try alternative root extraction heuristics.

  4.4  Wazan Identification:
  4.4.1  Given the root and the stem, determine which wazan (pattern) matches.
  4.4.2  Match by aligning root consonants with pattern slots:
         - فَعَلَ (faʿala): C₁aC₂aC₃a — basic Form I verb
         - فَعَّلَ (faʿʿala): C₁aC₂C₂aC₃a — Form II (geminated middle)
         - فَاعَلَ (fāʿala): C₁āC₂aC₃a — Form III
         - أَفْعَلَ (afʿala): aC₁C₂aC₃a — Form IV
         - تَفَعَّلَ (tafaʿʿala): taC₁aC₂C₂aC₃a — Form V
         - تَفَاعَلَ (tafāʿala): taC₁āC₂aC₃a — Form VI
         - اِنْفَعَلَ (infaʿala): inC₁aC₂aC₃a — Form VII
         - اِفْتَعَلَ (iftaʿala): iC₁taC₂aC₃a — Form VIII
         - اِفْعَلَّ (ifʿalla): iC₁C₂aC₃C₃a — Form IX
         - اِسْتَفْعَلَ (istafʿala): istaC₁C₂aC₃a — Form X
         اِفْعَوْعَلَ (ifʿawʿala), اِفْعَالَّ (ifʿālla), etc.
  4.4.3  For nouns: match against known noun patterns (KB-0004).
  4.4.4  If multiple patterns match → include all as ambiguity alternatives.

Step 5: Feature Extraction
  5.1  From the identified root + wazan + stem form, extract morphological features:

  5.2  For Verbs:
       - Tense: past (madi), present (mudari'), imperative (amr)
       - Person: first, second, third
       - Gender: masculine, feminine
       - Number: singular, dual, plural
       - Mood (present tense only): indicative (raf'), subjunctive (nasb),
         jussive (jazm), energetic (ta'kid)
       - Voice: active (ma'lum), passive (majhul)
       - Transitivity: transitive, intransitive (from root properties)
       - Verb form: I–XV

  5.3  For Nouns:
       - Gender: masculine, feminine
       - Number: singular, dual, plural (sound masculine, sound feminine,
         broken — determine broken plural pattern if applicable)
       - State: definite (ma'rifah), indefinite (nakirah)
       - Case (default, may be overridden by syntax): nominative (raf'),
         accusative (nasb), genitive (jarr)
       - Noun type: verbal noun (masdar), active participle (ism fa'il),
         passive participle (ism maf'ul), noun of place (ism makan),
         noun of time (ism zaman), noun of instrument (ism alah),
         adjective (sifah), etc.

  5.4  For Pronouns:
       - Person, gender, number, pronoun type (attached/munfasil)

  5.5  For Particles:
       - Particle type (preposition, conjunction, etc.)
       - Grammatical effect (e.g., which case it assigns)

Step 6: Handle Unknown Stems
  6.1  If enable_guess is true:
  6.1.1  Attempt root extraction with relaxed matching.
  6.1.2  Mark confidence as "low".
  6.1.3  Include a note in evidence explaining that the analysis is a guess.

  6.2  If enable_guess is false and no analysis found:
  6.2.1  Mark stem as "unknown".
  6.2.2  Include stem in metadata.unknown_stems.
  6.2.3  Downstream stages (Rule Engine, Explanation Engine) may still
         produce partial analysis.

Step 7: Build Ambiguity Sets
  7.1  For stems with multiple possible analyses:
  7.1.1  Order alternatives by confidence (highest first).
  7.1.2  If alternatives exceed max_morphological_analyses → truncate.
  7.1.3  Mark token as ambiguous.

Step 8: Build Metadata & Return
  8.1  Count analyzed, ambiguous, unknown tokens.
  8.2  Return morphological_analysis_output.
```

### 5.4 Arabic Verb Forms I–XV (Reference)

| Form | Pattern | Example | Meaning |
|------|---------|---------|---------|
| I | فَعَلَ | كَتَبَ (kataba) | Basic form "he wrote" |
| II | فَعَّلَ | كَتَّبَ (kattaba) | Intensive/causative "he caused to write" |
| III | فَاعَلَ | كَاتَبَ (kātaba) | Attemptive/reciprocal "he corresponded" |
| IV | أَفْعَلَ | أَكْتَبَ (aktaba) | Causative "he dictated" |
| V | تَفَعَّلَ | تَكَتَّبَ (takattaba) | Reflexive of II "he registered/enrolled" |
| VI | تَفَاعَلَ | تَكَاتَبَ (takātaba) | Reciprocal/reflexive of III "they corresponded" |
| VII | اِنْفَعَلَ | اِنْكَتَبَ (inkataba) | Passive/reflexive "he subscribed" |
| VIII | اِفْتَعَلَ | اِكْتَتَبَ (iktataba) | Reflexive "he copied/recorded" |
| IX | اِفْعَلَّ | اِحْمَرَّ (iḥmarra) | Colors/defects "he became red" |
| X | اِسْتَفْعَلَ | اِسْتَكْتَبَ (istaktaba) | Requestive "he asked to write" |
| XI | اِفْعَالَّ | اِحْمَارَّ (iḥmārra) | Intensive color (rare) |
| XII | اِفْعَوْعَلَ | اِحْدَوْدَبَ (iḥdawdaba) | Intensive (rare) |
| XIII | اِفْعَوَّلَ | اِجْلَوَّذَ (ijlawwadha) | Very rare |
| XIV | اِفْعَنْلَلَ | اِقْعَنْسَسَ (iqʿansasa) | Very rare |
| XV | اِفْعَنْلَى | اِحْرَنْجَمَ (iḥranjama) | Very rare |

### 5.5 Determinism

**Fully deterministic.** Given the same segmented tokens and the same KB versions, the MorphologicalParser MUST always produce identical analyses. Root extraction, wazan matching, and feature extraction are all rule-based algorithms with no random components.

### 5.6 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| KB-0001: Roots | Yes (semver) | Root dictionary with triliteral and quadriliteral roots |
| KB-0002: Wazan | Yes (semver) | Morphological pattern database |
| KB-0003: Verb Forms | Yes (semver) | Verb conjugation paradigms |
| KB-0004: Noun Patterns | Yes (semver) | Noun and adjective pattern database |
| KB-0005: Particles | Yes (semver) | Particle list with grammatical properties |
| KB-0006: Pronouns | Yes (semver) | Pronoun list with features |
| KB-0007: Morphological Features | Yes (semver) | Feature taxonomy and allowed values |
| School-specific morphology rules | Yes (semver) | Rules specific to the configured grammar school |

### 5.7 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Verb with weak root (أجوف, ناقص)** | Weak-root verbs (e.g., قال, باع, دعا) require special handling in root extraction. The root tables include weak root variants, and the extraction algorithm recognizes patterns with weak letters. |
| **Hamzated verb (مهموز)** | Hamza (ء) at the beginning, middle, or end of a root requires special matching. |
| **Doubled verb (مضاعف)** | Roots with identical second and third consonants (e.g., مَدَّ < م-د-د) require deduplication handling. |
| **Broken plural** | Broken plurals (جمع تكسير) do not follow regular suffix patterns. They are identified through the known words index and pattern matching. |
| **Homograph stems** | A single stem with multiple possible roots and meanings (e.g., عين as "eye" or "spring of water" or "to appoint") → all analyses included as ambiguity alternatives. |
| **Deficient verb (ناقص)** | Verbs with final weak letter (e.g., رَمَى, دَعَا) have special conjugation patterns. |
| **Assimilated verb (مثال)** | Verbs with initial waw or ya (e.g., وَجَدَ, يَسَرَ) that assimilate in certain tenses. |
| **Proper nouns** | Proper nouns are recognized from a separate list (future KB extension) or by the definite article + context. Unknown proper nouns are analyzed as regular nouns. |

### 5.8 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| MAX_ANALYSES_EXCEEDED | Per-stem ambiguity exceeds limit | Increase limit; simplify input |
| KB_MISSING | Required knowledge base not found | Verify knowledge base installation |
| KB_VERSION_MISMATCH | KB version incompatible with school | Update KB or select compatible school |
| INTERNAL_ERROR | Unexpected algorithm failure | Report bug |

### 5.9 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 10K stems/second |
| Latency (p50) | < 100 μs per stem |
| Latency (p99) | < 1 ms per stem |
| Memory | Depends on KB size; KBs SHOULD be memory-mapped for efficient access |

---

## 6. MOD-05: SyntaxParser

### 6.1 Purpose

Parse the syntactic (nahw) structure of the sentence. Given the morphological analysis of each token, determine how the tokens relate to each other grammatically: which is the verb, which is the subject, which is the object, and so on.

### 6.2 Data Schemas

#### 6.2.1 Input Schema

```
syntax_input = {
  morphology: morphological_analysis_output,   // From MOD-04
  config: {
    max_parse_trees: integer,                  // Max ambiguity alternatives (default: 8)
    max_sentence_length: integer,              // Max tokens for full parse (default: 200)
  }
}
```

#### 6.2.2 Output Schema

```
syntactic_role = "mubtada" | "khabar" | "fi'l" | "fa'il" | "maf'ul_bi-hi" |
                 "maf'ul_mutlaq" | "maf'ul_fih" | "maf'ul_lahu" | "maf'ul_ma'ahu" |
                 "hal" | "tamyiz" | "na'at" | "idafa" | "mudaf" | "mudaf_ilayh" |
                 "harf_jarr" | "majrur" | "harf_nasb" | "harf_jazm" | "zarf" |
                 "qayd" | "ta'kid" | "badal" | "atasf" | "istithna" | "nida" |
                 "jawab" | "shart" | "jaza" | "sila" | "rabit" | "unknown"

constituent = {
  type: "word" | "phrase" | "clause",
  role: syntactic_role,
  token_ids: integer[],                       // Token IDs that this constituent spans
  children: constituent[],                    // Sub-constituents (for phrases/clauses)
  features: { ... },                          // Syntactic features (case, mood, etc.)
}

syntax_tree = {
  type: "jumlah_ismiyyah" | "jumlah_fi'liyyah" | "jumlah_shartiyyah" |
         "jumlah_zar_fiyyah" | "phrase" | "incomplete" | "unknown",
  root: constituent,                          // Root of the parse tree
  confidence: float,
}

syntax_output = {
  trees: syntax_tree[],                       // Ordered by confidence (ambiguity forest)
  metadata: {
    sentence_count: integer,
    tokens_parsed: integer,
    ambiguity_count: integer,
    parse_time_ms: float,
  }
}
```

#### 6.2.3 Error Output Schema

```
syntax_error = {
  code: "MAX_TREES_EXCEEDED" | "SENTENCE_TOO_LONG" | "PARSE_FAILURE" | "INTERNAL_ERROR",
  message: string,
  position: integer | null,
}
```

### 6.3 Processing Algorithm

```
Algorithm: parse_syntax
Input: syntax_input
Output: syntax_output or syntax_error

Step 1: Sentence Segmentation
  1.1  Group tokens into sentence boundaries.
  1.2  Sentence boundaries are determined by:
       - Punctuation: period, question mark, exclamation mark, semicolon
       - Conjunctions that introduce new clauses: وَ, فَ, ثُمَّ, etc.
       - Quranic verse markers (ayah boundaries)
  1.3  If sentence length > max_sentence_length → SENTENCE_TOO_LONG (splittable).

Step 2: For Each Sentence:

Step 3: Identify Sentence Type
  3.1  Examine the first word's morphological features:
  3.1.1  If first word is a verb (fi'l) → sentence type = jumlah fi'liyyah (verbal).
  3.1.2  If first word is a noun/pronoun in nominative case → sentence type =
         jumlah ismiyyah (nominal).
  3.1.3  If first word is a conditional particle (in, law, etc.) → number shartiyyah.
  3.1.4  Otherwise → mark as "unknown" and attempt partial parse.

Step 4: Verbal Sentence Parsing (jumlah fi'liyyah)
  4.1  Locate the verb (fi'l):
  4.1.1  Often the first word, but may be preceded by harf_nasb/jazm or conjunction.
  4.1.2  Match verb's features with expected subject features.

  4.2  Locate the subject (fa'il):
  4.2.1  If verb has an overt subject pronoun suffix (e.g., تُ, تَ, وا, etc.):
         the subject is implicit (damir mustatir).
  4.2.2  If verb is in the active voice and has no overt subject pronoun:
         the subject is a separate noun in nominative case following the verb.
  4.2.3  If verb is passive (majhul): the subject is replaced by na'ib al-fa'il
         (deputy subject) in nominative case.

  4.3  Locate objects (maf'ul):
  4.3.1  Transitive verbs require objects.
  4.3.2  Direct object (maf'ul bi-hi): accusative case noun/pronoun.
  4.3.3  Absolute object (maf'ul mutlaq): cognate accusative.
  4.3.4  Adverbial object (maf'ul fih): time or place adverb in accusative.
  4.3.5  Purposive object (maf'ul lahu): reason/goal in accusative.
  4.3.6  Accompaniment object (maf'ul ma'ahu): "with" in accusative.

Step 5: Nominal Sentence Parsing (jumlah ismiyyah)
  5.1  Locate the topic (mubtada'):
  5.1.1  The first noun/pronoun in nominative case.
  5.1.2  Usually definite (ma'rifah).

  5.2  Locate the comment (khabar):
  5.2.1  The remainder of the sentence that provides information about the topic.
  5.2.2  May be: a noun in nominative, a prepositional phrase, a verb clause, etc.
  5.2.3  Usually indefinite (nakirah).

  5.3  Identify agreement:
  5.3.1  Mubtada' and khabar agree in gender and number (usually — with exceptions).

Step 6: Constructions Within Sentences
  6.1  Idafa (Construct State):
  6.1.1  Two nouns where the first is indefinite and the second is in genitive case.
  6.1.2  First noun loses tanwin and definite article.
  6.1.3  Link them as mudaf → mudaf_ilayh.

  6.2  Wasf (Adjective Agreement):
  6.2.1  An adjective (na'at) following a noun.
  6.2.2  Adjective agrees with the noun in: gender, number, case, state.
  6.2.3  Note: broken plural nouns take feminine singular adjective agreement.

  6.3  Tawkid (Emphasis):
  6.3.1  Emphatic constructions: nafs, 'ayn, kull, etc. following a noun.
  6.3.2  Emphasizer agrees with emphasized noun in case.

  6.4  Badal (Apposition):
  6.4.1  A noun following another noun, explaining/replacing it.
  6.4.2  Follows the case of the replaced noun.

Step 7: Handle Ambiguity
  7.1  For each ambiguity in morphology (multiple possible analyses per token):
  7.1.1  Try each morphological analysis in the parse.
  7.1.2  Some combinations will produce valid parses, others will fail syntactic rules.
  7.1.3  Each valid (morphological_analysis × syntax_rule) combination produces
         a distinct parse tree.

  7.2  If no parse tree is found:
  7.2.1  Fall back to partial parsing: identify recognizable constituents
         (prepositional phrases, known verb-subject pairs, etc.).
  7.2.2  Mark overall parse with low confidence.
  7.2.3  Record PARSE_FAILURE evidence.

  7.3  If > max_parse_trees → keep the highest-confidence ones (by syntactic rule priority).

Step 8: Return
  8.1  Return syntax_output.
```

### 6.4 Syntactic Role Reference

| Role | Arabic | Definition | Case |
|------|--------|------------|------|
| **mubtada'** | مبتدأ | Topic/subject of nominal sentence | Nominative (raf') |
| **khabar** | خبر | Comment/predicate of nominal sentence | Nominative (raf') |
| **fi'l** | فعل | Verb | — |
| **fa'il** | فاعل | Subject of a verb | Nominative (raf') |
| **na'ib al-fa'il** | نائب الفاعل | Deputy subject (passive) | Nominative (raf') |
| **maf'ul bihi** | مفعول به | Direct object | Accusative (nasb) |
| **maf'ul mutlaq** | مفعول مطلق | Absolute object/cognate accusative | Accusative (nasb) |
| **maf'ul fih** | مفعول فيه | Adverb of time/place (zarf) | Accusative (nasb) |
| **maf'ul lahu** | مفعول لأجله | Object of purpose/reason | Accusative (nasb) |
| **maf'ul ma'ahu** | مفعول معه | Object of accompaniment | Accusative (nasb) |
| **hal** | حال | Circumstantial accusative | Accusative (nasb) |
| **tamyiz** | تمييز | Specification/distinction | Accusative (nasb) |
| **na'at** | نعت | Adjective/qualifier | Follows noun |
| **tawkid** | توكيد | Emphasizer | Follows noun |
| **badal** | بدل | Apposition/substitute | Follows noun |
| **mudaf** | مضاف | First term of construct state | — |
| **mudaf ilayh** | مضاف إليه | Second term of construct state | Genitive (jarr) |
| **majrur** | مجرور | Noun governed by preposition | Genitive (jarr) |
| **harf jarr** | حرف جر | Preposition | — |
| **harf nasb** | حرف نصب | Accusative particle | — |
| **harf jazm** | حرف جزم | Jussive particle | — |

### 6.5 Determinism

**Fully deterministic.** Given the same morphological analysis and the same grammar school's syntax rules, the SyntaxParser MUST always produce the same parse trees (in the same order).

### 6.6 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| School-specific syntax rules | Yes | Rules for sentence structure per school |
| KB-0007: Morphological Features | Yes | Feature values used in agreement checking |

### 6.7 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Single-word utterance** | Parsed as a complete sentence (verbal or nominal depending on the word's POS). |
| **Verb-subject-object ambiguity** | In unvocalized text, ضَرَبَ مُوسَى عِيسَى could be "Musa hit 'Isa" or "Musa was hit by 'Isa" depending on which is subject. Both parses are produced. |
| **Apparent subject-verb disagreement** | In certain constructions (e.g., verb before plural non-human subject), apparent disagreement is valid. The parser handles this through school-specific rules. |
| **Ellipsis (hadhf)** | Omitted elements (e.g., implied verb, implied subject) are marked as implicit (damir mustatir / implicit constituent) rather than missing. |
| **Conjoined sentences** | Conjunctions (وَ, فَ) may join two complete sentences. Each is parsed independently, and the conjunction links them. |
| **Oath constructions** | Oaths (qasam) beginning with وَ or بِ have special parsing rules. |
| **Conditional sentences (jumlah shartiyyah)** | The shart (condition) + jawab (result) structure requires special handling. Both clauses are parsed, with the condition particle linking them. |

### 6.8 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| MAX_TREES_EXCEEDED | Ambiguity exceeds max_parse_trees | Increase limit |
| SENTENCE_TOO_LONG | Token count exceeds max_sentence_length | Split sentence manually |
| PARSE_FAILURE | No valid parse tree found for any morphological combination | Examine input for non-standard grammar or typos |
| INTERNAL_ERROR | Unexpected failure | Report bug |

### 6.9 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 1K sentences/second |
| Latency (p50) | < 1 ms per sentence |
| Latency (p99) | < 10 ms per sentence |
| Parse complexity | O(n³) worst case (CKY/Earley), O(n²) expected for typical Arabic sentences |

---

## 7. MOD-06: GIRConstructor

### 7.1 Purpose

Combine the morphological analyses and syntax trees into a single unified **Grammar Intermediate Representation (GIR)**. The GIR is the canonical representation of the grammatical state of the text at an intermediate point in the pipeline.

### 7.2 Data Schemas

#### 7.2.1 Input Schema

```
gir_input = {
  morphology: morphological_analysis_output,   // From MOD-04
  syntax: syntax_output,                       // From MOD-05
  original_text: string,                       // From MOD-01
  config: {
    gir_version: string,                       // GIR format version (e.g., "1.0")
  }
}
```

#### 7.2.2 Output Schema

```
gir_token = {
  index: integer,                              // Token index in text
  original_text: string,                       // Original token text
  normalized_text: string,                     // Normalized token text
  start_offset: integer,                       // Byte offset in original text
  end_offset: integer,                         // Byte offset (exclusive)
  morphology: morphological_analysis | morphological_analysis[],  // Single or ambiguous
  clitics: { prefix, suffix },                 // Separated clitics (from MOD-03)
}

gir_constituent = {
  type: "token" | "phrase" | "clause",
  role: syntactic_role,                        // e.g., "fi'l", "fa'il", "mubtada'"
  token_indices: integer[],                    // Spanned token indices
  children: gir_constituent[],                 // Sub-constituents
  features: { ... },                           // Syntactic features
  confidence: float,
}

gir_tree = {
  id: string,                                  // Unique tree ID
  sentence_type: string,                       // e.g., "jumlah_fi'liyyah"
  root: gir_constituent,                       // Root constituent
  confidence: float,
  source: string,                              // Grammar school that produced this
}

grammar_ir = {
  version: string,                             // GIR format version
  spec_id: "SPEC-0001",
  metadata: {
    created_at: string,                        // ISO 8601 timestamp
    pipeline_version: string,                  // AGOS platform version
    knowledge_versions: {                      // KB versions used
      "KB-0001": "1.2.3",
      "KB-0002": "2.0.1",
      ...
    },
    school: string,                            // Grammar school
  },
  text: string,                                // Original input text
  tokens: gir_token[],                         // All tokens
  trees: gir_tree[],                           // Parse trees (ambiguity forest)
  evidence: evidence_entry[],                  // All evidence so far
}
```

### 7.3 Processing Algorithm

```
Algorithm: construct_gir
Input: gir_input
Output: grammar_ir

Step 1: Align Tokens
  1.1  For each token in the original text:
  1.1.1  Look up the token's morphological analyses (from MOD-04).
  1.1.2  Look up the token's syntactic role(s) (from MOD-05).
  1.1.3  Create a gir_token with both pieces of information.

Step 2: Build Token-Tree Mapping
  2.1  For each syntax tree in MOD-05 output:
  2.1.1  Map each tree constituent to the gir_tokens it spans.
  2.1.2  Where MOD-04 has ambiguity (multiple morphological analyses per token),
         and MOD-05 has multiple parse trees, create gir_trees for each valid
         (morphology × syntax) combination.
  2.1.3  Prune combinations that are inconsistent (e.g., a morphology that
         parses as verb but the syntax tree expects a noun).

Step 3: Build Ambiguity Forest
  3.1  Group the resulting gir_trees into the ambiguity forest:
  3.1.1  Trees are ordered by confidence (highest first).
  3.1.2  Trees from the configured school are preferred.
  3.1.3  Trees from other schools are included with lower confidence.

Step 4: Collect Evidence Trail
  4.1  Gather all evidence entries from:
       - MOD-03: segmentation evidence
       - MOD-04: morphological analysis evidence
       - MOD-05: syntactic parse evidence
  4.2  Each evidence entry must include:
       - Stage name
       - Rule/algorithm applied
       - Input state (hash or reference)
       - Output produced
       - Confidence

Step 5: Assemble Metadata
  5.1  Record KB versions, pipeline version, school, timestamp.
  5.2  Validate GIR structure against the GIR version's schema.

Step 6: Return
  6.1  Return grammar_ir.
```

### 7.4 Determinism

**Fully deterministic.** The GIRConstructor is a pure structural transformation. Given the same inputs from MOD-04 and MOD-05, it MUST always produce the same GIR.

### 7.5 Knowledge Dependencies

None. The GIRConstructor is purely structural — it combines inputs without requiring any external knowledge.

### 7.6 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Empty text** | Produces a GIR with empty token and tree arrays. |
| **No syntax tree** | If SyntaxParser produced PARSE_FAILURE, the GIR contains morphological analysis only, with a partial parse flag in metadata. |
| **Token without morphology** | Extremely rare (MOD-04 produces at least unknown status for every token). Tokens are included with unknown marker. |
| **Mismatched token counts** | If MOD-04 and MOD-05 disagree on token count (should not happen — they share the same token stream), an INTERNAL_ERROR is raised. |

### 7.7 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| TOKEN_MISMATCH | MOD-04 and MOD-05 token counts disagree | Bug — report immediately |
| GIR_VERSION_INCOMPATIBLE | Output GIR version not supported | Update GIRConstructor |
| INTERNAL_ERROR | Unexpected failure | Report bug |

### 7.8 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 10K sentences/second |
| Latency | < 200 μs per sentence |
| Memory | O(t + p) where t = tokens, p = parse trees |

---

## 8. MOD-07: RuleEngine

### 8.1 Purpose

Apply grammatical rules (qawa'id) to the GIR. The Rule Engine is where the configured grammar school's rules are applied to confirm, reject, or modify the analyses produced by earlier stages.

### 8.2 Data Schemas

#### 8.2.1 Input Schema

```
rule_input = {
  gir: grammar_ir,                             // From MOD-06
  config: {
    school: string,                            // Grammar school (e.g., "basra")
    rule_set_version: string,                  // Specific rule set version
    max_rule_applications: integer,            // Max rules per token (default: 1000)
    strict_mode: boolean,                      // Fail on rule conflict (vs. preserve ambiguity)
  }
}
```

#### 8.2.2 Output Schema

```
rule_application = {
  rule_id: string,                             // Unique rule identifier
  rule_name: string,                           // Human-readable rule name
  school: string,                              // School that defines this rule
  version: string,                             // Rule version
  applies_to: {                                // What the rule was applied to
    token_indices: integer[],
    constituent_path: string[],                // Path in the syntax tree
  },
  condition: string,                           // The condition that triggered the rule
  action: string,                              // What the rule did
  result: {                                    // The result of rule application
    confirmed?: string[],                      // Analyses that were confirmed
    rejected?: string[],                       // Analyses that were rejected
    modified?: modification[],                 // Features that were modified
    flag?: flag,                               // Grammatical flag (error/warning)
  },
  evidence: evidence_entry,
}

annotated_gir = grammar_ir & {
  rule_applications: rule_application[],       // All rule applications
  flags: flag[],                               // All grammatical flags
  rule_set_version: string,                    // The rule set version applied
  school: string,                              // The school whose rules were applied
}

flag = {
  type: "error" | "warning" | "info",
  code: string,                                // e.g., "SUBJECT_VERB_AGREEMENT"
  message: string,                             // Human-readable
  token_indices: integer[],
  rule_id: string,
}
```

#### 8.2.3 Error Output Schema

```
rule_error = {
  code: "RULE_SET_NOT_FOUND" | "RULE_VERSION_MISMATCH" | "RULE_APPLICATION_LIMIT" |
         "RULE_CONFLICT" | "INTERNAL_ERROR",
  message: string,
  rule_id: string | null,
}
```

### 8.3 Processing Algorithm

```
Algorithm: apply_rules
Input: rule_input
Output: annotated_gir or rule_error

Step 1: Load Rule Set
  1.1  Load the Grammar DSL rule set for the configured school and version.
  1.2  If not found → RULE_SET_NOT_FOUND.
  1.3  If version is incompatible → RULE_VERSION_MISMATCH.
  1.4  Rules are ordered by priority (higher priority rules fire first).

Step 2: Initialize Application Context
  2.1  workflow = copy of gir (the rule engine works on a mutable copy)
  2.2  applications = empty list
  2.3  flags = empty list
  2.4  applied_count = 0

Step 3: Apply Rules (Ordered by Priority)
  3.1  For each rule r in rule_set (ordered by priority descending):

  3.2  Evaluate Rule Condition:
  3.2.1  Evaluate r.condition against the current state of workflow.
  3.2.2  Condition may reference:
         - Token features (gender, number, person, case, etc.)
         - Syntactic roles (is_fa'il, is_mubtada', etc.)
         - Constituent relationships (is_parent, is_child, etc.)
         - Current flags and annotations
  3.2.3  If condition is not met → skip this rule, continue to next.
  3.2.4  If condition is met → proceed to apply.

  3.3  Apply Rule Action:
  3.3.1  Execute r.action on the relevant tokens/constituents:
         - CONFIRM: Mark an ambiguous analysis as correct.
         - REJECT: Remove an ambiguous analysis from consideration.
         - MODIFY: Change a feature value (e.g., set case from nominative to accusative).
         - FLAG: Add a grammatical flag (error, warning, info).
         - RESOLVE: Resolve anaphora (pronoun → antecedent).
  3.3.2  Record the rule_application in applications.
  3.3.3  applied_count += 1

  3.4  Check Application Limit:
  3.4.1  If applied_count >= max_rule_applications → RULE_APPLICATION_LIMIT.

  3.5  Conflict Detection:
  3.5.1  If two rules produce contradictory modifications on the same token/constituent:
  3.5.2  If strict_mode is true → RULE_CONFLICT (fail).
  3.5.3  If strict_mode is false → preserve both alternatives as ambiguity,
         record conflict in evidence.

Step 4: Post-Processing
  4.1  After all rules have been applied:
  4.1.1  If ambiguity remains, sort alternatives by:
         - Number of confirming rules (more = higher confidence)
         - Rule priority (higher priority rules' confirmations weigh more)
         - School preference (default school's analysis preferred)

  4.2  Generate summary flags:
  4.2.1  If a known grammatical violation is detected → error flag.
  4.2.2  If an unusual but grammatical construction is found → info flag.

Step 5: Return
  5.1  Return annotated_gir with rule_applications, flags, rule_set_version, school.
```

### 8.4 Example Rule Applications

#### Example 1: Verbal Sentence Subject-Verb Agreement

```
Rule ID: basra-0103
School: Basra
Condition:
  IF constituent.type == "jumlah_fi'liyyah"
  AND fi'l.person != fa'il.person
  
Action:
  REJECT this parse tree (subject-verb person disagreement)
  FLAG: error: "SUBJECT_VERB_PERSON_MISMATCH"
```

#### Example 2: Case Assignment by Preposition

```
Rule ID: basra-0201
School: Basra
Condition:
  IF token.role == "majrur"
  AND governing_particle.type == "harf_jarr"
  AND token.case != "genitive"

Action:
  MODIFY token.case → "genitive"
  CONFIRM this analysis (preposition governs genitive)
```

#### Example 3: Kana and Her Sisters

```
Rule ID: basra-0305
School: Basra
Condition:
  IF sentence has verb in subset {"kāna", "ṣāra", "laysa", "aṣbaḥa", "amsā", ...}
  AND sentence.type == "jumlah_fi'liyyah"

Action:
  MODIFY sentence.type → "jumlah_kāna"
  MODIFY mubtada'.case → "accusative" (ism kāna)
  MODIFY khabar.case → "nominative" (khabar kāna)
  CONFIRM this re-analysis
```

### 8.5 Determinism

**Fully deterministic.** Given the same GIR and the same rule set (same school, same version), the RuleEngine MUST always produce the same annotated GIR with the same evidence trail. Rule ordering is fixed. Rule conditions are evaluated deterministically.

### 8.6 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| School-specific rule set (DSL) | Yes (semver) | The rules to apply. Each school has its own rule set. |
| Rule priority configuration | Yes | Rule ordering and conflict resolution priorities |

### 8.7 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **No rules match** | The GIR passes through unchanged. All original ambiguity is preserved. |
| **All analyses rejected** | All alternative analyses are rejected. The workflow becomes "empty" — no valid analysis. Flagged as UNRESOLVABLE_AMBIGUITY. |
| **Circular rule application** | Rules A→B→A would cause infinite loop. The RuleEngine detects that the workflow state is unchanged after a full pass and terminates. |
| **Rule not applicable** | Rules are defined broadly but may not apply to every input. Non-applicable rules are skipped silently. |
| **School-specific contradictions** | If a user configures two schools simultaneously, each school's rules are applied in order of priority. Contradictions are handled by strict_mode setting. |

### 8.8 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| RULE_SET_NOT_FOUND | School's rule set not available | Install rule set for configured school |
| RULE_VERSION_MISMATCH | Rule set version incompatible | Update rule set or platform |
| RULE_APPLICATION_LIMIT | Too many rule applications | Increase limit; optimize rule set |
| RULE_CONFLICT | Two rules contradict (strict mode) | Resolve conflict in rule set; switch schools |
| INTERNAL_ERROR | Unexpected failure | Report bug |

### 8.9 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 5K rule applications/second |
| Latency (p50) | < 500 μs per sentence |
| Latency (p99) | < 5 ms per sentence |
| Rule count limit | Support up to 10,000 rules per school |

---

## 9. MOD-08: KnowledgeGraphResolver

### 9.1 Purpose

Resolve all references in the annotated GIR against the AGOS knowledge bases. This enriches the analysis with linked linguistic data: full root definitions, wazan paradigms, dictionary entries, cross-references, and semantic information.

### 9.2 Data Schemas

#### 9.2.1 Input Schema

```
kg_input = {
  gir: annotated_gir,                         // From MOD-07
  config: {
    resolve_depth: integer,                    // How deep to resolve references (default: 3)
    enable_semantic: boolean,                  // Include semantic relations
    enable_etymology: boolean,                 // Include etymological data
    max_entries_per_reference: integer,        // Max KB entries per reference
  }
}
```

#### 9.2.2 Output Schema

```
root_entry = {
  id: string,                                  // KB-0001 entry ID
  root: string,                                // e.g., "كتب"
  meaning: string,                             // Core meaning
  forms: string[],                             // Verb forms (I-XV) with conjugations
  derived_nouns: string[],                     // Masdar, ism fa'il, ism maf'ul, etc.
  cognates: string[],                          // Related roots with similar meaning
  semantic_field: string,                      // e.g., "writing", "knowledge"
  cross_references: {                          // KB-0001 cross-references
    related_roots: string[],
    antonyms: string[],
    synonyms: string[],
  },
}

wazan_entry = {
  id: string,                                  // KB-0002 entry ID
  pattern: string,                             // e.g., "فَعَلَ"
  meaning: string,                             // Pattern meaning (e.g., "basic verb")
  form: integer,                               // Verb form number
  example: string,                             // Example word
}

resolved_token = gir_token & {
  root_entry?: root_entry,                     // Resolved root data
  wazan_entry?: wazan_entry,                   // Resolved wazan data
  dictionary_entry?: { ... },                  // Dictionary entry (if available)
  semantic_tags?: string[],                    // e.g., ["human", "action", "transitive"]
}

resolved_gir = annotated_gir & {
  tokens: resolved_token[],                    // Tokens with resolved data
  knowledge_versions: {                        // KB versions used for resolution
    "KB-0001": "1.2.3",
    "KB-0002": "2.0.1",
    ...
  },
  resolution_stats: {
    roots_resolved: integer,
    patterns_resolved: integer,
    unresolved_references: integer,
    resolution_time_ms: float,
  },
}
```

### 9.3 Processing Algorithm

```
Algorithm: resolve_knowledge_graph
Input: kg_input
Output: resolved_gir

Step 1: Load Knowledge Graph Indices
  1.1  Load all KBs (same as MOD-04).
  1.2  Build in-memory lookup index (if not already cached).

Step 2: For Each Token
  2.1  If token has a root reference:
  2.1.1  Look up root in KB-0001.
  2.1.2  If found → attach root_entry.
  2.1.3  If not found → mark as unresolved_references.

  2.2  If token has a wazan reference:
  2.2.1  Look up wazan in KB-0002.
  2.2.2  If found → attach wazan_entry.

  2.3  If token has a dictionary entry reference:
  2.3.1  Look up in dictionary (optional KB).
  2.3.2  If found → attach dictionary_entry.

  2.4  Semantic Enrichment (if enable_semantic):
  2.4.1  Look up semantic tags for the root or word.
  2.4.2  Attach semantic_tags.

  2.5  Etymological Enrichment (if enable_etymology):
  2.5.1  Look up etymological data (Arabic-only for now; future Quranic Aramaic).

Step 3: Build Resolution Statistics
  3.1  Count resolved roots, patterns, and unresolved references.

Step 4: Return
  4.1  Return resolved_gir.
```

### 9.4 Determinism

**Fully deterministic.** Given the same annotated GIR and the same KB versions, the KnowledgeGraphResolver MUST always resolve the same references and produce identical output.

### 9.5 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| KB-0001: Roots | Yes | Root definitions and semantic data |
| KB-0002: Wazan | Yes | Pattern descriptions |
| KB-0003: Verb Forms | Yes | Verb conjugation paradigms |
| KB-0004: Noun Patterns | Yes | Noun pattern descriptions |
| KB-0005: Particles | Yes | Particle meanings and usage |
| KB-0006: Pronouns | Yes | Pronoun reference data |
| KB-0007: Morphological Features | Yes | Feature value descriptions |
| Dictionary (optional) | Yes | Word definitions and translations |

### 9.6 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Root not found in KB-0001** | The token's root is marked as unresolved. Resolution continues for other tokens. |
| **Multiple roots matching** | Extremely rare (all roots in KB-0001 are unique). If occurs, all matching entries are included as ambiguity. |
| **Semantic field unknown** | Semantic tags are omitted rather than guessed. |
| **Knowledge base missing** | Resolution is skipped for that KB; metadata records which KBs were unavailable. |

### 9.7 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| KB_LOAD_FAILURE | Knowledge base cannot be loaded | Check KB file integrity and path |
| INTERNAL_ERROR | Unexpected failure | Report bug |

### 9.8 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 10K tokens/second |
| Latency (p50) | < 50 μs per token |
| KB access | O(1) average (hash-based lookup) |

---

## 10. MOD-09: BytecodeGenerator

### 10.1 Purpose

Compile the resolved GIR into executable **Grammar Bytecode** — a compact, serialized binary format that the Grammar Virtual Machine (MOD-10) can execute.

### 10.2 Data Schemas

#### 10.2.1 Input Schema

```
bytecode_input = {
  gir: resolved_gir,                           // From MOD-08
  config: {
    bytecode_version: string,                  // Target bytecode version (e.g., "1.0")
    optimization_level: 0 | 1 | 2,            // Optimization: none, basic, aggressive
    embed_text: boolean,                       // Include original text in bytecode
    embed_evidence: boolean,                   // Include evidence trail in bytecode
  }
}
```

#### 10.2.2 Output Schema

```
bytecode_header = {
  magic: bytes,                                // Magic number: "AGOS" (0x41474F53)
  version: {
    major: integer,                            // Breaking changes
    minor: integer,                            // Backward-compatible additions
    patch: integer,                            // Bug fixes
  },
  flags: bitmask,                              // Encoding flags
  bytecode_size: integer,                      // Total size in bytes
}

instruction = {
  opcode: integer,                             // Instruction opcode (0–255)
  operands: bytes[],                           // Variable-length operands
}

bytecode_section = {
  type: "header" | "metadata" | "tokens" | "morphology" | "syntax" |
        "rules" | "evidence" | "strings" | "end",
  data: bytes,
}

grammar_bytecode = {
  raw: bytes,                                  // Complete serialized bytecode
  sections: bytecode_section[],                // Logical sections (for debugging)
  size: integer,                               // Total bytecode size
  metadata: {
    input_text_hash: string,                   // SHA-256 of original text
    token_count: integer,
    tree_count: integer,
    rule_count: integer,
    compression_ratio: float,                  // Ratio of bytecode size to GIR JSON size
  }
}
```

### 10.3 Processing Algorithm

```
Algorithm: generate_bytecode
Input: bytecode_input
Output: grammar_bytecode

Step 1: Initialize
  1.1  Verify bytecode_version is supported.
  1.2  Create empty bytecode buffer.

Step 2: Emit Header
  2.1  Write magic number (4 bytes): 0x41474F53 ("AGOS").
  2.2  Write version (3 × 2-byte integers): major, minor, patch.
  2.3  Write flags bitmask (2 bytes).
  2.4  Reserve space for total size (4 bytes) — will be filled at end.

Step 3: Emit Metadata Section
  3.1  Write pipeline metadata: school, KB versions, timestamp.
  3.2  Write input text hash (SHA-256, 32 bytes).
  3.3  Write token count, tree count, rule count.

Step 4: Emit Token Section
  4.1  For each token in the GIR:
  4.1.1  Write token index (varint).
  4.1.2  Write string table reference for token text.
  4.1.3  Write morphological features as a compact bitfield.
  4.1.4  Write root and wazan IDs (varint references to KB entries).

Step 5: Emit Syntax Section
  5.1  For each parse tree:
  5.1.1  Write tree type (1 byte).
  5.1.2  Write constituent structure (serialized as depth-first traversal).
  5.1.3  Write syntactic roles for each constituent.

Step 6: Emit Rules Section
  6.1  For each rule application:
  6.1.1  Write rule ID (varint).
  6.1.2  Write rule action type (confirm/reject/modify/flag — 1 byte).
  6.1.3  Write affected token indices.

Step 7: Emit Evidence Section (if embed_evidence)
  7.1  For each evidence entry:
  7.1.1  Write stage ID, rule ID, input/output hashes.

Step 8: Emit String Table
  8.1  Collect all unique strings from the GIR.
  8.2  Write string count (varint).
  8.3  For each string: write length + UTF-8 bytes.

Step 9: Emit End Marker
  9.1  Write end-of-bytecode marker (4 bytes: 0x454E44).

Step 10: Optimize (if optimization_level > 0)
  10.1  Level 1: Deduplicate repeated instruction sequences.
  10.2  Level 2: Reorder sections for locality, apply delta encoding to indices.

Step 11: Finalize
  11.1  Fill in total size in header.
  11.2  Compute metadata (compression ratio vs. GIR JSON).
  11.3  Return grammar_bytecode.
```

### 10.4 Bytecode Format Overview

The Grammar Bytecode format is a binary container with the following structure (detailed specification in RFC-0002):

```
┌─────────────────────────┐
│ MAGIC: "AGOS" (4 bytes) │
├─────────────────────────┤
│ VERSION (6 bytes)       │
├─────────────────────────┤
│ FLAGS (2 bytes)         │
├─────────────────────────┤
│ TOTAL SIZE (4 bytes)    │
├─────────────────────────┤
│ SECTION TABLE           │
│ (start + size per sec.) │
├─────────────────────────┤
│ SECTION 1: METADATA     │
├─────────────────────────┤
│ SECTION 2: TOKENS       │
├─────────────────────────┤
│ SECTION 3: SYNTAX       │
├─────────────────────────┤
│ SECTION 4: RULES        │
├─────────────────────────┤
│ SECTION 5: EVIDENCE     │
├─────────────────────────┤
│ SECTION 6: STRINGS      │
├─────────────────────────┤
│ END MARKER (4 bytes)    │
└─────────────────────────┘
```

### 10.5 Determinism

**Fully deterministic.** Given the same resolved GIR and the same bytecode version, the BytecodeGenerator MUST always produce identical bytecode (byte-for-byte). Compression and optimization algorithms must be deterministic.

### 10.6 Knowledge Dependencies

| Dependency | Versioned | Purpose |
|------------|-----------|---------|
| GVM Instruction Set (RFC-0002) | Yes | Opcode definitions and encoding |

### 10.7 Edge Cases

| Edge Case | Behavior |
|-----------|----------|
| **Empty GIR** | Produces minimal bytecode with only header and end marker. |
| **String table overflow** | If string table exceeds 32-bit offset range, use 64-bit offsets (set flag bit). |
| **Large rule count** | All rule applications are included. If bytecode size exceeds configurable limit, emit warning. |
| **Corrupted GIR input** | BytecodeGenerator validates GIR structure before processing. Invalid GIR → error. |

### 10.8 Error Conditions

| Error Code | Trigger | Recovery |
|------------|---------|----------|
| BYTECODE_VERSION_UNSUPPORTED | Target version cannot be generated | Use supported bytecode version |
| GIR_VALIDATION_FAILED | Input GIR is structurally invalid | Debug the pipeline that produced the GIR |
| BYTECODE_TOO_LARGE | Exceeds maximum bytecode size | Simplify input; increase limit |
| INTERNAL_ERROR | Unexpected failure | Report bug |

### 10.9 Performance Targets

| Metric | Target |
|--------|--------|
| Throughput | > 5K sentences/second |
| Latency | < 1 ms per sentence |
| Bytecode size | < 10 KB per sentence (typical) |
| Compression ratio | < 20% of equivalent JSON GIR |

---

## 11. Cross-Stage Concerns

### 11.1 Pipeline Error Propagation

When a stage fails, the pipeline behavior depends on the error type:

```
Stage Error
   │
   ├── Recoverable (e.g., unknown token, partial parse)
   │     → Stage produces degraded output with error annotation
   │     → Downstream stages continue with reduced input
   │     → Final result includes error flags
   │
   └── Fatal (e.g., KB missing, invalid encoding)
         → Pipeline stops immediately
         → Error propagated to caller
         → No partial result returned
```

### 11.2 Caching Strategy

Each stage's output can be cached by the CacheManager (MOD-13):

```
Cache Key = hash(stage_input + stage_config + knowledge_versions)
Cache Value = serialized stage_output
```

Cache invalidation:
- **Knowledge version change:** All cache entries for stages that depend on the changed KB are invalidated.
- **Configuration change:** Cache entries for stages whose config has changed are invalidated.
- **TTL-based:** Optional time-based expiration for less critical caches.

### 11.3 Pipeline Configuration Shortcuts

When stages are skipped via mini-pipeline configuration, later stages that depend on the skipped stage's output must be adapted:

| Mini Pipeline | Stages Executed | Output Type |
|---------------|----------------|-------------|
| `full` | 01→02→03→04→05→06→07→08→09 | GrammarBytecode |
| `morphology-only` | 01→02→03→04 | MorphologicalAnalysis |
| `tokenization-only` | 01→02→03 | SegmentedTokenStream |
| `syntax-only` | (explicit input) 04→05 | SyntaxTree |

### 11.4 Ambiguity Propagation Rules

1. **Ambiguity is never discarded.** If MOD-03 finds 3 segmentations, MOD-04 finds 2 analyses each, and MOD-05 finds 2 parses: the GIR contains up to 12 (3×2×2) distinct paths through the ambiguity forest.

2. **Ambiguity is pruned by rules, not by heuristics.** The Rule Engine (MOD-07) may reject paths that violate grammatical rules. No other stage may prune ambiguity.

3. **Confidence ordering is advisory.** The final grammatical analysis is determined by rule application, not by confidence scores. Confidence scores are used only for ordering (display purposes) and as tiebreakers when rules cannot distinguish alternatives.

4. **Evidence trail preserves all paths.** Even rejected analyses are recorded in the evidence trail, along with the reason for rejection. This supports debugging, education, and auditing.

---

## 12. Cross-References

### 12.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | Module catalog and pipeline structure for all 9 stages |
| ADR-0001 | Compiler Architecture Rationale | Justifies the pipeline decomposition |
| SPEC-0101 | Morphology Engine | Detailed implementation of MOD-04 and MOD-05 |
| SPEC-0201 | Rule Engine | Detailed implementation of MOD-07 and rule DSL semantics |
| SPEC-0401 | Knowledge Graph Engine | Detailed implementation of MOD-08 |
| RFC-0001 | Grammar DSL | Rule authoring language consumed by MOD-07 |
| RFC-0002 | Grammar Bytecode | Bytecode format produced by MOD-09 |
| RFC-0003 | Grammar Virtual Machine | Bytecode execution model consumed by MOD-10 |
| KB-0001–0007 | Knowledge Bases | Linguistic data consumed by MOD-04 and MOD-08 |

### 12.2 External References

| Reference | Relevance |
|-----------|-----------|
| Unicode Standard, Chapter 8 (Arabic) | Character ranges, normalization, and shaping |
| Buckwalter Arabic Morphological Analyzer (BAMA) | Reference for morphological analysis approaches |
| Quranic Arabic Corpus | Reference corpus for validation and testing |
| Sibawayh's Al-Kitab | Foundational reference for Basra school grammar rules |

---

## Progress Summary

**SPEC-0001: Platform Architecture**

| Chapter | Title | Status |
|---------|-------|--------|
| Chapter 1 | Introduction and Scope | ✓ COMPLETE |
| Chapter 2 | System Architecture Overview | ✓ COMPLETE |
| **Chapter 3** | **Compilation Pipeline — Stage-by-Stage** | **✓ COMPLETE (this document)** |
| Chapter 4 | Module Responsibilities & Interfaces | Pending |
| Chapter 5 | Data Flow & Intermediate Representations | Pending |
| Chapter 6 | Deployment & Runtime Considerations | Pending |
| Chapter 7 | Extensibility & Plugin Architecture | Pending |
| Chapter 8 | Security, Validation & Error Handling | Pending |
| Chapter 9 | Performance Targets & Constraints | Pending |

**Dependencies:** Chapters 1–2, ADR-0001, KB-0001–0007, RFC-0001–0003.

**Recommended Next Chapter:** Chapter 4 — Module Responsibilities & Interfaces, which will define the formal API contracts for each module, including method signatures, data formats, error codes, and versioning policies.
