---
spec_id: SPEC-0501
title: Explanation Engine — Template System, I'rab Generation, LLM Integration & Output Formatting
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0001-C8: Security, Validation & Error Handling
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0301: Grammar Runtime — GVM & Explanation Engine
  - SPEC-0401: Knowledge Graph Engine
  - RFC-0002: Grammar Bytecode Format
  - RFC-0003: Grammar Virtual Machine
  - KB-0001: Roots
  - KB-0002: Wazan
  - KB-0003: Verb Forms
  - KB-0004: Noun Patterns
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features
  - ADR-0001: Compiler Architecture Rationale
---

# SPEC-0501: Explanation Engine — Template System, I'rab Generation, LLM Integration & Output Formatting

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Architecture Overview](#2-architecture-overview)
3. [Explanation Engine Pipeline](#3-explanation-engine-pipeline)
4. [Template System](#4-template-system)
5. [I'rab Generation Algorithms](#5-irab-generation-algorithms)
6. [Construction Identification](#6-construction-identification)
7. [Educational Levels](#7-educational-levels)
8. [LLM Prompt Engineering & Integration](#8-llm-prompt-engineering--integration)
9. [Localization Framework](#9-localization-framework)
10. [Output Format Specifications](#10-output-format-specifications)
11. [PDF Rendering](#11-pdf-rendering)
12. [Accessibility](#12-accessibility)
13. [Plugin Development Guide](#13-plugin-development-guide)
14. [Testing & Quality Assurance](#14-testing--quality-assurance)
15. [Cross-References](#15-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

This specification defines the **Explanation Engine (MOD-11)** — the component of the AGOS platform responsible for transforming deterministic grammatical analyses (`AnalysisResult`, IR-10) into human-readable, pedagogically useful explanations (`ExplanationOutput`, IR-11).

Where SPEC-0301 provides the Grammar Runtime's integrated view of MOD-11 (including its coordination with MOD-10/GVM), this specification provides the **deep-dive** into MOD-11's internal architecture: the template engine, I'rab generation algorithms, LLM prompt engineering, localization framework, output formatting, educational level adaptation, and accessibility considerations.

### 1.2 Scope

**In scope:**

| Area | Coverage |
|------|----------|
| **Template engine** | Template syntax specification, compilation pipeline, variable resolution, pluralization, conditional logic, iteration |
| **I'rab generation** | Per-sentence-type algorithms, edge case handling (ellipsis, poetic license, quranic constructions), ambiguity presentation |
| **Construction identification** | Algorithmic detection of 20+ grammatical constructions, construction composition rules |
| **Educational levels** | Pedagogical strategies for beginner (مبتدئ), intermediate (متوسط), and advanced (متقدم) learners |
| **LLM integration** | Prompt engineering, provider abstraction, response validation, caching, cost optimization |
| **Localization** | Translation workflow, pluralization rules by language, RTL text handling, locale-specific formatting |
| **Output formatting** | Complete format specifications for JSON, text, HTML, PDF with examples |
| **Accessibility** | WCAG 2.1 AA compliance, screen reader optimization, simplified text mode |
| **Plugin development** | Custom explanation plugins, template providers, LLM providers, sandbox considerations |
| **Testing** | I'rab accuracy metrics, LLM output quality evaluation, localization completeness, performance benchmarks |

**Out of scope (covered elsewhere):**

| Out of Scope | Covered By |
|-------------|------------|
| MOD-11 interface definition | SPEC-0001-C4 §13 |
| IR-11 (ExplanationOutput) schema | SPEC-0001-C5 §12 |
| Runtime caching strategy | SPEC-0301 §12.4 |
| Plugin loader (MOD-12) | SPEC-0001-C7, SPEC-0601 |
| GVM execution (MOD-10) | RFC-0003, SPEC-0301 |
| Knowledge base resolution (MOD-08) | SPEC-0401 |

### 1.3 Relationship to SPEC-0301

SPEC-0301 defines the MOD-11 component as part of the integrated Grammar Runtime, covering:
- Data structures (ExplainConfig, ExplanationOutput, IrabEntry)
- High-level architecture (Analyze → Template → Render phases)
- Basic I'rab generation (build_irab_entry, generate_token_explanation)
- Construction identification (identify_constructions with 7 constructions)
- Template system architecture (CompiledTemplate, TemplateRegistry)
- LLM integration flow and provider interface
- Output rendering (JSON, text, HTML)

**SPEC-0501 builds upon SPEC-0301 by providing:**

| Aspect | SPEC-0301 Coverage | SPEC-0501 Deep-Dive |
|--------|-------------------|---------------------|
| Template syntax | Brief enum definition | Full grammar specification, compilation pipeline, advanced constructs |
| I'rab algorithms | Single function per type | Per-sentence-type algorithms, 15+ edge cases |
| Constructions | 7 basic constructions | 20+ constructions with detection algorithms |
| Educational levels | Not covered | 3 defined levels with pedagogical strategies |
| LLM prompts | Single template | 6 prompt templates, few-shot chains, response validation |
| Localization | Language pack schema | Full translation workflow, pluralization rules by language |
| PDF rendering | Mentioned only | Complete layout specification |
| Accessibility | Not covered | WCAG 2.1 AA compliance, ARIA roles, screen reader output |
| Plugin development | ExplanationPlugin trait | Complete SDK guide, custom provider development |
| Testing | Basic categories | I'rab accuracy metrics, LLM quality evaluation framework |

### 1.4 Design Principles

1. **Analysis integrity is absolute.** The Explanation Engine operates on the completed `AnalysisResult`. It NEVER modifies, overrides, or reinterprets the deterministic analysis. Every explanation is a faithful representation of the analysis, not a re-analysis.

2. **Pedagogical value drives design.** Templates, output formats, and educational levels are designed to maximize learning outcomes. Grammatical terminology is presented alongside plain-language explanations.

3. **Template-first, LLM-enhanced.** The primary explanation pipeline uses compiled templates for speed, determinism, and reliability. LLM enhancement is additive and optional — it may augment template output but never replaces it as the baseline.

4. **Localization is first-class.** Every user-facing string is defined in language packs. The engine is language-agnostic at its core, with localization applied at the template rendering stage.

5. **No silent fallback.** If a requested language, format, or educational level is unavailable, the engine returns a structured error. It does not silently serve a different language or format.

6. **Explain by tracing.** Every explanation is grounded in the evidence trail. Users can trace any claim in the explanation back to the specific rule, algorithm, or KB entry that produced it.

### 1.5 Performance Targets (from SPEC-0001-C9)

| Metric | Target | Condition |
|--------|--------|-----------|
| **Template-based latency (p50)** | < 100 μs | 10-word sentence, JSON output |
| **Template-based latency (p99)** | < 500 μs | 30-word sentence, HTML output |
| **LLM-enhanced latency** | +500–2,000 ms | With LLM service call |
| **Throughput (template)** | > 10K sentences/s | Single core |
| **Language pack load** | < 50 ms per language | At initialization |
| **Template compile** | < 10 ms | 50 templates |
| **Memory per language pack** | ~200 KB | Full pack |

---

## 2. Architecture Overview

### 2.1 MOD-11 Internal Architecture

```
AnalysisResult (IR-10)
    │
    ▼
┌────────────────────────────────────────────────────────────────┐
│  EXPLANATION ENGINE (MOD-11)                                    │
│                                                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  LAYER 1: ANALYSIS                                       │   │
│  │                                                          │   │
│  │  Sentence Type ID ──► I'rab Generator ──► Construction   │   │
│  │  Overview Builder ──► Flag Collector ──► Evidence        │   │
│  │                                  Organizer               │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                   │
│  ┌──────────────────────────▼──────────────────────────────┐   │
│  │  LAYER 2: TEMPLATE ENGINE                                │   │
│  │                                                          │   │
│  │  Template Selector ──► Variable Resolver ──► Renderer    │   │
│  │       │                     │                    │        │   │
│  │  Language Packs ───► Variable Sources ───► Compiled      │   │
│  │  (JSON files)          (Analysis tokens,      Templates  │   │
│  │                         features, flags)                 │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                   │
│  ┌──────────────────────────▼──────────────────────────────┐   │
│  │  LAYER 3: ENHANCEMENT                                    │   │
│  │                                                          │   │
│  │  LLM Integration (optional) ──► Plugin Augmentation      │   │
│  │  • Prompt builder            • Custom explanation tokens │   │
│  │  • Provider dispatch         • Custom constructions      │   │
│  │  • Response validation       • Custom overviews          │   │
│  └──────────────────────────┬──────────────────────────────┘   │
│                             │                                   │
│  ┌──────────────────────────▼──────────────────────────────┐   │
│  │  LAYER 4: RENDERING                                      │   │
│  │                                                          │   │
│  │  JSON Renderer ──► Text Renderer ──► HTML Renderer       │   │
│  │  PDF Renderer ──► Accessibility Adapter                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                             │                                   │
└─────────────────────────────┼───────────────────────────────────┘
                              │
                              ▼
                  ExplanationOutput (IR-11)
```

### 2.2 Data Flow Through MOD-11

```
AnalysisResult (IR-10)
    │
    │  Fields consumed by MOD-11:
    │  • input_text, input_text_hash
    │  • trees[].tokens[].index, text, features (morphological, syntactic, semantic)
    │  • trees[].constituents[].role, token_ids, children
    │  • trees[].tree_type, confidence
    │  • flags[].flag_type, code, message, token_indices
    │  • evidence[].stage, rule_or_algorithm, input, output, confidence
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  STEP 1: PARSE & EXTRACT                                  │
│                                                           │
│  • Select primary analysis tree (highest confidence)      │
│  • Extract all tokens with their morphological features   │
│  • Extract syntactic roles from constituent tree          │
│  • Collect grammatical flags                              │
│  • If multiple trees exist → prepare ambiguity notes      │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  STEP 2: ANALYZE                                          │
│                                                           │
│  • Identify sentence type (verbal, nominal, etc.)         │
│  • Map each token to its I'rab attributes                 │
│  • Detect grammatical constructions from tree patterns    │
│  • Generate overview text from sentence structure         │
│  • Localize all feature values and role names             │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  STEP 3: EDUCATIONAL ADAPT                                │
│                                                           │
│  • Apply educational level filter (beginner/inter./adv.)  │
│  • Select appropriate terminology depth                  │
│  • Determine detail level for each explanation           │
│  • Add or suppress technical references                  │
│  • Adapt complexity of overview text                     │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  STEP 4: TEMPLATE                                         │
│                                                           │
│  • Select language pack for requested language            │
│  • Apply I'rab entry template to each token               │
│  • Apply construction templates                           │
│  • Apply overview template                                │
│  • Apply flag templates                                   │
│  • Apply evidence templates (if include_evidence)         │
│  • Resolve all variable references from analysis data     │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  STEP 5: ENHANCE (OPTIONAL)                                │
│                                                           │
│  • If enable_llm = true → build LLM prompt                │
│  • Send to configured provider                            │
│  • Validate response                                      │
│  • Merge LLM text with template output                    │
│  • Run explanation plugins (chain of priority order)      │
│  • Merge plugin augmentations                             │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│  STEP 6: RENDER                                           │
│                                                           │
│  • Render to requested format (JSON/text/HTML/PDF)        │
│  • Apply accessibility adaptations if needed              │
│  • Assemble final ExplanationOutput                       │
└──────────────────────────────────────────────────────────┘
    │
    ▼
ExplanationOutput (IR-11)
```

### 2.3 Configuration Model

```rust
/// Complete MOD-11 configuration.
struct ExplanationEngineConfig {
    // Core settings
    language: String,                    // Default: "en"
    format: OutputFormat,                // Default: JSON
    educational_level: EducationalLevel,  // Default: Intermediate

    // Content flags
    include_evidence: bool,              // Default: false
    include_flags: bool,                 // Default: true
    include_raw: bool,                   // Default: true
    include_ambiguity_notes: bool,       // Default: true
    include_etymology: bool,             // Default: false (requires MOD-08 data)

    // Educational settings
    terminology_style: TermStyle,        // Default: Dual (Arabic + English)
    example_format: ExampleFormat,       // Default: Inline
    max_explanation_length: u32,         // Default: 500 chars per token

    // LLM settings
    enable_llm: bool,                    // Default: false
    llm: Option<LLMConfig>,

    // Plugin settings
    plugin_ids: Vec<String>,             // Specific plugins to load (empty = all)
    plugin_chain_timeout_ms: u32,        // Default: 100 ms total for all plugins
}

enum OutputFormat { Text, Html, Json, Pdf }

enum EducationalLevel {
    Beginner,      // Simplified terms, minimal technical jargon
    Intermediate,  // Standard grammatical terminology with explanations
    Advanced,      // Full technical terminology, references to classical sources
}

enum TermStyle {
    EnglishOnly,       // "past tense verb"
    ArabicOnly,        // "فعل ماض"
    Dual,              // "past tense verb (فعل ماض)"
    ArabicFirst,       // "فعل ماضي (past tense verb)"
}

enum ExampleFormat {
    Inline,        // Examples inside explanation text
    Separate,      // Examples in a separate section
    None,          // No examples
}

struct LLMConfig {
    provider: String,                    // e.g., "openai", "anthropic", "custom"
    model: String,                       // e.g., "gpt-4", "claude-3.5-sonnet"
    temperature: f64,                    // 0.0–1.0; default: 0.3
    max_tokens: u32,                     // Default: 500
    timeout_ms: u32,                     // Default: 5000
    prompt_style: PromptStyle,           // Default: Standard
    cache_ttl_seconds: u64,              // Default: 86400 (24h)
}

enum PromptStyle {
    Standard,        // Single prompt: generate all explanations at once
    TokenByToken,    // One prompt per token (for long sentences)
    TeacherMode,     // Socratic dialogue style for educational use
    Concise,         // Brief explanations only
}
```

### 2.4 Educational Level Configuration Matrix

| Aspect | Beginner | Intermediate | Advanced |
|--------|----------|--------------|----------|
| **Terminology depth** | Plain language only ("subject" not "fa'il") | Standard terms with translations ("subject (fa'il)") | Full Arabic terminology ("فاعل") |
| **Feature detail** | 3–4 key features only | 7–8 standard features | All available features |
| **Explanation length** | 1–2 sentences per token | 2–4 sentences per token | 3–6 sentences with references |
| **Grammatical rules cited** | Never cited | Referenced by name | Full rule IDs and sources |
| **Evidence trail** | Hidden | Optional | Always included |
| **Ambiguity handling** | Hidden (best guess only) | Noted briefly | Explained with alternatives |
| **Constructions** | Named only | Named + described | Named + described + referenced |
| **Example sentences** | None | 1 per construction | 2+ per construction |
| **Classical sources** | Never cited | Cited on request | Cited routinely (Sibawayh, etc.) |

### 2.5 Parallelism & Caching

```rust
/// Explanation generation can be parallelized at the token level.
fn explain_parallel(
    result: &AnalysisResult,
    config: &ExplainConfig,
    pool: &ThreadPool,
) -> ExplanationOutput {
    let primary_tree = result.trees.first().unwrap();

    // Phase 1: Parallel I'rab generation (per token)
    let irab_entries: Vec<IrabEntry> = pool.instances()
        .map(|instance| {
            // Each instance processes one token independently
            build_irab_entry_parallel(instance, &primary_tree.tokens, config)
        })
        .collect();

    // Phase 2: Sequential (dependent on full I'rab list)
    let sentence_type = identify_sentence_type(primary_tree);
    let constructions = identify_constructions(primary_tree);
    let overview = generate_overview(&irab_entries, &sentence_type, &constructions, config);

    // Phase 3: Optional async LLM call
    let llm_enhanced = if config.enable_llm {
        spawn_async_llm_call(result, config)
    } else {
        None
    };

    // Phase 4: Render (single-threaded, fast)
    assemble_output(irab_entries, sentence_type, constructions, overview, llm_enhanced, config)
}

/// Cache key structure for explanation outputs.
/// Enables efficient reuse of explanations for identical analyses.
struct ExplanationCacheKey {
    analysis_hash: String,           // SHA-256 of AnalysisResult JSON
    language: String,
    format: OutputFormat,
    educational_level: EducationalLevel,
    include_evidence: bool,
    llm_enabled: bool,
}

// Cache hit ratio targets:
// Same analysis, same config:         > 99% (identical input → identical output)
// Same analysis, different language:  > 95% (language-independent layer cached)
// Different analysis, same sentence:  > 80% (same Arabic text, different config)
```

---

## 3. Explanation Engine Pipeline

### 3.1 Pipeline Stage Detail

The MOD-11 pipeline consists of 6 stages, each with specific responsibilities:

#### Stage 1: Input Validation

```rust
fn validate_input(result: &AnalysisResult) -> Result<(), ExplanationError> {
    // 1. Must have at least one analysis tree
    if result.trees.is_empty() {
        return Err(ExplanationError::InternalError {
            description: "AnalysisResult contains no trees".into(),
        });
    }

    // 2. Trees must be ordered by confidence (descending)
    for pair in result.trees.windows(2) {
        if pair[0].confidence < pair[1].confidence {
            return Err(ExplanationError::InternalError {
                description: "Trees not ordered by confidence".into(),
            });
        }
    }

    // 3. All tokens must have at least a POS
    for tree in &result.trees {
        for token in &tree.tokens {
            if token.features.morphological.pos.is_empty() {
                return Err(ExplanationError::InternalError {
                    description: format!("Token {} has no POS", token.index),
                });
            }
        }
    }

    Ok(())
}
```

#### Stage 2: Tree Selection & Ambiguity Analysis

```rust
struct SelectedAnalysis {
    primary_tree: AnalysisTree,
    alternative_trees: Vec<AnalysisTree>,
    ambiguity_notes: Vec<String>,
}

fn select_analysis(result: &AnalysisResult) -> SelectedAnalysis {
    let trees = &result.trees;

    // Primary tree = highest confidence
    let primary_tree = trees.first().cloned().unwrap();

    // Alternative trees = all others with confidence > 0.3
    let alternative_trees: Vec<AnalysisTree> = trees.iter()
        .skip(1)
        .filter(|t| t.confidence > 0.3)
        .cloned()
        .collect();

    // Generate ambiguity notes
    let mut ambiguity_notes = Vec::new();
    if !alternative_trees.is_empty() {
        ambiguity_notes.push(format!(
            "This sentence has {} alternative grammatical interpretation(s).",
            alternative_trees.len()
        ));
        for (i, alt) in alternative_trees.iter().enumerate() {
            let diff = primary_tree.confidence - alt.confidence;
            ambiguity_notes.push(format!(
                "Alternative {} (confidence: {:.0}%, {:.0}% lower than primary): {}",
                i + 1,
                alt.confidence * 100.0,
                diff * 100.0,
                describe_difference(&primary_tree, alt),
            ));
        }
    }

    SelectedAnalysis { primary_tree, alternative_trees, ambiguity_notes }
}

/// Describe the key difference between two analyses.
fn describe_difference(primary: &AnalysisTree, alternative: &AnalysisTree) -> String {
    // Compare sentence types
    if primary.tree_type != alternative.tree_type {
        return format!("different sentence type: {} vs {}", primary.tree_type, alternative.tree_type);
    }

    // Compare syntactic roles for the first differing token
    for (pt, at) in primary.tokens.iter().zip(alternative.tokens.iter()) {
        if pt.features.syntactic.role != at.features.syntactic.role {
            return format!(
                "different analysis of '{}': {} vs {}",
                pt.text,
                pt.features.syntactic.role.as_deref().unwrap_or("unknown"),
                at.features.syntactic.role.as_deref().unwrap_or("unknown"),
            );
        }
    }

    "minor differences in feature assignments".into()
}
```

#### Stage 3: Feature Extraction & Organization

```rust
/// Organized feature set for a single token.
struct OrganizedFeatures {
    /// Inflectional features (gender, number, person, tense, mood, voice, case, state)
    inflectional: Vec<FeatureDisplay>,

    /// Derivational features (verb_form, noun_type, transitivity, root_type)
    derivational: Vec<FeatureDisplay>,

    /// Semantic features (semantic tags, definition, root meaning)
    semantic: Vec<FeatureDisplay>,

    /// Prosodic features (stress, syllable count)
    prosodic: Vec<FeatureDisplay>,

    /// Orthographic features (has_shadda, has_hamza, etc.)
    orthographic: Vec<FeatureDisplay>,
}

fn organize_features(token: &AnalysisToken, language_pack: &LanguagePack) -> OrganizedFeatures {
    let mf = &token.features.morphological;
    let mut organized = OrganizedFeatures {
        inflectional: Vec::new(),
        derivational: Vec::new(),
        semantic: Vec::new(),
        prosodic: Vec::new(),
        orthographic: Vec::new(),
    };

    // Inflectional features (always shown)
    push_if_some(&mut organized.inflectional, "gender", &mf.gender, language_pack);
    push_if_some(&mut organized.inflectional, "number", &mf.number, language_pack);
    push_if_some(&mut organized.inflectional, "person", &mf.person, language_pack);
    push_if_some(&mut organized.inflectional, "tense", &mf.tense, language_pack);
    push_if_some(&mut organized.inflectional, "mood", &mf.mood, language_pack);
    push_if_some(&mut organized.inflectional, "voice", &mf.voice, language_pack);
    push_if_some(&mut organized.inflectional, "case", &mf.case, language_pack);
    push_if_some(&mut organized.inflectional, "state", &mf.state, language_pack);

    // Derivational features (shown at higher educational levels)
    if let Some(form) = &mf.verb_form {
        organized.derivational.push(FeatureDisplay {
            name: language_pack.feature_name("verb_form"),
            value: format!("Form {}", roman_numeral(*form)),
            category: "derivational".into(),
        });
    }
    push_if_some(&mut organized.derivational, "noun_type", &mf.noun_type, language_pack);
    push_if_some(&mut organized.derivational, "transitivity", &mf.transitivity, language_pack);

    // Semantic features
    let sf = &token.features.semantic;
    if !sf.tags.is_empty() {
        organized.semantic.push(FeatureDisplay {
            name: language_pack.feature_name("semantic_tags"),
            value: sf.tags.join(", "),
            category: "semantic".into(),
        });
    }
    if let Some(def) = &sf.definition {
        organized.semantic.push(FeatureDisplay {
            name: language_pack.feature_name("definition"),
            value: def.clone(),
            category: "semantic".into(),
        });
    }
    if let Some(rm) = &sf.root_meaning {
        organized.semantic.push(FeatureDisplay {
            name: language_pack.feature_name("root_meaning"),
            value: rm.clone(),
            category: "semantic".into(),
        });
    }

    organized
}

fn push_if_some(
    features: &mut Vec<FeatureDisplay>,
    name: &str,
    value: &Option<String>,
    language_pack: &LanguagePack,
) {
    if let Some(v) = value {
        features.push(FeatureDisplay {
            name: language_pack.feature_name(name),
            value: language_pack.feature_value(name, v),
            category: "inflectional".into(),
        });
    }
}
```

#### Stage 4: Educational Level Filter

```rust
/// Apply educational level filtering to features.
fn filter_features_by_level(
    organized: &OrganizedFeatures,
    level: EducationalLevel,
) -> Vec<FeatureDisplay> {
    match level {
        EducationalLevel::Beginner => {
            // Only show gender, number, tense for verbs; case + state for nouns
            let mut filtered = Vec::new();
            for f in &organized.inflectional {
                if matches!(f.name.as_str(), "Gender" | "Number" | "Tense" | "Person" | "Case" | "State") {
                    filtered.push(f.clone());
                }
            }
            filtered
        }
        EducationalLevel::Intermediate => {
            // Show all inflectional + key derivational
            let mut filtered = organized.inflectional.clone();
            for f in &organized.derivational {
                if matches!(f.name.as_str(), "Verb Form" | "Noun Type") {
                    filtered.push(f.clone());
                }
            }
            filtered
        }
        EducationalLevel::Advanced => {
            // Show everything
            let mut filtered = Vec::new();
            filtered.extend(organized.inflectional.clone());
            filtered.extend(organized.derivational.clone());
            filtered.extend(organized.semantic.clone());
            filtered
        }
    }
}
```

#### Stage 5: Evidence Organizer

```rust
/// Organize evidence by stage for structured display.
struct OrganizedEvidence {
    by_stage: Vec<StageEvidence>,
    total_entries: usize,
}

struct StageEvidence {
    stage_id: String,                        // e.g., "MOD-04"
    stage_name: String,                      // e.g., "Morphological Analysis"
    entries: Vec<EvidenceEntry>,
}

fn organize_evidence(
    evidence: &[EvidenceEntry],
    language_pack: &LanguagePack,
) -> OrganizedEvidence {
    let mut by_stage: HashMap<String, Vec<EvidenceEntry>> = HashMap::new();
    for entry in evidence {
        by_stage.entry(entry.stage.clone()).or_default().push(entry.clone());
    }

    let stage_order = [
        ("MOD-03", "Tokenization"),
        ("MOD-04", "Morphological Analysis"),
        ("MOD-05", "Syntactic Parsing"),
        ("MOD-06", "GIR Construction"),
        ("MOD-07", "Rule Application"),
        ("MOD-08", "Knowledge Graph Resolution"),
        ("MOD-09", "Bytecode Generation"),
        ("MOD-10", "GVM Execution"),
    ];

    let by_stage: Vec<StageEvidence> = stage_order.iter()
        .filter_map(|(id, name)| {
            by_stage.remove(*id).map(|entries| StageEvidence {
                stage_id: id.to_string(),
                stage_name: language_pack.stage_name(name),
                entries,
            })
        })
        .collect();

    let total_entries = evidence.len();

    OrganizedEvidence { by_stage, total_entries }
}
```

### 3.2 Error Handling

```rust
enum ExplanationError {
    UnsupportedLanguage { language: String, supported: Vec<String> },
    UnsupportedFormat { format: String, supported: Vec<String> },
    UnsupportedLevel { level: String, supported: Vec<String> },
    EmptyAnalysis,
    LLMServiceUnavailable { provider: String, reason: String },
    LLMInvalidResponse { provider: String, details: String },
    TemplateError { template_name: String, details: String },
    LanguagePackMissing { language: String },
    PluginError { plugin_id: String, details: String },
    InternalError { description: String },
}

impl ExplanationError {
    fn severity(&self) -> &str {
        match self {
            Self::LLMServiceUnavailable { .. } => "warning",
            Self::PluginError { .. } => "warning",
            _ => "error",
        }
    }

    fn is_fatal(&self) -> bool {
        matches!(self,
            Self::UnsupportedLanguage { .. } |
            Self::UnsupportedFormat { .. } |
            Self::UnsupportedLevel { .. } |
            Self::EmptyAnalysis |
            Self::LanguagePackMissing { .. } |
            Self::InternalError { .. }
        )
    }

    fn recovery_hint(&self) -> Option<String> {
        match self {
            Self::UnsupportedLanguage { supported, .. } =>
                Some(format!("Supported languages: {}", supported.join(", "))),
            Self::UnsupportedFormat { supported, .. } =>
                Some(format!("Supported formats: {}", supported.join(", "))),
            Self::LLMServiceUnavailable { .. } =>
                Some("Template-based explanation will be used instead.".into()),
            Self::TemplateError { template_name, .. } =>
                Some(format!("Check template '{}' for syntax errors.", template_name)),
            Self::LanguagePackMissing { language } =>
                Some(format!("Install the language pack for '{}' or use 'en' as fallback.", language)),
            _ => None,
        }
    }
}
```

---

## 4. Template System

### 4.1 Template Syntax Specification

The AGOS template language is a lightweight, domain-specific notation for generating localized explanations. It is designed for ease of authoring by linguists and translators without requiring programming knowledge.

#### 4.1.1 Variable Substitution

```
Syntax:     {variable_name}
Example:    "This {pos} is in the {case} case."
Resolves:   "This Noun is in the Genitive case."
```

| Variable Source | Available Variables |
|-----------------|-------------------|
| Token features | `{token}`, `{root}`, `{pos}`, `{gender}`, `{number}`, `{person}`, `{tense}`, `{mood}`, `{voice}`, `{case}`, `{state}`, `{verb_form}`, `{noun_type}`, `{transitivity}` |
| Syntactic role | `{role}`, `{governor}`, `{role_arabic}` |
| Semantic data | `{definition}`, `{root_meaning}`, `{semantic_tags}` |
| Sentence context | `{sentence_type}`, `{word_count}`, `{tree_count}` |
| Construction | `{construction_name}`, `{construction_description}`, `{involved_tokens}` |
| Flag | `{flag_type}`, `{flag_code}`, `{flag_message}`, `{flag_tokens}` |
| Metadata | `{language}`, `{format}`, `{level}`, `{generation_time}` |

#### 4.1.2 Conditional Blocks

```
Syntax:     {?variable}content{/variable}
            {?variable=value}content{/variable}
            {?variable}content{:otherwise}content{/variable}

Examples:
    {?root}The root of this word is {root}.{/root}
    {?case=nominative}This word is in the nominative case.{/case}
    {?tense}This verb is in the {tense} tense.{:tense}This word is not a verb.{/tense}
```

#### 4.1.3 Pluralization

```
Syntax:     {count:plural(one|other)}
            {count:plural(one|two|few|many|other)}

Examples:
    "This sentence has {word_count:plural(1 word|{} words)}."
    → "This sentence has 1 word."
    → "This sentence has 5 words."

    Arabic dual support:
    "{token_count:plural(كلمة واحدة|كلمتان|{} كلمات)}"
    → "كلمة واحدة" (1 word)
    → "كلمتان" (2 words)
    → "٥ كلمات" (5 words)
```

**Supported plural forms per language:**

| Language | Plural Forms | Rule |
|----------|-------------|------|
| English | one, other | Standard Germanic |
| Arabic | one, two, few, many, other | 6-form system |
| Urdu | one, other | Similar to Hindi |
| Malay/Indonesian | other | No plural marking |
| French | one, other | Similar to English |
| Turkish | one, other | No plural marking |
| Persian/Farsi | one, other | Similar to English |

#### 4.1.4 Iteration

```
Syntax:     {#list_variable}item template{/list_variable}
            {#list_variable}item template{:separator}separator{/list_variable}

Examples:
    "Features: {#features}{value}{:separator}, {/features}"
    → "Features: Masculine, Singular, Third Person"

    "Multiple analyses exist: {#alternatives}{description}{:separator} OR {/alternatives}"
```

#### 4.1.5 Variable Formatting

```
Syntax:     {variable:format}
            {variable:format(arg1|arg2)}

Formatters:
    {root:arabic}          → Renders root in Arabic script if available
    {case:abbreviated}     → "Nom." instead of "Nominative"
    {case:arabic}          → "الرفع" instead of "Nominative"
    {number:ordinal}       → "1st", "2nd", "3rd"
    {number:roman}         → "I", "II", "III" (for verb forms)
    {number:arabic_numerals} → "١", "٢", "٣"
    {tense:full}           → "Past Tense (الماضي)"
    {confidence:percent}   → "95%"
    {text:capitalize}      → Capitalize first letter
    {text:lowercase}       → Lowercase
    {text:truncate(N)}     → Truncate to N characters
```

#### 4.1.6 Comments

```
Syntax:     {# This is a comment and will not appear in output #}
```

#### 4.1.7 Template Inheritance

```
Syntax:     {extends "parent_template"}
            {block name}content{/block}

Example:
    {extends "irab_base"}
    {block token_explanation}
        A {gender} {pos} in the {case} case.
    {/block}
```

### 4.2 Template Compilation Pipeline

```
Raw Template String
    │
    ▼
┌────────────────────────────────────────────┐
│  1. LEXER                                  │
│     • Tokenize into: Text, Variable,       │
│       Conditional, Plural, Iteration,      │
│       Comment, Extends, Block tokens       │
│     • Error on unclosed brackets           │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  2. PARSER                                 │
│     • Build AST from token stream          │
│     • Validate nesting (conditionals,      │
│       iterations must be properly closed)  │
│     • Validate block inheritance           │
│     • Error on unknown variables           │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  3. SEMANTIC ANALYZER                      │
│     • Resolve template inheritance         │
│     • Merge parent and child blocks        │
│     • Verify variable names against        │
│       available variable sources           │
│     • Verify plural forms match target     │
│       language's plural system             │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  4. OPTIMIZER                              │
│     • Constant-fold static text segments   │
│     • Inline simple variable references    │
│     • Pre-compute formatter dispatch       │
│     • Eliminate unreachable branches       │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  5. CODE GENERATOR                         │
│     • Emit CompiledTemplate segment list   │
│     • Each segment is one of:              │
│       - TextSegment(&str)                  │
│       - VariableSegment(name, formatter)   │
│       - ConditionalSegment(var, eq, then, else)│
│       - PluralSegment(count, forms[])      │
│       - IterationSegment(list, template, sep)│
│     • Segments are linear for O(n) render  │
└────────────────────────────────────────────┘
    │
    ▼
CompiledTemplate (ready for rendering)
```

### 4.3 Template Registry

```rust
/// Manages all compiled templates for the Explanation Engine.
struct TemplateRegistry {
    /// Template name → compiled template
    templates: HashMap<String, CompiledTemplate>,

    /// Template inheritance graph
    inheritance: HashMap<String, String>,  // Child → Parent

    /// Fallback language for missing templates
    fallback_language: String,

    /// Default template set (always loaded)
    default_templates: HashMap<String, CompiledTemplate>,
}

impl TemplateRegistry {
    /// Register a template from source string.
    fn register(&mut self, name: &str, source: &str) -> Result<(), TemplateError> {
        let compiled = self.compile(source)?;
        self.templates.insert(name.to_string(), compiled);
        Ok(())
    }

    /// Load all templates from a directory.
    fn load_from_directory(&mut self, path: &Path) -> Result<(), TemplateError> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |e| e == "agos-tpl") {
                let source = fs::read_to_string(entry.path())?;
                let name = entry.path().file_stem().unwrap().to_str().unwrap();
                self.register(name, &source)?;
            }
        }
        Ok(())
    }

    /// Get a compiled template by name.
    /// Falls back to default template set if not found.
    fn get(&self, name: &str) -> Option<&CompiledTemplate> {
        self.templates.get(name)
            .or_else(|| self.default_templates.get(name))
    }

    /// Render a template with given variables.
    fn render(
        &self,
        name: &str,
        vars: &HashMap<String, TemplateValue>,
        language: &str,
    ) -> Result<String, TemplateError> {
        let template = self.get(name)
            .ok_or(TemplateError::NotFound(name.to_string()))?;
        self.render_template(template, vars, language)
    }

    fn compile(&self, source: &str) -> Result<CompiledTemplate, TemplateError> {
        let tokens = lex_template(source)?;
        let ast = parse_template(tokens)?;
        let resolved = resolve_inheritance(ast, &self.inheritance)?;
        let optimized = optimize_template(resolved);
        Ok(generate_code(optimized))
    }
}
```

### 4.4 Required Template Names

The following template names MUST be provided in every language pack:

| Template Name | Purpose | Variables Available |
|--------------|---------|-------------------|
| `irab_entry` | Single I'rab entry | `token`, `root`, `pos`, `features`, `role`, `explanation` |
| `irab_header` | I'rab section header | — |
| `irab_empty` | Empty/unknown token | `token` |
| `overview` | Sentence overview | `sentence_type`, `word_count`, `construction_names`, `has_ambiguity` |
| `sentence_type_verbal` | Verbal sentence type | — |
| `sentence_type_nominal` | Nominal sentence type | — |
| `sentence_type_conditional` | Conditional sentence type | — |
| `sentence_type_kana` | Kana sentence type | — |
| `sentence_type_inna` | Inna sentence type | — |
| `sentence_type_oath` | Oath sentence type | — |
| `sentence_type_unknown` | Unknown sentence type | — |
| `construction_idafa` | Construct state | `tokens`, `mudaf`, `mudaf_ilayh` |
| `construction_naat` | Adjective agreement | `tokens`, `described_noun`, `adjective` |
| `construction_tawkid` | Emphasis | `tokens`, `emphasized`, `emphasizer` |
| `construction_badal` | Apposition | `tokens`, `first`, `second` |
| `construction_istithna` | Exception | `tokens`, `excepted_from`, `excepting_word` |
| `construction_nida` | Vocative | `tokens`, `vocative_particle`, `addressed` |
| `construction_shart` | Conditional | `tokens`, `condition`, `result` |
| `construction_kana` | Kana and sisters | `tokens`, `kana_verb`, `subject`, `predicate` |
| `construction_inna` | Inna and sisters | `tokens`, `inna_particle`, `subject`, `predicate` |
| `construction_hal` | Circumstantial clause | `tokens`, `subject`, `hal_phrase` |
| `construction_tamyiz` | Specification | `tokens`, `specified`, `specifier` |
| `construction_idafah_fi'liyyah` | Verbal idafa | `tokens` |
| `construction_qasam` | Oath | `tokens`, `oath_particle`, `oath_phrase`, `answer` |
| `flag_error` | Error flag | `code`, `message`, `tokens` |
| `flag_warning` | Warning flag | `code`, `message`, `tokens` |
| `flag_info` | Info flag | `code`, `message`, `tokens` |
| `evidence_stage` | Evidence for a stage | `stage_name`, `entries` |
| `evidence_entry` | Single evidence entry | `rule`, `input`, `output`, `confidence` |
| `ambiguity_note` | Ambiguity note | `description`, `difference` |
| `llm_notice` | LLM enhancement notice | — |
| `footer` | Output footer | `format`, `language`, `generation_time` |

### 4.5 Template Variable Resolution

```rust
/// Source of variable values for template rendering.
struct VariableSource {
    /// Token-level variables
    token_vars: HashMap<String, TemplateValue>,

    /// Sentence-level variables
    sentence_vars: HashMap<String, TemplateValue>,

    /// Construction-level variables
    construction_vars: HashMap<String, TemplateValue>,

    /// Flag-level variables
    flag_vars: HashMap<String, TemplateValue>,

    /// Global variables
    global_vars: HashMap<String, TemplateValue>,
}

impl VariableSource {
    /// Build variable source from analysis data.
    fn from_analysis(result: &AnalysisResult, tree: &AnalysisTree, config: &ExplainConfig) -> Self {
        let mut global = HashMap::new();
        global.insert("language".into(), TemplateValue::String(config.language.clone()));
        global.insert("format".into(), TemplateValue::String(format!("{:?}", config.format)));
        global.insert("generation_time".into(), TemplateValue::String(now_iso8601()));

        // Build sentence-level variables
        let mut sentence = HashMap::new();
        sentence.insert("input_text".into(), TemplateValue::String(result.input_text.clone()));
        sentence.insert("tree_count".into(), TemplateValue::Number(result.trees.len() as i64));
        sentence.insert("sentence_type".into(), TemplateValue::String(tree.tree_type.clone()));

        VariableSource {
            token_vars: HashMap::new(),
            sentence_vars: sentence,
            construction_vars: HashMap::new(),
            flag_vars: HashMap::new(),
            global_vars: global,
        }
    }

    /// Resolve a single variable reference.
    fn resolve(&self, name: &str, formatter: Option<&str>) -> Option<TemplateValue> {
        // Check in order: token → sentence → construction → flag → global
        if let Some(val) = self.token_vars.get(name) {
            return Some(self.apply_formatter(val, formatter));
        }
        if let Some(val) = self.sentence_vars.get(name) {
            return Some(self.apply_formatter(val, formatter));
        }
        if let Some(val) = self.construction_vars.get(name) {
            return Some(self.apply_formatter(val, formatter));
        }
        if let Some(val) = self.flag_vars.get(name) {
            return Some(self.apply_formatter(val, formatter));
        }
        if let Some(val) = self.global_vars.get(name) {
            return Some(self.apply_formatter(val, formatter));
        }
        None
    }

    fn apply_formatter(&self, value: &TemplateValue, formatter: Option<&str>) -> TemplateValue {
        match (value, formatter) {
            (TemplateValue::String(s), Some("capitalize")) => {
                let mut c = s.chars();
                TemplateValue::String(c.next().map(|f| f.to_uppercase().to_string() + c.as_str()).unwrap_or_default())
            }
            (TemplateValue::String(s), Some("lowercase")) => {
                TemplateValue::String(s.to_lowercase())
            }
            (TemplateValue::Number(n), Some("roman")) => {
                TemplateValue::String(roman_numeral(*n as u8))
            }
            (TemplateValue::Number(n), Some("percent")) => {
                TemplateValue::String(format!("{:.0}%", n * 100.0))
            }
            (TemplateValue::String(s), Some(fmt)) if fmt.starts_with("truncate(") => {
                // parse truncate(N)
                if let Some(len) = fmt.trim_start_matches("truncate(").trim_end_matches(")").parse::<usize>().ok() {
                    let truncated: String = s.chars().take(len).collect();
                    TemplateValue::String(if s.len() > len { format!("{}...", truncated) } else { s.clone() })
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        }
    }
}

enum TemplateValue {
    String(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    List(Vec<TemplateValue>),
    Null,
}
```

### 4.6 Template Rendering Performance

| Template | Compile Time | Render Time (10 words) | Render Time (30 words) |
|----------|-------------|----------------------|----------------------|
| `irab_entry` | < 50 μs | < 2 μs per token | < 2 μs per token |
| `overview` | < 30 μs | < 5 μs | < 5 μs |
| `construction_*` | < 40 μs each | < 3 μs each | < 3 μs each |
| `flag_*` | < 20 μs each | < 1 μs each | < 1 μs each |
| All templates combined | < 3 ms total | < 50 μs total | < 150 μs total |

---

## 5. I'rab Generation Algorithms

### 5.1 Sentence Type Identification

```rust
/// Identify the sentence type with subtype detection.
enum SentenceType {
    /// Verbal sentence (جملة فعلية) — begins with a verb
    Verbal {
        verb_type: VerbType,       // past, present, imperative
        verb_transitivity: Option<String>,
    },
    /// Nominal sentence (جملة اسمية) — begins with a noun or pronoun
    Nominal {
        subject_type: SubjectType, // definite noun, indefinite, pronoun
        predicate_type: PredicateType,
    },
    /// Conditional sentence (جملة شرطية)
    Conditional {
        particle: String,          // إن, لو, لولا, etc.
        has_result: bool,
    },
    /// Kana and her sisters
    Kana {
        kana_verb: String,         // كان, أصبح, ظل, etc.
    },
    /// Inna and her sisters
    Inna {
        inna_particle: String,     // إن, أن, لكن, etc.
    },
    /// Oath construction (جملة قسم)
    Oath {
        particle: String,          // و, ت, ب
    },
    /// Adverbial clause (جملة ظرفية)
    Adverbial,
    /// Exceptive sentence (جملة استثنائية)
    Exceptive {
        exceptive_word: String,    // إلا, غير, سوى
    },
    /// Elliptical sentence (hadhf — omission)
    Elliptical {
        omitted_elements: Vec<String>,
    },
    /// Imperative / command
    Imperative,
    /// Interrogative
    Interrogative {
        particle: Option<String>,  // هل, أ, etc.
    },
    /// Exclamation / wonder
    Exclamatory,
    /// Incomplete (fragment)
    Incomplete,
    /// Unknown
    Unknown,
}

fn identify_sentence_type(tree: &AnalysisTree) -> SentenceType {
    // Check tree type first
    match tree.tree_type.as_str() {
        "jumlah_fi'liyyah" => {
            // Find the verb token
            if let Some(verb_token) = tree.tokens.iter()
                .find(|t| t.features.syntactic.role.as_deref() == Some("fi'l"))
            {
                let tense = verb_token.features.morphological.tense.as_deref().unwrap_or("past");
                let verb_type = match tense {
                    "past" => VerbType::Past,
                    "present" => VerbType::Present,
                    "imperative" => VerbType::Imperative,
                    _ => VerbType::Past,
                };
                SentenceType::Verbal {
                    verb_type,
                    verb_transitivity: verb_token.features.morphological.transitivity.clone(),
                }
            } else {
                SentenceType::Verbal { verb_type: VerbType::Past, verb_transitivity: None }
            }
        }
        "jumlah_ismiyyah" => {
            let subject_token = tree.tokens.iter()
                .find(|t| t.features.syntactic.role.as_deref() == Some("mubtada"));
            let predicate_token = tree.tokens.iter()
                .find(|t| t.features.syntactic.role.as_deref() == Some("khabar"));

            let subject_type = subject_token.map(|t| {
                match t.features.morphological.state.as_deref() {
                    Some("definite") => SubjectType::DefiniteNoun,
                    Some("indefinite") => SubjectType::IndefiniteNoun,
                    _ => SubjectType::Pronoun,
                }
            }).unwrap_or(SubjectType::DefiniteNoun);

            let predicate_type = predicate_token.map(|t| {
                match t.features.morphological.pos.as_str() {
                    "noun" => PredicateType::Noun,
                    "verb" => PredicateType::Verbal,
                    "particle" => PredicateType::Particle,
                    "preposition" => PredicateType::PrepositionalPhrase,
                    _ => PredicateType::Noun,
                }
            }).unwrap_or(PredicateType::Noun);

            SentenceType::Nominal { subject_type, predicate_type }
        }
        "jumlah_shartiyyah" => {
            let particle = tree.tokens.iter()
                .find(|t| matches!(t.features.morphological.pos.as_str(), "particle"))
                .map(|t| t.text.clone())
                .unwrap_or_default();
            let has_result = tree.tokens.iter()
                .any(|t| t.features.syntactic.role.as_deref() == Some("jaza"));
            SentenceType::Conditional { particle, has_result }
        }
        "jumlah_kana" => {
            let kana = tree.tokens.iter()
                .find(|t| t.features.syntactic.role.as_deref() == Some("fi'l"))
                .map(|t| t.text.clone())
                .unwrap_or("كان".into());
            SentenceType::Kana { kana_verb: kana }
        }
        "jumlah_inna" => {
            let inna = tree.tokens.iter()
                .find(|t| matches!(t.features.morphological.pos.as_str(), "particle"))
                .map(|t| t.text.clone())
                .unwrap_or("إن".into());
            SentenceType::Inna { inna_particle: inna }
        }
        "jumlah_qasam" => SentenceType::Oath {
            particle: tree.tokens.first()
                .map(|t| t.text.clone())
                .unwrap_or_default(),
        },
        "jumlah_zarfiyyah" => SentenceType::Adverbial,
        "jumlah_istithna" => SentenceType::Exceptive {
            exceptive_word: tree.tokens.iter()
                .find(|t| t.text == "إلا" || t.text == "غير" || t.text == "سوى")
                .map(|t| t.text.clone())
                .unwrap_or("إلا".into()),
        },
        _ => {
            // Heuristic detection from tree structure
            detect_implicit_type(tree)
        }
    }
}

fn detect_implicit_type(tree: &AnalysisTree) -> SentenceType {
    let first_token = tree.tokens.first();
    let has_verb = tree.tokens.iter().any(|t| t.features.morphological.pos == "verb");
    let has_interrog = tree.tokens.iter().any(|t| t.features.morphological.pos == "interrogative");
    let has_command = tree.tokens.iter().any(|t| t.features.morphological.mood.as_deref() == Some("imperative"));
    let has_omission = tree.constituents.iter().any(|c| c.implicit);

    if has_omission {
        let omitted: Vec<String> = tree.constituents.iter()
            .filter(|c| c.implicit)
            .map(|c| c.role.clone())
            .collect();
        return SentenceType::Elliptical { omitted_elements: omitted };
    }
    if has_interrog {
        return SentenceType::Interrogative { particle: None };
    }
    if has_command {
        return SentenceType::Imperative;
    }
    if !has_verb && first_token.map_or(false, |t| t.features.morphological.pos == "noun") {
        // Default to nominal if starts with a noun
        return SentenceType::Nominal {
            subject_type: SubjectType::DefiniteNoun,
            predicate_type: PredicateType::Noun,
        };
    }
    if has_verb {
        return SentenceType::Verbal { verb_type: VerbType::Past, verb_transitivity: None };
    }

    SentenceType::Unknown
}
```

### 5.2 Token Explanation Templates by POS

Each POS and syntactic role combination generates a specific explanation pattern:

```rust
/// Generate explanation for a token based on its POS and role.
/// This is the core I'rab generation function.
fn generate_irab_explanation(
    token: &AnalysisToken,
    sentence_type: &SentenceType,
    features: &OrganizedFeatures,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    let mf = &token.features.morphological;
    let sf = &token.features.syntactic;
    let pos = mf.pos.as_str();
    let role = sf.role.as_deref();

    match (pos, role) {
        // === VERB PATTERNS ===
        ("verb", Some("fi'l")) => explain_verb(token, mf, sf, sentence_type, language_pack, level),
        ("verb", Some("khabar")) => explain_verb_as_predicate(token, mf, sf, language_pack, level),
        ("verb", Some("hal")) => explain_verb_as_hal(token, mf, sf, language_pack, level),
        ("verb", Some("na'at")) => explain_verb_as_adjective(token, mf, sf, language_pack, level),
        ("verb", Some("fi'l_shart")) => explain_conditional_verb(token, mf, sf, language_pack, level, "condition"),
        ("verb", Some("jaza")) => explain_conditional_verb(token, mf, sf, language_pack, level, "result"),

        // === NOUN PATTERNS ===
        ("noun", Some("fa'il")) => explain_subject(token, mf, sf, language_pack, level),
        ("noun", Some("mubtada")) => explain_mubtada(token, mf, sf, sentence_type, language_pack, level),
        ("noun", Some("khabar")) => explain_khabar(token, mf, sf, sentence_type, language_pack, level),
        ("noun", Some("maf'ul_bi-hi")) => explain_object(token, mf, sf, language_pack, level),
        ("noun", Some("mudaf")) => explain_mudaf(token, mf, sf, language_pack, level),
        ("noun", Some("mudaf_ilayh")) => explain_mudaf_ilayh(token, mf, sf, language_pack, level),
        ("noun", Some("na'at")) => explain_adjective(token, mf, sf, language_pack, level),
        ("noun", Some("hal")) => explain_hal(token, mf, sf, language_pack, level),
        ("noun", Some("tamyiz")) => explain_tamyiz(token, mf, sf, language_pack, level),
        ("noun", Some("zarf")) => explain_zarf(token, mf, sf, language_pack, level),
        ("noun", Some("badal")) => explain_badal(token, mf, sf, language_pack, level),
        ("noun", Some("ta'kid")) => explain_tawkid(token, mf, sf, language_pack, level),
        ("noun", Some("majrur")) => explain_majrur(token, mf, sf, language_pack, level),
        ("noun", Some("mastar")) => explain_masdar(token, mf, sf, language_pack, level),

        // === PARTICLE / PREPOSITION PATTERNS ===
        ("particle", Some("harf_jarr")) => explain_preposition(token, mf, sf, language_pack, level),
        ("particle", Some("harf_nasb")) => explain_subjunctive_particle(token, mf, sf, language_pack, level),
        ("particle", Some("harf_jazm")) => explain_jussive_particle(token, mf, sf, language_pack, level),
        ("particle", Some("harf_istifham")) => explain_interrogative_particle(token, mf, sf, language_pack, level),
        ("particle", Some("harf_shart")) => explain_conditional_particle(token, mf, sf, language_pack, level),
        ("particle", Some("harf_nida")) => explain_vocative_particle(token, mf, sf, language_pack, level),
        ("particle", _) => explain_generic_particle(token, mf, sf, language_pack, level),

        // === PRONOUN PATTERNS ===
        ("pronoun", Some(role_str)) => explain_pronoun(token, mf, sf, role_str, language_pack, level),
        ("pronoun", None) => explain_pronoun(token, mf, sf, "unknown", language_pack, level),

        // === GENERIC / FALLBACK ===
        _ => explain_generic(token, mf, sf, language_pack, level),
    }
}
```

### 5.3 Verb Explanation Patterns

```rust
fn explain_verb(
    token: &AnalysisToken,
    mf: &MorphologicalFeatures,
    sf: &SyntacticFeatures,
    sentence_type: &SentenceType,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    let tense = feature_value(mf.tense.as_deref(), "past", language_pack, "tense");
    let voice = feature_value(mf.voice.as_deref(), "active", language_pack, "voice");
    let person = feature_value(mf.person.as_deref(), "third", language_pack, "person");
    let gender = feature_value(mf.gender.as_deref(), "masculine", language_pack, "gender");
    let number = feature_value(mf.number.as_deref(), "singular", language_pack, "number");

    match level {
        EducationalLevel::Beginner => {
            format!(
                "A {} verb. It is in the {} tense, {} voice.",
                person, tense, voice,
            )
        }
        EducationalLevel::Intermediate => {
            let form = mf.verb_form
                .map(|f| format!(", Form {}", roman_numeral(f)))
                .unwrap_or_default();
            let trans = mf.transitivity.as_ref()
                .map(|t| format!(", {}", language_pack.feature_value("transitivity", t)))
                .unwrap_or_default();

            match sentence_type {
                SentenceType::Verbal { .. } => {
                    // Find fa'il (subject) agreement
                    let subject_agreement = explain_subject_agreement(mf, language_pack);
                    format!(
                        "A {person} {tense} verb in the {voice} voice{form}{trans}. \
                         {subject_agreement}",
                        person = person, tense = tense, voice = voice,
                        form = form, trans = trans,
                        subject_agreement = subject_agreement,
                    )
                }
                _ => {
                    format!(
                        "A {person} {tense} verb in the {voice} voice{form}{trans}.",
                        person = person, tense = tense, voice = voice,
                        form = form, trans = trans,
                    )
                }
            }
        }
        EducationalLevel::Advanced => {
            let form = mf.verb_form
                .map(|f| format!(". Form {}", roman_numeral(f)))
                .unwrap_or_default();
            let root_info = mf.root.as_ref()
                .map(|r| format!(". Root: {} ({})", r, language_pack.feature_value("root_type", mf.root_type.as_deref().unwrap_or("sound"))))
                .unwrap_or_default();
            let mood_info = mf.mood.as_ref()
                .map(|m| format!(". Mood: {}", language_pack.feature_value("mood", m)))
                .unwrap_or_default();

            format!(
                "A {person} {gender} {number} {tense} verb in the {voice} voice{form}{mood_info}{root_info}. \
                 It functions as the predicate (fi'l) of the sentence.",
                person = person, gender = gender.to_lowercase(),
                number = number.to_lowercase(), tense = tense.to_lowercase(),
                voice = voice.to_lowercase(), form = form,
                mood_info = mood_info, root_info = root_info,
            )
        }
    }
}

fn explain_subject_agreement(mf: &MorphologicalFeatures, language_pack: &LanguagePack) -> String {
    let person = language_pack.feature_value("person", mf.person.as_deref().unwrap_or("third"));
    let gender = language_pack.feature_value("gender", mf.gender.as_deref().unwrap_or("masculine"));
    let number = language_pack.feature_value("number", mf.number.as_deref().unwrap_or("singular"));

    match mf.number.as_deref() {
        Some("singular") => {
            format!("It agrees with a {} {} {} subject.", person, gender.to_lowercase(), number.to_lowercase())
        }
        Some("dual") => {
            format!("It agrees with a {} {} dual subject.", person, gender.to_lowercase())
        }
        Some("plural") => {
            if mf.gender.as_deref() == Some("feminine") {
                format!("It agrees with a {} feminine plural subject.", person)
            } else {
                format!("It agrees with a {} masculine plural subject.", person)
            }
        }
        _ => String::new(),
    }
}
```

### 5.4 Noun Explanation Patterns

```rust
fn explain_subject(
    token: &AnalysisToken,
    mf: &MorphologicalFeatures,
    sf: &SyntacticFeatures,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    let gender = feature_value(mf.gender.as_deref(), "masculine", language_pack, "gender");
    let number = feature_value(mf.number.as_deref(), "singular", language_pack, "number");
    let state = feature_value(mf.state.as_deref(), "indefinite", language_pack, "state");

    match level {
        EducationalLevel::Beginner => {
            format!(
                "The subject (fa'il) of the sentence. It is {} {} {}.",
                state.to_lowercase(), gender.to_lowercase(), number.to_lowercase(),
            )
        }
        EducationalLevel::Intermediate => {
            format!(
                "The subject (fa'il) of the verb. A {} {} {} noun in the nominative case (الرفع), {}.",
                gender.to_lowercase(), number.to_lowercase(), state.to_lowercase(),
                state.to_lowercase(),
            )
        }
        EducationalLevel::Advanced => {
            let root_info = mf.root.as_ref()
                .map(|r| format!(" Root: {}.", r))
                .unwrap_or_default();
            format!(
                "The doer (فاعل) of the action. A {} {} {} noun, nominative (مرفوع), {}.{} \
                 The subject agrees with the verb in person, gender, and number.",
                gender.to_lowercase(), number.to_lowercase(), state.to_lowercase(),
                state.to_lowercase(), root_info,
            )
        }
    }
}

fn explain_object(
    token: &AnalysisToken,
    mf: &MorphologicalFeatures,
    sf: &SyntacticFeatures,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    let gender = feature_value(mf.gender.as_deref(), "masculine", language_pack, "gender");
    let number = feature_value(mf.number.as_deref(), "singular", language_pack, "number");
    let state = feature_value(mf.state.as_deref(), "indefinite", language_pack, "state");

    match level {
        EducationalLevel::Beginner => {
            format!(
                "The direct object (maf'ul bi-hi) of the verb. A {} {} {} noun in the accusative case.",
                gender.to_lowercase(), number.to_lowercase(), state.to_lowercase(),
            )
        }
        EducationalLevel::Intermediate => {
            format!(
                "The direct object (مفعول به) — the entity that receives the action. \
                 A {} {} {} noun in the accusative case (النصب), {}.",
                gender.to_lowercase(), number.to_lowercase(), state.to_lowercase(),
                state.to_lowercase(),
            )
        }
        EducationalLevel::Advanced => {
            format!(
                "The direct object (مفعول به) of the transitive verb. \
                 A {} {} {} noun, accusative (منصوب), {}. \
                 The object is governed by the verb and takes the accusative case \
                 as indicated by the {}.",
                gender.to_lowercase(), number.to_lowercase(), state.to_lowercase(),
                state.to_lowercase(),
                mf.case_ending(mf.case.as_deref(), mf.state.as_deref(), mf.number.as_deref()),
            )
        }
    }
}

fn explain_mubtada(
    token: &AnalysisToken,
    mf: &MorphologicalFeatures,
    sf: &SyntacticFeatures,
    sentence_type: &SentenceType,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    let state = feature_value(mf.state.as_deref(), "definite", language_pack, "state");

    match level {
        EducationalLevel::Beginner => {
            format!(
                "The topic (mubtada') of the sentence. It is {} and in the nominative case.",
                state.to_lowercase(),
            )
        }
        EducationalLevel::Intermediate => {
            format!(
                "The subject/topic (مبتدأ) of the nominal sentence. \
                 A {} noun in the nominative case (مرفوع).",
                state.to_lowercase(),
            )
        }
        EducationalLevel::Advanced => {
            let role_desc = match sentence_type {
                SentenceType::Inna { .. } => {
                    " governed by inna (or one of her sisters), which assigns the accusative case. \
                     However, in a standard nominal sentence it remains nominative.".into()
                }
                SentenceType::Kana { .. } => {
                    " governed by kana (or one of her sisters). Kana raises the subject \
                     to the nominative case.".into()
                }
                _ => ". It is the topic about which a statement (khabar) is made.".into(),
            };
            format!(
                "The topic (مبتدأ) — the subject of the nominal sentence. \
                 A {} noun in the nominative (مرفوع){role_desc}",
                state.to_lowercase(),
                role_desc = role_desc,
            )
        }
    }
}
```

### 5.5 Particle Explanation Patterns

```rust
fn explain_preposition(
    token: &AnalysisToken,
    mf: &MorphologicalFeatures,
    sf: &SyntacticFeatures,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    match level {
        EducationalLevel::Beginner => {
            format!("A preposition (harf jarr). It governs the genitive case.")
        }
        EducationalLevel::Intermediate => {
            format!(
                "A preposition (حرف جر) — '{}'. \
                 The following noun is in the genitive case (مجرور).",
                token.text,
            )
        }
        EducationalLevel::Advanced => {
            format!(
                "A preposition (حرف جر): '{}'. It governs the genitive case (الجر) \
                 on the following noun or phrase (المجرور). \
                 Prepositions are among the 'amil (عامل) that affect the case of nouns.",
                translate_preposition(&token.text, language_pack),
            )
        }
    }
}

fn explain_subjunctive_particle(
    token: &AnalysisToken,
    mf: &MorphologicalFeatures,
    sf: &SyntacticFeatures,
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> String {
    let text = token.text.as_str();
    let meaning = match text {
        "أَنْ" => "that/to",
        "لَنْ" => "will not (emphatic negation)",
        "كَيْ" => "in order to",
        "إِذَنْ" => "then / in that case",
        "لِـ" => "so that / in order to",
        _ => "subjunctive particle",
    };

    match level {
        EducationalLevel::Beginner => {
            format!("A particle that puts the following verb in the subjunctive mood (النصب).")
        }
        EducationalLevel::Intermediate => {
            format!(
                "A subjunctive particle (حرف نصب) — '{}' ({}). \
                 It assigns the subjunctive mood (النصب) to the following verb.",
                token.text, meaning,
            )
        }
        EducationalLevel::Advanced => {
            format!(
                "A subjunctive particle (حرف نصب): '{}' meaning '{}'. \
                 It is one of the 'awamil that assign the subjunctive mood (النصب) \
                 to the imperfect verb (الفعل المضارع). \
                 After this particle, the verb takes the subjunctive case.",
                token.text, meaning,
            )
        }
    }
}
```

### 5.6 Edge Case Handling

#### 5.6.1 Elliptical Sentences (Hadhdh)

When the analysis detects omitted elements (implicit constituents), the explanation adapts:

```rust
fn handle_ellipsis(
    tree: &AnalysisTree,
    language_pack: &LanguagePack,
) -> Option<String> {
    let implicit: Vec<&Constituent> = tree.constituents.iter()
        .filter(|c| c.implicit)
        .collect();

    if implicit.is_empty() {
        return None;
    }

    let omitted_roles: Vec<String> = implicit.iter()
        .map(|c| language_pack.role_name(&c.role))
        .collect();

    Some(format!(
        "This sentence contains an ellipsis (حذف). The following element(s) are implied \
         but not explicitly stated: {}. In Arabic grammar, this is a common and \
         grammatically valid construction.",
        omitted_roles.join(", "),
    ))
}
```

#### 5.6.2 Poetic License

```rust
fn detect_poetic_license(
    tree: &AnalysisTree,
    flags: &[GrammaticalFlag],
) -> Vec<String> {
    let mut licenses = Vec::new();

    for flag in flags {
        match flag.code.as_str() {
            "POETIC_WORD_ORDER" => licenses.push(
                "This word order is a poetic license (ضرورة شعرية). \
                 In prose, the expected order would be different.".into()
            ),
            "POETIC_CASE_USAGE" => licenses.push(
                "This case assignment follows poetic license. \
                 Classical poets sometimes use cases differently from prose rules.".into()
            ),
            "POETIC_VERB_FORM" => licenses.push(
                "This verb form is a poetic variant. The standard form differs.".into()
            ),
            _ => {}
        }
    }

    licenses
}
```

#### 5.6.3 Ambiguity Presentation

```rust
fn generate_ambiguity_explanation(
    primary: &AnalysisTree,
    alternatives: &[AnalysisTree],
    language_pack: &LanguagePack,
    level: EducationalLevel,
) -> Vec<String> {
    if alternatives.is_empty() {
        return vec![];
    }

    let mut notes = Vec::new();

    match level {
        EducationalLevel::Beginner => {
            notes.push(
                "This sentence can be understood in more than one way. \
                 The most likely interpretation is shown.".into()
            );
        }
        EducationalLevel::Intermediate => {
            notes.push(format!(
                "Grammatical ambiguity (إشتراك نحوي): This sentence has {} possible \
                 interpretation(s). The primary analysis (confidence: {:.0}%) is shown. \
                 Alternative interpretations are noted below.",
                alternatives.len() + 1,
                primary.confidence * 100.0,
            ));
        }
        EducationalLevel::Advanced => {
            notes.push(format!(
                "Grammatical ambiguity (إشتراك نحوي / تعدد الإعراب): \
                 This sentence admits {} valid grammatical analysis/analyses. \
                 The primary analysis (confidence: {:.0}%) is presented below, \
                 followed by the alternative(s) with explanations of the differences.",
                alternatives.len() + 1,
                primary.confidence * 100.0,
            ));
        }
    }

    for (i, alt) in alternatives.iter().enumerate() {
        let diff = describe_difference(primary, alt);
        notes.push(format!(
            "Alternative {} (confidence: {:.0}%): {}",
            i + 1,
            alt.confidence * 100.0,
            diff,
        ));
    }

    notes
}
```

### 5.7 Comprehensive I'rab Example

For the sentence **كتب محمد رسالة** (Muhammad wrote a letter):

```json
{
    "irab_breakdown": [
        {
            "token": "كتب",
            "root": "كتب",
            "pos": "Verb",
            "features": [
                { "name": "Gender", "value": "Masculine", "category": "inflectional" },
                { "name": "Number", "value": "Singular", "category": "inflectional" },
                { "name": "Person", "value": "Third", "category": "inflectional" },
                { "name": "Tense", "value": "Past (الماضي)", "category": "inflectional" },
                { "name": "Voice", "value": "Active (معلوم)", "category": "inflectional" }
            ],
            "syntactic_role": "Verb (فعل)",
            "explanation": "A third person masculine singular past tense verb in the active voice, Form I. It agrees with a third masculine singular subject."
        },
        {
            "token": "محمد",
            "root": "حمد",
            "pos": "Proper Noun",
            "features": [
                { "name": "Gender", "value": "Masculine", "category": "inflectional" },
                { "name": "Number", "value": "Singular", "category": "inflectional" },
                { "name": "Case", "value": "Nominative (الرفع)", "category": "inflectional" },
                { "name": "State", "value": "Indefinite (نكرة)", "category": "inflectional" }
            ],
            "syntactic_role": "Subject (فاعل)",
            "explanation": "The subject (fa'il) of the verb. A masculine singular proper noun in the nominative case (مرفوع), indefinite. The subject agrees with the verb in person, gender, and number."
        },
        {
            "token": "رسالة",
            "root": "ر س ل",
            "pos": "Noun",
            "features": [
                { "name": "Gender", "value": "Feminine", "category": "inflectional" },
                { "name": "Number", "value": "Singular", "category": "inflectional" },
                { "name": "Case", "value": "Accusative (النصب)", "category": "inflectional" },
                { "name": "State", "value": "Indefinite (نكرة)", "category": "inflectional" }
            ],
            "syntactic_role": "Direct Object (مفعول به)",
            "explanation": "The direct object (مفعول به) — the entity that receives the action. A feminine singular indefinite noun in the accusative case (منصوب). The object is governed by the transitive verb 'كتب'."
        }
    ]
}
```

---

## 6. Construction Identification

### 6.1 Construction Detection Algorithms

Each construction is detected by analyzing the constituent tree. The following algorithms identify 20+ grammatical constructions:

```rust
fn identify_constructions(tree: &AnalysisTree) -> Vec<Construction> {
    let mut constructions = Vec::new();

    // Walk the constituent tree depth-first
    for node in &tree.constituents {
        match node.role.as_str() {
            "idafa" => detect_idafa(node, tree, &mut constructions),
            "na'at" => detect_naat(node, tree, &mut constructions),
            "tawkid" => detect_tawkid(node, tree, &mut constructions),
            "badal" => detect_badal(node, tree, &mut constructions),
            "istithna" => detect_istithna(node, tree, &mut constructions),
            "nida" => detect_nida(node, tree, &mut constructions),
            "shart" => detect_shart(node, tree, &mut constructions),
            "jaza" => {} // Handled by shart detection
            "hal" => detect_hal(node, tree, &mut constructions),
            "tamyiz" => detect_tamyiz(node, tree, &mut constructions),
            "zarf" => detect_zarf(node, tree, &mut constructions),
            "idafah_fi'liyyah" => detect_idafah_filiyya(node, tree, &mut constructions),
            "qasam" => detect_qasam(node, tree, &mut constructions),
            "jumlah_fi'liyyah" => detect_verbal_sentence(node, tree, &mut constructions),
            "jumlah_ismiyyah" => detect_nominal_sentence(node, tree, &mut constructions),
            _ => {}
        }
    }

    // Sort by token position (earliest construction first)
    constructions.sort_by(|a, b| {
        let a_start = a.tokens.first().unwrap_or(&0);
        let b_start = b.tokens.first().unwrap_or(&0);
        a_start.cmp(b_start)
    });

    constructions
}
```

#### 6.1.1 Idafa (Construct State)

```rust
fn detect_idafa(node: &Constituent, tree: &AnalysisTree, constructions: &mut Vec<Construction>) {
    // Idafa has two parts: mudaf (first term) and mudaf_ilayh (second term in genitive)
    let mudaf = node.children.iter()
        .find(|c| c.role == "mudaf");
    let mudaf_ilayh = node.children.iter()
        .find(|c| c.role == "mudaf_ilayh");

    if let (Some(mudaf), Some(mudaf_ilayh)) = (mudaf, mudaf_ilayh) {
        let mudaf_text = get_token_texts(&mudaf.token_ids, tree);
        let mudaf_ilayh_text = get_token_texts(&mudaf_ilayh.token_ids, tree);

        constructions.push(Construction {
            name: "Idafa (Construct State / إضافة)".into(),
            description: format!(
                "A possessive construction: '{} of {}. The first term (مضاف) is in the \
                 construct state and cannot take the definite article. \
                 The second term (مضاف إليه) is in the genitive case (مجرور).",
                mudaf_text, mudaf_ilayh_text,
            ),
            tokens: node.token_ids.clone(),
        });
    }
}

fn get_token_texts(token_ids: &[u32], tree: &AnalysisTree) -> String {
    token_ids.iter()
        .filter_map(|id| tree.tokens.iter().find(|t| t.index == *id))
        .map(|t| t.text.as_str())
        .collect::<Vec<&str>>()
        .join(" ")
}
```

#### 6.1.2 Na'at (Adjective Agreement)

```rust
fn detect_naat(node: &Constituent, tree: &AnalysisTree, constructions: &mut Vec<Construction>) {
    // Na'at has two parts: the described noun (man'ut) and the adjective (na'at)
    let described = node.children.iter()
        .find(|c| c.role == "man'ut" || c.role == "mawsoof");
    let adjective = node.children.iter()
        .find(|c| c.role == "na'at");

    if let (Some(described), Some(adjective)) = (described, adjective) {
        let desc_text = get_token_texts(&described.token_ids, tree);
        let adj_text = get_token_texts(&adjective.token_ids, tree);

        // Determine agreement features
        let desc_token = described.token_ids.first()
            .and_then(|id| tree.tokens.iter().find(|t| t.index == *id));
        let adj_token = adjective.token_ids.first()
            .and_then(|id| tree.tokens.iter().find(|t| t.index == *id));

        let agreement = match (desc_token, adj_token) {
            (Some(dt), Some(at)) => {
                let d_g = dt.features.morphological.gender.as_deref().unwrap_or("?");
                let a_g = at.features.morphological.gender.as_deref().unwrap_or("?");
                let d_n = dt.features.morphological.number.as_deref().unwrap_or("?");
                let a_n = at.features.morphological.number.as_deref().unwrap_or("?");
                let d_c = dt.features.morphological.case.as_deref().unwrap_or("?");
                let a_c = at.features.morphological.case.as_deref().unwrap_or("?");
                let d_s = dt.features.morphological.state.as_deref().unwrap_or("?");
                let a_s = at.features.morphological.state.as_deref().unwrap_or("?");

                if d_g == a_g && d_n == a_n && d_c == a_c && d_s == a_s {
                    format!("The adjective agrees with '{}' in gender ({}/{}/{}), number ({}/{}), case ({}/{}), and definiteness ({}/{}).",
                        desc_text, d_g, a_g, d_n, a_n, d_c, a_c, d_s, a_s)
                } else {
                    let mismatches: Vec<String> = {
                        let mut m = Vec::new();
                        if d_g != a_g { m.push("gender".into()); }
                        if d_n != a_n { m.push("number".into()); }
                        if d_c != a_c { m.push("case".into()); }
                        if d_s != a_s { m.push("definiteness".into()); }
                        m
                    };
                    format!("The adjective '{}' modifies '{}', but note the {} agreement difference(s): {}.",
                        adj_text, desc_text, mismatches.len(), mismatches.join(", "))
                }
            }
            _ => String::new(),
        };

        constructions.push(Construction {
            name: "Na'at (Adjective Agreement / نعت)".into(),
            description: format!(
                "'{}' describes '{}'. {}", adj_text, desc_text, agreement
            ),
            tokens: node.token_ids.clone(),
        });
    }
}
```

#### 6.1.3 Shart wa Jaza (Conditional)

```rust
fn detect_shart(node: &Constituent, tree: &AnalysisTree, constructions: &mut Vec<Construction>) {
    let condition = node.children.iter()
        .find(|c| c.role == "shart");
    let result = node.children.iter()
        .find(|c| c.role == "jaza");
    let particle = tree.tokens.iter()
        .find(|t| matches!(t.features.morphological.pos.as_str(), "particle") &&
              t.features.syntactic.role.as_deref() == Some("harf_shart"));

    if let (Some(cond), Some(res)) = (condition, result) {
        let cond_text = get_token_texts(&cond.token_ids, tree);
        let res_text = get_token_texts(&res.token_ids, tree);
        let particle_info = particle.map(|p| format!(" using '{}'", p.text)).unwrap_or_default();

        constructions.push(Construction {
            name: "Shart wa Jaza (Conditional / شرط وجزاء)".into(),
            description: format!(
                "A conditional sentence{}. Condition (شرط): '{}'. Result (جزاء): '{}'. \
                 The condition particle governs the verb(s) in the jussive mood (مجزوم).",
                particle_info, cond_text, res_text,
            ),
            tokens: node.token_ids.clone(),
        });
    }
}
```

### 6.2 Complete Construction Catalog

| # | Construction | Arabic Term | Detection Method | Tokens Involved |
|---|-------------|-------------|-----------------|-----------------|
| 1 | **Idafa (Construct State)** | إضافة | Constituent role `idafa` with children `mudaf` + `mudaf_ilayh` | 2+ |
| 2 | **Na'at (Adjective)** | نعت | Constituent role `na'at` with `man'ut`/`mawsoof` child | 2 |
| 3 | **Tawkid (Emphasis)** | توكيد | Constituent role `tawkid` (nafs, 'ayn, kull, etc.) | 2 |
| 4 | **Badal (Apposition)** | بدل | Constituent role `badal` with `mubdal_minhu` child | 2 |
| 5 | **Istithna (Exception)** | استثناء | Constituent role `istithna` with إلا, غير, سوى | 2+ |
| 6 | **Nida (Vocative)** | نداء | Constituent role `nida` with يا, أ, etc. | 2 |
| 7 | **Shart (Conditional)** | شرط | Constituent role `shart` with `jaza` sibling | 3+ |
| 8 | **Verbal Sentence** | جملة فعلية | Tree type `jumlah_fi'liyyah` | All tokens |
| 9 | **Nominal Sentence** | جملة اسمية | Tree type `jumlah_ismiyyah` | All tokens |
| 10 | **Kana & Sisters** | كان وأخواتها | Tree type `jumlah_kana` | 3+ |
| 11 | **Inna & Sisters** | إن وأخواتها | Tree type `jumlah_inna` | 3+ |
| 12 | **Hal (Circumstantial)** | حال | Constituent role `hal` | 2+ |
| 13 | **Tamyiz (Specification)** | تمييز | Constituent role `tamyiz` | 2 |
| 14 | **Zarf (Adverb)** | ظرف | Constituent role `zarf` (time or place) | 1+ |
| 15 | **Idafah Fi'liyyah** | إضافة فعلية | Verb governing a genitive noun | 2 |
| 16 | **Qasam (Oath)** | قسم | Tree type `jumlah_qasam` | 2+ |
| 17 | **Maf'ul Mutlaq** | مفعول مطلق | Constituent role `maf'ul_mutlaq` | 1-2 |
| 18 | **Maf'ul Fih (Zarf)** | مفعول فيه | Constituent role `maf'ul_fih` | 1+ |
| 19 | **Maf'ul Lahu** | مفعول لأجله | Constituent role `maf'ul_lahu` | 1+ |
| 20 | **Maf'ul Ma'ahu** | مفعول معه | Constituent role `maf'ul_ma'ahu` | 2 |
| 21 | **Ta'kid (Emphatic)** | تأكيد | Double-construction (e.g., verb + verbal noun) | 2 |
| 22 | **Ighra' (Urging)** | إغراء | Special urging construction | 2+ |
| 23 | **Hadhf (Ellipsis)** | حذف | Implicit constituents detected | Variable |
| 24 | **Iltifat (Shift)** | إلتفات | Pronoun/person shift in discourse | 3+ |

### 6.3 Construction Composition Rules

Multiple constructions can overlap in the same sentence. The following rules govern how constructions are composed:

```rust
/// Rules governing construction composition.
/// Higher precedence constructions take priority when overlapping.
enum ConstructionComposition {
    /// Constructions are independent and non-overlapping
    Independent,

    /// One construction is contained within another (parent-child)
    Containment {
        parent: ConstructionType,
        child: ConstructionType,
    },

    /// Constructions cannot coexist (mutually exclusive)
    MutuallyExclusive {
        a: ConstructionType,
        b: ConstructionType,
    },

    /// One construction implies the presence of another
    Implies {
        if_present: ConstructionType,
        then_implies: ConstructionType,
    },
}

/// Precedence order (highest to lowest) for overlapping constructions:
/// 1. Qasam (Oath) — wraps entire sentence
/// 2. Shart wa Jaza (Conditional) — wraps entire sentence
/// 3. Kana / Inna — sentence-level operators
/// 4. Istithna (Exception) — clause-level
/// 5. Idafa (Construct) — phrase-level
/// 6. Na'at (Adjective) — phrase-level
/// 7. Badal, Tawkid, Hal, Tamyiz — word/phrase-level
```

---

## 7. Educational Levels

### 7.1 Level Definitions

Three educational levels are defined, each targeting a different learner profile:

#### Level 1: Beginner (مبتدئ)

**Target audience:** Students new to Arabic grammar, self-learners, children.

**Pedagogical approach:**
- Use plain language equivalents of grammatical terms
- Limit to 3–4 most important features per word
- Avoid mentioning grammar school differences
- Present only the primary analysis (hide ambiguity)
- Use visual markers for case endings where possible
- Provide contextual, real-world examples

**Terminology mapping:**
```
Arabic Term         →  Beginner Term
────────────────────────────────────
fa'il              →  subject (doer of the action)
fi'l               →  verb (action word)
mubtada'           →  topic (what the sentence is about)
khabar             →  comment (what is said about the topic)
maf'ul bi-hi       →  object (receiver of the action)
harf jarr          →  preposition (linking word)
majrur             →  noun after a preposition
na'at              →  describing word (adjective)
idafa              →  possession (of / 's)
marfu'             →  nominative (subject form)
mansub             →  accusative (object form)
majrur             →  genitive (after-preposition form)
```

#### Level 2: Intermediate (متوسط)

**Target audience:** Students with basic Arabic grammar knowledge, university undergraduates.

**Pedagogical approach:**
- Use standard Arabic grammatical terms with English translations
- Show 6–8 features per word (all standard inflectional features)
- Mention school differences when significant
- Present alternative analyses when confidence is close (within 10%)
- Include construction descriptions with Arabic terminology
- Show evidence trail on request

**Terminology style:** Dual (Arabic + English) by default.

#### Level 3: Advanced (متقدم)

**Target audience:** Graduate students, researchers, Arabic language teachers.

**Pedagogical approach:**
- Use full Arabic grammatical terminology
- Show all available features including derived and semantic
- Provide full rule citations (rule IDs, source references)
- Present all alternative analyses with detailed comparisons
- Include evidence trail by default
- Reference classical grammarians (Sibawayh, Al-Mubarrad, Ibn Malik, etc.)
- Include references to Quranic and poetic usage examples
- Show case ending justifications (e.g., "the damma indicates nominative case")
- Provide construction lineage (e.g., "this idafa consists of a mudaf in construct state...")

### 7.2 Level Adaptation Pipeline

```rust
/// Adapt the explanation output to the requested educational level.
fn adapt_to_level(
    analysis: &ExplanationAnalysis,
    level: EducationalLevel,
) -> ExplanationAnalysis {
    match level {
        EducationalLevel::Beginner => {
            // 1. Simplify terminology
            let irab_entries = analysis.irab_entries.iter()
                .map(|entry| simplify_entry_for_beginner(entry))
                .collect();

            // 2. Limit features to essential ones
            // 3. Remove ambiguity notes
            // 4. Simplify overview
            // 5. Remove evidence trail
            ExplanationAnalysis {
                irab_entries,
                overview: simplify_overview(&analysis.overview),
                ambiguity_notes: vec![],  // Hide ambiguity
                constructions: analysis.constructions.iter()
                    .map(|c| simplify_construction(c))
                    .collect(),
                ..analysis
            }
        }
        EducationalLevel::Intermediate => {
            // 1. Keep dual terminology
            // 2. Show standard features
            // 3. Show ambiguity notes (summarized)
            // 4. Include evidence trail (optional)
            analysis.clone()  // Intermediate is the default presentation
        }
        EducationalLevel::Advanced => {
            // 1. Use full Arabic terminology
            // 2. Show all features
            // 3. Show full ambiguity analysis
            // 4. Include rule references
            // 5. Include evidence trail
            let irab_entries = analysis.irab_entries.iter()
                .map(|entry| enrich_entry_for_advanced(entry))
                .collect();

            ExplanationAnalysis {
                irab_entries,
                overview: enrich_overview(&analysis.overview),
                ..analysis
            }
        }
    }
}

fn simplify_entry_for_beginner(entry: &IrabEntry) -> IrabEntry {
    IrabEntry {
        pos: beginner_pos_name(&entry.pos),
        features: entry.features.iter()
            .take(4)  // Only 4 most important features
            .map(|f| FeatureDisplay {
                name: beginner_feature_name(&f.name),
                value: beginner_feature_value(&f.name, &f.value),
                category: f.category.clone(),
            })
            .collect(),
        syntactic_role: entry.syntactic_role.as_ref()
            .map(|r| beginner_role_name(r)),
        explanation: simplify_explanation(&entry.explanation),
        ..entry.clone()
    }
}
```

### 7.3 Level-Specific Verb Explanation Examples

**Analysis:** كتب (third person masculine singular past tense verb, active voice, Form I)

| Level | Explanation |
|-------|-------------|
| **Beginner** | "A past tense verb. It describes an action that happened in the past. The doer is a 'he' (masculine, one person)." |
| **Intermediate** | "A third person masculine singular past tense verb in the active voice, Form I. It agrees with a third masculine singular subject (fa'il). The verb is مرفوع (nominative) in its original form." |
| **Advanced** | "A third person masculine singular past tense verb (فعل ماض) in the active voice (معلوم), Form I (فَعَلَ). It is مبني على الفتح (built on fatha) because it is past tense — past tense verbs are always مبني (indeclinable). The apparent damma on the subject (مُحَمَّدٌ) indicates its nominative case as the فاعل (subject). Root: كتب (k-t-b), meaning 'to write'. The verb is transitive and takes a direct object (رسالة)." |

---

## 8. LLM Prompt Engineering & Integration

### 8.1 Prompt Architecture

```
┌──────────────────────────────────────────────┐
│              LLM PROMPT SYSTEM                │
│                                              │
│  ┌──────────────────────────────────────┐    │
│  │  PROMPT TEMPLATES                     │    │
│  │                                       │    │
│  │  • system/arabic-teacher              │    │
│  │  • system/arabic-teacher-beginner     │    │
│  │  • system/arabic-teacher-advanced     │    │
│  │  • user/standard-explanation          │    │
│  │  • user/token-by-token                │    │
│  │  • user/teacher-mode                  │    │
│  │  • fewshot/verbal-sentence            │    │
│  │  • fewshot/nominal-sentence           │    │
│  │  • fewshot/conditional                │    │
│  └──────────────────────────────────────┘    │
│                     │                        │
│  ┌──────────────────▼───────────────────┐    │
│  │  PROMPT COMPILER                     │    │
│  │                                      │    │
│  │  1. Select base template (by level)  │    │
│  │  2. Inject AnalysisResult (JSON)     │    │
│  │  3. Add few-shot examples            │    │
│  │     (if applicable)                  │    │
│  │  4. Apply language + format params   │    │
│  │  5. Apply constraints (max_tokens)   │    │
│  └──────────────────────────────────────┘    │
│                     │                        │
│  ┌──────────────────▼───────────────────┐    │
│  │  PROVIDER DISPATCH                    │    │
│  │                                      │    │
│  │  • OpenAI    (gpt-4, gpt-4o-mini)    │    │
│  │  • Anthropic (claude-3.5-sonnet)     │    │
│  │  • Custom    (any OpenAI-compatible) │    │
│  │  • Noop      (disabled)             │    │
│  └──────────────────────────────────────┘    │
│                     │                        │
│  ┌──────────────────▼───────────────────┐    │
│  │  RESPONSE VALIDATOR                   │    │
│  │                                      │    │
│  │  1. Parse JSON/text response         │    │
│  │  2. Check against AnalysisResult     │    │
│  │  3. Detect contradictions            │    │
│  │  4. Detect hallucinated terms        │    │
│  │  5. Sanity-check token references    │    │
│  └──────────────────────────────────────┘    │
│                     │                        │
│  ┌──────────────────▼───────────────────┐    │
│  │  PROMPT CACHE                         │    │
│  │                                      │    │
│  │  • Key: hash(AnalysisResult + level  │    │
│  │    + language + model)               │    │
│  │  • TTL: configurable (default: 24h)  │    │
│  │  • Hit ratio target: > 60%           │    │
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

### 8.2 Prompt Templates

#### 8.2.1 System Prompt (Standard)

```text
You are an expert Arabic grammar teacher (nahw and sarf) named "AGOS Tutor".
You receive a structured grammatical analysis and must generate clear,
pedagogically helpful explanations.

CRITICAL RULES:
1. NEVER contradict or modify the provided analysis. The analysis is
   deterministic and correct. Your role is to explain it, not to re-analyze.
2. Do not add grammatical information not present in the analysis.
3. If the analysis has ambiguity, mention it neutrally.
4. Use Arabic grammatical terminology alongside English translations.
5. Keep explanations concise (2-4 sentences per token).
6. Focus on being helpful to a learner of Arabic grammar.

OUTPUT FORMAT:
Return your response as a JSON object with the following structure:
{
    "overview": "string - 2-3 sentence summary of the sentence's grammar",
    "tokens": [
        {
            "index": 0,
            "explanation": "string - 2-4 sentence explanation"
        }
    ],
    "constructions": [
        {
            "name": "string",
            "explanation": "string - 1-2 sentence explanation"
        }
    ],
    "teaching_tips": ["string - optional tips for the learner (max 3)"]
}
```

#### 8.2.2 System Prompt (Beginner)

```text
You are a friendly Arabic grammar tutor for beginners. You explain grammar
in simple, easy-to-understand language. Avoid technical terms when possible.

CRITICAL RULES:
1. NEVER contradict or modify the provided analysis.
2. Use simple words: say "action word" instead of "verb",
   "doer" instead of "subject", "receiver" instead of "object".
3. Keep each explanation to 1-2 short sentences.
4. Use examples the learner can relate to.
5. Be encouraging and positive.

OUTPUT FORMAT:
Return a simple JSON:
{
    "overview": "string - one simple sentence",
    "tokens": [
        {
            "index": 0,
            "explanation": "string - 1-2 simple sentences"
        }
    ],
    "tip": "string - one helpful tip for the learner"
}
```

#### 8.2.3 System Prompt (Advanced / Research)

```text
You are an advanced Arabic grammarian and researcher. Provide detailed
grammatical analysis suitable for scholars and advanced students.

Include when relevant:
- Complete i'rab (إعراب) with case ending justifications
- Grammatical rule citations (rule IDs, classical sources)
- School-specific differences (Basra vs. Kufa vs. Baghdad)
- Quranic or poetic precedents for the construction
- Morphological breakdown (root, pattern, form, features)
- Reference to classical grammarians (Sibawayh, Al-Mubarrad, Ibn Malik, etc.)

OUTPUT FORMAT:
Return a detailed JSON:
{
    "overview": "string - comprehensive sentence analysis",
    "irab_summary": "string - summary of the i'rab state",
    "tokens": [
        {
            "index": 0,
            "irab": "string - complete i'rab in Arabic",
            "explanation": "string - detailed explanation with references",
            "rule_references": ["string - rule IDs applied to this token"],
            "school_notes": "string - any school-specific differences"
        }
    ],
    "constructions": [...],
    "references": ["string - classical references"],
    "scholarly_notes": ["string - additional observations"]
}
```

#### 8.2.4 Few-Shot Example

```text
EXAMPLE 1:
Input Analysis (JSON): {
    "sentence_type": "jumlah_fi'liyyah",
    "tokens": [
        {"index": 0, "text": "كَتَبَ", "pos": "verb", "tense": "past", "gender": "masculine", "number": "singular", "person": "third", "voice": "active", "role": "fi'l"},
        {"index": 1, "text": "مُحَمَّدٌ", "pos": "proper_noun", "case": "nominative", "state": "indefinite", "gender": "masculine", "number": "singular", "role": "fa'il"},
        {"index": 2, "text": "رِسَالَةً", "pos": "noun", "case": "accusative", "state": "indefinite", "gender": "feminine", "number": "singular", "role": "maf'ul_bi-hi"}
    ]
}

Output:
{
    "overview": "This is a verbal sentence (jumlah fi'liyyah) in the past tense. The verb 'kataba' (to write) is followed by its subject 'Muhammadun' and its object 'risalatan' (a letter), forming a complete transitive verb construction.",
    "tokens": [
        {"index": 0, "explanation": "The verb كَتَبَ (kataba) is a third person masculine singular past tense verb in the active voice, Form I. As a past tense verb, it is مبني على الفتح (built on fatha). The verb agrees with its subject in person (third), gender (masculine), and number (singular)."},
        {"index": 1, "explanation": "مُحَمَّدٌ (Muhammadun) is a proper noun functioning as the subject (فاعل) of the verb. It is nominative (مرفوع), indicated by the apparent damma on the final letter. The subject performs the action of writing."},
        {"index": 2, "explanation": "رِسَالَةً (risalatan) is a feminine singular noun functioning as the direct object (مفعول به) of the transitive verb كَتَبَ. It is in the accusative case (منصوب), indicated by the fatha (and tanwin) on the final letter."}
    ],
    "constructions": [
        {"name": "Verbal Sentence", "explanation": "This is a canonical verbal sentence (جملة فعلية) consisting of verb + subject + object, the standard word order for Classical Arabic past tense sentences."}
    ],
    "teaching_tips": [
        "Notice how the verb comes before the subject in Arabic verbal sentences — this is the opposite of English word order.",
        "The verb 'kataba' is from Form I, the simplest verb pattern. Most Arabic verbs follow one of 15 forms."
    ]
}
```

### 8.3 Response Validation

```rust
/// Validate LLM response against the original analysis.
fn validate_llm_response(
    response: &LLMResponse,
    analysis: &AnalysisResult,
) -> Result<(), LLMValidationError> {
    // 1. Parse the response JSON
    let parsed: serde_json::Value = serde_json::from_str(&response.text)
        .map_err(|_| LLMValidationError::InvalidJson)?;

    // 2. Check token count matches
    let parsed_tokens = parsed["tokens"].as_array()
        .ok_or(LLMValidationError::MissingField("tokens"))?;
    let expected_count = analysis.trees[0].tokens.len();

    if parsed_tokens.len() != expected_count {
        return Err(LLMValidationError::TokenCountMismatch {
            expected: expected_count,
            got: parsed_tokens.len(),
        });
    }

    // 3. Check each token index is valid
    for token_entry in parsed_tokens {
        let index = token_entry["index"].as_u64()
            .ok_or(LLMValidationError::MissingField("token.index"))? as usize;
        if index >= expected_count {
            return Err(LLMValidationError::InvalidTokenIndex {
                index,
                max: expected_count - 1,
            });
        }
    }

    // 4. Check for contradictions
    // (LLM says "past tense" but analysis says "present", etc.)
    let contradictions = find_contradictions(&parsed, analysis);
    if !contradictions.is_empty() {
        return Err(LLMValidationError::ContradictionDetected(contradictions));
    }

    // 5. Check for hallucinated grammatical terms
    let known_terms = get_known_grammatical_terms();
    let unknown_terms = find_unknown_terms(&response.text, &known_terms);
    if unknown_terms.len() > 3 {
        // More than 3 unknown terms suggests hallucination
        return Err(LLMValidationError::SuspiciousTerms(unknown_terms));
    }

    Ok(())
}

fn find_contradictions(
    parsed: &serde_json::Value,
    analysis: &AnalysisResult,
) -> Vec<String> {
    let mut contradictions = Vec::new();
    let primary_tree = &analysis.trees[0];

    if let Some(tokens) = parsed["tokens"].as_array() {
        for token_entry in tokens {
            let index = token_entry["index"].as_u64().unwrap_or(0) as usize;
            let text = token_entry["explanation"].as_str().unwrap_or("");

            if index < primary_tree.tokens.len() {
                let token = &primary_tree.tokens[index];
                let mf = &token.features.morphological;

                // Check tense contradiction
                if let Some(tense) = &mf.tense {
                    let opposite_tense = match tense.as_str() {
                        "past" => ["present", "future"],
                        "present" => ["past"],
                        _ => [],
                    };
                    for opp in &opposite_tense {
                        if text.to_lowercase().contains(opp) {
                            contradictions.push(format!(
                                "Token {}: analysis says '{}' tense, but LLM mentions '{}'",
                                index, tense, opp
                            ));
                        }
                    }
                }

                // Check voice contradiction
                if let Some(voice) = &mf.voice {
                    if voice == "active" && text.to_lowercase().contains("passive") {
                        contradictions.push(format!(
                            "Token {}: analysis says 'active' voice, but LLM mentions 'passive'",
                            index
                        ));
                    }
                }

                // Check POS contradiction
                if let Some(pos) = token_entry["pos"].as_str() {
                    let analysis_pos = &mf.pos;
                    if !analysis_pos.contains(pos) && !pos.contains(analysis_pos) {
                        // This may be a terminology difference, not a contradiction
                        // Only flag if clearly different
                        if is_different_pos(analysis_pos, pos) {
                            contradictions.push(format!(
                                "Token {}: analysis says POS '{}', but LLM used '{}'",
                                index, analysis_pos, pos
                            ));
                        }
                    }
                }
            }
        }
    }

    contradictions
}
```

### 8.4 LLM Cost & Performance

```rust
/// LLM cost estimation and budget management.
struct LLMCostEstimator {
    /// Cost per 1K input tokens (USD)
    input_cost_per_1k: f64,
    /// Cost per 1K output tokens (USD)
    output_cost_per_1k: f64,
}

impl LLMCostEstimator {
    fn for_model(model: &str) -> Self {
        match model {
            "gpt-4" => Self { input_cost_per_1k: 0.03, output_cost_per_1k: 0.06 },
            "gpt-4o-mini" => Self { input_cost_per_1k: 0.0015, output_cost_per_1k: 0.006 },
            "claude-3.5-sonnet" => Self { input_cost_per_1k: 0.003, output_cost_per_1k: 0.015 },
            _ => Self { input_cost_per_1k: 0.01, output_cost_per_1k: 0.03 },
        }
    }

    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_cost_per_1k;
        input_cost + output_cost
    }
}

/// Estimated cost per sentence (10 words, intermediate level):
/// - Input: ~2,000 tokens (system prompt + analysis JSON + few-shot)
/// - Output: ~500 tokens
///
/// Model         | Cost per sentence | Cost per 1M sentences
/// ──────────────┼───────────────────┼──────────────────────
/// gpt-4         | $0.09             | $90,000
/// gpt-4o-mini   | $0.006            | $6,000
/// claude-3.5    | $0.0135           | $13,500

/// Recommended LLM cache strategy:
/// - Cache key: hash(AnalysisResult + language + level + model)
/// - TTL: 24 hours (configurable)
/// - Expected hit rate: 60-80% for educational content
/// - Cache reduces cost by 60-80%
```

### 8.5 LLM Provider Abstraction

```rust
/// Abstract LLM provider with retry and fallback support.
trait LLMProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn supported_models(&self) -> Vec<String>;

    fn generate(
        &self,
        prompt: &LLMPrompt,
        config: &LLMConfig,
    ) -> Result<LLMResponse, LLMError>;

    /// Check if the provider service is healthy.
    fn health_check(&self) -> Result<(), LLMError>;
}

/// Retry configuration.
struct LLMRetryConfig {
    max_retries: u32,           // Default: 2
    base_delay_ms: u64,         // Default: 500
    max_delay_ms: u64,          // Default: 5000
    backoff_factor: f64,        // Default: 2.0 (exponential backoff)
}

/// Default retry strategy:
/// Retry 0: immediate
/// Retry 1: after 500ms
/// Retry 2: after 1000ms
/// Total max wait: ~1.5s

/// Supported providers:
/// 1. OpenAI (gpt-4, gpt-4o, gpt-4o-mini, gpt-3.5-turbo)
/// 2. Anthropic (claude-3.5-sonnet, claude-3-haiku)
/// 3. Custom (any OpenAI-compatible API endpoint)
/// 4. Noop (for testing, always returns error)

struct LLMProviderFactory;
impl LLMProviderFactory {
    fn create(config: &LLMConfig) -> Box<dyn LLMProvider> {
        match config.provider.as_str() {
            "openai" => Box::new(OpenAIProvider::new(config)),
            "anthropic" => Box::new(AnthropicProvider::new(config)),
            "custom" => Box::new(CustomProvider::new(config)),
            _ => Box::new(NoopProvider),
        }
    }
}
```

### 8.6 Teacher Mode

The teacher mode (prompt_style = "teacher_mode") transforms the explanation into a Socratic dialogue:

```text
System Prompt for Teacher Mode:

You are an Arabic grammar tutor using the Socratic method. Your goal is
to guide the student to discover the grammatical analysis themselves
through questions and hints.

CRITICAL RULES:
1. NEVER simply state the answer. Use questions to guide.
2. Start with the most basic concept and build up.
3. Provide hints when the student struggles.
4. Confirm correct answers positively.
5. Use the provided AnalysisResult to know the correct answer,
   but do not reveal it directly.

Interaction format:
- You receive the analysis AND the student's current understanding level.
- Ask one question at a time.
- After each student response, provide feedback and the next question.
- When all concepts are covered, provide a complete summary.

Current sentence to analyze: "{input_text}"
Analysis: {analysis_json}
Student level: {educational_level}
```

---

## 9. Localization Framework

### 9.1 Language Pack Architecture

```
language_packs/
├── en/
│   ├── pack.json              # Core language pack (labels, features, roles)
│   ├── templates/             # Template strings
│   │   ├── irab_entry.agos-tpl
│   │   ├── overview.agos-tpl
│   │   ├── construction_*.agos-tpl
│   │   └── ...
│   └── examples.json          # Example sentences for templates
├── ar/
│   ├── pack.json
│   ├── templates/
│   └── examples.json
├── ur/
│   ├── pack.json
│   ├── templates/
│   └── examples.json
├── ms/
│   ├── pack.json
│   ├── templates/
│   └── examples.json
└── ... (community contributions)
```

### 9.2 Language Pack Schema (Complete)

```json
{
    "language": "en",
    "meta": {
        "name": "English",
        "native_name": "English",
        "ietf_tag": "en",
        "direction": "ltr",
        "plural_forms": "one_other",
        "version": "1.0.0",
        "author": "AGOS Localization Team",
        "requires_templates": true,
        "fallback_languages": []
    },
    "labels": {
        "section_grammatical_analysis": "Grammatical Analysis",
        "section_irab_breakdown": "I'rab (إعراب) Breakdown",
        "section_sentence_type": "Sentence Type",
        "section_overview": "Overview",
        "section_constructions": "Grammatical Constructions",
        "section_flags": "Grammatical Flags",
        "section_evidence": "Evidence Trail",
        "section_ambiguity": "Alternative Interpretations",
        "section_teaching_tips": "Learning Tips",
        "section_references": "References",

        "column_token": "Token",
        "column_root": "Root",
        "column_pos": "Part of Speech",
        "column_features": "Features",
        "column_role": "Syntactic Role",
        "column_explanation": "Explanation",
        "column_case": "Case",
        "column_state": "State",

        "label_hide": "Hide",
        "label_show": "Show",
        "label_more": "More",
        "label_less": "Less",
        "label_primary_analysis": "Primary Analysis",
        "label_alternative": "Alternative",
        "label_source": "Source",
        "label_confidence": "Confidence",
        "label_unknown": "Unknown",
        "label_none": "None",

        "notice_llm_enhanced": "AI-Enhanced Explanations",
        "notice_llm_disclaimer": "AI-generated explanations may contain errors. Always verify against the deterministic analysis.",
        "notice_evidence_trail": "This explanation is backed by a complete evidence trail of grammatical rule applications.",
        "notice_ambiguity": "This sentence has multiple valid grammatical interpretations."
    },
    "pos": {
        "verb": "Verb",
        "noun": "Noun",
        "particle": "Particle",
        "pronoun": "Pronoun",
        "adjective": "Adjective",
        "adverb": "Adverb",
        "preposition": "Preposition",
        "conjunction": "Conjunction",
        "proper_noun": "Proper Noun",
        "interrogative": "Interrogative",
        "unknown": "Unknown"
    },
    "features": {
        "gender": {
            "label": "Gender",
            "category": "inflectional",
            "values": {
                "masculine": "Masculine",
                "feminine": "Feminine",
                "common": "Common"
            }
        },
        "number": {
            "label": "Number",
            "category": "inflectional",
            "values": {
                "singular": "Singular",
                "dual": "Dual",
                "plural": "Plural"
            }
        },
        "person": {
            "label": "Person",
            "category": "inflectional",
            "values": {
                "first": "First Person",
                "second": "Second Person",
                "third": "Third Person"
            }
        },
        "tense": {
            "label": "Tense",
            "category": "inflectional",
            "values": {
                "past": "Past (الماضي)",
                "present": "Present (المضارع)",
                "imperative": "Imperative (الأمر)"
            }
        },
        "mood": {
            "label": "Mood",
            "category": "inflectional",
            "values": {
                "indicative": "Indicative (الرفع)",
                "subjunctive": "Subjunctive (النصب)",
                "jussive": "Jussive (الجزم)",
                "energetic": "Energetic (التوكيد)"
            }
        },
        "voice": {
            "label": "Voice",
            "category": "inflectional",
            "values": {
                "active": "Active (معلوم)",
                "passive": "Passive (مجھول)"
            }
        },
        "case": {
            "label": "Case",
            "category": "inflectional",
            "values": {
                "nominative": "Nominative (الرفع)",
                "accusative": "Accusative (النصب)",
                "genitive": "Genitive (الجر)"
            }
        },
        "state": {
            "label": "State",
            "category": "inflectional",
            "values": {
                "definite": "Definite (معرفة)",
                "indefinite": "Indefinite (نكرة)"
            }
        },
        "verb_form": {
            "label": "Verb Form",
            "category": "derivational",
            "values": {}
        },
        "noun_type": {
            "label": "Noun Type",
            "category": "derivational",
            "values": {
                "masdar": "Verbal Noun (مصدر)",
                "ism_fa'il": "Active Participle (اسم فاعل)",
                "ism_maf'ul": "Passive Participle (اسم مفعول)",
                "ism_zarf": "Noun of Time/Place (اسم ظرف)",
                "ism_alah": "Noun of Instrument (اسم آلة)",
                "ism_tafdil": "Comparative/Superlative (اسم تفضيل)",
                "ism_mushtaq": "Derived Noun (اسم مشتق)",
                "jamid": "Primary Noun (اسم جامد)"
            }
        },
        "transitivity": {
            "label": "Transitivity",
            "category": "derivational",
            "values": {
                "transitive": "Transitive (متعدي)",
                "intransitive": "Intransitive (لازم)",
                "ditransitive": "Ditransitive (متعدي إلى مفعولين)"
            }
        },
        "root_type": {
            "label": "Root Type",
            "category": "derivational",
            "values": {
                "sound": "Sound (صحيح)",
                "mithal": "Assimilated (مثال)",
                "ajwaf": "Hollow (أجوف)",
                "naqis": "Defective (ناقص)",
                "hamzated": "Hamzated (مهموز)",
                "doubled": "Doubled (مضاعف)",
                "lafif": "Complex Weak (لفيف)",
                "quadriliteral": "Quadriliteral (رباعي)"
            }
        }
    },
    "roles": {
        "fi'l": "Verb (فعل)",
        "fa'il": "Subject (فاعل)",
        "mubtada": "Topic (مبتدأ)",
        "khabar": "Predicate (خبر)",
        "maf'ul_bi-hi": "Direct Object (مفعول به)",
        "maf'ul_mutlaq": "Absolute Object (مفعول مطلق)",
        "maf'ul_fih": "Adverb of Time/Place (مفعول فيه)",
        "maf'ul_lahu": "Object of Purpose (مفعول لأجله)",
        "maf'ul_ma'ahu": "Accompanied Object (مفعول معه)",
        "idafa": "Construct State (إضافة)",
        "mudaf": "First Term of Idafa (مضاف)",
        "mudaf_ilayh": "Second Term of Idafa (مضاف إليه)",
        "harf_jarr": "Preposition (حرف جر)",
        "majrur": "Governed by Preposition (مجرور)",
        "harf_nasb": "Subjunctive Particle (حرف نصب)",
        "harf_jazm": "Jussive Particle (حرف جزم)",
        "na'at": "Adjective (نعت)",
        "man'ut": "Described Noun (منعوت)",
        "hal": "Circumstantial Accusative (حال)",
        "tamyiz": "Specification (تمييز)",
        "zarf": "Adverb (ظرف)",
        "ta'kid": "Emphasizer (توكيد)",
        "badal": "Apposition (بدل)",
        "istithna": "Exception (استثناء)",
        "nida": "Vocative (نداء)",
        "shart": "Condition (شرط)",
        "jaza": "Result (جزاء)",
        "qasam": "Oath (قسم)",
        "jawab": "Answer (جواب)",
        "fi'l_shart": "Conditional Verb (فعل الشرط)",
        "sila": "Relative Clause Link (صلة)",
        "rabit": "Connector (رابط)"
    },
    "sentence_types": {
        "jumlah_fi'liyyah": "Verbal Sentence (جملة فعلية)",
        "jumlah_ismiyyah": "Nominal Sentence (جملة اسمية)",
        "jumlah_shartiyyah": "Conditional Sentence (جملة شرطية)",
        "jumlah_zarfiyyah": "Adverbial Clause (جملة ظرفية)",
        "jumlah_kana": "Kana Sentence (جملة كان)",
        "jumlah_inna": "Inna Sentence (جملة إن)",
        "jumlah_qasam": "Oath Construction (جملة قسم)",
        "jumlah_istithna": "Exceptive Sentence (جملة استثناء)"
    },
    "errors": {
        "SUBJECT_VERB_PERSON_MISMATCH": "The subject and verb do not agree in person.",
        "SUBJECT_VERB_GENDER_MISMATCH": "The subject and verb do not agree in gender.",
        "SUBJECT_VERB_NUMBER_MISMATCH": "The subject and verb do not agree in number.",
        "CASE_MISMATCH": "The expected case does not match the assigned case.",
        "MOOD_MISMATCH": "The verb mood does not match the governing particle's requirement.",
        "SUBJECT_VERB_ORDER_INVERSION": "The subject precedes the verb, which may affect agreement.",
        "UNRESOLVABLE_AMBIGUITY": "This sentence has unresolvable grammatical ambiguity.",
        "IMPLICIT_ELEMENT_OMITTED": "A grammatically required element is implied but not explicitly stated."
    },
    "constructions": {
        "idafa_description": "A possessive construction: '{0} of {1}'. The first term (مضاف) is in the construct state and cannot take the definite article. The second term (مضاف إليه) is in the genitive case (مجرور).",
        "naat_description": "'{0}' describes '{1}'. The adjective agrees with the noun in gender, number, case, and definiteness.",
        "shart_description": "A conditional sentence using '{0}': if {1}, then {2}. The verb(s) in both clauses are in the jussive mood (مجزوم).",
        "kana_description": "This sentence begins with {0} (one of kana and her sisters). {0} raises the subject (اسمها) to the nominative and assigns the accusative case to the predicate (خبرها).",
        "inna_description": "This sentence begins with {0} (one of inna and her sisters). {0} assigns the accusative case to the subject (اسمها) and the nominative to the predicate (خبرها)."
    },
    "stages": {
        "MOD-03": "Tokenization",
        "MOD-04": "Morphological Analysis",
        "MOD-05": "Syntactic Parsing",
        "MOD-06": "GIR Construction",
        "MOD-07": "Rule Application",
        "MOD-08": "Knowledge Graph Resolution",
        "MOD-09": "Bytecode Generation",
        "MOD-10": "GVM Execution"
    }
}
```

### 9.3 Pluralization Rules by Language

```rust
/// CLDR-compliant pluralization rule evaluator.
trait PluralRule {
    fn plurals() -> Vec<&'static str>;  // e.g., ["one", "two", "few", "many", "other"]
    fn evaluate(count: i64) -> &'static str;
}

struct ArabicPlural;
impl PluralRule for ArabicPlural {
    fn plurals() -> Vec<&'static str> { vec!["one", "two", "few", "many", "other"] }

    fn evaluate(count: i64) -> &'static str {
        match count {
            1 => "one",
            2 => "two",
            3..=10 => "few",
            11..=99 => "many",
            0 | 100.. => "other",
            _ => "other",
        }
    }
}

struct EnglishPlural;
impl PluralRule for EnglishPlural {
    fn plurals() -> Vec<&'static str> { vec!["one", "other"] }

    fn evaluate(count: i64) -> &'static str {
        match count {
            1 => "one",
            _ => "other",
        }
    }
}

struct MalayPlural;
impl PluralRule for MalayPlural {
    fn plurals() -> Vec<&'static str> { vec!["other"] }

    fn evaluate(_count: i64) -> &'static str { "other" }
}

/// Get plural rule for a given language.
fn plural_rule_for_language(language: &str) -> Box<dyn PluralRule> {
    match language {
        "ar" => Box::new(ArabicPlural),
        "en" | "fr" | "de" | "es" => Box::new(EnglishPlural),
        "ms" | "id" => Box::new(MalayPlural),
        "ur" => Box::new(UrduPlural),
        "tr" => Box::new(MalayPlural),
        _ => Box::new(EnglishPlural),  // Default to English plural rules
    }
}
```

### 9.4 Translation Workflow

```text
Language Pack Creation Workflow:

1. CREATE LANGUAGE PACK
   - Copy en/pack.json as template
   - Translate all label, pos, feature, role, sentence_type, and error strings
   - Update meta fields (language, native_name, direction, plural_forms)
   - For RTL languages: set direction = "rtl"

2. CREATE TEMPLATES (optional but recommended)
   - Copy en/templates/ directory
   - Translate template strings while preserving variable syntax {var}
   - Adjust plural forms to match the language's plural system
   - Test each template with sample data

3. VALIDATE LANGUAGE PACK
   Tool: agos i18n validate --language=ur
   Checks:
   - All required keys present
   - All template variables resolvable
   - Plural forms match the language's CLDR plural rules
   - No missing translation strings

4. INSTALL LANGUAGE PACK
   Tool: agos i18n install ./ur/pack.json
   - Copies to language_packs/ directory
   - Registers with TemplateRegistry
   - Ready for use by Explanation Engine

5. TEST LOCALIZATION
   Tool: agos explain --text="السلام عليكم" --language=ur --format=text
   - Verify all output is in Urdu script
   - Verify RTL/LTR handling
   - Verify plural forms are correct
   - Verify Arabic script rendering
```

### 9.5 RTL Text Handling

```rust
/// Handle bidirectional text for Arabic-script languages.
struct BidiHandler {
    /// Whether the output language is RTL
    is_rtl: bool,

    /// Unicode bidi control characters
    lrm: char,   // LEFT-TO-RIGHT MARK (U+200E)
    rlm: char,   // RIGHT-TO-LEFT MARK (U+200F)
    lre: char,   // LEFT-TO-RIGHT EMBEDDING (U+202A)
    rle: char,   // RIGHT-TO-LEFT EMBEDDING (U+202B)
    pdf: char,   // POP DIRECTIONAL FORMATTING (U+202C)
}

impl BidiHandler {
    fn for_language(language: &str) -> Self {
        let rtl_languages = ["ar", "ur", "fa", "he", "ps"];
        Self {
            is_rtl: rtl_languages.contains(&language),
            lrm: '\u{200E}',
            rlm: '\u{200F}',
            lre: '\u{202A}',
            rle: '\u{202B}',
            pdf: '\u{202C}',
        }
    }

    /// Wrap mixed-script text to ensure correct display order.
    fn wrap_mixed_text(&self, arabic_text: &str, english_text: &str) -> String {
        if self.is_rtl {
            // Arabic text first, then English in left-to-right embedding
            format!(
                "{rle}{}{pdf} {lre}{}{pdf}",
                arabic_text, english_text,
                rle = self.rle, lre = self.lre, pdf = self.pdf,
            )
        } else {
            // English text first, then Arabic in right-to-left embedding
            format!(
                "{} {rle}{}{pdf}",
                english_text, arabic_text,
                rle = self.rle, pdf = self.pdf,
            )
        }
    }

    /// Ensure correct ordering for I'rab tables.
    fn format_irab_table(&self, entries: &[IrabEntry]) -> Vec<IrabEntry> {
        if self.is_rtl {
            // RTL tables: token on the right, explanation on the left
            entries.iter().cloned().map(|mut entry| {
                // Token text is Arabic, ensure RTL
                entry.token = format!("{}{}", self.rlm, entry.token);
                entry
            }).collect()
        } else {
            entries.to_vec()
        }
    }

    /// Direction attribute for HTML rendering.
    fn html_dir_attr(&self) -> &str {
        if self.is_rtl { "dir=\"rtl\"" } else { "dir=\"ltr\"" }
    }

    /// CSS direction property.
    fn css_direction(&self) -> &str {
        if self.is_rtl { "rtl" } else { "ltr" }
    }
}
```

---

## 10. Output Format Specifications

### 10.1 Format Selection Guide

| Use Case | Recommended Format | Rationale |
|----------|-------------------|-----------|
| API response | JSON | Structured, machine-readable |
| CLI display | Text | Simple, readable in terminal |
| Web display | HTML | Rich formatting, interactive |
| Printable report | PDF | Layout control, fonts |
| Mobile app | JSON + HTML | Flexible rendering |
| Screen reader | Text + ARIA | Accessibility |
| Embed in LMS | JSON | Standard API consumption |
| Export to spreadsheet | CSV (from JSON) | Data analysis |

### 10.2 JSON Output Specification

The JSON output is the canonical serialization format for ExplanationOutput (IR-11). See SPEC-0001-C5 §12 for the complete schema.

**Additional JSON conventions for SPEC-0501:**

```json
{
    "spec": "SPEC-0001/IR-11",
    "version": "1.0",
    "meta": {
        "educational_level": "intermediate",
        "terminology_style": "dual",
        "has_ambiguity": false,
        "has_llm": false,
        "template_version": "1.0.0"
    },
    "metadata": { ... },
    "input_text": "كتب محمد رسالة",
    "overview": "This is a verbal sentence (jumlah fi'liyyah)...",
    "sentence_type": "Verbal Sentence (جملة فعلية)",
    "irab_breakdown": [ ... ],
    "constructions": [ ... ],
    "flags": [ ... ],
    "evidence": [ ... ],
    "teaching_tips": [
        "Notice how the verb comes before the subject in Arabic verbal sentences.",
        "The tanwin (double vowel) on the last letter of a noun indicates it is indefinite (نكرة)."
    ],
    "ambiguity": {
        "has_alternatives": false,
        "notes": [],
        "alternatives": []
    },
    "references": {
        "rules_applied": ["basra-0101", "basra-0203", "basra-0301"],
        "kb_used": ["KB-0001", "KB-0002", "KB-0005"]
    }
}
```

### 10.3 Text Output Specification

**Plain text I'rab output format:**

```
╔══════════════════════════════════════════════════════════════╗
║  GRAMMATICAL ANALYSIS                                        ║
║  ─────────────────────                                       ║
║  Language: English   Level: Intermediate   Format: Text      ║
╚══════════════════════════════════════════════════════════════╝

OVERVIEW
────────────────────────────────────────────────────────────────
This is a verbal sentence (jumlah fi'liyyah) — it begins with
a verb. It consists of 3 words. The sentence follows the
standard Arabic word order for past tense: Verb → Subject →
Object.

SENTENCE TYPE: Verbal Sentence (جملة فعلية)

I'RAB BREAKDOWN (إعراب)
────────────────────────────────────────────────────────────────
  1. كَتَبَ  —  Verb
     Root: كتب
     Role: Verb (فعل)
     Gender: Masculine  |  Number: Singular  |  Person: Third
     Tense: Past (الماضي)  |  Voice: Active (معلوم)  |  Form: I
     → Third person masculine singular past tense verb in the
       active voice, Form I. It agrees with a third masculine
       singular subject.

  2. مُحَمَّدٌ  —  Proper Noun
     Root: حمد
     Role: Subject (فاعل)
     Gender: Masculine  |  Number: Singular
     Case: Nominative (الرفع)  |  State: Indefinite (نكرة)
     → The subject (fa'il) of the verb. A masculine singular
       proper noun in the nominative case, indefinite.

  3. رِسَالَةً  —  Noun
     Root: ر س ل
     Role: Direct Object (مفعول به)
     Gender: Feminine  |  Number: Singular
     Case: Accusative (النصب)  |  State: Indefinite (نكرة)
     → The direct object (maf'ul bi-hi) — the entity that
       receives the action. A feminine singular noun in the
       accusative case, indefinite.

GRAMMATICAL CONSTRUCTIONS
────────────────────────────────────────────────────────────────
  • Verbal Sentence — A verbal sentence beginning with a past
    tense verb, followed by its subject and object. This is the
    canonical word order for Arabic verbal sentences.

────────────────────────────────────────────────────────────────
  Generated by AGOS Explanation Engine v1.0
  Language: en | Level: intermediate | Format: text
────────────────────────────────────────────────────────────────
```

### 10.4 HTML Output Specification

**Complete HTML structure:**

```html
<!DOCTYPE html>
<html lang="en" dir="ltr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Grammatical Analysis</title>
    <link rel="stylesheet" href="agos-explanation.css">
</head>
<body>
    <div class="agos-explanation" role="main" aria-label="Grammatical Analysis">
        <!-- Header -->
        <header class="explanation-header">
            <h1 class="header-title">Grammatical Analysis</h1>
            <div class="header-meta">
                <span class="meta-language">English</span>
                <span class="meta-level">Intermediate</span>
                <span class="meta-format">HTML</span>
            </div>
        </header>

        <!-- Overview -->
        <section class="overview" aria-labelledby="overview-title">
            <h2 id="overview-title">Overview</h2>
            <p class="overview-text">...</p>
            <div class="sentence-type">
                <span class="type-label">Sentence Type:</span>
                <span class="type-value">Verbal Sentence (جملة فعلية)</span>
            </div>
        </section>

        <!-- I'rab Table -->
        <section class="irab-section" aria-labelledby="irab-title">
            <h2 id="irab-title">I'rab (إعراب) Breakdown</h2>
            <table class="irab-table" role="table">
                <thead>
                    <tr>
                        <th scope="col" class="col-token">#</th>
                        <th scope="col" class="col-token">Token</th>
                        <th scope="col" class="col-root">Root</th>
                        <th scope="col" class="col-pos">POS</th>
                        <th scope="col" class="col-role">Role</th>
                        <th scope="col" class="col-features">Features</th>
                        <th scope="col" class="col-explanation">Explanation</th>
                    </tr>
                </thead>
                <tbody>
                    <!-- Rows generated per token -->
                </tbody>
            </table>
        </section>

        <!-- Constructions -->
        <section class="constructions-section" aria-labelledby="constructions-title">
            <h2 id="constructions-title">Grammatical Constructions</h2>
            <ul class="constructions-list">
                <!-- Construction items -->
            </ul>
        </section>

        <!-- Flags -->
        <section class="flags-section" aria-labelledby="flags-title">
            <h2 id="flags-title">Flags</h2>
            <ul class="flags-list">
                <!-- Flag items -->
            </ul>
        </section>

        <!-- LLM Notice -->
        <div class="llm-notice" role="note">
            AI-enhanced explanations may contain errors.
            Always verify against the deterministic analysis.
        </div>

        <!-- Footer -->
        <footer class="explanation-footer">
            <p>Generated by AGOS Explanation Engine</p>
        </footer>
    </div>
</body>
</html>
```

**CSS Theme Variables:**

```css
:root {
    /* Color palette */
    --agos-primary: #1a1a2e;
    --agos-accent: #e94560;
    --agos-bg: #f8f9fa;
    --agos-surface: #ffffff;
    --agos-text: #1a1a2e;
    --agos-text-secondary: #4a5568;
    --agos-border: #dee2e6;
    --agos-hover: #e9ecef;

    /* Feature colors */
    --feature-inflectional: #dbeafe;
    --feature-derivational: #fef3c7;
    --feature-semantic: #d1fae5;
    --feature-prosodic: #e0e7ff;
    --feature-orthographic: #fce7f3;

    /* Flag colors */
    --flag-error: #dc2626;
    --flag-warning: #d97706;
    --flag-info: #2563eb;

    /* Typography */
    --font-sans: 'Segoe UI', system-ui, -apple-system, sans-serif;
    --font-arabic: 'Traditional Arabic', 'Scheherazade New', 'Amiri', serif;
    --font-mono: 'Cascadia Code', 'Fira Code', monospace;

    /* Spacing */
    --spacing-xs: 0.25rem;
    --spacing-sm: 0.5rem;
    --spacing-md: 1rem;
    --spacing-lg: 2rem;

    /* Border radius */
    --radius-sm: 4px;
    --radius-md: 8px;
    --radius-lg: 12px;
}

/* RTL overrides */
[dir="rtl"] .irab-table th,
[dir="rtl"] .irab-table td {
    text-align: right;
}

[dir="rtl"] .token-arabic {
    font-family: var(--font-arabic);
    font-size: 1.4em;
}
```

---

## 11. PDF Rendering

### 11.1 PDF Layout Specification

```
Page: A4 (210mm × 297mm)
Margins: 20mm all sides
Font: Noto Naskh Arabic (for Arabic text), Noto Sans (for Latin text)

┌─────────────────────────────────────────────────────────────┐
│  HEADER                                                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ AGOS Logo          Grammatical Analysis   2026-07-15  │  │
│  │                     Language: English                  │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  OVERVIEW                                                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ This is a verbal sentence (jumlah fi'liyyah)...       │  │
│  │                                                       │  │
│  │ Sentence Type: Verbal Sentence (جملة فعلية)           │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  I'RAB BREAKDOWN                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ #  │ Token   │ Root │ POS │ Role │ Features │ Explain│  │
│  │────┼─────────┼──────┼─────┼──────┼──────────┼────────│  │
│  │ 1  │ كتب     │ كتب  │Verb │ Fi'l │ Masc, Sg │ ...    │  │
│  │ 2  │ محمد    │ حمد  │PN   │ Fa'il│ Nom, Ind │ ...    │  │
│  │ 3  │ رسالة   │ ر س ل│Noun │ Maf'ul│ Fem, Acc │ ...    │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  CONSTRUCTIONS                                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ • Verbal Sentence — A verbal sentence beginning...    │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  FOOTER                                                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ Generated by AGOS v1.0 | Page 1 of 1                  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 11.2 PDF Generation Pipeline

```rust
/// PDF generation configuration.
struct PDFConfig {
    /// Page size (default: A4)
    page_size: PageSize,

    /// Margins in mm
    margins: Margins,

    /// Font configuration
    fonts: FontConfig,

    /// Whether to include header and footer
    show_header: bool,
    show_footer: bool,

    /// Color scheme
    color_scheme: PDFColorScheme,
}

struct FontConfig {
    arabic_font: String,           // "NotoNaskhArabic-Regular.ttf"
    latin_font: String,            // "NotoSans-Regular.ttf"
    bold_font: String,             // "NotoSans-Bold.ttf"
    italic_font: String,           // "NotoSans-Italic.ttf"
    size_body: u8,                 // 10pt
    size_header: u8,               // 14pt
    size_title: u8,                // 18pt
}

struct PDFColorScheme {
    primary: Color,                // Dark navy
    accent: Color,                 // Red accent
    background: Color,             // Light gray
    text: Color,                   // Dark
    text_secondary: Color,         // Gray
    border: Color,                 // Light gray
    feature_colors: HashMap<String, Color>,
    flag_colors: HashMap<String, Color>,
}

/// PDF generation process:
/// 1. Create document with page settings
/// 2. Add header (logo, title, metadata)
/// 3. Add overview section
/// 4. Add I'rab table (may span multiple pages)
/// 5. Add constructions section
/// 6. Add flags section
/// 7. Add evidence section (if included)
/// 8. Add footer (page numbers, generation info)
/// 9. Embed fonts and render Arabic text
/// 10. Finalize PDF binary
```

### 11.3 Arabic Font Requirements

| Requirement | Specification |
|------------|---------------|
| Arabic script | Full Unicode Arabic block (U+0600–U+06FF) support |
| Quranic symbols | Support for U+06D6–U+06ED |
| Diacritics | Proper tashkeel rendering (fatha, kasra, damma, sukun, shadda, etc.) |
| Ligatures | Required lam-alif and other standard ligatures |
| Font weight | Regular + Bold for Arabic (for emphasis) |
| Recommended fonts | Noto Naskh Arabic, Amiri, Scheherazade New, Traditional Arabic |
| Embedding | Fonts MUST be embedded in PDF for consistent rendering |
| Size | Embedded Arabic font: ~1-3 MB TTF/OTF |

---

## 12. Accessibility

### 12.1 WCAG 2.1 AA Compliance

| WCAG Criterion | AGOS Implementation |
|----------------|-------------------|
| **1.1.1 Non-text Content** | All non-text content (icons, flags) has text alternatives |
| **1.3.1 Info and Relationships** | HTML uses semantic elements (table, section, header, footer) |
| **1.3.2 Meaningful Sequence** | I'rab entries presented in reading order |
| **1.4.1 Use of Color** | Flags use icons AND color (✗ error, ⚠ warning, ℹ info) |
| **1.4.3 Contrast (Minimum)** | All text meets 4.5:1 contrast ratio against background |
| **1.4.4 Resize Text** | HTML output uses relative units (em, rem) |
| **1.4.5 Images of Text** | Text is used instead of images of text |
| **2.4.2 Page Titled** | HTML page has descriptive title |
| **2.4.4 Link Purpose (In Context)** | All links have descriptive text |
| **2.4.6 Headings and Labels** | Clear heading hierarchy (h1 → h2 → h3) |
| **3.1.1 Language of Page** | HTML lang attribute set correctly |
| **3.3.2 Labels or Instructions** | All form elements have labels (for interactive output) |
| **4.1.2 Name, Role, Value** | ARIA roles assigned to custom elements |

### 12.2 Screen Reader Optimization

```html
<!-- Screen reader optimized I'rab table -->
<table class="irab-table" role="table" aria-label="Word-by-word grammatical breakdown">
    <caption class="sr-only">I'rab breakdown showing each word's root, part of speech, syntactic role, features, and explanation</caption>
    <thead>
        <tr>
            <th scope="col" id="h-token">Word</th>
            <th scope="col" id="h-root">Root</th>
            <th scope="col" id="h-pos">Part of Speech</th>
            <th scope="col" id="h-role">Syntactic Role</th>
            <th scope="col" id="h-features">Features</th>
            <th scope="col" id="h-explanation">Explanation</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td headers="h-token" lang="ar" dir="rtl">كَتَبَ</td>
            <td headers="h-root" lang="ar">كتب</td>
            <td headers="h-pos">Verb</td>
            <td headers="h-role">Verb (fi'l)</td>
            <td headers="h-features">
                <ul class="sr-only">
                    <li>Gender: Masculine</li>
                    <li>Number: Singular</li>
                    <li>Person: Third</li>
                    <li>Tense: Past</li>
                </ul>
            </td>
            <td headers="h-explanation">Third person masculine singular past tense verb...</td>
        </tr>
    </tbody>
</table>

<!-- Screen reader only CSS -->
<style>
    .sr-only {
        position: absolute;
        width: 1px;
        height: 1px;
        padding: 0;
        margin: -1px;
        overflow: hidden;
        clip: rect(0, 0, 0, 0);
        white-space: nowrap;
        border-width: 0;
    }
</style>
```

### 12.3 Simplified Text Mode

```rust
/// Generate simplified text for screen readers or low-literacy users.
/// Features:
/// - Shorter sentences (max 15 words per sentence)
/// - Simpler vocabulary (Flesch-Kincaid grade 5 equivalent)
/// - No parenthetical Arabic terms
/// - Punctuation expanded (e.g., "—" → "is")
/// - Numbers spelled out
fn generate_simplified_text(entry: &IrabEntry) -> String {
    let role = entry.syntactic_role.as_deref()
        .map(|r| simplify_role(r))
        .unwrap_or("a word");
    let features = entry.features.iter()
        .take(2)  // Only 2 most important features
        .map(|f| format!("{} {}", f.name.to_lowercase(), f.value.to_lowercase()))
        .collect::<Vec<String>>()
        .join(", ");

    format!(
        "{} is {} with {}. {}",
        entry.token,
        role,
        features,
        simplify_explanation_for_accessibility(&entry.explanation),
    )
}

fn simplify_role(role: &str) -> &str {
    match role {
        r if r.contains("Verb") => "a verb",
        r if r.contains("Subject") => "the subject",
        r if r.contains("Object") => "the object",
        r if r.contains("Preposition") => "a preposition",
        r if r.contains("Adjective") => "an adjective",
        r if r.contains("Topic") => "the topic",
        r if r.contains("Predicate") => "the predicate",
        _ => "a word",
    }
}
```

### 12.4 Multimodal Output

For applications that support multiple output modes simultaneously:

```json
{
    "output": {
        "text": {
            "content": "...",            // Plain text
            "reading_time_seconds": 45,
            "word_count": 180,
            "flesch_kincaid_grade": 8.5
        },
        "simplified": {
            "content": "...",            // Simplified text for accessibility
            "reading_time_seconds": 30,
            "word_count": 120,
            "flesch_kincaid_grade": 5.0
        },
        "audio": {
            "url": "/audio/req-abc.mp3",  // TTS audio (generated externally)
            "duration_seconds": 60,
            "format": "mp3"
        },
        "braille": {
            "content": "...",            // Braille-ready text
            "cells": 240
        }
    }
}
```

---

## 13. Plugin Development Guide

### 13.1 Explanation Plugin Types

| Plugin Type | Interface | Use Case | Example |
|------------|-----------|----------|---------|
| **Token explainer** | `explain_token()` | Custom explanation for specific tokens | Quranic word explanations |
| **Overview augmenter** | `explain_overview()` | Add context to sentence overview | Historical/geographical context |
| **Construction detector** | `custom_constructions()` | New construction types | Special Quranic constructions |
| **Template provider** | `register_templates()` | Custom template language | Gamification templates |
| **LLM provider** | `LLMProvider` | Custom LLM backends | Local LLM integration |
| **Format renderer** | `render()` | New output formats | SVG, Markdown, LaTeX |
| **Educational adapter** | `adapt_level()` | Custom learning level | Child-appropriate explanations |
| **Accessibility adapter** | `accessify()` | Custom accessibility | Dyslexia-friendly formatting |

### 13.2 Plugin Development SDK

```rust
/// The ExplanationPlugin trait that all explanation plugins must implement.
#[agos_plugin]
trait ExplanationPlugin: Send + Sync {
    /// Unique plugin identifier.
    fn plugin_id(&self) -> &str;

    /// Plugin type classification.
    fn plugin_type(&self) -> ExplanationPluginType;

    /// Priority in the plugin chain (0-255, higher = applied first).
    fn priority(&self) -> u8;

    /// Languages supported by this plugin (empty = all languages).
    fn supported_languages(&self) -> Vec<String>;

    /// Formats supported by this plugin (empty = all formats).
    fn supported_formats(&self) -> Vec<OutputFormat>;

    /// Augment or replace the explanation for a single token.
    /// Return Some(text) to replace/augment, None to skip.
    fn explain_token(
        &self,
        token: &AnalysisToken,
        context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError>;

    /// Augment or replace the overview text.
    fn explain_overview(
        &self,
        result: &AnalysisResult,
        context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError>;

    /// Register custom construction types detected by this plugin.
    fn custom_constructions(&self) -> Vec<ConstructionDef>;

    /// Register custom templates.
    fn register_templates(&self, registry: &mut TemplateRegistry) -> Result<(), PluginError>;

    /// Initialize the plugin (called once at load time).
    fn init(&mut self, config: &PluginConfig) -> Result<(), PluginError>;

    /// Shutdown the plugin (called at unload time).
    fn shutdown(&mut self) -> Result<(), PluginError>;
}

/// Context passed to plugins during explanation generation.
struct ExplanationContext {
    /// Request context
    request_id: String,
    language: String,
    format: OutputFormat,
    educational_level: EducationalLevel,

    /// Full analysis data
    analysis: AnalysisResult,
    primary_tree: AnalysisTree,

    /// Template registry and language pack
    template_registry: TemplateRegistry,
    language_pack: LanguagePack,

    /// Helper methods
    log: Box<dyn Fn(LogLevel, &str)>,
    cache: CacheManager,
}
```

### 13.3 Plugin Examples

**Example 1: Quranic Word Plugin**

```rust
/// Plugin that provides specialized explanations for Quranic vocabulary.
struct QuranicExplanationPlugin {
    quranic_words: HashMap<String, QuranicWordInfo>,
}

#[derive(Debug)]
struct QuranicWordInfo {
    surah: String,
    ayah: u16,
    contextual_meaning: String,
    tafsir_reference: String,
}

impl ExplanationPlugin for QuranicExplanationPlugin {
    fn plugin_id(&self) -> &str { "quranic-explainer" }
    fn plugin_type(&self) -> ExplanationPluginType { ExplanationPluginType::TokenExplainer }
    fn priority(&self) -> u8 { 100 }

    fn init(&mut self, config: &PluginConfig) -> Result<(), PluginError> {
        // Load Quranic word index from bundled data
        self.quranic_words = load_quranic_word_index(config.path("words.json"))?;
        Ok(())
    }

    fn explain_token(
        &self,
        token: &AnalysisToken,
        context: &ExplanationContext,
    ) -> Result<Option<String>, PluginError> {
        // Check if this token is a Quranic word
        if let Some(info) = self.quranic_words.get(&token.text) {
            Ok(Some(format!(
                "In the Quranic context ({} {}), this word carries the meaning: '{}'. \
                 (Tafsir reference: {})",
                info.surah, info.ayah, info.contextual_meaning, info.tafsir_reference,
            )))
        } else {
            Ok(None)  // Not a Quranic word, skip
        }
    }

    fn custom_constructions(&self) -> Vec<ConstructionDef> {
        vec![ConstructionDef {
            name: "Quranic Oath Construction".into(),
            description_template: "A Quranic oath construction (قسم قرآني). \
                                   In the Quran, oaths frequently appear at the beginning of surahs \
                                   to draw attention to the importance of the following statement.".into(),
            condition: "sentence.type == 'jumlah_qasam'".into(),
        }]
    }

    fn register_templates(&self, registry: &mut TemplateRegistry) -> Result<(), PluginError> {
        registry.register("quranic_notice", "This word appears in {surah} {ayah}. {contextual_meaning}")?;
        Ok(())
    }

    fn explain_overview(&self, _result: &AnalysisResult, _context: &ExplanationContext) -> Result<Option<String>, PluginError> {
        Ok(None)  // Don't modify overview
    }

    fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
}
```

**Example 2: Urdu Language Plugin**

```rust
/// Plugin that provides Urdu-language explanations using custom templates.
struct UrduExplanationPlugin;

impl ExplanationPlugin for UrduExplanationPlugin {
    fn plugin_id(&self) -> &str { "explanation-urdu" }
    fn plugin_type(&self) -> ExplanationPluginType { ExplanationPluginType::TemplateProvider }
    fn priority(&self) -> u8 { 50 }
    fn supported_languages(&self) -> Vec<String> { vec!["ur".into()] }
    fn supported_formats(&self) -> Vec<OutputFormat> { vec![OutputFormat::Text, OutputFormat::Html] }

    fn register_templates(&self, registry: &mut TemplateRegistry) -> Result<(), PluginError> {
        registry.register("irab_entry", r#"
{?root}ریشہ: {root}{/root}
{?role}قسم: {role}{/role}
{?tense}زمانہ: {tense}{/tense}
{?case}حالت: {case}{/case}
{?features}خصوصیات: {#features}{value}{:separator}، {/features}{/features}
"#)?;
        Ok(())
    }

    fn explain_token(&self, _token: &AnalysisToken, _context: &ExplanationContext) -> Result<Option<String>, PluginError> {
        Ok(None)  // Use template-based generation
    }

    fn explain_overview(&self, _result: &AnalysisResult, _context: &ExplanationContext) -> Result<Option<String>, PluginError> {
        Ok(None)
    }

    fn custom_constructions(&self) -> Vec<ConstructionDef> { vec![] }
    fn init(&mut self, _config: &PluginConfig) -> Result<(), PluginError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
}
```

**Example 3: Gamification Plugin**

```rust
/// Plugin that generates quiz-like explanations for educational apps.
struct GamificationPlugin;

impl ExplanationPlugin for GamificationPlugin {
    fn plugin_id(&self) -> &str { "gamification" }
    fn plugin_type(&self) -> ExplanationPluginType { ExplanationPluginType::EducationalAdapter }
    fn priority(&self) -> u8 { 200 }  // Runs last

    fn explain_token(&self, token: &AnalysisToken, context: &ExplanationContext) -> Result<Option<String>, PluginError> {
        // Generate a fill-in-the-blank style quiz
        let mf = &token.features.morphological;
        let blanks: Vec<String> = vec![
            format!("Case: ____ (answer: {})", mf.case.as_deref().unwrap_or("?")),
            format!("Role: ____ (answer: {})", token.features.syntactic.role.as_deref().unwrap_or("?")),
        ];
        Ok(Some(format!(
            "🎯 Quiz: Analyze '{}'!\n{}\nHint: It is a {}.",
            token.text,
            blanks.join("\n"),
            mf.pos,
        )))
    }

    fn explain_overview(&self, _result: &AnalysisResult, _context: &ExplanationContext) -> Result<Option<String>, PluginError> {
        Ok(Some("🌟 Challenge: Try to identify all the grammatical constructions in this sentence!".into()))
    }

    fn custom_constructions(&self) -> Vec<ConstructionDef> { vec![] }
    fn register_templates(&self, _registry: &mut TemplateRegistry) -> Result<(), PluginError> { Ok(()) }
    fn init(&mut self, _config: &PluginConfig) -> Result<(), PluginError> { Ok(()) }
    fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
}
```

### 13.4 Plugin Packaging

```yaml
# example: urdu-explanation-plugin.yaml
api_version: "1.0"
id: "explanation-urdu"
name: "Urdu Explanation Plugin"
version: "1.0.0"
plugin_type: "explanation"
author:
  name: "Urdu Localization Team"
  email: "urdu@agos.org"
description: >
  Urdu language support for the AGOS Explanation Engine.
  Translates all grammatical terminology and templates to Urdu.

entry_point: "urdu-plugin.wasm"

# Plugin capabilities
capabilities:
  languages: ["ur"]
  formats: ["text", "html"]
  template_count: 35

# Dependencies
dependencies: []

# Resource requirements
resources:
  memory_max_mb: 16
  init_timeout_ms: 5000
  process_timeout_ms: 50

# Permissions
permissions:
  - read_cache
```

### 13.5 Plugin Chain Execution

```rust
/// Execute the explanation plugin chain in priority order.
fn execute_plugin_chain(
    plugins: &[Box<dyn ExplanationPlugin>],
    analysis: &ExplanationAnalysis,
    context: &ExplanationContext,
) -> Result<ExplanationAnalysis, PluginError> {
    let mut current = analysis.clone();

    for plugin in plugins.iter().sorted_by_key(|p| Reverse(p.priority())) {
        let start = Instant::now();

        // Apply token-level augmentations
        for entry in &mut current.irab_entries {
            if let Some(token) = find_token(entry.token.as_str(), &context.analysis) {
                if let Some(augmented) = plugin.explain_token(token, context)? {
                    entry.explanation = augmented;
                }
            }
        }

        // Apply overview augmentation
        if let Some(augmented) = plugin.explain_overview(&context.analysis, context)? {
            current.overview = augmented;
        }

        // Check timeout
        if start.elapsed() > context.plugin_timeout {
            return Err(PluginError::Timeout {
                plugin_id: plugin.plugin_id().into(),
                timeout_ms: context.plugin_timeout.as_millis() as u64,
            });
        }
    }

    Ok(current)
}
```

---

## 14. Testing & Quality Assurance

### 14.1 Test Categories

| Category | Tests | Scope | Target Coverage |
|----------|-------|-------|-----------------|
| **Unit** | 200+ | Template compilation, I'rab generation functions, localization | 95%+ |
| **Integration** | 100+ | Full AnalysisResult → ExplanationOutput pipeline | 90%+ |
| **Localization** | 50+ per language | All template strings, all feature names, plural forms | 100% of keys |
| **LLM Integration** | 30+ | Prompt generation, response validation, fallback behavior | 90%+ |
| **Educational Levels** | 30+ | All 3 levels, all 11+ POS types, all 20+ constructions | 100% of combinations |
| **Edge Cases** | 50+ | Empty input, maximum length, maximum ambiguity, RTL issues | 95%+ |
| **Performance** | 20+ | Latency budgets, throughput, memory limits | Pass/fail per target |
| **Accessibility** | 15+ | WCAG 2.1 AA checks, screen reader compatibility | 100% of criteria |

### 14.2 I'rab Accuracy Metrics

```rust
/// Evaluation metrics for I'rab generation accuracy.
struct IrabAccuracyMetrics {
    /// Overall accuracy score (0.0 - 1.0)
    overall: f64,

    /// Per-feature accuracy
    feature_accuracy: HashMap<String, f64>,

    /// Per-POS accuracy
    pos_accuracy: HashMap<String, f64>,

    /// Per-construction accuracy
    construction_accuracy: HashMap<String, f64>,

    /// Error analysis
    error_breakdown: HashMap<String, u32>,
}

/// I'rab accuracy targets:
/// - Overall accuracy:              > 99.5%
/// - POS identification:            > 99.9%
/// - Feature extraction:            > 99.0%
///   - Gender:                      > 99.5%
///   - Number:                      > 99.5%
///   - Person:                      > 99.5%
///   - Tense:                       > 99.8%
///   - Mood:                        > 99.0%
///   - Voice:                       > 99.5%
///   - Case:                        > 98.0% (most challenging)
///   - State (definiteness):        > 99.0%
/// - Syntactic role assignment:     > 98.0%
/// - Construction detection:        > 95.0%
/// - Template rendering:            > 99.9% (no unclosed variables)
/// - Localization completeness:     100% (all keys translated)
///
/// Measurement: Compare MOD-11 I'rab output against
/// a curated gold standard corpus of 5,000+ sentences
/// annotated by certified Arabic linguists.
```

### 14.3 Test Fixture Format

```json
{
    "spec": "SPEC-0501/test-fixture",
    "version": "1.0",
    "test_suite": "irab-generation-verbal",
    "description": "I'rab generation tests for verbal sentences",
    "test_cases": [
        {
            "id": "irab-verbal-001",
            "description": "Simple past tense verbal sentence",
            "input_text": "كتب محمد رسالة",
            "analysis": { ... },                          // Mock AnalysisResult
            "config": {
                "language": "en",
                "format": "json",
                "educational_level": "intermediate",
                "include_evidence": false
            },
            "expected": {
                "sentence_type": "Verbal Sentence (جملة فعلية)",
                "token_count": 3,
                "constructions_count": 1,
                "irab_entries": [
                    {
                        "token": "كَتَبَ",
                        "root": "كتب",
                        "pos": "Verb",
                        "features": [
                            { "name": "Gender", "value": "Masculine" },
                            { "name": "Number", "value": "Singular" },
                            { "name": "Person", "value": "Third Person" },
                            { "name": "Tense", "value": "Past (الماضي)" },
                            { "name": "Voice", "value": "Active (معلوم)" }
                        ],
                        "syntactic_role": "Verb (فعل)",
                        "explanation_must_contain": ["verb", "past", "masculine", "singular"]
                    },
                    {
                        "token": "مُحَمَّدٌ",
                        "root": "حمد",
                        "pos": "Proper Noun",
                        "features": [
                            { "name": "Gender", "value": "Masculine" },
                            { "name": "Number", "value": "Singular" },
                            { "name": "Case", "value": "Nominative (الرفع)" },
                            { "name": "State", "value": "Indefinite (نكرة)" }
                        ],
                        "syntactic_role": "Subject (فاعل)",
                        "explanation_must_contain": ["subject", "fa'il", "nominative"]
                    },
                    {
                        "token": "رِسَالَةً",
                        "root": "ر س ل",
                        "pos": "Noun",
                        "features": [
                            { "name": "Gender", "value": "Feminine" },
                            { "name": "Number", "value": "Singular" },
                            { "name": "Case", "value": "Accusative (النصب)" },
                            { "name": "State", "value": "Indefinite (نكرة)" }
                        ],
                        "syntactic_role": "Direct Object (مفعول به)",
                        "explanation_must_contain": ["object", "maf'ul", "accusative"]
                    }
                ],
                "overview_must_contain": ["verbal", "verb"],
                "must_not_contain": ["unknown", "error", "ambiguous"]
            }
        }
    ]
}
```

### 14.4 LLM Quality Evaluation

```rust
/// Framework for evaluating LLM explanation quality.
struct LLMQualityReport {
    model: String,
    prompt_style: PromptStyle,
    test_sentences: u32,

    // Accuracy metrics
    contradiction_rate: f64,            // % of responses with contradictions
    hallucination_rate: f64,            // % of responses with hallucinated terms
    token_accuracy: f64,                // % of tokens correctly described

    // Quality metrics
    avg_explanation_length: f64,        // Words per explanation
    avg_rating: f64,                    // Human rating (1-5)
    pedagogical_value: f64,             // Human rating for teaching quality

    // Performance metrics
    avg_latency_ms: f64,
    p99_latency_ms: f64,
    avg_cost_per_sentence: f64,

    // Comparison with template baseline
    improvement_over_template: f64,     // % improvement in user satisfaction
}

/// Quality targets for LLM enhancement:
/// - Contradiction rate:              < 1%
/// - Hallucination rate:              < 2%
/// - Token accuracy:                  > 95%
/// - Human rating:                    > 4.0 / 5.0
/// - Pedagogical value improvement:   > 20% over template baseline
/// - Latency p99 (LLM):              < 5,000 ms
```

### 14.5 Localization Completeness Check

```rust
/// Verify that a language pack is complete.
fn validate_language_pack(pack: &LanguagePack) -> Vec<LocalizationIssue> {
    let mut issues = Vec::new();

    // 1. Check all required labels exist
    let required_labels = [
        "section_grammatical_analysis", "section_irab_breakdown",
        "section_sentence_type", "section_overview",
        "section_constructions", "section_flags", "section_evidence",
        "column_token", "column_root", "column_pos",
        "column_features", "column_role", "column_explanation",
    ];
    for label in &required_labels {
        if !pack.labels.contains_key(*label) {
            issues.push(LocalizationIssue::MissingKey {
                key: format!("labels.{}", label),
            });
        }
    }

    // 2. Check all POS translations exist
    let required_pos = ["verb", "noun", "particle", "pronoun",
                        "adjective", "adverb", "preposition", "conjunction",
                        "proper_noun", "interrogative", "unknown"];
    for pos in &required_pos {
        if !pack.pos.contains_key(*pos) {
            issues.push(LocalizationIssue::MissingKey {
                key: format!("pos.{}", pos),
            });
        }
    }

    // 3. Check all feature labels and values exist
    let required_features = ["gender", "number", "person", "tense",
                             "mood", "voice", "case", "state"];
    for feat in &required_features {
        if let Some(feature) = pack.features.get(*feat) {
            if feature.values.is_empty() {
                issues.push(LocalizationIssue::EmptyFeatureValues {
                    feature: feat.to_string(),
                });
            }
        } else {
            issues.push(LocalizationIssue::MissingKey {
                key: format!("features.{}", feat),
            });
        }
    }

    // 4. Check all role translations exist
    let required_roles = ["fi'l", "fa'il", "mubtada", "khabar",
                          "maf'ul_bi-hi", "idafa", "mudaf", "mudaf_ilayh",
                          "harf_jarr", "majrur", "na'at"];

    // 5. Check plural form support
    let plural_forms = pack.meta.plural_forms.split('_').collect::<Vec<&str>>();
    if plural_forms.is_empty() {
        issues.push(LocalizationIssue::MissingPluralForms);
    }

    issues
}

enum LocalizationIssue {
    MissingKey { key: String },
    EmptyFeatureValues { feature: String },
    MissingPluralForms,
    InvalidPluralForms { expected: Vec<String>, actual: Vec<String> },
    UntranslatedString { key: String, value: String },
    EncodingIssue { details: String },
}
```

### 14.6 Benchmark Scenarios

```rust
/// Performance benchmark scenarios for MOD-11.
const EXPLANATION_BENCHMARKS: &[BenchmarkScenario] = &[
    // Short sentence benchmarks
    BenchmarkScenario {
        name: "short-verbal-3words",
        description: "3-word verbal sentence, JSON output",
        config: ExplainConfig {
            language: "en".into(),
            format: OutputFormat::Json,
            educational_level: EducationalLevel::Intermediate,
            include_evidence: false,
            ..Default::default()
        },
        target_p50_us: 50,    // < 50 μs
        target_p99_us: 200,   // < 200 μs
    },
    BenchmarkScenario {
        name: "short-verbal-3words-html",
        description: "3-word verbal sentence, HTML output",
        config: ExplainConfig {
            language: "en".into(),
            format: OutputFormat::Html,
            educational_level: EducationalLevel::Intermediate,
            include_evidence: false,
            ..Default::default()
        },
        target_p50_us: 100,
        target_p99_us: 400,
    },
    // Medium sentence benchmarks
    BenchmarkScenario {
        name: "medium-nominal-8words",
        description: "8-word nominal sentence with idafa",
        config: ExplainConfig {
            language: "en".into(),
            format: OutputFormat::Json,
            educational_level: EducationalLevel::Intermediate,
            include_evidence: false,
            ..Default::default()
        },
        target_p50_us: 150,
        target_p99_us: 500,
    },
    // Long sentence benchmarks
    BenchmarkScenario {
        name: "long-poetic-20words",
        description: "20-word poetic sentence with multiple constructions",
        config: ExplainConfig {
            language: "en".into(),
            format: OutputFormat::Html,
            educational_level: EducationalLevel::Advanced,
            include_evidence: true,
            ..Default::default()
        },
        target_p50_us: 500,
        target_p99_us: 2000,
    },
    // LLM benchmarks (measured separately)
    BenchmarkScenario {
        name: "llm-enhanced-10words",
        description: "10-word sentence with LLM enhancement",
        config: ExplainConfig {
            language: "en".into(),
            format: OutputFormat::Json,
            educational_level: EducationalLevel::Intermediate,
            enable_llm: true,
            llm: Some(LLMConfig {
                provider: "openai".into(),
                model: "gpt-4o-mini".into(),
                temperature: 0.3,
                max_tokens: 500,
                timeout_ms: 5000,
                prompt_style: PromptStyle::Standard,
                cache_ttl_seconds: 86400,
            }),
            ..Default::default()
        },
        target_p50_us: 1_000_000,     // < 1,000 ms
        target_p99_us: 5_000_000,     // < 5,000 ms
    },
];

struct BenchmarkScenario {
    name: &'static str,
    description: &'static str,
    config: ExplainConfig,
    target_p50_us: u64,
    target_p99_us: u64,
}
```

---

## 15. Cross-References

### 15.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 §4.11 | ExplanationEngine Module Description | High-level module responsibilities and pipeline position |
| SPEC-0001-C4 §13 | MOD-11 Interface | Formal interface definition (explain, formats, languages) |
| SPEC-0001-C5 §12 | IR-11 Schema | ExplanationOutput schema definition |
| SPEC-0001-C7 §3.2 | ExplanationPlugin | Plugin injection point for MOD-11 |
| SPEC-0001-C8 §4.3 | Explanation Error Catalog | Error codes: EXPLANATION_LANG_UNSUPPORTED, etc. |
| SPEC-0001-C9 §3.2 | MOD-11 Performance Targets | Latency, throughput, memory targets |
| SPEC-0301 §8–11 | Grammar Runtime MOD-11 | Core MOD-11 architecture and basic implementation |
| SPEC-0301 §9 | Template System & Localization | Template registry, language pack schema |
| SPEC-0301 §10 | LLM Integration | Basic LLM flow, provider interface, output validation |
| SPEC-0301 §11 | Output Formatting | JSON/text/HTML rendering |
| SPEC-0401 §3 | Knowledge Graph Data | Semantic data consumed by MOD-11 |
| RFC-0001 | Grammar DSL | Rule definitions that generate evidence for explanations |
| KB-0007 | Morphological Features | Feature taxonomy used in I'rab generation |
| ADR-0001 | Compiler Architecture Rationale | Architecture decisions affecting MOD-11 |

### 15.2 External References

| Reference | Relevance |
|-----------|-----------|
| CLDR (Unicode Common Locale Data Repository) | Plural rules for all supported languages |
| Unicode Bidirectional Algorithm (UAX #9) | RTL text handling in output |
| WCAG 2.1 | Web accessibility standards for HTML output |
| WAI-ARIA 1.2 | ARIA roles for accessible HTML output |
| ISO 32000 (PDF) | PDF generation standards |
| OpenAI API Reference | LLM provider integration |
| Anthropic API Reference | LLM provider integration |
| Sibawayh, Al-Kitab | Classical source for Arabic grammar explanations |
| Ibn Malik, Alfiyya | Classical reference for grammatical rules |

### 15.3 KB Dependencies

| KB | How MOD-11 Uses It |
|----|-------------------|
| KB-0001 (Roots) | Root meanings for semantic explanations |
| KB-0002 (Wazan) | Pattern descriptions for derivational explanations |
| KB-0003 (Verb Forms) | Conjugation paradigm generation |
| KB-0004 (Noun Patterns) | Noun type identification for explanations |
| KB-0005 (Particles) | Particle function descriptions |
| KB-0006 (Pronouns) | Pronoun reference explanations |
| KB-0007 (Features) | Feature taxonomy for localized display |

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2026-07-15 | AGOS Architecture Committee | Initial draft |

**Dependencies:** SPEC-0001-C2/C4/C5/C7/C8/C9, SPEC-0301, SPEC-0401, RFC-0001, KB-0001–0007, ADR-0001.

**Recommended next step:** SPEC-0601 (Plugin System) — the detailed specification for MOD-12 covering plugin lifecycle management, WASM-based sandboxing, distribution, and the plugin registry.
