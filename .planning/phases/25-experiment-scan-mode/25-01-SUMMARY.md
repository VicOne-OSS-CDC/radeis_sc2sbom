---
phase: 25-experiment-scan-mode
plan: 01
subsystem: sast
tags: [rust, ast-scanner, cwe, tdd, tree-sitter, internal-feature]

# Dependency graph
requires:
  - phase: 24-tune-high-fp-cwe-rules
    provides: "Tuned AST rule table (39 rules); structural check_* functions; AUTOSAR regression baseline"
provides:
  - "experimental: bool field on AstCweRule gating 17 high-FP CWEs"
  - "run_ast_scanner(experiment_scan: bool) public API"
  - "--experiment-scan CLI flag (internal feature-gated)"
  - "3 D-11 unit tests verifying experiment_scan=false suppresses and experiment_scan=true enables experimental CWEs"
affects: [25-02-PLAN, downstream-consumers-of-run_ast_scanner]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "experimental: bool field on rule table entries; visit_node guard: if rule.experimental && !experiment_scan { continue; }"
    - "Structural check_* functions receive experiment_scan param; guard at entry point"
    - "CLI flag as ArgAction::SetTrue under #[cfg(feature = 'internal')]"

key-files:
  created: []
  modified:
    - "src/vulnerability/ast_scanner.rs"
    - "src/cli.rs"
    - "src/main.rs"
    - "tests/vulnerability_tests/ast_scanner_tests.rs"
    - "tests/autosar_ast_regression.rs"
    - "tests/juliet_regen_test.rs"

key-decisions:
  - "10 table-driven experimental CWEs: 120, 122, 126, 190, 338, 426, 467, 676, 680, 780 — gated via visit_node rule.experimental filter"
  - "7 structural experimental CWEs: 478, 480, 483, 535, 562, 570, 571 — gated by experiment_scan param in each check_* function"
  - "CWE-484 (shares check_switch_structure with experimental CWE-478) remains non-experimental — only CWE-478 emission is guarded"
  - "Existing tests updated to pass experiment_scan=true so all prior TP assertions continue to hold"
  - "AUTOSAR regression updated to pass false — baseline 3 findings (CWE-362/367/369) unchanged"

patterns-established:
  - "experimental bool on static rule table — standard pattern for future opt-in rule additions"
  - "Parameter thread-through: public entry point -> file processor -> rule applier -> rule visitor"

requirements-completed:
  - PHASE-25-D-05
  - PHASE-25-D-07
  - PHASE-25-D-08
  - PHASE-25-D-09
  - PHASE-25-D-10
  - PHASE-25-D-11
  - PHASE-25-D-12
  - PHASE-25-D-13

# Metrics
duration: 38min
completed: 2026-05-13
---

# Phase 25 Plan 01: Experiment Scan Mode Summary

**`experimental: bool` field on AstCweRule gates 17 high-FP CWEs behind `--experiment-scan` CLI flag; `run_ast_scanner` gains `experiment_scan: bool` param threaded through the full call chain**

## Performance

- **Duration:** 38 min
- **Started:** 2026-05-13T07:33:30Z
- **Completed:** 2026-05-13T08:11:22Z
- **Tasks:** 2 (TDD: RED + GREEN)
- **Files modified:** 6

## Accomplishments

- Added `experimental: bool` to `AstCweRule` and annotated all 39 rule entries; 10 table-driven CWEs marked experimental (120, 122, 126, 190, 338, 426, 467, 676, 680, 780)
- Threaded `experiment_scan: bool` through `run_ast_scanner` → `scan_file_ast_or_lexical` → `apply_ast_rules` → `visit_node` with a single guard: `if rule.experimental && !experiment_scan { continue; }`
- Gated 7 structural CWEs in their respective `check_*` functions: 478 (not 484), 480, 483, 535, 562, 570/571
- Added `--experiment-scan` CLI flag (`ArgAction::SetTrue`, `#[cfg(feature = "internal")]`) to `src/cli.rs`
- All 3 D-11 unit tests pass; AUTOSAR regression still returns exactly 3 findings with `experiment_scan=false`

## Task Commits

1. **Task 1: RED — 3 failing unit tests for experiment_scan gating** - `2bb1b59` (test)
2. **Task 2: GREEN — implement experiment_scan gating throughout call chain** - `e4fcb87` (feat)

## Files Created/Modified

- `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/src/vulnerability/ast_scanner.rs` — AstCweRule struct, visit_node filter, structural check_* gates, run_ast_scanner signature
- `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/src/cli.rs` — `--experiment-scan` flag
- `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/src/main.rs` — pass `args.experiment_scan` to `run_ast_scanner`
- `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/tests/vulnerability_tests/ast_scanner_tests.rs` — 3 new D-11 tests; existing calls updated to pass `true`
- `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/tests/autosar_ast_regression.rs` — `run_ast_scanner(&dirs, false)`
- `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/tests/juliet_regen_test.rs` — `run_ast_scanner(&component_dirs, true)`

## Decisions Made

- Existing tests that verify experimental CWEs (CWE-120, 122, 126, etc.) updated to pass `experiment_scan=true` so all pre-existing TP assertions continue to hold — avoids false signal that those rules are broken
- `check_assignment_in_condition` (CWE-481) and `check_comparison_at_statement` (CWE-482) remain non-experimental (no `experiment_scan` param) per the plan's classification — these were not in the 17-experimental list
- CWE-427 (setenv/putenv) classified non-experimental per the plan's non-experimental list

## Deviations from Plan

**[Rule 3 - Blocking] Updated additional call sites not listed in plan**
- **Found during:** Task 2 (GREEN implementation)
- **Issue:** The plan listed 4 files to modify, but `run_ast_scanner` was also called in `tests/juliet_regen_test.rs` and internal tests inside `ast_scanner.rs` itself — both needed updating to compile
- **Fix:** Updated both call sites to pass `true` (preserving existing behavior)
- **Files modified:** `tests/juliet_regen_test.rs`, `src/vulnerability/ast_scanner.rs` (internal test module)
- **Verification:** `cargo build --features internal` passes; all tests pass
- **Committed in:** `e4fcb87`

---

**Total deviations:** 1 auto-fixed (blocking — additional call sites)
**Impact on plan:** Necessary for correctness. No scope creep.

## Issues Encountered

None — compilation succeeded on first attempt after threading the parameter.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes introduced. The `--experiment-scan` flag is a boolean with no user-supplied values beyond presence/absence (T-25-01..T-25-03 all accepted per plan threat model).

## Next Phase Readiness

- Phase 25 Plan 02 (`25-02-PLAN.md`) can proceed immediately
- `run_ast_scanner` API is stable; any future callers must pass `experiment_scan: bool`

---
*Phase: 25-experiment-scan-mode*
*Completed: 2026-05-13*
