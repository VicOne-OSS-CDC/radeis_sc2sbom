---
phase: 14-cppcheck-integration
plan: "01"
subsystem: vulnerability/cwe_scanner
tags: [struct-extension, sast, source-enum, wave-0]
dependency_graph:
  requires: []
  provides: [SastSource enum, SastFinding.source field]
  affects: [plans 14-02, 14-03, 14-04, 14-05]
tech_stack:
  added: []
  patterns: [enum discriminator on struct, pub re-export]
key_files:
  created: []
  modified:
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - tests/cyclonedx_sast_tests.rs
    - tests/format_tests/sast_report_tests.rs
decisions:
  - SastSource declared without #[non_exhaustive] per plan spec to keep later match arms simple
metrics:
  duration: ~8 minutes
  completed: "2026-05-10"
  tasks_completed: 2
  files_modified: 4
---

# Phase 14 Plan 01: SastSource Enum and SastFinding Source Field Summary

**One-liner:** Added `SastSource { Lexical, Cppcheck, Both }` enum and `source` field to `SastFinding` struct, updating all construction sites so the codebase compiles for Wave 1.

## What Was Done

This plan implements D-13 from the Phase 14 context: extending `SastFinding` with a `source: SastSource` discriminator field that identifies which scanner produced each finding. This is a pure structural change — no behavioral change to scanning logic.

### Task 1: SastSource enum + source field in cwe_scanner.rs

Added `SastSource { Lexical, Cppcheck, Both }` enum immediately above the `SastFinding` struct definition. Added `pub source: SastSource` as the final field of `SastFinding`. Updated both `findings.push(SastFinding { ... })` construction sites in `scan_file` (CWE rule loop + CWE-369 div-by-zero path) to include `source: SastSource::Lexical`. Extended the `mod.rs` re-export to include `SastSource`.

### Task 2: Test-side SastFinding literals updated

- `tests/cyclonedx_sast_tests.rs`: Updated import from `SastFinding` to `{SastFinding, SastSource}`. Added `source: SastSource::Lexical` to both struct literals (test_sast_vulnerability_in_output at line 33, test_unmatched_sast_finding_dropped at line 95).
- `tests/format_tests/sast_report_tests.rs`: Updated import to include `SastSource`. Added `source: SastSource::Lexical` to the `make_finding` helper struct literal.
- `tests/vulnerability_tests/cwe_scanner_tests.rs`: Confirmed zero `SastFinding {` literals — no changes needed.

## Verification Results

- `cargo build --features internal`: exits 0 (6 dead-code warnings expected — variants/field unused until later plans)
- `cargo build --features internal --tests`: exits 0
- `cargo test --features internal`: 327 passed, 1 pre-existing failure (spdx_validation_tests — requires `pyspdxtools` CLI and `example_target_repos/rclcpp` which are absent in this worktree environment; confirmed pre-existing)
- All cyclonedx_sast_tests (3/3): ok
- All sast_report_tests (5/5): ok
- All cwe_scanner unit tests: ok

## Deviations from Plan

None — plan executed exactly as written. The second CWE-369 construction site in `scan_file` was also updated (the plan showed the rule-loop site; the div-by-zero site is structurally identical and also required the new field per correct compilation).

## Known Stubs

None. This plan adds a field to an existing struct; no stub values or placeholders introduced.

## Threat Flags

None. This is a purely internal structural change with no new trust boundaries, network endpoints, or external input paths.

## Self-Check

- `src/vulnerability/cwe_scanner.rs`: exists with `pub enum SastSource`, `pub source: SastSource` field, 2x `source: SastSource::Lexical` construction
- `src/vulnerability/mod.rs`: re-exports `SastSource`
- `tests/cyclonedx_sast_tests.rs`: 2x `source: SastSource::Lexical`, 3x `SastSource` references
- `tests/format_tests/sast_report_tests.rs`: 1x `source: SastSource::Lexical`, 2x `SastSource` references
- Commits: ef4a023 (Task 1), 2270abd (Task 2)
