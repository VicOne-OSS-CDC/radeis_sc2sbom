---
phase: 19-cppcheck-removal
plan: "01"
subsystem: vulnerability/sast
tags: [cppcheck-removal, sast, refactor, CPP-01]
dependency_graph:
  requires: []
  provides: [clean-ast-sast-pipeline]
  affects: [src/vulnerability/cwe_scanner.rs, src/vulnerability/mod.rs, src/cli.rs, src/main.rs]
tech_stack:
  added: []
  patterns: [ast-scanner-primary, lexical-fallback, deduplicate-ast-lexical]
key_files:
  created: []
  modified:
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - src/cli.rs
    - src/main.rs
    - tests/vulnerability_tests/mod.rs
    - tests/vulnerability_tests/sarif_consistency_tests.rs
    - docs/BENCHMARK.md
  deleted:
    - tests/benchmark.rs
    - tests/vulnerability_tests/suppression_tests.rs
    - tests/vulnerability_tests/cppcheck_scanner_tests.rs
decisions:
  - "SastSource::Cppcheck variant deleted; SastSource::Both repurposed to AST∩Lexical"
  - "deduplicate_sast_findings renamed params from (lexical, cppcheck) to (ast, lexical)"
  - "SAST pipeline simplified to: run_ast_scanner -> deduplicate_sast_findings(ast, vec![])"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-12"
  tasks_completed: 3
  tasks_total: 3
---

# Phase 19 Plan 01: cppcheck Hard Removal Summary

Hard-removed cppcheck subprocess integration from sc2sbom SAST pipeline; AST scanner is now primary with no external subprocess dependency.

## What Was Done

### Task 1 — Delete cppcheck-dependent test files (commit dc324c3)

**Files deleted:**
- `tests/benchmark.rs` — Phase 18 benchmark test that ran all three scanners; served its decision purpose
- `tests/vulnerability_tests/suppression_tests.rs` — tested `suppress_lexical_false_positives` and used `SastSource::Cppcheck`
- `tests/vulnerability_tests/cppcheck_scanner_tests.rs` — tested `parse_cppcheck_xml`, `run_cppcheck_scanner`, and `deduplicate_sast_findings` with cppcheck findings

**Modified:**
- `tests/vulnerability_tests/mod.rs` — removed `cppcheck_scanner_tests` and `suppression_tests` module declarations

### Task 2 — Atomic source surgery (commit ea41a3e)

**Functions deleted from `src/vulnerability/cwe_scanner.rs`:**
- `run_cppcheck_scanner(component_dirs, cppcheck_bin)` — spawned external cppcheck subprocess
- `parse_cppcheck_xml(xml_bytes, component_name, component_ecosystem)` — parsed cppcheck XML v2 output
- `suppress_lexical_false_positives(findings, cppcheck_scanned_dirs, cppcheck_confirmed)` — dropped lexical FPs covered by cppcheck

**Constants deleted:**
- `const CPPCHECK_COVERED_CWES: &[u32]` — empty set of CWEs covered by both scanners
- `static CPPCHECK_CWE_OVERRIDES: &[(&str, u32)]` — 15-entry fallback table for cppcheck error IDs without CWE attributes

**Enum variant deleted:**
- `SastSource::Cppcheck` — replaced semantically by `SastSource::Ast`; `SastSource::Both` repurposed to mean AST∩Lexical

**CLI arg deleted:**
- `--cppcheck-path` field (`cppcheck_path: Option<PathBuf>`) from `src/cli.rs`

**`deduplicate_sast_findings` signature change:**
- BEFORE: `(lexical: Vec<SastFinding>, cppcheck: Vec<SastFinding>) -> Vec<SastFinding>`
- AFTER: `(ast: Vec<SastFinding>, lexical: Vec<SastFinding>) -> Vec<SastFinding>`
- First loop processes AST findings; second loop promotes to `SastSource::Both` on collision

**`SastSource::Both` semantic repurpose:**
- BEFORE: lexical finding confirmed by cppcheck (lexical∩cppcheck)
- AFTER: AST finding also detected by lexical fallback scanner (AST∩Lexical) — higher confidence

**Pipeline change in `src/main.rs`:**
- Removed: WR-01 warning block, `cppcheck_bin`, `run_cppcheck_scanner` call, `cppcheck_confirmed` BTreeSet, `suppress_lexical_false_positives` call
- Replaced with: `let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs); sast_findings = crate::vulnerability::deduplicate_sast_findings(ast_findings, vec![]);`

**Imports trimmed from `cwe_scanner.rs`:**
- `use indicatif::{ProgressBar, ProgressStyle}`
- `use quick_xml::events::Event`
- `use quick_xml::Reader`
- `use std::ffi::OsStr`
- `use std::process::{Command, Stdio}`
- `BTreeSet` from `std::collections`

**Also fixed (Rule 1 deviation):**
- `tests/vulnerability_tests/sarif_consistency_tests.rs` line 24: replaced `SastSource::Cppcheck` with `SastSource::Ast` to fix compile error caused by deleted variant

### Task 3 — Add dedup tests and update BENCHMARK.md (commit e46adae)

**New inline tests added to `src/vulnerability/cwe_scanner.rs`:**
- `test_deduplicate_ast_and_lexical_merge` — verifies same (file, line, cwe) in both inputs produces `SastSource::Both`
- `test_deduplicate_ast_only_passthrough` — verifies `deduplicate_sast_findings(ast, vec![])` returns findings unchanged with `SastSource::Ast`

**`docs/BENCHMARK.md` updated:**
- Header retitled: "Phase 18 Benchmark — AST vs cppcheck vs Lexical (Historical Artifact)"
- Added Phase 19 status note explaining why benchmark was deleted and that cppcheck columns are preserved as historical evidence
- Removed stale "To populate this file" instructions (test no longer exists)

## Final Test Results

```
test result: ok. 288 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Both new tests passed:
- `test vulnerability::cwe_scanner::tests::test_deduplicate_ast_and_lexical_merge ... ok`
- `test vulnerability::cwe_scanner::tests::test_deduplicate_ast_only_passthrough ... ok`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed SastSource::Cppcheck reference in sarif_consistency_tests.rs**
- **Found during:** Task 2 compile-clean pass
- **Issue:** `tests/vulnerability_tests/sarif_consistency_tests.rs` line 24 constructed a finding with `SastSource::Cppcheck` — compile error after enum variant deletion
- **Fix:** Replaced `SastSource::Cppcheck` with `SastSource::Ast`
- **Files modified:** `tests/vulnerability_tests/sarif_consistency_tests.rs`
- **Commit:** ea41a3e (included in Task 2 atomic commit)

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. This phase is a pure deletion — the only change to the trust surface is the removal of `Command::new("cppcheck")` (T-19-01, T-19-05, T-19-06 — all mitigated by removal).

## Self-Check: PASSED

- `src/vulnerability/cwe_scanner.rs` exists: FOUND
- `src/vulnerability/mod.rs` exists: FOUND
- `src/cli.rs` exists: FOUND
- `src/main.rs` exists: FOUND
- `tests/benchmark.rs` does NOT exist: CONFIRMED
- `tests/vulnerability_tests/suppression_tests.rs` does NOT exist: CONFIRMED
- `tests/vulnerability_tests/cppcheck_scanner_tests.rs` does NOT exist: CONFIRMED
- Commit dc324c3: FOUND
- Commit ea41a3e: FOUND
- Commit e46adae: FOUND
- `cargo test --features internal` passes 288 tests, 0 failed: CONFIRMED
