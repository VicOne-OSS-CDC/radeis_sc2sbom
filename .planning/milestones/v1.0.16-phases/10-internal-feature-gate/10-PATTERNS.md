# Phase 10: Internal Feature Gate - Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 14 files (modified) + 1 new file
**Analogs found:** 14 / 15 (1 new stub has no analog by design)

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `Cargo.toml` | config | build | `Cargo.toml` `cn-release = []` feature | exact (same file, same pattern) |
| `src/lib.rs` | config | build | `src/lib.rs` (existing `pub mod` declarations) | exact |
| `src/main.rs` | controller | request-response | `src/main.rs` (existing `mod`/`use` structure) | exact |
| `src/models/mod.rs` | model | transform | `src/models/mod.rs` (existing `pub use` blocks) | exact |
| `src/models/dependency.rs` | model | transform | `src/models/dependency.rs` (existing `#[serde(...)]` field annotations) | exact |
| `src/cli.rs` | config | request-response | `src/cli.rs` lines 83–95 (`#[cfg_attr(feature = "cn-release", ...)]`) | exact |
| `src/formats/cyclonedx.rs` | service | transform | `src/formats/cyclonedx.rs` (existing struct + fn layout) | exact |
| `src/formats/spdx.rs` | service | transform | `src/formats/spdx.rs` (existing fn layout) | exact |
| `src/formats/console.rs` | service | request-response | `src/formats/console.rs` (existing enum/fn layout) | exact |
| `src/vulnerability/mod.rs` | service | request-response | `src/models/mod.rs` (`pub mod` + `pub use` pattern) | role-match |
| `src/vulnerability/cwe_scanner.rs` | service | batch | none (new stub file) | no analog |
| `tests/all_tests.rs` | test | — | `tests/all_tests.rs` existing `#[path = ...]` `mod` declarations | exact |
| `tests/vulnerability_tests/mod.rs` | test | — | `tests/all_tests.rs` inner `mod` declarations | role-match |
| `.github/workflows/build-release.yml` | config | build | `.github/workflows/build-release.yml` lines 251, 382 | exact |
| `scripts/strip_vulnerability.sh` | config | batch | `scripts/strip_vulnerability.sh` step 1 comment block | exact |

---

## Pattern Assignments

### `Cargo.toml` (config, build)

**Analog:** `Cargo.toml` lines 12–14 — existing `[features]` and `[dependencies]` sections.

**Existing feature pattern** (lines 12–14):
```toml
[features]
# CN regional release: appends vulnerability assessment service info to --help output
cn-release = []
```

**Existing reqwest dep** (line 27):
```toml
reqwest = { version = "0.11", features = ["blocking", "json", "native-tls-vendored"] }
```

**Target pattern — add `internal` feature and make `reqwest` optional:**
```toml
[features]
cn-release = []
# Internal builds: enables CVE scanning (OSV), CWE enrichment (NVD), lexical CWE scanner
internal = ["dep:reqwest"]

[dependencies]
reqwest = { version = "0.11", optional = true, features = ["blocking", "json", "native-tls-vendored"] }
```

Note: `"dep:reqwest"` (Cargo 1.60+ syntax) avoids creating an implicit `reqwest` feature. Project has no `rust-version` constraint and targets current stable, so this syntax is safe.

---

### `src/lib.rs` (config, build)

**Analog:** `src/lib.rs` lines 1–12 — existing `pub mod` declarations.

**Current state** (lines 1–12):
```rust
// Library entry point for radeis_sc2sbom
// This allows integration tests and external crates to use the library

pub mod classifier;
pub mod cli;
pub mod formats;
pub mod models;
pub mod parsers;
pub mod scanner;
pub mod supplier;
pub mod util;
pub mod vulnerability;
```

**Target pattern — gate the vulnerability module declaration:**
```rust
pub mod classifier;
pub mod cli;
pub mod formats;
pub mod models;
pub mod parsers;
pub mod scanner;
pub mod supplier;
pub mod util;
#[cfg(feature = "internal")]
pub mod vulnerability;
```

---

### `src/main.rs` (controller, request-response)

**Analog:** `src/main.rs` lines 1–24 — existing `mod`/`use` block; lines 172–207 — vulnerability scanning block.

**Current mod/use block** (lines 1–24):
```rust
mod vulnerability;
// ...
use vulnerability::{clear_vulnerability_cache, enrich_cwe_ids, query_vulnerabilities_batch, OsvProvider};
```

**Current vulnerability scanning block** (lines 172–207):
```rust
// Phase 3: Optionally query vulnerabilities
if args.check_vulnerabilities {
    if args.clear_cache {
        if let Err(e) = clear_vulnerability_cache() {
            eprintln!("Warning: Failed to clear cache: {}", e);
        }
    }
    eprintln!("Querying OSV vulnerability database...");
    match OsvProvider::new(args.cache_ttl, args.vulnerability_timeout) {
        Ok(provider) => {
            if let Err(e) = query_vulnerabilities_batch(...) { ... }
        }
        Err(e) => { ... }
    }
    enrich_cwe_ids(&mut dependencies, ...);
}
```

**Target pattern — gate mod declaration, use statement, and scanning block:**
```rust
#[cfg(feature = "internal")]
mod vulnerability;
// ... other mods unchanged ...

#[cfg(feature = "internal")]
use vulnerability::{clear_vulnerability_cache, enrich_cwe_ids, query_vulnerabilities_batch, OsvProvider};

// ... later in main() ...
#[cfg(feature = "internal")]
{
    // Phase 3: Optionally query vulnerabilities
    if args.check_vulnerabilities {
        // ... entire block unchanged inside the cfg block ...
    }
}
```

Also gate any `args.vulnerability_output`, `args.sbom_mode`, `args.max_vulns_per_severity` references that appear outside the scanning block by verifying they are only used inside the `#[cfg(feature = "internal")]` block or within gated formatter calls.

---

### `src/models/mod.rs` (model, transform)

**Analog:** `src/models/mod.rs` lines 1–14 — existing `pub mod` + `pub use` block.

**Current state** (lines 1–14):
```rust
pub mod dependency;
pub mod graph;
pub mod sbom;
pub mod vulnerability;

pub use dependency::{
    AIModelMetadata, AutosarMetadata, BaseModelInfo, Dependency, DependencyRelationship,
    DependencyScope, DependencySource, LockFileData, ScanContext, SubModelInfo,
};
pub use graph::{DependencyGraph, DependencyNode};
pub use sbom::{RosPackageMetadata, RosPackageWithDeps, Sbom, ScopeStatistics};
pub use vulnerability::{
    Confidence, FixAction, FixRecommendation, Vulnerability, VulnerabilitySeverity,
};
```

**Target pattern — gate mod and re-exports:**
```rust
pub mod dependency;
pub mod graph;
pub mod sbom;
#[cfg(feature = "internal")]
pub mod vulnerability;

pub use dependency::{
    AIModelMetadata, AutosarMetadata, BaseModelInfo, Dependency, DependencyRelationship,
    DependencyScope, DependencySource, LockFileData, ScanContext, SubModelInfo,
};
pub use graph::{DependencyGraph, DependencyNode};
pub use sbom::{RosPackageMetadata, RosPackageWithDeps, Sbom, ScopeStatistics};
#[cfg(feature = "internal")]
pub use vulnerability::{
    Confidence, FixAction, FixRecommendation, Vulnerability, VulnerabilitySeverity,
};
```

---

### `src/models/dependency.rs` (model, transform)

**Analog:** `src/models/dependency.rs` — existing `#[serde(...)]` field annotations (lines 289–342) and `Default` impl (lines 353–378).

**Existing field with serde annotations** (lines 295–296):
```rust
#[serde(skip_serializing_if = "Vec::is_empty", default)]
pub vulnerabilities: Vec<Vulnerability>,
```

**Existing use statement** (line 2):
```rust
use super::vulnerability::Vulnerability;
```

**Existing Default impl** (lines 353–378): names `vulnerabilities: Vec::new()` explicitly.

**Target pattern — gate import, field, and Default field:**
```rust
// Line 2 — gate the import
#[cfg(feature = "internal")]
use super::vulnerability::Vulnerability;

// Field inside Dependency struct — gate the field declaration
#[cfg(feature = "internal")]
#[serde(skip_serializing_if = "Vec::is_empty", default)]
pub vulnerabilities: Vec<Vulnerability>,

// Inside Default impl — gate the field initializer
impl Default for Dependency {
    fn default() -> Self {
        Dependency {
            name: String::new(),
            // ... all other fields unchanged ...
            #[cfg(feature = "internal")]
            vulnerabilities: Vec::new(),
            // ...
        }
    }
}
```

**Construction site migration pattern** — 72 `vulnerabilities:` occurrences across 39 source files and 58 occurrences across 13 test files must be migrated. Use `Dependency::new()` (lines 384–390) as the reference — it already uses `..Default::default()`:
```rust
// Existing Dependency::new() — correct pattern already established (line 384)
pub fn new(name: String, version: String, ecosystem: String) -> Self {
    Dependency {
        name,
        version,
        ecosystem,
        ..Default::default()
    }
}
```
All construction sites with explicit `vulnerabilities: vec![]` or `vulnerabilities: Vec::new()` must switch to this pattern.

---

### `src/cli.rs` (config, request-response)

**Analog:** `src/cli.rs` lines 83–95 — existing `#[cfg_attr(feature = "cn-release", ...)]` on `Args` struct. This is the ONLY existing `#[cfg_attr]` / feature gate in the entire codebase — it is the canonical model.

**Existing `cfg_attr` pattern** (lines 83–95):
```rust
#[derive(Parser, Debug)]
#[command(name = "SBOM Scanner")]
#[command(about = "Scans a folder for open source dependencies and generates SBOM", long_about = None)]
#[cfg_attr(
    feature = "cn-release",
    command(after_help = "\
══════════════════════════════════════════════════════════════════════\n\
📊 SBOM vulnerability assessment service\n\
...")
)]
pub struct Args {
```

**Enums to gate** (lines 29–78): `MinSeverity` (lines 29–50), `VulnerabilityOutputMode` (lines 52–60), `SbomMode` (lines 72–78).

**Target pattern — gate entire enum definitions:**
```rust
#[cfg(feature = "internal")]
#[derive(Debug, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum MinSeverity { Low, Medium, High, Critical }

#[cfg(feature = "internal")]
impl MinSeverity {
    pub fn to_level(&self) -> u8 { ... }
}

#[cfg(feature = "internal")]
#[derive(Debug, Clone, ValueEnum)]
pub enum VulnerabilityOutputMode { Summary, Tree, Detailed }

#[cfg(feature = "internal")]
#[derive(Debug, Clone, ValueEnum)]
pub enum SbomMode { Complete, VulnerableOnly }
```

**Args fields to gate** (lines 118–159): `check_vulnerabilities`, `min_severity`, `vulnerability_timeout`, `vulnerability_output`, `cache_ttl`, `clear_cache`, `max_vulns_per_severity`, `sbom_mode` — 8 fields total.

**Target pattern for each Args field:**
```rust
#[cfg(feature = "internal")]
#[arg(long, action = ArgAction::Set, default_value_t = false)]
pub check_vulnerabilities: bool,

#[cfg(feature = "internal")]
#[arg(long, value_enum, default_value = "low")]
pub min_severity: MinSeverity,

// ... same pattern for all 8 fields ...
```

---

### `src/formats/cyclonedx.rs` (service, transform)

**Analog:** `src/formats/cyclonedx.rs` — existing struct definitions and function layout.

**Current imports to gate** (lines 1–5):
```rust
use crate::cli::SbomMode;
// ...
    VulnerabilitySeverity,
```

**Structs to gate** (lines 180–243): `CycloneDXVulnerability`, `CycloneDXVulnerabilitySource`, `CycloneDXVulnerabilityRating`, `CycloneDXVulnerabilityReference`, `CycloneDXVulnerabilityAffect`.

**Field to gate on `CycloneDXDocument`** (line 41):
```rust
pub vulnerabilities: Option<Vec<CycloneDXVulnerability>>,
```

**Function to gate** (lines 248–393): `build_cyclonedx_vulnerabilities`.

**Target pattern — gate imports, structs, field, and function with `#[cfg(feature = "internal")]`:**
```rust
#[cfg(feature = "internal")]
use crate::cli::SbomMode;
// Gate VulnerabilitySeverity in the models import block

// Gate the field inside CycloneDXDocument:
#[cfg(feature = "internal")]
pub vulnerabilities: Option<Vec<CycloneDXVulnerability>>,

// Gate entire supporting structs:
#[cfg(feature = "internal")]
#[derive(Debug, Serialize)]
pub struct CycloneDXVulnerability { ... }
// ... etc for all supporting structs ...

// Gate the builder function:
#[cfg(feature = "internal")]
fn build_cyclonedx_vulnerabilities(...) -> Vec<CycloneDXVulnerability> { ... }
```

The `convert_to_cyclonedx` function signature uses `mode: &SbomMode` — gate this parameter and the `SbomMode` filtering block inside the function body. Public-build signature removes `mode` parameter (matches what strip script does at steps 7–8).

---

### `src/formats/spdx.rs` (service, transform)

**Analog:** `src/formats/spdx.rs` — existing function signatures with `mode: &SbomMode`.

**Current import to gate** (line 1):
```rust
use crate::cli::SbomMode;
```

**Vulnerability reference block to gate** (lines 223–241): the `// Add vulnerability references` loop.

**SbomMode filtering block to gate** (lines 558–562) inside `convert_to_spdx`.

**Target pattern — same `#[cfg(feature = "internal")]` block wrapping as cyclonedx.rs:**
```rust
#[cfg(feature = "internal")]
use crate::cli::SbomMode;

// Inside convert_to_spdx, gate the SbomMode filtering:
#[cfg(feature = "internal")]
let filtered: Vec<_> = match mode {
    SbomMode::Complete => sbom.dependencies.iter().collect(),
    SbomMode::VulnerableOnly => sbom.dependencies.iter().filter(...).collect(),
};

// Gate the vulnerability reference block:
#[cfg(feature = "internal")]
{
    // Add vulnerability references
    for vuln in &dep.vulnerabilities { ... }
}
```

Public-build function signatures for `print_spdx_json`, `print_spdx_tag_value`, `convert_to_spdx`, `save_spdx_json`, `save_spdx_tag_value` will lose the `mode: &SbomMode` parameter.

---

### `src/formats/console.rs` (service, request-response)

**Analog:** `src/formats/console.rs` — existing enum and function definitions.

**Current imports to gate** (lines 1–4):
```rust
use crate::cli::{TreeStyle, VulnerabilityOutputMode};
use crate::models::{
    Dependency, DependencyGraph, DependencyNode, DependencyRelationship, Sbom, Vulnerability,
    VulnerabilitySeverity,
};
```

**Target pattern — gate vulnerability-related imports, keeping unconditional ones:**
```rust
use crate::cli::TreeStyle;
#[cfg(feature = "internal")]
use crate::cli::VulnerabilityOutputMode;
use crate::models::{
    Dependency, DependencyGraph, DependencyNode, DependencyRelationship, Sbom,
};
#[cfg(feature = "internal")]
use crate::models::{Vulnerability, VulnerabilitySeverity};
```

Gate the functions `count_unique_cves`, `print_vulnerabilities_hierarchical`, and all code blocks referencing `dep.vulnerabilities` or `VulnerabilityOutputMode`.

---

### `src/vulnerability/mod.rs` (service, request-response)

**Analog:** `src/models/mod.rs` — same `pub mod` + `pub use` pattern.

**Current state** (lines 1–6):
```rust
pub mod fix_recommendations;
pub mod nvd;
pub mod osv;

pub use osv::{clear_vulnerability_cache, query_vulnerabilities_batch, OsvProvider};
pub use nvd::enrich_cwe_ids;
```

**Target pattern — add `cwe_scanner` module declaration:**
```rust
pub mod cwe_scanner;
pub mod fix_recommendations;
pub mod nvd;
pub mod osv;

pub use osv::{clear_vulnerability_cache, query_vulnerabilities_batch, OsvProvider};
pub use nvd::enrich_cwe_ids;
```

No further `#[cfg]` needed here — the entire file is unreachable when `mod vulnerability;` in `lib.rs`/`main.rs` is gated. The parent gate is sufficient.

---

### `src/vulnerability/cwe_scanner.rs` (new stub, batch)

**No analog.** This is a new file with no existing counterpart.

**Target content** — minimal comment stub only:
```rust
// Phase 11: lexical CWE scanner — implementation pending
// This file is the landing zone for the C/C++ static analysis scanner (SCAN-01..SCAN-05)
```

File is compiled only when `internal` feature is active (parent module is gated in `lib.rs`/`main.rs`).

---

### `tests/all_tests.rs` (test, —)

**Analog:** `tests/all_tests.rs` lines 1–30 — existing `#[path = ...]` module declarations.

**Current state** (lines 25–26):
```rust
#[path = "vulnerability_tests/mod.rs"]
mod vulnerability_tests;
```

**Target pattern — gate the vulnerability_tests module:**
```rust
#[cfg(feature = "internal")]
#[path = "vulnerability_tests/mod.rs"]
mod vulnerability_tests;
```

This is cleaner than gating inside `vulnerability_tests/mod.rs` — `mod.rs` itself needs no change.

---

### `tests/vulnerability_tests/mod.rs` (test, —)

**Analog:** `tests/all_tests.rs` — parent file handles gating via `#[cfg]` on the `mod` declaration.

**Current state** (lines 1–2):
```rust
mod fix_recommendation_tests;
mod nvd_tests;
```

**Target pattern:** No change needed if `tests/all_tests.rs` is the gating point. The `#[cfg(feature = "internal")]` on the `mod` declaration in `all_tests.rs` excludes the entire subtree.

---

### Test construction sites — 13 test files (test, —)

**Analog:** `src/models/dependency.rs` lines 384–390 — `Dependency::new()` already uses `..Default::default()`.

**Files with `vulnerabilities:` explicit field** (58 occurrences across 13 files):
- `tests/scanner_tests/deduplication_tests.rs` (6 occurrences)
- `tests/format_tests/spdx_tests.rs` (16 occurrences)
- `tests/format_tests/cyclonedx_tests.rs` (6 occurrences)
- `tests/parser_tests/c_tests.rs` (2 occurrences)
- `tests/parser_tests/conan_tests.rs` (1 occurrence)
- `tests/parser_tests/ros_tests.rs` (6 occurrences)
- `tests/model_tests/dependency_tests.rs` (4 occurrences)
- `tests/model_tests/sbom_tests.rs` (2 occurrences)
- `tests/classifier_tests/scope_filter_tests.rs` (1 occurrence)
- `tests/classifier_tests/autosar_classification_tests.rs` (1 occurrence)
- `tests/integration_tests/mcu_project_tests.rs` (2 occurrences)
- `tests/integration_tests/autosar_e2e_tests.rs` (1 occurrence)
- `tests/integration_tests/scope_filtering_integration_tests.rs` (10 occurrences)

**Also: 72 occurrences in 39 `src/` files** (parsers, scanners, classifiers, formatters) — same migration required for all construction sites that name `vulnerabilities:` explicitly.

**Migration pattern** — copy from `Dependency::new()`:
```rust
// BEFORE
Dependency {
    name: "foo".to_string(),
    version: "1.0".to_string(),
    ecosystem: "npm".to_string(),
    source: DependencySource::LockFile,
    vulnerabilities: vec![],
    // ... all other fields ...
}

// AFTER
Dependency {
    name: "foo".to_string(),
    version: "1.0".to_string(),
    ecosystem: "npm".to_string(),
    source: DependencySource::LockFile,
    ..Default::default()
}
```

**SbomMode in format tests:** `tests/format_tests/cyclonedx_tests.rs` imports and uses `SbomMode`. These usages must be wrapped in `#[cfg(feature = "internal")]` blocks, or the `SbomMode` parameter removed from the tested function call (matching the non-internal function signature).

---

### `.github/workflows/build-release.yml` (config, build)

**Analog:** `.github/workflows/build-release.yml` lines 251, 382 — existing `cargo build --release` commands.

**Current state** (lines 251, 382):
```yaml
cargo build --release --target ${{ matrix.target }}
```

**Target pattern — add `--features internal`:**
```yaml
cargo build --release --features internal --target ${{ matrix.target }}
```

Apply to both occurrences. Also add `--features internal` to the validate-sbom step's SBOM generation command (line 444 per research).

---

### `scripts/strip_vulnerability.sh` (config, batch)

**Analog:** `scripts/strip_vulnerability.sh` lines 24–28 — existing step 1 that removes `src/vulnerability/` directory.

**Current state** (lines 24–25):
```bash
echo "  [1] Removing src/vulnerability/ directory..."
rm -rf src/vulnerability/
```

`rm -rf src/vulnerability/` already removes `cwe_scanner.rs` when present (D-14 is satisfied implicitly). The required change is minimal: add a comment noting that `cwe_scanner.rs` is covered by the directory removal. No additional `rm` command is needed.

---

## Shared Patterns

### Feature gate — module level
**Source:** Research patterns + `src/lib.rs:12` (current unconditional `pub mod vulnerability;`)
**Apply to:** `src/lib.rs`, `src/main.rs`, `src/models/mod.rs`
```rust
#[cfg(feature = "internal")]
pub mod vulnerability;
```

### Feature gate — item level (enum, function, struct field)
**Source:** Research patterns confirmed by `src/cli.rs:83` (`#[cfg_attr(feature = "cn-release", ...)]`) — the only existing feature gate in codebase
**Apply to:** All gated enums in `cli.rs`, all gated structs/functions in formatter files, `vulnerabilities` field in `dependency.rs`
```rust
#[cfg(feature = "internal")]
pub enum VulnerabilityOutputMode { ... }

#[cfg(feature = "internal")]
pub vulnerabilities: Vec<Vulnerability>,
```

### Struct update syntax (construction site migration)
**Source:** `src/models/dependency.rs:384–390` — `Dependency::new()` already established this pattern
**Apply to:** All 130 construction sites (`vulnerabilities:` occurrences) in `src/` and `tests/`
```rust
Dependency {
    name: "foo".to_string(),
    version: "1.0".to_string(),
    ecosystem: "npm".to_string(),
    ..Default::default()
}
```

### Optional dependency in Cargo features
**Source:** `Cargo.toml` lines 12–14 (`cn-release = []`) + Cargo Book `dep:` syntax
**Apply to:** `Cargo.toml` only
```toml
internal = ["dep:reqwest"]
reqwest = { version = "0.11", optional = true, features = [...] }
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/vulnerability/cwe_scanner.rs` | service | batch | New stub file; no existing lexical scanner in codebase |

---

## Metadata

**Analog search scope:** `src/`, `tests/`, `Cargo.toml`, `.github/workflows/`, `scripts/`
**Files scanned:** 15 modified/created files; 52 analog source files examined
**Pattern extraction date:** 2026-05-09
