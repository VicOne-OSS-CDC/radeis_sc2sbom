---
phase: 19-cppcheck-removal
fixed_at: 2026-05-12T00:00:00Z
review_path: .planning/phases/19-cppcheck-removal/19-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 19: Code Review Fix Report

**Fixed at:** 2026-05-12T00:00:00Z
**Source review:** .planning/phases/19-cppcheck-removal/19-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01, WR-02, WR-03, WR-04)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-02 + WR-03: Missing AST scanner CWE rules (combined commit)

**Files modified:** `src/vulnerability/ast_scanner.rs`
**Commit:** c8211ef
**Applied fix:**
- WR-03: Added `wolfSSL_CTX_set_verify` to the CWE-295 AST rule's function list alongside `SSL_CTX_set_verify` and `SSL_set_verify`.
- WR-02: Added two new CWE-319 `AstCweRule` entries for `curl_easy_setopt` with `ContainsTokens(&["CURLOPT_SSL_VERIFYPEER", "0"])` and `ContainsTokens(&["CURLOPT_SSL_VERIFYHOST", "0"])`, mirroring the existing lexical scanner rules. Both were absent from `AST_CWE_RULES`, creating a detection gap on successfully-parsed files.

### WR-01: `--sarif-baseline` silently ignored without `--output` on console format

**Files modified:** `src/main.rs`
**Commit:** ff6c366
**Applied fix:** Added a `#[cfg(feature = "internal")]` warning block in the `else` branch of the `OutputFormat::Console` arm (i.e., when `args.output` is `None`). When `--sarif-baseline` is provided without `--output`, the user now sees: `Warning: --sarif-baseline has no effect without --output when using --format console; use --output <dir> to enable baseline comparison`. This matches the pattern already used for `--sarif-output` in SpdxJson/CyclonedxJson modes and prevents silent CI gate bypass.

### WR-04: `std::process::exit(1)` bypasses Rust cleanup path

**Files modified:** `src/main.rs`
**Commit:** 5cb0c61
**Applied fix:** Replaced both `std::process::exit(1)` call sites (in the `OutputFormat::Console` and `OutputFormat::All` baseline comparison blocks) with `return Err(anyhow::anyhow!("CI gate: {} new finding(s) detected vs baseline {}", new_count, baseline_path))`. The function signature is `fn main() -> Result<()>`, so this is idiomatic and allows destructors and buffered writers to flush before process exit. The exit code will be 1 as before (anyhow propagates to a non-zero exit via the `Termination` impl).

## Skipped Issues

None — all findings were fixed.

---

_Fixed: 2026-05-12T00:00:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
