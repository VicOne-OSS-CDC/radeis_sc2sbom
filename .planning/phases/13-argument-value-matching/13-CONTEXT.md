# Phase 13: argument-value-matching - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Extend `src/vulnerability/cwe_scanner.rs` to detect insecure API usage via argument values. Add an optional `arg_value_contains` field to `CweRule`, implement a `paren_args_contain_all` helper for AND-all word-boundary token matching, add new rules for CWE-295 (TLS verify disabled), CWE-319 (curl insecure options), CWE-732 (permissive umask/DACL), and a separate CWE-369 code path (division-by-literal-zero). No changes to output formats or report generation.

</domain>

<decisions>
## Implementation Decisions

### CweRule struct extension (ARGVAL-05)
- **D-01:** Add `arg_value_contains: Option<&'static [&'static str]>` to the existing `CweRule` struct. All 14+ existing rules set this field to `None`. Rules with `None` continue to fire on function name match alone — backward compat is proven by existing tests still passing after the struct change.
- **D-02:** `arg_value_contains` semantics = **AND-all**: every token in the slice must appear in the paren-bound argument list. Fire only when ALL tokens match.
- **D-03:** Dedup `SastFinding`s by `(file, line, cwe)` in `run_lexical_scanner` using a `HashSet<(String, u32, u32)>` before returning. Prevents duplicate findings when name-only and arg-value rules would both match the same call site.

### Arg-value token matching helper
- **D-04:** Add private `fn paren_args_contain_all(paren_slice: &str, tokens: &[&str]) -> bool` in `cwe_scanner.rs` alongside `find_function_call`. Word-boundary check on both sides of each token (char before and after must be non-alnum, non-`_`). All tokens in the slice must match (AND-all). `paren_slice` is the raw `&str` starting from `(` up to end of line.
- **D-05:** Numeric token boundary: a digit token like `"0"` requires non-digit chars on both sides — so `umask(0077)` does NOT fire (0 followed by 7), but `umask(0)` fires (0 followed by `)`).

### CWE-295 rule (ARGVAL-01)
- **D-06:** One `CweRule` entry, `functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"]`, `arg_value_contains: Some(&["SSL_VERIFY_NONE"])`. All three functions share the same token and CWE.

### CWE-319 rules (ARGVAL-02)
- **D-07:** Separate `CweRule` entries per insecure curl option — all target `curl_easy_setopt`. AND-all semantics require both the option name and insecure value to appear:
  - Entry 1: `arg_value_contains: Some(&["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])`
  - Entry 2: `arg_value_contains: Some(&["CURLOPT_SSL_VERIFYPEER", "0"])`
  - Entry 3: `arg_value_contains: Some(&["CURLOPT_SSL_VERIFYHOST", "0"])`
  - Entry 4: `arg_value_contains: Some(&["CURLOPT_SSL_VERIFYHOST", "1"])` (value 1 = hostname existence check only, also insecure per ARGVAL-02)

### CWE-732 rules (ARGVAL-03)
- **D-08:** Two `CweRule` entries:
  - `umask` with `arg_value_contains: Some(&["0"])` — fires only when bare `0` token present (word-boundary rule prevents `0077` from matching)
  - `SetSecurityDescriptorDacl` with `arg_value_contains: Some(&["NULL"])` — permissive DACL via null pointer. Windows-only in practice; xcar-linux won't produce findings but the rule costs nothing.

### CWE-369 detection (ARGVAL-04)
- **D-09:** Separate code path in `scan_file` — NOT a `CweRule` entry. After the main `CWE_RULES` loop, check each line for the pattern `[/%]\s*0[^0-9.]` (division or modulo by standalone zero, allowing whitespace). Also catches `/ 0` and `% 0` with spaces.
- **D-10:** This path emits a `SastFinding` with `cwe_id: 369` directly. Component name/ecosystem come from the same parameters as the `CweRule` loop.

### Rule table count test
- **D-11:** Update `test_rule_table_has_fourteen_cwes` to assert 18 distinct CWE IDs (adds 295, 319, 732, 369 to the existing 14). Rename the test and update its doc comment.

### Test approach
- **D-12:** New ARGVAL tests follow the existing pattern: `tempfile::tempdir()` + inline C code written as a string, then `scan_file` called on the temp `.c` file. One test per ARGVAL requirement. No new test infrastructure or `tests/fixtures/` directory.
- **D-13:** ARGVAL-05 backward compat is implicitly covered — all existing tests must still pass after adding `arg_value_contains: None` to the 14+ existing rules. No explicit separate test needed.

### Fallback mode and output pipeline
- **D-14:** No changes needed. New `CweRule` entries live in the same `CWE_RULES` table consumed by `scan_file`. The Phase 11 fallback mode (scan from `scan_root` when `component_dirs` is empty) calls the same `scan_file` — new rules apply automatically.
- **D-15:** `_static_analysis.md` report and CycloneDX output consume `SastFinding.cwe_id` generically. No output format changes needed in Phase 13.

### Claude's Discretion
- `SetSecurityDescriptorDacl` with NULL token: included per ARGVAL-03 requirement wording (user chose "you decide" — Claude included it as a zero-cost rule that satisfies the requirement even though xcar-linux is Linux-only).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 13 requirements
- `.planning/REQUIREMENTS.md` §Argument-Value Matching — ARGVAL-01 through ARGVAL-05 (5 requirements for this phase)

### Existing scanner implementation (extend, don't rewrite)
- `src/vulnerability/cwe_scanner.rs` — Phase 11 lexical scanner. Contains `CweRule` struct, `CWE_RULES` static table, `find_function_call`, `format_arg_is_literal`, `scan_file`, `run_lexical_scanner`. Phase 13 extends this file in-place.

### Phase scope and success criteria
- `.planning/ROADMAP.md` §Phase 13 — Goal, success criteria (5 items), requirements list

### Project constraints
- `.planning/PROJECT.md` §Constraints — Rust-only, internal feature gate, musl target

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `find_function_call(line: &str, func: &str) -> Option<usize>`: word-boundary + paren-required match. Returns byte offset. Phase 13's `paren_args_contain_all` should apply the same word-boundary logic for token matching inside the arg slice.
- `format_arg_is_literal(after_func: &str, arg_index: u8) -> bool`: paren-slice parser that skips N args to find a string literal. The `paren_args_contain_all` helper is a sibling to this function — same input (`after_func` slice starting at `(`), different output (checks for token presence rather than literal type).
- `CWE_RULES: &[CweRule]`: static slice of all rules. All existing entries need `arg_value_contains: None` added. New entries added at the bottom grouped by CWE.

### Established Patterns
- All scanner code is gated with `#![cfg(feature = "internal")]` at the module level. No per-function cfg needed.
- `scan_file` returns `Vec<SastFinding>` and is called per-file. CWE-369 detection emits into the same vec.
- Errors in file I/O are graceful (continue/warn, never abort) — maintain this in any new code.
- `run_lexical_scanner` owns the per-component-dir walk. Dedup HashSet goes here, after collecting all findings.

### Integration Points
- `run_lexical_scanner` is the public entry point called by the scanner orchestration in `scanner/mod.rs`. Its return type `Vec<SastFinding>` is unchanged.
- `SastFinding` struct is unchanged — new CWEs flow through the existing CycloneDX and report pipeline without modification.

</code_context>

<specifics>
## Specific Ideas

- `paren_args_contain_all` signature: `fn paren_args_contain_all(after_func: &str, tokens: &[&str]) -> bool` — takes the raw line slice starting from `(`, extracts everything up to the matching `)` (tracking paren depth), then checks each token with word-boundary rules.
- Numeric token boundary rule: for a token like `"0"`, the char before must be non-digit and the char after must be non-digit and non-`.` (to exclude `0.0`, `0x...`, `0777`).
- CWE-369 pattern: after the main `CWE_RULES` loop per line, apply `if line contains [/%]\s*0[^0-9.]` → emit `SastFinding { cwe_id: 369, ... }`. Use a simple byte scan or compiled regex — team preference is no external regex crate, so implement as a manual scan.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 13-argument-value-matching*
*Context gathered: 2026-05-10*
