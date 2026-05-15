---
phase: 13-argument-value-matching
reviewed: 2026-05-10T00:00:00Z
depth: standard
files_reviewed: 1
files_reviewed_list:
  - src/vulnerability/cwe_scanner.rs
findings:
  critical: 0
  warning: 4
  info: 2
  total: 6
status: issues_found
---

# Phase 13: Code Review Report

**Reviewed:** 2026-05-10
**Depth:** standard
**Files Reviewed:** 1
**Status:** issues_found

## Summary

`cwe_scanner.rs` adds argument-value matching (CWE-295, CWE-319, CWE-732, CWE-369) on top of the existing name-only lexical scanner. The core logic — `paren_args_contain_all`, `token_present_with_boundary`, `contains_div_by_zero` — is generally sound, but there are two correctness defects in `token_present_with_boundary` that produce false positives: the left-boundary check does not exclude `.` for digit tokens, and the right-boundary check does not exclude hex/suffix letters (`x`, `L`, `u`). Both defects affect real rules in the CWE-319 and CWE-732 rule tables.

---

## Warnings

### WR-01: Digit token left-boundary does not exclude `.` — false positive on decimal trailing zeros

**File:** `src/vulnerability/cwe_scanner.rs:189-193`

**Issue:** `token_present_with_boundary` classifies a token as `is_digit_token` when all its bytes are ASCII digits, then applies a stricter right-boundary check. However, the left-boundary check is identical for both digit and non-digit tokens: it only rejects a preceding `is_ascii_alphanumeric || == b'_'`. The character `.` passes this check. As a result, the second `0` in `0.0` will match the token `"0"` when the left character is `.` and the right end is at end-of-slice. Concrete trigger: `curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0.0)` fires CWE-319 because `0.0` ends at the closing `)` (not in the slice returned by `paren_args_contain_all`), so the trailing `0` matches. The existing test suite does not cover this case.

**Fix:**
```rust
let left_ok = if pos == 0 {
    true
} else {
    let prev = bytes[pos - 1];
    if is_digit_token {
        // For digit tokens: also reject preceding '.' to avoid matching
        // trailing zeros in decimals like 0.0
        !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'.')
    } else {
        !(prev.is_ascii_alphanumeric() || prev == b'_')
    }
};
```

---

### WR-02: Digit token right-boundary does not exclude hex/suffix letters — false positive on `0x…`, `0L`, `0u`

**File:** `src/vulnerability/cwe_scanner.rs:199-201`

**Issue:** For `is_digit_token`, the right-boundary check only excludes the next character being a digit or `.`. It does not exclude the alphabetic hex/suffix characters (`x`, `X`, `b`, `B`, `o`, `O`, `l`, `L`, `u`, `U`). Consequently `0x1` has `0` at position 0, right character `x`, which is not a digit or `.`, so `right_ok = true` — the token `"0"` matches. This fires CWE-319 for `curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0x1)` (non-zero, secure), and CWE-732 for `umask(0L)` (which may or may not be zero, but `0L` is indistinguishable without knowing the value — typically a false positive for `0x10L`). Similarly `0u`, `0UL` etc.

**Fix:**
```rust
if is_digit_token {
    !(next.is_ascii_digit()
        || next == b'.'
        || next == b'x' || next == b'X'   // hex prefix
        || next == b'b' || next == b'B'   // binary prefix (GCC extension)
        || next == b'o' || next == b'O'   // octal prefix (C23)
        || next == b'l' || next == b'L'   // long suffix
        || next == b'u' || next == b'U')  // unsigned suffix
} else {
    !(next.is_ascii_alphanumeric() || next == b'_')
}
```

Note: The existing non-digit branch already uses `is_ascii_alphanumeric` which covers all letters, so only the digit branch is defective.

---

### WR-03: `contains_div_by_zero` fires inside C/C++ block comments and string literals

**File:** `src/vulnerability/cwe_scanner.rs:222-244`

**Issue:** `contains_div_by_zero` is a byte-level scan with no awareness of comment or string boundaries. A line like `printf("ratio is x / 0\n");` or `/* allowed: x / 0 per spec */` will produce a CWE-369 finding. The existing scanner intentionally accepts false positives for comments (see design note at line 219-221), but the case of division-by-zero inside a string literal is harder to dismiss as acceptable: it means any log message mentioning "divided by 0" triggers a security finding that appears in the SBOM output.

The impact is that CWE-369 findings cannot be trusted if the scanned codebase contains documentation strings or error-message strings containing `/` followed by `0`.

**Fix:** At minimum, skip the check when the `0` occurs inside a string literal context. A minimal guard: after finding a potential `/`-then-`0` match, scan backwards on the line for an unmatched `"` — if found, skip. This is imperfect but consistent with the existing scanner's heuristic philosophy and would eliminate the most common false positive category.

---

### WR-04: `run_lexical_scanner` dedup result is non-deterministic when a source file is reachable from two component directories

**File:** `src/vulnerability/cwe_scanner.rs:361-381`

**Issue:** `component_dirs` is a `HashMap<(String, String), PathBuf>`. HashMap iteration order is not guaranteed. If two different `(name, ecosystem)` keys resolve to overlapping directories (or if symlinks cause the same physical file to be visited via two different `dir` entries), the order in which their findings are appended to `all_findings` is non-deterministic. The dedup at line 380 keeps the **first** occurrence by `(file_path, line, cwe_id)`. When component names differ, the retained finding's `component_name` is non-deterministic across runs. This produces non-reproducible SBOM output for the same source tree.

**Fix:** Before dedup, sort `all_findings` by `(file_path, line, cwe_id, component_name)` to make the first-occurrence selection deterministic:
```rust
all_findings.sort_by(|a, b| {
    (&a.file_path, a.line, a.cwe_id, &a.component_name)
        .cmp(&(&b.file_path, b.line, b.cwe_id, &b.component_name))
});
```

---

## Info

### IN-01: `format_arg_is_literal` does not handle parenthesized string literals

**File:** `src/vulnerability/cwe_scanner.rs:100-102`

**Issue:** The check for a literal format argument is `trimmed.starts_with('"')`. A parenthesized literal `("format %s")` or a macro that expands to a string `(L"wide")` or a concat `("a" "b")` would match, but `((literal))` (double-paren cast pattern common in some codebases) would not. This produces a false negative: a function call with `((literal_format))` is flagged as unsafe. The impact is low (extra finding, not a missed vulnerability), and this is a known limitation of lexical analysis.

---

### IN-02: `find_function_call` does not check right character boundary after function name

**File:** `src/vulnerability/cwe_scanner.rs:263-270`

**Issue:** After matching the function name substring, `find_function_call` verifies that the next non-whitespace character is `(`. This correctly rejects `strcpy_s(` because `trim_start()` of `"_s(...)"` does not start with `(`. However, if a function name is a prefix of another and whitespace separates them from `(`, a pathological case like a macro `#define MYFUNC(x) strcpy (x,x)` on one line — searching for `strcpy` — would find `strcpy ` and correctly match because trimmed starts with `(`. This is the correct behavior. No actual bug, just noting the logic path is narrowly correct only because `trim_start` + `starts_with('(')` together enforce both "no suffix chars" and "next meaningful char is paren." No action needed.

---

_Reviewed: 2026-05-10_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
