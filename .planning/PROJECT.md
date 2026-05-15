# radeis_sc2sbom

## Current Milestone: v1.0.18 — Tree-sitter AST Scanner

**Goal:** Replace the cppcheck subprocess with an embedded tree-sitter-c AST scanner, eliminating the external install requirement while achieving higher CWE detection precision via parse-tree context.

**Target features:**
- AST scanner core: wire `ast_scanner.rs` as default C/C++ scanner, expand to full 14-CWE rule set, benchmark parallel to cppcheck on AUTOSAR_SampleProject_S32K144
- cppcheck fate: decide removal vs demotion based on Phase A false-positive comparison data
- Argument-value rules migrated to AST argument node inspection (CWE-295, CWE-319, CWE-732) for higher precision
- Parse failures fall back to lexical scanner; tree-sitter-c MIT license verified for static musl linking

## What This Is

A Rust CLI tool that scans source code directories and generates Software Bill of Materials (SBOM) in SPDX 2.3 and CycloneDX 1.5 formats. The internal build adds a SAST scanner that detects dangerous C/C++ function calls and API misuse (14+ CWEs), surfaces findings in CycloneDX output, static analysis report, and SARIF output. Supports C/C++, Rust, Python, Node, Ruby, PHP, Java, ROS, AI models, and AUTOSAR. Outputs are compatible with xZETA and other SBOM management platforms.

## Core Value

Accurate, spec-compliant SBOM output that downstream consumers (xZETA, compliance tools) can ingest without errors.

## Requirements

### Validated

- ✓ Multi-ecosystem parsing (C/C++, Rust, Python, npm, etc.) — v1.0.0–v1.0.3
- ✓ SPDX 2.3 and CycloneDX 1.5 output formats — v1.0.0
- ✓ Version resolution from .mk files and .so binaries — v1.0.5
- ✓ AI model support: GGUF and Safetensors — v1.0.9–v1.0.11
- ✓ Rich AI model metadata (multimodal sub-model decomposition) — v1.0.12–v1.0.13
- ✓ Vulnerability scanning via OSV API with Debian fallback — v1.0.7+
- ✓ Scope classification for dependencies — v1.0.6
- ✓ Broken symlink tolerance — all WalkDir sites warn and continue — v1.0.14 (REL-01, REL-02)
- ✓ Makefile variable references ($(...)) filtered — emit NOASSERTION — v1.0.14 (DAT-01, DAT-04)
- ✓ C/C++ library licenses resolve from .pc files and known-library lookup table — v1.0.14 (DAT-02, DAT-03)
- ✓ Linux binary statically linked via musl — no glibc dependency — v1.0.14 (DIST-01, DIST-02)
- ✓ AUTOSAR project detection via .arxml, BSW/MCAL/RTE dirs, build-file variables — v1.0.15 (DET-01..03)
- ✓ BSW module classification (layer, platform) in SPDX 2.3 and CycloneDX 1.5 — v1.0.15 (CLS-01..03, OUT-01, OUT-02)
- ✓ Supplier mapping via YAML config; NOASSERTION for unmatched components — v1.0.15 (OUT-03)
- ✓ NVD CWE enrichment for CVEs with TTL disk cache; CWE IDs in all output formats — v1.0.15 (CWE-01..04)
- ✓ `cargo feature = "internal"` gate — public binary contains zero vuln/scanner symbols — v1.0.16 (GATE-01..04)
- ✓ Dangerous-function lexical scanner over C/C++ files; 14 CWEs detected with file:line — v1.0.16 (SCAN-01..05)
- ✓ CycloneDX 1.5 `vulnerabilities[]` entries for SAST findings (cwes[], affects[].ref, properties) — v1.0.16 (CDX-01..04)
- ✓ Separate `_static_analysis.md` report with per-component CWE summary table + file:line findings — v1.0.16 (RPT-01)
- ✓ SAST findings section integrated into main `_report.md` — v1.0.16 (RPT-02)
- ✓ CLI disclaimer emitted when static analysis runs — v1.0.16 (RPT-03)
- ✓ Argument-value matching: CWE-295/319/732/369 via AND-all paren-arg token inspection — v1.0.17 (ARGVAL-01..05)
- ✓ cppcheck subprocess integration: SastSource enum, XML v2 parser, subprocess driver, pipeline dedup — v1.0.17 (CPPCHECK-01..05)
- ✓ SARIF 2.1 output as authoritative finding store: SHA-256 fingerprints, --sarif-baseline CI gate, cppcheck suppression of lexical FPs — v1.0.17 (SARIF-01..07)
- ✓ AUTOSAR arxml parser: SW-COMPONENT-PROTOTYPE, BSW-MODULE-DESCRIPTION, SWC type definitions; epd REVISION-LABEL + Doxygen SW Version populate real versions — v1.0.17 (BUG-01..04)

### Active

- [ ] AST scanner core: wire tree-sitter-c as default C/C++ scanner with full 14-CWE rule set (v1.0.18)
- [ ] cppcheck removal/demotion: decided based on Phase A false-positive benchmarks (v1.0.18)
- [ ] Argument-value rules migrated to AST argument node inspection — CWE-295/319/732 (v1.0.18)

### Out of Scope

- Full Makefile variable expansion (multi-hop, recursive make) — complexity vs. value tradeoff; emit NOASSERTION instead
- License detection from source headers/LICENSE files — future scope
- .NET / NuGet support — backlogged
- xZETA-specific schema alignment beyond fixing known bugs
- ARXML parsing for SWC extraction — heuristic detection was sufficient; full arxml parser now ships in v1.0.17
- CWE-401 (memory leak) — requires control-flow graph; lexical FP rate >50%
- CWE-457 (uninitialized variable) — requires liveness analysis
- CWE-415/416 (double free / use-after-free) — require dataflow; deferred to v1.0.17 tree-sitter
- CWE-476 (null pointer dereference) — requires dataflow; deferred to v1.0.17
- SARIF output — not required for xZETA ingestion; future scope
- Full dataflow / taint analysis — SAST scope, not SBOM scope

## Context

- Language: Rust, targeting x86_64-unknown-linux-musl (static binary)
- Current version: v1.0.17 (shipped 2026-05-11)
- Source: ~43,200 lines Rust (estimated; +16,421 lines in v1.0.17)
- Two build variants: public (no internal feature) and internal (with lexical CWE scanner + CVE/NVD enrichment)
- Test target: xcar-linux repo (automotive Linux, predominantly C/C++)
- Bug reporter: Koyama Chihiro (VicOne)
- OSV API with Debian fallback for native C/C++ libs
- Vendored ecosystem has two casings: "vendored" and "VENDORED"

## Constraints

- **Tech Stack**: Rust — all parsers and formatters must be in Rust
- **Compatibility**: SPDX 2.3 spec compliance required; CycloneDX 1.5 spec compliance required
- **Distribution**: Linux binary must run on Ubuntu 22.04+ (static musl binary as of v1.0.14)
- **SBOM Quality**: versionInfo must never contain raw Makefile variable references
- **Open-source safety**: Public binary must compile without any vulnerability or scanner symbols

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Gate CVE + CWE + lexical scanner behind single `cargo feature = "internal"` | Compiler enforces exclusion in public binary; no manual strip script maintenance needed; source can be open-sourced safely | ✓ Done — Phase 10 |
| component_dirs field unconditional on ScanContext | Pitfall 5 from Phase 11 planning: cfg-gating a struct field causes downstream cfg cascade; field carries no runtime cost when scanner is absent | ✓ Done — Phase 11 |
| Inline `#[cfg(feature = "internal")]` per-parameter gating on formatter functions | Consistent with existing SbomMode param pattern; avoids cfg-split function duplication | ✓ Done — Phase 11, 12 |
| dep_to_bom_ref rebuilt in SAST path (no shared helper extracted) | Avoids refactoring the CVE path for a minor structural improvement; acceptable duplication | ✓ Done — Phase 11 |
| SCAN-03 (CWE-134) uses next-token heuristic | Format string vulnerabilities fire only when format arg is not a string literal; reduces FP rate | ✓ Done — Phase 11 |
| SAST findings in CycloneDX output only (not SPDX 2.3) | SPDX 2.3 has no native vulnerability model; CDX-04 constraint | ✓ Done — Phase 11 |
| Fallback: synthetic (project, C/C++) → scan_root when component_dirs empty | Standalone C repos (Juliet, bare projects) have no manifest-derived component dirs; fallback enables scanning without requiring a build system | ✓ Done — Quick 260510-326 |
| detect_autosar() as pre-pass; result as is_autosar: bool param | Cleaner than post-hoc patching; scan_directory receives flag directly | ✓ Done — Phase 6 |
| Option<&SupplierResolver> as trailing param on all formatter functions | Avoid struct proliferation; trailing arg is idiomatic for optional context | ✓ Done — Phase 8 |
| NVD cache key = sha256(cve_id); TTL disk cache + 6s rate-limit sleep | NVD 2.0 anonymous rate limit; cache avoids repeat hits on re-runs | ✓ Done — Phase 9 |
| Phase 13 (arg-value) before Phase 14 (cppcheck): extend Rust rule engine first | Arg-value matching has zero external deps; must be complete before cppcheck merges into the same SastFinding pipeline | ✓ Done — Phase 13 |
| SARIF normalization via SastFinding (not piped cppcheck SARIF) | cppcheck native SARIF schema differs from ours; normalizing through SastFinding struct avoids schema mismatch | ✓ Done — Phase 14 |
| SARIF as authoritative store with SHA-256 partialFingerprints | Stable fingerprints enable reliable baseline diffing across runs; markdown rendered from SARIF ensures consistency | ✓ Done — Phase 16 |
| suppress_lexical_false_positives: drop Lexical when cppcheck covers the CWE and ran on the dir | cppcheck has higher precision on dataflow-backed CWEs; lexical hits in same scope are noise | ✓ Done — Phase 16 |
| arxml parser in Rust using quick-xml (no new dep) | quick-xml already in Cargo.toml; keeps binary size constant; no external XML lib needed | ✓ Done — Phase 17 |
| epd/Doxygen version maps built once per project scan, passed into parse_arxml | Avoids re-walking files per-arxml call; O(1) lookup at parse time | ✓ Done — Phase 17 |
| musl target for Linux binary | Eliminates glibc version dependency; zero runtime requirements | ✓ Done — Phase 4 |
| Filter $(VAR) version values at parser level + formatter guard | Defense-in-depth; parsers shouldn't store unexpandable refs | ✓ Done — Phase 2 |
| Known-library license lookup table for C system libs | Quick win for 286 NOASSERTION licenses; source scanning deferred | ✓ Done — Phase 3 |
| WalkDir: convert broken-symlink errors to warnings | Scan must not abort; broken symlinks are common in dev repos | ✓ Done — Phase 1 |

---
*Last updated: 2026-05-12 after Phase 23 complete — 49 CWEs now covered by AST scanner (5 domain-specific AstCweRule entries added: CWE-114, 272, 284, 427, 785; 3 structural helpers added: CWE-479, 591, 762; Juliet benchmark updated to 217,279 findings; CWE-762 (58.5% FP) added to backlog 999.2 for fine-tuning)*

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state
