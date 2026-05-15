---
phase: 14
slug: cppcheck-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml — existing test suite |
| **Quick run command** | `cargo test --features internal 2>&1` |
| **Full suite command** | `cargo test --features internal 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal 2>&1`
- **After every plan wave:** Run `cargo test --features internal 2>&1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 0 | CPPCHECK-01 | — | SastFinding struct compiles with new source field | unit | `cargo test --features internal 2>&1` | ✅ | ⬜ pending |
| 14-01-02 | 01 | 0 | CPPCHECK-01 | — | All SastFinding construction sites updated | compile | `cargo build --features internal 2>&1` | ✅ | ⬜ pending |
| 14-02-01 | 02 | 1 | CPPCHECK-02 | — | run_cppcheck_scanner returns findings from XML stderr | unit | `cargo test --features internal cppcheck 2>&1` | ❌ W0 | ⬜ pending |
| 14-02-02 | 02 | 1 | CPPCHECK-03 | — | CWE override table resolves known error IDs | unit | `cargo test --features internal cwe_override 2>&1` | ❌ W0 | ⬜ pending |
| 14-03-01 | 03 | 1 | CPPCHECK-04 | — | Missing cppcheck binary: warning to stderr, exit 0 | unit | `cargo test --features internal cppcheck_not_found 2>&1` | ❌ W0 | ⬜ pending |
| 14-03-02 | 03 | 1 | CPPCHECK-05 | — | --cppcheck-path flag uses specified binary | unit | `cargo test --features internal cppcheck_path_flag 2>&1` | ❌ W0 | ⬜ pending |
| 14-04-01 | 04 | 2 | CPPCHECK-01 | — | (file,line,cwe) dedup removes duplicates, sets SastSource::Both | unit | `cargo test --features internal dedup 2>&1` | ❌ W0 | ⬜ pending |
| 14-05-01 | 05 | 3 | CPPCHECK-01 | — | main.rs integration: cppcheck findings in final sast_findings | integration | `cargo test --features internal integration 2>&1` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/vulnerability_tests/cppcheck_scanner_tests.rs` — stubs for CPPCHECK-01 through CPPCHECK-05
- [ ] `tests/vulnerability_tests/mod.rs` — add cppcheck_scanner_tests module declaration

*Existing test infrastructure (cargo test) covers all phase requirements — no new framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| indicatif progress bar renders during cppcheck scan | CPPCHECK-01 | Terminal rendering cannot be unit-tested | Run `cargo run --features internal -- <sbom-with-c-libs>` and observe spinner |
| stderr completion line printed after all components | CPPCHECK-01 | stderr capture in tests may not match TTY behavior | Run with a multi-component SBOM, check stderr for "cppcheck: N findings from M components" |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
