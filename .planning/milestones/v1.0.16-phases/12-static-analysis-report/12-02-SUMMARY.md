---
phase: 12-static-analysis-report
plan: "02"
subsystem: src/formats
tags: [rust, markdown, static-analysis, sbom, cwe, formatter, feature-gated]
dependency_graph:
  requires: [12-01]
  provides: [save_static_analysis_report_impl, main_rs_call_sites]
  affects:
    - src/formats/console.rs
    - src/formats/mod.rs
    - src/main.rs
    - tests/format_tests/sast_report_tests.rs
tech_stack:
  added: []
  patterns: [BTreeMap-stable-ordering, feature-gated-pub-fn, cfg-feature-internal]
key_files:
  created: []
  modified:
    - src/formats/console.rs
    - src/formats/mod.rs
    - src/main.rs
    - tests/format_tests/sast_report_tests.rs
decisions:
  - "Bullet format is `- file:line` (no function suffix) — SastFinding has no function_name field per 12-01-SUMMARY.md"
  - "sast_findings already in scope from Phase 11 (line 177 of main.rs); no stub Vec needed"
  - "Wrapped 12-03 forward-ref tests with #[cfg(feature = sast_integration_12_03)] to fix pre-existing compile errors that blocked the 3 active tests"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-10"
  tasks_completed: 2
  files_changed: 4
---

# Phase 12 Plan 02: save_static_analysis_report Implementation Summary

Implemented `save_static_analysis_report()` in `src/formats/console.rs`, re-exported from `src/formats/mod.rs`, and wired two call sites into `src/main.rs` Console and All output arms — all gated behind `#[cfg(feature = "internal")]`.

## What Was Done

### Task 1: Implement save_static_analysis_report() in console.rs and re-export from mod.rs

Added to `src/formats/console.rs` (end of file):
- `cwe_name(cwe_id: u32) -> &'static str` — hardcoded match for 14 CWEs
- `pub fn save_static_analysis_report(project_name: &str, out_dir: &Path, findings: &[SastFinding]) -> Result<()>` — writes `{project}_static_analysis.md` with H1, blockquote disclaimer, summary table (grouped by component+CWE with BTreeMap for stable ordering), and findings section (## component / ### CWE-N subsections with `- file:line` bullets)

Added feature-gated re-export to `src/formats/mod.rs`:
```rust
#[cfg(feature = "internal")]
pub use console::save_static_analysis_report;
```

Also added feature-gated imports to `src/formats/console.rs`:
```rust
#[cfg(feature = "internal")]
use crate::vulnerability::cwe_scanner::SastFinding;
#[cfg(feature = "internal")]
use std::path::Path;
```

### Task 2: Wire call sites into main.rs

Added feature-gated import:
```rust
#[cfg(feature = "internal")]
use formats::save_static_analysis_report;
```

Added two call sites using the existing `sast_findings` vec (already in scope from Phase 11):
- Console arm: after `save_console_report` + eprintln
- All arm: after `save_console_report` + eprintln

## Final Function Signature

```rust
#[cfg(feature = "internal")]
pub fn save_static_analysis_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
) -> Result<()>
```

Function landed in `src/formats/console.rs` (not a new module).

## sast_findings Source at Call Site

Phase 11 already declared `sast_findings` in main.rs (line 177) and populates it via `run_lexical_scanner`. No stub Vec was needed — the existing variable is passed directly to both call sites.

## Test Pass/Fail Summary

| Test | Result |
|------|--------|
| test_save_static_analysis_report_with_findings | PASS |
| test_save_static_analysis_report_zero_findings | PASS |
| test_save_static_analysis_report_writes_correct_filename | PASS |
| test_save_static_analysis_report_emits_disclaimer_to_stderr | IGNORED (integration harness) |
| test_console_report_includes_sast_section_with_findings | IGNORED (12-03 wires 9th arg) |
| test_console_report_includes_sast_section_zero_findings | IGNORED (12-03 wires 9th arg) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Gated 12-03 forward-ref tests to fix compile error blocking active tests**
- **Found during:** Task 1 verification
- **Issue:** The two `#[ignore]` tests that call `save_console_report` with 9 arguments (12-03 adds the 9th param) caused compile errors even though they were marked `#[ignore]`. `#[ignore]` prevents execution but not compilation. This blocked the 3 active tests from running.
- **Fix:** Added `#[cfg(feature = "sast_integration_12_03")]` gate on both ignored tests. This non-existent feature means they compile only when explicitly enabled by plan 12-03 when it adds the trailing param. Plan 12-03 should remove the cfg gate when wiring the 9th arg.
- **Files modified:** `tests/format_tests/sast_report_tests.rs`
- **Commit:** 24b1762

## Clippy Note

`cargo clippy --features internal -- -D warnings` reports pre-existing errors in `print_sbom` (too many arguments) and `save_console_report` (too many arguments) — both functions exist from before this plan and are out of scope. No clippy errors in the new code.

## Self-Check: PASSED

- `src/formats/console.rs` (modified): FOUND
- `src/formats/mod.rs` (modified): FOUND
- `src/main.rs` (modified): FOUND
- `tests/format_tests/sast_report_tests.rs` (modified): FOUND
- Commit 24b1762 (Task 1): FOUND
- Commit e981318 (Task 2): FOUND
- Default build: 0 errors
- Internal build: 0 errors
- 3 RPT-01 tests: PASS
