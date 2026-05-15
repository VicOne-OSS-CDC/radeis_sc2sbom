---
phase: 23
slug: ast-cwes-domainspecific-expansion
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-12
---

# Phase 23 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (features = ["internal"]) |
| **Quick run command** | `cargo test --features internal ast_scanner -- --nocapture 2>&1 | tail -20` |
| **Full suite command** | `cargo test --features internal 2>&1 | tail -30` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal ast_scanner -- --nocapture 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --features internal 2>&1 | tail -30`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 23-01-01 | 01 | 1 | CWEXP-03 | — | CWE-114/272/284/427/785 AstCweRule entries in AST_CWE_RULES produce ≥1 TP on Juliet bad files | unit | `cargo test --features internal cwe_114 cwe_272 cwe_284 cwe_427 cwe_785` | ✅ | ⬜ pending |
| 23-01-02 | 01 | 1 | CWEXP-03 | — | CWE-479 two-pass via apply_signal_handler_rules fires on signal()+malloc/free pattern | unit | `cargo test --features internal cwe_479` | ✅ | ⬜ pending |
| 23-01-03 | 01 | 1 | CWEXP-03 | — | CWE-591 apply_paired_lock_rules fires on VirtualAlloc without VirtualLock | unit | `cargo test --features internal cwe_591` | ✅ | ⬜ pending |
| 23-01-04 | 01 | 1 | CWEXP-03 | — | CWE-762 raw-text scan fires on calloc+delete / new+free pattern in synthetic fixture | unit | `cargo test --features internal cwe_762` | ✅ | ⬜ pending |
| 23-02-01 | 02 | 2 | CWEXP-03 | — | No regression on existing 41 CWEs (full Juliet run TP/FP unchanged) | integration | `cargo test --features internal 2>&1 | tail -30` | ✅ | ⬜ pending |
| 23-02-02 | 02 | 2 | CWEXP-03 | — | ANALYSIS.md updated with 49-CWE coverage table | manual | inspect `benchmark/juliet/ANALYSIS.md` for 8 new CWE rows | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements — `cargo test --features internal` framework already present.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ANALYSIS.md 49-CWE table | CWEXP-03 | File update requires running sc2sbom against Juliet corpus and inspecting output | Run `cargo run --features internal -- scan example_target_repos/juliet-test-suite-c/testcases/` and verify benchmark/juliet/ANALYSIS.md updated |
| FP% ≤40% per new CWE | CWEXP-03 | Requires per-file oracle comparison against Juliet good/bad labels | Count TP/FP per CWE from ast.json output; verify FP% ≤40% |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
