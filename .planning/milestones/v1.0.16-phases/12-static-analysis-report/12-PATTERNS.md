# Phase 12: Static Analysis Report - Pattern Map

**Mapped:** 2026-05-09
**Files analyzed:** 4 (2 new/modified, plus 2 test files)
**Analogs found:** 4 / 4

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/formats/console.rs` (modify) | formatter | transform | `src/formats/console.rs` itself | exact — add function + section to existing file |
| `src/formats/sast_report.rs` (optional new) | formatter | transform | `src/formats/cyclonedx.rs` + `src/formats/console.rs` | role-match |
| `src/formats/mod.rs` (modify if new module) | config/module | — | `src/formats/mod.rs` itself | exact |
| `src/main.rs` (modify) | dispatcher | request-response | `src/main.rs` lines 237–254, 309–335 | exact |
| `tests/format_tests/sast_report_tests.rs` (new) | test | — | `tests/format_tests/cyclonedx_tests.rs` | role-match |
| `tests/format_tests/mod.rs` (modify) | test config | — | `tests/format_tests/mod.rs` itself | exact |

---

## Pattern Assignments

### `src/formats/console.rs` — new `save_static_analysis_report()` function

**Analog:** `src/formats/console.rs` lines 1109–1859 (`save_console_report`)

**Imports pattern** (lines 1–9 of `src/formats/console.rs`):
```rust
use crate::cli::{TreeStyle, VulnerabilityOutputMode};
use crate::models::{
    Dependency, DependencyGraph, DependencyNode, DependencyRelationship, Sbom, Vulnerability,
    VulnerabilitySeverity,
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs;
```

New imports needed inside `#[cfg(feature = "internal")]` block:
```rust
#[cfg(feature = "internal")]
use crate::vulnerability::cwe_scanner::SastFinding;
```

**Function signature pattern** (line 1109 of `src/formats/console.rs`):
```rust
pub fn save_console_report(
    sbom: &Sbom,
    path: &str,
    tree_style: &TreeStyle,
    vulnerability_output: &VulnerabilityOutputMode,
    max_vulns_per_severity: usize,
    relationships: &[DependencyRelationship],
    summary_only: bool,
    check_vulnerabilities: bool,
) -> Result<()> {
```

New function should follow this signature shape:
```rust
#[cfg(feature = "internal")]
pub fn save_static_analysis_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
) -> Result<()> {
```

**Core file-write pattern** (lines 1119 and 1857 of `src/formats/console.rs`):
```rust
let mut output = String::new();

writeln!(output, "# SBOM Report\n")?;
writeln!(output, "**Project Path:** `{}`", sbom.project_path.display())?;
writeln!(output, "**Generated At:** {}\n", sbom.generated_at)?;

// ... build content via writeln! ...

fs::write(path, output)?;
Ok(())
```

**Output path construction pattern** (lines 238–243 of `src/main.rs`):
```rust
let project_name = sbom.project_path.file_name()
    .and_then(|n| n.to_str()).unwrap_or("sbom");
let out_dir = Path::new(out);
std::fs::create_dir_all(out_dir)?;
let out_path = out_dir.join(format!("{}_report.md", project_name));
let out_path_str = out_path.to_string_lossy();
```

For the static analysis report, replace the `join(format!(...))` with:
```rust
let path = out_dir.join(format!("{}_static_analysis.md", project_name));
```

**Save confirmation pattern — `eprintln!`** (line 254 of `src/main.rs`):
```rust
eprintln!("✓ Console report saved to: {}", out_path.display());
```

The disclaimer and confirmation are both `eprintln!` calls inside `save_static_analysis_report()`:
```rust
eprintln!("Pattern-based — complex data-flow vulnerabilities not covered");
eprintln!("✓ Static analysis report saved to: {}", path.display());
```

**Vulnerability section written inside `save_console_report`** (lines 1300–1438):
The CVE section ends around line 1439 (transition to summary-only / Dependencies). The "## Static Analysis Findings" section must be written **before** that transition point, after the CVE block closes. The addition is purely additive — a `#[cfg(feature = "internal")]` block inserted at that position. No function signature change (use approach (c) from RESEARCH.md Open Question 2).

---

### `src/formats/console.rs` — SAST section injected into `save_console_report`

**Where to inject:** After the closing `}` of the `if check_vulnerabilities && total_vulns > 0 { ... }` block (around line 1438), before the `if summary_only` block that starts the Dependencies section (line 1440).

**Pattern for a new conditional section** (lines 1253–1294 of `src/formats/console.rs` as template):
```rust
if check_vulnerabilities {
    writeln!(output, "**Vulnerabilities:** {} total ...", ...)?;
    // ...
}
```

New section follows the same shape:
```rust
#[cfg(feature = "internal")]
{
    writeln!(output, "## Static Analysis Findings\n")?;
    // summary table only (D-08)
    writeln!(output, "| Component | CWE | Name | Count |")?;
    writeln!(output, "|-----------|-----|------|-------|")?;
    if findings.is_empty() {
        writeln!(output, "| — | — | No static analysis findings detected. | — |")?;
    } else {
        // one row per CWE per component
    }
    writeln!(output)?;
}
```

---

### `src/main.rs` — call site for `save_static_analysis_report`

**Analog:** `src/main.rs` lines 237–254 (Console arm) and lines 309–335 (All arm)

**Console arm pattern** (lines 237–254):
```rust
OutputFormat::Console => {
    if let Some(ref out) = args.output {
        let project_name = sbom.project_path.file_name()
            .and_then(|n| n.to_str()).unwrap_or("sbom");
        let out_dir = Path::new(out);
        std::fs::create_dir_all(out_dir)?;
        let out_path = out_dir.join(format!("{}_report.md", project_name));
        let out_path_str = out_path.to_string_lossy();
        save_console_report(
            &sbom,
            &out_path_str,
            &args.tree_style,
            &args.vulnerability_output,
            args.max_vulns_per_severity,
            &all_relationships,
            args.summary_only,
            args.check_vulnerabilities,
        )?;
        eprintln!("✓ Console report saved to: {}", out_path.display());
    } else {
        // ...
    }
}
```

New call is added immediately after `save_console_report(...)` in both the Console arm and the All arm:
```rust
#[cfg(feature = "internal")]
{
    save_static_analysis_report(project_name, out_dir, &sast_findings)?;
}
```

**All arm structure** (lines 309–335) — `project_name` and `out_dir` are already in scope as local variables. Same `#[cfg(feature = "internal")]` block pattern applies after the `save_console_report(...)` call at line 334.

**Feature-gated `use` import pattern** (lines 9–10 and 24–25 of `src/main.rs`):
```rust
#[cfg(feature = "internal")]
mod vulnerability;

#[cfg(feature = "internal")]
use vulnerability::{clear_vulnerability_cache, enrich_cwe_ids, query_vulnerabilities_batch, OsvProvider};
```

`save_static_analysis_report` must be added to the feature-gated use block:
```rust
#[cfg(feature = "internal")]
use formats::save_static_analysis_report;
```

---

### `src/formats/mod.rs` — re-export if `sast_report.rs` is created

**Analog:** `src/formats/mod.rs` lines 1–12 (current content)

**Current re-export pattern**:
```rust
pub mod console;
pub mod cyclonedx;
pub mod spdx;

// Re-export commonly used functions
pub use console::{print_sbom, save_console_report};

pub use spdx::{
    create_package_url, print_spdx_json, print_spdx_tag_value, save_spdx_json, save_spdx_tag_value,
};

pub use cyclonedx::{print_cyclonedx_json, save_cyclonedx_json};
```

If `save_static_analysis_report` lives in `console.rs`, add to the existing `pub use console::` line:
```rust
pub use console::{print_sbom, save_console_report, save_static_analysis_report};
```

If a separate `sast_report.rs` module is created:
```rust
#[cfg(feature = "internal")]
pub mod sast_report;
#[cfg(feature = "internal")]
pub use sast_report::save_static_analysis_report;
```

---

### `tests/format_tests/sast_report_tests.rs` — new test file

**Analog:** `tests/format_tests/cyclonedx_tests.rs` lines 1–50

**Imports and helper pattern** (lines 1–50 of `tests/format_tests/cyclonedx_tests.rs`):
```rust
use radeis_sc2sbom::cli::SbomMode;
use radeis_sc2sbom::formats::cyclonedx::{
    convert_to_cyclonedx, create_cyclonedx_metadata, create_dependency_component,
};
use radeis_sc2sbom::models::{
    AIModelMetadata, AutosarMetadata, Dependency, DependencyScope, DependencySource,
    RosPackageMetadata, RosPackageWithDeps, Sbom, SubModelInfo,
};
use serde_json;
use std::path::PathBuf;

// ---- Test helpers ----

fn make_dep(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        // ... fields ...
    }
}

fn make_sbom(deps: Vec<Dependency>) -> Sbom {
    Sbom {
        project_path: std::path::PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: deps,
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    }
}

// ---- End test helpers ----

#[test]
fn test_convert_to_cyclonedx_basic() { ... }
```

New test file should follow the same structure with `#[cfg(feature = "internal")]` gating:
```rust
#[cfg(feature = "internal")]
mod tests {
    use radeis_sc2sbom::formats::save_static_analysis_report;
    use radeis_sc2sbom::vulnerability::cwe_scanner::SastFinding;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_finding(component: &str, file: &str, line: u32, cwe_id: u32, func: &str) -> SastFinding {
        // construct from actual SastFinding fields — verify field names from cwe_scanner.rs
    }

    #[test]
    fn test_save_static_analysis_report() { ... }

    #[test]
    fn test_static_analysis_report_zero_findings() { ... }

    #[test]
    fn test_console_report_includes_sast_section() { ... }
}
```

---

### `tests/format_tests/mod.rs` — add new test module

**Analog:** `tests/format_tests/mod.rs` lines 1–4 (current content)

```rust
// Format test modules
pub mod cyclonedx_tests;
pub mod spdx_tests;
pub mod spdx_validation_tests;
```

Add:
```rust
#[cfg(feature = "internal")]
pub mod sast_report_tests;
```

---

## Shared Patterns

### Buffer-then-write (applies to `save_static_analysis_report`)

**Source:** `src/formats/console.rs` lines 1119 and 1857
**Apply to:** `save_static_analysis_report` body

```rust
let mut output = String::new();
// ... writeln! calls build content ...
fs::write(path, output)?;
Ok(())
```

### `writeln!` markdown table rows

**Source:** `src/formats/console.rs` throughout the report (e.g., summary stats inline use same `writeln!(output, "| {} | ...", ...)?` idiom)
**Apply to:** Summary table and findings sections in both `save_static_analysis_report` and the injected section in `save_console_report`

### `eprintln!` save confirmations

**Source:** `src/main.rs` line 254, 276, 290, 304, 335, 347
**Apply to:** End of `save_static_analysis_report` — two calls in sequence:
```rust
eprintln!("Pattern-based — complex data-flow vulnerabilities not covered");
eprintln!("✓ Static analysis report saved to: {}", path.display());
```

### `#[cfg(feature = "internal")]` block gating

**Source:** `src/main.rs` lines 9–10, 24–25, 174–212
**Apply to:** All new code in this phase — function definition, `use` import, call sites, test module declaration

```rust
#[cfg(feature = "internal")]
{
    // scanner / formatter code
}
```

### Project name derivation

**Source:** `src/main.rs` lines 238–239
**Apply to:** Call site in `main.rs` before constructing the static analysis file path

```rust
let project_name = sbom.project_path.file_name()
    .and_then(|n| n.to_str()).unwrap_or("sbom");
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src/vulnerability/cwe_scanner.rs` (SastFinding struct) | model | — | Phase 11 not yet executed; struct field names at Claude's discretion per Phase 11 D-01. Planner must insert a Wave 0 task to read `cwe_scanner.rs` and confirm actual field names before writing report formatter code. |

---

## Key Anti-Pattern Warnings (from RESEARCH.md)

1. **Do not change `save_console_report` signature** — add the SAST section inside the function body using a `#[cfg(feature = "internal")]` block, passing `findings` as a variable captured from scope or as a local empty vec in non-internal builds. Avoid two-version signature divergence.
2. **Do not stream writes** — use `String` buffer + `fs::write`; no `BufWriter`.
3. **Do not omit the section for zero findings** — D-09 requires the section always present in `_report.md` when feature is active.
4. **Do not emit disclaimer from `main.rs`** — single `eprintln!` inside `save_static_analysis_report()` only (prevents double-print).
5. **Gate the `SastFinding` import** — `use crate::vulnerability::cwe_scanner::SastFinding` must be inside `#[cfg(feature = "internal")]` or the non-internal build will fail.

---

## Metadata

**Analog search scope:** `src/formats/`, `src/main.rs`, `tests/format_tests/`
**Files scanned:** 8 source files read, 4 analog files
**Pattern extraction date:** 2026-05-09
