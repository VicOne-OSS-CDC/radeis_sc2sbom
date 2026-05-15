---
phase: 21-ast-cwes-anycall-argpattern-expansion
verified: 2026-05-12T20:00:00Z
status: passed
score: 4/4 roadmap success criteria verified (2 accepted deferrals)
overrides_applied: 2
override_reason: "SC#2 FP% tightening deferred to Phase 999.2 (backlog). SC#3 AUTOSAR regression deferred to Phase 22 Plan 04. Both deferrals accepted by developer 2026-05-12."
gaps:
  - truth: "FP% for each new CWE is ≤35% using the file-level oracle"
    status: failed
    reason: "8 of 12 new CWEs exceed the 35% FP gate: CWE-126 (94.8%), CWE-338 (99.9%), CWE-426 (100%), CWE-467 (65.4%), CWE-535 (50%), CWE-676 (100%), CWE-680 (97.5%), CWE-780 (95.3%). ROADMAP SC#2 has no 'or documented exception' language."
    artifacts:
      - path: "benchmark/juliet/ANALYSIS.md"
        issue: "Phase 21 Update section documents 8 EXCEEDS GATE entries with rationale and defers tightening to Phase 999.2 (BACKLOG). However SC#2 in ROADMAP.md is strict with no documented exception clause."
    missing:
      - "Either: ROADMAP SC#2 needs amendment to add an exception/deferred clause for the known high-FP AnyCall patterns, OR a decision is needed on whether Phase 999.2 backlog tracking counts as resolution of SC#2."
  - truth: "No regression on existing 13 CWEs — AUTOSAR fixture finding counts unchanged"
    status: failed
    reason: "ROADMAP SC#3 specifies AUTOSAR fixture as the regression target. Plan 03 only ran the Juliet corpus regression check (which shows 0% drift). No AUTOSAR fixture scan was executed or documented in Phase 21. New AnyCall rules for CWE-680 (malloc/calloc/realloc) and CWE-338 (rand) could produce new findings on AUTOSAR embedded C source."
    artifacts:
      - path: "benchmark/juliet/ANALYSIS.md"
        issue: "Regression check section covers Juliet only; AUTOSAR finding counts not mentioned."
    missing:
      - "Run sc2sbom on the AUTOSAR_SampleProject_S32K144 fixture before and after Phase 21 rules and document that existing 13 CWE finding counts are unchanged. Phase 22 Plan 04 already plans this check — confirm the deferred AUTOSAR check is acceptable for Phase 21 close."
deferred:
  - truth: "FP% tightening for CWE-126/338/426/467/535/676/680/780"
    addressed_in: "Phase 999.2"
    evidence: "Phase 999.2 (BACKLOG) goal: 'Audit and tighten the AnyCall/FixedSizeBuffer CWE rules added in phases 19-23 that exceed the 35% FP gate on the Juliet corpus. Target CWEs: CWE-126 (94.8%), CWE-338 (99.9%), CWE-426 (100%), CWE-467 (65.4%), CWE-535 (50%), CWE-676 (100%), CWE-680 (97.5%), CWE-780 (95.3%)'"
  - truth: "AUTOSAR fixture regression check for Phase 21 rules"
    addressed_in: "Phase 22"
    evidence: "Phase 22 Plan 04 must_have: 'AUTOSAR fixture regression check completed and recorded (D-15) — no existing CWE finding count changed'; 22-04-PLAN.md plans running AUTOSAR scan and recording regression evidence in ANALYSIS.md."
---

# Phase 21: ast-cwes-anycall-argpattern-expansion Verification Report

**Phase Goal:** Expand AST scanner from 13 to 25 CWEs by adding rules that require only call-site detection (AnyCall, ArgPattern, FixedSizeBuffer, OperatorPattern). All rules validated against Juliet ground truth with FP% tracked in benchmark/juliet/ANALYSIS.md.
**Verified:** 2026-05-12T20:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC#1 | All 12 new CWEs produce ≥1 TP on Juliet corpus (or synthetic-fixture unit test with documentation) | VERIFIED | ANALYSIS.md Phase 21 Update: CWE-328 (54 TPs), CWE-338 (36 TPs), CWE-467 (54 TPs), CWE-526 (18 TPs), CWE-535 (51 TPs), CWE-680 (546 TPs), CWE-780 (18 TPs from Juliet CryptEncrypt); CWE-121/369/426/676 via unit tests with corpus-gap documentation as permitted by SC#1 text |
| SC#2 | FP% for each new CWE is ≤35% using the file-level oracle | FAILED | 8 of 12 CWEs exceed gate: CWE-126 (94.8%), CWE-338 (99.9%), CWE-426 (100%), CWE-467 (65.4%), CWE-535 (50%), CWE-676 (100%), CWE-680 (97.5%), CWE-780 (95.3%). ROADMAP SC#2 has no exception clause. Tightening deferred to Phase 999.2 BACKLOG. |
| SC#3 | No regression on existing 13 CWEs — AUTOSAR fixture finding counts unchanged | FAILED | Phase 21 Plan 03 ran Juliet regression only (0% drift on all 13 CWEs confirmed). AUTOSAR fixture was NOT scanned. SC#3 explicitly specifies AUTOSAR. Phase 22 Plan 04 plans this check. |
| SC#4 | benchmark/juliet/ANALYSIS.md updated with new per-CWE TP/FP rows | VERIFIED | 12 new rows confirmed present: CWE-121, 126, 328, 338, 369, 426, 467, 526, 535, 676, 680, 780. Phase 21 Update section (## Phase 21 Update (2026-05-12)) present. |

**Score:** 2/4 roadmap success criteria verified (SC#1 and SC#4 pass; SC#2 and SC#3 fail)

Note: Despite 2/4 on roadmap SCs, the core goal (code expansion from 13→25 CWEs with tests) is fully implemented and working. The two failures are on benchmarking gates, not implementation correctness.

---

### Deferred Items

Items not yet met but explicitly addressed in later milestone phases.

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | FP% tightening for 8 high-FP CWEs (126, 338, 426, 467, 535, 676, 680, 780) | Phase 999.2 | Phase 999.2 BACKLOG goal explicitly targets these CWEs by FP% value |
| 2 | AUTOSAR fixture regression check for Phase 21's 25 CWEs | Phase 22 | Phase 22 Plan 04 must_have: "AUTOSAR fixture regression check completed and recorded (D-15)" |

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/vulnerability/ast_scanner.rs` | SizeofPointer variant + arm + pointer-scope collectors + apply_division_rules + 12 new rule entries | VERIFIED | All present: `ArgCheck::SizeofPointer` (line 46), `collect_function_scope_pointer_declarators` (line 681), `collect_file_scope_pointer_declarators` (line 688), `apply_division_rules` (line 337), `visit_binary_exprs` (line 348), 13 new AstCweRule entries in Phase 21 block |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | 12 unit tests for new CWEs (2 from Plan 01 + 10 from Plan 02) | VERIFIED | All 12 tests present and passing: `test_cwe_369_division_literal_zero`, `test_cwe_467_sizeof_pointer`, `test_cwe_121_anycall_alloca`, `test_cwe_126_fixed_size_buffer`, `test_cwe_328_weak_hash_argindex`, `test_cwe_338_weak_prng`, `test_cwe_426_untrusted_search_path`, `test_cwe_526_env_exposure`, `test_cwe_535_shell_error_stderr`, `test_cwe_676_dangerous_function_strtok`, `test_cwe_680_integer_overflow_alloc`, `test_cwe_780_rsa_no_oaep` |
| `benchmark/juliet/ast.json` | Regenerated from post-Phase-21 scanner | VERIFIED | File exists (60.7 MB); mtime 2026-05-12 18:32; 173,239 total findings per ANALYSIS.md; commit 5e7c69d |
| `benchmark/juliet/ANALYSIS.md` | Per-CWE TP/FP table with 12 new rows + Phase 21 Update section | VERIFIED | All 12 rows present; "## Phase 21 Update (2026-05-12)" section present with corpus-gap notes, regression check, ROADMAP SC status table; commit 79b3f6d |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `scan_file_ast_or_lexical` | `apply_division_rules` | second call after `apply_ast_rules`, appending into shared `findings` Vec | VERIFIED | Line 296: `apply_division_rules(tree.root_node(), code.as_bytes(), path, component_name, component_ecosystem, &mut findings)` |
| `ArgCheck::SizeofPointer arm` | `collect_function_scope_pointer_declarators` | scope lookup of pointer-typed identifier names | VERIFIED | Line 483: `ArgCheck::SizeofPointer` arm calls `collect_function_scope_pointer_declarators(fn_node, src)` |
| `AST_CWE_RULES new entries` | `ArgCheck::ArgAtIndex` (ALL-OF semantics) | CWE-328 (3 entries) and CWE-780 (2+1 entries) split due to ALL-OF token semantics | VERIFIED | 3 entries for CWE-328 (CALG_MD2, CALG_MD5, CALG_SHA1), 3 entries for CWE-780 (RSA_PKCS1_PADDING, RSA_NO_PADDING, CryptEncrypt) |
| `AST_CWE_RULES CWE-126 entry` | `ArgCheck::FixedSizeBuffer` | mirrors CWE-119/120/122/125 (D-06) | VERIFIED | Line 114-118: `cwe_id: 126, functions: &["strcat", "strncat"], arg_check: ArgCheck::FixedSizeBuffer` |

---

### Data-Flow Trace (Level 4)

Not applicable for this phase — the artifacts are rule tables and helper functions, not UI components or API endpoints rendering dynamic data. The behavioral correctness is verified through unit tests.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 12 Phase 21 unit tests pass | `cargo test --features internal` (Phase 21 tests) | All 12 tests ok | PASS |
| Phase 18 regression test unbroken | `test_ast_all_tractable_cwes` | ok | PASS |
| Full test suite (385 tests) | `cargo test --features internal` | 385 passed, 0 failed, 2 ignored | PASS |
| `ContainsTokens` removed (Phase 20 cleanup) | `grep -c ContainsTokens ast_scanner.rs` | 0 | PASS |
| CWE-605 absent (D-12 deferred) | `grep -c 'cwe_id: 605'` | 0 | PASS |
| CWE-369 not in AST_CWE_RULES (D-01) | `grep cwe_id: 369 ast_scanner.rs` | Only inside `visit_binary_exprs`, not in rule table | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CWEXP-01 | 21-01, 21-02, 21-03 PLANs | AST scanner CWE expansion (Phase 21) | ORPHANED in REQUIREMENTS.md | CWEXP-01 is referenced in ROADMAP.md (line 153) and all three PLANs, but does NOT appear in `.planning/REQUIREMENTS.md`. The CONTEXT.md acknowledges this: "REQUIREMENTS.md was last updated at Phase 20 milestone; Phase 21 CWE expansion is tracked in ROADMAP.md." The implementation against ROADMAP success criteria is what was verified. |

Note on CWEXP-01 orphan: This is a documentation gap in REQUIREMENTS.md, not an implementation gap. The requirement is tracked in ROADMAP.md and fully implemented. A follow-up to add CWEXP-01 to REQUIREMENTS.md is recommended.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/vulnerability/ast_scanner.rs` | 83-85 | Stale comment: `// NOTE: CWE-369 (Divide by Zero) is intentionally absent — AnyCall on div/ldiv/lldiv produces massive false positives... Deferred to lexical fallback only` | Info | Comment is from Phase 18 and contradicts lines 6-11 (module doc) which correctly states CWE-369 is now AST-detected via `apply_division_rules`. Comment should be removed. No functional impact. |

No other stubs, empty returns, TODOs, or placeholder patterns found in Phase 21 artifacts.

---

### Human Verification Required

None — all automated checks passed except for the two ROADMAP SC gaps which require developer decisions, not human testing.

---

## Gaps Summary

Two ROADMAP success criteria are not met:

**Gap 1: SC#2 — FP% gate (8 of 12 new CWEs exceed ≤35%)**

The ROADMAP states FP% ≤35% for each new CWE with no exception clause. 8 of 12 new CWEs exceed this gate: CWE-126, 338, 426, 467, 535, 676, 680, 780. The phase team accepted these with documented rationale and logged tightening to Phase 999.2 (BACKLOG). The ROADMAP SC as written is not met.

Developer decision needed: Either (a) amend the ROADMAP SC#2 to add an exception clause ("or documented exception with rationale deferred to Phase 999.2"), or (b) accept that Phase 21 is closed with SC#2 partially met given that Phase 999.2 tracks the remediation.

**Gap 2: SC#3 — AUTOSAR regression check**

The ROADMAP SC#3 specifically says "AUTOSAR fixture finding counts unchanged." Phase 21 Plan 03 ran Juliet corpus regression only (all 13 existing CWEs show 0% drift on Juliet). The AUTOSAR fixture at `example_target_repos/AUTOSAR_SampleProject_S32K144` was not scanned in Phase 21. New AnyCall rules (CWE-680 on malloc/calloc/realloc, CWE-338 on rand) could produce new findings on AUTOSAR embedded C code — this would not be a regression (new CWEs finding things is additive), but the finding counts for the pre-existing 13 CWEs must be confirmed unchanged.

Phase 22 Plan 04 explicitly plans this AUTOSAR regression check and will record it in ANALYSIS.md. Developer decision: confirm that the AUTOSAR regression check is deferred to Phase 22 and accept Phase 21 close on the Juliet evidence, or run the AUTOSAR check now.

Both gaps have clear deferred coverage in later phases. The implementation itself (code, tests, benchmark analysis) is complete and correct.

---

_Verified: 2026-05-12T20:00:00Z_
_Verifier: Claude (gsd-verifier)_
