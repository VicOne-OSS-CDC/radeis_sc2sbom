---
phase: 13
slug: argument-value-matching
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-10
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test -p radeis_sc2sbom vulnerability::cwe_scanner` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p radeis_sc2sbom vulnerability::cwe_scanner`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | ARGVAL-01 | — | SSL_VERIFY_NONE in SSL_CTX_set_verify arg produces CWE-295 finding | unit | `cargo test cwe_scanner::test_ssl_verify_none` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | ARGVAL-02 | — | CURLOPT_SSL_VERIFYPEER=0 in curl_easy_setopt arg produces CWE-319 finding | unit | `cargo test cwe_scanner::test_curl_ssl_verifypeer` | ❌ W0 | ⬜ pending |
| 13-01-03 | 01 | 1 | ARGVAL-03 | — | umask(0) produces CWE-732 finding | unit | `cargo test cwe_scanner::test_umask_zero` | ❌ W0 | ⬜ pending |
| 13-01-04 | 01 | 1 | ARGVAL-04 | — | division by literal 0 produces CWE-369 finding | unit | `cargo test cwe_scanner::test_div_by_zero` | ❌ W0 | ⬜ pending |
| 13-01-05 | 01 | 1 | ARGVAL-05 | — | CweRule with no arg_value_contains still fires on name match alone | unit | `cargo test cwe_scanner::test_name_only_rule_unaffected` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/vulnerability/cwe_scanner.rs` — add test stubs for ARGVAL-01 through ARGVAL-05
- [ ] `test_rule_table_has_fourteen_cwes` — rename to reflect new count of 18 CWE IDs

*Existing cargo test infrastructure covers all phase requirements. No new test framework installation needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CycloneDX JSON output contains CWE-295 finding | ARGVAL-01 | End-to-end output format | Run scanner on test C file, check output JSON for `cwe_id: "CWE-295"` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
