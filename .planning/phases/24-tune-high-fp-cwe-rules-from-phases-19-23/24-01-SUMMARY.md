---
phase: 24-tune-high-fp-cwe-rules-from-phases-19-23
plan: "01"
subsystem: vulnerability/ast-scanner
tags: [cwe, rust, ast-scanner, tightening, function-list, phase-24]
dependency_graph:
  requires: []
  provides: [tightened-CWE-338, tightened-CWE-426, tightened-CWE-676, tightened-CWE-780, removed-CWE-256-call]
  affects: [src/vulnerability/ast_scanner.rs, tests/vulnerability_tests/ast_scanner_tests.rs]
tech_stack:
  added: []
  patterns: [subtractive-table-edit, synthetic-tp-tn-unit-tests]
key_files:
  created: []
  modified:
    - src/vulnerability/ast_scanner.rs
    - tests/vulnerability_tests/ast_scanner_tests.rs
decisions:
  - "CWE-338 tightened to drand48/lrand48/random/mrand48; rand()/srand() removed (D-04)"
  - "CWE-426 replaced popen/_popen/system with dlopen/LoadLibraryExA/LoadLibraryExW (D-06)"
  - "CWE-676 dropped alloca, kept only strtok (D-05)"
  - "CWE-780 removed both RSA_public_encrypt entries; kept CryptEncrypt ArgAtIndex(3,0) only (D-07)"
  - "CWE-256 call removed from apply_ast_rules(); fn definition left as dead code (D-03)"
  - "Existing CWE-256 tests inverted to assert no-fire (consequence of D-03 removal)"
metrics:
  duration: "~20m"
  completed: "2026-05-13"
  tasks_completed: 2
  files_modified: 2
---

# Phase 24 Plan 01: AST CWE Function-List Tightening (CWE-256/338/426/676/780) Summary

Wave 1 subtractive table-only edits to AST_CWE_RULES: removed CWE-256 detection call, tightened function lists for CWE-338/426/676/780, updated 5 existing tests, and added 7 synthetic TP/TN unit tests proving the tightened rules still fire and no-longer-fire as expected.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Remove CWE-256 call + apply CWE-338/426/676/780 function-list edits | 9093f19 | src/vulnerability/ast_scanner.rs |
| 2 | Update existing tests + add 7 synthetic TP/TN tests | 9a771c3 | tests/vulnerability_tests/ast_scanner_tests.rs |

## What Was Built

Five surgical edits to `AST_CWE_RULES` and `apply_ast_rules()` in `ast_scanner.rs`:

1. **CWE-256 (D-03):** Removed `check_plaintext_password(root, src, ...)` call from `apply_ast_rules()`. Function definition retained as dead code (compiler permits this).

2. **CWE-338 (D-04):** Replaced `&["rand", "random", "srand"]` with `&["drand48", "lrand48", "random", "mrand48"]`. `rand()`/`srand()` were producing 99.9% FP on Juliet corpus.

3. **CWE-426 (D-06):** Replaced `&["popen", "_popen", "system"]` with `&["dlopen", "LoadLibraryExA", "LoadLibraryExW"]`. Prior list mapped to CWE-78 in the oracle, causing oracle-mismatch FPs.

4. **CWE-676 (D-05):** Replaced `&["alloca", "strtok"]` with `&["strtok"]`. `alloca` already fires CWE-121; removing it reduces noise.

5. **CWE-780 (D-07):** Removed both `RSA_public_encrypt` entries (one for `RSA_PKCS1_PADDING`, one for `RSA_NO_PADDING`). Kept only `CryptEncrypt ArgAtIndex(3, &["0"])`. The RSA entries fired in every OpenSSL file.

Test changes: 5 existing tests updated to match new function lists (3 per plan spec + 2 CWE-256 tests inverted per D-03); 7 new `phase_24_cwe_*` tests added covering drand48 TP, rand TN, dlopen TP, popen TN, strtok TP, alloca TN, RSA_public_encrypt TN.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated CWE-256 tests that became broken by Task 1's D-03 removal**
- **Found during:** Task 2 test run
- **Issue:** `test_cwe256_password_string_literal` and `test_cwe256_pwd_uppercase` asserted CWE-256 fires, but D-03 removes the detection call. Tests would fail with no code changes to those tests.
- **Fix:** Inverted assertions in both CWE-256 tests to assert CWE-256 does NOT fire, documenting the Phase 24 D-03 removal. The third CWE-256 test (`test_cwe256_non_password_var_no_finding`) was already asserting no-fire — left unchanged.
- **Files modified:** `tests/vulnerability_tests/ast_scanner_tests.rs`
- **Commit:** 9a771c3

## Verification Results

- `cargo build --features internal` exits 0
- All 7 `phase_24_cwe_*` tests pass
- `test_cwe_338_weak_prng` passes (fixture updated to drand48)
- `test_cwe_426_untrusted_search_path` passes (fixture updated to dlopen)
- `test_cwe_780_rsa_no_oaep` passes (fixture updated to CryptEncrypt)
- Pre-existing failure `test_spdx_output_passes_pyspdxtools_validation` is a missing fixture (`example_target_repos/rclcpp` absent in worktree) — pre-existing, unrelated to this plan.

## Known Stubs

None — no stub patterns found in modified files.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- `src/vulnerability/ast_scanner.rs` exists: FOUND
- `tests/vulnerability_tests/ast_scanner_tests.rs` exists: FOUND
- Commit 9093f19 exists in git log: FOUND
- Commit 9a771c3 exists in git log: FOUND
- All acceptance criteria grep checks: PASSED
- `cargo build --features internal`: PASSED
- 7 phase_24 tests: PASSED
