---
gsd_state_version: 1.0
milestone: v1.0.18
milestone_name: Tree-sitter AST Scanner
status: ready_to_plan
stopped_at: Phase 25 complete — v1.0.18 milestone shipped
last_updated: "2026-05-13T00:00:00.000Z"
last_activity: 2026-05-13
progress:
  total_phases: 9
  completed_phases: 9
  total_plans: 22
  completed_plans: 22
  percent: 100
---

# State: radeis_sc2sbom

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-11)

**Core value:** Accurate, spec-compliant SBOM output that downstream consumers (xZETA, compliance tools) can ingest without errors.
**Current focus:** v1.0.18 shipped — Phase 25 experiment-scan-mode complete

## Milestone Progress

**v1.0.18 COMPLETE** | 8 phases (18–25) | 8/8 complete

| Phase | Status | Plans |
|-------|--------|-------|
| 18 — AST Scanner Core & Benchmark | ✅ Complete | 3/3 |
| 19 — cppcheck Removal | ✅ Complete | 1/1 |
| 20 — Argument-Value AST Migration | ✅ Complete | 2/2 |
| 21 — AST CWE AnyCall/ArgPattern Expansion (+12 CWEs → 25 total) | ✅ Complete | 3/3 |
| 22 — AST CWE StructuralPattern Expansion (+15 CWEs → 40 total) | ✅ Complete | 4/4 |
| 23 — AST CWE DomainSpecific Expansion (+8 CWEs → 48 total) | ✅ Complete | 3/3 |
| 24 — Tune High-FP CWE Rules | ✅ Complete | 4/4 |
| 25 — Experiment Scan Mode | ✅ Complete | 2/2 |

Progress: [██████████] 100%

## Current Position

Phase: 999.1
Plan: Not started
Status: Ready to plan
Last activity: 2026-05-13
Note: experiment-scan flag gates 17 high-FP CWEs; default scan runs 22 CWEs

## Config

- Mode: interactive
- Granularity: standard
- Parallelization: sequential
- Commit docs: yes
- Model profile: balanced

## Accumulated Context

### Key Decisions (v1.0.16 — carried forward)

- Gate CVE + CWE + lexical scanner behind single `cargo feature = "internal"` — compiler enforces exclusion; no strip script needed
- SCAN-03 (CWE-134) uses next-token heuristic: flag `printf`/`fprintf`/`syslog` only when format arg is not a string literal
- SCAN-05: scanner is scoped to component-mapped C/C++ dirs only — not full source tree
- CDX-04: SAST findings in CycloneDX output only — SPDX 2.3 has no native vulnerability model

### Key Decisions (v1.0.17 — carried forward)

- Phase 13 before Phase 14: argument-value matching extends the existing Rust rule engine with zero external dependencies; must be complete before cppcheck output is merged into the same SastFinding pipeline
- Phase 14 before Phase 15: SARIF output consumes the unified finding stream (lexical + cppcheck); deduplication must be in place before SARIF serialization
- SARIF normalization via SastFinding: cppcheck's native SARIF format differs from our schema; normalize through SastFinding struct instead of piping cppcheck SARIF directly (Out of Scope in REQUIREMENTS.md)
- Phase 16 added 2026-05-11: SARIF as authoritative finding store — findings fingerprinted at source, markdown rendered from SARIF, cppcheck suppresses lexical false positives, --sarif-baseline enables new-findings-only CI workflows

### Roadmap Evolution

- Phase 21 added 2026-05-12: AST CWE AnyCall/ArgPattern expansion — 13 new CWEs (121,126,328,338,369,426,467,526,535,605,676,680,780); driven by full Juliet corpus benchmark showing 47 CWEs tractable with AST alone
- Phase 22 added 2026-05-12: AST CWE StructuralPattern expansion — 15 new CWEs (256,398,478,480,481,482,483,484,562,570,571,587,617,674,835); pure AST shape queries, no dataflow required
- Phase 23 added 2026-05-12: AST CWE DomainSpecific expansion — 8 new CWEs (114,272,284,427,479,591,762,785); narrow API-specific rules
- Milestone scope updated from Phases 18–20 to Phases 18–23; target coverage 13→49 CWEs

### Key Decisions (v1.0.18 — new)

- Phase 18 (benchmark) before Phase 19 (removal): cppcheck fate is data-driven; removal/demotion decision requires documented false-positive comparison on reference fixtures
- Phase 19 (removal) before Phase 20 (arg-value migration): argument-value AST rules verified against the final scanner configuration, not an intermediate cppcheck-still-present state
- SastFinding struct unchanged: AST scanner emits SastFinding identical to lexical scanner — SARIF writer, markdown report, and CycloneDX serializer require no downstream changes (AST-03)
- Parse failure fallback: tree-sitter parse failure on a single file falls back to lexical scan for that file and logs a warning; overall scan continues (AST-04)
- tree-sitter-c grammar embedded in binary: no runtime file dependency; grammar source compiled into the musl binary via build.rs (DIST-02)

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260510-2a4 | Fix component-mapping inflation in lexical scanner | 2026-05-10 | 6701972 | [260510-2a4-fix-component-mapping-inflation-in-lexic](.planning/quick/260510-2a4-fix-component-mapping-inflation-in-lexic/) |
| 260510-326 | Add fallback mode to lexical scanner for standalone C projects | 2026-05-10 | 0cf1809 | [260510-326-add-fallback-mode-to-lexical-scanner-for](.planning/quick/260510-326-add-fallback-mode-to-lexical-scanner-for/) |
| 260511-giz | Split CWE ID→name mapping out of console.rs into vulnerability/cwe_map.rs | 2026-05-11 | e045797 | [20260511-split-cwe-mapping](.planning/quick/20260511-split-cwe-mapping/) |
| 260511-d6p | Raise has_c_cpp_files depth 3→6 for deeply-nested projects (AUTOSAR fix) | 2026-05-11 | a8b9776 | — |
| 260513-fsg | Drop CWE-20 and CWE-22 from lexical scanner (100% FP, no tractable local fix) | 2026-05-13 | 8532e4e | [260513-fsg-drop-cwe-20-and-cwe-22-from-ast-scanner-](.planning/quick/260513-fsg-drop-cwe-20-and-cwe-22-from-ast-scanner-/) |

### Active Todos

None.

### Blockers

None.

## Deferred Items

Items acknowledged and deferred at milestone close on 2026-05-11 (v1.0.17):

| Category | Item | Status | Notes |
|----------|------|--------|-------|
| phase | Phase 10 — Internal Feature Gate (GATE-01..GATE-04) | promoted to Phase 11 in v1.0.16 | v1.0.15 close |
| quick_task | 260509-gwq-fix-output-flag-single-formats | false-positive (SUMMARY exists on disk; audit SDK slug-match bug) | shipped 2026-05-09 |
| quick_task | 260510-2a4-fix-component-mapping-inflation-in-lexic | false-positive (SUMMARY exists on disk; audit SDK slug-match bug) | shipped 2026-05-10 |
| quick_task | 260510-326-add-fallback-mode-to-lexical-scanner-for | false-positive (SUMMARY exists on disk; audit SDK slug-match bug) | shipped 2026-05-10 |
| quick_task | 20260511-autosar-version-extraction | complete — SUMMARY.md written; work captured in Phase 17 commits 14e60a5..d8ca713 | shipped 2026-05-11 |
| seed | SEED-001-v1.0.16-cwe-lexical-scanner | shipped as v1.0.16 milestone; seed dormant/superseded | completed |
| seed | SEED-002-v1.0.17-cppcheck-integration | shipped as v1.0.17 milestone; seed dormant/superseded | completed |
| seed | SEED-003-v1.0.18-tree-sitter-ast-scanner | active — roadmap defined as Phases 18–20 | in progress |

## Session Continuity

Last session: 2026-05-13T07:12:36.757Z
Stopped at: Milestone v1.0.18 complete — all phases done
Resume file: None
