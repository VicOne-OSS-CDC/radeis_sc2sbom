---
phase: 18-ast-scanner-core-and-benchmark
plan: 01
subsystem: vulnerability
tags: [rust, tree-sitter, feature-flag, sast, ast-scanner]

# Dependency graph
requires:
  - phase: 17-sarif-as-authoritative-finding-store
    provides: SastSource enum, SastFinding struct, cppcheck dedup pipeline
provides:
  - Feature flag merge: ast-scanner removed, tree-sitter deps in internal
  - SastSource::Ast variant for AST scanner provenance
  - ast_scanner module gated under #[cfg(feature = "internal")]
  - Wave 0 test scaffold: tests/vulnerability_tests/ast_scanner_tests.rs
  - Wave 0 benchmark scaffold: tests/benchmark.rs
affects:
  - 18-02: Plan 02 implements run_ast_scanner and removes #[ignore] from scaffolds
  - 18-03: Plan 03 implements benchmark body in tests/benchmark.rs

# Tech tracking
tech-stack:
  added: [tree-sitter 0.25.10 (now in internal feature), tree-sitter-c 0.24.2 (now in internal feature)]
  patterns: [SastSource enum extension for new scanner provenance, Wave 0 test scaffolding with #[ignore] markers]

key-files:
  created:
    - tests/vulnerability_tests/ast_scanner_tests.rs
    - tests/benchmark.rs
  modified:
    - Cargo.toml
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/ast_scanner.rs
    - src/vulnerability/mod.rs
    - tests/vulnerability_tests/mod.rs

key-decisions:
  - "D-01: ast-scanner feature merged into internal — one flag for all scanner code"
  - "D-03: SastSource::Ast variant added; dedup/suppress logic uses equality checks so no match arm exhaustiveness issue"
  - "DIST-01: tree-sitter-c MIT license documented in Cargo.toml comment (verified 0.24.2 crate metadata)"
  - "No match SastSource arms exist in codebase — only equality comparisons — so Ast variant addition required no arm updates"

patterns-established:
  - "Wave 0 scaffold pattern: create test files with #[ignore] markers and unimplemented!() stubs for tests that depend on Plan 02/03 symbols"
  - "Graceful-skip benchmark pattern: fixture_path() returns Option<PathBuf>, test early-returns with eprintln! when None"

requirements-completed: [AST-03, DIST-01]

# Metrics
duration: 35min
completed: 2026-05-11
---

# Phase 18 Plan 01: Foundation Summary

**Feature flag merge (ast-scanner into internal), SastSource::Ast variant, and Wave 0 test scaffolds for AST scanner and benchmark — foundation contracts for Plans 02/03**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-11T16:00:00Z
- **Completed:** 2026-05-11T16:28:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Merged `ast-scanner` feature into `internal` in Cargo.toml; both `cargo build --features internal` and `cargo build --no-default-features` succeed
- Added `SastSource::Ast` variant with doc comment; no non-exhaustive match errors because all existing SastSource usage is equality comparisons, not match arms
- Added tree-sitter-c MIT license documentation in Cargo.toml comment (DIST-01)
- Created `tests/vulnerability_tests/ast_scanner_tests.rs` with 3 `#[ignore]`-marked scaffold tests and 1 passing smoke test
- Created `tests/benchmark.rs` with 2 `#[ignore]`-marked benchmark tests and 1 passing smoke test
- All 4 existing PoC tests in ast_scanner.rs continue to pass under `--features internal`

## Task Commits

Each task was committed atomically:

1. **Task 1: Merge feature flag and document grammar license** - `289b37a` (feat)
2. **Task 2: Add SastSource::Ast variant and gate ast_scanner module** - `d35ad6c` (feat)
3. **Task 3: Create Wave 0 test scaffolds for ast_scanner and benchmark** - `95a2f99` (feat)

## Files Created/Modified

- `Cargo.toml` - Removed ast-scanner feature line; added tree-sitter/tree-sitter-c to internal feature; added MIT license comment
- `src/vulnerability/cwe_scanner.rs` - Added `Ast` variant with doc comment to SastSource enum
- `src/vulnerability/ast_scanner.rs` - Changed file-top attribute from `#![cfg(feature = "ast-scanner")]` to `#![cfg(feature = "internal")]`
- `src/vulnerability/mod.rs` - Added `#[cfg(feature = "internal")]` gate to `pub mod ast_scanner;` declaration
- `tests/vulnerability_tests/ast_scanner_tests.rs` - New: Wave 0 scaffold with setup_one_file helper, 3 #[ignore] tests, 1 smoke test
- `tests/vulnerability_tests/mod.rs` - Appended `#[cfg(feature = "internal")] mod ast_scanner_tests;` registration
- `tests/benchmark.rs` - New: Wave 0 benchmark scaffold with fixture_path helper, 2 #[ignore] benchmark tests, 1 smoke test

## Decisions Made

- No `match SastSource` arms exist in the codebase (all usages are equality comparisons with `==`), so adding `SastSource::Ast` required zero arm updates — the compiler confirmed exhaustiveness with no changes to `deduplicate_sast_findings()` or `suppress_lexical_false_positives()`
- Used `mod ast_scanner_tests` (not `pub mod`) to match the existing non-pub style in `tests/vulnerability_tests/mod.rs`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Pre-existing test `format_tests::spdx_validation_tests::test_spdx_output_passes_pyspdxtools_validation` fails because `pyspdxtools` external tool is not installed in the worktree environment. This is unrelated to this plan and pre-existed the changes.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 02 (`18-02-PLAN.md`) can now compile under `--features internal` and has test scaffolds to fill in
- `SastSource::Ast` is available for Plan 02 to use when emitting findings from `run_ast_scanner`
- `tests/vulnerability_tests/ast_scanner_tests.rs` scaffold tests ready for Plan 02 to remove `#[ignore]` markers
- `tests/benchmark.rs` scaffold ready for Plan 03 to implement the body

---
*Phase: 18-ast-scanner-core-and-benchmark*
*Completed: 2026-05-11*
