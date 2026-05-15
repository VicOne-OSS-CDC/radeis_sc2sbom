---
phase: 12-static-analysis-report
fixed_at: 2026-05-10T00:00:00Z
review_path: .planning/phases/12-static-analysis-report/12-REVIEW.md
iteration: 1
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 12: Code Review Fix Report

**Fixed at:** 2026-05-10T00:00:00Z
**Source review:** .planning/phases/12-static-analysis-report/12-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 6
- Fixed: 6
- Skipped: 0

## Fixed Issues

### CR-01: Panic on empty chain in `render_dependency_chain`

**Files modified:** `src/formats/console.rs`
**Commit:** 28e0ecc
**Applied fix:** Added `if chain.is_empty() { return String::new(); }` guard at the top of `render_dependency_chain` before `chain.len() - 1` is evaluated, preventing usize underflow/panic on empty input.

### WR-01: `transitive_count` is overcounted in `save_console_report` summary

**Files modified:** `src/formats/console.rs`
**Commit:** b580ec7
**Applied fix:** Replaced the graph-only lookup (which skipped deps absent from the graph) with the fallback pattern from `print_summary_section`: use `&node.dependency` if found, else fall back to the original `d`. This ensures deps with no graph entry are attributed to the correct bucket (direct vs transitive) using their original flags rather than silently inflating `transitive_count`.

### WR-02: `direct_count` logic differs between summary and ecosystem detail sections

**Files modified:** `src/formats/console.rs`
**Commit:** e370e95
**Applied fix:** Aligned the summary section's `direct_count` logic to match the ecosystem detail section: `direct && !dev` (excluding dev-direct from direct). Also switched transitive to be tracked independently (with its own counter) rather than derived as `total - direct`, matching the ecosystem section's approach exactly.

### WR-03: `Unknown` severity vulnerabilities silently dropped in Tree output mode

**Files modified:** `src/formats/console.rs`
**Commit:** 7664ec2
**Applied fix:** Added `VulnerabilitySeverity::Unknown` to the `severities` vec in the `VulnerabilityOutputMode::Tree` branch so Unknown-severity vulns are displayed and the printed count matches the header count.

### WR-04: Component heading level collision in `save_static_analysis_report` Findings section

**Files modified:** `src/formats/console.rs`, `tests/format_tests/sast_report_tests.rs`
**Commit:** c8f1c78
**Applied fix:** Changed component headings from `## {}` to `### {}` and CWE headings from `### CWE-{}` to `#### CWE-{}` so they nest correctly under `## Findings`. Updated the two test assertions that checked for the old heading levels (`## libfoo` and `### CWE-120`) to the corrected levels (`### libfoo` and `#### CWE-120`).

### WR-05: Non-deterministic ecosystem output order in `save_console_report`

**Files modified:** `src/formats/console.rs`
**Commit:** e526c0c
**Applied fix:** Applied sorted-key iteration to both ecosystem loops: the ROS multi-package loop (around original line 1539) and the standard loop (around original line 1598). Each now collects `by_ecosystem.keys()` into a `Vec`, sorts it, and iterates in deterministic order.

---

_Fixed: 2026-05-10T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
