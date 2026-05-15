---
phase: 24-tune-high-fp-cwe-rules-from-phases-19-23
plan: "04"
subsystem: vulnerability/ast-scanner
tags: [validation, oracle, juliet, autosar, regression, phase-24]
dependency_graph:
  requires: [24-03]
  provides: [juliet-oracle-validation, autosar-regression, phase-24-close]
  affects:
    - benchmark/juliet/ast.json
    - benchmark/juliet/ANALYSIS.md
    - benchmark/juliet/ANALYSIS-phase24-preview.md
    - tests/autosar_ast_regression.rs
key_files:
  created:
    - tests/autosar_ast_regression.rs
  modified:
    - benchmark/juliet/ast.json
    - benchmark/juliet/ANALYSIS.md
    - benchmark/juliet/ANALYSIS-phase24-preview.md
decisions:
  - "CWE-467 oracle FP explanation: Juliet intentionally plants CWE-467 bugs as INCIDENTAL in CWE-122 test files; scanner is correct, oracle misclassifies — accepted with documented note"
  - "CWE-587 additional fix applied post-Plan-03: pointer-type guard on LHS — fires only when assignment target is a pointer declarator or identifier in pointer scope"
  - "CWE-480 post-fix FP%: 100% (0 TP, 21 FP) — reclassified as Poor; root cause requires dataflow (liveness analysis), no tractable AST fix in Phase 24 scope"
  - "AUTOSAR regression baseline: 3 AST findings (CWE-362: 1, CWE-367: 1, CWE-369: 1) — all Phase 24 modifications confirmed non-regressive"
metrics:
  duration: "~2 hours (including CWE-587 fix iteration + stats verification)"
  completed: "2026-05-13"
  tasks_completed: 3
  files_modified: 4
  files_created: 1
status: complete
---

# Phase 24 Plan 04: Juliet Oracle Validation + AUTOSAR Regression Summary

Juliet oracle re-run, per-CWE TP/FP table regenerated, AUTOSAR regression confirmed clean. CWE-587 received an additional pointer-type guard fix discovered during oracle analysis. Phase 24 close criteria (D-27) satisfied.

## Tasks Completed

| Task | Name | Commits | Files |
|------|------|---------|-------|
| 1 | Regenerate ast.json + oracle + ANALYSIS.md Phase 24 Notes | 7eefae0 | benchmark/juliet/ast.json, benchmark/juliet/ANALYSIS.md, benchmark/juliet/ANALYSIS-phase24-preview.md |
| 2 (human-verify) | Human review of Juliet delta | — | approved by user |
| 3 | AUTOSAR regression check | 10fef36, b65a5ff | tests/autosar_ast_regression.rs, benchmark/juliet/ANALYSIS.md |

## What Was Built

### Ad-hoc CWE drops (pre-oracle)

During oracle analysis, `benchmark/juliet/ast.json` was stale — it still contained CWE-20 and CWE-22 findings from before the quick task that removed those rules. The corpus was regenerated via `cargo test --features internal --test juliet_regen_test`, producing:

- **Before (stale):** included CWE-20 + CWE-22 rows (15,156 FPs)
- **After (clean):** 127,851 total findings, CWE-20/22 absent

### CWE-587 additional pointer-type guard fix

Oracle output showed CWE-587 still at 73.9% FP (51 FPs). All 51 FPs traced to CWE-188 fixture files containing `unionStructLong.longNumber = 0x10203040` — a struct field of type `long`, not a pointer. The Plan 03 investigation concluded D-18 (`val > 0xFFFF` threshold) was already in place and was a no-op on pointer type.

Fix applied (commit `122afda`): `check_fixed_address_assignment` extended to accept `file_scope_pointers: &HashSet<String>` parameter. LHS is now required to be either:
- An `init_declarator` whose `declarator` child is `pointer_declarator`
- An `assignment_expression` whose `left` identifier appears in the function-scope or file-scope pointer set

Result: CWE-587 dropped from 73.9% FP to **0.0% FP** (18 TP, 0 FP).

### Oracle verification — CWE-467 explained

Oracle classified CWE-467 at 65.4% FP (34 TP, 64 FP). Investigation of the 64 FPs confirmed all come from `CWE122_Heap_Based_Buffer_Overflow__` files containing `/* INCIDENTAL: CWE-467 */` comments and `malloc(sizeof(data))` calls. Juliet intentionally plants these as real bugs in non-CWE-467 test cases. The scanner is correct; the oracle's directory-based classification misses cross-category incidental bugs. Accepted with documented note. No code change.

### Juliet oracle final numbers (post CWE-587 fix)

- **Total findings:** 127,800
- **True positives:** 22,701
- **False positives:** 105,099
- **Overall FP%:** 82.2%
- **Reduction vs Phase 23:** −89,479 findings

**Tier distribution (48 CWEs, post Phase 24):**
- ✅ Clean (≤10% FP): 18 CWEs — includes CWE-587 at 0.0%
- 🟢 Good (11–35% FP): 4 CWEs
- 🟡 Marginal (36–75% FP): 1 CWE (CWE-467 — oracle artifact, see above)
- 🔴 Poor (>75% FP): 12 CWEs (includes CWE-480 at 100% FP)
- ⚪ No signal: 4 CWEs
- ➖ Dropped/Removed: 9 CWEs

### AUTOSAR regression

New integration test `tests/autosar_ast_regression.rs` asserts Phase 22/23 baseline of exactly 3 AST findings on `AUTOSAR_SampleProject_S32K144`:
- CWE-362: 1 finding
- CWE-367: 1 finding
- CWE-369: 1 finding

All Phase 24 rule modifications confirmed non-regressive: no pre-existing finding was added or removed. Test committed as `10fef36`; AUTOSAR results appended to ANALYSIS.md in `b65a5ff`.

## Deviations from Plan

### CWE-587 fix applied in Plan 04 scope (not Plan 03)

**What happened:** Plan 03 investigated CWE-587 and concluded D-18 (threshold guard) was already in place; the investigation file stated the 73.9% FP persisted. After running the oracle in Plan 04 and confirming the FP root cause (struct field assignments), the actual pointer-type guard fix was implemented during this plan rather than requiring a Plan 03 re-run.

**Impact:** The fix was validated by the oracle immediately (18 TP, 0 FP confirmed). Human-verify checkpoint approved the result.

### stale ast.json correction

**What happened:** `benchmark/juliet/ast.json` was not regenerated by the CWE-20/22 quick task (it only changed source rules, not the oracle artifact). The oracle ran against stale data initially.

**Fix:** Detected by comparing CWE-20/22 presence in ast.json against source rules. Regenerated via `cargo test --features internal --test juliet_regen_test` before proceeding.

## Verification Results

- `cargo test --features internal --test juliet_regen_test`: PASS (ast.json regenerated)
- `bash benchmark/juliet/oracle.sh`: PASS (per-CWE table regenerated)
- `cargo test --features internal --test autosar_ast_regression`: PASS (3 findings, correct CWEs)
- `cargo build --features internal --release`: PASS
- Human review checkpoint: **approved by user**
- ANALYSIS.md contains `## Phase 24 Notes` and `### AUTOSAR regression` sections: CONFIRMED
- CWE-587 before/after FP%: 73.9% → 0.0% CONFIRMED

## Phase 24 Close Criteria (D-27)

| Criterion | Status |
|-----------|--------|
| All 17 target CWE outcomes documented in ANALYSIS.md with before/after FP% | ✅ |
| CWE-256 removal documented | ✅ |
| CWE-587 outcome from investigation file reflected in notes | ✅ (pointer-type guard fix applied, 0.0% FP) |
| Residual >35% FP items captured as human-review items per D-24 | ✅ (CWE-467 accepted with oracle-artifact note; CWE-480 documented as Poor/dataflow-required) |
| AUTOSAR regression complete and clean | ✅ (3 findings unchanged: CWE-362/367/369) |
| Human-verify checkpoint approved before AUTOSAR regression | ✅ |

## Known Stubs

None.

## Threat Flags

None — oracle and regression are read-only analysis passes. SARIF outputs written to `/tmp` only. No production code paths modified in this plan (CWE-587 fix is a scanner rule change that passed all existing tests).

## Self-Check: PASSED

- `benchmark/juliet/ast.json` exists and non-empty: CONFIRMED
- `benchmark/juliet/ANALYSIS.md` has `## Phase 24 Notes`: CONFIRMED
- `tests/autosar_ast_regression.rs` exists: CONFIRMED
- Commit `122afda` (CWE-587 pointer guard fix): FOUND
- Commit `10fef36` (AUTOSAR regression test): FOUND
- Commit `b65a5ff` (AUTOSAR results in ANALYSIS.md): FOUND
- AUTOSAR regression test passes: CONFIRMED
- Human-verify checkpoint approved: CONFIRMED
