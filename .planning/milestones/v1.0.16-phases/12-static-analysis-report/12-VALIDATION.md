---
phase: 12
slug: static-analysis-report
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-09
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `Cargo.toml` — no extra config |
| **Quick run command** | `cargo test --features internal 2>&1 | tail -20` |
| **Full suite command** | `cargo test --features internal 2>&1` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --features internal 2>&1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | RPT-01,02,03 | — | N/A | compile (RED) | `cargo build --features internal --tests 2>&1 \| grep -E "cannot find function .save_static_analysis_report." >/dev/null && echo RED_OK` | ✅ | ⬜ pending |
| 12-01-02 | 01 | 1 | RPT-02,03 | — | N/A | compile (RED) | `cargo build --features internal --tests 2>&1` (expected RED errors only) | ✅ | ⬜ pending |
| 12-02-01 | 02 | 2 | RPT-01 | — | N/A | unit (GREEN) | `cargo test --features internal -- test_save_static_analysis_report 2>&1` | ✅ | ⬜ pending |
| 12-02-01b | 02 | 2 | RPT-03 | — | N/A | grep (source) | `grep -c "Pattern-based — complex data-flow vulnerabilities not covered" src/formats/console.rs` >= 2 (acceptance_criteria of plan 12-02 Task 1, covers eprintln disclaimer) | ✅ | ⬜ pending |
| 12-02-02 | 02 | 2 | RPT-01 | — | N/A | compile + smoke | `cargo build && cargo build --features internal` both clean | ✅ | ⬜ pending |
| 12-03-01 | 03 | 3 | RPT-02 | — | N/A | compile + regression | `cargo build --features internal && cargo test --features internal -- test_save_static_analysis_report_with_findings test_save_static_analysis_report_zero_findings test_save_static_analysis_report_writes_correct_filename` (3 RPT-01 tests stay green) | ✅ | ⬜ pending |
| 12-03-02 | 03 | 3 | RPT-02 | — | N/A | unit (GREEN) | `cargo test --features internal -- sast_report_tests 2>&1 \| grep -E "test result: ok"` (5 active tests pass) | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Sampling continuity audit:** No 3 consecutive implementation tasks lack a test run.
- 12-01-01 (RED compile gate) → 12-01-02 (RED compile gate) → 12-02-01 (test run) ✅
- 12-02-01 (test run) → 12-02-01b (grep) → 12-02-02 (compile) → 12-03-01 (test run, regression) ✅
- 12-03-01 (test run) → 12-03-02 (test run) ✅

RPT-03 (stderr disclaimer) verification:
- Implementation: plan 12-02 Task 1 emits `eprintln!("Pattern-based — complex data-flow vulnerabilities not covered")` inside `save_static_analysis_report`.
- Automated check: plan 12-02 Task 1 acceptance_criteria includes `grep -c "Pattern-based — complex data-flow vulnerabilities not covered" src/formats/console.rs >= 2` (one in blockquote writeln, one in eprintln).
- Manual check: smoke test in plan 12-02 verification block runs the binary and greps stderr.

---

## Wave 0 Requirements

- [ ] Read `src/vulnerability/cwe_scanner.rs` — confirm `SastFinding` struct field names before implementing the formatter (handled inside plan 12-01 Task 1 `<read_first>` and plan 12-02 Task 1 `<read_first>`)

*Wave 0 is folded into the first read_first of plan 12-01; Rust compilation serves as the primary automated gate.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `_static_analysis.md` file written to `--output-dir` alongside other scan outputs | RPT-01 | File I/O integration test | Run `cargo run --features internal -- scan <sample-project> --output-dir /tmp/out` and verify `/tmp/out/{project}_static_analysis.md` exists |
| `_report.md` contains "Static Analysis Findings" section after CVE section | RPT-02 | Integration test with real output | Open `/tmp/out/{project}_report.md`, confirm section present and positioned after CVE section |
| Disclaimer printed to stderr when scanner runs | RPT-03 | stderr capture in real binary | Run scan and confirm stderr contains `Pattern-based — complex data-flow vulnerabilities not covered` |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ready
