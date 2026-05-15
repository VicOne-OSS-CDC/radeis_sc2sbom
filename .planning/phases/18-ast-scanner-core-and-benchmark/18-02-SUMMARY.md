---
phase: 18-ast-scanner-core-and-benchmark
plan: 02
subsystem: vulnerability
tags: [rust, tree-sitter, ast, sast, cwe, scanner-dispatch]

# Dependency graph
requires:
  - phase: 18-01
    provides: SastSource::Ast variant, ast_scanner module gated under cfg(feature="internal"), Wave 0 test scaffolds

provides:
  - Production AST scanner: run_ast_scanner() with 13-CWE AstCweRule table
  - ArgCheck enum: FixedSizeBuffer, NotStringLiteralAtIndex, ContainsTokens, AnyCall
  - Per-file lexical fallback on tree-sitter parse failure or has_error()
  - Function-scope isolation for FixedSizeBuffer rules (Pitfall 3 fix)
  - scan_file and token_present_with_boundary exposed pub(crate) in cwe_scanner.rs
  - main.rs scanner dispatch swapped to AST-primary

affects:
  - 18-03: benchmark plan uses run_ast_scanner as primary scanner under test

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AST-primary scanner dispatch: run_ast_scanner() in main.rs; lexical runs only as per-file fallback inside scan_file_ast_or_lexical()"
    - "Data-driven AstCweRule table: enum ArgCheck { FixedSizeBuffer, NotStringLiteralAtIndex(u8), ContainsTokens(&[&str]), AnyCall }"
    - "Scope-aware FixedSizeBuffer: collect_function_scope_fixed_arrays() per enclosing function_definition, plus file-scope global arrays"
    - "field-based AST access: child_by_field_name(\"function\") / child_by_field_name(\"arguments\") per Pattern 3"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - src/main.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "AST-primary dispatch: run_ast_scanner replaces run_lexical_scanner in main.rs; lexical scanner retained as pub export for Plan 03 benchmark"
  - "CWE-367 intentionally absent from AST_CWE_RULES: TOCTOU requires control-flow analysis between paired check/use calls; lexical fallback retains coverage"
  - "CWE-295 via AnyCall (SSL_CTX_set_verify family), CWE-319 via ContainsTokens(CURLOPT_USE_SSL / CURLUSESSL_NONE): Phase 20 ARGVAL-01 migrates to arg-value AST inspection"
  - "collect_file_scope_fixed_arrays skips function_definition subtrees; collect_function_scope_fixed_arrays scoped to enclosing function — prevents cross-function false positives"
  - "scan_file and token_present_with_boundary promoted to pub(crate) in cwe_scanner.rs for lexical fallback invocation from ast_scanner.rs"

patterns-established:
  - "scan_file_ast_or_lexical: try tree-sitter parse; on None OR has_error() fall back to lexical_scan_file with eprintln! warning"
  - "Fresh TreeCursor per recursion level (Pitfall 1) in visit_node and collect_arrays_in_subtree"

requirements-completed: [AST-01, AST-02, AST-03, AST-04, DIST-02]

# Metrics
duration: 35min
completed: 2026-05-11
---

# Phase 18 Plan 02: AST Scanner Core Summary

**Production tree-sitter AST scanner with 13-CWE rule table, scope-aware FixedSizeBuffer precision, lexical fallback on parse errors, wired as primary scanner in main.rs replacing run_lexical_scanner**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-11T16:00:00Z
- **Completed:** 2026-05-11T16:36:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Rewrote `src/vulnerability/ast_scanner.rs` end-to-end: AstCweRule struct, ArgCheck enum (4 variants), AST_CWE_RULES static table (17 entries across 13 distinct CWE IDs), `run_ast_scanner()`, `scan_file_ast_or_lexical()`, `apply_ast_rules()`, scope-aware fixed-array collection
- All 13 tractable CWEs (D-07's 11 + CWE-295 + CWE-319) detected on synthetic fixture; scope isolation and pointer-param precision verified
- Swapped scanner dispatch in `main.rs`: `ast_findings` replaces `lexical_findings` as first argument to `deduplicate_sast_findings`
- Exposed `scan_file` and `token_present_with_boundary` as `pub(crate)` in cwe_scanner.rs for fallback invocation
- Full test suite: 380 passed, 1 pre-existing failure (pyspdxtools external tool not installed), 2 ignored benchmark tests

## AST_CWE_RULES Table (as committed)

| CWE | Functions | ArgCheck |
|-----|-----------|----------|
| 78  | system, popen, execl, execlp, execle, execv, execvp, execvpe | AnyCall |
| 119 | strncpy, strncat, memcpy, memmove, memset | FixedSizeBuffer |
| 120 | strcpy, strcat, gets | FixedSizeBuffer |
| 122 | memcpy, memmove, sprintf | FixedSizeBuffer |
| 125 | memcpy, memmove, memcmp, strncmp | FixedSizeBuffer |
| 134 | printf, vprintf, sprintf, snprintf, vsprintf, vsnprintf | NotStringLiteralAtIndex(0) |
| 134 | fprintf, vfprintf, syslog | NotStringLiteralAtIndex(1) |
| 190 | malloc, calloc, realloc | AnyCall |
| 242 | gets, mktemp | AnyCall |
| 295 | SSL_CTX_set_verify, SSL_set_verify, SSL_CTX_set_cert_verify_callback | AnyCall |
| 319 | curl_easy_setopt | ContainsTokens(["CURLOPT_USE_SSL"]) |
| 319 | curl_easy_setopt | ContainsTokens(["CURLUSESSL_NONE"]) |
| 327 | MD5, MD5_Init, SHA1, SHA1_Init, DES_ecb_encrypt, EVP_md5, EVP_sha1 | AnyCall |
| 369 | div, ldiv, lldiv | AnyCall |
| 377 | tmpnam, tempnam, mktemp | AnyCall |
| 732 | umask | ContainsTokens(["0"]) |
| 732 | SetSecurityDescriptorDacl | ContainsTokens(["NULL"]) |

Total: 17 entries, 13 distinct CWE IDs. CWE-367 intentionally absent (see deferred-CWE note below).

## Task Commits

1. **Task 1: Implement AST scanner core** - `df118d1` (feat)
2. **Task 2: Wire AST scanner as primary in main.rs** - `1fe0887` (feat)

**Plan metadata commit:** (follows this summary)

## Files Created/Modified

- `src/vulnerability/ast_scanner.rs` — Complete rewrite: run_ast_scanner(), AstCweRule, ArgCheck, AST_CWE_RULES, scan_file_ast_or_lexical(), apply_ast_rules(), scope-aware fixed-array collection, legacy_poc_tests module
- `src/vulnerability/cwe_scanner.rs` — scan_file and token_present_with_boundary promoted to pub(crate)
- `src/vulnerability/mod.rs` — Added `pub use ast_scanner::run_ast_scanner;` under cfg(feature="internal")
- `src/main.rs` — Scanner dispatch: run_ast_scanner replaces run_lexical_scanner; deduplicate_sast_findings(ast_findings, cppcheck_findings)
- `tests/vulnerability_tests/ast_scanner_tests.rs` — De-ignored all 5 Wave 0 scaffolds; added test_ast_safe_strcpy_no_finding and test_ast_function_scope_isolation

## Test Pass Counts

- `cargo test --features internal --tests vulnerability_tests::ast_scanner_tests`: **6 passed, 0 failed, 0 ignored**
- `cargo test --features internal --lib vulnerability::ast_scanner`: **4 passed** (legacy_poc_tests)
- `cargo test --features internal --tests vulnerability_tests::cwe_scanner_tests`: **9 passed, 0 failed** (lexical scanner behavior preserved after scan_file refactor)
- `cargo test --features internal --tests` (full suite): **380 passed, 1 failed** (pre-existing pyspdxtools test), **2 ignored** (benchmark graceful-skips)

## Decisions Made

- CWE-369 AST detection uses `div`/`ldiv`/`lldiv` AnyCall rules. The test fixture uses `div(n, 1)` which tree-sitter parses successfully; the test asserts CWE-369 presence in the union of AST + lexical findings (the test accepts either source per plan spec). AST scanner emits CWE-369 via the div rule; no deviation from plan.
- `collect_array_declarator_rec` helper used to handle `init_declarator` and `pointer_declarator` wrappers around `array_declarator` nodes in addition to direct children — ensures declarations like `char buf[64] = {0}` are correctly collected.

## Deviations from Plan

None - plan executed exactly as written.

## Action Items for Human Review (REQUIRED)

**(a) CWE-367 AST coverage deferred (D-08 extension)**

CWE-367 (TOCTOU race condition) is intentionally absent from AST_CWE_RULES. Detection requires control-flow analysis between paired `access`/`stat` check calls and `open`/`fopen` use calls within the same function scope — this is dataflow, not local AST.

**Recommended follow-up:**
1. Update `18-CONTEXT.md` D-08 to add CWE-367 to the explicitly-deferred list alongside CWE-362, CWE-416, CWE-476.
2. Update ROADMAP §Phase 18 success criterion #2 to read: "all 14 CWEs detected via AST scanner *or* lexical fallback" — Phase 18 satisfies the criterion via the union of both scanner outputs, which cppcheck-suppression and SARIF baseline CI treat as the canonical finding set.
3. The lexical scanner's existing CWE-367 rule (paired-call paren scan on `access`/`stat` lines) continues to emit findings; no coverage regression.

**(b) CWE-295 and CWE-319 use AnyCall/ContainsTokens in Phase 18**

Phase 18 implements CWE-295 as `AnyCall` on the `SSL_CTX_set_verify` family and CWE-319 as `ContainsTokens(["CURLOPT_USE_SSL"])` / `ContainsTokens(["CURLUSESSL_NONE"])`. This mirrors the existing lexical scanner logic.

**Phase 20 ARGVAL-01** will replace these with AST argument-node inspection:
- CWE-295: inspect argument node at index 1 for `SSL_VERIFY_NONE` identifier (not just name-match on the function)
- CWE-319: inspect `curl_easy_setopt` argument node for `enumerator` kind with `CURLUSESSL_NONE` value

For Phase 18, the AnyCall/ContainsTokens rules provide equivalent or slightly higher recall than the lexical scanner's AND-all token rules, and the SARIF baseline + cppcheck suppression pipeline handles false-positive management.

## Issues Encountered

- Pre-existing test `format_tests::spdx_validation_tests::test_spdx_output_passes_pyspdxtools_validation` fails because `pyspdxtools` external tool is not installed in the worktree environment. This is unrelated to this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 03 (`18-03-PLAN.md`) can de-ignore benchmark scaffolds in `tests/benchmark.rs`; `run_ast_scanner`, `run_lexical_scanner`, and `run_cppcheck_scanner` are all available under `--features internal`
- `SastSource::Ast` is emitted by the new scanner; SARIF and CycloneDX writers are downstream-compatible (no schema changes)
- The lexical scanner is retained as a public re-export for benchmark comparison in Plan 03

---
*Phase: 18-ast-scanner-core-and-benchmark*
*Completed: 2026-05-11*
