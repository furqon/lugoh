# AGOS Progress

> **Updated:** July 22, 2026 · **Tests:** 113 ✅ · **Web:** ✅

---

## ✅ What's Built

### Core (agos-core)
- PipelineStage trait, PipelineContext, PipelineOrchestrator
- 64-bit FeatureBitfield with builder API (KB-0007 §10)
- 11 IR types (IR-1 through IR-11), all serializable
- PipelineError with 40+ error codes, evidence trail

### Front-End Pipeline (MOD-01 → MOD-05)

```
Input String
  → MOD-01 UnicodeValidator (NFC normalize, validate, strip tashkeel/tatweel)
  → MOD-02 Lexer (tokenize into RawTokens)
  → MOD-03 Tokenizer (segment proclitics + enclitics)
  → MOD-04 MorphologicalParser (roots, wazan, features, POS)
  → MOD-05 SyntaxParser (constituency trees, i'rab roles)
  → SyntaxTree (IR-5)
```

| Stage | What it does | Complexity |
|-------|-------------|-----------|
| **MOD-01** | Validates & normalizes Arabic text | ~300 LoC, 15 tests |
| **MOD-02** | Lexes into token stream | ~350 LoC, 13 tests |
| **MOD-03** | Segments morphemes (proclitics/enclitics) | ~300 LoC |
| **MOD-04** | Roots, wazan, features, POS (~1,900 LoC) | **Core engine** |
| **MOD-05** | Syntax trees, i'rab, sentence type | ~500 LoC |

### Knowledge Base (agos-kb)
- KB types for KB-0001 through KB-0008
- KbLoader / KbReader traits, KbSuite container
- **KB-0004:** Expanded to 405 stem overrides + 29 verb profiles + 55 noun profiles
- Noun/verb profiles fully wired into MOD-04 (Phases D1/D2)
- Knowledge directory: `knowledge/KB-0004/` (JSON seed data)

### Integration Tests
- 17 pipeline tests chaining MOD-01→02→03→04(→05)
- Covers: verbal sentences, nominal sentences, hollow verbs, particles, pronouns,
  definite articles, Andalus school, error cases

### Web API Server (agos-server)
- **`POST /analyze`** — Runs 5-stage pipeline, returns full JSON analysis
- **`GET /health`** — KB-0004 stats + server status
- KB-0004 auto-loaded at startup via `MorphologicalParser::with_kb()`
- CORS enabled, school selection, strip tashkeel/tatweel per-request
- Progressive error handling (partial results on failure)

### Web Frontend (agos-web)
- **Vite 6 + Vue 3.5 + Tailwind CSS 3.4 + TypeScript 5.7**
- RTL Arabic textarea with example buttons, school selector, options
- Dark mode (system preference + toggle)
- Results display: timing accordion → tokens → morphology → syntax tree
- Color-coded POS badges (6 types) and syntactic roles (9 types)
- PWA-ready (manifest, meta tags)
- `npm run build` → 26KB CSS + 91KB JS

---

## 📋 Next Milestones

### Phase A: Complete Front-End Pipeline (Priority: High)

| Step | What | Why |
|------|------|-----|
| **A1** | MOD-06 GIRConstructor | Merge IR-4 + IR-5 into unified GrammarIR (IR-6) |
| **A2** | MOD-07 RuleEngine | Apply Arabic grammar rules (SPEC-0201) — nahw/sarf |
| **A3** | Wire 7-stage pipeline | MOD-01→02→03→04→05→06→07 with integration test |

### Phase B: Knowledge Base Population (Priority: Medium)

| Step | What | Why |
|------|------|-----|
| **B1** | Expand KB-0004 stem overrides → 5,000+ entries | Replace heuristic lists fully |
| **B2** | Build KB-0001 root database | ~15,000 Arabic roots from spec |
| **B3** | Implement KB-0002 wazan matching | Replace check_verb_form() match arms |
| **B4** | Compiled binary format + mmap loading | Production performance targets |

### Phase C: Back-End Pipeline (Priority: High)

| Step | What | Why |
|------|------|-----|
| **C1** | MOD-08 KnowledgeGraphResolver | KB-0001/0002 resolution |
| **C2** | MOD-09 BytecodeGenerator | Serialize GIR → bytecode |
| **C3** | MOD-10 GVM | Execute bytecode → AnalysisResult |

### Phase D: Delivery & Polish (Priority: Medium)

| Step | What | Why |
|------|------|-----|
| **D1** | PWA service worker | Offline caching, installable app |
| **D2** | Request size limiting | Reject texts > 10K chars in server |
| **D3** | MOD-11 ExplanationEngine | Human-readable I'rab output |
| **D4** | CLI executable (`agos`) | Parse from command line |

---

## 🔑 Architecture Decisions

| Decision | Rationale |
|----------|-----------|
| KB-0004 is **authoritative** over heuristic lists | Gradual migration: KB present → KB wins, KB absent → heuristic fallback |
| Pipeline stages are **stateless** | All state in PipelineContext, stages are pure functions |
| `FeatureBitfield` uses **builder pattern** | `.with_gender(x).with_number(y)` — readable and composable |
| KB-0004 **JSON seed data** in `knowledge/KB-0004/` | Development format; production will use `.agos-kb` binary with mmap |
| **Web frontend** decoupled from Rust | Vite dev server proxies `/api` → Axum backend; CORS for production |

### KB-0004 Confidence Semantics

| Context | Noun conf | Verb conf | Winner |
|---------|-----------|-----------|--------|
| Known noun | Masdar 0.25 | Suppressed 0.0 | **Noun** ✅ |
| Known verb | Masdar 0.25 | **Boosted 0.35** | **Verb** ✅ |
| Unknown 3-letter | Masdar 0.25 | Form I 0.30 | **Verb** |
| 4+ letter stem | Participle 0.20 | Form-dep. 0.15–0.25 | **Depends** |

---

## 📊 Test Coverage

| Crate | Tests | Status |
|-------|-------|--------|
| `agos-core` | 4 | ✅ |
| `agos-morph` | 80 (63 unit + 17 integration) | ✅ |
| `agos-syntax` | 18 | ✅ |
| `agos-kb` | 11 | ✅ |
| `agos-server` | — | ✅ (compiles) |
| **Web frontend** | — | ✅ (vue-tsc + vite build) |
| **Total** | **113 + web** | **✅ All passing** |
