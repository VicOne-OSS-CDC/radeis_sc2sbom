# Phase 22: ast-cwes-structuralPattern-expansion - Pattern Map

**Mapped:** 2026-05-12
**Files analyzed:** 2 (1 modified source file + 1 modified benchmark doc)
**Analogs found:** 2 / 2

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/vulnerability/ast_scanner.rs` | service / utility | transform (AST → findings) | `src/vulnerability/ast_scanner.rs` (existing functions) | exact — all new code extends this file |
| `benchmark/juliet/ANALYSIS.md` | config / doc | — | `benchmark/juliet/ANALYSIS.md` (existing table) | exact — append rows to existing table |

---

## Pattern Assignments

### `src/vulnerability/ast_scanner.rs` — 15 new `check_*` private functions + `apply_ast_rules` extension

All new code lives in this single file under `#[cfg(feature = "internal")]`. No new files, no new modules (D-02).

---

#### Pattern A: Structural Visitor Function Signature

Every new `check_*` function follows the signature established by `visit_node` and `collect_arrays_in_subtree` (lines 188–294 and 349–367).

**Analog — recursive visitor with mutable-ref accumulator** (`ast_scanner.rs`, lines 188–294):

```rust
fn visit_node<'a>(
    node: Node<'a>,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    file_scope_arrays: &HashSet<String>,
    findings: &mut Vec<SastFinding>,
) {
    // ... match logic ...

    // Recurse into children (Pitfall 1: fresh cursor per call level)
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            visit_node(
                cursor.node(),
                src,
                path,
                component_name,
                component_ecosystem,
                file_scope_arrays,
                findings,
            );
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}
```

**Simplified signature for new check_* functions** (no `file_scope_arrays` needed for structural checks):

```rust
fn check_X(
    node: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "TARGET_NODE_KIND" {
        // inspect node — emit SastFinding if condition met
    }
    // Recurse — fresh cursor per call level (Pitfall 1)
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            check_X(cursor.node(), src, path, component_name, component_ecosystem, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

---

#### Pattern B: SastFinding Construction

**Analog** (`ast_scanner.rs`, lines 262–270):

```rust
findings.push(SastFinding {
    cwe_id: rule.cwe_id,
    component_name: component_name.to_string(),
    component_ecosystem: component_ecosystem.to_string(),
    file_path: path.to_string_lossy().into_owned(),
    line: (node.start_position().row as u32) + 1,
    source: SastSource::Ast,
});
```

All Phase 22 findings use `SastSource::Ast`. Line is always `node.start_position().row + 1`. Copy this pattern verbatim for every `findings.push(...)` in Phase 22.

---

#### Pattern C: `apply_ast_rules` Integration Point

**Analog** (`ast_scanner.rs`, lines 163–185):

```rust
fn apply_ast_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding> {
    let file_scope_arrays = collect_file_scope_fixed_arrays(root, src);

    let mut findings = Vec::new();
    visit_node(root, src, path, component_name, component_ecosystem, &file_scope_arrays, &mut findings);
    // Phase 22: add calls here, after existing visit_node:
    // check_switch_structure(root, src, path, component_name, component_ecosystem, &mut findings);
    // check_assignment_in_condition(root, src, path, component_name, component_ecosystem, &mut findings);
    // ... (one line per check_* function)
    findings
}
```

Each new `check_*` call is appended after the existing `visit_node(...)` call. The mutable-ref pattern (`&mut findings`) is used because the check functions recurse internally.

---

#### Pattern D: Collecting Names from a Subtree (for CWE-562, CWE-674)

**Analog** — `collect_function_scope_fixed_arrays` + `collect_arrays_in_subtree` (`ast_scanner.rs`, lines 343–367):

```rust
fn collect_function_scope_fixed_arrays(fn_node: Node, src: &[u8]) -> HashSet<String> {
    let mut result = HashSet::new();
    collect_arrays_in_subtree(fn_node, src, &mut result);
    result
}

fn collect_arrays_in_subtree(node: Node, src: &[u8], out: &mut HashSet<String>) {
    if node.kind() == "declaration" {
        collect_array_declarators(node, src, out);
    }
    let mut cursor = node.walk(); // fresh cursor per level (Pitfall 1)
    if cursor.goto_first_child() {
        loop {
            collect_arrays_in_subtree(cursor.node(), src, out);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

**For CWE-562 / CWE-674:** Copy this two-function pattern (public entry + private recursive helper). The entry function collects names into a `HashSet<String>` and the recursive helper walks the subtree with a fresh cursor per level.

---

#### Pattern E: `find_enclosing_function` (reuse as-is)

**Analog** (`ast_scanner.rs`, lines 297–307):

```rust
fn find_enclosing_function(node: Node) -> Option<Node> {
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind() == "function_definition" {
            return Some(n);
        }
        current = n.parent();
    }
    None
}
```

**CWE-674 usage:** Call `find_enclosing_function(call_expr_node)` to get the function that contains a `call_expression`. Alternatively, walk top-level `function_definition` nodes from root directly (the RESEARCH.md CWE-674 example does root-level iteration, which is cleaner for this case).

---

#### Pattern F: Field-Based Child Access

**Analog** (`ast_scanner.rs`, lines 199–207):

```rust
if let Some(func_node) = node.child_by_field_name("function") {
    if let Ok(func_name) = func_node.utf8_text(src) {
        // ...
    }
}

let args: Vec<Node> = if let Some(arg_list) = node.child_by_field_name("arguments") {
    let mut cursor = arg_list.walk();
    arg_list.named_children(&mut cursor).collect()
} else {
    vec![]
};
```

Always use `child_by_field_name("field")` for named fields. Use `child(i)` loops only for unnamed children (e.g., iterating case body statements). This is the Phase 18 established pattern.

---

#### Pattern G: AnyCall Rule in AST_CWE_RULES (for CWE-617)

**Analog** (`ast_scanner.rs`, lines 51, 58–59, 74):

```rust
AstCweRule { cwe_id: 78,  functions: &["system", "popen", ...], arg_check: ArgCheck::AnyCall },
AstCweRule { cwe_id: 190, functions: &["malloc", "calloc", "realloc"], arg_check: ArgCheck::AnyCall },
AstCweRule { cwe_id: 242, functions: &["gets", "mktemp"], arg_check: ArgCheck::AnyCall },
AstCweRule { cwe_id: 377, functions: &["tmpnam", "tempnam", "mktemp"], arg_check: ArgCheck::AnyCall },
```

CWE-617 (`assert`) is a call-site pattern, making it eligible for `AST_CWE_RULES` as `AnyCall` — the same table entry pattern used for CWE-78, 190, 242, 377. This is the minimal implementation. The planner should add:

```rust
AstCweRule { cwe_id: 617, functions: &["assert"], arg_check: ArgCheck::AnyCall },
```

This fires via the existing `visit_node` call — no dedicated visitor function needed for CWE-617.

---

#### Pattern H: Collecting Children for Inspection (node.children)

**Analog** (`ast_scanner.rs`, lines 371–376 in `collect_array_declarators`):

```rust
fn collect_array_declarators(node: Node, src: &[u8], out: &mut HashSet<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_array_declarator_rec(child, src, out);
        }
    }
}
```

For inspecting all children of a node (e.g., checking whether a `switch_statement` body contains a `default_case`), use a local cursor + `node.children(&mut cursor)` or the `for i in 0..node.child_count()` loop. The cursor must be created locally: `let mut cursor = node.walk();`.

---

### `benchmark/juliet/ANALYSIS.md` — append 15 new per-CWE rows

**Analog** (`benchmark/juliet/ANALYSIS.md`, lines 31–77 — existing Per-CWE TP/FP Table):

```markdown
| CWE | AST TPs | AST FPs | AST FP% | Lexical TPs | Lexical FPs | Lexical FP% | cppcheck TPs | cppcheck FPs | cppcheck FP% |
|-----|---------|---------|---------|-------------|-------------|-------------|--------------|--------------|--------------|
| CWE-256 | [N] | [N] | [%]% | 0 | 0 | — | 0 | 0 | — |
| CWE-398 | [N] | [N] | [%]% | 0 | 0 | — | 96 | 138158 | 99.9% |
...
```

After implementing and running the Juliet benchmark (D-14), append one row per Phase 22 CWE to the existing table. Fill in AST TPs and AST FPs from the run; set Lexical and cppcheck columns to the existing values from the table (or `0`/`—` if not measured).

---

## Shared Patterns

### Feature Gate
**Source:** `ast_scanner.rs`, line 13
**Apply to:** All Phase 22 code
```rust
#![cfg(feature = "internal")]
```
Every function added in Phase 22 must live inside this feature gate. The entire file is gated; no per-function annotation is needed since the module-level attribute covers everything.

### Imports (no new imports needed)
**Source:** `ast_scanner.rs`, lines 15–22
```rust
use crate::util::warn_on_walkdir_err;
use crate::vulnerability::cwe_scanner::scan_file as lexical_scan_file;
use crate::vulnerability::cwe_scanner::token_present_with_boundary;
use crate::vulnerability::{SastFinding, SastSource};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};
use walkdir::WalkDir;
```
`HashSet<String>` is already imported. No new `use` statements are needed for Phase 22 (all types already in scope).

### Fresh Cursor Per Call Level (Pitfall 1)
**Source:** `ast_scanner.rs`, lines 277–293, 331–339, 358–366
**Apply to:** Every recursive `check_*` function
```rust
// CORRECT: fresh cursor at each recursion level
let mut cursor = node.walk();
if cursor.goto_first_child() {
    loop {
        recursive_fn(cursor.node(), ...);
        if !cursor.goto_next_sibling() { break; }
    }
}
```
Never pass a cursor as a parameter or reuse one across call levels.

### Test Helper: `setup_one_file`
**Source:** `tests/vulnerability_tests/ast_scanner_tests.rs`, lines 11–17
```rust
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```
**Apply to:** All 15 new unit test functions in `ast_scanner_tests.rs`. Each test: one TP fixture (bad C code) + one TN fixture (good variant). Use `run_ast_scanner(&dirs)` and filter by `cwe_id`.

### Test Assertion Pattern
**Source:** `tests/vulnerability_tests/ast_scanner_tests.rs`, lines 21–30
```rust
#[test]
fn test_cwe_NNN_detected() {
    let (_t, dirs) = setup_one_file("a.c", b"/* bad C code */\n");
    let findings: Vec<SastFinding> = run_ast_scanner(&dirs);
    assert!(
        findings.iter().any(|f| f.cwe_id == NNN && f.source == SastSource::Ast),
        "Expected CWE-NNN with SastSource::Ast; got {:?}", findings
    );
}
```

---

## No Analog Found

All Phase 22 files have direct analogs in the codebase. No files without a close match.

---

## Per-CWE Implementation Notes (for Planner)

These notes summarize the exact analog functions to copy per CWE group, based on the patterns above.

| CWE(s) | Analog Pattern | Key Distinction |
|---|---|---|
| 478, 484 (switch structure) | Pattern A + B + H | Walk `switch_statement` children; check presence of `default_case` (CWE-478) and `break_statement` as last statement in each `case_statement` (CWE-484) |
| 481, 482, 480 (operator errors) | Pattern A + B + F | Walk `if_statement`/`while_statement` conditions; check top-level node kind: `assignment_expression` for CWE-481, `binary_expression` with `==` at `expression_statement` level for CWE-482 |
| 483 (block delimitation) | Pattern A + B + F | Walk `if_statement`; check `child_by_field_name("consequence")` kind is NOT `compound_statement` |
| 562 (return stack addr) | Pattern A + B + D + E | Per `function_definition`: collect non-static local array names (Pattern D); check `return_statement` child `identifier` is in that set |
| 570, 571 (constant condition) | Pattern A + B + F | Walk `if_statement`/`while_statement` conditions; check condition node kind is `number_literal` directly, OR `binary_expression` where both children are `number_literal` |
| 587 (fixed address) | Pattern A + B | Walk `init_declarator`; check initializer is `cast_expression` or `number_literal` with hex text > 0xFFFF |
| 617 (assert) | Pattern G (AnyCall) | Add one entry to `AST_CWE_RULES`; no dedicated visitor needed |
| 674 (self-recursion) | Pattern A + B + E | Walk `function_definition` nodes from root; extract function name via `function_declarator` child; walk body for `call_expression` matching the function name (Pattern E / D-04) |
| 835 (infinite loop) | Pattern A + B + H | Walk `while_statement` / `for_statement`; check condition is literal `1` or empty; check body has no `break_statement` / `return_statement` / `goto_statement` / `exit` call |
| 256 (plaintext password) | Pattern A + B + F | Walk `declaration` nodes; check declarator identifier name contains heuristic keywords (case-insensitive); check initializer is `string_literal` |
| 398 (poor code quality) | Pattern A + B | Walk `expression_statement` nodes; check child is bare `number_literal`, arithmetic `binary_expression`, `==` `binary_expression`, or self-assignment `assignment_expression` |

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `tests/vulnerability_tests/`, `benchmark/juliet/`
**Files scanned:** 3 (`ast_scanner.rs`, `ast_scanner_tests.rs`, `ANALYSIS.md`)
**Pattern extraction date:** 2026-05-12
