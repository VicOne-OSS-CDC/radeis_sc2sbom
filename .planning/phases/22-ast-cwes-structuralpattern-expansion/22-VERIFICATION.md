---
phase: 22-ast-cwes-structuralpattern-expansion
verified: 2026-05-12T23:59:00Z
status: human_needed
score: 3/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Confirm ROADMAP count discrepancy is acceptable: ROADMAP says 26→41 CWEs; actual is 25→40 (Phase 21 underdelivered by 1 CWE, inherited by Phase 22)"
    expected: "Developer confirms 40 total CWEs is the acceptable shipped state, or updates ROADMAP to read '25→40'"
    why_human: "The ROADMAP contract says 41 CWEs. The actual scanner has 40. This is a docs/contract discrepancy from a prior phase, not a code bug — needs developer decision."
  - test: "Confirm SC#1 acceptance: CWE-256 and CWE-674 both have 0 Juliet TPs (corpus mismatch). ROADMAP says '≥1 TP on Juliet corpus'. Plan D-11 policy and human checkpoint Task 4 approved. Is unit-test-only TP accepted as satisfying the ROADMAP criterion?"
    expected: "Developer confirms the Task 4 'approved' answer is sufficient to satisfy SC#1, or updates ROADMAP SC#1 wording to permit unit-test fallback"
    why_human: "SC#1 is written as a hard ROADMAP gate. The human checkpoint gave blanket approval, but the specific exception for CWE-256/674 needs explicit acknowledgment."
  - test: "Confirm SC#2 acceptance: 8 of 15 Phase 22 CWEs exceed 40% FP (CWE-256 100%, CWE-478 73.9%, CWE-480 99.9%, CWE-483 93.2%, CWE-562 99.8%, CWE-570 99.9%, CWE-571 100%, CWE-587 73.9%). ROADMAP says 'FP% ≤40%'. Plan D-11 policy and Task 4 approved. Is this acceptable?"
    expected: "Developer confirms the Task 4 blanket approval covers SC#2 FP exceptions and they are deferred to backlog 999.2"
    why_human: "SC#2 is a hard ROADMAP gate. 8/15 CWEs miss it. The human approval at Task 4 is documented in the SUMMARY, but the verifier cannot confirm which specific criteria the approver was aware of."
---

# Phase 22: ast-cwes-structuralPattern-expansion Verification Report

**Phase Goal:** Expand AST scanner from 26 to 41 CWEs by adding 15 new structural-shape rules (CWE-256, 398, 478, 480, 481, 482, 483, 484, 562, 570, 571, 587, 617, 674, 835). All rules validated against Juliet ground truth with FP% tracked in benchmark/juliet/ANALYSIS.md.
**Verified:** 2026-05-12T23:59:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | All 15 new CWEs produce ≥1 TP on the Juliet corpus (ROADMAP SC#1) | ? UNCERTAIN | 13/15 CWEs have Juliet TPs. CWE-256 (0 Juliet TPs, corpus mismatch: Juliet uses Windows LogonUserA API) and CWE-674 (0 Juliet TPs, corpus mismatch: preproc_ifdef guards hide function_definition from root-level iterator). Unit tests confirm TPs on synthetic fixtures. D-11 policy accepted and human Task 4 approved, but ROADMAP SC#1 says "≥1 TP on Juliet corpus" without unit-test fallback |
| 2 | FP% for each new CWE is ≤40% (ROADMAP SC#2) | ? UNCERTAIN | 7/15 CWEs pass (CWE-398 25.0%, CWE-481 0%, CWE-482 0%, CWE-484 0%, CWE-617 0%, CWE-674 n/a, CWE-835 0%). 8/15 exceed 40%: CWE-256 100%, CWE-478 73.9%, CWE-480 99.9%, CWE-483 93.2%, CWE-562 99.8%, CWE-570 99.9%, CWE-571 100%, CWE-587 73.9%. All documented per D-11; Task 4 human checkpoint approved. ROADMAP gate not met literally but human-accepted |
| 3 | No regression on existing 26 CWEs (ROADMAP SC#3) | ✓ VERIFIED | D-15 PASS: AUTOSAR_SampleProject_S32K144 scan shows Δ=0 for all pre-existing CWEs. Juliet pre-Phase-22 oracle rows unchanged. All 28 Phase 22 unit tests green; existing test suite regression-free (57 ast_scanner_tests pass). tmp/phase22_autosar_regression.md confirms PASS |
| 4 | benchmark/juliet/ANALYSIS.md updated with 15 new CWE rows (ROADMAP SC#4) | ✓ VERIFIED | `grep -cE '^\| CWE-(256\|398\|478\|480\|481\|482\|483\|484\|562\|570\|571\|587\|617\|674\|835) \|' benchmark/juliet/ANALYSIS.md` = 38 (exceeds 15 minimum). "## Phase 22 Notes" section present. D-05, D-06, D-15 all documented |
| 5 | AST scanner expanded to cover the 15 new CWEs with working implementations (all unit tests pass) | ✓ VERIFIED | 28 Phase 22 unit tests (TP + TN for all 15 CWEs) all pass green. 12 new check_* functions present. All 12 called from apply_ast_rules. CWE-617 in AST_CWE_RULES. CWE-570/571 via dynamic cwe_id variable in check_constant_condition |

**Score:** 3/5 truths definitively verified (SC#3, SC#4, implementation correct); 2 require human decision (SC#1 and SC#2 literal ROADMAP gates vs accepted exceptions)

### Deferred Items

None — all 15 Phase 22 CWEs have been implemented. High-FP tuning is tracked in backlog phase 999.2.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/vulnerability/ast_scanner.rs` | 12 new check_* functions + apply_ast_rules integration | ✓ VERIFIED | 1,914 lines; all 12 check_* functions present and called from apply_ast_rules with correct signature |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | 28 new unit tests (TP + TN per CWE) | ✓ VERIFIED | 835 lines; 40 total `fn test_cwe*` functions (28 are Phase 22 additions); all pass |
| `benchmark/juliet/ANALYSIS.md` | 15 new per-CWE rows + Phase 22 Notes section | ✓ VERIFIED | 38 matching rows found; "## Phase 22 Notes" section present with D-05/D-06/D-11/D-13/D-15 documentation |
| `benchmark/juliet/ast.json` | Refreshed Juliet scan output | ✓ VERIFIED | 74,936,821 bytes; mtime 2026-05-12 22:54 |
| `tmp/phase22_juliet_counts.md` | 15 per-CWE Juliet TP/FP counts | ✓ VERIFIED | 15 CWE rows confirmed by grep count |
| `tmp/phase22_autosar_regression.md` | D-15 regression evidence | ✓ VERIFIED | Contains "D-15 regression check: PASS" |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `ast_scanner.rs:apply_ast_rules` | `check_switch_structure` | direct call with `&mut findings` | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_block_delimitation` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_assignment_in_condition` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_comparison_at_statement` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_func_ptr_null_compare` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_return_stack_address` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_constant_condition` (CWE-570/571) | direct call | ✓ WIRED | Confirmed by grep; dynamic cwe_id variable |
| `ast_scanner.rs:apply_ast_rules` | `check_fixed_address_assignment` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_self_recursion` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_plaintext_password` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_infinite_loop` | direct call | ✓ WIRED | Confirmed by grep |
| `ast_scanner.rs:apply_ast_rules` | `check_poor_code_quality` | direct call | ✓ WIRED | Confirmed by grep |
| `AST_CWE_RULES` | `AnyCall for assert (CWE-617)` | table entry consumed by visit_node | ✓ WIRED | `cwe_id: 617, functions: &["assert"], arg_check: ArgCheck::AnyCall` confirmed |
| `check_return_stack_address` | `collect_local_var_names` | function call for local variable oracle | ✓ WIRED | `fn collect_local_var_names` and `collect_local_vars_in_subtree` + `collect_decl_identifiers` present |
| `check_constant_condition` | CWE-570/571 dynamic dispatch | `let cwe_id = if val == 0 { 570 } else { 571 }` | ✓ WIRED | Found at lines 872, 900 |

### Data-Flow Trace (Level 4)

Not applicable — this phase adds SAST scanner rules (analysis tools), not data-rendering components. The SastFinding structs are populated by the check_* functions and flow through the existing pipeline unchanged.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 28 Phase 22 CWE unit tests pass | `cargo test --features internal vulnerability_tests::ast_scanner_tests` | 57 passed, 0 failed | ✓ PASS |
| CWE-617 assert fires | test_cwe617_assert_call | ok | ✓ PASS |
| CWE-835 body-check TN (while(1)+break no fire) | test_cwe835_while_one_with_break_no_finding | ok | ✓ PASS |
| CWE-481 Pitfall 5 TN (nested assignment no fire) | test_cwe481_nested_assignment_in_while_no_finding | ok | ✓ PASS |
| CWE-256 TN (non-password var no fire) | test_cwe256_non_password_var_no_finding | ok | ✓ PASS |
| CWE-674 TN (no recursion no fire) | test_cwe674_no_recursion_no_finding | ok | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CWEXP-02 | 22-01, 22-02, 22-03, 22-04 | Structural-pattern CWE expansion (15 new CWEs) | ✓ SATISFIED | All 15 CWEs implemented and tested; Juliet benchmark documented; AUTOSAR regression clean |
| (orphaned) | — | CWEXP-02 is referenced in all 4 plans and ROADMAP but is NOT defined in `.planning/REQUIREMENTS.md` | ⚠️ ORPHANED | REQUIREMENTS.md covers AST-01 through DIST-02 only; CWEXP-01/CWEXP-02 are milestone-specific requirement IDs that were never added to REQUIREMENTS.md |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/vulnerability/ast_scanner.rs` | 3-5 | Module-level doc comment still says "Phase 21 expansion → 25 CWEs total" — not updated to reflect Phase 22 additions (40 CWEs) | ℹ️ Info | Documentation stale; does not affect runtime behavior |

### Human Verification Required

### 1. ROADMAP CWE Count Discrepancy (25→40 vs 26→41)

**Test:** Open `.planning/ROADMAP.md` line 68. It reads "total coverage 26→41 CWEs". The actual scanner starts at 25 CWEs (Phase 21 ANALYSIS.md: "Phase 21 expanded the AST scanner from 13 to 25 CWEs"). Phase 22 added 15, giving 40 total. The ROADMAP says 41 because Phase 21's goal was "13→26" but only reached "13→25".
**Expected:** Either: (a) developer accepts 40 CWEs as shipped and updates ROADMAP to "25→40", or (b) developer confirms 41 is correct and one CWE was missed.
**Why human:** This is a documentation contract discrepancy inherited from Phase 21. The code correctly added 15 CWEs. Whether 40 or 41 is the "right" number is a product decision.

### 2. SC#1 — CWE-256 and CWE-674 have 0 Juliet TPs

**Test:** In `benchmark/juliet/ANALYSIS.md`, Phase 22 Notes table shows CWE-256 (0 TPs, 1,056 FPs) and CWE-674 (0 TPs, 0 FPs). ROADMAP SC#1: "All 15 new CWEs produce ≥1 TP on the Juliet corpus."
**Expected:** Developer confirms Task 4 "approved" response constitutes acceptance that unit-test TP is sufficient for these two CWEs (corpus mismatch documented in ANALYSIS.md).
**Why human:** ROADMAP says "Juliet corpus" — unit tests are synthetic fixtures, not Juliet. The corpus mismatch is well-documented, but the ROADMAP gate does not contain the D-11 exception.

### 3. SC#2 — 8 of 15 CWEs exceed 40% FP

**Test:** ROADMAP SC#2: "FP% for each new CWE is ≤40%". CWE-256 (100%), CWE-478 (73.9%), CWE-480 (99.9%), CWE-483 (93.2%), CWE-562 (99.8%), CWE-570 (99.9%), CWE-571 (100%), CWE-587 (73.9%) all exceed the gate.
**Expected:** Developer confirms the Task 4 blanket "approved" covers this criterion, and the 8 exceptions are deferred to backlog 999.2 as documented.
**Why human:** 8 of 15 CWEs miss a hard ROADMAP gate. The human checkpoint at Plan 04 Task 4 gave blanket approval but did not list which specific ROADMAP SCs were being waived. This needs explicit acknowledgment to close the phase.

## Gaps Summary

No implementation gaps were found. All 15 CWEs are implemented, all 28 unit tests pass, all 12 check_* functions exist and are wired, the Juliet benchmark has been run and documented, and the AUTOSAR regression check passed. The three items requiring human decision are:

1. **ROADMAP count discrepancy** (26→41 vs actual 25→40) — inherited from Phase 21; needs docs fix or acceptance.
2. **SC#1 Juliet TP gate** — 2 CWEs accepted via unit-test fallback + D-11; needs explicit ROADMAP waiver or SC text update.
3. **SC#2 FP% gate** — 8/15 CWEs above 40%; D-11 policy accepted and Task 4 approved; needs explicit acknowledgment.

The code is fully implemented and correct. These are documentation/contract alignment items.

---

_Verified: 2026-05-12T23:59:00Z_
_Verifier: Claude (gsd-verifier)_
