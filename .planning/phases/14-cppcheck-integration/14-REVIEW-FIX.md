---
phase: 14-cppcheck-integration
fixed_at: 2026-05-10T12:30:00Z
review_path: .planning/phases/14-cppcheck-integration/14-REVIEW.md
iteration: 2
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 14: Code Review Fix Report

**Fixed at:** 2026-05-10T12:30:00Z
**Source review:** .planning/phases/14-cppcheck-integration/14-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 4
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: SAST scan silently skipped when `--check-vulnerabilities` is not set

**Files modified:** `src/main.rs`
**Commit:** 7ec10aa
**Applied fix:** Added a `#[cfg(feature = "internal")]` guard block in `main.rs` (after the `sast_findings` declaration) that emits `eprintln!("Warning: --cppcheck-path has no effect without --check-vulnerabilities")` when `args.cppcheck_path.is_some() && !args.check_vulnerabilities`.

---

### WR-02: `completed_components` counts unexpected-exit components as completed

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 09a3ff3
**Applied fix:** Added a `skipped_components: usize` counter. In the `Ok(out)` match arm, when cppcheck exits with a code other than 0 or 1, the code now increments `skipped_components` and calls `continue` before XML parsing, so partial/corrupt output is never parsed. `completed_components += 1` is only reached after successful processing. The summary `eprintln!` now includes the skipped count when nonzero.

---

### WR-03: `CURLOPT_SSL_VERIFYHOST = 1` false-positive CWE-319 rule

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** f6ae2b4
**Applied fix:** Removed the `CweRule` entry for `CURLOPT_SSL_VERIFYHOST` with value `"1"` from the `CWE_RULES` static array. Removed the companion test `test_argval_cwe319_curl_verifyhost_one` that validated the false-positive behavior. The `"0"` rule for `CURLOPT_SSL_VERIFYHOST` (the genuinely insecure value) is retained.

---

### WR-04: `deduplicate_sast_findings` uses `Path::canonicalize` which fails for absent paths

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 7b91454
**Applied fix:** Added a private `normalize_path(s: &str) -> String` helper that resolves `.` and `..` path components using `Path::components()` without any filesystem access. Replaced both `canonicalize().unwrap_or_else(...)` calls in `deduplicate_sast_findings` with `normalize_path(&f.file_path)`. Updated the function doc comment to reflect the new approach.

---

_Fixed: 2026-05-10T12:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 2_
