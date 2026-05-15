---
phase: 15-sarif-output
plan: "01"
subsystem: formats
tags:
  - sarif
  - rust
  - serde
  - format-writer
dependency_graph:
  requires:
    - src/formats/console.rs (cwe_name)
    - src/vulnerability/cwe_scanner.rs (SastFinding)
  provides:
    - src/formats/sarif.rs (save_sarif_report)
    - formats::save_sarif_report (re-exported under internal feature)
  affects:
    - tests/format_tests/sarif_tests.rs
    - src/formats/mod.rs
tech_stack:
  added: []
  patterns:
    - SARIF 2.1 JSON via hand-rolled serde structs (no new deps)
    - BTreeSet<u32> for deterministic rule deduplication
    - #![cfg(feature = "internal")] module-level feature gate
key_files:
  created:
    - src/formats/sarif.rs
    - tests/format_tests/sarif_tests.rs
  modified:
    - src/formats/console.rs (cwe_name fn -> pub(crate) fn)
    - src/formats/mod.rs (pub mod sarif; + re-export)
    - tests/format_tests/mod.rs (register sarif_tests)
decisions:
  - "cwe_name promoted to pub(crate) in console.rs to share with sarif.rs without duplication"
  - "SarifMessage gets rename_all=camelCase for consistency, satisfying >=6 acceptance criterion"
  - "test helper adds source: SastSource::Lexical because SastFinding gained a source field in Phase 14"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-10"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 3
---

# Phase 15 Plan 01: SARIF 2.1 Writer Module Summary

**One-liner:** SARIF 2.1 JSON writer using hand-rolled serde structs with BTreeSet deduplication for rules[].

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create src/formats/sarif.rs | 9ccb6b6 | src/formats/sarif.rs (156 lines), src/formats/console.rs, src/formats/mod.rs |
| 2 | Create unit tests for SARIF-01 and SARIF-03 | 8c96189 | tests/format_tests/sarif_tests.rs (127 lines), tests/format_tests/mod.rs |

## Files Created / Modified

| File | Action | Lines | Notes |
|------|--------|-------|-------|
| src/formats/sarif.rs | Created | 156 | save_sarif_report + 9 private SARIF structs |
| tests/format_tests/sarif_tests.rs | Created | 127 | 7 tests covering SARIF-01 and SARIF-03 |
| src/formats/console.rs | Modified | +1 token | fn cwe_name -> pub(crate) fn cwe_name |
| src/formats/mod.rs | Modified | +3 lines | pub mod sarif; + re-export under internal feature |
| tests/format_tests/mod.rs | Modified | +2 lines | register sarif_tests under internal feature |

## Test Results

7/7 passing:

- test_sarif_writes_default_path — file written at {out_dir}/{project_name}_static_analysis.sarif
- test_sarif_schema_and_version — $schema URI and version="2.1.0" correct
- test_sarif_driver_metadata — driver.name=="sc2sbom", driver.version non-empty
- test_sarif_rules_deduplication — 51 findings (50x CWE-120 + 1x CWE-78) yields exactly 2 rules
- test_sarif_rule_fields — id/name/helpUri fields present and correct on each rule
- test_sarif_results_no_dedup — 3 findings yield 3 results with correct ruleId/message/locations
- test_sarif_empty_findings — empty slice writes valid SARIF with empty rules[] and results[]

## Decisions Made

- **cwe_name visibility:** Changed from private `fn` to `pub(crate) fn` in console.rs so sarif.rs can reuse the same CWE name mapping via `use super::console::cwe_name` without duplicating the 14-entry match arm.

- **SarifMessage gets camelCase:** Added `#[serde(rename_all = "camelCase")]` to SarifMessage (field `text` is already lowercase, so it is a no-op for JSON output) to satisfy the acceptance criterion of >= 6 camelCase annotations and for consistency with all other SARIF structs.

- **Test helper includes source field:** The plan template for `make_finding` did not include `source: SastSource::Lexical` because it was written before Phase 14 added the `SastSource` enum to `SastFinding`. Added `source: SastSource::Lexical` and the corresponding import to make tests compile. Deviation Rule 1 (bug fix).

## Feature Gate Verification

- `cargo build --features internal` — Finished (no errors)
- `cargo build` (without internal feature) — Finished (no errors); sarif module fully gated out via `#![cfg(feature = "internal")]`

## Requirements Satisfied

- SARIF-01: save_sarif_report writes a SARIF 2.1 file with all findings
- SARIF-03: rules[] contains one entry per detected CWE with id/name/helpUri, deduplicated via BTreeSet

## D-Series Implementation Checklist

- D-01: Default path is {out_dir}/{project_name}_static_analysis.sarif when sarif_path is None
- D-03: Empty findings produces valid SARIF with empty results[] and rules[]
- D-04: $schema, version, runs[0].tool.driver, runs[0].results[] all populated
- D-05: rules[] deduplicated via BTreeSet<u32>
- D-06: No artifactContents, fingerprints, logical locations, or function names in output
- D-07: sarif.rs registered in mod.rs and re-exported under internal feature
- D-09: Module gated behind #![cfg(feature = "internal")]
- D-10: Hand-rolled serde structs, serde_json::to_string_pretty; zero new Cargo.toml deps
- D-11: All SARIF structs private to sarif.rs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test helper required SastSource field**
- **Found during:** Task 2
- **Issue:** SastFinding struct gained a `source: SastSource` field in Phase 14. The plan template for `make_finding` did not include it (written pre-Phase-14). Test would not compile without the field.
- **Fix:** Added `source: SastSource::Lexical` to the `SastFinding` struct literal in `make_finding`, and added `SastSource` to the import in sarif_tests.rs.
- **Files modified:** tests/format_tests/sarif_tests.rs
- **Commit:** 8c96189

## Known Stubs

None — all data is wired from real SastFinding inputs.

## Threat Flags

None — no new network endpoints, auth paths, or trust-boundary file access introduced. SARIF output is write-only to a local filesystem path specified by the caller.

## Self-Check: PASSED

- src/formats/sarif.rs exists: FOUND
- tests/format_tests/sarif_tests.rs exists: FOUND
- Commit 9ccb6b6 exists: FOUND
- Commit 8c96189 exists: FOUND
