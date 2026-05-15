---
phase: 11-lexical-scanner-cyclonedx-output
verified: 2026-05-09T12:00:00Z
status: passed
score: 9/9 must-haves verified
overrides_applied: 0
---

# Phase 11: Lexical Scanner + CycloneDX Output Verification Report

**Phase Goal:** Users running the tool with `--features internal` against a C/C++ project receive CWE findings in CycloneDX 1.5 output — each dangerous-function call site is detected by file, line, and CWE ID, linked to its owning component via bom-ref, and marked `in_triage`
**Verified:** 2026-05-09T12:00:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Running the tool against a C/C++ source tree with `--features internal` produces at least one CWE finding per triggered rule from the confirmed CWE list (120, 78, 242, 327, 377, 190, 134, 22, 807, 362, 367, 20, 126, 676) | ✓ VERIFIED | `test_all_thirteen_cwes` passes — all 14 named CWEs fire on `dangerous_calls.c` fixture. REQUIREMENTS.md lists 14 CWEs by name despite saying "13" in prose (typo in requirements text; all named CWEs are covered). |
| 2 | CWE-134 findings fire only when the format argument is not a string literal — calls with hardcoded format strings produce no finding | ✓ VERIFIED | `test_cwe134_skips_literal_format` and `test_cwe134_fires_on_variable_format` both pass. CWE_RULES splits CWE-134 into two entries (arg-0 for printf/sprintf variants, arg-1 for fprintf/syslog) with `format_arg_is_literal` heuristic. |
| 3 | Findings reference only files inside component-mapped C/C++ directories | ✓ VERIFIED | `test_scope_restriction` passes — file in unmapped sibling dir produces zero findings; scanner walks only `component_dirs` values. `run_lexical_scanner` skips dirs not in the HashMap. |
| 4 | Each finding records the file path and line number of the dangerous call site | ✓ VERIFIED | `SastFinding` struct has `file_path: String` and `line: u32`. `test_finding_location` asserts line=3 for call on line 3. `test_cwe120_strcpy` asserts line=1. |
| 5 | CycloneDX output contains a `vulnerabilities[]` entry per finding with `cwes[]`, `source.name = "radeis_sc2sbom static analysis"`, `analysis.state: "in_triage"`, `affects[].ref` to owning component bom-ref, and `properties` for file+line; SPDX 2.3 output is byte-for-byte unchanged | ✓ VERIFIED | `test_sast_vulnerability_in_output` passes all assertions. `test_spdx_has_no_vulnerabilities_field` and `test_spdx_top_level_keys_unchanged` both pass on default and internal builds. |
| 6 | ScanContext carries a (name, ecosystem) -> PathBuf mapping for every C/C++ dependency discovered | ✓ VERIFIED | `src/models/dependency.rs`: `pub component_dirs: HashMap<(String, String), PathBuf>` present unconditionally in `ScanContext`. |
| 7 | scan_directory populates component_dirs at every C/C++ parser call site | ✓ VERIFIED | `grep -c "component_dirs" src/scanner/mod.rs` returns 9 (>= 8 required per plan). Six parser arms (CMakeLists.txt, *.cmake, *.pc, configure.ac, Makefile.am, vendored) all populate the map. |
| 8 | All scanner code is gated behind `#[cfg(feature = "internal")]`; default builds compile without it | ✓ VERIFIED | `cwe_scanner.rs` has module-level `#![cfg(feature = "internal")]`. `vulnerability/mod.rs` gates `pub mod cwe_scanner` and re-exports. `cargo build` (default) succeeds with 0 errors. |
| 9 | Test coverage exists for all requirements (SCAN-01..05, CDX-01..04) | ✓ VERIFIED | 9 cwe_scanner unit tests, 3 cyclonedx_sast integration tests, 2 spdx_unchanged regression tests — all pass. |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/models/dependency.rs` | `ScanContext.component_dirs: HashMap<(String, String), PathBuf>` | ✓ VERIFIED | Field present at line ~467, unconditional, with full doc comment |
| `src/scanner/mod.rs` | component_dirs population at 6 C/C++ parser call sites | ✓ VERIFIED | 9 occurrences of `component_dirs` found (declaration + 6 arms + ScanContext construction + extraction in main.rs) |
| `src/vulnerability/cwe_scanner.rs` | SastFinding struct, CWE_RULES (14 entries, 14 distinct CWEs), run_lexical_scanner, format_arg_is_literal, find_function_call | ✓ VERIFIED | All present; module gated with `#![cfg(feature = "internal")]`; 263 lines |
| `src/vulnerability/mod.rs` | `pub mod cwe_scanner` + re-exports, both gated | ✓ VERIFIED | Lines 5-12 show correct gating pattern |
| `src/formats/cyclonedx.rs` | `build_sast_vulnerabilities`, `CycloneDXVulnerabilityAnalysis`, `analysis: Option<...>`, `properties: Vec<CycloneDXProperty>`, `url: Option<String>` on source | ✓ VERIFIED | All present; `build_sast_vulnerabilities` at line ~424, struct extensions at lines ~226-270 |
| `src/main.rs` | `run_lexical_scanner` invocation + `sast_findings` threaded to formatter calls | ✓ VERIFIED | Line 73: `component_dirs` extracted under `#[cfg]`; line 225: `run_lexical_scanner` call; lines 317, 320, 378: `save_cyclonedx_json`/`print_cyclonedx_json` with `&sast_findings` |
| `tests/vulnerability_tests/cwe_scanner_tests.rs` | 9 unit tests covering SCAN-01..05 | ✓ VERIFIED | 9 tests present; all pass under `--features internal` |
| `tests/cyclonedx_sast_tests.rs` | 3 integration tests covering CDX-01..03 | ✓ VERIFIED | 3 tests present; all pass under `--features internal` |
| `tests/spdx_unchanged_test.rs` | 2 regression tests (CDX-04) | ✓ VERIFIED | 2 tests present; pass on both default and internal builds |
| `tests/fixtures/c/dangerous_calls.c` | C fixture with one call per CWE rule | ✓ VERIFIED | Contains all 14 required function calls |
| `tests/fixtures/c/safe_printf.c` | Negative-case fixture for CWE-134 | ✓ VERIFIED | `printf("hello world\n")` and `fprintf(stderr, "x = %d\n", x)` — both literal format args |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/main.rs` | `run_lexical_scanner` | `crate::vulnerability::run_lexical_scanner(&component_dirs)` | ✓ WIRED | Line 225 inside `#[cfg(feature = "internal")]` block |
| `src/main.rs` | `save_cyclonedx_json` | `&sast_findings` trailing param | ✓ WIRED | Lines 317, 378 use inline `#[cfg]` gating for the param |
| `src/formats/cyclonedx.rs::build_sast_vulnerabilities` | `dep_to_bom_ref` HashMap | `dep_to_bom_ref.get(&key)` | ✓ WIRED | Rebuilt inside `convert_to_cyclonedx` for the SAST path; `v.extend(build_sast_vulnerabilities(sast_findings, &dep_map))` at line ~587 |
| `src/vulnerability/mod.rs` | `cwe_scanner.rs` | `#[cfg(feature = "internal")] pub mod cwe_scanner;` | ✓ WIRED | Lines 5-6 |
| `tests/vulnerability_tests/mod.rs` | `cwe_scanner_tests` | `#[cfg(feature = "internal")] mod cwe_scanner_tests;` | ✓ WIRED | Lines 3-4 |
| `tests/cyclonedx_sast_tests.rs` | `convert_to_cyclonedx` | `use radeis_sc2sbom::formats::cyclonedx::convert_to_cyclonedx;` | ✓ WIRED | Line 3; function called at lines 41, 102, 117 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `run_lexical_scanner` | `all_findings: Vec<SastFinding>` | `scan_file()` called for each `.c/.h/.cpp/.hpp/.cc` file in `component_dirs` entries | Yes — BufRead streams each file line-by-line, applies CWE_RULES | ✓ FLOWING |
| `build_sast_vulnerabilities` | `out: Vec<CycloneDXVulnerability>` | `sast_findings: &[SastFinding]` parameter | Yes — iterates findings, resolves bom-ref, constructs struct | ✓ FLOWING |
| `convert_to_cyclonedx` | `vulnerabilities` | `build_sast_vulnerabilities(sast_findings, &dep_map)` | Yes — real findings from scanner, real dep_map from components | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 9 cwe_scanner unit tests pass | `cargo test --features internal --test '*' cwe_scanner` | `test result: ok. 9 passed; 0 failed` | ✓ PASS |
| All 3 cyclonedx_sast integration tests pass | `cargo test --features internal --test cyclonedx_sast_tests` | `test result: ok. 3 passed; 0 failed` | ✓ PASS |
| SPDX regression tests pass on default build | `cargo test --test spdx_unchanged_test` | `test result: ok. 2 passed; 0 failed` | ✓ PASS |
| All 5 inline cwe_scanner unit tests pass | `cargo test --features internal --lib vulnerability::cwe_scanner` | `test result: ok. 5 passed; 0 failed` | ✓ PASS |
| Default build succeeds (no internal code leaked) | `cargo build 2>&1 \| grep -E "^error" \| wc -l` | `0` | ✓ PASS |
| Internal build succeeds | `cargo build --features internal 2>&1 \| grep -E "^error" \| wc -l` | `0` | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| SCAN-01 | 11-02, 11-04 | Scanner detects calls in .c/.h/.cpp/.hpp/.cc files | ✓ SATISFIED | `is_c_cpp_source()` filter; `test_extension_filter` verifies .txt files are ignored |
| SCAN-02 | 11-02, 11-04 | Scanner covers all 13 CWEs from SEED-001 list (note: REQUIREMENTS.md enumerates 14 CWEs by name despite saying "13" — all 14 named CWEs are covered) | ✓ SATISFIED | `CWE_RULES` has 14 entries for 14 distinct CWE IDs; `test_all_thirteen_cwes` asserts all 14 named IDs fire |
| SCAN-03 | 11-02, 11-04 | CWE-134 fires only when format arg is not a string literal | ✓ SATISFIED | `format_arg_is_literal` with `arg_index` param; split CWE-134 rules for arg-0/arg-1; both positive and negative tests pass |
| SCAN-04 | 11-02, 11-04 | Scanner records file path and line number | ✓ SATISFIED | `SastFinding.file_path` and `line` fields; `test_finding_location` asserts line=3 |
| SCAN-05 | 11-01, 11-02, 11-04 | Scanner scoped to component-mapped directories only | ✓ SATISFIED | `run_lexical_scanner` iterates only `component_dirs` HashMap; `test_scope_restriction` verifies unmapped file is not scanned |
| CDX-01 | 11-03, 11-04 | Each finding emits `vulnerabilities[]` entry with `cwes[]`, `source.name`, `analysis.state: "in_triage"` | ✓ SATISFIED | `build_sast_vulnerabilities` constructs full struct; `test_sast_vulnerability_in_output` asserts all three fields |
| CDX-02 | 11-03, 11-04 | Each entry includes `affects[].ref` to owning component bom-ref | ✓ SATISFIED | `dep_to_bom_ref` resolved in `convert_to_cyclonedx`; `test_sast_vulnerability_in_output` verifies `affects[0].ref == zlib component bom-ref` |
| CDX-03 | 11-03, 11-04 | File path and line in `properties` as `sc2sbom:finding:file` and `sc2sbom:finding:line` | ✓ SATISFIED | `build_sast_vulnerabilities` adds two `CycloneDXProperty` entries; test asserts both names present |
| CDX-04 | 11-03, 11-04 | SAST findings in CycloneDX only; SPDX 2.3 unchanged | ✓ SATISFIED | SPDX formatters untouched; `test_spdx_has_no_vulnerabilities_field` and `test_spdx_top_level_keys_unchanged` pass on both build variants |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/vulnerability/cwe_scanner.rs` | 260 | Test comment says "13 CWEs" but asserts `ids.len() == 14` | Info | Cosmetic only — test correctly asserts 14, which matches the 14 CWEs enumerated in REQUIREMENTS.md. No functional impact. |

No stubs, placeholder implementations, hardcoded empty data, or wiring gaps found.

### Human Verification Required

None. All must-haves were verifiable programmatically and automated test runs confirm correct behavior.

### SCAN-02 Count Discrepancy Note

REQUIREMENTS.md SCAN-02 prose says "13 CWEs" but the requirement's own enumerated list contains 14 IDs: 120, 78, 242, 327, 377, 190, 134, 22, 807, 362, 367, 20, 126, 676. The implementation covers all 14 enumerated IDs. The ROADMAP.md Phase 11 Success Criteria also says "13" in prose but repeats the same 14-item list. This is a documentation typo — the enumerated list is authoritative and the implementation is correct.

### Gaps Summary

No gaps. All 9 roadmap success criteria are verified, all 9 requirement IDs (SCAN-01..05, CDX-01..04) have passing automated test coverage, and both default and internal builds succeed.

---

_Verified: 2026-05-09T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
