---
phase: 22
slug: ast-cwes-structuralpattern-expansion
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-12
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + Juliet benchmark script |
| **Config file** | Cargo.toml (feature = "internal") |
| **Quick run command** | `cargo test --features internal 2>&1 \| tail -5` |
| **Full suite command** | `cargo test --features internal && cargo build --features internal --release 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal 2>&1 | tail -5`
- **After every plan wave:** Run full suite + Juliet benchmark for new CWEs
- **Before `/gsd-verify-work`:** Full suite must be green + ANALYSIS.md updated
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 22-01-01 | 01 | 1 | CWEXP-02 | — | Failing unit tests for CWE-478/484/481/482/480/483 exist before implementation | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests::test_cwe478 2>&1 \| tail -5` | ❌ W0 | ⬜ pending |
| 22-01-02 | 01 | 1 | CWEXP-02 | — | 5 check_* functions exist and are called from apply_ast_rules | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests::test_cwe478 vulnerability_tests::ast_scanner_tests::test_cwe484 vulnerability_tests::ast_scanner_tests::test_cwe483 2>&1 \| tail -10` | ❌ W0 | ⬜ pending |
| 22-01-03 | 01 | 1 | CWEXP-02 | — | Full suite passes with Group A CWEs implemented | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests 2>&1 \| tail -15` | ✅ | ⬜ pending |
| 22-02-01 | 02 | 2 | CWEXP-02 | — | Failing unit tests for CWE-562/570/571/587 exist | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests::test_cwe562 2>&1 \| tail -5` | ❌ W0 | ⬜ pending |
| 22-02-02 | 02 | 2 | CWEXP-02 | — | check_return_stack_address, check_constant_condition, check_fixed_address_assignment functions implemented | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests::test_cwe562 vulnerability_tests::ast_scanner_tests::test_cwe570 vulnerability_tests::ast_scanner_tests::test_cwe587 2>&1 \| tail -10` | ❌ W0 | ⬜ pending |
| 22-02-03 | 02 | 2 | CWEXP-02 | — | Full suite passes; Plan 01 tests still green | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests 2>&1 \| tail -15` | ✅ | ⬜ pending |
| 22-03-01 | 03 | 3 | CWEXP-02 | — | Failing unit tests for CWE-617/674/256/835/398 exist | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests::test_cwe674 2>&1 \| tail -5` | ❌ W0 | ⬜ pending |
| 22-03-02 | 03 | 3 | CWEXP-02 | — | CWE-617 in AST_CWE_RULES; check_self_recursion/check_plaintext_password/check_infinite_loop/check_poor_code_quality implemented | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests::test_cwe617 vulnerability_tests::ast_scanner_tests::test_cwe674 vulnerability_tests::ast_scanner_tests::test_cwe256 2>&1 \| tail -10` | ❌ W0 | ⬜ pending |
| 22-03-03 | 03 | 3 | CWEXP-02 | — | All 15 Phase 22 CWEs implemented; full suite passes | unit | `cargo test --features internal vulnerability_tests::ast_scanner_tests 2>&1 \| tail -15` | ✅ | ⬜ pending |
| 22-04-01 | 04 | 4 | CWEXP-02 | — | Juliet benchmark run produces per-CWE TP/FP counts for all 15 CWEs | benchmark | `ls benchmark/juliet/results/ && grep -c "CWE-" benchmark/juliet/results/phase22_counts.txt` | ✅ | ⬜ pending |
| 22-04-02 | 04 | 4 | CWEXP-02 | — | ANALYSIS.md updated with 15 new per-CWE TP/FP rows | manual | `grep -c "CWE-256\|CWE-398\|CWE-478\|CWE-480\|CWE-481\|CWE-482\|CWE-483\|CWE-484\|CWE-562\|CWE-570\|CWE-571\|CWE-587\|CWE-617\|CWE-674\|CWE-835" benchmark/juliet/ANALYSIS.md` | ✅ | ⬜ pending |
| 22-04-03 | 04 | 4 | CWEXP-02 | — | AUTOSAR regression recorded; no existing CWE finding count changed | regression | `grep "D-15\|AUTOSAR regression" benchmark/juliet/ANALYSIS.md` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test framework installation needed — the Juliet benchmark harness and `cargo test` suite exist from prior phases.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ANALYSIS.md TP/FP rows for 15 new CWEs | CWEXP-02 | Requires human review of benchmark output | Run Juliet benchmark, count TP/FP per CWE, verify each row exists in ANALYSIS.md |
| FP% ≤40% goal for each CWE | CWEXP-02 | Metric requires benchmark run + calculation | Compare TP/FP counts in ANALYSIS.md; flag any CWE exceeding 40% FP for documentation |
| No regression on AUTOSAR fixtures | CWEXP-02 | Requires diffing fixture finding counts | Run AUTOSAR scan before/after; compare finding counts per rule |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
