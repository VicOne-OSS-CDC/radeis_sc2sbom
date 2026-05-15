---
phase: 14-cppcheck-integration
plan: "04"
subsystem: vulnerability/cwe_scanner
tags: [subprocess, cppcheck, sast, graceful-degradation, indicatif, wave-2]
dependency_graph:
  requires: [14-02]
  provides: [run_cppcheck_scanner]
  affects: [plan 14-05]
tech_stack:
  added: [std::process::Command, std::process::Stdio, std::ffi::OsStr, indicatif spinner]
  patterns: [preflight-gate-then-loop, stderr-capture-not-stdout, unwrap_or_else-for-non-Result-fn]
key_files:
  created: []
  modified:
    - src/vulnerability/cwe_scanner.rs
    - src/vulnerability/mod.rs
    - tests/vulnerability_tests/cppcheck_scanner_tests.rs
decisions:
  - "unwrap_or_else(|_| ProgressStyle::default_spinner()) used instead of ? because run_cppcheck_scanner returns Vec not Result (Pitfall 6)"
  - "stderr(Stdio::piped()) used — cppcheck writes XML to stderr not stdout (Pitfall 1)"
  - "Command::new(bin).args([\"--version\"]).output() used for preflight — no which crate dependency"
metrics:
  duration: ~10 minutes
  completed: "2026-05-10"
  tasks_completed: 2
  files_modified: 3
---

# Phase 14 Plan 04: run_cppcheck_scanner Implementation Summary

**One-liner:** `run_cppcheck_scanner` invokes cppcheck per-component with PATH preflight, stderr XML capture, indicatif spinner, and graceful degradation for missing binary or per-component failures.

## What Was Done

### Task 1: Implement run_cppcheck_scanner with preflight, per-component loop, and stderr summary

Added to `src/vulnerability/cwe_scanner.rs`:

- New imports: `indicatif::{ProgressBar, ProgressStyle}`, `std::ffi::OsStr`, `std::process::{Command, Stdio}`
- `pub fn run_cppcheck_scanner(component_dirs: &HashMap<(String, String), PathBuf>, cppcheck_bin: Option<&OsStr>) -> Vec<SastFinding>` implementing:
  - Binary resolution: `cppcheck_bin.unwrap_or_else(|| OsStr::new("cppcheck"))` for override or PATH
  - Preflight gate (D-09): `Command::new(bin).args(["--version"]).output()` — emits warning and returns `Vec::new()` on failure
  - Indicatif spinner with per-component progress message (D-07)
  - Per-component loop: `Command::new(bin).args(["--xml", "--xml-version=2", "--enable=warning,style,security", dir])` with `stderr(Stdio::piped())` and `stdout(Stdio::null())`
  - Graceful per-component error handling (D-10): `Err(e)` and non-zero exit both warn and `continue`
  - Calls `parse_cppcheck_xml(&out.stderr, name, ecosystem)` for successful runs
  - `pb.finish_and_clear()` then completion summary line: `"cppcheck: N findings from M components"` (D-08)

Updated `src/vulnerability/mod.rs`: extended `pub use cwe_scanner::{...}` re-export to include `run_cppcheck_scanner` under `#[cfg(feature = "internal")]`.

### Task 2: Add subprocess-level tests for graceful-degradation behavior

Added to `tests/vulnerability_tests/cppcheck_scanner_tests.rs`:

- Extended import line to include `run_cppcheck_scanner`, `HashMap`, `OsStr`, `PathBuf`

| New Test | Behavior verified |
|----------|-------------------|
| `missing_cppcheck_binary_returns_empty_vec_no_panic` | Bogus binary path triggers D-09 preflight failure; returns empty Vec without panic |
| `empty_component_dirs_with_missing_binary_returns_empty` | Zero components + bogus binary still triggers preflight path and returns Vec::new() |

Both tests use a guaranteed-nonexistent binary path so no real cppcheck install is required on the dev machine.

## Verification Results

- `cargo build --features internal`: exits 0 (pre-existing dead-code warnings; expected until Plan 05 wires into main.rs)
- `cargo test --features internal cppcheck`: 8 passed, 0 failed (6 from Plan 02 + 2 new)
- `cargo test --features internal` full suite: 335 passed, 1 pre-existing failure (`test_spdx_output_passes_pyspdxtools_validation` — requires `pyspdxtools` CLI absent in worktree)

## Deviations from Plan

None — plan executed exactly as written. All implementation details (imports, function body, test cases) match the plan specification.

## Known Stubs

None. `run_cppcheck_scanner` is fully implemented. Plan 05 will wire it into `main.rs` — that is a separate integration step, not a stub in this plan.

## Threat Flags

None beyond what the plan's threat model documents:
- T-14-08 (EoP): args statically assembled — accepted per plan
- T-14-09 (DoS): no timeout — accepted per plan
- T-14-10 (info disclosure): summary line omits paths — accepted per plan
- T-14-11 (tampering): `dir.to_str().unwrap_or(".")` for non-UTF8 paths — mitigated per plan

## Self-Check

- `src/vulnerability/cwe_scanner.rs`: `pub fn run_cppcheck_scanner` exists (1 occurrence); `Stdio::piped` present; `parse_cppcheck_xml(&out.stderr` present; `--xml-version=2` present; `unwrap_or_else(|_| ProgressStyle::default_spinner())` present
- `src/vulnerability/mod.rs`: `run_cppcheck_scanner` in re-export line
- `tests/vulnerability_tests/cppcheck_scanner_tests.rs`: 2 new test functions; 3 occurrences of `run_cppcheck_scanner` (1 import + 2 calls)
- Commits: 6543d7a (Task 1), a3e7bef (Task 2)

## Self-Check: PASSED
