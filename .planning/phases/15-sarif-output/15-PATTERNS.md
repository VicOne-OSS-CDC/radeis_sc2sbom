# Phase 15: sarif-output - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 5 (1 new, 4 modified)
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/formats/sarif.rs` | format-writer (service) | file-I/O, transform | `src/formats/console.rs` (`save_static_analysis_report`) | exact |
| `src/formats/mod.rs` | config/routing | — | `src/formats/mod.rs` itself | self (surgical edit) |
| `src/cli.rs` | config | request-response | `src/cli.rs` lines 122–148 (internal feature-gated args) | exact |
| `src/main.rs` | controller | request-response | `src/main.rs` lines 285–286 and 369–370 (existing call sites) | exact |
| `tests/format_tests/sarif_tests.rs` | test | — | `tests/format_tests/sast_report_tests.rs` | exact |

---

## Pattern Assignments

### `src/formats/sarif.rs` (format-writer, file-I/O + transform)

**Analog:** `src/formats/console.rs` — `save_static_analysis_report` (lines 1938–2009+)

**Imports pattern** (analog: `console.rs` lines 1–14):
```rust
#[cfg(feature = "internal")]
use crate::vulnerability::cwe_scanner::SastFinding;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
```

**Feature gate + module-level cfg pattern** (analog: `cwe_scanner.rs` line 13):
```rust
// Option A — module-level gate (entire file disabled in non-internal builds):
#![cfg(feature = "internal")]

// Option B — per-item gate (used in console.rs):
#[cfg(feature = "internal")]
fn cwe_name(...) { ... }

#[cfg(feature = "internal")]
pub fn save_static_analysis_report(...) { ... }
```
`cwe_scanner.rs` uses the module-level `#![cfg(feature = "internal")]` pattern. Use the same for `sarif.rs` since every item in it is internal.

**Private struct + serde pattern** (new pattern for this file, following RESEARCH.md D-10/D-11):
```rust
#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRule {
    id: String,
    name: String,
    help_uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: u32,
}
```

**Core pattern — function signature** (analog: `console.rs` lines 1959–1964):
```rust
// Analog (console.rs lines 1959-1964):
#[cfg(feature = "internal")]
pub fn save_static_analysis_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
) -> Result<()> {

// New function mirrors this signature plus optional override path:
#[cfg(feature = "internal")]
pub fn save_sarif_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
    sarif_path: Option<&str>,
) -> Result<()> {
```

**Path resolution pattern** (analog: `console.rs` line 1934 `fs::write(path, output)?`; path construction from other format writers):
```rust
let path = match sarif_path {
    Some(p) => PathBuf::from(p),
    None => out_dir.join(format!("{}_static_analysis.sarif", project_name)),
};
if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
}
```

**BTreeSet deduplication pattern** (analog: `console.rs` line 1984 `BTreeMap` for stable order):
```rust
// Analog uses BTreeMap for deterministic grouping (console.rs line 1984):
use std::collections::BTreeMap;
let mut counts: BTreeMap<(String, u32), usize> = BTreeMap::new();

// sarif.rs uses BTreeSet for deterministic CWE deduplication:
let unique_cwes: BTreeSet<u32> = findings.iter().map(|f| f.cwe_id).collect();
let rules: Vec<SarifRule> = unique_cwes.iter().map(|&id| SarifRule {
    id: format!("CWE-{}", id),
    name: cwe_name(id).to_string(),
    help_uri: format!("https://cwe.mitre.org/data/definitions/{}.html", id),
}).collect();
```

**results[] construction** (no deduplication — one entry per finding):
```rust
let results: Vec<SarifResult> = findings.iter().map(|f| SarifResult {
    rule_id: format!("CWE-{}", f.cwe_id),
    message: SarifMessage { text: cwe_name(f.cwe_id).to_string() },
    locations: vec![SarifLocation {
        physical_location: SarifPhysicalLocation {
            artifact_location: SarifArtifactLocation { uri: f.file_path.clone() },
            region: SarifRegion { start_line: f.line },
        },
    }],
}).collect();
```

**JSON serialization + write pattern** (serde_json, consistent with cyclonedx.rs `to_string_pretty` usage):
```rust
let json = serde_json::to_string_pretty(&log)?;
fs::write(&path, json)?;
eprintln!("✓ SARIF report saved to: {}", path.display());
Ok(())
```

**eprintln confirmation pattern** (analog: `main.rs` line 284 and 368):
```rust
// main.rs line 284:
eprintln!("✓ Console report saved to: {}", out_path.display());
// sarif.rs mirrors this inside the function itself:
eprintln!("✓ SARIF report saved to: {}", path.display());
```

**cwe_name import** (from sibling module, after visibility change to `pub(crate)`):
```rust
use super::console::cwe_name;
```

---

### `src/formats/mod.rs` (config/routing — surgical edit)

**Analog:** `src/formats/mod.rs` itself (lines 1–14)

**Existing pattern to mirror** (lines 1–8):
```rust
// Current mod.rs (lines 1-8):
pub mod console;
pub mod cyclonedx;
pub mod spdx;

// Re-export commonly used functions
pub use console::{print_sbom, save_console_report};
#[cfg(feature = "internal")]
pub use console::save_static_analysis_report;
```

**Additions to make** — insert after line 1 (`pub mod console;`) and after line 8:
```rust
// After existing mod declarations (line 1 area):
pub mod sarif;

// After existing #[cfg(feature = "internal")] pub use console::save_static_analysis_report; (line 8):
#[cfg(feature = "internal")]
pub use sarif::save_sarif_report;
```

---

### `src/cli.rs` (config — surgical edit)

**Analog:** `src/cli.rs` lines 122–148 (feature-gated `Option<String>` args)

**Exact pattern to copy** (lines 122–124, the simplest gated optional arg):
```rust
/// Enable vulnerability checking (requires network connection)
#[cfg(feature = "internal")]
#[arg(long, action = ArgAction::Set, default_value_t = false)]
pub check_vulnerabilities: bool,
```

**Pattern for a simple gated `Option<String>` flag** (lines 168–171):
```rust
/// SBOM output mode: complete (all packages) or vulnerable-only (packages with CVEs)
#[cfg(feature = "internal")]
#[arg(long, value_enum, default_value = "complete")]
pub sbom_mode: SbomMode,
```

**Insertion target** — after `pub supplier_config: Option<PathBuf>` (line 271), before the closing `}` of `Args` (line 272):
```rust
/// SARIF output file path for static analysis findings (v1.0.17)
/// Defaults to {out_dir}/{project_name}_static_analysis.sarif
#[cfg(feature = "internal")]
#[arg(long)]
pub sarif_output: Option<String>,
```

---

### `src/main.rs` (controller — surgical edit at two call sites)

**Analog:** `src/main.rs` lines 285–286 and 369–370 (existing `save_static_analysis_report` call sites)

**Existing call site pattern** (lines 285–286):
```rust
#[cfg(feature = "internal")]
save_static_analysis_report(project_name, out_dir, &sast_findings)?;
```

**New import to add** (analog: line 27 `use formats::save_static_analysis_report`):
```rust
#[cfg(feature = "internal")]
use formats::save_sarif_report;
```

**Insertion at both call sites** — immediately after the existing `save_static_analysis_report` line:
```rust
// At line 286 (first call site):
#[cfg(feature = "internal")]
save_static_analysis_report(project_name, out_dir, &sast_findings)?;
#[cfg(feature = "internal")]
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;

// At line 370 (second call site) — identical pattern:
#[cfg(feature = "internal")]
save_static_analysis_report(project_name, out_dir, &sast_findings)?;
#[cfg(feature = "internal")]
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;
```

---

### `tests/format_tests/sarif_tests.rs` (test)

**Analog:** `tests/format_tests/sast_report_tests.rs` (lines 1–70+)

**File header + feature gate pattern** (lines 1–13):
```rust
#![cfg(feature = "internal")]

use radeis_sc2sbom::formats::save_sarif_report;
use radeis_sc2sbom::vulnerability::cwe_scanner::SastFinding;
use std::path::PathBuf;
use tempfile::TempDir;
```

**Test helper pattern** (analog: lines 17–25):
```rust
fn make_finding(component: &str, file: &str, line: u32, cwe_id: u32) -> SastFinding {
    SastFinding {
        cwe_id,
        component_name: component.to_string(),
        component_ecosystem: "vendored".to_string(),
        file_path: file.to_string(),
        line,
    }
}
```

**Test structure pattern** (analog: lines 29–42):
```rust
#[test]
fn test_save_sarif_report_with_findings() {
    let tmp = TempDir::new().unwrap();
    let findings = vec![
        make_finding("libfoo", "src/libfoo/buffer.c", 42, 120),
        make_finding("libfoo", "src/libfoo/exec.c", 17, 78),
        make_finding("libbar", "src/libbar/utils.c", 99, 120),
    ];
    save_sarif_report("myproj", tmp.path(), &findings, None).unwrap();
    let out = std::fs::read_to_string(
        tmp.path().join("myproj_static_analysis.sarif")
    ).unwrap();
    // assert on JSON structure...
}
```

**mod.rs registration pattern** (analog: `tests/format_tests/mod.rs` lines 5–6):
```rust
// In tests/format_tests/mod.rs — add after existing sast_report_tests entry:
#[cfg(feature = "internal")]
pub mod sarif_tests;
```

---

## Shared Patterns

### Feature Gate (`#[cfg(feature = "internal")]`)

**Source:** `src/formats/console.rs` lines 1938–1939 and 1959–1960; `src/cli.rs` lines 122–123
**Apply to:** `save_sarif_report` function, all SARIF structs (inherited via module-level `#![cfg(...)]`), `sarif_output` CLI field, `save_sarif_report` re-export in `mod.rs`, `use formats::save_sarif_report` import in `main.rs`

```rust
// Function-level gate (console.rs pattern):
#[cfg(feature = "internal")]
pub fn save_sarif_report(...) -> Result<()> { ... }

// Module-level gate (cwe_scanner.rs pattern — use this for sarif.rs):
#![cfg(feature = "internal")]
```

### Error Handling (anyhow `?` propagation)

**Source:** `src/formats/console.rs` line 1934 (`fs::write(path, output)?`)
**Apply to:** `save_sarif_report` — all `fs::write`, `fs::create_dir_all`, and `serde_json::to_string_pretty` calls use `?` operator directly.

```rust
// Analog pattern (console.rs line 1934):
fs::write(path, output)?;
Ok(())

// sarif.rs mirrors:
let json = serde_json::to_string_pretty(&log)?;
fs::write(&path, json)?;
Ok(())
```

### `cwe_name()` Visibility Change

**Source:** `src/formats/console.rs` line 1939
**Apply to:** `src/formats/console.rs` only — one character change from `fn` to `pub(crate) fn`

```rust
// Before (line 1939):
fn cwe_name(cwe_id: u32) -> &'static str {

// After:
pub(crate) fn cwe_name(cwe_id: u32) -> &'static str {
```

### `eprintln!` Success Confirmation

**Source:** `src/main.rs` line 284 (`eprintln!("✓ Console report saved to: {}", out_path.display())`)
**Apply to:** `save_sarif_report` — emit confirmation inside the function, same as the analog `save_static_analysis_report` context shows.

```rust
eprintln!("✓ SARIF report saved to: {}", path.display());
```

### VERSION constant

**Source:** `src/formats/console.rs` line 18 (`const VERSION: &str = env!("CARGO_PKG_VERSION")`)
**Apply to:** `sarif.rs` — use `env!("CARGO_PKG_VERSION")` directly inline in the `SarifDriver` struct literal; no need for a separate constant since it's used once.

```rust
driver: SarifDriver {
    name: "sc2sbom",
    version: env!("CARGO_PKG_VERSION"),
    rules,
},
```

---

## No Analog Found

None — all files have direct analogs in the codebase.

---

## Metadata

**Analog search scope:** `src/formats/`, `src/cli.rs`, `src/main.rs`, `tests/format_tests/`
**Files scanned:** 8 source files read; 51 test files discovered
**Pattern extraction date:** 2026-05-10
