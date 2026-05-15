---
phase: 20-argument-value-ast-migration
reviewed: 2026-05-12T09:14:44Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - src/vulnerability/ast_scanner.rs
  - src/vulnerability/cwe_scanner.rs
  - tests/vulnerability_tests/ast_scanner_tests.rs
findings:
  critical: 1
  warning: 2
  info: 1
  total: 4
status: issues_found
---

# Phase 20: Code Review Report

**Reviewed:** 2026-05-12T09:14:44Z
**Depth:** standard
**Files Reviewed:** 3
**Status:** issues_found

## Summary

This phase migrates three CWE rules (CWE-295, CWE-319, CWE-732) from the lexical scanner's `arg_value_contains` (scan-all-args) approach to the AST scanner's new `ArgAtIndex` (positional-arg) strategy, and removes the `ContainsTokens` variant from the AST scanner's `ArgCheck` enum. The core mechanics are well-implemented: the umask exact-literal guard, the `collect_subtree_text` recursive collector, and the `SetSecurityDescriptorDacl` NULL check all look correct.

One **critical** behavioral regression was introduced in the CWE-319 rule for `CURLOPT_USE_SSL`: the migration dropped the second-token guard (`CURLUSESSL_NONE`) that the previous `ContainsTokens` required, resulting in false positives on any call that passes `CURLOPT_USE_SSL` with a safe value. Two warnings cover a hidden coupling in the `ArgAtIndex` handler and a missing false-positive test that would have caught the regression.

---

## Critical Issues

### CR-01: CWE-319 `CURLOPT_USE_SSL` rule dropped the value guard — false positives on safe calls

**File:** `src/vulnerability/ast_scanner.rs:72`

**Issue:** The old `ContainsTokens(&["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` required *both* the option name and the insecure value `CURLUSESSL_NONE` to be present in any argument before firing. The replacement rule `ArgAtIndex(1, &["CURLOPT_USE_SSL"])` fires on *any* `curl_easy_setopt` call where argument 1 is `CURLOPT_USE_SSL`, regardless of the third argument. This means:

```c
curl_easy_setopt(h, CURLOPT_USE_SSL, CURLUSESSL_ALL);   // safe — full TLS required
curl_easy_setopt(h, CURLOPT_USE_SSL, CURLUSESSL_TRY);   // conditionally safe
```

Both of these calls will now generate a CWE-319 finding in production scans, which is incorrect. The module comment at lines 68–71 acknowledges that "option-name alone is the signal" but does not address the false positive implication for `CURLOPT_USE_SSL`: unlike `CURLOPT_SSL_VERIFYPEER` and `CURLOPT_SSL_VERIFYHOST` (where any use of these options is suspicious because disabling them is the only reason to set them), `CURLOPT_USE_SSL` is a legitimate option that must be set to *enable* TLS — the dangerous value is only `CURLUSESSL_NONE`.

The missing third-argument check cannot be expressed with a single `ArgAtIndex` on index 1 alone. Options:

**Option A — Two-token multi-index check (requires new `ArgAtTwoIndices` variant or inline logic):**
```rust
// Fire only when arg 1 == CURLOPT_USE_SSL AND arg 2 == CURLUSESSL_NONE
AstCweRule {
    cwe_id: 319,
    functions: &["curl_easy_setopt"],
    arg_check: ArgCheck::ArgAtIndex(1, &["CURLOPT_USE_SSL"]),
    // ... plus a second check on arg 2 for CURLUSESSL_NONE
}
```

**Option B — Restore the `ContainsTokens` approach for this rule only, or move it back to the lexical scanner which had correct AND-all semantics.** Given that the original lexical rule `arg_value_contains: Some(&["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` was correct, this rule should not have been migrated until the AST scanner supports multi-index AND matching.

**Fix (minimum viable):** Either revert this specific rule to the AND-all check pattern, or add a second-argument check for `CURLUSESSL_NONE` in the `ArgAtIndex` evaluation path. At minimum, remove `CURLOPT_USE_SSL` from the AST rule table and re-add it to the lexical scanner with the original two-token constraint until proper multi-index support is built.

---

## Warnings

### WR-01: `ArgAtIndex` handler hardcodes `tokens[0] == "0"` as the trigger for number_literal kind-check — fragile coupling

**File:** `src/vulnerability/ast_scanner.rs:237`

**Issue:** The `ArgAtIndex` match arm contains a special-case branch:
```rust
} else if tokens.len() == 1 && tokens[0] == "0" {
    // D-10: umask-style exact-literal guard
    if args[idx].kind() != "number_literal" {
        false
    } else {
        let arg_text = args[idx].utf8_text(src).unwrap_or("");
        arg_text == "0"
    }
}
```

The kind-check (`number_literal`) is implicitly tied to the `umask` rule by checking for the literal token `"0"` rather than by a flag in `ArgCheck::ArgAtIndex`. This means:

1. Any future rule that uses `ArgAtIndex(n, &["0"])` with different semantics (e.g., checking for a zero-value integer in a different context) will unexpectedly receive the number_literal kind-check — it cannot opt out.
2. The condition `tokens.len() == 1 && tokens[0] == "0"` is a magic-value guard scattered inside the evaluation logic, coupling rule data to evaluation behavior.

The cleaner fix is to add a separate `ArgAtIndex` variant or an explicit flag field:
```rust
/// Positional arg at index must be exactly a `number_literal` node with this text.
ArgAtIndexExactLiteral(u8, &'static str),
```

This separates the "exact numeric literal" semantics from the "subtree text contains tokens" semantics, making both paths explicit and independently extensible.

### WR-02: No false-positive test for `CURLOPT_USE_SSL` with a safe value

**File:** `tests/vulnerability_tests/ast_scanner_tests.rs:163`

**Issue:** `test_argval_cwe319_ast_use_ssl_none` only verifies the true-positive case (`CURLUSESSL_NONE`). There is no corresponding `_no_fp` test for a safe `CURLOPT_USE_SSL` call such as `curl_easy_setopt(curl, CURLOPT_USE_SSL, CURLUSESSL_ALL)`. The `CWE-295` rule has an equivalent negative test (`test_argval_cwe295_ast_ssl_verify_peer_no_fp`), and `CWE-732` has two negative tests (`test_argval_cwe732_ast_umask_octal_no_fp`, `test_argval_cwe732_ast_umask_nested_call_no_fp`). The absence of this test is what allowed CR-01 to ship undetected.

**Fix:** Add:
```rust
#[test]
fn test_argval_cwe319_ast_use_ssl_all_no_fp() {
    let src = b"void f(void *curl) { curl_easy_setopt(curl, CURLOPT_USE_SSL, CURLUSESSL_ALL); }\n";
    let (_t, dirs) = setup_one_file("a.c", src);
    let findings = run_ast_scanner(&dirs);
    assert!(
        !findings.iter().any(|f| f.cwe_id == 319),
        "Expected NO CWE-319 finding for CURLOPT_USE_SSL+CURLUSESSL_ALL; got {:?}", findings
    );
}
```

---

## Info

### IN-01: Test comment says "12 AST-tractable CWEs" but the assertion array has 13 elements

**File:** `tests/vulnerability_tests/ast_scanner_tests.rs:83`

**Issue:** The comment on line 83 reads "12 AST-tractable CWEs (CWE-367, CWE-369 deferred to lexical-fallback only)" but the array on line 84 contains 13 CWE IDs: `[78, 119, 120, 122, 125, 134, 190, 242, 295, 319, 327, 377, 732]`. The correct count is 13 (CWE-367 and CWE-369 are the two deferred, leaving 13 from the original 15 in scope). The assertion itself is correct; only the comment count is wrong.

**Fix:** Update the comment to say "13 AST-tractable CWEs":
```rust
// Assert each of the 13 AST-tractable CWEs (CWE-367, CWE-369 deferred to lexical-fallback only)
```

---

_Reviewed: 2026-05-12T09:14:44Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
