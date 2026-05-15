---
phase: "10"
plan: "03"
subsystem: internal-feature-gate
tags: [cfg, feature-gate, cli, formatters, parsers, public-build]
dependency_graph:
  requires: ["10-01", "10-02"]
  provides: ["clean-public-binary", "gated-cli-args", "gated-formatters", "gated-parser-metadata"]
  affects: ["src/cli.rs", "src/main.rs", "src/formats/console.rs", "src/formats/cyclonedx.rs", "src/formats/spdx.rs", "src/parsers/cargo.rs", "src/parsers/npm.rs", "src/parsers/php.rs", "src/parsers/python.rs", "src/parsers/ros.rs", "src/parsers/ruby.rs"]
tech_stack:
  added: []
  patterns: ["cfg-on-parameter", "cfg-on-arg", "parallel-cfg-blocks", "gated-loop"]
key_files:
  created: []
  modified:
    - src/cli.rs
    - src/main.rs
    - src/formats/console.rs
    - src/formats/cyclonedx.rs
    - src/formats/spdx.rs
    - src/parsers/cargo.rs
    - src/parsers/npm.rs
    - src/parsers/php.rs
    - src/parsers/python.rs
    - src/parsers/ros.rs
    - src/parsers/ruby.rs
decisions:
  - "Used cfg-on-parameter syntax for gating function parameters that vary by feature, keeping a single function signature rather than two separate functions"
  - "Used parallel #[cfg(feature = \"internal\")] / #[cfg(not(feature = \"internal\"))] blocks for HashMap variables that need different types per build mode"
  - "Gated entire rayon parallel block in resolve_ros_dependency_versions inside #[cfg(feature = \"internal\")] {} to avoid database variable scope issues"
  - "Auto-fixed 6 parser files (cargo, npm, php, python, ros, ruby) that had reqwest-using fetch functions not gated — these were missed by plans 10-01/02 since they live in public parser modules, not in the gated vulnerability scanner module"
metrics:
  duration: "~90 minutes"
  completed: "2026-05-09"
  tasks_completed: 2
  files_modified: 11
---

# Phase 10 Plan 03: Gate CLI Args, Formatters, and Parser Metadata Fetchers Summary

Gate CLI args, enum definitions, formatter vulnerability branches, and parser reqwest metadata fetchers behind `#[cfg(feature = "internal")]` so `cargo build --release` (public build) compiles cleanly with zero vulnerability-related strings in `--help` output.

## Tasks Completed

### Task 1: Gate CLI args and update main.rs call sites (commit 72f096e)

Gated 3 enums (`MinSeverity`, `VulnerabilityOutputMode`, `SbomMode`) and their impls in `src/cli.rs`. Gated 8 Args fields: `check_vulnerabilities`, `min_severity`, `vulnerability_timeout`, `vulnerability_output`, `cache_ttl`, `clear_cache`, `max_vulns_per_severity`, `sbom_mode`. Total: 12 cfg annotations (requirement: ≥11).

Updated `src/main.rs` to use `#[cfg(feature = "internal")]` on-argument syntax at all formatter call sites that use gated fields (`vulnerability_output`, `sbom_mode`, `check_vulnerabilities`, `max_vulns_per_severity`).

### Task 2: Gate formatters and parser metadata fetchers (commit 2c08396)

**Formatters (src/formats/):**

- `cyclonedx.rs`: Gated import of `SbomMode`, `VulnerabilitySeverity`; gated 5 vuln structs (`CycloneDXVulnerability`, `CycloneDXVulnerabilitySource`, `Rating`, `Reference`, `Affect`); gated `build_cyclonedx_vulnerabilities` function; gated `vulnerabilities` field on `CycloneDXDocument`; updated `convert_to_cyclonedx`, `print_cyclonedx_json`, `save_cyclonedx_json` with cfg-on-parameter for `mode`. Added `#[cfg(not(feature = "internal"))]` fallback for `filtered_deps`. Total: 17 cfg annotations (≥8 required).

- `spdx.rs`: Gated `SbomMode` import; gated vuln reference loop in `convert_to_spdx`; updated `convert_to_spdx`, `print_spdx_json`, `save_spdx_json` with cfg-on-parameter for `mode`. Added non-internal `filtered_deps` fallback. Total: 12 cfg annotations (≥5 required).

- `console.rs`: Gated `VulnerabilityOutputMode` and `Vulnerability`/`VulnerabilitySeverity` imports; gated 5 vuln-only functions; gated `check_vulnerabilities` branches and vuln output blocks in `print_sbom` and `save_console_report`; updated signatures with cfg-on-parameter for `vulnerability_output`, `max_vulns_per_severity`, `check_vulnerabilities`. Total: 34 cfg annotations (≥5 required).

**Parsers (src/parsers/) — Rule 3 auto-fix (blocking):**

Plans 10-01/02 gated the vulnerability scanner module but not the metadata fetcher functions embedded in public parser modules. Six parsers each use `reqwest` for optional API enrichment. Without gating, `cargo build --release` failed with "unresolved import reqwest" errors.

- `cargo.rs`: Gated `fetch_cargo_metadata_from_registry`, `load_cargo_metadata_hybrid`, `fetch_cargo_metadata_batch`; added non-internal HashMap fallback.
- `npm.rs`: Gated `fetch_package_metadata_from_registry`, `fetch_package_metadata_batch`, network branch in `load_package_metadata_hybrid`; added non-internal HashMap fallback.
- `php.rs`: Gated `fetch_php_metadata_from_packagist`, `fetch_php_metadata_batch`; added non-internal HashMap fallback.
- `python.rs`: Gated `fetch_python_metadata_from_pypi`, `fetch_python_metadata_batch`, network branch in `load_python_metadata_hybrid`; added non-internal HashMap fallbacks at two poetry/pipfile call sites.
- `ros.rs`: Gated `fetch_rosdistro_database`, `get_or_fetch_rosdistro_database`; added early return in `resolve_ros_dependency_versions` for non-internal; gated rayon parallel block.
- `ruby.rs`: Gated `fetch_ruby_metadata_from_rubygems`, `fetch_ruby_metadata_batch`; added non-internal HashMap fallback.

## Verification Results

| Check | Result |
|-------|--------|
| `cargo build --release` exits 0 | PASS |
| `cargo build --release --features internal` exits 0 | PASS |
| `--help \| grep -iE 'vulner\|cwe\|cvss\|cache-ttl'` returns empty | PASS |
| `grep -c 'cfg(feature = "internal")' src/cli.rs` ≥ 11 | PASS (12) |
| `grep -c 'cfg(feature = "internal")' src/formats/cyclonedx.rs` ≥ 8 | PASS (17) |
| `grep -c 'cfg(feature = "internal")' src/formats/spdx.rs` ≥ 5 | PASS (12) |
| `grep -c 'cfg(feature = "internal")' src/formats/console.rs` ≥ 5 | PASS (34) |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Gated reqwest-using functions in 6 parser modules**

- **Found during:** Task 2 build verification
- **Issue:** `cargo build --release` failed with `error[E0425]: cannot find value` and `unresolved import reqwest` errors in cargo.rs, npm.rs, php.rs, python.rs, ros.rs, ruby.rs. These parser modules contain optional API metadata fetchers that use `reqwest` directly, outside the gated vulnerability scanner module. Plans 10-01/02 made `reqwest` optional but did not gate the call sites in parser modules.
- **Fix:** Applied `#[cfg(feature = "internal")]` to all network-fetch functions and their call sites in all 6 parsers. Added `#[cfg(not(feature = "internal"))]` fallback HashMap declarations with explicit type annotations to prevent E0282 type inference errors.
- **Files modified:** src/parsers/cargo.rs, npm.rs, php.rs, python.rs, ros.rs, ruby.rs
- **Commits:** 2c08396

**2. [Rule 3 - Blocking] Gated rayon parallel block in ros.rs**

- **Found during:** Task 2 build verification (after parsers were gated)
- **Issue:** `error[E0425]: cannot find value 'database' in this scope` at the `par_iter_mut` block in `resolve_ros_dependency_versions`. The `database` variable is declared under `#[cfg(feature = "internal")]` but the `par_iter_mut` closure (which references `database`) was not gated.
- **Fix:** Wrapped the `use rayon::prelude::*` and `par_iter_mut` block inside `#[cfg(feature = "internal")] { ... }`.
- **Files modified:** src/parsers/ros.rs
- **Commits:** 2c08396

## Known Stubs

None. All code paths are fully implemented — non-internal paths use empty HashMap fallbacks (no metadata enrichment) which is correct behavior for the public build.

## Threat Flags

None. No new network endpoints or auth paths introduced; changes are purely conditional compilation gates that remove functionality in the public build.

## Self-Check: PASSED

- Task 1 commit 72f096e exists: confirmed
- Task 2 commit 2c08396 exists: confirmed
- `src/cli.rs` modified: confirmed
- `src/formats/cyclonedx.rs`, `spdx.rs`, `console.rs` modified: confirmed
- `src/parsers/cargo.rs`, `npm.rs`, `php.rs`, `python.rs`, `ros.rs`, `ruby.rs` modified: confirmed
- Public build binary shows zero vuln strings in --help: confirmed
- Internal build binary shows all 7 gated args in --help: confirmed
