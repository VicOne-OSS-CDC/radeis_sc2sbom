---
phase: 22-ast-cwes-structuralpattern-expansion
plan: 02
subsystem: sast
tags: [rust, tree-sitter, sast, cwe, ast-scanner]

# Dependency graph
requires:
  - phase: 22-01
    provides: check_* structural visitor pattern, unwrap_parens helper, SastFinding infrastructure
provides:
  - "collect_local_var_names + collect_local_vars_in_subtree + collect_decl_identifiers helpers"
  - "check_return_stack_address: CWE-562 (return of local array identifier)"
  - "parse_c_integer_literal: hex/octal/decimal with suffix stripping"
  - "check_constant_condition: CWE-570 (always false) / CWE-571 (always true)"
  - "is_large_hex_literal + check_fixed_address_assignment: CWE-587"
  - "8 unit tests in ast_scanner_tests.rs (TP + TN for CWE-562, 570, 571, 587)"
affects: [22-03, phase-23]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "collect_local_var_names: entry fn + collect_local_vars_in_subtree recursive helper mirroring collect_function_scope_fixed_arrays"
    - "Vec<Node> materialization required for children() iterator borrow lifetime (same as Plan 01)"
    - "parse_c_integer_literal: strip u/U/l/L suffixes, handle 0x prefix, 0-prefix octal, decimal"
    - "Dynamic cwe_id: let cwe_id = if val == 0 { 570 } else { 571 } — not a literal field in struct init"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "collect_local_var_names skips declarations with storage_class_specifier 'static' or 'extern' — materializing children into Vec<Node> required to avoid borrow lifetime errors"
  - "Parameter removal from local_names: walk declarator -> parameter_list -> parameter_declaration; materialize at each level to avoid cursor conflicts"
  - "parenthesized_expression unwrapping confirmed required for CWE-570/571 condition field (tree-sitter-c wraps if/while/for conditions in parenthesized_expression)"
  - "Dynamic cwe_id variable used for CWE-570/571 — grep for literal 'cwe_id: 570' finds 0 results but logic is correct"
  - "CWE-587: cast_expression value field used to unwrap (char*)0x400000 — child_by_field_name('value') correctly navigates to the number_literal"
  - "is_large_hex_literal threshold: > 0xFFFF excludes (int*)0 and (void*)0x0000 null-pointer idioms"

requirements-completed: [CWEXP-02]

# Metrics
duration: ~5min
completed: 2026-05-12
---

# Phase 22 Plan 02: Structural-Pattern CWE Expansion (Group B) Summary

**3 new check_* functions + collect_local_var_names helper infrastructure detecting CWE-562 (return of stack address), CWE-570/571 (constant conditions), CWE-587 (fixed address assignment) — all via pure tree-sitter AST node inspection**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 3 (TDD: RED + GREEN per group)
- **Files modified:** 2

## Accomplishments

- Added 8 unit tests (TP + TN for CWE-562, 570, 571, 587) — all RED before implementation, all GREEN after
- Implemented `collect_local_var_names` (entry) + `collect_local_vars_in_subtree` (recursive) + `collect_decl_identifiers` (identifier extraction) helpers
- Implemented `check_return_stack_address` (CWE-562): collects local names once per function_definition, strips parameter names, fires on `return identifier` where identifier is in local_names
- Added `parse_c_integer_literal` utility: handles hex/octal/decimal with suffix stripping
- Implemented `check_constant_condition` (CWE-570/571): fires on `if/while/for` with single number_literal or `number_literal == number_literal` condition
- Implemented `is_large_hex_literal` + `check_fixed_address_assignment` (CWE-587): fires on init_declarator/assignment_expression whose cast_expression wraps a hex literal > 0xFFFF
- TN tests all pass: static/global array does NOT fire CWE-562; (int*)0 does NOT fire CWE-587

## Task Commits

1. **Task 1: Failing unit tests for CWE-562, 570, 571, 587 (RED)** - `25ab437`
2. **Task 2: collect_local_var_names + check_return_stack_address + check_constant_condition** - `77ef012`
3. **Task 3: is_large_hex_literal + check_fixed_address_assignment (CWE-587)** - `9534edc`

## Files Created/Modified

- `src/vulnerability/ast_scanner.rs` — 3 new check_* functions + 5 helpers (~330 lines added); 3 apply_ast_rules call sites
- `tests/vulnerability_tests/ast_scanner_tests.rs` — 8 new test functions appended (~98 lines)

## Tree-sitter Node-Kind Assumptions Verified

| Assumption | Verified Result |
|------------|----------------|
| `if_statement` condition field returns `parenthesized_expression` | CONFIRMED — `unwrap_parens` required for CWE-570/571 |
| `for_statement` condition via `child_by_field_name("condition")` | CONFIRMED (used in check_constant_condition) |
| `init_declarator` value field via `child_by_field_name("value")` | CONFIRMED |
| `cast_expression` value field via `child_by_field_name("value")` | CONFIRMED |
| `number_literal` is the node kind for numeric constants | CONFIRMED (also accepted `integer_literal` for grammar variance) |
| `return_statement` first named child is the return value | CONFIRMED |
| `declaration` children include `storage_class_specifier` with text "static" | CONFIRMED |

**Key discovery:** `parenthesized_expression` unwrapping is required for CWE-570/571 condition extraction. The condition field on `if_statement` is a `parenthesized_expression`, so `unwrap_parens()` (introduced in Plan 01) was essential here.

## Pitfall Encountered

**Borrow lifetime conflict in `node.children(&mut cur).any(...)`:** The `children()` iterator borrows the cursor; when called inline in a boolean expression, the compiler raises "borrowed value does not live long enough". Fix: materialize into `Vec<Node>` first. This is the same Rule 1 pattern from Plan 01 (Rust cursor borrow lifetime conflicts in named_children()).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Borrow lifetime conflict in collect_local_vars_in_subtree**
- **Found during:** Task 2 compilation
- **Issue:** `node.children(&mut cur).any(|c| c.kind() == "storage_class_specifier" && ...)` caused "borrowed value does not live long enough" compile error
- **Fix:** Materialized `node.children(&mut cur)` into `Vec<Node>` before calling `.iter().any(...)`
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Committed in:** 77ef012

**2. [Rule 1 - Bug] Redundant double-call in parameter removal code**
- **Found during:** Task 2 implementation
- **Issue:** Initial code had a redundant nested `collect_decl_identifiers` inside a `&mut { ... }` block — would compile but the outer call was unused
- **Fix:** Rewrote to use a clean `let mut param_names = HashSet::new(); collect_decl_identifiers(...); for n in &param_names { local_names.remove(n); }` pattern with materialized child Vecs
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Committed in:** 77ef012

---

**Total deviations:** 2 auto-fixed (2 Rule 1 bugs — same cursor borrow class as Plan 01)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered

- Pre-existing test `format_tests::spdx_validation_tests::test_spdx_output_passes_pyspdxtools_validation` continues to fail (requires `example_target_repos/rclcpp` directory, unrelated to our changes). All other 401 tests pass.

## Known Stubs

None — all 3 new check_* functions are fully implemented and emit real findings.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All code is read-only static analysis inside `#[cfg(feature = "internal")]`. Threat mitigations from plan threat model:
- T-22-04 (DoS via recursion): fresh cursor per level per Pitfall 1 — implemented
- T-22-05 (hex literal parse): `u64::from_str_radix` returns Result; failure silently treats as not-large — implemented

## Self-Check: PASSED

- FOUND: src/vulnerability/ast_scanner.rs
- FOUND: tests/vulnerability_tests/ast_scanner_tests.rs
- FOUND commit 25ab437 (test: RED failing tests)
- FOUND commit 77ef012 (feat: CWE-562, 570, 571 GREEN)
- FOUND commit 9534edc (feat: CWE-587 GREEN)

## Cumulative Count (Plan 02)

- New check_* functions added: 3 (check_return_stack_address, check_constant_condition, check_fixed_address_assignment)
- New helper functions added: 5 (collect_local_var_names, collect_local_vars_in_subtree, collect_decl_identifiers, parse_c_integer_literal, is_large_hex_literal)
- New unit tests added: 8
- Cumulative Phase 22 check_* functions: 8 (5 from Plan 01 + 3 from Plan 02)
- Cumulative Phase 22 unit tests: 17 (9 from Plan 01 + 8 from Plan 02)

## Next Phase Readiness

- Plan 02 Group B (CWE-562/570/571/587) complete. All 4 CWEs have TP+TN unit test coverage.
- Ready for Plan 03: CWE-617, CWE-674, CWE-835, CWE-256, CWE-398 (remaining structural + domain-specific)
- Pattern established: `collect_local_var_names` available for CWE-674 (self-recursion) function name extraction

---
*Phase: 22-ast-cwes-structuralpattern-expansion*
*Completed: 2026-05-12*
