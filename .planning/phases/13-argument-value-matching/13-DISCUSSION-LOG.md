# Phase 13: argument-value-matching - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 13-argument-value-matching
**Areas discussed:** Arg-matching mechanism, CWE-369 division detection, Rule table structure, Test approach, Token extraction helper, False positive handling, Dedup across new rules, CWE-295 wolfSSL inclusion, arg_value_contains AND vs OR semantics, Fallback mode interaction, CURLOPT_SSL_VERIFYHOST value 1, arg_contains helper signature, Report output for new CWEs

---

## Arg-matching mechanism

### Positional vs token-anywhere

| Option | Description | Selected |
|--------|-------------|----------|
| Token-anywhere-in-parens | Scan full paren-bound arg list for token. Simpler — no arg index needed. | ✓ |
| Positional (arg index + token) | Like format_arg_is_literal: skip to arg N, then check. More precise but requires knowing arg position per function. | |
| You decide | Claude picks. | |

**User's choice:** Token-anywhere-in-parens

### curl_easy_setopt variants (one rule vs separate)

| Option | Description | Selected |
|--------|-------------|----------|
| One rule, multiple tokens OR-matched | Single curl_easy_setopt entry with all option tokens. | ✓ |
| Separate rule per curl option | Three entries for curl_easy_setopt. | |
| You decide | Claude picks. | |

**User's choice:** One rule, multiple tokens OR-matched
**Notes:** Later refined to AND-all semantics (separate entries per option+value pair) to avoid FP on CURLOPT_SSL_VERIFYPEER with value 1.

### SetSecurityDescriptorDacl inclusion

| Option | Description | Selected |
|--------|-------------|----------|
| umask only | Skip SetSecurityDescriptorDacl — Windows only, complex arg matching. | |
| Both — SetSecurityDescriptorDacl via NULL token | Detect NULL in paren list. | |
| You decide | Claude decides based on ARGVAL-03 and target platform. | ✓ |

**User's choice:** You decide (deferred to Claude)
**Notes:** Claude chose to include both — satisfies ARGVAL-03 requirement; zero cost on Linux targets.

---

## CWE-369 division detection

### Detection path placement

| Option | Description | Selected |
|--------|-------------|----------|
| Separate code path in scan_file | Independent check per line, not in CweRule loop. | ✓ |
| New rule type in CWE_RULES table | Add division_pattern flag to CweRule. | |
| You decide | Claude picks cleaner extension point. | |

**User's choice:** Separate code path in scan_file

### Pattern strictness

| Option | Description | Selected |
|--------|-------------|----------|
| Include whitespace variants (/ 0 and % 0) | Pattern: [/%]\s*0[^0-9.]. Fewer false negatives. | ✓ |
| Strict /0 and %0 only | Simpler pattern, may miss common formatting. | |
| You decide | Claude picks. | |

**User's choice:** Include whitespace variants

---

## Rule table structure

### Struct extension approach

| Option | Description | Selected |
|--------|-------------|----------|
| Add arg_value_contains to CweRule | Extend existing struct with Option<&'static [&'static str]>. | ✓ |
| Separate ArgValRule struct | Keep CweRule untouched, new struct for arg-value rules. | |

**User's choice:** Add arg_value_contains to CweRule
**Notes:** ARGVAL-05 specifically says "CweRule struct supports optional field" — the separate struct option would conflict with the requirement.

### CWE count test update

| Option | Description | Selected |
|--------|-------------|----------|
| Update test to 18 distinct CWEs | Add 295, 319, 732, 369 to the count. | ✓ |
| Leave test unchanged at 14 | Would leave the test comment wrong. | |

**User's choice:** Update test to 18 distinct CWEs

---

## Test approach

### Test pattern for new ARGVAL rules

| Option | Description | Selected |
|--------|-------------|----------|
| Inline tempfile (same as existing) | tempfile::tempdir() + inline C string. Consistent with Phase 11. | ✓ |
| Committed fixture files | More realistic but adds infrastructure; no tests/fixtures/ exists. | |

**User's choice:** Inline tempfile

### ARGVAL-05 backward compat test

| Option | Description | Selected |
|--------|-------------|----------|
| Implicit — existing tests cover it | All current tests must still pass after struct change. That IS the proof. | ✓ |
| Explicit test for None path | Redundant with existing coverage. | |

**User's choice:** Implicit coverage

---

## Token extraction helper

| Option | Description | Selected |
|--------|-------------|----------|
| str::contains with word-boundary check | Reuse find_function_call's word-boundary logic. fn paren_args_contain_all. | ✓ |
| New extract_paren_args helper | Parse args into Vec<&str>. Adds ~30 lines. | |
| Plain str::contains (no boundary) | High FP risk for token prefixes. | |

**User's choice:** str::contains with word-boundary check

---

## False positive handling (numeric tokens)

| Option | Description | Selected |
|--------|-------------|----------|
| Require 0 is standalone token | Word-boundary: 0 must be preceded/followed by non-digit chars. umask(0077) won't fire. | ✓ |
| Require exact string "0" via str::contains | Would match "0077" — insufficient. | |

**User's choice:** Require 0 is standalone token

---

## Dedup across new rules

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — dedup by (file, line, cwe) in run_lexical_scanner | HashSet removes duplicates before return. Consistent with Phase 14 CPPCHECK-05. | ✓ |
| No dedup in Phase 13 | Defer to Phase 14. Risk of duplicate findings if name-only and arg-value rules both match. | |

**User's choice:** Dedup in Phase 13

---

## CWE-295 wolfSSL inclusion

| Option | Description | Selected |
|--------|-------------|----------|
| One rule entry, all three in functions array | SSL_CTX_set_verify, SSL_set_verify, wolfSSL_CTX_set_verify. Same token, same CWE. | ✓ |
| Separate entry per function | Redundant. | |

**User's choice:** One rule entry, all three in functions array

---

## arg_value_contains AND vs OR semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Separate CweRule entries per option (AND-all per entry) | Each entry has both option name and insecure value. Satisfies ARGVAL-02's "value 0" requirement. | ✓ |
| OR-match with documented FP | Fire whenever option name appears, regardless of value. Violates ARGVAL-02. | |

**User's choice:** Separate entries, AND-all semantics

---

## Fallback mode interaction

| Option | Description | Selected |
|--------|-------------|----------|
| No special handling needed | New rules in same CWE_RULES table, same scan_file. Fallback inherits automatically. | ✓ |
| Guard new rules behind a flag | No reason to do this. | |

**User's choice:** No special handling needed

---

## CURLOPT_SSL_VERIFYHOST value 1

| Option | Description | Selected |
|--------|-------------|----------|
| Two entries, one per insecure value | &["CURLOPT_SSL_VERIFYHOST", "0"] and &["CURLOPT_SSL_VERIFYHOST", "1"]. Matches ARGVAL-02 exactly. | ✓ |
| One entry with OR on values | Inconsistent with AND-all semantics. | |

**User's choice:** Two entries confirmed

---

## arg_contains helper function signature

| Option | Description | Selected |
|--------|-------------|----------|
| Private fn in cwe_scanner.rs | fn paren_args_contain_all — private to module. | ✓ |
| Public fn exported | No other module needs it. | |

**User's choice:** Private function

---

## Report output for new CWEs

| Option | Description | Selected |
|--------|-------------|----------|
| No changes needed — generic pipeline handles it | Report and CDX consume SastFinding.cwe_id generically. | ✓ |
| Verify by checking report/CDX code | Quick sanity check. | |

**User's choice:** No changes needed

---

## Claude's Discretion

- **SetSecurityDescriptorDacl inclusion**: User chose "you decide". Claude included it with NULL token match to satisfy ARGVAL-03 wording. Zero cost on Linux/xcar-linux targets.

## Deferred Ideas

None — discussion stayed within phase scope.
