---
phase: 24-tune-high-fp-cwe-rules-from-phases-19-23
plan: "02"
subsystem: vulnerability/ast-scanner
tags: [cwe, rust, ast-scanner, argcheck, tightening, phase-24, cwe-126, cwe-467, cwe-535, cwe-680]
dependency_graph:
  requires: [24-01]
  provides: [tightened-CWE-126, tightened-CWE-467, tightened-CWE-535, tightened-CWE-680]
  affects: [src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs]
tech_stack:
  added: []
  patterns: [new-argcheck-variants, tdd-red-green, dedicated-visitor-function, struct-ptr-exclusion]
key_files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs
decisions:
  - "CWE-126 split: strcat uses FixedSizeBuffer; strncat uses new FixedSizeBufferWithoutSizeArg(2) (D-09)"
  - "CWE-680 switched from AnyCall to new SizeArgIsMultiplication(0): fires only on malloc(n*sizeof(T)) (D-10)"
  - "CWE-467: collect_pointer_declarators skips struct/union pointer declarations (D-11)"
  - "CWE-535: migrated from ArgAtIndex(0, stderr) table entry to dedicated check_stderr_format_string() visitor that requires arg1 to be non-literal (D-17)"
  - "test_cwe_535_shell_error_stderr updated to use non-literal format string fixture to match new semantics"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-13"
  tasks: 2
  files_modified: 2
---

# Phase 24 Plan 02: ArgCheck Variants + CWE-126/467/535/680 Tightening Summary

**One-liner:** Two new ArgCheck variants (`FixedSizeBufferWithoutSizeArg`, `SizeArgIsMultiplication`) plus struct-ptr exclusion in CWE-467 and a dedicated `check_stderr_format_string()` visitor for CWE-535 replacing the over-broad `ArgAtIndex(0, stderr)` rule.

## What Was Built

### Task 1: ArgCheck Variants + CWE-126/680 Table Changes

**Two new ArgCheck variants** added to the `ArgCheck` enum (7 total, up from 5):

- `FixedSizeBufferWithoutSizeArg(u8)` — fires when dest arg (index 0) is a fixed-size array AND the size arg at the given index is NOT a `sizeof_expression`. Used by CWE-126 `strncat` entry.
- `SizeArgIsMultiplication(u8)` — fires when the size arg at the given index is a `binary_expression` with operator `*`. Used by CWE-680.

**CWE-126 split** (was one combined entry for `["strcat", "strncat"]`):
- `strcat` keeps `ArgCheck::FixedSizeBuffer` (no size arg to check)
- `strncat` uses `ArgCheck::FixedSizeBufferWithoutSizeArg(2)` (size arg is index 2)

**CWE-680** switched from `AnyCall` to `SizeArgIsMultiplication(0)`:
- Before: any `malloc`/`calloc`/`realloc` call fired
- After: only fires when arg 0 is `n * sizeof(T)` or similar multiplication expression

**Match arms** added in `visit_node()` for both new variants.

### Task 2: CWE-467 Struct-Ptr Exclusion + CWE-535 Dedicated Visitor

**CWE-467 (`collect_pointer_declarators`)**: Added struct/union specifier check — declarations whose type contains `struct_specifier` or `union_specifier` are skipped. `sizeof(struct_ptr)` is intentional usage in embedded code, not a CWE-467 bug (D-11).

**CWE-535 migration**: Removed the `ArgAtIndex(0, &["stderr"])` entry from `AST_CWE_RULES`. Added `check_stderr_format_string()` dedicated visitor that fires only when:
1. Arg 0 is the identifier `stderr`
2. Arg 1 is NOT a `string_literal` node

This eliminates the FP on `fprintf(stderr, "literal\n", x)` while preserving the TP on `fprintf(stderr, fmt, x)`.

## Commits

| Hash | Type | Description |
|------|------|-------------|
| b58e76f | test | RED: failing tests for CWE-126 strncat sizeof guard + CWE-680 multiplication guard |
| d79fa25 | feat | GREEN: ArgCheck variants + CWE-126/680 table changes + visit_node match arms |
| 36e129b | test | RED: failing tests for CWE-467 struct-ptr + CWE-535 literal fmt guard |
| b072de7 | feat | GREEN: CWE-467 struct-ptr exclusion + CWE-535 dedicated visitor |

## Test Results

All 7 new phase_24 tests pass:
- `phase_24_cwe_126_strncat_nonsizeof_fires` — strncat with variable size arg fires CWE-126
- `phase_24_cwe_126_strncat_sizeof_no_fire` — strncat with sizeof(buf) does NOT fire CWE-126
- `phase_24_cwe_680_multiplication_fires` — malloc(n * sizeof(int)) fires CWE-680
- `phase_24_cwe_680_sizeof_only_no_fire` — malloc(sizeof(int)) does NOT fire CWE-680
- `phase_24_cwe_467_struct_ptr_no_fire` — sizeof(struct_ptr) does NOT fire CWE-467
- `phase_24_cwe_535_literal_fmt_no_fire` — fprintf(stderr, "literal") does NOT fire CWE-535
- `phase_24_cwe_535_nonliteral_fmt_fires` — fprintf(stderr, fmt) fires CWE-535

Full suite: 437 passed, 1 pre-existing failure (test_spdx_output_passes_pyspdxtools_validation — binary execution test, unrelated to this plan).

## Deviations from Plan

None — plan executed exactly as written. The CWE-467 struct-ptr test (`phase_24_cwe_467_struct_ptr_no_fire`) was already passing before the `collect_pointer_declarators` fix because function parameters (`struct Foo *sp`) are represented as `parameter_declaration` nodes in tree-sitter, not `declaration` nodes, so they were never collected by `collect_ptrs_in_subtree`. The fix is defensive and correctly handles file-scope struct pointer declarations.

## Known Stubs

None.

## Threat Flags

None — changes are entirely within the existing trust boundary. The new visitor `check_stderr_format_string()` uses the same fresh-cursor recursion pattern as `check_block_delimitation()`, satisfying T-24-05 (bounded by AST depth).

## Self-Check: PASSED

- `src/vulnerability/ast_scanner.rs` — modified, exists
- `tests/vulnerability_tests/ast_scanner_tests.rs` — modified, exists
- Commits b58e76f, d79fa25, 36e129b, b072de7 — all present in git log
- `cargo build --features internal` exits 0
- All 7 phase_24 tests pass
