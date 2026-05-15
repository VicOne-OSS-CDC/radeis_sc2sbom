---
phase: 11-lexical-scanner-cyclonedx-output
plan: "02"
subsystem: vulnerability/cwe_scanner
tags: [scanner, cwe, lexical, internal-feature-gate]
dependency_graph:
  requires: ["11-01"]
  provides: ["SastFinding struct", "run_lexical_scanner entry point", "CWE_RULES static table"]
  affects: ["src/vulnerability/cwe_scanner.rs", "src/vulnerability/mod.rs"]
tech_stack:
  added: []
  patterns: ["module-level #![cfg(feature=\"internal\")] gate", "WalkDir + filter_map(warn_on_walkdir_err)", "BufRead::lines() streaming per-line scan"]
key_files:
  created: ["src/vulnerability/cwe_scanner.rs"]
  modified: ["src/vulnerability/mod.rs"]
decisions:
  - "CWE_RULES table has 14 distinct CWE IDs (not 13) — the SEED-001 list in the plan's must_haves enumerates 14: 120, 78, 242, 327, 377, 190, 134, 22, 807, 362, 367, 20, 126, 676; the plan's body text saying '13 distinct' was a copy error in the narrative. Test assertion updated to 14."
  - "warn_on_walkdir_err path confirmed as crate::util::warn_on_walkdir_err (src/util/mod.rs line 8)"
  - "Module gated at file level with #![cfg(feature = \"internal\")] — all items excluded from default build without any per-item attributes"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-09"
  tasks_completed: 2
  files_created: 1
  files_modified: 1
---

# Phase 11 Plan 02: Lexical CWE Scanner Implementation Summary

Pure-Rust lexical CWE scanner with 14-entry CWE_RULES table (14 distinct CWE IDs), paren-bound word-boundary token matching, CWE-134 format-arg heuristic, and gated run_lexical_scanner entry point.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement cwe_scanner.rs | 3e5085d | src/vulnerability/cwe_scanner.rs |
| 2 | Wire cwe_scanner into vulnerability/mod.rs | f23551f | src/vulnerability/mod.rs |

## What Was Built

### src/vulnerability/cwe_scanner.rs

New file (replaces 2-line stub from Phase 10). Contains:

- `SastFinding` — public struct with cwe_id, component_name, component_ecosystem, file_path, line
- `CweRule` — private struct for the rule table
- `CWE_RULES` — static slice of 14 entries covering 14 distinct CWE IDs from SEED-001
- `format_arg_is_literal` — trims `(` and whitespace, checks if next char is `"` (CWE-134 safe-call heuristic)
- `find_function_call` — substring search with left word-boundary check and required `(` after identifier
- `scan_file` — streams file line-by-line via BufRead, gracefully returns empty vec on I/O errors
- `is_c_cpp_source` — extension filter for .c/.h/.cpp/.hpp/.cc
- `run_lexical_scanner` — public entry point; walks each component_dirs entry with WalkDir

The entire module is gated by `#![cfg(feature = "internal")]` at the file level.

### src/vulnerability/mod.rs

Replaced the ungated private `mod cwe_scanner;` stub declaration with:
- `#[cfg(feature = "internal")] pub mod cwe_scanner;`
- `#[cfg(feature = "internal")] pub use cwe_scanner::{run_lexical_scanner, SastFinding};`

## CWE_RULES Entry Count

14 entries in the table covering **14 distinct CWE IDs**:

| CWE | Functions (sample) |
|-----|--------------------|
| 120 | gets, strcpy, strcat, sprintf, vsprintf |
| 78  | system, popen, execl, execv, ... |
| 242 | gets, mktemp |
| 327 | MD5, SHA1, DES_ecb_encrypt, EVP_md5, EVP_sha1 |
| 377 | tmpnam, tempnam, mktemp |
| 190 | malloc, calloc, realloc |
| 134 | printf, fprintf, syslog, ... (format heuristic) |
| 22  | realpath, getcwd, open, fopen |
| 807 | getenv, getlogin, cuserid |
| 362 | access, stat, lstat |
| 367 | access, stat |
| 20  | atoi, atol, strtol, strtoul |
| 126 | strlen, wcslen |
| 676 | gets, scanf, fscanf, sscanf |

Total: 14 distinct IDs. The plan narrative mentioned "13" in several places but the SEED-001 enumeration in `must_haves` lists 14 IDs explicitly. The implementation follows the enumerated list; the test asserts 14.

## warn_on_walkdir_err Path

Confirmed correct: `crate::util::warn_on_walkdir_err` is defined at `src/util/mod.rs:8`. No adjustment needed.

## Build Verification

- `cargo build` (default): 0 errors, cwe_scanner module excluded
- `cargo build --features internal`: 0 errors, cwe_scanner compiles
- `cargo test --features internal --lib vulnerability::cwe_scanner`: 5/5 tests pass

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test assertion count corrected from 13 to 14 distinct CWE IDs**
- **Found during:** Task 1 verification
- **Issue:** The plan's test snippet asserted `ids.len() == 13`, but the SEED-001 CWE ID list in the plan's own `must_haves` section enumerates 14 distinct IDs (120, 78, 242, 327, 377, 190, 134, 22, 807, 362, 367, 20, 126, 676). The test failed with `left: 14, right: 13`.
- **Fix:** Updated the assertion to `assert_eq!(ids.len(), 14, ...)`. The CWE_RULES table itself was unchanged — it correctly implements all 14 IDs from SEED-001.
- **Files modified:** src/vulnerability/cwe_scanner.rs (test assertion only)
- **Commit:** 3e5085d

## Known Stubs

None. The scanner implementation is complete and functional. Plan 03 will wire run_lexical_scanner into the CycloneDX output path.

## Threat Flags

No new network endpoints, auth paths, or trust-boundary surfaces introduced beyond what is documented in the plan's threat_model section (filesystem -> scanner boundary, already analyzed as T-11-02-01 through T-11-02-05).

## Self-Check: PASSED

- src/vulnerability/cwe_scanner.rs: FOUND
- src/vulnerability/mod.rs: FOUND
- Commit 3e5085d (Task 1): FOUND
- Commit f23551f (Task 2): FOUND
