# Phase 18: ast-scanner-core-and-benchmark — Pattern Map

**Mapped:** 2026-05-11
**Files analyzed:** 6 new/modified files
**Analogs found:** 6 / 6

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/ast_scanner.rs` | service | transform (AST parse → findings) | `src/vulnerability/cwe_scanner.rs` | exact (same role, same output type) |
| `src/vulnerability/cwe_scanner.rs` | service | CRUD (add `SastSource::Ast` variant) | self | self-modification |
| `src/vulnerability/mod.rs` | config | — (re-exports, cfg gates) | self | self-modification |
| `src/main.rs` | controller | request-response (scanner dispatch) | self | self-modification |
| `Cargo.toml` | config | — (feature flag merge) | self | self-modification |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | test | CRUD (unit test per rule) | `tests/vulnerability_tests/cwe_scanner_tests.rs` | exact |
| `tests/benchmark.rs` | test | batch (run 3 scanners, write BENCHMARK.md) | `tests/vulnerability_tests/cppcheck_scanner_tests.rs` | role-match |

---

## Pattern Assignments

### `src/vulnerability/ast_scanner.rs` (service, transform)

**Analog:** `src/vulnerability/cwe_scanner.rs`

**Feature gate (file top) — change from PoC:**

The existing PoC has:
```rust
// src/vulnerability/ast_scanner.rs line 12 (current — to be replaced)
#![cfg(feature = "ast-scanner")]
```

Change to match every other internal scanner file:
```rust
// src/vulnerability/cwe_scanner.rs line 13 (exact pattern to copy)
#![cfg(feature = "internal")]
```

**Imports pattern** (`src/vulnerability/cwe_scanner.rs` lines 15-24):
```rust
use crate::util::warn_on_walkdir_err;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
```
For `ast_scanner.rs`, also add:
```rust
use tree_sitter::{Node, Parser, TreeCursor};
```
(tree-sitter-c is imported via `tree_sitter_c::LANGUAGE` — copy from existing PoC line 14.)

**Data-driven rule table pattern** (`src/vulnerability/cwe_scanner.rs` lines 50-98):

The `CweRule` struct + `CWE_RULES` static slice is the direct structural model for `AstCweRule` + `AST_CWE_RULES`. Copy the `struct` + `static` pattern exactly:

```rust
// Copy structure from cwe_scanner.rs lines 49-66, adapt fields:
struct AstCweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    arg_check: ArgCheck,
}

// ArgCheck enum — D-06: variants only, no closures, no heap allocation.
enum ArgCheck {
    FixedSizeBuffer,                         // dest arg (index 0) is array_declarator
    NotStringLiteralAtIndex(u8),             // format arg at 0-based index is NOT string_literal
    ContainsTokens(&'static [&'static str]), // ANY named arg contains all tokens (word-boundary)
    AnyCall,                                 // name match only — no argument inspection
}

static AST_CWE_RULES: &[AstCweRule] = &[
    // ... 11 tractable CWEs per D-07
];
```

**`SastFinding` construction pattern** (`src/vulnerability/cwe_scanner.rs` lines 371-379):
```rust
// Copy this exact shape — D-04: emit SastFinding directly, not AstFinding
findings.push(SastFinding {
    cwe_id: rule.cwe_id,
    component_name: component_name.to_string(),
    component_ecosystem: component_ecosystem.to_string(),
    file_path: path.to_string_lossy().into_owned(),
    line: line_num,
    source: SastSource::Ast,  // NEW variant — replaces SastSource::Lexical here
});
```

**WalkDir + file extension filter pattern** (`src/vulnerability/cwe_scanner.rs` lines 417-434):
```rust
// Copy this loop structure exactly for run_ast_scanner():
pub fn run_ast_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
) -> Vec<SastFinding> {
    let mut all_findings = Vec::new();
    // Create Parser ONCE outside the loop — D-06 anti-pattern: do NOT reinitialize per file
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into())
        .expect("tree-sitter-c grammar load failed");

    for ((name, ecosystem), dir) in component_dirs.iter() {
        if !dir.exists() {
            continue;
        }
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(warn_on_walkdir_err)
        {
            if !entry.file_type().is_file() { continue; }
            let p = entry.path();
            if !is_c_cpp_source(p) { continue; }
            all_findings.extend(scan_file_ast_or_lexical(p, name, ecosystem, &mut parser));
        }
    }
    all_findings
}
```

**`is_c_cpp_source` helper** (`src/vulnerability/cwe_scanner.rs` lines 408-413):
```rust
// Copy verbatim — identical extensions needed
fn is_c_cpp_source(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("c") | Some("h") | Some("cpp") | Some("hpp") | Some("cc") => true,
        _ => false,
    }
}
```

**Per-file AST parse with lexical fallback** (from PoC `ast_scanner.rs` lines 33-36 + RESEARCH.md Pattern 2):
```rust
// Adapt from existing PoC parse pattern (lines 33-36) + add has_error() guard (Pitfall 4)
fn scan_file_ast_or_lexical(
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    parser: &mut Parser,
) -> Vec<SastFinding> {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let tree = match parser.parse(&code, None) {
        Some(t) => t,
        None => {
            eprintln!("Warning: tree-sitter failed to parse {:?} — falling back to lexical scan", path);
            return scan_file(path, component_name, component_ecosystem);  // lexical fallback
        }
    };
    // Pitfall 4: has_error() == true means partial parse — fall back for safety
    if tree.root_node().has_error() {
        eprintln!("Warning: tree-sitter parse had errors in {:?} — falling back to lexical scan", path);
        return scan_file(path, component_name, component_ecosystem);
    }
    apply_ast_rules(tree.root_node(), code.as_bytes(), path, component_name, component_ecosystem)
}
```

**call_expression traversal pattern** (from PoC `ast_scanner.rs` lines 112-130, improved per RESEARCH.md Pattern 3):
```rust
// Use field-based access (Pattern 3) instead of positional child(0)/child(1) (Pitfall 5)
if node.kind() == "call_expression" {
    if let Some(func_node) = node.child_by_field_name("function") {
        if let Ok(func_name) = func_node.utf8_text(src) {
            // match against rule.functions
        }
    }
    if let Some(arg_list) = node.child_by_field_name("arguments") {
        let mut cursor = arg_list.walk();
        let args: Vec<Node> = arg_list.named_children(&mut cursor).collect();
        // args[0] = first arg, args[1] = second, etc. (no punctuation nodes)
    }
}
```

**Recursive AST walk cursor pattern** (from PoC `ast_scanner.rs` lines 133-141 — replicate, not positional):
```rust
// Copy this cursor walk pattern exactly — fresh cursor per call level (Pitfall 1)
let mut cursor = node.walk();
if cursor.goto_first_child() {
    loop {
        visit_node(cursor.node(), src, /* ... */);
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}
```

**`collect_fixed_arrays` scope fix** (PoC `ast_scanner.rs` lines 62-101, Pitfall 3 from RESEARCH.md):

The PoC collects from the whole file. For production, scope collection to the enclosing `function_definition` node. The collection logic itself (checking `"declaration"` → `"array_declarator"` → `"identifier"`) is correct and should be copied; only the scope anchor changes.

**Error handling pattern** (`src/vulnerability/cwe_scanner.rs` lines 338-342):
```rust
// File open errors — copy this exact graceful pattern
let file = match std::fs::File::open(path) {
    Ok(f) => f,
    Err(_) => return Vec::new(),
};
```

---

### `src/vulnerability/cwe_scanner.rs` (service, CRUD — add `SastSource::Ast`)

**Modification:** Add `Ast` variant to `SastSource` enum.

**Current enum** (`src/vulnerability/cwe_scanner.rs` lines 29-33):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SastSource {
    Lexical,
    Cppcheck,
    Both,
}
```

**After change:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SastSource {
    Lexical,
    Cppcheck,
    Both,
    Ast,  // Phase 18: AST scanner provenance
}
```

**Dedup impact** (`src/vulnerability/cwe_scanner.rs` lines 754-779): `deduplicate_sast_findings()` takes `lexical: Vec<SastFinding>` as first arg. Phase 18 passes `ast_findings` (which includes both `SastSource::Ast` and `SastSource::Lexical` fallback findings) as the first argument. The dedup key is `(normalized_file, line, cwe_id)` — no `source` comparison in the key, so adding `Ast` does not break dedup logic. However, grep all `match.*SastSource` patterns before adding the variant — Rust's exhaustiveness check will catch missing arms at compile time.

**Grep command to check all match sites:**
```
Grep("SastSource::", path: "src/", output_mode: "content")
```
Also check `src/main.rs` lines 267-268 which filter `SastSource::Cppcheck || Both` — that filter is correct as-is (no change needed for the Ast variant).

---

### `src/vulnerability/mod.rs` (config — re-exports and cfg gates)

**Current** (`src/vulnerability/mod.rs` lines 1-15):
```rust
pub mod ast_scanner;  // line 1: ungated — must change
// ...
#[cfg(feature = "internal")]
pub mod cwe_scanner;
// ...
#[cfg(feature = "internal")]
pub use cwe_scanner::{deduplicate_sast_findings, ..., SastSource, ...};
```

**Required changes:**
1. Change `pub mod ast_scanner;` (line 1, ungated) to `#[cfg(feature = "internal")] pub mod ast_scanner;`
2. Add `run_ast_scanner` to the `#[cfg(feature = "internal")] pub use cwe_scanner::{...}` line — or add a separate re-export line for `ast_scanner::run_ast_scanner`.

**Pattern to copy** (`src/vulnerability/mod.rs` lines 7-15):
```rust
// Copy this cfg-gated pub use pattern for the new ast_scanner export:
#[cfg(feature = "internal")]
pub mod ast_scanner;

#[cfg(feature = "internal")]
pub use ast_scanner::run_ast_scanner;
```

---

### `src/main.rs` (controller — scanner dispatch change)

**Current call site** (`src/main.rs` lines 247-277):

```rust
// Phase 11: current dispatch (to be replaced)
let lexical_findings = crate::vulnerability::run_lexical_scanner(&component_dirs);
let cppcheck_bin = ...;
let (cppcheck_findings, cppcheck_scanned_dirs) =
    crate::vulnerability::run_cppcheck_scanner(&component_dirs, cppcheck_bin);
sast_findings =
    crate::vulnerability::deduplicate_sast_findings(lexical_findings, cppcheck_findings);
```

**Phase 18 replacement pattern** (same cfg block, same variables):
```rust
// Phase 18: AST scanner as primary; lexical fallback is internal to run_ast_scanner()
let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs);
let cppcheck_bin = ...;
let (cppcheck_findings, cppcheck_scanned_dirs) =
    crate::vulnerability::run_cppcheck_scanner(&component_dirs, cppcheck_bin);
// ast_findings already contains both Ast-sourced and Lexical-fallback findings
sast_findings =
    crate::vulnerability::deduplicate_sast_findings(ast_findings, cppcheck_findings);
```

The `suppress_lexical_false_positives` call and `cppcheck_confirmed` set (lines 263-277) are **unchanged** — they operate on the post-dedup slice and do not care about `SastSource::Ast`.

---

### `Cargo.toml` (config — feature flag merge)

**Current** (`Cargo.toml` lines 16-17):
```toml
internal = ["dep:reqwest"]
ast-scanner = ["dep:tree-sitter", "dep:tree-sitter-c"]
```

**Phase 18 change** (D-01):
```toml
internal = ["dep:reqwest", "dep:tree-sitter", "dep:tree-sitter-c"]
# (remove the ast-scanner line entirely)
```

tree-sitter and tree-sitter-c are already declared as optional deps in `[dependencies]` (lines 40-41) — no new dep entries needed.

---

### `tests/vulnerability_tests/ast_scanner_tests.rs` (test — unit)

**Analog:** `tests/vulnerability_tests/cwe_scanner_tests.rs`

**File-top gate** (`tests/vulnerability_tests/cppcheck_scanner_tests.rs` line 1):
```rust
#![cfg(feature = "internal")]
```

**Imports pattern** (`tests/vulnerability_tests/cwe_scanner_tests.rs` lines 1-4):
```rust
use radeis_sc2sbom::vulnerability::{run_ast_scanner, SastFinding, SastSource};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
```

**Test helper pattern** (`tests/vulnerability_tests/cwe_scanner_tests.rs` lines 6-12):
```rust
// Copy this helper verbatim — reuse for ast_scanner_tests
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

**Test pattern for AST-03 (emits SastFinding, not AstFinding):**
```rust
#[test]
fn test_ast_emits_sast_finding() {
    let (_t, dirs) = setup_one_file("a.c", b"void f() { char buf[64]; strcpy(buf, \"x\"); }\n");
    let findings: Vec<SastFinding> = run_ast_scanner(&dirs);  // return type assertion
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.source == SastSource::Ast));
}
```

**Test pattern for AST-04 (parse failure falls back to lexical):**
```rust
#[test]
fn test_parse_failure_fallback() {
    // Malformed C that tree-sitter cannot produce a clean tree for
    let (_t, dirs) = setup_one_file("bad.c", b"!@#$%^&*() not valid C\n strcpy(x, y);\n");
    let findings = run_ast_scanner(&dirs);
    // Should not panic; lexical fallback runs and may or may not find CWE-120
    // (the goal is graceful exit 0 and no panic)
    let _ = findings;
}
```

**Test pattern for CWE coverage (AST-02) — copy from cwe_scanner_tests.rs lines 26-39:**
```rust
#[test]
fn test_ast_all_cwes() {
    // Use a fixture file that has one call site per tractable CWE
    // Use same setup_one_file pattern + check findings contain expected CWE IDs
    let (_t, dirs) = setup_one_file("all_cwes.c", FIXTURE_BYTES);
    let findings = run_ast_scanner(&dirs);
    let ids: Vec<u32> = findings.iter().map(|f| f.cwe_id).collect();
    for expected_cwe in [78u32, 119, 120, 122, 125, 134, 190, 242, 327, 369, 377, 732] {
        assert!(ids.contains(&expected_cwe), "missing CWE-{}", expected_cwe);
    }
}
```

---

### `tests/benchmark.rs` (test — batch, graceful skip)

**Analog:** `tests/vulnerability_tests/cppcheck_scanner_tests.rs` (for test structure); RESEARCH.md Pattern 5 for graceful-skip logic.

**File-top gate** (copy from `tests/vulnerability_tests/cppcheck_scanner_tests.rs` line 1):
```rust
#![cfg(feature = "internal")]
```

**Graceful-skip pattern** (RESEARCH.md Pattern 5, lines 298-317):
```rust
fn fixture_path(env_var: &str, default: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var(env_var).unwrap_or_else(|_| default.to_string());
    let p = std::path::PathBuf::from(&path);
    if p.exists() { Some(p) } else { None }
}

#[test]
fn benchmark_autosar_fixture() {
    let fixture = match fixture_path("AUTOSAR_FIXTURE_PATH", "../AUTOSAR_SampleProject_S32K144") {
        Some(p) => p,
        None => {
            eprintln!("SKIP benchmark_autosar_fixture: fixture not present — set AUTOSAR_FIXTURE_PATH");
            return;  // D-11: graceful skip, not panic!
        }
    };
    // ... run scanners, collect TPs/FPs, write BENCHMARK.md
}
```

**BENCHMARK.md write pattern** — use `std::fs::write` with a path relative to `env!("CARGO_MANIFEST_DIR")` so the file lands at `docs/BENCHMARK.md` relative to the repo root:
```rust
let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/BENCHMARK.md");
std::fs::write(&out_path, markdown_content).expect("failed to write BENCHMARK.md");
```

---

### Registration of `tests/vulnerability_tests/ast_scanner_tests.rs`

**Analog:** `tests/vulnerability_tests/mod.rs` lines 3-4 (existing pattern):
```rust
// Copy this exact pattern — add after existing #[cfg(feature = "internal")] entries:
#[cfg(feature = "internal")]
mod ast_scanner_tests;
```

**Analog for `tests/benchmark.rs` in `tests/all_tests.rs`** — `tests/benchmark.rs` is a top-level integration test file; it does NOT need an entry in `all_tests.rs`. Rust discovers it automatically as a separate test binary. No `all_tests.rs` change needed.

---

## Shared Patterns

### Feature Gate
**Source:** `src/vulnerability/cwe_scanner.rs` line 13
**Apply to:** `src/vulnerability/ast_scanner.rs` (file top), `tests/vulnerability_tests/ast_scanner_tests.rs` (file top), `tests/benchmark.rs` (file top)
```rust
#![cfg(feature = "internal")]
```

### Graceful File-Access Errors
**Source:** `src/vulnerability/cwe_scanner.rs` lines 338-342
**Apply to:** `ast_scanner.rs` `scan_file_ast_or_lexical()` for `read_to_string` errors
```rust
let code = match std::fs::read_to_string(path) {
    Ok(c) => c,
    Err(_) => return Vec::new(),
};
```

### `warn_on_walkdir_err` Utility
**Source:** `src/vulnerability/cwe_scanner.rs` line 16 + lines 425-426
**Apply to:** `run_ast_scanner()` WalkDir loop
```rust
use crate::util::warn_on_walkdir_err;
// ...
.filter_map(warn_on_walkdir_err)
```

### `eprintln!` for Warnings (not `log::warn!` or `panic!`)
**Source:** `src/vulnerability/cwe_scanner.rs` lines 348-350
**Apply to:** All fallback/skip paths in `ast_scanner.rs` and `tests/benchmark.rs`
```rust
eprintln!("Warning: ...");
```

### Test Helper `setup_one_file`
**Source:** `tests/vulnerability_tests/cwe_scanner_tests.rs` lines 6-12
**Apply to:** `tests/vulnerability_tests/ast_scanner_tests.rs` — copy verbatim

---

## No Analog Found

No files in this phase lack an analog. All files have strong existing matches.

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `src/main.rs`, `Cargo.toml`, `tests/vulnerability_tests/`, `tests/`
**Files scanned:** 8 source files read directly
**Pattern extraction date:** 2026-05-11
