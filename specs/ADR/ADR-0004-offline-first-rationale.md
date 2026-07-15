---
adr_id: ADR-0004
title: Why AGOS Adopts an Offline-First Architecture
version: 1.0.0
status: Accepted
author: AGOS Architecture Committee
created: 2026-07-15
updated: 2026-07-15
decided: 2026-07-15
references:
  - ADR-0001: Compiler Architecture Rationale
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C6: Deployment & Runtime Considerations
  - SPEC-0001-C9: Performance Targets & Constraints
  - SPEC-0101: Morphology Engine
  - SPEC-0301: Grammar Runtime
  - RFC-0003: Grammar Virtual Machine
  - SPEC-0601: Plugin System
supersedes: None
---

# ADR-0004: Why AGOS Adopts an Offline-First Architecture

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

AGOS is designed as a computational platform for Arabic grammar that can support applications ranging from mobile Quran study tools to enterprise-scale corpus analysis servers. These applications operate in fundamentally different network environments:

- **Mobile applications** (iOS/Android Quran apps, Nahwu tutors) are used in airplanes, remote areas, mosques, and classrooms — places with unreliable or absent network connectivity.
- **Desktop applications** (research tools, pedagogical software) are installed on laptops used in libraries, universities, and homes — often offline or intermittently connected.
- **Server deployments** (enterprise APIs, batch corpus analyzers) run in data centers with reliable connectivity — but the core grammatical analysis itself has no intrinsic need for network access.
- **Educational institutions** in developing regions may have limited or expensive internet access — offline capability is not a convenience but a requirement.

The fundamental question is: **should AGOS require network connectivity for its core grammatical analysis functionality, or should it be designed to function fully offline with optional online features?**

### 1.1 The Linguistic Knowledge Challenge

Arabic grammar analysis requires large linguistic knowledge bases:

| KB | Content | Size (Compact) |
|----|---------|----------------|
| KB-0001 | ~15,000–20,000 roots | ~20 MB |
| KB-0002 | ~300–450 morphological patterns | ~10 MB |
| KB-0003 | ~180–250 conjugation paradigms | ~15 MB |
| KB-0004 | ~135–180 noun patterns | ~10 MB |
| KB-0005 | ~120–200 particle entries | ~2 MB |
| KB-0006 | ~60–80 pronoun entries | ~1 MB |
| KB-0007 | Feature taxonomy (~19 features, ~107 values) | ~1 MB |
| **Total** | | **~59 MB (compact)** |

These are not trivial to stream on demand. A mobile app analyzing a Quranic verse cannot afford to download 59 MB of linguistic data on each use — nor does it make sense to do so when the data changes infrequently (on the order of months or years).

### 1.2 Current Assumptions in Specification

The existing specifications already assume offline-first in several places without explicitly stating it:

- **SPEC-0001-C6 §1.1 (Embedded Library):** "Fully offline" is listed as a property.
- **SPEC-0001-C6 §1.2 (Standalone Server):** "Fully offline (no external dependencies)" is listed as a property.
- **SPEC-0001-C2:** All pipeline stages operate on local data with no network calls.
- **RFC-0003 (GVM):** The Grammar Virtual Machine executes bytecode locally with no external dependencies.
- **SPEC-0601:** Plugins are loaded from local filesystem, only optionally fetched from a registry.

This ADR makes the offline-first commitment **explicit and binding** — it is not an accidental feature but a foundational architectural decision with specific consequences for design, implementation, and deployment.

---

## 2. Problem Statement

### 2.1 The Online-Only Approach

An online-only architecture would require network connectivity for core grammatical analysis. The pipeline would look like:

```diff
+ Online-Only Architecture:
  Input Text → [Network Request] → Cloud Pipeline → Analysis Result

  or:

  Input Text → Local Pipeline → [Network KB Lookup] → Analysis Result
```

In both cases, some essential capability requires network access. This approach fails across several dimensions:

#### 2.1.1 Mobile and Embedded Use Cases

The most impactful AGOS applications are mobile:
- Quranic Arabic learning apps used in mosques and homes
- I'rab analysis tools for students studying in libraries
- Hadith grammar explorers used offline during travel
- Educational apps deployed in regions with limited connectivity

An online-only requirement would render these applications non-functional in the very environments where they are most needed.

#### 2.1.2 Latency Sensitivity

Every network round-trip adds latency:
- Local analysis: 2–100 ms per sentence
- Cloud analysis with API call: 200–2,000 ms per sentence (network + server processing)
- Cloud analysis with multiple API calls: 500–5,000 ms per sentence

For interactive use (tutoring, exploration), sub-100 ms response times are expected. Online-only architectures cannot meet this target under real-world network conditions.

#### 2.1.3 Determinism and Reproducibility

Core Principle 10 (Reproducibility) requires that the same input produces the same output every time. Network services introduce non-determinism:
- Different server versions may process requests differently.
- Load balancers may route to different server instances.
- Caches may be in different states on different requests.
- Network timeouts may cause partial results.

An offline-first architecture eliminates these variables by performing all computation locally with versioned, immutable data.

#### 2.1.4 Cost and Scalability

Every analysis request in an online architecture consumes:
- Server CPU time
- Network bandwidth
- API gateway processing
- Possibly cloud database lookups

For educational use cases with thousands of concurrent users, this cost model is prohibitive. An offline-first architecture shifts computation to the client, making it essentially free to scale.

#### 2.1.5 Privacy and Data Sovereignty

Arabic grammar analysis involves processing text. Users may be analyzing:
- Quranic verses (sacred text, sensitive to modification)
- Personal study notes
- Educational materials
- Research corpora

Sending this data to a cloud service raises privacy concerns and may violate data sovereignty regulations in some jurisdictions. An offline-first architecture processes all data locally, never transmitting the input text.

### 2.2 Degrees of Offline Capability

There is a spectrum of offline capability:

| Level | Name | Description | Network Required? |
|-------|------|-------------|------------------|
| 0 | Online-only | All analysis requires network | Always |
| 1 | KB-streaming | KBs downloaded on demand, cached locally | First use + updates |
| 2 | KB-bundled | KBs bundled with application, no runtime streaming | Updates only |
| 3 | Fully offline | Everything bundled, no network needed ever | Never |

AGOS targets **Level 2** (KB-bundled) as the default, with the ability to reach **Level 3** for air-gapped deployments. Level 1 is an acceptable fallback for plugin distribution (optional extensions).

---

## 3. Decision

**AGOS SHALL adopt an offline-first architecture.** Core grammatical analysis (the full pipeline from Arabic text to AnalysisResult) SHALL function without any network connectivity.

Specifically:

1. **All knowledge bases (KB-0001 through KB-0007) are distributed as files** that are bundled with the application or downloaded once during installation. They are read from local storage (memory-mapped) at runtime. No KB data is fetched on demand during analysis.

2. **All pipeline stages (MOD-01 through MOD-11) execute entirely locally.** No stage makes network calls, queries remote APIs, or depends on external services. The pipeline is fully self-contained.

3. **Rule sets for grammar schools are local files** loaded from the filesystem. School switching does not require network access.

4. **Plugin distribution is optionally online**, but plugin execution is always local. Plugins are downloaded once (via registry) and cached locally. Analysis never requires a live registry.

5. **Caching is local** (in-process LRU or local Redis). There is no dependency on a remote cache service for correct operation.

6. **The Explanation Engine operates with local templates.** LLM-based enhancements (SPEC-0501) are optional and separately configured — the core explanation engine works without them.

7. **Updates are pull-based, not push-based.** KB and plugin updates require explicit user or administrator action. The system never makes background network calls without configuration.

```diff
+ Offline-First Pipeline:
  ┌────────────────────────────────────────────────────────────┐
  │                    Local Execution                           │
  │                                                              │
  │  KB Files (local) ──► Memory-mapped KBs                      │
  │  Rule Files (local) ──► Rule Engine                          │
  │  Plugin Files (local) ──► PluginLoader                       │
  │                                                              │
  │  Input Text ──► MOD-01..11 ──► AnalysisResult                │
  │                                                              │
  │  ═══════ NO NETWORK CALLS ═══════                            │
  └────────────────────────────────────────────────────────────┘

+ Optional Online Features:
  ┌────────────────────────────────────────────────────────────┐
  │  Plugin Registry    ──► Download new plugins (optional)      │
  │  KB Update Server   ──► Download KB updates (optional)       │
  │  LLM Service        ──► Enhanced explanations (optional)     │
  │  Telemetry Server   ──► Usage analytics (optional)           │
  └────────────────────────────────────────────────────────────┘
```

---

## 4. Alternatives Considered

### 4.1 Alternative A: Cloud-Only (Thin Client)

**Description:** All grammatical analysis runs on server infrastructure. The client is a thin shell that sends text to the API and receives results. No local KBs, no local pipeline, no offline capability.

**Advantages:**
- Simplest client — no large KB downloads, no complex local processing.
- Centralized KB updates — all clients always use the latest KB versions.
- Centralized rule updates — all clients always use the latest rule sets.
- No client-side resource requirements (CPU, memory, storage).

**Disadvantages (why rejected):**
- **No offline operation.** This violates the primary use case (mobile educational apps, offline study).
- **Network latency.** Every analysis adds 50–500 ms network overhead, making interactive use sluggish.
- **Server cost.** Every analysis burns server CPU. At scale (thousands of simultaneous educational users), costs are prohibitive.
- **Privacy.** User text is transmitted to and processed on servers.
- **No deterministic reproducibility.** Server-side changes can change analysis results without user knowledge.
- **Single point of failure.** If the server is down, the entire platform is non-functional.
- **Vendor lock-in.** Users cannot switch analysis providers without changing clients.

**Verdict: REJECTED.** The cloud-only approach fails on every non-functional requirement that matters for AGOS's target audience: offline capability, latency, privacy, determinism, and cost.

### 4.2 Alternative B: Hybrid Client-Cloud (Online Preferred, Offline Degraded)

**Description:** The primary analysis path is cloud-based for maximum accuracy and up-to-date KBs. A degraded offline mode uses a smaller, bundled KB subset for basic analysis when no network is available. The offline mode has reduced accuracy and feature coverage.

**Advantages:**
- Best of both worlds — cloud accuracy when online, basic functionality when offline.
- KB updates are automatic when online.
- Smaller offline KB bundle (~10 MB vs. ~59 MB full).

**Disadvantages (why rejected):**
- **Two code paths.** The online and offline analysis paths use different KBs, different algorithms, and potentially produce different results for the same input. This creates a testing and validation nightmare — two systems to maintain, two sets of bugs to fix.
- **Inconsistent results.** A user analyzing the same text online and offline may get different results. This is unacceptable for an educational platform where consistency is critical.
- **Offline path atrophy.** In practice, the offline path receives less testing and fewer updates, degrading over time until it becomes unusable.
- **Complexity.** The hybrid architecture requires network detection, fallback logic, state synchronization, and conflict resolution — all for a reduced experience.
- **User confusion.** "Why does this word have a different I'rab when I'm offline?" is not a question AGOS should generate.

**Verdict: REJECTED.** The two-code-path problem creates unacceptable inconsistency and maintenance burden. Either the platform works fully offline or it doesn't — half-measures create more problems than they solve.

### 4.3 Alternative C: Online-Only with Aggressive Caching

**Description:** All analysis is cloud-based, but results are aggressively cached on the client. Common phrases (Quranic verses, common MSA sentences) are pre-computed and bundled with the app. Cache hit = instant result. Cache miss = network request.

**Advantages:**
- Fast for common phrases (cache hit).
- Server-side KBs are always up to date.
- Pre-computed cache handles the 80% case.

**Disadvantages (why rejected):**
- **Fails on rare/unseen input.** For novel sentences, poetry, or rare constructions — precisely the cases where users most need grammatical analysis — the cache misses and network is required.
- **Cache management complexity.** Cache invalidation when KBs change requires version tracking, stale entry detection, and background re-computation.
- **Pre-computation is unbounded.** There are infinitely many possible Arabic sentences. Pre-computing all potentially useful analyses is impossible.
- **Still has latency on cache miss.** The first analysis of any rare text is slow (network + cloud processing).
- **Storage overhead.** A large pre-computed cache on mobile devices consumes significant storage (~500 MB for comprehensive coverage).

**Verdict: REJECTED.** Caching is a complementary optimization for an offline-first architecture, not a replacement for it. Using caching as the primary mechanism still fails on rare input and introduces cache management complexity.

### 4.4 Alternative D: Offline-First with Optional Online Enhancement (Chosen)

**Description:** The core pipeline runs entirely locally with bundled KBs and rule sets. Optional online features enhance but never replace local functionality: plugin downloads, KB updates, LLM-powered explanation enhancements, and telemetry.

This is the chosen approach. See [Section 5 — Detailed Rationale](#5-detailed-rationale) for the full justification.

**Disadvantages (addressed):**
- **Larger initial download (~59 MB compact KBs).** Mitigated by tiered downloads (compact KBs first, full KBs on demand) and incremental updates.
- **Slower KB update cycle.** Users must explicitly update KBs rather than getting automatic server-side updates. Mitigated by the low change frequency of Arabic linguistic data (roots, patterns, and particles change on the scale of years, not days).
- **Duplicated storage across devices.** Each device stores its own copy of the KBs. Mitigated by the compact KB format (~59 MB) which is negligible on modern devices.
- **Plugin distribution challenge.** Users must download plugins rather than having them automatically available. Mitigated by the plugin registry's optional background sync.

**Verdict: ACCEPTED.** Offline-first is the only architecture that satisfies the core requirements of determinism, latency, privacy, and offline operation while providing optional online enhancement.

---

## 5. Detailed Rationale

### 5.1 Five Reasons for Offline-First

#### Reason 1: Determinism Requires Local Computation

Core Principle 10 states: "Every decision must be reproducible." This is fundamentally incompatible with network-dependent computation:

```diff
+ Offline-First:
  Same input + same KB version + same rule version = 
    byte-for-byte identical output (always)

- Online-Only:
  Same input + "the server" = 
    potentially different output (server version, cache state, load, etc.)
```

Local computation with versioned, immutable data is the only way to guarantee deterministic, reproducible analysis. Network services introduce variables that cannot be controlled or replicated locally.

#### Reason 2: Target Audience Requires Offline Operation

The primary consumers of AGOS analysis are:

| User Group | Typical Environment | Network Availability |
|------------|-------------------|---------------------|
| Quran student | Mosque, home, library | Intermittent or none |
| Arabic language learner | Classroom, commuting | Variable |
| Researcher | University library, field work | Often limited |
| Mobile app user | Anywhere | Unreliable |
| Educational institution | Developing regions | Expensive or limited |

For these users, "works offline" is not a nice-to-have — it is the primary mode of operation. Requiring network connectivity would exclude AGOS from its most impactful applications.

#### Reason 3: Latency Requirements Favor Local Execution

SPEC-0001-C9 defines interactive latency targets:

| Mode | Target Latency |
|------|----------------|
| Interactive (p50) | < 15 ms per sentence |
| Interactive (p99) | < 100 ms per sentence |
| Morphology-only | < 10 ms per sentence |

These targets are achievable only with local execution:

```diff
Time budget for an interactive analysis:
   Local execution:     2–15 ms  ✅
   + Network round-trip: +50–500 ms  ❌ (even on fast connections)
   + Server processing:  +2–15 ms  ❌ (adds to network time)
   + Serialization:      +0.1–1 ms  ❌
   Total (online):       54–531 ms  ❌ exceeds target
   Total (offline):      2–15 ms    ✅ meets target
```

Even with an optimistic 50 ms network round-trip (local data center), the online approach adds 2–10× latency. For users on mobile networks or in remote areas, the multiplier is 10–100×.

#### Reason 4: Privacy by Design

AGOS processes Arabic text — potentially including:
- Personal study notes and reflections on religious texts
- Private correspondence in Arabic (for MSA analysis)
- Unpublished research corpora
- Educational records

An offline-first architecture ensures that:
- **No text ever leaves the user's device.** Input text is processed locally and only the analysis result is displayed locally.
- **No third party can observe what texts are being analyzed.** There is no server log of analysis requests.
- **No data breach can expose analysis history.** There is no central database of analyses to breach.
- **Compliance is automatic.** No data processing agreements, GDPR assessments, or data residency requirements apply when data never leaves the device.

Privacy is not an add-on or a configuration option — it is a fundamental property of the architecture.

#### Reason 5: Cost-Free Scaling

In an offline-first architecture, each user brings their own compute resources. The cost to serve additional users is effectively zero:

```diff
Cost to serve 1 million daily active users:

+ Online-Only:
  Server instances:  50 × $500/month = $300,000/month
  Bandwidth:         10 TB/month × $0.10/GB = $1,000/month
  KB storage:        $100/month
  Total:             ~$301,000/month

+ Offline-First:
  KB distribution (CDN, one-time): $500/month
  Plugin registry:                  $200/month
  Optional telemetry:               $100/month
  Total:                            ~$800/month
```

> **375× cost reduction.**

For an educational platform targeting institutions in developing regions, this cost difference is the difference between viable and impossible.

### 5.2 What "Offline-First" Does Not Mean

Offline-first does not mean:

- **No updates.** Updates are delivered as downloadable packages (KB bundles, plugin packages, application updates). They are pull-based, not push-based.
- **No network features.** Network features (plugin registry, telemetry, LLM explanations) are supported as optional enhancements. They are never required for core analysis.
- **No cloud deployment.** Cloud/server deployment is supported (SPEC-0001-C6 §1.2, §1.3). In server mode, the server itself is offline (no external dependencies) and clients connect to it over the local network.
- **No collaboration features.** Future collaborative features (shared corpora, shared analyses) can be built as optional network extensions on top of the offline core.

### 5.3 Comparison with Industry Patterns

| Pattern | Example | AGOS Approach |
|---------|---------|---------------|
| **Mobile-first** | Progressive web apps | ✅ KBs are mobile-sized (~59 MB compact) |
| **Local-first** | Local-first databases, CRDTs | ✅ All pipeline state is local |
| **Offline-first** | Offline-first maps, note apps | ✅ Core analysis works without network |
| **Edge computing** | Cloudflare Workers | ❌ Not applicable — AGOS is the edge |
| **Thin client** | Web apps | ❌ Rejected — requires network |
| **Hybrid** | Some apps with offline fallback | ❌ Rejected — two code paths |

AGOS follows the **Local-First** movement principles: data ownership, local execution, network as enhancement. This is a well-established pattern in application architecture.

### 5.4 The "KB Update" Objection

The most common objection to offline-first is: "How do users get updated knowledge bases?"

Response: KBs are linguistic data. Arabic roots, morphological patterns, and grammatical particles change on the scale of decades, not days or months. The KB update cadence is:

| KB | Expected Update Frequency | Typical Trigger |
|----|--------------------------|-----------------|
| KB-0001 (Roots) | 1–2× per year | New root discoveries in classical texts |
| KB-0002 (Wazan) | 1× per year | Pattern refinements from research |
| KB-0003 (Verb Forms) | < 1× per year | Paradigm corrections |
| KB-0004 (Noun Patterns) | < 1× per year | Pattern additions |
| KB-0005 (Particles) | < 1× per year | Particle classification updates |
| KB-0006 (Pronouns) | < 1× per 2 years | Pronoun system is stable |
| KB-0007 (Features) | < 1× per 2 years | Feature taxonomy is foundational |

This is not a web application where data changes daily. KB data is closer to a dictionary than a live dataset. Pull-based updates with explicit user action are entirely appropriate.

---

## 6. Consequences

### 6.1 Positive Consequences

1. **Full offline capability.** All core analysis functions without network. Users in any environment — airplane, remote classroom, library, mosque — can use AGOS for its primary purpose.

2. **Deterministic execution.** Local computation with versioned, immutable KB data guarantees reproducible analysis. The same input + same KB + same rules = identical output, every time, on every device.

3. **Low latency.** Sub-millisecond KB access (memory-mapped) and sub-100 μs pipeline processing. No network overhead.

4. **Privacy guarantee.** No text data ever leaves the device. No server-side processing of user content.

5. **Cost-effective scaling.** Each user brings their own compute. The marginal cost of serving additional users approaches zero.

6. **No single point of failure.** There is no central server whose failure disrupts the platform. AGOS continues working even if all network infrastructure goes down.

7. **Air-gapped deployment support.** Institutions with sensitive data or no internet can deploy AGOS by copying files via USB drive.

8. **Simple operational model.** No server management, no capacity planning, no network monitoring for the core analysis pipeline.

### 6.2 Negative Consequences

1. **Larger initial download (~59 MB compact KBs).** Users must download KBs before first use. Mitigated by tiered downloads (compact KBs ~59 MB for mobile, full KBs ~189 MB for server) and incremental updates (delta patches).

2. **Slower KB update adoption.** Users must explicitly update KBs rather than getting automatic updates. Mitigated by automatic background update checks (user-configurable) and update notifications.

3. **No centralized bug fixing.** If a KB error is discovered, all clients must update individually rather than the fix being applied server-side. Mitigated by the low error rate of curated linguistic data and the version compatibility system.

4. **Duplicated storage.** Each device stores its own copy of KBs. Mitigated by the small size of compact KBs (~59 MB) and the use of memory-mapped files (KB data is shared across processes).

5. **Plugin distribution friction.** Users or administrators must explicitly install plugins rather than having them automatically available. Mitigated by the plugin registry's optional background sync and one-click installation.

6. **No A/B testing of KBs.** Cannot test KB changes on a subset of users before full rollout. Mitigated by the formal KB versioning and release process.

### 6.3 Mitigations Summary

| Negative Consequence | Mitigation | Priority |
|---------------------|------------|----------|
| Large initial download | Tiered KBs (compact/full); incremental deltas | High |
| Slow KB update adoption | Background update checks; update notifications | Medium |
| No centralized bug fixing | Rigorous KB QA process; version compatibility | Medium |
| Duplicated storage | Memory-mapped files (shared across processes) | Low |
| Plugin distribution friction | Background sync; one-click install | Medium |

---

## 7. Implementation Guidance

### 7.1 KB Packaging for Offline Distribution

KBs are packaged as versioned, compressed archives for download:

```bash
# KB package names
agos-kb-compact-v1.0.0.agos-kb     # ~59 MB — mobile, desktop
agos-kb-full-v1.0.0.agos-kb        # ~189 MB — server, research

# Installation
agos kb install agos-kb-compact-v1.0.0.agos-kb

# Verification
agos kb verify

# Incremental update
agos kb update-compact agos-kb-compact-v1.1.0.delta
```

### 7.2 Update Distribution Channels

| Channel | Mechanism | Use Case |
|---------|-----------|----------|
| **Application bundle** | KBs included in app package | Mobile apps, desktop apps (App Store, Play Store) |
| **CDN download** | One-time download on first launch | Desktop apps, CLI tools |
| **Side-loading** | USB/network file copy | Air-gapped deployments |
| **Package manager** | apt, brew, choco | Server deployments |
| **Incremental updates** | Delta patches (binary diff) | All deployments (reduces update size) |

### 7.3 Offline-First Checklist

Every component MUST pass this checklist:

```yaml
offline_first_checklist:
  MOD-01..MOD-11:
    - No network calls in any stage
    - All KB access via memory-mapped files (no streaming)
    - All configuration from local files (not remote config service)
    - All rule sets loaded from local filesystem
    - Error handling assumes network is absent (no retry logic on network errors)

  KB Loading:
    - Fallback to last-known-good KB if update download fails
    - Graceful degradation if specific KB file is corrupt (log + skip)
    - Version compatibility checked at load time (no hash-lookup to remote)

  Plugin System (SPEC-0601):
    - Plugin execution does not require registry access
    - Plugins cached locally after first download
    - Registry sync is optional, configurable, non-blocking

  Explanation Engine (SPEC-0501):
    - Template-based explanations are fully local
    - LLM enhancement is optional and separately configured
    - No explanation capability is lost when offline

  Caching:
    - All caches are local (in-process or local Redis)
    - Cache does not depend on external services
    - Cache is invalidated by local version comparison, not remote checks
```

### 7.4 Example: Mobile App Deployment

```diff
  User installs AGOS-powered Quran app from App Store:
    1. App bundle includes compact KBs (~59 MB)
       → First launch: zero download, zero wait
    2. KB update available (v1.0.0 → v1.1.0):
       → Background download (~5 MB delta) when Wi-Fi available
       → Update applied on next app launch
    3. User opens app in airplane mode:
       → Full grammatical analysis works (all KBs local)
       → All features available except LLM explanations (if configured)
    4. User installs a plugin (e.g., Urdu explanations):
       → Downloaded once, cached locally
       → Works offline thereafter
```

---

## 8. Status

**Accepted.** This decision is binding on all AGOS architecture and implementation work.

This ADR supersedes no prior decision.

This ADR is referenced by:
- SPEC-0001-C6: Deployment & Runtime Considerations (§1, §2)
- SPEC-0001-C9: Performance Targets & Constraints (§5 — memory budgets imply local storage)
- SPEC-0301: Grammar Runtime (§3 — GVM execution is local)
- SPEC-0601: Plugin System (§8 — plugin distribution is optional)

---

## Progress Summary

**ADR-0004: Why Offline-First Architecture**

| Section | Title | Status |
|---------|-------|--------|
| Section 1 | Context | ✓ COMPLETE |
| Section 2 | Problem Statement | ✓ COMPLETE (5 limitations of online-only) |
| Section 3 | Decision | ✓ COMPLETE (7 binding decisions) |
| Section 4 | Alternatives Considered | ✓ COMPLETE (4 alternatives analyzed) |
| Section 5 | Detailed Rationale | ✓ COMPLETE (5 reasons + industry comparison) |
| Section 6 | Consequences | ✓ COMPLETE (8 positive, 6 negative) |
| Section 7 | Implementation Guidance | ✓ COMPLETE |
| Section 8 | Status | ✓ COMPLETE |

**Dependencies:** ADR-0001, SPEC-0001-C6, SPEC-0601.

**Recommended next document:** ADR-0005 — Why Plugin Architecture, the final planned ADR.
