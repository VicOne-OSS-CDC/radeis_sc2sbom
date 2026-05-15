---
phase: 23-ast-cwes-domainspecific-expansion
plan: 02
subsystem: vulnerability-scanning
tags: [rust, tree-sitter, ast-scanner, cwe, sast, signal-handler, paired-lock, delete-scan]

# Dependency graph
requires:
  - phase: 23-ast-cwes-domainspecific-expansion
    plan: 01
    provides: 5 table-driven CWEs (114, 272, 284, 427, 785) in AST_CWE_RULES

provides:
  - apply_signal_handler_rules(): CWE-479 two-pass signal handler detection
  - apply_paired_lock_rules(): CWE-591 VirtualAlloc/VirtualLock paired check
  - apply_delete_rules(): CWE-762 text-level delete token scan
  - NON_REENTRANT constant (11 non-reentrant functions)
  - tests/fixtures/c/cwe762_delete_bad.c synthetic fixture
  - 5 unit tests: 2 CWE-479, 2 CWE-591, 1 CWE-762

affects:
  - scan_file_ast_or_lexical() extended with 3 new helpers via findings.extend()
  - Total phase_23 test count: 11 (6 from Plan 01 + 5 from Plan 02)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-pass AST walk with HashMap<String, u32> for signal handler line tracking (CWE-479)"
    - "Function-scope paired-call check via HashSet accumulation (CWE-591)"
    - "Byte-level state-machine comment-aware token scan (CWE-762)"

key-files:
  created:
    - tests/fixtures/c/cwe762_delete_bad.c
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "apply_delete_rules uses text-level byte scan rather than AST (delete is C++ operator, not call_expression in tree-sitter-c)"
  - "CWE-479 finding emitted at signal() call site line, not at non-reentrant call line inside handler (Pitfall 3)"
  - "CWE-591 walk limited to root-level function_definition nodes (single-file scope per D-04)"
  - "apply_delete_rules comment-skip state machine avoids false positives from delete in comments"

requirements-completed: [CWEXP-03]

# Metrics
duration: 20min
completed: 2026-05-12
---

# Phase 23 Plan 02: ast-cwes-domainspecific-expansion Summary

**Three structural helpers (CWE-479 two-pass signal handler, CWE-591 VirtualAlloc/VirtualLock paired check, CWE-762 delete text scan) implemented and wired into scan_file_ast_or_lexical(); 5 new tests all pass; 11 total phase_23 tests green**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-05-12T15:00:00Z
- **Completed:** 2026-05-12T15:16:50Z
- **Tasks:** 3
- **Files modified:** 2 (ast_scanner.rs, ast_scanner_tests.rs) + 1 created (cwe762_delete_bad.c)

## Accomplishments

- Created `tests/fixtures/c/cwe762_delete_bad.c`: namespace-free synthetic fixture with `calloc()+delete` mismatch that tree-sitter-c parses without `has_error()`
- Added `NON_REENTRANT` constant (malloc, free, printf, fprintf, sprintf, snprintf, vprintf, vfprintf, exit, abort, syslog)
- Implemented `apply_signal_handler_rules()`: two-pass CWE-479 detection. Pass 1 collects `(handler_name → signal_call_line)` via HashMap. Pass 2 scans each handler's function body for NON_REENTRANT calls; emits finding at signal() call site line per Pitfall 3.
- Implemented `apply_paired_lock_rules()`: CWE-591 detection; walks root-level function_definition nodes, collects all call_expression names in the function body, fires if VirtualAlloc present without VirtualLock.
- Implemented `apply_delete_rules()`: CWE-762 text-level byte scan. State machine skips `//` line comments and `/* */` block comments; requires word-boundary match on `delete` token.
- Wired all three helpers into `scan_file_ast_or_lexical()` via `findings.extend(...)` chain.
- Added 5 new unit tests covering positive and negative cases for all three CWEs.
- All 11 phase_23 tests pass; 423 other tests pass (1 pre-existing pyspdxtools env failure unrelated).

## Task Commits

1. **Task 1: Add synthetic CWE-762 fixture file** - `d29abd6` (feat)
2. **Task 2: Implement three structural helpers and wire into scan_file_ast_or_lexical** - `30b797a` (feat)
3. **Task 3: Add unit tests for CWE-479, 591, 762** - `b7be06d` (test)

## Files Created/Modified

- `tests/fixtures/c/cwe762_delete_bad.c` - New namespace-free synthetic fixture: calloc()+delete mismatch
- `src/vulnerability/ast_scanner.rs` - NON_REENTRANT constant + 3 new helper functions + 3 private helpers + scan_file_ast_or_lexical wiring (267 lines added)
- `tests/vulnerability_tests/ast_scanner_tests.rs` - 5 new phase_23 test functions (65 lines added)

## Decisions Made

- Used text-level byte scan for CWE-762 instead of AST: `delete` is a C++ operator and never appears as `call_expression` in tree-sitter-c; AST walk would find 0 matches
- CWE-479 finding line uses Pass 1 signal_call_line, not Pass 2 non-reentrant call line — matches Juliet oracle and RESEARCH.md Pitfall 3
- CWE-591 scoped to root-level function_definition nodes only (no nested function definitions in C), consistent with check_self_recursion pattern
- `apply_delete_rules` _root parameter kept unused (prefixed with underscore) for signature uniformity with other helpers

## Deviations from Plan

None — plan executed exactly as written. The existing `apply_division_rules` already used a `&mut findings` parameter (not `Vec<SastFinding>` return), which was preserved. The three new Plan 02 helpers use `Vec<SastFinding>` return and are wired via `findings.extend(...)`.

## Issues Encountered

The pre-existing `test_spdx_output_passes_pyspdxtools_validation` failure continues (requires `pyspdxtools` binary in PATH, an environment issue present before these changes — confirmed in Plan 01 SUMMARY).

## Known Stubs

None — all three helpers fully implemented and wired.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. Rules are read-only byte/AST inspection of source files already scanned.

## Self-Check

- [x] `tests/fixtures/c/cwe762_delete_bad.c` exists and contains `delete p`
- [x] `src/vulnerability/ast_scanner.rs` contains `fn apply_signal_handler_rules` (1 match)
- [x] `src/vulnerability/ast_scanner.rs` contains `fn apply_paired_lock_rules` (1 match)
- [x] `src/vulnerability/ast_scanner.rs` contains `fn apply_delete_rules` (1 match)
- [x] `src/vulnerability/ast_scanner.rs` contains `const NON_REENTRANT` (1 match)
- [x] `src/vulnerability/ast_scanner.rs` contains `cwe_id: 479`, `cwe_id: 591`, `cwe_id: 762`
- [x] `tests/vulnerability_tests/ast_scanner_tests.rs` contains all 5 new test functions
- [x] Commit d29abd6 exists (Task 1)
- [x] Commit 30b797a exists (Task 2)
- [x] Commit b7be06d exists (Task 3)
- [x] All 11 phase_23 tests pass
- [x] cargo build --features internal exits 0

## Self-Check: PASSED

---
*Phase: 23-ast-cwes-domainspecific-expansion*
*Plan: 02*
*Completed: 2026-05-12*
