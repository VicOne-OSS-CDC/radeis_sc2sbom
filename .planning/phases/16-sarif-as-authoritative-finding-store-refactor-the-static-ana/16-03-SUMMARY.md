---
phase: 16-sarif-as-authoritative-finding-store-refactor-the-static-ana
plan: "03"
subsystem: sarif
tags: [sarif, baseline, ci-gate, diff, consistency, testing, rust]

# Dependency graph
requires:
  - phase: 16-01
    provides: "suppress_lexical_false_positives, sast_findings post-suppression slice in main.rs"
  - phase: 16-02
    provides: "sarif_fingerprint, extract_baseline_fingerprints, partialFingerprints in SARIF output"
provides:
  - "--sarif-baseline CLI flag (internal-gated) in Args struct"
  - "save_diff_sarif_report function returning Result<usize> in formats::sarif"
  - "Baseline diff invocation in Console and All output arms (extract_baseline_fingerprints + save_diff_sarif_report)"
  - "'no effect' warnings for --sarif-baseline in SpdxJson, SpdxTagValue, CyclonedxJson arms"
  - "5 unit tests for save_diff_sarif_report (sarif_baseline_tests.rs)"
  - "1 integration test asserting markdown row count == SARIF results length (sarif_consistency_tests.rs)"
affects: [any-plan-consuming-sarif-output, ci-workflows, sarif-baseline-diffing]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "save_diff_sarif_report reuses save_sarif_report with explicit diff_path to avoid duplicating SARIF construction"
    - "Baseline fingerprint matching: try hash_key (sha256-based) first, then fallback tuple_key for old SARIF without partialFingerprints"
    - "std::process::exit(1) placed AFTER both writers complete to ensure SARIF is written before CI gate fires"
    - "Diff SARIF path: sarif_output when set, otherwise {project}_static_analysis_diff.sarif (never overwrites full SARIF)"

key-files:
  created:
    - tests/vulnerability_tests/sarif_baseline_tests.rs
    - tests/vulnerability_tests/sarif_consistency_tests.rs
  modified:
    - src/cli.rs
    - src/formats/sarif.rs
    - src/main.rs
    - tests/vulnerability_tests/mod.rs

key-decisions:
  - "Reuse save_sarif_report inside save_diff_sarif_report by passing diff_path as the sarif_path argument — avoids duplicating SarifLog/SarifRun/SarifTool construction"
  - "Two-key matching (hash_key AND tuple_key) in save_diff_sarif_report to handle both new SARIF (with partialFingerprints) and old SARIF (fallback format from extract_baseline_fingerprints)"
  - "std::process::exit(1) is the correct CI gate mechanism — not returning Err — because writers must complete before exit (Pitfall 6 from RESEARCH.md)"
  - "Consistency test uses '- {file_path}:{line}' bullet pattern to count finding rows, matching the actual console.rs markdown format"

patterns-established:
  - "Pattern: Baseline diff block placed AFTER save_sarif_report in both Console and All arms, gated with #[cfg(feature = 'internal')]"
  - "Pattern: 'no effect' warnings for non-SARIF format arms mirror the existing --sarif-output pattern exactly"

requirements-completed: [SARIF-05, SARIF-06]

# Metrics
duration: 25min
completed: 2026-05-11
---

# Phase 16 Plan 03: SARIF-05 CI Gate + SARIF-06 Consistency Summary

**--sarif-baseline CI gate with save_diff_sarif_report diff writer and markdown/SARIF consistency test: exits 1 on new findings, 0 on none, writes diff SARIF only when regressions detected**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-11T00:00:00Z
- **Completed:** 2026-05-11T00:25:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Added `pub sarif_baseline: Option<String>` field to Args in `src/cli.rs` (internal-gated, mirrors `--sarif-output` pattern)
- Added `save_diff_sarif_report` to `src/formats/sarif.rs`: filters new-only findings using dual hash/tuple-key matching, writes diff SARIF only when count > 0, reuses `save_sarif_report`
- Wired baseline diff block in Console and All output arms after `save_sarif_report`; exits 1 on new findings, 0 on none
- Added 'no effect' warnings in SpdxJson, SpdxTagValue, CyclonedxJson arms
- 5 unit tests in `sarif_baseline_tests.rs` covering all diff scenarios (all-in-baseline, new-count, custom path, default path, fallback tuple keys)
- 1 integration test in `sarif_consistency_tests.rs` asserting markdown bullet count equals SARIF results array length

## Task Commits

Each task was committed atomically:

1. **Task 1: Add --sarif-baseline CLI flag and save_diff_sarif_report writer** - `36a6069` (feat)
2. **Task 2: Wire baseline diff into main.rs + add tests** - `ca33dd4` (feat)

## Files Created/Modified
- `src/cli.rs` - Added `sarif_baseline: Option<String>` field after `sarif_output`, gated behind `#[cfg(feature = "internal")]`
- `src/formats/sarif.rs` - Added `save_diff_sarif_report` function with dual-key matching and diff-path resolution
- `src/main.rs` - Added baseline diff block in Console and All arms; 'no effect' warnings in non-SARIF arms
- `tests/vulnerability_tests/sarif_baseline_tests.rs` - 5 unit tests for save_diff_sarif_report
- `tests/vulnerability_tests/sarif_consistency_tests.rs` - 1 integration test for markdown/SARIF count invariant
- `tests/vulnerability_tests/mod.rs` - Registered sarif_baseline_tests and sarif_consistency_tests modules

## Decisions Made
- Reused `save_sarif_report` inside `save_diff_sarif_report` by passing the diff path as `sarif_path` argument — avoids duplicating SARIF JSON construction
- Used `std::process::exit(1)` (not `Err`) for CI gate — ensures writers complete before exit fires
- Two-key fingerprint matching ensures backward compatibility with old SARIF baselines lacking `partialFingerprints`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed consistency test row-counting strategy**
- **Found during:** Task 2 (running sarif_consistency_tests)
- **Issue:** The plan's consistency test searched for lines containing `file_path`, `line`, and `CWE-N` in the same markdown line. The actual markdown writer produces `- src/a.c:10` bullet lines (file:line) with CWE info in section headers (`#### CWE-120 (...)`), not in the same line as the file path. Count was 0 vs expected 3.
- **Fix:** Changed matching strategy to look for `- {file_path}:{line}` bullet pattern, which appears exactly once per finding in the markdown output.
- **Files modified:** `tests/vulnerability_tests/sarif_consistency_tests.rs`
- **Verification:** `cargo test --features internal sarif_consistency` returns 1 passed
- **Committed in:** `ca33dd4` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (bug in test strategy)
**Impact on plan:** Necessary fix — the test logic did not match the actual markdown format. The invariant being tested is unchanged: markdown finding count must equal SARIF results count. No scope creep.

## Issues Encountered
- Pre-existing: `test_spdx_output_passes_pyspdxtools_validation` fails due to missing `example_target_repos/rclcpp` directory in the worktree. This is unrelated to Plan 03 changes and was noted in Plan 01 SUMMARY.

## Known Stubs

None — all functionality is fully implemented and wired.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries introduced.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SARIF-05 (`--sarif-baseline`) is complete and wired end-to-end
- SARIF-06 (markdown/SARIF consistency invariant) is tested and locked in
- Plans 16-04 and 16-05 can proceed; both depend on the SARIF authoritative pipeline established in 16-01 through 16-03

## Self-Check

Files exist:
- [x] src/cli.rs
- [x] src/formats/sarif.rs
- [x] src/main.rs
- [x] tests/vulnerability_tests/sarif_baseline_tests.rs
- [x] tests/vulnerability_tests/sarif_consistency_tests.rs
- [x] tests/vulnerability_tests/mod.rs

Commits exist:
- [x] 36a6069 — feat(16-03): add --sarif-baseline CLI flag and save_diff_sarif_report writer
- [x] ca33dd4 — feat(16-03): wire --sarif-baseline into main.rs + add baseline and consistency tests

## Self-Check: PASSED

---
*Phase: 16-sarif-as-authoritative-finding-store-refactor-the-static-ana*
*Completed: 2026-05-11*
