---
phase: 18-ast-scanner-core-and-benchmark
fixed_at: 2026-05-12T00:00:00Z
review_path: .planning/phases/18-ast-scanner-core-and-benchmark/18-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 18: Code Review Fix Report

**Fixed at:** 2026-05-12
**Source review:** .planning/phases/18-ast-scanner-core-and-benchmark/18-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (CR-01, CR-02, WR-01, WR-02, WR-03)
- Fixed: 5
- Skipped: 0

## Fixed Issues

### CR-01: Shared TreeCursor corrupts array-collection traversal

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** ab52f5f
**Applied fix:** Removed the shared `&mut tree_sitter::TreeCursor` parameter from both `collect_file_scope_arrays_rec` and `collect_arrays_in_subtree`. Each function now creates a fresh cursor via `node.walk()` at its call level, matching the Pitfall 1 pattern already used in `visit_node`. Updated both call sites (`collect_file_scope_fixed_arrays` and `collect_function_scope_fixed_arrays`) to drop the cursor argument. WR-03's dead `inside_function` parameter was removed as part of this same structural change (see WR-03 below).

---

### CR-02: Ground-truth TSV loader aborts on any unparseable line

**Files modified:** `tests/benchmark.rs`
**Commit:** 7641634
**Applied fix:** Replaced the `?` operator on `cols[1].parse().ok()?` and `cols[2].parse().ok()?` with explicit `match` arms that emit a warning via `eprintln!` and `continue` to the next line on parse failure. The loader now accumulates all parseable rows and returns `Some(map)` unconditionally at the end, rather than aborting the entire load on any single malformed row.

---

### WR-01: CWE-369 AnyCall rule fires on all div/ldiv/lldiv calls

**Files modified:** `src/vulnerability/ast_scanner.rs`, `tests/vulnerability_tests/ast_scanner_tests.rs`
**Commit:** cd19f66
**Applied fix:** Removed the `AstCweRule { cwe_id: 369, ... ArgCheck::AnyCall }` entry from `AST_CWE_RULES`. Added CWE-369 to the deferred list in the module-level doc comment alongside 362, 367, 416, 476, with a note explaining the false-positive rationale. Updated the test comment in `test_ast_all_tractable_cwes` from "via the div/ldiv/lldiv AnyCall rule, or accepted from lexical fallback" to "detected via lexical fallback only" to accurately reflect intent.

---

### WR-02: CWE-295 AST rule fires on ALL SSL_CTX_set_verify calls

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** 06560ef
**Applied fix:** Split the single CWE-295 rule into two. `SSL_CTX_set_verify` and `SSL_set_verify` now use `ArgCheck::ContainsTokens(&["SSL_VERIFY_NONE"])`, mirroring the lexical scanner's `arg_value_contains: Some(&["SSL_VERIFY_NONE"])` guard. `SSL_CTX_set_cert_verify_callback` retains `ArgCheck::AnyCall` since any custom callback replacing the verify logic is inherently suspicious. This eliminates false positives for secure `SSL_VERIFY_PEER` calls.

---

### WR-03: Dead `inside_function` parameter in `collect_file_scope_arrays_rec`

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** ab52f5f (same commit as CR-01)
**Applied fix:** The `inside_function: bool` parameter was removed from `collect_file_scope_arrays_rec` as part of the CR-01 structural fix. The early-return guard on `function_definition` (line 314) is the correct and sufficient mechanism for preventing descent into function bodies; the dead parameter and its associated `if !inside_function &&` guard were both eliminated.

## Skipped Issues

None — all findings were fixed.

---

_Fixed: 2026-05-12_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
