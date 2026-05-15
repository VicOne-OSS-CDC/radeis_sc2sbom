---
phase: 13-argument-value-matching
verified: 2026-05-10T00:00:00Z
status: passed
score: 6/6 must-haves verified
overrides_applied: 0
---

# Phase 13: Argument-Value Matching Verification Report

**Phase Goal:** Implement argument-value matching for CWE-295, CWE-319, CWE-732, CWE-369 and add finding deduplication
**Verified:** 2026-05-10
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Scanning C source with `SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL)` produces a CWE-295 SastFinding | VERIFIED | `test_argval_cwe295_ssl_verify_none` passes (ok); CWE-295 rule at cwe_scanner.rs line 74 with `arg_value_contains: Some(&["SSL_VERIFY_NONE"])` |
| 2 | Scanning C source with `curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0)` produces a CWE-319 SastFinding | VERIFIED | `test_argval_cwe319_curl_verifypeer_zero` passes (ok); CWE-319 rule at line 77 matches `CURLOPT_SSL_VERIFYPEER` + `0` |
| 3 | Scanning C source with `umask(0)` produces a CWE-732 SastFinding, but `umask(0077)` does NOT | VERIFIED | `test_argval_cwe732_umask_zero` and `test_argval_cwe732_umask_octal_does_not_fire` both pass; `token_present_with_boundary` enforces D-05 digit boundary so `0077` does not match token `"0"` |
| 4 | Scanning C source with `x = a / 0;` produces a CWE-369 SastFinding | VERIFIED | `test_argval_cwe369_div_by_zero` passes (ok); `contains_div_by_zero` helper at lines 222-244 fires on `/` or `%` followed by standalone `0` |
| 5 | Existing 14-CWE rules (no arg_value_contains) still fire on name match alone — all pre-Phase-13 tests pass | VERIFIED | `test_argval_05_name_only_rule_unaffected` passes; all 8 pre-existing tests pass (fallback_*, test_format_arg_is_literal_*, test_find_function_call_*); 15 existing `CweRule` entries each carry `arg_value_contains: None` |
| 6 | Identical (file, line, cwe) findings appear at most once in run_lexical_scanner output | VERIFIED | `test_run_lexical_scanner_dedups_by_file_line_cwe` passes; `run_lexical_scanner` uses `HashSet<(String, u32, u32)>` + `all_findings.retain` at lines 379-380 |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/vulnerability/cwe_scanner.rs` | Extended CweRule struct, paren_args_contain_all helper, CWE-295/319/732 rules, CWE-369 scan path, dedup in run_lexical_scanner, updated tests | VERIFIED | File is 601 lines; all components present and substantive |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `scan_file` rule loop | `paren_args_contain_all` | `rule.arg_value_contains.is_some()` branch | VERIFIED | Lines 307-311: `if let Some(tokens) = rule.arg_value_contains { ... if !paren_args_contain_all(after, tokens) { continue; } }` |
| `scan_file` | `contains_div_by_zero` | post-rule-loop per-line CWE-369 check | VERIFIED | Lines 323-332: `if contains_div_by_zero(&line)` inside `reader.lines().enumerate()` loop |
| `run_lexical_scanner` | HashSet dedup | `all_findings.retain` on `(file, line, cwe)` | VERIFIED | Lines 379-380: `let mut seen: HashSet<(String, u32, u32)> = HashSet::new(); all_findings.retain(|f| seen.insert(...))` |

### Data-Flow Trace (Level 4)

Not applicable — this phase modifies a scanner library, not a rendering component. Output is `Vec<SastFinding>` consumed by the existing CycloneDX pipeline unchanged.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 23 cwe_scanner module tests | `cargo test --features internal -p radeis_sc2sbom vulnerability::cwe_scanner` | 23 passed; 0 failed | PASS |
| Full internal test suite | `cargo test --features internal -p radeis_sc2sbom` | All pass (no failures) | PASS |
| Public binary builds without scanner code | `cargo build -p radeis_sc2sbom` | exit 0 | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| ARGVAL-01 | 13-01-PLAN.md | CWE-295 when SSL_CTX_set_verify/SSL_set_verify/wolfSSL_CTX_set_verify called with SSL_VERIFY_NONE | SATISFIED | CWE_RULES line 74; tests test_argval_cwe295_ssl_verify_none and test_argval_cwe295_secure_does_not_fire pass |
| ARGVAL-02 | 13-01-PLAN.md | CWE-319 when curl_easy_setopt called with insecure option combos | SATISFIED | CWE_RULES lines 76-79 (4 rules); tests test_argval_cwe319_* all pass |
| ARGVAL-03 | 13-01-PLAN.md | CWE-732 via umask and SetSecurityDescriptorDacl with permissive DACL | SATISFIED | CWE_RULES lines 81-82; tests test_argval_cwe732_* all pass |
| ARGVAL-04 | 13-01-PLAN.md | CWE-369 when literal /0 or %0 found in division expression | SATISFIED | contains_div_by_zero helper; tests test_argval_cwe369_* all pass |
| ARGVAL-05 | 13-01-PLAN.md | CweRule struct supports optional arg_value_contains; name-only rules unaffected | SATISFIED | struct field at line 48; 15 existing rules carry `None`; test_argval_05_name_only_rule_unaffected passes |

No orphaned requirements: REQUIREMENTS.md traceability table shows ARGVAL-01 through ARGVAL-05 as Phase 13, all marked Complete. No additional Phase 13 IDs exist in REQUIREMENTS.md.

### Anti-Patterns Found

No blockers or warnings. A scan of the modified file found:

- No TODO/FIXME/placeholder comments in implementation code
- No empty return values (`return null`, `return []`) in functional paths
- No stub handlers
- `contains_div_by_zero` has an acknowledged false-positive for C comment lines like `// divide by 0` — this is documented as an accepted design tradeoff (T-13-04, Pitfall 3 in 13-RESEARCH.md) consistent with the scanner's existing philosophy

### Human Verification Required

None. All phase deliverables are fully verifiable via automated tests.

### Gaps Summary

No gaps. All 6 must-have truths are verified. All 5 requirement IDs (ARGVAL-01 through ARGVAL-05) are satisfied with passing tests. The only modified file is `src/vulnerability/cwe_scanner.rs`, consistent with the plan's single-file constraint. All three implementation commits (5bfc964, ae9fc82, 6169846) exist in git history. The full `cargo test --features internal` suite is green.

---

_Verified: 2026-05-10_
_Verifier: Claude (gsd-verifier)_
