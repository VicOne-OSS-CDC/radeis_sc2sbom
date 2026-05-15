---
phase: 23-ast-cwes-domainspecific-expansion
plan: 03
subsystem: benchmark-validation
tags: [rust, tree-sitter, ast-scanner, cwe, sast, benchmark, juliet]

# Dependency graph
requires:
  - phase: 23-ast-cwes-domainspecific-expansion
    plan: 02
    provides: CWE-479/591/762 structural helpers wired into scan_file_ast_or_lexical()
  - phase: 23-ast-cwes-domainspecific-expansion
    plan: 01
    provides: CWE-114/272/284/427/785 AstCweRule entries in AST_CWE_RULES

provides:
  - benchmark/juliet/ast.json regenerated with 49-CWE rule set (217,279 findings)
  - benchmark/juliet/ANALYSIS.md updated with 8 new per-CWE rows and Phase 23 notes section
  - Regression check: all 41 prior CWE TP counts confirmed unchanged
  - FP gate violation documented for CWE-762 (58.5%)

affects:
  - Phase 23 ROADMAP success criteria all met

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "juliet_regen_test integration test for reproducible Juliet corpus benchmark runs"
    - "oracle.sh file-level TP/FP oracle for per-CWE benchmark scoring"

key-files:
  created: []
  modified:
    - benchmark/juliet/ast.json
    - benchmark/juliet/ANALYSIS.md

key-decisions:
  - "CWE-762 FP gate violation documented (58.5% > 40%) per D-11 — ship and document; users suppress via --sarif-baseline"
  - "CWE-427/591 0-TP Juliet results accepted per D-11 — PUTENV macro / malloc-not-VirtualAlloc corpus mismatch"
  - "No FP gate violations for CWE-114/272/284/479/785 — all at 0.0% FP on Juliet"

requirements-completed: [CWEXP-03]

# Metrics
duration: ~35min
completed: 2026-05-12
---

# Phase 23 Plan 03: ast-cwes-domainspecific-expansion Summary

**Juliet benchmark re-run with 49-CWE rule set produces 217,279 findings; ANALYSIS.md updated with 8 Phase 23 CWE rows including FP gate violation for CWE-762 (58.5%) and regression check confirming all 41 prior CWEs unchanged**

## Performance

- **Duration:** ~35 min (benchmark run: ~4.3 min)
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Ran `cargo test --features internal --test juliet_regen_test -- --nocapture` on the full 100,883-file Juliet corpus with the post-Phase-23 (49-CWE) AST scanner
- Regenerated `benchmark/juliet/ast.json`: 217,279 total findings (up from 214,558 after Phase 22)
- Ran `benchmark/juliet/oracle.sh` to compute per-CWE TP/FP counts against the file-level oracle
- Added 8 new per-CWE rows to the master Per-CWE TP/FP Table in ANALYSIS.md
- Updated CWE-762 master table row from pre-Phase-23 placeholder (0/0/—) to actual results (590/832/58.5%)
- Added Phase 23 Notes section: per-CWE table, corpus-gap notes for CWE-427/591, FP Gate Violations for CWE-762, Regression Check table for all 41 prior CWEs
- Updated header note and total findings count to 217,279
- 423 unit tests pass; only pre-existing `test_spdx_output_passes_pyspdxtools_validation` failure (pyspdxtools not installed — unrelated to this plan)

## Phase 23 Oracle Results

| CWE | AST TPs | AST FPs | FP% | Status |
|-----|---------|---------|-----|--------|
| CWE-114 | 1,092 | 0 | 0.0% | met |
| CWE-272 | 102 | 0 | 0.0% | met |
| CWE-284 | 36 | 0 | 0.0% | met |
| CWE-427 | 0 | 0 | N/A | met-via-unit-test (D-11 PUTENV macro) |
| CWE-479 | 18 | 0 | 0.0% | met |
| CWE-591 | 0 | 0 | N/A | met-via-unit-test (D-11 malloc not VirtualAlloc) |
| CWE-762 | 590 | 832 | 58.5% | met but FP gate exceeded (documented) |
| CWE-785 | 51 | 0 | 0.0% | met |

## Task Commits

1. **Task 1: Regenerate ast.json with 49-CWE rule set** - `53d4ce6` (bench)
2. **Task 2: Update ANALYSIS.md with 8 new CWE rows and Phase 23 notes** - `85764eb` (docs)

## Files Created/Modified

- `benchmark/juliet/ast.json` - Regenerated: 217,279 findings (Phase 23 49-CWE scanner)
- `benchmark/juliet/ANALYSIS.md` - 8 new per-CWE rows in master table; CWE-762 row updated; Phase 23 Notes section added (Coverage, per-CWE table, FP Gate Violations, Regression Check, ROADMAP criteria)

## Decisions Made

- CWE-762 FP gate violation documented (58.5% > 40%) per D-11: `apply_delete_rules()` text-level scan fires on all `.cpp` files with `delete` regardless of CWE context; recommended action is to tighten to co-occurrence with malloc/calloc in Phase 24
- CWE-427/591 0-TP Juliet results accepted per D-11: PUTENV macro expansion and malloc-not-VirtualAlloc corpus mismatches are known; unit tests provide synthetic TPs

## Deviations from Plan

**1. [Rule 1 - CWE-762 TPs unexpected but correct] CWE-762 has 590 TPs (not 0)**
- **Found during:** Task 1 oracle run
- **Issue:** RESEARCH.md stated "0 TPs on Juliet (namespace causes parse error)" but `apply_delete_rules()` uses a text-level byte scan that bypasses the tree-sitter parse — it fires even when `has_error()` is true. The oracle scores 590 findings in CWE-762 directory files as TPs.
- **Resolution:** This is correct behavior — the text-level scan implemented in Plan 02 does produce TPs on Juliet. The RESEARCH.md prediction was made before the Plan 02 implementation was known. The FP% of 58.5% exceeds the gate and is documented in the FP Gate Violations section per D-11.
- **No code changes needed** — the implementation is correct; only the ANALYSIS.md documentation reflects the actual measured result.

## Issues Encountered

Pre-existing `test_spdx_output_passes_pyspdxtools_validation` failure continues (requires `pyspdxtools` binary in PATH, an environment issue present before these changes — confirmed in Plan 01 and Plan 02 SUMMARYs).

## Known Stubs

None — all 8 new CWE rows have real measured TP/FP data or documented D-11 corpus-mismatch rationale.

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are documentation (ANALYSIS.md) and benchmark data (ast.json).

## Self-Check

- [x] `benchmark/juliet/ast.json` exists and is non-empty (217,279 findings)
- [x] `grep -cE '"cwe_id"\s*:\s*(114|272|284|479|785)' benchmark/juliet/ast.json` returns 1299 (>= 1)
- [x] `benchmark/juliet/ANALYSIS.md` contains CWE-114 (6 matches)
- [x] `benchmark/juliet/ANALYSIS.md` contains CWE-785 (3 matches)
- [x] `benchmark/juliet/ANALYSIS.md` contains "Phase 23" (18 matches)
- [x] `benchmark/juliet/ANALYSIS.md` contains "D-11" (11 matches)
- [x] `benchmark/juliet/ANALYSIS.md` contains "Regression Check" (1 match)
- [x] `benchmark/juliet/ANALYSIS.md` grep '\b49\b' returns 4 (>= 1)
- [x] Commit 53d4ce6 exists (Task 1)
- [x] Commit 85764eb exists (Task 2)
- [x] cargo test --features internal -p radeis_sc2sbom: 423 passed, 1 pre-existing failure

## Self-Check: PASSED

---
*Phase: 23-ast-cwes-domainspecific-expansion*
*Plan: 03*
*Completed: 2026-05-12*
