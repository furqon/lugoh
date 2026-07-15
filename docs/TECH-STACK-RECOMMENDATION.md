# AGOS Tech Stack Recommendation

## Summary

After analyzing all 39 specification documents (~82K lines), the specs consistently recommend **Rust** as the primary implementation language. Below is the distilled recommendation.

## Core Language: Rust

**Why Rust is the right choice (per specs):**
- SPEC-0101, SPEC-0303, ADR-0002, ADR-0005, SPEC-0601 all explicitly recommend Rust
- Zero-cost abstractions match the GVM's performance targets (sub-microsecond lookups)
- Memory safety without GC — critical for the GVM memory model (7 memory regions, bump allocator)
- `cargo test` / `cargo bench` / `cargo audit` are the referenced toolchain
- First-class WASM support via `wasmtime` for the plugin system (SPEC-0601)
- `mmap`-based KB loading (SPEC-0401, SPEC-0103) is idiomatic in Rust
- Cross-compilation to Tier 1 targets (Linux, macOS, Windows, Android) matches deployment topology A (Embedded Library)

**Secondary languages (for ecosystem ports / GVM reimplementations):**
- C — GVM port, WASM via Emscripten (SPEC-0303 mentions explicitly)
- TypeScript — browser-based AGOS via WASM
- Python — research / prototyping
- Go — CLI tools, cloud-native GVM

## Libraries & Dependencies

| Library | Purpose | Spec Ref |
|---------|---------|----------|
| `wasmtime` | WASM runtime for plugin sandboxing | SPEC-0601 §6 |
| `criterion` | Microbenchmarking (hot path perf) | SPEC-0103 §13 |
| `serde` / `serde_json` | IR serialization (JSON mode) | SPEC-0001-C5 |
| `minicbor` or `prost` | CBOR / Protobuf (production IR) | SPEC-0001-C5, ADR-0003 |
| `rmp-serde` | MessagePack for plugin boundary | SPEC-0601, ADR-0005 |
| `rusqlite` | SQLite for plugin registry | SPEC-0601 |
| `redis` | Remote cache backend | SPEC-0001-C6 |
| `handlebars` | Explanation Engine templates | SPEC-0501 |
| `ed25519-dalek` | Plugin binary signing | SPEC-0601 |
| `sha2` | SHA-256 checksums | SPEC-0601 |
| `regex` (or `regex-lite`) | RE2-compatible regex for DSL | RFC-0001 |
| `leb128` | Variable-length integer encoding | RFC-0002 |

## Data & Storage

| Data | Format | Storage | Lookup Target |
|------|--------|---------|---------------|
| KB-0001 (Roots) | YAML → binary trie | `mmap`-loaded | < 1 µs |
| KB-0002 (Wazan) | YAML → hash index | `mmap`-loaded | < 500 ns |
| KB-0003 (Verb Forms) | YAML → table binary | `mmap`-loaded | < 1 µs |
| KB-0004 (Noun Patterns) | YAML → table binary | `mmap`-loaded | < 2 µs |
| KB-0005 (Particles) | YAML → hash index | `mmap`-loaded | < 500 ns |
| KB-0006 (Pronouns) | YAML → hash index | `mmap`-loaded | < 500 ns |
| KB-0007 (Features) | YAML → feature map | `mmap`-loaded | < 200 ns |
| Grammar Bytecode | Custom `.agos` binary | On-disk / in-memory | N/A |
| Plugin Registry | SQLite | Local DB | N/A |
| Cache | In-memory LRU / Redis | Volatile | N/A |

## Build & CI

| Tool | Purpose |
|------|---------|
| `cargo` | Build, test, benchmark, audit |
| `cargo test` | Unit + integration tests |
| `cargo bench` (criterion) | Performance regression tracking |
| `cargo audit` + `trivy` | Vulnerability scanning per commit |
| GitHub Actions | CI gates for KB compatibility, benchmarks |

## Deployment Targets (per SPEC-0001-C6)

| Topology | Description | Primary Targets |
|----------|-------------|-----------------|
| **A: Embedded Library** | Native library linked into app, fully offline | Linux, macOS, Windows, Android, iOS |
| **B: Standalone Server** | REST API process | Linux (x86_64, aarch64) |
| **C: Distributed Pipeline** | Microservice stages | Linux (Docker/K8s) |

## What to NOT use

- **LLM for grammar decisions** — strictly forbidden (per charter). LLMs are only for explanation, tutoring, summarization.
- **Python for core pipeline** — too slow for the sub-microsecond lookup budgets. Use only for tooling/prototyping.
- **Dynamic languages for GVM** — Rust must be the reference implementation. Other langs can port later.
- **Cloud-only** — must be offline-first (ADR-0004). Redis is optional, never required.

## Recommendation

**Use Rust for the reference implementation.** Start with a single `cargo workspace` with these crates:

```
agos-core/          # Shared types, error types, IR definitions
agos-kb/            # KB compiler (YAML → binary) + runtime loader
agos-lexer/         # MOD-02: Unicode validation + Lexer
agos-tokenizer/     # MOD-03: Tokenizer
agos-morph/         # MOD-04: Morphological parser
agos-syntax/        # MOD-05: Syntax parser
agos-gir/           # MOD-06: GIR constructor + IR schemas
agos-rule-engine/   # MOD-07: Rule engine + DSL compiler
agos-kg/            # MOD-08: Knowledge graph resolver
agos-bytecode/      # MOD-09: Bytecode generator
agos-gvm/           # MOD-10: Grammar Virtual Machine
agos-explanation/   # MOD-11: Explanation engine
agos-plugin/        # MOD-12: Plugin system
agos-cache/         # MOD-13: Cache manager
agos-api/           # MOD-14: API gateway
agos-cli/           # CLI binary
agos-server/        # Server binary
```

Build order follows the pipeline from text to output, producing a working end-to-end minimal path before optimizing.
