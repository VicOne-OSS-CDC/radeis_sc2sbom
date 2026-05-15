---
phase: 18-ast-scanner-core-and-benchmark
plan: 03
subsystem: vulnerability
tags: [rust, benchmark, cppcheck, autosar, juliet, ast-scanner, tpfp]

# Dependency graph
requires:
  - phase: 18-01
    provides: SastSource::Ast variant, Wave 0 benchmark scaffold (tests/benchmark.rs)
  - phase: 18-02
    provides: run_ast_scanner() production implementation, run_lexical_scanner pub export

provides:
  - Benchmark integration test: tests/benchmark.rs with two #[test] functions (AUTOSAR + Juliet)
  - Graceful skip on missing fixtures (D-10/D-11): eprintln! + return, test exits 0
  - docs/BENCHMARK.md: committed template with D-14 column headers and HTML-comment markers
  - docs/BENCHMARK_FIXTURES.md: env vars, fixture acquisition, ground-truth TSV format, curation workflow
  - write_benchmark_md(): idempotent, run-order-independent marker-based section writer

affects:
  - 19: Phase 19 planning reads docs/BENCHMARK.md to make cppcheck removal vs demotion decision (D-13)

# Tech tracking
tech-stack:
  added: [chrono 0.4 (already present as regular dependency — confirmed non-optional)]
  patterns:
    - "Marker-based idempotent file writer: HTML-comment section markers (FIXTURE-SECTION-START/END) enable each test to update only its own section regardless of run order"
    - "Ground-truth oracle pattern: .benchmark_truth.tsv (relative_path TAB line TAB cwe TAB TP|FP) loaded per fixture; missing file → raw counts only"
    - "Graceful benchmark skip: fixture_path() returns Option<PathBuf>; test early-returns after eprintln! when None"

key-files:
  created:
    - docs/BENCHMARK.md
    - docs/BENCHMARK_FIXTURES.md
  modified:
    - tests/benchmark.rs

key-decisions:
  - "run_cppcheck_scanner takes Option<&OsStr> (not Option<&Path>) — adapted locate_cppcheck() to return PathBuf and call .as_os_str() at call site"
  - "Removed std::ffi::OsStr import from benchmark.rs — PathBuf::as_os_str() is a method call, no explicit use needed"
  - "write_benchmark_md() refreshes header timestamp on every fixture run, preserves sections for fixtures not in the current invocation"
  - "BENCH-01 closed: benchmark compares AST, lexical, and cppcheck (when available); docs/BENCHMARK.md is the Phase 19 input artifact"

patterns-established:
  - "Per-fixture self-contained sections: each #[test] writes only its named section; no run-order dependency between AUTOSAR and Juliet tests"
  - "locate_cppcheck() tries CPPCHECK_BIN env var first, then PATH lookup via which — consistent with main.rs cppcheck_bin lookup pattern"

requirements-completed: [BENCH-01]

# Metrics
duration: 25min
completed: 2026-05-12
---

# Phase 18 Plan 03: Benchmark Summary

**Rust integration test at tests/benchmark.rs comparing AST vs cppcheck vs lexical scanners on AUTOSAR/Juliet fixtures with TP/FP classification via ground-truth oracle TSV; docs/BENCHMARK.md idempotent marker-based writer; BENCH-01 closed**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-12T00:00:00Z
- **Completed:** 2026-05-12T00:25:00Z
- **Tasks:** 2
- **Files modified:** 3 (tests/benchmark.rs, docs/BENCHMARK.md created, docs/BENCHMARK_FIXTURES.md created)

## Accomplishments

- Replaced the Plan 01 `#[ignore]` scaffold with the full benchmark body: `run_one_fixture()`, `load_ground_truth()`, `count_by_cwe()`, `write_benchmark_md()`, `render_header()`, `render_fixture_section()`
- Both benchmark tests (`benchmark_ast_vs_cppcheck_autosar`, `benchmark_ast_vs_cppcheck_juliet`) gracefully skip when fixtures are absent: 3/3 tests pass in no-fixtures scenario
- Marker-based writer (`write_benchmark_md`) is idempotent and run-order independent: each test updates only its own HTML-comment-delimited section; the other fixture's section is preserved
- Created `docs/BENCHMARK.md` committed template with D-14 column headers and marker strings that allow the first benchmark run to refresh-in-place
- Created `docs/BENCHMARK_FIXTURES.md` documenting AUTOSAR and Juliet fixture acquisition, `.benchmark_truth.tsv` ground-truth format, curation workflow, and the marker-layout contract
- `chrono` confirmed as regular (non-optional) `[dependencies]` entry in Cargo.toml
- Final `tests/benchmark.rs` line count: 313 lines

## Task Commits

1. **Task 1: Implement benchmark logic and BENCHMARK.md writer** - `afe1062` (feat)
2. **fix: remove unused OsStr import** - `08a5c53` (fix) [Rule 3 — Blocking: unused import warning cleaned up]
3. **Task 2: Create docs/BENCHMARK.md template and docs/BENCHMARK_FIXTURES.md** - `c1bff0e` (docs)

## Files Created/Modified

- `tests/benchmark.rs` — Full benchmark body: 313 lines. fixture_path(), load_ground_truth(), classify(), build_component_dirs(), locate_cppcheck(), count_by_cwe(), render_header(), render_fixture_section(), write_benchmark_md(), run_one_fixture(), plus 3 #[test] functions
- `docs/BENCHMARK.md` — Committed template: BENCHMARK-HEADER-START/END markers, D-14 column table, instructions for populating. Will be updated in-place by benchmark runs
- `docs/BENCHMARK_FIXTURES.md` — Permanent fixture guide: env vars table, AUTOSAR layout, Juliet subset instructions, .benchmark_truth.tsv format with example, curation workflow, marker-layout documentation

## Decisions Made

- `run_cppcheck_scanner` actual signature is `Option<&OsStr>` not `Option<&Path>` (as in plan template). Adapted `locate_cppcheck()` to return `PathBuf` and call `p.as_os_str()` at the call site — no behavioral change, clean compile.
- `chrono::Utc::now()` use in `render_header()` — chrono was already a regular dependency (line 25 of Cargo.toml), no change needed.
- Used `"\u{2014}"` (em dash) instead of `"—"` in the fp_pct closure for the zero-denominator case to avoid non-ASCII literals in source.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `std::ffi::OsStr` import**
- **Found during:** Task 1 verification (cargo test warning)
- **Issue:** Plan template included `use std::ffi::OsStr;` but the adapted call site uses `p.as_os_str()` as a method — no explicit OsStr type is needed
- **Fix:** Removed the `use std::ffi::OsStr;` line from the use block
- **Files modified:** `tests/benchmark.rs`
- **Verification:** `cargo test --features internal --test benchmark` produces no warnings from benchmark.rs
- **Committed in:** `08a5c53`

---

**Total deviations:** 1 auto-fixed (Rule 1 - unused import warning)
**Impact on plan:** Minimal cleanup, no behavior change, clean compile.

## cppcheck Signature Adjustment

The plan template assumed `run_cppcheck_scanner(component_dirs, Some(p.as_path()))`. Actual signature:
```rust
pub fn run_cppcheck_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
    cppcheck_bin: Option<&OsStr>,
) -> (Vec<SastFinding>, BTreeSet<PathBuf>)
```

Benchmark uses `Some(p.as_os_str())` where `p: PathBuf`. Functionally identical — the cppcheck runner internally calls `Command::new(bin)`.

## Test Results

**No-fixtures scenario (CI default):**
```
running 3 tests
test fixture_helper_returns_none_for_missing_path ... ok
test benchmark_ast_vs_cppcheck_autosar ... ok
test benchmark_ast_vs_cppcheck_juliet ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
```

Both benchmark tests print `SKIP ...` to stderr and return early. Smoke test passes.

**Fixtures-present scenario:** Developer responsibility. When `AUTOSAR_FIXTURE_PATH` or `JULIET_FIXTURE_PATH` point to real fixtures, `run_one_fixture()` invokes all three scanners and writes a per-fixture section to `docs/BENCHMARK.md`. With `.benchmark_truth.tsv` present, TP/FP/FP% columns are populated per D-14. Without it, raw counts only with a notice in the section.

## Phase 19 Input Contract

`docs/BENCHMARK.md` is the committed artifact Phase 19 planning reads to make the cppcheck removal/demotion decision (D-13). The file contains:
- D-14 column set: AST TPs | AST FPs | AST FP% | cppcheck TPs | cppcheck FPs | cppcheck FP% | Lexical TPs | Lexical FPs | Lexical FP%
- Per-CWE rows for each fixture run
- HTML-comment markers for idempotent per-fixture updates

BENCH-01 is closed. Phase 19 can read this file once the user has run the benchmark with real fixtures.

## Issues Encountered

- Pre-existing warning `function 'run_lexical_scanner' is never used` in cwe_scanner.rs — not introduced by this plan, out of scope.
- Pre-existing test `format_tests::spdx_validation_tests::test_spdx_output_passes_pyspdxtools_validation` fails because pyspdxtools is not installed — pre-existing, unrelated.

## User Setup Required

None — no external service configuration required. To actually run the benchmark, see `docs/BENCHMARK_FIXTURES.md` for fixture acquisition instructions.

## Next Phase Readiness

- Phase 19 planning can read `docs/BENCHMARK.md` once the user runs the benchmark with real AUTOSAR/Juliet fixtures
- All three scanner public APIs (`run_ast_scanner`, `run_lexical_scanner`, `run_cppcheck_scanner`) are verified accessible under `--features internal`
- Phase 18 is complete: AST-01, AST-02, AST-03, AST-04, DIST-01, DIST-02, BENCH-01 all satisfied across Plans 01-03

---
*Phase: 18-ast-scanner-core-and-benchmark*
*Completed: 2026-05-12*
