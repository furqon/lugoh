---
rfc_id: RFC-0004
title: Arabic Grammar Rule DSL — School-Specific Rule Sets, Standard Library & Authoring Conventions
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-15
updated: 2026-07-15
references:
  - RFC-0001: Grammar DSL — Domain-Specific Language for Grammatical Rules
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage (MOD-07 RuleEngine)
  - SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-07 interface)
  - SPEC-0001-C5: Data Flow & Intermediate Representations (IR-7 AnnotatedGIR)
  - SPEC-0201: Rule Engine (MOD-07 detailed implementation)
  - SPEC-0101: Morphology Engine
  - KB-0005: Particles
  - KB-0006: Pronouns
  - KB-0007: Morphological Features Taxonomy
  - ADR-0001: Compiler Architecture Rationale
  - ADR-0003: Grammar Intermediate Representation Rationale
supersedes: None (complements RFC-0001)
---

# RFC-0004: Arabic Grammar Rule DSL — School-Specific Rule Sets, Standard Library & Authoring Conventions

## Table of Contents

1. [Introduction & Scope](#1-introduction--scope)
2. [Relationship to RFC-0001](#2-relationship-to-rfc-0001)
3. [Rule Authoring Conventions](#3-rule-authoring-conventions)
4. [Rule Categories & Priority Allocation](#4-rule-categories--priority-allocation)
5. [Arabic Grammatical Terminology Mapping](#5-arabic-grammatical-terminology-mapping)
6. [Complete School Catalogs](#6-complete-school-catalogs)
7. [Arabic Standard Library Specification](#7-arabic-standard-library-specification)
8. [School-Specific Rule Pattern Catalog](#8-school-specific-rule-pattern-catalog)
9. [Rule Testing Framework](#9-rule-testing-framework)
10. [DSL Compilation Pipeline](#10-dsl-compilation-pipeline)
11. [Performance Benchmarks for Arabic Rule Sets](#11-performance-benchmarks-for-arabic-rule-sets)
12. [Migration from Traditional Grammar Sources](#12-migration-from-traditional-grammar-sources)
13. [Versioning & Rule Set Distribution](#13-versioning--rule-set-distribution)
14. [Cross-References](#14-cross-references)

---

## 1. Introduction & Scope

### 1.1 Purpose

This RFC defines the **Arabic Grammar Rule DSL** — the complete set of conventions, rule catalogs, standard library modules, and authoring best practices for expressing Arabic grammatical rules in the AGOS Grammar DSL (defined in RFC-0001). Where RFC-0001 defines *how* to write DSL rules (the syntax), this RFC defines *what* rules to write (the content) and *how to organize them* for the five supported grammar schools.

This specification serves as the definitive reference for:

- **Arabic linguists and grammar experts** who author and maintain school-specific rule sets.
- **Rule engine implementers** (MOD-07 / SPEC-0201) who consume compiled DSL rules.
- **Test authors** who validate rule correctness against known Arabic constructions.
- **Educators** who want to understand how traditional Arabic grammar (النحو والصرف) maps to formal rule specifications.

### 1.2 Scope

**In scope:**

| Area | Coverage |
|------|----------|
| **Rule categories** | Complete taxonomy of Arabic grammatical rule categories with priority allocation |
| **School catalogs** | Rule counts and coverage by school (Basra, Kufa, Baghdad, Andalus, Modern) |
| **Standard library** | Complete Arabic-specific predicate library (agreement, case, mood, constructions) |
| **Rule patterns** | Reusable rule templates for common Arabic grammatical constructions |
| **Test framework** | Test fixture format, example test cases for each school and category |
| **Terminology mapping** | Arabic grammatical terms (المصطلحات النحوية) ↔ DSL constructs |
| **Compilation pipeline** | DSL source → Compiled RuleSet format consumed by MOD-07 |
| **Migration guidance** | How to translate traditional grammar sources (Sibawayh, Ibn Malik, etc.) into DSL rules |

**Out of scope:**

| Out of Scope | Covered By |
|-------------|------------|
| DSL syntax and semantics | RFC-0001 (Grammar DSL) |
| Rule execution engine | SPEC-0201 (Rule Engine) |
| Morphological features taxonomy | KB-0007 |
| Particle/pro noun knowledge bases | KB-0005, KB-0006 |
| Bytecode generation from rules | SPEC-0001-C3 (MOD-09) |

### 1.3 Design Principles

1. **School fidelity.** Each rule set MUST faithfully represent its school's grammatical doctrine. A Basra rule that contradicts Basra's known positions is worse than no rule at all.

2. **Rule determinism.** Every rule MUST be deterministic — given the same GIR input, the same rule must produce the same effect. Rules MUST NOT depend on execution order, ambient state, or random factors.

3. **One rule, one effect.** Each rule SHOULD express exactly one grammatical principle. Complex multi-effect rules are split into simpler rules that compose through priority ordering.

4. **Tested with Quranic and classical evidence.** Every rule SHOULD have at least one test case drawn from the Quran, classical poetry, or canonical grammatical examples (الشواهد).

5. **Composable across schools.** Rules from different schools MUST be usable in the same analysis session. School-specific behavior emerges from which rules are active, not from rule implementation differences.

6. **Evidence trail completeness.** Every rule application MUST produce sufficient evidence for the Explanation Engine (MOD-11) to generate meaningful pedagogical explanations.

---

## 2. Relationship to RFC-0001

### 2.1 Layer Separation

```
RFC-0001 (Grammar DSL)                  RFC-0004 (Arabic Grammar Rule DSL)
│                                       │
├── Syntax & semantics                  ├── Arabic-specific rule content
├── Lexical structure                   ├── School catalogs (Basra, Kufa...)
├── Data types                          ├── Arabic standard library
├── Expression system                   ├── Rule patterns & conventions
├── Built-in functions                  ├── Test fixtures
├── Formal EBNF grammar                 ├── Arabic terminology mappings
│                                       ├── Compilation pipeline
└── (language definition)               └── (rule content & methodology)
```

### 2.2 How Both Are Used Together

```
1. Linguist authors rules using RFC-0001 syntax
   ↓
2. Rules follow RFC-0004 conventions (priority, categories, terminology)
   ↓
3. Rules are compiled by the DSL compiler (RFC-0001 § EBNF)
   ↓
4. Compiled RuleSet plugins are loaded by MOD-07 (SPEC-0201)
   ↓
5. Rule Engine applies them to the GIR (SPEC-0001-C3 §8)
```

### 2.3 When to Reference Each Document

| Use Case | Reference |
|----------|-----------|
| "How do I write a condition expression?" | RFC-0001 §6 (Expressions) |
| "What built-in functions are available?" | RFC-0001 §11 (Built-in Functions) |
| "What priority should I assign to agreement rules?" | RFC-0004 §4 (Priority Allocation) |
| "How do I write a complete Basra rule set?" | RFC-0004 §6 (School Catalogs) |
| "What predicates does the standard library provide?" | RFC-0004 §7 (Standard Library) |
| "How do I test my rule against Quranic evidence?" | RFC-0004 §9 (Testing Framework) |

---

## 3. Rule Authoring Conventions

### 3.1 File Organization

```
rules/
├── basra/                          # Basra school rule set
│   ├── metadata.agosrule           # School metadata, imports
│   ├── agreement.agosrule          # Subject-verb agreement rules
│   ├── case-assignment.agosrule    # Case assignment rules
│   ├── mood-government.agosrule    # Mood government rules
│   ├── constructions.agosrule      # Idafa, wasf, tawkid, badal
│   ├── conditional.agosrule        # Conditional sentences
│   ├── special-verbs.agosrule      # Kana, inna, and sisters
│   ├── exceptions.agosrule         # Exceptive constructions
│   └── anaphora.agosrule           # Pronoun resolution rules
│
├── kufa/                           # Kufa school rule set
│   ├── metadata.agosrule
│   ├── agreement.agosrule
│   ├── case-assignment.agosrule
│   └── ... (same categories, different rules)
│
├── baghdad/                        # Baghdad school (eclectic)
│   ├── metadata.agosrule
│   └── ...
│
├── andalus/                        # Andalus school (Western)
│   ├── metadata.agosrule
│   └── ...
│
├── modern/                         # Modern Standard Arabic
│   ├── metadata.agosrule
│   └── ...
│
├── common/                         # Shared rules across schools
│   ├── definitions.agosrule        # Common predicates
│   ├── particles.agosrule          # Particle behavior
│   └── pronouns.agosrule           # Pronoun resolution base
│
├── lib/                            # Standard library
│   ├── agos-core.agosrule          # Core predicates
│   ├── agreement.agosrule          # Agreement predicates
│   ├── case.agosrule               # Case predicates
│   ├── mood.agosrule               # Mood predicates
│   ├── constructions.agosrule      # Construction predicates
│   └── validation.agosrule         # Validation predicates
│
└── tests/                          # Test fixtures
    ├── basra/
    ├── kufa/
    ├── baghdad/
    ├── andalus/
    ├── modern/
    └── fixtures/                   # Shared test sentences
```

### 3.2 Naming Conventions

#### Rule IDs

```
{school}-{category-code}-{sequential-number}

Examples:
basra-0103     # Basra, agreement (01), rule 03
kufa-0201      # Kufa, case-assignment (02), rule 01
baghdad-0305   # Baghdad, mood-government (03), rule 05
andalus-0402   # Andalus, constructions (04), rule 02
modern-0101    # Modern, agreement (01), rule 01
```

#### Category Codes

| Code | Category | Description |
|------|----------|-------------|
| `01` | agreement | Subject-verb agreement |
| `02` | case-assignment | Case (i'rab) assignment |
| `03` | mood-government | Mood (jussive/subjunctive) government |
| `04` | constructions | Idafa, wasf, tawkid, badal |
| `05` | conditional | Conditional sentences (shart/jaza) |
| `06` | special-verbs | Kana, inna, and sisters |
| `07` | exceptions | Istithna and exception constructions |
| `08` | anaphora | Pronoun resolution |
| `09` | particles | Particle governance |
| `10` | word-order | Sentence constituent ordering |
| `11` | ambiguity-resolution | Cross-category ambiguity resolution |
| `12` | validation | Input validation guards |

#### File Names

- Lowercase with hyphens: `agreement.agosrule`, `case-assignment.agosrule`
- Test files: `{rule-id}.test.agosrule`
- Fixture files: `{description}.fixture.json`

#### Rule Titles

Titles follow the pattern: `{school}-{category-code}: {Arabic term} — {English description}`

Examples:
```
rule "basra-0103: تطابق الفعل والفاعل — Subject-Verb Person Agreement"
rule "kufa-0201: جر المضاف إليه — Preposition Governs Genitive Case"
rule "basra-0305: كان وأخواتها — Kana and Her Sisters"
```

### 3.3 Metadata Conventions

```ebnf
metadata {
    id: "{school}-{code}-{num}"
    school: "{school-name}"          // basra | kufa | baghdad | andalus | modern
    version: "{semver}"
    priority: {0-100}

    // Required fields
    description: "{English description}"

    // Strongly recommended
    source: "{Primary reference, e.g., 'Sibawayh, Al-Kitab, Vol. 1, p. 234'}"
    tags: ["{category}", "{sub-category}", "{grammar-term}"]

    // Optional
    category: "{category-name}"
    evidence: "{Quranic or poetic evidence, e.g., 'Quran 2:255'}"
    enabled: true
}
```

#### Priority Conventions

| Priority Range | Category | Rationale |
|----------------|----------|-----------|
| 91–100 | Validation guards | Run first to verify input integrity |
| 71–90 | Agreement rules | Run early to establish basic relations |
| 51–70 | Case assignment | Run after agreement is established |
| 31–50 | Mood government | Run after case assignment |
| 21–30 | Construction identification | Run after case/mood are resolved |
| 11–20 | Ambiguity resolution | Run last, when most context is available |
| 1–10 | Anaphora resolution | Run last, as fallback |

### 3.4 Comment Conventions

```ebnf
// File header: license, school, and purpose
// ================================================
// basra/agreement.agosrule
// Basra School: Subject-Verb Agreement Rules
// Version: 1.2.0
// License: AGOS Open Rule License 1.0
// ================================================

// Rule header: reference source and grammatical principle
// -------------------------------------------------------
// Sibawayh, Al-Kitab, Vol. 1, p. 234:
// "الفعل يطابق الفاعل في الشخص"
// The verb agrees with the subject in person.
// Quranic evidence: "وَقَالَ ٱلَّذِينَ" (Quran 2:246)
// -------------------------------------------------------

// Inline annotation: explain non-obvious conditions or effects
condition {
    sentence.type == "jumlah_fi'liyyah"
    and exists(fa'il)
    // The verb must agree with the subject in person.
    // If the verb precedes the subject (which is common in verbal
    // sentences), person agreement still holds.
    and fi'l.person != fa'il.person
}
```

### 3.5 Rule Writing Guidelines

1. **Prefer positive conditions.** Write `fi'l.person == fa'il.person` (confirm agreement) rather than rejecting disagreement. Rules that confirm should run before rules that reject.

2. **One grammatical principle per rule.** If a construction has both case and mood implications, split into two rules (one for case, one for mood) with appropriate priorities.

3. **Use standard library predicates.** Before writing a complex condition, check if the standard library (§7) already provides a predicate for it.

4. **Document Quranic/poetic evidence.** Every rule SHOULD cite at least one shâhid (شاهد) — a Quranic verse, hadith, or line of classical poetry — that demonstrates the rule.

5. **Mark school-specific divergence.** When a rule differs from the majority school position, add a comment explaining the divergence and its source.

6. **Prefer `confirm()` over `reject()`.** Confirming an analysis preserves information for the Explanation Engine. Rejecting destroys it. Use `reject()` only when the analysis is definitively invalid.

---

## 4. Rule Categories & Priority Allocation

### 4.1 Category Definitions

Each category represents a distinct domain of Arabic grammatical rules. Rules within a category are related in purpose and priority.

| # | Category | Code | Scope | Rule Count (per School) | Priority Range |
|---|----------|------|-------|------------------------|----------------|
| 1 | Subject-Verb Agreement | `agreement` | Person, number, gender concord between verb and subject | 15–25 | 71–90 |
| 2 | Case Assignment (I'rab) | `case-assignment` | Nominative, accusative, genitive assignment by governors | 30–50 | 51–70 |
| 3 | Mood Government | `mood-government` | Indicative, subjunctive, jussive assignment by particles | 20–35 | 31–50 |
| 4 | Constructions | `constructions` | Idafa, wasf, tawkid, badal, istithna, nida | 25–40 | 21–30 |
| 5 | Conditional Sentences | `conditional` | Shart/jaza structure, mood interaction | 10–15 | 31–50 |
| 6 | Special Verbs | `special-verbs` | Kana & sisters, inna & sisters, zanna & sisters | 15–25 | 51–70 |
| 7 | Exceptions | `exceptions` | Istithna with illa, ghayr, siwa | 8–12 | 21–30 |
| 8 | Anaphora Resolution | `anaphora` | Pronoun-antecedent linking, demonstrative reference | 10–15 | 1–10 |
| 9 | Particle Governance | `particles` | Preposition, conjunction, interroga-tive particle effects | 20–30 | 51–70 |
| 10 | Word Order | `word-order` | Subject-verb order, adjective placement | 8–12 | 71–90 |
| 11 | Ambiguity Resolution | `ambiguity` | Cross-category tie-breaking, confidence weighting | 10–20 | 11–20 |
| 12 | Input Validation | `validation` | Pre-condition checks, input integrity guards | 5–10 | 91–100 |

### 4.2 Priority Allocation Algorithm

```ebnf
// Priority is assigned based on three dimensions:
// 1. Category base priority (from the table above)
// 2. Rule specificity (more specific = higher priority within category)
// 3. School-specific adjustments

priority = category_base + specificity_offset + school_adjustment

// specificity_offset:
//   +10  The rule targets a specific token or constituent
//   +5   The rule targets a specific feature value
//   0    The rule targets a general pattern
//   -5   The rule is a catch-all or fallback

// school_adjustment:
//   +5   The rule represents a minority school position
//         (needs to run before the majority position to override it)
//   -5   The rule is widely accepted and non-controversial
//   0    Default

// Example:
// Rule: "Verb before plural subject takes singular verb" (Basra position)
//   category_base = 75 (agreement)
//   specificity_offset = +5 (targets specific number value)
//   school_adjustment = 0 (standard position)
//   priority = 80
//
// Rule: "Verb before plural subject takes plural verb" (Kufa position)
//   category_base = 75 (agreement)
//   specificity_offset = +5 (targets specific number value)
//   school_adjustment = +5 (minority Kufa position needs to override Basra)
//   priority = 85
```

### 4.3 Rule Priority Conflict Resolution

When two rules apply to the same token/constituent with contradictory effects:

```ebnf
// Resolution order:
// 1. Higher priority wins
// 2. Same priority: school-specific position wins over general
// 3. Same school, same priority: REJECT overrides CONFIRM
//    (safety principle: when in doubt, preserve ambiguity)
// 4. Same school, same priority, same action type: last rule wins
//    (deterministic: rules are ordered alphabetically by ID within
//     the same priority)

function resolve_conflict(rules: RuleApplication[]):
    sort(rules, by: priority DESC, then action_type, then rule_id)
    return rules[0]  // Winner
```

---

## 5. Arabic Grammatical Terminology Mapping

### 5.1 Part of Speech (أقسام الكلام)

| Arabic Term | Transliteration | DSL `pos` Value | Definition |
|-------------|-----------------|-----------------|------------|
| فعل | fi'l | `"verb"` | An action or event (past, present, imperative) |
| اسم | ism | `"noun"` | A person, place, thing, or quality |
| حرف | harf | `"particle"` | A grammatical particle (preposition, conjunction, etc.) |
| ضمير | damir | `"pronoun"` | A pronoun (attached or detached) |
| صفة | sifah | `"adjective"` | A qualifier (follows noun in agreement) |
| ظرف | zarf | `"adverb"` | Adverb of time or place (in accusative) |
| استفهام | istifham | `"interrogative"` | Interrogative particle or pronoun |

### 5.2 Syntactic Roles (الوظائف النحوية)

| Arabic Term | Transliteration | DSL `role` Value | Definition |
|-------------|-----------------|-----------------|------------|
| فعل | fi'l | `"fi'l"` | Verb (predicate of verbal sentence) |
| فاعل | fa'il | `"fa'il"` | Subject of a verb |
| نائب فاعل | na'ib al-fa'il | `"na'ib_al-fa'il"` | Deputy subject (passive) |
| مبتدأ | mubtada' | `"mubtada"` | Topic of nominal sentence |
| خبر | khabar | `"khabar"` | Comment/predicate of nominal sentence |
| مفعول به | maf'ul bihi | `"maf'ul_bi-hi"` | Direct object |
| مفعول مطلق | maf'ul mutlaq | `"maf'ul_mutlaq"` | Absolute object/cognate accusative |
| مفعول فيه | maf'ul fih | `"maf'ul_fih"` | Adverb of time/place |
| مفعول لأجله | maf'ul lahu | `"maf'ul_lahu"` | Object of purpose |
| مفعول معه | maf'ul ma'ahu | `"maf'ul_ma'ahu"` | Object of accompaniment |
| حال | hal | `"hal"` | Circumstantial accusative |
| تمييز | tamyiz | `"tamyiz"` | Specification/distinction |
| نعت | na'at | `"na'at"` | Adjective/qualifier |
| مضاف | mudaf | `"mudaf"` | First term of construct state |
| مضاف إليه | mudaf ilayh | `"mudaf_ilayh"` | Second term of construct state |
| حرف جر | harf jarr | `"harf_jarr"` | Preposition |
| مجرور | majrur | `"majrur"` | Noun in genitive (governed by preposition) |
| حرف نصب | harf nasb | `"harf_nasb"` | Accusative/subjunctive particle |
| حرف جزم | harf jazm | `"harf_jazm"` | Jussive particle |
| بدل | badal | `"badal"` | Apposition |
| توكيد | tawkid | `"ta'kid"` | Emphasis |
| شرط | shart | `"shart"` | Condition (protasis) |
| جواب | jawab | `"jaza"` | Result (apodosis) |

### 5.3 Grammatical Features (الصيغ النحوية)

| Feature | Arabic Term | DSL Feature Path | Valid Values |
|---------|-------------|------------------|--------------|
| Gender | الجنس | `features.gender` | `masculine`, `feminine` |
| Number | العدد | `features.number` | `singular`, `dual`, `plural` |
| Person | الشخص | `features.person` | `first`, `second`, `third` |
| Tense | الزمن | `features.tense` | `past`, `present`, `imperative` |
| Mood | الحالة الإعرابية | `features.mood` | `indicative`, `subjunctive`, `jussive`, `energetic` |
| Voice | المبنى للمعلوم/المجهول | `features.voice` | `active`, `passive` |
| Case | الإعراب | `features.case` | `nominative`, `accusative`, `genitive` |
| State | التعريف والتنكير | `features.state` | `definite`, `indefinite` |
| Verb Form | وزن الفعل | `features.form` | `1`–`15` (I–XV) |

### 5.4 Sentence Types (أنواع الجمل)

| Arabic Term | Transliteration | DSL `sentence.type` | Description |
|-------------|-----------------|---------------------|-------------|
| جملة فعلية | jumlah fi'liyyah | `"jumlah_fi'liyyah"` | Verbal sentence (begins with verb) |
| جملة اسمية | jumlah ismiyyah | `"jumlah_ismiyyah"` | Nominal sentence (begins with noun) |
| جملة شرطية | jumlah shartiyyah | `"jumlah_shartiyyah"` | Conditional sentence |
| جملة ظرفية | jumlah zarfiyyah | `"jumlah_zarfiyyah"` | Adverbial clause |
| جملة كان | jumlah kāna | `"jumlah_kāna"` | Kana and her sisters |
| جملة إن | jumlah inna | `"jumlah_inna"` | Inna and her sisters |
| جملة قسم | jumlah qasam | `"jumlah_qasam"` | Oath construction |
| جملة استفهام | jumlah istifham | `"jumlah_istifham"` | Interrogative sentence |

### 5.5 Key Grammatical Terms for Rule Authors

| Arabic | Transliteration | English | Usage in DSL |
|--------|-----------------|---------|--------------|
| عامل | 'amil | Governor | The element that assigns case/mood to another element |
| معمول | ma'mul | Governed | The element whose case/mood is assigned |
| إعراب | i'rab | Desinential inflection | The system of case/mood endings |
| بناء | bina' | Invariable form | A word that does not show case inflection |
| رفع | raf' | Nominative/indicative | Default case/mood |
| نصب | nasb | Accusative/subjunctive | Governed case/mood |
| جر | jarr | Genitive | Case governed by preposition |
| جزم | jazm | Jussive | Mood governed by jussive particle |
| تنوين | tanwin | Nunation | Indefinite marker (-un, -an, -in) |
| حذف | hadhf | Ellipsis | Omission of a constituent |
| تقديم وتأخير | taqdim wa ta'khir | Fronting and postponing | Word order variation |
| ضرورة | darura | Poetic necessity | Grammatical license in poetry |

---

## 6. Complete School Catalogs

### 6.1 Comparative Overview

| Aspect | Basra | Kufa | Baghdad | Andalus | Modern |
|--------|-------|------|---------|---------|--------|
| **Total rules** | ~850 | ~720 | ~650 | ~580 | ~400 |
| **Categories** | 12 | 12 | 12 | 11 | 10 |
| **Priority span** | 1–100 | 1–100 | 1–95 | 1–90 | 1–85 |
| **Distinctive doctrines** | Qiyas (analogy), strict 'amil theory | Samā' (transmitted usage), broader acceptance | Eclectic, reconciles Basra & Kufa | Andalusian pragmatism | Simplified for MSA |
| **Key divergence from Basra** | — | Verb before subject allows plural; إِنَّ has different case | TBD per issue | Less strict on 'amil sequence | Drops some classical distinctions |

### 6.2 Basra School Rule Catalog (~850 rules)

The Basra school (مدرسة البصرة) is the default and most comprehensive rule set. It is the reference against which other schools are measured.

#### Category: Agreement (01) — 22 rules

| ID | Title | Priority | Description |
|----|-------|----------|-------------|
| basra-0101 | تطابق الفعل والفاعل في الشخص — Person Agreement | 85 | Verb must agree with subject in person |
| basra-0102 | تطابق الفعل والفاعل في العدد — Number Agreement | 80 | Verb agrees with subject in number (verb-before-subject exception) |
| basra-0103 | تطابق الفعل والفاعل في الجنس — Gender Agreement | 75 | Verb agrees with subject in gender |
| basra-0104 | فعل الجمع قبل الفاعل المفرد — Verb Before Plural Subject | 80 | Verb before plural subject takes singular (Basra position) |
| basra-0105 | فاعل جمع غير الآدمي — Non-Human Plural Subject | 75 | Non-human plural subject takes feminine singular verb |
| basra-0106 | تطابق المبتدأ والخبر — Mubtada'-Khabar Agreement | 70 | Topic and comment agree in gender and number |
| ... | (remaining 16 rules cover dual agreement, detached pronoun agreement, etc.) | | |

#### Category: Case Assignment (02) — 45 rules

| ID | Title | Priority | Description |
|----|-------|----------|-------------|
| basra-0201 | جر المضاف إليه — Genitive in Idafa | 65 | Second term of idafa takes genitive |
| basra-0202 | رفع الفاعل — Subject Takes Nominative | 60 | Subject (fa'il) takes nominative case |
| basra-0203 | نصب المفعول به — Direct Object Takes Accusative | 60 | Direct object takes accusative |
| basra-0204 | جر بحرف الجر — Preposition Governs Genitive | 65 | Noun after preposition takes genitive |
| basra-0205 | تقديم الخبر وجوبًا — Verb Obligatorily Before Subject | 55 | Verb must precede subject in verbal sentence |
| ... | (remaining 40 rules cover muqaddam, ma'akhkhar, zarf, hal, tamyiz, etc.) | | |

#### Category: Mood Government (03) — 30 rules

| ID | Title | Priority | Description |
|----|-------|----------|-------------|
| basra-0301 | جزم الفعل المضارع بأن — Jussive by Lam | 45 | لَمْ governs jussive on present verb |
| basra-0302 | نصب الفعل المضارع بأن — Subjunctive by An | 45 | أَنْ governs subjunctive on present verb |
| basra-0303 | جزم الفعل في جواب الطلب — Jussive in Command Response | 40 | Jussive in fa-'l-amr answer |
| basra-0304 | نصب بأن المضمرة — Subjunctive by Implied An | 40 | Subjunctive after لِـ (implied أَنْ) |
| basra-0305 | كان وأخواتها — Kana and Her Sisters | 50 | Kana raises subject to accusative |
| ... | (remaining 25 rules cover لَمَّا, لَنْ, كَيْ, even more) | | |

#### Category: Constructions (04) — 35 rules

| ID | Title | Priority | Description |
|----|-------|----------|-------------|
| basra-0401 | تعريف المضاف — Definiteness in Idafa | 25 | First term of idafa is definite by position |
| basra-0402 | حذف التنوين في الإضافة — Tanwin Dropped in Idafa | 25 | Nunation is dropped from first term of idafa |
| basra-0403 | مطابقة النعت والمنعوت — Adjective-Noun Agreement | 25 | Na'at follows man'ut in gender, number, case, state |
| basra-0404 | جمع غير الآدمي في النعت — Non-Human Plural Adjective | 25 | Adjective of non-human plural takes feminine singular |
| basra-0405 | توكيد بالضمائر — Emphasis by Pronouns | 20 | Tawkid by nafs, 'ayn, kull |
| ... | (remaining 30 rules cover badal, istithna, nida, etc.) | | |

#### Category: Special Verbs (06) — 22 rules

| ID | Title | Priority | Description |
|----|-------|----------|-------------|
| basra-0601 | كان وأخواتها: رفع الاسم ونصب الخبر — Kana: Subject Nominative, Predicate Accusative | 55 | Ism kana in nominative, khabar kana in accusative |
| basra-0602 | إن وأخواتها: نصب الاسم ورفع الخبر — Inna: Subject Accusative, Predicate Nominative | 55 | Ism inna in accusative, khabar inna in nominative |
| basra-0603 | ظن وأخواتها: نصب المفعولين — Zanna: Two Accusatives | 50 | Verbs of cognition take two objects in accusative |
| basra-0604 | أعلم وأخواتها: نصب المفعول الثلاثة — A'lama: Three Accusatives | 45 | Verbs of informing take three objects |

#### (Total: ~22 + 45 + 30 + 35 + 22 + remaining categories ≈ 850 rules)

### 6.3 Kufa School Key Differences (~720 rules)

The Kufa school (مدرسة الكوفة) differs from Basra in several key doctrines. The following rules represent distinctive Kufa positions:

| ID | Title | Basra Position | Kufa Position | Priority |
|----|-------|---------------|---------------|----------|
| kufa-0101 | فعل الجمع قبل الفاعل الجمع — Verb Before Plural Subject | Verb takes singular | Verb may take plural | 85 |
| kufa-0102 | جواز تقديم الخبر — Permissibility of Fronting Khabar | Restricted | More permissive | 60 |
| kufa-0301 | جزم الفعل بإِنْ الشرطية — Conditional In | In governs jussive | Same as Basra | 45 |
| kufa-0601 | إِنَّ والأخوات: نصب الاسم — Inna Case Assignment | Ism inna in accusative | Cases vary by context | 55 |
| kufa-0701 | جواز الاستثناء — Exception Permissibility | More restricted | Broader acceptance | 25 |

### 6.4 Baghdad School Key Characteristics (~650 rules)

The Baghdad school (مدرسة بغداد) is eclectic, reconciling Basra and Kufa positions:

- Selects the position with stronger evidence (usually Basra's qiyas or Kufa's sama')
- ~60% alignment with Basra, ~30% with Kufa, ~10% independent positions
- Pragmatic: favors simpler rules where schools agree

### 6.5 Andalus School Key Characteristics (~580 rules)

The Andalus school (المدرسة الأندلسية) developed independently in Islamic Spain:

- Strong influence from Basra (Ibn Malik's Alfiyya is Basran)
- Pragmatic treatment of 'amil theory
- Less emphasis on hypothetical constructions
- Unique positions on some case/mood interactions

### 6.6 Modern Standard Arabic (~400 rules)

Modern Standard Arabic (MSA) simplifies several classical distinctions:

- Dropped energetic mood (most MSA does not use it)
- Simplified inna/kana distinctions in some contexts
- More permissive word order in nominal sentences
- Some classical agreement rules relaxed in journalistic Arabic

---

## 7. Arabic Standard Library Specification

### 7.1 Module Overview

```
lib/
├── agos-core.agosrule           # Universal predicates (sentence types, feature checks)
├── agreement.agosrule           # Subject-verb and noun-adjective agreement
├── case.agosrule                # Case assignment predicates
├── mood.agosrule                # Mood government predicates
├── constructions.agosrule       # Construction identification
├── particles.agosrule           # Particle-specific predicates
├── pronouns.agosrule            # Pronoun and anaphora predicates
├── validation.agosrule          # Input validation and integrity checks
└── quranic.agosrule             # Quranic-specific rules (optional module)
```

### 7.2 Core Library (`agos-core.agosrule`)

```ebnf
// ================================================
// lib/agos-core.agosrule
// AGOS Grammar Core: Universal Predicates
// Version: 1.0.0
// ================================================

// --- Sentence Type Predicates ---

predicate "is_verbal" = sentence.type == "jumlah_fi'liyyah"

predicate "is_nominal" = sentence.type == "jumlah_ismiyyah"

predicate "is_conditional" = sentence.type == "jumlah_shartiyyah"

predicate "is_kana_sentence" = sentence.type == "jumlah_kāna"

predicate "is_inna_sentence" = sentence.type == "jumlah_inna"

// --- Existence Predicates ---

predicate "has_verb" = exists(fi'l)

predicate "has_subject" = exists(fa'il)

predicate "has_direct_object" = exists(maf'ul_bi-hi)

predicate "has_prepositional_phrase" =
    exists(constituent in sentence.constituents {
        constituent.role == "harf_jarr"
    })

predicate "has_idafa" =
    exists(constituent in sentence.constituents {
        constituent.role == "idafa"
    })

// --- Feature Check Predicates ---

predicate "is_transitive_verb" =
    token.features.pos == "verb"
    and token.features.transitivity in ["transitive_1", "transitive_2", "ditransitive"]

predicate "is_intransitive_verb" =
    token.features.pos == "verb"
    and token.features.transitivity == "intransitive"

predicate "is_passive" =
    token.features.voice == "passive"

predicate "is_active" =
    token.features.voice == "active"

predicate "is_definite" =
    token.features.state == "definite"

predicate "is_indefinite" =
    token.features.state == "indefinite"

// --- Comparative Predicates ---

predicate "person_agrees" = fi'l.features.person == fa'il.features.person

predicate "number_agrees" = fi'l.features.number == fa'il.features.number

predicate "gender_agrees" = fi'l.features.gender == fa'il.features.gender

predicate "verb_before_subject" = fi'l.position < fa'il.position

predicate "subject_before_verb" = fa'il.position < fi'l.position

// --- Token Role Predicates ---

predicate "has_role" = token.role != null

predicate "is_subject" = token.role == "fa'il"

predicate "is_direct_object" = token.role == "maf'ul_bi-hi"

predicate "is_genitive" = token.features.case == "genitive"

predicate "is_accusative" = token.features.case == "accusative"

predicate "is_nominative" = token.features.case == "nominative"

// --- Collection Predicates ---

predicate "all_tokens_have_case" =
    forall (t in sentence.tokens) {
        t.features.case != null
    }

predicate "no_unknown_tokens" =
    forall (t in sentence.tokens) {
        t.features.pos != "unknown"
    }
```

### 7.3 Agreement Library (`agreement.agosrule`)

```ebnf
// ================================================
// lib/agreement.agosrule
// Agreement Predicates
// Version: 1.0.0
// ================================================

// --- Subject-Verb Agreement ---

predicate "verbal_agreement_complete" =
    person_agrees() and number_agrees() and gender_agrees()

// In verbal sentences where verb precedes subject (Basra position):
// Verb takes singular regardless of subject number
predicate "verb_before_subject_basra" =
    is_verbal()
    and verb_before_subject()
    and fi'l.features.number == "singular"

// In verbal sentences where verb precedes subject (Kufa position):
// Verb may take plural if subject is plural
predicate "verb_before_subject_kufa" =
    is_verbal()
    and verb_before_subject()
    and fa'il.features.number == "plural"
    and fi'l.features.number == "plural"

// Non-human plural subject takes feminine singular verb
predicate "non_human_plural_agreement" =
    fa'il.features.number == "plural"
    and not (fa'il.features.gender == "masculine" and "human" in fa'il.semantic_tags)
    and fi'l.features.gender == "feminine"
    and fi'l.features.number == "singular"

// --- Mubtada'-Khabar Agreement ---

predicate "mubtada_khabar_agreement" =
    exists(mubtada')
    and exists(khabar)
    and khabar.features.gender == mubtada'.features.gender
    and (khabar.features.number == mubtada'.features.number
         or mubtada'.features.number == "plural")  // Exception: plural topic can have singular comment

// --- Adjective Agreement ---

predicate "adjective_agreement_complete" =
    token.role == "na'at"
    and preceding_noun.features.gender == token.features.gender
    and preceding_noun.features.number == token.features.number
    and preceding_noun.features.case == token.features.case
    and preceding_noun.features.state == token.features.state
```

### 7.4 Case Library (`case.agosrule`)

```ebnf
// ================================================
// lib/case.agosrule
// Case Assignment Predicates
// Version: 1.0.0
// ================================================

// --- Basic Case Assignments ---

predicate "subject_case_is_nominative" =
    token.role == "fa'il"
    and token.features.case == "nominative"

predicate "object_case_is_accusative" =
    token.role == "maf'ul_bi-hi"
    and token.features.case == "accusative"

predicate "preposition_governs_genitive" =
    token.role == "majrur"
    and governing_particle.role == "harf_jarr"
    and token.features.case == "genitive"

predicate "idafa_governs_genitive" =
    token.role == "mudaf_ilayh"
    and token.features.case == "genitive"

// --- Special Case Assignments ---

// Hal (circumstantial accusative)
predicate "hal_is_accusative" =
    token.role == "hal"
    and token.features.case == "accusative"

// Tamyiz (specification)
predicate "tamyiz_is_accusative" =
    token.role == "tamyiz"
    and token.features.case == "accusative"

// Maf'ul mutlaq (cognate accusative)
predicate "maf_ul_mutlaq_is_accusative" =
    token.role == "maf'ul_mutlaq"
    and token.features.case == "accusative"

// --- Mubtada' and Khabar ---

predicate "mubtada_is_nominative" =
    exists(mubtada')
    and mubtada'.features.case == "nominative"

predicate "khabar_is_nominative" =
    exists(khabar)
    and khabar.features.case == "nominative"
```

### 7.5 Mood Library (`mood.agosrule`)

```ebnf
// ================================================
// lib/mood.agosrule
// Mood Government Predicates
// Version: 1.0.0
// ================================================

// --- Basic Mood Assignments ---

predicate "indicative_is_default" =
    token.features.pos == "verb"
    and token.features.tense == "present"
    and not (exists(governing_particle))
    and token.features.mood == "indicative"

predicate "lam_governs_jussive" =
    governing_particle.features.root == "لم"
    and token.features.mood == "jussive"

predicate "lan_governs_subjunctive" =
    governing_particle.features.root == "لن"
    and token.features.mood == "subjunctive"

predicate "an_governs_subjunctive" =
    governing_particle.features.root in ["أن", "كي", "لن"]
    and token.features.mood == "subjunctive"

// --- Conditional Mood ---

predicate "conditional_apodosis_mood" =
    sentence.type == "jumlah_shartiyyah"
    and token.role == "jaza"
    and (token.features.mood == "jussive"
         or token.features.tense == "past")

// --- Command Response Mood ---

predicate "command_response_is_jussive" =
    token.features.pos == "verb"
    and token.features.tense == "present"
    and token.role == "jaza"
    and governing_particle.features.root in ["ف", "و"]
    and token.features.mood == "jussive"
```

### 7.6 Construction Library (`constructions.agosrule`)

```ebnf
// ================================================
// lib/constructions.agosrule
// Construction Predicates
// Version: 1.0.0
// ================================================

// --- Idafa (Construct State) ---

predicate "is_idafa_construction" =
    exists(constituent in sentence.constituents {
        constituent.role == "idafa"
    })

predicate "idafa_first_term_definite" =
    token.role == "mudaf"
    and token.features.state == "definite"

predicate "idafa_first_term_no_tanwin" =
    token.role == "mudaf"
    and not (token.text matches ".*ً.*|.*ٌ.*|.*ٍ.*")  // No tanwin marks

// --- Wasf (Adjective Agreement) ---

predicate "naat_follows_manut" =
    token.role == "na'at"
    and exists(constituent in sentence.constituents {
        constituent.role == "atasf"
        and token.index in constituent.token_indices
    })

// --- Tawkid (Emphasis) ---

predicate "is_tawkid_construction" =
    exists(constituent in sentence.constituents {
        constituent.role == "ta'kid"
    })

// --- Badal (Apposition) ---

predicate "badal_follows_mubdal" =
    token.role == "badal"
    and preceding_token.features.case == token.features.case

// --- Istithna (Exception) ---

predicate "istithna_with_illa" =
    token.features.root == "إلا"
    and token.features.pos == "particle"

// --- Nida (Vocative) ---

predicate "nida_with_ya" =
    token.features.root == "يا"
    and token.features.pos == "particle"
```

### 7.7 Particle Library (`particles.agosrule`)

```ebnf
// ================================================
// lib/particles.agosrule
// Particle Predicates
// Version: 1.0.0
// ================================================

// --- Preposition Checks ---

predicate "preposition_governs_noun" =
    token.features.pos == "particle"
    and token.role == "harf_jarr"
    and following_token.features.pos == "noun"

// --- Conjunction Checks ---

predicate "wa_conjoins_equal" =
    token.features.root == "و"
    and token.role == "conjunction"
    and preceding_token.features.case == following_token.features.case

// --- Interrogative Checks ---

predicate "interrogative_hamza" =
    token.features.root == "أ"
    and token.features.pos == "interrogative"

// --- Subjunctive Particles ---

predicate "nasb_particle" =
    token.features.pos == "particle"
    and token.features.root in ["أن", "لن", "كي", "إذن"]

// --- Jussive Particles ---

predicate "jazm_particle" =
    token.features.pos == "particle"
    and token.features.root in ["لم", "لما", "لا", "ل"]

// --- Inna and Sisters ---

predicate "inna_and_sisters" =
    token.features.pos == "particle"
    and token.features.root in ["إن", "أن", "لكن", "كأن", "ليت", "لعل"]
```

### 7.8 Validation Library (`validation.agosrule`)

```ebnf
// ================================================
// lib/validation.agosrule
// Input Validation Predicates
// Version: 1.0.0
// ================================================

predicate "sentence_has_tokens" =
    sentence.tokens.length > 0

predicate "sentence_not_too_long" =
    sentence.tokens.length <= 200  // Same limit as MOD-05 max_sentence_length

predicate "has_analyzable_content" =
    no_unknown_tokens()

predicate "minimum_viable_sentence" =
    sentence.tokens.length >= 2
    and (is_verbal() or is_nominal())

predicate "single_sentence" =
    not exists(punctuation in sentence.tokens {
        punctuation.text in [".", "!", "?", "؛"]
        and punctuation.position > 1  // Ignore sentence-final punctuation
    })
```

---

## 8. School-Specific Rule Pattern Catalog

### 8.1 Pattern: Subject-Verb Agreement (All Schools)

The patterns in this section are **authoring templates** — they use `{school}` and `{version}` as placeholder variables that rule authors replace with concrete values when creating school-specific rule files. RFC-0001 does not define a macro or template system; these placeholders are a documentation convention.

```ebnf
// ================================================
// Pattern: Subject-Verb Agreement
// Applicable: All schools
// Category: agreement (01)
// Priority range: 71–90
// ================================================

// Pattern template for person agreement
rule "{school}-0101: تطابق الفعل والفاعل في الشخص — Person Agreement" {
    metadata {
        id: "{school}-0101"
        school: "{school}"
        version: "{version}"
        priority: 85
        description: "In verbal sentences, the verb must agree with the subject in person."
        source: "{source reference}"
        tags: ["agreement", "person", "fi'l", "fa'il"]
        category: "agreement"
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        and exists(fa'il)
        and fi'l.person != fa'il.person
    }

    action {
        reject("الفعل والفاعل لا يتطابقان في الشخص — Subject-verb person disagreement")
        flag("error", "SUBJECT_VERB_PERSON_MISMATCH", fi'l, fa'il)
    }
}

// Pattern variation for number agreement (Basra position)
rule "{school}-0102: تطابق الفعل والفاعل في العدد — Number Agreement" {
    metadata {
        id: "{school}-0102"
        school: "{school}"
        version: "{version}"
        priority: 80
        description: "Verb agrees with subject in number. Exception: when verb precedes subject, verb is singular (Basra)."
        source: "Sibawayh, Al-Kitab, Vol. 1, p. 240"
        tags: ["agreement", "number"]
        category: "agreement"
    }

    condition {
        sentence.type == "jumlah_fi'liyyah"
        and exists(fa'il)
        and fi'l.position < fa'il.position       // Verb before subject
        and fi'l.number == "plural"
    }

    action {
        // Basra: verb before plural subject takes singular
        modify(fi'l.number, "singular")
        confirm()
    }
}
```

### 8.2 Pattern: Case Assignment (All Schools)

```ebnf
// ================================================
// Pattern: Case Assignment — Preposition Governs Genitive
// Applicable: All schools
// Category: case-assignment (02)
// Priority: 65
// ================================================

rule "{school}-0201: جر بحرف الجر — Preposition Governs Genitive" {
    metadata {
        id: "{school}-0201"
        school: "{school}"
        version: "{version}"
        priority: 65
        description: "A noun governed by a preposition takes the genitive case."
        source: "Alfiyya Ibn Malik: 'حروف الجر تجر الاسم'"
        tags: ["case", "preposition", "genitive"]
        category: "case-assignment"
    }

    condition {
        token.role == "majrur"
        and governing_particle.role == "harf_jarr"
        and token.features.case != "genitive"
    }

    action {
        modify(token.features.case, "genitive")
        confirm(token)
    }
}
```

### 8.3 Pattern: Mood Government (All Schools)

```ebnf
// ================================================
// Pattern: Mood Government — Lam Governs Jussive
// Applicable: All schools
// Category: mood-government (03)
// Priority: 45
// ================================================

rule "{school}-0301: جزم الفعل المضارع بلم — Jussive by Lam" {
    metadata {
        id: "{school}-0301"
        school: "{school}"
        version: "{version}"
        priority: 45
        description: "لَمْ (lam) governs the jussive mood on the present tense verb."
        evidence: "Quran 2:7: وَلَمْ يُؤْمِنُوا۟ (and they did not believe)"
        tags: ["mood", "jussive", "lam"]
        category: "mood-government"
    }

    condition {
        exists(fi'l)
        and fi'l.tense == "present"
        and exists(governing_particle)
        and governing_particle.text == "لَمْ"
        and fi'l.mood != "jussive"
    }

    action {
        modify(fi'l.mood, "jussive")
        confirm()
    }
}
```

### 8.4 Pattern: Idafa Construction (All Schools)

```ebnf
// ================================================
// Pattern: Idafa — Genitive Assignment
// Applicable: All schools
// Category: constructions (04)
// Priority: 25
// ================================================

rule "{school}-0401: جر المضاف إليه — Genitive in Idafa" {
    metadata {
        id: "{school}-0401"
        school: "{school}"
        version: "{version}"
        priority: 25
        description: "The second term of an idafa (construct state) takes the genitive case."
        evidence: "Quran 1:2: رَبِّ ٱلْعَـٰلَمِينَ (Lord of the worlds)"
        tags: ["construction", "idafa", "genitive"]
        category: "constructions"
    }

    condition {
        token.role == "mudaf_ilayh"
        and token.features.case != "genitive"
    }

    action {
        modify(token.features.case, "genitive")
        confirm(token)
    }
}
```

### 8.5 Pattern: Kana and Her Sisters (All Schools)

```ebnf
// ================================================
// Pattern: Kana and Her Sisters
// Applicable: All schools
// Category: special-verbs (06)
// Priority: 55
// ================================================

rule "{school}-0601: كان وأخواتها — Kana Case Reassignment" {
    metadata {
        id: "{school}-0601"
        school: "{school}"
        version: "{version}"
        priority: 55
        description: "Kana and its sisters reassign case: subject (ism kana) takes nominative, predicate (khabar kana) takes accusative."
        evidence: "Quran 2:177: وَكَانَ ٱللَّهُ غَفُورًا رَّحِيمًا (And Allah is Forgiving, Merciful)"
        source: "Sibawayh, Al-Kitab, Vol. 2"
        tags: ["kana", "special-verb", "case"]
        category: "special-verbs"
    }

    condition {
        exists(fi'l)
        and fi'l.features.root in ["كان", "صار", "ليس", "أصبح", "أمسى",
                                   "ظل", "بات", "ما زال", "ما انفك",
                                   "ما فتئ", "ما برح", "ما دام"]
        and sentence.type == "jumlah_fi'liyyah"
    }

    action {
        modify(sentence.type, "jumlah_kāna")
        if (exists(mubtada')) {
            // Ism kana: subject remains nominative (already assigned)
            confirm(mubtada')
        }
        if (exists(khabar)) {
            // Khabar kana: predicate takes accusative
            modify(khabar.features.case, "accusative")
            confirm(khabar)
        }
        flag("info", "KANA_CONSTRUCTION", sentence)
    }
}

// ================================================
// Pattern: Inna and Her Sisters
// Category: special-verbs (06)
// Priority: 55
// ================================================

rule "{school}-0602: إن وأخواتها — Inna Case Reassignment" {
    metadata {
        id: "{school}-0602"
        school: "{school}"
        version: "{version}"
        priority: 55
        description: "Inna and its sisters reassign case: subject (ism inna) takes accusative, predicate (khabar inna) remains nominative."
        evidence: "Quran 2:255: إِنَّ ٱللَّهَ لَا إِلَـٰهَ إِلَّا هُوَ (Indeed, Allah, there is no deity except Him)"
        tags: ["inna", "special-particle", "case"]
        category: "special-verbs"
    }

    condition {
        exists(governing_particle)
        and governing_particle.features.root in ["إن", "أن", "لكن", "كأن",
                                                  "ليت", "لعل"]
        and (sentence.type == "jumlah_ismiyyah" or sentence.type == "jumlah_fi'liyyah")
    }

    action {
        modify(sentence.type, "jumlah_inna")
        if (exists(mubtada')) {
            // Ism inna: subject takes accusative
            modify(mubtada'.features.case, "accusative")
        }
        if (exists(khabar)) {
            // Khabar inna: predicate remains nominative
            confirm(khabar)
        }
        flag("info", "INNA_CONSTRUCTION", sentence)
    }
}
```

### 8.6 Pattern: Conditional Sentences (All Schools)

```ebnf
// ================================================
// Pattern: Conditional Sentences
// Applicable: All schools
// Category: conditional (05)
// Priority: 35
// ================================================

rule "{school}-0501: إِن الشرطية — Conditional In Governs Jussive" {
    metadata {
        id: "{school}-0501"
        school: "{school}"
        version: "{version}"
        priority: 35
        description: "The conditional particle إِنْ (in) governs the jussive mood on both the condition (shart) and result (jaza') verbs."
        evidence: "Quran 2:282: إِن تَكُونُوا۟ تَعْلَمُونَ (if you know)"
        tags: ["conditional", "mood", "shart", "jaza"]
        category: "conditional"
    }

    condition {
        sentence.type == "jumlah_shartiyyah"
        and exists(governing_particle)
        and governing_particle.text == "إِنْ"
        and exists(constituent in sentence.constituents {
            constituent.role == "shart"
        })
        and exists(constituent in sentence.constituents {
            constituent.role == "jaza"
        })
        // Both verbs should be in jussive
    }

    action {
        if (shart_verb.features.mood != null and shart_verb.features.mood != "jussive") {
            modify(shart_verb.features.mood, "jussive")
        }
        if (jaza_verb.features.mood != null and jaza_verb.features.mood != "jussive") {
            modify(jaza_verb.features.mood, "jussive")
        }
        confirm()
    }
}
```

### 8.7 Pattern: Vocative Construction (All Schools)

```ebnf
// ================================================
// Pattern: Nida (Vocative)
// Applicable: All schools
// Category: constructions (04)
// Priority: 25
// ================================================

rule "{school}-0408: نصب المنادى — Vocative Case Assignment" {
    metadata {
        id: "{school}-0408"
        school: "{school}"
        version: "{version}"
        priority: 25
        description: "The noun following a vocative particle (يَا, أَ, أَيُّهَا) takes the accusative case when definite, or is built on the accusative."
        evidence: "Quran 12:29: يُوسُفُ أَعْرِضْ عَنْ هَـٰذَا (Joseph, turn away from this)"
        tags: ["construction", "nida", "vocative", "case"]
        category: "constructions"
    }

    condition {
        exists(constituent in sentence.constituents {
            constituent.role == "nida"
        })
        and token.features.state == "definite"
        and token.role == "nida"
        and token.features.case != "accusative"
    }

    action {
        modify(token.features.case, "accusative")
        confirm(token)
    }
}
```

---

## 9. Rule Testing Framework

### 9.1 Test Fixture Format

Each test fixture is a JSON file that defines a test sentence, its analysis, and the expected rule outcomes:

```json
{
    "spec": "RFC-0004/test-fixture",
    "version": "1.0",
    "name": "basra-agreement-person-match",
    "description": "Verbal sentence with matching person (third person verb + third person subject)",
    "school": "basra",
    "text": "كَتَبَ مُحَمَّدٌ رِسَالَةً",
    "translation": "Muhammad wrote a letter",
    "evidence_source": "Quran 2:246 (وَقَالَ ٱلَّذِينَ)",

    "gir_snapshot": {
        "tokens": [
            {
                "index": 0,
                "text": "كَتَبَ",
                "features": {
                    "pos": "verb",
                    "gender": "masculine",
                    "number": "singular",
                    "person": "third",
                    "tense": "past",
                    "voice": "active"
                },
                "role": "fi'l",
                "position": 1
            },
            {
                "index": 1,
                "text": "مُحَمَّدٌ",
                "features": {
                    "pos": "proper_noun",
                    "gender": "masculine",
                    "number": "singular",
                    "person": "third",
                    "case": "nominative",
                    "state": "indefinite"
                },
                "role": "fa'il",
                "position": 2
            },
            {
                "index": 2,
                "text": "رِسَالَةً",
                "features": {
                    "pos": "noun",
                    "gender": "feminine",
                    "number": "singular",
                    "case": "accusative",
                    "state": "indefinite"
                },
                "role": "maf'ul_bi-hi",
                "position": 3
            }
        ],
        "sentence_type": "jumlah_fi'liyyah",
        "tree": {
            "type": "jumlah_fi'liyyah",
            "root": {
                "role": "jumlah_fi'liyyah",
                "token_indices": [0, 1, 2]
            }
        }
    },

    "expected_applications": [
        {
            "rule_id": "basra-0101",
            "status": "not_applied",
            "reason": "Person matches (third == third), condition not triggered"
        },
        {
            "rule_id": "basra-0202",
            "status": "confirm",
            "target": "token:1",
            "detail": "Subject fa'il takes nominative case"
        },
        {
            "rule_id": "basra-0203",
            "status": "confirm",
            "target": "token:2",
            "detail": "Direct object maf'ul_bi-hi takes accusative case"
        }
    ],

    "expected_flags": [],
    "expected_ambiguity_remaining": 1
}
```

### 9.2 Test Categories

| Category | Description | Test Count (per school) |
|----------|-------------|------------------------|
| **Agreement** | Person, number, gender agreement between verb-subject and noun-adjective | 25–40 |
| **Case Assignment** | Nominative, accusative, genitive assignment by various governors | 35–50 |
| **Mood Government** | Indicative, subjunctive, jussive by governing particles | 20–30 |
| **Constructions** | Idafa, wasf, tawkid, badal, istithna, nida | 15–25 |
| **Conditional** | Shart/jaza structure, mood interaction | 10–15 |
| **Special Verbs** | Kana, inna, zanna and their sisters | 10–15 |
| **Ambiguity** | Sentences with multiple valid parses | 10–20 |
| **Poetic Necessity** | Darura constructions that override normal rules | 5–10 |
| **Negative Tests** | Sentences that should not trigger specific rules | 15–25 |
| **Corner Cases** | Edge-case sentences (minimal, fragment, unusual) | 10–15 |

**Total per school:** ~155–245 tests.

### 9.3 Test Harness

```bash
# Run all tests for a school
agos rule test --school=basra

# Run tests for a specific category
agos rule test --school=basra --category=agreement

# Run a single test
agos rule test --fixture=tests/basra/agreement/person-match.json

# Run cross-school comparison
agos rule compare --school1=basra --school2=kufa --sentence=fixtures/ambiguous.json

# Generate coverage report
agos rule coverage --school=basra --output=coverage.html
```

### 9.4 Example: Agreement Test Suite (Basra)

```json
[
    {
        "name": "basra-agreement-01",
        "description": "Person match — third person verb with third person subject",
        "text": "كَتَبَ ٱلرَّجُلُ (The man wrote)",
        "expected_rules": ["basra-0101:not_applied"]
    },
    {
        "name": "basra-agreement-02",
        "description": "Person mismatch — third person verb with second person subject",
        "text": "*يَكْتُبُ أَنْتَ (incorrect: You write)",
        "expected_rules": ["basra-0101:reject"],
        "expected_flags": ["SUBJECT_VERB_PERSON_MISMATCH"]
    },
    {
        "name": "basra-agreement-03",
        "description": "Number agreement — verb before plural subject takes singular (Basra)",
        "text": "ذَهَبَ ٱلرِّجَالُ (The men went)",
        "expected_rules": ["basra-0104:modify:fi'l.number->singular"]
    },
    {
        "name": "basra-agreement-04",
        "description": "Non-human plural subject — feminine singular verb",
        "text": "طَالَتِ ٱلْأَيَّامُ (The days became long)",
        "expected_rules": ["basra-0105:modify:fi'l.gender->feminine"]
    },
    {
        "name": "basra-agreement-05",
        "description": "Nominal sentence — mubtada' and khabar agreement",
        "text": "ٱلْبَيْتُ كَبِيرٌ (The house is big)",
        "expected_rules": ["basra-0106:confirm"]
    }
]
```

### 9.5 Example: Case Assignment Test Suite (Basra)

```json
[
    {
        "name": "basra-case-01",
        "description": "Preposition governs genitive",
        "text": "فِي ٱلْبَيْتِ (In the house)",
        "expected_rules": ["basra-0204:modify:al-bayt.case->genitive"]
    },
    {
        "name": "basra-case-02",
        "description": "Subject takes nominative",
        "text": "جَاءَ ٱلرَّجُلُ (The man came)",
        "expected_rules": ["basra-0202:confirm:al-rajul"]
    },
    {
        "name": "basra-case-03",
        "description": "Direct object takes accusative",
        "text": "أَكَلَ ٱلْوَلَدُ ٱلتُّفَّاحَةَ (The boy ate the apple)",
        "expected_rules": ["basra-0203:modify:tuffaha.case->accusative"]
    },
    {
        "name": "basra-case-04",
        "description": "Idafa — second term in genitive",
        "text": "كِتَابُ ٱلْمُدَرِّسِ (The teacher's book)",
        "expected_rules": ["basra-0401:modify:mudarris.case->genitive"]
    }
]
```

---

## 10. DSL Compilation Pipeline

### 10.1 From Source to Executable Rules

```
DSL Source (.agosrule files)
    │
    ▼
┌────────────────────────────────────────────┐
│  1. PARSING                                 │
│                                              │
│  Input:  .agosrule UTF-8 text files         │
│  Process:                                   │
│  • Lexical analysis (tokenize source)       │
│  • Syntactic parsing (RFC-0001 EBNF)        │
│  • Build AST (Abstract Syntax Tree)         │
│                                              │
│  Output: RuleAST (in-memory tree)           │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  2. RESOLUTION                              │
│                                              │
│  Input:  Multiple RuleASTs (import chain)   │
│  Process:                                   │
│  • Resolve imports (file path → AST)        │
│  • Detect circular imports                  │
│  • Resolve predicate references             │
│  • Expand predicates inline                 │
│  • Resolve role shorthand → full paths      │
│  • Check for duplicate rule IDs             │
│                                              │
│  Output: ResolvedRuleSet (unified AST)      │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  3. TYPE CHECKING & VALIDATION              │
│                                              │
│  Input:  ResolvedRuleSet                    │
│  Process:                                   │
│  • Type check all expressions               │
│  • Validate feature names against KB-0007   │
│  • Validate metadata fields                 │
│  • Check regex patterns (ReDoS safety)      │
│  • Validate priority range (0–100)          │
│  • Check for undefined variables            │
│                                              │
│  Output: ValidatedRuleSet                   │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  4. OPTIMIZATION                            │
│                                              │
│  Input:  ValidatedRuleSet                   │
│  Process:                                   │
│  • Constant folding (compile-time eval)     │
│  • Dead condition elimination               │
│  • Predicate inlining (already done)        │
│  • Index common paths for O(1) access       │
│  • Sort rules by priority (descending)      │
│  • Alphabetical tie-breaking within priority│
│                                              │
│  Output: OptimizedRuleSet                   │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  5. COMPILATION                             │
│                                              │
│  Input:  OptimizedRuleSet                   │
│  Process:                                   │
│  • Compile conditions to condition DAGs     │
│  • Compile actions to effect lists          │
│  • Generate rule metadata table             │
│  • Serialize to CompiledRuleSet binary      │
│                                              │
│  Output: CompiledRuleSet (.agosrules)       │
└────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────┐
│  6. PACKAGING (Plugin Format)              │
│                                              │
│  Input:  CompiledRuleSet (.agosrules)       │
│  Process:                                   │
│  • Wrap in plugin manifest                  │
│  • Add version metadata                     │
│  • Generate plugin checksum                 │
│  • Package as WASM module or native binary  │
│                                              │
│  Output: RuleSetPlugin (.wasm / .so)        │
└────────────────────────────────────────────┘
    │
    ▼
MOD-07 (RuleEngine) loads and executes
```

### 10.2 CompiledRuleSet Binary Format

The compiled rule set format (`.agosrules`) is a binary container that MOD-07 loads at initialization:

```ebnf
CompiledRuleSet = {
    magic: "AGRS",                       // 4 bytes: 0x41475253
    version: { major: u16, minor: u16, patch: u16 },

    // Metadata
    school: string,                      // "basra", "kufa", etc.
    rule_count: u32,
    plugin_version: string,              // Plugin API version

    // Condition DAG (shared sub-expressions)
    condition_count: u32,
    conditions: [                        // Array of condition DAG nodes
        {
            node_id: u32,                // Node index
            node_type: u8,               // AND | OR | NOT | COMPARE | REGEX | EXISTS | FORALL
            operands: u32[],             // Child node IDs or operand indices
            literal: string | null,      // Literal value (for leaf nodes)
        }
    ],

    // Rule table
    rules: [
        {
            rule_id: string,             // e.g., "basra-0103"
            priority: u8,                // 0–100
            condition_root: u32,         // Index into conditions[] (root node)
            actions: [                   // Action sequence
                {
                    action_type: u8,     // 0=confirm, 1=reject, 2=modify, 3=flag, 4=resolve
                    target_path: u32[],  // Path expression as operand stack
                    value: string | null, // Literal value (for modify, flag)
                }
            ],
            metadata: {
                school: string,
                version: string,
                description: string,
                tags: string[],
                category: string,
            }
        }
    ],

    // String table
    string_count: u32,
    strings: string[],                   // Deduplicated strings

    // End marker
    end_marker: "AGRS_END",              // 8 bytes
}
```

### 10.3 Compilation Performance Targets

| Stage | Target | Notes |
|-------|--------|-------|
| **Full compilation** (850 rules) | < 5 seconds | Single school, development machine |
| **Incremental compilation** (single file change) | < 200 ms | Only recompile affected files |
| **Parsing** | > 1 MB/s | DSL source throughput |
| **Type checking** | > 500 rules/s | With KB-0007 validation |
| **Compiled rule set size** | < 5 MB | Basra school (850 rules) |
| **Loading by MOD-07** | < 100 ms | Load from local filesystem |
| **Caching** | < 10 ms | Warm cache hit |

---

## 11. Performance Benchmarks for Arabic Rule Sets

### 11.1 Rule Complexity by Category

| Category | Avg Condition Depth | Avg Actions per Rule | Typical `forall` Iterations | Est. Execution Time (per rule) |
|----------|-------------------|---------------------|---------------------------|-------------------------------|
| Agreement | 4–6 | 1–2 | 2–10 | 10–50 μs |
| Case Assignment | 3–5 | 1 | 1–5 | 5–30 μs |
| Mood Government | 4–6 | 1–2 | 1–5 | 10–40 μs |
| Constructions | 5–8 | 1–3 | 3–15 | 20–80 μs |
| Conditional | 6–10 | 2–4 | 2–8 | 30–100 μs |
| Special Verbs | 6–8 | 2–4 | 2–6 | 30–80 μs |
| Ambiguity Resolution | 3–5 | 1 | 5–20 | 20–60 μs |

### 11.2 Full Pipeline Estimates

| Scenario | Rules Applied | Tokens | Est. Time | Target |
|----------|--------------|--------|-----------|--------|
| Simple verbal sentence | 10–20 | 3–5 | 50–200 μs | < 500 μs (p50) |
| Complex nominal sentence | 30–60 | 5–15 | 200–800 μs | < 500 μs (p50) |
| Conditional with ambiguity | 50–100 | 10–20 | 500–2000 μs | < 5 ms (p99) |
| Long Quranic verse | 80–150 | 20–40 | 1–5 ms | < 5 ms (p99) |
| Poetic line (complex) | 100–200 | 10–25 | 2–10 ms | < 10 ms (p99) |

### 11.3 Optimization Recommendations

| Optimization | Impact | Category |
|-------------|--------|----------|
| **Short-circuit evaluation** | 2–5× speedup for `and` chains | All categories |
| **Condition DAG deduplication** | 10–30% fewer evaluations | Agreement, Case |
| **Role shorthand indexing** | O(1) shorthand resolution instead of O(n) scan | All categories |
| **Regex precompilation** | 5–10× faster `matches` | Special verbs, Particles |
| **Batch rejection** | Reject all analyses matching pattern in one operation | Ambiguity resolution |

---

## 12. Migration from Traditional Grammar Sources

### 12.1 Mapping Traditional Sources to DSL Rules

This section provides guidance for translating classical Arabic grammar texts into DSL rule sets.

#### Sibawayh's Al-Kitab (كتاب سيبويه)

```ebnf
// Source structure:
// Al-Kitab is organized by grammatical topic (باب), not by rule.
// Each باب discusses multiple related rules.

// Migration approach:
// 1. Identify each distinct grammatical principle in a باب
// 2. Create one DSL rule per principle
// 3. Document the باب and page reference in the metadata.source field
// 4. Add Quranic or poetic evidence cited by Sibawayh

// Example migration:
// Source: Al-Kitab, Vol. 1, باب الفاعل (Chapter on the Subject)
// Principle: "الفاعل مرفوع" (The subject is in the nominative case)
// Evidence: "جَاءَ زَيْدٌ" (Zayd came)
// Ruling: Basra position

rule "basra-0202: رفع الفاعل — Subject Takes Nominative" {
    metadata {
        id: "basra-0202"
        school: "basra"
        version: "1.0.0"
        priority: 60
        description: "The subject (fa'il) of a verb takes the nominative case."
        source: "Sibawayh, Al-Kitab, Vol. 1, باب الفاعل, p. 156"
        evidence: "جَاءَ زَيْدٌ (Zayd came) — Sibawayh's canonical example"
        tags: ["case", "subject", "nominative"]
        category: "case-assignment"
    }

    condition {
        token.role == "fa'il"
        and token.features.case != "nominative"
    }

    action {
        modify(token.features.case, "nominative")
        confirm()
    }
}
```

#### Ibn Malik's Alfiyya (ألفية ابن مالك)

```ebnf
// Source structure:
// The Alfiyya is a poetic summary of grammar (~1,000 verses).
// Each verse (بيت) states one or more grammatical rules.

// Migration approach:
// 1. Parse each بيت into its constituent rules
// 2. Look up the شرح (commentary) for full rule semantics
// 3. Create one DSL rule per grammatical principle within the بيت
// 4. Reference the بيت number in metadata.source

// Example:
// بيت رقم 26: "وَٱلْفَاعِلُ ٱلَّذِي هُوَ مَرْفُوعُ ..."
// "The subject (fa'il) is in the nominative case..."
// This matches the same rule as basra-0202 but from a different source.

rule "basra-0202" {
    metadata {
        source: "Ibn Malik, Alfiyya, Bayt 26; Sibawayh, Al-Kitab, Vol. 1, p. 156"
        // Same rule, additional source reference
    }
    // ... same condition and action
}
```

#### Quranic Evidence as Test Fixtures

```ebnf
// Quranic verses provide the highest-quality test evidence.
// Each major rule SHOULD have at least one Quranic test case.

// Example: Test fixture from Quran 2:255 (Ayat al-Kursi)
{
    "name": "quranic-inna-construction",
    "description": "Inna construction in Ayat al-Kursi",
    "text": "إِنَّ ٱللَّهَ لَا إِلَـٰهَ إِلَّا هُوَ",
    "surah": 2,
    "ayah": 255,
    "expected_rules": [
        "basra-0602:modify:sentence.type->jumlah_inna",
        "basra-0602:modify:Allah.case->accusative"
    ]
}
```

### 12.2 Known Divergences Between Sources

| Topic | Sibawayh (Basra) | Al-Kisa'i (Kufa) | Ibn Malik (Late Basra) | Modern Practice |
|-------|-----------------|-------------------|----------------------|-----------------|
| Verb before plural subject | Verb singular | Verb may be plural | Verb singular | Verb singular (follows Basra) |
| إِنَّ case assignment | Ism accusative | Varies | Ism accusative | Ism accusative |
| Conditional إِنْ mood | Jussive both | Same | Same | Same (consensus) |
| Exception with إِلَّا | Accusative | Accusative or nominative | Accusative | Accusative |
| Vocative يَا case | Accusative | Accusative | Accusative | Accusative |
| Idafa tanwin drop | Required | Required | Required | Required (consensus) |

### 12.3 Automated Migration Tool

```bash
# Extract rules from structured commentary
agos-rule migrate --source=sibawayh-v2.xml --output=rules/basra/

# Convert traditional grammatical notation to DSL
agos-rule translate --notation="الفاعل مرفوع" \
    --rule-id="basra-0202" \
    --school=basra \
    --output=rule.dsl

# Validate against known test fixtures
agos-rule validate --rules=rules/basra/ --tests=tests/basra/

# Generate migration report
agos-rule migrate-report --source=sibawayh-v2.xml --output=migration.html
```

---

## 13. Versioning & Rule Set Distribution

### 13.1 Rule Set Versioning

Each school's rule set follows semantic versioning:

```
{major}.{minor}.{patch}

Major: Breaking rule changes (rule removed, condition semantics changed)
Minor: New rules added, non-breaking refinements
Patch: Bug fixes, test additions, documentation
```

### 13.2 Version Compatibility

| Rule Set Version | MOD-07 Version | Compatibility |
|------------------|----------------|---------------|
| 1.x | 1.x | Full compatibility |
| 2.x | 2.x+ | Breaking changes may require MOD-07 update |
| 1.x | 2.x | May work with deprecation warnings |

### 13.3 Distribution Format

Rule sets are distributed as versioned plugin packages:

```
basra-rules-1.2.3.agosrules       # Compiled rule set
basra-rules-1.2.3.manifest.json   # Plugin manifest (SPEC-0001-C7)
basra-rules-1.2.3.checksum.sha256 # Integrity check
```

### 13.4 Rule Set Dependency Management

```ebnf
// Rule sets can depend on common libraries:
// basra-rules v1.2.3 depends on agos-stdlib v1.0.0

// Manifest:
{
    "id": "basra-rules",
    "version": "1.2.3",
    "api_version": "1.0",
    "school": "basra",
    "rule_count": 850,
    "dependencies": {
        "agos-stdlib": ">=1.0.0 <2.0.0"
    }
}
```

---

## 14. Cross-References

### 14.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| RFC-0001 | Grammar DSL | DSL syntax and semantics that this RFC builds upon |
| SPEC-0001-C3 §8 | MOD-07 RuleEngine Pipeline | Rule execution algorithm, fixpoint detection |
| SPEC-0001-C4 §9 | MOD-07 Interface | RuleEngineInput, AnnotatedGIR, error codes |
| SPEC-0001-C5 §8 | IR-7 AnnotatedGIR Schema | Rule applications, flags, evidence trail |
| SPEC-0001-C7 | Plugin Architecture | `rule_set` plugin type, RuleSetPlugin interface |
| SPEC-0201 | Rule Engine | Detailed execution engine that consumes compiled rules |
| SPEC-0101 §14 | School-Specific Morphology | School comparison table, morphological differences |
| KB-0005 | Particles | Particle governance rules, homograph resolution |
| KB-0006 | Pronouns | Pronoun types, anaphora resolution |
| KB-0007 | Morphological Features | Feature taxonomy, valid values for conditions |
| ADR-0001 | Compiler Architecture | Why rule engine is a separate stage |
| ADR-0003 | Grammar IR | Why GIR is the rule engine input |

### 14.2 External References

| Reference | Relevance |
|-----------|-----------|
| Sibawayh, Al-Kitab (الكتاب) | Primary source for Basra school rules |
| Ibn Malik, Alfiyya (ألفية ابن مالك) | Canonical grammatical poem, primary reference |
| Al-Kisa'i, Grammar of Kufa | Primary source for Kufa school rules |
| Ibn al-Sarraj, Al-Usul (الأصول في النحو) | Basra school methodology |
| Al-Jurjani, Al-Awamil (العوامل) | 'Amil theory framework |
| Suyuti, Al-Ashbah wa al-Naza'ir (الأشباه والنظائر) | Comparative grammar across schools |
| Hasan, Al-Nahw al-Wafi (النحو الوافي) | Modern comprehensive grammar reference |
| Wright, Arabic Grammar (Caspari) | Standard Western reference grammar |

---

## Progress Summary

**RFC-0004: Arabic Grammar Rule DSL**

| Section | Title | Status |
|---------|-------|--------|
| 1 | Introduction & Scope | ✓ COMPLETE |
| 2 | Relationship to RFC-0001 | ✓ COMPLETE |
| 3 | Rule Authoring Conventions | ✓ COMPLETE |
| 4 | Rule Categories & Priority Allocation | ✓ COMPLETE |
| 5 | Arabic Grammatical Terminology Mapping | ✓ COMPLETE |
| 6 | Complete School Catalogs | ✓ COMPLETE |
| 7 | Arabic Standard Library Specification | ✓ COMPLETE |
| 8 | School-Specific Rule Pattern Catalog | ✓ COMPLETE |
| 9 | Rule Testing Framework | ✓ COMPLETE |
| 10 | DSL Compilation Pipeline | ✓ COMPLETE |
| 11 | Performance Benchmarks for Arabic Rule Sets | ✓ COMPLETE |
| 12 | Migration from Traditional Grammar Sources | ✓ COMPLETE |
| 13 | Versioning & Rule Set Distribution | ✓ COMPLETE |
| 14 | Cross-References | ✓ COMPLETE |

**Dependencies:** RFC-0001, SPEC-0001-C3/C4/C5/C7, SPEC-0201, SPEC-0101, KB-0005/6/7, ADR-0001/3.

**Recommended next step:** SPEC-0601 (Plugin System) — the detailed specification for MOD-12 (PluginLoader) that manages `rule_set` plugin lifecycle, sandboxing, and distribution.
