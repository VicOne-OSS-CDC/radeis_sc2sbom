---
phase: 16-sarif-as-authoritative-finding-store-refactor-the-static-ana
plan: 01
subsystem: vulnerability-scanner
tags: [cppcheck, sast, sarif, suppression, cwe, rust]

# Dependency graph
requires:
  - phase: 14-cppcheck-integration
    provides: "run_cppcheck_scanner, deduplicate_sast_findings, SastSource enum, SastFinding struct"
  - phase: 15-sarif-output
    provides: "save_sarif_report wired into main.rs output pipeline"
provides:
  - "CPPCHECK_COVERED_CWES const (12 CWEs cppcheck covers reliably)"
  - "suppress_lexical_false_positives function exported from vulnerability mod"
  - "run_cppcheck_scanner returns (Vec<SastFinding>, BTreeSet<PathBuf>) tuple"
  - "main.rs: suppression call between dedup and writers in both Console and All arms"
  - "8 unit tests verifying all SARIF-07 suppression invariants"
affects: [16-02, 16-03, 16-04, 16-05, sarif-baseline, any plan consuming sast_findings]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "suppress_lexical_false_positives: filter lexical findings when cppcheck covered the CWE and did not confirm the site"
    - "build cppcheck_confirmed set from POST-DEDUP slice (not from raw cppcheck_findings)"
    - "scanned_dirs inserted only on successful cppcheck invocations (exit 0 or 1)"

key-files:
  created:
    - tests/vulnerability_tests/suppression_tests.rs
  modified:
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - src/main.rs
    - tests/vulnerability_tests/mod.rs
    - tests/vulnerability_tests/cppcheck_scanner_tests.rs

key-decisions:
  - "Build cppcheck_confirmed from post-dedup slice, not raw cppcheck_findings — paths must match normalized form"
  - "scanned_dirs.insert only in Ok(out) success path — do not count skipped components as scanned"
  - "CPPCHECK_COVERED_CWES contains 12 CWEs that cppcheck's --enable=warning,style,security covers reliably"

patterns-established:
  - "Pattern: run_cppcheck_scanner returns tuple (findings, scanned_dirs) for downstream suppression scope check"
  - "Pattern: suppress_lexical_false_positives called between deduplicate_sast_findings and all writers"

requirements-completed: [SARIF-07]

# Metrics
duration: 20min
completed: 2026-05-10
---

# Phase 16 Plan 01: SARIF-07 Suppression Summary

**cppcheck-scope suppression for lexical false positives: suppress_lexical_false_positives drops Lexical findings when cppcheck covered the CWE and ran on the component dir without confirming the site**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-05-10T17:00:00Z
- **Completed:** 2026-05-10T17:19:48Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added `CPPCHECK_COVERED_CWES` const (12 CWEs: 78, 120, 122, 134, 190, 242, 327, 369, 676, 704, 732, 762)
- Changed `run_cppcheck_scanner` to return `(Vec<SastFinding>, BTreeSet<PathBuf>)` — scanned dirs tracked per successful invocation
- Added `suppress_lexical_false_positives` function and exported from `vulnerability` mod
- Wired suppression into main.rs between `deduplicate_sast_findings` and writers; both Console and All arms covered
- 8 unit tests covering all suppression invariants (covered/uncovered CWE, scanned/outside dir, confirmed, source type, empty dirs, path normalization)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add suppression logic and change run_cppcheck_scanner signature** - `596964f` (feat)
2. **Task 2: Write SARIF-07 suppression unit tests** - `7a5cd15` (test)
3. **Task 3: Wire suppression into main.rs** - `df0c85f` (feat)

## Files Created/Modified
- `src/vulnerability/cwe_scanner.rs` - Added CPPCHECK_COVERED_CWES, tuple return on run_cppcheck_scanner, suppress_lexical_false_positives
- `src/vulnerability/mod.rs` - Added suppress_lexical_false_positives to pub use list
- `src/main.rs` - Destructure tuple return, build confirmed set, call suppress before writers
- `tests/vulnerability_tests/suppression_tests.rs` - 8 new SARIF-07 suppression unit tests
- `tests/vulnerability_tests/mod.rs` - Registered suppression_tests module
- `tests/vulnerability_tests/cppcheck_scanner_tests.rs` - Fixed tuple destructure for existing tests (Rule 3 fix)

## Decisions Made
- Build `cppcheck_confirmed` from the post-dedup slice (not raw `cppcheck_findings`) — paths in the dedup output are normalized; building from raw findings would cause path mismatch
- Only insert into `scanned_dirs` when cppcheck invocation succeeds (exit 0 or 1); skipped components are excluded

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed tuple destructure in existing cppcheck_scanner_tests.rs**
- **Found during:** Task 2 (running suppression tests)
- **Issue:** `cppcheck_scanner_tests.rs` called `findings.is_empty()` on the return of `run_cppcheck_scanner`, which is now a tuple. This blocked all `cargo test` compilation.
- **Fix:** Changed `let findings = run_cppcheck_scanner(...)` to `let (findings, _scanned_dirs) = run_cppcheck_scanner(...)` in two test functions
- **Files modified:** `tests/vulnerability_tests/cppcheck_scanner_tests.rs`
- **Verification:** All 356+ tests compile and pass (8 new suppression tests pass)
- **Committed in:** `7a5cd15` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (blocking)
**Impact on plan:** Necessary fix — changing the return type of run_cppcheck_scanner required updating the existing call site in tests. No scope creep.

## Issues Encountered
- The `test_spdx_output_passes_pyspdxtools_validation` test fails due to the `example_target_repos/rclcpp` directory not being present in the worktree. This is a pre-existing environment issue unrelated to this plan's changes. Logged as a deferred item.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- SARIF-07 suppression is complete and wired
- `suppress_lexical_false_positives` and `CPPCHECK_COVERED_CWES` are exported and tested
- main.rs pipeline: lexical scan -> cppcheck scan -> dedup -> build confirmed set -> suppress -> writers
- Ready for subsequent Phase 16 plans (fingerprinting, baseline diffing, markdown-from-SARIF)

---
*Phase: 16-sarif-as-authoritative-finding-store-refactor-the-static-ana*
*Completed: 2026-05-10*
