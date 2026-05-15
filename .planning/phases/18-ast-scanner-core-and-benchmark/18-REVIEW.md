---
phase: 18-ast-scanner-core-and-benchmark
reviewed: 2026-05-12T00:00:00Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - Cargo.toml
  - docs/BENCHMARK.md
  - docs/BENCHMARK_FIXTURES.md
  - src/main.rs
  - src/vulnerability/ast_scanner.rs
  - src/vulnerability/cwe_scanner.rs
  - src/vulnerability/mod.rs
  - tests/benchmark.rs
  - tests/vulnerability_tests/ast_scanner_tests.rs
  - tests/vulnerability_tests/mod.rs
findings:
  critical: 2
  warning: 3
  info: 1
  total: 6
status: issues_found
---

# Phase 18: Code Review Report

**Reviewed:** 2026-05-12
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

This phase introduces the AST-based CWE scanner (`ast_scanner.rs`) using tree-sitter-c, a benchmark integration test, and tests covering the 13 tractable CWEs. The overall architecture is sound: the primary/fallback design (AST → lexical on parse error), the `FixedSizeBuffer` discrimination logic, and the deduplication pipeline are well-structured.

Two blockers are present. The first is a tree-sitter cursor sharing bug in the array-collection traversal functions — the same bug that `visit_node` explicitly avoided (noting it as Pitfall 1) but that `collect_file_scope_arrays_rec` and `collect_arrays_in_subtree` both reproduce. This causes incorrect fixed-size buffer detection (missed true positives and/or incorrect scoping). The second is a data-loss bug in the benchmark's ground-truth TSV loader: a `?` operator inside a loop body causes the entire load to abort on any unparseable line.

There are also three warnings: a false-positive over-approximation in the CWE-369 AST rule; a CWE-295 AST vs. lexical inconsistency; and a dead parameter in the file-scope array collector.

---

## Critical Issues

### CR-01: Shared TreeCursor corrupts array-collection traversal (missed TP findings)

**File:** `src/vulnerability/ast_scanner.rs:309-333` and `344-363`

**Issue:** Both `collect_file_scope_arrays_rec` and `collect_arrays_in_subtree` receive a single `&mut tree_sitter::TreeCursor` passed by mutable reference through all recursive call levels. A `TreeCursor` is a single stateful pointer into the tree. The pattern:

```
cursor.goto_first_child()
  loop:
    recurse(cursor.node(), cursor, ...)   // recursive call also calls goto_first_child on SAME cursor
    cursor.goto_next_sibling()
cursor.goto_parent()
```

is broken because after the recursive call returns (having moved the cursor deep into the subtree and back up via `goto_parent()`), the outer-level `cursor.goto_next_sibling()` is advancing from the cursor's restored position — but that position is the last child the inner traversal processed, not the next sibling of the outer-level node being iterated. Tree-sitter cursors do not maintain a stack of saved positions; each `goto_*` mutates the single position.

This bug was explicitly identified and avoided in `visit_node` (line 268: "Pitfall 1: fresh cursor per call level"), which creates a new cursor via `node.walk()` at each call level. The array-collection functions did not receive the same fix.

**Impact:** Fixed-size buffer array names can be missed or collected from wrong scopes. Missing array names means the `FixedSizeBuffer` ArgCheck fires `false` for arrays that should match, producing false negatives for CWE-119, CWE-120, CWE-122, CWE-125 (and CWE-134 via strncpy/sprintf for those CWEs). No test currently covers a multi-declaration or multi-function file deeply enough to expose this.

**Fix:** Follow the same pattern as `visit_node` — create a fresh cursor at each call level instead of sharing one. Remove the `cursor` parameter entirely:

```rust
fn collect_file_scope_arrays_rec(
    node: Node,
    src: &[u8],
    out: &mut HashSet<String>,
    inside_function: bool,
) {
    if node.kind() == "function_definition" {
        return;
    }
    if !inside_function && node.kind() == "declaration" {
        collect_array_declarators(node, src, out);
    }
    let mut cursor = node.walk(); // fresh cursor per level (Pitfall 1)
    if cursor.goto_first_child() {
        loop {
            collect_file_scope_arrays_rec(cursor.node(), src, out, inside_function);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

Apply the same change to `collect_arrays_in_subtree`. Update the two call sites (`collect_file_scope_fixed_arrays` line 305 and `collect_function_scope_fixed_arrays` line 340) to drop the cursor argument.

---

### CR-02: Ground-truth TSV loader aborts on any unparseable line — silently drops all data

**File:** `tests/benchmark.rs:32-49`

**Issue:** `load_ground_truth` iterates over lines and uses the `?` operator inside the loop body:

```rust
let line_no: u32 = cols[1].parse().ok()?;  // line 43
let cwe: u32     = cols[2].parse().ok()?;  // line 44
```

When `parse()` fails, `.ok()` converts the error to `None`, and `?` propagates `None` as the return value of the entire `load_ground_truth` function — meaning `None` is returned to `run_one_fixture`, which then treats the entire TSV as absent (`truth.is_none()`). All correctly-parsed rows accumulated before the bad line are silently discarded. Any malformed line (whitespace, trailing tab, Windows line endings) will cause all ground-truth data to be lost without any warning.

This transforms a data-quality issue in the TSV into an invisible correctness failure: the benchmark will show raw counts only (no TP/FP columns) even when a `.benchmark_truth.tsv` file is present and substantially correct.

**Fix:** Use `continue` on parse failure instead of `?`, and emit a warning:

```rust
for line in body.lines() {
    if line.starts_with('#') || line.trim().is_empty() { continue; }
    let cols: Vec<&str> = line.split('\t').collect();
    if cols.len() < 4 { continue; }
    let line_no: u32 = match cols[1].parse() {
        Ok(n) => n,
        Err(_) => { eprintln!("Warning: skipping malformed truth row: {:?}", line); continue; }
    };
    let cwe: u32 = match cols[2].parse() {
        Ok(n) => n,
        Err(_) => { eprintln!("Warning: skipping malformed truth row: {:?}", line); continue; }
    };
    let label = cols[3].trim().to_string();
    if label != "TP" && label != "FP" { continue; }
    map.insert((cols[0].to_string(), line_no, cwe), label);
}
Some(map)
```

---

## Warnings

### WR-01: CWE-369 AST rule fires on ALL `div`/`ldiv`/`lldiv` calls — massive false-positive rate

**File:** `src/vulnerability/ast_scanner.rs:64`

**Issue:** The rule:

```rust
AstCweRule { cwe_id: 369, functions: &["div", "ldiv", "lldiv"], arg_check: ArgCheck::AnyCall },
```

flags every call to `div()`, `ldiv()`, or `lldiv()` regardless of arguments. CWE-369 is "Divide by Zero" — the actual risk is calling these with a zero divisor. Flagging all calls is an extreme over-approximation with high false-positive rate; legitimate use of `div(n, 2)` would generate a finding.

The lexical scanner avoids this problem by using `contains_div_by_zero` which scans for `/` or `%` immediately followed by a literal `0` with word boundaries. The AST scanner has no equivalent argument-value check for the divisor.

**Fix:** Either change `arg_check` to `ArgCheck::ContainsTokens(&["0"])` to require the divisor argument to contain the token `"0"` (matching the lexical scanner's intent), or defer CWE-369 entirely to the lexical fallback (removing the rule from `AST_CWE_RULES`) consistent with the deferred-CWE rationale in the module doc comment.

---

### WR-02: CWE-295 AST rule fires on ALL `SSL_CTX_set_verify` calls — contradicts lexical scanner

**File:** `src/vulnerability/ast_scanner.rs:59`

**Issue:** The AST rule:

```rust
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify", "SSL_CTX_set_cert_verify_callback"], arg_check: ArgCheck::AnyCall },
```

flags every call to `SSL_CTX_set_verify` regardless of the verify mode argument. The lexical scanner's corresponding rule (line 92 of `cwe_scanner.rs`) correctly gates on the `SSL_VERIFY_NONE` token:

```rust
CweRule { cwe_id: 295, ..., arg_value_contains: Some(&["SSL_VERIFY_NONE"]) },
```

A call with `SSL_VERIFY_PEER` (the correct, secure mode) will produce a CWE-295 AST finding but NOT a lexical finding. After deduplication, the `SastSource::Ast` finding survives since it does not match any `SastSource::Cppcheck` finding. This is a precision regression introduced in Phase 18.

The module comment notes "Phase 20 ARGVAL-01 migrates to arg-value AST inspection" but the current AnyCall strategy emits false positives now, not "deferred" behavior — it actively degrades precision compared to the existing lexical scanner.

**Fix:** Use `ArgCheck::ContainsTokens(&["SSL_VERIFY_NONE"])` to mirror the lexical rule:

```rust
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify", "SSL_CTX_set_cert_verify_callback"], arg_check: ArgCheck::ContainsTokens(&["SSL_VERIFY_NONE"]) },
```

For `SSL_CTX_set_cert_verify_callback`, AnyCall may be appropriate (any non-NULL callback is suspicious), but `SSL_CTX_set_verify` and `SSL_set_verify` must be gated on the mode argument.

---

### WR-03: Dead `inside_function` parameter in `collect_file_scope_arrays_rec`

**File:** `src/vulnerability/ast_scanner.rs:314`

**Issue:** The `inside_function: bool` parameter is never set to `true` at any call site. The function returns early for `function_definition` nodes (line 317), which prevents descent into function bodies — so `inside_function` is always `false`. The guard `if !inside_function && node.kind() == "declaration"` (line 321) is functionally equivalent to `if node.kind() == "declaration"`. The parameter adds dead complexity and signals intent that the implementation does not actually fulfill (it implies there is a code path that descends into functions, which there is not — the early-return guard handles that instead).

**Fix:** Remove the `inside_function` parameter from the function signature and all call sites. The early-return on `function_definition` is the correct and sufficient guard.

---

## Info

### IN-01: `test_ast_all_tractable_cwes` test accepts CWE-369 from lexical fallback union, obscuring AST coverage gap

**File:** `tests/vulnerability_tests/ast_scanner_tests.rs:83-95`

**Issue:** The test comment on line 45 says "CWE-369 via the div/ldiv/lldiv AnyCall rule, or accepted from lexical fallback on same fixture" and the assertion uses a union of AST + lexical findings:

```rust
// CWE-369: accepted from AST or lexical fallback union
assert!(all_cwe_ids.contains(&369), ...);
```

The other 13 CWEs are asserted against `findings` (AST only) at line 84-89. If the AnyCall CWE-369 rule is removed (as WR-01 recommends), the test still passes via the lexical union — which is correct but the test comment claiming it is an AST rule remains misleading. The test structure means the AST CWE-369 rule is never independently verified.

**Fix:** If CWE-369 is intentionally delegated to lexical only, move the comment to the module-level deferred list (alongside 362, 367, 416, 476) and remove the `AnyCall` rule. Keep the union assertion in the test but update the comment to say "CWE-369 is detected via lexical fallback only."

---

_Reviewed: 2026-05-12_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
