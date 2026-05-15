# Phase 20: argument-value-ast-migration — Research

**Researched:** 2026-05-12
**Domain:** Rust / tree-sitter AST node inspection, enum variant design, lexical scanner cleanup
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**ArgCheck Variant Design**
- D-01: Add `ArgCheck::ArgAtIndex(u8, &'static [&'static str])` to the `ArgCheck` enum in `ast_scanner.rs`. The `u8` is the 0-based positional argument index; the `&'static [&'static str]` is the token slice that must ALL be present within that arg's AST subtree.
- D-02: The nested walk within `ArgAtIndex`: recursively collect all leaf node texts within the arg's AST subtree, then apply `token_present_with_boundary` on the concatenated text for each required token.
- D-03: When the arg count is fewer than the required index (`idx >= args.len()`): skip silently — return `false`, no finding, no panic.
- D-04: Delete `ArgCheck::ContainsTokens` from the enum entirely. All three uses (CWE-295/319/732) are replaced by `ArgAtIndex`.

**CWE-295 Migration**
- D-05: Migrate `SSL_CTX_set_verify` and `SSL_set_verify` rules to `ArgAtIndex(1, &["SSL_VERIFY_NONE"])`.
- D-06: Add `wolfSSL_CTX_set_verify` to the AST rules with `ArgAtIndex(1, &["SSL_VERIFY_NONE"])`.
- D-07: Keep `SSL_CTX_set_cert_verify_callback` as `ArgCheck::AnyCall`.

**CWE-319 Migration**
- D-08/D-09: Three separate `ArgAtIndex` rules for `curl_easy_setopt`. If AND-ing two `ArgAtIndex` checks requires a new enum variant, introduce `ArgAtTwoIndices(u8, &'static [&'static str], u8, &'static [&'static str])` only if needed. Otherwise, option-name-only detection (checking arg 1 for the unique constant) is sufficient to identify the dangerous call pattern.

**CWE-732 Migration**
- D-10: `umask` rule: the arg at index 0 must be an `integer_literal` AST node AND its text must be exactly `"0"`.
- D-11: `SetSecurityDescriptorDacl` rule: `ArgAtIndex(2, &["NULL"])`.

**Lexical Scanner Cleanup**
- D-12: Remove the 5 `arg_value_contains` rules for CWE-295, CWE-319, and CWE-732 from `cwe_scanner.rs` (lines ~89–96).
- D-13: Remove `arg_value_contains: Option<&'static [&'static str]>` field from `CweRule` struct and the `paren_args_contain_all` usage in `scan_file`.

**Test / Validation Strategy**
- D-14: Create synthetic `.c` fixture files under `tests/fixtures/` for each migrated CWE rule with TP, FP guard, and nested-expression cases.
- D-15: Wire fixtures into `#[test]` functions.
- D-16: No Juliet manifest.xml needed for Phase 20.

### Claude's Discretion

None specified — all decisions are locked.

### Deferred Ideas (OUT OF SCOPE)
- Juliet manifest.xml integration for CWE-295/319/732
- `ArgAtTwoIndices` variant (introduce only if needed to prevent FPs in Phase 20; otherwise defer to Phase 21)
- CWE-732 zero-value family (`0x0`, `0b0`) extended matching
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARGVAL-01 | CWE-295 (SSL_VERIFY_NONE), CWE-319 (CURLOPT_USE_SSL), and CWE-732 (umask/DACL) argument-value rules migrated from paren-bound string scanning to AST argument node inspection | Add `ArgAtIndex` variant; replace `ContainsTokens` rules; add `wolfSSL_CTX_set_verify` gap fix; exact integer_literal check for umask(0) |
| ARGVAL-02 | Migrated argument-value rules produce no new false positives vs v1.0.17 baseline on AUTOSAR_SampleProject_S32K144 | FP-guard fixtures; `ArgAtIndex` scopes token check to one positional arg instead of any arg; umask integer_literal exact check eliminates `umask(0077)` FP |
</phase_requirements>

---

## Summary

Phase 20 makes a targeted surgical change to `src/vulnerability/ast_scanner.rs`: replace the `ArgCheck::ContainsTokens` variant (which scans ALL named args for a token) with a new `ArgCheck::ArgAtIndex(u8, &'static [&'static str])` variant (which inspects one specific positional argument and walks its nested AST subtree). The three CWEs that currently use `ContainsTokens` — CWE-295, CWE-319, and CWE-732 — are migrated to `ArgAtIndex` rules. `ContainsTokens` is then deleted from the enum entirely.

The parallel change in `src/vulnerability/cwe_scanner.rs` removes the five `arg_value_contains: Some(...)` lexical rules for these three CWEs, along with the `arg_value_contains` field on `CweRule` and the `paren_args_contain_all` call in `scan_file`. After Phase 20, the AST scanner is the sole authoritative source for CWE-295/319/732; the lexical scanner retains them only via `wolfSSL_CTX_set_verify` — but that function name is being added to the AST rules to close the gap.

The key precision fix is CWE-732/umask: `ContainsTokens(&["0"])` fires on `umask(0077)` because the octal literal contains a '0' character. The replacement `ArgAtIndex(0, ...)` scopes the check to arg 0 only, AND the implementation checks `args[0].kind() == "integer_literal" && args[0].utf8_text(src) == Ok("0")` for an exact match, eliminating the false positive.

**Primary recommendation:** Implement `ArgAtIndex` by indexing the already-collected `args: Vec<Node>`, then recursively collecting leaf text from the selected arg's subtree and applying `token_present_with_boundary`. Pattern exactly mirrors how `NotStringLiteralAtIndex` accesses positional args — the same index-bounds guard applies.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ArgAtIndex evaluation | ast_scanner.rs (AST tier) | — | Tree-sitter node access is AST-only; lexical scanner cannot inspect argument AST nodes |
| ContainsTokens removal | ast_scanner.rs (AST tier) | — | Variant is used only in ast_scanner.rs |
| arg_value_contains cleanup | cwe_scanner.rs (Lexical tier) | — | Field and paren_args_contain_all live entirely in cwe_scanner.rs |
| umask(0) exact check | ast_scanner.rs (AST tier) | — | `node.kind() == "integer_literal"` is only accessible via tree-sitter node API |
| wolfSSL gap fix | ast_scanner.rs (AST tier) | — | Adds missing function to AST rule table |
| Synthetic fixture tests | tests/fixtures/c/ + ast_scanner.rs inline tests | — | Mirrors existing Phase 18 pattern for fixture-based TP/FP validation |

---

## Standard Stack

### Core (no new dependencies — everything already in Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tree-sitter | embedded (Phase 18) | AST node access: `node.kind()`, `node.utf8_text(src)`, `node.named_children()` | Project standard since Phase 18 |
| tree-sitter-c | embedded (Phase 18) | C grammar — produces `integer_literal`, `identifier`, `argument_list` nodes | Project standard since Phase 18 |

`token_present_with_boundary` from `cwe_scanner.rs` — already imported in `ast_scanner.rs` line 17 (`use crate::vulnerability::cwe_scanner::token_present_with_boundary`). [VERIFIED: codebase grep]

**Installation:** No new dependencies. Phase 20 is a pure code-refactor within existing crates.

---

## Architecture Patterns

### System Architecture Diagram

```
call_expression (AST node)
    │
    ├── function (field) → func_name: &str
    │
    └── arguments (field) → args: Vec<Node>
                               │
                               ├── args[0] → arg 0 subtree
                               ├── args[1] → arg 1 subtree
                               └── args[N] → arg N subtree

ArgCheck dispatch:
  ArgAtIndex(idx, tokens)
      │
      ├── idx >= args.len() → false (skip)
      ├── cwe_id 732 / func "umask" → special: kind() == "integer_literal" && text == "0"
      └── otherwise → collect_leaf_text(args[idx]) → for each tok: token_present_with_boundary(text, tok)
```

### Recommended Project Structure (unchanged from Phase 18)
```
src/vulnerability/
├── ast_scanner.rs       # PRIMARY: add ArgAtIndex, delete ContainsTokens, migrate rules
├── cwe_scanner.rs       # CLEANUP: remove arg_value_contains field + paren_args_contain_all
└── mod.rs               # CHECK: paren_args_contain_all re-export (none found — verified below)
tests/
├── fixtures/c/          # ADD: cwe_295_fixture.c, cwe_319_fixture.c, cwe_732_fixture.c
└── vulnerability_tests/
    └── ast_scanner_tests.rs   # ADD: test cases for migrated rules
```

### Pattern 1: ArgAtIndex Variant Implementation

**What:** New `ArgCheck` arm that inspects a specific positional argument's AST subtree.
**When to use:** When the dangerous condition is determined by a value at a known argument position.

```rust
// Source: [VERIFIED: codebase — ast_scanner.rs lines ~211-256]

// In the ArgCheck enum:
ArgAtIndex(u8, &'static [&'static str]),

// In the match block inside visit_node:
ArgCheck::ArgAtIndex(idx, tokens) => {
    let idx = *idx as usize;
    if idx >= args.len() {
        false // D-03: out of bounds — skip silently, mirrors NotStringLiteralAtIndex behavior
    } else {
        let arg_text = collect_subtree_text(args[idx], src);
        tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))
    }
}
```

### Pattern 2: Subtree Text Collection (new helper)

```rust
// Recursively collect leaf node texts from a subtree into a single string.
// Used by ArgAtIndex evaluation (D-02).
fn collect_subtree_text(node: Node, src: &[u8]) -> String {
    if node.child_count() == 0 {
        // Leaf node — return its text directly
        return node.utf8_text(src).unwrap_or("").to_string();
    }
    let mut result = String::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        let child_text = collect_subtree_text(child, src);
        if !child_text.is_empty() {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&child_text);
        }
    }
    result
}
```

**Note on separator:** Using a space separator between leaf texts is safe because `token_present_with_boundary` checks character boundaries. Joining without a separator is also acceptable since the tokens being searched (`SSL_VERIFY_NONE`, `CURLOPT_USE_SSL`, etc.) are identifiers with alphanumeric boundaries that would not accidentally merge across node boundaries.

### Pattern 3: umask Exact Integer Literal Check (D-10)

This is the clearest precision improvement. The existing `ContainsTokens(&["0"])` fires on `umask(0077)` because `token_present_with_boundary` is designed with right-boundary exclusions for digits, but the issue is that `umask(0077)` is represented as an `integer_literal` node with text `"0077"` — and this text does NOT contain a standalone `"0"` by `token_present_with_boundary` semantics. [VERIFIED: cwe_scanner.rs test `test_argval_cwe732_umask_octal_does_not_fire` already passes for the lexical scanner, meaning `token_present_with_boundary` correctly handles this.]

**However**, the AST rule uses `ContainsTokens` which scans ALL args, not just arg 0. If the call site has other args or nested identifiers in arg 0, a stray `0` could appear. The `ArgAtIndex(0, ...)` scope reduces the surface area. For maximum precision, the umask rule also checks the node kind:

```rust
// Special-cased inside ArgAtIndex evaluation for umask:
// The arg at index 0 must be an integer_literal with text exactly "0".
// This prevents matching umask(compute_mask(0)) where 0 is buried inside a call.
//
// Implementation: either check in the ArgAtIndex arm with a special-case,
// OR use a dedicated ArgCheck variant ArgExactIntAtIndex(u8, &'static str).
// D-10 says to check kind() AND text == "0" — this is cleanest as a special case
// inside the ArgAtIndex arm keyed on cwe_id, OR as a dedicated variant.
```

**Recommended implementation:** Add the kind-check inside a helper that `ArgAtIndex` calls, gated by an option on the variant. Since D-10 is the only exact-literal case, the simplest approach is to add logic inside the `ArgAtIndex` arm: if `tokens == &["0"]`, additionally require `args[idx].kind() == "integer_literal"`. This avoids a new enum variant while satisfying D-10.

Alternatively (cleaner): for `umask`, use `ArgAtIndex(0, &["0"])` and rely on the fact that `token_present_with_boundary` already correctly rejects `"0077"` — the right-boundary exclusion in `token_present_with_boundary` blocks hex/octal suffixes. Test this first before adding the kind check.

### Pattern 4: CWE-319 AND-Logic Across Two Positional Args

Three rules for `curl_easy_setopt`:
1. `ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` — checks that arg 1's subtree contains BOTH tokens. This is an AND-all of tokens within ONE arg, which `ArgAtIndex` already supports (the `tokens.iter().all(...)` loop).
2. `ArgAtIndex(1, &["CURLOPT_SSL_VERIFYPEER"])` — option-name-only detection for this arg; arg 2 being `0` is the dangerous value, but option name alone uniquely identifies the dangerous pattern (every call disabling peer verification is dangerous regardless of the specific value syntax).
3. `ArgAtIndex(1, &["CURLOPT_SSL_VERIFYHOST"])` — same as above.

The "AND across two positional args" concern from D-09 only applies if we want to detect `CURLOPT_SSL_VERIFYPEER` at arg 1 AND `0` at arg 2. Per D-09's resolution: **option-name-only detection is sufficient** — `ArgAtTwoIndices` is deferred. The existing lexical scanner rule `arg_value_contains: Some(&["CURLOPT_SSL_VERIFYPEER", "0"])` requires both tokens anywhere in the arg list; the AST rule with `ArgAtIndex(1, &["CURLOPT_SSL_VERIFYPEER"])` is already more precise (scoped to arg 1 only).

### Anti-Patterns to Avoid

- **Don't reuse `ContainsTokens` for the migrated rules:** It scans all args, not a specific position. Phase 20 purpose is positional scoping.
- **Don't walk unnamed AST children:** tree-sitter's `named_children()` skips punctuation nodes (commas, parens). Use `named_children` for the subtree walk, or `children` with a kind check, not `child(i)` positional indexing when collecting leaf texts.
- **Don't call `utf8_text` on non-leaf nodes expecting complete text:** `utf8_text` returns the source slice spanning the node's byte range, which includes all whitespace and child text. It is fine to use for collecting arg text, but `collect_subtree_text` using leaf nodes is more robust for concatenating identifiers without including punctuation.
- **Don't panic on args.len() < idx:** D-03 mandates silent `false`. Mirror `NotStringLiteralAtIndex` exactly.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Arg subtree text collection | Custom recursive walker with index tracking | `collect_subtree_text` helper (new, ~10 lines) using `named_children()` and `utf8_text()` | tree-sitter node API provides text directly; no custom indexing needed |
| Word-boundary token matching | New boundary-checking logic | `token_present_with_boundary` (already imported) | Existing function handles digit tokens, hex suffixes, alphanumeric boundaries correctly |
| Argument list extraction | Re-parse the source string | `node.child_by_field_name("arguments")` + `named_children()` | Phase 18 already collects `args: Vec<Node>` at every call site — reuse that |

---

## Common Pitfalls

### Pitfall 1: `ContainsTokens` and `ArgAtIndex` Fire Simultaneously During Migration

**What goes wrong:** If both the old `ContainsTokens` rule and the new `ArgAtIndex` rule are present in `AST_CWE_RULES` during development, a call site will produce two findings for the same CWE. The dedup logic in `run_lexical_scanner` operates per-scanner; `run_ast_scanner` does not internally dedup by rule — it dedupes only after `deduplicate_sast_findings` merges AST + lexical results.

**Why it happens:** The rule table is evaluated for every rule in sequence; both entries would fire on the same call site.

**How to avoid:** Replace all three `ContainsTokens` rules atomically in the same edit as adding the `ArgAtIndex` rules. Never have both active at once. Delete `ContainsTokens` from the enum in the same commit.

**Warning signs:** Test produces double findings for CWE-295/319/732 on a single call site.

### Pitfall 2: `named_children` vs `children` for Subtree Walk

**What goes wrong:** Using `node.children(&mut cursor)` includes unnamed nodes (commas, semicolons, parentheses) as leaf candidates. Their text would be included in the concatenated arg text and could corrupt word-boundary checks.

**Why it happens:** tree-sitter distinguishes named (semantic) and anonymous (punctuation) nodes. `named_children` returns only semantic nodes.

**How to avoid:** Use `node.named_children(&mut cursor)` in `collect_subtree_text`. If a node has zero named children but has unnamed children (e.g., an operator node), fall back to `node.utf8_text(src)` directly.

**Warning signs:** `token_present_with_boundary` returns false on a valid token because the concatenated text has commas or parens merged in.

### Pitfall 3: `umask(0)` vs `umask(0077)` — Confirm the Existing Token Boundary Already Works

**What goes wrong:** Assuming `ContainsTokens(&["0"])` on the current AST code fires on `umask(0077)`.

**Why it matters:** The lexical scanner test `test_argval_cwe732_umask_octal_does_not_fire` already passes, confirming `token_present_with_boundary` correctly rejects `"0077"`. So the question is whether the current AST `ContainsTokens` rule misfires on `umask(0077)`.

**Verification:** Write a test before migrating to confirm whether the current `ContainsTokens` rule produces an FP for `umask(0077)`. If it doesn't (because `token_present_with_boundary` handles it), then D-10's kind-check is a belt-and-suspenders improvement, not a FP fix. Either way, the kind-check is correct to add.

**How to avoid:** Test both `umask(0)` (should fire) and `umask(0077)` (should NOT fire) before and after migration.

### Pitfall 4: `paren_args_contain_all` May Be Exported

**What goes wrong:** Deleting `paren_args_contain_all` from `cwe_scanner.rs` (D-13) breaks callers outside that module.

**Why it happens:** `paren_args_contain_all` could be re-exported via `mod.rs`.

**Current state:** `mod.rs` re-exports are `deduplicate_sast_findings, has_c_cpp_files, run_lexical_scanner, SastFinding, SastSource` — `paren_args_contain_all` is NOT in this list. [VERIFIED: src/vulnerability/mod.rs line 16]

**How to avoid:** Grep `paren_args_contain_all` across `src/` before deletion. Currently only used in `cwe_scanner.rs` line ~155 and ~366. Safe to delete when the five `arg_value_contains: Some(...)` rules are removed.

### Pitfall 5: Lexical Scanner Tests for CWE-295/319/732 Must Still Pass

**What goes wrong:** After D-12 removes the `arg_value_contains` rules from `cwe_scanner.rs`, the existing tests `test_argval_cwe295_ssl_verify_none`, `test_argval_cwe319_*`, `test_argval_cwe732_*` in `cwe_scanner.rs` still call `scan_file` (the lexical scanner). If those tests relied on the `arg_value_contains` rules that are being deleted, they will fail.

**Current state:** The lexical scanner CWE-295/319/732 rules at lines 89–96 of `cwe_scanner.rs` ARE the `arg_value_contains` rules being deleted. The lexical scanner tests (lines ~579–648) call `scan_file` directly and will fail after D-12 deletes those rules.

**How to avoid:** Either (a) delete the lexical scanner tests for these CWEs alongside D-12 (since AST scanner tests replace them), or (b) update the tests to expect no finding from the lexical scanner. Option (a) is cleaner and matches the CONTEXT.md statement "After D-12, those tests will still pass (they test the lexical scanner)" — actually this is wrong per the analysis below.

**IMPORTANT CORRECTION:** CONTEXT.md code_context §"Integration Points" states: "After D-12, those tests will still pass (they test the lexical scanner)." This is INCORRECT — those tests WILL FAIL because D-12 deletes the rules those tests depend on. The plan must explicitly delete or update the six lexical-scanner `test_argval_cwe295/319/732` tests in `cwe_scanner.rs`.

### Pitfall 6: `arg_value_contains` Field Removal Breaks `CweRule` Struct Initializers

**What goes wrong:** `cwe_scanner.rs` has 21 `CweRule` entries, all with `arg_value_contains: None` or `arg_value_contains: Some(...)`. Removing the field from the struct means all 21 initializers need to drop that field.

**How to avoid:** Remove the field from the struct AND update all 21 initializers in one edit. The five `Some(...)` entries are deleted entirely (D-12); the remaining 16 `None` entries need the field dropped.

---

## Code Examples

### Adding ArgAtIndex to the Enum

```rust
// Source: [VERIFIED: codebase — ast_scanner.rs enum ArgCheck, current state]
// BEFORE:
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ContainsTokens(&'static [&'static str]),
    AnyCall,
}

// AFTER (Phase 20):
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ArgAtIndex(u8, &'static [&'static str]),  // NEW (D-01)
    AnyCall,
    // ContainsTokens DELETED (D-04)
}
```

### Migrated AST Rule Table Entries

```rust
// Source: [VERIFIED: codebase — ast_scanner.rs AST_CWE_RULES, current state + D-05/06/07/08/10/11]

// CWE-295 (D-05, D-06, D-07):
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"],
             arg_check: ArgCheck::ArgAtIndex(1, &["SSL_VERIFY_NONE"]) },
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_cert_verify_callback"],
             arg_check: ArgCheck::AnyCall },

// CWE-319 (D-08/D-09):
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"],
             arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"]) },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"],
             arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_SSL_VERIFYPEER"]) },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"],
             arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_SSL_VERIFYHOST"]) },

// CWE-732 (D-10, D-11):
AstCweRule { cwe_id: 732, functions: &["umask"],
             arg_check: ArgCheck::ArgAtIndex(0, &["0"]) },  // + kind check for integer_literal
AstCweRule { cwe_id: 732, functions: &["SetSecurityDescriptorDacl"],
             arg_check: ArgCheck::ArgAtIndex(2, &["NULL"]) },
```

### Removing arg_value_contains from CweRule

```rust
// Source: [VERIFIED: codebase — cwe_scanner.rs lines 49–64]
// BEFORE:
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
    arg_value_contains: Option<&'static [&'static str]>,  // DELETE THIS FIELD (D-13)
}

// AFTER:
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
}
```

### scan_file Cleanup

```rust
// Source: [VERIFIED: codebase — cwe_scanner.rs lines ~364–373]
// BEFORE:
if let Some(tokens) = rule.arg_value_contains {
    let after = &line[pos + func.len()..];
    if !paren_args_contain_all(after, tokens) {
        continue;
    }
}

// AFTER: Delete this block entirely (D-13). The paren_args_contain_all function
// also becomes dead code and must be deleted (lines ~155–183).
```

### Synthetic Fixture: CWE-732 umask

```c
/* tests/fixtures/c/cwe_732_umask_fixture.c */

/* TP: umask(0) — permissive, should fire */
void setup_insecure(void) {
    umask(0);
}

/* FP guard: umask(0077) — restrictive, must NOT fire */
void setup_secure(void) {
    umask(0077);
}

/* Phase 20 success criterion #3: dangerous arg buried in nested expression */
/* This is tricky for umask — umask(compute_umask()) is a call, not integer_literal.
   For ArgAtIndex with integer_literal check, this correctly does NOT fire (not a literal 0).
   The nested expression test for CWE-295 is more relevant: */

/* TP nested: SSL_CTX_set_verify with SSL_VERIFY_NONE in a parenthesized cast expression */
void ssl_nested(SSL_CTX *ctx) {
    SSL_CTX_set_verify(ctx, (int)SSL_VERIFY_NONE, NULL);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ContainsTokens` scans all args | `ArgAtIndex` scopes to one positional arg | Phase 20 | Eliminates cross-arg false positives |
| Lexical scanner owns CWE-295/319/732 | AST scanner owns CWE-295/319/732 | Phase 20 (D-12) | Parse-fail files lose these detections on fallback, but parse-fail is rare on well-formed C |
| wolfSSL covered only by lexical scanner | wolfSSL added to AST rule table | Phase 20 (D-06) | Closes gap where wolfSSL code in well-formed files would lose detection after D-12 |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `token_present_with_boundary` correctly rejects `"0077"` for token `"0"` | Common Pitfalls §3 | If it DOES fire on `"0077"`, the existing lexical scanner test `test_argval_cwe732_umask_octal_does_not_fire` would already be failing — it passes, so this is effectively verified |
| A2 | Lexical scanner tests for CWE-295/319/732 (`test_argval_cwe295_*`, `test_argval_cwe319_*`, `test_argval_cwe732_*` in cwe_scanner.rs) will fail after D-12 | Common Pitfalls §5 | If wrong (tests somehow still pass), Phase 20 still works but the planner needs to explicitly decide to delete or keep those tests |
| A3 | `collect_subtree_text` using `named_children` is sufficient to detect `SSL_VERIFY_NONE` inside `(int)SSL_VERIFY_NONE` | Code Examples §nested fixture | If named children don't include the identifier in a cast expression, the nested test case would fail; tree-sitter C grammar represents `(int)SSL_VERIFY_NONE` as a `cast_expression` with a named identifier child — this should work |

**If this table is empty (all verified):** A1 is verified by the passing test. A2 and A3 are medium-risk items the planner should address in task verification steps.

---

## Open Questions (RESOLVED)

1. **Should lexical scanner tests for CWE-295/319/732 be deleted or updated after D-12?**
   - What we know: D-12 removes the `arg_value_contains` rules those tests exercise. The tests call `scan_file` directly.
   - What's unclear: CONTEXT.md says "those tests will still pass" — this appears to be an error in the context since the tests specifically call `scan_file` which will no longer have those rules.
   - **RESOLVED:** Delete the six lexical-scanner `test_argval_cwe295_*`, `test_argval_cwe319_*`, `test_argval_cwe732_*` test functions in `cwe_scanner.rs` as part of Plan 02 (alongside D-12 rule removal). The AST scanner tests added in Plan 01 replace their validation role. CONTEXT.md's "those tests will still pass" note is incorrect and is superseded by this resolution.

2. **Does `collect_subtree_text` need to handle unnamed-node text for completeness?**
   - What we know: `named_children` skips punctuation. For tokens like `SSL_VERIFY_NONE` (pure identifiers), named children are sufficient.
   - What's unclear: Would a cast expression like `(SSL_VERIFY_NONE)` produce an identifier as a named child? In tree-sitter-c, a `parenthesized_expression` has its inner node as a named child, so yes.
   - **RESOLVED:** No. `collect_subtree_text` uses `named_children` recursively — unnamed nodes (commas, parens, operator punctuation) are not needed for the C `integer_literal` / `call_expression` / `cast_expression` use cases targeted by Phase 20. Identifiers, integer literals, and parenthesized/cast subexpressions all appear as named children in tree-sitter-c. The nested-cast test in Plan 01 Task 3 verifies this in practice.

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies. Phase 20 is a pure code change within existing Rust crates. tree-sitter-c grammar is already embedded in the project since Phase 18.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `cargo test` |
| Config file | none (inline tests in source files + integration tests in `tests/`) |
| Quick run command | `cargo test --features internal -- argval 2>&1` |
| Full suite command | `cargo test --features internal 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARGVAL-01 | CWE-295 ArgAtIndex fires on SSL_VERIFY_NONE at arg 1 | unit | `cargo test --features internal -- test_argval_cwe295_ast` | ❌ Wave 0 |
| ARGVAL-01 | CWE-295 does NOT fire on SSL_VERIFY_PEER (FP guard) | unit | `cargo test --features internal -- test_argval_cwe295_ast_no_fp` | ❌ Wave 0 |
| ARGVAL-01 | wolfSSL_CTX_set_verify fires on SSL_VERIFY_NONE (D-06 gap fix) | unit | `cargo test --features internal -- test_argval_cwe295_wolfssl` | ❌ Wave 0 |
| ARGVAL-01 | CWE-319 ArgAtIndex fires on CURLOPT_USE_SSL + CURLUSESSL_NONE at arg 1 | unit | `cargo test --features internal -- test_argval_cwe319_ast_use_ssl_none` | ❌ Wave 0 |
| ARGVAL-01 | CWE-319 ArgAtIndex fires on CURLOPT_SSL_VERIFYPEER at arg 1 | unit | `cargo test --features internal -- test_argval_cwe319_ast_verifypeer` | ❌ Wave 0 |
| ARGVAL-01 | CWE-319 does NOT fire on safe value 2 at arg 2 (option-name check still fires — verify this is acceptable) | unit | `cargo test --features internal -- test_argval_cwe319_ast_safe` | ❌ Wave 0 |
| ARGVAL-01 | CWE-732 umask ArgAtIndex fires on umask(0) | unit | `cargo test --features internal -- test_argval_cwe732_ast_umask_zero` | ❌ Wave 0 |
| ARGVAL-02 | CWE-732 umask does NOT fire on umask(0077) | unit | `cargo test --features internal -- test_argval_cwe732_ast_umask_octal_no_fp` | ❌ Wave 0 |
| ARGVAL-02 | CWE-732 SetSecurityDescriptorDacl fires with NULL at arg 2 | unit | `cargo test --features internal -- test_argval_cwe732_ast_dacl_null` | ❌ Wave 0 |
| ARGVAL-02 | Nested expression: dangerous arg buried in cast fires correctly (success criterion #3) | unit | `cargo test --features internal -- test_argval_nested_expression` | ❌ Wave 0 |
| ARGVAL-01/02 | Full test suite green after ContainsTokens deletion | integration | `cargo test --features internal` | ✅ exists |

### Sampling Rate
- **Per task commit:** `cargo test --features internal -- argval 2>&1 | tail -10`
- **Per wave merge:** `cargo test --features internal 2>&1 | tail -5`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] New test functions in `tests/vulnerability_tests/ast_scanner_tests.rs` — covers all ARGVAL-01/02 test cases above
- [ ] Synthetic fixture files `tests/fixtures/c/cwe_295_fixture.c`, `tests/fixtures/c/cwe_319_fixture.c`, `tests/fixtures/c/cwe_732_fixture.c` (or inline in test functions using `tempfile`)
- [ ] Decision: inline fixtures (using `b"..."` byte strings in test functions, as per Phase 18 pattern) vs. file-based fixtures under `tests/fixtures/c/` — both work; inline is simpler and mirrors existing `legacy_poc_tests` pattern in `ast_scanner.rs`

*(Note: D-14 says "create synthetic .c fixture files under `tests/fixtures/`" — but existing Phase 18 AST tests use inline `tempfile`-based fixtures in `tests/vulnerability_tests/ast_scanner_tests.rs`. Either approach is valid; the planner should pick one consistently.)*

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (indirectly) | The rules detect insecure API usage patterns that relate to input validation bypass (CWE-295 disables cert validation; CWE-319 disables TLS verification) |
| V6 Cryptography | yes (detection) | CWE-295/319 rules detect disabled TLS verification — the scanner reports misuse, not the TLS implementation itself |

### Known Threat Patterns for this Phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Phase 20 code change introduces a new false negative (missed finding) | T (Tampering) | Test suite verifies TP cases for each migrated rule; AUTOSAR fixture comparison validates no regression |
| `collect_subtree_text` walks an excessively deep subtree causing stack overflow | D (Denial of Service) | Rust default stack is 8MB; tree-sitter C AST depth is bounded by source nesting; no additional mitigation needed for typical source files |
| ArgAtIndex with large idx on a 0-arg function call | D (Denial of Service) | D-03 bounds check `idx >= args.len()` returns false immediately |

---

## Sources

### Primary (HIGH confidence)
- `src/vulnerability/ast_scanner.rs` — current ArgCheck enum, rule table, visit_node match block, args collection pattern [VERIFIED: codebase read]
- `src/vulnerability/cwe_scanner.rs` lines 49–97 — CweRule struct, arg_value_contains field, paren_args_contain_all function, five rules to delete [VERIFIED: codebase read]
- `src/vulnerability/mod.rs` — re-exports confirm paren_args_contain_all is not exported [VERIFIED: codebase read]
- `tests/vulnerability_tests/ast_scanner_tests.rs` — existing test patterns to mirror [VERIFIED: codebase read]

### Secondary (MEDIUM confidence)
- tree-sitter node API: `node.kind()`, `node.utf8_text()`, `node.named_children()`, `node.child_count()` — behavior derived from existing usage patterns in ast_scanner.rs [ASSUMED from codebase patterns; consistent with tree-sitter 0.x API]

### Tertiary (LOW confidence)
- tree-sitter-c grammar node kind `"integer_literal"` for `umask(0)` arg — this is the expected kind for numeric literal nodes in the C grammar; the exact kind name should be verified by a quick test or tree-sitter playground [ASSUMED based on tree-sitter-c grammar conventions]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new libraries; all existing crate usage verified in codebase
- Architecture: HIGH — implementation patterns directly extracted from existing ast_scanner.rs code
- Pitfalls: HIGH for §1-4 (verified by codebase analysis); MEDIUM for §5-6 (logic analysis, not run)
- Test gaps: HIGH — accurately reflects the current test file state

**Research date:** 2026-05-12
**Valid until:** 2026-06-12 (stable internal codebase — no external dependency churn)
