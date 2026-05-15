# Phase 19: cppcheck-removal - Pattern Map

**Mapped:** 2026-05-12
**Files analyzed:** 7 files to modify or delete
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/cwe_scanner.rs` | service | transform | self (revise in-place) | exact |
| `src/vulnerability/mod.rs` | config (re-exports) | — | self (revise in-place) | exact |
| `src/cli.rs` | config (CLI args) | request-response | self (revise in-place) | exact |
| `src/main.rs` | controller/pipeline | request-response | self (revise in-place) | exact |
| `tests/benchmark.rs` | test | — | — | deleted (no analog needed) |
| `tests/vulnerability_tests/suppression_tests.rs` | test | — | `tests/vulnerability_tests/cwe_scanner_tests.rs` | role-match |
| `tests/vulnerability_tests/mod.rs` | test config | — | self (revise in-place) | exact |

## Pattern Assignments

### `src/vulnerability/cwe_scanner.rs` (service, transform)

This is a pure deletion + in-place revision. No new code structure is introduced.

**Current imports block** (lines 13–25) — trim to remove unused imports after deletions:
```rust
#![cfg(feature = "internal")]

use crate::util::warn_on_walkdir_err;
// DELETE: use indicatif::{ProgressBar, ProgressStyle};
// DELETE: use quick_xml::events::Event;
// DELETE: use quick_xml::Reader;
use serde::Serialize;
use std::collections::{HashMap, HashSet};  // DELETE BTreeSet from here
// DELETE: use std::ffi::OsStr;
use std::io::BufRead;
use std::path::{Path, PathBuf};
// DELETE: use std::process::{Command, Stdio};
use walkdir::WalkDir;
```
Run `cargo check --features internal` after deletion — the compiler will confirm exactly which imports to remove.

**Current `SastSource` enum** (lines 29–36) — delete `Cppcheck` variant, update `Both` doc comment:
```rust
// BEFORE:
pub enum SastSource {
    Lexical,
    Cppcheck,
    Both,
    /// Phase 18: AST scanner provenance — finding produced by tree-sitter-based ast_scanner.
    Ast,
}

// AFTER (D-05, D-06):
pub enum SastSource {
    /// Finding produced by the lexical regex scanner (fallback when AST parse fails).
    Lexical,
    /// Finding confirmed by both AST scanner and Lexical fallback scanner — higher confidence.
    Both,
    /// Phase 18: finding produced by tree-sitter-based AST scanner.
    Ast,
}
```

**`CPPCHECK_COVERED_CWES` and `CPPCHECK_CWE_OVERRIDES`** (lines 454–483) — delete both entirely.

**`parse_cppcheck_xml()`** (lines 493–583) — delete entirely.

**`run_cppcheck_scanner()`** (lines 596–725) — delete entirely.

**`suppress_lexical_false_positives()`** (lines 785–818) — delete entirely.

**`deduplicate_sast_findings()`** (lines 757–783) — revise in-place. Current signature and body:
```rust
// BEFORE (lines 757–783):
/// Deduplicate the union of lexical and cppcheck findings by
/// `(normalized_file_path, line, cwe_id)` (D-11). When the same key appears
/// in both inputs, the surviving entry has its `source` set to
/// `SastSource::Both` (D-12); the lexical entry's other fields are kept as
/// the base because lexical findings carry richer component attribution.
pub fn deduplicate_sast_findings(
    lexical: Vec<SastFinding>,
    cppcheck: Vec<SastFinding>,
) -> Vec<SastFinding> {
    use std::collections::HashMap;
    let mut deduped: Vec<SastFinding> = Vec::with_capacity(lexical.len() + cppcheck.len());
    let mut seen: HashMap<(String, u32, u32), usize> = HashMap::new();

    for f in lexical {
        let key = (normalize_path(&f.file_path), f.line, f.cwe_id);
        seen.insert(key, deduped.len());
        deduped.push(f); // already SastSource::Lexical
    }

    for f in cppcheck {
        let key = (normalize_path(&f.file_path), f.line, f.cwe_id);
        if let Some(&idx) = seen.get(&key) {
            // D-12: dual-detected — promote existing lexical entry to Both.
            deduped[idx].source = SastSource::Both;
        } else {
            seen.insert(key, deduped.len());
            deduped.push(f); // already SastSource::Cppcheck
        }
    }

    deduped
}
```

```rust
// AFTER (D-07): rename params from (lexical, cppcheck) to (ast, lexical);
// first loop processes ast findings, second loop promotes to Both on collision.
/// Deduplicate the union of AST and Lexical fallback findings by
/// `(normalized_file_path, line, cwe_id)`. When the same key appears
/// in both inputs, the surviving entry has its `source` set to
/// `SastSource::Both` (indicating higher-confidence dual-detected finding);
/// the ast entry's other fields are kept as the base.
///
/// Path normalization uses `normalize_path` (resolves `.`/`..` without
/// filesystem access) so dedup works even when source files are absent
/// at call time (CI, temp dirs, archives).
pub fn deduplicate_sast_findings(
    ast: Vec<SastFinding>,
    lexical: Vec<SastFinding>,
) -> Vec<SastFinding> {
    use std::collections::HashMap;
    let mut deduped: Vec<SastFinding> = Vec::with_capacity(ast.len() + lexical.len());
    let mut seen: HashMap<(String, u32, u32), usize> = HashMap::new();

    for f in ast {
        let key = (normalize_path(&f.file_path), f.line, f.cwe_id);
        seen.insert(key, deduped.len());
        deduped.push(f); // already SastSource::Ast
    }

    for f in lexical {
        let key = (normalize_path(&f.file_path), f.line, f.cwe_id);
        if let Some(&idx) = seen.get(&key) {
            // dual-detected — promote existing ast entry to Both.
            deduped[idx].source = SastSource::Both;
        } else {
            seen.insert(key, deduped.len());
            deduped.push(f); // already SastSource::Lexical
        }
    }

    deduped
}
```

**New inline tests to add** in `#[cfg(test)] mod tests` (per RESEARCH.md Wave 0 Gaps):
```rust
// Analog pattern: existing inline tests use tempfile + SastFinding construction (lines 826–900+)
// Copy the make_finding helper pattern from suppression_tests.rs (lines 7–16):
fn make_finding(cwe_id: u32, file_path: &str, line: u32, source: SastSource) -> SastFinding {
    SastFinding {
        cwe_id,
        component_name: "lib".to_string(),
        component_ecosystem: "C/C++".to_string(),
        file_path: file_path.to_string(),
        line,
        source,
    }
}

#[test]
fn test_deduplicate_ast_and_lexical_merge() {
    // When same (file, line, cwe) appears in both ast and lexical inputs,
    // result entry has SastSource::Both.
    let ast_f = make_finding(120, "src/a.c", 10, SastSource::Ast);
    let lex_f = make_finding(120, "src/a.c", 10, SastSource::Lexical);
    let result = deduplicate_sast_findings(vec![ast_f], vec![lex_f]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].source, SastSource::Both);
}

#[test]
fn test_deduplicate_ast_only_passthrough() {
    // deduplicate_sast_findings(ast_findings, vec![]) returns findings unchanged.
    let ast_f = make_finding(120, "src/a.c", 5, SastSource::Ast);
    let result = deduplicate_sast_findings(vec![ast_f], vec![]);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].source, SastSource::Ast);
}
```

---

### `src/vulnerability/mod.rs` (config, re-exports)

**Current line 16:**
```rust
// BEFORE (line 16):
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, parse_cppcheck_xml, run_cppcheck_scanner, run_lexical_scanner, SastFinding, SastSource, suppress_lexical_false_positives};
```

```rust
// AFTER (D-04, D-10):
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, run_lexical_scanner, SastFinding, SastSource};
```

The `#[cfg(feature = "internal")]` wrapper on lines 15–16 is unchanged. The `run_ast_scanner` export on lines 18–19 is unchanged.

---

### `src/cli.rs` (config, request-response)

**Current cppcheck_path field** (lines 273–276) — delete entirely:
```rust
// DELETE these 4 lines (lines 273–276):
/// Path to cppcheck binary. When provided, uses this binary instead of PATH lookup. (v1.0.17)
#[cfg(feature = "internal")]
#[arg(long)]
pub cppcheck_path: Option<PathBuf>,
```

The surrounding fields (`supplier_config` above at lines 269–271, `sarif_output` below at lines 278–282) are unchanged.

---

### `src/main.rs` (controller/pipeline, request-response)

**Current WR-01 warning block** (lines 197–201) — delete entirely:
```rust
// DELETE (lines 197–201):
// WR-01: warn when --cppcheck-path is given without --check-vulnerabilities.
#[cfg(feature = "internal")]
if args.cppcheck_path.is_some() && !args.check_vulnerabilities {
    eprintln!("Warning: --cppcheck-path has no effect without --check-vulnerabilities");
}
```

**Current cppcheck pipeline block** (lines 248–276) — replace with simplified pipeline:
```rust
// BEFORE (lines 248–276):
// Phase 14 (D-06): cppcheck scanner runs sequentially after the AST
// scanner, in the same cfg(internal) block, before sast_findings is finalized.
let cppcheck_bin: Option<&std::ffi::OsStr> = args
    .cppcheck_path
    .as_deref()
    .map(|p| p.as_os_str());
let (cppcheck_findings, cppcheck_scanned_dirs) =
    crate::vulnerability::run_cppcheck_scanner(&component_dirs, cppcheck_bin);
sast_findings =
    crate::vulnerability::deduplicate_sast_findings(ast_findings, cppcheck_findings);
let cppcheck_confirmed: std::collections::BTreeSet<(String, u32, u32)> = sast_findings
    .iter()
    .filter(|f| f.source == crate::vulnerability::SastSource::Cppcheck
             || f.source == crate::vulnerability::SastSource::Both)
    .map(|f| (f.file_path.clone(), f.line, f.cwe_id))
    .collect();
sast_findings = crate::vulnerability::suppress_lexical_false_positives(
    sast_findings,
    &cppcheck_scanned_dirs,
    &cppcheck_confirmed,
);
```

```rust
// AFTER (D-08): ast_findings → deduplicate(ast, []) → sast_findings
// Phase 18 (D-02): AST scanner is primary; lexical fallback runs per-file inside run_ast_scanner.
// Lexical fallback findings are embedded in ast_findings with SastSource::Lexical.
let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs);
sast_findings = crate::vulnerability::deduplicate_sast_findings(ast_findings, vec![]);
```

Note: The `run_ast_scanner` call at line 246 is already present; the replacement removes only the block from line 248 onward through line 276. Retain the existing line 246 call and replace lines 248–276 with the two-line `deduplicate_sast_findings` call above.

---

### `tests/benchmark.rs` (test — delete entirely)

Delete the file. No replacement. Per D-12: the Phase 18 benchmark served its decision purpose.

---

### `tests/vulnerability_tests/suppression_tests.rs` (test — delete entirely)

Delete the file. It tests `suppress_lexical_false_positives` (being deleted, D-09) and uses `SastSource::Cppcheck` (being deleted, D-05). Leaving it causes compile failure under `--features internal`.

---

### `tests/vulnerability_tests/mod.rs` (test config — revise in-place)

**Current content** (lines 1–17) — remove `cppcheck_scanner_tests` and `suppression_tests` module declarations:
```rust
// BEFORE:
mod fix_recommendation_tests;
mod nvd_tests;
#[cfg(feature = "internal")]
mod cwe_scanner_tests;
#[cfg(feature = "internal")]
mod cppcheck_scanner_tests;    // DELETE this line
#[cfg(feature = "internal")]
mod suppression_tests;         // DELETE this line (and its cfg gate)
#[cfg(feature = "internal")]
mod sarif_fingerprint_tests;
#[cfg(feature = "internal")]
mod sarif_baseline_tests;
#[cfg(feature = "internal")]
mod sarif_consistency_tests;

#[cfg(feature = "internal")]
mod ast_scanner_tests;
```

```rust
// AFTER:
mod fix_recommendation_tests;
mod nvd_tests;
#[cfg(feature = "internal")]
mod cwe_scanner_tests;
#[cfg(feature = "internal")]
mod sarif_fingerprint_tests;
#[cfg(feature = "internal")]
mod sarif_baseline_tests;
#[cfg(feature = "internal")]
mod sarif_consistency_tests;

#[cfg(feature = "internal")]
mod ast_scanner_tests;
```

Also: `tests/vulnerability_tests/cppcheck_scanner_tests.rs` references `parse_cppcheck_xml`, `run_cppcheck_scanner`, and `SastSource` via `deduplicate_sast_findings` — this file must also be deleted, and its `mod cppcheck_scanner_tests` declaration removed from `mod.rs` (shown above).

---

## Shared Patterns

### Feature gate
**Source:** `src/vulnerability/cwe_scanner.rs` line 13
**Apply to:** All items in `cwe_scanner.rs` that are SAST-related
```rust
#![cfg(feature = "internal")]
```
No change required — the gate already exists. New tests added to `cwe_scanner.rs` inline test module inherit this gate automatically.

### `#[cfg(feature = "internal")]` on pub use exports
**Source:** `src/vulnerability/mod.rs` lines 15–16
**Apply to:** The revised `pub use` line
```rust
#[cfg(feature = "internal")]
pub use cwe_scanner::{...};
```

### Inline test helper pattern
**Source:** `tests/vulnerability_tests/suppression_tests.rs` lines 7–16
**Apply to:** New inline tests in `cwe_scanner.rs`
```rust
fn make_finding(cwe_id: u32, file_path: &str, line: u32, source: SastSource) -> SastFinding {
    SastFinding {
        cwe_id,
        component_name: "lib".to_string(),
        component_ecosystem: "C/C++".to_string(),
        file_path: file_path.to_string(),
        line,
        source,
    }
}
```

### Compile verification command
**Apply to:** After every file modification task
```
cargo check --features internal
```
**Full suite gate:** `cargo test --features internal` before phase sign-off.

---

## No Analog Found

All files in this phase are existing files being modified or deleted. No entirely new files are created, so there are no "no analog found" entries.

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `src/cli.rs`, `src/main.rs`, `tests/vulnerability_tests/`
**Files scanned:** 7 source files + 2 test files read directly
**Pattern extraction date:** 2026-05-12
**Branch:** `feature/v1.0.17-autosar-sast-sarif`
