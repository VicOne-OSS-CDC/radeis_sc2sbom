---
phase: 20-argument-value-ast-migration
plan: 02
subsystem: vulnerability
tags: [lexical-scanner, cwe-295, cwe-319, cwe-732, ast, cleanup, refactor]

# Dependency graph
requires:
  - phase: 20-argument-value-ast-migration/20-01
    provides: ArgAtIndex AST rules for CWE-295/319/732, 11 AST argval tests — makes lexical rules redundant

provides:
  - Cleaned CweRule struct with exactly 4 fields (no arg_value_contains)
  - Reduced CWE_RULES table: 15 entries, 14 distinct CWE IDs (CWE-295/319/732 removed)
  - paren_args_contain_all deleted (no longer needed)
  - Lexical scanner no longer detects CWE-295/319/732 (AST scanner is sole authority per D-12)

affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "D-12: AST scanner is sole authority for CWE-295/319/732; lexical scanner is name-match-only fallback for these"
    - "D-13: CweRule struct has exactly 4 fields; arg_value_contains machinery eliminated from lexical path"

key-files:
  created: []
  modified:
    - src/vulnerability/cwe_scanner.rs

key-decisions:
  - "D-12: Lexical scanner no longer owns CWE-295/319/732 — removed 6 arg_value_contains: Some(...) rule entries"
  - "D-13: CweRule struct reduced to 4 fields; paren_args_contain_all deleted as dead code"
  - "Rule-count test renamed test_rule_table_has_fourteen_cwes; assert_eq updated from 17 to 14 distinct CWE IDs"

patterns-established:
  - "After AST migration: lexical scanner is name-match-only for a CWE once AST owns it — no arg-value checks in scan_file"

requirements-completed: [ARGVAL-01, ARGVAL-02]

# Metrics
duration: 15min
completed: 2026-05-12
---

# Phase 20 Plan 02: Lexical Arg-Value Cleanup Summary

**CweRule struct stripped to 4 fields; 6 arg_value_contains: Some(...) entries and paren_args_contain_all deleted; AST scanner is sole authority for CWE-295/319/732 (D-12, D-13)**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 2 (committed atomically as one refactor commit)
- **Files modified:** 1

## Accomplishments

- Removed `arg_value_contains: Option<&'static [&'static str]>` field from `CweRule` struct (D-13)
- Deleted all 6 rule entries with `arg_value_contains: Some(...)`: CWE-295 (1 entry), CWE-319 (3 entries), CWE-732 (2 entries) (D-12)
- Removed `arg_value_contains: None` field from all 15 remaining `CweRule` initializers
- Deleted `paren_args_contain_all` function (no longer needed; dead code after rule removal)
- Deleted `if let Some(tokens) = rule.arg_value_contains { ... }` block in `scan_file`
- Deleted 8 lexical test functions for CWE-295/319/732 (their role taken over by 11 AST tests from Plan 01)
- Updated rule-count test: renamed to `test_rule_table_has_fourteen_cwes`, `assert_eq` updated from 17 → 14 distinct CWE IDs
- Full test suite green: 373 passed, 0 failed under `cargo test --features internal`
- All 11 Plan 01 AST argval tests still pass after cleanup

## Task Commits

1. **Tasks 1+2: Remove arg_value_contains field/rules + delete lexical CWE-295/319/732 tests** - `50fc6cf` (refactor)

**Plan metadata:** committed separately after SUMMARY.md creation

## Files Created/Modified

- `src/vulnerability/cwe_scanner.rs` — CweRule struct reduced to 4 fields; 6 Some(...) rule entries removed; paren_args_contain_all deleted; scan_file arg_value_contains check removed; 8 lexical argval tests deleted; rule-count test renamed and updated to 14

## Decisions Made

- D-12 implemented: lexical CWE-295/319/732 rules removed; AST scanner is sole authority for these CWEs after Plan 01 migration
- D-13 implemented: `arg_value_contains` field and `paren_args_contain_all` function deleted as dead code
- Rule-count test updated to reflect 14 distinct CWE IDs remaining in the lexical table (CWE-134 has 2 entries but 1 CWE ID)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. The changes were straightforward deletions with no unexpected compile errors or test failures.

## Known Stubs

None — all changes are clean deletions; no partial implementations or placeholder values introduced.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. The threat model's T-20-06 (accidental rule deletion) is mitigated by the passing rule-count test. T-20-05 (parse-fail coverage gap) is accepted per plan: lexical fallback on parse-fail files will no longer detect CWE-295/319/732, but parse failure is rare on well-formed C code and the wolfSSL gap was closed in Plan 01 (D-06).

## Self-Check: PASSED

Files exist:
- src/vulnerability/cwe_scanner.rs: FOUND

Commits exist:
- 50fc6cf: FOUND (verified via `git log --oneline`)

## Next Phase Readiness

Phase 20 is complete. Both plans executed:
- Plan 01: ArgAtIndex AST rules for CWE-295/319/732 with 11 AST tests
- Plan 02: Lexical scanner cleanup — arg_value_contains machinery removed

Remaining work: manual SARIF baseline diff on AUTOSAR_SampleProject_S32K144 to verify CWE-295/319/732 finding counts match or improve vs v1.0.17 baseline (ARGVAL-02 manual verification per VALIDATION.md). This is a post-execute verification, not a blocker.

---
*Phase: 20-argument-value-ast-migration*
*Completed: 2026-05-12*
