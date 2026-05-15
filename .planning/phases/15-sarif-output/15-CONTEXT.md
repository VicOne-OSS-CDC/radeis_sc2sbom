# Phase 15: sarif-output - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Write all `SastFinding` entries (from lexical scanner + cppcheck) to a SARIF 2.1 JSON file alongside the existing `_static_analysis.md` report. Add a `--sarif-output` CLI flag for custom path override.

</domain>

<decisions>
## Implementation Decisions

### SARIF File Location
- **D-01:** Default SARIF file path is `{out_dir}/{project_name}_static_analysis.sarif` — mirrors the `.md` file exactly, consistent with all other output files.
- **D-02:** `--sarif-output <PATH>` accepts any absolute or relative file path. It is fully independent of `--output`; the two flags are orthogonal.
- **D-03:** The SARIF file is always written, even when findings is empty — write a valid SARIF file with empty `results[]` and `rules[]` arrays so CI pipelines always find the artifact.

### SARIF Schema
- **D-04:** Populate: `$schema`, `version`, `runs[0].tool.driver` (name, version, `rules[]`), `runs[0].results[]` each with `ruleId`, `message.text`, and `locations[].physicalLocation` (`artifactLocation.uri` + `region.startLine`).
- **D-05:** `rules[]` entry per detected CWE: `id` = `"CWE-{N}"`, `name` = CWE name string (from existing `cwe_name()`), `helpUri` = `"https://cwe.mitre.org/data/definitions/{N}.html"` (SARIF-03).
- **D-06:** Do NOT add artifactContents, fingerprints, logical locations, or function names — not available from lexical `SastFinding` data.

### Module Placement
- **D-07:** New `src/formats/sarif.rs` — mirrors `console.rs` structure. Exports `save_sarif_report(project_name: &str, out_dir: &Path, findings: &[SastFinding], sarif_path: Option<&str>)`. Added to `src/formats/mod.rs` via `pub use sarif::save_sarif_report`.
- **D-08:** `main.rs` calls `save_sarif_report` immediately after each `save_static_analysis_report` call (lines 286 and 370). Same findings slice passed to both.
- **D-09:** Both `save_static_analysis_report` and `save_sarif_report` remain gated behind `#[cfg(feature = "internal")]`.

### Dependency Approach
- **D-10:** Hand-rolled SARIF Rust structs with `#[derive(Serialize)]` in `src/formats/sarif.rs` itself. Use `serde_json::to_string_pretty`. Zero new dependencies — `serde_json` is already in `Cargo.toml`.
- **D-11:** SARIF structs (`SarifLog`, `SarifRun`, `SarifTool`, `SarifDriver`, `SarifRule`, `SarifResult`, `SarifLocation`, `SarifPhysicalLocation`, `SarifArtifactLocation`, `SarifRegion`) are private to `sarif.rs`. No separate models file.

### Claude's Discretion
- `tool.driver.name` value (e.g. `"sc2sbom"` or `"radeis_sc2sbom"`) — Claude picks the consistent name.
- `tool.driver.version` — Claude reads from `Cargo.toml` or hardcodes the current version string consistently with how the rest of the tool reports it.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §SARIF Output — SARIF-01, SARIF-02, SARIF-03 (the three requirements for this phase)

### Existing Implementation — SAST pipeline
- `src/vulnerability/cwe_scanner.rs` — `SastFinding` struct definition (cwe_id, component_name, component_ecosystem, file_path, line) and `CWE_RULES` table
- `src/formats/console.rs:1939` — `cwe_name(cwe_id)` function: maps all 14 CWE IDs to name strings; REUSE this in sarif.rs
- `src/formats/console.rs:1960` — `save_static_analysis_report` — direct analog for the new `save_sarif_report`
- `src/formats/mod.rs` — where `pub use console::save_static_analysis_report` lives; add `pub use sarif::save_sarif_report` here

### Call sites in main.rs
- `src/main.rs:286` — first call to `save_static_analysis_report`; add `save_sarif_report` call immediately after
- `src/main.rs:370` — second call to `save_static_analysis_report`; add `save_sarif_report` call immediately after

### CLI
- `src/cli.rs` — where `--output` and all other CLI flags are defined; add `--sarif-output` here

### SARIF 2.1 schema reference
- No external spec file — SARIF 2.1 schema is at `https://docs.oasis-open.org/sarif/sarif/v2.1.0/` but the required fields are fully specified in D-04 through D-06 above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cwe_name(cwe_id: u32) -> &'static str` in `src/formats/console.rs:1939`: reuse directly in sarif.rs for both `rules[].name` and `results[].message.text`
- `serde_json` already in `Cargo.toml` — no new dep needed

### Established Patterns
- All format writers follow: `pub fn save_X(project_name: &str, out_dir: &Path, ...) -> Result<()>` — sarif.rs must match this signature shape
- Feature gate: `#[cfg(feature = "internal")]` on the public function, same as `save_static_analysis_report`
- Output file naming: `{project_name}_{type}.{ext}` — SARIF file is `{project_name}_static_analysis.sarif`

### Integration Points
- `src/formats/mod.rs`: add `mod sarif;` and `pub use sarif::save_sarif_report`
- `src/main.rs`: pass `args.sarif_output.as_deref()` to `save_sarif_report` (the flag value, or None to use default path)

</code_context>

<specifics>
## Specific Ideas

- SARIF `$schema` value: `"https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"`
- `runs[0].tool.driver.rules[]` must deduplicate by CWE — if 50 findings all have CWE-120, there's one rule entry for CWE-120
- `results[]` entry per SastFinding (no deduplication at result level — each file:line is a distinct result)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 15-sarif-output*
*Context gathered: 2026-05-10*
