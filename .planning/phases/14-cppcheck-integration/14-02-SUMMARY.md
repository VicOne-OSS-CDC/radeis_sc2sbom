---
phase: 14-cppcheck-integration
plan: "02"
subsystem: vulnerability/cwe_scanner
tags: [xml-parsing, cppcheck, sast, unit-tests, wave-1]
dependency_graph:
  requires: [14-01]
  provides: [parse_cppcheck_xml, CPPCHECK_CWE_OVERRIDES]
  affects: [plans 14-03, 14-04, 14-05]
tech_stack:
  added: [quick_xml event-loop for &[u8] input]
  patterns: [pure-function XML parser, static fallback lookup table, silent-drop for unresolvable CWEs]
key_files:
  created:
    - tests/vulnerability_tests/cppcheck_scanner_tests.rs
  modified:
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - tests/vulnerability_tests/mod.rs
decisions:
  - "parse_cppcheck_xml promoted from pub(crate) to pub to allow integration tests in tests/ crate to call it directly"
  - "Reader::from_reader(&[u8]) used for byte-slice input; trim_text(true) used per quick-xml 0.30 API (not config_mut)"
  - "Break-on-Err loop exit: malformed XML returns partial findings without panic, satisfying T-14-03"
metrics:
  duration: ~12 minutes
  completed: "2026-05-10"
  tasks_completed: 2
  files_modified: 4
---

# Phase 14 Plan 02: cppcheck XML Parser and Override Table Summary

**One-liner:** `parse_cppcheck_xml` parses cppcheck XML v2 stderr bytes into `SastFinding` entries using CWE attribute pass-through, 15-entry static override table, and silent drop for unresolvable IDs.

## What Was Done

### Task 1: CPPCHECK_CWE_OVERRIDES table and parse_cppcheck_xml function

Added to `src/vulnerability/cwe_scanner.rs`:

- `use quick_xml::events::Event` and `use quick_xml::Reader` imports
- `CPPCHECK_CWE_OVERRIDES: &[(&str, u32)]` — 15-entry static table mapping cppcheck error IDs that lack `cwe` attributes in older releases to canonical CWE numbers (uninitvar→457, nullPointer→476, memleak→401, etc.)
- `pub fn parse_cppcheck_xml(xml_bytes: &[u8], component_name: &str, component_ecosystem: &str) -> Vec<SastFinding>` — event-loop parser implementing:
  - D-01: CWE from `cwe` attribute on `<error>`, else CPPCHECK_CWE_OVERRIDES lookup by `id`
  - D-04: silent drop when no CWE resolvable
  - First-location-only rule: `location_taken` flag prevents second `<location>` from emitting
  - T-14-03: `Err(_) => break` — malformed XML returns partial results without panic
  - All findings carry `source: SastSource::Cppcheck`

Updated `src/vulnerability/mod.rs`: added `parse_cppcheck_xml` to the `pub use cwe_scanner::{...}` re-export under `#[cfg(feature = "internal")]`.

### Task 2: Integration tests in cppcheck_scanner_tests.rs

Created `tests/vulnerability_tests/cppcheck_scanner_tests.rs` with 6 tests:

| Test | Behavior verified |
|------|-------------------|
| `parses_cwe_attr_and_emits_cppcheck_source` | CWE attr pass-through; `source == SastSource::Cppcheck`; component fields propagated |
| `override_table_resolves_uninitvar_to_cwe_457` | Override table fallback when cwe attr absent |
| `unresolved_cwe_is_silently_dropped` | Unknown ID with no cwe attr → empty Vec (D-04) |
| `first_location_only_when_multiple_locations` | Two `<location>` children → exactly one finding from first location |
| `empty_bytes_returns_empty_vec` | `b""` input → empty Vec, no panic |
| `malformed_xml_does_not_panic` | `b"<<not really xml<<"` → no panic (T-14-03) |

Updated `tests/vulnerability_tests/mod.rs`: added `#[cfg(feature = "internal")] mod cppcheck_scanner_tests;`.

## Verification Results

- `cargo build --features internal`: exits 0 (pre-existing dead-code warnings on Cppcheck/Both variants; expected until Plan 04)
- `cargo test --features internal cppcheck`: 6 passed, 0 failed
- `cargo test --features internal` full suite: 333 passed, 1 pre-existing failure (`test_spdx_output_passes_pyspdxtools_validation` — requires `pyspdxtools` CLI absent in worktree; confirmed pre-existing from Plan 01)

## Deviations from Plan

### Auto-adjusted Issues

**1. [Rule 1 - API mismatch] Used trim_text(true) instead of config_mut().trim_text(true)**
- **Found during:** Task 1 implementation review
- **Issue:** Plan's reference implementation used `reader.config_mut().trim_text(true)` which is the quick-xml 0.37+ API. quick-xml 0.30 uses `reader.trim_text(true)` directly on the Reader struct.
- **Fix:** Used `reader.trim_text(true)` matching the existing code in `src/parsers/ros.rs`.
- **Files modified:** `src/vulnerability/cwe_scanner.rs`
- **Commit:** 6187219

**2. [Per plan spec] parse_cppcheck_xml promoted to pub**
- The plan explicitly directs promoting from `pub(crate)` to `pub` (step 1 in Task 2 action). Applied as specified.

## Known Stubs

None. `parse_cppcheck_xml` is a pure function returning populated `SastFinding` structs. No stub values or placeholder data.

## Threat Flags

None. `parse_cppcheck_xml` operates on in-process byte slices only. File paths from the XML `file` attribute are stored as strings with no filesystem operations (T-14-04: accepted per plan threat model). Malformed XML handled via break-on-error (T-14-03: mitigated).

## Self-Check

- `src/vulnerability/cwe_scanner.rs`: exists with `pub fn parse_cppcheck_xml`, `CPPCHECK_CWE_OVERRIDES` (15 entries), `use quick_xml`, `source: SastSource::Cppcheck`
- `src/vulnerability/mod.rs`: re-exports `parse_cppcheck_xml` under `#[cfg(feature = "internal")]`
- `tests/vulnerability_tests/cppcheck_scanner_tests.rs`: 6 `#[test]` functions, all calling `parse_cppcheck_xml`
- `tests/vulnerability_tests/mod.rs`: contains `mod cppcheck_scanner_tests`
- Commits: 6187219 (Task 1), 17c7203 (Task 2)

## Self-Check: PASSED
