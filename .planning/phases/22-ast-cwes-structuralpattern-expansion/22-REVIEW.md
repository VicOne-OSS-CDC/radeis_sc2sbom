---
phase: 22-ast-cwes-structuralpattern-expansion
reviewed: 2026-05-12T08:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - src/vulnerability/ast_scanner.rs
  - tests/vulnerability_tests/ast_scanner_tests.rs
  - benchmark/juliet/ANALYSIS.md
findings:
  critical: 0
  warning: 7
  info: 3
  total: 10
status: issues_found
---

# Phase 22: Code Review Report

**Reviewed:** 2026-05-12T08:00:00Z
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

Phase 22 added 15 new CWE structural-detection rules across three plan batches (Plans 01–03).
The new check functions cover switch structure (CWE-478/484), block delimitation (CWE-483),
assignment/comparison confusion (CWE-481/482), function pointer comparison (CWE-480), return
of stack address (CWE-562), constant conditions (CWE-570/571), fixed-address assignment
(CWE-587), self-recursion (CWE-674), plaintext password (CWE-256), infinite loop (CWE-835),
and poor code quality (CWE-398), plus CWE-617 via the existing AnyCall path.

The overall structure is sound and follows the established patterns for tree-sitter-c traversal.
However, several logic defects in the new check functions will produce missed findings (false
negatives) or unintended false positives in realistic code. None rise to the level of security
vulnerabilities, but several are correctness bugs that affect the accuracy of the scanner.
Test coverage for the new rules is largely TP-only with notable FP-guard gaps.

---

## Warnings

### WR-01: `check_switch_structure` recurses into the switch body it already inspected, causing double-fires on nested switches

**File:** `src/vulnerability/ast_scanner.rs:418-428`

The function processes `node` when it is a `switch_statement`, then **always** recurses into
all children via cursor, including `body`. If a switch is nested inside another switch, the
outer call processes the outer switch and then recurses — eventually reaching the inner switch
via the body children, which is correct. However, the outer call processes the outer switch
body (lines 377–416) **and** then recurses into that same body via the `goto_first_child` loop
(line 420). This means the outer switch's own body children (which include the inner switch)
are visited both during the "process this node" step and again during recursion. The net effect:
a nested `switch` without a default case produces **two** CWE-478 findings — one from the inner
switch's own recursive call and one from the outer call's recursion walking into the body. This
is the same double-fire pattern that other similar SAST tools encounter with recursive visitors.

The correct fix is to skip re-processing the body during recursion. The simplest approach is
to not recurse into the `body` child once it has been inspected, or to use a pre-order guard
(process only if not already dispatched from a parent switch call). The established pattern in
the codebase is that the recursion at the bottom of each check function is only for finding
the *next* occurrence of the triggering node kind, not for re-visiting children already handled.
For `check_switch_structure` this is harmless today only because the test fixture never nests
switches, but real-world firmware code frequently uses nested switches.

**Fix:**
```rust
// After processing the switch body, recurse only into non-body children.
// Body was already fully inspected above.
let mut cursor = node.walk();
if cursor.goto_first_child() {
    loop {
        let child = cursor.node();
        // Skip the body compound_statement — already walked above.
        if child.kind() != "compound_statement" {
            check_switch_structure(
                child, src, path, component_name, component_ecosystem, findings,
            );
        } else {
            // Still recurse INTO the body's children so nested switches are found,
            // but don't pass the body node itself (which would be the outer body again).
            let mut body_cur = child.walk();
            if body_cur.goto_first_child() {
                loop {
                    check_switch_structure(
                        body_cur.node(), src, path, component_name, component_ecosystem, findings,
                    );
                    if !body_cur.goto_next_sibling() { break; }
                }
            }
        }
        if !cursor.goto_next_sibling() { break; }
    }
}
```

---

### WR-02: `check_switch_structure` CWE-478 detection uses incorrect tree-sitter-c grammar assumption for `default_case`

**File:** `src/vulnerability/ast_scanner.rs:386-388`

The code detects the `default:` case by checking for a `case_statement` node whose `value`
field is `None`:
```rust
let has_default = body_children.iter().any(|c| {
    c.kind() == "case_statement" && c.child_by_field_name("value").is_none()
});
```

In tree-sitter-c (grammar version 0.21+), `default:` is represented as a distinct
**`default_case`** node kind — **not** as a `case_statement` with no value. Checking
`case_statement` with no `value` field will never match a `default_case` node, causing
`has_default` to always be `false` whenever a `default:` is present. Every switch with a
`default:` case will fire CWE-478 as a false positive.

The ANALYSIS.md benchmark reports 73.9% FP for CWE-478 (18 TPs, 51 FPs), which is consistent
with this bug — the FPs are switches that actually have a `default:` but the check fails to
recognize it. The Juliet TN test `test_cwe478_switch_with_default_no_finding` also likely only
passes because tree-sitter-c happens to represent the `default:` in a way the test happens to
catch via the `case_statement` check, or the test fixture is too narrow to expose this edge.

**Fix:**
```rust
let has_default = body_children.iter().any(|c| {
    c.kind() == "default_case"
        || (c.kind() == "case_statement" && c.child_by_field_name("value").is_none())
});
```

This handles both grammar representations across tree-sitter-c versions.

---

### WR-03: `check_return_stack_address` (CWE-562) does not skip the function body when recursing, causing it to re-collect local vars from nested function definitions (e.g., lambdas in C++)

**File:** `src/vulnerability/ast_scanner.rs:779-787`

After the CWE-562 logic runs on a `function_definition`, the tail recursion at line 779
visits **all** children of that function_definition, including the body. The body itself is
then walked by `check_return_stmts_in_subtree` (line 774) for return statements. But the
tail recursion also walks the body's children looking for nested `function_definition` nodes
(e.g., nested functions in GNU C extensions, or lambda closures in C++). When a nested
function_definition is found, `collect_local_var_names` is called on it — but the nested
function's local variables are added to the **outer** function's `local_names` set, so a
`return x` in the outer function may fire CWE-562 if `x` happens to be a local variable
in the nested function (even though `x` in the outer scope is a different entity).

This is a correctness bug that causes false positives when GNU nested functions or C++
lambdas are present in the scanned code.

**Fix:** After calling `check_return_stmts_in_subtree` on the current `fn_node`, the tail
recursion should skip the entire body of the current function_definition to avoid re-entering
the function that was already processed. The `check_return_stack_address` recursion is meant
only to find additional top-level function definitions (siblings), not to re-walk the current
body.

```rust
// Recurse — fresh cursor per call level (Pitfall 1)
// Skip the body of this function_definition to avoid re-processing it.
let mut cursor = node.walk();
if cursor.goto_first_child() {
    loop {
        let child = cursor.node();
        // Only recurse into non-body children for the same reasons as WR-01.
        if node.kind() == "function_definition" {
            if child.kind() != "compound_statement" {
                check_return_stack_address(
                    child, src, path, component_name, component_ecosystem, findings,
                );
            }
        } else {
            check_return_stack_address(
                child, src, path, component_name, component_ecosystem, findings,
            );
        }
        if !cursor.goto_next_sibling() { break; }
    }
}
```

---

### WR-04: `check_self_recursion` only inspects root-level `function_definition` children, silently missing functions inside `preproc_ifdef` / `preproc_if` blocks

**File:** `src/vulnerability/ast_scanner.rs:1048-1056`

```rust
let root_children: Vec<Node> = root.children(&mut cursor).collect();
for fn_node in root_children.into_iter().filter(|n| n.kind() == "function_definition") {
```

Only direct children of `translation_unit` (root) are examined. Functions guarded by
`#ifdef`, `#if`, `#ifndef`, `#else`, or `#elif` preprocessor blocks appear inside
`preproc_ifdef` / `preproc_if` nodes, not as direct root children. The ANALYSIS.md documents
this at D-13: "Juliet wraps `helperBad()` inside `#ifndef OMITBAD` preprocessor guards...
so the function_definition is nested inside preproc_ifdef and not visible at root level."

The bug is acknowledged in the analysis as "future work," but it means **all real-world code
that guards functions with `#ifdef` or `#if` will silently produce zero CWE-674 findings**,
even for obvious direct self-recursion. The Juliet corpus shows 0 TPs as a direct consequence.
In production firmware code, virtually every conditionally-compiled module uses preprocessor
guards, making this rule essentially inoperative on real code.

**Fix:** Recursively walk preproc_ifdef/preproc_if children when searching for
function_definition nodes, rather than limiting to direct root children:
```rust
fn check_self_recursion(root: Node, src: &[u8], path: &Path,
    component_name: &str, component_ecosystem: &str, findings: &mut Vec<SastFinding>) {
    collect_and_check_self_recursion(root, src, path, component_name, component_ecosystem, findings);
}

fn collect_and_check_self_recursion(node: Node, src: &[u8], path: &Path,
    component_name: &str, component_ecosystem: &str, findings: &mut Vec<SastFinding>) {
    if node.kind() == "function_definition" {
        if let Some(fn_name) = extract_function_name(node, src) {
            check_self_calls(node, src, &fn_name, path, component_name, component_ecosystem, findings);
        }
        return; // Don't recurse into function body for nested function_definitions
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            collect_and_check_self_recursion(
                cursor.node(), src, path, component_name, component_ecosystem, findings,
            );
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

---

### WR-05: `check_constant_condition` (CWE-570/571) fires on `while(1)` loops, double-firing alongside `check_infinite_loop` (CWE-835) — interplay not documented, CWE-571 finding emitted for every infinite loop

**File:** `src/vulnerability/ast_scanner.rs:863-927`

`check_constant_condition` fires CWE-571 for any `while_statement` or `for_statement` whose
condition is a non-zero literal. `check_infinite_loop` fires CWE-835 for a `while(1)` or
`for(;;)` body with no escape. Both fire independently. This means `while(1) { /* no break */ }`
emits **two** findings: CWE-571 (always-true condition) and CWE-835 (unreachable exit).

This is not incorrect per se — both CWEs are legitimately applicable — but the behavior is
undocumented and may surprise consumers who see two findings on the same line. More importantly,
`check_constant_condition` fires CWE-571 on `while(1) { break; }` (a common and intentional
idiom for structured loops), even though `check_infinite_loop` correctly suppresses CWE-835 in
that case via the body-check. So CWE-571 fires on intentional loop patterns, contributing to
the 100.0% FP rate documented in the analysis.

The design decision comment at D-06 says "variable patterns deferred" but does not address the
interaction between the two rules, leaving users to see CWE-571 on every `while(1)`.

**Fix (minimal):** Suppress CWE-571 for `while_statement` / `for_statement` contexts (where
`while(1)` is a common idiom) and only fire CWE-571 for `if_statement` contexts. Or add an
explicit note in the doc comment that CWE-571 deliberately overlaps with CWE-835 for loop
patterns.

Alternatively, exclude the `while_statement` and `for_statement` node kinds from
`check_constant_condition` when the non-zero literal is `1` (the canonical infinite-loop pattern):
```rust
// Don't emit CWE-571 for while(1)/for — that's CWE-835's domain.
let is_loop = matches!(node.kind(), "while_statement" | "for_statement");
let cwe_id = if val == 0 { 570 } else if is_loop && val == 1 { return } else { 571 };
```

---

### WR-06: `check_poor_code_quality` (CWE-398) sub-rule 2 fires on discarded binary expressions including `!=`, `<`, `>`, `<=`, `>=` operators, but the listed operators in the code only include `==`, causing inconsistent behavior with sub-rule 3

**File:** `src/vulnerability/ast_scanner.rs:1317-1323`

Sub-rule 3 of `check_poor_code_quality` fires on `==` at statement level, which is also
CWE-482 (comparison instead of assignment). Sub-rule 2 is documented as "discarded arithmetic"
with operators `(+,-,*,/,%,|,&,^)`. The `==` operator is included in sub-rule 2's `matches!`
check (line 1320), meaning a discarded `==` expression fires as CWE-398 via sub-rule 2 AND
as CWE-482 via `check_comparison_at_statement`. This is intentional per the comment in the
code, but the test `test_cwe482_comparison_at_statement_level` (with `x == 5;`) will also
produce a CWE-398 finding — the test does not guard against this, and downstream consumers
may see two findings on `x == 5;`.

More critically, discarded comparisons with `!=`, `<`, `>`, `<=`, `>=` operators are **not**
flagged by either sub-rule 2 (which lists only arithmetic operators and `==`) or CWE-482
(which checks only `==`). The expression `a != b;` at statement level is equally a "discarded
comparison" but produces no finding. This is an undocumented gap in coverage.

**Fix:** Either add the relational operators to sub-rule 2:
```rust
if matches!(op, "+" | "-" | "*" | "/" | "%" | "|" | "&" | "^"
              | "==" | "!=" | "<" | ">" | "<=" | ">=") {
```
Or document the intentional exclusion of relational operators in the function doc comment.

---

### WR-07: `check_plaintext_password` (CWE-256) keyword `"pwd"` will fire on short unrelated variable names like `current_pwd` (current working directory) — no word-boundary guard

**File:** `src/vulnerability/ast_scanner.rs:1129-1131`

The password heuristic uses `contains`:
```rust
let is_password_name = lower.contains("password")
    || lower.contains("passwd")
    || lower.contains("pwd")
    || lower.contains("secret");
```

The substring `"pwd"` matches any variable whose lowercase name contains "pwd" anywhere:
- `current_pwd` — common for "current working directory"
- `fwd` (contains "wd" but not "pwd" — this one is safe)
- `upwd` — ambiguous
- `cpwd` — ambiguous

The ANALYSIS.md reports 100.0% FP for CWE-256 in the Juliet corpus (1,056 FPs, 0 TPs),
partly attributed to corpus mismatch, but the substring match on `"pwd"` adds noise on
real-world codebases where `pwd`-named variables often mean "working directory" in system
code. The `"secret"` keyword similarly matches `secretariat`, `secretion`, etc.

**Fix:** Add a word-boundary check (matching the established `token_present_with_boundary`
pattern used elsewhere in the codebase) instead of bare `contains`:
```rust
let is_password_name = token_present_with_boundary(&lower, "password")
    || token_present_with_boundary(&lower, "passwd")
    || token_present_with_boundary(&lower, "pwd")
    || token_present_with_boundary(&lower, "secret");
```

Or restrict `"pwd"` to whole-name match: `lower == "pwd" || lower.ends_with("_pwd") || lower.starts_with("pwd_")`.

---

## Info

### IN-01: `case_falls_through` treats empty case bodies (no statements) as falling through, but does not detect cases whose only content is another nested case (chained cases)

**File:** `src/vulnerability/ast_scanner.rs:438-446`

```rust
if stmts.len() < 2 {
    // Empty case body — falls through
    return true;
}
```

A chained case like `case 1: case 2: do_thing(); break;` is represented in tree-sitter-c as a
`case_statement` whose body contains another `case_statement` (not a break/return). This will
correctly be detected as falling through since the last named child is not
break/return/goto. However, the `stmts.len() < 2` early return is hit when the case body has
only the `value` child with zero statements — this is the **intentional empty fallthrough**
pattern (e.g., `case 1: case 2: break;`). The comment says "falls through" but this is
actually the intended `case 1: /* fall to case 2 */` pattern. Flagging it as CWE-484 produces
a FP for the common intentional fallthrough pattern. There is no test for this edge case.

**Fix:** No code change required for correctness in isolation (intentional fallthrough is still
a CWE-484 finding by the CWE definition). Document the known FP for intentional chained cases
in the function doc comment.

---

### IN-02: `check_block_delimitation` (CWE-483) does not check the `else` branch — `if(x) { ... } else y++;` fires only on the `if` side

**File:** `src/vulnerability/ast_scanner.rs:460-473`

The `check_block_delimitation` function checks only the `consequence` field of `if_statement`:
```rust
if let Some(consequence) = node.child_by_field_name("consequence") {
    if consequence.kind() != "compound_statement" {
```

An `else` clause without braces (`if(x) { ... } else y++;`) also violates CWE-483 but is
not detected because the `alternative` field is not inspected. The benchmark shows 93.2% FP,
suggesting the rule already fires broadly; adding the else check would slightly increase TPs
on real code.

**Fix:**
```rust
if let Some(alternative) = node.child_by_field_name("alternative") {
    // "else_clause" in tree-sitter-c wraps the else; its body is the statement.
    // Check if the else body is not a compound_statement.
    if alternative.kind() != "compound_statement" {
        // Check inner body of else_clause if present
        let else_body = alternative.child_by_field_name("body")
            .unwrap_or(alternative);
        if else_body.kind() != "compound_statement" {
            findings.push(/* CWE-483 */);
        }
    }
}
```

---

### IN-03: `check_infinite_loop` does not handle `do { ... } while(1);` — a common infinite loop pattern in embedded C

**File:** `src/vulnerability/ast_scanner.rs:1231-1283`

The function handles `while_statement` and `for_statement` but silently skips
`do_statement`. The `do { ... } while(1);` pattern (with no break/return in the body) is a
well-established C infinite loop idiom used in embedded firmware and is defined as CWE-835.
The body-check approach (D-05) is already implemented and `body_has_escape` would work
identically for `do_statement` bodies. The omission produces a false negative on all
do-while infinite loops.

**Fix:**
```rust
} else if node.kind() == "do_statement" {
    if let Some(cond) = node.child_by_field_name("condition") {
        let inner = unwrap_parens(cond);
        let is_literal_nonzero = (inner.kind() == "number_literal"
            || inner.kind() == "integer_literal")
            && inner.utf8_text(src).ok()
                .and_then(|t| parse_c_integer_literal(t))
                .map(|v| v != 0)
                .unwrap_or(false);
        if is_literal_nonzero {
            if let Some(body) = node.child_by_field_name("body") {
                if !body_has_escape(body, src) {
                    findings.push(SastFinding { cwe_id: 835, ... });
                }
            }
        }
    }
}
```

---

_Reviewed: 2026-05-12T08:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
