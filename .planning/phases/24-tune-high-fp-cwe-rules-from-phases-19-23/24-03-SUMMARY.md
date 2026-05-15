---
phase: 24-tune-high-fp-cwe-rules-from-phases-19-23
plan: "03"
subsystem: vulnerability/ast-scanner
tags: [cwe, rust, ast-scanner, structural-guards, phase-24, cwe-478, cwe-480, cwe-483, cwe-562, cwe-570, cwe-571, cwe-587, cwe-762]
dependency_graph:
  requires: [24-02]
  provides: [guarded-CWE-478, guarded-CWE-480, guarded-CWE-483, guarded-CWE-562, narrowed-CWE-570, narrowed-CWE-571, cooccurrence-CWE-762, investigated-CWE-587]
  affects: [src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs]
tech_stack:
  added: []
  patterns: [structural-visitor-guard, early-return-co-occurrence, tdd-red-green]
key_files:
  created:
    - .planning/phases/24-tune-high-fp-cwe-rules-from-phases-19-23/24-03-CWE587-INVESTIGATION.md
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs
decisions:
  - "CWE-478 (D-19): guard fires only when >2 non-default cases in switch; 1-2 case switches are common idioms"
  - "CWE-483 (D-13): braceless if with single return/break/continue is acceptable; excluded from firing"
  - "CWE-570/571 (D-15/D-16): check_constant_condition narrowed to if_statement only; while/for loop constants are not constant-condition bugs"
  - "CWE-762 (D-20): co-occurrence guard — early return if no calloc/malloc/realloc found in file using token_present_with_boundary"
  - "CWE-480 (D-12): fire only when compared identifier is declared as a function pointer in enclosing function body (declaration text contains ident name and '(')"
  - "CWE-562 (D-14): collect_local_vars_in_subtree now skips array_declarator and struct/union_specifier locals; only scalars enter candidate set"
  - "CWE-587 (D-18): threshold val > 0xFFFF already present at line 1072 — D-18 is a confirmed no-op; investigation file documents root cause hypotheses"
metrics:
  duration: "~7 minutes"
  completed: "2026-05-13"
  tasks_completed: 2
  files_modified: 2
  files_created: 1
---

# Phase 24 Plan 03: Structural Guards for CWE-478/480/483/562/570/571/762 + CWE-587 Investigation Summary

Seven structural guards applied to six `check_*` functions and `apply_delete_rules`, plus a CWE-587 investigation confirming D-18 is a no-op with `val > 0xFFFF` already in place.

## Tasks Completed

| Task | Name | Commits | Files |
|------|------|---------|-------|
| 1 | Guards for CWE-478/483/570/571/762 | c4b9a64 (RED), 93b24a6 (GREEN) | src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs |
| 2 | Guards for CWE-480/562 + CWE-587 investigation | 9f589d9 (RED), 5e3f0db (GREEN) | src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs, 24-03-CWE587-INVESTIGATION.md |

## What Was Built

### Task 1: Four guards

**CWE-478 (D-19):** `non_default_case_count > 2` guard in `check_switch_structure`. Switches with 1–2 non-default cases no longer fire. The existing TP test was updated to use a 3-case fixture.

**CWE-483 (D-13):** `return_statement`, `break_statement`, `continue_statement` exclusions in `check_block_delimitation`. Braceless if with single control-flow statement is idiomatic C and not a CWE-483 bug.

**CWE-570/571 (D-15/D-16):** `check_constant_condition` narrowed from `matches!(node.kind(), "if_statement" | "while_statement" | "for_statement")` to `if node.kind() == "if_statement"`. `while(0)` and `while(1)` are common idioms (infinite loops handled by `check_infinite_loop` CWE-835).

**CWE-762 (D-20):** Co-occurrence pre-scan in `apply_delete_rules`. Uses existing `token_present_with_boundary` to check for `calloc`/`malloc`/`realloc` in the file. Returns empty Vec immediately if none found. Eliminates FPs in C++-only files that use `delete` without C allocation.

### Task 2: Two guards + investigation

**CWE-480 (D-12):** `is_func_ptr` check in `check_func_ptr_null_compare`. Walks the enclosing function's body (`compound_statement`) for `declaration` nodes whose text contains both the identifier name and `(`. Only fires if such a declaration exists. The TP test was updated from an unrelated function-comparison fixture to a proper function-pointer declaration fixture.

**CWE-562 (D-14):** Array/struct/union filter in `collect_local_vars_in_subtree`. Checks direct children of each `declaration` node for `array_declarator`, `struct_specifier`, or `union_specifier`. Skips them from the scalar candidate set. The TP test was updated from `return buf` (array, now excluded) to `return x` (scalar, still fires).

**CWE-587 (D-18):** Investigation confirmed `is_large_hex_literal()` at line 1072 already uses `val > 0xFFFF`. No code change required. Investigation file written to `.planning/phases/24-tune-high-fp-cwe-rules-from-phases-19-23/24-03-CWE587-INVESTIGATION.md`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated test_cwe478_switch_without_default to match new >2 threshold**
- **Found during:** Task 1 GREEN phase full-suite run
- **Issue:** Existing TP test used a 1-case switch (`case 1: break;`) which now falls below the >2 threshold and no longer fires
- **Fix:** Updated fixture to 3-case switch (`case 1`, `case 2`, `case 3`) matching the plan's stated regression behavior
- **Files modified:** `tests/vulnerability_tests/ast_scanner_tests.rs`
- **Commit:** 93b24a6

**2. [Rule 1 - Bug] CWE-480 guard used function_definition named_children instead of body children**
- **Found during:** Task 2 GREEN phase full-suite run
- **Issue:** `fn_node.named_children()` iterates direct children of `function_definition` (type, declarator, body) — declarations live inside the body, not directly on the fn node
- **Fix:** Changed `fn_node.named_children()` to `fn_node.child_by_field_name("body").named_children()` via `.and_then()`
- **Files modified:** `src/vulnerability/ast_scanner.rs`
- **Commit:** 5e3f0db

**3. [Rule 1 - Bug] Updated test_cwe480_func_ptr_compared_to_null to use function-pointer fixture**
- **Found during:** Task 2 GREEN phase full-suite run  
- **Issue:** Original fixture `int g(){} void f(){if(g == 0){}}` compared a function (not a declared function pointer) to 0. After D-12, this no longer fires (no `declaration` inside `f`'s body contains `g` with `(`)
- **Fix:** Updated to `void f(){void (*fp)(int) = 0; if(fp == 0){}}` — a proper function-pointer declaration inside the enclosing function
- **Files modified:** `tests/vulnerability_tests/ast_scanner_tests.rs`
- **Commit:** 5e3f0db

**4. [Rule 1 - Bug] Updated test_cwe562_return_local_array to use scalar return fixture**
- **Found during:** Task 2 GREEN phase full-suite run
- **Issue:** Original TP fixture `char buf[16]; return buf;` returned a local array; after D-14, array locals are excluded from the candidate set, so this no longer fires
- **Fix:** Updated to `int x = 0; return x;` — a scalar local variable, which still enters the candidate set
- **Files modified:** `tests/vulnerability_tests/ast_scanner_tests.rs`
- **Commit:** 5e3f0db

**5. [Rule 1 - Bug] Rust lifetime error in has_array_init_decl closure**
- **Found during:** Task 2 first build attempt
- **Issue:** Rust rejected `c.named_children(&mut ic).any(...)` as last expression in closure block — temporary borrow outlives block
- **Fix:** Saved to intermediate `let has_arr = ...` variable before returning from closure
- **Files modified:** `src/vulnerability/ast_scanner.rs`
- **Commit:** 5e3f0db

## Verification Results

- `cargo build --features internal` exits 0
- `cargo test --features internal`: 444 passed, 1 pre-existing failure (`test_spdx_output_passes_pyspdxtools_validation` — missing binary fixture, unrelated to this plan)
- All 7 new `phase_24_cwe_*` tests pass
- CWE-835 tests still pass (overlap with CWE-571 cleanly separated)
- Both `test_cwe587_*` tests pass (D-18 no-op confirmed)
- CWE-587 investigation file written and committed

## Known Stubs

None — no stub patterns found in modified files.

## Threat Flags

None — all changes are within existing trust boundaries. The `apply_delete_rules` co-occurrence scan is a single linear pass over `src` via `token_present_with_boundary`, satisfying T-24-08. The `check_func_ptr_null_compare` declaration walk is bounded by the enclosing function body, satisfying T-24-07.

## Self-Check: PASSED

- `src/vulnerability/ast_scanner.rs` exists: FOUND
- `tests/vulnerability_tests/ast_scanner_tests.rs` exists: FOUND
- `.planning/phases/24-tune-high-fp-cwe-rules-from-phases-19-23/24-03-CWE587-INVESTIGATION.md` exists: FOUND
- Commit c4b9a64 (RED Task 1) in git log: FOUND
- Commit 93b24a6 (GREEN Task 1) in git log: FOUND
- Commit 9f589d9 (RED Task 2) in git log: FOUND
- Commit 5e3f0db (GREEN Task 2) in git log: FOUND
- All 7 phase_24_cwe_* tests present in test file: VERIFIED
- `cargo build --features internal`: PASSED
- 444 tests pass (1 pre-existing failure): CONFIRMED
