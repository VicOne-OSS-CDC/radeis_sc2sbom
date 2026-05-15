---
phase: 19-cppcheck-removal
reviewed: 2026-05-12T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - src/vulnerability/cwe_scanner.rs
  - src/vulnerability/mod.rs
  - src/cli.rs
  - src/main.rs
  - tests/vulnerability_tests/mod.rs
  - tests/vulnerability_tests/sarif_consistency_tests.rs
  - docs/BENCHMARK.md
findings:
  critical: 0
  warning: 4
  info: 2
  total: 6
status: issues_found
---

# Phase 19: Code Review Report

**Reviewed:** 2026-05-12T00:00:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Phase 19 removes cppcheck from the scanning pipeline, promoting the AST scanner (tree-sitter) as the sole primary scanner with the lexical scanner as a per-file fallback. The overall design is sound and the implementation is largely correct. The review surfaced no blockers. Four warnings are present: one silent-failure path that swallows SAST findings when the user omits `--output` with `--sarif-baseline`, a divergence between the lexical and AST scanner CWE-319 rule sets that creates a detection gap, a lexical CWE-295 rule that covers `wolfSSL_CTX_set_verify` while the AST scanner does not, and a `std::process::exit(1)` call that bypasses Rust's normal cleanup path. Two info items cover a dead re-export and a minor comment staleness.

## Warnings

### WR-01: `--sarif-baseline` is silently ignored when `--output` is absent with `--format console`

**File:** `src/main.rs:267-329`

**Issue:** The baseline-comparison block (`args.sarif_baseline`) is nested inside `if let Some(ref out) = args.output { ... }`. When a user runs:

```
sc2sbom --format console --check-vulnerabilities --sarif-baseline old.sarif
```

without `--output`, the code falls into the `else` branch (`print_sbom`), and the baseline comparison is **never executed**. The CI gate therefore silently passes even when new findings exist. The same nesting structure appears for `OutputFormat::Console` (line 292) and `OutputFormat::All` (line 448). `OutputFormat::All` always derives an output directory (`out_dir_str = args.output.as_deref().unwrap_or("out")`), so that path is safe. Only `OutputFormat::Console` without `--output` is affected.

**Fix:** Either document and enforce (via `clap` `requires`) that `--sarif-baseline` requires `--output` when using `--format console`, or move the baseline comparison outside the `if let Some(ref out) = args.output` guard so it runs regardless of whether file output was requested:

```rust
// After `print_sbom(...)` in the else branch:
#[cfg(feature = "internal")]
if let Some(ref baseline_path) = args.sarif_baseline {
    // baseline comparison needs an out_dir; emit a clear error if missing
    eprintln!("Error: --sarif-baseline requires --output when using --format console");
    std::process::exit(2);
}
```

Alternatively, emit a warning in the `else` branch analogous to the warnings already emitted for `--sarif-output` in `SpdxJson`/`CyclonedxJson` modes.

---

### WR-02: AST scanner CWE-319 rules do not cover `CURLOPT_SSL_VERIFYPEER` and `CURLOPT_SSL_VERIFYHOST`

**File:** `src/vulnerability/ast_scanner.rs:65-66` vs `src/vulnerability/cwe_scanner.rs:92-93`

**Issue:** The lexical scanner contains two CWE-319 rules for `curl_easy_setopt` that gate on `CURLOPT_SSL_VERIFYPEER + "0"` and `CURLOPT_SSL_VERIFYHOST + "0"`:

```rust
// cwe_scanner.rs lines 92-93
CweRule { cwe_id: 319, functions: &["curl_easy_setopt"], ..., arg_value_contains: Some(&["CURLOPT_SSL_VERIFYPEER", "0"]) },
CweRule { cwe_id: 319, functions: &["curl_easy_setopt"], ..., arg_value_contains: Some(&["CURLOPT_SSL_VERIFYHOST", "0"]) },
```

The AST scanner has only:

```rust
// ast_scanner.rs lines 65-66
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"], arg_check: ArgCheck::ContainsTokens(&["CURLOPT_USE_SSL"]) },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"], arg_check: ArgCheck::ContainsTokens(&["CURLUSESSL_NONE"]) },
```

When tree-sitter successfully parses a file (no errors), the AST scanner runs and its result is used directly — the lexical scanner is NOT called as a fallback for that file (it is only called on parse failure). Therefore the two `CURLOPT_SSL_VERIFYPEER=0` / `CURLOPT_SSL_VERIFYHOST=0` patterns are **never detected** on successfully-parsed files after Phase 19.

**Fix:** Add the missing rules to `AST_CWE_RULES`:

```rust
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"], arg_check: ArgCheck::ContainsTokens(&["CURLOPT_SSL_VERIFYPEER", "0"]) },
AstCweRule { cwe_id: 319, functions: &["curl_easy_setopt"], arg_check: ArgCheck::ContainsTokens(&["CURLOPT_SSL_VERIFYHOST", "0"]) },
```

---

### WR-03: AST scanner does not cover `wolfSSL_CTX_set_verify` for CWE-295

**File:** `src/vulnerability/ast_scanner.rs:62` vs `src/vulnerability/cwe_scanner.rs:89`

**Issue:** The lexical CWE-295 rule covers three functions:

```rust
// cwe_scanner.rs line 89
functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"]
```

The AST rule covers only two:

```rust
// ast_scanner.rs line 62
functions: &["SSL_CTX_set_verify", "SSL_set_verify"]
```

On files that tree-sitter parses without errors, `wolfSSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL)` will not be flagged. This is a silent detection regression for wolfSSL-based codebases.

**Fix:** Add `wolfSSL_CTX_set_verify` to the AST CWE-295 rule:

```rust
AstCweRule { cwe_id: 295, functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"], arg_check: ArgCheck::ContainsTokens(&["SSL_VERIFY_NONE"]) },
```

---

### WR-04: `std::process::exit(1)` called inside `main()` bypasses `Result`-based cleanup

**File:** `src/main.rs:309, 465`

**Issue:** Two identical code paths call `std::process::exit(1)` after detecting new baseline findings. This abruptly terminates the process without running destructors, flushing file handles, or returning `Err(...)` up the call chain as is idiomatic in a Rust `fn main() -> Result<()>`. While typically benign here (files are already written before the exit), it prevents the compiler and runtime from running any registered cleanup (e.g., `tempfile` drop guards, buffered writers). Both call sites are structurally identical:

```rust
// lines 304-309 and 460-465
if new_count > 0 {
    eprintln!(...);
    std::process::exit(1);  // <-- bypasses cleanup
}
```

**Fix:** Return a typed exit-code error through `anyhow`, or use a sentinel error type:

```rust
if new_count > 0 {
    eprintln!("{} new finding(s) vs baseline {} — CI gate failed", new_count, baseline_path);
    // anyhow provides a way to carry an exit code
    return Err(anyhow::anyhow!("CI gate: {} new finding(s) detected", new_count));
}
```

If a non-1 exit code is required, use `std::process::ExitCode` via the `Termination` trait rather than `exit()`.

---

## Info

### IN-01: `run_lexical_scanner` re-exported from `vulnerability/mod.rs` but unused in `main.rs`

**File:** `src/vulnerability/mod.rs:16`

**Issue:** `run_lexical_scanner` is included in the public re-export:

```rust
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, run_lexical_scanner, SastFinding, SastSource};
```

After Phase 19, `main.rs` calls only `run_ast_scanner`; `run_lexical_scanner` is called from within `ast_scanner.rs` as a file-level fallback (not from the public API). Exporting it widens the public surface. It is still used directly by integration tests (`cwe_scanner_tests.rs`, `ast_scanner_tests.rs`), so removing it would break those. Consider marking it `pub(crate)` or restricting to `#[cfg(test)]` if tests are the only caller.

**Fix:** If external callers other than tests do not need it, change to `pub(crate)`:
```rust
pub(crate) use cwe_scanner::run_lexical_scanner;
```

---

### IN-02: Module-level doc comment in `cwe_scanner.rs` references Phase 11 and does not reflect Phase 19 changes

**File:** `src/vulnerability/cwe_scanner.rs:1-11`

**Issue:** The module doc comment reads "Phase 11 (v1.0.16): Pure-Rust lexical CWE scanner" and "All items in this module are gated behind `feature = \"internal\"`." Phase 19 promotes the AST scanner as primary and demotes this module to fallback role. The doc comment does not mention this demotion or cross-reference `ast_scanner.rs`. Minor, but misleading for future maintainers.

**Fix:** Add a one-line note to the module doc comment, e.g.:

```rust
//! Phase 19 (v1.0.18): demoted to fallback role; primary scanner is `ast_scanner`.
//! Called by `ast_scanner::scan_file_ast_or_lexical` on parse failure.
```

---

_Reviewed: 2026-05-12T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
