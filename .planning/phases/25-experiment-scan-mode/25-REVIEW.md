---
phase: 25-experiment-scan-mode
reviewed: 2026-05-13T00:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - src/vulnerability/ast_scanner.rs
  - src/cli.rs
  - src/main.rs
  - tests/vulnerability_tests/ast_scanner_tests.rs
  - tests/autosar_ast_regression.rs
  - tests/juliet_regen_test.rs
  - .github/workflows/build-release.yml
  - scripts/strip_vulnerability.sh
findings:
  critical: 0
  warning: 6
  info: 3
  total: 9
status: issues_found
---

# Phase 25: Code Review Report

**Reviewed:** 2026-05-13T00:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Phase 25 added `--experiment-scan` gating for 17 high-FP CWE rules, CC env-var fixes for OSXCross cross-compilation in CI, and two new internal test files added to the strip script deletion list. The gating logic itself is structurally correct and the new tests (Test A/B/C) cover the happy path. However, several defects are present: three structural check functions that should be experimental are not gated (`apply_signal_handler_rules`, `apply_paired_lock_rules`, `apply_delete_rules`), the autosar regression test asserts on CWE-362/367 which are lexical-only and not produced by the AST scanner, the `juliet_regen_test.rs` always runs with `experiment_scan=true` (bypassing the very flag being tested), and the umask literal-`"0"` fast-path does not handle the `integer_literal` grammar variant that other scanner paths handle. Two additional quality issues round out the findings.

---

## Warnings

### WR-01: `apply_signal_handler_rules`, `apply_paired_lock_rules`, `apply_delete_rules` are not gated by `experiment_scan`

**File:** `src/vulnerability/ast_scanner.rs:355-357`
**Issue:** These three structural-check functions are called unconditionally from `scan_file_ast_or_lexical`, regardless of the `experiment_scan` parameter. CWE-479, CWE-591, and CWE-762 therefore fire in default mode even without `--experiment-scan`. This contradicts the phase design (Phase 25 Plan 01 D-11) which requires experimental rules to be gated. None of these three functions accept or forward an `experiment_scan` parameter.

`apply_signal_handler_rules` (CWE-479) has known false-positive surface area on codebases that use `signal()` with function-pointer arguments whose bodies call common libc functions for non-security reasons. `apply_delete_rules` (CWE-762) uses a text-level scan that can fire on identifiers containing the word `delete` (e.g., `delete_record`) despite the word-boundary check. The table-driven rules for CWE-338, CWE-426, CWE-676, CWE-680, and CWE-780 — which were explicitly marked `experimental: true` — are correctly gated. These three structural helpers are not.

**Fix:**
```rust
// In scan_file_ast_or_lexical (around lines 355-357), wrap the three calls:
if experiment_scan {
    findings.extend(apply_signal_handler_rules(
        tree.root_node(), src, path, component_name, component_ecosystem,
    ));
    findings.extend(apply_paired_lock_rules(
        tree.root_node(), src, path, component_name, component_ecosystem,
    ));
    findings.extend(apply_delete_rules(
        tree.root_node(), src, path, component_name, component_ecosystem,
    ));
}
```

---

### WR-02: `autosar_ast_regression.rs` asserts CWE-362 and CWE-367 from `run_ast_scanner`, but those CWEs are intentionally absent from the AST scanner

**File:** `tests/autosar_ast_regression.rs:34-37`
**Issue:** Lines 34–35 assert that `run_ast_scanner` produces exactly 1 CWE-362 and 1 CWE-367 finding on the AUTOSAR corpus. The module-level doc comment at `src/vulnerability/ast_scanner.rs:16` explicitly states "Deferred to lexical fallback only: 362, 367, 416, 476", and `AST_CWE_RULES` has no entries for those CWEs. The test can only pass today because the AUTOSAR corpus fixture is absent (line 13–14: `if !fixture.exists() { return; }`). If the fixture is ever added to CI the test will fail asserting 1 CWE-362 and 1 CWE-367 while the scanner produces 0 for both.

**Fix:** Either (a) remove the CWE-362 and CWE-367 assertions and add a comment that these come from the lexical scanner, or (b) change the test to build the union of AST + lexical findings before asserting:
```rust
let ast_findings = run_ast_scanner(&dirs, false);
let lex_findings = run_lexical_scanner(&dirs);
let by_cwe: BTreeMap<u32, usize> = ast_findings.iter()
    .chain(lex_findings.iter())
    .fold(BTreeMap::new(), |mut m, f| { *m.entry(f.cwe_id).or_default() += 1; m });
```

---

### WR-03: `juliet_regen_test.rs` always calls `run_ast_scanner` with `experiment_scan=true`, producing benchmark data that does not reflect the default mode

**File:** `tests/juliet_regen_test.rs:60`
**Issue:** The regeneration test hardcodes `run_ast_scanner(&component_dirs, true)`. After Phase 25 made `experiment_scan=false` the production default, the Juliet JSON written to `benchmark/juliet/ast.json` always reflects the high-FP experimental mode. Any benchmark comparison cited in ANALYSIS.md or phase plans that references this file will silently be comparing against the wrong operating mode. The FP-reduction claim for Phase 25 cannot be validated from this output.

**Fix:** Run in default mode and write separate output files:
```rust
let ast_default = run_ast_scanner(&component_dirs, false);
save_ast_json(&out_dir.join("ast_default.json"), &ast_default);
// Keep experimental output for reference
let ast_exp = run_ast_scanner(&component_dirs, true);
save_ast_json(&out_dir.join("ast_experimental.json"), &ast_exp);
```

---

### WR-04: `ArgCheck::ArgAtIndex` literal-`"0"` fast-path checks only `number_literal`, not `integer_literal`

**File:** `src/vulnerability/ast_scanner.rs:1874`
**Issue:** The D-10 umask guard rejects non-literal arguments with `args[idx].kind() != "number_literal"`. The comment at line 1871 says tree-sitter-c uses `number_literal` for numeric constants, but the rest of the scanner consistently handles both `number_literal` and `integer_literal` to account for grammar version variance (Pitfall 2): see `apply_division_rules` line 1514, `check_constant_condition` line 1044, `is_large_hex_literal` line 1109. The umask guard alone does not follow this pattern. If a grammar update reclassifies literal `0` as `integer_literal`, the kind check fails, the code falls through to `collect_subtree_text` + `token_present_with_boundary`, and the "0" substring matches inside nested calls like `umask(compute_mask(0))` — the exact false positive the guard was designed to prevent.

**Fix:**
```rust
if args[idx].kind() != "number_literal" && args[idx].kind() != "integer_literal" {
    false
} else {
    let arg_text = args[idx].utf8_text(src).unwrap_or("");
    arg_text == "0"
}
```

---

### WR-05: `check_poor_code_quality` sub-rule 3 (`==` at statement level) fires on the same nodes as `check_comparison_at_statement` (CWE-482), producing duplicate findings

**File:** `src/vulnerability/ast_scanner.rs:1435`
**Issue:** When `x == 5;` appears as an expression statement, both `check_comparison_at_statement` (CWE-482, called on line 396) and `check_poor_code_quality` (CWE-398, sub-rule 3: `op == "=="` at line 1435) fire on the same `expression_statement` node. This produces two separate findings for one defect. The test `test_cwe482_comparison_at_statement_level` asserts CWE-482 fires but does not assert CWE-398 is absent, so this duplication is untested.

**Fix:** Remove `"=="` from the operator set in `check_poor_code_quality` sub-rules 2/3, since CWE-482 is the correct and already-active category for that pattern:
```rust
if matches!(op, "+" | "-" | "*" | "/" | "%" | "|" | "&" | "^") {
    fired = true;
}
// Remove: | "==" from the match arm above
```

---

### WR-06: `check_self_recursion` (CWE-674) is not gated by `experiment_scan` but produces findings on intentionally recursive code

**File:** `src/vulnerability/ast_scanner.rs:409`
**Issue:** `check_self_recursion` fires in default mode (no `experiment_scan` gating). Self-recursion patterns are common in legitimate code (tree traversal, factorial, etc.) and have high false-positive potential on general C/C++ codebases. The function fires one CWE-674 finding per self-call, not per function, so a function that self-recurses multiple times across code paths emits multiple findings. The test `test_cwe674_direct_self_recursion` only verifies a trivial unconditional self-call; conditional recursion (e.g., `if (n > 0) return f(n-1);`) also fires because the call-expression walk is unconditional inside the function body.

**Fix:** Either mark CWE-674 as experimental and gate it behind `experiment_scan`, or add a guard that requires the self-call to be unconditional (i.e., not nested inside an `if_statement` condition).

---

## Info

### IN-01: `apply_delete_rules` carries an unused `_root: Node` parameter

**File:** `src/vulnerability/ast_scanner.rs:1729`
**Issue:** The function signature includes `_root: Node` which is explicitly unused. The doc comment acknowledges this: "The `_root` parameter is unused but kept for signature uniformity." The function performs a pure text scan over `src: &[u8]`, not an AST walk. Keeping a dead AST parameter misleads callers into assuming AST traversal is happening.

**Fix:** Remove the `_root` parameter and update the call site at line 357. If signature uniformity is desired, document that this function is text-based rather than AST-based.

---

### IN-02: `check_self_recursion` only collects top-level `function_definition` nodes, missing nested function definitions

**File:** `src/vulnerability/ast_scanner.rs:1253-1255`
**Issue:** The function iterates `root.children()` and filters `kind() == "function_definition"`. Functions nested inside other functions (GNU C extension) or inside `extern "C" {}` blocks are not direct children of root and are silently skipped. This is a coverage gap rather than a crash. The same limitation affects `apply_paired_lock_rules` and `apply_signal_handler_rules`.

**Fix:** Walk the full AST recursively to collect all `function_definition` nodes, or add a comment documenting the known limitation.

---

### IN-03: Phase 25 Test A/B in `ast_scanner_tests.rs` do not verify that structural helpers (CWE-479, CWE-591, CWE-762) are gated when `experiment_scan=false`

**File:** `tests/vulnerability_tests/ast_scanner_tests.rs:697-733`
**Issue:** Tests A (`experiment_scan_false_excludes_experimental_cwe`) and B (`experiment_scan_true_includes_experimental_cwe`) verify CWE-120 gating and Test C verifies CWE-617 is always active. No Phase 25 test verifies that CWE-479, CWE-591, or CWE-762 are absent when `experiment_scan=false`. Given WR-01 above (those helpers are not gated), this test gap is why the regression was not caught.

**Fix:** Add three test cases:
```rust
#[test]
fn experiment_scan_false_excludes_cwe479() {
    let src = b"void h(int s){malloc(1);} void r(){signal(2,h);}\n";
    let (_t, dirs) = setup_one_file("cwe479_gate.c", src);
    let findings = run_ast_scanner(&dirs, false);
    assert!(!findings.iter().any(|f| f.cwe_id == 479), ...);
}
// Similarly for CWE-591 and CWE-762
```

---

_Reviewed: 2026-05-13T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
