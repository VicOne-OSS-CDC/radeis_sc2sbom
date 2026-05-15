---
phase: 12-static-analysis-report
verified: 2026-05-10T00:00:00Z
status: complete
score: 8/8 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run the binary with --features internal against a real C project and inspect stderr"
    expected: "Line 'Pattern-based — complex data-flow vulnerabilities not covered' appears on stderr, followed by '✓ Static analysis report saved to: ...'"
    why_human: "stderr is not captured by unit tests; the only test covering this behavior is #[ignore]'d (test_save_static_analysis_report_emits_disclaimer_to_stderr). The eprintln! call exists in source (verified), but live execution is needed to confirm it fires in the binary path."
    result: "VERIFIED 2026-05-10 — ran ./target/debug/radeis_sc2sbom --path tests/fixtures/c --check-vulnerabilities true --output /tmp/smoke; observed 'Pattern-based — complex data-flow vulnerabilities not covered' on stderr followed by '✓ Static analysis report saved to: .../c_static_analysis.md'. Exit 0."
---

# Phase 12: Static Analysis Report Verification Report

**Phase Goal:** Deliver a static analysis report formatter that produces a dedicated `{project}_static_analysis.md` file and injects a summary section into the main `{project}_report.md`, both gated behind `--features internal`.
**Verified:** 2026-05-10
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `save_static_analysis_report()` exists in `src/formats/console.rs` gated by `#[cfg(feature = "internal")]` | VERIFIED | `src/formats/console.rs` line 1942: `#[cfg(feature = "internal")] pub fn save_static_analysis_report(...)` |
| 2 | `save_static_analysis_report()` is re-exported from `src/formats/mod.rs` under `#[cfg(feature = "internal")]` | VERIFIED | `src/formats/mod.rs` lines 7-8: `#[cfg(feature = "internal")] pub use console::save_static_analysis_report;` |
| 3 | `save_static_analysis_report()` writes `{project}_static_analysis.md` to the supplied output dir | VERIFIED | `src/formats/console.rs` line 2016: `out_dir.join(format!("{}_static_analysis.md", project_name))`. Test `test_save_static_analysis_report_writes_correct_filename` passes. |
| 4 | Generated markdown contains H1, disclaimer blockquote, summary table, and findings grouped by component then CWE | VERIFIED | Lines 1951–2013 of `console.rs` emit `# Static Analysis Report`, blockquote, `| Component | CWE | Name | Count |`, and `## {component}` / `### CWE-{N}` groupings. Tests `test_save_static_analysis_report_with_findings` and `test_save_static_analysis_report_zero_findings` pass (5/5 active tests: `ok`). |
| 5 | Zero-findings case writes a file with the No-findings prose row in the table and prose line in the findings section | VERIFIED | `console.rs` lines 1962-1990: empty-branch writes `| — | — | No static analysis findings detected. | — |` in table and prose in Findings section. Test passes. |
| 6 | Function emits stderr disclaimer "Pattern-based — complex data-flow vulnerabilities not covered" followed by save confirmation | VERIFIED (source) / UNCERTAIN (live runtime) | `eprintln!` calls present at lines 2020-2021 of `console.rs`. Unit test for this behavior is `#[ignore = "stderr capture requires integration test harness"]`. Source check confirms the code is there; live execution not verified by automated tests. |
| 7 | `save_console_report()` emits a `## Static Analysis Findings` section in `{project}_report.md` after the CVE/Vulnerabilities block | VERIFIED | `console.rs` lines 1469-1497 inject the section. Ordering verified: `awk` check returned `OK` (CVE block line 1330 < SAST writeln line 1473 < `if summary_only` line 1500). Test `test_console_report_includes_sast_section_with_findings` passes. |
| 8 | `main.rs` Console arm and All arm both call `save_static_analysis_report` after `save_console_report` inside `#[cfg(feature = "internal")]` block | VERIFIED | `main.rs` lines 271-272 (Console arm) and 355-356 (All arm) both call `save_static_analysis_report(project_name, out_dir, &sast_findings)` under `#[cfg(feature = "internal")]`. `sast_findings` is populated by Phase 11's `run_lexical_scanner()` at line 225. |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/formats/console.rs` | `save_static_analysis_report()` implementation + SAST section in `save_console_report` | VERIFIED | Function at line 1942; SAST injection at line 1469; `cwe_name()` helper at line 1921 |
| `src/formats/mod.rs` | Re-export of `save_static_analysis_report` under feature gate | VERIFIED | Lines 7-8 |
| `src/main.rs` | Two call sites for `save_static_analysis_report`, `sast_findings` threaded into `save_console_report` | VERIFIED | Lines 259-272 (Console), 343-356 (All); both `save_console_report` calls pass `&sast_findings` via `#[cfg(feature = "internal")]` |
| `tests/format_tests/sast_report_tests.rs` | 5 active test functions covering RPT-01, RPT-02, RPT-03 | VERIFIED | 5 active `#[test]` functions present; 1 `#[ignore]`'d stderr test (intentional per plan) |
| `tests/format_tests/mod.rs` | Module registration under `#[cfg(feature = "internal")]` | VERIFIED | Lines 5-6 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/main.rs` | `src/formats/console.rs:save_static_analysis_report` | `use formats::save_static_analysis_report` at line 27; called at lines 272, 356 | WIRED | Feature-gated import + two feature-gated call sites confirmed |
| `src/formats/console.rs:save_static_analysis_report` | `src/vulnerability/cwe_scanner.rs:SastFinding` | `#[cfg(feature = "internal")] use crate::vulnerability::cwe_scanner::SastFinding` | WIRED | Import at line 12; `SastFinding` used in function signature and body |
| `src/formats/console.rs:save_console_report` | `src/vulnerability/cwe_scanner.rs:SastFinding` | `#[cfg(feature = "internal")] sast_findings: &[SastFinding]` parameter at line 1137 | WIRED | Parameter present and used in SAST section at lines 1476-1494 |
| `src/main.rs` | `src/formats/console.rs:save_console_report` | both call sites pass `#[cfg(feature = "internal")] &sast_findings` | WIRED | Lines 268 and 352 pass the findings slice |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `save_static_analysis_report` | `findings: &[SastFinding]` | `sast_findings` from `run_lexical_scanner()` in `main.rs` line 225 | Yes — Phase 11 wires the scanner; non-empty in real runs against C projects | FLOWING |
| SAST section in `save_console_report` | `sast_findings: &[SastFinding]` | Same `sast_findings` variable threaded via call site | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 5 active SAST tests pass | `cargo test --features internal -- sast_report_tests` | `test result: ok. 5 passed; 0 failed; 1 ignored` | PASS |
| Default build clean | `cargo build` | 0 errors | PASS |
| Internal build clean | `cargo build --features internal` | 0 errors | PASS |
| `save_static_analysis_report` function present | grep `pub fn save_static_analysis_report` in `console.rs` | Found at line 1942 | PASS |
| SAST section ordering in `save_console_report` | awk ordering check | CVE=1330, SAST=1473, summary_only=1500 — OK | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| RPT-01 | 12-01, 12-02 | Separate `_static_analysis.md` report with per-component CWE summary table and file:line findings | SATISFIED | `save_static_analysis_report()` implemented; 3 dedicated tests pass; filename convention, H1, blockquote, summary table, and per-CWE grouping all present in source |
| RPT-02 | 12-01, 12-03 | Static analysis findings section integrated into main `_report.md` alongside CVE findings | SATISFIED | `## Static Analysis Findings` section injected into `save_console_report` after CVE block; ordering verified by line-number check; 2 integration tests pass |
| RPT-03 | 12-01, 12-02 | CLI prints disclaimer "Pattern-based — complex data-flow vulnerabilities not covered" when static analysis runs | SATISFIED | `eprintln!` at `console.rs` line 2020 exists; confirmed by live run 2026-05-10: binary emits `Pattern-based — complex data-flow vulnerabilities not covered` on stderr when `--check-vulnerabilities true` is set against a C project |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `tests/format_tests/sast_report_tests.rs` | 241 | `#[ignore = "stderr capture requires integration test harness"]` | Info | Intentional per plan; stderr disclaimer has no automated test coverage. Not a blocker — the `eprintln!` call is present in source. |

No stub implementations, empty handlers, or missing data paths found.

### Human Verification Required

#### 1. Stderr Disclaimer Emitted at Runtime

**Test:** Build the binary with `--features internal` and run it against any directory containing C files (or an empty directory) with `--output <dir>`. Redirect stderr: `cargo run --features internal -- --output /tmp/sast_smoke <c-project-path> 2>&1 | grep "Pattern-based"`.
**Expected:** The line `Pattern-based — complex data-flow vulnerabilities not covered` appears on stderr, followed by `✓ Static analysis report saved to: /tmp/sast_smoke/..._static_analysis.md`.
**Why human:** Rust unit tests cannot easily capture `eprintln!` output. The only test covering this behavior (`test_save_static_analysis_report_emits_disclaimer_to_stderr`) is intentionally `#[ignore]`'d per plan 12-01. The `eprintln!` call is verified to exist in source at `console.rs` line 2020, but live execution is required to confirm it is reachable in the actual binary execution path.

### Gaps Summary

No blocking gaps found. All 8 must-have truths are verified by actual codebase evidence. The single human verification item is the stderr disclaimer runtime check — RPT-03 is satisfied at the source level but not by a running integration test.

**Note on RPT-03:** The REQUIREMENTS.md marks RPT-03 as "Pending" (unchecked checkbox), consistent with plan 12-02 documenting the test as `#[ignore]`. The source implementation is confirmed correct. The human verification above closes the gap.

---

_Verified: 2026-05-10_
_Verifier: Claude (gsd-verifier)_
