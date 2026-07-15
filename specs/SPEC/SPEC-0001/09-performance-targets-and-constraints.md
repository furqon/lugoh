---
spec_id: SPEC-0001
chapter: 9
title: Performance Targets & Constraints
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C3: Compilation Pipeline — Stage-by-Stage
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0001-C6: Deployment & Runtime Considerations
  - SPEC-0001-C7: Extensibility & Plugin Architecture
  - SPEC-0101: Morphology Engine (planned)
  - SPEC-0201: Rule Engine (planned)
  - SPEC-0301: Grammar Runtime
  - SPEC-0401: Knowledge Graph Engine
  - SPEC-0501: Explanation Engine
  - RFC-0003: Grammar Virtual Machine (proposed)
---

# Chapter 9: Performance Targets & Constraints

## Table of Contents

1. [Performance Philosophy](#1-performance-philosophy)
2. [Methodology](#2-methodology)
3. [Per-Stage Latency Targets](#3-per-stage-latency-targets)
4. [End-to-End Throughput Targets](#4-end-to-end-throughput-targets)
5. [Memory Budgets](#5-memory-budgets)
6. [Storage & I/O Targets](#6-storage--io-targets)
7. [Benchmarking Methodology](#7-benchmarking-methodology)
8. [Scalability Model](#8-scalability-model)
9. [Performance Regression Testing](#9-performance-regression-testing)
10. [Sizing Guide](#10-sizing-guide)
11. [SPEC-0001 Completion Summary](#11-spec-0001-completion-summary)
12. [Cross-References](#12-cross-references)

---

## 1. Performance Philosophy

### 1.1 Principles

1. **Determinism is non-negotiable.** Performance optimization MUST NOT introduce non-determinism. Caching is the primary mechanism for performance improvement; every cache is invalidated deterministically based on input + configuration + knowledge version.

2. **Latency targets drive architecture.** The pipeline architecture is designed to meet interactive latency targets (< 100 ms per sentence). Every stage MUST be optimized to meet its individual latency budget.

3. **Throughput scales horizontally.** The distributed pipeline topology (Chapter 6, Section 1.3) enables independent horizontal scaling of each stage. The embedded library topology uses vertical scaling.

4. **Memory is bounded.** All stages MUST operate within defined memory budgets. KBs are memory-mapped rather than loaded into process memory. Temporary allocations during analysis are bounded per-input.

5. **Caching is the first optimization.** Before optimizing a slow stage, evaluate whether caching can eliminate redundant computation. Cache hit rates > 90% are the target for production deployments.

6. **Benchmarks must be reproducible.** All performance benchmarks use a standard corpus (see Section 7) and publish the exact version of the pipeline, KBs, and hardware used.

### 1.2 Performance Classes

AGOS defines three performance classes corresponding to the three deployment topologies:

| Class | Topology | Typical Hardware | Use Case |
|-------|----------|-----------------|----------|
| **Class 1: Interactive** | A: Embedded Library | Mobile/desktop device | Real-time analysis, tutoring |
| **Class 2: Server** | B: Standalone Server | Cloud VM / server | Web API, enterprise |
| **Class 3: Batch** | C: Distributed | Cluster | Corpus analysis, research |

---

## 2. Methodology

### 2.1 Standard Benchmark Corpus

All performance targets are measured against the **AGOS Standard Benchmark Corpus** (version ASBC-1.0):

| Sub-corpus | Source | Sentence Count | Characteristic |
|------------|--------|---------------|----------------|
| **Quranic** | First 500 ayahs of the Quran | 500 sentences | Short, classical, highly structured |
| **Hadith** | 500 selected hadiths | 500 sentences | Medium length, classical |
| **Classical Poetry** | 200 lines of pre-Islamic poetry | 200 sentences | Long, complex syntax, rare vocabulary |
| **MSA News** | 500 sentences from Arabic news | 500 sentences | Modern, medium complexity |
| **MSA Literature** | 300 sentences from modern novels | 300 sentences | Varied length and complexity |
| **Mixed Ambiguity** | 100 sentences with high ambiguity | 100 sentences | Max morphological/syntactic ambiguity |
| **Edge Cases** | 100 edge-case sentences | 100 sentences | Minimum-length, fragments, special constructions |

**Total corpus:** 2,200 sentences, ~25,000 tokens.

### 2.2 Benchmark Hardware Reference

Performance targets are defined relative to this reference hardware:

| Component | Specification |
|-----------|---------------|
| **CPU** | AMD EPYC 7763 (Milan), 64 cores @ 2.45 GHz (single-threaded benchmarks use 1 core) |
| **RAM** | 256 GB DDR4-3200 |
| **Disk** | 1 TB NVMe SSD (Samsung PM9A3) |
| **OS** | Ubuntu 22.04 LTS, Linux kernel 6.2 |
| **Cache** | Redis 7.0 (for Server topology) |
| **Network** | 25 Gbps (for Distributed topology) |

For embedded/mobile targets, the reference is:

| Component | Specification |
|-----------|---------------|
| **CPU** | Apple M2 (4 performance + 4 efficiency cores), or equivalent |
| **RAM** | 8 GB unified memory |
| **Disk** | 256 GB NVMe |
| **OS** | iOS 17 / Android 14 / macOS 14 |

### 2.3 Measurement Methodology

| Metric | Measurement Method | Aggregation |
|--------|-------------------|-------------|
| **Latency (p50)** | Median of 10,000 consecutive requests | Median |
| **Latency (p95)** | 95th percentile of 10,000 consecutive requests | Percentile |
| **Latency (p99)** | 99th percentile of 10,000 consecutive requests | Percentile |
| **Latency (max)** | Maximum observed over 100,000 requests | Maximum |
| **Throughput** | Requests per second over 5-minute steady-state run | Mean |
| **Memory** | Peak RSS during benchmark | Maximum |
| **Cache hit rate** | Cache hits / (cache hits + cache misses) | Ratio |

All benchmarks are performed with:
- Cold cache (first run) and warm cache (after 1,000 similar sentences).
- Single-threaded (unless testing parallelism).
- Reported with ± confidence intervals (95% CI).

---

## 3. Per-Stage Latency Targets

### 3.1 Compilation Pipeline Latency Budget

```
Total Budget: 100 ms (interactive), 10 ms (morphology-only)

                  Budget (ms)    % of Total    Cache Target
Stage             ───────────    ──────────    ────────────
MOD-01 Validate       0.010          0.01%       99.9%
MOD-02 Lex            0.005          0.01%       99.9%
MOD-03 Tokenize       0.020          0.02%       99.5%
MOD-04 Morphology     0.500          0.50%       95.0%
MOD-05 Syntax         5.000          5.00%       90.0%
MOD-06 GIR            0.100          0.10%       99.0%
MOD-07 Rules          10.000        10.00%       85.0%
MOD-08 KG Resolve     0.200          0.20%       95.0%
MOD-09 Bytecode       0.200          0.20%       99.0%
MOD-10 GVM            2.000          2.00%       99.0%
MOD-11 Explanation    80.000        80.00%       90.0%
──────────────────────────────────────────────────────
Total (no cache)      98.035        98.04%        —
Reserve                1.965         1.96%        —
──────────────────────────────────────────────────────
Total Budget          100.000       100.00%
```

### 3.2 Detailed Per-Stage Targets

#### MOD-01: UnicodeValidator

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 1 μs per KB of input | Cold or warm cache |
| Latency (p99) | < 10 μs per KB | Cold |
| Throughput | > 100 MB/s | Single core |
| Cache hit time | < 0.5 μs | Cache hit |

#### MOD-02: Lexer

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 0.5 μs per KB | Cold or warm |
| Latency (p99) | < 2 μs per KB | Cold |
| Throughput | > 200 MB/s | Single core |

#### MOD-03: Tokenizer

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 2 μs per token | Cold, < 16 segmentations |
| Latency (p99) | < 10 μs per token | Cold, max segmentations |
| Throughput | > 500K tokens/s | Single core |

#### MOD-04: MorphologicalParser

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 100 μs per stem | Cold, KBs memory-mapped |
| Latency (p99) | < 1 ms per stem | Cold, max ambiguity |
| Throughput | > 10K stems/s | Single core |
| KB lookup | < 1 μs per lookup | Hash-based index |
| Cache hit latency | < 0.5 μs per stem | Cache hit |

#### MOD-05: SyntaxParser

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 1 ms per sentence | 10-word sentence, 1 parse tree |
| Latency (p99) | < 10 ms per sentence | 30-word sentence, max trees |
| Throughput | > 1K sentences/s | Single core |
| Ambiguity scaling | O(n²) typical, O(n³) worst | n = tokens in sentence |

#### MOD-06: GIRConstructor

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 200 μs per sentence | Cold |
| Latency (p99) | < 500 μs per sentence | Max ambiguity |
| Throughput | > 10K sentences/s | Single core |

#### MOD-07: RuleEngine

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 500 μs per sentence | 10 rules applied |
| Latency (p99) | < 5 ms per sentence | 200 rules applied |
| Throughput | > 5K rule applications/s | Single core |
| Rule load time | < 100 ms per 1,000 rules | At initialization |
| Cache hit latency | < 1 μs per sentence | Cache hit |

#### MOD-08: KnowledgeGraphResolver

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 50 μs per token | KBs memory-mapped |
| Latency (p99) | < 500 μs per token | All KBs queried |
| Throughput | > 10K tokens/s | Single core |
| KB lookup | < 1 μs per lookup | Hash-based index |

#### MOD-09: BytecodeGenerator

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 200 μs per sentence | Optimization level 1 |
| Latency (p99) | < 1 ms per sentence | Optimization level 2 |
| Throughput | > 5K sentences/s | Single core |
| Bytecode size | < 10 KB per sentence | Typical 10-word sentence |
| Compression ratio | < 20% of equivalent JSON GIR | Level 1 optimization |

#### MOD-10: GVM

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 1 ms per sentence | 10-word sentence |
| Latency (p99) | < 5 ms per sentence | 30-word sentence |
| Throughput | > 2K sentences/s | Single core |
| Execution speed | > 100K instructions/s | Interpreted |
| Cache hit latency | < 0.5 μs | Cache hit (bytecode → result) |

#### MOD-11: ExplanationEngine

| Metric | Target | Condition |
|--------|--------|-----------|
| Latency (p50) | < 1 ms per sentence | Template-based, no LLM |
| Latency (p99) | < 5 ms per sentence | Template-based |
| Throughput | > 1K sentences/s | Template-based |
| LLM enhancement | +500–2,000 ms | With LLM (non-deterministic) |

### 3.3 Cache Hit Latency Budget

```
Cache Hit Path:
                                Budget
Stage Cache Hit                ──────────
MOD-01 (text → normalized)      < 1 μs
MOD-02 (text → tokens)          < 1 μs
MOD-03 (tokens → segmented)     < 1 μs
MOD-04 (segmented → morph.)     < 1 μs
MOD-05 (morphology → syntax)    < 1 μs
MOD-06 (m+s → GIR)              < 1 μs
MOD-07 (GIR → annotated)        < 1 μs
MOD-08 (annotated → resolved)   < 1 μs
MOD-09 (resolved → bytecode)    < 1 μs
MOD-10 (bytecode → result)      < 1 μs
MOD-11 (result → explanation)   < 1 μs
──────────────────────────────────────────
Total (all cached):             < 10 μs
```

### 3.4 Degraded Mode Latency

When errors occur and the pipeline degrades, latency targets adjust:

| Degradation Level | Expected Latency (p50) | Expected Latency (p99) |
|-------------------|----------------------|----------------------|
| Level 0 (Full) | < 15 ms | < 100 ms |
| Level 1 (Degraded) | < 10 ms | < 80 ms |
| Level 2 (Limited) | < 5 ms | < 50 ms |
| Level 3 (Minimal) | < 2 ms | < 20 ms |
| Level 4 (Fallback) | < 1 ms | < 10 ms |

---

## 4. End-to-End Throughput Targets

### 4.1 Interactive Throughput (Class 1)

```
Metric                    Target
─────────────────────────────────────────────
Max concurrency           1 user
Peak throughput           500 sentences/s
Sustained throughput      100 sentences/s
Latency (p50)             < 15 ms
Latency (p99)             < 100 ms
Cache warm-up time        < 5 s (first 100 sentences)
```

### 4.2 Server Throughput (Class 2)

```
Metric                    Target (1 core)    Target (4 cores)
────────────────────────────────────────────────────────────────
Max concurrency           10 concurrent       50 concurrent
Peak throughput           2K sentences/s      10K sentences/s
Sustained throughput      500 sentences/s     2K sentences/s
Latency (p50)             < 15 ms             < 20 ms
Latency (p99)             < 100 ms            < 150 ms
Memory per instance       500 MB              2 GB
Cache hit rate target     > 85%               > 90%
```

### 4.3 Batch Throughput (Class 3)

```
Metric                    Target (10 nodes)
────────────────────────────────────────────────
Max concurrency           1,000 concurrent
Peak throughput           100K sentences/s
Sustained throughput      50K sentences/s
Latency (p50)             < 200 ms (incl. network)
Latency (p99)             < 500 ms
Corpus throughput         180M sentences/day
Cache hit rate target     > 95% (repeated patterns)
```

### 4.4 Throughput by Pipeline Mode

| Mode | Class 1 (Embedded) | Class 2 (Server, 4 cores) | Class 3 (Distributed, 10 nodes) |
|------|-------------------|--------------------------|-------------------------------|
| **Full** | 500 sentences/s | 10,000 sentences/s | 100,000 sentences/s |
| **Morphology-only** | 5,000 tokens/s | 100,000 tokens/s | 1,000,000 tokens/s |
| **Tokenization-only** | 50,000 tokens/s | 1,000,000 tokens/s | 10,000,000 tokens/s |
| **Syntax-only** | 2,000 sentences/s | 40,000 sentences/s | 400,000 sentences/s |

---

## 5. Memory Budgets

### 5.1 Static Memory (Loaded at Initialization)

| Component | Class 1 (Embedded) | Class 2 (Server) | Notes |
|-----------|-------------------|------------------|-------|
| **Pipeline code** | 10 MB | 50 MB | Compiled binary |
| **KB-0001 (Roots)** | 20 MB (compact trie) | 80 MB (full trie) | Memory-mapped |
| **KB-0002 (Wazan)** | 10 MB | 40 MB | Hash index |
| **KB-0003 (Verb Forms)** | 15 MB | 30 MB | Paradigm tables |
| **KB-0004 (Noun Patterns)** | 10 MB | 30 MB | Pattern database |
| **KB-0005 (Particles)** | 2 MB | 5 MB | Particle list |
| **KB-0006 (Pronouns)** | 1 MB | 2 MB | Pronoun list |
| **KB-0007 (Features)** | 1 MB | 2 MB | Feature taxonomy |
| **Rule sets** | 10 MB (1 school) | 50 MB (up to 5 schools) | Parsed DSL |
| **Cache (min)** | 0 MB (off by default) | 200 MB | Configurable |
| **Total static** | 79 MB | 489 MB | |

### 5.2 Dynamic Memory (Per-Request)

| Stage | Typical Allocation | Maximum Allocation | Freed When |
|-------|-------------------|-------------------|------------|
| MOD-01 | 2× input size | 2× max input (2 MB) | End of MOD-01 |
| MOD-02 | 1.5× input size | 1.5× max input (1.5 MB) | End of MOD-03 |
| MOD-03 | 2× input size | 4× max input (4 MB) | End of MOD-04 |
| MOD-04 | 10 KB per token | 100 KB per token | End of MOD-05 |
| MOD-05 | 50 KB per sentence | 500 KB per sentence | End of MOD-06 |
| MOD-06 | 20 KB per sentence | 200 KB per sentence | End of MOD-07 |
| MOD-07 | 10 KB per rule appl. | 1 MB per sentence | End of MOD-08 |
| MOD-08 | 1 KB per KB lookup | 50 KB per sentence | End of MOD-09 |
| MOD-09 | 10 KB per sentence | 50 KB per sentence | End of MOD-10 |
| MOD-10 | 10 KB per sentence | 100 KB per sentence | End of request |
| MOD-11 | 5 KB per sentence | 10 KB per sentence | End of request |

**Maximum per-request dynamic memory:** ~6 MB (worst case, all stages combined).

### 5.3 Memory Budget by Deployment

| Deployment | Static | Dynamic (per request) | Peak | Notes |
|------------|--------|----------------------|------|-------|
| **Mobile device** | 60 MB (compact KBs) | 6 MB | 70 MB | Cache disabled |
| **Desktop** | 80 MB | 6 MB | 90 MB | Small cache (64 MB) |
| **Server (1 core)** | 490 MB | 6 MB × 10 concurrent = 60 MB | 600 MB | 200 MB cache |
| **Server (4 cores)** | 490 MB | 6 MB × 50 concurrent = 300 MB | 1 GB | 256 MB cache |
| **Distributed (per node)** | 200 MB (subset of KBs) | 6 MB × 100 concurrent = 600 MB | 1 GB | Cache per node |

### 5.4 Memory Optimization Techniques

| Technique | Impact | Applied To |
|-----------|--------|------------|
| **Memory-mapped KBs** | KBs are not loaded into process memory; pages are demand-paged from disk | All KB files |
| **Compact KB format** | Trie-based root lookup instead of hash table for memory-constrained environments | KB-0001 on mobile |
| **String interning** | Repeated strings (feature names, role names) stored once | MOD-04 through MOD-11 |
| **Arena allocation** | Per-request memory arena freed in bulk at request end | MOD-01 through MOD-06 |
| **Zero-copy parsing** | Token text references original input string without copying | MOD-02, MOD-03 |
| **Lazy KB loading** | Only load the KBs needed for the configured school | MOD-04 configuration |

---

## 6. Storage & I/O Targets

### 6.1 Knowledge Base Storage

| KB | Format | Size (Compact) | Size (Full) | Load Time (NVMe) |
|----|--------|---------------|-------------|-----------------|
| KB-0001 (Roots) | Trie binary | 20 MB | 80 MB | < 100 ms |
| KB-0002 (Wazan) | Hash index | 10 MB | 40 MB | < 50 ms |
| KB-0003 (Verb Forms) | Table binary | 15 MB | 30 MB | < 50 ms |
| KB-0004 (Noun Patterns) | Table binary | 10 MB | 30 MB | < 50 ms |
| KB-0005 (Particles) | Array binary | 2 MB | 5 MB | < 10 ms |
| KB-0006 (Pronouns) | Array binary | 1 MB | 2 MB | < 5 ms |
| KB-0007 (Features) | Array binary | 1 MB | 2 MB | < 5 ms |
| Rule sets | DSL compiled | 10 MB | 50 MB | < 100 ms |

**Total KB storage:** 70 MB (compact) – 240 MB (full).

### 6.2 Log Storage

| Log Type | Daily Volume (Server, 1M req/day) | Retention |
|----------|-----------------------------------|-----------|
| Access logs | 500 MB | 30 days |
| Error logs | 50 MB | 90 days |
| Audit logs | 100 MB | 1 year |
| Evidence trail (optional) | 5 GB (if stored persistently) | 7 days |

### 6.3 Disk I/O Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| KB initial load | < 500 ms | Sequential read of all KB files |
| KB page fault (mmap) | < 1 ms | Random access to any KB entry |
| Cache read (Redis) | < 1 ms | Local network |
| Cache write (Redis) | < 1 ms | Local network |
| Log write (per entry) | < 100 μs | Async buffered write |
| Startup time | < 2 seconds | Full pipeline initialization |

---

## 7. Benchmarking Methodology

### 7.1 Benchmark Categories

```
Benchmark Suite: AGOS-Perf-v1
├── 01-latency/
│   ├── single-sentence          # One sentence, repeated (measure p50/p99)
│   ├── mixed-length             # Varying sentence lengths
│   └── ambiguous                # High-ambiguity sentences
├── 02-throughput/
│   ├── single-threaded          # Throughput on 1 core
│   ├── multi-threaded           # Throughput on N cores (scalability)
│   └── batch                    # Batch processing throughput
├── 03-memory/
│   ├── baseline                 # Memory at idle (after initialization)
│   ├── per-request              # Memory delta per request
│   └── peak                     # Peak memory under load
├── 04-cache/
│   ├── cold-start               # All cache misses
│   ├── warm-start               # All cache hits
│   └── mixed                    # Realistic cache hit ratio
├── 05-scale/
│   ├── linear                   # Throughput vs. cores
│   ├── horizontal               # Throughput vs. nodes (distributed)
│   └── concurrency              # Latency vs. concurrent requests
└── 06-regression/
    ├── perf-compare             # Compare against previous release
    └── memory-compare           # Memory comparison
```

### 7.2 Benchmark Command

```
# Run full benchmark suite
agos benchmark run --suite=full --output=results.json

# Run specific category
agos benchmark run --suite=latency --concurrency=10

# Compare against baseline
agos benchmark compare results.json baseline.json

# Generate report
agos benchmark report results.json --format=html
```

### 7.3 Benchmark Output Format

```json
{
    "suite": "AGOS-Perf-v1",
    "version": "1.0",
    "timestamp": "2026-07-13T15:04:23Z",
    "pipeline_version": "1.2.3",
    "knowledge_versions": {
        "KB-0001": "1.2.3",
        "KB-0002": "2.0.1"
    },
    "hardware": {
        "cpu": "AMD EPYC 7763",
        "cores": 64,
        "ram_gb": 256,
        "disk": "NVMe SSD",
        "os": "Ubuntu 22.04"
    },
    "results": {
        "latency_p50_ms": 12.3,
        "latency_p95_ms": 45.6,
        "latency_p99_ms": 89.1,
        "throughput_req_per_sec": 2345,
        "memory_mb": 512,
        "cache_hit_ratio": 0.87
    }
}
```

### 7.4 Performance Budget CI

In CI/CD, every commit is checked against a performance budget:

```
Performance Budget (CI Gate):
├── p50 latency < 20 ms          (soft limit, warning if exceeded)
├── p99 latency < 150 ms         (soft limit, warning if exceeded)
├── Throughput > 1,000 req/s     (soft limit, warning if exceeded)
├── Memory < 600 MB              (hard limit, FAIL if exceeded)
├── Cache hit ratio > 80%        (soft limit, warning if exceeded)
└── No regression > 10%          (hard limit, FAIL if exceeded)
    compared to previous release
```

---

## 8. Scalability Model

### 8.1 Single-Threaded Scaling (Knowledge Base Size)

As knowledge bases grow (more roots, more patterns), performance scales as follows:

```
KB Size     │ MOD-04 Latency    │ MOD-08 Latency    │ Memory
────────────┼───────────────────┼───────────────────┼────────
10K roots   │ 50 μs per stem    │ 20 μs per token   │ 80 MB
50K roots   │ 100 μs per stem   │ 50 μs per token   │ 240 MB
100K roots  │ 150 μs per stem   │ 80 μs per token   │ 400 MB
500K roots  │ 300 μs per stem   │ 200 μs per token  │ 1.2 GB

Scaling factor: O(log n) for trie-based root lookup
                 O(1) average for hash-based wazan lookup
```

### 8.2 Multi-Core Scaling (Server Topology)

```
Cores       │ Throughput        │ Speedup           │ Efficiency
────────────┼───────────────────┼───────────────────┼────────────
1 core      │ 2,000 req/s       │ 1.0×              │ 100%
2 cores     │ 3,800 req/s       │ 1.9×              │ 95%
4 cores     │ 7,200 req/s       │ 3.6×              │ 90%
8 cores     │ 13,600 req/s      │ 6.8×              │ 85%
16 cores    │ 24,000 req/s      │ 12.0×             │ 75%
32 cores    │ 40,000 req/s      │ 20.0×             │ 63%
64 cores    │ 60,000 req/s      │ 30.0×             │ 47%
```

Scaling is near-linear up to 8 cores, then memory bandwidth and cache contention reduce efficiency.

### 8.3 Horizontal Scaling (Distributed Topology)

```
Nodes       │ Throughput        │ Speedup           │ Efficiency
────────────┼───────────────────┼───────────────────┼────────────
1 node      │ 10,000 req/s      │ 1.0×              │ 100%
2 nodes     │ 19,500 req/s      │ 1.95×             │ 98%
5 nodes     │ 48,000 req/s      │ 4.8×              │ 96%
10 nodes    │ 92,000 req/s      │ 9.2×              │ 92%
20 nodes    │ 170,000 req/s     │ 17.0×             │ 85%
50 nodes    │ 350,000 req/s     │ 35.0×             │ 70%
```

Near-linear scaling up to 10 nodes. Beyond that, load balancer and cache synchronization overhead reduce efficiency.

### 8.4 Concurrency Scaling

```
Concurrent        │ Latency (p50)    │ Latency (p99)    │ Throughput
Requests          │                  │                  │
──────────────────┼──────────────────┼──────────────────┼────────────
1                 │ 12 ms            │ 15 ms            │ 83 req/s
10                │ 15 ms            │ 50 ms            │ 667 req/s
50                │ 30 ms            │ 120 ms           │ 1,667 req/s
100               │ 60 ms            │ 250 ms           │ 1,667 req/s
500               │ 300 ms           │ 1,200 ms         │ 1,667 req/s

(Server topology, 4 cores, warm cache)
Pipeline saturates at ~50 concurrent requests on 4 cores.
```

---

## 9. Performance Regression Testing

### 9.1 Regression Detection

Performance regressions are detected by comparing benchmark results against a baseline:

```
Regression Detection:
├── Compare p50 latency
│   ├── Increase < 5%  → ✅ Pass
│   ├── Increase 5-10% → ⚠️ Warning (review required)
│   └── Increase > 10% → ❌ FAIL (block merge)
├── Compare p99 latency
│   ├── Increase < 10%  → ✅ Pass
│   ├── Increase 10-20% → ⚠️ Warning
│   └── Increase > 20%  → ❌ FAIL
├── Compare memory
│   ├── Increase < 5%   → ✅ Pass
│   ├── Increase 5-10%  → ⚠️ Warning
│   └── Increase > 10%  → ❌ FAIL
└── Compare throughput
    ├── Decrease < 5%   → ✅ Pass
    ├── Decrease 5-10%  → ⚠️ Warning
    └── Decrease > 10%  → ❌ FAIL
```

### 9.2 CI Integration

```yaml
# .github/workflows/perf.yml (conceptual)
name: Performance Regression
on: [push, pull_request]
jobs:
  benchmark:
    runs-on: benchmark-runner
    steps:
      - uses: actions/checkout@v4
      - name: Build AGOS
        run: cargo build --release
      - name: Run benchmarks
        run: agos benchmark run --suite=perf-regression --output=current.json
      - name: Compare against baseline
        run: agos benchmark compare current.json baseline.json
      - name: Publish results
        run: agos benchmark report current.json --format=html --output=report.html
      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: perf-report
          path: report.html
```

### 9.3 Performance Budget Updates

The performance budget is reviewed and updated:
- **Minor release:** Budgets are reviewed; adjustments documented.
- **Major release:** Budgets are revised based on new features and architecture changes.
- **Emergency:** Budgets can be temporarily relaxed for critical security fixes (with documented rationale).

---

## 10. Sizing Guide

### 10.1 How to Choose a Deployment Size

| If you need... | Choose... | Estimated Cost |
|----------------|-----------|---------------|
| Real-time analysis for 1 user | Embedded Library (mobile/desktop) | $0 (existing device) |
| Web API for 100 users/day | Server (1 core, 1 GB RAM) | ~$20/month (cloud) |
| Web API for 1,000 users/day | Server (2 cores, 2 GB RAM) | ~$50/month (cloud) |
| Web API for 10,000 users/day | Server (4 cores, 4 GB RAM) | ~$150/month (cloud) |
| Web API for 100,000 users/day | Server (8 cores, 16 GB RAM + Redis) | ~$500/month (cloud) |
| Batch corpus of 1M sentences | Server (4 cores, 4 GB RAM) | ~1 hour processing |
| Batch corpus of 100M sentences | Distributed (10 nodes, 4 cores each) | ~1 hour processing |
| Research cluster | Distributed (50+ nodes) | Custom infrastructure |

### 10.2 Capacity Planning Formula

```
Required throughput (req/s) = (daily_users × avg_requests_per_user) / 86,400

Example:
  10,000 users × 10 requests/day = 100,000 req/day
  100,000 / 86,400 = 1.16 req/s average
  Peak = 5× average (conservative) = 5.8 req/s
  → Server topology, 1 core, is sufficient (2,000 req/s capacity)

Required instances = peak_throughput / throughput_per_instance
                  = 5.8 / 2000 = 0.003
                  → 1 instance (with headroom)
```

### 10.3 Batch Processing Formula

```
Time to process corpus (hours) = corpus_sentences / (throughput × 3600)

Example:
  Quran corpus: 6,236 ayahs
  6,236 / (2,000 × 3,600) = 0.0009 hours ≈ 3 seconds

  Hadith corpus: 100,000 hadiths (average 3 sentences each = 300,000 sentences)
  300,000 / (10,000 × 3,600) = 0.008 hours ≈ 30 seconds

  Full corpus: 1,000,000 sentences
  1,000,000 / (10,000 × 3,600) = 0.028 hours ≈ 1.7 minutes
```

---

## 11. SPEC-0001 Completion Summary

### 11.1 Completed Chapters

| Chapter | Title | Status | Pages (est.) |
|---------|-------|--------|-------------|
| 1 | Introduction and Scope | ✓ COMPLETE | ~15 |
| 2 | System Architecture Overview | ✓ COMPLETE | ~25 |
| 3 | Compilation Pipeline — Stage-by-Stage | ✓ COMPLETE | ~40 |
| 4 | Module Responsibilities & Interfaces | ✓ COMPLETE | ~30 |
| 5 | Data Flow & Intermediate Representations | ✓ COMPLETE | ~30 |
| 6 | Deployment & Runtime Considerations | ✓ COMPLETE | ~25 |
| 7 | Extensibility & Plugin Architecture | ✓ COMPLETE | ~30 |
| 8 | Security, Validation & Error Handling | ✓ COMPLETE | ~25 |
| **9** | **Performance Targets & Constraints** | **✓ COMPLETE (this document)** | **~20** |

**Total:** 9 chapters, ~240 pages (estimated in printed format).

### 11.2 SPEC-0001 Completion Checklist

```
SPEC-0001: Platform Architecture
├── Sections covered:      9/9 ✓
├── YAML front matter:     Present ✓
├── Version specified:     0.1.0 (Draft) ✓
├── Cross-references:      50+ ✓
├── Terminology defined:   Consistent across all chapters ✓
├── Diagrams:              ASCII box-drawing throughout ✓
├── Examples:              JSON examples throughout ✓
├── Implementation-ready:  Yes (algorithms, schemas, interfaces) ✓
├── Code conventions:      RFC 2119 keywords used ✓
└── Design rationale:      Included in all major decisions ✓
```

### 11.3 Next Steps (Planned Specifications)

With SPEC-0001 complete, the following specifications are ready to be authored:

```
Immediate Next:
├── ADR-0002: Why Grammar Bytecode
├── ADR-0003: Why Grammar IR
├── ADR-0004: Why Offline-First
├── ADR-0005: Why Plugin Architecture
├── RFC-0001: Grammar DSL
└── RFC-0002: Grammar Bytecode Format

Soon After:
├── SPEC-0101: Morphology Engine
├── SPEC-0201: Rule Engine
├── SPEC-0301: Grammar Runtime
├── SPEC-0401: Knowledge Graph Engine
├── SPEC-0501: Explanation Engine
└── SPEC-0601: Plugin System

Knowledge Bases:
├── KB-0001: Roots
├── KB-0002: Wazan (Morphological Patterns)
├── KB-0003: Verb Forms (Awzan al-Fi'l)
├── KB-0004: Noun Patterns
├── KB-0005: Particles (Huruf)
├── KB-0006: Pronouns (Dama'ir)
└── KB-0007: Morphological Features Taxonomy
```

---

## 12. Cross-References

### 12.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | Module boundaries that determine cache boundaries |
| SPEC-0001-C3 | Compilation Pipeline — Stage-by-Stage | Per-stage latency targets derived from algorithms |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | Cache key design and serialization overhead |
| SPEC-0001-C6 | Deployment & Runtime Considerations | Hardware requirements mapped to performance targets |
| SPEC-0001-C7 | Extensibility & Plugin Architecture | Plugin overhead budget |
| SPEC-0001-C8 | Security, Validation & Error Handling | Error handling performance overhead |
| SPEC-0101 | Morphology Engine | Detailed MOD-04 implementation performance |
| SPEC-0201 | Rule Engine | Detailed MOD-07 implementation performance |
| SPEC-0301 | Grammar Runtime | GVM execution performance |
| SPEC-0401 | Knowledge Graph Engine | KB lookup performance |
| RFC-0003 | Grammar Virtual Machine | Bytecode execution model and instruction costs |

### 12.2 External References

| Reference | Relevance |
|-----------|-----------|
| SPEC CPU 2017 | Benchmark methodology inspiration |
| Google SRE Handbook | Service level indicators (SLIs) and service level objectives (SLOs) |
| Prometheus Best Practices | Metrics naming and histogram configuration |
| The Tail at Scale (Dean & Barroso) | Latency distribution and tail latency mitigation |

---

## Progress Summary

**SPEC-0001: Platform Architecture**

| Chapter | Title | Status |
|---------|-------|--------|
| Chapter 1 | Introduction and Scope | ✓ COMPLETE |
| Chapter 2 | System Architecture Overview | ✓ COMPLETE |
| Chapter 3 | Compilation Pipeline — Stage-by-Stage | ✓ COMPLETE |
| Chapter 4 | Module Responsibilities & Interfaces | ✓ COMPLETE |
| Chapter 5 | Data Flow & Intermediate Representations | ✓ COMPLETE |
| Chapter 6 | Deployment & Runtime Considerations | ✓ COMPLETE |
| Chapter 7 | Extensibility & Plugin Architecture | ✓ COMPLETE |
| Chapter 8 | Security, Validation & Error Handling | ✓ COMPLETE |
| **Chapter 9** | **Performance Targets & Constraints** | **✓ COMPLETE (this document)** |

**SPEC-0001 is now COMPLETE.** All 9 chapters have been written.
