---
title: v1.0.17 cppcheck Integration & Argument-Value CWE Matching
trigger_when: v1.0.16 ships
planted_during: v1.0.16 milestone — curl CVE analysis session (2026-05-10)
---

# SEED-002: v1.0.17 — cppcheck Integration & Argument-Value CWE Matching

## Idea

Extend the internal C/C++ scanner beyond pure function-name lexical matching in two ways:

1. **cppcheck 2.16+ subprocess** — optional external analyzer invoked when `cppcheck` is on PATH; produces ~48 CWEs with dataflow backing, covering classes our lexical scanner cannot detect
2. **Argument-value matching** — extend the Rust rule engine to inspect specific argument values at call sites, enabling detection of TLS/crypto API misuse (CWE-295, CWE-319) without requiring cppcheck

Both are gated behind `--features internal`. cppcheck must be optional with graceful degradation (scanner continues if binary not found).

## Why This Matters

curl 8.19.0 ships 8 CVEs — none detectable by v1.0.16's function-name scanner. The root cause: all 8 are protocol logic or API-misuse vulnerabilities, not dangerous-function call sites. The two highest-value targets for v1.0.17:

| CWE | curl CVE | Pattern |
|-----|----------|---------|
| CWE-295 | CVE-2026-7009 | `SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL)` — cert validation disabled |
| CWE-319 | CVE-2026-4873 | `curl_easy_setopt(h, CURLOPT_USE_SSL, CURLUSESSL_NONE)` — TLS disabled |

These require inspecting the *value of a specific argument*, not just the function name — a capability v1.0.16 does not have.

## Scope

### Track A: cppcheck subprocess (broader coverage)

- Invoke `cppcheck --xml --enable=warning,style,security` on component-mapped C/C++ dirs
- Parse XML output → map cppcheck IDs to CWE IDs → emit as `SastFinding` entries (same struct as v1.0.16)
- Graceful degradation: if `cppcheck` not found → log warning, continue with lexical-only results
- Optional `--cppcheck-path` flag to specify binary location
- Estimated CWE coverage: ~48 CWEs including CWE-295, CWE-319, CWE-401, CWE-415, CWE-416, CWE-476 (dataflow-backed)

### Track B: argument-value matching in Rust rule engine (zero external dep)

Extend `CweRule` struct with optional `arg_value_contains: Option<&'static [&'static str]>` field. When set, a match fires only when the call-site argument list contains one of the specified tokens (constant name or string fragment).

New rules enabled by this:

| CWE | Function | Trigger arg value |
|-----|----------|------------------|
| CWE-295 | `SSL_CTX_set_verify` | `SSL_VERIFY_NONE` |
| CWE-295 | `SSL_set_verify` | `SSL_VERIFY_NONE` |
| CWE-295 | `wolfSSL_CTX_set_verify` | `SSL_VERIFY_NONE` |
| CWE-319 | `curl_easy_setopt` | `CURLOPT_USE_SSL` + `CURLUSESSL_NONE` |
| CWE-319 | `curl_easy_setopt` | `CURLOPT_SSL_VERIFYPEER` (value = 0) |
| CWE-319 | `curl_easy_setopt` | `CURLOPT_SSL_VERIFYHOST` (value = 0 or 1) |

Implementation: after matching the function name, scan the full argument list string (from `(` to matching `)`) for the trigger token(s) using the existing paren-bound extractor.

### Additional "Maybe" CWEs from v1.0.16 research to revisit

| CWE | Pattern | Note |
|-----|---------|------|
| CWE-732 | `umask`, `SetSecurityDescriptorDacl` | Linux/IVI targets |
| CWE-369 | literal `/0` or `%0` in expression | Trivial constant case only |
| CWE-590 | `free(&local_var)` | Detectable when `&` is visible |
| CWE-785 | `realpath(path, buf)` vs `realpath(path, NULL)` | Buffer-arg heuristic |

## Key Decisions to Make at Milestone Start

1. Ship Track A and Track B together, or pick one?
   - Recommendation: Track B first (zero dep, highest signal for TLS misuse) then Track A as a separate phase
2. How to merge cppcheck findings with lexical findings — dedup by file:line:cwe?
3. CI impact: cppcheck adds ~30s per component dir on large repos — acceptable for internal builds?
4. False positive handling: cppcheck will produce more findings than lexical scanner — suppress-list support needed?

## Relationship to v1.0.16

- v1.0.16 ships: `SastFinding` struct, `run_lexical_scanner`, CycloneDX serialization, `_static_analysis.md` report, `--features internal` gate, `resolve_component_dir` (component-mapping fix)
- v1.0.17 builds on all of it — new findings flow through the same pipeline unchanged

## Research Pre-completed in v1.0.16

- `.planning/research/v1.0.16/SUMMARY.md` — cppcheck evaluated and deferred to v1.0.17
- `.planning/research/v1.0.16/LEXICAL_CWES.md` — argument-value matching CWEs documented (rows 14–18, "Maybe" column)
- `.planning/research/v1.0.16/STACK.md` — tool comparison including cppcheck vs Semgrep vs clang-tidy

## Open Questions

1. Does cppcheck 2.16+ reliably detect CWE-295/319 in practice? Validate against curl fixtures before committing to Track A.
2. Argument-value matching: does scanning the full paren-bound string for a token have acceptable FP rate for `curl_easy_setopt`? The option constant and value are often on separate lines.
3. SARIF output — deferred from v1.0.16, natural fit here alongside cppcheck which natively produces SARIF.
