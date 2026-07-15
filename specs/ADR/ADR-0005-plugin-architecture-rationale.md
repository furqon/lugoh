---
adr_id: ADR-0005
title: Why AGOS Adopts a Plugin Architecture
version: 1.0.0
status: Accepted
author: AGOS Architecture Committee
created: 2026-07-15
updated: 2026-07-15
decided: 2026-07-15
references:
  - ADR-0001: Compiler Architecture Rationale
  - ADR-0004: Why Offline-First Architecture
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0601: Plugin System
  - RFC-0001: Grammar DSL
  - RFC-0004: Arabic Grammar Rule DSL
  - SPEC-0101: Morphology Engine
  - SPEC-0501: Explanation Engine
supersedes: None
---

# ADR-0005: Why AGOS Adopts a Plugin Architecture

## Table of Contents

1. [Context](#1-context)
2. [Problem Statement](#2-problem-statement)
3. [Decision](#3-decision)
4. [Alternatives Considered](#4-alternatives-considered)
5. [Detailed Rationale](#5-detailed-rationale)
6. [Consequences](#6-consequences)
7. [Implementation Guidance](#7-implementation-guidance)
8. [Status](#8-status)

---

## 1. Context

AGOS is designed as a comprehensive computational platform for Arabic grammar. The platform's scope spans:

- **Multiple grammar schools:** Basra, Kufa, Baghdad, Andalus, and Modern — each with mutually incompatible rule sets for the same grammatical constructions.
- **Multiple languages:** Explanations and pedagogical content in Arabic, English, Urdu, Malay, French, and potentially dozens more.
- **Multiple pedagogical styles:** Beginner-friendly simplified explanations, advanced scholarly I'rab breakdowns, gamified learning experiences, and research-grade formal analyses.
- **Multiple deployment contexts:** Mobile apps, desktop tools, server APIs, browser-based WASM modules, and embedded systems.
- **Future expansion:** Dialectal Arabic, historical linguistic analysis, comparative Semitic grammar, and entirely unforeseen use cases.

The central architectural question is: **how can a single platform serve all these diverse and evolving use cases without becoming a monolithic, unmaintainable system?**

### 1.1 The Core vs. Extension Problem

Every platform faces a tension between what is built into the core (and thus guaranteed, supported, and stable) and what is provided by extensions (and thus flexible, diverse, and independently maintained).

A platform that tries to build everything into the core becomes:
- **Brittle:** A change to any part affects all users, even those who don't need it.
- **Slow to evolve:** New features require core releases, testing across all configurations, and coordinated updates.
- **Difficult to specialize:** The core cannot anticipate every use case. Users who need something specific must either build it themselves (forking the platform) or go without.

A platform that puts everything in extensions becomes:
- **Inconsistent:** Different extensions may disagree on fundamental concepts, producing incompatible analyses.
- **Unreliable:** Extension quality varies. A critical grammatical analysis may depend on a poorly maintained plugin.
- **Fragmented:** Users must discover, evaluate, and assemble collections of extensions to get basic functionality.

The challenge is to find the right balance: a **minimal, stable, well-defined core** that provides essential functionality, with a **rich, flexible plugin system** that allows the platform to evolve, diversify, and specialize without destabilizing the core.

### 1.2 The Grammar School Challenge

Arabic grammar is not a single, unified system. Five major schools (Basra, Kufa, Baghdad, Andalus, Modern) have coexisted for over a millennium, each with differing analyses of the same constructions:

| Rule | Basra | Kufa | Impact |
|------|-------|------|--------|
| Mubtada' must be definite? | Yes | No | Changes sentence classification |
| Khabar may precede mubtada'? | No | Yes | Changes parse tree structure |
| Idafa strictness | Strict | Relaxed | Changes constituent analysis |
| Verb agreement with non-human plural | Feminine singular | Flexible | Changes feature assignment |

These differences are not implementation details — they are fundamental to the grammatical analysis. A platform that cannot support multiple schools with their full, potentially contradictory rule sets cannot claim to be a comprehensive Arabic grammar platform.

A plugin architecture addresses this by making each school's rule set a **plugin** — independently versioned, independently maintained, and independently loaded by the Rule Engine (MOD-07) on demand.

### 1.3 The Language and Audience Challenge

AGOS outputs (explanations, I'rab breakdowns, educational content) must be localized for users speaking different languages and learning at different levels:

- A beginner student needs simplified Arabic terms with English glosses.
- An advanced student needs full Arabic terminology with classical references.
- A researcher needs formal feature notations and KB cross-references.
- A mobile app needs concise, interactive explanations.
- A desktop tool needs detailed, printable I'rab reports.

No single explanation engine can optimally serve all these audiences. A plugin architecture allows multiple explanation plugins to coexist, each targeting a specific language, audience, or format — all operating on the same underlying AnalysisResult.

---

## 2. Problem Statement

### 2.1 The Monolithic Approach

A monolithic architecture would build all features into a single, unified codebase:

```
┌─────────────────────────────────────────────────────────────┐
│                    AGOS Monolithic Core                       │
│                                                               │
│  Pipeline: MOD-01..MOD-11                                     │
│  Schools:  Basra + Kufa + Baghdad + Andalus + Modern          │
│  Languages: ar + en + ur + ms + fr + ...                      │
│  Outputs:  text + json + html + pdf + svg + ...               │
│  Plugins:  None (everything is part of the core)              │
│                                                               │
│  Total:  2 million lines of code?                             │
│  Release cycle: 6 months?                                     │
│  Bug reports:  everywhere                                     │
└─────────────────────────────────────────────────────────────┘
```

This approach fails on multiple dimensions:

#### 2.1.1 Unbounded Core Growth

Every new feature — school, language, output format, pedagogical style — adds code to the core. The core grows without bound, becoming increasingly difficult to maintain, test, and reason about.

#### 2.1.2 Coupled Release Cycles

All features share the same release cycle. A bug fix in the Urdu explanation templates cannot be released independently of changes to the root extraction algorithm. This means:

- Urdu fixes are delayed until the next morphology release.
- Morphology fixes are delayed by Urdu template reviews.
- Everything moves at the speed of the slowest component.

#### 2.1.3 One-Size-Fits-None Design

Design decisions made for one use case constrain others. For example:
- The explanation engine designed for mobile apps (concise, minimal) cannot serve research needs (detailed, referenced).
- The rule engine optimized for Basra school (strict) cannot efficiently handle Kufa school (flexible).
- The output formatter designed for JSON cannot produce beautiful SVG I'rab trees.

Tradeoffs that are optimal for one audience are suboptimal for another. A monolithic core forces everyone to accept the same tradeoffs.

#### 2.1.4 Community Contribution Barrier

External contributors — linguists, educators, tool developers — cannot extend AGOS without modifying the core codebase. This means:
- Every contribution requires core team review, even for specialized, niche features.
- Contributors must understand the entire codebase to add a small, isolated feature.
- Experimental features cannot be tried without risking core stability.

This dramatically reduces the potential for community growth and ecosystem development.

#### 2.1.5 Testing Complexity

With N features in the core, the test matrix grows as O(N²) — every feature combination must be tested. For 5 schools × 10 languages × 5 output formats = 250 combinations, the testing burden becomes untenable.

### 2.2 The "Everything Is a Plugin" Extremism

The opposite extreme — putting everything in plugins — also fails:

- **No baseline experience.** A fresh installation with no plugins would do nothing. Users must discover and install plugins before they can analyze any text.
- **Discovery problem.** Users cannot easily find the right combination of plugins for their needs.
- **Quality inconsistency.** Without a curated core, every plugin is equally supported — a community-maintained Basra rule set is treated the same as the officially curated one.
- **Fragmentation.** Two users analyzing the same sentence with different plugin collections may get different results, undermining the platform's determinism guarantee.

---

## 3. Decision

**AGOS SHALL adopt a plugin architecture** with a clearly defined boundary between the **stable, minimal core** and the **extensible plugin layer**.

### 3.1 Core vs. Plugin Boundary

```diff
  ┌──────────────────────────────────────────────────────────────┐
  │                   AGOS CORE (Stable, Minimal)                 │
  │                                                               │
  │  Pipeline Stages:  MOD-01 through MOD-11                      │
  │                   (Fixed order, well-defined interfaces)       │
  │                                                               │
  │  Default Schools:  Basra (primary), Kufa (secondary)          │
  │                   (Always available, officially curated)       │
  │                                                               │
  │  Default Language: English (primary), Arabic (secondary)      │
  │                   (Core explanation templates)                │
  │                                                               │
  │  KB Suite:        KB-0001 through KB-0007                     │
  │                   (Versioned linguistic data, not plugins)     │
  │                                                               │
  │  Core Invariants: Pipeline order, GIR format, Bytecode format │
  │                   Evidence trail format, Determinism guarantee │
  └──────────────────────────────────────────────────────────────┘
                              │
                    Plugin Injection Points
                              │
  ┌──────────────────────────────────────────────────────────────┐
  │                  PLUGIN LAYER (Extensible)                    │
  │                                                               │
  │  Rule Sets:       Andalus, Baghdad, Modern, custom schools    │
  │  Languages:       Urdu, Malay, French, Turkish, etc.          │
  │  Output Formats:  HTML, PDF, SVG, LaTeX, speech, braille      │
  │  KB Extensions:   Custom dictionaries, etymology, loanwords   │
  │  Pedagogical:     Gamification, adaptive learning, quizzes    │
  │  Pipeline Hooks:  Pre/post processors, custom interceptors    │
  │  Telemetry:       Monitoring, analytics, performance tracking │
  └──────────────────────────────────────────────────────────────┘
```

### 3.2 What Goes in the Core

The core includes:

1. **The pipeline architecture itself** (stage ordering, IR contracts). This is the fundamental design established by ADR-0001 and cannot be modified by plugins.

2. **All pipeline stages (MOD-01 through MOD-11).** These are the fixed pipeline components that every analysis uses. Plugin injection points exist within stages, but stages themselves are not replaceable by plugins.

3. **Two default grammar schools (Basra and Kufa).** These are always available, officially curated, and guaranteed to work. They serve as the baseline against which plugin rule sets are compared.

4. **Two default explanation languages (English and Arabic).** Core explanation templates ensure that the platform is immediately usable without installing additional language packs.

5. **All knowledge bases (KB-0001 through KB-0007).** KBs are versioned linguistic data, not plugins. They are bundled with the application (per ADR-0004) and updated through the KB update mechanism.

6. **The PluginLoader itself (MOD-12).** The plugin system is not a plugin — it is a core component that manages plugin lifecycle.

### 3.3 What Goes in Plugins

Everything else that is:
- **School-specific** (beyond Basra and Kufa)
- **Language-specific** (beyond English and Arabic)
- **Format-specific** (beyond JSON and basic text)
- **User-specific** (pedagogical preferences, custom dictionaries)
- **Experimental** (new features not yet ready for core inclusion)

### 3.4 Core Stability Guarantee

The core makes the following guarantee to plugin authors:

> "The core API (plugin interfaces, GIR format, and injection points) will not break without a MAJOR version bump. Within a MAJOR version, existing plugins will continue to work without modification."

This stability enables a thriving plugin ecosystem by giving authors confidence that their plugins will not break with each core release.

### 3.5 Plugin Types (9 Defined)

| Type | What It Extends | Example |
|------|----------------|---------|
| `rule_set` | MOD-07 (Rule Engine) | Andalus school rules |
| `explanation` | MOD-11 (Explanation Engine) | Urdu explanations |
| `kb_resolver` | MOD-04, MOD-08 (KB lookups) | Custom etymology dictionary |
| `pre_processor` | MOD-01..MOD-10 (input) | Quranic Uthmani script normalizer |
| `post_processor` | MOD-10 (output) | Result validator |
| `output_renderer` | Pipeline (output) | SVG I'rab tree renderer |
| `telemetry` | Pipeline (monitoring) | Prometheus metrics exporter |
| `gamification` | Pipeline (education) | Quiz generator |
| `pipeline_interceptor` | Pipeline (request/response) | Rate limiter |

---

## 4. Alternatives Considered

### 4.1 Alternative A: Monolithic Core (Everything In)

**Description:** All features — all schools, all languages, all output formats — are built directly into the core codebase. There is no plugin system. Extending the platform requires modifying the core.

**Advantages:**
- Simplest architecture — no plugin system to design, build, or maintain.
- Single codebase — all features are tested together, released together.
- Consistent user experience — every installation has the same capabilities.
- No plugin compatibility concerns — there are no plugins to break.

**Disadvantages (why rejected):**
- **Unbounded core growth.** Every new school, language, or format adds to the core. The codebase grows linearly with feature count, eventually becoming unmaintainable.
- **Coupled release cycles.** All features share the same release. A small fix to one feature cannot be released independently.
- **Community contribution barrier.** External contributors cannot extend AGOS without going through the core team's PR process. This limits the ecosystem.
- **Testing matrix explosion.** N features × M configurations creates O(N×M) test cases. For N=20 features and M=10 configurations, that's 200 test permutations.
- **One-size-fits-all.** Design decisions optimized for one use case constrain others. The platform cannot simultaneously serve beginners and researchers with the same code.

**Verdict: REJECTED.** The monolithic approach does not scale to the diversity of use cases AGOS must support. It would create an unmaintainable codebase with slow release cycles and limited community contribution.

### 4.2 Alternative B: Fork-Based Extension

**Description:** There is no plugin system. Anyone who wants to extend AGOS must fork the repository and modify the codebase directly. Each fork independently maintains its own modifications.

**Advantages:**
- No plugin system design or maintenance cost.
- Complete flexibility — a fork can modify anything.
- No API stability concerns (forks are independent).

**Disadvantages (why rejected):**
- **No upstream integration.** Forks diverge permanently. Bug fixes in the original are not automatically inherited. Security patches must be manually ported.
- **Fragmented ecosystem.** Every fork is a separate project with its own release process, documentation, and support. Users must choose between incompatible forks.
- **High maintenance burden.** Fork maintainers must continuously merge upstream changes, resolving conflicts with their modifications.
- **No shared investment.** Improvements made in one fork are not available to others unless manually ported. Duplication of effort is the norm.
- **Community fragmentation.** The community is divided across forks, reducing the pool of contributors for any single project.

**Verdict: REJECTED.** Fork-based extension is the worst of both worlds: high maintenance burden for extenders, fragmented ecosystem for users, and no shared investment in the platform.

### 4.3 Alternative C: Configuration-Driven Extension

**Description:** Instead of a plugin system, AGOS supports extensibility through configuration files. Users can enable/disable features, set parameters, and define custom behaviors through YAML/JSON configuration — no code changes or plugins required.

**Advantages:**
- Simple — configuration files are easier to write than plugins.
- Safe — configuration cannot crash the pipeline or corrupt memory.
- Discoverable — all configurable options are documented in one place.
- No plugin system complexity (no sandbox, no lifecycle, no distribution).

**Disadvantages (why rejected):**
- **Limited expressiveness.** Configuration can only toggle or parameterize existing behaviors. It cannot introduce fundamentally new behaviors, algorithms, or data sources.
- **No new schools.** A new grammar school with novel rules cannot be expressed through configuration alone — it requires new rule logic.
- **No custom languages.** A new explanation language requires new template logic, not just parameter values.
- **Configuration explosion.** As more features are added, the configuration file grows. Users end up with 500-line YAML files that are as complex as code.
- **Testing burden.** Every configuration combination must be tested. The test matrix explodes as configuration options multiply.
- **No community distribution.** There is no mechanism to share configuration profiles, version them, or update them independently of the core.

**Verdict: REJECTED.** Configuration is a useful complement to plugins (for setting plugin options) but cannot replace them. The expressiveness gap — configuration can only toggle, not create — makes it insufficient for AGOS's extensibility needs.

### 4.4 Alternative D: Dynamic Linking / Shared Libraries

**Description:** Instead of a WASM-based plugin system, AGOS would load extensions as dynamically linked shared libraries (`.so`, `.dylib`, `.dll`). Extensions are native code loaded into the pipeline's process space.

**Advantages:**
- Maximum performance — native code, no sandbox overhead.
- Full access to system resources.
- Simple interface — function calls across library boundary.
- Established technology (POSIX dlopen, Windows LoadLibrary).

**Disadvantages (why rejected):**
- **No memory safety.** A bug in a shared library can corrupt the pipeline's memory, crash the process, or introduce security vulnerabilities.
- **No execution isolation.** A shared library has full access to the process memory, filesystem, network, and system calls.
- **No resource limits.** A shared library can consume unlimited CPU, memory, or file handles.
- **No graceful failure.** A crashing library takes down the entire pipeline.
- **Binary compatibility.** Shared libraries must match the exact compiler version, ABI, and runtime of the host process. This makes distribution and installation fragile.
- **Platform dependence.** `.so` files do not work on Windows; `.dll` files do not work on Linux; `.dylib` files are macOS-specific. Distributing plugins for all platforms requires building each plugin N times.
- **No cross-language support.** Plugins must be written in the same language (or at least the same C ABI) as the host. This excludes JavaScript, Python, Go, and other potential plugin languages.

**Verdict: REJECTED.** Shared libraries lack the safety, isolation, and cross-platform characteristics that AGOS requires for third-party extensions. Many of these limitations (memory safety, platform dependence, cross-language support) are directly addressed by WASM-based plugins.

### 4.5 Alternative E: Scripting Language (Lua, Python)

**Description:** Embed a scripting language (Lua, Python, JavaScript) as the plugin runtime. Plugin authors write scripts in the embedded language, which is sandboxed by the language runtime.

**Advantages:**
- Safe — scripting languages provide memory safety and bounded execution.
- Easy to learn — many developers already know Python, Lua, or JavaScript.
- No WASM toolchain — standard language tooling suffices.
- Rich standard library — scripting languages include string processing, JSON, HTTP, etc.

**Disadvantages (why rejected):**
- **Performance overhead.** Interpreted scripting languages are 10–100× slower than compiled WASM for computation-heavy tasks (rule matching, tree traversal).
- **Sandbox leakage.** Embedding Python or Lua securely requires extensive sandboxing (restricted imports, function whitelists, execution limits) that is easy to get wrong. WASM provides sandboxing by default.
- **Language lock-in.** Choosing Lua excludes Python developers; choosing Python excludes Lua developers. WASM is language-agnostic — plugins can be written in Rust, C, Go, AssemblyScript, or any language that compiles to WASM.
- **Runtime size.** Embedding a Python interpreter adds ~10–20 MB to the binary. A Lua interpreter adds ~2–3 MB. WASM runtimes are ~1–5 MB.
- **No cross-platform consistency.** Python's behavior differs across versions and implementations (CPython vs. PyPy). WASM execution is deterministic across all compliant runtimes.
- **Memory model mismatch.** Scripting languages use garbage collection, which introduces unpredictable pauses. AGOS requires deterministic execution with bounded memory.

**Verdict: REJECTED.** Scripting languages are a reasonable choice for simpler plugin systems but fail for AGOS on performance, sandbox robustness, and deterministic execution requirements.

### 4.6 Alternative F: Plugin Architecture (Chosen)

**Description:** A WASM-based plugin system with capability-based security, deterministic execution, and language-independent plugin development. The PluginLoader (MOD-12) manages the plugin lifecycle: discovery, validation, loading, sandboxing, execution, and cleanup.

**Advantages:**
- See [Section 5 — Detailed Rationale](#5-detailed-rationale).

**Disadvantages (addressed):**
- **Plugin system complexity.** The PluginLoader, WASM ABI, sandbox, registry, and dependency resolver must be designed, implemented, and maintained. Mitigated by modular design (components can be built incrementally) and existing WASM infrastructure.
- **WASM toolchain requirement.** Plugin authors must set up a WASM compilation toolchain. Mitigated by the Plugin SDK, pre-configured build templates, and the growing availability of WASM tooling.
- **Serialization overhead.** Data must be serialized (MsgPack) and copied across the WASM boundary, adding ~1–10 μs per call. Mitigated by instance pooling and the fact that plugin calls are infrequent compared to core pipeline operations.
- **Limited WASM ecosystem for domain-specific needs.** WASM is general-purpose; AGOS-specific host functions must be implemented from scratch. Mitigated by the well-defined ABI and the Plugin SDK.

**Verdict: ACCEPTED.** The WASM-based plugin architecture is the only approach that simultaneously satisfies all requirements: memory safety, deterministic execution, language independence, sandbox isolation, and cross-platform support.

---

## 5. Detailed Rationale

### 5.1 Six Reasons for the Plugin Architecture

#### Reason 1: Grammar School Diversity Requires Extensibility

The five major schools of Arabic grammar are not variations of a single system — they are fundamentally different analytical frameworks:

```diff
+ Basra School:
  Founder: Abu al-Aswad al-Du'ali
  Approach: Strict analogical reasoning (qiyas)
  Key principles: Mubtada' must be definite, khabar follows mubtada'
  Rule count: ~1,250
  Philosophy: "The rule comes first; exceptions must be justified"

+ Kufa School:
  Founder: Al-Kisa'i
  Approach: Broader analogical reasoning with more acceptance of rare constructions
  Key principles: Indefinite mubtada' allowed, khabar may precede
  Rule count: ~1,100
  Philosophy: "Usage (samā') and analogy (qiyās) are equal partners"

+ Andalus School:
  Founder: Ibn Malik
  Approach: Grammatical synthesis with extensive documentation of usage
  Key principles: Verb-subject order preferred, strict idafa
  Rule count: ~600
  Philosophy: "Document what speakers actually say; derive rules from usage"
```

These schools have coexisted for over 1,000 years. No one has "won" — each is valid within its own framework. A platform that claims to support all schools cannot have a single set of rules compiled into the core. Each school must be a **plugin** — independently authored, tested, and versioned.

With a plugin architecture:
- A linguist can author a new school's rule set without modifying AGOS core.
- A user can switch between schools with a configuration change.
- An institution can deploy a custom school plugin without waiting for an AGOS release.

#### Reason 2: Multiple Audiences Require Specialized Output

The same grammatical analysis serves different audiences differently:

```diff
For the sentence: "السلام عليكم"

+ Beginner Student (English, simplified):
  "Assalamu 'alaikum' means 'Peace be upon you'.
   The word 'السلام' (al-salam) is the subject (mubtada') in nominative case.
   The word 'عليكم' (alaikum) is a prepositional phrase serving as the predicate (khabar)."

+ Advanced Student (Arabic, detailed I'rab):
  "السَّلَامُ: مُبْتَدَأٌ مَرْفُوعٌ بِالضَّمَّةِ الظَّاهِرَةِ.
   عَلَيْكُمْ: شِبْهُ جُمْلَةٍ فِي مَحَلِّ رَفْعٍ خَبَرِ الْمُبْتَدَإِ."

+ Researcher (Feature notation):
  "Token 0 (السلام): POS=NOUN, CASE=NOMINATIVE, STATE=DEFINITE,
   ROLE=MUBTADA, AGREEMENT=[3MS], CONFIDENCE=0.95"

+ Mobile App (JSON):
  "{\"type\":\"jumlah_ismiyyah\",\"mubtada\":{\"text\":\"السلام\",\"case\":\"nominative\"},...}"
```

These are not the same output with different parameters. They are fundamentally different presentations of the same underlying analysis. A plugin architecture allows each presentation to be developed, tested, and maintained independently, all operating on the shared `AnalysisResult`.

#### Reason 3: Community Ecosystem Requires a Stable Extension Point

The most successful platforms (WordPress, VS Code, Jenkins, Kubernetes) have thriving plugin ecosystems. The key to a successful ecosystem is a **stable, well-documented extension point**:

```diff
  Platform                     Extension Point
  ─────────────────────────────────────────────────
  WordPress                    wp_filter, shortcodes
  VS Code                      Extension API (contribution points)
  Jenkins                      Plugin interface (Abstract class)
  Kubernetes                   CRDs + controllers
  AGOS                         Plugin trait + WASM ABI
```

A plugin architecture enables:

- **Third-party innovation.** External developers can create plugins for their specific needs without permission from the core team.
- **Niche specialization.** Plugins for rare use cases (e.g., Quranic orthographic analysis, pre-Islamic poetry meter) can exist even if the core team lacks expertise or interest.
- **Community validation.** Popular plugins can be promoted to curated or eventually core. Experimental plugins can be tried without risk.
- **Commercial ecosystem.** Consulting firms and educational publishers can build commercial plugins on top of the open-source platform.

Without a plugin architecture, all innovation must come from the core team. This limits the platform's growth and relevance.

#### Reason 4: Independent Versioning and Release Cycles

Different parts of AGOS evolve at different speeds:

| Component | Change Frequency | Typical Cycle | Plugin? |
|-----------|-----------------|---------------|---------|
| Core pipeline (MOD-01–11) | Low | 6–12 months | ❌ Core |
| KB-0001 (Roots) | Low | 1–2×/year | ❌ KB data |
| Basra school rules | Medium | 2–4×/year | ❌ Core (default) |
| Kufa school rules | Medium | 2–4×/year | ❌ Core (default) |
| Andalus school rules | Low | 1×/year | ✅ Plugin |
| Urdu explanations | Variable | As needed | ✅ Plugin |
| Gamification engine | High | Monthly | ✅ Plugin |
| Custom dictionary | Ad-hoc | When needed | ✅ Plugin |

A plugin architecture lets each component follow its own release cycle:
- The core releases every 6 months with stable APIs.
- The Andalus school plugin releases once a year.
- The gamification plugin releases monthly.
- Custom dictionaries are updated whenever their maintainers choose.

No component waits for another. No release is delayed by an unrelated feature.

#### Reason 5: Safety and Isolation

Plugins are untrusted code. They may contain bugs, security vulnerabilities, or (in a commercial ecosystem) intentionally malicious code. A plugin architecture must provide strong isolation between plugins and the core:

```diff
+ WASM Plugin Sandbox:
  Plugin A ─► [WASM Instance] ─► Linear Memory (64 MB max)
  Plugin B ─► [WASM Instance] ─► Linear Memory (64 MB max)
  Plugin C ─► [WASM Instance] ─► Linear Memory (64 MB max)
               │
               ├─ No access to host memory
               ├─ No access to other plugin's memory
               ├─ No system calls (files, network, processes)
               ├─ Bounded execution (timeout, instruction count)
               └─ Capability-gated host functions (opt-in)

+ Shared Library (no isolation):
  Plugin A ─► .so ─► Full process memory access ─► Can crash the host
  Plugin B ─► .so ─► Full process memory access ─► Can read Plugin A's data
```

WASM sandboxing provides memory safety, execution isolation, and resource limits **by default**, without requiring special operating system features (seccomp, SELinux) that may not be available on all platforms.

#### Reason 6: Language Independence

AGOS core is implemented in Rust. But plugin authors should not be required to write Rust:

```diff
  Plugin written in:   Can target AGOS WASM ABI?
  ─────────────────────────────────────────────────────
  Rust                  ✅  (primary SDK target)
  C / C++               ✅  (via Emscripten / WASM target)
  Go                    ✅  (via TinyGo)
  AssemblyScript        ✅  (TypeScript-like, WASM-native)
  Python                ✅  (via py2wasm / wasm-pack)
  Zig                   ✅  (native WASM target)
  Kotlin                ✅  (via Kotlin/WASM)
  C# / .NET             ✅  (via Blazor WASM)
```

This language independence is critical for building a diverse plugin ecosystem:
- Linguists can write rule set plugins in a high-level DSL (Rust + SDK macros).
- Educators can write explanation plugins in AssemblyScript (TypeScript-like).
- Researchers can write custom resolvers in Go or C.
- All plugins compile to WASM and run identically in the AGOS sandbox.

| Approach | Languages Supported | Cross-Platform | Safety |
|----------|-------------------|----------------|--------|
| Shared libraries | C ABI compatible only | No (per-OS binary) | None |
| Embedded scripting | One language | Yes | Medium (VM dependent) |
| **WASM plugins** | **Any WASM-capable language** | **Yes (same binary)** | **High (sandboxed)** |

### 5.2 The Core as Platform

The plugin architecture transforms AGOS from an application into a **platform**:

```diff
+ Application:
  "This tool does X. You use it as-is."

+ Platform:
  "This system provides a foundation. You build on it."
```

AGOS is not a grammar analyzer. AGOS is a **platform for building grammar analysis applications**. The plugin architecture is the key enabler of this platform vision.

### 5.3 Comparison with Industry Plugin Systems

| Feature | WordPress | VS Code | Jenkins | Kubernetes | AGOS |
|---------|-----------|---------|---------|------------|------|
| **Extension type** | PHP hooks/filters | JS extensions | Java plugins | CRDs + controllers | WASM plugins |
| **Sandboxing** | None (PHP) | None (JS) | None (Java) | Container | WASM sandbox |
| **Lifecycle** | Hook registration | Extension host | Plugin manager | Controller manager | PluginLoader |
| **Registry** | WordPress.org | VS Code Marketplace | Jenkins Update Center | Helm charts | AGOS Registry |
| **API stability** | Backward compat | Strict semver | Strict semver | API versions | Semver + major version guarantee |
| **Performance impact** | Moderate (PHP) | Low (JS) | Low (Java) | High (container) | Low (WASM) |

AGOS's plugin architecture is most similar to VS Code's extension model: a stable API with well-defined contribution points, a marketplace for distribution, and sandboxed execution (VS Code uses a separate extension host process; AGOS uses WASM instances).

---

## 6. Consequences

### 6.1 Positive Consequences

1. **Grammar school diversity.** Each school is an independently maintainable rule set. New schools can be added without core changes. Users choose which school's analysis to follow.

2. **Language and audience specialization.** Explanation plugins can target specific languages, pedagogical levels, and output formats — all operating on the same core analysis.

3. **Community ecosystem.** External developers can build, publish, and maintain plugins without modifying the AGOS core. This enables niche specializations and third-party innovation.

4. **Independent release cycles.** Plugins follow their own release cycles, decoupled from the core. A bug fix in a plugin does not require a core release, and vice versa.

5. **Stable core footprint.** The core remains focused on the essential pipeline stages, GIR format, and bytecode execution. Features that are not universally needed are deferred to plugins.

6. **Safe third-party code.** WASM sandboxing ensures that plugin bugs or vulnerabilities cannot crash the pipeline, corrupt memory, or access unauthorized resources.

7. **Plugin-as-testbed model.** Experimental features can be developed as plugins, validated in real-world use, and promoted to core when mature. This de-risks core feature development.

8. **Commercial ecosystem.** Consulting firms, educational publishers, and tool developers can build commercial plugins on top of the open-source platform, creating a sustainable economic model.

### 6.2 Negative Consequences

1. **Plugin system complexity.** The PluginLoader, WASM ABI, sandbox, registry, and dependency resolver add significant system complexity — approximately 10–15% of total platform code.

2. **Serialization overhead.** Data crossing the WASM boundary must be serialized (MsgPack) and copied, adding ~1–10 μs per plugin call. For most plugin types (explanation, rule set), this overhead is negligible relative to the processing time.

3. **Plugin discovery friction.** Users must know about and install plugins to access non-default features. Mitigated by bundled default plugins (Basra, Kufa, English, Arabic) that provide a complete out-of-box experience.

4. **API stability burden.** The plugin API must be stable across core releases. Breaking changes require MAJOR version bumps and migration tooling. This constrains core refactoring.

5. **Plugin quality variability.** Community plugins may have inconsistent quality, documentation, or maintenance. Mitigated by the plugin registry's rating and review system, and by the concept of "curated" plugins.

6. **WASM toolchain requirement.** Plugin authors must set up a WASM compilation toolchain. Mitigated by the Plugin SDK, pre-configured build templates, and the `agos plugin new` scaffolding command.

### 6.3 Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Plugin API breaks too often | Strict semver: within MAJOR version, plugins MUST NOT break. Breaking changes require 6-month deprecation notice and migration tools. |
| Low-quality plugins flood the registry | Curated plugin badges, mandatory review for curated status, user ratings and reviews. |
| Plugin conflicts (two plugins competing for the same slot) | Priority system, slot conflict detection at activation time, clear error messages. |
| WASM performance overhead | Instance pooling, careful ABI design (minimal copying), skip serialization for zero-copy data paths. |
| Dependency hell (plugin compatibility matrix) | Dependency resolver with version constraints, automated compatibility testing in CI. |
| Plugin authors abandon their plugins | Last-version-fallback: if a dependency is missing, the resolver uses the last cached version and warns. |

---

## 7. Implementation Guidance

### 7.1 Recommended Implementation Order

The plugin system should be implemented incrementally, with each phase delivering value independently:

**Phase 1: Core PluginLoader (MOD-12)**
- Define the PluginBase trait and plugin lifecycle
- Implement manifest validation
- Implement plugin discovery (filesystem scanning)
- Implement basic plugin loading (native modules, no sandbox)
- **Delivers:** Internal plugin support (e.g., loading default school rule sets from files)

**Phase 2: WASM Sandboxing**
- Implement WASM compilation and instance creation
- Define the host ABI (agos_abi functions)
- Implement capability gating (host functions exposed based on manifest)
- Implement resource quotas (memory, execution time, call counts)
- **Delivers:** Safe third-party plugin execution

**Phase 3: Plugin Types**
- Implement rule_set plugin type for MOD-07
- Implement explanation plugin type for MOD-11
- Implement kb_resolver plugin type for MOD-04/MOD-08
- **Delivers:** Most common plugin use cases

**Phase 4: Plugin Distribution**
- Define .agosplugin package format
- Implement plugin signing (Ed25519)
- Implement local plugin registry (SQLite database)
- **Delivers:** Plugin packaging and installation

**Phase 5: Plugin Registry**
- Implement remote registry protocol (REST API)
- Implement plugin search and discovery
- Implement update checking and auto-update
- **Delivers:** Plugin sharing and discovery

**Phase 6: Advanced Features**
- Hot-swap (update plugins without restart)
- Instance pooling (reuse WASM instances)
- Dependency resolution (version constraints, compatibility)
- Plugin SDK and tooling
- **Delivers:** Production-ready plugin ecosystem

### 7.2 Plugin SDK Priorities

| SDK Target | Priority | Rationale |
|------------|----------|-----------|
| Rust | P0 | Primary implementation language; most plugin authors will use Rust |
| AssemblyScript | P1 | TypeScript familiarity; WASM-native compilation |
| C | P1 | Universal FFI; minimal runtime overhead |
| Go (via TinyGo) | P2 | Growing ecosystem; good WASM support |
| Python (via py2wasm) | P3 | Research community; maturing tooling |

### 7.3 Testing Strategy

| Test Type | Coverage Target | Description |
|-----------|-----------------|-------------|
| Unit tests (PluginLoader) | 95% | Lifecycle, validation, dependency resolution |
| Unit tests (sandbox) | 100% | Capability enforcement, memory limits, timeout |
| Integration tests (lifecycle) | 100% | Full lifecycle: discover → validate → load → init → process → shutdown |
| Integration tests (plugin types) | 90% | Each plugin type with a reference implementation |
| Security tests | 100% | All T-01..T-10 threat scenarios |
| Performance tests | Within §11 targets | Latency, throughput, memory |

### 7.4 Migration Path

For users of earlier AGOS versions without the plugin system:

```diff
  v0.x (pre-plugin):
    - All schools compiled into the core
    - All languages compiled into the core
    - Customization requires forking

  v1.0 (plugin introduction):
    - Core: MOD-01..MOD-11, Basra, Kufa, English, Arabic
    - Plugin API: v1.0 stable
    - Migration: Existing customizations moved to plugins
    - Backward compatible: Existing configurations continue to work

  v1.x (plugin ecosystem growth):
    - Registry launched
    - Community plugins available
    - Core: Stable, minimal changes
    - Plugin API: v1.x (additive only)

  v2.0 (possible plugin API evolution):
    - Breaking changes to plugin API
    - Migration tools provided
    - Old plugins continue on v1.x
```

---

## 8. Status

**Accepted.** This decision is binding on all AGOS architecture and implementation work.

This ADR supersedes no prior decision.

This ADR is referenced by:
- SPEC-0001-C7: Extensibility & Plugin Architecture (the architectural overview)
- SPEC-0601: Plugin System (the implementation deep-dive)
- SPEC-0001-C4: Module Responsibilities & Interfaces (MOD-12 PluginLoader interface)
- RFC-0001: Grammar DSL (rule set plugins use the DSL)
- SPEC-0501: Explanation Engine (explanation plugins)

---

## Progress Summary

**ADR-0005: Why Plugin Architecture**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Context | ✓ COMPLETE |
| Section 2 | Problem Statement | ✓ COMPLETE (2 failure modes) |
| Section 3 | Decision | ✓ COMPLETE (core/plugin boundary, 9 types) |
| Section 4 | Alternatives Considered | ✓ COMPLETE (6 alternatives analyzed) |
| Section 5 | Detailed Rationale | ✓ COMPLETE (6 reasons + platform comparison) |
| Section 6 | Consequences | ✓ COMPLETE (8 positive, 6 negative) |
| Section 7 | Implementation Guidance | ✓ COMPLETE (6 phases, 5 SDK priorities) |
| Section 8 | Status | ✓ COMPLETE |

**Dependencies:** ADR-0001 (Compiler Architecture), ADR-0004 (Offline-First), SPEC-0001-C7 (Plugin Architecture overview), SPEC-0601 (Plugin System implementation).

**All 5 planned ADRs are now complete.** The ADR suite covers the foundational architectural decisions of the AGOS platform: compiler architecture, bytecode, intermediate representation, offline-first, and plugin architecture.
