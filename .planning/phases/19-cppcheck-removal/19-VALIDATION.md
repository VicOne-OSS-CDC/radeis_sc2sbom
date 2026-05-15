---
phase: 19
slug: cppcheck-removal
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-12
---

# Phase 19 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo check --features internal` |
| **Full suite command** | `cargo test --features internal` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check --features internal`
- **After every plan wave:** Run `cargo test --features internal`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 19-01-01 | 01 | 1 | CPP-01 | — | N/A | compile | `cargo check --features internal` | ✅ | ⬜ pending |
| 19-01-02 | 01 | 1 | CPP-01 | — | N/A | compile | `cargo check --features internal` | ✅ | ⬜ pending |
| 19-01-03 | 01 | 1 | CPP-01 | — | N/A | compile | `cargo check --features internal` | ✅ | ⬜ pending |
| 19-02-01 | 02 | 2 | CPP-01 | — | N/A | test | `cargo test --features internal` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
