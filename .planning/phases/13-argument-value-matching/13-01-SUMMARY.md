---
phase: 13-argument-value-matching
plan: 01
subsystem: vulnerability/cwe_scanner
tags: [sast, cwe, argument-value-matching, rust, tdd]
dependency_graph:
  requires: []
  provides: [CWE-295-detection, CWE-319-detection, CWE-732-detection, CWE-369-detection, finding-dedup]
  affects: [run_lexical_scanner, scan_file, CweRule, CWE_RULES]
tech_stack:
  added: []
  patterns: [word-boundary-byte-scan, AND-all-token-matching, HashSet-dedup]
key_files:
  created: []
  modified:
    - src/vulnerability/cwe_scanner.rs
decisions:
  - "D-01: arg_value_contains: Option<&'static [&'static str]> added to CweRule; all 15 existing rules get None"
  - "D-02: AND-all semantics — all tokens in arg_value_contains must appear in paren-bound args"
  - "D-03: Dedup by (file, line, cwe) in run_lexical_scanner via HashSet retain"
  - "D-04: paren_args_contain_all tracks paren depth for nested calls"
  - "D-05: Digit-only tokens require non-digit, non-dot right boundary (umask(0077) does not fire)"
  - "D-09: CWE-369 separate code path, not a CweRule entry; byte scan for /|% followed by standalone 0"
metrics:
  duration_minutes: 12
  completed_date: "2026-05-10"
  tasks_completed: 3
  files_modified: 1
---

# Phase 13 Plan 01: Argument-Value Matching Summary

**One-liner:** Extended CWE lexical scanner with AND-all paren-arg token matching for CWE-295 (TLS verify disabled), CWE-319 (insecure curl options), CWE-732 (permissive umask/DACL), and a separate byte-scan path for CWE-369 (division/modulo by literal zero), plus HashSet deduplication of findings.

## Objective Achieved

Single file modified — `src/vulnerability/cwe_scanner.rs` — covering ARGVAL-01 through ARGVAL-05 with 15 new inline unit tests (14 ARGVAL tests + 1 dedup test) proving each requirement and backward compatibility for all 8 pre-existing scanner tests.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add failing tests (TDD RED) | 5bfc964 | src/vulnerability/cwe_scanner.rs |
| 2 | Extend CweRule, add helpers, wire arg_value_contains | ae9fc82 | src/vulnerability/cwe_scanner.rs |
| 3 | Add CWE-369 scan path and HashSet dedup | 6169846 | src/vulnerability/cwe_scanner.rs |

## Metrics

- **Pre-existing tests:** 8 (all pass, unchanged)
- **New tests added:** 15 (14 ARGVAL tests + 1 dedup test)
- **Total tests in cwe_scanner module:** 23 (all green)
- **Distinct CWE IDs detected:** 18 (existing 14 + CWE-295 + CWE-319 + CWE-732 + CWE-369)
- **CWE_RULES table size:** 22 entries (15 existing + 7 new Phase 13 entries)
- **Lines added to cwe_scanner.rs:** ~298 (303 → 601 lines)
- **No new external dependencies** — Cargo.toml unchanged

## Requirements Covered

| Requirement | Test | Status |
|------------|------|--------|
| ARGVAL-01: CWE-295 TLS verify disabled | test_argval_cwe295_ssl_verify_none, test_argval_cwe295_secure_does_not_fire | PASS |
| ARGVAL-02: CWE-319 insecure curl options | test_argval_cwe319_curl_verifypeer_zero, test_argval_cwe319_curl_use_ssl_none, test_argval_cwe319_curl_verifyhost_one, test_argval_cwe319_secure_does_not_fire | PASS |
| ARGVAL-03: CWE-732 permissive umask/DACL | test_argval_cwe732_umask_zero, test_argval_cwe732_umask_octal_does_not_fire, test_argval_cwe732_setsd_dacl_null | PASS |
| ARGVAL-04: CWE-369 division-by-literal-zero | test_argval_cwe369_div_by_zero, test_argval_cwe369_modulo_by_zero, test_argval_cwe369_zero_in_number_does_not_fire, test_argval_cwe369_decimal_zero_does_not_fire | PASS |
| ARGVAL-05: Name-only rules backward compat | test_argval_05_name_only_rule_unaffected, all 8 pre-existing tests | PASS |

## Deviations from Plan

None — plan executed exactly as written.

All implementation decisions D-01 through D-15 implemented as specified in 13-CONTEXT.md.

The plan's RED phase expected "15 failures" but the "does not fire" negative-assertion tests (test_argval_cwe295_secure_does_not_fire, test_argval_cwe319_secure_does_not_fire, test_argval_cwe732_umask_octal_does_not_fire, test_argval_cwe369_zero_in_number_does_not_fire, test_argval_cwe369_decimal_zero_does_not_fire) correctly passed vacuously in RED state (no rules existed yet, so no findings were generated). This is the correct behavior for negative-assertion tests and was acknowledged as acceptable — the plan's "15 failures" count was an overestimate.

## Known Stubs

None.

## Threat Flags

No new threat surface introduced. All changes are within the existing `#![cfg(feature = "internal")]` gate, touching only `cwe_scanner.rs`. Token matching uses compile-time `&'static str` constants — no user-controlled patterns (T-13-01 mitigated). CWE-369 false positives from `// divide by 0` comments are an accepted design tradeoff (T-13-04 accepted per 13-RESEARCH.md Pitfall 3).

## Self-Check: PASSED

- [x] src/vulnerability/cwe_scanner.rs exists and is 601 lines
- [x] Commit 5bfc964 exists (TDD RED — failing tests)
- [x] Commit ae9fc82 exists (TDD GREEN — CWE-295/319/732 rules)
- [x] Commit 6169846 exists (TDD GREEN — CWE-369 + dedup)
- [x] cargo test --features internal passes (23/23 in cwe_scanner module, full suite green)
- [x] cargo build -p radeis_sc2sbom (no --features internal) exits 0
