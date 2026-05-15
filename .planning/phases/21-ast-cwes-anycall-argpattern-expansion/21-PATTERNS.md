# Phase 21: ast-cwes-anycall-argpattern-expansion - Pattern Map

**Mapped:** 2026-05-12
**Files analyzed:** 2 (1 modified, 1 extended)
**Analogs found:** 2 / 2

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/ast_scanner.rs` | service (AST analysis engine) | transform (AST → SastFinding) | `src/vulnerability/ast_scanner.rs` (current state) | exact — all new code extends this single file |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | test | request-response (inline C fixture → assertion) | `tests/vulnerability_tests/ast_scanner_tests.rs` (current state) | exact — new test functions appended to existing file |

---

## Pattern Assignments

### `src/vulnerability/ast_scanner.rs` — primary modification

**Analog:** `src/vulnerability/ast_scanner.rs` (self, current state)

---

#### 1. File header and feature gate (lines 1–22)

```rust
//! Phase 18 (v1.0.18): Production AST-based CWE scanner using tree-sitter-c.
//!
//! CWE coverage (Phase 18, per 18-02-PLAN.md):
//!   AST-detected (13 CWEs): 78, 119, 120, 122, 125, 134, 190, 242, 295, 319, 327, 377, 732
//!   ...

#![cfg(feature = "internal")]

use crate::util::warn_on_walkdir_err;
use crate::vulnerability::cwe_scanner::scan_file as lexical_scan_file;
use crate::vulnerability::cwe_scanner::token_present_with_boundary;
use crate::vulnerability::{SastFinding, SastSource};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser};
use walkdir::WalkDir;
```

**Phase 21 change:** Update the module-level doc comment CWE coverage list to include the 12 new CWEs. Add 25 (net) to the "AST-detected" line. No new `use` imports required.

---

#### 2. ArgCheck enum — add SizeofPointer variant (lines 25–37)

**Current state (lines 25–37):**
```rust
#[derive(Debug)]
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ContainsTokens(&'static [&'static str]),   // NOTE: Phase 20 deletes this; verify final state
    AnyCall,
}
```

**Phase 21 addition — append SizeofPointer after AnyCall:**
```rust
#[derive(Debug)]
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ArgAtIndex(u8, &'static [&'static str]),   // added Phase 20; ANY-OF token semantics
    AnyCall,
    SizeofPointer,    // NEW Phase 21 (D-04): sizeof(ptr) where ptr is pointer_declarator
}
```

**Critical:** Read the actual file after Phase 20 executes to confirm `ContainsTokens` was deleted and `ArgAtIndex` is present before adding `SizeofPointer`. The current `src/vulnerability/ast_scanner.rs` on disk still has `ContainsTokens` (Phase 20 is incomplete as of this mapping date).

---

#### 3. AST_CWE_RULES static table — 12 new entries (lines 50–78)

**Existing pattern to mirror (lines 51–77):**
```rust
static AST_CWE_RULES: &[AstCweRule] = &[
    AstCweRule { cwe_id: 78,  functions: &["system", "popen", ...], arg_check: ArgCheck::AnyCall },
    AstCweRule { cwe_id: 119, functions: &["strncpy", ...],         arg_check: ArgCheck::FixedSizeBuffer },
    AstCweRule { cwe_id: 327, functions: &["MD5", "MD5_Init", ...], arg_check: ArgCheck::AnyCall },
];
```

**New entries to append (one per new CWE, inside the `&[...]` before the closing `]`):**

```rust
    // --- Phase 21 additions ---

    // CWE-121: Stack-Based Buffer Overflow — AnyCall on alloca (D-13/D-14)
    AstCweRule { cwe_id: 121, functions: &["alloca"], arg_check: ArgCheck::AnyCall },

    // CWE-126: Buffer Over-Read — FixedSizeBuffer (D-06; mirrors CWE-119/120/122/125 pattern)
    AstCweRule { cwe_id: 126, functions: &["strcat", "strncat"], arg_check: ArgCheck::FixedSizeBuffer },

    // CWE-328: Reversible One-Way Hash — ArgAtIndex on CryptCreateHash alg parameter (index 1)
    // CALG_MD2, CALG_MD5, CALG_SHA1 are weak; ANY-OF semantics (one entry per token if ALL-OF)
    AstCweRule { cwe_id: 328, functions: &["CryptCreateHash"], arg_check: ArgCheck::ArgAtIndex(1, &["CALG_MD2", "CALG_MD5", "CALG_SHA1"]) },

    // CWE-338: Weak PRNG — AnyCall (D-13/D-14; Juliet uses rand())
    AstCweRule { cwe_id: 338, functions: &["rand", "random", "srand"], arg_check: ArgCheck::AnyCall },

    // CWE-369: Divide by Zero — NOT in AST_CWE_RULES (D-01); handled by apply_division_rules()

    // CWE-426: Untrusted Search Path — AnyCall; popen/system duplicate CWE-78 findings,
    //   accepted per design (dedup on (file, line, cwe_id) allows different CWE IDs)
    AstCweRule { cwe_id: 426, functions: &["popen", "_popen", "system"], arg_check: ArgCheck::AnyCall },

    // CWE-467: sizeof on Pointer Type — SizeofPointer variant (D-04/D-11)
    AstCweRule { cwe_id: 467, functions: &["malloc", "calloc", "realloc", "memcpy", "memset", "memmove"], arg_check: ArgCheck::SizeofPointer },

    // CWE-526: Info Exposure through Environment Variables — AnyCall (Juliet uses getenv())
    AstCweRule { cwe_id: 526, functions: &["getenv"], arg_check: ArgCheck::AnyCall },

    // CWE-535: Info Exposure through Shell Error — ArgAtIndex(0) on fprintf/vfprintf with "stderr"
    AstCweRule { cwe_id: 535, functions: &["fprintf", "vfprintf"], arg_check: ArgCheck::ArgAtIndex(0, &["stderr"]) },

    // CWE-676: Use of Potentially Dangerous Functions — tight list (D-08/D-09)
    //   alloca: unconditional stack allocation; strtok: non-reentrant
    //   Excludes gets/system/rand/getenv (covered by CWE-120/78/338/526)
    AstCweRule { cwe_id: 676, functions: &["alloca", "strtok"], arg_check: ArgCheck::AnyCall },

    // CWE-680: Integer Overflow to Buffer Overflow — AnyCall on allocation functions
    //   (Juliet uses malloc(data * sizeof(int))); duplicates CWE-190 at different CWE ID — acceptable
    AstCweRule { cwe_id: 680, functions: &["malloc", "calloc", "realloc"], arg_check: ArgCheck::AnyCall },

    // CWE-780: RSA Without OAEP Padding — OpenSSL rule (D-10); synthetic fixture needed for TP
    AstCweRule { cwe_id: 780, functions: &["RSA_public_encrypt"], arg_check: ArgCheck::ArgAtIndex(4, &["RSA_PKCS1_PADDING", "RSA_NO_PADDING"]) },
```

**CWE-328 token semantics caution:** If Phase 20 `ArgAtIndex` uses ALL-OF semantics (all tokens present simultaneously), split CWE-328 into three separate entries:
```rust
    AstCweRule { cwe_id: 328, functions: &["CryptCreateHash"], arg_check: ArgCheck::ArgAtIndex(1, &["CALG_MD2"]) },
    AstCweRule { cwe_id: 328, functions: &["CryptCreateHash"], arg_check: ArgCheck::ArgAtIndex(1, &["CALG_MD5"]) },
    AstCweRule { cwe_id: 328, functions: &["CryptCreateHash"], arg_check: ArgCheck::ArgAtIndex(1, &["CALG_SHA1"]) },
```
Verify by reading the Phase 20 `ArgAtIndex` arm implementation (look for `tokens.iter().all(...)` = ALL-OF vs `tokens.iter().any(...)` = ANY-OF).

---

#### 4. ArgCheck::SizeofPointer arm in visit_node() — new match arm (after line 258)

**Model on the existing ArgCheck::FixedSizeBuffer arm (lines 238–258):**
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

**New SizeofPointer arm — same enclosing-scope lookup, different declarator kind:**
```rust
ArgCheck::SizeofPointer => {
    // Scan ALL args for a sizeof_expression whose value is a pointer-typed identifier
    args.iter().any(|arg| {
        if arg.kind() == "sizeof_expression" {
            // tree-sitter-c: sizeof_expression has field "value"
            if let Some(value_node) = arg.child_by_field_name("value") {
                if value_node.kind() == "identifier" {
                    if let Ok(ident_name) = value_node.utf8_text(src) {
                        // Check if ident_name is declared as pointer_declarator in enclosing scope
                        let fn_scope = find_enclosing_function(node);
                        if let Some(fn_node) = fn_scope {
                            let ptrs = collect_function_scope_pointer_declarators(fn_node, src);
                            return ptrs.contains(ident_name);
                        }
                        // file-scope pointer declarators (less common but handle it)
                        let file_ptrs = collect_file_scope_pointer_declarators(*root_for_file_scope*, src);
                        return file_ptrs.contains(ident_name);
                    }
                }
            }
        }
        false
    })
}
```

**Note:** `collect_function_scope_pointer_declarators()` is a new helper mirroring `collect_function_scope_fixed_arrays()` (lines 343–367) but collecting `pointer_declarator` nodes instead of `array_declarator` nodes. See Shared Patterns section for implementation template.

---

#### 5. apply_division_rules() — new standalone function (D-01/D-02)

**Call site change in scan_file_ast_or_lexical() (currently lines 154–161):**
```rust
// CURRENT (lines 154-160):
apply_ast_rules(
    tree.root_node(),
    code.as_bytes(),
    path,
    component_name,
    component_ecosystem,
)

// PHASE 21 CHANGE:
let mut findings = apply_ast_rules(
    tree.root_node(),
    code.as_bytes(),
    path,
    component_name,
    component_ecosystem,
);
apply_division_rules(
    tree.root_node(),
    code.as_bytes(),
    path,
    component_name,
    component_ecosystem,
    &mut findings,
);
findings
```

**New function signatures mirroring apply_ast_rules() (lines 164–185) and visit_node() (lines 188–294):**
```rust
/// CWE-369: Walk binary_expression nodes for literal divide-by-zero.
/// Called alongside apply_ast_rules() from scan_file_ast_or_lexical().
/// Infrastructure intentionally reusable by Phase 22 CWE-480/481/482 (wrong operator detection).
fn apply_division_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    visit_binary_exprs(root, src, path, component_name, component_ecosystem, findings);
}

fn visit_binary_exprs<'a>(
    node: Node<'a>,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "binary_expression" {
        // child(1) is the operator token in tree-sitter-c binary_expression
        if let Some(op_node) = node.child(1) {
            if let Ok(op) = op_node.utf8_text(src) {
                if op == "/" || op == "%" {
                    if let Some(rhs) = node.child_by_field_name("right") {
                        // VERIFY: tree-sitter-c may use "number_literal" (not "integer_literal")
                        // Check by printing rhs.kind() in a test on `int x = 5 / 0;`
                        if rhs.kind() == "number_literal" || rhs.kind() == "integer_literal" {
                            if let Ok(text) = rhs.utf8_text(src) {
                                if text == "0" {
                                    findings.push(SastFinding {
                                        cwe_id: 369,
                                        component_name: component_name.to_string(),
                                        component_ecosystem: component_ecosystem.to_string(),
                                        file_path: path.to_string_lossy().into_owned(),
                                        line: (node.start_position().row as u32) + 1,
                                        source: SastSource::Ast,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Recurse — fresh cursor per level (mirrors visit_node() Pitfall 1 pattern, lines 276-293)
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            visit_binary_exprs(cursor.node(), src, path, component_name, component_ecosystem, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

**Reusability note for Phase 22:** The `visit_binary_exprs` function should accept a closure or a rule table parameter (similar to `AST_CWE_RULES` for `visit_node`) so Phase 22 can add CWE-480/481/482 without a second recursive walk. Consider a signature like:
```rust
fn visit_binary_exprs<'a, F>(node: Node<'a>, src: &[u8], path: &Path, ..., findings: &mut Vec<SastFinding>, check: &F)
where F: Fn(&str, Node<'a>, &[u8]) -> Option<u32>   // (operator, node, src) → Option<cwe_id>
```
But this is discretionary — the planner decides based on Phase 22 readiness.

---

### `tests/vulnerability_tests/ast_scanner_tests.rs` — new test functions appended

**Analog:** `tests/vulnerability_tests/ast_scanner_tests.rs` (existing file, lines 1–125)

---

#### 1. File header and imports (lines 1–16) — copy verbatim, no changes

```rust
#![cfg(feature = "internal")]
//! Phase 18 Plan 02 AST scanner integration tests.
//!
//! Tests AST-01..AST-04 acceptance criteria from 18-02-PLAN.md.

use radeis_sc2sbom::vulnerability::{run_ast_scanner, SastFinding, SastSource};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
```

---

#### 2. setup_one_file() helper (lines 11–17) — already present; do NOT duplicate

```rust
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

---

#### 3. New test function template — mirrors existing TP/FP pattern (lines 20–30 and 43–92)

**TP + FP guard pattern (copy from lines 43–92):**
```rust
#[test]
fn test_cwe_NNN_description() {
    // TRUE POSITIVE
    let tp_code = b"void f() { /* call that should fire */ }\n";
    let (_t, dirs) = setup_one_file("tp.c", tp_code);
    let findings = run_ast_scanner(&dirs);
    assert!(
        findings.iter().any(|f| f.cwe_id == NNN && f.source == SastSource::Ast),
        "Expected CWE-NNN TP; got {:?}", findings
    );

    // FALSE POSITIVE GUARD
    let fp_code = b"void f() { /* safe version that should NOT fire */ }\n";
    let (_t2, dirs2) = setup_one_file("fp.c", fp_code);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(
        !findings2.iter().any(|f| f.cwe_id == NNN && f.source == SastSource::Ast),
        "CWE-NNN fired on safe code (false positive): {:?}", findings2
    );
}
```

**Concrete new test functions required (from RESEARCH.md Wave 0 gaps):**

```rust
#[test]
fn test_cwe_121_anycall_alloca() {
    // TP: alloca call fires CWE-121
    let tp = b"#include <alloca.h>\nvoid f(int n) { void *p = alloca(n); }\n";
    let (_t, dirs) = setup_one_file("cwe121.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 121 && f.source == SastSource::Ast),
        "CWE-121: expected alloca to fire; got {:?}", findings);
}

#[test]
fn test_cwe_126_fixed_size_buffer() {
    // TP: strcat into fixed-size array fires CWE-126
    let tp = b"void f(const char *extra) { char buf[64]; strcat(buf, extra); }\n";
    let (_t, dirs) = setup_one_file("cwe126_tp.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 126 && f.source == SastSource::Ast),
        "CWE-126: expected strcat(buf[64]) to fire; got {:?}", findings);

    // FP guard: strcat into heap pointer does NOT fire
    let fp = b"void f(char *dst, const char *extra) { strcat(dst, extra); }\n";
    let (_t2, dirs2) = setup_one_file("cwe126_fp.c", fp);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(!findings2.iter().any(|f| f.cwe_id == 126 && f.source == SastSource::Ast),
        "CWE-126: strcat into pointer param should not fire; got {:?}", findings2);
}

#[test]
fn test_cwe_328_weak_hash_argindex() {
    // TP: CryptCreateHash with CALG_MD2 fires CWE-328
    let tp = b"void f(HCRYPTPROV p, HCRYPTHASH *h) { CryptCreateHash(p, CALG_MD2, 0, 0, h); }\n";
    let (_t, dirs) = setup_one_file("cwe328_tp.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 328 && f.source == SastSource::Ast),
        "CWE-328: expected CALG_MD2 to fire; got {:?}", findings);

    // FP guard: CryptCreateHash with strong algorithm does NOT fire
    let fp = b"void f(HCRYPTPROV p, HCRYPTHASH *h) { CryptCreateHash(p, CALG_SHA_256, 0, 0, h); }\n";
    let (_t2, dirs2) = setup_one_file("cwe328_fp.c", fp);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(!findings2.iter().any(|f| f.cwe_id == 328 && f.source == SastSource::Ast),
        "CWE-328: CALG_SHA_256 should not fire; got {:?}", findings2);
}

#[test]
fn test_cwe_338_weak_prng() {
    let tp = b"#include <stdlib.h>\nvoid f() { int x = rand(); }\n";
    let (_t, dirs) = setup_one_file("cwe338.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 338 && f.source == SastSource::Ast),
        "CWE-338: expected rand() to fire; got {:?}", findings);
}

#[test]
fn test_cwe_369_division_literal_zero() {
    // TP: integer literal 0 as divisor fires CWE-369
    let tp = b"void f(int x) { int y = x / 0; }\n";
    let (_t, dirs) = setup_one_file("cwe369_tp.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 369 && f.source == SastSource::Ast),
        "CWE-369: expected x/0 to fire; got {:?}", findings);

    // FP guard: non-zero divisor does NOT fire
    let fp = b"void f(int x) { int y = x / 10; }\n";
    let (_t2, dirs2) = setup_one_file("cwe369_fp.c", fp);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(!findings2.iter().any(|f| f.cwe_id == 369 && f.source == SastSource::Ast),
        "CWE-369: x/10 should not fire; got {:?}", findings2);
}

#[test]
fn test_cwe_426_untrusted_search_path() {
    let tp = b"void f(char *cmd) { popen(cmd, \"r\"); }\n";
    let (_t, dirs) = setup_one_file("cwe426.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 426 && f.source == SastSource::Ast),
        "CWE-426: expected popen() to fire; got {:?}", findings);
}

#[test]
fn test_cwe_467_sizeof_pointer() {
    // TP: sizeof(ptr) where ptr is char* fires CWE-467
    let tp = b"#include <stdlib.h>\nvoid f() { char *badChar = NULL; malloc(sizeof(badChar)); }\n";
    let (_t, dirs) = setup_one_file("cwe467_tp.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 467 && f.source == SastSource::Ast),
        "CWE-467: expected sizeof(char*) to fire; got {:?}", findings);

    // FP guard: sizeof(*ptr) or sizeof(char) should not fire
    let fp = b"#include <stdlib.h>\nvoid f() { char *p = NULL; malloc(sizeof(*p)); }\n";
    let (_t2, dirs2) = setup_one_file("cwe467_fp.c", fp);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(!findings2.iter().any(|f| f.cwe_id == 467 && f.source == SastSource::Ast),
        "CWE-467: sizeof(*p) should not fire; got {:?}", findings2);
}

#[test]
fn test_cwe_526_env_exposure() {
    let tp = b"#include <stdlib.h>\nvoid f() { char *p = getenv(\"PATH\"); }\n";
    let (_t, dirs) = setup_one_file("cwe526.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 526 && f.source == SastSource::Ast),
        "CWE-526: expected getenv() to fire; got {:?}", findings);
}

#[test]
fn test_cwe_535_shell_error_stderr() {
    // TP: fprintf(stderr, ...) fires CWE-535
    let tp = b"#include <stdio.h>\nvoid f(char *pw) { fprintf(stderr, \"%s\", pw); }\n";
    let (_t, dirs) = setup_one_file("cwe535_tp.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 535 && f.source == SastSource::Ast),
        "CWE-535: expected fprintf(stderr) to fire; got {:?}", findings);

    // FP guard: fprintf(stdout, ...) does NOT fire
    let fp = b"#include <stdio.h>\nvoid f(int x) { fprintf(stdout, \"%d\", x); }\n";
    let (_t2, dirs2) = setup_one_file("cwe535_fp.c", fp);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(!findings2.iter().any(|f| f.cwe_id == 535 && f.source == SastSource::Ast),
        "CWE-535: fprintf(stdout) should not fire; got {:?}", findings2);
}

#[test]
fn test_cwe_676_dangerous_function() {
    // TP: alloca fires CWE-676 (different CWE ID from CWE-121 — both valid per design)
    let tp = b"#include <alloca.h>\nvoid f(int n) { void *p = alloca(n); }\n";
    let (_t, dirs) = setup_one_file("cwe676.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 676 && f.source == SastSource::Ast),
        "CWE-676: expected alloca to fire; got {:?}", findings);
}

#[test]
fn test_cwe_680_integer_overflow_alloc() {
    let tp = b"#include <stdlib.h>\nvoid f(int data) { int *buf = malloc(data * sizeof(int)); }\n";
    let (_t, dirs) = setup_one_file("cwe680.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 680 && f.source == SastSource::Ast),
        "CWE-680: expected malloc(data * sizeof(int)) to fire; got {:?}", findings);
}

#[test]
fn test_cwe_780_rsa_no_oaep() {
    // TP: RSA_public_encrypt with PKCS1_PADDING fires CWE-780
    let tp = b"void f(int l, unsigned char *f, unsigned char *t, RSA *r) { RSA_public_encrypt(l, f, t, r, RSA_PKCS1_PADDING); }\n";
    let (_t, dirs) = setup_one_file("cwe780_tp.c", tp);
    let findings = run_ast_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 780 && f.source == SastSource::Ast),
        "CWE-780: expected RSA_PKCS1_PADDING to fire; got {:?}", findings);

    // FP guard: OAEP padding does NOT fire
    let fp = b"void f(int l, unsigned char *f, unsigned char *t, RSA *r) { RSA_public_encrypt(l, f, t, r, RSA_PKCS1_OAEP_PADDING); }\n";
    let (_t2, dirs2) = setup_one_file("cwe780_fp.c", fp);
    let findings2 = run_ast_scanner(&dirs2);
    assert!(!findings2.iter().any(|f| f.cwe_id == 780 && f.source == SastSource::Ast),
        "CWE-780: RSA_PKCS1_OAEP_PADDING should not fire; got {:?}", findings2);
}
```

---

## Shared Patterns

### Fresh cursor per recursion level (all recursive AST walk functions)

**Source:** `src/vulnerability/ast_scanner.rs` lines 276–293 (visit_node recursion tail)
**Apply to:** `visit_binary_exprs()`, `collect_function_scope_pointer_declarators()` (new helper)

```rust
// Recurse into children — fresh cursor per call level (Pitfall 1)
let mut cursor = node.walk();
if cursor.goto_first_child() {
    loop {
        visit_node(cursor.node(), src, path, component_name, component_ecosystem,
                   file_scope_arrays, findings);
        if !cursor.goto_next_sibling() { break; }
    }
}
```

### SastFinding construction

**Source:** `src/vulnerability/ast_scanner.rs` lines 261–269
**Apply to:** `visit_binary_exprs()` CWE-369 emit, `SizeofPointer` arm emit

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

### Pointer declarator collection helper (new, mirrors fixed-array pattern)

**Model on:** `collect_function_scope_fixed_arrays()` / `collect_array_declarators()` (lines 343–401)
**Apply to:** `ArgCheck::SizeofPointer` arm — need `collect_function_scope_pointer_declarators()`

```rust
// Pattern from collect_arrays_in_subtree (lines 349-367) — mirror for pointer_declarator:
fn collect_function_scope_pointer_declarators(fn_node: Node, src: &[u8]) -> HashSet<String> {
    let mut result = HashSet::new();
    collect_pointer_decls_in_subtree(fn_node, src, &mut result);
    result
}

fn collect_pointer_decls_in_subtree(node: Node, src: &[u8], out: &mut HashSet<String>) {
    if node.kind() == "declaration" {
        // Walk children for pointer_declarator nodes (mirrors collect_array_declarators)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                collect_pointer_declarator_rec(child, src, out);
            }
        }
    }
    let mut cursor = node.walk(); // fresh cursor per level (Pitfall 1)
    if cursor.goto_first_child() {
        loop {
            collect_pointer_decls_in_subtree(cursor.node(), src, out);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}

fn collect_pointer_declarator_rec(node: Node, src: &[u8], out: &mut HashSet<String>) {
    if node.kind() == "pointer_declarator" {
        // First named child of pointer_declarator is the identifier
        if let Some(ident) = node.child_by_field_name("declarator") {
            if ident.kind() == "identifier" {
                if let Ok(name) = ident.utf8_text(src) {
                    out.insert(name.to_string());
                }
            }
        }
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() != "pointer_declarator" {
                collect_pointer_declarator_rec(child, src, out);
            }
        }
    }
}
```

**Caution:** Verify the field name for the identifier inside `pointer_declarator` in tree-sitter-c grammar. It may be `"declarator"` or accessed via `child(1)`. Check against a test fixture.

### ArgAtIndex arm implementation (Phase 20 addition — read before writing CWE-328/535/780 rules)

**Source:** `src/vulnerability/ast_scanner.rs` — Phase 20 execution result (not yet visible in current file)
**Critical check:** Whether `tokens.iter().all(...)` (ALL-OF) or `tokens.iter().any(...)` (ANY-OF) is used in the arm implementation. This determines whether CWE-328 needs one rule entry or three.

From Phase 20 plan (`.planning/phases/20-argument-value-ast-migration/20-01-PLAN.md` line 34):
```
via: "tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))"
```
This indicates ALL-OF semantics. CWE-328 with `&["CALG_MD2", "CALG_MD5", "CALG_SHA1"]` would NEVER fire (arg can't contain all three simultaneously). **Split into three separate rule entries.**

### Test assertion pattern (TP + FP guard)

**Source:** `tests/vulnerability_tests/ast_scanner_tests.rs` lines 95–117
**Apply to:** all 12 new test functions

```rust
// TP assertion:
assert!(
    findings.iter().any(|f| f.cwe_id == NNN && f.source == SastSource::Ast),
    "Expected CWE-NNN with SastSource::Ast; got {:?}", findings
);
// FP guard assertion:
assert!(
    !findings2.iter().any(|f| f.cwe_id == NNN && f.source == SastSource::Ast),
    "CWE-NNN fired on safe code (false positive): {:?}", findings2
);
```

---

## No Analog Found

No files in this phase lack analogs. All new code is added to existing files following established patterns.

| File | Role | Data Flow | Note |
|------|------|-----------|------|
| `benchmark/juliet/ANALYSIS.md` | documentation | n/a | Updated post-implementation; no code pattern needed |

---

## Critical Implementation Notes for Planner

1. **Phase 20 completion dependency:** `src/vulnerability/ast_scanner.rs` still has `ContainsTokens` on disk as of 2026-05-12. Phase 20 must execute first. The planner must read the actual file state before writing Phase 21 tasks.

2. **ArgAtIndex semantics (ALL-OF confirmed):** Phase 20 plan evidence shows `tokens.iter().all(...)`. CWE-328 must use three separate rule entries (one per CALG_ constant). CWE-535 uses a single-token slice `&["stderr"]` so semantics do not matter. CWE-780 uses a two-token slice `&["RSA_PKCS1_PADDING", "RSA_NO_PADDING"]` — this would require both tokens simultaneously; split into two entries.

3. **CWE-369 Juliet corpus gap:** The AST `apply_division_rules()` fires on literal `/0`; Juliet uses variable divisors (`100/data`). Expect 0 Juliet TPs for CWE-369. The inline unit test (`test_cwe_369_division_literal_zero`) is the primary TP validation. Note in ANALYSIS.md update.

4. **CWE-676 Juliet corpus gap:** Juliet CWE-676 uses `cin >>` (not a call_expression). Expect 0 Juliet TPs. The inline unit test is the primary validation. Note in ANALYSIS.md.

5. **number_literal vs integer_literal:** Verify the tree-sitter-c node kind for `0` in `x / 0`. The `apply_division_rules()` implementation should check both (`rhs.kind() == "number_literal" || rhs.kind() == "integer_literal"`) until verified.

6. **CWE-121 and CWE-676 both fire on alloca:** This produces two findings at the same (file, line) with different CWE IDs — correct per design. The dedup pipeline on `(file, line, cwe_id)` preserves both.

7. **CWE-535 ArgAtIndex(0) on fprintf:** The `stderr` identifier is the first argument. The ArgAtIndex arm checks if `token_present_with_boundary(arg_text, "stderr")` on `args[0]`. Since `stderr` is typically a macro expanding to a `FILE*` expression, verify the arg subtree text contains the literal string `"stderr"` in Juliet's test files before assuming this works.

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `tests/vulnerability_tests/`, `tests/fixtures/c/`
**Files scanned:** `src/vulnerability/ast_scanner.rs` (full, 497 lines), `tests/vulnerability_tests/ast_scanner_tests.rs` (full, 125 lines), `tests/vulnerability_tests/cwe_scanner_tests.rs` (partial, 80 lines), `tests/fixtures/c/dangerous_calls.c` (full), `.planning/phases/20-argument-value-ast-migration/20-01-PLAN.md` (partial, 100 lines)
**Pattern extraction date:** 2026-05-12
