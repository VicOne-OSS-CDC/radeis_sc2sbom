# Phase 11: Lexical Scanner + CycloneDX Output - Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 9 new/modified files
**Analogs found:** 9 / 9

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/cwe_scanner.rs` | service | file-I/O + transform | `src/parsers/c/vendored_3rdparty.rs` | role-match (file walking + per-entry transform) |
| `src/vulnerability/mod.rs` | config | — | `src/vulnerability/mod.rs` (itself) | exact (add two lines) |
| `src/models/dependency.rs` (ScanContext) | model | — | `src/models/dependency.rs` (itself) | exact (add one field) |
| `src/scanner/mod.rs` (component_dirs population) | service | CRUD | `src/scanner/mod.rs` (itself) | exact (new field init + pass to parsers) |
| `src/formats/cyclonedx.rs` (struct extension) | model | — | `src/formats/cyclonedx.rs` (itself) | exact (add two fields + new struct) |
| `src/formats/cyclonedx.rs` (build_sast_vulnerabilities) | service | transform | `build_cyclonedx_vulnerabilities` fn in same file | exact (same data flow pattern) |
| `src/formats/cyclonedx.rs` (signature change) | service | request-response | `convert_to_cyclonedx` / `save_cyclonedx_json` / `print_cyclonedx_json` in same file | exact (add trailing param pattern) |
| `tests/vulnerability_tests/cwe_scanner_tests.rs` | test | — | `tests/vulnerability_tests/nvd_tests.rs` | exact (same gated test module pattern) |
| `tests/fixtures/c/dangerous_calls.c` | config | — | No analog (new fixture file) | none |

---

## Pattern Assignments

### `src/vulnerability/cwe_scanner.rs` (service, file-I/O + transform)

**Analog:** `src/parsers/c/vendored_3rdparty.rs` (WalkDir file iteration + per-file data extraction)

**Feature gate wrapper** — all code in this file is inside the gate established by Phase 10 (D-07/D-08). The top-level `#[cfg(feature = "internal")]` wraps the entire module:
```rust
// All items in this file must be inside:
#[cfg(feature = "internal")]
```

**Imports pattern** — copy from `src/parsers/c/vendored_3rdparty.rs` lines 21-27:
```rust
use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use crate::util::warn_on_walkdir_err;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
```
For `cwe_scanner.rs`, replace with:
```rust
use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
```
`warn_on_walkdir_err` is already available from `crate::util` — use `.filter_map(warn_on_walkdir_err)` in the WalkDir chain consistent with `scan_directory` in `src/scanner/mod.rs` (line ~504).

**WalkDir file iteration pattern** — copy structure from `src/scanner/mod.rs` lines 500-508:
```rust
for entry in WalkDir::new(path)
    .follow_links(true)
    .max_depth(50)
    .into_iter()
    .filter_entry(|e| should_process_entry(e, vendor_mode, excludes))
    .filter_map(warn_on_walkdir_err)
{
    let path = entry.path();
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
```
For the scanner's inner loop, the chain simplifies (no vendor-mode filtering needed):
```rust
for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_map(warn_on_walkdir_err) {
    let path = entry.path();
    if !entry.file_type().is_file() { continue; }
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else { continue };
    if !matches!(ext, "c" | "h" | "cpp" | "hpp" | "cc") { continue; }
    // ... scan_file(path, name, ecosystem)
}
```

**File read pattern** — graceful error handling on open/read, from `src/parsers/c/autotools.rs` lines 15-23:
```rust
let content = match fs::read_to_string(path) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("Warning: Failed to read {}: {}", path.display(), e);
        return Ok(Vec::new());
    }
};
```
For the scanner, prefer `BufRead::lines()` over `read_to_string` to handle non-UTF8 gracefully (RESEARCH.md Anti-Patterns):
```rust
let file = match std::fs::File::open(path) { Ok(f) => f, Err(_) => return vec![] };
let reader = std::io::BufReader::new(file);
for (line_idx, line_result) in reader.lines().enumerate() {
    let line = match line_result { Ok(l) => l, Err(_) => continue };
    // ...
}
```

**Static rule table pattern** — `const`/`static` array at file top (RESEARCH.md Pattern 1, decisions D-07/D-08):
```rust
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
}

static CWE_RULES: &[CweRule] = &[
    CweRule { cwe_id: 120, functions: &["gets", "strcpy", ...], requires_format_heuristic: false },
    // ... 13 total rules
    CweRule { cwe_id: 134, functions: &["printf", "fprintf", ...], requires_format_heuristic: true },
];
```

**Public entry point** — free function (Claude's Discretion), follows the pattern of `query_vulnerabilities_batch` and `enrich_cwe_ids` in `src/vulnerability/mod.rs` (public re-exports from module):
```rust
pub fn run_lexical_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
) -> Vec<SastFinding> {
    // iterate component_dirs.iter() as (name, ecosystem) -> dir
    // call scan_file per C/C++ file found
    // collect all findings
}
```

**SastFinding struct** — new public struct in this file (RESEARCH.md Code Examples):
```rust
#[derive(Debug, Clone)]
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,
    pub line: u32,
}
```

---

### `src/vulnerability/mod.rs` (config — add two lines)

**Analog:** `src/vulnerability/mod.rs` itself (lines 1-6 currently):
```rust
pub mod fix_recommendations;
pub mod nvd;
pub mod osv;

pub use osv::{clear_vulnerability_cache, query_vulnerabilities_batch, OsvProvider};
pub use nvd::enrich_cwe_ids;
```

**Change:** Add `pub mod cwe_scanner;` and `pub use cwe_scanner::{SastFinding, run_lexical_scanner};` following the exact re-export pattern already used for `nvd` and `osv`. Both new lines must be inside `#[cfg(feature = "internal")]` consistent with the gating of the whole module established in Phase 10.

---

### `src/models/dependency.rs` — ScanContext field addition (model)

**Analog:** `src/models/dependency.rs` lines 443-455 (ScanContext struct itself):
```rust
#[derive(Debug)]
pub struct ScanContext {
    pub dependencies: Vec<Dependency>,
    pub npm_relationships: Vec<DependencyRelationship>,
    pub cargo_relationships: Vec<DependencyRelationship>,
    pub python_lockfile_relationships: Vec<DependencyRelationship>,
    pub ros_metadata: Option<RosPackageMetadata>,
    pub ros_packages: Vec<RosPackageWithDeps>,
    pub git_submodule_relationships: Vec<DependencyRelationship>,
    pub is_autosar: bool,
}
```

**Change:** Add one field. As decided in RESEARCH.md Pitfall 5 (unconditional field, zero-cost when empty):
```rust
pub component_dirs: HashMap<(String, String), PathBuf>,
```
Requires `use std::collections::HashMap; use std::path::PathBuf;` if not already imported at the top of `dependency.rs`.

**Construction pattern** — `src/scanner/mod.rs` lines 1052-1061 shows how `ScanContext` is constructed; the new field needs to appear there too:
```rust
Ok(ScanContext {
    dependencies: all_dependencies,
    npm_relationships,
    // ...
    is_autosar,
    component_dirs,  // new — initialize as HashMap::new() at top of scan_directory, populate during parse loop
})
```

---

### `src/scanner/mod.rs` — component_dirs population (service, CRUD)

**Analog:** `src/scanner/mod.rs` lines 734-773 (Makefile parser call block) — the pattern for calling a C/C++ parser and collecting its output:
```rust
"Makefile" | "makefile" => {
    if !scan_c_build_systems { continue; }
    // ... autotools detection logic ...
    spinner.set_message("parsing Makefile...");
    if let Ok(deps) = parse_makefile(path, scan_c_build_systems, scan_so_files, target_arch, scan_root) {
        all_dependencies.extend(deps);
        manifest_count += 1;
    }
}
```

**Change pattern:** Add `component_dirs` init at the top of `scan_directory` (alongside other `let mut` bindings at line ~427), and insert population calls inside each C/C++ parser match arm. From RESEARCH.md Pitfall 4 — simplest approach is a `&mut HashMap` passed into each parser:

At top of function body:
```rust
let mut component_dirs: HashMap<(String, String), PathBuf> = HashMap::new();
```

Inside each C/C++ parser match arm (using Makefile as example):
```rust
if let Ok(deps) = parse_makefile(path, ...) {
    // record component_dirs entry for each dep returned
    if let Some(parent) = path.parent() {
        for dep in &deps {
            component_dirs.entry((dep.name.clone(), dep.ecosystem.clone()))
                .or_insert_with(|| parent.to_path_buf());
        }
    }
    all_dependencies.extend(deps);
    manifest_count += 1;
}
```

Six C/C++ parsers need this treatment (CONTEXT.md canonical_refs): `makefile.rs`, `cmake/`, `pkgconfig.rs`, `autotools.rs`, `makefile_am.rs`, `vendored_3rdparty.rs`.

---

### `src/formats/cyclonedx.rs` — struct extension (model)

**Analog:** `src/formats/cyclonedx.rs` lines 182-218 (existing `CycloneDXVulnerability` struct and lines 220-246 for its supporting types).

**Existing struct to extend** (lines 182-218):
```rust
#[derive(Debug, Serialize)]
pub struct CycloneDXVulnerability {
    #[serde(rename = "bom-ref")]
    bom_ref: String,
    id: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<CycloneDXVulnerabilitySource>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    ratings: Vec<CycloneDXVulnerabilityRating>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    cwes: Vec<u32>,
    // ... description, recommendation, published, updated, references
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    affects: Vec<CycloneDXVulnerabilityAffect>,
}
```

**Three changes required** (RESEARCH.md Pitfall 1/2):

1. Change `CycloneDXVulnerabilitySource.url` from `String` to `Option<String>` (line 222-223):
```rust
#[derive(Debug, Serialize)]
struct CycloneDXVulnerabilitySource {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,  // was: url: String
}
```
Also update the two existing CVE construction sites at lines ~371-373:
```rust
source: Some(CycloneDXVulnerabilitySource {
    name: "OSV".to_string(),
    url: Some(format!("https://osv.dev/vulnerability/{}", vuln.id)),  // wrap in Some()
}),
```

2. Add new `CycloneDXVulnerabilityAnalysis` struct (after existing supporting structs, around line 246):
```rust
#[derive(Debug, Serialize)]
struct CycloneDXVulnerabilityAnalysis {
    state: String,
}
```

3. Add two fields to `CycloneDXVulnerability` (both skipped for CVE entries, so existing serialization is byte-for-byte unchanged):
```rust
#[serde(skip_serializing_if = "Option::is_none")]
analysis: Option<CycloneDXVulnerabilityAnalysis>,

#[serde(skip_serializing_if = "Vec::is_empty", default)]
properties: Vec<CycloneDXProperty>,
```
Note: `CycloneDXProperty` already exists — it is used on `CycloneDXComponent` (lines ~91-92). Copy its usage pattern directly.

---

### `src/formats/cyclonedx.rs` — build_sast_vulnerabilities (service, transform)

**Analog:** `build_cyclonedx_vulnerabilities` in `src/formats/cyclonedx.rs` lines 248-392. This is the exact same data-flow pattern — iterate findings, resolve bom-ref via HashMap, construct `CycloneDXVulnerability`.

**dep_to_bom_ref construction pattern** (lines 261-305) — reuse the same HashMap. The SAST path uses the same `(name, ecosystem) → bom-ref` lookup. The HashMap is built once in `convert_to_cyclonedx` and passed by reference to both functions.

**Vulnerability construction pattern** (lines 367-387) — direct copy-and-adapt for SAST:
```rust
// CVE path (existing):
cyclonedx_vulnerabilities.push(CycloneDXVulnerability {
    bom_ref: vuln_bom_ref,
    id: vuln.id.clone(),
    aliases,
    source: Some(CycloneDXVulnerabilitySource {
        name: "OSV".to_string(),
        url: Some(format!("https://osv.dev/vulnerability/{}", vuln.id)),
    }),
    ratings,
    cwes,
    // ...
    affects,
    analysis: None,       // CVE entries leave these None/empty
    properties: vec![],   // so existing output is unchanged
});

// SAST path (new, in build_sast_vulnerabilities):
// bom-ref format from D-10:
let sanitized = finding.file_path.replace('/', "-").replace('.', "-");
let bom_ref = format!("sast-{}-{}-{}", finding.cwe_id, sanitized, finding.line);

CycloneDXVulnerability {
    bom_ref,
    id: format!("CWE-{}", finding.cwe_id),
    aliases: vec![],
    source: Some(CycloneDXVulnerabilitySource {
        name: "radeis_sc2sbom static analysis".to_string(),
        url: None,   // SAST entries have no advisory URL (D-11)
    }),
    ratings: vec![],
    cwes: vec![finding.cwe_id],
    description: None,
    recommendation: None,
    published: None,
    updated: None,
    references: vec![],
    affects,
    analysis: Some(CycloneDXVulnerabilityAnalysis {
        state: "in_triage".to_string(),   // D-11
    }),
    properties: vec![
        CycloneDXProperty { name: "sc2sbom:finding:file".to_string(), value: finding.file_path.clone() },
        CycloneDXProperty { name: "sc2sbom:finding:line".to_string(), value: finding.line.to_string() },
    ],
}
```

The entire `build_sast_vulnerabilities` function and its call site inside `convert_to_cyclonedx` must be inside `#[cfg(feature = "internal")]` blocks.

---

### `src/formats/cyclonedx.rs` — signature change (service, request-response)

**Analog:** Phase 8's trailing-param pattern — `supplier_resolver: Option<&SupplierResolver>` — already established on `convert_to_cyclonedx`, `save_cyclonedx_json`, and `print_cyclonedx_json`.

**Existing signatures** (lines 394, 1112, 1119):
```rust
pub fn convert_to_cyclonedx(sbom: &Sbom, mode: &SbomMode, supplier_resolver: Option<&SupplierResolver>) -> CycloneDXDocument

pub fn print_cyclonedx_json(sbom: &Sbom, mode: &SbomMode, supplier_resolver: Option<&SupplierResolver>) -> Result<()>

pub fn save_cyclonedx_json(sbom: &Sbom, path: &str, mode: &SbomMode, supplier_resolver: Option<&SupplierResolver>) -> Result<()>
```

**Change:** Add `sast_findings: &[SastFinding]` as the trailing parameter on all three, consistent with the `supplier_resolver` trailing-param convention. The parameter and its use inside `convert_to_cyclonedx` are gated with `#[cfg(feature = "internal")]`. For the non-internal build, use a cfg block to provide a non-internal version without the param (RESEARCH.md option b):

```rust
#[cfg(feature = "internal")]
pub fn convert_to_cyclonedx(
    sbom: &Sbom,
    mode: &SbomMode,
    supplier_resolver: Option<&SupplierResolver>,
    sast_findings: &[SastFinding],
) -> CycloneDXDocument { ... }

#[cfg(not(feature = "internal"))]
pub fn convert_to_cyclonedx(
    sbom: &Sbom,
    mode: &SbomMode,
    supplier_resolver: Option<&SupplierResolver>,
) -> CycloneDXDocument { ... }
```

Call sites in `main.rs` are already inside `#[cfg(feature = "internal")]` blocks (line ~174) for vulnerability usage, so passing `&sast_findings` at call sites within the internal block is safe.

---

### `tests/vulnerability_tests/cwe_scanner_tests.rs` (test)

**Analog:** `tests/vulnerability_tests/nvd_tests.rs` — exact same structure.

**Module gate pattern** — from `tests/vulnerability_tests/mod.rs` (lines 1-2) and Phase 10 D-09, the entire vulnerability_tests module is gated. The new file must be added as:
```rust
// In tests/vulnerability_tests/mod.rs — add:
#[cfg(feature = "internal")]
mod cwe_scanner_tests;
```

**Test file header pattern** — copy from `tests/vulnerability_tests/nvd_tests.rs` lines 1-13:
```rust
use radeis_sc2sbom::vulnerability::cwe_scanner::{run_lexical_scanner, SastFinding};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
```

**Test structure pattern** — copy from `nvd_tests.rs` lines 64-76 (one test function per requirement):
```rust
#[test]
fn test_cwe120_detected() {
    let tmp = TempDir::new().unwrap();
    // write a fixture .c file with a known dangerous call
    std::fs::write(tmp.path().join("test.c"), b"void f() { strcpy(dst, src); }").unwrap();

    let mut component_dirs = HashMap::new();
    component_dirs.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());

    let findings = run_lexical_scanner(&component_dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 120));
}
```

**Note:** The `tests/fixtures/c/dangerous_calls.c` fixture file is an alternative to inline `write` calls. Either approach follows the existing test pattern in `nvd_tests.rs`. If a shared fixture is preferred, it goes in `tests/fixtures/c/`.

---

## Shared Patterns

### Feature Gate (`#[cfg(feature = "internal")]`)
**Source:** Phase 10; established in `src/main.rs` lines 174-212 and `src/vulnerability/mod.rs`
**Apply to:** All new code in `cwe_scanner.rs`, all SAST-related additions in `cyclonedx.rs`, the `run_lexical_scanner` call site in `main.rs`, the new re-exports in `vulnerability/mod.rs`, and the test module entry in `vulnerability_tests/mod.rs`

Pattern from `src/main.rs` lines 174-175:
```rust
#[cfg(feature = "internal")]
{
    // ... scanner invocation, formatter call with sast_findings
}
```

### serde Skip Pattern for Optional/Empty Fields
**Source:** `src/formats/cyclonedx.rs` throughout (e.g., lines 34-35, 40, 78-79, 84-85)
**Apply to:** New `analysis` and `properties` fields on `CycloneDXVulnerability`, and the changed `url` field on `CycloneDXVulnerabilitySource`

```rust
#[serde(skip_serializing_if = "Option::is_none")]   // for Option fields
#[serde(skip_serializing_if = "Vec::is_empty", default)]   // for Vec fields
```

### Graceful Error Return from Parsers
**Source:** `src/parsers/c/autotools.rs` lines 15-23, `src/parsers/c/makefile.rs` lines 49-55
**Apply to:** `scan_file` function in `cwe_scanner.rs` — return empty vec on file open/read error rather than propagating

```rust
let content = match fs::read_to_string(path) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("Warning: Failed to read {}: {}", path.display(), e);
        return Ok(Vec::new());
    }
};
```

### `warn_on_walkdir_err` in WalkDir chains
**Source:** `src/scanner/mod.rs` line ~504, `src/parsers/c/vendored_3rdparty.rs` line ~27 (import)
**Apply to:** The WalkDir chain in `run_lexical_scanner`

```rust
.filter_map(warn_on_walkdir_err)
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `tests/fixtures/c/dangerous_calls.c` | test fixture | — | No C fixture files exist in the codebase today; this is the first |

---

## Metadata

**Analog search scope:** `src/formats/`, `src/models/`, `src/scanner/`, `src/parsers/c/`, `src/vulnerability/`, `tests/vulnerability_tests/`
**Files scanned:** 11 source files read directly
**Pattern extraction date:** 2026-05-09
