---
phase: 18
slug: ast-scanner-core-and-benchmark
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-11
updated: 2026-05-11
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test --features internal 2>&1 | tail -20` |
| **Full suite command** | `cargo test --features internal && cargo clippy --features internal -- -D warnings` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --features internal && cargo clippy --features internal -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 18-02-01 | 02 | 2 | AST-01, AST-02, AST-03, AST-04 | T-18-02-* | AST scanner emits SastFinding; 13 AST CWEs detected (D-07's 11 + CWE-295 + CWE-319); parse-failure → lexical fallback | unit | `cargo test --features internal --tests vulnerability_tests::ast_scanner_tests` | ✅ (Plan 01 scaffold) | ⬜ pending |
| 18-02-02 | 02 | 2 | AST-01 | T-18-02-05 | main.rs dispatches AST scanner as primary; full pipeline green | integration | `cargo test --features internal --tests` | ✅ | ⬜ pending |
| 18-02-03 | 02 | 2 | DIST-02 | T-18-02-06 | static musl build embeds grammar (no runtime grammar file dep) | integration | `cargo build --release --features internal --target x86_64-unknown-linux-musl` | ✅ (CI workflow) | ⬜ pending |
| 18-03-01 | 03 | 3 | BENCH-01 | T-18-03-* | benchmark runs without panic (fixtures absent: SKIP; fixtures present: writes docs/BENCHMARK.md) | integration | `cargo test --features internal --test benchmark` | ✅ (Plan 01 scaffold) | ⬜ pending |
| 18-03-02 | 03 | 3 | BENCH-01 | — | docs/BENCHMARK.md + docs/BENCHMARK_FIXTURES.md exist with D-14 columns and fixture-marker layout | unit | `grep -F 'BENCHMARK-HEADER-START' docs/BENCHMARK.md && grep -F 'AUTOSAR_FIXTURE_PATH' docs/BENCHMARK_FIXTURES.md` | ✅ (created in Plan 03) | ⬜ pending |
| 18-01-01 | 01 | 1 | DIST-01 | T-18-01-01 | tree-sitter-c MIT license documented in Cargo.toml | unit | `grep -c 'tree-sitter and tree-sitter-c are MIT-licensed' Cargo.toml` | ✅ | ⬜ pending |
| 18-01-02 | 01 | 1 | AST-03 (foundation) | T-18-01-02 | SastSource::Ast variant exists; ast_scanner module gated under `feature = "internal"` | unit | `cargo build --features internal && cargo build --no-default-features` | ✅ | ⬜ pending |
| 18-01-03 | 01 | 1 | AST-02/03/04 (Wave 0 scaffolds) | — | test scaffolds compile and smoke tests pass; AST/benchmark tests reported as ignored | unit | `cargo test --features internal --tests 2>&1 \| grep -E 'test_setup_one_file_helper_smoke\|fixture_helper_returns_none_for_missing_path'` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

(Wave 0 artifacts are produced by Plan 01, Task 3 — `18-01-PLAN.md`.)

- [ ] `tests/vulnerability_tests/ast_scanner_tests.rs` — scaffold with `#[ignore]` stubs for AST-02/03/04 (Plan 02 unignores) and a `test_setup_one_file_helper_smoke` test that compiles immediately
- [ ] `tests/vulnerability_tests/mod.rs` — registers `ast_scanner_tests` submodule under `#[cfg(feature = "internal")]`
- [ ] `tests/benchmark.rs` — scaffold with `#[ignore]` stubs for `benchmark_ast_vs_cppcheck_autosar` / `benchmark_ast_vs_cppcheck_juliet` (Plan 03 unignores) and a `fixture_helper_returns_none_for_missing_path` test that compiles immediately

*Existing cargo test infrastructure covers all phase requirements once these scaffolds are in place. Plan 02 adds `test_ast_emits_sast_finding`, `test_parse_failure_fallback`, `test_ast_all_tractable_cwes`, `test_ast_safe_strcpy_no_finding`, `test_ast_function_scope_isolation` to `tests/vulnerability_tests/ast_scanner_tests.rs`. Plan 03 implements the benchmark bodies in `tests/benchmark.rs`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| BENCHMARK.md TP/FP accuracy on real fixtures | BENCH-01 | Requires human judgment on TP/FP classification per CWE per call site (ground-truth oracle authorship) | Stage AUTOSAR_SampleProject_S32K144 (or set `AUTOSAR_FIXTURE_PATH`) and optionally Juliet subset; author `<fixture>/.benchmark_truth.tsv`; run `cargo test --features internal --test benchmark -- --nocapture`; review `docs/BENCHMARK.md` |
| MIT license attestation review | DIST-01 | Documentation review | Confirm `Cargo.toml` contains the line `# tree-sitter and tree-sitter-c are MIT-licensed ...` near the dep declarations (added in Plan 01 Task 1) |
| ROADMAP/CONTEXT alignment on CWE-367 deferral | AST-02 | Roadmap edit requires user approval | Per Plan 02 deferred-CWE note, recommend updating CONTEXT.md D-08 to add CWE-367 to deferred list, and ROADMAP §Phase 18 success criterion #2 wording to "detected via AST scanner *or* lexical fallback" |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter
- [ ] Wave 0 file paths reconcile with Plan 01 Task 3 artifacts (`tests/vulnerability_tests/ast_scanner_tests.rs`, `tests/vulnerability_tests/mod.rs`, `tests/benchmark.rs`)

**Approval:** pending
