---
phase: 11-lexical-scanner-cyclonedx-output
plan: "03"
subsystem: cyclonedx-formatter
tags: [cyclonedx, sast, formatter, cwe]
dependency_graph:
  requires: ["11-02"]
  provides: ["build_sast_vulnerabilities", "sast_findings-in-cyclonedx"]
  affects: ["src/formats/cyclonedx.rs", "src/main.rs"]
tech_stack:
  added: []
  patterns:
    - "inline #[cfg(feature = \"internal\")] parameter gating (consistent with existing formatter pattern)"
    - "dep_to_bom_ref rebuilt inside SAST path (acceptable duplication over refactor)"
key_files:
  created: []
  modified:
    - src/formats/cyclonedx.rs
    - src/main.rs
    - tests/format_tests/cyclonedx_tests.rs
    - tests/integration_tests/production_mode_e2e_tests.rs
    - tests/parser_tests/safetensors_tests.rs
decisions:
  - "Used inline #[cfg] parameter gating (not full cfg-split functions) — consistent with existing SbomMode param pattern in same functions"
  - "dep_to_bom_ref rebuilt in convert_to_cyclonedx for the SAST path rather than extracting build_dep_to_bom_ref helper — avoids refactoring CVE path, acceptable per plan"
  - "run_lexical_scanner always runs when --features internal is active (not gated on check_vulnerabilities) — lexical scanner is independent of OSV/NVD"
metrics:
  duration: "~20 minutes"
  completed: "2026-05-09"
  tasks_completed: 3
  files_changed: 5
---

# Phase 11 Plan 03: CycloneDX SAST Integration Summary

One-liner: Extended CycloneDX formatter to emit SAST findings as vulnerabilities[] entries with cwes[], analysis.state="in_triage", and sc2sbom:finding:file/line properties; wired run_lexical_scanner into main.rs after enrich_cwe_ids.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Extend CycloneDX structs (analysis, properties, optional url) | c4ddfd5 | src/formats/cyclonedx.rs |
| 2 | Add build_sast_vulnerabilities and thread sast_findings | c8467f0 | src/formats/cyclonedx.rs, 3 test files |
| 3 | Wire run_lexical_scanner into main.rs | 2250a56 | src/main.rs |

## Output Spec Answers

### dep_to_bom_ref approach

`dep_to_bom_ref` was **rebuilt** inside `convert_to_cyclonedx` for the SAST path rather than extracting a `build_dep_to_bom_ref` helper. The plan noted this duplication is acceptable. The PURL-to-ecosystem normalization code (pypi→pip, golang→go, gem→rubygems, generic+type qualifier handling) is duplicated from `build_cyclonedx_vulnerabilities` into the new block inside `convert_to_cyclonedx`.

### Formatter call sites updated in main.rs

| Line | Function | Context |
|------|----------|---------|
| 317 | `save_cyclonedx_json` | CycloneDX JSON output format with `--output` dir |
| 320 | `print_cyclonedx_json` | CycloneDX JSON output to stdout |
| 378 | `save_cyclonedx_json` | CycloneDX JSON in `--format all` mode |

### SastFinding.file_path — relative or absolute?

**Absolute** — `scan_file` in `cwe_scanner.rs` uses `path.to_string_lossy().into_owned()` where `path` comes from WalkDir, which returns the full path as passed to `WalkDir::new(dir)`. Since `component_dirs` values are `PathBuf`s populated from parser-discovered source trees, the paths are absolute or relative depending on how the scanner was invoked. In practice they will match the format of the `component_dirs` entries, which are populated from manifest file paths. Open Question #2 from RESEARCH.md notes relative-to-scan-root as a recommendation; this remains as-is (absolute) per Claude's Discretion.

## Deviations from Plan

### Auto-applied: Inline #[cfg] parameter pattern instead of cfg-split functions

**Rule 2 (CLAUDE.md Simplicity First + consistency)**

- **Found during:** Task 2 planning
- **Issue:** The plan specifies "cfg-split signatures (option b)" — two complete function copies per entry point. However, the existing `convert_to_cyclonedx`, `print_cyclonedx_json`, and `save_cyclonedx_json` already use inline `#[cfg(feature = "internal")]` attribute gating on the `mode: &SbomMode` parameter. Creating full function duplicates would be inconsistent with this established pattern and more verbose.
- **Fix:** Used the same inline attribute approach for the new `sast_findings: &[SastFinding]` parameter. Non-internal builds see 3-argument signatures; internal builds see 4- or 5-argument signatures.
- **Files modified:** src/formats/cyclonedx.rs, src/main.rs
- **Effect:** Functionally identical. All test call sites updated with `#[cfg(feature = "internal")] &[]` for the new parameter.

### Auto-applied: run_lexical_scanner runs unconditionally (not gated on check_vulnerabilities)

**Rule 2 (correctness)**

- **Found during:** Task 3 implementation
- **Issue:** The `enrich_cwe_ids` call is inside `if args.check_vulnerabilities { ... }`, but the lexical scanner does not depend on OSV/NVD data. Gating it on `check_vulnerabilities` would suppress SAST findings whenever the user doesn't pass `--check-vulnerabilities`.
- **Fix:** Placed `run_lexical_scanner` call inside the outer `#[cfg(feature = "internal")]` block but outside the `if args.check_vulnerabilities` block. It runs whenever the internal feature is active, regardless of the vulnerability flag.
- **Files modified:** src/main.rs

## Known Stubs

None — all plan goals are implemented. The `sast_findings` vec is always populated from the scanner (may be empty if no C/C++ components with mapped directories are found, but that is correct behavior, not a stub).

## Threat Flags

None beyond what the plan's threat model already covers. The T-11-03-02 mitigation (D-10 sanitization of `/` and `.` in bom-ref strings) is implemented in `build_sast_vulnerabilities`.

## Self-Check: PASSED

- src/formats/cyclonedx.rs exists and contains `build_sast_vulnerabilities`, `CycloneDXVulnerabilityAnalysis`, `analysis: Option<...>`, `properties: Vec<CycloneDXProperty>` on CycloneDXVulnerability
- src/main.rs exists and contains `run_lexical_scanner`, `sast_findings`, and updated call sites
- Commits c4ddfd5, c8467f0, 2250a56 all exist in git log
- `cargo build` and `cargo build --features internal` both succeed (0 errors)
- `cargo test` passes (294 pass, 1 pre-existing failure: spdx_validation_tests requiring missing test fixture)
- `cargo test --features internal` passes 308 tests (4 pre-existing failures: spdx_validation + 3 ROS network tests)
