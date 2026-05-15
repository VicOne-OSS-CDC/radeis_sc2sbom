---
phase: 20
slug: argument-value-ast-migration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-12
---

# Phase 20 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml — `[features] internal` |
| **Quick run command** | `cargo test --features internal -p radeis_sc2sbom 2>&1 | tail -20` |
| **Full suite command** | `cargo test --features internal 2>&1 | tail -40` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal -p radeis_sc2sbom 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --features internal 2>&1 | tail -40`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 20-01-01 | 01 | 1 | ARGVAL-01 | — | ArgAtIndex variant added to ArgCheck enum | unit | `cargo test --features internal 2>&1 \| grep argcheck` | ❌ W0 | ⬜ pending |
| 20-01-02 | 01 | 1 | ARGVAL-01 | — | CWE-295 SSL_VERIFY_NONE detected via ArgAtIndex | unit | `cargo test --features internal 2>&1 \| grep cwe295` | ❌ W0 | ⬜ pending |
| 20-01-03 | 01 | 1 | ARGVAL-01 | — | CWE-319 CURLOPT rules detected via ArgAtIndex | unit | `cargo test --features internal 2>&1 \| grep cwe319` | ❌ W0 | ⬜ pending |
| 20-01-04 | 01 | 1 | ARGVAL-01 | — | CWE-732 umask(0) detected, umask(0077) does not fire | unit | `cargo test --features internal 2>&1 \| grep cwe732` | ❌ W0 | ⬜ pending |
| 20-01-05 | 01 | 1 | ARGVAL-02 | — | Nested-expression arg detected (success criterion 3) | unit | `cargo test --features internal 2>&1 \| grep nested` | ❌ W0 | ⬜ pending |
| 20-02-01 | 02 | 2 | ARGVAL-01 | — | arg_value_contains rules removed from cwe_scanner.rs | unit | `cargo test --features internal 2>&1 \| tail -5` | ✅ | ⬜ pending |
| 20-02-02 | 02 | 2 | ARGVAL-02 | — | Full suite green after lexical cleanup | unit | `cargo test --features internal 2>&1 \| tail -5` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/fixtures/cwe295_ssl_verify.c` — TP + FP guard + nested expression fixture
- [ ] `tests/fixtures/cwe319_curl_ssl.c` — TP + FP guard fixture
- [ ] `tests/fixtures/cwe732_umask.c` — TP (umask(0)), FP guard (umask(0077)), nested expression
- [ ] `tests/fixtures/cwe732_dacl.c` — TP (SetSecurityDescriptorDacl NULL arg) fixture
- [ ] Test stubs in `src/vulnerability/ast_scanner.rs` (or dedicated `tests/scanner_tests/`) using `apply_ast_rules` pattern

*Existing `cargo test` infrastructure covers framework needs — no new test runner installation required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| AUTOSAR_SampleProject_S32K144 finding count matches v1.0.17 baseline | ARGVAL-02 | Requires full fixture scan on real project | Run `cargo run --features internal -- scan path/to/AUTOSAR_SampleProject_S32K144` and compare SARIF output finding counts for CWE-295/319/732 against v1.0.17 baseline |
| Zero new false positives vs v1.0.17 SARIF baseline | ARGVAL-02 | Requires diff of SARIF outputs | Diff SARIF findings arrays for CWE-295/319/732 between new run and stored v1.0.17 baseline |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
