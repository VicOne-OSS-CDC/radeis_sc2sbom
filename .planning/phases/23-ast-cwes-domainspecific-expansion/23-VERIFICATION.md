---
phase: 23-ast-cwes-domainspecific-expansion
verified: 2026-05-12T12:00:00Z
status: human_needed
score: 4/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Confirm SC#3 acceptance: CWE-762 FP% is 58.5%, exceeding the ROADMAP-stated 40% gate. The ANALYSIS.md FP Gate Violations subsection documents the cause (text-level delete scan fires on all .cpp files with delete regardless of CWE directory context) and defers tightening to Phase 24. Is the D-11 exception accepted for CWE-762 as it was for analogous high-FP CWEs in Phase 22?"
    expected: "Developer confirms D-11 policy covers CWE-762 at 58.5% FP, the FP Gate Violations documentation satisfies the ROADMAP SC#3 'or documented exception' intent, and Phase 24 tuning is tracked."
    why_human: "ROADMAP SC#3 says 'FP% ≤40% using file-level oracle where Juliet coverage exists'. CWE-762 has Juliet TPs (590) and FPs (832) — it is not a corpus-mismatch zero-TP case. The 58.5% FP exceeds the gate. The plan documents this via D-11, but the ROADMAP gate does not explicitly contain a D-11 bypass clause. This mirrors the Phase 22 SC#2 human item and needs explicit developer acceptance."
---

# Phase 23: ast-cwes-domainspecific-expansion Verification Report

**Phase Goal:** Expand the AST CWE rule set from 41 to 49 CWEs by adding 8 new detection patterns (5 table-driven AstCweRule entries + 3 structural helpers), update the Juliet benchmark, and document results in ANALYSIS.md.
**Verified:** 2026-05-12T12:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | All 8 new CWEs (114, 272, 284, 427, 479, 591, 762, 785) have rules implemented and documented (ROADMAP SC#1) | ✓ VERIFIED | 5 table entries in AST_CWE_RULES (lines 234–242 of ast_scanner.rs); 3 structural helpers (apply_signal_handler_rules at line 1476, apply_paired_lock_rules at line 1573, apply_delete_rules at line 1640); all 8 CWEs in ANALYSIS.md Phase 23 table |
| 2 | CWEs with Juliet test cases produce ≥1 TP; CWEs without Juliet coverage validated on synthetic fixtures (ROADMAP SC#2) | ✓ VERIFIED | Juliet TPs: CWE-114 (1,092), CWE-272 (102), CWE-284 (36), CWE-479 (18), CWE-762 (590), CWE-785 (51). Synthetic unit-test TPs: CWE-427 (phase_23_cwe427_setenv_fires), CWE-591 (phase_23_cwe591_virtualalloc_without_virtuallock_fires). 11 total phase_23 unit tests all pass. ANALYSIS.md documents D-11 corpus-mismatch rationale for CWE-427/591 |
| 3 | FP% for each new CWE is ≤40% where Juliet coverage exists (ROADMAP SC#3) | ? UNCERTAIN | CWE-114/272/284/479/785 all at 0.0% FP — meet the gate. CWE-427/591 at N/A (0 Juliet findings). CWE-762 at 58.5% FP exceeds the gate. ANALYSIS.md FP Gate Violations subsection documents this with cause and deferred action (Phase 24 tightening). Human decision required — see Human Verification section |
| 4 | No regression on existing 41 CWEs (ROADMAP SC#4) | ✓ VERIFIED | ANALYSIS.md Regression Check table shows all 41 prior CWE TP counts unchanged post-Phase-23. Benchmark total 214,558 → 217,279 (+2,721 from Phase 23 rules only). 423 unit tests pass (1 pre-existing pyspdxtools env failure unrelated) |
| 5 | benchmark/juliet/ANALYSIS.md updated with final 49-CWE coverage table (ROADMAP SC#5) | ✓ VERIFIED | ANALYSIS.md header: "AST scanner CWE coverage: 49 CWEs (updated Phase 23)". Per-CWE table contains rows for all 8 new CWEs (CWE-114, 272, 284, 427, 479, 591, 762, 785). Phase 23 Notes section present with Coverage, Expected zero-TP, FP Gate Violations, Regression Check (41 CWEs), and ROADMAP criteria subsections |

**Score:** 4/5 truths definitively verified (SC#1, SC#2, SC#4, SC#5); SC#3 requires human decision on CWE-762 FP gate.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/vulnerability/ast_scanner.rs` | 5 new AstCweRule entries (CWE-114/272/284/427/785) + 3 structural helpers + wiring | ✓ VERIFIED | Lines 234–242: 5 CWE entries present verbatim per plan spec. Lines 1476, 1573, 1640: apply_signal_handler_rules, apply_paired_lock_rules, apply_delete_rules. Lines 335–337: all 3 wired via findings.extend() in scan_file_ast_or_lexical(). Module doc updated to 49 CWEs with Win32-specific annotation (lines 3–14) |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | 11 new phase_23_* unit tests (6 from plan 01 + 5 from plan 02) | ✓ VERIFIED | All 11 functions present at lines 840, 851, 862, 873, 884, 895, 908, 927, 938, 949, 960. Both positive and negative test cases present for CWE-284, CWE-479, CWE-591 |
| `tests/fixtures/c/cwe762_delete_bad.c` | Namespace-free synthetic fixture with calloc+delete | ✓ VERIFIED | File exists; contains `delete p;` and `calloc(10, 1)`; no C++ namespace/class/template keywords |
| `benchmark/juliet/ast.json` | Regenerated with 49-CWE rule set | ✓ VERIFIED | File exists (1,738,233 lines); contains 1,299 findings for CWEs 114/272/284/479/785 alone; total 217,279 findings per ANALYSIS.md header |
| `benchmark/juliet/ANALYSIS.md` | 8 new per-CWE rows + Phase 23 Notes section + 49-CWE total | ✓ VERIFIED | All 8 CWE rows present in master table and Phase 23 per-CWE subtable; "Phase 23 Notes" section with Coverage, FP Gate Violations, Regression Check subsections; header shows 49 CWEs |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `AST_CWE_RULES` table entries | `apply_ast_rules()` AnyCall/ArgAtIndex arms | Existing table dispatch — no new arm code | ✓ WIRED | 5 new CWE entries confirmed at ast_scanner.rs lines 234–242; existing dispatch arms handle AnyCall and ArgAtIndex(4, &["GENERIC_ALL"]) |
| `scan_file_ast_or_lexical()` | `apply_signal_handler_rules` | `findings.extend(apply_signal_handler_rules(root, src, path, ...))` | ✓ WIRED | Line 335 confirmed |
| `scan_file_ast_or_lexical()` | `apply_paired_lock_rules` | `findings.extend(apply_paired_lock_rules(root, src, path, ...))` | ✓ WIRED | Line 336 confirmed |
| `scan_file_ast_or_lexical()` | `apply_delete_rules` | `findings.extend(apply_delete_rules(root, src, path, ...))` | ✓ WIRED | Line 337 confirmed |
| `apply_signal_handler_rules` pass 1 | pass 2 | `HashMap<String, u32>` handler_name → signal_call_line | ✓ WIRED | const NON_REENTRANT present at line 1463; cwe_id: 479 emitted at line 1502 at signal() call site line |
| `benchmark/juliet/ast.json` | `benchmark/juliet/ANALYSIS.md` | File-level oracle (CWE dir vs finding.cwe_id) | ✓ WIRED | ANALYSIS.md documents oracle method; Phase 23 per-CWE rows derived from oracle.sh run; 1,299 findings in ast.json for CWEs 114/272/284/479/785 match ANALYSIS.md TP counts |

### Data-Flow Trace (Level 4)

Not applicable — this phase adds static analysis scanner rules. The `SastFinding` structs are populated by the helper functions and flow through the existing pipeline unchanged. No rendering of dynamic user data.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 7 phase commits exist | `git log --oneline b3444d8 b4fabb3 d29abd6 30b797a b7be06d 53d4ce6 85764eb` | All 7 commits present | ✓ PASS |
| 5 new AstCweRule entries present | `grep -c 'cwe_id: 114\|cwe_id: 272\|cwe_id: 284\|cwe_id: 427\|cwe_id: 785' ast_scanner.rs` | 5 matches | ✓ PASS |
| 3 structural helper functions present | `grep 'fn apply_(signal_handler\|paired_lock\|delete)_rules' ast_scanner.rs` | 3 matches at lines 1476, 1573, 1640 | ✓ PASS |
| 11 phase_23 unit tests present | grep on test file | 11 matches | ✓ PASS |
| CWE-762 fixture exists with delete+calloc | file read | Contains `delete p;` and `calloc(10, 1)` | ✓ PASS |
| ast.json non-empty with Phase 23 CWE findings | `grep -c '"cwe_id"\s*:\s*(114\|272\|284\|479\|785)' ast.json` | 1,299 matches | ✓ PASS |
| ANALYSIS.md has 49-CWE count | `grep -E '\b49\b' ANALYSIS.md` | 4+ matches | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CWEXP-03 | 23-01, 23-02, 23-03 | Domain-specific CWE expansion (8 new CWEs, 41→49) | ✓ SATISFIED | All 8 CWEs implemented, tested, and benchmarked. ROADMAP Phase 23 success criteria 1, 2, 4, 5 met; SC#3 has one exception (CWE-762 58.5% FP documented via D-11) |
| (orphaned) | — | CWEXP-03 is referenced in all 3 plans and ROADMAP but is NOT defined in `.planning/REQUIREMENTS.md` | ⚠️ ORPHANED | REQUIREMENTS.md covers AST-01 through DIST-02 (v1.0.18 functional requirements) and XAST-01/02/03 (future). CWEXP-01/02/03 are milestone-internal CWE-expansion tracking IDs used by ROADMAP phases 21–23. Identical gap was noted in Phase 22 VERIFICATION. No functional code is missing — this is a documentation traceability gap only |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/vulnerability/ast_scanner.rs` | 8 | Module doc structural-helpers list includes CWE-617 twice (once in table-driven list line 6, once in structural helpers list line 8) | ℹ️ Info | Documentation cosmetic only; does not affect runtime behavior |
| `benchmark/juliet/ANALYSIS.md` | Overall Summary table | Row says "ast | 173,239 | 20,192 | 153,043 | 88.3% | 4" — this overall summary table was NOT updated to reflect Phase 23 total of 217,279 (only the Raw Finding Counts table and Phase 23 Notes section reflect the new total) | ⚠️ Warning | The Overall Summary table footnote says "ast totals updated in Phase 21" — the stale overall summary row could mislead readers, but the per-CWE master table and Phase 23 Notes section are authoritative and correct |

### Human Verification Required

### 1. CWE-762 FP Gate: 58.5% vs 40% ROADMAP Gate

**Test:** In `benchmark/juliet/ANALYSIS.md`, the Phase 23 per-CWE table shows CWE-762 at 58.5% FP (590 TPs, 832 FPs). ROADMAP SC#3: "FP% for each new CWE is ≤40% using file-level oracle where Juliet coverage exists."

The FP Gate Violations subsection in ANALYSIS.md documents:
- Root cause: `apply_delete_rules()` text-level byte scan fires on every `.cpp` file with `delete` keyword, regardless of CWE directory context. The oracle scores these as FPs because most Juliet files using `delete` are in non-CWE-762 directories.
- Recommended action: Phase 24 tightening (require co-occurrence with malloc/calloc/realloc in same file, or restrict to files without `namespace`).

CWE-762 is NOT a corpus-mismatch zero-TP case — it has 590 TPs. The D-11 exception applies to FP rate, not TP absence.

**Expected:** Developer confirms that D-11 policy ("ship and document; users suppress via --sarif-baseline") applies to this FP gate violation, that the documented rationale and Phase 24 deferral are acceptable, and that ROADMAP SC#3 is considered met with this exception. This mirrors the Phase 22 human checkpoint where 8 of 15 CWEs exceeded the 40% gate and were accepted.

**Why human:** CWE-762 has real Juliet coverage (590 TPs) so it cannot be accepted under the "corpus mismatch, unit-test-only TP" D-11 clause. The 58.5% FP exceeds a hard ROADMAP gate. The ANALYSIS.md FP Gate Violations section documents it correctly, but the ROADMAP SC#3 text does not contain an explicit D-11 bypass. Developer must explicitly accept this deviation or update the ROADMAP SC wording.

### Gaps Summary

No implementation gaps were found. All 8 CWEs are implemented and wired, all 11 unit tests are present, the Juliet benchmark has been re-run, and ANALYSIS.md has been updated with the 49-CWE table. The phase goal — "expand from 41 to 49 CWEs" — is substantively achieved in the codebase.

The single item requiring human decision is:

1. **CWE-762 FP gate** (58.5% > 40%) — D-11 exception documented in ANALYSIS.md; Phase 24 tightening deferred; needs explicit developer acceptance of the ROADMAP SC#3 deviation. (Same pattern as Phase 22 human checkpoints for 8/15 CWEs exceeding FP gate.)

The CWEXP-03 requirement traceability gap (ID not in REQUIREMENTS.md) is an inherited documentation issue identical to CWEXP-01/02 from earlier phases; it does not indicate missing implementation.

---

_Verified: 2026-05-12T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
