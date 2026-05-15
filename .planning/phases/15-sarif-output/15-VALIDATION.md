---
phase: 15
slug: sarif-output
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust built-in) |
| **Config file** | `Cargo.toml` |
| **Quick run command** | `cargo test --features internal sarif` |
| **Full suite command** | `cargo test --features internal` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal sarif`
- **After every plan wave:** Run `cargo test --features internal`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 1 | SARIF-01 | — | N/A | unit | `cargo test --features internal sarif::tests` | ✅ W0 | ⬜ pending |
| 15-01-02 | 01 | 1 | SARIF-01 | — | N/A | unit | `cargo test --features internal sarif::tests` | ✅ W0 | ⬜ pending |
| 15-02-01 | 02 | 2 | SARIF-02 | — | N/A | unit | `cargo test --features internal sarif::tests` | ✅ W0 | ⬜ pending |
| 15-03-01 | 03 | 3 | SARIF-03 | — | N/A | integration | `cargo test --features internal` | ✅ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/formats/sarif.rs` — stub module with empty `save_sarif_report` and SARIF structs
- [ ] `src/formats/sarif.rs` — `#[cfg(test)] mod tests` with placeholder unit tests for each requirement

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SARIF file validates against SARIF 2.1 JSON schema | SARIF-01 | No JSON schema validator in CI | Run `npx @microsoft/sarif-multitool validate output.sarif` or upload to SARIF viewer |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
