---
title: v1.0.16 C/C++ Lexical CWE Scanner
trigger_when: v1.0.15 ships (Phase 10 complete)
planted_during: v1.0.15 milestone planning
---

# SEED-001: v1.0.16 — Built-in Rust Lexical CWE Scanner for C/C++

## Idea

Add a pure-Rust lexical scanner that detects CWE IDs directly from C/C++ source code — no external tool dependencies, gated behind `cargo feature = "internal"`.

## Confirmed Scope

**13 CWEs via function-name/token matching:**

| CWE | Description | FP Risk |
|-----|-------------|---------|
| CWE-242 | Inherently dangerous function (`gets`) | Near-zero |
| CWE-120 | Classic buffer overflow (`strcpy`, `sprintf`, 40+ variants) | Low |
| CWE-78 | OS command injection (`system`, `popen`, `exec*`) | Low |
| CWE-327 | Weak crypto/PRNG (`rand`, `crypt`, `EVP_des_*`) | Low |
| CWE-377 | Insecure temp file (`tmpnam`, `mktemp`) | Low |
| CWE-190 | Integer overflow (`atoi`, `atol`, `atoll`) | Low-Med |
| CWE-134 | Format string (non-literal format arg) | Med |
| CWE-22 | Path traversal (`chroot`, `getwd`) | Low |
| CWE-807 | Untrusted env variable (`getenv`, `cuserid`) | Med |
| CWE-362/367 | TOCTOU race condition (`access`, `open`, `rename`) | Med |
| CWE-20 | Improper input validation (`recv`, `scanf`) | Med |
| CWE-126 | Buffer over-read (`strlen`, `wcslen`) | Med |
| CWE-676 | Potentially dangerous function (`getpass`, `memalign`) | Low |

**Do NOT implement lexically (require dataflow):** CWE-401, CWE-415, CWE-416, CWE-476

## Key Decisions

- Gate behind `cargo feature = "internal"` (same as CVE/CWE enrichment)
- Scan scope: component-mapped C/C++ directories only (not full source tree)
- Output: CycloneDX 1.5 `vulnerabilities[]` with `source.name = "radeis_sc2sbom static analysis"`, `analysis.state: "in_triage"`
- SPDX 2.3: no native model — CycloneDX output only
- CLI must note: "Pattern-based — complex data-flow vulnerabilities not covered"
- MISRA/AUTOSAR angle: flag banned functions as CWE-676 compliance violations

## Research

All research pre-completed in `.planning/research/v1.0.16/`:
- `STACK.md` — tool options evaluated
- `FEATURES.md` — CycloneDX modeling, table stakes vs differentiators
- `PITFALLS.md` — top risks and prevention
- `LEXICAL_CWES.md` — full CWE table with trigger patterns and FP risk
- `SUMMARY.md` — synthesized recommendation

## Open Questions (resolve at milestone start)

1. Scan scope: component-mapped dirs only vs. full source tree?
2. xZETA behavior: does it treat all `vulnerabilities[]` entries as CVEs regardless of `source.name`?
3. Benchmark false positive rate against xcar-linux before finalizing rule set

## Why This Matters

Closes the gap where a package has a weakness but no published CVE — the CVE→CWE chain in v1.0.15 can't detect those. Enables ISO 26262 / UN R155 compliance reporting without requiring a separate SAST tool.
