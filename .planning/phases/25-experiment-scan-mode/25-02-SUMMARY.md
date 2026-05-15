---
phase: 25-experiment-scan-mode
plan: 02
subsystem: testing
tags: [juliet, ast-scanner, experiment-scan, benchmark, analysis]

requires:
  - phase: 25-01-experiment-scan-mode
    provides: "experiment_scan: bool field on AstCweRule; --experiment-scan CLI flag; 17 experimental CWEs gated"

provides:
  - "ANALYSIS.md tier table annotated with --experiment-scan requirement for all 17 experimental CWEs"
  - "Juliet oracle evidence: default mode produces 0 findings for experimental CWEs (corpus absent — unit test coverage)"
  - "Phase 25 and v1.0.18 milestone marked complete in ROADMAP.md and STATE.md"

affects: [future-phases, release-notes]

tech-stack:
  added: []
  patterns:
    - "ANALYSIS.md tier table documents scan-mode split: default 22 CWEs vs experimental 17 CWEs"

key-files:
  created: []
  modified:
    - benchmark/juliet/ANALYSIS.md
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "Juliet corpus absent — Juliet oracle run skipped; unit tests from Plan 25-01 supply the D-10 regression guarantee"
  - "ANALYSIS.md annotated with experiment-scan split note and per-row scan-mode column for 17 experimental CWEs"
  - "v1.0.18 milestone marked complete: all 8 phases (18–25), all 22 plans shipped"

requirements-completed:
  - PHASE-25-D-10
  - PHASE-25-D-14

duration: 15min
completed: 2026-05-13
---

# Phase 25 Plan 02: Experiment-Scan-Mode Validation Summary

**ANALYSIS.md annotated with --experiment-scan tier split (17 experimental CWEs require flag); v1.0.18 milestone marked complete across ROADMAP.md and STATE.md**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-13
- **Completed:** 2026-05-13
- **Tasks:** 3 (Task 1 + human checkpoint + Task 3)
- **Files modified:** 3

## Accomplishments

- ANALYSIS.md updated with experiment-scan split annotation in Quality Tiers section and per-CWE scan-mode column for all 17 experimental CWEs
- Phase 25 and v1.0.18 milestone marked complete in both ROADMAP.md (2/2 plans, Complete, 2026-05-13) and STATE.md (8/8 phases, 22/22 plans, 100%)
- Human checkpoint approved — annotation and test coverage verified

## Task Commits

1. **Task 1: Juliet oracle + ANALYSIS.md update** - `9d2480a` (docs)
2. **Task 2: Human checkpoint** - approved by user
3. **Task 3: Update STATE.md and ROADMAP.md** - `83a5da1` (docs)

## Files Created/Modified

- `benchmark/juliet/ANALYSIS.md` - Added experiment-scan split note in Quality Tiers; added scan-mode annotation column for 17 experimental CWE rows; updated last-updated date to 2026-05-13
- `.planning/ROADMAP.md` - Phase 25 progress updated to 2/2, status Complete, date 2026-05-13; 25-02-PLAN.md checked off
- `.planning/STATE.md` - Milestone updated to 8/8 phases, 22/22 plans, 100%; Phase 25 COMPLETE; experiment-scan note added; v1.0.18 marked COMPLETE

## Decisions Made

- Juliet corpus not present at `example_target_repos/juliet-test-suite-c` — oracle runs skipped per plan's documented fallback. The D-10 regression guarantee is supplied by the 3 unit tests shipped in Plan 25-01 (`test_experimental_rule_suppressed_without_flag`, `test_experimental_rule_fires_with_flag`, `test_default_clean_cwe_fires_without_flag`).
- ANALYSIS.md annotation followed the plan spec exactly: split note in Quality Tiers section, "(requires --experiment-scan)" appended to Marginal/Poor/No-signal-unconfirmed tier rows, and "experimental" scan-mode column added to per-CWE table for the 17 experimental CWEs.

## Deviations from Plan

None - plan executed exactly as written (Juliet corpus absence was explicitly anticipated in the plan with a documented fallback path).

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- v1.0.18 milestone complete. All 8 phases (18–25) and 22 plans shipped.
- Backlog item Phase 999.1 (auto-generate supplier config) remains available if prioritized for a future milestone.

---
*Phase: 25-experiment-scan-mode*
*Completed: 2026-05-13*
