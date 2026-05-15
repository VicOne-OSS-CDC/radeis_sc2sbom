# Phase 25: experiment-scan-mode — Pattern Map

**Mapped:** 2026-05-13
**Files analyzed:** 3 (1 struct change, 1 CLI change, 1 call-site change)
**Analogs found:** 3 / 3

---

## Specific Answers

### Q1: Exact current signature of `run_ast_scanner()`

`src/vulnerability/ast_scanner.rs` lines 256–258:

```rust
pub fn run_ast_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
) -> Vec<SastFinding> {
```

One parameter. No boolean flags at all. Called from `src/main.rs` line 240 as:

```rust
let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs);
```

The public re-export is at `src/vulnerability/mod.rs` line 19:

```rust
pub use ast_scanner::run_ast_scanner;
```

### Q2: Fields `AstCweRule` struct currently has

`src/vulnerability/ast_scanner.rs` lines 67–71:

```rust
struct AstCweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    arg_check: ArgCheck,
}
```

Three fields. No `experimental` field yet. The struct derives `Debug` only (no `Clone`, `Copy`, or `PartialEq`). All 39+ rule entries are struct-literal initializers in the `AST_CWE_RULES` static slice — they do NOT use named-field update syntax, so adding a new field requires updating every literal.

### Q3: How boolean CLI flags are currently defined and passed

**Pattern A — `ArgAction::SetTrue` (default false, set to true when flag present):**

`src/cli.rs` lines 148–149:
```rust
#[arg(long, action = ArgAction::SetTrue)]
pub clear_cache: bool,
```

`src/cli.rs` lines 163–164:
```rust
#[arg(long, action = ArgAction::SetTrue)]
pub compact: bool,
```

**Pattern B — `ArgAction::Set` with explicit `default_value_t` (value set explicitly):**

`src/cli.rs` lines 122–124:
```rust
#[cfg(feature = "internal")]
#[arg(long, action = ArgAction::Set, default_value_t = false)]
pub check_vulnerabilities: bool,
```

**`--experiment-scan` should use Pattern A** (`ArgAction::SetTrue`, default false — the flag enables experimental rules when present). Decorate with `#[cfg(feature = "internal")]` since the AST scanner is internal-only.

**How a bool arg is passed to a scanner function** (existing pattern from `src/main.rs` lines 64–68):

```rust
args.scan_so_files,           // v1.0.5
```

Direct field access from `args`, passed positionally. The new flag would be passed to `run_ast_scanner` the same way: `crate::vulnerability::run_ast_scanner(&component_dirs, args.experiment_scan)`.

### Q4: Where the experimental filter goes

**Primary filter point — `visit_node()` inner loop** (`src/vulnerability/ast_scanner.rs` lines 1891–1893):

```rust
for rule in AST_CWE_RULES {
    if !rule.functions.contains(&func_name) {
        continue;
    }
    // ...
}
```

Add one guard after the existing `functions` check:
```rust
if !experiment_scan && rule.experimental {
    continue;
}
```

**Secondary filter point — `apply_ast_rules()` call-site guards** (lines 364–382). The structural `check_*` functions each handle exactly one CWE. Experimental structural checks (CWE-478, 480, 483, 535, 562, 570, 571) need their call lines wrapped:

```rust
if experiment_scan {
    check_switch_structure(...); // CWE-478/484 — only 478 is experimental
}
```

Note: `check_switch_structure` handles both CWE-478 (experimental) and CWE-484 (not experimental), so the filter must go *inside* that function on a per-CWE basis rather than wrapping the whole call — OR the function is split. The simpler approach: pass `experiment_scan` into each check function and guard inside.

**`run_ast_scanner()` becomes the entry point for the flag** — it passes the bool down through `scan_file_ast_or_lexical()` → `apply_ast_rules()` → `visit_node()` and each `check_*` call.

### Q5: Which `AST_CWE_RULES` entries correspond to the 17 experimental CWEs

Target list: CWE-120, 122, 126, 190, 338, 426, 467, 478, 480, 483, 535, 562, 570, 571, 676, 680, 780.

**Table-driven rules in `AST_CWE_RULES` (add `experimental: true`):**

| CWE | Line(s) in ast_scanner.rs | Rule entry |
|-----|---------------------------|------------|
| 120 | 79 | `AstCweRule { cwe_id: 120, functions: &["strcpy", "strcat", "gets"], arg_check: ArgCheck::FixedSizeBuffer }` |
| 122 | 80 | `AstCweRule { cwe_id: 122, functions: &["memcpy", "memmove", "sprintf"], arg_check: ArgCheck::FixedSizeBuffer }` |
| 126 | 131–140 | Two entries: `strcat` (FixedSizeBuffer) + `strncat` (FixedSizeBufferWithoutSizeArg(2)) |
| 190 | 84 | `AstCweRule { cwe_id: 190, functions: &["malloc", "calloc", "realloc"], arg_check: ArgCheck::AnyCall }` |
| 338 | 164–168 | `AstCweRule { cwe_id: 338, functions: &["drand48", "lrand48", "random", "mrand48"], arg_check: ArgCheck::AnyCall }` |
| 426 | 173–177 | `AstCweRule { cwe_id: 426, functions: &["dlopen", "LoadLibraryExA", "LoadLibraryExW"], arg_check: ArgCheck::AnyCall }` |
| 467 | 107 | `AstCweRule { cwe_id: 467, functions: &["malloc", "calloc", ...], arg_check: ArgCheck::SizeofPointer }` |
| 676 | 195–199 | `AstCweRule { cwe_id: 676, functions: &["strtok"], arg_check: ArgCheck::AnyCall }` |
| 680 | 204–208 | `AstCweRule { cwe_id: 680, functions: &["malloc", "realloc", "calloc"], arg_check: ArgCheck::SizeArgIsMultiplication(0) }` |
| 780 | 215–219 | `AstCweRule { cwe_id: 780, functions: &["CryptEncrypt"], arg_check: ArgCheck::ArgAtIndex(3, &["0"]) }` |

**Structural helpers (NOT in `AST_CWE_RULES`, handled by `check_*` functions):**

| CWE | Handler function | Line |
|-----|-----------------|------|
| 478 | `check_switch_structure()` (mixed with CWE-484) | 392 |
| 480 | `check_comparison_at_statement()` + `check_assignment_in_condition()` | 600, 645 |
| 483 | `check_block_delimitation()` | 485 |
| 535 | `check_stderr_format_string()` | 531 |
| 562 | `check_return_stack_address()` | 869 |
| 570 | `check_constant_condition()` (fires on `if (0)` / `if (false)`) | 993 |
| 571 | `check_constant_condition()` (fires on `if (1)` / `if (true)`) | 993 |

Note: CWE-570 and CWE-571 are handled by the same `check_constant_condition()` function. The experimental filter must be applied per-CWE inside that function (not by skipping the whole call).

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/ast_scanner.rs` — `AstCweRule` struct | model | — | existing struct in same file | exact (self-referential) |
| `src/vulnerability/ast_scanner.rs` — `run_ast_scanner()` signature | service | request-response | `run_lexical_scanner()` in `cwe_scanner.rs` | role-match |
| `src/cli.rs` — `--experiment-scan` flag | config | — | `clear_cache` / `check_vulnerabilities` flags | exact |
| `tests/vulnerability_tests/ast_scanner_tests.rs` — new tests | test | — | existing tests in same file (lines 20–117) | exact |
| `tests/autosar_ast_regression.rs` — baseline guard | test | — | existing test in same file (line 10) | exact |

---

## Pattern Assignments

### `AstCweRule` struct — add `experimental: bool` field

**Analog:** The `AstCweRule` struct itself (lines 66–71). This is a self-contained change.

**Current struct** (lines 66–71):
```rust
#[derive(Debug)]
struct AstCweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    arg_check: ArgCheck,
}
```

**Target struct:**
```rust
#[derive(Debug)]
struct AstCweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    arg_check: ArgCheck,
    experimental: bool,
}
```

**All existing rule literals** must add `experimental: false` (22 non-experimental entries) or `experimental: true` (17 experimental entries). The table uses both inline single-line style and multi-line struct style — both need the new field. Example of each style updated:

Inline style (line 77):
```rust
AstCweRule { cwe_id: 78, functions: &["system", ...], arg_check: ArgCheck::AnyCall, experimental: false },
```

Multi-line style (lines 121–125):
```rust
AstCweRule {
    cwe_id: 121,
    functions: &["alloca"],
    arg_check: ArgCheck::AnyCall,
    experimental: false,
},
```

---

### `run_ast_scanner()` — add `experiment_scan: bool` parameter

**Analog:** `run_lexical_scanner()` in `src/vulnerability/cwe_scanner.rs` (check its signature for the pattern of passing a simple bool).

**Current call chain:**
```
run_ast_scanner(&dirs)
  └─ scan_file_ast_or_lexical(path, name, ecosystem, &mut parser)
       ├─ apply_ast_rules(root, src, path, name, ecosystem)
       │    ├─ visit_node(..., &mut findings)
       │    └─ check_*(root, src, path, name, ecosystem, &mut findings)
       ├─ apply_division_rules(...)
       ├─ apply_signal_handler_rules(...)
       ├─ apply_paired_lock_rules(...)
       └─ apply_delete_rules(...)
```

**Target call chain** — thread `experiment_scan: bool` through every level:
```
run_ast_scanner(&dirs, experiment_scan)
  └─ scan_file_ast_or_lexical(..., experiment_scan)
       └─ apply_ast_rules(..., experiment_scan)
            ├─ visit_node(..., experiment_scan, &mut findings)   // filters AST_CWE_RULES
            └─ check_*(... experiment_scan ...)                  // filters structural checks
```

**Filter insertion in `visit_node()`** (after line 1892):
```rust
for rule in AST_CWE_RULES {
    if !rule.functions.contains(&func_name) {
        continue;
    }
    if rule.experimental && !experiment_scan {   // <-- NEW
        continue;
    }
    // existing arg_check match ...
}
```

**Filter insertion for structural checks** — wrap or guard each experimental `check_*` call at lines 364–382 of `apply_ast_rules()`. Since CWE-478/484 and CWE-570/571 share functions, the guard must be passed into those functions as a parameter, not wrap the call.

---

### `src/cli.rs` — `--experiment-scan` flag

**Analog:** `clear_cache` flag (lines 147–149) for `ArgAction::SetTrue` pattern; `check_vulnerabilities` (lines 122–124) for `#[cfg(feature = "internal")]` decoration.

**Pattern to copy** (combine both analogs):
```rust
/// Enable experimental CWE rules (higher false-positive rate) (v1.0.18)
#[cfg(feature = "internal")]
#[arg(long, action = ArgAction::SetTrue)]
pub experiment_scan: bool,
```

**Call-site in `src/main.rs`** (after line 240):
```rust
let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs, args.experiment_scan);
```

---

### Tests — new unit test pattern

**Analog:** `test_ast_emits_sast_finding` (lines 20–30) and `test_ast_safe_strcpy_no_finding` (lines 95–104) in `tests/vulnerability_tests/ast_scanner_tests.rs`.

**Setup helper** (lines 11–17) — reuse unchanged:
```rust
fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

**Test pattern for "experimental rule suppressed when flag off":**
```rust
#[test]
fn test_experimental_rule_suppressed_without_flag() {
    let (_t, dirs) = setup_one_file("a.c", b"void f() { char buf[64]; strcpy(buf, \"x\"); }\n");
    let findings = run_ast_scanner(&dirs, false);  // experiment_scan = false
    assert!(
        !findings.iter().any(|f| f.cwe_id == 120 && f.source == SastSource::Ast),
        "CWE-120 is experimental and must not fire without --experiment-scan; got {:?}",
        findings
    );
}
```

**Test pattern for "experimental rule fires when flag on":**
```rust
#[test]
fn test_experimental_rule_fires_with_flag() {
    let (_t, dirs) = setup_one_file("a.c", b"void f() { char buf[64]; strcpy(buf, \"x\"); }\n");
    let findings = run_ast_scanner(&dirs, true);   // experiment_scan = true
    assert!(
        findings.iter().any(|f| f.cwe_id == 120 && f.source == SastSource::Ast),
        "CWE-120 must fire with --experiment-scan; got {:?}",
        findings
    );
}
```

**AUTOSAR regression test** (`tests/autosar_ast_regression.rs` line 20) — update call:
```rust
let findings = run_ast_scanner(&dirs, false);  // baseline: no experimental rules
```

The baseline assertion (`findings.len() == 3`) should remain unchanged because the 3 baseline findings (CWE-362, 367, 369) are all non-experimental.

---

## Shared Patterns

### Feature gate
**Source:** `src/cli.rs` lines 122, 275, 283–284; `src/vulnerability/ast_scanner.rs` line 27.
**Apply to:** New CLI field, `run_ast_scanner` call site.

All AST scanner and vulnerability flags are `#[cfg(feature = "internal")]`. The `--experiment-scan` flag must carry the same gate. The `ast_scanner.rs` module itself is already `#![cfg(feature = "internal")]`, so internal gating in scanner code is automatic.

### `experimental: false` default in all non-experimental rule literals
**Apply to:** All 22 existing entries in `AST_CWE_RULES` that are NOT in the 17-CWE experimental list. Mechanical addition — same field, same value.

---

## No Analog Found

None. All changes have direct analogs in the existing codebase.

---

## Metadata

**Analog search scope:** `src/vulnerability/ast_scanner.rs`, `src/cli.rs`, `src/main.rs`, `tests/vulnerability_tests/ast_scanner_tests.rs`, `tests/autosar_ast_regression.rs`
**Files scanned:** 5
**Pattern extraction date:** 2026-05-13
