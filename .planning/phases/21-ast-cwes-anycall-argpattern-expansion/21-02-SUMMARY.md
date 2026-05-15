---
phase: 21-ast-cwes-anycall-argpattern-expansion
plan: 02
subsystem: vulnerability/ast_scanner
tags: [ast, tree-sitter, cwe, rust, tdd]

requires:
  - phase: 21-01
    provides: [ArgCheck::SizeofPointer, apply_division_rules, CWE-467 rule entry already in AST_CWE_RULES]
  - phase: 20
    provides: [ArgCheck::ArgAtIndex with ALL-OF token semantics (tokens.iter().all(...))]

provides:
  - 12 new AstCweRule entries in AST_CWE_RULES (CWE-121, 126, 328×3, 338, 426, 526, 535, 676, 680, 780×3)
  - 10 new unit tests in ast_scanner_tests.rs covering all 10 new CWEs
  - Total AST_CWE_RULES coverage: 25 CWEs (13 pre-Phase-21 + 12 Phase-21)

affects: [phase-22-ast-cwes-structuralpattern-expansion, phase-23-ast-cwes-domainspecific-expansion]

tech-stack:
  added: []
  patterns:
    - "ALL-OF split: ArgAtIndex with multiple-token semantics requires one rule entry per token to achieve any-of detection"
    - "CryptEncrypt ArgAtIndex(3, &[\"0\"]) mirrors umask(0) pattern — number_literal kind-check prevents nested-call FP"

key-files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs

key-decisions:
  - "CWE-328 uses 3 entries (one per CALG_ constant) because ArgAtIndex ALL-OF semantics would require all tokens simultaneously — each entry covers one weak constant independently"
  - "CWE-780 uses 3 entries: 2 for RSA_public_encrypt (RSA_PKCS1_PADDING + RSA_NO_PADDING), 1 for CryptEncrypt literal-0 (Juliet TP)"
  - "CWE-126 uses FixedSizeBuffer (not AnyCall) on strcat/strncat — matches existing CWE-119/120/122/125 pattern, reduces FP rate"
  - "CWE-676 tight list [alloca, strtok] only — gets/system/rand/getenv/popen excluded to avoid duplicate CWE-id findings"
  - "CWE-369 remains absent from AST_CWE_RULES — handled by apply_division_rules() binary_expression walk (D-01)"

patterns-established:
  - "Phase-block comment: each expansion phase gets a delimited block in AST_CWE_RULES with inline rationale for entry-count splits"

requirements-completed: [CWEXP-01]

duration: ~20min
completed: "2026-05-12"
---

# Phase 21 Plan 02: AST CWE AnyCall/ArgPattern Expansion (Rule Table) Summary

**12 new AstCweRule entries added to AST_CWE_RULES covering CWE-121/126/328×3/338/426/526/535/676/680/780×3 with 10 new unit tests; full cargo test suite green at 385 passed**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-05-12
- **Completed:** 2026-05-12
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Appended 12 new AstCweRule entries to AST_CWE_RULES (Phase 21 expansion block, after Plan 01's CWE-467 entry)
- Correctly applied ALL-OF split for CWE-328 (3 entries: CALG_MD2/CALG_MD5/CALG_SHA1) and CWE-780 (2 RSA_public_encrypt entries + 1 CryptEncrypt entry)
- Added 10 unit tests with TP assertions and FP guards where applicable; all pass
- Phase 18 `test_ast_all_tractable_cwes` still passes (no regression)

## New AstCweRule Entries

| CWE | Functions | ArgCheck | Note |
|-----|-----------|----------|------|
| 121 | alloca | AnyCall | D-13, D-14 |
| 126 | strcat, strncat | FixedSizeBuffer | D-06; mirrors CWE-119/120/122/125 |
| 328 | CryptCreateHash | ArgAtIndex(1, &["CALG_MD2"]) | ALL-OF split entry 1/3 |
| 328 | CryptCreateHash | ArgAtIndex(1, &["CALG_MD5"]) | ALL-OF split entry 2/3 |
| 328 | CryptCreateHash | ArgAtIndex(1, &["CALG_SHA1"]) | ALL-OF split entry 3/3 |
| 338 | rand, random, srand | AnyCall | D-13, D-14 |
| 426 | popen, _popen, system | AnyCall | D-13; duplicate CWE-78 findings acceptable |
| 526 | getenv | AnyCall | D-13 |
| 535 | fprintf, vfprintf | ArgAtIndex(0, &["stderr"]) | stderr token in first arg |
| 676 | alloca, strtok | AnyCall | D-08, D-09; tight list |
| 680 | malloc, calloc, realloc | AnyCall | D-13; duplicate CWE-190 findings acceptable |
| 780 | RSA_public_encrypt | ArgAtIndex(4, &["RSA_PKCS1_PADDING"]) | D-10; ALL-OF split entry 1/2 |
| 780 | RSA_public_encrypt | ArgAtIndex(4, &["RSA_NO_PADDING"]) | D-10; ALL-OF split entry 2/2 |
| 780 | CryptEncrypt | ArgAtIndex(3, &["0"]) | Juliet TP; ROADMAP SC#1 |

## New Unit Tests

| Test | Type | Notes |
|------|------|-------|
| test_cwe_121_anycall_alloca | TP only | alloca call |
| test_cwe_126_fixed_size_buffer | TP + FP guard | FP: strcat into pointer param |
| test_cwe_328_weak_hash_argindex | TP + FP guard | TP: CALG_MD2; FP: CALG_SHA_256 |
| test_cwe_338_weak_prng | TP only | rand() |
| test_cwe_426_untrusted_search_path | TP only | popen |
| test_cwe_526_env_exposure | TP only | getenv |
| test_cwe_535_shell_error_stderr | TP + FP guard | FP: fprintf(stdout,...) |
| test_cwe_676_dangerous_function_strtok | TP only | strtok (exercises 2nd function in list) |
| test_cwe_680_integer_overflow_alloc | TP only | malloc(data * sizeof(int)) |
| test_cwe_780_rsa_no_oaep | TP + FP guard × 2 | TP: RSA_PKCS1_PADDING; FP: RSA_PKCS1_OAEP_PADDING; TP: CryptEncrypt dwFlags=0; FP: CryptEncrypt CRYPT_OAEP |

## ALL-OF Token Semantics Rationale

`ArgAtIndex` uses `tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))` — ALL tokens must be present simultaneously in the arg's subtree text. For "any-of" detection (fire when constant A OR constant B appears), multiple single-token rule entries are required. CWE-328 and CWE-780 both need this splitting:

- **CWE-328**: 3 weak CALG_ constants → 3 rule entries
- **CWE-780 RSA**: 2 insecure padding constants → 2 rule entries

The CWE-780 CryptEncrypt entry uses the literal `"0"` token, which triggers the umask-style `number_literal` kind-check branch in the ArgAtIndex arm (prevents `compute_mask(0)` FP).

## Task Commits

1. **Task 1: Append 12 new AstCweRule entries** - `696b07a` (feat)
2. **Task 2: Add 10 unit tests for new CWEs** - `3ff11aa` (test)

## Files Created/Modified

- `src/vulnerability/ast_scanner.rs` — 12 new AstCweRule entries appended in Phase 21 expansion block
- `tests/vulnerability_tests/ast_scanner_tests.rs` — 10 new unit tests appended

## Decisions Made

- CWE-328 and CWE-780 split into multiple entries due to ArgAtIndex ALL-OF semantics (confirmed from Phase 20 20-01-PLAN.md line 34)
- CWE-780 CryptEncrypt entry added for Juliet TP (ROADMAP SC#1) using literal "0" ArgAtIndex pattern
- CWE-369 kept absent from AST_CWE_RULES (D-01 preserved; handled by apply_division_rules)
- CWE-605 deferred per D-12 (requires cross-call fd tracking)

## Deviations from Plan

None — plan executed exactly as written. All 12 rule entries and 10 tests match the verbatim specifications in the plan's action blocks.

## Issues Encountered

None. All tests passed on first run after implementation.

## Phase 18 Regression Confirmation

`test_ast_all_tractable_cwes` passes (all 13 pre-Phase-21 CWEs still detected). Full suite: 385 passed; 0 failed.

## Known Stubs

None — all rule entries fire on real C source patterns and are exercised by unit tests.

## Threat Flags

No new threat surface introduced. All changes are analysis-only rule table additions; no new network, file, or auth paths.

## Next Phase Readiness

Plan 03 (Juliet corpus re-run + benchmark/juliet/ANALYSIS.md update) receives:
- 25-CWE AST scanner fully implemented (13 pre-Phase-21 + 12 Phase-21)
- All 10 new CWEs unit-tested with TP + FP guards
- CWE-780 CryptEncrypt Juliet TP entry in place (ROADMAP SC#1 satisfied at unit-test level)
- Full test suite green

---
*Phase: 21-ast-cwes-anycall-argpattern-expansion*
*Completed: 2026-05-12*

## Self-Check: PASSED

- [x] `src/vulnerability/ast_scanner.rs` modified (12 new rule entries)
- [x] `tests/vulnerability_tests/ast_scanner_tests.rs` modified (10 new tests)
- [x] Commit `696b07a` exists (Task 1)
- [x] Commit `3ff11aa` exists (Task 2)
- [x] `cargo build --features internal` exits 0
- [x] All 10 new tests pass
- [x] Full suite: 385 passed; 0 failed; no regressions
- [x] CWE-467 from Plan 01 still present
- [x] No ContainsTokens references
- [x] CWE-369 not in AST_CWE_RULES (in apply_division_rules only)
- [x] CWE-605 not present
