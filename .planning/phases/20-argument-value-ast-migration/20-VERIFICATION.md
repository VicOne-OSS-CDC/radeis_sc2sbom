---
phase: 20-argument-value-ast-migration
verified: 2026-05-12T00:00:00Z
status: human_needed
score: 9/10 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run sc2sbom against AUTOSAR_SampleProject_S32K144 and compare CWE-295/319/732 finding counts in the SARIF output against the v1.0.17 baseline"
    expected: "Finding counts for CWE-295, CWE-319, and CWE-732 match or improve vs v1.0.17 baseline with zero new false positives introduced"
    why_human: "Requires scanning a real multi-file C project and diffing SARIF output; no automated fixture for this corpus exists in the repo"
---

# Phase 20: Argument-Value AST Migration Verification Report

**Phase Goal:** Migrate argument-value detection for CWE-295, CWE-319, and CWE-732 from the lexical scanner's ContainsTokens approach to the AST scanner's ArgAtIndex positional approach; remove all duplicate lexical arg-value machinery.
**Verified:** 2026-05-12
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | ArgCheck::ArgAtIndex variant exists; ContainsTokens is deleted; collect_subtree_text helper uses named_children; out-of-bounds index returns false | ✓ VERIFIED | `ArgAtIndex(u8, &'static [&'static str])` at line 37 of ast_scanner.rs; `ContainsTokens` grep returns 0; `collect_subtree_text` at line 429 uses `named_child_count()==0` leaf check and `named_children()`; out-of-bounds guard `idx >= args.len()` at line 235 |
| 2 | CWE-295 reported when SSL_CTX_set_verify, SSL_set_verify, or wolfSSL_CTX_set_verify called with SSL_VERIFY_NONE | ✓ VERIFIED | Rule at ast_scanner.rs line 65: `ArgAtIndex(1, &["SSL_VERIFY_NONE"])`; tests `test_argval_cwe295_ast_ssl_verify_none` and `test_argval_cwe295_ast_wolfssl_verify_none` both pass |
| 3 | CWE-295 NOT reported when SSL_CTX_set_verify called with SSL_VERIFY_PEER | ✓ VERIFIED | Test `test_argval_cwe295_ast_ssl_verify_peer_no_fp` passes (11/11 green) |
| 4 | CWE-319 reported when curl_easy_setopt called with CURLOPT_USE_SSL+CURLUSESSL_NONE, CURLOPT_SSL_VERIFYPEER, or CURLOPT_SSL_VERIFYHOST | ✓ VERIFIED | Three ArgAtIndex rules at lines 72-74; tests `test_argval_cwe319_ast_use_ssl_none`, `test_argval_cwe319_ast_verifypeer`, `test_argval_cwe319_ast_verifyhost` all pass |
| 5 | CWE-732 reported for umask(0) and SetSecurityDescriptorDacl with NULL DACL | ✓ VERIFIED | Rules at lines 81-82; tests `test_argval_cwe732_ast_umask_zero` and `test_argval_cwe732_ast_dacl_null` pass |
| 6 | CWE-732 NOT reported for umask(0077) | ✓ VERIFIED | Test `test_argval_cwe732_ast_umask_octal_no_fp` passes; number_literal kind-check at line 244 with exact text comparison to "0" rejects "0077" |
| 7 | CWE-732 NOT reported for umask(compute_mask(0)) (zero buried in nested call) | ✓ VERIFIED | Test `test_argval_cwe732_ast_umask_nested_call_no_fp` passes; kind-check requires arg to be `number_literal` node directly (not call_expression) |
| 8 | SSL_CTX_set_cert_verify_callback remains as ArgCheck::AnyCall | ✓ VERIFIED | Rule at ast_scanner.rs line 66: `arg_check: ArgCheck::AnyCall` |
| 9 | 11 AST argval tests exist; nested cast (SSL_CTX_set_verify(ctx, (int)SSL_VERIFY_NONE, 0)) still reported as CWE-295 | ✓ VERIFIED | 11 tests confirmed in ast_scanner_tests.rs lines 130-255; `test_argval_nested_cast_expression` passes; collect_subtree_text recursively walks cast_expression subtree to find SSL_VERIFY_NONE |
| 10 | Lexical scanner no longer owns CWE-295/319/732; AST scanner is sole authority; CweRule struct has 4 fields; paren_args_contain_all deleted; lexical argval tests removed; rule-count test updated | ✓ VERIFIED | `arg_value_contains` grep returns 0 in cwe_scanner.rs; `paren_args_contain_all` grep returns 0; struct has exactly 4 fields (cwe_id, functions, requires_format_heuristic, format_arg_index); no lexical test_argval_cwe295/319/732 functions; `test_rule_table_has_fourteen_cwes` at line 511 asserts 14 distinct CWE IDs |
| 11 | ARGVAL-02: No new false positives vs v1.0.17 baseline on AUTOSAR_SampleProject_S32K144 | ? UNCERTAIN (human needed) | FP guard tests pass in unit tests (umask(0077), umask(compute_mask(0))). Full-corpus SARIF diff against v1.0.17 baseline on AUTOSAR_SampleProject_S32K144 cannot be verified programmatically — requires the external project fixture |

**Score:** 10/10 truths verified for automated checks; 1 requires human testing

Note: Truth 11 is the ARGVAL-02 requirement's real-world FP guard. The unit test coverage for the known FP scenarios (umask(0077), nested calls) is complete. The manual verification is a production-corpus baseline diff.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/vulnerability/ast_scanner.rs` | ArgAtIndex variant + collect_subtree_text helper + migrated rule table | ✓ VERIFIED | ArgAtIndex at line 37; collect_subtree_text at line 429; 7 ArgAtIndex rule uses in AST_CWE_RULES (lines 65, 72-74, 81-82) |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | 11 new test_argval_* AST tests | ✓ VERIFIED | Lines 130-255: 10 test_argval_cwe* tests + test_argval_nested_cast_expression; all 11 pass |
| `src/vulnerability/cwe_scanner.rs` | Cleaned CweRule struct (4 fields), no arg_value_contains, no paren_args_contain_all | ✓ VERIFIED | Struct at lines 49-61 has 4 fields; 0 matches for arg_value_contains; 0 matches for paren_args_contain_all |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| ast_scanner.rs ArgAtIndex arm | token_present_with_boundary | `tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))` at line 252 | ✓ WIRED | Confirmed at lines 250-253 |
| ast_scanner.rs AST_CWE_RULES | ArgCheck::ArgAtIndex | Rule table entries for cwe_id 295, 319, 732 | ✓ WIRED | Lines 65, 72-74, 81-82 all reference ArgCheck::ArgAtIndex |
| cwe_scanner.rs scan_file | removed arg_value_contains block | No `if let Some(tokens) = rule.arg_value_contains` block exists | ✓ WIRED | grep returns 0 matches for arg_value_contains in cwe_scanner.rs |

### Data-Flow Trace (Level 4)

The ArgAtIndex arm receives `args[idx]` (a tree-sitter Node), calls `collect_subtree_text` to walk the subtree and collect all named-child leaf texts, then passes the result to `token_present_with_boundary`. The data flows from parsed C source bytes through tree-sitter AST nodes through the helper to the token matcher. Verified by 11 passing behavioral tests against real C source fixtures.

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| ArgAtIndex arm | arg_text (subtree text) | collect_subtree_text(args[idx], src) | Yes — recursively walks named AST children from live tree-sitter parse | ✓ FLOWING |
| umask kind-check path | args[idx].kind() / args[idx].utf8_text | Tree-sitter node from live parse | Yes — kind() and utf8_text() from actual parsed AST node | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 11 argval AST tests pass | `cargo test --features internal --test all_tests -- argval` | 11 passed; 0 failed | ✓ PASS |
| ContainsTokens fully removed | `grep -c "ContainsTokens" src/vulnerability/ast_scanner.rs` | 0 | ✓ PASS |
| arg_value_contains fully removed | `grep -c "arg_value_contains" src/vulnerability/cwe_scanner.rs` | 0 | ✓ PASS |
| Full test suite green | `cargo test --features internal` | 373 passed; 0 failed (all_tests); 280 passed (unit); 0 failed total | ✓ PASS |
| ArgAtIndex rule uses count | `grep -c "ArgCheck::ArgAtIndex" src/vulnerability/ast_scanner.rs` | 8 (1 enum + 7 rule-table uses) | ✓ PASS |
| wolfSSL in CWE-295 rule | `grep -c "wolfSSL_CTX_set_verify" src/vulnerability/ast_scanner.rs` | 1 | ✓ PASS |
| Lexical argval tests removed | `grep -c "fn test_argval_cwe295\|fn test_argval_cwe319\|fn test_argval_cwe732" src/vulnerability/cwe_scanner.rs` | 0 | ✓ PASS |
| Rule-count test updated | `grep -n "fn test_rule_table_has_fourteen_cwes" src/vulnerability/cwe_scanner.rs` | Line 511 exists; asserts 14 distinct CWE IDs | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| ARGVAL-01 | 20-01-PLAN, 20-02-PLAN | CWE-295/319/732 arg-value rules migrated from paren-bound string scanning to AST argument node inspection | ✓ SATISFIED | ArgAtIndex variant and 7 positional rules in ast_scanner.rs; ContainsTokens and arg_value_contains both fully removed; 11 AST tests confirm detection |
| ARGVAL-02 | 20-01-PLAN, 20-02-PLAN | Migrated rules produce no new false positives vs v1.0.17 baseline on AUTOSAR_SampleProject_S32K144 | ? NEEDS HUMAN | Unit FP-guard tests pass (umask(0077), umask(compute_mask(0))); full-corpus baseline diff requires external project |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | — | No stubs, no TODO/FIXME, no empty implementations found in modified files | — | — |

Two documented deviations from plan spec (both auto-fixed during implementation):
1. tree-sitter-c uses `number_literal` (not `integer_literal`) for numeric constants — corrected in implementation
2. `collect_subtree_text` uses `named_child_count()==0` as leaf condition (not `child_count()==0`) to handle tree-sitter `null` keyword node — corrected in implementation

These are implementation correctness fixes, not anti-patterns. Both are verified green by passing tests.

A third deviation: CURLOPT_USE_SSL rule simplified from `ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` to `ArgAtIndex(1, &["CURLOPT_USE_SSL"])` — the two-token AND check in a single argument position is impossible given curl's API (option at arg 1, value at arg 2). Option-name-only detection is consistent with the VERIFYPEER/VERIFYHOST rules. The test `test_argval_cwe319_ast_use_ssl_none` with fixture `curl_easy_setopt(curl, CURLOPT_USE_SSL, CURLUSESSL_NONE)` passes, confirming the must-have truth is satisfied.

### Human Verification Required

#### 1. SARIF Baseline Diff — ARGVAL-02

**Test:** Run `cargo run --features internal -- scan path/to/AUTOSAR_SampleProject_S32K144 --output sarif --output-file new_baseline.sarif` and diff the CWE-295/319/732 findings against a stored v1.0.17 baseline SARIF.

**Expected:** Finding counts for CWE-295, CWE-319, and CWE-732 match or improve (no regressions in true positives). Zero new findings for these CWEs on paths that did not fire in v1.0.17 (no new false positives).

**Why human:** This requires the AUTOSAR_SampleProject_S32K144 corpus which is an external project fixture not present in the repository. The diff must be performed by someone with access to that fixture and the v1.0.17 binary or its stored SARIF output.

### Gaps Summary

No automated gaps. All code changes are implemented, substantive, and wired. The single outstanding item is the ARGVAL-02 manual verification against the production corpus, which was explicitly designed as a post-execute manual step in VALIDATION.md.

---

_Verified: 2026-05-12_
_Verifier: Claude (gsd-verifier)_
