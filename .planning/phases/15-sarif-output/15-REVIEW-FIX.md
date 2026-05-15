---
phase: 15-sarif-output
fixed_at: 2026-05-10T00:00:00Z
review_path: .planning/phases/15-sarif-output/15-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 15: Code Review Fix Report

**Fixed at:** 2026-05-10T00:00:00Z
**Source review:** .planning/phases/15-sarif-output/15-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3
- Fixed: 3
- Skipped: 0

## Fixed Issues

### WR-01: `--sarif-output` silently ignored for SpdxJson / SpdxTagValue / CyclonedxJson formats

**Files modified:** `src/main.rs`
**Commit:** 2100ea2
**Applied fix:** Added `#[cfg(feature = "internal")] if args.sarif_output.is_some()` warning blocks at the top of each of the three non-SARIF format arms (SpdxJson, SpdxTagValue, CyclonedxJson). Each arm now emits a clear `Warning: --sarif-output has no effect with --format <name>` message to stderr when the user passes the flag with a format that does not produce SARIF output.

---

### WR-02: SARIF `artifactLocation.uri` emits raw OS paths instead of URI-scheme paths

**Files modified:** `src/formats/sarif.rs`
**Commit:** 7e32a5e
**Applied fix:** Replaced the direct `uri: f.file_path.clone()` assignment with a block that checks `Path::new(&f.file_path).is_absolute()`. Absolute paths are now emitted as `file://<path>` (with backslashes normalized to forward slashes for Windows compatibility); relative paths are passed through unchanged as they are already valid URIs per the SARIF 2.1 spec.

---

### WR-03: Inconsistent `direct_count` between `print_summary_section` and `save_console_report`

**Files modified:** `src/formats/console.rs`
**Commit:** 685f51d
**Applied fix:** Changed the counting block in `print_summary_section` (around line 947) to match the guard already present in `save_console_report`: dependencies that are both `is_direct` and `is_dev` now contribute only to `dev_count`, not to `direct_count`. Both code paths now agree: `direct_count` counts direct non-dev dependencies only.

---

_Fixed: 2026-05-10T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
