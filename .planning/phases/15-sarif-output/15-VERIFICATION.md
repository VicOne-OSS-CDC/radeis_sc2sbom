---
phase: 15-sarif-output
verified: 2026-05-10T00:00:00Z
status: passed
score: 14/14
overrides_applied: 0
---

# Phase 15: SARIF Output Verification Report

**Phase Goal:** Deliver a SARIF 2.1 output format for SAST findings, accessible via --sarif-output CLI flag, with deduplicated rules and full location data.
**Verified:** 2026-05-10
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | save_sarif_report writes SARIF 2.1 JSON at {out_dir}/{project_name}_static_analysis.sarif when sarif_path is None | VERIFIED | src/formats/sarif.rs:93-96; test_sarif_writes_default_path passes |
| 2  | SARIF file contains $schema, version, runs[0].tool.driver (name, version, rules[]), runs[0].results[] | VERIFIED | sarif.rs:137-150; test_sarif_schema_and_version, test_sarif_driver_metadata pass |
| 3  | rules[] is deduplicated by CWE — 50 findings of CWE-120 produce exactly one rule entry | VERIFIED | BTreeSet<u32> at sarif.rs:106; test_sarif_rules_deduplication passes |
| 4  | Each rule has id='CWE-{N}', name=cwe_name(N), helpUri='https://cwe.mitre.org/data/definitions/{N}.html' | VERIFIED | sarif.rs:109-113; test_sarif_rule_fields passes |
| 5  | results[] has one entry per SastFinding with ruleId, message.text, locations[].physicalLocation | VERIFIED | sarif.rs:117-135; test_sarif_results_no_dedup passes |
| 6  | Empty findings produces valid SARIF with empty results[] and rules[] (D-03) | VERIFIED | sarif.rs:106-114 produce empty vecs; test_sarif_empty_findings passes |
| 7  | D-01: Default SARIF path is {out_dir}/{project_name}_static_analysis.sarif when sarif_path is None | VERIFIED | sarif.rs:95; test_sarif_default_path_when_none passes |
| 8  | D-04: SARIF JSON populates $schema, version, runs[0].tool.driver, runs[0].results[] | VERIFIED | sarif.rs:137-150 |
| 9  | D-05: rules[] contains one entry per detected CWE, deduplicated via BTreeSet<u32> | VERIFIED | sarif.rs:106 `BTreeSet<u32>` |
| 10 | D-06: SARIF output excludes artifactContents, fingerprints, logical locations, function names | VERIFIED | sarif.rs structs contain only schema-required fields |
| 11 | D-07: src/formats/sarif.rs exported via src/formats/mod.rs under internal feature | VERIFIED | mod.rs:3 `pub mod sarif;` + mod.rs:11 `pub use sarif::save_sarif_report;` |
| 12 | D-09: save_sarif_report gated behind #[cfg(feature = "internal")] | VERIFIED | sarif.rs:1 `#![cfg(feature = "internal")]`; cargo build without internal succeeds |
| 13 | D-10: Hand-rolled #[derive(Serialize)] structs, serde_json::to_string_pretty; zero new Cargo.toml deps | VERIFIED | sarif.rs:152 `serde_json::to_string_pretty`; SUMMARY confirms zero new deps |
| 14 | User can pass --sarif-output /path to override default SARIF location | VERIFIED | cli.rs:282 `pub sarif_output: Option<String>`; main.rs:310,396 `args.sarif_output.as_deref()`; --help shows flag; test_sarif_custom_path_override passes |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/formats/sarif.rs` | save_sarif_report + private SARIF structs | VERIFIED | 156 lines; `pub fn save_sarif_report` at line 86; 9 private structs |
| `src/formats/mod.rs` | pub mod sarif + re-export under internal feature | VERIFIED | line 3 `pub mod sarif;`; lines 10-11 `#[cfg(feature = "internal")] pub use sarif::save_sarif_report;` |
| `src/formats/console.rs` | cwe_name visibility changed to pub(crate) | VERIFIED | line 1939 `pub(crate) fn cwe_name` |
| `src/cli.rs` | sarif_output: Option<String> CLI arg under internal feature | VERIFIED | line 282 under `#[cfg(feature = "internal")]` + `#[arg(long)]` |
| `src/main.rs` | Two save_sarif_report invocations after save_static_analysis_report | VERIFIED | line 29 import; line 310 first call site; line 396 second call site |
| `tests/format_tests/sarif_tests.rs` | 10 tests covering SARIF-01, SARIF-02, SARIF-03 | VERIFIED | 10 tests present (#[test] count confirmed), all pass |
| `tests/format_tests/mod.rs` | sarif_tests registered under internal feature | VERIFIED | lines 7-8 `#[cfg(feature = "internal")] pub mod sarif_tests;` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/formats/sarif.rs` | `src/formats/console.rs::cwe_name` | `use super::console::cwe_name` | WIRED | sarif.rs:10 confirmed |
| `src/formats/mod.rs` | `src/formats/sarif.rs` | `pub mod sarif; + pub use sarif::save_sarif_report` | WIRED | mod.rs:3,11 confirmed |
| `src/main.rs` | `src/formats/sarif.rs::save_sarif_report` | `use formats::save_sarif_report` + two call sites | WIRED | main.rs:29,310,396 confirmed; 3 hits total |
| `src/cli.rs Args.sarif_output` | `src/main.rs args.sarif_output.as_deref()` | `Option<String> -> Option<&str>` | WIRED | cli.rs:282; main.rs:310,396 use `.as_deref()` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `src/formats/sarif.rs` | `findings: &[SastFinding]` | Caller-supplied slice (same as save_static_analysis_report) | Yes — same sast_findings fed to both functions at call sites | FLOWING |
| `src/main.rs` call sites | `sast_findings` | Populated earlier in execution by SAST scanner | Yes — real SastFinding entries from lexical + cppcheck scanners | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 10 SARIF tests pass | `cargo test --features internal sarif` | 10 passed; 0 failed | PASS |
| Build with internal feature | `cargo build --features internal` | Finished, 0 errors | PASS |
| Build without internal feature | `cargo build` | Finished, 0 errors | PASS |
| --sarif-output visible in --help | `cargo run --features internal -- --help \| grep sarif` | `--sarif-output <SARIF_OUTPUT>` shown | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SARIF-01 | 15-01-PLAN.md | All SastFinding entries written to _static_analysis.sarif file | SATISFIED | save_sarif_report in sarif.rs; test_sarif_writes_default_path, test_sarif_results_no_dedup pass |
| SARIF-02 | 15-02-PLAN.md | --sarif-output CLI flag allows specifying SARIF output file path | SATISFIED | cli.rs:282 flag; main.rs:310,396 call sites; test_sarif_custom_path_override passes |
| SARIF-03 | 15-01-PLAN.md | SARIF rules[] has id/name/helpUri per detected CWE | SATISFIED | BTreeSet dedup in sarif.rs; test_sarif_rules_deduplication, test_sarif_rule_fields pass |

### Anti-Patterns Found

None identified. No TODO/FIXME/placeholder comments in SARIF module. No stub implementations. No hardcoded empty data passed to rendering. The `eprintln!` at sarif.rs:154 is a legitimate progress indicator, not a stub.

### Human Verification Required

None. All must-haves are programmatically verifiable and confirmed via automated tests and file inspection.

### Gaps Summary

No gaps. All three SARIF requirements (SARIF-01, SARIF-02, SARIF-03) are fully implemented and verified:

- **SARIF-01**: `save_sarif_report` writes a complete SARIF 2.1 JSON file with all SastFindings — proven by 7 unit tests in Plan 01.
- **SARIF-02**: `--sarif-output <PATH>` CLI flag correctly overrides the default output path — proven by 3 integration tests in Plan 02 and confirmed in --help output.
- **SARIF-03**: `rules[]` contains one deduplicated entry per CWE with id/name/helpUri — proven by test_sarif_rules_deduplication and test_sarif_rule_fields.

Both feature gates work correctly: internal build succeeds, non-internal build succeeds with the sarif module fully compiled out.

---

_Verified: 2026-05-10T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
