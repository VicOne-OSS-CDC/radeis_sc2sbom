---
phase: 11-lexical-scanner-cyclonedx-output
plan: "01"
subsystem: scanner/models
tags: [scanner, scan-context, c-cpp, component-dirs]
dependency_graph:
  requires: []
  provides: [ScanContext.component_dirs]
  affects: [src/models/dependency.rs, src/scanner/mod.rs]
tech_stack:
  added: []
  patterns: [HashMap accumulator in scan_directory, or_insert_with first-discovery semantics]
key_files:
  created: []
  modified:
    - src/models/dependency.rs
    - src/scanner/mod.rs
decisions:
  - "DependencySource has no path-bearing variant; used dep.source_file for vendored arm directory extraction"
  - "component_dirs field is unconditional on ScanContext per D-01/D-02/D-03 Pitfall 5"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-09"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
---

# Phase 11 Plan 01: ScanContext component_dirs Foundation Summary

ScanContext extended with unconditional `HashMap<(String, String), PathBuf>` field populated at all six C/C++ parser call sites in scan_directory; provides scope source for SCAN-05 lexical scanner in Plan 02.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add component_dirs field to ScanContext | 694ceb3 | src/models/dependency.rs |
| 2 | Populate component_dirs at all six C/C++ parser call sites | d0901c5 | src/scanner/mod.rs |

## Parser Arms Touched (with line numbers)

| Arm | Parser | Line (scanner/mod.rs) |
|-----|--------|----------------------|
| (a) | CMakeLists.txt — parse_cmake_file | ~659 |
| (b) | *.cmake — parse_cmake_file | ~674 |
| (c) | *.pc — parse_pc_file | ~688 |
| (d) | configure.ac/configure.in — parse_configure_ac | ~733 |
| (e) | Makefile.am — parse_makefile_am | ~751 |
| (f) | scan_vendored_3rdparty | ~1054 |

Declaration at line 426; ScanContext construction at line 1096.

## DependencySource Variant for Vendored Arm

`DependencySource` in `src/models/dependency.rs` has only unit variants (`Manifest`, `LockFile`, `ImportScan`) — none carry a path. The plan's suggested `DependencySource::Manifest { path: src_path, .. }` pattern was not applicable.

**Resolution used:** `dep.source_file` (an `Option<String>` on `Dependency`) was used as the directory source via `.and_then(|sf| std::path::Path::new(sf).parent())`. When `source_file` is absent, falls back to the vendored scan root `path`. This is functionally equivalent to the plan's intent: record the closest available manifest directory.

## Verification Results

- `cargo check` (default): 0 errors
- `cargo check --features internal`: 0 errors
- `cargo test --lib scanner`: 6 passed, 0 failed
- `grep -c "component_dirs" src/scanner/mod.rs`: 9 (>= 8 required)
- No `cfg(feature)` gates on population logic

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] DependencySource::Manifest has no path field**
- **Found during:** Task 2
- **Issue:** Plan's vendored arm code used `DependencySource::Manifest { path: src_path, .. }` struct destructuring, but the actual enum variant is a unit variant with no associated data.
- **Fix:** Used `dep.source_file` string field on `Dependency` instead. Extracts parent directory via `std::path::Path::new(sf).parent()`, falls back to scan root `path` when absent.
- **Files modified:** src/scanner/mod.rs
- **Commit:** d0901c5

## Threat Flags

No new network endpoints, auth paths, file access patterns, or schema changes introduced. `component_dirs` keys are inert `(String, String)` tuples; no shell, SQL, or path interpolation at this layer. T-11-01-02 (symlink disclosure) remains mitigated by existing `should_process_entry` walker filtering unchanged by this plan.

## Self-Check: PASSED

- src/models/dependency.rs: FOUND with component_dirs field
- src/scanner/mod.rs: FOUND with 9 component_dirs references
- Commit 694ceb3: FOUND
- Commit d0901c5: FOUND
