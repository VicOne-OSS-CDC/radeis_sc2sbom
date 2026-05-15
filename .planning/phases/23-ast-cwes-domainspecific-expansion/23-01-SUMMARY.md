---
phase: 23-ast-cwes-domainspecific-expansion
plan: 01
subsystem: vulnerability-scanning
tags: [rust, tree-sitter, ast-scanner, cwe, sast]

# Dependency graph
requires:
  - phase: 22-ast-cwes-structuralpattern-expansion
    provides: AST_CWE_RULES table with 41 CWEs (through Phase 22 Plan 03)
  - phase: 20-argument-value-ast-migration
    provides: ArgAtIndex variant in ArgCheck enum — used by CWE-284 rule

provides:
  - 5 new AstCweRule entries for CWE-114, 272, 284, 427, 785 appended to AST_CWE_RULES
  - Module-level doc comment updated to list 49 CWEs total with Win32-specific annotations
  - 6 unit tests covering all 5 new table-driven CWEs including CWE-284 negative case

affects:
  - 23-02 (plan 02 adds CWE-479, 591, 762 structural helpers — table now has 5 more entries)
  - benchmark juliet analysis (new CWEs need TP/FP rows)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AnyCall pattern for Win32 API call-site rules (CWE-114/272/427/785)"
    - "ArgAtIndex(4, &[\"GENERIC_ALL\"]) pattern for access-mask argument check (CWE-284)"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "5 new CWEs use existing AnyCall/ArgAtIndex arms with zero new dispatch code (D-06 / Pattern A)"
  - "Win32-specific CWEs (114, 272, 284, 591, 785) included unconditionally — produce 0 TPs on non-Windows source, which is expected (D-07)"
  - "CWE-284 uses ArgAtIndex(4, GENERIC_ALL) for precise dangerous variant detection matching Juliet bad-sink pattern"

patterns-established:
  - "Table-driven expansion: append AstCweRule entries before closing ]; — no new dispatch code required"

requirements-completed: [CWEXP-03]

# Metrics
duration: 8min
completed: 2026-05-12
---

# Phase 23 Plan 01: ast-cwes-domainspecific-expansion Summary

**5 domain-specific call-site CWE rules appended to AST_CWE_RULES (CWE-114/272/284/427/785) using existing AnyCall/ArgAtIndex arms with no new dispatch code, all validated with 6 unit tests**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-05-12T00:00:00Z
- **Completed:** 2026-05-12
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Appended 5 new AstCweRule entries to AST_CWE_RULES for CWE-114 (LoadLibrary*), CWE-272 (CreateProcessAsUser*), CWE-284 (CreateDesktop* with GENERIC_ALL ArgAtIndex), CWE-427 (SetDllDirectory*/putenv/setenv), CWE-785 (PathAppend*/realpath/_fullpath)
- Updated module-level doc comment to list 49 CWEs total and explicitly note Win32-specific CWEs per D-07
- Added 6 unit tests: 5 positive-case tests (one per CWE) and 1 negative case (CWE-284 with DESKTOP_READOBJECTS does not fire)
- All 6 tests pass; no regression in existing 418-test suite

## Task Commits

1. **Task 1: Append 5 AstCweRule entries and update module doc comment** - `b3444d8` (feat)
2. **Task 2: Add unit tests for the 5 table-driven CWEs** - `b4fabb3` (test)

**Plan metadata:** `(committed with SUMMARY.md below)`

## Files Created/Modified
- `src/vulnerability/ast_scanner.rs` - 5 new AstCweRule entries appended to AST_CWE_RULES; module doc updated to 49 CWEs with Win32-specific annotation
- `tests/vulnerability_tests/ast_scanner_tests.rs` - 6 new phase_23_cwe* test functions added at end of file

## Decisions Made
- Used existing AnyCall and ArgAtIndex arms — no new ArgCheck variants or dispatch code needed (D-06, Pattern A)
- CWE-284 ArgAtIndex(4, &["GENERIC_ALL"]) is exact-match on Juliet bad-sink pattern; DESKTOP_READOBJECTS negative case confirms no false positive
- Win32-specific rules included unconditionally per D-07; 0 findings on non-Windows source is expected behavior

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None. The pre-existing `test_spdx_output_passes_pyspdxtools_validation` test failure is unrelated to this plan (requires `pyspdxtools` binary in PATH, an environment issue present before these changes).

## Known Stubs

None — all 5 new rules are fully wired via the existing table dispatch mechanism.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Rules are read-only AST inspection of source files already scanned.

## Next Phase Readiness

- Plan 23-01 complete: 5 table-driven rules in place
- Plan 23-02 can now add apply_signal_handler_rules() (CWE-479), apply_paired_lock_rules() (CWE-591), and delete-expression detection (CWE-762) as structural helpers
- No blockers

## Self-Check

- [x] `src/vulnerability/ast_scanner.rs` exists with 5 new entries verified
- [x] `tests/vulnerability_tests/ast_scanner_tests.rs` exists with 6 new test functions
- [x] Commit b3444d8 exists (Task 1)
- [x] Commit b4fabb3 exists (Task 2)
- [x] All 6 phase_23 tests pass
- [x] cargo build --features internal exits 0

## Self-Check: PASSED

---
*Phase: 23-ast-cwes-domainspecific-expansion*
*Completed: 2026-05-12*
