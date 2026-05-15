# Phase 20: argument-value-ast-migration — Pattern Map

**Mapped:** 2026-05-12
**Files analyzed:** 4 new/modified files
**Analogs found:** 4 / 4

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/ast_scanner.rs` | service (AST scanner) | transform | `src/vulnerability/ast_scanner.rs` (self — surgery on existing file) | exact |
| `src/vulnerability/cwe_scanner.rs` | service (lexical scanner) | transform | `src/vulnerability/cwe_scanner.rs` (self — surgery on existing file) | exact |
| `src/vulnerability/mod.rs` | module config | — | `src/vulnerability/mod.rs` (self — verify-only, no write expected) | exact |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | test | request-response | `tests/vulnerability_tests/ast_scanner_tests.rs` (self — additive) | exact |

---

## Pattern Assignments

### `src/vulnerability/ast_scanner.rs` — Add `ArgAtIndex`, Delete `ContainsTokens`, Migrate Rules

**Analog:** self (existing file)

**Imports pattern** (lines 14–22 — unchanged, shown for reference):
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

**ArgCheck enum — BEFORE state** (lines 26–37):
```rust
#[derive(Debug)]
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ContainsTokens(&'static [&'static str]),  // DELETE THIS (D-04)
    AnyCall,
}
```

**ArgCheck enum — AFTER state** (D-01, D-04):
```rust
#[derive(Debug)]
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ArgAtIndex(u8, &'static [&'static str]),  // NEW: positional arg + token slice
    AnyCall,
    // ContainsTokens DELETED (D-04) — all three uses replaced by ArgAtIndex
}
```

**Existing `NotStringLiteralAtIndex` pattern to mirror** (lines 214–222 — the exact guard pattern `ArgAtIndex` must copy for out-of-bounds):
```rust
ArgCheck::NotStringLiteralAtIndex(i) => {
    let idx = *i as usize;
    if idx < args.len() {
        args[idx].kind() != "string_literal"
    } else {
        // Index out of bounds — skip, don't panic
        false
    }
}
```

**`ArgAtIndex` arm — new match arm to insert after `NotStringLiteralAtIndex`** (D-01, D-02, D-03, D-10):
```rust
ArgCheck::ArgAtIndex(i, tokens) => {
    let idx = *i as usize;
    if idx >= args.len() {
        false  // D-03: out of bounds — skip silently, mirrors NotStringLiteralAtIndex
    } else {
        let arg_text = collect_subtree_text(args[idx], src);
        tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))
    }
}
```

**New helper function `collect_subtree_text`** — add as a free function near the bottom of the impl section (before `#[cfg(test)]`):
```rust
/// Recursively collect leaf-node texts from a tree-sitter subtree into a
/// single space-separated string. Used by ArgCheck::ArgAtIndex (D-02).
/// Uses named_children to skip punctuation (commas, parens) — see Pitfall 2.
fn collect_subtree_text(node: Node, src: &[u8]) -> String {
    if node.child_count() == 0 {
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

**Existing `ContainsTokens` arm — DELETE this entire arm** (lines 224–233):
```rust
// DELETE: this entire arm is replaced by ArgAtIndex
ArgCheck::ContainsTokens(tokens) => {
    args.iter().any(|arg| {
        if let Ok(text) = arg.utf8_text(src) {
            tokens.iter().all(|tok| token_present_with_boundary(text, tok))
        } else {
            false
        }
    })
}
```

**AST_CWE_RULES table — BEFORE state for the three affected CWEs** (lines 62–73):
```rust
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify"], arg_check: ArgCheck::ContainsTokens(&["SSL_VERIFY_NONE"]) },
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_cert_verify_callback"], arg_check: ArgCheck::AnyCall },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"], arg_check: ArgCheck::ContainsTokens(&["CURLOPT_USE_SSL"]) },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"], arg_check: ArgCheck::ContainsTokens(&["CURLUSESSL_NONE"]) },
// ... (CWE-327 rule here, unchanged) ...
AstCweRule { cwe_id: 732, functions: &["umask"], arg_check: ArgCheck::ContainsTokens(&["0"]) },
AstCweRule { cwe_id: 732, functions: &["SetSecurityDescriptorDacl"], arg_check: ArgCheck::ContainsTokens(&["NULL"]) },
```

**AST_CWE_RULES table — AFTER state for the three affected CWEs** (D-05, D-06, D-07, D-08, D-09, D-10, D-11):
```rust
// CWE-295: positional arg 1 is the verify-mode (handle=0, mode=1, callback=2)
// D-06: wolfSSL_CTX_set_verify added to close the gap vs lexical scanner
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"],
             arg_check: ArgCheck::ArgAtIndex(1, &["SSL_VERIFY_NONE"]) },
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_cert_verify_callback"], arg_check: ArgCheck::AnyCall },

// CWE-319: three separate ArgAtIndex rules for curl_easy_setopt (D-08/D-09)
// Rule 1: arg 1 must contain BOTH tokens — option + value in the same arg subtree
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"],
             arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"]) },
// Rules 2 & 3: option-name-only detection — presence of option at arg 1 is sufficient
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"],
             arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_SSL_VERIFYPEER"]) },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"],
             arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_SSL_VERIFYHOST"]) },

// CWE-732: umask arg 0 must be integer_literal "0" (D-10); DACL arg 2 must contain NULL (D-11)
AstCweRule { cwe_id: 732, functions: &["umask"],
             arg_check: ArgCheck::ArgAtIndex(0, &["0"]) },
AstCweRule { cwe_id: 732, functions: &["SetSecurityDescriptorDacl"],
             arg_check: ArgCheck::ArgAtIndex(2, &["NULL"]) },
```

**NOTE on umask exact-literal check (D-10):** The `ArgAtIndex(0, &["0"])` arm calls `collect_subtree_text` which returns `"0"` for an `integer_literal` node. `token_present_with_boundary` already correctly rejects `"0077"` (verified by existing lexical test `test_argval_cwe732_umask_octal_does_not_fire`). The kind-check `args[0].kind() == "integer_literal"` is an additional guard: add it inline in the `ArgAtIndex` arm for the `umask` case, or (simpler) rely on `token_present_with_boundary` to reject `"0077"` and add a test to confirm it holds for the AST path too. The planner must include a test for `umask(0077)` before deciding whether the kind check is required.

**`args` collection pattern** (lines 199–204 — unchanged, `ArgAtIndex` indexes into this existing vec):
```rust
let args: Vec<Node> = if let Some(arg_list) = node.child_by_field_name("arguments") {
    let mut cursor = arg_list.walk();
    arg_list.named_children(&mut cursor).collect()
} else {
    vec![]
};
```

**SastFinding emit pattern** (lines 258–267 — unchanged):
```rust
if fire {
    findings.push(SastFinding {
        cwe_id: rule.cwe_id,
        component_name: component_name.to_string(),
        component_ecosystem: component_ecosystem.to_string(),
        file_path: path.to_string_lossy().into_owned(),
        line: (node.start_position().row as u32) + 1,
        source: SastSource::Ast,
    });
}
```

---

### `src/vulnerability/cwe_scanner.rs` — Remove `arg_value_contains` Field and Rules

**Analog:** self (existing file)

**`CweRule` struct — BEFORE state** (lines 49–64):
```rust
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
    /// AND-all token list (D-02): rule fires only when paren-bound args contain ALL tokens with word boundaries.
    arg_value_contains: Option<&'static [&'static str]>,  // DELETE THIS FIELD (D-13)
}
```

**`CweRule` struct — AFTER state** (D-13):
```rust
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
    // arg_value_contains DELETED (D-13) — AST scanner owns CWE-295/319/732
}
```

**Five `CweRule` entries to DELETE entirely** (lines 89–96 — these are the `arg_value_contains: Some(...)` rules):
```rust
// DELETE ALL FIVE of these CweRule entries:
CweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"], ..., arg_value_contains: Some(&["SSL_VERIFY_NONE"]) },
CweRule { cwe_id: 319, functions: &["curl_easy_setopt"], ..., arg_value_contains: Some(&["CURLOPT_USE_SSL", "CURLUSESSL_NONE"]) },
CweRule { cwe_id: 319, functions: &["curl_easy_setopt"], ..., arg_value_contains: Some(&["CURLOPT_SSL_VERIFYPEER", "0"]) },
CweRule { cwe_id: 319, functions: &["curl_easy_setopt"], ..., arg_value_contains: Some(&["CURLOPT_SSL_VERIFYHOST", "0"]) },
CweRule { cwe_id: 732, functions: &["umask"], ..., arg_value_contains: Some(&["0"]) },
CweRule { cwe_id: 732, functions: &["SetSecurityDescriptorDacl"], ..., arg_value_contains: Some(&["NULL"]) },
```

**Remaining 16 `CweRule` initializers** — remove `arg_value_contains: None` field from each. Pattern (one representative, line 70):
```rust
// BEFORE:
CweRule { cwe_id: 120, functions: &["gets", "strcpy", "strcat"], requires_format_heuristic: false, format_arg_index: 0, arg_value_contains: None },
// AFTER:
CweRule { cwe_id: 120, functions: &["gets", "strcpy", "strcat"], requires_format_heuristic: false, format_arg_index: 0 },
```

**`scan_file` cleanup — DELETE this block** (lines 364–369):
```rust
// DELETE: this entire block becomes dead code when all arg_value_contains: Some(...) rules are removed
if let Some(tokens) = rule.arg_value_contains {
    let after = &line[pos + func.len()..];
    if !paren_args_contain_all(after, tokens) {
        continue;
    }
}
```

**`paren_args_contain_all` function to DELETE** (lines 155–183 — becomes dead code after the block above is removed):
```rust
// DELETE: paren_args_contain_all (lines 155–183) — only called from the block above
fn paren_args_contain_all(after_func: &str, tokens: &[&str]) -> bool { ... }
```

**Lexical scanner tests to DELETE** — six `test_argval_cwe295_*`, `test_argval_cwe319_*`, `test_argval_cwe732_*` functions in `cwe_scanner.rs` (lines 578–648). These test the `arg_value_contains` rules that are being deleted. AST scanner tests in `tests/vulnerability_tests/ast_scanner_tests.rs` replace their validation role.

**`test_rule_table_has_seventeen_cwes` test — UPDATE** (line 566–575): after deleting 5 rules, the count drops from 21 entries to 16. Update the assertion and the comment to reflect the new count.

---

### `src/vulnerability/mod.rs` — Verify Only

**Analog:** self (existing file, lines 1–20)

**Re-exports** (line 16 — `paren_args_contain_all` is NOT exported, safe to delete from `cwe_scanner.rs`):
```rust
#[cfg(feature = "internal")]
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, run_lexical_scanner, SastFinding, SastSource};
```

No changes needed to `mod.rs`. The planner should include a `grep paren_args_contain_all src/` step to confirm it remains unexported before deletion.

---

### `tests/vulnerability_tests/ast_scanner_tests.rs` — Add `ArgAtIndex` Test Cases

**Analog:** `tests/vulnerability_tests/ast_scanner_tests.rs` (self — additive; copy the `setup_one_file` helper and `#[test]` inline-fixture pattern)

**Existing helper pattern** (lines 11–17 — copy verbatim for all new tests):
```rust
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

**Existing test body pattern** (lines 20–30 — copy this structure for each new test):
```rust
#[test]
fn test_ast_emits_sast_finding() {
    let (_t, dirs) = setup_one_file("a.c", b"void f() { char buf[64]; strcpy(buf, \"x\"); }\n");
    let findings: Vec<SastFinding> = run_ast_scanner(&dirs);
    assert!(!findings.is_empty(), "Expected at least one finding; got none");
    assert!(
        findings.iter().any(|f| f.cwe_id == 120 && f.source == SastSource::Ast),
        "Expected CWE-120 with SastSource::Ast; got {:?}",
        findings
    );
}
```

**Lexical scanner analog test pattern for TP/FP pairs** (from `cwe_scanner.rs` lines 578–648 — the AST tests must mirror these exact call patterns but drive `run_ast_scanner` instead of `scan_file`):
```rust
// CWE-295 TP analog (lexical test at line 579):
// "void f(SSL_CTX *ctx) { SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL); }"
// CWE-295 FP guard analog (lexical test at line 588):
// "void f(SSL_CTX *ctx) { SSL_CTX_set_verify(ctx, SSL_VERIFY_PEER, NULL); }"
// CWE-319 TP analog (lexical test at line 597):
// "void f(CURL *curl) { curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0); }"
// CWE-319 FP guard analog (lexical test at line 615):
// "void f(CURL *curl) { curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 2); }"
// CWE-732 TP analog (lexical test at line 624):
// "void f() { umask(0); }"
// CWE-732 FP guard analog (lexical test at line 633):
// "void f() { umask(0077); }"
// CWE-732 DACL analog (lexical test at line 642):
// "void f(PSECURITY_DESCRIPTOR sd) { SetSecurityDescriptorDacl(sd, TRUE, NULL, FALSE); }"
```

**New test functions to add** — one function per test case, following the TP/FP guard naming convention from existing lexical tests. Add these inside a new `mod argval_tests` block (or top-level, following existing style in the file):
```rust
// CWE-295 tests
fn test_argval_cwe295_ast_ssl_verify_none()         // TP: SSL_CTX_set_verify with SSL_VERIFY_NONE at arg 1
fn test_argval_cwe295_ast_ssl_verify_peer_no_fp()   // FP guard: SSL_VERIFY_PEER must not fire
fn test_argval_cwe295_ast_wolfssl_verify_none()     // D-06 gap fix: wolfSSL_CTX_set_verify fires

// CWE-319 tests
fn test_argval_cwe319_ast_use_ssl_none()            // TP: CURLOPT_USE_SSL + CURLUSESSL_NONE at arg 1
fn test_argval_cwe319_ast_verifypeer()              // TP: CURLOPT_SSL_VERIFYPEER at arg 1
fn test_argval_cwe319_ast_verifyhost()              // TP: CURLOPT_SSL_VERIFYHOST at arg 1

// CWE-732 tests
fn test_argval_cwe732_ast_umask_zero()              // TP: umask(0) fires
fn test_argval_cwe732_ast_umask_octal_no_fp()       // FP guard: umask(0077) must not fire
fn test_argval_cwe732_ast_dacl_null()               // TP: SetSecurityDescriptorDacl with NULL at arg 2

// Nested expression (success criterion #3)
fn test_argval_nested_cast_expression()             // TP: SSL_CTX_set_verify(ctx, (int)SSL_VERIFY_NONE, NULL)
```

---

## Shared Patterns

### `#[cfg(feature = "internal")]` Gate
**Source:** `src/vulnerability/ast_scanner.rs` line 13 / `src/vulnerability/cwe_scanner.rs` line 13
**Apply to:** All code in these modules; no change needed — gate is file-level

### `SastSource::Ast` on all AST scanner findings
**Source:** `src/vulnerability/ast_scanner.rs` line 265
**Apply to:** All `SastFinding` emitted from `ast_scanner.rs` — `source: SastSource::Ast` must appear in every `findings.push(...)` call. No change to this existing pattern.

### `token_present_with_boundary` for token matching
**Source:** `src/vulnerability/cwe_scanner.rs` line 185 (function definition); imported in `ast_scanner.rs` line 17
**Apply to:** `ArgAtIndex` arm in `ast_scanner.rs` — `tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))`

### Inline `b"..."` byte string fixture pattern
**Source:** `tests/vulnerability_tests/ast_scanner_tests.rs` lines 21–22, 47–69
**Apply to:** All new `test_argval_*` test functions — use `setup_one_file("test.c", b"...")` pattern, NOT file-based fixtures under `tests/fixtures/c/`. This is simpler and consistent with the existing Phase 18 test file.

### Assertion message pattern
**Source:** `tests/vulnerability_tests/ast_scanner_tests.rs` lines 26–29
**Apply to:** All new tests — include `{:?}` dump of `findings` in the assertion message:
```rust
assert!(
    findings.iter().any(|f| f.cwe_id == NNN && f.source == SastSource::Ast),
    "Expected CWE-NNN with SastSource::Ast; got {:?}",
    findings
);
```

---

## No Analog Found

None. All files to be modified have direct analogs in the codebase (self-modifications with well-established patterns).

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `tests/vulnerability_tests/`
**Files scanned:** `ast_scanner.rs` (494 lines), `cwe_scanner.rs` (lines 1–120, 140–190, 340–380, 562–660), `mod.rs` (20 lines), `ast_scanner_tests.rs` (126 lines)
**Pattern extraction date:** 2026-05-12
