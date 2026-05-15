---
phase: 23-ast-cwes-domainspecific-expansion
reviewed: 2026-05-12T00:00:00Z
depth: standard
files_reviewed: 4
files_reviewed_list:
  - src/vulnerability/ast_scanner.rs
  - tests/vulnerability_tests/ast_scanner_tests.rs
  - tests/fixtures/c/cwe762_delete_bad.c
  - benchmark/juliet/ANALYSIS.md
findings:
  critical: 0
  warning: 7
  info: 3
  total: 10
status: issues_found
---

# Phase 23: Code Review Report

**Reviewed:** 2026-05-12T00:00:00Z
**Depth:** standard
**Files Reviewed:** 4
**Status:** issues_found

## Summary

Phase 23 added 5 table-driven `AstCweRule` entries (CWE-114, 272, 284, 427, 785) and 3 structural helpers (`apply_signal_handler_rules` for CWE-479, `apply_paired_lock_rules` for CWE-591, `apply_delete_rules` for CWE-762) to `src/vulnerability/ast_scanner.rs`. Tests and one C fixture were added in parallel.

The overall implementation is consistent with established patterns in the codebase. The most significant issues are: (1) `apply_delete_rules` is a pure text-level byte scan that does not use the AST at all, contradicting its location in an AST scanner and producing false positives when `delete` appears in string literals; (2) `apply_paired_lock_rules` has function-level scope that makes a VirtualAlloc in one branch of an `if/else` suppress a finding even when the paired `VirtualLock` is only in the other branch; (3) the CWE-591 TN test passes trivially but does not cover the cross-branch false-negative scenario; (4) `collect_signal_handlers` uses a `HashMap` that can only store one line per handler name, silently dropping duplicate signal registrations.

---

## Warnings

### WR-01: `apply_delete_rules` fires on `delete` inside string literals

**File:** `src/vulnerability/ast_scanner.rs:1640`
**Issue:** The comment scanner in `apply_delete_rules` correctly skips `//` and `/* */` comments, but it does NOT skip string literal content. Any occurrence of the word `delete` inside a C string literal (e.g., `const char *msg = "delete this file";` or SQL strings like `"DELETE FROM ..."`) will emit a CWE-762 finding. This is a correctness defect, not a theoretical edge case — SQL-heavy C codebases routinely embed `DELETE` keywords in string literals (text comparison is case-sensitive so uppercase is safe, but lowercase is not).

Additionally, the function's `_root: Node` parameter is explicitly ignored (prefixed with `_`), and the comment on line 1638 says "Text-level scan ... Requires that the match be a standalone identifier token (word-boundary check)." Identifier tokens do not appear inside string literals at the text level, but the scanner does not know it is inside a string literal.

**Fix:** Add string-literal skipping to the byte scanner loop, similar to how block comments are tracked:
```rust
let mut in_string = false;
// Inside the while loop, before comment checks:
if !in_line_comment && !in_block_comment && b == b'"' {
    in_string = !in_string;
    i += 1;
    continue;
}
if in_string {
    // skip backslash-escaped characters
    if b == b'\\' { i += 1; }  // skip next byte
    i += 1;
    continue;
}
```
Alternatively — and more robustly — use the existing AST parse tree to enumerate string literal node byte ranges and skip those ranges during the text scan.

---

### WR-02: `apply_paired_lock_rules` has false negatives for cross-branch VirtualAlloc/VirtualLock

**File:** `src/vulnerability/ast_scanner.rs:1573`
**Issue:** The implementation collects all call names in a function body into a single `HashSet<String>`, then checks `call_names.contains("VirtualAlloc") && !call_names.contains("VirtualLock")`. If a function contains `VirtualAlloc` in one `if` branch and `VirtualLock` in an unrelated `else` branch (or anywhere else in the same function), the check incorrectly suppresses the finding. The following code would produce no CWE-591 finding even though the allocation in the `bad` branch has no lock:

```c
void f(int condition, unsigned long sz) {
    if (condition) {
        void *p = VirtualAlloc(0, sz, 0x1000, 0x04);
        (void)p;  // no VirtualLock — real bug
    } else {
        void *q = VirtualAlloc(0, sz, 0x1000, 0x04);
        VirtualLock(q, sz);  // this satisfies the whole-function check
    }
}
```

**Fix:** Either (a) track `VirtualAlloc` and `VirtualLock` at call-site pairing level (complex), or (b) document this known limitation explicitly in the code comment and the ANALYSIS.md FP gate violation section so future reviewers understand the false-negative risk. Option (b) is the pragmatic choice given the existing high-FP-rate tolerance.

---

### WR-03: `collect_signal_handlers` silently drops duplicate signal registrations for the same handler

**File:** `src/vulnerability/ast_scanner.rs:1483`
**Issue:** `handler_lines` is a `HashMap<String, u32>`. If the same handler function is registered multiple times with `signal()` (e.g., in different code paths), only the last registration line is retained because `HashMap::insert` overwrites existing entries. This produces a false negative for the earlier registration lines:

```c
void h(int s) { malloc(10); }
void setup(int mode) {
    if (mode == 1) signal(SIGTERM, h);  // line 3 — silently dropped
    signal(SIGINT, h);                  // line 4 — this one survives
}
```

If the SIGTERM registration is the dangerous one in practice, the finding is emitted at the SIGINT line instead. This is a low-severity issue in practice (both registrations are unsafe), but the line attribution is wrong.

**Fix:** Use `Vec<(String, u32)>` or `HashMap<String, Vec<u32>>` to preserve all registration sites, and emit one finding per registration line.

---

### WR-04: CWE-427 rule fires on all `setenv`/`putenv` calls regardless of which variable is modified — high false-positive risk for safe uses

**File:** `src/vulnerability/ast_scanner.rs:240`
**Issue:** The CWE-427 rule:
```rust
AstCweRule { cwe_id: 427, functions: &["SetDllDirectoryA", "SetDllDirectoryW", "putenv", "_putenv", "setenv"], arg_check: ArgCheck::AnyCall },
```
fires on every `setenv` and `putenv` call, including calls that set variables unrelated to the dynamic linker search path (e.g., `setenv("HOME", path, 1)` or `setenv("TZ", "UTC", 1)`). CWE-427 specifically concerns search path manipulation, so only modifications to `PATH`, `LD_LIBRARY_PATH`, `DYLD_LIBRARY_PATH`, `LD_PRELOAD`, and similar loader-relevant variables are actually dangerous. This is the same false-positive pattern that produced 100% FP for CWE-426 in the Juliet benchmark, and the ANALYSIS.md confirms CWE-427 shows 0 TPs and 0 FPs on Juliet (the Juliet corpus uses `PUTENV` macro, not `setenv` directly), so the true FP rate in the field is unknown.

The test `phase_23_cwe427_setenv_fires` passes `setenv("PATH", s, 1)` and confirms a TP on the PATH variable specifically, but the rule will also fire on `setenv("TZ", "UTC", 1)`.

**Fix:** Add `ArgAtIndex(0, &["PATH", "LD_LIBRARY_PATH", "LD_PRELOAD", "DYLD_LIBRARY_PATH"])` instead of `AnyCall`, restricting the rule to calls that modify search-path-relevant environment variables. Alternatively, document the expected high FP rate in ANALYSIS.md as done for other AnyCall rules.

---

### WR-05: CWE-762 fixture `cwe762_delete_bad.c` uses C++ `delete` in a C file — tree-sitter-c may parse it as an error

**File:** `tests/fixtures/c/cwe762_delete_bad.c:6`
**Issue:** The fixture file uses `delete p;` which is C++ syntax. The file has the `.c` extension and is parsed by tree-sitter-c (the C grammar, not C++). tree-sitter-c does not understand `delete` as a keyword — it will produce a parse error on this line, which triggers `has_error()` and forces the lexical fallback path in `scan_file_ast_or_lexical`. The test `phase_23_cwe762_delete_after_calloc_fires` passes because `apply_delete_rules` is a text-level scanner invoked from the lexical fallback (via `lexical_scan_file`), but the ANALYSIS.md explicitly states that Juliet CWE-762 files trigger the `has_error()` path for the same reason ("Juliet .cpp files with `namespace` trigger has_error() → lexical fallback"). The same applies to this `.c` fixture.

This means the test does not exercise the `apply_delete_rules` function directly — it exercises the lexical fallback's text scanner. The test is effectively testing behavior via an unintended code path.

**Fix:** Either (a) rename the fixture to `cwe762_delete_bad.cpp` and add `.cpp` to `is_c_cpp_source` (it is already listed), which will still trigger `has_error()` due to `namespace`-related constructs being absent but `delete` being unknown to tree-sitter-c; or (b) add a comment in the test explaining that `delete` in a `.c` file causes a parse error and exercises the lexical fallback path intentionally. Option (b) is lower risk.

---

### WR-06: `check_block_delimitation` (CWE-483) does not check `else` branches for braceless bodies

**File:** `src/vulnerability/ast_scanner.rs:479`
**Issue:** The function only checks `node.child_by_field_name("consequence")` (the `if` body), but does not check `child_by_field_name("alternative")` (the `else` body). A braceless `else` clause is equally dangerous (same CWE-483 pattern) but produces no finding:

```c
void f(int x) {
    if (x) { x++; }
    else x--;  // braceless else — not flagged
}
```

None of the existing tests cover `else` branches. The Juliet data (93.2% FP, 20 TPs) suggests most TPs are in the `if` body, but the absence of the `else` check is a gap in coverage.

**Fix:**
```rust
if let Some(alternative) = node.child_by_field_name("alternative") {
    // alternative of if_statement can be "else_clause"
    // whose body may be a compound_statement or bare statement
    if let Some(inner) = alternative.child_by_field_name("body") {
        if inner.kind() != "compound_statement" {
            findings.push(...cwe_id: 483...);
        }
    }
}
```

---

### WR-07: `case_falls_through` treats an empty case body as fall-through, which is correct but also fires on intentional `case X: case Y:` groupings (false positives)

**File:** `src/vulnerability/ast_scanner.rs:459`
**Issue:** `case_falls_through` returns `true` when `stmts.len() < 2` (the case has no statement body beyond the case value). This fires CWE-484 on intentional case groupings:

```c
switch (x) {
    case 1:
    case 2:
        do_something();
        break;
    default: break;
}
```

`case 1:` has no statements — it intentionally falls through to `case 2:`. This is standard C style for OR-matching cases and is NOT a defect. The ANALYSIS.md shows 0 FPs for CWE-484, which is surprising for Juliet. Either Juliet does not use this pattern, or the test happened not to encounter it. In real-world codebases this will produce false positives.

The test `test_cwe484_omitted_break` only covers the case where a case body has a statement but no break, not the empty-case-group scenario.

**Fix:** Treat a case with zero statements (other than the value) as intentional fall-through grouping and suppress CWE-484 in that scenario:
```rust
fn case_falls_through(case_node: Node) -> bool {
    let mut cursor = case_node.walk();
    let stmts: Vec<Node> = case_node.named_children(&mut cursor).collect();
    // stmts[0] is the case value expression.
    // If there are no statements beyond the value, this is an intentional
    // empty-case group (case 1: case 2: do_thing(); break;) — not a fall-through bug.
    if stmts.len() < 2 {
        return false;  // empty grouping case — suppress CWE-484
    }
    let last_stmt = stmts[stmts.len() - 1];
    let k = last_stmt.kind();
    k != "break_statement" && k != "return_statement" && k != "goto_statement"
}
```

---

## Info

### IN-01: `apply_delete_rules` does not use its `_root` parameter — the function signature diverges from all other structural helpers

**File:** `src/vulnerability/ast_scanner.rs:1640`
**Issue:** All other structural helpers (`apply_signal_handler_rules`, `apply_paired_lock_rules`) use the `root: Node` parameter to traverse the AST. `apply_delete_rules` prefixes it with `_` because the implementation is a raw byte scan over `src` directly. This makes the function inconsistent with the helper API contract and means the function cannot leverage AST structure (see WR-01). The inconsistency is also a code quality signal — reviewers may not notice the `_root` prefix and assume AST traversal is happening.

**Fix:** Add a comment directly on the `_root` parameter explaining why it is unused, and consider whether future work should replace the text scan with an AST-based approach.

---

### IN-02: No false-negative test for CWE-479 when the signal handler is defined in a different file (cross-file gap)

**File:** `tests/vulnerability_tests/ast_scanner_tests.rs:907`
**Issue:** `apply_signal_handler_rules` only scans root-level `function_definition` nodes in the same file as the `signal()` call. If `signal(SIGINT, handler_from_other_file)` appears in `main.c` and the handler is defined in `signals.c`, the two-pass logic will find no matching `function_definition` for `handler_from_other_file` and emit no finding. This is a documented limitation (single-file analysis is inherent to the design), but no test explicitly documents this boundary condition as expected behavior.

**Fix:** Add a TN test that calls `signal(2, externalHandler)` with no `externalHandler` definition in the same file, asserting no CWE-479 finding and adding a comment explaining this is expected due to single-file scope.

---

### IN-03: ANALYSIS.md CWE-762 FP gate violation is noted but the recommended action is imprecise

**File:** `benchmark/juliet/ANALYSIS.md:419`
**Issue:** The recommended action for CWE-762's 58.5% FP rate states "tighten to require co-occurrence with malloc/calloc/realloc in same file." However, `apply_delete_rules` fires on any `delete` token in any C/C++ file that passed tree-sitter's `has_error()` check — or that triggered the lexical fallback. The co-occurrence approach only reduces FPs when `malloc`/`calloc` are absent in the file; it does not address the root cause (text-level `delete` keyword in a C++ file with destructors, assignment operators, etc. that legitimately use `delete` without any C allocator). A more accurate recommendation would be "restrict to files that also contain malloc/calloc/realloc call sites, AND do not have a namespace declaration," which is closer to the actual pattern the rule is trying to detect.

**Fix:** Update the recommended action in the FP Gate Violations table to be more precise.

---

_Reviewed: 2026-05-12T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
