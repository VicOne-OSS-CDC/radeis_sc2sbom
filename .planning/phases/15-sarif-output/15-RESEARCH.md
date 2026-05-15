# Phase 15: sarif-output - Research

**Researched:** 2026-05-10
**Domain:** SARIF 2.1 JSON serialization in Rust; CLI flag addition; format writer module pattern
**Confidence:** HIGH

## Summary

Phase 15 writes SAST findings to a SARIF 2.1 JSON file. All architectural decisions were locked during discussion (D-01 through D-11). The implementation is entirely in-process: a new `src/formats/sarif.rs` module mirrors the existing `save_static_analysis_report` pattern from `src/formats/console.rs`, uses `serde_json::to_string_pretty` with hand-rolled structs, and adds a single `--sarif-output` CLI flag. Zero new dependencies are required — `serde_json` and `serde` (with `derive` feature) are already in `Cargo.toml`.

The SARIF 2.1 schema is well-specified in the decisions. The required field set is narrow: `$schema`, `version`, `runs[0].tool.driver` (name, version, `rules[]`), and `runs[0].results[]` (ruleId, message.text, locations with physicalLocation). CWE deduplication applies only to `rules[]` — each finding remains a distinct result entry.

The `cwe_name()` helper in `src/formats/console.rs:1939` is the canonical source for CWE names; it must be called from `sarif.rs` via `pub(crate)` visibility change or by moving it to a shared location accessible within the `formats` crate. The simplest approach (matching the zero-new-files spirit of D-11) is to make `cwe_name` `pub(crate)` and import it in `sarif.rs`.

**Primary recommendation:** Implement `save_sarif_report` following the exact same structure as `save_static_analysis_report`: feature-gated, same function signature shape, writes to `{out_dir}/{project_name}_static_analysis.sarif` by default, accepts `Option<&str>` override path.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Default SARIF file path is `{out_dir}/{project_name}_static_analysis.sarif` — mirrors the `.md` file exactly.
- **D-02:** `--sarif-output <PATH>` accepts any absolute or relative file path. It is fully independent of `--output`; the two flags are orthogonal.
- **D-03:** The SARIF file is always written, even when findings is empty — write a valid SARIF file with empty `results[]` and `rules[]` arrays.
- **D-04:** Populate: `$schema`, `version`, `runs[0].tool.driver` (name, version, `rules[]`), `runs[0].results[]` each with `ruleId`, `message.text`, and `locations[].physicalLocation` (`artifactLocation.uri` + `region.startLine`).
- **D-05:** `rules[]` entry per detected CWE: `id` = `"CWE-{N}"`, `name` = CWE name string (from existing `cwe_name()`), `helpUri` = `"https://cwe.mitre.org/data/definitions/{N}.html"`.
- **D-06:** Do NOT add artifactContents, fingerprints, logical locations, or function names.
- **D-07:** New `src/formats/sarif.rs`. Exports `save_sarif_report(project_name: &str, out_dir: &Path, findings: &[SastFinding], sarif_path: Option<&str>)`. Added to `src/formats/mod.rs` via `pub use sarif::save_sarif_report`.
- **D-08:** `main.rs` calls `save_sarif_report` immediately after each `save_static_analysis_report` call (lines 286 and 370). Same findings slice.
- **D-09:** Both `save_static_analysis_report` and `save_sarif_report` remain gated behind `#[cfg(feature = "internal")]`.
- **D-10:** Hand-rolled SARIF structs with `#[derive(Serialize)]`. Use `serde_json::to_string_pretty`. Zero new dependencies.
- **D-11:** SARIF structs are private to `sarif.rs`. No separate models file.

### Claude's Discretion

- `tool.driver.name` value — use `"sc2sbom"` (matches `src/formats/cyclonedx.rs:1193` which uses `"radeis_sc2sbom"` as the full name but `"sc2sbom:finding:file"` as the prefix; CycloneDX line 446 uses `"radeis_sc2sbom static analysis"` — use `"sc2sbom"` as the short form consistent with the finding property key prefix).
- `tool.driver.version` — use `env!("CARGO_PKG_VERSION")` at compile time (consistent with `src/formats/console.rs:18` which defines `const VERSION: &str = env!("CARGO_PKG_VERSION")`).

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SARIF-01 | All `SastFinding` entries (lexical + cppcheck) written to `_static_analysis.sarif` alongside `_static_analysis.md` | D-01, D-03, D-07, D-08: new module + two call sites in main.rs |
| SARIF-02 | `--sarif-output` CLI flag for custom path | D-02, D-07: `sarif_path: Option<&str>` param + clap arg in cli.rs |
| SARIF-03 | `rules[]` per detected CWE with `id`, `name`, `helpUri` | D-05: deduplicated by CWE, uses `cwe_name()`, links to CWE.mitre.org |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SARIF JSON serialization | API / Backend (CLI process) | — | Pure data transformation in-process; no browser/server tier |
| File I/O (write .sarif file) | API / Backend (CLI process) | — | Same as other format writers: `fs::write` in the format module |
| CLI flag parsing | API / Backend (CLI process) | — | Clap `Args` struct in `src/cli.rs` |
| CWE name lookup | API / Backend (CLI process) | — | `cwe_name()` from `console.rs` — reuse directly |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0 (already in Cargo.toml) | `#[derive(Serialize)]` on SARIF structs | Already a project dependency with `features = ["derive"]` [VERIFIED: Cargo.toml] |
| serde_json | 1.0 (already in Cargo.toml) | `to_string_pretty` for SARIF JSON output | Already a project dependency; used throughout formats/ [VERIFIED: Cargo.toml] |
| clap | 4.5 (already in Cargo.toml) | `--sarif-output` flag via `#[arg(long)]` | Already the CLI framework [VERIFIED: Cargo.toml] |

### Supporting

None — no supporting libraries needed beyond what is already present.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled structs + serde | `serde_sarif` crate | Adds a dependency; CONTEXT.md D-10 locked against this |
| `to_string_pretty` | `to_string` | Compact JSON is valid SARIF but less readable in diffs; pretty is consistent with how CycloneDX JSON is written |

**Installation:** No new dependencies. Zero `Cargo.toml` changes.

## Architecture Patterns

### System Architecture Diagram

```
CLI args (--sarif-output)
         |
         v
    src/cli.rs (Args.sarif_output: Option<String>)
         |
         v
    src/main.rs:286, src/main.rs:370
    (immediately after save_static_analysis_report call)
         |
         v
    src/formats/sarif.rs
    save_sarif_report(project_name, out_dir, findings, sarif_path)
         |
         +-- compute output path:
         |     if sarif_path.is_some() -> use provided path
         |     else -> out_dir/{project_name}_static_analysis.sarif
         |
         +-- build SarifLog struct:
         |     runs[0].tool.driver.rules = deduplicated CWEs from findings
         |     runs[0].results = one entry per SastFinding
         |
         +-- serde_json::to_string_pretty(&sarif_log)
         |
         v
    fs::write(path, json)   -->   {project_name}_static_analysis.sarif
```

### Recommended Project Structure

```
src/formats/
├── mod.rs          # add: mod sarif; pub use sarif::save_sarif_report
├── console.rs      # existing; cwe_name() becomes pub(crate)
├── sarif.rs        # NEW: SARIF structs + save_sarif_report
├── cyclonedx.rs    # existing
└── spdx.rs         # existing
```

### Pattern 1: SARIF Struct Layout

**What:** Hand-rolled structs with `#[derive(Serialize)]` and `#[serde(rename_all = "camelCase")]` where SARIF uses camelCase JSON keys.

**When to use:** Always — these are private to `sarif.rs` per D-11.

```rust
// Source: SARIF 2.1.0 spec §3 + project pattern from src/formats/console.rs

#[cfg(feature = "internal")]
use serde::Serialize;

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
    id: String,          // "CWE-120"
    name: String,        // "Buffer Copy without Checking Size of Input"
    help_uri: String,    // "https://cwe.mitre.org/data/definitions/120.html"
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

**SARIF field name note:** SARIF 2.1 JSON uses camelCase throughout. `#[serde(rename_all = "camelCase")]` handles the translation from Rust snake_case. `$schema` must use `#[serde(rename = "$schema")]` since `$` is not valid in Rust identifiers. [ASSUMED — based on SARIF spec knowledge; the camelCase requirement is well-established in the spec]

### Pattern 2: Path Resolution Logic

```rust
#[cfg(feature = "internal")]
pub fn save_sarif_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
    sarif_path: Option<&str>,
) -> Result<()> {
    let path = match sarif_path {
        Some(p) => PathBuf::from(p),
        None => out_dir.join(format!("{}_static_analysis.sarif", project_name)),
    };
    // ...
    fs::write(&path, json)?;
    eprintln!("✓ SARIF report saved to: {}", path.display());
    Ok(())
}
```

### Pattern 3: CLI Flag (matching existing gated flag pattern)

```rust
// In src/cli.rs, inside the `Args` struct, after `supplier_config`:
/// SARIF output file path for static analysis findings (v1.0.17)
/// Defaults to {out_dir}/{project_name}_static_analysis.sarif
#[cfg(feature = "internal")]
#[arg(long)]
pub sarif_output: Option<String>,
```

### Pattern 4: Call Sites in main.rs

Both call sites follow this pattern (lines 286 and 370):

```rust
#[cfg(feature = "internal")]
save_static_analysis_report(project_name, out_dir, &sast_findings)?;
#[cfg(feature = "internal")]
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;
```

### Pattern 5: rules[] Deduplication

```rust
// Collect unique CWE IDs, preserving order of first occurrence
use std::collections::BTreeSet;
let unique_cwes: BTreeSet<u32> = findings.iter().map(|f| f.cwe_id).collect();
let rules: Vec<SarifRule> = unique_cwes.iter().map(|&cwe_id| SarifRule {
    id: format!("CWE-{}", cwe_id),
    name: cwe_name(cwe_id).to_string(),
    help_uri: format!("https://cwe.mitre.org/data/definitions/{}.html", cwe_id),
}).collect();
```

`BTreeSet` gives deterministic sorted order in output — consistent with the `BTreeMap` usage in `save_static_analysis_report`. [VERIFIED: console.rs:1984 uses BTreeMap for the same reason]

### Anti-Patterns to Avoid

- **Importing `cwe_name` from console.rs as `pub`:** Make it `pub(crate)` — it should not be part of the public API. The function is internal to the formats crate.
- **Using `serde_json::json!` macro:** Requires runtime key-value construction with no type safety. The struct approach matches how other format writers in this project work.
- **Skipping the `#[cfg(feature = "internal")]` gate:** Both the struct definitions and the public function must be gated. The structs are private, so they inherit the cfg gate from the function, but the `use` declarations and `mod sarif` also need correct gating in `mod.rs`.
- **Writing `sarif_path` to `out_dir` when it's absolute:** When `sarif_path` is provided, use it as-is (`PathBuf::from(p)`); do NOT join it onto `out_dir`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON serialization | Custom JSON string builder | `serde_json::to_string_pretty` | Already in project; handles escaping, nesting, unicode |
| CLI arg parsing | Manual argv parsing | `clap` `#[arg(long)]` | Already the project framework; free help text, error handling |
| CWE name lookup | Duplicate match table | `cwe_name()` from `console.rs` | Single source of truth already exists; just change visibility |

**Key insight:** The entire implementation reuses what already exists. The only genuinely new code is the SARIF struct definitions and the path/serialization logic in `save_sarif_report`.

## Common Pitfalls

### Pitfall 1: `cwe_name` Visibility

**What goes wrong:** `cwe_name` is currently a private `fn` in `console.rs`. `sarif.rs` cannot call a private function from a sibling module.

**Why it happens:** Both modules are children of `formats`, but Rust's privacy rules mean a `fn` in `console.rs` without `pub` or `pub(crate)` is invisible to `sarif.rs`.

**How to avoid:** Change `fn cwe_name` to `pub(crate) fn cwe_name` in `console.rs`. This is the minimum visibility change and keeps it out of the public API.

**Warning signs:** `error[E0603]: function 'cwe_name' is private` at compile time.

### Pitfall 2: Missing `mod sarif` in mod.rs Feature Gate

**What goes wrong:** `pub mod sarif;` without `#[cfg(feature = "internal")]` will cause a compile error in non-internal builds because `sarif.rs` contains `#[cfg(feature = "internal")]` at the file level but the module declaration is still processed.

**Why it happens:** The existing `save_static_analysis_report` re-export in `mod.rs` is wrapped in `#[cfg(feature = "internal")]`, but the module declaration `pub mod console;` is not gated. For `sarif.rs`, the module file itself has `#![cfg(feature = "internal")]` at the top, so the declaration in `mod.rs` can be ungated — but the `pub use` of `save_sarif_report` must be cfg-gated.

**How to avoid:** Mirror the exact pattern already used for `console.rs`. The `pub mod sarif;` declaration can be ungated (since `sarif.rs` will use module-level `#[cfg(...)]` or the function-level attribute). The `pub use sarif::save_sarif_report` must be wrapped in `#[cfg(feature = "internal")]`.

**Warning signs:** `error[E0432]: unresolved import` or `error[E0425]: cannot find function 'save_sarif_report'` in non-internal builds.

### Pitfall 3: `$schema` Field Serialization

**What goes wrong:** Trying to name a Rust struct field `schema` and emit JSON key `$schema` without the rename attribute produces `"schema": "..."` in output. SARIF validators reject the file.

**Why it happens:** `$` is not a valid Rust identifier character, so the field must be renamed via `#[serde(rename = "$schema")]`.

**How to avoid:** Use `#[serde(rename = "$schema")]` on the `schema` field of `SarifLog`.

**Warning signs:** SARIF validators report missing `$schema` property despite the field being present.

### Pitfall 4: `--sarif-output` With Non-Existent Parent Directory

**What goes wrong:** If the user specifies `--sarif-output /tmp/results/my.sarif` but `/tmp/results/` does not exist, `fs::write` fails with "No such file or directory".

**Why it happens:** Unlike `--output` (which triggers `std::fs::create_dir_all`), the `--sarif-output` path is a direct file path; the parent directory may not exist.

**How to avoid:** Before `fs::write`, call `if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }`. This matches robust behavior expected for user-specified paths.

**Warning signs:** `Os { code: 2, kind: NotFound, message: "No such file or directory" }` when user provides a path with a new directory component.

### Pitfall 5: Duplicate `results[]` Entries vs. Duplicate `rules[]` Entries

**What goes wrong:** Deduplicating findings at the results level (emitting only one result per CWE) loses file+line precision that IDEs use for inline annotations.

**Why it happens:** CONTEXT.md `<specifics>` explicitly states: "results[] entry per SastFinding (no deduplication at result level)".

**How to avoid:** Deduplicate only `rules[]` by CWE. Keep all findings in `results[]`.

## Code Examples

### Complete save_sarif_report skeleton

```rust
// Source: project patterns from src/formats/console.rs:1960 and CONTEXT.md decisions

#[cfg(feature = "internal")]
pub fn save_sarif_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
    sarif_path: Option<&str>,
) -> Result<()> {
    use std::collections::BTreeSet;

    // Resolve output path (D-01, D-02)
    let path = match sarif_path {
        Some(p) => PathBuf::from(p),
        None => out_dir.join(format!("{}_static_analysis.sarif", project_name)),
    };

    // Create parent dir if needed (for --sarif-output with custom paths)
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // rules[]: one entry per unique CWE (D-05)
    let unique_cwes: BTreeSet<u32> = findings.iter().map(|f| f.cwe_id).collect();
    let rules: Vec<SarifRule> = unique_cwes.iter().map(|&id| SarifRule {
        id: format!("CWE-{}", id),
        name: cwe_name(id).to_string(),
        help_uri: format!("https://cwe.mitre.org/data/definitions/{}.html", id),
    }).collect();

    // results[]: one entry per finding (D-04, no deduplication)
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

    let log = SarifLog {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "sc2sbom",
                    version: env!("CARGO_PKG_VERSION"),
                    rules,
                },
            },
            results,
        }],
    };

    let json = serde_json::to_string_pretty(&log)?;
    fs::write(&path, json)?;
    eprintln!("✓ SARIF report saved to: {}", path.display());
    Ok(())
}
```

### mod.rs additions

```rust
// In src/formats/mod.rs
pub mod sarif;  // ungated, mirrors pub mod console;

#[cfg(feature = "internal")]
pub use sarif::save_sarif_report;
```

### console.rs visibility change

```rust
// Before (line 1939):
fn cwe_name(cwe_id: u32) -> &'static str {

// After:
pub(crate) fn cwe_name(cwe_id: u32) -> &'static str {
```

### sarif.rs imports

```rust
#[cfg(feature = "internal")]
use crate::formats::console::cwe_name;
// or, since sarif.rs is in the same crate:
use super::console::cwe_name;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SARIF 1.0 (pre-2018) | SARIF 2.1.0 (OASIS standard) | OASIS ratified 2019 | SARIF 2.1 is what GitHub Advanced Security, VS Code, and Azure DevOps consume |
| `serde_sarif` crate | Hand-rolled structs | N/A (CONTEXT.md D-10) | Zero new deps; structs are < 60 lines for our required subset |

**Deprecated/outdated:**
- SARIF 1.0: rejected by modern tooling; use 2.1.0 only.
- `sarif-spec` npm validation: not relevant here; `$schema` URI in the output is sufficient for tooling auto-detection.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | SARIF 2.1 uses camelCase JSON keys (e.g., `ruleId`, `startLine`, `physicalLocation`) | Architecture Patterns — struct layout | Wrong casing → SARIF consumers reject or silently ignore fields; LOW risk since SARIF spec is stable and this is well-known |
| A2 | `"sc2sbom"` is the right `driver.name` (shorter form over `"radeis_sc2sbom"`) | Claude's Discretion section | Cosmetic inconsistency only; no functional impact |
| A3 | `env!("CARGO_PKG_VERSION")` resolves at compile time to `"1.0.16"` for current build | Code examples | Version in SARIF will lag if version bump happens; acceptable since all format writers use the same macro |

## Open Questions

1. **cwe_name visibility in Phase 14 context**
   - What we know: `cwe_name` is private to `console.rs`; Phase 14 (cppcheck) may also need it for `results[].message.text`
   - What's unclear: Whether Phase 14 already changed the visibility (Phase 14 is not yet implemented as of research date)
   - Recommendation: Change to `pub(crate)` in Phase 15 regardless — this is safe and forward-compatible

## Environment Availability

Step 2.6: SKIPPED (no external dependencies — pure in-process Rust code with existing crate dependencies)

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | `Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test --features internal sarif` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SARIF-01 | `save_sarif_report` writes `{project_name}_static_analysis.sarif` with all findings | unit | `cargo test --features internal sarif` | ❌ Wave 0 |
| SARIF-01 | Empty findings produces valid SARIF with empty `results[]` and `rules[]` | unit | `cargo test --features internal sarif` | ❌ Wave 0 |
| SARIF-02 | `--sarif-output` flag overrides default path | unit | `cargo test --features internal sarif` | ❌ Wave 0 |
| SARIF-03 | `rules[]` contains deduplicated CWEs with `id`, `name`, `helpUri` | unit | `cargo test --features internal sarif` | ❌ Wave 0 |
| SARIF-03 | 50 findings all CWE-120 → exactly one rule entry | unit | `cargo test --features internal sarif` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --features internal sarif`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/format_tests/sarif_tests.rs` — covers all SARIF-01/02/03 behaviors; mirror structure of `tests/format_tests/sast_report_tests.rs`
- [ ] Add `mod sarif_tests;` entry to `tests/format_tests/mod.rs`

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | File path from user: `PathBuf::from(p)` — no shell expansion, no path traversal beyond what the OS enforces |
| V6 Cryptography | no | — |

### Known Threat Patterns for SARIF/file-output stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `--sarif-output ../../../../etc/cron.d/evil` | Tampering | `PathBuf::from` does not shell-expand; OS permissions enforce write access; acceptable risk for a local developer tool |
| CWE name injection in JSON | Tampering | `cwe_name()` returns `&'static str` — fixed strings, no user input; `serde_json` escapes all string values |

## Sources

### Primary (HIGH confidence)

- [VERIFIED: Cargo.toml] — serde, serde_json, clap versions confirmed in project
- [VERIFIED: src/formats/console.rs:1939] — `cwe_name()` function confirmed present with 14 CWE mappings
- [VERIFIED: src/formats/console.rs:1960] — `save_static_analysis_report` signature and pattern confirmed
- [VERIFIED: src/formats/mod.rs] — existing `pub use` and feature gate pattern confirmed
- [VERIFIED: src/cli.rs:219] — `output: Option<String>` flag pattern confirmed; end of Args struct confirmed at line 272
- [VERIFIED: src/main.rs:286,370] — both `save_static_analysis_report` call sites confirmed
- [VERIFIED: src/formats/cyclonedx.rs:1193] — `"radeis_sc2sbom"` / `"sc2sbom"` name usage confirmed
- [CITED: https://docs.oasis-open.org/sarif/sarif/v2.1.0/] — SARIF 2.1 field names (`ruleId`, `startLine`, etc.)
- [CITED: https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json] — canonical `$schema` URI per CONTEXT.md `<specifics>`

### Secondary (MEDIUM confidence)

- [VERIFIED: npm registry / Context7 serde_json docs] — `#[derive(Serialize)]` and `#[serde(rename_all = "camelCase")]` confirmed as standard serde patterns

### Tertiary (LOW confidence)

None.

## Project Constraints (from CLAUDE.md)

No project-level `CLAUDE.md` exists in the repository. Global CLAUDE.md directives apply:

- Minimum code that solves the problem — no speculative features
- Surgical changes — touch only what is required (one new file, two call sites, one visibility change, one CLI field)
- Match existing style — feature gates, eprintln! patterns, function signatures must mirror `console.rs`

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries verified in Cargo.toml
- Architecture: HIGH — all decisions locked in CONTEXT.md with exact file/line references verified in codebase
- Pitfalls: HIGH — each pitfall traced to verified codebase evidence
- SARIF schema field names: MEDIUM — well-established spec, not re-verified against schema JSON in this session

**Research date:** 2026-05-10
**Valid until:** 2026-06-10 (stable domain — SARIF 2.1 is ratified; Rust/serde APIs are stable)
