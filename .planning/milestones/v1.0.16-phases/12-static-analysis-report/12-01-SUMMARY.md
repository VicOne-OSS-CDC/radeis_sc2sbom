---
phase: 12-static-analysis-report
plan: "01"
subsystem: tests/format_tests
tags: [rust, testing, markdown, static-analysis, sbom]
dependency_graph:
  requires: []
  provides: [sast_report_tests_scaffold]
  affects: [tests/format_tests/mod.rs, tests/format_tests/sast_report_tests.rs]
tech_stack:
  added: []
  patterns: [TDD-RED, feature-gated-tests, tempfile-tempdir]
key_files:
  created:
    - tests/format_tests/sast_report_tests.rs
  modified:
    - tests/format_tests/mod.rs
decisions:
  - "SastFinding has no function_name field (Phase 11 omitted it); bullet format will be `- file:line` not `- file:line — function`"
  - "Ignored tests use #[ignore] with forward-reference comments so 12-03 can wire them"
  - "stderr disclaimer test stubbed as #[ignore] per plan instructions"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-10"
  tasks_completed: 2
  files_changed: 2
---

# Phase 12 Plan 01: SAST Report Test Scaffolding Summary

RED-phase test file for `save_static_analysis_report` and the SAST section in `save_console_report`, encoding all D-01..D-11 decisions.

## What Was Done

Created `tests/format_tests/sast_report_tests.rs` with 6 test functions (3 active + 2 ignored for 12-03 + 1 ignored for integration harness). Registered the module in `tests/format_tests/mod.rs` under `#[cfg(feature = "internal")]`.

## Confirmed SastFinding Field Names

Read from `src/vulnerability/cwe_scanner.rs` (Phase 11 output):

```rust
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,
    pub line: u32,
}
```

**Deviation from RESEARCH.md A1 expectation:** `function_name: String` field was NOT implemented in Phase 11. The test helper `make_finding()` has no `func` parameter. The finding bullet format asserts `- file:line` (without function suffix). This is documented in a file-level comment in `sast_report_tests.rs`.

## The 6 Test Functions and Requirements Coverage

| # | Function | Coverage | Active? |
|---|----------|----------|---------|
| 1 | `test_save_static_analysis_report_with_findings` | RPT-01: D-04, D-05, D-11 (H1, blockquote, summary table, ## component, ### CWE-) | Active (RED) |
| 2 | `test_save_static_analysis_report_zero_findings` | RPT-01: D-02, D-03, D-06, D-11 (file always written, note row, prose) | Active (RED) |
| 3 | `test_save_static_analysis_report_writes_correct_filename` | RPT-01: D-01 (filename convention) | Active (RED) |
| 4 | `test_console_report_includes_sast_section_with_findings` | RPT-02: D-07, D-08 (section header, summary table, no ### CWE- in _report.md) | Ignored (12-03) |
| 5 | `test_console_report_includes_sast_section_zero_findings` | RPT-02: D-09 (section present with zero findings) | Ignored (12-03) |
| 6 | `test_save_static_analysis_report_emits_disclaimer_to_stderr` | RPT-03: D-10 (stderr disclaimer) | Ignored (integration harness) |

## Intentional RED-State Compile Errors

When built with `--features internal`, the following errors are expected and intentional:

1. `error[E0432]: unresolved import 'radeis_sc2sbom::formats::save_static_analysis_report'`
   - **Resolved by:** Plan 12-02 (implements the formatter function)

2. `error[E0061]: this function takes 8 arguments but 9 arguments were supplied` (x2, for ignored tests)
   - **Resolved by:** Plan 12-03 (adds trailing `&[SastFinding]` param to `save_console_report`)

Default build (`cargo build --tests`, no `--features internal`) compiles cleanly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed incorrect VulnerabilityOutputMode variant and import paths in ignored tests**
- **Found during:** Task 2
- **Issue:** Plan template used `VulnerabilityOutputMode::Severity` (does not exist) and imported `TreeStyle` from `formats::console` (private). Actual variant is `::Summary`; type is in `radeis_sc2sbom::cli::TreeStyle`.
- **Fix:** Used correct variant `VulnerabilityOutputMode::Summary` and imported both types from `radeis_sc2sbom::cli`.
- **Files modified:** `tests/format_tests/sast_report_tests.rs`
- **Commit:** 65de8b6

## Self-Check: PASSED

- `tests/format_tests/sast_report_tests.rs`: FOUND
- `tests/format_tests/mod.rs` (modified): FOUND
- Commit `65de8b6`: FOUND
- Default build: clean (no errors)
- Internal build: RED errors only (save_static_analysis_report not found + arg count)
