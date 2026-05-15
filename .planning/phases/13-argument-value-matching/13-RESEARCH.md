# Phase 13: argument-value-matching - Research

**Researched:** 2026-05-10
**Domain:** Rust lexical scanner extension — paren-bound argument inspection, CWE rule table
**Confidence:** HIGH

## Summary

Phase 13 extends `src/vulnerability/cwe_scanner.rs` in-place. All implementation details are fully
specified in CONTEXT.md through a prior discussion session. There are no library choices to make, no
new dependencies, and no output format changes. The only codebase change is to one Rust source file.

The work is self-contained: add an `arg_value_contains` field to `CweRule`, implement a
`paren_args_contain_all` helper using the existing word-boundary convention, add four new CWE rule
groups, add a separate CWE-369 scan path, deduplicate findings in `run_lexical_scanner`, and update
the rule-table count assertion.

**Primary recommendation:** Extend `cwe_scanner.rs` exactly as specified in CONTEXT.md decisions
D-01 through D-15. No alternatives or deviations are warranted.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- D-01: Add `arg_value_contains: Option<&'static [&'static str]>` to `CweRule`; all existing rules get `None`.
- D-02: AND-all semantics — every token in the slice must appear in the paren argument list; fire only when all match.
- D-03: Dedup `SastFinding`s by `(file, line, cwe)` in `run_lexical_scanner` using `HashSet<(String, u32, u32)>`.
- D-04: `fn paren_args_contain_all(after_func: &str, tokens: &[&str]) -> bool` — word-boundary on both sides of each token.
- D-05: Numeric token boundary — digit token `"0"` requires non-digit on both sides; prevents `0077` from matching.
- D-06: CWE-295 rule: `functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"]`, `arg_value_contains: Some(&["SSL_VERIFY_NONE"])`.
- D-07: CWE-319 — four separate `CweRule` entries for `curl_easy_setopt` (CURLOPT_USE_SSL+CURLUSESSL_NONE; CURLOPT_SSL_VERIFYPEER+0; CURLOPT_SSL_VERIFYHOST+0; CURLOPT_SSL_VERIFYHOST+1).
- D-08: CWE-732 — two rules: `umask` + `"0"`, `SetSecurityDescriptorDacl` + `"NULL"`.
- D-09: CWE-369 — separate code path in `scan_file`, NOT a `CweRule`. Pattern `[/%]\s*0[^0-9.]` checked after main loop per line.
- D-10: CWE-369 emits `SastFinding { cwe_id: 369, ... }` directly.
- D-11: Rename `test_rule_table_has_fourteen_cwes` to assert 18 distinct CWE IDs (adds 295, 319, 732, 369).
- D-12: New tests use `tempfile::tempdir()` + inline C string + `scan_file`; one test per ARGVAL requirement.
- D-13: ARGVAL-05 backward compat implicitly covered by existing tests still passing.
- D-14: No changes to fallback mode or output pipeline.
- D-15: No changes to `_static_analysis.md` or CycloneDX output formatting.

### Claude's Discretion

- `SetSecurityDescriptorDacl` with NULL token: included per ARGVAL-03 requirement wording. Zero-cost rule; satisfies the requirement even though xcar-linux is Linux-only.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARGVAL-01 | Detect CWE-295 when SSL_CTX_set_verify/SSL_set_verify/wolfSSL_CTX_set_verify called with SSL_VERIFY_NONE | D-06: one CweRule with three functions and arg_value_contains |
| ARGVAL-02 | Detect CWE-319 for curl_easy_setopt with insecure CURLOPT_* options | D-07: four separate CweRule entries with AND-all token matching |
| ARGVAL-03 | Detect CWE-732 via umask(0) and SetSecurityDescriptorDacl with NULL | D-08: two CweRule entries |
| ARGVAL-04 | Detect CWE-369 on literal /0 or %0 in division expression | D-09/D-10: separate scan path, manual byte scan |
| ARGVAL-05 | CweRule supports optional arg_value_contains; match fires only on paren-bound token match | D-01 through D-05: struct extension + helper fn |

</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Arg-value token matching | `cwe_scanner.rs` (lexical scanner) | — | Pure in-file text matching, no pipeline changes |
| CweRule struct extension | `cwe_scanner.rs` (data model) | — | Struct lives in same file as rule table |
| Finding deduplication | `run_lexical_scanner` (public entry point) | — | D-03 places HashSet here, after all findings collected |
| CWE-369 detection | `scan_file` (per-file scan loop) | — | Same loop as rule-table path; separate code path after main loop |
| CycloneDX/report output | Unchanged (SastFinding.cwe_id consumed generically) | — | No tier changes needed |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std | (stable) | Byte scanning, HashSet, BufRead | No external dep needed for manual scan |
| tempfile | 3.15 (in Cargo.toml) | Test temp dirs | Already in dev-dependencies |

[VERIFIED: Cargo.toml in project root] — `tempfile = "3.15"` in both `[dependencies]` and `[dev-dependencies]`.

No new dependencies are required for Phase 13. The entire implementation uses Rust's standard library
and the already-imported types (`HashSet` from `std::collections`, `BufRead` from `std::io`).

**Installation:** No new `cargo add` commands needed.

## Architecture Patterns

### System Architecture Diagram

```
C/C++ source file
      |
      v
scan_file(path, component_name, ecosystem)
      |
      +--[per line]--+
      |               |
      |    [CWE_RULES loop] ─── find_function_call ──► word-boundary match
      |               |                                      |
      |               |              arg_value_contains? ────+
      |               |              Some(tokens) → paren_args_contain_all
      |               |              None         → fire immediately
      |               |
      |    [CWE-369 path] ─── manual byte scan for [/%]\s*0[^0-9.]
      |               |
      +---------------+
      |
      Vec<SastFinding> (may have duplicates from overlapping rules)
      |
      v
run_lexical_scanner
      |
      HashSet<(file, line, cwe)> ─── dedup
      |
      Vec<SastFinding> (deduplicated)
      |
      v
Existing CycloneDX + _static_analysis.md pipeline (UNCHANGED)
```

### Recommended Project Structure

No new files. All changes are in:
```
src/vulnerability/
└── cwe_scanner.rs    # all Phase 13 changes
```

### Pattern 1: CweRule struct extension

**What:** Add `arg_value_contains: Option<&'static [&'static str]>` as a new field. Struct literal
syntax in Rust requires all fields — every existing CweRule entry gets `, arg_value_contains: None`.

**When to use:** Anytime a rule should only fire when specific tokens appear in the argument list.

```rust
// Source: cwe_scanner.rs (existing struct, extended)
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
    arg_value_contains: Option<&'static [&'static str]>,
}
```

Existing entries all gain `, arg_value_contains: None` at the end of each struct literal.

### Pattern 2: paren_args_contain_all helper

**What:** Extracts the content between `(` and the matching `)` from `after_func`, then checks each
token with word-boundary logic. AND-all: returns false on first non-matching token.

**When to use:** Called from the `scan_file` rule loop when `rule.arg_value_contains` is `Some(tokens)`.

```rust
// Source: CONTEXT.md D-04, D-05
fn paren_args_contain_all(after_func: &str, tokens: &[&str]) -> bool {
    // 1. Strip leading whitespace, then expect '('
    let rest = after_func.trim_start();
    let inner = match rest.strip_prefix('(') {
        Some(r) => r,
        None => return false,
    };
    // 2. Extract up to matching ')' tracking paren depth
    let mut depth: i32 = 0;
    let mut end = inner.len();
    for (i, ch) in inner.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' if depth == 0 => { end = i; break; }
            ')' => depth -= 1,
            _ => {}
        }
    }
    let arg_slice = &inner[..end];
    // 3. Check each token with word-boundary rules
    for &token in tokens {
        if !token_present_with_boundary(arg_slice, token) {
            return false;
        }
    }
    true
}

fn token_present_with_boundary(haystack: &str, token: &str) -> bool {
    let bytes = haystack.as_bytes();
    let tok = token.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = haystack[i..].find(token) {
        let pos = i + rel;
        let left_ok = pos == 0 || {
            let prev = bytes[pos - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_')
        };
        let after_pos = pos + tok.len();
        let right_ok = after_pos >= bytes.len() || {
            let next = bytes[after_pos];
            let is_digit_token = tok.iter().all(|b| b.is_ascii_digit());
            if is_digit_token {
                // D-05: numeric boundary — next char must not be digit or '.'
                !(next.is_ascii_digit() || next == b'.')
            } else {
                !(next.is_ascii_alphanumeric() || next == b'_')
            }
        };
        if left_ok && right_ok {
            return true;
        }
        i = pos + 1;
        if i >= haystack.len() { break; }
    }
    false
}
```

### Pattern 3: CWE-369 separate scan path

**What:** After the main `CWE_RULES` loop for a line, check for division/modulo by literal zero.
No regex crate — manual byte scan.

**When to use:** Once per line, in `scan_file`, after the rule loop.

```rust
// Source: CONTEXT.md D-09
// After the CWE_RULES loop for the current line:
if contains_div_by_zero(&line) {
    findings.push(SastFinding {
        cwe_id: 369,
        component_name: component_name.to_string(),
        component_ecosystem: component_ecosystem.to_string(),
        file_path: path.to_string_lossy().into_owned(),
        line: line_num,
    });
}

fn contains_div_by_zero(line: &str) -> bool {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    while i < len {
        if bytes[i] == b'/' || bytes[i] == b'%' {
            // Skip whitespace after operator
            let mut j = i + 1;
            while j < len && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            if j < len && bytes[j] == b'0' {
                // Check right boundary: next must not be digit or '.'
                let after = j + 1;
                let right_ok = after >= len
                    || (!bytes[after].is_ascii_digit() && bytes[after] != b'.');
                if right_ok {
                    return true;
                }
            }
        }
        i += 1;
    }
    false
}
```

### Pattern 4: Deduplication in run_lexical_scanner

**What:** Collect all findings across all files into `all_findings`, then filter via a `HashSet`
keyed on `(file, line, cwe_id)` before returning.

```rust
// Source: CONTEXT.md D-03
// At end of run_lexical_scanner, before returning:
use std::collections::HashSet;
let mut seen: HashSet<(String, u32, u32)> = HashSet::new();
all_findings.retain(|f| {
    seen.insert((f.file_path.clone(), f.line, f.cwe_id))
});
all_findings
```

`HashSet::insert` returns `true` when the element was newly inserted, so `retain` keeps only the
first occurrence of each `(file, line, cwe)` triple.

### Anti-Patterns to Avoid

- **Modifying `scan_file` signature:** The function signature `fn scan_file(path, component_name, component_ecosystem)` is unchanged. CWE-369 is a new code path inside the function body, not a parameter.
- **Adding arg_value_contains to CWE-134 rules:** CWE-134 already has its own `requires_format_heuristic` path. The two mechanisms are independent; do not mix them.
- **Placing dedup inside scan_file:** D-03 explicitly places the HashSet in `run_lexical_scanner`, not per-file. Per-file dedup would miss cross-file rule interactions and change semantics.
- **Using the `regex` crate for CWE-369:** The project already includes `regex = "1.10"` in dependencies, but the convention established in CONTEXT.md is a manual byte scan (no external crate for this path). Keep consistent with `format_arg_is_literal` and `find_function_call` style.
- **Right-boundary omission for alphanumeric tokens:** `SSL_VERIFY_NONE` ends in `E` — must verify the char after position `E` is non-alnum/non-underscore to avoid matching `SSL_VERIFY_NONE_EXTRA`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Temp directories in tests | Custom temp path logic | `tempfile::tempdir()` | Already in Cargo.toml; auto-cleaned on drop |
| Finding deduplication data structure | Custom Vec scan | `HashSet<(String, u32, u32)>` | O(1) insert/lookup; idiomatic Rust |

**Key insight:** The scanner is intentionally zero-dependency for the scan logic itself. All matching
uses manual byte scanning consistent with the existing `find_function_call` and `format_arg_is_literal`
patterns.

## Common Pitfalls

### Pitfall 1: Forgetting arg_value_contains: None on all existing rules

**What goes wrong:** Rust struct literal syntax requires all fields to be specified (no default for
non-Default structs). Every one of the 15 existing `CweRule` entries in `CWE_RULES` must get
`, arg_value_contains: None` added.

**Why it happens:** The field addition is mechanical — easy to add to new rules but miss one of the
existing 15.

**How to avoid:** Add `arg_value_contains: None` to every existing entry before writing new ones.
Compiler will catch any missed entry at compile time (missing field error).

**Warning signs:** `error[E0063]: missing field 'arg_value_contains' in initializer of 'CweRule'`.

### Pitfall 2: Digit token boundary for "0" — left side only

**What goes wrong:** Only checking the right boundary for digit tokens. `umask(10)` would fire if
only the right side is checked (the `0` at position 1 has `)` on the right).

**Why it happens:** D-05 specifies non-digit on BOTH sides for digit tokens. The left boundary is
also non-alnum/non-underscore per the standard word-boundary rule, but the digit case is symmetric:
`10` has `1` on the left of `0`, which is a digit — must also reject.

**How to avoid:** The `token_present_with_boundary` function must apply: left side = not
alnum/underscore AND right side = not digit/dot (for numeric tokens). Test case: `umask(0077)`
must NOT fire; `umask(0)` must fire.

**Warning signs:** `test_argval_cwe732_umask` passing when it should reject `umask(0077)`.

### Pitfall 3: CWE-369 path matching C++ comment `//`

**What goes wrong:** The pattern `[/%]\s*0` could match on a line like `// offset = 0` because
the first `/` of the comment is found, then whitespace is skipped (there is none), then the second
`/` does not equal `0`. Actually no — but a pattern like `x / 0 // comment` is fine. The real
pitfall is a line like `x = result; // divide by 0` where `/ 0` appears inside a comment.

**Why it happens:** The scanner operates line-by-line with no comment stripping. `// divide by 0`
on a comment line would produce a false positive.

**How to avoid:** This is a known limitation of lexical (non-AST) scanning, documented in the
project's design. The context explicitly accepts this tradeoff. No comment stripping is needed —
accept the FP rate as consistent with the existing scanner's design philosophy. Document in test
that the detection is line-level.

**Warning signs:** A test that writes `// x / 0 is bad` expecting no finding but gets one.

### Pitfall 4: paren depth tracking in paren_args_contain_all

**What goes wrong:** Not tracking nested parentheses depth causes the arg-slice to be truncated
at a nested `)`.

**Why it happens:** C calls like `SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, callback(x))` have
a nested paren in the third arg. Stopping at the first `)` would truncate after `callback(x`.

**How to avoid:** Track depth starting at 0; only stop at `)` when depth is 0.

**Warning signs:** Tests with multi-arg calls that contain nested parens fail to detect tokens.

### Pitfall 5: AND-all with multiple curl rules on a single call site

**What goes wrong:** A single `curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0)` line could match
multiple CWE-319 rules if a rule's tokens appear on the line.

**Why it happens:** Rule D-07 has four separate entries. `CURLOPT_SSL_VERIFYPEER` with `0` matches
only the second entry — the first entry requires `CURLOPT_USE_SSL` AND `CURLUSESSL_NONE`, which are
not present. AND-all semantics prevent spurious matches here. But D-03 dedup ensures no duplicate
CWE-319 findings even if somehow two rules match.

**How to avoid:** AND-all semantics are the protection. Confirm each test case only fires the
expected rule.

## Code Examples

### Verified patterns from existing codebase

#### Word-boundary function matching (existing, to replicate in helper)
```rust
// Source: src/vulnerability/cwe_scanner.rs find_function_call
fn find_function_call(line: &str, func: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let func_bytes = func.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = line[search_from..].find(func) {
        let pos = search_from + rel;
        let left_ok = if pos == 0 {
            true
        } else {
            let prev = bytes[pos - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_')
        };
        if left_ok {
            let after_idx = pos + func_bytes.len();
            let after = &line[after_idx..];
            if after.trim_start().starts_with('(') {
                return Some(pos);
            }
        }
        search_from = pos + 1;
    }
    None
}
```

#### Existing test pattern (inline C, tempfile, scan_file direct call)
```rust
// Source: tests/scanner_tests/ pattern (established in Phase 11)
#[test]
fn test_argval_cwe295_ssl_verify_none() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("test.c");
    std::fs::write(&src, r#"
        SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL);
    "#).unwrap();
    let findings = scan_file(&src, "mylib", "C/C++");
    assert!(findings.iter().any(|f| f.cwe_id == 295),
        "Expected CWE-295 finding");
}
```

#### Dedup using HashSet retain
```rust
// Source: CONTEXT.md D-03 — idiomatic pattern for retain-with-side-effect
use std::collections::HashSet;
let mut seen: HashSet<(String, u32, u32)> = HashSet::new();
all_findings.retain(|f| seen.insert((f.file_path.clone(), f.line, f.cwe_id)));
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Name-only CWE rule matching | Name + optional arg-value token matching (AND-all) | Phase 13 | Reduces FP for context-dependent APIs (SSL verify, umask) |
| No deduplication in run_lexical_scanner | HashSet dedup by (file, line, cwe) | Phase 13 | Prevents duplicate findings when name-only and arg-value rules overlap |

**Deprecated/outdated:**
- `test_rule_table_has_fourteen_cwes`: renamed and assertion updated to 18 distinct CWE IDs (295, 319, 732, 369 added).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `scan_file` is not `pub` — tests call it from within the same module's `#[cfg(test)]` block | Code Examples | If tests live in a separate file, scan_file must be made pub(crate) |

**Note on A1:** Reading `cwe_scanner.rs` confirms `scan_file` is defined as `fn scan_file` (private)
and tests are in `mod tests` within the same file using `use super::*`. The existing test pattern
(Phase 11 tests in the same file) confirms this. [VERIFIED: cwe_scanner.rs line 154]

The Assumptions Log is effectively empty — A1 is self-verified by the codebase read.

## Open Questions

1. **HashSet import in run_lexical_scanner**
   - What we know: `std::collections::HashMap` is already imported at the top of `cwe_scanner.rs`
     (line 16: `use std::collections::HashMap;`).
   - What's unclear: Whether to add `HashSet` to the same use statement or add a separate import.
   - Recommendation: Extend the existing import to `use std::collections::{HashMap, HashSet};`.
   [VERIFIED: cwe_scanner.rs line 16]

## Environment Availability

Step 2.6: SKIPPED — Phase 13 is purely in-codebase Rust changes. No external tools, services, or
CLIs are required beyond the existing Rust toolchain and Cargo.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test (`#[test]`) |
| Config file | none — Cargo.toml features gate |
| Quick run command | `cargo test --features internal -p radeis_sc2sbom cwe_scanner` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARGVAL-01 | SSL_CTX_set_verify with SSL_VERIFY_NONE fires CWE-295 | unit | `cargo test --features internal cwe_scanner::tests::test_argval_cwe295` | ❌ Wave 0 |
| ARGVAL-02 | curl_easy_setopt with insecure CURLOPT_* fires CWE-319 | unit | `cargo test --features internal cwe_scanner::tests::test_argval_cwe319` | ❌ Wave 0 |
| ARGVAL-03 | umask(0) and SetSecurityDescriptorDacl(NULL) fire CWE-732 | unit | `cargo test --features internal cwe_scanner::tests::test_argval_cwe732` | ❌ Wave 0 |
| ARGVAL-04 | literal /0 or %0 fires CWE-369 | unit | `cargo test --features internal cwe_scanner::tests::test_argval_cwe369` | ❌ Wave 0 |
| ARGVAL-05 | Existing rules (arg_value_contains: None) still fire on name match alone | unit (implicit) | `cargo test --features internal cwe_scanner::tests` | ❌ Wave 0 (all existing tests must still pass) |

### Sampling Rate

- **Per task commit:** `cargo test --features internal cwe_scanner`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src/vulnerability/cwe_scanner.rs` test functions for ARGVAL-01 through ARGVAL-04 (added inline in `mod tests`)
- [ ] Updated `test_rule_table_has_fourteen_cwes` → asserts 18 distinct CWE IDs

*(All tests are inline unit tests inside `cwe_scanner.rs`; no separate test file is needed per D-12)*

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Manual byte scanning with explicit boundary checks |
| V6 Cryptography | no | — |

**Note:** Phase 13 implements a detector for cryptographic misuse (CWE-295, CWE-319) in scanned
code, not in the scanner tool itself. The scanner's own security posture is unchanged: it reads
files read-only, returns structured data, and has no network I/O.

### Known Threat Patterns for Rust lexical scanner

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal in scanned files | Information Disclosure | WalkDir scoped to component_dirs (existing SCAN-05 control) |
| Regex injection via token matching | Tampering | No regex used; manual byte scan with literal string matching only |

## Sources

### Primary (HIGH confidence)

- `src/vulnerability/cwe_scanner.rs` (project codebase, read directly) — existing CweRule struct, CWE_RULES table, find_function_call, format_arg_is_literal, scan_file, run_lexical_scanner
- `.planning/phases/13-argument-value-matching/13-CONTEXT.md` (project planning artifact) — D-01 through D-15, all implementation decisions locked
- `Cargo.toml` (project codebase) — dependency versions, feature flags, tempfile availability

### Secondary (MEDIUM confidence)

- `.planning/REQUIREMENTS.md` — ARGVAL-01 through ARGVAL-05 requirement text
- `.planning/PROJECT.md` — Constraints: Rust-only, internal feature gate, musl target

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new libraries; existing Cargo.toml fully verified
- Architecture: HIGH — CONTEXT.md decisions are fully specified; codebase read confirms all integration points
- Pitfalls: HIGH — derived from code reading and Rust language semantics (struct literal completeness, boundary logic)

**Research date:** 2026-05-10
**Valid until:** 2026-06-10 (stable domain — pure Rust internal extension, no ecosystem churn)
