---
phase: 14-cppcheck-integration
plan: "03"
subsystem: cli
tags: [cli, cppcheck, internal-feature, pathbuf]
dependency_graph:
  requires: [14-01]
  provides: [cppcheck_path field on Args]
  affects: [src/cli.rs]
tech_stack:
  added: []
  patterns: [cfg(feature = "internal") field gating, Option<PathBuf> clap arg]
key_files:
  created: []
  modified:
    - src/cli.rs
decisions:
  - "Used Option<PathBuf> matching supplier_config pattern rather than Option<String> for type safety"
  - "Placed field after supplier_config at end of Args struct to minimize diff noise"
metrics:
  duration: "~5 minutes"
  completed: "2026-05-10"
  tasks_completed: 1
  tasks_total: 1
---

# Phase 14 Plan 03: --cppcheck-path CLI Flag Summary

**One-liner:** Added `cppcheck_path: Option<PathBuf>` field to `Args` struct in `src/cli.rs`, gated behind `#[cfg(feature = "internal")]`, exposing `--cppcheck-path <PATH>` only in internal builds.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add --cppcheck-path flag to Args | 6bc3939 | src/cli.rs |

## Acceptance Criteria Verified

- `pub cppcheck_path: Option<PathBuf>` field exists at line 276 of src/cli.rs
- `#[cfg(feature = "internal")]` gate is present immediately above the field
- `#[arg(long)]` attribute is present, clap derives `--cppcheck-path` from field name
- `cargo build --features internal` exits 0
- `cargo build` (default features, no internal) exits 0
- `cargo run --features internal -- --help` shows `--cppcheck-path` in output
- `cargo run -- --help` (no internal) does NOT show `--cppcheck-path`

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - this plan adds a CLI field only; Plans 04 and 05 will wire it to the cppcheck invocation logic.

## Self-Check: PASSED

- src/cli.rs modified: FOUND
- Commit 6bc3939: FOUND
- Both cargo builds exit 0: VERIFIED
- --cppcheck-path in internal help, absent from public help: VERIFIED
