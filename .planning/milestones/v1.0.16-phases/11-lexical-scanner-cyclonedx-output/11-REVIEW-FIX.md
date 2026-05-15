---
phase: 11-lexical-scanner-cyclonedx-output
fixed_at: 2026-05-09T16:12:16Z
review_path: .planning/phases/11-lexical-scanner-cyclonedx-output/11-REVIEW.md
iteration: 1
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 11: Code Review Fix Report

**Fixed at:** 2026-05-09T16:12:16Z
**Source review:** .planning/phases/11-lexical-scanner-cyclonedx-output/11-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 5
- Fixed: 5
- Skipped: 0

## Fixed Issues

### WR-01: sprintf/vsprintf matched twice per call site

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** c1b432c
**Applied fix:** Removed `sprintf` and `vsprintf` from the CWE-120 rule. They remain in CWE-134 with the format-arg heuristic, which is the correct and sufficient coverage. Added an inline comment explaining the intentional omission.

---

### WR-02: sast_findings always runs even when --check-vulnerabilities is false

**Files modified:** `src/main.rs`
**Commit:** 5eb1c5b
**Applied fix:** Moved the `run_lexical_scanner` call inside the `if args.check_vulnerabilities` block (was positioned after the closing brace, running unconditionally within the `#[cfg(feature = "internal")]` block). The SAST scan now only executes when vulnerability checking is active.

---

### WR-03: Duplicate dep_to_bom_ref construction in convert_to_cyclonedx

**Files modified:** `src/formats/cyclonedx.rs`
**Commit:** b0103a7
**Applied fix:** Extracted the PURL-to-ecosystem normalization into two private `#[cfg(feature = "internal")]` helpers: `purl_to_dep_ecosystem(purl: &str) -> Option<String>` and `build_dep_bom_ref_map(components: &[CycloneDXComponent]) -> HashMap<(String, String), String>`. Both `build_cyclonedx_vulnerabilities` and `convert_to_cyclonedx` now call `build_dep_bom_ref_map`, eliminating ~44 lines of duplication.

---

### WR-04: scan_file silently swallows all I/O errors

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 43a43dc
**Applied fix:** Expanded the `Err(_) => continue` arm in the line-reading loop to emit `eprintln!("Warning: read error in {:?} at line {}: {}", path, line_idx + 1, e)` before continuing, consistent with the project's `warn_on_walkdir_err` diagnostic pattern.

---

### WR-05: Test name contradicts assertion — thirteen vs fourteen

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 4162c8a
**Applied fix:** Renamed `test_rule_table_has_thirteen_cwes` to `test_rule_table_has_fourteen_cwes`. Updated the doc comment to say "14 distinct CWE IDs (SCAN-02, updated)" and document CWE-126 as the Phase 11 addition. Also updated the `/// All 13 CWEs` static table comment to `/// All 14 CWEs`. The assertion value (14) was already correct.

---

_Fixed: 2026-05-09T16:12:16Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
