---
phase: 13-argument-value-matching
fixed_at: 2026-05-10T00:00:00Z
review_path: .planning/phases/13-argument-value-matching/13-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 13: Code Review Fix Report

**Fixed at:** 2026-05-10
**Source review:** .planning/phases/13-argument-value-matching/13-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01, WR-02, WR-03, WR-04)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01 + WR-02: Digit token boundary checks in `token_present_with_boundary`

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** e7ab60c
**Applied fix:** Both boundary defects were fixed together since they are adjacent code in the same function.

- Left boundary: added `|| prev == b'.'` to the digit-token branch so that the trailing `0` in `0.0` is not matched when preceded by `.`.
- Right boundary: extended the digit-token exclusion set from `{digit, '.'}` to also exclude hex/suffix letter bytes `x`, `X`, `b`, `B`, `o`, `O`, `l`, `L`, `u`, `U`. This prevents `0x1`, `0L`, `0u`, `0UL` etc. from falsely matching the token `"0"`.
- Updated the doc comment on `token_present_with_boundary` to document both boundary rules.

### WR-03: String-literal guard in `contains_div_by_zero`

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 37be9be
**Applied fix:** Before treating a `/` or `%` operator as a potential divide-by-zero, count the number of unescaped `"` characters that appear before it on the same line. If the count is odd the operator sits inside a string literal and the match is skipped. Backslash escapes (`\"`) are handled by counting consecutive preceding backslashes and only toggling quote_count when the backslash count is even. This eliminates the most common false-positive category (log message strings containing `/ 0`) while preserving detection in actual code. The updated doc comment explains the heuristic and notes the existing design choice to not strip C/C++ line comments.

### WR-04: Sort `all_findings` before dedup for deterministic output

**Files modified:** `src/vulnerability/cwe_scanner.rs`
**Commit:** 688906a
**Applied fix:** Added `all_findings.sort_by(...)` on the tuple `(file_path, line, cwe_id, component_name)` immediately before the `HashSet`-based dedup. This ensures that when multiple components map to overlapping directories, the first occurrence kept by the dedup is always the lexicographically smallest `component_name`, producing reproducible SBOM output regardless of `HashMap` iteration order.

---

_Fixed: 2026-05-10_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
