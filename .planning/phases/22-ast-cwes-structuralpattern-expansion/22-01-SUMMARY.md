---
phase: 22-ast-cwes-structuralpattern-expansion
plan: 01
subsystem: sast
tags: [rust, tree-sitter, sast, cwe, ast-scanner]

# Dependency graph
requires:
  - phase: 21-ast-cwes-anycall-argpattern-expansion
    provides: ast_scanner.rs with apply_ast_rules, visitor pattern, SastFinding infrastructure
provides:
  - "5 new check_* structural visitor functions in ast_scanner.rs (CWE-478, 484, 481, 482, 480, 483)"
  - "9 unit tests in ast_scanner_tests.rs (TP + TN per CWE)"
  - "check_switch_structure: switch without default (CWE-478) and case fall-through (CWE-484)"
  - "check_block_delimitation: braceless if (CWE-483)"
  - "check_assignment_in_condition: assignment in if/while/for condition (CWE-481)"
  - "check_comparison_at_statement: statement-level == comparison (CWE-482)"
  - "check_func_ptr_null_compare: identifier == null/0 in if condition (CWE-480)"
affects: [22-02, 22-03, phase-23]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Structural visitor pattern: check_*(node, src, path, name, ecosystem, findings) with fresh cursor per level"
    - "Default case detection: check case_statement with absent value field (no default_case node in tree-sitter-c)"
    - "Parenthesized expression unwrapping: unwrap_parens() helper for condition field access"
    - "Named children materialization: collect into Vec<Node> to avoid cursor borrow issues"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "tree-sitter-c uses case_statement for BOTH 'case X:' and 'default:' — default variant has no value field (no default_case node kind exists)"
  - "Named children must be materialized into Vec<Node> before use to avoid cursor lifetime borrow conflicts in Rust"
  - "unwrap_parens helper avoids duplicating parenthesized_expression unwrapping logic across check_assignment_in_condition and check_func_ptr_null_compare"
  - "CWE-481 Pitfall 5: only fire when condition field's direct inner node is assignment_expression (not nested inside binary_expression)"

patterns-established:
  - "Pattern: structural visitor with &mut Vec<SastFinding> accumulator and fresh cursor per recursion level"
  - "Pattern: body field access via child_by_field_name('body') on switch_statement"

requirements-completed: [CWEXP-02]

# Metrics
duration: 45min
completed: 2026-05-12
---

# Phase 22 Plan 01: Structural-Pattern CWE Expansion (Group A) Summary

**5 structural visitor functions added to ast_scanner.rs detecting CWE-478/484 (switch), CWE-481/482/480 (operator errors), and CWE-483 (block delimitation) via pure tree-sitter node shape inspection**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 3 (TDD: RED + GREEN per task)
- **Files modified:** 2

## Accomplishments

- Added 9 unit tests (TP + TN for each CWE) — all RED before implementation, all GREEN after
- Implemented `check_switch_structure` covering CWE-478 (no default) and CWE-484 (case fall-through)
- Implemented `check_block_delimitation` for CWE-483 (braceless if)
- Implemented `check_assignment_in_condition` (CWE-481), `check_comparison_at_statement` (CWE-482), `check_func_ptr_null_compare` (CWE-480)
- Pitfall 5 TN verified: nested `(c = 1) != 0` does NOT fire CWE-481

## Task Commits

1. **Task 1: Failing unit tests for 6 CWEs** - `1d0b309` (test)
2. **Task 2: check_switch_structure + check_block_delimitation** - `df7409c` (feat)
3. **Task 3: check_assignment_in_condition, check_comparison_at_statement, check_func_ptr_null_compare** - `61de663` (feat)

## Files Created/Modified

- `src/vulnerability/ast_scanner.rs` — 5 new check_* private functions (~188 lines added), 3 apply_ast_rules call sites, `unwrap_parens` and `is_identifier_vs_null` helpers
- `tests/vulnerability_tests/ast_scanner_tests.rs` — 9 new test functions appended (111 lines)

## Tree-sitter Node-Kind Assumptions Verified

| Assumption | Verified Result |
|------------|----------------|
| switch_statement body accessed via `child_by_field_name("body")` | CONFIRMED |
| `default:` case represented as `default_case` node | WRONG — it IS a `case_statement` with no `value` field |
| `case_statement` named children: [value_literal, statement...] | CONFIRMED |
| `if_statement` consequence via `child_by_field_name("consequence")` | CONFIRMED |
| `if_statement` condition via `child_by_field_name("condition")` returns `parenthesized_expression` | CONFIRMED |
| `for_statement` condition via `child_by_field_name("condition")` | Not tested in Plan 01 |

**Critical discovery:** tree-sitter-c v0.23 does NOT have a `default_case` node kind. The `default:` label in a switch is parsed as a `case_statement` with the `value` named field absent. The CWE-478 check detects `default:` by checking `child_by_field_name("value").is_none()` on `case_statement` nodes.

## Known FP Risks

- **CWE-483 (Pitfall 6):** High FP rate expected in real codebases — braceless single-line `if` is a common style. Accepted per D-11 (`--sarif-baseline` for user suppression).
- **CWE-480:** May fire on non-function-pointer identifiers compared to 0/NULL (e.g., error code checks). Narrow form accepted per research guidance.
- **CWE-482:** May fire on intentionally-discarded comparison in obfuscated code, though rare.

## Decisions Made

- Detected default case by `case_statement` with absent `value` field (not `default_case` node kind), after discovering the node kind assumption was wrong via AST dump diagnostic.
- Used `Vec<Node>` materialization pattern to avoid Rust cursor borrow issues — pattern established for all subsequent Phase 22 plans.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Default case detection used wrong node kind**
- **Found during:** Task 2 (check_switch_structure implementation)
- **Issue:** Plan specified looking for `default_case` node kind, but tree-sitter-c uses `case_statement` for default (no `value` field). Rule would produce FP on all switch-with-default code.
- **Fix:** Changed detection to `case_statement` with `child_by_field_name("value").is_none()`
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Verification:** TN test `test_cwe478_switch_with_default_no_finding` passed after fix
- **Committed in:** df7409c

**2. [Rule 1 - Bug] Rust cursor borrow lifetime conflicts in named_children()**
- **Found during:** Task 3 (check_comparison_at_statement, unwrap_parens)
- **Issue:** `node.named_children(&mut cursor).next()` caused "borrowed value does not live long enough" compile errors when cursor was declared inside an `if let` block
- **Fix:** Materialized named_children into `Vec<Node>` before use for both `unwrap_parens` and `check_comparison_at_statement`
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Verification:** Compiler error resolved; tests pass
- **Committed in:** 61de663

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bugs)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered

- A pre-existing test `format_tests::spdx_validation_tests::test_spdx_output_passes_pyspdxtools_validation` fails in the worktree environment (requires `example_target_repos/rclcpp` directory, not related to our changes). All other 393 tests pass.

## Known Stubs

None — all 5 functions are fully implemented and emit real findings.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All code is read-only static analysis inside the `#[cfg(feature = "internal")]` gate.

## Self-Check: PASSED

- FOUND: src/vulnerability/ast_scanner.rs
- FOUND: tests/vulnerability_tests/ast_scanner_tests.rs
- FOUND: .planning/phases/22-ast-cwes-structuralpattern-expansion/22-01-SUMMARY.md
- FOUND commit 1d0b309 (test: failing tests)
- FOUND commit df7409c (feat: check_switch_structure + check_block_delimitation)
- FOUND commit 61de663 (feat: check_assignment_in_condition, check_comparison_at_statement, check_func_ptr_null_compare)

## Next Phase Readiness

- Plan 01 Group A (switch/operator/block) complete. All 6 CWEs have TP+TN unit test coverage.
- Ready for Plan 02: subtree name collection CWEs (CWE-562, 570, 571, 587, 256, 398)
- Pattern established: structural visitor + `Vec<Node>` materialization + `unwrap_parens` helper

---
*Phase: 22-ast-cwes-structuralpattern-expansion*
*Completed: 2026-05-12*
