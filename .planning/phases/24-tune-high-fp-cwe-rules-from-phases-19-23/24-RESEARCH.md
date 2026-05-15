# Phase 24: tune-high-fp-cwe-rules-from-phases-19-23 — Research

**Researched:** 2026-05-13
**Domain:** Rust AST scanner (tree-sitter-c), CWE rule tightening, Juliet corpus FP reduction
**Confidence:** HIGH — all findings verified directly against source code and existing documentation

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Scope**
- D-01: All 17 CWE targets from phases 19–23 that exceed 35% FP gate receive a fix attempt. No CWE is pre-accepted without an attempt.
- D-02: Phase 24 covers all high-FP CWEs from phases 19–23. All 17 are in scope.
- D-03: CWE-256 is **removed** from `AST_CWE_RULES` — 100% FP, 0 Juliet TPs, corpus mismatch. Net coverage after Phase 24: 48 CWEs.

**Fix Strategy — AnyCall / Function List Restrictions**
- D-04: CWE-338 — Remove `rand()`/`srand()` from function list; keep only `drand48`, `lrand48`, `random`, `mrand48`.
- D-05: CWE-676 — Drop `alloca` from function list; keep only `strtok`.
- D-06: CWE-426 — Replace `popen`/`system` with `dlopen`, `LoadLibraryExA`, `LoadLibraryExW`.
- D-07: CWE-780 — Remove `RSA_public_encrypt` entries; keep only `CryptEncrypt` `ArgAtIndex(3, &["0"])` entry.
- D-08: CWE-676 and CWE-338 unit tests — New synthetic unit tests confirming TP on tightened function lists.

**Fix Strategy — New ArgCheck Variant**
- D-09: CWE-126 — Change from `ArgCheck::AnyCall` to new `ArgCheck::FixedSizeBufferWithoutSizeArg(size_arg_index)` variant.
- D-10: CWE-680 — Add size-arg `binary_expression *` guard: fire only when size argument is a multiplication expression.
- D-11: CWE-467 — Tighten `SizeofPointer` check: fire only when sizeof operand is a pointer type (not struct or array).

**Fix Strategy — Structural Visitor Function Guards**
- D-12: CWE-480 — In `check_func_ptr_null_compare()`: walk enclosing function declarations, fire only when compared identifier's declaration type contains `(`.
- D-13: CWE-483 — In `check_block_delimitation()`: exclude when braceless if-body is `return_statement`, `break_statement`, or `continue_statement`.
- D-14: CWE-562 — In `check_return_stack_address()`: restrict to plain scalar variables only (not arrays or structs).
- D-15: CWE-570 — In `check_constant_condition()`: remove detection in loop conditions (`while`/`for`/`do-while`). Keep only `if`-condition context.
- D-16: CWE-571 — In `check_constant_condition()`: remove detection in loop conditions. Keep only `if`/ternary.
- D-17: CWE-535 — In `AstCweRule` for `fprintf`/`vfprintf`: combine with `NotStringLiteralAtIndex(1)` guard.
- D-18: CWE-587 — In `check_fixed_address_assignment()`: raise fixed-address threshold to `> 0xFFFF` (already at this threshold — see analysis below).
- D-19: CWE-478 — In `check_switch_structure()`: do NOT fire when switch has ≤2 cases.
- D-20: CWE-762 — In `apply_delete_rules()`: fire only when `calloc`, `malloc`, or `realloc` also appears in same file.

**Validation Strategy**
- D-21: Implement all 17 fixes first, then re-run `oracle.sh` once (single-pass validation).
- D-22: CWEs that drop to 0 Juliet TPs after tightening validated via synthetic unit tests.
- D-23: AUTOSAR regression check runs after human verification of Juliet oracle delta.
- D-24: Any fix still leaving FP% >35% becomes a human-review item in verification checklist.

**ANALYSIS.md Update**
- D-25: Full oracle re-run after all 17 fixes — regenerate all rows in Per-CWE TP/FP table.
- D-26: Add `## Phase 24 Notes` section documenting which CWEs were fixed, removed, residual.

**Phase 24 Success Criteria**
- D-27: Phase 24 complete when: (1) all 17 code changes made, (2) oracle re-run, (3) ANALYSIS.md updated, (4) human reviews Juliet delta, (5) AUTOSAR regression run.
- D-28: No hard numeric bar on how many must reach <35% FP. Residual failures become documented human-review items.

### Claude's Discretion

- CWE-126 `strcat` (no size arg): whether to use one rule entry with two modes or two separate `AstCweRule` entries.
- CWE-680 guard implementation: whether to use new `ArgCheck` variant or inline logic in `apply_ast_rules`.
- CWE-570/571 loop-context exclusion: exact tree-sitter node types to check.

### Deferred Ideas (OUT OF SCOPE)

- CWE-338 context-aware detection (keep `rand()` but fire only in security-sensitive contexts).
- CWE-256 replacement rule.
- CWE-570/571 variable-folding (constant propagation).
- CWE-480 mutual-recursion detection.
</user_constraints>

---

## Summary

Phase 24 is a purely subtractive tightening pass on `src/vulnerability/ast_scanner.rs`. The 17 target CWEs all have confirmed root causes documented in ANALYSIS.md, and all corresponding code locations have been verified against the actual source file. No new `AstCweRule` entries, no new downstream format changes, no new helper functions beyond one or two new `ArgCheck` variants.

The changes split across three categories: (1) table edits in `AST_CWE_RULES` (function list changes and ArgCheck variant swaps for CWEs 126, 256, 338, 426, 535, 676, 680, 780), (2) guard additions to six existing `check_*` visitor functions (CWEs 478, 480, 483, 562, 570/571, 587), and (3) a co-occurrence guard in `apply_delete_rules()` (CWE-762). The `SizeofPointer` tightening for CWE-467 and the new `FixedSizeBufferWithoutSizeArg` variant for CWE-126 require extending `apply_ast_rules()`'s match arm logic.

**Primary recommendation:** Implement all 17 fixes in a single wave (table edits first, then structural guards), run oracle.sh once after all changes, then update ANALYSIS.md. This matches the D-21/D-25 single-pass validation pattern established in Phases 21/22/23.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CWE rule table edits | `AST_CWE_RULES` static (src/vulnerability/ast_scanner.rs) | `apply_ast_rules()` match arms | Table drives which functions/args trigger; match arms handle new variant logic |
| Structural visitor guards | `check_*` functions (same file) | `find_enclosing_function()`, `collect_function_scope_pointer_declarators()` | Guards inline within the visitor; reuse existing scope-collection helpers |
| CWE-762 co-occurrence | `apply_delete_rules()` (same file) | File byte content (`src: &[u8]`) | Text-level scan; add pre-scan for C-alloc keywords before emitting findings |
| Oracle validation | `benchmark/juliet/oracle.sh` | `benchmark/juliet/ast.json` | Shell script takes ast.json + corpus root; run from repo root |
| Test harness | `tests/vulnerability_tests/ast_scanner_tests.rs` | `run_ast_scanner()` | All existing CWE tests use `setup_one_file` + `run_ast_scanner` pattern |

---

## Current State of AST_CWE_RULES Entries for 17 Target CWEs

[VERIFIED: direct read of src/vulnerability/ast_scanner.rs lines 67–243]

### CWE-256 (REMOVE — D-03)
```rust
// check_plaintext_password() — NOT in AST_CWE_RULES table; it's a structural check
// called from apply_ast_rules() at line 378.
```
Action: Remove the call to `check_plaintext_password()` at line 378 of `apply_ast_rules()`. The function itself can stay (dead code, compiler warns only with `--deny(dead_code)` which isn't in use here — safe to leave).

### CWE-338 (D-04 — remove rand/srand)
```rust
// Lines 147–151:
AstCweRule {
    cwe_id: 338,
    functions: &["rand", "random", "srand"],
    arg_check: ArgCheck::AnyCall,
},
```
Action: Change `functions` to `&["drand48", "lrand48", "random", "mrand48"]`.

### CWE-426 (D-06 — replace popen/system)
```rust
// Lines 153–160:
AstCweRule {
    cwe_id: 426,
    functions: &["popen", "_popen", "system"],
    arg_check: ArgCheck::AnyCall,
},
```
Action: Change `functions` to `&["dlopen", "LoadLibraryExA", "LoadLibraryExW"]`.

### CWE-535 (D-17 — add NotStringLiteralAtIndex guard)
```rust
// Lines 171–176:
AstCweRule {
    cwe_id: 535,
    functions: &["fprintf", "vfprintf"],
    arg_check: ArgCheck::ArgAtIndex(0, &["stderr"]),
},
```
Problem: This fires on ANY `fprintf(stderr, ...)` call, not just non-literal format strings. The fix per D-17 requires firing only when arg 1 (the format string) is NOT a string literal AND arg 0 is `stderr`. This is a compound condition that `ArgAtIndex` alone cannot express. The planner must decide between: (a) a new two-condition `ArgCheck` variant, or (b) moving CWE-535 to a dedicated `check_stderr_format_string()` visitor, or (c) a new `ArgCheck::StderrNonLiteralFormat` variant that hardcodes both checks inline in `apply_ast_rules()`.

### CWE-676 (D-05 — drop alloca)
```rust
// Lines 178–186:
AstCweRule {
    cwe_id: 676,
    functions: &["alloca", "strtok"],
    arg_check: ArgCheck::AnyCall,
},
```
Action: Change `functions` to `&["strtok"]` (remove `"alloca"`).

### CWE-680 (D-10 — add multiplication guard)
```rust
// Lines 188–194:
AstCweRule {
    cwe_id: 680,
    functions: &["malloc", "calloc", "realloc"],
    arg_check: ArgCheck::AnyCall,
},
```
Action: Change `arg_check` to a new variant (or inline logic) that fires only when the size arg is a `binary_expression` with `*` operator. See Architecture Patterns section for tree-sitter node details.

### CWE-780 (D-07 — remove RSA_public_encrypt entries)
```rust
// Lines 196–218 (three entries):
AstCweRule { cwe_id: 780, functions: &["RSA_public_encrypt"], arg_check: ArgCheck::ArgAtIndex(4, &["RSA_PKCS1_PADDING"]) },
AstCweRule { cwe_id: 780, functions: &["RSA_public_encrypt"], arg_check: ArgCheck::ArgAtIndex(4, &["RSA_NO_PADDING"]) },
AstCweRule { cwe_id: 780, functions: &["CryptEncrypt"], arg_check: ArgCheck::ArgAtIndex(3, &["0"]) },
```
Action: Delete the two `RSA_public_encrypt` entries. Keep only the `CryptEncrypt` entry.

### CWE-126 (D-09 — new FixedSizeBufferWithoutSizeArg variant)
```rust
// Lines 120–125:
AstCweRule {
    cwe_id: 126,
    functions: &["strcat", "strncat"],
    arg_check: ArgCheck::FixedSizeBuffer,
},
```
Current behavior: `FixedSizeBuffer` checks only that dest arg (index 0) is a fixed-size array. The Phase 24 fix adds: for `strncat(dest, src, n)`, also require that `n` (arg index 2) is NOT a `sizeof()` expression. For `strcat` (2-arg, no size arg), `FixedSizeBuffer` semantics apply unchanged (dest-is-array check only).
Action: Either split into two `AstCweRule` entries or add a new `ArgCheck::FixedSizeBufferWithoutSizeArg(u8)` variant (planner decides per Claude's Discretion). [ASSUMED: two separate entries is simpler — `strcat` keeps `ArgCheck::FixedSizeBuffer`; `strncat` gets `FixedSizeBufferWithoutSizeArg(2)`]

---

## Structural Visitor Functions: Exact State and Required Guards

[VERIFIED: direct read of src/vulnerability/ast_scanner.rs]

### CWE-478: `check_switch_structure()` — line 390

**Current:** Fires CWE-478 if `switch_statement` body has no `default_case` child (no `case_statement` without a `value` field). Fires on all such switches regardless of case count.

**Guard needed (D-19):** Count the number of `case_statement` children with a `value` field (non-default cases). If count <= 2, do NOT fire. The `body_children` vector is already collected — add `let case_count = body_children.iter().filter(|c| c.kind() == "case_statement" && c.child_by_field_name("value").is_some()).count();` before the `if !has_default` check.

### CWE-483: `check_block_delimitation()` — line 479

**Current:** Fires CWE-483 if `if_statement` consequence is not a `compound_statement`.

**Guard needed (D-13):** Before pushing the finding, check `consequence.kind()`. If it is `return_statement`, `break_statement`, or `continue_statement`, do NOT fire.
```rust
if consequence.kind() != "compound_statement"
    && consequence.kind() != "return_statement"
    && consequence.kind() != "break_statement"
    && consequence.kind() != "continue_statement"
{
    findings.push(...)
}
```

### CWE-480: `check_func_ptr_null_compare()` — line 629

**Current:** Fires when `if_statement` condition is `identifier == null/0`. No type check on the identifier.

**Guard needed (D-12):** After confirming `is_identifier_vs_null()`, extract the identifier name and walk the enclosing function's declarations to check whether any declaration for that identifier contains `(` in its raw source text (pointer-to-function heuristic). Use `find_enclosing_function(node)` (line 1869) and walk declaration nodes looking for a `function_declarator` child in the identifier's declarator.

**Implementation approach:** Collect all `declaration` nodes in enclosing function. For each, check if the raw source text of the declaration contains both the identifier name and `(`. This is a text heuristic — it will match `void (*fp)(int)` correctly. Use `node.utf8_text(src)` on the full declaration node.

### CWE-562: `check_return_stack_address()` — line 758

**Current:** `check_return_stmts_in_subtree()` fires when a return value identifier matches `local_names` (non-static, non-parameter locals). `collect_local_var_names()` collects ALL local variable names including arrays and structs.

**Guard needed (D-14):** Before adding a name to `local_names` in `collect_local_vars_in_subtree()`, check that the declaration is NOT an `array_declarator` (fixed-size array) and does NOT have a struct/union type specifier. Only scalar pointer types and plain scalars should be in the set.

**Implementation approach:** In `collect_local_vars_in_subtree()`, after finding a `declaration` node, check its children. If any child is an `array_declarator`, skip. If any child is a `struct_specifier` or `union_specifier`, skip. Only call `collect_decl_identifiers()` for scalar declarations.

### CWE-570 / CWE-571: `check_constant_condition()` — line 882

**Current:** Fires on `if_statement | while_statement | for_statement` with literal-valued conditions.

**Guard needed (D-15/D-16):** Change the match pattern to `"if_statement"` only (remove `"while_statement"` and `"for_statement"`). CWE-835 via `check_infinite_loop()` already covers the dangerous `while(1)` / `for(;;)` cases.

```rust
// Before:
if matches!(node.kind(), "if_statement" | "while_statement" | "for_statement") {
// After:
if node.kind() == "if_statement" {
```

This is the minimal change. Note: `do_statement` is not currently in the match and does not need to be added.

### CWE-587: `check_fixed_address_assignment()` — line 978

**Important finding (D-18):** [VERIFIED: read of lines 956–971] The threshold is already `> 0xFFFF`. Function `is_large_hex_literal()` at line 959 already implements `val > 0xFFFF`. The existing tests (`test_cwe587_fixed_hex_address` uses `0x400000`; `test_cwe587_null_cast_no_finding` uses `0`). **D-18 is already implemented.** The planner should verify against ANALYSIS.md FP data to confirm whether this is genuinely a no-op or whether Juliet's 73.9% FP comes from a different cause (non-hex literals, small hex values, or literal casts not wrapped in `cast_expression`).

### CWE-762: `apply_delete_rules()` — line 1640

**Current:** Text-level byte scan for `delete` keyword with word-boundary check. Fires on every matching token regardless of whether the file uses C-allocation functions. The `_root: Node` parameter is unused.

**Guard needed (D-20):** Before the main scan loop, pre-scan `src` for presence of `calloc`, `malloc`, or `realloc` as word-boundary tokens. If none found, return empty `findings` immediately.

**Pattern (mirrors `apply_paired_lock_rules()`):**
```rust
fn apply_delete_rules(
    _root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding> {
    // Co-occurrence guard: only fire if file also contains C-alloc calls (D-20)
    let has_c_alloc = ["calloc", "malloc", "realloc"].iter().any(|needle| {
        token_present_with_boundary(
            &String::from_utf8_lossy(src),
            needle,
        )
    });
    if !has_c_alloc {
        return Vec::new();
    }
    // ... rest of existing scan loop unchanged
}
```
`token_present_with_boundary` is already imported from `cwe_scanner` (line 28).

---

## ArgCheck Enum: Current Variants

[VERIFIED: direct read of src/vulnerability/ast_scanner.rs lines 36–54]

```rust
enum ArgCheck {
    FixedSizeBuffer,                          // dest arg is fixed-size array in scope
    NotStringLiteralAtIndex(u8),             // arg[i] is not string_literal (CWE-134)
    ArgAtIndex(u8, &'static [&'static str]), // arg[i] contains all tokens (ALL-OF)
    AnyCall,                                 // name match sufficient
    SizeofPointer,                           // any arg is sizeof(ptr-typed-ident)
}
```

**New variants needed for Phase 24:**
- `FixedSizeBufferWithoutSizeArg(u8)` — for CWE-126 `strncat`: dest is fixed-size array AND size arg at index `u8` is NOT a `sizeof_expression`.
- Optionally `SizeArgIsMultiplication(u8)` — for CWE-680: size arg at index `u8` is a `binary_expression` with `*` operator. (Planner may also implement inline in `apply_ast_rules()` instead.)

**`apply_ast_rules()` match arm location:** Line 1747 in `visit_node()` (called from `apply_ast_rules()`). New variant match arms go in the `match &rule.arg_check { ... }` block at line 1747.

---

## Architecture Patterns

### Pattern 1: Adding a Guard to an Existing check_* Function

All structural check functions follow the same pattern: check node kind, apply logic, push finding, then recurse with fresh cursor. To add a guard:

```rust
// Before finding push, add the guard condition:
if <existing_conditions> && <new_guard_condition> {
    findings.push(SastFinding { ... });
}
```

Never modify the recursion — only add conditions before `findings.push(...)`.

### Pattern 2: New ArgCheck Variant in visit_node()

Add variant to enum, add match arm in `visit_node()` at line 1747:

```rust
ArgCheck::FixedSizeBufferWithoutSizeArg(size_idx) => {
    // 1. Check dest arg (index 0) is identifier in fixed-array scope
    //    (reuse existing FixedSizeBuffer logic verbatim)
    // 2. Check size arg at *size_idx is NOT a sizeof_expression
    let size_index = *size_idx as usize;
    let dest_is_array = /* FixedSizeBuffer logic */;
    let size_not_sizeof = if size_index < args.len() {
        args[size_index].kind() != "sizeof_expression"
    } else {
        true // no size arg provided = AnyCall semantics for dest check
    };
    dest_is_array && size_not_sizeof
}
```

### Pattern 3: Unit Test for Tightened Rule (D-22)

```rust
#[test]
fn test_cwe_338_drand48_fires() {
    let tp = b"double f(void) { return drand48(); }\n";
    let (_t, dirs) = setup_one_file("cwe338_new.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(
        findings.iter().any(|f| f.cwe_id == 338 && f.source == SastSource::Ast),
        "CWE-338: expected drand48() to fire; got {:?}", findings
    );
}

#[test]
fn test_cwe_338_rand_no_longer_fires() {
    // After Phase 24 tightening, rand() must NOT fire CWE-338
    let tn = b"#include <stdlib.h>\nvoid f(void) { int x = rand(); (void)x; }\n";
    let (_t, dirs) = setup_one_file("cwe338_tn.c", tn);
    let findings = run_ast_scanner(&dirs);
    assert!(
        !findings.iter().any(|f| f.cwe_id == 338 && f.source == SastSource::Ast),
        "CWE-338: rand() should not fire after Phase 24 tightening; got {:?}", findings
    );
}
```

### Pattern 4: tree-sitter Node Types for Phase 24

[VERIFIED: from existing code patterns in ast_scanner.rs]

| C construct | tree-sitter node kind | Example |
|---|---|---|
| Fixed-size array decl | `array_declarator` (child of `declaration`) | `char buf[64]` |
| Pointer decl | `pointer_declarator` (child of `declaration`) | `char *p` |
| Function pointer decl | `pointer_declarator` containing `function_declarator` | `void (*fp)(int)` |
| Struct type | `struct_specifier` (sibling of declarator in `declaration`) | `struct Foo s` |
| Union type | `union_specifier` | `union Bar u` |
| Multiplication expr | `binary_expression` with operator `*` at `child(1)` | `n * sizeof(T)` |
| sizeof expr | `sizeof_expression` | `sizeof(x)` |
| Return statement | `return_statement` | `return x;` |
| Break statement | `break_statement` | `break;` |
| Continue statement | `continue_statement` | `continue;` |
| While loop | `while_statement` | `while(cond)` |
| For loop | `for_statement` | `for(...)` |
| Do-while loop | `do_statement` | `do {...} while(...)` |

### Pattern 5: CWE-467 SizeofPointer Tightening (D-11)

**Current:** `SizeofPointer` fires when any arg is `sizeof(ident)` where `ident` is a `pointer_declarator` in scope. This catches `sizeof(ptr)` correctly but also fires on `sizeof(struct_ptr)` which points to a struct — the struct is the real allocation target, not a sizeof-pointer bug.

**Fix:** After confirming `ident` is in `file_scope_pointers` or function-scope pointers, additionally check that the declaration of `ident` is a simple pointer (not a pointer to a struct). The `collect_pointer_declarators()` function currently collects ALL `pointer_declarator` identifiers regardless of what they point to. The fix is to exclude pointer declarations whose type specifier is `struct_specifier` or `union_specifier`.

**Implementation:** Modify `collect_pointer_declarators()` or add a new `collect_simple_pointer_declarators()` that skips declarations where any sibling of the `pointer_declarator` is a `struct_specifier` or `union_specifier`.

### Recommended Project Structure (no change)

Phase 24 touches only one file:
```
src/vulnerability/ast_scanner.rs   # all code changes
tests/vulnerability_tests/ast_scanner_tests.rs   # new unit tests
benchmark/juliet/ANALYSIS.md   # updated after oracle re-run
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Word-boundary token check in co-occurrence guard | Custom loop | `token_present_with_boundary()` (already imported line 28) | Already handles word boundaries correctly; used throughout the file |
| Scope collection for declarations | New traversal | `find_enclosing_function()` + `collect_function_scope_pointer_declarators()` | Already implemented; reuse directly |
| Tree-sitter node text extraction | Manual byte slice | `node.utf8_text(src)` | The established pattern throughout the file |
| Oracle re-run | Reimplementation | `benchmark/juliet/oracle.sh` | Standalone script, run as-is from repo root |

---

## Common Pitfalls

### Pitfall 1: D-18 May Be a No-Op

**What goes wrong:** The CONTEXT.md D-18 says "raise threshold to >0xFFFF" but `is_large_hex_literal()` at line 959 already implements `val > 0xFFFF`. If the planner implements this as a code change, no lines change and the commit is a no-op.

**Why it happens:** D-18 was written without verifying the current threshold.

**How to avoid:** Planner should verify current threshold in code. If already `> 0xFFFF`, the Phase 24 task for CWE-587 should be to investigate why 73.9% FP remains (e.g., non-hex literals, literals not wrapped in cast_expression) and address the actual root cause — or document it as a residual human-review item per D-24.

**Warning signs:** Task for CWE-587 has zero diff after implementation.

### Pitfall 2: CWE-570/571 Share One Function

**What goes wrong:** Both CWE-570 and CWE-571 detection are in `check_constant_condition()`. The D-15/D-16 loop exclusion changes `"if_statement" | "while_statement" | "for_statement"` to just `"if_statement"`. This is ONE code change but satisfies TWO decisions (D-15 and D-16). Planner should implement as a single task.

**How to avoid:** D-15 and D-16 are co-located; one line change covers both. Verify that `check_infinite_loop()` still fires on `while(1)` after the change (it does — separate function, unaffected).

### Pitfall 3: Fresh Cursor Per Recursion Level

**What goes wrong:** Using the same `TreeCursor` across recursive calls causes borrow errors at compile time in Rust. The established pattern is to create a new `node.walk()` at each recursion level.

**How to avoid:** When adding new traversal logic (e.g., walking declarations in enclosing function for CWE-480), always use `let mut cursor = node.walk(); node.named_children(&mut cursor)` or materialize to `Vec<Node>` immediately.

### Pitfall 4: CWE-256 Removal Scope

**What goes wrong:** CWE-256 is implemented via `check_plaintext_password()` called from `apply_ast_rules()` at line 378. There is NO `AstCweRule` table entry for CWE-256. Removing CWE-256 means removing the `check_plaintext_password()` call at line 378, NOT editing `AST_CWE_RULES`.

**How to avoid:** Planner task for CWE-256 must target line 378 (the call site in `apply_ast_rules()`), not the table.

### Pitfall 5: CWE-780 Has Three Table Entries, Remove Two

**What goes wrong:** There are exactly three `AstCweRule` entries for CWE-780 (lines 199–219). D-07 says remove `RSA_public_encrypt` entries — that's two of the three. Keep only the `CryptEncrypt` entry.

**How to avoid:** Count entries before deleting. The `CryptEncrypt` entry at lines 215–219 is the keeper.

### Pitfall 6: CWE-562 Must Distinguish Arrays from Scalars

**What goes wrong:** `collect_local_var_names()` currently collects ALL local variable names via `collect_decl_identifiers()` which recurses into all declarator children. An array `char buf[16]` would produce `"buf"` in the set. D-14 says skip arrays. If the planner naively skips all identifiers from `array_declarator` subtrees, `collect_decl_identifiers()` already recurses into them — need to add the array check at the `declaration` node level, not inside `collect_decl_identifiers`.

**How to avoid:** In `collect_local_vars_in_subtree()`, before calling `collect_decl_identifiers(node, src, out)`, check whether the `declaration` node contains any `array_declarator` or `struct_specifier`/`union_specifier` child. If yes, skip it.

---

## Code Examples

### CWE-762 Co-occurrence Guard

```rust
// Source: apply_paired_lock_rules() pattern, lines 1573–1603
fn apply_delete_rules(
    _root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding> {
    // D-20: co-occurrence guard — only fire if file has C-alloc calls
    let src_str = String::from_utf8_lossy(src);
    let has_c_alloc = ["calloc", "malloc", "realloc"].iter()
        .any(|needle| token_present_with_boundary(&src_str, needle));
    if !has_c_alloc {
        return Vec::new();
    }
    // ... rest of existing scan loop unchanged (lines 1648–1716)
}
```

### CWE-478 Case Count Guard

```rust
// In check_switch_structure(), after collecting body_children and before has_default check:
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

### CWE-483 Return/Break/Continue Exclusion

```rust
// In check_block_delimitation():
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
```

### CWE-570/571 Loop Exclusion

```rust
// In check_constant_condition() — change line 890:
// Before: if matches!(node.kind(), "if_statement" | "while_statement" | "for_statement") {
// After:
if node.kind() == "if_statement" {
```

### CWE-680 Multiplication Guard (inline variant)

```rust
// In visit_node() match arm for ArgCheck::AnyCall (or new SizeArgIsMultiplication variant):
// For CWE-680 rule entry, change arg_check to inspect first arg:
// The multiplication check: size arg at index 0 is binary_expression with operator "*"
let size_arg_idx = 0usize; // malloc/realloc/calloc: first arg is size
let fire = if size_arg_idx < args.len() {
    let arg = args[size_arg_idx];
    if arg.kind() == "binary_expression" {
        if let Some(op_node) = arg.child(1) {
            op_node.utf8_text(src).ok().as_deref() == Some("*")
        } else { false }
    } else { false }
} else { false };
```

Note: `calloc(nmemb, size)` — the "size" is split across two args. For `calloc`, the dangerous pattern is `calloc(n * something, size)` where arg 0 is multiplication, OR `calloc(n, size * sizeof(T))` where arg 1 is multiplication. The simplest approach is to check ANY arg for multiplication. Planner should decide scope (arg 0 only, or any arg).

---

## Oracle Usage

[VERIFIED: read of benchmark/juliet/oracle.sh]

```bash
# Run from repo root — oracle.sh auto-computes paths from BASH_SOURCE
./benchmark/juliet/oracle.sh

# Or explicitly:
./benchmark/juliet/oracle.sh benchmark/juliet/ast.json \
    example_target_repos/juliet-test-suite-c
```

The script requires: (1) `benchmark/juliet/ast.json` — generated by running the scanner on the Juliet corpus and writing output as JSON (the existing workflow); (2) the Juliet corpus at `example_target_repos/juliet-test-suite-c`. The script uses Python 3 (inline heredoc) — no additional dependencies.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | Cargo.toml (feature = "internal" gate) |
| Quick run command | `cargo test --features internal -p radeis-sc2sbom vulnerability_tests::ast_scanner_tests 2>&1` |
| Full suite command | `cargo test --features internal 2>&1` |

### Phase Requirements → Test Map

All existing tests in `tests/vulnerability_tests/ast_scanner_tests.rs` must continue passing (regression). New tests required per D-22/D-08 for tightened function lists.

| Decision | Behavior | Test Type | Automated Command |
|----------|----------|-----------|-------------------|
| D-04 (CWE-338 tighten) | `drand48()` fires; `rand()` does NOT fire | unit | `cargo test --features internal test_cwe_338` |
| D-05 (CWE-676 tighten) | `strtok()` fires; `alloca()` does NOT fire CWE-676 | unit | `cargo test --features internal test_cwe_676` |
| D-06 (CWE-426 replace) | `dlopen()` fires; `popen()` does NOT fire CWE-426 | unit | `cargo test --features internal test_cwe_426` |
| D-07 (CWE-780 tighten) | `CryptEncrypt(0)` fires; `RSA_public_encrypt` does NOT fire | unit | `cargo test --features internal test_cwe_780` |
| D-09 (CWE-126 strncat) | `strncat(buf, src, n)` with non-sizeof n fires | unit | `cargo test --features internal test_cwe_126` |
| D-10 (CWE-680 mult) | `malloc(n * sizeof(T))` fires; `malloc(sizeof(T))` does NOT | unit | `cargo test --features internal test_cwe_680` |
| D-13 (CWE-483 return) | `if(x) return;` does NOT fire CWE-483 | unit | `cargo test --features internal test_cwe483` |
| D-19 (CWE-478 count) | 2-case switch without default does NOT fire | unit | `cargo test --features internal test_cwe478` |
| D-20 (CWE-762 cooccur) | `delete` in file without malloc does NOT fire | unit | `cargo test --features internal phase_23_cwe762` |
| D-15/D-16 (CWE-570/571) | `while(1)` does NOT fire CWE-570/571; `if(0)` still fires | unit | `cargo test --features internal test_cwe57` |

### Wave 0 Gaps

New test functions needed (not yet present in ast_scanner_tests.rs):

- [ ] `test_cwe_338_drand48_fires` — TP on `drand48()`
- [ ] `test_cwe_338_rand_no_longer_fires` — TN on `rand()`
- [ ] `test_cwe_676_alloca_no_longer_fires` — TN on `alloca()` for CWE-676
- [ ] `test_cwe_426_dlopen_fires` — TP on `dlopen()`
- [ ] `test_cwe_426_popen_no_longer_fires` — TN on `popen()` for CWE-426
- [ ] `test_cwe_780_rsa_no_longer_fires` — TN on `RSA_public_encrypt`
- [ ] `test_cwe_126_strncat_nonsizeof_fires` — TP on `strncat(buf, s, n)` where `n` is not sizeof
- [ ] `test_cwe_126_strncat_sizeof_no_fire` — TN on `strncat(buf, s, sizeof(buf))` 
- [ ] `test_cwe_680_multiplication_fires` — TP on `malloc(n * sizeof(int))`
- [ ] `test_cwe_680_sizeof_only_no_fire` — TN on `malloc(sizeof(int))`
- [ ] `test_cwe_483_braceless_return_no_fire` — TN: `if(x) return;`
- [ ] `test_cwe478_two_case_no_fire` — TN: 2-case switch without default
- [ ] `test_cwe762_no_malloc_no_fire` — TN: `delete` in file without malloc
- [ ] `test_cwe570_while_zero_no_fire` — TN: `while(0)` does NOT fire CWE-570 after fix
- [ ] `test_cwe571_while_one_no_fire` — TN: `while(1)` does NOT fire CWE-571 after fix (CWE-835 still fires)

Existing tests to keep passing (regression):
- `test_cwe_126_fixed_size_buffer` (strcat still fires for fixed-size buffer)
- `test_cwe_338_weak_prng` (this test uses `rand()` — will FAIL after D-04; MUST UPDATE the test to use `drand48()` or change assertion)
- `test_cwe_676_dangerous_function_strtok` (strtok still fires — no change needed)
- `test_cwe_426_untrusted_search_path` (uses `popen` — will FAIL after D-06; MUST UPDATE)
- `test_cwe_780_rsa_no_oaep` (uses `RSA_public_encrypt` — will FAIL after D-07; MUST UPDATE or delete first two sub-assertions)
- All CWE-570/571 tests that use `if(0)`/`if(1)` — these remain TPs
- `test_cwe835_while_one_no_break` — CWE-835 must still fire on `while(1)`

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | D-18 (CWE-587 threshold) is already implemented at `> 0xFFFF` | "D-18 May Be a No-Op" pitfall | If threshold is different, need actual code change; low risk since code is verified |
| A2 | `token_present_with_boundary` handles string input (not bytes) | CWE-762 co-occurrence guard | If it requires bytes not string, convert `src` differently; check function signature |
| A3 | For CWE-480 func-ptr check, using `(` in raw declaration text is sufficient heuristic | CWE-480 guard | False negatives on unusual typedef'd function pointers; acceptable per D-24 |
| A4 | Two separate AstCweRule entries is simpler for CWE-126 strcat/strncat | Claude's Discretion | If planner chooses FixedSizeBufferWithoutSizeArg variant, the variant needs implementing; both are valid |

---

## Environment Availability

Step 2.6: External dependencies are limited to existing toolchain.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo (Rust toolchain) | Running tests | check at execution | — | — |
| Python 3 | oracle.sh | check at execution | — | — |
| Juliet corpus | oracle.sh | at example_target_repos/juliet-test-suite-c | — | Cannot re-run oracle without it |

---

## Security Domain

`security_enforcement` not set to false in config — section required.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | The CWE rules themselves ARE input validation checks; no external input to the scanner tightening logic |
| V6 Cryptography | no | — |

Phase 24 modifies the scanner's detection rules. The scanner itself reads source files and emits findings. No external input validation beyond what the tree-sitter parser handles. No cryptographic operations introduced.

---

## Sources

### Primary (HIGH confidence)
- `src/vulnerability/ast_scanner.rs` (lines 36–2203) — all function signatures, exact line numbers, ArgCheck variants, and current behavior verified by direct read
- `tests/vulnerability_tests/ast_scanner_tests.rs` — test harness pattern (`setup_one_file`, `run_ast_scanner`) verified by direct read
- `benchmark/juliet/ANALYSIS.md` — FP% figures for all 17 target CWEs, root cause notes, phase history verified by direct read
- `benchmark/juliet/oracle.sh` — oracle script usage and dependencies verified by direct read
- `.planning/phases/24-tune-high-fp-cwe-rules-from-phases-19-23/24-CONTEXT.md` — all decisions locked

### Secondary (MEDIUM confidence)
- `.planning/STATE.md` — project context and milestone history
- `.planning/config.json` — nyquist_validation: true confirmed

---

## Metadata

**Confidence breakdown:**
- Current code state (line numbers, function signatures): HIGH — verified by direct read
- Fix implementations: HIGH — all derived from existing patterns in the same file
- D-18 no-op analysis: HIGH — `is_large_hex_literal()` threshold verified directly
- Test list for Wave 0: HIGH — derived from decisions and existing test patterns
- Existing test breakage list: HIGH — derived by tracing each decision against test assertions

**Research date:** 2026-05-13
**Valid until:** Until `src/vulnerability/ast_scanner.rs` is modified (all line references are point-in-time)
