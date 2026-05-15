---
phase: 11-lexical-scanner-cyclonedx-output
plan: "04"
subsystem: tests
tags: [tests, sast, cwe, cyclonedx, spdx, regression, fixtures]
dependency_graph:
  requires: ["11-02", "11-03"]
  provides: ["test coverage for SCAN-01..SCAN-05, CDX-01..CDX-04"]
  affects: []
tech_stack:
  added: []
  patterns: ["tempfile::TempDir for test isolation", "include_bytes! for fixture loading"]
key_files:
  created:
    - tests/fixtures/c/dangerous_calls.c
    - tests/fixtures/c/safe_printf.c
    - tests/vulnerability_tests/cwe_scanner_tests.rs
    - tests/cyclonedx_sast_tests.rs
    - tests/spdx_unchanged_test.rs
  modified:
    - tests/vulnerability_tests/mod.rs
    - src/vulnerability/cwe_scanner.rs
decisions:
  - "format_arg_is_literal extended with arg_index parameter to handle fprintf/syslog (format at position 1, not 0)"
  - "CWE-134 rule split into two entries in CWE_RULES: arg-0 for printf/sprintf/snprintf variants, arg-1 for fprintf/vfprintf/syslog"
  - "SPDX expected_keys list excludes 'files' (not emitted by this project's formatter)"
  - "Sbom constructed explicitly (no Default derive); PathBuf used for project_path field"
metrics:
  duration: "307 seconds"
  completed_date: "2026-05-09"
  tasks_completed: 3
  files_changed: 6
---

# Phase 11 Plan 04: Test Coverage for Lexical Scanner and CycloneDX Output

All nine Phase 11 requirements verified with automated tests. Plans 01-03 implemented; this plan proves it demonstrably works.

## Test Counts

- `tests/vulnerability_tests/cwe_scanner_tests.rs`: 9 unit tests (SCAN-01..SCAN-05)
- `tests/cyclonedx_sast_tests.rs`: 3 integration tests (CDX-01..CDX-03)
- `tests/spdx_unchanged_test.rs`: 2 regression tests (CDX-04)

## SPDX Entry Point

`radeis_sc2sbom::formats::spdx::convert_to_spdx` was the correct function name.

Signature under `internal` feature:
```rust
pub fn convert_to_spdx(sbom: &Sbom, mode: &SbomMode, compact_spdx: bool, supplier_resolver: Option<&SupplierResolver>) -> SPDXDocument
```
Without `internal`:
```rust
pub fn convert_to_spdx(sbom: &Sbom, compact_spdx: bool, supplier_resolver: Option<&SupplierResolver>) -> SPDXDocument
```

The test uses `#[cfg(feature = "internal")]` on the `mode` argument inline (not as a file-level gate) to handle both build variants.

## Sbom Construction

`Sbom::default()` is NOT available — the struct has no `Default` implementation. Fields constructed explicitly:

```rust
Sbom {
    project_path: PathBuf::from("/tmp/proj"),
    generated_at: "2026-05-09T00:00:00Z".to_string(),
    dependencies: vec![dep],
    ros_package: None,
    ros_packages: vec![],
    scope_statistics: None,
}
```

`Dependency::default()` IS available and was used with field overrides.

## Actual SPDX Top-Level Keys

The `SPDXDocument` struct serializes to these nine keys:

```
SPDXID, creationInfo, dataLicense, documentDescribes, documentNamespace,
name, packages, relationships, spdxVersion
```

The plan template included a `files` key that is NOT present in this project's formatter. The test uses the correct set.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed CWE-134 false positive on `fprintf(stderr, "literal")` calls**

- **Found during:** Task 1 — `test_cwe134_skips_literal_format` failed because `fprintf(stderr, "y\n")` triggered CWE-134. The original `format_arg_is_literal` only checked position 0 (first arg after `(`), but `fprintf`/`vfprintf`/`syslog` have their format string at position 1.
- **Fix:** Added `format_arg_index: u8` field to `CweRule`. Extended `format_arg_is_literal(after_func, arg_index)` to skip `arg_index` arguments by scanning for commas at paren depth 0. Split the CWE-134 rule into two entries: arg-0 for `printf`/`sprintf`/`snprintf` variants, arg-1 for `fprintf`/`vfprintf`/`syslog`.
- **Files modified:** `src/vulnerability/cwe_scanner.rs`
- **Commit:** ddbba7d (included in Task 1 commit)

## Known Stubs

None.

## Threat Flags

None. Test files are inert source fixtures and assertions; no new network endpoints or auth paths introduced.

## Self-Check: PASSED
