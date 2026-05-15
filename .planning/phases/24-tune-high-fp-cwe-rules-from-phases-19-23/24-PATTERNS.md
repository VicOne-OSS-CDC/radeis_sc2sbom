# Phase 24: tune-high-fp-cwe-rules-from-phases-19-23 — Pattern Map

**Mapped:** 2026-05-13
**Files analyzed:** 2 (src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs)
**Analogs found:** 2 / 2 — single file; all patterns are intra-file self-analogs

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `src/vulnerability/ast_scanner.rs` — `AST_CWE_RULES` table | config/rule-table | transform | Existing `AstCweRule` entries lines 67–243 | exact |
| `src/vulnerability/ast_scanner.rs` — `ArgCheck` enum | model | transform | Existing `ArgCheck` variants lines 36–53 | exact |
| `src/vulnerability/ast_scanner.rs` — `apply_ast_rules()` | service | transform | Existing `apply_ast_rules()` lines 342–382 | exact |
| `src/vulnerability/ast_scanner.rs` — `visit_node()` match arms | service | transform | Existing `ArgCheck::FixedSizeBuffer` arm lines 1783–1803 | exact |
| `src/vulnerability/ast_scanner.rs` — `check_block_delimitation()` | utility | request-response | Existing function lines 479–511 | exact |
| `src/vulnerability/ast_scanner.rs` — `check_func_ptr_null_compare()` | utility | request-response | Existing function lines 629–676 | exact |
| `src/vulnerability/ast_scanner.rs` — `check_return_stack_address()` | utility | request-response | Existing function lines 758–858 | exact |
| `src/vulnerability/ast_scanner.rs` — `check_constant_condition()` | utility | request-response | Existing function lines 882–954 | exact |
| `src/vulnerability/ast_scanner.rs` — `check_switch_structure()` | utility | request-response | Existing function lines 390–455 | exact |
| `src/vulnerability/ast_scanner.rs` — `apply_delete_rules()` | utility | transform | Existing function lines 1640–1717 | exact |
| `tests/vulnerability_tests/ast_scanner_tests.rs` — new unit tests | test | request-response | Existing tests `test_cwe_338_weak_prng` line 353, `phase_23_cwe762_*` lines 959–968 | exact |

---

## Pattern Assignments

### 1. `AST_CWE_RULES` table — function list edits (D-03, D-04, D-05, D-06, D-07)

**Analog:** Lines 147–243 of `src/vulnerability/ast_scanner.rs`

**Structure of one table entry** (lines 147–151):
```rust
AstCweRule {
    cwe_id: 338,
    functions: &["rand", "random", "srand"],
    arg_check: ArgCheck::AnyCall,
},
```

**Multiple entries for one CWE** — split when ArgAtIndex uses ALL-OF semantics (lines 199–219):
```rust
AstCweRule {
    cwe_id: 780,
    functions: &["RSA_public_encrypt"],
    arg_check: ArgCheck::ArgAtIndex(4, &["RSA_PKCS1_PADDING"]),
},
AstCweRule {
    cwe_id: 780,
    functions: &["RSA_public_encrypt"],
    arg_check: ArgCheck::ArgAtIndex(4, &["RSA_NO_PADDING"]),
},
AstCweRule {
    cwe_id: 780,
    functions: &["CryptEncrypt"],
    arg_check: ArgCheck::ArgAtIndex(3, &["0"]),
},
```

**Changes required:**
- CWE-256 (D-03): Remove the `check_plaintext_password()` call at `apply_ast_rules()` line 378. There is no `AstCweRule` entry for CWE-256 — the removal point is the call site, not the table.
- CWE-338 (D-04): Change `functions` from `&["rand", "random", "srand"]` to `&["drand48", "lrand48", "random", "mrand48"]`.
- CWE-676 (D-05): Change `functions` from `&["alloca", "strtok"]` to `&["strtok"]`.
- CWE-426 (D-06): Change `functions` from `&["popen", "_popen", "system"]` to `&["dlopen", "LoadLibraryExA", "LoadLibraryExW"]`.
- CWE-780 (D-07): Delete both `RSA_public_encrypt` entries (lines 199–208). Keep only the `CryptEncrypt` entry (lines 215–219).

---

### 2. `ArgCheck` enum — new variants (D-09, D-10)

**Analog:** Lines 36–53 of `src/vulnerability/ast_scanner.rs`

**Existing enum** (lines 36–53):
```rust
#[derive(Debug)]
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ArgAtIndex(u8, &'static [&'static str]),
    AnyCall,
    SizeofPointer,
}
```

**New variants to add** (after `SizeofPointer`):
```rust
    /// CWE-126 strncat: dest arg (index 0) is fixed-size array AND size arg
    /// at `size_arg_index` is NOT a sizeof_expression.
    FixedSizeBufferWithoutSizeArg(u8),
    /// CWE-680: size arg at index `size_arg_index` is a binary_expression
    /// with operator `*` (multiplication). Fires only on multiply-in-size pattern.
    SizeArgIsMultiplication(u8),
```

**Table entry pattern for CWE-126 split** (mirrors the CWE-780 three-entry split at lines 199–219):
```rust
// strcat: dest-is-array check only (no size arg)
AstCweRule {
    cwe_id: 126,
    functions: &["strcat"],
    arg_check: ArgCheck::FixedSizeBuffer,
},
// strncat: dest-is-array AND size arg (index 2) is NOT sizeof
AstCweRule {
    cwe_id: 126,
    functions: &["strncat"],
    arg_check: ArgCheck::FixedSizeBufferWithoutSizeArg(2),
},
```

**Table entry for CWE-680** (replaces `AnyCall` at line 188–194):
```rust
AstCweRule {
    cwe_id: 680,
    functions: &["malloc", "realloc"],
    arg_check: ArgCheck::SizeArgIsMultiplication(0),
},
AstCweRule {
    cwe_id: 680,
    functions: &["calloc"],
    arg_check: ArgCheck::SizeArgIsMultiplication(0), // check first arg
},
```

---

### 3. `visit_node()` — new match arms for ArgCheck variants

**Analog:** `ArgCheck::FixedSizeBuffer` arm lines 1783–1803, `ArgCheck::SizeofPointer` arm lines 1805–1828

**`FixedSizeBuffer` arm** (exact model to copy for `FixedSizeBufferWithoutSizeArg`):
```rust
ArgCheck::FixedSizeBuffer => {
    if !args.is_empty() && args[0].kind() == "identifier" {
        if let Ok(dest_name) = args[0].utf8_text(src) {
            let fn_scope = find_enclosing_function(node);
            let in_scope = if let Some(fn_node) = fn_scope {
                let local = collect_function_scope_fixed_arrays(fn_node, src);
                local.contains(dest_name)
            } else {
                false
            };
            in_scope || file_scope_arrays.contains(dest_name)
        } else {
            false
        }
    } else {
        false
    }
}
```

**New `FixedSizeBufferWithoutSizeArg(size_idx)` arm** — copy FixedSizeBuffer logic then add size check:
```rust
ArgCheck::FixedSizeBufferWithoutSizeArg(size_idx) => {
    let dest_is_array = if !args.is_empty() && args[0].kind() == "identifier" {
        if let Ok(dest_name) = args[0].utf8_text(src) {
            let fn_scope = find_enclosing_function(node);
            let in_scope = if let Some(fn_node) = fn_scope {
                let local = collect_function_scope_fixed_arrays(fn_node, src);
                local.contains(dest_name)
            } else {
                false
            };
            in_scope || file_scope_arrays.contains(dest_name)
        } else {
            false
        }
    } else {
        false
    };
    let size_index = *size_idx as usize;
    let size_not_sizeof = if size_index < args.len() {
        args[size_index].kind() != "sizeof_expression"
    } else {
        true // no size arg = fire if dest is array (strcat semantics)
    };
    dest_is_array && size_not_sizeof
}
```

**New `SizeArgIsMultiplication(size_idx)` arm** — modeled on `NotStringLiteralAtIndex` structure (lines 1750–1758) + binary_expression operator check:
```rust
ArgCheck::SizeArgIsMultiplication(size_idx) => {
    let idx = *size_idx as usize;
    if idx < args.len() {
        let arg = args[idx];
        if arg.kind() == "binary_expression" {
            if let Some(op_node) = arg.child(1) {
                op_node.utf8_text(src).ok().as_deref() == Some("*")
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}
```

The `SizeofPointer` arm (lines 1805–1828) shows the correct pattern for `arg.child(1)` operator access on a `binary_expression`:
```rust
if let Some(op_node) = inner.child(1) {
    if op_node.utf8_text(src).ok().as_deref() == Some("==") {
```

---

### 4. `check_switch_structure()` — CWE-478 case-count guard (D-19)

**Analog:** Lines 390–455 of `src/vulnerability/ast_scanner.rs`

**Current firing logic** (lines 413–425):
```rust
let has_default = body_children.iter().any(|c| {
    c.kind() == "case_statement" && c.child_by_field_name("value").is_none()
});
if !has_default {
    findings.push(SastFinding {
        cwe_id: 478,
        ...
    });
}
```

**Change:** Insert case count before the `if !has_default` check:
```rust
let non_default_case_count = body_children.iter()
    .filter(|c| c.kind() == "case_statement" && c.child_by_field_name("value").is_some())
    .count();
let has_default = body_children.iter().any(|c| {
    c.kind() == "case_statement" && c.child_by_field_name("value").is_none()
});
if !has_default && non_default_case_count > 2 {
    findings.push(SastFinding { cwe_id: 478, ... });
}
```

---

### 5. `check_block_delimitation()` — CWE-483 return/break/continue exclusion (D-13)

**Analog:** Lines 479–511 of `src/vulnerability/ast_scanner.rs`

**Current firing logic** (lines 487–499):
```rust
if node.kind() == "if_statement" {
    if let Some(consequence) = node.child_by_field_name("consequence") {
        if consequence.kind() != "compound_statement" {
            findings.push(SastFinding {
                cwe_id: 483,
                ...
            });
        }
    }
}
```

**Change:** Extend the kind check:
```rust
if node.kind() == "if_statement" {
    if let Some(consequence) = node.child_by_field_name("consequence") {
        let kind = consequence.kind();
        if kind != "compound_statement"
            && kind != "return_statement"
            && kind != "break_statement"
            && kind != "continue_statement"
        {
            findings.push(SastFinding { cwe_id: 483, ... });
        }
    }
}
```

Note: The recursion block (lines 502–510) is unchanged — never modify the recursion pattern.

---

### 6. `check_func_ptr_null_compare()` — CWE-480 function-pointer type guard (D-12)

**Analog:** Lines 629–676 of `src/vulnerability/ast_scanner.rs`

**Current firing logic** (lines 648–660):
```rust
if is_identifier_vs_null(lhs, rhs, src)
    || is_identifier_vs_null(rhs, lhs, src)
{
    findings.push(SastFinding { cwe_id: 480, ... });
}
```

**Change:** Before pushing, extract the identifier name and check enclosing function declarations for `(` in the raw declaration text (function-pointer heuristic). Pattern for collecting declarations uses `find_enclosing_function` (line 1869) and `node.utf8_text(src)` (the established text-extraction pattern throughout the file):

```rust
// After confirming is_identifier_vs_null, extract the identifier name:
let ident_node = if lhs.kind() == "identifier" { lhs } else { rhs };
if let Ok(ident_name) = ident_node.utf8_text(src) {
    let is_func_ptr = find_enclosing_function(node)
        .map(|fn_node| {
            // Walk declarations in function scope, check for identifier + '(' in raw text
            let mut found = false;
            let mut cursor = fn_node.walk();
            for child in fn_node.named_children(&mut cursor) {
                if child.kind() == "declaration" {
                    if let Ok(decl_text) = child.utf8_text(src) {
                        if decl_text.contains(ident_name) && decl_text.contains('(') {
                            found = true;
                            break;
                        }
                    }
                }
            }
            found
        })
        .unwrap_or(false);
    if is_func_ptr {
        findings.push(SastFinding { cwe_id: 480, ... });
    }
}
```

The cursor materialization pattern (fresh cursor per recursion level) is shown at lines 501–510, 611–621, 666–675.

---

### 7. `check_return_stack_address()` / `collect_local_vars_in_subtree()` — CWE-562 scalar-only guard (D-14)

**Analog:** Lines 710–751 of `src/vulnerability/ast_scanner.rs`

**Current `collect_local_vars_in_subtree`** (lines 710–733):
```rust
fn collect_local_vars_in_subtree(node: Node, src: &[u8], out: &mut HashSet<String>) {
    if node.kind() == "declaration" {
        let has_static_or_extern = {
            let mut cur = node.walk();
            let children: Vec<Node> = node.children(&mut cur).collect();
            children.iter().any(|c| {
                c.kind() == "storage_class_specifier"
                    && matches!(c.utf8_text(src).unwrap_or(""), "static" | "extern")
            })
        };
        if !has_static_or_extern {
            collect_decl_identifiers(node, src, out);
        }
    }
    // recursion...
}
```

**Change:** Add array/struct/union skip BEFORE calling `collect_decl_identifiers`, inside the `if !has_static_or_extern` block. The pattern for inspecting `declaration` children is already established above — just add:

```rust
if !has_static_or_extern {
    // D-14: skip arrays and struct/union types (CWE-562 — scalar-only)
    let mut cur2 = node.walk();
    let children2: Vec<Node> = node.children(&mut cur2).collect();
    let is_array_or_struct = children2.iter().any(|c| {
        c.kind() == "array_declarator"
            || c.kind() == "struct_specifier"
            || c.kind() == "union_specifier"
    });
    // Also need to check init_declarator children for array_declarator:
    let has_array_child = children2.iter().any(|c| {
        c.kind() == "init_declarator" && {
            let mut ic = c.walk();
            c.named_children(&mut ic).any(|gc| gc.kind() == "array_declarator")
        }
    });
    if !is_array_or_struct && !has_array_child {
        collect_decl_identifiers(node, src, out);
    }
}
```

---

### 8. `check_constant_condition()` — CWE-570/571 loop exclusion (D-15/D-16)

**Analog:** Lines 882–954 of `src/vulnerability/ast_scanner.rs`

**Current match** (line 890):
```rust
if matches!(node.kind(), "if_statement" | "while_statement" | "for_statement") {
```

**Change** (single line, covers both D-15 and D-16):
```rust
if node.kind() == "if_statement" {
```

The recursion block (lines 944–953) is unchanged. This one edit satisfies both CWE-570 and CWE-571 decisions simultaneously.

---

### 9. `apply_delete_rules()` — CWE-762 co-occurrence guard (D-20)

**Analog:** `apply_paired_lock_rules()` lines 1573–1603 (file-level function-scan pattern); `token_present_with_boundary` import at line 27.

**Current function signature** (lines 1640–1646):
```rust
fn apply_delete_rules(
    _root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding> {
    let mut findings = Vec::new();
    let needle = b"delete";
```

**Change:** Insert co-occurrence pre-scan immediately after `let mut findings = Vec::new();`:
```rust
// D-20: CWE-762 co-occurrence guard — only fire if file contains C-alloc calls
let src_str = String::from_utf8_lossy(src);
let has_c_alloc = ["calloc", "malloc", "realloc"]
    .iter()
    .any(|needle| token_present_with_boundary(&src_str, needle));
if !has_c_alloc {
    return Vec::new();
}
```

`token_present_with_boundary` signature takes `&str` (confirmed by import at line 27 and usage throughout the file). `String::from_utf8_lossy(src)` produces a `Cow<str>` which coerces to `&str` via `&src_str`.

---

### 10. CWE-467 `SizeofPointer` tightening — `collect_pointer_declarators()` (D-11)

**Analog:** `collect_pointer_declarator_rec()` lines 2056–2069 and `collect_ptrs_in_subtree()` lines 2033–2044.

**Current `collect_pointer_declarators`** (lines 2048–2054):
```rust
fn collect_pointer_declarators(node: Node, src: &[u8], out: &mut HashSet<String>) {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_pointer_declarator_rec(child, src, out);
        }
    }
}
```

**Change:** Before calling `collect_pointer_declarator_rec`, check whether any sibling of the pointer declarator in the `declaration` node is a `struct_specifier` or `union_specifier`:
```rust
fn collect_pointer_declarators(node: Node, src: &[u8], out: &mut HashSet<String>) {
    // D-11: skip pointer declarations whose type is struct or union
    // (sizeof(struct_ptr) is intentional; not a CWE-467 bug)
    let is_struct_or_union_ptr = (0..node.child_count())
        .filter_map(|i| node.child(i))
        .any(|c| c.kind() == "struct_specifier" || c.kind() == "union_specifier");
    if is_struct_or_union_ptr {
        return;
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_pointer_declarator_rec(child, src, out);
        }
    }
}
```

---

### 11. CWE-535 `ArgCheck` change (D-17)

**Analog:** `ArgCheck::NotStringLiteralAtIndex` arm lines 1750–1758; `ArgCheck::ArgAtIndex` arm lines 1760–1781.

**Current CWE-535 table entry** (lines 171–176):
```rust
AstCweRule {
    cwe_id: 535,
    functions: &["fprintf", "vfprintf"],
    arg_check: ArgCheck::ArgAtIndex(0, &["stderr"]),
},
```

**Fix options** — two valid approaches per D-17 (planner decides):

Option A — New two-condition `AstCweRule` entry with compound `ArgCheck` variant:
Change the entry to combine `ArgAtIndex(0, &["stderr"])` AND `NotStringLiteralAtIndex(1)` — requires a new `ArgCheck::StderrNonLiteralFormat` variant that hardcodes both checks inline in the match arm.

Option B — Move CWE-535 out of `AST_CWE_RULES` into a dedicated `check_stderr_format_string()` visitor (following the `check_block_delimitation` pattern at lines 479–511), called from `apply_ast_rules()` at line 382. This avoids a new variant.

Option B is simpler. The structural visitor pattern (lines 479–511) to copy:
```rust
fn check_stderr_format_string(
    node: Node, src: &[u8], path: &Path,
    component_name: &str, component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "call_expression" {
        if let Some(func_node) = node.child_by_field_name("function") {
            if matches!(func_node.utf8_text(src).unwrap_or(""), "fprintf" | "vfprintf") {
                if let Some(arg_list) = node.child_by_field_name("arguments") {
                    let mut cursor = arg_list.walk();
                    let args: Vec<Node> = arg_list.named_children(&mut cursor).collect();
                    // arg 0 must be "stderr", arg 1 must NOT be string_literal
                    let arg0_stderr = args.first()
                        .and_then(|a| a.utf8_text(src).ok())
                        .map(|t| t == "stderr")
                        .unwrap_or(false);
                    let arg1_non_literal = args.get(1)
                        .map(|a| a.kind() != "string_literal")
                        .unwrap_or(false);
                    if arg0_stderr && arg1_non_literal {
                        findings.push(SastFinding { cwe_id: 535, ... });
                    }
                }
            }
        }
    }
    // Recurse — fresh cursor per level (Pitfall 1)
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            check_stderr_format_string(cursor.node(), src, path,
                component_name, component_ecosystem, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

If Option B is chosen, remove the `ArgAtIndex(0, &["stderr"])` entry from `AST_CWE_RULES` and add the `check_stderr_format_string()` call in `apply_ast_rules()` after line 382.

---

## Shared Patterns

### Pattern A: `SastFinding` push (applies to all new/modified check_* functions)
**Source:** Lines 490–497, 653–659, 1702–1709
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
**Apply to:** Every new/modified `findings.push(...)` call in all changed functions.

### Pattern B: Fresh cursor per recursion level (Pitfall 1)
**Source:** Lines 502–510 (check_block_delimitation), 666–675 (check_func_ptr_null_compare)
```rust
let mut cursor = node.walk();
if cursor.goto_first_child() {
    loop {
        check_<function_name>(
            cursor.node(), src, path, component_name, component_ecosystem, findings,
        );
        if !cursor.goto_next_sibling() { break; }
    }
}
```
**Apply to:** All `check_*` functions. Never reuse a cursor across recursion levels.

### Pattern C: Materialized children (avoids borrow lifetime issues)
**Source:** Lines 714–720 (collect_local_vars_in_subtree), lines 776–779 (check_return_stack_address)
```rust
let mut cur = node.walk();
let children: Vec<Node> = node.children(&mut cur).collect();
// Now use children without lifetime conflicts
children.iter().any(|c| { ... })
```
**Apply to:** All new traversal logic that needs to inspect multiple children (CWE-480 declaration walk, CWE-562 array check).

### Pattern D: `find_enclosing_function` + scope collection
**Source:** `find_enclosing_function()` lines 1869–1878; `collect_function_scope_fixed_arrays()` lines 1914–1918; `collect_function_scope_pointer_declarators()` lines 2003–2007
```rust
let fn_scope = find_enclosing_function(node);
let in_scope = if let Some(fn_node) = fn_scope {
    let local = collect_function_scope_fixed_arrays(fn_node, src);
    local.contains(dest_name)
} else {
    false
};
```
**Apply to:** CWE-480 (walk declarations for func-ptr check), CWE-467 (already uses this pattern).

### Pattern E: `token_present_with_boundary` for text-level scan
**Source:** Import at line 27; used throughout `apply_delete_rules()` and `apply_paired_lock_rules()`
```rust
use crate::vulnerability::cwe_scanner::token_present_with_boundary;
// Usage:
let src_str = String::from_utf8_lossy(src);
token_present_with_boundary(&src_str, "malloc")
```
**Apply to:** CWE-762 co-occurrence guard in `apply_delete_rules()`.

---

## Unit Test Patterns

### Test harness (apply to all new tests)
**Source:** Lines 10–16 of `tests/vulnerability_tests/ast_scanner_tests.rs`
```rust
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

### TP test pattern (Phase 23 naming convention)
**Source:** Lines 840–848 (`phase_23_cwe114_load_library_fires`)
```rust
#[test]
fn phase_24_cwe_<N>_<fn_name>_fires() {
    let src = b"<inline C that exercises the tightened rule>\n";
    let (_t, dirs) = setup_one_file("<filename>.c", src);
    let findings = run_ast_scanner(&dirs);
    assert!(
        findings.iter().any(|f| f.cwe_id == <N> && f.source == SastSource::Ast),
        "CWE-<N>: expected <description>; got {:?}", findings
    );
}
```

### TN (negative) test pattern
**Source:** Lines 509–518 (`test_cwe478_switch_with_default_no_finding`)
```rust
#[test]
fn phase_24_cwe_<N>_<description>_no_fire() {
    let tn = b"<inline C that must NOT fire>\n";
    let (_t, dirs) = setup_one_file("<filename>.c", tn);
    let findings = run_ast_scanner(&dirs);
    assert!(
        !findings.iter().any(|f| f.cwe_id == <N> && f.source == SastSource::Ast),
        "CWE-<N>: <description> should not fire; got {:?}", findings
    );
}
```

### Existing tests that WILL BREAK after Phase 24 changes
These must be updated, not left failing:

| Test | Line | Breaks on | Required update |
|---|---|---|---|
| `test_cwe_338_weak_prng` | 353 | D-04: `rand()` removed | Change fixture to use `drand48()`; update assertion |
| `test_cwe_426_untrusted_search_path` | 364 | D-06: `popen` removed | Change fixture to use `dlopen()`; update assertion |
| `test_cwe_780_rsa_no_oaep` | 432 | D-07: `RSA_public_encrypt` entries removed | Remove/skip first sub-assertion (RSA_PKCS1_PADDING); keep CryptEncrypt sub-assertions |
| `test_cwe_535_shell_error_stderr` | 387 | D-17: if Option B taken, `ArgAtIndex` entry removed | Update to use non-literal format string; add TN for literal format string |
| `test_cwe570_if_zero` | 631 | D-15: loop exclusion; `if(0)` still fires — **no change needed** | No update required (if-context TPs remain) |
| `test_cwe571_if_one` | 656 | D-16: loop exclusion; `if(1)` still fires — **no change needed** | No update required |

---

## No Analog Found

All Phase 24 changes are within existing code; there are no files without analogs.

| File | Note |
|---|---|
| `benchmark/juliet/ANALYSIS.md` | Updated from oracle output after code changes; no code pattern needed |

---

## Metadata

**Analog search scope:** `src/vulnerability/ast_scanner.rs` (2,203 lines), `tests/vulnerability_tests/ast_scanner_tests.rs`
**Sections read:** ast_scanner.rs lines 1–243, 330–455, 479–511, 610–676, 700–858, 876–1024, 1560–1717, 1740–1878, 1920–2080; ast_scanner_tests.rs lines 1–969
**Pattern extraction date:** 2026-05-13
