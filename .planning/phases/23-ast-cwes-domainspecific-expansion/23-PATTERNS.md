# Phase 23: ast-cwes-domainSpecific-expansion - Pattern Map

**Mapped:** 2026-05-12
**Files analyzed:** 3 (1 primary modify, 1 test file modify, 1 new fixture)
**Analogs found:** 3 / 3

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/ast_scanner.rs` | service / rule-engine | transform (AST → findings) | itself (existing file, adding to it) | exact |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | test | request-response | itself (existing test file, extending) | exact |
| `tests/fixtures/c/cwe762_delete_bad.c` (new) | config / fixture | — | `tests/fixtures/c/dangerous_calls.c` | role-match |

---

## Pattern Assignments

### `src/vulnerability/ast_scanner.rs` (rule-engine, transform)

**Analog:** itself — the file is the only target; all additions follow existing patterns within it.

---

#### Pattern A: New AstCweRule table entries (CWE-114, 272, 284, 427, 785)

**Source:** `src/vulnerability/ast_scanner.rs` lines 53–84 (existing `AST_CWE_RULES`)

Append after the last existing rule (currently `SetSecurityDescriptorDacl` for CWE-732, line 82).
Follow the identical struct-literal style: one entry per line, `cwe_id`, `functions`, `arg_check`.

```rust
// Append to AST_CWE_RULES (after line 83, before the closing `];`)

// CWE-114: Process Control via dynamic library load (Win32-specific)
AstCweRule { cwe_id: 114, functions: &["LoadLibraryA", "LoadLibraryW", "LoadLibraryExA", "LoadLibraryExW"], arg_check: ArgCheck::AnyCall },
// CWE-272: Least Privilege Violation via privileged process creation (Win32-specific)
AstCweRule { cwe_id: 272, functions: &["CreateProcessAsUserA", "CreateProcessAsUserW"], arg_check: ArgCheck::AnyCall },
// CWE-284: Improper Access Control via over-privileged desktop creation (Win32-specific)
AstCweRule { cwe_id: 284, functions: &["CreateDesktopA", "CreateDesktopW"], arg_check: ArgCheck::ArgAtIndex(4, &["GENERIC_ALL"]) },
// CWE-427: Uncontrolled Search Path Element
AstCweRule { cwe_id: 427, functions: &["SetDllDirectoryA", "SetDllDirectoryW", "putenv", "_putenv", "setenv"], arg_check: ArgCheck::AnyCall },
// CWE-785: Path manipulation function without MAX_PATH buffer (Win32-specific + POSIX)
AstCweRule { cwe_id: 785, functions: &["PathAppendA", "PathAppendW", "realpath", "_fullpath"], arg_check: ArgCheck::AnyCall },
```

**ArgAtIndex arm reference** (lines 233–254): The existing `ArgAtIndex` arm in `visit_node()` already handles identifier-text subtree collection via `collect_subtree_text()` and `token_present_with_boundary()`. CWE-284 uses `ArgAtIndex(4, &["GENERIC_ALL"])` — index 4 is the 5th argument (0-based). The existing arm walks the arg subtree and calls `tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))`. No new code needed in the arm — only the new table entry.

**`SastFinding` construction pattern** (lines 280–288):
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
All new helper functions must construct `SastFinding` with this exact field set.

---

#### Pattern B: `apply_signal_handler_rules()` — CWE-479

**Model analog:** `visit_node()` recursive walk pattern (lines 194–312) — same cursor discipline.
**Sub-pattern:** `collect_arrays_in_subtree()` / `collect_file_scope_arrays_rec()` (lines 367–385) — the fresh-cursor-per-level recursion pattern used by all subtree collectors.

**Function signature** — must match `apply_division_rules()` shape (D-08/D-09, confirmed by Phase 21 context):
```rust
fn apply_signal_handler_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding>
```

**Pass 1 — collect `signal()` call sites:**

Walk all `call_expression` nodes. When `function` field text == `"signal"`, extract the `arguments` named children, take index 1 (the handler arg), and if it is an `identifier` node, store `(handler_name_string → signal_call_line)`. Use `HashMap<String, u32>`.

```rust
// Fresh-cursor recursion pattern (from collect_arrays_in_subtree, lines 376-384):
let mut cursor = node.walk(); // fresh cursor per level
if cursor.goto_first_child() {
    loop {
        // recurse on cursor.node()
        if !cursor.goto_next_sibling() { break; }
    }
}
```

**Named-children collection pattern** (from `visit_node` lines 208–213):
```rust
let args: Vec<Node> = if let Some(arg_list) = node.child_by_field_name("arguments") {
    let mut cursor = arg_list.walk();
    arg_list.named_children(&mut cursor).collect()
} else {
    vec![]
};
```

**Pass 2 — find function definitions and scan bodies:**

Walk root for `function_definition` nodes. For each, check the `function_declarator` subtree for an `identifier` whose text matches a collected handler name. If found, walk that function's body for `call_expression` nodes whose `function` field text is in the `NON_REENTRANT` set. If any match, emit one `SastFinding` at the `signal()` call site line (from pass 1 map), NOT at the non-reentrant call line.

```rust
// Non-reentrant function set:
const NON_REENTRANT: &[&str] = &[
    "malloc", "free", "printf", "fprintf", "sprintf", "snprintf",
    "vprintf", "vfprintf", "exit", "abort", "syslog",
];
```

**Finding line:** Use the line stored in pass 1 (`signal_call_line`), not the line of the non-reentrant call in the handler body. This matches the Juliet oracle and RESEARCH.md Pitfall 3.

**Call site** in `scan_file_ast_or_lexical()` (line 160–167, current):
```rust
// Current call (lines 160-167):
apply_ast_rules(root, code.as_bytes(), path, component_name, component_ecosystem)

// New pattern after Phase 23 (return Vec, extend):
let mut findings = apply_ast_rules(root, src, path, component_name, component_ecosystem);
findings.extend(apply_signal_handler_rules(root, src, path, component_name, component_ecosystem));
findings.extend(apply_paired_lock_rules(root, src, path, component_name, component_ecosystem));
findings
```

The `scan_file_ast_or_lexical` function currently returns the result of `apply_ast_rules()` directly (line 160). It must be changed to a `let mut findings = ...` pattern and then extended. The `code` variable (String) at line 135 must be coerced to `&[u8]` via `code.as_bytes()` — this is already the pattern at line 162.

---

#### Pattern C: `apply_paired_lock_rules()` — CWE-591

**Function signature** — same shape as `apply_signal_handler_rules()`:
```rust
fn apply_paired_lock_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding>
```

**Algorithm:** Walk all `function_definition` nodes at root level. For each function definition, collect all `call_expression` function-name texts recursively within the function body subtree (all descendants, not just direct children — see RESEARCH.md Pitfall 5). If `"VirtualAlloc"` is present in that set AND `"VirtualLock"` is NOT, emit one finding per `VirtualAlloc` call site.

**Call-name collection reuses `collect_arrays_in_subtree` recursion pattern** (lines 367–385), adapted to collect identifier texts from `call_expression` function fields into a `HashSet<String>`.

**Finding line:** The line of the `VirtualAlloc` `call_expression` node, not the function definition line.

---

#### Pattern D: CWE-762 — `delete` detection via text-level scan

**Context from RESEARCH.md:** tree-sitter-c v0.24.2 cannot parse C++ `delete` operator. Juliet CWE-762 files all use `namespace` + `class`, triggering `has_error() == true` → lexical fallback → 0 TPs. A synthetic namespace-free fixture is required to achieve ≥1 TP.

**Implementation approach (planner discretion per D-01):** Add a new helper `apply_delete_rules()` that is called from `scan_file_ast_or_lexical()` only when `has_error()` is false (i.e., the AST path was taken). Walk all `expression_statement` nodes and check if the raw source text of the node starts with `"delete"` (after trimming whitespace). If so, emit a CWE-762 finding.

Alternatively, since this path only fires for files that pass tree-sitter-c parse without error (namespace-free `.cpp` or simple `.c` files), a minimal approach is to add `"delete"` as an `AnyCall`-style rule but implemented at the expression-statement level rather than call-expression level, OR to add a raw-bytes scan of the file source before calling `apply_ast_rules`.

**The simplest viable implementation:** After `apply_ast_rules`, scan `code.as_bytes()` for the token `delete` using a simple byte-pattern check (analogous to `token_present_with_boundary` from `cwe_scanner.rs`). Emit one finding per occurrence with line number derived from byte offset. This fires on any file that reached the AST path (no parse error), so the synthetic fixture must also be namespace-free to reach this path.

**`token_present_with_boundary` import** (line 17 — already imported):
```rust
use crate::vulnerability::cwe_scanner::token_present_with_boundary;
```

---

#### Pattern E: Module-level doc comment update

**Source:** `src/vulnerability/ast_scanner.rs` lines 1–12 (module-level doc comment)

Update the CWE coverage list in the doc comment to reflect 49 CWEs (from 41) and note Win32-specific CWEs per D-07. Add a note that CWEs 114/272/284/591/785 are Win32-specific and will produce 0 TPs on non-Windows source.

---

### `tests/vulnerability_tests/ast_scanner_tests.rs` (test, request-response)

**Analog:** itself — the file already exists with Phase 18/20 tests. Extend with Phase 23 tests.

**Test helper pattern** (lines 11–17):
```rust
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

**Test naming convention** (from lines 21, 44, 96, 106, 129, 163, 197, 219, 233, 245):
`fn test_argval_cwe{N}_{description}()` or `fn test_ast_{description}()`.
For Phase 23 use: `fn phase_23_cwe{N}_{bad|good|description}()`.

**Inline C fixture pattern** (line 22):
```rust
let src = b"void f(void *ctx) { SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, 0); }\n";
let (_t, dirs) = setup_one_file("a.c", src);
let findings = run_ast_scanner(&dirs);
assert!(
    findings.iter().any(|f| f.cwe_id == 295 && f.source == SastSource::Ast),
    "Expected CWE-295 with SastSource::Ast; got {:?}", findings
);
```

**Negative-case pattern** (lines 140–149):
```rust
assert!(
    !findings.iter().any(|f| f.cwe_id == 295),
    "Expected NO CWE-295 finding for SSL_VERIFY_PEER; got {:?}", findings
);
```

**Per CWE-479 test structure:** Two test functions:
1. `phase_23_cwe479_bad_signal_handler_fires` — inline C with `signal(SIGINT, helperBad)` where `helperBad` calls `malloc`. Assert finding at `signal()` call site line.
2. `phase_23_cwe479_good_signal_handler_no_finding` — inline C with `signal(SIGINT, helperGood)` where `helperGood` only assigns to a `volatile` int. Assert no CWE-479 finding.

**Per CWE-591 test structure:** Two test functions:
1. `phase_23_cwe591_virtualalloc_without_virtuallock_fires`
2. `phase_23_cwe591_virtualalloc_with_virtuallock_no_finding`

**Per CWE-762 test structure:** One test using the synthetic fixture file (not inline C, since it uses C++ `delete`). Use `setup_one_file("cwe762_bad.c", include_bytes!("../fixtures/c/cwe762_delete_bad.c"))` or an inline namespace-free snippet like:
```rust
// Namespace-free C-compatible delete syntax for tree-sitter-c
let src = b"void f(void) { char *p = (char*)calloc(10, 1); delete p; }\n";
```

---

### `tests/fixtures/c/cwe762_delete_bad.c` (new fixture)

**Analog:** `tests/fixtures/c/dangerous_calls.c` (lines 1–18)

**Pattern:** Simple namespace-free C file with `calloc` + `delete` that tree-sitter-c can parse without `has_error()`. Must NOT use `namespace`, `class`, or other C++ keywords that trigger parse errors.

```c
/* Phase 23 CWE-762 synthetic fixture — calloc + delete mismatch.
   Intentionally namespace-free so tree-sitter-c parses without error. */
void cwe762_bad(void) {
    char *p = (char*)calloc(10, 1);
    delete p;
}
```

---

## Shared Patterns

### `SastFinding` construction
**Source:** `src/vulnerability/ast_scanner.rs` lines 280–288
**Apply to:** All new helper functions (`apply_signal_handler_rules`, `apply_paired_lock_rules`, and any CWE-762 helper)

```rust
findings.push(SastFinding {
    cwe_id: <N>,
    component_name: component_name.to_string(),
    component_ecosystem: component_ecosystem.to_string(),
    file_path: path.to_string_lossy().into_owned(),
    line: (node.start_position().row as u32) + 1,
    source: SastSource::Ast,
});
```

### Fresh-cursor recursion
**Source:** `src/vulnerability/ast_scanner.rs` lines 294–311 (visit_node recursion tail), lines 376–384 (collect_arrays_in_subtree)
**Apply to:** All new recursive tree-walking helpers in Phase 23

```rust
let mut cursor = node.walk(); // fresh cursor per level — avoids Pitfall 1
if cursor.goto_first_child() {
    loop {
        // recurse on cursor.node()
        if !cursor.goto_next_sibling() { break; }
    }
}
```

### Named-children argument collection
**Source:** `src/vulnerability/ast_scanner.rs` lines 208–213
**Apply to:** `apply_signal_handler_rules` pass 1 (extracting `signal()` arguments)

```rust
let args: Vec<Node> = if let Some(arg_list) = node.child_by_field_name("arguments") {
    let mut cursor = arg_list.walk();
    arg_list.named_children(&mut cursor).collect()
} else {
    vec![]
};
```

### `#[cfg(feature = "internal")]` gate
**Source:** `src/vulnerability/ast_scanner.rs` line 13; `tests/vulnerability_tests/ast_scanner_tests.rs` line 1
**Apply to:** All new test functions (file-level gate already in place in both files)

### Test helper `setup_one_file`
**Source:** `tests/vulnerability_tests/ast_scanner_tests.rs` lines 11–17
**Apply to:** All new Phase 23 unit tests — reuse the existing function, do not add a new one

---

## No Analog Found

No files in Phase 23 lack a codebase analog. All patterns are direct continuations of existing Phase 18/20/21 patterns in `ast_scanner.rs` and `ast_scanner_tests.rs`.

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `tests/fixtures/c/cwe762_delete_bad.c` | fixture | — | New file type for Phase 23, but `dangerous_calls.c` is a sufficient model |

---

## Critical Implementation Notes for Planner

1. **`scan_file_ast_or_lexical` must be refactored** from returning `apply_ast_rules(...)` directly (line 160) to collecting findings from all three helpers. Current lines 160–167 return immediately; they must be changed to `let mut findings = ...; findings.extend(...); findings`.

2. **Phase 21/22 CWEs not yet in code.** STATE.md records Phases 20–22 not complete. The current `AST_CWE_RULES` table ends at CWE-732 (Phase 18 baseline). Phase 23 planner must account for this: if Phase 23 executes before Phase 21/22, the Phase 21/22 CWEs will not be present. Phase 23 adds only its own 8 CWEs (114, 272, 284, 427, 479, 591, 762, 785).

3. **CWE-762 cannot use `AnyCall` on `"delete"` in `AST_CWE_RULES`.** `delete` is a C++ operator, never a `call_expression` in tree-sitter-c. Adding `"delete"` to a table rule will silently produce 0 TPs. Use a separate text-level or expression-statement scan.

4. **CWE-479 findings emit at `signal()` call site**, not at the non-reentrant call inside the handler. Pass 1 must store the line number of the `signal()` call expression for use in pass 2.

5. **CWE-284 `ArgAtIndex(4, ...)` — 0-based index.** Arg 4 is the 5th positional argument. The existing `ArgAtIndex` arm silently skips (returns false) when `idx >= args.len()`, so no panic risk.

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `tests/vulnerability_tests/`, `tests/fixtures/c/`
**Files scanned:** 3 source files read in full
**Pattern extraction date:** 2026-05-12
