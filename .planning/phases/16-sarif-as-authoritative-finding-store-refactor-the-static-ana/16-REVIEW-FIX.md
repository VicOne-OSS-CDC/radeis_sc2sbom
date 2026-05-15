---
phase: 16-sarif-as-authoritative-finding-store-refactor-the-static-ana
fixed_at: 2026-05-11T01:49:00Z
review_path: .planning/phases/16-sarif-as-authoritative-finding-store-refactor-the-static-ana/16-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 16: Code Review Fix Report

**Fixed at:** 2026-05-11T01:49:00Z
**Source review:** .planning/phases/16-sarif-as-authoritative-finding-store-refactor-the-static-ana/16-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (2 critical, 5 warnings; info findings excluded per fix_scope=critical_warning)
- Fixed: 7
- Skipped: 0

## Fixed Issues

### CR-01: `save_diff_sarif_report` passes `--sarif-output` path to `save_sarif_report` for the diff, which overwrites the full SARIF

**Files modified:** `src/formats/sarif.rs`
**Commit:** e0c6050
**Applied fix:** Changed `save_diff_sarif_report` to derive a sibling path by inserting `_diff` before the extension when `sarif_output` is `Some`. For example, `--sarif-output /tmp/out.sarif` now writes the diff to `/tmp/out_diff.sarif`. Previously both the full and diff SARIF were written to the same path, silently destroying the full-scan output.

---

### CR-02: `cwe_name()` returns `"Unknown CWE"` for Phase-13 CWEs (295, 319, 369, 732)

**Files modified:** `src/formats/console.rs`
**Commit:** adc1122
**Applied fix:** Added four missing arms to the `cwe_name()` match in `src/formats/console.rs`:
- `295` => `"Improper Certificate Validation"`
- `319` => `"Cleartext Transmission of Sensitive Information"`
- `369` => `"Divide By Zero"`
- `732` => `"Incorrect Permission Assignment for Critical Resource"`

---

### WR-01: `extract_baseline_fingerprints` only inspects `runs[0]` — multi-run SARIF baselines silently drop findings

**Files modified:** `src/formats/sarif.rs`
**Commit:** bd7cab7
**Applied fix:** Replaced the hard-coded `json["runs"][0]["results"]` access with a loop over all runs via `json["runs"].as_array()`. Fingerprints from all runs are now collected into the result set.

---

### WR-02: CI gate exits with code 1 even when `--sarif-baseline` path is missing or malformed

**Files modified:** `src/formats/sarif.rs`, `src/main.rs`, `tests/vulnerability_tests/sarif_fingerprint_tests.rs`, `tests/vulnerability_tests/sarif_baseline_tests.rs`
**Commit:** 06fbfee
**Applied fix:** Changed `extract_baseline_fingerprints` signature from `HashSet<String>` to `Result<HashSet<String>>`. Both call sites in `main.rs` (Console and All output branches) now use a `match` that skips `save_diff_sarif_report` and emits `"Warning: skipping baseline comparison: {e}"` when the baseline cannot be loaded, preventing the spurious exit(1). Updated test assertions: the two error-case tests now assert `is_err()`; the success-case tests add `.unwrap()`.

---

### WR-03: `normalize_path` silently drops path root when `..` steps above the root

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 258d4a9
**Applied fix:** Guarded `out.pop()` with an `if !out.pop()` check and pushed `".."` back when already at root, matching OS behavior. Previously a no-op pop would silently produce a wrong path, causing suppression misses for paths with excess `..` segments.

---

### WR-04: Summary count understated when component dirs don't exist or have non-UTF-8 paths

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** d8955fc
**Applied fix:** Added a `missing_components` counter incremented for both `!dir.exists()` and non-UTF-8 path early-continues. The summary message now reports `({N} skipped due to unexpected exit, {M} skipped due to missing/non-UTF-8 dir)` when either counter is non-zero.

---

### WR-05: Suppression test `no_cppcheck_no_suppression` tests the wrong invariant

**Files modified:** `tests/vulnerability_tests/suppression_tests.rs`
**Commit:** a1fab54
**Applied fix:** Renamed `no_cppcheck_no_suppression` to `empty_scanned_dirs_returns_input_unchanged` and added a clarifying comment explaining the distinction: this test exercises the early-return guard (empty `scanned_dirs`), not the "cppcheck binary unavailable" scenario. Points reader to `suppress_lexical_cwe_covered_not_confirmed` for the complementary case.

---

_Fixed: 2026-05-11T01:49:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
