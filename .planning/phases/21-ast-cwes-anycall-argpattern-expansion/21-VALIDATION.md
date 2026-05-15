---
phase: 21
slug: ast-cwes-anycall-argpattern-expansion
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-12
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | Cargo.toml (feature = "internal") |
| **Quick run command** | `cargo test --features internal -- ast` |
| **Full suite command** | `cargo test --features internal` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal -- ast`
- **After every plan wave:** Run `cargo test --features internal`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 1 | CWEXP-01 | — | ArgCheck::SizeofPointer detects sizeof(ptr) | unit | `cargo test --features internal -- sizeof_pointer` | ❌ W0 | ⬜ pending |
| 21-01-02 | 01 | 1 | CWEXP-01 | — | apply_division_rules() fires on x/0 literal | unit | `cargo test --features internal -- division_rules` | ❌ W0 | ⬜ pending |
| 21-02-01 | 02 | 2 | CWEXP-01 | — | Each new AnyCall CWE produces ≥1 TP on Juliet/synthetic | integration | `cargo test --features internal -- juliet_regression` | ❌ W0 | ⬜ pending |
| 21-02-02 | 02 | 2 | CWEXP-01 | — | No regression on existing 13 CWEs | integration | `cargo test --features internal -- ast_regression` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/ast_regression.rs` — stubs for CWEXP-01 CWE regression tests (behind `#[cfg(feature = "internal")]`)
- [ ] Synthetic fixture C files for CWEs not in Juliet: CWE-369 (literal /0), CWE-676 (alloca/strtok), CWE-780 (RSA_public_encrypt), CWE-526 (getenv), CWE-535 (fprintf stderr)

*Existing `cargo test` infrastructure covers Rust unit tests; Wave 0 adds the regression test file and synthetic fixtures.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ANALYSIS.md updated with per-CWE TP/FP rows | CWEXP-01 | Requires running sc2sbom against full Juliet corpus (~minutes) | Run `cargo run --features internal -- --sast <juliet_dir> --format sarif` and tally per-CWE results |
| FP% ≤35% per new CWE | CWEXP-01 | File-level oracle requires human review of false-positive directories | Review benchmark/juliet/ast.json findings against Juliet directory CWE family |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
