---
phase: 14-cppcheck-integration
verified: 2026-05-10T00:00:00Z
status: passed
score: 12/12
overrides_applied: 0
gaps: []
human_verification:
  - test: "Run sc2sbom on a C/C++ project with cppcheck installed on PATH"
    expected: "cppcheck findings appear as SastFinding entries in CycloneDX output and _static_analysis.md"
    why_human: "cppcheck is not installed in dev/CI environment; cannot exercise the real subprocess invocation path in an automated check"
---

# Phase 14: cppcheck-integration Verification Report

**Phase Goal:** Integrate cppcheck as a second SAST scanner alongside the existing lexical scanner. Add --cppcheck-path CLI flag, implement cppcheck XML v2 parser, implement the scanner subprocess driver, wire it into the pipeline, and deduplicate combined findings by (file, line, cwe_id) with SastSource::Both for dual-detected findings.
**Verified:** 2026-05-10T00:00:00Z
**Status:** passed (one human verification item — cannot run real cppcheck binary in dev environment)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SastFinding carries a SastSource discriminator (Lexical/Cppcheck/Both) | VERIFIED | `pub enum SastSource { Lexical, Cppcheck, Both, }` at cwe_scanner.rs:29-33; `pub source: SastSource` field on SastFinding at cwe_scanner.rs:46 |
| 2 | All existing lexical scanner construction sites set source=Lexical | VERIFIED | cwe_scanner.rs:378 and 391 both set `source: SastSource::Lexical`; tests/cyclonedx_sast_tests.rs:39 and tests/format_tests/sast_report_tests.rs:24 both use `SastSource::Lexical` |
| 3 | parse_cppcheck_xml extracts (cwe, file, line) from cppcheck XML v2 stderr output | VERIFIED | `pub fn parse_cppcheck_xml` at cwe_scanner.rs:481; reads `cwe` attribute from `<error>`, `file` and `line` from `<location>`; 6 parser unit tests pass |
| 4 | Findings without a cwe attribute fall back to CPPCHECK_CWE_OVERRIDES | VERIFIED | cwe_scanner.rs:517-524 performs override lookup when `current_cwe == 0`; `CPPCHECK_CWE_OVERRIDES` table at cwe_scanner.rs:455 has 15 entries; test `override_table_resolves_uninitvar_to_cwe_457` passes |
| 5 | Findings with no resolvable CWE are silently dropped | VERIFIED | Location event only fires when `current_cwe != 0` (cwe_scanner.rs:529); test `unresolved_cwe_is_silently_dropped` passes |
| 6 | All cppcheck-derived findings have source = SastSource::Cppcheck | VERIFIED | cwe_scanner.rs:553 sets `source: SastSource::Cppcheck` in parse_cppcheck_xml construction site |
| 7 | Args struct exposes a --cppcheck-path CLI flag of type Option<PathBuf> gated behind #[cfg(feature = "internal")] | VERIFIED | cli.rs:274-276: `#[cfg(feature = "internal")]` + `#[arg(long)]` + `pub cppcheck_path: Option<PathBuf>`; both `cargo build` and `cargo build --features internal` exit 0 |
| 8 | run_cppcheck_scanner invokes cppcheck with --xml --xml-version=2 --enable=warning,style,security, captures stderr | VERIFIED | cwe_scanner.rs:635-643: `Command::new(bin).args(["--xml","--xml-version=2","--enable=warning,style,security", dir_str]).stdout(Stdio::null()).stderr(Stdio::piped())` |
| 9 | Missing cppcheck binary causes warning to stderr and returns Vec::new() — no abort | VERIFIED | cwe_scanner.rs:592-601 preflight check; test `missing_cppcheck_binary_returns_empty_vec_no_panic` passes (11 cppcheck tests total pass) |
| 10 | run_cppcheck_scanner is wired into main.rs after run_lexical_scanner in the cfg(internal) block | VERIFIED | main.rs:239-253: `let lexical_findings = run_lexical_scanner(...)` → `run_cppcheck_scanner(...)` → `deduplicate_sast_findings(...)` all inside the existing internal feature block |
| 11 | args.cppcheck_path is forwarded to run_cppcheck_scanner as Option<&OsStr> | VERIFIED | main.rs:243-246: `args.cppcheck_path.as_deref().map(|p| p.as_os_str())` passed to `run_cppcheck_scanner` |
| 12 | Findings are deduplicated by (canonical_file_path, line, cwe_id); dual-detected findings get SastSource::Both | VERIFIED | `pub fn deduplicate_sast_findings` at cwe_scanner.rs:706; line 730: `deduped[idx].source = SastSource::Both`; re-exported in mod.rs:12; 3 dedup behavioral tests pass (unique-preserves-all, collision-promotes-to-Both, distinct-CWE-at-same-file-line-kept-separate) |

**Score:** 12/12 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/vulnerability/cwe_scanner.rs` | SastSource enum + source field on SastFinding | VERIFIED | SastSource enum at line 29, source field at line 46, all three variants present |
| `src/vulnerability/cwe_scanner.rs` | parse_cppcheck_xml() and CPPCHECK_CWE_OVERRIDES static table | VERIFIED | parse_cppcheck_xml at line 481, CPPCHECK_CWE_OVERRIDES at line 455 with 15 entries (>= 10 required) |
| `src/vulnerability/cwe_scanner.rs` | run_cppcheck_scanner function | VERIFIED | Function at line 584, exports subprocess invocation with preflight, spinner, per-component loop, stderr capture |
| `src/vulnerability/cwe_scanner.rs` | deduplicate_sast_findings helper | VERIFIED | Function at line 706, uses HashMap keyed by (canonical_path, line, cwe_id), sets SastSource::Both on collision |
| `src/vulnerability/mod.rs` | Re-exports of all new symbols | VERIFIED | Line 12 exports: deduplicate_sast_findings, has_c_cpp_files, parse_cppcheck_xml, run_cppcheck_scanner, run_lexical_scanner, SastFinding, SastSource |
| `src/cli.rs` | cppcheck_path field on Args | VERIFIED | Line 274-276 — cfg-gated, arg(long), Option<PathBuf> type |
| `src/main.rs` | run_cppcheck_scanner call site + deduplication block | VERIFIED | Lines 239-253: lexical → cppcheck → dedup pipeline, args.cppcheck_path forwarded |
| `tests/vulnerability_tests/cppcheck_scanner_tests.rs` | Parser and dedup unit tests | VERIFIED | 11 tests total: 6 parser, 2 graceful-degradation, 3 dedup |
| `tests/vulnerability_tests/mod.rs` | Module declaration for cppcheck_scanner_tests | VERIFIED | Lines 5-6: `#[cfg(feature = "internal")] mod cppcheck_scanner_tests;` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| SastFinding struct | SastSource enum | source field | WIRED | cwe_scanner.rs:46 `pub source: SastSource` |
| scan_file construction site | SastSource::Lexical | struct literal field | WIRED | cwe_scanner.rs:378,391 `source: SastSource::Lexical` |
| parse_cppcheck_xml | SastSource::Cppcheck | struct literal field | WIRED | cwe_scanner.rs:553 `source: SastSource::Cppcheck` |
| parse_cppcheck_xml | CPPCHECK_CWE_OVERRIDES | lookup when cwe attr absent | WIRED | cwe_scanner.rs:517-524 — iterates table when `current_cwe == 0` |
| run_cppcheck_scanner | parse_cppcheck_xml | call with out.stderr bytes | WIRED | cwe_scanner.rs:666 `parse_cppcheck_xml(&out.stderr, name, ecosystem)` |
| run_cppcheck_scanner | Command::new | subprocess invocation with --xml --xml-version=2 | WIRED | cwe_scanner.rs:635-644 |
| src/main.rs run_lexical_scanner | run_cppcheck_scanner | sequential call in cfg(internal) block | WIRED | main.rs:248 `crate::vulnerability::run_cppcheck_scanner(...)` |
| deduplication block | SastSource::Both | mutation of existing entry on key collision | WIRED | cwe_scanner.rs:730 `deduped[idx].source = SastSource::Both` |
| cli.rs Args.cppcheck_path | run_cppcheck_scanner | forwarded as Option<&OsStr> | WIRED | main.rs:243-248 `args.cppcheck_path.as_deref().map(|p| p.as_os_str())` |

### Data-Flow Trace (Level 4)

Not applicable — this phase produces library code (scanner functions + CLI wiring), not a rendering component. Output flows to formatters (cyclonedx.rs, console.rs) that were verified in prior phases.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| cargo build --features internal exits 0 | `cargo build --features internal` | Finished dev profile (warnings only, no errors) | PASS |
| cargo build (no internal) exits 0 | `cargo build` | Finished dev profile (warnings only, no errors) | PASS |
| 11 cppcheck tests pass | `cargo test --features internal cppcheck` | test result: ok. 11 passed; 0 failed | PASS |
| Full test suite passes | `cargo test --features internal` | test result: ok. 339 passed; 0 failed; 2 ignored | PASS |
| SastSource enum has all three variants | grep in cwe_scanner.rs | Lexical, Cppcheck, Both all present at line 29-33 | PASS |
| CPPCHECK_CWE_OVERRIDES has >= 10 entries | grep count | 15 entries in the table | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| CPPCHECK-01 | Plans 04, 05 | Scanner invokes cppcheck --xml on component dirs when on PATH | SATISFIED | run_cppcheck_scanner uses Command::new with --xml --xml-version=2 --enable=warning,style,security; wired in main.rs |
| CPPCHECK-02 | Plans 01, 02 | cppcheck XML output parsed; findings emitted as SastFinding through CycloneDX pipeline | SATISFIED | parse_cppcheck_xml + CPPCHECK_CWE_OVERRIDES; SastFinding.source field added; findings reach sast_findings consumed by formatters |
| CPPCHECK-03 | Plan 04 | If cppcheck not found, logs warning and continues with lexical-only results — no abort | SATISFIED | Preflight check at cwe_scanner.rs:592-601; test `missing_cppcheck_binary_returns_empty_vec_no_panic` passes |
| CPPCHECK-04 | Plan 03 | --cppcheck-path CLI flag allows explicit binary location | SATISFIED | cli.rs:274-276 `pub cppcheck_path: Option<PathBuf>` behind cfg(feature="internal"); forwarded to run_cppcheck_scanner in main.rs |
| CPPCHECK-05 | Plan 05 | Findings deduplicated by (file, line, cwe) — no duplicates in output | SATISFIED | deduplicate_sast_findings uses HashMap keyed by canonical (path, line, cwe_id); SastSource::Both on collision; 3 dedup tests pass |

All 5 phase requirements (CPPCHECK-01 through CPPCHECK-05) satisfied. No orphaned requirements found in REQUIREMENTS.md for Phase 14.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | — | — | — | No TODOs, placeholders, or stub implementations found in the phase's new code |

Note: The `Err` branch of the cppcheck output match (cwe_scanner.rs:647-651) originally had a `continue` that skipped successful exits. The actual implementation (lines 652-683) handles non-zero exit codes (cppcheck returns exit 1 when findings exist — treated as normal) and still calls parse_cppcheck_xml. This is correct behavior and not a stub.

### Human Verification Required

#### 1. Real cppcheck binary integration

**Test:** Install cppcheck and run `sc2sbom --features internal` on a C/C++ project that contains a buffer overflow call (e.g., `gets(buf)`)
**Expected:** The _static_analysis.md and CycloneDX output contain CWE findings from cppcheck alongside lexical scanner results; the indicatif spinner appears during scan; stderr shows "cppcheck: N findings from M components" summary line
**Why human:** cppcheck is not installed on the dev machine (confirmed by RESEARCH.md "Environment Availability"); the preflight path exercised by tests uses a guaranteed-nonexistent binary path — real binary behavior with actual XML output cannot be exercised without installing cppcheck

### Gaps Summary

No blocking gaps found. All 12 observable truths are VERIFIED against the actual codebase.

One human verification item exists for end-to-end validation with a real cppcheck binary. This does not block the phase from being considered complete — all automated checks pass (11 cppcheck unit tests, full 339-test suite green, both feature-gated and default builds succeed).

The implementation deviation in run_cppcheck_scanner (cppcheck exit code 1 is treated as normal rather than an error, since cppcheck exits 1 when it finds issues) is correct behavior per cppcheck semantics and represents a quality improvement over the plan's simpler exit-code check.

---

_Verified: 2026-05-10T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
