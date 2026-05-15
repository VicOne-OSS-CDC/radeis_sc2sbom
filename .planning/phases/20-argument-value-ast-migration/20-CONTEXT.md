# Phase 20: argument-value-ast-migration — Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Migrate the CWE-295, CWE-319, and CWE-732 argument-value detection rules in `ast_scanner.rs` from `ArgCheck::ContainsTokens` to a new `ArgCheck::ArgAtIndex(u8, &'static [&'static str])` variant that inspects a specific positional argument and walks its nested AST subtree. Remove the corresponding `arg_value_contains` rules from `cwe_scanner.rs` (the lexical scanner). Add synthetic `.c` fixture files to validate true-positive detection and false-positive guards for each rule.

</domain>

<decisions>
## Implementation Decisions

### ArgCheck Variant Design

- **D-01:** Add `ArgAtIndex(u8, &'static [&'static str])` to the `ArgCheck` enum in `ast_scanner.rs`. The `u8` is the 0-based positional argument index; the `&'static [&'static str]` is the token slice that must ALL be present within that arg's AST subtree.
- **D-02:** The nested walk within `ArgAtIndex`: recursively collect all leaf node texts within the arg's AST subtree, then apply `token_present_with_boundary` on the concatenated text for each required token. Reuses the existing `token_present_with_boundary` logic already imported from `cwe_scanner.rs`.
- **D-03:** When the arg count is fewer than the required index (`idx >= args.len()`): skip silently — return `false`, no finding, no panic. Mirrors the existing `NotStringLiteralAtIndex` behavior (line ~219 in `ast_scanner.rs`).
- **D-04:** Delete `ArgCheck::ContainsTokens` from the enum entirely in Phase 20. All three uses (CWE-295/319/732) are replaced by `ArgAtIndex`. Phases 21–23 should use `ArgAtIndex` from the start for any argument-value rules — no need to keep `ContainsTokens`.

### CWE-295 Migration (SSL Certificate Verification)

- **D-05:** Migrate `SSL_CTX_set_verify` and `SSL_set_verify` rules to `ArgAtIndex(1, &["SSL_VERIFY_NONE"])` — the verify mode is argument index 1 (0-based: handle, mode, callback).
- **D-06:** Add `wolfSSL_CTX_set_verify` to the AST rules (it was in the lexical scanner at `cwe_scanner.rs:93` but missing from `AST_CWE_RULES`). Rule: `ArgAtIndex(1, &["SSL_VERIFY_NONE"])`.
- **D-07:** Keep `SSL_CTX_set_cert_verify_callback` as `ArgCheck::AnyCall` — its current rule is already correct and doesn't use argument-value inspection.

### CWE-319 Migration (Cleartext Transmission — curl)

- **D-08:** Three separate `ArgAtIndex` rules for `curl_easy_setopt`:
  - Rule 1: `ArgAtIndex(1, &["CURLOPT_USE_SSL"])` + `ArgAtIndex(2, &["CURLUSESSL_NONE"])` — detect USE_SSL set to NONE. **Note:** Since a single `ArgAtIndex` checks one positional arg, this AND-condition across two positional args requires either two rules OR a new approach. See D-09.
- **D-09:** For CWE-319 rules where the dangerous condition spans two positional args (option constant at arg 1, value at arg 2), use two separate `AstCweRule` entries with `ArgAtIndex` per arg, and let the deduplication step merge them — OR use the existing `ContainsTokens`-style scan but scoped to the full arg list text. **Decision: keep the 3-rule structure but each rule uses `ArgAtIndex` on the arg that uniquely identifies the danger:** `ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` checks that arg 1's subtree contains BOTH tokens (or the value token is in the option arg). This mirrors what ContainsTokens already does but is positionally scoped to arg 1. For `VERIFYPEER`/`VERIFYHOST` + `0`, the `0` in arg 2 is the dangerous value — use `ArgAtIndex` across both args via the same text-contains approach across the full arg list but only for the specific known-dangerous combinations.
  - Revised rule structure:
    - `ArgAtIndex(1, &["CURLOPT_USE_SSL", "CURLUSESSL_NONE"])` — arg 1 contains BOTH tokens (the option + the dangerous value enum are serialized adjacently in the source).
    - `ArgAtIndex(1, &["CURLOPT_SSL_VERIFYPEER"])` with arg 2 text == `"0"` — but since `ArgAtIndex` checks one arg, use `ArgAtIndex(1, &["CURLOPT_SSL_VERIFYPEER"])` AND a separate `ArgAtIndex(2, &["0"])` — planner to decide the best way to AND these without adding a new variant.
  - **Planner guidance:** If AND-ing two `ArgAtIndex` checks requires a new enum variant, introduce `ArgAtTwoIndices(u8, &'static [&'static str], u8, &'static [&'static str])` only if needed to avoid false positives. Otherwise, fall back to checking only the option-name arg (arg 1) for the unique constant — that alone is sufficient to identify the dangerous call pattern.

### CWE-732 Migration (Permissive Permissions)

- **D-10:** `umask` rule: `ArgAtIndex(0, &["0"])` but with an **exact integer_literal check**: the arg at index 0 must be an `integer_literal` AST node AND its text must be exactly `"0"`. This fixes the current `ContainsTokens(&["0"])` FP where `umask(0077)` fires because it contains a '0'. The exact check: `args[0].kind() == "integer_literal" && args[0].utf8_text(src) == Ok("0")`.
- **D-11:** `SetSecurityDescriptorDacl` rule: `ArgAtIndex(2, &["NULL"])` — check that arg at index 2 (pDacl) contains `NULL`. The current `ContainsTokens` fires if NULL appears anywhere; positional scoping to arg 2 is more precise.

### Lexical Scanner Cleanup

- **D-12:** Remove the `arg_value_contains` rules for CWE-295, CWE-319, and CWE-732 from `cwe_scanner.rs`. Specifically: delete the 5 `CweRule` entries at lines 93–100 (CWE-295: 1 rule, CWE-319: 3 rules, CWE-732: 2 rules). The AST scanner is now authoritative for these CWEs; parse-fail files fall back to lexical scan but will no longer detect these argument-value patterns (acceptable — parse-fail files are rare on well-formed C code).
- **D-13:** The `arg_value_contains: Option<&'static [&'static str]>` field and the `paren_args_contain_all` check in `cwe_scanner.rs::scan_file` (lines 368–373) should be removed along with the rules. The field becomes dead code once all `arg_value_contains: Some(...)` rules are deleted.

### Test / Validation Strategy

- **D-14:** Create synthetic `.c` fixture files under `tests/fixtures/` for each migrated CWE rule. Each fixture must have:
  - A "should fire" case (TP): the dangerous call pattern
  - A "should not fire" case (FP guard): a safe variant (e.g., `umask(0077)`, `SSL_VERIFY_PEER`, `CURLOPT_SSL_VERIFYPEER = 2`)
  - A "nested expression" case for Phase 20 success criterion #3: the dangerous arg buried in a binary/cast expression
- **D-15:** Wire fixtures into `#[test]` functions in `ast_scanner.rs` (or a dedicated test file under `tests/scanner_tests/`) using `run_ast_scanner` or the internal `apply_ast_rules` test helper pattern.
- **D-16:** Juliet test suite does NOT cover CWE-295/319/732 (Juliet focuses on buffer overflows and string handling). Validation relies entirely on the synthetic fixture files (D-14). No manifest.xml converter needed for Phase 20.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — ARGVAL-01 and ARGVAL-02 (the two requirements for this phase)
- `.planning/ROADMAP.md` §Phase 20 — success criteria and phase dependencies

### Code to Modify
- `src/vulnerability/ast_scanner.rs` — primary file: add `ArgAtIndex` variant, migrate CWE-295/319/732 rules, delete `ContainsTokens`
- `src/vulnerability/cwe_scanner.rs` — remove 5 `arg_value_contains` rules (lines ~93–100), remove `arg_value_contains` field from `CweRule` struct, remove `paren_args_contain_all` usage (lines ~368–373)
- `src/vulnerability/mod.rs` — check if `paren_args_contain_all` is re-exported; remove if so

### Prior Phase Context
- `.planning/phases/19-cppcheck-removal/19-CONTEXT.md` — D-05 (SastSource::Both repurposed), D-07 (deduplicate_sast_findings revised to AST+Lexical)
- `.planning/phases/18-ast-scanner-core-and-benchmark/18-CONTEXT.md` — D-02 (AST primary + lexical fallback), D-03 (SastSource::Ast variant)

### Existing Tests to Preserve/Mirror
- `src/vulnerability/cwe_scanner.rs` lines ~890–1000 — `test_argval_*` test functions to mirror in the AST scanner tests

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ArgCheck::NotStringLiteralAtIndex(u8)` in `ast_scanner.rs` — the pattern for positional arg index checking already exists. `ArgAtIndex(u8, &[&str])` follows the exact same shape.
- `token_present_with_boundary(text, tok)` imported from `cwe_scanner.rs` — reuse for the nested subtree text-contains check in `ArgAtIndex` evaluation.
- `args: Vec<Node>` already collected per call site (lines ~199–204 in `ast_scanner.rs`) — `ArgAtIndex` just indexes into this existing vec.

### Established Patterns
- `#[cfg(feature = "internal")]` gate — all SAST code is inside this feature; no change needed.
- `visit_node` recursion + `apply_ast_rules` pattern — new `ArgAtIndex` arm slots into the existing `match &rule.arg_check` block (lines ~211–256).
- `SastSource::Ast` on all findings from `ast_scanner.rs` — unchanged.

### Integration Points
- The existing `test_argval_cwe295_ssl_verify_none` and related tests in `cwe_scanner.rs` (lines ~890–1000) call the **lexical** `scan_file`. After D-12, those tests will still pass (they test the lexical scanner). New tests in `ast_scanner.rs` tests will test the **AST** scanner path. Both should exist and pass.
- `deduplicate_sast_findings` in Phase 19 (D-07) merges AST+Lexical findings. After Phase 20, CWE-295/319/732 findings will ONLY appear from the AST scanner (not lexical fallback). The dedup logic is unaffected.

</code_context>

<specifics>
## Specific Ideas

- The CWE-732/umask fix is the clearest precision improvement in Phase 20: `ContainsTokens(&["0"])` firing on `umask(0077)` is a concrete existing FP. The `integer_literal == "0"` exact check (D-10) directly fixes this.
- For CWE-319 AND-logic across two positional args: if a single `ArgAtIndex` variant can't cleanly express it, the planner should introduce `ArgAtTwoIndices` only if needed to prevent FPs. Option-name-only detection (checking arg 1 for `CURLOPT_SSL_VERIFYPEER`) is sufficient as a first approximation — the option name alone identifies the dangerous pattern.
- The `wolfSSL_CTX_set_verify` addition (D-06) fixes a gap where AST and lexical coverage diverged. After Phase 19 removes lexical fallback for these CWEs (D-12), wolfSSL code on parse-fail files would silently lose detection. Adding it to AST rules closes this.

</specifics>

<deferred>
## Deferred Ideas

- **Juliet manifest.xml integration for CWE-295/319/732** — Juliet doesn't have test cases for these CWEs. If the Juliet corpus is extended in the future to include OpenSSL/curl API misuse patterns, the Phase 18 manifest converter can be reused then.
- **`ArgAtTwoIndices` variant** — if the CWE-319 AND-logic across two positional args requires a second enum variant, it can be introduced in Phase 21 if the same pattern arises for newly added CWEs. Phase 20 should try to avoid adding it unless required.
- **CWE-732 zero-value family (`0x0`, `0b0`)** — extended zero-value literal matching deferred; exact `"0"` string check is sufficient for the known patterns in the AUTOSAR fixture.

</deferred>

---

*Phase: 20-argument-value-ast-migration*
*Context gathered: 2026-05-12*
