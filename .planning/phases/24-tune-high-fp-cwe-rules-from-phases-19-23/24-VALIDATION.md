---
phase: 24
slug: tune-high-fp-cwe-rules-from-phases-19-23
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-13
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (features = ["internal"]) |
| **Quick run command** | `cargo test --features internal 2>&1 \| tail -20` |
| **Full suite command** | `cargo test --features internal 2>&1` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --features internal 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test --features internal 2>&1`
- **Before `/gsd-verify-work`:** Full suite must be green + oracle.sh run
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 24-01-01 | 01 | 1 | D-03 | — | CWE-256 removed from output | unit | `cargo test --features internal -- cwe_256 2>&1` | ✅ | ⬜ pending |
| 24-01-02 | 01 | 1 | D-04 | — | CWE-338 fires only on drand48/lrand48/random/mrand48 | unit | `cargo test --features internal -- cwe_338 2>&1` | ✅ | ⬜ pending |
| 24-01-03 | 01 | 1 | D-05 | — | CWE-676 fires only on strtok | unit | `cargo test --features internal -- cwe_676 2>&1` | ✅ | ⬜ pending |
| 24-01-04 | 01 | 1 | D-06 | — | CWE-426 fires on dlopen/LoadLibraryExA/LoadLibraryExW | unit | `cargo test --features internal -- cwe_426 2>&1` | ✅ | ⬜ pending |
| 24-01-05 | 01 | 1 | D-07 | — | CWE-780 fires only on CryptEncrypt ArgAtIndex(3,"0") | unit | `cargo test --features internal -- cwe_780 2>&1` | ✅ | ⬜ pending |
| 24-02-01 | 02 | 1 | D-09 | — | CWE-126 fires on fixed-size buf without sizeof size arg | unit | `cargo test --features internal -- cwe_126 2>&1` | ✅ | ⬜ pending |
| 24-02-02 | 02 | 1 | D-10 | — | CWE-680 fires only on malloc(n * sizeof(T)) | unit | `cargo test --features internal -- cwe_680 2>&1` | ✅ | ⬜ pending |
| 24-02-03 | 02 | 1 | D-11 | — | CWE-467 fires only when sizeof operand is pointer type | unit | `cargo test --features internal -- cwe_467 2>&1` | ✅ | ⬜ pending |
| 24-02-04 | 02 | 1 | D-17 | — | CWE-535 fires only on fprintf(stderr, non_literal_fmt) | unit | `cargo test --features internal -- cwe_535 2>&1` | ✅ | ⬜ pending |
| 24-03-01 | 03 | 2 | D-12 | — | CWE-480 fires only on func-ptr null compares | unit | `cargo test --features internal -- cwe_480 2>&1` | ✅ | ⬜ pending |
| 24-03-02 | 03 | 2 | D-13 | — | CWE-483 does not fire on return/break/continue bodies | unit | `cargo test --features internal -- cwe_483 2>&1` | ✅ | ⬜ pending |
| 24-03-03 | 03 | 2 | D-14 | — | CWE-562 does not fire on array/struct local returns | unit | `cargo test --features internal -- cwe_562 2>&1` | ✅ | ⬜ pending |
| 24-03-04 | 03 | 2 | D-15/D-16 | — | CWE-570/571 do not fire in loop conditions | unit | `cargo test --features internal -- cwe_57 2>&1` | ✅ | ⬜ pending |
| 24-03-05 | 03 | 2 | D-18 | — | CWE-587 investigate root cause + guard applied | unit | `cargo test --features internal -- cwe_587 2>&1` | ✅ | ⬜ pending |
| 24-03-06 | 03 | 2 | D-19 | — | CWE-478 does not fire on ≤2-case switches | unit | `cargo test --features internal -- cwe_478 2>&1` | ✅ | ⬜ pending |
| 24-03-07 | 03 | 2 | D-20 | — | CWE-762 fires only when C-alloc also in file | unit | `cargo test --features internal -- cwe_762 2>&1` | ✅ | ⬜ pending |
| 24-04-01 | 04 | 3 | D-25/D-26 | — | ANALYSIS.md regenerated with Phase 24 Notes | manual | oracle.sh run + human review | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.* The `#[cfg(test)]` blocks with `run_ast_scanner()` and inline C strings are already established in `ast_scanner.rs`. No new test infrastructure needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Juliet oracle delta review | D-23/D-24 | Requires human judgment on TP/FP counts | Run `oracle.sh`; compare Phase 24 table against Phase 23 baseline in ANALYSIS.md |
| AUTOSAR regression | D-23 | Requires running scanner on AUTOSAR_SampleProject_S32K144 | Run scanner on AUTOSAR fixture; confirm no new findings on pre-existing CWEs |
| Residual FP% >35% review | D-24 | Decision on demote/accept requires human | Document any remaining >35% CWEs as human-review items in ANALYSIS.md Phase 24 Notes |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
