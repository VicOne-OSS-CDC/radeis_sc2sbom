---
quick_id: 260513-fsg
status: complete
date: 2026-05-13
commit: 8532e4e
---

# Quick Task 260513-fsg: Drop CWE-20 and CWE-22

## What was done

Removed the CWE-20 (`atoi`/`strtol`/`atof` family) and CWE-22 (`realpath`/`getcwd`/`open`/`fopen` family) rules from `CWE_RULES` in `src/vulnerability/cwe_scanner.rs`.

Both rules produced 0 true positives and 100% false positives on the Juliet corpus. Correct detection requires taint/dataflow analysis — not achievable with a local lexical rule.

## Files changed

- `src/vulnerability/cwe_scanner.rs` — removed 2 `CweRule` entries; updated distinct CWE count 14 → 12
- `tests/vulnerability_tests/cwe_scanner_tests.rs` — renamed test functions; updated count assertions; removed `20u32, 22` from expected CWE array
- `tests/fixtures/c/dangerous_calls.c` — removed `atoi` and `realpath` call lines
- `src/vulnerability/ast_scanner.rs` — updated module-level doc comment: 49 → 48 CWEs (reflects Phase 24 CWE-256 removal)
- `benchmark/juliet/ANALYSIS.md` — appended Ad-hoc Drops section documenting the removal

## Test result

All 297 tests pass. No regressions.

## Self-Check: PASSED
