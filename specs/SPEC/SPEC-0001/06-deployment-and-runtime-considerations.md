---
spec_id: SPEC-0001
chapter: 6
title: Deployment & Runtime Considerations
version: 0.1.0
status: Draft
author: AGOS Architecture Committee
created: 2026-07-13
updated: 2026-07-13
references:
  - SPEC-0001-C2: System Architecture Overview
  - SPEC-0001-C4: Module Responsibilities & Interfaces
  - SPEC-0001-C5: Data Flow & Intermediate Representations
  - SPEC-0301: Grammar Runtime (planned)
  - SPEC-0601: Plugin System (planned)
  - RFC-0003: Grammar Virtual Machine (proposed)
  - ADR-0001: Compiler Architecture Rationale
  - ADR-0004: Why Offline-First (planned)
---

# Chapter 6: Deployment & Runtime Considerations

## Table of Contents

1. [Deployment Topologies](#1-deployment-topologies)
2. [Runtime Environments](#2-runtime-environments)
3. [Packaging & Distribution](#3-packaging--distribution)
4. [Resource Requirements](#4-resource-requirements)
5. [Configuration Management](#5-configuration-management)
6. [Logging & Observability](#6-logging--observability)
7. [Monitoring & Alerting](#7-monitoring--alerting)
8. [Health Checks](#8-health-checks)
9. [Backup & Recovery](#9-backup--recovery)
10. [Upgrade & Migration Strategy](#10-upgrade--migration-strategy)
11. [Operational Runbooks](#11-operational-runbooks)
12. [Security Considerations](#12-security-considerations)
13. [Cross-References](#13-cross-references)

---

## 1. Deployment Topologies

AGOS supports three primary deployment topologies. Each trades off complexity, performance, scalability, and offline capability.

### 1.1 Topology A: Embedded Library (Default)

The entire AGOS pipeline is compiled as a native library and linked directly into the consuming application. All stages run in-process.

```
┌──────────────────────────────────────────┐
│           Consumer Application           │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │         AGOS Native Library        │  │
│  │                                    │  │
│  │  MOD-01  MOD-02  MOD-03  MOD-04   │  │
│  │  MOD-05  MOD-06  MOD-07  MOD-08   │  │
│  │  MOD-09  MOD-10  MOD-11            │  │
│  │                                    │  │
│  │  KB-01..07  (memory-mapped)        │  │
│  │  Rules      (compiled in)          │  │
│  │  Cache      (in-process LRU)       │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

| Property | Specification |
|----------|--------------|
| **Latency** | Lowest (no IPC overhead) |
| **Throughput** | Highest (no serialization between stages) |
| **Offline** | Fully offline |
| **Memory** | Shared with host process |
| **Scalability** | Vertical only |
| **Language support** | C ABI, Rust, C++, Go, Python (via FFI) |
| **Ideal for** | Mobile apps, desktop apps, CLI tools, embedded systems |

**Implementation approach:** The library exposes a minimal C ABI (see Chapter 4, MOD-14). Language-specific bindings (Rust crate, Python wheel, npm package, etc.) wrap this ABI.

### 1.2 Topology B: Standalone Server

AGOS runs as a standalone server process. Applications communicate via the REST/gRPC API.

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Application  │     │  Application  │     │  Application  │
│  (Mobile)     │     │  (Web)        │     │  (CLI)        │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │     Load Balancer       │
              │  (nginx / HAProxy)      │
              └─────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │   AGOS Server Instance  │
              │                         │
              │  ┌───────────────────┐  │
              │  │  API Gateway      │  │
              │  │  (REST / gRPC)    │  │
              │  ├───────────────────┤  │
              │  │  Pipeline         │  │
              │  │  (in-process)     │  │
              │  ├───────────────────┤  │
              │  │  Cache (Redis)    │  │
              │  │  KBs (mmap'd)     │  │
              │  └───────────────────┘  │
              └─────────────────────────┘
```

| Property | Specification |
|----------|--------------|
| **Latency** | Low (local network, < 5 ms overhead) |
| **Throughput** | High (horizontal scaling) |
| **Offline** | Fully offline (no external dependencies) |
| **Memory** | 500 MB–2 GB per instance |
| **Scalability** | Horizontal (behind load balancer) |
| **Ideal for** | Web services, enterprise deployments, batch processing |

### 1.3 Topology C: Distributed Pipeline

Pipeline stages are split across multiple services, each independently deployable and scalable.

```
┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
│  Client   │   │  Client   │   │  Client   │   │  Client   │
└─────┬─────┘   └─────┬─────┘   └─────┬─────┘   └─────┬─────┘
      └───────────────┴───────────────┴───────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │   API Gateway    │
                    └──────────────────┘
                              │
                              ▼
              ┌──────────────────────────────┐
              │    Orchestrator / Router     │
              │  (message queue + work queue)│
              └──────────────────────────────┘
                    │      │      │      │
         ┌──────────┘      │      │      └──────────┐
         ▼                 ▼      ▼                 ▼
  ┌────────────┐   ┌────────────┐   ┌────────────┐
  │  MOD-01-03  │   │  MOD-04-05  │   │  MOD-06-08  │
  │  (Lex/Tok)  │   │  (Morph/    │   │  (GIR/Rule/ │
  │             │   │   Syntax)   │   │   KG)        │
  └─────────────┘   └─────────────┘   └─────────────┘
         │                  │                  │
         └──────────────────┼──────────────────┘
                            ▼
                    ┌──────────────┐
                    │  MOD-09-11   │
                    │  (Bytecode/  │
                    │   GVM/Explain)│
                    └──────────────┘
                            │
                            ▼
                    ┌──────────────┐
                    │  Cache Layer  │
                    │  (Redis/Memcached)│
                    └──────────────┘
```

| Property | Specification |
|----------|--------------|
| **Latency** | Moderate (inter-service serialization + network) |
| **Throughput** | Very high (independent scaling per stage) |
| **Offline** | Partially offline (each service is self-contained) |
| **Memory** | 200 MB–1 GB per service instance |
| **Scalability** | Independent horizontal scaling per stage |
| **Ideal for** | Large-scale SaaS, research clusters, corpus analysis |

### 1.4 Topology Selection Guide

| Use Case | Recommended Topology | Rationale |
|----------|---------------------|-----------|
| Mobile app (iOS/Android) | A: Embedded Library | Offline-first, minimal latency, no server required |
| Desktop app (learning tool) | A: Embedded Library | Self-contained, single-user, offline |
| CLI tool | A: Embedded Library | Instant startup, no server management |
| Web API (small scale) | B: Standalone Server | Easy deployment, good for < 100 req/s |
| Enterprise SaaS | B or C: Standalone or Distributed | Depends on scale requirements |
| Quran corpus analysis | C: Distributed | Large batch jobs, scales independently |
| Research cluster | C: Distributed | Maximum throughput, flexible resource allocation |

---

## 2. Runtime Environments

### 2.1 Supported Platforms

The AGOS core (Compilation Layer + GVM) MUST support the following platforms. The Explanation Engine and API Gateway additional layers MAY have reduced platform support.

| Platform | Architecture | Minimum Support | Recommended for Production |
|----------|-------------|-----------------|---------------------------|
| **Linux** | x86_64 | Tier 1 | ✓ |
| **Linux** | aarch64 | Tier 1 | ✓ |
| **Linux** | armv7 (32-bit) | Tier 3 | — |
| **macOS** | x86_64 | Tier 1 | ✓ |
| **macOS** | arm64 (Apple Silicon) | Tier 1 | ✓ |
| **Windows** | x86_64 | Tier 2 | ✓ |
| **Windows** | aarch64 | Tier 3 | — |
| **WebAssembly** | wasm32 | Tier 2 | ✓ (browser) |
| **Android** | aarch64 | Tier 2 | ✓ |
| **iOS** | aarch64 | Tier 2 | ✓ (via Embedded Library) |

**Tier definitions:**

- **Tier 1:** CI/CD tested on every commit. Official binaries provided. Full support.
- **Tier 2:** CI/CD tested nightly. Community binaries may be provided. Best-effort support.
- **Tier 3:** Community maintained. No official binaries or CI.

### 2.2 Operating System Requirements

#### Linux

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **Kernel** | 4.15+ | 5.10+ |
| **glibc** | 2.27+ | 2.35+ |
| **musl** | 1.2+ | 1.2+ (Alpine support) |
| **Filesystem** | Any | ext4, xfs, btrfs |
| **Page size** | 4 KB | 4 KB or 16 KB |

#### macOS

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **Version** | 11.0 (Big Sur) | 13.0+ (Ventura) |
| **Architecture** | x86_64 or arm64 | arm64 (Apple Silicon) |

#### Windows

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **Version** | Windows 10 20H2 | Windows 11 |
| **Architecture** | x86_64 | x86_64 |
| **VC Runtime** | VC++ 2022 Redistributable | VC++ 2022 Redistributable |

### 2.3 WASM Requirements

For browser-based deployments (AGOS in the browser):

| Requirement | Specification |
|-------------|---------------|
| **WASM runtime** | Any browser supporting WebAssembly MVP+ |
| **Memory** | 64 MB initial, up to 256 MB maximum |
| **Threading** | Optional (SharedArrayBuffer for parallel stages) |
| **Filesystem** | WASI virtual filesystem for KB loading |
| **Supported browsers** | Chrome 57+, Firefox 52+, Safari 11+, Edge 16+ |

---

## 3. Packaging & Distribution

### 3.1 Distribution Formats

| Format | Topology | Contents |
|--------|----------|----------|
| **Native library** (`.so`, `.dylib`, `.dll`) | A: Embedded | Core pipeline (MOD-01–11), no KBs |
| **CLI binary** (static) | A or B: Embedded/Server | Core pipeline + minimal KBs |
| **Server binary** (static) | B: Server | Core pipeline + API Gateway + Cache Manager |
| **Docker image** | B: Server | Server binary + full KBs + rule sets |
| **WASM module** (`.wasm`) | A: Embedded (browser) | Core pipeline (MOD-01–10), compact KBs |
| **Python wheel** (`.whl`) | A: Embedded | Native library + Python bindings |
| **npm package** | A: Embedded | WASM module + JS API bindings |
| **Mobile framework** (AAR, XCFramework) | A: Embedded (mobile) | Native library per platform |

### 3.2 Docker Images

```
agos-server:latest
├── agos-server:latest             # Latest stable release
├── agos-server:1.2.3              # Specific version
├── agos-server:1.2                # Latest patch of 1.2
├── agos-server:1                  # Latest minor of 1
├── agos-server:nightly            # Daily build from main
└── agos-server:beta               # Pre-release build

agos-server-slim:latest
├── agos-server-slim:latest        # Without KBs (download separately)
└── agos-server-slim:1.2.3         # Specific version without KBs

agos-kb:latest
├── agos-kb:latest                 # Full knowledge base bundle
└── agos-kb:1.2.3                  # Specific KB version
```

### 3.3 Image Size Targets

| Image | Size Target | Notes |
|-------|-------------|-------|
| `agos-server` (full) | < 200 MB | Includes KBs (compressed), all rule sets |
| `agos-server-slim` | < 30 MB | No KBs; KBs mounted as separate volume |
| `agos-kb` | < 150 MB | Compressed KB data (roots, wazan, etc.) |
| `agos-cli` (static binary) | < 20 MB | Statically linked, no runtime dependencies |

### 3.4 KB Packaging

Knowledge bases are distributed as versioned, compressed binary bundles:

```
KB format: .agos-kb (compressed archive)
├── manifest.json          # KB version metadata
├── kb-0001.agos           # Roots (compressed trie)
├── kb-0002.agos           # Wazan (compressed index)
├── kb-0003.agos           # Verb forms
├── kb-0004.agos           # Noun patterns
├── kb-0005.agos           # Particles
├── kb-0006.agos           # Pronouns
├── kb-0007.agos           # Features taxonomy
├── rules/                 # Grammar DSL rule sets per school
│   ├── basra_v2.agosrules
│   ├── kufa_v2.agosrules
│   ├── baghdad_v1.agosrules
│   └── andalus_v1.agosrules
└── checksums.sha256       # Integrity checksums
```

---

## 4. Resource Requirements

### 4.1 Minimum Hardware Requirements

| Resource | Embedded Library | Standalone Server | Distributed (per service) |
|----------|-----------------|-------------------|--------------------------|
| **CPU** | 1 core, 1.5 GHz | 2 cores, 2.0 GHz | 1 core, 2.0 GHz |
| **RAM** | 128 MB | 512 MB | 256 MB |
| **Disk (KB + rules)** | 500 MB | 500 MB | 500 MB (shared) |
| **Disk (logs)** | 100 MB | 1 GB | 1 GB |
| **Network** | None (local) | 100 Mbps | 1 Gbps |

### 4.2 Recommended Hardware Requirements

| Resource | Embedded Library | Standalone Server | Distributed (per service) |
|----------|-----------------|-------------------|--------------------------|
| **CPU** | 2 cores, 2.0 GHz | 4 cores, 2.5 GHz | 2 cores, 2.5 GHz |
| **RAM** | 256 MB | 2 GB | 512 MB |
| **Disk (KB + rules)** | 500 MB | 500 MB (SSD) | 500 MB (SSD) |
| **Disk (logs)** | 500 MB | 10 GB (SSD) | 10 GB (SSD) |
| **Network** | None (local) | 1 Gbps | 10 Gbps |

### 4.3 Memory Breakdown (Standalone Server)

| Component | Memory (Minimum) | Memory (Recommended) | Notes |
|-----------|-----------------|---------------------|-------|
| **Core pipeline** | 50 MB | 100 MB | Compiled code + runtime |
| **KB-0001 (Roots)** | 80 MB | 80 MB | Trie, memory-mapped |
| **KB-0002 (Wazan)** | 40 MB | 40 MB | Hash index, memory-mapped |
| **KB-0003–0004** | 60 MB | 60 MB | Regex patterns + paradigms |
| **KB-0005–0007** | 20 MB | 20 MB | Particle/pronoun/feature tables |
| **Rule sets** | 30 MB | 50 MB | Parsed DSL rule sets |
| **Cache** | 0 MB (off) | 200 MB–1 GB | Configurable |
| **Working set** | 100 MB | 300 MB | Temporary allocations during analysis |
| **Total** | 380 MB | 850 MB–1.65 GB | |

### 4.4 Performance Benchmarks (Target)

| Benchmark | Embedded Library | Standalone Server | Distributed |
|-----------|-----------------|-------------------|-------------|
| **Throughput (full pipeline)** | 500 sentences/s (1 core) | 2,000 sentences/s (4 cores) | 10,000 sentences/s (10 nodes) |
| **Throughput (morphology-only)** | 5,000 tokens/s | 20,000 tokens/s | 100,000 tokens/s |
| **Latency (p50, full pipeline)** | < 10 ms | < 15 ms (incl. network) | < 50 ms (incl. network) |
| **Latency (p99, full pipeline)** | < 50 ms | < 100 ms | < 200 ms |
| **Cache hit latency** | < 1 μs | < 1 ms (Redis) | < 2 ms (Redis, remote) |
| **Batch throughput** | 1,000 sentences/s | 10,000 sentences/s | 100,000 sentences/s |

---

## 5. Configuration Management

### 5.1 Configuration Sources

Configuration is resolved in the following order (highest priority last):

```
1. Built-in defaults (compiled into each module)
2. Configuration file (agos.yaml / agos.json / agos.toml)
3. Environment variables (prefix AGOS_)
4. Command-line arguments
5. API request parameters (per-request overrides)
```

### 5.2 Configuration File Example

```yaml
# /etc/agos/agos.yaml
# AGOS Platform Configuration

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090
  max_request_size: 1048576    # 1 MB
  read_timeout: 30             # seconds
  write_timeout: 60
  shutdown_timeout: 10

pipeline:
  default_school: "basra"
  default_mode: "full"         # full | morphology-only | tokenization-only
  max_sentence_length: 200
  max_parse_trees: 8
  max_morphological_analyses: 32
  enable_guess: false
  strict_mode: false

knowledge:
  base_path: "/var/lib/agos/kb"
  auto_update: true
  update_interval_hours: 24
  verify_checksums: true

cache:
  enabled: true
  backend: "memory"            # memory | redis | filesystem
  max_size_mb: 512
  default_ttl_seconds: 3600
  redis:
    url: "redis://localhost:6379"
    prefix: "agos:cache:"

plugins:
  directory: "/var/lib/agos/plugins"
  sandbox_enabled: true
  allowed_types: ["rule_set", "explanation"]

explanation:
  default_language: "en"
  default_format: "json"
  enable_llm: false
  llm:
    provider: "openai"
    model: "gpt-4"
    temperature: 0.3
    max_tokens: 500

logging:
  level: "info"                # debug | info | warn | error
  format: "json"               # json | text
  output: "stdout"             # stdout | stderr | file
  file:
    path: "/var/log/agos/agos.log"
    max_size_mb: 100
    max_files: 10
    compress: true
```

### 5.3 Environment Variables

All configuration options are also available as environment variables:

| Variable | Maps To | Example |
|----------|---------|---------|
| `AGOS_SERVER_PORT` | `server.port` | `8080` |
| `AGOS_PIPELINE_SCHOOL` | `pipeline.default_school` | `basra` |
| `AGOS_KNOWLEDGE_BASE_PATH` | `knowledge.base_path` | `/data/agos/kb` |
| `AGOS_CACHE_BACKEND` | `cache.backend` | `redis` |
| `AGOS_CACHE_REDIS_URL` | `cache.redis.url` | `redis://redis:6379` |
| `AGOS_LOG_LEVEL` | `logging.level` | `debug` |
| `AGOS_EXPLANATION_LANGUAGE` | `explanation.default_language` | `ar` |
| `AGOS_PLUGIN_DIRECTORY` | `plugins.directory` | `/data/agos/plugins` |

### 5.4 Configuration Validation

1. All configuration files MUST be validated against a JSON Schema on load.
2. Invalid configurations MUST produce a structured error with the path to the invalid field and the expected schema.
3. Unknown configuration keys MUST produce a warning (not an error), to support forward compatibility.
4. Configuration can be hot-reloaded by sending SIGHUP or calling a reload endpoint.

---

## 6. Logging & Observability

### 6.1 Log Levels

| Level | Usage | Example |
|-------|-------|---------|
| `ERROR` | Fatal or unrecoverable errors | KB load failure, pipeline crash |
| `WARN` | Recoverable issues, degraded state | Missing optional KB, partial parse |
| `INFO` | Normal operational events | Server started, KB loaded, request completed |
| `DEBUG` | Detailed diagnostic information | Rule application trace, cache hit/miss |
| `TRACE` | Per-stage input/output dumps | Full GIR JSON, evidence trail entries |

### 6.2 Structured Logging

All logs MUST be structured JSON. Example:

```json
{
    "timestamp": "2026-07-13T15:04:23.123Z",
    "level": "INFO",
    "logger": "agos.pipeline.rule_engine",
    "message": "Rule applied",
    "request_id": "req-abc123",
    "module": "MOD-07",
    "rule_id": "basra-0103",
    "action": "REJECT",
    "token_indices": [2, 3],
    "duration_ms": 0.45,
    "correlation_id": "corr-def456"
}
```

### 6.3 Request ID Tracing

Every analysis request is assigned a unique `request_id` that propagates through all log entries:

```
Request ID: req-abc123
    │
    ├── MOD-01 (duration: 12μs)
    ├── MOD-02 (duration: 5μs)
    ├── MOD-03 (duration: 18μs)
    ├── MOD-04 (duration: 450μs)
    ├── MOD-05 (duration: 1200μs)
    ├── MOD-06 (duration: 80μs)
    ├── MOD-07 (duration: 520μs, 14 rules applied)
    ├── MOD-08 (duration: 95μs, 5 KB lookups)
    ├── MOD-09 (duration: 210μs, bytecode: 2.3KB)
    ├── MOD-10 (duration: 340μs, 1024 steps)
    └── MOD-11 (duration: 1800μs, LLM enhanced)
    └── Total: 4732μs
```

### 6.4 Log Shipping

| Environment | Recommended Approach |
|-------------|---------------------|
| **Embedded Library** | Application manages its own logging; AGOS logs via callback |
| **Standalone Server** | Structured JSON to stdout; Docker log driver ships to aggregator |
| **Distributed** | Structured JSON to stdout + Fluentd/Logstash → Elasticsearch |

---

## 7. Monitoring & Alerting

### 7.1 Key Metrics

All metrics MUST be exposed via a `/metrics` endpoint in Prometheus format:

#### Pipeline Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `agos_requests_total` | Counter | `status`, `school`, `mode` | Total analysis requests |
| `agos_request_duration_ms` | Histogram | `school`, `mode` | Request latency distribution |
| `agos_pipeline_stage_duration_ms` | Histogram | `stage`, `school` | Per-stage latency |
| `agos_tokens_analyzed_total` | Counter | `school` | Total tokens analyzed |
| `agos_sentences_analyzed_total` | Counter | `school` | Total sentences analyzed |
| `agos_ambiguity_ratio` | Gauge | `stage` | Average ambiguity per token/stage |
| `agos_unknown_tokens_total` | Counter | `school` | Tokens that could not be analyzed |

#### Cache Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `agos_cache_hits_total` | Counter | `stage` | Cache hit count |
| `agos_cache_misses_total` | Counter | `stage` | Cache miss count |
| `agos_cache_hit_ratio` | Gauge | `stage` | Cache hit rate (0.0–1.0) |
| `agos_cache_size_bytes` | Gauge | `stage` | Current cache size |
| `agos_cache_evictions_total` | Counter | `stage` | Cache eviction count |

#### System Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `agos_memory_usage_bytes` | Gauge | `region` | Memory usage by component |
| `agos_kb_load_time_ms` | Gauge | `kb_id` | KB load time |
| `agos_kb_version` | Gauge | `kb_id`, `version` | Current KB version (as metric) |
| `agos_uptime_seconds` | Counter | — | Server uptime |

#### Business Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `agos_grammatical_errors_total` | Counter | `code`, `school` | Grammatical errors flagged |
| `agos_grammatical_warnings_total` | Counter | `code`, `school` | Grammatical warnings flagged |
| `agos_top_errors` | TopK | `code` | Most frequent grammatical errors |
| `agos_top_unknown_stems` | TopK | `stem` | Most common unknown stems |

### 7.2 Alerting Rules

| Alert Name | Condition | Severity | Description |
|------------|-----------|----------|-------------|
| `PipelineLatencyHigh` | p99 latency > 500 ms for 5 minutes | Warning | Pipeline may be overloaded |
| `PipelineLatencyCritical` | p99 latency > 2 s for 5 minutes | Critical | Pipeline is severely degraded |
| `CacheHitRateLow` | Cache hit rate < 50% for 10 minutes | Warning | Cache may need resizing or warming |
| `UnknownTokenRateHigh` | Unknown token rate > 10% for 5 minutes | Warning | KB may be missing common words |
| `ErrorRateHigh` | Error rate > 5% for 5 minutes | Critical | Pipeline is failing on many requests |
| `KBLoadFailure` | KB fails to load | Critical | Platform is non-functional |
| `MemoryHigh` | Memory usage > 90% for 5 minutes | Warning | May need to scale up or reduce cache |
| `DiskSpaceLow` | Disk space < 10% for 5 minutes | Critical | Logs may fill disk |

### 7.3 Health Dashboard (Recommended)

A Grafana dashboard SHOULD include:

1. **Request rate & latency** (rate, p50, p95, p99) over time, broken down by school.
2. **Pipeline stage breakdown** — stacked area chart showing time spent per stage.
3. **Cache performance** — hit ratio, size, evictions over time.
4. **KB version tracker** — which KB versions are loaded and when they were last updated.
5. **Error/warning trends** — top grammatical errors over time.
6. **System resources** — CPU, memory, disk, network.
7. **Top unknown stems** — bar chart of most common unrecognized words.

---

## 8. Health Checks

### 8.1 Health Endpoint

```
GET /v1/health

Response:
{
    "status": "healthy" | "degraded" | "unhealthy",
    "version": "1.2.3",
    "uptime_seconds": 123456,
    "active_requests": 5,
    "kb_loaded": ["KB-0001:1.2.3", "KB-0002:2.0.1", ...],
    "schools_available": ["basra", "kufa", "baghdad"],
    "plugins_loaded": 2,
    "cache_stats": {
        "hit_rate": 0.87,
        "total_entries": 15234,
        "total_size_mb": 234
    },
    "checks": {
        "kb_integrity": "pass",
        "pipeline_ready": "pass",
        "gvm_available": "pass",
        "explanation_languages": ["en", "ar", "ur"]
    }
}
```

### 8.2 Health Check Definitions

| Check | Description | Failure Consequence |
|-------|-------------|---------------------|
| **kb_integrity** | All required KBs are loaded and checksums verified | Pipeline cannot analyze text |
| **pipeline_ready** | All pipeline stages respond to a test input | Partial functionality |
| **gvm_available** | GVM can execute bytecode | Full analysis not possible |
| **explanation_languages** | At least one explanation language is available | Fallback to English |

### 8.3 Startup Probe

On startup, AGOS performs:

1. Load configuration (fail → unhealthy)
2. Load and verify KB checksums (fail → unhealthy)
3. Verify rule set integrity (fail → degraded — rule engine disabled)
4. Run a test sentence through the pipeline (fail → degraded — partial functionality)
5. Start accepting requests (pass → healthy)

---

## 9. Backup & Recovery

### 9.1 What to Back Up

| Component | Backup Required? | Frequency | Retention |
|-----------|-----------------|-----------|-----------|
| Knowledge bases | No (re-downloadable from distribution) | — | — |
| Rule sets | No (re-downloadable from distribution) | — | — |
| Configuration files | Yes | On change | 6 months |
| Cache (Redis) | Optional (rebuilt from live traffic) | Configurable | Configurable |
| Analysis logs | Yes (for audit/compliance) | Daily | 1 year |
| Metrics data (Prometheus) | Yes | Per retention policy | Configurable |

### 9.2 Recovery Procedures

#### KB Corruption Recovery

```bash
# 1. Stop the AGOS server
systemctl stop agos-server

# 2. Remove corrupted KB directory
rm -rf /var/lib/agos/kb

# 3. Re-download KBs
agos-cli kb download --version latest

# 4. Verify checksums
agos-cli kb verify

# 5. Restart AGOS server
systemctl start agos-server
```

#### Configuration Recovery

```bash
# 1. Restore from backup
cp /backup/agos/agos.yaml /etc/agos/agos.yaml

# 2. Reload configuration
kill -HUP $(pgrep agos-server)
# or: curl -X POST http://localhost:8080/v1/admin/reload
```

#### Full Server Recovery

```bash
# 1. Provision new server (bare metal, VM, or container)
# 2. Install AGOS package
apt-get install agos-server=1.2.3

# 3. Restore configuration
cp /backup/agos/agos.yaml /etc/agos/agos.yaml

# 4. Download KBs
agos-cli kb download --version 1.2.3

# 5. Start server
systemctl start agos-server

# 6. Verify health
curl http://localhost:8080/v1/health
```

---

## 10. Upgrade & Migration Strategy

### 10.1 Versioning Scheme

AGOS follows Semantic Versioning 2.0.0:

- **MAJOR:** Breaking changes to the pipeline API, bytecode format, or KB format.
- **MINOR:** New features, new KB entries, new language support (backward-compatible).
- **PATCH:** Bug fixes, performance improvements, documentation.

### 10.2 Upgrade Types

| Type | Version Change | Downtime Required | Migration Steps |
|------|---------------|-------------------|-----------------|
| **Patch** | 1.2.3 → 1.2.4 | No | Rolling restart |
| **Minor** | 1.2.3 → 1.3.0 | No | Rolling restart; optional KB update |
| **Major** | 1.2.3 → 2.0.0 | Yes (or blue/green) | KB migration + config migration |

### 10.3 Blue/Green Deployment (Major Upgrades)

```
Before upgrade:
┌──────────────┐     ┌──────────────┐
│   Blue (v1)  │     │   Green (v1) │
│   Active     │     │   Standby    │
└──────────────┘     └──────────────┘

Step 1: Deploy v2 to Green
┌──────────────┐     ┌──────────────┐
│   Blue (v1)  │     │   Green (v2) │
│   Active     │     │   Standby    │
└──────────────┘     └──────────────┘

Step 2: Migrate Green (KB upgrade, config)
         Smoke test Green

Step 3: Switch traffic to Green
┌──────────────┐     ┌──────────────┐
│   Blue (v1)  │     │   Green (v2) │
│   Standby    │     │   Active     │
└──────────────┘     └──────────────┘
```

### 10.4 KB Migration

KB version independence is guaranteed within a MAJOR version:

| KB Change | Version Impact | Migration |
|-----------|---------------|-----------|
| **Add new root entry** | MINOR (KB-0001 1.2.3 → 1.3.0) | No migration needed |
| **Add new root field** | MINOR | Backward-compatible; old bytecode can be executed |
| **Change root format** | MAJOR | Requires KB conversion tool |
| **Remove root entry** | MAJOR | Requires KB conversion tool |

### 10.5 Bytecode Compatibility

| GVM Version | Can Execute Bytecode Version |
|-------------|------------------------------|
| 1.0.x | 1.0.x |
| 1.1.x | 1.0.x, 1.1.x |
| 2.0.x | 2.0.x (NOT 1.x) |
| 2.1.x | 2.0.x, 2.1.x |

Bytecode is forward-compatible within a MAJOR version only.

---

## 11. Operational Runbooks

### 11.1 Routine Operations

#### Daily

```bash
# Check server health
curl http://localhost:8080/v1/health

# Check cache performance
curl http://localhost:8080/v1/stats/cache

# Check error rate (last 5 minutes)
curl http://localhost:8080/v1/stats/errors?window=5m
```

#### Weekly

```bash
# Check KB version currency
agos-cli kb check-updates

# Rotate logs (if not using log shipper)
logrotate /etc/logrotate.d/agos

# Review top unknown stems
curl http://localhost:8080/v1/stats/unknown-stems?limit=10
```

#### Monthly

```bash
# Update KBs (if auto-update disabled)
agos-cli kb update --version latest

# Run full regression test suite
agos-cli test --suite=regression

# Review performance trends
curl http://localhost:8080/v1/stats/performance?window=30d
```

### 11.2 Incident Response

#### Incident: Pipeline Latency Spike

```bash
# 1. Check current latency
curl http://localhost:8080/v1/stats/latency?window=5m

# 2. Check cache performance
curl http://localhost:8080/v1/stats/cache

# 3. Check request rate
curl http://localhost:8080/v1/stats/requests?window=5m

# 4. If cache miss rate is high:
#    - Warm cache with common phrases
agos-cli cache warm --corpus=common_phrases.txt

#    - Increase cache size
export AGOS_CACHE_MAX_SIZE_MB=1024

#    - Reload configuration
kill -HUP $(pgrep agos-server)
```

#### Incident: KB Load Failure

```bash
# 1. Check KB directory
ls -la /var/lib/agos/kb/

# 2. Verify checksums
agos-cli kb verify

# 3. If corrupted, re-download
agos-cli kb download --version latest

# 4. Restart server
systemctl restart agos-server

# 5. Verify health
curl http://localhost:8080/v1/health
```

#### Incident: Out of Memory

```bash
# 1. Check current memory usage
curl http://localhost:8080/v1/stats/memory

# 2. Reduce cache size
export AGOS_CACHE_MAX_SIZE_MB=256
kill -HUP $(pgrep agos-server)

# 3. If persistent, scale up or add more nodes
# 4. Review memory leak detection
agos-cli debug memory-report
```

### 11.3 Capacity Planning

| Signal | Implication | Action |
|--------|-------------|--------|
| p99 latency > 500 ms | Pipeline overload | Add more server instances |
| Cache hit rate < 60% | Cache too small or cold | Increase cache size; warm cache |
| Memory usage > 85% | Out of memory risk | Reduce cache; scale up |
| CPU usage > 80% sustained | CPU-bound | Scale horizontally |
| Error rate > 2% | Possible bug or KB issue | Investigate error distribution |

---

## 12. Security Considerations

### 12.1 Input Validation

| Threat | Mitigation |
|--------|------------|
| **Malformed UTF-8** | MOD-01 rejects with INVALID_ENCODING |
| **Extremely long input** | MOD-01 rejects with MAX_LENGTH_EXCEEDED (configurable limit) |
| **Injection attacks** | All pipeline stages treat input as text data; no query/command execution |
| **ReDoS (ReDoS)** | All regex-based stages (MorphologicalParser) use ReDoS-safe patterns with timeouts |

### 12.2 Plugin Security

| Threat | Mitigation |
|--------|------------|
| **Plugin with malicious code** | Sandboxed execution (WASM or similar) |
| **Plugin accessing host filesystem** | Restricted to plugin directory only |
| **Plugin making network calls** | Blocked unless explicitly permitted in manifest |
| **Plugin consuming excessive resources** | CPU/memory limits per plugin; watchdog timeout |

### 12.3 Network Security

| Threat | Mitigation |
|--------|------------|
| **Unauthorized API access** | API key authentication (configurable); rate limiting |
| **DDoS** | Rate limiting (per IP, per API key); connection limits |
| **Man-in-the-middle** | TLS 1.3 required for gRPC; optional for REST |
| **Cache poisoning** | Cache keys include input hash; cache entries are read-only |

### 12.4 Dependency Security

| Practice | Description |
|----------|-------------|
| **SBOM generation** | Software Bill of Materials generated with every release |
| **Vulnerability scanning** | All dependencies scanned for CVEs in CI pipeline |
| **Minimal dependencies** | Core pipeline has zero third-party runtime dependencies |
| **Static linking** | CLI and server binaries are statically linked; no system library dependencies |

---

## 13. Cross-References

### 13.1 Internal References

| Reference | Title | Relationship |
|-----------|-------|--------------|
| SPEC-0001-C2 | System Architecture Overview | Module responsibilities that determine deployment boundaries |
| SPEC-0001-C4 | Module Responsibilities & Interfaces | API contracts that must be versioned |
| SPEC-0001-C5 | Data Flow & Intermediate Representations | Caching boundaries and serialization formats |
| SPEC-0301 | Grammar Runtime | GVM execution model and resource requirements |
| SPEC-0601 | Plugin System | Plugin loading and sandboxing |
| RFC-0003 | Grammar Virtual Machine | Bytecode execution and GVM portability |
| ADR-0004 | Why Offline-First | Justifies the offline-first deployment approach |

### 13.2 External References

| Reference | Relevance |
|-----------|-----------|
| Docker Documentation | Container packaging and distribution |
| Prometheus Documentation | Metrics exposition format and best practices |
| Grafana Documentation | Dashboard design for monitoring |
| Twelve-Factor App | Configuration, logging, and process model guidance |
| Open Container Initiative | Container image format standards |
| Semantic Versioning 2.0.0 | Versioning scheme for all AGOS components |

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
| **Chapter 6** | **Deployment & Runtime Considerations** | **✓ COMPLETE (this document)** |
| Chapter 7 | Extensibility & Plugin Architecture | Pending |
| Chapter 8 | Security, Validation & Error Handling | Pending |
| Chapter 9 | Performance Targets & Constraints | Pending |

**Dependencies:** Chapters 1–5, SPEC-0301, SPEC-0601, RFC-0003, ADR-0004.

**Recommended Next Chapter:** Chapter 7 — Extensibility & Plugin Architecture, which will define the plugin system in depth: plugin types, lifecycle, APIs, sandboxing, and the Grammar DSL for rule authors.
