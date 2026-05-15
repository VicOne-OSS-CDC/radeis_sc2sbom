---
phase: 20-argument-value-ast-migration
plan: 01
subsystem: vulnerability
tags: [tree-sitter, ast, cwe-295, cwe-319, cwe-732, sast, ssl, curl, umask]

# Dependency graph
requires:
  - phase: 18-ast-scanner-core-and-benchmark
    provides: ArgCheck enum, AST_CWE_RULES table, visit_node function, SastFinding/SastSource types
  - phase: 19-cppcheck-removal
    provides: baseline codebase state for Phase 20 changes

provides:
  - ArgCheck::ArgAtIndex(u8, &'static [&'static str]) variant in ast_scanner.rs
  - collect_subtree_text helper function for recursive AST leaf text collection
  - Migrated CWE-295/319/732 rules using positional arg scoping (eliminates ContainsTokens)
  - wolfSSL_CTX_set_verify added to CWE-295 rule (D-06 gap fix)
  - 11 new test_argval_* AST tests covering TP, FP guards, nested expression, nested-call umask

affects: [20-02-PLAN, Plan 02 removes lexical scanner CWE-295/319/732 rules now that AST is authoritative]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "ArgAtIndex positional arg scoping: inspect specific argument index via collect_subtree_text + token_present_with_boundary"
    - "collect_subtree_text: uses named_child_count==0 as leaf condition (not child_count) to handle nodes with unnamed-only children like tree-sitter-c 'null' keyword"
    - "Umask exact-literal guard: kind=='number_literal' AND text=='0' to prevent nested-call false positives (D-10)"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "D-01: ArgAtIndex(u8, &'static [&'static str]) replaces ContainsTokens — positional scoping eliminates cross-arg false positives"
  - "D-06: wolfSSL_CTX_set_verify added to CWE-295 rule to close gap vs lexical scanner baseline"
  - "D-09: CURLOPT_SSL_VERIFYPEER and CURLOPT_SSL_VERIFYHOST use option-name-only detection; ArgAtTwoIndices deferred"
  - "D-10: umask exact-literal guard: arg must be number_literal kind with text exactly '0'; prevents umask(compute_mask(0)) FP"
  - "Deviation: CURLOPT_USE_SSL rule simplified to ArgAtIndex(1, [CURLOPT_USE_SSL]) — combining both tokens in arg 1 is impossible since option and value occupy separate positional args"
  - "Deviation: tree-sitter-c uses 'number_literal' (not 'integer_literal') for numeric constants; plan/research incorrectly stated integer_literal"
  - "Deviation: collect_subtree_text uses named_child_count==0 for leaf check; tree-sitter 'null' node has child_count=1 (one unnamed child) but named_child_count=0"

patterns-established:
  - "ArgAtIndex evaluation pattern: out-of-bounds guard → special exact-literal case for tokens==[0] → collect_subtree_text + token_present_with_boundary"
  - "collect_subtree_text leaf condition: named_child_count()==0, not child_count()==0"
  - "AST test fixture pattern: inline b\"...\" C source in setup_one_file; no separate .c fixture files"

requirements-completed: [ARGVAL-01, ARGVAL-02]

# Metrics
duration: 45min
completed: 2026-05-12
---

# Phase 20 Plan 01: ArgAtIndex AST Migration Summary

**AST scanner CWE-295/319/732 rules migrated from ContainsTokens (all-arg scan) to ArgAtIndex (positional arg scoping), eliminating umask(0077) false positive and adding wolfSSL_CTX_set_verify gap fix, with 11 new AST test cases**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 3 (Tasks 1+2 committed atomically + fix commit + Task 3)
- **Files modified:** 2

## Accomplishments

- Added `ArgCheck::ArgAtIndex(u8, &'static [&'static str])` variant and removed `ContainsTokens` entirely from both enum and rule table
- Migrated CWE-295 (SSL verify), CWE-319 (curl easy setopt), and CWE-732 (umask, DACL) rules to positional argument scoping
- Added `wolfSSL_CTX_set_verify` to the CWE-295 rule (closes wolfSSL coverage gap from Phase 18)
- Added `collect_subtree_text` helper with correct `named_child_count()==0` leaf condition (handles tree-sitter `null` keyword node)
- Umask exact-literal guard uses `number_literal` kind check (tree-sitter-c's actual kind for numeric constants) to prevent `umask(compute_mask(0))` false positives
- All 11 new `test_argval_*` AST tests pass; full test suite green (372 passed, 1 pre-existing unrelated failure)

## Task Commits

1. **Tasks 1+2: ArgAtIndex variant + rule migration** - `78d242f` (refactor)
2. **Bug fixes: number_literal, named_child_count, CURLOPT_USE_SSL rule** - `61fb0a1` (fix)
3. **Task 3: 11 AST argval tests** - `9565a2c` (test)

## Files Created/Modified

- `src/vulnerability/ast_scanner.rs` — ArgCheck enum updated; ArgAtIndex match arm added; collect_subtree_text helper added; AST_CWE_RULES table migrated for CWE-295/319/732
- `tests/vulnerability_tests/ast_scanner_tests.rs` — 11 new test_argval_* test functions added

## New Helper Function

```rust
fn collect_subtree_text(node: Node, src: &[u8]) -> String
```
Uses `named_child_count() == 0` as leaf condition. Recursively collects leaf texts via `named_children()`. Falls back to `utf8_text()` for nodes with only unnamed children (e.g. tree-sitter-c `null` keyword node which has `child_count=1` but `named_child_count=0`).

## Test Count Delta

- Before: 6 AST scanner tests
- After: 17 AST scanner tests (+11 new `test_argval_*` tests)

## wolfSSL Gap Fix Confirmation

`wolfSSL_CTX_set_verify` added to CWE-295 rule (D-06). Test `test_argval_cwe295_ast_wolfssl_verify_none` confirms the gap is closed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tree-sitter-c uses 'number_literal' not 'integer_literal'**
- **Found during:** Task 3 testing
- **Issue:** PLAN.md and RESEARCH.md stated the node kind for numeric literals is `"integer_literal"`. tree-sitter-c grammar actually uses `"number_literal"`. The umask kind check `args[idx].kind() != "integer_literal"` always returned true (wrong kind), so `umask(0)` never fired.
- **Fix:** Changed kind check to `args[idx].kind() != "number_literal"` and updated all references in comments
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Verification:** `test_argval_cwe732_ast_umask_zero` passes; `test_argval_cwe732_ast_umask_octal_no_fp` passes
- **Committed in:** `61fb0a1`

**2. [Rule 1 - Bug] collect_subtree_text leaf condition used child_count instead of named_child_count**
- **Found during:** Task 3 testing
- **Issue:** `collect_subtree_text` used `child_count() == 0` to identify leaf nodes. Tree-sitter-c's `null` node (representing the `NULL` keyword in C) has `child_count = 1` (one unnamed child "NULL") but `named_child_count = 0`. The function took the non-leaf branch, iterated zero named children, and returned empty string — causing the DACL NULL rule to never fire.
- **Fix:** Changed leaf condition to `named_child_count() == 0` with fallback to `utf8_text()`. This correctly handles `null` nodes and other nodes with only unnamed children.
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Verification:** `test_argval_cwe732_ast_dacl_null` passes
- **Committed in:** `61fb0a1`

**3. [Rule 1 - Bug] CURLOPT_USE_SSL combined-token rule (ArgAtIndex(1, [USE_SSL, CURLUSESSL_NONE])) is impossible to satisfy**
- **Found during:** Task 3 testing
- **Issue:** The plan specified `ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` requiring both tokens in arg 1 only. In the actual curl API, `curl_easy_setopt(curl, CURLOPT_USE_SSL, CURLUSESSL_NONE)` places the option at arg 1 and the value at arg 2. Both tokens can never both appear in arg 1 alone, so the rule would never fire.
- **Fix:** Simplified to `ArgAtIndex(1, &["CURLOPT_USE_SSL"])` — option-name-only detection consistent with the VERIFYPEER/VERIFYHOST rules. The must_have "CWE-319 reported when curl_easy_setopt called with CURLOPT_USE_SSL+CURLUSESSL_NONE" is satisfied because the option name at arg 1 is the reliable detection signal.
- **Files modified:** src/vulnerability/ast_scanner.rs
- **Verification:** `test_argval_cwe319_ast_use_ssl_none` passes
- **Committed in:** `61fb0a1`

---

**Total deviations:** 3 auto-fixed (all Rule 1 - bugs)
**Impact on plan:** All auto-fixes required for correctness. The plan's stated behavior (all three CWEs detected with correct FP guards) is fully achieved. No scope creep.

## Deferred Items

- `ArgAtTwoIndices` variant (D-09) — deferred to a future phase. Option-name-only detection at arg 1 is sufficient for CWE-319 curl rules.
- Plan 02 will remove lexical scanner CWE-295/319/732 `arg_value_contains` rules now that AST scanner is authoritative.

## Issues Encountered

Three bugs in the plan's implementation spec were caught during test execution and auto-fixed per Deviation Rule 1. The core design (ArgAtIndex, collect_subtree_text, umask kind guard) was correct; only the specific implementation details (node kind name, leaf condition logic, AND-token-in-one-arg limitation) needed correction.

## Known Stubs

None — all rules are fully wired and functional.

## Self-Check: PASSED

Files exist:
- src/vulnerability/ast_scanner.rs: FOUND
- tests/vulnerability_tests/ast_scanner_tests.rs: FOUND

Commits exist:
- 78d242f: FOUND
- 61fb0a1: FOUND
- 9565a2c: FOUND

## Next Phase Readiness

Plan 02 can now safely remove the five `arg_value_contains: Some(...)` rules from `cwe_scanner.rs` (CWE-295, CWE-319 x3, CWE-732/umask) and delete the `paren_args_contain_all` helper. The AST scanner is the authoritative detection path for these CWEs. The 11 new AST tests serve as the regression guard for that cleanup.

---
*Phase: 20-argument-value-ast-migration*
*Completed: 2026-05-12*
