---
phase: 18-ast-scanner-core-and-benchmark
verified: 2026-05-12T10:00:00Z
status: human_needed
score: 5/7 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run benchmark with AUTOSAR fixture and verify BENCHMARK.md is populated with per-CWE rows"
    expected: "docs/BENCHMARK.md contains a FIXTURE-SECTION-START: AUTOSAR_SampleProject_S32K144 section with CWE-ID rows showing finding counts for AST, cppcheck, and lexical scanners"
    why_human: "Fixture is not committed to the repo; developer must stage AUTOSAR_SampleProject_S32K144 and run `cargo test --features internal --test benchmark -- --nocapture`. The benchmark infrastructure and graceful-skip path both compile and run (3/3 tests pass), but no real fixture data has been produced."
  - test: "Verify all 14 ROADMAP CWEs are detectable on AUTOSAR fixture via union of AST + lexical"
    expected: "Running AST scanner and lexical scanner over AUTOSAR_SampleProject_S32K144 produces findings covering all 14 CWEs from ROADMAP SC #2 (CWE-78, 119, 120, 122, 125, 134, 190, 295, 319, 362, 367, 369, 416, 476, 732). CWE-362, CWE-367, CWE-416, CWE-476 come from lexical fallback per D-08."
    why_human: "SC #2 is defined as detection on the AUTOSAR fixture. Without the fixture, verifying the union-coverage claim requires the developer to run the benchmark. The code paths for lexical fallback are wired correctly, but actual fixture output cannot be verified programmatically from this repo."
---

# Phase 18: ast-scanner-core-and-benchmark Verification Report

**Phase Goal:** Implement production AST scanner using tree-sitter-c for C/C++ SAST, covering 13 tractable CWEs with per-argument inspection, lexical fallback on parse failure, integrated as the primary scanner in main.rs, with a benchmark comparing AST vs cppcheck vs lexical on local fixtures.
**Verified:** 2026-05-12T10:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo build --features internal` compiles tree-sitter and tree-sitter-c under the `internal` feature | VERIFIED | Build exits 0. `internal = ["dep:reqwest", "dep:tree-sitter", "dep:tree-sitter-c"]` confirmed in Cargo.toml line 16. No `ast-scanner` feature line remains. |
| 2 | `SastSource::Ast` variant exists in the enum | VERIFIED | `src/vulnerability/cwe_scanner.rs` lines 29-35 show `Ast,` variant with doc comment. |
| 3 | `run_ast_scanner()` is the primary scanner in main.rs, replacing `run_lexical_scanner` | VERIFIED | `src/main.rs` line 246: `let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs);`, line 260: `deduplicate_sast_findings(ast_findings, cppcheck_findings)`. Zero occurrences of `run_lexical_scanner(&component_dirs)` in main.rs. |
| 4 | AST scanner detects 13 tractable CWEs (D-07's 11 + CWE-295 + CWE-319) on synthetic fixtures | VERIFIED | `tests/vulnerability_tests/ast_scanner_tests.rs::test_ast_all_tractable_cwes` passes (6/6 ast_scanner_tests pass). `AST_CWE_RULES` has 17 entries across 13 distinct CWE IDs: 78, 119, 120, 122, 125, 134, 190, 242, 295, 319, 327, 369, 377, 732. |
| 5 | Parse failure triggers lexical fallback with warning, overall run exits 0 | VERIFIED | `scan_file_ast_or_lexical()` at lines 126-143 handles both `None` from `parser.parse()` and `has_error() == true` with `eprintln!` warning + `lexical_scan_file()` call. `test_parse_failure_fallback` passes. |
| 6 | Benchmark test runs without panic when fixtures are absent, exits 0 | VERIFIED | `cargo test --features internal --test benchmark` produces `ok. 3 passed; 0 failed; 0 ignored`. Both AUTOSAR and Juliet tests print SKIP and early-return cleanly. |
| 7 | `docs/BENCHMARK.md` documents AST vs cppcheck vs lexical comparison on real fixtures | UNCERTAIN — needs human | `docs/BENCHMARK.md` exists with the D-14 column headers and HTML-comment markers. However, it contains only a placeholder template — no real fixture data has been run. ROADMAP SC #4 requires "documented comparison on AUTOSAR_SampleProject_S32K144 and at least one additional fixture." The benchmark infrastructure is fully wired and correct, but the developer must run it against real fixtures to fulfill the SC. |

**Score:** 5/7 (but 6 truths are VERIFIED or effectively wired — truth 7 requires human action to generate the output artifact)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | `internal` feature includes tree-sitter deps; `ast-scanner` feature removed; MIT license comment | VERIFIED | Line 16: `internal = ["dep:reqwest", "dep:tree-sitter", "dep:tree-sitter-c"]`. No `ast-scanner` line. License comment at line 39. |
| `src/vulnerability/ast_scanner.rs` | `run_ast_scanner()`, `AstCweRule`, `ArgCheck`, `AST_CWE_RULES`, `scan_file_ast_or_lexical()`, 493 lines | VERIFIED | All structures present and substantive. Production implementation with scope-aware FixedSizeBuffer, lexical fallback, field-based AST access (Pattern 3). |
| `src/vulnerability/cwe_scanner.rs` | `pub(crate) fn scan_file` and `pub(crate) fn token_present_with_boundary` exposed | VERIFIED | `scan_file` at line 340, `token_present_with_boundary` at line 194, both `pub(crate)`. |
| `src/vulnerability/mod.rs` | Re-export `run_ast_scanner` under `#[cfg(feature = "internal")]` | VERIFIED | Lines 18-19 confirmed. |
| `src/main.rs` | Scanner dispatch uses `run_ast_scanner` as primary | VERIFIED | Lines 245-260 confirmed. Phase 18 comment present. |
| `tests/vulnerability_tests/ast_scanner_tests.rs` | 6 passing tests (5 de-ignored + 1 smoke), 0 ignored | VERIFIED | 6 passed, 0 failed, 0 ignored confirmed by test run. |
| `tests/benchmark.rs` | Two benchmark tests + smoke test, no `#[ignore]` marks, graceful-skip logic | VERIFIED | 0 `#[ignore]` markers. 3/3 tests pass. Fully implemented with `run_one_fixture`, `write_benchmark_md`, etc. |
| `tests/vulnerability_tests/mod.rs` | `#[cfg(feature = "internal")] mod ast_scanner_tests;` registered | VERIFIED | Line 17 confirmed. |
| `docs/BENCHMARK.md` | Template with D-14 column headers and HTML-comment markers | VERIFIED (template only) | Exists. Has `BENCHMARK-HEADER-START/END` markers, `| CWE ID |` table header. No real fixture data yet. |
| `docs/BENCHMARK_FIXTURES.md` | Env vars, ground-truth format, fixture acquisition guide | VERIFIED | Exists with AUTOSAR layout, Juliet instructions, `.benchmark_truth.tsv` format, curation workflow, marker-layout docs. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `src/main.rs` | `src/vulnerability/ast_scanner.rs::run_ast_scanner` | function call in `#[cfg(feature = "internal")]` block | WIRED | `crate::vulnerability::run_ast_scanner(&component_dirs)` at line 246 |
| `src/vulnerability/ast_scanner.rs::scan_file_ast_or_lexical` | `src/vulnerability/cwe_scanner.rs::scan_file` | lexical fallback call on parse failure / has_error() | WIRED | `lexical_scan_file(path, component_name, component_ecosystem)` called on both `None` and `has_error()` paths |
| `ast_findings` | `deduplicate_sast_findings` | first argument of dedup pipeline | WIRED | `deduplicate_sast_findings(ast_findings, cppcheck_findings)` at main.rs line 260 |
| `tests/benchmark.rs` | `run_ast_scanner` | scanner invocation in `run_one_fixture` | WIRED | `run_ast_scanner(&component_dirs)` at benchmark.rs line 249 |
| `tests/benchmark.rs` | `docs/BENCHMARK.md` | `std::fs::write` via `write_benchmark_md` | WIRED | `Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/BENCHMARK.md")` at line 194 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `src/vulnerability/ast_scanner.rs` | `all_findings: Vec<SastFinding>` | `scan_file_ast_or_lexical()` → `apply_ast_rules()` or `lexical_scan_file()` | Yes — WalkDir over real filesystem, tree-sitter parse of real C source | FLOWING |
| `src/main.rs` | `ast_findings` | `run_ast_scanner(&component_dirs)` which walks real component dirs | Yes — dispatched from real component_dirs populated by scanner context | FLOWING |
| `tests/benchmark.rs` | fixture findings | `run_ast_scanner`, `run_lexical_scanner`, `run_cppcheck_scanner` on fixture dir | Conditional — flows when fixture present, graceful-skips when absent | FLOWING (conditional on fixture) |
| `docs/BENCHMARK.md` | per-CWE comparison table | `write_benchmark_md()` called from `run_one_fixture()` | Not yet — template only, no fixture has been run | STATIC (placeholder) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `cargo build --features internal` exits 0 | `cargo build --features internal` | Finished in 13.61s, exit 0 | PASS |
| `cargo build --no-default-features` exits 0 | `cargo build --no-default-features` | Finished in 6.22s, exit 0 | PASS |
| AST scanner integration tests pass | `cargo test --features internal --tests vulnerability_tests::ast_scanner_tests` | 6 passed, 0 failed, 0 ignored | PASS |
| Legacy PoC unit tests pass | `cargo test --features internal --lib vulnerability::ast_scanner` | 4 passed, 0 failed | PASS |
| Benchmark tests run without panic | `cargo test --features internal --test benchmark` | 3 passed, 0 failed, 0 ignored | PASS |
| Full vulnerability test suite | `cargo test --features internal --tests vulnerability_tests` | 63 passed, 0 failed, 0 ignored | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| AST-01 | 18-02 | sc2sbom with AST scanner as default C/C++ analysis path | SATISFIED | `run_ast_scanner` is primary in main.rs; no cppcheck required for AST findings |
| AST-02 | 18-02 | AST scanner detects all 14 CWEs from v1.0.17 rule set | PARTIALLY SATISFIED | 13 CWEs detected by AST rules; CWE-362, 367, 416, 476 deferred to lexical fallback per D-08. ROADMAP SC #2 explicitly requires detection on AUTOSAR fixture — untested on real fixture. Union coverage claim (AST + lexical) requires human verification with fixture. |
| AST-03 | 18-01, 18-02 | `SastFinding` output compatible with SARIF, markdown, CycloneDX | SATISFIED | `SastSource::Ast` variant emitted; 63 integration tests pass including sarif/dedup/suppression tests |
| AST-04 | 18-02 | Parse failure falls back to lexical scan with warning | SATISFIED | `scan_file_ast_or_lexical` handles `None` and `has_error()` both; `test_parse_failure_fallback` passes |
| BENCH-01 | 18-03 | AST scanner benchmarked against cppcheck and lexical on fixtures | PARTIALLY SATISFIED | Benchmark infrastructure fully implemented and compiles. No fixture has been run; BENCHMARK.md contains only the placeholder template. "Benchmarked" requires running against real fixtures. |
| DIST-01 | 18-01 | tree-sitter-c license verified MIT-compatible, documented | SATISFIED | Cargo.toml line 39: MIT-licensed comment confirmed |
| DIST-02 | 18-02 | Grammar embedded in binary; no runtime filesystem dependency | SATISFIED | tree-sitter-c's `build.rs` compiles `parser.c` into static archive. `cargo build --features internal` and `--no-default-features` both exit 0. RESEARCH.md documents this pattern. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `docs/BENCHMARK.md` | 21 | `(placeholder) | — | — | — | ...` — placeholder row with no real data | Info | Does not block code functionality; expected until fixture is run. The benchmark infrastructure is wired correctly to replace this on first fixture run. |
| `src/vulnerability/cwe_scanner.rs` (pre-existing) | 419 | `run_lexical_scanner` unused warning | Info | Pre-existing warning noted in SUMMARYs. Not introduced by this phase. Intentionally retained as a public export for benchmark use. |

No TODO/FIXME/PLACEHOLDER comments found in the new scanner code. The `unimplemented!()` stubs in Plan 01 scaffolds have all been removed by Plan 02.

### Human Verification Required

#### 1. Run benchmark with real AUTOSAR fixture

**Test:** Stage `AUTOSAR_SampleProject_S32K144` at `../AUTOSAR_SampleProject_S32K144` (or set `AUTOSAR_FIXTURE_PATH`). Run: `cargo test --features internal --test benchmark -- --nocapture`.

**Expected:** `docs/BENCHMARK.md` is updated with a `<!-- FIXTURE-SECTION-START: AUTOSAR_SampleProject_S32K144 -->` section containing per-CWE rows. All three scanners (AST, lexical, cppcheck if available) produce finding counts. The section confirms the benchmark infrastructure works end-to-end on real C/C++ code.

**Why human:** The AUTOSAR fixture is not committed to the repository. ROADMAP SC #4 explicitly requires "documented comparison on AUTOSAR_SampleProject_S32K144 and at least one additional fixture." Without a fixture run, SC #4 is unverifiable by static analysis.

#### 2. Verify 14-CWE union coverage on AUTOSAR fixture

**Test:** After running the benchmark (test 1 above), inspect the BENCHMARK.md output or run the AST and lexical scanners separately over the AUTOSAR fixture. Verify that findings cover all 14 ROADMAP CWEs — specifically that CWE-362, CWE-367, CWE-416, CWE-476 appear in lexical scanner output on the fixture (these are excluded from AST_CWE_RULES per D-08 but must be present via lexical fallback for ROADMAP SC #2).

**Expected:** Union of `run_ast_scanner` + `run_lexical_scanner` on the AUTOSAR fixture produces at least one finding for each of CWE-78, 119, 120, 122, 125, 134, 190, 295, 319, 362, 367, 369, 416, 476, 732. CWE-367 and CWE-362 come from lexical scanner's existing rules (`access`/`stat` rules in CWE_RULES table).

**Why human:** The D-08 deferral is a code-level decision that can only be verified against real C/C++ code that actually uses the relevant call patterns. Synthetic tests cover the AST-detected 13 CWEs; the deferred 4 CWEs are only verifiable against a fixture containing the relevant call sites.

### Gaps Summary

No blocking gaps in the core implementation. All must-have artifacts exist, are substantive, and are wired. The phase goal is architecturally achieved:

- Production AST scanner (`run_ast_scanner`) is implemented, tested (6/6 integration tests pass), and wired as primary in main.rs.
- 13 tractable CWEs covered via AST rules with per-argument precision.
- Lexical fallback on parse failure or `has_error()` is implemented and tested.
- Benchmark infrastructure (tests/benchmark.rs, docs/BENCHMARK.md template, docs/BENCHMARK_FIXTURES.md) is complete and functional.
- All distribution requirements (DIST-01, DIST-02) are satisfied.

The two human verification items concern ROADMAP success criteria #2 and #4, which both require running against the AUTOSAR_SampleProject_S32K144 fixture. These are not implementation gaps — they are execution gaps (the fixture must be staged and the benchmark run). The infrastructure is correctly implemented and will produce the required output once the developer runs it.

---

_Verified: 2026-05-12T10:00:00Z_
_Verifier: Claude (gsd-verifier)_
