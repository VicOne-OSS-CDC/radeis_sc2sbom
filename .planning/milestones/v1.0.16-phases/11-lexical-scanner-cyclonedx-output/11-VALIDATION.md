---
phase: 11
slug: lexical-scanner-cyclonedx-output
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-09
updated: 2026-05-09
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml (`[features] internal`) |
| **Quick run command** | `cargo test --features internal 2>&1 \| tail -20` |
| **Full suite command** | `cargo test --features internal 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --features internal 2>&1`
- **Before `/gsd-verify-work`:** Full suite must be green (both default and `--features internal`)
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 11-02-01 | 02 | 2 | SCAN-01..SCAN-05 | T-11-02-01..05 | Scanner module compiles + inline unit tests pass | unit | `cargo test --features internal --lib vulnerability::cwe_scanner` | ❌ W0 (inline) | ⬜ pending |
| 11-02-02 | 02 | 2 | SCAN-01..SCAN-05 | — | mod gating preserved across both build variants | build | `cargo build && cargo build --features internal` | ✅ existing | ⬜ pending |
| 11-03-01 | 03 | 3 | CDX-01, CDX-02, CDX-03 | — | CycloneDXVulnerability gains analysis + properties; source.url Option | unit | `cargo test --lib formats::cyclonedx` | ✅ existing (extends) | ⬜ pending |
| 11-03-02 | 03 | 3 | CDX-01..CDX-04 | — | build_sast_vulnerabilities + signature thread-through | unit | `cargo test --features internal --lib formats::cyclonedx` | ✅ existing (extends) | ⬜ pending |
| 11-04-01 | 04 | 4 | SCAN-01..SCAN-05 | — | End-to-end scan against fixture produces expected findings | integration | `cargo test --features internal --test vulnerability_tests cwe_scanner_tests` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

The following test files do not exist today and must be created before they can fail-then-pass. Plan 02 creates inline tests inside the scanner module; Plan 04 creates the integration test file with C fixtures.

- [ ] `src/vulnerability/cwe_scanner.rs` — inline `#[cfg(test)] mod tests { ... }` covering `format_arg_is_literal`, `find_function_call`, and `CWE_RULES` distinct-CWE-ID count (created by Plan 02 Task 1).
- [ ] `tests/vulnerability_tests/cwe_scanner_tests.rs` — integration tests for SCAN-01..SCAN-05 against `run_lexical_scanner` with TempDir-backed `component_dirs` (created by Plan 04 Task 1).
- [ ] `tests/vulnerability_tests/mod.rs` — add `#[cfg(feature = "internal")] mod cwe_scanner_tests;` declaration (Plan 04 Task 1 step 4).
- [ ] `tests/fixtures/c/dangerous_calls.c` — fixture with one call per CWE rule (Plan 04 Task 1 step 1).
- [ ] `tests/fixtures/c/safe_printf.c` — literal-format printf calls; must NOT trigger CWE-134 (Plan 04 Task 1 step 2).
- [ ] CycloneDX test extensions live inline in `src/formats/cyclonedx.rs` (`#[cfg(test)] mod` already present per existing project pattern — Plan 03 extends, not creates).

> Note: Earlier draft listed paths under `src/vulnerability/tests/` and `src/cyclonedx/tests/`. These were incorrect. Plan 02 uses inline `#[cfg(test)] mod tests` inside `src/vulnerability/cwe_scanner.rs`; Plan 03 extends inline tests inside `src/formats/cyclonedx.rs`; Plan 04 creates external integration tests under `tests/vulnerability_tests/` matching the existing project layout (see `tests/vulnerability_tests/nvd_tests.rs`).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| xZETA ingestion of `analysis.state = "in_triage"` SAST entries | CDX-01 | Cannot verify from source (RESEARCH.md Open Question 1, RESOLVED — validate post-ship) | After ship: ingest a CycloneDX output with SAST findings into xZETA, confirm `in_triage` state is preserved and SAST entries are not auto-converted to remediation tickets |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies declared
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (paths corrected to match Plan 02 inline tests + Plan 04 `tests/vulnerability_tests/cwe_scanner_tests.rs`)
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved
