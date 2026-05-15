# Juliet Benchmark — AST Scanner

Fixture: `example_target_repos/juliet-test-suite-c`  
Last updated: 2026-05-13 (Phase 25 / v1.0.18)

---

## Before and After v1.0.18

v1.0.18 (Phases 18–24) introduced the tree-sitter AST scanner and ran a full tuning pass (Phase 24) against this corpus. Phase 25 split the rule set into 22 high-confidence defaults and 17 experimental CWEs behind `--experiment-scan`.

| | Pre-v1.0.18 (Phase 23) | **Default mode** (22 CWEs) | **Experimental mode** (all 39 CWEs) |
|---|---|---|---|
| Total AST findings | 217,279 | **22,939** | **127,800** |
| True positives | — | **17,895** | **22,701** |
| False positives | — | **5,044** | **105,099** |
| FP% | ~88.3% | **22.0%** | **82.2%** |
| Net FP reduction | — | **−194,340** vs Phase 23 | **−89,479** vs Phase 23 |


---

## Quality Tiers (Post-v1.0.18)

> **Phase 25 experiment-scan split:** The 17 Experimental CWEs (🟡 Marginal, 🔴 Poor,
> and the 4 ⚪ No-signal-unconfirmed CWEs) now require `--experiment-scan` to activate.
> Default scan runs only the 22 high-confidence CWEs (✅ Clean + 🟢 Good +

| Tier | Threshold | CWEs | Count |
|------|-----------|------|-------|


**20 of 48 covered CWEs are Clean or Good — these are the most actionable signals.**

---



|-----|-----|-----|-----|------|-----------------|-----------|
| **TOTAL** | **22,701** | **105,099** | **82.2%** | | | |

---

## Phase 24 Top Changes by FP Reduction

|-----|----------|----------|-----------|--------------|
| **TOTAL** | 88.3% | **82.2%** | **−89,479** | |


---

## Scanner Comparison

| Scanner | Total findings | TPs | FPs | FP% |
|---------|---------------|-----|-----|-----|
| AST default (v1.0.18 / Phase 25, 22 CWEs) | 22,939 | 17,895 | 5,044 | **22.0%** |
| AST experimental (v1.0.18 / Phase 25, 39 CWEs) | 127,800 | 22,701 | 105,099 | 82.2% |
| cppcheck (Phase 23 baseline) | 204,058 | 3,559 | 200,496 | 98.3% |

**Best unique signals by scanner:**

|-----|-------------|-----|-------|

---

## AUTOSAR Regression (v1.0.18 / Phase 24)

Fixture: `AUTOSAR_SampleProject_S32K144`  
Test: `cargo test --features internal --test autosar_ast_regression`

|-----|----------|----------|--------|
| **Total** | **3** | **3** | **PASS** |

---

## Poor-Tier Disposition

These 12 CWEs remain > 75% FP. Each has an accepted rationale.

|-----|-----|-------------------------|--------------------|

---

## Oracle Limitations

3. **Good/bad function pattern**: ~50% of lines in each Juliet file are intentionally safe variants; scanners firing on all calls have inflated counts.

Re-run oracle: `./benchmark/juliet/oracle.sh` (reproducible).
