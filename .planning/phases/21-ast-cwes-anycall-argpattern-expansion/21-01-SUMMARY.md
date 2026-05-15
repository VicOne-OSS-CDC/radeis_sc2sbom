---
phase: 21
plan: 01
subsystem: vulnerability/ast_scanner
tags: [ast, tree-sitter, cwe, rust, tdd]
completed: "2026-05-12T09:50:00Z"
duration: "~30 minutes"

dependency_graph:
  requires: [phase-20]
  provides: [ArgCheck::SizeofPointer, apply_division_rules, collect_function_scope_pointer_declarators, collect_file_scope_pointer_declarators]
  affects: [src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs]

tech_stack:
  added: []
  patterns: [TDD RED/GREEN, fresh-cursor-per-level (Pitfall 1), file-scope pre-collection (Pitfall 6)]

key_files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

decisions:
  - D-01: CWE-369 via apply_division_rules() binary_expression walk (literal /0 or %0)
  - D-02: apply_division_rules() called alongside apply_ast_rules() in scan_file_ast_or_lexical
  - D-03: Lexical CWE-369 gate in cwe_scanner.rs preserved untouched
  - D-04: ArgCheck::SizeofPointer variant added for sizeof(ptr) detection
  - D-11: SizeofPointer arm consults function-scope and file-scope pointer-declarator sets

metrics:
  completed: "2026-05-12"
  tasks_completed: 3
  files_modified: 2
---

# Phase 21 Plan 01: AST Scanner Infrastructure (SizeofPointer + Division Rules) Summary

Added two pieces of AST scanner infrastructure required before Plan 02 can add 12 new rule-table entries: `ArgCheck::SizeofPointer` variant for CWE-467 pointer sizeof detection, and `apply_division_rules()` for CWE-369 literal divide-by-zero detection via binary_expression walk.

## What Was Built

### Task 0: Preflight
Verified Phase 20 dependency (`ArgCheck::ArgAtIndex` present, `ContainsTokens` deleted), test helpers (`setup_one_file`, `run_ast_scanner`) confirmed in place. No file changes; guard task confirmed safe to proceed.

### Task 1: ArgCheck::SizeofPointer + Pointer-Declarator Collectors

**Files modified:** `src/vulnerability/ast_scanner.rs`, `tests/vulnerability_tests/ast_scanner_tests.rs`

- Added `ArgCheck::SizeofPointer` variant to the `ArgCheck` enum (line 46)
- Added `sizeof_inner_identifier()` helper to unwrap parenthesized sizeof expressions (tree-sitter-c represents `sizeof(x)` as `sizeof_expression → parenthesized_expression → identifier`)
- Added `SizeofPointer` match arm in `visit_node` that fires when any arg is `sizeof(ident)` and `ident` is a pointer-typed variable in function or file scope
- Added collector helpers: `collect_function_scope_pointer_declarators`, `collect_file_scope_pointer_declarators`, `collect_ptrs_in_subtree`, `collect_file_scope_ptrs_rec`, `collect_pointer_declarators`, `collect_pointer_declarator_rec`, `extract_ident_under`
- Updated `apply_ast_rules` to pre-collect `file_scope_pointers` once per file (Pitfall 6 prevention)
- Updated `visit_node` signature to thread `file_scope_pointers: &HashSet<String>` through all recursive calls
- Added temporary CWE-467 rule table entry (malloc/calloc/realloc/memcpy/memset/memmove) to make TDD test green
- Updated module doc header to list 25-CWE coverage
- Added `test_cwe_467_sizeof_pointer` (TP: `sizeof(char*)` fires; FP: `sizeof(*p)` does not)

**Key deviation from plan design:** tree-sitter-c wraps `sizeof(x)` as `sizeof_expression → parenthesized_expression → identifier`, not `sizeof_expression → identifier`. Added `sizeof_inner_identifier()` helper to handle this. Without it, the `SizeofPointer` arm never matched.

**Key bug fixed in pointer collection:** Original `collect_pointer_declarator_rec` skipped `pointer_declarator` children when iterating siblings (used `if child.kind() != "pointer_declarator"` for recursion). Fixed to return early after `extract_ident_under` on match, and recurse into ALL non-pointer_declarator children.

### Task 2: apply_division_rules() + CWE-369 Unit Test

**Files modified:** `src/vulnerability/ast_scanner.rs`, `tests/vulnerability_tests/ast_scanner_tests.rs`

- Added `apply_division_rules(root, src, path, component_name, component_ecosystem, findings: &mut Vec<SastFinding>)` - calls `visit_binary_exprs`
- Added `visit_binary_exprs` - recursive walk matching `binary_expression` nodes where: operator is `/` or `%` (via `child(1)`), and RHS is `number_literal` or `integer_literal` (both accepted for grammar version variance), with text exactly `"0"`
- Modified `scan_file_ast_or_lexical` tail to call both `apply_ast_rules` and `apply_division_rules`, merging findings into a single Vec
- `cwe_scanner.rs` untouched (D-03 verified via `git diff`)
- Added `test_cwe_369_division_literal_zero` with: TP `x/0`, TP `x%0`, FP guard `x/10`, FP guard `x/0.0`

## Commits

| Task | Commit | Message |
|------|--------|---------|
| Task 1 | `81109d5` | feat(21-01): add ArgCheck::SizeofPointer variant, arm, and pointer-scope collectors |
| Task 2 | `5e91f1c` | feat(21-01): add apply_division_rules() helper for CWE-369 literal divide-by-zero |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tree-sitter-c sizeof(x) structure differs from plan assumption**
- **Found during:** Task 1 GREEN phase debugging
- **Issue:** Plan assumed `sizeof_expression` has a direct `identifier` named child. Actual grammar: `sizeof(x)` produces `sizeof_expression → sizeof (unnamed) + parenthesized_expression → identifier`. The `SizeofPointer` arm returned false for all sizeof args.
- **Fix:** Added `sizeof_inner_identifier()` helper that unwraps `parenthesized_expression` to find the inner identifier.
- **Files modified:** `src/vulnerability/ast_scanner.rs`
- **Commit:** `81109d5`

**2. [Rule 1 - Bug] collect_pointer_declarator_rec skipped pointer_declarator subtrees**
- **Found during:** Task 1 debug test (ptr names found: {})
- **Issue:** `collect_pointer_declarator_rec` used `if child.kind() != "pointer_declarator"` guard for all recursive calls, which prevented visiting `pointer_declarator` nodes found as children of `init_declarator`. Since `init_declarator` contains `pointer_declarator` as a child, the check blocked the descent.
- **Fix:** Changed to return early after `extract_ident_under` on `pointer_declarator` match; recurse into all children unconditionally for non-pointer_declarator nodes.
- **Files modified:** `src/vulnerability/ast_scanner.rs`
- **Commit:** `81109d5`

## Known Stubs

None — all infrastructure is wired and functional. CWE-467 test passes end-to-end.

## Threat Flags

No new threat surface introduced. All changes are analysis-only; no new network, file, or auth paths.

## TDD Gate Compliance

- RED gate: Both tests confirmed failing before implementation
- GREEN gate: Both tests pass after implementation
- No REFACTOR phase needed

## Hand-off to Plan 02

Plan 02 receives:
- `ArgCheck::SizeofPointer` fully implemented and tested
- `apply_division_rules()` fully implemented and tested
- CWE-467 rule table entry already present (Plan 02 should leave it in place)
- 11 more rule table entries to add: CWE-121, CWE-126, CWE-328 (×3 entries), CWE-338, CWE-426, CWE-526, CWE-535, CWE-676, CWE-680, CWE-780 (×2 entries)
- 10 more unit tests from 21-PATTERNS.md Wave 0 fixtures to add in Plan 02

## Self-Check: PASSED

- [x] `src/vulnerability/ast_scanner.rs` exists and modified
- [x] `tests/vulnerability_tests/ast_scanner_tests.rs` exists and modified
- [x] Commit `81109d5` exists (Task 1)
- [x] Commit `5e91f1c` exists (Task 2)
- [x] `cargo build --features internal` exits 0
- [x] `test_cwe_467_sizeof_pointer` passes
- [x] `test_cwe_369_division_literal_zero` passes
- [x] Full `cargo test --features internal` suite: all pass, no regressions
- [x] `git diff src/vulnerability/cwe_scanner.rs` is empty (D-03)
- [x] `grep -c "ContainsTokens" src/vulnerability/ast_scanner.rs` = 0 (Phase 20 cleanup preserved)
