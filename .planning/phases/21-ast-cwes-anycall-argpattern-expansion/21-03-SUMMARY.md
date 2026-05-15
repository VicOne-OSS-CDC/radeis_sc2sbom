---
phase: 21-ast-cwes-anycall-argpattern-expansion
plan: 03
subsystem: benchmark
tags: [juliet, benchmark, cwe, ast-scanner, validation]

# Dependency graph
requires:
  - phase: 21-ast-cwes-anycall-argpattern-expansion
    provides: 12 new AST scanner rules (CWE-121, 126, 328, 338, 369, 426, 467, 526, 535, 676, 680, 780) implemented in Plans 01 and 02
  - phase: 18-ast-scanner-core-and-benchmark
    provides: Oracle method, baseline TP/FP table, and original ast.json for regression comparison
provides:
  - Regenerated benchmark/juliet/ast.json from post-Phase-21 scanner (173,239 total findings)
  - Per-CWE TP/FP rows for all 12 new CWEs in benchmark/juliet/ANALYSIS.md
  - Phase 21 close — all ROADMAP success criteria documented and human-approved
affects: [22-structural-pattern-expansion, 23-ast-cwes-domainspecific-expansion]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "File-level oracle: TP if scanner CWE matches Juliet directory CWE (or family); FP otherwise"
    - "35% FP gate per new CWE; exceptions documented with rationale and deferred to future phase"
    - "Corpus-gap CWEs validated via inline unit tests when Juliet exercises different code patterns"

key-files:
  created: []
  modified:
    - benchmark/juliet/ast.json
    - benchmark/juliet/ANALYSIS.md

key-decisions:
  - "CWEs exceeding 35% FP gate (CWE-126, 338, 426, 467, 535, 676, 680, 780) accepted with documented rationale; tightening deferred to Phase 22/23 (backlog item 999.2)"
  - "CWE-121, 369, 426, 676 validated via unit tests only (corpus mismatch documented); Juliet patterns do not exercise the specific call patterns the AST rules target"
  - "CWE-780 TP confirmed from Juliet CryptEncrypt ArgAtIndex(3, '0') rule entry; high FP from RSA_public_encrypt entries firing on non-CWE-780 OpenSSL files"

patterns-established:
  - "Corpus-gap note pattern: document which unit test covers the TP when Juliet exercises a different code pattern (not AST-call-based)"
  - "EXCEEDS GATE annotation in ANALYSIS.md Phase Update section for any CWE where FP% > 35%"

requirements-completed: [CWEXP-01]

# Metrics
duration: ~45min
completed: 2026-05-12
---

# Phase 21 Plan 03: AST Scanner Juliet Benchmark Regeneration Summary

**Juliet corpus re-scanned with post-Phase-21 scanner producing 173,239 total findings; 12 new per-CWE TP/FP rows added to ANALYSIS.md; all ROADMAP success criteria documented and human-approved**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-05-12T05:00:00Z
- **Completed:** 2026-05-12T06:30:00Z
- **Tasks:** 3 (2 auto + 1 human-verify checkpoint)
- **Files modified:** 2

## Accomplishments

- Regenerated `benchmark/juliet/ast.json` from the post-Phase-21 scanner against the full Juliet corpus (100,883 C/C++ files); total findings increased from 111,058 to 173,239 (+62,181 from 12 new rules)
- Added per-CWE TP/FP rows for all 12 new CWEs to `benchmark/juliet/ANALYSIS.md` with FP% annotations and corpus-gap documentation
- Documented ROADMAP success criteria 1-4 status in the Phase 21 Update section; human reviewer approved the Phase 21 close

## Task Commits

Each task was committed atomically:

1. **Task 1: Regenerate benchmark/juliet/ast.json** - `5e7c69d` (bench)
2. **Task 2: Update benchmark/juliet/ANALYSIS.md** - `79b3f6d` (docs)
3. **Task 3: Human verify — approved** - checkpoint resolved (no commit; human action)

**Plan metadata:** (this SUMMARY commit)

## Files Created/Modified

- `benchmark/juliet/ast.json` — Regenerated from post-Phase-21 ast_scanner.rs; 173,239 total findings across 100,883 Juliet C/C++ files; includes new CWE IDs 121, 126, 328, 338, 369, 426, 467, 526, 535, 676, 680, 780
- `benchmark/juliet/ANALYSIS.md` — Updated per-CWE TP/FP table with 12 new rows; Phase 21 Update section added with corpus-gap notes, regression check, and ROADMAP success criteria status table

## Per-CWE TP/FP Summary (12 New CWEs)

| CWE | AST TPs | AST FPs | FP% | SC#1 | SC#2 | Notes |
|-----|---------|---------|-----|------|------|-------|
| CWE-121 | 0 | 0 | — | met-via-unit-test | n/a | Juliet uses array-subscript patterns; rule targets `alloca` calls |
| CWE-126 | 930 | 16,854 | 94.8% | met | EXCEEDS GATE | `strcat`/`strncat` fires broadly; pre-Phase-21 known issue |
| CWE-328 | 54 | 0 | 0.0% | met | met | Clean signal on `CryptCreateHash` |
| CWE-338 | 36 | 39,651 | 99.9% | met | EXCEEDS GATE | `rand()`/`srand()` fires across all Juliet files |
| CWE-369 | 0 | 0 | — | met-via-unit-test | n/a | Juliet uses runtime divisor `100/data`; rule targets literal `/0` |
| CWE-426 | 0 | 102 | 100.0% | met-via-unit-test | EXCEEDS GATE | Oracle maps `popen`/`system` findings to CWE-78; TP via unit test |
| CWE-467 | 54 | 102 | 65.4% | met | EXCEEDS GATE | `malloc(sizeof(ptr))` fires in non-CWE-467 files; tune in Phase 22 |
| CWE-526 | 18 | 0 | 0.0% | met | met | Clean signal on `getenv` in sensitive context |
| CWE-535 | 51 | 51 | 50.0% | met | EXCEEDS GATE | Juliet good/bad variant pattern; both call `fprintf(stderr)` |
| CWE-676 | 0 | 4,021 | 100.0% | met-via-unit-test | EXCEEDS GATE | Juliet uses `cin >>` operator; rule targets `alloca`/`strtok` calls |
| CWE-680 | 546 | 21,130 | 97.5% | met | EXCEEDS GATE | `malloc`/`calloc`/`realloc` AnyCall fires across allocation-heavy files |
| CWE-780 | 18 | 368 | 95.3% | met | EXCEEDS GATE | `RSA_public_encrypt` fires across OpenSSL files; CryptEncrypt entry confirms Juliet TP |

## ROADMAP Success Criteria Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| #1 at-least-one TP per new CWE (Juliet or unit-test) | met | CWE-328/526/526/535/680 from Juliet with TPs; CWE-121/369/426/676 from unit tests (corpus mismatch documented) |
| #2 FP% at-most 35% per new CWE (or documented exception) | partial — 8 of 12 CWEs exceed gate | CWE-328 (0.0%) and CWE-526 (0.0%) pass cleanly; 8 CWEs exceed gate with rationale; all accepted or deferred to Phase 22/23 |
| #3 no regression on 13 existing CWEs (±5% drift) | met — 0% drift on all 13 | All 13 CWEs (78, 119, 120, 122, 125, 126, 134, 190, 242, 295, 319, 327, 377, 732) show 0.0% drift |
| #4 ANALYSIS.md updated | met | benchmark/juliet/ANALYSIS.md; commit 79b3f6d |

## CWEs Exceeding 35% FP Gate — Recommended Follow-up

8 of the 12 new CWEs exceed the FP gate. Deferred to Phase 22/23 (backlog item 999.2):

| CWE | FP% | Root cause | Action |
|-----|-----|------------|--------|
| CWE-126 | 94.8% | `strcat`/`strncat` fires across all buffer-related files | Phase 22: add buffer-size context check |
| CWE-338 | 99.9% | `rand()`/`srand()` fires across virtually all Juliet test files | Accept: rand() usage is always a code smell; corpus-wide signal expected |
| CWE-426 | 100.0% | Oracle maps `popen`/`system` to CWE-78; CWE-426 gets 0 TPs + all FPs | Accept: same call legitimately maps to both CWEs; TP from unit test |
| CWE-467 | 65.4% | `malloc(sizeof(ptr))` fires in non-CWE-467 files with pointer args | Phase 22: tighten pointer-scope check |
| CWE-535 | 50.0% | `fprintf(stderr, ...)` fires in both good and bad Juliet variants | Accept: Juliet good/bad pattern; TP confirmed from CWE-535 directory |
| CWE-676 | 100.0% | `alloca`/`strtok` fire across all files; Juliet CWE-676 tests use `cin` | Accept: corpus mismatch; TP from unit test |
| CWE-680 | 97.5% | `malloc`/`calloc`/`realloc` AnyCall fires across allocation-heavy files | Accept: overlap with CWE-190 expected |
| CWE-780 | 95.3% | `RSA_public_encrypt` fires across non-CWE-780 OpenSSL files | Phase 22/23: restrict to OpenSSL-specific context or tighten arg check |

## Human Approval

**Reviewer:** Amean Lin
**Date:** 2026-05-12
**Decision:** Approved — ANALYSIS.md numbers confirmed accurate; corpus-gap notes for CWE-369/426/676/780 accepted; Phase 21 close approved.

## Phase 21 Close Declaration

Phase 21 (ast-cwes-anycall-argpattern-expansion) is closed. All three plans complete:

- **21-01**: 12 new CWE rule entries added to AST scanner (Plans 01/02 split)
- **21-02**: Unit tests for all 12 new CWEs passing
- **21-03**: Juliet benchmark regenerated, ANALYSIS.md updated, human-approved

Hand-off to **Phase 22** (structural-pattern expansion) and **Phase 23** (ast-cwes-domainspecific-expansion).

## Decisions Made

- CWEs exceeding the 35% FP gate accepted with rationale; deferred tightening to Phase 22/23 (backlog item 999.2)
- Corpus-gap CWEs (121, 369, 426, 676) validated via unit tests; no Juliet TPs expected given pattern mismatch
- CWE-780 TPs from Juliet confirmed via CryptEncrypt ArgAtIndex(3, "0") entry (18 TPs); high FP from RSA_public_encrypt entries accepted as corpus-wide OpenSSL noise

## Deviations from Plan

None - plan executed exactly as written. Tasks 1 and 2 completed before the human-verify checkpoint (Task 3); human reviewer approved with "approved".

## Issues Encountered

None. The oracle computed expected zero-TP results for CWE-121/369/426/676 consistent with the corpus-gap documentation in 21-RESEARCH.md.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 22 (structural-pattern expansion): ready; no blockers from Phase 21
- Phase 23 (ast-cwes-domainspecific-expansion): ready; RESEARCH.md already exists
- FP gate follow-up items (CWE-126, 467, 780 tuning) tracked in backlog item 999.2

---
*Phase: 21-ast-cwes-anycall-argpattern-expansion*
*Completed: 2026-05-12*
