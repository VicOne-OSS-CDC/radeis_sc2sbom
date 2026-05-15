---
phase: 14-cppcheck-integration
plan: "05"
subsystem: vulnerability/cwe_scanner
tags: [cppcheck, sast, deduplication, SastSource::Both, main-rs-wiring, wave-3]
dependency_graph:
  requires:
    - phase: 14-03
      provides: args.cppcheck_path field on Args struct
    - phase: 14-04
      provides: run_cppcheck_scanner function
  provides:
    - deduplicate_sast_findings public helper with D-11/D-12 semantics
    - main.rs wired to run lexical -> cppcheck -> dedup in sequence (D-06)
    - sast_findings reaching formatters is the merged, deduplicated vector
  affects: [phase-15-SARIF-output]
tech-stack:
  added: []
  patterns: [HashMap-dedup-with-canonicalize-fallback, SastSource::Both-promotion-on-collision]
key-files:
  created: []
  modified:
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - src/main.rs
    - tests/vulnerability_tests/cppcheck_scanner_tests.rs
key-decisions:
  - "deduplicate_sast_findings placed as free function in cwe_scanner.rs (not main.rs) so both main.rs and the test crate can call it"
  - "Lexical entry's fields are preserved as base on collision (D-12) — lexical scanner has richer component attribution"
  - "Path::canonicalize with unwrap_or_else fallback for dedup keys (T-14-12 mitigated per threat model)"
  - "Fully-qualified paths (std::ffi::OsStr) used in main.rs change to keep diff localized — no new top-level use imports"
patterns-established:
  - "Dedup pattern: canonicalize-or-fallback-to-raw-string for HashMap key, then mutate source field on collision"
requirements-completed: [CPPCHECK-01, CPPCHECK-05]
duration: ~15min
completed: "2026-05-10"
---

# Phase 14 Plan 05: Pipeline Wiring and Deduplication Summary

**Pipeline wiring complete: main.rs now calls run_lexical_scanner -> run_cppcheck_scanner -> deduplicate_sast_findings in sequence; dual-detected findings tagged SastSource::Both; all formatters receive the merged, deduplicated sast_findings vector**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-10
- **Completed:** 2026-05-10
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `pub fn deduplicate_sast_findings(lexical, cppcheck) -> Vec<SastFinding>` to `cwe_scanner.rs` implementing D-11 (dedup by `(canonical_file_path, line, cwe_id)`) and D-12 (promote to `SastSource::Both` on collision, preserving lexical entry as base)
- Extended `mod.rs` re-export to include `deduplicate_sast_findings`
- Added 3 dedup behavioral tests: unique-pass-through, collision-promotes-to-Both, same-file-line-distinct-CWE-kept-separate
- Replaced single `sast_findings = run_lexical_scanner(...)` in `main.rs` with the lexical+cppcheck+dedup sequence; `args.cppcheck_path` forwarded as `Option<&OsStr>`
- Phase 14 is feature-complete: CPPCHECK-01 (findings reach output pipeline) and CPPCHECK-05 (no duplicate `(file,line,cwe)` entries) are satisfied

## Task Commits

1. **Task 1: Add deduplicate_sast_findings helper and dedup tests** - `d66c28d` (feat)
2. **Task 2: Wire run_cppcheck_scanner + dedup into main.rs** - `59af977` (feat)

**Plan metadata:** `(committed with SUMMARY.md)` (docs: complete plan)

## Files Created/Modified

- `src/vulnerability/cwe_scanner.rs` - Added `pub fn deduplicate_sast_findings` (~45 lines) after `run_cppcheck_scanner`
- `src/vulnerability/mod.rs` - Extended `pub use cwe_scanner::{...}` to include `deduplicate_sast_findings`
- `src/main.rs` - Replaced 3-line lexical scanner call with 16-line lexical+cppcheck+dedup block
- `tests/vulnerability_tests/cppcheck_scanner_tests.rs` - Extended import; added `lex`/`cpp` helpers and 3 test functions

## Decisions Made

- `deduplicate_sast_findings` is a free function in `cwe_scanner.rs` rather than a closure in `main.rs` — keeps dedup logic unit-testable and co-located with the scanner types
- Lexical entry is preserved as base on collision (D-12) because lexical findings carry direct `(name, ecosystem)` component attribution from the `component_dirs` map key, whereas cppcheck findings derive attribution from the parent component loop variable
- `Path::canonicalize` with `unwrap_or_else(|_| PathBuf::from(...))` fallback for all dedup keys — matches existing pattern in `parsers/npm.rs:228` and `parsers/ros.rs:39`; T-14-12 mitigated
- No new top-level `use` imports added to `main.rs` — `std::ffi::OsStr` used fully-qualified to keep diff localized

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None. All three scanner stages (lexical, cppcheck, dedup) are fully wired. The `source` field on `SastFinding` is populated but not yet serialized in any output format — this is intentional per D-14 (metadata-only until Phase 15 SARIF).

## Threat Flags

None beyond what the plan's threat model documents:
- T-14-12 (Tampering): `canonicalize` with fallback — mitigated as planned
- T-14-13 (Info disclosure): `SastSource::Both` not serialized — accepted per plan
- T-14-14 (Repudiation): lexical entry as base on collision — accepted/intentional per plan
- T-14-15 (DoS): O(n) HashMap dedup — accepted per plan

## Verification Results

- `cargo build --features internal`: exits 0 (pre-existing dead-code warnings; expected until Phase 15 serializes `source`)
- `cargo build` (no internal): exits 0 — cfg gate intact
- `cargo test --features internal cppcheck`: 11 passed (6 from Plan 02 + 2 from Plan 04 + 3 new dedup tests)
- `cargo test --features internal` full suite: 338 passed, 1 pre-existing failure (`test_spdx_output_passes_pyspdxtools_validation` — requires `pyspdxtools` CLI absent in worktree; unrelated to this plan)

## Next Phase Readiness

- Phase 14 (cppcheck integration) is fully complete: CPPCHECK-01..05 all satisfied across plans 01-05
- Phase 15 (SARIF output) can now consume `sast_findings` (merged, deduplicated) and serialize `source` field
- No blockers

## Self-Check

- `src/vulnerability/cwe_scanner.rs`: `pub fn deduplicate_sast_findings` exists (1 occurrence) — FOUND
- `src/vulnerability/cwe_scanner.rs`: `SastSource::Both` appears at least 2 times (enum variant + dedup mutation) — FOUND
- `src/vulnerability/mod.rs`: `deduplicate_sast_findings` in re-export — FOUND
- `src/main.rs`: `run_cppcheck_scanner` present (1 occurrence) — FOUND
- `src/main.rs`: `deduplicate_sast_findings` present (1 occurrence) — FOUND
- `src/main.rs`: `cppcheck_path` present (1 occurrence) — FOUND
- `src/main.rs`: `let lexical_findings` present (1 occurrence) — FOUND
- Commits: d66c28d (Task 1), 59af977 (Task 2) — FOUND

## Self-Check: PASSED

---
*Phase: 14-cppcheck-integration*
*Completed: 2026-05-10*
