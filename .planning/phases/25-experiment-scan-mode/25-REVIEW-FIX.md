---
phase: 25-experiment-scan-mode
fixed_at: 2026-05-13T00:00:00Z
review_path: .planning/phases/25-experiment-scan-mode/25-REVIEW.md
iteration: 1
findings_in_scope: 9
fixed: 9
skipped: 0
status: all_fixed
---

# Phase 25: Code Review Fix Report

**Fixed at:** 2026-05-13T00:00:00Z
**Source review:** .planning/phases/25-experiment-scan-mode/25-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 9
- Fixed: 9
- Skipped: 0

## Fixed Issues

### WR-01: Gate apply_signal_handler_rules, apply_paired_lock_rules, apply_delete_rules behind experiment_scan

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 19af2ea
**Applied fix:** Wrapped the three call sites in `scan_file_ast_or_lexical` inside `if experiment_scan { ... }`. CWE-479, CWE-591, and CWE-762 no longer fire in default (non-experimental) mode.

---

### WR-02: Remove incorrect CWE-362/367 assertions from autosar_ast_regression.rs

**Files modified:** `tests/autosar_ast_regression.rs`
**Commit:** 25303f3
**Applied fix:** Removed `assert_eq!(by_cwe.get(&362)...)` and `assert_eq!(by_cwe.get(&367)...)` and updated total-findings assertion from 3 to 1. Added a comment documenting that CWE-362 and CWE-367 are lexical-only per the ast_scanner.rs module doc.

---

### WR-03: Run juliet regen test in default mode (experiment_scan=false)

**Files modified:** `tests/juliet_regen_test.rs`
**Commit:** 31732bd
**Applied fix:** Changed `run_ast_scanner(&component_dirs, true)` to `run_ast_scanner(&component_dirs, false)` so the benchmark JSON reflects the production default mode. FP-reduction claims for Phase 25 can now be validated from this output.

---

### WR-04: Add integer_literal to umask ArgAtIndex fast-path guard

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 19af2ea
**Applied fix:** Changed `args[idx].kind() != "number_literal"` to `args[idx].kind() != "number_literal" && args[idx].kind() != "integer_literal"` to match the dual-kind pattern used consistently elsewhere in the scanner. Updated comment to document the rationale.

---

### WR-05: Remove "==" from check_poor_code_quality to eliminate CWE-398/CWE-482 duplicates

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 19af2ea
**Applied fix:** Removed `"=="` from the `matches!` arm in sub-rule 2/3 of `check_poor_code_quality`. Added a comment explaining that `check_comparison_at_statement` (CWE-482) already covers this pattern.

---

### WR-06: Gate check_self_recursion (CWE-674) behind experiment_scan

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 19af2ea
**Applied fix:** Wrapped the `check_self_recursion` call site in `apply_ast_rules` inside `if experiment_scan { ... }`. Added a comment documenting the high-FP rationale (legitimate recursive code such as tree traversal, factorial, etc.).

---

### IN-01: Remove unused _root: Node parameter from apply_delete_rules

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 19af2ea
**Applied fix:** Removed `_root: Node` from the `apply_delete_rules` function signature and updated the call site (inside the `if experiment_scan` block added by WR-01). Updated the doc comment to clarify this is a text-level scan, not an AST walk.

---

### IN-02: Add known-limitation comment to check_self_recursion traversal

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 19af2ea
**Applied fix:** Added a multi-line comment above the `root.children()` loop in `check_self_recursion` documenting that only top-level `function_definition` nodes are visited, and that functions nested in other functions (GNU C extension) or `extern "C" {}` blocks are silently skipped.

---

### IN-03: Add gating tests for CWE-479, CWE-591, CWE-762 when experiment_scan=false

**Files modified:** `tests/vulnerability_tests/ast_scanner_tests.rs`
**Commit:** 55affba
**Applied fix:** Added three tests — `experiment_scan_false_excludes_cwe479`, `experiment_scan_false_excludes_cwe591`, `experiment_scan_false_excludes_cwe762` — each asserting the respective CWE is absent when `experiment_scan=false`. All three pass (`cargo test --features internal --test all_tests experiment_scan_false_excludes_cwe` confirmed).

---

_Fixed: 2026-05-13T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
