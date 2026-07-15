# How to Read the AGOS Specifications (For Non-Technical Readers)

This guide helps you navigate the AGOS specification suite — **39 documents, ~82,000 lines** — without getting lost in implementation details, binary format schemas, or opcode tables. You'll know exactly what to read, what to skim, and what to skip.

---

## First: Understand What AGOS Is

AGOS is not an app. It's a **platform** — a foundation that other applications (I'rab analyzers, grammar tutors, Quran/Hadith explorers) can be built on top of.

Think of it like this:
- **Android** is a platform; WhatsApp, Instagram, and Chrome are apps built on it.
- **AGOS** is a platform; an I'rab Analyzer or Nahwu Tutor are apps that would be built on it.

AGOS takes Arabic text and runs it through a **pipeline** (like a factory assembly line) where each station does one specific job:

```
Arabic text → clean it → split into words → analyze each word →
figure out grammar → check rules → generate explanation
```

Everything is **deterministic** — the same input always produces the same output. No AI guesses. Every answer is traceable to a specific grammar rule.

---

## Spec Document Types (Plain English)

| Type | Full Name | What It Is | Example |
|------|-----------|------------|---------|
| **ADR** | Architecture Decision Record | "Why we decided to build it this way" | Why use a compiler? Why offline-first? |
| **RFC** | Request for Comments | "A proposal for something new" | A language for writing grammar rules |
| **SPEC** | Specification | "How exactly to build it" | The blueprint for each component |
| **KB** | Knowledge Base | "The linguistic data AGOS needs" | List of Arabic roots, verb patterns, particles |

---

## What to Read (Path for Non-Technical Readers)

Read in this order. Each item tells you which sections to focus on and which to skip.

### Step 1: The Big Picture — SPEC-0001 Chapter 1

**File:** `SPEC/SPEC-0001/01-introduction-and-scope.md`

This is the most important document for you. Read it nearly in full.

| Section | Read? | Why |
|---------|-------|-----|
| §1 Purpose | ✅ Full | What AGOS is for |
| §2 Scope | ✅ Full | What it includes and excludes |
| §3 Audience | ✅ Full | Who this is for |
| §4 What AGOS Is | ✅ Full | The vision |
| §5 What AGOS Is Not | ✅ Full | Critical — prevents misunderstanding |
| §6 Terminology | ✅ Full | Key terms defined |
| §7 Design Rationale | ✅ Full | Why it works this way |
| §8 Core Principles | ✅ Full | The 12 rules that govern everything |
| §9 Roadmap | ⚡ Skim | Just note what's planned |
| §10-11 Conventions | ⚡ Skim | For technical readers |

**Takeaway after reading:** You should understand what AGOS is, what it isn't, and the 12 core principles.

---

### Step 2: Why Compiler Architecture — ADR-0001

**File:** `ADR/ADR-0001-compiler-architecture-rationale.md`

This explains the biggest decision: why AGOS uses a compiler-style pipeline instead of AI/NLP.

| Section | Read? | Why |
|---------|-------|-----|
| §1 Context | ✅ Full | The problem they were solving |
| §2 Problem Statement | ✅ Full | What question they asked |
| §3 Decision | ✅ Full | What they chose |
| §4 Alternatives Considered | ✅ Full | What they rejected and why |
| §5 Detailed Rationale | ⚡ Skim | Gets technical; read first few paragraphs |
| §6 Consequences | ✅ Full | Good and bad effects of this decision |
| §7 Implementation Guidance | ❌ Skip | For engineers |

**Takeaway after reading:** You should understand why AGOS avoids AI for grammar decisions and why it's built like a compiler instead.

---

### Step 3: The Other Architecture Decisions — ADR-0002 to ADR-0005

These are shorter. Read the **Context, Decision, Alternatives, and Consequences** sections of each (skip Implementation Guidance).

| File | The Key Question | Why It Matters |
|------|------------------|----------------|
| `ADR/ADR-0002` | Why compile grammar into bytecode? | Enables speed, versioning, and plugin-ability |
| `ADR/ADR-0003` | Why have a "middle language" (GIR)? | Lets front-end and back-end evolve independently |
| `ADR/ADR-0004` | Why work fully offline? | 375x cheaper than cloud-only; works anywhere |
| `ADR/ADR-0005` | Why support plugins? | Lets anyone add new grammar schools or features |

**Takeaway:** You should understand the 5 major architectural bets AGOS is making.

---

### Step 4: System Architecture Overview — SPEC-0001 Chapter 2

**File:** `SPEC/SPEC-0001/02-system-architecture-overview.md`

This is a bird's-eye view of the entire platform.

| Section | Read? | Why |
|---------|-------|-----|
| §1 Layered Architecture | ✅ Full | The 3-layer model |
| §2 Module Catalog | ✅ Full | Lists all 14 modules with 1-paragraph descriptions |
| §3 Pipeline Design | ⚡ Skim | Technical flow diagram |
| §4 Design Precepts | ✅ Full | Design rules |
| §5-6 Data/Error | ❌ Skip | For engineers |

**How to read §2 (Module Catalog):** Just read each module's **Purpose** line (1 sentence). Don't read the function signatures or interface details. This gives you the complete map of AGOS without any code.

---

### Step 5: The Linguistic Data — KB Overview

**File:** `KB/KB-OVERVIEW.md`

This is short (~230 lines). Read it all.

Then, if you care about specific types of data, read the **Scope** and **Data Format** sections of these KB files (skip compilation details, binary format specs, and performance budgets):

| File | What It Defines |
|------|-----------------|
| `KB-0001` | ~15,000-20,000 Arabic roots (like س-ل-م) |
| `KB-0002` | ~300-450 morphological patterns (wazan) |
| `KB-0003` | Verb conjugation tables for 15 classes |
| `KB-0004` | ~135-180 noun patterns |
| `KB-0005` | ~120-200 particles (prepositions, conjunctions, etc.) |
| `KB-0006` | ~60-80 pronouns |
| `KB-0007` | The complete feature taxonomy (~19 features) |

---

### Step 6: Grammar DSL (The Rule Language) — RFC-0001

**File:** `RFC/RFC-0001-grammar-dsl.md`

If you're a grammar expert, this is for you. It defines the language used to write grammar rules.

| Section | Read? | Why |
|---------|-------|-----|
| §1 Purpose & Scope | ✅ Full | What this language is for |
| §2 Rule Structure | ✅ Full | How rules are organized |
| §3 Conditions | ⚡ Skim | Read the concepts, skip the formal grammar |
| §4 Actions | ⚡ Skim | Read the concepts, skip the formal grammar |
| §5 Built-in Functions | ✅ Full | What operations are available |
| §7 DSL Example | ✅ Full | See actual rules in action |
| §8 Compilation | ❌ Skip | For engineers |
| §6/9-10 | ❌ Skip | For engineers |

---

### Step 7: Everything Else — What to Skip Entirely

The following documents are **pure engineering blueprints**. Unless you plan to write code, skip them:

| File | What's Inside |
|------|---------------|
| `SPEC-0101` | Morphology engine algorithms, root extraction code |
| `SPEC-0102` | 64-bit bitfield layouts for features |
| `SPEC-0103` | Performance optimization (caching, benchmarking) |
| `SPEC-0201` | Rule engine internals, 7 algorithms |
| `SPEC-0301` | GVM runtime: registers, dispatch, verification |
| `SPEC-0302` | All ~50 opcodes (machine instructions) |
| `SPEC-0303` | How to port GVM to different programming languages |
| `SPEC-0304` | Memory model: 7 regions, 2 stacks, allocator |
| `SPEC-0401` | Knowledge graph query engine |
| `SPEC-0501` | Template rendering, I'rab generation code |
| `SPEC-0601` | Plugin system: WASM sandbox, signing, registry |
| `RFC-0002` | Binary bytecode format (hex dumps, section tables) |
| `RFC-0003` | Virtual machine instruction set |
| `RFC-0004` | Advanced rule DSL for school-specific rules |
| `KB-0008` | Compiled particle module (binary format) |

**One exception:** If you're curious about what a specific component does, open the file and read only the **first page** (intro, purpose, scope). The YAML front matter and §1 of every spec document are written for general understanding.

---

## Quick Reference: What Each Component Does

This is the "elevator pitch" version of every module. Read this instead of the full specs.

| Module | Name | One-Liner |
|--------|------|-----------|
| MOD-01 | UnicodeValidator | Checks that the input text is valid Arabic |
| MOD-02 | Lexer | Splits text into sentences and word boundaries |
| MOD-03 | Tokenizer | Identifies particles, pronouns, and potential word stems |
| MOD-04 | MorphologicalParser | Finds the root, pattern, and features of each word |
| MOD-05 | SyntaxParser | Figures out each word's role in the sentence (subject, object, etc.) |
| MOD-06 | GIRConstructor | Packages the analysis into a standardized format (GIR) |
| MOD-07 | RuleEngine | Applies grammar rules (e.g., "subject is nominative") |
| MOD-08 | KnowledgeGraphResolver | Looks up cross-references between linguistic entities |
| MOD-09 | BytecodeGenerator | Compiles the analysis into compact binary instructions |
| MOD-10 | GVM | Executes the binary instructions to produce grammatical results |
| MOD-11 | ExplanationEngine | Turns results into human-readable explanations (Arabic/English/HTML) |
| MOD-12 | PluginLoader | Loads third-party plugins (new grammar schools, features) |
| MOD-13 | CacheManager | Speeds things up by remembering previous results |
| MOD-14 | APIGateway | Provides a web API for other applications to use AGOS |

---

## Glossary of Technical Terms (Translated)

| Spec Term | Plain English |
|-----------|---------------|
| **Pipeline** | Assembly line — text goes in one end, explanation comes out the other |
| **Module** | A station on the assembly line that does one specific job |
| **IR (Intermediate Representation)** | A snapshot of the analysis at a specific stage, saved in a standard format |
| **GIR** | Grammar IR — the main snapshot after grammatical analysis |
| **Bytecode** | Compact binary instructions that a virtual machine can execute quickly |
| **GVM** | Grammar Virtual Machine — the engine that runs bytecode instructions |
| **Lexer** | Text splitter |
| **Tokenizer** | Word analyzer (is this a particle? a verb? a noun?) |
| **Morphology** | Word structure (root + pattern + features) |
| **Syntax (I'rab)** | Grammatical function in a sentence (رفع، نصب، جر، جزم) |
| **Deterministic** | Same input always gives same output (no randomness) |
| **Evidence trail** | A step-by-step record of every rule that was applied |
| **WASM** | WebAssembly — a sandbox for running plugins safely |
| **KB** | Knowledge Base — the dictionary of linguistic data |
| **DSL** | Domain-Specific Language — a custom programming language for grammar rules |
| **Wazan (وزن)** | Morphological pattern (e.g., فَعَلَ, مَفْعُول) |
| **School** | A grammar tradition (Basra, Kufa, Baghdad, Andalus, Modern) |

---

## Suggested Reading Time

| Step | Documents | Time to Read | Depth |
|------|-----------|--------------|-------|
| 1. SPEC-0001-C1 | 1 | 20-30 min | Full |
| 2. ADR-0001 | 1 | 15-20 min | Full |
| 3. ADR-0002 to 0005 | 4 | 10 min each | Context + Decision + Consequences |
| 4. SPEC-0001-C2 | 1 | 20-30 min | Architecture + Module Catalog |
| 5. KB-OVERVIEW + KB-0007 | 2 | 15 min | Full overview; feature concepts |
| 6. RFC-0001 (DSL) | 1 | 20 min (skip tech sections) | If you're a grammar expert |

**Total:** ~2-3 hours to understand the complete platform vision.

After these, you'll understand AGOS well enough to discuss it with engineers, contribute to rule authoring, evaluate the design, or explain it to others.

---

## Final Advice

1. **Don't read cover to cover.** These documents are engineering standards, not a book. Use the table of contents in each file to jump to what matters.

2. **Start with the ADRs.** They explain *why* before *what* or *how*.

3. **Use the Quick Reference table** (above) instead of reading every module spec.

4. **If a section has code, hex dumps, or formal grammar notation, skip it.** It's for implementers.

5. **Ask questions.** If a spec says "we chose X because of Y" and you disagree or don't understand, that's a valuable conversation. The specs are living documents meant to be challenged and improved.
