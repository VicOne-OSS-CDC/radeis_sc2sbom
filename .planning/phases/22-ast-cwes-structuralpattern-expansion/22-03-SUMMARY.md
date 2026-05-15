---
phase: 22-ast-cwes-structuralpattern-expansion
plan: 03
subsystem: sast
tags: [rust, tree-sitter, sast, cwe, ast-scanner]

# Dependency graph
requires:
  - phase: 22-02
    provides: check_* structural visitor pattern, collect_local_var_names, parse_c_integer_literal, unwrap_parens helpers
provides:
  - "AstCweRule entry for assert (CWE-617) in AST_CWE_RULES"
  - "extract_function_name + extract_ident_from_declarator helpers"
  - "check_self_recursion: CWE-674 (direct self-recursion detection)"
  - "check_self_calls: recursive body walker for self-call detection"
  - "check_plaintext_password: CWE-256 (identifier heuristic + string_literal initializer)"
  - "drill_to_identifier: pointer_declarator unwrapping helper"
  - "body_has_escape: break/return/goto/exit presence check"
  - "check_infinite_loop: CWE-835 (while(1)/for(;;) with body-check D-05)"
  - "check_poor_code_quality: CWE-398 (4 sub-rules: bare literal, discarded arithmetic, discarded comparison, self-assignment)"
  - "11 unit tests in ast_scanner_tests.rs (TP + TN for CWE-617, 674, 256, 835, 398)"
affects: [22-04, phase-23]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Root-level function_definition iteration: root.children(&mut cursor) filtered by .kind() == 'function_definition'"
    - "extract_ident_from_declarator: recursive declarator chain traversal (function_declarator -> pointer_declarator -> identifier)"
    - "body_has_escape: early-return recursive traversal returning bool"
    - "check_poor_code_quality: at-most-once-per-expression_statement rule with bool flag"
    - "Case-insensitive substring matching: name.to_lowercase().contains('password')"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "CWE-617 implemented as AnyCall entry in AST_CWE_RULES (Pattern G) — assert is a call-site pattern, fires via existing visit_node path"
  - "CWE-674 check_self_recursion iterates root-level function_definition children (not recursive walk) to avoid false positives from nested functions"
  - "CWE-256 uses drill_to_identifier helper separate from existing extract_ident_under to handle declaration-specific pointer_declarator chain"
  - "CWE-835 body-check approach confirmed (D-05): while(1)+break TN test passes"
  - "CWE-398 sub-rule 3 (== at statement level) overlaps with CWE-482 by design — per RESEARCH.md note, same defect, intentional"
  - "check_poor_code_quality fires AT MOST ONCE per expression_statement (first sub-rule wins)"

requirements-completed: [CWEXP-02]

# Metrics
duration: ~18min
completed: 2026-05-12
---

# Phase 22 Plan 03: Structural-Pattern CWE Expansion (Group C) Summary

**4 new check_* structural visitor functions + 1 AST_CWE_RULES AnyCall entry completing all 15 Phase 22 CWEs (CWE-617 via table, CWE-674/256/835/398 via dedicated visitors)**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 3 (TDD: RED + GREEN per group)
- **Files modified:** 2

## Accomplishments

- Added 11 unit tests (TP + TN for CWE-617, 674x2, 256x3, 835x2, 398x3) — all RED before implementation, all GREEN after
- Added `AstCweRule { cwe_id: 617, functions: &["assert"], arg_check: ArgCheck::AnyCall }` to AST_CWE_RULES (Pattern G)
- Implemented `extract_function_name` + `extract_ident_from_declarator` (declarator chain traversal helpers)
- Implemented `check_self_recursion` + `check_self_calls` (CWE-674 — direct function self-call detection)
- Implemented `drill_to_identifier` (pointer_declarator chain helper for CWE-256)
- Implemented `check_plaintext_password` (CWE-256 — identifier heuristic + string_literal initializer)
- Implemented `body_has_escape` (recursive bool helper for CWE-835 D-05 body-check)
- Implemented `check_infinite_loop` (CWE-835 — while(1) and for(;;) with body-check; confirmed TN with break)
- Implemented `check_poor_code_quality` (CWE-398 — 4 sub-rules: bare literal, discarded arithmetic, discarded comparison, self-assignment)
- Full suite: 412 tests pass (1 pre-existing env-only SPDX validation failure unrelated to our changes)

## Task Commits

1. **Task 1: 11 failing tests for CWE-617, 674, 256, 835, 398 (RED)** - `b82f299`
2. **Task 2: CWE-617 AnyCall + check_self_recursion + check_plaintext_password** - `4ece7c0`
3. **Task 3: check_infinite_loop (CWE-835) + check_poor_code_quality (CWE-398)** - `e2f26b1`

## Files Created/Modified

- `src/vulnerability/ast_scanner.rs` — 1 new AST_CWE_RULES entry + 4 new check_* functions + 4 helper functions (~382 lines added); total file: 1,914 lines
- `tests/vulnerability_tests/ast_scanner_tests.rs` — 11 new test functions appended (134 lines)

## New Functions Added (Plan 03)

| Function | CWE | Lines | Purpose |
|----------|-----|-------|---------|
| `extract_function_name` | 674 | ~8 | Entry helper: extracts fn name from function_definition declarator |
| `extract_ident_from_declarator` | 674 | ~18 | Recursive: drills function_declarator → pointer_declarator → identifier |
| `check_self_recursion` | 674 | ~15 | Root-level visitor: finds self-recursive function_definition nodes |
| `check_self_calls` | 674 | ~20 | Body walker: fires CWE-674 when call_expression matches enclosing fn name |
| `drill_to_identifier` | 256 | ~15 | Helper: extracts identifier from declarator chain (pointer_declarator layers) |
| `check_plaintext_password` | 256 | ~35 | Declaration visitor: fires CWE-256 when password keyword + string_literal init |
| `body_has_escape` | 835 | ~20 | Bool helper: returns true if subtree has break/return/goto/exit/abort |
| `check_infinite_loop` | 835 | ~35 | while/for visitor: fires CWE-835 when literal-infinite loop has no escape |
| `check_poor_code_quality` | 398 | ~50 | expression_statement visitor: fires CWE-398 on 4 no-effect sub-patterns |

## Cumulative Phase 22 Function Summary (Plans 01-03)

| Plan | Functions Added | Helper Functions | Tests Added |
|------|----------------|------------------|-------------|
| 01 | 5 check_* (478/484, 483, 481, 482, 480) | unwrap_parens, is_identifier_vs_null | 9 |
| 02 | 3 check_* (562, 570/571, 587) | collect_local_var_names, collect_local_vars_in_subtree, collect_decl_identifiers, parse_c_integer_literal, is_large_hex_literal | 8 |
| 03 | 4 check_* (617 via table, 674, 256, 835, 398) | extract_function_name, extract_ident_from_declarator, drill_to_identifier, body_has_escape | 11 |
| **Total** | **12 check_* + 1 table entry** | **12 helper functions** | **28 tests** |

## Final AST_CWE_RULES Table (Phase 22 addition)

Phase 22 adds one new entry to AST_CWE_RULES:
- `AstCweRule { cwe_id: 617, functions: &["assert"], arg_check: ArgCheck::AnyCall }` — appended after CryptEncrypt entry

## Per-CWE TP Results from Unit Tests (All 15 Phase 22 CWEs)

| CWE | Description | Test TP | Test TN | Status |
|-----|-------------|---------|---------|--------|
| 256 | Plaintext Password Storage | password declaration fires | username does NOT fire | GREEN |
| 398 | Poor Code Quality | 5;, x=x;, a+b; each fire | — | GREEN |
| 478 | Missing Default in Switch | switch w/o default fires | switch WITH default doesn't | GREEN |
| 480 | Use of Incorrect Operator | fn_ptr == 0 fires | — | GREEN |
| 481 | Assignment Instead of Comparison | if(x=5) fires | while((c=1)!=0) doesn't | GREEN |
| 482 | Comparison Instead of Assignment | x==5; fires | — | GREEN |
| 483 | Incorrect Block Delimitation | braceless if fires | if{} doesn't | GREEN |
| 484 | Omitted Break in Switch | fall-through case fires | — | GREEN |
| 562 | Return of Stack Variable Address | return local array fires | return global doesn't | GREEN |
| 570 | Expression Always False | if(0) fires | — | GREEN |
| 571 | Expression Always True | if(1) fires | — | GREEN |
| 587 | Fixed Address Assignment | (char*)0x400000 fires | (int*)0 doesn't | GREEN |
| 617 | Reachable Assertion | assert(x>0) fires | — | GREEN |
| 674 | Uncontrolled Recursion | helperBad(){helperBad()} fires | f(){g()} doesn't | GREEN |
| 835 | Infinite Loop | while(1){} fires | while(1){break;} doesn't | GREEN |

## Body-Check Approach Confirmed (CWE-835)

- `test_cwe835_while_one_with_break_no_finding` is GREEN — body-check D-05 correctly suppresses `while(1)` with `break` in body.
- This aligns with AUTOSAR embedded polling pattern tolerance requirement (D-15).

## Deviations from Plan

None — plan executed exactly as written.

All implementation details matched the specified patterns:
- CWE-617 added to AST_CWE_RULES as AnyCall entry (Pattern G)
- check_self_recursion iterates root-level function_definition nodes as specified in Code Example
- check_plaintext_password uses substring match on lowercased identifier name (D-08, D-10)
- body_has_escape covers break_statement, return_statement, goto_statement, exit/abort calls
- check_poor_code_quality fires at most once per expression_statement

## Known Stubs

None — all 4 new check_* functions and the AnyCall entry are fully implemented and emit real findings.

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All code is read-only static analysis inside `#[cfg(feature = "internal")]`. Threat mitigations from plan threat model:
- T-22-07 (DoS via check_self_recursion recursion): fresh cursor per level, bounded by AST depth — implemented
- T-22-08 (Tampering: function name extraction): utf8_text returns Result; missing name → check_self_calls not invoked — implemented
- T-22-09 (Info disclosure: CWE-256 password match): only file path + line reported in SastFinding; actual password string NOT included — implemented

## Self-Check: PASSED

- FOUND: src/vulnerability/ast_scanner.rs (1,914 lines)
- FOUND: tests/vulnerability_tests/ast_scanner_tests.rs (835 lines)
- FOUND: .planning/phases/22-ast-cwes-structuralpattern-expansion/22-03-SUMMARY.md
- FOUND commit b82f299 (test: 11 RED tests for CWE-617/674/256/835/398)
- FOUND commit 4ece7c0 (feat: CWE-617 AnyCall + check_self_recursion + check_plaintext_password)
- FOUND commit e2f26b1 (feat: check_infinite_loop + check_poor_code_quality)
- VERIFIED: `cargo test --features internal --test all_tests` → 412 passed, 1 failed (pre-existing env-only)

## Phase 22 Completion Status

All 15 Phase 22 CWEs now implemented:
- Group A (Plan 01): CWE-478, 484, 481, 482, 480, 483 — COMPLETE
- Group B (Plan 02): CWE-562, 570, 571, 587 — COMPLETE
- Group C (Plan 03): CWE-617, 674, 256, 835, 398 — COMPLETE

Total: 28 new unit tests, 12 new check_* functions, 12 helper functions, 1 new AST_CWE_RULES entry. Ready for Plan 04 (benchmark).

---
*Phase: 22-ast-cwes-structuralpattern-expansion*
*Completed: 2026-05-12*
