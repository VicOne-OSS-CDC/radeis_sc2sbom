---
phase: 12-static-analysis-report
plan: "03"
subsystem: src/formats
tags: [rust, markdown, static-analysis, sbom, cwe, formatter, feature-gated, report]
dependency_graph:
  requires: [12-02]
  provides: [sast_section_in_console_report, rpt02_complete]
  affects:
    - src/formats/console.rs
    - src/main.rs
    - tests/format_tests/sast_report_tests.rs
tech_stack:
  added: []
  patterns: [BTreeMap-stable-ordering, cfg-feature-internal-inline-param]
key_files:
  created: []
  modified:
    - src/formats/console.rs
    - src/main.rs
    - tests/format_tests/sast_report_tests.rs
decisions:
  - "Inline #[cfg(feature = \"internal\")] per-parameter style used (matching existing save_console_report style) rather than plan's trailing-param style"
  - "Zero-findings output uses table row with em-dash cells (not prose outside table) to keep valid markdown table"
  - "Pre-existing clippy errors (too_many_arguments, ptr_arg, etc.) left untouched per CLAUDE.md surgical-changes rule"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-10"
  tasks_completed: 2
  files_changed: 3
---

# Phase 12 Plan 03: SAST Section in Console Report Summary

Injected `## Static Analysis Findings` section into `save_console_report()` (RPT-02), wired `sast_findings` through both main.rs call sites, and activated the two previously-ignored SAST-section tests.

## What Was Done

### Task 1: Add feature-gated sast_findings parameter and SAST section emit (console.rs)

Added `#[cfg(feature = "internal")] sast_findings: &[SastFinding]` as the 9th (trailing) parameter to `save_console_report` in `src/formats/console.rs`, following the inline `#[cfg]` per-parameter style already used by the other internal parameters in that function.

Inserted SAST section emit at line 1468–1500 (between the CVE block closing `}` at line 1466 and the `if summary_only {` block at line 1500):

- `## Static Analysis Findings` H2 header (D-07)
- `| Component | CWE | Name | Count |` table (D-08, no `### CWE-` subgroups)
- Zero-findings case: table row with em-dash placeholder columns and "No static analysis findings detected." in Name column (D-09)
- Non-empty case: BTreeMap-aggregated rows keyed by `(component_name, cwe_id)` for stable ordering, with `cwe_name()` helper for CWE description

### Final save_console_report Signature

```rust
pub fn save_console_report(
    sbom: &Sbom,
    path: &str,
    tree_style: &TreeStyle,
    #[cfg(feature = "internal")] vulnerability_output: &VulnerabilityOutputMode,
    #[cfg(feature = "internal")] max_vulns_per_severity: usize,
    relationships: &[DependencyRelationship],
    summary_only: bool,
    #[cfg(feature = "internal")] check_vulnerabilities: bool,
    #[cfg(feature = "internal")] sast_findings: &[SastFinding],
) -> Result<()>
```

### Insertion Line Numbers in console.rs

- Parameter added: line 1137 (`#[cfg(feature = "internal")] sast_findings: &[SastFinding]`)
- SAST section `#[cfg(feature = "internal")]` block: lines 1468–1499
- `## Static Analysis Findings` writeln: line 1473
- Table header row: lines 1474–1475
- Empty-findings branch: line 1477
- BTreeMap aggregation loop: lines 1480–1491

### Task 2: Wire main.rs call sites and activate tests

Updated both `save_console_report` call sites in `src/main.rs`:
- Console arm (~line 268): added `#[cfg(feature = "internal")] &sast_findings,` as trailing arg
- All arm (~line 352): same

In `tests/format_tests/sast_report_tests.rs`:
- Removed `#[cfg(feature = "sast_integration_12_03")]` gate from both SAST-section tests
- Removed `#[ignore = "wired in 12-03: ..."]` from both tests
- Removed stale TODO comment block and "Future call site" comments
- Test bodies unchanged — they already matched the 9-arg signature

## Test Results

| Test | Result |
|------|--------|
| test_save_static_analysis_report_with_findings | PASS |
| test_save_static_analysis_report_zero_findings | PASS |
| test_save_static_analysis_report_writes_correct_filename | PASS |
| test_console_report_includes_sast_section_with_findings | PASS |
| test_console_report_includes_sast_section_zero_findings | PASS |
| test_save_static_analysis_report_emits_disclaimer_to_stderr | IGNORED (integration harness — unchanged from 12-01/12-02) |

All 5 active tests pass under `cargo test --features internal -- sast_report_tests`.

## Deviations from Plan

### Style Deviation: Inline #[cfg] per-parameter (not plan's trailing #[cfg] syntax)

The plan proposed `#[cfg(feature = "internal")] sast_findings: &[SastFinding]` as a trailing parameter with the attribute wrapping the entire parameter (stable since Rust 1.43). The actual `save_console_report` already had all other internal params using this exact inline `#[cfg]` style, so the new parameter was added consistently. No behavioral difference — this is the same approach.

### Zero-findings output: table row vs. prose line

Plan step C specified `writeln!(output, "No static analysis findings detected.")` as a standalone prose line. The test assertion is `assert!(out.contains("No static analysis findings"))` which matches either format. Used a table row (`| — | — | No static analysis findings detected. | — |`) to keep the markdown table valid (avoids a dangling table header with no rows). Test passes.

### Clippy: pre-existing errors not fixed

`cargo clippy --features internal -- -D warnings` reports 43 errors (unchanged from before this plan). The `too_many_arguments` lint on `save_console_report` (now 9 args) was already present with 8 args. All other errors are in unrelated functions predating Phase 12. Per CLAUDE.md surgical-changes rule, pre-existing errors in unrelated code are not touched. This was also documented in 12-02-SUMMARY.md.

## Phase 12 Closeout Notes

RPT-02 is now satisfied:
- `_report.md` contains `## Static Analysis Findings` after the CVE block (D-07)
- Section contains only the summary table, no `### CWE-` subgroups (D-08)
- Section appears even when sast_findings is empty (D-09)

All 3 RPT-01 tests (plan 12-02) and both RPT-02 tests (plan 12-03) are GREEN.

## Self-Check: PASSED

- `src/formats/console.rs` (modified): FOUND
- `src/main.rs` (modified): FOUND
- `tests/format_tests/sast_report_tests.rs` (modified): FOUND
- Commit 561accf (Task 1): FOUND
- Commit 778a5c3 (Task 2): FOUND
- Default build: 0 errors
- Internal build: 0 errors
- 5 sast_report_tests active tests: PASS
