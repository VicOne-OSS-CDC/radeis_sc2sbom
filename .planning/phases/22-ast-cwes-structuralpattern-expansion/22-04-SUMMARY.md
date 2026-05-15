---
phase: 22-ast-cwes-structuralpattern-expansion
plan: 04
subsystem: sast
tags: [benchmark, juliet, validation, documentation, ast-scanner]

# Dependency graph
requires:
  - phase: 22-01
    provides: check_switch_structure, check_block_delimitation, check_assignment_in_condition, check_comparison_at_statement, check_func_ptr_null_compare
  - phase: 22-02
    provides: check_return_stack_address, check_constant_condition, check_fixed_address_assignment
  - phase: 22-03
    provides: CWE-617 AnyCall entry, check_self_recursion, check_plaintext_password, check_infinite_loop, check_poor_code_quality
provides:
  - "benchmark/juliet/ast.json refreshed with post-Phase-22 scanner output (214,558 total findings)"
  - "benchmark/juliet/ANALYSIS.md: 15 new per-CWE TP/FP rows (4 updated + 11 inserted)"
  - "Phase 22 Notes section in ANALYSIS.md documenting D-05, D-06, D-07, D-11, D-13, D-15"
  - "tmp/phase22_juliet_counts.md: scratch TP/FP counts for 15 Phase 22 CWEs"
  - "tmp/phase22_autosar_regression.md: D-15 regression evidence (PASS)"
affects: [phase-23]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Juliet regen: cargo test --features internal --test juliet_regen_test -- --nocapture"
    - "Oracle: benchmark/juliet/oracle.sh (Python inline; file-level TP/FP; all sources combined)"
    - "AUTOSAR scan: run_ast_scanner with component_dirs pointing to AUTOSAR_SampleProject_S32K144"

key-files:
  created:
    - tmp/phase22_juliet_counts.md
    - tmp/phase22_autosar_regression.md
  modified:
    - benchmark/juliet/ast.json
    - benchmark/juliet/ANALYSIS.md

key-decisions:
  - "CWE-256 and CWE-674: 0 Juliet TPs accepted as corpus mismatch (unit tests confirm TP on synthetic fixtures)"
  - "AUTOSAR regression baseline: confirmed from post-Phase-22 scan (0 findings for all 13 pre-existing CWEs)"
  - "D-15 PASS: AUTOSAR total findings = 3 (CWE-362/367/369, unchanged from pre-Phase-22)"

requirements-completed: [CWEXP-02]

# Metrics
duration: ~30min (scan: ~4min, oracle: ~1min, AUTOSAR: ~30s, docs: ~25min)
completed: 2026-05-12
---

# Phase 22 Plan 04: Juliet Benchmark Validation Summary

**Post-Phase-22 Juliet benchmark run producing 214,558 total findings; 15 new per-CWE TP/FP rows added to ANALYSIS.md; D-15 AUTOSAR regression check PASS; human checkpoint approved**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 4 completed (3 auto + 1 human-verify — approved)
- **Files modified:** 4 (ast.json, ANALYSIS.md, tmp/phase22_juliet_counts.md, tmp/phase22_autosar_regression.md)

## Accomplishments

- Ran `juliet_regen_test` against the Juliet C corpus (100,883 C/C++ files) with the post-Phase-22 scanner
- Regenerated `benchmark/juliet/ast.json`: 214,558 total findings (147,457 AST + 67,101 Lexical)
- Ran `benchmark/juliet/oracle.sh` to compute per-CWE TP/FP for all 15 Phase 22 CWEs
- Ran `run_ast_scanner` on AUTOSAR_SampleProject_S32K144 (193 C files): 3 total findings, all pre-existing
- Created `tmp/phase22_juliet_counts.md`: 15 per-CWE rows with TP/FP and known-gap documentation
- Created `tmp/phase22_autosar_regression.md`: D-15 regression evidence table
- Updated `benchmark/juliet/ANALYSIS.md`: 15 new rows + Phase 22 Notes section

## Task Commits

1. **Task 1: Regenerate ast.json + capture per-CWE counts + AUTOSAR regression** - `e42101e` (bench)
2. **Task 2/3: Update ANALYSIS.md with 15 rows + Phase 22 Notes** - `1e3cfaf` (docs)
3. **Task 4: Human checkpoint** — APPROVED (human response: "approved"; high-FP CWEs deferred to backlog 999.2)

## Phase 22 Per-CWE TP/FP Results (15 New CWEs)

| CWE | AST TPs | AST FPs | FP% | SC#1 | D-11 | Notes |
|-----|---------|---------|-----|------|------|-------|
| CWE-256 | 0 | 1,056 | 100.0% | met-via-unit-test | EXCEEDS GATE | Corpus mismatch: Juliet uses Windows API |
| CWE-398 | 54 | 18 | 25.0% | met | met | 4 sub-rules; below 40% FP gate |
| CWE-478 | 18 | 51 | 73.9% | met | EXCEEDS GATE | switch-without-default fires across all CWEs |
| CWE-480 | 18 | 22,755 | 99.9% | met | EXCEEDS GATE | identifier==0 pattern too broad |
| CWE-481 | 18 | 0 | 0.0% | met | met | Excellent precision |
| CWE-482 | 18 | 0 | 0.0% | met | met | Excellent precision |
| CWE-483 | 20 | 275 | 93.2% | met | EXCEEDS GATE | Braceless if is common style (Pitfall 6) |
| CWE-484 | 18 | 0 | 0.0% | met | met | Excellent precision |
| CWE-562 | 3 | 1,368 | 99.8% | met | EXCEEDS GATE | return-local-var pattern very broad |
| CWE-570 | 2 | 1,581 | 99.9% | met | EXCEEDS GATE | By design per D-06; literal-only rule |
| CWE-571 | 2 | 13,376 | 100.0% | met | EXCEEDS GATE | By design per D-06; literal-only rule |
| CWE-587 | 18 | 51 | 73.9% | met | EXCEEDS GATE | (type*)0xNNNN fires in embedded code |
| CWE-617 | 597 | 0 | 0.0% | met | met | Best performer: 0% FP, 597 TPs |
| CWE-674 | 0 | 0 | — | met-via-unit-test | n/a | Corpus mismatch: preprocessor guards |
| CWE-835 | 2 | 0 | 0.0% | met | met | Body-check (D-05) works correctly |

**Standout results:**
- CWE-617 (assert): 597 TPs, 0 FPs — best Phase 22 performer
- CWE-481, 482, 484: 18 TPs each, 0 FPs — perfect precision
- CWE-398: 25.0% FP — below the 40% gate

## D-15 AUTOSAR Regression Evidence

Fixture: AUTOSAR_SampleProject_S32K144 (193 C files)

| CWE | Pre-Phase-22 count | Post-Phase-22 count | Δ |
|-----|---------------------|---------------------|---|
| CWE-78 through CWE-732 (13 pre-existing) | 0 | 0 | 0 |
| CWE-362 (pre-existing lexical) | 1 | 1 | 0 |
| CWE-367 (pre-existing lexical) | 1 | 1 | 0 |
| CWE-369 (pre-existing AST) | 1 | 1 | 0 |
| CWE-256..835 (15 new Phase 22) | — | 0 | +0 |

**D-15 regression check: PASS** — all pre-existing AUTOSAR CWE counts unchanged. Phase 22 adds 0 new findings on AUTOSAR.

## Known Corpus Gaps (0 Juliet TPs)

### CWE-256 (Plaintext Password Storage)
- Juliet pattern: Windows API (LogonUserA) with file-read into `char[100]` buffer — no string literal
- AST rule: identifier name containing "password"/"passwd"/"pwd" with string_literal initializer
- Confirmed via: `test_cwe256_password_string_literal` TP; `test_cwe256_pwd_uppercase` TP

### CWE-674 (Uncontrolled Recursion)
- Juliet pattern: `static void helperBad()` inside `#ifndef OMITBAD` preprocessor guard
- AST rule: iterates root-level `function_definition` children (misses functions inside `preproc_ifdef`)
- Confirmed via: `test_cwe674_direct_self_recursion` TP (raw fixture, no guards)
- Remediation: extend `check_self_recursion` to walk into `preproc_ifdef` children (future work, tracked)

## ROADMAP Success Criteria (Phase 22)

| Criterion | Status | Evidence |
|-----------|--------|----------|
| #1 ≥1 TP per new CWE (Juliet or unit-test) | met | 13 CWEs with Juliet TPs; CWE-256/674 via unit tests (corpus mismatch) |
| #2 FP% documented for each new CWE | met | Per-CWE table above; 8 CWEs exceed 40% with rationale |
| #3 No regression on existing 26 CWEs | met | D-15 PASS; AUTOSAR Δ = 0; Juliet pre-Phase-22 rows unchanged |
| #4 ANALYSIS.md updated | met | 15 new rows + Phase 22 Notes section |

## Cumulative Phase 22 Outcome

**Phase 22 expanded AST scanner coverage from 25 to 40 CWEs** (+15 structural-pattern rules):
- Plan 01 (Group A): CWE-478, 484, 481, 482, 480, 483
- Plan 02 (Group B): CWE-562, 570, 571, 587
- Plan 03 (Group C): CWE-617, 674, 256, 835, 398
- Plan 04 (Benchmark): Juliet validation + AUTOSAR regression

Cumulative Phase 22 code added: 12 check_* functions + 12 helper functions + 1 AST_CWE_RULES entry + 28 unit tests (all GREEN).

## Human Checkpoint Result

**Task 4 (human-verify):** APPROVED
- Human response: "approved" — Phase 22 ships as documented
- High-FP CWEs (256, 478, 480, 483, 562, 570, 571, 587) appended to backlog phase 999.2 for future fine-tuning
- No gap closure plans requested

## Known Stubs

None — all 4 tasks complete.

## Deviations from Plan

### Deviation: Updated existing ANALYSIS.md rows (CWE-398, 562, 570, 571)

- **Rule:** None (documentation accuracy)
- **Found during:** Task 2 (ANALYSIS.md update)
- **Issue:** Phase 22 CWEs 398, 562, 570, 571 already had rows in ANALYSIS.md as placeholder "0/0/—" entries from Phase 18 corpus scan (before Phase 22 rules existed). Plan said "no changes to existing rows" meaning pre-Phase-22 findings rows should not be altered. The placeholder rows were expected to be zero-filled until Phase 22 added the rules.
- **Fix:** Updated the 4 existing zero rows with Phase 22 actual counts. Inserted 11 new rows for the other Phase 22 CWEs.
- **Rationale:** Updating zero-placeholder rows with actual data is the correct outcome; "no changes to existing rows" was intended to protect real Phase 18/21 finding data, not perpetuate stale zeros.

---

**Total deviations:** 1 (documentation accuracy adjustment)

## Phase 23 Entry Recommendation

Phase 23 (AST CWE DomainSpecific Expansion) is ready to start. No blockers from Phase 22.

Follow-on gap closure items for consideration (tracked in backlog item 999.2):
1. **CWE-674 preprocessor guard gap**: extend `check_self_recursion` to walk `preproc_ifdef` children
2. **CWE-480 FP reduction**: narrow `identifier == 0/NULL` to function pointer context only
3. **CWE-483 FP reduction**: add threshold (e.g., only multi-statement bodies) or style flag
4. **CWE-587 FP reduction**: restrict to HAL/driver directories or require specific type cast patterns
5. **CWE-562 FP reduction**: requires declaration-site scope check (more complex)

## Threat Surface Scan

No new network endpoints, auth paths, file access patterns, or schema changes introduced. All changes are documentation (ANALYSIS.md) and benchmark data (ast.json, tmp/ files).

Threat model mitigations (T-22-12: ANALYSIS.md numbers from ast.json + oracle — reproducible; verified via git history):
- ast.json is reproduced by `cargo test --features internal --test juliet_regen_test`
- oracle.sh is deterministic given the same ast.json and corpus

## Self-Check: PASSED

- FOUND: benchmark/juliet/ast.json (74,936,821 bytes, mtime 2026-05-12 22:38)
- FOUND: benchmark/juliet/ANALYSIS.md (updated with 15 new rows + Phase 22 Notes)
- FOUND: tmp/phase22_juliet_counts.md (15 CWE rows)
- FOUND: tmp/phase22_autosar_regression.md (D-15 PASS evidence)
- FOUND commit e42101e (bench(22-04): regenerate ast.json + capture Phase 22 counts)
- FOUND commit 1e3cfaf (docs(22-04): update ANALYSIS.md)
- VERIFIED: grep -cE 'Phase 22 CWE rows' = 38 (≥15)
- VERIFIED: D-15 regression check: PASS present in ANALYSIS.md

---
*Phase: 22-ast-cwes-structuralpattern-expansion*
*Completed: 2026-05-12*
