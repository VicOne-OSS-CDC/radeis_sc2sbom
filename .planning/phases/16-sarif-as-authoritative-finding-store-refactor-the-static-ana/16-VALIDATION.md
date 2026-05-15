---
phase: 16
slug: sarif-as-authoritative-finding-store
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-11
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test 2>&1` |
| **Full suite command** | `cargo test -- --nocapture 2>&1` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test 2>&1`
- **After every plan wave:** Run `cargo test -- --nocapture 2>&1`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 16-01-01 | 01 | 1 | SARIF-04 | — | Fingerprint collision resistance | unit | `cargo test fingerprint` | ✅ W0 | ⬜ pending |
| 16-01-02 | 01 | 1 | SARIF-06 | — | SARIF and markdown finding counts match | unit | `cargo test consistency` | ✅ W0 | ⬜ pending |
| 16-02-01 | 02 | 1 | SARIF-07 | — | Lexical suppression leaves cppcheck findings intact | unit | `cargo test suppress` | ✅ W0 | ⬜ pending |
| 16-03-01 | 03 | 2 | SARIF-05 | — | Baseline diff exits 1 on new findings | unit | `cargo test baseline` | ✅ W0 | ⬜ pending |
| 16-03-02 | 03 | 2 | SARIF-05 | — | Baseline diff exits 0 when no new findings | unit | `cargo test baseline` | ✅ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/sarif.rs` — fingerprint unit test stubs
- [ ] `src/sast_scanner.rs` (or equivalent) — suppression unit test stubs
- [ ] `src/main.rs` (or integration test) — baseline diff exit code stubs

*Existing cargo test infrastructure covers the framework — no new install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `--sarif-baseline` with missing file gracefully continues | SARIF-05 | Requires CLI invocation with invalid path | Run `cargo run -- --sarif-output /tmp/out.sarif --sarif-baseline /nonexistent/baseline.sarif` on a real sbom; verify stderr message and exit 0 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
